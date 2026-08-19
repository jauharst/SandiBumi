//! Results-QC — cross-method water-saturation spread ("does the Sw model choice change the answer?").
//!
//! The five app Sw models are pure functions in [`crate::sandimin`]; the frontend cannot call them, so
//! the per-depth Sw *envelope* is the one genuinely-new backend metric the Results-QC dashboard needs.
//! Everything else the dashboard shows — Buckles (Sw·φ), unity, recon rollup, cutoff sensitivity, Monte
//! Carlo P10/P50/P90 — reuses commands that already exist, computed frontend-side.
//!
//! Per depth we evaluate every model whose inputs are present and report the envelope (min/max/spread)
//! plus a divergence summary. Archie / Simandoux / Indonesia / Juhász run from the always-available logs
//! (Juhász's `phit_sh` is a documented [`FluidProps`] default, not an invented constant). The
//! excess-conductivity pair — Waxman-Smits, Dual-Water — join **only** when a Qv / Swb curve is supplied;
//! we never fabricate CEC or a bound-water saturation to force them into the envelope.

use duckdb::Connection;
use serde::{Deserialize, Serialize};

use crate::sandimin::{
    fluid_calc, sw_archie, sw_dual_nonlinear, sw_indonesia, sw_juhasz,
    sw_simandoux_bardon_pied, sw_simandoux_modified_slb, sw_waxman_smits, waxman_b,
    FluidProps,
};

/// Default Sw-unit gap above which a depth counts as "the model choice matters here".
/// DEC-077 (2026-08-19): a QC display convention, not rock — ruled the owner's starting value
/// with practitioner attribution per DEC-059 (0.10 Sw units is one decile of the answer's own
/// scale); every request may override it, and the request's value always wins.
const DEFAULT_DIVERGENCE: f64 = 0.10;

#[derive(Debug, Clone, Deserialize)]
pub struct SwSpreadRequest {
    /// Read the curves this run consumes from THIS log set's stored values (latest version per
    /// well) rather than from whatever the current values are. Curves the set never wrote fall
    /// back to normal resolution; an empty name means "current values", which is what every
    /// caller did before this existed (Jauhar, 2026-08-05).
    #[serde(default)]
    pub input_set: Option<String>,
    pub well_id: String,
    #[serde(default)]
    pub depth_min: Option<f32>,
    #[serde(default)]
    pub depth_max: Option<f32>,
    /// Curve-name overrides. When `None` a candidate list is tried (first present wins).
    #[serde(default)]
    pub rt_curve: Option<String>,
    #[serde(default)]
    pub phie_curve: Option<String>,
    #[serde(default)]
    pub phit_curve: Option<String>,
    #[serde(default)]
    pub vsh_curve: Option<String>,
    /// Optional Qv (meq/mL) curve — its presence is what enables Waxman-Smits.
    #[serde(default)]
    pub qv_curve: Option<String>,
    /// Optional bound-water saturation (v_bw/φt) curve — its presence enables Dual-Water.
    #[serde(default)]
    pub swb_curve: Option<String>,
    /// Rw / temperature / m / n / Rsh / a / φ_sh — the same block the SandiMin dialog carries.
    pub fluid: FluidProps,
    /// Sw-unit gap above which a depth is flagged divergent. Default [`DEFAULT_DIVERGENCE`].
    #[serde(default)]
    pub divergence_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SwMethodSeries {
    pub name: String,
    /// Curve chosen for this model's inputs (for the panel to echo), e.g. "PHIE".
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SwSpreadResult {
    pub depth: Vec<f32>,
    /// One entry per model that had its inputs available (NaN where a single sample was non-physical).
    pub methods: Vec<SwMethodSeries>,
    pub sw_min: Vec<f32>,
    pub sw_max: Vec<f32>,
    pub spread: Vec<f32>,
    pub mean_spread: f32,
    pub max_spread: f32,
    pub max_spread_depth: f32,
    /// Fraction of comparable depths (≥2 models) whose spread exceeds the threshold.
    pub frac_divergent: f32,
    /// Comparable depths (≥2 finite models) used in the summary stats.
    pub n_samples: usize,
    /// Human-readable trail: which models ran, which were skipped and why, fallbacks used.
    pub notes: Vec<String>,
}

/// Scalar inputs shared by every per-depth model, derived once from [`FluidProps`].
struct SpreadParams {
    /// Formation-water resistivity at formation temperature (ohm·m) — Archie/Simandoux/Indonesia.
    rw: f64,
    /// Formation-water conductivity at formation temperature (mho/m) — Juhász/WS/Dual-Water.
    cw: f64,
    /// Bound-water conductivity, virgin zone (mho/m) — Dual-Water `cwb`.
    cwb: f64,
    /// Waxman-Smits B (mho·mL/(m·meq)).
    b: f64,
    m: f64,
    n: f64,
    /// Tortuosity a for Archie/Simandoux/Indonesia (dual-water uses a = 1 internally).
    a: f64,
    rsh: f64,
    phit_sh: f64,
    indonesia_k: f64,
    simandoux_c: f64,
    threshold: f64,
}

impl SpreadParams {
    fn from_fluid(f: &FluidProps, threshold: f64) -> Self {
        let fc = fluid_calc(f);
        let cw = fc.cw;
        let rw = 1.0 / cw.max(1e-9); // Rw at formation temperature (cw is already at ftemp)
        let b = if f.ws_b > 0.0 {
            f.ws_b
        } else {
            let t_c = (f.ftemp_f - 32.0) * 5.0 / 9.0;
            waxman_b(t_c, rw)
        };
        SpreadParams {
            rw,
            cw,
            cwb: fc.cbw_u,
            b,
            m: f.m,
            n: f.n,
            a: f.archie_a,
            rsh: f.rsh,
            phit_sh: f.phit_sh,
            indonesia_k: f.indonesia_k,
            simandoux_c: f.simandoux_c,
            threshold,
        }
    }
}

#[inline]
fn at(o: Option<&[f32]>, i: usize) -> f64 {
    o.and_then(|s| s.get(i)).map(|&x| x as f64).unwrap_or(f64::NAN)
}

/// Total porosity at sample `i`: the PHIT curve when present, else PHIE as a documented fall-back.
#[inline]
fn phit_at(phit: Option<&[f32]>, phie: Option<&[f32]>, i: usize) -> f64 {
    let t = at(phit, i);
    if t.is_finite() {
        t
    } else {
        at(phie, i)
    }
}

/// Pure spread computation over aligned curve slices. DB-free so it is unit-testable directly.
#[allow(clippy::too_many_arguments)]
fn compute_spread(
    depth: &[f32],
    rt: Option<&[f32]>,
    phie: Option<&[f32]>,
    phit: Option<&[f32]>,
    vsh: Option<&[f32]>,
    qv: Option<&[f32]>,
    swb: Option<&[f32]>,
    p: &SpreadParams,
    mut notes: Vec<String>,
) -> SwSpreadResult {
    let n_d = depth.len();
    let has_rt = rt.is_some();
    let has_phi = phit.is_some() || phie.is_some();
    let has_vsh = vsh.is_some();

    if phit.is_none() && phie.is_some() {
        notes.push("No PHIT curve — using PHIE for the total-porosity models (Archie/Juhász/WS/Dual-Water); total > effective, so their Sw reads slightly high.".into());
    }

    let mut methods: Vec<SwMethodSeries> = Vec::new();
    let mut dropped: Vec<String> = Vec::new();
    // Keep a model only if it produced at least one finite Sw. A column that exists but is entirely
    // null must not inflate the "active model" count, nor silently suppress the insufficient-data note.
    let mut consider = |name: &str, active: bool, f: &dyn Fn(usize) -> f64| {
        if !active {
            return;
        }
        let values: Vec<f32> = (0..n_d).map(|i| f(i) as f32).collect();
        if values.iter().any(|v| v.is_finite()) {
            methods.push(SwMethodSeries { name: name.into(), values });
        } else {
            dropped.push(name.to_string());
        }
    };

    // Archie — clean-sand baseline (ignores clay); the reference every clay-aware model is compared to.
    consider("archie_total", has_rt && has_phi, &|i| {
        sw_archie(at(rt, i), phit_at(phit, phie, i), p.rw, p.m, p.n, p.a)
    });
    // Both typed Simandoux equations and Indonesia are effective-porosity shaly-sand forms.
    consider("simandoux_bardon_pied", has_rt && phie.is_some() && has_vsh, &|i| {
        sw_simandoux_bardon_pied(at(rt, i), at(phie, i), at(vsh, i), p.rw, p.rsh, p.m, p.n, p.a)
    });
    consider("simandoux_modified_slb", has_rt && phie.is_some() && has_vsh, &|i| {
        sw_simandoux_modified_slb(
            at(rt, i),
            at(phie, i),
            at(vsh, i),
            p.rw,
            p.rsh,
            p.m,
            p.n,
            p.a,
            p.simandoux_c,
        )
    });
    consider("indonesia", has_rt && phie.is_some() && has_vsh, &|i| {
        sw_indonesia(
            at(rt, i),
            at(phie, i),
            at(vsh, i),
            p.rw,
            p.rsh,
            p.m,
            p.n,
            p.a,
            p.indonesia_k,
        )
    });
    // Juhász — normalized Waxman-Smits from the shale point (total-φ, needs VSH; φ_sh is a Fluid default).
    consider("juhasz", has_rt && has_phi && has_vsh, &|i| {
        sw_juhasz(at(rt, i), phit_at(phit, phie, i), at(vsh, i), p.cw, p.rsh, p.phit_sh, p.m, p.n)
    });
    // Waxman-Smits — only with a Qv curve, and only where the Qv sample is finite and non-negative. A
    // null Qv (NaN or a −999.25 sentinel) must yield NaN here: `sw_waxman_smits` folds Qv through
    // `(B·Qv).max(0)`, so a null would otherwise collapse to the clean-sand (Archie) branch and both
    // mislabel WS as evaluated and understate the spread in exactly the shaly zone that matters.
    consider("waxman_smits", has_rt && has_phi && qv.is_some(), &|i| {
        let q = at(qv, i);
        if !(q.is_finite() && q >= 0.0) {
            return f64::NAN;
        }
        sw_waxman_smits(at(rt, i), phit_at(phit, phie, i), q, p.cw, p.b, p.m, p.n)
    });
    // Dual-Water — only when a bound-water-saturation curve is supplied (no Swb fabrication).
    consider("dual_water_nonlinear", has_rt && has_phi && swb.is_some(), &|i| {
        sw_dual_nonlinear(at(rt, i), phit_at(phit, phie, i), at(swb, i), p.cw, p.cwb, p.m, p.n, 1.0)
    });

    if phie.is_none() {
        notes.push("No PHIE (effective-φ) curve — Simandoux/Indonesia skipped.".into());
    }
    if !has_vsh {
        notes.push("No VSH curve — Simandoux/Indonesia/Juhász skipped (they need clay volume).".into());
    }
    if qv.is_none() {
        notes.push("No Qv curve — Waxman-Smits skipped (Qv not fabricated from CEC).".into());
    }
    if swb.is_none() {
        notes.push("No bound-water-saturation curve — Dual-Water skipped (Swb not fabricated).".into());
    }
    if !dropped.is_empty() {
        notes.push(format!(
            "Input curve present but all-null (no finite Sw), skipped: {}.",
            dropped.join(", ")
        ));
    }
    notes.push(format!(
        "Envelope over {} model(s): {}.",
        methods.len(),
        methods.iter().map(|m| m.name.as_str()).collect::<Vec<_>>().join(", ")
    ));

    let threshold = p.threshold;
    let mut sw_min = vec![f32::NAN; n_d];
    let mut sw_max = vec![f32::NAN; n_d];
    let mut spread = vec![f32::NAN; n_d];
    for i in 0..n_d {
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        let mut cnt = 0u32;
        for mth in &methods {
            let v = mth.values[i] as f64;
            if v.is_finite() {
                lo = lo.min(v);
                hi = hi.max(v);
                cnt += 1;
            }
        }
        if cnt >= 2 {
            sw_min[i] = lo as f32;
            sw_max[i] = hi as f32;
            spread[i] = (hi - lo) as f32; // NaN spread ⇒ fewer than two comparable models here
        } else if cnt == 1 {
            sw_min[i] = lo as f32;
            sw_max[i] = lo as f32;
        }
    }

    let mut sum = 0.0f64;
    let mut cnt = 0usize;
    let mut mx = f64::NEG_INFINITY;
    let mut mx_d = f32::NAN;
    let mut div = 0usize;
    for i in 0..n_d {
        let s = spread[i] as f64;
        if s.is_finite() {
            sum += s;
            cnt += 1;
            if s > mx {
                mx = s;
                mx_d = depth[i];
            }
            if s > threshold {
                div += 1;
            }
        }
    }
    let mean_spread = if cnt > 0 { (sum / cnt as f64) as f32 } else { f32::NAN };
    let max_spread = if cnt > 0 { mx as f32 } else { f32::NAN };
    let frac_divergent = if cnt > 0 { div as f32 / cnt as f32 } else { f32::NAN };
    if cnt == 0 {
        notes.push("No comparable depths — need ≥2 models with finite Sw at the same depth. Check that the PHIE/VSH (and any Qv/Swb) curves actually contain data in this window.".into());
    }

    SwSpreadResult {
        depth: depth.to_vec(),
        methods,
        sw_min,
        sw_max,
        spread,
        mean_spread,
        max_spread,
        max_spread_depth: mx_d,
        frac_divergent,
        n_samples: cnt,
        notes,
    }
}

/// Resolve a logical input to the first present candidate (explicit override wins), returning the chosen
/// name and its values filtered to `keep`. Records the choice into `notes`.
fn resolve(
    map: &std::collections::HashMap<String, Vec<f32>>,
    explicit: &Option<String>,
    candidates: &[&str],
    keep: &[usize],
    label: &str,
    notes: &mut Vec<String>,
) -> Option<Vec<f32>> {
    let names: Vec<String> = match explicit {
        Some(s) if !s.trim().is_empty() => vec![s.trim().to_string()],
        _ => candidates.iter().map(|s| s.to_string()).collect(),
    };
    for name in &names {
        // case-insensitive match against whatever fetch_curve_frame returned
        if let Some((k, vals)) = map.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)) {
            let filtered: Vec<f32> = keep.iter().map(|&i| vals.get(i).copied().unwrap_or(f32::NAN)).collect();
            notes.push(format!("{label}: {k}"));
            return Some(filtered);
        }
    }
    None
}

/// DB entry point: fetch the well's curves, resolve inputs, and compute the Sw-method spread.
pub fn sw_method_spread(conn: &Connection, req: &SwSpreadRequest) -> Result<SwSpreadResult, String> {
    // Everything we might need — fetch_curve_frame silently drops names it can't find.
    let want: Vec<String> = [
        "RT", "RES_DEEP", "ILD", "LLD", "RESD", "RD", "AT90", "RLA5", // resistivity
        "PHIE", "PHI", "PHIT", "PIGN", // porosity
        "VSH", "VCL", "VWCL", "VCLAY", "VSHALE", // clay volume
        "QV", "SWB", // optional conductivity-model drivers (BQV excluded: it is B·Qv, not Qv)
    ]
    .iter()
    .map(|s| s.to_string())
    // include any explicit overrides too
    .chain(
        [
            &req.rt_curve,
            &req.phie_curve,
            &req.phit_curve,
            &req.vsh_curve,
            &req.qv_curve,
            &req.swb_curve,
        ]
        .into_iter()
        .flatten()
        .cloned(),
    )
    .collect();

    let (depth_all, map) =
        crate::equations::fetch_curve_frame_from_set(conn, &req.well_id, &want, req.input_set.as_deref(), None)
            .map_err(|e| e.to_string())?;
    if depth_all.is_empty() {
        return Err("No curve data for this well".into());
    }

    let keep: Vec<usize> = (0..depth_all.len())
        .filter(|&i| {
            let d = depth_all[i];
            d.is_finite()
                && req.depth_min.map_or(true, |lo| d >= lo)
                && req.depth_max.map_or(true, |hi| d <= hi)
        })
        .collect();
    if keep.is_empty() {
        return Err("No samples in the requested depth window".into());
    }
    let depth: Vec<f32> = keep.iter().map(|&i| depth_all[i]).collect();

    let mut notes: Vec<String> = Vec::new();
    let rt = resolve(&map, &req.rt_curve, &["RT", "RES_DEEP", "ILD", "LLD", "RESD", "RD", "AT90", "RLA5"], &keep, "Rt", &mut notes);
    // PHIE must be effective φ only — "PHI" is ambiguous (often total) so it seeds PHIT, never PHIE.
    let phie = resolve(&map, &req.phie_curve, &["PHIE"], &keep, "PHIE", &mut notes);
    let phit = resolve(&map, &req.phit_curve, &["PHIT", "PHI"], &keep, "PHIT", &mut notes);
    let vsh = resolve(&map, &req.vsh_curve, &["VSH", "VCL", "VWCL", "VCLAY", "VSHALE"], &keep, "Vsh", &mut notes);
    // Only "QV" (meq/mL) auto-resolves — "BQV" is B·Qv and would double-count B, so it needs an explicit override.
    let qv = resolve(&map, &req.qv_curve, &["QV"], &keep, "Qv", &mut notes);
    let swb = resolve(&map, &req.swb_curve, &["SWB"], &keep, "Swb", &mut notes);

    if rt.is_none() {
        return Err("No deep-resistivity curve found (tried RT/RES_DEEP/ILD/LLD…). Set one explicitly.".into());
    }
    if phie.is_none() && phit.is_none() {
        return Err("No porosity curve found (tried PHIE/PHIT). Set one explicitly.".into());
    }

    let threshold = req.divergence_threshold.filter(|t| *t > 0.0).unwrap_or(DEFAULT_DIVERGENCE);
    let params = SpreadParams::from_fluid(&req.fluid, threshold);

    Ok(compute_spread(
        &depth,
        rt.as_deref(),
        phie.as_deref(),
        phit.as_deref(),
        vsh.as_deref(),
        qv.as_deref(),
        swb.as_deref(),
        &params,
        notes,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fluid() -> FluidProps {
        FluidProps {
            rw: 0.1,
            rw_temp_f: 75.0,
            rmf: 0.1,
            rmf_temp_f: 75.0,
            ftemp_f: 167.0, // 75 °C
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
            rsh: 2.0,
            archie_a: 1.0,
            indonesia_k: 1.0,
            simandoux_c: 1.0,
            phit_sh: 0.10,
            ws_b: 0.0,
        }
    }

    fn params() -> SpreadParams {
        SpreadParams::from_fluid(&fluid(), DEFAULT_DIVERGENCE)
    }

    #[test]
    fn shaly_sand_makes_archie_diverge_from_clay_models() {
        // A clay-bearing interval: Archie (no clay term) over-reads Sw vs Simandoux/Indonesia.
        let depth = vec![1000.0f32, 1001.0, 1002.0];
        let rt = vec![5.0f32, 5.0, 5.0];
        let phie = vec![0.20f32, 0.20, 0.20];
        let vsh = vec![0.40f32, 0.40, 0.40];
        let p = params();
        let r = compute_spread(&depth, Some(&rt), Some(&phie), None, Some(&vsh), None, None, &p, vec![]);
        // Archie, both typed Simandoux equations, Indonesia, and Juhász are active.
        assert_eq!(r.methods.len(), 5, "notes: {:?}", r.notes);
        // Rw is temperature-corrected by fluid_calc (0.1 @ 75 °F → ~0.047 @ 167 °F), so these are the
        // formation-temperature Sw values, not the surface-Rw hand figures.
        let archie = &r.methods.iter().find(|m| m.name == "archie_total").unwrap().values[0];
        let sima = &r.methods.iter().find(|m| m.name == "simandoux_modified_slb").unwrap().values[0];
        let juh = &r.methods.iter().find(|m| m.name == "juhasz").unwrap().values[0];
        assert!(*archie > 0.45 && *archie < 0.52, "Archie {archie}");
        assert!(*sima > 0.28 && *sima < 0.34, "Simandoux {sima}");
        assert!(*juh > 0.33 && *juh < 0.41, "Juhasz {juh}"); // numeric guard against a Cw/Rw swap
        assert!(*archie - *sima > 0.15, "Archie should over-read Sw vs Simandoux: {archie} vs {sima}");
        assert!(r.max_spread > 0.15, "spread should be large in shaly sand: {}", r.max_spread);
        assert!(r.frac_divergent > 0.99, "every depth divergent: {}", r.frac_divergent);
        assert_eq!(r.n_samples, 3);
    }

    #[test]
    fn clean_sand_all_models_agree() {
        // Vsh = 0 collapses every shaly-sand model to Archie ⇒ spread ~0.
        let depth = vec![2000.0f32, 2001.0];
        let rt = vec![20.0f32, 20.0];
        let phie = vec![0.25f32, 0.25];
        let vsh = vec![0.0f32, 0.0];
        let p = params();
        let r = compute_spread(&depth, Some(&rt), Some(&phie), None, Some(&vsh), None, None, &p, vec![]);
        assert_eq!(r.methods.len(), 5);
        assert!(r.max_spread < 1e-3, "clean sand should not diverge: {}", r.max_spread);
        assert_eq!(r.frac_divergent, 0.0);
    }

    #[test]
    fn waxman_and_dual_join_when_curves_present() {
        let depth = vec![1500.0f32, 1501.0];
        let rt = vec![8.0f32, 8.0];
        let phie = vec![0.22f32, 0.22];
        let phit = vec![0.24f32, 0.24];
        let vsh = vec![0.30f32, 0.30];
        let qv = vec![0.5f32, 0.5];
        let swb = vec![0.15f32, 0.15];
        let p = params();
        let r = compute_spread(
            &depth,
            Some(&rt),
            Some(&phie),
            Some(&phit),
            Some(&vsh),
            Some(&qv),
            Some(&swb),
            &p,
            vec![],
        );
        // All seven typed equations now active.
        assert_eq!(r.methods.len(), 7, "notes: {:?}", r.notes);
        assert!(r.methods.iter().any(|m| m.name == "waxman_smits"));
        assert!(r.methods.iter().any(|m| m.name == "dual_water_nonlinear"));
        // every model finite here
        for m in &r.methods {
            assert!(m.values[0].is_finite(), "{} NaN", m.name);
        }
    }

    #[test]
    fn ws_and_dual_reduce_to_archie_at_zero_excess() {
        // Qv = 0 ⇒ Waxman-Smits is clean-sand Archie; Swb = 0 ⇒ Dual-Water is clean-sand Archie.
        // (Both use a = 1 internally, and archie_a = 1 here, so they must match Archie exactly.)
        let depth = vec![1800.0f32];
        let rt = vec![12.0f32];
        let phit = vec![0.24f32];
        let qv = vec![0.0f32];
        let swb = vec![0.0f32];
        let p = params();
        let r = compute_spread(&depth, Some(&rt), Some(&phit), Some(&phit), None, Some(&qv), Some(&swb), &p, vec![]);
        let archie = r.methods.iter().find(|m| m.name == "archie_total").unwrap().values[0];
        let ws = r.methods.iter().find(|m| m.name == "waxman_smits").unwrap().values[0];
        let dw = r.methods.iter().find(|m| m.name == "dual_water_nonlinear").unwrap().values[0];
        assert!((ws - archie).abs() < 1e-5, "WS at Qv=0 must equal Archie: {ws} vs {archie}");
        assert!((dw - archie).abs() < 1e-5, "DW at Swb=0 must equal Archie: {dw} vs {archie}");
    }

    #[test]
    fn waxman_is_nan_not_archie_at_null_qv() {
        // A null Qv sample must give NaN, not the clean-sand Archie value (the (B·Qv).max(0) trap).
        let depth = vec![1600.0f32, 1601.0];
        let rt = vec![8.0f32, 8.0];
        let phit = vec![0.24f32, 0.24];
        let qv = vec![f32::NAN, 0.5]; // first sample null
        let p = params();
        let r = compute_spread(&depth, Some(&rt), Some(&phit), Some(&phit), None, Some(&qv), None, &p, vec![]);
        let ws = &r.methods.iter().find(|m| m.name == "waxman_smits").unwrap().values;
        assert!(!ws[0].is_finite(), "WS at null Qv must be NaN, was {}", ws[0]);
        assert!(ws[1].is_finite(), "WS at valid Qv must be finite");
    }

    #[test]
    fn all_null_input_column_is_dropped_and_reported() {
        // VSH column exists but is entirely null: the clay-aware models must be dropped (not counted as
        // "active"), leaving only Archie, and the insufficient-data note must fire.
        let depth = vec![1000.0f32, 1001.0];
        let rt = vec![5.0f32, 5.0];
        let phie = vec![0.20f32, 0.20];
        let vsh = vec![f32::NAN, f32::NAN];
        let p = params();
        let r = compute_spread(&depth, Some(&rt), Some(&phie), None, Some(&vsh), None, None, &p, vec![]);
        assert_eq!(r.methods.len(), 1, "only Archie survives; got {:?}", r.methods.iter().map(|m| &m.name).collect::<Vec<_>>());
        assert_eq!(r.methods[0].name, "archie_total");
        assert_eq!(r.n_samples, 0);
        assert!(r.notes.iter().any(|s| s.contains("all-null")), "should report the dropped column: {:?}", r.notes);
        assert!(r.notes.iter().any(|s| s.contains("No comparable depths")), "should warn insufficient: {:?}", r.notes);
    }

    #[test]
    fn nan_rt_rows_are_excluded_from_stats() {
        let depth = vec![900.0f32, 901.0, 902.0];
        let rt = vec![f32::NAN, 5.0, 5.0]; // first row unusable
        let phie = vec![0.20f32, 0.20, 0.20];
        let vsh = vec![0.35f32, 0.35, 0.35];
        let p = params();
        let r = compute_spread(&depth, Some(&rt), Some(&phie), None, Some(&vsh), None, None, &p, vec![]);
        assert!(!r.spread[0].is_finite(), "row with NaN Rt has no spread");
        assert_eq!(r.n_samples, 2, "only the two finite rows count");
    }

    #[test]
    fn single_model_yields_no_spread() {
        // Only Rt + PHIE, no VSH/Qv/Swb ⇒ only Archie ⇒ nothing to compare.
        let depth = vec![1200.0f32, 1201.0];
        let rt = vec![10.0f32, 10.0];
        let phie = vec![0.20f32, 0.20];
        let p = params();
        let r = compute_spread(&depth, Some(&rt), Some(&phie), None, None, None, None, &p, vec![]);
        assert_eq!(r.methods.len(), 1);
        assert!(!r.mean_spread.is_finite(), "no spread with one model");
        assert_eq!(r.n_samples, 0);
        assert!(r.notes.iter().any(|s| s.contains("No comparable depths")));
    }
}
