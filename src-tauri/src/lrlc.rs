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
              THE DEFAULT COEFFICIENTS ARE ONE STUDY'S CALIBRATION (0.45, 0.0057, −0.0071, \
              RSF 2.25) FROM ONE FIELD — they are a starting point, not a constant, and a \
              foreign calibration here does not announce itself: it yields a smooth, plausible \
              Sw that is simply wrong. Fit your own with Advance ▸ Calibrate RtC…, which \
              regresses A_CAP/B_QV/C0 from excess conductivity over an interval you declare \
              water-bearing. CAPBW pairs naturally with SSC's CWSH or \
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
        let qv = qv_at(qv_log[i] as f64, pt, ctx.p("CEC", i), ctx.p("RHOG", i));

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
}
