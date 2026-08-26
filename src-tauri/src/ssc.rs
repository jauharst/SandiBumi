//! Sand-Silt-Clay (SSC) and Sandstone Petrophysical Workflow (SSPW) modules.
//!
//! SSC is a faithful port of Jauhar's Loglan `ssc_lqr_gap_edit_jau.lls`
//! ("Modifikasi Metoda Sand Silt Clay untuk Perhitungan Zona LQR", GAP 2023, after
//! Kuttan et al., "Log Interpretation in the Malay Basin", 21st SPWLA). Designed for
//! very fine grained sediments with fresh–brackish formation water, where classic
//! shaly-sand analysis overestimates clay and underestimates porosity.
//!
//! SSPW is a three-component sandstone workflow (quartz + shale + water, density
//! porosity with a dry-shale matrix). Its key message: PHIE = PHIT − clay-bound
//! water only; capillary-bound water stays inside PHIE, and PHIFF = PHIT − CBW −
//! CAPBW is what can actually flow. The Loglan exec body is not on disk, so the
//! arithmetic here is reconstructed from the module spec (`porosity_sspw.lls`) and
//! the same physics the SSC source spells out — verify against the reference LAS output.
//!
//! Deviations from the Loglan, all deliberate:
//! - `RANNORMAL(SWIRR_MIN*PHIT, 0.005)` becomes deterministic `SWIRR_MIN*PHIT`.
//! - NPHIMA is limited to [0,1] (the Loglan's 0.5–5 limit is a copy-paste of the
//!   RHOMA limit and would clamp every neutron matrix value up to 0.5).
//! - Gas/HC conditioning is a DECLARED PARAMETER, `GAS_C`, not a constant (DEC-086).
//!   The two source files disagree — the 2022 spec-only `porosity_sspw.lls` writes
//!   c = 1.6, the 2025 exec body `sspw.lls` writes the even split c = 1 — and the
//!   disagreement is real petrophysics rather than a transcription error to
//!   adjudicate, so it ships as the user's dial. Jauhar ruled 1.6 for SSPW on the
//!   field observation that the even split still reads optimistic in his rock
//!   (DEC-086), then extended the same experience to SSC (DEC-088), so BOTH now
//!   default to 1.6. **DEC-088 moved SSC's gas numbers**, which was the ruling and
//!   not a side effect: SSC ran the even split from 2026-07-29 until then.
//!
//! Reference-inherited quirks (line-verified against the .lls, 2026-07-29) — kept
//! for parity, do NOT "fix" without changing the Loglan too:
//! - PHIFF_GR: when PHIFF ≤ 0.005 the Loglan sets PHIFF_GR = 1−VSHGR (the whole
//!   sand-side volume as free-fluid porosity) and leaves VSAND_GR unset (NaN here).
//! - CWSH algebra reduces to VSH·PHIT_SH/(1−PHIT_SH): VWSH = VSH/(1−PHIT_SH)
//!   re-wets the already-wet VWCL, so BW double-counts clay-bound water.
//! - VSHND uses the fraction-mixed (RHOMA, NPHIMA) as its matrix endpoint — near-
//!   degenerate with the projection by construction — and is unclamped (.lls 181).
//! - Branch-1 fraction split omits the −NPHI_MA offset (exact only at NPHI_MA = 0,
//!   discontinuous at the silt point otherwise); matrix mix treats the dry-silt
//!   point as clay-free although it sits at DCLF_SI clay (.lls 338-340).
//! - SWIRR_T/SWIRR_EFF are the pre-conditioning pair (.lls 213-216).
//!
//! SSPW divergence (found 2026-07-29): a full exec body DOES exist on disk —
//! `sspw.lls` (2025-02-28, two-branch N-D PHIT solve using NPHI_SH/NPHI_MA/NPHI_FL)
//! next to the spec-only `porosity_sspw.lls` (2022) this port was reconstructed
//! from. Re-porting sspw() against it is pending Jauhar's sign-off; until then the
//! declared NPHI_* parameters here are read by the UI but unused by the math.
//! The GAS COEFFICIENT half of that divergence is SETTLED under DEC-086 and is not
//! part of the pending re-port: whichever PHIT solve wins, `GAS_C` stays the dial.

use crate::modules::{
    log_in, log_out, opt, param, param_open, ModuleContext, ModuleOutputs, ModuleSpec,
};
use std::collections::HashMap;

fn limit(v: f64, lo: f64, hi: f64) -> f64 {
    // Mirror modules::limit — `f64::clamp` panics on a NaN bound (release builds are
    // panic = "abort"), and `hi` can be NaN-poisoned here (e.g. `phit - cbw` downstream
    // of an unset parameter).
    if v.is_nan() || !(lo <= hi) {
        f64::NAN
    } else {
        v.clamp(lo, hi)
    }
}

fn vsh_from_gr(method: &str, mut v: f64) -> f64 {
    match method {
        "STIEBER1" => { v = limit(v, -10.0, 1.49); v / (3.0 - 2.0 * v) }
        "STIEBER2" => { v = limit(v, -10.0, 1.99); v / (2.0 - v) }
        "STIEBER3" => { v = limit(v, -10.0, 1.33); v / (4.0 - 3.0 * v) }
        // SB-CLY-004 / DEC-096: the EXACT normalised Larionov, `(2^(k*I) - 1)/(2^k - 1)`,
        // which closes at exactly 1.0 where the published decimals below fall 1.00% and
        // 0.43% short. Separate IDS rather than changed arithmetic: the id is what
        // `params_json` stores, so re-pointing it would move every saved run in silence.
        "LARINOV1_NORM" => (2.0_f64.powf(2.0 * v) - 1.0) / (2.0_f64.powf(2.0) - 1.0),
        "LARINOV2_NORM" => (2.0_f64.powf(3.7 * v) - 1.0) / (2.0_f64.powf(3.7) - 1.0),
        // SB-CLY-005: the vendors' published decimals, kept reachable for digit-for-digit
        // parity against an existing curve. They do NOT reach 1.0 at IGR = 1.
        "LARINOV1" => 0.33 * (2.0_f64.powf(2.0 * v) - 1.0),
        "LARINOV2" => 0.083 * (2.0_f64.powf(3.7 * v) - 1.0),
        "LARINOV3" => 0.127 * (3.15_f64.powf(2.0 * v) - 1.0),
        "CLAVIER" => { v = limit(v, -2.53, 1.13); 1.7 - (3.38 - (v + 0.7).powi(2)).sqrt() }
        _ => v, // LINEAR
    }
}

// ---------------------------------------------------------------------------
// SSC — Sand-Silt-Clay model (Kuttan / GAP 2023, LQR edit)
// ---------------------------------------------------------------------------

pub fn ssc_spec() -> ModuleSpec {
    ModuleSpec {
        name: "ssc".into(),
        title: "SSC — Sand-Silt-Clay (Kuttan)".into(),
        category: "Porosity".into(),
        doc: "Sand-Silt-Clay model on the N-D crossplot (Kuttan Malay Basin, SandiBumi \
              edit). Data points are projected from the fluid point onto the dry rock line \
              (matrix→dry clay); sand/silt/clay fractions come from the projection position, \
              matrix density from the fraction mix, PHIT from density. Bound water is split \
              into clay-bound (CBW) and capillary-bound in silt/shale (CWSH): PHIE = PHIT − \
              VWCL·PHIT_CL, PHIFF = PHIT − CBW − CWSH, SWIRR_T = BW/PHIT. GR-equivalent \
              volumes rescale the SSC volumes to honour VSHGR. Study-specific crossplot endpoints \
              ship absent and must be supplied from the active interpretation."
            .into(),
        args: vec![
            opt(
                "OPT_VSHGR",
                "VSH from gamma ray method",
                "LINEAR",
                &["LINEAR", "STIEBER1", "STIEBER2", "STIEBER3", "LARINOV1", "LARINOV2", "LARINOV3", "CLAVIER"],
            ),
            param_open("GR_MA", "Gamma ray matrix (clean)", "gapi", 0.0, 100.0, true),
            param_open("GR_SH", "Gamma ray clay", "gapi", 0.0, 1000.0, true),
            param(
                "RHOB_MA", "Density matrix", "g/cc", 2.65, 1.0, 4.0,
                "IP/Techlog/SandiMin sandstone matrix endpoint 2.65 g/cm3; docs/PRD_v2/11_porosity.md §5.1",
            ),
            param_open("NPHI_MA", "Neutron matrix", "v/v", -0.1, 1.2, true),
            param(
                "RHOB_FL", "Density fluid", "g/cc", 1.0, 0.5, 4.0,
                "IP basicloganalysis.htm fresh-water 1.0 gm/cc; Geolog phi_den.info RHO_FL 1000 k/m3; docs/PRD_v2/11_porosity.md §5.1",
            ),
            param(
                "NPHI_FL", "Neutron fluid", "v/v", 1.0, -0.1, 1.2,
                "Geolog V14 vsh_dn.info and Techlog VSH neutron-density NPHI fluid 1.0; docs/PRD_v2/10_clay-volume.md §5",
            ),
            param_open("RHOB_WCL", "Bulk density wet clay", "g/cc", 1.0, 4.0, true),
            param_open("NPHI_WCL", "Neutron porosity wet clay", "v/v", -0.1, 1.2, true),
            param_open("RHOB_DCL", "Bulk density dry clay", "g/cc", 1.0, 4.0, true),
            param_open("NPHI_WSI", "Neutron porosity wet silt", "v/v", -0.1, 1.2, true),
            param_open("DCLF_SI", "Dry clay fraction at dry silt", "v/v", 0.0, 1.0, true),
            param_open("PHIT_CL", "Total porosity of clay", "v/v", 0.0, 0.8, true),
            param_open("SWIRR_MIN", "Minimum total irreducible Sw", "v/v", 0.0, 1.0, true),
            param(
                "PHIT_TIGHT",
                "Total porosity below which all non-clay-bound porosity is capillary-held",
                "v/v", 0.05, 0.0, 0.5,
                "SandiBumi's own SSC conditioning rule, not the Loglan's: added to keep CWSH positive and reliable, since CWSH always exists even where small; KEPT and parameterised under DEC-093 (2026-08-22). 0.05 is the value the port has run since it was written; it is a parameter now so a tight carbonate stringer and a shaly sand need not share it",
            ),
            param(
                "GAS_C", "Gas-conditioning weight (0 = density only, 1 = even, 2 = neutron only)",
                "", 1.6, 0.0, 2.0,
                "sspw.lls (2025-02-28) gas branch writes the even split, PHIT = ((phiD^2 + NPHI^2)/2)^0.5, i.e. c = 1 - and that is what SSC ran until DEC-088 OVERRODE it, ruling 1.6 here too and extending DEC-086's field observation that the even split still reads optimistic. The source is unchanged; the shipped default departs from it deliberately",
            ),
            log_in("GR", "Gamma ray (normalized)", "gapi", "GRN", true),
            log_in("RHOB", "Bulk density (corrected)", "g/cc", "RHOB", true),
            log_in("NPHI", "Neutron porosity (sandstone units)", "v/v", "NPHI", true),
            log_out("VSAND", "Dry sand volume (bulk)", "v/v"),
            log_out("VSILT", "Dry silt volume (bulk)", "v/v"),
            log_out("VDCL", "Dry clay volume (bulk)", "v/v"),
            log_out("VWCL", "Wet clay volume", "v/v"),
            log_out("VSH_SSC", "Vshale equivalent (VWCL + VSILT)", "v/v"),
            log_out("VSHGR", "VSH from gamma ray", "v/v"),
            log_out("VSHND", "VSH from density-neutron", "v/v"),
            log_out("PHIT_SSC", "Total porosity", "v/v"),
            log_out("PHIE_SSC", "Effective porosity (PHIT − CBW)", "v/v"),
            log_out("PHIFF_SSC", "Free fluid porosity (PHIT − CBW − CWSH)", "v/v"),
            log_out("CBW", "Clay-bound water", "v/v"),
            log_out("CWSH", "Capillary-bound water in silt/shale", "v/v"),
            log_out("BW", "Total bound water", "v/v"),
            log_out("SWIRR_T", "Total irreducible water saturation", "v/v"),
            log_out("SWIRR_EFF", "Effective irreducible water saturation", "v/v"),
            log_out("VSAND_GR", "Sand volume, GR-equivalent", "v/v"),
            log_out("VSILT_GR", "Silt volume, GR-equivalent", "v/v"),
            log_out("VDCL_GR", "Dry clay volume, GR-equivalent", "v/v"),
            log_out("CBW_GR", "Clay-bound water, GR-equivalent", "v/v"),
            log_out("CWSH_GR", "Capillary water, GR-equivalent", "v/v"),
            log_out("PHIFF_GR", "Free fluid, GR-equivalent", "v/v"),
            log_out("PHIE_GR", "Effective porosity, GR-equivalent", "v/v"),
            log_out("PHIT_GR", "Total porosity, GR-equivalent", "v/v"),
        ],
    }
}

pub fn ssc(ctx: &ModuleContext) -> ModuleOutputs {
    let gr = ctx.log("GR");
    let rhob = ctx.log("RHOB");
    let nphi = ctx.log("NPHI");
    let method = ctx.o("OPT_VSHGR").to_string();

    let names = [
        "VSAND", "VSILT", "VDCL", "VWCL", "VSH_SSC", "VSHGR", "VSHND", "PHIT_SSC", "PHIE_SSC",
        "PHIFF_SSC", "CBW", "CWSH", "BW", "SWIRR_T", "SWIRR_EFF", "VSAND_GR", "VSILT_GR",
        "VDCL_GR", "CBW_GR", "CWSH_GR", "PHIFF_GR", "PHIE_GR", "PHIT_GR",
    ];
    let mut out: HashMap<String, Vec<f32>> =
        names.iter().map(|n| (n.to_string(), vec![f32::NAN; ctx.n])).collect();

    for i in 0..ctx.n {
        let (g, r, np) = (gr[i] as f64, rhob[i] as f64, nphi[i] as f64);
        let gr_ma = ctx.p("GR_MA", i);
        let gr_sh = ctx.p("GR_SH", i);
        let rhob_ma = ctx.p("RHOB_MA", i);
        let nphi_ma = ctx.p("NPHI_MA", i);
        let rhob_fl = ctx.p("RHOB_FL", i);
        let nphi_fl = ctx.p("NPHI_FL", i);
        let rhob_wcl = ctx.p("RHOB_WCL", i);
        let nphi_wcl = ctx.p("NPHI_WCL", i);
        let rhob_dcl = ctx.p("RHOB_DCL", i);
        let nphi_wsi = ctx.p("NPHI_WSI", i);
        let dclf_si = ctx.p("DCLF_SI", i);
        let phit_cl = ctx.p("PHIT_CL", i);
        let swirr_min = ctx.p("SWIRR_MIN", i);
        let gas_c = ctx.p("GAS_C", i);
        // GAS_C joins the guard rather than being allowed to propagate: `f64::NAN.max(0.0)`
        // returns 0.0, so a NaN weight would silently make the corrected porosity ZERO on
        // every gas sample instead of failing visibly.
        if r.is_nan() || np.is_nan() || rhob_ma.is_nan() || rhob_fl.is_nan() || gas_c.is_nan() {
            continue;
        }

        // Gas/HC conditioning: pull a point above the sand base line back onto it.
        //
        // ONE equation with a dial (DEC-086). Writing Δ = |φDI² − φN²|, the corrected pair is
        // φD² = φDI² − c·Δ/2 and φN² = φN² + c·Δ/2, so c weights where the answer lands:
        // c = 0 keeps the density untouched, c = 1 is the even split — the RMS midpoint
        // sqrt((φDI²+φN²)/2) that both legs share, matching sspw.lls's gas branch
        // (`PHIT = (((dphi_volan)**2+(NPHI)**2)/2)**(0.5)`) — and c = 2 hands the answer to
        // the neutron outright. Above c = 1 the two corrected legs CROSS: at 1.6 the corrected
        // density is 0.2·φD²+0.8·φN² and the corrected neutron its mirror, so the D-N crossover
        // is not closed but reversed. BOTH modules default to 1.6 — SSPW under DEC-086 and SSC
        // under DEC-088, on Jauhar's own field observation that the even split still reads
        // optimistic. Neither is hard-coded any more — the rock decides, per well and per zone.
        //
        // NPHI enters squared, so a negative neutron loses its sign here — the Loglan squares
        // it identically.
        let phidi = (rhob_ma - r) / (rhob_ma - rhob_fl);
        let gas_pull = np <= 1.05 * phidi;
        // SB-POR-003: the gas branch is a per-sample identity for the run's custody comment.
        crate::modules::record_branch(if gas_pull {
            "gas-conditioned (pulled onto the sand base line)"
        } else {
            "no gas conditioning"
        });
        let (rhob_cor, nphi_cor) = if gas_pull {
            let (d2, n2) = (phidi * phidi, np * np);
            let pull = gas_c * (d2 - n2).abs() / 2.0;
            let phid = (d2 - pull).max(0.0).sqrt();
            let nphi_c = (n2 + pull).max(0.0).sqrt();
            (rhob_ma - (rhob_ma - rhob_fl) * phid, nphi_c)
        } else {
            (r, np)
        };

        // --- SSC framework lines (all y = RHOB over x = NPHI). The clay-water line is
        // anchored at (1,1) exactly as the Loglan writes it (literal 1s, not the fluid params).
        let m1 = (1.0 - rhob_wcl) / (1.0 - nphi_wcl);
        let c1 = rhob_wcl - m1 * nphi_wcl;
        let nphi_dcl = (rhob_dcl - c1) / m1;

        let m2 = (rhob_wcl - rhob_ma) / (nphi_wcl - nphi_ma);
        let c2 = rhob_wcl - m2 * nphi_wcl;
        let rhob_wsi = m2 * nphi_wsi + c2;

        let m3 = (rhob_fl - rhob_wsi) / (nphi_fl - nphi_wsi);
        let c3 = rhob_fl - m3 * nphi_fl;
        let m4 = (rhob_dcl - rhob_ma) / (nphi_dcl - nphi_ma);
        let c4 = rhob_ma - m4 * nphi_ma;
        let nphi_dsi = (c4 - c3) / (m3 - m4);
        let rhob_dsi = m3 * nphi_dsi + c3;

        // Project the data point from the fluid point onto the dry rock line.
        let m5 = (rhob_fl - rhob_cor) / (nphi_fl - nphi_cor);
        let c5 = rhob_fl - m5 * nphi_fl;
        let nphi_proj = (c4 - c5) / (m5 - m4);

        // Sand-silt-clay fractions from the projection position.
        // SB-POR-003: which side of the dry-silt point the projection landed on - the two
        // sides are different fraction systems, not one equation with a different constant.
        crate::modules::record_branch(if nphi_proj < nphi_dsi {
            "left of the dry-silt point (clay-sand-silt)"
        } else {
            "right of the dry-silt point (clay-silt)"
        });
        let (dclf, dsaf, dsif) = if nphi_proj < nphi_dsi {
            let m6 = dclf_si / (nphi_dsi - nphi_ma);
            let m7 = (1.0 - dclf_si) / (nphi_dsi - nphi_ma);
            let dclf = limit(m6 * nphi_proj, 0.0, 1.0);
            let dsaf = limit(-m7 * nphi_proj + 1.0 - dclf, 0.0, 1.0);
            (dclf, dsaf, limit(1.0 - dclf - dsaf, 0.0, 1.0))
        } else {
            let m6 = (1.0 - dclf_si) / (nphi_dcl - nphi_dsi);
            let c6 = 1.0 - m6 * nphi_dcl;
            let dclf = limit(m6 * nphi_proj + c6, 0.0, 1.0);
            (dclf, 0.0, limit(1.0 - dclf, 0.0, 1.0))
        };

        // Total porosity from the fraction-mixed matrix density.
        let rhoma = limit(dsaf * rhob_ma + dsif * rhob_dsi + dclf * rhob_dcl, 0.5, 5.0);
        let nphima = limit(dsaf * nphi_ma + dsif * nphi_dsi + dclf * nphi_dcl, 0.0, 1.0);
        let phit_raw = (rhoma - rhob_cor) / (rhoma - rhob_fl);
        let phit = limit(phit_raw, 0.001, 0.75);
        if !phit_raw.is_nan() && phit != phit_raw {
            crate::modules::record_bound_limit("PHIT");
        }

        // Shale volumes from GR and N-D for the GR-equivalent rescaling.
        let vshgr = if g.is_nan() || gr_ma >= gr_sh {
            f64::NAN
        } else {
            limit(vsh_from_gr(&method, (g - gr_ma) / (gr_sh - gr_ma)), 0.0, 1.0)
        };
        let vshnd = ((rhob_fl - rhoma) * (nphi_cor - nphima)
            - (rhob_cor - rhoma) * (nphi_fl - nphima))
            / ((rhob_fl - rhoma) * (nphi_wcl - nphima) - (rhob_wcl - rhoma) * (nphi_fl - nphima));

        // Bulk volumes.
        let vdcl = limit(dclf * (1.0 - phit), 0.0, 1.0);
        let vsand = limit(dsaf * (1.0 - phit), 0.0, 1.0);
        let vsilt = limit(dsif * (1.0 - phit), 0.0, 1.0);

        // Effective porosity and the bound-water split.
        let vwcl = vdcl / (1.0 - phit_cl);
        let vsh = vwcl + vsilt;
        let phie_raw = phit - vwcl * phit_cl;
        let phie = limit(phie_raw, 0.0, phit);
        if !phie_raw.is_nan() && phie != phie_raw {
            crate::modules::record_bound_limit("PHIE");
        }
        let cbw = phit - phie;
        // SB-POR-008 / F16: this is NOT the clay-bound-water PHIT_SH and must not carry that name.
        // `rhob_dsi` is the intersection of the fluid-anchored line m3 with the dry-clay line m4,
        // so this is the wet-silt point's fractional distance along m3 from dry silt toward the
        // fluid point. Its denominator must stay `rhob_fl`: change it and the expression stops
        // being a fraction along the very line that defined its numerator. Arithmetic unchanged.
        let silt_water_fraction = (rhob_dsi - rhob_wsi) / (rhob_dsi - rhob_fl);
        let vwsh = limit(vsh / (1.0 - silt_water_fraction), 0.0, 1.0);
        let mut cwsh = vwsh - vdcl - cbw - vsilt;
        let mut bw = cbw + cwsh;

        let swirr_t = limit(bw / phit, 0.0, 1.0);
        // Guard the /phie divide: at the wet-clay point phie is floored to 0, where the
        // original expression gives -inf->0 ("all water movable") or 0/0->NaN. A zero-
        // effective-porosity sample is fully bound, so report 1.0. (Only degenerate
        // phie==0 samples change; every phie>0 result is unchanged.)
        // A NaN PHIE must stay NaN (missing-data contract) — `NaN > 0.0` is false, so
        // without the explicit branch a missing sample would fall into the else and
        // read as fully bound (1.0) instead of absent.
        let swirr_eff = if phie.is_nan() {
            f64::NAN
        } else if phie > 0.0 {
            limit(1.0 - phit * (1.0 - swirr_t) / phie, 0.0, 1.0)
        } else {
            1.0
        };

        // Capillary bound water conditioning (Loglan order preserved; the RANNORMAL
        // draw is replaced with its deterministic mean).
        if phie <= 0.002 {
            cwsh = phit - cbw;
        }
        // DEC-093 rule 2. `phie - cwsh` IS PHIFF (because PHIE = PHIT - CBW), so this clamps
        // free-fluid porosity at zero instead of letting the SSC triangle drive it negative -
        // Jauhar's "avoid minus". The dead band is `PHIE_FLOOR`, the smallest porosity this
        // application treats as real anywhere, rather than a second bare literal meaning the
        // same thing. `bw = phit` is the same value the recompute below reaches; kept because
        // the Loglan order is preserved, harmless either way.
        if phie - cwsh <= crate::modules::PHIE_FLOOR {
            cwsh = phie;
            bw = phit;
        }
        if !swirr_min.is_nan() && bw / phit < swirr_min {
            cwsh = swirr_min * phit;
            if cbw > 0.0 {
                cwsh -= cbw;
            }
        }
        cwsh = limit(cwsh, 0.0, phit);
        // DEC-093 rule 4. Below PHIT_TIGHT every non-clay-bound pore is declared capillary-held,
        // so PHIFF goes to zero and SWIRR_T to 1.0: Jauhar's "cwsh will always be exist even so
        // small", at the porosity where the SSC triangle stops discriminating. CWSH is `sw_rtc`'s
        // declared CAPBW input, so this moves Sw on tight streaks - which is why the threshold is
        // a per-zone parameter and not a literal.
        //
        // The Loglan-order companion test `cbw < 0.05` is GONE because it could never decide
        // anything: `cbw = phit - phie` with PHIE clamped into [0, PHIT], so `cbw <= phit` always
        // and `phit < 0.05` already implies it. Pinned by
        // `the_tight_rock_floor_is_a_declared_parameter_whose_default_changes_nothing`.
        if phit < ctx.p("PHIT_TIGHT", i) {
            cwsh = phit - cbw;
        }
        let phiff = phit - cbw - cwsh;
        bw = cbw + cwsh;

        // GR-equivalent volumes: rescale shale-side volumes by VSHGR/VWSH and
        // sand-side volumes by (1-VSHGR)/(1-VWSH) so the track sums honour VSHGR.
        let (mut vsand_g, mut vsilt_g, mut vdcl_g, mut cbw_g, mut cwsh_g, mut phiff_g) =
            (f64::NAN, f64::NAN, f64::NAN, f64::NAN, f64::NAN, f64::NAN);
        if !vshgr.is_nan() && vwsh > 1e-9 && vwsh < 1.0 - 1e-9 {
            vdcl_g = (vdcl / vwsh) * vshgr;
            vsilt_g = (vsilt / vwsh) * vshgr;
            cbw_g = (cbw / vwsh) * vshgr;
            cwsh_g = if cwsh <= 0.005 { 0.0 } else { (cwsh / vwsh) * vshgr };
            vsand_g = if vsand <= 0.005 {
                0.0
            } else if phiff > 0.005 {
                (vsand / (1.0 - vwsh)) * (1.0 - vshgr)
            } else {
                f64::NAN
            };
            phiff_g = if phiff <= 0.005 {
                1.0 - vshgr
            } else {
                (phiff / (1.0 - vwsh)) * (1.0 - vshgr)
            };
        }
        let phie_g = phiff_g + cwsh_g;
        let phit_g = phie_g + cbw_g;

        let mut set = |k: &str, v: f64| out.get_mut(k).unwrap()[i] = v as f32;
        set("VSAND", vsand);
        set("VSILT", vsilt);
        set("VDCL", vdcl);
        set("VWCL", limit(vwcl, 0.0, 1.0));
        set("VSH_SSC", limit(vsh, 0.0, 1.0));
        set("VSHGR", vshgr);
        set("VSHND", vshnd);
        set("PHIT_SSC", phit);
        set("PHIE_SSC", phie);
        set("PHIFF_SSC", limit(phiff, 0.0, phit));
        set("CBW", cbw);
        set("CWSH", cwsh);
        set("BW", limit(bw, 0.0, phit));
        // The Loglan computes SWIRR_T and SWIRR_EFF together BEFORE the capillary
        // conditioning (.lls lines 213-216) and never revisits them; recomputing only
        // SWIRR_T from the post-conditioning BW here made the written pair mutually
        // inconsistent whenever the SWIRR_MIN floor or the PHIT_TIGHT rule fired.
        // Write the reference's consistent pre-conditioning pair.
        set("SWIRR_T", swirr_t);
        set("SWIRR_EFF", swirr_eff);
        set("VSAND_GR", vsand_g);
        set("VSILT_GR", vsilt_g);
        set("VDCL_GR", vdcl_g);
        set("CBW_GR", cbw_g);
        set("CWSH_GR", cwsh_g);
        set("PHIFF_GR", phiff_g);
        set("PHIE_GR", phie_g);
        set("PHIT_GR", phit_g);
    }

    out
}

// ---------------------------------------------------------------------------
// SSPW — Sandstone Petrophysical Workflow (March 2022)
// ---------------------------------------------------------------------------

pub fn sspw_spec() -> ModuleSpec {
    ModuleSpec {
        name: "sspw".into(),
        title: "SSPW — Sandstone Petrophysical Workflow".into(),
        category: "Porosity".into(),
        doc: "Three-component sandstone workflow (quartz + shale + water). PHIT from density \
              with a VSH-mixed dry matrix (RHOB_MAT / RHOB_DSH); shale total porosity \
              PHIT_SH = (RHOB_DSH − RHOB_SH)/(RHOB_DSH − RHOB_FL); CBW = VSH·VOL_CBW_SH; \
              CAPBW = VSH·(PHIT_SH − VOL_CBW_SH). Key message: PHIE = PHIT − CBW (clay \
              bound only); PHIFF = PHIT − CBW − CAPBW is the movable-fluid porosity; \
              SWIRR = (CBW+CAPBW)/PHIT floored at SWIRR_MIN. NPHI must be sandstone units. \
              Exec arithmetic reconstructed from the reference spec — check against the reference \
              PHIT/PHIE LAS output."
            .into(),
        args: vec![
            param(
                "RHOB_MAT", "Bulk density of matrix point", "g/cc", 2.65, 2.0, 3.0,
                "IP/Techlog/SandiMin sandstone matrix endpoint 2.65 g/cm3; docs/PRD_v2/11_porosity.md §5.1",
            ),
            param_open("NPHI_MAT", "Neutron of matrix point", "v/v", -0.1, 0.2, true),
            param_open("RHOB_SH", "Bulk density of measured (wet) shale", "g/cc", 1.5, 3.5, true),
            param_open("NPHI_SH", "Neutron of measured shale", "v/v", 0.0, 1.0, true),
            param_open("RHOB_DSH", "Dry shale grain density (0 p.u. shale)", "g/cc", 2.0, 3.0, true),
            param_open("VOL_CBW_SH", "Clay-bound water volume in wet shale", "v/v", 0.0, 1.0, true),
            param_open("SWIRR_MIN", "Minimum irreducible water saturation", "v/v", 0.0, 1.0, true),
            param(
                "GAS_C", "Gas-conditioning weight (0 = density only, 1 = even, 2 = neutron only)",
                "", 1.6, 0.0, 2.0,
                "porosity_sspw.lls (2022) gas branch c = 1.6; RULED by DEC-086 on field observation that the even split still reads optimistic",
            ),
            // SB-POR-008: the water filling shale porosity is FORMATION water, so PHIT_SH is
            // anchored on RHO_W and not on the invaded-zone RHOB_FL below. Both ship at 1.00, so
            // this separates only once salt water is selected.
            crate::modules::with_sources(
                param(
                    "RHO_W", "Formation water density", "g/cc", 1.0, 0.8, 1.3,
                    "Geolog V14 phi_den.info RHO_W DEFAULT 1000 k/m3; docs/PRD_v2/11_porosity.md §5.1",
                ),
                crate::param_sources::FORMATION_WATER_DENSITY,
            ),
            param(
                "RHOB_FL", "Density of invaded-zone fluid", "g/cc", 1.0, 0.5, 1.5,
                "Geolog V14 phi_dnh.info RHO_MF DEFAULT 1000 k/m3; docs/PRD_v2/11_porosity.md §5.4",
            ),
            param(
                "NPHI_FL", "Neutron response of flushed-zone fluid", "v/v", 1.0, 0.5, 1.2,
                "Geolog V14 vsh_dn.info and Techlog VSH neutron-density NPHI fluid 1.0; docs/PRD_v2/10_clay-volume.md §5",
            ),
            log_in("RHOB", "Bulk density", "g/cc", "RHOB", true),
            log_in("NPHI", "Neutron porosity (sandstone units)", "v/v", "NPHI", false),
            log_in("VSH", "Shale volume", "v/v", "VSH", true),
            log_out("PHIT_SSPW", "Total porosity", "v/v"),
            log_out("PHIE_SSPW", "Effective porosity (PHIT − CBW)", "v/v"),
            log_out("PHIFF_SSPW", "Free fluid porosity", "v/v"),
            log_out("CBW_SSPW", "Clay-bound water volume", "v/v"),
            log_out("CAPBW_SSPW", "Capillary-bound water volume", "v/v"),
            log_out("BW_SSPW", "Total bound water volume", "v/v"),
            log_out("SWIRR_SSPW", "Irreducible water saturation", "v/v"),
        ],
    }
}

pub fn sspw(ctx: &ModuleContext) -> ModuleOutputs {
    let rhob = ctx.log("RHOB");
    let nphi = ctx.log("NPHI");
    let vsh_in = ctx.log("VSH");

    let mut phit_o = vec![f32::NAN; ctx.n];
    let mut phie_o = vec![f32::NAN; ctx.n];
    let mut phiff_o = vec![f32::NAN; ctx.n];
    let mut cbw_o = vec![f32::NAN; ctx.n];
    let mut capbw_o = vec![f32::NAN; ctx.n];
    let mut bw_o = vec![f32::NAN; ctx.n];
    let mut swirr_o = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let (r, np, vsh) = (rhob[i] as f64, nphi[i] as f64, vsh_in[i] as f64);
        let rhob_mat = ctx.p("RHOB_MAT", i);
        let rhob_sh = ctx.p("RHOB_SH", i);
        let rhob_dsh = ctx.p("RHOB_DSH", i);
        let vol_cbw_sh = ctx.p("VOL_CBW_SH", i);
        let swirr_min = ctx.p("SWIRR_MIN", i);
        let rhob_fl = ctx.p("RHOB_FL", i);
        let rho_w = ctx.p("RHO_W", i);
        let gas_c = ctx.p("GAS_C", i);
        // Shale/CBW params joined the guard: with any of them NaN, `(phit_sh -
        // vol_cbw_sh).max(0.0)` silently swallowed the NaN into 0.0 and a NaN `cbw`
        // then reached `limit` as a NaN clamp bound. Outputs are NaN-initialised, so
        // skipping the sample is the contract-correct result. SWIRR_MIN stays out —
        // it is legitimately optional and checked with `!is_nan()` at use.
        if r.is_nan()
            || vsh.is_nan()
            || rhob_mat.is_nan()
            || rhob_fl.is_nan()
            || rhob_sh.is_nan()
            || rhob_dsh.is_nan()
            || vol_cbw_sh.is_nan()
            // Same reason as SSC's: `f64::NAN.max(0.0)` is 0.0, so a NaN weight would put
            // PHIT at zero on every gas sample rather than failing where it can be seen.
            || gas_c.is_nan()
        {
            continue;
        }
        let vsh = limit(vsh, 0.0, 1.0);

        // Same gas conditioning as SSC (DEC-086), and the same dial — only the DEFAULT differs
        // (1.6 here, 1 there). SSPW corrects the density leg only, because its PHIT is a
        // density porosity against the VSH-mixed dry matrix; there is no corrected neutron to
        // carry, so the leg-crossing that c > 1 produces in SSC has no visible counterpart here
        // — the symptom is a PHIT biased low, which is exactly the conservatism Jauhar ruled for.
        let phidi = (rhob_mat - r) / (rhob_mat - rhob_fl);
        let gas_pull = !np.is_nan() && np <= 1.05 * phidi;
        // SB-POR-003: same custody identity as SSC's gas branch.
        crate::modules::record_branch(if gas_pull {
            "gas-conditioned (pulled onto the sand base line)"
        } else {
            "no gas conditioning"
        });
        let rhob_cor = if gas_pull {
            let (d2, n2) = (phidi * phidi, np * np);
            let phid = (d2 - gas_c * (d2 - n2).abs() / 2.0).max(0.0).sqrt();
            rhob_mat - (rhob_mat - rhob_fl) * phid
        } else {
            r
        };

        // Dry-matrix mix and total (density) porosity.
        let rhoma = (1.0 - vsh) * rhob_mat + vsh * rhob_dsh;
        let phit_raw = (rhoma - rhob_cor) / (rhoma - rhob_fl);
        let phit = limit(phit_raw, 0.0, 0.75);
        if !phit_raw.is_nan() && phit != phit_raw {
            crate::modules::record_bound_limit("PHIT");
        }

        // Wet-shale total porosity and the bound-water split. SB-POR-008: this is the product's
        // one clay-bound-water porosity, so it comes from the shared definition and is anchored on
        // FORMATION water, not on the invaded-zone `rhob_fl` the density porosity above uses. The
        // existing [0,1] clamp is retained exactly as it was.
        let phit_sh = limit(
            crate::modules::shale_total_porosity(rhob_dsh, rhob_sh, rho_w),
            0.0,
            1.0,
        );
        let cbw = limit(vsh * vol_cbw_sh, 0.0, phit);
        let capbw_raw = vsh * (phit_sh - vol_cbw_sh).max(0.0);
        let capbw = limit(capbw_raw, 0.0, phit - cbw);
        let phie_raw = phit - cbw;
        let phie = limit(phie_raw, 0.0, phit);
        if !phie_raw.is_nan() && phie != phie_raw {
            crate::modules::record_bound_limit("PHIE");
        }
        let mut bw = cbw + capbw;

        // SWIRR floor: pad capillary water up to SWIRR_MIN·PHIT if needed.
        let mut cap = capbw;
        if phit > 0.0 && !swirr_min.is_nan() && bw / phit < swirr_min {
            cap = limit(swirr_min * phit - cbw, 0.0, phit - cbw);
            bw = cbw + cap;
        }
        let phiff = limit(phit - cbw - cap, 0.0, phie);
        let swirr = if phit > 0.0 { limit(bw / phit, 0.0, 1.0) } else { f64::NAN };

        phit_o[i] = phit as f32;
        phie_o[i] = phie as f32;
        phiff_o[i] = phiff as f32;
        cbw_o[i] = cbw as f32;
        capbw_o[i] = cap as f32;
        bw_o[i] = bw as f32;
        swirr_o[i] = swirr as f32;
    }

    HashMap::from([
        ("PHIT_SSPW".to_string(), phit_o),
        ("PHIE_SSPW".to_string(), phie_o),
        ("PHIFF_SSPW".to_string(), phiff_o),
        ("CBW_SSPW".to_string(), cbw_o),
        ("CAPBW_SSPW".to_string(), capbw_o),
        ("BW_SSPW".to_string(), bw_o),
        ("SWIRR_SSPW".to_string(), swirr_o),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with(logs: Vec<(&str, Vec<f32>)>, spec: &ModuleSpec, n: usize) -> ModuleContext {
        let mut params = HashMap::new();
        let mut opts = HashMap::new();
        // CHARACTERIZATION INPUTS — the existing SSC/SSPW equation fixtures retain their
        // historical endpoints explicitly; SB-CORE-004 removes them from the shipping manifest.
        // Source: the pre-SB-CORE-004 manifests in git and the named crossplot fixtures below.
        let fixture_value = |name: &str| match (spec.name.as_str(), name) {
            ("ssc", "GR_MA") => 10.0,
            ("ssc", "GR_SH") => 150.0,
            ("ssc", "RHOB_MA") => 2.65,
            ("ssc", "NPHI_MA") => 0.0,
            ("ssc", "RHOB_FL") | ("ssc", "NPHI_FL") => 1.0,
            ("ssc", "RHOB_WCL") => 2.3,
            ("ssc", "NPHI_WCL") => 0.6,
            ("ssc", "RHOB_DCL") => 2.71,
            ("ssc", "NPHI_WSI") => 0.3,
            ("ssc", "DCLF_SI") => 0.1,
            ("ssc", "PHIT_CL") => 0.24,
            ("ssc", "SWIRR_MIN") => 0.0,
            // DEC-086 then DEC-088: each module's shipped default, stated here so the fixtures
            // exercise what actually ships. SSC's moved 1.0 -> 1.6 under DEC-088, which DOES move
            // SSC gas numbers - that was the ruling, not a side effect.
            ("ssc", "GAS_C") => 1.6,
            // DEC-093: the literal this parameter replaced, stated here so every existing SSC
            // fixture keeps running the tight-rock rule exactly as it always did.
            ("ssc", "PHIT_TIGHT") => 0.05,
            ("sspw", "RHOB_MAT") => 2.65,
            ("sspw", "NPHI_MAT") => 0.0,
            ("sspw", "RHOB_SH") => 2.4,
            ("sspw", "NPHI_SH") => 0.55,
            ("sspw", "RHOB_DSH") => 2.71,
            ("sspw", "VOL_CBW_SH") => 0.1,
            ("sspw", "SWIRR_MIN") => 0.0,
            ("sspw", "GAS_C") => 1.6,
            ("sspw", "RHOB_FL") | ("sspw", "NPHI_FL") => 1.0,
            // SB-POR-008: fresh formation water, equal to this fixture's fluid density on purpose.
            // Holding them equal is what makes these existing assertions a control proving the
            // shared-helper change is behaviour-neutral wherever the two anchors agree.
            ("sspw", "RHO_W") => 1.0,
            _ => panic!("no explicit SSC test fixture for {}.{name}", spec.name),
        };
        for a in &spec.args {
            match a.kind {
                crate::modules::ArgKind::Param => {
                    let v = fixture_value(&a.name);
                    params.insert(a.name.clone(), vec![v; n]);
                }
                crate::modules::ArgKind::Option => {
                    opts.insert(a.name.clone(), a.default.clone());
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
    fn ssc_clean_sand_is_mostly_sand() {
        // Clean water-wet sand: GR near matrix, RHOB 2.4 / NPHI 0.15 sits close to the
        // quartz-fluid line, well left of the silt point.
        let spec = ssc_spec();
        let ctx = ctx_with(
            vec![("GR", vec![15.0]), ("RHOB", vec![2.40]), ("NPHI", vec![0.15])],
            &spec,
            1,
        );
        let out = ssc(&ctx);
        let vsand = out["VSAND"][0];
        let vdcl = out["VDCL"][0];
        let phit = out["PHIT_SSC"][0];
        let phie = out["PHIE_SSC"][0];
        assert!(vsand > 0.5, "clean sand should be sand-dominated, got VSAND={vsand}");
        assert!(vdcl < 0.1, "clean sand should carry little dry clay, got VDCL={vdcl}");
        assert!(phit > 0.10 && phit < 0.25, "PHIT out of range: {phit}");
        assert!(phie <= phit && phie > 0.0);
        // Volumes + porosity close within tolerance.
        let total = out["VSAND"][0] + out["VSILT"][0] + out["VDCL"][0] + phit;
        assert!((total - 1.0).abs() < 0.02, "bulk closure violated: {total}");
    }

    #[test]
    fn ssc_shale_point_is_clay_dominated_with_low_phie() {
        // At the wet clay point itself (RHOB 2.3, NPHI 0.6) the model must read clay.
        let spec = ssc_spec();
        let ctx = ctx_with(
            vec![("GR", vec![150.0]), ("RHOB", vec![2.30]), ("NPHI", vec![0.60])],
            &spec,
            1,
        );
        let out = ssc(&ctx);
        assert!(out["VDCL"][0] > 0.3, "shale should be clay-dominated: VDCL={}", out["VDCL"][0]);
        assert_eq!(out["VSAND"][0], 0.0, "no sand at the clay point");
        assert!(out["PHIE_SSC"][0] < out["PHIT_SSC"][0], "CBW must reduce PHIE");
        assert!(out["SWIRR_T"][0] > 0.5, "shale is bound-water dominated");
    }

    /// The 8-curve GR-equivalent (*_GR) family: with a finite VSHGR and 0 < VWSH < 1, the
    /// GR-rescaled volumes + porosity close to unity (VSAND_GR+VSILT_GR+VDCL_GR+PHIT_GR ≈ 1)
    /// and honour their definitions; at a degenerate VWSH (pure clay point → VWSH ≈ 1) the whole
    /// block is skipped, leaving all eight NaN.
    #[test]
    fn ssc_gr_equivalent_family_closes_and_guards_degenerate_vwsh() {
        let spec = ssc_spec();

        // Silty sand: a genuine sand/silt/clay mix so VWSH is strictly interior.
        let out = ssc(&ctx_with(
            vec![("GR", vec![70.0]), ("RHOB", vec![2.38]), ("NPHI", vec![0.25])],
            &spec,
            1,
        ));
        let g = |k: &str| out[k][0] as f64;
        for k in ["VSAND_GR", "VSILT_GR", "VDCL_GR", "CBW_GR", "CWSH_GR", "PHIFF_GR", "PHIE_GR", "PHIT_GR"] {
            assert!(out[k][0].is_finite(), "{k} should be finite for a mixed sample, got {}", out[k][0]);
        }
        let closure = g("VSAND_GR") + g("VSILT_GR") + g("VDCL_GR") + g("PHIT_GR");
        assert!((closure - 1.0).abs() < 0.02, "GR-track closure violated: {closure}");
        // Definitional: PHIT_GR = PHIE_GR + CBW_GR, PHIE_GR = PHIFF_GR + CWSH_GR.
        assert!((g("PHIT_GR") - (g("PHIE_GR") + g("CBW_GR"))).abs() < 1e-4);
        assert!((g("PHIE_GR") - (g("PHIFF_GR") + g("CWSH_GR"))).abs() < 1e-4);

        // Pure wet-clay point → VWSH ≈ 1 → the block's `vwsh < 1-1e-9` guard skips it.
        let sh = ssc(&ctx_with(
            vec![("GR", vec![150.0]), ("RHOB", vec![2.30]), ("NPHI", vec![0.60])],
            &spec,
            1,
        ));
        assert!(sh["VSAND_GR"][0].is_nan(), "degenerate VWSH must leave *_GR NaN, got {}", sh["VSAND_GR"][0]);
        assert!(sh["PHIT_GR"][0].is_nan());
    }

    /// DEC-093. `ssc()` runs FOUR capillary-water conditioning rules where
    /// `docs/method_ssc_sspw.md` carried TWO. The extra pair is Jauhar's own, added to keep CWSH
    /// non-negative and to keep some capillary water everywhere ("cwsh will always be exist even
    /// so small", 2026-08-20); he KEPT both on 2026-08-22. What changes is that the tight-rock
    /// threshold stops being a bare literal and becomes a per-zone parameter, because CWSH is
    /// `sw_rtc`'s declared CAPBW input and this rule moves Sw.
    ///
    /// Rule 4 decides only CLEAN tight rock. Its assignment `cwsh = phit - cbw` IS `cwsh = phie`,
    /// the same value rule 2 writes - so where a tight sample is shaly enough that the triangle
    /// has already spent PHIE on capillary water, rule 2 clamps first and rule 4 changes nothing.
    /// On clean rock the triangle leaves free porosity and rule 4 takes it.
    ///
    /// Pinned from both sides, because either half alone passes for the wrong reason. That the
    /// parameter DRIVES the branch would be satisfied by a parameter whose default quietly
    /// differs from the literal it replaced; that the default reproduces today's answer would be
    /// satisfied by a parameter nothing reads.
    #[test]
    fn the_tight_rock_floor_is_a_declared_parameter_whose_default_changes_nothing() {
        let spec = ssc_spec();
        // A clean 4-p.u. streak - the rock this rule is about.
        let mk = || ctx_with(vec![("GR", vec![10.0]), ("RHOB", vec![2.58]), ("NPHI", vec![0.04])], &spec, 1);

        // Arm B: the shipped default is the literal it replaced, and it still turns this sample
        // fully capillary-bound - the answer the module has given since it was written.
        let shipped: f64 = spec
            .args
            .iter()
            .find(|arg| arg.name == "PHIT_TIGHT")
            .expect("the threshold is declared")
            .default
            .parse()
            .expect("a numeric default");
        assert_eq!(
            shipped, 0.05,
            "parameterising this rule must not move the number it replaced",
        );
        let bound = ssc(&mk());
        let phit = bound["PHIT_SSC"][0] as f64;
        assert!(
            phit > 0.0 && phit < shipped,
            "the fixture must be TIGHTER than the threshold or this test proves nothing: {phit}",
        );
        assert!(
            bound["PHIFF_SSC"][0] as f64 <= 1e-9,
            "below the threshold no porosity is free: {}",
            bound["PHIFF_SSC"][0],
        );

        // Arm A: the parameter really decides the branch. Put it below the sample's own porosity
        // and the triangle's answer survives - 3.9 p.u. here, which is the size of what this rule
        // spends on one clean streak and the reason it is a per-zone number now.
        let mut open = mk();
        open.params.insert("PHIT_TIGHT".into(), vec![0.0]);
        let free = ssc(&open)["PHIFF_SSC"][0] as f64;
        assert!(
            free > 0.03,
            "with the threshold below PHIT the triangle's free porosity must survive: {free}",
        );
    }

    #[test]
    fn ssc_swirr_floor_pads_capillary_water() {
        let spec = ssc_spec();
        let mut ctx = ctx_with(
            vec![("GR", vec![15.0]), ("RHOB", vec![2.40]), ("NPHI", vec![0.15])],
            &spec,
            1,
        );
        // Baseline with the default SWIRR_MIN = 0 (floor inactive).
        let base = ssc(&ctx);
        let (base_bw, base_cwsh) = (base["BW"][0] as f64, base["CWSH"][0] as f64);
        let phit = base["PHIT_SSC"][0] as f64;
        let base_ratio = base_bw / phit;
        assert!(
            base_ratio < 0.35,
            "fixture must start BELOW the floor or this test proves nothing: {base_ratio}"
        );

        ctx.params.insert("SWIRR_MIN".into(), vec![0.35]);
        let out = ssc(&ctx);

        // The floor pads CAPILLARY water (what this test is named for): CWSH rises until
        // total bound water reaches SWIRR_MIN*PHIT (ssc.rs `if ... bw / phit < swirr_min`).
        let bw = out["BW"][0] as f64;
        assert!(
            out["CWSH"][0] as f64 > base_cwsh,
            "floor must raise CWSH: {base_cwsh} -> {}",
            out["CWSH"][0]
        );
        assert!(
            bw / phit >= 0.35 - 1e-6,
            "bound water not padded to the floor: {}",
            bw / phit
        );

        // ...but SWIRR_T is the PRE-conditioning pair (.lls 213-216, docs/method_ssc_sspw.md
        // §8 computes SWIRR before listing the conditioning), so it must NOT move. Writing
        // the post-conditioning ratio here would make the written SWIRR_T/SWIRR_EFF pair
        // mutually inconsistent whenever this floor fires.
        assert!(
            ((out["SWIRR_T"][0] as f64) - base_ratio).abs() < 1e-6,
            "SWIRR_T must stay the pre-conditioning ratio {base_ratio}, got {}",
            out["SWIRR_T"][0]
        );
    }

    #[test]
    fn sspw_phie_removes_only_clay_bound_water() {
        let spec = sspw_spec();
        let ctx = ctx_with(
            vec![("RHOB", vec![2.45]), ("NPHI", vec![0.18]), ("VSH", vec![0.3])],
            &spec,
            1,
        );
        let out = sspw(&ctx);
        let phit = out["PHIT_SSPW"][0];
        let phie = out["PHIE_SSPW"][0];
        let phiff = out["PHIFF_SSPW"][0];
        let cbw = out["CBW_SSPW"][0];
        let capbw = out["CAPBW_SSPW"][0];
        assert!((phit - phie - cbw).abs() < 1e-6, "PHIE = PHIT - CBW");
        assert!((phie - phiff - capbw).abs() < 1e-6, "PHIFF = PHIE - CAPBW");
        assert!((cbw - 0.3 * 0.1).abs() < 1e-6, "CBW = VSH * VOL_CBW_SH");
        assert!(phiff > 0.0 && phiff < phie && phie < phit);
        let swirr = out["SWIRR_SSPW"][0];
        assert!((swirr - (cbw + capbw) / phit).abs() < 1e-6);
    }

    #[test]
    fn sspw_clean_sand_has_no_bound_water() {
        let spec = sspw_spec();
        let ctx = ctx_with(
            vec![("RHOB", vec![2.40]), ("NPHI", vec![0.15]), ("VSH", vec![0.0])],
            &spec,
            1,
        );
        let out = sspw(&ctx);
        assert_eq!(out["CBW_SSPW"][0], 0.0);
        assert_eq!(out["CAPBW_SSPW"][0], 0.0);
        assert!((out["PHIT_SSPW"][0] - out["PHIFF_SSPW"][0]).abs() < 1e-6);
        // Pure density porosity: (2.65-2.40)/(2.65-1.0) = 0.1515
        assert!((out["PHIT_SSPW"][0] - 0.1515).abs() < 0.002);
    }

    /// DEC-086 then DEC-088. The gas-conditioning weight is the user's dial, and BOTH modules now
    /// ship 1.6 on Jauhar's field observation that the even split still reads optimistic — first
    /// for SSPW (DEC-086), then extended to SSC when he was asked whether the same experience
    /// applied to his SSC work (DEC-088).
    ///
    /// **Agreement is not the contract here; the RULED VALUE is.** The two arrived at 1.6 by two
    /// separate rulings over two different reference files, so this pins the number and its source
    /// rather than the fact that they match — a later ruling could legitimately move one alone.
    /// The dial itself is pinned for the reason it always was: harmonising the two defaults and
    /// harmonising the two ARITHMETICS are different mistakes, and only checking the defaults
    /// would let the second one through.
    #[test]
    fn the_gas_conditioning_weight_is_the_users_dial_and_both_modules_ship_the_ruled_default() {
        // One gas sand, read by both modules. Clean (VSH 0.10 is the existing SSPW fixture's
        // shale) so the density-porosity arithmetic is visible in PHIT with little dilution.
        // RHOB 2.20 against a 2.65 matrix and 1.00 fluid gives phiDI = 0.272727; NPHI 0.10 is
        // well under 1.05*phiDI, so both modules take the gas branch.
        let d2: f64 = ((2.65 - 2.20) / (2.65 - 1.0f64)).powi(2);
        let n2: f64 = 0.10f64 * 0.10;
        let phi_at = |c: f64| (d2 - c * (d2 - n2).abs() / 2.0).max(0.0).sqrt();

        // The algebra this test is written against, stated independently of the module code:
        // c = 1 is the RMS midpoint, c = 0 leaves the density alone, c = 2 hands it to NPHI.
        assert!((phi_at(1.0) - ((d2 + n2) / 2.0).sqrt()).abs() < 1e-12, "c = 1 is the even split");
        assert!((phi_at(0.0) - d2.sqrt()).abs() < 1e-12, "c = 0 is no correction at all");
        assert!((phi_at(2.0) - n2.sqrt()).abs() < 1e-12, "c = 2 is the neutron outright");

        // The SHIPPED defaults, read off the manifests. Asserted separately from the runs below
        // because `ctx_with` fills every parameter from the fixture table rather than from the
        // spec — so a run alone would pin the fixture and let a changed manifest default through,
        // which is exactly what the first draft of this test did.
        let declared = |spec: &ModuleSpec| -> f64 {
            spec.args
                .iter()
                .find(|a| a.name == "GAS_C")
                .unwrap_or_else(|| panic!("{} must declare GAS_C", spec.name))
                .default
                .parse::<f64>()
                .expect("GAS_C default must be numeric")
        };
        assert!((declared(&ssc_spec()) - 1.6).abs() < 1e-12, "SSC ships 1.6 (DEC-088)");
        assert!((declared(&sspw_spec()) - 1.6).abs() < 1e-12, "SSPW ships 1.6 (DEC-086)");

        let run_sspw = |gas_c: Option<f64>| -> f64 {
            let spec = sspw_spec();
            let mut ctx = ctx_with(
                vec![("RHOB", vec![2.20]), ("NPHI", vec![0.10]), ("VSH", vec![0.0])],
                &spec,
                1,
            );
            if let Some(c) = gas_c {
                ctx.params.insert("GAS_C".into(), vec![c]);
            }
            let out = sspw(&ctx);
            out["PHIT_SSPW"][0] as f64
        };
        let run_ssc = |gas_c: Option<f64>| -> f64 {
            let spec = ssc_spec();
            let mut ctx = ctx_with(
                vec![("GR", vec![10.0]), ("RHOB", vec![2.20]), ("NPHI", vec![0.10])],
                &spec,
                1,
            );
            if let Some(c) = gas_c {
                ctx.params.insert("GAS_C".into(), vec![c]);
            }
            let out = ssc(&ctx);
            out["PHIT_SSC"][0] as f64
        };

        // VSH 0 in SSPW makes the dry matrix pure quartz, so PHIT IS the corrected density
        // porosity and the coefficient's effect is readable directly.
        let sspw_default = run_sspw(None);
        assert!(
            (sspw_default - phi_at(1.6)).abs() < 1e-5,
            "SSPW defaults to 1.6 (DEC-086): expected {}, got {sspw_default}",
            phi_at(1.6)
        );
        assert!(
            (run_sspw(Some(1.0)) - phi_at(1.0)).abs() < 1e-5,
            "and the dial reaches the even split when the user asks for it"
        );
        // 1.6 must be the CONSERVATIVE side of the even split — that is the whole content of both
        // rulings, and the reason a well-meaning tidy-up back to the RMS midpoint is a defect.
        assert!(
            sspw_default < phi_at(1.0) - 1e-4,
            "1.6 must read lower than the even split, got {sspw_default} vs {}",
            phi_at(1.0)
        );

        // DEC-088: SSC now ships the same ruled weight, asserted through the RUN as well as the
        // manifest, because the ruling is about the number the module computes with rather than
        // the string it advertises. The dial still reaches the even split, which must read HIGHER.
        // Compared as "much nearer 1.6 than the even split" rather than against an absolute
        // tolerance, because SSC's PHIT is not the bare corrected density porosity: it runs
        // through the sand-silt-clay framework, whose RHOMA follows the projected point. Past
        // c = 1 the corrected legs CROSS, which shifts that projection — so SSC lands close to
        // phi(1.6) but not exactly on it, while SSPW (whose PHIT is the density leg itself) does.
        let ssc_default = run_ssc(None);
        let (to_ruled, to_even) =
            ((ssc_default - phi_at(1.6)).abs(), (ssc_default - phi_at(1.0)).abs());
        assert!(
            to_ruled < to_even / 3.0,
            "SSC must default to the ruled 1.6, not the even split: got {ssc_default}, which is \
             {to_ruled} from phi(1.6) = {} and {to_even} from phi(1.0) = {}",
            phi_at(1.6),
            phi_at(1.0)
        );
        assert!(
            run_ssc(Some(1.0)) > ssc_default + 1e-4,
            "and the dial still reaches the even split, which reads higher: {} vs {ssc_default}",
            run_ssc(Some(1.0))
        );
    }

    /// SB-POR-003's SSC/SSPW half (DEC-039 form): the two flagship porosity methods record
    /// their POROSITY branches and binds through the same capture channel the phi_* family
    /// uses - the gas-conditioning branch, SSC's dry-silt-point split, and a PHIT that hit
    /// its published ceiling - so the workflow runner can write them into the run's version
    /// comment. Scope is the chapter's own: porosity branches and porosity limits; the
    /// saturation machinery is not part of SB-POR-003.
    #[test]
    fn ssc_and_sspw_record_their_porosity_branches_and_binds_for_the_runs_custody_comment() {
        let spec = ssc_spec();
        // Three samples: no-gas sand (NPHI 0.30 > 1.05*PHID 0.16), gas sand (NPHI 0.10 under
        // the line at PHID 0.273), and an off-scale washout reading whose raw PHIT ~0.88
        // hits the 0.75 ceiling (NPHI 0.95 keeps it off the gas branch: 0.95 > 1.05*0.879).
        let ctx = ctx_with(
            vec![
                ("GR", vec![15.0, 15.0, 15.0]),
                ("RHOB", vec![2.40, 2.20, 1.20]),
                ("NPHI", vec![0.30, 0.10, 0.95]),
            ],
            &spec,
            3,
        );
        let (_, _, _, _, (bound, branches)) = crate::modules::run_module_with_degradations(
            "ssc",
            &ctx,
            crate::modules::DefaultUsage::default(),
        )
        .unwrap();
        let count = |list: &[(String, usize)], name: &str| {
            list.iter().find(|(n, _)| n == name).map(|(_, c)| *c).unwrap_or(0)
        };
        assert_eq!(
            count(&branches, "gas-conditioned (pulled onto the sand base line)"),
            1,
            "branches: {branches:?}"
        );
        assert_eq!(count(&branches, "no gas conditioning"), 2, "branches: {branches:?}");
        // Every computed sample lands on exactly one side of the dry-silt point.
        assert_eq!(
            count(&branches, "left of the dry-silt point (clay-sand-silt)")
                + count(&branches, "right of the dry-silt point (clay-silt)"),
            3,
            "branches: {branches:?}"
        );
        assert_eq!(count(&bound, "PHIT"), 1, "the washout sample's PHIT ceiling must be counted, got {bound:?}");

        let spec = sspw_spec();
        let ctx = ctx_with(
            vec![
                ("RHOB", vec![2.40, 2.20]),
                ("NPHI", vec![0.35, 0.10]),
                ("VSH", vec![0.10, 0.10]),
            ],
            &spec,
            2,
        );
        let (_, _, _, _, (_, branches)) = crate::modules::run_module_with_degradations(
            "sspw",
            &ctx,
            crate::modules::DefaultUsage::default(),
        )
        .unwrap();
        assert_eq!(
            count(&branches, "gas-conditioned (pulled onto the sand base line)"),
            1,
            "branches: {branches:?}"
        );
        assert_eq!(count(&branches, "no gas conditioning"), 1, "branches: {branches:?}");
    }
}
