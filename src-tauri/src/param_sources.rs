//! Competing shipped values for one parameter, with their sources (`SB-CORE-013`, `SB-MLA-031`).
//!
//! Three packages routinely ship three different values for one constant, and **none of them tells
//! the interpreter that the others exist** — none of them can, because no vendor can credibly
//! publish a competitor's defaults. SandiBumi has no such constraint, which is the whole reason this
//! module exists: showing the disagreement at the point of choice is a capability the incumbents are
//! structurally unable to copy (`docs/PRD_v2/03_EVIDENCE_BASE.md` §14.2).
//!
//! **The boundary, from `04_CORE_REQUIREMENTS.md` `SB-CORE-013`: this surfaces VALUES WITH SOURCES,
//! never vendor algorithms, tables or text.** A shipped default is one documented fact about a
//! product, cited to the page that documents it. A lookup table is somebody's work product, and
//! `CONTRACT.md` §2.1 keeps it out of this tree. Nothing here transcribes one, and an entry that
//! needed a table to be understood would be the wrong entry.
//!
//! **An absence is an entry.** Geolog stating no cluster count anywhere in Facimage is as much a
//! finding as Techlog shipping 5, and it is the entry that tells an interpreter the number is not
//! settled. Dropping it because it has no value to print would leave two vendors looking like a
//! consensus.
//!
//! **SandiBumi's own default is listed with the others, and never at the top.** A panel that showed
//! three competitors and hid our own provenance would be making exactly the omission it exists to
//! correct.
//!
//! No tier letters. The ML chapter cites `(T2)` / `(T3)` as CORPUS identifiers — which vendor's
//! documentation set a claim came from — while `03_EVIDENCE_BASE.md` §2 uses `T1`–`T4` for
//! something else entirely, the kind of artefact a claim was read from. Printing one letter under
//! the other's meaning is how a provenance record starts lying, so each entry names its product and
//! its document instead, and points at the chapter that holds the reasoning.

use serde::Serialize;

/// One product's shipped or advised value for a parameter.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ParamSource {
    /// The product that ships or advises it — including SandiBumi.
    pub product: &'static str,
    /// The value as the source states it: a number, a range, or the fact that none is stated.
    /// A STRING because "15-20" and "none stated" are both real answers and neither is an `f64`.
    pub value: &'static str,
    /// What the value is FOR, where the source distinguishes stages or modules.
    pub note: &'static str,
    /// Where the claim was read. Empty for SandiBumi's own, which is this repository.
    pub source: &'static str,
}

/// Topic key for the number of clusters / facies / classes.
pub const CLUSTER_COUNT: &str = "cluster_count";

/// The corpus's densest disagreement (`24_ml-advanced.md` SB-MLA-031). Ordered vendor-first with
/// SandiBumi last, deliberately — see the module note.
const CLUSTER_COUNT_SOURCES: &[ParamSource] = &[
    ParamSource {
        product: "Interactive Petrophysics",
        value: "15-20",
        note: "advised as a FIRST-STAGE count, to be consolidated afterwards",
        source: "cluster_analysis.htm (IP install helpset)",
    },
    ParamSource {
        product: "Interactive Petrophysics",
        value: "4-5",
        note: "advised as the CONSOLIDATED count, after merging the first-stage clusters",
        source: "cluster_analysis.htm (IP install helpset)",
    },
    ParamSource {
        product: "Techlog",
        value: "5",
        note: "a hard shipped default, the same in two independent modules",
        source: "TechCore petrophysical groups + SOM parameter tables (Techlog help)",
    },
    ParamSource {
        product: "Geolog",
        value: "none stated",
        note: "no default and no advised count anywhere in the Facimage suite - the reading that \
               says this number is not settled",
        source: "Facimage help set (Geolog)",
    },
    ParamSource {
        product: "SandiBumi",
        value: "5",
        note: "this application's shipped default. It is not a fitted or field-derived number and \
               carries no external authority - it is a starting point, and the values above are \
               why it is offered as one rather than as an answer",
        source: "",
    },
];

/// The competing values recorded for `topic`, or empty where the corpus records none.
pub fn sources_for(topic: &str) -> &'static [ParamSource] {
    match topic {
        CLUSTER_COUNT => CLUSTER_COUNT_SOURCES,
        _ => &[],
    }
}

/// One line recording what the interpreter actually chose, for the run's own provenance.
///
/// The value alone is already stored in every run's parameters; what this adds is that it was
/// chosen against a KNOWN disagreement, and where it sits in it. "K = 5" read back in a year says
/// nothing about whether anybody considered 15; "K = 5, matching Techlog's shipped default, where
/// IP advises 15-20 first-stage" is a decision.
pub fn decision_note(topic: &str, value: f64) -> Option<String> {
    let sources = sources_for(topic);
    if sources.is_empty() {
        return None;
    }
    let shown = format!("{}", (value * 1000.0).round() / 1000.0);
    // Which cited values, if any, the chosen number agrees with. A range counts when the value
    // falls inside it — an interpreter who typed 17 did take IP's advice, and a record that only
    // matched exact numbers would say they invented it.
    let agrees: Vec<String> = sources
        .iter()
        .filter(|s| value_agrees(s.value, value))
        .map(|s| format!("{} ({})", s.product, s.value))
        .collect();
    let all: Vec<String> =
        sources.iter().map(|s| format!("{} {}", s.product, s.value)).collect();
    Some(format!(
        "cluster count = {shown}, chosen where the corpus records competing values [{}]. {}",
        all.join("; "),
        if agrees.is_empty() {
            "This value matches none of them - it is the interpreter's own.".to_string()
        } else {
            format!("It agrees with: {}.", agrees.join(", "))
        }
    ))
}

/// Whether a cited value — a number or an inclusive `a-b` range — covers `value`.
///
/// "none stated" agrees with nothing, and that is the point: a vendor that ships no default cannot
/// be cited as endorsing whatever the user typed.
fn value_agrees(cited: &str, value: f64) -> bool {
    let c = cited.trim();
    if let Some((lo, hi)) = c.split_once('-') {
        if let (Ok(lo), Ok(hi)) = (lo.trim().parse::<f64>(), hi.trim().parse::<f64>()) {
            return value >= lo && value <= hi;
        }
        return false;
    }
    c.parse::<f64>().map(|v| (v - value).abs() < 1e-9).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **SB-MLA-031 / SB-CORE-013.** The panel exists to show that a number is contested. Three
    /// things would each quietly defeat that, and none would fail any other test:
    /// dropping the vendor that states nothing (leaving two values that look like a consensus),
    /// hiding SandiBumi's own default (making our starting point look like neutral ground), and
    /// shipping an entry with no source (which is an assertion, not evidence).
    #[test]
    fn every_competing_value_names_its_product_and_the_absence_of_one_is_itself_shown() {
        let s = sources_for(CLUSTER_COUNT);
        assert!(s.len() >= 4, "the corpus records at least four positions on cluster count");
        for e in s {
            assert!(!e.product.is_empty(), "an unattributed value is an assertion, not evidence");
            assert!(!e.value.is_empty());
            assert!(!e.note.is_empty(), "a bare number cannot be judged: {e:?}");
            // Every VENDOR claim carries the document it was read from. SandiBumi's own is this
            // repository and says so by carrying none.
            if e.product != "SandiBumi" {
                assert!(!e.source.is_empty(), "vendor claim with no source: {e:?}");
            }
        }
        // The vendor that ships nothing is present, and its absence is stated as a value.
        assert!(
            s.iter().any(|e| e.product == "Geolog" && e.value == "none stated"),
            "a vendor stating no default is a finding, not a gap to omit"
        );
        // Our own default is listed, and is NOT first — the panel must not read as ours plus
        // three footnotes.
        let ours = s.iter().position(|e| e.product == "SandiBumi").expect("our own default is shown");
        assert!(ours > 0, "SandiBumi's default must not head the list");
        assert_eq!(s[ours].value, "5", "must track facies.rs's shipped K default");
        assert!(
            s[ours].note.contains("not a fitted or field-derived number"),
            "our own default must disclaim authority it does not have"
        );
        // An unknown topic yields nothing rather than a plausible-looking empty panel elsewhere.
        assert!(sources_for("no_such_topic").is_empty());
    }

    /// **The decision record.** A stored `K = 5` says nothing about whether anybody knew 15 was on
    /// the table. Pinned from both sides: a value inside a cited range must be recorded as agreeing
    /// with it — an interpreter who typed 17 DID take IP's advice — and a value matching nothing
    /// must be recorded as their own rather than silently attributed to the nearest vendor.
    #[test]
    fn the_recorded_choice_says_which_cited_values_it_agrees_with_and_when_it_agrees_with_none() {
        let note = decision_note(CLUSTER_COUNT, 5.0).expect("a contested parameter records its choice");
        assert!(note.contains("Techlog 5"), "every competing value is listed: {note}");
        assert!(note.contains("Geolog none stated"), "the absence is listed too: {note}");
        assert!(note.contains("agrees with"), "{note}");
        assert!(note.contains("Techlog (5)"), "{note}");

        // Inside IP's first-stage range: agreement with a RANGE counts.
        let seventeen = decision_note(CLUSTER_COUNT, 17.0).unwrap();
        assert!(seventeen.contains("Interactive Petrophysics (15-20)"), "{seventeen}");
        assert!(!seventeen.contains("Techlog (5)"), "17 does not agree with 5: {seventeen}");

        // Matching nothing is recorded as the interpreter's own, not rounded to a vendor.
        let nine = decision_note(CLUSTER_COUNT, 9.0).unwrap();
        assert!(nine.contains("matches none of them"), "{nine}");
        assert!(nine.contains("interpreter's own"), "{nine}");

        // "none stated" endorses nothing, whatever was typed.
        for k in [3.0, 5.0, 17.0, 100.0] {
            let n = decision_note(CLUSTER_COUNT, k).unwrap();
            assert!(!n.contains("Geolog (none stated)"), "a vendor with no default endorses nothing: {n}");
        }
        // A parameter the corpus says nothing about records nothing, or every field grows a panel.
        assert!(decision_note("no_such_topic", 5.0).is_none());
    }
}
