//! Sand-Silt-Clay (SSC) and Sandstone Petrophysical Workflow (SSPW) modules.
//!
//! SSC is a faithful port of Jauhar's Loglan `ssc_lqr_gap_edit_jau.lls`
//! ("Modifikasi Metoda Sand Silt Clay untuk Perhitungan Zona LQR", GAP 2023, after
//! Kuttan et al., "Log Interpretation in the Malay Basin", 21st SPWLA). Designed for
//! very fine grained sediments with fresh–brackish formation water, where classic
//! shaly-sand analysis overestimates clay and underestimates porosity.
//!
//! SSPW is the PHR-standard sandstone workflow (quartz + shale + water, density
//! porosity with a dry-shale matrix). Its key message: PHIE = PHIT − clay-bound
//! water only; capillary-bound water stays inside PHIE, and PHIFF = PHIT − CBW −
//! CAPBW is what can actually flow. The Loglan exec body is not on disk, so the
//! arithmetic here is reconstructed from the module spec (`porosity_sspw.lls`) and
//! the same physics the SSC source spells out — verify against the reference LAS output.
//!
//! Deviations from the Loglan, both deliberate:
//! - `RANNORMAL(SWIRR_MIN*PHIT, 0.005)` becomes deterministic `SWIRR_MIN*PHIT`.
//! - NPHIMA is limited to [0,1] (the Loglan's 0.5–5 limit is a copy-paste of the
//!   RHOMA limit and would clamp every neutron matrix value up to 0.5).

use crate::modules::{log_in, log_out, opt, param, ModuleContext, ModuleOutputs, ModuleSpec};
use std::collections::HashMap;

fn limit(v: f64, lo: f64, hi: f64) -> f64 {
    if v.is_nan() { v } else { v.clamp(lo, hi) }
}

fn vsh_from_gr(method: &str, mut v: f64) -> f64 {
    match method {
        "STIEBER1" => { v = limit(v, -10.0, 1.49); v / (3.0 - 2.0 * v) }
        "STIEBER2" => { v = limit(v, -10.0, 1.99); v / (2.0 - v) }
        "STIEBER3" => { v = limit(v, -10.0, 1.33); v / (4.0 - 3.0 * v) }
        "LARINOV1" => 0.33 * (2.0_f64.powf(2.0 * v) - 1.0),
        "LARINOV2" => 0.083 * (2.0_f64.powf(3.7 * v) - 1.0),
        "LARINOV3" => 0.127 * (3.15_f64.powf(2.0 * v) - 1.0),
        "CLAVIER" => { v = limit(v, -2.53, 1.13); 1.7 - (3.38 - (v + 0.7).powi(2)).sqrt() }
        _ => v, // LINEAR
    }
}

// ---------------------------------------------------------------------------
// SSC — Sand-Silt-Clay model (Kuttan / GAP 2023, LQR Balam South edit)
// ---------------------------------------------------------------------------

pub fn ssc_spec() -> ModuleSpec {
    ModuleSpec {
        name: "ssc".into(),
        title: "SSC — Sand-Silt-Clay (Kuttan/LQR)".into(),
        category: "Porosity".into(),
        doc: "Sand-Silt-Clay model on the N-D crossplot (Kuttan Malay Basin, GAP 2023 LQR \
              edit). Data points are projected from the fluid point onto the dry rock line \
              (matrix→dry clay); sand/silt/clay fractions come from the projection position, \
              matrix density from the fraction mix, PHIT from density. Bound water is split \
              into clay-bound (CBW) and capillary-bound in silt/shale (CWSH): PHIE = PHIT − \
              VWCL·PHIT_CL, PHIFF = PHIT − CBW − CWSH, SWIRR_T = BW/PHIT. GR-equivalent \
              volumes rescale the SSC volumes to honour VSHGR. Defaults are the LQR reference \
              values."
            .into(),
        args: vec![
            opt(
                "OPT_VSHGR",
                "VSH from gamma ray method",
                "LINEAR",
                &["LINEAR", "STIEBER1", "STIEBER2", "STIEBER3", "LARINOV1", "LARINOV2", "LARINOV3", "CLAVIER"],
            ),
            param("GR_MA", "Gamma ray matrix (clean)", "gapi", 10.0, 0.0, 100.0),
            param("GR_SH", "Gamma ray clay", "gapi", 150.0, 0.0, 1000.0),
            param("RHOB_MA", "Density matrix", "g/cc", 2.65, 1.0, 4.0),
            param("NPHI_MA", "Neutron matrix", "v/v", 0.0, -0.1, 1.2),
            param("RHOB_FL", "Density fluid", "g/cc", 1.0, 0.5, 4.0),
            param("NPHI_FL", "Neutron fluid", "v/v", 1.0, -0.1, 1.2),
            param("RHOB_WCL", "Bulk density wet clay", "g/cc", 2.3, 1.0, 4.0),
            param("NPHI_WCL", "Neutron porosity wet clay", "v/v", 0.6, -0.1, 1.2),
            param("RHOB_DCL", "Bulk density dry clay", "g/cc", 2.71, 1.0, 4.0),
            param("NPHI_WSI", "Neutron porosity wet silt", "v/v", 0.3, -0.1, 1.2),
            param("DCLF_SI", "Dry clay fraction at dry silt", "v/v", 0.1, 0.0, 1.0),
            param("PHIT_CL", "Total porosity of clay", "v/v", 0.24, 0.0, 0.8),
            param("SWIRR_MIN", "Minimum total irreducible Sw", "v/v", 0.0, 0.0, 1.0),
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
        if r.is_nan() || np.is_nan() || rhob_ma.is_nan() || rhob_fl.is_nan() {
            continue;
        }

        // Gas/HC conditioning: pull points above the sand base line back onto it.
        let phidi = (rhob_ma - r) / (rhob_ma - rhob_fl);
        let (rhob_cor, nphi_cor) = if np <= 1.05 * phidi {
            let phid = (phidi * phidi - 1.6 * (phidi * phidi - np * np).abs() / 2.0).max(0.0).sqrt();
            (
                rhob_ma - (rhob_ma - rhob_fl) * phid,
                (np * np + 1.6 * (phidi * phidi - np * np).abs() / 2.0).max(0.0).sqrt(),
            )
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
        let phit = limit((rhoma - rhob_cor) / (rhoma - rhob_fl), 0.001, 0.75);

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
        let phie = limit(phit - vwcl * phit_cl, 0.0, phit);
        let cbw = phit - phie;
        let phit_sh = (rhob_dsi - rhob_wsi) / (rhob_dsi - rhob_fl);
        let vwsh = limit(vsh / (1.0 - phit_sh), 0.0, 1.0);
        let mut cwsh = vwsh - vdcl - cbw - vsilt;
        let mut bw = cbw + cwsh;

        let swirr_t = limit(bw / phit, 0.0, 1.0);
        // Guard the /phie divide: at the wet-clay point phie is floored to 0, where the
        // original expression gives -inf->0 ("all water movable") or 0/0->NaN. A zero-
        // effective-porosity sample is fully bound, so report 1.0. (Only degenerate
        // phie==0 samples change; every phie>0 result is unchanged.)
        let swirr_eff = if phie > 0.0 {
            limit(1.0 - phit * (1.0 - swirr_t) / phie, 0.0, 1.0)
        } else {
            1.0
        };

        // Capillary bound water conditioning (Loglan order preserved; the RANNORMAL
        // draw is replaced with its deterministic mean).
        if phie <= 0.002 {
            cwsh = phit - cbw;
        }
        if phie - cwsh <= 0.001 {
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
        if phit < 0.05 && cbw < 0.05 {
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
        set("SWIRR_T", limit(bw / phit, 0.0, 1.0));
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
// SSPW — Sandstone Petrophysical Workflow (PHR standard, March 2022)
// ---------------------------------------------------------------------------

pub fn sspw_spec() -> ModuleSpec {
    ModuleSpec {
        name: "sspw".into(),
        title: "SSPW — Sandstone Petrophysical Workflow".into(),
        category: "Porosity".into(),
        doc: "PHR-standard sandstone workflow (quartz + shale + water). PHIT from density \
              with a VSH-mixed dry matrix (RHOB_MAT / RHOB_DSH); shale total porosity \
              PHIT_SH = (RHOB_DSH − RHOB_SH)/(RHOB_DSH − RHOB_FL); CBW = VSH·VOL_CBW_SH; \
              CAPBW = VSH·(PHIT_SH − VOL_CBW_SH). Key message: PHIE = PHIT − CBW (clay \
              bound only); PHIFF = PHIT − CBW − CAPBW is the movable-fluid porosity; \
              SWIRR = (CBW+CAPBW)/PHIT floored at SWIRR_MIN. NPHI must be sandstone units. \
              Exec arithmetic reconstructed from the reference spec — check against the reference \
              PHIT/PHIE LAS output."
            .into(),
        args: vec![
            param("RHOB_MAT", "Bulk density of matrix point", "g/cc", 2.65, 2.0, 3.0),
            param("NPHI_MAT", "Neutron of matrix point", "v/v", 0.0, -0.1, 0.2),
            param("RHOB_SH", "Bulk density of measured (wet) shale", "g/cc", 2.4, 1.5, 3.5),
            param("NPHI_SH", "Neutron of measured shale", "v/v", 0.55, 0.0, 1.0),
            param("RHOB_DSH", "Dry shale grain density (0 p.u. shale)", "g/cc", 2.71, 2.0, 3.0),
            param("VOL_CBW_SH", "Clay-bound water volume in wet shale", "v/v", 0.1, 0.0, 1.0),
            param("SWIRR_MIN", "Minimum irreducible water saturation", "v/v", 0.0, 0.0, 1.0),
            param("RHOB_FL", "Density of invaded-zone fluid", "g/cc", 1.0, 0.5, 1.5),
            param("NPHI_FL", "Neutron response of flushed-zone fluid", "v/v", 1.0, 0.5, 1.2),
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
        if r.is_nan() || vsh.is_nan() || rhob_mat.is_nan() || rhob_fl.is_nan() {
            continue;
        }
        let vsh = limit(vsh, 0.0, 1.0);

        // Same gas conditioning as SSC when a neutron log is available.
        let phidi = (rhob_mat - r) / (rhob_mat - rhob_fl);
        let rhob_cor = if !np.is_nan() && np <= 1.05 * phidi {
            let phid = (phidi * phidi - 1.6 * (phidi * phidi - np * np).abs() / 2.0).max(0.0).sqrt();
            rhob_mat - (rhob_mat - rhob_fl) * phid
        } else {
            r
        };

        // Dry-matrix mix and total (density) porosity.
        let rhoma = (1.0 - vsh) * rhob_mat + vsh * rhob_dsh;
        let phit = limit((rhoma - rhob_cor) / (rhoma - rhob_fl), 0.0, 0.75);

        // Wet-shale total porosity and the bound-water split.
        let phit_sh = limit((rhob_dsh - rhob_sh) / (rhob_dsh - rhob_fl), 0.0, 1.0);
        let cbw = limit(vsh * vol_cbw_sh, 0.0, phit);
        let capbw_raw = vsh * (phit_sh - vol_cbw_sh).max(0.0);
        let capbw = limit(capbw_raw, 0.0, phit - cbw);
        let phie = limit(phit - cbw, 0.0, phit);
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
        for a in &spec.args {
            match a.kind {
                crate::modules::ArgKind::Param => {
                    let v: f64 = a.default.parse().unwrap();
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

    #[test]
    fn ssc_swirr_floor_pads_capillary_water() {
        let spec = ssc_spec();
        let mut ctx = ctx_with(
            vec![("GR", vec![15.0]), ("RHOB", vec![2.40]), ("NPHI", vec![0.15])],
            &spec,
            1,
        );
        ctx.params.insert("SWIRR_MIN".into(), vec![0.35]);
        let out = ssc(&ctx);
        let swirr = out["SWIRR_T"][0];
        assert!(swirr >= 0.34, "SWIRR floor not applied: {swirr}");
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
}
