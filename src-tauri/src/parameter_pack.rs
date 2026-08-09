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

pub fn load_parameter_pack(path: &Path) -> Result<ParameterPack, String> {
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

        let pack = load_parameter_pack(&path).expect("duplicate labels are presentation only");
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
}
