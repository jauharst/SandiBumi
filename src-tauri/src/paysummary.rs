//! Pay summary: cutoffs, flags and per-zone statistics.
//!
//! AUDIT-2026-08-20 finding 49. This was the lower 2,082 lines of `workflow.rs`, which had grown
//! to 16,154 lines carrying two subsystems that share a file and almost nothing else. The
//! dependency between them is real, ONE-WAY, and a single function wide: this half calls
//! `workflow::first_available_input_alias` to resolve a porosity curve, and nothing in the runner
//! names anything here.
//!
//! That count is the compiler's, and it corrects an earlier measurement of mine. A text sweep for
//! the seam reported this half calling `run_workflow_module` as well; the one hit was inside a
//! COMMENT ("same graceful degradation as run_workflow_module"), and the import is unused. A
//! grep counts occurrences, a compiler counts calls, and only the second answers "what does this
//! depend on". The narrower true seam is what makes the split cheap rather than risky.
//!
//! Nothing is re-exported from `workflow`. A glob re-export would have made every existing
//! `workflow::PaySummaryRow` keep compiling, which is exactly the problem - one item reachable
//! under two names, and a reader of the call site with no way to tell which file to open.

use crate::db;
use crate::equations;
use crate::modules;
use crate::workflow::first_available_input_alias;
use duckdb::Connection;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::sync::Mutex;

#[derive(Debug, Clone, Deserialize)]
pub struct PaySummaryRequest {
    pub well_ids: Vec<String>,
    /// SB-CUT-001 (DEC-071): the thickness discretisation model. Defaults to CENTRED per
    /// the ruling; FORWARD ("TOPS") stays selectable to reproduce a legacy run's numbers.
    #[serde(default)]
    pub discretisation: DiscretisationModel,
    /// SB-CUT-016. VSH <= vsh_max counts as sand. **`None` means UNFILTERED** — the property is
    /// not used to exclude anything, and the result says so. There is deliberately no default:
    /// four shipped vendor sets disagree, two of them from one vendor, and Jauhar's own delivered
    /// work spans Vsh 0.20-0.85 across intervals of a single area.
    /// SB-CUT-019: carried AS ENTERED, with its unit, and canonicalised on receipt. A bare
    /// number is refused rather than guessed at.
    pub vsh_max: Option<CutoffSpec>,
    /// SB-CUT-016. PHIE >= phie_min counts as reservoir (with sand). `None` = unfiltered.
    pub phie_min: Option<CutoffSpec>,
    /// SB-CUT-016. SWE <= swe_max counts as pay (with reservoir). `None` = unfiltered.
    pub swe_max: Option<CutoffSpec>,
    /// PERM >= perm_min added to the pay flag when PERM exists. `None` = unfiltered.
    pub perm_min: Option<CutoffSpec>,
    /// SB-CUT-016. Cut-offs the caller switched ON and left without a value. A summation **MUST
    /// NOT** run against one, so any name here refuses the whole request.
    ///
    /// Separate from a `None` value on purpose: *"I am not filtering on Sw"* and *"I meant to
    /// filter on Sw and have not said what"* are different statements, and only one of them may
    /// produce a number. `#[serde(default)]`, so every record written before this existed still
    /// deserializes and still means what it meant.
    #[serde(default)]
    pub enabled_unset: Vec<String>,
    /// Read the curves this run consumes from THIS log set's stored values (latest version per
    /// well) rather than from whatever the current values are. Curves the set never wrote fall
    /// back to normal resolution; an empty name means "current values", which is what every
    /// caller did before this existed (Jauhar, 2026-08-05).
    #[serde(default)]
    pub input_set: Option<String>,
    /// SB-CUT-009. Per-curve averaging weighting, keyed by the SLOT the curve fills — one of
    /// [`AVERAGED_SLOTS`], a role rather than a mnemonic. Absent slots take [`default_weighting`],
    /// so a caller who declares nothing gets exactly the behaviour that shipped before this
    /// existed. Persisted with the rest of the run's configuration in `log_sets.params_json`,
    /// which is what makes it *stored with the curve's averaging configuration* rather than an
    /// argument that evaporates after the run.
    #[serde(default)]
    pub weighting: BTreeMap<String, AverageWeighting>,
    /// SB-CUT-022. Which report tiers each cut-off is USED at, keyed by SLOT. An absent slot takes
    /// [`default_cutoff_use`], which is the ladder that shipped before this existed — so a caller
    /// who declares nothing sees no number move. Persisted with the rest of the run's
    /// configuration, which is what makes the activation auditable FROM A RESULT rather than
    /// re-derivable only by knowing which rule the engine happened to apply.
    #[serde(default)]
    pub cutoff_use: BTreeMap<String, CutoffUse>,
    /// SB-CUT-012. The depth frame to summate in. Defaults to MD, which is the only frame
    /// SandiBumi can currently weight; any other is REFUSED rather than served MD numbers under
    /// a different label.
    #[serde(default)]
    pub frame: SummationFrame,
    /// RETAINED AS A REFUSAL, not as a switch. It once meant "write FLAG_* in place without
    /// versioning", set by the report/composite render pass so a render would not churn the
    /// archive. That behaviour is gone: a pay flag with no ancestry cannot say which cutoffs
    /// produced it, so a run that asks for one is REFUSED BY NAME rather than served. The field
    /// stays on the wire so a saved or older caller that still sets it gets that named refusal
    /// instead of a deserialization error. A render pass that wants no flags written sets
    /// [`PaySummaryRequest::stats_only`] instead, which is what every in-repo caller does.
    #[serde(default)]
    pub skip_version: bool,
    /// When true, compute and return the per-zone statistics WITHOUT persisting any FLAG_*
    /// curves at all. The Field Dashboard sets this: it recomputes on every cutoff tweak and
    /// only consumes the returned rows, so writing 3 FLAG curves × every well each refresh
    /// (~1,600 delete+append+flush transactions on 540 wells) was pure waste that dominated
    /// its runtime. Persisting flags stays the job of the explicit Cutoffs & Summary run.
    #[serde(default)]
    pub stats_only: bool,
    #[serde(default)]
    pub custody: Option<crate::ancestry::RunCustody>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaySummaryRow {
    pub well_id: String,
    pub well_name: String,
    pub zone: String,
    pub flag: String, // SAND | RESERVOIR | PAY
    pub top: f32,
    pub bottom: f32,
    pub gross: f32,
    pub net: f32,
    /// SB-CUT-002: the discretisation model this row's thicknesses were computed under. A
    /// consumer must never have to infer it — two tools disagreeing by half a step at every
    /// zone contact both print plausible nets.
    pub discretisation_model: String,
    /// SB-CUT-002: the sample interval (project depth unit) the summation ran on — the median
    /// forward step of this well's frame. Net-to-gross is not scale-invariant, so two rows
    /// computed at different steps are different statements even over the same rock.
    pub sample_interval: f32,
    /// SB-CUT-003. Footage the classifier EVALUATED and rejected — it saw the sample and the
    /// sample failed a cutoff.
    ///
    /// Kept strictly apart from [`Self::unknown`] because the two are the same number on a page
    /// and completely different rock. A zone reading 40 % net-to-gross because 60 % is shale and a
    /// zone reading 40 % because 55 % was never logged both print 0.40, and only the split says
    /// which. Techlog books a non-positive clipped interval as UNKNOWN distinct from NOT-NET; IP
    /// marks nulls in-band inside the numeric column and never separates them at all.
    pub not_net: f32,
    /// SB-CUT-003. Footage whose flag could not be EVALUATED, so that
    /// `gross = net + not_net + unknown` holds exactly.
    ///
    /// **Derived rather than accumulated, and that is the substance of the requirement.** Two
    /// separate things make footage unjudgeable and only one of them is a sample: an in-zone
    /// sample whose VSH/PHIE/SWE are missing, and footage carrying no sample at all — a logging
    /// gap, or the ordinary case of a zone bottomed on a marker below the TD of the run that
    /// logged it. Summing the first alone would leave the identity broken over exactly the
    /// intervals where a reader most needs it to close.
    pub unknown: f32,
    /// SB-CUT-004. Net-to-gross over the footage the classifier could actually judge —
    /// `net / (gross - unknown)`, the chapter's `N:(G−Unknown)`.
    ///
    /// Reported BESIDE [`Self::ntg`] rather than instead of it, because the two answer different
    /// questions and the gap between them is the null fraction. Over a washed-out or
    /// partially-logged interval that gap is the whole argument about whether a net-to-gross is
    /// defensible; no incumbent surfaces both, so an interpreter comparing one tool's number with
    /// another's cannot tell which was quoted.
    ///
    /// **MISSING, never zero, where nothing was judged.** With no judged footage there is no
    /// denominator, and a printed 0.00 would be a claim about rock nobody looked at — the same
    /// reasoning as [`Self::n_classified`]. Crosses IPC as JSON `null`, like the `avg_*` fields.
    pub ntg_known: f32,
    /// SB-CUT-030. True when an emitted zonal average falls outside its quantity's physical
    /// bounds. The value is **emitted as computed, not corrected** — a corrected average is a
    /// number nobody derived, and the condition that produced it is exactly what a reviewer needs
    /// to see. It rides in its own typed field for the SB-CUT-029 reason: a marker inside the
    /// numeric column would stop being arithmetic.
    #[serde(default)]
    pub out_of_range: bool,
    /// SB-CUT-005. Footage moved into the largest component so the partition closes — reported
    /// rather than printed, which is the whole point of the requirement. Zero on any run whose
    /// partition already closed, which is every ordinary run; a non-zero value here is the record
    /// that a correction happened and how big it was.
    pub residual_absorbed: f32,
    /// SB-CUT-012. The depth frame these weights were measured in — part of the result's identity.
    /// An MD and a TVD summation are separate records, never one rescaled into the other.
    pub frame: SummationFrame,
    /// SB-CUT-012. What the per-sample weights were differenced from. Naming the frame alone does
    /// not say WHICH depths produced the increments.
    pub weights_source: String,
    /// SB-CUT-016. Cut-offs NOT applied to this summation, in VSH/PHIE/SWE/PERM order. An
    /// unfiltered summation must be reported AS unfiltered - a net that quietly stopped being
    /// filtered, with nothing on the result to say so, is the whole failure this prevents.
    pub unfiltered: Vec<String>,
    pub ntg: f32,
    pub avg_vsh: f32,
    pub avg_phie: f32,
    /// PHIE-weighted average SWE (pay-summary convention).
    pub avg_swe: f32,
    pub hpv: f32, // sum of PHIE*(1-SWE)*thickness over net
    /// In-zone samples the classifier could actually judge. **0 means the well was never
    /// interpreted** — VSH/PHIE/SWE resolved to all-NaN — as opposed to a genuine zero-net
    /// result, which the identical `net`/`ntg`/`hpv` zeros cannot distinguish on their own.
    /// Consumers must render "—" rather than 0.00 when this is 0.
    pub n_classified: usize,
    /// **A permeability cutoff is active and this well carries no PERM at all**, so every sample
    /// failed it for want of data and the zero below is an absence of evidence, not a dry zone.
    /// Per well, so it is the same on every zone row of that well.
    ///
    /// Jauhar's call, 2026-08-01 (`docs/review_triage.md` finding 7): *"no relation between em,
    /// wells still can have perm curves"* — whether a cutoff applies has no relation to whether
    /// this particular well was cored, and permeability can be MODELLED where it was not measured
    /// (`perm_coates`, `perm_timur`, the rocktyping family), so lacking a measured PERM is not a
    /// reason to be let off. The cutoff is now active whenever it is requested.
    ///
    /// That settles a rule the code used to hold in two contradictory halves. At the SAMPLE level
    /// a missing PERM correctly FAILED an active cutoff — confirmed `[x]` in `REVIEW.md` — but a
    /// well with no PERM anywhere switched the cutoff off for ITSELF one line earlier. Two wells
    /// of identical rock reported 0 and full net pay with `n_classified > 0` on both, and in a
    /// field roll-up they simply added together: **the less permeability data a well had, the more
    /// pay it booked.** The well-level test is gone and the sample-level rule now does the work.
    ///
    /// The flag survives the change with its meaning inverted, because the reader's problem is
    /// unchanged and only its direction moved. A well that books zero net pay across every zone
    /// looks exactly like a wet well; this is what says the interpretation never had the curve the
    /// cutoff asks about. **It means "a cutoff was requested and this well has nothing to answer it
    /// with", never "this well has no permeability"** — with no cutoff asked for there is nothing
    /// to report, and a flag that fired anyway would appear on every report anyone ever ran.
    #[serde(default)]
    pub perm_cutoff_no_data: bool,
    /// SB-POR-057 (DEC-070, RULED 2026-08-18: "quick look only shows pay summation as
    /// visual not pay curves"). **This well's porosity exists ONLY as the quick-look D-N
    /// comparison curve (`PHIE_DN_LIM`), and it was deliberately not summed** - the
    /// quick-look shortcuts may be OVERLAID on a display as a visual comparison, but never
    /// feed net/NTG/HPV. Without this mark the zeros on such a well read exactly like a
    /// wet well; with it a reader sees the curve existed and why it was refused. Supersedes
    /// the pay-eligible fallback DEC-042 shipped. Per well, like
    /// [`Self::perm_cutoff_no_data`]; false both when an authoritative `PHIE` was summed
    /// and when the well simply has no porosity at all - the flag means "present and
    /// excluded", never "absent".
    #[serde(default)]
    pub quicklook_phie_excluded: bool,
}

const SUMMARY_FLAGS: [&str; 3] = ["SAND", "RESERVOIR", "PAY"];

/// SB-CUT-019. The quantity a cut-off constrains, which fixes both its canonical unit and the
/// physical range it cannot leave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutoffQuantity {
    /// A volume fraction: Vsh, porosity, saturation. Canonical `v/v`, bounded 0..=1.
    VolumeFraction,
    /// Permeability. Canonical `mD`, bounded to non-negative.
    Permeability,
}

impl CutoffQuantity {
    pub fn canonical_unit(self) -> &'static str {
        match self {
            CutoffQuantity::VolumeFraction => "v/v",
            CutoffQuantity::Permeability => "mD",
        }
    }
}

/// SB-CUT-019. A cut-off AS ENTERED — a number and the unit it was entered in.
///
/// The unit is not decoration. IP's own manual expresses the sensitivity-sweep example in porosity
/// units and the cut-off default in `v/v` **for the same quantity, with no unit tag on the field**.
/// Entering `35` where `0.1` is meant is a **350x** error whose symptom is an all-net result: a
/// good-looking well, not a visible failure. So a bare number is refused rather than guessed at.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CutoffEntry {
    pub value: f64,
    /// The unit the user typed. Empty is a REFUSAL, never an assumption.
    pub unit: String,
}

/// A bare number on the wire still DESERIALIZES — it becomes an entry with an empty unit, which
/// then fails [`CutoffEntry::canonical`] with the message that names the field and says why.
///
/// Deliberate: refusing at the parse layer would return serde's *invalid type* text, which tells
/// an analyst nothing about porosity units, and would also break every request shape written
/// before this existed. The value is rejected either way; this controls WHICH message they get.
impl<'de> Deserialize<'de> for CutoffEntry {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            Bare(f64),
            Tagged { value: f64, unit: String },
        }
        Ok(match Wire::deserialize(deserializer)? {
            Wire::Bare(value) => CutoffEntry { value, unit: String::new() },
            Wire::Tagged { value, unit } => CutoffEntry { value, unit },
        })
    }
}

impl CutoffEntry {
    /// Convert to the quantity's canonical unit, refusing a bare number, an unknown unit, and a
    /// value outside the quantity's physical range.
    ///
    /// `35 pu` becomes `0.35 v/v`; `35 v/v` is refused as out of bounds - the same number, and
    /// only the unit says which of those two the user meant.
    pub fn canonical(&self, quantity: CutoffQuantity, label: &str) -> Result<f64, String> {
        let unit = self.unit.trim();
        if unit.is_empty() {
            return Err(format!(
                "{label} was entered as a bare number ({}) with no unit. A porosity cut-off \
                 typed as 35 is 0.35 in porosity units and impossible in v/v, and the 350x \
                 error looks like an all-net well rather than a failure - so state the unit.",
                self.value
            ));
        }
        if !self.value.is_finite() {
            return Err(format!("{label} is not a finite number"));
        }
        let lower = unit.to_ascii_lowercase();
        let canonical = match quantity {
            CutoffQuantity::VolumeFraction => match lower.as_str() {
                "v/v" | "frac" | "fraction" | "dec" => self.value,
                "pu" | "p.u." | "%" | "pct" | "percent" => self.value / 100.0,
                _ => {
                    return Err(format!(
                        "{label} is in '{unit}', which is not a unit of volume fraction. \
                         Use v/v, pu or %."
                    ))
                }
            },
            CutoffQuantity::Permeability => match lower.as_str() {
                "md" => self.value,
                "d" | "darcy" => self.value * 1000.0,
                _ => {
                    return Err(format!(
                        "{label} is in '{unit}', which is not a unit of permeability. Use mD or D."
                    ))
                }
            },
        };
        let out_of_range = match quantity {
            CutoffQuantity::VolumeFraction => !(0.0..=1.0).contains(&canonical),
            CutoffQuantity::Permeability => canonical < 0.0,
        };
        if out_of_range {
            return Err(format!(
                "{label} is {} {unit}, which is {canonical} {} - outside the physical range \
                 of the quantity. A volume fraction cannot exceed 1; if porosity units were \
                 meant, enter the unit as pu.",
                self.value,
                quantity.canonical_unit()
            ));
        }
        Ok(canonical)
    }
}

/// SB-CUT-030. The three named stages a value passes through, and whether each clamps.
///
/// **`Accumulate` never clamps, and that is the whole requirement.** Clamping inside a sum is not
/// a display choice; it moves the MEAN. For a truly wet interval the unclamped hydrocarbon
/// contribution `phi*(1-Sw)` has expectation zero under symmetric noise, while the clamped
/// contribution `phi*max(0, 1-Sw)` has expectation `phi*sigma/sqrt(2*pi)` = `0.3989*phi*sigma > 0`
/// — always toward MORE hydrocarbon, by an amount independent of iteration count
/// (`docs/PRD_v2/14_cutoffs-summation-mc.md:789-794`). A clamp that is correct for one
/// deterministic evaluation is a bias in expectation over an ensemble.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClampStage {
    /// Summation. NEVER clamped.
    Accumulate,
    /// The cut-off comparison. Clamped to the quantity's bounds.
    FlagTest,
    /// What a reader is shown. Clamped to the quantity's bounds.
    Present,
}

/// SB-CUT-030. A quantity's physical bounds — **attached to the QUANTITY, never to a curve-type
/// string**.
///
/// Binding bounds to a type string is the specific failure that makes IP's clipping worse than
/// Techlog's unconditional clamp: IP clips by *declared curve type*, so mis-typing a curve silently
/// changes its numerics, and the change is **invisible in the data**. A quantity cannot be
/// mis-typed by a label because it is not a label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundedQuantity {
    /// A volume fraction: Vsh, porosity, saturation. Bounded `0..=1`.
    VolumeFraction,
    /// Permeability. Bounded below at zero and **unbounded above**.
    Permeability,
    /// A quantity with no physical bounds at all — a reconstruction error, a resistivity, a
    /// coefficient. It must NOT be clamped to `[0,1]` merely because that is the common case.
    Unbounded,
}

impl BoundedQuantity {
    /// The bounds, or `None` where the quantity has none. An open upper bound is `f64::INFINITY`
    /// rather than a large number, so nothing accidentally clips a real permeability.
    pub fn bounds(self) -> Option<(f64, f64)> {
        match self {
            BoundedQuantity::VolumeFraction => Some((0.0, 1.0)),
            BoundedQuantity::Permeability => Some((0.0, f64::INFINITY)),
            BoundedQuantity::Unbounded => None,
        }
    }

    /// Whether a value lies outside the quantity's bounds. A NaN is not out of range — it is
    /// absent, which is a different statement and already has its own carrier (SB-CUT-029).
    pub fn is_out_of_range(self, value: f32) -> bool {
        match self.bounds() {
            // NaN is not out of range: absent is a different statement and has its own carrier
            // (SB-CUT-029). An INFINITY is - it lies outside any finite bound - and the blanket
            // `is_finite` guard this used to open with answered no for it, so an infinite average
            // was reported unflagged and clamped to a clean-looking 1.0.
            Some(_) if value.is_nan() => false,
            Some((lo, hi)) => (value as f64) < lo || (value as f64) > hi,
            None => false,
        }
    }
}

/// SB-CUT-030. Apply one stage's clamping policy to one value of one quantity.
///
/// The single place the policy is expressed, so `accumulate` cannot quietly acquire a clamp in one
/// caller while the others keep theirs.
pub fn stage_value(stage: ClampStage, quantity: BoundedQuantity, value: f32) -> f32 {
    match (stage, quantity.bounds()) {
        // Never, for any quantity. This arm is the requirement.
        (ClampStage::Accumulate, _) => value,
        // An unbounded quantity is not clamped at any stage — there is nothing to clamp it to.
        (_, None) => value,
        (_, Some((lo, hi))) => {
            if value.is_nan() {
                value
            } else {
                value.clamp(lo as f32, hi as f32)
            }
        }
    }
}

/// SB-CUT-020. Which side of a bound a sample sitting exactly ON it falls.
///
/// Spelled as words rather than as `>=` / `>`, because a symbol on the wire invites parsing and
/// this is the one field where a misread is invisible: it changes the verdict only for samples
/// exactly on the cut-off, which is exactly the population a marginal-pay result turns on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BoundOperator {
    /// A sample exactly equal to the bound is INSIDE. `x >= min`, `x <= max`.
    #[default]
    Inclusive,
    /// A sample exactly equal to the bound is OUTSIDE. `x > min`, `x < max`.
    Exclusive,
}

/// SB-CUT-020. One side of a cut-off range, in the quantity's canonical unit.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CutoffBound {
    pub value: f64,
    pub operator: BoundOperator,
}

/// SB-CUT-020. Which side of a range a single-sided cut-off occupies.
///
/// A slot named `phie_min` has always meant *at least this*, and `vsh_max` *at most this*. The
/// sense is the slot's, not the value's, so the degenerate form cannot land on the wrong side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutoffSense {
    Minimum,
    Maximum,
}

/// SB-CUT-020. A cut-off as a two-sided range — **and this doc comment is the specification that
/// SB-CUT-T24 tests against**, deliberately, because the vendor cannot be the oracle: Techlog
/// documents its modes 2 and 3 as outside tests and implements them as inside tests.
///
/// **The specification.** A sample value `x` passes the cut-off when it satisfies BOTH bounds:
///
/// | Side | Operator | Passes when | A sample exactly on the bound |
/// |---|---|---|---|
/// | low  | `INCLUSIVE` | `x >= value` | **inside** |
/// | low  | `EXCLUSIVE` | `x > value`  | **outside** |
/// | high | `INCLUSIVE` | `x <= value` | **inside** |
/// | high | `EXCLUSIVE` | `x < value`  | **outside** |
/// | either | *absent* | always | *not applicable — the far bound is open* |
///
/// An absent bound is an OPEN far bound, satisfied by every value. The single-sided `>=` / `<=`
/// forms are therefore this range with one side absent and the other `INCLUSIVE` — the degenerate
/// case, not a separate mechanism, so a project saved before ranges existed classifies identically.
///
/// `INCLUSIVE` is the default on both sides for the same reason: it is what the single-sided forms
/// have always meant, and a generalisation that silently moved the boundary would rewrite every
/// existing marginal result.
///
/// A range that can admit no value is REFUSED rather than quietly booking zero net — see
/// [`CutoffSpec::canonical`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Default)]
pub struct CutoffRange {
    pub low: Option<CutoffBound>,
    pub high: Option<CutoffBound>,
}

impl CutoffRange {
    /// The specification above, and the only place a cut-off comparison is made.
    ///
    /// **The comparison happens in `f32`, the precision the DATA has, and that is required rather
    /// than convenient.** A continuous log is `f32` (collaboration rule 2) while a cut-off is
    /// entered as a decimal and held as `f64`. Widen the sample instead and `0.30f32` becomes
    /// `0.30000001192…`, which is strictly GREATER than `0.30f64` — so a sample the user entered
    /// `0.30` to sit exactly on never sits on it, and the EXCLUSIVE operator silently excludes
    /// nothing at all. That is Techlog's mode 7 arrived at by arithmetic instead of by a bug.
    /// Narrowing the bound instead compares two numbers the data can actually distinguish, which
    /// is the only reading under which "exactly equal to the bound" means anything.
    pub fn contains(&self, sample: f32) -> bool {
        // A NaN satisfies no comparison, which is the honest answer: an unmeasured sample cannot
        // demonstrate that it passes. The callers handle missing data before reaching here.
        let low_ok = match self.low {
            Some(CutoffBound { value, operator: BoundOperator::Inclusive }) => sample >= value as f32,
            Some(CutoffBound { value, operator: BoundOperator::Exclusive }) => sample > value as f32,
            None => true,
        };
        let high_ok = match self.high {
            Some(CutoffBound { value, operator: BoundOperator::Inclusive }) => sample <= value as f32,
            Some(CutoffBound { value, operator: BoundOperator::Exclusive }) => sample < value as f32,
            None => true,
        };
        low_ok && high_ok
    }
}

/// SB-CUT-020. One side of a cut-off range AS ENTERED — a value, its unit, and its operator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CutoffSpecBound {
    #[serde(flatten)]
    pub entry: CutoffEntry,
    #[serde(default)]
    pub operator: BoundOperator,
}

/// SB-CUT-020. A cut-off as it arrives on the wire: a bare number, a `{value, unit}` entry, or a
/// `{min, max}` range with a per-bound operator.
///
/// The first two forms are the degenerate single-sided case and are accepted unchanged, so every
/// caller written before ranges existed keeps working and keeps meaning what it meant.
#[derive(Debug, Clone, PartialEq)]
pub struct CutoffSpec {
    pub min: Option<CutoffSpecBound>,
    pub max: Option<CutoffSpecBound>,
    /// The single-sided form, held until the slot's [`CutoffSense`] says which side it belongs to.
    pub single: Option<CutoffSpecBound>,
}

/// Serialization is the INVERSE of deserialization, deliberately: a persisted run has to reload
/// as the cut-off it was. A degenerate single-sided spec therefore writes the same object it
/// arrived as - now carrying its operator - and a range writes `{min, max}` with an absent side
/// omitted rather than written as a null.
impl Serialize for CutoffSpec {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        match (&self.min, &self.max, &self.single) {
            (_, _, Some(single)) => single.serialize(serializer),
            (min, max, None) => {
                let mut map = serializer.serialize_map(None)?;
                if let Some(bound) = min {
                    map.serialize_entry("min", bound)?;
                }
                if let Some(bound) = max {
                    map.serialize_entry("max", bound)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for CutoffSpec {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            // NARROWEST FIRST, and it has to be. Both of `Range`'s fields are optional, so that
            // arm matches ANY object — including `{value, unit}`, which it would silently accept
            // as a range with no bounds at all: a cut-off that filters nothing, configured. That
            // is Techlog's mode 7 reproduced by an ordering mistake, so `Single` is tried first.
            // A `{min, max}` object carries no `value` field, so it cannot match `Single`.
            Single(CutoffSpecBound),
            /// A bare number is not a map, so it cannot reach the flattened `Single` arm - and
            /// SB-CUT-019 requires it to PARSE and then be refused by name rather than returning
            /// serde's *invalid type* text. It becomes a unitless single bound, which `canonical`
            /// rejects with the message about porosity units.
            Bare(f64),
            Range {
                #[serde(default)]
                min: Option<CutoffSpecBound>,
                #[serde(default)]
                max: Option<CutoffSpecBound>,
            },
        }
        Ok(match Wire::deserialize(deserializer)? {
            Wire::Range { min, max } => CutoffSpec { min, max, single: None },
            Wire::Single(single) => CutoffSpec { min: None, max: None, single: Some(single) },
            Wire::Bare(value) => CutoffSpec {
                min: None,
                max: None,
                single: Some(CutoffSpecBound {
                    entry: CutoffEntry { value, unit: String::new() },
                    operator: BoundOperator::default(),
                }),
            },
        })
    }
}

/// SB-CUT-020. The degenerate case, as a conversion: a single entered value becomes the
/// single-sided range whose far bound is open and whose operator is `INCLUSIVE`.
///
/// The same statement the wire form makes, available in Rust so a caller that already holds an
/// entry does not have to spell the range out and risk spelling it differently.
impl From<CutoffEntry> for CutoffSpec {
    fn from(entry: CutoffEntry) -> Self {
        CutoffSpec {
            min: None,
            max: None,
            single: Some(CutoffSpecBound { entry, operator: BoundOperator::default() }),
        }
    }
}

impl CutoffSpec {
    /// Convert to canonical units and resolve the slot's sense, refusing anything the range
    /// specification cannot mean.
    pub fn canonical(
        &self,
        quantity: CutoffQuantity,
        sense: CutoffSense,
        label: &str,
    ) -> Result<CutoffRange, String> {
        let bound = |side: &Option<CutoffSpecBound>, side_label: &str| {
            side.as_ref()
                .map(|b| {
                    b.entry
                        .canonical(quantity, &format!("{label} {side_label}"))
                        .map(|value| CutoffBound { value, operator: b.operator })
                })
                .transpose()
        };
        let mut range = CutoffRange {
            low: bound(&self.min, "lower bound")?,
            high: bound(&self.max, "upper bound")?,
        };
        if let Some(single) = bound(&self.single, "")? {
            match sense {
                CutoffSense::Minimum => range.low = Some(single),
                CutoffSense::Maximum => range.high = Some(single),
            }
        }
        // A window nobody could have meant is refused, not run. Zero net from an inverted range
        // computes and plots exactly like zero net from tight rock, which is this row's risk class.
        if let (Some(low), Some(high)) = (range.low, range.high) {
            let empty = low.value > high.value
                || (low.value == high.value
                    && (low.operator == BoundOperator::Exclusive
                        || high.operator == BoundOperator::Exclusive));
            if empty {
                return Err(format!(
                    "{label} is the range {} to {}, which admits no value at all. A cut-off that \
                     cannot pass books zero net and looks exactly like tight rock, so it is \
                     refused rather than run.",
                    low.value, high.value
                ));
            }
        }
        Ok(range)
    }
}

/// SB-CUT-016. Render a cut-off for a deliverable: its value, or the word that says it was never
/// applied.
///
/// One helper rather than a spelling per surface. The two failures it exists to prevent are
/// printing nothing - a reader then assumes the cut-off was used - and printing a number that was
/// never applied, which is worse because it is checkable and wrong.
pub fn cutoff_label(value: Option<&CutoffSpec>, decimals: usize) -> String {
    // SB-CUT-019: the unit is printed WITH the number. A deliverable that says "PHIE >= 0.10"
    // without saying in what has reproduced the very ambiguity the entry rule exists to stop.
    let Some(spec) = value else {
        return "unfiltered".to_string();
    };
    // SB-CUT-020: a two-sided range prints in interval notation, where the bracket IS the operator
    // and an engineer reads it without a legend. The single-sided inclusive form keeps its bare
    // number, because that is what every existing deliverable shows and it has not changed meaning.
    match (&spec.min, &spec.max, &spec.single) {
        (_, _, Some(single)) => {
            let unit = single.entry.unit.trim();
            match single.operator {
                BoundOperator::Inclusive => format!("{:.decimals$} {unit}", single.entry.value),
                BoundOperator::Exclusive => {
                    format!("{:.decimals$} {unit} (exclusive)", single.entry.value)
                }
            }
        }
        (low, high, None) => {
            let unit = low
                .as_ref()
                .or(high.as_ref())
                .map(|b| b.entry.unit.trim().to_string())
                .unwrap_or_default();
            let open_bracket = match low {
                Some(b) if b.operator == BoundOperator::Exclusive => "(",
                Some(_) => "[",
                None => "(",
            };
            let close_bracket = match high {
                Some(b) if b.operator == BoundOperator::Exclusive => ")",
                Some(_) => "]",
                None => ")",
            };
            let lo = low
                .as_ref()
                .map(|b| format!("{:.decimals$}", b.entry.value))
                .unwrap_or_else(|| "-inf".into());
            let hi = high
                .as_ref()
                .map(|b| format!("{:.decimals$}", b.entry.value))
                .unwrap_or_else(|| "+inf".into());
            format!("{open_bracket}{lo}, {hi}{close_bracket} {unit}")
                .trim_end()
                .to_string()
        }
    }
}

/// SB-CUT-020. Render a cut-off as a PHRASE for running prose — `>= 0.10 mD`, `> 0.10 mD`,
/// `in [0.10, 0.35] v/v` — or the empty string when the cut-off was never applied.
///
/// Separate from [`cutoff_label`] because prose needs the comparison spelled out while a table cell
/// gets its sense from the row label. One helper rather than a spelling per surface: three call
/// sites used to hard-code `>=`, which a two-sided range or an exclusive bound makes untrue.
pub fn cutoff_phrase(value: Option<&CutoffSpec>, sense: CutoffSense, decimals: usize) -> String {
    let Some(spec) = value else {
        return String::new();
    };
    match (&spec.min, &spec.max, &spec.single) {
        (_, _, Some(single)) => {
            let comparison = match (sense, single.operator) {
                (CutoffSense::Minimum, BoundOperator::Inclusive) => ">=",
                (CutoffSense::Minimum, BoundOperator::Exclusive) => ">",
                (CutoffSense::Maximum, BoundOperator::Inclusive) => "<=",
                (CutoffSense::Maximum, BoundOperator::Exclusive) => "<",
            };
            format!(
                "{comparison} {:.decimals$} {}",
                single.entry.value,
                single.entry.unit.trim()
            )
        }
        _ => format!("in {}", cutoff_label(value, decimals)),
    }
}

/// SB-CUT-012. The depth frame a summation's per-sample weights were measured in.
///
/// Part of a result's IDENTITY, not a display option. The per-sample weight is `Δz` in MD and
/// `Δz·cos θ` in TVD, so the weights differ rather than merely the totals — in a 60° hold section
/// by a factor of two, which is why IP says TVD zonal averages *"could be considerably
/// different"*. A net thickness quoted in a deviated field without its frame is a number a reader
/// cannot use. Techlog offers four frames, IP two; the union is the vocabulary here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SummationFrame {
    /// Measured depth along hole — the log's own depth column, differenced.
    #[default]
    Md,
    /// True vertical depth.
    Tvd,
    /// True vertical depth subsea.
    Tvdss,
    /// True stratigraphic thickness.
    Tst,
}

impl SummationFrame {
    /// The chapter's own spelling of each frame. Its `match` is exhaustive, so it is also the
    /// compile-time guard: a fifth variant cannot be added without deciding here what it is
    /// called, which is a stronger guarantee than a list a test could let go stale.
    pub fn as_str(self) -> &'static str {
        match self {
            SummationFrame::Md => "MD",
            SummationFrame::Tvd => "TVD",
            SummationFrame::Tvdss => "TVDSS",
            SummationFrame::Tst => "TST",
        }
    }
}

/// SB-CUT-012. What an MD summation's weights were differenced from.
///
/// Recorded beside the frame because naming the frame alone does not say WHICH depths produced the
/// increments — the same reason a calibration records the curves it was fitted on.
pub const MD_WEIGHTS_SOURCE: &str = "log depth increments (MD)";

/// SB-CUT-009. How an averaged curve is weighted across a zone.
///
/// Declared per curve, never inferred from the curve's name or family. Techlog's own behaviour is
/// the harm the chapter names: *"the SW curve is weighted by POR but the SWE is not weighted"* —
/// a curve loses its φ-weighting because of how it happens to be spelled, and nothing on the page
/// says which form produced the number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AverageWeighting {
    /// `Σ(C·h) / Σh`
    Thickness,
    /// `Σ(C·φ·h) / Σ(φ·h)`
    Porosity,
}

/// SB-CUT-009. The curve slots the summation averages. A SLOT is a role fixed at compile time —
/// which input of the summation a curve fills — not the mnemonic it happens to be stored under.
pub const AVERAGED_SLOTS: [&str; 3] = ["VSH", "PHIE", "SWE"];

/// SB-CUT-022 / AUDIT-2026-08-20 finding 55. The four cut-off SLOTS, as a type.
///
/// These were `&str` and every match over them carried a catch-all arm - and the arm a typo or a
/// future fifth slot landed on was PERM, the one branch with teeth ("a requested permeability
/// cut-off is always active", so its absent-PERM samples FAIL rather than pass). A slot that
/// silently inherits the permeability cut-off is a well quietly losing pay. The requirement text
/// itself says a bound attaches "to the QUANTITY, never to a curve-type string".
///
/// As an enum every match is exhaustive and the fallback CANNOT BE WRITTEN - a fifth slot stops
/// the build at each place that has to decide what it means, which is the whole point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Slot {
    Vsh,
    Phie,
    Swe,
    Perm,
}

impl Slot {
    /// The wire spelling, which is what the request map is keyed by and what a result reports.
    pub(crate) fn as_wire(self) -> &'static str {
        match self {
            Slot::Vsh => "VSH",
            Slot::Phie => "PHIE",
            Slot::Swe => "SWE",
            Slot::Perm => "PERM",
        }
    }

    /// SB-CUT-030: which quantity's bounds a sample in this slot is clamped to before the
    /// FLAG_TEST comparison. Exhaustive on purpose - this is the decision a new slot must make.
    ///
    /// `Slot::Perm` does not currently REACH this: the permeability term in `classify_sample` is
    /// its own expression (missing PERM must fail rather than pass, which the other three slots
    /// have no equivalent of) and compares the raw value. That was equally true of the
    /// `if slot == "PERM"` conditional this replaces, so nothing changed - and it costs nothing,
    /// because permeability's bounds are (0, inf) and the only sample a clamp would move is a
    /// NEGATIVE permeability, which fails a positive cut-off either way. Stated rather than
    /// quietly dropped, so a fifth slot still has to answer the question.
    fn bounded_quantity(self) -> BoundedQuantity {
        match self {
            Slot::Perm => BoundedQuantity::Permeability,
            Slot::Vsh | Slot::Phie | Slot::Swe => BoundedQuantity::VolumeFraction,
        }
    }
}

/// SB-CUT-022 / AUDIT-2026-08-20 finding 55. The three report tiers, as a type - same reasoning as
/// [`Slot`]. The catch-all here resolved to PAY, so a mistyped tier silently judged the strictest
/// one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tier {
    Sand,
    Reservoir,
    Pay,
}

/// SB-CUT-022. Which report tiers a cut-off is USED at.
///
/// An explicit flag per tier, never an inference. F-17 is the reason: Geolog changed the activation
/// trigger between two modules of ONE product — `Determin` fires on the presence of the *curve*,
/// `determin_mc` on the presence of the *value*. Either rule is defensible; what is not defensible
/// is that a result cannot say which one applied, because an inference leaves no record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CutoffUse {
    pub sand: bool,
    pub reservoir: bool,
    pub pay: bool,
}

impl CutoffUse {
    /// Whether this cut-off's value is applied at one tier.
    fn at(&self, tier: Tier) -> bool {
        match tier {
            Tier::Sand => self.sand,
            Tier::Reservoir => self.reservoir,
            Tier::Pay => self.pay,
        }
    }
}

/// SB-CUT-022. The tiers a cut-off is used at when the caller declares nothing.
///
/// Cited, not chosen, and chosen to move no number: this is the ladder the engine already applied,
/// stated as flags instead of as nesting. Net sand is clay-driven, net reservoir adds porosity and
/// net pay adds saturation — T4 Bentley & Ringrose, `docs/PRD_v2/14_cutoffs-summation-mc.md:1296-1297`.
/// **`SWE` is off at the reservoir tier**, which is F-25 `:494-495`: IP's `Sw Net Use` and
/// `Sw Pay Use` are separate ordinals and Net Reservoir is described as porosity- and clay-driven.
pub(crate) fn default_cutoff_use(slot: Slot) -> CutoffUse {
    match slot {
        Slot::Vsh => CutoffUse { sand: true, reservoir: true, pay: true },
        Slot::Phie => CutoffUse { sand: false, reservoir: true, pay: true },
        Slot::Swe | Slot::Perm => CutoffUse { sand: false, reservoir: false, pay: true },
    }
}

/// SB-CUT-022. Resolve the tiers one cut-off is used at.
///
/// Takes a SLOT and the run's declaration — nothing else. It cannot see whether a curve exists or
/// whether a value was supplied, which is what makes *never inferred from the presence of a curve
/// or of a value* a property of the signature rather than of today's body.
pub(crate) fn cutoff_use_for(declared: &BTreeMap<String, CutoffUse>, slot: Slot) -> CutoffUse {
    declared.get(slot.as_wire()).copied().unwrap_or_else(|| default_cutoff_use(slot))
}

/// SB-CUT-022. The four cut-offs and the tiers each is used at, resolved once per run.
///
/// **One value per property, read by every tier that uses it.** That is F-25's shape exactly: IP
/// ships `Phi Cutoff` as a single ordinal *"for Pay and Reservoir report"* with `Phi Net Use` and
/// `Phi Pay Use` as two independent ordinals beside it. Two values would be a different product
/// and a different requirement — SB-CUT-024's, which owns arbitrary named tiers and their own
/// cut-off sets, and which is outside this gate.
/// Deliberately NOT `Default`: an all-false [`CutoffUse`] is a cut-off switched off everywhere,
/// which is a real and occasionally wanted state but a catastrophic thing to arrive at by
/// forgetting a field. Every construction names its four use declarations.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TierCutoffs {
    pub(crate) vsh: Option<CutoffRange>,
    pub(crate) phie: Option<CutoffRange>,
    pub(crate) swe: Option<CutoffRange>,
    pub(crate) perm: Option<CutoffRange>,
    pub(crate) vsh_use: CutoffUse,
    pub(crate) phie_use: CutoffUse,
    pub(crate) swe_use: CutoffUse,
    pub(crate) perm_use: CutoffUse,
}

impl TierCutoffs {
    /// The cut-off applied to one property at one tier: its value where the tier uses it, and
    /// `None` — which filters nothing — where the tier does not.
    fn applied(&self, tier: Tier, slot: Slot) -> Option<CutoffRange> {
        let (value, used) = match slot {
            Slot::Vsh => (self.vsh, self.vsh_use),
            Slot::Phie => (self.phie, self.phie_use),
            Slot::Swe => (self.swe, self.swe_use),
            Slot::Perm => (self.perm, self.perm_use),
        };
        used.at(tier).then_some(value).flatten()
    }
}

/// SB-CUT-009. The weighting applied when the caller declares nothing.
///
/// Cited, not chosen. The φ-weighted saturation `Σ(Sw·φ·h)/Σ(φ·h)` is agreed by all three vendors
/// and is required for SB-CUT-010's volumetric identity to hold at all
/// (`docs/PRD_v2/14_cutoffs-summation-mc.md:1041-1042`); thickness weighting for the rest is what
/// the engine already did, so a caller who declares nothing sees no number move.
pub fn default_weighting(slot: &str) -> AverageWeighting {
    if slot == "SWE" {
        AverageWeighting::Porosity
    } else {
        AverageWeighting::Thickness
    }
}

/// SB-CUT-009. Resolve the weighting for one averaged slot.
///
/// Takes a SLOT and the run's declaration — nothing else. It has no access to which curve filled
/// that slot, which is what makes "never inferred from the name" a property of the signature
/// rather than of the current implementation.
pub fn weighting_for(
    declared: &BTreeMap<String, AverageWeighting>,
    slot: &str,
) -> AverageWeighting {
    declared.get(slot).copied().unwrap_or_else(|| default_weighting(slot))
}

/// SB-CUT-009. One weighted average, accumulated sample by sample.
///
/// A sample joins the numerator AND the denominator together or not at all, so an average is
/// always normalised over exactly the footage its own curve was valid on — a SAND-row sample with
/// a good Vsh but a missing φ must not drag the porosity average toward zero.
#[derive(Debug, Default, Clone, Copy)]
struct WeightedMean {
    sum_wc: f64,
    sum_w: f64,
}

impl WeightedMean {
    /// `weight` is NaN where the weighting basis itself is missing — a φ-weighted average cannot
    /// use a sample with no porosity, however good its own value is.
    fn add(&mut self, value: f32, weight: f64) {
        if value.is_nan() || !weight.is_finite() {
            return;
        }
        self.sum_wc += value as f64 * weight;
        self.sum_w += weight;
    }

    fn value(&self) -> f32 {
        if self.sum_w > 0.0 {
            (self.sum_wc / self.sum_w) as f32
        } else {
            f32::NAN
        }
    }
}

/// SB-CUT-005. Relative tolerance on `gross - (net + not_net + unknown)`.
///
/// `1e-7`, cited: `docs/PRD_v2/14_cutoffs-summation-mc.md:2083` (SB-CUT-T22), which adopts
/// Techlog's `adjustFinal` reconciliation shape with the `print` → result-field refinement. It is
/// a NUMERICAL tolerance on closure arithmetic, not a petrophysical cutoff.
pub const PARTITION_TOLERANCE: f64 = 1e-7;
// SB-CUT-017: the registry entry `cut.partition_tolerance` carries this same number beside
// the citation that authorises it, and the named test asserts the two agree - so the
// disclosure cannot drift away from the behaviour.

/// SB-CUT-005. A footage partition that has been reconciled, and by how much.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReconciledPartition {
    pub net: f32,
    pub not_net: f32,
    pub unknown: f32,
    /// Footage moved into the largest component to make the partition close. **Reported, not
    /// printed** — that distinction IS the requirement. Techlog computes the same correction and
    /// sends it to a console, where it is lost, and a reconciliation whose correction is not
    /// recorded is indistinguishable from no reconciliation.
    pub absorbed: f32,
}

/// SB-CUT-005. A partition that does not close within [`PARTITION_TOLERANCE`], with every number a
/// reader needs to act on it. Structured rather than a bare message, for the same reason the
/// absorbed amount is a field: a diagnostic that only exists as prose cannot be checked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartitionResidual {
    pub gross: f32,
    pub net: f32,
    pub not_net: f32,
    pub unknown: f32,
    pub residual: f64,
    pub relative: f64,
    pub tolerance: f64,
}

impl std::fmt::Display for PartitionResidual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "footage partition does not close: gross {} against net {} + not-net {} + unknown {} \
             leaves a residual of {:e} ({:e} relative), outside the {:e} tolerance",
            self.gross, self.net, self.not_net, self.unknown, self.residual, self.relative,
            self.tolerance
        )
    }
}

/// SB-CUT-005. Check `gross - (net + not_net + unknown)` against [`PARTITION_TOLERANCE`], absorb a
/// residual within it into the LARGEST component, and report what was absorbed.
///
/// Evaluated in `f64` on the values as they will be REPORTED, so it checks the partition a reader
/// actually receives rather than an intermediate nobody sees. Absorption targets the largest
/// component because that is where a relative correction is least distorting — moving an ulp of
/// gross onto a small component could shift it by a large fraction of itself.
pub fn reconcile_partition(
    gross: f32,
    net: f32,
    not_net: f32,
    unknown: f32,
) -> Result<ReconciledPartition, PartitionResidual> {
    let residual = gross as f64 - (net as f64 + not_net as f64 + unknown as f64);
    // A zero-thickness zone has nothing to be relative TO; its components are zero as well, so the
    // residual is zero and the absolute value is the honest comparison.
    let scale = if gross.abs() > 0.0 { gross.abs() as f64 } else { 1.0 };
    let relative = residual.abs() / scale;
    if relative > PARTITION_TOLERANCE {
        return Err(PartitionResidual {
            gross,
            net,
            not_net,
            unknown,
            residual,
            relative,
            tolerance: PARTITION_TOLERANCE,
        });
    }
    let mut out =
        ReconciledPartition { net, not_net, unknown, absorbed: residual as f32 };
    if net >= not_net && net >= unknown {
        out.net = (net as f64 + residual) as f32;
    } else if not_net >= unknown {
        out.not_net = (not_net as f64 + residual) as f32;
    } else {
        out.unknown = (unknown as f64 + residual) as f32;
    }
    Ok(out)
}

/// PHIE as a pay calculation is allowed to read it: never negative, MISSING preserved.
///
/// The porosity modules already floor what they WRITE (`modules::PHIE_FLOOR`), but the motivating
/// case never passes through one — `docs/review_triage.md` finding 16. A vendor PHIE arriving by
/// LAS reads slightly negative over a tight carbonate streak, which is a routine artefact of a
/// sandstone-matrix density porosity rather than a corrupt curve. That streak reads low GR, clears
/// the VSH cutoff and is flagged SAND, and its `PHIE·(1−SWE)·h` is then SUBTRACTED from the SAND
/// row's hydrocarbon column. Measured, that took HPV more than 20 % below the floored answer while
/// RESERVOIR and PAY stayed byte-identical — so the two rows anyone checks first agreed with each
/// other while the third quietly did not, and the understatement was in the reassuring direction.
///
/// Applied ONCE per well so every consumer downstream sees one number: `hpv`, `avg_phie` and the
/// classifier cannot end up disagreeing about what the porosity at a depth was.
///
/// **`f32::max` returns the other side when one is NaN**, so the guard is load-bearing rather than
/// defensive: without it a MISSING sample would become a real 0.001 and start counting toward
/// `n_classified`, which is the one field that says whether the well was interpreted at all.
///
/// One function rather than a copy in each pay path — the cutoff SWEEP and the summary must agree
/// at the same cutoffs, and two copies is two places for the rule to drift.
pub(crate) fn floored_phie(raw: &[f32]) -> Vec<f32> {
    raw.iter().map(|&v| if v.is_nan() { v } else { v.max(modules::PHIE_FLOOR as f32) }).collect()
}

/// Everything one well's summation READS, fetched before any of it is summed.
///
/// #129: the four reads below were the Field Dashboard.
/// `PERF-DASHBOARD-2026-08-23.md` measured them at **101.7% of the 500-well wall clock** - more
/// than the whole operation, because the dashboard ran last and had the warmest cache - against an
/// arithmetic cost too small for the instrument to separate from noise. And they ran ONE WELL AT A
/// TIME, each taking the connection mutex, on a single thread.
///
/// So the fix is not a faster query, it is doing them at the same time. Splitting the reads out
/// like this keeps every line of the summation itself exactly where it was: the cut-offs, the zone
/// sweep and the row building are untouched and still run serially, in well order. Nothing about
/// what a number MEANS is on this path.
struct WellSummationInputs {
    well_id: String,
    well_name: String,
    phie_curve: String,
    quicklook_phie_excluded: bool,
    depth: Vec<f32>,
    columns: std::collections::HashMap<String, Vec<f32>>,
    zones: Vec<db::ZoneEntry>,
    /// The names the fetch above actually asked for. Carried rather than rebuilt at the write
    /// site: it is recorded as the run's `inputs_json`, so a second construction of "the same"
    /// list is a provenance record that can drift from what was read.
    curve_names: Vec<String>,
}

/// Read one well's inputs. `None` means SKIP THIS WELL, and it is the same three skips the serial
/// loop always made - an unresolvable PHIE ancestry, no curve frame, or an unreadable zone list.
///
/// A well is skipped rather than failing the run because a single bad well would otherwise zero
/// the entire Field Dashboard. That was true before this was parallel and is unchanged; it is
/// deliberately NOT the same thing as a read that could not be performed at all, which is an error
/// and is propagated - see the caller.
fn read_well_summation_inputs(
    conn: &Connection,
    well_id: &str,
    req: &PaySummaryRequest,
) -> Option<WellSummationInputs> {
    let well_name: String = conn
        .query_row(
            "SELECT well_name FROM wells WHERE well_id = ?1",
            duckdb::params![well_id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| well_id.to_string());
    // SB-POR-057 (DEC-070, RULED 2026-08-18): the candidate list is the ONE canonical
    // name. The quick-look D-N limited curve is no longer a fallback - "quick look only
    // shows pay summation as visual not pay curves" - superseding the DEC-042 two-name
    // pair this list used to carry. Displays may overlay PHIE_DN_LIM; the summed
    // numbers never read it.
    let phie_candidates = vec!["PHIE".to_string()];
    let (phie_curve, phie_resolved) = match first_available_input_alias(
        conn,
        well_id,
        "PHIE",
        &phie_candidates,
        req.input_set.as_deref(),
        None,
        &HashSet::new(),
    ) {
        Ok(Some(curve)) => (curve, true),
        Ok(None) => ("PHIE".to_string(), false),
        Err(_) => return None,
    };
    // DEC-070's observable half: when the ONLY porosity here is the quick-look curve,
    // the row says so - the zeros below mean "not interpreted for pay", never "wet".
    // Deliberately NOT set when the well has no porosity at all: the flag means
    // "present and excluded", and conflating it with absence would erase the reason
    // the mark exists.
    let quicklook_phie_excluded = !phie_resolved
        && crate::ancestry::try_resolve_ancestry_input(
            conn,
            well_id,
            "PHIE",
            modules::PHIE_DN_LIMITED_DEFAULT,
            req.input_set.as_deref(),
            None,
        )
        .ok()
        .flatten()
        .is_some();
    let curve_names: Vec<String> =
        vec!["VSH".into(), phie_curve.clone(), "SWE".into(), "PERM".into()];

    // Per-well isolation: a well with no curves - or a transient fetch/zone read error - is
    // skipped, keeping every other well's rows, rather than `?`-aborting the whole batch (a
    // single bad well would otherwise zero the entire Field Dashboard / summary response).
    // The cutoffs decide net pay, so WHICH version of PHIE and SWE they read is part of the
    // answer - a summary that cannot name its inputs' version cannot be reproduced.
    let (depth, columns) = match equations::fetch_curve_frame_from_set(
        conn, well_id, &curve_names, req.input_set.as_deref(), None,
    ) {
        Ok((d, c)) if !d.is_empty() => (d, c),
        _ => return None,
    };
    let zones = match db::list_zones(conn, well_id) {
        Ok(z) => z,
        Err(_) => return None,
    };
    Some(WellSummationInputs {
        well_id: well_id.to_string(),
        well_name,
        phie_curve,
        quicklook_phie_excluded,
        depth,
        columns,
        zones,
        curve_names,
    })
}

/// Computes the pay summary per well per zone and writes FLAG_SAND / FLAG_RESERVOIR /
/// FLAG_PAY curves. Wells without zones get a single whole-well "ALL" zone.
pub fn run_pay_summary(
    db: &Mutex<Connection>,
    pool: &crate::reader_pool::ReaderPool,
    req: &PaySummaryRequest,
) -> Result<Vec<PaySummaryRow>, String> {
    // SB-CUT-012: refuse a frame whose per-sample weights cannot be computed, before any work.
    // The per-sample weight is dz in MD and dz*cos(theta) in TVD, so a TVD summation is not a
    // rescaling of an MD one - it is a different set of weights - and serving MD numbers under a
    // TVD label is exactly what the requirement forbids.
    // SB-CUT-016: a cut-off switched on and left blank stops the run, before any work and
    // whatever else is set. Naming them all at once beats refusing one at a time.
    if !req.enabled_unset.is_empty() {
        return Err(format!(
            "cannot summate: {} enabled with no value. A cut-off has no default - four shipped vendor sets disagree and delivered work spans a wide range even within one field -              so set a value, or turn the cut-off off and the summation will report it unfiltered.",
            req.enabled_unset.join(", ")
        ));
    }
    // SB-CUT-019: canonicalise every entered cut-off before anything is computed. A bare number,
    // an unknown unit or a physically impossible value stops the run here, naming the field.
    // SB-CUT-020: and resolve each into a RANGE. A single-sided entry becomes the degenerate
    // range with an open far bound, so a request written before ranges existed means what it meant.
    let cut = |spec: &Option<CutoffSpec>,
               quantity: CutoffQuantity,
               sense: CutoffSense,
               label: &str| {
        spec.as_ref().map(|s| s.canonical(quantity, sense, label)).transpose()
    };
    let vsh_max = cut(&req.vsh_max, CutoffQuantity::VolumeFraction, CutoffSense::Maximum, "the VSH cut-off")?;
    let phie_min = cut(&req.phie_min, CutoffQuantity::VolumeFraction, CutoffSense::Minimum, "the PHIE cut-off")?;
    let swe_max = cut(&req.swe_max, CutoffQuantity::VolumeFraction, CutoffSense::Maximum, "the SWE cut-off")?;
    let perm_min = cut(&req.perm_min, CutoffQuantity::Permeability, CutoffSense::Minimum, "the PERM cut-off")?;
    // SB-CUT-022: resolve which tiers each cut-off is used at, once per run and from the SLOT plus
    // the caller's declaration only.
    let tier_cuts = TierCutoffs {
        vsh: vsh_max,
        phie: phie_min,
        swe: swe_max,
        perm: perm_min,
        vsh_use: cutoff_use_for(&req.cutoff_use, Slot::Vsh),
        phie_use: cutoff_use_for(&req.cutoff_use, Slot::Phie),
        swe_use: cutoff_use_for(&req.cutoff_use, Slot::Swe),
        perm_use: cutoff_use_for(&req.cutoff_use, Slot::Perm),
    };
    let unfiltered: Vec<String> = [
        ("VSH", vsh_max.is_none()),
        ("PHIE", phie_min.is_none()),
        ("SWE", swe_max.is_none()),
        ("PERM", perm_min.is_none()),
    ]
    .iter()
    .filter(|(_, absent)| *absent)
    .map(|(name, _)| (*name).to_string())
    .collect();
    if req.frame != SummationFrame::Md {
        return Err(format!(
            "cannot summate in {}: the per-sample weights would be dz*cos(theta) from the well's deviation survey, and SandiBumi computes only MD (dz) weights today. Run in MD, or ask for a {} summation to be built as its own record.",
            req.frame.as_str(),
            req.frame.as_str()
        ));
    }
    let mut all_rows = Vec::new();

    // #129: every well's reads, at the same time, before any of them is summed. This used to be
    // four queries per well on ONE connection on ONE thread, which
    // `PERF-DASHBOARD-2026-08-23.md` measured as the whole of the Field Dashboard.
    //
    // A `None` here is a well the serial loop would have `continue`d past, and it stays a silent
    // skip. An `Err` is something else entirely - the read could not be performed, because the
    // project was replaced underneath it - and it FAILS THE RUN rather than being folded into the
    // skip. Quietly dropping every well of a summary because the project moved would produce a
    // field total that is simply too small, with nothing on screen to say so.
    let prefetched: Vec<WellSummationInputs> = {
        let read: Result<Vec<Option<WellSummationInputs>>, String> = req
            .well_ids
            .par_iter()
            .map(|well_id| {
                pool.read(db, |conn| Ok(read_well_summation_inputs(conn, well_id, req)))
            })
            .collect();
        read?.into_iter().flatten().collect()
    };
    /// One well's FLAG-curve run, collected during the summation loop and resolved afterwards.
    ///
    /// It carries only what the resolution needs, so the summation loop stays free of the project:
    /// the rows to write, and the three things the provenance record is built from.
    struct PendingFlagRun {
        well_id: String,
        depth: Vec<f32>,
        flags: Vec<(String, Vec<f32>)>,
        spec: crate::ancestry::LogSetSpec,
        inputs: Vec<(String, String, String)>,
        zone_scope: crate::ancestry::AncestryZoneScope,
    }
    // Every well's FLAG work, resolved ONCE after the loop rather than well by well. The rows were
    // never the cost and neither is the arithmetic; it was a hundred round trips to the project.
    let mut pending_flag_runs: Vec<PendingFlagRun> = Vec::new();
    // The skipped wells are gone from this list, so the loop walks the inputs rather than the
    // requested ids - a well and its inputs can never be paired wrongly, because they are one
    // value.
    for inputs in prefetched {
        let WellSummationInputs {
            well_id,
            well_name,
            phie_curve,
            quicklook_phie_excluded,
            depth,
            columns,
            mut zones,
            curve_names,
        } = inputs;

        let had_declared_zones = !zones.is_empty();
        if zones.is_empty() {
            zones.push(db::ZoneEntry {
                zone_name: "ALL".into(),
                top_depth: depth[0],
                bottom_depth: *depth.last().unwrap(),
                depth_datum: crate::schema_vocab::DepthDatum::Md,
            });
        }

        let n = depth.len();
        let vsh = &columns["VSH"];
        let phie_col = floored_phie(&columns[&phie_curve]);
        let phie = &phie_col;
        let swe = &columns["SWE"];
        let perm = &columns["PERM"];
        // A requested cutoff is ALWAYS active — see `PaySummaryRow::perm_cutoff_no_data` for why
        // the well-level "does this well have any PERM?" test was removed. `classify_sample` fails
        // a sample whose PERM is missing, which is now the only rule in play.
        let has_perm_cut = req.perm_min.is_some();
        let perm_cutoff_no_data = has_perm_cut && !perm.iter().any(|v| !v.is_nan());

        // Sample thickness: forward depth difference, last sample reuses the previous step.
        let mut step = vec![0.0f32; n];
        for i in 0..n {
            step[i] = if i + 1 < n {
                depth[i + 1] - depth[i]
            } else if i > 0 {
                step[i - 1]
            } else {
                0.0
            };
        }

        // SB-CUT-002: the interval every row of this well will record.
        let sample_interval = median_sample_interval(&step);

        // Flags per sample: NaN inputs exclude the sample (flag stays NaN). Single-sourced
        // through `classify_sample` so the sweep engine below applies identical cutoff logic.
        let mut flag_sand = vec![f32::NAN; n];
        let mut flag_res = vec![f32::NAN; n];
        let mut flag_pay = vec![f32::NAN; n];
        for i in 0..n {
            let (fs, fr, fp) = classify_sample(
                vsh[i], phie[i], swe[i], perm[i], &tier_cuts, has_perm_cut,
            );
            flag_sand[i] = fs;
            flag_res[i] = fr;
            flag_pay[i] = fp;
        }

        if !req.stats_only {
            // No connection is taken here any more. Everything below is CPU: the reads this well
            // needs are collected and performed together, after the loop, through the pool.
            if req.skip_version {
                // Refused, never served: an in-place FLAG_* write leaves a pay flag that cannot
                // say which cutoffs produced it. `stats_only` is the supported way to want no
                // flags written, and it is checked above this.
                return Err("pay-summary write refused: skip_version would create ancestry-free FLAG curves; use a versioned run"
                    .into());
            } else {
                // Version the pay flags into a log set with provenance — module + the CUTOFFS
                // that produced them + the inputs — like any other module output, so a re-run
                // keeps history, any version is restorable/prunable from the catalog, and the
                // cutoffs are retrievable from log_sets.params_json.
                let params_json = serde_json::json!({
                    "vsh_max": req.vsh_max,
                    "phie_min": req.phie_min,
                    "swe_max": req.swe_max,
                    "perm_min": req.perm_min,
                })
                .to_string();
                let spec = crate::ancestry::LogSetSpec {
                    set_name: "PAYFLAG".into(),
                    module: "pay_summary".into(),
                    params_json,
                    inputs_json: serde_json::to_string(&curve_names).unwrap_or_default(),
                };
                let custody = req.custody.as_ref().ok_or_else(|| {
                    "pay-summary write refused: explicit run custody is required".to_string()
                })?;
                let mut ancestry_curves =
                    vec!["VSH".to_string(), phie_curve.clone(), "SWE".to_string()];
                if req.perm_min.is_some() && perm.iter().any(|value| value.is_finite()) {
                    ancestry_curves.push("PERM".into());
                }
                let inputs = ancestry_curves
                    .iter()
                    .map(|curve| (well_id.clone(), curve.clone(), curve.clone()))
                    .collect::<Vec<_>>();
                let zone_scope = if had_declared_zones {
                    crate::ancestry::AncestryZoneScope::Defined(
                        zones
                            .iter()
                            .filter(|zone| zone.top_depth < zone.bottom_depth)
                            .map(|zone| crate::ancestry::AncestryZone {
                                name: zone.zone_name.clone(),
                                top: zone.top_depth,
                                base: zone.bottom_depth,
                                source: custody.source_note.clone(),
                            })
                            .collect(),
                    )
                } else {
                    crate::ancestry::AncestryZoneScope::WholeWell
                };
                // Collected, not resolved here. Building this well's provenance record reads the
                // project (one `resolve_ancestry_input` per input curve) and so does the
                // already-current check (one `curve_ancestry` per output), and doing them inside
                // this loop meant ~700 small reads taken one after another - which is the same
                // shape `PERF-DASHBOARD-2026-08-23.md` found in the Field Dashboard, in the same
                // function that already prefetches its INPUTS through the pool.
                pending_flag_runs.push(PendingFlagRun {
                    well_id: well_id.clone(),
                    depth: depth.clone(),
                    flags: vec![
                        ("FLAG_SAND".to_string(), flag_sand.clone()),
                        ("FLAG_RESERVOIR".to_string(), flag_res.clone()),
                        ("FLAG_PAY".to_string(), flag_pay.clone()),
                    ],
                    spec,
                    inputs,
                    zone_scope,
                });
            }
        }

        for zone in &zones {
            for flag_name in SUMMARY_FLAGS {
                let flags = match flag_name {
                    "SAND" => &flag_sand,
                    "RESERVOIR" => &flag_res,
                    _ => &flag_pay,
                };
                let mut net = 0.0f64;
                // SB-CUT-003: footage the classifier saw and REJECTED. Only samples it could
                // actually evaluate land here; see the `unknown` derivation below.
                let mut not_net = 0.0f64;
                // SB-CUT-009: one accumulator per averaged slot, each carrying whichever weighting
                // the run DECLARED for that slot. The φ-weighted form used to be hard-wired to the
                // saturation slot, so it could be neither requested elsewhere nor switched off.
                let mut avg = [WeightedMean::default(); AVERAGED_SLOTS.len()];
                let mode: Vec<AverageWeighting> =
                    AVERAGED_SLOTS.iter().map(|s| weighting_for(&req.weighting, s)).collect();
                let mut hpv = 0.0f64;
                // Samples in this zone that the classifier could actually judge. A well whose
                // VSH/PHIE/SWE were never computed classifies to NaN everywhere, which leaves
                // net/ntg/hpv at 0.0 — byte-identical to a genuine wet or shaly zone. Carrying
                // the count lets the UI and the client PDF say "not interpreted" instead of
                // printing a hard zero that reads as a real answer.
                let mut n_classified = 0usize;

                for i in 0..n {
                    // Each sample represents the forward interval [depth[i], depth[i]+step].
                    // Clamp its contribution to the overlap with [zone.top, zone.bottom): the
                    // last in-zone sample no longer bleeds a full step past the base, a sample
                    // straddling the zone top is counted for its in-zone part, and net can never
                    // exceed gross (a sub-step-thick zone previously could).
                    // SB-CUT-001: ONE discretisation rule, shared. This site used to inline
                    // its own copy of the clamp; a second copy is a second thing to keep in
                    // step, and net pay is where a silent divergence costs most.
                    let (s_top, s_bot) =
                        sample_slab(depth[i] as f64, step[i] as f64, req.discretisation);
                    let h = sample_incl_thickness(
                        s_top,
                        s_bot,
                        zone.top_depth as f64,
                        zone.bottom_depth as f64,
                        None,
                    );
                    if h <= 0.0 {
                        continue;
                    }
                    if !flags[i].is_nan() {
                        n_classified += 1;
                    }
                    if flags[i] != 1.0 {
                        // SB-CUT-003: only an EVALUATED rejection is NotNet. A NaN flag means the
                        // classifier had nothing to judge, so its footage must fall through to
                        // `unknown` — folding it in here still closes the identity, which is
                        // precisely why the requirement names it.
                        if !flags[i].is_nan() {
                            not_net += h;
                        }
                        continue;
                    }
                    net += h;
                    // SB-CUT-009: the two weight bases. The φ basis is MISSING where porosity is,
                    // so a φ-weighted average silently skips a sample it cannot weight rather than
                    // treating it as weightless — the same rule the hard-wired version followed.
                    let w = |m: AverageWeighting| match m {
                        AverageWeighting::Thickness => h,
                        AverageWeighting::Porosity => {
                            if phie[i].is_nan() { f64::NAN } else { phie[i] as f64 * h }
                        }
                    };
                    for (slot, value) in [vsh[i], phie[i], swe[i]].into_iter().enumerate() {
                        // SB-CUT-030: values enter the sum through the ACCUMULATE stage, which
                        // never clamps. A clamp inside a sum does not relocate a tail - it moves
                        // the MEAN, by 0.3989*phi*sigma toward more hydrocarbon, independent of
                        // iteration count. Named at the site so a future edit has to argue with it.
                        let accumulated = stage_value(
                            ClampStage::Accumulate,
                            BoundedQuantity::VolumeFraction,
                            value,
                        );
                        avg[slot].add(accumulated, w(mode[slot]));
                    }
                    if !phie[i].is_nan() && !swe[i].is_nan() {
                        hpv += phie[i] as f64 * (1.0 - swe[i] as f64) * h;
                    }
                }

                let gross = zone.bottom_depth - zone.top_depth;
                // SB-CUT-003: the remainder, so the partition closes exactly. It absorbs both
                // kinds of unevaluable footage — an in-zone sample the classifier could not judge,
                // and footage no sample covers at all. Computed in f64 against the same f64 sums
                // the other two came from, then rounded once.
                let unknown = gross as f64 - net - not_net;
                // SB-CUT-005: check the partition AS REPORTED — the three f64 sums are each
                // rounded once on the way into the row, so the closure a reader receives is not
                // automatically the closure the arithmetic had. Within tolerance the drift is
                // absorbed into the largest component and recorded; outside it the summation
                // refuses rather than shipping a partition that does not add up.
                let recon = reconcile_partition(gross, net as f32, not_net as f32, unknown as f32)
                    .map_err(|residual| {
                        format!(
                            "{well_name} zone {} flag {flag_name}: {residual}",
                            zone.zone_name
                        )
                    })?;
                all_rows.push(PaySummaryRow {
                    well_id: well_id.clone(),
                    well_name: well_name.clone(),
                    zone: zone.zone_name.clone(),
                    flag: flag_name.to_string(),
                    discretisation_model: req.discretisation.token().to_string(),
                    sample_interval,
                    top: zone.top_depth,
                    bottom: zone.bottom_depth,
                    gross,
                    net: recon.net,
                    not_net: recon.not_net,
                    unknown: recon.unknown,
                    residual_absorbed: recon.absorbed,
                    frame: req.frame,
                    weights_source: MD_WEIGHTS_SOURCE.to_string(),
                    unfiltered: unfiltered.clone(),
                    // SB-CUT-004: the same net over the footage that was actually judged. MISSING
                    // rather than 0.0 when nothing was — there is no denominator, and a printed
                    // zero would be a claim about rock nobody looked at.
                    ntg_known: {
                        let judged = gross as f64 - unknown;
                        if judged > 0.0 { (net / judged) as f32 } else { f32::NAN }
                    },
                    ntg: if gross > 0.0 { (net / gross as f64) as f32 } else { 0.0 },
                    // Averages are normalised over the footage THAT curve was valid on — not total
                    // net — so a SAND-row sample with valid VSH but missing PHIE does not drag
                    // avg_phie toward zero. Each carries the weighting its slot declared.
                    // SB-CUT-030: the three averages are emitted through the PRESENT stage, and
                    // an average outside its quantity's bounds is FLAGGED rather than corrected.
                    // All three are volume fractions - the bound comes from the QUANTITY, not from
                    // the curve's name or declared type, which is the failure mode IP has.
                    out_of_range: [avg[0].value(), avg[1].value(), avg[2].value()]
                        .iter()
                        .any(|v| BoundedQuantity::VolumeFraction.is_out_of_range(*v)),
                    // No clamp between the average and the row, deliberately: a wrapper here
                    // returned every in-range value and every out-of-range value unchanged, and
                    // the one input it did change was an INFINITY, which it reported as a clean
                    // 1.0 beside an unset flag. "Emitted as computed" has no exceptions.
                    avg_vsh: avg[0].value(),
                    avg_phie: avg[1].value(),
                    avg_swe: avg[2].value(),
                    // SB-CUT-030: HPV is a volume-thickness, not a fraction - it routinely
                    // exceeds 1 - so it goes through the PRESENT stage as an UNBOUNDED quantity.
                    // That is the clause "an unbounded quantity MUST NOT be clamped to [0,1]"
                    // stated at the one site where a careless clamp would destroy the number
                    // rather than merely round it.
                    hpv: stage_value(
                        ClampStage::Present,
                        BoundedQuantity::Unbounded,
                        hpv as f32,
                    ),
                    n_classified,
                    perm_cutoff_no_data,
                    quicklook_phie_excluded,
                });
            }
        }
    }

    // The FLAG curves, in three passes: read the whole field at once, then allocate versions, then
    // write the rows once. Nothing here is per-well round-tripping any more.
    if !pending_flag_runs.is_empty() {
        let custody = req.custody.as_ref().ok_or_else(|| {
            "pay-summary write refused: explicit run custody is required".to_string()
        })?;
        let output_names: Vec<String> =
            vec!["FLAG_SAND".into(), "FLAG_RESERVOIR".into(), "FLAG_PAY".into()];

        // Pass 1 - every well's provenance record and already-current check, at the same time,
        // through the pool. These are READS, and they were the majority of what was left once the
        // write stopped being per-well.
        //
        // An error here FAILS THE RUN and is never folded into a skip, for the same reason the
        // input prefetch above states: quietly dropping a well would silently produce a field that
        // is simply missing flags, with nothing on screen to say so. The prefetch can treat a
        // `None` as a skip because that is a well with no curves; there is no equivalent here -
        // every well in this list has already been summed.
        let resolved: Vec<Option<(String, Vec<f32>, Vec<(String, Vec<f32>)>, crate::ancestry::CompleteLogSetSpec)>> =
            pending_flag_runs
                .into_par_iter()
                .map(|job| {
                    let output_names = &output_names;
                    pool.read(db, move |conn| {
                        let PendingFlagRun { well_id, depth, flags, spec, inputs, zone_scope } = job;
                        #[cfg(test)]
                        let _ps_spec = crate::lock_probe::ps_spec();
                        let mut complete = crate::ancestry::complete_curve_run_spec(
                            conn,
                            &well_id,
                            &spec.set_name,
                            &spec.module,
                            custody,
                            &inputs,
                            req.input_set.as_deref(),
                            serde_json::from_str(&spec.params_json).map_err(|error| {
                                format!("cannot record pay-summary parameters: {error}")
                            })?,
                            zone_scope,
                            output_names,
                        )?;
                        complete
                            .record_parameter_decisions(crate::param_sources::PAY_PARAMETER_TOPICS)?;
                        #[cfg(test)]
                        drop(_ps_spec);
                        // Previewing and then exporting the same report must not create two
                        // indistinguishable PAYFLAG versions. Reuse is allowed only when every
                        // material part of the live record matches; a changed input version,
                        // value/source, zone/source, operator, output, or implementation creates
                        // a new append-only version as usual.
                        #[cfg(test)]
                        let _ps_ancestry = crate::lock_probe::ps_ancestry();
                        let already_current = output_names.iter().all(|curve| {
                            crate::ancestry::curve_ancestry(conn, &well_id, curve)
                                .is_ok_and(|existing| existing.same_computation(complete.ancestry()))
                        });
                        #[cfg(test)]
                        drop(_ps_ancestry);
                        Ok((!already_current).then_some((well_id, depth, flags, complete)))
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;

        // Pass 2 - allocating a log-set version is a WRITE, so it stays serial under the one
        // connection, exactly as it always was. Only the reads above were parallelised.
        #[cfg(test)]
        let _ps_lock = crate::lock_probe::ps_lock();
        let conn = db.lock().unwrap();
        #[cfg(test)]
        drop(_ps_lock);
        let mut writes: Vec<crate::ancestry::CompleteWellWrite> = Vec::new();
        for (well_id, depth, flags, complete) in resolved.into_iter().flatten() {
            #[cfg(test)]
            let _ps_set = crate::lock_probe::ps_set();
            let (set_id, _) = crate::ancestry::create_complete_log_set(&conn, &well_id, &complete)?;
            #[cfg(test)]
            drop(_ps_set);
            writes.push(crate::ancestry::CompleteWellWrite {
                well_id,
                depth,
                curves: flags,
                set_id,
                // The pay summary has no degradation vocabulary and the single-well write this
                // replaced never classified a PAYFLAG version. Passing None keeps that exactly
                // true, so batching does not quietly start marking these runs CLEAN.
                degradation_module: None,
                degradations: None,
            });
        }

        // Pass 3 - one transaction for the whole field's FLAG curves, the same shape a chain step
        // already uses.
        //
        // This is the one place the batched form differs from the per-well one it replaces, and it
        // is stated rather than buried: the write is ALL-OR-NOTHING. Before, a failure at well 50
        // left wells 1-49 carrying committed flags and the run still returned an error, so a field
        // could be left half-flagged by a summary that reported failure. Now nothing commits unless
        // every well's rows do. A partially flagged field is the worse of the two outcomes - it
        // looks complete to every reader downstream - so this is the direction to fail in, and it
        // matches what a chain step has always done.
        if !writes.is_empty() {
            #[cfg(test)]
            let _ps_rows = crate::lock_probe::ps_rows();
            crate::ancestry::write_computed_curves_with_ancestry_batch(&conn, &writes)?;
            #[cfg(test)]
            drop(_ps_rows);
        }
    }

    Ok(all_rows)
}

// ---------------------------------------------------------------------------
// Cutoff sensitivity (ROADMAP Wave E item 21) — sweep the pay engine over a range
// of candidate cutoffs, holding the other two fixed, to find the elbow where pay
// stops responding to the cutoff. This is the sensitivity-sweep method; the companion
// method (DST-highlighted crossplots) lives in the frontend cutoff pane. Both follow the
// standard cutoff-selection practice: pick the cutoff where net stops responding, then
// confirm it against tested rock rather than against the sweep alone.
// ---------------------------------------------------------------------------

/// Per-sample SAND / RESERVOIR / PAY classification against the cutoffs, matching the
/// Pay-summary NaN propagation: a missing VSH excludes all three (returns NaN,NaN,NaN);
/// a missing PHIE excludes RESERVOIR and PAY; a missing SWE excludes PAY. Each returned
/// value is `f32::NAN` when the sample is excluded, else `0.0`/`1.0`. `has_perm_cut` is the
/// caller's decision that a PERM cutoff is active (perm_min set and PERM present in the set).
#[inline]
fn classify_sample(
    vsh: f32,
    phie: f32,
    swe: f32,
    perm: f32,
    cuts: &TierCutoffs,
    has_perm_cut: bool,
) -> (f32, f32, f32) {
    // SB-CUT-016: an ABSENT cut-off does not filter. The NaN cascade below is deliberately
    // untouched by that - a sample with no VSH is unjudgeable whether or not VSH is being used as
    // a cut-off, and making an unfiltered cut-off also stop requiring its curve would let a well
    // with no VSH book pay it never booked. The requirement says nothing about NaN handling, so
    // the rule stands.
    //
    // SB-CUT-022 leaves it alone for the same reason. The use flags govern whether a cut-off's
    // VALUE is applied at a tier; they say nothing about whether the tier needs that curve to be
    // judgeable at all. Those are two different questions and only one of them is a cut-off.
    if vsh.is_nan() {
        return (f32::NAN, f32::NAN, f32::NAN);
    }
    // SB-CUT-022: each tier applies exactly the cut-offs DECLARED for it. The ladder that used to
    // be expressed by nesting — reservoir built on sand, pay built on reservoir — is now expressed
    // by the default flags, which say the same thing wherever nobody declares otherwise.
    let judge = |tier: Tier| {
        let passes = |slot: Slot, sample: f32| {
            // SB-CUT-030: the FLAG_TEST stage compares the value clamped to its QUANTITY's bounds
            // - the bound comes from the quantity, never from the curve's name or declared type,
            // which is the failure that makes IP's clipping invisible in the data. Inert for an
            // in-range sample, which is every ordinary one, so no number moves; what it does is put
            // the stage boundary somewhere a reader can find it.
            let tested = stage_value(ClampStage::FlagTest, slot.bounded_quantity(), sample);
            cuts.applied(tier, slot).map_or(true, |range| range.contains(tested))
        };
        passes(Slot::Vsh, vsh)
            && passes(Slot::Phie, phie)
            && passes(Slot::Swe, swe)
            // A sample with no PERM value cannot demonstrate it passes the cutoff — missing PERM
            // must FAIL rather than silently pass, at whichever tier the cut-off is applied.
            && (!has_perm_cut
                || cuts
                    .applied(tier, Slot::Perm)
                    .map_or(true, |range| !perm.is_nan() && range.contains(perm)))
    };
    let fs = judge(Tier::Sand) as u8 as f32;
    if phie.is_nan() {
        return (fs, f32::NAN, f32::NAN);
    }
    let fr = judge(Tier::Reservoir) as u8 as f32;
    if swe.is_nan() {
        return (fs, fr, f32::NAN);
    }
    (fs, fr, judge(Tier::Pay) as u8 as f32)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SweepProp {
    Vsh,
    Phie,
    Swe,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Metric {
    Net,
    Hpv,
    Ntg,
}

/// Evaluates the pay metric at every candidate cutoff. Pure over pre-assembled arrays so it
/// is unit-testable without a database; `incl_h[i]` is the sample's clamped geometric
/// thickness within the analysed interval (zone ∩ DST) — 0 excludes it, and net accumulates
/// this clamped overlap (NOT the raw sample step) so net can never exceed gross, matching
/// run_pay_summary. `gross` is the geometric denominator for NTG. Returns
/// (cutoffs, values, peak) where `peak` is the maximum value over the sweep.
#[allow(clippy::too_many_arguments)]
fn compute_sweep(
    vsh: &[f32],
    phie: &[f32],
    swe: &[f32],
    perm: &[f32],
    incl_h: &[f64],
    prop: SweepProp,
    fixed_vsh: Option<CutoffRange>,
    fixed_phie: Option<CutoffRange>,
    fixed_swe: Option<CutoffRange>,
    perm_min: Option<CutoffRange>,
    sweep_min: f64,
    sweep_max: f64,
    steps: usize,
    metric: Metric,
    gross: f64,
) -> (Vec<f64>, Vec<f64>, f64) {
    let steps = steps.clamp(2, 500);
    let n = vsh.len();
    // A REQUESTED cutoff is always active — Jauhar's DEC-084 ruling, verbatim: "no well with no
    // perm can escape cutoff … dont let it off, its independent". Lacking a measured PERM is not
    // an exemption; it is absence of evidence, and `classify_sample` fails a sample whose PERM is
    // missing. This site kept the well-level "does this well have any PERM?" test after
    // `run_pay_summary` (:4552) and the Monte Carlo path had both dropped it, so the same held
    // cutoffs gave OPPOSITE answers on a well with no PERM: the pay summary booked zero net with
    // its perm_cutoff_no_data evidence flag, while the sensitivity curve beside it dropped the
    // cutoff and reported a full, optimistic net. The comment here even claimed agreement with
    // run_pay_summary, which stopped being true when that function was corrected.
    let has_perm_cut = perm_min.is_some();

    let mut cutoffs = Vec::with_capacity(steps);
    let mut values = Vec::with_capacity(steps);
    let mut peak = f64::NEG_INFINITY;

    for k in 0..steps {
        let t = k as f64 / (steps - 1) as f64;
        let cut = sweep_min + (sweep_max - sweep_min) * t;
        let (mut vsh_max, mut phie_min, mut swe_max) = (fixed_vsh, fixed_phie, fixed_swe);
        // SB-CUT-020: the SWEPT bound is the degenerate single-sided range in the slot's own
        // sense, inclusive - the sweep varies a cut-off VALUE and says nothing about inclusivity,
        // so it uses the same default the single-sided forms have always carried. The HELD
        // cut-offs keep whatever operators the caller declared.
        let swept = |low: bool| {
            let bound = Some(CutoffBound { value: cut, operator: BoundOperator::Inclusive });
            if low {
                CutoffRange { low: bound, high: None }
            } else {
                CutoffRange { low: None, high: bound }
            }
        };
        match prop {
            SweepProp::Vsh => vsh_max = Some(swept(false)),
            SweepProp::Phie => phie_min = Some(swept(true)),
            SweepProp::Swe => swe_max = Some(swept(false)),
        }
        // SB-CUT-022: the sweep is a plot of one cut-off VALUE against a metric, so it carries the
        // shipped tier declaration. A caller who wants a different one runs the summary, which is
        // where a declaration belongs.
        let swept_cuts = TierCutoffs {
            vsh: vsh_max,
            phie: phie_min,
            swe: swe_max,
            perm: perm_min,
            vsh_use: default_cutoff_use(Slot::Vsh),
            phie_use: default_cutoff_use(Slot::Phie),
            swe_use: default_cutoff_use(Slot::Swe),
            perm_use: default_cutoff_use(Slot::Perm),
        };

        let mut net = 0.0f64;
        let mut hpv = 0.0f64;
        for i in 0..n {
            let h = incl_h[i];
            if h <= 0.0 {
                continue;
            }
            let (_s, _r, pay) = classify_sample(
                vsh[i], phie[i], swe[i], perm[i], &swept_cuts, has_perm_cut,
            );
            if pay == 1.0 {
                net += h;
                if !phie[i].is_nan() && !swe[i].is_nan() {
                    hpv += phie[i] as f64 * (1.0 - swe[i] as f64) * h;
                }
            }
        }

        let value = match metric {
            Metric::Net => net,
            Metric::Hpv => hpv,
            Metric::Ntg => {
                if gross > 0.0 {
                    net / gross
                } else {
                    0.0
                }
            }
        };
        cutoffs.push(cut);
        values.push(value);
        if value > peak {
            peak = value;
        }
    }
    if !peak.is_finite() {
        peak = 0.0;
    }
    (cutoffs, values, peak)
}

#[derive(Debug, Clone, Deserialize)]
pub struct CutoffSweepRequest {
    /// Sweep the cutoffs against THIS log set's stored curves rather than the current values —
    /// the same freedom the pay summary it informs has (Jauhar, 2026-08-05).
    #[serde(default)]
    pub input_set: Option<String>,
    pub well_ids: Vec<String>,
    /// SB-CUT-001 (DEC-071): the discretisation model, shared with the pay summary the
    /// sweep informs - a sweep read against one model and a summary run under another
    /// would put the elbow in the wrong place.
    #[serde(default)]
    pub discretisation: DiscretisationModel,
    /// Which cutoff to sweep: "VSH" | "PHIE" | "SWE".
    pub property: String,
    /// Fixed values for the two cutoffs NOT being swept (the swept one's field is ignored).
    /// SB-CUT-016: `None` = that property is not filtered while this sweep runs. No default.
    /// SB-CUT-019: carried as entered, with its unit.
    pub vsh_max: Option<CutoffSpec>,
    pub phie_min: Option<CutoffSpec>,
    pub swe_max: Option<CutoffSpec>,
    pub perm_min: Option<CutoffSpec>,
    pub sweep_min: f64,
    pub sweep_max: f64,
    pub steps: usize,
    /// Metric plotted on Y: "NET" (net thickness) | "HPV" (hydrocarbon pore-thickness) | "NTG".
    pub metric: String,
    /// Restrict to one named zone; None/empty = whole well.
    #[serde(default)]
    pub zone: Option<String>,
    /// Restrict to samples inside an aux_data interval set (e.g. "PERFORATION" / "DST");
    /// None/empty = every sample in the zone.
    #[serde(default)]
    pub dst_dataset: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CutoffSweepSeries {
    pub well_id: String,
    pub well_name: String,
    pub cutoffs: Vec<f64>,
    pub values: Vec<f64>,
    /// Maximum value over the sweep (the frontend normalises each well to its own peak).
    pub peak: f64,
    /// Geometric gross thickness of the analysed interval (NTG denominator).
    pub gross: f64,
    /// Number of samples that entered the analysis (0 ⇒ nothing to plot; UI warns).
    pub n_samples: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CutoffSweepResult {
    pub series: Vec<CutoffSweepSeries>,
    pub property: String,
    pub metric: String,
}

/// Collapses an aux_data set to its distinct, non-overlapping depth intervals (rows with a
/// base depth, merged) for DST/perforation filtering. Point rows (no base) are ignored — a
/// test needs an interval, not a marker. Overlapping or touching intervals are unioned so a
/// re-perforation or redundant row cannot inflate the summed DST gross (the NTG denominator):
/// membership already counts each sample once (via `any`), so the gross must too.
fn aux_intervals(rows: &[db::AuxRow]) -> Vec<(f32, f32)> {
    let mut iv: Vec<(f32, f32)> = rows
        .iter()
        .filter_map(|r| r.depth_base.map(|b| (r.depth_top, b)))
        .filter(|(t, b)| b > t)
        .collect();
    iv.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut merged: Vec<(f32, f32)> = Vec::with_capacity(iv.len());
    for (t, b) in iv {
        match merged.last_mut() {
            Some(last) if t <= last.1 => {
                if b > last.1 {
                    last.1 = b;
                }
            }
            _ => merged.push((t, b)),
        }
    }
    merged
}

/// Geometric overlap thickness of a sample's forward interval `[s_top, s_bot]` with the
/// zone `[ztop, zbot)`, further intersected with the (merged, non-overlapping) DST intervals
/// when present. Mirrors run_pay_summary's zone clamp so a sample straddling the zone/DST
/// boundary contributes only its in-interval part and net can never exceed gross.
/// SB-CUT-001 (DEC-071, RULED 2026-08-18): the thickness discretisation model is a
/// PARAMETER of the one shared rule, defaulting to CENTRED per the requirement text - a
/// sample's slab straddles its depth, representing the rock AROUND the measurement.
/// FORWARD (the chapter's TOPS rule, Techlog computeGross) stays selectable so a legacy
/// run's numbers can be reproduced bit-for-bit. Jauhar accepted that the CENTRED default
/// moves every existing net-pay and NTG number by up to half a sample step at each
/// pay/zone edge ("3, centred", after the difference was explained in thickness terms).
/// ONE vocabulary everywhere: the serde wire form IS the record token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DiscretisationModel {
    #[default]
    #[serde(rename = "CENTRED")]
    Centred,
    #[serde(rename = "TOPS")]
    Forward,
}

impl DiscretisationModel {
    pub fn token(self) -> &'static str {
        match self {
            Self::Centred => "CENTRED",
            Self::Forward => DISCRETISATION_MODEL,
        }
    }
}

/// SB-CUT-001: the ONE place a sample's slab is derived from its depth and step - the
/// model choice can never again be inlined divergently at a call site.
pub(crate) fn sample_slab(depth: f64, step: f64, model: DiscretisationModel) -> (f64, f64) {
    match model {
        DiscretisationModel::Forward => (depth, depth + step),
        DiscretisationModel::Centred => (depth - step / 2.0, depth + step / 2.0),
    }
}

/// SB-CUT-002: the record token of the FORWARD rule - the forward-interval, zone-clipped
/// rule the chapter's T02 hand-traces from Techlog's computeGross and names TOPS. A
/// summation number without its model is not reproducible: IP ships two different "Net"
/// definitions under one column heading and labels neither. Since DEC-071 the model is a
/// request parameter defaulting to CENTRED; every record carries the token of the model
/// that actually produced it.
pub const DISCRETISATION_MODEL: &str = "TOPS";

/// SB-CUT-002: the sample interval a summation was computed on — the median forward step, the
/// same summary `reframe`'s regularize already uses for "the source's own spacing". Recorded
/// per record because net-to-gross is NOT scale-invariant (the chapter's T4: 0.55 → 0.75 → 1.0
/// across three blocking steps). NaN when no positive step exists.
pub(crate) fn median_sample_interval(step: &[f32]) -> f32 {
    let mut positive: Vec<f32> = step.iter().copied().filter(|s| *s > 0.0).collect();
    if positive.is_empty() {
        return f32::NAN;
    }
    positive.sort_by(|a, b| a.partial_cmp(b).unwrap());
    positive[positive.len() / 2]
}

pub(crate) fn sample_incl_thickness(
    s_top: f64,
    s_bot: f64,
    ztop: f64,
    zbot: f64,
    dst: Option<&[(f32, f32)]>,
) -> f64 {
    let lo = s_top.max(ztop);
    let hi = s_bot.min(zbot);
    let base = hi - lo;
    if base <= 0.0 {
        return 0.0;
    }
    match dst {
        None => base,
        // DST intervals are pre-merged (non-overlapping) by aux_intervals, so summing the
        // per-interval overlaps counts each unit of thickness at most once.
        Some(iv) => iv
            .iter()
            .map(|(t, b)| {
                let l2 = lo.max(*t as f64);
                let h2 = hi.min(*b as f64);
                (h2 - l2).max(0.0)
            })
            .sum(),
    }
}

/// A 0-sample sweep row so a well that can't be analysed (no curves, missing zone, or a
/// transient DB read error) still shows in the legend as "(0 samples)" instead of vanishing
/// and making the well count undercount.
fn empty_sweep_series(well_id: &str, well_name: String) -> CutoffSweepSeries {
    CutoffSweepSeries {
        well_id: well_id.to_string(),
        well_name,
        cutoffs: Vec::new(),
        values: Vec::new(),
        peak: 0.0,
        gross: 0.0,
        n_samples: 0,
    }
}

/// Method 1 of the cutoff study: for each well, sweep one cutoff across `[sweep_min,
/// sweep_max]` (holding the other two fixed) and report the pay metric at each step, so the
/// user can pick the cutoff at the response elbow. Reads VSH/PHIE/SWE/PERM, filters to an
/// optional zone and optional DST interval set, and writes nothing (pure analysis).
pub fn run_cutoff_sweep(
    db: &Mutex<Connection>,
    req: &CutoffSweepRequest,
) -> Result<CutoffSweepResult, String> {
    // SB-CUT-019: the two HELD cut-offs are entered values and are canonicalised before any
    // sweep runs. The swept property's range is a plot bound, not a cut-off, and keeps its own
    // units by construction - it is expressed in whatever the swept quantity's canonical unit is.
    let cut = |spec: &Option<CutoffSpec>,
               quantity: CutoffQuantity,
               sense: CutoffSense,
               label: &str| {
        spec.as_ref().map(|s| s.canonical(quantity, sense, label)).transpose()
    };
    let held_vsh = cut(&req.vsh_max, CutoffQuantity::VolumeFraction, CutoffSense::Maximum, "the held VSH cut-off")?;
    let held_phie = cut(&req.phie_min, CutoffQuantity::VolumeFraction, CutoffSense::Minimum, "the held PHIE cut-off")?;
    let held_swe = cut(&req.swe_max, CutoffQuantity::VolumeFraction, CutoffSense::Maximum, "the held SWE cut-off")?;
    let held_perm = cut(&req.perm_min, CutoffQuantity::Permeability, CutoffSense::Minimum, "the held PERM cut-off")?;
    let prop = match req.property.to_uppercase().as_str() {
        "VSH" => SweepProp::Vsh,
        "PHIE" => SweepProp::Phie,
        "SWE" => SweepProp::Swe,
        other => return Err(format!("unknown sweep property '{other}' (VSH|PHIE|SWE)")),
    };
    let metric = match req.metric.to_uppercase().as_str() {
        "NET" => Metric::Net,
        "HPV" => Metric::Hpv,
        "NTG" => Metric::Ntg,
        other => return Err(format!("unknown metric '{other}' (NET|HPV|NTG)")),
    };
    if !(req.sweep_max > req.sweep_min) {
        return Err("sweep max must be greater than sweep min".into());
    }
    let steps = req.steps.clamp(2, 500);
    let dst_name = req.dst_dataset.as_deref().filter(|s| !s.is_empty());
    let zone_name = req.zone.as_deref().filter(|s| !s.is_empty());
    let curve_names: Vec<String> = vec!["VSH".into(), "PHIE".into(), "SWE".into(), "PERM".into()];
    let mut series = Vec::new();

    for well_id in &req.well_ids {
        let conn = db.lock().unwrap();
        let well_name: String = conn
            .query_row(
                "SELECT well_name FROM wells WHERE well_id = ?1",
                duckdb::params![well_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| well_id.clone());
        // Per-well isolation: a transient fetch/zone/aux read error skips just this well (a
        // 0-sample legend row) instead of `?`-aborting the whole batch and discarding every
        // well already accumulated — same graceful degradation as run_workflow_module.
        let (depth, columns) = match equations::fetch_curve_frame_from_set(
            &conn, well_id, &curve_names, req.input_set.as_deref(), None,
        ) {
            Ok((d, c)) if !d.is_empty() => (d, c),
            _ => {
                drop(conn);
                series.push(empty_sweep_series(well_id, well_name));
                continue;
            }
        };
        let zones = match db::list_zones(&conn, well_id) {
            Ok(z) => z,
            Err(_) => {
                drop(conn);
                series.push(empty_sweep_series(well_id, well_name));
                continue;
            }
        };
        let dst = match dst_name {
            Some(ds) => match db::list_aux_data(&conn, well_id, Some(ds)) {
                Ok(rows) => Some(aux_intervals(&rows)),
                Err(_) => {
                    drop(conn);
                    series.push(empty_sweep_series(well_id, well_name));
                    continue;
                }
            },
            None => None,
        };
        drop(conn);

        let n = depth.len();
        let vsh = &columns["VSH"];
        let phie_col = floored_phie(&columns["PHIE"]);
        let phie = &phie_col;
        let swe = &columns["SWE"];
        let perm = &columns["PERM"];

        // Sample thickness: forward depth difference, last sample reuses the previous step
        // (same convention as run_pay_summary).
        let mut step = vec![0.0f32; n];
        for i in 0..n {
            step[i] = if i + 1 < n {
                depth[i + 1] - depth[i]
            } else if i > 0 {
                step[i - 1]
            } else {
                0.0
            };
        }

        // Zone bounds: a named zone that a well lacks yields an empty (0-sample) series so
        // the run still returns a row for that well rather than silently dropping it.
        let (ztop, zbot) = match zone_name {
            Some(z) => match zones.iter().find(|zz| zz.zone_name == z) {
                Some(zz) => (zz.top_depth, zz.bottom_depth),
                None => {
                    series.push(empty_sweep_series(well_id, well_name));
                    continue;
                }
            },
            None => (depth[0], *depth.last().unwrap()),
        };

        // Per-sample clamped geometric thickness within [ztop, zbot) ∩ DST — mirrors
        // run_pay_summary's zone clamp so net can never exceed gross. A sample straddling the
        // zone/DST boundary contributes only its in-interval part, not its whole step; a DST
        // boundary landing mid-sample counts that sample's actual overlap fraction.
        let mut incl_h = vec![0.0f64; n];
        let mut n_incl = 0usize;
        for i in 0..n {
            let (s_top, s_bot) = sample_slab(depth[i] as f64, step[i] as f64, req.discretisation);
            let h = sample_incl_thickness(s_top, s_bot, ztop as f64, zbot as f64, dst.as_deref());
            incl_h[i] = h;
            if h > 0.0 {
                n_incl += 1;
            }
        }

        // Geometric gross (NTG denominator): DST intervals clipped to the zone, else the
        // whole zone length.
        let gross = match &dst {
            None => (zbot - ztop).max(0.0) as f64,
            Some(iv) => iv
                .iter()
                .map(|(t, b)| {
                    let lo = (*t).max(ztop);
                    let hi = (*b).min(zbot);
                    (hi - lo).max(0.0) as f64
                })
                .sum(),
        };

        let (cutoffs, values, peak) = compute_sweep(
            vsh, phie, swe, perm, &incl_h, prop, held_vsh, held_phie, held_swe,
            held_perm, req.sweep_min, req.sweep_max, steps, metric, gross,
        );
        series.push(CutoffSweepSeries {
            well_id: well_id.clone(),
            well_name,
            cutoffs,
            values,
            peak,
            gross,
            n_samples: n_incl,
        });
    }

    Ok(CutoffSweepResult {
        series,
        property: req.property.to_uppercase(),
        metric: req.metric.to_uppercase(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ArgKind;

    /// AUDIT-2026-08-20 finding 49. A file boundary that nothing holds is a boundary that drifts
    /// back: the first time someone needs a pay-summary type inside the runner, adding one `use`
    /// is easier than moving the code, and after three of those the two halves share a file again
    /// in everything but name.
    ///
    /// Pinned from BOTH sides, because either arm alone has a lazier way to pass. Downward must be
    /// ZERO - the runner has no business naming a summation type, and that is what makes this a
    /// module rather than a circular pair. Upward must be NARROW and stated as a number, because
    /// the cheap way to satisfy the first arm is to re-export the whole runner into this file and
    /// call the dependency "one import".
    ///
    /// The upward count is the compiler's, not a grep's. An earlier text sweep put the seam at two
    /// functions; the second hit was inside a comment, and the unused import proved it.
    #[test]
    fn the_runner_never_names_the_pay_summary_and_the_pay_summary_names_one_function_of_it() {
        let runner = include_str!("workflow.rs");
        // Production only. The four tests that legitimately exercise the seam live in the runner's
        // test module and DO name this file - that is the seam being tested, not a leak.
        let runner_production =
            runner.split("
mod tests").next().expect("a split always yields one piece");
        let needle = ["paysummary", "::"].concat();
        assert_eq!(
            runner_production.matches(needle.as_str()).count(),
            0,
            "the module runner must not name the pay summary; the dependency runs one way only",
        );

        let mine = include_str!("paysummary.rs");
        let production = mine.split("
mod tests").next().expect("a split always yields one piece");
        // CODE lines only. This file's own module doc explains the seam in prose and names the
        // runner while doing so; counting that would be counting the explanation, not the
        // dependency.
        let reached: Vec<&str> = production
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with("//"))
            .filter(|line| line.contains(["workflow", "::"].concat().as_str()))
            .collect();
        assert_eq!(
            reached,
            vec![["use crate", "::workflow::first_available_input_alias;"].concat()],
            "exactly one item of the runner is consumed here, named rather than reached through a glob; a wider seam means the split has stopped being a boundary",
        );
    }

    fn at_most(value: f64) -> Option<CutoffRange> {
        Some(CutoffRange {
            low: None,
            high: Some(CutoffBound { value, operator: BoundOperator::Inclusive }),
        })
    }

    fn at_least(value: f64) -> Option<CutoffRange> {
        Some(CutoffRange {
            low: Some(CutoffBound { value, operator: BoundOperator::Inclusive }),
            high: None,
        })
    }

    /// The shared cutoff classifier must reproduce the .paysum NaN propagation exactly:
    /// a missing input excludes it and everything downstream, and a missing PERM fails an
    /// active PERM cutoff instead of passing.
    /// SB-CUT-020. The degenerate single-sided cut-offs these classification tests were written
    /// against, named rather than positional: `at_most` is a high bound, `at_least` a low one, both
    /// INCLUSIVE, which is exactly what a bare `>=` / `<=` cut-off has always meant.
    /// SB-CUT-022. The shipped tier ladder over four cut-off values — what a run that declares
    /// nothing applies. These classification tests predate the flags and must keep asserting the
    /// same behaviour through them, which is the point: the ladder moved from nesting to
    /// declaration without moving a number.
    fn ladder(
        vsh: Option<CutoffRange>,
        phie: Option<CutoffRange>,
        swe: Option<CutoffRange>,
        perm: Option<CutoffRange>,
    ) -> TierCutoffs {
        TierCutoffs {
            vsh,
            phie,
            swe,
            perm,
            vsh_use: default_cutoff_use(Slot::Vsh),
            phie_use: default_cutoff_use(Slot::Phie),
            swe_use: default_cutoff_use(Slot::Swe),
            perm_use: default_cutoff_use(Slot::Perm),
        }
    }

    /// Eleven samples one unit apart from 1000, every one of which passes every cutoff, with φ,
    /// Sw and Vsh each stepping halfway down. The whole point is that φ and the other curves are
    /// ANTI-correlated, so a thickness-weighted average and a φ-weighted one give visibly
    /// different answers — over the ten in-zone units:
    ///
    /// * `Σφh = 5(0.30) + 5(0.10) = 2.0`
    /// * Sw thickness-weighted `= 0.40`, φ-weighted `= 0.30`
    /// * Vsh thickness-weighted `= 0.25`, φ-weighted `= 0.175`
    ///
    /// `porosity_name` fills the porosity slot under a chosen mnemonic, which is what arm D needs.
    fn seed_weighting_well(conn: &duckdb::Connection, name: &str, porosity_name: &str) -> String {
        let id = uuid::Uuid::new_v4();
        db::insert_well(conn, id, name, Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let n = 11usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n], vec![2.35; n],
            nan.clone(), nan,
        )
        .unwrap();
        let half = |lo: f32, hi: f32| -> Vec<f32> {
            (0..n).map(|i| if i < 5 { lo } else { hi }).collect()
        };
        equations::write_computed_curve(conn, &well, &depth, "VSH", &half(0.10, 0.40)).unwrap();
        equations::write_computed_curve(conn, &well, &depth, porosity_name, &half(0.30, 0.10))
            .unwrap();
        equations::write_computed_curve(conn, &well, &depth, "SWE", &half(0.20, 0.60)).unwrap();
        well
    }

    /// 21 samples one unit apart from 1000, split so that all three kinds of footage are present
    /// and none of them is zero. VSH alone decides the split, because it is the one curve whose
    /// ABSENCE makes a sample unjudgeable rather than merely failing:
    ///
    /// * `1000..1010` — VSH 0.2, passes the 0.5 cutoff  → **10 units NET**
    /// * `1010..1015` — VSH 0.8, fails the 0.5 cutoff   → **5 units NOT-NET**
    /// * `1015..`     — VSH MISSING, cannot be judged   → **UNKNOWN**
    fn seed_partition_well(conn: &duckdb::Connection, name: &str) -> String {
        let id = uuid::Uuid::new_v4();
        db::insert_well(conn, id, name, Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let n = 21usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n], vec![2.35; n],
            nan.clone(), nan,
        )
        .unwrap();
        let mut vsh = vec![0.2f32; n];
        vsh[10..15].fill(0.8);
        vsh[15..].fill(f32::NAN);
        equations::write_computed_curve(conn, &well, &depth, "VSH", &vsh).unwrap();
        for (curve, v) in [("PHIE", 0.20f32), ("SWE", 0.30)] {
            equations::write_computed_curve(conn, &well, &depth, curve, &vec![v; n]).unwrap();
        }
        well
    }

    /// A clean, porous, WET sand: every sample passes a clay and a porosity cut-off and fails any
    /// ordinary saturation cut-off. It is the rock SB-CUT-026 exists to protect.
    fn seed_wet_reservoir(conn: &duckdb::Connection, name: &str) -> String {
        let id = uuid::Uuid::new_v4();
        db::insert_well(conn, id, name, Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let n = 11usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            conn, id, depth.clone(), vec![30.0; n], vec![4.0; n], vec![0.25; n], vec![2.3; n],
            nan.clone(), nan,
        )
        .unwrap();
        for (curve, value) in [("VSH", 0.10f32), ("PHIE", 0.30), ("SWE", 0.80)] {
            equations::write_computed_curve(conn, &well, &depth, curve, &vec![value; n]).unwrap();
        }
        well
    }

    /// A clean, porous, low-Sw sand where every sample passes VSH/PHIE/SWE on its own, so the
    /// only thing that can exclude a sample is the PERM cutoff. `perm` is the permeability the
    /// well MEASURED — `None` means the well carries none at all, which is the case under test.
    fn seed_pay_well(conn: &duckdb::Connection, name: &str, perm: Option<f32>) -> String {
        let id = uuid::Uuid::new_v4();
        db::insert_well(conn, id, name, Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let n = 20usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n], vec![2.35; n],
            nan.clone(), nan,
        )
        .unwrap();
        for (curve, v) in [("VSH", 0.2f32), ("PHIE", 0.20), ("SWE", 0.30)] {
            equations::write_computed_curve(conn, &well, &depth, curve, &vec![v; n]).unwrap();
        }
        if let Some(k) = perm {
            equations::write_computed_curve(conn, &well, &depth, "PERM", &vec![k; n]).unwrap();
        }
        well
    }

    #[test]
    fn classify_sample_nan_propagation() {
        // Clean pay (no perm cut).
        assert_eq!(
            classify_sample(0.2, 0.2, 0.3, f32::NAN, &ladder(at_most(0.5), at_least(0.1), at_most(0.6), None), false),
            (1.0, 1.0, 1.0)
        );
        // Missing VSH → all excluded.
        let (s, r, p) = classify_sample(f32::NAN, 0.2, 0.3, f32::NAN, &ladder(at_most(0.5), at_least(0.1), at_most(0.6), None), false);
        assert!(s.is_nan() && r.is_nan() && p.is_nan());
        // Missing PHIE → SAND set, RES/PAY excluded.
        let (s, r, p) = classify_sample(0.2, f32::NAN, 0.3, f32::NAN, &ladder(at_most(0.5), at_least(0.1), at_most(0.6), None), false);
        assert_eq!(s, 1.0);
        assert!(r.is_nan() && p.is_nan());
        // Missing SWE → SAND+RES set, PAY excluded.
        let (s, r, p) = classify_sample(0.2, 0.2, f32::NAN, f32::NAN, &ladder(at_most(0.5), at_least(0.1), at_most(0.6), None), false);
        assert_eq!((s, r), (1.0, 1.0));
        assert!(p.is_nan());
        // Fails the sand cutoff → SAND 0 cascades to RES/PAY 0.
        assert_eq!(
            classify_sample(0.9, 0.2, 0.3, f32::NAN, &ladder(at_most(0.5), at_least(0.1), at_most(0.6), None), false),
            (0.0, 0.0, 0.0)
        );
        // Active PERM cutoff: missing PERM fails; sufficient PERM passes.
        let (_, _, p) = classify_sample(0.2, 0.2, 0.3, f32::NAN, &ladder(at_most(0.5), at_least(0.1), at_most(0.6), at_least(1.0)), true);
        assert_eq!(p, 0.0);
        let (_, _, p) = classify_sample(0.2, 0.2, 0.3, 5.0, &ladder(at_most(0.5), at_least(0.1), at_most(0.6), at_least(1.0)), true);
        assert_eq!(p, 1.0);
    }

    /// A well whose VSH/PHIE/SWE were never computed classifies to NaN at every sample, which
    /// leaves net/ntg/hpv at exactly 0.0 — byte-identical to a genuine wet or shaly zone. The
    /// dialog, the Field Dashboard and the client PDF all printed that zero as if it were an
    /// answer. `n_classified` is the discriminator, so it must be 0 there and non-zero for a real
    /// interpretation; the zeros themselves stay unchanged.
    #[test]
    fn pay_summary_marks_an_uninterpreted_well_as_classifying_nothing() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "PAY-1", Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();

        // Only raw logs — exactly the state after importing a LAS and running nothing.
        let n = 20usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth.clone(), vec![50.0; n], nan.clone(), vec![0.2; n], vec![2.4; n],
            nan.clone(), nan,
        )
        .unwrap();

        let dbm = Mutex::new(conn);
        let req = PaySummaryRequest {
            discretisation: DiscretisationModel::Forward,
            input_set: None,
            well_ids: vec![well.clone()],
            vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
            swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
            perm_min: None,
            enabled_unset: Vec::new(),
            cutoff_use: Default::default(),
            skip_version: false,
            // Stats only: the point of the test is the returned rows, and this keeps it from
            // writing FLAG_* curves as a side effect.
            stats_only: true,
            custody: None,
            frame: Default::default(),
            weighting: Default::default(),
        };
        let rows = run_pay_summary(&dbm, &crate::reader_pool::ReaderPool::new(), &req).expect("summary runs on an uninterpreted well");
        assert!(!rows.is_empty(), "rows are still emitted — the well and its zone exist");
        for r in &rows {
            assert_eq!(
                r.n_classified, 0,
                "no sample can be classified without VSH/PHIE/SWE ({} {})",
                r.zone, r.flag
            );
            // The zeros are unchanged; the counter is what tells the consumer not to print them.
            assert_eq!(r.net, 0.0);
            assert_eq!(r.hpv, 0.0);
        }
    }

    /// SB-CUT-003 (P1). `14_cutoffs-summation-mc.md:944-955` — a summation **MUST** report
    /// `Gross`, `Net`, `NotNet` and `Unknown` as four separate quantities satisfying
    /// `Gross = Net + NotNet + Unknown` exactly, and `Unknown` — the footage whose flag could not
    /// be EVALUATED — **MUST NOT** be folded into `NotNet`.
    ///
    /// Techlog books a non-positive clipped interval as UNKNOWN, distinct from NOT-NET; IP marks
    /// nulls in-band with a `$$` pair inside a numeric column. Only the four-way partition is
    /// auditable: a zone reading 40 % net-to-gross because 60 % is shale and a zone reading 40 %
    /// because 55 % was never logged are the same two numbers and completely different rock.
    ///
    /// Pinned from both sides, because the invariant alone is satisfiable by the exact error the
    /// requirement names — fold every unjudgeable sample into `NotNet` and `Gross` still closes:
    ///
    /// * **A** — every component is its own expected footage on a zone the samples tile exactly,
    ///   so `NotNet` cannot silently absorb the missing-VSH interval.
    /// * **B** — footage carrying NO SAMPLE AT ALL lands in `Unknown`. This is what makes
    ///   deriving `Unknown` from the other three correct rather than convenient: accumulating it
    ///   from missing-flag samples alone would leave the identity broken wherever a zone extends
    ///   past the log, which is every zone bottomed on a marker the logging run did not reach.
    #[test]
    fn a_summation_partitions_gross_four_ways_and_books_unjudgeable_footage_as_unknown_not_as_notnet(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let tiled = seed_partition_well(&conn, "CUT-TILED");
        let overhang = seed_partition_well(&conn, "CUT-OVERHANG");
        // Declared 10 units deeper than the log reaches — the ordinary case of a zone bottomed on
        // a marker below TD of the run that logged it.
        db::upsert_zone_with_datum(
            &conn,
            &overhang,
            "OVERHANG",
            1000.0,
            1030.0,
            crate::schema_vocab::DepthDatum::Md,
        )
        .unwrap();

        let dbm = Mutex::new(conn);
        let rows = run_pay_summary(
            &dbm,
            &crate::reader_pool::ReaderPool::new(),
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![tiled.clone(), overhang.clone()],
                vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
            },
        )
        .expect("summary runs");

        // A — the undeclared "ALL" zone runs 1000..1020, which the samples tile exactly. PHIE and
        // SWE pass everywhere, so SAND, RESERVOIR and PAY partition identically and all three are
        // checked rather than one standing in for the others.
        let tiled_rows: Vec<_> = rows.iter().filter(|r| r.well_id == tiled).collect();
        assert_eq!(tiled_rows.len(), 3, "one row per summary flag");
        for r in &tiled_rows {
            assert_eq!(r.gross, 20.0, "{} gross", r.flag);
            assert_eq!(r.net, 10.0, "{} net — the ten samples that passed", r.flag);
            assert_eq!(
                r.not_net, 5.0,
                "{} not-net — the five samples that FAILED the cutoff, and ONLY those. 10.0 here \
                 means the missing-VSH interval was folded in, which is the error this pins.",
                r.flag
            );
            assert_eq!(
                r.unknown, 5.0,
                "{} unknown — the five samples with no VSH to judge",
                r.flag
            );
            assert_eq!(
                r.net + r.not_net + r.unknown,
                r.gross,
                "{} partition must close exactly",
                r.flag
            );
        }

        // B — the declared zone runs 1000..1030 while the log stops at 1020. Six sampled units are
        // unjudgeable (1015..1021, the last sample's forward interval now falling inside the zone)
        // and nine units carry no sample at all; both are footage whose flag could not be
        // evaluated, so both are Unknown.
        let over: Vec<_> = rows.iter().filter(|r| r.well_id == overhang).collect();
        assert_eq!(over.len(), 3, "one row per summary flag");
        for r in &over {
            assert_eq!(r.gross, 30.0, "{} gross is the declared zone, not the logged span", r.flag);
            assert_eq!(r.net, 10.0, "{} net", r.flag);
            assert_eq!(r.not_net, 5.0, "{} not-net", r.flag);
            assert_eq!(
                r.unknown, 15.0,
                "{} unknown — 6 unjudgeable sampled units plus 9 units nothing logged at all",
                r.flag
            );
            assert_eq!(
                r.net + r.not_net + r.unknown,
                r.gross,
                "{} partition must close exactly even where the samples do not reach the base",
                r.flag
            );
        }
    }

    /// SB-CUT-004 (P2). `14_cutoffs-summation-mc.md:966-975` — a summation **MUST** report both
    /// `N:G = Net/Gross` and `N:(G−Unknown)`, each labelled.
    ///
    /// The two differ by exactly the null fraction. Over a washed-out or partially-logged interval
    /// that difference is the whole argument about whether a net-to-gross is defensible, and no
    /// incumbent surfaces both — so an interpreter comparing one tool's number with another's has
    /// no way to know they are answering different questions.
    ///
    /// Pinned on three cases, because either ratio alone looks reasonable:
    ///
    /// * the zone the samples tile exactly, where the two still differ because some samples had
    ///   nothing to judge;
    /// * the zone declared below the logged interval, where they diverge by half — the case that
    ///   makes the pair worth reporting at all;
    /// * the well nobody interpreted, where the second ratio has NO denominator and must come back
    ///   MISSING rather than 0.00, which would read as "none of the judged rock is net".
    #[test]
    fn a_summation_reports_net_to_gross_over_all_footage_and_over_only_the_footage_it_could_judge() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let tiled = seed_partition_well(&conn, "NG-TILED");
        let overhang = seed_partition_well(&conn, "NG-OVERHANG");
        db::upsert_zone_with_datum(
            &conn,
            &overhang,
            "OVERHANG",
            1000.0,
            1030.0,
            crate::schema_vocab::DepthDatum::Md,
        )
        .unwrap();
        // Raw logs only — exactly the state after importing a LAS and running nothing, so every
        // sample is unjudgeable and Gross − Unknown is zero.
        let blank_id = uuid::Uuid::new_v4();
        db::insert_well(&conn, blank_id, "NG-BLANK", Some("Synthetic"), None, None).unwrap();
        let blank = blank_id.to_string();
        let bn = 20usize;
        let bdepth: Vec<f32> = (0..bn).map(|i| 1000.0 + i as f32).collect();
        let bnan = vec![f32::NAN; bn];
        db::insert_standard_curves(
            &conn, blank_id, bdepth, vec![50.0; bn], bnan.clone(), vec![0.2; bn],
            vec![2.4; bn], bnan.clone(), bnan,
        )
        .unwrap();

        let dbm = Mutex::new(conn);
        let rows = run_pay_summary(
            &dbm,
            &crate::reader_pool::ReaderPool::new(),
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![tiled.clone(), overhang.clone(), blank.clone()],
                vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
            },
        )
        .expect("summary runs");
        let near = |a: f32, b: f32| (a - b).abs() < 1e-6;

        // A — samples tile the zone exactly: 10 net of 20 gross, 5 of it unjudged.
        for r in rows.iter().filter(|r| r.well_id == tiled) {
            assert!(near(r.ntg, 0.5), "{} N:G is net/gross = 10/20, got {}", r.flag, r.ntg);
            assert!(
                near(r.ntg_known, 10.0 / 15.0),
                "{} N:(G-Unknown) is net over judged footage = 10/15, got {}",
                r.flag,
                r.ntg_known
            );
        }

        // B — the zone runs 10 units below the log. Half its footage was never judged, and the two
        // ratios diverge from 0.33 to 0.67: the same rock, described twice, honestly.
        for r in rows.iter().filter(|r| r.well_id == overhang) {
            assert!(near(r.ntg, 10.0 / 30.0), "{} N:G, got {}", r.flag, r.ntg);
            assert!(near(r.ntg_known, 10.0 / 15.0), "{} N:(G-Unknown), got {}", r.flag, r.ntg_known);
            assert!(
                r.ntg_known > r.ntg,
                "{} excluding unjudged footage can only RAISE the ratio; a second number equal to \
                 the first means Unknown never reached the denominator",
                r.flag
            );
        }

        // C — nothing was interpreted, so there is no judged footage to divide by. MISSING, never
        // zero: a printed 0.00 is a claim about rock nobody looked at.
        for r in rows.iter().filter(|r| r.well_id == blank) {
            assert_eq!(r.n_classified, 0, "{} the well really is uninterpreted", r.flag);
            assert!(near(r.unknown, r.gross), "{} every unit of it is Unknown", r.flag);
            assert!(
                r.ntg_known.is_nan(),
                "{} N:(G-Unknown) has no denominator here and must be MISSING, got {}",
                r.flag,
                r.ntg_known
            );
        }
    }

    /// SB-CUT-005 (P2). `14_cutoffs-summation-mc.md:972-985` — SandiBumi **MUST** check
    /// `Gross − (Net + NotNet + Unknown)` against a NAMED relative tolerance. Within tolerance the
    /// residual **MUST** be absorbed into the largest component **and the absorbed amount MUST
    /// appear in the result record**; outside it the summation **MUST** fail with a structured
    /// error.
    ///
    /// Tolerance `1e-7` relative, cited: `14_cutoffs-summation-mc.md:2083` (SB-CUT-T22), which is
    /// Techlog's `adjustFinal` shape with the `print` → result-field refinement. Nothing here is a
    /// petrophysical value; the footages below are NUMERICAL fixtures chosen so that a residual at
    /// the tolerance boundary is exactly representable in `f32` — at a realistic gross of tens of
    /// metres, `1e-7` relative is far below one ulp and no absorption could be observed at all.
    ///
    /// **The recorded amount is the whole requirement.** Techlog computes the same correction and
    /// prints it, which loses it: a reconciliation whose correction is not recorded is
    /// indistinguishable from no reconciliation.
    #[test]
    fn a_footage_partition_is_absorbed_into_its_largest_component_and_the_amount_recorded_or_else_refused(
    ) {
        // Gross 1e6 with ulp 0.0625, so a residual of exactly one ulp is 6.25e-8 relative — inside
        // the tolerance and still large enough to move an f32.
        let g = 1_000_000.0f32;
        let ulp = 0.0625f32;

        // A — within tolerance, and NET is the largest, so net is what moves.
        let r = reconcile_partition(g, 400_000.0, 300_000.0, 300_000.0 - ulp)
            .expect("one ulp of gross is inside 1e-7 relative");
        assert_eq!(r.net, 400_000.0 + ulp, "the residual lands on the largest component");
        assert_eq!(r.not_net, 300_000.0, "the other components are untouched");
        assert_eq!(r.unknown, 300_000.0 - ulp);
        assert_eq!(r.absorbed, ulp, "the absorbed amount is RECORDED, not printed and lost");
        assert_eq!(r.net + r.not_net + r.unknown, g, "and the partition now closes");

        // B — LARGEST, not first. Same residual, but Unknown now carries the most footage.
        let r = reconcile_partition(g, 200_000.0, 100_000.0, 700_000.0 - ulp)
            .expect("inside tolerance");
        assert_eq!(r.net, 200_000.0, "net must NOT absorb it merely for being first");
        assert_eq!(r.not_net, 100_000.0);
        assert_eq!(r.unknown, 700_000.0, "the largest component absorbs the residual");
        assert_eq!(r.absorbed, ulp);

        // C — outside tolerance the summation REFUSES, and the refusal carries the numbers. Four
        // ulps is 2.5e-7 relative, past 1e-7.
        let err = reconcile_partition(g, 400_000.0, 300_000.0, 300_000.0 - 4.0 * ulp)
            .expect_err("2.5e-7 relative is outside the 1e-7 tolerance and must refuse");
        assert_eq!(err.tolerance, PARTITION_TOLERANCE);
        assert!(
            (err.relative - 2.5e-7).abs() < 1e-12,
            "the refusal states the relative residual it measured, got {}",
            err.relative
        );
        let text = err.to_string();
        for needle in ["1000000", "residual", "1e-7"] {
            assert!(
                text.contains(needle),
                "a structured refusal names {needle} so a reader can act on it: {text}"
            );
        }

        // D — a residual of exactly zero is still a successful reconciliation recording zero, not a
        // special case that skips the check.
        let r = reconcile_partition(g, 400_000.0, 300_000.0, 300_000.0).expect("closes exactly");
        assert_eq!(r.absorbed, 0.0);
        assert_eq!(r.net, 400_000.0);

        // E — WIRED IN. An ordinary summary run carries the field and its partition closes, so the
        // guard above is protecting the real path rather than sitting in a test.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_partition_well(&conn, "RECON-1");
        let dbm = Mutex::new(conn);
        let rows = run_pay_summary(
            &dbm,
            &crate::reader_pool::ReaderPool::new(),
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well],
                vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
            },
        )
        .expect("an ordinary summary reconciles rather than refusing");
        assert!(!rows.is_empty());
        for r in &rows {
            assert_eq!(
                r.net + r.not_net + r.unknown,
                r.gross,
                "{} the reported partition closes after reconciliation",
                r.flag
            );
            assert!(
                r.residual_absorbed.abs() as f64 <= PARTITION_TOLERANCE * r.gross as f64,
                "{} a real run absorbs at most the tolerance, got {}",
                r.flag,
                r.residual_absorbed
            );
        }
    }

    /// SB-POR-057 (DEC-070, RULED 2026-08-18: "quick look only shows pay summation as
    /// visual not pay curves", confirmed "8, correct"). The D-N quick-look shortcuts are
    /// structurally a comparison-only class (Comparison* roles, custody mnemonics distinct
    /// from the shared PHIE/PHIT, ancestry module identity as provenance) and the pay
    /// engine never reads them: the candidate list is the one canonical name, a well whose
    /// only porosity is the quick-look curve is reported NOT INTERPRETED with the refusal
    /// recorded on the row, and absence of any porosity is deliberately NOT marked - the
    /// flag means "present and excluded". Supersedes the DEC-042 pay-eligible fallback.
    /// Display overlay needs no gate here: plot layers read curves by mnemonic and nothing
    /// added excludes PHIE_DN_LIM from them.
    #[test]
    fn the_quick_look_porosity_never_feeds_the_summed_numbers_and_its_refusal_is_recorded_on_the_row(
    ) {
        // A - the comparison-only class is structural: every registered phi_dn porosity
        // output carries a Comparison* role, and the limited pair lands under its own
        // custody mnemonics, never the shared authoritative names.
        let dn = modules::list_modules()
            .into_iter()
            .find(|spec| spec.name == "phi_dn")
            .expect("phi_dn ships");
        let mut classified = 0usize;
        for argument in &dn.args {
            let Some(contract) = argument.porosity_output.as_ref() else { continue };
            classified += 1;
            assert!(
                format!("{:?}", contract.output_role).starts_with("Comparison"),
                "phi_dn.{} must stay comparison-typed, got {:?}",
                argument.name,
                contract.output_role
            );
        }
        assert_eq!(classified, 4, "the whole quick-look output set is comparison-typed");
        let limited = dn
            .args
            .iter()
            .find(|argument| argument.name == "PHIE")
            .expect("the limited effective output exists");
        // `log_out_as` records the custody rename in the argument's default pattern.
        assert_eq!(
            limited.default,
            modules::PHIE_DN_LIMITED_DEFAULT,
            "the limited quick-look curve writes under its own custody mnemonic, not PHIE"
        );

        // Fixture wells share seed_weighting_well's rock (avg_swe 0.30 phi-weighted,
        // avg_phie 0.20 thickness-weighted over 10 net units when summed).
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let auth = seed_weighting_well(&conn, "QL-AUTH", "PHIE");
        let ql_only = seed_weighting_well(&conn, "QL-ONLY", modules::PHIE_DN_LIMITED_DEFAULT);
        // A well carrying BOTH: the quick-look curve holds DIFFERENT numbers (0.05
        // everywhere), so a leak into the summation would move avg_phie visibly.
        let both_well = seed_weighting_well(&conn, "QL-BOTH", "PHIE");
        let depth: Vec<f32> = (0..11).map(|i| 1000.0 + i as f32).collect();
        equations::write_computed_curve(
            &conn, &both_well, &depth, modules::PHIE_DN_LIMITED_DEFAULT, &vec![0.05f32; 11],
        )
        .unwrap();
        // A well with NO porosity of any kind (the seeded curve lands under an alien name
        // nothing resolves), to prove absence is not marked as exclusion.
        let none = seed_weighting_well(&conn, "QL-NONE", "PHIX_UNRESOLVED");
        let dbm = Mutex::new(conn);

        let rows = run_pay_summary(
            &dbm,
            &crate::reader_pool::ReaderPool::new(),
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![auth.clone(), ql_only.clone(), both_well.clone(), none.clone()],
                vsh_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.05, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
            },
        )
        .expect("the summary runs");
        let pay = |well: &str| -> &PaySummaryRow {
            rows.iter()
                .find(|row| row.well_id == well && row.flag == "PAY")
                .expect("a PAY row")
        };
        let near = |a: f32, b: f32| (a - b).abs() < 1e-6;

        // B - the authoritative well sums exactly as before, unmarked.
        let a = pay(&auth);
        assert!(near(a.avg_phie, 0.20) && near(a.avg_swe, 0.30) && near(a.net, 10.0));
        assert!(!a.quicklook_phie_excluded);

        // C - the quick-look-only well is NOT summed: not interpreted for pay, with the
        // refusal recorded. Its curve held real numbers; none of them reached a sum.
        let q = pay(&ql_only);
        assert_eq!(q.n_classified, 0, "the quick-look curve never feeds classification");
        assert!(near(q.net, 0.0) && near(q.unknown, q.gross), "unjudged, not wet");
        assert!(q.quicklook_phie_excluded, "the row records the DEC-070 refusal");
        assert!(
            rows.iter()
                .filter(|row| row.well_id == ql_only)
                .all(|row| row.quicklook_phie_excluded),
            "per well: every flag row of the well carries the mark"
        );

        // D - beside an authoritative PHIE the quick-look curve neither leaks nor marks:
        // identical averages to the plain well, flag false.
        let b = pay(&both_well);
        assert!(
            near(b.avg_phie, 0.20) && near(b.avg_swe, 0.30),
            "the 0.05 quick-look values must not move a summed average: {} {}",
            b.avg_phie,
            b.avg_swe
        );
        assert!(!b.quicklook_phie_excluded, "nothing was excluded - PHIE answered");

        // E - a well with no porosity AT ALL is not marked: the flag means "present and
        // excluded", never "absent".
        let n = pay(&none);
        assert_eq!(n.n_classified, 0);
        assert!(!n.quicklook_phie_excluded, "absence is not exclusion");

        // F - the refusal crosses the wire as a typed boolean, like its precedent.
        let wire = serde_json::to_value(q).expect("a row serializes");
        assert!(wire["quicklook_phie_excluded"].is_boolean());
    }

    /// SB-CUT-009 (P1, SILENT-WRONGNESS). `14_cutoffs-summation-mc.md:1033-1048` — porosity
    /// weighting of an averaged curve **MUST** be controlled by an explicit per-curve flag stored
    /// with the curve's averaging configuration, and SandiBumi **MUST NOT** infer it from the
    /// curve's name or family.
    ///
    /// The harm is Techlog's, quoted in the chapter: *"the SW curve is weighted by POR but the SWE
    /// is not weighted"* — a curve loses its φ-weighting because of how it happens to be spelled,
    /// and on this fixture that is 0.40 against 0.30, ten saturation units, with nothing on the
    /// page to say which was used.
    ///
    /// The as-built named two gaps and both are closed here: the φ-weighted form could not be
    /// REQUESTED for another curve, and could not be SWITCHED OFF.
    ///
    /// Defaults are cited, not chosen: the φ-weighted saturation `Σ(Sw·φ·h)/Σ(φ·h)` is agreed by
    /// all three vendors (`:1041-1042`) and is what the engine already did, so nothing moves for a
    /// caller who declares nothing.
    #[test]
    fn zone_averaging_weighting_is_declared_per_curve_and_never_inferred_from_the_curve_name() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let plain = seed_weighting_well(&conn, "WGT-PLAIN", "PHIE");
        let aliased = seed_weighting_well(&conn, "WGT-ALIAS", modules::PHIE_DN_LIMITED_DEFAULT);
        let dbm = Mutex::new(conn);
        let near = |a: f32, b: f32| (a - b).abs() < 1e-6;

        let run = |wells: Vec<String>, weighting: BTreeMap<String, AverageWeighting>| {
            run_pay_summary(
                &dbm,
                &crate::reader_pool::ReaderPool::new(),
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: wells,
                    // Permissive on purpose: every sample must pass, so the only thing that can
                    // move an average is the weighting under test.
                    vsh_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                    phie_min: Some(CutoffEntry { value: 0.05, unit: "v/v".into() }.into()),
                    swe_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                    perm_min: None,
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    frame: Default::default(),
                    weighting,
                },
            )
            .expect("summary runs")
        };
        let pay = |rows: &[PaySummaryRow], well: &str| -> PaySummaryRow {
            rows.iter().find(|r| r.well_id == well && r.flag == "PAY").expect("a PAY row").clone()
        };

        // A — DECLARING NOTHING keeps the vendor-agreed behaviour: saturation φ-weighted, the
        // others by thickness. A caller who never heard of this flag sees no change.
        let base = run(vec![plain.clone()], BTreeMap::new());
        let r = pay(&base, &plain);
        assert!(near(r.avg_swe, 0.30), "default SWE is phi-weighted 0.30, got {}", r.avg_swe);
        assert!(near(r.avg_vsh, 0.25), "default VSH is thickness-weighted 0.25, got {}", r.avg_vsh);
        assert!(near(r.avg_phie, 0.20), "default PHIE is thickness-weighted 0.20, got {}", r.avg_phie);

        // B — it can be SWITCHED OFF. Declaring thickness weighting for the saturation slot moves
        // the answer to 0.40, which is the number Techlog silently produces for a curve spelled
        // the wrong way. Here it is a declaration, not an accident.
        let off = run(
            vec![plain.clone()],
            BTreeMap::from([("SWE".to_string(), AverageWeighting::Thickness)]),
        );
        assert!(
            near(pay(&off, &plain).avg_swe, 0.40),
            "declared thickness weighting must actually change the average, got {}",
            pay(&off, &plain).avg_swe
        );

        // C — it can be REQUESTED FOR ANOTHER CURVE, which the hard-wired version could not do at
        // all. Vsh φ-weighted is 0.175 against its thickness-weighted 0.25.
        let on = run(
            vec![plain.clone()],
            BTreeMap::from([("VSH".to_string(), AverageWeighting::Porosity)]),
        );
        assert!(
            near(pay(&on, &plain).avg_vsh, 0.175),
            "phi weighting must be available to any averaged curve, got {}",
            pay(&on, &plain).avg_vsh
        );
        assert!(
            near(pay(&on, &plain).avg_swe, 0.30),
            "and declaring one curve must not disturb another"
        );

        // D — updated under DEC-070 (RULED 2026-08-18), which removed the DEC-042 fallback
        // this arm rode on: a well whose porosity exists ONLY under the quick-look custody
        // mnemonic is no longer summed AT ALL, so the name cannot influence a weighting
        // decision because the curve never reaches the averaging - the strongest form of
        // "never inferred from the name". The refusal is observable on the row rather than
        // silent, and the anti-inference contract stays behaviourally pinned by arms A-C
        // and structurally by the scan below.
        let both = run(vec![plain.clone(), aliased.clone()], BTreeMap::new());
        let (p, a) = (pay(&both, &plain), pay(&both, &aliased));
        assert!(near(p.avg_swe, 0.30), "the authoritative well still sums");
        assert_eq!(a.n_classified, 0, "the quick-look-only well is not interpreted for pay");
        assert!(a.quicklook_phie_excluded, "and the row records why");
        assert!(!p.quicklook_phie_excluded, "a well summed from PHIE carries no such mark");

        // ...and structurally, so a future edit cannot quietly reintroduce the inference. The
        // resolver is keyed on the SLOT a curve fills — a role, fixed at compile time — and the
        // one place the summation holds a resolved MNEMONIC is `phie_curve`. Proving that name
        // never reaches the resolver is the difference between "does not infer from the name" and
        // "happens not to today". A slot key spelled like a mnemonic is not an inference: it is
        // the position, and arm D above is what proves it behaviourally.
        // Truncated at the test module, or the scan matches the very strings it is asserting
        // about and passes for free — this file is its own subject. Cut on `mod tests {` rather
        // than on `#[cfg(test)]`, which also marks three production-side test helpers far above
        // here and would silently truncate away the code actually under scan.
        let whole = include_str!("paysummary.rs");
        let source = &whole[..whole.find("\nmod tests {").expect("the test module is below")];
        assert!(
            !source.contains("weighting_for(req, &phie_curve")
                && !source.contains("weighting_for(&req, &phie_curve"),
            "the resolved porosity mnemonic must never be passed to the weighting resolver"
        );
        let start = source.find("pub fn weighting_for").expect("the resolver exists");
        let body = &source[start..start + 700];
        for banned in ["phie_curve", "family", "curve_meta", "mnemonic"] {
            assert!(
                !body.contains(banned),
                "the weighting resolver must not consult {banned}; it sees a slot and a declaration"
            );
        }
    }

    /// SB-CUT-010 (P1, SILENT-WRONGNESS). `14_cutoffs-summation-mc.md:1050-1062` — `HCPV` computed
    /// by direct summation `Σφ(1−Sw)h` **MUST** equal `Net × φ̄ × (1 − S̄w)` rebuilt from the
    /// reported averages, to floating-point tolerance, for every emitted zone.
    ///
    /// The expected value is an INDEPENDENT algebraic identity, not a re-derivation of the code —
    /// which is what the register meant by *shared implementation is not an independent proof*:
    ///
    /// ```text
    /// Net · φ̄ · (1 − S̄w) = Net · (Σφh/Net) · (1 − Σ Sw·φ·h / Σφh)
    ///                     = Σφh − Σ Sw·φ·h  =  Σ φh(1 − Sw)  =  HCPV
    /// ```
    ///
    /// It cancels ONLY because `S̄w` is φ-weighted. With a thickness-weighted `S̄w` the `Σφh` does
    /// not cancel and the two sides part company — so the identity is what locks SB-CUT-009's
    /// weighting choice in place, and the negative control is the half that carries the proof. On
    /// this fixture that is 1.4 against 1.2, a 14 % error in the hydrocarbon column.
    ///
    /// **Precondition, stated rather than assumed:** φ and Sw must be valid across the whole net
    /// interval. Where Sw is missing over part of net, `Net · φ̄` counts footage `HCPV` cannot, and
    /// the identity is not claimed — the engine deliberately normalises each average over the
    /// footage ITS OWN curve was valid on, which is a separate pinned rule. T07's fixture is a
    /// flagged interval with varying φ and Sw, so the precondition holds here by construction.
    #[test]
    fn hydrocarbon_pore_volume_summed_directly_equals_the_volume_rebuilt_from_the_reported_averages_in_both_engines(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_weighting_well(&conn, "HCPV-1", "PHIE");
        let dbm = Mutex::new(conn);
        let run = |weighting: BTreeMap<String, AverageWeighting>| {
            run_pay_summary(
                &dbm,
                &crate::reader_pool::ReaderPool::new(),
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                    phie_min: Some(CutoffEntry { value: 0.05, unit: "v/v".into() }.into()),
                    swe_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                    perm_min: None,
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    frame: Default::default(),
                    weighting,
                },
            )
            .expect("summary runs")
        };
        let rebuilt = |r: &PaySummaryRow| -> f64 {
            r.net as f64 * r.avg_phie as f64 * (1.0 - r.avg_swe as f64)
        };

        // A — every emitted zone and flag closes. The absolute value is asserted too, so an
        // engine that returned zeros everywhere could not satisfy the identity vacuously.
        let rows = run(BTreeMap::new());
        assert!(!rows.is_empty());
        for r in &rows {
            assert!(
                (r.hpv as f64 - 1.4).abs() < 1e-6,
                "{} HCPV by direct summation is 1.4 on this fixture, got {}",
                r.flag,
                r.hpv
            );
            assert!(
                (r.hpv as f64 - rebuilt(r)).abs() / r.hpv as f64 <= 1e-6,
                "{} the identity must close: summed {} against rebuilt {}",
                r.flag,
                r.hpv,
                rebuilt(r)
            );
        }

        // B — the negative control the chapter demands. Declaring thickness-weighted Sw leaves the
        // direct summation alone (it never used an average) and moves the rebuilt side to 1.2. If
        // this ever stops failing, the two sides have stopped being independent.
        let off = run(BTreeMap::from([("SWE".to_string(), AverageWeighting::Thickness)]));
        for r in &off {
            assert!(
                (r.hpv as f64 - 1.4).abs() < 1e-6,
                "{} direct summation is unaffected by a weighting choice",
                r.flag
            );
            assert!(
                (rebuilt(r) - 1.2).abs() < 1e-6,
                "{} thickness-weighted Sw rebuilds 1.2, got {}",
                r.flag,
                rebuilt(r)
            );
            assert!(
                (r.hpv as f64 - rebuilt(r)).abs() / r.hpv as f64 > 1e-3,
                "{} the identity MUST fail with thickness-weighted Sw - if it holds either way it \
                 is proving nothing about the weighting",
                r.flag
            );
        }

        // C — the same identity in the Monte Carlo engine, which emits its own per-zone averages
        // and its own HPV per realization. Checked on the realization's metrics, NOT on the
        // P10/P50/P90 bundle: percentiles do not commute with a product, so the identity is a
        // statement about one realization and asserting it across percentiles would be false.
        let n = 11usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let step = vec![1.0f32; n];
        let half = |lo: f32, hi: f32| -> Vec<f32> {
            (0..n).map(|i| if i < 5 { lo } else { hi }).collect()
        };
        let m = crate::montecarlo::zone_metrics(
            DiscretisationModel::Forward, // DEC-071: fixture derived under FORWARD
            &half(0.10, 0.40),
            &half(0.30, 0.10),
            &half(0.20, 0.60),
            &vec![f32::NAN; n],
            &depth,
            &step,
            &db::ZoneEntry {
                zone_name: "ALL".into(),
                top_depth: 1000.0,
                bottom_depth: 1010.0,
                depth_datum: crate::schema_vocab::DepthDatum::Md,
            },
            &crate::montecarlo::Cutoffs {
                vsh_max: at_most(0.9),
                phie_min: at_least(0.05),
                swe_max: at_most(0.9),
                perm_min: None,
            },
            false,
        );
        let mc_rebuilt = m.net as f64 * m.avg_phie as f64 * (1.0 - m.avg_swe as f64);
        assert!(
            (m.hpv as f64 - 1.4).abs() < 1e-6,
            "Monte Carlo sums the same 1.4, got {}",
            m.hpv
        );
        assert!(
            (m.hpv as f64 - mc_rebuilt).abs() / m.hpv as f64 <= 1e-6,
            "the identity must close in the Monte Carlo engine too: {} against {}",
            m.hpv,
            mc_rebuilt
        );
    }

    /// SB-CUT-001 (DEC-071, RULED 2026-08-18: "centred"): the thickness discretisation
    /// model is a request parameter DEFAULTING TO CENTRED - a sample's slab straddles its
    /// depth - while FORWARD (the chapter's TOPS rule) stays selectable and reproduces the
    /// shipped numbers. The two models are proven DISTINCT on a data-edge zone, hand-derived
    /// both ways; every record names the model that produced it; and the Monte Carlo engine
    /// agrees with the deterministic summary under BOTH models, so an MC P50 can never
    /// disagree with the pay summary for this reason.
    #[test]
    fn the_discretisation_model_defaults_to_centred_and_forward_reproduces_the_shipped_numbers() {
        // A - the default is CENTRED, on the enum and over the wire: a request that never
        // mentions the field deserializes to the ruled default, so every pre-ruling caller
        // gets CENTRED rather than silently keeping the old rule.
        assert_eq!(DiscretisationModel::default(), DiscretisationModel::Centred);
        assert_eq!(DiscretisationModel::Centred.token(), "CENTRED");
        assert_eq!(DiscretisationModel::Forward.token(), "TOPS");
        let wire: PaySummaryRequest = serde_json::from_value(serde_json::json!({
            "well_ids": [],
            "vsh_max": null,
            "phie_min": null,
            "swe_max": null,
            "perm_min": null,
        }))
        .expect("a pre-ruling request still deserializes");
        assert_eq!(wire.discretisation, DiscretisationModel::Centred);

        // B - the ONE slab derivation: centred straddles, forward hangs down.
        assert_eq!(
            sample_slab(1000.0, 1.0, DiscretisationModel::Centred),
            (999.5, 1000.5)
        );
        assert_eq!(
            sample_slab(1000.0, 1.0, DiscretisationModel::Forward),
            (1000.0, 1001.0)
        );

        // C - hand-derived, both ways, on a zone straddling the data edge. Samples at
        // 1000..=1003 m, step 1 m, all-pay curves; zone [999, 1001). FORWARD: only the
        // 1000 m slab [1000, 1001) overlaps -> net 1.0. CENTRED: the 1000 m slab
        // [999.5, 1000.5) contributes 1.0 and the 1001 m slab [1000.5, 1001.5) contributes
        // 0.5 -> net 1.5. Jauhar accepted exactly this kind of movement.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-CENTRED", None, None, None).unwrap();
        let well = id.to_string();
        let n = 4usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n],
            vec![2.35; n], nan.clone(), nan,
        )
        .unwrap();
        for (curve, value) in [("VSH", 0.10), ("PHIE", 0.30), ("SWE", 0.20)] {
            equations::write_computed_curve(&conn, &well, &depth, curve, &vec![value; n])
                .unwrap();
        }
        db::upsert_zone_with_datum(
            &conn, &well, "EDGE", 999.0, 1001.0, crate::schema_vocab::DepthDatum::Md,
        )
        .unwrap();
        let dbm = Mutex::new(conn);
        let run = |model: DiscretisationModel| -> PaySummaryRow {
            run_pay_summary(
                &dbm,
                &crate::reader_pool::ReaderPool::new(),
                &PaySummaryRequest {
                    discretisation: model,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                    phie_min: Some(CutoffEntry { value: 0.05, unit: "v/v".into() }.into()),
                    swe_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                    perm_min: None,
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    frame: Default::default(),
                    weighting: Default::default(),
                },
            )
            .unwrap()
            .into_iter()
            .find(|row| row.flag == "PAY")
            .expect("a PAY row")
        };
        let forward = run(DiscretisationModel::Forward);
        let centred = run(DiscretisationModel::Centred);
        assert!(
            (forward.net - 1.0).abs() < 1e-6,
            "FORWARD reproduces the shipped number 1.0, got {}",
            forward.net
        );
        assert!(
            (centred.net - 1.5).abs() < 1e-6,
            "CENTRED counts the straddling halves: expected 1.5, got {}",
            centred.net
        );
        assert_eq!(forward.discretisation_model, "TOPS");
        assert_eq!(centred.discretisation_model, "CENTRED");

        // D - the DEC-071-noted contract: the Monte Carlo engine's net agrees with the
        // deterministic pay summary for the same inputs, under BOTH models.
        let step = vec![1.0f32; n];
        for (model, expected) in [
            (DiscretisationModel::Forward, forward.net),
            (DiscretisationModel::Centred, centred.net),
        ] {
            let m = crate::montecarlo::zone_metrics(
                model,
                &vec![0.10f32; n],
                &vec![0.30f32; n],
                &vec![0.20f32; n],
                &vec![f32::NAN; n],
                &depth,
                &step,
                &db::ZoneEntry {
                    zone_name: "EDGE".into(),
                    top_depth: 999.0,
                    bottom_depth: 1001.0,
                    depth_datum: crate::schema_vocab::DepthDatum::Md,
                },
                &crate::montecarlo::Cutoffs {
                    vsh_max: at_most(0.9),
                    phie_min: at_least(0.05),
                    swe_max: at_most(0.9),
                    perm_min: None,
                },
                false,
            );
            assert!(
                (m.net - expected).abs() < 1e-6,
                "Monte Carlo net {} must agree with the pay summary {} under {:?}",
                m.net,
                expected,
                model
            );
        }
    }

    /// SB-CUT-002 / SB-CUT-T02b's identity half. Source: `14_cutoffs-summation-mc.md:927-942` —
    /// every record carrying a thickness, a net, a net-to-gross or a thickness-weighted average
    /// MUST carry the discretisation model that produced it and the sample interval it was
    /// computed on; a consumer must never have to infer either. IP ships TWO definitions of
    /// "Net" in one product under the same heading and labels neither, and net-to-gross is not
    /// scale-invariant (T4: 0.55 → 0.75 → 1.0 across three blocking steps).
    #[test]
    fn every_thickness_bearing_result_names_its_discretisation_model_and_the_step_it_ran_on() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        // Two wells, SAME rock, DIFFERENT frames: 1.0 m and 0.5 m steps.
        let mut wells = Vec::new();
        for (name, step) in [("STEP-ONE", 1.0f32), ("STEP-HALF", 0.5f32)] {
            let id = uuid::Uuid::new_v4();
            db::insert_well(&conn, id, name, Some("Synthetic"), None, None).unwrap();
            let well = id.to_string();
            let n = (20.0 / step) as usize + 1;
            let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * step).collect();
            let nan = vec![f32::NAN; n];
            db::insert_standard_curves(
                &conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n],
                vec![2.35; n], nan.clone(), nan,
            )
            .unwrap();
            for (curve, value) in [("VSH", 0.10), ("PHIE", 0.30), ("SWE", 0.20)] {
                equations::write_computed_curve(&conn, &well, &depth, curve, &vec![value; n])
                    .unwrap();
            }
            db::upsert_zone_with_datum(
                &conn, &well, "Z", 1000.0, 1010.0, crate::schema_vocab::DepthDatum::Md,
            )
            .unwrap();
            wells.push((well, step));
        }
        let dbm = Mutex::new(conn);
        let rows = run_pay_summary(
            &dbm,
            &crate::reader_pool::ReaderPool::new(),
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: wells.iter().map(|(w, _)| w.clone()).collect(),
                vsh_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.05, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
            },
        )
        .unwrap();

        // A. Every row STATES the model — the shipped TOPS rule — and its own well's step.
        for r in &rows {
            assert_eq!(
                r.discretisation_model, DISCRETISATION_MODEL,
                "a thickness-bearing record must name the model that produced it"
            );
            let expected = wells.iter().find(|(w, _)| *w == r.well_id).unwrap().1;
            assert!(
                (r.sample_interval - expected).abs() < 1e-6,
                "{}: the record must carry ITS OWN frame's step {expected}, got {}",
                r.well_name,
                r.sample_interval
            );
        }

        // B. The whole point: two records over the SAME rock at different steps are
        //    distinguishable BY THE RECORD, with no depth column to re-derive it from.
        let one = rows.iter().find(|r| r.well_name == "STEP-ONE").unwrap();
        let half = rows.iter().find(|r| r.well_name == "STEP-HALF").unwrap();
        assert!(
            (one.sample_interval - half.sample_interval).abs() > 0.4,
            "records computed at different steps must be distinguishable"
        );

        // C. The workbook carries both — per row, since wells in one workbook differ in frame —
        //    and as NUMBERS-stay-numbers: the step is a numeric cell, the model a text cell.
        let sheet = crate::office::pay_sheet(&rows, "m");
        let model_col = sheet
            .columns
            .iter()
            .position(|c| c.header == "Model")
            .expect("the pay sheet must carry the discretisation model");
        let step_col = sheet
            .columns
            .iter()
            .position(|c| c.header.starts_with("Step"))
            .expect("the pay sheet must carry the sample interval");
        let mut seen_steps = std::collections::BTreeSet::new();
        for row in &sheet.rows {
            match (&row[model_col], &row[step_col]) {
                (crate::office::Cell::Text(model), crate::office::Cell::Num(step)) => {
                    assert_eq!(model, DISCRETISATION_MODEL);
                    seen_steps.insert((step * 1000.0).round() as i64);
                }
                other => panic!("model must be text and step numeric, got {other:?}"),
            }
        }
        assert_eq!(
            seen_steps.into_iter().collect::<Vec<_>>(),
            vec![500, 1000],
            "both frames' steps must survive into the workbook"
        );

        // D. The Monte Carlo bundle carries the same identity fields (populated by the same
        //    helpers); their presence on the struct is pinned here, their end-to-end values by
        //    the MC engine's own DB tests running under the same construction site.
        let median = median_sample_interval(&[0.5, 0.5, 0.5, 1.0]);
        assert!((median - 0.5).abs() < 1e-9, "the median step is the regularize convention");
        assert!(median_sample_interval(&[0.0, -1.0]).is_nan(), "no positive step is NaN, not zero");
    }

    /// SB-CUT-011 (P1). `14_cutoffs-summation-mc.md:1064-1075` — a sample that passes every
    /// cut-off but lies outside every defined zone **MUST NOT** contribute to any cumulative
    /// result or summary statistic (IP's stated zone-membership rule).
    ///
    /// Easy to violate in a single-pass implementation that applies cut-offs before zone
    /// membership, and easy to test WRONGLY: an out-of-zone sample that also fails a cut-off is
    /// excluded for the wrong reason and proves nothing. So the samples outside every zone here
    /// are asserted to pass all three cut-offs on their own, and they carry values found nowhere
    /// else — φ 0.50 against the zones' 0.30 and 0.10 — so any leak moves a number.
    ///
    /// Three intervals, two of them declared, prove membership is what decides: the same engine
    /// counts a sample in the zone that contains it, and not in the one next door.
    #[test]
    fn a_sample_outside_every_declared_zone_contributes_to_no_summary_statistic_however_well_it_passes_the_cutoffs(
    ) {
        // First, the guard that makes the rest meaningful: these values clear every cut-off.
        assert_eq!(
            classify_sample(0.80, 0.50, 0.85, f32::NAN, &ladder(at_most(0.9), at_least(0.05), at_most(0.9), None), false),
            (1.0, 1.0, 1.0),
            "the out-of-zone samples must pass SAND, RESERVOIR and PAY on their own merits - \
             otherwise their absence below proves a cut-off worked, not the zone rule"
        );

        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "ZONE-EDGE", Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let n = 25usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n],
            vec![2.35; n], nan.clone(), nan,
        )
        .unwrap();
        // Three bands: UPPER (declared), LOWER (declared), and BELOW — outside every zone.
        let band = |a: f32, b: f32, c: f32| -> Vec<f32> {
            (0..n).map(|i| if i < 10 { a } else if i < 20 { b } else { c }).collect()
        };
        equations::write_computed_curve(&conn, &well, &depth, "VSH", &band(0.10, 0.40, 0.80))
            .unwrap();
        equations::write_computed_curve(&conn, &well, &depth, "PHIE", &band(0.30, 0.10, 0.50))
            .unwrap();
        equations::write_computed_curve(&conn, &well, &depth, "SWE", &band(0.20, 0.60, 0.85))
            .unwrap();
        for (name, top, base) in
            [("UPPER", 1000.0, 1010.0), ("LOWER", 1010.0, 1020.0)]
        {
            db::upsert_zone_with_datum(
                &conn, &well, name, top, base, crate::schema_vocab::DepthDatum::Md,
            )
            .unwrap();
        }

        let dbm = Mutex::new(conn);
        let rows = run_pay_summary(
            &dbm,
            &crate::reader_pool::ReaderPool::new(),
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well.clone()],
                vsh_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.05, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
            },
        )
        .expect("summary runs");
        let near = |a: f32, b: f32| (a - b).abs() < 1e-6;
        let row = |zone: &str, flag: &str| -> PaySummaryRow {
            rows.iter()
                .find(|r| r.zone == zone && r.flag == flag)
                .unwrap_or_else(|| panic!("a {flag} row for {zone}"))
                .clone()
        };

        assert_eq!(rows.len(), 6, "two declared zones by three flags, and nothing for BELOW");
        assert!(
            !rows.iter().any(|r| r.zone != "UPPER" && r.zone != "LOWER"),
            "the footage below every zone must not produce a zone of its own"
        );

        for flag in SUMMARY_FLAGS {
            let u = row("UPPER", flag);
            assert!(near(u.net, 10.0), "{flag} UPPER net is its own ten units, got {}", u.net);
            assert!(near(u.avg_phie, 0.30), "{flag} UPPER phi is 0.30, got {}", u.avg_phie);
            assert!(near(u.avg_swe, 0.20), "{flag} UPPER Sw is 0.20, got {}", u.avg_swe);
            assert!(near(u.hpv, 2.4), "{flag} UPPER HPV is 2.4, got {}", u.hpv);

            let l = row("LOWER", flag);
            assert!(near(l.net, 10.0), "{flag} LOWER net is its own ten units, got {}", l.net);
            assert!(near(l.avg_phie, 0.10), "{flag} LOWER phi is 0.10, got {}", l.avg_phie);
            assert!(near(l.avg_swe, 0.60), "{flag} LOWER Sw is 0.60, got {}", l.avg_swe);
            assert!(near(l.hpv, 0.4), "{flag} LOWER HPV is 0.4, got {}", l.hpv);

            // The below-every-zone band carries φ 0.50 and Sw 0.85, which appear in neither row —
            // stated as its own assertion so a leak reads as what it is rather than as a stray
            // arithmetic error somewhere above.
            for r in [&u, &l] {
                assert!(
                    r.avg_phie < 0.31 && r.avg_swe < 0.61,
                    "{flag} {} shows a trace of the samples below every zone: phi {} Sw {}",
                    r.zone, r.avg_phie, r.avg_swe
                );
            }
        }

        // The register asks for ONE fixture across all three paths, because the rule is easy to
        // hold in the summation and lose in a sibling that walks the same curves.

        // Path 2 — the cutoff SWEEP, restricted to UPPER. Every sample clears a VSH cutoff of 0.9,
        // so net is decided by zone membership alone and must be UPPER's own ten units.
        let sweep = run_cutoff_sweep(
            &dbm,
            &CutoffSweepRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well.clone()],
                property: "VSH".into(),
                vsh_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.05, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                perm_min: None,
                // Every sample's VSH is at most 0.80, so it clears every step of this range and
                // net is decided by zone membership alone across the whole sweep.
                sweep_min: 0.85,
                sweep_max: 0.95,
                steps: 3,
                metric: "NET".into(),
                zone: Some("UPPER".into()),
                dst_dataset: None,
            },
        )
        .expect("sweep runs");
        let series = sweep.series.first().expect("one well, one series");
        assert!(
            near(series.gross as f32, 10.0),
            "the sweep's gross is UPPER's own thickness, got {}",
            series.gross
        );
        assert!(
            series.values.iter().all(|v| near(*v as f32, 10.0)),
            "the sweep must count only UPPER's samples, got {:?}",
            series.values
        );

        // Path 3 — MONTE CARLO's per-realization zone metrics on the same arrays.
        let step = vec![1.0f32; n];
        let m = crate::montecarlo::zone_metrics(
            DiscretisationModel::Forward, // DEC-071: fixture derived under FORWARD
            &band(0.10, 0.40, 0.80),
            &band(0.30, 0.10, 0.50),
            &band(0.20, 0.60, 0.85),
            &vec![f32::NAN; n],
            &depth,
            &step,
            &db::ZoneEntry {
                zone_name: "UPPER".into(),
                top_depth: 1000.0,
                bottom_depth: 1010.0,
                depth_datum: crate::schema_vocab::DepthDatum::Md,
            },
            &crate::montecarlo::Cutoffs {
                vsh_max: at_most(0.9),
                phie_min: at_least(0.05),
                swe_max: at_most(0.9),
                perm_min: None,
            },
            false,
        );
        assert!(near(m.net, 10.0), "Monte Carlo counts only UPPER's samples, got {}", m.net);
        assert!(near(m.avg_phie, 0.30), "Monte Carlo phi is UPPER's 0.30, got {}", m.avg_phie);
        assert!(near(m.hpv, 2.4), "Monte Carlo HPV is UPPER's 2.4, got {}", m.hpv);
    }

    /// SB-CUT-012 (P2). `14_cutoffs-summation-mc.md:1078-1091` — a summation result **MUST** carry
    /// `{frame, weights_source}` with `frame` one of MD, TVD, TVDSS or TST; MD and TVD summations
    /// **MUST** be separate records; and SandiBumi **MUST NOT** present a TVD result as a
    /// rescaling of an MD result.
    ///
    /// The per-sample weight is `Δz` in MD and `Δz·cos θ` in TVD, so it is the WEIGHTS that
    /// differ, not merely the totals. In a 60° hold section they differ by a factor of two, which
    /// is why IP says TVD zonal averages *"could be considerably different"* — the frame is part of
    /// a result's identity, not a display option. A net thickness quoted in a deviated field
    /// without its frame is a number a reader cannot use.
    ///
    /// **The summation is MD-only and this row does not change that.** It closes the MUST the
    /// honest way for an ABSENT row: every result declares the frame it was actually computed in,
    /// and a request for a frame whose weights SandiBumi cannot compute is REFUSED by name rather
    /// than served MD numbers under a TVD label — which is precisely the third clause.
    #[test]
    fn a_summation_declares_the_depth_frame_its_weights_came_from_and_refuses_one_it_cannot_weight()
    {
        // The four frames the chapter names — Techlog offers four, IP two, and the union is the
        // vocabulary. `as_str` matches exhaustively in production, so a fifth variant cannot be
        // added without naming it there; this pins what those names ARE.
        assert_eq!(
            [
                SummationFrame::Md.as_str(),
                SummationFrame::Tvd.as_str(),
                SummationFrame::Tvdss.as_str(),
                SummationFrame::Tst.as_str()
            ],
            ["MD", "TVD", "TVDSS", "TST"]
        );
        assert_eq!(SummationFrame::default(), SummationFrame::Md);

        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_partition_well(&conn, "FRAME-1");
        let dbm = Mutex::new(conn);
        let req = |frame: SummationFrame| PaySummaryRequest {
            discretisation: DiscretisationModel::Forward,
            input_set: None,
            well_ids: vec![well.clone()],
            vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
            swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
            perm_min: None,
            enabled_unset: Vec::new(),
            cutoff_use: Default::default(),
            skip_version: false,
            stats_only: true,
            custody: None,
            weighting: Default::default(),
            frame,
        };

        // A — every emitted row declares the frame AND where its weights came from. Both, because
        // "MD" alone does not say which depths were differenced.
        let rows = run_pay_summary(&dbm, &crate::reader_pool::ReaderPool::new(), &req(SummationFrame::Md)).expect("an MD summation runs");
        assert!(!rows.is_empty());
        for r in &rows {
            assert_eq!(r.frame, SummationFrame::Md, "{} frame", r.flag);
            assert_eq!(
                r.weights_source, MD_WEIGHTS_SOURCE,
                "{} must name the numbers its weights were differenced from",
                r.flag
            );
        }

        // B — the other three are REFUSED, by name, with the reason. Not returned empty, and above
        // all not returned as MD numbers wearing a different label.
        for frame in [SummationFrame::Tvd, SummationFrame::Tvdss, SummationFrame::Tst] {
            let err = run_pay_summary(&dbm, &crate::reader_pool::ReaderPool::new(), &req(frame))
                .expect_err("a frame whose weights cannot be computed must refuse");
            assert!(
                err.contains(frame.as_str()),
                "the refusal must name the frame that was asked for: {err}"
            );
            assert!(
                err.contains("cos") || err.contains("deviation") || err.contains("survey"),
                "the refusal must say what is missing, not merely that it declines: {err}"
            );
        }

        // C — and the refusal is a REFUSAL, not a fallback. If a TVD request ever starts returning
        // rows, this is the assertion that catches it before anybody quotes them.
        assert!(
            run_pay_summary(&dbm, &crate::reader_pool::ReaderPool::new(), &req(SummationFrame::Tvd)).is_err(),
            "a TVD result must never be an MD result relabelled"
        );
    }

    /// SB-CUT-016 (P0, SILENT-WRONGNESS). `14_cutoffs-summation-mc.md:1138-1160` — SandiBumi
    /// **MUST NOT** ship a numeric default for any cut-off; every cut-off field **MUST** ship in
    /// the first-class state *no default — user must set*; an unfiltered summation **MUST** be
    /// reported as unfiltered on the result and in the report; and a summation **MUST NOT** run
    /// against an unset cut-off that has been enabled.
    ///
    /// Four shipped vendor sets, no two identical, **two of them from one vendor**: IP φ 0.1 /
    /// Sw 0.5 / Vsh 0.5; Techlog 0.15 / 0.85 / 0.5; Geolog `default_*.paysum` 0.08 / 0.5 / 0.3;
    /// Geolog `determin_mc.info` 0.08 / 0.5 / **0.5**. Jauhar's own delivered work spans Vsh
    /// 0.20–0.85 and one record spans Vsh 0.55–0.85 *across intervals of a single area* — the
    /// quantity is not constant even within one field, so there is no number to pick.
    ///
    /// **What this row deliberately does NOT change:** the NaN cascade. A sample with no VSH is
    /// still unjudgeable whether or not VSH is being used as a cut-off. Making an unfiltered
    /// cut-off also stop requiring its curve would let a well with no VSH book pay it never
    /// booked, and the requirement says nothing about it — so the rule stands untouched.
    #[test]
    fn no_cutoff_ships_a_value_an_unapplied_one_is_reported_unfiltered_and_an_enabled_blank_one_refuses(
    ) {
        // A — the UI ships no numeric cut-off default. This is where the violation lived: the
        // backend always required values, while two frontend surfaces pre-filled them.
        for (path, src) in [
            ("src/ui/cutoffs.ts", include_str!("../../src/ui/cutoffs.ts")),
            ("src/ui/dashboardPanel.ts", include_str!("../../src/ui/dashboardPanel.ts")),
        ] {
            for banned in ["0.5", "0.15", "0.85", "0.08", "0.3", "0.1", "0.6"] {
                let seeded = format!("\"{banned}\"");
                assert!(
                    !src.contains(&seeded),
                    "{path} seeds a cut-off field with {seeded} - no vendor's number is \
                     defensible here, and a pre-filled box is a shipped default"
                );
            }
        }

        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        // Every sample passes on VSH and PHIE; SWE 0.20 and 0.60 straddle a 0.4 cut-off.
        let well = seed_weighting_well(&conn, "CUTOFF-1", "PHIE");
        let dbm = Mutex::new(conn);
        let vv = |v: Option<f64>| v.map(|x| CutoffSpec::from(CutoffEntry { value: x, unit: "v/v".into() }));
        let req = |vsh: Option<f64>, phie: Option<f64>, swe: Option<f64>, blank: Vec<String>| {
            PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well.clone()],
                vsh_max: vv(vsh),
                phie_min: vv(phie),
                swe_max: vv(swe),
                perm_min: None,
                skip_version: false,
                stats_only: true,
                custody: None,
                weighting: Default::default(),
                frame: Default::default(),
                enabled_unset: blank,
                cutoff_use: Default::default(),
            }
        };
        let pay = |rows: &[PaySummaryRow]| -> PaySummaryRow {
            rows.iter().find(|r| r.flag == "PAY").expect("a PAY row").clone()
        };

        // B — an SWE cut-off of 0.4 excludes the five deep samples, so PAY net is 5 of 10.
        let filtered = run_pay_summary(&dbm, &crate::reader_pool::ReaderPool::new(), &req(Some(0.9), Some(0.05), Some(0.4), vec![]))
            .expect("a fully specified summation runs");
        assert!((pay(&filtered).net - 5.0).abs() < 1e-6, "the SWE cut-off must bite");
        assert_eq!(
            pay(&filtered).unfiltered,
            vec!["PERM".to_string()],
            "only PERM is unfiltered here - not asking for a permeability cut-off is itself an unfiltered summation on that property, and the result says so rather than staying silent about it"
        );

        // C — omitting it makes the summation UNFILTERED on SWE: all ten units count, AND the row
        // says so. Both halves matter - a number that quietly stopped being filtered, with nothing
        // on the result to say so, is the whole failure this clause exists to prevent.
        let unfiltered = run_pay_summary(&dbm, &crate::reader_pool::ReaderPool::new(), &req(Some(0.9), Some(0.05), None, vec![]))
            .expect("an unfiltered summation is legitimate and runs");
        assert!(
            (pay(&unfiltered).net - 10.0).abs() < 1e-6,
            "an absent cut-off must not filter, got net {}",
            pay(&unfiltered).net
        );
        assert_eq!(
            pay(&unfiltered).unfiltered,
            vec!["SWE".to_string(), "PERM".to_string()],
            "the result must REPORT every cut-off that was not applied, in VSH/PHIE/SWE/PERM order"
        );

        // D — ABSENT MEANS ABSENT, not a fallback. Rock that fails EVERY vendor default - Vsh 0.80
        // against their 0.5, φ 0.02 against 0.08/0.1/0.15, Sw 0.95 against 0.5/0.6/0.85 - must
        // count in full when no cut-off is set. Arm C alone could not catch a silent fallback,
        // because its φ 0.30 and Vsh 0.40 clear those numbers anyway; this is the arm that bites.
        let shale_id = uuid::Uuid::new_v4();
        {
            let conn = dbm.lock().unwrap();
            db::insert_well(&conn, shale_id, "CUTOFF-SHALE", Some("Synthetic"), None, None).unwrap();
            let sid = shale_id.to_string();
            let n = 11usize;
            let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
            let nan = vec![f32::NAN; n];
            db::insert_standard_curves(
                &conn, shale_id, depth.clone(), vec![120.0; n], vec![40.0; n], vec![0.02; n],
                vec![2.6; n], nan.clone(), nan,
            )
            .unwrap();
            for (curve, v) in [("VSH", 0.80f32), ("PHIE", 0.02), ("SWE", 0.95)] {
                equations::write_computed_curve(&conn, &sid, &depth, curve, &vec![v; n]).unwrap();
            }
        }
        let shale = shale_id.to_string();
        let mut all_absent = req(None, None, None, vec![]);
        all_absent.well_ids = vec![shale.clone()];
        let rows = run_pay_summary(&dbm, &crate::reader_pool::ReaderPool::new(), &all_absent).expect("an entirely unfiltered run is legitimate");
        let r = rows.iter().find(|r| r.flag == "PAY").expect("a PAY row");
        assert!(
            (r.net - 10.0).abs() < 1e-6,
            "with no cut-off set, rock that fails every vendor default still counts in full - a \
             net below 10 here means an absent cut-off quietly became somebody's number, got {}",
            r.net
        );
        assert_eq!(
            r.unfiltered,
            vec!["VSH".to_string(), "PHIE".to_string(), "SWE".to_string(), "PERM".to_string()],
            "and all four are reported unfiltered"
        );

        // E — a cut-off the user switched on and left blank REFUSES. Distinct from C on purpose:
        // "I am not filtering on Sw" and "I meant to filter on Sw and have not said what" are
        // different statements, and only one of them may produce a number.
        let err = run_pay_summary(&dbm, &crate::reader_pool::ReaderPool::new(), &req(Some(0.9), Some(0.05), None, vec!["SWE".into()]))
            .expect_err("an enabled but unset cut-off must refuse");
        assert!(
            err.contains("SWE"),
            "the refusal must name the cut-off that was left blank: {err}"
        );
    }

    /// SB-CUT-019 (P1). `14_cutoffs-summation-mc.md:1204-1221` and `:2087` (SB-CUT-T26) — a
    /// cut-off **MUST** be entered with a unit and stored with it; a bare number **MUST** be
    /// rejected; `35 pu` **MUST** be accepted and stored as `0.35 v/v`; `35 v/v` **MUST** be
    /// rejected as out of bounds; dimensionless cut-offs **MUST** be bounded to their quantity's
    /// physical range.
    ///
    /// IP's own manual expresses the sensitivity-sweep example in porosity units and the cut-off
    /// default in `v/v` **for the same quantity, with no unit tag on the field**. `35` where `0.1`
    /// is meant is a **350x** error, and its symptom is an all-net result — a good-looking well,
    /// not a visible failure. The unit is the only thing that separates the two readings, so it is
    /// required rather than guessed.
    #[test]
    fn a_cutoff_is_refused_without_a_unit_and_thirty_five_is_porosity_units_or_out_of_bounds() {
        let por = CutoffQuantity::VolumeFraction;

        // A — a bare number is REFUSED for the MISSING UNIT, not for the number being implausible.
        // `0.10` is a perfectly ordinary porosity cut-off in v/v, so an implementation that only
        // range-checked would let it through — and would then be silently choosing between
        // 0.10 v/v and 0.10 pu, which differ by the same 100x the rule exists to stop.
        let plausible = CutoffEntry { value: 0.10, unit: String::new() };
        let err = plausible.canonical(por, "the PHIE cut-off").expect_err("a bare number refuses");
        assert!(err.contains("PHIE"), "the refusal names the field: {err}");
        assert!(err.contains("no unit"), "and refuses for the RIGHT reason: {err}");

        // and the chapter's own example carries a message that explains the trap rather than
        // saying "no" — a refusal an analyst cannot act on gets worked around, not obeyed.
        let bare = CutoffEntry { value: 35.0, unit: String::new() };
        let err = bare.canonical(por, "the PHIE cut-off").expect_err("a bare number must refuse");
        assert!(err.contains("350"), "and states the size of the error it prevents: {err}");

        // B — `35 pu` is accepted and canonicalised to 0.35 v/v.
        let pu = CutoffEntry { value: 35.0, unit: "pu".into() };
        assert!(
            (pu.canonical(por, "the PHIE cut-off").expect("35 pu is a real porosity") - 0.35).abs()
                < 1e-12,
            "35 pu is 0.35 v/v"
        );

        // C — `35 v/v` is REFUSED as out of bounds. Same number as B, opposite verdict, and only
        // the unit distinguishes them: that is the whole requirement in one pair of assertions.
        let vv = CutoffEntry { value: 35.0, unit: "v/v".into() };
        let err = vv.canonical(por, "the PHIE cut-off").expect_err("35 v/v is impossible");
        assert!(err.contains("physical range"), "{err}");

        // D — the bounds are the quantity's own, both ends.
        assert!(CutoffEntry { value: -0.1, unit: "v/v".into() }.canonical(por, "x").is_err());
        assert!(CutoffEntry { value: 1.0, unit: "v/v".into() }.canonical(por, "x").is_ok());
        assert!(CutoffEntry { value: 100.0, unit: "%".into() }.canonical(por, "x").is_ok());

        // E — permeability has its own unit family and its own bound, so the rule is a property of
        // the QUANTITY rather than a single hard-coded 0..1.
        let perm = CutoffQuantity::Permeability;
        assert!(
            (CutoffEntry { value: 1.0, unit: "D".into() }.canonical(perm, "k").unwrap() - 1000.0)
                .abs()
                < 1e-9,
            "1 darcy is 1000 mD"
        );
        assert!(CutoffEntry { value: -1.0, unit: "mD".into() }.canonical(perm, "k").is_err());
        assert!(
            CutoffEntry { value: 1.0, unit: "v/v".into() }.canonical(perm, "k").is_err(),
            "a volume fraction is not a permeability, however plausible the number"
        );

        // F — WIRED IN: the summation refuses before it computes anything, so a bare number can
        // never reach the pay arithmetic. A refusal that only exists in a helper is not a contract.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_weighting_well(&conn, "UNIT-1", "PHIE");
        let dbm = Mutex::new(conn);
        let err = run_pay_summary(
            &dbm,
            &crate::reader_pool::ReaderPool::new(),
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well],
                vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 35.0, unit: String::new() }.into()),
                swe_max: None,
                perm_min: None,
                skip_version: false,
                stats_only: true,
                custody: None,
                weighting: Default::default(),
                frame: Default::default(),
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
            },
        )
        .expect_err("a bare cut-off must stop the run");
        assert!(err.contains("no unit"), "{err}");
    }

    #[test]
    fn accumulate_never_clamps_while_flag_test_and_present_clamp_to_the_quantitys_own_bounds() {
        use BoundedQuantity::{Permeability, Unbounded, VolumeFraction};
        use ClampStage::{Accumulate, FlagTest, Present};

        // A — the stage rule, at values outside the bounds on both sides. `accumulate` returns the
        // value untouched for EVERY quantity; the other two clamp.
        for quantity in [VolumeFraction, Permeability, Unbounded] {
            for value in [-0.4f32, 1.7, 42.0] {
                assert_eq!(
                    stage_value(Accumulate, quantity, value),
                    value,
                    "accumulate must never clamp: {quantity:?} at {value}"
                );
            }
        }
        assert_eq!(stage_value(FlagTest, VolumeFraction, 1.7), 1.0);
        assert_eq!(stage_value(Present, VolumeFraction, -0.4), 0.0);

        // B — the quantified consequence, reproduced. A symmetric spread of `1 - Sw` about zero
        // accumulates to zero unclamped and to a POSITIVE mean clamped: the bias is not a tail
        // effect, it is the mean moving, and it moves toward more hydrocarbon every time.
        let draws: Vec<f32> = (-50..=50).map(|i| i as f32 * 0.01).collect();
        let unclamped: f64 = draws
            .iter()
            .map(|hc| stage_value(Accumulate, VolumeFraction, *hc) as f64)
            .sum();
        let clamped: f64 = draws
            .iter()
            .map(|hc| stage_value(Present, VolumeFraction, *hc) as f64)
            .sum();
        assert!(unclamped.abs() < 1e-6, "a symmetric spread accumulates to zero: {unclamped}");
        assert!(
            clamped > 0.0,
            "and clamping the same spread moves the mean toward hydrocarbon: {clamped}"
        );

        // C — BOUNDS ATTACH TO THE QUANTITY. Permeability is bounded BELOW and open above, so a
        // real 4,000 mD must survive; an unbounded quantity is not clamped to [0,1] merely because
        // that is the common case. Binding bounds to a curve-type string is the specific failure
        // that makes IP's clipping invisible in the data — a quantity cannot be mis-typed by a
        // label, because it is not a label.
        assert_eq!(stage_value(Present, Permeability, 4000.0), 4000.0);
        assert_eq!(stage_value(Present, Permeability, -3.0), 0.0);
        assert_eq!(
            stage_value(Present, Unbounded, 42.0),
            42.0,
            "an unbounded quantity must not be clamped to [0,1]"
        );
        assert_eq!(Unbounded.bounds(), None);
        assert_eq!(Permeability.bounds(), Some((0.0, f64::INFINITY)));

        // D — an out-of-range value is DETECTED, and a NaN is not out of range: absent is a
        // different statement and already has its own carrier (SB-CUT-029).
        assert!(VolumeFraction.is_out_of_range(1.2) && VolumeFraction.is_out_of_range(-0.1));
        assert!(!VolumeFraction.is_out_of_range(0.5));
        assert!(!VolumeFraction.is_out_of_range(f32::NAN), "missing is not out of range");
        assert!(!Unbounded.is_out_of_range(1e9));

        // AUDIT-2026-08-20 finding 61. An INFINITY is outside a finite bound and must say so.
        // This used to open with a blanket `is_finite` guard and answer NO, so an infinite average
        // was reported UNFLAGGED - and it then fell through to a PRESENT-stage wrapper that
        // clamped it to a clean-looking 1.0. Both halves are gone: the flag catches it, and
        // nothing stands between the average and the row. The third assertion is the one that
        // says why it mattered - the stage it used to route through still reports a perfect
        // volume fraction for an arithmetic that broke.
        assert!(VolumeFraction.is_out_of_range(f32::INFINITY), "an infinite average is not in 0..1");
        assert!(VolumeFraction.is_out_of_range(f32::NEG_INFINITY));
        assert_eq!(stage_value(Present, VolumeFraction, f32::INFINITY), 1.0);

        // E — PERCENT-TO-FRACTION CONVERSION AND THE BOUND CHECK ARE SEPARATE OPERATIONS, and an
        // over-bound value AFTER conversion raises. `35 pu` converts to 0.35 and passes; `35 v/v`
        // needs no conversion and fails the check; `200 pu` converts to 2.0 and fails AFTER the
        // conversion, which is the ordering the requirement asks for.
        let por = CutoffQuantity::VolumeFraction;
        assert!((CutoffEntry { value: 35.0, unit: "pu".into() }.canonical(por, "x").unwrap() - 0.35).abs() < 1e-12);
        assert!(CutoffEntry { value: 35.0, unit: "v/v".into() }.canonical(por, "x").is_err());
        let after = CutoffEntry { value: 200.0, unit: "pu".into() }
            .canonical(por, "the PHIE cut-off")
            .expect_err("2.0 v/v is impossible");
        assert!(
            after.contains("physical range") && after.contains('2'),
            "the refusal must name the CONVERTED value, which is what proves the check ran after \
             the conversion rather than instead of it: {after}"
        );

        // F — WIRED IN: a zonal average outside its bounds is EMITTED with the flag rather than
        // corrected. An ordinary well flags nothing, which is the control that stops the flag from
        // being stuck on.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_wet_reservoir(&conn, "SANDI-CLAMP-1");
        let dbm = Mutex::new(conn);
        let rows = run_pay_summary(
            &dbm,
            &crate::reader_pool::ReaderPool::new(),
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well],
                vsh_max: None,
                phie_min: None,
                swe_max: None,
                perm_min: None,
                skip_version: false,
                stats_only: true,
                custody: None,
                weighting: Default::default(),
                frame: Default::default(),
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
            },
        )
        .expect("the run is valid");
        assert!(!rows.is_empty());
        assert!(
            rows.iter().all(|r| !r.out_of_range),
            "an in-range well must not be flagged - a flag that is always on says nothing"
        );
        let wire = serde_json::to_value(&rows[0]).unwrap();
        assert!(
            wire["out_of_range"].is_boolean(),
            "and the condition rides a typed sibling, not the numeric column"
        );

        // G — the POSITIVE half, and it is the half the requirement actually turns on: a zonal
        // average outside its bounds is EMITTED WITH THE FLAG AND NOT CORRECTED. Without this the
        // in-range control above is satisfied by a flag hard-wired to `false`, which is a check
        // that cannot fail — a mutation proved exactly that before this arm existed.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-CLAMP-2", Some("Synthetic"), None, None).unwrap();
        let impossible = id.to_string();
        let n = 8usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth.clone(), vec![30.0; n], vec![4.0; n], nan.clone(), nan.clone(),
            nan.clone(), nan,
        )
        .unwrap();
        // A supersaturated combination: a saturation above 1 is physically impossible and is
        // exactly what an unclamped chain output looks like when a sampled parameter set is wrong.
        for (curve, value) in [("VSH", 0.10f32), ("PHIE", 0.30), ("SWE", 1.40)] {
            equations::write_computed_curve(&conn, &impossible, &depth, curve, &vec![value; n])
                .unwrap();
        }
        let dbm = Mutex::new(conn);
        let flagged = run_pay_summary(
            &dbm,
            &crate::reader_pool::ReaderPool::new(),
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![impossible],
                vsh_max: None,
                phie_min: None,
                swe_max: None,
                perm_min: None,
                skip_version: false,
                stats_only: true,
                custody: None,
                weighting: Default::default(),
                frame: Default::default(),
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
            },
        )
        .expect("an impossible saturation must still produce a result, flagged");
        let row = flagged
            .iter()
            .find(|r| r.avg_swe.is_finite())
            .expect("some row carries the saturation average");
        assert!(
            row.out_of_range,
            "an average of {} is outside 0..1 and must be FLAGGED",
            row.avg_swe
        );
        assert!(
            (row.avg_swe - 1.40).abs() < 1e-5,
            "and emitted AS COMPUTED, not corrected to 1.0 - a corrected average is a number \
             nobody derived: {}",
            row.avg_swe
        );
    }

    /// SB-CUT-029 (P1). `14_cutoffs-summation-mc.md:1361-1376` — a null or not-computed condition
    /// **MUST** be carried in a typed sibling field, **never as an in-band marker inside a numeric
    /// field**, and consumers **MUST** render a dash rather than a zero when the count is zero.
    ///
    /// F-15: IP prints `$$` **inside a numeric report column** to mean *"nulls present"* —
    /// unparseable, uncarryable through a calculation, and invisible to a downstream consumer that
    /// reads the column as a number. The chapter's as-built says the marker discipline is already
    /// right and that *"the remaining work is the footage partition of `SB-CUT-003`"*; that row
    /// landed, so what is left is this proof.
    #[test]
    fn a_not_computed_condition_rides_a_typed_sibling_and_never_a_marker_inside_a_numeric_column() {
        let row = |name: &str, n_classified: usize, perm_no_data: bool| PaySummaryRow {
            well_id: "w".into(),
            well_name: name.into(),
            discretisation_model: DISCRETISATION_MODEL.to_string(),
            sample_interval: 0.5,
            zone: "WHOLE".into(),
            flag: "PAY".into(),
            top: 1000.0,
            bottom: 1010.0,
            gross: 10.0,
            net: 0.0,
            not_net: if n_classified == 0 { 0.0 } else { 10.0 },
            unknown: if n_classified == 0 { 10.0 } else { 0.0 },
            ntg: 0.0,
            ntg_known: if n_classified == 0 { f32::NAN } else { 0.0 },
            avg_vsh: f32::NAN,
            avg_phie: f32::NAN,
            avg_swe: f32::NAN,
            hpv: 0.0,
            n_classified,
            perm_cutoff_no_data: perm_no_data,
            quicklook_phie_excluded: false,
            residual_absorbed: 0.0,
            out_of_range: false,
            unfiltered: vec!["PERM".into()],
            frame: Default::default(),
            weights_source: MD_WEIGHTS_SOURCE.into(),
        };
        let uninterpreted = serde_json::to_value(row("NEVER_INTERPRETED", 0, false)).unwrap();
        let barren = serde_json::to_value(row("INTERPRETED_AND_BARREN", 40, true)).unwrap();

        // A — NO IN-BAND MARKER. Every field that carries a quantity must serialize as a JSON
        // number or null. A string in a numeric column is F-15 exactly: it survives the wire, it
        // reads as data, and it stops being arithmetic.
        for (label, value) in [("uninterpreted", &uninterpreted), ("barren", &barren)] {
            let object = value.as_object().expect("a row is an object");
            for field in [
                "top", "bottom", "gross", "net", "not_net", "unknown", "ntg", "ntg_known",
                "avg_vsh", "avg_phie", "avg_swe", "hpv", "residual_absorbed",
            ] {
                let cell = &object[field];
                assert!(
                    cell.is_number() || cell.is_null(),
                    "{label} row: '{field}' carries {cell}, which is not a number - a marker \
                     inside a numeric column is unparseable and uncarryable through a calculation"
                );
            }
        }

        // B — THE TYPED SIBLINGS, and their types. A count that arrived as a string, or a flag as
        // "true", would satisfy arm A and defeat the requirement.
        for (label, value) in [("uninterpreted", &uninterpreted), ("barren", &barren)] {
            let object = value.as_object().expect("an object");
            assert!(object["n_classified"].is_u64(), "{label}: the count is an integer");
            assert!(
                object["perm_cutoff_no_data"].is_boolean(),
                "{label}: the missing-permeability condition is a boolean"
            );
            assert!(
                object["unfiltered"]
                    .as_array()
                    .is_some_and(|names| names.iter().all(|name| name.is_string())),
                "{label}: the unfiltered cut-offs are a list of names, not a packed string"
            );
        }

        // C — the two rows are distinguishable PURELY from the typed fields. Their numeric columns
        // are the same shape - net 0, N:G 0, HCPV 0 - which is the whole trap: a reader looking at
        // the numbers alone cannot tell a well nobody interpreted from a well found barren, and
        // nothing in the numbers is allowed to tell them apart either.
        for field in ["net", "ntg", "hpv"] {
            assert_eq!(
                uninterpreted[field], barren[field],
                "the numeric columns must NOT encode the difference - '{field}' does"
            );
        }
        assert_ne!(
            uninterpreted["n_classified"], barren["n_classified"],
            "and the typed sibling must be what carries it"
        );

        // D — WIRED IN. A real run emits the siblings, so the discipline is a property of the
        // engine's output rather than of a struct somebody could bypass.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_wet_reservoir(&conn, "SANDI-MARKER-1");
        let dbm = Mutex::new(conn);
        let rows = run_pay_summary(
            &dbm,
            &crate::reader_pool::ReaderPool::new(),
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well],
                vsh_max: None,
                phie_min: None,
                swe_max: None,
                perm_min: Some(CutoffEntry { value: 1.0, unit: "mD".into() }.into()),
                skip_version: false,
                stats_only: true,
                custody: None,
                weighting: Default::default(),
                frame: Default::default(),
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
            },
        )
        .expect("the run is valid");
        let pay = rows.iter().find(|r| r.flag == "PAY").expect("a pay row is emitted");
        // This well carries no permeability at all, so every sample fails the cut-off for want of
        // data. The zero is real arithmetic; the REASON rides beside it in its own typed field.
        assert!(
            pay.perm_cutoff_no_data,
            "the well has no PERM, and the run must SAY so rather than leaving a bare zero"
        );
        assert_eq!(pay.net, 0.0, "while the number itself stays a number");
        let wire = serde_json::to_value(pay).unwrap();
        assert!(wire["net"].is_number() && wire["perm_cutoff_no_data"].is_boolean());
    }

    /// SB-CUT-028 (P1). `14_cutoffs-summation-mc.md:1346-1359` — saturation quantities **MUST** be
    /// named `SWE` or `SWT` explicitly wherever a cut-off, an average or a result field refers to
    /// one, and a bare `SW` **MUST NOT** appear in a cut-off record or a result field.
    ///
    /// FINDINGS §6 rule 8, sharpened by T3: in Techlog the mnemonic silently changes the
    /// weighting — *"the SW curve is weighted by POR but the SWE is not"* — so a bare name is both
    /// ambiguous about which saturation is meant AND load-bearing on the arithmetic. That is why
    /// this is P1: the ambiguity does not stay an ambiguity, it becomes a different number.
    ///
    /// The chapter's `Verified by` points at SB-CUT-T06, which its own test-to-requirement map
    /// assigns to SB-CUT-009 and which pins the average-form identity — the CONSEQUENCE of the
    /// naming rather than the naming. The naming contract therefore needs this test.
    #[test]
    fn no_module_output_cutoff_field_or_result_field_is_a_bare_sw_rather_than_swe_or_swt() {
        // A — the registry, from BOTH sides. A scan that only forbids would pass by finding
        // nothing at all, which is how a negative test quietly stops testing.
        let catalog = modules::list_modules();
        let outputs: Vec<(String, String)> = catalog
            .iter()
            .flat_map(|module| {
                module
                    .args
                    .iter()
                    .filter(|arg| arg.kind == ArgKind::LogOut)
                    .map(move |arg| (module.name.clone(), arg.name.clone()))
            })
            .collect();
        assert!(
            outputs.len() > 50,
            "the scan must see a real catalog, not an empty one: {} outputs",
            outputs.len()
        );
        for (module, output) in &outputs {
            assert_ne!(
                output.to_ascii_uppercase(),
                "SW",
                "module '{module}' emits a bare SW; a saturation output must say SWE or SWT"
            );
        }
        // and the positive control: the explicit identities really are what gets emitted.
        for wanted in ["SWE", "SWT"] {
            assert!(
                outputs.iter().any(|(_, output)| output == wanted),
                "some shipping module must emit '{wanted}'"
            );
        }

        // B — a RESULT FIELD. The pay-summary row is what a consumer reads, and its saturation
        // average must name its flavour there too, because the row outlives the run that made it.
        let row = PaySummaryRow {
            well_id: "w".into(),
            well_name: "SANDI-SW-1".into(),
            discretisation_model: DISCRETISATION_MODEL.to_string(),
            sample_interval: 0.5,
            zone: "A".into(),
            flag: "PAY".into(),
            top: 0.0,
            bottom: 1.0,
            gross: 1.0,
            net: 1.0,
            not_net: 0.0,
            unknown: 0.0,
            ntg: 1.0,
            ntg_known: 1.0,
            avg_vsh: 0.1,
            avg_phie: 0.2,
            avg_swe: 0.3,
            hpv: 0.14,
            n_classified: 1,
            perm_cutoff_no_data: false,
            quicklook_phie_excluded: false,
            residual_absorbed: 0.0,
            out_of_range: false,
            unfiltered: Vec::new(),
            frame: Default::default(),
            weights_source: MD_WEIGHTS_SOURCE.into(),
        };
        let serialized = serde_json::to_value(&row).expect("a row serializes");
        let fields: Vec<String> = serialized
            .as_object()
            .expect("a row is an object")
            .keys()
            .cloned()
            .collect();
        assert!(fields.iter().any(|f| f == "avg_swe"), "the row names the flavour: {fields:?}");
        for field in &fields {
            assert!(
                field != "avg_sw" && field != "sw" && field != "sw_max",
                "result field '{field}' is a bare SW"
            );
        }

        // C — a CUT-OFF RECORD. What is persisted with the run has to name the flavour, because
        // that record is read years later by somebody who cannot ask which Sw was meant.
        let request = PaySummaryRequest {
            discretisation: DiscretisationModel::Forward,
            input_set: None,
            well_ids: vec!["w".into()],
            vsh_max: None,
            phie_min: None,
            swe_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            perm_min: None,
            skip_version: false,
            stats_only: true,
            custody: None,
            weighting: Default::default(),
            frame: Default::default(),
            enabled_unset: Vec::new(),
            cutoff_use: Default::default(),
        };
        // `PaySummaryRequest` is a wire type read from the frontend, so the record to inspect is
        // the FIELD NAME the wire uses. Round-tripping the JSON the frontend sends is what proves
        // the persisted cut-off names its flavour; a bare `sw_max` would not deserialize at all.
        let wire = serde_json::json!({
            "well_ids": ["w"],
            "swe_max": {"value": 0.5, "unit": "v/v"},
        });
        let parsed: PaySummaryRequest =
            serde_json::from_value(wire).expect("the cut-off record uses swe_max");
        assert!(parsed.swe_max.is_some(), "and it carries the value");
        let bare = serde_json::json!({ "well_ids": ["w"], "sw_max": {"value": 0.5, "unit": "v/v"} });
        let parsed_bare: PaySummaryRequest =
            serde_json::from_value(bare).expect("an unknown field is ignored, not accepted as SWE");
        assert!(
            parsed_bare.swe_max.is_none(),
            "a bare sw_max must NOT be read as the saturation cut-off - silently accepting it \r
             is exactly the ambiguity this row forbids"
        );
        let _ = &request;

        // D — the exemption is NAMED and narrow. A bare `SW` may appear as an INPUT, because an
        // input names the user's own curve and the requirement governs cut-off records and result
        // fields. What it may never be is an OUTPUT, which arm A already forbids — so this arm
        // states the boundary rather than leaving it to be rediscovered as a false positive.
        let bare_sw_inputs: Vec<String> = catalog
            .iter()
            .flat_map(|module| {
                module
                    .args
                    .iter()
                    .filter(|arg| arg.kind == ArgKind::LogIn && arg.name.eq_ignore_ascii_case("SW"))
                    .map(move |_| module.name.clone())
            })
            .collect();
        for module in &bare_sw_inputs {
            let emits_bare = outputs
                .iter()
                .any(|(m, o)| m == module && o.eq_ignore_ascii_case("SW"));
            assert!(
                !emits_bare,
                "module '{module}' may READ a curve called SW, but it must not emit one"
            );
        }
    }

    /// SB-CUT-027 (P2). `14_cutoffs-summation-mc.md:1331-1344` — SandiBumi **MUST NOT** impose a
    /// fixed maximum on the number of input curves, cut-offs, report tiers or flag curves.
    ///
    /// Ledger D-5.4: IP's parameter model stops at **Curve 10**, its 2025 prose claims **50**, and
    /// IP2018's *"up to 10 input curves … the additional 7"* was correct — the 2025 edit introduced
    /// the error. All of them are vendor implementation limits with no physical basis, and
    /// SandiBumi should inherit neither the caps nor the confusion.
    ///
    /// **A fixed ARITY is not a cap, and the distinction is the whole row.** Four cut-off fields
    /// exist because four quantities are cut on; three tiers exist because three are emitted.
    /// Neither is a budget a user can exhaust. A cap is a maximum imposed on a collection that
    /// would otherwise grow — which is what this asserts the absence of.
    #[test]
    fn a_run_carries_more_curves_than_any_vendor_cap_and_the_fixed_cutoff_and_tier_counts_are_arities_not_maxima(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_wet_reservoir(&conn, "SANDI-NOCAP-1");
        let depth: Vec<f32> = (0..11).map(|i| 1000.0 + i as f32).collect();

        // A — sixty curves on one well, fetched in ONE frame. Sixty is chosen to clear BOTH of the
        // vendor numbers the ledger records: past Curve 10, and past the 2025 prose's 50.
        const CURVES: usize = 60;
        let names: Vec<String> = (0..CURVES).map(|i| format!("NOCAP_{i:02}")).collect();
        for (i, name) in names.iter().enumerate() {
            equations::write_computed_curve(&conn, &well, &depth, name, &vec![i as f32; 11])
                .unwrap();
        }
        let (frame_depth, columns) = equations::fetch_curve_frame(&conn, &well, &names)
            .expect("a frame of sixty curves must resolve");
        assert_eq!(frame_depth.len(), depth.len());
        assert_eq!(
            columns.len(),
            CURVES,
            "every requested curve must come back - a silent truncation IS a cap"
        );
        for (i, name) in names.iter().enumerate() {
            assert_eq!(
                columns[name][0], i as f32,
                "curve {name} must carry its own values, not a neighbour's"
            );
        }

        // B — the four cut-off fields are an ARITY. Each is independently absent-capable, so the
        // four are not a budget: a run may use none of them, all of them, or any subset, and the
        // count is not a resource anything competes for.
        let dbm = Mutex::new(conn);
        let vv = |value: f64| Some(CutoffSpec::from(CutoffEntry { value, unit: "v/v".into() }));
        let summary = |vsh, phie, swe| {
            run_pay_summary(
                &dbm,
                &crate::reader_pool::ReaderPool::new(),
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: vsh,
                    phie_min: phie,
                    swe_max: swe,
                    perm_min: None,
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    weighting: Default::default(),
                    frame: Default::default(),
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                },
            )
            .expect("every subset of the cut-offs is a valid run")
        };
        let none = summary(None, None, None);
        let all = summary(vv(0.50), vv(0.10), vv(0.90));
        assert!(!none.is_empty() && !all.is_empty());

        // C — the tier count is DATA, not a hard-coded three scattered through the engine. The row
        // emission iterates `SUMMARY_FLAGS`, so the output carries exactly the tiers that constant
        // names — which is what makes adding one a change to a list rather than a search for every
        // place a `3` is written down.
        let mut emitted: Vec<String> =
            all.iter().map(|row| row.flag.clone()).collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
        emitted.sort();
        let mut declared: Vec<String> = SUMMARY_FLAGS.iter().map(|f| f.to_string()).collect();
        declared.sort();
        assert_eq!(
            emitted, declared,
            "the emitted tiers must be exactly the declared ones, with nothing dropped and nothing \
             invented"
        );

        // D — and no cap is expressed anywhere in the summation engine as a maximum COUNT of
        // curves, cut-offs, tiers or flags. The clamps this domain does carry are on ITERATIONS
        // and SWEEP STEPS, which are compute budgets on a loop rather than limits on how much
        // rock a study may describe, so they are named here rather than exempted silently.
        let source = include_str!("paysummary.rs");
        let body = source.split("\nmod tests {").next().unwrap_or(source);
        for banned in [
            "curves.len() >",
            "cutoffs.len() >",
            "flags.len() >",
            "tiers.len() >",
            "curve_names.len() >",
            ".take(10)",
            ".take(50)",
        ] {
            assert!(
                !body.contains(banned),
                "the summation engine must impose no maximum on how much a study describes, but \
                 it contains '{banned}'"
            );
        }
    }

    /// SB-CUT-026 (P1). `14_cutoffs-summation-mc.md:1318-1329` — the net reservoir tier **MUST
    /// NOT** apply a saturation cut-off by default; net reservoir **MUST** be porosity- and
    /// clay-driven, and saturation **MUST** enter at the pay tier.
    ///
    /// F-25: IP's `Sw Net Use` and `Sw Pay Use` are separate ordinals and Net Reservoir is
    /// described as porosity- and clay-driven. The consequence of getting it wrong is stated in
    /// the chapter and is the reason this is P1 rather than a preference — **it reclassifies wet
    /// reservoir as non-reservoir**. A water-bearing sand is still reservoir rock; it is the pay
    /// tier that is allowed to care that it is wet.
    #[test]
    fn a_wet_but_porous_clean_sand_is_reservoir_and_not_pay_because_saturation_enters_at_the_pay_tier(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_wet_reservoir(&conn, "SANDI-WET-1");
        let dbm = Mutex::new(conn);
        let vv = |value: f64| Some(CutoffSpec::from(CutoffEntry { value, unit: "v/v".into() }));
        let run = |swe: Option<CutoffSpec>, use_at: Vec<(&str, CutoffUse)>| {
            let rows = run_pay_summary(
                &dbm,
                &crate::reader_pool::ReaderPool::new(),
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: vv(0.50),
                    phie_min: vv(0.10),
                    swe_max: swe,
                    perm_min: None,
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    weighting: Default::default(),
                    frame: Default::default(),
                    enabled_unset: Vec::new(),
                    cutoff_use: use_at
                        .into_iter()
                        .map(|(slot, u)| (slot.to_string(), u))
                        .collect(),
                },
            )
            .expect("the run is valid");
            let net = |flag: &str| {
                rows.iter()
                    .filter(|row| row.flag == flag)
                    .map(|row| row.net as f64)
                    .sum::<f64>()
            };
            (net("SAND"), net("RESERVOIR"), net("PAY"))
        };

        // A — the requirement's own failure mode. This sand is clean (VSH 0.10) and porous
        // (PHIE 0.30) and WET (SWE 0.80). With a 0.50 saturation cut-off it must book as
        // reservoir in full and as pay not at all.
        let (sand, reservoir, pay) = run(vv(0.50), vec![]);
        assert!(sand > 0.0, "the fixture books sand");
        assert_eq!(
            reservoir, sand,
            "a wet sand is still reservoir rock: applying the saturation cut-off at the reservoir \
             tier would reclassify it as non-reservoir, which is the defect this row prevents"
        );
        assert_eq!(pay, 0.0, "and saturation DOES enter at the pay tier");

        // B — the reservoir tier is independent of the saturation cut-off's VALUE. Moving it must
        // move pay and leave reservoir where it is; that is what "does not apply" means, as
        // distinct from "applies but happens not to bite on this fixture".
        let (_, reservoir_loose, pay_loose) = run(vv(0.90), vec![]);
        assert_eq!(reservoir_loose, reservoir, "reservoir must not move with the SWE cut-off");
        assert!(pay_loose > pay, "while pay must: {pay_loose} against {pay}");

        // C — and reservoir IS porosity- and clay-driven, pinned from the positive side too. A
        // tier that applied NOTHING would satisfy every assertion above and be a different bug.
        let strict_clay = run(vv(0.50), vec![]).1;
        let (_, reservoir_clay, _) = {
            let rows = run_pay_summary(
                &dbm,
                &crate::reader_pool::ReaderPool::new(),
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: vv(0.05),
                    phie_min: vv(0.10),
                    swe_max: vv(0.50),
                    perm_min: None,
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    weighting: Default::default(),
                    frame: Default::default(),
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                },
            )
            .expect("the run is valid");
            let net = |flag: &str| {
                rows.iter()
                    .filter(|row| row.flag == flag)
                    .map(|row| row.net as f64)
                    .sum::<f64>()
            };
            (net("SAND"), net("RESERVOIR"), net("PAY"))
        };
        assert!(
            reservoir_clay < strict_clay,
            "a clay cut-off below the sand's VSH must reduce reservoir: {reservoir_clay} against \
             {strict_clay}"
        );

        // D — it is a DEFAULT, not a prohibition. The requirement says the reservoir tier must not
        // apply saturation *by default*; a user who declares otherwise is entitled to it, and
        // removing the capability would be a different requirement.
        let (_, reservoir_declared, _) = run(
            vv(0.50),
            vec![("SWE", CutoffUse { sand: false, reservoir: true, pay: true })],
        );
        assert_eq!(
            reservoir_declared, 0.0,
            "an explicit declaration must reach the reservoir tier - the rule is a default, not a \
             prohibition"
        );

        // E — and the default itself is DECLARED rather than emergent, so it can be read off the
        // configuration instead of inferred from a result.
        assert!(
            !default_cutoff_use(Slot::Swe).reservoir && default_cutoff_use(Slot::Swe).pay,
            "SWE ships off at the reservoir tier and on at pay"
        );
    }

    /// SB-CUT-022 (P1). `14_cutoffs-summation-mc.md:1254-1272` and F-25 at `:489-501` — each
    /// cut-off **MUST** carry an explicit enable flag per report tier; activation **MUST NOT** be
    /// inferred from the presence of a curve or of a value; and the reservoir and pay tiers
    /// **MUST** share **one value** with **two independent use flags**.
    ///
    /// IP ships exactly that shape: `Phi Net Use`, `Phi Pay Use` and `Phi Cutoff`, the last
    /// described as *"Porosity cutoff value for Pay and Reservoir report"* — one value, two flags.
    /// The reason it must be a flag and not an inference is F-17: Geolog changed the activation
    /// trigger between two modules of ONE product, `Determin` firing on the presence of the curve
    /// and `determin_mc` on the presence of the value. An inferred rule cannot be audited from a
    /// result, because the result does not record what was inferred.
    #[test]
    fn each_cutoff_declares_the_tiers_it_is_used_at_and_reservoir_and_pay_share_one_value_with_independent_flags(
    ) {
        // A — the shipped defaults ARE the ladder, declared rather than nested. Net sand is clay
        // driven, net reservoir adds porosity, net pay adds saturation (T4 Bentley & Ringrose,
        // `:1296-1297`), and Sw is OFF at the reservoir tier — F-25 `:494-495`, which is also
        // SB-CUT-026's whole subject.
        assert_eq!(
            default_cutoff_use(Slot::Vsh),
            CutoffUse { sand: true, reservoir: true, pay: true }
        );
        assert_eq!(
            default_cutoff_use(Slot::Phie),
            CutoffUse { sand: false, reservoir: true, pay: true }
        );
        assert_eq!(
            default_cutoff_use(Slot::Swe),
            CutoffUse { sand: false, reservoir: false, pay: true },
            "IP describes Net Reservoir as porosity- and clay-driven; Sw is off there by default"
        );
        assert_eq!(
            default_cutoff_use(Slot::Perm),
            CutoffUse { sand: false, reservoir: false, pay: true }
        );

        // B — ONE VALUE, TWO FLAGS. The reservoir and pay tiers read the same `phie_min`; turning
        // it off for one tier must not change the other, and must not change the value.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_weighting_well(&conn, "SANDI-TIER-1", "PHIE");
        let dbm = Mutex::new(conn);
        let run = |use_at: Vec<(&str, CutoffUse)>| {
            let rows = run_pay_summary(
                &dbm,
                &crate::reader_pool::ReaderPool::new(),
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: None,
                    phie_min: Some(
                        CutoffEntry { value: 0.20, unit: "v/v".into() }.into(),
                    ),
                    swe_max: None,
                    perm_min: None,
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    weighting: Default::default(),
                    frame: Default::default(),
                    enabled_unset: Vec::new(),
                    cutoff_use: use_at
                        .into_iter()
                        .map(|(slot, use_at)| (slot.to_string(), use_at))
                        .collect(),
                },
            )
            .expect("the run itself is valid");
            let net = |flag: &str| {
                rows.iter()
                    .filter(|row| row.flag == flag)
                    .map(|row| row.net as f64)
                    .sum::<f64>()
            };
            (net("RESERVOIR"), net("PAY"))
        };

        // The fixture's PHIE is 0.30 over its shallow half and 0.10 over its deep half, so a 0.20
        // cut-off is a real filter: with it on, half the footage books.
        let (res_both, pay_both) = run(vec![]);
        assert!(res_both > 0.0 && pay_both > 0.0, "the default run books something");

        // Off at RESERVOIR only. Pay must not move — that is what INDEPENDENT means.
        let (res_off, pay_off) = run(vec![(
            "PHIE",
            CutoffUse { sand: false, reservoir: false, pay: true },
        )]);
        assert!(
            res_off > res_both,
            "with the porosity cut-off disabled at the reservoir tier that tier must book MORE \
             footage: {res_off} against {res_both}"
        );
        assert_eq!(
            pay_off, pay_both,
            "and the pay tier must not move — one value, two independent flags"
        );

        // Off at PAY only. Now the mirror: reservoir must not move.
        let (res_pay_off, pay_pay_off) = run(vec![(
            "PHIE",
            CutoffUse { sand: false, reservoir: true, pay: false },
        )]);
        assert_eq!(res_pay_off, res_both, "the reservoir tier must not move");
        assert!(
            pay_pay_off > pay_both,
            "and the pay tier must book more: {pay_pay_off} against {pay_both}"
        );

        // C — ACTIVATION IS NEVER INFERRED. `use_at` is resolved from the SLOT and the run's own
        // declaration and nothing else, so neither the presence of a curve nor the presence of a
        // value can turn a cut-off on. That is a property of the signature, not of today's body:
        // the resolver has no access to either.
        let declared = BTreeMap::from([(
            "PHIE".to_string(),
            CutoffUse { sand: true, reservoir: false, pay: false },
        )]);
        assert_eq!(
            cutoff_use_for(&declared, Slot::Phie),
            CutoffUse { sand: true, reservoir: false, pay: false },
            "a declaration is honoured verbatim"
        );
        assert_eq!(
            cutoff_use_for(&declared, Slot::Swe),
            default_cutoff_use(Slot::Swe),
            "and an undeclared slot takes its documented default, not its neighbour's declaration"
        );

        // D — a cut-off disabled at EVERY tier books exactly what no cut-off at all books. The two
        // are different statements about intent and must be the same statement about rock.
        let all_off = run(vec![(
            "PHIE",
            CutoffUse { sand: false, reservoir: false, pay: false },
        )]);
        let unfiltered = {
            let rows = run_pay_summary(
                &dbm,
                &crate::reader_pool::ReaderPool::new(),
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: None,
                    phie_min: None,
                    swe_max: None,
                    perm_min: None,
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    weighting: Default::default(),
                    frame: Default::default(),
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                },
            )
            .expect("an unfiltered run is valid");
            let net = |flag: &str| {
                rows.iter()
                    .filter(|row| row.flag == flag)
                    .map(|row| row.net as f64)
                    .sum::<f64>()
            };
            (net("RESERVOIR"), net("PAY"))
        };
        assert_eq!(
            all_off, unfiltered,
            "a cut-off switched off at every tier filters nothing, exactly as an absent one does"
        );
    }

    /// SB-CUT-020 (P2). `14_cutoffs-summation-mc.md:1223-1240` and SB-CUT-T24 at `:2085` — a
    /// cut-off **MUST** be expressible as a two-sided range with an explicit operator selecting the
    /// inclusivity of each bound; the single-sided `>=` / `<=` forms **MUST** be the degenerate
    /// case with an open far bound; and every operator's boundary behaviour **MUST** be tested
    /// against SandiBumi's **own written specification**, which is [`CutoffRange`]'s doc comment.
    ///
    /// The oracle is deliberately ours. Techlog's `limitType` is strictly more general than IP's
    /// single-sided form, but its shipped implementation is the warning rather than the model:
    /// modes 4/5/6 raise, mode 7 is a silent always-pass, and modes 2/3 are **documented as outside
    /// tests and implemented as inside tests**. A boundary convention not tested against its own
    /// spec is a coin flip at every sample sitting exactly on the cut-off — which is precisely the
    /// population that decides a marginal-pay result.
    #[test]
    fn a_sample_exactly_on_a_cutoff_bound_is_included_or_excluded_by_that_bounds_own_declared_operator(
    ) {
        // A — the specification itself, at exactly the bound, for every operator on every side.
        // This is the T24 case: a value equal to `min` and a value equal to `max`.
        let low = |operator| CutoffRange {
            low: Some(CutoffBound { value: 0.10, operator }),
            high: None,
        };
        let high = |operator| CutoffRange {
            low: None,
            high: Some(CutoffBound { value: 0.50, operator }),
        };
        assert!(low(BoundOperator::Inclusive).contains(0.10f32), "x >= min admits x == min");
        assert!(!low(BoundOperator::Exclusive).contains(0.10f32), "x > min excludes x == min");
        assert!(high(BoundOperator::Inclusive).contains(0.50f32), "x <= max admits x == max");
        assert!(!high(BoundOperator::Exclusive).contains(0.50f32), "x < max excludes x == max");
        // and away from the bound every operator agrees, so the arms above isolate the boundary.
        for operator in [BoundOperator::Inclusive, BoundOperator::Exclusive] {
            assert!(low(operator).contains(0.11f32) && !low(operator).contains(0.09f32));
            assert!(high(operator).contains(0.49f32) && !high(operator).contains(0.51f32));
        }

        // A2 — and "exactly on the bound" is decided at the precision the DATA has. A continuous
        // log is f32; a cut-off is entered as a decimal. Widen the sample and 0.30f32 becomes
        // 0.30000001192…, strictly GREATER than 0.30f64 — so the sample the user typed `0.30` to
        // sit exactly on would not sit on it, and the exclusive operator would exclude nothing at
        // all. Both sides are pinned, because an implementation comparing in f64 passes the
        // inclusive half and fails only here.
        let three_tenths = CutoffRange {
            low: Some(CutoffBound { value: 0.30, operator: BoundOperator::Exclusive }),
            high: None,
        };
        assert!(
            (0.30f32 as f64) > 0.30f64,
            "the premise: widening an f32 sample overshoots the f64 bound"
        );
        assert!(
            !three_tenths.contains(0.30f32),
            "an f32 sample of 0.30 sits exactly on a 0.30 bound and an exclusive bound excludes it"
        );
        assert!(
            CutoffRange {
                low: Some(CutoffBound { value: 0.30, operator: BoundOperator::Inclusive }),
                high: None,
            }
            .contains(0.30f32),
            "and an inclusive bound admits it"
        );

        // B — an ABSENT bound is an OPEN far bound and admits everything on that side. That is
        // what makes the single-sided form a degenerate range rather than a separate mechanism.
        let open = CutoffRange { low: None, high: None };
        assert!(open.contains(-1e9f32) && open.contains(1e9f32));
        assert!(low(BoundOperator::Inclusive).contains(1e9f32), "no high bound admits any large value");

        // C — the DEGENERATE wire form is unchanged. A slot that has always meant "at least this"
        // still means it, inclusively, and a slot that has always meant "at most this" likewise -
        // the requirement makes the single-sided forms the degenerate case, so a project saved
        // before ranges existed must classify every sample exactly as it did.
        let entry: CutoffSpec = serde_json::from_str(r#"{"value":0.10,"unit":"v/v"}"#).unwrap();
        let as_min = entry
            .canonical(CutoffQuantity::VolumeFraction, CutoffSense::Minimum, "PHIE")
            .unwrap();
        assert_eq!(as_min.low, Some(CutoffBound { value: 0.10, operator: BoundOperator::Inclusive }));
        assert_eq!(as_min.high, None, "the far side stays open");
        assert!(as_min.contains(0.10f32), "and a sample exactly on it still passes, as it always did");
        let as_max = entry
            .canonical(CutoffQuantity::VolumeFraction, CutoffSense::Maximum, "VSH")
            .unwrap();
        assert_eq!(as_max.high, Some(CutoffBound { value: 0.10, operator: BoundOperator::Inclusive }));
        assert_eq!(as_max.low, None);

        // D — a genuine two-sided range, with a different operator on each side, crosses the wire
        // and filters both ends. `35 pu` is canonicalised per bound, so the unit rule of SB-CUT-019
        // reaches inside a range rather than stopping at its edge.
        let spec: CutoffSpec = serde_json::from_str(
            r#"{"min":{"value":10,"unit":"pu","operator":"EXCLUSIVE"},
                "max":{"value":35,"unit":"pu","operator":"INCLUSIVE"}}"#,
        )
        .unwrap();
        let range = spec
            .canonical(CutoffQuantity::VolumeFraction, CutoffSense::Minimum, "PHIE")
            .expect("a two-sided porosity window is a real cut-off");
        assert!(!range.contains(0.10f32), "the low bound is exclusive, so 0.10 fails");
        assert!(range.contains(0.35f32), "the high bound is inclusive, so 0.35 passes");
        assert!(range.contains(0.20f32) && !range.contains(0.40f32));

        // E — a range that can admit NOTHING is refused. Booking zero net from a window nobody
        // could have meant is this row's own risk class: it computes, it plots, and it is wrong.
        let empty: CutoffSpec = serde_json::from_str(
            r#"{"min":{"value":0.40,"unit":"v/v"},"max":{"value":0.20,"unit":"v/v"}}"#,
        )
        .unwrap();
        let error = empty
            .canonical(CutoffQuantity::VolumeFraction, CutoffSense::Minimum, "the PHIE cut-off")
            .expect_err("an inverted window must refuse");
        assert!(error.contains("PHIE"), "and name the cut-off: {error}");
        let touching: CutoffSpec = serde_json::from_str(
            r#"{"min":{"value":0.20,"unit":"v/v","operator":"EXCLUSIVE"},
                "max":{"value":0.20,"unit":"v/v"}}"#,
        )
        .unwrap();
        assert!(
            touching
                .canonical(CutoffQuantity::VolumeFraction, CutoffSense::Minimum, "PHIE")
                .is_err(),
            "bounds that meet with either side exclusive admit nothing either"
        );

        // F — WIRED IN, and the pair is the point: the SAME well and the SAME number classify
        // differently on the operator alone. A sample sitting exactly on the cut-off is the
        // population that decides a marginal result, so the operator has to reach the arithmetic.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_weighting_well(&conn, "SANDI-BOUND-1", "PHIE");
        let dbm = Mutex::new(conn);
        let net_with = |operator: &str| {
            let spec: CutoffSpec = serde_json::from_str(&format!(
                r#"{{"min":{{"value":0.30,"unit":"v/v","operator":"{operator}"}}}}"#
            ))
            .unwrap();
            run_pay_summary(
                &dbm,
                &crate::reader_pool::ReaderPool::new(),
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: None,
                    phie_min: Some(spec),
                    swe_max: None,
                    perm_min: None,
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    weighting: Default::default(),
                    frame: Default::default(),
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                },
            )
            .expect("the run itself is valid under both operators")
            .iter()
            .filter(|row| row.flag == "PAY")
            .map(|row| row.net as f64)
            .sum::<f64>()
        };
        let inclusive = net_with("INCLUSIVE");
        let exclusive = net_with("EXCLUSIVE");
        assert!(
            inclusive > exclusive,
            "the fixture's PHIE sits exactly on 0.30, so an inclusive bound must book footage an \
             exclusive one does not: inclusive {inclusive}, exclusive {exclusive}"
        );
    }

    /// T-BATCH-08 (1) — a permeability cutoff applies to every well it is asked for, including the
    /// ones with no permeability.
    ///
    /// `classify_sample` is emphatic that a SAMPLE with no PERM cannot demonstrate it passes an
    /// active cutoff, so it fails (`classify_sample_nan_propagation` pins that, and it is a
    /// confirmed `[x]` in REVIEW.md). Until 2026-08-01 whether the cutoff was active at all was
    /// decided per WELL one line earlier — `perm_min.is_some() && perm.iter().any(|v| !v.is_nan())`
    /// — so a well carrying NO permeability anywhere switched the cutoff off for itself and
    /// reported its full pay. Two halves of one rule, disagreeing in the damaging direction: the
    /// well that measured 1 mD against a 1000 mD cutoff was excluded while the well that measured
    /// nothing sailed through, and in a field roll-up those rows added together.
    ///
    /// Jauhar's call, 2026-08-01 (`docs/review_triage.md` finding 7): *"no relation between em,
    /// wells still can have perm curves"* — a cutoff's applicability has no relation to whether
    /// this well happened to be cored, and permeability can be modelled where it was not measured.
    /// The well-level test is gone; the sample-level rule is the only one left.
    ///
    /// Both halves of the outcome are asserted, because the reason this needed a decision at all is
    /// that the safe-looking half is only half: the uncored well now books zero, which on a page is
    /// indistinguishable from a wet well. `perm_cutoff_no_data` is what separates them, and it is
    /// asserted here rather than left to the report to remember.
    #[test]
    fn a_well_with_no_perm_fails_the_cutoff_and_says_why() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        // Identical rock. The ONLY difference is whether permeability was measured.
        let no_perm = seed_pay_well(&conn, "PAY-NOPERM", None);
        let low_perm = seed_pay_well(&conn, "PAY-LOWPERM", Some(1.0));
        let dbm = Mutex::new(conn);

        let summary = |perm_min: Option<f64>| -> Vec<PaySummaryRow> {
            run_pay_summary(
                &dbm,
                &crate::reader_pool::ReaderPool::new(),
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![no_perm.clone(), low_perm.clone()],
                    vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                    phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
                    swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                    perm_min: perm_min.map(|p| CutoffEntry { value: p, unit: "mD".into() }.into()),
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    frame: Default::default(),
                    weighting: Default::default(),
                },
            )
            .expect("summary runs")
        };
        let pay = |rows: &[PaySummaryRow], w: &str| -> PaySummaryRow {
            rows.iter().find(|r| r.well_id == w && r.flag == "PAY").expect("a PAY row per well").clone()
        };

        // Baseline: with no PERM cutoff at all, both wells are full pay. This is the control —
        // it establishes the rock is identical, so anything below is the cutoff's doing.
        let open = summary(None);
        let base_no_perm = pay(&open, &no_perm).net;
        let base_low_perm = pay(&open, &low_perm).net;
        assert!(base_no_perm > 0.0, "the test rock must be pay before any cutoff is applied");
        assert_eq!(base_no_perm, base_low_perm, "both wells must start as the same rock");

        // Now a cutoff nothing in either well could pass.
        let cut = summary(Some(1000.0));

        // The well that MEASURED permeability, at 1 mD, is correctly excluded.
        assert_eq!(pay(&cut, &low_perm).net, 0.0, "1 mD cannot pass a 1000 mD cutoff");

        // And so is the well that measured none — it cannot be SHOWN to pass, which is the same
        // test the sample-level rule already applied. The two halves now agree.
        assert_eq!(
            pay(&cut, &no_perm).net,
            0.0,
            "a well with no PERM must fail an active cutoff, not be exempted from it"
        );
        assert_eq!(pay(&cut, &no_perm).hpv, 0.0, "and it books no hydrocarbon volume on missing data");

        // Both wells were fully interpreted, so `n_classified` is > 0 on both and cannot say why
        // either one came back at zero. It never could — which is why a SECOND discriminator was
        // needed rather than a cleverer reading of this one.
        assert!(pay(&cut, &no_perm).n_classified > 0);
        assert!(pay(&cut, &low_perm).n_classified > 0);

        // `perm_cutoff_no_data` is that discriminator, and it is the whole reason a zero here is
        // readable: the uncored well's zero means "nothing to judge with", the cored well's means
        // "judged and failed". Identical numbers, opposite statements.
        assert!(pay(&cut, &no_perm).perm_cutoff_no_data, "the well with no data must be marked");
        assert!(!pay(&cut, &low_perm).perm_cutoff_no_data, "the well that was judged must not be");

        // And it means "a cutoff was requested and this well has nothing to answer it with" — not
        // "this well has no permeability". With no cutoff asked for there is nothing to report, and
        // a flag that fired anyway would appear on every report anyone ever ran without one.
        assert!(!pay(&open, &no_perm).perm_cutoff_no_data, "no cutoff requested, nothing to say");
        assert_eq!(pay(&open, &no_perm).net, base_no_perm, "and with no cutoff the pay is untouched");
    }

    /// T-BATCH-08 (3) — one unusable well must not zero the whole response.
    ///
    /// `run_pay_summary` `continue`s past a well whose curve frame or zone read fails instead of
    /// `?`-aborting the batch. The bare well is listed FIRST here on purpose: an abort would take
    /// the good well's rows with it, and a test that put the good well first would pass either way.
    #[test]
    fn one_unusable_well_cannot_zero_the_whole_pay_summary() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        // A well record with no curve data at all — an import that failed, or a well created by hand.
        let bare_id = uuid::Uuid::new_v4();
        db::insert_well(&conn, bare_id, "PAY-BARE", Some("Synthetic"), None, None).unwrap();
        let bare = bare_id.to_string();
        let good = seed_pay_well(&conn, "PAY-GOOD", Some(500.0));
        let dbm = Mutex::new(conn);

        let rows = run_pay_summary(
            &dbm,
            &crate::reader_pool::ReaderPool::new(),
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![bare.clone(), good.clone()],
                vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
            },
        )
        .expect("a bare well must not fail the batch");

        let good_pay = rows.iter().find(|r| r.well_id == good && r.flag == "PAY").expect("the good well still reports");
        assert!(good_pay.net > 0.0, "the good well keeps its full answer: {good_pay:?}");

        // The bare well contributes NO rows — it is skipped, not reported as a zero. A zero row
        // would be indistinguishable from a genuinely wet zone in the Field Dashboard.
        assert!(
            !rows.iter().any(|r| r.well_id == bare),
            "a well with no curves must be absent, not present with zeros"
        );
    }

    /// The NaN guard in `floored_phie` is load-bearing rather than defensive: `f32::max` returns
    /// the OTHER side when one is NaN, so without it a MISSING porosity would come back as a real
    /// 0.001 and start counting toward `n_classified` — the one field that says whether the well
    /// was interpreted at all.
    #[test]
    fn flooring_phie_leaves_missing_missing() {
        let out = floored_phie(&[-0.05, 0.0, f32::NAN, 0.25]);
        let floor = modules::PHIE_FLOOR as f32;
        assert_eq!(out[0], floor, "a negative porosity is floored");
        assert_eq!(out[1], floor, "and so is a hard zero — the floor is 0.001, not 0.0");
        assert!(out[2].is_nan(), "MISSING must stay MISSING");
        assert_eq!(out[3], 0.25, "a real porosity is untouched");
    }

    /// Sweeping the VSH (sand) cutoff upward can only admit more pay, so the metric is
    /// monotone non-decreasing; the peak lands at the most permissive cutoff.
    #[test]
    fn cutoff_sweep_vsh_monotone() {
        let vsh = [0.1f32, 0.3, 0.5, 0.7, 0.9];
        let phie = [0.2f32; 5];
        let swe = [0.3f32; 5];
        let perm = [f32::NAN; 5];
        // Each sample contributes a full 1 m of clamped thickness.
        let incl_h = [1.0f64; 5];
        let (cuts, vals, peak) = compute_sweep(
            &vsh, &phie, &swe, &perm, &incl_h, SweepProp::Vsh, at_most(0.5), at_least(0.1), at_most(0.6), None, 0.0, 1.0,
            11, Metric::Net, 5.0,
        );
        assert_eq!(cuts.len(), 11);
        for w in vals.windows(2) {
            assert!(w[1] >= w[0] - 1e-9, "not monotone: {:?}", vals);
        }
        assert!((vals[0] - 0.0).abs() < 1e-9); // cutoff 0.0 → no sample has VSH ≤ 0
        assert!((peak - 5.0).abs() < 1e-9); // cutoff 1.0 → all 5 m of pay
    }

    /// DEC-084, verbatim: *"no well with no perm can escape cutoff … dont let it off, its
    /// independent."* The ruling was applied to `run_pay_summary` and to the Monte Carlo path,
    /// and this third site — the cutoff SENSITIVITY sweep — kept the well-level "does this well
    /// have any PERM?" exemption, with a comment claiming agreement with `run_pay_summary` that
    /// stopped being true when that function was corrected.
    ///
    /// The consequence was two screens disagreeing about the same held cutoffs on the same well:
    /// the pay summary booked ZERO net and marked `perm_cutoff_no_data`, while the sensitivity
    /// curve beside it dropped the cutoff entirely and drew a full, optimistic pay curve — the
    /// optimistic one being the one a user reads when choosing where to set a cutoff.
    ///
    /// Pinned from three sides so neither an always-active nor a never-active implementation
    /// passes: no PERM under an active cutoff books nothing, a measured PERM that clears the
    /// cutoff still books everything, and a well with no PERM and NO cutoff requested is
    /// untouched — absence of evidence must only bite when the evidence was actually asked for.
    #[test]
    fn a_sensitivity_sweep_cannot_drop_a_perm_cutoff_the_well_has_no_data_for() {
        let vsh = [0.1f32; 4];
        let phie = [0.2f32; 4];
        let swe = [0.3f32; 4];
        let incl_h = [1.0f64; 4];
        let sweep = |perm: &[f32; 4], perm_min: Option<CutoffRange>| {
            compute_sweep(
                &vsh, &phie, &swe, perm, &incl_h, SweepProp::Vsh,
                at_most(0.5), at_least(0.1), at_most(0.6), perm_min,
                0.0, 1.0, 11, Metric::Net, 4.0,
            )
        };

        // No permeability anywhere, and a PERM >= 10 mD cutoff explicitly entered. Nothing here
        // can demonstrate it passes, so nothing books — at EVERY step of the sweep, not just the
        // strict end, because the swept property is VSH and the PERM gate is independent of it.
        let (_, vals, peak) = sweep(&[f32::NAN; 4], at_least(10.0));
        assert!(
            peak.abs() < 1e-9 && vals.iter().all(|v| v.abs() < 1e-9),
            "a well with no PERM must book zero against an active PERM cutoff, got peak {peak} \
             over {vals:?}"
        );

        // The control: measured permeability that clears the same cutoff still books the full
        // 4 m at the permissive end. Without this the assertion above would pass just as well
        // against a sweep that had stopped booking anything at all.
        let (_, _, measured_peak) = sweep(&[50.0f32; 4], at_least(10.0));
        assert!(
            (measured_peak - 4.0).abs() < 1e-9,
            "a well whose PERM clears the cutoff must still book its pay, got {measured_peak}"
        );

        // And absence of evidence only bites when the evidence was asked for: no PERM curve and
        // no PERM cutoff is an ordinary run that must be completely unaffected.
        let (_, _, no_cutoff_peak) = sweep(&[f32::NAN; 4], None);
        assert!(
            (no_cutoff_peak - 4.0).abs() < 1e-9,
            "with no PERM cutoff requested a missing PERM curve must change nothing, got \
             {no_cutoff_peak}"
        );
    }

    /// NTG divides by the geometric gross; the DST `included` mask drops samples and scales
    /// net down accordingly.
    #[test]
    fn cutoff_sweep_ntg_and_dst_mask() {
        let vsh = [0.2f32; 4];
        let phie = [0.2f32; 4];
        let swe = [0.3f32; 4];
        let perm = [f32::NAN; 4];
        // All four samples at full 1 m thickness, gross 4 → every sample pays at a generous
        // SWE cutoff → NTG 1.0.
        let all = [1.0f64; 4];
        let (_, vals, _) = compute_sweep(
            &vsh, &phie, &swe, &perm, &all, SweepProp::Swe, at_most(0.5), at_least(0.1), at_most(0.6), None, 0.0, 1.0, 3,
            Metric::Ntg, 4.0,
        );
        assert!((vals[2] - 1.0).abs() < 1e-9);
        // DST clips two samples to zero thickness → NET tops out at 2 m.
        let half = [1.0f64, 1.0, 0.0, 0.0];
        let (_, vals2, _) = compute_sweep(
            &vsh, &phie, &swe, &perm, &half, SweepProp::Swe, at_most(0.5), at_least(0.1), at_most(0.6), None, 0.0, 1.0,
            3, Metric::Net, 2.0,
        );
        assert!((vals2[2] - 2.0).abs() < 1e-9);
    }

    /// Overlapping perforation/DST rows must union, not double-count: two rows (2000,2010) and
    /// (2005,2015) cover 15 m, not 20 m, so the NTG gross stays consistent with net thickness.
    #[test]
    fn aux_intervals_merges_overlaps() {
        let row = |t: f32, b: Option<f32>| db::AuxRow {
            dataset: "DST".into(),
            depth_top: t,
            depth_base: b,
            item: String::new(),
            value_num: None,
            value_text: None,
        };
        // Overlapping + a nested + an exact duplicate + a point row (dropped).
        let rows = vec![
            row(2000.0, Some(2010.0)),
            row(2005.0, Some(2015.0)), // overlaps the first → union to (2000,2015)
            row(2006.0, Some(2008.0)), // nested inside → absorbed
            row(2005.0, Some(2015.0)), // exact duplicate → absorbed
            row(2100.0, None),         // point row → ignored
            row(2050.0, Some(2050.0)), // zero-length → ignored
            row(2030.0, Some(2040.0)), // disjoint → its own interval
        ];
        let iv = aux_intervals(&rows);
        assert_eq!(iv, vec![(2000.0, 2015.0), (2030.0, 2040.0)]);
        let gross: f32 = iv.iter().map(|(t, b)| b - t).sum();
        assert!((gross - 25.0).abs() < 1e-4, "gross should be 15+10, got {gross}");
    }

    /// Regression for the "step bleed past boundary" bug in the sweep engine: when a zone base
    /// falls mid-sample, the sweep must count only each sample's in-zone overlap (fed via
    /// incl_h), so net ≤ gross and NTG ≤ 1 — matching run_pay_summary on the identical fixture.
    /// Previously compute_sweep summed each included sample's full step and reported NTG ≈ 1.33.
    #[test]
    fn compute_sweep_clamps_thickness_via_incl_h() {
        // depths 1000..1003 (step 1.0), zone [1000, 1001.5): overlaps 1.0, 0.5, 0, 0 → gross 1.5.
        let vsh = [0.1f32; 4];
        let phie = [0.2f32; 4];
        let swe = [0.3f32; 4];
        let perm = [f32::NAN; 4];
        let incl_h = [1.0f64, 0.5, 0.0, 0.0];
        // Permissive cutoffs: every in-zone sample pays → net = 1.5 (the clamped overlap), NOT
        // 2.0 (two full steps), so peak net is 1.5 and NTG never exceeds 1.
        let (_, _, peak) = compute_sweep(
            &vsh, &phie, &swe, &perm, &incl_h, SweepProp::Swe, at_most(0.9), at_least(0.0), at_most(1.0), None, 0.0, 1.0, 2,
            Metric::Net, 1.5,
        );
        assert!((peak - 1.5).abs() < 1e-9, "net must be the clamped 1.5 m, not 2.0; got {peak}");
        let (_, ntg, _) = compute_sweep(
            &vsh, &phie, &swe, &perm, &incl_h, SweepProp::Swe, at_most(0.9), at_least(0.0), at_most(1.0), None, 0.0, 1.0, 2,
            Metric::Ntg, 1.5,
        );
        assert!(ntg[1] <= 1.0 + 1e-9, "NTG must not exceed 1; got {}", ntg[1]);
    }

    /// The per-sample geometric clamp: a sample's overlap with the zone, then intersected with
    /// the DST intervals when present.
    #[test]
    fn sample_incl_thickness_clamps_zone_and_dst() {
        // Sample [1001,1002] vs zone [1000,1001.5): 0.5 m in zone.
        assert!((sample_incl_thickness(1001.0, 1002.0, 1000.0, 1001.5, None) - 0.5).abs() < 1e-9);
        // Fully outside the zone → 0.
        assert_eq!(sample_incl_thickness(1002.0, 1003.0, 1000.0, 1001.5, None), 0.0);
        // Zone overlap [1000,1002]; DST intervals (1000.5,1001)+(1001.5,1002) → 0.5+0.5 = 1.0.
        let dst = [(1000.5f32, 1001.0f32), (1001.5, 1002.0)];
        let h = sample_incl_thickness(1000.0, 1002.0, 999.0, 1003.0, Some(&dst));
        assert!((h - 1.0).abs() < 1e-9, "DST-clipped overlap should be 1.0, got {h}");
    }

    #[test]
    fn pay_summary_clamps_thin_zone_and_normalizes_avg_phie_over_valid() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "PAY-1", None, None, Some(0.0)).unwrap();
        let w = wid.to_string();

        let depths = vec![1000.0f32, 1001.0, 1002.0, 1003.0];
        let n = depths.len();
        // Standard curves supply the depth spine; the interpretation curves are computed.
        db::insert_standard_curves(
            &conn, wid, depths.clone(),
            vec![50.0; n], vec![f32::NAN; n], vec![f32::NAN; n],
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        // All sand; sample 1 has valid VSH but MISSING PHIE (the SAND-row dilution case).
        equations::write_computed_curve(&conn, &w, &depths, "VSH", &[0.1, 0.1, 0.1, 0.1]).unwrap();
        equations::write_computed_curve(&conn, &w, &depths, "PHIE", &[0.2, f32::NAN, 0.2, 0.2]).unwrap();
        equations::write_computed_curve(&conn, &w, &depths, "SWE", &[0.3, 0.3, 0.3, 0.3]).unwrap();
        equations::write_computed_curve(&conn, &w, &depths, "PERM", &[f32::NAN; 4]).unwrap();
        // A zone thinner than one sample step (1.5 m vs 1.0 m steps): the last in-zone sample
        // must not bleed past the base, so net must equal gross (1.5), not overshoot to 2.0.
        db::upsert_md_zone(&conn, &w, "Z1", 1000.0, 1001.5).unwrap();

        let dbm = Mutex::new(conn);
        let req = PaySummaryRequest {
            discretisation: DiscretisationModel::Forward,
            input_set: None,
            well_ids: vec![w.clone()],
            vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
            swe_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            perm_min: None,
            enabled_unset: Vec::new(),
            cutoff_use: Default::default(),
            skip_version: false,
            stats_only: true,
            custody: None,
            frame: Default::default(),
            weighting: Default::default(),
        };
        let rows = run_pay_summary(&dbm, &crate::reader_pool::ReaderPool::new(), &req).unwrap();
        let sand = rows.iter().find(|r| r.zone == "Z1" && r.flag == "SAND").expect("SAND row");

        // Overlap clamp: net never exceeds gross (old forward-step gave net 2.0 > gross 1.5).
        assert!((sand.gross - 1.5).abs() < 1e-3, "gross={}", sand.gross);
        assert!(sand.net <= sand.gross + 1e-4, "net {} must not exceed gross {}", sand.net, sand.gross);
        assert!((sand.net - 1.5).abs() < 1e-3, "net={}", sand.net);
        // avg_phie normalised over PHIE-valid net (→ 0.2), not diluted by the missing-PHIE
        // sample (old code divided sum_phie by total net → ~0.1).
        assert!((sand.avg_phie - 0.2).abs() < 1e-3, "avg_phie={}", sand.avg_phie);
    }

    /// AUDIT-2026-08-20 finding 55. The cut-off slots and report tiers were `&str`, and every
    /// match over them carried a catch-all. The slot fallback landed on PERM - the branch with
    /// teeth, because a requested permeability cut-off is always active and its missing samples
    /// FAIL rather than pass - and the tier fallback landed on PAY, the strictest. Both are enums
    /// now, so neither fallback can be written: a fifth slot stops the BUILD at each place that
    /// has to decide what it means.
    ///
    /// A compile error cannot be pinned at runtime, so this pins the two routings the catch-alls
    /// governed, from BOTH sides - one cut-off declared, and the two questions it answers
    /// separated:
    ///   A - WHICH TIERS it is applied at (`CutoffUse::at`, whose fallback was PAY).
    ///   B - WHICH SLOT's value it filters (`TierCutoffs::applied`, whose fallback was PERM).
    #[test]
    fn one_cut_off_reaches_its_own_slot_and_only_the_tiers_that_use_it() {
        // Only a POROSITY cut-off is declared, and the sample is below it. SB-CUT-022's shipped
        // ladder makes porosity a reservoir-and-pay cut-off, never a sand one - net sand is
        // clay-driven.
        let (sand, reservoir, pay) =
            classify_sample(0.2, 0.05, 0.3, f32::NAN, &ladder(None, at_least(0.1), None, None), false);

        // A - SAND is untouched. A tier match that collapsed to PAY would apply the porosity
        // cut-off here too, and the well would lose net sand it never lost.
        assert_eq!(
            sand, 1.0,
            "porosity is not a net-sand cut-off; a tier fallback would have applied it anyway"
        );

        // B - and RESERVOIR and PAY do fail, which is the proof the value reached PHIE's own slot.
        // A slot match that fell through to PERM would have read the (absent) permeability
        // cut-off instead, filtered nothing, and booked all three tiers as pay.
        assert_eq!(
            (reservoir, pay),
            (0.0, 0.0),
            "the declared porosity cut-off must filter PHIE at the tiers that use it"
        );
    }
}
