//! Phase 6: mnemonic dictionary (alias → petrophysical family) and unit canonicalization
//! for the generic curve store. When a LAS/DLIS file arrives with an arbitrary curve, this
//! decides which family bucket it belongs to (so the catalog can group and the modules can
//! find it) and rescales its samples into SandiBumi's canonical unit for that family.
//!
//! This mirrors what commercial tool/curve dictionaries and IP's CurveAlias.txt do, but kept
//! small and code-resident (no external reference file to drift out of sync).

/// Canonical family + its canonical unit. Families are deliberately coarse — enough to
/// group the catalog and let modules ask "give me the GR" without caring whether the file
/// called it GR, GRN, or CGR.
pub struct FamilySpec {
    pub family: &'static str,
    pub canonical_unit: &'static str,
    /// Uppercased mnemonic aliases that map to this family.
    pub aliases: &'static [&'static str],
}

/// The dictionary. First match wins, checked in order, so put the more specific families
/// before the generic ones if aliases ever overlap.
pub const FAMILIES: &[FamilySpec] = &[
    FamilySpec { family: "GR", canonical_unit: "gAPI", aliases: &["GR", "GRN", "GRD", "CGR", "SGR", "GRGC", "GRKT"] },
    FamilySpec { family: "SP", canonical_unit: "mV", aliases: &["SP", "SPC", "SPR"] },
    FamilySpec { family: "CALI", canonical_unit: "in", aliases: &["CALI", "CAL", "CALS", "CALX", "CALY", "HCAL", "LCAL", "DCAL", "HORD"] },
    FamilySpec { family: "BS", canonical_unit: "in", aliases: &["BS", "BITSIZE", "BIT"] },
    FamilySpec { family: "RHOB", canonical_unit: "g/cc", aliases: &["RHOB", "RHOZ", "RHOBED", "DEN", "ZDEN", "ROBB", "SBD2"] },
    FamilySpec { family: "DRHO", canonical_unit: "g/cc", aliases: &["DRHO", "HDRA", "ZCOR", "DCOR"] },
    FamilySpec { family: "PEF", canonical_unit: "b/e", aliases: &["PEF", "PE", "PEFZ", "PEB", "PDPE"] },
    FamilySpec { family: "NPHI", canonical_unit: "v/v", aliases: &["NPHI", "TNPH", "NPHIED", "NPHI_LS", "NPOR", "NEUT", "APLC", "FPLC", "SNP", "HNPO", "FSTP"] },
    FamilySpec { family: "DT", canonical_unit: "us/ft", aliases: &["DT", "DTC", "DTCO", "AC", "DT24", "DTP", "DTCOMP"] },
    FamilySpec { family: "DTS", canonical_unit: "us/ft", aliases: &["DTS", "DTSM", "DTSH", "DTSHEAR", "DT_S"] },
    FamilySpec { family: "TEMP", canonical_unit: "DEGC", aliases: &["FTEMP"] },
    // Resistivity: deep first, then medium/shallow/micro so the primary Rt wins the "RES" bucket.
    FamilySpec { family: "RES_DEEP", canonical_unit: "ohm.m", aliases: &["RES_DEEP", "RESD", "RT", "RDEEP", "RDEP", "DRES", "ILD", "LLD", "AT90", "AHT90", "RLA5", "ATR", "BDAV", "RING", "PSR"] },
    FamilySpec { family: "RES_MED", canonical_unit: "ohm.m", aliases: &["RES_MED", "RESM", "RMED", "ILM", "LLM", "AT30", "AHT30", "RLA3"] },
    FamilySpec { family: "RES_SHAL", canonical_unit: "ohm.m", aliases: &["RES_SHAL", "RESS", "RSHAL", "SFL", "SFLU", "LL8", "SN", "AT10", "AHT10", "RLA1", "R25P", "BSAV"] },
    FamilySpec { family: "RXO", canonical_unit: "ohm.m", aliases: &["RXO", "RXOZ", "MSFL", "RMLL"] },
];

/// The exact quantity families for which at least one reviewed numeric transform exists.
/// This is a capability list, not a claim that every spelling within the family converts.
pub const CONVERTIBLE_FAMILIES: &[&str] = &["CALI", "BS", "RHOB", "DRHO", "NPHI", "DT", "DTS", "TEMP"];

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

pub const UNIT_RULES: &[UnitRule] = &[
    UnitRule {
        families: &["CALI", "BS"],
        from_unit: "mm",
        to_unit: "in",
        factor: 1.0 / 25.4,
        offset: 0.0,
        derivation: "1 in = 25.4 mm exactly; mm -> in divides by 25.4",
        automatic: true,
    },
    UnitRule {
        families: &["CALI", "BS"],
        from_unit: "cm",
        to_unit: "in",
        factor: 1.0 / 2.54,
        offset: 0.0,
        derivation: "1 in = 2.54 cm exactly; cm -> in divides by 2.54",
        automatic: true,
    },
    UnitRule {
        families: &["DT", "DTS"],
        from_unit: "us/m",
        to_unit: "us/ft",
        factor: 0.3048,
        offset: 0.0,
        derivation: "1 international ft = 0.3048 m; (us/m) x (m/ft) = us/ft",
        automatic: true,
    },
    UnitRule {
        families: &["DT", "DTS"],
        from_unit: "usec/m",
        to_unit: "us/ft",
        factor: 0.3048,
        offset: 0.0,
        derivation: "1 usec = 1 us and 1 international ft = 0.3048 m; factor = 0.3048",
        automatic: true,
    },
    UnitRule {
        families: &["RHOB", "DRHO"],
        from_unit: "kg/m3",
        to_unit: "g/cc",
        factor: 0.001,
        offset: 0.0,
        derivation: "1 g/cc = 1000 kg/m3; kg/m3 -> g/cc divides by 1000",
        automatic: true,
    },
    UnitRule {
        families: &["NPHI"],
        from_unit: "pu",
        to_unit: "v/v",
        factor: 0.01,
        offset: 0.0,
        derivation: "1 porosity unit = 1 percent = 0.01 v/v",
        automatic: true,
    },
    UnitRule {
        families: &["NPHI"],
        from_unit: "%",
        to_unit: "v/v",
        factor: 0.01,
        offset: 0.0,
        derivation: "1 percent = 1/100 = 0.01 v/v",
        automatic: true,
    },
    UnitRule {
        families: &["NPHI"],
        from_unit: "p.u.",
        to_unit: "v/v",
        factor: 0.01,
        offset: 0.0,
        derivation: "p.u. denotes porosity percent; 1 percent = 0.01 v/v",
        automatic: true,
    },
    UnitRule {
        families: &["TEMP"],
        from_unit: "DEGF",
        to_unit: "DEGC",
        factor: 1.0 / 1.8,
        offset: -32.0,
        derivation: "T42 fixes degC = (degF - 32) / 1.8; factor = 1/1.8 and offset = -32 degF",
        automatic: true,
    },
    UnitRule {
        families: &["QV"],
        from_unit: "MEQ/L",
        to_unit: "meq/mL",
        factor: 1.0e-3,
        offset: 0.0,
        derivation: "1 L = 10^3 mL; meq/L -> meq/mL therefore multiplies by 10^-3",
        // Chapter §7.1 O-2: files affected by the vendor defect may already hold
        // meq/mL values under the wrong MEQ/L label, so the correct arithmetic still
        // requires per-file confirmation before it may touch values.
        automatic: false,
    },
];

pub fn convertible_unit_families() -> Vec<String> {
    CONVERTIBLE_FAMILIES.iter().map(|family| (*family).to_string()).collect()
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
        .is_some_and(|canonical| normalize_unit(canonical) == normalize_unit(declared))
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
    if normalize_unit(declared) == "meq/l" {
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
    if normalize_unit(declared) != "ppg" {
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
    let src = src_unit.map(normalize_unit).unwrap_or_default();
    let tgt = normalize_unit(target);
    if src.is_empty() || src == tgt {
        return None;
    }

    let rule = UNIT_RULES.iter().find(|rule| {
        rule.automatic
            && rule.families.contains(&family)
            && normalize_unit(rule.from_unit) == src
            && normalize_unit(rule.to_unit) == tgt
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

/// Lowercases and strips punctuation/spacing so "US/FT", "us/ft", "usft", "US / FT" all
/// compare equal.
fn normalize_unit(u: &str) -> String {
    u.trim()
        .to_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
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
}
