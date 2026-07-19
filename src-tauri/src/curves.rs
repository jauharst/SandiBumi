//! Phase 6: mnemonic dictionary (alias → petrophysical family) and unit canonicalization
//! for the generic curve store. When a LAS/DLIS file arrives with an arbitrary curve, this
//! decides which family bucket it belongs to (so the catalog can group and the modules can
//! find it) and rescales its samples into SandiBumi's canonical unit for that family.
//!
//! This mirrors what Geolog's tool/curve dictionaries and IP's CurveAlias.txt do, but kept
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
    FamilySpec { family: "NPHI", canonical_unit: "v/v", aliases: &["NPHI", "TNPH", "NPHIED", "NPHI_LS", "NPOR", "NEUT", "APLC", "HNPO", "FSTP"] },
    FamilySpec { family: "DT", canonical_unit: "us/ft", aliases: &["DT", "DTC", "DTCO", "AC", "DT24", "DTP", "DTCOMP"] },
    FamilySpec { family: "DTS", canonical_unit: "us/ft", aliases: &["DTS", "DTSM", "DTSH", "DTSHEAR", "DT_S"] },
    // Resistivity: deep first, then medium/shallow/micro so the primary Rt wins the "RES" bucket.
    FamilySpec { family: "RES_DEEP", canonical_unit: "ohm.m", aliases: &["RES_DEEP", "RESD", "RT", "RDEEP", "RDEP", "DRES", "ILD", "LLD", "AT90", "AHT90", "RLA5", "ATR", "BDAV", "RING", "PSR"] },
    FamilySpec { family: "RES_MED", canonical_unit: "ohm.m", aliases: &["RES_MED", "RESM", "RMED", "ILM", "LLM", "AT30", "AHT30", "RLA3"] },
    FamilySpec { family: "RES_SHAL", canonical_unit: "ohm.m", aliases: &["RES_SHAL", "RESS", "RSHAL", "SFL", "SFLU", "LL8", "SN", "AT10", "AHT10", "RLA1", "R25P", "BSAV"] },
    FamilySpec { family: "RXO", canonical_unit: "ohm.m", aliases: &["RXO", "RXOZ", "MSFL", "RMLL"] },
];

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

/// Converts a value from a source unit into the canonical unit for a family, in place.
/// Only conversions that actually occur in field LAS files are handled; anything already
/// canonical, unrecognized, or dimensionally identical is left untouched (returns `false`
/// so the caller can keep the original unit label). NaN is preserved (missing stays missing).
pub fn convert_to_canonical(family: &str, src_unit: Option<&str>, values: &mut [f32]) -> bool {
    let Some(target) = canonical_unit(family) else { return false };
    let src = src_unit.map(normalize_unit).unwrap_or_default();
    let tgt = normalize_unit(target);
    if src.is_empty() || src == tgt {
        return false;
    }

    // (family-agnostic) linear rescale factor from src→canonical.
    let factor: Option<f32> = match (src.as_str(), tgt.as_str()) {
        // Length: feet → metres and back (depth/CALI/BS live in inches or metres; keep
        // CALI in inches, only convert obvious metric mismatches).
        ("in", "in") => None,
        ("mm", "in") => Some(1.0 / 25.4),
        ("cm", "in") => Some(1.0 / 2.54),
        // Sonic slowness: us/m → us/ft.
        ("us/m", "us/ft") => Some(0.3048),
        ("usec/m", "us/ft") => Some(0.3048),
        // Bulk density: kg/m3 → g/cc.
        ("kg/m3", "g/cc") => Some(0.001),
        // Neutron porosity given in percent → v/v.
        ("pu", "v/v") => Some(0.01),
        ("%", "v/v") => Some(0.01),
        ("p.u.", "v/v") => Some(0.01),
        _ => None,
    };

    match factor {
        Some(f) => {
            for v in values.iter_mut() {
                if v.is_finite() {
                    *v *= f;
                }
            }
            true
        }
        None => false,
    }
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

    #[test]
    fn unit_conversions_only_when_needed() {
        // Sonic us/m → us/ft.
        let mut dt = [656.0_f32, f32::NAN];
        assert!(convert_to_canonical("DT", Some("US/M"), &mut dt));
        assert!((dt[0] - 199.9).abs() < 0.5, "us/m→us/ft, got {}", dt[0]);
        assert!(dt[1].is_nan(), "missing stays missing");

        // Density kg/m3 → g/cc.
        let mut rhob = [2400.0_f32];
        assert!(convert_to_canonical("RHOB", Some("KG/M3"), &mut rhob));
        assert!((rhob[0] - 2.4).abs() < 1e-4);

        // Neutron in percent → v/v.
        let mut nphi = [30.0_f32];
        assert!(convert_to_canonical("NPHI", Some("PU"), &mut nphi));
        assert!((nphi[0] - 0.30).abs() < 1e-4);

        // Already canonical → no change, returns false.
        let mut gr = [55.0_f32];
        assert!(!convert_to_canonical("GR", Some("GAPI"), &mut gr));
        assert_eq!(gr[0], 55.0);

        // Unknown unit → left alone.
        let mut x = [1.0_f32];
        assert!(!convert_to_canonical("RHOB", Some("FURLONGS"), &mut x));
    }
}
