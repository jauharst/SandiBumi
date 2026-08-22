//! **Pass 1 of the performance brief: the instrument.** It measures and changes nothing.
//!
//! Run it explicitly - it is `#[ignore]`d, so the green gate never waits on it:
//!
//! ```text
//! cargo test --release --lib perf_baseline_test::perf_baseline -- --exact --ignored --nocapture
//! ```
//!
//! **`--exact` is not decoration.** A bare `perf_baseline` filter is a SUBSTRING match against
//! the full test path, so it also matches every other test in the module `perf_baseline_test` -
//! and cargo runs them CONCURRENTLY. Measured 2026-08-23: that put the 100-well chain at ~83 s
//! against ~36 s for the same work run alone. The rows look completely normal either way.
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

/// Builds a synthetic project on disk and returns its well ids plus the build duration. Shared by
/// both harnesses so they measure the same field rather than two similar ones.
fn build_project(
    db_path: &std::path::Path,
    n_wells: usize,
    n_samples: usize,
) -> (Vec<String>, std::time::Duration) {
    let mut well_ids = Vec::with_capacity(n_wells);
    let conn = crate::db::init_db(db_path.to_str().unwrap()).expect("init_db");
    let t = Instant::now();
    for i in 0..n_wells {
        let id = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, id, &format!("SANDI-{:04}", i + 1), Some("Sandi synthetic"), None, None)
            .expect("insert_well");
        let w = synthetic_well(i as u64, n_samples);
        crate::db::insert_standard_curves(&conn, id, w.depth, w.gr, w.res_deep, w.nphi, w.rhob, w.dt, w.sp)
            .expect("insert_standard_curves");
        well_ids.push(id.to_string());
    }
    let elapsed = t.elapsed();
    // conn dropped by the caller's scope end, so a cold open afterwards really is cold
    drop(conn);
    (well_ids, elapsed)
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
    let (well_ids, build) = build_project(&db_path, n_wells, n_samples);
    println!("build (setup, not an app operation): {:.1}ms", ms(build));

    // ---- OPEN -------------------------------------------------------------------------------
    // TWO operations, not one, and reporting them under a single label hid the interesting half.
    // `bench` times every repetition with no warm-up, so the FIRST open pays for a cold OS file
    // cache and unread DuckDB metadata while every later one does not. At 500 wells that was
    // 32.8s against a 130ms median: the median under a row labelled "cold" was a WARM re-open,
    // and the cold number - the one a user waits on after launching the app - was hiding in MAX.
    // The cold open scales with well count; the warm re-open barely moves. They are separate rows.
    let open_n = |label: &'static str, reps: usize| {
        bench(label, reps, || {
            let conn = crate::project::open_and_migrate(db_path.to_str().unwrap()).expect("open");
            let wells: i64 =
                conn.query_row("SELECT COUNT(*) FROM wells", [], |r| r.get(0)).unwrap_or(-1);
            format!("{wells} wells")
        })
    };
    // Order matters: the cold one must be FIRST, because it is only cold once.
    let cold = open_n("first project open (COLD)", 1);
    let warm = open_n("project re-open (warm)", 3);
    print_table("== OPEN ==", &[cold, warm]);

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

// ============================================================================================
// WHERE THE TIME GOES - pass 3 increment 1. It DIAGNOSES and changes nothing.
//
// Passes 1 and 2 measured that a chain over N wells costs N times one well, even though
// `workflow.rs` hands the wells to `par_iter` and this machine reports 32 rayon threads. That
// established the SYMPTOM. It did not establish the CAUSE, and fixing before the cause is known
// is the guessing this whole brief exists to avoid.
//
// The suspect is the single `Mutex<Connection>`: the per-well closure takes that lock four
// separate times, once for input resolution (which is also the largest read), once for the run
// mask, once for the neutron-basis declaration, and once to write. `SB-CORE-032` predicts exactly
// this - "no operation whose duration scales with well count or sample count may hold the global
// database mutex for its duration" - and is already recorded PRESENT-DIVERGENT.
//
// So: read the same curves for every well three ways, changing only HOW THE CONNECTION IS SHARED.
//
//   A  serial, one shared connection            - the reference
//   B  PARALLEL, one shared connection          - what the app does today
//   C  PARALLEL, one connection per thread      - what #129 (connection pool) would do
//
// B/A near 1.0 proves the lock serialises the parallel loop. C/A is then the measured answer to
// the brief's "measure whether #129 is worth doing" - WITHOUT implementing it, because
// `Connection::try_clone` gives a second handle on the already-open database, which is the same
// primitive a pool would be built from.
//
// This is a read-only experiment on purpose. Writes are genuinely serial in DuckDB (one writer is
// fundamental), so a read that refuses to parallelise is the half that could be fixed, and the
// half worth measuring first.

#[test]
#[ignore]
fn perf_where_time_goes() {
    use rayon::prelude::*;

    let n_wells = env_usize("SANDIBUMI_PERF_WELLS", DEFAULT_WELLS);
    let n_samples = env_usize("SANDIBUMI_PERF_SAMPLES", DEFAULT_SAMPLES);
    let threads = rayon::current_num_threads();

    let db_path = std::env::temp_dir().join(format!("sandibumi_lock_{n_wells}w_{n_samples}s.duckdb"));
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));

    println!("\n========= WHERE THE TIME GOES: is the shared connection the bottleneck? =========");
    println!("wells {n_wells} x {n_samples} samples ; rayon threads: {threads}");
    println!("build: {}", if cfg!(debug_assertions) { "DEBUG (unoptimised)" } else { "release" });

    let (well_ids, _) = build_project(&db_path, n_wells, n_samples);
    let curves: Vec<String> = ["GR", "RES_DEEP", "NPHI", "RHOB", "DT", "SP"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    let base = crate::project::open_and_migrate(db_path.to_str().unwrap()).expect("open");

    let db = Mutex::new(base);

    let chunk = n_wells.div_ceil(threads).max(1);
    let read_all = |conn: &Connection, wells: &[String]| -> usize {
        wells
            .iter()
            .map(|w| crate::equations::fetch_curve_frame(conn, w, &curves).map(|f| f.0.len()).unwrap_or(0))
            .sum()
    };

    // A: serial, through the shared lock.
    let serial = |()| -> (std::time::Duration, usize) {
        let t = Instant::now();
        let conn = db.lock().unwrap();
        let n = read_all(&conn, &well_ids);
        (t.elapsed(), n)
    };
    // B: parallel, through the shared lock - today's behaviour.
    let parallel_shared = |()| -> (std::time::Duration, usize) {
        let t = Instant::now();
        let n: usize = well_ids
            .par_iter()
            .map(|w| {
                let conn = db.lock().unwrap();
                crate::equations::fetch_curve_frame(&conn, w, &curves).map(|f| f.0.len()).unwrap_or(0)
            })
            .sum();
        (t.elapsed(), n)
    };
    // C: parallel, one connection per thread - no shared lock at all.
    //
    // `Connection` is `Send` but deliberately NOT `Sync` (it owns a `RefCell`), which is precisely
    // why the app wraps one in a `Mutex` in the first place. So the connections are MOVED into the
    // worker threads, one each, rather than shared by reference - which is also what a pool is.
    // The clones are made OUTSIDE the timer: a real pool builds them once per session, not per
    // read, so charging them here would understate the benefit.
    let parallel_pooled = || -> (std::time::Duration, usize) {
        let work: Vec<(Connection, Vec<String>)> = {
            let base = db.lock().unwrap();
            well_ids
                .chunks(chunk)
                .map(|wells| (base.try_clone().expect("try_clone"), wells.to_vec()))
                .collect()
        };
        let t = Instant::now();
        let n: usize = work.into_par_iter().map(|(conn, wells)| read_all(&conn, &wells)).sum();
        (t.elapsed(), n)
    };

    // Warm-up pass, discarded: the first read of a freshly built file pays for a cold OS cache,
    // and charging that to whichever variant happened to run first would decide the answer.
    let _ = serial(());
    let _ = parallel_shared(());
    let _ = parallel_pooled();

    let (t_serial, n_serial) = serial(());
    let (t_shared, n_shared) = parallel_shared(());
    let (t_pooled, n_pooled) = parallel_pooled();

    // Every variant must have read the same data, or the comparison is between different jobs.
    assert_eq!(n_serial, n_shared, "shared-lock parallel read returned a different sample count");
    assert_eq!(n_serial, n_pooled, "pooled parallel read returned a different sample count");

    let a = ms(t_serial);
    let b = ms(t_shared);
    let c = ms(t_pooled);
    println!("\nReading {} curves for all {n_wells} wells ({n_serial} samples), three ways:\n", curves.len());
    println!("{:<44} {:>10} {:>12}", "VARIANT", "TIME", "SPEED-UP");
    println!("{:<44} {:>9.1}ms {:>12}", "A  serial, one shared connection", a, "1.00x");
    println!("{:<44} {:>9.1}ms {:>11.2}x", "B  PARALLEL, one shared connection (today)", b, a / b);
    println!("{:<44} {:>9.1}ms {:>11.2}x", "C  PARALLEL, one connection per thread (#129)", c, a / c);

    println!(
        "\n{}",
        if a / b < 1.5 {
            "B is no faster than A: the shared connection SERIALISES the parallel loop."
        } else {
            "B is meaningfully faster than A: the shared connection is NOT the read bottleneck."
        }
    );
    println!(
        "C is {:.2}x variant B, which is what a connection pool would buy on the READ half.\n\
         Writes stay serial regardless - DuckDB is single-writer - so this is an upper bound on\n\
         the read side only, not on a whole module run.",
        b / c
    );

    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));
}

// ============================================================================================
// THE READ/WRITE SPLIT - pass 3 increment 2. It also changes nothing.
//
// Increment 1 proved the shared connection serialises parallel reads, and measured that one
// connection per thread would buy 4-8x on the read IT timed. It could not say how much of a real
// module run that read is, so it could not turn "4-8x on reads" into "X minutes off a 23-minute
// chain". This measures the split, using `lock_probe` (see that module for why the probe is in
// production code and why it cannot reach a shipped build).
//
// FOUR MODULES, NOT ONE AND NOT ALL SIXTY-TWO. The split is driven by how many curves a module
// READS and WRITES, not by its arithmetic, so one module cannot speak for the rest - and pass 2
// measured a 4.2x spread in per-well cost across exactly these four. They are also the workload
// every other number in this brief was taken on, so the answer lands against the 23-minute chain
// directly instead of having to be transferred to it. If the four agree, that generalises with
// evidence; if they disagree, the disagreement is the finding.

#[test]
#[ignore]
fn perf_read_write_split() {
    let n_wells = env_usize("SANDIBUMI_PERF_WELLS", DEFAULT_WELLS);
    let n_samples = env_usize("SANDIBUMI_PERF_SAMPLES", DEFAULT_SAMPLES);

    let db_path = std::env::temp_dir().join(format!("sandibumi_split_{n_wells}w_{n_samples}s.duckdb"));
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));

    println!("\n============ THE READ/WRITE SPLIT INSIDE A MODULE RUN ============");
    println!("wells {n_wells} x {n_samples} samples ; rayon threads: {}", rayon::current_num_threads());
    println!("build: {}", if cfg!(debug_assertions) { "DEBUG (unoptimised)" } else { "release" });

    let (well_ids, _) = build_project(&db_path, n_wells, n_samples);
    let conn = crate::project::open_and_migrate(db_path.to_str().unwrap()).expect("open");
    let db = Mutex::new(conn);

    println!(
        "\n{:<18} {:>9} {:>11} {:>11} {:>11} {:>11}  {}",
        "MODULE", "WALL", "QUEUE", "READ", "COMPUTE", "WRITE", "PRODUCED"
    );

    let mut totals = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for module in ["vsh_gr", "phi_den", "sw_indo", "perm_wyllie_rose"] {
        crate::lock_probe::reset();
        let t = Instant::now();
        let produced = run_and_count(&db, module, &well_ids);
        let wall = ms(t.elapsed());
        let (wait_ns, read_ns, write_ns, well_ns) = crate::lock_probe::snapshot();

        // COMPUTE is per-well work minus the read scopes inside it. The write is batched AFTER the
        // parallel loop (one DELETE plus one Appender for every well - the phase 9 write-path
        // work), so it is deliberately not inside the per-well total and is added separately.
        let queue = wait_ns as f64 / 1e6;
        let read = read_ns as f64 / 1e6;
        let write = write_ns as f64 / 1e6;
        let compute = (well_ns as f64 / 1e6 - read - queue).max(0.0);

        println!(
            "{:<18} {:>8.1}ms {:>10.1}ms {:>10.1}ms {:>10.1}ms {:>10.1}ms  {}",
            module, wall, queue, read, compute, write, produced
        );
        totals.0 += wall;
        totals.1 += queue;
        totals.2 += read;
        totals.3 += compute;
        totals.4 += write;
    }

    let (wall, queue, read, compute, write) = totals;
    let accounted = queue + read + compute + write;
    println!(
        "{:<18} {:>8.1}ms {:>10.1}ms {:>10.1}ms {:>10.1}ms {:>10.1}ms  4-module chain",
        "TOTAL", wall, queue, read, compute, write
    );
    println!(
        "\nShare of the work actually accounted for ({:.1}ms of {:.1}ms wall, {:.0}%):",
        accounted, wall, 100.0 * accounted / wall
    );
    println!("  QUEUE   {:>5.1}%   <- waiting for the shared connection; a pool DELETES this", 100.0 * queue / accounted);
    println!("  READ    {:>5.1}%   <- lock held, reading; a pool PARALLELISES this", 100.0 * read / accounted);
    println!("  COMPUTE {:>5.1}%   <- already parallel; rayon keeps this", 100.0 * compute / accounted);
    println!("  WRITE   {:>5.1}%   <- serial regardless; DuckDB is single-writer by design", 100.0 * write / accounted);

    // Increment 1 measured a 4-8x speed-up on the read phase. Amdahl's law then bounds what the
    // whole chain can gain: the read shrinks, everything else does not.
    // Amdahl's law, expressed in WALL CLOCK - the only clock anyone waits on. A pool removes the
    // QUEUE outright and divides the READ; COMPUTE and WRITE do not move, and the write is already
    // one batched single-threaded transaction, so its summed time IS its wall time.
    //
    // Quoting the ratio against `accounted` instead would report ~14x, because blocked threads
    // inflate the summed total. That is a real number about thread-seconds and a false one about
    // how long a chain takes.
    println!("
What a connection pool would buy, in WALL CLOCK:");
    for factor in [4.0f64, 8.0] {
        let after = read / factor + compute + write;
        println!(
            "  queue gone, reads {:.0}x faster: {:.1}s -> {:.1}s  ({:.2}x)  - of which write {:.1}s ({:.0}%)",
            factor,
            wall / 1000.0,
            after / 1000.0,
            wall / after,
            write / 1000.0,
            100.0 * write / after
        );
    }
    println!(
        "  The WRITE is the floor. It is one batched transaction and DuckDB is single-writer by
           design, so no change to connection semantics moves it."
    );
    println!(
        "\nThe phases are SUMS across wells and threads, not wall-clock. While the loop is\n\
         serialised the sum tracks the wall-clock, which is the check on the probe; once a pool\n\
         lands they must diverge, and that divergence is the parallelism working."
    );

    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));
}

// ---------------------------------------------------------------------------------------------
// PASS 3 INCREMENT 4: where the Field Dashboard's time goes, and why it gets worse per well.
// ---------------------------------------------------------------------------------------------
// Pass 2 measured the dashboard as the fastest-degrading operation in the sweep: 5.7 ms per well
// in a 10-well project and 24.5 ms per well in a 2000-well one, exponent 1.61 over the last
// segment. Nothing about a well changes when other wells are added, so a per-well cost that rises
// with the project is the finding, and this attributes it to a NAMED phase.
//
// It measures FROM OUTSIDE. `run_pay_summary`'s per-well loop takes the lock once and makes four
// reads under it; every one of those functions is reachable from this crate, so the sequence is
// replayed here rather than bracketed inside paysummary.rs. Nothing in the production path is
// touched - not even a #[cfg(test)] statement.
//
// The honest caveat, stated rather than buried: the replay runs AFTER a discarded dashboard pass,
// so both it and the measured pass see the same warm caches. It cannot be otherwise - a cold
// measurement of one phase is a warm measurement of every phase after it - and the comparison
// that matters here is between SIZES, which that does not affect.

#[test]
#[ignore]
fn perf_dashboard_scale() {
    let n_samples = env_usize("SANDIBUMI_PERF_SAMPLES", DEFAULT_SAMPLES);
    let sizes: Vec<usize> = std::env::var("SANDIBUMI_PERF_DASH_SIZES")
        .unwrap_or_else(|_| "10,100,500".into())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    assert!(!sizes.is_empty(), "SANDIBUMI_PERF_DASH_SIZES parsed to nothing");

    println!("\n============ WHERE THE FIELD DASHBOARD SPENDS ITS TIME ============");
    println!("sizes {sizes:?} x {n_samples} samples per well");
    println!("build: {}", if cfg!(debug_assertions) { "DEBUG (unoptimised)" } else { "release" });
    println!(
        "\n{:<7} {:>10} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9}  {}",
        "WELLS", "WALL", "name", "alias", "curves", "zones", "READS", "compute", "PRODUCED"
    );

    // Per-well cost of each phase at each size, kept so the growth can be printed as a ratio.
    let mut per_well: Vec<(usize, [f64; 8])> = Vec::new();

    for &n_wells in &sizes {
        let db_path = std::env::temp_dir()
            .join(format!("sandibumi_dash_{n_wells}w_{n_samples}s.duckdb"));
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));

        let (well_ids, _) = build_project(&db_path, n_wells, n_samples);
        let conn = crate::project::open_and_migrate(db_path.to_str().unwrap()).expect("open");
        let db = Mutex::new(conn);

        // The dashboard summarises interpreted curves, so the interpretation has to exist first.
        // Same four modules as every other number in this brief.
        for module in ["vsh_gr", "phi_den", "sw_indo", "perm_wyllie_rose"] {
            let produced = run_and_count(&db, module, &well_ids);
            assert!(
                !produced.contains("FAILED"),
                "{n_wells}-well fixture: chain step {module} produced no answer: {produced}"
            );
        }

        let dash = |db: &Mutex<Connection>| {
            run_pay_summary(
                db,
                &PaySummaryRequest {
                    well_ids: well_ids.clone(),
                    vsh_max: Some(cutoff(0.5)),
                    phie_min: Some(cutoff(0.10)),
                    swe_max: Some(cutoff(0.60)),
                    perm_min: None,
                    input_set: None,
                    skip_version: false,
                    stats_only: true,
                    enabled_unset: Vec::new(),
                    discretisation: crate::paysummary::DiscretisationModel::Forward,
                    cutoff_use: Default::default(),
                    custody: Some(crate::workflow::test_run_custody()),
                    frame: Default::default(),
                    weighting: Default::default(),
                },
            )
        };

        // Discarded: it warms whatever a first pass would otherwise pay for, so the replay below
        // and the measured pass start level with each other.
        let _ = dash(&db).expect("warm-up pay summary");

        // ---- replay the per-well read sequence, one lock per well, as the loop does ------------
        let curve_names: Vec<String> =
            vec!["VSH".into(), "PHIE".into(), "SWE".into(), "PERM".into()];
        let phie_candidates = vec!["PHIE".to_string()];
        let empty: std::collections::HashSet<String> = std::collections::HashSet::new();
        let (mut t_name, mut t_alias, mut t_curves, mut t_zones) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        // Inside the alias step, the one query that scans: which log set does this well's PHIE
        // belong to. `LIMIT 2` is timed FIRST and the unlimited form second, so the unlimited
        // one runs on the warmer cache - the comparison is biased AGAINST the cheaper variant,
        // which is the direction that makes a win believable.
        let (mut t_setid_lim, mut t_setid) = (0.0f64, 0.0f64);
        let mut frames = 0usize;
        for id in &well_ids {
            let conn = db.lock().unwrap();

            let t = Instant::now();
            let _name: String = conn
                .query_row(
                    "SELECT well_name FROM wells WHERE well_id = ?1",
                    duckdb::params![id],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| id.clone());
            t_name += ms(t.elapsed());

            let t = Instant::now();
            let _alias = crate::workflow::first_available_input_alias(
                &conn, id, "PHIE", &phie_candidates, None, None, &empty,
            );
            t_alias += ms(t.elapsed());

            let t = Instant::now();
            let frame =
                crate::equations::fetch_curve_frame_from_set(&conn, id, &curve_names, None, None);
            t_curves += ms(t.elapsed());
            if matches!(&frame, Ok((d, _)) if !d.is_empty()) {
                frames += 1;
            }

            let t = Instant::now();
            let _zones = crate::db::list_zones(&conn, id);
            t_zones += ms(t.elapsed());

            // The scan inside `try_resolve_ancestry_input`, timed on its own so the finding
            // names a QUERY and not a function. Both forms answer the same question the
            // caller asks - is there exactly one live set for this curve - because the caller
            // errors on any count other than one, so a second row is all it ever needs to see.
            let run = |sql: &str| -> usize {
                let mut stmt = conn.prepare(sql).expect("prepare");
                let rows: Vec<Option<String>> = stmt
                    .query_map(duckdb::params![id, "PHIE"], |row| row.get(0))
                    .expect("query")
                    .collect::<duckdb::Result<_>>()
                    .expect("collect");
                rows.len()
            };
            let t = Instant::now();
            let _ = run(
                "SELECT DISTINCT CAST(set_id AS VARCHAR) FROM computed_curves \
                 WHERE well_id = ?1 AND upper(curve_name) = ?2 LIMIT 2",
            );
            t_setid_lim += ms(t.elapsed());
            let t = Instant::now();
            let _ = run(
                "SELECT DISTINCT CAST(set_id AS VARCHAR) FROM computed_curves \
                 WHERE well_id = ?1 AND upper(curve_name) = ?2",
            );
            t_setid += ms(t.elapsed());
        }
        assert_eq!(frames, n_wells, "{n_wells}-well fixture: a well returned no curve frame");

        // ---- and the operation itself ----------------------------------------------------------
        let t = Instant::now();
        let rows = dash(&db).expect("pay summary");
        let wall = ms(t.elapsed());

        let reads = t_name + t_alias + t_curves + t_zones;
        // By subtraction: the cut-off arithmetic, the zone sweep, the row building, and whatever
        // lock overhead the replay did not reproduce. Named by how it was derived, not called
        // "compute" as though it had been timed directly.
        let compute = (wall - reads).max(0.0);
        let w = n_wells as f64;

        println!(
            "{:<7} {:>9.1}ms {:>8.1}ms {:>8.1}ms {:>8.1}ms {:>8.1}ms {:>8.1}ms {:>8.1}ms  {} rows",
            n_wells, wall, t_name, t_alias, t_curves, t_zones, reads, compute, rows.len()
        );
        println!(
            "        of which the set-id scan: {:>8.1}ms unlimited vs {:>8.1}ms with LIMIT 2",
            t_setid, t_setid_lim
        );
        per_well.push((
            n_wells,
            [wall / w, t_name / w, t_alias / w, t_curves / w, t_zones / w, compute / w,
             t_setid / w, t_setid_lim / w],
        ));

        drop(db);
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));
    }

    // ---- the finding: which phase costs more PER WELL as the field grows ----------------------
    println!("\n-- PER-WELL COST (ms/well) - a well does not change, so whatever rises here is the cause --");
    let header: String = per_well.iter().map(|(n, _)| format!("{n:>10}")).collect();
    println!("{:<12} {header}{:>8}", "PHASE", "growth");
    for (i, phase) in ["TOTAL", "well name", "alias", "curves", "zones", "compute",
                       "  set-id q", "  same+LIM"]
        .iter()
        .enumerate()
    {
        let cells: String = per_well.iter().map(|(_, v)| format!("{:>10.3}", v[i])).collect();
        let growth = if per_well.len() > 1 {
            let (first, last) = (per_well[0].1[i], per_well[per_well.len() - 1].1[i]);
            if first > 0.0 { format!("{:>7.1}x", last / first) } else { "      -".into() }
        } else {
            String::new()
        };
        println!("{phase:<12} {cells}{growth}");
    }
    println!("\nA phase whose per-well cost is FLAT is not the cause, however large its share.");
}
