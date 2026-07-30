//! Saturation-height modeling: core capillary-pressure data
//! (`scal_pc` table), Leverett-J function fitting, and the `sw_height` module that
//! writes a saturation-vs-height-above-FWL curve (SWH).
//!
//! Conventions: Pc in psi, IFT (sigma*cos theta) in dyn/cm, perm in mD, porosity v/v,
//! depths/heights in metres (converted to ft only inside the Pc gradient formula).
//! J = 0.21645 * (Pc / IFT) * sqrt(k / phi)   (the classic oilfield-unit Leverett J)
//! Pc_res = 0.433 * (RHO_W - RHO_HC) * h_ft   (psi; 0.433 psi/ft per unit sp. gravity)

use crate::modules::{log_in, log_out, opt, param, ModuleContext, ModuleOutputs, ModuleSpec};
use serde::Serialize;
use std::collections::HashMap;

pub(crate) const J_CONST: f64 = 0.21645;
/// Hydrostatic gradient per unit specific gravity, per FOOT of column — so any height fed
/// to it must be converted to feet first (`units::to_feet`). The old `FT_PER_M` constant
/// that used to live here is gone: it encoded the assumption that heights arrive in
/// metres, which is exactly what broke Pc on foot-declared projects. Conversion now goes
/// through `units.rs`, which knows what unit the project is actually in.
pub(crate) const PSI_PER_FT_PER_SG: f64 = 0.433;

// ---------------------------------------------------------------------------
// Leverett-J fit: Sw = A * J^B by least squares in log-log space
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct LeverettFit {
    pub a: f64,
    pub b: f64,
    pub r2: f64,
    pub n_points: usize,
}

/// One core capillary-pressure measurement (a row of `scal_pc`).
#[derive(Clone)]
pub struct ScalPoint {
    pub pc: f32,   // psi (lab system)
    pub sw: f32,   // v/v
    pub perm: f32, // mD
    pub poro: f32, // v/v
}

/// Fits Sw = A * J^B over all valid points (Sw and J strictly positive, Sw <= 1),
/// with J computed from each sample's own perm/poro at the LAB interfacial tension.
/// Log-log linear regression: ln Sw = ln A + B ln J. Returns None with < 3 usable points.
pub fn fit_leverett_j(points: &[ScalPoint], ift_lab: f64) -> Option<LeverettFit> {
    if ift_lab <= 0.0 {
        return None;
    }
    let mut xs = Vec::new(); // ln J
    let mut ys = Vec::new(); // ln Sw
    for p in points {
        let (pc, sw, k, phi) = (p.pc as f64, p.sw as f64, p.perm as f64, p.poro as f64);
        if !(pc > 0.0 && sw > 0.0 && sw <= 1.0 && k > 0.0 && phi > 0.0) {
            continue;
        }
        let j = J_CONST * pc / ift_lab * (k / phi).sqrt();
        if j > 0.0 && j.is_finite() {
            xs.push(j.ln());
            ys.push(sw.ln());
        }
    }
    let n = xs.len();
    if n < 3 {
        return None;
    }
    let nf = n as f64;
    let sx: f64 = xs.iter().sum();
    let sy: f64 = ys.iter().sum();
    let sxx: f64 = xs.iter().map(|x| x * x).sum();
    let sxy: f64 = xs.iter().zip(&ys).map(|(x, y)| x * y).sum();
    let denom = nf * sxx - sx * sx;
    if denom.abs() < 1e-12 {
        return None;
    }
    let b = (nf * sxy - sx * sy) / denom;
    let ln_a = (sy - b * sx) / nf;

    // R² in log space (where the fit was done).
    let mean_y = sy / nf;
    let ss_tot: f64 = ys.iter().map(|y| (y - mean_y).powi(2)).sum();
    let ss_res: f64 =
        xs.iter().zip(&ys).map(|(x, y)| (y - (ln_a + b * x)).powi(2)).sum();
    let r2 = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 1.0 };

    Some(LeverettFit { a: ln_a.exp(), b, r2, n_points: n })
}

// ---------------------------------------------------------------------------
// SW_HEIGHT — saturation from height above the free-water level
// ---------------------------------------------------------------------------

pub(crate) fn sw_height_spec() -> ModuleSpec {
    ModuleSpec {
        name: "sw_height".into(),
        title: "SW — Saturation-Height".into(),
        category: "Saturation".into(),
        doc: "SWH from height above the free-water level. LEVERETT: Pc = 0.433*(RHO_W-RHO_HC)*h_ft, \
              J = 0.21645*Pc/IFT_RES*sqrt(PERM/PHIE), SWH = SWH_A * J^SWH_B (fit SWH_A/SWH_B from \
              core Pc data via Import SCAL — the fit is reported there). SKELT (Skelt-Harrison): \
              SWH = 1 - SH_A*exp(-(SH_B/(h+SH_D))^SH_C), h in metres. Below the FWL (h <= 0) \
              SWH = 1. Result limited to [SWT_IRR, 1]. FWL is zone-overridable for stacked \
              reservoirs with different contacts. Height is measured from the TVD input when a \
              TVD curve is supplied (else measured depth) so deviated wells are not over-stated \
              — MD height overstates true height by ~1/cos(inc); enter FWL on the SAME reference \
              (a negative value for a sub-sea TVDSS FWL)."
            .into(),
        args: vec![
            opt("OPT_SWH", "Saturation-height model", "LEVERETT", &["LEVERETT", "SKELT"]),
            param("FWL", "Free-water level (same reference as the vertical-depth input; negative = subsea TVDSS)", "m", 2000.0, -10000.0, 20000.0),
            param("RHO_W", "Water density", "g/cc", 1.0, 0.8, 1.3),
            param("RHO_HC", "Hydrocarbon density", "g/cc", 0.8, 0.05, 1.1),
            param("IFT_RES", "Reservoir sigma*cos(theta)", "dyn/cm", 26.0, 1.0, 500.0),
            param("SWH_A", "Leverett coefficient A (from J-fit)", "", 0.5, 0.001, 100.0),
            param("SWH_B", "Leverett exponent B (from J-fit, usually negative)", "", -0.4, -5.0, 0.0),
            param("SH_A", "Skelt-Harrison A", "", 1.0, 0.0, 1.0),
            param("SH_B", "Skelt-Harrison B", "m", 30.0, 0.1, 5000.0),
            param("SH_C", "Skelt-Harrison C", "", 1.5, 0.1, 10.0),
            param("SH_D", "Skelt-Harrison D", "m", 0.0, -100.0, 1000.0),
            param("SWT_IRR", "Irreducible water saturation (lower clamp)", "v/v", 0.0, 0.0, 0.8),
            log_in("PHIE", "Limited effective porosity", "v/v", "PHIE", true),
            log_in("PERM", "Working permeability (LEVERETT only)", "mD", "PERM", false),
            log_in("TVD", "True vertical (sub-sea) depth for height; defaults to measured depth", "m", "TVD", false),
            log_out("SWH", "Water saturation from height function", "v/v"),
            log_out("HAFWL", "Height above free-water level", "m"),
        ],
    }
}

pub(crate) fn sw_height(ctx: &ModuleContext) -> ModuleOutputs {
    let depth = ctx.log("DEPTH");
    let tvd = ctx.log("TVD");
    let phie = ctx.log("PHIE");
    let perm = ctx.log("PERM");
    let skelt = ctx.o("OPT_SWH") == "SKELT";

    let mut swh_out = vec![f32::NAN; ctx.n];
    let mut hafwl_out = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let pe = phie[i] as f64;
        // Vertical depth for the height calc: prefer the TVD curve so deviated wells aren't
        // over-stated (MD height overstates true height above the contact by ~1/cos(inc));
        // fall back to measured depth when no TVD is supplied.
        let dv = {
            let t = tvd[i] as f64;
            if t.is_nan() { depth[i] as f64 } else { t }
        };
        if dv.is_nan() || pe.is_nan() {
            continue;
        }
        let fwl = ctx.p("FWL", i);
        let swt_irr = ctx.p("SWT_IRR", i);
        let h = fwl - dv; // metres above the FWL (negative below it); FWL shares dv's reference
        hafwl_out[i] = h as f32;

        if h <= 0.0 {
            swh_out[i] = 1.0; // at/below the free-water level: fully water
            continue;
        }
        // Tight/zero porosity carries no meaningful saturation-height signal.
        if pe < 0.005 {
            swh_out[i] = 1.0;
            continue;
        }

        let sw = if skelt {
            let a = ctx.p("SH_A", i);
            let b = ctx.p("SH_B", i);
            let c = ctx.p("SH_C", i);
            let dd = ctx.p("SH_D", i);
            if h + dd <= 0.0 {
                1.0
            } else {
                1.0 - a * (-(b / (h + dd)).powf(c)).exp()
            }
        } else {
            let k = perm[i] as f64;
            if k.is_nan() || k <= 0.0 {
                continue; // Leverett needs permeability
            }
            let rho_w = ctx.p("RHO_W", i);
            let rho_hc = ctx.p("RHO_HC", i);
            let ift = ctx.p("IFT_RES", i);
            // PSI_PER_FT_PER_SG is per FOOT of column, so the height must be in feet. This
            // used to be `h * FT_PER_M`, which assumed h arrived in metres — on a project
            // declared in feet that scaled an already-foot height and returned Pc 3.28x
            // too high (his Central Sumatra projects are foot-declared).
            let pc = PSI_PER_FT_PER_SG * (rho_w - rho_hc) * crate::units::to_feet(h, ctx.depth_unit);
            if pc <= 0.0 || ift <= 0.0 {
                continue;
            }
            let j = J_CONST * pc / ift * (k / pe).sqrt();
            ctx.p("SWH_A", i) * j.powf(ctx.p("SWH_B", i))
        };

        swh_out[i] = sw.clamp(swt_irr.max(0.0), 1.0) as f32;
    }

    HashMap::from([("SWH".to_string(), swh_out), ("HAFWL".to_string(), hafwl_out)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ArgKind;

    fn ctx_from_spec(n: usize, logs: &[(&str, Vec<f32>)], overrides: &[(&str, f64)], opts: &[(&str, &str)]) -> ModuleContext {
        ctx_in_unit(n, logs, overrides, opts, crate::units::DepthUnit::Metres)
    }

    /// Same as `ctx_from_spec` but with an explicit project depth unit, so a test can pin
    /// that the SAME physical well gives the SAME answer in either declaration.
    fn ctx_in_unit(
        n: usize,
        logs: &[(&str, Vec<f32>)],
        overrides: &[(&str, f64)],
        opts: &[(&str, &str)],
        depth_unit: crate::units::DepthUnit,
    ) -> ModuleContext {
        let spec = sw_height_spec();
        let mut params: HashMap<String, Vec<f64>> = spec
            .args
            .iter()
            .filter(|a| a.kind == ArgKind::Param)
            .map(|a| (a.name.clone(), vec![a.default.parse().unwrap(); n]))
            .collect();
        for (k, v) in overrides {
            params.insert(k.to_string(), vec![*v; n]);
        }
        ModuleContext {
            n,
            logs: logs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            params,
            opts: opts.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            depth_unit,
        }
    }

    /// The regression this whole units change exists for. One physical well, described
    /// twice: 100 m above the FWL in a metre-declared project, and the identical 328.084
    /// ft above the FWL in a foot-declared one. The capillary-pressure law is per FOOT, so
    /// before the fix the foot project multiplied an already-foot height by 3.28084 and
    /// returned a Pc 3.28x too high — a wrong Sw that computed, plotted and shipped.
    #[test]
    fn saturation_height_is_identical_whichever_unit_the_project_declares() {
        use crate::units::DepthUnit;
        let logs_m: Vec<(&str, Vec<f32>)> =
            vec![("DEPTH", vec![1900.0]), ("TVD", vec![1900.0]), ("PHIE", vec![0.25]), ("PERM", vec![100.0])];
        // 2000 m FWL - 1900 m TVD = 100 m of column.
        let m_ctx = ctx_in_unit(1, &logs_m, &[("FWL", 2000.0)], &[("OPT_SWH", "LEVERETT")], DepthUnit::Metres);
        let metric = sw_height(&m_ctx)["SWH"][0];

        // The same well in feet: 6561.68 ft TVD, 6889.764 ft FWL — still 328.084 ft = 100 m.
        let logs_ft: Vec<(&str, Vec<f32>)> = vec![
            ("DEPTH", vec![6561.6797]),
            ("TVD", vec![6561.6797]),
            ("PHIE", vec![0.25]),
            ("PERM", vec![100.0]),
        ];
        let ft_ctx =
            ctx_in_unit(1, &logs_ft, &[("FWL", 6889.7638)], &[("OPT_SWH", "LEVERETT")], DepthUnit::Feet);
        let imperial = sw_height(&ft_ctx)["SWH"][0];

        assert!(metric.is_finite() && metric > 0.0 && metric < 1.0, "metric Sw not usable: {metric}");
        assert!(
            (metric - imperial).abs() < 1e-3,
            "same well, same height above FWL, different declared unit: metric Sw {metric} vs foot Sw {imperial}"
        );
    }

    #[test]
    fn leverett_fit_recovers_synthetic_curve() {
        // Generate points from a known Sw = 0.4 * J^-0.5 and check the fit recovers it.
        let (a_true, b_true) = (0.4f64, -0.5f64);
        let ift = 72.0;
        let mut points = Vec::new();
        for i in 1..=20 {
            let pc = i as f64 * 2.0; // psi
            let (k, phi) = (100.0f64, 0.25f64);
            let j = J_CONST * pc / ift * (k / phi).sqrt();
            let sw = (a_true * j.powf(b_true)).min(1.0);
            points.push(ScalPoint { pc: pc as f32, sw: sw as f32, perm: k as f32, poro: phi as f32 });
        }
        let fit = fit_leverett_j(&points, ift).expect("fit");
        // Points where Sw clamped to 1.0 dilute the fit slightly; stay within a few percent.
        assert!((fit.b - b_true).abs() < 0.05, "b={}", fit.b);
        assert!((fit.a - a_true).abs() / a_true < 0.15, "a={}", fit.a);
        assert!(fit.r2 > 0.98, "r2={}", fit.r2);
    }

    #[test]
    fn fit_rejects_degenerate_input() {
        assert!(fit_leverett_j(&[], 72.0).is_none());
        let bad = vec![ScalPoint { pc: -1.0, sw: 0.5, perm: 100.0, poro: 0.2 }; 5];
        assert!(fit_leverett_j(&bad, 72.0).is_none());
    }

    #[test]
    fn sw_height_leverett_transition_zone_shape() {
        // Three samples: below FWL, just above, and far above. SWH must be 1 below the
        // FWL and decrease with height above it.
        let ctx = ctx_from_spec(
            3,
            &[
                ("DEPTH", vec![2050.0, 1995.0, 1900.0]),
                ("PHIE", vec![0.25, 0.25, 0.25]),
                ("PERM", vec![200.0, 200.0, 200.0]),
            ],
            &[("FWL", 2000.0)],
            &[("OPT_SWH", "LEVERETT")],
        );
        let out = sw_height(&ctx);
        let swh = &out["SWH"];
        assert_eq!(swh[0], 1.0, "below FWL is all water");
        assert!(swh[1] > swh[2], "saturation decreases with height: {} vs {}", swh[1], swh[2]);
        assert!(swh[2] >= 0.0 && swh[2] < 1.0);
        assert!((out["HAFWL"][2] - 100.0).abs() < 1e-3);
    }

    #[test]
    fn sw_height_skelt_needs_no_perm() {
        let ctx = ctx_from_spec(
            1,
            &[("DEPTH", vec![1950.0]), ("PHIE", vec![0.2])], // no PERM log at all
            &[("FWL", 2000.0)],
            &[("OPT_SWH", "SKELT")],
        );
        let out = sw_height(&ctx);
        let s = out["SWH"][0];
        assert!(s.is_finite() && s > 0.0 && s <= 1.0, "SWH={s}");
    }

    #[test]
    fn sw_height_uses_tvd_and_allows_tvdss_fwl() {
        // Deviated well: measured depth (3000) runs well ahead of true vertical depth. With a
        // sub-sea (negative TVDSS) FWL and a TVD curve, height must come from TVD, not MD.
        // Using MD, h = -1400 - 3000 = -4400 ⇒ below FWL ⇒ SWH = 1 (the optimistic-pay bug);
        // using TVD, h = -1400 - (-1450) = +50 m ⇒ transition zone ⇒ SWH < 1.
        let ctx = ctx_from_spec(
            1,
            &[
                ("DEPTH", vec![3000.0]),
                ("TVD", vec![-1450.0]),
                ("PHIE", vec![0.25]),
                ("PERM", vec![200.0]),
            ],
            &[("FWL", -1400.0)],
            &[("OPT_SWH", "LEVERETT")],
        );
        let out = sw_height(&ctx);
        assert!((out["HAFWL"][0] - 50.0).abs() < 1e-3, "HAFWL={}", out["HAFWL"][0]);
        let s = out["SWH"][0];
        assert!(s.is_finite() && s > 0.0 && s < 1.0, "SWH in transition zone: {s}");
    }
}
