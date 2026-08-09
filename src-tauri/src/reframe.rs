//! **Reframe** — resampling a log set onto a different depth sampling, as a new set.
//!
//! Jauhar, 2026-08-05: *"log cons/set should be have independent sampling compared to other log
//! cons. i.e. i have cons in well A 'wire_input' with 0.1523 sampling, meanwhile other wells 0.5,
//! user should have to resample well A cons to new cons with 0.5 sampling"*.
//!
//! ## Why this could not be a module, and what it fixes
//!
//! A module's outputs are written at the RUN's own depth frame, so an operation that changes how
//! often a well is sampled cannot be one — it has to write a different depth column. That was
//! recorded in [`crate::frame`] as a deferral; this is the tool it was deferred to.
//!
//! It also closes a quieter hole. **Every curve read in this app is an exact depth match onto the
//! well's standard grid** — `fetch_curve_frame` reads `standard_curves ORDER BY depth`, and both
//! the generic store and the log-set archive are then looked up depth by depth with
//! `by_depth.get(&d.to_bits())`. A 0.1523 m delivery attached to a well whose standard grid came
//! from a 0.5 m LAS therefore contributes almost nothing: not an error, not a warning, just a
//! curve that reads mostly MISSING. Re-framing it onto the grid in use is the fix, and re-framing
//! the grid itself onto a finer one is the other direction of the same fix.
//!
//! ## Five rules
//!
//! **A re-framed set carries its OWN depths, and says so.** `log_sets.frame = 'OWN'` is what makes
//! the read path use the set's depth column instead of the well's standard grid. Explicit rather
//! than inferred from the depths themselves: a set that happens to fall on the standard grid and a
//! set deliberately re-framed onto it are different claims, and guessing between them would make
//! the behaviour of every existing project depend on a coincidence.
//!
//! **An own-frame set is written to the ARCHIVE only, never to the current store.**
//! `write_computed_curves_versioned` DELETEs a curve's current rows before appending, so writing
//! a 0.5 m copy of `PHIE` into `computed_curves` would blank the readable 0.1523 m `PHIE` and
//! replace it with rows that align with nothing — the interpretation silently emptied by a
//! resample. The current store is the well's own frame by definition, so a set on a different
//! frame has no business in it.
//!
//! **Downsampling AVERAGES, upsampling INTERPOLATES, and neither is a default that fits every
//! curve.** A box average over the output interval is right for a porosity and wrong for a facies
//! code; nearest-neighbour is right for a facies code and wrong for a porosity. The method is per
//! curve, and [`Method::Auto`] chooses by looking at the values rather than at the name — a curve
//! whose every finite sample is a small non-negative integer is a class curve however it is
//! called.
//!
//! **A permeability is averaged in the geometry the flow has, not arithmetically.** The same rule
//! [`crate::frame`]'s blocking carries, for the same reason: 1000 mD and 0.01 mD average to 500 mD
//! arithmetically and 0.02 mD harmonically, and the arithmetic answer is the one that always reads
//! highest. Offered as GEOMETRIC and HARMONIC beside MEAN, and the doc names the case for each
//! rather than picking one. Jauhar, 2026-08-05: *"we should have option with logarithmic data,
//! even using geometric or harmonic"*.
//!
//! **An output sample with no input inside it is MISSING, never the nearest value.** Holding the
//! last value across a gap draws rock nobody logged — the rule `extract_core_log`'s resampler
//! already follows, and this is the same operation, so it is the same code shape.

use crate::equations;
use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MISSING: f32 = f32::NAN;
const CURVE_SELECTION_DOC_TYPE: &str = "curve_selection";

/// How one curve's samples are carried onto the new frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Method {
    /// Arithmetic mean of every input sample inside the output interval. The honest default for a
    /// volume fraction, a porosity, a saturation — anything that adds up.
    Mean,
    /// Geometric mean — the standard estimate for a permeability through randomly arranged
    /// heterogeneous rock, and the right average for anything read on a log scale (RT, PERM).
    Geometric,
    /// Harmonic mean — permeability through layers in SERIES with the flow, i.e. across bedding.
    Harmonic,
    /// Median: an average that a single spike cannot drag.
    Median,
    /// Linear interpolation at the output depth. For upsampling, where a box average would have
    /// nothing to average.
    Interpolate,
    /// The value of the nearest input sample. The only correct choice for a class or flag curve —
    /// the mean of facies 1 and facies 4 is not facies 2.5.
    Nearest,
    /// Whichever value occurs most often inside the interval. The downsampling counterpart of
    /// NEAREST for a class curve: the bed that dominates the output sample wins it.
    Mode,
    /// Look at the values and choose: MODE for a curve that is all small non-negative integers
    /// (a class or a flag), MEAN otherwise. Downsampling only — upsampling always interpolates a
    /// continuous curve and takes the nearest sample of a discrete one.
    Auto,
}

impl Default for Method {
    fn default() -> Self {
        Method::Auto
    }
}

/// Where the curves being re-framed are read from.
#[derive(Debug, Clone, Deserialize)]
pub struct SourceSpec {
    /// `"logset"` (a versioned output set), `"import"` (a delivery set in the generic store), or
    /// `"standard"` (the well's raw standard curves).
    pub kind: String,
    /// Set name for `logset` / `import`. Ignored for `standard`.
    #[serde(default)]
    pub name: Option<String>,
}

/// The frame to land on.
#[derive(Debug, Clone, Deserialize)]
pub struct TargetSpec {
    /// `"step"` (a uniform sampling), `"regularize"` (the source's OWN spacing, made uniform),
    /// `"match_well"` (another well's standard grid) or `"match_set"` (another set's frame, in the
    /// same well).
    pub kind: String,
    /// Uniform sampling in the project's depth unit. Optional for `regularize`, which falls back to
    /// the source's median spacing.
    #[serde(default)]
    pub step: Option<f64>,
    /// Put every well of the run on ONE frame — the same top, base and step — instead of anchoring
    /// each on its own source top.
    ///
    /// Without this, a `step` target gives each well a grid anchored on that well's own first
    /// depth, so ten wells re-framed at 0.5 come out sharing a STEP but not a single DEPTH
    /// (1500.00, 1500.50 … against 1498.25, 1498.75 …). Every read in this app is an exact depth
    /// match, so nothing downstream can line those wells up — which is the failure Reframe exists
    /// to fix, reappearing one level up. `match_well`/`match_set` never had the problem because a
    /// borrowed frame is taken whole; this gives the same guarantee to a frame we compute.
    ///
    /// Depths a given well has no data for come back MISSING, deliberately — the same answer, and
    /// for the same reason, as a borrowed frame that overhangs the source.
    #[serde(default)]
    pub align: bool,
    /// Well whose frame to copy, for `match_well`.
    #[serde(default)]
    pub well_id: Option<String>,
    /// Set whose frame to copy, for `match_set`.
    #[serde(default)]
    pub set_name: Option<String>,
    /// Interval to cover. Absent means the source's own extent.
    #[serde(default)]
    pub top: Option<f64>,
    #[serde(default)]
    pub base: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReframeRequest {
    pub well_ids: Vec<String>,
    pub source: SourceSpec,
    /// The named, saved, inspectable selection whose members are carried. There is deliberately no
    /// empty/all fallback: that would make the chosen curves a hidden default again.
    pub selection_name: String,
    /// An unavailable requested mnemonic may be replaced only by a named source curve the user
    /// explicitly accepted. The substitute keeps its OWN mnemonic in the output; recording the
    /// mapping as run provenance must never turn into supplying its data under `requested`.
    #[serde(default)]
    pub substitutions: Vec<CurveSubstitution>,
    pub target: TargetSpec,
    /// Per-curve method overrides, keyed by upper-case mnemonic.
    #[serde(default)]
    pub methods: HashMap<String, Method>,
    #[serde(default)]
    pub default_method: Method,
    /// The set to write. Reused as-is if it already exists — a new VERSION of the same name, which
    /// is how every other set in this app behaves (Jauhar, 2026-08-05: *"when its already there,
    /// it can replace with version number for logs, but for cons still same"*).
    pub output_set: String,
    /// Probe only: compute and report, write nothing.
    #[serde(default)]
    pub preview: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurveSubstitution {
    pub requested: String,
    pub substitute: String,
    /// No serde default is needed for safety: a missing field fails deserialization instead of
    /// turning an absent decision into consent.
    pub accepted: bool,
}

/// The only selection mode currently supported. It is stored anyway: a member list without a mode
/// is ambiguous about whether listed curves are included or excluded, which is D-19's failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurveSelectionMode {
    Selected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurveSelection {
    pub name: String,
    pub mode: CurveSelectionMode,
    /// Ordered exact mnemonics. Order survives save/reload because ordinal addressing is part of
    /// the selection's inspectable identity, not an implementation detail to sort away.
    pub members: Vec<String>,
}

fn normalized_selection(selection: &CurveSelection) -> Result<CurveSelection, String> {
    let name = selection.name.trim();
    if name.is_empty() {
        return Err("name the curve selection".into());
    }
    let mut members = Vec::new();
    for raw in &selection.members {
        let member = raw.trim().to_uppercase();
        if member.is_empty() {
            return Err(format!("curve selection '{name}' contains an empty member"));
        }
        if !members.contains(&member) {
            members.push(member);
        }
    }
    if members.is_empty() {
        return Err(format!(
            "curve selection '{name}' lists no members; there is no hidden all-curves fallback"
        ));
    }
    Ok(CurveSelection { name: name.to_string(), mode: selection.mode, members })
}

pub fn save_curve_selection(conn: &Connection, selection: &CurveSelection) -> Result<CurveSelection, String> {
    let selection = normalized_selection(selection)?;
    let json = serde_json::to_string(&selection).map_err(|e| e.to_string())?;
    crate::db::save_document(conn, CURVE_SELECTION_DOC_TYPE, &selection.name, &json).map_err(|e| e.to_string())?;
    Ok(selection)
}

pub fn list_curve_selections(conn: &Connection) -> Result<Vec<CurveSelection>, String> {
    let docs = crate::db::list_documents(conn, CURVE_SELECTION_DOC_TYPE).map_err(|e| e.to_string())?;
    let mut selections = Vec::with_capacity(docs.len());
    for doc in docs {
        let selection: CurveSelection = serde_json::from_str(&doc.json)
            .map_err(|e| format!("saved curve selection '{}' is unreadable: {e}", doc.name))?;
        let selection = normalized_selection(&selection)?;
        if selection.name != doc.name {
            return Err(format!(
                "saved curve selection key '{}' disagrees with its inspectable name '{}'",
                doc.name, selection.name
            ));
        }
        selections.push(selection);
    }
    Ok(selections)
}

pub fn delete_curve_selection(conn: &Connection, name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("name the curve selection to delete".into());
    }
    crate::db::delete_document(conn, CURVE_SELECTION_DOC_TYPE, name).map_err(|e| e.to_string())
}

fn load_curve_selection(conn: &Connection, name: &str) -> Result<CurveSelection, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("pick a saved curve selection; no hidden all-curves selection is applied".into());
    }
    list_curve_selections(conn)?
        .into_iter()
        .find(|selection| selection.name == name)
        .ok_or_else(|| format!("saved curve selection '{name}' does not exist"))
}

#[derive(Debug, Clone, Serialize)]
pub struct ReframeCurve {
    pub name: String,
    pub method: String,
    /// Input samples that carried a value.
    pub samples_in: usize,
    /// Output samples that got one.
    pub samples_out: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReframeResult {
    pub well_id: String,
    pub well_name: String,
    /// Median spacing of the source, in the project depth unit — the number the user is deciding
    /// against, and the reason a probe exists at all.
    pub source_step: f64,
    pub target_step: f64,
    pub depth_top: f64,
    pub depth_base: f64,
    pub rows: usize,
    pub curves: Vec<ReframeCurve>,
    pub version: Option<i64>,
    pub notes: Vec<String>,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// The resampler
// ---------------------------------------------------------------------------

/// Median spacing between consecutive finite depths — what "this set is sampled at 0.1523" means.
///
/// The MEDIAN rather than the mean, because a log with one gap in it has a mean spacing that is
/// nobody's sampling: a 0.1524 m curve missing 30 m in the middle averages out at something like
/// 0.18, and a user reading that would resample to a frame the tool invented.
pub fn median_step(depth: &[f32]) -> f64 {
    let mut gaps: Vec<f32> = depth
        .windows(2)
        .filter_map(|w| {
            let d = w[1] - w[0];
            (w[0].is_finite() && w[1].is_finite() && d > 0.0).then_some(d)
        })
        .collect();
    if gaps.is_empty() {
        return f64::NAN;
    }
    crate::distribution::percentile(&mut gaps, 50.0) as f64
}

/// The largest value a class code is allowed to take. A facies scheme, a rock type, a lithology
/// code and a flag are all small; a curve reaching past this is a measurement.
const MAX_CLASS_CODE: f32 = 64.0;

/// Below this, small whole numbers are taken as a code scheme whatever gaps they leave — a
/// lithology numbered 1, 5, 9 is still a lithology. Above it the codes must also be DENSE.
const OBVIOUS_CLASS_CODE: f32 = 20.0;

/// True when the values are a set of CODES — a facies scheme, a rock type, a 0/1 flag — rather
/// than a measurement.
///
/// Deliberately a property of the VALUES, not of the name: a curve called FACIES may hold anything
/// and a class curve may be called anything, and averaging a class curve is the error this exists
/// to prevent.
///
/// **Whole numbers alone are not enough, and the test that proved it is worth keeping in mind.** A
/// gamma ray alternating 40 and 80 API is two non-negative integers with only two distinct values
/// — a "small integers" rule calls it a class curve, picks the commonest value, and returns 80
/// where the rock averages 60. Silently, on a curve that looks entirely normal.
///
/// So there are two signals. Below [`OBVIOUS_CLASS_CODE`] small whole numbers are a code scheme
/// whatever gaps they leave — a lithology numbered 1, 5, 9 is still a lithology, and no
/// measurement in this app is a whole number under 20 throughout. Above it the codes must also be
/// DENSE in their own range: a scheme numbered 0..40 uses most of the values available to it,
/// while {40, 80} uses two of eighty-one.
///
/// **It stays a guess, and that is why the resolved method is REPORTED per curve.** A caliper that
/// happens to be whole inches throughout would be read as a code scheme here; the run says
/// "CALI → MODE" and the user sets it to MEAN. A guess nobody can see is the thing to avoid, not a
/// guess.
pub fn looks_discrete(vals: &[f32]) -> bool {
    let mut seen = 0usize;
    let mut distinct: Vec<f32> = Vec::new();
    let mut max = 0.0f32;
    for v in vals.iter().filter(|v| v.is_finite()) {
        seen += 1;
        if *v < 0.0 || *v > MAX_CLASS_CODE || (v.round() - v).abs() > 1e-6 {
            return false;
        }
        max = max.max(*v);
        if !distinct.iter().any(|d| (*d - *v).abs() < 1e-6) {
            distinct.push(*v);
        }
    }
    seen > 0 && (max <= OBVIOUS_CLASS_CODE || distinct.len() as f32 >= (max + 1.0) * 0.5)
}

/// The method a curve of CLASS IDENTIFIERS must actually be resampled by, whatever was asked for.
///
/// `SB-MLA-055`. A class code is a name that happens to be written as a number. Averaging facies 1
/// and facies 4 gives 2.5; interpolating between them gives every value in between; the geometric
/// mean gives 2. None of those is a facies. The result still computes, still plots as a block
/// track, still exports to LAS and still reads back into the next module — which is why this is
/// enforced rather than left to a warning.
///
/// MEDIAN is in the unsafe list and that is not an oversight: `combine` takes it through
/// `distribution::percentile`, which is R type 7 and interpolates between the two middle samples,
/// so an even-count interval of {1, 2} returns 1.5. A median that lands on a real sample is safe;
/// this one does not always.
///
/// The two safe destinations are the two the doc on `Method` already names. A box average becomes
/// MODE — the bed that dominates the output sample wins it, which is the honest upscale of a class
/// curve. An interpolation becomes NEAREST. AUTO is already safe: it resolves to MODE for anything
/// `looks_discrete` accepts, and a declared class curve it did not accept still lands on MODE here.
///
/// **Only a DECLARATION may reach this function, never `looks_discrete`.** The heuristic reads the
/// values, and values lie: a caliper logged in whole inches looks exactly like a code scheme, and
/// the doc above promises that user can set it to MEAN. So a guess picks the default and a
/// declaration overrides the choice — those are different powers and conflating them would make
/// the caliper case unfixable.
pub fn class_safe_method(m: Method) -> Method {
    match m {
        Method::Interpolate => Method::Nearest,
        Method::Mean | Method::Geometric | Method::Harmonic | Method::Median | Method::Auto => Method::Mode,
        safe @ (Method::Nearest | Method::Mode) => safe,
    }
}

/// Resamples `vals`, sampled at ascending `src_depth`, onto `out_depth`.
///
/// The ONE resampler: the tool and the read path both call it, so what a user sees after
/// re-framing a set and what a module reads when it runs against that set cannot disagree.
///
/// Deliberately a pure numeric kernel — it takes the method it is given and does not consult the
/// class registry. Policy lives with the layer that can REPORT it (`class_safe_method` at the run
/// path, which writes the resolved method into `ReframeCurve.method`); a kernel that silently
/// substituted a method would be a coercion nobody could see.
///
/// Each output sample owns the half-open interval reaching HALFWAY to its neighbours, and takes
/// every input sample inside it. That is a box average, not an interpolation between the two
/// nearest samples: on a downsample those two are a small sample of what the interval holds, so
/// picking them is aliasing — a lamination every few centimetres would beat against the output
/// sampling and come back as a trend that is not in the rock. On an UPSAMPLE the box is usually
/// empty, which is why `Interpolate`/`Nearest` exist and why `Auto` is a downsampling rule.
///
/// An output sample whose interval holds nothing is MISSING. Nothing is held across a gap.
pub fn resample_onto(src_depth: &[f32], vals: &[f32], out_depth: &[f32], method: Method) -> Vec<f32> {
    let n = out_depth.len();
    let mut out = vec![MISSING; n];
    if src_depth.is_empty() {
        return out;
    }
    let method = match method {
        Method::Auto => {
            if looks_discrete(vals) {
                Method::Mode
            } else {
                Method::Mean
            }
        }
        m => m,
    };
    if matches!(method, Method::Interpolate | Method::Nearest) {
        for (i, d) in out_depth.iter().enumerate() {
            if !d.is_finite() {
                continue;
            }
            out[i] = match method {
                Method::Interpolate => interp_at(src_depth, vals, *d as f64),
                _ => nearest_at(src_depth, vals, *d as f64),
            };
        }
        return out;
    }

    // Box bounds: halfway to each neighbour, the ends extended by half a step so the first and
    // last output samples own as much rock as the ones between them.
    let mut buf: Vec<f32> = Vec::new();
    let mut cursor = 0usize;
    for i in 0..n {
        let d = out_depth[i] as f64;
        if !d.is_finite() {
            continue;
        }
        let prev = (0..i).rev().find(|k| out_depth[*k].is_finite()).map(|k| out_depth[k] as f64);
        let next = (i + 1..n).find(|k| out_depth[*k].is_finite()).map(|k| out_depth[k] as f64);
        // A frame of ONE sample owns the whole source — that is what "average this onto one
        // depth" means, and the alternative (a box of zero width, so nothing inside it) would
        // return MISSING without saying why.
        let (lo, hi) = match (prev, next) {
            (None, None) => (f64::NEG_INFINITY, f64::INFINITY),
            _ => {
                let half_up = prev.map(|p| (d - p) / 2.0).or_else(|| next.map(|nx| (nx - d) / 2.0)).unwrap_or(0.0);
                let half_dn = next.map(|nx| (nx - d) / 2.0).or_else(|| prev.map(|p| (d - p) / 2.0)).unwrap_or(0.0);
                (d - half_up, d + half_dn)
            }
        };

        while cursor > 0 && src_depth[cursor - 1].is_finite() && (src_depth[cursor - 1] as f64) >= lo {
            cursor -= 1;
        }
        while cursor < src_depth.len() && (!src_depth[cursor].is_finite() || (src_depth[cursor] as f64) < lo) {
            cursor += 1;
        }
        buf.clear();
        let mut k = cursor;
        while k < src_depth.len() {
            let sd = src_depth[k] as f64;
            // HALF-OPEN [lo, hi): a source sample landing exactly on a box boundary belongs to
            // the box BELOW it and to nothing else. Closed at both ends — which is what this was
            // first — counts that sample twice, once in each neighbour, so a resampled curve no
            // longer averages to what the source held and the error is largest exactly where the
            // sampling divides evenly, which is the ordinary case. The last box closes at its top
            // so the deepest sample is not dropped.
            let last_box = next.is_none();
            if sd.is_finite() && (sd > hi || (sd >= hi && !last_box)) {
                break;
            }
            if sd.is_finite() && vals[k].is_finite() {
                buf.push(vals[k]);
            }
            k += 1;
        }
        if buf.is_empty() {
            continue;
        }
        out[i] = combine(&mut buf, method);
    }
    out
}

/// The one place a set of samples becomes one sample.
fn combine(buf: &mut Vec<f32>, method: Method) -> f32 {
    match method {
        Method::Median => crate::distribution::percentile(buf, 50.0),
        Method::Mode => {
            let mut best = (buf[0], 0usize);
            for v in buf.iter() {
                let c = buf.iter().filter(|x| (**x - *v).abs() < 1e-6).count();
                if c > best.1 {
                    best = (*v, c);
                }
            }
            best.0
        }
        // A non-positive sample has no logarithm and no reciprocal. It is DROPPED and the rest of
        // the interval still answers, rather than the whole output sample collapsing to zero or
        // MISSING — the rule `frame::block` already holds for the same two means.
        Method::Geometric => {
            let live: Vec<f64> = buf.iter().map(|v| *v as f64).filter(|v| *v > 0.0).collect();
            if live.is_empty() {
                return MISSING;
            }
            (live.iter().map(|v| v.ln()).sum::<f64>() / live.len() as f64).exp() as f32
        }
        Method::Harmonic => {
            let live: Vec<f64> = buf.iter().map(|v| *v as f64).filter(|v| *v > 0.0).collect();
            if live.is_empty() {
                return MISSING;
            }
            (live.len() as f64 / live.iter().map(|v| 1.0 / v).sum::<f64>()) as f32
        }
        _ => (buf.iter().map(|v| *v as f64).sum::<f64>() / buf.len() as f64) as f32,
    }
}

/// Linear interpolation at `target`. MISSING outside the range and across a gap — the same rule
/// `modules::interp_at` follows, so an interpolated sample never spans rock nobody logged.
fn interp_at(depth: &[f32], vals: &[f32], target: f64) -> f32 {
    let live: Vec<usize> = (0..depth.len()).filter(|i| depth[*i].is_finite() && vals[*i].is_finite()).collect();
    if live.is_empty() {
        return MISSING;
    }
    let (first, last) = (depth[live[0]] as f64, depth[*live.last().unwrap()] as f64);
    if target < first || target > last {
        return MISSING;
    }
    let pos = live.partition_point(|i| (depth[*i] as f64) < target);
    if pos < live.len() && (depth[live[pos]] as f64 - target).abs() < 1e-9 {
        return vals[live[pos]];
    }
    if pos == 0 {
        return vals[live[0]];
    }
    let (a, b) = (live[pos - 1], live[pos.min(live.len() - 1)]);
    let (d0, d1) = (depth[a] as f64, depth[b] as f64);
    if (d1 - d0).abs() < 1e-12 {
        return vals[a];
    }
    let t = (target - d0) / (d1 - d0);
    (vals[a] as f64 + t * (vals[b] as f64 - vals[a] as f64)) as f32
}

/// The value of the nearest live sample, or MISSING outside the logged interval.
fn nearest_at(depth: &[f32], vals: &[f32], target: f64) -> f32 {
    let mut best: Option<(f64, f32)> = None;
    for i in 0..depth.len() {
        if !depth[i].is_finite() || !vals[i].is_finite() {
            continue;
        }
        let dist = (depth[i] as f64 - target).abs();
        if best.is_none_or(|(b, _)| dist < b) {
            best = Some((dist, vals[i]));
        }
    }
    match best {
        // Outside the logged interval there is no nearest sample worth the name — holding the end
        // value out into undrilled rock is the one thing this family never does.
        Some((dist, v)) => {
            let live: Vec<f64> =
                (0..depth.len()).filter(|i| depth[*i].is_finite() && vals[*i].is_finite()).map(|i| depth[i] as f64).collect();
            let (first, last) = (live[0], *live.last().unwrap());
            if target < first - dist.max(0.0) || target > last + dist.max(0.0) {
                MISSING
            } else if target < first || target > last {
                MISSING
            } else {
                v
            }
        }
        None => MISSING,
    }
}

/// Builds the output depth column.
/// `shared` is the run-wide interval computed by [`shared_extent`] when the target asks to align.
/// It overrides the source's own extent so every well of the run lands on identical depths.
fn build_frame(
    target: &TargetSpec,
    src_depth: &[f32],
    conn: &Connection,
    well_id: &str,
    shared: Option<(f64, f64)>,
) -> Result<Vec<f32>, String> {
    let live: Vec<f64> = src_depth.iter().filter(|d| d.is_finite()).map(|d| *d as f64).collect();
    if live.is_empty() {
        return Err("the source has no depths to re-frame".into());
    }
    let src_top = *live.first().unwrap();
    let src_base = *live.last().unwrap();

    match target.kind.as_str() {
        "match_well" | "match_set" => {
            let borrowed: Vec<f32> = if target.kind == "match_well" {
                let other = target.well_id.as_deref().ok_or("pick the well whose sampling to match")?;
                let mut stmt = conn
                    .prepare("SELECT depth FROM standard_curves WHERE well_id = ?1 ORDER BY depth")
                    .map_err(|e| e.to_string())?;
                let rows = stmt.query_map(params![other], |r| r.get::<_, f32>(0)).map_err(|e| e.to_string())?;
                rows.collect::<duckdb::Result<_>>().map_err(|e| e.to_string())?
            } else {
                let set = target.set_name.as_deref().ok_or("pick the set whose sampling to match")?;
                set_frame(conn, well_id, set).map_err(|e| e.to_string())?.ok_or_else(|| {
                    format!("set '{set}' has no frame of its own on this well")
                })?
            };
            if borrowed.is_empty() {
                return Err("the frame being matched has no depths".into());
            }
            // Matched frames are taken WHOLE rather than clipped to the source: the point of
            // matching is that two wells come out on the same rows, and clipping each to its own
            // logged interval would put them back on different ones. Depths with nothing to read
            // simply come back MISSING, which is the honest answer.
            let (top, base) = (target.top.unwrap_or(f64::NEG_INFINITY), target.base.unwrap_or(f64::INFINITY));
            Ok(borrowed.into_iter().filter(|d| (*d as f64) >= top && (*d as f64) <= base).collect())
        }
        _ => {
            // REGULARIZE is the same frame builder with the step supplied by the source instead of
            // by the user: the point is to make an irregular sampling uniform WITHOUT changing how
            // finely it was logged, so re-typing the number off the probe would only be a chance to
            // get it wrong. An explicit step still wins — regularize-and-coarsen is one operation.
            let step = match target.step.filter(|s| *s > 0.0) {
                Some(s) => s,
                None if target.kind == "regularize" => {
                    let s = median_step(src_depth);
                    if !(s > 0.0) {
                        return Err("this source has no usable spacing to regularize onto — its \
                                    depths do not advance"
                            .into());
                    }
                    s
                }
                None => {
                    return Err("set the sampling to re-frame onto — there is no generic value for \
                                it, and a wrong one is invisible once the curve is written"
                        .into())
                }
            };
            let (top, base) = match shared {
                Some(iv) => iv,
                None => (target.top.unwrap_or(src_top), target.base.unwrap_or(src_base)),
            };
            if !(base > top) {
                return Err("the interval's base must be below its top".into());
            }
            let n = ((base - top) / step).floor() as usize + 1;
            if n > 2_000_000 {
                return Err(format!(
                    "a {step} sampling over {top:.1}–{base:.1} would be {n} samples. Check the \
                     sampling — this looks like a unit mix-up."
                ));
            }
            // Anchored on the interval's own top rather than on zero, so the frame does not depend
            // on where the depth datum happens to be.
            Ok((0..n).map(|i| (top + i as f64 * step) as f32).collect())
        }
    }
}

/// The depth column of a set that carries its own frame, or `None` for a set on the well's grid.
pub(crate) fn set_frame(conn: &Connection, well_id: &str, set_name: &str) -> duckdb::Result<Option<Vec<f32>>> {
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT set_id, COALESCE(frame, 'STANDARD') FROM log_sets
             WHERE well_id = ?1 AND upper(set_name) = upper(?2) ORDER BY version DESC LIMIT 1",
            params![well_id, set_name],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();
    let Some((set_id, frame)) = row else { return Ok(None) };
    if frame != "OWN" {
        return Ok(None);
    }
    let mut stmt = conn.prepare(
        "SELECT DISTINCT depth FROM computed_curves_archive WHERE set_id = ?1 ORDER BY depth",
    )?;
    let rows = stmt.query_map(params![set_id], |r| r.get::<_, f32>(0))?;
    let depths: Vec<f32> = rows.collect::<duckdb::Result<_>>()?;
    Ok((!depths.is_empty()).then_some(depths))
}

// ---------------------------------------------------------------------------
// Reading the source
// ---------------------------------------------------------------------------

/// The curves a source holds, at the source's OWN depths — which is the whole point, since the
/// aligned readers everywhere else would have already dropped whatever does not fall on the
/// standard grid.
fn read_source(
    conn: &Connection,
    well_id: &str,
    source: &SourceSpec,
    wanted: &[String],
) -> Result<(Vec<f32>, Vec<(String, Vec<f32>)>), String> {
    let mut by_depth: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    let mut depths: Vec<f32> = Vec::new();
    let mut cols: Vec<(String, Vec<f32>)> = Vec::new();

    // (curve name, [(depth, value)]) from whichever store the source names.
    let raw: Vec<(String, Vec<(f32, f32)>)> = match source.kind.as_str() {
        "import" => {
            let set = source.name.as_deref().unwrap_or("RAW");
            let mut stmt = conn
                .prepare(
                    "SELECT curve_id, upper(mnemonic) FROM curve_meta
                     WHERE well_id = ?1 AND upper(set_name) = upper(?2)",
                )
                .map_err(|e| e.to_string())?;
            let metas: Vec<(String, String)> = stmt
                .query_map(params![well_id, set], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(|e| e.to_string())?
                .collect::<duckdb::Result<_>>()
                .map_err(|e| e.to_string())?;
            let mut out = Vec::new();
            let mut sstmt = conn
                .prepare("SELECT depth, value FROM curve_samples WHERE curve_id = ?1 ORDER BY depth")
                .map_err(|e| e.to_string())?;
            for (id, mnemonic) in metas {
                if !wanted.is_empty() && !wanted.iter().any(|w| w.eq_ignore_ascii_case(&mnemonic)) {
                    continue;
                }
                let rows: Vec<(f32, f32)> = sstmt
                    .query_map(params![id], |r| Ok((r.get(0)?, r.get(1)?)))
                    .map_err(|e| e.to_string())?
                    .collect::<duckdb::Result<_>>()
                    .map_err(|e| e.to_string())?;
                out.push((mnemonic, rows));
            }
            out
        }
        "logset" => {
            let set = source.name.as_deref().ok_or("pick the log set to re-frame")?;
            let set_id: String = conn
                .query_row(
                    "SELECT set_id FROM log_sets WHERE well_id = ?1 AND upper(set_name) = upper(?2)
                     ORDER BY version DESC LIMIT 1",
                    params![well_id, set],
                    |r| r.get(0),
                )
                .map_err(|_| format!("no log set '{set}' on this well"))?;
            let mut stmt = conn
                .prepare(
                    "SELECT upper(curve_name), depth, value FROM computed_curves_archive
                     WHERE set_id = ?1 ORDER BY curve_name, depth",
                )
                .map_err(|e| e.to_string())?;
            let rows: Vec<(String, f32, f32)> = stmt
                .query_map(params![set_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .map_err(|e| e.to_string())?
                .collect::<duckdb::Result<_>>()
                .map_err(|e| e.to_string())?;
            let mut grouped: std::collections::BTreeMap<String, Vec<(f32, f32)>> = Default::default();
            for (name, d, v) in rows {
                if !wanted.is_empty() && !wanted.iter().any(|w| w.eq_ignore_ascii_case(&name)) {
                    continue;
                }
                grouped.entry(name).or_default().push((d, v));
            }
            grouped.into_iter().collect()
        }
        _ => {
            let mut stmt = conn
                .prepare(
                    "SELECT depth, gr, res_deep, nphi, rhob, dt, sp FROM standard_curves
                     WHERE well_id = ?1 ORDER BY depth",
                )
                .map_err(|e| e.to_string())?;
            let rows: Vec<[f32; 7]> = stmt
                .query_map(params![well_id], |r| {
                    Ok([r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?])
                })
                .map_err(|e| e.to_string())?
                .collect::<duckdb::Result<_>>()
                .map_err(|e| e.to_string())?;
            ["GR", "RES_DEEP", "NPHI", "RHOB", "DT", "SP"]
                .iter()
                .enumerate()
                .filter(|(_, n)| wanted.is_empty() || wanted.iter().any(|w| w.eq_ignore_ascii_case(n)))
                .map(|(k, n)| {
                    ((*n).to_string(), rows.iter().map(|r| (r[0], r[k + 1])).collect::<Vec<_>>())
                })
                // A standard column that was never delivered is all-NaN; carrying it would write a
                // curve of nothing and report it as re-framed.
                .filter(|(_, s): &(String, Vec<(f32, f32)>)| s.iter().any(|(_, v)| v.is_finite()))
                .collect()
        }
    };
    if raw.is_empty() {
        return Err("that source holds none of the curves asked for".into());
    }

    for (_, samples) in &raw {
        for (d, _) in samples {
            if d.is_finite() && !by_depth.contains_key(&d.to_bits()) {
                by_depth.insert(d.to_bits(), 0);
            }
        }
    }
    // One depth column for the whole source, ascending — curves of one delivery usually share it,
    // and where they do not the union is the only frame that loses nothing.
    let mut keys: Vec<f32> = by_depth.keys().map(|b| f32::from_bits(*b)).collect();
    keys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    for (i, d) in keys.iter().enumerate() {
        by_depth.insert(d.to_bits(), i);
        depths.push(*d);
    }
    for (name, samples) in raw {
        let mut col = vec![MISSING; depths.len()];
        for (d, v) in samples {
            if let Some(i) = by_depth.get(&d.to_bits()) {
                col[*i] = v;
            }
        }
        cols.push((name, col));
    }
    Ok((depths, cols))
}

/// Names the source can actually supply, without using a family/type classification. This is the
/// precondition for an explicit substitution: it applies only when `requested` is unavailable,
/// and `substitute` must itself be present by exact mnemonic.
pub fn source_curve_names(conn: &Connection, well_id: &str, source: &SourceSpec) -> Result<Vec<String>, String> {
    match source.kind.as_str() {
        "import" => {
            let set = source.name.as_deref().unwrap_or("RAW");
            let mut stmt = conn
                .prepare(
                    "SELECT DISTINCT upper(mnemonic) FROM curve_meta
                     WHERE well_id = ?1 AND upper(set_name) = upper(?2)
                     ORDER BY upper(mnemonic)",
                )
                .map_err(|e| e.to_string())?;
            stmt.query_map(params![well_id, set], |r| r.get::<_, String>(0))
                .map_err(|e| e.to_string())?
                .collect::<duckdb::Result<_>>()
                .map_err(|e| e.to_string())
        }
        "logset" => {
            let set = source.name.as_deref().ok_or("pick the log set to re-frame")?;
            let set_id: String = conn
                .query_row(
                    "SELECT set_id FROM log_sets WHERE well_id = ?1 AND upper(set_name) = upper(?2)
                     ORDER BY version DESC LIMIT 1",
                    params![well_id, set],
                    |r| r.get(0),
                )
                .map_err(|_| format!("no log set '{set}' on this well"))?;
            let mut stmt = conn
                .prepare(
                    "SELECT DISTINCT upper(curve_name) FROM computed_curves_archive
                     WHERE set_id = ?1 ORDER BY upper(curve_name)",
                )
                .map_err(|e| e.to_string())?;
            stmt.query_map(params![set_id], |r| r.get::<_, String>(0))
                .map_err(|e| e.to_string())?
                .collect::<duckdb::Result<_>>()
                .map_err(|e| e.to_string())
        }
        _ => {
            let counts: (i64, i64, i64, i64, i64, i64) = conn
                .query_row(
                    "SELECT
                         COUNT(*) FILTER (WHERE NOT isnan(gr)),
                         COUNT(*) FILTER (WHERE NOT isnan(res_deep)),
                         COUNT(*) FILTER (WHERE NOT isnan(nphi)),
                         COUNT(*) FILTER (WHERE NOT isnan(rhob)),
                         COUNT(*) FILTER (WHERE dt IS NOT NULL AND NOT isnan(dt)),
                         COUNT(*) FILTER (WHERE sp IS NOT NULL AND NOT isnan(sp))
                     FROM standard_curves WHERE well_id = ?1",
                    params![well_id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
                )
                .map_err(|e| e.to_string())?;
            Ok(["GR", "RES_DEEP", "NPHI", "RHOB", "DT", "SP"]
                .into_iter()
                .zip([counts.0, counts.1, counts.2, counts.3, counts.4, counts.5])
                .filter_map(|(name, count)| (count > 0).then_some(name.to_string()))
                .collect())
        }
    }
}

fn resolve_substitutions(
    conn: &Connection,
    well_id: &str,
    source: &SourceSpec,
    requested: &[String],
    substitutions: &[CurveSubstitution],
) -> Result<(Vec<String>, Vec<CurveSubstitution>), String> {
    if substitutions.is_empty() {
        return Ok((requested.to_vec(), Vec::new()));
    }
    if requested.is_empty() {
        return Err("a substitution must name one of the explicitly requested curves".into());
    }
    let available = source_curve_names(conn, well_id, source)?;
    let contains = |name: &str| available.iter().any(|held| held.eq_ignore_ascii_case(name.trim()));
    let mut used = Vec::new();
    let mut effective = Vec::with_capacity(requested.len());

    for decision in substitutions {
        let matching = substitutions
            .iter()
            .filter(|other| other.requested.trim().eq_ignore_ascii_case(decision.requested.trim()))
            .count();
        if matching > 1 {
            return Err(format!(
                "requested curve '{}' has more than one substitution decision",
                decision.requested.trim()
            ));
        }
        if !requested.iter().any(|name| name.trim().eq_ignore_ascii_case(decision.requested.trim())) {
            return Err(format!(
                "substitution for '{}' is not attached to an explicitly requested curve",
                decision.requested.trim()
            ));
        }
    }

    for name in requested {
        let name = name.trim().to_uppercase();
        let decision = substitutions.iter().find(|d| d.requested.trim().eq_ignore_ascii_case(&name));
        let Some(decision) = decision else {
            effective.push(name);
            continue;
        };
        let substitute = decision.substitute.trim().to_uppercase();
        if name == substitute {
            return Err(format!("'{name}' cannot substitute for itself"));
        }
        if contains(&name) {
            return Err(format!(
                "requested curve '{name}' is available; a substitution is only valid for an unavailable curve"
            ));
        }
        if !decision.accepted {
            return Err(format!(
                "substitution {name} -> {substitute} was not explicitly accepted; nothing was written"
            ));
        }
        if !contains(&substitute) {
            return Err(format!("named substitute '{substitute}' is not present in the selected source"));
        }
        effective.push(substitute.clone());
        used.push(CurveSubstitution { requested: name, substitute, accepted: true });
    }
    effective.sort();
    effective.dedup();
    Ok((effective, used))
}

// ---------------------------------------------------------------------------
// The run
// ---------------------------------------------------------------------------

/// Depth extent of a source, without reading a single value.
///
/// The aligned frame has to span every well of the run, which is known only after looking at all of
/// them — and holding a whole field's curve data in memory to learn two numbers per well is not a
/// trade worth making. This asks the database for the extent instead, so the per-well pass below
/// still reads each source exactly once.
///
/// The extent covers the source's WHOLE depth range rather than only the curves asked for. That is
/// deliberate: a superset frame costs some rows that come back MISSING, and MISSING is already the
/// documented answer for a frame that overhangs its source. A frame clipped per curve would put the
/// wells back on different rows, which is the entire thing this is here to prevent.
fn source_extent(conn: &Connection, well_id: &str, source: &SourceSpec) -> Result<(f64, f64), String> {
    let row: Option<(Option<f64>, Option<f64>)> = match source.kind.as_str() {
        "import" => {
            let set = source.name.as_deref().unwrap_or("RAW");
            conn.query_row(
                "SELECT MIN(s.depth), MAX(s.depth) FROM curve_samples s
                 JOIN curve_meta m ON m.curve_id = s.curve_id
                 WHERE m.well_id = ?1 AND upper(m.set_name) = upper(?2)",
                params![well_id, set],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok()
        }
        "logset" => {
            let set = source.name.as_deref().ok_or("pick the log set to re-frame")?;
            let set_id: String = conn
                .query_row(
                    "SELECT set_id FROM log_sets WHERE well_id = ?1 AND upper(set_name) = upper(?2)
                     ORDER BY version DESC LIMIT 1",
                    params![well_id, set],
                    |r| r.get(0),
                )
                .map_err(|_| format!("no log set '{set}' on this well"))?;
            conn.query_row(
                "SELECT MIN(depth), MAX(depth) FROM computed_curves_archive WHERE set_id = ?1",
                params![set_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok()
        }
        _ => conn
            .query_row(
                "SELECT MIN(depth), MAX(depth) FROM standard_curves WHERE well_id = ?1",
                params![well_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok(),
    };
    match row {
        Some((Some(t), Some(b))) if b >= t => Ok((t, b)),
        _ => Err("that source has no depths on this well".into()),
    }
}

/// The interval every well of an aligned run shares: shallowest top to deepest base.
///
/// Wells whose source cannot be read are simply not represented here — they fail on their own in
/// the pass below, with their own message, and letting one unreadable well collapse the frame for
/// the rest would turn a single bad well into a failed batch.
fn shared_extent(conn: &Connection, req: &ReframeRequest) -> Option<(f64, f64)> {
    let mut acc: Option<(f64, f64)> = None;
    for w in &req.well_ids {
        if let Ok((t, b)) = source_extent(conn, w, &req.source) {
            acc = Some(match acc {
                Some((at, ab)) => (at.min(t), ab.max(b)),
                None => (t, b),
            });
        }
    }
    acc
}

pub fn run_reframe(conn: &Connection, req: &ReframeRequest) -> Vec<ReframeResult> {
    let computed = matches!(req.target.kind.as_str(), "step" | "regularize");

    // A borrowed frame (match_well / match_set) is already identical across wells by construction,
    // so aligning one is not an error, just nothing to do.
    if req.target.align && !computed {
        return req.well_ids.iter().map(|w| one_well(conn, w, req, None)).collect();
    }

    // REGULARIZE takes its step from the source, and across wells there is no single source to take
    // it from. Picking one well's spacing for the whole field would be a silent decision about
    // which well is representative, so it is refused by name and the user states the step.
    if req.target.align && req.target.kind == "regularize" && !req.target.step.is_some_and(|s| s > 0.0) {
        return req
            .well_ids
            .iter()
            .map(|w| {
                let mut r = one_well_shell(conn, w);
                r.error = Some(
                    "regularizing several wells onto one frame needs the sampling stated: each \
                     well has its own spacing, and adopting one of them would silently make that \
                     well the standard for the field. Give a step, or regularize one well at a time."
                        .into(),
                );
                r
            })
            .collect();
    }

    let shared = if req.target.align { shared_extent(conn, req) } else { None };
    req.well_ids.iter().map(|w| one_well(conn, w, req, shared)).collect()
}

/// An empty result carrying only the well's identity — the shape every early refusal returns.
fn one_well_shell(conn: &Connection, well_id: &str) -> ReframeResult {
    let well_name: String = conn
        .query_row("SELECT well_name FROM wells WHERE well_id = ?1", params![well_id], |r| r.get(0))
        .unwrap_or_else(|_| well_id.to_string());
    ReframeResult {
        well_id: well_id.to_string(),
        well_name,
        source_step: f64::NAN,
        target_step: f64::NAN,
        depth_top: f64::NAN,
        depth_base: f64::NAN,
        rows: 0,
        curves: vec![],
        version: None,
        notes: vec![],
        error: None,
    }
}

fn one_well(
    conn: &Connection,
    well_id: &str,
    req: &ReframeRequest,
    shared: Option<(f64, f64)>,
) -> ReframeResult {
    let mut res = one_well_shell(conn, well_id);

    let selection = match load_curve_selection(conn, &req.selection_name) {
        Ok(selection) => selection,
        Err(e) => {
            res.error = Some(e);
            return res;
        }
    };

    let (wanted, substitutions) = match resolve_substitutions(
        conn,
        well_id,
        &req.source,
        &selection.members,
        &req.substitutions,
    ) {
        Ok(v) => v,
        Err(e) => {
            res.error = Some(e);
            return res;
        }
    };
    let (src_depth, cols) = match read_source(conn, well_id, &req.source, &wanted) {
        Ok(v) => v,
        Err(e) => {
            res.error = Some(e);
            return res;
        }
    };
    let out_depth = match build_frame(&req.target, &src_depth, conn, well_id, shared) {
        Ok(v) => v,
        Err(e) => {
            res.error = Some(e);
            return res;
        }
    };
    if out_depth.is_empty() {
        res.error = Some("the target frame came out empty over this well's interval".into());
        return res;
    }
    res.source_step = median_step(&src_depth);
    res.target_step = median_step(&out_depth);
    res.depth_top = out_depth.first().copied().unwrap_or(f32::NAN) as f64;
    res.depth_base = out_depth.last().copied().unwrap_or(f32::NAN) as f64;
    res.rows = out_depth.len();

    for decision in &substitutions {
        res.notes.push(format!(
            "Accepted substitution: requested {} was unavailable; used {} under its own name and recorded this decision on the output set.",
            decision.requested, decision.substitute
        ));
    }

    if shared.is_some() {
        // Said per well and not once for the run, because this is the number that explains a well
        // whose output starts above its own first reading — the rows are there so the field lines
        // up, and they are MISSING rather than wrong.
        res.notes.push(format!(
            "Aligned: every well in this run is on one frame, {:.4}–{:.4} at {:.4}. Depths outside \
             this well's own logged interval are MISSING.",
            res.depth_top, res.depth_base, res.target_step
        ));
    }

    // Upsampling a curve by box average would leave most output samples empty, so the method has
    // to change with the direction — and saying so beats returning a curve full of holes.
    let upsampling = res.target_step < res.source_step * 0.999;
    // One query for the whole well, outside the loop — see `db::class_curves_for_well`. A failure
    // to read the registry leaves the set empty, which is the pre-registry behaviour: it must not
    // fail a re-frame, and `looks_discrete` still guards the AUTO path underneath.
    let class_curves = crate::db::class_curves_for_well(conn, well_id).unwrap_or_default();
    let mut written: Vec<(String, Vec<f32>)> = Vec::new();
    let mut coerced: Vec<String> = Vec::new();
    for (name, vals) in cols {
        let asked = req.methods.get(&name).copied();
        let method = match asked.unwrap_or(req.default_method) {
            Method::Auto if upsampling => {
                if looks_discrete(&vals) {
                    Method::Nearest
                } else {
                    Method::Interpolate
                }
            }
            m => m,
        };
        // SB-MLA-055. A DECLARED class curve overrides the choice, including one the user made
        // explicitly — the mean of two facies codes is not a facies, and unlike a bad porosity
        // average nothing downstream can tell that it is wrong. `looks_discrete` above is
        // deliberately not consulted here: it may pick a default, never overrule a decision.
        let method = if class_curves.contains(&name.to_uppercase()) {
            let safe = class_safe_method(method);
            if safe != method {
                coerced.push(format!("{name} ({method:?} → {safe:?})").to_uppercase());
            }
            safe
        } else {
            method
        };
        let out = resample_onto(&src_depth, &vals, &out_depth, method);
        res.curves.push(ReframeCurve {
            name: name.clone(),
            method: format!("{method:?}").to_uppercase(),
            samples_in: vals.iter().filter(|v| v.is_finite()).count(),
            samples_out: out.iter().filter(|v| v.is_finite()).count(),
        });
        written.push((name, out));
    }
    if !coerced.is_empty() {
        // Named, not merely done. The user asked for a method and got another one; a substitution
        // they cannot see is the thing this rule exists to prevent, one level up.
        res.notes.push(format!(
            "Class curves cannot be averaged or interpolated — their values are codes, and the \
             mean of facies 1 and facies 4 is not a facies. Resampled by the nearest safe method \
             instead: {}.",
            coerced.join(", ")
        ));
    }

    if upsampling {
        res.notes.push(format!(
            "Upsampling {:.4} to {:.4}: no new measurement is created, only points between the \
             ones logged. Anything read off the finer curve is a property of the interpolation.",
            res.source_step, res.target_step
        ));
    }
    if let Some(empty) = res.curves.iter().find(|c| c.samples_out == 0 && c.samples_in > 0) {
        res.notes.push(format!(
            "{} had {} samples and none landed on the new frame — check that the interval overlaps.",
            empty.name, empty.samples_in
        ));
    }
    if req.preview {
        return res;
    }

    let spec = equations::LogSetSpec {
        set_name: req.output_set.trim().to_uppercase(),
        module: "reframe".into(),
        params_json: serde_json::to_string(&serde_json::json!({
            "source": req.source.kind,
            "source_set": req.source.name,
            "curve_selection": selection,
            "target_step": res.target_step,
            "source_step": res.source_step,
            "substitutions": substitutions,
        }))
        .unwrap_or_default(),
        inputs_json: serde_json::to_string(&res.curves.iter().map(|c| &c.name).collect::<Vec<_>>())
            .unwrap_or_default(),
    };
    match write_own_frame(conn, well_id, &spec, &out_depth, &written) {
        Ok(version) => res.version = Some(version),
        Err(e) => res.error = Some(e.to_string()),
    }
    res
}

/// Registers the set, marks it as carrying its own frame, and appends the rows to the ARCHIVE
/// only.
///
/// The current store (`computed_curves`) is the well's own frame by definition and every reader
/// joins it onto the standard grid, so a re-framed copy has no business there — and
/// `write_computed_curves_versioned` would DELETE the curve's current rows before appending, which
/// on a re-frame means blanking the interpretation and replacing it with rows that align with
/// nothing. Silently: the run would report success and the log view would go empty.
fn write_own_frame(
    conn: &Connection,
    well_id: &str,
    spec: &equations::LogSetSpec,
    depth: &[f32],
    curves: &[(String, Vec<f32>)],
) -> duckdb::Result<i64> {
    crate::db::with_txn(conn, |conn| {
        let (set_id, version) = equations::create_log_set(conn, well_id, spec)?;
        conn.execute("UPDATE log_sets SET frame = 'OWN' WHERE set_id = ?1", params![set_id])?;
        // The appender is POSITIONAL: (set_id, well_id, depth, curve_name, value), which is not
        // the order `computed_curves` uses.
        let mut app = conn.appender("computed_curves_archive")?;
        for (name, values) in curves {
            for (d, v) in depth.iter().zip(values.iter()) {
                app.append_row(params![set_id, well_id, d, name, v])?;
            }
        }
        app.flush()?;
        Ok(version)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp(n: usize, step: f64, start: f64) -> Vec<f32> {
        (0..n).map(|i| (start + i as f64 * step) as f32).collect()
    }

    /// **A downsample AVERAGES the interval rather than picking a sample out of it.**
    ///
    /// The distinction is invisible on a smooth curve and decisive on a laminated one: picking
    /// every Nth sample of an alternating lamination returns whichever phase the output frame
    /// happened to land on, which is a number about the sampling rather than about the rock. It is
    /// the same failure `extract_core_log`'s resampler was written to avoid, and this test feeds
    /// the same fixture — a curve alternating sample by sample, which must come back at its mean.
    #[test]
    fn a_downsample_averages_the_rock_instead_of_sampling_one_phase_of_it() {
        let src = ramp(100, 0.1, 1000.0);
        let lamination: Vec<f32> = (0..100).map(|i| if i % 2 == 0 { 10.0 } else { 30.0 }).collect();
        let out_depth = ramp(20, 0.5, 1000.0);
        let out = resample_onto(&src, &lamination, &out_depth, Method::Mean);
        // Every output sample lands strictly BETWEEN the two phases: it is a statement about the
        // interval, not about which phase the frame happened to hit. (Not exactly 20 — a 0.5 m box
        // over a 0.1 m source holds five samples, and five alternating values average to 18 or 22.
        // That is the arithmetic being right, not the resampler being wrong.)
        for (i, v) in out.iter().enumerate().take(19).skip(1) {
            assert!(*v > 10.0 && *v < 30.0, "sample {i} came back {v}, which is one phase rather than the rock");
        }
        // The control: taking the nearest sample instead returns a phase every time — which is
        // exactly why NEAREST is offered for class curves and never used as the default.
        let picked = resample_onto(&src, &lamination, &out_depth, Method::Nearest);
        assert!(
            picked.iter().all(|v| (*v - 10.0).abs() < 1e-3 || (*v - 30.0).abs() < 1e-3),
            "nearest returns a real sample every time, so the two methods genuinely differ: {picked:?}"
        );
    }

    /// **A source sample belongs to exactly ONE output box.** Closed boxes at both ends count the
    /// sample on a boundary twice — once in each neighbour — so a resampled curve no longer
    /// averages to what the source held, and the error is worst exactly where the sampling divides
    /// evenly, which is the ordinary case.
    ///
    /// The fixture is chosen so the box BOUNDARIES land exactly on source samples — offset the
    /// output frame by half a box and every boundary is a source depth. With the value equal to
    /// the sample index, an interior box of five consecutive samples has an exact mean, and a
    /// closed-at-both-ends box would hold six and miss it by half. Boundaries that fall between
    /// samples (the obvious fixture) exercise nothing at all.
    #[test]
    fn a_source_sample_is_counted_by_one_output_box_and_no_other() {
        let src = ramp(100, 0.1, 1000.0);
        let vals: Vec<f32> = (0..100).map(|i| i as f32).collect();
        // Boxes [1000.0, 1000.5), [1000.5, 1001.0), … — every edge is a source depth.
        let out_depth = ramp(19, 0.5, 1000.25);
        let out = resample_onto(&src, &vals, &out_depth, Method::Mean);
        for i in 1..18 {
            let want = 5.0 * i as f32 + 2.0; // samples 5i..5i+4
            assert!(
                (out[i] - want).abs() < 1e-3,
                "box {i} averaged {} rather than {want} — a boundary sample is being counted twice",
                out[i]
            );
        }
    }

    /// **A permeability averaged arithmetically is a rock that does not exist**, and the three
    /// means differ by orders of magnitude on a laminated sand-shale. Same rule and same numbers
    /// as `frame::block`'s upscaling test — stated again here because a resample is an upscale by
    /// another name, and the two must not disagree about it.
    ///
    /// Jauhar, 2026-08-05: *"we should have option with logarithmic data, even using geometric or
    /// harmonic"*.
    #[test]
    fn the_three_means_are_offered_because_they_are_orders_of_magnitude_apart() {
        let src = ramp(10, 0.1, 1000.0);
        let perm: Vec<f32> = (0..10).map(|i| if i % 2 == 0 { 1000.0 } else { 0.01 }).collect();
        let out_depth = vec![1000.45f32];
        let a = resample_onto(&src, &perm, &out_depth, Method::Mean)[0];
        let g = resample_onto(&src, &perm, &out_depth, Method::Geometric)[0];
        let h = resample_onto(&src, &perm, &out_depth, Method::Harmonic)[0];
        assert!((a - 500.0).abs() < 1.0, "arithmetic: {a}");
        assert!((g - 3.16).abs() < 0.1, "geometric: {g}");
        assert!(h < 0.03, "harmonic: {h}");
        assert!(a > g && g > h, "and the arithmetic answer is always the flattering one");
    }

    /// **A class curve is never averaged**, and `Auto` decides that from the VALUES.
    ///
    /// The mean of facies 1 and facies 4 is not facies 2.5 — it is a facies that does not exist,
    /// and every reader downstream will colour it, count it and put it in a table. A name-based
    /// rule would miss a class curve called anything else, which is most of them.
    #[test]
    fn a_class_curve_is_carried_by_its_commonest_value_rather_than_averaged() {
        let src = ramp(20, 0.1, 1000.0);
        // Facies 1 for the first three samples of each output box, then 4 — the mode is 1.
        let facies: Vec<f32> = (0..20).map(|i| if i % 5 < 3 { 1.0 } else { 4.0 }).collect();
        let out_depth = ramp(4, 0.5, 1000.0);
        let auto = resample_onto(&src, &facies, &out_depth, Method::Auto);
        for v in auto.iter() {
            assert!(
                (*v - 1.0).abs() < 1e-6 || (*v - 4.0).abs() < 1e-6,
                "a class curve must come back as one of its own classes, not {v}"
            );
        }
        assert!(looks_discrete(&facies), "and the test rests on the detector agreeing");
        // The near miss, found by the end-to-end test rather than by reading the code: a gamma ray
        // alternating 40 and 80 API is two non-negative whole numbers, which a "small integers"
        // rule calls a class curve — and then returns 80 where the rock averages 60.
        let gr: Vec<f32> = (0..20).map(|i| if i % 2 == 0 { 40.0 } else { 80.0 }).collect();
        assert!(!looks_discrete(&gr), "two round API values are not a facies scheme");
        assert!(
            !looks_discrete(&[0.0, 1.0, 200.0, 0.0]),
            "and anything reaching past a plausible code is a measurement"
        );
        assert!(looks_discrete(&[0.0, 1.0, 0.0, 1.0]), "while a 0/1 flag still is one");
        // The control: a continuous curve of the same shape IS averaged, so `Auto` is really
        // looking rather than always choosing MODE.
        let porosity: Vec<f32> = (0..20).map(|i| if i % 5 < 3 { 0.10 } else { 0.40 }).collect();
        assert!(!looks_discrete(&porosity));
        let mixed = resample_onto(&src, &porosity, &out_depth, Method::Auto);
        assert!(
            mixed.iter().any(|v| *v > 0.11 && *v < 0.39),
            "a continuous curve must be averaged: {mixed:?}"
        );
    }

    /// **An output sample with no input inside it is MISSING, never the nearest value.** A gap in
    /// the source is rock nobody logged, and a resample that bridges it hands every reader
    /// downstream a measurement that was never made.
    #[test]
    fn a_gap_in_the_source_stays_a_gap_on_the_new_frame() {
        let mut src = ramp(100, 0.1, 1000.0);
        let mut vals = vec![50.0f32; 100];
        // A 3 m hole in the middle: the depths are still there, the values are not.
        for v in vals.iter_mut().take(70).skip(40) {
            *v = MISSING;
        }
        let out = resample_onto(&src, &vals, &ramp(20, 0.5, 1000.0), Method::Mean);
        assert!(out[10].is_nan(), "the middle of the hole must stay MISSING, got {}", out[10]);
        assert!((out[0] - 50.0).abs() < 1e-3, "and the logged part is untouched");

        // Depths missing entirely (not just values) behave the same way.
        src.truncate(40);
        vals.truncate(40);
        let out = resample_onto(&src, &vals, &ramp(20, 0.5, 1000.0), Method::Mean);
        assert!(out[19].is_nan(), "below the source's last sample there is nothing to average");
    }

    /// **The case this tool exists for, end to end.** A well logged at 0.1524 m re-framed onto the
    /// 0.5 m the rest of the field is on — and then READ back through the set, which is the half
    /// that makes it worth anything.
    ///
    /// Two things are checked that nothing else covers. The re-framed set must not touch the
    /// current store: `write_computed_curves_versioned` DELETEs a curve's current rows before
    /// appending, so a re-frame through the ordinary path would blank the readable 0.1524 m
    /// interpretation and replace it with rows that align with nothing — silently, reporting
    /// success. And a module reading through the set must come back on the SET's frame with the
    /// standard curves resampled onto it, or a run would pair a 0.5 m PHIE with a 0.1524 m GR and
    /// average samples from different rock.
    #[test]
    fn a_fine_well_can_be_re_framed_onto_the_field_sampling_and_read_back_on_it() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-FINE", None, None, Some(0.0)).unwrap();
        // 0.1524 m sampling — a 6-inch wireline delivery.
        let n = 400usize;
        let depths: Vec<f32> = (0..n).map(|i| (1000.0 + i as f64 * 0.1524) as f32).collect();
        let gr: Vec<f32> = (0..n).map(|i| if i % 2 == 0 { 40.0 } else { 80.0 }).collect();
        db::insert_standard_curves(
            &conn,
            wid,
            depths.clone(),
            gr.clone(),
            vec![f32::NAN; n],
            vec![f32::NAN; n],
            vec![f32::NAN; n],
            vec![f32::NAN; n],
            vec![f32::NAN; n],
        )
        .unwrap();
        save_curve_selection(
            &conn,
            &CurveSelection {
                name: "FINE_GR".into(),
                mode: CurveSelectionMode::Selected,
                members: vec!["GR".into()],
            },
        )
        .unwrap();

        let req = ReframeRequest {
            well_ids: vec![wid.to_string()],
            source: SourceSpec { kind: "standard".into(), name: None },
            selection_name: "FINE_GR".into(),
            substitutions: vec![],
            target: TargetSpec {
                kind: "step".into(),
                step: Some(0.5),
                align: false,
                well_id: None,
                set_name: None,
                top: None,
                base: None,
            },
            methods: HashMap::new(),
            default_method: Method::Auto,
            output_set: "FIELD_05".into(),
            preview: false,
        };
        let res = run_reframe(&conn, &req);
        assert!(res[0].error.is_none(), "{:?}", res[0].error);
        assert!((res[0].source_step - 0.1524).abs() < 1e-3, "source step: {}", res[0].source_step);
        assert!((res[0].target_step - 0.5).abs() < 1e-3, "target step: {}", res[0].target_step);
        assert_eq!(res[0].version, Some(1));
        let carried = res[0].curves.iter().find(|c| c.name == "GR").expect("GR must be carried");
        assert!(carried.samples_out > 100, "the new frame must actually hold values: {carried:?}");

        // The current store is untouched: the well is still readable at its own 0.1524 m.
        let (raw_depth, raw_cols) =
            crate::equations::fetch_curve_frame(&conn, &wid.to_string(), &["GR".to_string()]).unwrap();
        assert_eq!(raw_depth.len(), n, "the well's own frame is unchanged");
        assert_eq!(raw_cols["GR"].iter().filter(|v| v.is_finite()).count(), n);

        // Read THROUGH the set: the frame becomes 0.5 m and GR comes with it, averaged rather
        // than sampled — the lamination averages to 60, neither of its two phases.
        let (set_depth, set_cols) = crate::equations::fetch_curve_frame_from_set(
            &conn,
            &wid.to_string(),
            &["GR".to_string()],
            Some("FIELD_05"),
            None,
        )
        .unwrap();
        assert!((median_step(&set_depth) - 0.5).abs() < 1e-3, "the run frame follows the set");
        assert!(set_depth.len() < n / 2, "and it is coarser: {} rows", set_depth.len());
        let mid = set_cols["GR"][set_depth.len() / 2];
        assert!(mid > 45.0 && mid < 75.0, "GR must be averaged onto the new frame, got {mid}");
    }

    /// **An accepted named substitute is recorded on the resulting curve as provenance.**
    ///
    /// `SB-DIO-032` / T48, sourced to data-I/O finding D-15: substitution is legitimate only as
    /// an explicit user act. Pinned from both sides in one contract test: an unaccepted decision
    /// writes no set at all, while the accepted decision writes the substitute under its OWN name
    /// and carries the requested -> substitute mapping on that curve's log-set ancestry.
    #[test]
    fn an_accepted_named_substitute_is_recorded_on_the_resulting_curve_as_provenance() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-SUB", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(
            &conn,
            wid,
            vec![1000.0, 1001.0, 1002.0],
            vec![40.0, 50.0, 60.0],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
        )
        .unwrap();
        save_curve_selection(
            &conn,
            &CurveSelection {
                name: "CALI_REQUEST".into(),
                mode: CurveSelectionMode::Selected,
                members: vec!["CALI".into()],
            },
        )
        .unwrap();

        let mut req = ReframeRequest {
            well_ids: vec![wid.to_string()],
            source: SourceSpec { kind: "standard".into(), name: None },
            selection_name: "CALI_REQUEST".into(),
            substitutions: vec![CurveSubstitution {
                requested: "CALI".into(),
                substitute: "GR".into(),
                accepted: false,
            }],
            target: step_target("step", Some(1.0), false),
            methods: Default::default(),
            default_method: Method::Mean,
            output_set: "SUBSTITUTED".into(),
            preview: false,
        };

        let refused = run_reframe(&conn, &req);
        assert!(
            refused[0].error.as_deref().unwrap_or("").contains("not explicitly accepted"),
            "an absent acceptance must be a refusal: {:?}",
            refused[0].error
        );
        let before: i64 = conn
            .query_row("SELECT COUNT(*) FROM log_sets WHERE well_id = ?1", params![wid.to_string()], |r| r.get(0))
            .unwrap();
        assert_eq!(before, 0, "refusal must happen before the run record or curve is written");

        req.substitutions[0].accepted = true;
        let committed = run_reframe(&conn, &req);
        assert!(committed[0].error.is_none(), "{:?}", committed[0].error);
        assert_eq!(committed[0].curves.len(), 1);
        assert_eq!(committed[0].curves[0].name, "GR", "another curve is never relabelled as CALI");

        let (params_json, curve_name): (String, String) = conn
            .query_row(
                "SELECT s.params_json, a.curve_name
                 FROM log_sets s
                 JOIN computed_curves_archive a ON a.set_id = s.set_id
                 WHERE s.well_id = ?1
                 LIMIT 1",
                params![wid.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(curve_name, "GR", "the substitute keeps its source identity");
        let provenance: serde_json::Value = serde_json::from_str(&params_json).unwrap();
        assert_eq!(
            provenance["substitutions"],
            serde_json::json!([{"requested": "CALI", "substitute": "GR", "accepted": true}]),
            "the resulting curve's ancestry must preserve the explicit decision"
        );
    }

    /// **A saved curve selection reloads as a named object listing its members.**
    ///
    /// `SB-DIO-033` / T49, sourced to data-I/O finding D-19. The member order is part of the
    /// object and its `selected` mode is stored rather than implied; the control proves a document
    /// with no mode cannot deserialize into a selection and therefore cannot acquire a hidden one.
    #[test]
    fn a_saved_curve_selection_reloads_as_a_named_object_listing_its_members() {
        use crate::db;
        use duckdb::Connection;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let saved = save_curve_selection(
            &conn,
            &CurveSelection {
                name: "  PRIMARY INPUTS  ".into(),
                mode: CurveSelectionMode::Selected,
                members: vec!["rhob".into(), "GR".into(), "RHOB".into()],
            },
        )
        .unwrap();
        assert_eq!(saved.name, "PRIMARY INPUTS");
        assert_eq!(saved.members, vec!["RHOB", "GR"], "order survives and duplicates do not hide in the object");

        let reloaded = list_curve_selections(&conn).unwrap();
        assert_eq!(
            reloaded,
            vec![CurveSelection {
                name: "PRIMARY INPUTS".into(),
                mode: CurveSelectionMode::Selected,
                members: vec!["RHOB".into(), "GR".into()],
            }]
        );
        assert!(
            serde_json::from_str::<CurveSelection>(r#"{"name":"AMBIGUOUS","members":["GR"]}"#).is_err(),
            "a member list without a stated mode must not acquire a hidden interpretation"
        );
    }

    /// **The sampling reported is the MEDIAN spacing, not the mean.** A curve with one gap in it
    /// has a mean spacing that is nobody's sampling — a 0.1524 m log missing 30 m averages out
    /// near 0.18, and a user reading that would re-frame onto a number the tool invented.
    #[test]
    fn the_sampling_reported_is_one_a_gap_cannot_move() {
        let mut d = ramp(200, 0.1524, 1000.0);
        assert!((median_step(&d) - 0.1524).abs() < 1e-4);
        // Drop 30 m out of the middle: one enormous gap, 199 ordinary ones.
        d.drain(100..197);
        let median = median_step(&d);
        let mean: f64 = (d.last().unwrap() - d.first().unwrap()) as f64 / (d.len() - 1) as f64;
        assert!((median - 0.1524).abs() < 1e-4, "the median is unmoved: {median}");
        assert!(mean > 0.20, "while the mean has become a number nobody logged at: {mean}");
    }

    fn step_target(kind: &str, step: Option<f64>, align: bool) -> TargetSpec {
        TargetSpec {
            kind: kind.into(),
            step,
            align,
            well_id: None,
            set_name: None,
            top: None,
            base: None,
        }
    }

    /// **Aligned wells come out on the SAME depths, not merely the same step.**
    ///
    /// Two wells spudded at different depths, re-framed at one sampling. Anchored on their own
    /// source tops they share a step and no depth at all, and since every read in this app is an
    /// exact depth match, nothing downstream can put them side by side — the failure Reframe was
    /// built to fix, one level up. Pinned from BOTH sides: aligned must be identical, and unaligned
    /// must NOT be, or the test would pass on a build where the flag does nothing.
    #[test]
    fn aligned_wells_land_on_identical_depths_not_merely_the_same_step() {
        let conn = Connection::open_in_memory().unwrap();
        let a = ramp(100, 0.1, 1500.00);
        let b = ramp(100, 0.1, 1498.25);

        let shared = Some((1498.25, 1510.00));
        let fa = build_frame(&step_target("step", Some(0.5), true), &a, &conn, "A", shared).unwrap();
        let fb = build_frame(&step_target("step", Some(0.5), true), &b, &conn, "B", shared).unwrap();
        assert_eq!(fa, fb, "aligned wells must land on one frame, sample for sample");

        // The control. Without the shared interval each well anchors on its own first depth, so the
        // two frames share a spacing and not a single depth — which is precisely the bug.
        let ua = build_frame(&step_target("step", Some(0.5), false), &a, &conn, "A", None).unwrap();
        let ub = build_frame(&step_target("step", Some(0.5), false), &b, &conn, "B", None).unwrap();
        assert_ne!(ua, ub, "unaligned wells anchor on their own tops - if these match, align is inert");
        assert!(
            !ua.iter().any(|d| ub.contains(d)),
            "and they overlap at NO depth, which is why an exact-match read finds nothing: {:?} vs {:?}",
            &ua[..3],
            &ub[..3]
        );
    }

    /// **Regularize takes the source's own spacing, and does not quietly change how finely it was
    /// logged.** The operation is "make this uniform", not "make this coarser" — a user who wanted
    /// coarser would say so. Re-typing the number off the probe is the only alternative, and it is
    /// a chance to get it wrong that buys nothing.
    #[test]
    fn regularize_adopts_the_sources_own_spacing_when_no_step_is_given() {
        let conn = Connection::open_in_memory().unwrap();
        // An irregular source: mostly 0.1524 with two gaps, so the MEDIAN is the logged sampling
        // while the mean is a number nobody logged at.
        let mut d: Vec<f32> = ramp(40, 0.1524, 2000.0);
        d.extend(ramp(40, 0.1524, 2020.0));
        d.extend(ramp(40, 0.1524, 2050.0));

        let f = build_frame(&step_target("regularize", None, false), &d, &conn, "A", None).unwrap();
        let got = median_step(&f);
        assert!(
            (got - 0.1524).abs() < 1e-4,
            "regularize should land on the logged sampling, got {got}"
        );
        // And it is genuinely uniform now, which the source was not.
        let spans: Vec<f64> = f.windows(2).map(|w| (w[1] - w[0]) as f64).collect();
        assert!(
            spans.iter().all(|s| (s - got).abs() < 1e-3),
            "every gap is now the same; the whole point of regularizing"
        );
        assert!(
            d.windows(2).any(|w| ((w[1] - w[0]) as f64 - 0.1524).abs() > 1.0),
            "the fixture really was irregular, or this test proves nothing"
        );
    }

    /// **Regularizing several wells onto one frame is REFUSED rather than electing a well.**
    ///
    /// Regularize gets its step from the source; align needs one step for the whole run. Resolving
    /// that by taking some well's spacing would silently make that well the standard for the field,
    /// and the output would look entirely normal either way. The user states the step instead.
    #[test]
    fn regularize_across_wells_refuses_rather_than_electing_one_wells_spacing() {
        let conn = Connection::open_in_memory().unwrap();
        let req = ReframeRequest {
            well_ids: vec!["A".into(), "B".into()],
            source: SourceSpec { kind: "standard".into(), name: None },
            selection_name: "ANY_EXPLICIT_SELECTION".into(),
            substitutions: vec![],
            target: step_target("regularize", None, true),
            methods: Default::default(),
            default_method: Method::default(),
            output_set: "R".into(),
            preview: true,
        };
        let out = run_reframe(&conn, &req);
        assert_eq!(out.len(), 2, "every well still gets a row saying why");
        for r in &out {
            let msg = r.error.as_deref().unwrap_or("");
            assert!(msg.contains("sampling stated"), "refused by name, got {msg:?}");
        }

        // The control: state the step and the same request is accepted, so the refusal is about the
        // missing number and not about aligning at all.
        let mut ok = req;
        ok.target.step = Some(0.5);
        let out = run_reframe(&conn, &ok);
        assert!(
            out.iter().all(|r| r.error.as_deref().unwrap_or("") != {
                "regularizing several wells onto one frame needs the sampling stated"
            }),
            "with a step given, the sampling refusal is gone"
        );
    }
}
