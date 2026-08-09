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
