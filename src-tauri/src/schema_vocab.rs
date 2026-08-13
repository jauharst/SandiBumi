//! One typed source for vocabularies that describe the project-store schema.
//!
//! SB-DBM-023 is stronger than making two lists equal. Readers and writers consume these exports,
//! and a projection (such as editable standard columns) is computed from the registered entries.
//! Adding a member therefore reaches every projection without copying a second literal list.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StandardColumn {
    pub mnemonic: &'static str,
    pub storage_column: &'static str,
    pub editable: bool,
    pub required: bool,
}

pub(crate) const STANDARD_COLUMNS: &[StandardColumn] = &[
    StandardColumn {
        mnemonic: "DEPTH",
        storage_column: "depth",
        editable: false,
        required: true,
    },
    StandardColumn {
        mnemonic: "GR",
        storage_column: "gr",
        editable: true,
        required: false,
    },
    StandardColumn {
        mnemonic: "RES_DEEP",
        storage_column: "res_deep",
        editable: true,
        required: false,
    },
    StandardColumn {
        mnemonic: "NPHI",
        storage_column: "nphi",
        editable: true,
        required: false,
    },
    StandardColumn {
        mnemonic: "RHOB",
        storage_column: "rhob",
        editable: true,
        required: false,
    },
    StandardColumn {
        mnemonic: "DT",
        storage_column: "dt",
        editable: true,
        required: false,
    },
    StandardColumn {
        mnemonic: "SP",
        storage_column: "sp",
        editable: true,
        required: false,
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StandardColumnProjections {
    pub select_list: String,
    pub editable: Vec<(&'static str, &'static str)>,
    pub inspector_columns: Vec<&'static str>,
    pub table_ddl: String,
    pub migration_ddl: String,
}

pub(crate) fn derive_standard_projections(columns: &[StandardColumn]) -> StandardColumnProjections {
    let select_list = columns
        .iter()
        .map(|column| column.storage_column)
        .collect::<Vec<_>>()
        .join(", ");
    let editable = columns
        .iter()
        .filter(|column| column.editable)
        .map(|column| (column.mnemonic, column.storage_column))
        .collect();
    let inspector_columns = columns.iter().map(|column| column.storage_column).collect();
    let table_ddl = columns
        .iter()
        .map(|column| {
            format!(
                "            {:<11} FLOAT{}",
                column.storage_column,
                if column.required { " NOT NULL" } else { "" }
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    let migration_ddl = columns
        .iter()
        .filter(|column| !column.required)
        .map(|column| {
            format!(
                "ALTER TABLE standard_curves ADD COLUMN IF NOT EXISTS {} FLOAT;",
                column.storage_column
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    StandardColumnProjections {
        select_list,
        editable,
        inspector_columns,
        table_ddl,
        migration_ddl,
    }
}

pub(crate) fn standard_projections() -> StandardColumnProjections {
    derive_standard_projections(STANDARD_COLUMNS)
}

pub(crate) fn standard_column(mnemonic: &str) -> Option<&'static StandardColumn> {
    STANDARD_COLUMNS
        .iter()
        .find(|column| column.mnemonic.eq_ignore_ascii_case(mnemonic.trim()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProvenanceAbsentState {
    NotApplicable,
    RequiredUnset,
    LegacyUnrecorded,
}

impl ProvenanceAbsentState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotApplicable => "NOT_APPLICABLE",
            Self::RequiredUnset => "REQUIRED_UNSET",
            Self::LegacyUnrecorded => "LEGACY_UNRECORDED",
        }
    }
}

pub(crate) const PROVENANCE_ABSENT_STATES: &[ProvenanceAbsentState] = &[
    ProvenanceAbsentState::NotApplicable,
    ProvenanceAbsentState::RequiredUnset,
    ProvenanceAbsentState::LegacyUnrecorded,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SamplingStyle {
    ContinuousRegular,
    ContinuousIrregular,
    Point,
}

impl SamplingStyle {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ContinuousRegular => "CONTINUOUS_REGULAR",
            Self::ContinuousIrregular => "CONTINUOUS_IRREGULAR",
            Self::Point => "POINT",
        }
    }
}

pub(crate) const SAMPLING_STYLES: &[SamplingStyle] = &[
    SamplingStyle::ContinuousRegular,
    SamplingStyle::ContinuousIrregular,
    SamplingStyle::Point,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LogSetFrame {
    Standard,
    Own,
}

impl LogSetFrame {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "STANDARD",
            Self::Own => "OWN",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "STANDARD" => Some(Self::Standard),
            "OWN" => Some(Self::Own),
            _ => None,
        }
    }
}

pub(crate) const LOG_SET_FRAMES: &[LogSetFrame] = &[LogSetFrame::Standard, LogSetFrame::Own];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DepthDatum {
    Md,
    Tvd,
    Tvdss,
    Tvdkb,
    Twt,
    Owt,
    Cdepth,
}

impl DepthDatum {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Md => "MD",
            Self::Tvd => "TVD",
            Self::Tvdss => "TVDSS",
            Self::Tvdkb => "TVDKB",
            Self::Twt => "TWT",
            Self::Owt => "OWT",
            Self::Cdepth => "CDEPTH",
        }
    }
}

pub(crate) const DEPTH_DATUMS: &[DepthDatum] = &[
    DepthDatum::Md,
    DepthDatum::Tvd,
    DepthDatum::Tvdss,
    DepthDatum::Tvdkb,
    DepthDatum::Twt,
    DepthDatum::Owt,
    DepthDatum::Cdepth,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuditLocation {
    Parameter,
    Comment,
    Set,
    Constant,
    Interval,
    Log,
    Attribute,
}

impl AuditLocation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Parameter => "PARAMETER",
            Self::Comment => "COMMENT",
            Self::Set => "SET",
            Self::Constant => "CONSTANT",
            Self::Interval => "INTERVAL",
            Self::Log => "LOG",
            Self::Attribute => "ATTRIBUTE",
        }
    }
}

pub(crate) const AUDIT_LOCATIONS: &[AuditLocation] = &[
    AuditLocation::Parameter,
    AuditLocation::Comment,
    AuditLocation::Set,
    AuditLocation::Constant,
    AuditLocation::Interval,
    AuditLocation::Log,
    AuditLocation::Attribute,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuditMode {
    Input,
    Output,
    Delete,
    Rename,
    Save,
    SaveAs,
    SaveCancel,
}

impl AuditMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Input => "INPUT",
            Self::Output => "OUTPUT",
            Self::Delete => "DELETE",
            Self::Rename => "RENAME",
            Self::Save => "SAVE",
            Self::SaveAs => "SAVE_AS",
            Self::SaveCancel => "SAVE_CANCEL",
        }
    }
}

pub(crate) const AUDIT_MODES: &[AuditMode] = &[
    AuditMode::Input,
    AuditMode::Output,
    AuditMode::Delete,
    AuditMode::Rename,
    AuditMode::Save,
    AuditMode::SaveAs,
    AuditMode::SaveCancel,
];

pub(crate) fn validate_schema_vocabularies() -> Result<(), String> {
    fn unique<'a>(name: &str, values: impl IntoIterator<Item = &'a str>) -> Result<(), String> {
        let mut seen = std::collections::HashSet::new();
        for value in values {
            if !seen.insert(value) {
                return Err(format!("schema vocabulary '{name}' repeats '{value}'"));
            }
        }
        Ok(())
    }

    unique(
        "standard mnemonic",
        STANDARD_COLUMNS.iter().map(|column| column.mnemonic),
    )?;
    unique(
        "standard storage column",
        STANDARD_COLUMNS.iter().map(|column| column.storage_column),
    )?;
    if STANDARD_COLUMNS
        .first()
        .map(|column| (column.mnemonic, column.storage_column, column.required))
        != Some(("DEPTH", "depth", true))
    {
        return Err(
            "the required DEPTH/depth entry must remain first in the standard-column registry"
                .into(),
        );
    }
    unique(
        "sampling style",
        SAMPLING_STYLES.iter().map(|value| value.as_str()),
    )?;
    unique(
        "log-set frame",
        LOG_SET_FRAMES.iter().map(|value| value.as_str()),
    )?;
    unique(
        "depth datum",
        DEPTH_DATUMS.iter().map(|value| value.as_str()),
    )?;
    unique(
        "audit location",
        AUDIT_LOCATIONS.iter().map(|value| value.as_str()),
    )?;
    unique("audit mode", AUDIT_MODES.iter().map(|value| value.as_str()))?;
    unique(
        "provenance absent state",
        PROVENANCE_ABSENT_STATES.iter().map(|value| value.as_str()),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn rust_files(path: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(path).expect("read Rust source directory") {
            let path = entry.expect("read Rust source entry").path();
            if path.is_dir() {
                rust_files(&path, files);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }

    fn literal_declarations(source: &str) -> Vec<&str> {
        let mut declarations = Vec::new();
        for marker in ["const ", "static "] {
            for (start, _) in source.match_indices(marker) {
                if let Some(end) = source[start..].find(';') {
                    declarations.push(&source[start..=start + end]);
                }
            }
        }
        declarations
    }

    /// CORRECTNESS — SB-DBM-023 / SB-DBM-T23. The seven vocabulary populations, the
    /// one-declaration rule, the eighth-column mutation and the second-literal refusal come from
    /// `docs/PRD_v2/22_database-model.md` sections 4.D and 6.5. The synthetic PEF column supplies
    /// no petrophysical value; it proves propagation of a schema name and storage projection only.
    #[test]
    fn vocabularies_have_one_source_and_every_projection_derives_from_it() {
        let mut extended = STANDARD_COLUMNS.to_vec();
        extended.push(StandardColumn {
            mnemonic: "PEF",
            storage_column: "pef",
            editable: true,
            required: false,
        });
        let projections = derive_standard_projections(&extended);
        assert!(projections.select_list.ends_with(", pef"));
        assert!(projections.editable.contains(&("PEF", "pef")));
        assert!(projections.inspector_columns.contains(&"pef"));
        assert!(projections.table_ddl.contains("pef         FLOAT"));
        assert!(projections
            .migration_ddl
            .contains("ADD COLUMN IF NOT EXISTS pef FLOAT"));

        assert_eq!(
            SAMPLING_STYLES
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>(),
            ["CONTINUOUS_REGULAR", "CONTINUOUS_IRREGULAR", "POINT"]
        );
        assert_eq!(
            LOG_SET_FRAMES
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>(),
            ["STANDARD", "OWN"]
        );
        assert_eq!(
            DEPTH_DATUMS
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>(),
            ["MD", "TVD", "TVDSS", "TVDKB", "TWT", "OWT", "CDEPTH"]
        );
        assert_eq!(
            AUDIT_LOCATIONS
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>(),
            [
                "PARAMETER",
                "COMMENT",
                "SET",
                "CONSTANT",
                "INTERVAL",
                "LOG",
                "ATTRIBUTE",
            ]
        );
        assert_eq!(
            AUDIT_MODES
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>(),
            [
                "INPUT",
                "OUTPUT",
                "DELETE",
                "RENAME",
                "SAVE",
                "SAVE_AS",
                "SAVE_CANCEL",
            ]
        );
        assert_eq!(
            [
                ProvenanceAbsentState::NotApplicable.as_str(),
                ProvenanceAbsentState::RequiredUnset.as_str(),
                ProvenanceAbsentState::LegacyUnrecorded.as_str(),
            ],
            ["NOT_APPLICABLE", "REQUIRED_UNSET", "LEGACY_UNRECORDED"]
        );

        let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let registry = source_root.join("schema_vocab.rs");
        let mut files = Vec::new();
        rust_files(&source_root, &mut files);
        let declaration_markers = [
            "const STANDARD_COLUMNS",
            "enum ProvenanceAbsentState",
            "enum SamplingStyle",
            "enum LogSetFrame",
            "enum DepthDatum",
            "enum AuditLocation",
            "enum AuditMode",
        ];
        for marker in declaration_markers {
            let owners = files
                .iter()
                .filter(|path| {
                    crate::parsers::read_text_file(path)
                        .expect("read Rust source")
                        .contains(marker)
                })
                .collect::<Vec<_>>();
            assert_eq!(
                owners,
                vec![&registry],
                "'{marker}' must have exactly one declaration owner"
            );
        }

        let registered_literals = STANDARD_COLUMNS
            .iter()
            .flat_map(|column| {
                [
                    format!("\"{}\"", column.mnemonic),
                    format!("\"{}\"", column.storage_column),
                ]
            })
            .collect::<Vec<_>>();
        for path in files.iter().filter(|path| **path != registry) {
            let source = crate::parsers::read_text_file(path).expect("read Rust source");
            for declaration in literal_declarations(&source) {
                assert!(
                    !registered_literals
                        .iter()
                        .all(|literal| declaration.contains(literal)),
                    "{} re-declares the registered standard-column vocabulary",
                    path.display()
                );
            }
        }
    }
}
