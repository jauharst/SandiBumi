use duckdb::{params, Connection};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Deserialize)]
pub struct EvaluationParams {
    pub vsh_cutoff: f32,
    pub a: f32,
    pub m: f32,
    pub n: f32,
    pub rw: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct WellEvaluationResult {
    pub well_id: String,
    pub depth: Vec<f32>,
    pub vsh: Vec<f32>,
    pub phie: Vec<f32>,
    pub sw: Vec<f32>,
    pub net_pay: Vec<bool>,
}

const RHOB_MATRIX: f32 = 2.65; // sandstone matrix density, g/cc
const RHOB_FLUID: f32 = 1.0; // formation fluid density, g/cc

type WellCurves = (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>);

fn fetch_well_curves(conn: &Connection, well_id: &str) -> duckdb::Result<WellCurves> {
    let mut stmt = conn.prepare(
        "SELECT depth, gr, res_deep, rhob FROM standard_curves WHERE well_id = ?1 ORDER BY depth",
    )?;

    let mut depth = Vec::new();
    let mut gr = Vec::new();
    let mut res_deep = Vec::new();
    let mut rhob = Vec::new();

    let rows = stmt.query_map(params![well_id], |row| {
        Ok((
            row.get::<_, f32>(0)?,
            row.get::<_, f32>(1)?,
            row.get::<_, f32>(2)?,
            row.get::<_, f32>(3)?,
        ))
    })?;
    for r in rows {
        let (d, g, rd, rb) = r?;
        depth.push(d);
        gr.push(g);
        res_deep.push(rd);
        rhob.push(rb);
    }
    Ok((depth, gr, res_deep, rhob))
}

/// Linear gamma-ray shale volume index. NaN input strictly propagates to NaN output.
fn vshale_linear(gr: &[f32], gr_clean: f32, gr_shale: f32) -> Vec<f32> {
    gr.iter()
        .map(|&g| {
            if g.is_nan() {
                f32::NAN
            } else {
                ((g - gr_clean) / (gr_shale - gr_clean)).clamp(0.0, 1.0)
            }
        })
        .collect()
}

/// Density porosity: phi = (rhob_matrix - rhob) / (rhob_matrix - rhob_fluid).
fn density_porosity(rhob: &[f32]) -> Vec<f32> {
    rhob.iter()
        .map(|&rb| {
            if rb.is_nan() {
                f32::NAN
            } else {
                ((RHOB_MATRIX - rb) / (RHOB_MATRIX - RHOB_FLUID)).clamp(0.0, 1.0)
            }
        })
        .collect()
}

/// Archie water saturation: Sw = ((a * Rw) / (phi^m * Rt))^(1/n). Missing or non-physical
/// (<= 0) inputs map to NaN rather than producing an inf/NaN-by-accident result.
fn archie_sw(phie: &[f32], rt: &[f32], params: &EvaluationParams) -> Vec<f32> {
    phie.iter()
        .zip(rt.iter())
        .map(|(&phi, &rt_val)| {
            if phi.is_nan() || rt_val.is_nan() || phi <= 0.0 || rt_val <= 0.0 {
                return f32::NAN;
            }
            let sw = ((params.a * params.rw) / (phi.powf(params.m) * rt_val)).powf(1.0 / params.n);
            sw.clamp(0.0, 1.0)
        })
        .collect()
}

/// Evaluates every well in `well_ids` concurrently across CPU cores via `rayon`. Each well's
/// curves are read from DuckDB under a short-lived lock; the deterministic math itself runs
/// lock-free so 2,000+ wells scale with available cores rather than serializing on I/O.
pub fn run_multi_well_evaluation(
    db: &Mutex<Connection>,
    well_ids: &[String],
    params: &EvaluationParams,
) -> Vec<WellEvaluationResult> {
    well_ids
        .par_iter()
        .filter_map(|well_id| {
            let (depth, gr, res_deep, rhob) = {
                let conn = db.lock().unwrap();
                fetch_well_curves(&conn, well_id).ok()?
            };
            if depth.is_empty() {
                return None;
            }

            let gr_clean = gr.iter().copied().filter(|v| !v.is_nan()).fold(f32::INFINITY, f32::min);
            let gr_shale = gr.iter().copied().filter(|v| !v.is_nan()).fold(f32::NEG_INFINITY, f32::max);

            let vsh = vshale_linear(&gr, gr_clean, gr_shale);
            let phie = density_porosity(&rhob);
            let sw = archie_sw(&phie, &res_deep, params);
            let net_pay = vsh.iter().map(|v| !v.is_nan() && *v <= params.vsh_cutoff).collect();

            Some(WellEvaluationResult {
                well_id: well_id.clone(),
                depth,
                vsh,
                phie,
                sw,
                net_pay,
            })
        })
        .collect()
}
