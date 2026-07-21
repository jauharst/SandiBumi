//! End-to-end pipeline smoke/validation harness against real BLSO (Balam South) field
//! LAS files. Not part of the normal test run (marked `#[ignore]`); invoke explicitly:
//!
//!   cargo test --release pipeline_blso -- --ignored --nocapture
//!
//! It imports the four delivered wells, runs every module in catalogue order with default
//! parameters (the "run all modules until print" request), runs the pay summary, renders a
//! PDF report, validates computed PHIE/SWE/VSH against the delivery's own curves, and then
//! stress-tests 100 duplicated wells to observe rayon parallelism.

use crate::composite::{CompositeSpec, PageSize};
use crate::modules::{self, ArgKind};
use crate::workflow::{run_pay_summary, run_workflow_module, PaySummaryRequest, RunModuleRequest};
use duckdb::{params, Connection};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

const LAS_FILES: [&str; 4] = [
    r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00025_lapi2023_fprooh.las",
    r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00358d1_lapi2023_fprooh.las",
    r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00368d1_lapi2023_fprooh.las",
    r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00176_lapi2023_fprooh.las",
];

/// Count of finite (non-NaN) samples for a computed curve.
fn finite_count(conn: &Connection, well_id: &str, curve: &str) -> i64 {
    conn.query_row(
        "SELECT count(*) FROM computed_curves WHERE well_id = ?1 AND curve_name = ?2 AND NOT isnan(value)",
        params![well_id, curve],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

/// Mean of finite samples for a computed curve (NaN if none).
fn computed_mean(conn: &Connection, well_id: &str, curve: &str) -> f64 {
    conn.query_row(
        "SELECT avg(value) FROM computed_curves WHERE well_id = ?1 AND curve_name = ?2 AND NOT isnan(value)",
        params![well_id, curve],
        |r| r.get::<_, Option<f64>>(0),
    )
    .ok()
    .flatten()
    .unwrap_or(f64::NAN)
}

/// Mean of finite samples for a RAW generic-store curve (the delivery's own answer).
fn raw_mean(conn: &Connection, well_id: &str, mnemonic: &str) -> f64 {
    conn.query_row(
        "SELECT avg(s.value) FROM curve_samples s JOIN curve_meta m ON s.curve_id = m.curve_id \
         WHERE m.well_id = ?1 AND upper(m.mnemonic) = ?2 AND NOT isnan(s.value)",
        params![well_id, mnemonic.to_uppercase()],
        |r| r.get::<_, Option<f64>>(0),
    )
    .ok()
    .flatten()
    .unwrap_or(f64::NAN)
}

#[test]
#[ignore]
fn pipeline_blso_full_run() {
    let db_path = std::env::temp_dir().join("arshilla_blso_pipeline.duckdb");
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));
    let conn = crate::db::init_db(db_path.to_str().unwrap()).expect("init_db");

    // ---- 1. Import -------------------------------------------------------
    let paths: Vec<String> = LAS_FILES.iter().map(|s| s.to_string()).collect();
    let t0 = Instant::now();
    let results = crate::ingest::import_las_files(&conn, &paths, None);
    println!("\n=== IMPORT ({:?}) ===", t0.elapsed());
    for r in &results {
        println!(
            "  {:<12} rows={:<6} err={:?}",
            r.well_name.clone().unwrap_or_default(),
            r.rows,
            r.error
        );
    }
    let failures: Vec<_> = results.iter().filter(|r| r.error.is_some()).collect();
    assert!(failures.is_empty(), "import failures: {failures:?}");

    let wells = crate::db::list_wells(&conn).expect("list_wells");
    let well_ids: Vec<String> = wells.iter().map(|w| w.well_id.clone()).collect();
    assert_eq!(well_ids.len(), 4);

    // Sanity: did the standard-curve mapping actually capture the inputs?
    println!("\n=== STANDARD CURVE COVERAGE (finite sample counts) ===");
    for w in &wells {
        let mut counts = HashMap::new();
        for col in ["gr", "res_deep", "nphi", "rhob", "dt", "sp"] {
            let c: i64 = conn
                .query_row(
                    &format!(
                        "SELECT count(*) FROM standard_curves WHERE well_id = ?1 AND NOT isnan({col})"
                    ),
                    params![w.well_id],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            counts.insert(col, c);
        }
        println!("  {:<12} {:?}", w.well_name, counts);
    }

    // ---- 2. Run every module in catalogue order -------------------------
    let db = Mutex::new(conn);
    let specs = modules::list_modules();
    println!("\n=== MODULE RUNS (default params, {} modules) ===", specs.len());
    let mut hard_errors: Vec<String> = Vec::new();
    let mut empty_outputs: Vec<String> = Vec::new();

    for spec in &specs {
        let req = RunModuleRequest {
            module: spec.name.clone(),
            well_ids: well_ids.clone(),
            log_inputs: HashMap::new(),
            params: HashMap::new(),
            opts: HashMap::new(),
            output_set: None,
            input_set: None,
        };
        let t = Instant::now();
        let runs = run_workflow_module(&db, &req);
        let elapsed = t.elapsed();

        let out_names: Vec<String> = spec
            .args
            .iter()
            .filter(|a| a.kind == ArgKind::LogOut)
            .map(|a| a.name.clone())
            .collect();

        let mut total_finite = 0i64;
        let mut per_out: Vec<String> = Vec::new();
        {
            let conn = db.lock().unwrap();
            for out in &out_names {
                let mut c = 0i64;
                for wid in &well_ids {
                    c += finite_count(&conn, wid, out);
                }
                total_finite += c;
                per_out.push(format!("{out}={c}"));
            }
        }

        let errs: Vec<String> = runs.iter().filter_map(|r| r.error.clone()).collect();
        let status = if !errs.is_empty() {
            for e in &errs {
                hard_errors.push(format!("{}: {e}", spec.name));
            }
            format!("ERROR: {}", errs[0])
        } else if total_finite == 0 && !out_names.is_empty() {
            empty_outputs.push(spec.name.clone());
            "all-NaN".into()
        } else {
            "ok".into()
        };

        println!(
            "  {:<16} [{:>9}] {:<8} {}",
            spec.name,
            format!("{:?}", elapsed),
            status,
            per_out.join(" ")
        );
    }

    // ---- 3. Pay summary --------------------------------------------------
    println!("\n=== PAY SUMMARY (VSH<=0.5 PHIE>=0.10 SWE<=0.60) ===");
    let pay_req = PaySummaryRequest {
        well_ids: well_ids.clone(),
        vsh_max: 0.5,
        phie_min: 0.10,
        swe_max: 0.60,
        perm_min: None,
        skip_version: true,
        stats_only: false,
    };
    match run_pay_summary(&db, &pay_req) {
        Ok(rows) => {
            println!("  {} summary rows", rows.len());
            for r in rows.iter().filter(|r| r.flag == "PAY").take(12) {
                println!(
                    "  {:<10} {:<8} {:<4} net={:>6.1} ntg={:.2} phie={:.3} swe={:.3} hpv={:.2}",
                    r.well_name, r.zone, r.flag, r.net, r.ntg, r.avg_phie, r.avg_swe, r.hpv
                );
            }
        }
        Err(e) => hard_errors.push(format!("pay_summary: {e}")),
    }

    // ---- 4. Render report PDF -------------------------------------------
    println!("\n=== REPORT RENDER ===");
    let spec = crate::report::ReportSpec {
        composite: CompositeSpec {
            well_id: well_ids[0].clone(),
            layout: crate::layout::standard_layout(),
            depth_top: None,
            depth_bottom: None,
            scale: 500,
            page_size: PageSize::A4,
        },
        title: "Petrophysical Evaluation — Balam South".into(),
        author: "Arshilla pipeline test".into(),
        methodology: vec![],
        vsh_max: 0.5,
        phie_min: 0.10,
        swe_max: 0.60,
        perm_min: None,
        tables_only: false,
    };
    match crate::report::render_report_pdf(&db, &spec) {
        Ok(bytes) => {
            let out = std::env::temp_dir().join("arshilla_blso_report.pdf");
            std::fs::write(&out, &bytes).ok();
            println!("  PDF {} bytes -> {}", bytes.len(), out.display());
            assert!(bytes.len() > 1000, "PDF suspiciously small");
        }
        Err(e) => hard_errors.push(format!("report: {e}")),
    }

    // ---- 5. Validation vs delivery's own curves -------------------------
    // Re-run a clean coherent chain (vsh_gr -> phi_dn -> sw_indo) so PHIE/SWE/VSH aren't
    // left holding whatever the last porosity/saturation module wrote in the sweep above.
    println!("\n=== VALIDATION vs delivered PHIE/SWE/VSH (well {}) ===", wells[0].well_name);
    for m in ["vsh_gr", "phi_dn", "sw_indo"] {
        let req = RunModuleRequest {
            module: m.into(),
            well_ids: well_ids.clone(),
            log_inputs: HashMap::new(),
            params: HashMap::new(),
            opts: HashMap::new(),
            output_set: None,
            input_set: None,
        };
        let _ = run_workflow_module(&db, &req);
    }
    {
        let conn = db.lock().unwrap();
        let wid = &well_ids[0];
        for (computed, delivered) in [("VSH", "VSH"), ("PHIE", "PHIE"), ("SWE", "SWE")] {
            let cm = computed_mean(&conn, wid, computed);
            let dm = raw_mean(&conn, wid, delivered);
            println!(
                "  {:<5} arshilla_mean={:>8.4}  delivered_mean={:>8.4}  Δ={:>8.4}",
                computed,
                cm,
                dm,
                cm - dm
            );
        }
    }

    // ---- Report problems -------------------------------------------------
    println!("\n=== ISSUES ===");
    if hard_errors.is_empty() {
        println!("  no hard errors");
    } else {
        for e in &hard_errors {
            println!("  ERROR {e}");
        }
    }
    println!("  all-NaN modules (default params — many are expected, need core/scal/targets): {empty_outputs:?}");

    assert!(hard_errors.is_empty(), "hard errors: {hard_errors:?}");
}

#[test]
#[ignore]
fn pipeline_blso_100well_stress() {
    let db_path = std::env::temp_dir().join("arshilla_blso_stress.duckdb");
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));
    let conn = crate::db::init_db(db_path.to_str().unwrap()).expect("init_db");

    // Import one real well, then read back its standard curves to clone.
    let one = vec![LAS_FILES[0].to_string()];
    let res = crate::ingest::import_las_files(&conn, &one, None);
    let src_id = res[0].well_id.clone().expect("import");

    let mut depth = Vec::new();
    let mut gr = Vec::new();
    let mut rd = Vec::new();
    let mut np = Vec::new();
    let mut rb = Vec::new();
    let mut dt = Vec::new();
    let mut sp = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT depth, gr, res_deep, nphi, rhob, dt, sp FROM standard_curves WHERE well_id = ?1 ORDER BY depth")
            .unwrap();
        let rows = stmt
            .query_map(params![src_id], |r| {
                Ok((
                    r.get::<_, f32>(0)?,
                    r.get::<_, f32>(1)?,
                    r.get::<_, f32>(2)?,
                    r.get::<_, f32>(3)?,
                    r.get::<_, f32>(4)?,
                    r.get::<_, f32>(5)?,
                    r.get::<_, f32>(6)?,
                ))
            })
            .unwrap();
        for row in rows {
            let (a, b, c, d, e, f, g) = row.unwrap();
            depth.push(a);
            gr.push(b);
            rd.push(c);
            np.push(d);
            rb.push(e);
            dt.push(f);
            sp.push(g);
        }
    }
    let samples_per_well = depth.len();
    const N_WELLS: usize = 100;

    let t0 = Instant::now();
    let mut ids = Vec::with_capacity(N_WELLS);
    for i in 0..N_WELLS {
        let id = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, id, &format!("BLSO_DUP_{i:03}"), Some("Balam South"), None, None).unwrap();
        crate::db::insert_standard_curves(
            &conn,
            id,
            depth.clone(),
            gr.clone(),
            rd.clone(),
            np.clone(),
            rb.clone(),
            dt.clone(),
            sp.clone(),
        )
        .unwrap();
        ids.push(id.to_string());
    }
    println!(
        "\n=== 100-WELL STRESS: built {} wells × {} samples = {} samples in {:?} ===",
        N_WELLS,
        samples_per_well,
        N_WELLS * samples_per_well,
        t0.elapsed()
    );
    println!("  rayon threads available: {}", rayon::current_num_threads());

    let db = Mutex::new(conn);
    // A representative interpretation chain (each call fans across all 100 wells via rayon).
    let chain = ["vsh_gr", "phi_dn", "sw_indo", "perm_wyllie_rose"];
    let mut grand = std::time::Duration::ZERO;
    for m in chain {
        let req = RunModuleRequest {
            module: m.into(),
            well_ids: ids.clone(),
            log_inputs: HashMap::new(),
            params: HashMap::new(),
            opts: HashMap::new(),
            output_set: None,
            input_set: None,
        };
        let t = Instant::now();
        let runs = run_workflow_module(&db, &req);
        let el = t.elapsed();
        grand += el;
        let errs = runs.iter().filter(|r| r.error.is_some()).count();
        println!(
            "  {:<18} {:?}  ({:.0} wells/s, {} errors)",
            m,
            el,
            N_WELLS as f64 / el.as_secs_f64(),
            errs
        );
    }
    let total_samples = (N_WELLS * samples_per_well * chain.len()) as f64;
    println!(
        "  chain total {:?} → {:.1}M sample-evals/s",
        grand,
        total_samples / grand.as_secs_f64() / 1e6
    );

    // Pay summary across all 100 wells.
    let t = Instant::now();
    let pay = run_pay_summary(
        &db,
        &PaySummaryRequest { well_ids: ids.clone(), vsh_max: 0.5, phie_min: 0.10, swe_max: 0.60, perm_min: None, skip_version: true, stats_only: false },
    );
    println!("  pay_summary(100 wells) {:?} → {} rows", t.elapsed(), pay.map(|r| r.len()).unwrap_or(0));

    // ---- Write-cost probe: the control that justified dropping the PK ---
    // Same row volume appended into (a) a table carrying the OLD 3-column PRIMARY KEY
    // (well_id, depth, curve_name) whose ART uniqueness index is updated on every inserted
    // row, versus (b) a PK-less table. `computed_curves` itself is now PK-less (see
    // db::create_schema + migrate_drop_computed_curves_pk) precisely because (b) is far
    // faster — this probe stays as a standing regression control on that decision.
    {
        let conn = db.lock().unwrap();
        let probe_wells = 20usize;
        let rows = probe_wells * depth.len();

        // Two identical staging tables — the only difference is the legacy 3-column PRIMARY
        // KEY that computed_curves used to carry.
        conn.execute_batch(
            "CREATE TABLE stage_pk   (well_id TEXT, depth REAL, curve_name TEXT, value REAL, PRIMARY KEY (well_id, depth, curve_name));
             CREATE TABLE stage_nopk (well_id TEXT, depth REAL, curve_name TEXT, value REAL);",
        )
        .unwrap();

        let t = Instant::now();
        {
            let mut ap = conn.appender("stage_pk").unwrap();
            for wi in 0..probe_wells {
                let wid = format!("probe-{wi}");
                for (d, v) in depth.iter().zip(gr.iter()) {
                    ap.append_row(params![wid, d, "PROBE", v]).unwrap();
                }
            }
            ap.flush().unwrap();
        }
        let pk_time = t.elapsed();

        let t = Instant::now();
        {
            let mut ap = conn.appender("stage_nopk").unwrap();
            for wi in 0..probe_wells {
                let wid = format!("probe-{wi}");
                for (d, v) in depth.iter().zip(gr.iter()) {
                    ap.append_row(params![wid, d, "PROBE", v]).unwrap();
                }
            }
            ap.flush().unwrap();
        }
        let nopk_time = t.elapsed();

        println!(
            "\n=== WRITE-COST PROBE ({} rows, single thread) ===\n  legacy 3-col PK table           : {:?}  ({:.0}k rows/s)\n  PK-less table (now computed_curves): {:?}  ({:.0}k rows/s)\n  → PK index overhead ~{:.1}x (eliminated)",
            rows,
            pk_time,
            rows as f64 / pk_time.as_secs_f64() / 1e3,
            nopk_time,
            rows as f64 / nopk_time.as_secs_f64() / 1e3,
            pk_time.as_secs_f64() / nopk_time.as_secs_f64().max(1e-9),
        );
    }
}
