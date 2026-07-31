//! Stratigraphic Modified Lorenz Plot (Wave B item 8 / playbook #3, increment 3a) — flow-unit
//! identification from a well's continuous porosity + permeability profile, complementing the
//! FIXED-bin GHE typing in `rocktyping.rs` and the data-driven φ-k clustering in `hfu.rs`.
//!
//! Method (Gunter et al. 1997, SPE 38679, "Early Determination of Reservoir Flow Units Using an
//! Integrated Petrophysical Method"; heterogeneity index after Schmalz & Rahme 1950, reviewed in
//! Lake & Jensen 1991, SPE 20156):
//!   - **Stratigraphic Modified Lorenz Plot (SMLP):** walk the samples in DEPTH order and
//!     accumulate flow capacity Σ(k·h) on Y against storage capacity Σ(φ·h) on X, each normalized
//!     to its total (0..1). Because the walk is stratigraphic (NOT reordered), the curve is
//!     monotone but kinked — its LOCAL SLOPE at sample i is (k_i/φ_i)·(Σφh/Σkh), independent of
//!     thickness (the h cancels). Contiguous runs of similar slope are flow units: slope > 1 ⇒ the
//!     interval contributes more flow than storage (a "speed zone"/reservoir conduit), slope < 1 ⇒
//!     a baffle, slope ≈ 0 ⇒ a seal. The diagonal (slope 1) is the well-average k/φ, so 1 is a
//!     principled split — not a tuned constant.
//!   - **Flow-unit segmentation:** an EXACT contiguous dynamic program partitions the depth-ordered
//!     log10(k/φ) profile into K segments minimizing within-segment sum of squares (same Ward
//!     criterion as `hfu.rs`, but here the natural depth order is preserved instead of sorting, so
//!     segments are true depth intervals). K is caller-set, or auto-selected by marginal-gain: keep
//!     splitting while the next split removes ≥ AUTO_K_TOL of the single-segment SSE.
//!   - **Lorenz coefficient Lc:** the classic heterogeneity index from the REORDERED Lorenz plot
//!     (samples sorted by descending k/φ) — Lc = 2·(area between that curve and the 45° line),
//!     0 = homogeneous, →1 = highly heterogeneous.
//!
//! Reads continuous curves via `equations::fetch_curve_frame` (so PERM can be an imported KLOGH, a
//! computed PERM, or the rock-typing PERM_RT estimate). Samples with φ∉(0,1) or k≤0 are skipped.

use duckdb::Connection;
use std::sync::Mutex;

/// Auto-K stops when the next split would remove less than this fraction of the single-segment
/// SSE — i.e. a new flow-unit boundary must explain ≥2 % of the total slope variance to be kept.
const AUTO_K_TOL: f64 = 0.02;
/// Upper bound on auto-selected flow units (matches the HFU cluster cap).
const AUTO_K_MAX: usize = 12;

#[derive(serde::Deserialize)]
pub struct LorenzRequest {
    pub well_id: String,
    /// Porosity curve mnemonic (resolved through the standard/computed/generic precedence).
    pub phi_curve: String,
    /// Permeability curve mnemonic (mD) — imported KLOGH, computed PERM, or PERM_RT.
    pub perm_curve: String,
    /// Optional depth window (inclusive); None = whole well.
    pub depth_from: Option<f64>,
    pub depth_to: Option<f64>,
    /// Requested flow-unit count; 0 = auto-select by marginal gain.
    pub n_units: u32,
}

#[derive(serde::Serialize)]
pub struct LorenzPoint {
    pub depth: f32,
    pub phi: f32,
    pub perm: f32,
    /// Cumulative storage fraction Σ(φh)/Σφh at this sample, 0..1 (depth order).
    pub cum_storage: f64,
    /// Cumulative flow fraction Σ(kh)/Σkh at this sample, 0..1 (depth order).
    pub cum_flow: f64,
    /// Local SMLP slope (k_i/φ_i)·(Σφh/Σkh); 1 = the well-average k/φ.
    pub slope: f64,
    /// Flow-unit id, 1..K in DEPTH order (unit 1 = shallowest).
    pub unit: u32,
}

#[derive(serde::Serialize)]
pub struct LorenzUnit {
    pub unit: u32,
    pub depth_top: f32,
    pub depth_base: f32,
    pub n: usize,
    /// Share of the well's total storage capacity (Σφh) in this unit (Δx across the unit).
    pub storage_frac: f64,
    /// Share of the well's total flow capacity (Σkh) in this unit (Δy across the unit).
    pub flow_frac: f64,
    /// Unit SMLP slope = flow_frac / storage_frac (thickness-weighted, normalized). >1 speed zone.
    pub slope: f64,
    /// Thickness-weighted mean porosity Σφh/Σh (v/v).
    pub phi_mean: f64,
    /// Thickness-weighted mean permeability Σkh/Σh (mD).
    pub perm_mean: f64,
    /// Advisory character from `slope` vs the 45° diagonal: "speed" (>1), "baffle" (<1), "balanced".
    pub character: String,
}

#[derive(serde::Serialize)]
pub struct LorenzResult {
    pub points: Vec<LorenzPoint>,
    pub units: Vec<LorenzUnit>,
    /// Lorenz heterogeneity coefficient (0 homogeneous … 1 heterogeneous).
    pub lorenz_coefficient: f64,
    /// Total flow capacity Σ(k·h) and storage capacity Σ(φ·h) over the valid samples.
    pub total_kh: f64,
    pub total_phih: f64,
    pub n_samples: usize,
    pub skipped: usize,
    pub note: Option<String>,
    pub error: Option<String>,
}

fn lorenz_err(msg: &str) -> LorenzResult {
    LorenzResult {
        points: vec![],
        units: vec![],
        lorenz_coefficient: f64::NAN,
        total_kh: 0.0,
        total_phih: 0.0,
        n_samples: 0,
        skipped: 0,
        note: None,
        error: Some(msg.to_string()),
    }
}

/// Per-sample bed thickness from the midpoint rule on the FULL depth grid (interior sample gets
/// half the gap to each neighbour; edges get the one-sided gap). Computed on the full frame BEFORE
/// screening, so a valid sample flanked by missing ones still carries only its own grid step — the
/// missing interval simply contributes nothing rather than smearing onto its neighbours.
fn local_thickness(depth: &[f32]) -> Vec<f64> {
    let n = depth.len();
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![1.0];
    }
    let mut h = vec![0.0f64; n];
    for i in 0..n {
        let d = depth[i] as f64;
        let hi = if i == 0 {
            (depth[1] as f64 - d).abs()
        } else if i == n - 1 {
            (d - depth[n - 2] as f64).abs()
        } else {
            (depth[i + 1] as f64 - depth[i - 1] as f64).abs() / 2.0
        };
        // A degenerate (duplicate-depth) step would zero a sample's weight; give it a unit share.
        h[i] = if hi > 0.0 { hi } else { 1.0 };
    }
    h
}

/// Exact contiguous K-segmentation dp of `vals` (kept in the given order) minimizing total
/// within-segment sum of squares. Returns (`sse_by_k`, `arg`) where `sse_by_k[j]` is the optimal
/// total SSE using j segments (index 0 unused) and `arg` is the backtracking table. O(kmax·m²).
fn segment_dp(vals: &[f64], kmax: usize) -> (Vec<f64>, Vec<Vec<usize>>) {
    let m = vals.len();
    let k = kmax.clamp(1, m.max(1));
    // Prefix sums for O(1) segment SS: cost[a,b) = Σx² − (Σx)²/(b−a).
    let mut ps = vec![0.0f64; m + 1];
    let mut ps2 = vec![0.0f64; m + 1];
    for i in 0..m {
        ps[i + 1] = ps[i] + vals[i];
        ps2[i + 1] = ps2[i] + vals[i] * vals[i];
    }
    let cost = |a: usize, b: usize| -> f64 {
        let cnt = (b - a) as f64;
        if cnt <= 0.0 {
            return 0.0;
        }
        let s = ps[b] - ps[a];
        (ps2[b] - ps2[a] - s * s / cnt).max(0.0)
    };
    let inf = f64::INFINITY;
    // dp[j][i] = min WCSS of the first i elements split into j contiguous segments.
    let mut dp = vec![vec![inf; m + 1]; k + 1];
    let mut arg = vec![vec![0usize; m + 1]; k + 1];
    dp[0][0] = 0.0;
    for j in 1..=k {
        for i in j..=m {
            for t in (j - 1)..i {
                let c = dp[j - 1][t] + cost(t, i);
                if c < dp[j][i] {
                    dp[j][i] = c;
                    arg[j][i] = t;
                }
            }
        }
    }
    let sse_by_k: Vec<f64> = (0..=k).map(|j| dp[j][m]).collect();
    (sse_by_k, arg)
}

/// Backtrack the segment id (0-based, ascending position) of each element for a K-segmentation.
fn backtrack(arg: &[Vec<usize>], k: usize, m: usize) -> Vec<usize> {
    let mut assign = vec![0usize; m];
    let mut i = m;
    for j in (1..=k).rev() {
        let t = arg[j][i];
        for a in assign.iter_mut().take(i).skip(t) {
            *a = j - 1;
        }
        i = t;
    }
    assign
}

/// Lorenz heterogeneity coefficient from flow/storage pairs: sort by descending k/φ, accumulate
/// normalized (Σφh, Σkh) into a convex curve, and return 2·(area under it − ½), clamped to [0,1].
fn lorenz_coefficient(phih: &[f64], kh: &[f64], ratio: &[f64], total_phih: f64, total_kh: f64) -> f64 {
    let m = phih.len();
    if m == 0 || total_phih <= 0.0 || total_kh <= 0.0 {
        return f64::NAN;
    }
    let mut order: Vec<usize> = (0..m).collect();
    order.sort_by(|&a, &b| ratio[b].partial_cmp(&ratio[a]).unwrap_or(std::cmp::Ordering::Equal));
    // Trapezoidal area under the (x=cum storage, y=cum flow) curve from (0,0).
    let (mut x_prev, mut y_prev, mut area) = (0.0f64, 0.0f64, 0.0f64);
    for &i in &order {
        let x = x_prev + phih[i] / total_phih;
        let y = y_prev + kh[i] / total_kh;
        area += (x - x_prev) * (y + y_prev) / 2.0;
        x_prev = x;
        y_prev = y;
    }
    (2.0 * area - 1.0).clamp(0.0, 1.0)
}

/// Core SMLP + flow-unit + Lorenz computation over a depth-ordered continuous frame. DB-free and
/// deterministic so it is directly unit-testable; `run_lorenz` supplies the frame from the store.
/// `n_units` = 0 auto-selects K by marginal gain; ≥1 forces exactly that many (capped to sample n).
pub fn compute_lorenz(depth: &[f32], phi: &[f32], perm: &[f32], n_units: u32) -> LorenzResult {
    let nframe = depth.len();
    if phi.len() != nframe || perm.len() != nframe {
        return lorenz_err("phi/perm/depth length mismatch");
    }
    let thick = local_thickness(depth);

    // Screen to valid samples, carrying each one's own grid-step thickness.
    let mut d = Vec::new();
    let mut vphi = Vec::new();
    let mut vperm = Vec::new();
    let mut h = Vec::new();
    let mut m_slope = Vec::new(); // log10(k/φ), the segmentation metric
    let mut ratio = Vec::new(); // k/φ, for the Lorenz coefficient ordering
    let mut skipped = 0usize;
    for i in 0..nframe {
        let p = phi[i] as f64;
        let k = perm[i] as f64;
        if !(p.is_finite() && k.is_finite()) || p <= 0.0 || p >= 1.0 || k <= 0.0 {
            skipped += 1;
            continue;
        }
        d.push(depth[i]);
        vphi.push(phi[i]);
        vperm.push(perm[i]);
        h.push(thick[i]);
        ratio.push(k / p);
        m_slope.push((k / p).log10());
    }
    let m = d.len();
    if m == 0 {
        return lorenz_err("no samples with valid φ (0–1) and k (>0) in range — check the porosity and permeability curves");
    }

    // Flow (k·h) and storage (φ·h) capacity per sample, and their totals.
    let kh: Vec<f64> = (0..m).map(|i| vperm[i] as f64 * h[i]).collect();
    let phih: Vec<f64> = (0..m).map(|i| vphi[i] as f64 * h[i]).collect();
    let total_kh: f64 = kh.iter().sum();
    let total_phih: f64 = phih.iter().sum();
    if total_kh <= 0.0 || total_phih <= 0.0 {
        return lorenz_err("total flow or storage capacity is zero");
    }

    // Flow-unit segmentation on the depth-ordered log10(k/φ) profile.
    let kmax = if n_units == 0 { AUTO_K_MAX.min(m) } else { (n_units as usize).min(m) };
    let (sse_by_k, arg) = segment_dp(&m_slope, kmax);
    let total_sse = sse_by_k.get(1).copied().unwrap_or(0.0);
    let eff_k = if n_units >= 1 {
        (n_units as usize).min(m)
    } else if total_sse <= 1e-12 {
        1 // uniform slope profile → a single flow unit
    } else {
        let mut kk = 1usize;
        while kk < kmax {
            let gain = (sse_by_k[kk] - sse_by_k[kk + 1]) / total_sse;
            if gain >= AUTO_K_TOL {
                kk += 1;
            } else {
                break;
            }
        }
        kk
    };
    let assign = backtrack(&arg, eff_k, m);

    // Cumulative curve + per-sample slope, in depth order.
    let scale = total_phih / total_kh; // normalizes local slope so the 45° line is slope 1
    let mut points = Vec::with_capacity(m);
    let (mut cx, mut cy) = (0.0f64, 0.0f64);
    for i in 0..m {
        cx += phih[i] / total_phih;
        cy += kh[i] / total_kh;
        points.push(LorenzPoint {
            depth: d[i],
            phi: vphi[i],
            perm: vperm[i],
            cum_storage: cx.min(1.0),
            cum_flow: cy.min(1.0),
            slope: ratio[i] * scale,
            unit: (assign[i] + 1) as u32,
        });
    }

    // Per-unit aggregates (units are contiguous in depth, id 1 = shallowest).
    let n_units_actual = assign.iter().copied().max().map(|x| x + 1).unwrap_or(0);
    let mut units = Vec::with_capacity(n_units_actual);
    for c in 0..n_units_actual {
        let idx: Vec<usize> = (0..m).filter(|&i| assign[i] == c).collect();
        if idx.is_empty() {
            continue;
        }
        let u_kh: f64 = idx.iter().map(|&i| kh[i]).sum();
        let u_phih: f64 = idx.iter().map(|&i| phih[i]).sum();
        let u_h: f64 = idx.iter().map(|&i| h[i]).sum();
        let storage_frac = u_phih / total_phih;
        let flow_frac = u_kh / total_kh;
        let slope = if storage_frac > 0.0 { flow_frac / storage_frac } else { f64::NAN };
        let character = if !slope.is_finite() {
            "n/a"
        } else if slope > 1.0 {
            "speed"
        } else if slope < 1.0 {
            "baffle"
        } else {
            "balanced"
        };
        units.push(LorenzUnit {
            unit: (c + 1) as u32,
            depth_top: idx.iter().map(|&i| d[i]).fold(f32::INFINITY, f32::min),
            depth_base: idx.iter().map(|&i| d[i]).fold(f32::NEG_INFINITY, f32::max),
            n: idx.len(),
            storage_frac,
            flow_frac,
            slope,
            phi_mean: if u_h > 0.0 { u_phih / u_h } else { f64::NAN },
            perm_mean: if u_h > 0.0 { u_kh / u_h } else { f64::NAN },
            character: character.to_string(),
        });
    }

    let lc = lorenz_coefficient(&phih, &kh, &ratio, total_phih, total_kh);

    let note = if n_units >= 1 && (n_units as usize) > m {
        Some(format!("requested {n_units} units but only {m} valid samples — used {m}"))
    } else {
        None
    };

    LorenzResult {
        points,
        units,
        lorenz_coefficient: lc,
        total_kh,
        total_phih,
        n_samples: m,
        skipped,
        note,
        error: None,
    }
}

/// Reads `phi_curve` + `perm_curve` for one well (via the standard/computed/generic resolver),
/// optionally windows to [depth_from, depth_to], and runs `compute_lorenz`.
pub fn run_lorenz(db: &Mutex<Connection>, req: &LorenzRequest) -> LorenzResult {
    if req.well_id.trim().is_empty() {
        return lorenz_err("no well selected");
    }
    if req.phi_curve.trim().is_empty() || req.perm_curve.trim().is_empty() {
        return lorenz_err("choose a porosity curve and a permeability curve");
    }

    let conn = db.lock().unwrap();
    let names = vec![req.phi_curve.trim().to_uppercase(), req.perm_curve.trim().to_uppercase()];
    let (depth, cols) = match crate::equations::fetch_curve_frame(&conn, &req.well_id, &names) {
        Ok(f) => f,
        Err(e) => return lorenz_err(&format!("reading curves: {e}")),
    };
    drop(conn);

    let phi_key = req.phi_curve.trim().to_uppercase();
    let perm_key = req.perm_curve.trim().to_uppercase();
    let empty = Vec::new();
    let phi_col = cols.get(&phi_key).unwrap_or(&empty);
    let perm_col = cols.get(&perm_key).unwrap_or(&empty);
    if phi_col.len() != depth.len() || phi_col.iter().all(|v| v.is_nan()) {
        return lorenz_err(&format!("porosity curve '{}' has no data in this well", req.phi_curve));
    }
    if perm_col.len() != depth.len() || perm_col.iter().all(|v| v.is_nan()) {
        return lorenz_err(&format!("permeability curve '{}' has no data in this well", req.perm_curve));
    }

    // Apply the optional depth window on the aligned frame.
    let lo = req.depth_from.unwrap_or(f64::NEG_INFINITY);
    let hi = req.depth_to.unwrap_or(f64::INFINITY);
    let (mut wd, mut wphi, mut wperm) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..depth.len() {
        let z = depth[i] as f64;
        if z >= lo && z <= hi {
            wd.push(depth[i]);
            wphi.push(phi_col[i]);
            wperm.push(perm_col[i]);
        }
    }
    if wd.is_empty() {
        return lorenz_err("no samples in the selected depth window");
    }

    compute_lorenz(&wd, &wphi, &wperm, req.n_units)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a synthetic uniform-step column of `runs`, each `(n_samples, k_over_phi)` at a fixed
    /// porosity, so log10(k/φ) is piecewise-constant and the flow units are known by construction.
    fn column(phi: f32, runs: &[(usize, f64)]) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let (mut d, mut vphi, mut vperm) = (Vec::new(), Vec::new(), Vec::new());
        let mut depth = 2000.0f32;
        for &(n, kphi) in runs {
            for _ in 0..n {
                d.push(depth);
                vphi.push(phi);
                vperm.push((kphi * phi as f64) as f32);
                depth += 0.5;
            }
        }
        (d, vphi, vperm)
    }

    #[test]
    fn smlp_auto_finds_three_flow_units() {
        // Three well-separated slope regimes: k/φ = 1000, 10, 100 → log10 = 3, 1, 2.
        let (d, phi, perm) = column(0.20, &[(10, 1000.0), (10, 10.0), (10, 100.0)]);
        let r = compute_lorenz(&d, &phi, &perm, 0);
        assert!(r.error.is_none(), "err={:?}", r.error);
        assert_eq!(r.units.len(), 3, "expected 3 flow units, got {}", r.units.len());
        assert_eq!(r.n_samples, 30);
        // Units are contiguous depth intervals in order; each spans exactly its 10-sample run.
        assert_eq!(r.units[0].n, 10);
        assert_eq!(r.units[1].n, 10);
        assert_eq!(r.units[2].n, 10);
        // The top run has the highest k/φ → highest slope (a speed zone); the middle the lowest.
        assert!(r.units[0].slope > r.units[2].slope);
        assert!(r.units[2].slope > r.units[1].slope);
        assert_eq!(r.units[0].character, "speed");
    }

    #[test]
    fn forced_k_overrides_auto_selection() {
        let (d, phi, perm) = column(0.20, &[(10, 1000.0), (10, 10.0), (10, 100.0)]);
        let r = compute_lorenz(&d, &phi, &perm, 2);
        assert!(r.error.is_none());
        assert_eq!(r.units.len(), 2, "forced K=2 must yield exactly two units");
        // The best 2-split isolates the extreme-slope run from the other two.
        assert!(r.units.iter().map(|u| u.n).sum::<usize>() == 30);
    }

    #[test]
    fn cumulatives_are_normalized_and_monotone() {
        let (d, phi, perm) = column(0.18, &[(8, 500.0), (8, 5.0), (8, 50.0)]);
        let r = compute_lorenz(&d, &phi, &perm, 0);
        assert!(r.error.is_none());
        let p = &r.points;
        // End at ~1, start above 0, both strictly non-decreasing.
        assert!((p.last().unwrap().cum_storage - 1.0).abs() < 1e-6);
        assert!((p.last().unwrap().cum_flow - 1.0).abs() < 1e-6);
        for w in p.windows(2) {
            assert!(w[1].cum_storage >= w[0].cum_storage - 1e-9);
            assert!(w[1].cum_flow >= w[0].cum_flow - 1e-9);
        }
    }

    #[test]
    fn lorenz_coefficient_zero_when_homogeneous() {
        // One uniform k/φ everywhere → the reordered curve is the diagonal → Lc ≈ 0.
        let (d, phi, perm) = column(0.20, &[(30, 100.0)]);
        let r = compute_lorenz(&d, &phi, &perm, 0);
        assert!(r.error.is_none());
        assert!(r.lorenz_coefficient < 0.02, "Lc={}", r.lorenz_coefficient);
        // A single slope level also collapses to ONE flow unit under auto-K.
        assert_eq!(r.units.len(), 1);
    }

    #[test]
    fn lorenz_coefficient_high_when_one_sample_dominates_flow() {
        // 29 tight low-k samples + 1 very-high-k sample → strong flow concentration → high Lc.
        let mut runs = vec![(29usize, 1.0f64)];
        runs.push((1, 1_000_000.0));
        let (d, phi, perm) = column(0.20, &runs);
        let r = compute_lorenz(&d, &phi, &perm, 0);
        assert!(r.error.is_none());
        assert!(r.lorenz_coefficient > 0.5, "Lc={}", r.lorenz_coefficient);
    }

    #[test]
    fn invalid_samples_are_skipped_not_counted() {
        let d = vec![2000.0f32, 2000.5, 2001.0, 2001.5, 2002.0];
        let phi = vec![0.20f32, f32::NAN, 0.0, 1.0, 0.20];
        let perm = vec![100.0f32, 100.0, 100.0, 100.0, -5.0];
        let r = compute_lorenz(&d, &phi, &perm, 0);
        assert!(r.error.is_none());
        assert_eq!(r.n_samples, 1, "only the first sample is valid");
        assert_eq!(r.skipped, 4);
    }

    /// A Lorenz/flow-unit run needs a real permeability curve. If the requested one is absent
    /// the run must FAIL AND NAME IT — never fall through to whatever else the frame happens to
    /// carry. This is silent-wrongness class: a flow-unit split computed off the wrong curve
    /// still produces a plausible Lc and a plausible unit count, and nothing downstream can tell.
    ///
    /// The messages existed in `run_lorenz` but nothing asserted them; `errors_when_no_valid_samples`
    /// is a different claim (every sample invalid, message unchecked) on the pure function.
    ///
    /// The control is the reversed case: swapping which curve is missing must move the name in
    /// the message. A guard that always blamed the same curve, or blamed both, would pass a
    /// one-sided test and still be useless for finding out what is actually wrong.
    #[test]
    fn a_missing_curve_fails_by_name_rather_than_computing_on_another() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, id, "SANDI-LZ", Some("Synthetic"), None, None).unwrap();

        // NPHI carries data; no permeability curve of any name exists in this well.
        let n = 20usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let nan = vec![f32::NAN; n];
        crate::db::insert_standard_curves(
            &conn,
            id,
            depth,
            vec![50.0; n],
            nan.clone(),
            vec![0.20; n],
            vec![2.45; n],
            nan.clone(),
            nan,
        )
        .unwrap();

        let dbm = std::sync::Mutex::new(conn);
        let well = id.to_string();
        let req = |phi: &str, perm: &str| LorenzRequest {
            well_id: well.clone(),
            phi_curve: phi.to_string(),
            perm_curve: perm.to_string(),
            depth_from: None,
            depth_to: None,
            n_units: 0,
        };

        // Permeability absent: refused, and the message names the permeability curve.
        let r = run_lorenz(&dbm, &req("NPHI", "KLOGH"));
        let msg = r.error.expect("a missing permeability curve must fail the run");
        assert!(msg.contains("KLOGH"), "the message must name the missing curve; got {msg}");
        assert!(
            !msg.contains("NPHI"),
            "the porosity curve is present and must not be blamed; got {msg}"
        );
        assert_eq!(r.n_samples, 0, "a refused run must compute nothing");
        assert!(r.units.is_empty(), "a refused run must produce no flow units");

        // Control: move the absence to the porosity side and the name in the message moves too.
        let r2 = run_lorenz(&dbm, &req("PHIT_NOT_HERE", "NPHI"));
        let msg2 = r2.error.expect("a missing porosity curve must also fail the run");
        assert!(
            msg2.contains("PHIT_NOT_HERE"),
            "the message must name the missing porosity curve; got {msg2}"
        );
        assert_eq!(r2.n_samples, 0);
    }

    #[test]
    fn errors_when_no_valid_samples() {
        let d = vec![2000.0f32, 2000.5];
        let phi = vec![0.0f32, 1.0];
        let perm = vec![10.0f32, 10.0];
        let r = compute_lorenz(&d, &phi, &perm, 0);
        assert!(r.error.is_some());
        assert_eq!(r.n_samples, 0);
    }

    #[test]
    fn segment_dp_matches_known_best_split() {
        // Values 3,3,1,1,2,2: the best 2-split is {3,3}{1,1,2,2} (SSE 0+2 = 2), not {3,3,1,1}{2,2}.
        let (sse, arg) = segment_dp(&[3.0, 3.0, 1.0, 1.0, 2.0, 2.0], 3);
        let a2 = backtrack(&arg, 2, 6);
        assert_eq!(a2, vec![0, 0, 1, 1, 1, 1], "assign={a2:?}");
        // The 3-split recovers the three runs exactly (SSE 0).
        assert!(sse[3] < 1e-9, "sse3={}", sse[3]);
        let a3 = backtrack(&arg, 3, 6);
        assert_eq!(a3, vec![0, 0, 1, 1, 2, 2], "assign={a3:?}");
    }

    #[test]
    fn run_lorenz_reads_curves_from_the_store() {
        use crate::db;
        use uuid::Uuid;
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = Uuid::new_v4();
        db::insert_well(&conn, id, "LZ-1", None, None, None).unwrap();
        let n = 30usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        // Standard grid (values immaterial to the Lorenz math; they anchor the depth frame).
        db::insert_standard_curves(
            &conn,
            id,
            depth.clone(),
            vec![40.0; n],
            vec![10.0; n],
            vec![0.2; n],
            vec![2.4; n],
            vec![80.0; n],
            vec![f32::NAN; n],
        )
        .unwrap();
        // PHIE + PERM as computed curves, three slope regimes like the auto-K test.
        let phie = vec![0.20f32; n];
        let mut perm = vec![0.0f32; n];
        for (i, p) in perm.iter_mut().enumerate() {
            let kphi = if i < 10 { 1000.0 } else if i < 20 { 10.0 } else { 100.0 };
            *p = (kphi * 0.20) as f32;
        }
        let ids = id.to_string();
        crate::equations::write_computed_curve(&conn, &ids, &depth, "PHIE", &phie).unwrap();
        crate::equations::write_computed_curve(&conn, &ids, &depth, "PERM", &perm).unwrap();

        let db = Mutex::new(conn);
        let req = LorenzRequest {
            well_id: ids,
            phi_curve: "PHIE".into(),
            perm_curve: "PERM".into(),
            depth_from: None,
            depth_to: None,
            n_units: 0,
        };
        let r = run_lorenz(&db, &req);
        assert!(r.error.is_none(), "err={:?}", r.error);
        assert_eq!(r.n_samples, 30);
        assert_eq!(r.units.len(), 3, "should recover 3 flow units through the DB path");
        assert!(r.lorenz_coefficient > 0.0 && r.lorenz_coefficient < 1.0);
    }
}
