use crate::db;
use crate::parsers::{self, CurveColumns, ParseError};
use duckdb::{params, Connection};
use rayon::prelude::*;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub path: String,
    pub well_id: Option<String>,
    pub well_name: Option<String>,
    pub rows: usize,
    /// Encoding selected by the mandatory byte-tolerant text reader.
    pub text_encoding: Option<String>,
    /// Non-fatal note for a successful import, e.g. rows dropped for a bad/duplicate depth.
    pub warning: Option<String>,
    pub error: Option<String>,
    /// Set name the curves landed under when this file ATTACHED to an existing well
    /// (import-sets mode) instead of creating a new record. None = a well was created.
    pub attached_set: Option<String>,
    /// Typed audit trail for every standard target that matched more than one LAS column.
    pub alias_decisions: Vec<parsers::AliasDecision>,
    pub index_resolution: Option<parsers::IndexResolution>,
    /// Every automatic value conversion, including the source unit and applied factor.
    pub unit_conversions: Vec<crate::curves::UnitConversion>,
    /// Declared units that were preserved because no reviewed conversion applied.
    pub unconverted_units: Vec<crate::curves::UnconvertedUnit>,
    /// Per-file answers to genuinely ambiguous unit symbols.
    pub unit_designations: Vec<crate::curves::UnitDesignation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NonMonotonicIndexDecision {
    AcceptAsDelivered,
}

/// Options for a LAS import batch (the Import LAS dialog's choices).
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct LasImportOptions {
    /// Set name for every curve of this batch in the generic store (one delivery = one
    /// set). None/empty → "RAW". Auto-suffixed PER WELL (`FPROOH` taken → `FPROOH_1`,
    /// Geolog-style) so a re-import can never overwrite an earlier delivery.
    pub set_name: Option<String>,
    /// When true (the dialog default), a file whose well name matches exactly one
    /// existing well ATTACHES its curves to that well as a new set instead of creating
    /// a duplicate well record. False = always create records (the legacy behavior).
    pub attach: bool,
    /// Explicit unit for files whose index declares none. None means no confirmation:
    /// such a file is refused even when the project already has a unit.
    pub file_depth_unit: Option<String>,
    /// Resolved per-channel null lists. An absent mnemonic uses the LAS file/global
    /// convention; a present mnemonic is screened only against its own plural list.
    #[serde(default)]
    pub channel_nulls: parsers::ChannelNullValues,
    /// Many-to-many vendor exception rules, resolved against the file's actual channel names.
    #[serde(default)]
    pub null_rules: Vec<parsers::NullExceptionRule>,
    /// Absent by default. A descending index is a splice, wrap or wrong column until
    /// the user explicitly decides to accept the file's order as delivered.
    #[serde(default, alias = "nonMonotonicIndex")]
    pub non_monotonic_index: Option<NonMonotonicIndexDecision>,
    /// Absent until repeated depths are present and the user chooses one of the
    /// chapter's four policies.
    #[serde(default, alias = "duplicateDepthPolicy")]
    pub duplicate_depth_policy: Option<parsers::DuplicateDepthPolicy>,
    /// Explicit answers keyed by the exact source path; absent is deliberately not a default.
    #[serde(default, alias = "msPerFtMeanings")]
    pub ms_per_ft_meanings: std::collections::HashMap<String, crate::curves::MsPerFtMeaning>,
}

/// Normalizes a user/derived set name to the store's convention: trimmed, upper-cased,
/// spaces collapsed to `_`; empty → RAW.
pub fn canonical_set_name(raw: Option<&str>) -> String {
    let s = raw.unwrap_or("").trim().to_uppercase().replace(' ', "_");
    if s.is_empty() { "RAW".to_string() } else { s }
}

/// Returns `desired` if this well has no curves under it yet, else the first free
/// `desired_1`, `desired_2`, … — the Geolog re-import convention (WIRE, WIRE_1, …):
/// an import NEVER overwrites an existing set of the same name.
pub fn resolve_set_name(conn: &Connection, well_id: &str, desired: &str) -> String {
    let taken = |name: &str| -> bool {
        conn.query_row(
            "SELECT 1 FROM curve_meta WHERE well_id = ?1 AND set_name = ?2 LIMIT 1",
            params![well_id, name],
            |_| Ok(()),
        )
        .is_ok()
    };
    if !taken(desired) {
        return desired.to_string();
    }
    for i in 1.. {
        let candidate = format!("{desired}_{i}");
        if !taken(&candidate) {
            return candidate;
        }
    }
    unreachable!()
}

/// Parses every given LAS file concurrently via `rayon` (CPU-bound), then inserts each
/// well and its curves into DuckDB sequentially — the connection is behind a single lock,
/// so only the parsing step benefits from parallelism, which is also the expensive part.
/// Legacy entry point (no set naming, always create well records) — kept for the test
/// suite's many import call sites; production goes through `import_las_files_with`.
#[cfg(test)]
pub fn import_las_files(
    conn: &Connection,
    paths: &[String],
    progress: Option<&crate::jobs::JobHandle>,
) -> Vec<ImportResult> {
    import_las_files_with(conn, paths, progress, &LasImportOptions::default())
}

/// Import-sets-aware batch import (Phase 9-3 / T-IMP-02): every curve of the batch lands
/// under one named set; files whose well name matches an existing well attach instead of
/// duplicating (when `opts.attach`).
pub fn import_las_files_with(
    conn: &Connection,
    paths: &[String],
    progress: Option<&crate::jobs::JobHandle>,
    opts: &LasImportOptions,
) -> Vec<ImportResult> {
    let parsed: Vec<(String, Result<(String, CurveColumns), ParseError>)> = paths
        .par_iter()
        .map(|path| {
            let result = (|| {
                let well_name = parsers::extract_well_name(path)?;
                let columns = parsers::parse_las_2_with_unit_designation(
                    path,
                    &opts.channel_nulls,
                    &opts.null_rules,
                    opts.ms_per_ft_meanings.get(path).copied(),
                )?;
                Ok::<_, ParseError>((well_name, columns))
            })();
            (path.clone(), result)
        })
        .collect();

    parsed
        .into_iter()
        .map(|(path, result)| {
            // Cancel before the DB write, so clicking Cancel actually stops wells being created.
            // Without this the flag was flipped, every remaining file was still inserted, and the
            // job was then labelled "Cancelled" — the user was told the import stopped while the
            // project filled up with unwanted wells. The parse pass above has already run by this
            // point (it is one up-front par_iter), so cancel stops the writes, not the parsing.
            if progress.map_or(false, |p| p.is_cancelled()) {
                if let Some(p) = progress {
                    p.finish_item(&path, crate::jobs::ItemState::Warned, Some("cancelled".into()));
                }
                return ImportResult {
                    path: path.clone(),
                    well_id: None,
                    well_name: None,
                    rows: 0,
                    text_encoding: None,
                    warning: Some("cancelled before import".into()),
                    error: None,
                    attached_set: None,
                    alias_decisions: Vec::new(),
                    index_resolution: None,
                    unit_conversions: Vec::new(),
                    unconverted_units: Vec::new(),
                    unit_designations: Vec::new(),
                };
            }
            if let Some(p) = progress {
                let base = path.rsplit(['/', '\\']).next().unwrap_or(&path);
                p.set_current(Some(format!("Importing {base}")));
                p.start_item(&path);
            }
            let out = match result {
                Ok((well_name, columns)) => insert_parsed_well(conn, path.clone(), well_name, columns, opts),
                Err(e) => ImportResult { path: path.clone(), well_id: None, well_name: None, rows: 0, text_encoding: None, warning: None, error: Some(e.to_string()), attached_set: None, alias_decisions: Vec::new(), index_resolution: None, unit_conversions: Vec::new(), unconverted_units: Vec::new(), unit_designations: Vec::new() },
            };
            if let Some(p) = progress {
                let (state, msg) = if out.error.is_some() {
                    (crate::jobs::ItemState::Failed, out.error.clone())
                } else if out.warning.is_some() {
                    (crate::jobs::ItemState::Warned, out.warning.clone())
                } else {
                    (crate::jobs::ItemState::Ok, None)
                };
                p.finish_item(&path, state, msg);
            }
            out
        })
        .collect()
}

fn insert_parsed_well(
    conn: &Connection,
    path: String,
    well_name: String,
    mut columns: CurveColumns,
    opts: &LasImportOptions,
) -> ImportResult {
    let well_id = Uuid::new_v4();
    let alias_decisions = columns.alias_decisions.clone();
    let index_resolution = columns.index_resolution.clone();
    let unit_designations = columns.unit_designations.clone();
    let las_version = columns.las_version.clone();
    let unread_sections = columns.unread_sections.clone();
    let text_encoding = columns.text_encoding.clone();
    let declared_step_note = parsers::declared_step_mismatch_note(
        columns.declared_step.as_deref(),
        &columns.depth,
    );

    // Reconcile the file's depth index with the project's declared unit BEFORE anything
    // else touches the depths. A project holds exactly one depth unit (units.rs); a
    // foot-indexed LAS landing its raw numbers in a metric project used to be
    // reported as a clean import while every cross-well comparison silently put 8,000
    // against 2,438 for the same formation.
    let declared = crate::units::project_depth_unit(conn).ok().flatten();
    let declared_file_unit = columns.depth_unit.as_deref().and_then(crate::units::DepthUnit::parse);
    let confirmed_file_unit = match opts.file_depth_unit.as_deref() {
        Some(raw) => match crate::units::DepthUnit::parse(raw) {
            Some(unit) => Some(unit),
            None => {
                return ImportResult {
                    path,
                    well_id: None,
                    well_name: None,
                    rows: 0,
                    text_encoding: Some(text_encoding.clone()),
                    warning: None,
                    error: Some(format!("unrecognized confirmed file depth unit '{raw}'")),
                    attached_set: None,
                    alias_decisions: alias_decisions.clone(),
                    index_resolution: index_resolution.clone(),
                    unit_conversions: Vec::new(),
                    unconverted_units: Vec::new(),
                    unit_designations: unit_designations.clone(),
                }
            }
        },
        None => None,
    };
    let file_unit = declared_file_unit.or(confirmed_file_unit);
    let unit_action = match crate::units::resolve_index_unit(declared, file_unit) {
        Ok(action) => action,
        Err(error) => {
            return ImportResult {
                path,
                well_id: None,
                well_name: None,
                rows: 0,
                text_encoding: Some(text_encoding.clone()),
                warning: None,
                error: Some(error),
                attached_set: None,
                alias_decisions: alias_decisions.clone(),
                index_resolution: index_resolution.clone(),
                unit_conversions: Vec::new(),
                unconverted_units: Vec::new(),
                unit_designations: unit_designations.clone(),
            }
        }
    };
    let stored_unit = match unit_action {
        crate::units::IndexUnitAction::Convert { from, to } => {
            crate::units::convert_depths(&mut columns.depth, from, to);
            to
        }
        crate::units::IndexUnitAction::Adopted(u) => u,
        crate::units::IndexUnitAction::Matches(u) => u,
    };

    let descending_row = columns
        .depth
        .windows(2)
        .position(|pair| pair[1].is_finite() && pair[0].is_finite() && pair[1] < pair[0])
        .map(|previous| previous + 2);
    let non_monotonic_note = if let Some(row) = descending_row {
        match opts.non_monotonic_index {
            Some(NonMonotonicIndexDecision::AcceptAsDelivered) => Some(format!(
                "non-increasing index accepted as delivered; first decrease is at data row {row}"
            )),
            None => {
                return ImportResult {
                    path,
                    well_id: None,
                    well_name: None,
                    rows: 0,
                    text_encoding: Some(text_encoding.clone()),
                    warning: None,
                    error: Some(format!(
                        "non-increasing index at data row {row}; a user decision is required before commit"
                    )),
                    attached_set: None,
                    alias_decisions,
                    index_resolution,
                    unit_conversions: Vec::new(),
                    unconverted_units: Vec::new(),
                    unit_designations: unit_designations.clone(),
                }
            }
        }
    } else {
        None
    };

    let duplicate_count = parsers::duplicate_depth_count(&columns.depth);
    let duplicate_note = if duplicate_count > 0 {
        match opts.duplicate_depth_policy {
            None => {
                return ImportResult {
                    path,
                    well_id: None,
                    well_name: None,
                    rows: 0,
                    text_encoding: Some(text_encoding.clone()),
                    warning: None,
                    error: Some(format!(
                        "{duplicate_count} repeated depth row(s) require a declared duplicate policy before commit"
                    )),
                    attached_set: None,
                    alias_decisions,
                    index_resolution,
                    unit_conversions: Vec::new(),
                    unconverted_units: Vec::new(),
                    unit_designations: unit_designations.clone(),
                }
            }
            Some(parsers::DuplicateDepthPolicy::Refuse) => {
                return ImportResult {
                    path,
                    well_id: None,
                    well_name: None,
                    rows: 0,
                    text_encoding: Some(text_encoding.clone()),
                    warning: None,
                    error: Some(format!(
                        "duplicate-depth policy refuse blocked {duplicate_count} repeated row(s)"
                    )),
                    attached_set: None,
                    alias_decisions,
                    index_resolution,
                    unit_conversions: Vec::new(),
                    unconverted_units: Vec::new(),
                    unit_designations: unit_designations.clone(),
                }
            }
            Some(policy) => {
                let resolved = parsers::resolve_curve_column_duplicates(&mut columns, policy);
                debug_assert_eq!(resolved, duplicate_count);
                Some(format!(
                    "resolved {duplicate_count} repeated depth row(s) with duplicate policy {}",
                    policy.label()
                ))
            }
        }
    } else {
        None
    };

    // Drop non-finite / duplicate depths so the (well_id, depth) PK can't trip and abort the
    // whole file (which would also orphan the well row); report what was removed.
    let report = parsers::sanitize_curve_columns(&mut columns);
    let rows = columns.depth.len();

    // Every row dropped (all depths missing/duplicate — e.g. an unrecognized index whose
    // column 0 is entirely the null sentinel): don't commit a curve-less orphan well, error.
    if rows == 0 {
        return ImportResult {
            path,
            well_id: None,
            well_name: None,
            rows: 0,
            text_encoding: Some(text_encoding.clone()),
            warning: None,
            error: Some(format!(
                "no importable rows: {} had missing depth, {} duplicated an earlier depth",
                report.nonfinite, report.duplicate
            )),
            attached_set: None,
            alias_decisions: alias_decisions.clone(),
            index_resolution: index_resolution.clone(),
            unit_conversions: Vec::new(),
            unconverted_units: Vec::new(),
            unit_designations: unit_designations.clone(),
        };
    }

    // A LAS index is monotonic by spec; a non-monotonic depth after sanitation usually means
    // column 0 was not the true index (an unrecognized-mnemonic file whose first curve is data,
    // imported as depth via the column-0 fallback) — surface it rather than import silently.
    let non_monotonic = columns.depth.windows(2).any(|w| w[0] < w[1])
        && columns.depth.windows(2).any(|w| w[0] > w[1]);
    let mut notes: Vec<String> = Vec::new();
    if let Some(note) = declared_step_note {
        notes.push(note);
    }
    if las_version.as_deref().and_then(|value| value.parse::<f32>().ok()) == Some(3.0) {
        if unread_sections.is_empty() {
            notes.push("LAS 3.0 recognized; no unread sections were present".into());
        } else {
            notes.push(format!(
                "LAS 3.0 recognized; unread sections: {}",
                unread_sections.join(", ")
            ));
        }
    }
    notes.extend(unit_designations.iter().map(crate::curves::UnitDesignation::note));
    notes.extend(alias_decisions.iter().filter_map(|decision| {
        decision.table_entry.as_ref().map(|entry| {
            format!(
                "alias renamed {} to {} via {entry}",
                decision.chosen, decision.target
            )
        })
    }));
    if let Some(note) = non_monotonic_note {
        notes.push(note);
    }
    if let Some(note) = duplicate_note {
        notes.push(note);
    }
    if let Some(n) = unit_action.note() {
        notes.push(n);
    }
    if declared_file_unit.is_none() {
        if let Some(unit) = confirmed_file_unit {
            notes.push(format!("file depth unit explicitly confirmed as {}", unit.code()));
        }
    }
    if !report.is_clean() {
        notes.push(format!(
            "dropped {} row(s) with missing depth and {} with duplicate depth",
            report.nonfinite, report.duplicate
        ));
    }
    if non_monotonic {
        notes.push("depth index is non-monotonic — column 0 may not be the true depth curve".to_string());
    }
    // Wells of the same (normalized) name already in the project. With `opts.attach` (the
    // dialog default) and exactly ONE match, this file's curves ATTACH to that well as a
    // new named set — the Geolog/IP set model (T-IMP-02): a re-delivery lands beside the
    // earlier one instead of fragmenting the well across duplicate records. Ambiguous
    // (several same-named records, from pre-set-era imports) or attach-off falls back to
    // the legacy separate-record behavior, with a warning either way.
    let name_norm = well_name.trim().to_uppercase();
    let matches: Vec<String> = {
        let mut stmt = match conn
            .prepare("SELECT well_id FROM wells WHERE upper(trim(well_name)) = ?1 ORDER BY well_id")
        {
            Ok(s) => s,
            Err(e) => {
                return ImportResult { path, well_id: None, well_name: None, rows: 0, text_encoding: Some(text_encoding.clone()), warning: None, error: Some(e.to_string()), attached_set: None, alias_decisions: alias_decisions.clone(), index_resolution: index_resolution.clone(), unit_conversions: Vec::new(), unconverted_units: Vec::new(), unit_designations: unit_designations.clone() }
            }
        };
        match stmt
            .query_map(params![name_norm], |r| r.get::<_, String>(0))
            .and_then(|rows| rows.collect::<Result<Vec<_>, _>>())
        {
            Ok(v) => v,
            Err(e) => {
                return ImportResult { path, well_id: None, well_name: None, rows: 0, text_encoding: Some(text_encoding.clone()), warning: None, error: Some(e.to_string()), attached_set: None, alias_decisions: alias_decisions.clone(), index_resolution: index_resolution.clone(), unit_conversions: Vec::new(), unconverted_units: Vec::new(), unit_designations: unit_designations.clone() }
            }
        }
    };
    if opts.attach && matches.len() == 1 {
        let out = attach_curves_to_existing_well(
            conn,
            path,
            well_name,
            &matches[0],
            opts,
            notes,
            alias_decisions.clone(),
            index_resolution.clone(),
            unit_designations.clone(),
            text_encoding.clone(),
        );
        if out.error.is_none() {
            if let crate::units::IndexUnitAction::Adopted(unit) = unit_action {
                if let Err(e) = crate::units::set_project_depth_unit(conn, unit) {
                    eprintln!("warning: could not record the project depth unit: {e}");
                }
            }
        }
        return out;
    }
    if matches.len() > 1 {
        notes.push(format!(
            "{} wells named '{well_name}' already exist — ambiguous, imported as a separate record (merge or delete the duplicates first)",
            matches.len()
        ));
    } else if matches.len() == 1 {
        notes.push(format!(
            "a well named '{well_name}' already exists — imported as a separate record"
        ));
    }
    // `warning` is joined AFTER the generic-store load below, so its failure can be reported to
    // the user as a note. It cannot be joined here and it cannot move the load earlier: the load
    // writes curve_meta rows and must run after the well row is committed.

    // Well row + standard curves as one transaction: a failure rolls the well row back
    // instead of stranding a curve-less orphan (with_txn = BEGIN/COMMIT/ROLLBACK).
    let result: db::DbResult<()> = db::with_txn(conn, |conn| {
        db::insert_well(conn, well_id, &well_name, None, None, None)?;
        // Record the unit the stored depths are actually in, alongside the data itself so
        // the two can never drift apart.
        conn.execute(
            "UPDATE wells SET depth_unit = ?2 WHERE well_id = ?1",
            params![well_id.to_string(), stored_unit.code()],
        )?;
        db::insert_standard_curves(
            conn,
            well_id,
            columns.depth,
            columns.gr,
            columns.res,
            columns.nphi,
            columns.rhob,
            columns.dt,
            columns.sp,
        )?;
        Ok(())
    });

    match result {
        Ok(()) => {
            // The project adopts this file's unit only once the well actually committed,
            // so a failed import can't leave a project declaring a unit it holds no data in.
            if let crate::units::IndexUnitAction::Adopted(u) = unit_action {
                if let Err(e) = crate::units::set_project_depth_unit(conn, u) {
                    eprintln!("warning: could not record the project depth unit: {e}");
                }
            }
            // Phase 6: additionally load *every* curve from the file into the generic
            // store (under the batch's set name, default RAW), so PEF/CALI/multiple-runs —
            // anything beyond the fixed 6 — is available even though the legacy
            // `standard_curves` path above still feeds the current UI. A failure here must
            // not fail the whole import (the standard curves are already in), so it's
            // logged, not propagated.
            let set = resolve_set_name(conn, &well_id.to_string(), &canonical_set_name(opts.set_name.as_deref()));
            let mut unit_conversions = Vec::new();
            let mut unconverted_units = Vec::new();
            match import_all_curves_into_generic_store_with_channel_nulls(
                conn,
                &well_id.to_string(),
                &path,
                &set,
                confirmed_file_unit,
                &opts.channel_nulls,
                &opts.null_rules,
                opts.duplicate_depth_policy,
                opts.ms_per_ft_meanings.get(&path).copied(),
            ) {
                Ok(report) => {
                    unit_conversions = report.unit_conversions;
                    unconverted_units = report.unconverted_units;
                    notes.extend(unit_conversions.iter().map(crate::curves::UnitConversion::note));
                    notes.extend(unconverted_units.iter().map(crate::curves::UnconvertedUnit::note));
                }
                Err(e) => {
                    eprintln!("warning: generic-store import for {well_name} failed (standard curves still imported): {e}");
                    // stderr alone is invisible in a release build, so the import used to report a
                    // clean success while every curve beyond the fixed six — PEF, CALI, DTS, a second
                    // run — was silently missing. Modules that later resolve those mnemonics just get
                    // the all-NaN fallback, with no trace anywhere in the app of why.
                    notes.push(format!(
                        "only the six standard curves were loaded — the full-curve load failed: {e}"
                    ));
                }
            }
            let warning = (!notes.is_empty()).then(|| notes.join("; "));
            ImportResult { path, well_id: Some(well_id.to_string()), well_name: Some(well_name), rows, text_encoding: Some(text_encoding), warning, error: None, attached_set: None, alias_decisions, index_resolution, unit_conversions, unconverted_units, unit_designations }
        }
        Err(e) => ImportResult { path, well_id: None, well_name: None, rows: 0, text_encoding: Some(text_encoding), warning: None, error: Some(e.to_string()), attached_set: None, alias_decisions, index_resolution, unit_conversions: Vec::new(), unconverted_units: Vec::new(), unit_designations },
    }
}

/// The attach half of the import-sets model: writes every curve of `path` into an
/// EXISTING well's generic store under the batch's set name (auto-suffixed per well so
/// nothing is ever overwritten), touching neither the well row nor `standard_curves` —
/// the first delivery's six keep driving the legacy log-view path. The generic-store
/// loader applies the same depth-unit reconciliation as the create path.
fn attach_curves_to_existing_well(
    conn: &Connection,
    path: String,
    well_name: String,
    well_id: &str,
    opts: &LasImportOptions,
    notes: Vec<String>,
    alias_decisions: Vec<parsers::AliasDecision>,
    index_resolution: Option<parsers::IndexResolution>,
    unit_designations: Vec<crate::curves::UnitDesignation>,
    text_encoding: String,
) -> ImportResult {
    let set = resolve_set_name(conn, well_id, &canonical_set_name(opts.set_name.as_deref()));
    match import_all_curves_into_generic_store_with_channel_nulls(
        conn,
        well_id,
        &path,
        &set,
        opts.file_depth_unit.as_deref().and_then(crate::units::DepthUnit::parse),
        &opts.channel_nulls,
        &opts.null_rules,
        opts.duplicate_depth_policy,
        opts.ms_per_ft_meanings.get(&path).copied(),
    ) {
        // A normal attach is a SUCCESS, not a warning — `attached_set` carries the story
        // and the frontend reports it separately. Only genuine notes (unit reconciliation,
        // dropped rows) reach `warning`.
        Ok(report) => {
            let mut notes = notes;
            notes.extend(report.unit_conversions.iter().map(crate::curves::UnitConversion::note));
            notes.extend(report.unconverted_units.iter().map(crate::curves::UnconvertedUnit::note));
            ImportResult {
                path,
                well_id: Some(well_id.to_string()),
                well_name: Some(well_name),
                rows: report.rows,
                text_encoding: Some(text_encoding),
                warning: (!notes.is_empty()).then(|| notes.join("; ")),
                error: None,
                attached_set: Some(set),
                alias_decisions,
                index_resolution,
                unit_conversions: report.unit_conversions,
                unconverted_units: report.unconverted_units,
                unit_designations,
            }
        }
        // Attaching IS the import here (no well/standard-curve write happened), so a
        // loader failure is a real per-file error, not a note.
        Err(e) => ImportResult { path, well_id: None, well_name: None, rows: 0, text_encoding: Some(text_encoding), warning: None, error: Some(e.to_string()), attached_set: None, alias_decisions, index_resolution, unit_conversions: Vec::new(), unconverted_units: Vec::new(), unit_designations },
    }
}

/// Re-reads a LAS file keeping all curves and writes each into `curve_meta`/`curve_samples`
/// under `set_name`, tagging family (via the mnemonic dictionary) and normalizing units
/// where a conversion is known. The unit stored is the canonical one when converted, else
/// the file's original unit. Returns `(curves_written, rows)`.
#[allow(dead_code)] // compatibility entry point for tests/callers with no channel override
pub fn import_all_curves_into_generic_store(
    conn: &Connection,
    well_id: &str,
    path: &str,
    set_name: &str,
    confirmed_file_unit: Option<crate::units::DepthUnit>,
) -> db::DbResult<(usize, usize)> {
    import_all_curves_into_generic_store_with_channel_nulls(
        conn,
        well_id,
        path,
        set_name,
        confirmed_file_unit,
        &parsers::ChannelNullValues::new(),
        &[],
        None,
        None,
    )
    .map(|report| (report.curves_written, report.rows))
}

struct GenericCurveImportReport {
    curves_written: usize,
    rows: usize,
    unit_conversions: Vec<crate::curves::UnitConversion>,
    unconverted_units: Vec<crate::curves::UnconvertedUnit>,
}

fn import_all_curves_into_generic_store_with_channel_nulls(
    conn: &Connection,
    well_id: &str,
    path: &str,
    set_name: &str,
    confirmed_file_unit: Option<crate::units::DepthUnit>,
    channel_nulls: &parsers::ChannelNullValues,
    null_rules: &[parsers::NullExceptionRule],
    duplicate_depth_policy: Option<parsers::DuplicateDepthPolicy>,
    ms_per_ft_meaning: Option<crate::curves::MsPerFtMeaning>,
) -> db::DbResult<GenericCurveImportReport> {
    let mut frame = match parsers::parse_las_2_all_with_null_rules(path, channel_nulls, null_rules) {
        Ok(f) => f,
        Err(e) => return Err(db::DbError::LengthMismatch(format!("parse_las_2_all: {e}"))),
    };
    // This re-reads the same file the standard-curve path already imported, so it MUST
    // apply the identical index conversion — otherwise the two stores would hold the same
    // curves at depths 3.28x apart and every generic-store lookup would miss.
    let declared = crate::units::project_depth_unit(conn)?;
    let file_unit = frame
        .depth_unit
        .as_deref()
        .and_then(crate::units::DepthUnit::parse)
        .or(confirmed_file_unit);
    let action = crate::units::resolve_index_unit(declared, file_unit)
        .map_err(db::DbError::LengthMismatch)?;
    match action {
        crate::units::IndexUnitAction::Convert { from, to } => {
            crate::units::convert_depths(&mut frame.depth, from, to);
        }
        crate::units::IndexUnitAction::Adopted(_)
        | crate::units::IndexUnitAction::Matches(_) => {}
    }
    let duplicate_count = parsers::duplicate_depth_count(&frame.depth);
    if duplicate_count > 0 {
        match duplicate_depth_policy {
            Some(parsers::DuplicateDepthPolicy::Refuse) | None => {
                return Err(db::DbError::LengthMismatch(format!(
                    "{duplicate_count} repeated depth row(s) have no resolving duplicate policy"
                )))
            }
            Some(policy) => {
                parsers::resolve_las_frame_duplicates(&mut frame, policy);
            }
        }
    }
    // curve_samples has PK (curve_id, depth) just like standard_curves, so the same non-finite
    // / duplicate depths the standard-curves path drops would otherwise abort each curve's
    // insert here — silently, since this whole import is best-effort (its Err is only logged).
    // Sanitize depth + every curve in lockstep before writing (identical keep-set to the
    // standard path, so both stores hold the same rows for the same file).
    parsers::sanitize_las_frame(&mut frame);
    if frame.depth.is_empty() {
        return Ok(GenericCurveImportReport {
            curves_written: 0,
            rows: 0,
            unit_conversions: Vec::new(),
            unconverted_units: Vec::new(),
        });
    }

    let mut curves_written = 0usize;
    let mut unit_conversions = Vec::new();
    let mut unconverted_units = Vec::new();
    for raw in &frame.curves {
        let mut values = raw.values.clone();
        // Align to the depth column length (defensive: malformed files can short a column).
        if values.len() != frame.depth.len() {
            values.resize(frame.depth.len(), f32::NAN);
        }
        let mut unit = raw.unit.clone();
        let resolved_ms_per_ft = crate::curves::is_ms_per_ft(raw.unit.as_deref());
        let (fam, rejected_alias) = if resolved_ms_per_ft {
            let meaning = ms_per_ft_meaning.ok_or_else(|| {
                db::DbError::LengthMismatch(format!(
                    "curve {} declares MS/FT and has no per-file quantity designation",
                    raw.mnemonic
                ))
            })?;
            match meaning {
                crate::curves::MsPerFtMeaning::MicrosecondsPerFoot => {
                    let family = crate::curves::family_for(&raw.mnemonic)
                        .filter(|family| matches!(family.family, "DT" | "DTS"));
                    unit = Some("us/ft".to_string());
                    (family, None)
                }
                crate::curves::MsPerFtMeaning::MillisiemensPerFoot => (None, None),
            }
        } else {
            crate::curves::family_for_import(&raw.mnemonic, raw.unit.as_deref())
        };
        let family = fam.map(|f| f.family);
        if let Some(rejected) = rejected_alias {
            unconverted_units.push(rejected);
        } else if resolved_ms_per_ft {
            // The per-file designation is already returned by the standard parser. It is
            // neither an automatic conversion nor an unresolved pass-through.
        } else if let Some(f) = fam {
            if let Some(conversion) = crate::curves::convert_to_canonical(
                &raw.mnemonic,
                f.family,
                raw.unit.as_deref(),
                &mut values,
            ) {
                unit = Some(f.canonical_unit.to_string());
                unit_conversions.push(conversion);
            } else if let Some(unconverted) = crate::curves::unconverted_unit(
                &raw.mnemonic,
                Some(f.family),
                raw.unit.as_deref(),
            ) {
                unconverted_units.push(unconverted);
            }
        } else if let Some(unconverted) =
            crate::curves::unconverted_unit(&raw.mnemonic, None, raw.unit.as_deref())
        {
            unconverted_units.push(unconverted);
        }
        let curve_id =
            db::upsert_curve_meta(conn, well_id, set_name, &raw.mnemonic, unit.as_deref(), family, Some("LAS import"), None)?;
        db::insert_curve_samples(conn, &curve_id, &frame.depth, &values)?;
        curves_written += 1;
    }
    Ok(GenericCurveImportReport {
        curves_written,
        rows: frame.depth.len(),
        unit_conversions,
        unconverted_units,
    })
}

/// Parses a deviation-survey CSV (columns MD/INC/AZI, alias-tolerant) and stores the
/// computed minimum-curvature TVD/TVDSS in `well_path` for one well. `datum_elevation`
/// (KB above MSL) is used for TVDSS; if omitted, the well's `kb` is used, else 0.
///
/// `survey_name` (T-IMP-12) versions the survey: a definitive survey imported over a
/// preliminary one becomes a SECOND survey (auto-suffixed if the name is taken), not a
/// replacement, and the new one becomes active — so the TVD/TVDSS materialized below is
/// the geometry the user just delivered, while the old survey stays switchable.
pub fn import_deviation_csv(
    conn: &Connection,
    well_id: &str,
    path: &str,
    datum_elevation: Option<f32>,
    survey_name: Option<&str>,
) -> CoreImportResult {
    let exists: bool = conn
        .query_row("SELECT 1 FROM wells WHERE well_id = ?1", params![well_id], |_| Ok(true))
        .unwrap_or(false);
    if !exists {
        return CoreImportResult { path: path.to_string(), rows: 0, error: Some(format!("unknown well '{well_id}'")), index_resolution: None };
    }

    let survey = match parsers::parse_deviation_csv(path) {
        Ok(s) => s,
        Err(e) => return CoreImportResult { path: path.to_string(), rows: 0, error: Some(e.to_string()), index_resolution: None },
    };
    if survey.md.is_empty() {
        return CoreImportResult { path: path.to_string(), rows: 0, error: Some("no survey stations found".into()), index_resolution: None };
    }

    let datum = datum_elevation.unwrap_or_else(|| {
        conn.query_row("SELECT kb FROM wells WHERE well_id = ?1", params![well_id], |r| r.get::<_, Option<f32>>(0))
            .ok()
            .flatten()
            .unwrap_or(0.0)
    });
    let stations = crate::deviation::minimum_curvature(&survey.md, &survey.inc, &survey.azi, datum);
    let rows = stations.len();
    let desired = survey_name
        .map(|s| s.trim().to_uppercase().replace(' ', "_"))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "SURVEY".to_string());
    let name = match db::resolve_survey_name(conn, well_id, &desired) {
        Ok(n) => n,
        Err(e) => return CoreImportResult { path: path.to_string(), rows: 0, error: Some(e.to_string()), index_resolution: None },
    };
    match db::insert_well_path(conn, well_id, &name, Some(path), Some(datum), &stations) {
        Ok(()) => {
            // Materialize TVD/TVDSS onto the log grid so height modules (sw_height, the SHF
            // fits, the TVDSS correlation view) can fetch them by name. Best-effort: the
            // survey itself is already saved; a well with no logs yet is a no-op (0 samples)
            // and the user can recompute via `materialize_tvd` after importing logs.
            let _ = materialize_tvd_curves(conn, well_id);
            CoreImportResult { path: path.to_string(), rows, error: None, index_resolution: None }
        }
        Err(e) => CoreImportResult { path: path.to_string(), rows: 0, error: Some(e.to_string()), index_resolution: None },
    }
}

/// Resamples a well's deviation survey (`well_path`) onto its standard-curve depth grid and
/// writes the result as fetchable `TVD` and `TVDSS` computed curves — the bridge that lets
/// `sw_height`'s TVD input, the SHF-fitting modules' TVDSS input, and the correlation TVDSS
/// depth-mode consume the survey. Returns the number of samples written (per curve); 0 when
/// the well has no survey (vertical — TVD == MD, and callers already fall back to MD) or no
/// logs yet (no depth grid to hang the curves on). Refreshes its OWN prior computed TVD/TVDSS
/// in place, but NEVER overwrites a TVD/TVDSS the user imported from a vendor LAS/DLIS (see the
/// import guard below), so a re-import or a KB edit + recompute is safe.
pub fn materialize_tvd_curves(conn: &Connection, well_id: &str) -> db::DbResult<usize> {
    let stations = db::get_well_path(conn, well_id)?;
    if stations.is_empty() {
        return Ok(0);
    }
    let path: Vec<crate::deviation::Station> = stations
        .iter()
        .map(|s| crate::deviation::Station { md: s.md, inc: s.inc, azi: s.azi, tvd: s.tvd, tvdss: s.tvdss })
        .collect();
    // Empty name list → just the standard depth grid for this well.
    let (depth, _cols) = crate::equations::fetch_curve_frame(conn, well_id, &[])?;
    if depth.is_empty() {
        return Ok(0);
    }
    let mut tvd = Vec::with_capacity(depth.len());
    let mut tvdss = Vec::with_capacity(depth.len());
    for &d in &depth {
        let (t, ss) = crate::deviation::sample_at(&path, d);
        tvd.push(t);
        tvdss.push(ss);
    }
    // A survey-derived COMPUTED curve outranks the generic RAW store in fetch_curve_frame, so
    // writing TVD/TVDSS unconditionally would SILENTLY shadow an authoritative curve the user
    // imported from a vendor LAS/DLIS — with a possibly wrong datum (a well with no KB falls
    // back to a sea-level datum → TVDSS = −TVD) or NaN outside the survey's MD range, and no
    // recourse via the Curve Catalog's Promote (it is disabled on a "served by computed" row).
    // So: only materialize a name the well does NOT already resolve from an import, and clear
    // any prior survey-derived computed curve when an import IS present, so the import wins.
    let mut written = 0usize;
    for (name, values) in [("TVD", &tvd), ("TVDSS", &tvdss)] {
        let imported: bool = conn
            .query_row(
                "SELECT 1 FROM curve_meta WHERE well_id = ?1 AND set_name = 'RAW'
                   AND (upper(mnemonic) = upper(?2) OR upper(family) = upper(?2)) LIMIT 1",
                params![well_id, name],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if imported {
            crate::equations::delete_computed_curve(conn, well_id, name)?;
        } else {
            crate::equations::write_computed_curve(conn, well_id, &depth, name, values)?;
            written = depth.len();
        }
    }
    Ok(written)
}

#[derive(Debug, Clone, Serialize)]
pub struct CoreImportResult {
    pub path: String,
    pub rows: usize,
    pub error: Option<String>,
    pub index_resolution: Option<parsers::IndexResolution>,
}

/// Parses a routine-core-analysis CSV into a NEW core set on the given well (legacy
/// single-well path, kept for the tests and any caller that has no wizard). The set is
/// named CORE, auto-suffixed if that name is taken — an import never overwrites an earlier
/// delivery. Unlike LAS import, this attaches to an existing well rather than creating one.
pub fn import_core_csv(conn: &Connection, well_id: &str, path: &str) -> CoreImportResult {
    import_core_csv_with_depth_column(conn, well_id, path, None)
}

pub fn import_core_csv_with_depth_column(
    conn: &Connection,
    well_id: &str,
    path: &str,
    designated_depth_column: Option<usize>,
) -> CoreImportResult {
    let exists: bool = conn
        .query_row("SELECT 1 FROM wells WHERE well_id = ?1", params![well_id], |_| Ok(true))
        .unwrap_or(false);
    if !exists {
        return CoreImportResult { path: path.to_string(), rows: 0, error: Some(format!("unknown well '{well_id}'")), index_resolution: None };
    }

    let columns = match parsers::parse_core_csv_with_depth_column(path, designated_depth_column) {
        Ok(c) => c,
        Err(e) => return CoreImportResult { path: path.to_string(), rows: 0, error: Some(e.to_string()), index_resolution: None },
    };
    let rows = columns.depth.len();
    let index_resolution = columns.index_resolution.clone();
    let set = match db::resolve_core_set_name(conn, well_id, "CORE") {
        Ok(s) => s,
        Err(e) => return CoreImportResult { path: path.to_string(), rows: 0, error: Some(e.to_string()), index_resolution },
    };
    match db::insert_core_data(
        conn,
        well_id,
        &set,
        Some(path),
        &columns.depth,
        &columns.cpor,
        &columns.cperm,
        &columns.cgd,
        &columns.csw,
    ) {
        Ok(()) => CoreImportResult { path: path.to_string(), rows, error: None, index_resolution },
        Err(e) => CoreImportResult { path: path.to_string(), rows: 0, error: Some(e.to_string()), index_resolution },
    }
}

/// Per-well outcome of a multi-well core table import (T-IMP-07).
#[derive(Debug, Clone, Serialize)]
pub struct CoreWellOutcome {
    /// The well name as written in the FILE (or the fallback well's name when the file
    /// has no well column).
    pub well_name: String,
    /// Rows carried for this name in the file.
    pub rows: usize,
    /// Rows actually stored (post depth-dedup); 0 when the name didn't import.
    pub imported: usize,
    /// The core SET the plugs landed in on THIS well — the requested name, or an
    /// auto-suffixed one when that well already carried a delivery of that name
    /// (T-IMP-08: an import never overwrites an earlier core delivery).
    pub set_name: Option<String>,
    /// None = imported cleanly; Some = why this name's rows were skipped.
    pub problem: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoreTableImportResult {
    pub path: String,
    pub rows_imported: usize,
    pub wells_imported: usize,
    pub outcomes: Vec<CoreWellOutcome>,
    /// Rows with a blank well cell in a well-routed file — skipped, never misrouted
    /// (same rule as multi-well tops import).
    pub skipped_blank_well: usize,
    /// Aux point-data rows written from the file's EXTRA columns (0 when none were asked
    /// for), and which columns they came from — reported so the dialog can say out loud
    /// what landed beside the four core measurements.
    pub extra_rows: usize,
    pub extra_items: Vec<String>,
    /// Numeric text is parsed at f64 precision before the deliberate f32 storage cast.
    /// The report names that boundary and counts only values that actually changed.
    pub precision: parsers::SamplePrecisionReport,
    pub error: Option<String>,
}

/// Imports one core table under a dialog-confirmed mapping (probe → confirm → commit).
///
/// Routing: with a well column mapped, rows go to the project well matching their cell's
/// normalized name — exactly one match imports; zero or several report and skip (the
/// same rules LAS attach uses, so pre-set-era duplicate records can't be guessed at).
/// Without a well column, everything goes to `fallback_well_id` (the selected well).
/// Depths convert from `depth_unit` (the dialog's confirmed file unit; None = already
/// the project unit) to the project's declared unit — a feet-plugged core CSV landing
/// raw in a metric project would overlay 3.28× off, silently. Per-well semantics stay
/// replace-on-reimport (`insert_core_data`).
///
/// `mapping.extras` names columns beyond the four core measurements (lithology text,
/// So, Kv/Kh, sample ids …): those land in `aux_data` under `extras_dataset` (default
/// "CORE") at the same converted plug depths — numeric cells as numbers, everything else
/// as text — so a wide lab export imports whole in one pass instead of needing a second
/// Import Aux run. Replace-on-reimport per (well, dataset), matching the core discipline.
///
/// `set_name` (T-IMP-08) names the DELIVERY. It is resolved PER WELL, so a name already
/// used on one well is suffixed there (`RCAL` → `RCAL_1`) while other wells still get the
/// plain name — the plugs of an earlier delivery are never overwritten, and the newly
/// imported set becomes the well's active one.
pub fn import_core_table(
    conn: &Connection,
    path: &str,
    mapping: &parsers::CoreMapping,
    depth_unit: Option<&str>,
    fallback_well_id: Option<&str>,
    extras_dataset: Option<&str>,
    set_name: Option<&str>,
    follow_core: bool,
) -> CoreTableImportResult {
    let fail = |e: String| CoreTableImportResult {
        path: path.to_string(),
        rows_imported: 0,
        wells_imported: 0,
        outcomes: Vec::new(),
        skipped_blank_well: 0,
        extra_rows: 0,
        extra_items: Vec::new(),
        precision: parsers::SamplePrecisionReport::new("f64 numeric parse", "f32 storage", 0),
        error: Some(e),
    };

    let table = match parsers::parse_core_table_mapped(path, mapping) {
        Ok(r) => r,
        Err(e) => return fail(e.to_string()),
    };
    let precision = parsers::SamplePrecisionReport::new(
        "f64 numeric parse",
        "f32 storage",
        table.precision_reduced_values,
    );
    let rows = table.rows;
    let extra_names = table.extra_names;
    let extras_dataset = extras_dataset
        .map(|d| d.trim().to_uppercase())
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| "CORE".to_string());
    let desired_set = set_name
        .map(|s| s.trim().to_uppercase().replace(' ', "_"))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "CORE".to_string());
    if rows.is_empty() {
        return fail("no rows with a parsable depth".into());
    }

    // Group rows by their routing target, keeping file order within each group.
    let mut skipped_blank_well = 0usize;
    let mut groups: Vec<(String, Vec<&parsers::MappedCoreRow>)> = Vec::new();
    let mut fallback_rows: Vec<&parsers::MappedCoreRow> = Vec::new();
    for r in &rows {
        match (&r.well, mapping.well) {
            (Some(name), Some(_)) => match groups.iter_mut().find(|(n, _)| n == name) {
                Some((_, list)) => list.push(r),
                None => groups.push((name.clone(), vec![r])),
            },
            // A blank cell in a well-routed file: skipping is the only safe answer —
            // guessing "probably the previous row's well" would misroute lab padding rows.
            (None, Some(_)) => skipped_blank_well += 1,
            _ => fallback_rows.push(r),
        }
    }

    let project_unit = crate::units::project_depth_unit_or_default(conn);
    let file_unit = depth_unit.and_then(crate::units::DepthUnit::parse).unwrap_or(project_unit);

    let mut outcomes: Vec<CoreWellOutcome> = Vec::new();
    let mut rows_imported = 0usize;
    let mut wells_imported = 0usize;
    let mut extra_rows = 0usize;

    let mut store = |well_id: &str, well_name: &str, list: &[&parsers::MappedCoreRow], outcomes: &mut Vec<CoreWellOutcome>| {
        let mut depth: Vec<f32> = list.iter().map(|r| r.depth).collect();
        crate::units::convert_depths(&mut depth, file_unit, project_unit);
        // "These depths came from the core report": place every row through this well's own core
        // depth record, exactly as a late point-data or SCAL delivery does. Resolved PER WELL
        // because a multi-well file routes by its WELL column and each well has its own record.
        //
        // Off by default and never inferred — nothing in a delimited text file says which depth
        // scale it was written on, so this is the user's declaration. Ticking it on a FRESH core
        // delivery would place the new plugs by the OLD core's correction, which is why the box
        // says what it says rather than "correct the depths".
        let mut follow_note: Option<String> = None;
        if follow_core {
            let mut notes: Vec<String> = Vec::new();
            let pairs = core_record(conn, well_id, true, well_name, &mut notes);
            if !pairs.is_empty() {
                let mut outside = 0usize;
                for d in depth.iter_mut() {
                    if !d.is_finite() {
                        continue;
                    }
                    let (mapped, extrapolated) = db::map_core_depth(&pairs, *d);
                    if extrapolated {
                        outside += 1;
                    }
                    *d = mapped;
                }
                note_mapping(&mut notes, well_name, &pairs, depth.len(), outside);
            }
            if !notes.is_empty() {
                follow_note = Some(notes.join("; "));
            }
        }
        let mut cpor: Vec<f32> = list.iter().map(|r| r.cpor).collect();
        let mut cperm: Vec<f32> = list.iter().map(|r| r.cperm).collect();
        let mut cgd: Vec<f32> = list.iter().map(|r| r.cgd).collect();
        let mut csw: Vec<f32> = list.iter().map(|r| r.csw).collect();
        let mut extras: Vec<&Vec<Option<String>>> = list.iter().map(|r| &r.extras).collect();
        // Depth-dedup per WELL (first kept), matching the legacy path — the core_data PK
        // is (well_id, depth), so one repeated plug depth would abort the well's insert.
        // The extras ride along on the same surviving rows, so they stay depth-aligned.
        let (keep, report) = parsers::depth_keep_indices(&depth);
        if !report.is_clean() {
            let take = |src: &[f32]| -> Vec<f32> { keep.iter().map(|&i| src[i]).collect() };
            depth = take(&depth);
            cpor = take(&cpor);
            cperm = take(&cperm);
            cgd = take(&cgd);
            csw = take(&csw);
            extras = keep.iter().map(|&i| extras[i]).collect();
        }
        // Resolved PER WELL: the same delivery name may be free on one well and already
        // used on another, and neither well's earlier plugs may be overwritten.
        let set = match db::resolve_core_set_name(conn, well_id, &desired_set) {
            Ok(s) => s,
            Err(e) => {
                outcomes.push(CoreWellOutcome {
                    well_name: well_name.to_string(),
                    rows: list.len(),
                    imported: 0,
                    set_name: None,
                    problem: Some(e.to_string()),
                });
                return;
            }
        };
        // A table claiming NO core measurement is point data, not a core delivery, and writing
        // one anyway is destructive in a way nothing on screen would show: `insert_core_data`
        // registers the set and makes it ACTIVE, so importing an XRD or CEC table through Intake
        // would displace the well's real plugs with a set of empty ones — and every core reader
        // (the phi-k cloud, Plug QC, Register Depth, the S-factor fit) follows the active set, so
        // they would all go quiet at once. Found by the follow-core test, whose own first import
        // silently replaced the core it was meant to follow.
        //
        // The extras are still written, at the same mapped depths, under their own delivery name.
        let has_core_measurement = cpor.iter().chain(&cperm).chain(&cgd).chain(&csw).any(|v| v.is_finite());
        let core_write = if has_core_measurement {
            db::insert_core_data(conn, well_id, &set, Some(path), &depth, &cpor, &cperm, &cgd, &csw)
        } else {
            Ok(())
        };
        match core_write {
            Ok(()) => {
                rows_imported += depth.len();
                wells_imported += 1;
                let mut problem = (!report.is_clean())
                    .then(|| format!("{} duplicate depth row(s) dropped (first kept)", report.duplicate));
                // Following the core is reported even when it changed nothing — a user who ticked
                // the box and saw silence has no way to tell whether it worked.
                if let Some(n) = follow_note.clone() {
                    problem = Some(match problem {
                        Some(p) => format!("{p}; {n}"),
                        None => n,
                    });
                }
                if !extra_names.is_empty() {
                    let mut aux: Vec<db::AuxRow> = Vec::new();
                    for (d, cells) in depth.iter().zip(&extras) {
                        for (item, raw) in extra_names.iter().zip(cells.iter()) {
                            let Some(raw) = raw else { continue };
                            let num = parsers::parse_numeric_text_to_f32(raw).map(|(stored, _)| stored);
                            aux.push(db::AuxRow {
                                dataset: extras_dataset.clone(),
                                depth_top: *d,
                                depth_base: None,
                                item: item.clone(),
                                value_num: num,
                                value_text: if num.is_some() { None } else { Some(raw.clone()) },
                            });
                        }
                    }
                    // The extras ARE part of this core delivery, so they carry the same set
                    // name — switching the well's core set switches its extras with it
                    // instead of leaving a mismatched pair behind.
                    // A pure point-data table has no core set to belong to, so its delivery is
                    // named by the user's own set name rather than by a core set that was never
                    // written.
                    let aux_set = if has_core_measurement { set.clone() } else { desired_set.clone() };
                    match db::insert_aux_data(conn, well_id, &extras_dataset, &aux_set, Some(path), &aux) {
                        Ok(()) => extra_rows += aux.len(),
                        Err(e) => {
                            let note = format!("extra columns not stored: {e}");
                            problem = Some(match problem {
                                Some(p) => format!("{p}; {note}"),
                                None => note,
                            });
                        }
                    }
                }
                outcomes.push(CoreWellOutcome {
                    well_name: well_name.to_string(),
                    rows: list.len(),
                    imported: if has_core_measurement { depth.len() } else { 0 },
                    set_name: Some(if has_core_measurement { set.clone() } else { desired_set.clone() }),
                    problem,
                });
            }
            Err(e) => outcomes.push(CoreWellOutcome {
                well_name: well_name.to_string(),
                rows: list.len(),
                imported: 0,
                set_name: None,
                problem: Some(e.to_string()),
            }),
        }
    };

    for (name, list) in &groups {
        // Normalized-name match against the project, LAS-attach rules: 1 → import,
        // 0 / many → report and skip.
        let norm = name.trim().to_uppercase();
        let ids: Vec<String> = {
            let mut stmt = match conn
                .prepare("SELECT well_id FROM wells WHERE upper(trim(well_name)) = ?1 ORDER BY well_id")
            {
                Ok(s) => s,
                Err(e) => return fail(e.to_string()),
            };
            match stmt
                .query_map(params![norm], |r| r.get::<_, String>(0))
                .and_then(|rows| rows.collect::<Result<Vec<_>, _>>())
            {
                Ok(v) => v,
                Err(e) => return fail(e.to_string()),
            }
        };
        match ids.len() {
            1 => store(&ids[0], name, list, &mut outcomes),
            0 => outcomes.push(CoreWellOutcome {
                well_name: name.clone(),
                rows: list.len(),
                imported: 0,
                set_name: None,
                problem: Some("no well of this name in the project".into()),
            }),
            n => outcomes.push(CoreWellOutcome {
                well_name: name.clone(),
                rows: list.len(),
                imported: 0,
                set_name: None,
                problem: Some(format!("{n} wells share this name — ambiguous, merge or delete duplicates first")),
            }),
        }
    }

    if !fallback_rows.is_empty() {
        match fallback_well_id {
            Some(wid) => {
                let name: String = conn
                    .query_row("SELECT well_name FROM wells WHERE well_id = ?1", params![wid], |r| r.get(0))
                    .unwrap_or_else(|_| wid.to_string());
                store(wid, &name, &fallback_rows, &mut outcomes);
            }
            None => {
                return fail("file has no well column and no well is selected — select a well or map a WELL column".into())
            }
        }
    }

    CoreTableImportResult {
        path: path.to_string(),
        rows_imported,
        wells_imported,
        outcomes,
        skipped_blank_well,
        extra_rows,
        extra_items: if extra_rows > 0 { extra_names } else { Vec::new() },
        precision,
        error: None,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ScalImportResult {
    pub path: String,
    pub rows: usize,
    /// Leverett-J fit over the imported points at the given lab IFT, when solvable —
    /// reported straight back to the import dialog so the user can carry SWH_A/SWH_B
    /// into the sw_height module.
    pub fit: Option<crate::satheight::LeverettFit>,
    /// The SCAL set these points landed in (auto-suffixed when the name was taken).
    pub set_name: Option<String>,
    /// What following the core depth record did, when it was asked for.
    #[serde(default)]
    pub note: Option<String>,
    pub error: Option<String>,
}

/// Parses a SCAL capillary-pressure CSV (flat/long shape), replaces the well's `scal_pc`
/// rows, and fits the Leverett-J function (Sw = A·J^B) over the points at `ift_lab`
/// (sigma·cosθ of the lab fluid system, dyn/cm — e.g. 72 air-brine, 367 air-mercury).
pub fn import_scal_csv(conn: &Connection, well_id: &str, path: &str, ift_lab: f64) -> ScalImportResult {
    import_scal_files(conn, well_id, &[path.to_string()], "long", "", ift_lab, None, false)
}

/// Multi-file, multi-format SCAL Pc import. Each file is parsed with `format` — "long"
/// (flat Pc/Sw CSV), "porous_plate" (Corelab-style wide table: pressure columns × plug
/// rows), "centrifuge" (per-plug key-value blocks + Pc/Sw tables), or "auto" to sniff
/// each file — so a set of single-plug centrifuge exports imports in one shot. The files
/// selected together form ONE delivery: their combined records land in the SCAL set
/// `set_name` (auto-suffixed if the well already carries that name, so a later report never
/// overwrites an earlier one), which becomes the well's live SCAL data, and the Leverett-J
/// function is fitted over all of them at `ift_lab`. `system` labels every stored point with
/// the lab fluid system ('air_brine', 'hg_air', ...; "" = not recorded) alongside `ift_lab`,
/// so later standardization (Thomeer, J-from-SCAL) knows which system each point was
/// measured in.
pub fn import_scal_files(
    conn: &Connection,
    well_id: &str,
    paths: &[String],
    format: &str,
    system: &str,
    ift_lab: f64,
    set_name: Option<&str>,
    follow_core: bool,
) -> ScalImportResult {
    let joined = paths.join("; ");
    let fail = |error: String| ScalImportResult {
        path: joined.clone(),
        rows: 0,
        fit: None,
        set_name: None,
        note: None,
        error: Some(error),
    };

    let exists: bool = conn
        .query_row("SELECT 1 FROM wells WHERE well_id = ?1", params![well_id], |_| Ok(true))
        .unwrap_or(false);
    if !exists {
        return fail(format!("unknown well '{well_id}'"));
    }
    if paths.is_empty() {
        return fail("no files selected".into());
    }

    let mut records: Vec<parsers::ScalPcRecord> = Vec::new();
    for path in paths {
        let base = path.rsplit(['/', '\\']).next().unwrap_or(path);
        let fmt = if format == "auto" {
            match parsers::sniff_scal_format(path) {
                Ok(f) => f,
                Err(e) => return fail(format!("{base}: {e}")),
            }
        } else {
            format
        };
        let parsed = match fmt {
            "long" => parsers::parse_scal_csv(path),
            "porous_plate" => parsers::parse_scal_wide_csv(path),
            "centrifuge" => parsers::parse_scal_centrifuge_csv(path),
            other => return fail(format!("unknown SCAL format '{other}'")),
        };
        match parsed {
            Ok(mut r) => records.append(&mut r),
            Err(e) => return fail(format!("{base} ({fmt}): {e}")),
        }
    }
    // A structurally-valid file can still yield zero points (header-only export, cells in
    // a format no rule parses). Refuse the replace-write then — otherwise a degenerate
    // re-import would silently DELETE the well's existing SCAL dataset.
    if records.is_empty() {
        return fail(
            "no Pc/Sw data rows parsed from the selected file(s) — nothing was imported and the well's existing SCAL points are untouched (check the file format choice)".into(),
        );
    }

    // SCAL plugs are core plugs, so their depths are the core report's depths and move with the
    // core. A record with no depth at all is left alone — there is nothing to correct.
    let core_pairs: Vec<(f32, f32)> = if follow_core {
        db::core_depth_pairs(conn, well_id).unwrap_or_default()
    } else {
        Vec::new()
    };
    let mut outside_core = 0usize;
    let mut mapped_any = 0usize;

    let sys: Option<String> = if system.trim().is_empty() { None } else { Some(system.trim().to_string()) };
    let rows: Vec<db::ScalPcRow> = records
        .iter()
        .map(|r| db::ScalPcRow {
            sample_no: r.sample_no,
            depth: match r.depth {
                Some(d) if !core_pairs.is_empty() && d.is_finite() => {
                    let (m, ex) = db::map_core_depth(&core_pairs, d);
                    if ex {
                        outside_core += 1;
                    }
                    mapped_any += 1;
                    Some(m)
                }
                other => other,
            },
            perm: r.perm,
            poro: r.poro,
            pc: r.pc,
            sw: r.sw,
            system: sys.clone(),
            ift: Some(ift_lab as f32),
        })
        .collect();
    let desired = set_name
        .map(|s| s.trim().to_uppercase().replace(' ', "_"))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "SCAL".to_string());
    let set = match db::resolve_scal_set_name(conn, well_id, &desired) {
        Ok(s) => s,
        Err(e) => return fail(e.to_string()),
    };
    if let Err(e) = db::insert_scal_pc(conn, well_id, &set, Some(&joined), &rows) {
        return fail(e.to_string());
    }
    if follow_core {
        let _ = db::mark_scal_set_on_core(conn, well_id, &set);
    }

    let points: Vec<crate::satheight::ScalPoint> = records
        .iter()
        .map(|r| crate::satheight::ScalPoint { pc: r.pc, sw: r.sw, perm: r.perm, poro: r.poro })
        .collect();
    let fit = crate::satheight::fit_leverett_j(&points, ift_lab);
    let note = if !follow_core {
        None
    } else if core_pairs.is_empty() {
        Some("no core to follow, depths used as written".into())
    } else if core_pairs.iter().all(|(o, d)| (o - d).abs() <= 1e-4) {
        Some("core has not been shifted, so depths are unchanged".into())
    } else if mapped_any == 0 {
        Some("these points carry no depth, so there was nothing to place".into())
    } else if outside_core > 0 {
        Some(format!(
            "placed from the core depth record; {outside_core} point(s) fell outside the cored \
             interval and were placed by holding the nearest correction"
        ))
    } else {
        Some("placed from the core depth record".into())
    };
    ScalImportResult { path: joined, rows: rows.len(), fit, set_name: Some(set), note, error: None }
}

#[derive(Debug, Clone, Serialize)]
pub struct TopsImportResult {
    pub path: String,
    pub tops_written: usize,
    pub wells_matched: usize,
    /// Well names in the file that matched nothing in the project (rows skipped).
    pub unmatched_wells: Vec<String>,
    pub error: Option<String>,
}

/// Imports formation tops from a CSV/TXT file. Files with a WELL column update every
/// matching well (name match, case-insensitive); files without one need
/// `default_well_id` (the selected well). Tops upsert by (well, name) — re-import
/// updates depths, existing colors are kept.
pub fn import_tops_file(conn: &Connection, default_well_id: Option<&str>, path: &str) -> TopsImportResult {
    let fail = |e: String| TopsImportResult {
        path: path.to_string(),
        tops_written: 0,
        wells_matched: 0,
        unmatched_wells: vec![],
        error: Some(e),
    };
    let (has_well_column, records) = match parsers::parse_tops_file(path) {
        Ok(r) => r,
        Err(e) => return fail(e.to_string()),
    };

    // Project well-name → id map (upper-trimmed).
    let mut name_to_id: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    {
        let mut stmt = match conn.prepare("SELECT well_name, well_id FROM wells") {
            Ok(s) => s,
            Err(e) => return fail(e.to_string()),
        };
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        });
        match rows {
            Ok(rows) => {
                for r in rows.flatten() {
                    name_to_id.insert(r.0.trim().to_uppercase(), r.1);
                }
            }
            Err(e) => return fail(e.to_string()),
        }
    }

    // All-or-nothing: a mid-file DB error must not leave some tops written and others not
    // (which would otherwise report tops_written=0 while rows are already persisted). Mirrors
    // import_locations_file below.
    if let Err(e) = conn.execute_batch("BEGIN") {
        return fail(e.to_string());
    }
    let mut written = 0usize;
    let mut wells_hit: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut unmatched: Vec<String> = Vec::new();
    let mut blank_rows = 0usize;
    for rec in &records {
        let well_id = match &rec.well {
            Some(name) => match name_to_id.get(&name.trim().to_uppercase()) {
                Some(id) => id.clone(),
                None => {
                    let label = name.trim().to_string();
                    if !unmatched.contains(&label) {
                        unmatched.push(label);
                    }
                    continue;
                }
            },
            // File HAS a WELL column but this row's cell is blank/ragged — skip it; misrouting a
            // blank-cell top to the selected well would silently attach it to an unrelated well.
            None if has_well_column => {
                blank_rows += 1;
                continue;
            }
            // Genuinely column-less (single-well) file → the dialog's selected well.
            None => match default_well_id {
                Some(id) => id.to_string(),
                None => {
                    let _ = conn.execute_batch("ROLLBACK");
                    return fail("file has no WELL column — select a well first".into());
                }
            },
        };
        match db::upsert_top(conn, &well_id, &rec.top_name, rec.depth, None) {
            Ok(()) => {
                written += 1;
                wells_hit.insert(well_id);
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                return fail(e.to_string());
            }
        }
    }
    if let Err(e) = conn.execute_batch("COMMIT") {
        let _ = conn.execute_batch("ROLLBACK");
        return fail(e.to_string());
    }
    // Surface dropped blank-WELL rows so the skip is never silent.
    if blank_rows > 0 {
        unmatched.push(format!("{blank_rows} blank-WELL row(s)"));
    }
    TopsImportResult {
        path: path.to_string(),
        tops_written: written,
        wells_matched: wells_hit.len(),
        unmatched_wells: unmatched,
        error: None,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LocationsImportResult {
    pub path: String,
    pub wells_located: usize,
    /// Well names in the file that matched nothing in the project (rows skipped).
    pub unmatched_wells: Vec<String>,
    pub error: Option<String>,
}

/// Imports well surface locations from a CSV/TXT file. Files with a WELL column locate
/// every matching well (name match, case-insensitive); files without one locate
/// `default_well_id` (the selected well). `default_zone` fills the UTM zone for rows that
/// carry no ZONE column value (the dialog's chosen zone). Re-import overwrites a well's
/// previous location.
pub fn import_locations_file(
    conn: &Connection,
    default_well_id: Option<&str>,
    default_zone: Option<&str>,
    path: &str,
) -> LocationsImportResult {
    let fail = |e: String| LocationsImportResult {
        path: path.to_string(),
        wells_located: 0,
        unmatched_wells: vec![],
        error: Some(e),
    };
    let (has_well_column, records) = match parsers::parse_locations_file(path) {
        Ok(r) => r,
        Err(e) => return fail(e.to_string()),
    };

    // Project well-name → id map (upper-trimmed), same convention as the tops importer.
    let mut name_to_id: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    {
        let mut stmt = match conn.prepare("SELECT well_name, well_id FROM wells") {
            Ok(s) => s,
            Err(e) => return fail(e.to_string()),
        };
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)));
        match rows {
            Ok(rows) => {
                for r in rows.flatten() {
                    name_to_id.insert(r.0.trim().to_uppercase(), r.1);
                }
            }
            Err(e) => return fail(e.to_string()),
        }
    }

    // All-or-nothing: a mid-file DB error must not leave some wells relocated and others not
    // (which would otherwise report wells_located = 0 while rows are already persisted).
    if let Err(e) = conn.execute_batch("BEGIN") {
        return fail(e.to_string());
    }
    let mut located: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut unmatched: Vec<String> = Vec::new();
    let mut blank_rows = 0usize;
    for rec in &records {
        let well_id = match &rec.well {
            Some(name) => match name_to_id.get(&name.trim().to_uppercase()) {
                Some(id) => id.clone(),
                None => {
                    let label = name.trim().to_string();
                    if !unmatched.contains(&label) {
                        unmatched.push(label);
                    }
                    continue;
                }
            },
            // File HAS a WELL column but this row's cell is blank/ragged — a dropped row, not
            // a single-well file. Skip it; misrouting it to the selected well would silently
            // overwrite an unrelated well's real surface location.
            None if has_well_column => {
                blank_rows += 1;
                continue;
            }
            // Genuinely column-less (single-well) file → the dialog's selected well.
            None => match default_well_id {
                Some(id) => id.to_string(),
                None => {
                    let _ = conn.execute_batch("ROLLBACK");
                    return fail("file has no WELL column — select a well first".into());
                }
            },
        };
        let zone = rec.zone.as_deref().or(default_zone);
        if let Err(e) = db::set_well_location(conn, &well_id, Some(rec.x), Some(rec.y), zone) {
            let _ = conn.execute_batch("ROLLBACK");
            return fail(e.to_string());
        }
        located.insert(well_id);
    }
    if let Err(e) = conn.execute_batch("COMMIT") {
        let _ = conn.execute_batch("ROLLBACK");
        return fail(e.to_string());
    }
    // Surface dropped blank-WELL rows so the skip is never silent.
    if blank_rows > 0 {
        unmatched.push(format!("{blank_rows} blank-WELL row(s)"));
    }
    LocationsImportResult {
        path: path.to_string(),
        wells_located: located.len(),
        unmatched_wells: unmatched,
        error: None,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AuxImportResult {
    pub path: String,
    pub dataset: String,
    pub rows: usize,
    /// Value columns found in the file (QUARTZ, STATUS, …).
    pub items: Vec<String>,
    /// Wells that received rows (1 for a single-well file).
    pub wells_imported: usize,
    /// The set name(s) the delivery actually landed in — more than one when some wells
    /// already carried that name and theirs was suffixed.
    pub sets: Vec<String>,
    /// Routing story for a multi-well file: unmatched/ambiguous names, blank-well rows.
    pub notes: Option<String>,
    pub error: Option<String>,
}

/// Imports a tops-style dataset (petrography / XRD / perforations), replacing each
/// receiving well's previous rows of the same dataset. Numeric cells land in value_num,
/// everything else in value_text.
///
/// Routing (T-IMP-11, same rules as tops/core): a file WITH a well column routes every
/// row by its cell's normalized name — exactly-one-match imports, unmatched/ambiguous
/// names are reported and skipped, blank cells are skipped (never misrouted). A file
/// WITHOUT a well column binds wholly to `well_id` (the selected well).
///
/// `set_name` versions the DELIVERY within the dataset, exactly as core sets do: a second
/// XRD (or CEC, oil show, …) delivery lands beside the first, auto-suffixed per well, and
/// becomes the live one for that dataset. Nothing is ever overwritten.
/// The well's core depth record, or an empty slice when the caller did not ask to follow it — or
/// asked but the well has no core. **Not following is never silent**: a user who ticked the box
/// and got raw depths would have no way to tell, and the samples would be wrong by exactly the
/// amount the core was corrected by.
fn core_record(
    conn: &Connection,
    well_id: &str,
    follow_core: bool,
    label: &str,
    notes: &mut Vec<String>,
) -> Vec<(f32, f32)> {
    if !follow_core {
        return Vec::new();
    }
    match db::core_depth_pairs(conn, well_id) {
        Ok(p) if p.is_empty() => {
            notes.push(format!("{label}: no core to follow, depths used as written"));
            Vec::new()
        }
        Ok(p) => p,
        Err(e) => {
            notes.push(format!("{label}: could not read the core depth record ({e}), depths used as written"));
            Vec::new()
        }
    }
}

/// Says what the mapping did. A core that has never been shifted maps every depth to itself, and
/// saying so beats silence — it tells the user the box worked and simply had nothing to correct.
fn note_mapping(notes: &mut Vec<String>, label: &str, pairs: &[(f32, f32)], rows: usize, outside: usize) {
    if pairs.is_empty() || rows == 0 {
        return;
    }
    let shifted = pairs.iter().any(|(o, d)| (o - d).abs() > 1e-4);
    if !shifted {
        notes.push(format!("{label}: core has not been shifted, so depths are unchanged"));
        return;
    }
    if outside > 0 {
        notes.push(format!(
            "{label}: placed from the core depth record; {outside} sample(s) fell outside the cored \
             interval and were placed by holding the nearest correction"
        ));
    } else {
        notes.push(format!("{label}: placed from the core depth record"));
    }
}

pub fn import_aux_file(
    conn: &Connection,
    well_id: &str,
    dataset: &str,
    path: &str,
    set_name: Option<&str>,
    follow_core: bool,
) -> AuxImportResult {
    let fail = |e: String| AuxImportResult {
        path: path.to_string(),
        dataset: dataset.to_string(),
        rows: 0,
        items: vec![],
        wells_imported: 0,
        sets: vec![],
        notes: None,
        error: Some(e),
    };
    let dataset = dataset.trim().to_uppercase();
    if dataset.is_empty() {
        return fail("dataset name is empty".into());
    }
    let desired_set = set_name
        .map(|s| s.trim().to_uppercase().replace(' ', "_"))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "RAW".to_string());

    let data = match parsers::parse_interval_file(path) {
        Ok(d) => d,
        Err(e) => return fail(e.to_string()),
    };

    // One AuxRow batch per routing target. `None` key = the selected-well fallback
    // (only used when the file has no well column).
    //
    // `follow_core` places the file's depths through the target well's core depth record: a
    // laboratory writes the depths from the original core report, and if that core has since been
    // registered against the log those depths are stale by however far the core moved. The record
    // is per WELL, so the mapping is resolved inside this closure rather than once for the file.
    //
    // An interval is placed by its TOP and its base takes the same offset — the same rule the
    // barrel shifts use. Mapping the two ends independently could invert a thin sample where the
    // correction changes steeply across a barrel boundary, and a sample that measured 20 cm of
    // rock still measured 20 cm of rock.
    let to_aux_rows = |idx: &[usize], pairs: &[(f32, f32)], outside: &mut usize| -> Vec<db::AuxRow> {
        let mut rows: Vec<db::AuxRow> = Vec::new();
        for &i in idx {
            let (raw_top, raw_base, values) = &data.rows[i];
            let (top, base) = if pairs.is_empty() {
                (*raw_top, *raw_base)
            } else {
                let (mapped, ex) = db::map_core_depth(pairs, *raw_top);
                if ex {
                    *outside += 1;
                }
                let offset = mapped - *raw_top;
                (mapped, raw_base.map(|b| b + offset))
            };
            let (top, base) = (&top, &base);
            for (item, raw) in data.items.iter().zip(values) {
                let Some(raw) = raw else { continue };
                let num = raw.replace(',', ".").parse::<f32>().ok();
                rows.push(db::AuxRow {
                    dataset: dataset.clone(),
                    depth_top: *top,
                    depth_base: *base,
                    item: item.clone(),
                    value_num: num,
                    value_text: if num.is_some() { None } else { Some(raw.clone()) },
                });
            }
        }
        rows
    };

    let mut notes: Vec<String> = Vec::new();
    let mut rows_written = 0usize;
    let mut wells_imported = 0usize;
    let mut sets_used: std::collections::BTreeSet<String> = Default::default();

    if data.has_well_column {
        // Group row indices by well cell, keeping file order.
        let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
        let mut blank = 0usize;
        for (i, w) in data.wells.iter().enumerate() {
            match w {
                Some(name) => match groups.iter_mut().find(|(n, _)| n == name) {
                    Some((_, list)) => list.push(i),
                    None => groups.push((name.clone(), vec![i])),
                },
                None => blank += 1,
            }
        }
        if blank > 0 {
            notes.push(format!("{blank} row(s) with a blank well cell skipped"));
        }
        let mut unmatched: Vec<String> = Vec::new();
        for (name, idx) in &groups {
            let norm = name.trim().to_uppercase();
            let ids: Vec<String> = {
                let mut stmt = match conn
                    .prepare("SELECT well_id FROM wells WHERE upper(trim(well_name)) = ?1 ORDER BY well_id")
                {
                    Ok(s) => s,
                    Err(e) => return fail(e.to_string()),
                };
                match stmt
                    .query_map(params![norm], |r| r.get::<_, String>(0))
                    .and_then(|rows| rows.collect::<Result<Vec<_>, _>>())
                {
                    Ok(v) => v,
                    Err(e) => return fail(e.to_string()),
                }
            };
            match ids.len() {
                1 => {
                    let pairs = core_record(conn, &ids[0], follow_core, name, &mut notes);
                    let mut outside = 0usize;
                    let rows = to_aux_rows(idx, &pairs, &mut outside);
                    note_mapping(&mut notes, name, &pairs, rows.len(), outside);
                    // Per well, like core sets: a name free on one well may be taken on another.
                    let set = match db::resolve_aux_set_name(conn, &ids[0], &dataset, &desired_set) {
                        Ok(s) => s,
                        Err(e) => {
                            notes.push(format!("{name}: {e}"));
                            continue;
                        }
                    };
                    match db::insert_aux_data(conn, &ids[0], &dataset, &set, Some(path), &rows) {
                        Ok(()) => {
                            rows_written += rows.len();
                            wells_imported += 1;
                            // Record the depth basis, so a later core registration knows whether
                            // this delivery should move with the core.
                            if follow_core {
                                let _ = db::mark_aux_set_on_core(conn, &ids[0], &dataset, &set);
                            }
                            sets_used.insert(set);
                        }
                        Err(e) => notes.push(format!("{name}: {e}")),
                    }
                }
                0 => unmatched.push(name.clone()),
                n => notes.push(format!("{name}: {n} wells share this name — ambiguous, skipped")),
            }
        }
        if !unmatched.is_empty() {
            notes.push(format!(
                "{} name(s) not in the project, skipped: {}",
                unmatched.len(),
                unmatched.join(", ")
            ));
        }
        if wells_imported == 0 {
            return fail(format!(
                "no rows imported — none of the file's well names matched the project ({})",
                notes.join("; ")
            ));
        }
    } else {
        let exists: bool = conn
            .query_row("SELECT 1 FROM wells WHERE well_id = ?1", params![well_id], |_| Ok(true))
            .unwrap_or(false);
        if !exists {
            return fail(format!("unknown well '{well_id}'"));
        }
        let idx: Vec<usize> = (0..data.rows.len()).collect();
        let pairs = core_record(conn, well_id, follow_core, "this well", &mut notes);
        let mut outside = 0usize;
        let rows = to_aux_rows(&idx, &pairs, &mut outside);
        note_mapping(&mut notes, "this well", &pairs, rows.len(), outside);
        let set = match db::resolve_aux_set_name(conn, well_id, &dataset, &desired_set) {
            Ok(s) => s,
            Err(e) => return fail(e.to_string()),
        };
        match db::insert_aux_data(conn, well_id, &dataset, &set, Some(path), &rows) {
            Ok(()) => {
                rows_written = rows.len();
                wells_imported = 1;
                if follow_core {
                    let _ = db::mark_aux_set_on_core(conn, well_id, &dataset, &set);
                }
                sets_used.insert(set);
            }
            Err(e) => return fail(e.to_string()),
        }
    }

    AuxImportResult {
        path: path.to_string(),
        dataset,
        rows: rows_written,
        items: data.items,
        wells_imported,
        sets: sets_used.into_iter().collect(),
        notes: (!notes.is_empty()).then(|| notes.join("; ")),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Core import round-trip + the SET discipline (T-IMP-08): a second delivery never
    /// overwrites the first, exactly one set is live, and every reader follows it.
    #[test]
    fn core_import_roundtrip_and_replace() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "SANDI-1", None, None, None).unwrap();
        let ids = well_id.to_string();

        let path = std::env::temp_dir().join("arshilla_core_roundtrip.csv");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"DEPTH,CPOR (%),KAIR (mD)\n2001.0,22.5,150\n2002.0,18.0,20\n").unwrap();
        drop(f);
        let csv = path.to_str().unwrap();

        let result = import_core_csv(&conn, &ids, csv);
        assert!(result.error.is_none(), "{:?}", result.error);
        assert_eq!(result.rows, 2);

        let (n, cpor0): (i64, f32) = conn
            .query_row(
                "SELECT COUNT(*), MIN(cpor) FROM core_data WHERE well_id = ?1",
                params![ids],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(n, 2);
        assert!((cpor0 - 0.18).abs() < 1e-6, "percent porosity must land as v/v, got {cpor0}");

        // Re-import KEEPS the first delivery and lands beside it as a second SET
        // (T-IMP-08): the old behaviour silently overwrote plugs the lab had sent once.
        let again = import_core_csv(&conn, &ids, csv);
        assert!(again.error.is_none());
        let sets = db::list_core_sets(&conn, &ids).unwrap();
        assert_eq!(sets.len(), 2, "second delivery is a second set: {sets:?}");
        assert_eq!(sets[0].set_name, "CORE_1", "the newest import is active and listed first");
        assert!(sets[0].active);
        assert!(sets.iter().all(|s| s.rows == 2));
        // …but a READER still sees one delivery's worth of plugs, never both merged.
        assert_eq!(db::get_core_plugs(&conn, &ids).unwrap().len(), 2, "readers see the ACTIVE set only");

        // Switching back makes the first delivery live again.
        db::set_active_core_set(&conn, &ids, "CORE").unwrap();
        assert_eq!(db::get_core_plugs(&conn, &ids).unwrap().len(), 2);
        assert!(db::list_core_sets(&conn, &ids).unwrap().iter().filter(|s| s.active).count() == 1);

        // Unknown well is rejected cleanly.
        let bad = import_core_csv(&conn, "no-such-well", csv);
        assert!(bad.error.is_some());

        // Core-to-log shift moves the ACTIVE set's plugs by the same delta and reverses
        // exactly; the other delivery keeps its own depths.
        let shifted = db::shift_core_depths(&mut conn, &ids, 2.5, &Default::default(), &Default::default()).unwrap();
        assert_eq!(shifted.plugs, 2);
        let min_depth: f32 = conn
            .query_row(
                "SELECT MIN(depth) FROM core_data WHERE well_id = ?1 AND set_name = 'CORE'",
                params![ids],
                |r| r.get(0),
            )
            .unwrap();
        assert!((min_depth - 2003.5).abs() < 1e-4);
        let untouched: f32 = conn
            .query_row(
                "SELECT MIN(depth) FROM core_data WHERE well_id = ?1 AND set_name = 'CORE_1'",
                params![ids],
                |r| r.get(0),
            )
            .unwrap();
        assert!((untouched - 2001.0).abs() < 1e-4, "the inactive delivery must not move");
        db::shift_core_depths(&mut conn, &ids, -2.5, &Default::default(), &Default::default()).unwrap();
        let min_depth: f32 = conn
            .query_row(
                "SELECT MIN(depth) FROM core_data WHERE well_id = ?1 AND set_name = 'CORE'",
                params![ids],
                |r| r.get(0),
            )
            .unwrap();
        assert!((min_depth - 2001.0).abs() < 1e-4);

        // Deleting the live set hands over to the survivor — never leaves plugs unreadable.
        db::delete_core_set(&conn, &ids, "CORE").unwrap();
        let sets = db::list_core_sets(&conn, &ids).unwrap();
        assert_eq!(sets.len(), 1);
        assert!(sets[0].active && sets[0].set_name == "CORE_1");
        assert_eq!(db::get_core_plugs(&conn, &ids).unwrap().len(), 2);
        std::fs::remove_file(&path).ok();
    }

    /// A second LAS import of a well whose name already exists still creates a separate record
    /// (auto-merge needs a confirmation flow) but must surface a warning, not silently fragment.
    #[test]
    fn las_import_warns_on_duplicate_well_name() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();

        let cols = || CurveColumns {
            las_version: None,
            unread_sections: Vec::new(),
            text_encoding: "test fixture".into(),
            depth_unit: Some("M".into()),
            declared_step: None,
            depth: vec![1000.0, 1000.5, 1001.0],
            gr: vec![40.0, 45.0, 50.0],
            res: vec![f32::NAN; 3],
            nphi: vec![f32::NAN; 3],
            rhob: vec![f32::NAN; 3],
            dt: vec![f32::NAN; 3],
            sp: vec![f32::NAN; 3],
            alias_decisions: Vec::new(),
            index_resolution: None,
            unit_designations: Vec::new(),
        };

        // First import: a fresh well, no duplicate warning.
        let r1 = insert_parsed_well(&conn, "a.las".into(), "DUP-1".into(), cols(), &LasImportOptions::default());
        assert!(r1.error.is_none(), "{:?}", r1.error);
        assert!(
            r1.warning.as_deref().map_or(true, |w| !w.contains("already exists")),
            "first import must not warn about a duplicate, got {:?}",
            r1.warning
        );

        // Second import of the SAME well name (normalized: lower-case + trailing space): a
        // separate record, but a duplicate warning.
        let r2 = insert_parsed_well(&conn, "b.las".into(), "dup-1  ".into(), cols(), &LasImportOptions::default());
        assert!(r2.error.is_none(), "{:?}", r2.error);
        assert!(
            r2.warning.as_deref().unwrap_or("").contains("already exists"),
            "re-import of a same-named (normalized) well must warn, got {:?}",
            r2.warning
        );
        assert_ne!(r1.well_id, r2.well_id, "still two distinct records (no auto-merge)");
    }

    /// SB-DIO-009 / SB-DIO-T14. The ordered NPHI aliases and finite-coverage
    /// tie-break are specified in `docs/PRD_v2/21_data-io.md` §5.3.
    #[test]
    fn the_alias_result_names_the_chosen_and_passed_over_columns_with_both_coverage_counts() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let path = std::env::temp_dir().join("sandibumi_alias_decision_coverage.las");
        std::fs::write(
            &path,
            "~VERSION\nVERS. 2.0 :\n~WELL\nNULL. -999.25 :\nWELL. ALIASES :\n\
             ~CURVE\nDEPT.M :\nGR.API :\nNPHIED.V/V :\nNPHI_LS.V/V :\n~ASCII\n\
             1000.0 50.0 -999.25 0.20\n1000.5 51.0 -999.25 0.21\n",
        )
        .unwrap();
        let result = import_las_files(
            &conn,
            &[path.to_str().unwrap().to_string()],
            None,
        )
        .remove(0);
        std::fs::remove_file(&path).ok();

        assert!(result.error.is_none(), "the fixture must import: {:?}", result.error);
        assert_eq!(
            result.alias_decisions.len(),
            1,
            "the single GR match is not reported as a choice, while the two NPHI matches are"
        );
        let decision = &result.alias_decisions[0];
        assert_eq!(decision.target, "NPHI");
        assert_eq!(decision.chosen, "NPHI_LS");
        assert_eq!(
            decision.candidates,
            vec![
                parsers::AliasCandidateCoverage {
                    mnemonic: "NPHIED".into(),
                    finite_samples: 0,
                    chosen: false,
                },
                parsers::AliasCandidateCoverage {
                    mnemonic: "NPHI_LS".into(),
                    finite_samples: 2,
                    chosen: true,
                },
            ],
            "the per-file result carries both the chosen and passed-over coverage"
        );
    }

    /// SB-DIO-030 / SB-DIO-T46. `SGR` is the source identity and `GR` is the
    /// applied standard target. The generic store must preserve the former while
    /// the standard store exposes the latter, and the exact parser table row must
    /// make the rename auditable even though no second alias competed.
    #[test]
    fn an_alias_rename_keeps_both_names_and_records_the_table_entry_that_fired() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let path = std::env::temp_dir().join(format!(
            "sandibumi-dio030-{}-{}.las",
            std::process::id(),
            Uuid::new_v4()
        ));
        std::fs::write(
            &path,
            "~VERSION\nVERS. 2.0 :\n~WELL\nWELL. DIO-030 :\n\
             ~CURVE\nDEPT.M : depth\nSGR.GAPI : spectral gamma\n\
             ~ASCII\n1000.0 71.0\n1000.5 72.0\n",
        )
        .unwrap();
        let result = import_las_files(
            &conn,
            &[path.to_string_lossy().to_string()],
            None,
        )
        .remove(0);
        std::fs::remove_file(&path).ok();

        assert!(result.error.is_none(), "fixture import failed: {:?}", result.error);
        let rename = result.alias_decisions.iter().find(|decision| decision.chosen == "SGR").unwrap();
        assert_eq!(rename.target, "GR");
        assert_eq!(rename.candidates.len(), 1, "a rename is reported even without competition");
        assert_eq!(rename.table_entry.as_deref(), Some("GR_ALIASES: SGR -> GR"));
        assert!(
            result.warning.as_deref().is_some_and(|warning| {
                warning.contains("SGR")
                    && warning.contains("GR")
                    && warning.contains("GR_ALIASES: SGR -> GR")
            }),
            "the rename must be displayed in the import note: {:?}",
            result.warning
        );

        let well_id = result.well_id.unwrap();
        let standard_gr: f32 = conn
            .query_row(
                "SELECT gr FROM standard_curves WHERE well_id = ?1 ORDER BY depth LIMIT 1",
                params![&well_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(standard_gr, 71.0, "the applied GR target receives the SGR samples");
        let source = db::list_generic_curve_catalog(&conn, &well_id)
            .unwrap()
            .into_iter()
            .find(|curve| curve.mnemonic == "SGR")
            .expect("original SGR identity retained");
        assert_eq!(source.family.as_deref(), Some("GR"), "the applied family remains visible beside SGR");
    }

    /// SB-DIO-010 / SB-DIO-T15..T16. Geolog's per-column `REFERENCE | LOG`
    /// declaration and LAS's first-column guarantee are cited in chapter §5.3.
    #[test]
    fn a_structural_index_wins_and_every_resolution_records_the_mechanism_that_fired() {
        let headers = vec!["GR".to_string(), "SCD".to_string()];
        let classes = vec!["LOG".to_string(), "REFERENCE".to_string()];
        let structural = parsers::resolve_index_column(
            &headers,
            Some(&classes),
            &["DEPTH"],
            Some(0),
            None,
        )
        .unwrap();
        assert_eq!(structural.column, 1, "the non-first structural declaration wins");
        assert_eq!(
            structural.mechanism,
            parsers::IndexResolutionMechanism::StructuralDeclaration
        );

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let path = std::env::temp_dir().join("sandibumi_positional_index_audit.las");
        std::fs::write(
            &path,
            "~VERSION\nVERS. 2.0 :\n~WELL\nWELL. POSITIONAL :\n\
             ~CURVE\nXREF.M : index\nMD.M : auxiliary track\nGR.API :\n~ASCII\n\
             1000.0 3000.0 50.0\n1000.5 3000.5 51.0\n",
        )
        .unwrap();
        let result = import_las_files(&conn, &[path.to_str().unwrap().to_string()], None).remove(0);
        std::fs::remove_file(&path).ok();
        assert!(result.error.is_none(), "the LAS fixture must import: {:?}", result.error);
        let positional = result.index_resolution.expect("the result must carry its index decision");
        assert_eq!(positional.column, 0, "a second-column MD track cannot steal the LAS index");
        assert_eq!(positional.mnemonic, "XREF");
        assert_eq!(
            positional.mechanism,
            parsers::IndexResolutionMechanism::PositionalGuarantee
        );
    }

    /// SB-DIO-012 / SB-DIO-T18. The mandatory strictly-increasing constraint is
    /// cited from Techlog's ASCII reference control in chapter §5.3.
    #[test]
    fn a_non_increasing_index_is_blocked_at_the_reported_row_until_the_user_accepts_it() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let path = std::env::temp_dir().join("sandibumi_non_increasing_row_400.las");
        let mut samples = String::new();
        for row in 1..=400 {
            let depth = if row == 400 { 1397.5 } else { 1000.0 + (row - 1) as f32 };
            samples.push_str(&format!("{depth:.1} {row}.0\n"));
        }
        std::fs::write(
            &path,
            format!(
                "~VERSION\nVERS. 2.0 :\n~WELL\nWELL. ORDER :\n\
                 ~CURVE\nDEPT.M :\nGR.API :\n~ASCII\n{samples}"
            ),
        )
        .unwrap();

        let blocked = import_las_files_with(
            &conn,
            &[path.to_str().unwrap().to_string()],
            None,
            &LasImportOptions::default(),
        )
        .remove(0);
        assert!(
            blocked.error.as_deref().is_some_and(|error| {
                error.contains("data row 400") && error.contains("user decision")
            }),
            "the first decrease and required decision must be named: {:?}",
            blocked.error
        );
        let before: i64 = conn.query_row("SELECT COUNT(*) FROM wells", [], |row| row.get(0)).unwrap();
        assert_eq!(before, 0, "a blocked index cannot commit a well");

        let accepted = import_las_files_with(
            &conn,
            &[path.to_str().unwrap().to_string()],
            None,
            &LasImportOptions {
                non_monotonic_index: Some(NonMonotonicIndexDecision::AcceptAsDelivered),
                ..LasImportOptions::default()
            },
        )
        .remove(0);
        std::fs::remove_file(&path).ok();
        assert!(accepted.error.is_none(), "the explicit decision permits commit: {:?}", accepted.error);
        assert!(
            accepted.warning.as_deref().is_some_and(|warning| warning.contains("data row 400")),
            "the accepted conflict remains in the audit result: {:?}",
            accepted.warning
        );
    }

    /// SB-DIO-013 / SB-DIO-T19. Techlog's mandatory reference designation for
    /// a table with no structural or name resolution is cited in chapter §5.3.
    #[test]
    fn a_delimited_table_without_an_index_name_commits_nothing_until_the_user_designates_one() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = Uuid::new_v4();
        db::insert_well(&conn, well, "DESIGNATE", None, None, None).unwrap();
        let well_id = well.to_string();
        let path = std::env::temp_dir().join("sandibumi_designated_core_index.csv");
        std::fs::write(&path, "SAMPLE,CPOR\n1000.0,18.0\n1000.5,19.0\n").unwrap();

        let blocked = import_core_csv(&conn, &well_id, path.to_str().unwrap());
        assert!(
            blocked.error.as_deref().is_some_and(|error| {
                error.contains("user designation is required")
            }),
            "no positional column may be guessed: {:?}",
            blocked.error
        );
        let before: i64 = conn
            .query_row("SELECT COUNT(*) FROM core_data WHERE well_id = ?1", params![well_id], |row| row.get(0))
            .unwrap();
        assert_eq!(before, 0, "the undecided table commits no rows");

        let imported = import_core_csv_with_depth_column(
            &conn,
            &well_id,
            path.to_str().unwrap(),
            Some(0),
        );
        std::fs::remove_file(&path).ok();
        assert!(imported.error.is_none(), "the explicit designation imports: {:?}", imported.error);
        assert_eq!(imported.rows, 2);
        let resolution = imported.index_resolution.expect("the designation is recorded");
        assert_eq!(resolution.column, 0);
        assert_eq!(resolution.mnemonic, "SAMPLE");
        assert_eq!(
            resolution.mechanism,
            parsers::IndexResolutionMechanism::UserDesignation
        );
    }

    /// SB-DIO-050 / T70. No read-side STEP tolerance is cited in §5, so this pins the
    /// exact declared-versus-observed disagreement without introducing one. The matching
    /// control prevents an implementation that warns on every file carrying STEP.
    #[test]
    fn a_declared_step_that_disagrees_with_actual_spacing_is_flagged_as_possibly_regridded_and_a_matching_step_is_not() {
        let import = |tag: &str, step: &str| {
            let conn = Connection::open_in_memory().unwrap();
            db::create_schema(&conn).unwrap();
            let path = std::env::temp_dir().join(format!("sandibumi-step-{tag}.las"));
            std::fs::write(
                &path,
                format!(
                    "~VERSION\nVERS. 2.0 :\nWRAP. NO :\n~WELL\nSTEP.M {step} : declared step\nNULL. -999.25 :\nWELL. STEP-{tag} :\n~CURVE\nDEPT.M : depth\nGR.GAPI : gamma\n~ASCII\n1000 50\n1001 51\n1002 52\n"
                ),
            )
            .unwrap();
            let result = import_las_files(&conn, &[path.to_str().unwrap().to_string()], None).remove(0);
            std::fs::remove_file(&path).ok();
            result
        };

        let mismatch = import("MISMATCH", "0.5");
        assert!(mismatch.error.is_none(), "a re-grid flag is a warning, not a refusal: {:?}", mismatch.error);
        let warning = mismatch.warning.as_deref().unwrap_or("");
        assert!(warning.contains("possibly re-gridded"), "the risk must be named: {warning}");
        assert!(warning.contains("declared STEP 0.5"), "the file's declaration must be named: {warning}");
        assert!(warning.contains("actual spacing 1"), "the observed spacing must be named: {warning}");
        assert!(warning.contains("data rows 1 and 2"), "the first disagreement must be locatable: {warning}");

        let matching = import("MATCHING", "1.0");
        assert!(matching.error.is_none(), "the matching control must import: {:?}", matching.error);
        assert!(
            !matching.warning.as_deref().unwrap_or("").contains("possibly re-gridded"),
            "a declared step matching every interval must not be flagged: {:?}",
            matching.warning
        );
    }

    /// SB-DIO-020 / SB-DIO-T32..T33. The four policy names are the complete
    /// declared set in chapter §4.5; none is a default.
    #[test]
    fn duplicate_depths_wait_for_a_declared_policy_and_report_the_count_for_each_resolution() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let path = std::env::temp_dir().join("sandibumi_three_repeated_depths.las");
        std::fs::write(
            &path,
            "~VERSION\nVERS. 2.0 :\n~WELL\nWELL. DUPLICATES :\n\
             ~CURVE\nDEPT.M :\nGR.API :\n~ASCII\n\
             1000.0 10.0\n1000.0 20.0\n1000.0 30.0\n1000.0 40.0\n1001.0 50.0\n",
        )
        .unwrap();
        let file = path.to_str().unwrap().to_string();

        let undecided = import_las_files_with(
            &conn,
            &[file.clone()],
            None,
            &LasImportOptions::default(),
        )
        .remove(0);
        assert!(
            undecided.error.as_deref().is_some_and(|error| {
                error.contains("3 repeated depth row(s)") && error.contains("declared duplicate policy")
            }),
            "the count and missing decision are both named: {:?}",
            undecided.error
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM wells", [], |row| row.get::<_, i64>(0)).unwrap(),
            0,
            "no policy means no commit"
        );

        let refused = import_las_files_with(
            &conn,
            &[file.clone()],
            None,
            &LasImportOptions {
                duplicate_depth_policy: Some(parsers::DuplicateDepthPolicy::Refuse),
                ..Default::default()
            },
        )
        .remove(0);
        assert!(refused.error.as_deref().is_some_and(|error| error.contains("blocked 3")));

        let kept = import_las_files_with(
            &conn,
            &[file],
            None,
            &LasImportOptions {
                duplicate_depth_policy: Some(parsers::DuplicateDepthPolicy::KeepFirst),
                ..Default::default()
            },
        )
        .remove(0);
        std::fs::remove_file(&path).ok();
        assert!(kept.error.is_none(), "keep-first imports: {:?}", kept.error);
        assert_eq!(kept.rows, 2, "three repeated rows are resolved to one depth");
        assert!(
            kept.warning.as_deref().is_some_and(|warning| {
                warning.contains("3 repeated depth row(s)") && warning.contains("keep-first")
            }),
            "the policy and affected count remain visible: {:?}",
            kept.warning
        );
        let first_gr: f32 = conn
            .query_row(
                "SELECT gr FROM standard_curves WHERE well_id = ?1 AND depth = 1000.0",
                params![kept.well_id.as_deref().unwrap()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(first_gr, 10.0, "keep-first keeps the first sample, not the PK's accident");

        let make_columns = || CurveColumns {
            las_version: None,
            unread_sections: Vec::new(),
            text_encoding: "test fixture".into(),
            depth_unit: Some("M".into()),
            declared_step: None,
            depth: vec![1000.0, 1000.0, 1000.0, 1000.0, 1001.0],
            gr: vec![10.0, 20.0, 30.0, 40.0, 50.0],
            res: vec![f32::NAN; 5],
            nphi: vec![f32::NAN; 5],
            rhob: vec![f32::NAN; 5],
            dt: vec![f32::NAN; 5],
            sp: vec![f32::NAN; 5],
            alias_decisions: Vec::new(),
            index_resolution: None,
            unit_designations: Vec::new(),
        };
        let mut last = make_columns();
        assert_eq!(
            parsers::resolve_curve_column_duplicates(&mut last, parsers::DuplicateDepthPolicy::KeepLast),
            3
        );
        assert_eq!(last.gr, vec![40.0, 50.0], "keep-last keeps the last repeated sample");
        let mut mean = make_columns();
        parsers::resolve_curve_column_duplicates(&mut mean, parsers::DuplicateDepthPolicy::Mean);
        assert_eq!(mean.gr, vec![25.0, 50.0], "mean averages the four finite repeated samples");
    }

    /// SB-DIO-015 / SB-DIO-T22..T24. The accepted depth-unit spellings and the
    /// international-foot factor are cited in `docs/PRD_v2/21_data-io.md` §5.1.
    /// A project declaration is deliberately not used as evidence for an undeclared file.
    #[test]
    fn an_undeclared_index_unit_refuses_until_the_files_unit_is_explicitly_confirmed() {
        let make = |name: &str, unit: &str, well: &str| {
            let path = std::env::temp_dir().join(name);
            std::fs::write(
                &path,
                format!(
                    "~VERSION\nVERS. 2.0 :\n~WELL\nWELL. {well} :\n~CURVE\nDEPT.{unit} : depth\nGR.GAPI :\n~ASCII\n1000 50\n1001 55\n"
                ),
            )
            .unwrap();
            path
        };

        let fresh = Connection::open_in_memory().unwrap();
        db::create_schema(&fresh).unwrap();
        let no_unit = make("sandibumi_dio015_none.las", "", "SANDI-NONE");
        let fresh_result = &import_las_files_with(
            &fresh,
            &[no_unit.to_str().unwrap().to_string()],
            None,
            &LasImportOptions::default(),
        )[0];
        assert!(
            fresh_result.error.as_deref().is_some_and(|e| e.contains("file index") && e.contains("project")),
            "the refusal must name both possible sources: {:?}",
            fresh_result.error
        );

        let metric = Connection::open_in_memory().unwrap();
        db::create_schema(&metric).unwrap();
        crate::units::set_project_depth_unit(&metric, crate::units::DepthUnit::Metres).unwrap();
        let still_refused = &import_las_files_with(
            &metric,
            &[no_unit.to_str().unwrap().to_string()],
            None,
            &LasImportOptions::default(),
        )[0];
        assert!(
            still_refused.error.as_deref().is_some_and(|e| e.contains("project setting is not a file declaration")),
            "a declared project must not silently lend its unit to the file: {:?}",
            still_refused.error
        );

        let confirmed = LasImportOptions {
            file_depth_unit: Some("FT".into()),
            ..Default::default()
        };
        let accepted = &import_las_files_with(
            &metric,
            &[no_unit.to_str().unwrap().to_string()],
            None,
            &confirmed,
        )[0];
        assert!(accepted.error.is_none(), "explicit confirmation must unblock: {:?}", accepted.error);
        assert!(
            accepted.warning.as_deref().unwrap_or("").contains("explicitly confirmed as FT"),
            "the confirmation is part of the import record: {:?}",
            accepted.warning
        );
        let first_depth: f32 = metric
            .query_row(
                "SELECT MIN(depth) FROM standard_curves WHERE well_id = ?1",
                params![accepted.well_id.as_deref().unwrap()],
                |row| row.get(0),
            )
            .unwrap();
        assert!((first_depth - 304.8).abs() < 1e-3, "confirmed feet convert by the cited 0.3048 factor");

        let declared = make("sandibumi_dio015_declared.las", "FT", "SANDI-DECLARED");
        let declared_result = &import_las_files_with(
            &metric,
            &[declared.to_str().unwrap().to_string()],
            None,
            &LasImportOptions::default(),
        )[0];
        assert!(declared_result.error.is_none(), "a declared file needs no confirmation");
        assert!(
            declared_result.warning.as_deref().unwrap_or("").contains("converted from ft"),
            "the declared-unit conversion is still reported: {:?}",
            declared_result.warning
        );

        std::fs::remove_file(no_unit).ok();
        std::fs::remove_file(declared).ok();
    }

    /// SB-DIO-024 / SB-DIO-T39. The international-foot factor is 0.3048 m/ft
    /// (NIST SP 811, chapter §5.1). Reporting alone is not enough: the stored sample
    /// is checked too, so a no-op conversion with a plausible audit record cannot pass.
    #[test]
    fn a_converted_sonic_reports_its_from_unit_to_unit_and_factor() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let las = "~Version\n\
                   VERS. 2.0 :\n\
                   ~Well\n\
                   WELL. DIO-024 :\n\
                   ~Curve\n\
                   DEPT .M    : depth\n\
                   DTCO .US/M : sonic\n\
                   ~ASCII\n\
                   1000.0 100.0\n\
                   1000.5 200.0\n";
        let path = std::env::temp_dir().join(format!(
            "sandibumi-dio024-{}-{}.las",
            std::process::id(),
            Uuid::new_v4()
        ));
        std::fs::write(&path, las).unwrap();

        let result = import_las_files_with(
            &conn,
            &[path.to_string_lossy().to_string()],
            None,
            &LasImportOptions::default(),
        )
        .remove(0);
        std::fs::remove_file(&path).ok();
        assert!(result.error.is_none(), "import failed: {:?}", result.error);
        assert_eq!(result.unit_conversions.len(), 1);
        let conversion = &result.unit_conversions[0];
        assert_eq!(conversion.curve, "DTCO");
        assert_eq!(conversion.from_unit, "US/M");
        assert_eq!(conversion.to_unit, "us/ft");
        assert_eq!(conversion.factor, 0.3048_f32);
        assert_eq!(conversion.offset, 0.0, "a multiplicative conversion carries an explicit zero offset");
        assert!(
            result.warning.as_deref().is_some_and(|note| {
                note.contains("DTCO")
                    && note.contains("US/M")
                    && note.contains("us/ft")
                    && note.contains("0.3048")
            }),
            "the visible import note must carry the same audit: {:?}",
            result.warning
        );

        let well_id = result.well_id.unwrap();
        let dt = db::list_generic_curve_catalog(&conn, &well_id)
            .unwrap()
            .into_iter()
            .find(|curve| curve.mnemonic == "DTCO")
            .expect("DTCO generic curve");
        let samples = db::get_curve_samples(&conn, &dt.curve_id).unwrap();
        assert!((samples[0].value - 30.48).abs() < 1e-4, "100 us/m must become 30.48 us/ft");
    }

    /// SB-DIO-025 / SB-DIO-T40. A declared unit with no reviewed transform remains
    /// attached to unchanged samples and is explicitly reported as unconverted. The
    /// deliberately absurd unit makes accidental canonical treatment unambiguous.
    #[test]
    fn an_unknown_declared_unit_is_stored_verbatim_and_flagged_unconverted() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let las = "~Version\n\
                   VERS. 2.0 :\n\
                   ~Well\n\
                   WELL. DIO-025 :\n\
                   ~Curve\n\
                   DEPT .M         : depth\n\
                   RHOZ .FURLONGS  : unsupported density unit\n\
                   ~ASCII\n\
                   1000.0 2400.0\n\
                   1000.5 2500.0\n";
        let path = std::env::temp_dir().join(format!(
            "sandibumi-dio025-{}-{}.las",
            std::process::id(),
            Uuid::new_v4()
        ));
        std::fs::write(&path, las).unwrap();
        let result = import_las_files_with(
            &conn,
            &[path.to_string_lossy().to_string()],
            None,
            &LasImportOptions::default(),
        )
        .remove(0);
        std::fs::remove_file(&path).ok();

        assert!(result.error.is_none(), "import failed: {:?}", result.error);
        assert!(result.unit_conversions.is_empty(), "an unknown unit must not masquerade as converted");
        assert_eq!(result.unconverted_units.len(), 1);
        let issue = &result.unconverted_units[0];
        assert_eq!(issue.curve, "RHOZ");
        assert_eq!(issue.declared_unit, "FURLONGS");
        assert_eq!(issue.family.as_deref(), Some("RHOB"));
        assert!(
            result.warning.as_deref().is_some_and(|note| {
                note.contains("RHOZ") && note.contains("FURLONGS") && note.contains("unconverted")
            }),
            "the pass-through must be visible: {:?}",
            result.warning
        );

        let well_id = result.well_id.unwrap();
        let rhob = db::list_generic_curve_catalog(&conn, &well_id)
            .unwrap()
            .into_iter()
            .find(|curve| curve.mnemonic == "RHOZ")
            .expect("RHOZ generic curve");
        assert_eq!(rhob.unit.as_deref(), Some("FURLONGS"));
        let samples = db::get_curve_samples(&conn, &rhob.curve_id).unwrap();
        assert_eq!(samples[0].value, 2400.0, "unconvertible data must be stored unchanged");
    }

    /// SB-DIO-026 / SB-DIO-T42. Chapter §5.1 cites the 32 °F offset, and T42
    /// fixes the complete expected transform: 200 °F → 93.33 °C. The explicit
    /// comparison with the multiplicative-only 111.11 answer pins both sides.
    #[test]
    fn a_fahrenheit_temperature_applies_its_affine_offset_before_its_factor() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let las = "~Version\n\
                   VERS. 2.0 :\n\
                   ~Well\n\
                   WELL. DIO-026 :\n\
                   ~Curve\n\
                   DEPT .M    : depth\n\
                   FTEMP.DEGF : formation temperature\n\
                   ~ASCII\n\
                   1000.0 200.0\n\
                   1000.5 32.0\n";
        let path = std::env::temp_dir().join(format!(
            "sandibumi-dio026-{}-{}.las",
            std::process::id(),
            Uuid::new_v4()
        ));
        std::fs::write(&path, las).unwrap();
        let result = import_las_files_with(
            &conn,
            &[path.to_string_lossy().to_string()],
            None,
            &LasImportOptions::default(),
        )
        .remove(0);
        std::fs::remove_file(&path).ok();

        assert!(result.error.is_none(), "import failed: {:?}", result.error);
        let conversion = result.unit_conversions.iter().find(|item| item.curve == "FTEMP").unwrap();
        assert_eq!(conversion.from_unit, "DEGF");
        assert_eq!(conversion.to_unit, "DEGC");
        assert!((conversion.factor - 1.0 / 1.8).abs() < 1e-7);
        assert_eq!(conversion.offset, -32.0);
        let well_id = result.well_id.unwrap();
        let temperature = db::list_generic_curve_catalog(&conn, &well_id)
            .unwrap()
            .into_iter()
            .find(|curve| curve.mnemonic == "FTEMP")
            .expect("FTEMP generic curve");
        assert_eq!(temperature.family.as_deref(), Some("TEMP"));
        assert_eq!(temperature.unit.as_deref(), Some("DEGC"));
        let samples = db::get_curve_samples(&conn, &temperature.curve_id).unwrap();
        assert!((samples[0].value - 93.333_336).abs() < 1e-4, "200 °F must become 93.33 °C");
        assert!((samples[0].value - 111.111_115).abs() > 1.0, "the offset must not be omitted");
        assert!(samples[1].value.abs() < 1e-6, "32 °F must become 0 °C");
    }

    /// SB-DIO-027 / SB-DIO-T43. Finding D-14 and chapter §5.1 mark the vendor
    /// `density.units: PPG → density` entry NON-ADOPTABLE because PPG denotes a
    /// pressure-gradient quantity, not bulk density. Both stores are checked: it
    /// must neither populate standard RHOB nor acquire a generic RHOB family tag.
    #[test]
    fn a_ppg_column_is_not_bound_to_density_and_is_flagged_for_designation() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let las = "~Version\n\
                   VERS. 2.0 :\n\
                   ~Well\n\
                   WELL. DIO-027 :\n\
                   ~Curve\n\
                   DEPT.M   : depth\n\
                   RHOZ.PPG : vendor-labelled mud weight\n\
                   ~ASCII\n\
                   1000.0 9.5\n\
                   1000.5 10.0\n";
        let path = std::env::temp_dir().join(format!(
            "sandibumi-dio027-{}-{}.las",
            std::process::id(),
            Uuid::new_v4()
        ));
        std::fs::write(&path, las).unwrap();
        let result = import_las_files_with(
            &conn,
            &[path.to_string_lossy().to_string()],
            None,
            &LasImportOptions::default(),
        )
        .remove(0);
        std::fs::remove_file(&path).ok();

        assert!(result.error.is_none(), "import failed: {:?}", result.error);
        let rejection = result
            .unconverted_units
            .iter()
            .find(|issue| issue.curve == "RHOZ")
            .expect("PPG rejection record");
        assert_eq!(rejection.declared_unit, "PPG");
        assert_eq!(rejection.family, None, "a rejected binding must not report an assigned family");
        assert!(rejection.designation_required);
        assert_eq!(rejection.rejected_entry.as_deref(), Some("density.units: PPG -> density"));
        assert!(
            result.warning.as_deref().is_some_and(|note| {
                note.contains("PPG") && note.contains("pressure-gradient") && note.contains("designation")
            }),
            "the rejected entry must be visible: {:?}",
            result.warning
        );

        let well_id = result.well_id.unwrap();
        let standard_rhob: f32 = conn
            .query_row(
                "SELECT rhob FROM standard_curves WHERE well_id = ?1 ORDER BY depth LIMIT 1",
                params![&well_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(standard_rhob.is_nan(), "PPG data must not populate the standard RHOB channel");
        let raw = db::list_generic_curve_catalog(&conn, &well_id)
            .unwrap()
            .into_iter()
            .find(|curve| curve.mnemonic == "RHOZ")
            .expect("RHOZ generic curve retained for designation");
        assert_eq!(raw.family, None);
        assert_eq!(raw.unit.as_deref(), Some("PPG"));
        let samples = db::get_curve_samples(&conn, &raw.curve_id).unwrap();
        assert_eq!(samples[0].value, 9.5, "rejection retains the source data for later designation");
    }

    /// SB-DIO-029 / SB-DIO-T45. Finding D-12 establishes two legitimate readings
    /// for MS/FT and no evidence in the file that selects between them. The no-answer,
    /// microsecond and millisiemens paths are all pinned so an implementation that always
    /// assumes the legacy sonic reading cannot pass.
    #[test]
    fn an_ms_per_ft_curve_waits_for_a_per_file_quantity_answer_and_records_either_answer() {
        let las = "~Version\n\
                   VERS. 2.0 :\n\
                   ~Well\n\
                   WELL. DIO-029 :\n\
                   ~Curve\n\
                   DEPT.M     : depth\n\
                   DTCO.MS/FT : ambiguous channel\n\
                   ~ASCII\n\
                   1000.0 100.0\n\
                   1000.5 110.0\n";
        let path = std::env::temp_dir().join(format!(
            "sandibumi-dio029-{}-{}.las",
            std::process::id(),
            Uuid::new_v4()
        ));
        std::fs::write(&path, las).unwrap();
        let file = path.to_string_lossy().to_string();

        let undecided = Connection::open_in_memory().unwrap();
        db::create_schema(&undecided).unwrap();
        let blocked = import_las_files_with(
            &undecided,
            std::slice::from_ref(&file),
            None,
            &LasImportOptions::default(),
        )
        .remove(0);
        assert!(
            blocked.error.as_deref().is_some_and(|error| {
                error.contains("MS/FT")
                    && error.contains("microseconds per foot")
                    && error.contains("millisiemens per foot")
                    && error.contains("per-file")
            }),
            "the ambiguity and both meanings must be named: {:?}",
            blocked.error
        );
        assert_eq!(
            undecided.query_row("SELECT COUNT(*) FROM wells", [], |row| row.get::<_, i64>(0)).unwrap(),
            0,
            "no answer means no commit"
        );

        let sonic = Connection::open_in_memory().unwrap();
        db::create_schema(&sonic).unwrap();
        let sonic_result = import_las_files_with(
            &sonic,
            std::slice::from_ref(&file),
            None,
            &LasImportOptions {
                ms_per_ft_meanings: std::collections::HashMap::from([(
                    file.clone(),
                    crate::curves::MsPerFtMeaning::MicrosecondsPerFoot,
                )]),
                ..Default::default()
            },
        )
        .remove(0);
        assert!(sonic_result.error.is_none(), "sonic designation failed: {:?}", sonic_result.error);
        let sonic_answer = sonic_result.unit_designations.first().expect("sonic answer recorded");
        assert_eq!(sonic_answer.meaning, "microseconds_per_foot");
        assert_eq!(sonic_answer.recorded_unit, "us/ft");
        assert_eq!(sonic_answer.family.as_deref(), Some("DT"));
        assert!(
            sonic_result.warning.as_deref().is_some_and(|note| {
                note.contains("DTCO") && note.contains("MS/FT") && note.contains("microseconds_per_foot")
            }),
            "the per-file answer must also be visible: {:?}",
            sonic_result.warning
        );
        let sonic_curve = db::list_generic_curve_catalog(&sonic, sonic_result.well_id.as_deref().unwrap())
            .unwrap()
            .into_iter()
            .find(|curve| curve.mnemonic == "DTCO")
            .unwrap();
        assert_eq!(sonic_curve.family.as_deref(), Some("DT"));
        assert_eq!(sonic_curve.unit.as_deref(), Some("us/ft"));
        assert_eq!(db::get_curve_samples(&sonic, &sonic_curve.curve_id).unwrap()[0].value, 100.0);

        let conductivity = Connection::open_in_memory().unwrap();
        db::create_schema(&conductivity).unwrap();
        let conductivity_result = import_las_files_with(
            &conductivity,
            std::slice::from_ref(&file),
            None,
            &LasImportOptions {
                ms_per_ft_meanings: std::collections::HashMap::from([(
                    file.clone(),
                    crate::curves::MsPerFtMeaning::MillisiemensPerFoot,
                )]),
                ..Default::default()
            },
        )
        .remove(0);
        std::fs::remove_file(&path).ok();
        assert!(
            conductivity_result.error.is_none(),
            "conductivity designation failed: {:?}",
            conductivity_result.error
        );
        let conductivity_answer = conductivity_result.unit_designations.first().expect("conductivity answer recorded");
        assert_eq!(conductivity_answer.meaning, "millisiemens_per_foot");
        assert_eq!(conductivity_answer.recorded_unit, "MS/FT");
        assert_eq!(conductivity_answer.family, None);
        let conductivity_well = conductivity_result.well_id.as_deref().unwrap();
        let standard_dt: f32 = conductivity
            .query_row(
                "SELECT dt FROM standard_curves WHERE well_id = ?1 ORDER BY depth LIMIT 1",
                params![conductivity_well],
                |row| row.get(0),
            )
            .unwrap();
        assert!(standard_dt.is_nan(), "a conductivity designation must not populate standard DT");
        let conductivity_curve = db::list_generic_curve_catalog(&conductivity, conductivity_well)
            .unwrap()
            .into_iter()
            .find(|curve| curve.mnemonic == "DTCO")
            .unwrap();
        assert_eq!(conductivity_curve.family, None);
        assert_eq!(conductivity_curve.unit.as_deref(), Some("MS/FT"));
        assert_eq!(db::get_curve_samples(&conductivity, &conductivity_curve.curve_id).unwrap()[0].value, 100.0);
    }

    /// Phase 6b: a full LAS with curves beyond the fixed 6 (PEF, CALI, a metric-unit
    /// sonic) must import whole into the generic store, with families tagged and units
    /// canonicalized. Also exercises the deviation-survey → minimum-curvature → well_path
    /// path end to end.
    #[test]
    fn generic_las_import_keeps_all_curves_and_converts_units() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "PEF-1", None, None, Some(25.0)).unwrap();
        let ids = well_id.to_string();

        // A minimal LAS 2.0 with DEPT, GR, PEF, HCAL (caliper), and DTCO given in us/m.
        let las = "~Version\n\
                   VERS. 2.0 :\n\
                   ~Curve\n\
                   DEPT .M    : depth\n\
                   GR   .GAPI : gamma\n\
                   PEF  .B/E  : photoelectric\n\
                   HCAL .IN   : caliper\n\
                   DTCO .US/M : sonic\n\
                   ~ASCII\n\
                   1000.0 55.0 5.1 8.5 656.0\n\
                   1000.5 60.0 5.2 8.6 660.0\n\
                   1001.0 -999.25 5.0 8.4 650.0\n";
        let path = std::env::temp_dir().join(format!("arshilla_pef_test_{ids}.las"));
        std::fs::write(&path, las).unwrap();

        import_all_curves_into_generic_store(&conn, &ids, path.to_str().unwrap(), "RAW", None).unwrap();
        std::fs::remove_file(&path).ok();

        let catalog = db::list_generic_curve_catalog(&conn, &ids).unwrap();
        // PEF and CALI (family of HCAL) must be present with correct families.
        let pef = catalog.iter().find(|c| c.mnemonic == "PEF").expect("PEF imported");
        assert_eq!(pef.family.as_deref(), Some("PEF"));
        assert_eq!(pef.n_samples, 3);
        let cali = catalog.iter().find(|c| c.mnemonic == "HCAL").expect("HCAL imported");
        assert_eq!(cali.family.as_deref(), Some("CALI"));

        // DTCO in us/m must have been converted to us/ft and relabeled.
        let dt = catalog.iter().find(|c| c.mnemonic == "DTCO").expect("DTCO imported");
        assert_eq!(dt.unit.as_deref(), Some("us/ft"));
        let dt_samples = db::get_curve_samples(&conn, &dt.curve_id).unwrap();
        assert!((dt_samples[0].value - 656.0 * 0.3048).abs() < 0.5, "us/m→us/ft, got {}", dt_samples[0].value);

        // The LAS null (-999.25) in GR must be NaN in the store.
        let gr = catalog.iter().find(|c| c.mnemonic == "GR").expect("GR imported");
        let gr_samples = db::get_curve_samples(&conn, &gr.curve_id).unwrap();
        assert!(gr_samples[2].value.is_nan(), "LAS null must become NaN");

        // Deviation survey → TVD/TVDSS.
        let dev = std::env::temp_dir().join(format!("arshilla_dev_test_{ids}.csv"));
        std::fs::write(&dev, "MD,INC,AZI\n0,0,0\n1000,0,0\n2000,60,45\n").unwrap();
        let res = import_deviation_csv(&conn, &ids, dev.to_str().unwrap(), Some(25.0), None);
        std::fs::remove_file(&dev).ok();
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.rows, 3);
        let path = db::get_well_path(&conn, &ids).unwrap();
        assert_eq!(path.len(), 3);
        assert!((path[1].tvd - 1000.0).abs() < 1e-2, "vertical section TVD == MD");
        assert!(path[2].tvd < path[2].md, "deviated station TVD shallower than MD");
        assert!((path[1].tvdss - (25.0 - 1000.0)).abs() < 1e-2, "TVDSS = datum - TVD");
    }

    #[test]
    fn deviation_import_materializes_tvd_tvdss_curves() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "DEV-MAT-1", None, None, None).unwrap();
        let ids = wid.to_string();

        // A log depth (MD) grid spanning the whole survey, incl. a deviated section.
        let depth = vec![0.0f32, 1000.0, 1500.0, 2000.0, 3000.0];
        let f = vec![1.0f32; depth.len()];
        crate::db::insert_standard_curves(
            &conn, wid, depth.clone(), vec![50.0f32; depth.len()],
            f.clone(), f.clone(), f.clone(), f.clone(), f.clone(),
        )
        .unwrap();

        // Vertical to 1000, build to 60° by 2000, hold to 3000.
        let dev = std::env::temp_dir().join(format!("arshilla_devmat_{ids}.csv"));
        std::fs::write(&dev, "MD,INC,AZI\n0,0,0\n1000,0,0\n2000,60,45\n3000,60,45\n").unwrap();
        let res = import_deviation_csv(&conn, &ids, dev.to_str().unwrap(), Some(25.0), None);
        std::fs::remove_file(&dev).ok();
        assert!(res.error.is_none(), "{:?}", res.error);

        // Import auto-materialized TVD + TVDSS onto the log grid, fetchable by name.
        let (grid, cols) =
            crate::equations::fetch_curve_frame(&conn, &ids, &["TVD".to_string(), "TVDSS".to_string()]).unwrap();
        assert_eq!(grid, depth, "curves land on the standard depth grid");
        let (tvd, tvdss) = (&cols["TVD"], &cols["TVDSS"]);
        let i1000 = grid.iter().position(|&d| d == 1000.0).unwrap();
        assert!((tvd[i1000] - 1000.0).abs() < 1e-1, "vertical section TVD == MD: {}", tvd[i1000]);
        let i3000 = grid.iter().position(|&d| d == 3000.0).unwrap();
        assert!(tvd[i3000] < 2900.0, "deviated section TVD shallower than MD: {}", tvd[i3000]);
        // TVDSS = datum(25) − TVD everywhere (the interpolation preserves the affine relation).
        for (t, ss) in tvd.iter().zip(tvdss.iter()) {
            assert!((ss - (25.0 - t)).abs() < 1e-1, "TVDSS = 25 - TVD: {ss} vs {}", 25.0 - t);
        }
    }

    /// Survey versioning (T-IMP-12): a second survey lands beside the first instead of
    /// replacing it, the newest drives TVD, and switching back RE-materializes the older
    /// geometry — a stale TVD would silently poison every height calculation.
    #[test]
    fn deviation_import_versions_surveys_and_switching_rebuilds_tvd() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "DEV-VER-1", None, None, None).unwrap();
        let ids = wid.to_string();
        let depth = vec![0.0f32, 1000.0, 2000.0, 3000.0];
        let f = vec![1.0f32; depth.len()];
        crate::db::insert_standard_curves(
            &conn, wid, depth.clone(), vec![50.0f32; depth.len()],
            f.clone(), f.clone(), f.clone(), f.clone(), f,
        )
        .unwrap();

        let write = |name: &str, body: &str| -> String {
            let p = std::env::temp_dir().join(format!("arshilla_devver_{name}_{ids}.csv"));
            std::fs::write(&p, body).unwrap();
            p.to_string_lossy().into_owned()
        };
        // Preliminary: vertical all the way. Definitive: builds to 60°.
        let prelim = write("prelim", "MD,INC,AZI\n0,0,0\n1000,0,0\n2000,0,0\n3000,0,0\n");
        let defin = write("defin", "MD,INC,AZI\n0,0,0\n1000,0,0\n2000,60,45\n3000,60,45\n");

        assert!(import_deviation_csv(&conn, &ids, &prelim, Some(25.0), Some("PRELIM")).error.is_none());
        assert!(import_deviation_csv(&conn, &ids, &defin, Some(25.0), Some("DEFINITIVE")).error.is_none());

        let surveys = db::list_surveys(&conn, &ids).unwrap();
        assert_eq!(surveys.len(), 2, "the preliminary survey survives: {surveys:?}");
        assert_eq!(surveys[0].survey_name, "DEFINITIVE", "newest import is active");
        assert!(surveys[0].active && !surveys[1].active);
        assert_eq!(surveys.iter().map(|s| s.stations).sum::<i64>(), 8);

        // Readers see ONE survey, and it is the definitive (deviated) one.
        let path = db::get_well_path(&conn, &ids).unwrap();
        assert_eq!(path.len(), 4, "never both surveys merged");
        assert!(path[3].tvd < 2900.0, "definitive geometry is deviated: {}", path[3].tvd);

        // Switch back: TVD must be rebuilt from the preliminary (vertical) survey.
        db::set_active_survey(&conn, &ids, "PRELIM").unwrap();
        materialize_tvd_curves(&conn, &ids).unwrap();
        let (_g, cols) = crate::equations::fetch_curve_frame(&conn, &ids, &["TVD".to_string()]).unwrap();
        let last = *cols["TVD"].last().unwrap();
        assert!((last - 3000.0).abs() < 1e-1, "vertical survey → TVD == MD, got {last}");

        std::fs::remove_file(&prelim).ok();
        std::fs::remove_file(&defin).ok();
    }

    #[test]
    fn materialize_tvd_no_survey_writes_nothing() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "DEV-MAT-2", None, None, None).unwrap();
        let ids = wid.to_string();
        let depth = vec![0.0f32, 100.0, 200.0];
        let f = vec![1.0f32; depth.len()];
        crate::db::insert_standard_curves(
            &conn, wid, depth.clone(), vec![50.0f32; depth.len()],
            f.clone(), f.clone(), f.clone(), f.clone(), f,
        )
        .unwrap();
        // No survey → 0 samples written, and TVD stays absent (all-NaN via generic fallback).
        assert_eq!(materialize_tvd_curves(&conn, &ids).unwrap(), 0);
        let (_d, cols) = crate::equations::fetch_curve_frame(&conn, &ids, &["TVD".to_string()]).unwrap();
        assert!(cols["TVD"].iter().all(|v| v.is_nan()), "no survey → no TVD curve");
    }

    /// A vendor TVDSS imported into the generic RAW store must stay authoritative after a
    /// deviation survey is imported — the survey-derived COMPUTED TVDSS (which outranks the
    /// generic store in fetch_curve_frame) must not silently shadow it. TVD, with no import,
    /// is still materialized. Guards the cross-feature seam between TVD materialization and the
    /// standard→computed→generic resolution precedence.
    #[test]
    fn materialize_tvd_keeps_imported_tvdss_authoritative() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "DEV-MAT-3", None, None, None).unwrap();
        let ids = wid.to_string();
        let depth = vec![0.0f32, 1000.0, 2000.0, 3000.0];
        let f = vec![1.0f32; depth.len()];
        crate::db::insert_standard_curves(
            &conn, wid, depth.clone(), vec![50.0f32; depth.len()],
            f.clone(), f.clone(), f.clone(), f.clone(), f,
        )
        .unwrap();

        // A vendor TVDSS in the generic RAW store: a constant sentinel no survey-derived TVDSS
        // could produce, so we can tell which one resolves.
        let cid = crate::db::upsert_curve_meta(
            &conn, &ids, "RAW", "TVDSS", Some("m"), Some("TVDSS"), Some("LAS import"), None,
        )
        .unwrap();
        crate::db::insert_curve_samples(&conn, &cid, &depth, &vec![-777.0f32; depth.len()]).unwrap();

        // Import a deviated survey (would compute a very DIFFERENT TVDSS = 25 − TVD).
        let dev = std::env::temp_dir().join(format!("arshilla_devmat3_{ids}.csv"));
        std::fs::write(&dev, "MD,INC,AZI\n0,0,0\n1000,0,0\n2000,60,45\n3000,60,45\n").unwrap();
        let res = import_deviation_csv(&conn, &ids, dev.to_str().unwrap(), Some(25.0), None);
        std::fs::remove_file(&dev).ok();
        assert!(res.error.is_none(), "{:?}", res.error);

        let (_g, cols) = crate::equations::fetch_curve_frame(
            &conn, &ids, &["TVDSS".to_string(), "TVD".to_string()],
        )
        .unwrap();
        // Imported TVDSS still wins — NOT overwritten by the survey-derived computed curve.
        assert!(
            cols["TVDSS"].iter().all(|&v| (v - (-777.0)).abs() < 1e-3),
            "imported TVDSS must remain authoritative, got {:?}",
            cols["TVDSS"]
        );
        // TVD had no import → it IS materialized from the survey (shallower than MD when deviated).
        assert!(cols["TVD"].iter().any(|v| !v.is_nan()), "TVD still materialized from the survey");

        // And the stale-cleanup path: even if a computed TVDSS already existed (a survey was
        // materialized BEFORE the vendor curve arrived), a recompute clears it so the import wins.
        crate::equations::write_computed_curve(&conn, &ids, &depth, "TVDSS", &vec![9.9f32; depth.len()]).unwrap();
        materialize_tvd_curves(&conn, &ids).unwrap();
        let (_g2, cols2) = crate::equations::fetch_curve_frame(&conn, &ids, &["TVDSS".to_string()]).unwrap();
        assert!(
            cols2["TVDSS"].iter().all(|&v| (v - (-777.0)).abs() < 1e-3),
            "recompute must clear a stale survey TVDSS so the import wins, got {:?}",
            cols2["TVDSS"]
        );
    }

    /// #118 follow-up: a spliced LAS with a duplicate depth must import cleanly on BOTH the
    /// standard-curves AND the generic-store path. The generic path re-parses the file and
    /// writes curve_samples (curve_id, depth) PK, so without the same depth dedup it aborts
    /// silently (Err only logged), leaving the well missing its PEF/extra curves.
    #[test]
    fn duplicate_depth_las_imports_standard_and_generic_curves() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();

        // Two rows at 1000.0 (a re-spliced section) plus PEF beyond the standard 6.
        let las = "~Version\nVERS. 2.0 :\n\
                   ~Curve\nDEPT .M : depth\nGR .GAPI : gamma\nPEF .B/E : pe\n\
                   ~ASCII\n1000.0 55.0 5.1\n1000.0 56.0 5.2\n1000.5 60.0 5.0\n";
        let path = std::env::temp_dir().join("arshilla_dupdepth_test.las");
        std::fs::write(&path, las).unwrap();

        let results = import_las_files_with(
            &conn,
            &[path.to_str().unwrap().to_string()],
            None,
            &LasImportOptions {
                duplicate_depth_policy: Some(parsers::DuplicateDepthPolicy::KeepFirst),
                ..Default::default()
            },
        );
        std::fs::remove_file(&path).ok();

        assert_eq!(results.len(), 1);
        let r = &results[0];
        assert!(r.error.is_none(), "import must succeed, got {:?}", r.error);
        assert_eq!(r.rows, 2, "duplicate 1000.0 row dropped → 2 unique depths");
        assert!(
            r.warning.as_deref().unwrap_or("").contains("duplicate"),
            "duplicate-depth warning surfaced, got {:?}",
            r.warning
        );

        let ids = r.well_id.clone().unwrap();
        let n_std: i64 = conn
            .query_row("SELECT COUNT(*) FROM standard_curves WHERE well_id = ?1", params![ids], |r| r.get(0))
            .unwrap();
        assert_eq!(n_std, 2, "standard_curves deduped to 2 rows");
        // The generic store must ALSO carry PEF — not silently missing from a PK abort.
        let catalog = db::list_generic_curve_catalog(&conn, &ids).unwrap();
        let pef = catalog.iter().find(|c| c.mnemonic == "PEF").expect("PEF must reach the generic store");
        assert_eq!(pef.n_samples, 2, "generic PEF deduped to 2 rows, not aborted");
    }

    /// Core import v2 (T-IMP-07): a real delivery shape end-to-end — WN well column,
    /// units row, feet depths, percent porosity, an unmatched name, an ambiguous name,
    /// and a blank well cell. Probe must SEE all of it; commit must route, convert, and
    /// report without guessing.
    #[test]
    fn core_table_probe_and_multiwell_import() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        // Metric project (declared explicitly, as a LAS import would have).
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let wa = Uuid::new_v4();
        let wb = Uuid::new_v4();
        db::insert_well(&conn, wa, "W-A", None, None, None).unwrap();
        db::insert_well(&conn, wb, "W-B", None, None, None).unwrap();
        // Two records sharing a name → ambiguous, must never be guessed at.
        db::insert_well(&conn, Uuid::new_v4(), "DUP-C", None, None, None).unwrap();
        db::insert_well(&conn, Uuid::new_v4(), "DUP-C", None, None, None).unwrap();

        let csv = "TAPE_NAME,TOOL_STRING,WN,DEPTH,CPERM_1,CPOR_2,CSO_1,CSW_1,GDEN_1,LITH\n\
                   \"\",\"\",\"\",FEET,MD,V/V,V/V,V/V,G/C3,\n\
                   \"\",\"\",W-A,1000.0,120.0,24.5,15.0,55.0,2.66,SANDSTONE\n\
                   \"\",\"\",W-A,1001.0,85.0,22.0,20.0,60.0,2.65,SHALY SAND\n\
                   \"\",\"\",W-B,2000.0,10.0,18.0,5.0,80.0,2.68,SANDSTONE\n\
                   \"\",\"\",W-B,2001.0,12.0,19.0,6.0,78.0,2.67,\n\
                   \"\",\"\",GHOST-9,3000.0,1.0,10.0,1.0,90.0,2.70,SILTSTONE\n\
                   \"\",\"\",DUP-C,4000.0,2.0,11.0,2.0,88.0,2.69,SANDSTONE\n\
                   \"\",\"\",,5000.0,3.0,12.0,3.0,85.0,2.71,SANDSTONE\n";
        let path = std::env::temp_dir().join("sandibumi_core_v2_test.csv");
        std::fs::write(&path, csv).unwrap();
        let spath = path.to_str().unwrap();

        // --- Probe: everything the dialog shows must be detected. ---
        let probe = parsers::probe_core_table(&path).unwrap();
        assert_eq!(probe.well, Some(2), "WN resolves as the well column");
        assert_eq!(probe.depth, Some(3));
        assert_eq!(probe.cperm, Some(4), "CPERM_1 resolves");
        assert_eq!(probe.cpor, Some(5), "CPOR_2 resolves");
        assert_eq!(probe.cgd, Some(8), "GDEN_1 resolves");
        assert!(probe.units_row_skipped, "the FEET/MD/V-V row is a units row, not a plug");
        assert_eq!(probe.depth_unit_guess.as_deref(), Some("ft"), "unit read from the units row");
        assert_eq!(probe.n_rows, 7, "7 data rows (units row excluded)");
        assert!(probe.percent_roles.iter().any(|r| r == "CPOR"), "24.5/22/18/19 read as percent");
        assert_eq!(probe.wells.len(), 4, "W-A, W-B, GHOST-9, DUP-C (blank cell not a well)");
        assert_eq!(probe.wells[0].name, "W-A");
        assert_eq!(probe.wells[0].rows, 2);

        // --- Commit under the probed mapping, feet → metres, extras as point data. ---
        // CSO_1 (numeric) and LITH (text) are beyond core_data's fixed four measurements:
        // they ride along into aux_data under the confirmed dataset name.
        let mapping = parsers::CoreMapping {
            well: probe.well,
            depth: probe.depth.unwrap(),
            cpor: probe.cpor,
            cperm: probe.cperm,
            cgd: probe.cgd,
            csw: probe.csw,
            extras: vec![6, 9],
        };
        let res = import_core_table(&conn, spath, &mapping, Some("ft"), None, Some("core"), None, false);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.wells_imported, 2, "W-A and W-B only");
        assert_eq!(res.rows_imported, 4);
        assert_eq!(res.skipped_blank_well, 1, "the blank-well row is skipped, never misrouted");
        let ghost = res.outcomes.iter().find(|o| o.well_name == "GHOST-9").unwrap();
        assert!(ghost.problem.as_deref().unwrap_or("").contains("no well"), "unmatched reported");
        let dup = res.outcomes.iter().find(|o| o.well_name == "DUP-C").unwrap();
        assert!(dup.problem.as_deref().unwrap_or("").contains("ambiguous"), "ambiguous reported");

        // Depths landed in METRES (1000 ft = 304.8 m) and porosity in v/v.
        let (d, p): (f32, f32) = conn
            .query_row(
                "SELECT min(depth), min(cpor) FROM core_data WHERE well_id = ?1",
                params![wa.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!((d - 304.8).abs() < 0.05, "feet converted to project metres, got {d}");
        assert!((p - 0.22).abs() < 1e-3, "percent porosity converted to v/v, got {p}");

        // --- Extras: numeric and text, at the SAME converted plug depths, in aux_data. ---
        assert_eq!(res.extra_rows, 7, "2 items x 4 plugs, minus W-B's blank LITH cell");
        assert!(res.extra_items.iter().any(|i| i == "LITH"));
        let aux = db::list_aux_data(&conn, &wa.to_string(), Some("CORE")).unwrap();
        let cso = aux.iter().find(|r| r.item == "CSO_1" && (r.depth_top - 304.8).abs() < 0.05).unwrap();
        assert_eq!(cso.value_num, Some(15.0), "numeric extra stored verbatim (no % conversion)");
        assert!(cso.value_text.is_none());
        let lith = aux.iter().find(|r| r.item == "LITH").unwrap();
        assert_eq!(lith.value_text.as_deref(), Some("SANDSTONE"), "text extra stays text");
        assert!(lith.value_num.is_none());
        assert!(
            aux.iter().all(|r| r.depth_base.is_none()),
            "plug extras are POINT samples, not intervals"
        );
        // Blank cells are skipped, not stored as empty text: W-B's second plug has no LITH.
        let aux_b = db::list_aux_data(&conn, &wb.to_string(), Some("CORE")).unwrap();
        assert_eq!(aux_b.len(), 3, "2 x CSO_1 + 1 x LITH (the blank one skipped): {aux_b:?}");

        std::fs::remove_file(&path).ok();
    }

    /// SB-DIO-047 / T66. §5.4 cites f32 as SandiBumi's sample storage; the LAS writer's
    /// existing format is four decimal places. The deliberately long CPERM decimal loses
    /// one f64-to-f32 value while 0.125 and 1000 remain exact, so a blanket "all values
    /// reduced" implementation fails the first half. The LAS fixture likewise has one
    /// value beyond four decimal places and otherwise exactly writable values.
    #[test]
    fn a_float64_core_import_and_a_four_decimal_las_export_state_their_precision_reductions() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "SANDI-PRECISION", None, None, None).unwrap();
        let well = well_id.to_string();

        let source_cperm = 123.12345678901234_f64;
        let core_path = std::env::temp_dir().join(format!(
            "sandibumi-float64-core-{}.csv",
            std::process::id()
        ));
        std::fs::write(
            &core_path,
            format!("DEPTH,CPOR,CPERM\n1000,0.125,{source_cperm:.14}\n"),
        )
        .unwrap();
        let mapping = parsers::CoreMapping {
            well: None,
            depth: 0,
            cpor: Some(1),
            cperm: Some(2),
            cgd: None,
            csw: None,
            extras: Vec::new(),
        };
        let imported = import_core_table(
            &conn,
            core_path.to_str().unwrap(),
            &mapping,
            None,
            Some(&well),
            None,
            Some("PRECISION"),
            false,
        );
        assert!(imported.error.is_none(), "{:?}", imported.error);
        assert_eq!(imported.precision.source_precision, "f64 numeric parse");
        assert_eq!(imported.precision.destination_precision, "f32 storage");
        assert!(imported.precision.reduced, "the long CPERM value must be declared as narrowed");
        assert_eq!(
            imported.precision.values_reduced, 1,
            "the exact depth and porosity must not be falsely counted as reduced"
        );
        let stored_cperm: f32 = conn
            .query_row(
                "SELECT cperm FROM core_data WHERE well_id = ?1",
                params![well],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored_cperm, source_cperm as f32, "the declared f32 cast is the value stored");
        assert_ne!(stored_cperm as f64, source_cperm, "the fixture must genuinely exceed f32 precision");

        let depth = vec![1000.0f32, 1000.5];
        let exact = vec![2.0f32, 2.5];
        db::insert_standard_curves(
            &conn,
            well_id,
            depth,
            vec![12.34567f32, 12.5],
            exact.clone(),
            vec![0.25f32, 0.5],
            exact.clone(),
            vec![80.0f32, 81.0],
            vec![0.0f32, 0.5],
        )
        .unwrap();
        let las_path = std::env::temp_dir().join(format!(
            "sandibumi-precision-export-{}.las",
            std::process::id()
        ));
        let exported = crate::export::export_las(&conn, &well_id.to_string(), las_path.to_str().unwrap()).unwrap();
        assert_eq!(exported.precision.source_precision, "f32 storage");
        assert_eq!(exported.precision.destination_precision, "fixed-decimal-4 LAS text");
        assert!(exported.precision.reduced, "12.34567 must be declared as rounded on write");
        assert_eq!(
            exported.precision.values_reduced, 1,
            "exactly writable depths and samples must not be falsely counted"
        );
        let las = parsers::read_text_file(&las_path).unwrap();
        assert!(las.contains("SANDIBUMI_PRECISION_V1"), "the file itself must carry the declaration");
        assert!(las.contains("\"values_reduced\":1"), "the deliverable must state the actual loss count");

        std::fs::remove_file(&core_path).ok();
        std::fs::remove_file(&las_path).ok();
    }

    /// Aux import v2 (T-IMP-11): a WELL-columned petrography file routes rows by name;
    /// unmatched names and blank cells are reported, and a file with no well column
    /// still binds wholly to the selected well.
    #[test]
    fn aux_import_routes_by_well_column() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wa = Uuid::new_v4();
        let wb = Uuid::new_v4();
        db::insert_well(&conn, wa, "W-A", None, None, None).unwrap();
        db::insert_well(&conn, wb, "W-B", None, None, None).unwrap();

        let csv = "WELL,TOP,BASE,LITHOLOGY,QUARTZ\n\
                   W-A,1000.0,1002.0,Sandstone,72.1\n\
                   W-B,2000.0,2001.5,Claystone,38.0\n\
                   NOPE-1,3000.0,3001.0,Limestone,5.0\n\
                   ,4000.0,4001.0,Coal,1.0\n";
        let path = std::env::temp_dir().join("sandibumi_aux_v2_test.csv");
        std::fs::write(&path, csv).unwrap();

        let res = import_aux_file(&conn, &wa.to_string(), "PETROGRAPHY", path.to_str().unwrap(), None, false);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.wells_imported, 2, "W-A and W-B routed by the WELL column");
        let notes = res.notes.as_deref().unwrap_or("");
        assert!(notes.contains("NOPE-1"), "unmatched name reported: {notes}");
        assert!(notes.contains("blank well cell"), "blank-cell skip reported: {notes}");
        // W-B's rows must have gone to W-B, NOT the selected fallback well.
        let n_b: i64 = conn
            .query_row(
                "SELECT count(*) FROM aux_data WHERE well_id = ?1 AND dataset = 'PETROGRAPHY'",
                params![wb.to_string()],
                |r| r.get(0),
            )
            .unwrap();
        assert!(n_b > 0, "W-B received its own rows");
        std::fs::remove_file(&path).ok();
    }

    /// Import sets (T-IMP-02): a second delivery of the SAME well attaches as a named set
    /// on the ONE existing record instead of creating a duplicate; a third lands beside it
    /// auto-suffixed; and the resolver reaches attached-set curves while RAW keeps
    /// absolute priority for anything it already carries.
    #[test]
    fn import_sets_attach_suffix_and_resolution() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();

        // Delivery 1 (RAW): the field print — GR 55 everywhere.
        let raw_las = "~Version\nVERS. 2.0 :\n~Well\nWELL. SETW-1 :\n\
                       ~Curve\nDEPT .M : depth\nGR .GAPI : gamma\nPEF .B/E : pe\n\
                       ~ASCII\n1000.0 55.0 5.1\n1000.5 55.0 5.2\n1001.0 55.0 5.0\n";
        // Delivery 2 (FPROOH): same well, a reprocessed GR (99 — must NOT shadow RAW's)
        // plus a curve RAW does not have (PHIFF — must resolve from here).
        let fp_las = "~Version\nVERS. 2.0 :\n~Well\nWELL. SETW-1 :\n\
                      ~Curve\nDEPT .M : depth\nGR .GAPI : gamma\nPHIFF .V/V : free fluid\n\
                      ~ASCII\n1000.0 99.0 0.21\n1000.5 99.0 0.22\n1001.0 99.0 0.23\n";
        let p1 = std::env::temp_dir().join("sandibumi_set_raw_test.las");
        let p2 = std::env::temp_dir().join("sandibumi_set_fprooh_test.las");
        std::fs::write(&p1, raw_las).unwrap();
        std::fs::write(&p2, fp_las).unwrap();
        let attach = |set: &str| LasImportOptions {
            set_name: Some(set.into()),
            attach: true,
            ..Default::default()
        };

        // 1. First import creates the well (attach on, but nothing to attach to).
        let r1 = &import_las_files_with(&conn, &[p1.to_str().unwrap().into()], None, &attach("RAW"))[0];
        assert!(r1.error.is_none(), "{:?}", r1.error);
        assert!(r1.attached_set.is_none(), "a fresh well is created, not attached");
        let well_id = r1.well_id.clone().unwrap();

        // 2. Second delivery ATTACHES to that record as set FPROOH — still ONE well.
        let r2 = &import_las_files_with(&conn, &[p2.to_str().unwrap().into()], None, &attach("FPROOH"))[0];
        assert!(r2.error.is_none(), "{:?}", r2.error);
        assert_eq!(r2.attached_set.as_deref(), Some("FPROOH"));
        assert_eq!(r2.well_id.as_deref(), Some(well_id.as_str()), "attached to the SAME record");
        let n_wells: i64 = conn.query_row("SELECT COUNT(*) FROM wells", [], |r| r.get(0)).unwrap();
        assert_eq!(n_wells, 1, "no duplicate well record");

        // 3. Re-importing the same delivery auto-suffixes (Geolog WIRE → WIRE_1), never overwrites.
        let r3 = &import_las_files_with(&conn, &[p2.to_str().unwrap().into()], None, &attach("FPROOH"))[0];
        assert_eq!(r3.attached_set.as_deref(), Some("FPROOH_1"));
        let catalog = db::list_generic_curve_catalog(&conn, &well_id).unwrap();
        let mut sets: Vec<&str> = catalog.iter().map(|c| c.set_name.as_str()).collect();
        sets.sort();
        sets.dedup();
        assert_eq!(sets, vec!["FPROOH", "FPROOH_1", "RAW"]);

        // 4. Resolution: RAW keeps absolute priority (GR = 55, not FPROOH's 99), and a
        //    mnemonic RAW lacks (PHIFF) resolves from the attached set.
        let (_grid, cols) =
            crate::equations::fetch_curve_frame(&conn, &well_id, &["GR".into(), "PHIFF".into()]).unwrap();
        assert!(cols["GR"].iter().all(|&v| (v - 55.0).abs() < 1e-3), "RAW GR must win: {:?}", cols["GR"]);
        assert!(
            cols["PHIFF"].iter().zip([0.21f32, 0.22, 0.23]).all(|(&v, e)| (v - e).abs() < 1e-3),
            "PHIFF must resolve from the attached FPROOH set: {:?}",
            cols["PHIFF"]
        );

        std::fs::remove_file(&p1).ok();
        std::fs::remove_file(&p2).ok();
    }

    /// #118 follow-up: a file whose (unrecognized) index column is entirely the null sentinel
    /// leaves zero rows after depth sanitation. That must ERROR — not commit a curve-less
    /// orphan well — and must create no wells row.
    #[test]
    fn all_null_depth_las_errors_without_creating_well() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();

        // XREF (unrecognized index) at column 0, every value the -999.25 null sentinel.
        let las = "~Version\nVERS. 2.0 :\n~Well\nNULL. -999.25 :\n\
                   ~Curve\nXREF .M : idx\nGR .GAPI : gamma\n\
                   ~ASCII\n-999.25 55.0\n-999.25 60.0\n";
        let path = std::env::temp_dir().join("arshilla_allnull_depth_test.las");
        std::fs::write(&path, las).unwrap();

        let results = import_las_files(&conn, &[path.to_str().unwrap().to_string()], None);
        std::fs::remove_file(&path).ok();

        let r = &results[0];
        assert!(r.error.is_some(), "all-null depth must error, not create an empty well");
        assert!(r.well_id.is_none(), "no well_id on the errored import");
        let n_wells: i64 = conn.query_row("SELECT COUNT(*) FROM wells", [], |r| r.get(0)).unwrap();
        assert_eq!(n_wells, 0, "no orphan well row created");
    }

    /// Well-locations CSV: alias-resolved EASTING/NORTHING/ZONE headers, name→well match,
    /// per-row zone overriding the dialog default, unmatched names reported, and re-import
    /// overwriting a previous fix.
    #[test]
    fn locations_import_matches_zones_and_overwrites() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        db::insert_well(&conn, a, "MHK-1", Some("Mahakam"), None, None).unwrap();
        db::insert_well(&conn, b, "MHK-2", Some("Mahakam"), None, None).unwrap();

        // MHK-1 carries its own zone column value; MHK-2's is blank → dialog default; the
        // third row's well isn't in the project → unmatched. Southern-hemisphere northings.
        let path = std::env::temp_dir().join(format!("arshilla_loc_{a}.csv"));
        std::fs::write(
            &path,
            "WELL,EASTING,NORTHING,ZONE\nMHK-1,485000.0,9450000.0,50S\nMHK-2,486200.5,9451750.0,\nGHOST,1,2,50S\n",
        )
        .unwrap();

        let res = import_locations_file(&conn, None, Some("50M"), path.to_str().unwrap());
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.wells_located, 2);
        assert_eq!(res.unmatched_wells, vec!["GHOST".to_string()]);

        let read = |id: &Uuid| -> (f64, f64, String) {
            conn.query_row(
                "SELECT surface_x, surface_y, utm_zone FROM wells WHERE well_id = ?1",
                params![id.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get::<_, String>(2)?)),
            )
            .unwrap()
        };
        let (x1, y1, z1) = read(&a);
        assert!((x1 - 485000.0).abs() < 1e-6 && (y1 - 9450000.0).abs() < 1e-6);
        assert_eq!(z1, "50S", "explicit ZONE cell wins");
        let (_, _, z2) = read(&b);
        assert_eq!(z2, "50M", "blank ZONE cell falls back to the dialog default");

        // Re-import with a new easting overwrites rather than erroring or duplicating.
        std::fs::write(&path, "WELL,X,Y\nMHK-1,490000.0,9460000.0\n").unwrap();
        let res2 = import_locations_file(&conn, None, Some("50S"), path.to_str().unwrap());
        assert!(res2.error.is_none());
        assert_eq!(res2.wells_located, 1);
        let (x1b, _, _) = read(&a);
        assert!((x1b - 490000.0).abs() < 1e-6, "re-import overwrote the location");
        std::fs::remove_file(&path).ok();

        // A file with no X/Y column fails cleanly.
        let bad = std::env::temp_dir().join(format!("arshilla_loc_bad_{a}.csv"));
        std::fs::write(&bad, "WELL,DEPTH\nMHK-1,1000\n").unwrap();
        let res3 = import_locations_file(&conn, None, None, bad.to_str().unwrap());
        assert!(res3.error.is_some(), "missing coordinate columns must error");
        std::fs::remove_file(&bad).ok();
    }

    /// A blank WELL cell in a multi-well file must NOT be routed to the selected well (that
    /// would silently corrupt an unrelated well's surface location) — it is skipped and
    /// surfaced. The headerless single-well fallback must still route to the selected well.
    #[test]
    fn locations_import_skips_blank_well_cell_not_default() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        db::insert_well(&conn, a, "MHK-1", Some("Mahakam"), None, None).unwrap();
        db::insert_well(&conn, b, "MHK-2", Some("Mahakam"), None, None).unwrap();

        // Multi-well file (HAS a WELL column) whose 2nd row's WELL cell is blank but carries
        // valid coordinates. MHK-1 is "selected" (default_well_id = a); the blank row must
        // not overwrite MHK-1 — MHK-1 never appears in the file.
        let path = std::env::temp_dir().join(format!("arshilla_locblank_{a}.csv"));
        std::fs::write(&path, "WELL,EASTING,NORTHING\nMHK-2,486000.0,9451000.0\n,999999.0,888888.0\n").unwrap();
        let res = import_locations_file(&conn, Some(&a.to_string()), Some("50S"), path.to_str().unwrap());
        std::fs::remove_file(&path).ok();

        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.wells_located, 1, "only MHK-2 located; the blank row is skipped, not routed to MHK-1");
        assert!(
            res.unmatched_wells.iter().any(|s| s.contains("blank")),
            "blank row must be surfaced, got {:?}",
            res.unmatched_wells
        );
        let x1: Option<f64> = conn
            .query_row("SELECT surface_x FROM wells WHERE well_id = ?1", params![a.to_string()], |r| r.get(0))
            .unwrap();
        assert!(x1.is_none(), "selected well must be untouched by a blank-WELL row, got {x1:?}");
        let x2: f64 = conn
            .query_row("SELECT surface_x FROM wells WHERE well_id = ?1", params![b.to_string()], |r| r.get(0))
            .unwrap();
        assert!((x2 - 486000.0).abs() < 1e-6, "MHK-2 located from its named row");

        // A genuinely headerless (no WELL column) single-well file still routes to the
        // selected well — the fallback the fix must NOT break.
        let path2 = std::env::temp_dir().join(format!("arshilla_locnohdr_{a}.csv"));
        std::fs::write(&path2, "EASTING,NORTHING\n500000.0,9400000.0\n").unwrap();
        let res2 = import_locations_file(&conn, Some(&a.to_string()), Some("50S"), path2.to_str().unwrap());
        std::fs::remove_file(&path2).ok();
        assert!(res2.error.is_none(), "{:?}", res2.error);
        assert_eq!(res2.wells_located, 1, "headerless file routes to the selected well");
        let x1b: f64 = conn
            .query_row("SELECT surface_x FROM wells WHERE well_id = ?1", params![a.to_string()], |r| r.get(0))
            .unwrap();
        assert!((x1b - 500000.0).abs() < 1e-6, "selected well located from the headerless file");
    }

    /// SCAL Pc CSV import: alias headers, percent Sw/poro detection, replace-on-reimport,
    /// and the Leverett-J fit coming back with the import result.
    #[test]
    fn scal_import_fits_leverett_j() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "SCAL-1", None, None, None).unwrap();
        let ids = well_id.to_string();

        // Synthesize points on Sw = 0.4 * J^-0.5 at IFT 72 (like the satheight unit test),
        // but delivered the way labs do: Sw in percent, headers with units.
        let mut body = String::from("Sample,Depth (m),Kair (mD),CPOR (%),Pc (psi),Sw (%)\n");
        for i in 1..=12 {
            let pc = i as f64 * 3.0;
            let j = 0.21645 * pc / 72.0 * (150.0f64 / 0.22).sqrt();
            let sw = (0.4 * j.powf(-0.5)).min(1.0) * 100.0;
            body.push_str(&format!("1,2000.5,150,22,{pc},{sw:.2}\n"));
        }
        let path = std::env::temp_dir().join(format!("arshilla_scal_test_{ids}.csv"));
        std::fs::write(&path, &body).unwrap();

        let res = import_scal_csv(&conn, &ids, path.to_str().unwrap(), 72.0);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.rows, 12);
        let fit = res.fit.expect("fit should solve");
        assert!((fit.b - -0.5).abs() < 0.05, "b={}", fit.b);
        assert!((fit.a - 0.4).abs() < 0.1, "a={}", fit.a);

        // Re-import replaces rather than duplicates; rows readable back.
        let res2 = import_scal_csv(&conn, &ids, path.to_str().unwrap(), 72.0);
        std::fs::remove_file(&path).ok();
        assert_eq!(res2.rows, 12);
        let rows = db::get_scal_pc(&conn, &ids).unwrap();
        assert_eq!(rows.len(), 12);
        assert!((rows[0].poro - 0.22).abs() < 1e-4, "percent poro converted to v/v");
        assert!(rows.iter().all(|r| r.sw <= 1.0), "percent Sw converted to v/v");

        // Unknown well errors cleanly.
        let bad = import_scal_csv(&conn, "nope", "x.csv", 72.0);
        assert!(bad.error.is_some());
    }

    /// Multi-file SCAL import (increment 2): two single-plug centrifuge exports sniffed
    /// by "auto" land in one combined replace-write; a later porous-plate import REPLACES
    /// them (not appends); a bad file fails the whole import with the filename named.
    #[test]
    fn scal_import_files_multi_format_and_replace() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "SCAL-2", None, None, None).unwrap();
        let ids = well_id.to_string();

        let cf = |sample: &str, depth: f32| {
            format!(
                "SAMPLE,{sample}\nDEPTH,{depth}\nPERM,45.0\nPORO,18.0\n\
                 Speed (RPM),Pc (psi),Sw (%PV)\n500,2.1,95.0\n1000,8.4,78.2\n2000,33.6,55.4\n4000,120.0,41.0\n"
            )
        };
        let p1 = std::env::temp_dir().join(format!("sandibumi_scal_cf1_{ids}.csv"));
        let p2 = std::env::temp_dir().join(format!("sandibumi_scal_cf2_{ids}.csv"));
        std::fs::write(&p1, cf("12A", 2695.3)).unwrap();
        std::fs::write(&p2, cf("S-16A", 2701.8)).unwrap();
        let paths = vec![p1.to_str().unwrap().to_string(), p2.to_str().unwrap().to_string()];

        let res = import_scal_files(&conn, &ids, &paths, "auto", "air_brine", 72.0, None, false);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.rows, 8, "both plugs land in one combined import");
        assert!(res.fit.is_some(), "J-fit solves over the pooled points");
        let rows = db::get_scal_pc(&conn, &ids).unwrap();
        assert_eq!(rows.len(), 8);
        assert!(rows.iter().any(|r| r.sample_no == Some(12)) && rows.iter().any(|r| r.sample_no == Some(16)));
        assert!(
            rows.iter().all(|r| r.system.as_deref() == Some("air_brine") && r.ift == Some(72.0)),
            "fluid system + IFT stored on every point"
        );

        // A porous-plate re-import is a SECOND delivery: the centrifuge report is kept,
        // the new one goes live, and a reader still sees exactly one delivery's points.
        let wide = "Sample,Depth (m),Perm (mD),Poro (%),1,2,4,8\n4,2001.5,150.0,22.5,98.5,95.2,88.1,79.4\n";
        let p3 = std::env::temp_dir().join(format!("sandibumi_scal_pp_{ids}.csv"));
        std::fs::write(&p3, wide).unwrap();
        let res2 =
            import_scal_files(&conn, &ids, &[p3.to_str().unwrap().to_string()], "porous_plate", "air_brine", 72.0, None, false);
        assert!(res2.error.is_none(), "{:?}", res2.error);
        assert_eq!(res2.rows, 4);
        assert_eq!(res2.set_name.as_deref(), Some("SCAL_1"), "auto-suffixed, first report kept");
        assert_eq!(db::get_scal_pc(&conn, &ids).unwrap().len(), 4, "one delivery read, never both merged");
        let scal_sets = db::list_scal_sets(&conn, &ids).unwrap();
        assert_eq!(scal_sets.len(), 2, "both reports on the well: {scal_sets:?}");
        assert!(scal_sets[0].active && scal_sets[0].set_name == "SCAL_1" && scal_sets[0].rows == 4);
        assert!(!scal_sets[1].active && scal_sets[1].rows == 8, "the centrifuge report is intact");
        // Switching back restores the centrifuge points wholesale.
        db::set_active_scal_set(&conn, &ids, "SCAL").unwrap();
        assert_eq!(db::get_scal_pc(&conn, &ids).unwrap().len(), 8);
        db::set_active_scal_set(&conn, &ids, "SCAL_1").unwrap();

        // One bad file fails the whole import and names the file.
        let res3 = import_scal_files(
            &conn,
            &ids,
            &[p1.to_str().unwrap().to_string(), "missing_dir/nope.csv".to_string()],
            "auto",
            "air_brine",
            72.0,
            None,
            false,
        );
        assert!(res3.error.as_deref().is_some_and(|e| e.contains("nope.csv")));
        assert_eq!(db::get_scal_pc(&conn, &ids).unwrap().len(), 4, "failed import leaves prior rows intact");

        for p in [&p1, &p2, &p3] {
            std::fs::remove_file(p).ok();
        }
    }

    /// Post-review hardening: a structurally-valid file that parses to ZERO points must
    /// refuse the replace-write instead of silently wiping the well's existing SCAL data.
    #[test]
    fn scal_import_zero_rows_leaves_existing_data() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "SCAL-3", None, None, None).unwrap();
        let ids = well_id.to_string();

        let good = std::env::temp_dir().join(format!("sandibumi_scal_good_{ids}.csv"));
        std::fs::write(&good, "PC,SW\n5,0.55\n10,0.45\n20,0.35\n").unwrap();
        let res = import_scal_files(&conn, &ids, &[good.to_str().unwrap().to_string()], "long", "hg_air", 367.0, None, false);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(db::get_scal_pc(&conn, &ids).unwrap().len(), 3);

        // Header-only export (e.g. a filtered/template sheet) → error, data intact.
        let empty = std::env::temp_dir().join(format!("sandibumi_scal_empty_{ids}.csv"));
        std::fs::write(&empty, "PC,SW\n").unwrap();
        let res2 = import_scal_files(&conn, &ids, &[empty.to_str().unwrap().to_string()], "auto", "hg_air", 367.0, None, false);
        assert!(res2.error.as_deref().is_some_and(|e| e.contains("untouched")), "{:?}", res2.error);
        assert_eq!(db::get_scal_pc(&conn, &ids).unwrap().len(), 3, "existing points survive");

        for p in [&good, &empty] {
            std::fs::remove_file(p).ok();
        }
    }

    /// P2 tops import: multi-well CSV matches wells by name, no-well-column file needs
    /// the selected well, re-import updates depth but keeps an existing color.
    #[test]
    fn tops_import_multiwell_and_default() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let w1 = Uuid::new_v4();
        let w2 = Uuid::new_v4();
        db::insert_well(&conn, w1, "SANDI-1", None, None, None).unwrap();
        db::insert_well(&conn, w2, "SANDI-2", None, None, None).unwrap();
        let id1 = w1.to_string();

        let path = std::env::temp_dir().join("arshilla_tops_import.csv");
        std::fs::write(
            &path,
            "WELL,TOP,MD\nsandi-1,TOP_A,1000.0\nSANDI-1,TOP_B,1100.0\nSANDI-2,TOP_A,1010.0\nGHOST-9,TOP_A,900.0\n",
        )
        .unwrap();
        let res = import_tops_file(&conn, None, path.to_str().unwrap());
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.tops_written, 3);
        assert_eq!(res.wells_matched, 2, "case-insensitive well matching");
        assert_eq!(res.unmatched_wells, vec!["GHOST-9".to_string()]);

        // Give TOP_A a color, then re-import a new depth: depth moves, color survives.
        db::upsert_top(&conn, &id1, "TOP_A", 1000.0, Some("#ff0000")).unwrap();
        std::fs::write(&path, "WELL,TOP,MD\nSANDI-1,TOP_A,1005.0\n").unwrap();
        let res2 = import_tops_file(&conn, None, path.to_str().unwrap());
        assert!(res2.error.is_none());
        let tops = db::list_tops(&conn, &id1).unwrap();
        let a = tops.iter().find(|t| t.top_name == "TOP_A").unwrap();
        assert!((a.depth - 1005.0).abs() < 1e-3, "re-import updates depth");
        assert_eq!(a.color.as_deref(), Some("#ff0000"), "existing color kept");

        // No WELL column: needs a default well; with one it lands there.
        std::fs::write(&path, "TOP,DEPTH\nTOP_C,1200.0\n").unwrap();
        let need = import_tops_file(&conn, None, path.to_str().unwrap());
        assert!(need.error.is_some(), "no well column and no selection must error");
        let ok = import_tops_file(&conn, Some(&id1), path.to_str().unwrap());
        assert!(ok.error.is_none());
        assert!(db::list_tops(&conn, &id1).unwrap().iter().any(|t| t.top_name == "TOP_C"));
        std::fs::remove_file(&path).ok();
    }

    /// T-IMP-10's remaining half: a BLANK cell in the WELL column.
    ///
    /// `tops_import_multiwell_and_default` already covers multi-well routing, case-insensitive
    /// matching, unmatched wells and the no-WELL-column file. A blank cell is a different thing
    /// from an unmatched name, and it is the common one — spreadsheets carry a merged or
    /// forward-filled well column, and whoever exported it left the repeats empty.
    ///
    /// **A blank must never fall through to the selected well.** That is the tempting reading —
    /// "no well named, so use the one they picked" — and it is how a marker from a different well
    /// ends up on this one. There is nothing on the log to catch it: the top lands at a plausible
    /// depth, it just belongs to somebody else's well, and every zone below it is then wrong.
    /// Skipped and counted is the only safe answer.
    #[test]
    fn a_blank_well_cell_is_skipped_rather_than_charged_to_the_selected_well() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let w1 = Uuid::new_v4();
        db::insert_well(&conn, w1, "SANDI-10", None, None, None).unwrap();
        let id1 = w1.to_string();

        let path = std::env::temp_dir().join("sandibumi_tops_blank_well.csv");
        std::fs::write(
            &path,
            "WELL,TOP,MD\nSANDI-10,TOP_A,1000.0\n,TOP_ORPHAN,1100.0\n   ,TOP_SPACES,1200.0\n",
        )
        .unwrap();
        // A default well IS supplied — the case where falling through would be silent.
        let res = import_tops_file(&conn, Some(&id1), path.to_str().unwrap());
        std::fs::remove_file(&path).ok();
        assert!(res.error.is_none(), "{:?}", res.error);

        let tops = db::list_tops(&conn, &id1).unwrap();
        let names: Vec<&str> = tops.iter().map(|t| t.top_name.as_str()).collect();
        assert!(names.contains(&"TOP_A"), "the named row must still land: {names:?}");
        assert!(
            !names.contains(&"TOP_ORPHAN") && !names.contains(&"TOP_SPACES"),
            "a blank WELL cell was charged to the selected well — a marker from an unknown well \
             is now sitting on SANDI-10 and nothing downstream can tell: {names:?}"
        );
        assert_eq!(res.tops_written, 1, "one row had a well, so one top was written");
    }

    /// P2 aux import: XRD point data (numeric + text cells) and perforation intervals
    /// land in aux_data long format; datasets are independent; and every dataset follows
    /// the SET discipline (a re-delivery is kept beside the first, one is live).
    #[test]
    fn aux_import_xrd_and_perforation() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let w = Uuid::new_v4();
        db::insert_well(&conn, w, "AUX-1", None, None, None).unwrap();
        let ids = w.to_string();

        let xrd = std::env::temp_dir().join("arshilla_aux_xrd.csv");
        std::fs::write(&xrd, "Depth,Quartz,Illite,Remarks\n2000.0,45.2,12.1,clean\n2001.0,40.0,,silty\n").unwrap();
        let res = import_aux_file(&conn, &ids, "xrd", xrd.to_str().unwrap(), None, false);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.dataset, "XRD", "dataset name normalized upper");
        assert_eq!(res.rows, 5, "empty cell is skipped, text cell kept");

        let rows = db::list_aux_data(&conn, &ids, Some("XRD")).unwrap();
        assert_eq!(rows.len(), 5);
        let quartz0 = rows.iter().find(|r| r.item == "QUARTZ" && r.depth_top == 2000.0).unwrap();
        assert!((quartz0.value_num.unwrap() - 45.2).abs() < 1e-3);
        assert!(quartz0.value_text.is_none());
        let remark = rows.iter().find(|r| r.item == "REMARKS" && r.depth_top == 2001.0).unwrap();
        assert_eq!(remark.value_text.as_deref(), Some("silty"));
        assert!(remark.value_num.is_none());

        // Perforation intervals in a second dataset; both coexist.
        let perf = std::env::temp_dir().join("arshilla_aux_perf.csv");
        std::fs::write(&perf, "FROM,TO,STATUS\n2050.0,2055.0,OPEN\n2100.0,2104.0,SQUEEZED\n").unwrap();
        let res2 = import_aux_file(&conn, &ids, "PERFORATION", perf.to_str().unwrap(), None, false);
        assert!(res2.error.is_none());
        assert_eq!(res2.rows, 2);
        let perfs = db::list_aux_data(&conn, &ids, Some("PERFORATION")).unwrap();
        assert_eq!(perfs[0].depth_base, Some(2055.0), "interval BASE kept");
        assert_eq!(perfs[1].value_text.as_deref(), Some("SQUEEZED"));
        let sets = db::list_aux_datasets(&conn, &ids).unwrap();
        assert_eq!(sets, vec![("PERFORATION".to_string(), 2i64), ("XRD".to_string(), 5i64)]);

        // A SECOND XRD delivery: kept beside the first (never overwritten), live, and
        // counted alone — the whole point of the set model applied to point data.
        std::fs::write(&xrd, "Depth,Quartz\n2000.0,50.0\n").unwrap();
        let res3 = import_aux_file(&conn, &ids, "XRD", xrd.to_str().unwrap(), None, false);
        assert!(res3.error.is_none());
        assert_eq!(res3.sets, vec!["RAW_1".to_string()], "auto-suffixed, not overwritten");
        let counts = db::list_aux_datasets(&conn, &ids).unwrap();
        assert_eq!(
            counts,
            vec![("PERFORATION".to_string(), 2i64), ("XRD".to_string(), 1i64)],
            "counts follow the ACTIVE delivery — never the sum of both"
        );
        let aux_sets = db::list_aux_sets(&conn, &ids).unwrap();
        let xrd_sets: Vec<_> = aux_sets.iter().filter(|s| s.dataset == "XRD").collect();
        assert_eq!(xrd_sets.len(), 2, "both XRD deliveries kept: {aux_sets:?}");
        assert!(xrd_sets[0].active && xrd_sets[0].set_name == "RAW_1");
        assert!(!xrd_sets[1].active && xrd_sets[1].rows == 5, "the first delivery is intact");
        // Perforation is a different dataset and is untouched by the XRD switch.
        assert!(aux_sets.iter().any(|s| s.dataset == "PERFORATION" && s.active && s.rows == 2));

        // Switching back restores the first delivery's rows, wholesale.
        db::set_active_aux_set(&conn, &ids, "XRD", "RAW").unwrap();
        assert_eq!(db::list_aux_data(&conn, &ids, Some("XRD")).unwrap().len(), 5);
        // Deleting the live delivery hands over to the survivor.
        db::delete_aux_set(&conn, &ids, "XRD", "RAW").unwrap();
        let rows = db::list_aux_data(&conn, &ids, Some("XRD")).unwrap();
        assert_eq!(rows.len(), 1, "the remaining delivery became live: {rows:?}");
        std::fs::remove_file(&xrd).ok();
        std::fs::remove_file(&perf).ok();

        // Unknown well errors cleanly.
        let bad = import_aux_file(&conn, "nope", "XRD", "x.csv", None, false);
        assert!(bad.error.is_some());
    }

    /// SCAL plugs ARE core plugs, so their depths are the core report's depths and must be able to
    /// follow the same correction.
    #[test]
    fn scal_points_can_follow_the_core_they_were_cut_from() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-SCAL-FOLLOW", None, None, None).unwrap();
        let w = wid.to_string();

        let d: Vec<f32> = (0..20).map(|i| 2000.0 + i as f32).collect();
        let v = vec![0.2f32; 20];
        let nan = vec![f32::NAN; 20];
        db::insert_core_data(&conn, &w, "RAW", None, &d, &v, &nan, &nan, &nan).unwrap();
        db::apply_core_run_shifts(&mut conn, &w, &[db::RunShift { top: 2000.0, base: 2019.0, delta: 2.0, ..Default::default() }], &Default::default(), &Default::default())
            .unwrap();

        let path = std::env::temp_dir().join("sandi_scal_follow.csv");
        std::fs::write(
            &path,
            "SAMPLE,DEPTH,PERM,PORO,PC,SW\n1,2005,100,0.20,1,1.0\n1,2005,100,0.20,10,0.5\n\
             2,2010,50,0.18,1,1.0\n2,2010,50,0.18,10,0.6\n",
        )
        .unwrap();
        let p = path.to_str().unwrap().to_string();

        let res = import_scal_files(&conn, &w, &[p.clone()], "long", "air_brine", 72.0, Some("FOLLOWED"), true);
        assert!(res.error.is_none(), "{:?}", res.error);
        let rows = db::get_scal_pc(&conn, &w).unwrap();
        let depths: Vec<f32> = {
            let mut d: Vec<f32> = rows.iter().filter_map(|r| r.depth).collect();
            d.sort_by(f32::total_cmp);
            d.dedup_by(|a, b| (*a - *b).abs() < 1e-3);
            d
        };
        assert_eq!(depths.len(), 2, "{depths:?}");
        assert!((depths[0] - 2007.0).abs() < 1e-3, "2005 + 2 m: {depths:?}");
        assert!((depths[1] - 2012.0).abs() < 1e-3, "2010 + 2 m: {depths:?}");
        assert_eq!(res.note.as_deref(), Some("placed from the core depth record"));

        // Off, the depths stay exactly as the file wrote them.
        let plain = import_scal_files(&conn, &w, &[p], "long", "air_brine", 72.0, Some("ASWRITTEN"), false);
        assert!(plain.error.is_none(), "{:?}", plain.error);
        assert!(plain.note.is_none(), "nothing to report when the box was not ticked");
        let rows = db::get_scal_pc(&conn, &w).unwrap();
        assert!(
            rows.iter().any(|r| r.depth.is_some_and(|d| (d - 2005.0).abs() < 1e-3)),
            "unmapped import keeps the delivered depth"
        );
        std::fs::remove_file(&path).ok();
    }

    /// The whole point of keeping the core's as-delivered depths: a laboratory sends XRD months
    /// after the core was registered, still written at the depths from the original core report,
    /// **A table claiming no core measurement never becomes a core delivery.**
    ///
    /// `insert_core_data` registers the set and makes it ACTIVE, so importing an XRD or CEC table
    /// through Intake — which routes everything through `import_core_table` — would have replaced
    /// the well's real plugs with a set of empty ones. Nothing on screen would show it, and every
    /// core reader follows the active set, so the phi-k cloud, Plug QC, Register Depth and the
    /// S-factor fit would all go quiet at once.
    ///
    /// Found by the follow-core test below, whose own first import silently replaced the core it
    /// was about to follow.
    #[test]
    fn a_point_data_table_does_not_displace_the_wells_core() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-NOTCORE", None, None, None).unwrap();
        let w = wid.to_string();

        let d: Vec<f32> = (0..10).map(|i| 2000.0 + i as f32).collect();
        let por = vec![0.2f32; 10];
        let nan = vec![f32::NAN; 10];
        db::insert_core_data(&conn, &w, "PLUGS", None, &d, &por, &nan, &nan, &nan).unwrap();

        let path_buf = std::env::temp_dir().join("sandi_pointdata_only.csv");
        std::fs::write(&path_buf, "DEPTH,CEC
2003,4.5
2007,6.5
").unwrap();
        let path = path_buf.to_str().unwrap();
        let mapping = crate::intake::mapping_from_roles(&["DEPTH".into(), "CEC".into()]).unwrap();
        let res = import_core_table(&conn, path, &mapping, None, Some(&w), Some("CEC"), Some("LAB"), false);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert!(res.extra_rows > 0, "the measurements themselves must still be stored");

        // The core is untouched: the live delivery is still the plugs, with their porosity.
        // Read through ACTIVE_CORE_SET, which is what every core reader in the app follows.
        let sets = db::list_core_sets(&conn, &w).unwrap();
        let live = sets.iter().find(|s| s.active).expect("the well must still have a live core set");
        let (n_live, live_set) = (live.rows as i64, live.set_name.clone());
        assert_eq!(n_live, 10, "the well's plugs are still the active core delivery");
        assert_eq!(live_set, "PLUGS", "and a CEC table did not become one");

        // The control: a table that DOES carry a core measurement still creates a core delivery,
        // so this is not a blanket refusal to import core.
        let path2 = std::env::temp_dir().join("sandi_realcore.csv");
        std::fs::write(&path2, "DEPTH,CPOR
2100,0.3
2101,0.31
").unwrap();
        let m2 = crate::intake::mapping_from_roles(&["DEPTH".into(), "CPOR".into()]).unwrap();
        let res2 = import_core_table(&conn, path2.to_str().unwrap(), &m2, None, Some(&w), None, Some("RUN2"), false);
        assert!(res2.error.is_none(), "{:?}", res2.error);
        assert_eq!(res2.rows_imported, 2, "a real core table is still a core delivery");
    }

    /// **Intake honours "these depths came from the core report" too.** The pane offered the
    /// tick-box from the day it shipped and `IntakeCommit` carried the field — and
    /// `import_core_table` never took it, so the setting was read from the form, sent over IPC and
    /// dropped. A user who ticked it got the delivered depths back with nothing to say so.
    ///
    /// That matters now that Intake is the only route for point data: the aux import dialog it
    /// replaces DID follow the core (Jauhar, 2026-08-05: *"capabilites that already aux have,
    /// intake also should have it"*), so removing that dialog without this would have quietly
    /// taken the capability away.
    #[test]
    fn a_table_imported_through_intake_can_follow_the_core_it_was_measured_on() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-INTAKE", None, None, None).unwrap();
        let w = wid.to_string();

        // Plugs 2000-2019 as delivered, then registered: upper half +1, lower half +3.
        let d: Vec<f32> = (0..20).map(|i| 2000.0 + i as f32).collect();
        let v = vec![0.2f32; 20];
        let nan = vec![f32::NAN; 20];
        db::insert_core_data(&conn, &w, "RAW", None, &d, &v, &nan, &nan, &nan).unwrap();
        db::apply_core_run_shifts(
            &mut conn,
            &w,
            &[
                db::RunShift { top: 2000.0, base: 2009.0, delta: 1.0, ..Default::default() },
                db::RunShift { top: 2010.0, base: 2019.0, delta: 3.0, ..Default::default() },
            ],
            &db::ShiftTargets::default(),
            &Default::default(),
        )
        .unwrap();

        let path_buf = std::env::temp_dir().join("sandi_intake_followcore.csv");
        // A lab table at the ORIGINAL core-report depths: one sample per barrel.
        std::fs::write(&path_buf, "DEPTH,CEC
2005,4.5
2015,6.5
").unwrap();
        let path = path_buf.to_str().unwrap();
        let mapping = crate::intake::mapping_from_roles(&["DEPTH".into(), "CEC".into()]).unwrap();

        // Off: the samples land where the file says, which is now the wrong rock.
        let plain = import_core_table(&conn, path, &mapping, None, Some(&w), Some("CEC_ASWRITTEN"), Some("A"), false);
        assert!(plain.error.is_none(), "{:?}", plain.error);
        let rows = db::list_aux_data(&conn, &w, Some("CEC_ASWRITTEN")).unwrap();
        assert!(rows.iter().any(|r| (r.depth_top - 2005.0).abs() < 1e-3), "unmapped keeps the delivered depth");

        // On: each sample follows the barrel it was cut from — +1 above, +3 below.
        let followed = import_core_table(&conn, path, &mapping, None, Some(&w), Some("CEC_FOLLOWED"), Some("B"), true);
        assert!(followed.error.is_none(), "{:?}", followed.error);
        let rows = db::list_aux_data(&conn, &w, Some("CEC_FOLLOWED")).unwrap();
        let at = |want: f32| rows.iter().any(|r| (r.depth_top - want).abs() < 1e-3);
        assert!(at(2006.0), "the upper barrel's sample moves +1: {:?}", rows.iter().map(|r| r.depth_top).collect::<Vec<_>>());
        assert!(at(2018.0), "and the lower barrel's moves +3, not by the upper barrel's correction");

        // And the run SAYS it followed, rather than leaving the user to infer it.
        assert!(
            followed.outcomes.iter().any(|o| o.problem.as_deref().is_some_and(|p| p.contains("core depth record"))),
            "the mapping must be reported: {:?}",
            followed.outcomes
        );
    }

    /// and those samples land on the rock they were measured from.
    #[test]
    fn a_late_delivery_can_follow_the_core_it_was_measured_on() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-LATE", None, None, None).unwrap();
        let w = wid.to_string();

        // Plugs 2000–2019 as delivered, then registered: upper half +1, lower half +3.
        let d: Vec<f32> = (0..20).map(|i| 2000.0 + i as f32).collect();
        let v = vec![0.2f32; 20];
        let nan = vec![f32::NAN; 20];
        db::insert_core_data(&conn, &w, "RAW", None, &d, &v, &nan, &nan, &nan).unwrap();
        db::apply_core_run_shifts(
            &mut conn,
            &w,
            &[
                db::RunShift { top: 2000.0, base: 2009.0, delta: 1.0, ..Default::default() },
                db::RunShift { top: 2010.0, base: 2019.0, delta: 3.0, ..Default::default() },
            ],
            &db::ShiftTargets::default(),
            &Default::default(),
        )
        .unwrap();

        let dir = std::env::temp_dir();
        let xrd = dir.join("sandi_late_xrd.csv");
        // Written at the ORIGINAL depths, as the lab would have them.
        std::fs::write(&xrd, "DEPTH,KAOLINITE,ILLITE\n2005,0.10,0.20\n2015,0.30,0.40\n1990,0.05,0.06\n").unwrap();
        let path = xrd.to_str().unwrap();

        // Off: the samples land where the file says, which is now the wrong rock.
        let plain = import_aux_file(&conn, &w, "XRD", path, Some("ASWRITTEN"), false);
        assert!(plain.error.is_none(), "{:?}", plain.error);
        let rows = db::list_aux_data(&conn, &w, Some("XRD")).unwrap();
        assert!(
            rows.iter().any(|r| (r.depth_top - 2005.0).abs() < 1e-3),
            "unmapped import keeps the delivered depth"
        );

        // On: each sample follows the barrel it was cut from.
        let followed = import_aux_file(&conn, &w, "XRD", path, Some("FOLLOWED"), true);
        assert!(followed.error.is_none(), "{:?}", followed.error);
        let rows = db::list_aux_data(&conn, &w, Some("XRD")).unwrap();
        let depths: Vec<f32> = {
            let mut d: Vec<f32> = rows.iter().map(|r| r.depth_top).collect();
            d.sort_by(f32::total_cmp);
            d.dedup_by(|a, b| (*a - *b).abs() < 1e-3);
            d
        };
        assert!(
            depths.iter().any(|&x| (x - 2006.0).abs() < 1e-3),
            "2005 was in the barrel that moved 1 m: {depths:?}"
        );
        assert!(
            depths.iter().any(|&x| (x - 2018.0).abs() < 1e-3),
            "2015 was in the barrel that moved 3 m: {depths:?}"
        );
        assert!(
            depths.iter().any(|&x| (x - 1991.0).abs() < 1e-3),
            "1990 is above the core, so it holds the nearest correction: {depths:?}"
        );
        let notes = followed.notes.unwrap_or_default();
        assert!(
            notes.contains("outside the cored interval"),
            "the sample above the core must be reported as a guess, not placed silently: {notes}"
        );

        // A well with no core says so rather than pretending it mapped anything.
        let w2 = uuid::Uuid::new_v4();
        db::insert_well(&conn, w2, "SANDI-NOCORE", None, None, None).unwrap();
        let none = import_aux_file(&conn, &w2.to_string(), "XRD", path, None, true);
        assert!(none.error.is_none(), "{:?}", none.error);
        assert!(
            none.notes.unwrap_or_default().contains("no core to follow"),
            "asking to follow a core that is not there must be said out loud"
        );
        std::fs::remove_file(&xrd).ok();
    }

    /// SB-DIO-062 / SB-DIO-T95. The required encodings and reported choice are specified
    /// in `docs/PRD_v2/21_data-io.md` §§4 and 6.
    #[test]
    fn utf8_utf16_in_both_byte_orders_with_and_without_boms_and_windows_1252_are_imported_and_reported() {
        // SB-DIO-062 and SB-DIO-T95, 21_data-io.md §4 and §6: the UTF-16LE BOM
        // pair is the named acceptance fixture; the other required decoder branches are
        // pinned in the same import-level contract so a byte-order branch cannot regress
        // while the mandatory reader still compiles.
        let body = "~VERSION\nVERS. 2.0\nWRAP. NO\n~WELL\nNULL. -999.25\nWELL. ENCODING-ρ\n~CURVE\nDEPT.M\nGR.GAPI\n~ASCII\n1000 50\n1001 51\n";
        let utf16 = |big_endian: bool, bom: bool| {
            let mut bytes = if bom {
                if big_endian { vec![0xFE, 0xFF] } else { vec![0xFF, 0xFE] }
            } else {
                Vec::new()
            };
            for unit in body.encode_utf16() {
                let encoded = if big_endian { unit.to_be_bytes() } else { unit.to_le_bytes() };
                bytes.extend_from_slice(&encoded);
            }
            bytes
        };
        let mut utf8_bom = vec![0xEF, 0xBB, 0xBF];
        utf8_bom.extend_from_slice(body.as_bytes());
        let windows_body = body.replace("ENCODING-ρ", "ENCODING");
        let mut windows_1252 = windows_body.as_bytes().to_vec();
        let name_start = windows_body.find("ENCODING").unwrap();
        windows_1252.splice(name_start..name_start + "ENCODING".len(), [b'E', b'N', b'C', b'-', 0x95]);

        let cases = [
            ("utf8", body.as_bytes().to_vec(), "UTF-8"),
            ("utf8_bom", utf8_bom, "UTF-8 with BOM"),
            ("utf16le_bom", utf16(false, true), "UTF-16LE with BOM"),
            ("utf16le_no_bom", utf16(false, false), "UTF-16LE without BOM"),
            ("utf16be_bom", utf16(true, true), "UTF-16BE with BOM"),
            ("utf16be_no_bom", utf16(true, false), "UTF-16BE without BOM"),
            ("windows_1252", windows_1252, "Windows-1252"),
        ];

        for (label, bytes, expected) in cases {
            let conn = Connection::open_in_memory().unwrap();
            db::create_schema(&conn).unwrap();
            let path = std::env::temp_dir().join(format!(
                "sandibumi_dio_062_{label}_{}.las",
                Uuid::new_v4()
            ));
            std::fs::write(&path, bytes).unwrap();
            let result = import_las_files(
                &conn,
                &[path.to_string_lossy().into_owned()],
                None,
            )
            .remove(0);
            assert!(result.error.is_none(), "{label} import failed: {:?}", result.error);
            assert_eq!(result.rows, 2, "{label} must import both rows");
            assert_eq!(result.text_encoding.as_deref(), Some(expected), "{label}");
            assert!(
                !result.warning.as_deref().unwrap_or("").contains("encoding"),
                "the encoding report is structured data, not a warning: {:?}",
                result.warning
            );
            std::fs::remove_file(path).ok();
        }
    }

    /// SB-DIO-041 / SB-DIO-T59. LAS 3.0 recognition and named unread sections are specified
    /// in `docs/PRD_v2/21_data-io.md` §§4.8 and 6.8 (D-25).
    #[test]
    fn a_las_3_file_is_recognised_as_3_0_and_every_unread_section_is_named_in_the_result() {
        let path = std::env::temp_dir().join(format!(
            "sandibumi-dio-041-las3-{}.las",
            std::process::id()
        ));
        let body = "~Version\nVERS. 3.0 : CWLS LAS 3\nWRAP. NO :\n\
                    ~Well\nWELL. LAS-THREE :\nNULL. -999.25 :\n\
                    ~Curve\nDEPT.M : depth\nGR.GAPI : gamma ray\n\
                    ~Core_Data\nPLUG_A | 1000.0 | 0.18\n\
                    ~Tops\nSAND_A | 1000.5\n\
                    ~ASCII\n1000.0 50\n1000.5 55\n";
        std::fs::write(&path, body).unwrap();

        let frame = parsers::parse_las_2_all(&path).unwrap();
        assert_eq!(frame.las_version.as_deref(), Some("3.0"));
        assert_eq!(frame.unread_sections, vec!["~Core_Data", "~Tops"]);
        assert_eq!(frame.curves.len(), 1, "an associated Core_Data section is not a ~Curve block");

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let result = import_las_files_with(
            &conn,
            &[path.to_string_lossy().into_owned()],
            None,
            &LasImportOptions::default(),
        )
        .remove(0);
        assert!(result.error.is_none(), "LAS 3.0's ordinary log array still imports: {:?}", result.error);
        let warning = result.warning.unwrap_or_default();
        assert!(warning.contains("LAS 3.0 recognized"));
        assert!(warning.contains("~Core_Data") && warning.contains("~Tops"));

        // The opposite side: changing only the declaration to 2.0 must not label that file 3.0.
        std::fs::write(&path, body.replacen("VERS. 3.0", "VERS. 2.0", 1)).unwrap();
        let frame2 = parsers::parse_las_2_all(&path).unwrap();
        assert_eq!(frame2.las_version.as_deref(), Some("2.0"));
        assert!(frame2.unread_sections.is_empty());
        std::fs::remove_file(&path).ok();
    }

    /// Ad-hoc verification against a real field delivery — whatever LAS files sit in the
    /// configured fixture folder (`SANDIBUMI_FIELD_FIXTURES/las/`). Ignored by default and
    /// skipped with a printed reason when no folder is configured; run explicitly with
    /// `cargo test --release -- --ignored --nocapture test_import_real_field_files`.
    #[test]
    #[ignore]
    fn test_import_real_field_files() {
        let paths = crate::field_fixtures::las_files(12);
        if crate::field_fixtures::skip("test_import_real_field_files", paths.len(), 1) {
            return;
        }

        let db_path = crate::field_fixtures::temp_db("import_test");
        let conn = db::init_db(db_path.to_str().unwrap()).expect("init_db failed");

        let results = import_las_files(&conn, &paths, None);
        for r in &results {
            println!(
                "{} -> well_name={:?} rows={} error={:?}",
                r.path, r.well_name, r.rows, r.error
            );
        }

        let failures: Vec<_> = results.iter().filter(|r| r.error.is_some()).collect();
        assert!(failures.is_empty(), "{failures:?}");

        let well_count: i64 = conn
            .query_row("SELECT count(*) FROM wells", [], |row| row.get(0))
            .unwrap();
        assert_eq!(well_count, paths.len() as i64);
    }
}
