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

/// How many §4c audit findings the CODE cites by number while the backlog still shows them open.
///
/// A ratchet, for the reason the guidebook's em-dash ceiling is one: a sweep leaves nothing behind
/// that the next session has to pass. Measured 2026-09-01 after verifying findings 1-28, 63 and 64
/// item by item - 27 of those 30 were closed and had never been ticked. The rest of the section
/// (the structure and slop blocks, findings 29-48 and 49-85) has NOT had that pass, which is what
/// this number counts. Verify a block, tick what it closed, lower the constant.
const AUDIT_CITED_BUT_UNTICKED_CEILING: usize = 42;

fn find_from(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    let first = needle[0];
    (from..=hay.len() - needle.len())
        .find(|&i| hay[i] == first && &hay[i..i + needle.len()] == needle)
}

/// The number after a `finding` token: an optional plural, then spaces or a `#`, then digits.
fn finding_number_at(hay: &[u8], mut i: usize) -> Option<u32> {
    if hay.get(i) == Some(&b's') {
        i += 1;
    }
    while matches!(hay.get(i), Some(b' ') | Some(b'#')) {
        i += 1;
    }
    let start = i;
    while hay.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
    }
    if i == start {
        return None;
    }
    std::str::from_utf8(&hay[start..i]).ok()?.parse().ok()
}

/// Finding numbers this text cites IN THE AUDIT'S OWN REGISTER.
///
/// The anchor matters: `docs/review_triage.md` numbers its findings too, and a bare "finding 16"
/// in the tree is that document's, not this one's - so an unanchored scan reported findings 16, 20
/// and 21 as closed when 16 is the one item of the verified block that is genuinely still open.
fn audit_findings_cited(text: &str) -> std::collections::BTreeSet<u32> {
    let lower = text.to_ascii_lowercase();
    let hay = lower.as_bytes();
    let mut cited = std::collections::BTreeSet::new();

    // Form 1: the audit named, then `finding N` close behind it.
    let anchor = b"audit-2026-08-20";
    let mut from = 0usize;
    while let Some(at) = find_from(hay, anchor, from) {
        let window = (at + 76).min(hay.len());
        if let Some(f) = find_from(&hay[..window], b"finding", at) {
            if let Some(n) = finding_number_at(hay, f + "finding".len()) {
                cited.insert(n);
            }
        }
        from = at + anchor.len();
    }

    // Form 2: `audit finding N`, where the register is named without its date.
    let phrase = b"audit finding";
    let mut from = 0usize;
    while let Some(at) = find_from(hay, phrase, from) {
        if let Some(n) = finding_number_at(hay, at + phrase.len()) {
            cited.insert(n);
        }
        from = at + phrase.len();
    }
    cited
}

fn source_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            source_files(&path, out);
        } else if matches!(path.extension().and_then(|e| e.to_str()), Some("rs") | Some("ts")) {
            out.push(path);
        }
    }
}

/// A finding the code says it closed must not still read as open in the backlog.
///
/// Two consecutive picks off §4c on 2026-09-01 turned out to be work already done - the fixes had
/// landed under their own decision ids (DEC-084/-085/-089/-090/-094/-096, the units family), each
/// ticking ITS own entry, and each citing the audit finding faithfully in the code. Nothing carried
/// that citation back to the checkbox, so the record of what is DONE lived in the source while the
/// record of what is LEFT lived in ROADMAP, and only one of the two was maintained. A stale backlog
/// is worse than no backlog: every plan drawn from it is wrong, and the wasted pick is paid by
/// whoever picks next.
#[test]
fn a_finding_the_code_says_it_closed_is_ticked_in_the_backlog() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits above src-tauri")
        .to_path_buf();

    let roadmap = std::fs::read_to_string(root.join("ROADMAP.md")).expect("read ROADMAP.md");
    let start = roadmap.find("## B1c.").expect("the whole-code audit backlog section");
    let after = &roadmap[start + "## B1c.".len()..];
    let end = after
        .find("\n## ")
        .map_or(roadmap.len(), |i| start + "## B1c.".len() + i);

    // Unticked finding numbers. Every `#N` inside the item's bold marker counts, because the P3
    // long tail bundles a dozen findings onto one line.
    let mut unticked = std::collections::BTreeSet::new();
    for line in roadmap[start..end].lines() {
        let Some(rest) = line.trim_start().strip_prefix("- [ ] **") else {
            continue;
        };
        let marker = rest.split("**").next().unwrap_or("");
        for piece in marker.split('#').skip(1) {
            let digits: String = piece.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<u32>() {
                unticked.insert(n);
            }
        }
    }

    let mut files = Vec::new();
    source_files(&root.join("src-tauri").join("src"), &mut files);
    source_files(&root.join("src"), &mut files);
    assert!(files.len() > 50, "the source scan found only {} files", files.len());

    let mut cited = std::collections::BTreeSet::new();
    for path in &files {
        if let Ok(text) = std::fs::read_to_string(path) {
            cited.extend(audit_findings_cited(&text));
        }
    }

    let stale: Vec<u32> = cited.intersection(&unticked).copied().collect();
    let listing = stale.iter().map(u32::to_string).collect::<Vec<_>>().join(", ");

    assert!(
        stale.len() <= AUDIT_CITED_BUT_UNTICKED_CEILING,
        "{} audit findings are cited by number in the code and still show as open in ROADMAP section B1c, above the ratchet of {}. Read the citation, and if the finding is closed tick its box with the file:line as evidence; if the code merely mentions it as context for another decision, say so on the ROADMAP line. Findings: {}",
        stale.len(),
        AUDIT_CITED_BUT_UNTICKED_CEILING,
        listing
    );
}
