//! Thomeer capillary-pressure hyperbola fitting (Wave B item 8, increment 2 — MICP side).
//!
//! Thomeer (1960) models the mercury-injection curve per pore system as a hyperbola in
//! log Pc – log Bv space: Bv(Pc) = Bv∞ · exp(−G / log10(Pc/Pd)) for Pc > Pd, else 0,
//! where Bv = φ·S_nw is the invaded BULK-volume fraction, Pd the displacement (entry)
//! pressure, G the pore geometrical factor (typ. 0.1–1, lower = better sorted) and Bv∞
//! the asymptotic invaded volume. The log10 form follows the published convention
//! (Thomeer 1960; Clerke's Arab-D work), which matches the reference notes' G range.
//!
//! Per-sample fits give the (Pd, G) plane used for Thomeer-class rock typing, plus the
//! Swanson apex point. This increment fits ONE pore system per sample; multi-modal rocks
//! (2–3 stacked systems, detected from dBv/dlogPc) are a later increment. The Swanson
//! permeability k = 399·(Bv%/Pc)apex^1.691 (Swanson 1981) ships flagged: verify the
//! constants against the paper before field release (same policy as the PGS exponent).
//!
//! STANDARDIZATION: Pc is converted to the Hg-air system (×367/σcosθ, using the per-row
//! `ift` stored at import) BEFORE fitting — G is invariant under that scaling, so only Pd
//! and the apex move, and plugs measured in different lab systems land comparably on one
//! Pd–G plane (the reference doc's "standardize Pc" calibration step). Legacy rows with
//! no recorded ift fit on their raw Pc, are marked unstandardized, and get NO Swanson k
//! (the correlation is a mercury-system relation).

use crate::shf_fit::nelder_mead;
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Thomeer model evaluation: invaded bulk-volume fraction at Pc.
pub fn thomeer_bv(pd: f64, g: f64, bv_inf: f64, pc: f64) -> f64 {
    if pc <= pd || pd <= 0.0 {
        return 0.0;
    }
    let l = (pc / pd).log10();
    if l <= 0.0 {
        return 0.0;
    }
    bv_inf * (-g / l).exp()
}

/// Core parameters of one fitted Thomeer hyperbola.
#[derive(Debug, Clone, Copy)]
pub struct ThomeerCore {
    pub pd: f64,
    pub g: f64,
    pub bv_inf: f64,
    pub r2: f64,
    pub n: usize,
    /// True when the optimizer pinned Pd at a search bound — an entry-truncated curve
    /// (first pressure step already above Pd) or barely-invaded data. The Pd value is
    /// then a bound artifact, not a resolved entry pressure.
    pub pd_at_bound: bool,
}

/// Fits (Pd, G, Bv∞) to per-sample (Pc, Bv) points by bounded Nelder-Mead on the sum of
/// squared Bv residuals (linear Bv keeps the many near-zero points below entry from
/// dominating, unlike a log-Bv loss). Needs ≥ 4 usable points, ≥ 3 of them invaded
/// (Bv > 0). Pd is searched in log10 space.
pub fn fit_thomeer(points: &[(f64, f64)]) -> Option<ThomeerCore> {
    let clean: Vec<(f64, f64)> = points
        .iter()
        .copied()
        .filter(|&(pc, bv)| pc.is_finite() && bv.is_finite() && pc > 0.0 && bv >= 0.0)
        .collect();
    let invaded: Vec<(f64, f64)> = clean.iter().copied().filter(|&(_, bv)| bv > 0.0).collect();
    if clean.len() < 4 || invaded.len() < 3 {
        return None;
    }

    let max_bv = invaded.iter().map(|&(_, bv)| bv).fold(0.0f64, f64::max);
    let min_pc = clean.iter().map(|&(pc, _)| pc).fold(f64::INFINITY, f64::min);
    let max_pc = clean.iter().map(|&(pc, _)| pc).fold(0.0f64, f64::max);
    // Entry estimate: the lowest Pc that is meaningfully invaded (> 2% of the plateau).
    let entry = invaded
        .iter()
        .filter(|&&(_, bv)| bv > 0.02 * max_bv)
        .map(|&(pc, _)| pc)
        .fold(f64::INFINITY, f64::min)
        .min(max_pc);

    let loss = |x: &[f64; 4]| -> f64 {
        let (pd, g, bv_inf) = (10f64.powf(x[0]), x[1], x[2]);
        clean
            .iter()
            .map(|&(pc, bv)| {
                let e = thomeer_bv(pd, g, bv_inf, pc) - bv;
                e * e
            })
            .sum()
    };
    let x0 = [(entry * 0.8).max(min_pc * 0.5).log10(), 0.4, max_bv * 1.02, 0.0];
    let lo = [(min_pc * 0.3).log10(), 0.01, max_bv * 0.8, 0.0];
    let hi = [max_pc.log10(), 5.0, max_bv * 2.5, 0.0];
    let best = nelder_mead(loss, x0, lo, hi, 500);

    let (pd, g, bv_inf) = (10f64.powf(best[0]), best[1], best[2]);
    let mean_bv = clean.iter().map(|&(_, bv)| bv).sum::<f64>() / clean.len() as f64;
    let ss_tot: f64 = clean.iter().map(|&(_, bv)| (bv - mean_bv) * (bv - mean_bv)).sum();
    let ss_res = loss(&best);
    // All-equal-Bv data (plateau-only ladder) has ss_tot = 0; a near-zero residual is
    // then a PERFECT fit of the constant, not the worst one.
    let r2 = if ss_tot > 0.0 {
        (1.0 - ss_res / ss_tot).clamp(0.0, 1.0)
    } else if ss_res < 1e-12 {
        1.0
    } else {
        0.0
    };
    let pd_at_bound = (best[0] - lo[0]).abs() < 1e-6 || (hi[0] - best[0]).abs() < 1e-6;
    Some(ThomeerCore { pd, g, bv_inf, r2, n: clean.len(), pd_at_bound })
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThomeerRequest {
    pub well_ids: Vec<String>,
}

/// One plug's fitted Thomeer hyperbola with its data for the QC plot.
#[derive(Debug, Clone, Serialize)]
pub struct ThomeerSampleFit {
    pub well_name: String,
    pub sample_no: Option<i32>,
    pub depth: Option<f32>,
    pub perm: f64,
    pub poro: f64,
    /// Displacement pressure, psi — Hg-air EQUIVALENT when `standardized` (see module doc).
    pub pd: f64,
    pub g: f64,
    pub bv_inf: f64,
    pub r2: f64,
    pub n: usize,
    /// True when Pd pinned at a search bound (entry-truncated / barely-invaded curve) —
    /// the Pd is then an artifact, not a resolved entry pressure.
    pub pd_at_bound: bool,
    /// Lab fluid system of this plug's points (from import), for display.
    pub system: Option<String>,
    /// True when every point carried a σcosθ and Pc was converted to Hg-air equivalent.
    pub standardized: bool,
    /// Swanson apex max(Bv/Pc) over the DATA points (Bv fraction, Pc Hg-air-equiv psi).
    pub apex_bv_pc: f64,
    /// Swanson (1981) k_air = 399·(Bv%/Pc)apex^1.691, mD. Only computed on standardized
    /// (Hg-air-equivalent) data — the correlation is a mercury-system relation; NaN/null
    /// otherwise. Constants flagged for verification before field release (module doc).
    pub swanson_k: f64,
    /// (Pc, Bv) data points, Pc ascending (Hg-air equivalent when `standardized`).
    pub scatter: Vec<[f64; 2]>,
    /// Fitted (Pc, Bv) curve, log-spaced from just above Pd.
    pub curve: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThomeerResult {
    pub fits: Vec<ThomeerSampleFit>,
    /// Samples seen but skipped (no porosity, or too few Pc points to fit).
    pub skipped: usize,
    pub error: Option<String>,
}

fn thomeer_err(msg: &str) -> ThomeerResult {
    ThomeerResult { fits: Vec::new(), skipped: 0, error: Some(msg.into()) }
}

/// σcosθ of the mercury-air system — the standardization target (see module doc).
const HG_AIR_IFT: f64 = 367.0;

/// Fits a Thomeer hyperbola per plug across the selected wells' `scal_pc` points.
/// Bv = φ_plug · (1 − Sw); plugs without a porosity (Bv undefined) are skipped and
/// counted. Grouping is per WELL ID, then by sample_no when the delivery numbers its
/// plugs (depth is display-only then — blank depth cells must not split a plug), falling
/// back to depth for unnumbered plugs so those at distinct depths stay separate.
pub fn run_thomeer_fit(db: &Mutex<Connection>, req: &ThomeerRequest) -> ThomeerResult {
    if req.well_ids.is_empty() {
        return thomeer_err("select at least one well");
    }

    // (well_id, sample_no, depth_bits-when-unnumbered) -> per-plug accumulator.
    type Key = (String, Option<i32>, Option<u32>);
    struct Plug {
        well_name: String,
        poro: f64,
        perm: f64,
        depth: Option<f32>,
        system: Option<String>,
        all_ift: bool,
        pcsw: Vec<(f64, f64)>, // (Pc Hg-air-equivalent where ift known, Sw)
    }
    let mut groups: std::collections::BTreeMap<Key, Plug> = std::collections::BTreeMap::new();
    {
        let conn = db.lock().unwrap();
        for wid in &req.well_ids {
            let well_name: String = conn
                .query_row("SELECT well_name FROM wells WHERE well_id = ?1", [wid], |r| r.get(0))
                .unwrap_or_else(|_| wid.clone());
            let rows = match crate::db::get_scal_pc(&conn, wid) {
                Ok(r) => r,
                Err(e) => return thomeer_err(&format!("{well_name}: {e}")),
            };
            for r in rows {
                // Numbered plugs key on the number alone; only unnumbered ones fall back
                // to depth (a blank depth cell must not split a numbered plug in two).
                let key = match r.sample_no {
                    Some(s) => (wid.clone(), Some(s), None),
                    None => (wid.clone(), None, r.depth.map(f32::to_bits)),
                };
                let entry = groups.entry(key).or_insert_with(|| Plug {
                    well_name: well_name.clone(),
                    poro: f64::NAN,
                    perm: f64::NAN,
                    depth: None,
                    system: None,
                    all_ift: true,
                    pcsw: Vec::new(),
                });
                if entry.poro.is_nan() && r.poro.is_finite() {
                    entry.poro = r.poro as f64;
                }
                if entry.perm.is_nan() && r.perm.is_finite() {
                    entry.perm = r.perm as f64;
                }
                if entry.depth.is_none() {
                    entry.depth = r.depth;
                }
                if entry.system.is_none() {
                    entry.system = r.system.clone();
                }
                // Standardize Pc to Hg-air equivalent when the σcosθ is recorded.
                let scale = r.ift.filter(|v| v.is_finite() && *v > 0.0).map(|v| HG_AIR_IFT / v as f64);
                if scale.is_none() {
                    entry.all_ift = false;
                }
                entry.pcsw.push((r.pc as f64 * scale.unwrap_or(1.0), r.sw as f64));
            }
        }
    }
    if groups.is_empty() {
        return thomeer_err("no SCAL Pc points in the selected wells — import SCAL data first");
    }

    let mut fits = Vec::new();
    let mut skipped = 0usize;
    for ((_wid, sample_no, _dbits), plug) in groups {
        let Plug { well_name, poro, perm, depth, system, all_ift, pcsw } = plug;
        if !(poro.is_finite() && poro > 0.0) {
            skipped += 1; // Bv needs the plug porosity
            continue;
        }
        let mut pts: Vec<(f64, f64)> = pcsw
            .iter()
            .filter(|&&(pc, sw)| pc.is_finite() && sw.is_finite() && pc > 0.0 && (0.0..=1.0).contains(&sw))
            .map(|&(pc, sw)| (pc, poro * (1.0 - sw)))
            .collect();
        pts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let Some(core) = fit_thomeer(&pts) else {
            skipped += 1;
            continue;
        };

        let apex = pts
            .iter()
            .filter(|&&(pc, bv)| pc > 0.0 && bv > 0.0)
            .map(|&(pc, bv)| bv / pc)
            .fold(0.0f64, f64::max);
        // Mercury-system correlation — meaningless on unstandardized (unknown-σcosθ) Pc.
        let swanson_k =
            if all_ift && apex > 0.0 { 399.0 * (apex * 100.0).powf(1.691) } else { f64::NAN };

        let max_pc = pts.last().map(|&(pc, _)| pc).unwrap_or(core.pd * 100.0);
        let mut curve = Vec::with_capacity(60);
        let lo = (core.pd * 1.02).ln();
        let hi = (max_pc * 1.5).max(core.pd * 2.0).ln();
        for i in 0..60 {
            let pc = (lo + (hi - lo) * i as f64 / 59.0).exp();
            curve.push([pc, thomeer_bv(core.pd, core.g, core.bv_inf, pc)]);
        }

        fits.push(ThomeerSampleFit {
            well_name,
            sample_no,
            depth,
            perm,
            poro,
            pd: core.pd,
            g: core.g,
            bv_inf: core.bv_inf,
            r2: core.r2,
            n: core.n,
            pd_at_bound: core.pd_at_bound,
            system,
            standardized: all_ift,
            apex_bv_pc: apex,
            swanson_k,
            scatter: pts.iter().map(|&(pc, bv)| [pc, bv]).collect(),
            curve,
        });
    }
    if fits.is_empty() {
        return ThomeerResult {
            fits,
            skipped,
            error: Some("no fittable plugs (each needs a porosity and ≥ 4 Pc points, ≥ 3 invaded)".into()),
        };
    }
    ThomeerResult { fits, skipped, error: None }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth_points(pd: f64, g: f64, bv_inf: f64) -> Vec<(f64, f64)> {
        // Log-spaced MICP-style pressure ladder 1..2000 psi, incl. points below entry.
        (0..24)
            .map(|i| {
                let pc = 10f64.powf(0.0 + 3.3 * i as f64 / 23.0);
                (pc, thomeer_bv(pd, g, bv_inf, pc))
            })
            .collect()
    }

    #[test]
    fn thomeer_recovers_synthetic_hyperbola() {
        let (pd, g, bv_inf) = (10.0, 0.5, 0.15);
        let fit = fit_thomeer(&synth_points(pd, g, bv_inf)).expect("fit should solve");
        assert!(fit.r2 > 0.98, "r2={}", fit.r2);
        assert!((fit.pd - pd).abs() / pd < 0.25, "pd={}", fit.pd);
        assert!((fit.g - g).abs() < 0.2, "g={}", fit.g);
        assert!((fit.bv_inf - bv_inf).abs() / bv_inf < 0.15, "bv_inf={}", fit.bv_inf);
    }

    #[test]
    fn thomeer_rejects_too_few_points() {
        assert!(fit_thomeer(&[(1.0, 0.0), (10.0, 0.05), (100.0, 0.1)]).is_none(), "3 points is too few");
        assert!(
            fit_thomeer(&[(1.0, 0.0), (2.0, 0.0), (10.0, 0.05), (100.0, 0.1)]).is_none(),
            "needs 3 invaded points"
        );
    }

    #[test]
    fn run_thomeer_groups_plugs_and_skips_poroless() {
        use duckdb::params;
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "THOM-1", None, None, None).unwrap();
        let ids = wid.to_string();

        // Two plugs with distinct known hyperbolas + one poro-less plug that must be skipped.
        let mut rows = Vec::new();
        for (sample, depth, poro, pd, g, bv_inf) in
            [(1, 2000.0f32, 0.20f32, 8.0, 0.4, 0.16), (2, 2010.0, 0.12, 40.0, 0.8, 0.08)]
        {
            for (pc, bv) in synth_points(pd, g, bv_inf) {
                rows.push(crate::db::ScalPcRow {
                    sample_no: Some(sample),
                    depth: Some(depth),
                    perm: 100.0,
                    poro,
                    pc: pc as f32,
                    sw: (1.0 - bv / poro as f64) as f32,
                    system: Some("hg_air".into()),
                    ift: Some(367.0),
                });
            }
        }
        rows.push(crate::db::ScalPcRow {
            sample_no: Some(3),
            depth: Some(2020.0),
            perm: f32::NAN,
            poro: f32::NAN, // no porosity — Bv undefined
            pc: 10.0,
            sw: 0.5,
            system: None,
            ift: None,
        });
        crate::db::insert_scal_pc(&conn, &ids, &rows).unwrap();
        // sanity: the new columns round-trip
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM scal_pc WHERE system = 'hg_air' AND ift = 367.0", params![], |r| r.get(0))
            .unwrap();
        assert_eq!(n as usize, rows.len() - 1);

        let db = Mutex::new(conn);
        let res = run_thomeer_fit(&db, &ThomeerRequest { well_ids: vec![ids] });
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.fits.len(), 2, "two fittable plugs");
        assert_eq!(res.skipped, 1, "the poro-less plug is skipped, not fitted");
        let f1 = res.fits.iter().find(|f| f.sample_no == Some(1)).unwrap();
        let f2 = res.fits.iter().find(|f| f.sample_no == Some(2)).unwrap();
        assert!((f1.pd - 8.0).abs() / 8.0 < 0.3, "pd1={}", f1.pd);
        assert!((f2.pd - 40.0).abs() / 40.0 < 0.3, "pd2={}", f2.pd);
        assert!(f2.g > f1.g, "poorer-sorted plug 2 must fit a larger G");
        assert!(!f1.scatter.is_empty() && !f1.curve.is_empty());
        assert!(f1.r2 > 0.95 && f2.r2 > 0.95);
        assert!(f1.standardized && f1.system.as_deref() == Some("hg_air"));
        assert!(!f1.pd_at_bound && !f2.pd_at_bound, "well-resolved entries must not flag");
        assert!(f1.swanson_k.is_finite(), "standardized plug gets a Swanson k");
    }

    /// The review-driven standardization: the SAME rock delivered as air-brine data
    /// (Pc ÷ 367/72) must fit the SAME Hg-air-equivalent Pd, and a plug with no recorded
    /// σcosθ must fit raw, be marked unstandardized, and get NO Swanson k.
    #[test]
    fn run_thomeer_standardizes_fluid_systems() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "THOM-2", None, None, None).unwrap();
        let ids = wid.to_string();

        let (pd_hg, g, bv_inf, poro) = (20.0, 0.5, 0.15, 0.20f32);
        let mut rows = Vec::new();
        for (pc_hg, bv) in synth_points(pd_hg, g, bv_inf) {
            // Plug 1: the same physics measured air-brine — lab Pc is 72/367 of Hg-air.
            rows.push(crate::db::ScalPcRow {
                sample_no: Some(1),
                depth: Some(2000.0),
                perm: 100.0,
                poro,
                pc: (pc_hg * 72.0 / 367.0) as f32,
                sw: (1.0 - bv / poro as f64) as f32,
                system: Some("air_brine".into()),
                ift: Some(72.0),
            });
            // Plug 2: legacy rows with no recorded system/ift.
            rows.push(crate::db::ScalPcRow {
                sample_no: Some(2),
                depth: Some(2010.0),
                perm: 100.0,
                poro,
                pc: pc_hg as f32,
                sw: (1.0 - bv / poro as f64) as f32,
                system: None,
                ift: None,
            });
        }
        crate::db::insert_scal_pc(&conn, &ids, &rows).unwrap();

        let db = Mutex::new(conn);
        let res = run_thomeer_fit(&db, &ThomeerRequest { well_ids: vec![ids] });
        assert!(res.error.is_none(), "{:?}", res.error);
        let f1 = res.fits.iter().find(|f| f.sample_no == Some(1)).unwrap();
        let f2 = res.fits.iter().find(|f| f.sample_no == Some(2)).unwrap();
        assert!(f1.standardized, "air-brine plug with recorded ift standardizes");
        assert!(
            (f1.pd - pd_hg).abs() / pd_hg < 0.3,
            "air-brine Pd converts to Hg-air equivalent: pd={} vs {}",
            f1.pd,
            pd_hg
        );
        assert!(f1.swanson_k.is_finite());
        assert!(!f2.standardized, "no recorded ift → unstandardized");
        assert!(f2.swanson_k.is_nan(), "Swanson is a mercury-system relation — suppressed");
        // Both plugs describe the same Hg-air physics, so their standardized/raw Pd agree.
        assert!((f1.pd - f2.pd).abs() / f2.pd < 0.2, "pd1={} pd2={}", f1.pd, f2.pd);
    }
}
