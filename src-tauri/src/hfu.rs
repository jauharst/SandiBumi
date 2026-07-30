//! Hydraulic Flow Unit clustering (Wave B item 8, increment 2 — the data-driven half of rock
//! typing). Where `rocktyping.rs` bins FZI on FIXED global boundaries (Corbett-Potter GHE), this
//! clusters the well set's OWN core φ-k cloud into HFUs, so the boundaries come from the data.
//!
//! FZI (Amaefule 1993): RQI = 0.0314·√(k/φ) [µm]; φz = φ/(1−φ); FZI = RQI/φz. Samples of one HFU
//! fall on a unit-slope line on log-log RQI–φz with intercept FZI_mean at φz = 1. Clustering runs
//! on x = log10(FZI):
//!   - "ward": the global optimum of the Ward within-cluster-variance criterion. 1-D optima are
//!     contiguous after sorting, so an exact O(K·m²) dynamic program finds it (no greedy drift).
//!   - "histogram": boundaries at the deepest antimodes (valleys) of the log10 FZI histogram —
//!     the classic "FZI is log-normal within an HFU, multi-modal across HFUs" break-picking. When
//!     the data has fewer natural valleys than requested, it returns fewer clusters and says so.
//!
//! Per HFU: geometric-mean FZI and the Amaefule inverse permeability transform
//! k = 1014.24·FZI_mean²·φ³/(1−φ)², with the transform's R² (log-k) as a fit-quality readout.
//! Reads `core_data` (routine core φ-k); MICP plugs are a subset, so core is the fuller cloud.

use duckdb::Connection;
use std::sync::Mutex;

/// Amaefule RQI constant (0.0314 makes RQI come out in µm for k[mD], φ[v/v]).
const RQI_C: f64 = 0.0314;
/// Inverse permeability-transform constant 1/RQI_C² = 1014.24 (recovers k from FZI_mean + φ).
const PERM_C: f64 = 1014.24;

#[derive(serde::Deserialize)]
pub struct HfuRequest {
    pub well_ids: Vec<String>,
    /// Requested number of HFUs (clamped to 1..=12 and to the distinct-FZI count).
    pub n_clusters: u32,
    /// "ward" (exact minimum-variance partition) or "histogram" (antimode break-picking).
    pub method: String,
}

#[derive(serde::Serialize)]
pub struct HfuCluster {
    /// 1..K, ascending FZI (HFU 1 = lowest FZI = poorest quality).
    pub hfu: u32,
    pub n: usize,
    pub fzi_min: f64,
    pub fzi_max: f64,
    /// Geometric mean FZI (= unit-slope-line intercept at φz = 1).
    pub fzi_gm: f64,
    pub poro_mean: f64,
    /// R² (log-k) of k = 1014.24·FZI_gm²·φ³/(1−φ)² against the cluster's measured k. 1.0 for a
    /// single plug (the transform is exact); lower for a spread cluster.
    pub perm_r2: f64,
}

#[derive(serde::Serialize)]
pub struct HfuPoint {
    pub well_name: String,
    pub depth: Option<f32>,
    pub poro: f64,
    pub perm: f64,
    pub rqi: f64,
    pub phiz: f64,
    pub fzi: f64,
    pub hfu: u32,
}

#[derive(serde::Serialize)]
pub struct HfuResult {
    pub clusters: Vec<HfuCluster>,
    pub points: Vec<HfuPoint>,
    /// K−1 FZI cut values (ascending) separating the HFUs.
    pub boundaries: Vec<f64>,
    pub method: String,
    pub n_plugs: usize,
    pub skipped: usize,
    /// Set when the result deviates from the request (fewer clusters than asked, etc.).
    pub note: Option<String>,
    pub error: Option<String>,
}

fn hfu_err(msg: &str) -> HfuResult {
    HfuResult {
        clusters: vec![],
        points: vec![],
        boundaries: vec![],
        method: String::new(),
        n_plugs: 0,
        skipped: 0,
        note: None,
        error: Some(msg.to_string()),
    }
}

/// One core plug carried through clustering (already screened to finite φ∈(0,1), k>0).
struct Plug {
    well_name: String,
    depth: Option<f32>,
    poro: f64,
    perm: f64,
    rqi: f64,
    phiz: f64,
    fzi: f64,
    x: f64, // log10(fzi)
}

/// Exact optimal K-partition of a SORTED slice minimizing total within-cluster sum of squares
/// (the Ward criterion). Returns the 0-based cluster id of each element in sorted order.
fn ward_partition(sorted: &[f64], k: usize) -> Vec<usize> {
    let m = sorted.len();
    if k <= 1 || m == 0 {
        return vec![0; m];
    }
    // Prefix sums for O(1) segment SS: cost[a,b) = Σx² − (Σx)²/(b−a).
    let mut ps = vec![0.0f64; m + 1];
    let mut ps2 = vec![0.0f64; m + 1];
    for i in 0..m {
        ps[i + 1] = ps[i] + sorted[i];
        ps2[i + 1] = ps2[i] + sorted[i] * sorted[i];
    }
    let cost = |a: usize, b: usize| -> f64 {
        let cnt = (b - a) as f64;
        if cnt <= 0.0 {
            return 0.0;
        }
        let s = ps[b] - ps[a];
        (ps2[b] - ps2[a] - s * s / cnt).max(0.0)
    };
    let k = k.min(m);
    let inf = f64::INFINITY;
    // dp[j][i] = min WCSS of the first i elements split into j clusters.
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

/// Boundaries (in x = log10 FZI) at up to K−1 deepest interior antimodes of the histogram of
/// `sorted`. Greedy by valley depth with a one-bin minimum separation, then sorted ascending.
fn histogram_boundaries(sorted: &[f64], k: usize) -> Vec<f64> {
    let m = sorted.len();
    if k <= 1 || m < 4 {
        return vec![];
    }
    let (xmin, xmax) = (sorted[0], sorted[m - 1]);
    if !(xmax > xmin) {
        return vec![];
    }
    let bins = ((m as f64).sqrt().round() as usize).clamp(8, 40);
    let width = (xmax - xmin) / bins as f64;
    let mut hist = vec![0usize; bins];
    for &x in sorted {
        let b = (((x - xmin) / width) as usize).min(bins - 1);
        hist[b] += 1;
    }
    // Interior local minima, scored by valley depth = min(tallest bin on each side) − valley.
    let mut valleys: Vec<(usize, usize)> = vec![]; // (depth, bin)
    for b in 1..bins - 1 {
        let is_min = hist[b] <= hist[b - 1]
            && hist[b] <= hist[b + 1]
            && (hist[b] < hist[b - 1] || hist[b] < hist[b + 1]);
        if !is_min {
            continue;
        }
        let left_peak = hist[..b].iter().copied().max().unwrap_or(0);
        let right_peak = hist[b + 1..].iter().copied().max().unwrap_or(0);
        let depth = left_peak.min(right_peak).saturating_sub(hist[b]);
        if depth > 0 {
            valleys.push((depth, b));
        }
    }
    // Deepest first; accept only if ≥1 bin from every already-accepted valley.
    valleys.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut chosen: Vec<usize> = vec![];
    for (_, b) in valleys {
        if chosen.len() >= k - 1 {
            break;
        }
        if chosen.iter().all(|&c| (c as isize - b as isize).abs() > 1) {
            chosen.push(b);
        }
    }
    chosen.sort_unstable();
    chosen.iter().map(|&b| xmin + (b as f64 + 0.5) * width).collect()
}

/// Cluster id (count of boundaries strictly below x) for the histogram assignment.
fn assign_by_boundaries(x: f64, boundaries: &[f64]) -> usize {
    boundaries.iter().filter(|&&b| x >= b).count()
}

pub fn run_hfu_cluster(db: &Mutex<Connection>, req: &HfuRequest) -> HfuResult {
    if req.well_ids.is_empty() {
        return hfu_err("no wells in scope");
    }
    let requested = (req.n_clusters as usize).clamp(1, 12);
    let method = if req.method == "histogram" { "histogram" } else { "ward" };

    // Gather + screen core plugs across the scoped wells.
    let mut plugs: Vec<Plug> = vec![];
    let mut skipped = 0usize;
    {
        let conn = db.lock().unwrap();
        for wid in &req.well_ids {
            let well_name: String = conn
                .query_row("SELECT well_name FROM wells WHERE well_id = ?1", [wid], |r| r.get(0))
                .unwrap_or_else(|_| wid.clone());
            let rows = match crate::db::get_core_plugs(&conn, wid) {
                Ok(r) => r,
                Err(e) => return hfu_err(&format!("{well_name}: {e}")),
            };
            for r in rows {
                let phi = r.cpor as f64;
                let k = r.cperm as f64;
                if !(phi.is_finite() && k.is_finite()) || phi <= 0.0 || phi >= 1.0 || k <= 0.0 {
                    skipped += 1;
                    continue;
                }
                let rqi = RQI_C * (k / phi).sqrt();
                let phiz = phi / (1.0 - phi);
                let fzi = if phiz > 0.0 { rqi / phiz } else { f64::NAN };
                if !(fzi.is_finite() && fzi > 0.0) {
                    skipped += 1;
                    continue;
                }
                plugs.push(Plug {
                    well_name: well_name.clone(),
                    depth: Some(r.depth),
                    poro: phi,
                    perm: k,
                    rqi,
                    phiz,
                    fzi,
                    x: fzi.log10(),
                });
            }
        }
    }
    let n_plugs = plugs.len();
    if n_plugs == 0 {
        return hfu_err(
            "no core plugs with valid φ (0–1) and k (>0) in the selected wells — import core data first",
        );
    }

    // Sort indices by x; count distinct FZI levels so we never ask for more clusters than exist.
    let mut order: Vec<usize> = (0..n_plugs).collect();
    order.sort_by(|&a, &b| plugs[a].x.partial_cmp(&plugs[b].x).unwrap_or(std::cmp::Ordering::Equal));
    let sorted_x: Vec<f64> = order.iter().map(|&i| plugs[i].x).collect();
    let distinct = {
        let mut d = 1usize;
        for w in sorted_x.windows(2) {
            if (w[1] - w[0]).abs() > 1e-9 {
                d += 1;
            }
        }
        d
    };
    let eff_k = requested.min(distinct).max(1);

    // Assign each plug (original order) to a raw cluster id, ascending FZI.
    let mut hfu_of = vec![0usize; n_plugs];
    if method == "histogram" {
        let cuts = histogram_boundaries(&sorted_x, eff_k);
        for (i, p) in plugs.iter().enumerate() {
            hfu_of[i] = assign_by_boundaries(p.x, &cuts);
        }
    } else {
        let assign_sorted = ward_partition(&sorted_x, eff_k);
        for (pos, &orig) in order.iter().enumerate() {
            hfu_of[orig] = assign_sorted[pos];
        }
    }

    // Collapse raw ids to CONTIGUOUS ascending ids. The histogram path can flag two valleys that
    // flank an empty bin range, leaving an interior cluster with zero plugs; without this remap the
    // emitted HFU ids would skip a number (e.g. {1,3}) and boundaries.len() would not equal K−1.
    let mut used: Vec<usize> = hfu_of.clone();
    used.sort_unstable();
    used.dedup();
    let remap: std::collections::HashMap<usize, usize> =
        used.iter().enumerate().map(|(new, &old)| (old, new)).collect();
    for h in hfu_of.iter_mut() {
        *h = remap[h];
    }
    let n_clusters_actual = used.len();

    // Boundaries from the FINAL assignment along sorted x: the geometric midpoint at each cluster
    // transition. Unified across both methods → boundaries.len() == n_clusters_actual − 1 always,
    // and every cut sits between two populated clusters (never in an empty gap).
    let mut boundaries_x: Vec<f64> = vec![];
    for pos in 1..order.len() {
        if hfu_of[order[pos]] != hfu_of[order[pos - 1]] {
            boundaries_x.push((sorted_x[pos] + sorted_x[pos - 1]) / 2.0);
        }
    }

    // Note only when the delivered unit count falls short of the request — either the histogram
    // found fewer natural breaks, or the data has too few distinct FZI levels.
    let note = if n_clusters_actual < requested {
        if method == "histogram" && n_clusters_actual < eff_k {
            Some(format!(
                "histogram found only {n_clusters_actual} natural HFU level(s) for {n_plugs} plug(s) (requested {requested})"
            ))
        } else {
            Some(format!(
                "using {n_clusters_actual} HFU(s) — the data has only {distinct} distinct FZI level(s) (requested {requested})"
            ))
        }
    } else {
        None
    };

    // Per-cluster stats. hfu id (1-based) = cluster index + 1, ascending FZI.
    let mut clusters: Vec<HfuCluster> = Vec::with_capacity(n_clusters_actual);
    for c in 0..n_clusters_actual {
        let members: Vec<&Plug> = plugs.iter().enumerate().filter(|(i, _)| hfu_of[*i] == c).map(|(_, p)| p).collect();
        if members.is_empty() {
            continue;
        }
        let n = members.len();
        let ln_mean = members.iter().map(|p| p.x).sum::<f64>() / n as f64;
        let fzi_gm = 10f64.powf(ln_mean);
        let fzi_min = members.iter().map(|p| p.fzi).fold(f64::INFINITY, f64::min);
        let fzi_max = members.iter().map(|p| p.fzi).fold(f64::NEG_INFINITY, f64::max);
        let poro_mean = members.iter().map(|p| p.poro).sum::<f64>() / n as f64;

        // R² (log-k) of the geometric-mean-FZI transform against measured k.
        let ys: Vec<f64> = members.iter().map(|p| p.perm.log10()).collect();
        let yhats: Vec<f64> = members
            .iter()
            .map(|p| (PERM_C * fzi_gm * fzi_gm * p.poro.powi(3) / (1.0 - p.poro).powi(2)).log10())
            .collect();
        let ybar = ys.iter().sum::<f64>() / n as f64;
        let ss_res: f64 = ys.iter().zip(&yhats).map(|(y, yh)| (y - yh).powi(2)).sum();
        let ss_tot: f64 = ys.iter().map(|y| (y - ybar).powi(2)).sum();
        let perm_r2 = if ss_tot > 1e-12 {
            (1.0 - ss_res / ss_tot).clamp(0.0, 1.0)
        } else if ss_res < 1e-12 {
            1.0
        } else {
            0.0
        };

        clusters.push(HfuCluster {
            hfu: (c + 1) as u32,
            n,
            fzi_min,
            fzi_max,
            fzi_gm,
            poro_mean,
            perm_r2,
        });
    }

    let points: Vec<HfuPoint> = plugs
        .iter()
        .enumerate()
        .map(|(i, p)| HfuPoint {
            well_name: p.well_name.clone(),
            depth: p.depth,
            poro: p.poro,
            perm: p.perm,
            rqi: p.rqi,
            phiz: p.phiz,
            fzi: p.fzi,
            hfu: (hfu_of[i] + 1) as u32,
        })
        .collect();

    let boundaries: Vec<f64> = boundaries_x.iter().map(|&x| 10f64.powf(x)).collect();

    HfuResult {
        clusters,
        points,
        boundaries,
        method: method.to_string(),
        n_plugs,
        skipped,
        note,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FZI for a plug, matching the module math (for building synthetic inputs).
    fn fzi_of(phi: f64, k: f64) -> f64 {
        let rqi = RQI_C * (k / phi).sqrt();
        rqi / (phi / (1.0 - phi))
    }

    /// Given a target FZI and porosity, the permeability that produces it (inverse transform).
    fn k_for_fzi(fzi: f64, phi: f64) -> f64 {
        PERM_C * fzi * fzi * phi.powi(3) / (1.0 - phi).powi(2)
    }

    #[test]
    fn ward_partition_splits_two_separated_groups() {
        // Six points in two tight, well-separated bands → the 2-partition must cut at the gap.
        let xs = [0.0, 0.05, 0.1, 2.0, 2.05, 2.1];
        let a = ward_partition(&xs, 2);
        assert_eq!(a, vec![0, 0, 0, 1, 1, 1], "got {a:?}");
    }

    #[test]
    fn ward_partition_k1_is_all_one_cluster() {
        assert_eq!(ward_partition(&[1.0, 2.0, 3.0], 1), vec![0, 0, 0]);
    }

    #[test]
    fn histogram_finds_the_bimodal_valley() {
        // Two dense modes at ~0 and ~2 with a sparse gap → exactly one boundary in the gap.
        let mut xs: Vec<f64> = vec![];
        for i in 0..20 {
            xs.push(0.0 + (i as f64) * 0.01);
        }
        for i in 0..20 {
            xs.push(2.0 + (i as f64) * 0.01);
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let b = histogram_boundaries(&xs, 2);
        assert_eq!(b.len(), 1, "boundaries={b:?}");
        assert!(b[0] > 0.3 && b[0] < 1.9, "valley at {b:?}");
    }

    fn make_conn_with_core(plugs: &[(f32, f32, f32)]) -> (Connection, String) {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "HFU-1", None, None, None).unwrap();
        let depths: Vec<f32> = plugs.iter().map(|p| p.0).collect();
        let cpor: Vec<f32> = plugs.iter().map(|p| p.1).collect();
        let cperm: Vec<f32> = plugs.iter().map(|p| p.2).collect();
        let cgd = vec![f32::NAN; plugs.len()];
        let csw = vec![f32::NAN; plugs.len()];
        crate::db::insert_core_data(&conn, &wid.to_string(), "RAW", None, &depths, &cpor, &cperm, &cgd, &csw).unwrap();
        (conn, wid.to_string())
    }

    #[test]
    fn run_hfu_ward_groups_core_plugs_by_fzi() {
        // Two known FZI bands (0.5 and 4.0), 4 plugs each at assorted porosities.
        let mut rows: Vec<(f32, f32, f32)> = vec![];
        let mut depth = 2000.0f32;
        for &fzi in &[0.5f64, 4.0] {
            for &phi in &[0.10f64, 0.15, 0.20, 0.25] {
                let k = k_for_fzi(fzi, phi);
                rows.push((depth, phi as f32, k as f32));
                depth += 1.0;
            }
        }
        let (conn, wid) = make_conn_with_core(&rows);
        let db = Mutex::new(conn);
        let res = run_hfu_cluster(&db, &HfuRequest { well_ids: vec![wid], n_clusters: 2, method: "ward".into() });
        assert!(res.error.is_none(), "err={:?}", res.error);
        assert_eq!(res.clusters.len(), 2);
        assert_eq!(res.n_plugs, 8);
        // Ascending FZI: cluster 1 ≈ 0.5, cluster 2 ≈ 4.0.
        assert!((res.clusters[0].fzi_gm - 0.5).abs() < 0.05, "gm1={}", res.clusters[0].fzi_gm);
        assert!((res.clusters[1].fzi_gm - 4.0).abs() < 0.2, "gm2={}", res.clusters[1].fzi_gm);
        // Each band is exactly one FZI, so the transform recovers k perfectly.
        assert!(res.clusters[0].perm_r2 > 0.999);
        assert!(res.clusters[1].perm_r2 > 0.999);
        // One boundary, between the two bands.
        assert_eq!(res.boundaries.len(), 1);
        assert!(res.boundaries[0] > 0.5 && res.boundaries[0] < 4.0, "bnd={:?}", res.boundaries);
        // Every point carries an HFU in 1..=2.
        assert!(res.points.iter().all(|p| p.hfu == 1 || p.hfu == 2));
        assert_eq!(res.points.iter().filter(|p| p.hfu == 1).count(), 4);
    }

    #[test]
    fn run_hfu_skips_invalid_and_notes_capped_k() {
        // Two valid plugs at the SAME FZI + two invalid (φ=0, k<0). Ask for 3 clusters → capped
        // to 1 distinct level, skipped=2, and a note explaining the cap.
        let phi = 0.20f64;
        let k = k_for_fzi(1.0, phi) as f32;
        let rows = vec![
            (2000.0f32, phi as f32, k),
            (2001.0, phi as f32, k),
            (2002.0, 0.0, 10.0),   // φ=0 invalid
            (2003.0, 0.20, -3.0),  // k<0 invalid
        ];
        let (conn, wid) = make_conn_with_core(&rows);
        let db = Mutex::new(conn);
        let res = run_hfu_cluster(&db, &HfuRequest { well_ids: vec![wid], n_clusters: 3, method: "ward".into() });
        assert!(res.error.is_none());
        assert_eq!(res.n_plugs, 2);
        assert_eq!(res.skipped, 2);
        assert_eq!(res.clusters.len(), 1);
        assert!(res.note.is_some(), "expected a cap note");
        assert!((res.clusters[0].fzi_gm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn run_hfu_histogram_keeps_hfu_ids_contiguous_across_empty_gap() {
        // Two tight, far-apart FZI modes (~0.3 and ~8) with nothing between → the histogram flags a
        // valley on each shoulder of the empty gap. Requesting K=3 must NOT emit HFU {1,3}: the empty
        // interior unit is collapsed so ids stay contiguous and boundaries == clusters − 1.
        let mut rows: Vec<(f32, f32, f32)> = vec![];
        let mut depth = 2000.0f32;
        for &fzi in &[0.3f64, 8.0] {
            for &phi in &[0.10f64, 0.14, 0.18, 0.22, 0.26] {
                rows.push((depth, phi as f32, k_for_fzi(fzi, phi) as f32));
                depth += 1.0;
            }
        }
        let (conn, wid) = make_conn_with_core(&rows);
        let db = Mutex::new(conn);
        let res = run_hfu_cluster(&db, &HfuRequest { well_ids: vec![wid], n_clusters: 3, method: "histogram".into() });
        assert!(res.error.is_none(), "err={:?}", res.error);
        // Contiguous ids 1..=len, no gaps.
        let ids: Vec<u32> = res.clusters.iter().map(|c| c.hfu).collect();
        assert_eq!(ids, (1..=res.clusters.len() as u32).collect::<Vec<_>>(), "ids={ids:?}");
        // Contract: exactly one cut per adjacent-cluster pair, each in a populated gap.
        assert_eq!(res.boundaries.len(), res.clusters.len().saturating_sub(1));
        // Every point lands in a real unit.
        assert!(res.points.iter().all(|p| p.hfu >= 1 && p.hfu as usize <= res.clusters.len()));
        // The two modes are separable, so we get 2 units (not 3), and the shortfall is noted.
        assert_eq!(res.clusters.len(), 2, "expected 2 populated units");
        assert!(res.note.is_some(), "shortfall should be noted");
    }

    #[test]
    fn run_hfu_errors_when_no_valid_plugs() {
        let rows = vec![(2000.0f32, 0.0f32, 10.0f32), (2001.0, 0.20, -1.0)];
        let (conn, wid) = make_conn_with_core(&rows);
        let db = Mutex::new(conn);
        let res = run_hfu_cluster(&db, &HfuRequest { well_ids: vec![wid], n_clusters: 2, method: "ward".into() });
        assert!(res.error.is_some());
        assert_eq!(res.n_plugs, 0);
    }

    #[test]
    fn fzi_helpers_are_self_consistent() {
        // k_for_fzi ∘ fzi_of round-trips (guards the test fixtures themselves). Tolerance is 1e-5,
        // not 0: the published constants 0.0314 and 1014.24 are not exact reciprocal-squares
        // (1/0.0314² = 1014.24001), so the round trip carries ~1e-7 — kept as the literature values.
        let (phi, target) = (0.18, 2.5);
        let k = k_for_fzi(target, phi);
        assert!((fzi_of(phi, k) - target).abs() < 1e-5, "got {}", fzi_of(phi, k));
    }
}
