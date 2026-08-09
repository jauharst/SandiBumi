//! Installation and release qualification contracts.
//!
//! A local bundle is not a qualified release. The release lane records observations made on
//! the actual signed artifact and clean target machine, then this module refuses publication
//! unless those observations agree with the executable identity compiled from this tree.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

const CAPABILITY_MANIFEST_JSON: &str = include_str!("../resources/install/capabilities.json");

pub const CAPABILITY_PYTHON_EQUATIONS: &str = "python_equations";
pub const CAPABILITY_DLIS_IMPORT: &str = "dlis_import";
pub const CAPABILITY_PLATE_EXTRACTION: &str = "spreadsheet_plate_extraction";
pub const CAPABILITY_WORKBOOK_EXPORT: &str = "workbook_export";
pub const CAPABILITY_DOCUMENT_EXPORT: &str = "document_export";
pub const CAPABILITY_DECK_EXPORT: &str = "deck_export";

/// One generic package probe for every Python-backed capability. Package names arrive as JSON
/// from the capability manifest; this runner owns no second inventory. Its stdin is bytes by
/// contract so non-ASCII paths survive the Windows ANSI-codepage boundary.
const PYTHON_PACKAGE_PROBE: &str = r#"
import importlib
import importlib.metadata
import json
import sys

request = json.loads(sys.stdin.buffer.read().decode("utf-8"))
packages = []
for item in request["packages"]:
    available = False
    version = None
    error = None
    try:
        module = importlib.import_module(item["import_name"])
        available = True
        try:
            version = importlib.metadata.version(item["distribution"])
        except Exception:
            observed = getattr(module, "__version__", None)
            version = str(observed) if observed is not None else None
    except Exception as exc:
        error = str(exc)
    packages.append({
        "distribution": item["distribution"],
        "import_name": item["import_name"],
        "available": available,
        "version": version,
        "error": error,
    })

reply = {
    "executable": sys.executable,
    "python_version": ".".join(str(part) for part in sys.version_info[:3]),
    "packages": packages,
}
sys.stdout.buffer.write(json.dumps(reply, ensure_ascii=False).encode("utf-8"))
"#;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InterpreterRequirement {
    pub id: String,
    pub execution: String,
    pub minimum_version: String,
    pub selection_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageRequirement {
    pub distribution: String,
    pub import_name: String,
    pub required: bool,
    /// Deliberately `None` in the source manifest. The deployment-owner decision says exact
    /// versions come from the SandiBumi-qualified release lock; no plausible number is allowed
    /// to leak in here before qualification produces it.
    pub minimum_supported_version: Option<String>,
    pub version_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityRequirement {
    pub id: String,
    pub display_name: String,
    pub owning_domain: String,
    pub interpreter: String,
    pub offline_route: String,
    pub packages: Vec<PackageRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityManifest {
    pub schema_version: u32,
    pub interpreter: InterpreterRequirement,
    pub capabilities: Vec<CapabilityRequirement>,
}

pub fn capability_manifest() -> Result<CapabilityManifest, String> {
    let manifest: CapabilityManifest = serde_json::from_str(CAPABILITY_MANIFEST_JSON)
        .map_err(|e| format!("bundled capability manifest is invalid: {e}"))?;
    validate_capability_manifest(&manifest)?;
    Ok(manifest)
}

fn validate_capability_manifest(manifest: &CapabilityManifest) -> Result<(), String> {
    let mut errors = Vec::new();
    if manifest.schema_version == 0 {
        errors.push("schema_version must be non-zero".to_string());
    }
    if manifest.interpreter.id.trim().is_empty()
        || manifest.interpreter.execution != "subprocess"
        || manifest.interpreter.minimum_version.trim().is_empty()
        || manifest.interpreter.selection_scope != "session"
    {
        errors.push(
            "interpreter must name a session-scoped subprocess and its cited minimum version"
                .to_string(),
        );
    }

    let mut capability_ids = BTreeSet::new();
    for capability in &manifest.capabilities {
        if capability.id.trim().is_empty()
            || capability.display_name.trim().is_empty()
            || capability.owning_domain.trim().is_empty()
        {
            errors
                .push("every capability needs a non-empty id, display name and owner".to_string());
        }
        if !capability_ids.insert(capability.id.as_str()) {
            errors.push(format!("duplicate capability id {}", capability.id));
        }
        if capability.interpreter != manifest.interpreter.id {
            errors.push(format!(
                "capability {} names interpreter {}, expected {}",
                capability.id, capability.interpreter, manifest.interpreter.id
            ));
        }
        if capability.offline_route.trim().is_empty() {
            errors.push(format!(
                "capability {} has no offline availability route",
                capability.id
            ));
        }
        if capability.packages.is_empty() {
            errors.push(format!(
                "capability {} has no package mapping",
                capability.id
            ));
        }
        let mut distributions = BTreeSet::new();
        for package in &capability.packages {
            if package.distribution.trim().is_empty() || package.import_name.trim().is_empty() {
                errors.push(format!(
                    "capability {} has an empty package key",
                    capability.id
                ));
            }
            if !distributions.insert(package.distribution.to_ascii_lowercase()) {
                errors.push(format!(
                    "capability {} repeats package {}",
                    capability.id, package.distribution
                ));
            }
            match package.minimum_supported_version.as_deref() {
                Some(version) if version.trim().is_empty() => errors.push(format!(
                    "capability {} package {} has an empty minimum version",
                    capability.id, package.distribution
                )),
                None if package.version_source != "qualified_release_lock" => errors.push(format!(
                    "capability {} package {} has neither a cited minimum version nor the qualified release-lock source",
                    capability.id, package.distribution
                )),
                _ => {}
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("capability manifest: {}", errors.join("; ")))
    }
}

pub fn capability_requirement(id: &str) -> Result<CapabilityRequirement, String> {
    capability_manifest()?
        .capabilities
        .into_iter()
        .find(|capability| capability.id == id)
        .ok_or_else(|| format!("capability manifest has no {id}"))
}

pub fn package_requirement(distribution: &str) -> Result<PackageRequirement, String> {
    capability_manifest()?
        .capabilities
        .into_iter()
        .flat_map(|capability| capability.packages)
        .find(|package| package.distribution.eq_ignore_ascii_case(distribution))
        .ok_or_else(|| format!("capability manifest has no package {distribution}"))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageProbe {
    pub distribution: String,
    pub import_name: String,
    pub available: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PythonPackageProbe {
    pub executable: String,
    pub python_version: String,
    pub packages: Vec<PackageProbe>,
}

fn probe_requirements(
    python: &Path,
    requirements: &[PackageRequirement],
) -> Result<PythonPackageProbe, String> {
    let mut unique = BTreeMap::<String, &PackageRequirement>::new();
    for package in requirements {
        unique
            .entry(package.distribution.to_ascii_lowercase())
            .or_insert(package);
    }
    let request = serde_json::json!({
        "packages": unique.values().map(|package| serde_json::json!({
            "distribution": package.distribution,
            "import_name": package.import_name,
        })).collect::<Vec<_>>(),
    });

    let mut command = Command::new(python);
    command
        .args(["-c", PYTHON_PACKAGE_PROBE])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    crate::python_engine::hide_console(&mut command);
    let mut child = command.spawn().map_err(|e| {
        format!(
            "cannot start Python package probe {}: {e}",
            python.display()
        )
    })?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| format!("Python package probe {} has no stdin", python.display()))?;
        let bytes = serde_json::to_vec(&request)
            .map_err(|e| format!("cannot encode Python package probe: {e}"))?;
        stdin
            .write_all(&bytes)
            .map_err(|e| format!("cannot write Python package probe: {e}"))?;
    }
    drop(child.stdin.take());
    let output = child
        .wait_with_output()
        .map_err(|e| format!("Python package probe {} failed: {e}", python.display()))?;
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "Python package probe {} exited {}{}",
            python.display(),
            output.status,
            if error.is_empty() {
                String::new()
            } else {
                format!(": {error}")
            }
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| {
        format!(
            "Python package probe {} returned invalid JSON: {e}",
            python.display()
        )
    })
}

pub fn probe_python_capability(
    python: &Path,
    capability_id: &str,
) -> Result<PythonPackageProbe, String> {
    let capability = capability_requirement(capability_id)?;
    probe_requirements(python, &capability.packages)
}

pub fn probe_all_python_packages(python: &Path) -> Result<PythonPackageProbe, String> {
    let requirements = capability_manifest()?
        .capabilities
        .into_iter()
        .flat_map(|capability| capability.packages)
        .collect::<Vec<_>>();
    probe_requirements(python, &requirements)
}

pub fn probe_manifest_package(
    python: &Path,
    distribution: &str,
) -> Result<PythonPackageProbe, String> {
    probe_requirements(python, &[package_requirement(distribution)?])
}

pub fn package_is_available(probe: &PythonPackageProbe, distribution: &str) -> bool {
    probe
        .packages
        .iter()
        .any(|package| package.distribution.eq_ignore_ascii_case(distribution) && package.available)
}

pub fn capability_is_available(
    probe: &PythonPackageProbe,
    capability_id: &str,
) -> Result<bool, String> {
    let capability = capability_requirement(capability_id)?;
    Ok(capability
        .packages
        .iter()
        .filter(|package| package.required)
        .all(|package| package_is_available(probe, &package.distribution)))
}

pub fn require_python_capability(
    python: &Path,
    capability_id: &str,
) -> Result<PythonPackageProbe, String> {
    let probe = probe_python_capability(python, capability_id)?;
    if capability_is_available(&probe, capability_id)? {
        Ok(probe)
    } else {
        Err(capability_message(
            capability_id,
            Some(python),
            Some(&probe),
        ))
    }
}

pub fn package_remediation(distribution: &str, python: Option<&Path>) -> String {
    let package = package_requirement(distribution);
    match (package, python) {
        (Ok(package), Some(path)) => format!(
            "{} is unavailable in {}. Run: \"{}\" -m pip install {} then re-probe, or repair the qualified offline Python pack.",
            package.distribution,
            path.display(),
            path.display(),
            package.distribution
        ),
        (Ok(package), None) => format!(
            "{} is unavailable because no session Python is selected; install or repair the qualified offline Python pack, then re-probe.",
            package.distribution
        ),
        (Err(error), _) => error,
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PackageRuntimeSupport {
    pub distribution: String,
    pub selected_interpreter: Option<String>,
    pub available: bool,
    pub version: Option<String>,
    pub message: String,
}

pub fn package_runtime_support(
    distribution: &str,
    python: Option<&Path>,
) -> Result<PackageRuntimeSupport, String> {
    let requirement = package_requirement(distribution)?;
    let Some(path) = python else {
        return Ok(PackageRuntimeSupport {
            distribution: requirement.distribution.clone(),
            selected_interpreter: None,
            available: false,
            version: None,
            message: package_remediation(&requirement.distribution, None),
        });
    };
    let probe = probe_manifest_package(path, &requirement.distribution)?;
    let observed = probe.packages.iter().find(|package| {
        package
            .distribution
            .eq_ignore_ascii_case(&requirement.distribution)
    });
    let available = observed.is_some_and(|package| package.available);
    Ok(PackageRuntimeSupport {
        distribution: requirement.distribution.clone(),
        selected_interpreter: Some(path.to_string_lossy().into_owned()),
        available,
        version: observed.and_then(|package| package.version.clone()),
        message: if available {
            format!(
                "{} is available in {}",
                requirement.distribution,
                path.display()
            )
        } else {
            package_remediation(&requirement.distribution, Some(path))
        },
    })
}

pub fn capability_message(
    capability_id: &str,
    python: Option<&Path>,
    probe: Option<&PythonPackageProbe>,
) -> String {
    let Ok(capability) = capability_requirement(capability_id) else {
        return format!("capability manifest has no {capability_id}");
    };
    let missing = capability
        .packages
        .iter()
        .filter(|package| {
            package.required
                && probe
                    .map(|observed| !package_is_available(observed, &package.distribution))
                    .unwrap_or(true)
        })
        .map(|package| package.distribution.as_str())
        .collect::<Vec<_>>();
    let package_names = if missing.is_empty() {
        capability
            .packages
            .iter()
            .filter(|package| package.required)
            .map(|package| package.distribution.as_str())
            .collect::<Vec<_>>()
    } else {
        missing
    };
    match python {
        Some(path) => format!(
            "{} is unavailable in {}: missing {}. Repair the qualified offline Python pack, then re-probe.",
            capability.display_name,
            path.display(),
            package_names.join(", ")
        ),
        None => format!(
            "{} is unavailable: no session-resolved Python {}+ interpreter with {}. Install or repair the qualified offline Python pack, or set {} to an approved interpreter, then re-probe.",
            capability.display_name,
            capability_manifest()
                .map(|manifest| manifest.interpreter.minimum_version)
                .unwrap_or_else(|_| "the manifest-declared minimum".to_string()),
            package_names.join(", "),
            crate::python_engine::PYTHON_ENV
        ),
    }
}

pub fn capability_status_message(
    capability_id: &str,
    python: Option<&Path>,
    probe: Option<&PythonPackageProbe>,
) -> String {
    let Some(path) = python else {
        return capability_message(capability_id, None, probe);
    };
    let Some(observed) = probe else {
        return capability_message(capability_id, Some(path), None);
    };
    if !capability_is_available(observed, capability_id).unwrap_or(false) {
        return capability_message(capability_id, Some(path), Some(observed));
    }
    let Ok(capability) = capability_requirement(capability_id) else {
        return format!("capability manifest has no {capability_id}");
    };
    let optional_missing = capability
        .packages
        .iter()
        .filter(|package| {
            !package.required && !package_is_available(observed, &package.distribution)
        })
        .map(|package| package.distribution.as_str())
        .collect::<Vec<_>>();
    if optional_missing.is_empty() {
        format!(
            "{} is available in {}",
            capability.display_name,
            path.display()
        )
    } else {
        format!(
            "{} is available in {}; optional package unavailable: {}",
            capability.display_name,
            path.display(),
            optional_missing.join(", ")
        )
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilitySupport {
    pub id: String,
    pub display_name: String,
    pub owning_domain: String,
    pub packages: Vec<PackageRequirement>,
    /// `false` means known unavailable; `None` means the interpreter exists but package probes
    /// have not yet supplied a truthful answer. It is never optimistically `true`.
    pub available: Option<bool>,
    pub reason: String,
    pub package_status: Vec<PackageProbe>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InstallationSupport {
    pub manifest_schema_version: u32,
    pub interpreter_minimum_version: String,
    pub selected_interpreter: Option<String>,
    pub capabilities: Vec<CapabilitySupport>,
}

/// Build the in-app prerequisite view from the same manifest used by release copy and probes.
pub fn installation_support(
    selected_interpreter: Option<String>,
) -> Result<InstallationSupport, String> {
    let manifest = capability_manifest()?;
    let probe = selected_interpreter
        .as_deref()
        .map(Path::new)
        .map(probe_all_python_packages);
    let capabilities = manifest
        .capabilities
        .iter()
        .map(|capability| {
            let (available, reason, package_status) = match (&selected_interpreter, &probe) {
                (None, _) => (
                    Some(false),
                    format!(
                        "Unavailable: no session-resolved Python {}+ interpreter",
                        manifest.interpreter.minimum_version
                    ),
                    Vec::new(),
                ),
                (Some(path), Some(Ok(observed))) => {
                    let package_status = capability
                        .packages
                        .iter()
                        .filter_map(|requirement| {
                            observed.packages.iter().find(|package| {
                                package
                                    .distribution
                                    .eq_ignore_ascii_case(&requirement.distribution)
                            })
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    let available = capability_is_available(observed, &capability.id)?;
                    let reason = capability_status_message(
                        &capability.id,
                        Some(Path::new(path)),
                        Some(observed),
                    );
                    (Some(available), reason, package_status)
                }
                (Some(_), Some(Err(error))) => (
                    Some(false),
                    format!("Unavailable: package probe failed: {error}"),
                    Vec::new(),
                ),
                (Some(_), None) => unreachable!("a selected interpreter always has a probe"),
            };
            Ok(CapabilitySupport {
                id: capability.id.clone(),
                display_name: capability.display_name.clone(),
                owning_domain: capability.owning_domain.clone(),
                packages: capability.packages.clone(),
                available,
                reason,
                package_status,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(InstallationSupport {
        manifest_schema_version: manifest.schema_version,
        interpreter_minimum_version: manifest.interpreter.minimum_version,
        selected_interpreter,
        capabilities,
    })
}

fn package_clause(capability: &CapabilityRequirement) -> String {
    let required = capability
        .packages
        .iter()
        .filter(|package| package.required)
        .map(|package| package.distribution.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let optional = capability
        .packages
        .iter()
        .filter(|package| !package.required)
        .map(|package| package.distribution.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    if optional.is_empty() {
        format!("requires {required}")
    } else {
        format!("requires {required}; optional {optional}")
    }
}

fn capability_markdown_lines(manifest: &CapabilityManifest) -> String {
    manifest
        .capabilities
        .iter()
        .map(|capability| {
            format!(
                "- **{}** — {} (owner: `{}`).",
                capability.display_name,
                package_clause(capability),
                capability.owning_domain
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn readme_prerequisite_block() -> Result<String, String> {
    let manifest = capability_manifest()?;
    Ok(format!(
        "<!-- capability-prerequisites:start -->\n\
**Runtime prerequisites.** The native core, project open, plotting and native exports do not\n\
require Python. These optional capabilities use one session-resolved Python {}+ subprocess:\n\n\
{}\n\n\
Package versions are never guessed here: each release takes them from the SandiBumi-qualified\n\
offline Python pack lock. Open **Project → Help → Prerequisites** to see availability on this\n\
machine.\n\
<!-- capability-prerequisites:end -->",
        manifest.interpreter.minimum_version,
        capability_markdown_lines(&manifest)
    ))
}

pub fn release_prerequisite_markdown() -> Result<String, String> {
    let manifest = capability_manifest()?;
    Ok(format!(
        "# Capability prerequisites — generated release-note fragment\n\n\
The native core, project open, plotting and native exports do not require Python. The following\n\
optional capabilities use one session-resolved Python {}+ subprocess:\n\n\
{}\n\n\
The supported offline route is the separately signed, versioned SandiBumi-qualified Python pack.\n\
Exact package versions are supplied only by that release's qualification lock.\n",
        manifest.interpreter.minimum_version,
        capability_markdown_lines(&manifest)
    ))
}

pub fn installer_prerequisite_text() -> Result<String, String> {
    let manifest = capability_manifest()?;
    let lines = manifest
        .capabilities
        .iter()
        .map(|capability| {
            format!(
                "{}: {}.",
                capability.display_name,
                package_clause(capability)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!(
        "SandiBumi capability prerequisites\n\n\
Native core, project open, plotting and native exports do not require Python.\n\
Optional capabilities use one session-resolved Python {}+ subprocess:\n\n\
{}\n\n\
Offline deployment uses the separately signed, versioned SandiBumi-qualified Python pack.\n\
Exact package versions come only from the qualified release lock.\n",
        manifest.interpreter.minimum_version, lines
    ))
}

pub fn installer_long_description() -> String {
    "Native core runs without Python. Optional Python-backed capabilities use one session-resolved Python 3.10+ subprocess; their exact packages are listed in the bundled capability-prerequisites notice and versions come only from the qualified release lock.".to_string()
}

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

    fn generated_readme_block(readme: &str) -> &str {
        let start = readme
            .find("<!-- capability-prerequisites:start -->")
            .expect("README generated prerequisite block starts");
        let end_marker = "<!-- capability-prerequisites:end -->";
        let end = readme[start..]
            .find(end_marker)
            .map(|offset| start + offset + end_marker.len())
            .expect("README generated prerequisite block ends");
        &readme[start..end]
    }

    /// SB-INS-004 / SB-INS-T04. The interpreter minimum and exact package rows come from
    /// chapter section 5. The qualified release lock is the deployment-owner decision supplied
    /// 2026-08-09; package minimums remain absent until that qualification cites exact versions.
    #[test]
    fn each_optional_capability_maps_to_the_cited_packages_and_no_detector_carries_a_second_package_list(
    ) {
        let manifest = capability_manifest().expect("valid bundled capability manifest");
        assert_eq!(manifest.interpreter.execution, "subprocess");
        assert_eq!(manifest.interpreter.minimum_version, "3.10");
        assert_eq!(manifest.interpreter.selection_scope, "session");

        let expected = [
            (
                CAPABILITY_PYTHON_EQUATIONS,
                vec![("numpy", true), ("scipy", false)],
            ),
            (CAPABILITY_DLIS_IMPORT, vec![("dlisio", true)]),
            (
                CAPABILITY_PLATE_EXTRACTION,
                vec![("openpyxl", true), ("Pillow", true)],
            ),
            (CAPABILITY_WORKBOOK_EXPORT, vec![("xlsxwriter", true)]),
            (CAPABILITY_DOCUMENT_EXPORT, vec![("python-docx", true)]),
            (
                CAPABILITY_DECK_EXPORT,
                vec![("python-pptx", true), ("matplotlib", true)],
            ),
        ];
        assert_eq!(manifest.capabilities.len(), expected.len());
        for (capability_id, packages) in expected {
            let capability = capability_requirement(capability_id)
                .unwrap_or_else(|error| panic!("{capability_id}: {error}"));
            assert_eq!(capability.interpreter, manifest.interpreter.id);
            assert_eq!(capability.offline_route, "qualified_python_pack");
            assert_eq!(
                capability
                    .packages
                    .iter()
                    .map(|package| (package.distribution.as_str(), package.required))
                    .collect::<Vec<_>>(),
                packages
            );
            for package in &capability.packages {
                assert_eq!(package.minimum_supported_version, None);
                assert_eq!(package.version_source, "qualified_release_lock");
            }
        }

        let all_present = PythonPackageProbe {
            executable: "qualified-python.exe".to_string(),
            python_version: "3.10.0".to_string(),
            packages: manifest
                .capabilities
                .iter()
                .flat_map(|capability| &capability.packages)
                .map(|package| PackageProbe {
                    distribution: package.distribution.clone(),
                    import_name: package.import_name.clone(),
                    available: true,
                    version: None,
                    error: None,
                })
                .collect(),
        };
        for capability in &manifest.capabilities {
            assert!(capability_is_available(&all_present, &capability.id).unwrap());
        }

        let mut missing_optional = all_present.clone();
        missing_optional
            .packages
            .iter_mut()
            .find(|package| package.distribution == "scipy")
            .expect("SciPy manifest row")
            .available = false;
        assert!(capability_is_available(&missing_optional, CAPABILITY_PYTHON_EQUATIONS).unwrap());

        let mut missing_required = all_present;
        missing_required
            .packages
            .iter_mut()
            .find(|package| package.distribution == "numpy")
            .expect("NumPy manifest row")
            .available = false;
        assert!(!capability_is_available(&missing_required, CAPABILITY_PYTHON_EQUATIONS).unwrap());

        assert!(PYTHON_PACKAGE_PROBE.contains("sys.stdin.buffer"));
        let office = include_str!("office.rs");
        assert!(office.contains("probe_all_python_packages"));
        assert!(!office.contains("const SUPPORT_PROBE"));
        assert!(include_str!("python_engine.rs").contains("probe_python_capability"));
        assert!(include_str!("dlis.rs").contains("require_python_capability"));
        assert!(include_str!("images.rs").contains("probe_manifest_package"));
    }

    /// SB-INS-003 / SB-INS-T03. The capability inventory and package names come from
    /// chapter §5 / Finding INS-6. The native/Python boundary comes from SB-INS-002 and the
    /// offline-lock wording from the deployment-owner decision supplied 2026-08-09.
    #[test]
    fn a_machine_without_python_names_every_python_capability_unavailable_and_makes_no_blanket_runtime_claim(
    ) {
        let manifest = capability_manifest().unwrap();
        let support = installation_support(None).unwrap();
        assert_eq!(support.capabilities.len(), manifest.capabilities.len());
        for expected in &manifest.capabilities {
            let actual = support
                .capabilities
                .iter()
                .find(|capability| capability.id == expected.id)
                .unwrap_or_else(|| panic!("support omitted {}", expected.display_name));
            assert_eq!(actual.available, Some(false));
            assert!(actual.reason.contains("Unavailable"), "{}", actual.reason);
        }

        let readme = include_str!("../../README.md").replace("\r\n", "\n");
        assert_eq!(
            generated_readme_block(&readme),
            readme_prerequisite_block().unwrap()
        );
        assert!(
            !readme
                .to_ascii_lowercase()
                .contains("no external database or runtime dependencies"),
            "the divergent blanket claim from Finding INS-6 must not return"
        );
        assert_eq!(
            include_str!("../../docs/INSTALLATION_PREREQUISITES.md").replace("\r\n", "\n"),
            release_prerequisite_markdown().unwrap()
        );
        assert_eq!(
            include_str!("../resources/install/capability-prerequisites.txt").replace("\r\n", "\n"),
            installer_prerequisite_text().unwrap()
        );

        let tauri_config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(
            tauri_config["bundle"]["longDescription"].as_str(),
            Some(installer_long_description().as_str())
        );
        assert!(tauri_config["bundle"]["resources"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| {
                item.as_str() == Some("resources/install/capability-prerequisites.txt")
            })));

        let support_ui = include_str!("../../src/ui/installationSupportDialog.ts");
        assert!(support_ui.contains("installationSupport()"));
        assert!(include_str!("../../index.html").contains("installation-support-btn"));
    }

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
