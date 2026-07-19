//! Vectorized Python (numpy) equation engine, tp_evaluate-style: the user script sees
//! every input curve as a float32 numpy array (NaN = missing) plus `depth`, and assigns
//! the output curve name as an array — one exec per well, numpy does the per-sample work.
//!
//! Python runs as a SUBPROCESS rather than an embedded interpreter: the app binary has
//! no link-time Python dependency, so a missing/foreign Python can never stop SandiBumi
//! from launching — a run just fails with a clear message. Discovery order:
//! `ARSHILLA_PYTHON` env var, then recent py.org per-user installs, then PATH; the first
//! interpreter that can `import numpy` wins (cached for the session).

use crate::equations::{fetch_curve_frame, write_computed_curve, EquationDef, EquationRunResult};
use duckdb::Connection;
use rayon::prelude::*;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};

/// In-process runner: reads a JSON header line then raw f32 arrays from stdin, execs the
/// user script with numpy bound, writes the output array's raw bytes to stdout.
const RUNNER: &str = r#"
import sys, json
import numpy as np

header = json.loads(sys.stdin.buffer.readline().decode("utf-8"))
n = header["n"]
names = header["names"]
out_name = header["output"]
raw = sys.stdin.buffer.read(4 * n * len(names))
ns = {"np": np, "numpy": np}
for i, name in enumerate(names):
    ns[name] = np.frombuffer(raw, dtype=np.float32, count=n, offset=4 * n * i).copy()
try:
    exec(compile(header["script"], "<equation>", "exec"), ns)
except Exception as e:
    print(f"script error: {e}", file=sys.stderr)
    sys.exit(2)
if out_name not in ns:
    print(f"script never assigned the output curve '{out_name}'", file=sys.stderr)
    sys.exit(2)
out = np.asarray(ns[out_name], dtype=np.float32)
if out.ndim == 0:
    out = np.full(n, float(out), dtype=np.float32)
if out.shape != (n,):
    print(f"output '{out_name}' has shape {out.shape}, expected ({n},)", file=sys.stderr)
    sys.exit(2)
sys.stdout.buffer.write(out.tobytes())
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

/// Executes one script against one well's arrays. `names` are the lowercase variable
/// names (starting with "depth"), `arrays` the matching f32 columns, all length `n`.
pub fn exec_python_script(
    python: &PathBuf,
    script: &str,
    names: &[String],
    arrays: &[&[f32]],
    n: usize,
    output_name: &str,
) -> Result<Vec<f32>, String> {
    let header = serde_json::json!({ "n": n, "names": names, "output": output_name, "script": script });

    let mut cmd = Command::new(python);
    cmd.args(["-c", RUNNER]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("failed to start python: {e}"))?;

    {
        let stdin = child.stdin.as_mut().ok_or("failed to open python stdin")?;
        stdin.write_all(header.to_string().as_bytes()).map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        for arr in arrays {
            stdin.write_all(bytemuck::cast_slice(arr)).map_err(|e| e.to_string())?;
        }
    } // drop closes stdin

    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("python failed");
        return Err(last.trim().to_string());
    }
    if output.stdout.len() != n * 4 {
        return Err(format!("python returned {} bytes, expected {}", output.stdout.len(), n * 4));
    }
    let mut result = vec![0f32; n];
    bytemuck::cast_slice_mut::<f32, u8>(&mut result).copy_from_slice(&output.stdout);
    Ok(result)
}

/// Runs a python equation across wells (rayon-parallel, one subprocess per well),
/// writing results into `computed_curves` exactly like the Rhai path.
pub fn run_python_equation(db: &Mutex<Connection>, equation: &EquationDef, well_ids: &[String]) -> Vec<EquationRunResult> {
    let Some(python) = find_python() else {
        let msg = "no Python with numpy found — install Python 3.10+ with numpy, or set ARSHILLA_PYTHON to its python.exe";
        return well_ids.iter().map(|w| EquationRunResult { well_id: w.clone(), rows_written: 0, error: Some(msg.into()) }).collect();
    };

    well_ids
        .par_iter()
        .map(|well_id| {
            let (depth, columns) = {
                let conn = db.lock().unwrap();
                match fetch_curve_frame(&conn, well_id, &equation.input_curves) {
                    Ok(v) => v,
                    Err(e) => return EquationRunResult { well_id: well_id.clone(), rows_written: 0, error: Some(e.to_string()) },
                }
            };
            if depth.is_empty() {
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
            match exec_python_script(&python, &equation.script, &names, &arrays, depth.len(), &output_name) {
                Ok(mut result) => {
                    for v in &mut result {
                        if !v.is_finite() {
                            *v = f32::NAN;
                        }
                    }
                    let conn = db.lock().unwrap();
                    match write_computed_curve(&conn, well_id, &depth, &equation.output_curve, &result) {
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
        let Some(python) = find_python() else {
            eprintln!("skipping: no python+numpy on this machine");
            return;
        };
        let depth: Vec<f32> = (0..100).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let mut gr: Vec<f32> = (0..100).map(|i| 20.0 + (i as f32 % 60.0) * 2.0).collect();
        gr[7] = f32::NAN; // NaN must propagate, not crash

        let names = vec!["depth".to_string(), "gr".to_string()];
        let arrays: Vec<&[f32]> = vec![&depth, &gr];
        let result = exec_python_script(
            &python,
            "vsh = np.clip((gr - 20.0) / (140.0 - 20.0), 0.0, 1.0)",
            &names,
            &arrays,
            100,
            "vsh",
        )
        .expect("python run failed");

        assert_eq!(result.len(), 100);
        assert!((result[0] - 0.0).abs() < 1e-6);
        // gr[30] = 20 + 30·2 = 80 → vsh = (80 − 20) / 120 = 0.5
        assert!((result[30] - 0.5).abs() < 1e-6);
        assert!(result[7].is_nan(), "NaN input must yield NaN output");
    }

    #[test]
    fn python_reports_script_errors() {
        let Some(python) = find_python() else {
            eprintln!("skipping: no python+numpy on this machine");
            return;
        };
        let depth = vec![1.0f32, 2.0];
        let names = vec!["depth".to_string()];
        let arrays: Vec<&[f32]> = vec![&depth];
        let err = exec_python_script(&python, "vsh = undefined_name + 1", &names, &arrays, 2, "vsh").unwrap_err();
        assert!(err.contains("script error"), "got: {err}");
    }
}
