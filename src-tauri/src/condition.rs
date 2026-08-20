//! **Condition** — the curve-conditioning module family: despike, smooth, clip, fill gaps and
//! flip polarity. The first half of the Condition & Frame work (Jauhar, 2026-08-05); `Frame`
//! (block, resample, regularize) is the companion category and lives separately.
//!
//! These are MODULES rather than a separate editing tool, and that is the design decision worth
//! recording. The reference suite keeps its log-edit family in its own launcher because it has no
//! module framework; SandiBumi does, so writing a Rust fn plus a manifest buys multi-well
//! rayon-parallel runs, zone-overridable parameters, workflow chaining, the universal run mask,
//! log-set versioning with provenance and an auto-generated dialog — none of which a bespoke
//! editor would have on day one. `curve_edit.rs` remains the INTERACTIVE path (one interval, one
//! curve, from the log view's right-click); this is the batch path.
//!
//! ## Four rules the whole family holds
//!
//! **A window is a THICKNESS, never a sample count.** Every window here is stated in the project's
//! depth unit and resolved against the curve's own depth column ([`Frame::windows`]). A window in
//! samples silently changes the amount of rock it covers the moment a curve is resampled, or when
//! one curve of a run came in at 2 inches and another at 6 — and nothing downstream can see that
//! it did. This is the same reasoning that put `fov_um` in the picture store rather than
//! micrometres per pixel: state the physical quantity, derive the pixel count.
//!
//! **Nothing here invents a sample except Fill Gaps, which says so.** Smoothing never bridges a
//! gap — a MISSING sample stays MISSING — because filling and smoothing are different claims and
//! a curve that quietly acquired values across a washout reads exactly like one that was logged
//! there. Fill Gaps is the one module allowed to write a value nobody measured, it is bounded by a
//! maximum gap the user sets, and every sample it writes is marked in a companion flag curve.
//!
//! **The output is never the input's own mnemonic, and that is not a style choice.**
//! `equations::fetch_curve_frame` resolves the six standard mnemonics (GR, RES_DEEP, NPHI, RHOB,
//! DT, SP) from `standard_curves` FIRST, falling through to `computed_curves` only when the
//! standard column is entirely NaN. A despiked curve written back as plain `GR` would therefore be
//! stored, counted, reported — and invisible to every module, plot and export that reads GR. The
//! same shape as the `CPHOTO_*` trace that was written at the photograph's own sampling and came
//! back all-NaN to every reader: the worst kind of bug, because the run reports success.
//! [`crate::workflow::resolve_output_names`] refuses such a name BY NAME with the reason rather
//! than writing a curve nothing can open — one check, in front of every module, rather than a copy
//! of it here and another in `frame.rs`.
//!
//! **A parameter that cannot have a generic value has NO default.** The despike window and the
//! fill-gap limit open empty and the run refuses until they are set (Jauhar: *"No default — I set
//! it every run"*). See [`crate::modules::param_open`].

use crate::modules::{
    log_in, log_out_as, log_out_flag_as, opt_labelled, param, param_open, param_open_when,
    FlagCurve, FlagKind, FlagValue, ModuleContext, ModuleOutputs, ModuleSpec,
    PROJECT_DEPTH_UNIT_TOKEN,
};
use serde::Serialize;
use std::collections::HashMap;

const MISSING: f32 = f32::NAN;

// ---------------------------------------------------------------------------
// Shared machinery
// ---------------------------------------------------------------------------

/// The finite-depth samples of a run, compacted in order, plus the depth-window sweep.
///
/// Compacting first is what makes the two-pointer sweep correct: a NaN depth compares false
/// against everything, so a pointer walking the raw column would stop dead at the first one and
/// every window below it would be silently short. Samples with no depth have no place on a depth
/// window and are simply passed through unchanged by every caller.
struct Frame {
    /// Index into the run's full-length arrays, ascending.
    idx: Vec<usize>,
    /// The depth of `idx[k]`, ascending (the frame is read `ORDER BY depth`).
    dep: Vec<f64>,
}

impl Frame {
    fn new(depth: &[f32]) -> Frame {
        let mut idx = Vec::new();
        let mut dep = Vec::new();
        for (i, d) in depth.iter().enumerate() {
            if d.is_finite() {
                idx.push(i);
                dep.push(*d as f64);
            }
        }
        Frame { idx, dep }
    }

    fn len(&self) -> usize {
        self.idx.len()
    }

    /// For each compacted sample `k`, the half-open range `[lo, hi)` of compacted samples whose
    /// depth lies within ±`half` of it. O(n) — both pointers are monotone because the centre
    /// depth is.
    ///
    /// The range always contains `k` itself, so a window narrower than the sample spacing degrades
    /// to "this sample only" rather than to an empty set. That matters: an empty window would make
    /// a median MISSING and a too-small window would then blank the curve instead of leaving it
    /// alone, which is a far louder failure than the one the user made.
    fn windows(&self, half: f64) -> Vec<(usize, usize)> {
        let n = self.dep.len();
        let mut out = Vec::with_capacity(n);
        let (mut lo, mut hi) = (0usize, 0usize);
        for k in 0..n {
            let c = self.dep[k];
            while lo < k && self.dep[lo] < c - half {
                lo += 1;
            }
            if hi < k + 1 {
                hi = k + 1;
            }
            while hi < n && self.dep[hi] <= c + half {
                hi += 1;
            }
            out.push((lo, hi));
        }
        out
    }
}

/// Median of the finite values among `vals[idx[lo..hi]]`, through the project's one percentile
/// implementation so a median here is the same operation as a median in a histogram, a box plot,
/// a pore-size distribution or a Monte Carlo band. NaN when the window holds nothing finite.
fn window_median(vals: &[f32], idx: &[usize], lo: usize, hi: usize, buf: &mut Vec<f32>) -> f32 {
    buf.clear();
    for &i in &idx[lo..hi] {
        let v = vals[i];
        if v.is_finite() {
            buf.push(v);
        }
    }
    if buf.is_empty() {
        return MISSING;
    }
    buf.sort_by(|a, b| a.partial_cmp(b).expect("finite by construction"));
    crate::distribution::percentile(buf, 50.0)
}

/// The robust spread of a window about `centre` — the scale the Hampel test measures a sample
/// against.
///
/// Median absolute deviation, scaled by [`crate::robust::C_MAD`] so it estimates the standard
/// deviation of a normal population. That scaling is what makes one `K` readable as "this many
/// deviations out" on GR, RHOB, NPHI and RT alike, rather than a different number per curve.
///
/// **With a fall-back, because MAD IMPLODES.** It is zero whenever more than half the window is
/// identical — a quiet interval, a curve quantized to a coarse step, a tool sitting on its rail —
/// and a zero scale makes every sample infinitely many deviations out, or none, depending on
/// which arbitrary choice the code makes. That is not a corner case: a single spike in an
/// otherwise flat window gives MAD = 0 EXACTLY, so the classic implementation fails on the
/// cleanest possible example of the thing it exists to find.
///
/// The fall-back is the MEAN absolute deviation about the same median, which only collapses when
/// the window is constant INCLUDING the centre — and there is genuinely nothing to reject then.
/// It is less resistant to a second spike inside the same window, which is exactly the trade
/// worth making: the alternative is not resisting anything at all.
///
/// A window of three samples cannot support this test — with one spike among two neighbours the
/// mean deviation is a third of the spike, so a K of 3 lands precisely on the boundary and the
/// answer is decided by a rounding bit. [`MIN_HAMPEL_SAMPLES`] is the floor, and the run refuses
/// rather than returning a coin toss.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DespikeEstimator {
    TrueMad,
    MeanDeviationFallback,
    /// Not offered by SandiBumi. Kept as a distinct contract branch so a future method cannot
    /// inherit one of the Hampel formulae merely because its parameter is also named K.
    #[allow(dead_code)]
    MeanSigmaPopulation,
}

#[derive(Debug, Clone, Copy)]
struct WindowSpread {
    value: f32,
    estimator: DespikeEstimator,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DespikeContaminationBranch {
    pub estimator: DespikeEstimator,
    pub ceiling_pct: f64,
    pub sample_count: usize,
}

/// The contamination ceiling belonging to one explicitly named estimator branch. These are
/// mathematical properties of the estimators, not field cutoffs or calibration defaults.
pub fn contamination_ceiling_pct(estimator: DespikeEstimator, k: f64) -> Result<f64, String> {
    if !k.is_finite() || k <= 0.0 {
        return Err("K must be a finite number greater than zero before a contamination ceiling can be computed".into());
    }
    let fraction = match estimator {
        DespikeEstimator::TrueMad => 0.5,
        DespikeEstimator::MeanDeviationFallback => (1.0 / k).min(0.5),
        DespikeEstimator::MeanSigmaPopulation => 1.0 / (k * k + 1.0),
    };
    Ok(fraction * 100.0)
}

fn window_spread(
    vals: &[f32],
    idx: &[usize],
    lo: usize,
    hi: usize,
    centre: f32,
    buf: &mut Vec<f32>,
) -> WindowSpread {
    buf.clear();
    for &i in &idx[lo..hi] {
        let v = vals[i];
        if v.is_finite() {
            buf.push((v - centre).abs());
        }
    }
    if buf.is_empty() {
        return WindowSpread { value: MISSING, estimator: DespikeEstimator::MeanDeviationFallback };
    }
    buf.sort_by(|a, b| a.partial_cmp(b).expect("finite by construction"));
    let mad = crate::robust::C_MAD as f32 * crate::distribution::percentile(buf, 50.0);
    if mad > 0.0 {
        return WindowSpread { value: mad, estimator: DespikeEstimator::TrueMad };
    }
    let mean_dev = buf.iter().map(|v| *v as f64).sum::<f64>() / buf.len() as f64;
    WindowSpread {
        value: mean_dev as f32,
        estimator: DespikeEstimator::MeanDeviationFallback,
    }
}

/// DEC-077 (2026-08-19): an estimator property, not rock — ruled the owner's convention with
/// practitioner attribution per DEC-059; it stays a code constant because it bounds when the
/// estimator is meaningful at all, which is not an interpreter dial.
/// Fewest samples a HAMPEL window may cover. Below this the spread estimate is dominated by the
/// very sample being judged (see [`window_spread`]), so the test is not measuring anything.
const MIN_HAMPEL_SAMPLES: usize = 5;

fn validate_hampel_window(
    frame: &Frame,
    wins: &[(usize, usize)],
    window: f64,
) -> Result<(), String> {
    if frame.idx.is_empty() {
        return Ok(());
    }
    let mut widths: Vec<usize> = wins.iter().map(|(lo, hi)| hi - lo).collect();
    widths.sort_unstable();
    let typical = widths[widths.len() / 2];
    if typical >= MIN_HAMPEL_SAMPLES {
        return Ok(());
    }
    let spacing = if frame.len() > 1 {
        (frame.dep[frame.len() - 1] - frame.dep[0]) / (frame.len() - 1) as f64
    } else {
        0.0
    };
    Err(format!(
        "WINDOW = {window} covers about {typical} samples at this well's {spacing:.3} \
         sampling, and the HAMPEL test needs at least {MIN_HAMPEL_SAMPLES}: below that \
         the spread it measures against is set by the very sample being judged. Widen \
         WINDOW to about {:.3}, or use ABS, which needs no spread estimate.",
        spacing * MIN_HAMPEL_SAMPLES as f64
    ))
}

/// Inspect the same windows and spread branch the Hampel run will use, returning only branch
/// names, mathematical ceilings and counts. Curve samples stay in Rust when this is used by the
/// dialog preflight; no curve array is serialized over IPC.
pub fn despike_contamination_profile(
    depth: &[f32],
    values: &[f32],
    window: f64,
    k: f64,
) -> Result<Vec<DespikeContaminationBranch>, String> {
    if depth.len() != values.len() {
        return Err("DEPTH and CURVE must have equal lengths for the despike ceiling preview".into());
    }
    if !window.is_finite() || window <= 0.0 {
        return Err("WINDOW must be a finite thickness greater than zero before the despike ceiling can be previewed".into());
    }
    let fallback_ceiling =
        contamination_ceiling_pct(DespikeEstimator::MeanDeviationFallback, k)?;
    let true_mad_ceiling = contamination_ceiling_pct(DespikeEstimator::TrueMad, k)?;
    let frame = Frame::new(depth);
    let wins = frame.windows(window / 2.0);
    validate_hampel_window(&frame, &wins, window)?;

    let mut true_mad_samples = 0usize;
    let mut fallback_samples = 0usize;
    let mut median_buf = Vec::new();
    let mut spread_buf = Vec::new();
    for (frame_index, &(lo, hi)) in wins.iter().enumerate() {
        let sample = frame.idx[frame_index];
        if !values[sample].is_finite() {
            continue;
        }
        let centre = window_median(values, &frame.idx, lo, hi, &mut median_buf);
        if !centre.is_finite() {
            continue;
        }
        match window_spread(values, &frame.idx, lo, hi, centre, &mut spread_buf).estimator {
            DespikeEstimator::TrueMad => true_mad_samples += 1,
            DespikeEstimator::MeanDeviationFallback => fallback_samples += 1,
            DespikeEstimator::MeanSigmaPopulation => unreachable!("Hampel never uses mean-sigma"),
        }
    }

    let mut branches = Vec::with_capacity(2);
    if true_mad_samples > 0 {
        branches.push(DespikeContaminationBranch {
            estimator: DespikeEstimator::TrueMad,
            ceiling_pct: true_mad_ceiling,
            sample_count: true_mad_samples,
        });
    }
    if fallback_samples > 0 {
        branches.push(DespikeContaminationBranch {
            estimator: DespikeEstimator::MeanDeviationFallback,
            ceiling_pct: fallback_ceiling,
            sample_count: fallback_samples,
        });
    }
    Ok(branches)
}

/// A run parameter that is CONSTANT for the well — the window of a filter, the limit of a gap.
///
/// Reads the value at the first sample that has one. These are zone-overridable like every other
/// Param, but a filter window genuinely cannot vary sample by sample: the window at a zone
/// boundary would have to be two thicknesses at once, and the two answers either side would
/// disagree about the same rock. Taking one value and saying so beats producing a seam nobody can
/// see.
fn constant(ctx: &ModuleContext, name: &str) -> f64 {
    (0..ctx.n).map(|i| ctx.p(name, i)).find(|v| v.is_finite()).unwrap_or(f64::NAN)
}

/// `YES`/`NO` option → bool, defaulting to the safe answer when the key is absent.
fn yes(ctx: &ModuleContext, name: &str, default: bool) -> bool {
    match ctx.o(name) {
        "YES" => true,
        "NO" => false,
        _ => default,
    }
}

/// The shared trailing args of every Condition module: the output name and the changed-sample
/// flag. One definition so the wording cannot drift between five dialogs.
fn out_args(
    flag_desc: &str,
    flag_suffix: &str,
    flag_pattern: &str,
    flag_kind: Option<FlagKind>,
) -> Vec<crate::modules::ArgSpec> {
    let companion = match flag_kind {
        Some(kind) => log_out_flag_as("OUT_FLAG", flag_pattern, flag_suffix, kind),
        None => log_out_as("OUT_FLAG", flag_pattern, flag_suffix, ""),
    };
    vec![
        opt_labelled(
            "OPT_FLAG",
            flag_desc,
            "YES",
            &[
                ("YES", "YES — write the flag curve"),
                ("NO", "NO — the conditioned curve only"),
            ],
        ),
        log_out_as("OUT_CURVE", "{CURVE}_C", "Conditioned curve", ""),
        companion,
    ]
}

/// SB-ENV-033 (DEC-034 constraint 1): the Hampel fallback-used diagnostic is its OWN typed
/// channel and is never `OUT_FLAG` - a sample whose value was replaced and a sample judged on a
/// fallback scale are different statements, and one channel cannot carry both.
fn fallback_scale_arg() -> crate::modules::ArgSpec {
    log_out_flag_as(
        "OUT_FBSCALE",
        "{CURVE}_FBSCALE",
        "Hampel scale diagnostic: 1 = judged on the mean-deviation fallback scale (zero-MAD \
         window), 0 = judged on the true MAD, MISSING where no judgement was made",
        FlagKind::DiagnosticIndicator,
    )
}

/// `SB-ENV-037`: the bit-exact recovery record for one conditioning operation.
///
/// Carries the ORIGINAL value at every sample the operation changed, and `MISSING` everywhere
/// else. A record naming only WHICH samples changed is not a recovery record — restoring needs
/// the value that was there, and the shipped flag channel alone cannot supply it.
///
/// **The flag is what disambiguates a restored absence, so the two travel together or neither
/// means anything.** `fill_gaps` changes samples whose original WAS missing, so its record is
/// `MISSING` exactly where it restores; read without the companion flag, "the original was
/// absent" and "this sample was never touched" are the same bits.
///
/// Bit custody is literal, per `DEC-035`: the original `f32` is carried through unchanged, so a
/// NaN arrives back with the payload it had rather than a canonical quiet NaN elected here.
///
/// Scoped to the operations the pilot ships. `smooth` changes every sample, so its record would
/// be the whole input curve — a different shape, deliberately not folded in here — and `flip` is
/// analytically invertible. Culling's recovery ships with culling under the deferred
/// `SB-ENV-036`.
fn recovery_record(original: &[f32], flag: &crate::modules::FlagCurve) -> Vec<f32> {
    (0..original.len())
        .map(|i| if flag.is_flagged(i) { original[i] } else { MISSING })
        .collect()
}

/// The one output declaration for [`recovery_record`], so its name cannot drift between the
/// three specs that carry it.
fn recovery_arg() -> crate::modules::ArgSpec {
    log_out_as(
        "OUT_ORIG",
        "{OUT_CURVE}_ORIG",
        "Original values at every changed sample",
        "",
    )
}

// ---------------------------------------------------------------------------
// DESPIKE
// ---------------------------------------------------------------------------

pub fn despike_spec() -> ModuleSpec {
    ModuleSpec {
        name: "despike".into(),
        title: "Despike".into(),
        category: "Condition".into(),
        doc: "Replaces samples that stand off their neighbours with the local median. WINDOW is a \
              THICKNESS, not a sample count, so it means the same amount of rock whatever the \
              sampling.\n\n\
              **Set WINDOW narrower than the thinnest bed you intend to keep.** A spike is a tool \
              artefact and a thin bed is rock; the only thing separating them here is thickness \
              against the window, so a bed no thicker than the window is indistinguishable from a \
              spike and will be flattened. Even a bed comfortably wider than the window loses its \
              top and bottom sample to the shoulder, where the window straddles the contact.\n\n\
              METHOD:\n\
              • HAMPEL — replace when the sample is more than K robust deviations from the window \
              median (the deviation is the cited Gaussian consistency constant x MAD, so one K \
              reads the same on GR, RHOB, NPHI and RT). Needs a WINDOW covering at least five \
              samples, and the run refuses a \
              narrower one: below that the spread being measured against is set by the very \
              sample under test. Where more than half the window is identical — a quiet interval, \
              a coarsely quantized curve, a tool on its rail — the MAD is zero and the mean \
              deviation is used instead, so a lone spike in flat rock is still found.\n\
              • ABS — replace when it is more than THRESH away, in the curve's own units.\n\
              • MEDIAN — replace every sample with the window median, no test. Changes samples \
              that were fine, which is why the flag curve is near-useless for this method.\n\
              • RATE — replace when the change from the previous live sample exceeds MAX_RATE per \
              depth unit. Catches the step (a stuck tool, a bad splice) that a median window can \
              miss when several bad samples sit together.\n\n\
              WINDOW has no default: what counts as a spike is a property of the tool, the \
              sampling and the rock, and no one number is right in two basins."
            .into(),
        args: {
            let mut a = vec![
                crate::modules::with_sources(
                    param_open(
                        "WINDOW",
                        "Filter window (thickness, centred)",
                        PROJECT_DEPTH_UNIT_TOKEN,
                        0.0,
                        1000.0,
                        true,
                    ),
                    crate::param_sources::CONDITIONING_WINDOW,
                ),
                // K = 3 is the ordinary three-deviation convention (the same generic statistical
                // choice as Tukey's 1.5 x IQR already used in `distribution.rs`), NOT a field
                // calibration — round, and stated as such. DEC-077 ruled it a shipping starting
                // value; it is inert unless OPT_METHOD = HAMPEL.
                param(
                    "K",
                    "HAMPEL: deviations from the median before a sample is a spike",
                    "",
                    3.0,
                    0.5,
                    20.0,
                    "Ordinary three-deviation convention (same family as Tukey 1.5 x IQR in distribution.rs), NOT a field calibration; ruled a shipping starting value by Jauhar adjudication DEC-077 (2026-08-19); docs/takeover/DECISIONS.md",
                ),
                param_open_when(
                    "THRESH",
                    "ABS: distance from the median, in the curve's units",
                    "",
                    0.0,
                    1e9,
                    &[("OPT_METHOD", "ABS")],
                    "docs/PRD_v2/20_envcorr-qc.md §5.3 conditioning parameters",
                ),
                param_open_when(
                    "MAX_RATE",
                    "RATE: largest honest change per depth unit",
                    "",
                    0.0,
                    1e9,
                    &[("OPT_METHOD", "RATE")],
                    "docs/PRD_v2/20_envcorr-qc.md §5.3 conditioning parameters",
                ),
                opt_labelled(
                    "OPT_METHOD",
                    "How a spike is told from rock",
                    "HAMPEL",
                    &[
                        ("HAMPEL", "HAMPEL — off its neighbours vs the curve's own noise (K x MAD)"),
                        ("ABS", "ABS — off the local median by more than THRESH"),
                        ("MEDIAN", "MEDIAN — plain median filter, every sample replaced"),
                        ("RATE", "RATE — change per depth unit above MAX_RATE"),
                    ],
                ),
                log_in("CURVE", "Curve to despike", "", "GR", true),
            ];
            a.extend(out_args(
                "Write a flag curve marking every replaced sample",
                "Replaced-sample flag",
                "{OUT_CURVE}_SPK",
                Some(FlagKind::DiagnosticIndicator),
            ));
            a.push(recovery_arg());
            a.push(fallback_scale_arg());
            a
        },
    }
}

pub fn despike(ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    let vals = ctx.log("CURVE");
    let depth = ctx.log("DEPTH");
    let method = ctx.o("OPT_METHOD").to_string();
    // SB-MLA-055, and the reasoning is a step beyond `smooth`'s. Every method here replaces the
    // suspect sample with a local MEDIAN, which on an even-count window interpolates between the
    // two middle codes. But the deeper problem is that a spike has no meaning in a class log: a
    // single sample of facies 5 between two of facies 1 is a thin bed, not an outlier, and there is
    // no reading of the values that can tell the two apart.
    if ctx.input_is_class_curve("CURVE") {
        return Err(format!(
            "{} holds class codes and cannot be despiked - a lone code between two others is a thin \
             bed, not an outlier, and nothing in the values distinguishes them. To remove beds below \
             a thickness, upscale with Frame > Block at OPT_STAT = MODE.",
            ctx.in_curve("CURVE")
        ));
    }
    let window = constant(ctx, "WINDOW");
    if !(window > 0.0) {
        return Err("WINDOW must be set — a despike window is a thickness, and there is no \
                    generic value for it. Set it narrower than the thinnest bed you mean to keep."
            .into());
    }
    let k = constant(ctx, "K");
    let thresh = constant(ctx, "THRESH");
    let max_rate = constant(ctx, "MAX_RATE");
    match method.as_str() {
        "ABS" if !thresh.is_finite() => {
            return Err("THRESH must be set for the ABS method — it is the distance from the \
                        local median, in the curve's own units."
                .into())
        }
        "RATE" if !max_rate.is_finite() => {
            return Err("MAX_RATE must be set for the RATE method — it is the largest change per \
                        depth unit you accept as rock."
                .into())
        }
        _ => {}
    }

    let frame = Frame::new(&depth);
    let wins = frame.windows(window / 2.0);

    // HAMPEL needs enough samples under the window for the spread estimate to be about the
    // NEIGHBOURS rather than about the sample being judged (see `window_spread`). Measured on the
    // typical window rather than the narrowest, so a short window at the very top of the log does
    // not refuse a run that is well posed everywhere else — and refused rather than run, because
    // the answer either side of that boundary is a coin toss and the curve would look fine.
    if matches!(method.as_str(), "HAMPEL" | "") {
        validate_hampel_window(&frame, &wins, window)?;
    }

    let mut out = vals.clone();
    let mut flag = FlagCurve::clear(ctx.n);
    // SB-ENV-033 (DEC-034): per-sample record of WHICH scale judged a Hampel sample. MISSING
    // until a judgement happens, so a refused or non-Hampel run claims nothing.
    let mut fbscale = FlagCurve::missing(ctx.n);
    let mut buf: Vec<f32> = Vec::new();

    // RATE walks the frame in order and needs the PREVIOUS live sample, which is not the previous
    // index when the curve has holes in it.
    let mut prev: Option<(f64, f32)> = None;

    for k_i in 0..frame.len() {
        let i = frame.idx[k_i];
        let v = vals[i];
        let (lo, hi) = wins[k_i];
        if !v.is_finite() {
            // A MISSING sample stays MISSING. Despiking is a rejection, not a repair — inventing
            // a value here would be Fill Gaps doing its job under another module's name, and it
            // would arrive unflagged.
            flag.set(i, FlagValue::Clear);
            continue;
        }
        let med = window_median(&vals, &frame.idx, lo, hi, &mut buf);
        if !med.is_finite() {
            prev = Some((frame.dep[k_i], v));
            continue;
        }
        let reject = match method.as_str() {
            "MEDIAN" => true,
            "ABS" => ((v - med) as f64).abs() > thresh,
            "RATE" => match prev {
                Some((pd, pv)) => {
                    let dz = frame.dep[k_i] - pd;
                    dz > 0.0 && (((v - pv) as f64).abs() / dz) > max_rate
                }
                None => false,
            },
            // HAMPEL and anything unrecognised. Falling back to the declared default rather than
            // to "reject nothing" — a typo in an option must not turn a conditioning run into a
            // silent copy.
            _ => {
                let spread = window_spread(&vals, &frame.idx, lo, hi, med, &mut buf);
                // DEC-034: a zero-MAD window RUNS - all samples identical means there is no
                // scale to judge against, not too little evidence to judge with - and its use
                // of the declared fallback is reported per sample on the separate diagnostic.
                if spread.value.is_finite() {
                    fbscale.set(
                        i,
                        if matches!(spread.estimator, DespikeEstimator::MeanDeviationFallback) {
                            FlagValue::Flagged
                        } else {
                            FlagValue::Clear
                        },
                    );
                }
                // A window that is constant INCLUDING this sample has zero spread and nothing to
                // reject — every sample in it is the median. Guarded rather than left to
                // `0.0 * K = 0.0`, which would flag every sample differing by a rounding step.
                spread.value.is_finite()
                    && spread.value > 0.0
                    && ((v - med) as f64).abs() > k * spread.value as f64
            }
        };
        if reject {
            out[i] = med;
            flag.set(i, FlagValue::Flagged);
        }
        prev = Some((frame.dep[k_i], v));
    }

    let mut res: ModuleOutputs = HashMap::from([("OUT_CURVE".to_string(), out)]);
    // SB-ENV-033 (DEC-034): the diagnostic ships on every Hampel run, NOT behind OPT_FLAG -
    // it is the disclosure that a fallback scale judged a sample, not the replaced-sample flag.
    if !matches!(method.as_str(), "MEDIAN" | "ABS" | "RATE") {
        res.insert("OUT_FBSCALE".to_string(), fbscale.into_f32());
    }
    if yes(ctx, "OPT_FLAG", true) {
        // SB-ENV-037: record built BEFORE the flag is consumed; both are written or neither.
        res.insert("OUT_ORIG".to_string(), recovery_record(&vals, &flag));
        res.insert("OUT_FLAG".to_string(), flag.into_f32());
    }
    Ok(res)
}

// ---------------------------------------------------------------------------
// SMOOTH
// ---------------------------------------------------------------------------

pub fn smooth_spec() -> ModuleSpec {
    ModuleSpec {
        name: "smooth".into(),
        title: "Smooth".into(),
        category: "Condition".into(),
        doc: "Averages a curve over a WINDOW stated as a THICKNESS.\n\n\
              **Despike first.** A least-squares smoother fits whatever is in the window, so over \
              an un-despiked curve it fits the spike — the spike is not removed, it is spread over \
              the window and made to look like rock.\n\n\
              METHOD:\n\
              • MEAN — arithmetic mean of the live samples in the window.\n\
              • MEDIAN — window median; keeps a step edge where a mean would ramp across it.\n\
              • SAVGOL — local quadratic least-squares fit evaluated at the sample. Fitted on the \
              real (depth, value) pairs rather than with the textbook fixed coefficients, which \
              assume even sampling and are wrong on an irregular frame.\n\n\
              A MISSING sample stays MISSING and no window is bridged: smoothing does not fill \
              gaps, because a filled sample is a claim about rock nobody logged. Use Fill Gaps, \
              which marks what it invented."
            .into(),
        args: {
            let mut a = vec![
                crate::modules::with_sources(
                    param_open(
                        "WINDOW",
                        "Smoothing window (thickness, centred)",
                        PROJECT_DEPTH_UNIT_TOKEN,
                        0.0,
                        1000.0,
                        true,
                    ),
                    crate::param_sources::CONDITIONING_WINDOW,
                ),
                opt_labelled(
                    "OPT_METHOD",
                    "How the window is averaged",
                    "MEAN",
                    &[
                        ("MEAN", "MEAN — arithmetic mean over the window"),
                        ("MEDIAN", "MEDIAN — window median, keeps step edges"),
                        ("SAVGOL", "SAVGOL — local quadratic fit (Savitzky-Golay)"),
                    ],
                ),
                log_in("CURVE", "Curve to smooth", "", "GR", true),
            ];
            a.extend(out_args(
                "Write a flag curve marking every sample the smoother changed",
                "Changed-sample flag",
                "{OUT_CURVE}_SPK",
                Some(FlagKind::DiagnosticIndicator),
            ));
            a
        },
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum SmoothingKernel {
    UniformMean,
    WindowMedian,
    LocalQuadraticLeastSquares,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum SmoothingNormalisation {
    DivideByFiniteSampleCount,
    FiniteOrderStatistic,
    LocalLeastSquaresNormalEquations,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum SmoothingEndBehaviour {
    TruncateCenteredWindowToAvailableDepths,
    TruncateCenteredWindowAndUseFiniteMeanIfUnderdetermined,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum SmoothingGapEdgeBehaviour {
    PreserveMissingTargetAndUseFiniteNeighboursWithinWindow,
}

/// The reproducibility declaration attached to every smoothed output by the workflow writer.
///
/// This describes the arithmetic below rather than choosing a new filter. In particular, a live
/// sample's centred window may use finite neighbours on both sides of a MISSING interval, while
/// the MISSING target itself remains MISSING. Stating that edge behavior explicitly prevents the
/// less precise phrase "preserve gaps" from being mistaken for window segmentation.
pub(crate) fn smoothing_policy(method: &str) -> serde_json::Value {
    let (kernel, normalisation, end_behaviour) = match method {
        "MEDIAN" => (
            SmoothingKernel::WindowMedian,
            SmoothingNormalisation::FiniteOrderStatistic,
            SmoothingEndBehaviour::TruncateCenteredWindowToAvailableDepths,
        ),
        "SAVGOL" => (
            SmoothingKernel::LocalQuadraticLeastSquares,
            SmoothingNormalisation::LocalLeastSquaresNormalEquations,
            SmoothingEndBehaviour::TruncateCenteredWindowAndUseFiniteMeanIfUnderdetermined,
        ),
        _ => (
            SmoothingKernel::UniformMean,
            SmoothingNormalisation::DivideByFiniteSampleCount,
            SmoothingEndBehaviour::TruncateCenteredWindowToAvailableDepths,
        ),
    };
    serde_json::json!({
        "schema_version": 1,
        "kernel": kernel,
        "normalisation": normalisation,
        "end_behaviour": end_behaviour,
        "gap_edge_behaviour": SmoothingGapEdgeBehaviour::PreserveMissingTargetAndUseFiniteNeighboursWithinWindow,
    })
}

/// Local quadratic least-squares fit over `[lo, hi)`, evaluated at `centre` depth.
///
/// Fitted on the actual depths rather than on sample index: the classic Savitzky-Golay
/// coefficients are derived for an evenly spaced window, and a log frame that has been spliced,
/// depth-shifted or merged is not evenly spaced. Depths are taken relative to the centre so the
/// normal matrix stays well conditioned — absolute depths of 3000 give a x^4 term near 1e14 and
/// the solve loses most of its precision.
fn savgol_at(dep: &[f64], vals: &[f32], idx: &[usize], lo: usize, hi: usize, centre: f64) -> f32 {
    // Sums of x^0..x^4 and of y, xy, x^2 y.
    let (mut s0, mut s1, mut s2, mut s3, mut s4) = (0.0f64, 0.0, 0.0, 0.0, 0.0);
    let (mut t0, mut t1, mut t2) = (0.0f64, 0.0, 0.0);
    for k in lo..hi {
        let v = vals[idx[k]];
        if !v.is_finite() {
            continue;
        }
        let x = dep[k] - centre;
        let (x2, x3, x4) = (x * x, x * x * x, x * x * x * x);
        s0 += 1.0;
        s1 += x;
        s2 += x2;
        s3 += x3;
        s4 += x4;
        let y = v as f64;
        t0 += y;
        t1 += x * y;
        t2 += x2 * y;
    }
    // A quadratic needs three points. Below that the fit is under-determined and the honest
    // answer is the mean of what is there, not a curve drawn through two points and extrapolated.
    if s0 < 3.0 {
        return if s0 > 0.0 { (t0 / s0) as f32 } else { MISSING };
    }
    // 3x3 normal equations, solved by Cramer's rule.
    let det = s0 * (s2 * s4 - s3 * s3) - s1 * (s1 * s4 - s3 * s2) + s2 * (s1 * s3 - s2 * s2);
    if !det.is_finite() || det.abs() < 1e-30 {
        return (t0 / s0) as f32;
    }
    // Only the constant term is wanted: the fit is evaluated AT the centre, where x = 0.
    let a0 = t0 * (s2 * s4 - s3 * s3) - s1 * (t1 * s4 - s3 * t2) + s2 * (t1 * s3 - s2 * t2);
    (a0 / det) as f32
}

pub fn smooth(ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    let vals = ctx.log("CURVE");
    let depth = ctx.log("DEPTH");
    let method = ctx.o("OPT_METHOD").to_string();
    // SB-MLA-055. Refused outright rather than coerced, because unlike blocking there is no safe
    // version of this: smoothing means taking values BETWEEN the ones measured, and there is
    // nothing between facies 2 and facies 3. A moving mode over a class log would be a different
    // operation with a different name, not a smoother.
    if ctx.input_is_class_curve("CURVE") {
        return Err(format!(
            "{} holds class codes and cannot be smoothed - smoothing produces values between the \
             ones measured, and there is nothing between facies 2 and facies 3. To simplify a class \
             log, upscale it with Frame > Block at OPT_STAT = MODE.",
            ctx.in_curve("CURVE")
        ));
    }
    let window = constant(ctx, "WINDOW");
    if !(window > 0.0) {
        return Err("WINDOW must be set — a smoothing window is a thickness, and how much \
                    detail it is honest to remove depends on the tool and the beds."
            .into());
    }

    let frame = Frame::new(&depth);
    let wins = frame.windows(window / 2.0);
    let mut out = vals.clone();
    let mut flag = FlagCurve::clear(ctx.n);
    let mut buf: Vec<f32> = Vec::new();

    for k_i in 0..frame.len() {
        let i = frame.idx[k_i];
        if !vals[i].is_finite() {
            continue;
        }
        let (lo, hi) = wins[k_i];
        let sm = match method.as_str() {
            "MEDIAN" => window_median(&vals, &frame.idx, lo, hi, &mut buf),
            "SAVGOL" => savgol_at(&frame.dep, &vals, &frame.idx, lo, hi, frame.dep[k_i]),
            _ => {
                let mut sum = 0.0f64;
                let mut n = 0u32;
                for &j in &frame.idx[lo..hi] {
                    if vals[j].is_finite() {
                        sum += vals[j] as f64;
                        n += 1;
                    }
                }
                if n == 0 {
                    MISSING
                } else {
                    (sum / n as f64) as f32
                }
            }
        };
        if sm.is_finite() {
            if sm != vals[i] {
                flag.set(i, FlagValue::Flagged);
            }
            out[i] = sm;
        }
    }

    let mut res: ModuleOutputs = HashMap::from([("OUT_CURVE".to_string(), out)]);
    if yes(ctx, "OPT_FLAG", true) {
        res.insert("OUT_FLAG".to_string(), flag.into_f32());
    }
    Ok(res)
}

// ---------------------------------------------------------------------------
// CLIP
// ---------------------------------------------------------------------------

pub fn clip_spec() -> ModuleSpec {
    ModuleSpec {
        name: "clip".into(),
        title: "Clip".into(),
        category: "Condition".into(),
        doc: "Holds a curve inside a range. MIN and MAX are in the curve's own units and either \
              may be left EMPTY, which is a statement that the curve is unbounded on that side \
              rather than an omission.\n\n\
              ACTION:\n\
              • BLANK — a sample outside the range becomes MISSING. The right answer when the \
              range is a validity check: a resistivity of 1e6 is not a very resistive rock, it is \
              a reading the tool could not make, and pinning it to the bound would leave a real \
              number where there is no measurement.\n\
              • CLAMP — a sample outside the range is pulled to the bound. Only defensible when \
              the excursion is a small arithmetic overshoot of a known physical limit, the way \
              PHIE is floored at 0.001 rather than blanked.\n\n\
              BLANK is the default because it is the one that cannot manufacture a measurement."
            .into(),
        args: {
            let mut a = vec![
                param_open("MIN", "Lowest honest value — blank for no lower bound", "", -1e9, 1e9, false),
                param_open("MAX", "Highest honest value — blank for no upper bound", "", -1e9, 1e9, false),
                opt_labelled(
                    "OPT_ACTION",
                    "What happens to a sample outside the range",
                    "BLANK",
                    &[
                        ("BLANK", "BLANK — outside the range becomes MISSING"),
                        ("CLAMP", "CLAMP — outside the range is pulled to the bound"),
                    ],
                ),
                log_in("CURVE", "Curve to clip", "", "GR", true),
            ];
            a.extend(out_args(
                "Write a flag curve marking every sample outside the range",
                "Out-of-range flag",
                "{OUT_CURVE}_CLP",
                Some(FlagKind::DiagnosticIndicator),
            ));
            a.push(recovery_arg());
            a
        },
    }
}

pub fn clip(ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    let vals = ctx.log("CURVE");
    let lo_b = constant(ctx, "MIN");
    let hi_b = constant(ctx, "MAX");
    if !lo_b.is_finite() && !hi_b.is_finite() {
        return Err("Set MIN, MAX or both — with neither bound given there is nothing to clip \
                    and the run would copy the curve under a new name."
            .into());
    }
    if lo_b.is_finite() && hi_b.is_finite() && lo_b > hi_b {
        return Err(format!(
            "MIN ({lo_b}) is above MAX ({hi_b}) — refused rather than swapped, because a reversed \
             pair is a typo or the wrong curve, and guessing which hides it."
        ));
    }
    let blank = ctx.o("OPT_ACTION") != "CLAMP";

    let mut out = vals.clone();
    let mut flag = FlagCurve::clear(ctx.n);
    for i in 0..ctx.n {
        let v = vals[i];
        if !v.is_finite() {
            continue;
        }
        let below = lo_b.is_finite() && (v as f64) < lo_b;
        let above = hi_b.is_finite() && (v as f64) > hi_b;
        if below || above {
            flag.set(i, FlagValue::Flagged);
            out[i] = if blank {
                MISSING
            } else if below {
                lo_b as f32
            } else {
                hi_b as f32
            };
        }
    }

    let mut res: ModuleOutputs = HashMap::from([("OUT_CURVE".to_string(), out)]);
    if yes(ctx, "OPT_FLAG", true) {
        // SB-ENV-037: the record is built BEFORE the flag is consumed, and both are written or
        // neither is — a recovery record without its flag cannot be read back.
        res.insert("OUT_ORIG".to_string(), recovery_record(&vals, &flag));
        res.insert("OUT_FLAG".to_string(), flag.into_f32());
    }
    Ok(res)
}

// ---------------------------------------------------------------------------
// FILL GAPS
// ---------------------------------------------------------------------------

pub fn fill_gaps_spec() -> ModuleSpec {
    ModuleSpec {
        name: "fill_gaps".into(),
        title: "Fill Gaps".into(),
        category: "Condition".into(),
        doc: "Fills holes in a curve that are no wider than MAX_GAP, and marks every sample it \
              invented in <OUT>_FILL.\n\n\
              A filled sample is not a measurement. That is the whole reason for the flag curve \
              (Jauhar, 2026-08-05): without it a filled value is indistinguishable from a logged \
              one in a crossplot, a histogram, a net count or a report, and the person reading the \
              number is not the person who chose the limit. Mask on <OUT>_FILL to take them back \
              out of any run.\n\n\
              **A gap open at one end is never filled.** A hole at the top or the bottom of the \
              curve has live data on one side only, so filling it is extrapolation — inventing \
              rock past where the tool stopped. Only a gap bounded above AND below is a candidate.\n\n\
              METHOD:\n\
              • LINEAR — a straight line between the live samples either side.\n\
              • HOLD — the last live value carried down. The honest choice for a curve that is \
              blocky by nature (a facies code, a flag, a zone constant), where a ramp would draw a \
              transition the rock does not have.\n\n\
              MAX_GAP has no default: how far it is defensible to interpolate depends on why the \
              data is missing and on the bed thickness, and no single value is right twice."
            .into(),
        args: {
            let mut a = vec![
                param_open(
                    "MAX_GAP",
                    "Widest hole that may be filled (thickness)",
                    PROJECT_DEPTH_UNIT_TOKEN,
                    0.0,
                    10000.0,
                    true,
                ),
                opt_labelled(
                    "OPT_METHOD",
                    "How the hole is filled",
                    "LINEAR",
                    &[
                        ("LINEAR", "LINEAR — straight line between the live samples either side"),
                        ("HOLD", "HOLD — carry the last live value down"),
                    ],
                ),
                log_in("CURVE", "Curve to fill", "", "GR", true),
            ];
            a.extend(out_args(
                "Write a flag curve marking every invented sample",
                "Filled-sample flag",
                "{OUT_CURVE}_FILL",
                Some(FlagKind::DiagnosticIndicator),
            ));
            a.push(recovery_arg());
            a
        },
    }
}

pub fn fill_gaps(ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    let vals = ctx.log("CURVE");
    let depth = ctx.log("DEPTH");
    let max_gap = constant(ctx, "MAX_GAP");
    if !(max_gap > 0.0) {
        return Err("MAX_GAP must be set — how far it is defensible to interpolate across a hole \
                    depends on why the data is missing and on the beds, and there is no generic \
                    answer."
            .into());
    }
    let hold = ctx.o("OPT_METHOD") == "HOLD";

    let frame = Frame::new(&depth);
    let mut out = vals.clone();
    let mut flag = FlagCurve::clear(ctx.n);

    // Walk the frame finding runs of MISSING bounded by live samples on BOTH sides. A run that
    // reaches either end of the frame is skipped — it has no second anchor, and filling it would
    // be extrapolation past where the tool logged.
    let n = frame.len();
    let mut k = 0usize;
    while k < n {
        if vals[frame.idx[k]].is_finite() {
            k += 1;
            continue;
        }
        let start = k;
        while k < n && !vals[frame.idx[k]].is_finite() {
            k += 1;
        }
        let end = k; // first live sample after the run, or n
        if start == 0 || end >= n {
            continue;
        }
        let (d0, v0) = (frame.dep[start - 1], vals[frame.idx[start - 1]] as f64);
        let (d1, v1) = (frame.dep[end], vals[frame.idx[end]] as f64);
        // The gap is measured between the LIVE samples either side, not between the first and
        // last missing one: that span is what actually goes unmeasured, and it is the number the
        // user is judging when they set a limit.
        if (d1 - d0) > max_gap {
            continue;
        }
        for j in start..end {
            let i = frame.idx[j];
            out[i] = if hold {
                v0 as f32
            } else if d1 > d0 {
                (v0 + (v1 - v0) * (frame.dep[j] - d0) / (d1 - d0)) as f32
            } else {
                v0 as f32
            };
            flag.set(i, FlagValue::Flagged);
        }
    }

    let mut res: ModuleOutputs = HashMap::from([("OUT_CURVE".to_string(), out)]);
    if yes(ctx, "OPT_FLAG", true) {
        // SB-ENV-037: record built BEFORE the flag is consumed; both are written or neither.
        res.insert("OUT_ORIG".to_string(), recovery_record(&vals, &flag));
        res.insert("OUT_FLAG".to_string(), flag.into_f32());
    }
    Ok(res)
}

// ---------------------------------------------------------------------------
// FLIP
// ---------------------------------------------------------------------------

pub fn flip_spec() -> ModuleSpec {
    ModuleSpec {
        name: "flip".into(),
        title: "Flip Polarity".into(),
        category: "Condition".into(),
        doc: "Mirrors a curve about a pivot: OUT = 2 x pivot - CURVE. For an SP recorded with the \
              wrong sign convention, or any reading delivered inverted.\n\n\
              PIVOT_FROM:\n\
              • VALUE — mirror about PIVOT, a number you give. The only reproducible choice, and \
              the one to use when the pivot is a physical reference (an SP shale baseline).\n\
              • MIDRANGE — mirror about (min + max) / 2 of this well's own curve.\n\
              • MEAN — mirror about this well's own mean.\n\n\
              MIDRANGE and MEAN are computed PER WELL, so the same run gives each well a different \
              pivot and two wells' flipped curves are no longer on a common scale. That is often \
              what is wanted for a quick look and is almost never what should go into a \
              correlation — the run leaves the pivot it used in the flag curve so it is at least \
              recoverable."
            .into(),
        args: {
            let mut a = vec![
                param_open("PIVOT", "Value to mirror about (PIVOT_FROM = VALUE)", "", -1e9, 1e9, false),
                opt_labelled(
                    "OPT_PIVOT",
                    "Where the mirror line sits",
                    "VALUE",
                    &[
                        ("VALUE", "VALUE — mirror about PIVOT"),
                        ("MIDRANGE", "MIDRANGE — about (min + max) / 2 of this well's curve"),
                        ("MEAN", "MEAN — about this well's own mean"),
                    ],
                ),
                log_in("CURVE", "Curve to flip", "", "SP", true),
            ];
            a.extend(out_args(
                "Write a curve carrying the pivot actually used",
                "Pivot actually used",
                "{OUT_CURVE}_PIV",
                None,
            ));
            a
        },
    }
}

pub fn flip(ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    let vals = ctx.log("CURVE");
    let mode = ctx.o("OPT_PIVOT").to_string();

    let live: Vec<f32> = vals.iter().copied().filter(|v| v.is_finite()).collect();
    let pivot = match mode.as_str() {
        "MIDRANGE" => {
            if live.is_empty() {
                f64::NAN
            } else {
                let lo = live.iter().copied().fold(f32::INFINITY, f32::min) as f64;
                let hi = live.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
                0.5 * (lo + hi)
            }
        }
        "MEAN" => {
            if live.is_empty() {
                f64::NAN
            } else {
                live.iter().map(|v| *v as f64).sum::<f64>() / live.len() as f64
            }
        }
        _ => constant(ctx, "PIVOT"),
    };
    if !pivot.is_finite() {
        return Err(if mode == "VALUE" || mode.is_empty() {
            "PIVOT must be set — mirroring about an unstated value would silently pick zero, and \
             a curve mirrored about the wrong line is still a plausible-looking curve."
                .into()
        } else {
            format!("No live samples to take a {mode} pivot from.")
        });
    }

    let out: Vec<f32> = vals
        .iter()
        .map(|v| if v.is_finite() { (2.0 * pivot - *v as f64) as f32 } else { MISSING })
        .collect();

    let mut res: ModuleOutputs = HashMap::from([("OUT_CURVE".to_string(), out)]);
    if yes(ctx, "OPT_FLAG", true) {
        // Not a flag in the 0/1 sense: a per-well pivot is the one number a reader needs to undo
        // or reproduce the flip, and there is nowhere else in a module run to put it.
        res.insert("OUT_FLAG".to_string(), vec![pivot as f32; ctx.n]);
    }
    Ok(res)
}

// ---------------------------------------------------------------------------
// NORMALIZE
// ---------------------------------------------------------------------------

/// Normalization for ANY curve, and deliberately the ONLY one.
///
/// Jauhar, 2026-08-05: *"dont dupilcates, normalize tools here should be universal for all logs"*.
/// The app already had `gr_normalize`, whose arithmetic is not about gamma rays at all — a
/// two-point percentile map is the same operation on a neutron log, a sonic, a density or a
/// resistivity, and every one of them drifts between tools and between runs in the same way. So
/// this generalizes that module rather than sitting beside it: `gr_normalize` now delegates here
/// and is kept only so saved chains still resolve.
///
/// **The reference pair has NO default and the run refuses without one.** That is the same rule
/// `gr_normalize` already carried and the reason is worth repeating: a reference from one basin is
/// the wrong reference in another, normalized output always looks plausible, and nothing
/// downstream can tell a well normalized to the field's own endpoints from one normalized to
/// somebody else's. It is derived from the field's own multi-well distribution, or from a
/// reference well everyone agrees on, and then used unchanged for every well in the study.
///
/// **The window is the RUN, not the well.** Percentiles are read from the samples this run sees,
/// so masking the run to a common reference interval is what makes two wells comparable — measure
/// one well over its whole logged section and another over a sand and the two P97s are answering
/// different questions.
pub fn normalize_spec() -> ModuleSpec {
    ModuleSpec {
        name: "normalize".into(),
        title: "Normalize".into(),
        category: "Condition".into(),
        doc: "Maps a curve onto a common reference frame so wells can be compared and pooled. \
              Works on ANY curve — the arithmetic is not specific to gamma ray.\n\n\
              METHOD:\n\
              • TWO_POINT — the workhorse. Reads this run's P_LOW and P_HIGH of the curve and \
              maps them linearly onto REF_LOW and REF_HIGH. Percentiles rather than min/max \
              because a single spike sets a min or a max, and one bad sample would then re-scale \
              the whole well.\n\
              • RANGE — the same map from the curve's own MIN and MAX. Reproducible, and \
              spike-sensitive by construction: use it on a curve you have already despiked.\n\
              • MEAN_SD — z-score to REF_MEAN and REF_SD. The right choice when the distribution \
              matters more than its ends (feeding a classifier, comparing shapes).\n\n\
              SPACE: LOG works in log10 and inverts afterwards, which is the honest frame for a \
              resistivity or a permeability — those are read on a log scale, and a linear map \
              stretches the bottom decade out of all proportion to the top. Non-positive samples \
              have no logarithm and become MISSING, and the run says how many.\n\n\
              THE REFERENCE PAIR HAS NO DEFAULT, and that is the point of the module. A pair from \
              one basin is the wrong pair in another; normalized output looks entirely plausible \
              either way, and nothing downstream can catch it. Derive yours from the field's own \
              multi-well distribution or from a reference well everyone agrees on, then use the \
              SAME pair for every well in the study. QC by overlaying the normalized histograms — \
              every well's P_LOW and P_HIGH should coincide."
            .into(),
        args: vec![
            opt_labelled(
                "OPT_METHOD",
                "How the curve is mapped",
                "TWO_POINT",
                &[
                    ("TWO_POINT", "TWO_POINT — percentiles onto a reference pair"),
                    ("RANGE", "RANGE — min and max onto a reference pair"),
                    ("MEAN_SD", "MEAN_SD — z-score onto a reference mean and spread"),
                ],
            ),
            opt_labelled(
                "OPT_SPACE",
                "Linear or logarithmic",
                "LINEAR",
                &[
                    ("LINEAR", "LINEAR — for GR, NPHI, RHOB, DT, a volume fraction"),
                    ("LOG", "LOG — for RT, PERM, anything read on a log scale"),
                ],
            ),
            crate::modules::with_sources(
                param(
                    "P_LOW", "TWO_POINT: low percentile", "%", 3.0, 0.0, 50.0,
                    "docs/workflow_standards.md GR normalization P3/P97; docs/PRD_v2/20_envcorr-qc.md §5.3",
                ),
                crate::param_sources::PERCENTILE_REFERENCE_LOW,
            ),
            crate::modules::with_sources(
                param(
                    "P_HIGH", "TWO_POINT: high percentile", "%", 97.0, 50.0, 100.0,
                    "docs/workflow_standards.md GR normalization P3/P97; docs/PRD_v2/20_envcorr-qc.md §5.3",
                ),
                crate::param_sources::PERCENTILE_REFERENCE_HIGH,
            ),
            param_open_when(
                "REF_LOW", "TWO_POINT / RANGE: reference value at the low end", "", -1e9, 1e9,
                &[("OPT_METHOD", "TWO_POINT"), ("OPT_METHOD", "RANGE")],
                "docs/PRD_v2/20_envcorr-qc.md §5.3 normalization parameters",
            ),
            param_open_when(
                "REF_HIGH", "TWO_POINT / RANGE: reference value at the high end", "", -1e9, 1e9,
                &[("OPT_METHOD", "TWO_POINT"), ("OPT_METHOD", "RANGE")],
                "docs/PRD_v2/20_envcorr-qc.md §5.3 normalization parameters",
            ),
            // A plain z-score IS the generic answer here, unlike the reference pair — mean 0,
            // spread 1 is a definition rather than somebody's field calibration.
            param_open_when(
                "REF_MEAN", "MEAN_SD: reference mean", "", -1e9, 1e9,
                &[("OPT_METHOD", "MEAN_SD")],
                "docs/PRD_v2/20_envcorr-qc.md §5.3 normalization parameters",
            ),
            param_open_when(
                "REF_SD", "MEAN_SD: reference standard deviation", "", 1e-9, 1e9,
                &[("OPT_METHOD", "MEAN_SD")],
                "docs/PRD_v2/20_envcorr-qc.md §5.3 normalization parameters",
            ),
            log_in("CURVE", "Curve to normalize", "", "GR", true),
            log_out_as("OUT_CURVE", "{CURVE}_N", "Normalized curve", ""),
        ],
    }
}

pub fn normalize(ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    let vals = ctx.log("CURVE");
    let log_space = ctx.o("OPT_SPACE") == "LOG";
    let method = ctx.o("OPT_METHOD").to_string();

    // In LOG space every step — the percentiles, the mean, the spread, the map — happens on the
    // logarithm, and the answer is raised back at the end. Taking percentiles linearly and then
    // mapping the logs would be a third thing that is neither.
    let mut live: Vec<f32> = Vec::new();
    for v in vals.iter().filter(|v| v.is_finite()) {
        if !log_space {
            live.push(*v);
        } else if *v > 0.0 {
            live.push(v.log10());
        }
        // A non-positive sample has no logarithm, so it anchors nothing and gets no answer. Left
        // MISSING rather than floored to some small positive number, which would put a made-up
        // value at the very end of the range the whole map is anchored on.
    }
    if live.len() < 2 {
        return Err(format!(
            "NORMALIZE needs at least two live samples of {} and this run has {}. Widen the run, \
             or check the mask.",
            ctx.o("__IN_CURVE"),
            live.len()
        ));
    }

    // (from_low, from_high) → (to_low, to_high): every method reduces to one linear map, so there
    // is one place the arithmetic lives and three ways of choosing its ends.
    let (from_lo, from_hi, to_lo, to_hi) = match method.as_str() {
        "MEAN_SD" => {
            let mean = live.iter().map(|v| *v as f64).sum::<f64>() / live.len() as f64;
            let var = live.iter().map(|v| (*v as f64 - mean).powi(2)).sum::<f64>() / live.len() as f64;
            let sd = var.sqrt();
            if !(sd > 0.0) {
                return Err("NORMALIZE MEAN_SD needs a curve that varies — this run's samples are \
                            all the same value, so there is no spread to scale."
                    .into());
            }
            let (rm, rsd) = (constant(ctx, "REF_MEAN"), constant(ctx, "REF_SD"));
            (mean, mean + sd, rm, rm + rsd)
        }
        _ => {
            let (lo_ref, hi_ref) = (constant(ctx, "REF_LOW"), constant(ctx, "REF_HIGH"));
            if !lo_ref.is_finite() || !hi_ref.is_finite() {
                return Err("REF_LOW and REF_HIGH must both be set — the reference pair IS the \
                            normalization, and there is no value that is right in two fields. \
                            Take it from this field's own multi-well distribution, or from a \
                            reference well, and use the same pair for every well."
                    .into());
            }
            if (hi_ref - lo_ref).abs() < 1e-12 {
                return Err("REF_LOW and REF_HIGH are the same value, which would map every \
                            sample onto one number."
                    .into());
            }
            // `distribution::percentile` takes an ALREADY-SORTED slice — its parameter is named
            // `sorted` and it indexes straight into it. Handing it the samples in depth order
            // returns whatever value happens to sit 3% of the way down the well, which is a number
            // about the drilling order rather than about the distribution, and every normalized
            // curve built on it would look entirely reasonable.
            let mut sorted = live.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let (lo, hi) = if method == "RANGE" {
                (sorted[0] as f64, sorted[sorted.len() - 1] as f64)
            } else {
                let p_lo = crate::distribution::percentile(&sorted, constant(ctx, "P_LOW") as f32) as f64;
                let p_hi = crate::distribution::percentile(&sorted, constant(ctx, "P_HIGH") as f32) as f64;
                (p_lo, p_hi)
            };
            // In LOG space the reference pair is given in the curve's OWN units — nobody quotes a
            // reference resistivity as 0.3010 — so it is taken to the logarithm here rather than
            // asking the user to.
            let (to_lo, to_hi) = if log_space {
                if !(lo_ref > 0.0 && hi_ref > 0.0) {
                    return Err("In LOG space the reference pair must be positive — a logarithm of \
                                zero or a negative number is not a number."
                        .into());
                }
                (lo_ref.log10(), hi_ref.log10())
            } else {
                (lo_ref, hi_ref)
            };
            (lo, hi, to_lo, to_hi)
        }
    };
    if (from_hi - from_lo).abs() < 1e-12 {
        return Err("this run's curve has no spread between its two anchors, so there is nothing \
                    to stretch onto the reference. Widen the run, or check the mask."
            .into());
    }

    let scale = (to_hi - to_lo) / (from_hi - from_lo);
    let mut out = vec![MISSING; ctx.n];
    for i in 0..ctx.n {
        let v = vals[i] as f64;
        if !v.is_finite() {
            continue;
        }
        let x = if log_space {
            if v <= 0.0 {
                continue; // no logarithm, so no answer — never a floored stand-in
            }
            v.log10()
        } else {
            v
        };
        let y = (x - from_lo) * scale + to_lo;
        out[i] = (if log_space { 10f64.powf(y) } else { y }) as f32;
    }
    Ok(HashMap::from([("OUT_CURVE".to_string(), out)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::DepthUnit;

    /// Builds a run context: one input curve on a regular frame, plus options and params.
    fn ctx_for(
        depth: &[f32],
        curve: &[f32],
        params: &[(&str, f64)],
        opts: &[(&str, &str)],
    ) -> ModuleContext {
        let n = depth.len();
        let mut logs = HashMap::new();
        logs.insert("DEPTH".to_string(), depth.to_vec());
        logs.insert("CURVE".to_string(), curve.to_vec());
        let mut ps = HashMap::new();
        for (k, v) in params {
            ps.insert(k.to_string(), vec![*v; n]);
        }
        let mut os: HashMap<String, String> = HashMap::new();
        os.insert("__IN_CURVE".to_string(), "GR".to_string());
        for (k, v) in opts {
            os.insert(k.to_string(), v.to_string());
        }
        ModuleContext { n, logs, params: ps, opts: os, depth_unit: DepthUnit::Metres }
    }

    fn regular(n: usize, step: f32, start: f32) -> Vec<f32> {
        (0..n).map(|i| start + i as f32 * step).collect()
    }

    /// SB-ENV-037 / SB-ENV-T45. Source: `DEC-035` (2026-08-17) — the bit-exact recovery contract
    /// covers the operations the pilot SHIPS (despike, clip, gap fill); culling's recovery ships
    /// with culling under the deferred `SB-ENV-036`, and T45's cull arm is removed citing that
    /// ruling. "Bit-identical" is read literally: the original 32 bits are stored and restored.
    ///
    /// **The subject is the ROUND TRIP, not the record's shape.** A test asserting only that
    /// `OUT_ORIG` holds certain values would pass for a record nothing can actually restore from,
    /// which is the defect this row names — the shipped flag channel already identified WHICH
    /// samples changed and that was never enough.
    #[test]
    fn every_shipped_conditioning_operation_restores_its_input_bit_for_bit_from_its_own_record() {
        // Restoration is the whole contract, so it is written once and applied to all three:
        // where the flag is set, take the recorded original; elsewhere keep the conditioned value.
        fn restore(out: &[f32], orig: &[f32], flag: &[f32]) -> Vec<f32> {
            (0..out.len())
                .map(|i| if flag[i] == 1.0 { orig[i] } else { out[i] })
                .collect()
        }
        // Compared as BITS, not as values: `assert_eq!` on f32 makes every NaN unequal, and NaN is
        // exactly the case that matters here.
        fn bits(v: &[f32]) -> Vec<u32> {
            v.iter().map(|x| x.to_bits()).collect()
        }

        let depth = regular(9, 0.5, 1000.0);

        // A. CLIP, blanking out-of-range samples. The originals are ordinary numbers destroyed by
        //    the operation, so a record that kept only the flag could never bring them back.
        let clip_in = vec![10.0f32, 12.0, 900.0, 11.0, -50.0, 13.0, 14.0, 1000.0, 12.5];
        let out = clip(&ctx_for(
            &depth,
            &clip_in,
            &[("MIN", 0.0), ("MAX", 100.0)],
            &[("OPT_ACTION", "BLANK"), ("OPT_FLAG", "YES")],
        ))
        .expect("the fixture is in range of the module's own guards");
        assert_eq!(
            bits(&restore(&out["OUT_CURVE"], &out["OUT_ORIG"], &out["OUT_FLAG"])),
            bits(&clip_in),
            "clip must restore bit for bit from its own record"
        );

        // B. FILL GAPS, which is the arm that proves why the flag and the record travel together.
        //    Every sample it changes had a MISSING original, so the record is NaN exactly where it
        //    restores. Read without the flag, "the original was absent" and "this sample was never
        //    touched" are the same bits - so a record alone cannot be interpreted.
        // The absent samples carry a NON-CANONICAL quiet NaN payload, and that is deliberate.
        // With a plain `f32::NAN` fixture, "carried the original's bits" and "wrote the module's
        // own MISSING constant" are the same bits, so the round trip would pass for an
        // implementation that had lost the payload entirely - which is exactly what DEC-035's
        // "bit-identical, read literally" forbids. A mutation proved that hole was real.
        let absent = f32::from_bits(0x7FC0_1234);
        assert!(absent.is_nan() && absent.to_bits() != f32::NAN.to_bits());
        let fill_in = vec![10.0f32, 12.0, absent, absent, 18.0, 20.0, 22.0, 24.0, 26.0];
        let out = fill_gaps(&ctx_for(
            &depth,
            &fill_in,
            &[("MAX_GAP", 5.0)],
            &[("OPT_METHOD", "LINEAR"), ("OPT_FLAG", "YES")],
        ))
        .expect("a two-sample hole is inside MAX_GAP");
        let filled = &out["OUT_CURVE"];
        assert!(
            filled[2].is_finite() && filled[3].is_finite(),
            "the fixture must actually fill something, or the round trip proves nothing"
        );
        assert_eq!(
            bits(&restore(filled, &out["OUT_ORIG"], &out["OUT_FLAG"])),
            bits(&fill_in),
            "gap fill must restore the ORIGINAL ABSENCE, NaN included, not merely a number"
        );

        // C. DESPIKE. Same contract on the third shipped operation.
        let spike_in = vec![10.0f32, 10.2, 9.8, 10.1, 250.0, 10.0, 9.9, 10.3, 10.1];
        let out = despike(&ctx_for(
            &depth,
            &spike_in,
            &[("WINDOW", 3.0), ("K", 3.0)],
            &[("OPT_METHOD", "HAMPEL"), ("OPT_FLAG", "YES")],
        ))
        .expect("a nine-sample window clears the Hampel floor");
        assert_eq!(
            bits(&restore(&out["OUT_CURVE"], &out["OUT_ORIG"], &out["OUT_FLAG"])),
            bits(&spike_in),
            "despike must restore bit for bit from its own record"
        );

        // D. The record and its flag are written together or not at all. With the flag declined
        //    there is nothing to interpret a record against, so neither is emitted - and this arm
        //    fails an implementation that emits an uninterpretable record anyway.
        let bare = clip(&ctx_for(
            &depth,
            &clip_in,
            &[("MIN", 0.0), ("MAX", 100.0)],
            &[("OPT_ACTION", "BLANK"), ("OPT_FLAG", "NO")],
        ))
        .unwrap();
        assert!(!bare.contains_key("OUT_FLAG"));
        assert!(
            !bare.contains_key("OUT_ORIG"),
            "a recovery record without its flag cannot be read back and must not be written"
        );
    }
    /// **SB-MLA-055 here is a refusal with no safe alternative, which is why it is a refusal.**
    ///
    /// `frame::block` could offer MODE, so it refuses the averaging statistics and keeps the one
    /// that works. Neither of these has that: smoothing MEANS producing values between the ones
    /// measured, and there is nothing between facies 2 and facies 3. Despiking is worse than
    /// useless — a lone code between two others is a thin bed, and nothing in the numbers
    /// distinguishes a thin bed from an outlier — so a "cleaned" class log is one with its thinnest
    /// beds silently deleted.
    ///
    /// Pinned from both sides. An UNDECLARED curve runs untouched, because the alternative is a
    /// heuristic on the values, and a caliper logged in whole inches or a 0/1 flag would then be
    /// refused a smoothing the user legitimately asked for.
    #[test]
    fn a_class_curve_is_refused_by_smooth_and_despike_and_an_undeclared_one_is_not() {
        let n = 41;
        let depth = regular(n, 0.1, 1000.0);
        // A class log: two beds of facies 1 and 3, with one sample of facies 5 between them. That
        // lone sample is exactly what a despiker would remove and a petrophysicist would log.
        let mut facies = vec![1.0f32; n];
        facies[20] = 5.0;
        for v in facies.iter_mut().skip(21) {
            *v = 3.0;
        }
        let declared: &[(&str, &str)] = &[("__IN_CURVE", "FACIES"), ("__CLASS_CURVES", "FACIES")];
        let params: &[(&str, f64)] = &[("WINDOW", 0.5), ("K", 3.0)];

        for (what, run) in [
            ("smooth", smooth as fn(&ModuleContext) -> Result<ModuleOutputs, String>),
            ("despike", despike),
        ] {
            let e = run(&ctx_for(&depth, &facies, params, declared))
                .expect_err("{what} on a declared class curve must be refused");
            assert!(e.contains("FACIES"), "{what}: names the curve refused: {e}");
            assert!(e.contains("MODE"), "{what}: names where the user should go instead: {e}");

            // The other side. `ctx_for` leaves __IN_CURVE as GR and declares nothing, so the same
            // values — spike and all — are the user's to condition however they asked.
            assert!(
                run(&ctx_for(&depth, &facies, params, &[])).is_ok(),
                "{what}: an undeclared curve is conditioned as asked, whatever its values look like",
            );
        }
    }

    /// **The window is a thickness, and that is the whole reason it is stated in depth.**
    ///
    /// The same rock is logged twice — once every 0.1 m, once every 0.05 m — carrying the same
    /// one-sample spike. A window of 0.5 m must reject it on both. A window expressed in SAMPLES
    /// cannot: five samples is 0.5 m on one frame and 0.25 m on the other, so the same declared
    /// setting covers different amounts of rock and nothing downstream can see that it did.
    ///
    /// The fixture is a NOISELESS background on purpose. It is the cleanest possible statement of
    /// the problem and the case a median-absolute-deviation scale gets exactly wrong: with one
    /// spike among identical neighbours the MAD is zero, so the textbook Hampel test finds no
    /// spike at all. See [`window_spread`].
    #[test]
    fn a_despike_window_covers_the_same_rock_at_any_sampling() {
        for step in [0.1f32, 0.05] {
            let n = 81;
            let depth = regular(n, step, 1000.0);
            let mut curve = vec![50.0f32; n];
            curve[40] = 300.0; // one-sample spike
            let ctx = ctx_for(
                &depth,
                &curve,
                &[("WINDOW", 0.5), ("K", 3.0)],
                &[("OPT_METHOD", "HAMPEL")],
            );
            let out = despike(&ctx).expect("run");
            let got = &out["OUT_CURVE"];
            let flag = &out["OUT_FLAG"];
            assert!(
                (got[40] - 50.0).abs() < 1e-3,
                "step {step}: the spike should be replaced by the local median, got {}",
                got[40]
            );
            assert_eq!(flag[40], 1.0, "step {step}: the replaced sample must be flagged");
            assert_eq!(
                flag.iter().filter(|f| **f == 1.0).count(),
                1,
                "step {step}: exactly one sample was a spike"
            );
        }
    }

    /// CORRECTNESS — SB-ENV-034 / exact SB-ENV-T43 in
    /// `docs/PRD_v2/20_envcorr-qc.md` supplies the 1.0 m window and the 0.1 m / 0.5 m samplings.
    /// The expected window populations (eleven and three) and their common 1.0 m span follow
    /// independently from those values. MEDIAN isolates this physical-window contract from
    /// SB-ENV-T42's separately blocked Hampel minimum-sample/fallback decision.
    ///
    /// The narrow 0.4 m feature and wide 2.0 m bed pin both sides: the former must be removed at
    /// both samplings while the latter survives. A fixed sample-count window can satisfy only one
    /// side when the sampling changes fivefold.
    #[test]
    fn every_conditioning_and_framing_distance_is_physical_thickness_and_a_one_metre_despike_covers_one_metre_at_two_samplings(
    ) {
        use std::collections::BTreeSet;

        let expected: BTreeSet<(String, String)> = [
            ("despike", "WINDOW"),
            ("smooth", "WINDOW"),
            ("fill_gaps", "MAX_GAP"),
            ("block", "INTERVAL"),
            ("block", "MIN_BED"),
            ("bed_detect", "MIN_BED"),
            ("condflag", "MIN_THICK"),
            ("condflag", "SHOULDER"),
        ]
        .into_iter()
        .map(|(module, argument)| (module.to_string(), argument.to_string()))
        .collect();
        let mut declared = BTreeSet::new();
        for module in crate::modules::list_modules().into_iter().filter(|module| {
            matches!(module.category.as_str(), "Condition" | "Frame") || module.name == "condflag"
        }) {
            for argument in module.args.into_iter().filter(|argument| {
                matches!(
                    argument.name.as_str(),
                    "WINDOW" | "MAX_GAP" | "INTERVAL" | "MIN_BED" | "MIN_THICK" | "SHOULDER"
                ) || argument.name.contains("LENGTH")
                    || argument.name.contains("WIDTH")
            }) {
                assert!(
                    matches!(argument.kind, crate::modules::ArgKind::Param),
                    "{}.{} is a physical distance and must remain a numeric parameter",
                    module.name,
                    argument.name
                );
                assert!(
                    argument.unit == PROJECT_DEPTH_UNIT_TOKEN,
                    "{}.{} declares {:?}; SB-ENV-034 permits only the project's depth unit, never samples",
                    module.name,
                    argument.name,
                    argument.unit
                );
                declared.insert((module.name.clone(), argument.name));
            }
        }
        assert_eq!(
            declared, expected,
            "the whole conditioning/framing distance inventory must be reviewed when one is added, removed or renamed"
        );

        let mut fine_result = Vec::new();
        for (step, expected_samples) in [(0.1f32, 11usize), (0.5, 3)] {
            let n = (10.0 / step) as usize + 1;
            let depth = regular(n, step, 0.0);
            let mut curve = vec![0.0f32; n];
            for (index, value) in curve.iter_mut().enumerate() {
                let position = index as f32 * step;
                if (1.8..=2.2).contains(&position) || (5.0..=7.0).contains(&position) {
                    *value = 10.0;
                }
            }

            let centre = (4.0 / step).round() as usize;
            let frame = Frame::new(&depth);
            let (lo, hi) = frame.windows(0.5)[centre];
            assert_eq!(hi - lo, expected_samples, "step {step}: wrong 1.0 m window population");
            assert!(
                ((frame.dep[hi - 1] - frame.dep[lo]) - 1.0).abs() < 1e-6,
                "step {step}: the window spans {} m instead of 1.0 m",
                frame.dep[hi - 1] - frame.dep[lo]
            );

            let output = despike(&ctx_for(
                &depth,
                &curve,
                &[("WINDOW", 1.0)],
                &[("OPT_METHOD", "MEDIAN")],
            ))
            .expect("a physical median window runs at both samplings")["OUT_CURVE"]
                .clone();
            let narrow = (2.0 / step).round() as usize;
            let wide = (6.0 / step).round() as usize;
            assert_eq!(output[narrow], 0.0, "step {step}: a 0.4 m feature survives a 1.0 m window");
            assert_eq!(output[wide], 10.0, "step {step}: a 2.0 m bed is erased by a 1.0 m window");

            if step == 0.1 {
                fine_result = output;
            } else {
                for (coarse_index, coarse_value) in output.iter().copied().enumerate() {
                    assert_eq!(
                        coarse_value,
                        fine_result[coarse_index * 5],
                        "the two results disagree at physical depth {} m",
                        coarse_index as f32 * step
                    );
                }
            }
        }
    }

    /// **A despike must not despike the rock**, and the discriminator is thickness against the
    /// window. A bed comfortably wider than the window survives; a single bad sample does not.
    ///
    /// The bed's own top and bottom sample DO get taken — that is the shoulder, where the window
    /// straddles the contact and the median is still the background — which is exactly why the
    /// module doc says to set the window narrower than the thinnest bed worth keeping. Asserting
    /// the interior rather than the whole bed records the real behaviour instead of an ideal one.
    #[test]
    fn a_bed_wider_than_the_window_survives_a_despike() {
        let n = 101;
        let depth = regular(n, 0.1, 1000.0);
        let mut curve = vec![50.0f32; n];
        for c in curve.iter_mut().take(59).skip(50) {
            *c = 120.0; // a 0.9 m bed against a 0.5 m window
        }
        curve[80] = 300.0; // and a one-sample spike elsewhere
        let ctx = ctx_for(
            &depth,
            &curve,
            &[("WINDOW", 0.5), ("K", 3.0)],
            &[("OPT_METHOD", "HAMPEL")],
        );
        let out = despike(&ctx).expect("run");
        let got = &out["OUT_CURVE"];
        for i in 51..58 {
            assert!(
                (got[i] - 120.0).abs() < 1e-3,
                "sample {i} is inside a bed three times the window and must be left alone, got {}",
                got[i]
            );
        }
        assert!((got[80] - 50.0).abs() < 1e-3, "the one-sample spike must still go");
    }

    /// **The textbook Hampel scale is zero on the cleanest possible spike, and this pins the fix.**
    ///
    /// A single spike among identical neighbours makes more than half the window's deviations
    /// exactly zero, so the median absolute deviation is zero and `|v − m| > K·0` is false for
    /// every sample: the filter finds nothing. `window_spread` falls back to the MEAN deviation,
    /// which only collapses on a genuinely constant window.
    ///
    /// The control matters as much as the case: a window that really is constant must reject
    /// NOTHING, or the fall-back has simply moved the failure to the other extreme and would flag
    /// every sample of a flag curve or a zone constant.
    /// SB-ENV-033 (DEC-034), pinned from BOTH sides: the four-sample window REFUSES with the
    /// shipped named guard (the floor holds - at four samples the spike contributes a quarter
    /// of the scale used to condemn it), while a zero-MAD window RUNS on the declared
    /// mean-deviation fallback and reports that per sample on its OWN typed diagnostic -
    /// never on OUT_FLAG, because a replaced sample and a fallback-judged sample are
    /// different statements.
    #[test]
    fn the_four_sample_window_still_refuses_while_a_zero_mad_window_runs_and_reports_its_fallback_scale(
    ) {
        // A. The undersized window returns the structured refusal - no run, no channel claimed.
        let depth = regular(41, 0.1, 1000.0);
        let noisy: Vec<f32> = (0..41).map(|i| 50.0 + ((i * 7) % 5) as f32).collect();
        let err = despike(&ctx_for(
            &depth,
            &noisy,
            &[("WINDOW", 0.3), ("K", 3.0)],
            &[("OPT_METHOD", "HAMPEL")],
        ))
        .expect_err("a four-sample window must refuse, not run degraded");
        assert!(err.contains("at least 5"), "the refusal names the floor: {err}");

        // B. The zero-MAD window RUNS: a quiet interval with one spike - the MAD is zero, the
        //    mean-deviation fallback finds the spike, and the diagnostic says which scale
        //    judged each sample.
        let mut quiet = vec![50.0f32; 41];
        quiet[20] = 300.0;
        quiet[40] = f32::NAN;
        let out = despike(&ctx_for(
            &depth,
            &quiet,
            &[("WINDOW", 0.5), ("K", 3.0)],
            &[("OPT_METHOD", "HAMPEL")],
        ))
        .expect("a zero-MAD window is a different situation and DOES run");
        assert_eq!(out["OUT_CURVE"][20].to_bits(), 50.0f32.to_bits(), "the spike is repaired");
        let fb = &out["OUT_FBSCALE"];
        assert_eq!(fb[20].to_bits(), 1.0f32.to_bits(), "the spike was judged on the fallback scale");
        assert!(fb[40].is_nan(), "no judgement, no claim - got {}", fb[40]);
        assert!(
            out["OUT_FLAG"].iter().zip(fb.iter()).any(|(flag, fb)| *flag == 0.0 && *fb == 1.0),
            "the diagnostic is its own channel, not a copy of the replaced-sample flag"
        );

        // C. A window with real spread is judged on the TRUE MAD and the diagnostic says so.
        let out = despike(&ctx_for(
            &depth,
            &noisy,
            &[("WINDOW", 0.9), ("K", 8.0)],
            &[("OPT_METHOD", "HAMPEL")],
        ))
        .expect("run");
        assert!(
            out["OUT_FBSCALE"].iter().filter(|v| v.is_finite()).all(|v| *v == 0.0),
            "a true-MAD judgement must read 0 on the diagnostic"
        );

        // D. A non-Hampel method makes no scale judgement and carries no diagnostic at all.
        let out = despike(&ctx_for(
            &depth,
            &noisy,
            &[("WINDOW", 0.9), ("THRESH", 100.0)],
            &[("OPT_METHOD", "ABS")],
        ))
        .expect("run");
        assert!(!out.contains_key("OUT_FBSCALE"), "ABS makes no scale judgement");
    }

    #[test]
    fn a_spike_in_a_quiet_interval_is_still_a_spike() {
        let n = 41;
        let depth = regular(n, 0.1, 1000.0);
        let mut curve = vec![50.0f32; n];
        curve[20] = 300.0;
        let hit = despike(&ctx_for(
            &depth,
            &curve,
            &[("WINDOW", 0.5), ("K", 3.0)],
            &[("OPT_METHOD", "HAMPEL")],
        ))
        .expect("run");
        assert_eq!(
            hit["OUT_FLAG"].iter().filter(|f| **f == 1.0).count(),
            1,
            "a median-absolute-deviation scale is exactly zero here and finds nothing"
        );

        // Control: a perfectly constant curve has nothing to reject at all.
        let flat = vec![50.0f32; n];
        let quiet = despike(&ctx_for(
            &depth,
            &flat,
            &[("WINDOW", 0.5), ("K", 3.0)],
            &[("OPT_METHOD", "HAMPEL")],
        ))
        .expect("run");
        assert_eq!(
            quiet["OUT_FLAG"].iter().filter(|f| **f == 1.0).count(),
            0,
            "a constant window must reject nothing — otherwise the fall-back has just moved the \
             failure to the other end and would eat every flag curve in the project"
        );
    }

    #[test]
    fn the_zero_mad_fallback_ceiling_stops_at_half_and_updates_with_k() {
        // CORRECTNESS — SB-ENV-T40. docs/PRD_v2/20_envcorr-qc.md §2.5 derives the
        // mean-deviation fallback ceiling as min(1/k, 1/2); §6 T40 fixes these three
        // displayed values to 33.33 %, 50.00 % and 50.00 % (±0.01 percentage point).
        let depth = regular(41, 0.1, 1000.0);
        let mut curve = vec![50.0f32; depth.len()];
        curve[20] = 300.0;

        for (k, expected_pct) in [(3.0, 33.333_333), (2.0, 50.0), (1.5, 50.0)] {
            let branches = despike_contamination_profile(&depth, &curve, 0.5, k)
                .expect("the same valid window the despiker accepts must be previewable");
            assert_eq!(branches.len(), 1, "the quiet fixture uses one estimator branch");
            assert_eq!(branches[0].estimator, DespikeEstimator::MeanDeviationFallback);
            assert!(
                (branches[0].ceiling_pct - expected_pct).abs() <= 0.01,
                "k={k}: expected {expected_pct:.2} %, got {:.5} %",
                branches[0].ceiling_pct,
            );
        }
    }

    #[test]
    fn a_positive_mad_window_reports_the_true_mad_ceiling_not_the_fallback_ceiling() {
        // CORRECTNESS — SB-ENV-T69. docs/PRD_v2/20_envcorr-qc.md §2.5 and §6 T69
        // specify 50.00 % for the true-MAD branch at k=3, explicitly not the fallback's
        // 33.33 %. A ramp supplies clean scatter in every evaluated window.
        let depth = regular(41, 0.1, 1000.0);
        let curve: Vec<f32> = (0..depth.len()).map(|i| 50.0 + i as f32 * 0.25).collect();
        let branches = despike_contamination_profile(&depth, &curve, 0.5, 3.0)
            .expect("positive-MAD windows must be previewable");

        assert_eq!(branches.len(), 1, "the scattered fixture uses one estimator branch");
        assert_eq!(branches[0].estimator, DespikeEstimator::TrueMad);
        assert!((branches[0].ceiling_pct - 50.0).abs() <= 0.01);
        assert!(
            (branches[0].ceiling_pct - 33.333_333).abs() > 0.01,
            "the UI must not infer the fallback formula from k alone",
        );
    }

    #[test]
    fn a_future_mean_sigma_estimator_cannot_inherit_a_hampel_ceiling() {
        // CORRECTNESS — SB-ENV-T70. docs/PRD_v2/20_envcorr-qc.md §2.5 derives the
        // population-sigma masking ceiling as 1/(k^2+1), hence exactly 20 % at k=2.
        // The two Hampel branches are asserted beside it so one hard-coded formula cannot pass.
        let mean_sigma = contamination_ceiling_pct(DespikeEstimator::MeanSigmaPopulation, 2.0)
            .expect("positive k");
        let true_mad = contamination_ceiling_pct(DespikeEstimator::TrueMad, 2.0)
            .expect("positive k");
        let fallback = contamination_ceiling_pct(DespikeEstimator::MeanDeviationFallback, 2.0)
            .expect("positive k");

        assert!((mean_sigma - 20.0).abs() <= 0.01);
        assert!((true_mad - 50.0).abs() <= 0.01);
        assert!((fallback - 50.0).abs() <= 0.01);
    }

    /// A HAMPEL window too narrow to hold a spread estimate is REFUSED, not run. With three
    /// samples the deviation the test measures against is a third of the spike being judged, so a
    /// K of 3 lands on the boundary and the answer is decided by a rounding bit — and the curve
    /// that comes out looks entirely normal either way.
    #[test]
    fn a_hampel_window_too_narrow_to_measure_a_spread_is_refused() {
        let n = 41;
        let depth = regular(n, 0.1, 1000.0);
        let mut curve = vec![50.0f32; n];
        curve[20] = 300.0;
        let err = despike(&ctx_for(
            &depth,
            &curve,
            &[("WINDOW", 0.2), ("K", 3.0)],
            &[("OPT_METHOD", "HAMPEL")],
        ))
        .expect_err("must refuse");
        assert!(err.contains("HAMPEL"), "the refusal names the method: {err}");
        assert!(err.contains("ABS"), "and offers the method that needs no spread: {err}");

        // ABS needs no spread estimate, so the same narrow window is fine for it.
        let out = despike(&ctx_for(
            &depth,
            &curve,
            &[("WINDOW", 0.2), ("THRESH", 100.0)],
            &[("OPT_METHOD", "ABS")],
        ))
        .expect("ABS runs on a narrow window");
        assert!((out["OUT_CURVE"][20] - 50.0).abs() < 1e-3);
    }

    /// A despike with no WINDOW is refused rather than run on a guess (Jauhar's "no default").
    #[test]
    fn a_despike_without_a_window_refuses_to_run() {
        let depth = regular(20, 0.1, 1000.0);
        let curve = vec![50.0f32; 20];
        let ctx = ctx_for(&depth, &curve, &[("K", 3.0)], &[("OPT_METHOD", "HAMPEL")]);
        let err = despike(&ctx).expect_err("must refuse");
        assert!(err.contains("WINDOW"), "the refusal must name the parameter: {err}");
    }

    #[test]
    fn a_smoothed_curve_never_fills_a_gap() {
        let n = 41;
        let depth = regular(n, 0.1, 1000.0);
        let mut curve: Vec<f32> = (0..n).map(|i| 50.0 + i as f32).collect();
        for c in curve.iter_mut().take(23).skip(20) {
            *c = f32::NAN;
        }
        for method in ["MEAN", "MEDIAN", "SAVGOL"] {
            let ctx = ctx_for(&depth, &curve, &[("WINDOW", 0.5)], &[("OPT_METHOD", method)]);
            let out = smooth(&ctx).expect("run");
            let got = &out["OUT_CURVE"];
            for i in 20..23 {
                assert!(got[i].is_nan(), "{method}: sample {i} was MISSING and must stay MISSING");
            }
            assert!(got[10].is_finite(), "{method}: live samples are still smoothed");
        }
    }

    /// The Savitzky-Golay branch fits a QUADRATIC, so a quadratic passes through it unchanged —
    /// which a moving mean does not. That is the whole reason to prefer it on a curve with real
    /// curvature: a mean biases a peak downward and a trough upward, every time.
    #[test]
    fn the_savgol_branch_leaves_a_quadratic_alone_where_a_mean_would_not() {
        let n = 41;
        let depth = regular(n, 0.1, 1000.0);
        // y = (d - 1002)^2, sampled on the frame.
        let curve: Vec<f32> = depth.iter().map(|d| ((d - 1002.0) as f64).powi(2) as f32).collect();
        let sg = smooth(
            &ctx_for(&depth, &curve, &[("WINDOW", 0.5)], &[("OPT_METHOD", "SAVGOL")]),
        )
        .expect("run");
        let mean = smooth(
            &ctx_for(&depth, &curve, &[("WINDOW", 0.5)], &[("OPT_METHOD", "MEAN")]),
        )
        .expect("run");
        let i = 5; // away from the ends, where a centred window is complete
        assert!(
            (sg["OUT_CURVE"][i] - curve[i]).abs() < 1e-4,
            "SAVGOL should reproduce a quadratic: {} vs {}",
            sg["OUT_CURVE"][i],
            curve[i]
        );
        assert!(
            (mean["OUT_CURVE"][i] - curve[i]).abs() > 1e-3,
            "a moving mean should NOT — if it does, this test is proving nothing"
        );
    }

    /// **A gap open at one end is never filled**, and a gap wider than the limit is left alone.
    /// Every invented sample is flagged and nothing else is.
    #[test]
    fn fill_gaps_bridges_only_a_bounded_hole_inside_the_limit() {
        let n = 60;
        let depth = regular(n, 0.1, 1000.0);
        let mut curve = vec![10.0f32; n];
        for c in curve.iter_mut().take(3) {
            *c = f32::NAN; // open at the TOP — extrapolation, never filled
        }
        for c in curve.iter_mut().take(22).skip(20) {
            *c = f32::NAN; // 0.3 m hole, inside a 0.5 m limit
        }
        for c in curve.iter_mut().take(50).skip(40) {
            *c = f32::NAN; // 1.1 m hole, outside it
        }
        for c in curve.iter_mut().skip(58) {
            *c = f32::NAN; // open at the BOTTOM
        }
        for i in 22..40 {
            curve[i] = 20.0; // so the bridged values are checkable
        }
        let ctx = ctx_for(&depth, &curve, &[("MAX_GAP", 0.5)], &[("OPT_METHOD", "LINEAR")]);
        let out = fill_gaps(&ctx).expect("run");
        let got = &out["OUT_CURVE"];
        let flag = &out["OUT_FLAG"];

        for i in 0..3 {
            assert!(got[i].is_nan(), "sample {i} is open at the top — filling it is extrapolation");
        }
        for i in 58..60 {
            assert!(got[i].is_nan(), "sample {i} is open at the bottom");
        }
        for i in 40..50 {
            assert!(got[i].is_nan(), "sample {i} sits in a hole wider than MAX_GAP");
        }
        for i in 20..22 {
            assert!(got[i].is_finite(), "sample {i} sits in a hole inside MAX_GAP");
            assert!(
                got[i] > 10.0 && got[i] < 20.0,
                "and is interpolated between 10 and 20, got {}",
                got[i]
            );
            assert_eq!(flag[i], 1.0, "and is marked as invented");
        }
        assert_eq!(
            flag.iter().filter(|f| **f == 1.0).count(),
            2,
            "exactly the two invented samples are flagged — a flag on a measured sample would be \
             as misleading as no flag on an invented one"
        );
    }

    /// CORRECTNESS — SB-ENV-T46 in `docs/PRD_v2/20_envcorr-qc.md` supplies the boundary
    /// contract. The three bounded gaps span MAX_GAP - epsilon, MAX_GAP, and MAX_GAP + epsilon
    /// between their live anchors; epsilon is 0.125 m here so every fixture depth is represented
    /// exactly in binary. One missing row in each gap pins that physical span, not row count, makes
    /// the decision. The same source requires both open ends to remain missing and every inserted
    /// sample to be flagged.
    #[test]
    fn a_gap_at_or_below_the_maximum_is_filled_while_epsilon_over_and_both_open_ends_are_not() {
        let depth = vec![
            0.0, 0.25, 0.5, 0.75, 1.375, 2.0, 2.25, 3.0, 3.5, 3.75, 4.625, 5.0, 5.25,
        ];
        let curve = vec![
            f32::NAN,
            f32::NAN,
            10.0,
            f32::NAN, // live-anchor span 0.875 m: MAX_GAP - epsilon
            20.0,
            30.0,
            f32::NAN, // live-anchor span 1.000 m: exactly MAX_GAP
            50.0,
            60.0,
            f32::NAN, // live-anchor span 1.125 m: MAX_GAP + epsilon
            80.0,
            90.0,
            f32::NAN,
        ];
        let out = fill_gaps(&ctx_for(
            &depth,
            &curve,
            &[("MAX_GAP", 1.0)],
            &[("OPT_METHOD", "LINEAR")],
        ))
        .expect("run");
        let got = &out["OUT_CURVE"];
        let flag = &out["OUT_FLAG"];

        assert!(got[3].is_finite(), "MAX_GAP - epsilon must be filled");
        assert!(got[6].is_finite(), "exactly MAX_GAP must be filled");
        assert!(got[9].is_nan(), "MAX_GAP + epsilon must remain missing");
        assert!(got[0].is_nan() && got[1].is_nan(), "an open top must not be extrapolated");
        assert!(got[12].is_nan(), "an open bottom must not be extrapolated");

        assert_eq!(flag[3], 1.0, "the under-limit inserted sample must be flagged");
        assert_eq!(flag[6], 1.0, "the boundary inserted sample must be flagged");
        assert_eq!(flag[9], 0.0, "an unfilled sample must not be flagged as invented");
        assert_eq!(flag[0], 0.0, "the open top must not be flagged as invented");
        assert_eq!(flag[1], 0.0, "the open top must not be flagged as invented");
        assert_eq!(flag[12], 0.0, "the open bottom must not be flagged as invented");
        assert_eq!(
            flag.iter().filter(|sample| **sample == 1.0).count(),
            2,
            "only the two inserted samples are flagged"
        );
        for &i in &[2usize, 4, 5, 7, 8, 10, 11] {
            assert_eq!(got[i], curve[i], "measured sample {i} must remain unchanged");
            assert_eq!(flag[i], 0.0, "measured sample {i} must not be flagged as invented");
        }
    }

    /// HOLD carries the last live value rather than ramping — the honest fill for a blocky curve,
    /// where a straight line would draw a transition the rock does not have.
    #[test]
    fn holding_a_gap_draws_no_transition_the_rock_does_not_have() {
        let n = 20;
        let depth = regular(n, 0.1, 1000.0);
        let mut curve = vec![1.0f32; n];
        for c in curve.iter_mut().take(14).skip(10) {
            *c = f32::NAN;
        }
        for c in curve.iter_mut().skip(14) {
            *c = 4.0;
        }
        let ctx = ctx_for(&depth, &curve, &[("MAX_GAP", 1.0)], &[("OPT_METHOD", "HOLD")]);
        let out = fill_gaps(&ctx).expect("run");
        for i in 10..14 {
            assert_eq!(out["OUT_CURVE"][i], 1.0, "HOLD carries the value above, sample {i}");
        }
    }

    /// Clip: BLANK removes a reading the tool could not make; CLAMP pulls it to the bound; an
    /// empty side is genuinely not a bound.
    #[test]
    fn clipping_can_blank_or_clamp_and_an_empty_side_is_not_a_bound() {
        let depth = regular(5, 0.1, 1000.0);
        let curve = vec![-5.0f32, 10.0, 50.0, 400.0, 20.0];

        let blanked = clip(&ctx_for(&depth, &curve, &[("MIN", 0.0), ("MAX", 200.0)], &[]))
            .expect("run");
        assert!(blanked["OUT_CURVE"][0].is_nan(), "below MIN becomes MISSING");
        assert!(blanked["OUT_CURVE"][3].is_nan(), "above MAX becomes MISSING");
        assert_eq!(blanked["OUT_CURVE"][2], 50.0, "an in-range sample is untouched");
        assert_eq!(blanked["OUT_FLAG"][3], 1.0, "and the out-of-range sample is flagged");

        let clamped = clip(&ctx_for(
            &depth,
            &curve,
            &[("MIN", 0.0), ("MAX", 200.0)],
            &[("OPT_ACTION", "CLAMP")],
        ))
        .expect("run");
        assert_eq!(clamped["OUT_CURVE"][0], 0.0);
        assert_eq!(clamped["OUT_CURVE"][3], 200.0);

        // Lower bound only: the high reading is left alone, because no upper bound was declared.
        let one_sided = clip(&ctx_for(&depth, &curve, &[("MIN", 0.0)], &[])).expect("run");
        assert!(one_sided["OUT_CURVE"][0].is_nan(), "the declared bound still applies");
        assert_eq!(one_sided["OUT_CURVE"][3], 400.0, "the undeclared side is not a bound");

        // Neither bound is refused rather than silently copying the curve.
        assert!(clip(&ctx_for(&depth, &curve, &[], &[])).is_err());
        // A reversed pair is refused rather than swapped.
        assert!(clip(&ctx_for(&depth, &curve, &[("MIN", 200.0), ("MAX", 0.0)], &[])).is_err());
    }

    /// **The percentile map lands the well's own P_LOW and P_HIGH exactly on the reference pair.**
    /// That is the entire claim of a two-point normalization, and the thing that makes two wells
    /// comparable afterwards.
    ///
    /// The near miss worth recording: `distribution::percentile` takes an ALREADY-SORTED slice.
    /// Handing it the samples in depth order returns whatever value sits 3% of the way down the
    /// WELL — a number about the drilling order rather than about the distribution — and every
    /// curve built on it looks entirely reasonable. Caught by
    /// `workflow::tests::gr_normalization_anchors_each_well_on_its_own_percentiles`, not by
    /// reading the code.
    #[test]
    fn a_two_point_map_lands_the_wells_own_percentiles_on_the_reference_pair() {
        let n = 200;
        let depth = regular(n, 0.1, 1000.0);
        // Deliberately NOT monotone with depth, so a resampler that forgot to sort is caught.
        let curve: Vec<f32> = (0..n).map(|i| 15.0 + 60.0 * (((i * 37) % n) as f32 / (n - 1) as f32)).collect();
        let out = normalize(&ctx_for(
            &depth,
            &curve,
            &[("P_LOW", 3.0), ("P_HIGH", 97.0), ("REF_LOW", 20.0), ("REF_HIGH", 120.0)],
            &[("OPT_METHOD", "TWO_POINT"), ("OPT_SPACE", "LINEAR")],
        ))
        .expect("run");
        let mut v = out["OUT_CURVE"].clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!((crate::distribution::percentile(&v, 3.0) - 20.0).abs() < 0.5, "P3 -> {:?}", v.first());
        assert!((crate::distribution::percentile(&v, 97.0) - 120.0).abs() < 0.5);
    }

    /// **A LOG normalization works on the logarithm and comes back, and the reference pair is
    /// given in the curve's own units.** Nobody quotes a reference resistivity as 0.3010.
    ///
    /// The distinction is not cosmetic. A resistivity spanning 0.2 to 200 ohm-m covers three
    /// decades; a linear map onto 1..100 puts nearly every sample below 1.5 and stretches the top
    /// decade over the whole range — which is what a linear normalization of a log-scale curve
    /// does, silently. Jauhar, 2026-08-05: *"we should have option with logarithmic data"*.
    #[test]
    fn a_log_normalization_maps_decades_rather_than_differences() {
        let n = 61;
        let depth = regular(n, 0.1, 1000.0);
        // Evenly spaced in log10 from 0.1 to 100 — three decades.
        let curve: Vec<f32> = (0..n).map(|i| 10f32.powf(-1.0 + 3.0 * i as f32 / (n - 1) as f32)).collect();
        // 1..100 rather than 1..1000: with 1..1000 the linear scale factor comes out at exactly
        // 10 and both methods land on the same number by coincidence, so the test would prove
        // nothing while passing.
        let args = &[("P_LOW", 0.0), ("P_HIGH", 100.0), ("REF_LOW", 1.0), ("REF_HIGH", 100.0)];
        let log = normalize(&ctx_for(&depth, &curve, args, &[("OPT_SPACE", "LOG")])).expect("run");
        let lin = normalize(&ctx_for(&depth, &curve, args, &[("OPT_SPACE", "LINEAR")])).expect("run");

        // The ends land on the reference pair either way — that is not what separates them.
        assert!((log["OUT_CURVE"][0] - 1.0).abs() < 0.01, "{}", log["OUT_CURVE"][0]);
        assert!((log["OUT_CURVE"][n - 1] - 100.0).abs() < 0.5);
        // The MIDDLE is. A curve evenly spaced in decades stays evenly spaced: the middle sample
        // sits at the geometric centre of 1 and 100, which is 10.
        let mid_log = log["OUT_CURVE"][n / 2];
        assert!((mid_log - 10.0).abs() < 0.2, "log space keeps the decades even: {mid_log}");
        // Linearly, the same sample is dragged to the bottom of the range and three decades of
        // rock come back indistinguishable.
        let mid_lin = lin["OUT_CURVE"][n / 2];
        assert!(mid_lin < 6.0, "a linear map of a log-scale curve crushes its middle: {mid_lin}");

        // A non-positive sample has no logarithm, so it gets no answer rather than a floored one.
        let mut with_zero = curve.clone();
        with_zero[5] = 0.0;
        let z = normalize(&ctx_for(&depth, &with_zero, args, &[("OPT_SPACE", "LOG")])).expect("run");
        assert!(z["OUT_CURVE"][5].is_nan(), "zero must stay MISSING, not become the low reference");
    }

    /// **The reference pair has NO default and the run refuses without it.** A pair from one basin
    /// is the wrong pair in another, and normalized output looks entirely plausible either way —
    /// so a silent fallback would be the most dangerous default in the module.
    ///
    /// MEAN_SD is the deliberate exception: mean 0, spread 1 is a definition rather than somebody
    /// else's field calibration, so it runs unconfigured.
    #[test]
    fn normalize_refuses_a_reference_pair_it_was_not_given() {
        let depth = regular(50, 0.1, 1000.0);
        let curve: Vec<f32> = (0..50).map(|i| 20.0 + i as f32).collect();
        let err = normalize(&ctx_for(&depth, &curve, &[("P_LOW", 3.0), ("P_HIGH", 97.0)], &[]))
            .expect_err("must refuse without a reference pair");
        assert!(err.contains("REF_LOW"), "the refusal names what is missing: {err}");

        // A pair that maps every sample onto one number is refused rather than written flat.
        assert!(normalize(&ctx_for(
            &depth,
            &curve,
            &[("P_LOW", 3.0), ("P_HIGH", 97.0), ("REF_LOW", 50.0), ("REF_HIGH", 50.0)],
            &[]
        ))
        .is_err());

        // MEAN_SD needs nothing: a z-score is a definition.
        let z = normalize(&ctx_for(&depth, &curve, &[("REF_MEAN", 0.0), ("REF_SD", 1.0)], &[("OPT_METHOD", "MEAN_SD")]))
            .expect("a z-score is generic");
        let mean: f32 = z["OUT_CURVE"].iter().sum::<f32>() / 50.0;
        assert!(mean.abs() < 1e-4, "a z-score is centred: {mean}");
    }

    /// Flipping twice about the same pivot returns the original — the property that makes the
    /// operation reversible, and the one that fails first if the sign is ever fumbled.
    #[test]
    fn flipping_twice_about_the_same_pivot_returns_the_original() {
        let depth = regular(6, 0.1, 1000.0);
        let curve = vec![-80.0f32, -20.0, 0.0, f32::NAN, 35.0, 60.0];
        let once = flip(&ctx_for(&depth, &curve, &[("PIVOT", -10.0)], &[])).expect("run");
        let back = {
            let n = depth.len();
            let mut logs = HashMap::new();
            logs.insert("DEPTH".to_string(), depth.clone());
            logs.insert("CURVE".to_string(), once["OUT_CURVE"].clone());
            let mut ps = HashMap::new();
            ps.insert("PIVOT".to_string(), vec![-10.0; n]);
            let mut os: HashMap<String, String> = HashMap::new();
            os.insert("__IN_CURVE".to_string(), "GR".to_string());
            flip(&ModuleContext { n, logs, params: ps, opts: os, depth_unit: DepthUnit::Metres })
                .expect("run")
        };
        for i in 0..curve.len() {
            if curve[i].is_nan() {
                assert!(back["OUT_CURVE"][i].is_nan(), "MISSING stays MISSING through a flip");
            } else {
                assert!((back["OUT_CURVE"][i] - curve[i]).abs() < 1e-4, "sample {i} did not come back");
            }
        }
        assert_eq!(once["OUT_FLAG"][0], -10.0, "the pivot used is recorded");
        // No pivot and no rule to derive one is refused, never taken as zero.
        assert!(flip(&ctx_for(&depth, &curve, &[], &[])).is_err());
    }
}
