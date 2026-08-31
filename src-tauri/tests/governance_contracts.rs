#[test]
fn characterizes_the_green_gate_as_machine_enforced_but_still_manually_invoked() {
    // CHARACTERIZATION — SB-CORE-042 says build, lint and test must run automatically
    // on every change. The assertions below pin today's PARTIAL machine gate and the
    // absence of a repository workflow; they do not claim manual invocation is compliant.
    let gate = include_str!("../../tools/check.ps1");
    assert!(gate.contains("generate-verification-matrix.mjs"));
    assert!(gate.contains("npm run test:frontend"));
    assert!(gate.contains("npm run build"));
    assert!(gate.contains("cargo test"));

    let workflows = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(".github")
        .join("workflows");
    assert!(
        !workflows.exists(),
        "the current PARTIAL state has no automatic per-change workflow"
    );
}

/// AUDIT-2026-08-20 finding 59. Stage 1 was twelve copy-pasted blocks that ALL failed as
/// "GATE FAILED at takeover ledger" - so a broken unit registry or a stale chart derivation
/// reported the wrong gate, on the project's own green gate. That is `needWell.ts`'s rule (refuse
/// BY NAME, with the fix stated) applied to the tooling.
///
/// Pinned from BOTH sides. A refusal that names its own gate is the fix; a loop that quietly
/// DROPS a gate is what a twelve-blocks-to-one-loop rewrite gets wrong, and it would leave the
/// gate greener than the repository.
#[test]
fn every_stage_one_gate_fails_under_its_own_name_and_none_of_them_went_missing() {
    let gate = include_str!("../../tools/check.ps1");

    // A - the failure carries the gate's own name, not a hardcoded stage label.
    assert!(
        gate.contains("if ($code -ne 0) { Fail $failed $code }"),
        "stage 1 must fail with the name of the gate that refused"
    );
    assert!(
        !gate.contains("Fail \"takeover ledger\""),
        "no gate may refuse under another gate's name"
    );

    // B - and every gate that was in the twelve blocks is still run. A rewrite that loses one
    // makes the gate report green over a check it never performed.
    for command in [
        "test:takeover-ledger",
        "check:takeover-ledger",
        "test:gate2-program",
        "check:gate2-program",
        "test:gate2-hygiene",
        "check:gate2-hygiene",
        "test:frontend-exports",
        "check:frontend-exports",
        "test:unit-registry",
        "check:unit-registry",
        "test:chart-derivation",
        "check:chart-derivation",
        "test:release-inventory",
        "test:generated-artifact",
        "gen-third-party-licenses.mjs",
    ] {
        assert!(
            gate.contains(command),
            "stage 1 no longer runs '{command}' - a dropped gate is a check that silently stopped"
        );
    }
}

#[test]
fn characterizes_the_tier_c_register_as_shipped_policy_with_asset_specific_design_around_routes() {
    // CHARACTERIZATION — SB-CORE-044 and 01_PRODUCT §5.8 require a maintained Tier-C
    // register and primary-sourced design-arounds for every similar capability. The
    // assertions pin the current repository policy and known-asset routes; they do not
    // claim exhaustive capability coverage beyond the maintained register.
    let policy = include_str!("../../docs/IP_PROVENANCE.md");
    assert!(policy.contains("**Maintenance rule:** any new asset derived from an external source gets a row here"));
    assert!(policy.contains("**C** | Material we believe is protected, or whose status is unclear"));
    assert!(policy.contains("**Blocked** — not used until cleared; tracked in the standing Tier-C register"));
    assert!(policy.contains("**Re-derived** from the publication, never transcribed from a vendor implementation"));
    assert!(policy.contains("Fallback options"));
    assert!(policy.contains("underlying primary publications and tool-physics papers"));
}

/// The guidebook's voice regressed twice because nothing measured it. A sweep on 2026-08-26 took
/// the 52 chapters then written from 324 em dashes to 12 and HELD; by 2026-08-28 the book carried
/// 295 again, and 274 of them (93%) sat in prose authored after that sweep. A second sweep cleaned
/// it; a third would have followed, because a sweep leaves nothing behind that a later chapter has
/// to pass.
///
/// So this is a RATCHET, not a style rule. It has no opinion on any individual dash - it cannot,
/// because it cannot tell a connector from a quotation, and the survivors below are exactly that
/// distinction: every one sits inside an `<i>"..."</i>` quote of the application, a
/// `<pre class="equations">` block of the app's own output, or a menu label the reader has to
/// match on screen ("TOC - Passey"). Rewriting one of those would make the quote WRONG, which is
/// the bug the second sweep had to fix in the Curve Catalog quotation. The count is therefore the
/// contract: adding a dash to PROSE fails the gate, and adding one inside a genuine new quotation
/// means raising this constant deliberately, in a diff a reviewer sees.
///
/// EN dashes are not counted. `0.87-0.97` and `1528.0-1544.0` are numeric ranges and correct
/// typography; there are 53 of them and they are not the voice this measures.
const GUIDEBOOK_EM_DASH_CEILING: usize = 35;

/// Both encodings count, and so does the numeric one. A gate that only reads the literal character
/// is walked around by typing the entity instead - which is not hypothetical: of the 42 found when
/// this was written, 9 were entities, and 7 of those were one chapter rewritten three days earlier.
fn em_dashes(text: &str) -> usize {
    text.matches('\u{2014}').count() + text.matches("&mdash;").count() + text.matches("&#8212;").count()
}

#[test]
fn the_guidebook_chapters_do_not_drift_back_into_the_em_dash_voice() {
    let chapters = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root is the crate's parent")
        .join("docs/guide/chapters");

    let mut per_chapter: Vec<(usize, String)> = Vec::new();
    let mut total = 0;
    for entry in std::fs::read_dir(&chapters).expect("read docs/guide/chapters") {
        let path = entry.expect("read a chapter entry").path();
        if path.extension().and_then(|value| value.to_str()) != Some("html") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read a UTF-8 chapter");
        let count = em_dashes(&text);
        total += count;
        if count > 0 {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("?")
                .to_string();
            per_chapter.push((count, name));
        }
    }

    // Sorted heaviest first, because the chapter that pushed the count over is the one to open.
    per_chapter.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    let listing = per_chapter
        .iter()
        .map(|(count, name)| format!("{count} {name}"))
        .collect::<Vec<_>>()
        .join(", ");

    assert!(
        total <= GUIDEBOOK_EM_DASH_CEILING,
        "the guidebook chapters carry {total} em dashes, above the ratchet of {GUIDEBOOK_EM_DASH_CEILING}. \
Recast the new ones as a colon, a full stop, or a bracketed clause - the guidebook's voice does not use \
the em dash as a connector. If a new one is inside a genuine quotation of the application, raise \
GUIDEBOOK_EM_DASH_CEILING in this file and say so in the commit. By chapter: {listing}"
    );
}
