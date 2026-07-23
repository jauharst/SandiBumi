//! Unconventional / shale suite (enrichment #7) — organic-richness, kerogen, gas-in-place, and
//! brittleness modules. All method math + primary-source citations + Tier-A vendor default seeds are
//! banked in `docs/ref_unconventional.md` (portable, not machine-local memory). Every method here is
//! Tier B (published science, reimplemented from the primary paper); default parameter values are
//! Tier-A IP/Techlog seeds, exposed as per-well-overridable params.
//!
//! Increment 1 — `toc_passey`: total organic carbon from the Passey (1990) ΔlogR overlay (deep
//! resistivity vs a baselined porosity curve) with the LOM→TOC maturity conversion, plus the
//! Schmoker-Hester (1983) density-TOC as a cross-check. See `docs/ref_unconventional.md` §1.

use crate::modules::{log_in, log_out, opt, param, ModuleContext, ModuleOutputs, ModuleSpec};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// TOC — Passey ΔlogR + Schmoker density  (docs/ref_unconventional.md §1)
// ---------------------------------------------------------------------------

pub fn toc_passey_spec() -> ModuleSpec {
    ModuleSpec {
        name: "toc_passey".into(),
        title: "TOC — Passey ΔlogR + Schmoker".into(),
        category: "Unconventional".into(),
        doc: "Total organic carbon from the Passey (1990) ΔlogR overlay — the separation between deep \
              resistivity and a baselined porosity curve — converted to TOC with the maturity term \
              10^(2.297−0.1688·LOM). ΔlogR = log10(R/R_base) + 0.02·(DT−DT_base) [sonic overlay] or \
              −2.5·(RHOB−RHOB_base) [density overlay]. Baselines are picked on a non-source, clay-rich \
              interval (params, per-zone overridable) where the two curves overlie; where ΔlogR<0 the \
              rock is non-source and TOC floors to the background value. Also writes the Schmoker-Hester \
              (1983) density-TOC 154.497/RHOB−57.261 as an independent cross-check whenever RHOB is \
              present. TOC in wt%. LOM 6..12 (Passey is calibrated to LOM≤12). Cite: Passey, Creaney, \
              Kulla, Moretti & Stroud 1990, AAPG Bull. 74(12); Schmoker & Hester 1983, AAPG Bull. \
              67(12). See docs/ref_unconventional.md §1. (Neutron overlay deferred — sign convention \
              needs core verification.)"
            .into(),
        args: vec![
            opt("OVERLAY", "Porosity curve paired with resistivity for ΔlogR", "sonic",
                &["sonic", "density"]),
            param("R_BASE", "Baseline resistivity (non-source interval)", "ohm.m", 2.0, 0.001, 100000.0),
            param("DT_BASE", "Baseline sonic Δt (sonic overlay)", "us/ft", 70.0, 40.0, 200.0),
            param("RHOB_BASE", "Baseline bulk density (density overlay)", "g/cc", 2.65, 1.5, 3.5),
            param("LOM", "Level of organic maturity (Hood scale, 6..12)", "-", 10.6, 6.0, 12.0),
            param("TOC_BG", "Background TOC of the baseline rock", "wt%", 0.0, 0.0, 10.0),
            log_in("RES", "Deep resistivity", "ohm.m", "RT", true),
            log_in("DT", "Sonic Δt (sonic overlay)", "us/ft", "DT", false),
            log_in("RHOB", "Bulk density (density overlay + Schmoker)", "g/cc", "RHOB", false),
            log_out("DLOGR", "Passey resistivity–porosity separation (log10 cycles)", "-"),
            log_out("TOC", "Total organic carbon (Passey ΔlogR)", "wt%"),
            log_out("TOC_SCHMOKER", "Density-TOC cross-check (Schmoker-Hester 1983)", "wt%"),
        ],
    }
}

pub fn toc_passey(ctx: &ModuleContext) -> ModuleOutputs {
    let res = ctx.log("RES");
    let dt = ctx.log("DT");
    let rhob = ctx.log("RHOB");
    let overlay = ctx.o("OVERLAY").to_string();
    let n = ctx.n;

    let mut dlogr = vec![f32::NAN; n];
    let mut toc = vec![f32::NAN; n];
    let mut toc_schmoker = vec![f32::NAN; n];

    for i in 0..n {
        // --- Passey ΔlogR + TOC (Passey et al. 1990) ---
        let r = res[i] as f64;
        let r_base = ctx.p("R_BASE", i);
        let lom = ctx.p("LOM", i);
        let toc_bg = ctx.p("TOC_BG", i);
        // log10(R/R_base) needs both strictly positive.
        if r.is_finite() && r > 0.0 && r_base.is_finite() && r_base > 0.0 {
            // Porosity term for the chosen overlay; NaN if the curve/baseline it needs is absent.
            let poro_term = if overlay == "density" {
                let v = rhob[i] as f64;
                let base = ctx.p("RHOB_BASE", i);
                if v.is_finite() && base.is_finite() { -2.5 * (v - base) } else { f64::NAN }
            } else {
                // sonic (default)
                let v = dt[i] as f64;
                let base = ctx.p("DT_BASE", i);
                if v.is_finite() && base.is_finite() { 0.02 * (v - base) } else { f64::NAN }
            };
            if poro_term.is_finite() {
                let d = (r / r_base).log10() + poro_term;
                // ΔlogR is LOM-independent, so it is emitted regardless of maturity, and stored even
                // when negative (the overlay panel shades the separation either way).
                dlogr[i] = d as f32;
                if lom.is_finite() {
                    // TOC = max(0, ΔlogR·10^(2.297−0.1688·LOM)) + background. Clamp the Passey term at
                    // 0 FIRST (ΔlogR<0 ⇒ non-source), THEN add the baseline TOC, so TOC never falls
                    // BELOW the background value (Passey et al. 1990).
                    let factor = 10f64.powf(2.297 - 0.1688 * lom);
                    let bg = if toc_bg.is_finite() { toc_bg } else { 0.0 };
                    toc[i] = ((d * factor).max(0.0) + bg) as f32;
                }
            }
        }

        // --- Schmoker-Hester (1983) density-TOC cross-check (independent of the overlay) ---
        let rb = rhob[i] as f64;
        if rb.is_finite() && rb > 0.0 {
            toc_schmoker[i] = (154.497 / rb - 57.261).max(0.0) as f32;
        }
    }

    HashMap::from([
        ("DLOGR".into(), dlogr),
        ("TOC".into(), toc),
        ("TOC_SCHMOKER".into(), toc_schmoker),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Passey TOC conversion factor at a given LOM: 10^(2.297 − 0.1688·LOM).
    fn factor(lom: f64) -> f64 {
        10f64.powf(2.297 - 0.1688 * lom)
    }

    /// Minimal ModuleContext for toc_passey. Params filled full-length (ctx.p reads per-sample).
    fn ctx(
        overlay: &str,
        res: Vec<f32>,
        dt: Vec<f32>,
        rhob: Vec<f32>,
        r_base: f64,
        dt_base: f64,
        rhob_base: f64,
        lom: f64,
        toc_bg: f64,
    ) -> ModuleContext {
        let n = res.len();
        let mut logs = HashMap::new();
        logs.insert("RES".to_string(), res);
        logs.insert("DT".to_string(), dt);
        logs.insert("RHOB".to_string(), rhob);
        let mut params = HashMap::new();
        params.insert("R_BASE".to_string(), vec![r_base; n]);
        params.insert("DT_BASE".to_string(), vec![dt_base; n]);
        params.insert("RHOB_BASE".to_string(), vec![rhob_base; n]);
        params.insert("LOM".to_string(), vec![lom; n]);
        params.insert("TOC_BG".to_string(), vec![toc_bg; n]);
        let mut opts = HashMap::new();
        opts.insert("OVERLAY".to_string(), overlay.to_string());
        ModuleContext { n, logs, params, opts }
    }

    #[test]
    fn sonic_overlay_recovers_known_toc() {
        // Sample A: R=20 vs R_base=2 → log10(10)=1.0; DT=70=DT_base → sonic term 0 → ΔlogR=1.0.
        // Sample B: R=2=R_base → 0; DT=120 → 0.02·(120−70)=1.0 → ΔlogR=1.0 (proves the sonic term).
        let nan = f32::NAN;
        let c = ctx("sonic", vec![20.0, 2.0], vec![70.0, 120.0], vec![nan, nan],
                    2.0, 70.0, 2.65, 10.6, 0.0);
        let out = toc_passey(&c);
        let dlr = &out["DLOGR"];
        let toc = &out["TOC"];
        assert!((dlr[0] as f64 - 1.0).abs() < 1e-4, "ΔlogR A = {}", dlr[0]);
        assert!((dlr[1] as f64 - 1.0).abs() < 1e-4, "ΔlogR B = {}", dlr[1]);
        // Hand literal (guards a typo in the 2.297/0.1688 constants): factor(10.6)=10^0.50772=3.219,
        // ΔlogR=1.0 ⇒ TOC=3.219 wt%.
        let expect = 3.219;
        assert!((factor(10.6) - expect).abs() < 3e-3, "factor(10.6) = {}", factor(10.6));
        assert!((toc[0] as f64 - expect).abs() < 3e-3, "TOC A = {} vs {}", toc[0], expect);
        assert!((toc[1] as f64 - expect).abs() < 3e-3, "TOC B = {} vs {}", toc[1], expect);
    }

    #[test]
    fn density_overlay_recovers_known_toc_and_schmoker() {
        // R=20 vs 2 → 1.0; RHOB=2.25 vs base 2.65 → −2.5·(−0.40)=1.0 → ΔlogR=2.0.
        let c = ctx("density", vec![20.0], vec![f32::NAN], vec![2.25],
                    2.0, 70.0, 2.65, 10.6, 0.0);
        let out = toc_passey(&c);
        assert!((out["DLOGR"][0] as f64 - 2.0).abs() < 1e-4, "ΔlogR = {}", out["DLOGR"][0]);
        let expect = 2.0 * factor(10.6); // ≈ 6.437 wt%
        assert!((out["TOC"][0] as f64 - expect).abs() < 2e-3, "TOC = {} vs {}", out["TOC"][0], expect);
        // Schmoker-Hester: 154.497/2.25 − 57.261 = 11.404 wt%.
        let sch = 154.497 / 2.25 - 57.261;
        assert!((out["TOC_SCHMOKER"][0] as f64 - sch).abs() < 2e-3,
                "Schmoker = {} vs {}", out["TOC_SCHMOKER"][0], sch);
    }

    #[test]
    fn non_source_interval_floors_toc_to_zero() {
        // R=1 below R_base=2 → log10(0.5)=−0.301; DT=70=base → ΔlogR=−0.301 (negative).
        // TOC floors at 0 (non-source), but DLOGR keeps the real negative separation.
        let c = ctx("sonic", vec![1.0], vec![70.0], vec![f32::NAN],
                    2.0, 70.0, 2.65, 10.6, 0.0);
        let out = toc_passey(&c);
        assert!(out["DLOGR"][0] < 0.0, "ΔlogR should be negative, got {}", out["DLOGR"][0]);
        assert_eq!(out["TOC"][0], 0.0, "non-source TOC floors to 0");
    }

    #[test]
    fn non_source_interval_floors_toc_to_background_not_zero() {
        // ΔlogR<0 (non-source) with a positive background TOC: the Passey term floors at 0, then the
        // baseline TOC is ADDED, so TOC floors to background (0.7) and never below it. Regression for
        // the clamp-order fix — (d·factor).max(0)+bg, NOT (d·factor+bg).max(0).
        let c = ctx("sonic", vec![1.0], vec![70.0], vec![f32::NAN], 2.0, 70.0, 2.65, 10.6, 0.7);
        let out = toc_passey(&c);
        assert!(out["DLOGR"][0] < 0.0, "ΔlogR should be negative, got {}", out["DLOGR"][0]);
        assert!((out["TOC"][0] as f64 - 0.7).abs() < 1e-4,
                "non-source TOC floors to background 0.7, got {}", out["TOC"][0]);
    }

    #[test]
    fn toc_decreases_with_maturity_lom() {
        // Same ΔlogR (=1.0), lower LOM (less mature) ⇒ larger conversion factor ⇒ higher TOC.
        let mk = |lom: f64| {
            let c = ctx("sonic", vec![20.0], vec![70.0], vec![f32::NAN], 2.0, 70.0, 2.65, lom, 0.0);
            toc_passey(&c)["TOC"][0] as f64
        };
        let t8 = mk(8.0);
        let t12 = mk(12.0);
        assert!(t8 > t12, "immature LOM 8 ({t8}) should read higher TOC than overmature LOM 12 ({t12})");
        assert!((t8 - factor(8.0)).abs() < 1e-2, "TOC at LOM 8 = ΔlogR·factor");
    }

    #[test]
    fn background_toc_offsets_the_curve() {
        // ΔlogR=1.0 with a 0.5 wt% background ⇒ TOC = factor + 0.5.
        let c = ctx("sonic", vec![20.0], vec![70.0], vec![f32::NAN], 2.0, 70.0, 2.65, 10.6, 0.5);
        let out = toc_passey(&c);
        let expect = factor(10.6) + 0.5;
        assert!((out["TOC"][0] as f64 - expect).abs() < 2e-3, "TOC = {} vs {}", out["TOC"][0], expect);
    }

    #[test]
    fn missing_overlay_curve_leaves_passey_nan_but_schmoker_runs() {
        // Sonic overlay selected but DT absent (all NaN) ⇒ ΔlogR/TOC stay MISSING; the RHOB-based
        // Schmoker cross-check still runs from RHOB alone.
        let c = ctx("sonic", vec![20.0], vec![f32::NAN], vec![2.40], 2.0, 70.0, 2.65, 10.6, 0.0);
        let out = toc_passey(&c);
        assert!(out["DLOGR"][0].is_nan(), "no DT ⇒ ΔlogR missing");
        assert!(out["TOC"][0].is_nan(), "no DT ⇒ Passey TOC missing");
        let sch = 154.497 / 2.40 - 57.261; // 7.113 wt%
        assert!((out["TOC_SCHMOKER"][0] as f64 - sch).abs() < 2e-3,
                "Schmoker runs from RHOB: {} vs {}", out["TOC_SCHMOKER"][0], sch);
    }
}
