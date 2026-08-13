//! Versioned installation-time parameter packs.
//!
//! Display labels are presentation only. A loaded row is addressable by its exact semantic
//! identifier and ordinal, and duplicate labels never participate in selection.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ParameterPackRow {
    pub semantic_id: String,
    pub module_schema_version: String,
    pub ordinal: u32,
    pub display_label: String,
    /// The installation domain preserves a supplied value but does not interpret or default it.
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawParameterPackRow {
    semantic_id: String,
    module_schema_version: String,
    ordinal: Option<u32>,
    display_label: String,
    value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawParameterPack {
    rows: Vec<RawParameterPackRow>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ParameterPack {
    pub source_file: String,
    pub rows: Vec<ParameterPackRow>,
    #[serde(skip)]
    by_semantic_id: BTreeMap<String, usize>,
    #[serde(skip)]
    by_ordinal: BTreeMap<u32, usize>,
}

impl ParameterPack {
    pub fn by_semantic_id(&self, semantic_id: &str) -> Option<&ParameterPackRow> {
        self.by_semantic_id
            .get(semantic_id)
            .and_then(|index| self.rows.get(*index))
    }

    pub fn by_ordinal(&self, ordinal: u32) -> Option<&ParameterPackRow> {
        self.by_ordinal
            .get(&ordinal)
            .and_then(|index| self.rows.get(*index))
    }

    pub fn by_key(&self, semantic_id: &str, ordinal: u32) -> Option<&ParameterPackRow> {
        let semantic_index = self.by_semantic_id.get(semantic_id)?;
        let ordinal_index = self.by_ordinal.get(&ordinal)?;
        (semantic_index == ordinal_index).then(|| &self.rows[*semantic_index])
    }
}

fn parse_parameter_pack_structure(path: &Path) -> Result<ParameterPack, String> {
    let text = crate::parsers::read_text_file(path)
        .map_err(|error| format!("{}: cannot read parameter pack: {error}", path.display()))?;
    let raw: RawParameterPack = serde_json::from_str(&text)
        .map_err(|error| format!("{}: invalid parameter-pack JSON: {error}", path.display()))?;

    let mut rows = Vec::with_capacity(raw.rows.len());
    let mut by_semantic_id = BTreeMap::new();
    let mut by_ordinal = BTreeMap::new();
    for (index, row) in raw.rows.into_iter().enumerate() {
        let row_number = index + 1;
        if row.semantic_id.trim().is_empty() {
            return Err(format!(
                "{}: row {row_number} has an empty semantic identifier",
                path.display()
            ));
        }
        if row.module_schema_version.trim().is_empty() {
            return Err(format!(
                "{}: row {row_number} has an empty module schema version",
                path.display()
            ));
        }
        let ordinal = row.ordinal.ok_or_else(|| {
            format!(
                "{}: row {row_number} ({}) has no ordinal",
                path.display(),
                row.semantic_id
            )
        })?;
        if let Some(previous) = by_semantic_id.insert(row.semantic_id.clone(), index) {
            return Err(format!(
                "{}: rows {} and {row_number} claim semantic identifier {}",
                path.display(),
                previous + 1,
                row.semantic_id
            ));
        }
        if let Some(previous) = by_ordinal.insert(ordinal, index) {
            return Err(format!(
                "{}: rows {} and {row_number} claim ordinal {ordinal}",
                path.display(),
                previous + 1
            ));
        }
        rows.push(ParameterPackRow {
            semantic_id: row.semantic_id,
            module_schema_version: row.module_schema_version,
            ordinal,
            display_label: row.display_label,
            value: row.value,
        });
    }

    Ok(ParameterPack {
        source_file: path.to_string_lossy().into_owned(),
        rows,
        by_semantic_id,
        by_ordinal,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ParameterSchemaEntry {
    pub semantic_id: String,
    pub ordinal: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ParameterModuleSchema {
    /// Supplied by the owning module. Installation code ships no guessed supported version.
    pub module_schema_version: String,
    pub parameters: Vec<ParameterSchemaEntry>,
}

fn is_pack_parameter(kind: &crate::modules::ArgKind) -> bool {
    matches!(
        kind,
        crate::modules::ArgKind::Param
            | crate::modules::ArgKind::Option
            | crate::modules::ArgKind::Text
    )
}

/// Return the schema owned by a shipping module. `ArgSpec::name` is the stable wire id already
/// persisted in saved runs; the human-facing description is deliberately excluded from identity.
/// The version is a deterministic digest of the module's configurable manifest, so no caller can
/// invent a version or keep using one after that manifest changes.
pub fn module_parameter_schema(module_name: &str) -> Result<ParameterModuleSchema, String> {
    let modules = crate::modules::list_modules();
    let module = modules
        .iter()
        .find(|candidate| candidate.name == module_name)
        .ok_or_else(|| format!("unknown parameter module '{module_name}'"))?;
    let configurable = module
        .args
        .iter()
        .filter(|argument| is_pack_parameter(&argument.kind))
        .collect::<Vec<_>>();
    let canonical_manifest = serde_json::to_vec(&(module.name.as_str(), &configurable))
        .map_err(|error| format!("cannot version parameter schema for {module_name}: {error}"))?;
    let digest = Sha256::digest(canonical_manifest);
    let digest_hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    let parameters = configurable
        .iter()
        .enumerate()
        .map(|(index, argument)| {
            let ordinal = u32::try_from(index + 1)
                .map_err(|_| format!("parameter schema for {module_name} exceeds u32 ordinals"))?;
            Ok(ParameterSchemaEntry {
                semantic_id: format!("{}.{}", module.name, argument.name),
                ordinal,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(ParameterModuleSchema {
        module_schema_version: format!("sha256:{digest_hex}"),
        parameters,
    })
}

/// Product loader: the caller chooses a shipping module and a file, while the backend supplies
/// the authoritative schema. A frontend-supplied schema can therefore never legitimise crossed
/// identifiers or ordinals.
pub fn load_parameter_pack_for_module(
    path: &Path,
    module_name: &str,
) -> Result<ParameterPack, String> {
    let schema = module_parameter_schema(module_name)?;
    load_parameter_pack_against_schema(path, &schema)
}

#[tauri::command]
pub fn get_parameter_module_schema(module_name: String) -> Result<ParameterModuleSchema, String> {
    module_parameter_schema(&module_name)
}

#[tauri::command]
pub fn load_parameter_pack(module_name: String, path: String) -> Result<ParameterPack, String> {
    load_parameter_pack_for_module(Path::new(&path), &module_name)
}

type SchemaById<'a> = BTreeMap<&'a str, (u32, usize)>;
type SchemaByOrdinal<'a> = BTreeMap<u32, (&'a str, usize)>;

fn schema_indexes(
    schema: &ParameterModuleSchema,
) -> Result<(SchemaById<'_>, SchemaByOrdinal<'_>), String> {
    if schema.module_schema_version.trim().is_empty() {
        return Err("parameter module schema has an empty version".to_string());
    }
    let mut by_semantic_id = BTreeMap::new();
    let mut by_ordinal = BTreeMap::new();
    for (index, parameter) in schema.parameters.iter().enumerate() {
        let row_number = index + 1;
        if parameter.semantic_id.trim().is_empty() {
            return Err(format!(
                "parameter module schema row {row_number} has an empty semantic identifier"
            ));
        }
        if let Some((_, previous)) =
            by_semantic_id.insert(parameter.semantic_id.as_str(), (parameter.ordinal, index))
        {
            return Err(format!(
                "parameter module schema rows {} and {row_number} claim semantic identifier {}",
                previous + 1,
                parameter.semantic_id
            ));
        }
        if let Some((previous_id, previous)) =
            by_ordinal.insert(parameter.ordinal, (parameter.semantic_id.as_str(), index))
        {
            return Err(format!(
                "parameter module schema rows {} ({previous_id}) and {row_number} ({}) claim ordinal {}",
                previous + 1,
                parameter.semantic_id,
                parameter.ordinal
            ));
        }
    }
    Ok((by_semantic_id, by_ordinal))
}

/// Load atomically against the owning module's supplied schema. No row is returned until every
/// semantic identifier, ordinal and schema version agrees.
pub fn load_parameter_pack_against_schema(
    path: &Path,
    schema: &ParameterModuleSchema,
) -> Result<ParameterPack, String> {
    let (schema_by_id, schema_by_ordinal) =
        schema_indexes(schema).map_err(|error| format!("{}: {error}", path.display()))?;
    let pack = parse_parameter_pack_structure(path)?;
    for (index, row) in pack.rows.iter().enumerate() {
        let row_number = index + 1;
        if row.module_schema_version != schema.module_schema_version {
            return Err(format!(
                "{}: row {row_number} ({}) uses unsupported schema {}; owning module supplies {}",
                path.display(),
                row.semantic_id,
                row.module_schema_version,
                schema.module_schema_version
            ));
        }
        let Some((expected_ordinal, semantic_schema_index)) =
            schema_by_id.get(row.semantic_id.as_str())
        else {
            let owner = schema_by_ordinal
                .get(&row.ordinal)
                .map(|(semantic_id, schema_index)| {
                    format!(
                        "; ordinal {} belongs to schema row {} ({semantic_id})",
                        row.ordinal,
                        schema_index + 1
                    )
                })
                .unwrap_or_default();
            return Err(format!(
                "{}: row {row_number} has unknown semantic identifier {}{owner}",
                path.display(),
                row.semantic_id
            ));
        };
        if row.ordinal != *expected_ordinal {
            let actual_owner = schema_by_ordinal
                .get(&row.ordinal)
                .map(|(semantic_id, schema_index)| {
                    format!("schema row {} ({semantic_id})", schema_index + 1)
                })
                .unwrap_or_else(|| "no schema row".to_string());
            return Err(format!(
                "{}: pack row {row_number} identifies {} from schema row {} (ordinal {}), but carries ordinal {} assigned to {actual_owner}",
                path.display(),
                row.semantic_id,
                semantic_schema_index + 1,
                expected_ordinal,
                row.ordinal
            ));
        }
    }
    Ok(pack)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SB-INS-014 / SB-INS-T16. Duplicate display labels with unique semantic identifiers and
    /// ordinals come from dossier section 2.4. Fixture values are opaque test markers, not
    /// scientific parameters or shipped defaults. This exercises the product-reachable loader,
    /// not the private structural parser, and pins its Tauri and TypeScript registrations so the
    /// safe loader cannot silently become orphaned again.
    #[test]
    fn two_identically_labelled_loaded_parameter_rows_remain_separately_addressable_by_semantic_identifier_and_ordinal(
    ) {
        let temp =
            std::env::temp_dir().join(format!("sandibumi-parameter-pack-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp).unwrap();
        let path = temp.join("duplicate-labels.json");
        let schema = module_parameter_schema("vsh_gr").expect("a shipping module owns its schema");
        let first = &schema.parameters[0];
        let second = &schema.parameters[1];
        let fixture = serde_json::json!({
            "rows": [
                {
                    "semantic_id": first.semantic_id,
                    "module_schema_version": schema.module_schema_version,
                    "ordinal": first.ordinal,
                    "display_label": "Repeated label",
                    "value": { "fixture": "first" }
                },
                {
                    "semantic_id": second.semantic_id,
                    "module_schema_version": schema.module_schema_version,
                    "ordinal": second.ordinal,
                    "display_label": "Repeated label",
                    "value": { "fixture": "second" }
                }
            ]
        });
        std::fs::write(&path, serde_json::to_vec_pretty(&fixture).unwrap()).unwrap();

        let pack = load_parameter_pack_for_module(&path, "vsh_gr")
            .expect("duplicate labels are presentation only");
        assert_eq!(pack.rows.len(), 2);
        assert_eq!(pack.rows[0].display_label, pack.rows[1].display_label);
        assert_eq!(
            pack.by_semantic_id(&first.semantic_id)
                .map(|row| row.ordinal),
            Some(first.ordinal)
        );
        assert_eq!(
            pack.by_ordinal(second.ordinal)
                .map(|row| row.semantic_id.as_str()),
            Some(second.semantic_id.as_str())
        );
        assert!(pack.by_key(&first.semantic_id, first.ordinal).is_some());
        assert!(pack.by_key(&second.semantic_id, second.ordinal).is_some());
        assert!(pack.by_key(&first.semantic_id, second.ordinal).is_none());

        let backend = include_str!("lib.rs");
        assert!(
            backend.contains("parameter_pack::load_parameter_pack"),
            "the governed loader must remain registered at the Tauri boundary"
        );
        let frontend = include_str!("../../src/ipc.ts");
        assert!(
            frontend.contains("export function loadParameterPack")
                && frontend.contains("invoke<ParameterPack>(\"load_parameter_pack\""),
            "the governed loader must remain addressable through typed IPC"
        );

        std::fs::remove_dir_all(&temp).unwrap();
    }

    fn write_fixture(
        directory: &Path,
        name: &str,
        fixture: serde_json::Value,
    ) -> std::path::PathBuf {
        let path = directory.join(name);
        std::fs::write(&path, serde_json::to_vec_pretty(&fixture).unwrap()).unwrap();
        path
    }

    /// SB-INS-015 / SB-INS-T17. Bidirectional identifier/ordinal agreement and naming both rows
    /// come from dossier section 3.8. The fixture takes both identities from the backend-owned
    /// shipping schema and crosses only their keys; its value is an opaque structural marker.
    #[test]
    fn an_identifier_ordinal_disagreement_stops_loading_and_names_both_schema_rows() {
        let temp = std::env::temp_dir().join(format!(
            "sandibumi-parameter-mismatch-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&temp).unwrap();
        let schema = module_parameter_schema("vsh_gr").expect("a shipping module owns its schema");
        let first = &schema.parameters[0];
        let second = &schema.parameters[1];
        let path = write_fixture(
            &temp,
            "crossed-key.json",
            serde_json::json!({
                "rows": [{
                    "semantic_id": first.semantic_id,
                    "module_schema_version": schema.module_schema_version,
                    "ordinal": second.ordinal,
                    "display_label": "Fixture",
                    "value": { "fixture": true }
                }]
            }),
        );

        let error = load_parameter_pack("vsh_gr".to_string(), path.to_string_lossy().into_owned())
            .unwrap_err();
        assert!(error.contains(path.to_string_lossy().as_ref()), "{error}");
        assert!(error.contains("pack row 1"), "{error}");
        assert!(error.contains(&first.semantic_id), "{error}");
        assert!(error.contains("schema row 1"), "{error}");
        assert!(error.contains(&second.semantic_id), "{error}");
        assert!(error.contains("schema row 2"), "{error}");

        std::fs::remove_dir_all(&temp).unwrap();
    }

    /// SB-INS-015 / SB-INS-T18. The four refusal shapes and all-or-nothing load come from dossier
    /// sections 2.4 and 2.13. Every identity/version comes from the backend-owned shipping schema;
    /// no fixture value is interpreted or returned after any refusal.
    #[test]
    fn missing_ordinals_duplicate_keys_unsupported_schemas_and_empty_keys_are_all_refused_without_partial_activation(
    ) {
        let temp = std::env::temp_dir().join(format!(
            "sandibumi-parameter-refusals-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&temp).unwrap();
        let schema = module_parameter_schema("vsh_gr").expect("a shipping module owns its schema");
        let first = &schema.parameters[0];
        let fixtures = [
            (
                "missing-ordinal.json",
                serde_json::json!({ "rows": [{
                    "semantic_id": first.semantic_id,
                    "module_schema_version": schema.module_schema_version,
                    "display_label": "Fixture",
                    "value": { "fixture": true }
                }]}),
                "has no ordinal",
            ),
            (
                "duplicate-key.json",
                serde_json::json!({ "rows": [
                    {
                        "semantic_id": first.semantic_id,
                        "module_schema_version": schema.module_schema_version,
                        "ordinal": first.ordinal,
                        "display_label": "First",
                        "value": { "fixture": "first" }
                    },
                    {
                        "semantic_id": first.semantic_id,
                        "module_schema_version": schema.module_schema_version,
                        "ordinal": first.ordinal,
                        "display_label": "Second",
                        "value": { "fixture": "second" }
                    }
                ]}),
                "rows 1 and 2 claim semantic identifier",
            ),
            (
                "unsupported-schema.json",
                serde_json::json!({ "rows": [{
                    "semantic_id": first.semantic_id,
                    "module_schema_version": "fixture/unsupported",
                    "ordinal": first.ordinal,
                    "display_label": "Fixture",
                    "value": { "fixture": true }
                }]}),
                "unsupported schema",
            ),
            (
                "empty-key.json",
                serde_json::json!({ "rows": [{
                    "semantic_id": "",
                    "module_schema_version": schema.module_schema_version,
                    "ordinal": first.ordinal,
                    "display_label": "Fixture",
                    "value": { "fixture": true }
                }]}),
                "empty semantic identifier",
            ),
        ];

        let mut activated = Vec::new();
        for (name, fixture, expected) in fixtures {
            let path = write_fixture(&temp, name, fixture);
            match load_parameter_pack("vsh_gr".to_string(), path.to_string_lossy().into_owned()) {
                Ok(pack) => activated.push(pack),
                Err(error) => {
                    assert!(error.contains(path.to_string_lossy().as_ref()), "{error}");
                    assert!(error.contains(expected), "{name}: {error}");
                }
            }
        }
        assert!(
            activated.is_empty(),
            "no refused fixture may partially activate"
        );

        std::fs::remove_dir_all(&temp).unwrap();
    }
}
