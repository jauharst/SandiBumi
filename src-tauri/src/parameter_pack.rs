//! Versioned installation-time parameter packs.
//!
//! Display labels are presentation only. A loaded row is addressable by its exact semantic
//! identifier and ordinal, and duplicate labels never participate in selection.

use serde::{Deserialize, Serialize};
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
    /// scientific parameters or shipped defaults.
    #[test]
    fn duplicate_display_labels_remain_separately_addressable_by_semantic_identifier_and_ordinal() {
        let temp =
            std::env::temp_dir().join(format!("sandibumi-parameter-pack-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp).unwrap();
        let path = temp.join("duplicate-labels.json");
        let fixture = serde_json::json!({
            "rows": [
                {
                    "semantic_id": "fixture.parameter.alpha",
                    "module_schema_version": "fixture/v1",
                    "ordinal": 1,
                    "display_label": "Repeated label",
                    "value": { "fixture": "first" }
                },
                {
                    "semantic_id": "fixture.parameter.beta",
                    "module_schema_version": "fixture/v1",
                    "ordinal": 2,
                    "display_label": "Repeated label",
                    "value": { "fixture": "second" }
                }
            ]
        });
        std::fs::write(&path, serde_json::to_vec_pretty(&fixture).unwrap()).unwrap();

        let pack =
            parse_parameter_pack_structure(&path).expect("duplicate labels are presentation only");
        assert_eq!(pack.rows.len(), 2);
        assert_eq!(pack.rows[0].display_label, pack.rows[1].display_label);
        assert_eq!(
            pack.by_semantic_id("fixture.parameter.alpha")
                .map(|row| row.ordinal),
            Some(1)
        );
        assert_eq!(
            pack.by_ordinal(2).map(|row| row.semantic_id.as_str()),
            Some("fixture.parameter.beta")
        );
        assert!(pack.by_key("fixture.parameter.alpha", 1).is_some());
        assert!(pack.by_key("fixture.parameter.alpha", 2).is_none());

        std::fs::remove_dir_all(&temp).unwrap();
    }

    fn fixture_schema() -> ParameterModuleSchema {
        ParameterModuleSchema {
            module_schema_version: "fixture/v1".to_string(),
            parameters: vec![
                ParameterSchemaEntry {
                    semantic_id: "fixture.parameter.alpha".to_string(),
                    ordinal: 1,
                },
                ParameterSchemaEntry {
                    semantic_id: "fixture.parameter.beta".to_string(),
                    ordinal: 2,
                },
            ],
        }
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
    /// come from dossier section 3.8. Schema IDs and ordinals are structural test fixtures only.
    #[test]
    fn an_identifier_ordinal_disagreement_stops_loading_and_names_both_schema_rows() {
        let temp = std::env::temp_dir().join(format!(
            "sandibumi-parameter-mismatch-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&temp).unwrap();
        let path = write_fixture(
            &temp,
            "crossed-key.json",
            serde_json::json!({
                "rows": [{
                    "semantic_id": "fixture.parameter.alpha",
                    "module_schema_version": "fixture/v1",
                    "ordinal": 2,
                    "display_label": "Fixture",
                    "value": { "fixture": true }
                }]
            }),
        );

        let error = load_parameter_pack_against_schema(&path, &fixture_schema()).unwrap_err();
        assert!(error.contains(path.to_string_lossy().as_ref()), "{error}");
        assert!(error.contains("pack row 1"), "{error}");
        assert!(error.contains("fixture.parameter.alpha"), "{error}");
        assert!(error.contains("schema row 1"), "{error}");
        assert!(error.contains("fixture.parameter.beta"), "{error}");
        assert!(error.contains("schema row 2"), "{error}");

        std::fs::remove_dir_all(&temp).unwrap();
    }

    /// SB-INS-015 / SB-INS-T18. The four refusal shapes and all-or-nothing load come from dossier
    /// sections 2.4 and 2.13. No fixture value is interpreted or activated after any refusal.
    #[test]
    fn missing_ordinals_duplicate_keys_unsupported_schemas_and_empty_keys_are_all_refused_without_partial_activation(
    ) {
        let temp = std::env::temp_dir().join(format!(
            "sandibumi-parameter-refusals-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&temp).unwrap();
        let fixtures = [
            (
                "missing-ordinal.json",
                serde_json::json!({ "rows": [{
                    "semantic_id": "fixture.parameter.alpha",
                    "module_schema_version": "fixture/v1",
                    "display_label": "Fixture",
                    "value": { "fixture": true }
                }]}),
                "has no ordinal",
            ),
            (
                "duplicate-key.json",
                serde_json::json!({ "rows": [
                    {
                        "semantic_id": "fixture.parameter.alpha",
                        "module_schema_version": "fixture/v1",
                        "ordinal": 1,
                        "display_label": "First",
                        "value": { "fixture": "first" }
                    },
                    {
                        "semantic_id": "fixture.parameter.alpha",
                        "module_schema_version": "fixture/v1",
                        "ordinal": 1,
                        "display_label": "Second",
                        "value": { "fixture": "second" }
                    }
                ]}),
                "rows 1 and 2 claim semantic identifier",
            ),
            (
                "unsupported-schema.json",
                serde_json::json!({ "rows": [{
                    "semantic_id": "fixture.parameter.alpha",
                    "module_schema_version": "fixture/unsupported",
                    "ordinal": 1,
                    "display_label": "Fixture",
                    "value": { "fixture": true }
                }]}),
                "unsupported schema",
            ),
            (
                "empty-key.json",
                serde_json::json!({ "rows": [{
                    "semantic_id": "",
                    "module_schema_version": "fixture/v1",
                    "ordinal": 1,
                    "display_label": "Fixture",
                    "value": { "fixture": true }
                }]}),
                "empty semantic identifier",
            ),
        ];

        let mut activated = Vec::new();
        for (name, fixture, expected) in fixtures {
            let path = write_fixture(&temp, name, fixture);
            match load_parameter_pack_against_schema(&path, &fixture_schema()) {
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
