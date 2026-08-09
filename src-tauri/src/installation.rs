//! Installation and release qualification contracts.
//!
//! A local bundle is not a qualified release. The release lane records observations made on
//! the actual signed artifact and clean target machine, then this module refuses publication
//! unless those observations agree with the executable identity compiled from this tree.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Deployment-owner decision, 2026-08-09: IT deploys one MSI device-wide, under the system
/// context; ordinary users launch it afterwards. This is a product-policy value supplied by
/// the owner, not a package-manager default inferred by the code.
pub const WINDOWS_PACKAGE_TYPE: &str = "msi";
pub const WINDOWS_INSTALL_SCOPE: &str = "per_machine";
pub const WINDOWS_DEPLOYMENT_CONTEXT: &str = "system";
pub const WINDOWS_LAUNCH_CONTEXT: &str = "standard_user";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredIdentity {
    pub product_name: String,
    pub version: String,
    pub identifier: String,
}

/// Read the identity that Tauri actually compiles and bundles. Keeping this derived from
/// `tauri.conf.json` prevents a release checker from becoming a second identity manifest.
pub fn configured_identity() -> Result<ConfiguredIdentity, String> {
    let config: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json"))
        .map_err(|e| format!("tauri.conf.json is not valid JSON: {e}"))?;
    let required = |key: &str| {
        config[key]
            .as_str()
            .filter(|value| !value.trim().is_empty())
            .map(str::to_owned)
            .ok_or_else(|| format!("tauri.conf.json has no non-empty {key}"))
    };
    Ok(ConfiguredIdentity {
        product_name: required("productName")?,
        version: required("version")?,
        identifier: required("identifier")?,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureEvidence {
    /// Result of Windows Authenticode verification against the final MSI bytes.
    pub valid: bool,
    /// Certificate identity as observed by the qualification runner. No certificate value is
    /// hard-coded here: release/security owns which certificate is approved.
    pub certificate_thumbprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerQualification {
    pub installer_file: String,
    pub package_type: String,
    pub install_scope: String,
    pub deployment_context: String,
    pub installer_sha256: String,
    pub build_commit: String,
    pub signature: SignatureEvidence,
    pub developer_tools_absent: bool,
    pub supported_windows_target: bool,
    pub install_succeeded: bool,
    pub launch_succeeded: bool,
    pub launch_context: String,
    pub installed_product_name: String,
    pub installed_version: String,
    pub installed_identifier: String,
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Validate evidence captured from the final artifact and its clean-machine installation.
/// Every failure is release-blocking; no field is advisory.
pub fn validate_installer_qualification(evidence: &InstallerQualification) -> Result<(), String> {
    let identity = configured_identity()?;
    let mut errors = Vec::new();

    if evidence.installer_file.trim().is_empty()
        || !evidence
            .installer_file
            .to_ascii_lowercase()
            .ends_with(".msi")
    {
        errors.push("installer_file must name the qualified .msi artifact".to_string());
    }
    if evidence.package_type != WINDOWS_PACKAGE_TYPE {
        errors.push(format!(
            "package_type must be {WINDOWS_PACKAGE_TYPE}, got {}",
            evidence.package_type
        ));
    }
    if evidence.install_scope != WINDOWS_INSTALL_SCOPE {
        errors.push(format!(
            "install_scope must be {WINDOWS_INSTALL_SCOPE}, got {}",
            evidence.install_scope
        ));
    }
    if evidence.deployment_context != WINDOWS_DEPLOYMENT_CONTEXT {
        errors.push(format!(
            "deployment_context must be {WINDOWS_DEPLOYMENT_CONTEXT}, got {}",
            evidence.deployment_context
        ));
    }
    if evidence.launch_context != WINDOWS_LAUNCH_CONTEXT {
        errors.push(format!(
            "launch_context must be {WINDOWS_LAUNCH_CONTEXT}, got {}",
            evidence.launch_context
        ));
    }
    if !is_sha256(&evidence.installer_sha256) {
        errors.push("installer_sha256 must be the 64-hex digest of the final MSI".to_string());
    }
    if evidence.build_commit.trim().is_empty() {
        errors.push("build_commit is required build provenance".to_string());
    }
    if !evidence.signature.valid || evidence.signature.certificate_thumbprint.trim().is_empty() {
        errors.push(
            "the final MSI must have a valid Authenticode signature and certificate thumbprint"
                .to_string(),
        );
    }
    if !evidence.developer_tools_absent {
        errors.push(
            "qualification machine must not contain Rust or Node.js developer tools".to_string(),
        );
    }
    if !evidence.supported_windows_target {
        errors.push(
            "installation target has not passed the supported-Windows qualification".to_string(),
        );
    }
    if !evidence.install_succeeded || !evidence.launch_succeeded {
        errors.push(
            "clean-machine installation and standard-user launch must both succeed".to_string(),
        );
    }
    if evidence.installed_product_name != identity.product_name {
        errors.push(format!(
            "installed product name {} does not match {}",
            evidence.installed_product_name, identity.product_name
        ));
    }
    if evidence.installed_version != identity.version {
        errors.push(format!(
            "installed version {} does not match {}",
            evidence.installed_version, identity.version
        ));
    }
    if evidence.installed_identifier != identity.identifier {
        errors.push(format!(
            "installed identifier {} does not match {}",
            evidence.installed_identifier, identity.identifier
        ));
    }
    if identity.version != env!("CARGO_PKG_VERSION") {
        errors.push(format!(
            "tauri version {} does not match Cargo version {}",
            identity.version,
            env!("CARGO_PKG_VERSION")
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

/// Release-tool boundary. Text evidence deliberately uses the shared tolerant decoder: one
/// stray byte in a delivery must not make this one importer a UTF-8-only exception.
pub fn validate_installer_qualification_file(path: &Path) -> Result<(), String> {
    let text = crate::parsers::read_text_file(path).map_err(|e| {
        format!(
            "{}: cannot read installer qualification: {e}",
            path.display()
        )
    })?;
    let evidence: InstallerQualification = serde_json::from_str(&text).map_err(|e| {
        format!(
            "{}: invalid installer qualification JSON: {e}",
            path.display()
        )
    })?;
    validate_installer_qualification(&evidence).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn qualified_fixture() -> InstallerQualification {
        let identity = configured_identity().expect("configured identity");
        InstallerQualification {
            installer_file: "SandiBumi_qualified_x64.msi".to_string(),
            package_type: WINDOWS_PACKAGE_TYPE.to_string(),
            install_scope: WINDOWS_INSTALL_SCOPE.to_string(),
            deployment_context: WINDOWS_DEPLOYMENT_CONTEXT.to_string(),
            installer_sha256: "a".repeat(64),
            build_commit: "qualification-fixture-commit".to_string(),
            signature: SignatureEvidence {
                valid: true,
                certificate_thumbprint: "qualification-fixture-certificate".to_string(),
            },
            developer_tools_absent: true,
            supported_windows_target: true,
            install_succeeded: true,
            launch_succeeded: true,
            launch_context: WINDOWS_LAUNCH_CONTEXT.to_string(),
            installed_product_name: identity.product_name,
            installed_version: identity.version,
            installed_identifier: identity.identifier,
        }
    }

    /// SB-INS-001 / SB-INS-T01. Identity and version come from the cited Tauri manifest;
    /// MSI/per-machine/system deployment and standard-user launch are the deployment-owner
    /// decision supplied 2026-08-09. The test pins acceptance and refusal so a checker that
    /// merely sees a file called `.msi` cannot pass.
    #[test]
    fn a_signed_clean_machine_msi_must_match_the_installed_identity_and_version() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(config["bundle"]["targets"], serde_json::json!(["msi"]));

        let valid = qualified_fixture();
        validate_installer_qualification(&valid).expect("complete qualification passes");

        let mut wrong_identity = valid.clone();
        wrong_identity.installed_identifier.push_str(".wrong");
        let err = validate_installer_qualification(&wrong_identity).unwrap_err();
        assert!(err.contains("installed identifier"), "{err}");

        let mut unsigned = valid.clone();
        unsigned.signature.valid = false;
        let err = validate_installer_qualification(&unsigned).unwrap_err();
        assert!(err.contains("Authenticode"), "{err}");

        let mut per_user = valid;
        per_user.install_scope = "per_user".to_string();
        let err = validate_installer_qualification(&per_user).unwrap_err();
        assert!(err.contains("per_machine"), "{err}");
    }
}
