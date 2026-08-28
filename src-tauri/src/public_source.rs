//! What a citation looks like to the person USING SandiBumi, as opposed to the person building it.
//!
//! A parameter's `default_source`, a precondition's `source` and a module's `doc` are the repository's
//! provenance record: they name the specification chapter, the vendor help file or the Loglan source a
//! value was adopted from, and that record is why a default can be defended. A customer has none of
//! those files. Printed into a tooltip or a refusal, an internal path is a pointer at something they
//! cannot open, and it states more about how the software is built than a manual should.
//!
//! So the stored strings are never edited. This module renders them, at the point where a source
//! becomes text a user reads - the module manifest handed over IPC, and the refusal messages built in
//! `modules.rs`. Every existing gate that asserts on a raw source keeps passing, because the raw
//! source is what it still asserts on.
//!
//! Two rules do most of the work, and the second is the one that is easy to get wrong:
//!
//! - An internal path leaves. So does the REFERENCE TAIL after it - section numbers, figure ids,
//!   requirement ids, line numbers, and the conjunctions joining them - because "F17 and section 5"
//!   left standing on its own points at nothing. The tail stops at the first word that is none of
//!   those, so "section 5 porosity limits" keeps "porosity limits", which is the half worth reading.
//! - A vendor FILE name leaves and the vendor PRODUCT name stays. "Geolog phi_den.info RHO_W DEFAULT
//!   1000 k/m3" becomes "Geolog RHO_W DEFAULT 1000 k/m3": a petrophysicist knows Geolog, and the
//!   attribution survives, which is what `CLAUDE.md`'s never-strip-an-attribution rule protects. The
//!   filename was only ever how WE find it again.
//!
//! [`ABSENT_DEFAULT_SOURCE`](crate::modules::ABSENT_DEFAULT_SOURCE) passes through untouched: it is a
//! sentinel the frontend matches on, not prose, and rewriting it would silently break that check.

use regex::Regex;
use std::sync::OnceLock;

use crate::modules::{ModuleSpec, ABSENT_DEFAULT_SOURCE};

/// Raw sources whose mechanical rendering is correct but reads badly - a lost possessive, a clause
/// whose subject was the filename itself, a stray section number the tail rule cannot reach because
/// it sits mid-sentence. Authored rather than regex-tuned: eleven exceptions are cheaper to read than
/// the rules that would be needed to catch them, and each rule added for one string risks the other
/// eighty-two.
///
/// Keyed on the exact raw string, so an edit upstream silently falls back to the mechanical form.
/// `every_override_still_matches_a_shipping_source` is what stops that being silent.
const OVERRIDES: &[(&str, &str)] = &[
    (
        "docs/PRD_v2/20_envcorr-qc.md SB-ENV-006 and section 6.2 T12",
        "SandiBumi environmental-correction specification",
    ),
    (
        "docs/PRD_v2/20_envcorr-qc.md SB-ENV-006 and section 6.2 T11/T12; DEC-031 (2026-08-17)",
        "SandiBumi environmental-correction specification; DEC-031 (2026-08-17)",
    ),
    (
        "docs/PRD_v2/10_clay-volume.md §3.5 F17 and §5.1; docs/workflow_standards.md",
        "SandiBumi clay-volume specification and house workflow standards",
    ),
    (
        "docs/PRD_v2/10_clay-volume.md §3.5 F17 and §5; docs/workflow_standards.md",
        "SandiBumi clay-volume specification and house workflow standards",
    ),
    (
        "docs/PRD_v2/19_toc-unconventional.md SB-TOC-019 and §5",
        "SandiBumi unconventional-methods specification",
    ),
    (
        "docs/PRD_v2/11_porosity.md §5.6 Bateman-Konen crossplot constants, §5 porosity limits and DEC-015",
        "Bateman-Konen crossplot constants; porosity limits and DEC-015",
    ),
    (
        "IP MINDEF, Techlog QM_MineralTable and SandiMin all 2.65 (3-way AGREE); docs/PRD_v2/11_porosity.md §5.1. SB-POR-011: one shared matrix density across chained modules, owner-selected 2026-08-16 over Geolog phi_den.info's shipped 2645 k/m3.",
        "IP MINDEF, Techlog QM_MineralTable and SandiMin all 2.65 (3-way AGREE); one shared matrix density across chained modules, owner-selected 2026-08-16 over Geolog's shipped 2645 k/m3.",
    ),
    (
        "Geolog V14 phi_*.lls hard-coded VSH >= 0.95 (all six modules); docs/PRD_v2/11_porosity.md §5 line 1229 makes it a parameter in SandiBumi defaulting to 0.95 with this source",
        "Geolog V14 hard-coded VSH >= 0.95 (all six modules); SandiBumi makes it a parameter defaulting to 0.95 with this source",
    ),
    (
        "sspw.lls (2025-02-28) gas branch writes the even split, PHIT = ((phiD^2 + NPHI^2)/2)^0.5, i.e. c = 1 - and that is what SSC ran until DEC-088 OVERRODE it, ruling 1.6 here too and extending DEC-086's field observation that the even split still reads optimistic. The source is unchanged; the shipped default departs from it deliberately",
        "The originating SSPW source (2025-02-28) writes the even split in its gas branch, PHIT = ((phiD^2 + NPHI^2)/2)^0.5, i.e. c = 1 - and that is what SSC ran until DEC-088 OVERRODE it, ruling 1.6 here too and extending DEC-086's field observation that the even split still reads optimistic. The source is unchanged; the shipped default departs from it deliberately",
    ),
    (
        "porosity_sspw.lls (2022) gas branch c = 1.6; RULED by DEC-086 on field observation that the even split still reads optimistic",
        "The originating SSPW source (2022) gas branch c = 1.6; RULED by DEC-086 on field observation that the even split still reads optimistic",
    ),
    (
        "Geolog V14 phi_dn.lls SCH_TNPH branch: phix = ((RHO_FL-1000)*(phit_2-phit_1)/190)+phit_1 - the input is the well's own fluid density and Geolog ships no default; docs/PRD_v2/11_porosity.md SB-POR-025 + F13",
        "Geolog V14 SCH_TNPH branch: phix = ((RHO_FL-1000)*(phit_2-phit_1)/190)+phit_1 - the input is the well's own fluid density and Geolog ships no default",
    ),
    (
        "SandiMin's own endpoint library carries this grain density (sandimin.rs LIB: clay Kaolinite RHOB 2.62, clay Illite RHOB 2.78), and docs/multimin_ref_spec.md:62 verifies the same pair against the reference-suite Multimin bound-water coefficients (Illite 0.1841, Kaolinite 0.0694). IP 2025 ships the matching un-expanded illite coefficient 0.185 (docs/research_2026-07/ip2025_chm_ingest/C_mineral_solver.md 3.4), so the two tools agree on this pair to three decimals",
        "SandiMin's own endpoint library carries this grain density (clay Kaolinite RHOB 2.62, clay Illite RHOB 2.78), verified against the reference-suite Multimin bound-water coefficients (Illite 0.1841, Kaolinite 0.0694). IP 2025 ships the matching un-expanded illite coefficient 0.185, so the two tools agree on this pair to three decimals",
    ),
];

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("public-source pattern must compile")
}

struct Patterns {
    internal_doc: Regex,
    vendor_file: Regex,
    rust_file: Regex,
    see_sentence: Regex,
    adjudication: Regex,
    bracketed: Regex,
    tidy: Vec<(Regex, &'static str)>,
}

fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| {
        // The reference tail: everything after an internal path that is a pointer rather than prose.
        let tail = concat!(
            r"(?:\s*(?:",
            r"§§?\s*[\d.]+(?:\s*,\s*[\d.]+)*",
            r"|§§?",
            r"|F\d+\b",
            r"|T\d+\b",
            r"|SB-[A-Z]{2,}-\d+(?:\([a-z]\))?",
            r"|line\s+\d+\b",
            r"|section\s+[\d.]+\b",
            r"|:\s*\d+\b",
            r"|and\b",
            r"|,",
            // "T01/T03", "T11/T12" - one reference written as a pair, not two.
            r"|/",
            r"|\x{2014}",
            r"|-\s",
            r"))*",
        );
        Patterns {
            internal_doc: re(&format!(r"(?i)\bdocs/[A-Za-z0-9_./-]+\.md\b{tail}")),
            // A vendor help file or Loglan source, with any line locator and possessive that follow it.
            vendor_file: re(r"(?i)\b[A-Za-z0-9_*.-]+\.(?:lls|info|htm|html|chm)\b(?:'s)?(?::\d+)?(?:\s+L\d+(?:\s*-\s*L?\d+)?)?"),
            rust_file: re(r"(?i)\b[a-z_]+\.rs\b(?::\d+(?:-\d+)?)?"),
            // A pointer SENTENCE ("See <internal chapter>.") goes whole, not just its path.
            see_sentence: re(r"(?i)\s*\bSee\s+docs/[A-Za-z0-9_./-]+\.md\b[^.]*\.?"),
            adjudication: re(r"(?i)^(?:Absent by adjudication|Adjudication|Ruling)\s+DEC-[\d\sR.-]+\([^)]*\)\s*:\s*"),
            bracketed: re(r"\(([^()]*)\)"),
            tidy: vec![
                (re(r"\(\s*[;,]?\s*\)"), ""),
                (re(r"\[\s*[;,]?\s*\]"), ""),
                (re(r"\s*[;,]\s*([)\]])"), "$1"),
                (re(r"([(\[])\s*[;,]\s*"), "$1"),
                (re(r"\s*;(?:\s*;)+"), ";"),
                (re(r"\s+([;,.:)\]])"), "$1"),
                (re(r"\s{2,}"), " "),
                (re(r"^[\s;,:.]+"), ""),
                (re(r"^[-\x{2014}]\s*"), ""),
                (re(r"[\s;,:]+$"), ""),
                (re(r"\s+[-\x{2014}]$"), ""),
            ],
        }
    })
}

fn tidy(text: &str) -> String {
    let mut out = text.to_string();
    for (pattern, replacement) in &patterns().tidy {
        out = pattern.replace_all(&out, *replacement).into_owned();
    }
    out.trim().to_string()
}

fn strip(text: &str) -> String {
    let p = patterns();
    let out = p.see_sentence.replace_all(text, " ");
    let out = p.internal_doc.replace_all(&out, " ");
    let out = p.vendor_file.replace_all(&out, " ");
    p.rust_file.replace_all(&out, " ").into_owned()
}

fn has_word(text: &str) -> bool {
    text.chars().any(|c| c.is_alphanumeric())
}

/// The reader-facing rendering of one stored source string.
///
/// Empty in, empty out. [`ABSENT_DEFAULT_SOURCE`] passes through verbatim.
pub fn public_source(raw: &str) -> String {
    if raw.is_empty() || raw == ABSENT_DEFAULT_SOURCE {
        return raw.to_string();
    }
    for (from, to) in OVERRIDES {
        if raw == *from {
            return (*to).to_string();
        }
    }
    let p = patterns();
    let without_prefix = p.adjudication.replace(raw, "").into_owned();

    // Inside brackets the references go but their siblings stay: "(<internal doc>; DEC-079)" must
    // still say DEC-079, which is an adjudication number a reader can act on. A bracket left holding
    // nothing is dropped whole rather than printed empty.
    let stripped_brackets = p.bracketed.replace_all(&without_prefix, |caps: &regex::Captures| {
        let kept = tidy(&strip(&caps[1]));
        if kept.is_empty() { String::new() } else { format!("({kept})") }
    });

    // Then clause by clause, because a clause that was ONLY a reference has to leave entirely:
    // what follows an internal path is that document's section, not an independent citation.
    let joined = stripped_brackets
        .split(';')
        .map(|clause| tidy(&strip(clause)))
        .filter(|clause| has_word(clause))
        .collect::<Vec<_>>()
        .join("; ");

    let out = tidy(&joined);
    if has_word(&out) { out } else { String::new() }
}

/// Render every source a module manifest carries, in place, before it is handed to the frontend.
///
/// One walk over the whole tree rather than a call at each render site: a new pane that prints a
/// source it fetched from the manifest gets this for free, and cannot forget it.
pub fn publicize_specs(specs: &mut [ModuleSpec]) {
    for spec in specs.iter_mut() {
        spec.doc = public_source(&spec.doc);
        for arg in spec.args.iter_mut() {
            arg.default_source = public_source(&arg.default_source);
            for condition in arg.validity_conditions.iter_mut() {
                condition.source = public_source(&condition.source);
            }
            for guidance in arg.guidance.iter_mut() {
                guidance.source = public_source(&guidance.source);
            }
            if let Some(contract) = arg.porosity_output.as_mut() {
                contract.limiting_policy_source = public_source(&contract.limiting_policy_source);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every string a manifest can print, gathered the way the walk gathers them.
    fn shipping_sources() -> Vec<String> {
        let mut all = Vec::new();
        for spec in crate::modules::list_modules() {
            all.push(spec.doc.clone());
            for arg in &spec.args {
                all.push(arg.default_source.clone());
                all.extend(arg.validity_conditions.iter().map(|c| c.source.clone()));
                all.extend(arg.guidance.iter().map(|g| g.source.clone()));
                if let Some(contract) = &arg.porosity_output {
                    all.push(contract.limiting_policy_source.clone());
                }
            }
        }
        all.retain(|s| !s.is_empty());
        all
    }

    fn leaks(text: &str) -> bool {
        let p = patterns();
        p.internal_doc.is_match(text) || p.vendor_file.is_match(text) || p.rust_file.is_match(text)
    }

    /// Pinned from BOTH sides on purpose. A renderer that returned the empty string for everything
    /// would satisfy "no internal document reaches a reader" perfectly and be useless, so the
    /// surviving text is asserted too: a source that said something must still say something.
    #[test]
    fn no_shipping_source_shows_a_reader_a_file_they_do_not_have() {
        let mut leaked = Vec::new();
        let mut emptied = Vec::new();
        for raw in shipping_sources() {
            let shown = public_source(&raw);
            if leaks(&shown) {
                leaked.push(format!("{raw}\n  -> {shown}"));
            }
            if shown.trim().is_empty() {
                emptied.push(raw);
            }
        }
        assert!(leaked.is_empty(), "internal reference still reaches a reader:\n{}", leaked.join("\n"));
        assert!(emptied.is_empty(), "source rendered to nothing at all:\n{}", emptied.join("\n"));
    }

    /// The other half of the same contract: an attribution a petrophysicist can act on SURVIVES.
    /// Deleting the vendor's name along with the vendor's filename would be the concealment
    /// `CLAUDE.md` forbids, not the fix this module is for.
    #[test]
    fn a_published_citation_and_a_vendor_name_both_survive_the_rendering() {
        let cases = [
            ("Archie 1942 Trans. AIME 146:54-62 (Geolog sw_arch.info References block; docs/PRD_v2/12_saturation.md:470)", "Archie 1942"),
            ("Geolog V14 phi_den.info RHO_W DEFAULT 1000 k/m3; docs/PRD_v2/11_porosity.md §5.1", "Geolog"),
            ("IP basicloganalysis.htm fresh-water 1.0 gm/cc; docs/PRD_v2/11_porosity.md §5.1", "IP"),
            ("docs/PRD_v2/11_porosity.md §5 porosity limits and DEC-015", "porosity limits"),
        ];
        for (raw, expected) in cases {
            let shown = public_source(raw);
            assert!(shown.contains(expected), "{expected:?} did not survive: {raw:?} -> {shown:?}");
            assert!(!leaks(&shown), "still leaks: {raw:?} -> {shown:?}");
        }
    }

    /// The sentinel is matched by the frontend, not read as prose. Rewriting it would turn a
    /// deliberate absence into a source that names nothing, and the pane would stop saying so.
    #[test]
    fn the_absent_sentinel_is_never_rewritten() {
        assert_eq!(public_source(ABSENT_DEFAULT_SOURCE), ABSENT_DEFAULT_SOURCE);
        assert_eq!(public_source(""), "");
    }

    /// The offline guidebook is generated by Node and cannot call this module, so `gen-guidebook.mjs`
    /// restates these rules. Two renderings of one citation are two chances to describe a source
    /// differently, and a reader comparing the manual against the pane would have no way to tell
    /// which was right - so the drift is a build failure, the arrangement `FACIES_PALETTE` already
    /// uses for the screen and the print.
    #[test]
    fn the_book_and_the_app_render_a_source_the_same_way() {
        let generator = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../tools/gen-guidebook.mjs");
        let js = std::fs::read_to_string(&generator)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", generator.display()));

        for (from, to) in OVERRIDES {
            assert!(js.contains(from), "the book's override table is missing a raw source:\n{from}");
            assert!(js.contains(to), "the book's override table is missing a rendering:\n{to}");
        }
        // The two load-bearing rules, pinned by their distinctive fragments: the reference tail that
        // takes section and line numbers with the path, and the vendor-file extensions.
        for fragment in [r"line\\s+\\d+\\b", "(?:lls|info|htm|html|chm)"] {
            assert!(js.contains(fragment), "the book no longer applies the rule {fragment:?}");
        }
    }

    /// An override is keyed on the exact raw string, so an edit upstream would silently drop it back
    /// to the mechanical rendering. This is what makes that loud instead.
    #[test]
    fn every_override_still_matches_a_shipping_source() {
        let shipping = shipping_sources();
        let orphans: Vec<_> = OVERRIDES
            .iter()
            .map(|(from, _)| *from)
            .filter(|from| !shipping.iter().any(|raw| raw == from))
            .collect();
        assert!(
            orphans.is_empty(),
            "override no longer matches any shipping source (edit it or delete it):\n{}",
            orphans.join("\n")
        );
    }
}
