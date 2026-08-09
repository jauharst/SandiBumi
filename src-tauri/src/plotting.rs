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
}
