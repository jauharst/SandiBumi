//! LAS 2.0 export: one well's standard + computed curves on the standard depth grid.
//! NaN (missing) writes as the project's declared export sentinel.

use crate::equations::fetch_curve_frame;
use duckdb::{params, Connection};
use sha2::{Digest, Sha256};
use std::io::{BufWriter, Write};

/// CWLS-conventional LAS null, cited in `docs/PRD_v2/21_data-io.md` §5.2. It is the
/// project default, not a writer-owned constant: every registered writer receives the
/// project's resolved setting through [`WriterSettings`].
pub const DEFAULT_NULL_SENTINEL: f32 = -999.25;

const SETTINGS_DOC_TYPE: &str = "settings";
const SETTINGS_DOC_NAME: &str = "data-io";

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
struct DataIoSettings {
    null_sentinel: f32,
}

/// Required context for every data writer. The registry function type takes this value
/// by value and not as an `Option`, so a writer that omits the sentinel cannot register.
#[derive(Debug, Clone, Copy)]
struct WriterSettings {
    null_sentinel: f32,
}

type WriterFn = fn(&Connection, &str, &str, WriterSettings) -> Result<LasExportResult, String>;
type SelfReaderFn = fn(&Connection, &str, &LasExportResult) -> Result<(), String>;

#[derive(Debug, Clone, Copy)]
enum SentinelSupport {
    Honours,
    #[allow(dead_code)] // no incapable format ships yet; registration still has to declare the case
    Incapable(&'static str),
}

struct RegisteredWriter {
    id: &'static str,
    label: &'static str,
    extension: &'static str,
    is_default: bool,
    sentinel_support: SentinelSupport,
    write: WriterFn,
    self_read: SelfReaderFn,
}

const REGISTERED_WRITERS: &[RegisteredWriter] = &[RegisteredWriter {
    id: "las-2.0",
    label: "LAS 2.0",
    extension: "las",
    is_default: true,
    sentinel_support: SentinelSupport::Honours,
    write: write_las,
    self_read: validate_las_output,
}];

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportFormatInfo {
    pub id: String,
    pub label: String,
    pub extension: String,
    pub is_default: bool,
    pub honours_project_sentinel: bool,
    /// Present for a format that cannot honour the project sentinel. A format picker
    /// displays this instead of presenting the format as equivalent to a capable one.
    pub sentinel_limitation: Option<String>,
}

fn format_info(writer: &RegisteredWriter) -> ExportFormatInfo {
    let (honours_project_sentinel, sentinel_limitation) = match writer.sentinel_support {
        SentinelSupport::Honours => (true, None),
        SentinelSupport::Incapable(reason) => (false, Some(reason.to_string())),
    };
    ExportFormatInfo {
        id: writer.id.to_string(),
        label: writer.label.to_string(),
        extension: writer.extension.to_string(),
        is_default: writer.is_default,
        honours_project_sentinel,
        sentinel_limitation,
    }
}

pub fn export_formats() -> Vec<ExportFormatInfo> {
    REGISTERED_WRITERS.iter().map(format_info).collect()
}

fn default_writer() -> Result<&'static RegisteredWriter, String> {
    let writer = REGISTERED_WRITERS
        .iter()
        .find(|writer| writer.is_default)
        .ok_or_else(|| "no default data export format is registered".to_string())?;
    match writer.sentinel_support {
        SentinelSupport::Honours => Ok(writer),
        SentinelSupport::Incapable(reason) => Err(format!(
            "the default export format '{}' cannot honour the project sentinel: {reason}",
            writer.label
        )),
    }
}

/// The project's one declared export sentinel. Older projects have no data-I/O settings
/// document and therefore resolve to the cited CWLS convention.
pub fn project_null_sentinel(conn: &Connection) -> Result<f32, String> {
    let json: Option<String> = conn
        .query_row(
            "SELECT json FROM documents WHERE doc_type = ?1 AND name = ?2",
            params![SETTINGS_DOC_TYPE, SETTINGS_DOC_NAME],
            |row| row.get(0),
        )
        .ok();
    let Some(json) = json else { return Ok(DEFAULT_NULL_SENTINEL) ;
    };
    let settings: DataIoSettings = serde_json::from_str(&json)
        .map_err(|e| format!("invalid project data-I/O settings: {e}"))?;
    if !settings.null_sentinel.is_finite() {
        return Err("the project export sentinel must be finite".into());
    }
    Ok(settings.null_sentinel)
}

/// Sets the project-wide export sentinel through the existing whitelisted document writer.
pub fn set_project_null_sentinel(conn: &Connection, null_sentinel: f32) -> Result<(), String> {
    if !null_sentinel.is_finite() {
        return Err("the project export sentinel must be finite".into());
    }
    let json = serde_json::to_string(&DataIoSettings { null_sentinel }).map_err(|e| e.to_string())?;
    crate::db::save_document(conn, SETTINGS_DOC_TYPE, SETTINGS_DOC_NAME, &json)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

const PROVENANCE_PREFIX: &str = "SANDIBUMI_PROVENANCE_V1 ";
const PROVENANCE_SUMMARY_PREFIX: &str = "SANDIBUMI_PROVENANCE_SUMMARY_V1 ";
const MODEL_PROVENANCE_PREFIX: &str = "SANDIBUMI_MODEL_PROVENANCE_V1 ";
const OMISSION_PREFIX: &str = "SANDIBUMI_OMISSION_V1 ";
const PRECISION_PREFIX: &str = "SANDIBUMI_PRECISION_V1 ";
const CURVE_STATE_PREFIX: &str = "SANDIBUMI_CURVE_STATE_V1 ";

#[derive(Debug, Clone, serde::Serialize)]
pub struct LasOmission {
    pub curve: String,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LasCurveState {
    /// Mnemonic written into this LAS. It may carry a `_FINAL`/`_WORKING` suffix when
    /// two generic sets hold the same source mnemonic and both must remain addressable.
    pub export_curve: String,
    pub source_curve: String,
    pub set_name: String,
    pub state: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LasExportResult {
    pub rows: usize,
    pub curves_written: usize,
    pub curves_held: usize,
    pub omitted: Vec<LasOmission>,
    pub curve_states: Vec<LasCurveState>,
    /// Number of exported computed curve identities whose stored rows do not resolve to a run
    /// record. They remain in the file only under the explicit LEGACY_UNRECORDED label.
    pub legacy_unrecorded_curves: usize,
    /// LAS text is currently written at four decimal places. This report declares that
    /// boundary and counts the stored f32 values whose written representation changes.
    pub precision: crate::parsers::SamplePrecisionReport,
    /// True only after the registered reader accepted the completed file and its declared
    /// depth unit, row count, and curve count matched what the writer says it emitted.
    pub self_checked: bool,
    /// Set when the index is not uniformly sampled and `STEP` was therefore written as `0`.
    /// Names the first depth at which the spacing changes and both spacings, so the user can
    /// tell real drift from a delivery defect. `None` means the index is uniform and `STEP`
    /// carries its real value — a silent `0` would be a degraded result presented as clean.
    pub nonuniform_step: Option<String>,
}

fn fixed_decimal_4_reduces(value: f32) -> bool {
    value.is_finite()
        && format!("{value:.4}")
            .parse::<f32>()
            .is_ok_and(|written| written != value)
}

/// `curve_meta` declares FINAL as the QC'd delivery set; RAW, EDIT and other named
/// import/work sets have not been designated final.
fn curve_state_for_set(set_name: &str) -> &'static str {
    if set_name.trim().eq_ignore_ascii_case("FINAL") { "final" } else { "working" }
}

fn collect_model_ids(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(id) = map.get("model_id").and_then(serde_json::Value::as_str) {
                if !out.iter().any(|seen| seen == id) {
                    out.push(id.to_string());
                }
            }
            for child in map.values() {
                collect_model_ids(child, out);
            }
        }
        serde_json::Value::Array(values) => {
            for child in values {
                collect_model_ids(child, out);
            }
        }
        _ => {}
    }
}

/// LAS 2.0 leaves `~O` as free text. SandiBumi's house convention is one compact JSON object per
/// prefixed line: still readable without software, and independently parseable without relying on
/// punctuation in a prose sentence. The prefix versions the convention, not the run record.
fn provenance_lines(
    conn: &Connection,
    well_id: &str,
    curve_names: &[String],
) -> Result<(Vec<String>, usize), String> {
    let standard = ["GR", "RES_DEEP", "NPHI", "RHOB", "DT", "SP"];
    let mut lines = Vec::new();
    let mut legacy_curves = std::collections::BTreeSet::new();
    let mut groups_by_curve = std::collections::BTreeMap::<
        String,
        Vec<crate::equations::ComputedProvenanceGroup>,
    >::new();
    for group in crate::equations::computed_provenance_groups(conn, well_id)? {
        groups_by_curve
            .entry(group.curve_name.trim().to_uppercase())
            .or_default()
            .push(group);
    }

    for name in curve_names {
        let upper = name.trim().to_uppercase();
        if standard.contains(&upper.as_str()) {
            lines.push(format!(
                "{PROVENANCE_PREFIX}{}",
                serde_json::json!({ "curve": upper, "origin": "measured" })
            ));
            continue;
        }
        let groups = groups_by_curve.remove(&upper).unwrap_or_default();
        if groups.is_empty() {
            lines.push(format!(
                "{PROVENANCE_PREFIX}{}",
                serde_json::json!({ "curve": upper, "origin": "measured" })
            ));
            continue;
        }

        for group in groups {
            if group.provenance_class
                == crate::equations::ComputedProvenanceClass::LegacyUnrecorded
            {
                legacy_curves.insert(upper.clone());
                lines.push(format!(
                    "{PROVENANCE_PREFIX}{}",
                    serde_json::json!({
                        "curve": upper,
                        "origin": "computed",
                        "provenance_class": crate::equations::LEGACY_UNRECORDED,
                        "row_count": group.row_count,
                        "state": "unprovenanced"
                    })
                ));
                continue;
            }

            let set_id = group.set_id.as_deref().expect("recorded groups carry a set id");
            let (set_name, version, module, params_json, inputs_json, custody, created_at): (
                String,
                i64,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                String,
            ) = conn
                .query_row(
                    "SELECT set_name, version, module, params_json, inputs_json, comment,
                            strftime(created_at, '%Y-%m-%d %H:%M')
                     FROM log_sets WHERE set_id = ?1",
                    params![set_id],
                    |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                        ))
                    },
                )
                .map_err(|_| {
                    format!(
                        "computed curve '{name}' cites missing log-set record '{set_id}'; export refused"
                    )
                })?;
            let params_text = params_json
                .as_deref()
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .ok_or_else(|| {
                    format!("computed curve '{name}' has no recorded parameters; export refused")
                })?;
            let parameters: serde_json::Value = serde_json::from_str(params_text)
                .map_err(|e| format!("computed curve '{name}' has invalid parameter JSON: {e}"))?;
            let inputs: serde_json::Value = match inputs_json.as_deref().map(str::trim) {
                Some(text) if !text.is_empty() => serde_json::from_str(text)
                    .map_err(|e| format!("computed curve '{name}' has invalid input JSON: {e}"))?,
                _ => serde_json::Value::Array(Vec::new()),
            };
            let ancestry = crate::equations::parse_curve_ancestry(params_text).map_err(|error| {
                format!(
                    "computed curve '{name}' cannot be exported without its complete ancestry: {error}"
                )
            })?;
            let state = curve_state_for_set(&set_name);
            let mut record = serde_json::json!({
                "curve": upper,
                "origin": "computed",
                "provenance_class": "RECORDED",
                "row_count": group.row_count,
                "method": module,
                "parameters": parameters,
                "inputs": inputs,
                "log_set": set_name,
                "version": version,
                "run_date": created_at,
                "state": state,
                "ancestry": ancestry,
            });
            // SB-POR-003: the run's custody comment (branches taken, limits bound) survives
            // export on the same `~O` line as the parameters it qualifies.
            if let Some(text) = custody.as_deref().map(str::trim).filter(|text| !text.is_empty())
            {
                record["custody"] = serde_json::json!(text);
            }
            lines.push(format!("{PROVENANCE_PREFIX}{record}"));

            if module.starts_with("ml:") {
                let mut model_ids = Vec::new();
                collect_model_ids(&parameters, &mut model_ids);
                if model_ids.is_empty() {
                    return Err(format!(
                        "model-derived curve '{name}' has no saved model identity in its run record; export refused"
                    ));
                }
                for model_id in model_ids {
                    let (info, artifact) = crate::db::get_ml_model(conn, &model_id).map_err(|_| {
                        format!(
                            "model-derived curve '{name}' cites unavailable model '{model_id}'; export refused"
                        )
                    })?;
                    let mut record = serde_json::to_value(info).map_err(|e| e.to_string())?;
                    let object = record.as_object_mut().ok_or_else(|| {
                        "saved model record did not serialize as an object".to_string()
                    })?;
                    object.insert(
                        "artifact_sha256".into(),
                        serde_json::Value::String(format!("{:x}", Sha256::digest(&artifact))),
                    );
                    lines.push(format!(
                        "{MODEL_PROVENANCE_PREFIX}{}",
                        serde_json::json!({ "curve": upper, "record": record })
                    ));
                }
            }
        }
    }
    Ok((lines, legacy_curves.len()))
}

/// Adds generic-store curves only when their exact samples fit the LAS frame. Resolution is by
/// `curve_id`, never by family, so a duplicate mnemonic cannot cause another delivery's data to be
/// written under this curve's label. Every curve not added becomes a named omission.
fn add_generic_curves(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    curve_names: &mut Vec<String>,
    units: &mut Vec<String>,
    columns: &mut std::collections::HashMap<String, Vec<f32>>,
) -> Result<(usize, Vec<LasOmission>, Vec<LasCurveState>), String> {
    let mut stmt = conn
        .prepare(
            "SELECT curve_id, upper(mnemonic), COALESCE(unit, ''), set_name, run_no
             FROM curve_meta WHERE well_id = ?1
             ORDER BY (set_name = 'RAW') DESC, upper(mnemonic), set_name,
                      run_no NULLS FIRST, curve_id",
        )
        .map_err(|e| e.to_string())?;
    let metas: Vec<(String, String, String, String, Option<i32>)> = stmt
        .query_map(params![well_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<duckdb::Result<_>>()
        .map_err(|e| e.to_string())?;
    let held = metas.len();
    let depth_index: std::collections::HashMap<u32, usize> =
        depth.iter().enumerate().map(|(i, d)| (d.to_bits(), i)).collect();
    let mut sample_stmt = conn
        .prepare("SELECT depth, value FROM curve_samples WHERE curve_id = ?1 ORDER BY depth")
        .map_err(|e| e.to_string())?;
    let mut omitted = Vec::new();
    let mut curve_states = Vec::new();
    let mut first_generic_state = std::collections::HashMap::<String, String>::new();

    for (curve_id, mnemonic, unit, set_name, run_no) in metas {
        let label = match run_no {
            Some(run) => format!("{mnemonic} [set {set_name}, run {run}]"),
            None => format!("{mnemonic} [set {set_name}]"),
        };
        let omit = |reason: &str| LasOmission { curve: label.clone(), reason: reason.to_string() };
        if mnemonic == "DEPT" || mnemonic == "DEPTH" {
            omitted.push(omit("the well index is already written as the LAS DEPT column"));
            continue;
        }
        let state = curve_state_for_set(&set_name);
        let mut export_mnemonic = mnemonic.clone();
        if curve_names.iter().any(|name| name.eq_ignore_ascii_case(&mnemonic)) {
            let Some(existing_state) = first_generic_state.get(&mnemonic) else {
                omitted.push(omit(
                    "that LAS mnemonic is already written from another held standard, computed, set or run curve",
                ));
                continue;
            };
            if existing_state == state {
                omitted.push(omit(
                    "that LAS mnemonic is already written from another held standard, computed, set or run curve",
                ));
                continue;
            }
            let candidate = format!("{mnemonic}_{}", state.to_ascii_uppercase());
            if curve_names.iter().any(|name| name.eq_ignore_ascii_case(&candidate)) {
                omitted.push(omit(
                    "its final/working export mnemonic is already held by another curve",
                ));
                continue;
            }
            export_mnemonic = candidate;
        }
        let samples: Vec<(f32, f32)> = sample_stmt
            .query_map(params![curve_id], |row| {
                Ok((row.get::<_, f32>(0)?, row.get::<_, Option<f32>>(1)?.unwrap_or(f32::NAN)))
            })
            .map_err(|e| e.to_string())?
            .collect::<duckdb::Result<_>>()
            .map_err(|e| e.to_string())?;
        if samples.is_empty() {
            omitted.push(omit("the held curve has no samples"));
            continue;
        }
        if samples.iter().any(|(d, _)| !depth_index.contains_key(&d.to_bits())) {
            omitted.push(omit(
                "its samples are on a different depth frame; this LAS writes the well's standard frame without re-gridding",
            ));
            continue;
        }
        let mut values = vec![f32::NAN; depth.len()];
        for (d, value) in samples {
            values[depth_index[&d.to_bits()]] = value;
        }
        columns.insert(export_mnemonic.clone(), values);
        curve_names.push(export_mnemonic.clone());
        units.push(unit);
        first_generic_state.entry(mnemonic.clone()).or_insert_with(|| state.to_string());
        curve_states.push(LasCurveState {
            export_curve: export_mnemonic,
            source_curve: mnemonic,
            set_name,
            state: state.to_string(),
        });
    }
    Ok((held, omitted, curve_states))
}

pub fn export_las(conn: &Connection, well_id: &str, dest_path: &str) -> Result<LasExportResult, String> {
    let settings = WriterSettings { null_sentinel: project_null_sentinel(conn)? };
    let writer = default_writer()?;
    export_with_writer(conn, well_id, dest_path, settings, writer)
}

fn export_with_writer(
    conn: &Connection,
    well_id: &str,
    dest_path: &str,
    settings: WriterSettings,
    writer: &RegisteredWriter,
) -> Result<LasExportResult, String> {
    let mut result = (writer.write)(conn, well_id, dest_path, settings)?;
    (writer.self_read)(conn, dest_path, &result)?;
    result.self_checked = true;
    Ok(result)
}

/// Reads a completed LAS with the same full-curve reader used by import, then verifies
/// the semantic declarations that a syntax-only parse cannot: a feet project labelled
/// as metres is readable but still wrong by 3.28×.
fn validate_las_output(
    conn: &Connection,
    dest_path: &str,
    result: &LasExportResult,
) -> Result<(), String> {
    let frame = crate::parsers::parse_las_2_all(dest_path)
        .map_err(|error| format!("LAS self-check failed: SandiBumi's LAS reader rejected the output: {error}"))?;
    let expected_unit = crate::units::require_project_depth_unit(conn, "LAS self-check")?;
    let written_unit = frame
        .depth_unit
        .as_deref()
        .and_then(crate::units::DepthUnit::parse)
        .ok_or_else(|| {
            format!(
                "LAS self-check failed: the written depth unit {:?} is missing or unrecognized",
                frame.depth_unit
            )
        })?;
    if written_unit != expected_unit {
        return Err(format!(
            "LAS self-check failed: project depths are {}, but the written DEPT curve declares {}",
            expected_unit.code(),
            written_unit.code()
        ));
    }
    if frame.depth.len() != result.rows {
        return Err(format!(
            "LAS self-check failed: writer reported {} row(s), but its reader found {}",
            result.rows,
            frame.depth.len()
        ));
    }
    if frame.curves.len() != result.curves_written {
        return Err(format!(
            "LAS self-check failed: writer reported {} curve(s), but its reader found {}",
            result.curves_written,
            frame.curves.len()
        ));
    }
    if let Some(curve) = frame.curves.iter().find(|curve| curve.values.len() != result.rows) {
        return Err(format!(
            "LAS self-check failed: curve {} has {} value(s), expected {}",
            curve.mnemonic,
            curve.values.len(),
            result.rows
        ));
    }
    Ok(())
}

fn write_las(
    conn: &Connection,
    well_id: &str,
    dest_path: &str,
    settings: WriterSettings,
) -> Result<LasExportResult, String> {
    let (well_name, field_name): (String, Option<String>) = conn
        .query_row(
            "SELECT well_name, field_name FROM wells WHERE well_id = ?1",
            params![well_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    // Every curve this well actually has: the six standard ones + its computed curves.
    let mut curve_names: Vec<String> =
        ["GR", "RES_DEEP", "NPHI", "RHOB", "DT", "SP"].iter().map(|s| s.to_string()).collect();
    let mut units: Vec<String> = curve_names
        .iter()
        .map(|name| crate::curves::canonical_unit(name).unwrap_or("").to_string())
        .collect();
    {
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT cc.curve_name, COALESCE(e.output_units, '')
                 FROM computed_curves cc
                 LEFT JOIN equations e ON e.output_curve = cc.curve_name
                 WHERE cc.well_id = ?1 ORDER BY cc.curve_name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![well_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        for r in rows {
            let (name, unit) = r.map_err(|e| e.to_string())?;
            if crate::schema_vocab::standard_column(&name).is_some() {
                return Err(format!(
                    "computed curve '{name}' shadows a measured standard curve; LAS export refused because the deliverable cannot distinguish their provenance"
                ));
            }
            // A unit DECLARED by whatever wrote the curve beats one inferred from an equation of
            // the same name (SB-MLA-035). The case this exists for is a prediction fitted in log
            // space: exported with a blank unit — or worse, with the units of the quantity it is a
            // logarithm OF — a log10(mD) column reads as a permeability, and every negative value
            // in it reads as a physically impossible one rather than as a number below 1 mD.
            let unit = crate::db::curve_unit_for(conn, well_id, &name).unwrap_or(unit);
            curve_names.push(name);
            units.push(unit);
        }
    }

    let (depth, mut columns) = fetch_curve_frame(conn, well_id, &curve_names).map_err(|e| e.to_string())?;
    if depth.is_empty() {
        return Err("well has no curve data".into());
    }
    let initially_written = curve_names.len();
    let (generic_held, omitted, curve_states) = add_generic_curves(
        conn,
        well_id,
        &depth,
        &mut curve_names,
        &mut units,
        &mut columns,
    )?;
    let curves_held = initially_written + generic_held;
    let depth_unit = crate::units::require_project_depth_unit(conn, "LAS export")?.code();
    let (provenance, legacy_unrecorded_curves) =
        provenance_lines(conn, well_id, &curve_names)?;
    // SB-DIO-056: the step is verified across the WHOLE index, on the fixed-decimal-4 text this
    // writer is about to emit, and is declared as 0 when it varies — LAS 2.0's own provision for
    // a non-uniform index (21_data-io.md:867). Declaring depth[1] - depth[0] tells a reader the
    // file is uniformly sampled when it is not, and a conforming reader is then entitled to
    // rebuild depths from STRT/STEP — silently re-gridding the data.
    let written_depths: Vec<String> = depth.iter().map(|value| format!("{value:.4}")).collect();
    let (step, nonuniform_step) = match crate::parsers::verify_written_step(&written_depths) {
        crate::parsers::WrittenStep::Uniform(step) => (step.parse::<f32>().unwrap_or(0.0), None),
        crate::parsers::WrittenStep::NoInterval => (0.0, None),
        crate::parsers::WrittenStep::Varies(note) => (0.0, Some(note)),
    };
    let mut values_reduced = depth.iter().filter(|&&value| fixed_decimal_4_reduces(value)).count();
    for name in &curve_names {
        let key = name.trim().to_uppercase();
        if let Some(values) = columns.get(&key) {
            values_reduced += values.iter().filter(|&&value| fixed_decimal_4_reduces(value)).count();
        }
    }
    let precision = crate::parsers::SamplePrecisionReport::new(
        "f32 storage",
        "fixed-decimal-4 LAS text",
        values_reduced,
    );

    let file = std::fs::File::create(dest_path).map_err(|e| e.to_string())?;
    let mut w = BufWriter::new(file);
    let null_sentinel = settings.null_sentinel;
    let fmt = |v: f32| -> String {
        if v.is_nan() {
            format!("{null_sentinel:.4}")
        } else {
            format!("{v:.4}")
        }
    };

    let write = || -> std::io::Result<()> {
        writeln!(w, "~Version Information")?;
        writeln!(w, " VERS.                 2.0 : CWLS log ASCII Standard - VERSION 2.0")?;
        writeln!(w, " WRAP.                  NO : One line per depth step")?;
        writeln!(w, "~Well Information")?;
        writeln!(w, " STRT.{depth_unit}   {:>12.4} : START DEPTH", depth[0])?;
        writeln!(w, " STOP.{depth_unit}   {:>12.4} : STOP DEPTH", depth[depth.len() - 1])?;
        writeln!(w, " STEP.{depth_unit}   {:>12.4} : STEP", step)?;
        writeln!(w, " NULL.    {:>12.4} : NULL VALUE", null_sentinel)?;
        writeln!(w, " WELL.    {} : WELL NAME", well_name)?;
        writeln!(w, " FLD .    {} : FIELD", field_name.unwrap_or_default())?;
        writeln!(w, " SRVC.    SandiBumi : EXPORTED BY")?;
        writeln!(w, "~Curve Information")?;
        writeln!(w, " DEPT.{depth_unit}                     : Depth")?;
        for (name, unit) in curve_names.iter().zip(units.iter()) {
            writeln!(w, " {:<8}.{:<8}          : {}", name, unit, name)?;
        }
        writeln!(w, "~Other Information")?;
        writeln!(w, "# SandiBumi provenance: prefixed JSON Lines; convention version 1")?;
        writeln!(
            w,
            " {PROVENANCE_SUMMARY_PREFIX}{}",
            serde_json::json!({
                "legacy_unrecorded_curves": legacy_unrecorded_curves
            })
        )?;
        writeln!(
            w,
            " {PRECISION_PREFIX}{}",
            serde_json::to_string(&precision).expect("serializing a precision report cannot fail")
        )?;
        for line in &provenance {
            writeln!(w, " {line}")?;
        }
        for state in &curve_states {
            writeln!(
                w,
                " {CURVE_STATE_PREFIX}{}",
                serde_json::to_string(state).expect("serializing a curve state cannot fail")
            )?;
        }
        for omission in &omitted {
            writeln!(
                w,
                " {OMISSION_PREFIX}{}",
                serde_json::to_string(omission).expect("serializing two strings cannot fail")
            )?;
        }
        writeln!(w, "~ASCII")?;
        for i in 0..depth.len() {
            let mut line = format!("{:>12.4}", depth[i]);
            for name in &curve_names {
                // fetch_curve_frame keys the column map by name.trim().to_uppercase(); a mixed-case
                // computed curve (e.g. "Vsh_final") missed here and exported an all-NULL column.
                let key = name.trim().to_uppercase();
                let v = columns.get(&key).map(|c| *c.get(i).unwrap_or(&f32::NAN)).unwrap_or(f32::NAN);
                line.push_str(&format!(" {:>12}", fmt(v)));
            }
            writeln!(w, "{line}")?;
        }
        w.flush()
    };
    write().map_err(|e| e.to_string())?;
    Ok(LasExportResult {
        rows: depth.len(),
        curves_written: curve_names.len(),
        curves_held,
        omitted,
        curve_states,
        legacy_unrecorded_curves,
        precision,
        self_checked: false,
        nonuniform_step,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use uuid::Uuid;

    fn tmp_path(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("sandibumi-export-{}-{}.las", tag, std::process::id()))
    }

    /// CHARACTERIZATION — **Characterizes thirty wrapped LAS curves as aligned and every LAS
    /// export as unwrapped.** `SB-DIO-040` / `SB-DIO-T58`. Source: 21_data-io.md D-24 and T58
    /// specify the 30-curve `WRAP.YES` fixture; the chapter explicitly classifies T58 as `char`.
    #[test]
    fn characterizes_thirty_wrapped_las_curves_as_aligned_and_every_las_export_as_unwrapped() {
        let source_path = tmp_path("wrapped-thirty-source");
        let export_path = tmp_path("wrapped-thirty-export");
        let mut las = String::from(
            "~Version Information\nVERS. 2.0\nWRAP. YES\n~Well Information\nWELL. WRAPPED_THIRTY_CURVES\nNULL. -999.25\n~Curve Information\nDEPT.M : Depth\n",
        );
        for curve in 1..=30 {
            las.push_str(&format!("C{curve:02}.UNIT : Wrapped curve {curve:02}\n"));
        }
        las.push_str("~ASCII\n");
        for row in 1..=2 {
            let mut tokens = vec![format!("{}", 1000 + row)];
            tokens.extend((1..=30).map(|curve| format!("{}", curve * 100 + row)));
            for chunk in tokens.chunks(7) {
                las.push_str(&chunk.join(" "));
                las.push('\n');
            }
        }
        std::fs::write(&source_path, las).unwrap();

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let result = crate::ingest::import_las_files(
            &conn,
            &[source_path.to_string_lossy().to_string()],
            None,
        )
        .remove(0);
        assert!(result.error.is_none(), "{:?}", result.error);
        let well_id = result.well_id.expect("the wrapped delivery creates one imported object");
        for curve in 1..=30 {
            let mnemonic = format!("C{curve:02}");
            let samples: Vec<(f32, f32)> = {
                let mut statement = conn
                    .prepare(
                        "SELECT s.depth, s.value FROM curve_samples s
                         JOIN curve_meta m ON m.curve_id = s.curve_id
                         WHERE m.well_id = ?1 AND m.mnemonic = ?2 ORDER BY s.depth",
                    )
                    .unwrap();
                statement
                    .query_map(params![&well_id, &mnemonic], |row| Ok((row.get(0)?, row.get(1)?)))
                    .unwrap()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap()
            };
            assert_eq!(
                samples,
                vec![(1001.0, (curve * 100 + 1) as f32), (1002.0, (curve * 100 + 2) as f32)],
                "{mnemonic} must keep both uniquely identifiable source-column values",
            );
        }

        let exported = export_las(&conn, &well_id, export_path.to_str().unwrap()).unwrap();
        let text = crate::parsers::read_text_file(&export_path).unwrap();
        let wrap = text
            .lines()
            .find(|line| line.trim_start().starts_with("WRAP."))
            .expect("the LAS version block declares its wrapping mode");
        assert!(wrap.split(':').next().unwrap_or("").split_whitespace().any(|token| token == "NO"));
        let data_rows: Vec<&str> = text
            .split("~ASCII")
            .nth(1)
            .expect("the writer emits an ASCII data section")
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        assert_eq!(data_rows.len(), 2, "one physical line must carry each complete logical depth row");
        assert!(
            data_rows
                .iter()
                .all(|line| line.split_whitespace().count() == exported.curves_written + 1),
            "each physical row must contain depth plus every exported curve",
        );

        std::fs::remove_file(source_path).unwrap();
        std::fs::remove_file(export_path).unwrap();
    }

    /// A well with one missing sample per curve, plus a MIXED-CASE computed curve.
    fn seed(conn: &Connection) -> (Uuid, Vec<f32>, Vec<f32>) {
        db::create_schema(conn).unwrap();
        crate::units::set_project_depth_unit(conn, crate::units::DepthUnit::Metres).unwrap();
        let id = Uuid::new_v4();
        db::insert_well(conn, id, "SANDI-EXP", Some("Synthetic"), None, None).unwrap();

        let depth: Vec<f32> = (0..6).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let gr = vec![40.0f32, 55.5, f32::NAN, 88.25, 120.0, 33.125];
        let res = vec![2.0f32; 6];
        let nphi = vec![0.30f32; 6];
        let rhob = vec![2.45f32; 6];
        let nan = vec![f32::NAN; 6];
        db::insert_standard_curves(
            conn,
            id,
            depth.clone(),
            gr.clone(),
            res,
            nphi,
            rhob,
            nan.clone(),
            nan,
        )
        .unwrap();

        // Deliberately mixed case — see the regression note below.
        let vsh = vec![0.1f32, 0.25, 0.5, f32::NAN, 0.75, 0.9];
        let custody = crate::workflow::test_run_custody();
        let spec = crate::equations::complete_curve_run_spec(
            conn,
            &id.to_string(),
            "INTERP",
                "vsh_gr",
            &custody,
            &[(id.to_string(), "GR".into(),
                "GR".into())],
            None,
            serde_json::json!({ "gr_clean": 25.0, "gr_shale": 125.0 }),
                crate::equations::AncestryZoneScope::WholeWell,
            &["Vsh_final".into()]).unwrap();
        let (set_id, _) =
            crate::equations::create_complete_log_set(conn, &id.to_string()
            , &spec)
        .unwrap();
        crate::equations::write_computed_curves_with_ancestry(
            conn,
            &id.to_string(),
            &depth,
            &[("Vsh_final", &vsh)],
            &set_id,
        )
        .unwrap();

        (id, gr, vsh)
    }

    /// CORRECTNESS - SB-DIO-018 / SB-DIO-T29. The expected ownership boundary is
    /// specified by `21_data-io.md` section 6 T29: no writer-owned table, and the
    /// production writer must query `curves::canonical_unit`.
    #[test]
    fn the_las_writer_has_no_unit_table_and_queries_the_canonical_family_registry() {
        let source = include_str!("export.rs");
        let production_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("export production source precedes its test module");
        let duplicate_definition = ["fn standard", "_units"].concat();
        let registry_call = ["crate::curves::", "canonical_unit"].concat();
        assert!(
            !production_source.contains(&duplicate_definition),
            "the duplicate writer table must stay deleted"
        );
        assert!(
            production_source.contains(&registry_call),
            "the writer must consult curves::canonical_unit"
        );
    }

    /// CORRECTNESS - SB-DIO-018 / SB-DIO-T30. The expected unit for each family
    /// is the reviewed canonical family table cited by `21_data-io.md` section 5.1;
    /// comparison is exact so spelling and case cannot drift at the file boundary.
    #[test]
    fn every_exported_family_declares_the_canonical_registry_unit_with_exact_case() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let id = Uuid::new_v4();
        let well_id = id.to_string();
        db::insert_well(&conn, id, "CANONICAL-UNITS", None, None, None).unwrap();
        let depth = vec![1000.0_f32, 1000.5, 1001.0];
        db::insert_standard_curves(
            &conn,
            id,
            depth.clone(),
            vec![50.0; 3],
            vec![2.0; 3],
            vec![0.2; 3],
            vec![2.4; 3],
            vec![80.0; 3],
            vec![10.0; 3],
        )
        .unwrap();
        let standard = ["GR", "RES_DEEP", "NPHI", "RHOB", "DT", "SP"];
        for family in crate::curves::FAMILIES {
            if standard.contains(&family.family) {
                continue;
            }
            let curve_id = db::upsert_curve_meta(
                &conn,
                &well_id,
                "RAW",
                family.family,
                Some(family.canonical_unit),
                Some(family.family),
                Some("synthetic canonical-unit fixture"),
                None,
            )
            .unwrap();
            db::insert_curve_samples(&conn, &curve_id, &depth, &[1.0, 1.0, 1.0]).unwrap();
        }

        let dest = tmp_path("canonical-units");
        export_las(&conn, &well_id, dest.to_str().unwrap()).unwrap();
        let text = crate::parsers::read_text_file(&dest).unwrap();
        std::fs::remove_file(&dest).ok();
        let curve_block = text
            .split("~Curve Information")
            .nth(1)
            .and_then(|tail| tail.split("~Other Information").next())
            .unwrap();
        for family in crate::curves::FAMILIES {
            let declared = curve_block.lines().find_map(|line| {
                let entry = line.trim().split(':').next()?;
                let (mnemonic, unit) = entry.split_once('.')?;
                mnemonic.trim().eq(family.family).then_some(unit.trim())
            })
            .unwrap_or_else(|| panic!("{} was not exported", family.family));
            assert_eq!(
                declared,
                family.canonical_unit,
                "{} must retain the table's exact unit spelling and case",
                family.family
            );
        }
    }

    /// SB-DIO-022 / SB-DIO-T35. D-22 and D-23 require writer-side re-gridding to be
    /// an explicit resample and to default OFF. The synthetic values are deliberately
    /// non-linear as well as irregularly spaced: a writer that silently regularized the
    /// depths or interpolated the curve would fail one side of the assertion even if it
    /// happened to preserve the other.
    #[test]
    fn an_export_at_defaults_writes_the_irregular_stored_samples_without_regridding() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let id = Uuid::new_v4();
        db::insert_well(&conn, id, "IRREGULAR-EXPORT", None, None, None).unwrap();
        let stored_depth = vec![1000.0_f32, 1000.1, 1000.35, 1001.0];
        let stored_gr = vec![10.0_f32, 40.0, 15.0, 90.0];
        let missing = vec![f32::NAN; stored_depth.len()];
        db::insert_standard_curves(
            &conn,
            id,
            stored_depth.clone(),
            stored_gr.clone(),
            missing.clone(),
            missing.clone(),
            missing.clone(),
            missing.clone(),
            missing,
        )
        .unwrap();

        let settings = WriterSettings { null_sentinel: project_null_sentinel(&conn).unwrap() };
        for writer in REGISTERED_WRITERS {
            let dest = tmp_path(&format!("irregular-default-{}", writer.id))
                .with_extension(writer.extension);
            let result = export_with_writer(
                &conn,
                &id.to_string(),
                dest.to_str().unwrap(),
                settings,
                writer,
            )
            .unwrap();
            assert!(result.self_checked, "{} must pass its registered reader", writer.id);

            // Test adapters are deliberately exhaustive over REGISTERED_WRITERS. Adding a
            // format without teaching T35 to inspect its stored index must fail rather than
            // silently reducing the universal writer contract to today's default LAS.
            let rows: Vec<Vec<f32>> = match writer.id {
                "las-2.0" => crate::parsers::read_text_file(&dest)
                    .unwrap()
                    .split("~ASCII")
                    .nth(1)
                    .unwrap()
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(|line| {
                        line.split_whitespace()
                            .map(|field| field.parse::<f32>().unwrap())
                            .collect()
                    })
                    .collect(),
                unknown => panic!(
                    "registered writer '{unknown}' has no SB-DIO-T35 native-sample inspection adapter"
                ),
            };
            std::fs::remove_file(&dest).ok();

            assert_eq!(
                rows.iter().map(|row| row[0]).collect::<Vec<_>>(),
                stored_depth,
                "{} must not replace an irregular stored index with a regular grid",
                writer.id
            );
            assert_eq!(
                rows.iter().map(|row| row[1]).collect::<Vec<_>>(),
                stored_gr,
                "{} must not interpolate stored values onto different samples",
                writer.id
            );
        }
    }

    /// `export.rs` shipped with no tests at all. Three claims matter and none was pinned.
    ///
    /// **Missing writes as the null value, never as a blank or a zero.** A 0.0 where a sample was
    /// absent is a reading nobody took, and it re-imports as real data.
    ///
    /// **Computed curves are exported, not just the six standard ones** — otherwise a delivered
    /// LAS silently omits the entire interpretation.
    ///
    /// **A mixed-case computed name still carries its values.** `fetch_curve_frame` keys its
    /// column map by `trim().to_uppercase()`, so looking it up under the stored spelling misses
    /// and the column exports as ALL NULL — a full-length curve of nothing, in a file that looks
    /// perfectly well formed. The exporter uppercases the key for exactly this reason; this test
    /// is what stops that line being "tidied" away.
    #[test]
    fn export_writes_missing_as_null_and_carries_mixed_case_computed_curves() {
        let conn = Connection::open_in_memory().unwrap();
        let (id, gr, vsh) = seed(&conn);
        let dest = tmp_path("null");

        let result = export_las(&conn, &id.to_string(), dest.to_str().unwrap()).unwrap();
        assert_eq!(result.rows, 6, "one row per depth sample");

        let text = std::fs::read_to_string(&dest).unwrap();
        let _ = std::fs::remove_file(&dest);

        assert!(text.contains("NULL."), "the header must declare a null value");
        assert!(
            text.contains("-999.2500"),
            "a missing sample must be written as the declared null, not blank or 0"
        );
        assert!(
            text.contains("Vsh_final"),
            "computed curves must appear in ~Curve Information"
        );

        // The data block, column by column: DEPT, the six standard curves, then Vsh_final.
        let body = text.split("~ASCII").nth(1).unwrap();
        let rows: Vec<Vec<f32>> = body
            .lines()
            .filter(|l| l.trim_start().starts_with(|c: char| c.is_ascii_digit() || c == '-'))
            .map(|l| l.split_whitespace().map(|t| t.parse::<f32>().unwrap()).collect())
            .collect();
        assert_eq!(rows.len(), 6);

        let vsh_col: Vec<f32> = rows.iter().map(|r| *r.last().unwrap()).collect();
        assert!(
            vsh_col.iter().any(|v| (*v - DEFAULT_NULL_SENTINEL).abs() > 1e-3),
            "the mixed-case computed curve exported as ALL NULL — the uppercase key lookup broke"
        );
        for (i, expect) in vsh.iter().enumerate() {
            let got = vsh_col[i];
            if expect.is_nan() {
                assert!((got - DEFAULT_NULL_SENTINEL).abs() < 1e-3, "row {i}: missing must be the null value");
            } else {
                assert!((got - expect).abs() < 5e-4, "row {i}: {got} != {expect}");
            }
        }

        // GR is the second column (after DEPT) and carries the missing sample at index 2.
        let gr_col: Vec<f32> = rows.iter().map(|r| r[1]).collect();
        assert!((gr_col[2] - DEFAULT_NULL_SENTINEL).abs() < 1e-3, "GR's missing sample must be null");
        assert!((gr_col[0] - gr[0]).abs() < 5e-4);
    }

    /// SB-DIO-001 / SB-DIO-T01. The non-default value is Baker's cited waveform
    /// sentinel from `docs/PRD_v2/21_data-io.md` §5.2, not an invented product default.
    #[test]
    fn a_declared_sentinel_reaches_every_registered_writer_and_no_writer_emits_its_own() {
        let conn = Connection::open_in_memory().unwrap();
        let (id, _, _) = seed(&conn);
        let declared = -32767.0_f32;
        set_project_null_sentinel(&conn, declared).unwrap();
        assert_eq!(project_null_sentinel(&conn).unwrap(), declared);

        let settings = WriterSettings {
            null_sentinel: project_null_sentinel(&conn).unwrap(),
        };
        for writer in REGISTERED_WRITERS {
            let dest = tmp_path(&format!("declared-sentinel-{}", writer.id))
                .with_extension(writer.extension);
            let result = export_with_writer(
                &conn,
                &id.to_string(),
                dest.to_str().unwrap(),
                settings,
                writer,
            )
            .unwrap();
            let text = crate::parsers::read_text_file(&dest).unwrap();
            let _ = std::fs::remove_file(&dest);

            assert!(
                result.self_checked,
                "registered writer {} must pass its reader",
                writer.id
            );
            assert!(
                text.lines().any(|line| line.contains("NULL.") && line.contains("-32767.0000")),
                "registered writer {} must declare the project sentinel",
                writer.id
            );
            assert!(
                text.contains("-32767.0000"),
                "registered writer {} must use the declared sentinel for missing samples",
                writer.id
            );
            assert!(
                !text.contains("-999.2500"),
                "registered writer {} must not emit a private default",
                writer.id
            );
        }
    }

    /// SB-DIO-001 / SB-DIO-T02. `WriterFn` is the registry boundary: removing the final,
    /// non-optional `WriterSettings` argument from a writer makes this assignment and the
    /// registry constant fail to compile.
    /// SB-DIO-056 / SB-DIO-T82, SB-DIO-T83. Source: `docs/PRD_v2/21_data-io.md:1861-1878` — the
    /// writer MUST compute the step over every adjacent pair, MUST write `STEP` as `0` when the
    /// interval is not constant, and MUST NOT declare the first interval as the step. The `STEP = 0`
    /// mechanism is LAS 2.0's own provision for a non-uniform index and is cited by the chapter at
    /// `:867`; it is not a convention invented here. DEC-055 supplies the one thing the chapter left
    /// open — "within the stated tolerance" — as EXACT equality with no epsilon, matching the
    /// read-side rule already stated at `parsers.rs:549-552`.
    ///
    /// The comparison is made on the fixed-decimal-4 text the writer actually emits, not on the
    /// stored `f32`s, because that text is what a conforming reader sees when it reconstructs
    /// depths from `STRT`/`STEP` instead of reading the `DEPT` column.
    #[test]
    fn a_las_export_declares_a_step_only_when_every_written_interval_is_identical_and_writes_zero_otherwise(
    ) {
        fn export(depths: &[f32], tag: &str) -> (String, Option<String>) {
            let conn = Connection::open_in_memory().unwrap();
            db::create_schema(&conn).unwrap();
            crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
            let id = Uuid::new_v4();
            db::insert_well(&conn, id, "SANDI-STEP", Some("Synthetic"), None, None).unwrap();
            let n = depths.len();
            let gr: Vec<f32> = (0..n).map(|i| 40.0 + i as f32).collect();
            let nan = vec![f32::NAN; n];
            db::insert_standard_curves(
                &conn,
                id,
                depths.to_vec(),
                gr,
                nan.clone(),
                nan.clone(),
                nan.clone(),
                nan.clone(),
                nan,
            )
            .unwrap();
            let dest = tmp_path(tag);
            let result = export_las(&conn, &id.to_string(), dest.to_str().unwrap()).unwrap();
            let text = crate::parsers::read_text_file(&dest).unwrap();
            let _ = std::fs::remove_file(&dest);
            let line = text
                .lines()
                .find(|line| line.trim_start().starts_with("STEP."))
                .expect("~W must always carry a STEP line")
                .to_string();
            (line, result.nonuniform_step)
        }

        // A. A genuinely uniform frame still declares its own step. This arm is why the
        //    comparison cannot be f32 subtraction: at ~1000 m an f32 resolves to about
        //    0.00006, so the successive differences of a perfect 0.1524 m frame do not come
        //    out bit-identical, and an implementation that subtracts stored f32s would call
        //    this log irregular and write 0 — losing a true step on the commonest frame there
        //    is. Without this arm, "always write 0" passes.
        let (uniform, uniform_note) = export(
            &[1000.0, 1000.1524, 1000.3048, 1000.4572],
            "step-uniform",
        );
        assert!(
            uniform.contains("0.1524"),
            "a uniform frame must declare its real step: {uniform}"
        );
        assert!(
            uniform_note.is_none(),
            "a uniform frame must not be reported as non-uniform: {uniform_note:?}"
        );

        // B. Irregular, but only from the THIRD interval. The first two are identical, so this
        //    fails an implementation that reads depth[1] - depth[0] (the divergence the chapter
        //    names at :862) and equally one that checks only the first two pairs.
        let (drifts, drift_note) = export(
            &[1000.0, 1000.1524, 1000.3048, 1000.5],
            "step-drifts-late",
        );
        assert!(
            !drifts.contains("0.1524"),
            "the first interval must never be declared as the step: {drifts}"
        );
        let declared: f32 = drifts
            .split_whitespace()
            .find_map(|token| token.parse::<f32>().ok())
            .expect("the STEP line must carry a number");
        assert_eq!(declared, 0.0, "a non-constant index must declare STEP 0: {drifts}");

        // C. The zero is explained, not silent. A file that declares STEP 0 with nothing said
        //    is a degraded result presented as clean — the chapter's own SB-CORE-002 complaint
        //    against the neighbouring SB-DIO-055. Both spacings and the depth are named so the
        //    user can tell real drift from a delivery defect.
        let note = drift_note.expect("writing STEP 0 must be reported, never silent");
        assert!(note.contains("1000.3048"), "the depth where spacing changes must be named: {note}");
        assert!(note.contains("0.1524"), "the prior spacing must be named: {note}");
        assert!(note.contains("0.1952"), "the new spacing must be named: {note}");

        // D. A single-sample export has no interval at all, so it declares no step and is not
        //    reported as drifting — absence of evidence is not non-uniformity.
        let (single, single_note) = export(&[1000.0], "step-single-sample");
        let only: f32 = single
            .split_whitespace()
            .find_map(|token| token.parse::<f32>().ok())
            .expect("the STEP line must carry a number");
        assert_eq!(only, 0.0, "one sample cannot declare a step: {single}");
        assert!(single_note.is_none(), "one sample is not a drift finding: {single_note:?}");
    }
    #[test]
    fn a_registered_writer_cannot_omit_the_required_sentinel_argument() {
        let _: WriterFn = write_las;
        for writer in REGISTERED_WRITERS {
            let _: WriterFn = writer.write;
        }
    }

    /// SB-DIO-002 / SB-DIO-T03. `-32767` is the cited Baker waveform sentinel in
    /// `docs/PRD_v2/21_data-io.md` §5.2; it is used only to prove the default path
    /// honours a non-default declaration.
    #[test]
    fn the_default_export_format_honours_the_sentinel_and_an_incapable_format_is_marked() {
        let defaults: Vec<&RegisteredWriter> =
            REGISTERED_WRITERS.iter().filter(|writer| writer.is_default).collect();
        assert_eq!(defaults.len(), 1, "the format picker must have one unambiguous default");
        assert!(matches!(defaults[0].sentinel_support, SentinelSupport::Honours));

        fn cannot_write(
            _: &Connection,
            _: &str,
            _: &str,
            _: WriterSettings,
        ) -> Result<LasExportResult, String> {
            Err("format cannot carry an arbitrary null sentinel".into())
        }
        let incapable = RegisteredWriter {
            id: "incapable-test-format",
            label: "Incapable test format",
            extension: "test",
            is_default: false,
            sentinel_support: SentinelSupport::Incapable("fixed null convention"),
            write: cannot_write,
            self_read: validate_las_output,
        };
        let shown = format_info(&incapable);
        assert!(!shown.honours_project_sentinel);
        assert_eq!(shown.sentinel_limitation.as_deref(), Some("fixed null convention"));

        let conn = Connection::open_in_memory().unwrap();
        let (id, _, _) = seed(&conn);
        set_project_null_sentinel(&conn, -32767.0).unwrap();
        let dest = tmp_path("default-format-sentinel");
        export_las(&conn, &id.to_string(), dest.to_str().unwrap()).unwrap();
        let text = crate::parsers::read_text_file(&dest).unwrap();
        let _ = std::fs::remove_file(&dest);
        assert!(text.contains("-32767.0000"));
        assert!(!text.contains("-999.2500"));
    }

    /// The round trip: export a well, import the file into a FRESH project, and the numbers must
    /// come back unchanged.
    ///
    /// This is worth more than either half tested alone, because it is the only check that the
    /// writer and the reader agree about the SAME conventions — the null value, the fixed-width
    /// columns, the well name, the depth grid. A writer and a reader can each be
    /// self-consistently wrong; they cannot both be wrong in a way that survives a round trip.
    ///
    /// Missing must survive as MISSING. If the declared -999.25 came back as the number
    /// -999.25 it would be a porosity of minus a thousand, and it would plot.
    #[test]
    fn an_exported_las_reimports_with_the_same_values() {
        let src = Connection::open_in_memory().unwrap();
        let (id, gr, _) = seed(&src);
        let dest = tmp_path("roundtrip");
        export_las(&src, &id.to_string(), dest.to_str().unwrap()).unwrap();

        // A brand new project — nothing carried over except the file itself.
        let dst = Connection::open_in_memory().unwrap();
        db::create_schema(&dst).unwrap();
        let results =
            crate::ingest::import_las_files(&dst, &[dest.to_str().unwrap().to_string()], None);
        let _ = std::fs::remove_file(&dest);

        assert_eq!(results.len(), 1);
        assert!(results[0].error.is_none(), "re-import failed: {:?}", results[0].error);

        let new_id: String = dst
            .query_row("SELECT well_id FROM wells", [], |r| r.get(0))
            .expect("the re-imported well must exist");
        let name: String = dst
            .query_row("SELECT well_name FROM wells", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "SANDI-EXP", "the well name must survive the round trip");

        let (depth, cols) =
            crate::equations::fetch_curve_frame(&dst, &new_id, &["GR".to_string()]).unwrap();
        assert_eq!(depth.len(), 6, "every depth sample must come back");

        let back = cols.get("GR").expect("GR must be present after re-import");
        for (i, expect) in gr.iter().enumerate() {
            if expect.is_nan() {
                assert!(
                    back[i].is_nan(),
                    "row {i}: the declared null must return as MISSING, not as the number -999.25"
                );
            } else {
                assert!((back[i] - expect).abs() < 5e-4, "row {i}: {} != {expect}", back[i]);
            }
        }
    }

    /// SB-DIO-049 / T68. Registration requires a format's own reader, and the shared
    /// export wrapper is the only route to success. The corrupt control returns a normal
    /// writer result after emitting half an ASCII row; only the mandatory read-back can
    /// turn that apparent success into the required error.
    #[test]
    fn every_registered_writer_reads_its_output_and_a_rejected_round_trip_is_an_error() {
        let conn = Connection::open_in_memory().unwrap();
        let (id, _, _) = seed(&conn);
        let settings = WriterSettings { null_sentinel: project_null_sentinel(&conn).unwrap() };
        for writer in REGISTERED_WRITERS {
            let dest = tmp_path(&format!("{}-self-read", writer.id));
            let result = export_with_writer(
                &conn,
                &id.to_string(),
                dest.to_str().unwrap(),
                settings,
                writer,
            )
            .unwrap();
            assert!(result.self_checked, "{} must not report success before its reader passes", writer.id);
            let _ = std::fs::remove_file(&dest);
        }

        fn write_corrupt(
            _: &Connection,
            _: &str,
            dest_path: &str,
            _: WriterSettings,
        ) -> Result<LasExportResult, String> {
            std::fs::write(
                dest_path,
                "~Version\nVERS. 2.0\nWRAP. NO\n~Well\nWELL. ROW-WIDTH-CONTROL :\n~Curve\nDEPT.M\nGR.GAPI\n~ASCII\n1000\n",
            )
            .map_err(|error| error.to_string())?;
            Ok(LasExportResult {
                rows: 1,
                curves_written: 1,
                curves_held: 1,
                omitted: Vec::new(),
                curve_states: Vec::new(),
                legacy_unrecorded_curves: 0,
                precision: crate::parsers::SamplePrecisionReport::new(
                    "f32 storage",
                    "fixed-decimal-4 LAS text",
                    0,
                ),
                self_checked: false,
                nonuniform_step: None,
            })
        }
        let corrupt = RegisteredWriter {
            id: "corrupt-test",
            label: "Corrupt test writer",
            extension: "las",
            is_default: false,
            sentinel_support: SentinelSupport::Honours,
            write: write_corrupt,
            self_read: validate_las_output,
        };
        let dest = tmp_path("self-read-corrupt");
        let error = export_with_writer(
            &conn,
            &id.to_string(),
            dest.to_str().unwrap(),
            settings,
            &corrupt,
        )
        .unwrap_err();
        let _ = std::fs::remove_file(&dest);
        assert!(error.contains("LAS self-check failed"), "reader rejection must be the export error: {error}");
        assert!(error.contains("ASCII row has 1 value(s)"), "the reader's exact rejection must remain actionable: {error}");
    }

    /// SB-DIO-049 / T69. A unit lie is syntactically valid LAS, so parse success alone is
    /// insufficient: the self-reader must compare the DEPT declaration with the depth
    /// unit of the samples the writer was asked to deliver.
    #[test]
    fn a_feet_las_misdeclared_as_metres_fails_its_self_check_before_success() {
        fn write_metres_label(
            _: &Connection,
            _: &str,
            dest_path: &str,
            _: WriterSettings,
        ) -> Result<LasExportResult, String> {
            std::fs::write(
                dest_path,
                "~Version\nVERS. 2.0\nWRAP. NO\n~Well\nWELL. UNIT-LABEL-CONTROL :\n~Curve\nDEPT.M\nGR.GAPI\n~ASCII\n1000 50\n",
            )
            .map_err(|error| error.to_string())?;
            Ok(LasExportResult {
                rows: 1,
                curves_written: 1,
                curves_held: 1,
                omitted: Vec::new(),
                curve_states: Vec::new(),
                legacy_unrecorded_curves: 0,
                precision: crate::parsers::SamplePrecisionReport::new(
                    "f32 storage",
                    "fixed-decimal-4 LAS text",
                    0,
                ),
                self_checked: false,
                nonuniform_step: None,
            })
        }

        let conn = Connection::open_in_memory().unwrap();
        let (id, _, _) = seed(&conn);
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Feet).unwrap();
        let writer = RegisteredWriter {
            id: "wrong-unit-test",
            label: "Wrong-unit test writer",
            extension: "las",
            is_default: false,
            sentinel_support: SentinelSupport::Honours,
            write: write_metres_label,
            self_read: validate_las_output,
        };
        let dest = tmp_path("feet-misdeclared-metres");
        let error = export_with_writer(
            &conn,
            &id.to_string(),
            dest.to_str().unwrap(),
            WriterSettings { null_sentinel: project_null_sentinel(&conn).unwrap() },
            &writer,
        )
        .unwrap_err();
        let _ = std::fs::remove_file(&dest);
        assert!(error.contains("project depths are FT"), "the expected unit must be named: {error}");
        assert!(error.contains("declares M"), "the false declaration must be named: {error}");
    }

    fn assert_las_depth_unit_round_trip(unit: crate::units::DepthUnit, code: &str, tag: &str) {
        let src = Connection::open_in_memory().unwrap();
        let (id, _, _) = seed(&src);
        crate::units::set_project_depth_unit(&src, unit).unwrap();
        let dest = tmp_path(tag);
        export_las(&src, &id.to_string(), dest.to_str().unwrap()).unwrap();

        let text = crate::parsers::read_text_file(&dest).unwrap();
        for mnemonic in ["STRT", "STOP", "STEP", "DEPT"] {
            assert!(
                text.lines().any(|line| line.trim_start().starts_with(&format!("{mnemonic}.{code}"))),
                "{mnemonic} must declare {code} when those are the depths written"
            );
        }
        let other = if code == "FT" { "M" } else { "FT" };
        assert!(
            !text.lines().any(|line| {
                ["STRT", "STOP", "STEP", "DEPT"]
                    .iter()
                    .any(|mnemonic| line.trim_start().starts_with(&format!("{mnemonic}.{other}")))
            }),
            "the opposite unit must not remain on any depth declaration"
        );

        let dst = Connection::open_in_memory().unwrap();
        db::create_schema(&dst).unwrap();
        let imported = crate::ingest::import_las_files(&dst, &[dest.to_str().unwrap().to_string()], None);
        let _ = std::fs::remove_file(&dest);
        assert!(imported[0].error.is_none(), "{code} round trip failed: {:?}", imported[0].error);
        let imported_unit = crate::units::project_depth_unit(&dst).unwrap();
        assert_eq!(imported_unit, Some(unit));
        let first_depth: f32 = dst
            .query_row("SELECT depth FROM standard_curves ORDER BY depth LIMIT 1", [], |row| row.get(0))
            .unwrap();
        assert!((first_depth - 2000.0).abs() < 1e-4, "{code} depths must survive unchanged");
    }

    #[test]
    fn a_feet_project_las_round_trip_preserves_depths_and_declares_ft_on_every_depth_header() {
        // CORRECTNESS — source: docs/PRD_v2/21_data-io.md §6 SB-DIO-T27.
        assert_las_depth_unit_round_trip(crate::units::DepthUnit::Feet, "FT", "feet-unit");
    }

    #[test]
    fn characterizes_a_metre_project_las_round_trip_as_preserving_depths_and_declaring_m() {
        // CHARACTERIZATION — docs/PRD_v2/21_data-io.md §6 labels SB-DIO-T28 as char.
        assert_las_depth_unit_round_trip(crate::units::DepthUnit::Metres, "M", "metre-unit");
    }

    fn prefixed_json(text: &str, prefix: &str) -> Vec<serde_json::Value> {
        text.lines()
            .filter_map(|line| line.trim().strip_prefix(prefix))
            .map(|json| serde_json::from_str(json).expect("provenance line must be JSON"))
            .collect()
    }

    /// SB-DBM-001 / SB-DBM-T03. CORRECTNESS: the expected classes and counts come from
    /// `docs/PRD_v2/22_database-model.md` sections 4.A and 6.2. The recorded and legacy sides
    /// are both required: an unconditional "legacy" label and a resolver that silently drops
    /// NULL `set_id` rows would each satisfy only one side.
    #[test]
    fn every_computed_value_resolves_to_one_run_or_is_counted_and_labelled_legacy_unrecorded() {
        let conn = Connection::open_in_memory().unwrap();
        let (id, depth, _) = seed(&conn);
        let well_id = id.to_string();
        conn.execute(
            "INSERT INTO computed_curves (well_id, depth, curve_name, value, set_id)
             VALUES (?1, ?2, 'LEGACY_PHIE', 0.17, NULL),
                    (?1, ?3, 'LEGACY_PHIE', 0.19, NULL)",
            params![well_id, depth[0], depth[1]],
        )
        .unwrap();

        let catalog = serde_json::to_value(
            crate::equations::list_computed_catalog(&conn, &well_id).unwrap(),
        )
        .unwrap();
        let catalog = catalog.as_array().unwrap();
        let recorded = catalog
            .iter()
            .find(|row| row["curve_name"] == "Vsh_final")
            .expect("the current computed curve must remain catalogued");
        assert_eq!(recorded["provenance_class"], "RECORDED");
        assert_eq!(recorded["provenance_row_count"], depth.len());
        assert!(recorded["ancestry"].is_object());
        let legacy = catalog
            .iter()
            .find(|row| row["curve_name"] == "LEGACY_PHIE")
            .expect("a legacy curve must not disappear from the catalog");
        assert_eq!(legacy["provenance_class"], "LEGACY_UNRECORDED");
        assert_eq!(legacy["provenance_row_count"], 2);
        assert!(legacy["ancestry"].is_null());

        let disclosures = serde_json::to_value(
            crate::equations::curve_ancestry_disclosures(&conn, &[well_id.clone()], None)
                .unwrap(),
        )
        .unwrap();
        let disclosures = disclosures.as_array().unwrap();
        assert!(disclosures.iter().any(|row| {
            row["curve_name"] == "Vsh_final"
                && row["provenance_class"] == "RECORDED"
                && row["provenance_row_count"] == depth.len()
        }));
        assert!(disclosures.iter().any(|row| {
            row["curve_name"] == "LEGACY_PHIE"
                && row["provenance_class"] == "LEGACY_UNRECORDED"
                && row["provenance_row_count"] == 2
                && row["ancestry"].is_null()
        }));

        let dest = tmp_path("legacy-unrecorded");
        let result = export_las(&conn, &well_id, dest.to_str().unwrap())
            .expect("legacy computed values remain exportable only with an explicit label");
        let text = crate::parsers::read_text_file(&dest).unwrap();
        let _ = std::fs::remove_file(&dest);
        let provenance = prefixed_json(&text, PROVENANCE_PREFIX);
        let legacy_line = provenance
            .iter()
            .find(|row| row["curve"] == "LEGACY_PHIE")
            .expect("the exported legacy curve needs an in-file provenance line");
        assert_eq!(legacy_line["origin"], "computed");
        assert_eq!(legacy_line["provenance_class"], "LEGACY_UNRECORDED");
        assert_eq!(legacy_line["row_count"], 2);
        assert!(legacy_line.get("method").is_none());
        assert!(legacy_line.get("parameters").is_none());
        assert_eq!(result.legacy_unrecorded_curves, 1);
        let summary = prefixed_json(&text, "SANDIBUMI_PROVENANCE_SUMMARY_V1 ");
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0]["legacy_unrecorded_curves"], 1);
    }

    /// SB-DIO-051 / SB-DIO-T71..T73. Required fields are specified in
    /// `docs/PRD_v2/21_data-io.md` §4.10 and `04_CORE_REQUIREMENTS.md` SB-CORE-014.
    #[test]
    fn every_las_export_carries_measured_computed_and_model_provenance_in_the_file() {
        // The measured-only side: `~O` is not conditional on there being a computed curve.
        let measured = Connection::open_in_memory().unwrap();
        db::create_schema(&measured).unwrap();
        crate::units::set_project_depth_unit(&measured, crate::units::DepthUnit::Metres).unwrap();
        let measured_id = Uuid::new_v4();
        db::insert_well(&measured, measured_id, "MEASURED-ONLY", None, None, None).unwrap();
        let measured_depth = vec![1000.0_f32, 1000.5];
        db::insert_standard_curves(
            &measured,
            measured_id,
            measured_depth,
            vec![50.0, 55.0],
            vec![2.0; 2],
            vec![0.2; 2],
            vec![2.4; 2],
            vec![f32::NAN; 2],
            vec![f32::NAN; 2],
        )
        .unwrap();
        let measured_dest = tmp_path("measured-provenance");
        export_las(&measured, &measured_id.to_string(), measured_dest.to_str().unwrap()).unwrap();
        let measured_text = crate::parsers::read_text_file(&measured_dest).unwrap();
        let _ = std::fs::remove_file(&measured_dest);
        let measured_rows = prefixed_json(&measured_text, PROVENANCE_PREFIX);
        assert_eq!(measured_rows.len(), 6, "every written standard curve needs a record");
        assert!(measured_rows.iter().all(|row| row["origin"] == "measured"));

        let conn = Connection::open_in_memory().unwrap();
        let (id, _, _) = seed(&conn);
        let well_id = id.to_string();
        let model_bytes = b"synthetic saved model artifact";
        let features = vec!["GR".to_string(), "RHOB".to_string()];
        let trained_on = vec!["TRAIN-A".to_string(), "TRAIN-B".to_string()];
        let (model_id, model_name) = crate::db::insert_ml_model(
            &conn,
            &crate::db::NewMlModel {
                name: "VSH_RF",
                task: "regression",
                algorithm: "rf",
                feature_curves: &features,
                target_curve: Some("VSH"),
                params_json: r#"{"n_estimators":17,"seed":42}"#,
                metrics_json: r#"{"r2_blind":0.61}"#,
                trained_on: &trained_on,
                n_train: 12,
                standardize: true,
                note: None,
                data: model_bytes,
                train_hash: Some("training-row-hash"),
                training_json: Some(r#"{"input_set":"WIRE","wells":["TRAIN-A","TRAIN-B"]}"#),
                runtime_json: Some(r#"{"python":"3.12","sklearn":"1.7"}"#),
                sklearn_version: Some("1.7"),
            },
        )
        .unwrap();
        let run_params = serde_json::json!({
            "model_id": model_id.clone(),
            "model_name": model_name,
            "target": "VSH",
            "blind": { "performed": true, "metric": "R2", "value": 0.61,
                       "answers_new_well": true, "n_blind_wells": 1, "n_blind_rows": 4 },
            "train_hash": "training-row-hash",
            "trained_on": ["TRAIN-A", "TRAIN-B"],
        });
        let custody = crate::workflow::test_run_custody() ;
        let spec = crate::equations::complete_curve_run_spec(
            &conn,
            &well_id,
            "PREDICTED",
                "ml:rf",
                &custody,
                &[(well_id.clone(), "feature_1".into(), "GR".into()),
                (well_id.clone(), "feature_2".into(), "RHOB".into()),
            ],
            None,
            run_params,
            crate::equations::AncestryZoneScope::WholeWell,
            &["VSH_PRED".into()]).unwrap()
            ;
        let (set_id, _) =
            crate::equations::create_complete_log_set(&conn, &well_id, &spec)
        .unwrap();
        let depth: Vec<f32> = (0..6).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let predicted = vec![0.2_f32; depth.len()];
        crate::equations::write_computed_curves_with_ancestry(
            &conn,
            &well_id,
            &depth,
            &[("VSH_PRED", &predicted)],
            &set_id,
        )
        .unwrap();

        let dest = tmp_path("provenance");
        export_las(&conn, &well_id, dest.to_str().unwrap()).unwrap();
        let text = crate::parsers::read_text_file(&dest).unwrap();
        let _ = std::fs::remove_file(&dest);
        assert!(text.contains("~Other Information"));
        let rows = prefixed_json(&text, PROVENANCE_PREFIX);
        let gr = rows.iter().find(|row| row["curve"] == "GR").unwrap();
        assert_eq!(gr["origin"], "measured");
        let vsh = rows.iter().find(|row| row["curve"] == "VSH_FINAL").unwrap();
        assert_eq!(vsh["origin"], "computed");
        assert_eq!(vsh["method"], "vsh_gr");
        assert_eq!(vsh["parameters"]["gr_clean"], 25.0);
        assert_eq!(vsh["parameters"]["gr_shale"], 125.0);
        let stored_run_parameters: String = conn
            .query_row(
                "SELECT params_json FROM log_sets WHERE module = 'vsh_gr'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let stored_run_parameters: serde_json::Value =
            serde_json::from_str(&stored_run_parameters).unwrap();
        assert_eq!(
            vsh["parameters"], stored_run_parameters,
            "every stored run parameter and value must cross into the deliverable"
        );

        let models = prefixed_json(&text, MODEL_PROVENANCE_PREFIX);
        let model = models.iter().find(|row| row["curve"] == "VSH_PRED").unwrap();
        assert_eq!(model["record"]["feature_curves"], serde_json::json!(["GR", "RHOB"]));
        assert_eq!(model["record"]["params_json"], r#"{"n_estimators":17,"seed":42}"#);
        assert_eq!(model["record"]["train_hash"], "training-row-hash");
        assert_eq!(model["record"]["runtime_json"], r#"{"python":"3.12","sklearn":"1.7"}"#);
        let artifact_hash = model["record"]["artifact_sha256"].as_str().unwrap();
        assert_eq!(artifact_hash.len(), 64, "the fitted artifact has its own SHA-256 identity");
        let (stored_model, stored_artifact) = crate::db::get_ml_model(&conn, &model_id).unwrap();
        assert_eq!(stored_artifact, model_bytes, "the fixture's saved artifact is the cited one");
        let mut expected_model_record = serde_json::to_value(stored_model).unwrap();
        expected_model_record.as_object_mut().unwrap().insert(
            "artifact_sha256".into(),
            serde_json::Value::String(format!("{:x}", Sha256::digest(model_bytes))),
        );
        assert_eq!(
            model["record"], expected_model_record,
            "the complete saved-model record, not a selected subset, must cross into the file"
        );

        // SB-DBM-001 deliberately strengthens the legacy side: an unversioned computed curve must
        // not be relabelled as measured or receive invented ancestry, but it remains exportable
        // under the explicit LEGACY_UNRECORDED class and exact row count.
        let mut appender = conn.appender("computed_curves").unwrap();
        for (sample_depth,
            value) in depth.iter().zip(predicted.iter()) {
            appender
                .append_row(params![well_id,
                    sample_depth,
                    "LEGACY_NO_PROVENANCE", value,
                    None::<String>
                ],
        )
                .unwrap();
        }
        appender.flush()
        .unwrap();
        let legacy_dest = tmp_path("missing-provenance");
        let _ = std::fs::remove_file(&legacy_dest);
        let result = export_las(&conn, &well_id, legacy_dest.to_str().unwrap()).unwrap();
        let text = crate::parsers::read_text_file(&legacy_dest).unwrap();
        let _ = std::fs::remove_file(&legacy_dest);
        let rows = prefixed_json(&text, PROVENANCE_PREFIX);
        let legacy = rows
            .iter()
            .find(|row| row["curve"] == "LEGACY_NO_PROVENANCE")
            .expect("legacy computed curve needs an in-file provenance record");
        assert_eq!(legacy["origin"], "computed");
        assert_eq!(legacy["provenance_class"], crate::equations::LEGACY_UNRECORDED);
        assert_eq!(legacy["row_count"], depth.len());
        assert!(legacy.get("method").is_none(), "legacy ancestry must never be invented");
        assert_eq!(result.legacy_unrecorded_curves, 1);

        crate::db::delete_ml_model(&conn, &model_id).unwrap();
        let missing_model_dest = tmp_path("unavailable-model-provenance");
        let error = export_las(&conn, &well_id, missing_model_dest.to_str().unwrap())
            .expect_err("a model-derived curve whose saved record is unavailable must be refused");
        let _ = std::fs::remove_file(&missing_model_dest);
        assert!(error.contains("VSH_PRED"), "the refusal must name the model-derived curve: {error}");
        assert!(error.contains(&model_id), "the refusal must name the unavailable model: {error}");
        assert!(error.contains("refused"), "the result must be an explicit refusal: {error}");

        // The opposite side: a computed identity that shadows a measured standard mnemonic cannot
        // be described truthfully as one LAS curve. SB-DIO-051 requires the file to distinguish
        // measured from computed; exporting two GR columns as "measured" would satisfy the happy
        // fixture above while lying about the second column. Refusal is the only truthful result.
        let shadow_spec = crate::equations::complete_curve_run_spec(
            &conn,
            &well_id,
            "RECONSTRUCTED",
            "synthetic:reconstruction",
            &custody,
            &[(well_id.clone(), "SOURCE".into(), "GR".into())],
            None,
            serde_json::json!({ "operation": "identity" }),
            crate::equations::AncestryZoneScope::WholeWell,
            &["GR".into()],
        )
        .unwrap();
        let (shadow_set_id, _) =
            crate::equations::create_complete_log_set(&conn, &well_id, &shadow_spec).unwrap();
        crate::equations::write_computed_curves_with_ancestry(
            &conn,
            &well_id,
            &depth,
            &[("GR", predicted.as_slice())],
            &shadow_set_id,
        )
        .unwrap();
        let shadow_dest = tmp_path("shadowed-standard-provenance");
        let error = export_las(&conn, &well_id, shadow_dest.to_str().unwrap())
            .expect_err("a measured/computed mnemonic collision must be refused");
        let _ = std::fs::remove_file(&shadow_dest);
        assert!(error.contains("GR"), "the refusal must name the ambiguous curve: {error}");
        assert!(
            error.contains("measured") && error.contains("computed"),
            "the refusal must explain the provenance conflict: {error}"
        );
    }

    /// SB-DIO-052 / T74. `curve_meta` declares FINAL as QC'd for delivery and RAW as
    /// imported/working. Both sides use the same source mnemonic, so this also pins that
    /// marking a state cannot be implemented by silently omitting the collision.
    #[test]
    fn a_working_and_final_phie_are_both_exported_and_each_is_marked_in_the_file() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let id = Uuid::new_v4();
        let well_id = id.to_string();
        db::insert_well(&conn, id, "PHIE-STATES", None, None, None).unwrap();
        let depth = vec![1800.0_f32, 1800.5, 1801.0];
        db::insert_standard_curves(
            &conn,
            id,
            depth.clone(),
            vec![50.0; 3],
            vec![2.0; 3],
            vec![0.2; 3],
            vec![2.4; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
        )
        .unwrap();
        let working = db::upsert_curve_meta(
            &conn,
            &well_id,
            "RAW",
            "PHIE",
            Some("V/V"),
            Some("PHIE"),
            Some("synthetic working curve"),
            None,
        )
        .unwrap();
        let working_values = vec![0.10_f32, 0.11, 0.12];
        db::insert_curve_samples(&conn, &working, &depth, &working_values).unwrap();
        let final_curve = db::upsert_curve_meta(
            &conn,
            &well_id,
            "FINAL",
            "PHIE",
            Some("V/V"),
            Some("PHIE"),
            Some("synthetic final curve"),
            None,
        )
        .unwrap();
        let final_values = vec![0.20_f32, 0.21, 0.22];
        db::insert_curve_samples(&conn, &final_curve, &depth, &final_values).unwrap();

        let dest = tmp_path("phie-final-working");
        let result = export_las(&conn, &well_id, dest.to_str().unwrap()).unwrap();
        let text = crate::parsers::read_text_file(&dest).unwrap();
        let frame = crate::parsers::parse_las_2_all(&dest).unwrap();
        let _ = std::fs::remove_file(&dest);

        let exported: Vec<&str> = frame.curves.iter().map(|curve| curve.mnemonic.as_str()).collect();
        assert!(exported.contains(&"PHIE"), "the working PHIE must remain in the file: {exported:?}");
        assert!(exported.contains(&"PHIE_FINAL"), "the final collision must be retained with its state: {exported:?}");
        let exported_working = frame.curves.iter().find(|curve| curve.mnemonic == "PHIE").unwrap();
        let exported_final = frame.curves.iter().find(|curve| curve.mnemonic == "PHIE_FINAL").unwrap();
        assert_eq!(
            exported_working.values, working_values,
            "the working identity must carry the working samples, not a copy of the final curve"
        );
        assert_eq!(
            exported_final.values, final_values,
            "the final identity must carry the final samples, not a renamed working curve"
        );
        assert!(
            !result.omitted.iter().any(|omission| omission.curve.starts_with("PHIE ")),
            "neither state may be hidden as a duplicate: {:?}",
            result.omitted
        );
        assert_eq!(result.curve_states.len(), 2);
        assert!(result.curve_states.iter().any(|curve| {
            curve.export_curve == "PHIE" && curve.source_curve == "PHIE"
                && curve.set_name == "RAW" && curve.state == "working"
        }));
        assert!(result.curve_states.iter().any(|curve| {
            curve.export_curve == "PHIE_FINAL" && curve.source_curve == "PHIE"
                && curve.set_name == "FINAL" && curve.state == "final"
        }));

        let file_states = prefixed_json(&text, CURVE_STATE_PREFIX);
        assert_eq!(file_states.len(), 2, "both identities must travel in ~Other");
        assert!(file_states.iter().any(|row| {
            row["export_curve"] == "PHIE" && row["source_curve"] == "PHIE"
                && row["set_name"] == "RAW" && row["state"] == "working"
        }));
        assert!(file_states.iter().any(|row| {
            row["export_curve"] == "PHIE_FINAL" && row["source_curve"] == "PHIE"
                && row["set_name"] == "FINAL" && row["state"] == "final"
        }));
    }

    /// SB-DIO-055 / SB-DIO-T80..T81. The completeness and two-surface omission rule is
    /// specified in `docs/PRD_v2/21_data-io.md` §§4.10 and 6.10.
    #[test]
    fn every_held_curve_is_written_or_named_with_the_same_reason_in_the_file_and_result() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let id = Uuid::new_v4();
        let well_id = id.to_string();
        db::insert_well(&conn, id, "FORTY-CURVES", None, None, None).unwrap();
        let depth = vec![1500.0_f32, 1500.5, 1501.0];
        db::insert_standard_curves(
            &conn,
            id,
            depth.clone(),
            vec![50.0; 3],
            vec![2.0; 3],
            vec![0.2; 3],
            vec![2.4; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
        )
        .unwrap();
        for i in 0..40 {
            let name = format!("X{i:02}");
            let curve_id = db::upsert_curve_meta(
                &conn,
                &well_id,
                "RAW",
                &name,
                Some("V/V"),
                None,
                Some("synthetic"),
                None,
            )
            .unwrap();
            db::insert_curve_samples(&conn, &curve_id, &depth, &vec![i as f32; depth.len()]).unwrap();
        }
        let collision = db::upsert_curve_meta(
            &conn,
            &well_id,
            "WIRE",
            "GR",
            Some("GAPI"),
            None,
            Some("synthetic"),
            Some(1),
        )
        .unwrap();
        db::insert_curve_samples(&conn, &collision, &depth, &[60.0; 3]).unwrap();
        let off_grid = db::upsert_curve_meta(
            &conn,
            &well_id,
            "LWD",
            "OFFGRID",
            Some("V/V"),
            None,
            Some("synthetic"),
            None,
        )
        .unwrap();
        db::insert_curve_samples(&conn, &off_grid, &[1500.25], &[1.0]).unwrap();

        let dest = tmp_path("forty-curves");
        let result = export_las(&conn, &well_id, dest.to_str().unwrap()).unwrap();
        let text = crate::parsers::read_text_file(&dest).unwrap();
        let parsed = crate::parsers::parse_las_2_all(&dest).unwrap();
        // CORRECTNESS: T80 supplies forty imported curves. The six standard columns and two
        // deliberately unwriteable controls are fixtures stated above, so 46 written of 48 held
        // is independently derived from the input rather than copied from the implementation.
        assert_eq!(parsed.curves.len(), 46, "the recipient-facing LAS must contain every reported written curve");
        for i in 0..40 {
            let mnemonic = format!("X{i:02}");
            let curve = parsed
                .curves
                .iter()
                .find(|curve| curve.mnemonic == mnemonic)
                .unwrap_or_else(|| panic!("aligned generic curve {mnemonic} was not written as a LAS column"));
            assert_eq!(
                curve.values,
                vec![i as f32; depth.len()],
                "{mnemonic} must carry its own supplied samples, not merely appear in metadata"
            );
        }
        assert_eq!(result.curves_written, 46, "six standard plus all forty aligned curves");
        assert_eq!(result.curves_held, 48, "written plus the two deliberately unwriteable curves");
        assert_eq!(result.omitted.len(), 2);
        assert!(result.omitted.iter().any(|o| o.curve.starts_with("GR ") && o.reason.contains("already written")));
        assert!(result.omitted.iter().any(|o| o.curve.starts_with("OFFGRID ") && o.reason.contains("different depth frame")));

        let file_omissions = prefixed_json(&text, OMISSION_PREFIX);
        assert_eq!(file_omissions.len(), result.omitted.len());
        for omission in &result.omitted {
            assert!(file_omissions.iter().any(|row| {
                row["curve"] == omission.curve && row["reason"] == omission.reason
            }));
        }

        let ribbon = include_str!("../../src/ui/ribbon.ts");
        assert!(
            ribbon.contains("${result.curves_written} of ${result.curves_held} held curves written."),
            "T81 counts must remain user-visible rather than stopping at the IPC result"
        );
        assert!(
            ribbon.contains(r#"result.omitted.map((item) => `${item.curve}: ${item.reason}`)"#),
            "the exact omitted identity and reason must remain user-visible"
        );
        let _ = std::fs::remove_file(&dest);
    }
}
