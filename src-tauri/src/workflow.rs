//! Workflow runner: executes deterministic modules across wells (rayon-parallel),
//! resolving interval parameters per zone (Geolog-style), and the cutoff/summary
//! engine modeled on Geolog's .paysum specs.

use crate::db;
use crate::equations;
use crate::modules::{self, ArgKind, ModuleContext};
use duckdb::Connection;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Deserialize)]
pub struct RunModuleRequest {
    pub module: String,
    pub well_ids: Vec<String>,
    /// Arg name → curve mnemonic chosen in the dialog (defaults come from the manifest).
    pub log_inputs: HashMap<String, String>,
    /// Numeric interval parameters from the dialog (whole-well values; zone_params override).
    pub params: HashMap<String, f64>,
    /// String options from the dialog.
    pub opts: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleRunResult {
    pub well_id: String,
    pub rows_written: usize,
    pub output_curves: Vec<String>,
    pub error: Option<String>,
}

/// Builds per-sample parameter arrays for every Param arg: dialog value (or manifest
/// default) as the base, then zone_params overrides — '*' applies well-wide, named zones
/// apply over their depth range. This is the Geolog interval-parameter model.
fn resolve_param_arrays(
    conn: &Connection,
    well_id: &str,
    spec: &modules::ModuleSpec,
    req_params: &HashMap<String, f64>,
    depth: &[f32],
) -> Result<HashMap<String, Vec<f64>>, String> {
    let zones = db::list_zones(conn, well_id).map_err(|e| e.to_string())?;
    let zone_params = db::list_zone_params(conn, well_id).map_err(|e| e.to_string())?;
    let zone_range: HashMap<&str, (f32, f32)> =
        zones.iter().map(|z| (z.zone_name.as_str(), (z.top_depth, z.bottom_depth))).collect();

    let mut out = HashMap::new();
    for arg in spec.args.iter().filter(|a| a.kind == ArgKind::Param) {
        let base = req_params
            .get(&arg.name)
            .copied()
            .or_else(|| arg.default.parse().ok())
            .unwrap_or(f64::NAN);
        let mut arr = vec![base; depth.len()];

        // Well-wide default first, then named zones override it.
        for zp in zone_params.iter().filter(|z| z.param_name == arg.name) {
            let Some(v) = zp.value_num else { continue };
            if zp.zone_name == "*" {
                arr.fill(v as f64);
            }
        }
        for zp in zone_params.iter().filter(|z| z.param_name == arg.name) {
            let Some(v) = zp.value_num else { continue };
            if let Some(&(top, bottom)) = zone_range.get(zp.zone_name.as_str()) {
                for (i, d) in depth.iter().enumerate() {
                    if *d >= top && *d < bottom {
                        arr[i] = v as f64;
                    }
                }
            }
        }
        out.insert(arg.name.clone(), arr);
    }
    Ok(out)
}

/// Runs one module across every well: parse inputs, resolve zone parameters, evaluate,
/// and write output curves to computed_curves. Wells are processed in parallel.
pub fn run_workflow_module(db: &Mutex<Connection>, req: &RunModuleRequest) -> Vec<ModuleRunResult> {
    let spec = match modules::list_modules().into_iter().find(|m| m.name == req.module) {
        Some(s) => s,
        None => {
            return req
                .well_ids
                .iter()
                .map(|w| ModuleRunResult {
                    well_id: w.clone(),
                    rows_written: 0,
                    output_curves: vec![],
                    error: Some(format!("unknown module '{}'", req.module)),
                })
                .collect()
        }
    };

    // Options: dialog values over manifest defaults.
    let mut opts: HashMap<String, String> = spec
        .args
        .iter()
        .filter(|a| a.kind == ArgKind::Option)
        .map(|a| (a.name.clone(), a.default.clone()))
        .collect();
    for (k, v) in &req.opts {
        opts.insert(k.clone(), v.clone());
    }

    // Input curves: dialog mnemonic over manifest default mnemonic.
    let log_args: Vec<(String, String)> = spec
        .args
        .iter()
        .filter(|a| a.kind == ArgKind::LogIn)
        .map(|a| {
            let mnemonic = req.log_inputs.get(&a.name).cloned().unwrap_or_else(|| a.default.clone());
            (a.name.clone(), mnemonic)
        })
        .collect();
    // Expose each input's resolved mnemonic to the module as "__IN_<arg>", so modules
    // that derive their output names from their input (depth_shift → GR_DS) can.
    for (arg_name, mnemonic) in &log_args {
        opts.insert(format!("__IN_{arg_name}"), mnemonic.trim().to_uppercase());
    }

    req.well_ids
        .par_iter()
        .map(|well_id| {
            let run = || -> Result<(usize, Vec<String>), String> {
                let curve_names: Vec<String> = log_args.iter().map(|(_, m)| m.clone()).collect();
                let (depth, columns, params) = {
                    let conn = db.lock().unwrap();
                    let (depth, columns) = equations::fetch_curve_frame(&conn, well_id, &curve_names)
                        .map_err(|e| e.to_string())?;
                    if depth.is_empty() {
                        return Err("no curve data for well".into());
                    }
                    let params = resolve_param_arrays(&conn, well_id, &spec, &req.params, &depth)?;
                    (depth, columns, params)
                };

                let mut logs: HashMap<String, Vec<f32>> = HashMap::new();
                logs.insert("DEPTH".to_string(), depth.clone());
                for (arg_name, mnemonic) in &log_args {
                    let values = columns
                        .get(&mnemonic.trim().to_uppercase())
                        .cloned()
                        .unwrap_or_else(|| vec![f32::NAN; depth.len()]);
                    logs.insert(arg_name.clone(), values);
                }

                let ctx = ModuleContext { n: depth.len(), logs, params, opts: opts.clone() };
                let mut outputs = modules::run_module(&req.module, &ctx)?;

                // Optional bad-hole (or any flag) mask: samples where the mask curve == 1
                // are set missing in every output, so flagged intervals never pollute
                // results. The mask is resolved like any other input (generic store aware).
                let mask_name = req.opts.get("MASK").map(|s| s.trim()).unwrap_or("");
                if !mask_name.is_empty() {
                    let conn = db.lock().unwrap();
                    let (_, mcols) =
                        equations::fetch_curve_frame(&conn, well_id, &[mask_name.to_string()])
                            .map_err(|e| e.to_string())?;
                    drop(conn);
                    if let Some(mask) = mcols.get(&mask_name.to_uppercase()) {
                        for values in outputs.values_mut() {
                            for (v, m) in values.iter_mut().zip(mask.iter()) {
                                if *m == 1.0 {
                                    *v = f32::NAN;
                                }
                            }
                        }
                    }
                }

                let conn = db.lock().unwrap();
                let batch: Vec<(&str, &[f32])> =
                    outputs.iter().map(|(k, v)| (k.as_str(), v.as_slice())).collect();
                equations::write_computed_curves_batch(&conn, well_id, &depth, &batch)
                    .map_err(|e| e.to_string())?;
                let mut names: Vec<String> = outputs.keys().cloned().collect();
                names.sort();
                Ok((depth.len(), names))
            };

            match run() {
                Ok((rows, names)) => ModuleRunResult {
                    well_id: well_id.clone(),
                    rows_written: rows,
                    output_curves: names,
                    error: None,
                },
                Err(e) => ModuleRunResult { well_id: well_id.clone(), rows_written: 0, output_curves: vec![], error: Some(e) },
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Pay summary — cutoffs → flags → per-zone statistics (Geolog .paysum model)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct PaySummaryRequest {
    pub well_ids: Vec<String>,
    /// VSH <= vsh_max counts as sand.
    pub vsh_max: f64,
    /// PHIE >= phie_min counts as reservoir (with sand).
    pub phie_min: f64,
    /// SWE <= swe_max counts as pay (with reservoir).
    pub swe_max: f64,
    /// Optional PERM >= perm_min added to the pay flag when PERM exists.
    pub perm_min: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaySummaryRow {
    pub well_id: String,
    pub well_name: String,
    pub zone: String,
    pub flag: String, // SAND | RESERVOIR | PAY
    pub top: f32,
    pub bottom: f32,
    pub gross: f32,
    pub net: f32,
    pub ntg: f32,
    pub avg_vsh: f32,
    pub avg_phie: f32,
    /// PHIE-weighted average SWE (Geolog .paysum convention).
    pub avg_swe: f32,
    pub hpv: f32, // sum of PHIE*(1-SWE)*thickness over net
}

const SUMMARY_FLAGS: [&str; 3] = ["SAND", "RESERVOIR", "PAY"];

/// Computes the pay summary per well per zone and writes FLAG_SAND / FLAG_RESERVOIR /
/// FLAG_PAY curves. Wells without zones get a single whole-well "ALL" zone.
pub fn run_pay_summary(db: &Mutex<Connection>, req: &PaySummaryRequest) -> Result<Vec<PaySummaryRow>, String> {
    let curve_names: Vec<String> = vec!["VSH".into(), "PHIE".into(), "SWE".into(), "PERM".into()];
    let mut all_rows = Vec::new();

    for well_id in &req.well_ids {
        let conn = db.lock().unwrap();
        let well_name: String = conn
            .query_row(
                "SELECT well_name FROM wells WHERE well_id = ?1",
                duckdb::params![well_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| well_id.clone());

        let (depth, columns) =
            equations::fetch_curve_frame(&conn, well_id, &curve_names).map_err(|e| e.to_string())?;
        if depth.is_empty() {
            continue;
        }
        let mut zones = db::list_zones(&conn, well_id).map_err(|e| e.to_string())?;
        drop(conn);

        if zones.is_empty() {
            zones.push(db::ZoneEntry {
                zone_name: "ALL".into(),
                top_depth: depth[0],
                bottom_depth: *depth.last().unwrap(),
            });
        }

        let n = depth.len();
        let vsh = &columns["VSH"];
        let phie = &columns["PHIE"];
        let swe = &columns["SWE"];
        let perm = &columns["PERM"];
        let has_perm_cut = req.perm_min.is_some() && perm.iter().any(|v| !v.is_nan());

        // Sample thickness: forward depth difference, last sample reuses the previous step.
        let mut step = vec![0.0f32; n];
        for i in 0..n {
            step[i] = if i + 1 < n {
                depth[i + 1] - depth[i]
            } else if i > 0 {
                step[i - 1]
            } else {
                0.0
            };
        }

        // Flags per sample: NaN inputs exclude the sample (flag stays NaN).
        let mut flag_sand = vec![f32::NAN; n];
        let mut flag_res = vec![f32::NAN; n];
        let mut flag_pay = vec![f32::NAN; n];
        for i in 0..n {
            if vsh[i].is_nan() {
                continue;
            }
            let sand = (vsh[i] as f64) <= req.vsh_max;
            flag_sand[i] = sand as u8 as f32;
            if phie[i].is_nan() {
                continue;
            }
            let res = sand && (phie[i] as f64) >= req.phie_min;
            flag_res[i] = res as u8 as f32;
            if swe[i].is_nan() {
                continue;
            }
            let mut pay = res && (swe[i] as f64) <= req.swe_max;
            if has_perm_cut && !perm[i].is_nan() {
                pay = pay && (perm[i] as f64) >= req.perm_min.unwrap();
            }
            flag_pay[i] = pay as u8 as f32;
        }

        {
            let conn = db.lock().unwrap();
            for (name, values) in
                [("FLAG_SAND", &flag_sand), ("FLAG_RESERVOIR", &flag_res), ("FLAG_PAY", &flag_pay)]
            {
                equations::write_computed_curve(&conn, well_id, &depth, name, values).map_err(|e| e.to_string())?;
            }
        }

        for zone in &zones {
            for flag_name in SUMMARY_FLAGS {
                let flags = match flag_name {
                    "SAND" => &flag_sand,
                    "RESERVOIR" => &flag_res,
                    _ => &flag_pay,
                };
                let mut net = 0.0f64;
                let mut sum_vsh = 0.0f64;
                let mut sum_phie = 0.0f64;
                let mut sum_phie_swe = 0.0f64;
                let mut sum_phie_w = 0.0f64;
                let mut hpv = 0.0f64;

                for i in 0..n {
                    let d = depth[i];
                    if d < zone.top_depth || d >= zone.bottom_depth {
                        continue;
                    }
                    if flags[i] != 1.0 {
                        continue;
                    }
                    let h = step[i] as f64;
                    net += h;
                    if !vsh[i].is_nan() {
                        sum_vsh += vsh[i] as f64 * h;
                    }
                    if !phie[i].is_nan() {
                        sum_phie += phie[i] as f64 * h;
                        if !swe[i].is_nan() {
                            sum_phie_swe += phie[i] as f64 * swe[i] as f64 * h;
                            sum_phie_w += phie[i] as f64 * h;
                            hpv += phie[i] as f64 * (1.0 - swe[i] as f64) * h;
                        }
                    }
                }

                let gross = zone.bottom_depth - zone.top_depth;
                all_rows.push(PaySummaryRow {
                    well_id: well_id.clone(),
                    well_name: well_name.clone(),
                    zone: zone.zone_name.clone(),
                    flag: flag_name.to_string(),
                    top: zone.top_depth,
                    bottom: zone.bottom_depth,
                    gross,
                    net: net as f32,
                    ntg: if gross > 0.0 { (net / gross as f64) as f32 } else { 0.0 },
                    avg_vsh: if net > 0.0 { (sum_vsh / net) as f32 } else { f32::NAN },
                    avg_phie: if net > 0.0 { (sum_phie / net) as f32 } else { f32::NAN },
                    avg_swe: if sum_phie_w > 0.0 { (sum_phie_swe / sum_phie_w) as f32 } else { f32::NAN },
                    hpv: hpv as f32,
                });
            }
        }
    }

    Ok(all_rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest;
    use std::collections::HashMap;

    /// Phase 7 wiring test — no field files, no vcvars: a well whose PEF, DRHO and CALI
    /// live ONLY in the generic curve store (never the fixed six) drives (1) multimin,
    /// proving the generic-store read fallback feeds a real module through the runner;
    /// (2) the badhole flag from generic DRHO/CALI; and (3) a masked vsh_gr run, proving
    /// flagged intervals are NaN'd out of module outputs.
    #[test]
    fn phase7_generic_store_feeds_modules_and_mask() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "MM-1", None, None, Some(0.0)).unwrap();
        let w = wid.to_string();

        // Forward-model a clean wet sand at every depth (70% sand / 30% water) so we know
        // the answer, plus one washed-out sample flagged by CALI.
        let depths = vec![1000.0f32, 1000.5, 1001.0, 1001.5];
        let (vs, vw) = (0.70f64, 0.30f64);
        let rhob_v = (vs * 2.65 + vw * 1.0) as f32;
        let nphi_v = (vs * -0.02 + vw * 1.0) as f32;
        let dt_v = (vs * 55.5 + vw * 189.0) as f32;
        let pef_v = (vs * 1.81 + vw * 0.36) as f32;
        let n = depths.len();

        // RHOB/NPHI/DT go in the fixed table; GR too (for the masked run). RES/SP unused.
        db::insert_standard_curves(
            &conn,
            wid,
            depths.clone(),
            vec![40.0; n],       // GR
            vec![f32::NAN; n],   // RES_DEEP
            vec![nphi_v; n],     // NPHI
            vec![rhob_v; n],     // RHOB
            vec![dt_v; n],       // DT
            vec![f32::NAN; n],   // SP
        )
        .unwrap();

        // PEF, DRHO, CALI ONLY in the generic store. CALI is huge (washout) at sample 2.
        let put = |mnem: &str, family: &str, unit: &str, vals: Vec<f32>| {
            let id = db::upsert_curve_meta(&conn, &w, "RAW", mnem, Some(unit), Some(family), Some("test"), None).unwrap();
            db::insert_curve_samples(&conn, &id, &depths, &vals).unwrap();
        };
        put("PEFZ", "PEF", "b/e", vec![pef_v; n]); // mnemonic differs → must resolve by family
        put("HDRA", "DRHO", "g/cc", vec![0.01, 0.01, 0.20, 0.01]); // big DRHO at sample 2
        put("HCAL", "CALI", "in", vec![8.6, 8.6, 14.0, 8.6]); // washout at sample 2 (BS 8.5)

        let dbm = Mutex::new(conn);
        let run = |module: &str, params: &[(&str, f64)], opts: &[(&str, &str)]| -> Vec<ModuleRunResult> {
            let req = RunModuleRequest {
                module: module.into(),
                well_ids: vec![w.clone()],
                log_inputs: HashMap::new(),
                params: params.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                opts: opts.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            };
            run_workflow_module(&dbm, &req)
        };

        // (1) multimin — PEF comes only from the generic store. If the fallback were
        // broken, PEF would be all-NaN but the other three tools still solve; so to prove
        // PEF was actually read we check the clean-sand recovery is tight (all four tools).
        let r = run("multimin", &[], &[]);
        assert!(r[0].error.is_none(), "multimin: {:?}", r[0].error);
        assert!(r[0].output_curves.contains(&"VOL_SAND".to_string()));
        {
            let conn = dbm.lock().unwrap();
            let (_, cols) = equations::fetch_curve_frame(&conn, &w, &["VOL_SAND".into(), "VOL_WATER".into(), "VOL_CLAY".into()]).unwrap();
            assert!((cols["VOL_SAND"][0] - 0.70).abs() < 0.02, "sand={}", cols["VOL_SAND"][0]);
            assert!((cols["VOL_WATER"][0] - 0.30).abs() < 0.02, "water={}", cols["VOL_WATER"][0]);
            assert!(cols["VOL_CLAY"][0] < 0.03, "clay leaked (PEF likely not read): {}", cols["VOL_CLAY"][0]);
        }

        // (2) badhole — DRHO and CALI resolve from the generic store; sample 2 is bad.
        let r = run("badhole", &[("DRHO_MAX", 0.05), ("DCAL_MAX", 1.0), ("BS_DEF", 8.5)], &[]);
        assert!(r[0].error.is_none(), "badhole: {:?}", r[0].error);
        {
            let conn = dbm.lock().unwrap();
            let (_, cols) = equations::fetch_curve_frame(&conn, &w, &["BADHOLE".into()]).unwrap();
            let bh = &cols["BADHOLE"];
            assert_eq!(bh[0], 0.0, "good hole");
            assert_eq!(bh[2], 1.0, "washout must flag bad");
        }

        // (3) masked vsh_gr — the badhole flag masks sample 2 out of the output.
        let r = run("vsh_gr", &[("GR_MA", 20.0), ("GR_SH", 120.0)], &[("MASK", "BADHOLE")]);
        assert!(r[0].error.is_none(), "vsh_gr masked: {:?}", r[0].error);
        {
            let conn = dbm.lock().unwrap();
            let (_, cols) = equations::fetch_curve_frame(&conn, &w, &["VSH".into()]).unwrap();
            let vsh = &cols["VSH"];
            assert!(!vsh[0].is_nan(), "good-hole sample kept");
            assert!(vsh[2].is_nan(), "bad-hole sample must be masked to NaN");
        }
    }

    /// Full deterministic chain against real field LAS files: import → VSH(GR) →
    /// PHI(D-N) → SW(Indonesia) → PERM(Timur) → pay summary. Ignored by default
    /// (machine-specific paths); run with:
    /// `cargo test --release -- --ignored --nocapture test_full_deterministic_chain`
    #[test]
    #[ignore]
    fn test_full_deterministic_chain() {
        let paths: Vec<String> = vec![
            r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00001_lapi2023_fprooh.las",
            r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00002_lapi2023_fprooh.las",
            r"D:\01. Work\00. Guidebook\02. Guidebook Geolog\Loglan\mina01060d1_study_minas_itb2022_final.las",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let db_path = std::env::temp_dir().join("arshilla_workflow_test.duckdb");
        let _ = std::fs::remove_file(&db_path);
        let conn = crate::db::init_db(db_path.to_str().unwrap()).expect("init_db failed");

        let results = ingest::import_las_files(&conn, &paths);
        let well_ids: Vec<String> = results
            .iter()
            .map(|r| r.well_id.clone().unwrap_or_else(|| panic!("import failed: {:?}", r.error)))
            .collect();

        let db = Mutex::new(conn);
        let run = |module: &str, params: &[(&str, f64)], opts: &[(&str, &str)]| {
            let req = RunModuleRequest {
                module: module.into(),
                well_ids: well_ids.clone(),
                log_inputs: HashMap::new(),
                params: params.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                opts: opts.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            };
            let results = run_workflow_module(&db, &req);
            for r in &results {
                println!("{module}: well={} rows={} outputs={:?} err={:?}", r.well_id, r.rows_written, r.output_curves, r.error);
                assert!(r.error.is_none(), "{module} failed: {:?}", r.error);
            }
        };

        run("vsh_gr", &[("GR_MA", 25.0), ("GR_SH", 130.0)], &[("OPT_GR", "LINEAR")]);
        run(
            "phi_dn",
            &[("RHO_MA", 2.645), ("RHO_SH", 2.5), ("NPHI_SH", 0.35), ("RHO_DSH", 2.65), ("PHIE_MAX", 0.35)],
            &[("OPT_XPLOT", "AVERAGE")],
        );
        run(
            "sw_indo",
            &[("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.2), ("RT_SH", 4.0)],
            &[("OPT_INDO", "FULL"), ("OPT_RW", "CONSTANT")],
        );
        run("perm_wyllie_rose", &[("SWE_IRR", 0.15)], &[("OPT_WR", "TIMUR")]);

        // Physical sanity: VSH/PHIE/SWE within [0,1], PERM non-negative, and each
        // well has a meaningful number of valid samples.
        {
            let conn = db.lock().unwrap();
            for (curve, lo, hi) in [("VSH", 0.0, 1.0), ("PHIE", 0.0, 0.5), ("SWE", 0.0, 1.0), ("PERM", 0.0, f64::MAX)] {
                let (count, min, max): (i64, f64, f64) = conn
                    .query_row(
                        "SELECT count(value), min(value), max(value) FROM computed_curves
                         WHERE curve_name = ?1 AND NOT isnan(value)",
                        duckdb::params![curve],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .unwrap();
                println!("{curve}: n={count} min={min:.4} max={max:.4}");
                assert!(count > 1000, "{curve}: too few valid samples ({count})");
                assert!(min >= lo && max <= hi, "{curve} out of physical range: [{min}, {max}]");
            }
        }

        // Pay summary over the whole wells (no zones defined → single ALL zone).
        let rows = run_pay_summary(
            &db,
            &PaySummaryRequest { well_ids: well_ids.clone(), vsh_max: 0.5, phie_min: 0.1, swe_max: 0.6, perm_min: None },
        )
        .expect("pay summary failed");
        assert_eq!(rows.len(), well_ids.len() * 3); // SAND/RESERVOIR/PAY per well
        for r in &rows {
            println!(
                "{} {} {}: gross={:.1} net={:.1} ntg={:.3} avgPHIE={:.3} avgSWE={:.3} HPV={:.2}",
                r.well_name, r.zone, r.flag, r.gross, r.net, r.ntg, r.avg_phie, r.avg_swe, r.hpv
            );
            assert!(r.net <= r.gross + 0.01);
            if r.flag == "PAY" {
                let res = rows
                    .iter()
                    .find(|x| x.well_id == r.well_id && x.zone == r.zone && x.flag == "RESERVOIR")
                    .unwrap();
                assert!(r.net <= res.net + 0.01, "PAY net exceeds RESERVOIR net");
            }
        }
    }
}
