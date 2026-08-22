//! **Pass 1 of the performance brief: the instrument.** It measures and changes nothing.
//!
//! Run it explicitly - it is `#[ignore]`d, so the green gate never waits on it:
//!
//! ```text
//! cargo test --release perf_baseline -- --ignored --nocapture
//! ```
//!
//! `SANDIBUMI_PERF_WELLS` (default 10) and `SANDIBUMI_PERF_SAMPLES` (default 1562, the sample
//! count of a real logged well on this machine) set the size. Pass 2 drives the same harness at
//! 10 / 100 / larger to find where a number stops being linear.
//!
//! ## Why it builds its own wells
//!
//! `pipeline_field_test.rs` already prints timings, and it is the reason this file exists rather
//! than an extension of it. Two problems:
//!
//! 1. It needs `SANDIBUMI_FIELD_FIXTURES` - a real delivery - so on a fresh clone it prints
//!    nothing at all. A baseline nobody else can reproduce is not a baseline.
//! 2. **It reports timings for work that did not happen.** Measured 2026-08-22, before this file
//!    was written: `pipeline_field_100well_stress` printed a 59.4 s chain in which every one of
//!    the 400 module runs returned an error, and a pay summary of 0 rows - and passed, because it
//!    asserts nothing about them. A stopwatch on a failing operation times the failure.
//!
//! So this harness generates deterministic synthetic wells (no fixture, no client identifier, the
//! `SANDI-*` convention), and **counts errors on every operation and prints the first one**. A
//! number here is only reported beside the count of runs that actually produced an answer.
//!
//! ## What a number here does and does not include
//!
//! These are BACKEND timings: the work behind a click, not the click. A user's click-to-paint is
//! this, plus Tauri's IPC hop, plus the canvas paint. The IPC leg is not measurable from a unit
//! test and the paint is measured separately in the browser, so **every figure here is a lower
//! bound on what the user feels**, and the report says so rather than implying otherwise.

use crate::composite::{CompositeSpec, PageSize};
use crate::equations::TrackCurveRequest;
use crate::paysummary::{run_pay_summary, PaySummaryRequest};
use crate::workflow::{run_workflow_module, RunModuleRequest};
use duckdb::Connection;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Wells built when `SANDIBUMI_PERF_WELLS` is unset. Ten is enough to see per-well cost without
/// making the default run long enough that nobody runs it.
const DEFAULT_WELLS: usize = 10;
/// Samples per well when `SANDIBUMI_PERF_SAMPLES` is unset. 1562 is what a real logged well on
/// this machine carries, so the default size is not an invented one.
const DEFAULT_SAMPLES: usize = 1562;
/// The app's standard half-foot metric grid, same constant `tools/make_example_data.py` uses.
const STEP: f32 = 0.1524;
const TOP_DEPTH: f32 = 1500.0;

fn env_usize(key: &str, fallback: usize) -> usize {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(fallback)
}

// ---------------------------------------------------------------------------------------------
// Deterministic synthetic curves. Same shape as the example generator: a seeded LCG, smoothed,
// blended between zones, so the numbers are plausible logs rather than noise - a module that
// refuses implausible input would otherwise be measured refusing rather than computing.
// ---------------------------------------------------------------------------------------------

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) as f32) / (u32::MAX as f32 / 2.0) - 1.0
    }
}

struct SyntheticWell {
    depth: Vec<f32>,
    gr: Vec<f32>,
    res_deep: Vec<f32>,
    nphi: Vec<f32>,
    rhob: Vec<f32>,
    dt: Vec<f32>,
    sp: Vec<f32>,
}

fn synthetic_well(seed: u64, samples: usize) -> SyntheticWell {
    let mut rng = Lcg(seed.wrapping_mul(2_654_435_761).wrapping_add(1));
    let mut well = SyntheticWell {
        depth: Vec::with_capacity(samples),
        gr: Vec::with_capacity(samples),
        res_deep: Vec::with_capacity(samples),
        nphi: Vec::with_capacity(samples),
        rhob: Vec::with_capacity(samples),
        dt: Vec::with_capacity(samples),
        sp: Vec::with_capacity(samples),
    };
    // A slow shaliness cycle so zones exist and cut-offs actually divide the well; without one,
    // every sample lands on the same side of every cut-off and the pay summary measures nothing.
    for i in 0..samples {
        let depth = TOP_DEPTH + STEP * i as f32;
        let cycle = ((i as f32) / 140.0).sin() * 0.5 + 0.5; // 0 = clean sand, 1 = shale
        well.depth.push(depth);
        well.gr.push(25.0 + 105.0 * cycle + rng.next() * 4.0);
        // Clean sand is resistive, shale is conductive.
        well.res_deep.push((60.0 * (1.0 - cycle) + 1.5).max(0.4) + rng.next() * 0.4);
        well.nphi.push(0.12 + 0.30 * cycle + rng.next() * 0.01);
        well.rhob.push(2.20 + 0.35 * cycle + rng.next() * 0.02);
        well.dt.push(95.0 - 20.0 * cycle + rng.next() * 1.5);
        well.sp.push(-70.0 + 60.0 * cycle + rng.next() * 2.0);
    }
    well
}

// ---------------------------------------------------------------------------------------------
// Timing
// ---------------------------------------------------------------------------------------------

/// One operation's timings. The MEDIAN is reported rather than the mean because a single
/// scheduler hiccup drags a mean and leaves a median alone, and min/max ride along so a reader
/// can see the spread instead of trusting one number.
struct Timing {
    label: &'static str,
    runs: Vec<Duration>,
    /// What the operation actually produced. A duration with nothing beside it cannot be
    /// distinguished from the time taken to fail - which is exactly how the existing stress test
    /// reported a 59 s chain of 400 errors.
    produced: String,
}

impl Timing {
    fn median(&self) -> Duration {
        let mut sorted = self.runs.clone();
        sorted.sort();
        sorted[sorted.len() / 2]
    }
    fn min(&self) -> Duration {
        *self.runs.iter().min().expect("at least one run")
    }
    fn max(&self) -> Duration {
        *self.runs.iter().max().expect("at least one run")
    }
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

/// Runs `f` `reps` times, keeping every duration. `f` returns what it produced, for the record.
fn bench<F>(label: &'static str, reps: usize, mut f: F) -> Timing
where
    F: FnMut() -> String,
{
    let mut runs = Vec::with_capacity(reps);
    let mut produced = String::new();
    for _ in 0..reps {
        let t = Instant::now();
        produced = f();
        runs.push(t.elapsed());
    }
    Timing { label, runs, produced }
}

fn print_table(title: &str, rows: &[Timing]) {
    println!("\n{title}");
    println!("{:<34} {:>10} {:>10} {:>10}  {}", "OPERATION", "MEDIAN", "MIN", "MAX", "PRODUCED");
    for row in rows {
        println!(
            "{:<34} {:>9.1}ms {:>9.1}ms {:>9.1}ms  {}",
            row.label,
            ms(row.median()),
            ms(row.min()),
            ms(row.max()),
            row.produced
        );
    }
}

fn chain_params(module: &str) -> HashMap<String, f64> {
    // Same values `pipeline_field_test::generic_chain_params` uses. They are a RUN CONFIGURATION
    // for a timing harness, not a recommended interpretation, and they are duplicated rather than
    // shared so that changing one harness cannot silently change the other's numbers.
    let pairs: &[(&str, f64)] = match module {
        "vsh_gr" => &[("GR_MA", 25.0), ("GR_SH", 130.0)],
        "phi_den" => &[("RHO_SH", 2.5)],
        "sw_indo" => &[
            ("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.2), ("RT_SH", 4.0), ("SWE_IRR", 0.0),
        ],
        "perm_wyllie_rose" => &[("SWE_IRR", 0.15)],
        _ => &[],
    };
    pairs.iter().map(|(k, v)| ((*k).to_string(), *v)).collect()
}

fn module_request(module: &str, well_ids: &[String]) -> RunModuleRequest {
    RunModuleRequest {
        module: module.into(),
        well_ids: well_ids.to_vec(),
        log_inputs: HashMap::new(),
        params: chain_params(module),
        opts: HashMap::new(),
        output_set: None,
        input_set: None,
        custody: crate::workflow::test_run_custody(),
    }
}

/// Runs one module across `well_ids` and reports how many wells produced an answer, naming the
/// first error if any did not. The naming is the point: a silent error count is how a harness
/// ends up timing failures.
fn run_and_count(db: &Mutex<Connection>, module: &str, well_ids: &[String]) -> String {
    let runs = run_workflow_module(db, &module_request(module, well_ids));
    let failed: Vec<&String> = runs.iter().filter_map(|r| r.error.as_ref()).collect();
    if failed.is_empty() {
        format!("{}/{} wells ok", runs.len(), well_ids.len())
    } else {
        format!("{}/{} FAILED - first: {}", failed.len(), runs.len(), failed[0])
    }
}

fn cutoff(value: f64) -> crate::paysummary::CutoffSpec {
    crate::paysummary::CutoffEntry { value, unit: "v/v".into() }.into()
}

#[test]
#[ignore]
fn perf_baseline() {
    let n_wells = env_usize("SANDIBUMI_PERF_WELLS", DEFAULT_WELLS);
    let n_samples = env_usize("SANDIBUMI_PERF_SAMPLES", DEFAULT_SAMPLES);

    let db_path = std::env::temp_dir().join(format!("sandibumi_perf_{n_wells}w_{n_samples}s.duckdb"));
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));

    println!("\n================ SandiBumi performance baseline ================");
    println!("wells {n_wells} x {n_samples} samples = {} samples", n_wells * n_samples);
    println!("rayon threads: {}", rayon::current_num_threads());
    println!("build: {}", if cfg!(debug_assertions) { "DEBUG (unoptimised)" } else { "release" });

    // ---- build the project ------------------------------------------------------------------
    let mut well_ids = Vec::with_capacity(n_wells);
    let build = {
        let conn = crate::db::init_db(db_path.to_str().unwrap()).expect("init_db");
        let t = Instant::now();
        for i in 0..n_wells {
            let id = uuid::Uuid::new_v4();
            crate::db::insert_well(
                &conn,
                id,
                &format!("SANDI-{:04}", i + 1),
                Some("Sandi synthetic"),
                None,
                None,
            )
            .expect("insert_well");
            let w = synthetic_well(i as u64, n_samples);
            crate::db::insert_standard_curves(&conn, id, w.depth, w.gr, w.res_deep, w.nphi, w.rhob, w.dt, w.sp)
                .expect("insert_standard_curves");
            well_ids.push(id.to_string());
        }
        t.elapsed()
        // conn dropped here, so the cold open below really is cold
    };
    println!("build (setup, not an app operation): {:.1}ms", ms(build));

    // ---- COLD OPEN --------------------------------------------------------------------------
    // Measured on its own connection and reported before anything warms a cache. This is the one
    // operation a user waits on before the window is usable.
    let open = bench("cold project open", 3, || {
        let conn = crate::project::open_and_migrate(db_path.to_str().unwrap()).expect("open");
        let wells: i64 = conn.query_row("SELECT COUNT(*) FROM wells", [], |r| r.get(0)).unwrap_or(-1);
        format!("{wells} wells")
    });
    print_table("== OPEN ==", &[open]);

    let conn = crate::project::open_and_migrate(db_path.to_str().unwrap()).expect("open");
    let db = Mutex::new(conn);

    // ---- READ PATH: what a click costs the backend -------------------------------------------
    let standard = ["GR", "RES_DEEP", "NPHI", "RHOB", "DT", "SP"];
    let requests: Vec<TrackCurveRequest> = standard
        .iter()
        .map(|c| TrackCurveRequest { curve_name: (*c).to_string(), set_name: None, class_curve: false })
        .collect();
    let first = well_ids[0].clone();
    let depth_span = n_samples as f32 * STEP;

    let mut reads = Vec::new();
    reads.push(bench("curve catalog (every plot opens)", 10, || {
        let conn = db.lock().unwrap();
        let entries = crate::equations::list_curve_catalog(&conn).expect("catalog");
        format!("{} curves", entries.len())
    }));
    reads.push(bench("well switch: log view, 6 curves", 10, || {
        let conn = db.lock().unwrap();
        let series =
            crate::equations::fetch_track_data(&conn, &first, &requests, 1000, None, None).expect("track");
        format!("{} series", series.len())
    }));
    reads.push(bench("log scroll: 10% depth window", 10, || {
        let conn = db.lock().unwrap();
        let series = crate::equations::fetch_track_data(
            &conn,
            &first,
            &requests,
            1000,
            Some(TOP_DEPTH),
            Some(TOP_DEPTH + depth_span * 0.1),
        )
        .expect("track");
        format!("{} series", series.len())
    }));
    reads.push(bench("log zoom: 1% depth window", 10, || {
        let conn = db.lock().unwrap();
        let series = crate::equations::fetch_track_data(
            &conn,
            &first,
            &requests,
            1000,
            Some(TOP_DEPTH),
            Some(TOP_DEPTH + depth_span * 0.01),
        )
        .expect("track");
        format!("{} series", series.len())
    }));
    reads.push(bench("plot data: 2 curves, FULL res", 10, || {
        let conn = db.lock().unwrap();
        let series = crate::equations::fetch_curve_data(
            &conn,
            &first,
            &["NPHI".to_string(), "RHOB".to_string()],
            None,
            None,
        )
        .expect("curve data");
        let n: usize = series.iter().map(|s| s.point_count).sum();
        format!("{n} values")
    }));
    reads.push(bench("plot data: ALL wells, 2 curves", 3, || {
        let conn = db.lock().unwrap();
        let mut total = 0usize;
        for id in &well_ids {
            let series = crate::equations::fetch_curve_data(
                &conn,
                id,
                &["NPHI".to_string(), "RHOB".to_string()],
                None,
                None,
            )
            .expect("curve data");
            total += series.iter().map(|s| s.point_count).sum::<usize>();
        }
        format!("{total} values")
    }));
    print_table("== READ PATH (backend half of a click) ==", &reads);

    // ---- WRITE PATH: modules and chains ------------------------------------------------------
    let chain = ["vsh_gr", "phi_den", "sw_indo", "perm_wyllie_rose"];
    let mut writes = Vec::new();
    let one = vec![first.clone()];
    writes.push(bench("module vsh_gr, 1 well", 3, || run_and_count(&db, "vsh_gr", &one)));
    for module in chain {
        let ids = well_ids.clone();
        let label: &'static str = match module {
            "vsh_gr" => "chain 1/4 vsh_gr, all wells",
            "phi_den" => "chain 2/4 phi_den, all wells",
            "sw_indo" => "chain 3/4 sw_indo, all wells",
            _ => "chain 4/4 perm_wyllie_rose, all",
        };
        writes.push(bench(label, 1, || run_and_count(&db, module, &ids)));
    }
    print_table("== WRITE PATH (module runs) ==", &writes);

    // ---- DERIVED VIEWS ------------------------------------------------------------------------
    let mut derived = Vec::new();
    derived.push(bench("field dashboard (pay summary)", 3, || {
        let result = run_pay_summary(
            &db,
            &PaySummaryRequest {
                well_ids: well_ids.clone(),
                vsh_max: Some(cutoff(0.5)),
                phie_min: Some(cutoff(0.10)),
                swe_max: Some(cutoff(0.60)),
                perm_min: None,
                input_set: None,
                skip_version: false,
                // stats_only: the dashboard never writes FLAG curves, and timing the write here
                // would measure a different operation than the one the button performs.
                stats_only: true,
                enabled_unset: Vec::new(),
                discretisation: crate::paysummary::DiscretisationModel::Forward,
                cutoff_use: Default::default(),
                custody: Some(crate::workflow::test_run_custody()),
                frame: Default::default(),
                weighting: Default::default(),
            },
        );
        match result {
            Ok(rows) => format!("{} rows", rows.len()),
            Err(e) => format!("FAILED - {e}"),
        }
    }));
    derived.push(bench("report render, 1 well", 3, || {
        let spec = crate::report::ReportSpec {
            input_set: None,
            custody: Some(crate::workflow::test_run_custody()),
            composite: CompositeSpec {
                well_id: first.clone(),
                layout: crate::layout::standard_layout(),
                depth_top: None,
                depth_bottom: None,
                scale: 500,
                page_size: PageSize::A4,
            },
            title: "Performance baseline".into(),
            author: "perf harness".into(),
            methodology: vec![],
            vsh_max: Some(cutoff(0.5)),
            phie_min: Some(cutoff(0.10)),
            swe_max: Some(cutoff(0.60)),
            perm_min: None,
            tables_only: false,
        };
        match crate::report::render_report(&db, &spec) {
            Ok(result) => format!("{} pages", result.pages.len()),
            Err(e) => format!("FAILED - {e}"),
        }
    }));
    print_table("== DERIVED VIEWS ==", &derived);

    println!(
        "\nNOTE: backend only. A user's click-to-paint is this plus Tauri IPC plus the canvas\n\
         paint, so every figure above is a LOWER BOUND on what is felt.\n"
    );

    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));
}
