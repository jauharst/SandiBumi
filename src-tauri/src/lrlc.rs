//! LRLC saturation modules from Jauhar's Pertamina Upstream Innovation research
//! ("Study of LRLC caused by High Clay Volume and Microporosity in Pertamina Fields",
//! PHE UI + LAPI ITB): low-resistivity / low-contrast pay unlocked by treating BOTH
//! conductivity paths that classic Archie ignores — the clay-chemistry path (Qv) and
//! the pore-geometry path carried by capillary-bound water (microporosity).
//!
//! Two methods:
//! - `sw_rtc`  — RtC (Resistivity correction by Clay & Capillary Water): subtract a
//!   regressed excess conductivity Cex = (A_CAP·CAPBW + B_QV·Qv + C0)·PHIT·RSF from
//!   the measured conductivity, then apply Archie to the corrected term.
//! - `sw_imts` — IMTS (Integrated Mineral-Textural Scaling): Waxman-Smits-family
//!   iteration with Qv_eff = Qv_bulk/(1−Swirr) built from XRD clay volumes scaled by
//!   a lab-CEC calibration factor S, and shaly-sand exponents m*, n*.

use crate::modules::{log_in, log_out, param, ModuleContext, ModuleOutputs, ModuleSpec};
use std::collections::HashMap;

fn limit(v: f64, lo: f64, hi: f64) -> f64 {
    if v.is_nan() { v } else { v.clamp(lo, hi) }
}

/// Per-sample preference: use `primary`, else the `alt` curve where `primary` is missing.
/// Lets a module declared against SSC output names (PHIT_SSC/CWSH/CBW) also run on a well
/// processed through the SSPW workflow (PHIT_SSPW/CAPBW_SSPW/CBW_SSPW) without a silent
/// all-NaN run — a missing `alt` stays NaN, so an SSC-only well is byte-for-byte unchanged.
fn prefer(primary: &[f32], alt: &[f32]) -> Vec<f32> {
    primary
        .iter()
        .zip(alt.iter())
        .map(|(&p, &a)| if p.is_nan() { a } else { p })
        .collect()
}

/// Juhasz (1981) counterion mobility B as a function of temperature (degC) and Rw.
/// Standard Waxman-Smits-family temperature form.
fn juhasz_b(temp_c: f64, rw: f64) -> f64 {
    (-1.28 + 0.225 * temp_c - 0.0004059 * temp_c * temp_c)
        / (1.0 + rw.powf(1.23) * (0.045 * temp_c - 0.27))
}

// ---------------------------------------------------------------------------
// SW_RTC — Resistivity correction by Clay & Capillary Water
// ---------------------------------------------------------------------------

pub fn sw_rtc_spec() -> ModuleSpec {
    ModuleSpec {
        name: "sw_rtc".into(),
        title: "SW — RtC (Clay + Capillary Correction)".into(),
        category: "Saturation".into(),
        doc: "LRLC RtC method: excess conductivity from clay chemistry and capillary \
              (micropore) water is regressed as Cex = (A_CAP·CAPBW + B_QV·Qv + C0)·PHIT·RSF \
              and removed from the measured conductivity before Archie: \
              Sw = [Rw·(1/Rt − Cex)/PHIT^M]^(1/N). Qv comes from the QV input log when \
              present, else from CEC·RHOG·(1−PHIT)/(100·PHIT). Default coefficients are the \
              study's calibration (0.45, 0.0057, −0.0071, RSF 2.25) — recalibrate per field \
              from water-zone excess conductivity. CAPBW pairs naturally with SSC's CWSH or \
              SSPW's CAPBW_SSPW. The correction is capped at 98% of the measured \
              conductivity so Rt_corr stays finite."
            .into(),
        args: vec![
            param("RW", "Formation water resistivity at FT", "ohm.m", 0.3, 0.001, 100.0),
            param("M", "Cementation exponent", "", 2.0, 1.0, 4.0),
            param("N", "Saturation exponent", "", 2.0, 1.0, 4.0),
            param("A_CAP", "Capillary water coefficient", "", 0.45, -10.0, 10.0),
            param("B_QV", "Qv coefficient", "", 0.0057, -1.0, 1.0),
            param("C0", "Regression intercept", "", -0.0071, -1.0, 1.0),
            param("RSF", "Resistivity scaling factor", "", 2.25, 0.0, 20.0),
            param("CEC", "CEC when no QV log (meq/100g)", "meq/100g", 0.0, 0.0, 100.0),
            param("RHOG", "Grain density for Qv", "g/cc", 2.65, 2.0, 3.2),
            log_in("RT", "Deep resistivity", "ohm.m", "RES_DEEP", true),
            log_in("PHIT", "Total porosity", "v/v", "PHIT_SSC", true),
            log_in("CAPBW", "Capillary-bound water volume", "v/v", "CWSH", false),
            log_in("QV", "Qv log (meq/cm3), optional", "meq/cm3", "QV", false),
            log_in("CBW", "Clay-bound water (for SWE), optional", "v/v", "CBW", false),
            log_in("PHIT_SSPW", "Total porosity — SSPW fallback (used where PHIT is absent)", "v/v", "PHIT_SSPW", false),
            log_in("CAPBW_SSPW", "Capillary water — SSPW fallback", "v/v", "CAPBW_SSPW", false),
            log_in("CBW_SSPW", "Clay-bound water — SSPW fallback", "v/v", "CBW_SSPW", false),
            log_out("SWT_RTC", "Total water saturation, RtC", "v/v"),
            log_out("SWE_RTC", "Effective water saturation, RtC", "v/v"),
            log_out("RT_CORR", "Clay/capillary-corrected resistivity", "ohm.m"),
            log_out("CEX_RTC", "Excess conductivity removed", "mho/m"),
        ],
    }
}

pub fn sw_rtc(ctx: &ModuleContext) -> ModuleOutputs {
    let rt = ctx.log("RT");
    // Prefer the SSC-named curve; fall back per-sample to the SSPW equivalent so the module
    // also runs on a well processed through the SSPW porosity workflow (its outputs carry the
    // _SSPW suffix). A missing fallback stays all-NaN, so an SSC-only well is unchanged.
    let phit = prefer(&ctx.log("PHIT"), &ctx.log("PHIT_SSPW"));
    let capbw = prefer(&ctx.log("CAPBW"), &ctx.log("CAPBW_SSPW"));
    let qv_log = ctx.log("QV");
    let cbw = prefer(&ctx.log("CBW"), &ctx.log("CBW_SSPW"));

    let mut swt_o = vec![f32::NAN; ctx.n];
    let mut swe_o = vec![f32::NAN; ctx.n];
    let mut rtc_o = vec![f32::NAN; ctx.n];
    let mut cex_o = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let (rt_i, pt) = (rt[i] as f64, phit[i] as f64);
        let rw = ctx.p("RW", i);
        let m = ctx.p("M", i);
        let n_exp = ctx.p("N", i);
        if rt_i.is_nan() || pt.is_nan() || rw.is_nan() || rt_i <= 0.0 || pt <= 0.0 {
            continue;
        }
        let cap = if (capbw[i] as f64).is_nan() { 0.0 } else { limit(capbw[i] as f64, 0.0, 1.0) };
        let qv = if !(qv_log[i] as f64).is_nan() {
            (qv_log[i] as f64).max(0.0)
        } else {
            let cec = ctx.p("CEC", i);
            let rhog = ctx.p("RHOG", i);
            if cec.is_nan() || cec <= 0.0 { 0.0 } else { cec * rhog * (1.0 - pt) / (100.0 * pt) }
        };

        let ct = 1.0 / rt_i;
        let cex = (ctx.p("A_CAP", i) * cap + ctx.p("B_QV", i) * qv + ctx.p("C0", i))
            * pt
            * ctx.p("RSF", i);
        // Never remove more than 98% of the measured conductivity (and never add).
        let cex_applied = cex.clamp(0.0, 0.98 * ct);
        let ct_corr = ct - cex_applied;
        let rt_corr = 1.0 / ct_corr;

        let swt = limit((rw * ct_corr / pt.powf(m)).powf(1.0 / n_exp), 0.0, 1.0);
        swt_o[i] = swt as f32;
        rtc_o[i] = rt_corr as f32;
        cex_o[i] = cex_applied as f32;

        let cb = cbw[i] as f64;
        if !cb.is_nan() && pt > cb {
            let swb = limit(cb / pt, 0.0, 0.99);
            swe_o[i] = limit((swt - swb) / (1.0 - swb), 0.0, 1.0) as f32;
        } else {
            swe_o[i] = swt as f32;
        }
    }

    HashMap::from([
        ("SWT_RTC".to_string(), swt_o),
        ("SWE_RTC".to_string(), swe_o),
        ("RT_CORR".to_string(), rtc_o),
        ("CEX_RTC".to_string(), cex_o),
    ])
}

// ---------------------------------------------------------------------------
// SW_IMTS — Integrated Mineral-Textural Scaling (iterative)
// ---------------------------------------------------------------------------

pub fn sw_imts_spec() -> ModuleSpec {
    ModuleSpec {
        name: "sw_imts".into(),
        title: "SW — IMTS (Mineral-Textural Scaling)".into(),
        category: "Saturation".into(),
        doc: "LRLC IMTS model: Waxman-Smits-family conductivity with the clay charge \
              referenced to the ACTIVE water — Qv_eff = Qv_bulk/(1−Swirr), where Qv_bulk \
              is built from clay volumes (kaolinite/illite) times literature CEC constants \
              (8 / 25 meq/100g), calibrated to lab CEC by scaling factor S. Iterates \
              Ct = SwT^N*/F*·(Cw + B·Qv_eff/SwT) with F* = A/PHIT^M* and Juhasz B(T, Rw) \
              until SwT is stable. SWE from CBW. VKAOL/VILL default to SSC's VDCL and a \
              zero illite curve; S ≈ lab CEC / XRD-theoretical CEC (typically < 1)."
            .into(),
        args: vec![
            param("RW", "Formation water resistivity at FT", "ohm.m", 0.3, 0.001, 100.0),
            param("TEMP_C", "Formation temperature", "degC", 60.0, 15.0, 200.0),
            param("A", "Tortuosity factor a", "", 1.0, 0.5, 3.0),
            param("MSTAR", "Shaly-sand cementation exponent m*", "", 1.9, 1.0, 4.0),
            param("NSTAR", "Shaly-sand saturation exponent n*", "", 1.9, 1.0, 4.0),
            param("S_FACTOR", "CEC scaling factor S (lab/XRD)", "", 0.5, 0.01, 2.0),
            param("CEC_KAOL", "Kaolinite CEC constant", "meq/100g", 8.0, 0.0, 50.0),
            param("CEC_ILL", "Illite CEC constant", "meq/100g", 25.0, 0.0, 100.0),
            param("RHOG", "Grain density", "g/cc", 2.65, 2.0, 3.2),
            param("SWIRR_DEF", "Swirr fallback when no SWIRR log", "v/v", 0.2, 0.0, 0.95),
            log_in("RT", "Deep resistivity", "ohm.m", "RES_DEEP", true),
            log_in("PHIT", "Total porosity", "v/v", "PHIT_SSC", true),
            log_in("VKAOL", "Kaolinite volume fraction", "v/v", "VDCL", false),
            log_in("VILL", "Illite volume fraction", "v/v", "VILL", false),
            log_in("SWIRR", "Irreducible Sw (for Qv_eff)", "v/v", "SWIRR_T", false),
            log_in("CBW", "Clay-bound water (for SWE), optional", "v/v", "CBW", false),
            log_in("PHIT_SSPW", "Total porosity — SSPW fallback (used where PHIT is absent)", "v/v", "PHIT_SSPW", false),
            log_in("CBW_SSPW", "Clay-bound water — SSPW fallback", "v/v", "CBW_SSPW", false),
            log_out("SWT_IMTS", "Total water saturation, IMTS", "v/v"),
            log_out("SWE_IMTS", "Effective water saturation, IMTS", "v/v"),
            log_out("QVEFF", "Effective Qv (meq/cm3)", "meq/cm3"),
        ],
    }
}

pub fn sw_imts(ctx: &ModuleContext) -> ModuleOutputs {
    let rt = ctx.log("RT");
    // SSPW fallback (see sw_rtc): PHIT/CBW fall back to their _SSPW equivalents when absent.
    let phit = prefer(&ctx.log("PHIT"), &ctx.log("PHIT_SSPW"));
    let vkaol = ctx.log("VKAOL");
    let vill = ctx.log("VILL");
    let swirr_log = ctx.log("SWIRR");
    let cbw = prefer(&ctx.log("CBW"), &ctx.log("CBW_SSPW"));

    let mut swt_o = vec![f32::NAN; ctx.n];
    let mut swe_o = vec![f32::NAN; ctx.n];
    let mut qveff_o = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let (rt_i, pt) = (rt[i] as f64, phit[i] as f64);
        let rw = ctx.p("RW", i);
        let temp_c = ctx.p("TEMP_C", i);
        let a = ctx.p("A", i);
        let mstar = ctx.p("MSTAR", i);
        let nstar = ctx.p("NSTAR", i);
        if rt_i.is_nan() || pt.is_nan() || rw.is_nan() || rt_i <= 0.0 || pt <= 0.001 {
            continue;
        }

        let vk = if (vkaol[i] as f64).is_nan() { 0.0 } else { limit(vkaol[i] as f64, 0.0, 1.0) };
        let vi = if (vill[i] as f64).is_nan() { 0.0 } else { limit(vill[i] as f64, 0.0, 1.0) };
        let swirr = {
            let s = swirr_log[i] as f64;
            let s = if s.is_nan() { ctx.p("SWIRR_DEF", i) } else { s };
            limit(s, 0.0, 0.95)
        };

        // Qv_bulk from scaled XRD clay charge; Qv_eff references the active water.
        let cec_bulk = ctx.p("S_FACTOR", i)
            * (vk * ctx.p("CEC_KAOL", i) + vi * ctx.p("CEC_ILL", i));
        let qv_bulk = cec_bulk * ctx.p("RHOG", i) * (1.0 - pt) / (100.0 * pt);
        let qv_eff = qv_bulk / (1.0 - swirr);
        qveff_o[i] = qv_eff as f32;

        let ct = 1.0 / rt_i;
        let cw = 1.0 / rw;
        let fstar = a / pt.powf(mstar);
        let b = juhasz_b(temp_c, rw).max(0.0);

        // Iterate SwT^n*/F*·(Cw + B·Qv_eff/SwT) = Ct, seeded with the Archie-like value.
        let mut sw = limit((fstar * ct / cw).powf(1.0 / nstar), 0.01, 1.0);
        for _ in 0..100 {
            // Waxman-Smits form: the excess-conductivity term is referenced to the ACTIVE
            // water, so it DIVIDES by Sw — it grows as hydrocarbon displaces water instead of
            // vanishing (the old `* sw` gave Sw^(n*+1), understating clay conductivity and
            // overstating Sw in pay). Floor Sw to keep the division finite near zero.
            let denom = cw + b * qv_eff / sw.max(1e-6);
            if denom <= 0.0 {
                sw = f64::NAN;
                break;
            }
            let next = limit((fstar * ct / denom).powf(1.0 / nstar), 0.0, 1.0);
            if (next - sw).abs() < 1e-6 {
                sw = next;
                break;
            }
            sw = next;
        }
        if sw.is_nan() {
            continue;
        }
        swt_o[i] = sw as f32;

        let cb = cbw[i] as f64;
        if !cb.is_nan() && pt > cb {
            let swb = limit(cb / pt, 0.0, 0.99);
            swe_o[i] = limit((sw - swb) / (1.0 - swb), 0.0, 1.0) as f32;
        } else {
            swe_o[i] = sw as f32;
        }
    }

    HashMap::from([
        ("SWT_IMTS".to_string(), swt_o),
        ("SWE_IMTS".to_string(), swe_o),
        ("QVEFF".to_string(), qveff_o),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ArgKind;

    fn ctx_with(logs: Vec<(&str, Vec<f32>)>, spec: &ModuleSpec, n: usize) -> ModuleContext {
        let mut params = HashMap::new();
        let mut opts = HashMap::new();
        for arg in &spec.args {
            match arg.kind {
                ArgKind::Param => {
                    params.insert(arg.name.clone(), vec![arg.default.parse::<f64>().unwrap(); n]);
                }
                ArgKind::Option => {
                    opts.insert(arg.name.clone(), arg.default.clone());
                }
                _ => {}
            }
        }
        ModuleContext {
            n,
            logs: logs.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            params,
            opts,
            depth_unit: Default::default(),
        }
    }

    #[test]
    fn rtc_lowers_sw_versus_archie_when_capillary_water_present() {
        // LRLC scenario: Rt 4 ohm.m, PHIT 0.25, plenty of capillary water.
        let spec = sw_rtc_spec();
        let ctx = ctx_with(
            vec![
                ("RT", vec![4.0]),
                ("PHIT", vec![0.25]),
                ("CAPBW", vec![0.08]),
                ("CBW", vec![0.03]),
            ],
            &spec,
            1,
        );
        let out = sw_rtc(&ctx);
        let swt = out["SWT_RTC"][0] as f64;
        // Plain Archie with the same params.
        let archie = (0.3 * (1.0 / 4.0) / 0.25_f64.powf(2.0)).powf(0.5);
        assert!(swt < archie, "RtC must lower Sw vs Archie: {swt} vs {archie}");
        assert!(out["RT_CORR"][0] > 4.0, "corrected Rt must rise");
        assert!(out["SWE_RTC"][0] <= out["SWT_RTC"][0], "SWE <= SWT");
        assert!(out["CEX_RTC"][0] > 0.0);
    }

    /// SSPW fallback: sw_rtc must run on a well whose porosity came from the SSPW workflow
    /// (PHIT_SSPW / CAPBW_SSPW / CBW_SSPW) even though the input defaults name the SSC curves —
    /// otherwise it silently produced an all-NaN 'success' on SSPW-only wells.
    #[test]
    fn rtc_falls_back_to_sspw_curve_names() {
        let spec = sw_rtc_spec();
        // Only the SSPW-named curves are present (PHIT/CAPBW/CBW absent → all-NaN).
        let ctx = ctx_with(
            vec![
                ("RT", vec![4.0]),
                ("PHIT_SSPW", vec![0.25]),
                ("CAPBW_SSPW", vec![0.08]),
                ("CBW_SSPW", vec![0.03]),
            ],
            &spec,
            1,
        );
        let out = sw_rtc(&ctx);
        assert!(
            out["SWT_RTC"][0].is_finite(),
            "SWT_RTC must be computed from the SSPW fallback curves, got NaN"
        );
        assert!(out["RT_CORR"][0] > 4.0, "capillary correction (via CAPBW_SSPW) must raise Rt");
        assert!(out["SWE_RTC"][0] <= out["SWT_RTC"][0], "SWE <= SWT");
    }

    #[test]
    fn rtc_with_no_excess_terms_matches_archie() {
        let spec = sw_rtc_spec();
        // RT 8 ohm.m keeps the Archie answer below 1 so the limit does not bite.
        let mut ctx = ctx_with(vec![("RT", vec![8.0]), ("PHIT", vec![0.25])], &spec, 1);
        // Zero out the regression so Cex = 0.
        ctx.params.insert("A_CAP".into(), vec![0.0]);
        ctx.params.insert("B_QV".into(), vec![0.0]);
        ctx.params.insert("C0".into(), vec![0.0]);
        let out = sw_rtc(&ctx);
        let archie = (0.3 * (1.0 / 8.0) / 0.25_f64.powf(2.0)).powf(0.5) as f32;
        assert!((out["SWT_RTC"][0] - archie).abs() < 1e-5);
        assert!((out["RT_CORR"][0] - 8.0).abs() < 1e-5);
    }

    #[test]
    fn rtc_correction_is_capped_so_rt_stays_finite() {
        let spec = sw_rtc_spec();
        let ctx = ctx_with(
            vec![("RT", vec![50.0]), ("PHIT", vec![0.3]), ("CAPBW", vec![0.3])],
            &spec,
            1,
        );
        let out = sw_rtc(&ctx);
        assert!(out["RT_CORR"][0].is_finite() && out["RT_CORR"][0] > 0.0);
        assert!(out["SWT_RTC"][0] >= 0.0);
    }

    #[test]
    fn imts_converges_and_sits_below_archie_in_clayey_rock() {
        let spec = sw_imts_spec();
        let ctx = ctx_with(
            vec![
                ("RT", vec![4.0]),
                ("PHIT", vec![0.25]),
                ("VKAOL", vec![0.20]),
                ("VILL", vec![0.05]),
                ("SWIRR", vec![0.30]),
                ("CBW", vec![0.03]),
            ],
            &spec,
            1,
        );
        let out = sw_imts(&ctx);
        let swt = out["SWT_IMTS"][0] as f64;
        assert!(swt > 0.0 && swt <= 1.0, "SwT out of range: {swt}");
        // With extra clay conductivity explained, SwT must be below the Archie-like seed.
        let seed = ((1.0 / 0.25_f64.powf(1.9)) * (1.0 / 4.0) / (1.0 / 0.3)).powf(1.0 / 1.9);
        assert!(swt < seed, "IMTS must sit below Archie seed: {swt} vs {seed}");
        assert!(out["QVEFF"][0] > 0.0);
        assert!(out["SWE_IMTS"][0] <= out["SWT_IMTS"][0]);
    }

    #[test]
    fn imts_without_clay_reduces_to_archie_form() {
        let spec = sw_imts_spec();
        // RT 8 ohm.m keeps the Archie-form answer below 1 so the limit does not bite.
        let ctx = ctx_with(
            vec![("RT", vec![8.0]), ("PHIT", vec![0.25]), ("SWIRR", vec![0.2])],
            &spec,
            1,
        );
        let out = sw_imts(&ctx);
        let expect = ((1.0 / 0.25_f64.powf(1.9)) * (1.0 / 8.0) * 0.3).powf(1.0 / 1.9) as f32;
        assert!((out["SWT_IMTS"][0] - expect).abs() < 1e-4, "{} vs {}", out["SWT_IMTS"][0], expect);
        assert_eq!(out["QVEFF"][0], 0.0);
    }

    #[test]
    fn imts_credits_clay_conductivity_in_pay_zone() {
        // High-Rt clayey pay zone. The excess-conductivity term references the ACTIVE water
        // (divides by Sw), so as hydrocarbon displaces water the clay conductivity is credited
        // MORE, not less — pulling IMTS SwT well below the Archie seed (~0.67·seed here). The
        // old (·Sw) form let the clay term vanish in pay (Sw^(n*+1)), leaving SwT ≈ 0.93·seed;
        // this assertion fails under that bug.
        let spec = sw_imts_spec();
        let ctx = ctx_with(
            vec![
                ("RT", vec![20.0]),
                ("PHIT", vec![0.25]),
                ("VKAOL", vec![0.20]),
                ("VILL", vec![0.05]),
                ("SWIRR", vec![0.30]),
            ],
            &spec,
            1,
        );
        let out = sw_imts(&ctx);
        let swt = out["SWT_IMTS"][0] as f64;
        assert!(swt > 0.0 && swt <= 1.0, "SwT out of range: {swt}");
        let seed = ((1.0 / 0.25_f64.powf(1.9)) * (1.0 / 20.0) / (1.0 / 0.3)).powf(1.0 / 1.9);
        assert!(
            swt < 0.85 * seed,
            "IMTS must credit clay conductivity in pay: {swt} vs seed {seed}"
        );
    }

    #[test]
    fn juhasz_b_is_positive_and_grows_with_temperature() {
        let b60 = juhasz_b(60.0, 0.3);
        let b90 = juhasz_b(90.0, 0.3);
        assert!(b60 > 0.0 && b90 > b60);
    }
}
