use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

use duckdb::{params, Connection};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DisplayRange {
    pub low: f32,
    pub high: f32,
}

/// What the plot asked for. This is kept beside, never replaced by, the concrete
/// curve selected independently in each well.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlotChannelIntent {
    pub channel: String,
    pub semantic_request: String,
    pub required: bool,
}

/// One well's concrete answer to a semantic channel request. Strings are used for
/// quantity and conversion because this record is persisted and must remain readable
/// when the unit registry gains a new typed quantity or transform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedPlotCurve {
    pub well_id: String,
    pub curve_id: String,
    pub mnemonic: String,
    pub quantity: String,
    pub source_unit: String,
    pub display_unit: String,
    pub conversion: String,
    pub sample_count: usize,
    pub resolution_reason: String,
    pub source_revision: String,
    /// Optional, concrete curve-header display range. It is user/project metadata, not a
    /// validity range and not a mnemonic-derived family default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header_display: Option<DisplayRange>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotChannelBinding {
    pub intent: PlotChannelIntent,
    pub resolved: Vec<ResolvedPlotCurve>,
}

pub const PLOT_STATE_SCHEMA_VERSION: u32 = 1;

/// Durable plot state keeps the reusable/display options separate from the exact
/// concrete curve answers that produced the visible marks. `well_ids` is the
/// represented set, not merely the user's broader requested scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedPlotState {
    pub schema_version: u32,
    pub plot_type: String,
    pub well_ids: Vec<String>,
    pub options: serde_json::Value,
    pub bindings: Vec<PlotChannelBinding>,
    /// Empty only while reading a pre-SB-PLT-002 legacy document. Every new typed write
    /// and export requires the exact displayed ranges and the tier that supplied them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub axis_ranges: Vec<PlotAxisRange>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedPlotDocument {
    pub name: String,
    pub state: PersistedPlotState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotBindingExport {
    pub schema_version: u32,
    pub well_ids: Vec<String>,
    pub bindings: Vec<PlotChannelBinding>,
    pub axis_ranges: Vec<PlotAxisRange>,
    pub statistics_records: Vec<PlotStatisticsRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotStatisticsInterval {
    pub low: Option<f64>,
    pub high: Option<f64>,
    pub closure: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotStatisticsSelection {
    pub kind: String,
    pub selection_id: Option<String>,
    pub label: String,
    pub applied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlotStatisticsExclusions {
    pub input_count: usize,
    pub non_finite: usize,
    pub log_domain: usize,
    pub validity: usize,
    pub selection: usize,
    pub unpaired_or_unclassified: usize,
    pub display_hidden: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotStatisticsValues {
    pub count: usize,
    pub mean: Option<f64>,
    pub std: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub p5: Option<f64>,
    pub p25: Option<f64>,
    pub p50: Option<f64>,
    pub p75: Option<f64>,
    pub p95: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotStatisticsRecord {
    pub schema_version: u32,
    pub binding_channel: String,
    pub channel: String,
    pub population: String,
    pub well_ids: Vec<String>,
    pub interval: PlotStatisticsInterval,
    pub selection: PlotStatisticsSelection,
    pub finite_pair_count: usize,
    pub exclusions: PlotStatisticsExclusions,
    pub percentile_interpolation: String,
    pub standard_deviation: String,
    pub values: PlotStatisticsValues,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotAxisRange {
    pub axis: String,
    pub min: f32,
    pub max: f32,
    pub tier: AxisRangeTier,
}

fn validate_plot_state_doc_type(doc_type: &str) -> Result<(), String> {
    if doc_type == "plotprops" {
        return Ok(());
    }
    if doc_type
        .strip_prefix("plottmpl:")
        .is_some_and(|plot_type| !plot_type.trim().is_empty())
    {
        return Ok(());
    }
    Err(format!(
        "plot state document type '{doc_type}' is not whitelisted"
    ))
}

pub fn validate_plot_bindings(
    well_ids: &[String],
    bindings: &[PlotChannelBinding],
) -> Result<(), String> {
    if well_ids.is_empty() {
        return Err("persisted plot state has no represented wells".into());
    }
    let mut represented = BTreeSet::new();
    for well_id in well_ids {
        non_blank(well_id, "well id")?;
        if !represented.insert(well_id.as_str()) {
            return Err(format!("persisted plot repeats represented well {well_id}"));
        }
    }
    if bindings.is_empty() {
        return Err("persisted plot state has no channel bindings".into());
    }
    let mut channels = BTreeSet::new();
    for binding in bindings {
        if !channels.insert(binding.intent.channel.trim().to_ascii_uppercase()) {
            return Err(format!(
                "persisted plot repeats channel '{}'",
                binding.intent.channel
            ));
        }
        persist_plot_binding(binding.intent.clone(), binding.resolved.clone())?;
        let mut resolved_wells = BTreeSet::new();
        for curve in &binding.resolved {
            if !represented.contains(curve.well_id.as_str()) {
                return Err(format!(
                    "channel '{}' resolves unrepresented well {}",
                    binding.intent.channel, curve.well_id
                ));
            }
            if !resolved_wells.insert(curve.well_id.as_str()) {
                return Err(format!(
                    "channel '{}' resolves well {} more than once",
                    binding.intent.channel, curve.well_id
                ));
            }
        }
        if binding.intent.required && resolved_wells != represented {
            let missing = represented
                .difference(&resolved_wells)
                .copied()
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "required channel '{}' is unresolved for represented well(s): {missing}",
                binding.intent.semantic_request
            ));
        }
    }
    Ok(())
}

pub fn validate_persisted_plot_state(state: &PersistedPlotState) -> Result<(), String> {
    if state.schema_version != PLOT_STATE_SCHEMA_VERSION {
        return Err(format!(
            "unsupported plot state schema version {}",
            state.schema_version
        ));
    }
    non_blank(&state.plot_type, "plot type")?;
    if !state.options.is_object() {
        return Err("persisted plot options must be a JSON object".into());
    }
    validate_plot_bindings(&state.well_ids, &state.bindings)?;
    validate_plot_axis_ranges(&state.axis_ranges, true)
}

pub fn validate_plot_axis_ranges(
    axis_ranges: &[PlotAxisRange],
    allow_legacy_empty: bool,
) -> Result<(), String> {
    if axis_ranges.is_empty() {
        return if allow_legacy_empty {
            Ok(())
        } else {
            Err("plot state has no resolved axis ranges".into())
        };
    }
    let mut axes = BTreeSet::new();
    for range in axis_ranges {
        non_blank(&range.axis, "axis name")?;
        if !axes.insert(range.axis.trim().to_ascii_lowercase()) {
            return Err(format!("plot state repeats axis '{}'", range.axis));
        }
        if !range.min.is_finite() || !range.max.is_finite() || range.min == range.max {
            return Err(format!(
                "plot axis '{}' requires two distinct finite display limits",
                range.axis
            ));
        }
    }
    Ok(())
}

pub fn save_persisted_plot_state(
    conn: &Connection,
    doc_type: &str,
    name: &str,
    state: &PersistedPlotState,
) -> Result<(), String> {
    validate_plot_state_doc_type(doc_type)?;
    if name.trim().is_empty() {
        return Err("persisted plot state requires a document name".into());
    }
    validate_persisted_plot_state(state)?;
    validate_plot_axis_ranges(&state.axis_ranges, false)?;
    let json = serde_json::to_string(state).map_err(|error| error.to_string())?;
    crate::db::save_document(conn, doc_type, name, &json).map_err(|error| error.to_string())
}

#[cfg(test)]
pub fn list_persisted_plot_states(
    conn: &Connection,
    doc_type: &str,
) -> Result<Vec<PersistedPlotDocument>, String> {
    validate_plot_state_doc_type(doc_type)?;
    crate::db::list_documents(conn, doc_type)
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|document| {
            let state: PersistedPlotState = serde_json::from_str(&document.json)
                .map_err(|error| format!("plot state '{}' is unreadable: {error}", document.name))?;
            validate_persisted_plot_state(&state)?;
            Ok(PersistedPlotDocument {
                name: document.name,
                state,
            })
        })
        .collect()
}

pub fn serialize_plot_binding_export(
    well_ids: &[String],
    bindings: &[PlotChannelBinding],
    axis_ranges: &[PlotAxisRange],
    statistics_records: &[PlotStatisticsRecord],
) -> Result<String, String> {
    validate_plot_bindings(well_ids, bindings)?;
    validate_plot_axis_ranges(axis_ranges, false)?;
    validate_plot_statistics_records(well_ids, bindings, statistics_records)?;
    serde_json::to_string(&PlotBindingExport {
        schema_version: PLOT_STATE_SCHEMA_VERSION,
        well_ids: well_ids.to_vec(),
        bindings: bindings.to_vec(),
        axis_ranges: axis_ranges.to_vec(),
        statistics_records: statistics_records.to_vec(),
    })
    .map_err(|error| error.to_string())
}

pub fn validate_plot_statistics_records(
    represented_well_ids: &[String],
    bindings: &[PlotChannelBinding],
    records: &[PlotStatisticsRecord],
) -> Result<(), String> {
    let represented = represented_well_ids
        .iter()
        .map(|well_id| well_id.as_str())
        .collect::<BTreeSet<_>>();
    let binding_channels = bindings
        .iter()
        .map(|binding| binding.intent.channel.trim().to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let mut channels = BTreeSet::new();
    for record in records {
        if record.schema_version != 1 {
            return Err(format!(
                "unsupported plot statistics schema version {}",
                record.schema_version
            ));
        }
        non_blank(&record.binding_channel, "plot statistics binding channel")?;
        if !binding_channels.contains(&record.binding_channel.trim().to_ascii_lowercase()) {
            return Err(format!(
                "plot statistics references unbound channel '{}'",
                record.binding_channel
            ));
        }
        non_blank(&record.channel, "plot statistics channel")?;
        if !channels.insert(record.channel.trim().to_ascii_lowercase()) {
            return Err(format!("plot statistics repeats channel '{}'", record.channel));
        }
        if record.well_ids.is_empty() || record.well_ids.iter().any(|well_id| well_id.trim().is_empty()) {
            return Err("plot statistics require every represented well identity".into());
        }
        let record_wells = record
            .well_ids
            .iter()
            .map(|well_id| well_id.as_str())
            .collect::<BTreeSet<_>>();
        if record_wells.len() != record.well_ids.len() {
            return Err("plot statistics cannot repeat a represented well identity".into());
        }
        if let Some(foreign) = record_wells.difference(&represented).next() {
            return Err(format!(
                "plot statistics references unrepresented well {foreign}"
            ));
        }
        match record.population.as_str() {
            "active_well" if record.well_ids.len() == 1 => {}
            "pooled" if record.well_ids.len() >= 2 => {}
            "active_well" => return Err("active-well statistics require exactly one represented well".into()),
            "pooled" => return Err("pooled statistics require at least two represented wells".into()),
            other => return Err(format!("unknown plot statistics population '{other}'")),
        }
        match record.interval.closure.as_str() {
            "[lo,hi)" => {
                let low = record.interval.low.ok_or_else(|| "plot statistics interval has no low limit".to_string())?;
                let high = record.interval.high.ok_or_else(|| "plot statistics interval has no high limit".to_string())?;
                if !low.is_finite() || !high.is_finite() || low >= high {
                    return Err("plot statistics interval requires increasing finite limits".into());
                }
            }
            "[lo,+inf)" if record.interval.low.is_some_and(f64::is_finite)
                && record.interval.high.is_none() => {}
            "[lo,+inf)" => {
                return Err("lower-bounded plot statistics require one finite low limit".into())
            }
            "(-inf,hi)" if record.interval.low.is_none()
                && record.interval.high.is_some_and(f64::is_finite) => {}
            "(-inf,hi)" => {
                return Err("upper-bounded plot statistics require one finite high limit".into())
            }
            "all" if record.interval.low.is_none() && record.interval.high.is_none() => {}
            "all" => return Err("all-depth statistics cannot carry numeric interval limits".into()),
            other => return Err(format!("unknown plot statistics interval closure '{other}'")),
        }
        non_blank(&record.selection.label, "plot statistics selection label")?;
        match record.selection.kind.as_str() {
            "all_eligible" if record.selection.selection_id.is_none() => {}
            "named" if record.selection.applied
                && record.selection.selection_id.as_deref().is_some_and(|id| !id.trim().is_empty()) => {}
            "all_eligible" => return Err("all-eligible plot statistics selection cannot carry an identity".into()),
            "named" => return Err("named plot statistics selection requires an applied identity".into()),
            other => return Err(format!("unknown plot statistics selection kind '{other}'")),
        }
        if record.finite_pair_count != record.values.count {
            return Err("plot statistics finite-pair count disagrees with its values".into());
        }
        if record.finite_pair_count == 0 {
            return Err("plot statistics records require a non-empty finite population".into());
        }
        let accounted = [
            record.finite_pair_count,
            record.exclusions.non_finite,
            record.exclusions.log_domain,
            record.exclusions.validity,
            record.exclusions.selection,
            record.exclusions.unpaired_or_unclassified,
        ]
        .into_iter()
        .try_fold(0_usize, |sum, count| sum.checked_add(count))
        .ok_or_else(|| "plot statistics exclusion counts overflow".to_string())?;
        if accounted != record.exclusions.input_count {
            return Err("plot statistics exclusions do not reconcile to the input count".into());
        }
        if record.exclusions.display_hidden > record.finite_pair_count {
            return Err("plot statistics display-hidden count exceeds the finite population".into());
        }
        if record.percentile_interpolation != "linear_index_n_minus_one" {
            return Err(format!(
                "unsupported percentile interpolation '{}'",
                record.percentile_interpolation
            ));
        }
        if record.standard_deviation != "sample_n_minus_one"
            && record.standard_deviation != "population_n"
        {
            return Err(format!(
                "unknown standard-deviation choice '{}'",
                record.standard_deviation
            ));
        }
        for (name, value) in [
            ("mean", record.values.mean),
            ("std", record.values.std),
            ("min", record.values.min),
            ("max", record.values.max),
            ("p5", record.values.p5),
            ("p25", record.values.p25),
            ("p50", record.values.p50),
            ("p75", record.values.p75),
            ("p95", record.values.p95),
        ] {
            if value.is_some_and(|number| !number.is_finite()) {
                return Err(format!("plot statistics {name} is non-finite"));
            }
        }
        let required = [
            ("mean", record.values.mean),
            ("min", record.values.min),
            ("max", record.values.max),
            ("p5", record.values.p5),
            ("p25", record.values.p25),
            ("p50", record.values.p50),
            ("p75", record.values.p75),
            ("p95", record.values.p95),
        ];
        if let Some((name, _)) = required.iter().find(|(_, value)| value.is_none()) {
            return Err(format!("plot statistics {name} is absent for a finite population"));
        }
        if record.values.std.is_none()
            && (record.standard_deviation == "population_n" || record.finite_pair_count > 1)
        {
            return Err("plot statistics standard deviation is absent for its estimator and population".into());
        }
        let min = record.values.min.unwrap();
        let max = record.values.max.unwrap();
        let ordered = [
            min,
            record.values.p5.unwrap(),
            record.values.p25.unwrap(),
            record.values.p50.unwrap(),
            record.values.p75.unwrap(),
            record.values.p95.unwrap(),
            max,
        ];
        if ordered.windows(2).any(|pair| pair[0] > pair[1]) {
            return Err("plot statistics percentiles are not monotone within min and max".into());
        }
        let mean = record.values.mean.unwrap();
        if mean < min || mean > max {
            return Err("plot statistics mean is outside min and max".into());
        }
    }
    Ok(())
}

const CURVE_HEADER_DISPLAY_DOC_TYPE: &str = "curve_header_display";
const CURVE_HEADER_DISPLAY_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CurveHeaderDisplayDocument {
    schema_version: u32,
    curve_id: String,
    range: DisplayRange,
}

fn validate_curve_header_display(
    curve_id: &str,
    range: DisplayRange,
) -> Result<(), String> {
    non_blank(curve_id, "curve id")?;
    if !range.low.is_finite() || !range.high.is_finite() || range.low == range.high {
        return Err("curve-header display range requires two distinct finite limits".into());
    }
    Ok(())
}

pub fn curve_header_display_range(
    conn: &Connection,
    curve_id: &str,
) -> Result<Option<DisplayRange>, String> {
    non_blank(curve_id, "curve id")?;
    let Some(document) = crate::db::list_documents(conn, CURVE_HEADER_DISPLAY_DOC_TYPE)
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|document| document.name == curve_id)
    else {
        return Ok(None);
    };
    let value: CurveHeaderDisplayDocument = serde_json::from_str(&document.json)
        .map_err(|error| format!("curve-header display range for {curve_id} is unreadable: {error}"))?;
    if value.schema_version != CURVE_HEADER_DISPLAY_SCHEMA_VERSION {
        return Err(format!(
            "curve-header display range for {curve_id} has unsupported schema version {}",
            value.schema_version
        ));
    }
    if value.curve_id != curve_id {
        return Err(format!(
            "curve-header display document key {curve_id} contains curve id {}",
            value.curve_id
        ));
    }
    validate_curve_header_display(curve_id, value.range)?;
    Ok(Some(value.range))
}

/// Typed, whitelisted curve-header metadata edit. Returning the previous object lets the
/// frontend register one exact undo/redo action; `None` deletes the declaration rather than
/// replacing it with sentinel numbers.
pub fn set_curve_header_display_range(
    conn: &Connection,
    curve_id: &str,
    range: Option<DisplayRange>,
) -> Result<Option<DisplayRange>, String> {
    let previous = curve_header_display_range(conn, curve_id)?;
    match range {
        Some(range) => {
            validate_curve_header_display(curve_id, range)?;
            let value = CurveHeaderDisplayDocument {
                schema_version: CURVE_HEADER_DISPLAY_SCHEMA_VERSION,
                curve_id: curve_id.into(),
                range,
            };
            let json = serde_json::to_string(&value).map_err(|error| error.to_string())?;
            crate::db::save_document(conn, CURVE_HEADER_DISPLAY_DOC_TYPE, curve_id, &json)
                .map_err(|error| error.to_string())?;
        }
        None => crate::db::delete_document(conn, CURVE_HEADER_DISPLAY_DOC_TYPE, curve_id)
            .map_err(|error| error.to_string())?,
    }
    Ok(previous)
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AxisRangeCandidates {
    pub user: Option<DisplayRange>,
    pub header_display: Option<DisplayRange>,
    pub audited_family_display: Option<DisplayRange>,
    pub finite_data: Option<DisplayRange>,
    /// Kept in the request so callers cannot accidentally omit the distinction.
    /// It is deliberately never consulted by `resolve_axis_range`.
    pub validity: Option<DisplayRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AxisRangeTier {
    User,
    HeaderDisplay,
    AuditedFamilyDisplay,
    FiniteData,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AxisRangeResolution {
    pub range: DisplayRange,
    pub tier: AxisRangeTier,
}

#[cfg(test)]
fn usable_range(range: DisplayRange) -> bool {
    range.low.is_finite() && range.high.is_finite() && range.low != range.high
}

#[cfg(test)]
pub fn resolve_axis_range(candidates: &AxisRangeCandidates) -> Result<AxisRangeResolution, String> {
    let ordered = [
        (candidates.user, AxisRangeTier::User),
        (candidates.header_display, AxisRangeTier::HeaderDisplay),
        (candidates.audited_family_display, AxisRangeTier::AuditedFamilyDisplay),
        (candidates.finite_data, AxisRangeTier::FiniteData),
    ];
    ordered
        .into_iter()
        .find_map(|(range, tier)| range.filter(|value| usable_range(*value)).map(|range| AxisRangeResolution { range, tier }))
        .ok_or_else(|| "no display range is available; validity limits are not display limits".into())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverlayAxisContract {
    pub quantity: String,
    pub canonical_unit: String,
    pub orientation: String,
    pub admissible_transform: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlotAxisSource {
    pub mnemonic: String,
    pub quantity: String,
    pub source_unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverlayAxisBinding {
    pub quantity: String,
    pub source_unit: String,
    pub display_unit: String,
    pub orientation: String,
    pub factor: f32,
    pub offset: f32,
    pub transform: String,
}

pub fn bind_overlay_axis(
    contract: &OverlayAxisContract,
    source: &PlotAxisSource,
) -> Result<OverlayAxisBinding, String> {
    if contract.quantity != source.quantity {
        return Err(format!(
            "overlay quantity {} is incompatible with plotted quantity {}",
            contract.quantity, source.quantity
        ));
    }
    let bridge = crate::curves::validate_unit_bridge(&source.source_unit, &contract.canonical_unit)
        .map_err(|error| error.to_string())?;
    if bridge.from_unit == bridge.to_unit {
        return Ok(OverlayAxisBinding {
            quantity: contract.quantity.clone(),
            source_unit: bridge.from_unit.into(),
            display_unit: bridge.to_unit.into(),
            orientation: contract.orientation.clone(),
            factor: 1.0,
            offset: 0.0,
            transform: "identity".into(),
        });
    }
    if contract.admissible_transform != "affine" {
        return Err(format!(
            "overlay axis admits {}, not a unit conversion",
            contract.admissible_transform
        ));
    }
    let rule = crate::curves::UNIT_RULES.iter().find(|rule| {
        crate::curves::validate_unit_bridge(rule.from_unit, rule.to_unit)
            .map(|candidate| {
                candidate.from_unit == bridge.from_unit && candidate.to_unit == bridge.to_unit
            })
            .unwrap_or(false)
    });
    let Some(rule) = rule else {
        return Err(format!(
            "no registered conversion from {} to {}",
            bridge.from_unit, bridge.to_unit
        ));
    };
    Ok(OverlayAxisBinding {
        quantity: contract.quantity.clone(),
        source_unit: bridge.from_unit.into(),
        display_unit: bridge.to_unit.into(),
        orientation: contract.orientation.clone(),
        factor: rule.factor as f32,
        offset: rule.offset as f32,
        transform: format!(
            "(source + {}) * {}; {}",
            rule.offset, rule.factor, rule.derivation
        ),
    })
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RangePolicyReport {
    pub input_count: usize,
    pub non_finite_excluded: usize,
    pub validity_excluded: usize,
    pub display_hidden: usize,
    pub statistics_count: usize,
    pub kept_values: Vec<f32>,
}

#[cfg(test)]
pub fn apply_range_policy(
    values: &[f32],
    display: DisplayRange,
    validity: Option<DisplayRange>,
    apply_validity: bool,
) -> RangePolicyReport {
    let mut report = RangePolicyReport {
        input_count: values.len(),
        non_finite_excluded: 0,
        validity_excluded: 0,
        display_hidden: 0,
        statistics_count: 0,
        kept_values: Vec::new(),
    };
    for &value in values {
        if !value.is_finite() {
            report.non_finite_excluded += 1;
            continue;
        }
        if apply_validity
            && validity
                .map(|range| value < range.low.min(range.high) || value > range.low.max(range.high))
                .unwrap_or(false)
        {
            report.validity_excluded += 1;
            continue;
        }
        report.statistics_count += 1;
        report.kept_values.push(value);
        if value < display.low.min(display.high) || value > display.low.max(display.high) {
            report.display_hidden += 1;
        }
    }
    report
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PercentageKind {
    PercentileP,
    RangePositionPct,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PercentileP {
    pub kind: PercentageKind,
    value: f32,
}

impl TryFrom<f32> for PercentileP {
    type Error = String;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_finite() && (0.0..=100.0).contains(&value) {
            Ok(Self { kind: PercentageKind::PercentileP, value })
        } else {
            Err("PercentileP must be finite and inside [0,100]".into())
        }
    }
}

impl PercentileP {
    pub fn value(self) -> f32 {
        self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RangePositionPct {
    pub kind: PercentageKind,
    value: f32,
}

impl RangePositionPct {
    pub fn new(value: f32) -> Result<Self, String> {
        if !value.is_finite() {
            return Err("RangePositionPct must be finite".into());
        }
        Ok(Self { kind: PercentageKind::RangePositionPct, value })
    }

    pub fn value(self) -> f32 {
        self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlotChannelPolicy {
    Cartesian { log_axis: bool, display: DisplayRange },
    Colour { log_axis: bool, display: DisplayRange },
    ArrayWaveform { log_axis: bool, display: DisplayRange },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RangeEdge {
    None,
    Low,
    High,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotChannelPolicyReport {
    pub values: Vec<f32>,
    pub included: Vec<bool>,
    pub edge_marks: Vec<RangeEdge>,
    pub non_finite_excluded: usize,
    pub log_domain_excluded: usize,
    pub display_clipped: usize,
    pub clamped: usize,
}

/// Applies the policy for one visual channel to a copy. Source samples are borrowed
/// and never changed: X/Y overflow is reported for clipping, while colour and array
/// waveform overflow are clamped in the derived display vector.
pub fn apply_plot_channel_policy(
    source: &[f32],
    policy: PlotChannelPolicy,
) -> PlotChannelPolicyReport {
    let mut report = PlotChannelPolicyReport {
        values: source.to_vec(),
        included: vec![false; source.len()],
        edge_marks: vec![RangeEdge::None; source.len()],
        non_finite_excluded: 0,
        log_domain_excluded: 0,
        display_clipped: 0,
        clamped: 0,
    };
    let (display, log_axis, clamp) = match policy {
        PlotChannelPolicy::Cartesian { log_axis, display } => (display, log_axis, false),
        PlotChannelPolicy::Colour { log_axis, display } => (display, log_axis, true),
        PlotChannelPolicy::ArrayWaveform { log_axis, display } => (display, log_axis, true),
    };
    let low = display.low.min(display.high);
    let high = display.low.max(display.high);
    for (index, &value) in source.iter().enumerate() {
        if !value.is_finite() {
            report.non_finite_excluded += 1;
            continue;
        }
        if log_axis && value <= 0.0 {
            report.log_domain_excluded += 1;
            continue;
        }
        report.included[index] = true;
        if value < low {
            if clamp {
                report.values[index] = low;
                report.edge_marks[index] = RangeEdge::Low;
                report.clamped += 1;
            } else {
                report.display_clipped += 1;
            }
        } else if value > high {
            if clamp {
                report.values[index] = high;
                report.edge_marks[index] = RangeEdge::High;
                report.clamped += 1;
            } else {
                report.display_clipped += 1;
            }
        }
    }
    report
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WellRequiredChannels {
    pub well_id: String,
    pub channels: Vec<Vec<f32>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WellPointAllocation {
    pub well_id: String,
    pub finite_pair_count: usize,
    pub quota: usize,
    pub source_indices: Vec<usize>,
    pub manifest: ReductionManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbsentWellAllocation {
    pub well_id: String,
    pub reason: String,
    pub quota: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiWellPointAllocation {
    pub wells: Vec<WellPointAllocation>,
    pub absent: Vec<AbsentWellAllocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReductionManifest {
    pub original_count: usize,
    pub displayed_count: usize,
    pub algorithm: String,
    pub stride: usize,
    pub endpoints_forced: bool,
    pub source_indices: Vec<usize>,
}

/// One disclosed reduction in a user-exportable plot manifest. This intentionally
/// carries counts and method metadata, not the numerical sample arrays themselves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReductionExportItem {
    pub subject_kind: String,
    pub subject_id: String,
    pub original_count: usize,
    pub displayed_count: usize,
    pub algorithm: String,
    pub stride: Option<usize>,
    pub endpoints_forced: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbsentReductionSubject {
    pub subject_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlotReductionExport {
    pub schema_version: u32,
    pub plot_type: String,
    pub items: Vec<ReductionExportItem>,
    pub absent: Vec<AbsentReductionSubject>,
    pub refusal: Option<String>,
}

/// Validates and formats the exact object written by the whitelisted manifest
/// command. A file with no actual reduction/refusal is not a reduction manifest.
pub fn serialize_reduction_export(export: &PlotReductionExport) -> Result<String, String> {
    if export.schema_version != 1 {
        return Err(format!(
            "unsupported plot reduction manifest schema version {}",
            export.schema_version
        ));
    }
    if export.plot_type.trim().is_empty() {
        return Err("plot reduction manifest is missing plot type".into());
    }
    let mut reduced = false;
    for item in &export.items {
        if item.subject_kind.trim().is_empty()
            || item.subject_id.trim().is_empty()
            || item.algorithm.trim().is_empty()
        {
            return Err("plot reduction item is missing subject or algorithm".into());
        }
        if item.displayed_count > item.original_count {
            return Err(format!(
                "plot reduction item {} displays more records than its original count",
                item.subject_id
            ));
        }
        if item.algorithm == "stride_from_first_with_forced_final_endpoint" {
            match item.stride {
                Some(0) | None => {
                    return Err(format!(
                        "stride reduction item {} is missing a positive stride",
                        item.subject_id
                    ));
                }
                Some(_) => {}
            }
            if item.endpoints_forced.is_none() {
                return Err(format!(
                    "stride reduction item {} is missing forced-endpoint state",
                    item.subject_id
                ));
            }
        }
        reduced |= item.displayed_count < item.original_count;
    }
    for absent in &export.absent {
        if absent.subject_id.trim().is_empty() || absent.reason.trim().is_empty() {
            return Err("absent plot subject is missing identity or reason".into());
        }
    }
    let refused = export
        .refusal
        .as_deref()
        .is_some_and(|reason| !reason.trim().is_empty());
    if export.refusal.is_some() && !refused {
        return Err("plot reduction refusal reason is blank".into());
    }
    if !reduced && !refused {
        return Err("plot reduction manifest contains no reduction or refusal".into());
    }
    serde_json::to_string_pretty(export).map_err(|error| error.to_string())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg(test)]
pub struct SharedChannelReduction {
    pub channels: Vec<Vec<f32>>,
    pub manifest: ReductionManifest,
}

fn stride_source_indices(eligible: &[usize], stride: usize) -> Result<(Vec<usize>, bool), String> {
    if stride == 0 {
        return Err("decimation stride must be at least 1".into());
    }
    if eligible.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err("eligible source indices must be strictly increasing".into());
    }
    let mut source_indices: Vec<usize> = eligible.iter().copied().step_by(stride).collect();
    let mut endpoints_forced = false;
    if let Some(&last) = eligible.last() {
        if source_indices.last().copied() != Some(last) {
            source_indices.push(last);
            endpoints_forced = true;
        }
    }
    Ok((source_indices, endpoints_forced))
}

#[cfg(test)]
pub fn decimate_shared_channels(
    channels: &[Vec<f32>],
    eligible: &[usize],
    stride: usize,
) -> Result<SharedChannelReduction, String> {
    let (source_indices, endpoints_forced) = stride_source_indices(eligible, stride)?;
    if let Some(index) = source_indices.last().copied() {
        if channels.iter().any(|channel| index >= channel.len()) {
            return Err("shared decimation index exceeds one or more channel lengths".into());
        }
    }
    let reduced = channels
        .iter()
        .map(|channel| source_indices.iter().map(|&index| channel[index]).collect())
        .collect();
    Ok(SharedChannelReduction {
        channels: reduced,
        manifest: ReductionManifest {
            original_count: eligible.len(),
            displayed_count: source_indices.len(),
            algorithm: "stride_from_first_with_forced_final_endpoint".into(),
            stride,
            endpoints_forced,
            source_indices,
        },
    })
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepthStepReconciliation {
    pub coarsest_step: f32,
    pub decimation_factors: Vec<usize>,
}

/// Chooses only among exact relationships. No tolerance or resampling kernel is
/// introduced: equality keeps factor 1, exact integer multiples decimate toward
/// the coarsest step, and every other relationship is routed to Data I/O.
#[cfg(test)]
pub fn reconcile_depth_steps(steps: &[f32]) -> Result<DepthStepReconciliation, String> {
    if steps.is_empty() || steps.iter().any(|step| !step.is_finite() || *step <= 0.0) {
        return Err("depth steps must be finite and positive".into());
    }
    let coarsest_step = steps.iter().copied().reduce(f32::max).unwrap();
    let mut decimation_factors = Vec::with_capacity(steps.len());
    for &step in steps {
        let ratio = coarsest_step / step;
        if !ratio.is_finite() || ratio < 1.0 || ratio.fract() != 0.0 {
            return Err(format!(
                "depth steps are not exact integer multiples; route this plot to the DIO resampling workflow ({step} versus {coarsest_step})"
            ));
        }
        decimation_factors.push(ratio as usize);
    }
    Ok(DepthStepReconciliation { coarsest_step, decimation_factors })
}

#[cfg(test)]
pub fn half_open_depth_indices(depth: &[f32], low: f32, high: f32) -> Vec<usize> {
    depth
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            (value.is_finite() && *value >= low && *value < high).then_some(index)
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWriteAxisBinding {
    pub channel: String,
    pub curve_id: String,
    pub mnemonic: String,
    pub quantity: String,
    pub source_unit: String,
    pub display_unit: String,
    pub conversion: String,
    pub source_revision: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlotWriteViewport {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub x_log: bool,
    pub y_log: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlotWriteSelection {
    pub kind: String,
    pub selection_id: Option<String>,
    pub member_count: usize,
    pub revision: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWriteInterval {
    pub low: Option<f32>,
    pub high: Option<f32>,
    pub closure: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWriteTarget {
    pub well_id: String,
    pub zone_name: String,
    pub parameter_name: String,
    pub value: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWriteProvenanceInput {
    pub plot_id: String,
    pub plot_type: String,
    pub x_axis: PlotWriteAxisBinding,
    pub y_axis: PlotWriteAxisBinding,
    pub z_axis: Option<PlotWriteAxisBinding>,
    pub viewport: PlotWriteViewport,
    pub selection: PlotWriteSelection,
    pub interval: PlotWriteInterval,
    pub method: String,
    pub fit_record: Option<serde_json::Value>,
    pub target: PlotWriteTarget,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWriteProvenance {
    pub source: PlotWriteProvenanceInput,
    pub user: String,
    pub timestamp_utc_ms: u64,
}

fn require_provenance_text(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("plot-derived write provenance is missing {field}"))
    } else {
        Ok(())
    }
}

fn validate_plot_write_axis(axis: &PlotWriteAxisBinding) -> Result<(), String> {
    for (value, field) in [
        (&axis.channel, "axis channel"),
        (&axis.curve_id, "axis curve id"),
        (&axis.mnemonic, "axis mnemonic"),
        (&axis.quantity, "axis quantity"),
        (&axis.source_unit, "axis source unit"),
        (&axis.display_unit, "axis display unit"),
        (&axis.conversion, "axis conversion"),
    ] {
        require_provenance_text(value, field)?;
    }
    if axis.source_revision.len() != 64
        || !axis.source_revision.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("plot-derived write provenance needs a SHA-256 source revision for every axis".into());
    }
    Ok(())
}

pub fn finalize_plot_write_provenance(
    source: Option<PlotWriteProvenanceInput>,
    user: &str,
    timestamp_utc_ms: u64,
) -> Result<PlotWriteProvenance, String> {
    let source = source.ok_or_else(|| "plot-derived write rejected: null source note".to_string())?;
    require_provenance_text(&source.plot_id, "plot id")?;
    require_provenance_text(&source.plot_type, "plot type")?;
    validate_plot_write_axis(&source.x_axis)?;
    validate_plot_write_axis(&source.y_axis)?;
    if let Some(axis) = &source.z_axis {
        validate_plot_write_axis(axis)?;
    }
    let viewport = source.viewport;
    if ![
        viewport.x_min,
        viewport.x_max,
        viewport.y_min,
        viewport.y_max,
    ]
    .into_iter()
    .all(f32::is_finite)
        || viewport.x_min == viewport.x_max
        || viewport.y_min == viewport.y_max
    {
        return Err("plot-derived write provenance needs a finite non-degenerate viewport".into());
    }
    require_provenance_text(&source.selection.kind, "selection kind")?;
    if source.selection.kind == "none" {
        if source.selection.member_count != 0 {
            return Err("a none selection must have zero members".into());
        }
    } else {
        let selection_id = source
            .selection
            .selection_id
            .as_deref()
            .ok_or_else(|| "an active selection needs an id".to_string())?;
        require_provenance_text(selection_id, "selection id")?;
        let revision = source
            .selection
            .revision
            .as_deref()
            .ok_or_else(|| "an active selection needs a revision".to_string())?;
        if revision.len() != 64 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("an active selection revision must be a SHA-256 digest".into());
        }
    }
    if source.interval.closure != "[lo,hi)" {
        return Err("plot-derived write interval must declare [lo,hi)".into());
    }
    if source.interval.low.is_some_and(|value| !value.is_finite())
        || source.interval.high.is_some_and(|value| !value.is_finite())
        || matches!((source.interval.low, source.interval.high), (Some(low), Some(high)) if low >= high)
    {
        return Err("plot-derived write provenance contains an invalid data interval".into());
    }
    require_provenance_text(&source.method, "method")?;
    if source.method.contains("fit") && source.fit_record.is_none() {
        return Err("a fit-derived write needs its fit record".into());
    }
    require_provenance_text(&source.target.well_id, "target well id")?;
    require_provenance_text(&source.target.zone_name, "target zone")?;
    require_provenance_text(&source.target.parameter_name, "target parameter")?;
    if !source.target.value.is_finite() {
        return Err("plot-derived write target value must be finite".into());
    }
    require_provenance_text(user, "user")?;
    if timestamp_utc_ms == 0 {
        return Err("plot-derived write provenance needs a timestamp".into());
    }
    Ok(PlotWriteProvenance {
        source,
        user: user.trim().into(),
        timestamp_utc_ms,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartRenderRecord {
    pub chart_id: String,
    pub title: String,
    pub chart_type: String,
    pub x_quantity: String,
    pub x_unit: String,
    pub y_quantity: String,
    pub y_unit: String,
    pub citation: String,
    pub publisher: String,
    pub revision_date: String,
    pub digitizer: Option<String>,
    pub approved_derivation_path: String,
    pub payload_checksum: String,
    pub transform_applied: String,
}

pub fn validate_chart_render_record(record: Option<&ChartRenderRecord>) -> Result<(), String> {
    let record = record.ok_or_else(|| {
        "deliverable chart rendering is blocked: chart provenance is absent".to_string()
    })?;
    for (value, field) in [
        (&record.chart_id, "chart id"),
        (&record.title, "chart title"),
        (&record.chart_type, "chart type"),
        (&record.x_quantity, "X quantity"),
        (&record.x_unit, "X unit"),
        (&record.y_quantity, "Y quantity"),
        (&record.y_unit, "Y unit"),
        (&record.citation, "citation"),
        (&record.publisher, "publisher"),
        (&record.revision_date, "source revision/date"),
        (&record.approved_derivation_path, "approved derivation path"),
        (&record.transform_applied, "transform applied"),
    ] {
        require_provenance_text(value, field)?;
    }
    if !matches!(
        record.approved_derivation_path.as_str(),
        "licensed_source" | "independently_digitized_public_primary_source"
    ) {
        return Err("deliverable chart rendering needs an approved derivation path".into());
    }
    if record.approved_derivation_path == "independently_digitized_public_primary_source" {
        let digitizer = record.digitizer.as_deref().ok_or_else(|| {
            "an independently digitized chart needs its digitizer".to_string()
        })?;
        require_provenance_text(digitizer, "digitizer")?;
    }
    if record.payload_checksum.len() != 64
        || !record
            .payload_checksum
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("deliverable chart rendering needs the rendered payload SHA-256 checksum".into());
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaperExportBounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaperExportRecord {
    pub schema_version: u32,
    pub medium: String,
    pub unit: String,
    pub source_width: f64,
    pub source_height: f64,
    pub margin_pt: f64,
    pub content_bounds: PaperExportBounds,
    pub page_bounds: PaperExportBounds,
    pub provenance_footer: String,
    pub crop_proof: String,
}

fn finite_ordered_bounds(bounds: &PaperExportBounds) -> bool {
    bounds.min_x.is_finite()
        && bounds.min_y.is_finite()
        && bounds.max_x.is_finite()
        && bounds.max_y.is_finite()
        && bounds.max_x >= bounds.min_x
        && bounds.max_y >= bounds.min_y
}

pub fn validate_paper_export_record(record: &PaperExportRecord) -> Result<(), String> {
    if !matches!(record.medium.as_str(), "svg-vector" | "pdf-vector" | "print-raster") {
        return Err("paper export has an unsupported output medium".into());
    }
    let raster = record.medium == "print-raster";
    if record.schema_version != 1 || record.unit != if raster { "px" } else { "pt" } {
        return Err("paper export has an unsupported schema or medium-specific unit".into());
    }
    if !record.source_width.is_finite()
        || !record.source_height.is_finite()
        || record.source_width <= 0.0
        || record.source_height <= 0.0
        || !record.margin_pt.is_finite()
        || record.margin_pt <= 0.0
        || !finite_ordered_bounds(&record.content_bounds)
        || !finite_ordered_bounds(&record.page_bounds)
    {
        return Err("paper export has invalid source geometry, margin or bounds".into());
    }
    if record.content_bounds.min_x > 0.0
        || record.content_bounds.min_y > 0.0
        || record.content_bounds.max_x < record.source_width
        || record.content_bounds.max_y < record.source_height
    {
        return Err("paper export source canvas is cropped by its declared content bounds".into());
    }
    if record.page_bounds.min_x > record.content_bounds.min_x
        || record.page_bounds.min_y > record.content_bounds.min_y
        || record.page_bounds.max_x < record.content_bounds.max_x
        || record.page_bounds.max_y < record.content_bounds.max_y
    {
        return Err("paper export content is cropped by its declared page".into());
    }
    require_provenance_text(&record.provenance_footer, "paper provenance footer")?;
    let expected_proof = if raster {
        "raster_pixels_preserved_before_browser_print_layout"
    } else {
        "all_recorded_bounds_inside_page"
    };
    if record.crop_proof != expected_proof {
        return Err("paper export lacks the measured no-crop proof".into());
    }
    Ok(())
}

/// Screens aligned required channels before assigning any part of the total point
/// budget. Each represented well first receives enough capacity for both eligible
/// endpoints (or its single eligible sample), then remaining capacity is shared in
/// stable input order without exceeding the total budget.
pub fn allocate_finite_pair_budget(
    wells: &[WellRequiredChannels],
    budget: usize,
) -> Result<MultiWellPointAllocation, String> {
    let mut absent = Vec::new();
    let mut screened: Vec<(String, Vec<usize>)> = Vec::new();
    for well in wells {
        let aligned_len = well.channels.iter().map(Vec::len).min().unwrap_or(0);
        let eligible: Vec<usize> = (0..aligned_len)
            .filter(|&index| well.channels.iter().all(|channel| channel[index].is_finite()))
            .collect();
        if eligible.is_empty() {
            absent.push(AbsentWellAllocation {
                well_id: well.well_id.clone(),
                reason: "zero finite aligned pairs across required channels".into(),
                quota: 0,
            });
        } else {
            screened.push((well.well_id.clone(), eligible));
        }
    }
    if screened.is_empty() {
        return Ok(MultiWellPointAllocation { wells: Vec::new(), absent });
    }
    let minimum_required: usize = screened
        .iter()
        .map(|(_, eligible)| eligible.len().min(2))
        .sum();
    if budget < minimum_required {
        return Err(format!(
            "point budget {budget} cannot retain both endpoints for {} represented wells; at least {minimum_required} points are required",
            screened.len()
        ));
    }
    let mut quotas: Vec<usize> = screened
        .iter()
        .map(|(_, eligible)| eligible.len().min(2))
        .collect();
    let mut remaining = budget - minimum_required;
    while remaining > 0 {
        let mut advanced = false;
        for (index, (_, eligible)) in screened.iter().enumerate() {
            if remaining == 0 {
                break;
            }
            if quotas[index] < eligible.len() {
                quotas[index] += 1;
                remaining -= 1;
                advanced = true;
            }
        }
        if !advanced {
            break;
        }
    }
    let wells = screened
        .into_iter()
        .zip(quotas)
        .map(|((well_id, eligible), quota)| {
            let stride = if quota >= eligible.len() {
                1
            } else {
                (eligible.len() - 1).div_ceil(quota - 1)
            };
            let (source_indices, endpoints_forced) = stride_source_indices(&eligible, stride)
                .expect("screened eligible indices are strictly increasing and stride is positive");
            let manifest = ReductionManifest {
                original_count: eligible.len(),
                displayed_count: source_indices.len(),
                algorithm: "stride_from_first_with_forced_final_endpoint".into(),
                stride,
                endpoints_forced,
                source_indices: source_indices.clone(),
            };
            WellPointAllocation {
                well_id,
                finite_pair_count: eligible.len(),
                quota: source_indices.len(),
                source_indices,
                manifest,
            }
        })
        .collect();
    Ok(MultiWellPointAllocation { wells, absent })
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PickettFit {
    pub m: f32,
    pub a_rw: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourcedPickettValue {
    pub value: f32,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PickettDisclosure {
    pub m: f32,
    pub a_rw: f32,
    pub a: Option<SourcedPickettValue>,
    pub rw: Option<SourcedPickettValue>,
}

fn validate_sourced_pickett_value(
    name: &str,
    value: &SourcedPickettValue,
) -> Result<(), String> {
    if !value.value.is_finite() || value.value <= 0.0 {
        return Err(format!("sourced Pickett {name} must be finite and positive"));
    }
    if value.provenance.trim().is_empty() {
        return Err(format!("sourced Pickett {name} requires provenance"));
    }
    Ok(())
}

/// Discloses only what a two-point Pickett fit identifies. The intercept is the
/// product a·Rw. One independently sourced factor may separate the other; neither
/// factor is otherwise inferred, and no scientific default is introduced.
pub fn disclose_pickett_fit(
    fit: PickettFit,
    supplied_a: Option<SourcedPickettValue>,
    supplied_rw: Option<SourcedPickettValue>,
) -> Result<PickettDisclosure, String> {
    if !fit.m.is_finite() || fit.m <= 0.0 || !fit.a_rw.is_finite() || fit.a_rw <= 0.0 {
        return Err("Pickett fit m and a·Rw must be finite and positive".into());
    }
    if supplied_a.is_some() && supplied_rw.is_some() {
        return Err(
            "supply only one independently sourced Pickett factor; the fitted a·Rw derives the other"
                .into(),
        );
    }
    if let Some(a) = supplied_a {
        validate_sourced_pickett_value("a", &a)?;
        let rw = SourcedPickettValue {
            value: fit.a_rw / a.value,
            provenance: format!(
                "derived from fitted a·Rw using sourced a: {}",
                a.provenance
            ),
        };
        return Ok(PickettDisclosure {
            m: fit.m,
            a_rw: fit.a_rw,
            a: Some(a),
            rw: Some(rw),
        });
    }
    if let Some(rw) = supplied_rw {
        validate_sourced_pickett_value("Rw", &rw)?;
        let a = SourcedPickettValue {
            value: fit.a_rw / rw.value,
            provenance: format!(
                "derived from fitted a·Rw using sourced Rw: {}",
                rw.provenance
            ),
        };
        return Ok(PickettDisclosure {
            m: fit.m,
            a_rw: fit.a_rw,
            a: Some(a),
            rw: Some(rw),
        });
    }
    Ok(PickettDisclosure {
        m: fit.m,
        a_rw: fit.a_rw,
        a: None,
        rw: None,
    })
}

fn non_blank(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("resolved plot curve is missing {field}"))
    } else {
        Ok(())
    }
}

/// Validates the durable binding record before a plot may keep or export it.
/// A required intent with no concrete per-well answer is an error, never an
/// invitation to substitute a same-named curve later.
pub fn persist_plot_binding(
    intent: PlotChannelIntent,
    resolved: Vec<ResolvedPlotCurve>,
) -> Result<PlotChannelBinding, String> {
    non_blank(&intent.channel, "channel")?;
    non_blank(&intent.semantic_request, "semantic request")?;
    if intent.required && resolved.is_empty() {
        return Err(format!(
            "required channel '{}' could not be resolved",
            intent.semantic_request
        ));
    }
    for curve in &resolved {
        non_blank(&curve.well_id, "well id")?;
        non_blank(&curve.curve_id, "curve id")?;
        non_blank(&curve.mnemonic, "mnemonic")?;
        non_blank(&curve.quantity, "quantity")?;
        non_blank(&curve.source_unit, "source unit")?;
        non_blank(&curve.display_unit, "display unit")?;
        non_blank(&curve.conversion, "conversion")?;
        non_blank(&curve.resolution_reason, "resolution reason")?;
        if curve.source_revision.len() != 64
            || !curve.source_revision.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err("resolved plot curve source revision must be a SHA-256 digest".into());
        }
    }
    Ok(PlotChannelBinding { intent, resolved })
}

fn quantity_name(unit: &str, mnemonic_or_family: &str) -> Option<String> {
    let kind = crate::curves::resolve_unit_token(unit)
        .map(|entry| entry.quantity_kind)
        .or_else(|| {
            crate::curves::family_for(mnemonic_or_family)
                .and_then(|family| crate::curves::resolve_unit_token(family.canonical_unit))
                .map(|entry| entry.quantity_kind)
        })?;
    let name = match kind {
        crate::curves::QuantityKind::GammaRay => "gamma_ray",
        crate::curves::QuantityKind::ElectricPotential => "electric_potential",
        crate::curves::QuantityKind::Length => "length",
        crate::curves::QuantityKind::BulkDensity => "bulk_density",
        crate::curves::QuantityKind::PhotoelectricFactor => "photoelectric_factor",
        crate::curves::QuantityKind::Fraction => "fraction",
        crate::curves::QuantityKind::Slowness => "slowness",
        crate::curves::QuantityKind::Temperature => "temperature",
        crate::curves::QuantityKind::Resistivity => "resistivity",
        crate::curves::QuantityKind::ChargePerVolume => "charge_per_volume",
        crate::curves::QuantityKind::Permeability => "permeability",
        crate::curves::QuantityKind::Categorical => "categorical",
    };
    Some(name.into())
}

fn plotted_bytes(conn: &Connection, well_id: &str, request: &str) -> Result<Vec<u8>, String> {
    let series = crate::equations::fetch_curve_data(
        conn,
        well_id,
        &[request.to_string()],
        None,
        None,
    )
    .map_err(|error| error.to_string())?;
    Ok(series.into_iter().next().map(|item| item.data).unwrap_or_default())
}

fn standard_source(request: &str) -> Option<(&'static str, &'static str)> {
    let column = crate::schema_vocab::standard_column(request)?;
    let unit = if column.mnemonic == "DEPTH" {
        "m"
    } else {
        crate::curves::family_for(column.mnemonic)?.canonical_unit
    };
    Some((column.storage_column, unit))
}

fn finite_standard_count(conn: &Connection, well_id: &str, column: &str) -> i64 {
    // `column` is selected exclusively by `standard_source`; it is never caller SQL.
    conn.query_row(
        &format!(
            "SELECT COUNT(*) FROM standard_curves WHERE well_id = ?1 AND {column} IS NOT NULL AND isfinite({column})"
        ),
        params![well_id],
        |row| row.get(0),
    )
    .unwrap_or(0)
}

fn resolve_one_curve(
    conn: &Connection,
    well_id: &str,
    semantic_request: &str,
) -> Result<Option<ResolvedPlotCurve>, String> {
    let request = semantic_request.trim().to_uppercase();
    if request.is_empty() {
        return Ok(None);
    }
    let bytes = plotted_bytes(conn, well_id, &request)?;
    let source_revision = format!("{:x}", Sha256::digest(&bytes));

    if let Some((column, unit)) = standard_source(&request) {
        let count = finite_standard_count(conn, well_id, column);
        if count > 0 {
            let quantity = quantity_name(unit, &request)
                .ok_or_else(|| format!("{request} has no typed quantity for unit {unit}"))?;
            let curve_id = format!("standard:{well_id}:{request}");
            let header_display = curve_header_display_range(conn, &curve_id)?;
            return Ok(Some(ResolvedPlotCurve {
                well_id: well_id.into(),
                curve_id,
                mnemonic: request,
                quantity,
                source_unit: unit.into(),
                display_unit: unit.into(),
                conversion: "identity".into(),
                sample_count: count as usize,
                resolution_reason: "finite standard curve wins the plot resolution order".into(),
                source_revision,
                header_display,
            }));
        }
    }

    let computed_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM computed_curves
             WHERE well_id = ?1 AND upper(curve_name) = ?2
               AND value IS NOT NULL AND isfinite(value)",
            params![well_id, request],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if computed_count > 0 {
        let unit = crate::db::curve_unit_for(conn, well_id, &request)
            .ok_or_else(|| format!("resolved computed curve {request} has no declared source unit"))?;
        let quantity = quantity_name(&unit, &request)
            .ok_or_else(|| format!("resolved computed curve {request} has no typed quantity for unit {unit}"))?;
        let curve_id = format!("computed:{well_id}:{request}");
        let header_display = curve_header_display_range(conn, &curve_id)?;
        return Ok(Some(ResolvedPlotCurve {
            well_id: well_id.into(),
            curve_id,
            mnemonic: request,
            quantity,
            source_unit: unit.clone(),
            display_unit: unit,
            conversion: "identity".into(),
            sample_count: computed_count as usize,
            resolution_reason: "exact computed mnemonic after no finite standard curve".into(),
            source_revision,
            header_display,
        }));
    }

    // SB-DIO-034: a track request is a SEMANTIC request (a GR track shows the well's GRN
    // where that is what was delivered) - and the concrete mnemonic travels back to the
    // header via resolution_reason, so the substitution is visible, never silent.
    let generic = crate::equations::resolve_generic_curve_id(
        conn,
        well_id,
        &request,
        crate::equations::CurveRequest::SemanticFamily,
    )
        .map_err(|error| error.to_string())?
        .and_then(|curve_id| conn
        .query_row(
            "SELECT curve_id, mnemonic, unit, family,
                    (SELECT COUNT(*) FROM curve_samples s
                     WHERE s.curve_id = m.curve_id AND s.value IS NOT NULL AND isfinite(s.value)),
                    set_name, COALESCE(pinned, 0), run_no
             FROM curve_meta m
             WHERE curve_id = ?1",
            params![curve_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i32>(6)? != 0,
                    row.get::<_, Option<i32>>(7)?,
                ))
            },
        )
        .ok());
    let Some((curve_id, mnemonic, unit, family, count, set_name, pinned, run_no)) = generic else {
        return Ok(None);
    };
    if count == 0 {
        return Ok(None);
    }
    let unit = unit.ok_or_else(|| format!("resolved imported curve {mnemonic} has no declared source unit"))?;
    let quantity = family
        .as_deref()
        .and_then(|name| quantity_name(&unit, name))
        .or_else(|| quantity_name(&unit, &mnemonic))
        .ok_or_else(|| format!("resolved imported curve {mnemonic} has no typed quantity for unit {unit}"))?;
    let match_kind = if mnemonic.eq_ignore_ascii_case(&request) { "exact mnemonic" } else { "typed family" };
    let pin_note = if pinned { ", user-pinned" } else { "" };
    let run_note = run_no.map(|value| format!(", run {value}")).unwrap_or_default();
    let header_display = curve_header_display_range(conn, &curve_id)?;
    Ok(Some(ResolvedPlotCurve {
        well_id: well_id.into(),
        curve_id,
        mnemonic,
        quantity,
        source_unit: unit.clone(),
        display_unit: unit,
        conversion: "identity".into(),
        sample_count: count as usize,
        resolution_reason: format!("{match_kind} in set {set_name}{pin_note}{run_note}"),
        source_revision,
        header_display,
    }))
}

/// Resolves every semantic request independently in every well and then validates
/// the durable record. Required channels fail the entire plot build when any well
/// cannot supply a concrete typed curve.
pub fn resolve_plot_bindings(
    conn: &Connection,
    intents: Vec<PlotChannelIntent>,
    well_ids: &[String],
) -> Result<Vec<PlotChannelBinding>, String> {
    intents
        .into_iter()
        .map(|intent| {
            let mut resolved = Vec::with_capacity(well_ids.len());
            for well_id in well_ids {
                if let Some(curve) = resolve_one_curve(conn, well_id, &intent.semantic_request)? {
                    resolved.push(curve);
                } else if intent.required {
                    return Err(format!(
                        "required channel '{}' is unresolved for well {}",
                        intent.semantic_request, well_id
                    ));
                }
            }
            persist_plot_binding(intent, resolved)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SB-DIO-034 (DEC-030): the plot surface's family selection is a TYPED semantic request
    /// whose answer NAMES the concrete curve it chose - a GR track showing the well's GRN says
    /// so in its resolution reason, never a silent stand-in - and an exact-mnemonic hit says
    /// that instead, so the two cases can never be confused on a header.
    #[test]
    fn a_family_resolved_track_names_the_concrete_curve_it_chose_never_a_silent_stand_in() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, id, "SANDI-TRACK", None, None, None).unwrap();
        let well = id.to_string();
        let curve = crate::db::upsert_curve_meta(
            &conn, &well, "RAW", "GRN", Some("gAPI"), Some("GR"), None, None,
        )
        .unwrap();
        crate::db::insert_curve_samples(&conn, &curve, &[1000.0, 1001.0], &[50.0, 60.0])
            .unwrap();
        let resolved = resolve_one_curve(&conn, &well, "GR")
            .unwrap()
            .expect("the family request resolves");
        assert_eq!(resolved.mnemonic, "GRN", "the concrete identity travels to the header");
        assert!(
            resolved.resolution_reason.contains("typed family"),
            "the substitution is visible: {}",
            resolved.resolution_reason
        );
        let exact = resolve_one_curve(&conn, &well, "GRN")
            .unwrap()
            .expect("the exact request resolves");
        assert!(
            exact.resolution_reason.contains("exact mnemonic"),
            "an exact hit says so: {}",
            exact.resolution_reason
        );
    }

    #[test]
    fn a_plot_binding_keeps_the_request_and_each_wells_concrete_resolution() {
        let intent = PlotChannelIntent {
            channel: "x".into(),
            semantic_request: "bulk density".into(),
            required: true,
        };
        let concrete = ResolvedPlotCurve {
            well_id: "00000000-0000-0000-0000-000000000001".into(),
            curve_id: "curve-1".into(),
            mnemonic: "RHOB".into(),
            quantity: "bulk_density".into(),
            source_unit: "kg/m3".into(),
            display_unit: "g/cc".into(),
            conversion: "(source + 0) * 0.001".into(),
            sample_count: 3,
            resolution_reason: "exact mnemonic in active delivery".into(),
            source_revision: "a".repeat(64),
            header_display: None,
        };

        let binding = persist_plot_binding(intent.clone(), vec![concrete.clone()]).unwrap();
        assert_eq!(binding.intent, intent);
        assert_eq!(binding.resolved, vec![concrete]);

        let error = persist_plot_binding(
            PlotChannelIntent { channel: "y".into(), semantic_request: "neutron porosity".into(), required: true },
            Vec::new(),
        )
        .unwrap_err();
        assert!(error.contains("required channel"));
    }

    #[test]
    fn a_saved_plot_template_and_export_keep_one_request_and_each_wells_distinct_concrete_resolution() {
        // CORRECTNESS: docs/PRD_v2/23_plotting-interactivity.md §4.1, SB-PLT-001.
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let well_a = "00000000-0000-0000-0000-000000000001".to_string();
        let well_b = "00000000-0000-0000-0000-000000000002".to_string();
        let binding = PlotChannelBinding {
            intent: PlotChannelIntent {
                channel: "x".into(),
                semantic_request: "bulk density".into(),
                required: true,
            },
            resolved: vec![
                ResolvedPlotCurve {
                    well_id: well_a.clone(),
                    curve_id: "curve-density-a".into(),
                    mnemonic: "RHOB".into(),
                    quantity: "bulk_density".into(),
                    source_unit: "kg/m3".into(),
                    display_unit: "g/cc".into(),
                    conversion: "source * 0.001".into(),
                    sample_count: 3,
                    resolution_reason: "typed family in imported set WIRE".into(),
                    source_revision: "a".repeat(64),
                    header_display: None,
                },
                ResolvedPlotCurve {
                    well_id: well_b.clone(),
                    curve_id: "curve-density-b".into(),
                    mnemonic: "RHOZ".into(),
                    quantity: "bulk_density".into(),
                    source_unit: "g/cc".into(),
                    display_unit: "g/cc".into(),
                    conversion: "identity".into(),
                    sample_count: 4,
                    resolution_reason: "typed family in imported set RAW".into(),
                    source_revision: "b".repeat(64),
                    header_display: None,
                },
            ],
        };
        let state = PersistedPlotState {
            schema_version: PLOT_STATE_SCHEMA_VERSION,
            plot_type: "crossplot".into(),
            well_ids: vec![well_a.clone(), well_b.clone()],
            options: serde_json::json!({"x": "bulk density", "y": "neutron porosity"}),
            bindings: vec![binding.clone()],
            axis_ranges: vec![PlotAxisRange {
                axis: "x".into(),
                min: 1.95,
                max: 2.95,
                tier: AxisRangeTier::AuditedFamilyDisplay,
            }],
        };

        save_persisted_plot_state(&conn, "plotprops", "crossplot", &state).unwrap();
        save_persisted_plot_state(&conn, "plottmpl:crossplot", "Density comparison", &state)
            .unwrap();

        let project = list_persisted_plot_states(&conn, "plotprops").unwrap();
        let templates = list_persisted_plot_states(&conn, "plottmpl:crossplot").unwrap();
        assert_eq!(project[0].state, state);
        assert_eq!(templates[0].state, state);
        assert_ne!(
            project[0].state.bindings[0].resolved[0].curve_id,
            project[0].state.bindings[0].resolved[1].curve_id,
            "the same request must retain each well's different concrete answer",
        );

        let export_json = serialize_plot_binding_export(
            &state.well_ids,
            &state.bindings,
            &state.axis_ranges,
            &[],
        )
        .unwrap();
        let exported: PlotBindingExport = serde_json::from_str(&export_json).unwrap();
        assert_eq!(exported.well_ids, vec![well_a.clone(), well_b.clone()]);
        assert_eq!(exported.bindings, vec![binding]);
        assert_eq!(exported.axis_ranges, state.axis_ranges);
        assert!(exported.statistics_records.is_empty());
        let svg = crate::composite::embed_plot_bindings_json_in_svg(
            "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>",
            &export_json,
        )
        .unwrap();
        assert!(svg.contains("sandibumi-plot-bindings-v1"));
        assert!(svg.contains("curve-density-a"));
        assert!(svg.contains("curve-density-b"));

        let unresolved = PersistedPlotState {
            schema_version: PLOT_STATE_SCHEMA_VERSION,
            plot_type: "crossplot".into(),
            well_ids: vec![well_a, well_b],
            options: serde_json::json!({"x": "bulk density"}),
            bindings: vec![PlotChannelBinding {
                intent: PlotChannelIntent {
                    channel: "x".into(),
                    semantic_request: "bulk density".into(),
                    required: true,
                },
                resolved: Vec::new(),
            }],
            axis_ranges: vec![PlotAxisRange {
                axis: "x".into(),
                min: 1.95,
                max: 2.95,
                tier: AxisRangeTier::AuditedFamilyDisplay,
            }],
        };
        let save_error =
            save_persisted_plot_state(&conn, "plotprops", "unresolved", &unresolved)
                .unwrap_err();
        assert!(save_error.contains("required channel"));
        assert!(list_persisted_plot_states(&conn, "plotprops")
            .unwrap()
            .iter()
            .all(|document| document.name != "unresolved"));
        let export_error =
            serialize_plot_binding_export(
                &unresolved.well_ids,
                &unresolved.bindings,
                &unresolved.axis_ranges,
                &[],
            )
                .unwrap_err();
        assert!(export_error.contains("required channel"));
    }

    #[test]
    fn a_plot_statistics_export_preserves_a_reconciled_record_and_refuses_unreconciled_exclusions() {
        // CORRECTNESS - SB-PLT-009 / T12, 23_plotting-interactivity.md sections 4.2 and 6.
        // T12 supplies [1,2,3,NaN,+Inf] => n=3, mean/P50=2 and two exclusions; the
        // remaining percentiles are independent linear-index-(n-1) arithmetic on [1,2,3].
        let well_id = "00000000-0000-0000-0000-000000000001".to_string();
        let binding = PlotChannelBinding {
            intent: PlotChannelIntent {
                channel: "x".into(),
                semantic_request: "value".into(),
                required: true,
            },
            resolved: vec![ResolvedPlotCurve {
                well_id: well_id.clone(),
                curve_id: "curve-value".into(),
                mnemonic: "VALUE".into(),
                quantity: "unspecified".into(),
                source_unit: "unitless".into(),
                display_unit: "unitless".into(),
                conversion: "identity".into(),
                sample_count: 5,
                resolution_reason: "exact mnemonic in active delivery".into(),
                source_revision: "a".repeat(64),
                header_display: None,
            }],
        };
        let axis_ranges = vec![PlotAxisRange {
            axis: "x".into(),
            min: 1.0,
            max: 3.0,
            tier: AxisRangeTier::FiniteData,
        }];
        let record = PlotStatisticsRecord {
            schema_version: 1,
            binding_channel: "x".into(),
            channel: "x:VALUE".into(),
            population: "active_well".into(),
            well_ids: vec![well_id.clone()],
            interval: PlotStatisticsInterval {
                low: Some(100.0),
                high: Some(101.0),
                closure: "[lo,hi)".into(),
            },
            selection: PlotStatisticsSelection {
                kind: "all_eligible".into(),
                selection_id: None,
                label: "all eligible".into(),
                applied: false,
            },
            finite_pair_count: 3,
            exclusions: PlotStatisticsExclusions {
                input_count: 5,
                non_finite: 2,
                log_domain: 0,
                validity: 0,
                selection: 0,
                unpaired_or_unclassified: 0,
                display_hidden: 0,
            },
            percentile_interpolation: "linear_index_n_minus_one".into(),
            standard_deviation: "sample_n_minus_one".into(),
            values: PlotStatisticsValues {
                count: 3,
                mean: Some(2.0),
                std: Some(1.0),
                min: Some(1.0),
                max: Some(3.0),
                p5: Some(1.1),
                p25: Some(1.5),
                p50: Some(2.0),
                p75: Some(2.5),
                p95: Some(2.9),
            },
        };

        let json = serialize_plot_binding_export(
            std::slice::from_ref(&well_id),
            std::slice::from_ref(&binding),
            &axis_ranges,
            std::slice::from_ref(&record),
        )
        .unwrap();
        let exported: PlotBindingExport = serde_json::from_str(&json).unwrap();
        assert_eq!(exported.statistics_records, vec![record.clone()]);

        let mut unreconciled = record.clone();
        unreconciled.exclusions.selection = 1;
        let error = serialize_plot_binding_export(
            std::slice::from_ref(&well_id),
            std::slice::from_ref(&binding),
            &axis_ranges,
            &[unreconciled],
        )
        .unwrap_err();
        assert!(error.contains("do not reconcile"));

        let mut unbound = record.clone();
        unbound.binding_channel = "y".into();
        let error = serialize_plot_binding_export(
            std::slice::from_ref(&well_id),
            std::slice::from_ref(&binding),
            &axis_ranges,
            &[unbound],
        )
        .unwrap_err();
        assert!(error.contains("unbound channel"));

        let mut foreign = record;
        foreign.well_ids = vec!["00000000-0000-0000-0000-000000000002".into()];
        let error = serialize_plot_binding_export(
            std::slice::from_ref(&well_id),
            std::slice::from_ref(&binding),
            &axis_ranges,
            &[foreign],
        )
        .unwrap_err();
        assert!(error.contains("unrepresented well"));
    }

    #[test]
    fn a_user_axis_range_wins_and_without_it_the_header_display_range_wins() {
        let candidates = AxisRangeCandidates {
            user: Some(DisplayRange { low: 10.0, high: 20.0 }),
            header_display: Some(DisplayRange { low: 1.0, high: 2.0 }),
            audited_family_display: Some(DisplayRange { low: 3.0, high: 4.0 }),
            finite_data: Some(DisplayRange { low: 5.0, high: 6.0 }),
            validity: Some(DisplayRange { low: 100.0, high: 200.0 }),
        };
        let user = resolve_axis_range(&candidates).unwrap();
        assert_eq!(user.tier, AxisRangeTier::User);
        assert_eq!(user.range, DisplayRange { low: 10.0, high: 20.0 });

        let without_user = AxisRangeCandidates { user: None, ..candidates };
        let header = resolve_axis_range(&without_user).unwrap();
        assert_eq!(header.tier, AxisRangeTier::HeaderDisplay);
        assert_eq!(header.range, DisplayRange { low: 1.0, high: 2.0 });

        let validity_only = AxisRangeCandidates {
            user: None,
            header_display: None,
            audited_family_display: None,
            finite_data: None,
            validity: Some(DisplayRange { low: 100.0, high: 200.0 }),
        };
        assert!(resolve_axis_range(&validity_only).is_err());
    }

    #[test]
    fn an_overlay_requires_quantity_compatible_units_and_records_any_registered_conversion() {
        let incompatible = bind_overlay_axis(
            &OverlayAxisContract {
                quantity: "bulk_density".into(),
                canonical_unit: "g/cc".into(),
                orientation: "y".into(),
                admissible_transform: "identity".into(),
            },
            &PlotAxisSource {
                mnemonic: "RHOB".into(),
                quantity: "gamma_ray".into(),
                source_unit: "gAPI".into(),
            },
        );
        assert!(incompatible.unwrap_err().contains("quantity"));

        let converted = bind_overlay_axis(
            &OverlayAxisContract {
                quantity: "slowness".into(),
                canonical_unit: "us/ft".into(),
                orientation: "x".into(),
                admissible_transform: "affine".into(),
            },
            &PlotAxisSource {
                mnemonic: "DT".into(),
                quantity: "slowness".into(),
                source_unit: "us/m".into(),
            },
        )
        .unwrap();
        assert_eq!(converted.source_unit, "us/m");
        assert_eq!(converted.display_unit, "us/ft");
        assert_eq!(converted.factor, 0.3048);
        assert_eq!(converted.offset, 0.0);
        assert!(converted.transform.contains("0.3048"));
    }

    #[test]
    fn display_clipping_counts_hidden_points_while_validity_filtering_changes_the_population_explicitly() {
        let values = [0.0, 1.0, 2.0, 3.0, 4.0];
        let clipped = apply_range_policy(
            &values,
            DisplayRange { low: 1.0, high: 3.0 },
            Some(DisplayRange { low: 1.0, high: 3.0 }),
            false,
        );
        assert_eq!(clipped.statistics_count, 5);
        assert_eq!(clipped.display_hidden, 2);
        assert_eq!(clipped.validity_excluded, 0);

        let filtered = apply_range_policy(
            &values,
            DisplayRange { low: 0.0, high: 4.0 },
            Some(DisplayRange { low: 1.0, high: 3.0 }),
            true,
        );
        assert_eq!(filtered.statistics_count, 3);
        assert_eq!(filtered.validity_excluded, 2);
        assert_eq!(filtered.display_hidden, 0);
        assert_eq!(filtered.kept_values, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn the_documented_attenuation_pair_is_6_56x_divergent_and_exceeds_the_cited_screen_in_the_wrong_direction() {
        // CORRECTNESS — critique A-4, dossier §3.3a and SB-PLT-T05. One international
        // foot is exactly 0.3048 m, so 100 dB/ft is 328.08398... dB/m, not 50 dB/m.
        let source_db_per_ft = 100.0_f32;
        let declared_db_per_m = 50.0_f32;
        let expected_db_per_m = source_db_per_ft / 0.3048_f32;
        let divergence_factor = expected_db_per_m / declared_db_per_m;
        let relative_error = (expected_db_per_m - declared_db_per_m) / expected_db_per_m;

        assert_eq!((divergence_factor * 100.0).round() / 100.0, 6.56);
        assert!(relative_error > 0.15, "the row exceeds the cited 15% activation screen");
        assert!(expected_db_per_m > source_db_per_ft, "a per-metre rate must be numerically larger");
        assert!(declared_db_per_m < source_db_per_ft, "the documented alternate moved the wrong way");
    }

    #[test]
    fn histogram_bins_are_half_open_except_for_the_final_upper_endpoint_and_non_finite_values_are_counted() {
        // CORRECTNESS - SB-PLT-006 / T06-T07, 23_plotting-interactivity.md sections 5-6.
        assert_eq!(crate::distribution::HISTOGRAM_BINS_MIN, 1);
        assert_eq!(crate::distribution::HISTOGRAM_BINS_MAX, 200);

        let endpoints = crate::distribution::canonical_histogram(
            &[0.0, 1.0, 2.0, 3.0],
            0.0,
            3.0,
            3,
        );
        assert_eq!(endpoints.counts, vec![1, 1, 2]);
        assert_eq!(endpoints.displayed_total, 4);
        assert_eq!(endpoints.non_finite_excluded, 0);

        let missing = crate::distribution::canonical_histogram(
            &[0.0, f32::NAN, f32::INFINITY, 1.0],
            0.0,
            1.0,
            3,
        );
        assert_eq!(missing.displayed_total, 2);
        assert_eq!(missing.counts.iter().sum::<u32>(), 2);
        assert_eq!(missing.non_finite_excluded, 2);
    }

    #[test]
    fn percentile_probability_rejects_130_while_range_position_preserves_130_and_minus_5() {
        assert!(PercentileP::try_from(130.0).is_err());
        assert_eq!(PercentileP::try_from(0.0).unwrap().value(), 0.0);
        assert_eq!(PercentileP::try_from(100.0).unwrap().value(), 100.0);

        let above = RangePositionPct::new(130.0).unwrap();
        let below = RangePositionPct::new(-5.0).unwrap();
        assert_eq!(above.value().to_bits(), 130.0f32.to_bits());
        assert_eq!(below.value().to_bits(), (-5.0f32).to_bits());
    }

    #[test]
    fn a_pickett_fit_without_sourced_a_or_rw_exposes_only_their_product() {
        // SB-PLT-011 / SB-PLT-T17: these are arithmetic fixtures, not shipped
        // petrophysical defaults. The fit identifies m and a·Rw, not a and Rw separately.
        let fit = PickettFit { m: 2.0, a_rw: 0.04 };
        let product_only = disclose_pickett_fit(fit, None, None).unwrap();
        assert_eq!(product_only.m.to_bits(), 2.0f32.to_bits());
        assert_eq!(product_only.a_rw.to_bits(), 0.04f32.to_bits());
        assert!(product_only.a.is_none());
        assert!(product_only.rw.is_none());

        let sourced_a = SourcedPickettValue {
            value: 2.0,
            provenance: "explicit test fixture for supplied a".into(),
        };
        let separated = disclose_pickett_fit(fit, Some(sourced_a), None).unwrap();
        assert_eq!(separated.a.unwrap().value.to_bits(), 2.0f32.to_bits());
        assert_eq!(separated.rw.unwrap().value.to_bits(), 0.02f32.to_bits());

        let unsourced = SourcedPickettValue { value: 2.0, provenance: "".into() };
        assert!(disclose_pickett_fit(fit, Some(unsourced), None)
            .unwrap_err()
            .contains("provenance"));
    }

    #[test]
    fn missing_log_xy_z_and_waveform_values_follow_their_own_reported_policies() {
        // SB-PLT-013: 0 and 10 are display-range arithmetic fixtures, not scientific limits.
        let source = [-1.0, 5.0, 20.0, f32::NAN];
        let source_bits: Vec<u32> = source.iter().map(|value| value.to_bits()).collect();

        let xy = apply_plot_channel_policy(
            &source,
            PlotChannelPolicy::Cartesian {
                log_axis: true,
                display: DisplayRange { low: 0.0, high: 10.0 },
            },
        );
        assert_eq!(xy.non_finite_excluded, 1);
        assert_eq!(xy.log_domain_excluded, 1);
        assert_eq!(xy.display_clipped, 1);
        assert_eq!(xy.clamped, 0);
        assert_eq!(xy.values[2].to_bits(), 20.0f32.to_bits());

        let z = apply_plot_channel_policy(
            &source,
            PlotChannelPolicy::Colour {
                log_axis: false,
                display: DisplayRange { low: 0.0, high: 10.0 },
            },
        );
        assert_eq!(z.non_finite_excluded, 1);
        assert_eq!(z.clamped, 2);
        assert_eq!(z.values[0].to_bits(), 0.0f32.to_bits());
        assert_eq!(z.values[2].to_bits(), 10.0f32.to_bits());
        assert_eq!(z.edge_marks, vec![RangeEdge::Low, RangeEdge::None, RangeEdge::High, RangeEdge::None]);

        let waveform = apply_plot_channel_policy(
            &source,
            PlotChannelPolicy::ArrayWaveform {
                log_axis: false,
                display: DisplayRange { low: 0.0, high: 10.0 },
            },
        );
        assert_eq!(waveform.non_finite_excluded, 1);
        assert_eq!(waveform.clamped, 2);
        assert_eq!(waveform.values[0].to_bits(), 0.0f32.to_bits());
        assert_eq!(waveform.values[2].to_bits(), 10.0f32.to_bits());

        let log_waveform = apply_plot_channel_policy(
            &source,
            PlotChannelPolicy::ArrayWaveform {
                log_axis: true,
                display: DisplayRange { low: 1.0, high: 10.0 },
            },
        );
        assert_eq!(log_waveform.non_finite_excluded, 1);
        assert_eq!(log_waveform.log_domain_excluded, 1);
        assert_eq!(log_waveform.clamped, 1);
        assert_eq!(
            log_waveform.edge_marks,
            vec![RangeEdge::None, RangeEdge::None, RangeEdge::High, RangeEdge::None]
        );

        assert_eq!(source.iter().map(|value| value.to_bits()).collect::<Vec<_>>(), source_bits);
    }

    #[test]
    fn an_all_nan_required_channel_consumes_no_quota_while_represented_wells_keep_both_endpoints() {
        // SB-PLT-014 / SB-PLT-T20: values are alignment fixtures; no domain limit is implied.
        let allocation = allocate_finite_pair_budget(
            &[
                WellRequiredChannels {
                    well_id: "missing".into(),
                    channels: vec![vec![1.0, 2.0, 3.0], vec![f32::NAN; 3]],
                },
                WellRequiredChannels {
                    well_id: "represented".into(),
                    channels: vec![vec![10.0, 20.0, 30.0], vec![1.0, 2.0, 3.0]],
                },
            ],
            2,
        )
        .unwrap();

        assert_eq!(allocation.wells.len(), 1);
        assert_eq!(allocation.wells[0].well_id, "represented");
        assert_eq!(allocation.wells[0].quota, 2);
        assert_eq!(allocation.wells[0].source_indices, vec![0, 2]);
        assert_eq!(allocation.absent.len(), 1);
        assert_eq!(allocation.absent[0].well_id, "missing");
        assert!(allocation.absent[0].reason.contains("zero finite aligned pairs"));
        assert_eq!(allocation.absent[0].quota, 0);
    }

    #[test]
    fn decimation_uses_one_shared_index_vector_and_reports_the_forced_final_endpoint() {
        // SB-PLT-015 / SB-PLT-T21/T22: 0..10 and stride 4 are the cited acceptance fixture.
        let eligible: Vec<usize> = (0..=10).collect();
        let channels = vec![
            (0..=10).map(|value| value as f32).collect::<Vec<_>>(),
            (0..=10).map(|value| 100.0 + value as f32).collect::<Vec<_>>(),
            (0..=10).map(|value| 200.0 + value as f32).collect::<Vec<_>>(),
            (0..=10).map(|value| 300.0 + value as f32).collect::<Vec<_>>(),
        ];
        let reduced = decimate_shared_channels(&channels, &eligible, 4).unwrap();

        assert_eq!(reduced.manifest.original_count, 11);
        assert_eq!(reduced.manifest.displayed_count, 4);
        assert_eq!(reduced.manifest.algorithm, "stride_from_first_with_forced_final_endpoint");
        assert_eq!(reduced.manifest.stride, 4);
        assert!(reduced.manifest.endpoints_forced);
        assert_eq!(reduced.manifest.source_indices, vec![0, 4, 8, 10]);
        for (channel_index, channel) in reduced.channels.iter().enumerate() {
            let base = 100.0 * channel_index as f32;
            assert_eq!(channel, &vec![base, base + 4.0, base + 8.0, base + 10.0]);
        }
    }

    #[test]
    fn equal_and_integer_multiple_depth_steps_proceed_but_non_integer_steps_route_to_dio_and_intervals_stay_half_open() {
        // SB-PLT-016 / SB-PLT-T23–T26: all values are the chapter's shown fixtures.
        let equal = reconcile_depth_steps(&[0.5, 0.5]).unwrap();
        assert_eq!(equal.coarsest_step.to_bits(), 0.5f32.to_bits());
        assert_eq!(equal.decimation_factors, vec![1, 1]);

        let multiple = reconcile_depth_steps(&[0.5, 1.0]).unwrap();
        assert_eq!(multiple.coarsest_step.to_bits(), 1.0f32.to_bits());
        assert_eq!(multiple.decimation_factors, vec![2, 1]);

        let refusal = reconcile_depth_steps(&[0.5, 0.8]).unwrap_err();
        assert!(refusal.contains("DIO resampling"));

        assert_eq!(half_open_depth_indices(&[100.0, 100.5, 101.0], 100.0, 101.0), vec![0, 1]);
    }

    #[test]
    fn a_plot_derived_parameter_write_is_undoable_and_requires_complete_non_null_provenance() {
        // SB-PLT-020 / SB-PLT-T30/T31: numeric values are metadata fixtures, not defaults.
        let axis = PlotWriteAxisBinding {
            channel: "x".into(),
            curve_id: "curve-x".into(),
            mnemonic: "X".into(),
            quantity: "fraction".into(),
            source_unit: "v/v".into(),
            display_unit: "v/v".into(),
            conversion: "identity".into(),
            source_revision: "a".repeat(64),
        };
        let input = PlotWriteProvenanceInput {
            plot_id: "plot-1".into(),
            plot_type: "crossplot".into(),
            x_axis: axis.clone(),
            y_axis: PlotWriteAxisBinding { channel: "y".into(), ..axis },
            z_axis: None,
            viewport: PlotWriteViewport {
                x_min: 0.0,
                x_max: 1.0,
                y_min: 0.0,
                y_max: 1.0,
                x_log: false,
                y_log: false,
            },
            selection: PlotWriteSelection {
                kind: "none".into(),
                selection_id: None,
                member_count: 0,
                revision: None,
            },
            interval: PlotWriteInterval {
                low: Some(100.0),
                high: Some(101.0),
                closure: "[lo,hi)".into(),
            },
            method: "manual_handle_drag".into(),
            fit_record: None,
            target: PlotWriteTarget {
                well_id: "well-1".into(),
                zone_name: "*".into(),
                parameter_name: "PARAM".into(),
                value: 0.25,
            },
        };

        let complete = finalize_plot_write_provenance(Some(input.clone()), "test-user", 1).unwrap();
        assert_eq!(complete.source.plot_id, "plot-1");
        assert_eq!(complete.user, "test-user");
        assert_eq!(complete.timestamp_utc_ms, 1);

        assert!(finalize_plot_write_provenance(None, "test-user", 1)
            .unwrap_err()
            .contains("null source note"));
        let mut missing_revision = input.clone();
        missing_revision.x_axis.source_revision.clear();
        assert!(finalize_plot_write_provenance(Some(missing_revision), "test-user", 1)
            .unwrap_err()
            .contains("source revision"));

        let mut missing_plot = input.clone();
        missing_plot.plot_id.clear();
        assert!(finalize_plot_write_provenance(Some(missing_plot), "test-user", 1).is_err());
        let mut missing_type = input.clone();
        missing_type.plot_type.clear();
        assert!(finalize_plot_write_provenance(Some(missing_type), "test-user", 1).is_err());
        let mut bad_viewport = input.clone();
        bad_viewport.viewport.x_min = f32::NAN;
        assert!(finalize_plot_write_provenance(Some(bad_viewport), "test-user", 1).is_err());
        let mut missing_selection = input.clone();
        missing_selection.selection.kind.clear();
        assert!(finalize_plot_write_provenance(Some(missing_selection), "test-user", 1).is_err());
        let mut missing_interval = input.clone();
        missing_interval.interval.closure = "closed".into();
        assert!(finalize_plot_write_provenance(Some(missing_interval), "test-user", 1).is_err());
        let mut missing_method = input.clone();
        missing_method.method.clear();
        assert!(finalize_plot_write_provenance(Some(missing_method), "test-user", 1).is_err());
        let mut missing_fit_record = input.clone();
        missing_fit_record.method = "two_point_fit".into();
        assert!(finalize_plot_write_provenance(Some(missing_fit_record), "test-user", 1).is_err());
        let mut missing_target = input.clone();
        missing_target.target.parameter_name.clear();
        assert!(finalize_plot_write_provenance(Some(missing_target), "test-user", 1).is_err());
        assert!(finalize_plot_write_provenance(Some(input.clone()), "", 1).is_err());
        assert!(finalize_plot_write_provenance(Some(input), "test-user", 0).is_err());

        // The UI adapter must validate before writing and retain the exact inverse/redo
        // operations. This pins the undoable half of SB-PLT-020 at the integration seam.
        let adapter = include_str!("../../src/ui/plotCommon.ts");
        let validation = adapter
            .find("finalizePlotWriteProvenance(completeSource)")
            .expect("plot write must finalize complete provenance");
        let write = adapter.find("await applyNew();").expect("plot write must be applied");
        assert!(validation < write, "provenance validation must precede the write");
        assert!(adapter.contains("undo: applyOld"));
        assert!(adapter.contains("redo: applyNew"));
    }

    #[test]
    fn the_backend_refuses_an_incomplete_chart_record_and_embeds_a_complete_one_in_vector_deliverables() {
        // Supporting backend proof for SB-PLT-023 / SB-PLT-T35. The cross-surface
        // acceptance test lives in tools/frontend-acceptance.test.mjs.
        let complete = ChartRenderRecord {
            chart_id: "chart-id".into(),
            title: "Chart title".into(),
            chart_type: "crossplot_overlay".into(),
            x_quantity: "fraction".into(),
            x_unit: "v/v".into(),
            y_quantity: "bulk_density".into(),
            y_unit: "g/cc".into(),
            citation: "Public primary source citation".into(),
            publisher: "Publisher".into(),
            revision_date: "revision fixture".into(),
            digitizer: Some("Digitizer".into()),
            approved_derivation_path: "independently_digitized_public_primary_source".into(),
            payload_checksum: "b".repeat(64),
            transform_applied: "orientation=normal;x=identity;y=identity".into(),
        };
        assert!(validate_chart_render_record(Some(&complete)).is_ok());

        let json = serde_json::to_string(&complete).unwrap();
        let svg = crate::composite::embed_chart_render_record_json_in_svg(
            "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>",
            &json,
        )
        .unwrap();
        assert!(svg.contains("sandibumi-chart-render-record-v1"));
        assert!(svg.contains("chart-id"));

        let pdf = crate::composite::assemble_single_page_pdf("", 100.0, 100.0);
        let pdf = crate::composite::embed_chart_render_record_json_in_pdf(pdf, &json).unwrap();
        use base64::Engine as _;
        let marker = format!(
            "SANDIBUMI_CHART_RENDER_RECORD_V1_BASE64:{}",
            base64::engine::general_purpose::STANDARD.encode(json.as_bytes())
        );
        assert!(String::from_utf8_lossy(&pdf).contains(&marker));

        let mut missing_revision = complete.clone();
        missing_revision.revision_date.clear();
        assert!(validate_chart_render_record(Some(&missing_revision))
            .unwrap_err()
            .contains("revision/date"));
        assert!(validate_chart_render_record(None).unwrap_err().contains("provenance"));

        let renderer = include_str!("../../src/ui/crossplotPanel.ts");
        let gate = renderer
            .find("const decision = authorizeProvenancedChart(")
            .expect("chart renderer must authorize provenance");
        let draw = renderer
            .find("drawChartOverlay(plot, overlayDef")
            .expect("chart renderer call must remain inventoried");
        assert!(gate < draw, "provenance authorization must precede chart rendering");
        assert!(renderer.contains("chartProvenance: chartProvenance ? JSON.stringify(chartProvenance)"));
    }

    #[test]
    fn the_backend_accepts_only_a_page_that_contains_every_recorded_mark_and_never_calls_raster_pixels_points() {
        // CORRECTNESS — supporting SB-PLT-026 / T37-T38 write-boundary proof. The
        // expected inclusion and medium-specific unit follow chapter 23 §§4.5/6;
        // coordinates are non-scientific discriminator geometry from the frontend fixture.
        let complete = PaperExportRecord {
            schema_version: 1,
            medium: "svg-vector".into(),
            unit: "pt".into(),
            source_width: 100.0,
            source_height: 80.0,
            margin_pt: 30.0,
            content_bounds: PaperExportBounds {
                min_x: -50.0,
                min_y: 0.0,
                max_x: 130.0,
                max_y: 120.0,
            },
            page_bounds: PaperExportBounds {
                min_x: -80.0,
                min_y: -30.0,
                max_x: 160.0,
                max_y: 150.0,
            },
            provenance_footer: "SandiBumi provenance: full records embedded".into(),
            crop_proof: "all_recorded_bounds_inside_page".into(),
        };
        validate_paper_export_record(&complete).unwrap();

        let json = serde_json::to_string(&complete).unwrap();
        let svg = crate::composite::embed_paper_export_record_json_in_svg(
            "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>",
            &json,
        )
        .unwrap();
        assert!(svg.contains("sandibumi-paper-export-validated-v1"));
        assert!(svg.contains("all_recorded_bounds_inside_page"));

        let pdf = crate::composite::assemble_single_page_pdf("", 240.0, 180.0);
        let pdf = crate::composite::embed_paper_export_record_json_in_pdf(pdf, &json).unwrap();
        assert!(String::from_utf8_lossy(&pdf).contains("SANDIBUMI_PAPER_EXPORT_V1_BASE64"));

        let mut cropped = complete.clone();
        cropped.page_bounds.min_x = 0.0;
        assert!(validate_paper_export_record(&cropped)
            .unwrap_err()
            .contains("cropped"));

        let mut source_cropping_lie = complete.clone();
        source_cropping_lie.content_bounds.max_x = 99.0;
        assert!(validate_paper_export_record(&source_cropping_lie)
            .unwrap_err()
            .contains("source canvas is cropped"));

        let mut raster_lie = complete;
        raster_lie.medium = "print-raster".into();
        raster_lie.crop_proof = "raster_pixels_preserved_before_browser_print_layout".into();
        assert!(validate_paper_export_record(&raster_lie)
            .unwrap_err()
            .contains("medium-specific unit"));
    }

    #[test]
    fn an_export_after_budget_reduction_includes_original_and_displayed_counts_and_the_algorithm_while_a_hard_maximum_refuses() {
        // SB-PLT-031 / SB-PLT-T40: 0..10 at stride 4 is the cited SB-PLT-T21
        // reduction fixture. The too-small budget is an arithmetic refusal fixture.
        let channel = (0..=10).map(|value| value as f32).collect::<Vec<_>>();
        let eligible = (0..=10).collect::<Vec<_>>();
        let reduced = decimate_shared_channels(&[channel], &eligible, 4).unwrap();
        let export = PlotReductionExport {
            schema_version: 1,
            plot_type: "crossplot".into(),
            items: vec![ReductionExportItem {
                subject_kind: "points".into(),
                subject_id: "represented-well".into(),
                original_count: reduced.manifest.original_count,
                displayed_count: reduced.manifest.displayed_count,
                algorithm: reduced.manifest.algorithm,
                stride: Some(reduced.manifest.stride),
                endpoints_forced: Some(reduced.manifest.endpoints_forced),
            }],
            absent: Vec::new(),
            refusal: None,
        };
        let json = serialize_reduction_export(&export).unwrap();
        assert!(json.contains("\"original_count\": 11"));
        assert!(json.contains("\"displayed_count\": 4"));
        assert!(json.contains("\"algorithm\": \"stride_from_first_with_forced_final_endpoint\""));
        assert!(json.contains("\"stride\": 4"));
        assert!(json.contains("\"endpoints_forced\": true"));

        let mut missing_algorithm = export.clone();
        missing_algorithm.items[0].algorithm.clear();
        assert!(serialize_reduction_export(&missing_algorithm)
            .unwrap_err()
            .contains("algorithm"));
        let mut impossible_counts = export;
        impossible_counts.items[0].displayed_count = 12;
        assert!(serialize_reduction_export(&impossible_counts)
            .unwrap_err()
            .contains("more records"));

        let refusal = allocate_finite_pair_budget(
            &[WellRequiredChannels {
                well_id: "represented-well".into(),
                channels: vec![vec![0.0, 1.0], vec![2.0, 3.0]],
            }],
            1,
        )
        .unwrap_err();
        assert!(refusal.contains("cannot retain both endpoints"));

        let export_ui = include_str!("../../src/ui/plotExport.ts");
        assert!(export_ui.contains("savePlotReductionManifest(dest, JSON.stringify(manifest))"));
        assert!(export_ui.contains("Export reduction counts, algorithm, stride and endpoint handling"));
        let command_adapter = include_str!("lib.rs");
        assert!(command_adapter.contains("fn save_plot_reduction_manifest"));
        assert!(command_adapter.contains("save_plot_reduction_manifest,"));
        let common_ui = include_str!("../../src/ui/plotCommon.ts");
        assert!(common_ui.contains("original_count: layer.reduction.originalCount"));
        assert!(common_ui.contains("displayed_count: layer.reduction.displayedCount"));
        assert!(common_ui.contains("algorithm: layer.reduction.algorithm"));
        assert!(common_ui.contains("stride: layer.reduction.stride"));
        assert!(common_ui.contains("endpoints_forced: layer.reduction.endpointsForced"));
        let histogram_ui = include_str!("../../src/ui/histogramPanel.ts");
        assert!(!histogram_ui.contains(".sort((a, b) => a - b).slice(0, 8)"));
        let limits_ui = include_str!("../../src/ui/plotLimits.ts");
        assert!(limits_ui.contains("id: \"context_point_budget\""));
        assert!(limits_ui.contains("maximum: 60_000"));
        assert!(limits_ui.contains("policy: \"refuse_above_hard_maximum\""));
        let vega_ui = include_str!("../../src/ui/vegaPanel.ts");
        assert!(vega_ui.contains("applyPlotRecordLimit(\"vega_categorical_groups\""));
        assert!(vega_ui.contains("() => reductionManifest"));
        assert!(vega_ui.contains("pick a categorical curve"));
    }
}
