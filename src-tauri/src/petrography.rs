//! Measurements taken off a thin section, starting with pore area from blue-dyed epoxy.
//!
//! The deliverable is an **area fraction per plate**, which under the standard stereological
//! argument (Delesse) estimates the volume fraction. It is deliberately the first of the three
//! measurement families because it is **dimensionless**: an area fraction needs no micrometres per
//! pixel, so it runs on every plate rather than only the calibrated ones (see
//! `docs/plan_image_analysis.md` §2.0).
//!
//! Four rules hold this together, and each closes a way of being confidently wrong.
//!
//! **A plate must be DECLARED impregnated, and an undeclared one is refused by name.** This is the
//! whole reason `well_images.prepared` exists. A blue rule run over a section nobody impregnated
//! does not fail — it returns a porosity assembled from blue-ish feldspar, stain bleed and edge
//! artefact, and that number then plots against core helium porosity looking entirely reasonable.
//! Nor can the app work it out from the pixels: the evidence for "this is blue epoxy" is the blue
//! it was about to measure, which is the same circle as reading a water zone off the saturation
//! being calibrated.
//!
//! **The colour band is the user's, not the app's.** The defaults here are a plain blue band —
//! round numbers, no calibration behind them — offered as a starting point for a VISUAL tuning
//! task, never as a constant that ships silently. The dialog shows the mask over the plate and the
//! user adjusts until it matches what they see down the microscope.
//!
//! **The preview comes from the SAME code as the measurement.** Drawing the mask in the frontend
//! would put the segmentation in two languages, and the two would drift — the mistake this repo
//! keeps a standing warning about for `composite.rs` against the log-view renderer. So the runner
//! returns the overlay PNG, and what the user tunes against is literally what gets measured.
//!
//! **No morphological cleaning.** Opening or closing a mask needs a structuring element measured
//! in PIXELS, which is a size — and a plate may carry no scale at all, so that size could not be
//! stated in microns for every plate. Rather than pick a pixel count that means a different
//! physical distance on every plate, nothing is smoothed and the speckle is left visible in the
//! preview where it can be judged.
//!
//! Results are POINT DATA at the plate's depth, not a curve. A thin section is cut from one plug
//! and measures that plug; joining a column of them with a line would state a continuity the data
//! does not have — the same argument that made point data a track kind rather than a `CurveStyle`.

use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::python_engine::{find_python, hide_console};

/// Plates per subprocess. Bounds the bytes held in memory and piped at once: a core-photograph
/// delivery can be hundreds of plates at roughly a megabyte each, and one batch of all of them
/// would be a gigabyte in flight for no gain.
const CHUNK: usize = 16;

/// The colour rule, in HSV. Hue in degrees, saturation and value 0..1.
///
/// **These defaults are a generic blue band, not a calibration.** Blue-dyed epoxy sits in the
/// blue-to-violet part of the wheel on any microscope; where exactly depends on the dye, the lamp,
/// the white balance and the scan, none of which this app knows. They exist so the preview has
/// something to draw on the first click, and they are round numbers on purpose — a two-decimal
/// threshold would be a regression result, and there is no regression behind these.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoreColorBand {
    pub hue_lo: f32,
    pub hue_hi: f32,
    pub sat_min: f32,
    pub val_min: f32,
}

impl Default for PoreColorBand {
    fn default() -> Self {
        Self { hue_lo: 180.0, hue_hi: 260.0, sat_min: 0.15, val_min: 0.10 }
    }
}

/// One depth interval, and the plate every section inside it is corrected onto.
///
/// A delivery that spans two cored intervals is two different rocks, usually photographed on two
/// different days, and one reference plate serves both only by accident. Measured on a real
/// delivery, giving each interval its own reference lifted rank agreement with core porosity in
/// BOTH of them (0.19 to 0.24 in the shallow core, 0.49 to 0.53 in the deep one). That is a
/// refinement rather than a rescue — and the point is that it is now something the user can
/// MEASURE on their own rock rather than be told.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReferenceZone {
    /// Shallowest depth this reference serves. `None` reaches up to the top of the well.
    #[serde(default)]
    pub top: Option<f32>,
    /// Deepest. `None` reaches down to total depth.
    #[serde(default)]
    pub base: Option<f32>,
    pub image_id: String,
}

impl ReferenceZone {
    /// Inclusive at BOTH ends, so `2000-2010` and `2010-2020` is how anyone writes two adjacent
    /// intervals and neither has to be typed a millimetre short. A plate landing exactly on the
    /// shared depth goes to whichever interval is listed first — the same rule the per-barrel core
    /// shifts follow, and the reason a true overlap is refused rather than resolved that way.
    fn contains(&self, d: f32) -> bool {
        let below_top = match self.top {
            Some(t) => d >= t,
            None => true,
        };
        let above_base = match self.base {
            Some(b) => d <= b,
            None => true,
        };
        below_top && above_base
    }
}

/// The interval, in words, for a message. `zone_span` rather than a raw pair because "2000 and
/// below" and "2000 to 2010" are different statements and a dash cannot make that difference.
fn zone_span(z: &ReferenceZone) -> String {
    match (z.top, z.base) {
        (Some(t), Some(b)) => format!("{t} to {b}"),
        (Some(t), None) => format!("{t} and below"),
        (None, Some(b)) => format!("{b} and above"),
        (None, None) => "the whole well".to_string(),
    }
}

/// One run of the pore measurement over a well's live image delivery.
#[derive(Debug, Clone, Deserialize)]
pub struct PoreSpec {
    pub well_id: String,
    pub dataset: String,
    #[serde(default)]
    pub band: PoreColorBand,
    /// Draw the overlay for this plate and return it. `None` measures without a picture back.
    #[serde(default)]
    pub preview_image_id: Option<String>,
    /// Measure only this plate. Used by the tuning preview so adjusting a slider does not
    /// re-measure a 300-plate delivery.
    #[serde(default)]
    pub only_image_id: Option<String>,
    /// Store the results as point data under this delivery name. `None` measures without writing —
    /// tuning must not leave a trail of half-judged answers in the project.
    #[serde(default)]
    pub set_name: Option<String>,
    /// Also measure the shape and size of each individual pore. Needs scipy; off by default so the
    /// area fraction still runs where scipy is not installed.
    #[serde(default)]
    pub geometry: bool,
    /// Smallest thing counted as a pore, in PIXELS. Deliberately in pixels rather than microns:
    /// it is a statement about what the picture can resolve, not about the rock, and it has to
    /// mean the same thing on a plate that carries no scale at all.
    #[serde(default = "default_min_pore_px")]
    pub min_pore_px: u32,
    /// Also outline the individual GRAINS and measure their size. Needs scipy, like the pore
    /// geometry, and inherits the same blue-epoxy refusal: the grain phase here is defined as
    /// everything the pore rule did not claim, so a plate where pore cannot be told from solid
    /// cannot have its grains outlined either.
    #[serde(default)]
    pub grains: bool,
    /// Smallest thing counted as a grain, in PIXELS — same reasoning as `min_pore_px`.
    #[serde(default = "default_min_grain_px")]
    pub min_grain_px: u32,
    /// How far apart two grain centres must be before the watershed calls them two grains, in
    /// PIXELS. This is the knob that decides over-segmentation, which is the failure mode of a
    /// distance-transform watershed, and it is judged against the preview.
    #[serde(default = "default_grain_sep_px")]
    pub grain_sep_px: u32,
    /// Also report the Wicksell-corrected size distribution beside the apparent one. OFF by
    /// default (Jauhar, 2026-07-31: "apply wicksell correction is optional") — a correction
    /// carries assumptions of its own, and a corrected number must never leave the app without
    /// having been asked for. The two are stored under DIFFERENT item names, so neither can be
    /// mistaken for the other downstream.
    #[serde(default)]
    pub wicksell: bool,
    /// The plate the band was tuned on. Every other plate is colour-corrected onto this one before
    /// the band is applied, which is what lets one band serve a delivery photographed under more
    /// than one light. `None` reads every plate exactly as delivered — the behaviour this app
    /// always had, and still the right one for a delivery shot in a single session.
    ///
    /// Naming a reference also turns on the empty-measurement refusal: see [`band_missed`].
    #[serde(default)]
    pub reference_image_id: Option<String>,
    /// References for particular depth intervals, overruling the one above where they reach. Empty
    /// is the delivery-wide behaviour and stays the default.
    ///
    /// A plate that no interval covers falls back to [`PoreSpec::reference_image_id`]; where that is
    /// `None` too the plate is REFUSED by name rather than read as delivered. One stored set holding
    /// both corrected and uncorrected fractions would be two measurements under one name, and
    /// [`band_missed`] — which only ever fires on a corrected plate — would quietly switch off for
    /// half of them.
    #[serde(default)]
    pub reference_zones: Vec<ReferenceZone>,
    /// Read the STAIN as well, giving a mineral area fraction per class. `None` means no stain is
    /// assumed — the default, and the only safe one: a stain assumed is a mineral fraction
    /// invented (`docs/plan_image_analysis.md` §2.1 A2).
    #[serde(default)]
    pub stain: Option<StainSpec>,
    /// Score this run against an independent measurement of the same plugs — the core porosity the
    /// laboratory measured on the plug each section was cut from, usually. `None` skips the check.
    ///
    /// This exists because [`PoreSpec::reference_image_id`] turned out to be a bigger lever on the
    /// answer than the colour band is (a 3.5x spread in rank agreement across three reference
    /// plates from one cored interval, with the worst pick worse than not correcting at all), and
    /// the dialog offered nothing to tell a good choice from a bad one except the preview. A
    /// setting judged by eye against a picture is a setting judged on how the picture looks; this
    /// is the number that says whether it also tracks the rock.
    #[serde(default)]
    pub check_against: Option<crate::plugqc::PlugSource>,
    /// Two measurements further apart than this are not the same plug. Defaults to `plugqc`'s own.
    #[serde(default)]
    pub check_depth_tol: f32,
}

/// Whether a plate's numbers may leave the run.
///
/// The single statement of a rule the write path and the agreement check must never disagree
/// about. Both refusals are already documented where they are computed — [`scene_dominated`] and
/// its mirror [`band_missed`] — and what matters here is that a plate the run has ALREADY declared
/// unmeasurable must not vote on whether the run is any good. Score it and the scene-dominance
/// failures this guard exists to catch would flatter or wreck the very number the user is choosing
/// a reference plate on.
fn storable(p: &PlatePore) -> bool {
    !p.scene_dominated && !p.band_missed
}

/// Refuses a set of intervals the run could not act on unambiguously, before any picture is decoded.
///
/// Intervals may TOUCH — `2000-2010` beside `2010-2020` is how anyone writes two adjacent cored
/// sections, and neither should have to be typed a millimetre short. What is refused is a genuine
/// OVERLAP: across one, which reference a plate is corrected onto would be decided by the order of a
/// list nobody sees, so the same settings could give two answers and nothing on screen would say
/// why. The same rule `db::apply_core_run_shifts` enforces on core barrels, for the same reason.
fn check_zones(spec: &PoreSpec) -> Result<(), String> {
    for z in &spec.reference_zones {
        if z.image_id.trim().is_empty() {
            return Err(format!(
                "the interval covering {} has no reference plate. Every plate in an interval \
                 is colour-corrected onto that one plate, so without it there is nothing to \
                 correct onto. Choose one in Reference plate, or delete the interval.",
                zone_span(z)
            ));
        }
        // Refused, not silently swapped. A base above its top is a typo or a transposed column, and
        // guessing which number was meant is how it survives into a deliverable.
        if let (Some(t), Some(b)) = (z.top, z.base) {
            if b < t {
                return Err(format!(
                    "an interval runs from {t} down to {b}, which is backwards - a base above its \
                     top is a typo, and quietly swapping them would hide it"
                ));
            }
        }
    }
    for (i, a) in spec.reference_zones.iter().enumerate() {
        for b in spec.reference_zones.iter().skip(i + 1) {
            let lo_a = a.top.unwrap_or(f32::NEG_INFINITY);
            let hi_a = a.base.unwrap_or(f32::INFINITY);
            let lo_b = b.top.unwrap_or(f32::NEG_INFINITY);
            let hi_b = b.base.unwrap_or(f32::INFINITY);
            // Strict on both sides, so a shared boundary depth is not an overlap.
            if lo_a < hi_b && lo_b < hi_a {
                return Err(format!(
                    "two intervals overlap ({} and {}). Which reference a plate inside the overlap \
                     is corrected onto would come down to the order of the list, which is not \
                     something to leave to chance - intervals may touch at one depth, but not cross.",
                    zone_span(a),
                    zone_span(b)
                ));
            }
        }
    }
    Ok(())
}

/// The plate a section at this depth is corrected onto: the first interval that covers it, else the
/// delivery-wide reference. `None` means nothing was named for it.
///
/// A plate with no usable depth matches no interval (every comparison against NaN is false) and so
/// falls through to the delivery-wide reference, which is the honest answer — nothing about it says
/// which interval it belongs to.
fn reference_for<'a>(spec: &'a PoreSpec, depth: f32) -> Option<&'a str> {
    for z in &spec.reference_zones {
        if z.contains(depth) {
            return Some(z.image_id.as_str());
        }
    }
    spec.reference_image_id.as_deref()
}

/// The plates that go into the agreement check, at the depths they will be paired on.
///
/// Split out from [`run_pore_area`] so the rule can be pinned without a Python subprocess: what
/// this returns is what decides whether the number the user picks a reference plate on was
/// computed over the same rock twice.
fn storable_samples(plates: &[PlatePore]) -> Vec<crate::plugqc::MeasuredSample> {
    plates
        .iter()
        .filter(|p| storable(p))
        .map(|p| crate::plugqc::MeasuredSample {
            // An interval plate is anchored at its MIDDLE — the convention `plugqc` and the point
            // tracks already use, so this number, the plot and the log view agree about where the
            // plate is.
            depth: match p.depth_base.filter(|b| b.is_finite()) {
                Some(b) => (p.depth_top + b) * 0.5,
                None => p.depth_top,
            },
            value: p.pore_fraction,
        })
        .collect()
}

/// An HSV window. Richer than [`PoreColorBand`] because a stain scheme has to be able to say
/// **unstained** — dolomite under alizarin red S is identified by staying colourless, which is a
/// saturation CEILING and cannot be written as a floor.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StainBand {
    pub hue_lo: f32,
    pub hue_hi: f32,
    pub sat_min: f32,
    pub sat_max: f32,
    pub val_min: f32,
    pub val_max: f32,
}

/// One mineral the stain is expected to reveal, and the colour it shows as.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StainClass {
    pub mineral: String,
    pub band: StainBand,
}

/// What stain was applied, and how it is being read.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StainSpec {
    /// The stain this scheme is for. Matched against each plate's OWN declared stain, and a plate
    /// that disagrees is refused BY NAME — reading an alizarin-red scheme off a section stained
    /// with something else returns mineral fractions that are wrong and entirely plausible.
    pub stain: String,
    /// Tested IN ORDER, first match wins. A pixel is one mineral; overlapping bands are resolved
    /// by the order the user put them in rather than silently counted twice.
    pub classes: Vec<StainClass>,
}

/// Published stain identifications, offered as starting points for the class list.
///
/// **The mineral identifications are standard carbonate petrography** — Friedman (1959) for
/// alizarin red S, Dickson (1966) for the combined alizarin red S + potassium ferricyanide stain,
/// both reproduced in every carbonate text and already named in `docs/plan_image_analysis.md` §2.1.
///
/// **The colour bands are not from any paper.** What hue a stained calcite photographs as depends
/// on the dye batch, the concentration, the etch, the lamp, the white balance and the scan, none of
/// which this app knows. They are round numbers to start a VISUAL tuning from, exactly like the
/// epoxy band, and the preview is how they get judged.
pub fn stain_scheme(name: &str) -> Option<Vec<StainClass>> {
    let cls = |mineral: &str, hue_lo: f32, hue_hi: f32| StainClass {
        mineral: mineral.to_string(),
        band: StainBand { hue_lo, hue_hi, sat_min: 0.2, sat_max: 1.0, val_min: 0.1, val_max: 1.0 },
    };
    // A mineral the stain leaves colourless, identified by the ABSENCE of colour — which is the
    // whole reason the band model needs a saturation ceiling.
    let unstained = |mineral: &str| StainClass {
        mineral: mineral.to_string(),
        band: StainBand {
            hue_lo: 0.0,
            hue_hi: 360.0,
            sat_min: 0.0,
            sat_max: 0.15,
            val_min: 0.3,
            val_max: 1.0,
        },
    };
    match normalize_stain(name).as_str() {
        // Friedman (1959): alizarin red S stains calcite, leaves dolomite colourless.
        "alizarinreds" => Some(vec![cls("Calcite", 330.0, 20.0), unstained("Dolomite")]),
        // Dickson (1966): the combined stain separates the ferroan phases as well.
        "alizarinredspotassiumferricyanide" | "dickson" => Some(vec![
            cls("Ferroan calcite", 260.0, 330.0),
            cls("Calcite", 330.0, 20.0),
            cls("Ferroan dolomite", 170.0, 260.0),
            unstained("Dolomite"),
        ]),
        // Potassium ferricyanide alone marks the ferroan phases and nothing else.
        "potassiumferricyanide" => {
            Some(vec![cls("Ferroan phases", 170.0, 260.0), unstained("Non-ferroan")])
        }
        _ => None,
    }
}

/// Every scheme this build ships, for the dialog's picker.
pub fn stain_scheme_names() -> Vec<String> {
    ["Alizarin red S", "Alizarin red S + potassium ferricyanide", "Potassium ferricyanide"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Stain names are free text off a laboratory report, so they are compared with punctuation and
/// spacing thrown away — "Alizarin Red S" and "alizarin-red-s" are one stain.
pub fn normalize_stain(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_alphanumeric()).flat_map(|c| c.to_lowercase()).collect()
}

/// Whether a stain class could be confused with the blue epoxy that marks pore.
///
/// This is a real collision, not a hypothetical: under Dickson's stain ferroan dolomite goes
/// TURQUOISE, and blue-dyed epoxy is blue. On a section that was both impregnated and stained, the
/// pore rule claims those pixels first, so a ferroan dolomite grain is silently counted as
/// porosity — inflating `VPORE_TS` and removing a mineral, both plausibly. Reported, never
/// resolved automatically: which band to narrow is a judgement made looking at the plate.
pub fn epoxy_collides(pore: &PoreColorBand, band: &StainBand) -> bool {
    let hue_overlaps = |a_lo: f32, a_hi: f32, b_lo: f32, b_hi: f32| {
        let inside = |x: f32, lo: f32, hi: f32| {
            if lo <= hi {
                x >= lo && x <= hi
            } else {
                x >= lo || x <= hi
            }
        };
        inside(b_lo, a_lo, a_hi) || inside(b_hi, a_lo, a_hi) || inside(a_lo, b_lo, b_hi)
    };
    hue_overlaps(pore.hue_lo, pore.hue_hi, band.hue_lo, band.hue_hi)
        && band.sat_max >= pore.sat_min
        && band.val_max >= pore.val_min
}

/// What the stain came to on one plate.
///
/// Fractions are of the WHOLE plate, so pore plus every mineral plus the unclassified remainder is
/// 1 — the only form in which they can be read beside `VPORE_TS` as a modal analysis.
#[derive(Debug, Clone, Serialize)]
pub struct PlateStain {
    pub fractions: Vec<(String, f32)>,
    /// Solid that fell in no band. **The honesty number for this family**: a section where a third
    /// of the rock matched nothing has not been given a mineralogy, whatever the other rows say.
    pub unclassified: f32,
}

fn default_min_pore_px() -> u32 {
    MIN_PORE_PX
}

fn default_min_grain_px() -> u32 {
    MIN_GRAIN_PX
}

fn default_grain_sep_px() -> u32 {
    GRAIN_SEP_PX
}

/// Below this a blob is speckle rather than a pore. Round, and stated in pixels for the reason
/// given on `PoreSpec::min_pore_px`.
pub const MIN_PORE_PX: u32 = 20;

/// Below this a patch of solid is not a grain. Larger than the pore floor because a grain is the
/// larger object, and round for the same reason the colour band is: it is a starting point for a
/// visual judgement, not a calibration.
pub const MIN_GRAIN_PX: u32 = 50;

/// Default minimum distance between two grain centres, in pixels. Round on purpose — see
/// `PoreSpec::grain_sep_px`.
pub const GRAIN_SEP_PX: u32 = 20;

/// Saltykov classes for the Wicksell unfolding. Twelve is the published convention, and it is
/// about as far as the inversion can be pushed before the class-to-class subtraction turns into
/// noise amplification.
const SALTYKOV_CLASSES: usize = 12;

/// What one plate came to.
#[derive(Debug, Clone, Serialize)]
pub struct PlatePore {
    pub image_id: String,
    pub name: String,
    pub depth_top: f32,
    pub depth_base: Option<f32>,
    /// Pore area as a fraction of the plate, v/v.
    pub pore_fraction: f32,
    /// The plate's OWN median hue in degrees — what colour this picture mostly is.
    pub scene_hue: f32,
    /// Set when that median hue falls inside the declared pore band, which means the band is
    /// describing the scene rather than the pores. The fraction above is still reported so the
    /// band can be tuned against it, but the plate is left out of the write. See
    /// [`scene_dominated`].
    pub scene_dominated: bool,
    /// How far this photograph's light sat from the reference plate's, in degrees of hue — the
    /// size of the correction that was applied. NaN when no reference was named. Diagnostic, never
    /// a threshold: it is what turns "the band found nothing here" into a reason.
    pub cast_shift: f32,
    /// The plate this one was corrected onto, by name; empty when nothing was. Reported because with
    /// more than one reference in play, a shift of 40 degrees means nothing until you know which
    /// plate it is 40 degrees from.
    pub reference_name: String,
    /// Set when the band, transferred onto this plate, claimed less than one resolvable pore. Only
    /// ever set on a normalized run — see [`band_missed`]. Kept out of the write like a
    /// scene-dominated plate, and for the mirror reason.
    pub band_missed: bool,
    /// Pixels examined — the whole plate, since nothing is masked out.
    pub pixels: i64,
    /// Shape and size of the individual pores, when geometry was asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<PoreGeometry>,
    /// Size of the individual grains, when grains were asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grains: Option<GrainStats>,
    /// Mineral area fractions, when a stain was declared and read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stain: Option<PlateStain>,
}

/// What the individual grains on one plate came to.
///
/// **Everything dimensional here is APPARENT unless its name says otherwise.** A random plane
/// rarely cuts a grain through its centre, so section diameters run systematically small and
/// section sorting systematically worse than the rock's. Reporting apparent values under an
/// unqualified name is how that bias travels into a deliverable unnoticed, which is why the
/// apparent and corrected statistics carry different item names rather than one name and a flag.
#[derive(Debug, Clone, Serialize)]
pub struct GrainStats {
    /// Grains measured: big enough, and not cut by the frame.
    pub n: usize,
    /// Grains dropped for touching the plate edge — their true size is unknown, and keeping them
    /// biases the distribution small. The same stereological edge rule the pores follow.
    pub n_edge: usize,
    /// Grains dropped as too small to be anything but debris or a watershed sliver.
    pub n_small: usize,
    /// Median equivalent-ellipse aspect ratio of the SECTIONS. Dimensionless, so every plate.
    pub aspect_p50: f32,
    /// Median fraction of a grain's outline that is a grain-to-grain contact rather than open
    /// pore. **This is the honesty number for the whole family.** Where grains are welded by
    /// cement or an overgrowth there is nothing in the picture to separate them, and the watershed
    /// will put a line at the narrowest point of the blob anyway — a geometric artefact, not a
    /// grain boundary. A high value says most of what was called a boundary was inferred.
    pub contact_p50: f32,
    /// Apparent equivalent-circle diameters in MICROMETRES, area-weighted. `None` on a plate with
    /// no declared scale — a diameter in pixels is not a diameter.
    pub d10_app_um: Option<f32>,
    pub d50_app_um: Option<f32>,
    pub d90_app_um: Option<f32>,
    /// Folk & Ward (1957) inclusive graphic standard deviation of the APPARENT distribution, in
    /// phi units. Phi is −log2(d in mm), so this needs a scale as much as the diameters do.
    pub sort_app_phi: Option<f32>,
    /// The same four after the Wicksell correction, when it was asked for.
    pub d10_w_um: Option<f32>,
    pub d50_w_um: Option<f32>,
    pub d90_w_um: Option<f32>,
    pub sort_w_phi: Option<f32>,
    /// Saltykov classes whose unfolded population came out NEGATIVE and was clamped to zero. The
    /// inversion is ill-conditioned — that is a property of Wicksell's problem, not of this
    /// implementation — so a plate with several of these has an unstable correction and the number
    /// is here to say so rather than to be hidden.
    pub w_clamped: usize,
}

/// What the individual pores on one plate came to.
#[derive(Debug, Clone, Serialize)]
pub struct PoreGeometry {
    /// Pores measured: big enough, and not cut by the frame.
    pub n: usize,
    /// Pores dropped for touching the plate edge. Reported because their true size is unknown and
    /// excluding them is what keeps the size distribution honest — see [`run_pore_area`].
    pub n_edge: usize,
    /// Pores dropped as too small to be anything but speckle.
    pub n_small: usize,
    /// Median and spread of the equivalent-ellipse aspect ratio. Dimensionless, so it is reported
    /// for every plate including the uncalibrated ones.
    pub aspect_p50: f32,
    pub aspect_p90: f32,
    /// Median circularity, 4·pi·A/P². 1 is a circle. Dimensionless.
    pub shape_p50: f32,
    /// Equivalent-circle diameter in MICROMETRES, AREA-WEIGHTED. `None` on a plate with no
    /// declared scale — a diameter in pixels is not a diameter.
    pub d10_um: Option<f32>,
    pub d50_um: Option<f32>,
    pub d90_um: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PoreResult {
    pub plates: Vec<PlatePore>,
    /// Plates left out and why, one entry each — never a silent subset.
    pub skipped: Vec<String>,
    /// Base64 PNG of the mask drawn over the requested plate, when one was asked for.
    pub preview_png: Option<String>,
    /// The same picture, same size, WITHOUT the mask — the corrected plate as it actually looks.
    /// Sent with the overlay rather than fetched separately so the two can never be one plate's
    /// mask over another plate's pixels, the argument `CorePreview.before_png` already makes.
    pub plain_png: Option<String>,
    pub preview_width: i32,
    pub preview_height: i32,
    /// Point dataset and delivery written, when `set_name` was given.
    pub written: Option<(String, String)>,
    /// How the STORABLE plates agreed with an independent plug measurement, when one was named.
    /// Computed whether or not the run was saved, so a setting can be judged before it is kept.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agreement: Option<crate::plugqc::Agreement>,
    pub notes: Vec<String>,
}

/// The point-data item a measured pore fraction is stored as.
pub const PORE_ITEM: &str = "VPORE_TS";
/// The point dataset it lands in. Deliberately its own dataset rather than the image delivery's
/// name: it is a MEASUREMENT derived from the pictures, not part of the delivery, and re-running
/// the measurement must not look like a second delivery of plates.
pub const PORE_DATASET: &str = "PETROGRAPHY";

const PORE_RUNNER: &str = r#"
import sys, json, io, base64
try:
    import numpy as np
    from PIL import Image
except Exception as e:
    sys.stderr.write("needs numpy and Pillow: %s\n" % e)
    sys.exit(1)

# stdin.buffer, never stdin: a piped child's TEXT stdin decodes with the Windows ANSI codepage
# while serde_json emits UTF-8, so a plate name with any non-ASCII character arrives as mojibake.
header = json.loads(sys.stdin.buffer.readline().decode("utf-8"))
band = header["band"]
sizes = header["sizes"]
ids = header["ids"]
preview = header.get("preview")
# The reference plate's own matrix colour, when the user named one. Absent means every plate is
# read exactly as delivered, which is what the app always did.
reference_rgb = header.get("reference_rgb")

blobs = []
for n in sizes:
    blobs.append(sys.stdin.buffer.read(n))

def mask_from(h, s, v):
    # Where the pixel is grey the hue is undefined; `hsv_from` leaves it at 0 and the saturation
    # floor is what actually rejects it, so an undefined hue never counts as blue.
    return in_band(h, s, v, float(band["hue_lo"]), float(band["hue_hi"]),
                   float(band["sat_min"]), 1.0, float(band["val_min"]), 1.0)


def rgb_of(img):
    return np.asarray(img.convert("RGB"), dtype=np.float32) / 255.0


def matrix_rgb(a, pore):
    """This picture's MATRIX colour: channel-wise medians of the pixels the band did NOT claim.

    Channel-wise medians rather than the median pixel, because three numbers are exactly what a
    diagonal correction needs.

    Deliberately NOT the whole plate's median, which is what this first shipped as and which is
    wrong in a way that looks right. The whole-plate median moves with how much epoxy is in the
    field of view - a plate with more pore has a bluer median - so anchoring the correction on it
    partly normalizes away the very contrast being measured. That is the grey-world trap reached by
    a different route. Measured on a real delivery against the petrographer's own point count: rank
    agreement 0.19 uncorrected, 0.05 on the whole-plate anchor, 0.20 on this one. The same delivery
    photographed each plug twice, and the two fields of view differ in whole-plate median hue by 66
    degrees at p90 - far more than one lamp can explain, which is what says the whole-plate median
    is measuring the rock rather than the light.

    One iteration, and it terminates: the uncorrected band defines the matrix, the gain follows,
    the band is applied again. `None` where the band claimed nearly everything - there is no matrix
    left to anchor on, and the scene guard is about to refuse that plate anyway."""
    solid = ~pore
    if int(np.count_nonzero(solid)) < 100:
        return None
    return [float(np.median(a[..., c][solid])) for c in range(3)]


def gain_for(med):
    """Per-channel gain putting this plate's matrix colour where the reference plate's sits.

    The von Kries diagonal model, with the delivery's OWN rock as the reference patch instead of
    an assumed grey. Grey-world would be wrong here in a way that would never look wrong: a
    blue-epoxy section IS genuinely blue-biased, and the more porous it is the more so, so forcing
    the three channel means together would normalize away the very signal being measured.

    Scaled so the LARGEST gain is 1. The correction is a relative rebalance, and this way no
    channel can be pushed past 1 and clipped - clipping would distort the hue of exactly the
    brightest pixels. The cost is a slight uniform darkening, which the value floor can see."""
    if not reference_rgb or not med:
        return None
    g = [float(reference_rgb[c]) / max(med[c], 1e-4) for c in range(3)]
    mx = max(g)
    if not (mx > 0.0):
        return None
    return [x / mx for x in g]


def hsv_from(a):
    """Hue in degrees, saturation and value 0..1 — the one conversion, used by every rule here."""
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    mx = np.max(a, axis=-1)
    mn = np.min(a, axis=-1)
    d = mx - mn
    h = np.zeros_like(mx)
    safe = d > 1e-6
    rmax = safe & (mx == r)
    gmax = safe & (mx == g) & ~rmax
    bmax = safe & ~rmax & ~gmax
    with np.errstate(invalid="ignore", divide="ignore"):
        h[rmax] = (60.0 * ((g[rmax] - b[rmax]) / d[rmax])) % 360.0
        h[gmax] = 60.0 * ((b[gmax] - r[gmax]) / d[gmax]) + 120.0
        h[bmax] = 60.0 * ((r[bmax] - g[bmax]) / d[bmax]) + 240.0
    s = np.where(mx > 0, d / np.maximum(mx, 1e-6), 0.0)
    return h, s, mx


def in_band(h, s, v, lo, hi, smin, smax, vmin, vmax):
    if lo <= hi:
        inband = (h >= lo) & (h <= hi)
    else:
        # A band written across 0 degrees is two arcs, not an empty range.
        inband = (h >= lo) | (h <= hi)
    return inband & (s >= smin) & (s <= smax) & (v >= vmin) & (v <= vmax)


def stain_from(h, s, v, pore, classes):
    """Mineral area fractions from a stained section.

    Takes the SAME h, s, v the pore rule was read from, so where a colour correction was applied
    the minerals and the porosity describe one corrected picture. Reading the stain off the
    uncorrected copy would put the two families on different photographs of the same section.

    Pore is claimed FIRST and excluded: the epoxy filled it, so those pixels are not rock. Classes
    are then tested IN ORDER and the first match wins, so a pixel is one mineral. What matched
    nothing is reported as `unclassified` rather than being distributed over the classes - a
    section where a third of the rock matched no band has not been given a mineralogy."""
    left = ~pore
    total = float(h.size) or 1.0
    fracs = []
    for c in classes:
        b = c["band"]
        hit = left & in_band(h, s, v, float(b["hue_lo"]), float(b["hue_hi"]),
                             float(b["sat_min"]), float(b["sat_max"]),
                             float(b["val_min"]), float(b["val_max"]))
        fracs.append([c["mineral"], float(np.count_nonzero(hit)) / total])
        left = left & ~hit
    return {"fractions": fracs, "unclassified": float(np.count_nonzero(left)) / total}


def shape_stats(lab, n):
    """Per-object area, perimeter, aspect and edge flags from a label image.

    Shared by the pore phase and the grain phase on purpose: they are the same measurement of
    different objects, and two copies of the Crofton perimeter and the second-moment ellipse is
    two places for them to drift apart."""
    area = np.bincount(lab.ravel(), minlength=n + 1).astype(np.float64)

    # Crofton perimeter from directional transition counts, NOT a boundary-pixel count. A
    # staircase boundary overestimates a diagonal edge by up to sqrt(2), which biases circularity
    # systematically LOW - and systematically, so it would never look like noise.
    #   P = (pi/8) * [ (Nh + Nv) + (Nd1 + Nd2)/sqrt(2) ]
    # which returns 2*pi*R for a disc of radius R (checked against a synthetic disc).
    per = np.zeros(n + 1, dtype=np.float64)

    def add(a, b, weight):
        # BOTH sides of a transition are credited. For the pore phase one side is always 0 so this
        # is the obvious thing; for the grain phase, where two labels meet, the contact is real
        # boundary of both grains and crediting only one would halve the other's perimeter.
        t = a != b
        if not np.any(t):
            return
        for side in (a[t], b[t]):
            g = side > 0
            if np.any(g):
                per[: n + 1] += weight * np.bincount(side[g], minlength=n + 1)

    add(lab[:, :-1], lab[:, 1:], 1.0)
    add(lab[:-1, :], lab[1:, :], 1.0)
    add(lab[:-1, :-1], lab[1:, 1:], 1.0 / np.sqrt(2.0))
    add(lab[:-1, 1:], lab[1:, :-1], 1.0 / np.sqrt(2.0))
    # A pore against the image border has no transition there; it is excluded below anyway.
    per *= np.pi / 8.0

    # Second moments give the equivalent ellipse without needing the boundary at all, so the
    # aspect ratio carries none of the perimeter's discretization bias.
    ys, xs = np.nonzero(lab)
    idx = lab[ys, xs]
    xs = xs.astype(np.float64)
    ys = ys.astype(np.float64)
    sx = np.bincount(idx, weights=xs, minlength=n + 1)
    sy = np.bincount(idx, weights=ys, minlength=n + 1)
    sxx = np.bincount(idx, weights=xs * xs, minlength=n + 1)
    syy = np.bincount(idx, weights=ys * ys, minlength=n + 1)
    sxy = np.bincount(idx, weights=xs * ys, minlength=n + 1)

    edge = np.zeros(n + 1, dtype=bool)
    for band in (lab[0, :], lab[-1, :], lab[:, 0], lab[:, -1]):
        edge[np.unique(band[band > 0])] = True

    a = np.maximum(area, 1.0)
    mx = sx / a
    my = sy / a
    # +1/12 is the standard discrete correction: a pixel is a unit square, not a point mass, so
    # its own variance belongs in the second moment. Without it a small round pore reads as
    # elongated purely from the sampling.
    m20 = sxx / a - mx * mx + 1.0 / 12.0
    m02 = syy / a - my * my + 1.0 / 12.0
    m11 = sxy / a - mx * my
    tmp = np.sqrt(np.maximum((m20 - m02) ** 2 + 4.0 * m11 * m11, 0.0))
    l1 = 0.5 * (m20 + m02 + tmp)
    l2 = 0.5 * (m20 + m02 - tmp)
    aspect = np.sqrt(np.maximum(l1, 1e-12) / np.maximum(l2, 1e-12))

    with np.errstate(divide="ignore", invalid="ignore"):
        circ = 4.0 * np.pi * area / np.maximum(per, 1e-9) ** 2
    return area, aspect, circ, edge


def select(area, edge, min_px):
    """The keep mask plus the two counts, shared by both phases.

    `exists` matters for the grain phase: a watershed can leave a marker with no territory at all,
    and a label owning no pixels is neither a grain that was measured nor one that was dropped for
    being small - it is not there."""
    exists = area > 0
    small = exists & (area < float(min_px))
    small[0] = False
    n_edge = int(np.count_nonzero(edge[1:] & exists[1:]))
    n_small = int(np.count_nonzero(small[1:] & ~edge[1:]))
    keep = exists & ~edge & ~small
    keep[0] = False
    return keep, n_edge, n_small


def geometry_of(m):
    # scipy only for the labelling: connected components in pure numpy would be a Python-level
    # union-find over millions of pixels. Absent, the area fraction above still works.
    from scipy import ndimage

    # FOUR-connectivity for the pore phase. Two pores meeting at a single corner are joined by a
    # throat of zero width - that is not one pore body, and 8-connectivity would fuse them.
    lab, n = ndimage.label(m, structure=np.array([[0, 1, 0], [1, 1, 1], [0, 1, 0]]))
    if n == 0:
        return None
    area, aspect, circ, edge = shape_stats(lab, n)
    keep, n_edge, n_small = select(area, edge, header.get("min_pore_px", 20))
    return {
        "area": area[keep].tolist(),
        "aspect": aspect[keep].tolist(),
        "circ": circ[keep].tolist(),
        "n_edge": n_edge,
        "n_small": n_small,
    }


def grain_labels(m, sep_px):
    """Split the solid phase into grains at the necks between their distance-map centres.

    The grain phase is everything the pore rule did not claim. Grains touch, so a plain connected
    component returns the whole rock as one blob; the distance transform's ridges are the grain
    centres, and each solid pixel then goes to the centre nearest it.

    NOT scipy's `watershed_ift`, which was tried first and measured: on a welded pair that should
    split evenly it gave one grain 47792 pixels and the other 9, because its tie-breaking across the
    quantized cost plateaus lets whichever marker is reached first take almost everything. The
    nearest-centre partition splits the same pair 23957 / 23844. (scikit-image's proper watershed
    would do it too, at the price of a whole new dependency for one function.)

    The search is confined to ONE connected blob of solid at a time, and that is load-bearing:
    without it a pixel can be nearer a centre across open pore than its own, and the two
    disconnected pieces would then carry one label - one grain in two places, with an area and a
    shape that belong to neither.

    The honest limit: a boundary lands midway between two centres, which for convex grains of
    similar size is the neck and for a strongly elongated or embayed one is not. That is a second
    reason the grain-to-grain contact fraction is reported alongside every size."""
    from scipy import ndimage

    solid = ~m
    if not np.any(solid):
        return None, None
    dist = ndimage.distance_transform_edt(solid)
    # Markers are the local maxima of the distance map. `sep_px` is the footprint, so it sets how
    # close two centres may be before they count as one grain - the knob for over-segmentation,
    # which is what this kind of split gets wrong when it gets anything wrong.
    size = max(3, int(sep_px) | 1)
    peaks = solid & (dist >= ndimage.maximum_filter(dist, size=size, mode="constant")) & (dist > 0)
    markers, nm = ndimage.label(peaks)
    if nm == 0:
        return None, None
    lab = np.zeros(solid.shape, dtype=np.int32)
    # EIGHT-connectivity for the solid phase, the complement of the pore phase's four: two grains
    # meeting at a corner are one piece of rock even though the pores either side of them are not
    # one pore.
    blobs, _ = ndimage.label(solid, structure=np.ones((3, 3), dtype=int))
    for bi, sl in enumerate(ndimage.find_objects(blobs), start=1):
        if sl is None:
            continue
        sub = blobs[sl] == bi
        smk = np.where(sub, markers[sl], 0)
        if not smk.any():
            continue
        idx = ndimage.distance_transform_edt(smk == 0, return_distances=False, return_indices=True)
        lab[sl] = np.where(sub, smk[tuple(idx)], lab[sl])
    return lab, int(lab.max())


def grains_of(m, min_px, sep_px):
    lab, n = grain_labels(m, sep_px)
    if lab is None or not n:
        return None
    area, aspect, circ, edge = shape_stats(lab, n)

    # How much of each grain's outline is OPEN pore rather than a contact with the next grain.
    # Deliberately a ratio of two counts gathered the same way rather than two Crofton perimeters:
    # the staircase bias affects both counts alike and cancels, and this is a quality indicator,
    # not a length. Four-connected, because a corner touch is not a contact.
    open_n = np.zeros(n + 1, dtype=np.float64)
    total_n = np.zeros(n + 1, dtype=np.float64)

    def tally(a, b):
        t = a != b
        if not np.any(t):
            return
        for side, other in ((a, b), (b, a)):
            s = side[t]
            o = other[t]
            g = s > 0
            if not np.any(g):
                continue
            total_n[: n + 1] += np.bincount(s[g], minlength=n + 1)
            free = g & (o == 0)
            if np.any(free):
                open_n[: n + 1] += np.bincount(s[free], minlength=n + 1)

    tally(lab[:, :-1], lab[:, 1:])
    tally(lab[:-1, :], lab[1:, :])
    with np.errstate(divide="ignore", invalid="ignore"):
        contact = 1.0 - open_n / np.maximum(total_n, 1e-9)

    keep, n_edge, n_small = select(area, edge, min_px)
    return {
        "area": area[keep].tolist(),
        "aspect": aspect[keep].tolist(),
        "contact": contact[keep].tolist(),
        "n_edge": n_edge,
        "n_small": n_small,
    }

out = {"results": [], "preview_png": None, "plain_png": None, "preview_w": 0, "preview_h": 0}
for i, blob in enumerate(blobs):
    try:
        img = Image.open(io.BytesIO(blob))
        img.load()
    except Exception as e:
        out["results"].append({"image_id": ids[i], "error": "cannot decode: %s" % e})
        continue
    a0 = rgb_of(img)
    hr, sr, vr = hsv_from(a0)
    # The uncorrected band first, only so the matrix can be told from the pore phase. What is
    # measured is the mask below, taken after the correction.
    med = matrix_rgb(a0, mask_from(hr, sr, vr))
    gain = gain_for(med)
    # The median hue reported is of the picture AS DELIVERED, never of the corrected copy. It is
    # what says how far this photograph's light sat from the reference's, and correcting it first
    # would erase exactly that. Rock is mostly rock, so on a plate the band is reading correctly
    # the typical pixel is a grain and its hue sits OUTSIDE the pore band; when it sits inside,
    # the band is describing the scene - see `scene_dominated`.
    scene_hue = float(np.median(hr))
    if gain is None:
        h, s, v = hr, sr, vr
        shown = a0
    else:
        shown = a0 * np.asarray(gain, dtype=np.float32)
        h, s, v = hsv_from(shown)
    m = mask_from(h, s, v)
    total = int(m.size)
    hits = int(np.count_nonzero(m))
    row = {
        "image_id": ids[i],
        "pore_fraction": (hits / total) if total else 0.0,
        "pixels": total,
        "width": int(img.width),
        "scene_hue": scene_hue,
        "median_rgb": med,
    }
    if header.get("geometry"):
        # Same mask, so the fraction and the pore shapes can never describe different pictures.
        try:
            row["geom"] = geometry_of(m)
        except ImportError:
            row["error"] = "pore geometry needs scipy (pip install scipy)"
        except Exception as e:
            row["error"] = "geometry failed: %s" % e
    if header.get("stain"):
        # Same decode, same pore mask: the mineral fractions and VPORE_TS sum against each other,
        # so they have to describe one segmentation.
        try:
            row["stain"] = stain_from(h, s, v, m, header["stain"]["classes"])
        except Exception as e:
            row["error"] = "stain reading failed: %s" % e
    if header.get("grains"):
        # Same mask again: the grain phase is defined as what the pore rule did not claim, so the
        # porosity and the grains describe one segmentation rather than two.
        try:
            row["grain"] = grains_of(m, header.get("min_grain_px", 50), header.get("grain_sep_px", 20))
        except ImportError:
            row["error"] = "grain sizing needs scipy (pip install scipy)"
        except Exception as e:
            row["error"] = "grain sizing failed: %s" % e
    out["results"].append(row)
    if preview is not None and ids[i] == preview:
        # The overlay is drawn from the SAME mask that produced the number above, over the SAME
        # corrected picture the band was applied to. What the user tunes against is literally what
        # was measured - a preview of the delivered colours under a band applied to corrected ones
        # would be a picture nothing in the run ever looked at.
        rgb = (np.clip(shown, 0.0, 1.0) * 255.0).astype(np.uint8)
        # The SAME picture without the mask on it, sent alongside. Two jobs, and both need the
        # corrected pixels rather than the delivered ones: holding to compare shows what the band
        # claimed against what is actually there, and clicking a pore to set the band has to read
        # the colour the band will be applied to. Thumbnailed identically so a click maps across.
        plain = Image.fromarray(rgb.copy())
        plain.thumbnail((900, 900))
        pbuf = io.BytesIO()
        plain.save(pbuf, format="PNG")
        out["plain_png"] = base64.b64encode(pbuf.getvalue()).decode("ascii")
        rgb[m] = (0.35 * rgb[m] + 0.65 * np.array([255, 40, 40], dtype=np.float32)).astype(np.uint8)
        if header.get("grains"):
            # Grain outlines on the same picture. Over-segmentation is what the separation knob is
            # for and it cannot be judged from a number - a section chopped into fifty slivers and
            # one sensibly split into twelve grains produce equally plausible tables.
            try:
                glab, gn = grain_labels(m, header.get("grain_sep_px", 20))
                if glab is not None:
                    b = np.zeros(glab.shape, dtype=bool)
                    b[:, :-1] |= (glab[:, :-1] != glab[:, 1:]) & (glab[:, :-1] > 0) & (glab[:, 1:] > 0)
                    b[:-1, :] |= (glab[:-1, :] != glab[1:, :]) & (glab[:-1, :] > 0) & (glab[1:, :] > 0)
                    rgb[b] = np.array([255, 230, 60], dtype=np.uint8)
            except Exception:
                pass
        small = Image.fromarray(rgb)
        small.thumbnail((900, 900))
        buf = io.BytesIO()
        small.save(buf, format="PNG")
        out["preview_png"] = base64.b64encode(buf.getvalue()).decode("ascii")
        out["preview_w"] = small.width
        out["preview_h"] = small.height

sys.stdout.write(json.dumps(out))
"#;

// ---------------------------------------------------------------------------
// A3 — the trained classifier
// ---------------------------------------------------------------------------

/// One point the user clicked, and what they called it.
///
/// Position is a FRACTION of the picture, never a pixel: the stored copy is resampled to a long-edge
/// cap, so a pixel coordinate belongs to whichever copy it was taken on and nothing in the number
/// says which. The same argument that made a field of view the right thing to store.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlateLabel {
    pub image_id: String,
    pub x: f32,
    pub y: f32,
    pub mineral: String,
}

/// One training-and-apply run over a delivery.
#[derive(Debug, Clone, Deserialize)]
pub struct ClassifySpec {
    pub well_id: String,
    pub dataset: String,
    /// The user's own point counts. There is no shipped model and there never will be — see the
    /// module note.
    pub labels: Vec<PlateLabel>,
    /// Half-width in pixels of the patch taken around each click. A click is one observation; its
    /// immediate neighbourhood gives the fit some support without pretending a region was labelled.
    #[serde(default = "default_patch_px")]
    pub patch_px: u32,
    #[serde(default)]
    pub set_name: Option<String>,
    #[serde(default)]
    pub preview_image_id: Option<String>,
}

fn default_patch_px() -> u32 {
    PATCH_PX
}

/// Round, and in pixels for the `min_pore_px` reason.
pub const PATCH_PX: u32 = 2;

/// The fewest clicks a class needs before it can be cross-validated at all.
pub const MIN_CLICKS_PER_CLASS: usize = 3;

/// How one class did in the held-out check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassPerf {
    pub mineral: String,
    /// Fraction of held-out clicks of this mineral the model got right. **A class with a low recall
    /// has a fraction made of noise**, and it is per class rather than only overall because one
    /// unseparable pair drags nothing else down with it.
    pub recall: f32,
    pub clicks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlateClasses {
    pub image_id: String,
    pub name: String,
    pub depth_top: f32,
    pub depth_base: Option<f32>,
    pub fractions: Vec<(String, f32)>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ClassifyResult {
    pub plates: Vec<PlateClasses>,
    /// Overall held-out accuracy, cross-validated BY CLICK.
    pub accuracy: f32,
    pub per_class: Vec<ClassPerf>,
    pub skipped: Vec<String>,
    pub preview_png: Option<String>,
    pub preview_width: i32,
    pub preview_height: i32,
    pub written: Option<(String, String)>,
    pub notes: Vec<String>,
}

/// The point-data item prefix for a classified fraction.
///
/// **Deliberately not `MIN_`**, which the stain rule uses. A fraction a colour rule produced from a
/// published stain identification and one a classifier produced from this user's clicks are
/// different claims with different provenance, and a single name would make a report unable to say
/// which it quoted — the same argument that keeps `GRAIN_D50_APP` apart from `GRAIN_D50_W`.
pub const CLASS_PREFIX: &str = "CLS_";

const CLASSIFY_RUNNER: &str = r#"
import sys, json, io, base64
try:
    import numpy as np
    from PIL import Image
except Exception as e:
    sys.stderr.write("needs numpy and Pillow: %s\n" % e)
    sys.exit(1)
try:
    from scipy import ndimage
    from sklearn.ensemble import RandomForestClassifier
    from sklearn.model_selection import GroupKFold
except Exception as e:
    sys.stderr.write("needs scipy and scikit-learn: %s\n" % e)
    sys.exit(1)

header = json.loads(sys.stdin.buffer.readline().decode("utf-8"))
ids = header["ids"]
sizes = header["sizes"]
patch = int(header.get("patch_px", 2))
preview = header.get("preview")
# Cap the pixels a fraction is estimated from. A systematic sample of a few hundred thousand pixels
# has a standard error far below any real uncertainty here, and the count is reported rather than
# being a silent truncation.
MAX_PIXELS = 400000

blobs = [sys.stdin.buffer.read(n) for n in sizes]


def features(img):
    a = np.asarray(img.convert("RGB"), dtype=np.float32) / 255.0
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    mx = np.max(a, axis=-1)
    mn = np.min(a, axis=-1)
    d = mx - mn
    h = np.zeros_like(mx)
    safe = d > 1e-6
    rmax = safe & (mx == r)
    gmax = safe & (mx == g) & ~rmax
    bmax = safe & ~rmax & ~gmax
    with np.errstate(invalid="ignore", divide="ignore"):
        h[rmax] = (60.0 * ((g[rmax] - b[rmax]) / d[rmax])) % 360.0
        h[gmax] = 60.0 * ((b[gmax] - r[gmax]) / d[gmax]) + 120.0
        h[bmax] = 60.0 * ((r[bmax] - g[bmax]) / d[bmax]) + 240.0
    s = np.where(mx > 0, d / np.maximum(mx, 1e-6), 0.0)
    # Local mean and spread of brightness: TEXTURE, and the only reason this can attempt a pair
    # that colour alone cannot separate. Cloudy altered feldspar is rougher than clear quartz at
    # the same colour; twinning and cleavage show here too.
    mean5 = ndimage.uniform_filter(mx, size=5)
    sq5 = ndimage.uniform_filter(mx * mx, size=5)
    std5 = np.sqrt(np.maximum(sq5 - mean5 * mean5, 0.0))
    # Hue is circular, so it enters as its sine and cosine - 359 and 1 degrees are neighbours, and
    # a raw hue would put them at opposite ends of the feature.
    hr = np.deg2rad(h)
    return np.stack([r, g, b, np.cos(hr), np.sin(hr), s, mx, mean5, std5], axis=-1)


feats = {}
shapes = {}
for i, blob in enumerate(blobs):
    try:
        img = Image.open(io.BytesIO(blob))
        img.load()
    except Exception as e:
        feats[ids[i]] = None
        shapes[ids[i]] = str(e)
        continue
    feats[ids[i]] = features(img)
    shapes[ids[i]] = None

X, y, groups = [], [], []
for gi, lab in enumerate(header["labels"]):
    f = feats.get(lab["image_id"])
    if f is None:
        continue
    hgt, wid = f.shape[0], f.shape[1]
    cx = int(round(float(lab["x"]) * (wid - 1)))
    cy = int(round(float(lab["y"]) * (hgt - 1)))
    x0, x1 = max(0, cx - patch), min(wid, cx + patch + 1)
    y0, y1 = max(0, cy - patch), min(hgt, cy + patch + 1)
    block = f[y0:y1, x0:x1].reshape(-1, f.shape[-1])
    if not block.size:
        continue
    X.append(block)
    y.extend([lab["mineral"]] * block.shape[0])
    # GROUP is the click, not the pixel. Neighbouring pixels of one click are near-identical, so
    # splitting them across the fold boundary would score the model on data it had already seen and
    # report an accuracy nobody can reproduce on a new plate.
    groups.extend([gi] * block.shape[0])

out = {"plates": [], "accuracy": None, "per_class": [], "notes": [], "preview_png": None,
       "preview_w": 0, "preview_h": 0, "sampled": 0}

if not X:
    sys.stdout.write(json.dumps({"error": "no usable labels"}))
    sys.exit(0)

X = np.vstack(X)
y = np.array(y)
groups = np.array(groups)
classes = sorted(set(y.tolist()))

clicks_per = {c: len(set(groups[y == c].tolist())) for c in classes}
n_groups = len(set(groups.tolist()))
folds = min(5, min(clicks_per.values()))
if folds >= 2 and n_groups > folds:
    hit = {c: [0, 0] for c in classes}
    for tr, te in GroupKFold(n_splits=folds).split(X, y, groups):
        m = RandomForestClassifier(n_estimators=120, random_state=0, n_jobs=1)
        m.fit(X[tr], y[tr])
        p = m.predict(X[te])
        for c in classes:
            sel = y[te] == c
            hit[c][0] += int(np.count_nonzero(p[sel] == c))
            hit[c][1] += int(np.count_nonzero(sel))
    tot = sum(v[1] for v in hit.values())
    out["accuracy"] = (sum(v[0] for v in hit.values()) / tot) if tot else None
    out["per_class"] = [
        {"mineral": c, "recall": (hit[c][0] / hit[c][1]) if hit[c][1] else 0.0,
         "clicks": clicks_per[c]}
        for c in classes
    ]
else:
    out["notes"].append("too few clicks per mineral to cross-validate - accuracy not reported")
    out["per_class"] = [{"mineral": c, "recall": -1.0, "clicks": clicks_per[c]} for c in classes]

model = RandomForestClassifier(n_estimators=200, random_state=0, n_jobs=1)
model.fit(X, y)

for i, iid in enumerate(ids):
    f = feats.get(iid)
    if f is None:
        out["plates"].append({"image_id": iid, "error": shapes.get(iid) or "cannot decode"})
        continue
    flat = f.reshape(-1, f.shape[-1])
    step = max(1, int(np.ceil(flat.shape[0] / MAX_PIXELS)))
    sample = flat[::step]
    pred = model.predict(sample)
    total = float(sample.shape[0]) or 1.0
    out["sampled"] = int(sample.shape[0])
    out["plates"].append({
        "image_id": iid,
        "fractions": [[c, float(np.count_nonzero(pred == c)) / total] for c in classes],
    })
    if preview is not None and iid == preview:
        full = model.predict(flat).reshape(f.shape[0], f.shape[1])
        # One fixed hue per class so the map reads the same between runs.
        rgb = np.zeros(f.shape[:2] + (3,), dtype=np.uint8)
        for ci, c in enumerate(classes):
            hue = (ci * 360.0 / max(1, len(classes)))
            k = np.deg2rad(hue)
            col = np.array([
                128 + 110 * np.cos(k),
                128 + 110 * np.cos(k - 2.094),
                128 + 110 * np.cos(k + 2.094),
            ], dtype=np.float32)
            rgb[full == c] = np.clip(col, 0, 255).astype(np.uint8)
        small = Image.fromarray(rgb)
        small.thumbnail((900, 900))
        buf = io.BytesIO()
        small.save(buf, format="PNG")
        out["preview_png"] = base64.b64encode(buf.getvalue()).decode("ascii")
        out["preview_w"] = small.width
        out["preview_h"] = small.height

sys.stdout.write(json.dumps(out))
"#;

const CLASSIFY_SUPPORT_RUNNER: &str = r#"
import sys
ok = True
try:
    import numpy  # noqa: F401
    from PIL import Image  # noqa: F401
    import scipy  # noqa: F401
    import sklearn  # noqa: F401
except Exception:
    ok = False
sys.stdout.write("1" if ok else "0")
"#;

const SUPPORT_RUNNER: &str = r#"
import sys
ok = True
try:
    import numpy  # noqa: F401
    from PIL import Image  # noqa: F401
except Exception:
    ok = False
sys.stdout.write("1" if ok else "0")
"#;

/// Can the pore measurement run at all? Probed once so a dialog can say what is missing and name
/// the interpreter to install into, rather than failing at the end of a long run.
pub fn pore_support() -> Result<bool, String> {
    let python = find_python().ok_or("no Python interpreter found")?;
    let mut cmd = Command::new(&python);
    cmd.args(["-c", SUPPORT_RUNNER]).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);
    let out = cmd.output().map_err(|e| format!("failed to start python: {e}"))?;
    Ok(String::from_utf8_lossy(&out.stdout).trim() == "1")
}

#[derive(Deserialize)]
struct RunnerOut {
    results: Vec<RunnerRow>,
    preview_png: Option<String>,
    #[serde(default)]
    plain_png: Option<String>,
    #[serde(default)]
    preview_w: i32,
    #[serde(default)]
    preview_h: i32,
}

#[derive(Deserialize)]
struct RunnerRow {
    image_id: String,
    #[serde(default)]
    pore_fraction: Option<f32>,
    #[serde(default)]
    pixels: Option<i64>,
    #[serde(default)]
    width: Option<i32>,
    /// The plate's own median hue. `#[serde(default)]` like its siblings, so a row from an older
    /// runner still deserializes — it simply cannot be checked.
    #[serde(default)]
    scene_hue: Option<f32>,
    /// The plate's matrix colour, channel by channel. Only the reference plate's is used, and only
    /// to build the gain the rest of the delivery is corrected by.
    #[serde(default)]
    median_rgb: Option<[f32; 3]>,
    #[serde(default)]
    geom: Option<RunnerGeom>,
    #[serde(default)]
    grain: Option<RunnerGrain>,
    #[serde(default)]
    stain: Option<PlateStainRaw>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Deserialize)]
struct PlateStainRaw {
    fractions: Vec<(String, f32)>,
    unclassified: f32,
}

/// Per-GRAIN arrays. Same discipline as `RunnerGeom`: the runner outlines, Rust does the
/// arithmetic, so Folk & Ward and the Wicksell unfolding sit under `cargo test`.
#[derive(Deserialize)]
struct RunnerGrain {
    area: Vec<f64>,
    aspect: Vec<f64>,
    /// Fraction of each grain's outline that is a contact with another grain rather than pore.
    contact: Vec<f64>,
    n_edge: usize,
    n_small: usize,
}

/// Per-PORE arrays, not summaries. The runner stays deliberately dumb (the `office.rs` rule): every
/// statistic is computed here, through `distribution.rs`, so a pore percentile and a log percentile
/// are the same operation and cannot disagree.
#[derive(Deserialize)]
struct RunnerGeom {
    area: Vec<f64>,
    aspect: Vec<f64>,
    circ: Vec<f64>,
    n_edge: usize,
    n_small: usize,
}

/// Percentile of `values` weighted by `weights`, both same length.
///
/// Kept here rather than added to `distribution.rs` on purpose: that module is source-agnostic on a
/// bare value slice, and a parallel weight vector is a different contract that only this caller
/// needs. It is a stereological summary, not a display statistic.
///
/// **Pore diameters are weighted by AREA.** Capillary pressure fills volume, and a count-weighted
/// median on a digitized section is dominated by the smallest features the picture can resolve —
/// which says more about the scan than about the rock.
fn weighted_percentile(values: &[f64], weights: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    let mut idx: Vec<usize> = (0..values.len()).collect();
    idx.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        return f64::NAN;
    }
    let target = total * (p / 100.0);
    let mut run = 0.0;
    for &i in &idx {
        run += weights[i];
        if run >= target {
            return values[i];
        }
    }
    values[*idx.last().unwrap()]
}

/// Folk & Ward (1957) inclusive graphic standard deviation, in phi units.
///
/// `σ_I = (φ84 − φ16)/4 + (φ95 − φ5)/6`, where `φ = −log2(d in mm)`. It is the standard
/// sedimentological sorting measure and the one that maps onto the verbal scale (very well sorted,
/// well sorted, …), which is why it is used rather than a plain standard deviation: it is what a
/// core description compares against.
///
/// Phi RISES as grains get finer, so `φ16` is the coarse end and the difference comes out positive.
/// Needs a scale, like every diameter here — phi is a logarithm of millimetres, not a pure number.
fn folk_ward_sorting(diam_um: &[f64], weights: &[f64]) -> f64 {
    if diam_um.len() < 2 {
        return f64::NAN;
    }
    let phi: Vec<f64> = diam_um.iter().map(|&d| -((d / 1000.0).log2())).collect();
    let p = |q: f64| weighted_percentile(&phi, weights, q);
    (p(84.0) - p(16.0)) / 4.0 + (p(95.0) - p(5.0)) / 6.0
}

/// Fraction of a sphere of diameter `d` whose random sections come out no wider than `x`.
///
/// A plane at distance `h` from the centre cuts a circle of diameter `√(d² − 4h²)`, and `h` is
/// uniform across the sphere, so `F(x) = 1 − √(d² − x²)/d`.
fn section_cdf(d: f64, x: f64) -> f64 {
    if x >= d {
        return 1.0;
    }
    if x <= 0.0 {
        return 0.0;
    }
    1.0 - ((d * d - x * x).max(0.0)).sqrt() / d
}

/// Wicksell's problem, solved by Saltykov's unfolding.
///
/// A random section through a population of spheres shows diameters that are systematically too
/// small, because a plane rarely passes near a centre. Saltykov strips that off class by class from
/// the coarse end: the largest class can only have come from spheres of that size, so its
/// population is known, and its contribution to every smaller class is then subtracted before the
/// next one is solved.
///
/// **The class-to-class probabilities are DERIVED, not transcribed.** The published coefficient
/// table is a set of numbers that can be mis-copied and would then be wrong silently — the same
/// hazard as any tabulated constant in this repo. They come instead from [`section_cdf`] plus the
/// fact that a random plane meets a sphere at a rate proportional to its diameter, which makes the
/// arithmetic checkable against a population whose answer is known.
///
/// `counts[i]` is the number of observed sections in class `i` and `upper[i]` its upper diameter
/// bound, classes ascending, class 0 starting at zero. Returns the population per class and how
/// many classes had to be clamped at zero — the inversion is ill-conditioned by nature, and a
/// clamped class is the honest signal that this plate's correction is unstable.
fn saltykov(counts: &[f64], upper: &[f64]) -> (Vec<f64>, usize) {
    let k = counts.len().min(upper.len());
    let mut nv = vec![0.0; k];
    let mut resid = counts[..k].to_vec();
    let total: f64 = counts.iter().sum();
    let noise = total.abs() * 1e-9;
    let mut clamped = 0usize;
    let lower = |i: usize| if i == 0 { 0.0 } else { upper[i - 1] };

    for j in (0..k).rev() {
        let d = upper[j];
        if !(d > 0.0) {
            continue;
        }
        // A sphere of class j lands in class j when the cut is near its centre.
        let self_p = d * (section_cdf(d, d) - section_cdf(d, lower(j)));
        if !(self_p > 0.0) {
            continue;
        }
        let mut v = resid[j] / self_p;
        if v < -noise {
            clamped += 1;
            v = 0.0;
        } else if v < 0.0 {
            v = 0.0;
        }
        nv[j] = v;
        if v > 0.0 {
            for i in 0..j {
                resid[i] -= v * d * (section_cdf(d, upper[i]) - section_cdf(d, lower(i)));
            }
        }
    }
    (nv, clamped)
}

/// Saltykov class bounds: twelve logarithmic classes ending at the largest section seen.
///
/// Class 0 reaches down to ZERO rather than stopping a decade below the maximum, so nothing
/// measured falls outside the scheme. The published version drops that tail; losing real sections
/// to a class boundary would be a silent subset, which this repo refuses everywhere else.
fn saltykov_bounds(d_max: f64) -> Vec<f64> {
    (0..SALTYKOV_CLASSES)
        .map(|i| d_max * 10f64.powf(-0.1 * (SALTYKOV_CLASSES - 1 - i) as f64))
        .collect()
}

/// The corrected size distribution: representative diameter per class and its VOLUME weight.
///
/// **The representative diameter is the class UPPER bound, because that is the diameter the
/// unfolding solved for.** [`saltykov`] builds its class-to-class probabilities from spheres of
/// diameter `upper[j]`; reporting the class midpoint instead would quote a population the
/// arithmetic never solved, and on a single-size population it would come back ~11% fine purely
/// from where the bin edges happened to fall. It is Saltykov's own convention, and its cost is
/// that every class is quoted at its coarse edge.
///
/// Volume-weighted to match the apparent statistics, which are area-weighted — and area weighting
/// on a section IS volume weighting, because the chance of a plane meeting a grain already scales
/// with its diameter and the mean cut area with its square, so the section area attributable to a
/// size class goes as `n·D³`. That is what makes apparent and corrected comparable, and it is what
/// makes either of them comparable to a sieve, which weighs.
fn unfolded_distribution(nv: &[f64], upper: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let mut diam = Vec::new();
    let mut wt = Vec::new();
    for (i, &n) in nv.iter().enumerate() {
        if !(n > 0.0) {
            continue;
        }
        let d = upper[i];
        diam.push(d);
        wt.push(n * d * d * d);
    }
    (diam, wt)
}

/// Whether a plate may be measured by a blue-epoxy rule, and why not when it may not.
///
/// Split out and public so the test suite can pin the refusal without needing Pillow: this is the
/// rule that matters most and the one that must never quietly become a default.
/// The refusal when a well has no plates in the chosen dataset.
///
/// One wording rather than one per call site - the `requireWell` argument. Two panes reach
/// this same decision, and two copies is two places for the wording to drift apart.
fn no_plates(dataset: &str) -> String {
    format!(
        "no plates in {dataset} for this well. A thin-section measurement reads pictures that \
         are already in the project - it cannot open them from disk. Import them with Import \
         Images, giving the dataset name {dataset}."
    )
}

pub fn epoxy_check(prepared: &str) -> Result<(), &'static str> {
    match prepared.trim() {
        "blue_epoxy" => Ok(()),
        "" => Err("preparation not stated - an undeclared section is not an impregnated one"),
        "plain" => Err("not impregnated - a blue rule would return a porosity anyway"),
        _ => Err("preparation is not blue-dyed epoxy - a blue rule reads only blue-dyed sections"),
    }
}

/// Whether the colour band is describing the SCENE rather than the pores.
///
/// `epoxy_check` catches the plate nobody impregnated. It cannot catch the other half of the
/// problem, which only showed up on a real delivery: a plate that IS impregnated, photographed
/// under a light the band was never tuned for, where the rule swallows the matrix and returns a
/// porosity anyway. Across 134 photomicrographs of one carbonate delivery — one laboratory, one
/// well, one report — the median hue of the picture ran from 26 to 310 degrees, and 52 plates had
/// their whole scene sitting inside the default blue band. Those plates measured up to 0.97 v/v.
///
/// **The test is the plate's own median hue, not a cap on the answer.** A cap would be arbitrary:
/// one field of view crossing a large vug genuinely can be mostly pore. But rock is mostly rock,
/// so on a plate the band is reading correctly the TYPICAL pixel is a grain and its hue falls
/// OUTSIDE the pore band. When the median pixel is pore-coloured, the band has stopped
/// discriminating. On that delivery this flagged every one of the 28 plates reading above 0.5 v/v,
/// and the highest an unflagged plate reached was 0.387 — a plausible carbonate. Pinned by
/// `a_plate_whose_own_median_hue_is_pore_coloured_is_not_measured`.
///
/// The fraction is still MEASURED and previewed, because tuning the band is exactly how the user
/// fixes this and they cannot tune against a number they are not shown. What is refused is the
/// WRITE: a 0.97 stored at a real depth would go on to plot against helium porosity, and its
/// wrongness would be silent.
pub fn scene_dominated(median_hue: f32, band: &PoreColorBand) -> bool {
    if !median_hue.is_finite() {
        return false;
    }
    let h = median_hue.rem_euclid(360.0);
    if band.hue_lo <= band.hue_hi {
        h >= band.hue_lo && h <= band.hue_hi
    } else {
        // A band written across 0 degrees is two arcs, not an empty range — same reading the
        // runner's `in_band` gives it, and the two must agree or the guard would fire on the
        // wrong plates.
        h >= band.hue_lo || h <= band.hue_hi
    }
}

/// The smallest angle between two hues, in degrees, 0..180.
///
/// Hue is a circle, so 350 and 10 are twenty degrees apart, not three hundred and forty. Getting
/// that wrong would make every warm-cast plate look like the most badly cast in the delivery.
pub fn hue_delta(a: f32, b: f32) -> f32 {
    if !a.is_finite() || !b.is_finite() {
        return f32::NAN;
    }
    let d = (a - b).rem_euclid(360.0);
    if d > 180.0 {
        360.0 - d
    } else {
        d
    }
}

/// Did the band, carried onto this plate, land where the picture has nothing?
///
/// This is the MIRROR of [`scene_dominated`], and the failure it catches is the more dangerous of
/// the two because it does not look like a failure. A plate cast AWAY from the band returns a
/// fraction near zero, and near zero is a perfectly plausible reading for a tight rock — it plots
/// against helium porosity without ever drawing attention to itself. On the delivery this was
/// found on, a green-cast plate returned 0.04% where the petrographer had counted 15%.
///
/// **It only applies on a normalized run, and that is the whole design.** Without a reference
/// plate there is no evidence that this band finds epoxy anywhere in this delivery, so an empty
/// answer could equally mean the band has never been tuned — refusing then would refuse a first
/// click. Naming a reference is the user's statement that the band works on THAT plate, and once
/// that is on the record a plate that shows nothing after being corrected onto it is either
/// nonporous or mis-corrected, and nothing in the picture separates those two. Refusing is the
/// conservative call.
///
/// **"Empty" is one resolvable pore's worth of pixels, which is the user's own `min_pore_px` and
/// not a new constant.** A band that has not claimed even a single countable pore over a whole
/// field of view has not found a pore phase; it is not a small porosity, it is no measurement.
pub fn band_missed(pore_fraction: f32, pixels: i64, min_pore_px: u32, normalized: bool) -> bool {
    if !normalized || pixels <= 0 || !pore_fraction.is_finite() {
        return false;
    }
    let claimed = pore_fraction as f64 * pixels as f64;
    claimed < min_pore_px.max(1) as f64
}

/// Turns one plate's per-pore arrays into the numbers that get stored.
fn summarise(g: &RunnerGeom, um_per_px: Option<f64>) -> PoreGeometry {
    let pct = |v: &[f64], p: f32| -> f32 {
        let mut s: Vec<f32> = v.iter().map(|&x| x as f32).collect();
        s.sort_by(f32::total_cmp);
        crate::distribution::percentile(&s, p)
    };
    // Equivalent-circle diameter: the diameter a circle of the same area would have. Reported
    // only in micrometres, and only when the plate carries a scale.
    let (d10, d50, d90) = match um_per_px {
        Some(k) => {
            let diam: Vec<f64> =
                g.area.iter().map(|&a| 2.0 * (a / std::f64::consts::PI).sqrt() * k).collect();
            (
                Some(weighted_percentile(&diam, &g.area, 10.0) as f32),
                Some(weighted_percentile(&diam, &g.area, 50.0) as f32),
                Some(weighted_percentile(&diam, &g.area, 90.0) as f32),
            )
        }
        None => (None, None, None),
    };
    PoreGeometry {
        n: g.area.len(),
        n_edge: g.n_edge,
        n_small: g.n_small,
        aspect_p50: pct(&g.aspect, 50.0),
        aspect_p90: pct(&g.aspect, 90.0),
        shape_p50: pct(&g.circ, 50.0),
        d10_um: d10,
        d50_um: d50,
        d90_um: d90,
    }
}

/// Can the classifier run? Needs scikit-learn as well as scipy, so it is probed separately.
pub fn classify_support() -> Result<bool, String> {
    let python = find_python().ok_or("no Python interpreter found")?;
    let mut cmd = Command::new(&python);
    cmd.args(["-c", CLASSIFY_SUPPORT_RUNNER]).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);
    let out = cmd.output().map_err(|e| format!("failed to start python: {e}"))?;
    Ok(String::from_utf8_lossy(&out.stdout).trim() == "1")
}

#[derive(Deserialize)]
struct ClassifyOut {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    plates: Vec<ClassifyRow>,
    #[serde(default)]
    accuracy: Option<f32>,
    #[serde(default)]
    per_class: Vec<ClassPerf>,
    #[serde(default)]
    notes: Vec<String>,
    #[serde(default)]
    preview_png: Option<String>,
    #[serde(default)]
    preview_w: i32,
    #[serde(default)]
    preview_h: i32,
    #[serde(default)]
    sampled: i64,
}

#[derive(Deserialize)]
struct ClassifyRow {
    image_id: String,
    #[serde(default)]
    fractions: Vec<(String, f32)>,
    #[serde(default)]
    error: Option<String>,
}

/// Trains a per-pixel classifier on the user's own clicks and applies it to the delivery.
///
/// **There is no shipped model and there will not be one.** Quartz against feldspar in plane light
/// is not a colour problem, and a model trained on somebody else's sections under somebody else's
/// lamp would produce numbers with the shape of a modal analysis and none of the content
/// (`docs/plan_image_analysis.md` §2.1 A3). The labels are this user's, on these plates.
pub fn run_plate_classifier(
    conn: &Connection,
    spec: &ClassifySpec,
) -> Result<ClassifyResult, String> {
    let python = find_python().ok_or("no Python interpreter found (see SANDIBUMI_PYTHON)")?;
    let all = crate::db::list_well_images(conn, &spec.well_id, Some(&spec.dataset))
        .map_err(|e| e.to_string())?;
    if all.is_empty() {
        return Err(no_plates(&spec.dataset));
    }

    // Enough clicks per mineral to hold some out, or the accuracy is a number about nothing.
    let mut per: Vec<(String, usize)> = Vec::new();
    for l in &spec.labels {
        match per.iter_mut().find(|(m, _)| *m == l.mineral) {
            Some((_, n)) => *n += 1,
            None => per.push((l.mineral.clone(), 1)),
        }
    }
    if per.len() < 2 {
        return Err("the classifier needs at least two minerals labelled, and this run has \
                    one. A classifier with a single class has nothing to decide, so its 100% \
                    is a number about nothing. Click examples of a second mineral, then train."
            .into());
    }
    if let Some((m, n)) = per.iter().find(|(_, n)| *n < MIN_CLICKS_PER_CLASS) {
        return Err(format!(
            "'{m}' has {n} click(s); every mineral needs at least {MIN_CLICKS_PER_CLASS} before \
             the model can be checked on clicks it has not seen"
        ));
    }

    let mut result = ClassifyResult::default();
    let mut blobs = Vec::with_capacity(all.len());
    for info in &all {
        let (_, bytes) = crate::db::get_well_image(conn, &info.image_id).map_err(|e| e.to_string())?;
        blobs.push(bytes);
    }
    let header = serde_json::json!({
        "ids": all.iter().map(|i| i.image_id.clone()).collect::<Vec<_>>(),
        "sizes": blobs.iter().map(|b| b.len()).collect::<Vec<_>>(),
        "labels": spec.labels,
        "patch_px": spec.patch_px.min(8),
        "preview": spec.preview_image_id,
    });

    let mut cmd = Command::new(&python);
    cmd.args(["-c", CLASSIFY_RUNNER])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_console(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("failed to start python: {e}"))?;
    {
        let stdin = child.stdin.as_mut().ok_or("failed to open python stdin")?;
        stdin.write_all(header.to_string().as_bytes()).map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        for b in &blobs {
            stdin.write_all(b).map_err(|e| e.to_string())?;
        }
    }
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("classifier failed");
        return Err(last.trim().to_string());
    }
    let parsed: ClassifyOut =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("bad classifier result: {e}"))?;
    if let Some(e) = parsed.error {
        return Err(e);
    }

    result.accuracy = parsed.accuracy.unwrap_or(f32::NAN);
    result.per_class = parsed.per_class;
    result.notes = parsed.notes;
    result.preview_png = parsed.preview_png;
    result.preview_width = parsed.preview_w;
    result.preview_height = parsed.preview_h;

    for row in parsed.plates {
        let Some(info) = all.iter().find(|i| i.image_id == row.image_id) else { continue };
        match row.error {
            Some(e) => result.skipped.push(format!("{}: {}", info.name, e)),
            None => result.plates.push(PlateClasses {
                image_id: info.image_id.clone(),
                name: info.name.clone(),
                depth_top: info.depth_top,
                depth_base: info.depth_base,
                fractions: row.fractions,
            }),
        }
    }
    result.plates.sort_by(|a, b| a.depth_top.total_cmp(&b.depth_top));

    if let Some(set) = &spec.set_name {
        let mut rows: Vec<crate::db::AuxRow> = Vec::new();
        for p in &result.plates {
            for (mineral, v) in &p.fractions {
                rows.push(crate::db::AuxRow {
                    dataset: PORE_DATASET.to_string(),
                    depth_top: p.depth_top,
                    depth_base: p.depth_base,
                    item: format!("{CLASS_PREFIX}{}", mineral_item(mineral)),
                    value_num: Some(*v),
                    value_text: None,
                });
            }
        }
        let name = crate::db::resolve_aux_set_name(conn, &spec.well_id, PORE_DATASET, set)
            .map_err(|e| e.to_string())?;
        crate::db::insert_aux_data(
            conn,
            &spec.well_id,
            PORE_DATASET,
            &name,
            Some(&spec.dataset),
            &rows,
        )
        .map_err(|e| e.to_string())?;
        result.written = Some((PORE_DATASET.to_string(), name));
    }

    // The weak classes named, not just an overall number. One unseparable pair drags nothing else
    // down with it, so an overall 0.9 can sit on top of a mineral the model cannot see at all.
    let weak: Vec<&str> = result
        .per_class
        .iter()
        .filter(|c| c.recall >= 0.0 && c.recall < 0.7)
        .map(|c| c.mineral.as_str())
        .collect();
    if !weak.is_empty() {
        result.notes.push(format!(
            "The model cannot reliably tell {} apart from the rest. Those fractions are noise \
             until there are more clicks on them, or until the pair is one class.",
            weak.join(", ")
        ));
    }
    if parsed.sampled > 0 {
        result.notes.push(format!(
            "Each fraction is from {} pixels sampled evenly across the plate.",
            parsed.sampled
        ));
    }
    result.notes.push(
        "Trained on your clicks on these plates. The lamp, the white balance and the scanner are \
         part of what it learned, so it is not a model for a differently photographed delivery."
            .to_string(),
    );
    Ok(result)
}

/// A mineral name as a point-data item suffix: upper case, non-alphanumerics to underscore.
/// "Ferroan calcite" becomes `MIN_FERROAN_CALCITE`.
fn mineral_item(mineral: &str) -> String {
    let s: String = mineral
        .trim()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() } else { '_' })
        .collect();
    // Collapse runs and trim, so "Ferroan  calcite " is not FERROAN__CALCITE_.
    let mut out = String::new();
    for c in s.chars() {
        if c == '_' && out.ends_with('_') {
            continue;
        }
        out.push(c);
    }
    out.trim_matches('_').to_string()
}

/// Turns one plate's per-grain arrays into the numbers that get stored.
///
/// Apparent always; corrected only when it was asked for, and under its own names.
fn summarise_grains(g: &RunnerGrain, um_per_px: Option<f64>, wicksell: bool) -> GrainStats {
    let pct = |v: &[f64], p: f32| -> f32 {
        let mut s: Vec<f32> = v.iter().map(|&x| x as f32).collect();
        s.sort_by(f32::total_cmp);
        crate::distribution::percentile(&s, p)
    };
    let mut out = GrainStats {
        n: g.area.len(),
        n_edge: g.n_edge,
        n_small: g.n_small,
        aspect_p50: pct(&g.aspect, 50.0),
        contact_p50: pct(&g.contact, 50.0),
        d10_app_um: None,
        d50_app_um: None,
        d90_app_um: None,
        sort_app_phi: None,
        d10_w_um: None,
        d50_w_um: None,
        d90_w_um: None,
        sort_w_phi: None,
        w_clamped: 0,
    };
    let Some(k) = um_per_px else { return out };
    if g.area.is_empty() {
        return out;
    }
    let diam: Vec<f64> =
        g.area.iter().map(|&a| 2.0 * (a / std::f64::consts::PI).sqrt() * k).collect();
    out.d10_app_um = Some(weighted_percentile(&diam, &g.area, 10.0) as f32);
    out.d50_app_um = Some(weighted_percentile(&diam, &g.area, 50.0) as f32);
    out.d90_app_um = Some(weighted_percentile(&diam, &g.area, 90.0) as f32);
    let s = folk_ward_sorting(&diam, &g.area);
    if s.is_finite() {
        out.sort_app_phi = Some(s as f32);
    }

    if wicksell {
        let d_max = diam.iter().cloned().fold(0.0f64, f64::max);
        if d_max > 0.0 {
            let upper = saltykov_bounds(d_max);
            let mut counts = vec![0.0f64; upper.len()];
            for &d in &diam {
                // Counts, NOT areas: Saltykov unfolds a population of objects. The volume
                // weighting is applied afterwards, to the unfolded classes.
                let i = upper.iter().position(|&u| d <= u).unwrap_or(upper.len() - 1);
                counts[i] += 1.0;
            }
            let (nv, clamped) = saltykov(&counts, &upper);
            out.w_clamped = clamped;
            let (wd, ww) = unfolded_distribution(&nv, &upper);
            if !wd.is_empty() {
                out.d10_w_um = Some(weighted_percentile(&wd, &ww, 10.0) as f32);
                out.d50_w_um = Some(weighted_percentile(&wd, &ww, 50.0) as f32);
                out.d90_w_um = Some(weighted_percentile(&wd, &ww, 90.0) as f32);
                let sw = folk_ward_sorting(&wd, &ww);
                if sw.is_finite() {
                    out.sort_w_phi = Some(sw as f32);
                }
            }
        }
    }
    out
}

/// One subprocess: some plates in, their measurements out.
///
/// Factored out because the run makes two passes — one to harvest each reference plate's own matrix
/// colour, one to measure every plate corrected onto the reference its depth assigns it — and two
/// copies of the pipe protocol is two places for the header to drift.
fn run_batch(
    conn: &Connection,
    python: &std::path::Path,
    spec: &PoreSpec,
    batch: &[crate::db::ImageInfo],
    reference_rgb: Option<[f32; 3]>,
    preview: bool,
) -> Result<RunnerOut, String> {
    let mut blobs = Vec::with_capacity(batch.len());
    for info in batch {
        let (_, bytes) =
            crate::db::get_well_image(conn, &info.image_id).map_err(|e| e.to_string())?;
        blobs.push(bytes);
    }
    let header = serde_json::json!({
        "band": spec.band,
        "geometry": spec.geometry,
        "min_pore_px": spec.min_pore_px.max(1),
        "grains": spec.grains,
        "min_grain_px": spec.min_grain_px.max(1),
        "grain_sep_px": spec.grain_sep_px.max(3),
        "stain": spec.stain,
        "reference_rgb": reference_rgb,
        "ids": batch.iter().map(|i| i.image_id.clone()).collect::<Vec<_>>(),
        "sizes": blobs.iter().map(|b| b.len()).collect::<Vec<_>>(),
        // The colour-harvest pass draws nothing. What the user tunes against has to be the
        // CORRECTED picture the stored number was taken from, and that is the second pass.
        "preview": if preview { spec.preview_image_id.clone() } else { None },
    });

    let mut cmd = Command::new(python);
    cmd.args(["-c", PORE_RUNNER]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("failed to start python: {e}"))?;
    {
        let stdin = child.stdin.as_mut().ok_or("failed to open python stdin")?;
        stdin.write_all(header.to_string().as_bytes()).map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        for b in &blobs {
            stdin.write_all(b).map_err(|e| e.to_string())?;
        }
    }
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("pore run failed");
        return Err(last.trim().to_string());
    }
    serde_json::from_slice(&output.stdout).map_err(|e| format!("bad pore result: {e}"))
}

/// Measures pore area on a well's live image delivery.
pub fn run_pore_area(conn: &Connection, spec: &PoreSpec) -> Result<PoreResult, String> {
    // Before a single picture is decoded: an overlap has no right answer, and finding that out after
    // a 250-plate run would waste the run.
    check_zones(spec)?;
    let python = find_python().ok_or("no Python interpreter found (see SANDIBUMI_PYTHON)")?;

    let all = crate::db::list_well_images(conn, &spec.well_id, Some(&spec.dataset))
        .map_err(|e| e.to_string())?;
    if all.is_empty() {
        return Err(no_plates(&spec.dataset));
    }

    let mut skipped: Vec<String> = Vec::new();
    let mut wanted = Vec::new();
    for info in &all {
        // The refusal is BY NAME and counted. A silent subset reads as a complete answer, which is
        // exactly how a half-measured delivery would end up in a report.
        if let Err(why) = epoxy_check(&info.prepared) {
            skipped.push(format!("{}: {}", info.name, why));
            continue;
        }
        // A stain scheme applied to the wrong stain returns mineral fractions that are wrong and
        // entirely plausible, so the plate's OWN declared stain has to agree. Undeclared is
        // refused for the same reason `prepared` is: it cannot be read off the pixels, because the
        // evidence for "this is alizarin red" is the red about to be measured.
        if let Some(st) = &spec.stain {
            let want = normalize_stain(&st.stain);
            let have = normalize_stain(&info.stain);
            if have.is_empty() {
                skipped.push(format!(
                    "{}: stain not stated - set it in Plate Details, it is the laboratory's fact",
                    info.name
                ));
                continue;
            }
            if have != want {
                skipped.push(format!(
                    "{}: stained '{}', but the scheme is for '{}'",
                    info.name,
                    info.stain,
                    st.stain
                ));
                continue;
            }
        }
        wanted.push(info.clone());
    }
    if wanted.is_empty() {
        return Err(format!(
            "no plate in {} is declared as blue-dyed epoxy. Set the impregnation in Plate Details - \
             it is a fact about the section, not something the picture can be asked.",
            spec.dataset
        ));
    }

    let mut plates: Vec<PlatePore> = Vec::new();
    let mut preview_png = None;
    let mut plain_png = None;
    let mut preview_width = 0;
    let mut preview_height = 0;

    // A run is normalized if ANY reference was named — a delivery-wide one, an interval one, or
    // both. It stays a run-level fact rather than a per-plate one because of the refusal below: a
    // plate no reference reaches is left out entirely, so no run ever mixes the two.
    let normalized = spec.reference_image_id.is_some() || !spec.reference_zones.is_empty();

    // Which plates this run will actually measure. A tuning preview measures one; a full run
    // measures everything the delivery declared.
    let targets: Vec<crate::db::ImageInfo> = match &spec.only_image_id {
        Some(only) => wanted.iter().filter(|i| &i.image_id == only).cloned().collect(),
        None => wanted.clone(),
    };

    // Each plate to the reference its DEPTH assigns it, falling back to the delivery-wide one.
    let mut assigned: Vec<(crate::db::ImageInfo, Option<String>)> = Vec::new();
    for info in targets {
        match reference_for(spec, info.depth_top) {
            Some(id) => assigned.push((info, Some(id.to_string()))),
            // Refused by name, never read as delivered. One stored set holding both corrected and
            // uncorrected fractions would be two measurements under one name, and `band_missed`
            // would quietly switch off for the uncorrected half of them.
            None if normalized => skipped.push(format!(
                "{}: not measured - no reference plate covers {}. Widen an interval to reach it, or \
                 name a reference plate for the rest of the delivery.",
                info.name, info.depth_top
            )),
            None => assigned.push((info, None)),
        }
    }

    // ---- pass 1: what colour is each reference plate's matrix ----------------
    //
    // Before any other plate is decoded, because every section in an interval is corrected onto its
    // reference. Measured UNCORRECTED, which is exactly what a reference is: correcting a plate onto
    // itself is the identity, so this needs no special case in the runner.
    let mut needed: Vec<crate::db::ImageInfo> = Vec::new();
    for (_, r) in &assigned {
        let Some(id) = r else { continue };
        if needed.iter().any(|i| &i.image_id == id) {
            continue;
        }
        let Some(found) = wanted.iter().find(|i| &i.image_id == id).cloned() else {
            let name =
                all.iter().find(|i| &i.image_id == id).map_or(id.clone(), |i| i.name.clone());
            return Err(format!(
                "{name} cannot be a reference plate: it is not among the plates this run can \
                 measure, so its impregnation or its stain is undeclared. Set it in Plate Details."
            ));
        };
        needed.push(found);
    }

    let mut anchors: HashMap<String, ([f32; 3], f32)> = HashMap::new();
    for batch in needed.chunks(CHUNK) {
        let parsed = run_batch(conn, &python, spec, batch, None, false)?;
        for row in parsed.results {
            let Some(info) = batch.iter().find(|i| i.image_id == row.image_id) else { continue };
            if let Some(e) = row.error {
                return Err(format!("{} cannot be a reference plate: {}", info.name, e));
            }
            let hue = row.scene_hue.unwrap_or(f32::NAN);
            // Everything in this interval is corrected onto this plate, so a reference that is
            // itself mostly the colour called pore anchors the interval to the mistake — and
            // silently, because every corrected plate then inherits its median hue and the per-plate
            // test would only agree with itself.
            if scene_dominated(hue, &spec.band) {
                return Err(format!(
                    "{} cannot be a reference plate: its own median hue ({:.0} deg) is inside the \
                     pore band, so on this plate the band is matching the background. Tune the band \
                     here first, or choose a plate the band reads correctly.",
                    info.name, hue
                ));
            }
            let Some(rgb) = row.median_rgb else {
                return Err(format!(
                    "{} gave no matrix colour, so there is nothing to correct the interval \
                     onto. The plate read as empty under the current band, which usually \
                     means the band or the picture is wrong for this stain. Choose another \
                     reference plate, or tune the band on this one first.",
                    info.name
                ));
            };
            anchors.insert(info.image_id.clone(), (rgb, hue));
        }
    }

    // ---- pass 2: measure every plate against its own reference ---------------
    //
    // Grouped by reference, because the correction travels in the header: one batch, one anchor.
    let mut groups: Vec<(Option<String>, Vec<crate::db::ImageInfo>)> = Vec::new();
    for (info, r) in assigned {
        match groups.iter_mut().find(|(k, _)| k.as_deref() == r.as_deref()) {
            Some((_, v)) => v.push(info),
            None => groups.push((r, vec![info])),
        }
    }

    for (rid, members) in &groups {
        let anchor = rid.as_ref().and_then(|id| anchors.get(id).copied());
        let ref_name = rid
            .as_ref()
            .and_then(|id| needed.iter().find(|i| &i.image_id == id))
            .map_or(String::new(), |i| i.name.clone());
        for batch in members.chunks(CHUNK) {
            let parsed = run_batch(conn, &python, spec, batch, anchor.map(|(rgb, _)| rgb), true)?;
            if let Some(p) = parsed.preview_png {
                preview_png = Some(p);
                plain_png = parsed.plain_png;
                preview_width = parsed.preview_w;
                preview_height = parsed.preview_h;
            }
            for row in parsed.results {
                let Some(info) = batch.iter().find(|i| i.image_id == row.image_id) else { continue };
                match (row.error, row.pore_fraction) {
                    (Some(e), _) => skipped.push(format!("{}: {}", info.name, e)),
                    (None, Some(f)) => {
                        // Micrometres per pixel of THIS copy, from the plate's own field of view.
                        // `None` where no scale was declared, and then no dimensional number is
                        // reported at all — a diameter in pixels is not a diameter.
                        let px_w = row.width.unwrap_or(info.width).max(1) as f64;
                        let um_per_px = info.fov_um.map(|fov| fov as f64 / px_w);
                        let geometry = row.geom.as_ref().map(|g| summarise(g, um_per_px));
                        let grains = row
                            .grain
                            .as_ref()
                            .map(|g| summarise_grains(g, um_per_px, spec.wicksell));
                        // A plate the band cannot discriminate on is still measured and previewed —
                        // tuning the band is how the user fixes it — but it is kept out of the write.
                        let scene_hue = row.scene_hue.unwrap_or(f32::NAN);
                        // On a corrected plate the median hue is the reference's by construction, so
                        // the plain scene test would only ever restate the reference's — which was
                        // checked once, in pass 1. What replaces it is the case the correction could
                        // not be applied to AT ALL: no matrix left to anchor on, which means the
                        // band claimed the whole picture. That is precisely what the scene test is
                        // for, so the same refusal and the same message serve it, and a plate that
                        // would otherwise be read uncorrected and stored at nearly 1.0 is caught.
                        // The other half of the pair is `band_missed`.
                        let dominated = if anchor.is_some() {
                            row.median_rgb.is_none()
                        } else {
                            scene_dominated(scene_hue, &spec.band)
                        };
                        // Against THIS plate's own reference. With more than one in play, a shift
                        // measured from whichever reference happened to be last would be a number
                        // about the wrong pair of photographs.
                        let cast_shift = match anchor {
                            Some((_, rh)) => hue_delta(scene_hue, rh),
                            None => f32::NAN,
                        };
                        let missed =
                            band_missed(f, row.pixels.unwrap_or(0), spec.min_pore_px, normalized);
                        plates.push(PlatePore {
                            image_id: info.image_id.clone(),
                            name: info.name.clone(),
                            depth_top: info.depth_top,
                            depth_base: info.depth_base,
                            pore_fraction: f,
                            scene_hue,
                            scene_dominated: dominated,
                            cast_shift,
                            reference_name: ref_name.clone(),
                            band_missed: missed,
                            pixels: row.pixels.unwrap_or(0),
                            geometry,
                            grains,
                            stain: row.stain.map(|s| PlateStain {
                                fractions: s.fractions,
                                unclassified: s.unclassified,
                            }),
                        })
                    }
                    (None, None) => skipped.push(format!("{}: no result", info.name)),
                }
            }
        }
    }

    plates.sort_by(|a, b| a.depth_top.total_cmp(&b.depth_top));

    let mut notes = Vec::new();
    let mut written = None;
    if let Some(set) = &spec.set_name {
        // Point data, not a curve: a thin section measures the one plug it was cut from, and a
        // line drawn between two of them would claim rock nobody looked at.
        let mut rows: Vec<crate::db::AuxRow> = Vec::new();
        for p in &plates {
            // ONE predicate, so the plates that are stored and the plates that are scored can never
            // come apart. The two messages differ because the two failures do.
            if !storable(p) {
                skipped.push(if p.scene_dominated {
                    // Nothing from a scene-dominated plate is stored — not the fraction, not the
                    // pore shapes, not the minerals. They all come off the same mask, so if the
                    // mask is the background then every number derived from it is about the
                    // background.
                    format!(
                        "{}: not stored - the picture's own median hue ({:.0} deg) is inside the \
                         pore band, so the band is matching the background rather than the pores. \
                         Tune the band on this plate, or exclude it.",
                        p.name, p.scene_hue
                    )
                } else {
                    // The mirror case, and the more dangerous one: near zero looks like a tight
                    // rock, so nothing downstream would ever query it. Refused only because a
                    // reference plate was named — see `band_missed`.
                    format!(
                        "{}: not stored - the band claimed less than one pore's worth of this \
                         plate. Either the section is nonporous or the correction did not reach it \
                         (its light sat {:.0} deg from the reference plate's), and the picture \
                         cannot say which. Tune a band on this plate, or make it the reference.",
                        p.name, p.cast_shift
                    )
                });
                continue;
            }
            let mut put = |item: &str, v: f32| {
                rows.push(crate::db::AuxRow {
                    dataset: PORE_DATASET.to_string(),
                    depth_top: p.depth_top,
                    depth_base: p.depth_base,
                    item: item.to_string(),
                    value_num: Some(v),
                    value_text: None,
                });
            };
            put(PORE_ITEM, p.pore_fraction);
            if let Some(g) = &p.geometry {
                put("PORE_N", g.n as f32);
                put("PORE_ASPECT", g.aspect_p50);
                put("PORE_SHAPE", g.shape_p50);
                // The dimensional three are written ONLY where the plate had a scale. Writing a
                // pixel diameter under a micrometre name would be the one failure this whole tier
                // is built to avoid, and a NaN in its place would still occupy the item.
                for (item, v) in [("PORE_D10", g.d10_um), ("PORE_D50", g.d50_um), ("PORE_D90", g.d90_um)] {
                    if let Some(v) = v {
                        put(item, v);
                    }
                }
            }
            if let Some(s) = &p.stain {
                for (mineral, v) in &s.fractions {
                    put(&format!("MIN_{}", mineral_item(mineral)), *v);
                }
                // Written every time, never only when it is large. The remainder is what says
                // whether the rows above are a mineralogy or a partial one.
                put("MIN_UNCLASS", s.unclassified);
            }
            if let Some(g) = &p.grains {
                put("GRAIN_N", g.n as f32);
                put("GRAIN_ASPECT", g.aspect_p50);
                // The honesty number rides with every grain run, never optionally: a size measured
                // on a section whose boundaries were mostly inferred is a different statement from
                // one measured on a loose sand, and nothing else in the table would say so.
                put("GRAIN_CONTACT", g.contact_p50);
                // APPARENT and CORRECTED under different names, never one name and a flag. A
                // `GRAIN_D50` that sometimes means one and sometimes the other cannot be read by
                // anything downstream, and a report quoting it has no way to say which it got.
                for (item, v) in [
                    ("GRAIN_D10_APP", g.d10_app_um),
                    ("GRAIN_D50_APP", g.d50_app_um),
                    ("GRAIN_D90_APP", g.d90_app_um),
                    ("GRAIN_SORT_APP", g.sort_app_phi),
                    ("GRAIN_D10_W", g.d10_w_um),
                    ("GRAIN_D50_W", g.d50_w_um),
                    ("GRAIN_D90_W", g.d90_w_um),
                    ("GRAIN_SORT_W", g.sort_w_phi),
                ] {
                    if let Some(v) = v {
                        put(item, v);
                    }
                }
            }
        }
        let name = crate::db::resolve_aux_set_name(conn, &spec.well_id, PORE_DATASET, set)
            .map_err(|e| e.to_string())?;
        crate::db::insert_aux_data(conn, &spec.well_id, PORE_DATASET, &name, Some(&spec.dataset), &rows)
            .map_err(|e| e.to_string())?;
        written = Some((PORE_DATASET.to_string(), name));
    }
    if !skipped.is_empty() {
        notes.push(format!("{} plate(s) left out - see the list", skipped.len()));
    }
    // Said whether or not a set was named, because it is the answer to "why is my porosity 97%"
    // and the user meets that question while tuning, long before they save anything.
    let dominated = plates.iter().filter(|p| p.scene_dominated).count();
    if dominated > 0 {
        notes.push(format!(
            "{} of {} plate(s) are mostly the colour you called pore - their own median hue falls \
             inside the band, so the rule is matching the background and the fraction is not a \
             porosity. Tune the band against one of them on the preview. A delivery photographed \
             under more than one light needs more than one band.",
            dominated,
            plates.len()
        ));
    }
    // The spread itself, because it is the thing that decides whether ONE band can serve the whole
    // delivery. On a real carbonate delivery this ran to 283 degrees across 141 plates.
    let hues: Vec<f32> = plates.iter().map(|p| p.scene_hue).filter(|h| h.is_finite()).collect();
    if hues.len() >= 2 {
        let lo = hues.iter().cloned().fold(f32::INFINITY, f32::min);
        let hi = hues.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        if hi - lo > 60.0 {
            notes.push(if normalized {
                format!(
                    "These plates were not photographed under one light - their median hue spans \
                     {:.0} degrees ({:.0} to {:.0}) - so each was corrected onto the reference \
                     plate before the band was applied. The correction is a channel rebalance, and \
                     it gets less exact the further a plate had to move: read the shift column \
                     beside each result.",
                    hi - lo, lo, hi
                )
            } else {
                format!(
                    "These plates were not photographed under one light: their median hue spans \
                     {:.0} degrees ({:.0} to {:.0}). One colour band cannot serve all of them - \
                     name a reference plate so each is corrected onto it, and give a cored interval \
                     its own reference where the light changed between them.",
                    hi - lo, lo, hi
                )
            });
        }
    }
    // Said whenever more than one reference was in play, because from the table alone there is no
    // way to tell which plate a given row was corrected onto - and that choice moves the answer more
    // than the colour band does.
    if !spec.reference_zones.is_empty() {
        let name_of = |id: &str| {
            all.iter().find(|i| i.image_id == id).map_or_else(|| id.to_string(), |i| i.name.clone())
        };
        let mut lines: Vec<String> = spec
            .reference_zones
            .iter()
            .map(|z| format!("{} on {}", name_of(&z.image_id), zone_span(z)))
            .collect();
        if let Some(id) = &spec.reference_image_id {
            lines.push(format!("{} everywhere else", name_of(id)));
        }
        notes.push(format!(
            "Corrected onto more than one reference plate: {}. Each interval's sections were \
             corrected onto its own plate, so fractions from different intervals are only as \
             comparable as those two plates are. Compare intervals on the agreement figure rather \
             than by reading their medians against each other.",
            lines.join("; ")
        ));
    }
    let missed = plates.iter().filter(|p| p.band_missed).count();
    if missed > 0 {
        notes.push(format!(
            "{} of {} plate(s) showed less than one pore's worth of the band and were not stored. \
             Near zero reads as a tight rock, which is why it is refused rather than kept: on a \
             plate corrected from a long way off, an empty answer and a missed band look the same.",
            missed,
            plates.len()
        ));
    }
    notes.push(
        "Area fraction estimates volume fraction by the Delesse relation. Where it disagrees with \
         core helium porosity the disagreement is informative: microporosity below the resolution \
         of the section, plucked grains, or epoxy that did not penetrate."
            .to_string(),
    );
    if let Some(st) = &spec.stain {
        // The collision is real and specific: under Dickson's stain ferroan dolomite goes
        // turquoise and blue-dyed epoxy is blue, so on a section that was both impregnated and
        // stained the pore rule eats the mineral. Named, never resolved automatically — which band
        // to narrow is a judgement made looking at the plate.
        let clashing: Vec<&str> = st
            .classes
            .iter()
            .filter(|c| epoxy_collides(&spec.band, &c.band))
            .map(|c| c.mineral.as_str())
            .collect();
        if !clashing.is_empty() {
            notes.push(format!(
                "The pore band overlaps the colour of {} - these plates are impregnated AND \
                 stained, so pore is claimed first and that mineral is being counted as porosity. \
                 Narrow one of the two bands on the preview before trusting either number.",
                clashing.join(", ")
            ));
        }
        let worst = plates
            .iter()
            .filter_map(|p| p.stain.as_ref())
            .map(|s| s.unclassified)
            .fold(0.0f32, f32::max);
        if worst > 0.25 {
            notes.push(format!(
                "Up to {:.0}% of the rock on a plate fell in no colour band. The mineral rows are \
                 a partial answer until that comes down - widen the bands on the preview, or say \
                 so when the numbers are quoted.",
                worst * 100.0
            ));
        }
        notes.push(
            "Mineral fractions are of the WHOLE plate, so pore + minerals + unclassified is 1."
                .to_string(),
        );
    }
    if spec.grains {
        notes.push(
            "Grain sizes are APPARENT unless the item name says _W. A random plane rarely cuts a \
             grain through its centre, so section diameters run small and section sorting runs \
             worse than the rock's."
                .to_string(),
        );
        // Not a caveat in a footnote: where the section is cemented there is nothing in the
        // picture to separate one grain from the next, and the watershed will draw a line anyway.
        let welded = plates
            .iter()
            .filter_map(|p| p.grains.as_ref())
            .filter(|g| g.contact_p50 > 0.7)
            .count();
        if welded > 0 {
            notes.push(format!(
                "{welded} plate(s) have most of their grain outline as grain-to-grain contact \
                 (GRAIN_CONTACT above 0.7). There is no pore between those grains for the picture \
                 to see, so the boundary was placed by the watershed rather than observed - read \
                 their sizes as a rock-fabric description, not a grain-size analysis."
            ));
        }
        if plates.iter().filter_map(|p| p.grains.as_ref()).any(|g| g.w_clamped > 0) {
            notes.push(
                "Some Wicksell classes unfolded to a negative population and were clamped at zero. \
                 That is the inversion being ill-conditioned, not a bug - treat the corrected \
                 numbers on those plates as indicative."
                    .to_string(),
            );
        }
    }

    // Nothing above this point can say whether the settings were any good. The preview shows what
    // the band claimed; it cannot show whether what the band claimed is the rock. This can.
    let mut agreement = None;
    if let Some(src) = &spec.check_against {
        if spec.only_image_id.is_some() {
            // A tuning preview measures ONE plate. An agreement over one plug is not a number, and
            // saying nothing beats printing a blank the user has to work out the meaning of.
        } else {
            // EXACTLY the plates that would be stored, by the same predicate the write uses. A
            // plate the run has already refused must not vote on whether the run is any good.
            let measured = storable_samples(&plates);
            let a = crate::plugqc::score_against_plugs(
                conn,
                &spec.well_id,
                &measured,
                src,
                spec.check_depth_tol,
            )?;
            if a.n_pairs > 0 && measured.len() > a.n_pairs {
                // The comparability warning, and it is not pedantic: change the reference plate and
                // a different set of plates gets refused, so two runs can be scored on different
                // rock. A coefficient that rose because the awkward plugs dropped out is not an
                // improvement, and nothing else on the screen would say so.
                notes.push(format!(
                    "The agreement is over {} of the {} plate(s) that would be stored - the rest \
                     found no plug within the depth tolerance. Comparing two settings is only fair \
                     when this count is the same for both.",
                    a.n_pairs,
                    measured.len()
                ));
            }
            agreement = Some(a);
        }
    }

    Ok(PoreResult {
        plates,
        skipped,
        preview_png,
        plain_png,
        preview_width,
        preview_height,
        written,
        agreement,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The rule the whole feature rests on. A plate whose preparation was never stated must be
    /// REFUSED, not measured, because the measurement succeeds either way — it returns a porosity
    /// built from blue-ish feldspar and edge artefact instead of failing, and nothing downstream
    /// can tell that number from a real one.
    #[test]
    fn a_plate_that_was_never_declared_impregnated_is_refused() {
        assert!(epoxy_check("blue_epoxy").is_ok());
        assert!(epoxy_check("").is_err(), "unknown must never be treated as impregnated");
        assert!(epoxy_check("plain").is_err());
        assert!(epoxy_check("something else").is_err());
    }

    fn plate(name: &str, top: f32, base: Option<f32>, fraction: f32) -> PlatePore {
        PlatePore {
            image_id: name.into(),
            name: name.into(),
            depth_top: top,
            depth_base: base,
            pore_fraction: fraction,
            scene_hue: 40.0,
            scene_dominated: false,
            cast_shift: f32::NAN,
            reference_name: String::new(),
            band_missed: false,
            pixels: 1_000_000,
            geometry: None,
            grains: None,
            stain: None,
        }
    }

    /// The agreement check has to score EXACTLY the plates the write would store.
    ///
    /// A plate the run has already refused — the band matched the background, or it claimed less
    /// than one pore — is a plate whose fraction is not a porosity. Letting one into the number the
    /// user picks a reference plate on would be the tool grading itself on the answers it already
    /// threw away, and the failure would be quiet: a scene-dominated plate reads near 1.0, which is
    /// exactly the kind of outlier that moves a correlation on its own.
    #[test]
    fn the_agreement_scores_only_the_plates_the_write_would_keep() {
        let mut dominated = plate("blue-cast", 2001.0, None, 0.97);
        dominated.scene_dominated = true;
        let mut missed = plate("green-cast", 2002.0, None, 0.0004);
        missed.band_missed = true;

        let plates = vec![
            plate("good", 2000.0, None, 0.21),
            dominated,
            missed,
            // A core photograph spans an interval; a thin section does not. Anchored at its middle,
            // the same place the point tracks draw it.
            plate("slab", 2003.0, Some(2004.0), 0.18),
        ];

        let got = storable_samples(&plates);
        assert_eq!(got.len(), 2, "the two refused plates are not scored");
        assert!((got[0].depth - 2000.0).abs() < 1e-6, "a point plate stays a point");
        assert!((got[0].value - 0.21).abs() < 1e-6);
        assert!((got[1].depth - 2003.5).abs() < 1e-6, "an interval plate pairs on its middle");
    }

    /// The other half of the impregnation problem, and the one a real delivery found.
    ///
    /// `epoxy_check` refuses the plate nobody impregnated. It says nothing about a plate that WAS
    /// impregnated but photographed under a light the band was never tuned for — there the rule
    /// swallows the matrix and returns a porosity that looks entirely reasonable. The numbers here
    /// are the ones measured on a real carbonate delivery: one blue-cast plate whose whole scene
    /// sat at 221 degrees read 0.97 v/v, while a green-cast plate from the same core at 149
    /// degrees read 0.06.
    #[test]
    fn a_plate_whose_own_median_hue_is_pore_coloured_is_not_measured() {
        let band = PoreColorBand::default();
        assert!(scene_dominated(221.0, &band), "a blue-cast plate is the band, not the pores");
        assert!(!scene_dominated(149.0, &band), "a green-cast plate still has grains to see");
        assert!(!scene_dominated(41.0, &band), "a warm-cast plate is nowhere near the band");
        // A plate that produced no hue at all cannot be judged, and an unjudgeable plate must not
        // be refused on a guess — it is the same discipline as an absent scale.
        assert!(!scene_dominated(f32::NAN, &band));
    }

    /// The guard has to read a wrap-around band the same way the runner's `in_band` does, or it
    /// would fire on exactly the wrong plates: a band written 340..20 is two arcs across red, and
    /// reading it as an empty range would silently disable the check for anyone using one.
    #[test]
    fn the_scene_check_reads_a_wrapped_band_the_way_the_runner_does() {
        let wrapped = PoreColorBand { hue_lo: 340.0, hue_hi: 20.0, ..PoreColorBand::default() };
        assert!(scene_dominated(350.0, &wrapped));
        assert!(scene_dominated(10.0, &wrapped));
        assert!(!scene_dominated(180.0, &wrapped), "the middle of the wheel is outside both arcs");
        // Degrees are periodic; a hue arriving as 370 is 10.
        assert!(scene_dominated(370.0, &wrapped));
    }

    /// The colour band ships as a generic starting point for a visual tuning task, never as a
    /// calibration. Same discipline as `gr_normalize`'s reference percentiles: a two-decimal
    /// threshold would be somebody's regression result, and this has no regression behind it.
    #[test]
    fn the_default_colour_band_is_generic_not_a_calibration() {
        let b = PoreColorBand::default();
        for v in [b.hue_lo, b.hue_hi] {
            assert_eq!(v.fract(), 0.0, "a fractional hue would be a fitted number, not a starting point");
            assert_eq!(v % 10.0, 0.0, "the band is round numbers on purpose");
        }
        assert!(b.hue_lo < b.hue_hi && b.hue_lo >= 150.0 && b.hue_hi <= 280.0, "a plain blue band");
        // Floors low enough to admit pale, thin epoxy: the user tightens them against the preview.
        assert!(b.sat_min > 0.0 && b.sat_min <= 0.2);
        assert!(b.val_min > 0.0 && b.val_min <= 0.2);
    }

    /// A hue band written across 0 degrees is two arcs. Nothing blue needs it, but a user who
    /// types one must not silently measure zero — the runner handles it and this records why.
    #[test]
    fn a_band_written_across_zero_is_not_an_empty_range() {
        assert!(PORE_RUNNER.contains("(h >= lo) | (h <= hi)"));
    }

    /// The real round trip, on a plate whose blue fraction is known exactly by construction.
    /// `#[ignore]`d because it needs Pillow: the green gate must never depend on an optional
    /// package (rule 7), the same reason the office round-trips are ignored.
    ///
    /// The plate is a quarter blue epoxy, with a pale violet patch that a hue test alone would
    /// count — it is the SATURATION floor that rejects it, which is why the floor exists.
    #[test]
    #[ignore]
    fn a_quarter_blue_plate_measures_a_quarter() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-TS", None, None, None).unwrap();
        let w = wid.to_string();

        // ONE delivery holding all three: exactly one set per (well, dataset) is live, so three
        // separate sets would leave only the last one visible.
        let png = synthetic_plate();
        let plates: Vec<crate::db::NewImage> = [("TS-1", "blue_epoxy"), ("TS-2", ""), ("TS-3", "plain")]
            .iter()
            .enumerate()
            .map(|(i, (name, prepared))| crate::db::NewImage {
                depth_top: 2000.0 + i as f32,
                name: (*name).to_string(),
                mime: "image/bmp".into(),
                width: 200,
                height: 200,
                data: png.clone(),
                printable: true,
                prepared: if prepared.is_empty() { None } else { Some((*prepared).into()) },
                ..Default::default()
            })
            .collect();
        crate::db::insert_well_images(&conn, &w, "THIN SECTION", "LAB", None, &plates).unwrap();

        let spec = PoreSpec {
            well_id: w.clone(),
            dataset: "THIN SECTION".into(),
            band: PoreColorBand::default(),
            reference_image_id: None,
            reference_zones: Vec::new(),
            preview_image_id: None,
            only_image_id: None,
            set_name: Some("TS".into()),
            geometry: false,
            min_pore_px: MIN_PORE_PX,
            grains: false,
            min_grain_px: MIN_GRAIN_PX,
            grain_sep_px: GRAIN_SEP_PX,
            wicksell: false,
            stain: None,
            check_against: None,
            check_depth_tol: 0.0,
        };
        let res = run_pore_area(&conn, &spec).expect("pore run");

        // One declared plate measured, two refused BY NAME — a silent subset would read as a
        // complete answer.
        assert_eq!(res.plates.len(), 1, "only the declared plate is measured");
        assert_eq!(res.plates[0].name, "TS-1");
        assert!(
            (res.plates[0].pore_fraction - 0.25).abs() < 1e-4,
            "a quarter-blue plate came to {}",
            res.plates[0].pore_fraction
        );
        assert_eq!(res.plates[0].pixels, 40_000);
        assert_eq!(res.skipped.len(), 2);
        assert!(res.skipped.iter().any(|s| s.starts_with("TS-2") && s.contains("not stated")));
        assert!(res.skipped.iter().any(|s| s.starts_with("TS-3") && s.contains("not impregnated")));

        // Stored as point data under its own dataset, at the plate's depth.
        let (ds, set) = res.written.clone().expect("written");
        assert_eq!(ds, PORE_DATASET);
        let rows = crate::db::list_aux_data(&conn, &w, Some(PORE_DATASET)).unwrap();
        assert_eq!(rows.len(), 1, "one measured plate, one point sample (set {set})");
        assert_eq!(rows[0].item, PORE_ITEM);
        assert!((rows[0].value_num.unwrap() - 0.25).abs() < 1e-4);
        assert!((rows[0].depth_top - 2000.0).abs() < 1e-4);
    }

    /// The failure a real delivery found, end to end: a plate photographed under a light that puts
    /// the WHOLE scene inside the pore band is measured and shown, but nothing off it is stored.
    ///
    /// Both plates here are declared `blue_epoxy`, so `epoxy_check` passes them both — which is the
    /// point. The first is a normal section, a quarter blue on grey. The second is the same rock
    /// under a blue cast: every pixel is some shade of blue, so the rule returns a porosity near 1
    /// and it looks like an answer. On the real carbonate delivery 52 of 134 plates were this.
    #[test]
    #[ignore = "runs the real Python runner; needs numpy and Pillow"]
    fn a_blue_cast_plate_is_shown_but_never_stored() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-TS", None, None, None).unwrap();
        let w = wid.to_string();

        // Every pixel blue, only the shade differs — the matrix is dark blue, the pores bright.
        let cast = bmp(200, 200, |x, _y| if x < 40 { (60, 120, 235) } else { (20, 40, 90) });
        let mk = |name: &str, depth: f32, data: Vec<u8>| crate::db::NewImage {
            depth_top: depth,
            name: name.into(),
            mime: "image/bmp".into(),
            width: 200,
            height: 200,
            data,
            printable: true,
            prepared: Some("blue_epoxy".into()),
            ..Default::default()
        };
        crate::db::insert_well_images(
            &conn,
            &w,
            "THIN SECTION",
            "LAB",
            None,
            &[mk("NORMAL", 2000.0, synthetic_plate()), mk("CAST", 2001.0, cast)],
        )
        .unwrap();

        let res = run_pore_area(
            &conn,
            &PoreSpec {
                well_id: w.clone(),
                dataset: "THIN SECTION".into(),
                band: PoreColorBand::default(),
                reference_image_id: None,
                reference_zones: Vec::new(),
                preview_image_id: None,
                only_image_id: None,
                set_name: Some("TS".into()),
                geometry: false,
                min_pore_px: MIN_PORE_PX,
                grains: false,
                min_grain_px: MIN_GRAIN_PX,
                grain_sep_px: GRAIN_SEP_PX,
                wicksell: false,
                stain: None,
                check_against: None,
                check_depth_tol: 0.0,
            },
        )
        .expect("pore run");

        let plate = |n: &str| res.plates.iter().find(|p| p.name == n).expect(n);
        // BOTH are measured and returned: the number is what the band gets tuned against, and a
        // plate the user cannot see is a plate they cannot fix.
        assert_eq!(res.plates.len(), 2);
        assert!(!plate("NORMAL").scene_dominated, "a quarter-blue plate is a normal section");
        assert!(plate("CAST").scene_dominated, "an all-blue plate is the band, not the pores");
        assert!(
            plate("CAST").pore_fraction > 0.5,
            "the point is that it returns a big plausible number: {}",
            plate("CAST").pore_fraction
        );

        // Only the normal plate reaches the store. This is the assertion that matters: a 0.9 at a
        // real depth would go on to plot against helium porosity and nothing downstream could tell.
        let rows = crate::db::list_aux_data(&conn, &w, Some(PORE_DATASET)).unwrap();
        assert_eq!(rows.len(), 1, "one storable plate");
        assert!((rows[0].depth_top - 2000.0).abs() < 1e-4, "the normal plate's depth");
        assert!(res.skipped.iter().any(|s| s.starts_with("CAST") && s.contains("median hue")));
    }

    /// 200x200: a quarter pure blue epoxy, the rest grey, plus a pale violet square whose hue is
    /// in the band and whose saturation is not.
    #[cfg(test)]
    fn synthetic_plate() -> Vec<u8> {
        // Written as a raw PNG through Pillow would need Python; instead the test is ignored and
        // the fixture is produced by the same runner's dependency. Kept as an uncompressed BMP,
        // which Pillow reads and which needs no encoder here.
        let (w, h) = (200usize, 200usize);
        let row = w * 3;
        let pad = (4 - row % 4) % 4;
        let stride = row + pad;
        let pixels = stride * h;
        let mut out = Vec::with_capacity(54 + pixels);
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&((54 + pixels) as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&54u32.to_le_bytes());
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&(w as i32).to_le_bytes());
        out.extend_from_slice(&(h as i32).to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&24u16.to_le_bytes());
        for _ in 0..6 {
            out.extend_from_slice(&0u32.to_le_bytes());
        }
        for y in 0..h {
            let yy = h - 1 - y; // BMP rows run bottom-up
            for x in 0..w {
                let (r, g, b) = if yy < 100 && x < 100 {
                    (32u8, 64u8, 192u8) // blue epoxy, a quarter of the plate
                } else if yy >= 150 && x >= 150 {
                    (200, 196, 225) // pale violet: right hue, not enough saturation
                } else {
                    (180, 178, 172) // grey matrix
                };
                out.extend_from_slice(&[b, g, r]);
            }
            out.extend(std::iter::repeat(0u8).take(pad));
        }
        out
    }

    /// Pore diameters are weighted by AREA, because capillary pressure fills volume. A
    /// count-weighted median on a digitized section is dominated by the smallest features the scan
    /// can resolve, which says more about the scan than about the rock.
    #[test]
    fn a_pore_median_is_weighted_by_area_not_by_count() {
        // Nine small pores and one large one. By count the median is small; by area the single
        // large pore holds most of the volume and the median moves onto it.
        let mut diam: Vec<f64> = vec![1.0; 9];
        diam.push(10.0);
        let area: Vec<f64> = diam.iter().map(|d| d * d).collect(); // area goes as diameter squared
        assert!((weighted_percentile(&diam, &area, 50.0) - 10.0).abs() < 1e-9);

        // The same numbers weighted equally give the count median, so the difference above is the
        // weighting and not the algorithm.
        let flat = vec![1.0; diam.len()];
        assert!((weighted_percentile(&diam, &flat, 50.0) - 1.0).abs() < 1e-9);
    }

    /// A classifier cannot be checked on clicks it was fitted on, and one class cannot be a
    /// classification. Both are refused before a subprocess is even started.
    #[test]
    fn the_classifier_refuses_a_training_set_it_could_not_be_checked_on() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-TS", None, None, None).unwrap();
        let w = wid.to_string();
        crate::db::insert_well_images(
            &conn,
            &w,
            "THIN SECTION",
            "LAB",
            None,
            &[crate::db::NewImage {
                depth_top: 2000.0,
                name: "TS-1".into(),
                mime: "image/bmp".into(),
                width: 200,
                height: 200,
                data: disc_plate(),
                printable: true,
                ..Default::default()
            }],
        )
        .unwrap();
        let spec = |labels: Vec<PlateLabel>| ClassifySpec {
            well_id: w.clone(),
            dataset: "THIN SECTION".into(),
            labels,
            patch_px: PATCH_PX,
            set_name: None,
            preview_image_id: None,
        };
        let click = |m: &str, x: f32| PlateLabel {
            image_id: "x".into(),
            x,
            y: 0.5,
            mineral: m.to_string(),
        };

        // One class: a model that always says "quartz" is right every time and knows nothing.
        let one: Vec<PlateLabel> = (0..5).map(|i| click("Quartz", i as f32 / 10.0)).collect();
        let err = run_plate_classifier(&conn, &spec(one)).unwrap_err();
        assert!(err.contains("at least two minerals"), "{err}");

        // Two classes, but one of them clicked once — nothing can be held out for it.
        let mut thin: Vec<PlateLabel> = (0..5).map(|i| click("Quartz", i as f32 / 10.0)).collect();
        thin.push(click("Feldspar", 0.9));
        let err = run_plate_classifier(&conn, &spec(thin)).unwrap_err();
        assert!(err.contains("Feldspar") && err.contains("at least"), "{err}");
    }

    /// The cross-validation groups by CLICK. Neighbouring pixels of one click are near-identical,
    /// so splitting them across the fold boundary scores the model on data it has already seen and
    /// reports an accuracy nobody can reproduce on a new plate.
    #[test]
    fn the_classifier_is_cross_validated_by_click_not_by_pixel() {
        assert!(CLASSIFY_RUNNER.contains("GroupKFold"));
        assert!(CLASSIFY_RUNNER.contains("groups.extend([gi]"));
        assert!(CLASSIFY_RUNNER.contains("GROUP is the click, not the pixel"));
    }

    /// A classified fraction and a stain fraction are different claims and must not share a name.
    #[test]
    fn a_classified_fraction_is_not_stored_as_a_stain_fraction() {
        assert_eq!(CLASS_PREFIX, "CLS_");
        assert_ne!(CLASS_PREFIX, "MIN_");
        let src = include_str!("petrography.rs");
        assert!(src.contains("{CLASS_PREFIX}{}"), "the classifier writes under its own prefix");
    }

    /// Labels are stored as FRACTIONS of the picture, never pixels — the stored copy is resampled,
    /// so a pixel coordinate belongs to a copy and nothing in it says which.
    #[test]
    fn a_label_is_a_fraction_of_the_picture_not_a_pixel() {
        let src = include_str!("petrography.rs");
        assert!(src.contains("Position is a FRACTION of the picture, never a pixel"));
        assert!(CLASSIFY_RUNNER.contains("float(lab[\"x\"]) * (wid - 1)"));
    }

    /// Hue is circular, so it must not enter a distance-based model as a raw angle: 359 and 1
    /// degrees are neighbours, and a raw hue puts them at opposite ends of the feature.
    #[test]
    fn hue_enters_the_model_as_a_circle() {
        assert!(CLASSIFY_RUNNER.contains("np.cos(hr), np.sin(hr)"));
        assert!(CLASSIFY_RUNNER.contains("Hue is circular"));
    }

    /// A stain scheme read off the wrong stain returns mineral fractions that are wrong and
    /// entirely plausible, so the plate's OWN declared stain has to agree — and "not stated" is
    /// refused, not assumed, exactly as `prepared` is.
    #[test]
    fn a_stain_scheme_only_applies_to_the_stain_it_is_for() {
        assert_eq!(normalize_stain("Alizarin Red S"), "alizarinreds");
        assert_eq!(normalize_stain("alizarin-red-s"), "alizarinreds");
        // A spelling difference is one stain; a different stain is not.
        assert_ne!(normalize_stain("Alizarin red S"), normalize_stain("Potassium ferricyanide"));
        assert_eq!(normalize_stain("  "), "", "an undeclared stain normalizes to nothing");
        // The refusal text, so the two branches cannot quietly become one.
        let src = include_str!("petrography.rs");
        assert!(src.contains("stain not stated"));
        assert!(src.contains("but the scheme is for"));
    }

    /// The shipped schemes carry PUBLISHED mineral identifications and GENERIC colour bands — the
    /// same split as the epoxy band, and the reason a stain scheme can ship at all.
    #[test]
    fn the_stain_schemes_are_published_identifications_with_generic_bands() {
        let dickson = stain_scheme("Alizarin red S + potassium ferricyanide").expect("scheme");
        let names: Vec<&str> = dickson.iter().map(|c| c.mineral.as_str()).collect();
        // Dickson (1966): the combined stain separates the ferroan phases from the plain ones.
        assert!(names.contains(&"Calcite") && names.contains(&"Dolomite"));
        assert!(names.contains(&"Ferroan calcite") && names.contains(&"Ferroan dolomite"));
        // Dolomite is identified by staying COLOURLESS, which a saturation floor cannot express —
        // it is the whole reason StainBand carries a ceiling.
        let dol = dickson.iter().find(|c| c.mineral == "Dolomite").unwrap();
        assert!(dol.band.sat_max < 0.5, "unstained means a saturation CEILING");

        for c in &dickson {
            for v in [c.band.hue_lo, c.band.hue_hi] {
                assert_eq!(v % 10.0, 0.0, "a fitted hue would be somebody's regression result");
            }
        }
        assert!(stain_scheme("something nobody applied").is_none(), "no scheme is invented");
    }

    /// Blue-dyed epoxy and turquoise ferroan dolomite are the same colour, and on a section that
    /// was both impregnated and stained the pore rule eats the mineral. It has to be reported.
    #[test]
    fn blue_epoxy_and_ferroan_dolomite_are_flagged_as_the_same_colour() {
        let pore = PoreColorBand::default(); // 180..260
        let dickson = stain_scheme("dickson").unwrap();
        let fdol = dickson.iter().find(|c| c.mineral == "Ferroan dolomite").unwrap();
        assert!(epoxy_collides(&pore, &fdol.band), "turquoise is inside the epoxy band");
        // …while the red end is not, so the check is not simply always true.
        let cal = dickson.iter().find(|c| c.mineral == "Calcite").unwrap();
        assert!(!epoxy_collides(&pore, &cal.band), "calcite red is nowhere near blue");
        let src = include_str!("petrography.rs");
        assert!(src.contains("that mineral is being counted as porosity"));
    }

    /// Mineral names are the user's own text, and the item they land under must stay readable.
    #[test]
    fn a_mineral_name_becomes_a_readable_item() {
        assert_eq!(mineral_item("Ferroan calcite"), "FERROAN_CALCITE");
        assert_eq!(mineral_item("  Ferroan  calcite "), "FERROAN_CALCITE");
        assert_eq!(mineral_item("K-feldspar"), "K_FELDSPAR");
    }

    /// The remainder is written on every run, never only when it is large: it is what says whether
    /// the mineral rows above are a mineralogy or a partial one.
    #[test]
    fn the_unclassified_remainder_is_always_stored() {
        let src = include_str!("petrography.rs");
        assert!(src.contains("put(\"MIN_UNCLASS\""));
        assert!(src.contains("fell in no colour band"));
        // Pore is excluded before the classes are tested, so the fractions sum against VPORE_TS.
        assert!(src.contains("Pore is claimed FIRST and excluded"));
    }

    /// Wicksell's problem has a known answer for one sphere size, and this is it.
    ///
    /// Section a population of identical spheres and the apparent diameters spread over every class
    /// below the true one. The unfolding must put the whole population back in the top class — if it
    /// does not, the correction is inventing a fine tail that is purely the sectioning.
    #[test]
    fn a_single_sphere_size_unfolds_back_to_one_class() {
        let d = 100.0f64;
        let upper = saltykov_bounds(d);
        // The EXACT apparent distribution of a monodisperse population, straight from the chord
        // geometry — no sampling, so a failure here is the unfolding and nothing else.
        let counts: Vec<f64> = (0..upper.len())
            .map(|i| {
                let lo = if i == 0 { 0.0 } else { upper[i - 1] };
                d * (section_cdf(d, upper[i]) - section_cdf(d, lo))
            })
            .collect();
        // Most sections of a sphere are near its full width, but far from all of them.
        assert!(counts.last().unwrap() / counts.iter().sum::<f64>() < 0.7);

        let (nv, clamped) = saltykov(&counts, &upper);
        assert_eq!(clamped, 0, "an exact input needs no clamping");
        let top = nv[nv.len() - 1];
        assert!(top > 0.0);
        for (i, &v) in nv.iter().enumerate().take(nv.len() - 1) {
            assert!(v < top * 1e-6, "class {i} kept {v} of a population that is all one size");
        }

        // And the corrected median lands EXACTLY on the true sphere size, because the class the
        // population went into is represented by the diameter the unfolding solved it for.
        let (wd, ww) = unfolded_distribution(&nv, &upper);
        let w50 = weighted_percentile(&wd, &ww, 50.0);
        assert!((w50 - d).abs() < 1e-9, "corrected D50 = {w50} against a true {d}");
    }

    /// **What the correction actually buys, measured rather than assumed.**
    ///
    /// A population of identical spheres is perfectly sorted, but its sections are not: cuts land
    /// anywhere from a sliver to the full width, so the section distribution has a real spread.
    /// That is the dominant Wicksell effect and it is on SORTING, not on the median — the apparent
    /// median of a monodisperse population is only about 13% low (the median chord of a sphere is
    /// √3/2 of its diameter), and area weighting pulls even that most of the way back, because it
    /// up-weights exactly the near-central cuts.
    ///
    /// So: the correction earns its place on the sorting number, and a user who applies it hoping
    /// to move D50 is applying it for the wrong reason.
    #[test]
    fn the_correction_earns_its_place_on_sorting_not_on_the_median() {
        let d = 200.0f64;
        let upper = saltykov_bounds(d);
        let counts: Vec<f64> = (0..upper.len())
            .map(|i| {
                let lo = if i == 0 { 0.0 } else { upper[i - 1] };
                d * (section_cdf(d, upper[i]) - section_cdf(d, lo))
            })
            .collect();
        let mid: Vec<f64> = (0..upper.len())
            .map(|i| {
                let lo = if i == 0 { upper[i] * 0.5 } else { upper[i - 1] };
                (lo * upper[i]).sqrt()
            })
            .collect();

        // By COUNT the sections read finer than the rock is — the classic bias.
        let c50 = weighted_percentile(&mid, &counts, 50.0);
        assert!(c50 < d * 0.95, "count median {c50} against a true {d}");

        // Apparent sorting on a perfectly sorted population is non-zero: entirely an artefact of
        // the sectioning, and exactly what the correction is for. Area-weighted it measures about
        // 0.19 phi, which on the Folk & Ward verbal scale is still inside "very well sorted"
        // (< 0.35) — so the artefact is real but modest, and worth knowing before anyone reads a
        // corrected number as a large discovery.
        let app_wt: Vec<f64> = counts.iter().zip(&mid).map(|(c, m)| c * m * m).collect();
        let app_sort = folk_ward_sorting(&mid, &app_wt);
        assert!(app_sort > 0.15, "sections of one grain size still spread: {app_sort} phi");

        // By COUNT the artefact is worse, which is the other half of the same point: the weighting
        // choice moves this number more than the correction does.
        let cnt_sort = folk_ward_sorting(&mid, &counts);
        assert!(cnt_sort > app_sort, "count sorting {cnt_sort} vs area {app_sort}");

        // Unfolded, the whole population is one class and the sorting collapses to zero.
        let (nv, _) = saltykov(&counts, &upper);
        let (wd, ww) = unfolded_distribution(&nv, &upper);
        let w_sort = folk_ward_sorting(&wd, &ww);
        assert!(w_sort.abs() < 1e-9 || w_sort.is_nan(), "corrected sorting {w_sort}");
    }

    /// Folk & Ward against a distribution whose spread is known by construction: a population
    /// spread over exactly two phi units, so (φ84−φ16)/4 + (φ95−φ5)/6 has a value that can be
    /// worked out by hand rather than trusted.
    #[test]
    fn folk_ward_sorting_is_in_phi_units_and_rises_with_spread() {
        // Uniform in phi from 1 to 3 (500 µm down to 125 µm), equal weights.
        let n = 2001;
        let diam: Vec<f64> = (0..n)
            .map(|i| {
                let phi = 1.0 + 2.0 * i as f64 / (n - 1) as f64;
                2f64.powf(-phi) * 1000.0
            })
            .collect();
        let w = vec![1.0; n];
        let s = folk_ward_sorting(&diam, &w);
        // Uniform on [1,3]: φ16 = 1.32, φ84 = 2.68, φ5 = 1.10, φ95 = 2.90.
        let want = (2.68 - 1.32) / 4.0 + (2.90 - 1.10) / 6.0;
        assert!((s - want).abs() < 0.02, "sorting {s} against {want}");

        // A single grain size is perfectly sorted, and sorting has to fall to zero there — the
        // direction of the scale matters as much as the number.
        let one = vec![250.0; 50];
        assert!(folk_ward_sorting(&one, &vec![1.0; 50]).abs() < 1e-9);
    }

    /// Phi runs the other way from diameter, and a sign slip there would flip every sorting number
    /// in a deliverable while leaving it looking entirely reasonable.
    #[test]
    fn phi_rises_as_grains_get_finer() {
        // 1 mm is phi 0, 0.5 mm is phi 1, 0.25 mm is phi 2.
        let f = |um: f64| -((um / 1000.0).log2());
        assert!((f(1000.0) - 0.0).abs() < 1e-12);
        assert!((f(500.0) - 1.0).abs() < 1e-12);
        assert!((f(250.0) - 2.0).abs() < 1e-12);
    }

    /// The apparent and corrected statistics must never share an item name.
    #[test]
    fn apparent_and_corrected_grain_sizes_are_stored_under_different_names() {
        let src = include_str!("petrography.rs");
        for item in ["GRAIN_D50_APP", "GRAIN_SORT_APP", "GRAIN_D50_W", "GRAIN_SORT_W"] {
            assert!(src.contains(item), "{item} is not written anywhere");
        }
        // A bare GRAIN_D50 would be readable as either, which is the one thing D3's answer rules
        // out: a corrected number must never leave the app without saying that it is corrected.
        // Matched on the WRITE call, not the bare name — a test that scans its own source has to
        // avoid tripping over the string it is looking for.
        for bad in ["put(\"GRAIN_D50\"", "put(\"GRAIN_SORT\"", "put(\"GRAIN_D10\"", "put(\"GRAIN_D90\""] {
            assert!(!src.contains(bad), "{bad} stores an unqualified name");
        }
    }

    /// The grain defaults are round starting points for a visual judgement, not calibrations —
    /// the same rule the colour band follows.
    #[test]
    fn the_grain_defaults_are_generic_not_a_calibration() {
        for v in [MIN_GRAIN_PX, GRAIN_SEP_PX] {
            assert_eq!(v % 10, 0, "a non-round pixel default would be somebody's regression result");
        }
        assert!(MIN_GRAIN_PX > MIN_PORE_PX, "a grain is the larger object");
    }

    /// Where the section is cemented there is no pore between the grains, so the watershed places
    /// a boundary that the picture never showed. That has to be reported, not assumed away.
    #[test]
    fn a_welded_fabric_is_reported_rather_than_measured_silently() {
        let src = include_str!("petrography.rs");
        assert!(src.contains("GRAIN_CONTACT"), "the contact fraction is stored");
        assert!(
            src.contains("placed by the watershed rather than observed"),
            "and a heavily welded plate says so in the notes"
        );
    }

    /// The perimeter estimator's honest character, recorded so nobody "fixes" it into a boundary
    /// count later.
    ///
    /// A boundary-PIXEL count overestimates a diagonal edge by up to √2 and so biases circularity
    /// systematically LOW — systematically, which means it never looks like noise. The four-
    /// direction Crofton estimate used instead is essentially exact for a circle (measured 630.1
    /// against 628.3 for radius 100, and circularity 0.994) and is at its worst on a perfectly
    /// axis-aligned rectangle, where it reads about 5% low: for a `w × h` rectangle it returns
    /// `(π/4)(w+h)(1+√2)` against a true `2(w+h)`, a ratio of 0.948 regardless of the shape.
    ///
    /// Pores are neither circles nor axis-aligned boxes, and circularity is read comparatively, so
    /// a few percent of consistent bias does not change which pore is rounder than which.
    #[test]
    fn the_perimeter_estimator_is_crofton_not_a_boundary_pixel_count() {
        assert!(PORE_RUNNER.contains("np.pi / 8.0"), "the Crofton weighting");
        assert!(PORE_RUNNER.contains("1.0 / np.sqrt(2.0)"), "diagonal families carry 1/sqrt(2)");
        // The rectangle ratio above, stated as arithmetic so the claim is checkable here.
        let ratio = (std::f64::consts::PI / 4.0) * (1.0 + 2f64.sqrt()) / 2.0;
        assert!((ratio - 0.948).abs() < 0.001);
    }

    /// Two pores meeting at a single corner are joined by a throat of zero width. That is not one
    /// pore body, and 8-connectivity would fuse them — so the pore phase is labelled 4-connected.
    #[test]
    fn the_pore_phase_is_labelled_four_connected() {
        assert!(PORE_RUNNER.contains("[[0, 1, 0], [1, 1, 1], [0, 1, 0]]"));
    }

    /// The real geometry round trip. `#[ignore]`d: it needs Pillow AND scipy.
    #[test]
    #[ignore]
    fn a_disc_reads_as_round_and_its_diameter_follows_the_declared_scale() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-TS", None, None, None).unwrap();
        let w = wid.to_string();

        // 200 px wide, a disc of radius 40 at the centre. Declared 2000 µm across, so 10 µm/px,
        // and an 80 px disc is 800 µm.
        crate::db::insert_well_images(
            &conn,
            &w,
            "THIN SECTION",
            "LAB",
            None,
            &[
                crate::db::NewImage {
                    depth_top: 2000.0,
                    name: "SCALED".into(),
                    mime: "image/bmp".into(),
                    width: 200,
                    height: 200,
                    data: disc_plate(),
                    printable: true,
                    prepared: Some("blue_epoxy".into()),
                    fov_um: Some(2000.0),
                    ..Default::default()
                },
                crate::db::NewImage {
                    depth_top: 2001.0,
                    name: "UNSCALED".into(),
                    mime: "image/bmp".into(),
                    width: 200,
                    height: 200,
                    data: disc_plate(),
                    printable: true,
                    prepared: Some("blue_epoxy".into()),
                    fov_um: None,
                    ..Default::default()
                },
            ],
        )
        .unwrap();

        let res = run_pore_area(
            &conn,
            &PoreSpec {
                well_id: w.clone(),
                dataset: "THIN SECTION".into(),
                band: PoreColorBand::default(),
                reference_image_id: None,
                reference_zones: Vec::new(),
                preview_image_id: None,
                only_image_id: None,
                set_name: Some("TS".into()),
                geometry: true,
                min_pore_px: MIN_PORE_PX,
                grains: false,
                min_grain_px: MIN_GRAIN_PX,
                grain_sep_px: GRAIN_SEP_PX,
                wicksell: false,
                stain: None,
                check_against: None,
                check_depth_tol: 0.0,
            },
        )
        .expect("geometry run");

        let plate = |n: &str| res.plates.iter().find(|p| p.name == n).unwrap();
        let g = plate("SCALED").geometry.as_ref().unwrap();
        assert_eq!(g.n, 1, "one disc, one pore");
        assert!((g.aspect_p50 - 1.0).abs() < 0.02, "a disc is not elongated: {}", g.aspect_p50);
        assert!(g.shape_p50 > 0.98 && g.shape_p50 < 1.02, "a disc is round: {}", g.shape_p50);
        assert!(
            (g.d50_um.unwrap() - 800.0).abs() < 8.0,
            "80 px at 10 µm/px is 800 µm, got {:?}",
            g.d50_um
        );

        // The same disc with no declared scale reports its SHAPE and no size at all — a diameter
        // in pixels is not a diameter.
        let u = plate("UNSCALED").geometry.as_ref().unwrap();
        assert!((u.aspect_p50 - 1.0).abs() < 0.02);
        assert!(u.d50_um.is_none() && u.d10_um.is_none() && u.d90_um.is_none());

        // And the point data carries the dimensional items only for the calibrated plate.
        let rows = crate::db::list_aux_data(&conn, &w, Some(PORE_DATASET)).unwrap();
        let at = |d: f32, item: &str| rows.iter().find(|r| (r.depth_top - d).abs() < 1e-3 && r.item == item);
        assert!(at(2000.0, "PORE_D50").is_some());
        assert!(at(2001.0, "PORE_D50").is_none(), "no scale, no diameter — not even a NaN");
        assert!(at(2001.0, "PORE_ASPECT").is_some(), "shape is dimensionless and always reported");
    }

    /// 200x200 BMP: a blue-epoxy disc of radius 40 at the centre, grey elsewhere.
    #[cfg(test)]
    /// **The measured demonstration that this family is worth having, and the one that shows its
    /// honesty machinery firing.** `#[ignore]`d because it needs scikit-learn.
    ///
    /// Two halves of the plate share the SAME mean colour and differ only in texture — one smooth,
    /// one cloudy. Colour alone cannot separate them, which is exactly the case
    /// `docs/plan_image_analysis.md` §2.1 says a colour rule must not pretend to handle. Measured
    /// through the real runner: accuracy 1.000, both recalls 1.000, fractions 0.504 / 0.496
    /// against a true half and half.
    ///
    /// The CONTROL is the more important half. Label one uniform material as two minerals and the
    /// model has nothing to learn: held-out accuracy fell to 0.410, recalls 0.38 and 0.44 — near
    /// chance — and `run_plate_classifier` then names both classes as unreliable. A classifier that
    /// cannot be caught inventing a distinction is worse than no classifier.
    #[test]
    #[ignore]
    fn the_classifier_separates_on_texture_and_admits_when_it_cannot() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-TS", None, None, None).unwrap();
        let w = wid.to_string();

        // A cheap deterministic speckle: no rng in the test, so the fixture is reproducible.
        let mut seed = 12345u64;
        let mut noise = move |amp: f64| -> f64 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((seed >> 33) as f64 / (1u64 << 31) as f64 - 1.0) * amp
        };
        let mut px = vec![0u8; 200 * 200];
        for y in 0..200usize {
            for x in 0..200usize {
                let amp = if x < 100 { 2.0 } else { 24.0 };
                px[y * 200 + x] = (205.0 + noise(amp)).clamp(0.0, 255.0) as u8;
            }
        }
        let plate = bmp(200, 200, |x, y| {
            let v = px[y * 200 + x];
            (v, v, v)
        });
        crate::db::insert_well_images(
            &conn,
            &w,
            "THIN SECTION",
            "LAB",
            None,
            &[crate::db::NewImage {
                depth_top: 2000.0,
                name: "TS-1".into(),
                mime: "image/bmp".into(),
                width: 200,
                height: 200,
                data: plate,
                printable: true,
                ..Default::default()
            }],
        )
        .unwrap();
        let images = crate::db::list_well_images(&conn, &w, Some("THIN SECTION")).unwrap();
        let iid = images[0].image_id.clone();

        let mut labels = Vec::new();
        for i in 0..6 {
            labels.push(PlateLabel {
                image_id: iid.clone(),
                x: 0.10 + 0.05 * i as f32,
                y: 0.2 + 0.1 * i as f32,
                mineral: "Quartz".into(),
            });
            labels.push(PlateLabel {
                image_id: iid.clone(),
                x: 0.60 + 0.05 * i as f32,
                y: 0.2 + 0.1 * i as f32,
                mineral: "Feldspar".into(),
            });
        }
        let res = run_plate_classifier(
            &conn,
            &ClassifySpec {
                well_id: w.clone(),
                dataset: "THIN SECTION".into(),
                labels,
                patch_px: PATCH_PX,
                set_name: Some("CLS".into()),
                preview_image_id: None,
            },
        )
        .expect("classifier run");

        assert!(res.accuracy > 0.85, "same colour, different texture: accuracy {}", res.accuracy);
        let q = res.plates[0].fractions.iter().find(|(m, _)| m == "Quartz").unwrap().1;
        assert!((q - 0.5).abs() < 0.08, "half the plate is quartz, got {q}");
        // Its own prefix, so a classified fraction can never be read as a stain fraction.
        let rows = crate::db::list_aux_data(&conn, &w, Some(PORE_DATASET)).unwrap();
        assert!(rows.iter().any(|r| r.item == "CLS_QUARTZ"));
        assert!(!rows.iter().any(|r| r.item.starts_with("MIN_")));
        assert!(res.notes.iter().any(|n| n.contains("part of what it learned")));
    }

    /// The real grain round trip. `#[ignore]`d for the usual reason: it needs scipy, and the green
    /// gate must never depend on an optional package.
    ///
    /// Two plates, both 200 px wide and declared 2000 µm across, so 10 µm/px:
    ///   LOOSE  — four separated discs of radius 20. Every boundary is open pore, so the contact
    ///            fraction must be ~0 and each diameter must come back as 400 µm.
    ///   WELDED — two discs of radius 30 overlapping into one blob. They must still come out as
    ///            TWO grains, and the contact fraction must rise to say that the boundary between
    ///            them was placed by the algorithm rather than seen in the picture.
    #[test]
    #[ignore]
    fn welded_grains_still_split_but_say_that_the_boundary_was_inferred() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-TS", None, None, None).unwrap();
        let w = wid.to_string();

        let grain = |x: usize, y: usize, centres: &[(f64, f64, f64)]| {
            for (cx, cy, r) in centres {
                let (dx, dy) = (x as f64 - cx, y as f64 - cy);
                if dx * dx + dy * dy <= r * r {
                    return (180u8, 178u8, 172u8); // grain
                }
            }
            (32, 64, 192) // blue epoxy
        };
        // GRAINS DOMINATE BOTH PLATES, and that is not cosmetic. These used to be small discs
        // floating in epoxy — 87% pore, which is a mount rather than a rock, and `scene_dominated`
        // rightly refuses to store anything measured off it. A fixture that could not exist is a
        // fixture that cannot catch the bug the real delivery found.
        let loose = bmp(200, 200, |x, y| {
            grain(x, y, &[(52.0, 52.0, 45.0), (148.0, 52.0, 45.0), (52.0, 148.0, 45.0), (148.0, 148.0, 45.0)])
        });
        // Same neck-to-radius ratio as before (1.48), so the separation problem is unchanged; only
        // the frame is cropped to the grains, the way a real field of view is.
        let welded = bmp(200, 110, |x, y| grain(x, y, &[(65.0, 55.0, 52.0), (135.0, 55.0, 52.0)]));

        let mk = |name: &str, depth: f32, data: Vec<u8>, height: i32| crate::db::NewImage {
            depth_top: depth,
            name: name.into(),
            mime: "image/bmp".into(),
            width: 200,
            height,
            data,
            printable: true,
            prepared: Some("blue_epoxy".into()),
            fov_um: Some(2000.0),
            ..Default::default()
        };
        crate::db::insert_well_images(
            &conn,
            &w,
            "THIN SECTION",
            "LAB",
            None,
            &[mk("LOOSE", 2000.0, loose, 200), mk("WELDED", 2001.0, welded, 110)],
        )
        .unwrap();

        let res = run_pore_area(
            &conn,
            &PoreSpec {
                well_id: w.clone(),
                dataset: "THIN SECTION".into(),
                band: PoreColorBand::default(),
                reference_image_id: None,
                reference_zones: Vec::new(),
                preview_image_id: None,
                only_image_id: None,
                set_name: Some("TS".into()),
                geometry: false,
                min_pore_px: MIN_PORE_PX,
                grains: true,
                min_grain_px: MIN_GRAIN_PX,
                grain_sep_px: GRAIN_SEP_PX,
                wicksell: true,
                stain: None,
                check_against: None,
                check_depth_tol: 0.0,
            },
        )
        .expect("grain run");

        let plate = |n: &str| res.plates.iter().find(|p| p.name == n).unwrap();
        let l = plate("LOOSE").grains.as_ref().expect("loose grains");
        assert_eq!(l.n, 4, "four separated discs are four grains");
        assert!(l.contact_p50 < 0.02, "open pore all round: contact {}", l.contact_p50);
        // Radius 45 px at 10 µm/px (2000 µm across 200 px) is a 900 µm grain.
        let d50 = l.d50_app_um.expect("a declared scale gives a diameter");
        assert!((d50 - 900.0).abs() < 25.0, "apparent D50 {d50} against a true 900");
        assert!((l.aspect_p50 - 1.0).abs() < 0.03, "a disc is round: {}", l.aspect_p50);

        let we = plate("WELDED").grains.as_ref().expect("welded grains");
        assert_eq!(we.n, 2, "one blob, but two grains");
        assert!(
            we.contact_p50 > 0.05,
            "the neck must register as an inferred boundary: contact {}",
            we.contact_p50
        );

        // Both plates carry a scale, so the corrected items exist and are COARSER than apparent.
        let wd = l.d50_w_um.expect("wicksell was asked for");
        assert!(wd >= d50 * 0.9, "corrected {wd} against apparent {d50}");

        // Stored under names that say which they are.
        let rows = crate::db::list_aux_data(&conn, &w, Some(PORE_DATASET)).unwrap();
        let has = |item: &str| rows.iter().any(|r| r.item == item);
        for item in ["GRAIN_N", "GRAIN_CONTACT", "GRAIN_D50_APP", "GRAIN_SORT_APP", "GRAIN_D50_W"] {
            assert!(has(item), "{item} was not written");
        }
        assert!(!has("GRAIN_D50"), "nothing is stored under an unqualified name");
    }

    fn disc_plate() -> Vec<u8> {
        bmp(200, 200, |x, y| {
            let (dx, dy) = (x as f64 - 100.0, y as f64 - 100.0);
            if dx * dx + dy * dy <= 40.0 * 40.0 { (32, 64, 192) } else { (180, 178, 172) }
        })
    }

    /// Uncompressed 24-bit BMP, which Pillow reads and which needs no encoder here.
    #[cfg(test)]
    fn bmp(w: usize, h: usize, px: impl Fn(usize, usize) -> (u8, u8, u8)) -> Vec<u8> {
        let row = w * 3;
        let pad = (4 - row % 4) % 4;
        let pixels = (row + pad) * h;
        let mut out = Vec::with_capacity(54 + pixels);
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&((54 + pixels) as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&54u32.to_le_bytes());
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&(w as i32).to_le_bytes());
        out.extend_from_slice(&(h as i32).to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&24u16.to_le_bytes());
        for _ in 0..6 {
            out.extend_from_slice(&0u32.to_le_bytes());
        }
        for y in 0..h {
            let yy = h - 1 - y; // BMP rows run bottom-up
            for x in 0..w {
                let (r, g, b) = px(x, yy);
                out.extend_from_slice(&[b, g, r]);
            }
            out.extend(std::iter::repeat(0u8).take(pad));
        }
        out
    }

    /// Results are point data at the plug depth, under their own dataset. Re-running a measurement
    /// must not look like a second delivery of pictures.
    #[test]
    fn the_measurement_is_its_own_point_dataset() {
        assert_eq!(PORE_DATASET, "PETROGRAPHY");
        assert_ne!(PORE_DATASET, "THIN SECTION");
        assert_eq!(PORE_ITEM, "VPORE_TS");
    }

    /// Hue is a circle. A warm-cast plate at 10 degrees is twenty degrees from a reference at 350,
    /// not three hundred and forty — and reading it the wrong way round would make the plates
    /// closest to the reference look like the worst cast in the delivery.
    #[test]
    fn the_cast_shift_measures_the_short_way_round_the_colour_wheel() {
        assert!((hue_delta(10.0, 350.0) - 20.0).abs() < 1e-3);
        assert!((hue_delta(350.0, 10.0) - 20.0).abs() < 1e-3);
        assert!((hue_delta(20.0, 200.0) - 180.0).abs() < 1e-3);
        // Never more than half a turn, whichever way it is written.
        assert!((hue_delta(20.0, 210.0) - 170.0).abs() < 1e-3);
        assert!((hue_delta(149.0, 195.0) - 46.0).abs() < 1e-3);
        assert!(hue_delta(f32::NAN, 10.0).is_nan(), "an unknown hue is not a zero shift");
    }

    /// The empty-measurement refusal needs BOTH of its conditions, and the pair is the point.
    ///
    /// Refusing a near-zero fraction on its own would refuse a delivery where the band has simply
    /// never been tuned, which is every first click. Naming a reference plate is the user's
    /// statement that the band finds epoxy on THAT plate; only after that does a plate showing
    /// nothing mean something.
    #[test]
    fn an_empty_measurement_is_refused_only_once_a_reference_plate_says_the_band_works() {
        let px = 1_000_000i64;
        // 20 pixels of a million is 2e-5 — less than one resolvable pore.
        let empty = 1e-5f32;
        let real = 0.08f32;

        assert!(band_missed(empty, px, MIN_PORE_PX, true), "empty, and a reference was named");
        assert!(!band_missed(empty, px, MIN_PORE_PX, false), "no reference: nothing is claimed yet");
        assert!(!band_missed(real, px, MIN_PORE_PX, true), "a real 8% is a measurement");
        assert!(!band_missed(real, px, MIN_PORE_PX, false));

        // The yardstick is the user's OWN resolution floor, not a new constant: exactly one pore's
        // worth is a measurement, anything under it is not.
        let one_pore = MIN_PORE_PX as f32 / px as f32;
        assert!(!band_missed(one_pore * 1.01, px, MIN_PORE_PX, true));
        assert!(band_missed(one_pore * 0.5, px, MIN_PORE_PX, true));
        // Raising the floor raises the bar with it, in both directions.
        assert!(band_missed(one_pore * 1.01, px, MIN_PORE_PX * 4, true));

        assert!(!band_missed(f32::NAN, px, MIN_PORE_PX, true), "no answer is not an empty answer");
        assert!(!band_missed(empty, 0, MIN_PORE_PX, true), "a plate with no pixels was not read");
    }

    /// The correction is anchored on the delivery's own rock, and it must stay that way.
    ///
    /// Grey-world — forcing the three channel means together — is the textbook white balance and is
    /// actively wrong here: a blue-epoxy section IS genuinely blue-biased, and the more porous it
    /// is the more so, so grey-world would normalize away the very signal being measured and
    /// compress every plate toward the same answer. Using the reference plate's own matrix colour
    /// assumes only that the rock is the same rock.
    #[test]
    fn the_colour_correction_is_anchored_on_the_reference_plate_not_on_grey() {
        let src = PORE_RUNNER;
        assert!(src.contains("def gain_for("), "the correction is gone");
        assert!(src.contains("reference_rgb[c]"), "the anchor must be the reference plate");
        // The anchor is the MATRIX, never the whole plate: the whole-plate median moves with how
        // much epoxy is in the field of view, so anchoring on it normalizes away the contrast.
        assert!(src.contains("def matrix_rgb(a, pore):"), "the anchor is no longer the matrix");
        assert!(src.contains("a[..., c][solid]"), "the median must exclude the pore phase");
        assert!(src.contains("np.median"), "the matrix colour is a median, not a mean");
        // No channel may be pushed past 1 and clipped: clipping distorts the hue of exactly the
        // brightest pixels, which is where a stained or strongly lit section carries its colour.
        assert!(src.contains("mx = max(g)"), "the gain must be scaled so nothing clips");
        // The stain has to be read off the SAME corrected picture the pore rule was read from.
        assert!(src.contains("def stain_from(h, s, v,"));
        assert!(!src.contains("hsv_of(img)"), "a second uncorrected conversion has come back");
    }

    /// The same rock under a different lamp reads as the same rock.
    ///
    /// This is the finding from the first real delivery, turned into a fixture. Two plates cut from
    /// identical rock — a quarter blue epoxy on a warm brown matrix — one photographed as delivered
    /// and one through a green-cast lamp (channel gains 1.0 / 2.0 / 0.55, chosen so nothing clips,
    /// which is what makes the cast a genuine white-balance error rather than a repaint). On the
    /// real delivery that failure looked like 0.04% against a counted 15%.
    ///
    /// Two more plates make the refusals honest: one with no epoxy at all, and one that is entirely
    /// the colour called pore.
    #[test]
    #[ignore = "needs numpy and Pillow"]
    fn the_same_rock_under_a_different_lamp_reads_as_the_same_rock() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-TS-2", None, None, None).unwrap();
        let w = wid.to_string();

        const MATRIX: (u8, u8, u8) = (120, 80, 60); // warm brown rock, hue 20 deg
        const PORE: (u8, u8, u8) = (40, 60, 150); // blue epoxy, hue 229 deg
        // The same two colours through a lamp 2.0x on green and 0.55x on blue. The matrix lands at
        // hue 79 and the epoxy at 152 — outside the 180..260 band, which is the whole failure.
        const MATRIX_CAST: (u8, u8, u8) = (120, 160, 33);
        const PORE_CAST: (u8, u8, u8) = (40, 120, 82);

        let quarter = |a: (u8, u8, u8), b: (u8, u8, u8)| {
            bmp(200, 200, move |x, y| if x < 100 && y < 100 { b } else { a })
        };
        let flat = |c: (u8, u8, u8)| bmp(200, 200, move |_, _| c);

        let mk = |name: &str, depth: f32, data: Vec<u8>| crate::db::NewImage {
            depth_top: depth,
            name: name.into(),
            mime: "image/bmp".into(),
            width: 200,
            height: 200,
            data,
            printable: true,
            prepared: Some("blue_epoxy".into()),
            ..Default::default()
        };
        crate::db::insert_well_images(
            &conn,
            &w,
            "THIN SECTION",
            "LAB",
            None,
            &[
                mk("REF", 2000.0, quarter(MATRIX, PORE)),
                mk("CAST", 2001.0, quarter(MATRIX_CAST, PORE_CAST)),
                mk("TIGHT", 2002.0, flat(MATRIX)),
                mk("ALLBLUE", 2003.0, flat(PORE)),
            ],
        )
        .unwrap();
        let ids = crate::db::list_well_images(&conn, &w, Some("THIN SECTION")).unwrap();
        let id_of = |n: &str| {
            ids.iter().find(|i| i.name == n).map(|i| i.image_id.clone()).expect("plate")
        };

        let spec = |reference: Option<String>, zones: Vec<ReferenceZone>, set: Option<&str>| PoreSpec {
            well_id: w.clone(),
            dataset: "THIN SECTION".into(),
            band: PoreColorBand::default(),
            reference_image_id: reference,
            reference_zones: zones,
            preview_image_id: None,
            only_image_id: None,
            set_name: set.map(str::to_string),
            geometry: false,
            min_pore_px: MIN_PORE_PX,
            grains: false,
            min_grain_px: MIN_GRAIN_PX,
            grain_sep_px: GRAIN_SEP_PX,
            wicksell: false,
            stain: None,
            check_against: None,
            check_depth_tol: 0.0,
        };

        // --- as the app behaved before: one absolute band over the whole delivery --------------
        let plain = run_pore_area(&conn, &spec(None, vec![], None)).expect("uncorrected run");
        let find = |r: &PoreResult, n: &str| r.plates.iter().find(|p| p.name == n).unwrap().clone();
        let (a_ref, a_cast) = (find(&plain, "REF"), find(&plain, "CAST"));
        assert!((a_ref.pore_fraction - 0.25).abs() < 0.01, "REF {}", a_ref.pore_fraction);
        // The failure this increment exists for: identical rock, and the answer is gone.
        assert!(
            a_cast.pore_fraction < 0.01,
            "the cast plate should read as tight before correction, not {}",
            a_cast.pore_fraction
        );
        assert!(find(&plain, "ALLBLUE").scene_dominated, "a wholly blue plate is the band");
        assert!(!a_cast.band_missed, "with no reference there is nothing to say the band works");
        assert!(a_cast.cast_shift.is_nan(), "no reference means no shift to report");

        // --- corrected onto the plate the band was tuned on --------------------------------
        let fixed = run_pore_area(&conn, &spec(Some(id_of("REF")), vec![], Some("TS"))).expect("run");
        let (b_ref, b_cast) = (find(&fixed, "REF"), find(&fixed, "CAST"));
        assert!((b_ref.pore_fraction - 0.25).abs() < 0.01, "the reference is unchanged by itself");
        assert!(
            (b_cast.pore_fraction - 0.25).abs() < 0.02,
            "the cast plate must read the same quarter: {}",
            b_cast.pore_fraction
        );
        assert!((b_cast.cast_shift - 59.0).abs() < 5.0, "shift {}", b_cast.cast_shift);
        assert!(b_ref.cast_shift.abs() < 0.01, "the reference is zero from itself");

        // The two refusals, reached by different routes and both landing on "not stored".
        assert!(find(&fixed, "TIGHT").band_missed, "no epoxy at all is not a measurement");
        // The other end: a wholly blue plate leaves no matrix to anchor a correction on, so it
        // cannot be corrected at all and is refused rather than read as delivered — which would
        // have stored it at nearly 1.0. The two guards cover opposite failures; neither can go.
        let blue = find(&fixed, "ALLBLUE");
        assert!(blue.scene_dominated, "a plate with no matrix cannot be corrected onto anything");

        // Only the two real plates reached the project.
        let rows = crate::db::list_aux_data(&conn, &w, Some(PORE_DATASET)).unwrap();
        let mut depths: Vec<f32> =
            rows.iter().filter(|r| r.item == PORE_ITEM).map(|r| r.depth_top).collect();
        depths.sort_by(f32::total_cmp);
        assert_eq!(depths, vec![2000.0, 2001.0], "only REF and CAST are measurements");

        // --- and a reference the band cannot read condemns the whole run -----------------------
        // Every plate is corrected onto it, so its median hue becomes the delivery's. A mistake
        // here would be inherited by all of them and would agree with itself everywhere.
        let err = run_pore_area(&conn, &spec(Some(id_of("ALLBLUE")), vec![], None)).unwrap_err();
        assert!(err.contains("cannot be a reference plate"), "{err}");
    }

    /// Correcting a plate onto one shot under the SAME lamp must change nothing.
    ///
    /// This is the invariant the first version of the correction broke, and it broke it silently.
    /// It anchored on the plate's WHOLE-PLATE median colour, which moves with how much epoxy is in
    /// the field of view — so two plates of the same rock under the same light, differing only in
    /// porosity, were "corrected" onto each other and the porosity contrast was partly flattened.
    /// The grey-world trap, reached by a different route. On a real delivery it cost most of the
    /// agreement with the petrographer's own point count.
    ///
    /// The fixture is built to be able to catch it. The matrix carries a top-to-bottom gradient,
    /// as an unevenly lit field of view really does, and the pore is SCATTERED evenly through the
    /// frame rather than stacked at one end — which is the load-bearing detail. Scattered pore
    /// hides the same proportion of every part of the gradient, so the MATRIX median is identical
    /// on both plates while the WHOLE-PLATE median moves with the pore fraction. Stack the pore at
    /// one end instead and it hides one end of the gradient, which biases both anchors and the
    /// test proves nothing.
    ///
    /// That discriminating power is asserted first: if the two plates' scene hues agreed, an
    /// anchor on either would pass.
    #[test]
    #[ignore = "needs numpy and Pillow"]
    fn a_plate_corrected_onto_one_lit_the_same_way_is_left_alone() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-TS-3", None, None, None).unwrap();
        let w = wid.to_string();

        // Warm brown at the top of the frame shading to olive at the bottom — one lamp, one
        // unevenly lit field. The hues run 24 to 96 degrees, well clear of the pore band at both
        // ends, so the gradient never strays into the colour being measured.
        let matrix = |y: usize| -> (u8, u8, u8) {
            let t = y as f32 / 200.0;
            ((130.0 - 45.0 * t) as u8, (85.0 + 45.0 * t) as u8, 55)
        };
        const PORE: (u8, u8, u8) = (40, 60, 150);
        // Pore scattered evenly through the frame, so it hides the same share of every part of the
        // gradient and the MATRIX median is the same on both plates.
        let plate = |rate: usize| {
            bmp(200, 200, move |x, y| {
                if (x * 7 + y * 13) % 100 < rate {
                    PORE
                } else {
                    matrix(y)
                }
            })
        };

        let mk = |name: &str, depth: f32, data: Vec<u8>| crate::db::NewImage {
            depth_top: depth,
            name: name.into(),
            mime: "image/bmp".into(),
            width: 200,
            height: 200,
            data,
            printable: true,
            prepared: Some("blue_epoxy".into()),
            ..Default::default()
        };
        crate::db::insert_well_images(
            &conn,
            &w,
            "THIN SECTION",
            "LAB",
            None,
            &[mk("LEAN", 2000.0, plate(20)), mk("RICH", 2001.0, plate(40))],
        )
        .unwrap();
        let ids = crate::db::list_well_images(&conn, &w, Some("THIN SECTION")).unwrap();
        let lean_id = ids.iter().find(|i| i.name == "LEAN").unwrap().image_id.clone();

        let spec = |reference: Option<String>| PoreSpec {
            well_id: w.clone(),
            dataset: "THIN SECTION".into(),
            band: PoreColorBand::default(),
            reference_image_id: reference,
            reference_zones: Vec::new(),
            preview_image_id: None,
            only_image_id: None,
            set_name: None,
            geometry: false,
            min_pore_px: MIN_PORE_PX,
            grains: false,
            min_grain_px: MIN_GRAIN_PX,
            grain_sep_px: GRAIN_SEP_PX,
            wicksell: false,
            stain: None,
            check_against: None,
            check_depth_tol: 0.0,
        };

        let plain = run_pore_area(&conn, &spec(None)).expect("uncorrected run");
        let get = |r: &PoreResult, n: &str| r.plates.iter().find(|p| p.name == n).unwrap().clone();
        let (lean0, rich0) = (get(&plain, "LEAN"), get(&plain, "RICH"));
        assert!((lean0.pore_fraction - 0.20).abs() < 0.01, "LEAN {}", lean0.pore_fraction);
        assert!((rich0.pore_fraction - 0.40).abs() < 0.01, "RICH {}", rich0.pore_fraction);

        // The fixture really would fool an anchor on the whole plate: the two scene hues differ,
        // and they differ only because the porosity does.
        let spread = hue_delta(lean0.scene_hue, rich0.scene_hue);
        assert!(
            spread > 5.0,
            "the fixture cannot catch a whole-plate anchor: scene hues {} and {}",
            lean0.scene_hue,
            rich0.scene_hue
        );

        let fixed = run_pore_area(&conn, &spec(Some(lean_id))).expect("corrected run");
        let rich1 = get(&fixed, "RICH");
        assert!(
            (rich1.pore_fraction - rich0.pore_fraction).abs() < 0.01,
            "same lamp, so the correction must be the identity: {} became {}",
            rich0.pore_fraction,
            rich1.pore_fraction
        );
    }

    /// A spec carrying nothing but the reference settings under test.
    fn zoned(reference: Option<&str>, zones: Vec<ReferenceZone>) -> PoreSpec {
        PoreSpec {
            well_id: String::new(),
            dataset: String::new(),
            band: PoreColorBand::default(),
            reference_image_id: reference.map(str::to_string),
            reference_zones: zones,
            preview_image_id: None,
            only_image_id: None,
            set_name: None,
            geometry: false,
            min_pore_px: MIN_PORE_PX,
            grains: false,
            min_grain_px: MIN_GRAIN_PX,
            grain_sep_px: GRAIN_SEP_PX,
            wicksell: false,
            stain: None,
            check_against: None,
            check_depth_tol: 0.0,
        }
    }

    fn zone(top: Option<f32>, base: Option<f32>, id: &str) -> ReferenceZone {
        ReferenceZone { top, base, image_id: id.to_string() }
    }

    /// Two adjacent cored intervals are written `2000-2010` and `2010-2020`, so a shared boundary
    /// depth has to be legal — neither should have to be typed a millimetre short. A genuine
    /// crossing must not be: inside one, which reference a plate is corrected onto would come down
    /// to the order of a list nobody can see.
    #[test]
    fn reference_intervals_may_touch_but_never_cross() {
        let touching = zoned(
            None,
            vec![zone(Some(2000.0), Some(2010.0), "A"), zone(Some(2010.0), Some(2020.0), "B")],
        );
        assert!(check_zones(&touching).is_ok(), "adjacent intervals are how anyone writes two runs");
        // And the shared depth is not left ambiguous: it goes to the one listed first, which is the
        // rule the per-barrel core shifts already follow.
        assert_eq!(reference_for(&touching, 2010.0), Some("A"));

        let crossing = zoned(
            None,
            vec![zone(Some(2000.0), Some(2010.0), "A"), zone(Some(2005.0), Some(2020.0), "B")],
        );
        assert!(check_zones(&crossing).unwrap_err().contains("overlap"));

        // An open-ended interval swallows every other one, which is still a crossing.
        let open = zoned(None, vec![zone(None, None, "A"), zone(Some(2000.0), Some(2010.0), "B")]);
        assert!(check_zones(&open).is_err(), "an interval covering everything crosses every other");

        // A base above its top is a typo or a transposed column; swapping them silently hides it.
        let backwards = zoned(None, vec![zone(Some(2010.0), Some(2000.0), "A")]);
        assert!(check_zones(&backwards).unwrap_err().contains("backwards"));

        // An interval with no plate chosen is refused rather than quietly reading as delivered.
        assert!(check_zones(&zoned(None, vec![zone(Some(2000.0), None, "  ")])).is_err());
    }

    /// The assignment rule: the first interval that covers the depth, then the delivery-wide plate.
    #[test]
    fn a_plate_takes_its_own_intervals_reference_then_the_delivery_wide_one() {
        let s = zoned(
            Some("WIDE"),
            vec![zone(None, Some(2000.0), "SHALLOW"), zone(Some(2500.0), None, "DEEP")],
        );
        assert_eq!(reference_for(&s, 1800.0), Some("SHALLOW"), "an open top reaches up");
        assert_eq!(reference_for(&s, 2000.0), Some("SHALLOW"), "inclusive at the base");
        assert_eq!(reference_for(&s, 2200.0), Some("WIDE"), "between the intervals: the fallback");
        assert_eq!(reference_for(&s, 2500.0), Some("DEEP"), "inclusive at the top");
        // A plate whose depth never arrived belongs to no interval, and saying so beats putting it
        // in whichever one happens to be listed first.
        assert_eq!(reference_for(&s, f32::NAN), Some("WIDE"));

        // With no delivery-wide plate the gap between the intervals has no reference at all — which
        // is what makes those plates REFUSED rather than read as delivered beside corrected ones.
        // A stored set holding both would be two measurements under one name.
        let no_fallback = zoned(None, s.reference_zones.clone());
        assert_eq!(reference_for(&no_fallback, 2200.0), None);
        assert_eq!(reference_for(&no_fallback, 1800.0), Some("SHALLOW"));
    }

    /// Two cored intervals photographed differently, each corrected onto its own reference — and a
    /// section that belongs to neither.
    ///
    /// The two lamps here are deliberately NOT a pure channel gain apart, which is the realistic
    /// case and the reason a single reference stops serving a whole delivery: the correction is
    /// exact only for a true white-balance error, and it gets less exact the further a plate has to
    /// move. So the deep sections read correctly when corrected onto a deep reference and are
    /// refused when dragged onto the shallow one — which is the same failure the real delivery
    /// showed, reproduced small.
    ///
    /// The orphan is the other half: a plate no interval reaches is refused BY NAME. Reading it as
    /// delivered would put an uncorrected fraction in the same stored set as corrected ones, and
    /// `band_missed` — which only fires on a corrected plate — would quietly switch off for it.
    #[test]
    #[ignore = "needs numpy and Pillow"]
    fn each_interval_is_corrected_onto_its_own_reference() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-TS-4", None, None, None).unwrap();
        let w = wid.to_string();

        // Lamp A: warm brown rock at hue 20, epoxy at 229.
        const MATRIX_A: (u8, u8, u8) = (120, 80, 60);
        const PORE_A: (u8, u8, u8) = (40, 60, 150);
        // Lamp B: a green cast, rock at hue 141, epoxy still inside the band at 202.
        const MATRIX_B: (u8, u8, u8) = (60, 130, 90);
        const PORE_B: (u8, u8, u8) = (40, 100, 150);

        let quarter = |a: (u8, u8, u8), b: (u8, u8, u8)| {
            bmp(200, 200, move |x, y| if x < 100 && y < 100 { b } else { a })
        };
        let mk = |name: &str, depth: f32, data: Vec<u8>| crate::db::NewImage {
            depth_top: depth,
            name: name.into(),
            mime: "image/bmp".into(),
            width: 200,
            height: 200,
            data,
            printable: true,
            prepared: Some("blue_epoxy".into()),
            ..Default::default()
        };
        crate::db::insert_well_images(
            &conn,
            &w,
            "THIN SECTION",
            "LAB",
            None,
            &[
                mk("SHREF", 2000.0, quarter(MATRIX_A, PORE_A)),
                mk("SHMEM", 2001.0, quarter(MATRIX_A, PORE_A)),
                mk("ORPHAN", 2500.0, quarter(MATRIX_A, PORE_A)),
                mk("DPREF", 3000.0, quarter(MATRIX_B, PORE_B)),
                mk("DPMEM", 3001.0, quarter(MATRIX_B, PORE_B)),
            ],
        )
        .unwrap();
        let ids = crate::db::list_well_images(&conn, &w, Some("THIN SECTION")).unwrap();
        let id_of = |n: &str| ids.iter().find(|i| i.name == n).map(|i| i.image_id.clone()).unwrap();

        let mut s = zoned(None, Vec::new());
        s.well_id = w.clone();
        s.dataset = "THIN SECTION".into();

        // --- one reference for the whole delivery: the deep interval is lost -------------------
        let mut wide = s.clone();
        wide.reference_image_id = Some(id_of("SHREF"));
        let one = run_pore_area(&conn, &wide).expect("run");
        let find = |r: &PoreResult, n: &str| {
            r.plates.iter().find(|p| p.name == n).unwrap_or_else(|| panic!("{n}")).clone()
        };
        let deep_one = find(&one, "DPMEM");
        assert!(
            deep_one.cast_shift > 100.0,
            "the deep sections had to move a long way to reach a shallow reference: {}",
            deep_one.cast_shift
        );
        assert!(deep_one.band_missed, "and the band did not survive the trip: {}", deep_one.pore_fraction);

        // --- an interval each: every section corrected onto its own lamp ------------------------
        let mut split = s.clone();
        split.reference_zones = vec![
            zone(None, Some(2100.0), &id_of("SHREF")),
            zone(Some(2900.0), None, &id_of("DPREF")),
        ];
        let two = run_pore_area(&conn, &split).expect("run");

        for (name, reference) in [("SHREF", "SHREF"), ("SHMEM", "SHREF"), ("DPREF", "DPREF"), ("DPMEM", "DPREF")] {
            let p = find(&two, name);
            assert_eq!(p.reference_name, reference, "{name} was corrected onto the wrong plate");
            assert!(p.cast_shift.abs() < 5.0, "{name} shift {}", p.cast_shift);
            assert!(
                (p.pore_fraction - 0.25).abs() < 0.02,
                "{name} reads {} where the rock is a quarter pore",
                p.pore_fraction
            );
            assert!(!p.band_missed, "{name} was measured");
        }

        // The orphan sits between the two intervals and no delivery-wide plate was named, so it is
        // named in the refusals rather than being read as delivered.
        assert!(two.plates.iter().all(|p| p.name != "ORPHAN"), "an uncovered plate is not measured");
        assert!(
            two.skipped.iter().any(|s| s.contains("ORPHAN") && s.contains("no reference plate covers")),
            "{:?}",
            two.skipped
        );
    }
}

/// The whole road, on a real delivery: a workbook of plates in, a number checked against an
/// independent measurement out.
#[cfg(test)]
mod field_tests {
    use super::*;

    /// Workbook -> plates -> pore area -> checked against the petrographer's own point count.
    ///
    /// Every increment so far has been verified against synthetic plates, which can only ever
    /// prove the arithmetic. This one asks the question the arithmetic cannot: does the automatic
    /// measurement agree with a human who counted the same sections by eye?
    ///
    /// The comparison is deliberately the POINT COUNT rather than helium porosity. A plug's helium
    /// porosity and a section's area fraction differ for two reasons at once — the measurement and
    /// the depth registration — and a disagreement could not be attributed to either. The
    /// petrographer counted the SAME picture, so only the measurement is under test.
    ///
    /// Runs only when `SANDIBUMI_FIELD_FIXTURES` names a folder holding
    /// `workbooks/` (the delivered `.xlsx`) and `petrography/` (a delimited table with a WELL
    /// column, a depth and the counted porosity). SKIPS with a printed reason otherwise.
    #[test]
    #[ignore = "needs a real petrography delivery; set SANDIBUMI_FIELD_FIXTURES"]
    fn a_delivered_book_measures_against_the_petrographers_own_point_count() {
        let Some(root) = crate::field_fixtures::root() else {
            eprintln!("SKIP: set SANDIBUMI_FIELD_FIXTURES to a folder with workbooks/ and petrography/");
            return;
        };
        let books: Vec<String> = match std::fs::read_dir(root.join("workbooks")) {
            Ok(rd) => rd
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .is_some_and(|e| e.eq_ignore_ascii_case("xlsx"))
                })
                .map(|p| p.to_string_lossy().into_owned())
                .collect(),
            Err(_) => {
                eprintln!("SKIP: no workbooks/ under {}", root.display());
                return;
            }
        };
        let counts: Vec<String> = match std::fs::read_dir(root.join("petrography")) {
            Ok(rd) => rd
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .is_some_and(|e| e.eq_ignore_ascii_case("csv"))
                })
                .map(|p| p.to_string_lossy().into_owned())
                .collect(),
            Err(_) => {
                eprintln!("SKIP: no petrography/ under {}", root.display());
                return;
            }
        };
        if books.is_empty() || counts.is_empty() {
            eprintln!("SKIP: need at least one .xlsx and one .csv");
            return;
        }

        // --- 1. the plates come out of the book, depths and all -------------------------------
        let out = std::env::temp_dir().join("sandibumi_pore_e2e");
        let _ = std::fs::remove_dir_all(&out);
        let probe = crate::images::probe_plate_workbooks(&books, &out).expect("workbook read");
        eprintln!(
            "extracted {} plate(s) from {} book(s); unit {:?}; {} note(s)",
            probe.plates.len(),
            books.len(),
            probe.depth_unit,
            probe.notes.len()
        );
        for n in probe.notes.iter().take(6) {
            eprintln!("   note: {n}");
        }
        assert!(!probe.plates.is_empty(), "no plate came out of the delivery");

        // --- 2. a project holding one well --------------------------------------------------
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        // The delivery states feet and the point-count table is written in feet. Working in the
        // delivered unit keeps every depth in this test the number the laboratory wrote down.
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Feet).unwrap();
        let wid = uuid::Uuid::new_v4();
        let well_name = "SANDI-TS-1";
        crate::db::insert_well(&conn, wid, well_name, None, None, None).unwrap();
        let w = wid.to_string();

        // --- 3. the ordinary importer takes them --------------------------------------------
        //
        // `prepared` is DECLARED here, exactly as a user declares it in the wizard. It is a fact
        // about how the section was made and the picture cannot be asked — the evidence for "this
        // is blue epoxy" is the blue about to be measured.
        let items: Vec<crate::images::ImageImportItem> = probe
            .plates
            .iter()
            .filter_map(|p| {
                Some(crate::images::ImageImportItem {
                    path: p.path.clone(),
                    name: p.name.clone(),
                    depth_top: p.depth_top?,
                    depth_base: p.depth_base,
                    ..Default::default()
                })
            })
            .collect();
        assert!(!items.is_empty(), "no plate carried a depth from its sheet");
        let req = crate::images::ImageImportRequest {
            depth_datum: "MD".into(),
            well_id: w.clone(),
            dataset: "THIN SECTION".into(),
            set_name: "LAB".into(),
            depth_unit: probe.depth_unit.clone(),
            prepared: Some("blue_epoxy".into()),
            items,
            ..Default::default()
        };
        let imported = crate::images::import_images(&conn, &req).expect("image import");
        eprintln!("imported {} plate(s)", imported.imported);

        // --- 4. the measurement --------------------------------------------------------------
        // Spelled out rather than `..Default::default()`: `PoreSpec` deliberately has no Default,
        // because a zeroed colour band would be a band nobody chose.
        let spec = PoreSpec {
            well_id: w.clone(),
            dataset: "THIN SECTION".into(),
            band: PoreColorBand::default(),
            reference_image_id: None,
            reference_zones: Vec::new(),
            preview_image_id: None,
            only_image_id: None,
            set_name: Some("TS".into()),
            geometry: false,
            min_pore_px: MIN_PORE_PX,
            grains: false,
            min_grain_px: MIN_GRAIN_PX,
            grain_sep_px: GRAIN_SEP_PX,
            wicksell: false,
            stain: None,
            check_against: None,
            check_depth_tol: 0.0,
        };
        let res = run_pore_area(&conn, &spec).expect("pore run");
        let flagged = res.plates.iter().filter(|p| p.scene_dominated).count();
        let stored: Vec<f32> = res
            .plates
            .iter()
            .filter(|p| !p.scene_dominated)
            .map(|p| p.pore_fraction)
            .collect();
        eprintln!(
            "measured {} plate(s); {flagged} flagged as scene-dominated; {} stored",
            res.plates.len(),
            stored.len()
        );
        for n in &res.notes {
            eprintln!("   note: {n}");
        }
        assert!(!stored.is_empty(), "the whole delivery was refused");

        // The delivery's own spread of light, which is what the correction exists for.
        let mut hues: Vec<(f32, String, String)> = res
            .plates
            .iter()
            .filter(|p| p.scene_hue.is_finite() && !p.scene_dominated)
            .map(|p| (p.scene_hue, p.image_id.clone(), p.name.clone()))
            .collect();
        hues.sort_by(|a, b| a.0.total_cmp(&b.0));
        if let (Some(lo), Some(hi)) = (hues.first(), hues.last()) {
            eprintln!("plate median hue spans {:.0} to {:.0} deg", lo.0, hi.0);
        }

        // --- 5. the independent measurement, imported the ordinary way -----------------------
        //
        // Its OWN dataset, not PETROGRAPHY: exactly one delivery per (well, dataset) is live, so
        // writing the point count into the same dataset would switch off the measurement it is
        // meant to check.
        let mut counted = 0usize;
        for path in &counts {
            let r = crate::ingest::import_aux_file(&conn, &w, "POINTCOUNT", path, Some("LAB"), false, "MD", None);
            assert!(r.error.is_none(), "point-count import failed: {:?}", r.error);
            counted += r.rows;
        }
        eprintln!("imported {counted} point-count row(s)");
        assert!(counted > 0, "the point-count table brought in nothing");

        // --- 6. the two measurements meet ----------------------------------------------------
        let qc = crate::plugqc::run_plug_qc(
            &conn,
            &crate::plugqc::PlugQcRequest {
                well_ids: vec![w.clone()],
                x: crate::plugqc::PlugSource {
                    kind: "aux".into(),
                    dataset: "POINTCOUNT".into(),
                    item: "VISPOR_PC".into(),
                    saturation: 0.0,
                },
                y: crate::plugqc::PlugSource {
                    kind: "aux".into(),
                    dataset: PORE_DATASET.into(),
                    item: PORE_ITEM.into(),
                    saturation: 0.0,
                },
                depth_tol: 0.15,
            },
        )
        .expect("plug qc");

        eprintln!(
            "paired {} plug(s); point count median {:.2}, measured median {:.4}",
            qc.n_pairs, qc.x_median, qc.y_median
        );
        eprintln!("   pearson {:.3}  spearman {:.3}", qc.pearson, qc.spearman);
        for (why, n) in &qc.excluded {
            eprintln!("   excluded {n}: {why}");
        }
        for n in &qc.notes {
            eprintln!("   note: {n}");
        }

        // The claim under test is that the two measurements are OF THE SAME THING. That is a
        // pairing question, not a correlation threshold — a delivery where nothing pairs has not
        // been checked at all, whatever the coefficient says.
        assert!(qc.n_pairs > 0, "not one plate met its own point count");

        // --- 7. and again, corrected onto a reference plate -----------------------------------
        //
        // The same comparison with the colour correction on, which is the only way to say whether
        // it moved the answer on real rock. The reference is chosen by a STATED rule rather than
        // by whichever gives the best number: the plate whose own median hue is nearest the
        // delivery's median — the most typical light, and the choice that leaves every other plate
        // the shortest distance to travel. Picking the reference that maximises agreement would be
        // fitting the answer, which is the exact mistake this delivery already taught once.
        //
        // Several candidates are run because "does the choice matter?" is an open question and the
        // only way to answer it is to vary it. Nothing here is asserted — a correlation threshold
        // on somebody's rock is not a property of this code.
        let pick = |q: f32| -> Option<(f32, String, String)> {
            if hues.is_empty() {
                return None;
            }
            let i = (((hues.len() - 1) as f32) * q).round() as usize;
            Some(hues[i].clone())
        };
        eprintln!("--- with the colour correction on ---");
        for (label, q) in [("p10", 0.10), ("p30", 0.30), ("p50", 0.50), ("p70", 0.70), ("p90", 0.90)] {
            let Some((hue, id, name)) = pick(q) else { continue };
            let mut s2 = spec.clone();
            s2.reference_image_id = Some(id);
            // A fresh delivery name each time: exactly one point-data set per (well, dataset) is
            // live, so each run supersedes the last and the pairing below reads the newest.
            s2.set_name = Some(format!("TS_{label}"));
            let r2 = match run_pore_area(&conn, &s2) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{label}: reference {name} refused - {e}");
                    continue;
                }
            };
            let missed = r2.plates.iter().filter(|p| p.band_missed).count();
            let shift_max = r2
                .plates
                .iter()
                .map(|p| p.cast_shift)
                .filter(|s| s.is_finite())
                .fold(0.0f32, f32::max);
            let q2 = crate::plugqc::run_plug_qc(
                &conn,
                &crate::plugqc::PlugQcRequest {
                    well_ids: vec![w.clone()],
                    x: crate::plugqc::PlugSource {
                        kind: "aux".into(),
                        dataset: "POINTCOUNT".into(),
                        item: "VISPOR_PC".into(),
                        saturation: 0.0,
                    },
                    y: crate::plugqc::PlugSource {
                        kind: "aux".into(),
                        dataset: PORE_DATASET.into(),
                        item: PORE_ITEM.into(),
                        saturation: 0.0,
                    },
                    depth_tol: 0.15,
                },
            )
            .expect("plug qc");
            eprintln!(
                "{label} ref {name} (hue {hue:.0}): {} refused as empty, max shift {shift_max:.0} deg, \
                 {} pairs, measured median {:.4}, pearson {:.3} spearman {:.3}",
                missed, q2.n_pairs, q2.y_median, q2.pearson, q2.spearman
            );
        }
        let _ = std::fs::remove_dir_all(&out);
    }
}
