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
