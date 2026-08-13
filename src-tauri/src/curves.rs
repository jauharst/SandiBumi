//! Phase 6: mnemonic dictionary (alias → petrophysical family) and unit canonicalization
//! for the generic curve store. When a LAS/DLIS file arrives with an arbitrary curve, this
//! decides which family bucket it belongs to (so the catalog can group and the modules can
//! find it) and rescales its samples into SandiBumi's canonical unit for that family.
//!
//! Runtime tables are generated from `registry/unit-registry.json`; the release gate refuses
//! drift between that reviewed source and its Rust, TypeScript, documentation and test consumers.

/// Canonical family + its canonical unit. Families are deliberately coarse — enough to
/// group the catalog and let modules ask "give me the GR" without caring whether the file
/// called it GR, GRN, or CGR.
pub struct FamilySpec {
    pub family: &'static str,
    pub canonical_unit: &'static str,
    pub quantity_kind: QuantityKind,
    /// Uppercased mnemonic aliases that map to this family.
    pub aliases: &'static [&'static str],
}

/// One independently checkable affine conversion rule. `derivation` is mandatory data,
/// not a nearby comment: a factor cannot enter the table without carrying the arithmetic
/// a reviewer needs to reproduce it. Values use `(source + offset) × factor`.
pub struct UnitRule {
    pub families: &'static [&'static str],
    pub from_unit: &'static str,
    pub to_unit: &'static str,
    pub factor: f32,
    pub offset: f32,
    pub derivation: &'static str,
    /// False where the arithmetic is known but the incoming label is not trustworthy
    /// enough to apply without a per-file user confirmation.
    pub automatic: bool,
}

/// One recognised spelling and its typed canonical interpretation. This table carries no
/// conversion factors; arithmetic remains exclusively in the independently derived UNIT_RULES.
pub struct UnitTokenSpec {
    pub token: &'static str,
    pub quantity_kind: QuantityKind,
    pub canonical_unit: &'static str,
}

include!("generated/unit_registry.rs");

pub fn resolve_unit_token(token: &str) -> Option<&'static UnitTokenSpec> {
    let observed = token.trim();
    UNIT_TOKENS.iter().find(|entry| entry.token == observed)
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UnitTokenState {
    MissingUnit,
    Recognized,
    Unrecognized,
}

pub fn unit_token_state(token: Option<&str>) -> UnitTokenState {
    let observed = token.map(str::trim);
    if matches!(observed, None | Some("" | "-" | "?")) {
        UnitTokenState::MissingUnit
    } else if resolve_unit_token(observed.unwrap()).is_some() {
        UnitTokenState::Recognized
    } else {
        UnitTokenState::Unrecognized
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct UnitTokenObservation {
    pub curve: String,
    pub state: UnitTokenState,
    pub raw_token: Option<String>,
    pub canonical_unit: Option<String>,
    pub quantity_kind: Option<QuantityKind>,
    /// Present only when a reviewed registry row explicitly maps this spelling to another
    /// canonical spelling. Similar typography alone never creates an alias.
    pub explicit_alias: Option<String>,
}

/// Preserve each observed spelling before interpretation and report look-alike vocabulary that
/// has no explicit equivalence row. The comparison is warning-only; it never selects a unit.
pub fn observe_unit_tokens(
    tokens: &[(String, Option<String>)],
) -> (Vec<UnitTokenObservation>, Vec<String>) {
    let observations = tokens
        .iter()
        .map(|(curve, token)| {
            let raw = token.as_deref().map(str::trim);
            let state = unit_token_state(raw);
            let resolved = (state == UnitTokenState::Recognized)
                .then(|| resolve_unit_token(raw.unwrap()))
                .flatten();
            UnitTokenObservation {
                curve: curve.clone(),
                state,
                raw_token: raw.filter(|token| !token.is_empty()).map(str::to_string),
                canonical_unit: resolved.map(|entry| entry.canonical_unit.to_string()),
                quantity_kind: resolved.map(|entry| entry.quantity_kind),
                explicit_alias: resolved
                    .filter(|entry| entry.token != entry.canonical_unit)
                    .map(|entry| format!("{} -> {}", entry.token, entry.canonical_unit)),
            }
        })
        .collect::<Vec<_>>();

    let mut warnings = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (index, left) in observations.iter().enumerate() {
        for right in observations.iter().skip(index + 1) {
            let (Some(left_token), Some(right_token)) =
                (left.raw_token.as_deref(), right.raw_token.as_deref())
            else {
                continue;
            };
            if left.state == UnitTokenState::MissingUnit
                || right.state == UnitTokenState::MissingUnit
                || left_token == right_token
                || !left_token.eq_ignore_ascii_case(right_token)
            {
                continue;
            }
            let explicitly_equivalent = match (
                resolve_unit_token(left_token),
                resolve_unit_token(right_token),
            ) {
                (Some(left), Some(right)) => {
                    left.quantity_kind == right.quantity_kind
                        && left.canonical_unit == right.canonical_unit
                }
                _ => false,
            };
            if explicitly_equivalent {
                continue;
            }
            let key = if left_token < right_token {
                (left_token.to_string(), right_token.to_string())
            } else {
                (right_token.to_string(), left_token.to_string())
            };
            if seen.insert(key) {
                warnings.push(format!(
                    "unit-token drift: observed '{}' and '{}' remain distinct because no explicit alias declares them equivalent",
                    left_token, right_token
                ));
            }
        }
    }
    (observations, warnings)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitMappingRowState {
    MissingUnit,
    Registered(ValidatedUnitBridge),
}

/// Load mapping rows without letting absent/empty/placeholder spellings create bridges. A valid
/// row still registers normally, which keeps `MissingUnit` from becoming a catch-all success.
pub fn load_unit_mapping_rows(
    rows: &[(Option<&str>, Option<&str>)],
) -> Result<Vec<UnitMappingRowState>, UnitRegistryError> {
    rows.iter()
        .map(|(from, to)| {
            if unit_token_state(*from) == UnitTokenState::MissingUnit
                || unit_token_state(*to) == UnitTokenState::MissingUnit
            {
                return Ok(UnitMappingRowState::MissingUnit);
            }
            validate_unit_bridge(from.unwrap(), to.unwrap()).map(UnitMappingRowState::Registered)
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedUnitBridge {
    pub from_unit: &'static str,
    pub to_unit: &'static str,
    pub quantity_kind: QuantityKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitRegistryError {
    UnknownUnit { token: String },
    InvalidRegistryIdentity,
    MissingUnitMapping {
        from_unit: Option<String>,
        to_unit: Option<String>,
    },
    QuantityKindMismatch {
        from_unit: String,
        from_kind: QuantityKind,
        to_unit: String,
        to_kind: QuantityKind,
    },
}

impl std::fmt::Display for UnitRegistryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownUnit { token } => write!(formatter, "unknown unit token {token}"),
            Self::InvalidRegistryIdentity => {
                write!(formatter, "unit registry has no valid generated version and SHA-256")
            }
            Self::MissingUnitMapping { from_unit, to_unit } => write!(
                formatter,
                "unit mapping is missing a unit: from={from_unit:?}, to={to_unit:?}"
            ),
            Self::QuantityKindMismatch {
                from_unit,
                from_kind,
                to_unit,
                to_kind,
            } => write!(
                formatter,
                "quantity-kind mismatch: {from_unit} is {from_kind:?}, but {to_unit} is {to_kind:?}"
            ),
        }
    }
}

pub fn validate_unit_bridge(
    from_unit: &str,
    to_unit: &str,
) -> Result<ValidatedUnitBridge, UnitRegistryError> {
    let from = resolve_unit_token(from_unit).ok_or_else(|| UnitRegistryError::UnknownUnit {
        token: from_unit.to_string(),
    })?;
    let to = resolve_unit_token(to_unit).ok_or_else(|| UnitRegistryError::UnknownUnit {
        token: to_unit.to_string(),
    })?;
    if from.quantity_kind != to.quantity_kind {
        return Err(UnitRegistryError::QuantityKindMismatch {
            from_unit: from.canonical_unit.to_string(),
            from_kind: from.quantity_kind,
            to_unit: to.canonical_unit.to_string(),
            to_kind: to.quantity_kind,
        });
    }
    Ok(ValidatedUnitBridge {
        from_unit: from.canonical_unit,
        to_unit: to.canonical_unit,
        quantity_kind: from.quantity_kind,
    })
}

pub fn validate_unit_registry() -> Result<(), UnitRegistryError> {
    if UNIT_REGISTRY_VERSION.is_empty()
        || UNIT_REGISTRY_SHA256.len() != 64
        || !UNIT_REGISTRY_SHA256.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(UnitRegistryError::InvalidRegistryIdentity);
    }
    for family in FAMILIES {
        let canonical = resolve_unit_token(family.canonical_unit).ok_or_else(|| UnitRegistryError::UnknownUnit {
            token: family.canonical_unit.to_string(),
        })?;
        if canonical.quantity_kind != family.quantity_kind {
            return Err(UnitRegistryError::QuantityKindMismatch {
                from_unit: family.family.to_string(),
                from_kind: family.quantity_kind,
                to_unit: canonical.canonical_unit.to_string(),
                to_kind: canonical.quantity_kind,
            });
        }
    }
    let mapping_rows = UNIT_RULES
        .iter()
        .map(|rule| (Some(rule.from_unit), Some(rule.to_unit)))
        .collect::<Vec<_>>();
    for ((from_unit, to_unit), state) in mapping_rows
        .iter()
        .zip(load_unit_mapping_rows(&mapping_rows)?)
    {
        if state == UnitMappingRowState::MissingUnit {
            return Err(UnitRegistryError::MissingUnitMapping {
                from_unit: from_unit.map(str::to_string),
                to_unit: to_unit.map(str::to_string),
            });
        }
    }
    Ok(())
}

pub fn convertible_unit_families() -> Vec<String> {
    CONVERTIBLE_FAMILIES.iter().map(|family| (*family).to_string()).collect()
}

/// The two legitimate meanings of `MS/FT` identified by chapter finding D-12.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MsPerFtMeaning {
    MicrosecondsPerFoot,
    MillisiemensPerFoot,
}

impl MsPerFtMeaning {
    pub fn label(self) -> &'static str {
        match self {
            Self::MicrosecondsPerFoot => "microseconds_per_foot",
            Self::MillisiemensPerFoot => "millisiemens_per_foot",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct UnitDesignation {
    pub curve: String,
    pub declared_unit: String,
    pub meaning: String,
    pub recorded_unit: String,
    pub family: Option<String>,
}

impl UnitDesignation {
    pub fn note(&self) -> String {
        format!(
            "designated {} unit {} as {}; recorded unit {}{}",
            self.curve,
            self.declared_unit,
            self.meaning,
            self.recorded_unit,
            self.family
                .as_deref()
                .map(|family| format!(" and family {family}"))
                .unwrap_or_default()
        )
    }
}

pub fn is_ms_per_ft(unit: Option<&str>) -> bool {
    matches!(unit.map(str::trim), Some("MS/FT" | "ms/ft" | "MSFT" | "msft"))
}

pub fn ms_per_ft_designation(
    curve: &str,
    declared_unit: &str,
    meaning: MsPerFtMeaning,
) -> UnitDesignation {
    let inferred = family_for(curve).filter(|family| matches!(family.family, "DT" | "DTS"));
    match meaning {
        MsPerFtMeaning::MicrosecondsPerFoot => UnitDesignation {
            curve: curve.to_string(),
            declared_unit: declared_unit.to_string(),
            meaning: meaning.label().to_string(),
            recorded_unit: "us/ft".to_string(),
            family: inferred.map(|family| family.family.to_string()),
        },
        MsPerFtMeaning::MillisiemensPerFoot => UnitDesignation {
            curve: curve.to_string(),
            declared_unit: declared_unit.to_string(),
            meaning: meaning.label().to_string(),
            recorded_unit: declared_unit.to_string(),
            family: None,
        },
    }
}

/// Returns the canonical family for a mnemonic, or `None` if it isn't recognized (the
/// curve is still imported — it just goes in the catalog family-less, and modules that
/// need a family won't auto-pick it).
pub fn family_for(mnemonic: &str) -> Option<&'static FamilySpec> {
    let m = mnemonic.trim().to_uppercase();
    FAMILIES.iter().find(|f| f.aliases.iter().any(|a| *a == m))
}

/// Canonical unit string SandiBumi stores a given family in.
pub fn canonical_unit(family: &str) -> Option<&'static str> {
    FAMILIES.iter().find(|f| f.family == family).map(|f| f.canonical_unit)
}

/// One unit conversion that was actually applied during import. This is returned to the
/// caller instead of a bare boolean so an automatic conversion can never be silent.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct UnitConversion {
    pub curve: String,
    pub from_unit: String,
    pub to_unit: String,
    pub factor: f32,
    /// Source-space offset: canonical = (source + offset) × factor.
    pub offset: f32,
    pub derivation: String,
}

impl UnitConversion {
    pub fn note(&self) -> String {
        format!(
            "converted {} from {} to {} with factor {} and offset {}",
            self.curve, self.from_unit, self.to_unit, self.factor, self.offset
        )
    }
}

/// A declared source unit for which no reviewed conversion was applied. Values and the
/// original unit label remain intact; the record prevents that pass-through from looking
/// canonical merely because the import itself succeeded.
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct UnconvertedUnit {
    pub curve: String,
    pub declared_unit: String,
    pub family: Option<String>,
    pub reason: String,
    /// True when values cannot be interpreted safely until the user identifies the quantity.
    pub designation_required: bool,
    /// The reviewed vendor-table entry that was rejected, when this is an alias rejection.
    pub rejected_entry: Option<String>,
}

impl UnconvertedUnit {
    pub fn note(&self) -> String {
        format!(
            "left {} in declared unit {} (unconverted: {})",
            self.curve, self.declared_unit, self.reason
        )
    }
}

/// Returns an audit record only when a non-empty declared unit is neither canonical nor
/// converted. An unknown mnemonic is included: without a known quantity family there is
/// no dimensional basis on which conversion could be safe.
pub fn unconverted_unit(
    curve: &str,
    family: Option<&str>,
    src_unit: Option<&str>,
) -> Option<UnconvertedUnit> {
    let declared = src_unit?.trim();
    if declared.is_empty() {
        return None;
    }
    if family
        .and_then(canonical_unit)
        .is_some_and(|canonical| {
            matches!(
                (resolve_unit_token(canonical), resolve_unit_token(declared)),
                (Some(expected), Some(observed))
                    if expected.quantity_kind == observed.quantity_kind
                        && expected.canonical_unit == observed.canonical_unit
            )
        })
    {
        return None;
    }
    let reason = match family {
        None => "the mnemonic has no known quantity family".to_string(),
        Some(name) if !CONVERTIBLE_FAMILIES.contains(&name) => {
            format!("family {name} has no declared numeric conversion coverage")
        }
        Some(name) => format!("unit {declared} has no reviewed conversion rule for family {name}"),
    };
    Some(UnconvertedUnit {
        curve: curve.to_string(),
        declared_unit: declared.to_string(),
        family: family.map(str::to_string),
        reason,
        designation_required: false,
        rejected_entry: None,
    })
}

/// Resolves an import mnemonic only after applying reviewed unit-alias rejections.
/// `PPG → density` is explicitly NON-ADOPTABLE in chapter §5.1 / finding D-14:
/// it is a pressure-gradient quantity, not a bulk-density unit.
pub fn family_for_import(
    curve: &str,
    src_unit: Option<&str>,
) -> (Option<&'static FamilySpec>, Option<UnconvertedUnit>) {
    let inferred = family_for(curve);
    let declared = src_unit.map(str::trim).unwrap_or_default();
    if matches!(declared, "MEQ/L" | "meq/L") {
        return (
            None,
            Some(UnconvertedUnit {
                curve: curve.to_string(),
                declared_unit: declared.to_string(),
                family: None,
                reason: "the corrected MEQ/L factor is 10^-3, but §7.1 O-2 records that affected files may already contain meq/mL values; per-file confirmation is required"
                    .to_string(),
                designation_required: true,
                rejected_entry: Some("elec_charge_per_vol.units: MEQ/L -> 1.0".to_string()),
            }),
        );
    }
    if declared != "PPG" {
        return (inferred, None);
    }
    (
        None,
        Some(UnconvertedUnit {
            curve: curve.to_string(),
            declared_unit: declared.to_string(),
            family: None,
            reason: "PPG is a pressure-gradient quantity, not a bulk-density unit; quantity designation is required"
                .to_string(),
            designation_required: true,
            rejected_entry: Some("density.units: PPG -> density".to_string()),
        }),
    )
}

/// Converts a value from a source unit into the canonical unit for a family, in place.
/// Only conversions that actually occur in field LAS files are handled; anything already
/// canonical, unrecognized, or dimensionally identical is left untouched (returns `None`
/// so the caller can keep the original unit label). NaN is preserved (missing stays missing).
pub fn convert_to_canonical(
    curve: &str,
    family: &str,
    src_unit: Option<&str>,
    values: &mut [f32],
) -> Option<UnitConversion> {
    let target = canonical_unit(family)?;
    let declared = src_unit?.trim();
    if declared.is_empty() {
        return None;
    }
    let typed_bridge = validate_unit_bridge(declared, target).ok()?;
    let target_token = resolve_unit_token(target)?;
    let declared_token = resolve_unit_token(declared)?;
    if declared_token.quantity_kind == target_token.quantity_kind
        && declared_token.canonical_unit == target_token.canonical_unit
    {
        return None;
    }

    let rule = UNIT_RULES.iter().find(|rule| {
        let from = resolve_unit_token(rule.from_unit);
        let to = resolve_unit_token(rule.to_unit);
        rule.automatic
            && rule.families.contains(&family)
            && from.is_some_and(|entry| {
                entry.quantity_kind == declared_token.quantity_kind
                    && entry.canonical_unit == declared_token.canonical_unit
            })
            && to.is_some_and(|entry| {
                entry.quantity_kind == target_token.quantity_kind
                    && entry.canonical_unit == target_token.canonical_unit
            })
            && validate_unit_bridge(rule.from_unit, rule.to_unit)
                .is_ok_and(|rule_bridge| rule_bridge.quantity_kind == typed_bridge.quantity_kind)
    })?;
    let (factor, offset) = (rule.factor, rule.offset);
    for v in values.iter_mut() {
        if v.is_finite() {
            *v = (*v + offset) * factor;
        }
    }
    Some(UnitConversion {
        curve: curve.to_string(),
        from_unit: src_unit.unwrap_or_default().trim().to_string(),
        to_unit: target.to_string(),
        factor,
        offset,
        derivation: rule.derivation.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn families_resolve_common_mnemonics() {
        assert_eq!(family_for("GR").unwrap().family, "GR");
        assert_eq!(family_for("grn").unwrap().family, "GR");
        assert_eq!(family_for("RHOZ").unwrap().family, "RHOB");
        assert_eq!(family_for("PEFZ").unwrap().family, "PEF");
        assert_eq!(family_for("HCAL").unwrap().family, "CALI");
        assert_eq!(family_for("AT90").unwrap().family, "RES_DEEP");
        assert_eq!(family_for("AT10").unwrap().family, "RES_SHAL");
        assert!(family_for("ZZ_UNKNOWN").is_none());
    }

    /// SB-DIO-025 / SB-DIO-T41. The query must expose exactly the families backed by
    /// the code-resident transforms above; a vocabulary entry without arithmetic is not
    /// conversion coverage (chapter finding D-9).
    #[test]
    fn the_unit_system_reports_the_exact_families_it_can_convert() {
        assert_eq!(
            convertible_unit_families(),
            ["CALI", "BS", "RHOB", "DRHO", "NPHI", "DT", "DTS", "TEMP"]
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        );
        assert_eq!(convertible_unit_families().len(), CONVERTIBLE_FAMILIES.len());
    }

    /// SB-DIO-028 / SB-DIO-T44. Chapter §5.1 independently derives the corrected
    /// MEQ/L factor from 1 L = 10^3 mL. Every other numeric rule must meet the same
    /// standard: arithmetic lives in the row, not only in a vendor citation or comment.
    #[test]
    fn every_conversion_factor_carries_an_independent_arithmetic_derivation() {
        assert!(!UNIT_RULES.is_empty());
        for rule in UNIT_RULES {
            assert!(rule.factor.is_finite() && rule.factor != 0.0, "invalid factor for {} -> {}", rule.from_unit, rule.to_unit);
            assert!(rule.offset.is_finite(), "invalid offset for {} -> {}", rule.from_unit, rule.to_unit);
            assert!(!rule.derivation.trim().is_empty(), "missing derivation for {} -> {}", rule.from_unit, rule.to_unit);
            assert!(rule.derivation.contains('='), "derivation must show its arithmetic: {}", rule.derivation);
            assert!(
                !rule.derivation.to_ascii_lowercase().contains("copied from")
                    && !rule.derivation.to_ascii_lowercase().contains("vendor factor"),
                "a vendor table alone is not an arithmetic source: {}",
                rule.derivation
            );
        }

        let qv = UNIT_RULES
            .iter()
            .find(|rule| rule.from_unit == "MEQ/L" && rule.to_unit == "meq/mL")
            .expect("the corrected G-D-1 row");
        assert_eq!(qv.factor, 1.0e-3, "1 L = 10^3 mL, so meq/L -> meq/mL is x10^-3");
        assert!(!qv.automatic, "§7.1 O-2 requires per-file confirmation despite the known arithmetic");
        let (family, issue) = family_for_import("QV", Some("MEQ/L"));
        assert!(family.is_none(), "the unconfirmed label must not bind or convert");
        assert!(issue.is_some_and(|item| item.designation_required && item.reason.contains("10^-3")));
    }

    #[test]
    fn unit_conversions_only_when_needed() {
        // Sonic us/m → us/ft.
        let mut dt = [656.0_f32, f32::NAN];
        assert!(convert_to_canonical("DTCO", "DT", Some("US/M"), &mut dt).is_some());
        assert!((dt[0] - 199.9).abs() < 0.5, "us/m→us/ft, got {}", dt[0]);
        assert!(dt[1].is_nan(), "missing stays missing");

        // Density kg/m3 → g/cc.
        let mut rhob = [2400.0_f32];
        assert!(convert_to_canonical("RHOZ", "RHOB", Some("KG/M3"), &mut rhob).is_some());
        assert!((rhob[0] - 2.4).abs() < 1e-4);

        // Neutron in percent → v/v.
        let mut nphi = [30.0_f32];
        assert!(convert_to_canonical("TNPH", "NPHI", Some("PU"), &mut nphi).is_some());
        assert!((nphi[0] - 0.30).abs() < 1e-4);

        // Already canonical → no change, returns false.
        let mut gr = [55.0_f32];
        assert!(convert_to_canonical("GR", "GR", Some("GAPI"), &mut gr).is_none());
        assert_eq!(gr[0], 55.0);

        // Unknown unit → left alone.
        let mut x = [1.0_f32];
        assert!(convert_to_canonical("RHOB", "RHOB", Some("FURLONGS"), &mut x).is_none());
    }

    /// SB-INS-016 / SB-INS-T19. The demonstrated `md` permeability → `m` length bridge and its
    /// required refusal come from dossier sections 2.13/2.16.1 and N-NEW-5. No factor exists in
    /// this test because a cross-kind bridge must fail before numeric conversion is constructed.
    #[test]
    fn a_permeability_to_length_bridge_is_refused_before_any_numeric_conversion_exists() {
        let permeability = resolve_unit_token("md").expect("millidarcy is recognised");
        let length = resolve_unit_token("m").expect("metre is recognised");
        assert_eq!(permeability.quantity_kind, QuantityKind::Permeability);
        assert_eq!(length.quantity_kind, QuantityKind::Length);

        let error = validate_unit_bridge("md", "m").unwrap_err();
        assert!(matches!(
            error,
            UnitRegistryError::QuantityKindMismatch {
                from_kind: QuantityKind::Permeability,
                to_kind: QuantityKind::Length,
                ..
            }
        ));
        assert!(error.to_string().contains("quantity-kind mismatch"));
    }

    /// SB-INS-016 / SB-INS-T20. `1 in = 25.4 mm` and `1 ft = 0.3048 m` are the exact
    /// derivations already cited on curves.rs rules 64-80. Both conversions must stay within
    /// their declared quantity kind, and NaN remains missing.
    #[test]
    fn startup_validates_the_typed_unit_registry_and_only_same_kind_bridges_convert() {
        validate_unit_registry().expect("every runtime token and rule is typed");
        let startup = include_str!("lib.rs");
        let validation = startup
            .find("curves::validate_unit_registry()")
            .expect("startup must validate the shipping registry");
        let builder = startup
            .find("tauri::Builder::default()")
            .expect("the desktop builder must remain visible");
        assert!(
            validation < builder,
            "the registry must be validated before the desktop runtime is constructed"
        );
        assert_eq!(
            validate_unit_bridge("mm", "in")
                .unwrap()
                .quantity_kind,
            QuantityKind::Length
        );
        assert_eq!(
            validate_unit_bridge("us/m", "us/ft")
                .unwrap()
                .quantity_kind,
            QuantityKind::Slowness
        );

        let mut diameter = [25.4_f32, f32::NAN];
        let length_conversion =
            convert_to_canonical("CALI", "CALI", Some("mm"), &mut diameter).unwrap();
        assert_eq!(length_conversion.factor, 1.0 / 25.4);
        assert!((diameter[0] - 1.0).abs() < f32::EPSILON);
        assert!(diameter[1].is_nan());

        let mut slowness = [1.0_f32];
        let slowness_conversion =
            convert_to_canonical("DT", "DT", Some("us/m"), &mut slowness).unwrap();
        assert_eq!(slowness_conversion.factor, 0.3048);
        assert!((slowness[0] - 0.3048).abs() < f32::EPSILON);
    }

}
