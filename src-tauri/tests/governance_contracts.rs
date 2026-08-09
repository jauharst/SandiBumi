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
