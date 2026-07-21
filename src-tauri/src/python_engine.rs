//! Vectorized Python (numpy) equation engine, tp_evaluate-style: the user script sees
//! every input curve as a float32 numpy array (NaN = missing) plus `depth`, and assigns
//! the output curve name as an array — numpy does the per-sample work.
//!
//! Python runs as a SUBPROCESS rather than an embedded interpreter: the app binary has
//! no link-time Python dependency, so a missing/foreign Python can never stop SandiBumi
//! from launching — a run just fails with a clear message. Discovery order:
//! `ARSHILLA_PYTHON` env var, then recent py.org per-user installs, then PATH; the first
//! interpreter that can `import numpy` wins (cached for the session).
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

const NO_PYTHON: &str =
    "no Python with numpy found — install Python 3.10+ with numpy, or set ARSHILLA_PYTHON to its python.exe";

/// Persistent request/response loop: read a JSON header line + raw f32 arrays, exec the
/// user script with numpy bound (fresh namespace each request), and reply with a
/// length-prefixed JSON status + the output array's bytes. Loops until stdin hits EOF (the
/// app closed the pipe), so one worker serves every well. A script exception is caught and
/// returned as `{"ok":false}` — the loop keeps going, so one bad script never kills the
/// worker.
const RUNNER_LOOP: &str = r#"
import sys, json
import numpy as np

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
    ns = {"np": np, "numpy": np}
    for i, name in enumerate(names):
        ns[name] = np.frombuffer(payload, dtype=np.float32, count=n, offset=4 * n * i).copy()
    try:
        exec(compile(script, "<equation>", "exec"), ns)
        if out_name not in ns:
            send({"ok": False, "error": f"script never assigned the output curve '{out_name}'"}); continue
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
            if let Ok(p) = std::env::var("ARSHILLA_PYTHON") {
                candidates.push(PathBuf::from(p));
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
                    let conn = db.lock().unwrap();
                    match write_equation_output(&conn, well_id, &depth, equation, &result) {
                        Ok(()) => EquationRunResult { well_id: well_id.clone(), rows_written: depth.len(), error: None },
                        Err(e) => EquationRunResult { well_id: well_id.clone(), rows_written: 0, error: Some(e.to_string()) },
                    }
                }
                Err(e) => EquationRunResult { well_id: well_id.clone(), rows_written: 0, error: Some(e) },
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
