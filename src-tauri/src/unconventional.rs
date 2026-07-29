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

// ---------------------------------------------------------------------------
// Kerogen volume + OM-corrected porosity  (docs/ref_unconventional.md §2)
// ---------------------------------------------------------------------------

pub fn kerogen_spec() -> ModuleSpec {
    ModuleSpec {
        name: "kerogen".into(),
        title: "Kerogen volume + OM-corrected porosity".into(),
        category: "Unconventional".into(),
        doc: "Converts TOC (weight %) to kerogen VOLUME and corrects total porosity for the organic \
              matter that low-density kerogen inflates on the density log. TOM = k_toc2om·TOC/100 \
              (organic-matter weight fraction; k_toc2om≈1.2 accounts for the H/O/N/S beyond carbon), \
              then VKER = TOM·RHOB/ρ_kero (kerogen volume fraction of the BULK rock — the Passey/Vernik \
              bulk-density conversion, directly comparable to SandiMin VOL_KEROGEN). PHIT_OMC = \
              PHIT − VKER removes kerogen's apparent-porosity contribution (ρ_kero≈ρ_fluid, first \
              order). ρ_kero default 1.10 g/cc matches the SandiMin Kerogen mineral (multimin2.rs), so \
              VKER reconciles with VOL_KEROGEN (IP's RHOTOC seed is 1.25 — override if you prefer it). \
              Cite: Passey et al. 2010 (SPE 131350); Vernik & Nur 1992. See docs/ref_unconventional.md §2."
            .into(),
        args: vec![
            param("RHO_KERO", "Kerogen (organic-matter) grain density", "g/cc", 1.10, 0.9, 1.6),
            param("K_TOC2OM", "TOC→organic-matter factor (1.2 immature .. 1.35 mature)", "-", 1.2, 1.0, 1.6),
            log_in("TOC", "Total organic carbon", "wt%", "TOC", true),
            log_in("RHOB", "Bulk density", "g/cc", "RHOB", true),
            log_in("PHIT", "Total porosity to OM-correct (optional)", "v/v", "PHIT", false),
            log_out("TOM", "Organic-matter weight fraction", "wt/wt"),
            log_out("VKER", "Kerogen volume fraction (bulk)", "v/v"),
            log_out("PHIT_OMC", "OM-corrected total porosity", "v/v"),
        ],
    }
}

pub fn kerogen(ctx: &ModuleContext) -> ModuleOutputs {
    let toc_log = ctx.log("TOC");
    let rhob = ctx.log("RHOB");
    let phit = ctx.log("PHIT");
    let n = ctx.n;

    let mut tom = vec![f32::NAN; n];
    let mut vker = vec![f32::NAN; n];
    let mut phit_omc = vec![f32::NAN; n];

    for i in 0..n {
        let toc = toc_log[i] as f64; // wt%
        let k = ctx.p("K_TOC2OM", i);
        let rho_k = ctx.p("RHO_KERO", i);
        // Organic-matter weight fraction — needs only TOC + the conversion factor.
        if toc.is_finite() && toc >= 0.0 && k.is_finite() && k > 0.0 {
            let tom_i = k * toc / 100.0;
            tom[i] = tom_i as f32;
            // Kerogen bulk volume fraction — needs bulk density + kerogen density.
            let rb = rhob[i] as f64;
            if rb.is_finite() && rb > 0.0 && rho_k.is_finite() && rho_k > 0.0 {
                let vk = (tom_i * rb / rho_k).clamp(0.0, 1.0);
                vker[i] = vk as f32;
                // OM-corrected total porosity: strip kerogen's apparent-porosity contribution.
                let p = phit[i] as f64;
                if p.is_finite() {
                    phit_omc[i] = (p - vk).max(0.0) as f32;
                }
            }
        }
    }

    HashMap::from([
        ("TOM".into(), tom),
        ("VKER".into(), vker),
        ("PHIT_OMC".into(), phit_omc),
    ])
}

#[cfg(test)]
mod kerogen_tests {
    use super::*;

    fn ctx(toc: Vec<f32>, rhob: Vec<f32>, phit: Vec<f32>, rho_kero: f64, k: f64) -> ModuleContext {
        let n = toc.len();
        let mut logs = HashMap::new();
        logs.insert("TOC".to_string(), toc);
        logs.insert("RHOB".to_string(), rhob);
        logs.insert("PHIT".to_string(), phit);
        let mut params = HashMap::new();
        params.insert("RHO_KERO".to_string(), vec![rho_kero; n]);
        params.insert("K_TOC2OM".to_string(), vec![k; n]);
        ModuleContext { n, logs, params, opts: HashMap::new(), depth_unit: Default::default() }
    }

    #[test]
    fn kerogen_volume_matches_bulk_massbalance() {
        // TOC 3 wt%, RHOB 2.4, ρ_kero 1.2, k 1.2: TOM = 1.2·0.03 = 0.036; VKER = 0.036·2.4/1.2 = 0.072.
        let out = kerogen(&ctx(vec![3.0], vec![2.4], vec![f32::NAN], 1.2, 1.2));
        assert!((out["TOM"][0] as f64 - 0.036).abs() < 1e-5, "TOM = {}", out["TOM"][0]);
        assert!((out["VKER"][0] as f64 - 0.072).abs() < 1e-5, "VKER = {}", out["VKER"][0]);
    }

    #[test]
    fn om_corrected_porosity_subtracts_kerogen_and_floors() {
        // PHIT 0.20 → 0.20 − 0.072 = 0.128.
        let out = kerogen(&ctx(vec![3.0], vec![2.4], vec![0.20], 1.2, 1.2));
        assert!((out["PHIT_OMC"][0] as f64 - 0.128).abs() < 1e-4, "PHIT_OMC = {}", out["PHIT_OMC"][0]);
        // Thin porosity floors at 0 when kerogen exceeds it.
        let out2 = kerogen(&ctx(vec![3.0], vec![2.4], vec![0.05], 1.2, 1.2));
        assert_eq!(out2["PHIT_OMC"][0], 0.0, "PHIT_OMC floors at 0");
    }

    #[test]
    fn zero_toc_gives_zero_kerogen_and_unchanged_porosity() {
        let out = kerogen(&ctx(vec![0.0], vec![2.65], vec![0.15], 1.2, 1.2));
        assert_eq!(out["TOM"][0], 0.0);
        assert_eq!(out["VKER"][0], 0.0);
        assert!((out["PHIT_OMC"][0] as f64 - 0.15).abs() < 1e-6, "PHIT unchanged when no kerogen");
    }

    #[test]
    fn kerogen_volume_increases_with_toc() {
        let lo = kerogen(&ctx(vec![2.0], vec![2.5], vec![f32::NAN], 1.2, 1.2))["VKER"][0];
        let hi = kerogen(&ctx(vec![6.0], vec![2.5], vec![f32::NAN], 1.2, 1.2))["VKER"][0];
        assert!(hi > lo, "VKER should rise with TOC: {lo} -> {hi}");
    }

    #[test]
    fn missing_rhob_leaves_vker_nan_but_tom_runs() {
        // TOM needs only TOC; VKER (and thus PHIT_OMC) need RHOB.
        let out = kerogen(&ctx(vec![3.0], vec![f32::NAN], vec![0.20], 1.2, 1.2));
        assert!((out["TOM"][0] as f64 - 0.036).abs() < 1e-5, "TOM runs from TOC alone");
        assert!(out["VKER"][0].is_nan(), "no RHOB ⇒ VKER missing");
        assert!(out["PHIT_OMC"][0].is_nan(), "no VKER ⇒ PHIT_OMC missing");
    }
}

// ---------------------------------------------------------------------------
// Gas-in-place — free + Langmuir-adsorbed (+ CBM)  (docs/ref_unconventional.md §3)
// ---------------------------------------------------------------------------

pub fn gip_spec() -> ModuleSpec {
    ModuleSpec {
        name: "gip".into(),
        title: "Gas-in-place (free + Langmuir adsorbed)".into(),
        category: "Unconventional".into(),
        doc: "Per-sample gas-in-place as gas CONTENT (scf per ton of rock), so it composites like any \
              curve. Adsorbed via the Langmuir isotherm GIP_ADS = VL·P/(PL+P); free via \
              GIP_FREE = 32.0368·φ·(1−Sw)/(RHOB·Bg) with Bg = 0.02827·z·T/P (T in Rankine); \
              GIP_TOTAL = free + adsorbed. MODE=cbm applies the dry-ash-free correction \
              GIP_ADS·(1−F_ASH−F_MOIST) and, given a measured in-situ gas content GC, emits the \
              critical desorption pressure PCD = PL·GC/(VL−GC). Langmuir VL/PL default to shale \
              placeholders — override with core desorption/isotherm data (IP seeds 60 cm³/g ≈ 1920 \
              scf/ton and 7000 kPaa ≈ 1015 psia for coal). Ambrose pore-volume correction deferred. \
              Cite: Langmuir 1918; Ambrose et al. 2010; GRI/Mavor-Nelson 1996. See \
              docs/ref_unconventional.md §3."
            .into(),
        args: vec![
            opt("MODE", "Reservoir type (cbm adds ash/moisture + critical desorption)", "shale",
                &["shale", "cbm"]),
            param("RES_P", "Reservoir (pore) pressure", "psia", 3000.0, 1.0, 30000.0),
            param("TEMP_F", "Reservoir temperature", "degF", 200.0, 32.0, 600.0),
            param("Z_FAC", "Gas deviation (compressibility) factor z", "-", 0.9, 0.2, 2.0),
            param("VL", "Langmuir volume (max sorption)", "scf/ton", 100.0, 0.0, 5000.0),
            param("PL", "Langmuir pressure (Gs = VL/2)", "psia", 1000.0, 1.0, 30000.0),
            param("F_ASH", "Ash weight fraction (cbm)", "-", 0.0, 0.0, 1.0),
            param("F_MOIST", "Moisture weight fraction (cbm)", "-", 0.0, 0.0, 1.0),
            param("GC", "In-situ gas content for PCD (cbm; 0 = saturated)", "scf/ton", 0.0, 0.0, 5000.0),
            log_in("PHI", "Porosity (effective, or OM-corrected total)", "v/v", "PHIE", true),
            log_in("SW", "Water saturation", "v/v", "SWE", true),
            log_in("RHOB", "Bulk density", "g/cc", "RHOB", true),
            log_out("BG", "Gas formation volume factor", "rcf/scf"),
            log_out("GIP_ADS", "Adsorbed gas content (Langmuir)", "scf/ton"),
            log_out("GIP_FREE", "Free gas content", "scf/ton"),
            log_out("GIP_TOTAL", "Total gas content (free + adsorbed)", "scf/ton"),
            log_out("PCD", "Critical desorption pressure (cbm)", "psia"),
        ],
    }
}

pub fn gip(ctx: &ModuleContext) -> ModuleOutputs {
    let phi_log = ctx.log("PHI");
    let sw_log = ctx.log("SW");
    let rhob = ctx.log("RHOB");
    let cbm = ctx.o("MODE") == "cbm";
    let n = ctx.n;

    let mut bg = vec![f32::NAN; n];
    let mut gip_ads = vec![f32::NAN; n];
    let mut gip_free = vec![f32::NAN; n];
    let mut gip_total = vec![f32::NAN; n];
    let mut pcd = vec![f32::NAN; n];

    for i in 0..n {
        let p = ctx.p("RES_P", i);
        let t_f = ctx.p("TEMP_F", i);
        let z = ctx.p("Z_FAC", i);
        let vl = ctx.p("VL", i);
        let pl = ctx.p("PL", i);

        // --- Gas FVF: Bg = 0.02827·z·T/P, T in Rankine (Bg→1 at standard conditions) ---
        let mut bg_i = f64::NAN;
        if p.is_finite() && p > 0.0 && z.is_finite() && z > 0.0 && t_f.is_finite() {
            let t_r = t_f + 459.67;
            if t_r > 0.0 {
                bg_i = 0.02827 * z * t_r / p;
                bg[i] = bg_i as f32;
            }
        }

        // --- Adsorbed gas: Langmuir Gs = VL·P/(PL+P), scf/ton (+ cbm ash/moisture) ---
        let mut gs = f64::NAN;
        if p.is_finite() && p >= 0.0 && vl.is_finite() && vl >= 0.0 && pl.is_finite() && (pl + p) > 0.0 {
            gs = vl * p / (pl + p);
            if cbm {
                let fa = ctx.p("F_ASH", i);
                let fm = ctx.p("F_MOIST", i);
                let fa = if fa.is_finite() { fa } else { 0.0 };
                let fm = if fm.is_finite() { fm } else { 0.0 };
                gs *= (1.0 - fa - fm).clamp(0.0, 1.0); // dry-ash-free → in-situ
            }
            gip_ads[i] = gs as f32;
        }

        // --- Free gas: 32.0368·φ·(1−Sw)/(ρb·Bg), scf/ton ---
        let phi = phi_log[i] as f64;
        let sw = sw_log[i] as f64;
        let rb = rhob[i] as f64;
        if bg_i.is_finite()
            && bg_i > 0.0
            && phi.is_finite()
            && (0.0..=1.0).contains(&phi)
            && sw.is_finite()
            && (0.0..=1.0).contains(&sw)
            && rb.is_finite()
            && rb > 0.0
        {
            let gf = 32.0368 * phi * (1.0 - sw) / (rb * bg_i);
            gip_free[i] = gf as f32;
            if gs.is_finite() {
                gip_total[i] = (gf + gs) as f32;
            }
        }

        // --- Critical desorption pressure (cbm): PCD = PL·GC/(VL−GC), only for undersaturated GC ---
        if cbm {
            let gc = ctx.p("GC", i);
            if gc.is_finite() && gc > 0.0 && vl.is_finite() && gc < vl && pl.is_finite() {
                pcd[i] = (pl * gc / (vl - gc)) as f32;
            }
        }
    }

    HashMap::from([
        ("BG".into(), bg),
        ("GIP_ADS".into(), gip_ads),
        ("GIP_FREE".into(), gip_free),
        ("GIP_TOTAL".into(), gip_total),
        ("PCD".into(), pcd),
    ])
}

#[cfg(test)]
mod gip_tests {
    use super::*;

    #[allow(clippy::too_many_arguments)]
    fn ctx(
        mode: &str,
        phi: Vec<f32>,
        sw: Vec<f32>,
        rhob: Vec<f32>,
        res_p: Vec<f64>,
        temp_f: f64,
        z: f64,
        vl: f64,
        pl: f64,
        f_ash: f64,
        f_moist: f64,
        gc: f64,
    ) -> ModuleContext {
        let n = phi.len();
        let mut logs = HashMap::new();
        logs.insert("PHI".to_string(), phi);
        logs.insert("SW".to_string(), sw);
        logs.insert("RHOB".to_string(), rhob);
        let mut params = HashMap::new();
        params.insert("RES_P".to_string(), res_p);
        params.insert("TEMP_F".to_string(), vec![temp_f; n]);
        params.insert("Z_FAC".to_string(), vec![z; n]);
        params.insert("VL".to_string(), vec![vl; n]);
        params.insert("PL".to_string(), vec![pl; n]);
        params.insert("F_ASH".to_string(), vec![f_ash; n]);
        params.insert("F_MOIST".to_string(), vec![f_moist; n]);
        params.insert("GC".to_string(), vec![gc; n]);
        let mut opts = HashMap::new();
        opts.insert("MODE".to_string(), mode.to_string());
        ModuleContext { n, logs, params, opts, depth_unit: Default::default() }
    }

    #[test]
    fn langmuir_isotherm_half_at_pl_and_saturates() {
        // VL=100, PL=1000: Gs(PL)=VL/2=50; Gs(0)=0; Gs(huge)→VL.
        let nan = f32::NAN;
        let c = ctx("shale", vec![nan; 3], vec![nan; 3], vec![nan; 3],
                    vec![1000.0, 0.0, 1e9], 200.0, 0.9, 100.0, 1000.0, 0.0, 0.0, 0.0);
        let out = gip(&c);
        assert!((out["GIP_ADS"][0] as f64 - 50.0).abs() < 1e-3, "Gs(PL)=VL/2=50, got {}", out["GIP_ADS"][0]);
        assert!((out["GIP_ADS"][1] as f64).abs() < 1e-6, "Gs(0)=0, got {}", out["GIP_ADS"][1]);
        assert!((out["GIP_ADS"][2] as f64 - 100.0).abs() < 1e-2, "Gs(inf)->VL=100, got {}", out["GIP_ADS"][2]);
    }

    #[test]
    fn free_gas_matches_volumetric_and_bg() {
        // φ=0.10, Sw=0.30, ρb=2.4, P=3000, T=200°F, z=0.9. Independent HAND literals (guard a typo in
        // the 32.0368 / 0.02827 constants — not re-derived from them): Bg = 0.0055947 rcf/scf; Gf =
        // 167.0 scf/ton (cross-check: 1 ton at 2.4 g/cc = 13.35 bulk-ft³ → HCPV 0.9344 → ÷Bg = 167).
        let c = ctx("shale", vec![0.10], vec![0.30], vec![2.4],
                    vec![3000.0], 200.0, 0.9, 100.0, 1000.0, 0.0, 0.0, 0.0);
        let out = gip(&c);
        assert!((out["BG"][0] as f64 - 0.0055947).abs() < 1e-5, "Bg = {}", out["BG"][0]);
        assert!((out["GIP_FREE"][0] as f64 - 167.0).abs() < 0.15, "GIP_FREE = {}", out["GIP_FREE"][0]);
    }

    #[test]
    fn free_gas_zero_at_full_water_and_rejects_out_of_range() {
        // Sw=1 ⇒ no hydrocarbon pore volume ⇒ Gf=0; out-of-range φ (>1) ⇒ rejected to NaN.
        let sat = gip(&ctx("shale", vec![0.10], vec![1.0], vec![2.4],
                     vec![3000.0], 200.0, 0.9, 100.0, 1000.0, 0.0, 0.0, 0.0));
        assert_eq!(sat["GIP_FREE"][0], 0.0, "Sw=1 ⇒ free gas 0");
        let bad = gip(&ctx("shale", vec![1.5], vec![0.30], vec![2.4],
                     vec![3000.0], 200.0, 0.9, 100.0, 1000.0, 0.0, 0.0, 0.0));
        assert!(bad["GIP_FREE"][0].is_nan(), "φ>1 ⇒ free gas rejected to NaN");
    }

    #[test]
    fn total_is_free_plus_adsorbed() {
        let c = ctx("shale", vec![0.10], vec![0.30], vec![2.4],
                    vec![3000.0], 200.0, 0.9, 100.0, 1000.0, 0.0, 0.0, 0.0);
        let out = gip(&c);
        let sum = out["GIP_FREE"][0] as f64 + out["GIP_ADS"][0] as f64;
        assert!((out["GIP_TOTAL"][0] as f64 - sum).abs() < 1e-3, "total = free + adsorbed");
    }

    #[test]
    fn cbm_ash_moisture_reduces_adsorbed() {
        // cbm F_ASH=0.10, F_MOIST=0.05 ⇒ Gs·0.85 vs shale (no correction).
        let shale = gip(&ctx("shale", vec![f32::NAN], vec![f32::NAN], vec![f32::NAN],
                     vec![1000.0], 200.0, 0.9, 100.0, 1000.0, 0.0, 0.0, 0.0))["GIP_ADS"][0] as f64;
        let cbm = gip(&ctx("cbm", vec![f32::NAN], vec![f32::NAN], vec![f32::NAN],
                     vec![1000.0], 200.0, 0.9, 100.0, 1000.0, 0.10, 0.05, 0.0))["GIP_ADS"][0] as f64;
        assert!((cbm - shale * 0.85).abs() < 1e-3, "cbm Gs = shale·0.85: {cbm} vs {}", shale * 0.85);
    }

    #[test]
    fn cbm_critical_desorption_pressure() {
        // VL=100, PL=1000, GC=50 ⇒ Pcd = 1000·50/(100−50) = 1000 psia.
        let out = gip(&ctx("cbm", vec![f32::NAN], vec![f32::NAN], vec![f32::NAN],
                     vec![3000.0], 200.0, 0.9, 100.0, 1000.0, 0.0, 0.0, 50.0));
        assert!((out["PCD"][0] as f64 - 1000.0).abs() < 1e-2, "Pcd = {}", out["PCD"][0]);
        let out2 = gip(&ctx("cbm", vec![f32::NAN], vec![f32::NAN], vec![f32::NAN],
                     vec![3000.0], 200.0, 0.9, 100.0, 1000.0, 0.0, 0.0, 25.0));
        assert!((out2["PCD"][0] as f64 - 1000.0 * 25.0 / 75.0).abs() < 1e-2, "Pcd = {}", out2["PCD"][0]);
        let out3 = gip(&ctx("shale", vec![f32::NAN], vec![f32::NAN], vec![f32::NAN],
                     vec![3000.0], 200.0, 0.9, 100.0, 1000.0, 0.0, 0.0, 50.0));
        assert!(out3["PCD"][0].is_nan(), "shale mode emits no Pcd");
    }

    #[test]
    fn missing_porosity_leaves_free_nan_but_adsorbed_runs() {
        let out = gip(&ctx("shale", vec![f32::NAN], vec![0.30], vec![2.4],
                     vec![3000.0], 200.0, 0.9, 100.0, 1000.0, 0.0, 0.0, 0.0));
        assert!(out["GIP_FREE"][0].is_nan(), "no φ ⇒ free gas missing");
        assert!(out["GIP_ADS"][0].is_finite(), "adsorbed runs from pressure alone");
        assert!(out["GIP_TOTAL"][0].is_nan(), "no free ⇒ total missing");
    }
}

// ---------------------------------------------------------------------------
// Brittleness — elastic (Rickman) + mineralogical (Jarvie / Wang-Gale)
// (docs/ref_unconventional.md §4)
// ---------------------------------------------------------------------------

pub fn brittleness_spec() -> ModuleSpec {
    ModuleSpec {
        name: "brittleness".into(),
        title: "Brittleness index (elastic / mineralogical)".into(),
        category: "Unconventional".into(),
        doc: "Brittleness index (0 ductile .. 1 brittle) two ways. METHOD=elastic: dynamic Young's \
              modulus and Poisson's ratio from DT, DTS, RHOB (G=ρ·Vs², K=ρ·(Vp²−4/3·Vs²), \
              ν=(3K−2G)/(2(3K+G)), E=9KG/(3K+G), with Vp,Vs in km/s = 304.8/slowness, moduli in GPa, \
              E→Mpsi), then Rickman et al. 2008 BI=(E_norm+ν_norm)/2 with E normalized over E_LO..E_HI \
              Mpsi and ν over NU_LO..NU_HI (Barnett defaults 1..8 and 0.4..0.15 — recalibrate per \
              basin). METHOD=mineral_jarvie: Jarvie 2007 BI=Qz/(Qz+carbonate+clay). \
              METHOD=mineral_wanggale: Wang & Gale 2009 BI=(Qz+Dol)/(Qz+Dol+calcite+clay+organic) — \
              dolomite counts brittle. Mineral volumes come from a SandiMin run (VOL_*); a missing \
              mineral is treated as absent. Elastic E,ν are DYNAMIC (apply a static correlation before \
              geomechanics). Cite: Rickman et al. 2008 (SPE 115258); Jarvie et al. 2007; Wang & Gale \
              2009. See docs/ref_unconventional.md §4."
            .into(),
        args: vec![
            opt("METHOD", "Brittleness basis", "elastic",
                &["elastic", "mineral_jarvie", "mineral_wanggale"]),
            param("E_LO", "Young's modulus at BI=0 (ductile)", "Mpsi", 1.0, 0.0, 20.0),
            param("E_HI", "Young's modulus at BI=1 (brittle)", "Mpsi", 8.0, 0.0, 20.0),
            param("NU_LO", "Poisson's ratio at BI=0 (ductile)", "-", 0.4, 0.0, 0.5),
            param("NU_HI", "Poisson's ratio at BI=1 (brittle)", "-", 0.15, 0.0, 0.5),
            log_in("DT", "Compressional Δt (elastic)", "us/ft", "DT", false),
            log_in("DTS", "Shear Δt (elastic)", "us/ft", "DTS", false),
            log_in("RHOB", "Bulk density (elastic)", "g/cc", "RHOB", false),
            log_in("VQTZ", "Quartz volume (mineral)", "v/v", "VOL_QUARTZ", false),
            log_in("VCARB", "Calcite volume (mineral)", "v/v", "VOL_CALCITE", false),
            log_in("VDOL", "Dolomite volume (mineral)", "v/v", "VOL_DOLOMITE", false),
            log_in("VCLAY", "Clay / shale volume (mineral)", "v/v", "VSH", false),
            log_in("VORG", "Organic / kerogen volume (Wang-Gale)", "v/v", "VKER", false),
            log_out("BI", "Brittleness index (0 ductile .. 1 brittle)", "-"),
            log_out("YME", "Dynamic Young's modulus (elastic)", "Mpsi"),
            log_out("PR", "Dynamic Poisson's ratio (elastic)", "-"),
        ],
    }
}

pub fn brittleness(ctx: &ModuleContext) -> ModuleOutputs {
    let dt = ctx.log("DT");
    let dts = ctx.log("DTS");
    let rhob = ctx.log("RHOB");
    let vqtz = ctx.log("VQTZ");
    let vcarb = ctx.log("VCARB");
    let vdol = ctx.log("VDOL");
    let vclay = ctx.log("VCLAY");
    let vorg = ctx.log("VORG");
    let method = ctx.o("METHOD").to_string();
    let n = ctx.n;

    let mut bi = vec![f32::NAN; n];
    let mut yme = vec![f32::NAN; n];
    let mut pr = vec![f32::NAN; n];

    for i in 0..n {
        if method == "elastic" {
            // Dynamic elastic moduli from slowness (Rock Physics Handbook / Techlog RockPhyEquations).
            let d = dt[i] as f64;
            let ds = dts[i] as f64;
            let rb = rhob[i] as f64;
            let e_lo = ctx.p("E_LO", i);
            let e_hi = ctx.p("E_HI", i);
            let nu_lo = ctx.p("NU_LO", i);
            let nu_hi = ctx.p("NU_HI", i);
            if d.is_finite() && d > 0.0
                && ds.is_finite() && ds > 0.0
                && rb.is_finite() && rb > 0.0
                && ds > d // shear slower than compressional ⇒ Vs < Vp
                && (e_hi - e_lo).abs() > 1e-9
                && (nu_hi - nu_lo).abs() > 1e-9
            {
                let vp = 304.8 / d; // km/s
                let vs = 304.8 / ds; // km/s
                let g = rb * vs * vs; // GPa  (ρ[g/cc]·V²[(km/s)²] = GPa)
                let k = rb * (vp * vp - 4.0 / 3.0 * vs * vs); // GPa
                if k > 0.0 && (3.0 * k + g) > 0.0 {
                    let nu = (3.0 * k - 2.0 * g) / (2.0 * (3.0 * k + g));
                    let e_gpa = 9.0 * k * g / (3.0 * k + g);
                    let e_mpsi = e_gpa * 0.145038; // GPa → Mpsi
                    // ν<0 (Vp/Vs < √2) is auxetic — nonphysical for sedimentary rock, so it flags a
                    // bad/spiky shear log. Reject rather than emit a negative PR and a falsely
                    // max-brittle (clamped-to-1) BI.
                    if nu >= 0.0 && e_mpsi > 0.0 {
                        yme[i] = e_mpsi as f32;
                        pr[i] = nu as f32;
                        // Rickman 2008: normalize E (brittle=high) and ν (brittle=low), average.
                        let e_norm = (e_mpsi - e_lo) / (e_hi - e_lo);
                        let nu_norm = (nu - nu_lo) / (nu_hi - nu_lo);
                        bi[i] = (0.5 * (e_norm + nu_norm)).clamp(0.0, 1.0) as f32;
                    }
                }
            }
        } else {
            // Mineralogical — a missing/NaN volume means the mineral is absent (0).
            let f = |x: f32| {
                let v = x as f64;
                if v.is_finite() { v.max(0.0) } else { 0.0 }
            };
            let qz = f(vqtz[i]);
            let cc = f(vcarb[i]);
            let dol = f(vdol[i]);
            let clay = f(vclay[i]);
            let org = f(vorg[i]);
            let (num, den) = if method == "mineral_wanggale" {
                // Wang & Gale 2009: dolomite brittle; calcite + clay + organic ductile.
                (qz + dol, qz + dol + cc + clay + org)
            } else {
                // Jarvie 2007: quartz brittle; all carbonate + clay ductile.
                (qz, qz + cc + dol + clay)
            };
            if den > 0.0 {
                bi[i] = (num / den).clamp(0.0, 1.0) as f32;
            }
        }
    }

    HashMap::from([("BI".into(), bi), ("YME".into(), yme), ("PR".into(), pr)])
}

#[cfg(test)]
mod brittleness_tests {
    use super::*;

    fn elastic_ctx(dt: f32, dts: f32, rhob: f32) -> ModuleContext {
        let mut logs = HashMap::new();
        logs.insert("DT".to_string(), vec![dt]);
        logs.insert("DTS".to_string(), vec![dts]);
        logs.insert("RHOB".to_string(), vec![rhob]);
        let mut params = HashMap::new();
        params.insert("E_LO".to_string(), vec![1.0]);
        params.insert("E_HI".to_string(), vec![8.0]);
        params.insert("NU_LO".to_string(), vec![0.4]);
        params.insert("NU_HI".to_string(), vec![0.15]);
        let mut opts = HashMap::new();
        opts.insert("METHOD".to_string(), "elastic".to_string());
        ModuleContext { n: 1, logs, params, opts, depth_unit: Default::default() }
    }

    fn mineral_ctx(method: &str, qz: f32, cc: f32, dol: f32, clay: f32, org: f32) -> ModuleContext {
        let mut logs = HashMap::new();
        logs.insert("VQTZ".to_string(), vec![qz]);
        logs.insert("VCARB".to_string(), vec![cc]);
        logs.insert("VDOL".to_string(), vec![dol]);
        logs.insert("VCLAY".to_string(), vec![clay]);
        logs.insert("VORG".to_string(), vec![org]);
        let mut opts = HashMap::new();
        opts.insert("METHOD".to_string(), method.to_string());
        ModuleContext { n: 1, logs, params: HashMap::new(), opts, depth_unit: Default::default() }
    }

    #[test]
    fn elastic_bi_from_known_slowness() {
        // DT=100, DTS=170, RHOB=2.5 → Vp=3.048, Vs=1.7929 km/s → E≈2.880 Mpsi, ν≈0.2354, BI≈0.4634.
        let out = brittleness(&elastic_ctx(100.0, 170.0, 2.5));
        assert!((out["YME"][0] as f64 - 2.880).abs() < 0.01, "YME = {}", out["YME"][0]);
        assert!((out["PR"][0] as f64 - 0.2354).abs() < 0.002, "PR = {}", out["PR"][0]);
        assert!((out["BI"][0] as f64 - 0.4634).abs() < 0.003, "BI = {}", out["BI"][0]);
    }

    #[test]
    fn elastic_requires_valid_shear() {
        // Missing DTS → NaN; Vs>Vp (DTS<DT, unphysical) → NaN.
        let no_shear = brittleness(&elastic_ctx(100.0, f32::NAN, 2.5));
        assert!(no_shear["BI"][0].is_nan() && no_shear["YME"][0].is_nan(), "no DTS ⇒ NaN");
        let bad = brittleness(&elastic_ctx(170.0, 100.0, 2.5)); // DTS<DT ⇒ Vs>Vp
        assert!(bad["BI"][0].is_nan(), "Vs>Vp ⇒ rejected");
    }

    #[test]
    fn elastic_rejects_negative_poisson() {
        // Vp/Vs ∈ (1.155, 1.414): K>0 but ν<0 (auxetic — bad shear log). DT=100, DTS=130 → Vp/Vs=1.30,
        // ν≈−0.22. Must NaN all three, not emit a negative PR and a clamped BI=1.
        let out = brittleness(&elastic_ctx(100.0, 130.0, 2.5));
        assert!(out["PR"][0].is_nan() && out["BI"][0].is_nan() && out["YME"][0].is_nan(),
                "negative-ν (bad shear) rejected, got PR={} BI={}", out["PR"][0], out["BI"][0]);
    }

    #[test]
    fn jarvie_bi_from_mineralogy() {
        // Qz 0.6, carbonate 0.1, clay 0.3 → 0.6/(0.6+0.1+0.3) = 0.6.
        let out = brittleness(&mineral_ctx("mineral_jarvie", 0.6, 0.1, 0.0, 0.3, 0.0));
        assert!((out["BI"][0] as f64 - 0.6).abs() < 1e-5, "jarvie BI = {}", out["BI"][0]);
    }

    #[test]
    fn wanggale_moves_dolomite_to_brittle() {
        // Qz 0.5, Dol 0.2, Cc 0.1, Clay 0.2: jarvie 0.5/1.0=0.5; wang-gale (0.5+0.2)/1.0=0.7.
        let j = brittleness(&mineral_ctx("mineral_jarvie", 0.5, 0.1, 0.2, 0.2, 0.0))["BI"][0] as f64;
        let w = brittleness(&mineral_ctx("mineral_wanggale", 0.5, 0.1, 0.2, 0.2, 0.0))["BI"][0] as f64;
        assert!((j - 0.5).abs() < 1e-5, "jarvie = {j}");
        assert!((w - 0.7).abs() < 1e-5, "wang-gale = {w}");
        assert!(w > j, "dolomite counts brittle in Wang-Gale");
    }

    #[test]
    fn bi_monotone_in_quartz() {
        // Prompt-required: BI rises with quartz fraction (Jarvie, others fixed).
        let lo = brittleness(&mineral_ctx("mineral_jarvie", 0.4, 0.1, 0.0, 0.5, 0.0))["BI"][0];
        let hi = brittleness(&mineral_ctx("mineral_jarvie", 0.7, 0.1, 0.0, 0.5, 0.0))["BI"][0];
        assert!(hi > lo, "BI monotone in quartz: {lo} -> {hi}");
    }

    #[test]
    fn mineral_all_absent_is_nan() {
        // All mineral volumes missing → denominator 0 → NaN (no spurious BI).
        let out = brittleness(&mineral_ctx("mineral_jarvie", f32::NAN, f32::NAN, f32::NAN, f32::NAN, f32::NAN));
        assert!(out["BI"][0].is_nan(), "no minerals ⇒ NaN");
    }
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
        ModuleContext { n, logs, params, opts, depth_unit: Default::default() }
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
