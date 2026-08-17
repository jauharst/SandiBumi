//! Versioned installation-time parameter packs.
//!
//! Display labels are presentation only. A loaded row is addressable by its exact semantic
//! identifier and ordinal, and duplicate labels never participate in selection.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

/// SB-DBM-032, from F-19 (`22_database-model.md:433-441`). A tilt is a property of the VALUE, not a
/// display mode: IP's `Lg` prefix marks a value interpolated logarithmically between its zone
/// endpoints, so a tilted parameter flattened to a scalar has lost physics rather than formatting.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ParameterTilt {
    None,
    Linear,
    Log,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ParameterPackRow {
    pub semantic_id: String,
    pub module_schema_version: String,
    pub ordinal: u32,
    pub display_label: String,
    /// The installation domain preserves a supplied value but does not interpret or default it.
    pub value: serde_json::Value,
    /// The value's own unit. `None` is a genuinely unitless parameter (a mode, a ratio), never an
    /// unknown one — an unstated unit on a numeric parameter is refused at load.
    pub unit: Option<String>,
    /// Mandatory for a numeric value (§5.4). A silently defaulted number is not a legal state.
    pub source: Option<String>,
    pub tilt: ParameterTilt,
}

impl ParameterPackRow {
    /// The value at a depth inside the zone this parameter belongs to.
    ///
    /// **Within-zone only** (`22_database-model.md:2953`): a depth outside `[zone_top, zone_base]`
    /// returns `None` rather than an extrapolated or clamped number, because the parameter STEPS at
    /// the boundary and the neighbouring zone carries its own endpoints. Returning a clamped value
    /// here would silently spread one zone's calibration into the next.
    pub fn value_at_depth(&self, zone_top: f64, zone_base: f64, depth: f64) -> Option<f64> {
        let (lo, hi) = if zone_top <= zone_base {
            (zone_top, zone_base)
        } else {
            (zone_base, zone_top)
        };
        if !(depth >= lo && depth <= hi) {
            return None;
        }
        match self.tilt {
            ParameterTilt::None => self.value.as_f64(),
            tilt => {
                let (top, base) = tilt_endpoints(&self.value)?;
                let span = zone_base - zone_top;
                // A zero-thickness zone has no interior to interpolate across; its own top value is
                // the only answer that is not a division by zero.
                let fraction = if span == 0.0 { 0.0 } else { (depth - zone_top) / span };
                Some(match tilt {
                    ParameterTilt::Log => {
                        (top.ln() + (base.ln() - top.ln()) * fraction).exp()
                    }
                    _ => top + (base - top) * fraction,
                })
            }
        }
    }
}

/// The two endpoints a tilted value interpolates between, as the chapter's "two-endpoint range".
fn tilt_endpoints(value: &serde_json::Value) -> Option<(f64, f64)> {
    let pair = value.as_array()?;
    if pair.len() != 2 {
        return None;
    }
    Some((pair[0].as_f64()?, pair[1].as_f64()?))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawParameterPackRow {
    semantic_id: String,
    module_schema_version: String,
    ordinal: Option<u32>,
    display_label: String,
    value: serde_json::Value,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    source: Option<String>,
    /// Read as a raw token so an unrecognised tilt is refused BY NAME rather than failing as a
    /// generic deserialization error or, worse, defaulting to `NONE`.
    #[serde(default)]
    tilt: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawParameterPack {
    #[serde(default)]
    text_encoding: Option<String>,
    rows: Vec<RawParameterPackRow>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TextFileProvenance {
    pub declared_encoding: Option<String>,
    pub decoded_encoding: String,
    /// Reversible source-byte representation. A string is deliberate: raw byte vectors must
    /// never cross Tauri as JSON number arrays.
    pub original_bytes_hex: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ParameterPack {
    pub source_file: String,
    pub text_provenance: TextFileProvenance,
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
    let decoded = crate::parsers::read_text_file_with_encoding(path)
        .map_err(|error| format!("{}: cannot read parameter pack: {error}", path.display()))?;
    let raw: RawParameterPack = serde_json::from_str(&decoded.text)
        .map_err(|error| format!("{}: invalid parameter-pack JSON: {error}", path.display()))?;
    if let Some(declared) = raw.text_encoding.as_deref() {
        let declared_interpretation = match declared {
            "CP1252" | "Windows-1252" => "Windows-1252",
            "UTF-8" => "UTF-8",
            "UTF-8 with BOM" => "UTF-8 with BOM",
            "UTF-16LE with BOM" => "UTF-16LE with BOM",
            "UTF-16BE with BOM" => "UTF-16BE with BOM",
            "UTF-16LE without BOM" => "UTF-16LE without BOM",
            "UTF-16BE without BOM" => "UTF-16BE without BOM",
            _ => {
                return Err(format!(
                    "{}: parameter pack declares unsupported text encoding {declared}",
                    path.display()
                ))
            }
        };
        if declared_interpretation != decoded.encoding {
            return Err(format!(
                "{}: parameter pack declares text encoding {declared}, but source bytes decoded as {}",
                path.display(),
                decoded.encoding
            ));
        }
    }
    let text_provenance = TextFileProvenance {
        declared_encoding: raw.text_encoding,
        decoded_encoding: decoded.encoding,
        original_bytes_hex: decoded
            .original_bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    };

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
        // SB-DBM-032. The tilt token is validated before anything is read from the value, because
        // an unrecognised tilt must never fall through to NONE — that is the one failure mode that
        // returns a plausible number instead of an error.
        let tilt = match row.tilt.as_deref() {
            None | Some("NONE") => ParameterTilt::None,
            Some("LINEAR") => ParameterTilt::Linear,
            Some("LOG") => ParameterTilt::Log,
            Some(other) => {
                return Err(format!(
                    "{}: row {row_number} ({}) declares unrecognised tilt {other}; expected NONE, LINEAR or LOG",
                    path.display(),
                    row.semantic_id
                ))
            }
        };
        if tilt != ParameterTilt::None {
            let (top, base) = tilt_endpoints(&row.value).ok_or_else(|| {
                format!(
                    "{}: row {row_number} ({}) declares tilt {:?} but its value is not a two-endpoint range",
                    path.display(),
                    row.semantic_id,
                    tilt
                )
            })?;
            // A logarithmic tilt through zero or a negative endpoint has no logarithm. Refusing
            // here beats handing back a NaN that a caller has to notice.
            if tilt == ParameterTilt::Log && (top <= 0.0 || base <= 0.0) {
                return Err(format!(
                    "{}: row {row_number} ({}) is tilted LOG between {top} and {base}; a logarithmic tilt needs two positive endpoints",
                    path.display(),
                    row.semantic_id
                ));
            }
        }
        // §5.4: `source` is mandatory for any numeric parameter, and a silently defaulted value is
        // not a legal state. A tilted value is numeric by construction even though it is an array.
        let numeric = row.value.is_number() || tilt != ParameterTilt::None;
        if numeric && row.source.as_deref().map(str::trim).unwrap_or_default().is_empty() {
            return Err(format!(
                "{}: row {row_number} ({}) carries a numeric value with no source; a silently defaulted parameter is not a legal state",
                path.display(),
                row.semantic_id
            ));
        }
        rows.push(ParameterPackRow {
            semantic_id: row.semantic_id,
            module_schema_version: row.module_schema_version,
            ordinal,
            display_label: row.display_label,
            value: row.value,
            unit: row.unit,
            source: row.source,
            tilt,
        });
    }

    Ok(ParameterPack {
        source_file: path.to_string_lossy().into_owned(),
        text_provenance,
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
    module_parameter_schema_from_spec(module)
}

/// Deterministic identity of a supplied manifest. Kept beside the shipping-module lookup so the
/// workflow can persist the exact manifest it already has in hand, and acceptance fixtures can
/// prove that changing a default changes the identity without registering a fake product module.
pub(crate) fn module_parameter_schema_from_spec(
    module: &crate::modules::ModuleSpec,
) -> Result<ParameterModuleSchema, String> {
    let module_name = module.name.as_str();
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

    /// SB-DBM-032 / SB-DBM-T32, the arms the one-handle refusal below does not reach. Sources:
    /// `22_database-model.md:1749-1752` (each value carries its unit and its `tilt`, and `tilt` is a
    /// property of the VALUE, not a display mode), `:1753` (ordinals are append-only and are never
    /// renumbered; retiring one leaves a gap), `§5.4` as routed at `:3217` (dual handle, `tilt`,
    /// **mandatory `source`**, append-only ordinals), and **F-19** at `:433-441`, which supplies the
    /// witness used here: `Rw` tilted logarithmically between 0.28 and 0.19 across a zone "is not
    /// 0.235". Within-zone-only interpolation and the step at a zone boundary are `:2953` and T32.
    ///
    /// **Why 0.235 is the whole point.** It is the LINEAR midpoint of the same two endpoints, so a
    /// tilt stored as a display mode — or a LOG tilt quietly evaluated linearly — returns a number
    /// that looks entirely reasonable and is wrong by 0.0043 ohm.m on `Rw`. That propagates straight
    /// into Sw. The chapter names the wrong answer rather than only the right one, so this test
    /// pins BOTH: the log answer must be produced, and the linear answer must not be.
    #[test]
    fn a_stored_parameter_carries_its_unit_and_source_and_a_tilted_value_never_interpolates_across_a_zone_boundary(
    ) {
        let temp = std::env::temp_dir().join(format!("sandibumi-tilt-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp).unwrap();
        let write = |name: &str, rows: serde_json::Value| -> std::path::PathBuf {
            let path = temp.join(name);
            std::fs::write(
                &path,
                serde_json::to_vec_pretty(&serde_json::json!({ "rows": rows })).unwrap(),
            )
            .unwrap();
            path
        };
        let row = |id: &str, ordinal: u32, extra: serde_json::Value| {
            let mut base = serde_json::json!({
                "semantic_id": id,
                "module_schema_version": "1",
                "ordinal": ordinal,
                "display_label": id,
            });
            let object = base.as_object_mut().unwrap();
            for (key, value) in extra.as_object().unwrap() {
                object.insert(key.clone(), value.clone());
            }
            base
        };

        // A. ROUND TRIP. The unit, the source and the tilt survive the load as fields of the row,
        //    which is what "a property of the value" means in storage terms.
        let tilted = write(
            "tilted.json",
            serde_json::json!([row(
                "RW",
                7,
                serde_json::json!({
                    "value": [0.28, 0.19],
                    "unit": "ohm.m",
                    "source": "F-19 worked example, 22_database-model.md:439",
                    "tilt": "LOG"
                })
            )]),
        );
        let pack = parse_parameter_pack_structure(&tilted).expect("a fully declared row must load");
        let rw = pack.by_ordinal(7).expect("the row keeps the ordinal it declared");
        assert_eq!(rw.unit.as_deref(), Some("ohm.m"));
        assert_eq!(rw.tilt, ParameterTilt::Log);
        assert!(rw.source.is_some(), "a numeric value must arrive with its source");

        // B. The chapter's own witness, pinned from BOTH sides. At the zone midpoint the LOG tilt
        //    is the geometric mean of the endpoints and the LINEAR tilt is the arithmetic one; the
        //    chapter states the log answer is NOT 0.235, so an implementation that ignored the tilt
        //    token would land exactly on the number it names as wrong.
        let (top, base) = (2000.0_f64, 2100.0_f64);
        let mid = rw.value_at_depth(top, base, 2050.0).expect("the midpoint is inside the zone");
        assert!(
            (mid - (0.28_f64 * 0.19).sqrt()).abs() < 1e-9,
            "a LOG tilt interpolates geometrically, got {mid}"
        );
        assert!(
            (mid - 0.235).abs() > 1e-3,
            "0.235 is the linear midpoint the chapter names as the wrong answer, got {mid}"
        );
        // Its endpoints are exact, or the interpolation is not anchored to the zone at all.
        assert!((rw.value_at_depth(top, base, top).unwrap() - 0.28).abs() < 1e-12);
        assert!((rw.value_at_depth(top, base, base).unwrap() - 0.19).abs() < 1e-12);

        // The linear twin on the same endpoints must produce the number the log twin must not, or
        // arm B proves only that some arithmetic happened.
        let linear = write(
            "linear.json",
            serde_json::json!([row(
                "RW",
                7,
                serde_json::json!({
                    "value": [0.28, 0.19],
                    "unit": "ohm.m",
                    "source": "F-19 worked example, 22_database-model.md:439",
                    "tilt": "LINEAR"
                })
            )]),
        );
        let linear_mid = parse_parameter_pack_structure(&linear)
            .unwrap()
            .by_ordinal(7)
            .unwrap()
            .value_at_depth(top, base, 2050.0)
            .unwrap();
        assert!((linear_mid - 0.235).abs() < 1e-12, "LINEAR midpoint is 0.235, got {linear_mid}");

        // C. WITHIN-ZONE ONLY, and the step at the boundary. A depth outside the zone has no value
        //    from this row at all — it is not extrapolated and not clamped — because the parameter
        //    belongs to the next zone there, and that zone carries its own endpoints.
        assert!(
            rw.value_at_depth(top, base, base + 0.1).is_none(),
            "interpolation is within-zone only; the sample below the base belongs to the next zone"
        );
        assert!(rw.value_at_depth(top, base, top - 0.1).is_none());

        // D. MANDATORY SOURCE for a numeric value. §5.4: a silently defaulted value is not a legal
        //    state, so the refusal happens at load rather than leaving an uncited number in a pack.
        let sourceless = write(
            "sourceless.json",
            serde_json::json!([row("RW", 7, serde_json::json!({ "value": 0.28, "unit": "ohm.m" }))]),
        );
        let err = parse_parameter_pack_structure(&sourceless)
            .expect_err("a numeric value with no source must be refused");
        assert!(err.contains("source") && err.contains("RW"), "refuse by name: {err}");

        // E. APPEND-ONLY ORDINALS. A gap is a RETIRED parameter and is legal; what must never
        //    happen is compaction, because renumbering is how ledger R-10's ClayVol #41 bound one
        //    parameter's value to another. Pinned by asking for the declared ordinal, not the
        //    position: under compaction 9 would have become 4 and this lookup would miss.
        let sparse = write(
            "sparse.json",
            serde_json::json!([
                row("A", 1, serde_json::json!({ "value": { "f": 1 } })),
                row("B", 2, serde_json::json!({ "value": { "f": 2 } })),
                row("C", 5, serde_json::json!({ "value": { "f": 3 } })),
                row("D", 9, serde_json::json!({ "value": { "f": 4 } })),
            ]),
        );
        let sparse = parse_parameter_pack_structure(&sparse).expect("a sparse pack is legal");
        assert_eq!(
            sparse.rows.iter().map(|r| r.ordinal).collect::<Vec<_>>(),
            vec![1, 2, 5, 9],
            "a gap marks a retired parameter and must survive the load uncompacted"
        );
        assert_eq!(sparse.by_ordinal(9).unwrap().semantic_id, "D");
        assert!(sparse.by_ordinal(3).is_none(), "a retired slot resolves to nothing, not to a neighbour");

        // F. The tilt declaration must be honest about its own shape. A tilt needs two endpoints to
        //    interpolate between; a scalar carrying a tilt token is a value that has already lost
        //    the physics the token claims, and an unknown token must never fall back to NONE.
        let scalar_tilt = write(
            "scalar-tilt.json",
            serde_json::json!([row(
                "RW",
                7,
                serde_json::json!({ "value": 0.28, "unit": "ohm.m", "source": "s", "tilt": "LOG" })
            )]),
        );
        let err = parse_parameter_pack_structure(&scalar_tilt)
            .expect_err("a tilted value must carry the two endpoints it interpolates between");
        assert!(err.contains("endpoint"), "{err}");

        let bad_token = write(
            "bad-token.json",
            serde_json::json!([row(
                "RW",
                7,
                serde_json::json!({ "value": [0.28, 0.19], "source": "s", "tilt": "SPLINE" })
            )]),
        );
        let err = parse_parameter_pack_structure(&bad_token)
            .expect_err("an unrecognised tilt must be refused, never silently treated as NONE");
        assert!(err.contains("SPLINE"), "the refusal must name the token it rejected: {err}");

        // A LOG tilt through zero or a negative endpoint has no logarithm; refusing at load beats
        // returning a NaN a caller would have to notice.
        let bad_log = write(
            "bad-log.json",
            serde_json::json!([row(
                "RW",
                7,
                serde_json::json!({ "value": [0.28, 0.0], "source": "s", "tilt": "LOG" })
            )]),
        );
        assert!(parse_parameter_pack_structure(&bad_log).is_err());
    }

    /// SB-DBM-032 / SB-DBM-T32. Source: `DEC-028` (2026-08-17) — **refuse BOTH one-handle forms.**
    /// A parameter row is addressed by a semantic identifier AND an ordinal; a row carrying only
    /// one of them is refused rather than loaded with a warning.
    ///
    /// **The expectation was CORRECTED, not the guard weakened.** The row as adjudicated expected a
    /// one-handle legacy row to load with a warning, which disagreed with the closed installer
    /// contract already refusing a missing ordinal (`SB-INS-015` / `SB-INS-T18`). Jauhar ruled the
    /// refusal stands and this row matches it. Nothing existing is loosened and no test is ignored.
    ///
    /// **Why an ordinal alone is the dangerous half.** A semantic identifier is a name — wrong, it
    /// fails to resolve and says so. An ordinal is a POSITION: reinterpreting a legacy row by
    /// position alone silently binds a value to whichever parameter now sits at that index, which
    /// computes, plots and ships. That is why neither handle may stand in for the pair.
    #[test]
    fn a_parameter_row_carrying_only_one_of_its_two_handles_is_refused_by_name() {
        let temp =
            std::env::temp_dir().join(format!("sandibumi-one-handle-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp).unwrap();
        let schema = module_parameter_schema("vsh_gr").expect("a shipping module owns its schema");
        let first = &schema.parameters[0];

        let write = |name: &str, row: serde_json::Value| -> std::path::PathBuf {
            let path = temp.join(name);
            let fixture = serde_json::json!({ "rows": [row] });
            std::fs::write(&path, serde_json::to_vec_pretty(&fixture).unwrap()).unwrap();
            path
        };

        // A. Semantic identifier only — no ordinal. Refused, and the message names the row.
        let semantic_only = write(
            "semantic-only.json",
            serde_json::json!({
                "semantic_id": first.semantic_id,
                "module_schema_version": schema.module_schema_version,
                "display_label": "Legacy one-handle row",
                "value": { "fixture": "semantic-only" }
            }),
        );
        let err = load_parameter_pack_for_module(&semantic_only, "vsh_gr")
            .expect_err("a row with no ordinal must be refused, never loaded with a warning");
        assert!(
            err.contains("ordinal"),
            "the refusal must name the missing handle: {err}"
        );

        // B. Ordinal only — empty semantic identifier. Refused for the same reason and separately,
        //    because this is the half that would otherwise bind a value BY POSITION.
        let ordinal_only = write(
            "ordinal-only.json",
            serde_json::json!({
                "semantic_id": "",
                "module_schema_version": schema.module_schema_version,
                "ordinal": first.ordinal,
                "display_label": "Legacy one-handle row",
                "value": { "fixture": "ordinal-only" }
            }),
        );
        let err = load_parameter_pack_for_module(&ordinal_only, "vsh_gr")
            .expect_err("a row with no semantic identifier must be refused");
        assert!(
            err.contains("semantic"),
            "the refusal must name the missing handle: {err}"
        );

        // C. Both handles present — loads. Without this the row would be satisfied by a loader
        //    that refuses everything, which is not the contract.
        let complete = write(
            "both-handles.json",
            serde_json::json!({
                "semantic_id": first.semantic_id,
                "module_schema_version": schema.module_schema_version,
                "ordinal": first.ordinal,
                "display_label": "Complete row",
                "value": { "fixture": "complete" }
            }),
        );
        let pack = load_parameter_pack_for_module(&complete, "vsh_gr")
            .expect("a row carrying both handles is well formed and must load");

        // D. The two handles must agree with EACH OTHER, and that existing refusal is untouched:
        //    a lookup pairing this row's semantic identifier with a different row's ordinal
        //    resolves to nothing rather than to whichever row matched one half.
        assert!(pack.by_key(&first.semantic_id, first.ordinal).is_some());
        let other_ordinal = schema.parameters[1].ordinal;
        assert_ne!(other_ordinal, first.ordinal, "the fixture needs two distinct ordinals");
        assert!(
            pack.by_key(&first.semantic_id, other_ordinal).is_none(),
            "a semantic identifier paired with a foreign ordinal must not resolve"
        );

        let _ = std::fs::remove_dir_all(&temp);
    }

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

    /// CORRECTNESS — SB-INS-017 / SB-INS-T22. The CP1252 byte `0x92`, explicit CP1252
    /// declaration, and required exported byte/encoding provenance come from dossier sections
    /// 2.1 and 3.9 plus N-NEW-12. A contradictory declaration is the opposite-side control:
    /// merely auto-decoding the same bytes must not make the declaration ceremonial.
    #[test]
    fn a_declared_cp1252_pack_records_its_decoded_encoding_and_original_byte_representation_in_exported_provenance(
    ) {
        let temp = std::env::temp_dir().join(format!(
            "sandibumi-parameter-encoding-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&temp).unwrap();
        let schema = module_parameter_schema("vsh_gr").expect("a shipping module owns its schema");
        let parameter = &schema.parameters[0];
        let encoded_fixture = |declared_encoding: &str| {
            let fixture = serde_json::json!({
                "text_encoding": declared_encoding,
                "rows": [{
                    "semantic_id": parameter.semantic_id,
                    "module_schema_version": schema.module_schema_version,
                    "ordinal": parameter.ordinal,
                    "display_label": "Observed ’ label",
                    "value": { "fixture": true }
                }]
            });
            let mut utf8 = serde_json::to_vec_pretty(&fixture).unwrap();
            let position = utf8
                .windows(3)
                .position(|window| window == [0xE2, 0x80, 0x99])
                .expect("fixture contains the CP1252 smart-apostrophe character");
            utf8.splice(position..position + 3, [0x92]);
            utf8
        };

        let body = encoded_fixture("CP1252");
        let path = temp.join("declared-cp1252.json");
        std::fs::write(&path, &body).unwrap();
        let pack = load_parameter_pack_for_module(&path, "vsh_gr")
            .expect("the matching explicit encoding declaration must load");
        assert_eq!(
            pack.text_provenance.declared_encoding.as_deref(),
            Some("CP1252")
        );
        assert_eq!(pack.text_provenance.decoded_encoding, "Windows-1252");
        let reconstructed = pack
            .text_provenance
            .original_bytes_hex
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                let digits = std::str::from_utf8(pair).unwrap();
                u8::from_str_radix(digits, 16).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(reconstructed, body, "the exported hex must represent every source byte");
        assert!(reconstructed.contains(&0x92));

        let exported = serde_json::to_value(&pack).expect("the product result exports provenance");
        assert_eq!(
            exported["text_provenance"]["declared_encoding"],
            "CP1252"
        );
        assert_eq!(
            exported["text_provenance"]["decoded_encoding"],
            "Windows-1252"
        );
        assert_eq!(
            exported["text_provenance"]["original_bytes_hex"],
            pack.text_provenance.original_bytes_hex
        );

        let mismatch = temp.join("contradictory-utf8.json");
        std::fs::write(&mismatch, encoded_fixture("UTF-8")).unwrap();
        let error = load_parameter_pack_for_module(&mismatch, "vsh_gr").unwrap_err();
        assert!(error.contains("declares text encoding UTF-8"), "{error}");
        assert!(error.contains("decoded as Windows-1252"), "{error}");

        std::fs::remove_dir_all(&temp).unwrap();
    }
}
