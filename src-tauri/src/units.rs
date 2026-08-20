//! Depth-index units — metres vs feet.
//!
//! **The contract: one depth unit per PROJECT.** Every depth SandiBumi stores —
//! `standard_curves.depth`, `curve_samples.depth`, tops, zones, `well_path`, `kb`/`td` —
//! is in the project's declared unit. Files whose index is in the other unit are
//! converted on the way in and the import is flagged. Nothing downstream has to ask
//! "which unit is this number in?", because within a project there is only one answer.
//!
//! That is deliberately NOT the same as the unit the user *reads*. The display unit is a
//! view setting that can be flipped at any time without touching stored data; only the
//! label and the number shown change. Keeping the two apart is what makes the toggle
//! safe — a display switch can never corrupt a depth.
//!
//! Why this exists (engineering review F2e, verified): the LAS index unit was parsed at
//! `parsers.rs` and thrown away under `#[allow(dead_code)]`, and `curves.rs` FAMILIES has
//! no DEPTH entry, so `convert_to_canonical` never touched the index. A foot-indexed
//! A foot-indexed Central Sumatra LAS therefore landed its raw foot numbers in the same column as a metric
//! Mahakam well, and the mixing was reported as a clean import. Two places then produced
//! WRONG NUMBERS rather than merely wrong labels:
//!   * `satheight.rs` / `shf_fit.rs` compute `pc = 0.433 * dRho * (h * FT_PER_M)`, i.e.
//!     they assume the height above free water arrived in metres — 3.28x off for a
//!     foot-indexed well.
//!   * `LogCanvasRenderer.PX_PER_UNIT_1_1` derives the true 1:N print scale from
//!     96 px/in / 0.0254 m/in, so every named scale on a foot-indexed well was mislabelled
//!     by the same 3.28x.

use crate::db::{self, DbResult};
use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};

/// Exact international foot (NIST SP 811). The US survey foot (1200/3937 m) differs by
/// 2 ppm — about 5 mm over a 2,500 m well, orders below log sample resolution — so the
/// international foot is used throughout and the difference is not modelled.
pub const M_PER_FT: f64 = 0.3048;

/// The depth-index units field data actually arrives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepthUnit {
    #[serde(rename = "M", alias = "Metres", alias = "metres")]
    Metres,
    #[serde(rename = "FT", alias = "Feet", alias = "feet")]
    Feet,
}

/// An explicitly supplied depth offset. It has no default: callers must provide both magnitude
/// and unit before a point-data duplicate may be perturbed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DepthOffset {
    pub value: f64,
    pub unit: DepthUnit,
}

/// An explicitly supplied tolerance for checking whether a declared depth increment is regular.
/// This is deliberately a distinct type from [`DepthOffset`]: SB-DBM-028 forbids borrowing a snap
/// or duplicate-perturbation distance for sampling-style verification.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DepthTolerance {
    pub value: f64,
    pub unit: DepthUnit,
}

/// Metres — the unit the whole codebase assumed before units existed, and the unit
/// `wells.kb`/`td` and the Field Map's UTM coordinates are already documented in. Every
/// pre-existing test was written against that assumption, so defaulting here keeps their
/// expected numbers meaningful rather than quietly reinterpreting them.
impl Default for DepthUnit {
    fn default() -> Self {
        DepthUnit::Metres
    }
}

impl DepthUnit {
    /// Stable code persisted in the project settings document and `wells.depth_unit`.
    pub fn code(self) -> &'static str {
        match self {
            DepthUnit::Metres => "M",
            DepthUnit::Feet => "FT",
        }
    }

    /// The label shown next to a depth in the UI.
    pub fn label(self) -> &'static str {
        match self {
            DepthUnit::Metres => "m",
            DepthUnit::Feet => "ft",
        }
    }

    pub fn from_code(s: &str) -> Option<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "M" => Some(DepthUnit::Metres),
            "FT" => Some(DepthUnit::Feet),
            _ => None,
        }
    }

    /// Reads a LAS/DLIS index unit string. Returns `None` for anything unrecognized
    /// (including an empty unit) so the caller can decide — never guess a unit, because
    /// guessing wrong is exactly the silent 3.28x error this module exists to stop.
    pub fn parse(s: &str) -> Option<Self> {
        // LAS files write the index unit with no agreed spelling: M, m, METERS, METRES,
        // MT, F, FT, FEET, and the prime mark all occur in real Indonesian deliveries.
        let t: String = s
            .trim()
            .trim_matches(|c: char| c == '.' || c == '"')
            .to_ascii_uppercase();
        match t.as_str() {
            "M" | "MT" | "METER" | "METERS" | "METRE" | "METRES" => Some(DepthUnit::Metres),
            "F" | "FT" | "FOOT" | "FEET" | "'" => Some(DepthUnit::Feet),
            _ => None,
        }
    }
}

/// Converts one depth between units. Non-finite input passes through untouched, so a
/// missing depth stays missing rather than becoming a finite number.
pub fn convert_depth(value: f64, from: DepthUnit, to: DepthUnit) -> f64 {
    if from == to || !value.is_finite() {
        return value;
    }
    match (from, to) {
        (DepthUnit::Feet, DepthUnit::Metres) => value * M_PER_FT,
        (DepthUnit::Metres, DepthUnit::Feet) => value / M_PER_FT,
        _ => value,
    }
}

/// A height expressed in `from` units, converted to FEET.
///
/// Exists for the capillary-pressure law `pc = 0.433 psi/ft/SG · Δρ · h[ft]`, whose
/// constant is per FOOT of column: `satheight.rs` and `shf_fit.rs` used to hardcode
/// `h * FT_PER_M`, silently assuming the height arrived in metres. On a project declared
/// in feet that multiply ran on a height already in feet and returned a Pc 3.28x too
/// high. Call this instead of multiplying — it reads as the unit conversion it is.
pub fn to_feet(value: f64, from: DepthUnit) -> f64 {
    convert_depth(value, from, DepthUnit::Feet)
}

/// Paper millimetres per ONE stored depth unit at a true 1:1 print scale.
///
/// A named print scale is a physical ratio of rock to paper: "1:200" claims 200 units of section
/// per unit of paper, so the answer depends on how long a stored unit actually IS. One metre of
/// rock is 1000 mm on paper at 1:1; one foot is 304.8 mm. Divide by the scale denominator to get
/// millimetres per stored unit at that scale.
///
/// The composite exporter used to compute `1000.0 / scale` unconditionally and then add the
/// resulting metre count straight onto a stored depth. On a foot-declared project that drew a
/// 100 ft interval as if it were 100 m — the sheet is 3.28084x too long, the printed "1:500" is
/// physically about 1:152, and the page breaks land in the wrong place. It is the exact defect
/// `pxPerUnitAt1to1` already fixed on the screen side; the print path did not share it.
///
/// Derived from `M_PER_FT` rather than writing 304.8, so there is one foot in this codebase.
pub fn mm_per_stored_unit(stored: DepthUnit) -> f64 {
    convert_depth(1.0, stored, DepthUnit::Metres) * 1000.0
}

/// How far a core plug may sit from a log sample, or from another measurement of the same plug,
/// and still be treated as the same depth — expressed in the project's OWN unit.
///
/// The intent is ONE standard 6-inch log sample, which is what `plugqc` and the S-factor
/// calibration have always documented: a plug quoted to the centimetre lands inside one sample of
/// its true depth once the core is registered against the log, and anything looser pairs a
/// measurement with rock it did not come from. Both of those shipped the metre figure `0.15` as a
/// bare number and compared it against depths in the project's unit — so on a FOOT project the
/// tolerance silently became 0.15 ft, 1.8 inches, roughly a third of a sample. Pairs that should
/// have matched were reported as "no plug within the depth tolerance", and an S fit was made on
/// whatever handful happened to survive.
///
/// The metre figure stays exactly `0.15` — it is what shipped, it is what the documentation says,
/// and moving it to 0.1524 would change every metre project's pairing for a millimetre. The foot
/// figure is `0.5`, which is six inches exactly. Both are the same statement in their own unit.
pub fn same_depth_tolerance(unit: DepthUnit) -> f64 {
    match unit {
        DepthUnit::Metres => 0.15,
        DepthUnit::Feet => 0.5,
    }
}

/// Converts a whole depth index in place. NaN is preserved (missing stays missing).
/// Defers to `convert_depth` per sample so the scalar and array paths can never drift.
pub fn convert_depths(values: &mut [f32], from: DepthUnit, to: DepthUnit) {
    if from == to {
        return;
    }
    for v in values.iter_mut() {
        *v = convert_depth(*v as f64, from, to) as f32;
    }
}

// --- Project setting -------------------------------------------------------------
//
// Stored as a `documents` row (doc_type "settings", name "units") rather than a new
// table: it is one small JSON blob per project and the documents table already carries
// exactly this kind of project-scoped setting, so no schema migration is needed.

const SETTINGS_DOC_TYPE: &str = "settings";
const SETTINGS_DOC_NAME: &str = "units";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UnitSettings {
    depth: String,
}

/// The project's declared depth unit, or `None` when the project has not declared one
/// yet (a fresh project, before its first import). An undeclared project adopts the unit
/// of the first file imported into it — that is what makes the common case need no
/// decision from the user at all.
pub fn project_depth_unit(conn: &Connection) -> DbResult<Option<DepthUnit>> {
    let json: Option<String> = conn
        .query_row(
            "SELECT json FROM documents WHERE doc_type = ?1 AND name = ?2",
            params![SETTINGS_DOC_TYPE, SETTINGS_DOC_NAME],
            |row| row.get(0),
        )
        .ok();
    let Some(json) = json else { return Ok(None) };
    Ok(serde_json::from_str::<UnitSettings>(&json)
        .ok()
        .and_then(|s| DepthUnit::from_code(&s.depth)))
}

/// Declares (or re-declares) the project's depth unit.
///
/// This does NOT convert data that is already stored — changing the declaration on a
/// project that already holds wells would silently reinterpret every depth in it. The
/// caller is responsible for refusing that; `set_project_depth_unit_checked` is the
/// guarded entry point the UI uses.
pub fn set_project_depth_unit(conn: &Connection, unit: DepthUnit) -> DbResult<()> {
    let json = serde_json::to_string(&UnitSettings { depth: unit.code().to_string() })
        .unwrap_or_else(|_| format!("{{\"depth\":\"{}\"}}", unit.code()));
    db::save_document(conn, SETTINGS_DOC_TYPE, SETTINGS_DOC_NAME, &json)
}

/// Guarded project-unit declaration used by every interactive caller. Once a project
/// contains wells, changing this declaration would reinterpret stored numbers; the only
/// allowed behaviour here is refusal. A real conversion remains a separate migration.
pub fn set_project_depth_unit_checked(conn: &Connection, target: DepthUnit) -> Result<(), String> {
    let current = project_depth_unit(conn).map_err(|error| error.to_string())?;
    if current == Some(target) {
        return Ok(());
    }
    let wells: i64 = conn
        .query_row("SELECT COUNT(*) FROM wells", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if wells > 0 {
        return Err(format!(
            "this project already holds {wells} well(s) whose depths are stored in {}. \
             Changing the unit here would reinterpret every stored depth rather than convert it — \
             switch the DISPLAY unit instead, or start a new project.",
            current.unwrap_or_default().label()
        ));
    }
    set_project_depth_unit(conn, target).map_err(|error| error.to_string())
}

/// Returns the project's declared depth unit or an actionable refusal. A caller that
/// handles physical depths must use this rather than manufacture a default: an
/// undeclared legacy frame can contain either feet or metres, and both interpretations
/// produce plausible-looking numbers.
pub fn require_project_depth_unit(conn: &Connection, operation: &str) -> Result<DepthUnit, String> {
    project_depth_unit(conn)
        .map_err(|error| format!("{operation} cannot read the project's declared depth unit: {error}"))?
        .ok_or_else(|| {
            format!(
                "{operation} requires a declared project depth unit; set it in Data Conventions before continuing"
            )
        })
}

/// Resolves what to do with an imported file's index unit, given the project's state.
#[derive(Debug, Clone, PartialEq)]
pub enum IndexUnitAction {
    /// Project had no declared unit; it adopts the file's.
    Adopted(DepthUnit),
    /// File already matches the project — store as-is.
    Matches(DepthUnit),
    /// File differs — convert the index into the project unit before storing.
    Convert { from: DepthUnit, to: DepthUnit },
}

impl IndexUnitAction {
    /// The non-fatal note to fold into `ImportResult.warning`, or `None` when the file
    /// matched cleanly and there is nothing the user needs to know.
    pub fn note(&self) -> Option<String> {
        match self {
            IndexUnitAction::Matches(_) => None,
            IndexUnitAction::Adopted(u) => {
                Some(format!("project depth unit set to {} from this file's index", u.label()))
            }
            IndexUnitAction::Convert { from, to } => Some(format!(
                "depth index converted from {} to the project unit ({})",
                from.label(),
                to.label()
            )),
        }
    }
}

/// Decides the action for one file. `declared` is the project's unit (None = undeclared),
/// `file` is what the file's index unit parsed to (None = absent/unrecognized).
pub fn resolve_index_unit(
    declared: Option<DepthUnit>,
    file: Option<DepthUnit>,
) -> Result<IndexUnitAction, String> {
    match (declared, file) {
        (None, Some(f)) => Ok(IndexUnitAction::Adopted(f)),
        (None, None) => Err(
            "depth index unit is undeclared: the file index has no usable unit and the project has no declared depth unit; confirm the file's unit before import"
                .to_string(),
        ),
        (Some(p), None) => Err(format!(
            "the file index has no usable depth unit; the project is {}, but the project setting is not a file declaration — confirm the file's unit before import",
            p.code()
        )),
        (Some(p), Some(f)) if p == f => Ok(IndexUnitAction::Matches(p)),
        (Some(p), Some(f)) => Ok(IndexUnitAction::Convert { from: f, to: p }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_index_unit_spellings_that_occur_in_field_las() {
        for s in ["M", "m", " M ", "METRES", "meters", "MT", ".M"] {
            assert_eq!(DepthUnit::parse(s), Some(DepthUnit::Metres), "{s}");
        }
        for s in ["F", "ft", "FEET", "Foot", "'", ".FT"] {
            assert_eq!(DepthUnit::parse(s), Some(DepthUnit::Feet), "{s}");
        }
        // Unrecognized must stay None — guessing is the failure mode this prevents.
        for s in ["", "  ", "IN", "0.1IN", "US/FT", "X"] {
            assert_eq!(DepthUnit::parse(s), None, "{s}");
        }
    }

    #[test]
    fn converts_a_known_depth_both_ways() {
        // 8000 ft is 2438.4 m exactly under the international foot.
        assert!((convert_depth(8000.0, DepthUnit::Feet, DepthUnit::Metres) - 2438.4).abs() < 1e-9);
        assert!((convert_depth(2438.4, DepthUnit::Metres, DepthUnit::Feet) - 8000.0).abs() < 1e-9);
        // Same unit is an identity, and a round trip returns the original.
        assert_eq!(convert_depth(1234.5, DepthUnit::Feet, DepthUnit::Feet), 1234.5);
        let round = convert_depth(
            convert_depth(1234.5, DepthUnit::Feet, DepthUnit::Metres),
            DepthUnit::Metres,
            DepthUnit::Feet,
        );
        assert!((round - 1234.5).abs() < 1e-9);
    }

    #[test]
    fn array_conversion_preserves_missing_values() {
        let mut v = vec![8000.0_f32, f32::NAN, 8001.0];
        convert_depths(&mut v, DepthUnit::Feet, DepthUnit::Metres);
        assert!((v[0] - 2438.4).abs() < 0.01);
        assert!(v[1].is_nan(), "NaN must stay NaN, not become a finite depth");
        assert!((v[2] - 2438.7048).abs() < 0.01);
    }

    #[test]
    fn same_unit_conversion_does_not_touch_the_array() {
        let mut v = vec![8000.0_f32, f32::NAN];
        convert_depths(&mut v, DepthUnit::Feet, DepthUnit::Feet);
        assert_eq!(v[0], 8000.0);
        assert!(v[1].is_nan());
    }

    #[test]
    fn resolves_every_project_file_unit_combination() {
        use DepthUnit::{Feet, Metres};
        // Fresh project adopts the file's unit — the common case, no user decision.
        assert_eq!(resolve_index_unit(None, Some(Feet)).unwrap(), IndexUnitAction::Adopted(Feet));
        // Matching file: stored as-is, and nothing to tell the user.
        assert_eq!(resolve_index_unit(Some(Metres), Some(Metres)).unwrap(), IndexUnitAction::Matches(Metres));
        assert!(resolve_index_unit(Some(Metres), Some(Metres)).unwrap().note().is_none());
        // The case that used to corrupt a project silently.
        assert_eq!(
            resolve_index_unit(Some(Metres), Some(Feet)).unwrap(),
            IndexUnitAction::Convert { from: Feet, to: Metres }
        );
        assert!(resolve_index_unit(Some(Metres), Some(Feet)).unwrap().note().is_some());
        assert!(resolve_index_unit(Some(Feet), None).is_err());
        assert!(resolve_index_unit(None, None).is_err());
    }

    /// SB-CORE-T02. CORRECTNESS. Source: SB-CORE-001 in
    /// `docs/PRD_v2/04_CORE_REQUIREMENTS.md`: an undeclared unit refuses rather
    /// than becoming metres. The declared-feet side prevents a lazy implementation
    /// from always refusing.
    #[test]
    fn an_undeclared_project_depth_unit_refuses_instead_of_becoming_metres() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();

        let error = require_project_depth_unit(&conn, "depth-bearing operation")
            .expect_err("an undeclared project must refuse");
        assert!(error.contains("depth-bearing operation") && error.contains("Data Conventions"));

        set_project_depth_unit(&conn, DepthUnit::Feet).unwrap();
        assert_eq!(
            require_project_depth_unit(&conn, "depth-bearing operation").unwrap(),
            DepthUnit::Feet,
            "a declaration must be carried exactly, never normalized to the old metres default"
        );
    }

    /// CORRECTNESS - SB-DIO-019 / SB-DIO-T31. `21_data-io.md` section 6 T31
    /// permits either an explicit counted migration or refusal while committed data
    /// exists; this project deliberately chooses refusal. The depths are test inputs,
    /// not sourced physical defaults, and their packed bytes are the expected custody.
    #[test]
    fn changing_the_project_depth_unit_is_refused_while_committed_curves_exist_and_nothing_is_rescaled() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        set_project_depth_unit(&conn, DepthUnit::Metres).unwrap();
        let well = uuid::Uuid::new_v4();
        db::insert_well(&conn, well, "UNIT-GUARD", None, None, None).unwrap();
        let depth = vec![1000.0_f32, 1000.5, 1001.0];
        db::insert_standard_curves(
            &conn,
            well,
            depth.clone(),
            vec![50.0; 3],
            vec![2.0; 3],
            vec![0.2; 3],
            vec![2.4; 3],
            vec![80.0; 3],
            vec![10.0; 3],
        )
        .unwrap();

        let stored_before: Vec<f32> = conn
            .prepare("SELECT depth FROM standard_curves WHERE well_id = ?1 ORDER BY depth")
            .unwrap()
            .query_map(params![well.to_string()], |row| row.get(0))
            .unwrap()
            .collect::<duckdb::Result<_>>()
            .unwrap();
        set_project_depth_unit_checked(&conn, DepthUnit::Metres)
            .expect("reasserting the stored unit remains a safe no-op");
        assert_eq!(project_depth_unit(&conn).unwrap(), Some(DepthUnit::Metres));

        let refusal = set_project_depth_unit_checked(&conn, DepthUnit::Feet).unwrap_err();
        assert!(refusal.contains("1 well(s)"), "the affected well count is named: {refusal}");
        assert!(refusal.contains("reinterpret"), "the refusal explains the failure mode: {refusal}");
        assert_eq!(project_depth_unit(&conn).unwrap(), Some(DepthUnit::Metres));
        let stored_after: Vec<f32> = conn
            .prepare("SELECT depth FROM standard_curves WHERE well_id = ?1 ORDER BY depth")
            .unwrap()
            .query_map(params![well.to_string()], |row| row.get(0))
            .unwrap()
            .collect::<duckdb::Result<_>>()
            .unwrap();
        assert_eq!(
            bytemuck::cast_slice::<f32, u8>(&stored_after),
            bytemuck::cast_slice::<f32, u8>(&stored_before),
            "a refused declaration change cannot alter any stored depth byte"
        );
    }
}
