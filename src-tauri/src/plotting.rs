use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use duckdb::{params, Connection};

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlotChannelBinding {
    pub intent: PlotChannelIntent,
    pub resolved: Vec<ResolvedPlotCurve>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DisplayRange {
    pub low: f32,
    pub high: f32,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AxisRangeResolution {
    pub range: DisplayRange,
    pub tier: AxisRangeTier,
}

fn usable_range(range: DisplayRange) -> bool {
    range.low.is_finite() && range.high.is_finite() && range.low != range.high
}

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
        factor: rule.factor,
        offset: rule.offset,
        transform: format!(
            "(source + {}) * {}; {}",
            rule.offset, rule.factor, rule.derivation
        ),
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RangePolicyReport {
    pub input_count: usize,
    pub non_finite_excluded: usize,
    pub validity_excluded: usize,
    pub display_hidden: usize,
    pub statistics_count: usize,
    pub kept_values: Vec<f32>,
}

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitLimitPair {
    pub quantity: String,
    pub source_unit: String,
    pub converted_unit: String,
    pub source_value: f32,
    pub converted_value: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitLimitAudit {
    pub divergence_factor: f32,
    pub enabled: bool,
    pub reason: String,
}

/// Audits one converted-unit row. Activation is exact: the pair needs a typed,
/// registered conversion and the supplied converted value must be the reviewed
/// affine result byte-for-byte. No unpublished tolerance is introduced here.
pub fn audit_unit_limit_pair(pair: UnitLimitPair) -> UnitLimitAudit {
    let divergence_factor = if pair.source_value.is_finite()
        && pair.converted_value.is_finite()
        && pair.source_value != 0.0
        && pair.converted_value != 0.0
    {
        let ratio = (pair.converted_value / pair.source_value).abs();
        ratio.max(1.0 / ratio)
    } else {
        f32::INFINITY
    };
    let bridge = crate::curves::validate_unit_bridge(&pair.source_unit, &pair.converted_unit);
    let Ok(bridge) = bridge else {
        return UnitLimitAudit {
            divergence_factor,
            enabled: false,
            reason: "disabled: no registered dimensional conversion proves this unit-limit pair".into(),
        };
    };
    let rule = crate::curves::UNIT_RULES.iter().find(|rule| {
        crate::curves::validate_unit_bridge(rule.from_unit, rule.to_unit)
            .map(|candidate| candidate.from_unit == bridge.from_unit && candidate.to_unit == bridge.to_unit)
            .unwrap_or(false)
    });
    let Some(rule) = rule else {
        return UnitLimitAudit {
            divergence_factor,
            enabled: false,
            reason: "disabled: compatible dimensions have no registered numeric conversion".into(),
        };
    };
    let expected = (pair.source_value + rule.offset) * rule.factor;
    if expected.to_bits() != pair.converted_value.to_bits() {
        return UnitLimitAudit {
            divergence_factor,
            enabled: false,
            reason: format!(
                "disabled: converted value does not equal the registered transform ({})",
                rule.derivation
            ),
        };
    }
    UnitLimitAudit {
        divergence_factor,
        enabled: true,
        reason: format!("enabled after exact registered conversion audit: {}", rule.derivation),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistogramContract {
    pub counts: Vec<u32>,
    pub displayed_total: u32,
    pub non_finite_excluded: usize,
}

pub fn canonical_histogram(
    values: &[f32],
    minimum: f32,
    maximum: f32,
    bins: usize,
) -> HistogramContract {
    let counts = crate::distribution::histogram(values, minimum, maximum, bins.max(1));
    HistogramContract {
        displayed_total: counts.iter().sum(),
        non_finite_excluded: values.iter().filter(|value| !value.is_finite()).count(),
        counts,
    }
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
    ArrayWaveform { display: DisplayRange },
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
        edge_marks: vec![RangeEdge::None; source.len()],
        non_finite_excluded: 0,
        log_domain_excluded: 0,
        display_clipped: 0,
        clamped: 0,
    };
    let (display, log_axis, clamp) = match policy {
        PlotChannelPolicy::Cartesian { log_axis, display } => (display, log_axis, false),
        PlotChannelPolicy::Colour { log_axis, display } => (display, log_axis, true),
        PlotChannelPolicy::ArrayWaveform { display } => (display, false, true),
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepthStepReconciliation {
    pub coarsest_step: f32,
    pub decimation_factors: Vec<usize>,
}

/// Chooses only among exact relationships. No tolerance or resampling kernel is
/// introduced: equality keeps factor 1, exact integer multiples decimate toward
/// the coarsest step, and every other relationship is routed to Data I/O.
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
    match request {
        "DEPTH" => Some(("depth", "m")),
        "GR" => Some(("gr", "gAPI")),
        "RES_DEEP" => Some(("res_deep", "ohm.m")),
        "NPHI" => Some(("nphi", "v/v")),
        "RHOB" => Some(("rhob", "g/cc")),
        "DT" => Some(("dt", "us/ft")),
        "SP" => Some(("sp", "mV")),
        _ => None,
    }
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
            return Ok(Some(ResolvedPlotCurve {
                well_id: well_id.into(),
                curve_id: format!("standard:{well_id}:{request}"),
                mnemonic: request,
                quantity,
                source_unit: unit.into(),
                display_unit: unit.into(),
                conversion: "identity".into(),
                sample_count: count as usize,
                resolution_reason: "finite standard curve wins the plot resolution order".into(),
                source_revision,
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
        return Ok(Some(ResolvedPlotCurve {
            well_id: well_id.into(),
            curve_id: format!("computed:{well_id}:{request}"),
            mnemonic: request,
            quantity,
            source_unit: unit.clone(),
            display_unit: unit,
            conversion: "identity".into(),
            sample_count: computed_count as usize,
            resolution_reason: "exact computed mnemonic after no finite standard curve".into(),
            source_revision,
        }));
    }

    let generic = conn
        .query_row(
            "SELECT curve_id, mnemonic, unit, family,
                    (SELECT COUNT(*) FROM curve_samples s
                     WHERE s.curve_id = m.curve_id AND s.value IS NOT NULL AND isfinite(s.value)),
                    set_name, COALESCE(pinned, 0), run_no
             FROM curve_meta m
             WHERE well_id = ?1 AND (upper(mnemonic) = ?2 OR upper(family) = ?2)
             ORDER BY (set_name = 'RAW') DESC,
                      (upper(mnemonic) = ?2) DESC,
                      (CASE WHEN upper(mnemonic) = ?2 THEN COALESCE(pinned, 0) ELSE 0 END) DESC,
                      set_name, run_no NULLS FIRST, curve_id
             LIMIT 1",
            params![well_id, request],
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
        .ok();
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
    fn a_dimensionally_divergent_unit_limit_row_stays_disabled_with_its_reason() {
        // The 6.56× expected divergence is cited by critique A-4 and §3.3a / SB-PLT-T05.
        // 1.0 is only a dimensionless arithmetic normalization, not a shipped limit.
        let audited = audit_unit_limit_pair(UnitLimitPair {
            quantity: "acoustic_attenuation".into(),
            source_unit: "unregistered_source_unit".into(),
            converted_unit: "unregistered_converted_unit".into(),
            source_value: 1.0,
            converted_value: 6.56,
        });
        assert!((audited.divergence_factor - 6.56).abs() < f32::EPSILON);
        assert!(!audited.enabled);
        assert!(audited.reason.contains("registered dimensional conversion"));
    }

    #[test]
    fn histogram_bins_are_half_open_except_for_the_final_upper_endpoint_and_non_finite_values_are_counted() {
        let endpoints = canonical_histogram(&[0.0, 1.0, 2.0, 3.0], 0.0, 3.0, 3);
        assert_eq!(endpoints.counts, vec![1, 1, 2]);
        assert_eq!(endpoints.displayed_total, 4);
        assert_eq!(endpoints.non_finite_excluded, 0);

        let missing = canonical_histogram(&[0.0, f32::NAN, f32::INFINITY, 1.0], 0.0, 1.0, 3);
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
            PlotChannelPolicy::ArrayWaveform { display: DisplayRange { low: 0.0, high: 10.0 } },
        );
        assert_eq!(waveform.non_finite_excluded, 1);
        assert_eq!(waveform.clamped, 2);
        assert_eq!(waveform.values[0].to_bits(), 0.0f32.to_bits());
        assert_eq!(waveform.values[2].to_bits(), 10.0f32.to_bits());

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
    fn a_chart_record_missing_its_source_revision_cannot_render_in_a_deliverable() {
        // SB-PLT-023 / SB-PLT-T35: strings are provenance fixtures, not shipped content.
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

        let mut missing_revision = complete;
        missing_revision.revision_date.clear();
        assert!(validate_chart_render_record(Some(&missing_revision))
            .unwrap_err()
            .contains("revision/date"));
        assert!(validate_chart_render_record(None).unwrap_err().contains("provenance"));

        let renderer = include_str!("../../src/ui/crossplotPanel.ts");
        let gate = renderer
            .find("authorizeProvenancedChart(overlayDef")
            .expect("chart renderer must authorize provenance");
        let draw = renderer
            .find("drawChartOverlay(plot, overlayDef")
            .expect("chart renderer call must remain inventoried");
        assert!(gate < draw, "provenance authorization must precede chart rendering");
        assert!(renderer.contains("chartProvenance: chartProvenance ? JSON.stringify(chartProvenance)"));
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
            }],
            absent: Vec::new(),
            refusal: None,
        };
        let json = serialize_reduction_export(&export).unwrap();
        assert!(json.contains("\"original_count\": 11"));
        assert!(json.contains("\"displayed_count\": 4"));
        assert!(json.contains("\"algorithm\": \"stride_from_first_with_forced_final_endpoint\""));

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
        assert!(export_ui.contains("Export original/displayed counts and reduction algorithms"));
        let command_adapter = include_str!("lib.rs");
        assert!(command_adapter.contains("fn save_plot_reduction_manifest"));
        assert!(command_adapter.contains("save_plot_reduction_manifest,"));
        let common_ui = include_str!("../../src/ui/plotCommon.ts");
        assert!(common_ui.contains("original_count: layer.reduction.originalCount"));
        assert!(common_ui.contains("displayed_count: layer.reduction.displayedCount"));
        assert!(common_ui.contains("algorithm: layer.reduction.algorithm"));
        let histogram_ui = include_str!("../../src/ui/histogramPanel.ts");
        assert!(!histogram_ui.contains(".sort((a, b) => a - b).slice(0, 8)"));
        let vega_ui = include_str!("../../src/ui/vegaPanel.ts");
        assert!(vega_ui.contains("if (order.length > MAX_GROUPS)"));
        assert!(vega_ui.contains("pick a categorical curve"));
    }
}
