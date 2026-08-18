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

use crate::modules::{
    log_in, log_in_one_of, log_out, param_open, ModuleContext, ModuleOutputs, ModuleSpec,
};
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

/// Qv (meq/cm³) at one sample: the QV log when it carries a value, else built from a CEC
/// parameter — `Qv = CEC·ρg·(1−φt)/(100·φt)` (`docs/method_lrlc_rtc_imts.md`, RtC §2).
///
/// Shared by the `sw_rtc` module and the RtC calibration fit below **on purpose**: the fit
/// regresses against exactly the Qv the run will later use, so a change here can never make
/// the calibration and the saturation disagree about what Qv means.
fn qv_at(qv_log: f64, phit: f64, cec: f64, rhog: f64) -> f64 {
    if !qv_log.is_nan() {
        return qv_log.max(0.0);
    }
    if cec.is_nan() || cec <= 0.0 || phit <= 0.0 {
        return 0.0;
    }
    cec * rhog * (1.0 - phit) / (100.0 * phit)
}

/// Theoretical bulk CEC (meq/100g) from the clay model alone, BEFORE the lab scaling factor S:
/// `Σ V_clay · CEC_literature` with the literature constants kaolinite 8 / illite 25 meq/100g
/// (`docs/method_lrlc_rtc_imts.md`, IMTS §1).
///
/// Shared by the `sw_imts` module and the S-factor calibration below **on purpose**, for the same
/// reason `qv_at` is shared with the RtC fit: `S = CEC_lab / cec_theo_at(...)` is then the exact
/// algebraic inverse of what the run computes, so the calibration cannot quietly stop inverting
/// the model it was fitted for.
fn cec_theo_at(vkaol: f64, vill: f64, cec_kaol: f64, cec_ill: f64) -> f64 {
    vkaol * cec_kaol + vill * cec_ill
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
              present, else from CEC·RHOG·(1−PHIT)/(100·PHIT). \
              NO CALIBRATION COEFFICIENTS SHIP AS DEFAULTS. A foreign calibration here does not \
              announce itself: it yields a smooth, plausible \
              Sw that is simply wrong. Fit your own with Advance ▸ Calibrate RtC…, which \
              regresses A_CAP/B_QV/C0 from excess conductivity over an interval you declare \
              water-bearing. CAPBW pairs naturally with SSC's CWSH or \
              SSPW's CAPBW_SSPW. The correction is capped at 98% of the measured \
              conductivity so Rt_corr stays finite."
            .into(),
        args: vec![
            param_open(
                "RW",
                "Formation water resistivity at FT",
                "ohm.m",
                0.001,
                100.0,
                true,
            ),
            param_open("M", "Cementation exponent", "", 1.0, 4.0, true),
            param_open("N", "Saturation exponent", "", 1.0, 4.0, true),
            param_open(
                "A_CAP",
                "Capillary water coefficient",
                "",
                -10.0,
                10.0,
                true,
            ),
            param_open("B_QV", "Qv coefficient", "", -1.0, 1.0, true),
            param_open("C0", "Regression intercept", "", -1.0, 1.0, true),
            param_open("RSF", "Resistivity scaling factor", "", 0.0, 20.0, true),
            param_open(
                "CEC",
                "CEC when no QV log (meq/100g)",
                "meq/100g",
                0.0,
                100.0,
                true,
            ),
            param_open("RHOG", "Grain density for Qv", "g/cc", 2.0, 3.2, true),
            log_in("RT", "Deep resistivity", "ohm.m", "RES_DEEP", true),
            log_in_one_of("PHIT", "Total porosity", "v/v", "PHIT_SSC", &["PHIT_SSPW"]),
            log_in("CAPBW", "Capillary-bound water volume", "v/v", "CWSH", false),
            log_in("QV", "Qv log (meq/cm3), optional", "meq/cm3", "QV", false),
            log_in("CBW", "Clay-bound water (for SWE), optional", "v/v", "CBW", false),
            log_in("PHIT_SSPW", "Total porosity — SSPW fallback (used where PHIT is absent)", "v/v", "PHIT_SSPW", false),
            log_in("CAPBW_SSPW", "Capillary water — SSPW fallback", "v/v", "CAPBW_SSPW", false),
            log_in("CBW_SSPW", "Clay-bound water — SSPW fallback", "v/v", "CBW_SSPW", false),
            // SB-SAT-025: the method-named curves are the UNCLIPPED diagnostics, per the
            // family convention (SWT_ARCH, SWE_INDO, SWE_SIM); the plain pair is clipped and
            // carries exactly the values this module always emitted under the method names.
            log_out("SWT_RTC", "SWT from RtC (unlimited)", "v/v"),
            log_out("SWE_RTC", "SWE from RtC (unlimited)", "v/v"),
            log_out("SWT", "Limited total water saturation", "v/v"),
            log_out("SWE", "Limited effective water saturation", "v/v"),
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
    let mut swt_raw_o = vec![f32::NAN; ctx.n];
    let mut swe_raw_o = vec![f32::NAN; ctx.n];
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
        let qv = qv_at(qv_log[i] as f64, pt, ctx.p("CEC", i), ctx.p("RHOG", i));

        let ct = 1.0 / rt_i;
        let cex = (ctx.p("A_CAP", i) * cap + ctx.p("B_QV", i) * qv + ctx.p("C0", i))
            * pt
            * ctx.p("RSF", i);
        // Never remove more than 98% of the measured conductivity (and never add).
        let cex_applied = cex.clamp(0.0, 0.98 * ct);
        let ct_corr = ct - cex_applied;
        let rt_corr = 1.0 / ct_corr;

        // SB-SAT-025: raw first, clip second. The clipped pair carries exactly the values
        // this module always produced; the diagnostics keep what the model actually said.
        let swt_raw = (rw * ct_corr / pt.powf(m)).powf(1.0 / n_exp);
        let swt = limit(swt_raw, 0.0, 1.0);
        swt_raw_o[i] = swt_raw as f32;
        swt_o[i] = swt as f32;
        rtc_o[i] = rt_corr as f32;
        cex_o[i] = cex_applied as f32;

        let cb = cbw[i] as f64;
        if !cb.is_nan() && pt > cb {
            let swb = limit(cb / pt, 0.0, 0.99);
            swe_raw_o[i] = ((swt_raw - swb) / (1.0 - swb)) as f32;
            swe_o[i] = limit((swt - swb) / (1.0 - swb), 0.0, 1.0) as f32;
        } else {
            swe_raw_o[i] = swt_raw as f32;
            swe_o[i] = swt as f32;
        }
    }

    HashMap::from([
        ("SWT_RTC".to_string(), swt_raw_o),
        ("SWE_RTC".to_string(), swe_raw_o),
        ("SWT".to_string(), swt_o),
        ("SWE".to_string(), swe_o),
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
              until SwT is stable. SWE from CBW. VKAOL/VILL resolve from the selected clay curves. \
              S = measured lab CEC / XRD-theoretical CEC, so it is A PROPERTY OF THE ROCK AND \
              OF THE CLAY CURVES IT IS PAIRED WITH and ships absent. S multiplies the whole \
              clay-charge term, so getting \
              it wrong scales Qv_eff directly and moves Sw with no outward sign. Fit your own \
              with Advance ▸ Calibrate S…, which regresses S from lab CEC measurements against \
              the clay content of the very curves this run will use."
            .into(),
        args: vec![
            param_open(
                "RW",
                "Formation water resistivity at FT",
                "ohm.m",
                0.001,
                100.0,
                true,
            ),
            param_open("TEMP_C", "Formation temperature", "degC", 15.0, 200.0, true),
            param_open("A", "Tortuosity factor a", "", 0.5, 3.0, true),
            param_open(
                "MSTAR",
                "Shaly-sand cementation exponent m*",
                "",
                1.0,
                4.0,
                true,
            ),
            param_open(
                "NSTAR",
                "Shaly-sand saturation exponent n*",
                "",
                1.0,
                4.0,
                true,
            ),
            param_open(
                "S_FACTOR",
                "CEC scaling factor S (lab/XRD)",
                "",
                0.01,
                2.0,
                true,
            ),
            param_open(
                "CEC_KAOL",
                "Kaolinite CEC constant",
                "meq/100g",
                0.0,
                50.0,
                true,
            ),
            param_open(
                "CEC_ILL",
                "Illite CEC constant",
                "meq/100g",
                0.0,
                100.0,
                true,
            ),
            param_open("RHOG", "Grain density", "g/cc", 2.0, 3.2, true),
            param_open(
                "SWIRR_DEF",
                "Swirr fallback when no SWIRR log",
                "v/v",
                0.0,
                0.95,
                true,
            ),
            log_in("RT", "Deep resistivity", "ohm.m", "RES_DEEP", true),
            log_in("PHIT", "Total porosity", "v/v", "PHIT_SSC", true),
            log_in("VKAOL", "Kaolinite volume fraction", "v/v", "VDCL", false),
            log_in("VILL", "Illite volume fraction", "v/v", "VILL", false),
            log_in("SWIRR", "Irreducible Sw (for Qv_eff)", "v/v", "SWIRR_T", false),
            log_in("CBW", "Clay-bound water (for SWE), optional", "v/v", "CBW", false),
            log_in("PHIT_SSPW", "Total porosity — SSPW fallback (used where PHIT is absent)", "v/v", "PHIT_SSPW", false),
            log_in("CBW_SSPW", "Clay-bound water — SSPW fallback", "v/v", "CBW_SSPW", false),
            // SB-SAT-025: method-named = unclipped diagnostic, plain pair = clipped, as
            // across the whole saturation family.
            log_out("SWT_IMTS", "SWT from IMTS (unlimited)", "v/v"),
            log_out("SWE_IMTS", "SWE from IMTS (unlimited)", "v/v"),
            log_out("SWT", "Limited total water saturation", "v/v"),
            log_out("SWE", "Limited effective water saturation", "v/v"),
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
    let mut swt_raw_o = vec![f32::NAN; ctx.n];
    let mut swe_raw_o = vec![f32::NAN; ctx.n];
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
            * cec_theo_at(vk, vi, ctx.p("CEC_KAOL", i), ctx.p("CEC_ILL", i));
        let qv_bulk = cec_bulk * ctx.p("RHOG", i) * (1.0 - pt) / (100.0 * pt);
        let qv_eff = qv_bulk / (1.0 - swirr);
        qveff_o[i] = qv_eff as f32;

        let ct = 1.0 / rt_i;
        let cw = 1.0 / rw;
        let fstar = a / pt.powf(mstar);
        let b = juhasz_b(temp_c, rw).max(0.0);

        // Iterate SwT^n*/F*·(Cw + B·Qv_eff/SwT) = Ct, seeded with the Archie-like value.
        let mut sw = limit((fstar * ct / cw).powf(1.0 / nstar), 0.01, 1.0);
        // SB-SAT-028: a solver that exhausts its iteration budget MUST return null, never the
        // last iterate. A partial iterate is a finite saturation in the right range, so it is
        // indistinguishable from a converged answer on the log — it is read, mapped and booked.
        // `gascorr` (modules.rs) already refuses to write its 20th pass for the same reason,
        // calling it "an internally inconsistent triple masquerading as a converged answer";
        // this is SandiBumi's own method finally doing what its vendor-derived one always did.
        let mut converged = false;
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
                converged = true;
                break;
            }
            sw = next;
        }
        // A non-converged sample stays MISSING rather than shipping its last iterate.
        if sw.is_nan() || !converged {
            continue;
        }
        // SB-SAT-025: the diagnostic is the converged evaluation UNPROJECTED — the fixed point
        // the iteration landed on, without the final [0,1] clamp. Interior fixed points are
        // unchanged; a solve that converged AT the bound shows how far past it the model reads.
        let denom = cw + b * qv_eff / sw.max(1e-6);
        let sw_raw = (fstar * ct / denom).powf(1.0 / nstar);
        swt_raw_o[i] = sw_raw as f32;
        swt_o[i] = sw as f32;

        let cb = cbw[i] as f64;
        if !cb.is_nan() && pt > cb {
            let swb = limit(cb / pt, 0.0, 0.99);
            swe_raw_o[i] = ((sw_raw - swb) / (1.0 - swb)) as f32;
            swe_o[i] = limit((sw - swb) / (1.0 - swb), 0.0, 1.0) as f32;
        } else {
            swe_raw_o[i] = sw_raw as f32;
            swe_o[i] = sw as f32;
        }
    }

    HashMap::from([
        ("SWT_IMTS".to_string(), swt_raw_o),
        ("SWE_IMTS".to_string(), swe_raw_o),
        ("SWT".to_string(), swt_o),
        ("SWE".to_string(), swe_o),
        ("QVEFF".to_string(), qveff_o),
    ])
}

// ---------------------------------------------------------------------------
// RtC CALIBRATION — fit A_CAP / B_QV / C0 to the user's own water zone
// ---------------------------------------------------------------------------
//
// `sw_rtc`'s doc string has always told the user to "recalibrate per field from water-zone
// excess conductivity". Until now the app gave them no way to do it, so in practice everybody
// ran one study's coefficients on every field — and a foreign excess-conductivity calibration
// does not announce itself: it produces a smooth, plausible Sw curve that is simply wrong,
// usually optimistic, in exactly the low-contrast pay the method exists to unlock.
//
// The regression is the ALGEBRAIC INVERSE of the module's own equation, not a re-derivation.
// `sw_rtc` computes  Sw = [Rw·(1/Rt − Cex)/φt^M]^(1/N)  with  Cex = (a·CAPBW + b·Qv + c)·φt·RSF.
// Set Sw = 1 — which is what "water zone" means — and the measured excess falls straight out:
//
//     Cex_measured = 1/Rt − φt^M / Rw
//     y = Cex_measured / (φt·RSF) = a·CAPBW + b·Qv + c        <- ordinary least squares in 3
//
// Deriving it this way rather than from the method note's baseline line guarantees the fit and
// the run can never disagree: any future change to the saturation equation breaks this
// derivation visibly instead of leaving a calibration that quietly no longer inverts it.
//
// **RSF is held FIXED and is not fitted.** It multiplies the whole bracket, so (a, b, c, RSF)
// are not jointly identifiable from this regression — scale RSF and the fit absorbs it exactly.
// The returned coefficients belong to the RSF they were fitted with; changing RSF afterwards
// invalidates them, and the result says so.

use crate::equations::fetch_curve_frame;
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Deserialize)]
pub struct RtcFitRequest {
    pub well_ids: Vec<String>,
    /// Deep resistivity, total porosity and capillary-bound water. Defaults in the dialog
    /// mirror `sw_rtc`'s own inputs so the fit and the run see the same curves.
    pub rt_curve: String,
    pub phit_curve: String,
    pub capbw_curve: String,
    /// Optional Qv log; when absent Qv is built from `cec`/`rhog`, exactly as the module does.
    #[serde(default)]
    pub qv_curve: String,
    #[serde(default)]
    pub cec: f64,
    #[serde(default = "default_rhog")]
    pub rhog: f64,
    /// The Archie parameters that DEFINE the clean baseline. These must be the ones the run
    /// will use — a calibration fitted against a different Rw is a calibration for a different
    /// rock.
    pub rw: f64,
    pub m: f64,
    /// Held fixed through the fit (see the note above).
    pub rsf: f64,
    /// The water-bearing interval. **At least one of these must be given** — see `run_rtc_fit`.
    #[serde(default)]
    pub depth_min: Option<f64>,
    #[serde(default)]
    pub depth_max: Option<f64>,
    /// Optional flag curve marking the wet samples (non-zero = use).
    #[serde(default)]
    pub wet_flag_curve: String,
}

fn default_rhog() -> f64 {
    2.65
}

#[derive(Debug, Clone, Serialize)]
pub struct RtcFitPoint {
    pub well_id: String,
    pub depth: f64,
    pub capbw: f64,
    pub qv: f64,
    /// Measured excess conductivity, normalized by φt·RSF — the regression's y.
    pub y: f64,
    /// The fitted model at this sample, for a QC scatter of fitted vs measured.
    pub y_fit: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RtcFitResult {
    pub a_cap: f64,
    pub b_qv: f64,
    pub c0: f64,
    /// Echoed back: the coefficients are only valid for THIS RSF.
    pub rsf_used: f64,
    pub r2: f64,
    /// RMS of (y − y_fit) in the regression's own units.
    pub rms: f64,
    pub n_points: usize,
    pub n_wells: usize,
    /// The wells that actually contributed a sample, in id form. Reported SEPARATELY from
    /// `points` on purpose: `points` is decimated for the UI, so a well can vanish from it
    /// entirely, and "apply this calibration to the wells it was fitted from" must never be
    /// derived from a display sample.
    pub wells_fitted: Vec<String>,
    pub points: Vec<RtcFitPoint>,
    /// (reason, count) for every candidate sample not fitted — never drop one silently.
    pub excluded: Vec<(String, usize)>,
    pub notes: Vec<String>,
    pub error: Option<String>,
}

fn rtc_err(msg: &str) -> RtcFitResult {
    RtcFitResult {
        a_cap: f64::NAN,
        b_qv: f64::NAN,
        c0: f64::NAN,
        rsf_used: f64::NAN,
        r2: f64::NAN,
        rms: f64::NAN,
        n_points: 0,
        n_wells: 0,
        wells_fitted: vec![],
        points: vec![],
        excluded: vec![],
        notes: vec![],
        error: Some(msg.to_string()),
    }
}

/// At most this many scatter points go back to the UI (uniformly decimated beyond it).
const MAX_RTC_POINTS: usize = 4000;

/// Solves a symmetric 3x3 normal-equation system by Gaussian elimination with partial
/// pivoting. Returns `None` when the system is singular to working precision — which happens
/// for a real and important reason (see `run_rtc_fit`), so it must be reported, never patched
/// over with a ridge term that would silently invent a coefficient.
fn solve3(mut a: [[f64; 3]; 3], mut b: [f64; 3]) -> Option<[f64; 3]> {
    // Scale-aware singularity threshold: an absolute epsilon would call a well-conditioned
    // system with tiny conductivities singular.
    let scale = a.iter().flatten().fold(0.0f64, |m, v| m.max(v.abs()));
    if scale <= 0.0 || !scale.is_finite() {
        return None;
    }
    let eps = 1e-12 * scale;
    for col in 0..3 {
        let piv = (col..3).max_by(|&r1, &r2| {
            a[r1][col].abs().partial_cmp(&a[r2][col].abs()).unwrap_or(std::cmp::Ordering::Equal)
        })?;
        if a[piv][col].abs() <= eps {
            return None;
        }
        a.swap(col, piv);
        b.swap(col, piv);
        for row in (col + 1)..3 {
            let f = a[row][col] / a[col][col];
            for k in col..3 {
                a[row][k] -= f * a[col][k];
            }
            b[row] -= f * b[col];
        }
    }
    let mut x = [0.0f64; 3];
    for row in (0..3).rev() {
        let mut s = b[row];
        for k in (row + 1)..3 {
            s -= a[row][k] * x[k];
        }
        x[row] = s / a[row][row];
    }
    if x.iter().all(|v| v.is_finite()) {
        Some(x)
    } else {
        None
    }
}

/// Fits the RtC excess-conductivity coefficients to the user's own water-bearing rock.
pub fn run_rtc_fit(db: &Mutex<Connection>, req: &RtcFitRequest) -> RtcFitResult {
    if req.well_ids.is_empty() {
        return rtc_err("select at least one well");
    }
    // THE guard of this whole feature. The fit assumes Sw = 1; over hydrocarbon-bearing rock
    // the measured conductivity deficit is HYDROCARBON, and fitting it would hand that deficit
    // to the clay and capillary terms — producing a calibration that erases pay wherever the
    // clay is high. Refuse rather than fit the whole well.
    let has_interval = req.depth_min.is_some() || req.depth_max.is_some();
    let flag = req.wet_flag_curve.trim().to_uppercase();
    if !has_interval && flag.is_empty() {
        return rtc_err(
            "select the WATER-BEARING interval (a depth range or a wet-flag curve). The fit \
             assumes Sw = 1: over hydrocarbon-bearing rock it would attribute the hydrocarbon \
             resistivity to clay and capillary water, and quietly erase pay",
        );
    }
    if req.rw <= 0.0 || !req.rw.is_finite() {
        return rtc_err("Rw must be positive — it sets the clean-rock baseline the excess is measured against");
    }
    if !(req.m.is_finite() && req.m > 0.0) {
        return rtc_err("M must be positive");
    }
    if !(req.rsf.is_finite() && req.rsf > 0.0) {
        return rtc_err("RSF must be positive (it scales the whole excess term)");
    }

    let rt_n = req.rt_curve.trim().to_uppercase();
    let phit_n = req.phit_curve.trim().to_uppercase();
    let cap_n = req.capbw_curve.trim().to_uppercase();
    let qv_n = req.qv_curve.trim().to_uppercase();
    if rt_n.is_empty() || phit_n.is_empty() || cap_n.is_empty() {
        return rtc_err("RT, PHIT and CAPBW curves are all required");
    }

    let mut pts: Vec<RtcFitPoint> = Vec::new();
    let (mut ex_incomplete, mut ex_range, mut ex_flag, mut ex_negative, mut ex_phit) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    let mut empty_wells: Vec<String> = Vec::new();
    let mut wells_used: std::collections::BTreeSet<String> = Default::default();

    {
        let conn = db.lock().unwrap();
        let mut names = vec![rt_n.clone(), phit_n.clone(), cap_n.clone()];
        if !qv_n.is_empty() {
            names.push(qv_n.clone());
        }
        if !flag.is_empty() {
            names.push(flag.clone());
        }
        for well_id in &req.well_ids {
            let before = pts.len();
            'well: {
                let Ok((depth, cols)) = fetch_curve_frame(&conn, well_id, &names) else { break 'well };
                let (Some(rtv), Some(ptv), Some(capv)) =
                    (cols.get(&rt_n), cols.get(&phit_n), cols.get(&cap_n))
                else {
                    break 'well;
                };
                let qvv = cols.get(&qv_n);
                let flv = cols.get(&flag);
                let n = depth.len().min(rtv.len()).min(ptv.len()).min(capv.len());
                for i in 0..n {
                    let d = depth[i] as f64;
                    if let Some(lo) = req.depth_min {
                        if d < lo {
                            ex_range += 1;
                            continue;
                        }
                    }
                    if let Some(hi) = req.depth_max {
                        if d > hi {
                            ex_range += 1;
                            continue;
                        }
                    }
                    if let Some(f) = flv {
                        let v = f.get(i).copied().unwrap_or(f32::NAN) as f64;
                        // A NaN flag is NOT wet. "Unknown" must never be read as "yes" when the
                        // whole point of the flag is to keep hydrocarbon out of the fit.
                        if !(v.is_finite() && v != 0.0) {
                            ex_flag += 1;
                            continue;
                        }
                    }
                    let (rt, pt, cap) = (rtv[i] as f64, ptv[i] as f64, capv[i] as f64);
                    if !(rt.is_finite() && pt.is_finite() && cap.is_finite()) || rt <= 0.0 {
                        ex_incomplete += 1;
                        continue;
                    }
                    if pt <= 0.0 {
                        ex_phit += 1;
                        continue;
                    }
                    let qv_log = qvv.and_then(|q| q.get(i)).map(|v| *v as f64).unwrap_or(f64::NAN);
                    let qv = qv_at(qv_log, pt, req.cec, req.rhog);

                    // Measured excess = total conductivity minus what clean Archie predicts at
                    // Sw = 1 (this is sw_rtc's own equation inverted — see the header note).
                    let cex = 1.0 / rt - pt.powf(req.m) / req.rw;
                    if !cex.is_finite() {
                        ex_incomplete += 1;
                        continue;
                    }
                    if cex <= 0.0 {
                        // The rock reads MORE resistive than clean Archie says water-filled rock
                        // can be. There is no excess conductivity to explain, so the sample
                        // carries no information about the clay/capillary terms — and including
                        // it would drag the intercept negative. Usually means Rw is too fresh
                        // for this interval, or the interval is not actually wet.
                        ex_negative += 1;
                        continue;
                    }
                    pts.push(RtcFitPoint {
                        well_id: well_id.clone(),
                        depth: d,
                        capbw: cap,
                        qv,
                        y: cex / (pt * req.rsf),
                        y_fit: f64::NAN, // filled after the solve
                    });
                }
            }
            if pts.len() == before {
                empty_wells.push(well_id.clone());
            } else {
                wells_used.insert(well_id.clone());
            }
        }
    }

    if pts.len() < 4 {
        return rtc_err(&format!(
            "only {} usable water-zone sample(s) — need at least 4 to fit three coefficients \
             with anything left over to judge the fit by",
            pts.len()
        ));
    }

    // Normal equations for y = a·cap + b·qv + c.
    let mut ata = [[0.0f64; 3]; 3];
    let mut aty = [0.0f64; 3];
    for p in &pts {
        let row = [p.capbw, p.qv, 1.0];
        for r in 0..3 {
            for c in 0..3 {
                ata[r][c] += row[r] * row[c];
            }
            aty[r] += row[r] * p.y;
        }
    }

    let mut notes: Vec<String> = Vec::new();
    // Is Qv actually varying? With no QV log and no CEC it is identically zero, and its column
    // is then collinear with nothing at all — the solve is singular. Detect it HERE so the
    // message names the cause, instead of returning a generic singular-matrix error.
    let qv_mean = pts.iter().map(|p| p.qv).sum::<f64>() / pts.len() as f64;
    let qv_var = pts.iter().map(|p| (p.qv - qv_mean).powi(2)).sum::<f64>() / pts.len() as f64;
    let cap_mean = pts.iter().map(|p| p.capbw).sum::<f64>() / pts.len() as f64;
    let cap_var = pts.iter().map(|p| (p.capbw - cap_mean).powi(2)).sum::<f64>() / pts.len() as f64;

    let (a_cap, b_qv, c0) = if qv_var <= 1e-18 {
        // Drop the Qv term rather than invent one: fit y = a·cap + c and report b = 0 plainly.
        notes.push(
            "Qv does not vary over the fitted samples (no QV log and no CEC given), so the \
             clay-chemistry term could not be fitted and is reported as 0 — the capillary term \
             has absorbed whatever constant clay conductivity is present"
                .into(),
        );
        if cap_var <= 1e-18 {
            return rtc_err(
                "neither CAPBW nor Qv varies over the fitted samples — there is nothing to \
                 regress against. Check the CAPBW curve resolves to data over this interval",
            );
        }
        let n = pts.len() as f64;
        let (sxx, sxy) = (ata[0][0], aty[0]);
        let (sx, sy) = (ata[0][2], aty[2]);
        let den = n * sxx - sx * sx;
        if den.abs() <= 1e-18 {
            return rtc_err("the CAPBW regression is singular — check the curve varies over the interval");
        }
        ((n * sxy - sx * sy) / den, 0.0, (sxx * sy - sx * sxy) / den)
    } else {
        match solve3(ata, aty) {
            Some([a, b, c]) => (a, b, c),
            None => {
                return rtc_err(
                    "the regression is singular — CAPBW and Qv are collinear over these samples \
                     (they carry the same information here), so the two paths cannot be \
                     separated. Widen the interval, add wells, or fit without the Qv log",
                )
            }
        }
    };

    // Fitted values, R² and RMS.
    let y_mean = pts.iter().map(|p| p.y).sum::<f64>() / pts.len() as f64;
    let (mut ss_res, mut ss_tot) = (0.0f64, 0.0f64);
    for p in &mut pts {
        p.y_fit = a_cap * p.capbw + b_qv * p.qv + c0;
        ss_res += (p.y - p.y_fit).powi(2);
        ss_tot += (p.y - y_mean).powi(2);
    }
    let r2 = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { f64::NAN };
    let rms = (ss_res / pts.len() as f64).sqrt();

    let n_points = pts.len();
    let n_wells = wells_used.len();

    // A scoped well that contributed nothing must be named — a "field" calibration fitted from
    // a subset of the scoped wells is not a field calibration.
    if !empty_wells.is_empty() {
        notes.push(format!(
            "{} scoped well(s) contributed no water-zone samples and are not in this fit",
            empty_wells.len()
        ));
    }
    if c0 < 0.0 {
        notes.push(
            "the intercept is negative, which the study's own calibration also is — at low \
             CAPBW and Qv the model then predicts no excess, and sw_rtc clamps it to zero rather \
             than adding conductivity"
                .into(),
        );
    }
    if r2.is_finite() && r2 < 0.3 {
        notes.push(format!(
            "R2 = {r2:.2} is low: the excess conductivity here is not well explained by CAPBW \
             and Qv. Check the interval really is wet and that Rw is right for it before using \
             these coefficients"
        ));
    }
    notes.push(format!(
        "coefficients are valid for RSF = {} only — RSF multiplies the whole bracket, so the \
         four are not jointly identifiable and changing RSF afterwards invalidates them",
        req.rsf
    ));

    let excluded: Vec<(String, usize)> = [
        ("outside the selected interval".to_string(), ex_range),
        ("not flagged wet".to_string(), ex_flag),
        ("incomplete or non-physical inputs".to_string(), ex_incomplete),
        ("PHIT <= 0".to_string(), ex_phit),
        ("no excess to explain (Rt above the clean-water baseline)".to_string(), ex_negative),
    ]
    .into_iter()
    .filter(|(_, n)| *n > 0)
    .collect();

    // Decimate for the UI only — the fit above used every sample.
    if pts.len() > MAX_RTC_POINTS {
        let step = pts.len() as f64 / MAX_RTC_POINTS as f64;
        pts = (0..MAX_RTC_POINTS).map(|k| pts[((k as f64) * step) as usize].clone()).collect();
    }

    RtcFitResult {
        a_cap,
        b_qv,
        c0,
        rsf_used: req.rsf,
        r2,
        rms,
        n_points,
        n_wells,
        wells_fitted: wells_used.into_iter().collect(),
        points: pts,
        excluded,
        notes,
        error: None,
    }
}

// ---------------------------------------------------------------------------
// S-FACTOR CALIBRATION — fit S to the user's own laboratory CEC measurements
// ---------------------------------------------------------------------------
//
// `sw_imts`'s S factor has exactly the problem `sw_rtc`'s coefficients had: it is defined as a
// measurement — S = lab CEC / XRD-theoretical CEC (`docs/method_lrlc_rtc_imts.md`, IMTS §1) — and
// the app shipped a placeholder for it. S multiplies the entire clay-charge term, so a wrong S
// scales Qv_eff directly and moves SwT with nothing on the log to show for it.
//
// Same discipline as the RtC fit. The regression is the ALGEBRAIC INVERSE of the module's own
// line — `sw_imts` computes `cec_bulk = S · cec_theo_at(vk, vi, CEC_KAOL, CEC_ILL)`, so
//
//     S = CEC_lab / cec_theo_at(...)          <- least squares THROUGH THE ORIGIN in one unknown
//
// and `cec_theo_at` is the very function the run calls, so the two cannot drift apart.
//
// **Through the origin, no intercept.** S is defined as a pure scaling factor. An intercept
// would assert measurable cation exchange where the clay model says there is no clay — a
// different physical claim, and one the module's equation has nowhere to put.
//
// **Least squares, not the mean of the per-plug ratios.** Through-origin OLS weights each plug
// by its clay content, which is right: those are the plugs where Qv actually drives the answer,
// and on a nearly clean plug the ratio is mostly measurement noise divided by a small number.
// The median ratio is reported ALONGSIDE it, because the two agree only when S really is
// constant across the clay range — a wide gap between them is the diagnosis that it is not.
//
// **The clay must come from the curves the run will use, not from the XRD table.** This is the
// trap. If S is calibrated against XRD weight fractions and then applied to a VDCL-derived
// VKAOL curve, S is wrong by the ratio between those two estimates of clay — silently, because
// both look like clay volumes. So the fit reads VKAOL/VILL through the normal curve resolution,
// exactly as the module does.
//
// **S and the literature CEC constants are not jointly identifiable**, the same way RSF and the
// RtC coefficients are not: S multiplies (V·CEC_KAOL + V·CEC_ILL), so scaling the constants and
// scaling S are the same operation. The fitted S belongs to the constants it was fitted with,
// and the result says so.

use crate::db;

#[derive(Debug, Clone, Deserialize)]
pub struct SFactorFitRequest {
    pub well_ids: Vec<String>,
    /// Point dataset holding the laboratory CEC measurements — "CEC", or "CORE" when they
    /// arrived as an extra column on a core table. Read from the ACTIVE delivery of that
    /// dataset, like every other point-data reader.
    pub cec_dataset: String,
    /// Item name within that dataset carrying the CEC value (meq/100g).
    pub cec_item: String,
    /// Clay curves, resolved the way `sw_imts` resolves them. See the header note on why these
    /// must be the run's curves and not the XRD table the lab CEC came from.
    pub vkaol_curve: String,
    #[serde(default)]
    pub vill_curve: String,
    /// Literature CEC constants. Held FIXED — see the header note on identifiability.
    #[serde(default = "default_cec_kaol")]
    pub cec_kaol: f64,
    #[serde(default = "default_cec_ill")]
    pub cec_ill: f64,
    /// How far a plug depth may sit from the nearest log sample and still be paired with it.
    #[serde(default = "default_depth_tol")]
    pub depth_tol: f64,
}

fn default_cec_kaol() -> f64 {
    8.0
}
fn default_cec_ill() -> f64 {
    25.0
}
/// One standard 6-inch log sample, in metres. A plug quoted to the centimetre lands inside one
/// sample of its true depth once the core is depth-shifted to the log; anything looser is
/// pairing a measurement with rock it did not come from.
fn default_depth_tol() -> f64 {
    0.15
}

#[derive(Debug, Clone, Serialize)]
pub struct SFactorPoint {
    pub well_id: String,
    /// Depth of the plug, as delivered.
    pub depth: f64,
    /// Depth of the log sample it was paired with, so a suspicious pairing is visible.
    pub log_depth: f64,
    pub vkaol: f64,
    pub vill: f64,
    /// Theoretical bulk CEC from the clay model — the regression's x.
    pub cec_theo: f64,
    /// Measured laboratory CEC — the regression's y.
    pub cec_lab: f64,
    /// This plug's own ratio, for the scatter.
    pub ratio: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SFactorFitResult {
    /// Through-origin least squares — the value to put in S_FACTOR.
    pub s_factor: f64,
    /// Median of the per-plug ratios. Reported so a gap between the two can be seen.
    pub s_median_ratio: f64,
    /// P10/P90 of the per-plug ratios — how far the individual plugs disagree about S. This,
    /// not the median-vs-fit gap, is the real drift detector: the fit weights by clay content
    /// and the median does not, but both are central values, so the two can only ever differ by
    /// as much as the ratio changes between the median plug and the clay-weighted one. The
    /// spread has no such ceiling.
    pub ratio_p10: f64,
    pub ratio_p90: f64,
    /// R² about the MEAN (the conventional, comparable one — see the note it can go negative).
    pub r2: f64,
    /// RMS of (CEC_lab − S·CEC_theo), in meq/100g.
    pub rms: f64,
    pub n_points: usize,
    pub n_wells: usize,
    /// The wells that actually contributed a plug. See `RtcFitResult::wells_fitted`.
    pub wells_fitted: Vec<String>,
    /// Echoed back: S is only valid for these constants.
    pub cec_kaol_used: f64,
    pub cec_ill_used: f64,
    pub points: Vec<SFactorPoint>,
    pub excluded: Vec<(String, usize)>,
    pub notes: Vec<String>,
    pub error: Option<String>,
}

fn s_err(msg: &str) -> SFactorFitResult {
    SFactorFitResult {
        s_factor: f64::NAN,
        s_median_ratio: f64::NAN,
        ratio_p10: f64::NAN,
        ratio_p90: f64::NAN,
        r2: f64::NAN,
        rms: f64::NAN,
        n_points: 0,
        n_wells: 0,
        wells_fitted: vec![],
        cec_kaol_used: f64::NAN,
        cec_ill_used: f64::NAN,
        points: vec![],
        excluded: vec![],
        notes: vec![],
        error: Some(msg.to_string()),
    }
}

/// Index of the depth nearest `target` in an ASCENDING slice, or `None` when the slice is empty.
fn nearest_depth(depths: &[f32], target: f64) -> Option<usize> {
    if depths.is_empty() {
        return None;
    }
    let pos = depths.partition_point(|d| (*d as f64) < target);
    let mut best = pos.min(depths.len() - 1);
    if pos > 0 {
        let prev = pos - 1;
        if (depths[prev] as f64 - target).abs() <= (depths[best] as f64 - target).abs() {
            best = prev;
        }
    }
    Some(best)
}

/// Fits the IMTS CEC scaling factor S to the user's own laboratory CEC measurements.
pub fn run_s_factor_fit(db_mx: &Mutex<Connection>, req: &SFactorFitRequest) -> SFactorFitResult {
    if req.well_ids.is_empty() {
        return s_err("select at least one well");
    }
    let dataset = req.cec_dataset.trim().to_uppercase();
    let item = req.cec_item.trim().to_uppercase();
    if dataset.is_empty() || item.is_empty() {
        return s_err("name the point dataset and the item holding the laboratory CEC values");
    }
    let vk_n = req.vkaol_curve.trim().to_uppercase();
    let vi_n = req.vill_curve.trim().to_uppercase();
    if vk_n.is_empty() && vi_n.is_empty() {
        return s_err("at least one clay curve is required — S scales the clay charge, so with no clay there is nothing to scale");
    }
    if !(req.cec_kaol.is_finite() && req.cec_ill.is_finite())
        || req.cec_kaol < 0.0
        || req.cec_ill < 0.0
    {
        return s_err("the literature CEC constants must be finite and non-negative");
    }
    if req.cec_kaol <= 0.0 && req.cec_ill <= 0.0 {
        return s_err("both literature CEC constants are zero — the clay model then predicts no exchange capacity anywhere and S cannot be defined");
    }
    if !(req.depth_tol.is_finite() && req.depth_tol > 0.0) {
        return s_err("the depth tolerance must be positive");
    }

    let mut pts: Vec<SFactorPoint> = Vec::new();
    let (mut ex_nomatch, mut ex_noclay, mut ex_nolab, mut ex_noclaydata) =
        (0usize, 0usize, 0usize, 0usize);
    let mut items_seen: std::collections::BTreeSet<String> = Default::default();
    let mut wells_used: std::collections::BTreeSet<String> = Default::default();
    let mut empty_wells: Vec<String> = Vec::new();
    let mut any_rows = false;

    {
        let conn = db_mx.lock().unwrap();
        let mut names: Vec<String> = Vec::new();
        if !vk_n.is_empty() {
            names.push(vk_n.clone());
        }
        if !vi_n.is_empty() {
            names.push(vi_n.clone());
        }
        for well_id in &req.well_ids {
            let before = pts.len();
            'well: {
                let Ok(aux) = db::list_aux_data(&conn, well_id, Some(&dataset)) else { break 'well };
                if aux.is_empty() {
                    break 'well;
                }
                any_rows = true;
                for r in &aux {
                    items_seen.insert(r.item.to_uppercase());
                }
                let Ok((depth, cols)) = fetch_curve_frame(&conn, well_id, &names) else { break 'well };
                if depth.is_empty() {
                    break 'well;
                }
                let vkv = cols.get(&vk_n);
                let viv = cols.get(&vi_n);
                if vkv.is_none() && viv.is_none() {
                    break 'well;
                }

                for r in aux.iter().filter(|r| r.item.eq_ignore_ascii_case(&item)) {
                    let d = r.depth_top as f64;
                    // A lab CEC is measured on ONE plug. An interval row (depth_base present) is
                    // anchored at its middle, the same convention the point-data tracks use.
                    let d = match r.depth_base {
                        Some(b) if (b as f64) > d => 0.5 * (d + b as f64),
                        _ => d,
                    };
                    let lab = match r.value_num {
                        Some(v) if (v as f64).is_finite() && v >= 0.0 => v as f64,
                        _ => {
                            ex_nolab += 1;
                            continue;
                        }
                    };
                    let Some(idx) = nearest_depth(&depth, d) else {
                        ex_nomatch += 1;
                        continue;
                    };
                    let ld = depth[idx] as f64;
                    if (ld - d).abs() > req.depth_tol {
                        // Never stretch to the nearest sample regardless of distance: a CEC
                        // paired with rock it was not cut from is a fabricated data point, and
                        // it would look exactly like a real one in the scatter.
                        ex_nomatch += 1;
                        continue;
                    }
                    let vk = vkv.and_then(|c| c.get(idx)).map(|v| *v as f64).unwrap_or(f64::NAN);
                    let vi = viv.and_then(|c| c.get(idx)).map(|v| *v as f64).unwrap_or(f64::NAN);
                    // A missing clay curve reads as ZERO clay of that mineral (what the module
                    // does), but a plug where BOTH are missing carries no clay information at all
                    // and must not be read as a clean plug.
                    if vk.is_nan() && vi.is_nan() {
                        ex_noclaydata += 1;
                        continue;
                    }
                    let vk = if vk.is_nan() { 0.0 } else { limit(vk, 0.0, 1.0) };
                    let vi = if vi.is_nan() { 0.0 } else { limit(vi, 0.0, 1.0) };
                    let theo = cec_theo_at(vk, vi, req.cec_kaol, req.cec_ill);
                    if !(theo.is_finite() && theo > 0.0) {
                        // The clay model says there is no clay here. A ratio would divide by
                        // zero, and a lab CEC that is NOT zero at such a depth is evidence
                        // against the clay curves — not a point that can set a scaling factor.
                        ex_noclay += 1;
                        continue;
                    }
                    pts.push(SFactorPoint {
                        well_id: well_id.clone(),
                        depth: d,
                        log_depth: ld,
                        vkaol: vk,
                        vill: vi,
                        cec_theo: theo,
                        cec_lab: lab,
                        ratio: lab / theo,
                    });
                }
            }
            if pts.len() == before {
                empty_wells.push(well_id.clone());
            } else {
                wells_used.insert(well_id.clone());
            }
        }
    }

    if pts.is_empty() && any_rows && !items_seen.contains(&item) {
        let list: Vec<&str> = items_seen.iter().map(|s| s.as_str()).take(24).collect();
        return s_err(&format!(
            "no item named '{item}' in the {dataset} data of these wells. Items present: {}",
            if list.is_empty() { "(none)".to_string() } else { list.join(", ") }
        ));
    }
    if pts.len() < 3 {
        return s_err(&format!(
            "only {} usable plug(s) — need at least 3, so the scatter around S can be judged \
             rather than taken on trust from a single measurement",
            pts.len()
        ));
    }

    // Through-origin least squares: S = Σ(x·y) / Σ(x²).
    let sxy: f64 = pts.iter().map(|p| p.cec_theo * p.cec_lab).sum();
    let sxx: f64 = pts.iter().map(|p| p.cec_theo * p.cec_theo).sum();
    if !(sxx.is_finite() && sxx > 0.0) {
        return s_err("the theoretical CEC is zero across every paired plug — check the clay curves resolve to data at the plug depths");
    }
    let s_factor = sxy / sxx;

    let mut ratios: Vec<f32> = pts.iter().map(|p| p.ratio as f32).collect();
    ratios.sort_by(|a, b| a.partial_cmp(b).expect("ratios are finite by construction"));
    let s_median_ratio = crate::distribution::percentile(&ratios, 50.0) as f64;
    let ratio_p10 = crate::distribution::percentile(&ratios, 10.0) as f64;
    let ratio_p90 = crate::distribution::percentile(&ratios, 90.0) as f64;

    // R² about the MEAN — the conventional definition, deliberately NOT the no-intercept variant
    // (which measures against zero and flatters every through-origin fit into looking excellent).
    let y_mean = pts.iter().map(|p| p.cec_lab).sum::<f64>() / pts.len() as f64;
    let (mut ss_res, mut ss_tot) = (0.0f64, 0.0f64);
    for p in &pts {
        let resid = p.cec_lab - s_factor * p.cec_theo;
        ss_res += resid * resid;
        ss_tot += (p.cec_lab - y_mean).powi(2);
    }
    let r2 = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { f64::NAN };
    let rms = (ss_res / pts.len() as f64).sqrt();

    let n_points = pts.len();
    let n_wells = wells_used.len();
    let mut notes: Vec<String> = Vec::new();

    if !empty_wells.is_empty() {
        notes.push(format!(
            "{} scoped well(s) contributed no paired plugs and are not in this fit",
            empty_wells.len()
        ));
    }
    if s_factor > 1.0 {
        // The method's own expectation is S < 1: measured lab CEC runs BELOW the XRD-theoretical
        // value. Above 1 the clay model is under-calling exchange capacity, and the usual cause
        // is a mineral it does not carry — smectite runs 80-150 meq/100g against illite's 25, so
        // even a few percent of it dwarfs the modelled charge and S absorbs the difference. An S
        // carrying a missing mineral is then wrong at every depth where that mineral's fraction
        // differs from the cored plugs'.
        notes.push(format!(
            "S = {s_factor:.3} is above 1, where the method expects lab CEC to sit BELOW the \
             XRD-theoretical value. The clay model is under-calling exchange capacity — most \
             often a CEC-active mineral it does not carry (smectite is 80-150 meq/100g against \
             illite's 25). S is absorbing that, and will be wrong wherever its fraction differs \
             from these plugs"
        ));
    }
    // THE drift detector. Two central values (the clay-weighted fit and the plain median) can
    // only differ by as much as the ratio changes between the median plug and the clay-weighted
    // one — on a wide clay range with a linear drift that is barely 30%, so a gap threshold
    // alone misses real drift. The SPREAD of the individual ratios has no such ceiling.
    if ratio_p10.is_finite() && ratio_p90.is_finite() && ratio_p10 > 0.0 && ratio_p90 / ratio_p10 > 2.0
    {
        notes.push(format!(
            "the plugs disagree about S: their own ratios run {ratio_p10:.3} (P10) to \
             {ratio_p90:.3} (P90), a factor of {:.1}. No single S describes them. Either S \
             genuinely drifts with clay content, or the lean plugs are carrying measurement \
             scatter — a small absolute CEC divided by a small modelled clay volume is a noisy \
             ratio. Look at the scatter before quoting one number",
            ratio_p90 / ratio_p10
        ));
    }
    // A systematic gap on top of that spread says the disagreement tracks clay content, which
    // is the part that decides WHICH plugs the single fitted S actually suits.
    if s_median_ratio.is_finite() && s_factor.is_finite() && s_factor > 0.0 {
        let gap = (s_median_ratio / s_factor).max(s_factor / s_median_ratio);
        if gap.is_finite() && gap > 1.25 {
            notes.push(format!(
                "the median per-plug ratio is {s_median_ratio:.3} against a fitted S of \
                 {s_factor:.3}. The fit is weighted toward the clayey plugs and the median is \
                 not, so a gap this wide means the disagreement is systematic with clay content: \
                 the fitted S suits the clay-rich rock, which is where Qv drives the answer, and \
                 over-corrects nothing in the clean sand where it barely matters"
            ));
        }
    }
    if r2.is_finite() && r2 < 0.0 {
        notes.push(format!(
            "R2 = {r2:.2} is negative: proportional to clay describes the lab CEC WORSE than a \
             flat average would. The clay curves are not tracking exchange capacity here, and no \
             single S will fix that"
        ));
    } else if r2.is_finite() && r2 < 0.3 {
        notes.push(format!(
            "R2 = {r2:.2} is low — the clay curves explain little of the CEC variation. Check \
             they resolve to real data at the plug depths and that the core is depth-shifted to \
             the log"
        ));
    }
    notes.push(format!(
        "S is valid for CEC_KAOL = {} and CEC_ILL = {} only — S multiplies those constants, so \
         the three are not jointly identifiable and changing them afterwards invalidates it",
        req.cec_kaol, req.cec_ill
    ));

    let excluded: Vec<(String, usize)> = [
        ("no log sample within the depth tolerance".to_string(), ex_nomatch),
        ("clay curves carry no data at that depth".to_string(), ex_noclaydata),
        ("clay model says no clay there (no ratio to take)".to_string(), ex_noclay),
        ("laboratory CEC missing or negative".to_string(), ex_nolab),
    ]
    .into_iter()
    .filter(|(_, n)| *n > 0)
    .collect();

    if pts.len() > MAX_RTC_POINTS {
        let step = pts.len() as f64 / MAX_RTC_POINTS as f64;
        pts = (0..MAX_RTC_POINTS).map(|k| pts[((k as f64) * step) as usize].clone()).collect();
    }

    SFactorFitResult {
        s_factor,
        s_median_ratio,
        ratio_p10,
        ratio_p90,
        r2,
        rms,
        n_points,
        n_wells,
        wells_fitted: wells_used.into_iter().collect(),
        cec_kaol_used: req.cec_kaol,
        cec_ill_used: req.cec_ill,
        points: pts,
        excluded,
        notes,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ArgKind;

    /// SB-SAT-028 (P0). `12_saturation.md:1399-1410` — a saturation solver that fails to converge
    /// within its iteration budget MUST return null for that sample, and MUST NOT emit the last
    /// iterate.
    ///
    /// A partial iterate is a finite saturation in the right range. It is not a visible error: it
    /// is a plausible number a petrophysicist reads, maps and books, indistinguishable on the log
    /// from one the equation actually solved. That is why no pre-existing test caught it — every
    /// assertion about finiteness or bounds passes on a partial iterate.
    #[test]
    fn a_non_converged_imts_sample_is_missing_rather_than_its_last_iterate() {
        let spec = sw_imts_spec();

        // A — the ordinary path is untouched: a converging sample still carries its value. The
        // guard must refuse non-convergence, not refuse everything.
        let out = sw_imts(&ctx_with(
            vec![
                ("RT", vec![4.0]),
                ("PHIT", vec![0.25]),
                ("CBW", vec![0.03]),
                ("VKAOL", vec![0.10]),
                ("VILL", vec![0.05]),
            ],
            &spec,
            1,
        ));
        assert!(
            out["SWT_IMTS"][0].is_finite(),
            "a converging sample must keep its value, got {}",
            out["SWT_IMTS"][0]
        );

        // B — the structural guard. A behavioural non-convergent input could not be constructed:
        // the iteration is a contraction over the whole admissible parameter range, which is why
        // the defect survived. Rather than contrive one — or quietly drop the arm — the write is
        // pinned to sit behind the convergence flag, so removing the guard fails here.
        let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lrlc.rs"))
            .expect("lrlc.rs is readable");
        let production = source.split_once("#[cfg(test)]").map(|(a, _)| a).unwrap_or(&source);
        let guard = production
            .find("if sw.is_nan() || !converged {")
            .expect("the non-convergence guard is gone — sw_imts would ship its last iterate");
        let write = production
            .find("swt_o[i] = sw as f32;")
            .expect("sw_imts no longer writes SWT_IMTS");
        assert!(
            guard < write,
            "the convergence guard must come BEFORE the write, or a non-converged iterate still ships"
        );
        assert!(
            production.contains("converged = true;"),
            "nothing ever sets the convergence flag, so the guard would reject every sample"
        );
    }

    fn ctx_with(logs: Vec<(&str, Vec<f32>)>, spec: &ModuleSpec, n: usize) -> ModuleContext {
        let mut params = HashMap::new();
        let mut opts = HashMap::new();
        // CHARACTERIZATION INPUTS — these are the pre-SB-CORE-004 study fixtures that the
        // existing LRLC equation tests were written against. They are explicit test data now,
        // never shipping defaults. Source: the former manifests recorded in git immediately
        // before SB-CORE-004 and the equations/inputs named by each test below.
        let fixture_value = |name: &str| match (spec.name.as_str(), name) {
            ("sw_rtc", "RW") => 0.3,
            ("sw_rtc", "M") | ("sw_rtc", "N") => 2.0,
            ("sw_rtc", "A_CAP") => 0.45,
            ("sw_rtc", "B_QV") => 0.0057,
            ("sw_rtc", "C0") => -0.0071,
            ("sw_rtc", "RSF") => 2.25,
            ("sw_rtc", "CEC") => 0.0,
            ("sw_rtc", "RHOG") => 2.65,
            ("sw_imts", "RW") => 0.3,
            ("sw_imts", "TEMP_C") => 60.0,
            ("sw_imts", "A") => 1.0,
            ("sw_imts", "MSTAR") | ("sw_imts", "NSTAR") => 1.9,
            ("sw_imts", "S_FACTOR") => 0.5,
            ("sw_imts", "CEC_KAOL") => 8.0,
            ("sw_imts", "CEC_ILL") => 25.0,
            ("sw_imts", "RHOG") => 2.65,
            ("sw_imts", "SWIRR_DEF") => 0.2,
            _ => panic!("no explicit LRLC test fixture for {}.{name}", spec.name),
        };
        for arg in &spec.args {
            match arg.kind {
                ArgKind::Param => {
                    params.insert(arg.name.clone(), vec![fixture_value(&arg.name); n]);
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
        // SB-SAT-025 moved the clipped values to the plain pair (bit-identical); the method-
        // named curves are now unclipped diagnostics, which legitimately break this inequality
        // above 1 - that is exactly the out-of-range evidence they exist to carry.
        assert!(out["SWE"][0] <= out["SWT"][0], "SWE <= SWT");
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
        // SB-SAT-025 moved the clipped values to the plain pair (bit-identical); the method-
        // named curves are now unclipped diagnostics, which legitimately break this inequality
        // above 1 - that is exactly the out-of-range evidence they exist to carry.
        assert!(out["SWE"][0] <= out["SWT"][0], "SWE <= SWT");
    }

    /// T-ADV-10 — the SSPW fallback, for **sw_imts** and applied per SAMPLE.
    ///
    /// `rtc_falls_back_to_sspw_curve_names` covers sw_rtc on a wholly SSPW well. Two things it
    /// does not: sw_imts, which the manual test asks to repeat in its own pane, and the fact that
    /// `prefer` chooses per SAMPLE rather than per curve. The per-sample part is what matters on
    /// a real well: a section reprocessed through SSPW leaves PHIT_SSC populated above and below
    /// it and PHIT_SSPW only across the reprocessed interval, and a curve-level fallback would
    /// either ignore the new work or discard the old.
    ///
    /// Three samples, one of each case, in one run:
    ///   0 — SSC names only          → the primary is used
    ///   1 — SSPW names only         → the fallback is used
    ///   2 — BOTH present, differing → the primary must win
    #[test]
    fn the_sspw_fallback_covers_imts_and_chooses_sample_by_sample() {
        let nan = f32::NAN;

        // `prefer` itself, stated directly — the three cases above plus "neither" staying MISSING.
        let picked = prefer(&[0.20, nan, 0.20, nan], &[0.30, 0.30, 0.30, nan]);
        assert_eq!(picked[0], 0.20, "the primary wins where it exists");
        assert_eq!(picked[1], 0.30, "the fallback fills where the primary is missing");
        assert_eq!(picked[2], 0.20, "the primary still wins when both are present");
        assert!(picked[3].is_nan(), "with neither curve the sample stays MISSING, never zero");

        // sw_imts through the module, on the mixed well the manual test describes.
        let spec = sw_imts_spec();
        let ctx = ctx_with(
            vec![
                ("RT", vec![4.0, 4.0, 4.0]),
                ("PHIT", vec![0.25, nan, 0.25]),
                ("PHIT_SSPW", vec![nan, 0.25, 0.10]),
                ("CBW", vec![0.03, nan, 0.03]),
                ("CBW_SSPW", vec![nan, 0.03, 0.03]),
                ("VKAOL", vec![0.10, 0.10, 0.10]),
                ("VILL", vec![0.05, 0.05, 0.05]),
            ],
            &spec,
            3,
        );
        let out = sw_imts(&ctx);
        for i in 0..3 {
            assert!(out["SWT_IMTS"][i].is_finite(), "sample {i}: SWT_IMTS must be computed, got NaN");
            assert!(out["SWE_IMTS"][i] <= out["SWT_IMTS"][i], "sample {i}: SWE <= SWT");
        }
        // Samples 0 and 1 are the same rock reached by the two different curve names, so the
        // fallback must land on the same answer rather than merely on *an* answer.
        assert!(
            (out["SWT_IMTS"][0] - out["SWT_IMTS"][1]).abs() < 1e-5,
            "the SSPW fallback must reproduce the SSC result: {} vs {}",
            out["SWT_IMTS"][0],
            out["SWT_IMTS"][1]
        );
        // Sample 2 carries both, with a much tighter SSPW porosity. If the fallback ever took
        // precedence this would move — 0.10 v/v porosity cannot give the same Sw as 0.25.
        assert!(
            (out["SWT_IMTS"][2] - out["SWT_IMTS"][0]).abs() < 1e-5,
            "where both exist the SSC curve must win: {} vs {}",
            out["SWT_IMTS"][2],
            out["SWT_IMTS"][0]
        );

        // Control: an SSC-only well is byte-for-byte what it was before the fallback existed, so
        // this cannot have changed any existing interpretation.
        let ssc_only = sw_imts(&ctx_with(
            vec![
                ("RT", vec![4.0]),
                ("PHIT", vec![0.25]),
                ("CBW", vec![0.03]),
                ("VKAOL", vec![0.10]),
                ("VILL", vec![0.05]),
            ],
            &spec,
            1,
        ));
        assert_eq!(ssc_only["SWT_IMTS"][0], out["SWT_IMTS"][0]);
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

    // -----------------------------------------------------------------------
    // RtC calibration fit
    // -----------------------------------------------------------------------

    /// Builds a well whose resistivity is SYNTHESISED from sw_rtc's own forward model with
    /// known coefficients, so the fit has a right answer to be judged against.
    ///
    /// `sw` lets a caller make the lower part of the well hydrocarbon-bearing: the wet leg
    /// (Sw = 1) is what the calibration is entitled to see, and the pay leg is the trap.
    fn synth_well(
        conn: &Connection,
        name: &str,
        truth: (f64, f64, f64),
        rw: f64,
        m: f64,
        rsf: f64,
        n_samples: usize,
        sw_at: impl Fn(usize) -> f64,
    ) -> String {
        use crate::db;
        use uuid::Uuid;
        let (a, b, c) = truth;
        let wid = Uuid::new_v4();
        db::insert_well(conn, wid, name, None, None, Some(0.0)).unwrap();

        let mut depth = Vec::with_capacity(n_samples);
        let (mut rt, mut phit, mut capbw, mut qv) = (vec![], vec![], vec![], vec![]);
        for i in 0..n_samples {
            let f = i as f64 / (n_samples - 1) as f64;
            let d = 1000.0 + 0.5 * i as f64;
            // Vary CAPBW and Qv INDEPENDENTLY — collinear inputs cannot separate the two
            // conductivity paths, and the solver is supposed to say so rather than guess.
            // Ranges are LRLC-like on purpose: a microporous, clay-rich rock in fresh-brackish
            // water (`method_ssc_sspw.md`), where the excess conductivity is a LARGE fraction of
            // the total. In clean rock under saline water the excess is a few percent of Archie
            // and every one of these tests passes trivially — which would prove nothing about
            // the rock the method exists for.
            let cap = 0.05 + 0.20 * f;
            let qv_i = 0.10 + 0.50 * (1.0 - f) * (0.5 + 0.5 * ((i as f64) * 0.7).sin());
            let pt = 0.30 - 0.06 * f;
            let cex = (a * cap + b * qv_i + c) * pt * rsf;
            let sw = sw_at(i);
            // Forward: Ct = Sw^n·φ^m/Rw + Cex, with n = 2. Invert for the logged Rt.
            let ct = sw.powi(2) * pt.powf(m) / rw + cex;
            depth.push(d as f32);
            rt.push((1.0 / ct) as f32);
            phit.push(pt as f32);
            capbw.push(cap as f32);
            qv.push(qv_i as f32);
        }
        let nan = vec![f32::NAN; n_samples];
        db::insert_standard_curves(
            conn, wid, depth.clone(),
            nan.clone(), nan.clone(), nan.clone(), nan.clone(), nan.clone(), nan.clone(),
        )
        .unwrap();
        crate::equations::write_computed_curves_batch(
            conn, &wid.to_string(), &depth,
            &[("RT_SYN", &rt), ("PHIT_SYN", &phit), ("CAPBW_SYN", &capbw), ("QV_SYN", &qv)],
        )
        .unwrap();
        wid.to_string()
    }

    fn fit_req(wells: Vec<String>, rw: f64, m: f64, rsf: f64) -> RtcFitRequest {
        RtcFitRequest {
            well_ids: wells,
            rt_curve: "RT_SYN".into(),
            phit_curve: "PHIT_SYN".into(),
            capbw_curve: "CAPBW_SYN".into(),
            qv_curve: "QV_SYN".into(),
            cec: 0.0,
            rhog: 2.65,
            rw,
            m,
            rsf,
            depth_min: None,
            depth_max: None,
            wet_flag_curve: String::new(),
        }
    }

    /// The fit must recover the coefficients that generated the rock. This is the whole claim:
    /// a user pointing it at their own water leg gets THEIR calibration, not a foreign one.
    #[test]
    fn the_fit_recovers_the_coefficients_that_generated_the_rock() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let (rw, m, rsf) = (1.0, 2.0, 2.25);
        let truth = (0.45, 0.0057, -0.0071); // the shipped study values, used here as a target
        let w = synth_well(&conn, "WET-1", truth, rw, m, rsf, 120, |_| 1.0);
        let db = Mutex::new(conn);

        let mut req = fit_req(vec![w], rw, m, rsf);
        req.depth_min = Some(0.0); // declare the whole well wet — it is
        let r = run_rtc_fit(&db, &req);

        assert!(r.error.is_none(), "fit failed: {:?}", r.error);
        assert!(r.n_points > 100, "expected the whole wet leg, got {}", r.n_points);
        assert!((r.a_cap - truth.0).abs() < 1e-6, "A_CAP {} vs {}", r.a_cap, truth.0);
        assert!((r.b_qv - truth.1).abs() < 1e-6, "B_QV {} vs {}", r.b_qv, truth.1);
        assert!((r.c0 - truth.2).abs() < 1e-6, "C0 {} vs {}", r.c0, truth.2);
        assert!(r.r2 > 0.999, "a noiseless fit must be near-perfect, got R2 {}", r.r2);
        assert_eq!(r.rsf_used, rsf);
    }

    /// Fitting over hydrocarbon-bearing rock is the failure mode this feature exists to
    /// prevent, so the interval selection must actually bite.
    ///
    /// The mechanism, worth stating because the sign is counter-intuitive: hydrocarbon REMOVES
    /// conductivity, so a pay sample's apparent excess (1/Rt − φ^m/Rw) comes out too SMALL. A
    /// calibration fitted through those samples therefore UNDER-predicts excess conductivity,
    /// under-corrects Rt, and returns Sw too high — it erases pay rather than inventing it.
    ///
    /// Sw = 0.9 in the pay leg is deliberately mild. Anything stronger and the apparent excess
    /// goes negative and the samples are rejected outright (see the companion test) — which
    /// means the dangerous case is not obvious pay, it is the LIGHTLY hydrocarbon-bearing
    /// interval a user might reasonably believe is wet.
    #[test]
    fn including_the_pay_leg_biases_the_calibration_low_so_the_interval_must_be_selectable() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        // Fresh formation water — the LRLC setting (`method_ssc_sspw.md`), and what makes the
        // excess term comparable to the Archie term instead of a rounding error.
        let (rw, m, rsf) = (1.0, 2.0, 2.25);
        let truth = (0.45, 0.0057, -0.0071);
        let w = synth_well(&conn, "MIXED-1", truth, rw, m, rsf, 120, |i| if i < 60 { 1.0 } else { 0.9 });
        let db = Mutex::new(conn);

        // The model's predicted excess at a representative sample — what actually reaches Sw,
        // and a fairer measure than any single coefficient, which can move either way.
        let predict = |r: &RtcFitResult| r.a_cap * 0.15 + r.b_qv * 0.30 + r.c0;
        let truth_pred = truth.0 * 0.15 + truth.1 * 0.30 + truth.2;

        // Wet leg only: samples 0..59 are depths 1000.0 .. 1029.5.
        let mut wet = fit_req(vec![w.clone()], rw, m, rsf);
        wet.depth_min = Some(1000.0);
        wet.depth_max = Some(1029.5);
        let good = run_rtc_fit(&db, &wet);
        assert!(good.error.is_none(), "{:?}", good.error);
        assert_eq!(good.n_points, 60);
        assert!((good.a_cap - truth.0).abs() < 1e-5, "wet-leg A_CAP {} vs {}", good.a_cap, truth.0);
        assert!(
            (predict(&good) - truth_pred).abs() < 1e-6,
            "the wet-leg fit must reproduce the truth: {} vs {}",
            predict(&good), truth_pred
        );

        // Whole well, pay included.
        let mut all = fit_req(vec![w], rw, m, rsf);
        all.depth_min = Some(0.0);
        let bad = run_rtc_fit(&db, &all);
        assert!(bad.error.is_none(), "{:?}", bad.error);
        assert!(bad.n_points > 60, "the pay samples must actually enter this fit: {}", bad.n_points);
        assert!(
            predict(&bad) < predict(&good) * 0.9,
            "including pay must visibly under-predict the excess, else the interval guard is \
             pointless: {} vs wet-leg {}",
            predict(&bad), predict(&good)
        );
    }

    /// The positivity guard catches MOST obvious pay but is not reliable protection, and this
    /// records precisely how it leaks — because the leak is the argument for making the
    /// water-zone declaration mandatory rather than advisory.
    ///
    /// A sample whose apparent excess is negative reads as more resistive than clean
    /// water-filled rock can be, so it is dropped. At Sw = 0.3 that rejects most of the pay
    /// leg. But it does NOT reject all of it: where the rock is most microporous the true
    /// excess conductivity is large enough to mask the hydrocarbon, the apparent excess stays
    /// positive, and the sample sails through. That is the LRLC problem stated in reverse — the
    /// guard is weakest exactly where this method is used — and the survivors still bend the
    /// calibration.
    #[test]
    fn the_positivity_guard_catches_most_obvious_pay_but_leaks_in_the_most_microporous_rock() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let (rw, m, rsf) = (1.0, 2.0, 2.25);
        let truth = (0.45, 0.0057, -0.0071);
        let w = synth_well(&conn, "OBVIOUS-1", truth, rw, m, rsf, 120, |i| if i < 60 { 1.0 } else { 0.3 });
        let db = Mutex::new(conn);

        let mut all = fit_req(vec![w.clone()], rw, m, rsf);
        all.depth_min = Some(0.0);
        let r = run_rtc_fit(&db, &all);
        assert!(r.error.is_none(), "{:?}", r.error);

        // Most of the pay leg rejects itself...
        let rejected = r.excluded.iter().find(|(w, _)| w.contains("no excess")).map(|(_, n)| *n).unwrap_or(0);
        assert!(rejected >= 40, "the guard must catch the bulk of obvious pay, got {rejected} of 60");
        // ...but not all of it, and what survives is not wet rock.
        assert!(
            r.n_points > 60,
            "if the guard caught everything this test would be asserting the wrong thing; \
             re-check the synthetic rock. n_points = {}",
            r.n_points
        );
        assert!(
            r.excluded.iter().any(|(why, _)| why.contains("no excess")),
            "rejected samples must be named and counted, never dropped silently: {:?}",
            r.excluded
        );

        // And the leak matters: the calibration is measurably off the truth the wet leg alone
        // recovers exactly. THIS is why the water zone must be declared, not inferred.
        let mut wet = fit_req(vec![w], rw, m, rsf);
        wet.depth_min = Some(1000.0);
        wet.depth_max = Some(1029.5);
        let good = run_rtc_fit(&db, &wet);
        assert!((good.a_cap - truth.0).abs() < 1e-5, "wet leg must be exact: {}", good.a_cap);
        assert!(
            (r.a_cap - truth.0).abs() > 100.0 * (good.a_cap - truth.0).abs().max(1e-12),
            "the surviving pay samples must visibly bend the fit: whole-well A_CAP {} vs \
             wet-leg {} vs truth {}",
            r.a_cap, good.a_cap, truth.0
        );
    }

    /// With no interval and no wet flag the fit must REFUSE. Defaulting to the whole well is
    /// the single most damaging thing it could do, and it would look like it worked.
    #[test]
    fn the_fit_refuses_when_no_water_zone_is_declared() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let w = synth_well(&conn, "W-1", (0.45, 0.0057, -0.0071), 1.0, 2.0, 2.25, 40, |_| 1.0);
        let db = Mutex::new(conn);
        let r = run_rtc_fit(&db, &fit_req(vec![w], 1.0, 2.0, 2.25));
        let e = r.error.expect("must refuse without a declared water zone");
        assert!(e.to_uppercase().contains("WATER"), "the refusal must say why: {e}");
        assert!(r.n_points == 0);
    }

    /// A wet-flag curve selects the same samples a depth range would. NaN is not "wet" —
    /// "unknown" must never be read as "yes" by the guard that keeps hydrocarbon out.
    #[test]
    fn a_nan_wet_flag_is_not_treated_as_wet() {
        use crate::equations::write_computed_curves_batch;
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let (rw, m, rsf) = (1.0, 2.0, 2.25);
        let truth = (0.45, 0.0057, -0.0071);
        let w = synth_well(&conn, "FLAG-1", truth, rw, m, rsf, 120, |i| if i < 60 { 1.0 } else { 0.3 });
        // Flag the wet half 1.0, leave the pay half NaN (not 0) — the realistic shape when a
        // flag curve is only computed over part of a well.
        let depth: Vec<f32> = (0..120).map(|i| 1000.0 + 0.5 * i as f32).collect();
        let flag: Vec<f32> = (0..120).map(|i| if i < 60 { 1.0 } else { f32::NAN }).collect();
        write_computed_curves_batch(&conn, &w, &depth, &[("WETFLAG", &flag)]).unwrap();
        let db = Mutex::new(conn);

        let mut req = fit_req(vec![w], rw, m, rsf);
        req.wet_flag_curve = "WETFLAG".into();
        let r = run_rtc_fit(&db, &req);
        assert!(r.error.is_none(), "{:?}", r.error);
        assert_eq!(r.n_points, 60, "only the flagged wet samples may be fitted");
        assert!((r.a_cap - truth.0).abs() < 1e-5, "A_CAP {} vs {}", r.a_cap, truth.0);
        assert!(
            r.excluded.iter().any(|(why, n)| why.contains("flagged") && *n == 60),
            "the 60 unflagged samples must be reported, not dropped silently: {:?}",
            r.excluded
        );
    }

    /// Every candidate sample that does not make it into the fit must be counted and named.
    /// A calibration quoted from 12 samples of an interval the user thought held 500 is a
    /// different statement, and silence about it is the failure.
    #[test]
    fn excluded_samples_are_counted_and_named() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let (rw, m, rsf) = (1.0, 2.0, 2.25);
        // Rw far too saline for this rock -> the clean baseline sits ABOVE the measured
        // conductivity everywhere, so every sample has "no excess to explain".
        let w = synth_well(&conn, "SALTY-1", (0.45, 0.0057, -0.0071), rw, m, rsf, 60, |_| 1.0);
        let db = Mutex::new(conn);
        let mut req = fit_req(vec![w], 0.02, m, rsf); // fit with a much fresher Rw than generated
        req.depth_min = Some(0.0);
        let r = run_rtc_fit(&db, &req);
        assert!(r.error.is_some(), "a fit with no usable samples must fail, not return zeros");
        // And the failure names the count, so the user can tell "no data" from "all rejected".
        assert!(
            r.error.as_deref().unwrap().contains("water-zone sample"),
            "{:?}",
            r.error
        );
    }

    /// Collinear inputs cannot separate the clay path from the capillary path. The solver must
    /// say so rather than return a confident arbitrary split of one effect across two terms.
    #[test]
    fn a_constant_qv_drops_its_term_instead_of_inventing_one() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let (rw, m, rsf) = (1.0, 2.0, 2.25);
        let w = synth_well(&conn, "NOQV-1", (0.45, 0.0, -0.0071), rw, m, rsf, 80, |_| 1.0);
        let db = Mutex::new(conn);

        let mut req = fit_req(vec![w], rw, m, rsf);
        req.qv_curve = String::new(); // no QV log, and cec = 0 -> Qv is identically zero
        req.depth_min = Some(0.0);
        let r = run_rtc_fit(&db, &req);

        assert!(r.error.is_none(), "{:?}", r.error);
        assert_eq!(r.b_qv, 0.0, "an unfittable term must be reported as 0, not guessed");
        assert!((r.a_cap - 0.45).abs() < 1e-5, "the capillary term must still be right: {}", r.a_cap);
        assert!(
            r.notes.iter().any(|n| n.contains("Qv does not vary")),
            "the user must be told the clay term was not fitted: {:?}",
            r.notes
        );
    }

    /// "Apply this calibration to the wells it was fitted from" has to mean the wells that
    /// actually contributed. A scoped well that gave nothing was never calibrated, and writing
    /// coefficients to it extends the claim past the data — so it must not appear in the list
    /// the apply step reads.
    #[test]
    fn a_scoped_well_that_contributed_nothing_is_not_in_the_fitted_list() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let (rw, m, rsf) = (1.0, 2.0, 2.25);
        let good = synth_well(&conn, "WET-1", (0.45, 0.0057, -0.0071), rw, m, rsf, 80, |_| 1.0);
        let empty = uuid::Uuid::new_v4().to_string();
        let db = Mutex::new(conn);

        let mut req = fit_req(vec![good.clone(), empty.clone()], rw, m, rsf);
        req.depth_min = Some(0.0);
        let r = run_rtc_fit(&db, &req);

        assert!(r.error.is_none(), "{:?}", r.error);
        assert_eq!(r.wells_fitted, vec![good], "only the contributing well");
        assert!(!r.wells_fitted.contains(&empty));
        assert_eq!(r.n_wells, r.wells_fitted.len());
        assert!(
            r.notes.iter().any(|n| n.contains("contributed no water-zone samples")),
            "{:?}",
            r.notes
        );
    }

    // -----------------------------------------------------------------------
    // S-factor calibration
    // -----------------------------------------------------------------------

    /// A well with clay curves on a 0.5 m log grid and laboratory CEC plugs whose values were
    /// generated as `s_true · (VKAOL·8 + VILL·25)` — the module's own clay-charge line.
    ///
    /// `plug_offset` shifts every plug depth off the log grid, which is how a core that has not
    /// been depth-shifted to the log actually presents itself.
    fn synth_cec_well(
        conn: &Connection,
        name: &str,
        s_true: f64,
        n_samples: usize,
        n_plugs: usize,
        plug_offset: f64,
        // vk, vi at fraction f along the well
        clay_at: impl Fn(f64) -> (f64, f64),
        // scales the lab CEC of plug k, for the "S is not constant" case
        lab_scale: impl Fn(usize, f64) -> f64,
    ) -> String {
        use crate::db;
        use uuid::Uuid;
        let wid = Uuid::new_v4();
        db::insert_well(conn, wid, name, None, None, Some(0.0)).unwrap();

        let mut depth = Vec::with_capacity(n_samples);
        let (mut vk, mut vi) = (vec![], vec![]);
        for i in 0..n_samples {
            let f = i as f64 / (n_samples - 1) as f64;
            depth.push((1000.0 + 0.5 * i as f64) as f32);
            let (a, b) = clay_at(f);
            vk.push(a as f32);
            vi.push(b as f32);
        }
        let nan = vec![f32::NAN; n_samples];
        db::insert_standard_curves(
            conn, wid, depth.clone(),
            nan.clone(), nan.clone(), nan.clone(), nan.clone(), nan.clone(), nan.clone(),
        )
        .unwrap();
        crate::equations::write_computed_curves_batch(
            conn, &wid.to_string(), &depth, &[("VKAOL_SYN", &vk), ("VILL_SYN", &vi)],
        )
        .unwrap();

        // `n_plugs == 0` means a well with logs and NO CEC delivery — the ordinary case in a
        // field study, and the one the fitted-well list has to exclude.
        if n_plugs == 0 {
            return wid.to_string();
        }
        // Plugs sit on every (n_samples / n_plugs)-th log sample, offset by `plug_offset`.
        let step = (n_samples / n_plugs).max(1);
        let rows: Vec<db::AuxRow> = (0..n_plugs)
            .map(|k| {
                let i = (k * step).min(n_samples - 1);
                let theo = cec_theo_at(vk[i] as f64, vi[i] as f64, 8.0, 25.0);
                db::AuxRow {
                    dataset: "CEC".into(),
                    depth_top: (depth[i] as f64 + plug_offset) as f32,
                    depth_base: None,
                    item: "CEC".into(),
                    value_num: Some((s_true * theo * lab_scale(k, theo)) as f32),
                    value_text: None,
                }
            })
            .collect();
        db::insert_aux_data(conn, &wid.to_string(), "CEC", "RAW", Some("test"), &rows).unwrap();
        wid.to_string()
    }

    fn s_req(wells: Vec<String>) -> SFactorFitRequest {
        SFactorFitRequest {
            well_ids: wells,
            cec_dataset: "CEC".into(),
            cec_item: "CEC".into(),
            vkaol_curve: "VKAOL_SYN".into(),
            vill_curve: "VILL_SYN".into(),
            cec_kaol: 8.0,
            cec_ill: 25.0,
            depth_tol: 0.15,
        }
    }

    /// Clay that varies over a wide range, so the fit is not being asked to find a slope through
    /// a cloud of near-identical plugs. Both minerals rise together, giving a theoretical CEC
    /// from ~0.4 to ~4.8 meq/100g — a first attempt had them move in opposite directions, which
    /// held the total nearly constant and made every clay-range test vacuous.
    fn spread_clay(f: f64) -> (f64, f64) {
        (0.02 + 0.30 * f, 0.01 + 0.08 * f)
    }

    /// The claim of the whole feature: point it at your own lab CEC and you get YOUR S.
    #[test]
    fn the_s_fit_recovers_the_scaling_factor_that_generated_the_plugs() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let w = synth_cec_well(&conn, "CEC-1", 0.42, 120, 30, 0.0, spread_clay, |_, _| 1.0);
        let db = Mutex::new(conn);

        let r = run_s_factor_fit(&db, &s_req(vec![w]));
        assert!(r.error.is_none(), "fit failed: {:?}", r.error);
        assert_eq!(r.n_points, 30, "every plug should pair: {:?}", r.excluded);
        assert!((r.s_factor - 0.42).abs() < 1e-4, "S {} vs 0.42", r.s_factor);
        // With no scatter, the weighted fit and the plain median of ratios must agree — that is
        // what "S is constant across the clay range" means, and the gap note must stay silent.
        assert!((r.s_median_ratio - 0.42).abs() < 1e-4, "median {}", r.s_median_ratio);
        assert!(r.r2 > 0.999, "a noiseless fit must be near-perfect: R2 {}", r.r2);
        assert!((r.ratio_p90 / r.ratio_p10 - 1.0).abs() < 1e-3, "a constant S has no spread");
        assert!(
            !r.notes.iter().any(|n| n.contains("plugs disagree") || n.contains("median per-plug")),
            "no spread here, so no spread warning: {:?}",
            r.notes
        );
    }

    /// The fitted S must make `sw_imts` reproduce the measured CEC — that is the point of
    /// deriving the fit as the algebraic inverse of `cec_theo_at` rather than from the method
    /// note. QVEFF is the module's only exposed view of the clay charge, so check it there.
    #[test]
    fn the_fitted_s_makes_the_module_reproduce_the_measured_cec() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let w = synth_cec_well(&conn, "CEC-2", 0.37, 120, 30, 0.0, spread_clay, |_, _| 1.0);
        let db = Mutex::new(conn);
        let r = run_s_factor_fit(&db, &s_req(vec![w]));
        assert!(r.error.is_none(), "{:?}", r.error);

        // Take a real plug and run the module at its clay content with the fitted S.
        let p = &r.points[10];
        let (rhog, swirr, phit) = (2.65f64, 0.2f64, 0.25f64);
        let spec = sw_imts_spec();
        let mut ctx = ctx_with(
            vec![
                ("RT", vec![10.0]),
                ("PHIT", vec![phit as f32]),
                ("VKAOL", vec![p.vkaol as f32]),
                ("VILL", vec![p.vill as f32]),
            ],
            &spec,
            1,
        );
        ctx.params.insert("S_FACTOR".into(), vec![r.s_factor]);
        ctx.params.insert("SWIRR_DEF".into(), vec![swirr]);
        ctx.params.insert("RHOG".into(), vec![rhog]);
        let qveff = sw_imts(&ctx)["QVEFF"][0] as f64;

        // What the laboratory measurement itself says Qv_eff is at this plug.
        let expect = p.cec_lab * rhog * (1.0 - phit) / (100.0 * phit * (1.0 - swirr));
        assert!(
            (qveff - expect).abs() < 1e-6 * expect.max(1.0),
            "the module must land on the lab CEC: {qveff} vs {expect}"
        );
    }

    /// A plug depth that does not line up with the log is a core that has not been depth-shifted.
    /// Pairing it with the nearest sample regardless of distance would fabricate a data point
    /// that looks exactly like a real one, so it must be dropped and counted.
    ///
    /// The offset here is a QUARTER of the log sampling, deliberately. Writing this test with a
    /// 3.0 m shift on a 0.5 m grid proved nothing: 3.0 is six whole samples, so every plug landed
    /// exactly on a log depth and paired perfectly. That is not a flaw in the check but a real
    /// limit of it — **a shift that is a whole number of sample intervals is invisible to any
    /// depth-tolerance test**, because the log grid has no way to see it. The tolerance keeps a
    /// measurement off rock it did not come from; it is not a substitute for depth-shifting the
    /// core against the core gamma.
    #[test]
    fn a_plug_off_the_log_depth_is_dropped_not_snapped_to_the_nearest_sample() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let w = synth_cec_well(&conn, "CEC-3", 0.42, 120, 30, 0.25, spread_clay, |_, _| 1.0);
        let db = Mutex::new(conn);

        let r = run_s_factor_fit(&db, &s_req(vec![w.clone()]));
        assert!(r.error.is_some(), "an unshifted core must not quietly produce an S");
        let msg = r.error.unwrap();
        assert!(msg.contains("usable plug"), "{msg}");

        // Widen the tolerance past the shift and the same plugs pair — so the guard is the
        // tolerance doing its job, not a broken depth lookup.
        let mut wide = s_req(vec![w]);
        wide.depth_tol = 0.3;
        let r2 = run_s_factor_fit(&db, &wide);
        assert!(r2.error.is_none(), "{:?}", r2.error);
        assert_eq!(r2.n_points, 30);
    }

    /// Where the clay curves say there is no clay there is no ratio to take, and a lab CEC that
    /// is nevertheless non-zero is evidence AGAINST the clay model rather than a data point.
    #[test]
    fn a_plug_with_no_modelled_clay_is_excluded_instead_of_dividing_by_zero() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        // Clean over the top half of the well, clayey below.
        let w = synth_cec_well(
            &conn, "CEC-4", 0.42, 120, 30, 0.0,
            |f| if f < 0.5 { (0.0, 0.0) } else { (0.10 + 0.20 * f, 0.05) },
            |_, _| 1.0,
        );
        let db = Mutex::new(conn);
        let r = run_s_factor_fit(&db, &s_req(vec![w]));

        assert!(r.error.is_none(), "{:?}", r.error);
        assert!(r.s_factor.is_finite(), "S must not be NaN from a zero divide");
        assert!((r.s_factor - 0.42).abs() < 1e-4, "the clayey plugs still give S: {}", r.s_factor);
        let dropped: usize = r
            .excluded
            .iter()
            .filter(|(why, _)| why.contains("no clay"))
            .map(|(_, n)| *n)
            .sum();
        assert!(dropped > 0, "the clean plugs must be counted out loud: {:?}", r.excluded);
        assert_eq!(dropped + r.n_points, 30, "every plug is either fitted or named");
    }

    /// S above 1 means the clay model is under-calling exchange capacity — most often a mineral
    /// it does not carry. Smectite at 80-150 meq/100g dwarfs illite's 25, so a few percent of it
    /// is enough. The fit must say so rather than return a number that looks ordinary.
    #[test]
    fn an_s_above_one_is_flagged_because_a_missing_clay_mineral_hides_there() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        // Lab CEC 1.8x what kaolinite+illite can account for: unmodelled smectite.
        let w = synth_cec_well(&conn, "CEC-5", 1.8, 120, 30, 0.0, spread_clay, |_, _| 1.0);
        let db = Mutex::new(conn);
        let r = run_s_factor_fit(&db, &s_req(vec![w]));

        assert!(r.error.is_none(), "{:?}", r.error);
        assert!(r.s_factor > 1.0, "S {}", r.s_factor);
        assert!(
            r.notes.iter().any(|n| n.contains("smectite")),
            "the likely cause must be named: {:?}",
            r.notes
        );
    }

    /// When S drifts with clay content it is not a scaling factor at all, and the user must be
    /// told before quoting one number.
    ///
    /// This test is why the spread, not the median-vs-fit gap, is the detector. A ratio drifting
    /// linearly from 1.13 on the leanest plugs to 0.40 on the clayiest — a factor of nearly three,
    /// unmistakable in a crossplot — moves the median only 28% away from the fit, because both
    /// are central values and the clay-weighted centre sits at x ≈ 3.6 against the median's
    /// x ≈ 2.6. A gap threshold loose enough to survive noise would never fire on real drift. The
    /// P10-P90 spread of the plugs' own ratios is bounded by nothing and catches it at 2.8x.
    #[test]
    fn a_drifting_s_shows_up_in_the_spread_of_the_per_plug_ratios() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        // Lab CEC runs 3x high on the leanest plugs and falls to 1x on the clayiest — the shape
        // a non-clay conductive mineral or a detection-limit floor puts into a CEC suite.
        let w = synth_cec_well(&conn, "CEC-6", 0.4, 200, 40, 0.0, spread_clay, |_, theo| {
            1.0 + 2.0 * (1.0 - (theo / 4.8).clamp(0.0, 1.0))
        });
        let db = Mutex::new(conn);
        let r = run_s_factor_fit(&db, &s_req(vec![w]));

        assert!(r.error.is_none(), "{:?}", r.error);
        assert!(
            r.ratio_p90 / r.ratio_p10 > 2.0,
            "the plugs' own ratios must show the drift: P10 {} P90 {}",
            r.ratio_p10,
            r.ratio_p90
        );
        assert!(
            r.notes.iter().any(|n| n.contains("plugs disagree about S")),
            "the user must be told S drifts: {:?}",
            r.notes
        );
        // The median-vs-fit gap is real but small — pinned here so the ceiling argument in the
        // doc comment above stays honest if anyone retunes the thresholds.
        assert!(
            r.s_median_ratio > r.s_factor && r.s_median_ratio < r.s_factor * 1.5,
            "median {} vs fit {}",
            r.s_median_ratio,
            r.s_factor
        );
    }

    /// Getting the item name wrong is the most likely first mistake, and "no data" is a useless
    /// answer to it. Say which items the delivery actually holds.
    #[test]
    fn a_wrong_item_name_reports_the_items_that_are_actually_there() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let w = synth_cec_well(&conn, "CEC-7", 0.42, 120, 30, 0.0, spread_clay, |_, _| 1.0);
        let db = Mutex::new(conn);
        let mut req = s_req(vec![w]);
        req.cec_item = "CEC_MEAS".into();
        let r = run_s_factor_fit(&db, &req);

        let msg = r.error.expect("a missing item must be an error, not an empty fit");
        assert!(msg.contains("CEC_MEAS"), "{msg}");
        assert!(msg.contains("Items present"), "{msg}");
        assert!(msg.contains("CEC"), "{msg}");
    }

    /// S and the literature constants are the same knob twice — halve the constants and the
    /// fitted S doubles, landing on the same clay charge. The result must therefore pin the
    /// constants it belongs to.
    #[test]
    fn s_is_reported_against_the_constants_it_was_fitted_with() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let w = synth_cec_well(&conn, "CEC-8", 0.42, 120, 30, 0.0, spread_clay, |_, _| 1.0);
        let db = Mutex::new(conn);

        let base = run_s_factor_fit(&db, &s_req(vec![w.clone()]));
        let mut halved = s_req(vec![w]);
        halved.cec_kaol = 4.0;
        halved.cec_ill = 12.5;
        let half = run_s_factor_fit(&db, &halved);

        assert!((half.s_factor - 2.0 * base.s_factor).abs() < 1e-4, "{} vs {}", half.s_factor, base.s_factor);
        assert_eq!(half.cec_kaol_used, 4.0);
        assert_eq!(half.cec_ill_used, 12.5);
        assert!(
            half.notes.iter().any(|n| n.contains("not jointly identifiable")),
            "{:?}",
            half.notes
        );
    }

    /// The S fit's own version of the fitted-well contract, and the reason it is reported
    /// separately from `points`: the display points are decimated, so they are not a safe
    /// source for "which wells does this calibration belong to".
    #[test]
    fn the_s_fit_names_only_the_wells_that_gave_a_plug() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let cored = synth_cec_well(&conn, "CEC-9", 0.42, 120, 30, 0.0, spread_clay, |_, _| 1.0);
        // A well with logs but no CEC delivery — the ordinary case in a field study.
        let uncored = synth_cec_well(&conn, "CEC-10", 0.42, 120, 0, 0.0, spread_clay, |_, _| 1.0);
        let db = Mutex::new(conn);

        let r = run_s_factor_fit(&db, &s_req(vec![cored.clone(), uncored.clone()]));
        assert!(r.error.is_none(), "{:?}", r.error);
        assert_eq!(r.wells_fitted, vec![cored], "only the cored well is calibrated");
        assert!(!r.wells_fitted.contains(&uncored));
        assert_eq!(r.n_wells, 1);
    }
}
