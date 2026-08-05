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
    log_in, log_out_as, opt_labelled, param, param_open, ModuleContext, ModuleOutputs, ModuleSpec,
};
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
/// Median absolute deviation, scaled by 1.4826 so it estimates the standard deviation of a normal
/// population. That scaling is what makes one `K` readable as "this many deviations out" on GR,
/// RHOB, NPHI and RT alike, rather than a different number per curve.
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
fn window_spread(vals: &[f32], idx: &[usize], lo: usize, hi: usize, centre: f32, buf: &mut Vec<f32>) -> f32 {
    buf.clear();
    for &i in &idx[lo..hi] {
        let v = vals[i];
        if v.is_finite() {
            buf.push((v - centre).abs());
        }
    }
    if buf.is_empty() {
        return MISSING;
    }
    buf.sort_by(|a, b| a.partial_cmp(b).expect("finite by construction"));
    let mad = 1.4826 * crate::distribution::percentile(buf, 50.0);
    if mad > 0.0 {
        return mad;
    }
    let mean_dev = buf.iter().map(|v| *v as f64).sum::<f64>() / buf.len() as f64;
    mean_dev as f32
}

/// Fewest samples a HAMPEL window may cover. Below this the spread estimate is dominated by the
/// very sample being judged (see [`window_spread`]), so the test is not measuring anything.
const MIN_HAMPEL_SAMPLES: usize = 5;

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
fn out_args(flag_desc: &str, flag_suffix: &str, flag_pattern: &str) -> Vec<crate::modules::ArgSpec> {
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
        log_out_as("OUT_FLAG", flag_pattern, flag_suffix, ""),
    ]
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
              median (the deviation is 1.4826 x MAD, so one K reads the same on GR, RHOB, NPHI \
              and RT). Needs a WINDOW covering at least five samples, and the run refuses a \
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
                param_open("WINDOW", "Filter window (thickness, centred)", "depth", 0.0, 1000.0, true),
                // K = 3 is the ordinary three-deviation convention (the same generic statistical
                // choice as Tukey's 1.5 x IQR already used in `distribution.rs`), NOT a field
                // calibration — round, and stated as such.
                param("K", "HAMPEL: deviations from the median before a sample is a spike", "", 3.0, 0.5, 20.0),
                param_open("THRESH", "ABS: distance from the median, in the curve's units", "", 0.0, 1e9, false),
                param_open("MAX_RATE", "RATE: largest honest change per depth unit", "", 0.0, 1e9, false),
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
            ));
            a
        },
    }
}

pub fn despike(ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    let vals = ctx.log("CURVE");
    let depth = ctx.log("DEPTH");
    let method = ctx.o("OPT_METHOD").to_string();
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
    if !frame.idx.is_empty() && matches!(method.as_str(), "HAMPEL" | "") {
        let mut widths: Vec<usize> = wins.iter().map(|(lo, hi)| hi - lo).collect();
        widths.sort_unstable();
        let typical = widths[widths.len() / 2];
        if typical < MIN_HAMPEL_SAMPLES {
            let spacing = if frame.len() > 1 {
                (frame.dep[frame.len() - 1] - frame.dep[0]) / (frame.len() - 1) as f64
            } else {
                0.0
            };
            return Err(format!(
                "WINDOW = {window} covers about {typical} samples at this well's {spacing:.3} \
                 sampling, and the HAMPEL test needs at least {MIN_HAMPEL_SAMPLES}: below that \
                 the spread it measures against is set by the very sample being judged. Widen \
                 WINDOW to about {:.3}, or use ABS, which needs no spread estimate.",
                spacing * MIN_HAMPEL_SAMPLES as f64
            ));
        }
    }

    let mut out = vals.clone();
    let mut flag = vec![0.0f32; ctx.n];
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
            flag[i] = 0.0;
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
                let sd = window_spread(&vals, &frame.idx, lo, hi, med, &mut buf);
                // A window that is constant INCLUDING this sample has zero spread and nothing to
                // reject — every sample in it is the median. Guarded rather than left to
                // `0.0 * K = 0.0`, which would flag every sample differing by a rounding step.
                sd.is_finite() && sd > 0.0 && ((v - med) as f64).abs() > k * sd as f64
            }
        };
        if reject {
            out[i] = med;
            flag[i] = 1.0;
        }
        prev = Some((frame.dep[k_i], v));
    }

    let mut res: ModuleOutputs = HashMap::from([("OUT_CURVE".to_string(), out)]);
    if yes(ctx, "OPT_FLAG", true) {
        res.insert("OUT_FLAG".to_string(), flag);
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
                param_open("WINDOW", "Smoothing window (thickness, centred)", "depth", 0.0, 1000.0, true),
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
            ));
            a
        },
    }
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
    let window = constant(ctx, "WINDOW");
    if !(window > 0.0) {
        return Err("WINDOW must be set — a smoothing window is a thickness, and how much \
                    detail it is honest to remove depends on the tool and the beds."
            .into());
    }

    let frame = Frame::new(&depth);
    let wins = frame.windows(window / 2.0);
    let mut out = vals.clone();
    let mut flag = vec![0.0f32; ctx.n];
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
                flag[i] = 1.0;
            }
            out[i] = sm;
        }
    }

    let mut res: ModuleOutputs = HashMap::from([("OUT_CURVE".to_string(), out)]);
    if yes(ctx, "OPT_FLAG", true) {
        res.insert("OUT_FLAG".to_string(), flag);
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
            ));
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
    let mut flag = vec![0.0f32; ctx.n];
    for i in 0..ctx.n {
        let v = vals[i];
        if !v.is_finite() {
            continue;
        }
        let below = lo_b.is_finite() && (v as f64) < lo_b;
        let above = hi_b.is_finite() && (v as f64) > hi_b;
        if below || above {
            flag[i] = 1.0;
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
        res.insert("OUT_FLAG".to_string(), flag);
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
                param_open("MAX_GAP", "Widest hole that may be filled (thickness)", "depth", 0.0, 10000.0, true),
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
            ));
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
    let mut flag = vec![0.0f32; ctx.n];

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
            flag[i] = 1.0;
        }
    }

    let mut res: ModuleOutputs = HashMap::from([("OUT_CURVE".to_string(), out)]);
    if yes(ctx, "OPT_FLAG", true) {
        res.insert("OUT_FLAG".to_string(), flag);
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
