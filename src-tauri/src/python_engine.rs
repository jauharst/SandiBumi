//! Vectorized Python (numpy) equation engine, tp_evaluate-style: the user script sees
//! every input curve as a float32 numpy array (NaN = missing) plus `depth`, and assigns
//! the output curve name as an array — numpy does the per-sample work.
//!
//! Python runs as a SUBPROCESS rather than an embedded interpreter: the app binary has
//! no link-time Python dependency, so a missing/foreign Python can never stop SandiBumi
//! from launching — a run just fails with a clear message. Discovery order:
//! `SANDIBUMI_PYTHON` env var, then the legacy `ARSHILLA_PYTHON` (the app's former name —
//! honoured silently so nobody's existing setup breaks, but never named in a message), then
//! recent py.org per-user installs, then PATH; the first interpreter that can `import numpy`
//! wins (cached for the session).
//!
//! **scipy is optional (2026-07-31).** The worker binds `scipy` plus `signal`, `interpolate`,
//! `optimize`, `stats` and `ndimage` directly into the script namespace, so despiking
//! (`signal.medfilt`), Savitzky-Golay smoothing (`signal.savgol_filter`), resampling
//! (`interpolate.interp1d`) and curve fitting (`optimize.curve_fit`) are one line in a user
//! equation. **numpy stays the only requirement** — with scipy absent, each name is a stub whose
//! first use raises a message naming the interpreter and the exact pip command, instead of a bare
//! `NameError` that says nothing about what to install or where. Core petrophysics stays in Rust;
//! this is for the user's own equations.
//!
//! **Persistent worker (perf):** rather than spawn a fresh `python.exe` per well (which
//! re-imports numpy every time — the dominant cost of a field-scale equation run), one
//! long-lived worker process runs a request/response loop and is reused for every well and
//! every subsequent run in the session. Each request is a JSON header line + raw f32 input
//! arrays; each response is a 4-byte length + JSON status (`{"ok":true}` / `{"ok":false,
//! "error":...}`) + (on success) the output array's raw f32 bytes. A **script error is
//! reported per request without killing the worker** (fresh namespace per request, so no
//! state leaks between wells); if the worker process dies (broken pipe) it is respawned and
//! the request retried once. The worker exits on its own when the app closes its stdin (EOF).

use crate::equations::{fetch_curve_frame, write_equation_output, EquationDef, EquationRunResult};
use duckdb::Connection;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};

/// The environment variable naming the interpreter to use. Every message the user ever sees
/// names THIS one.
pub const PYTHON_ENV: &str = "SANDIBUMI_PYTHON";
/// The pre-rename name, still honoured so an existing setup keeps working. Deliberately never
/// mentioned in a message: telling a customer to set a variable named after a product that no
/// longer exists is how the old name outlives the rename.
pub const PYTHON_ENV_LEGACY: &str = "ARSHILLA_PYTHON";

const NO_PYTHON: &str =
    "no Python with numpy found — install Python 3.10+ with numpy, or set SANDIBUMI_PYTHON to its python.exe";

/// Persistent request/response loop: read a JSON header line + raw f32 arrays, exec the
/// user script with numpy bound (fresh namespace each request), and reply with a
/// length-prefixed JSON status + the output array's bytes. Loops until stdin hits EOF (the
/// app closed the pipe), so one worker serves every well. A script exception is caught and
/// returned as `{"ok":false}` — the loop keeps going, so one bad script never kills the
/// worker.
const RUNNER_LOOP: &str = r#"
import sys, json
import numpy as np

# scipy is OPTIONAL and must stay that way. numpy is the engine (an equation language without
# arrays is nothing); scipy is a bonus, so an interpreter with only numpy behaves exactly as it
# did before this existed. Submodules are imported EXPLICITLY: `import scipy` alone does not
# bind scipy.signal, so a script writing `signal.savgol_filter(...)` after a bare scipy import
# fails with an AttributeError that says nothing useful.
SCIPY_SUBMODULES = ("signal", "interpolate", "optimize", "stats", "ndimage")

class _MissingScipy:
    """Stands in for a scipy submodule when scipy is absent.

    Without this a script using `signal.medfilt` dies on `NameError: name 'signal' is not
    defined`, which tells the user nothing about what to install or into WHICH interpreter —
    and SandiBumi picks its own interpreter, so "install scipy" is genuinely ambiguous on a
    machine with three Pythons. Naming sys.executable is the whole point.
    """
    __slots__ = ("_name",)

    def __init__(self, name):
        self._name = name

    def __getattr__(self, attr):
        raise RuntimeError(
            "scipy is not installed in the interpreter this app is using ("
            + sys.executable
            + ") - run:  \"" + sys.executable + "\" -m pip install scipy   "
            + "(needed for " + self._name + "." + str(attr) + ")"
        )

    def __call__(self, *a, **k):
        raise RuntimeError(
            "scipy is not installed in the interpreter this app is using ("
            + sys.executable + ") - run:  \"" + sys.executable + "\" -m pip install scipy"
        )

SCIPY_NS = {}
try:
    import scipy as _scipy
    SCIPY_NS["scipy"] = _scipy
except Exception:
    SCIPY_NS["scipy"] = _MissingScipy("scipy")
for _m in SCIPY_SUBMODULES:
    try:
        SCIPY_NS[_m] = __import__("scipy." + _m, fromlist=[_m])
    except Exception:
        SCIPY_NS[_m] = _MissingScipy(_m)

stdin = sys.stdin.buffer
stdout = sys.stdout.buffer

def read_exact(n):
    if n == 0:
        return b""
    buf = bytearray()
    while len(buf) < n:
        chunk = stdin.read(n - len(buf))
        if not chunk:
            return None
        buf.extend(chunk)
    return bytes(buf)

def send(obj, payload=None):
    body = json.dumps(obj).encode("utf-8")
    stdout.write(len(body).to_bytes(4, "little"))
    stdout.write(body)
    if payload is not None:
        stdout.write(payload)
    stdout.flush()

while True:
    line = stdin.readline()
    if not line:
        break  # EOF: parent closed stdin -> exit cleanly
    try:
        header = json.loads(line.decode("utf-8"))
        n = int(header["n"]); names = header["names"]
        out_name = header["output"]; script = header["script"]
    except Exception:
        break  # unframeable request; let the parent respawn
    payload = read_exact(4 * n * len(names))
    if payload is None:
        break  # EOF mid-request
    # scipy names go in FIRST so a curve mnemonic always wins the collision. A well whose log
    # is called STATS must shadow scipy.stats, not the other way round — the user's data is
    # never the thing that yields.
    ns = {"np": np, "numpy": np}
    ns.update(SCIPY_NS)
    for i, name in enumerate(names):
        ns[name] = np.frombuffer(payload, dtype=np.float32, count=n, offset=4 * n * i).copy()
    # When the output name collides with an input (or "depth") it is ALREADY bound here, so a
    # bare "out_name in ns" presence check can't tell a real in-place result from a no-op script
    # that never touched it. Snapshot the pre-exec values so a script must actually change them
    # (reassignment OR in-place mutation both count) — otherwise the untouched input would be
    # written back as though it were the computed result.
    had_out_input = out_name in names
    prev_out = ns[out_name].copy() if had_out_input else None
    try:
        exec(compile(script, "<equation>", "exec"), ns)
        if out_name not in ns:
            send({"ok": False, "error": f"script never assigned the output curve '{out_name}'"}); continue
        if had_out_input:
            try:
                unchanged = np.array_equal(np.asarray(ns[out_name], dtype=np.float32), prev_out, equal_nan=True)
            except Exception:
                unchanged = False
            if unchanged:
                send({"ok": False, "error": f"script never assigned the output curve '{out_name}' (it still equals the input '{out_name}')"}); continue
        out = np.asarray(ns[out_name], dtype=np.float32)
        if out.ndim == 0:
            out = np.full(n, float(out), dtype=np.float32)
        if out.shape != (n,):
            send({"ok": False, "error": f"output '{out_name}' has shape {out.shape}, expected ({n},)"}); continue
        send({"ok": True}, np.ascontiguousarray(out, dtype=np.float32).tobytes())
    except Exception as e:
        send({"ok": False, "error": f"script error: {e}"})
"#;

/// Finds a Python with numpy, once per session.
pub fn find_python() -> Option<PathBuf> {
    static FOUND: OnceLock<Option<PathBuf>> = OnceLock::new();
    FOUND
        .get_or_init(|| {
            let mut candidates: Vec<PathBuf> = Vec::new();
            // Current name first, then the pre-rename one. A user who set ARSHILLA_PYTHON
            // years ago keeps working without being told to change anything; a user reading
            // an error message today is only ever told the current name.
            for var in [PYTHON_ENV, PYTHON_ENV_LEGACY] {
                if let Ok(p) = std::env::var(var) {
                    let p = p.trim();
                    if !p.is_empty() {
                        candidates.push(PathBuf::from(p));
                    }
                }
            }
            if let Ok(local) = std::env::var("LOCALAPPDATA") {
                for ver in ["Python313", "Python312", "Python311", "Python310"] {
                    candidates.push(PathBuf::from(&local).join("Programs").join("Python").join(ver).join("python.exe"));
                }
            }
            candidates.push(PathBuf::from("python3"));
            candidates.push(PathBuf::from("python"));
            candidates.into_iter().find(|c| has_numpy(c))
        })
        .clone()
}

fn has_numpy(python: &PathBuf) -> bool {
    let mut cmd = Command::new(python);
    cmd.args(["-c", "import numpy"]).stdout(Stdio::null()).stderr(Stdio::null());
    hide_console(&mut cmd);
    cmd.status().map(|s| s.success()).unwrap_or(false)
}

/// What the equation engine can actually offer, probed once per session.
///
/// Probed UP FRONT rather than discovered when a script fails, for the same reason
/// `office.rs` probes its packages before opening a save dialog: a user should learn that
/// `optimize.curve_fit` is unavailable while writing the equation, not after queuing it across
/// ninety wells.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PythonStatus {
    /// Interpreter the engine will use; `None` when no Python with numpy was found.
    pub path: Option<String>,
    /// scipy's version string when it is importable in that interpreter.
    pub scipy: Option<String>,
}

/// The scipy version in `python`, if any. One extra subprocess, cached for the session.
fn scipy_version(python: &PathBuf) -> Option<String> {
    let mut cmd = Command::new(python);
    cmd.args(["-c", "import scipy; print(scipy.__version__)"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    hide_console(&mut cmd);
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

/// Interpreter + optional-package status, cached for the session.
pub fn python_status() -> PythonStatus {
    static STATUS: OnceLock<PythonStatus> = OnceLock::new();
    STATUS
        .get_or_init(|| match find_python() {
            None => PythonStatus { path: None, scipy: None },
            Some(p) => PythonStatus {
                scipy: scipy_version(&p),
                path: Some(p.to_string_lossy().to_string()),
            },
        })
        .clone()
}

/// Keeps per-run python subprocesses from flashing console windows on Windows.
pub fn hide_console(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let _ = cmd;
    }
}

// ---------------------------------------------------------------------------
// Persistent worker
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct WorkerResp {
    ok: bool,
    #[serde(default)]
    error: String,
}

/// Which kind of failure an `exec` hit: `Script` = the worker is alive and reported a
/// user-script error (report it, keep the worker); `Io` = the pipe broke / the worker died
/// (respawn and retry).
enum WorkerErr {
    Io(String),
    Script(String),
}

struct PyWorker {
    child: Child,
}

impl PyWorker {
    fn spawn(python: &PathBuf) -> Result<PyWorker, String> {
        let mut cmd = Command::new(python);
        cmd.args(["-c", RUNNER_LOOP])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()); // errors come back framed on stdout; nothing reads stderr
        hide_console(&mut cmd);
        let child = cmd.spawn().map_err(|e| format!("failed to start python: {e}"))?;
        Ok(PyWorker { child })
    }

    /// One request → one response on the live worker. Writes the header line + input arrays,
    /// then reads the length-prefixed status and (on success) the `n`-length output array.
    fn exec(&mut self, header_json: &str, arrays: &[&[f32]], n: usize) -> Result<Vec<f32>, WorkerErr> {
        let io = |e: std::io::Error| WorkerErr::Io(e.to_string());
        {
            let stdin = self.child.stdin.as_mut().ok_or_else(|| WorkerErr::Io("worker stdin closed".into()))?;
            stdin.write_all(header_json.as_bytes()).map_err(io)?;
            stdin.write_all(b"\n").map_err(io)?;
            for arr in arrays {
                stdin.write_all(bytemuck::cast_slice(arr)).map_err(io)?;
            }
            stdin.flush().map_err(io)?;
        } // stdin borrow ends before we read stdout
        let stdout = self.child.stdout.as_mut().ok_or_else(|| WorkerErr::Io("worker stdout closed".into()))?;
        let mut lenb = [0u8; 4];
        stdout.read_exact(&mut lenb).map_err(io)?;
        let len = u32::from_le_bytes(lenb) as usize;
        if len > (1 << 20) {
            return Err(WorkerErr::Io(format!("worker response header too large ({len} bytes)")));
        }
        let mut jb = vec![0u8; len];
        stdout.read_exact(&mut jb).map_err(io)?;
        let resp: WorkerResp =
            serde_json::from_slice(&jb).map_err(|e| WorkerErr::Io(format!("bad worker response: {e}")))?;
        if !resp.ok {
            return Err(WorkerErr::Script(resp.error));
        }
        let mut out = vec![0f32; n];
        stdout.read_exact(bytemuck::cast_slice_mut(&mut out)).map_err(io)?;
        Ok(out)
    }
}

fn worker_cell() -> &'static Mutex<Option<PyWorker>> {
    static WORKER: OnceLock<Mutex<Option<PyWorker>>> = OnceLock::new();
    WORKER.get_or_init(|| Mutex::new(None))
}

/// Runs one script request on the shared persistent worker, spawning it on first use and
/// respawning once if it has died (broken pipe). A user-script error returns `Err` but
/// leaves the worker alive for the next request.
fn run_on_worker(header_json: &str, arrays: &[&[f32]], n: usize) -> Result<Vec<f32>, String> {
    let python = find_python().ok_or_else(|| NO_PYTHON.to_string())?;
    let mut guard = worker_cell().lock().unwrap_or_else(|e| e.into_inner());
    for attempt in 0..2 {
        if guard.is_none() {
            *guard = Some(PyWorker::spawn(&python)?);
        }
        match guard.as_mut().unwrap().exec(header_json, arrays, n) {
            Ok(out) => return Ok(out),
            Err(WorkerErr::Script(s)) => return Err(s),
            Err(WorkerErr::Io(e)) => {
                *guard = None; // worker died — drop it, then respawn+retry once
                if attempt == 1 {
                    return Err(format!("python worker failed: {e}"));
                }
            }
        }
    }
    unreachable!()
}

/// Runs one script (with `names`/`arrays` starting at "depth") on the persistent worker.
/// `output` is the lowercase output variable name the script must assign.
fn exec_script(script: &str, names: &[String], arrays: &[&[f32]], n: usize, output: &str) -> Result<Vec<f32>, String> {
    let header = serde_json::json!({ "n": n, "names": names, "output": output, "script": script }).to_string();
    run_on_worker(&header, arrays, n)
}

/// Runs a python equation across wells (sequentially, one shared persistent worker),
/// writing results into `computed_curves` exactly like the Rhai path. Sequential (not
/// rayon) because the single worker and the single `Mutex<Connection>` serialize the work
/// anyway, and the win here is eliminating the per-well process spawn, not parallel compute.
pub fn run_python_equation(
    db: &Mutex<Connection>,
    equation: &EquationDef,
    well_ids: &[String],
    progress: Option<&crate::jobs::JobHandle>,
) -> Vec<EquationRunResult> {
    if find_python().is_none() {
        if let Some(p) = progress {
            for w in well_ids {
                p.finish_item(w, crate::jobs::ItemState::Failed, Some(NO_PYTHON.into()));
            }
        }
        return well_ids
            .iter()
            .map(|w| EquationRunResult { well_id: w.clone(), rows_written: 0, error: Some(NO_PYTHON.into()) })
            .collect();
    }

    well_ids
        .iter()
        .enumerate()
        .map(|(wi, well_id)| {
            if let Some(p) = progress {
                if p.is_cancelled() {
                    p.finish_item(well_id, crate::jobs::ItemState::Warned, Some("cancelled".into()));
                    return EquationRunResult { well_id: well_id.clone(), rows_written: 0, error: Some("cancelled".into()) };
                }
                p.set_current(Some(format!("Python equation: well {}/{}", wi + 1, well_ids.len())));
                p.start_item(well_id);
            }
            let (depth, columns) = {
                let conn = db.lock().unwrap();
                match fetch_curve_frame(&conn, well_id, &equation.input_curves) {
                    Ok(v) => v,
                    Err(e) => {
                        if let Some(p) = progress {
                            p.finish_item(well_id, crate::jobs::ItemState::Failed, Some(e.to_string()));
                        }
                        return EquationRunResult { well_id: well_id.clone(), rows_written: 0, error: Some(e.to_string()) };
                    }
                }
            };
            if depth.is_empty() {
                if let Some(p) = progress {
                    p.finish_item(well_id, crate::jobs::ItemState::Failed, Some("no curve data for well".into()));
                }
                return EquationRunResult { well_id: well_id.clone(), rows_written: 0, error: Some("no curve data for well".into()) };
            }

            let mut names: Vec<String> = vec!["depth".into()];
            let mut arrays: Vec<&[f32]> = vec![&depth];
            for curve in &equation.input_curves {
                let upper = curve.trim().to_uppercase();
                if let Some(values) = columns.get(&upper) {
                    names.push(upper.to_lowercase());
                    arrays.push(values);
                }
            }

            let output_name = equation.output_curve.trim().to_lowercase();
            match exec_script(&equation.script, &names, &arrays, depth.len(), &output_name) {
                Ok(mut result) => {
                    for v in &mut result {
                        if !v.is_finite() {
                            *v = f32::NAN;
                        }
                    }
                    // Same honesty guard as the Rhai path: an all-MISSING output (unresolvable
                    // input/output curve name) must not read as a clean success.
                    if !result.iter().any(|v| v.is_finite()) {
                        let msg = "equation produced no finite output — check the input/output curve name(s) resolve to data".to_string();
                        if let Some(p) = progress {
                            p.finish_item(well_id, crate::jobs::ItemState::Warned, Some(msg.clone()));
                        }
                        return EquationRunResult { well_id: well_id.clone(), rows_written: 0, error: Some(msg) };
                    }
                    let conn = db.lock().unwrap();
                    // Report the terminal state on EVERY branch, exactly as the Rhai sibling does
                    // (equations.rs). finish_item is the sole incrementer of the job's `done`, so a
                    // silent success left the bar at 0% with the well stuck amber "Running" under a
                    // "Completed" card, and a silent write/script error was visually identical to a
                    // success — no failure surface for the commonest authoring mistake (a bad script).
                    match write_equation_output(&conn, well_id, &depth, equation, &result) {
                        Ok(()) => {
                            if let Some(p) = progress {
                                p.finish_item(well_id, crate::jobs::ItemState::Ok, None);
                            }
                            EquationRunResult { well_id: well_id.clone(), rows_written: depth.len(), error: None }
                        }
                        Err(e) => {
                            if let Some(p) = progress {
                                p.finish_item(well_id, crate::jobs::ItemState::Failed, Some(e.to_string()));
                            }
                            EquationRunResult { well_id: well_id.clone(), rows_written: 0, error: Some(e.to_string()) }
                        }
                    }
                }
                Err(e) => {
                    if let Some(p) = progress {
                        p.finish_item(well_id, crate::jobs::ItemState::Failed, Some(e.clone()));
                    }
                    EquationRunResult { well_id: well_id.clone(), rows_written: 0, error: Some(e) }
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod env_name_tests {
    use super::*;

    /// A customer who has no Python is told what to set. Until 2026-07-31 they were told to
    /// set `ARSHILLA_PYTHON` — a variable named after this app's PREVIOUS name — in ten
    /// separate messages across DLIS import, ML, images and all three office exports. That is
    /// an instruction to configure a product that does not exist.
    ///
    /// The rule this pins: discovery still ACCEPTS the old name (nobody's setup breaks), but
    /// no message ever NAMES it.
    #[test]
    fn no_user_facing_message_names_the_pre_rename_variable() {
        assert!(
            NO_PYTHON.contains(PYTHON_ENV),
            "the message must name the current variable: {NO_PYTHON}"
        );
        assert!(
            !NO_PYTHON.contains(PYTHON_ENV_LEGACY),
            "the message must not name the pre-rename variable: {NO_PYTHON}"
        );
        assert_ne!(PYTHON_ENV, PYTHON_ENV_LEGACY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs one script on the worker against a 5-sample ramp and returns the output.
    /// Used by the scipy tests below.
    fn run_script(script: &str, output: &str) -> Result<Vec<f32>, String> {
        let depth: Vec<f32> = vec![1000.0, 1000.5, 1001.0, 1001.5, 1002.0];
        let gr: Vec<f32> = vec![20.0, 90.0, 25.0, 30.0, 35.0]; // one spike, for the despike test
        let names = vec!["depth".to_string(), "gr".to_string()];
        exec_script(script, &names, &[&depth, &gr], depth.len(), output)
    }

    /// scipy is bound into the script namespace when it is installed, and its absence is
    /// reported in a way a user can act on.
    ///
    /// The engine's ONE hard requirement is numpy; scipy is a bonus. So this test adapts to the
    /// machine rather than being `#[ignore]`d — on a box with scipy it proves the real call
    /// works, on a box without it proves the failure message names the interpreter and the pip
    /// command. Both are green, and the green gate never depends on an optional package.
    #[test]
    fn scipy_is_available_when_installed_and_names_the_fix_when_not() {
        let Some(python) = find_python() else {
            eprintln!("SKIP scipy test: no python with numpy on this machine");
            return;
        };
        let status = python_status();
        assert_eq!(status.path.as_deref(), Some(python.to_string_lossy().as_ref()));

        // Savitzky-Golay over 5 samples: a real scipy.signal call, not an import check.
        let out = run_script("grs = signal.savgol_filter(gr, 5, 2)", "grs");

        match status.scipy {
            Some(ref v) => {
                assert!(!v.is_empty(), "a reported scipy version must not be blank");
                let vals = out.unwrap_or_else(|e| panic!("scipy {v} is installed but the call failed: {e}"));
                assert_eq!(vals.len(), 5);
                assert!(vals.iter().all(|x| x.is_finite()), "smoothed GR must be finite: {vals:?}");
                // The point of smoothing: the 90 gAPI spike is pulled down toward its neighbours.
                assert!(vals[1] < 90.0, "the spike must be reduced, got {}", vals[1]);
            }
            None => {
                let e = out.expect_err("without scipy the script cannot succeed");
                let lower = e.to_lowercase();
                assert!(lower.contains("scipy"), "the error must name scipy: {e}");
                assert!(lower.contains("pip install"), "the error must give the fix: {e}");
                assert!(
                    e.contains(&python.to_string_lossy().to_string()),
                    "the error must name WHICH interpreter to install into: {e}"
                );
            }
        }
    }

    /// A curve mnemonic must win a name collision with a scipy submodule. A well logged with a
    /// curve called STATS is unusual but legal, and silently handing the script scipy.stats
    /// instead of the user's data would produce a confident wrong answer rather than an error.
    #[test]
    fn a_curve_named_like_a_scipy_module_shadows_it() {
        if find_python().is_none() {
            eprintln!("SKIP collision test: no python with numpy on this machine");
            return;
        }
        let depth: Vec<f32> = vec![1000.0, 1000.5, 1001.0];
        let stats: Vec<f32> = vec![1.0, 2.0, 3.0];
        let names = vec!["depth".to_string(), "stats".to_string()];
        let out = exec_script("doubled = stats * 2.0", &names, &[&depth, &stats], 3, "doubled")
            .expect("the curve must shadow scipy.stats");
        assert_eq!(out, vec![2.0f32, 4.0, 6.0]);
    }

    /// Every terminal branch must report its per-well state to the job. `finish_item` is the sole
    /// incrementer of `done`, so a branch that stays silent leaves the Processing panel at 0% with
    /// the well amber "Running" under a "Completed" card — and, for a script error, with no failure
    /// surface at all. Asserts on the JobView the panel actually renders, not on the return value.
    #[test]
    fn python_equation_reports_progress_on_every_terminal_branch() {
        use crate::db;
        use crate::jobs::{self, ItemState};
        use std::sync::atomic::AtomicBool;
        use std::sync::Arc;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "PY-1", None, None, Some(0.0)).unwrap();
        let depths = vec![1000.0f32, 1000.5, 1001.0];
        let n = depths.len();
        db::insert_standard_curves(
            &conn, wid, depths.clone(),
            vec![40.0; n], vec![f32::NAN; n], vec![f32::NAN; n],
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        let w = wid.to_string();
        let dbm = Mutex::new(conn);

        let eq = |output: &str, script: &str| EquationDef {
            equation_id: Uuid::new_v4().to_string(),
            name: "t".into(),
            description: None,
            script: script.into(),
            input_curves: vec!["GR".into()],
            output_curve: output.into(),
            output_units: None,
            language: "python".into(),
        };

        // Runs one equation against a fresh job and returns (done, item state) as the panel sees it.
        let run_with_job = |equation: &EquationDef| -> (usize, ItemState) {
            let reg = jobs::new_registry();
            let id = Uuid::new_v4();
            let h = jobs::register(
                &reg,
                id,
                "Equation",
                "python",
                vec![(w.clone(), "PY-1".to_string())],
                Arc::new(AtomicBool::new(false)),
                true,
            );
            h.running(1);
            let _ = run_python_equation(&dbm, equation, std::slice::from_ref(&w), Some(&h));
            let v = jobs::list(&reg).remove(0);
            (v.done, v.items[0].state)
        };

        if find_python().is_none() {
            // No python: the early-return branch must still finish the item (regression guard for
            // the one branch that is reachable on a machine without python).
            let (done, state) = run_with_job(&eq("VSHP", "vshp = gr / 100.0"));
            assert_eq!(done, 1, "the no-python branch must still count the well");
            assert_eq!(state, ItemState::Failed);
            eprintln!("skipping the python-backed branches: no python+numpy on this machine");
            return;
        }

        // Success: the branch that was silent — a healthy run must report 1/1 done and Ok.
        let (done, state) = run_with_job(&eq("VSHP", "vshp = gr / 100.0"));
        assert_eq!(done, 1, "a successful write must count one unit of progress (was stuck at 0)");
        assert_eq!(state, ItemState::Ok, "a successful well must not stay Running");

        // Script error: must surface as Failed, not sit amber "Running" under a Completed card.
        let (done, state) = run_with_job(&eq("VSHP", "vshp = undefined_name + 1"));
        assert_eq!(done, 1, "a script error must count one unit of progress");
        assert_eq!(state, ItemState::Failed, "a script error must be visible as a failure");
    }

    #[test]
    fn python_vectorized_roundtrip() {
        if find_python().is_none() {
            eprintln!("skipping: no python+numpy on this machine");
            return;
        }
        let depth: Vec<f32> = (0..100).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let mut gr: Vec<f32> = (0..100).map(|i| 20.0 + (i as f32 % 60.0) * 2.0).collect();
        gr[7] = f32::NAN; // NaN must propagate, not crash

        let names = vec!["depth".to_string(), "gr".to_string()];
        let arrays: Vec<&[f32]> = vec![&depth, &gr];
        let result = exec_script("vsh = np.clip((gr - 20.0) / (140.0 - 20.0), 0.0, 1.0)", &names, &arrays, 100, "vsh")
            .expect("python run failed");

        assert_eq!(result.len(), 100);
        assert!((result[0] - 0.0).abs() < 1e-6);
        // gr[30] = 20 + 30·2 = 80 → vsh = (80 − 20) / 120 = 0.5
        assert!((result[30] - 0.5).abs() < 1e-6);
        assert!(result[7].is_nan(), "NaN input must yield NaN output");
    }

    /// An output curve that collides with an input name must not silently succeed as a no-op —
    /// a script that never (re)assigns it is caught, while a real in-place edit passes.
    #[test]
    fn python_output_input_name_collision_guard() {
        if find_python().is_none() {
            eprintln!("skipping: no python+numpy on this machine");
            return;
        }
        let depth: Vec<f32> = (0..10).map(|i| 1000.0 + i as f32).collect();
        let gr: Vec<f32> = (0..10).map(|i| 50.0 + i as f32).collect();
        let names = vec!["depth".to_string(), "gr".to_string()];
        let arrays: Vec<&[f32]> = vec![&depth, &gr];

        // No-op: the script never touches gr (its own output name) → must error, not echo input.
        let noop = exec_script("x = gr * 2.0", &names, &arrays, 10, "gr");
        assert!(noop.is_err(), "no-op in-place output must be rejected, got {:?}", noop);

        // Real reassignment of the colliding name → succeeds with the changed values.
        let ok = exec_script("gr = gr + 100.0", &names, &arrays, 10, "gr")
            .expect("in-place reassign should succeed");
        assert!((ok[0] - 150.0).abs() < 1e-4, "gr should be input+100, got {}", ok[0]);

        // Regression: an output name that collides with a PRE-SEEDED namespace entry ("np"/
        // "numpy" = the numpy module, not an input curve) must NOT crash the worker — the guard
        // keys on the input-curve names, so a script that assigns it computes normally.
        let np_out = exec_script("np = gr + 1.0", &names, &arrays, 10, "np")
            .expect("output named 'np' must not crash the worker");
        assert!((np_out[0] - 51.0).abs() < 1e-4, "np output should be gr+1, got {}", np_out[0]);
    }

    #[test]
    fn python_reports_script_errors() {
        if find_python().is_none() {
            eprintln!("skipping: no python+numpy on this machine");
            return;
        }
        let depth = vec![1.0f32, 2.0];
        let names = vec!["depth".to_string()];
        let arrays: Vec<&[f32]> = vec![&depth];
        let err = exec_script("vsh = undefined_name + 1", &names, &arrays, 2, "vsh").unwrap_err();
        assert!(err.contains("script error"), "got: {err}");
    }

    #[test]
    fn worker_survives_a_script_error() {
        // The persistent worker must isolate a per-request script error: a bad script is
        // reported, and the SAME worker still serves the next good request (no respawn, no
        // leaked namespace state between requests).
        if find_python().is_none() {
            eprintln!("skipping: no python+numpy on this machine");
            return;
        }
        let depth = vec![1.0f32, 2.0, 3.0];
        let names = vec!["depth".to_string()];
        let arrays: Vec<&[f32]> = vec![&depth];

        let err = exec_script("boom = does_not_exist + 1", &names, &arrays, 3, "boom").unwrap_err();
        assert!(err.contains("script error"), "got: {err}");

        let ok = exec_script("out = depth * 2.0", &names, &arrays, 3, "out")
            .expect("worker should survive a prior script error and serve the next request");
        assert_eq!(ok, vec![2.0, 4.0, 6.0]);
    }

    /// T-AUX-17's main scenario, which is the PYTHON path — a different function from the Rhai
    /// one (`lib.rs` dispatches on `equation.language`), so the Rhai batch test in
    /// `equations.rs` says nothing about it.
    ///
    /// `worker_survives_a_script_error` already pins that the persistent worker survives one bad
    /// request. What is unpinned is the level above: that a raise in ONE well leaves that well
    /// unwritten and every other well complete. The failing well is listed FIRST, because this
    /// path is SEQUENTIAL (`.iter()`, not the Rhai path's `.par_iter()` — Python spawns a worker
    /// per well and parallelising it would multiply subprocesses), so a `?` in the wrong place
    /// would abandon the wells behind it.
    ///
    /// Skips with a printed reason where there is no interpreter, the same as its neighbours —
    /// so it is real coverage on a machine that has numpy and never a red gate on one that does
    /// not.
    #[test]
    fn a_python_raise_in_one_well_leaves_the_rest_of_the_batch_intact() {
        use crate::db;
        use uuid::Uuid;
        if find_python().is_none() {
            eprintln!("skipping: no python+numpy on this machine");
            return;
        }
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let seed = |name: &str, nphi: f32| -> String {
            let wid = Uuid::new_v4();
            db::insert_well(&conn, wid, name, None, None, Some(0.0)).unwrap();
            let depths = vec![1000.0f32, 1000.5, 1001.0, 1001.5];
            let n = depths.len();
            db::insert_standard_curves(
                &conn,
                wid,
                depths,
                vec![40.0; n],
                vec![2.0; n],
                vec![nphi; n],
                vec![2.4; n],
                vec![f32::NAN; n],
                vec![f32::NAN; n],
            )
            .unwrap();
            wid.to_string()
        };
        let no_nphi = seed("SANDI-PY-BARE", f32::NAN);
        let good_a = seed("SANDI-PY-A", 0.25);
        let good_b = seed("SANDI-PY-B", 0.30);
        let dbm = Mutex::new(conn);

        // Verbatim the script the manual step asks the user to type.
        let eq = EquationDef {
            equation_id: Uuid::new_v4().to_string(),
            name: "PHIN_TEST".into(),
            description: None,
            script: "if np.all(np.isnan(nphi)): raise ValueError(\"NPHI missing in this well\")\nphin_test = np.clip(nphi, 0, 0.6)".into(),
            input_curves: vec!["NPHI".into()],
            output_curve: "PHIN_TEST".into(),
            output_units: None,
            language: "python".into(),
        };
        let wells = [no_nphi.clone(), good_a.clone(), good_b.clone()];
        let res = run_python_equation(&dbm, &eq, &wells, None);

        assert_eq!(res.len(), 3);
        let bare = res[0].error.as_ref().expect("the NPHI-less well must fail");
        assert!(
            bare.contains("NPHI missing in this well"),
            "the user's OWN message must reach the run summary, not a generic failure: {bare}"
        );
        for (label, r) in [("A", &res[1]), ("B", &res[2])] {
            assert!(r.error.is_none(), "healthy well {label} must survive the raise ahead of it: {:?}", r.error);
            assert_eq!(r.rows_written, 4);
        }

        let conn = dbm.lock().unwrap();
        let rows = |w: &str| -> i64 {
            conn.query_row(
                "SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1 AND UPPER(curve_name) = 'PHIN_TEST'",
                duckdb::params![w],
                |r| r.get(0),
            )
            .unwrap()
        };
        assert_eq!(rows(&no_nphi), 0, "the raising well must leave no curve behind");
        assert_eq!(rows(&good_a), 4);
        assert_eq!(rows(&good_b), 4);
    }
}
