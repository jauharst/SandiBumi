//! Installation and release qualification contracts.
//!
//! A local bundle is not a qualified release. The release lane records observations made on
//! the actual signed artifact and clean target machine, then this module refuses publication
//! unless those observations agree with the executable identity compiled from this tree.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
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
pub const QUALIFIED_PYTHON_PACK_ROUTE: &str = "qualified_python_pack";
pub const APPLICATION_LOCAL_RUNTIME_SCOPE: &str = "sandibumi_application_local";

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
        if capability.offline_route != QUALIFIED_PYTHON_PACK_ROUTE {
            errors.push(format!(
                "capability {} offline route must be {}",
                capability.id, QUALIFIED_PYTHON_PACK_ROUTE
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
    probe_python_capability_with(python, capability_id, probe_requirements)
}

pub(crate) fn probe_python_capability_with<F>(
    python: &Path,
    capability_id: &str,
    execute: F,
) -> Result<PythonPackageProbe, String>
where
    F: FnOnce(&Path, &[PackageRequirement]) -> Result<PythonPackageProbe, String>,
{
    let capability = capability_requirement(capability_id)?;
    execute(python, &capability.packages)
}

pub fn probe_all_python_packages(python: &Path) -> Result<PythonPackageProbe, String> {
    probe_all_python_packages_with(python, probe_requirements)
}

pub(crate) fn probe_all_python_packages_with<F>(
    python: &Path,
    execute: F,
) -> Result<PythonPackageProbe, String>
where
    F: FnOnce(&Path, &[PackageRequirement]) -> Result<PythonPackageProbe, String>,
{
    let requirements = capability_manifest()?
        .capabilities
        .into_iter()
        .flat_map(|capability| capability.packages)
        .collect::<Vec<_>>();
    execute(python, &requirements)
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
    pub selected_interpreter_rule: Option<String>,
    pub interpreter_candidates: Vec<crate::python_engine::PythonCandidateReport>,
    pub capabilities: Vec<CapabilitySupport>,
}

/// Build the in-app prerequisite view from the same manifest used by release copy and probes.
pub fn installation_support(
    resolution: crate::python_engine::PythonResolution,
) -> Result<InstallationSupport, String> {
    let manifest = capability_manifest()?;
    let selected_interpreter = resolution.selected_interpreter.clone();
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
        selected_interpreter_rule: resolution.selected_rule,
        interpreter_candidates: resolution.candidates,
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
Offline deployment has one supported route: IT silently deploys the separately signed, versioned\n\
SandiBumi-qualified Python pack per machine. The pack configures `SANDIBUMI_PYTHON` to its\n\
application-local interpreter; qualification blocks public network access. Exact package versions\n\
come only from that release's lock. Open **Project → Help → Prerequisites** for local status.\n\
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
The supported offline route is the separately signed, versioned SandiBumi-qualified Python pack,\n\
silently deployed per machine by IT. It configures `SANDIBUMI_PYTHON` to its application-local\n\
interpreter. The release gate blocks public network access and accepts zero observed requests.\n\
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
Offline deployment uses the separately signed, versioned SandiBumi-qualified Python pack, silently\n\
deployed per machine by IT. It configures SANDIBUMI_PYTHON to its application-local interpreter.\n\
Qualification blocks public network access and accepts zero observed requests. Exact package\n\
versions come only from the qualified release lock.\n",
        manifest.interpreter.minimum_version, lines
    ))
}

pub fn installer_long_description() -> String {
    "Native core runs without Python. Offline Python-backed capabilities require the separately signed SandiBumi-qualified pack, silently deployed per machine by IT; exact packages are listed in the bundled capability-prerequisites notice and versions come only from the qualified release lock.".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InstalledSettingsTemplate {
    pub template_version: String,
    /// No values are inferred from a mutable profile. This map is empty until a requirement
    /// supplies a cited product default.
    pub settings: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SettingsTemplateOrigin {
    pub template_version: String,
    pub template_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct UserSettingsDocument {
    pub origin: SettingsTemplateOrigin,
    pub settings: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SettingsMaterialization {
    pub user_path: String,
    pub origin: SettingsTemplateOrigin,
    pub created: bool,
}

fn sha256_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn read_user_settings(path: &Path) -> Result<UserSettingsDocument, String> {
    let text = crate::parsers::read_text_file(path)
        .map_err(|e| format!("{}: cannot read user settings: {e}", path.display()))?;
    let document: UserSettingsDocument = serde_json::from_str(&text)
        .map_err(|e| format!("{}: invalid user settings JSON: {e}", path.display()))?;
    if document.origin.template_version.trim().is_empty()
        || !is_sha256(&document.origin.template_sha256)
    {
        return Err(format!(
            "{}: user settings do not record a template version and SHA-256",
            path.display()
        ));
    }
    Ok(document)
}

/// First-run materialisation boundary. The installed resource is read and hashed but never opened
/// for writing; the user copy is created once with `create_new` outside the installation tree.
pub fn materialize_user_settings(
    installed_template: &Path,
    user_path: &Path,
) -> Result<SettingsMaterialization, String> {
    let installed_dir = installed_template
        .parent()
        .ok_or_else(|| "installed settings template has no parent directory".to_string())?;
    if user_path.starts_with(installed_dir) {
        return Err(format!(
            "writable user settings {} must be outside installation directory {}",
            user_path.display(),
            installed_dir.display()
        ));
    }

    if user_path.exists() {
        let existing = read_user_settings(user_path)?;
        return Ok(SettingsMaterialization {
            user_path: user_path.to_string_lossy().into_owned(),
            origin: existing.origin,
            created: false,
        });
    }

    let template_text = crate::parsers::read_text_file(installed_template).map_err(|e| {
        format!(
            "{}: cannot read installed settings template: {e}",
            installed_template.display()
        )
    })?;
    let template: InstalledSettingsTemplate =
        serde_json::from_str(&template_text).map_err(|e| {
            format!(
                "{}: invalid installed settings template JSON: {e}",
                installed_template.display()
            )
        })?;
    let identity = configured_identity()?;
    if template.template_version != identity.version {
        return Err(format!(
            "installed settings template version {} does not match application version {}",
            template.template_version, identity.version
        ));
    }
    let template_bytes = std::fs::read(installed_template).map_err(|e| {
        format!(
            "{}: cannot hash installed settings template: {e}",
            installed_template.display()
        )
    })?;
    let origin = SettingsTemplateOrigin {
        template_version: template.template_version,
        template_sha256: sha256_bytes(&template_bytes),
    };
    let user_document = UserSettingsDocument {
        origin: origin.clone(),
        settings: template.settings,
    };
    let bytes = serde_json::to_vec_pretty(&user_document)
        .map_err(|e| format!("cannot encode first-run user settings: {e}"))?;
    let parent = user_path
        .parent()
        .ok_or_else(|| "user settings path has no parent directory".to_string())?;
    std::fs::create_dir_all(parent).map_err(|e| {
        format!(
            "cannot create user configuration directory {}: {e}",
            parent.display()
        )
    })?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(user_path)
        .map_err(|e| format!("cannot create user settings {}: {e}", user_path.display()))?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| {
            format!(
                "cannot materialise user settings {}: {e}",
                user_path.display()
            )
        })?;

    Ok(SettingsMaterialization {
        user_path: user_path.to_string_lossy().into_owned(),
        origin,
        created: true,
    })
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
pub fn read_installer_qualification_file(path: &Path) -> Result<InstallerQualification, String> {
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
    Ok(evidence)
}

pub fn validate_installer_qualification_file(path: &Path) -> Result<(), String> {
    let evidence = read_installer_qualification_file(path)?;
    validate_installer_qualification(&evidence).map_err(|e| format!("{}: {e}", path.display()))
}

/// Evidence captured while both the application MSI and the separately signed Python pack are
/// installed on a clean machine whose public network path is blocked. Exact runtime/package
/// versions live in the referenced release lock; this schema deliberately carries no default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineDeploymentQualification {
    pub pack_file: String,
    pub pack_version: String,
    pub pack_sha256: String,
    pub release_lock_file: String,
    pub release_lock_sha256: String,
    pub signature: SignatureEvidence,
    pub deployment_route: String,
    pub install_scope: String,
    pub deployment_context: String,
    pub runtime_scope: String,
    pub application_msi_silent_install_succeeded: bool,
    pub pack_silent_install_succeeded: bool,
    pub public_network_blocked: bool,
    pub network_requests_observed: u64,
    pub network_monitor_evidence: String,
    pub selected_interpreter: String,
    pub selection_rule: String,
    pub claimed_capabilities: Vec<String>,
    pub probe: PythonPackageProbe,
}

/// Refuse release media unless the application installer is intrinsically offline and the
/// qualified pack proves every manifest capability while the public network is unavailable.
pub fn validate_offline_deployment(
    evidence: &OfflineDeploymentQualification,
) -> Result<(), String> {
    let manifest = capability_manifest()?;
    let config: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json"))
        .map_err(|e| format!("tauri.conf.json is invalid: {e}"))?;
    let mut errors = Vec::new();

    if config["bundle"]["windows"]["webviewInstallMode"]["type"].as_str()
        != Some("offlineInstaller")
    {
        errors.push(
            "application MSI must embed Tauri's offline WebView2 installer; a downloaded bootstrapper is not an offline route"
                .to_string(),
        );
    }
    if evidence.pack_file.trim().is_empty() || evidence.pack_version.trim().is_empty() {
        errors.push("qualified Python pack file and version are required".to_string());
    }
    if !is_sha256(&evidence.pack_sha256) {
        errors.push("pack_sha256 must be the 64-hex digest of the qualified pack".to_string());
    }
    if evidence.release_lock_file.trim().is_empty() || !is_sha256(&evidence.release_lock_sha256) {
        errors.push(
            "the exact-version release lock file and its 64-hex SHA-256 are required".to_string(),
        );
    }
    if !evidence.signature.valid || evidence.signature.certificate_thumbprint.trim().is_empty() {
        errors.push(
            "the qualified Python pack must have a valid signature and certificate thumbprint"
                .to_string(),
        );
    }
    if evidence.deployment_route != QUALIFIED_PYTHON_PACK_ROUTE {
        errors.push(format!(
            "deployment_route must be {QUALIFIED_PYTHON_PACK_ROUTE}"
        ));
    }
    if evidence.install_scope != WINDOWS_INSTALL_SCOPE
        || evidence.deployment_context != WINDOWS_DEPLOYMENT_CONTEXT
    {
        errors.push(format!(
            "the offline pack must be deployed {} by {}",
            WINDOWS_INSTALL_SCOPE, WINDOWS_DEPLOYMENT_CONTEXT
        ));
    }
    if evidence.runtime_scope != APPLICATION_LOCAL_RUNTIME_SCOPE {
        errors.push(format!(
            "runtime_scope must be {APPLICATION_LOCAL_RUNTIME_SCOPE}"
        ));
    }
    if !evidence.application_msi_silent_install_succeeded || !evidence.pack_silent_install_succeeded
    {
        errors.push(
            "both the application MSI and qualified Python pack must install silently".to_string(),
        );
    }
    if !evidence.public_network_blocked
        || evidence.network_requests_observed != 0
        || evidence.network_monitor_evidence.trim().is_empty()
    {
        errors.push(
            "offline qualification requires a blocked public network, zero observed network requests and named monitor evidence"
                .to_string(),
        );
    }
    if evidence.selected_interpreter.trim().is_empty()
        || evidence.probe.executable != evidence.selected_interpreter
    {
        errors.push(
            "the package probe must report the exact selected application-local interpreter"
                .to_string(),
        );
    }
    if evidence.selection_rule != crate::python_engine::PYTHON_OVERRIDE_RULE {
        errors.push(format!(
            "the offline pack must configure the existing {} resolver rule",
            crate::python_engine::PYTHON_OVERRIDE_RULE
        ));
    }
    if !crate::python_engine::version_at_least(
        &evidence.probe.python_version,
        &manifest.interpreter.minimum_version,
    ) {
        errors.push(format!(
            "qualified pack Python {} is below the cited minimum {}",
            evidence.probe.python_version, manifest.interpreter.minimum_version
        ));
    }

    let expected_capabilities = manifest
        .capabilities
        .iter()
        .map(|capability| capability.id.as_str())
        .collect::<BTreeSet<_>>();
    let claimed_capabilities = evidence
        .claimed_capabilities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if claimed_capabilities.len() != evidence.claimed_capabilities.len() {
        errors.push("offline feature set repeats a capability id".to_string());
    }
    for missing in expected_capabilities.difference(&claimed_capabilities) {
        errors.push(format!(
            "offline feature set omits manifest capability {missing}"
        ));
    }
    for unknown in claimed_capabilities.difference(&expected_capabilities) {
        errors.push(format!(
            "offline feature set claims unknown capability {unknown}"
        ));
    }

    for capability in &manifest.capabilities {
        for package in &capability.packages {
            if !evidence.probe.packages.iter().any(|observed| {
                observed
                    .distribution
                    .eq_ignore_ascii_case(&package.distribution)
            }) {
                errors.push(format!(
                    "offline probe omitted {} package {}",
                    capability.display_name, package.distribution
                ));
            }
        }
        if claimed_capabilities.contains(capability.id.as_str()) {
            match capability_is_available(&evidence.probe, &capability.id) {
                Ok(true) => {}
                Ok(false) => errors.push(capability_message(
                    &capability.id,
                    Some(Path::new(&evidence.selected_interpreter)),
                    Some(&evidence.probe),
                )),
                Err(error) => errors.push(error),
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

/// Release-tool boundary; all qualification JSON uses the repository's tolerant text decoder.
pub fn validate_offline_deployment_file(path: &Path) -> Result<(), String> {
    let text = crate::parsers::read_text_file(path).map_err(|e| {
        format!(
            "{}: cannot read offline deployment qualification: {e}",
            path.display()
        )
    })?;
    let evidence: OfflineDeploymentQualification = serde_json::from_str(&text).map_err(|e| {
        format!(
            "{}: invalid offline deployment qualification JSON: {e}",
            path.display()
        )
    })?;
    validate_offline_deployment(&evidence).map_err(|e| format!("{}: {e}", path.display()))
}

/// Deployment-owner decision, 2026-08-09: qualify every Microsoft-serviced Windows 11 x64
/// feature release for both Pro and Enterprise. Feature-release names are release evidence,
/// never a list frozen into the executable.
pub const WINDOWS_PRODUCT: &str = "Windows 11";
pub const WINDOWS_ARCHITECTURE: &str = "x64";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub enum WindowsEdition {
    Pro,
    Enterprise,
}

impl WindowsEdition {
    fn label(self) -> &'static str {
        match self {
            Self::Pro => "Pro",
            Self::Enterprise => "Enterprise",
        }
    }
}

pub const QUALIFIED_WINDOWS_EDITIONS: &[WindowsEdition] =
    &[WindowsEdition::Pro, WindowsEdition::Enterprise];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum CleanMachineScenario {
    StandardUser,
    LockedDownUser,
    OfflineInstall,
    NoPythonCoreUse,
    SupportedExternalPython,
    MissingPackage,
    Upgrade,
    Rollback,
    UninstallPreservation,
}

impl CleanMachineScenario {
    pub fn id(self) -> &'static str {
        match self {
            Self::StandardUser => "standard_user",
            Self::LockedDownUser => "locked_down_user",
            Self::OfflineInstall => "offline_install",
            Self::NoPythonCoreUse => "no_python_core_use",
            Self::SupportedExternalPython => "supported_external_python",
            Self::MissingPackage => "missing_package",
            Self::Upgrade => "upgrade",
            Self::Rollback => "rollback",
            Self::UninstallPreservation => "uninstall_preservation",
        }
    }
}

pub const CLEAN_MACHINE_SCENARIOS: &[CleanMachineScenario] = &[
    CleanMachineScenario::StandardUser,
    CleanMachineScenario::LockedDownUser,
    CleanMachineScenario::OfflineInstall,
    CleanMachineScenario::NoPythonCoreUse,
    CleanMachineScenario::SupportedExternalPython,
    CleanMachineScenario::MissingPackage,
    CleanMachineScenario::Upgrade,
    CleanMachineScenario::Rollback,
    CleanMachineScenario::UninstallPreservation,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftServicedWindowsTarget {
    pub feature_release: String,
    pub edition: WindowsEdition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftServicedWindowsInventory {
    pub product: String,
    pub architecture: String,
    /// Release-lane observation that binds the inventory to this release qualification.
    pub observed_at_release: String,
    /// Named Microsoft lifecycle source or preserved source snapshot.
    pub source: String,
    pub source_sha256: String,
    /// Populated from the source at release time. Edition is part of the target because Microsoft
    /// servicing can differ by edition; release names remain absent from compiled constants.
    pub targets: Vec<MicrosoftServicedWindowsTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanMachineScenarioResult {
    pub feature_release: String,
    pub edition: WindowsEdition,
    pub scenario: CleanMachineScenario,
    pub passed: bool,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanMachineQualification {
    pub installer_sha256: String,
    pub qualified_build_commit: String,
    pub microsoft_serviced_inventory: MicrosoftServicedWindowsInventory,
    pub results: Vec<CleanMachineScenarioResult>,
}

fn scenario_key(
    feature_release: &str,
    edition: WindowsEdition,
    scenario: CleanMachineScenario,
) -> (String, WindowsEdition, CleanMachineScenario) {
    (feature_release.to_string(), edition, scenario)
}

fn scenario_name(
    feature_release: &str,
    edition: WindowsEdition,
    scenario: CleanMachineScenario,
) -> String {
    format!(
        "{} {} {} / {}",
        WINDOWS_PRODUCT,
        feature_release,
        edition.label(),
        scenario.id()
    )
}

/// Refuse publication unless the exact installer is qualified across the complete matrix derived
/// from the release-time Microsoft-serviced inventory. No Windows feature release is assumed by
/// this executable; omissions are visible because the inventory-to-matrix cross-product is exact.
pub fn validate_clean_machine_qualification(
    evidence: &CleanMachineQualification,
    installer: &InstallerQualification,
) -> Result<(), String> {
    let mut errors = Vec::new();
    if let Err(error) = validate_installer_qualification(installer) {
        errors.push(format!("installer qualification failed: {error}"));
    }
    if evidence.installer_sha256 != installer.installer_sha256 {
        errors
            .push("clean-machine matrix does not name the qualified installer SHA-256".to_string());
    }
    if evidence.qualified_build_commit != installer.build_commit {
        errors.push("clean-machine matrix does not name the qualified build commit".to_string());
    }

    let inventory = &evidence.microsoft_serviced_inventory;
    if inventory.product != WINDOWS_PRODUCT {
        errors.push(format!(
            "serviced inventory product must be {WINDOWS_PRODUCT}"
        ));
    }
    if inventory.architecture != WINDOWS_ARCHITECTURE {
        errors.push(format!(
            "serviced inventory architecture must be {WINDOWS_ARCHITECTURE}"
        ));
    }
    if inventory.observed_at_release.trim().is_empty() {
        errors.push("serviced inventory needs a release-time observation".to_string());
    }
    if inventory.source.trim().is_empty() || !is_sha256(&inventory.source_sha256) {
        errors.push(
            "serviced inventory needs a named Microsoft source and its 64-hex SHA-256".to_string(),
        );
    }
    if inventory.targets.is_empty() {
        errors.push("serviced inventory contains no Windows 11 target".to_string());
    }

    let mut target_keys = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for target in &inventory.targets {
        let trimmed = target.feature_release.trim();
        if trimmed.is_empty() || trimmed != target.feature_release {
            errors.push("serviced feature-release names must be non-empty and trimmed".to_string());
            continue;
        }
        if !target_keys.insert((target.feature_release.to_ascii_lowercase(), target.edition)) {
            errors.push(format!(
                "serviced inventory repeats {} {} {}",
                WINDOWS_PRODUCT,
                target.feature_release,
                target.edition.label()
            ));
        }
        targets.insert((target.feature_release.as_str(), target.edition));
    }

    let mut expected = BTreeSet::new();
    for (feature_release, edition) in &targets {
        for scenario in CLEAN_MACHINE_SCENARIOS {
            expected.insert(scenario_key(feature_release, *edition, *scenario));
        }
    }

    let mut actual = BTreeSet::new();
    for result in &evidence.results {
        let key = scenario_key(&result.feature_release, result.edition, result.scenario);
        if !actual.insert(key.clone()) {
            errors.push(format!(
                "clean-machine matrix repeats {}",
                scenario_name(&result.feature_release, result.edition, result.scenario)
            ));
        }
        if result.evidence.trim().is_empty() {
            errors.push(format!(
                "clean-machine matrix has no evidence for {}",
                scenario_name(&result.feature_release, result.edition, result.scenario)
            ));
        }
        if !result.passed {
            errors.push(format!(
                "clean-machine scenario failed: {}",
                scenario_name(&result.feature_release, result.edition, result.scenario)
            ));
        }
    }

    for missing in expected.difference(&actual) {
        errors.push(format!(
            "clean-machine matrix is missing {}",
            scenario_name(&missing.0, missing.1, missing.2)
        ));
    }
    for extra in actual.difference(&expected) {
        errors.push(format!(
            "clean-machine matrix contains a target outside the serviced inventory: {}",
            scenario_name(&extra.0, extra.1, extra.2)
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "installer is not publishable: {}",
            errors.join("; ")
        ))
    }
}

/// Release-tool boundary; qualification JSON uses the shared tolerant text decoder.
pub fn validate_clean_machine_qualification_file(
    path: &Path,
    installer: &InstallerQualification,
) -> Result<(), String> {
    let text = crate::parsers::read_text_file(path).map_err(|e| {
        format!(
            "{}: cannot read clean-machine qualification: {e}",
            path.display()
        )
    })?;
    let evidence: CleanMachineQualification = serde_json::from_str(&text).map_err(|e| {
        format!(
            "{}: invalid clean-machine qualification JSON: {e}",
            path.display()
        )
    })?;
    validate_clean_machine_qualification(&evidence, installer)
        .map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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

    /// CORRECTNESS — SB-INS-002 / SB-INS-T02 states that no Python executable may
    /// disable only Python-backed capabilities while project open, native computation,
    /// plotting and native export remain usable. The tiny arrays are non-physical smoke
    /// fixtures; the sourced expected value is successful availability, not a scientific result.
    #[test]
    fn missing_python_does_not_block_project_open_native_computation_plotting_or_native_export() {
        let support = installation_support(crate::python_engine::PythonResolution {
            selected_interpreter: None,
            selected_rule: None,
            candidates: Vec::new(),
        })
        .expect("native launch support resolves without Python");
        assert!(support
            .capabilities
            .iter()
            .all(|capability| capability.available == Some(false)));

        let path = std::env::temp_dir().join(format!(
            "sandibumi-native-without-python-{}.sdb",
            uuid::Uuid::new_v4()
        ));
        let path_text = path.to_string_lossy().into_owned();
        let connection = crate::project::open_and_migrate(&path_text)
            .expect("an existing-project-capable native store opens without Python");

        let context = crate::modules::ModuleContext {
            n: 2,
            logs: std::collections::HashMap::from([
                ("DEPTH".to_string(), vec![0.0, 1.0]),
                ("CURVE".to_string(), vec![10.0, 20.0]),
            ]),
            params: std::collections::HashMap::from([("SHIFT".to_string(), vec![0.0, 0.0])]),
            opts: std::collections::HashMap::new(),
            depth_unit: crate::units::DepthUnit::Metres,
        };
        let shifted = crate::modules::run_module("depth_shift", &context)
            .expect("the native module dispatch remains available");
        assert_eq!(shifted["CURVE_DS"], [10.0, 20.0]);

        let histogram = crate::plotting::canonical_histogram(&[0.0, 1.0], 0.0, 1.0, 1);
        assert_eq!(histogram.displayed_total, 2);
        let formats = crate::export::export_formats();
        assert!(formats.iter().any(|format| format.is_default && format.extension == "las"));

        drop(connection);
        std::fs::remove_file(&path).expect("remove isolated native project fixture");
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
        let support = installation_support(crate::python_engine::PythonResolution {
            selected_interpreter: None,
            selected_rule: None,
            candidates: Vec::new(),
        })
        .unwrap();
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

    /// SB-INS-010 / SB-INS-T12. The immutable-template/user-copy split and provenance fields
    /// come from dossier section 2.6; the template version is the cited application version in
    /// tauri.conf.json. The settings map stays empty because no factory values are cited.
    #[test]
    fn first_run_materialises_a_user_copy_with_the_immutable_template_version_and_digest() {
        let installed_template = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("install")
            .join("settings-template.json");
        let installed_before = std::fs::read(&installed_template).expect("installed template");
        let installed_digest = sha256_bytes(&installed_before);
        let temp = std::env::temp_dir().join(format!(
            "sandibumi-settings-materialisation-{}",
            uuid::Uuid::new_v4()
        ));
        let user_path = temp.join("config").join("settings.json");

        let first = materialize_user_settings(&installed_template, &user_path)
            .expect("first run materialises settings");
        assert!(first.created);
        assert_eq!(first.origin.template_sha256, installed_digest);
        assert_eq!(
            first.origin.template_version,
            configured_identity().unwrap().version
        );

        let text = crate::parsers::read_text_file(&user_path).expect("user settings text");
        let mut user: UserSettingsDocument = serde_json::from_str(&text).unwrap();
        assert!(
            user.settings.is_empty(),
            "uncited defaults must stay absent"
        );
        assert_eq!(user.origin, first.origin);
        user.settings.insert(
            "test-only-user-edit".to_string(),
            serde_json::Value::Bool(true),
        );
        std::fs::write(&user_path, serde_json::to_vec_pretty(&user).unwrap())
            .expect("write user edit");

        let second = materialize_user_settings(&installed_template, &user_path)
            .expect("later launch preserves user settings");
        assert!(!second.created);
        assert_eq!(second.origin, first.origin);
        let edited_text =
            crate::parsers::read_text_file(&user_path).expect("edited user settings text");
        let edited: UserSettingsDocument = serde_json::from_str(&edited_text).unwrap();
        assert_eq!(
            edited.settings.get("test-only-user-edit"),
            Some(&serde_json::Value::Bool(true))
        );
        assert_eq!(
            std::fs::read(&installed_template).unwrap(),
            installed_before
        );
        assert_eq!(
            sha256_bytes(&std::fs::read(&installed_template).unwrap()),
            installed_digest
        );

        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert!(config["bundle"]["resources"]
            .as_array()
            .is_some_and(|resources| resources.iter().any(|resource| {
                resource.as_str() == Some("resources/install/settings-template.json")
            })));

        std::fs::remove_dir_all(&temp).expect("remove isolated settings fixture");
    }

    fn offline_qualification_fixture() -> OfflineDeploymentQualification {
        let manifest = capability_manifest().expect("capability manifest");
        OfflineDeploymentQualification {
            pack_file: "qualified-python-pack".to_string(),
            pack_version: "version-from-release-lock".to_string(),
            pack_sha256: "b".repeat(64),
            release_lock_file: "qualified-release-lock.json".to_string(),
            release_lock_sha256: "c".repeat(64),
            signature: SignatureEvidence {
                valid: true,
                certificate_thumbprint: "qualification-fixture-certificate".to_string(),
            },
            deployment_route: QUALIFIED_PYTHON_PACK_ROUTE.to_string(),
            install_scope: WINDOWS_INSTALL_SCOPE.to_string(),
            deployment_context: WINDOWS_DEPLOYMENT_CONTEXT.to_string(),
            runtime_scope: APPLICATION_LOCAL_RUNTIME_SCOPE.to_string(),
            application_msi_silent_install_succeeded: true,
            pack_silent_install_succeeded: true,
            public_network_blocked: true,
            network_requests_observed: 0,
            network_monitor_evidence: "clean-machine network trace".to_string(),
            selected_interpreter: "C:/Program Files/SandiBumi Runtime/python.exe".to_string(),
            selection_rule: crate::python_engine::PYTHON_OVERRIDE_RULE.to_string(),
            claimed_capabilities: manifest
                .capabilities
                .iter()
                .map(|capability| capability.id.clone())
                .collect(),
            probe: PythonPackageProbe {
                executable: "C:/Program Files/SandiBumi Runtime/python.exe".to_string(),
                python_version: "3.10.0".to_string(),
                packages: manifest
                    .capabilities
                    .iter()
                    .flat_map(|capability| &capability.packages)
                    .map(|package| PackageProbe {
                        distribution: package.distribution.clone(),
                        import_name: package.import_name.clone(),
                        available: package.distribution != "scipy",
                        version: None,
                        error: (package.distribution == "scipy")
                            .then(|| "optional package absent".to_string()),
                    })
                    .collect(),
            },
        }
    }

    /// SB-INS-008 / SB-INS-T10. Zero network use and every manifest capability come from T10;
    /// Tauri documents `offlineInstaller` as its no-network WebView2 mode. The signed, versioned,
    /// application-local, per-machine pack is the deployment-owner decision supplied 2026-08-09.
    #[test]
    fn an_offline_clean_machine_makes_no_network_request_and_every_claimed_capability_passes_its_probe(
    ) {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(
            config["bundle"]["windows"]["webviewInstallMode"]["type"].as_str(),
            Some("offlineInstaller")
        );

        let valid = offline_qualification_fixture();
        validate_offline_deployment(&valid).expect("complete offline qualification passes");
        assert!(valid
            .probe
            .packages
            .iter()
            .any(|package| package.distribution == "scipy" && !package.available));

        let mut network_used = valid.clone();
        network_used.network_requests_observed = 1;
        let error = validate_offline_deployment(&network_used).unwrap_err();
        assert!(error.contains("zero observed network requests"), "{error}");

        let mut missing_required = valid.clone();
        let numpy = missing_required
            .probe
            .packages
            .iter_mut()
            .find(|package| package.distribution == "numpy")
            .expect("NumPy probe row");
        numpy.available = false;
        numpy.error = Some("required package absent".to_string());
        let error = validate_offline_deployment(&missing_required).unwrap_err();
        assert!(error.contains("Python equations"), "{error}");
        assert!(error.contains("numpy"), "{error}");

        let mut incomplete_claim = valid.clone();
        let omitted = incomplete_claim.claimed_capabilities.pop().unwrap();
        let error = validate_offline_deployment(&incomplete_claim).unwrap_err();
        assert!(error.contains(&omitted), "{error}");

        let mut unsigned = valid;
        unsigned.signature.valid = false;
        let error = validate_offline_deployment(&unsigned).unwrap_err();
        assert!(error.contains("valid signature"), "{error}");

        let gate = include_str!("../examples/installation_gate.rs");
        assert!(gate.contains("validate_installer_qualification_file"));
        assert!(gate.contains("validate_offline_deployment_file"));
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

    fn clean_machine_qualification_fixture(
        installer: &InstallerQualification,
    ) -> CleanMachineQualification {
        let targets = vec![
            MicrosoftServicedWindowsTarget {
                feature_release: "fixture-shared-serviced-release".to_string(),
                edition: WindowsEdition::Pro,
            },
            MicrosoftServicedWindowsTarget {
                feature_release: "fixture-shared-serviced-release".to_string(),
                edition: WindowsEdition::Enterprise,
            },
            MicrosoftServicedWindowsTarget {
                feature_release: "fixture-enterprise-only-serviced-release".to_string(),
                edition: WindowsEdition::Enterprise,
            },
        ];
        let mut results = Vec::new();
        for target in &targets {
            for scenario in CLEAN_MACHINE_SCENARIOS {
                results.push(CleanMachineScenarioResult {
                    feature_release: target.feature_release.clone(),
                    edition: target.edition,
                    scenario: *scenario,
                    passed: true,
                    evidence: "isolated clean-machine result".to_string(),
                });
            }
        }
        CleanMachineQualification {
            installer_sha256: installer.installer_sha256.clone(),
            qualified_build_commit: installer.build_commit.clone(),
            microsoft_serviced_inventory: MicrosoftServicedWindowsInventory {
                product: WINDOWS_PRODUCT.to_string(),
                architecture: WINDOWS_ARCHITECTURE.to_string(),
                observed_at_release: "fixture-release-time-observation".to_string(),
                source: "preserved Microsoft servicing snapshot".to_string(),
                source_sha256: "d".repeat(64),
                targets,
            },
            results,
        }
    }

    /// SB-INS-023 / SB-INS-T28. The nine scenarios are the complete list in SB-INS-023;
    /// Windows 11 x64 Pro/Enterprise across every Microsoft-serviced feature release is the
    /// deployment-owner decision supplied 2026-08-09. The passing and failing sides share the
    /// same exact installer digest and build provenance so a gate cannot qualify another artifact.
    #[test]
    fn one_failing_clean_machine_scenario_blocks_release_and_names_the_scenario() {
        assert_eq!(
            CLEAN_MACHINE_SCENARIOS
                .iter()
                .map(|scenario| scenario.id())
                .collect::<Vec<_>>(),
            vec![
                "standard_user",
                "locked_down_user",
                "offline_install",
                "no_python_core_use",
                "supported_external_python",
                "missing_package",
                "upgrade",
                "rollback",
                "uninstall_preservation",
            ]
        );
        assert_eq!(
            QUALIFIED_WINDOWS_EDITIONS,
            &[WindowsEdition::Pro, WindowsEdition::Enterprise]
        );

        let installer = qualified_fixture();
        let valid = clean_machine_qualification_fixture(&installer);
        assert_eq!(
            valid.results.len(),
            valid.microsoft_serviced_inventory.targets.len() * CLEAN_MACHINE_SCENARIOS.len()
        );
        validate_clean_machine_qualification(&valid, &installer)
            .expect("the complete passing cross-product is publishable");

        let mut failing = valid.clone();
        failing
            .results
            .iter_mut()
            .find(|result| {
                result.feature_release == "fixture-shared-serviced-release"
                    && result.edition == WindowsEdition::Enterprise
                    && result.scenario == CleanMachineScenario::Rollback
            })
            .expect("Enterprise rollback scenario")
            .passed = false;
        let error = validate_clean_machine_qualification(&failing, &installer).unwrap_err();
        assert!(error.contains("installer is not publishable"), "{error}");
        assert!(error.contains("fixture-shared-serviced-release"), "{error}");
        assert!(error.contains("Enterprise"), "{error}");
        assert!(error.contains("rollback"), "{error}");

        let mut omitted = valid.clone();
        omitted.results.retain(|result| {
            !(result.feature_release == "fixture-shared-serviced-release"
                && result.edition == WindowsEdition::Pro
                && result.scenario == CleanMachineScenario::StandardUser)
        });
        let error = validate_clean_machine_qualification(&omitted, &installer).unwrap_err();
        assert!(error.contains("matrix is missing"), "{error}");
        assert!(error.contains("Pro / standard_user"), "{error}");

        let mut expanded_inventory = valid;
        expanded_inventory
            .microsoft_serviced_inventory
            .targets
            .push(MicrosoftServicedWindowsTarget {
                feature_release: "fixture-newly-serviced-release".to_string(),
                edition: WindowsEdition::Pro,
            });
        let error =
            validate_clean_machine_qualification(&expanded_inventory, &installer).unwrap_err();
        assert!(error.contains("fixture-newly-serviced-release"), "{error}");
        assert!(error.contains("matrix is missing"), "{error}");

        let gate = include_str!("../examples/installation_gate.rs");
        assert!(gate.contains("<clean-machine-qualification.json>"));
        assert!(gate.contains("validate_clean_machine_qualification_file"));
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
