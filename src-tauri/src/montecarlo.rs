//! Monte Carlo uncertainty (Phase 9, PT06). Puts probability distributions on chosen model
//! parameters (GR endpoints, matrix density, Archie a/m/n, Rw, …), runs N seeded realizations
//! of a module chain, and reports the resulting spread of net pay / NTG / average PHIE /
//! average SWE / hydrocarbon pore volume (HPV) per zone as P10/P50/P90 plus an HPV histogram.
//!
//! **Why it is fast and write-safe:** each realization runs the whole chain *in memory*
//! (`modules::run_module` returns curve vectors; nothing touches `computed_curves`), so the
//! single-writer DuckDB bottleneck measured for field-scale runs never applies. The only
//! database access is the one-time read of the input curves and the zonation up front. 1000
//! realizations across a typical well finish in well under a second. Realizations are
//! rayon-parallel and each is seeded deterministically from `(seed, realization_index)`, so a
//! given request is fully reproducible.
//!
//! Scope of this first increment: uncertain parameters are sampled *well-wide* (one draw per
//! realization, broadcast over depth); non-uncertain parameters still follow the well's zone
//! parameters exactly like a deterministic run. Per-zone parameter distributions and
//! persisted P10/P50/P90 *curves* are deliberate follow-ups — the headline deliverable here is
//! the zonal HPV/pay distribution a client wants.

use crate::chain::ChainStep;
use crate::db::{self, ZoneEntry, ZoneParamEntry};
use crate::equations;
use crate::modules::{self, ArgKind, ModuleContext};
use duckdb::Connection;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

/// Distribution attached to one uncertain parameter.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Distribution {
    Normal { mean: f64, sd: f64 },
    Uniform { lo: f64, hi: f64 },
    Triangular { lo: f64, mode: f64, hi: f64 },
}

impl Distribution {
    fn sample(&self, rng: &mut Rng) -> f64 {
        match *self {
            Distribution::Normal { mean, sd } => mean + sd * rng.normal(),
            Distribution::Uniform { lo, hi } => lo + (hi - lo) * rng.unit(),
            Distribution::Triangular { lo, mode, hi } => {
                if hi <= lo {
                    return lo;
                }
                let u = rng.unit();
                let c = ((mode - lo) / (hi - lo)).clamp(0.0, 1.0);
                if u < c {
                    lo + (u * (hi - lo) * (mode - lo)).sqrt()
                } else {
                    hi - ((1.0 - u) * (hi - lo) * (hi - mode)).sqrt()
                }
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct McParam {
    /// Module parameter name to vary (e.g. "GR_MA"); applies to every step that has it.
    pub param: String,
    pub dist: Distribution,
}

#[derive(Debug, Clone, Deserialize)]
pub struct McRequest {
    pub well_ids: Vec<String>,
    /// The deterministic chain to run each realization (same shape as a workflow chain).
    pub steps: Vec<ChainStep>,
    pub mc_params: Vec<McParam>,
    pub iterations: usize,
    pub seed: u64,
    // Pay/HPV cutoffs (same semantics as the pay summary).
    pub vsh_max: f64,
    pub phie_min: f64,
    pub swe_max: f64,
    #[serde(default)]
    pub perm_min: Option<f64>,
    /// HPV histogram bin count.
    pub bins: usize,
}

/// P10/P50/P90 + mean/sd for one metric across realizations.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Pctl {
    pub p10: f32,
    pub p50: f32,
    pub p90: f32,
    pub mean: f32,
    pub sd: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct McZoneResult {
    pub well_id: String,
    pub well_name: String,
    pub zone: String,
    pub top: f32,
    pub bottom: f32,
    pub gross: f32,
    pub iterations: usize,
    pub net: Pctl,
    pub ntg: Pctl,
    pub avg_phie: Pctl,
    pub avg_swe: Pctl,
    pub hpv: Pctl,
    /// HPV histogram: `bins` counts spanning [`hist_lo`, `hist_lo` + bins*`hist_w`].
    pub hpv_hist: Vec<u32>,
    pub hist_lo: f32,
    pub hist_w: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct McResult {
    pub zones: Vec<McZoneResult>,
    pub errors: Vec<String>,
}

// ---------------------------------------------------------------------------
// Deterministic PRNG (SplitMix64 + Box–Muller) — dependency-free, seedable.
// ---------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in [0, 1).
    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    /// Standard normal via Box–Muller.
    fn normal(&mut self) -> f64 {
        let u1 = self.unit().max(1e-12);
        let u2 = self.unit();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

// ---------------------------------------------------------------------------
// Per-step execution plan, precomputed once per well (outside the realization loop).
// ---------------------------------------------------------------------------

struct StepPlan {
    module: String,
    opts: HashMap<String, String>,
    /// (arg name, resolved input curve name UPPERCASE).
    log_args: Vec<(String, String)>,
    param_args: Vec<String>,
    /// Zone-resolved base value arrays for parameters NOT under Monte Carlo.
    base_params: HashMap<String, Vec<f64>>,
}

/// Cutoffs bundled for the per-zone pay/HPV accumulation.
#[derive(Clone, Copy)]
struct Cutoffs {
    vsh_max: f64,
    phie_min: f64,
    swe_max: f64,
    perm_min: Option<f64>,
}

#[derive(Clone, Copy)]
struct ZoneMetrics {
    net: f32,
    ntg: f32,
    avg_phie: f32,
    avg_swe: f32,
    hpv: f32,
}

/// Zone-resolved parameter array: manifest/step base, then well-wide '*' then named zones.
fn resolve_zone_param(
    name: &str,
    base: f64,
    zones: &[ZoneEntry],
    zone_params: &[ZoneParamEntry],
    depth: &[f32],
) -> Vec<f64> {
    let mut arr = vec![base; depth.len()];
    for zp in zone_params.iter().filter(|z| z.param_name == name) {
        if let (Some(v), "*") = (zp.value_num, zp.zone_name.as_str()) {
            arr.fill(v as f64);
        }
    }
    for zp in zone_params.iter().filter(|z| z.param_name == name) {
        let Some(v) = zp.value_num else { continue };
        if let Some(z) = zones.iter().find(|z| z.zone_name == zp.zone_name) {
            for (i, d) in depth.iter().enumerate() {
                if *d >= z.top_depth && *d < z.bottom_depth {
                    arr[i] = v as f64;
                }
            }
        }
    }
    arr
}

/// Per-zone net-pay / NTG / averages / HPV from a realization's output curves.
fn zone_metrics(
    vsh: &[f32],
    phie: &[f32],
    swe: &[f32],
    perm: &[f32],
    depth: &[f32],
    step: &[f32],
    zone: &ZoneEntry,
    cut: &Cutoffs,
    has_perm_cut: bool,
) -> ZoneMetrics {
    let mut net = 0.0f64;
    let mut sum_phie = 0.0f64;
    let mut sum_phie_swe = 0.0f64;
    let mut sum_phie_w = 0.0f64;
    let mut hpv = 0.0f64;
    for i in 0..depth.len() {
        // Clamp each sample's forward interval to its overlap with the zone (mirrors
        // run_pay_summary): the last in-zone sample no longer bleeds a full step past the
        // base, and net can never exceed gross — so MC P10/P50/P90 net/NTG/HPV agree with
        // the pay summary.
        let s_top = depth[i] as f64;
        let s_bot = (depth[i] + step[i]) as f64;
        let lo = s_top.max(zone.top_depth as f64);
        let hi = s_bot.min(zone.bottom_depth as f64);
        let h = hi - lo;
        if h <= 0.0 {
            continue;
        }
        let v = vsh[i] as f64;
        let p = phie[i] as f64;
        let s = swe[i] as f64;
        if v.is_nan() || p.is_nan() || s.is_nan() {
            continue;
        }
        let mut pay = v <= cut.vsh_max && p >= cut.phie_min && s <= cut.swe_max;
        if has_perm_cut {
            // A sample with no PERM value cannot demonstrate it passes the cutoff — missing
            // PERM must fail, not silently pass (matches run_pay_summary's classify_sample, so
            // MC and the pay summary agree for identical cutoffs).
            pay = pay && !perm[i].is_nan() && (perm[i] as f64) >= cut.perm_min.unwrap();
        }
        if !pay {
            continue;
        }
        net += h;
        sum_phie += p * h;
        sum_phie_swe += p * s * h;
        sum_phie_w += p * h;
        hpv += p * (1.0 - s) * h;
    }
    let gross = (zone.bottom_depth - zone.top_depth) as f64;
    ZoneMetrics {
        net: net as f32,
        ntg: if gross > 0.0 { (net / gross) as f32 } else { 0.0 },
        avg_phie: if net > 0.0 { (sum_phie / net) as f32 } else { f32::NAN },
        avg_swe: if sum_phie_w > 0.0 { (sum_phie_swe / sum_phie_w) as f32 } else { f32::NAN },
        hpv: hpv as f32,
    }
}

/// Percentile (linear interpolation, type-7) over finite values; NaN if none finite.
fn percentile(sorted: &[f32], p: f64) -> f32 {
    if sorted.is_empty() {
        return f32::NAN;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let rank = p * (sorted.len() - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    let frac = (rank - lo as f64) as f32;
    sorted[lo] + (sorted[hi] - sorted[lo]) * frac
}

fn summarize(values: &[f32]) -> Pctl {
    let mut finite: Vec<f32> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.is_empty() {
        return Pctl::default();
    }
    finite.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = finite.len() as f64;
    let mean = finite.iter().map(|v| *v as f64).sum::<f64>() / n;
    let var = finite.iter().map(|v| (*v as f64 - mean).powi(2)).sum::<f64>() / n;
    Pctl {
        p10: percentile(&finite, 0.10),
        p50: percentile(&finite, 0.50),
        p90: percentile(&finite, 0.90),
        mean: mean as f32,
        sd: var.sqrt() as f32,
    }
}

fn histogram(values: &[f32], bins: usize) -> (Vec<u32>, f32, f32) {
    let bins = bins.max(1);
    let finite: Vec<f32> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.is_empty() {
        return (vec![0; bins], 0.0, 0.0);
    }
    let lo = finite.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = finite.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut counts = vec![0u32; bins];
    if hi <= lo {
        counts[0] = finite.len() as u32; // degenerate: all identical
        return (counts, lo, 0.0);
    }
    let w = (hi - lo) / bins as f32;
    for v in finite {
        let mut b = ((v - lo) / w) as usize;
        if b >= bins {
            b = bins - 1;
        }
        counts[b] += 1;
    }
    (counts, lo, w)
}

/// Builds the per-step plan for one well: resolves inputs, options, and zone-resolved base
/// parameter arrays. Returns the plans plus the raw input-curve pool and depth/step arrays.
fn build_plans(
    conn: &Connection,
    well_id: &str,
    steps: &[ChainStep],
    specs: &HashMap<String, modules::ModuleSpec>,
) -> Result<(Vec<StepPlan>, HashMap<String, Vec<f32>>, Vec<f32>, Vec<f32>, Vec<ZoneEntry>), String> {
    // Curves produced somewhere in the chain don't need a DB read — they come from the pool.
    let mut produced: HashSet<String> = HashSet::new();
    for step in steps {
        let spec = specs.get(&step.module).ok_or_else(|| format!("unknown module '{}'", step.module))?;
        for a in spec.args.iter().filter(|a| a.kind == ArgKind::LogOut) {
            produced.insert(a.name.to_uppercase());
        }
    }

    // External inputs = LogIn mnemonics not produced by any step.
    let mut external: HashSet<String> = HashSet::new();
    for step in steps {
        let spec = &specs[&step.module];
        for a in spec.args.iter().filter(|a| a.kind == ArgKind::LogIn) {
            let mnem = step.log_inputs.get(&a.name).cloned().unwrap_or_else(|| a.default.clone());
            let up = mnem.trim().to_uppercase();
            if !produced.contains(&up) {
                external.insert(up);
            }
        }
    }

    let names: Vec<String> = external.iter().cloned().collect();
    let (depth, columns) = equations::fetch_curve_frame(conn, well_id, &names).map_err(|e| e.to_string())?;
    if depth.is_empty() {
        return Err("no curve data for well".into());
    }
    let n = depth.len();

    // Raw pool keyed UPPERCASE, matching how chain steps resolve their inputs.
    let mut raw_pool: HashMap<String, Vec<f32>> = HashMap::new();
    for name in &names {
        let v = columns.get(&name.to_uppercase()).cloned().unwrap_or_else(|| vec![f32::NAN; n]);
        raw_pool.insert(name.to_uppercase(), v);
    }

    let zones_raw = db::list_zones(conn, well_id).map_err(|e| e.to_string())?;
    let zone_params = db::list_zone_params(conn, well_id).map_err(|e| e.to_string())?;

    // Sample thickness: forward difference, last reuses previous step.
    let mut step_thick = vec![0.0f32; n];
    for i in 0..n {
        step_thick[i] = if i + 1 < n {
            depth[i + 1] - depth[i]
        } else if i > 0 {
            step_thick[i - 1]
        } else {
            0.0
        };
    }

    let mut plans = Vec::with_capacity(steps.len());
    for step in steps {
        let spec = &specs[&step.module];

        // Options: manifest defaults, then step overrides, plus __IN_<arg> mnemonics.
        let mut opts: HashMap<String, String> = spec
            .args
            .iter()
            .filter(|a| a.kind == ArgKind::Option)
            .map(|a| (a.name.clone(), a.default.clone()))
            .collect();
        for (k, v) in &step.opts {
            opts.insert(k.clone(), v.clone());
        }

        let mut log_args = Vec::new();
        for a in spec.args.iter().filter(|a| a.kind == ArgKind::LogIn) {
            let mnem = step.log_inputs.get(&a.name).cloned().unwrap_or_else(|| a.default.clone());
            let up = mnem.trim().to_uppercase();
            opts.insert(format!("__IN_{}", a.name), up.clone());
            log_args.push((a.name.clone(), up));
        }

        let mut param_args = Vec::new();
        let mut base_params = HashMap::new();
        for a in spec.args.iter().filter(|a| a.kind == ArgKind::Param) {
            param_args.push(a.name.clone());
            let base = step.params.get(&a.name).copied().or_else(|| a.default.parse().ok()).unwrap_or(f64::NAN);
            base_params.insert(a.name.clone(), resolve_zone_param(&a.name, base, &zones_raw, &zone_params, &depth));
        }

        plans.push(StepPlan { module: step.module.clone(), opts, log_args, param_args, base_params });
    }

    Ok((plans, raw_pool, depth, step_thick, zones_raw))
}

/// Runs the chain in memory for one realization and returns the resulting curve pool.
fn run_realization(
    plans: &[StepPlan],
    raw_pool: &HashMap<String, Vec<f32>>,
    depth: &[f32],
    mc_values: &HashMap<String, f64>,
    n: usize,
) -> HashMap<String, Vec<f32>> {
    let mut pool = raw_pool.clone();
    for plan in plans {
        let mut logs: HashMap<String, Vec<f32>> = HashMap::with_capacity(plan.log_args.len() + 1);
        logs.insert("DEPTH".to_string(), depth.to_vec());
        for (arg, mnem) in &plan.log_args {
            let v = pool.get(mnem).cloned().unwrap_or_else(|| vec![f32::NAN; n]);
            logs.insert(arg.clone(), v);
        }
        let mut params: HashMap<String, Vec<f64>> = HashMap::with_capacity(plan.param_args.len());
        for pname in &plan.param_args {
            if let Some(v) = mc_values.get(pname) {
                params.insert(pname.clone(), vec![*v; n]);
            } else if let Some(base) = plan.base_params.get(pname) {
                params.insert(pname.clone(), base.clone());
            }
        }
        let ctx = ModuleContext { n, logs, params, opts: plan.opts.clone() };
        if let Ok(outputs) = modules::run_module(&plan.module, &ctx) {
            for (k, v) in outputs {
                pool.insert(k.to_uppercase(), v);
            }
        }
        // A failed step just leaves the pool unchanged; downstream sees NaNs.
    }
    pool
}

/// Runs the Monte Carlo study across the requested wells. All computation is in memory; the
/// only DB access is the per-well input read in [`build_plans`].
pub fn run_monte_carlo(
    db: &Mutex<Connection>,
    req: &McRequest,
    progress: Option<&crate::jobs::JobHandle>,
) -> McResult {
    let iterations = req.iterations.clamp(1, 100_000);
    let specs: HashMap<String, modules::ModuleSpec> =
        modules::list_modules().into_iter().map(|s| (s.name.clone(), s)).collect();
    let cut = Cutoffs {
        vsh_max: req.vsh_max,
        phie_min: req.phie_min,
        swe_max: req.swe_max,
        perm_min: req.perm_min,
    };

    let mut zones_out: Vec<McZoneResult> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for (wi, well_id) in req.well_ids.iter().enumerate() {
        if let Some(p) = progress {
            if p.is_cancelled() {
                break;
            }
            p.set_current(Some(format!("Monte Carlo: well {}/{}", wi + 1, req.well_ids.len())));
            p.start_item(well_id);
        }
        // One-time DB phase: build plans + read inputs + zonation.
        let (plans, raw_pool, depth, step_thick, mut zones, well_name) = {
            let conn = db.lock().unwrap();
            let well_name: String = conn
                .query_row("SELECT well_name FROM wells WHERE well_id = ?1", duckdb::params![well_id], |r| r.get(0))
                .unwrap_or_else(|_| well_id.clone());
            match build_plans(&conn, well_id, &req.steps, &specs) {
                Ok((p, rp, d, st, z)) => (p, rp, d, st, z, well_name),
                Err(e) => {
                    if let Some(p) = progress {
                        p.finish_item(well_id, crate::jobs::ItemState::Failed, Some(e.to_string()));
                    }
                    errors.push(format!("{well_id}: {e}"));
                    continue;
                }
            }
        };
        let n = depth.len();
        if zones.is_empty() {
            zones.push(ZoneEntry { zone_name: "ALL".into(), top_depth: depth[0], bottom_depth: *depth.last().unwrap() });
        }
        let has_perm_cut =
            req.perm_min.is_some() && raw_pool.get("PERM").map(|c| c.iter().any(|v| !v.is_nan())).unwrap_or(false);

        // Parallel realizations, each seeded from (seed, index) for reproducibility.
        let per_real: Vec<Vec<ZoneMetrics>> = (0..iterations)
            .into_par_iter()
            .map(|r| {
                let mut rng = Rng::new(req.seed ^ (r as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
                let mc_values: HashMap<String, f64> =
                    req.mc_params.iter().map(|p| (p.param.clone(), p.dist.sample(&mut rng))).collect();
                let pool = run_realization(&plans, &raw_pool, &depth, &mc_values, n);
                let nanv = vec![f32::NAN; n];
                let vsh = pool.get("VSH").unwrap_or(&nanv);
                let phie = pool.get("PHIE").unwrap_or(&nanv);
                let swe = pool.get("SWE").unwrap_or(&nanv);
                let perm = pool.get("PERM").unwrap_or(&nanv);
                zones
                    .iter()
                    .map(|z| zone_metrics(vsh, phie, swe, perm, &depth, &step_thick, z, &cut, has_perm_cut))
                    .collect()
            })
            .collect();

        // Transpose (realization × zone) → per-zone metric vectors, then summarize.
        for (zi, zone) in zones.iter().enumerate() {
            let net: Vec<f32> = per_real.iter().map(|m| m[zi].net).collect();
            let ntg: Vec<f32> = per_real.iter().map(|m| m[zi].ntg).collect();
            let avg_phie: Vec<f32> = per_real.iter().map(|m| m[zi].avg_phie).collect();
            let avg_swe: Vec<f32> = per_real.iter().map(|m| m[zi].avg_swe).collect();
            let hpv: Vec<f32> = per_real.iter().map(|m| m[zi].hpv).collect();
            let (hpv_hist, hist_lo, hist_w) = histogram(&hpv, req.bins);
            zones_out.push(McZoneResult {
                well_id: well_id.clone(),
                well_name: well_name.clone(),
                zone: zone.zone_name.clone(),
                top: zone.top_depth,
                bottom: zone.bottom_depth,
                gross: zone.bottom_depth - zone.top_depth,
                iterations,
                net: summarize(&net),
                ntg: summarize(&ntg),
                avg_phie: summarize(&avg_phie),
                avg_swe: summarize(&avg_swe),
                hpv: summarize(&hpv),
                hpv_hist,
                hist_lo,
                hist_w,
            });
        }
        if let Some(p) = progress {
            p.finish_item(well_id, crate::jobs::ItemState::Ok, None);
        }
    }

    McResult { zones: zones_out, errors }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use uuid::Uuid;

    fn step(module: &str) -> ChainStep {
        ChainStep { module: module.into(), log_inputs: HashMap::new(), params: HashMap::new(), opts: HashMap::new() }
    }

    /// A clean-ish sand: low GR, moderate porosity, low water saturation, so vsh_gr → phi_dn →
    /// sw_indo yields real pay and a positive HPV.
    fn seed_well(conn: &Connection) -> String {
        let id = Uuid::new_v4();
        db::insert_well(conn, id, "MC_TEST", Some("Synthetic"), None, None).unwrap();
        let n = 300usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let gr: Vec<f32> = (0..n).map(|i| 25.0 + (i % 30) as f32).collect();
        let res: Vec<f32> = vec![20.0; n];
        let nphi: Vec<f32> = vec![0.20; n];
        let rhob: Vec<f32> = vec![2.35; n];
        let dt: Vec<f32> = vec![80.0; n];
        let sp: Vec<f32> = vec![f32::NAN; n];
        db::insert_standard_curves(conn, id, depth, gr, res, nphi, rhob, dt, sp).unwrap();
        id.to_string()
    }

    fn base_request(well: &str, mc: Vec<McParam>, iterations: usize, seed: u64) -> McRequest {
        McRequest {
            well_ids: vec![well.into()],
            steps: vec![step("vsh_gr"), step("phi_dn"), step("sw_indo")],
            mc_params: mc,
            iterations,
            seed,
            vsh_max: 0.5,
            phie_min: 0.08,
            swe_max: 0.6,
            perm_min: None,
            bins: 10,
        }
    }

    #[test]
    fn hpv_distribution_is_ordered_and_reproducible() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);

        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 8.0 } }];
        let res = run_monte_carlo(&dbm, &base_request(&well, mc.clone(), 500, 42), None);
        assert!(res.errors.is_empty(), "unexpected errors: {:?}", res.errors);
        assert_eq!(res.zones.len(), 1);
        let z = &res.zones[0];
        // Percentiles ordered, spread present, HPV positive, histogram counts all iterations.
        assert!(z.hpv.p10 <= z.hpv.p50 && z.hpv.p50 <= z.hpv.p90, "HPV percentiles unordered: {:?}", z.hpv);
        assert!(z.hpv.p90 > z.hpv.p10, "expected spread from GR_MA uncertainty");
        assert!(z.hpv.p50 > 0.0, "expected positive HPV in a clean sand");
        assert!(z.net.p50 > 0.0 && z.avg_phie.p50 > 0.0 && z.avg_swe.p50 > 0.0);
        assert_eq!(z.hpv_hist.iter().sum::<u32>(), 500);

        // Same seed → identical result (reproducible).
        let res2 = run_monte_carlo(&dbm, &base_request(&well, mc, 500, 42), None);
        assert_eq!(res.zones[0].hpv.p50, res2.zones[0].hpv.p50);
        assert_eq!(res.zones[0].hpv.mean, res2.zones[0].hpv.mean);
    }

    #[test]
    fn zero_variance_param_collapses_distribution() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);

        // sd = 0 → every realization identical → P10 == P50 == P90, sd ≈ 0.
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 0.0 } }];
        let res = run_monte_carlo(&dbm, &base_request(&well, mc, 200, 7), None);
        let z = &res.zones[0];
        assert_eq!(z.hpv.p10, z.hpv.p90, "no variance should collapse the spread");
        assert!(z.hpv.sd.abs() < 1e-6, "sd should be ~0, got {}", z.hpv.sd);
    }

    #[test]
    fn uniform_and_triangular_sample_in_range() {
        let mut rng = Rng::new(123);
        for _ in 0..10_000 {
            let u = Distribution::Uniform { lo: 2.0, hi: 5.0 }.sample(&mut rng);
            assert!((2.0..=5.0).contains(&u), "uniform out of range: {u}");
            let t = Distribution::Triangular { lo: 0.0, mode: 3.0, hi: 4.0 }.sample(&mut rng);
            assert!((0.0..=4.0).contains(&t), "triangular out of range: {t}");
        }
    }
}
