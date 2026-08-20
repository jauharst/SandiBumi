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
    /// Every non-comment LAS `~W` record in source order, with only cited mappings labelled.
    pub well_headers: Vec<parsers::LasWellHeader>,
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
    /// Effective per-source-channel null handling, including explicit `NoNull` versus `Unset`.
    pub null_resolutions: Vec<parsers::ChannelNullResolution>,
    pub index_resolution: Option<parsers::IndexResolution>,
    /// Versioned LAS section policy plus every non-fatal tolerance that fired.
    pub section_policy: String,
    pub section_handling: Vec<parsers::LasSectionHandling>,
    /// Every automatic value conversion, including the source unit and applied factor.
    pub unit_conversions: Vec<crate::curves::UnitConversion>,
    /// Declared units that were preserved because no reviewed conversion applied.
    pub unconverted_units: Vec<crate::curves::UnconvertedUnit>,
    /// Per-file answers to genuinely ambiguous unit symbols.
    pub unit_designations: Vec<crate::curves::UnitDesignation>,
    /// Every non-empty source spelling paired with only its registry-declared interpretation.
    pub unit_tokens: Vec<crate::curves::UnitTokenObservation>,
    /// Look-alike spellings that remain distinct because no explicit alias joins them.
    pub unit_token_warnings: Vec<String>,
    /// SB-CLY-034 (DEC-037): present when the import BLOCKED on undeclared vendor-sentinel
    /// values - the structured question (value, every affected curve, sample count) the
    /// dialog must put to the user. Nothing from this file was written.
    pub sentinel_question: Option<UndeclaredSentinelQuestion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NonMonotonicIndexDecision {
    AcceptAsDelivered,
}

/// SB-CLY-034 (DEC-037): the user's answer to the undeclared-sentinel question. There is
/// deliberately no default - conversion on magnitude alone is the forbidden path, so an
/// absent decision while candidates exist BLOCKS the import with the question.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SentinelDecision {
    Convert,
    Keep,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SentinelCurveCount {
    pub mnemonic: String,
    pub samples: usize,
}

/// SB-CLY-034 (DEC-037): the blocking question - the value, every affected curve and the
/// sample count, exactly what the ruling says the question must name.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UndeclaredSentinelQuestion {
    pub value: f32,
    pub curves: Vec<SentinelCurveCount>,
}

/// Options for a LAS import batch (the Import LAS dialog's choices).
#[derive(Debug, Clone, serde::Deserialize)]
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
    /// Explicit density-correction unit used only when a DRHO-family source channel declares
    /// none. Absence remains a refusal; this is a user statement, never a mnemonic-based default.
    #[serde(default, alias = "undeclaredDrhoUnit")]
    pub undeclared_drho_unit: Option<String>,
    /// Explicit well-name confirmations keyed by exact source path. The map is consulted only
    /// when the LAS container has no `WELL` identity; a container value always wins.
    #[serde(default, alias = "confirmedWellNames")]
    pub confirmed_well_names: std::collections::HashMap<String, String>,
    /// Required declaration for this imported curve set. It is never inferred from the observed
    /// depth sequence; POINT belongs in the point-delivery store and is refused here.
    #[serde(default, alias = "samplingStyle")]
    pub sampling_style: Option<crate::schema_vocab::SamplingStyle>,
    /// Required only for CONTINUOUS_REGULAR. It has no default and carries its own unit so the
    /// verification cannot silently borrow the project's unit or an unrelated snap tolerance.
    #[serde(default, alias = "samplingStyleVerifyTolerance")]
    pub sampling_style_verify_tolerance: Option<crate::units::DepthTolerance>,
    /// SB-CLY-034 (DEC-037): the user's decision on undeclared vendor-sentinel values.
    /// Absent while candidates exist means the import BLOCKS with the question.
    #[serde(default, alias = "undeclaredSentinelDecision")]
    pub undeclared_sentinel_decision: Option<SentinelDecision>,
}

#[cfg(test)]
impl Default for LasImportOptions {
    fn default() -> Self {
        // Existing tests that are not about SB-DBM-028 declare IRREGULAR explicitly through this
        // fixture constructor. Production has no Default implementation and must supply a choice.
        Self {
            set_name: None,
            attach: false,
            file_depth_unit: None,
            channel_nulls: Default::default(),
            null_rules: Vec::new(),
            non_monotonic_index: None,
            duplicate_depth_policy: None,
            ms_per_ft_meanings: Default::default(),
            undeclared_drho_unit: None,
            confirmed_well_names: Default::default(),
            sampling_style: Some(crate::schema_vocab::SamplingStyle::ContinuousIrregular),
            sampling_style_verify_tolerance: None,
            undeclared_sentinel_decision: None,
        }
    }
}

/// Normalizes a user/derived set name to the store's convention: trimmed, upper-cased,
/// spaces collapsed to `_`; empty → RAW.
pub fn canonical_set_name(raw: Option<&str>) -> String {
    let s = raw.unwrap_or("").trim().to_uppercase().replace(' ', "_");
    if s.is_empty() { "RAW".to_string() } else { s }
}

#[derive(Debug, Clone)]
struct ImportSetSamplingVerdict {
    declared: crate::schema_vocab::SamplingStyle,
    effective: crate::schema_vocab::SamplingStyle,
    tolerance: Option<crate::units::DepthTolerance>,
    warning: Option<String>,
    gap_depth: Option<f32>,
    gap_row_count: Option<i64>,
}

fn verify_import_set_sampling(
    depth: &[f32],
    declared_step: Option<&str>,
    file_unit: crate::units::DepthUnit,
    stored_unit: crate::units::DepthUnit,
    options: &LasImportOptions,
) -> Result<ImportSetSamplingVerdict, String> {
    let declared = options.sampling_style.ok_or_else(|| {
        "sampling style declaration is required before import; it is never inferred from depths"
            .to_string()
    })?;
    match declared {
        crate::schema_vocab::SamplingStyle::Point => Err(
            "POINT sampling must use the point-delivery store, not continuous LAS ingest".into(),
        ),
        crate::schema_vocab::SamplingStyle::ContinuousIrregular => Ok(ImportSetSamplingVerdict {
            declared,
            effective: declared,
            tolerance: None,
            warning: None,
            gap_depth: None,
            gap_row_count: None,
        }),
        crate::schema_vocab::SamplingStyle::ContinuousRegular => {
            let tolerance = options.sampling_style_verify_tolerance.ok_or_else(|| {
                "CONTINUOUS_REGULAR requires an explicit unit-typed sampling verification tolerance; no default ships"
                    .to_string()
            })?;
            if !tolerance.value.is_finite() || tolerance.value < 0.0 {
                return Err("sampling verification tolerance must be finite and non-negative".into());
            }
            let raw_step = declared_step
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    "CONTINUOUS_REGULAR requires a declared STEP in the delivery".to_string()
                })?
                .parse::<f64>()
                .map_err(|_| "CONTINUOUS_REGULAR has an unreadable declared STEP".to_string())?;
            if !raw_step.is_finite() || raw_step == 0.0 {
                return Err("CONTINUOUS_REGULAR requires a finite non-zero declared STEP".into());
            }
            let expected_step = crate::units::convert_depth(raw_step, file_unit, stored_unit);
            let stored_tolerance = crate::units::convert_depth(
                tolerance.value,
                tolerance.unit,
                stored_unit,
            )
            .abs();
            let contradiction = depth.windows(2).enumerate().find_map(|(previous, pair)| {
                let actual_step = pair[1] as f64 - pair[0] as f64;
                ((actual_step - expected_step).abs() > stored_tolerance).then(|| {
                    let ratio = (actual_step / expected_step).abs();
                    let missing_rows = if ratio.is_finite() {
                        (ratio.round() as i64 - 1).max(0)
                    } else {
                        0
                    };
                    (previous + 2, pair[1], missing_rows, actual_step)
                })
            });
            if let Some((data_row, gap_depth, gap_row_count, actual_step)) = contradiction {
                let warning = format!(
                    "sampling declaration contradicted at depth {gap_depth:.4} {} (data row {data_row}): {gap_row_count} missing row(s); declared STEP {expected_step:.6} {}, observed increment {actual_step:.6} {}, explicit tolerance {:.6} {}",
                    stored_unit.code(),
                    stored_unit.code(),
                    stored_unit.code(),
                    tolerance.value,
                    tolerance.unit.code()
                );
                Ok(ImportSetSamplingVerdict {
                    declared,
                    effective: crate::schema_vocab::SamplingStyle::ContinuousIrregular,
                    tolerance: Some(tolerance),
                    warning: Some(warning),
                    gap_depth: Some(gap_depth),
                    gap_row_count: Some(gap_row_count),
                })
            } else {
                Ok(ImportSetSamplingVerdict {
                    declared,
                    effective: declared,
                    tolerance: Some(tolerance),
                    warning: None,
                    gap_depth: None,
                    gap_row_count: None,
                })
            }
        }
    }
}

fn record_import_set_sampling(
    conn: &Connection,
    well_id: &str,
    set_name: &str,
    verdict: &ImportSetSamplingVerdict,
) -> db::DbResult<()> {
    conn.execute(
        "INSERT INTO import_sets
            (well_id, set_name, declared_sampling_style, effective_sampling_style,
             sampling_verified, verification_tolerance, verification_tolerance_unit,
             verification_warning, gap_depth, gap_row_count)
         VALUES (?1, ?2, ?3, ?4, true, ?5, ?6, ?7, ?8, ?9)",
        params![
            well_id,
            set_name,
            verdict.declared.as_str(),
            verdict.effective.as_str(),
            verdict.tolerance.map(|value| value.value),
            verdict.tolerance.map(|value| value.unit.code()),
            verdict.warning,
            verdict.gap_depth,
            verdict.gap_row_count,
        ],
    )?;
    Ok(())
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

fn cancelled_las_import(path: &str) -> ImportResult {
    ImportResult {
        path: path.to_string(),
        well_id: None,
        well_name: None,
        well_headers: Vec::new(),
        rows: 0,
        text_encoding: None,
        warning: Some("cancelled before import".into()),
        error: None,
        attached_set: None,
        alias_decisions: Vec::new(),
        null_resolutions: Vec::new(),
        index_resolution: None,
        section_policy: parsers::LAS_SECTION_POLICY_ID.to_string(),
        section_handling: Vec::new(),
        unit_conversions: Vec::new(),
        unconverted_units: Vec::new(),
        unit_designations: Vec::new(),
        unit_tokens: Vec::new(),
        unit_token_warnings: Vec::new(), sentinel_question: None
    }
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
    // The primary parse now retains every channel. Bound concurrently-live parsed files to
    // the Rayon worker count instead of retaining the whole batch before the first write.
    let batch_size = rayon::current_num_threads().max(1);
    let mut imported = Vec::with_capacity(paths.len());
    for (chunk_index, chunk) in paths.chunks(batch_size).enumerate() {
        if progress.map_or(false, |p| p.is_cancelled()) {
            for path in &paths[chunk_index * batch_size..] {
                if let Some(p) = progress {
                    p.finish_item(path, crate::jobs::ItemState::Warned, Some("cancelled".into()));
                }
                imported.push(cancelled_las_import(path));
            }
            break;
        }
        let parsed: Vec<(String, Result<(String, CurveColumns), ParseError>)> = chunk
            .par_iter()
            .map(|path| {
                let result = (|| {
                    let columns = parsers::parse_las_2_import(
                        path,
                        &opts.channel_nulls,
                        &opts.null_rules,
                        opts.ms_per_ft_meanings.get(path).copied(),
                        // SB-CLY-034 (DEC-037): conversion happens only on the user's
                        // explicit confirmation - never on magnitude alone.
                        matches!(
                            opts.undeclared_sentinel_decision,
                            Some(SentinelDecision::Convert)
                        ),
                    )?;
                    let identity = parsers::las_well_identity_from_container(
                        std::path::Path::new(path),
                        columns.well_name.clone(),
                    );
                    let well_name = if let Some(container_well_name) = identity.container_well_name {
                        container_well_name
                    } else if let Some(confirmed) = opts
                        .confirmed_well_names
                        .get(path)
                        .map(|value| value.trim())
                        .filter(|value| !value.is_empty())
                    {
                        confirmed.to_string()
                    } else {
                        let proposal = identity
                            .filename_proposal
                            .as_deref()
                            .map(|value| format!("; filename proposal '{value}'"))
                            .unwrap_or_else(|| "; no filename proposal is available".to_string());
                        return Err(ParseError::Las(format!(
                            "source well identity is absent in {path}{proposal}; explicit confirmation is required before import"
                        )));
                    };
                    Ok::<_, ParseError>((well_name, columns))
                })();
                (path.clone(), result)
            })
            .collect();

        for (path, result) in parsed {
            // Cancel before the DB write, so clicking Cancel actually stops wells being created.
            // Without this the flag was flipped, every remaining file was still inserted, and the
            // job was then labelled "Cancelled" — the user was told the import stopped while the
            // project filled up with unwanted wells. The parse pass above has already run by this
            // point (for this bounded chunk), so cancel stops its writes and prevents later
            // chunks from being parsed.
            if progress.map_or(false, |p| p.is_cancelled()) {
                if let Some(p) = progress {
                    p.finish_item(&path, crate::jobs::ItemState::Warned, Some("cancelled".into()));
                }
                imported.push(cancelled_las_import(&path));
                continue;
            }
            if let Some(p) = progress {
                let base = path.rsplit(['/', '\\']).next().unwrap_or(&path);
                p.set_current(Some(format!("Importing {base}")));
                p.start_item(&path);
            }
            let out = match result {
                Ok((well_name, columns)) => insert_parsed_well(conn, path.clone(), well_name, columns, opts),
                Err(e) => ImportResult { path: path.clone(), well_id: None, well_name: None, well_headers: Vec::new(), rows: 0, text_encoding: None, warning: None, error: Some(e.to_string()), attached_set: None, alias_decisions: Vec::new(), null_resolutions: Vec::new(), index_resolution: None, section_policy: parsers::LAS_SECTION_POLICY_ID.to_string(), section_handling: Vec::new(), unit_conversions: Vec::new(), unconverted_units: Vec::new(), unit_designations: Vec::new(), unit_tokens: Vec::new(), unit_token_warnings: Vec::new() , sentinel_question: None},
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
            imported.push(out);
        }
        if progress.map_or(false, |p| p.is_cancelled()) {
            let next_path = ((chunk_index + 1) * batch_size).min(paths.len());
            for path in &paths[next_path..] {
                if let Some(p) = progress {
                    p.finish_item(
                        path,
                        crate::jobs::ItemState::Warned,
                        Some("cancelled".into()),
                    );
                }
                imported.push(cancelled_las_import(path));
            }
            break;
        }
    }
    imported
}

fn insert_parsed_well(
    conn: &Connection,
    path: String,
    well_name: String,
    mut columns: CurveColumns,
    opts: &LasImportOptions,
) -> ImportResult {
    let well_id = Uuid::new_v4();
    let well_headers = columns.well_headers.clone();
    let alias_decisions = columns.alias_decisions.clone();
    let null_resolutions = columns.null_resolutions.clone();
    let index_resolution = columns.index_resolution.clone();
    let section_policy = columns.section_policy.clone();
    let section_handling = columns.section_handling.clone();
    let mut unit_designations = columns.unit_designations.clone();
    let las_version = columns.las_version.clone();
    let unread_sections = columns.unread_sections.clone();
    let text_encoding = columns.text_encoding.clone();
    let declared_step_note = columns.declared_step_mismatch_note.clone();
    let observed_units = columns
        .raw_curves
        .iter()
        .map(|curve| (curve.mnemonic.clone(), curve.unit.clone()))
        .collect::<Vec<_>>();
    let (unit_tokens, unit_token_warnings) = crate::curves::observe_unit_tokens(&observed_units);

    // SB-CLY-034 (DEC-037): quarantine and ASK. The parser detected undeclared values equal
    // to the known vendor bad-hole sentinel; without the user's decision the import BLOCKS
    // here - before any write - with a question naming the value, every affected curve and
    // the sample count. BOTH answers are recorded (a kept sentinel is itself a finding about
    // the delivery), and an explicit NoNull channel never reaches this point at all.
    let sentinel_candidates = columns.undeclared_sentinel_candidates.clone();
    let mut sentinel_note: Option<String> = None;
    if !sentinel_candidates.is_empty() {
        let listing = sentinel_candidates
            .iter()
            .map(|(mnemonic, samples)| format!("{mnemonic} ({samples} sample(s))"))
            .collect::<Vec<_>>()
            .join(", ");
        match opts.undeclared_sentinel_decision {
            None => {
                return ImportResult {
                    path,
                    well_id: None,
                    well_name: Some(well_name),
                    well_headers,
                    rows: 0,
                    text_encoding: Some(columns.text_encoding),
                    warning: None,
                    error: Some(format!(
                        "import blocked: value {} matches the known vendor bad-hole sentinel \
                         but this file does not declare it as null - affected: {listing}. \
                         Decide whether these cells are absent values or measurements; \
                         nothing was imported.",
                        parsers::VENDOR_BADHOLE_SENTINEL
                    )),
                    attached_set: None,
                    alias_decisions,
                    null_resolutions,
                    index_resolution,
                    section_policy,
                    section_handling,
                    unit_conversions: Vec::new(),
                    unconverted_units: Vec::new(),
                    unit_designations,
                    unit_tokens,
                    unit_token_warnings,
                    sentinel_question: Some(UndeclaredSentinelQuestion {
                        value: parsers::VENDOR_BADHOLE_SENTINEL,
                        curves: sentinel_candidates
                            .iter()
                            .map(|(mnemonic, samples)| SentinelCurveCount {
                                mnemonic: mnemonic.clone(),
                                samples: *samples,
                            })
                            .collect(),
                    }),
                };
            }
            Some(SentinelDecision::Convert) => {
                sentinel_note = Some(format!(
                    "undeclared {} vendor bad-hole sentinel converted to absent on user \
                     confirmation (DEC-037): {listing}",
                    parsers::VENDOR_BADHOLE_SENTINEL
                ));
            }
            Some(SentinelDecision::Keep) => {
                sentinel_note = Some(format!(
                    "undeclared {} sentinel-shaped values KEPT as measurements on user \
                     decision (DEC-037): {listing}",
                    parsers::VENDOR_BADHOLE_SENTINEL
                ));
            }
        }
    }

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
                    well_headers: well_headers.clone(),
                    rows: 0,
                    text_encoding: Some(text_encoding.clone()),
                    warning: None,
                    error: Some(format!("unrecognized confirmed file depth unit '{raw}'")),
                    attached_set: None,
                    alias_decisions: alias_decisions.clone(),
                    null_resolutions: null_resolutions.clone(),
                    index_resolution: index_resolution.clone(),
                    section_policy: section_policy.clone(),
                    section_handling: section_handling.clone(),
                    unit_conversions: Vec::new(),
                    unconverted_units: Vec::new(),
                    unit_designations: unit_designations.clone(),
                    unit_tokens: unit_tokens.clone(),
                    unit_token_warnings: unit_token_warnings.clone(), sentinel_question: None
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
                well_headers: well_headers.clone(),
                rows: 0,
                text_encoding: Some(text_encoding.clone()),
                warning: None,
                error: Some(error),
                attached_set: None,
                alias_decisions: alias_decisions.clone(),
                null_resolutions: null_resolutions.clone(),
                index_resolution: index_resolution.clone(),
                section_policy: section_policy.clone(),
                section_handling: section_handling.clone(),
                unit_conversions: Vec::new(),
                unconverted_units: Vec::new(),
                unit_designations: unit_designations.clone(),
                unit_tokens: unit_tokens.clone(),
                unit_token_warnings: unit_token_warnings.clone(), sentinel_question: None
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
                    well_headers: well_headers.clone(),
                    rows: 0,
                    text_encoding: Some(text_encoding.clone()),
                    warning: None,
                    error: Some(format!(
                        "non-increasing index at data row {row}; a user decision is required before commit"
                    )),
                    attached_set: None,
                    alias_decisions,
                    null_resolutions: null_resolutions.clone(),
                    index_resolution,
                    section_policy,
                    section_handling,
                    unit_conversions: Vec::new(),
                    unconverted_units: Vec::new(),
                    unit_designations: unit_designations.clone(),
                    unit_tokens: unit_tokens.clone(),
                    unit_token_warnings: unit_token_warnings.clone(), sentinel_question: None
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
                    well_headers: well_headers.clone(),
                    rows: 0,
                    text_encoding: Some(text_encoding.clone()),
                    warning: None,
                    error: Some(format!(
                        "{duplicate_count} repeated depth row(s) require a declared duplicate policy before commit"
                    )),
                    attached_set: None,
                    alias_decisions,
                    null_resolutions: null_resolutions.clone(),
                    index_resolution,
                    section_policy,
                    section_handling,
                    unit_conversions: Vec::new(),
                    unconverted_units: Vec::new(),
                    unit_designations: unit_designations.clone(),
                    unit_tokens: unit_tokens.clone(),
                    unit_token_warnings: unit_token_warnings.clone(), sentinel_question: None
                }
            }
            Some(parsers::DuplicateDepthPolicy::Refuse) => {
                return ImportResult {
                    path,
                    well_id: None,
                    well_name: None,
                    well_headers: well_headers.clone(),
                    rows: 0,
                    text_encoding: Some(text_encoding.clone()),
                    warning: None,
                    error: Some(format!(
                        "duplicate-depth policy refuse blocked {duplicate_count} repeated row(s)"
                    )),
                    attached_set: None,
                    alias_decisions,
                    null_resolutions: null_resolutions.clone(),
                    index_resolution,
                    section_policy,
                    section_handling,
                    unit_conversions: Vec::new(),
                    unconverted_units: Vec::new(),
                    unit_designations: unit_designations.clone(),
                    unit_tokens: unit_tokens.clone(),
                    unit_token_warnings: unit_token_warnings.clone(), sentinel_question: None
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
            well_headers: well_headers.clone(),
            rows: 0,
            text_encoding: Some(text_encoding.clone()),
            warning: None,
            error: Some(format!(
                "no importable rows: {} had missing depth, {} duplicated an earlier depth",
                report.nonfinite, report.duplicate
            )),
            attached_set: None,
            alias_decisions: alias_decisions.clone(),
            null_resolutions: null_resolutions.clone(),
            index_resolution: index_resolution.clone(),
            section_policy: section_policy.clone(),
            section_handling: section_handling.clone(),
            unit_conversions: Vec::new(),
            unconverted_units: Vec::new(),
            unit_designations: unit_designations.clone(),
            unit_tokens: unit_tokens.clone(),
            unit_token_warnings: unit_token_warnings.clone(), sentinel_question: None
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
    if !section_handling.is_empty() {
        notes.push(format!(
            "{}: {}",
            section_policy,
            section_handling
                .iter()
                .map(parsers::LasSectionHandling::note)
                .collect::<Vec<_>>()
                .join("; ")
        ));
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
    // SB-CLY-034 (DEC-037) constraint 2: the answer is recorded either way - in the import
    // warning (which the shell writes into the durable process history) and per curve in
    // `curve_meta.source` below.
    notes.extend(sentinel_note.iter().cloned());
    notes.extend(unit_token_warnings.iter().cloned());
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
    let sampling_verdict = match verify_import_set_sampling(
        &columns.depth,
        columns.declared_step.as_deref(),
        file_unit.expect("resolve_index_unit accepts an import only with a declared file unit"),
        stored_unit,
        opts,
    ) {
        Ok(verdict) => verdict,
        Err(error) => {
            return ImportResult {
                path,
                well_id: None,
                well_name: None,
                well_headers: well_headers.clone(),
                rows: 0,
                text_encoding: Some(text_encoding),
                warning: None,
                error: Some(error),
                attached_set: None,
                alias_decisions,
                null_resolutions,
                index_resolution,
                section_policy,
                section_handling,
                unit_conversions: Vec::new(),
                unconverted_units: Vec::new(),
                unit_designations,
                unit_tokens,
                unit_token_warnings, sentinel_question: None
            };
        }
    };
    if let Some(warning) = sampling_verdict.warning.as_ref() {
        notes.push(warning.clone());
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
                return ImportResult { path, well_id: None, well_name: None, well_headers: well_headers.clone(), rows: 0, text_encoding: Some(text_encoding.clone()), warning: None, error: Some(e.to_string()), attached_set: None, alias_decisions: alias_decisions.clone(), null_resolutions: null_resolutions.clone(), index_resolution: index_resolution.clone(), section_policy: section_policy.clone(), section_handling: section_handling.clone(), unit_conversions: Vec::new(), unconverted_units: Vec::new(), unit_designations: unit_designations.clone(), unit_tokens: unit_tokens.clone(), unit_token_warnings: unit_token_warnings.clone() , sentinel_question: None}
            }
        };
        match stmt
            .query_map(params![name_norm], |r| r.get::<_, String>(0))
            .and_then(|rows| rows.collect::<Result<Vec<_>, _>>())
        {
            Ok(v) => v,
            Err(e) => {
                return ImportResult { path, well_id: None, well_name: None, well_headers: well_headers.clone(), rows: 0, text_encoding: Some(text_encoding.clone()), warning: None, error: Some(e.to_string()), attached_set: None, alias_decisions: alias_decisions.clone(), null_resolutions: null_resolutions.clone(), index_resolution: index_resolution.clone(), section_policy: section_policy.clone(), section_handling: section_handling.clone(), unit_conversions: Vec::new(), unconverted_units: Vec::new(), unit_designations: unit_designations.clone(), unit_tokens: unit_tokens.clone(), unit_token_warnings: unit_token_warnings.clone() , sentinel_question: None}
            }
        }
    };
    if opts.attach && matches.len() == 1 {
        let ms_per_ft_meaning = opts.ms_per_ft_meanings.get(&path).copied();
        let out = attach_curves_to_existing_well(
            conn,
            path,
            well_name,
            &matches[0],
            opts,
            notes,
            alias_decisions.clone(),
            null_resolutions.clone(),
            index_resolution.clone(),
            section_policy.clone(),
            section_handling.clone(),
            well_headers.clone(),
            unit_designations.clone(),
            unit_tokens.clone(),
            unit_token_warnings.clone(),
            text_encoding.clone(),
            &columns.depth,
            &columns.raw_curves,
            ms_per_ft_meaning,
            &sampling_verdict,
            &sentinel_candidates,
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
    // Prepare every native curve before the transaction. The well row, standard projection,
    // project-unit adoption, generic metadata, and every native sample then commit together.
    let generic_depth = columns.depth.clone();
    let generic_curves = std::mem::take(&mut columns.raw_curves);
    let prepared = match prepare_generic_curves(
        &generic_depth,
        &generic_curves,
        opts.ms_per_ft_meanings.get(&path).copied(),
        opts.undeclared_drho_unit.as_deref(),
    ) {
        Ok(mut prepared) => {
            // SB-CLY-034 (DEC-037) constraint 2: the sentinel answer travels on the
            // affected curves' own provenance in the create path too.
            apply_sentinel_source_notes(
                &mut prepared.curves,
                &sentinel_candidates,
                opts.undeclared_sentinel_decision,
            );
            prepared
        }
        Err(error) => {
            return ImportResult {
                path,
                well_id: None,
                well_name: None,
                well_headers: well_headers.clone(),
                rows: 0,
                text_encoding: Some(text_encoding),
                warning: None,
                error: Some(error.to_string()),
                attached_set: None,
                alias_decisions,
                null_resolutions,
                index_resolution,
                section_policy,
                section_handling,
                unit_conversions: Vec::new(),
                unconverted_units: Vec::new(),
                unit_designations,
                unit_tokens,
                unit_token_warnings, sentinel_question: None
            };
        }
    };
    unit_designations.extend(prepared.unit_designations.iter().cloned());
    let mut null_screened: Vec<(String, usize)> = Vec::new();
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
        let set = resolve_set_name(
            conn,
            &well_id.to_string(),
            &canonical_set_name(opts.set_name.as_deref()),
        );
        record_import_set_sampling(conn, &well_id.to_string(), &set, &sampling_verdict)?;
        // SB-DBM-030: the generic store carries every curve of this delivery (the standard
        // projection holds the same columns), so its flag channel alone names every screened
        // mnemonic exactly once.
        null_screened = write_prepared_generic_curves_in_transaction(
            conn,
            &well_id.to_string(),
            &generic_depth,
            &set,
            &prepared.curves,
        )?;
        // This delivery populated both the standard projection and the native generic store from
        // the same decoded columns in this transaction. Mark the legacy backfill complete now;
        // otherwise the next open adds random duplicate RAW identities and breaks reproducible
        // ancestry across copied projects.
        db::mark_standard_curve_migration_done(conn, &well_id.to_string())?;
        if let crate::units::IndexUnitAction::Adopted(unit) = unit_action {
            crate::units::set_project_depth_unit(conn, unit)?;
        }
        Ok(())
    });

    match result {
        Ok(()) => {
            let unit_conversions = prepared.unit_conversions;
            let unconverted_units = prepared.unconverted_units;
            // Conversion and unresolved-unit notes describe only values that committed with
            // this complete delivery.
            notes.extend(unit_conversions.iter().map(crate::curves::UnitConversion::note));
            notes.extend(unconverted_units.iter().map(crate::curves::UnconvertedUnit::note));
            for (mnemonic, count) in &null_screened {
                notes.push(format!(
                    "null screen: {count} large-negative sample(s) on {mnemonic} stored as missing (undeclared Geolog-family null sentinel)"
                ));
            }
            let warning = (!notes.is_empty()).then(|| notes.join("; "));
            ImportResult { path, well_id: Some(well_id.to_string()), well_name: Some(well_name), well_headers, rows, text_encoding: Some(text_encoding), warning, error: None, attached_set: None, alias_decisions, null_resolutions, index_resolution, section_policy, section_handling, unit_conversions, unconverted_units, unit_designations, unit_tokens, unit_token_warnings, sentinel_question: None }
        }
        Err(e) => ImportResult { path, well_id: None, well_name: None, well_headers, rows: 0, text_encoding: Some(text_encoding), warning: None, error: Some(e.to_string()), attached_set: None, alias_decisions, null_resolutions, index_resolution, section_policy, section_handling, unit_conversions: Vec::new(), unconverted_units: Vec::new(), unit_designations, unit_tokens, unit_token_warnings, sentinel_question: None },
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
    null_resolutions: Vec<parsers::ChannelNullResolution>,
    index_resolution: Option<parsers::IndexResolution>,
    section_policy: String,
    section_handling: Vec<parsers::LasSectionHandling>,
    well_headers: Vec<parsers::LasWellHeader>,
    mut unit_designations: Vec<crate::curves::UnitDesignation>,
    unit_tokens: Vec<crate::curves::UnitTokenObservation>,
    unit_token_warnings: Vec<String>,
    text_encoding: String,
    depth: &[f32],
    curves: &[parsers::RawLasCurve],
    ms_per_ft_meaning: Option<crate::curves::MsPerFtMeaning>,
    sampling_verdict: &ImportSetSamplingVerdict,
    sentinel_candidates: &[(String, usize)],
) -> ImportResult {
    let set = resolve_set_name(conn, well_id, &canonical_set_name(opts.set_name.as_deref()));
    match import_parsed_curves_into_generic_store(
        conn,
        well_id,
        depth,
        curves,
        &set,
        ms_per_ft_meaning,
        opts.undeclared_drho_unit.as_deref(),
        Some(sampling_verdict),
        sentinel_candidates,
        opts.undeclared_sentinel_decision,
    ) {
        // A normal attach is a SUCCESS, not a warning — `attached_set` carries the story
        // and the frontend reports it separately. Only genuine notes (unit reconciliation,
        // dropped rows) reach `warning`.
        Ok(report) => {
            let mut notes = notes;
            notes.extend(report.unit_conversions.iter().map(crate::curves::UnitConversion::note));
            notes.extend(report.unconverted_units.iter().map(crate::curves::UnconvertedUnit::note));
            for (mnemonic, count) in &report.null_screened {
                notes.push(format!(
                    "null screen: {count} large-negative sample(s) on {mnemonic} stored as missing (undeclared Geolog-family null sentinel)"
                ));
            }
            unit_designations.extend(report.unit_designations.iter().cloned());
            ImportResult {
                path,
                well_id: Some(well_id.to_string()),
                well_name: Some(well_name),
                well_headers,
                rows: report.rows,
                text_encoding: Some(text_encoding),
                warning: (!notes.is_empty()).then(|| notes.join("; ")),
                error: None,
                attached_set: Some(set),
                alias_decisions,
                null_resolutions,
                index_resolution,
                section_policy,
                section_handling,
                unit_conversions: report.unit_conversions,
                unconverted_units: report.unconverted_units,
                unit_designations,
                unit_tokens,
                unit_token_warnings, sentinel_question: None
            }
        }
        // Attaching IS the import here (no well/standard-curve write happened), so a
        // loader failure is a real per-file error, not a note.
        Err(e) => ImportResult { path, well_id: None, well_name: None, well_headers, rows: 0, text_encoding: Some(text_encoding), warning: None, error: Some(e.to_string()), attached_set: None, alias_decisions, null_resolutions, index_resolution, section_policy, section_handling, unit_conversions: Vec::new(), unconverted_units: Vec::new(), unit_designations, unit_tokens, unit_token_warnings, sentinel_question: None },
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
    let mut frame = parsers::parse_las_2_all(path)
        .map_err(|error| db::DbError::LengthMismatch(format!("parse_las_2_all: {error}")))?;
    let declared = crate::units::project_depth_unit(conn)?;
    let file_unit = frame
        .depth_unit
        .as_deref()
        .and_then(crate::units::DepthUnit::parse)
        .or(confirmed_file_unit);
    let action = crate::units::resolve_index_unit(declared, file_unit)
        .map_err(db::DbError::LengthMismatch)?;
    if let crate::units::IndexUnitAction::Convert { from, to } = action {
        crate::units::convert_depths(&mut frame.depth, from, to);
    }
    let duplicates = parsers::duplicate_depth_count(&frame.depth);
    if duplicates > 0 {
        return Err(db::DbError::LengthMismatch(format!(
            "{duplicates} repeated depth row(s) have no resolving duplicate policy"
        )));
    }
    parsers::sanitize_las_frame(&mut frame);
    // Legacy generic-store backfill for wells imported before this feature existed - no
    // sentinel question was ever asked for them, so there is no answer to stamp.
    import_parsed_curves_into_generic_store(
        conn,
        well_id,
        &frame.depth,
        &frame.curves,
        set_name,
        None,
        None,
        None,
        &[],
        None,
    )
    .map(|report| (report.curves_written, report.rows))
}

struct GenericCurveImportReport {
    curves_written: usize,
    rows: usize,
    unit_conversions: Vec<crate::curves::UnitConversion>,
    unconverted_units: Vec<crate::curves::UnconvertedUnit>,
    unit_designations: Vec<crate::curves::UnitDesignation>,
    /// SB-DBM-030 flag channel: per delivered mnemonic, samples screened to SQL NULL.
    null_screened: Vec<(String, usize)>,
}

struct PreparedGenericCurve {
    mnemonic: String,
    unit: Option<String>,
    family: Option<&'static str>,
    values: Vec<f32>,
    /// SB-CLY-034 (DEC-037) constraint 2: the per-curve provenance record of the
    /// undeclared-sentinel answer. `None` writes the ordinary "LAS import" source.
    source_note: Option<String>,
}

/// SB-CLY-034 (DEC-037): stamp the user's sentinel answer onto the affected curves'
/// provenance, so a later reader of `curve_meta.source` can tell the question was asked
/// and what was decided - for a KEEP as much as for a conversion.
fn apply_sentinel_source_notes(
    prepared: &mut [PreparedGenericCurve],
    candidates: &[(String, usize)],
    decision: Option<SentinelDecision>,
) {
    let Some(decision) = decision else { return };
    for (mnemonic, samples) in candidates {
        if let Some(curve) = prepared
            .iter_mut()
            .find(|curve| curve.mnemonic.eq_ignore_ascii_case(mnemonic))
        {
            curve.source_note = Some(match decision {
                SentinelDecision::Convert => format!(
                    "LAS import; undeclared {} vendor sentinel ({samples} sample(s)) \
                     converted to absent on user confirmation (DEC-037)",
                    parsers::VENDOR_BADHOLE_SENTINEL
                ),
                SentinelDecision::Keep => format!(
                    "LAS import; undeclared {} sentinel-shaped values ({samples} sample(s)) \
                     kept as measurements on user decision (DEC-037)",
                    parsers::VENDOR_BADHOLE_SENTINEL
                ),
            });
        }
    }
}

struct PreparedGenericImport {
    curves: Vec<PreparedGenericCurve>,
    unit_conversions: Vec<crate::curves::UnitConversion>,
    unconverted_units: Vec<crate::curves::UnconvertedUnit>,
    unit_designations: Vec<crate::curves::UnitDesignation>,
}

fn import_parsed_curves_into_generic_store(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    curves: &[parsers::RawLasCurve],
    set_name: &str,
    ms_per_ft_meaning: Option<crate::curves::MsPerFtMeaning>,
    undeclared_drho_unit: Option<&str>,
    sampling_verdict: Option<&ImportSetSamplingVerdict>,
    sentinel_candidates: &[(String, usize)],
    sentinel_decision: Option<SentinelDecision>,
) -> db::DbResult<GenericCurveImportReport> {
    // `depth` and `curves` came from the same primary parse and passed one shared unit,
    // duplicate-depth, and sanitation decision. Keep that exact row alignment here: the
    // generic store must hold the same source depths as the standard projection, never a
    // separately parsed or independently cleaned approximation.
    if depth.is_empty() {
        return Ok(GenericCurveImportReport {
            curves_written: 0,
            rows: 0,
            unit_conversions: Vec::new(),
            unconverted_units: Vec::new(),
            unit_designations: Vec::new(),
            null_screened: Vec::new(),
        });
    }

    let mut prepared = prepare_generic_curves(
        depth,
        curves,
        ms_per_ft_meaning,
        undeclared_drho_unit,
    )?;
    apply_sentinel_source_notes(&mut prepared.curves, sentinel_candidates, sentinel_decision);
    let null_screened = db::with_txn(conn, |conn| {
        if let Some(verdict) = sampling_verdict {
            record_import_set_sampling(conn, well_id, set_name, verdict)?;
        }
        write_prepared_generic_curves_in_transaction(conn, well_id, depth, set_name, &prepared.curves)
    })?;
    Ok(GenericCurveImportReport {
        null_screened,
        curves_written: prepared.curves.len(),
        rows: depth.len(),
        unit_conversions: prepared.unit_conversions,
        unconverted_units: prepared.unconverted_units,
        unit_designations: prepared.unit_designations,
    })
}

fn prepare_generic_curves(
    depth: &[f32],
    curves: &[parsers::RawLasCurve],
    ms_per_ft_meaning: Option<crate::curves::MsPerFtMeaning>,
    undeclared_drho_unit: Option<&str>,
) -> db::DbResult<PreparedGenericImport> {
    let mut prepared = Vec::with_capacity(curves.len());
    let mut unit_conversions = Vec::new();
    let mut unconverted_units = Vec::new();
    let mut unit_designations = Vec::new();
    for raw in curves {
        // SB-CLY-055 (DEC-036 constraint 3): a CLY provenance token curve is validated
        // against the registry BEFORE anything is written - an unknown code REFUSES the
        // import, naming the code and the registry version it could not resolve. A token
        // whose meaning is not in the reader's table is not a token, and silently passing
        // it through would let a later vocabulary's code be read as whatever this version
        // happens to assign. Runs pre-transaction in both LAS paths, so a refused delivery
        // writes nothing at all.
        if raw.mnemonic.trim().eq_ignore_ascii_case("VSH_PROV") {
            if let Some(unknown) = raw.values.iter().find(|value| {
                value.is_finite() && crate::param_sources::cly_prov_token(**value).is_none()
            }) {
                return Err(db::DbError::Invalid(format!(
                    "curve {} carries code {unknown}, which CLY provenance registry v{} does \
                     not define - the import is refused rather than reading an unknown token \
                     as something this version happens to assign. Re-import with a build that \
                     knows the writing registry's version, or correct the curve.",
                    raw.mnemonic,
                    crate::param_sources::CLY_PROV_REGISTRY_VERSION
                )));
            }
        }
        let mut values = raw.values.clone();
        // Align to the depth column length (defensive: malformed files can short a column).
        if values.len() != depth.len() {
            values.resize(depth.len(), f32::NAN);
        }
        let source_unit_missing =
            crate::curves::unit_token_state(raw.unit.as_deref())
                == crate::curves::UnitTokenState::MissingUnit;
        let mut unit = match crate::curves::unit_token_state(raw.unit.as_deref()) {
            crate::curves::UnitTokenState::MissingUnit => None,
            _ => raw.unit.clone(),
        };
        if source_unit_missing
            && crate::curves::family_for(&raw.mnemonic)
                .is_some_and(|family| family.family == "DRHO")
        {
            let stated = undeclared_drho_unit.map(str::trim).filter(|value| !value.is_empty()).ok_or_else(|| {
                db::DbError::LengthMismatch(format!(
                    "DRHO-family curve {} has no declared unit; state g/cc or kg/m3 before import",
                    raw.mnemonic
                ))
            })?;
            let token = crate::curves::resolve_unit_token(stated).ok_or_else(|| {
                db::DbError::LengthMismatch(format!(
                    "DRHO-family curve {} has unsupported stated unit '{stated}'; state g/cc or kg/m3",
                    raw.mnemonic
                ))
            })?;
            if !matches!(token.canonical_unit, "g/cc" | "kg/m3") {
                return Err(db::DbError::LengthMismatch(format!(
                    "DRHO-family curve {} has incompatible stated unit '{stated}'; state g/cc or kg/m3",
                    raw.mnemonic
                )));
            }
            unit = Some(token.canonical_unit.to_string());
        }
        let resolved_ms_per_ft = crate::curves::is_ms_per_ft(unit.as_deref());
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
            crate::curves::family_for_import(&raw.mnemonic, unit.as_deref())
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
                unit.as_deref(),
                &mut values,
            ) {
                unit = Some(f.canonical_unit.to_string());
                unit_conversions.push(conversion);
            } else if let Some(unconverted) = crate::curves::unconverted_unit(
                &raw.mnemonic,
                Some(f.family),
                unit.as_deref(),
            ) {
                unconverted_units.push(unconverted);
            }
        } else if let Some(unconverted) =
            crate::curves::unconverted_unit(&raw.mnemonic, None, unit.as_deref())
        {
            unconverted_units.push(unconverted);
        }
        prepared.push(PreparedGenericCurve {
            source_note: None,
            mnemonic: raw.mnemonic.clone(),
            unit,
            family,
            values,
        });
        if source_unit_missing && family == Some("DRHO") {
            unit_designations.push(crate::curves::UnitDesignation {
                curve: raw.mnemonic.clone(),
                declared_unit: "ABSENT".to_string(),
                meaning: "explicit_density_correction_unit".to_string(),
                recorded_unit: prepared
                    .last()
                    .and_then(|curve| curve.unit.clone())
                    .unwrap_or_default(),
                family: Some("DRHO".to_string()),
            });
        }
    }
    Ok(PreparedGenericImport {
        curves: prepared,
        unit_conversions,
        unconverted_units,
        unit_designations,
    })
}

/// Transaction-free generic writer. The attach path wraps this as its whole delivery; the
/// new-well path places it inside the same outer transaction as the well and standard view.
fn write_prepared_generic_curves_in_transaction(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    set_name: &str,
    prepared: &[PreparedGenericCurve],
) -> db::DbResult<Vec<(String, usize)>> {
    let mut curve_ids = Vec::with_capacity(prepared.len());
    for curve in prepared {
        curve_ids.push(db::upsert_curve_meta(
            conn,
            well_id,
            set_name,
            &curve.mnemonic,
            curve.unit.as_deref(),
            curve.family,
            // SB-CLY-034 (DEC-037): the sentinel answer travels on the curve's own source.
            Some(curve.source_note.as_deref().unwrap_or("LAS import")),
            None,
        )?);
    }
    let batch: Vec<(&str, &[f32])> = curve_ids
        .iter()
        .zip(prepared.iter())
        .map(|(curve_id, curve)| (curve_id.as_str(), curve.values.as_slice()))
        .collect();
    // SB-DBM-030: map the store's flag channel back to the delivery's own mnemonics - the
    // importer's warning must name the curve the user delivered, not an internal id.
    let screened = db::insert_curve_samples_batch_in_transaction(conn, depth, &batch)?;
    Ok(screened
        .into_iter()
        .map(|(curve_id, count)| {
            let mnemonic = curve_ids
                .iter()
                .position(|id| *id == curve_id)
                .map(|index| prepared[index].mnemonic.clone())
                .unwrap_or(curve_id);
            (mnemonic, count)
        })
        .collect())
}

/// Parses a deviation-survey CSV (columns MD/INC/AZI, alias-tolerant) and stores the
/// computed minimum-curvature TVD/TVDSS in `well_path` for one well. `datum_elevation`
/// (KB above MSL) is used for TVDSS; if omitted, the well's `kb` is used, else 0.
///
/// `survey_name` (T-IMP-12) versions the survey: a definitive survey imported over a
/// preliminary one becomes a SECOND survey (auto-suffixed if the name is taken), not a
/// replacement, and the new one becomes active — so the TVD/TVDSS materialized below is
/// the geometry the user just delivered, while the old survey stays switchable.
///
/// `depth_unit` is the unit the FILE's MD column is written in (audit finding 8). This was
/// the only depth-bearing importer with no unit resolution at all: an 8000 ft survey imported
/// into a metre-declared project stored 8000, putting every station 3.28084x too deep, and the
/// error does not stop there — `materialize_tvd_curves` below writes TVD/TVDSS onto the log
/// grid, which then feeds `sw_height`, the saturation-height fits and the TVDSS correlation
/// view. `None` means "already the project unit", which is what every existing caller and every
/// saved workflow sends, so nothing that worked before changes.
///
/// Deliberately NOT applied to `datum_elevation`: that is typed by the user in the dialog,
/// which labels it in the project's own unit, and `wells.kb` is already stored in it. A file's
/// unit governs the file's numbers and nothing else.
pub fn import_deviation_csv(
    conn: &Connection,
    well_id: &str,
    path: &str,
    datum_elevation: Option<f32>,
    survey_name: Option<&str>,
    depth_unit: Option<&str>,
) -> CoreImportResult {
    let exists: bool = conn
        .query_row("SELECT 1 FROM wells WHERE well_id = ?1", params![well_id], |_| Ok(true))
        .unwrap_or(false);
    if !exists {
        return CoreImportResult { path: path.to_string(), rows: 0, error: Some(format!("unknown well '{well_id}'")), index_resolution: None };
    }

    let mut survey = match parsers::parse_deviation_csv(path) {
        Ok(s) => s,
        Err(e) => return CoreImportResult { path: path.to_string(), rows: 0, error: Some(e.to_string()), index_resolution: None },
    };
    if survey.md.is_empty() {
        return CoreImportResult { path: path.to_string(), rows: 0, error: Some("no survey stations found".into()), index_resolution: None };
    }

    // The file's MD column, brought onto the project's declared depth scale before any geometry
    // is computed — the same one-line discipline `import_core_table` already follows. Done here
    // rather than after minimum curvature so TVD, TVDSS and the stored station MDs all come out
    // in the project unit together, which is the only way they can stay consistent with each
    // other and with the log grid `materialize_tvd_curves` writes them onto.
    let project_unit = match crate::units::require_project_depth_unit(conn, "deviation-survey import") {
        Ok(unit) => unit,
        Err(error) => {
            return CoreImportResult { path: path.to_string(), rows: 0, error: Some(error), index_resolution: None }
        }
    };
    let file_unit = depth_unit.and_then(crate::units::DepthUnit::parse).unwrap_or(project_unit);
    crate::units::convert_depths(&mut survey.md, file_unit, project_unit);

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
    // back to a sea-level datum → TVDSS = TVD) or NaN outside the survey's MD range, and no
    // recourse via the Curve Catalog's Promote (it is disabled on a "served by computed" row).
    // So: only materialize a name the well does NOT already resolve from an import, and clear
    // any prior survey-derived computed curve when an import IS present, so the import wins.
    let mut output_curves: Vec<(String, Vec<f32>)> = Vec::new();
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
            output_curves.push((name.to_string(), values.clone()));
        }
    }
    if output_curves.is_empty() {
        return Ok(0);
    }
    let survey = db::list_surveys(conn, well_id)?
        .into_iter()
        .find(|survey| survey.active)
        .ok_or_else(|| {
            db::DbError::LengthMismatch("the active deviation survey has no custody record".into())
        })?;
    let source = survey
        .source
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            db::DbError::LengthMismatch("the active deviation survey has no source string".into())
        })?;
    let imported_at = survey
        .imported_at
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            db::DbError::LengthMismatch(
                "the active deviation survey has no import timestamp".into(),
            )
        })?;
    let actor = crate::equations::AncestryActor {
        kind: crate::equations::AncestryActorKind::Automated,
        identity: "SANDIBUMI_DEVIATION_SURVEY".into(),
    };
    let parameters_json = serde_json::json!({
        "survey_name": survey.survey_name,
        "datum": survey.datum.map(serde_json::Value::from).unwrap_or_else(|| serde_json::json!("ABSENT")),
        "interpolation": "linear_between_stored_minimum_curvature_stations",
    });
    let parameters: Vec<_> = parameters_json
        .as_object()
        .expect("constructed as an object")
        .iter()
        .map(|(name, value)| crate::equations::AncestryParameter {
            name: name.clone(),
            value: value.clone(),
            source: source.to_string(),
            resolution: Some(crate::equations::ParameterResolution::Explicit),
            manifest_version: None,
            decision: None,
        })
        .collect();
    let module = "deviation:materialize_tvd";
    let ancestry = crate::equations::CurveAncestry {
        schema_version: crate::equations::CURVE_ANCESTRY_SCHEMA_VERSION,
        method_derivation: crate::equations::method_derivation_citation(module),
        module: module.into(),
        module_version: env!("CARGO_PKG_VERSION").into(),
        inputs: vec![crate::equations::AncestryInput {
            well_id: well_id.to_string(),
            argument: "active deviation survey".into(),
            curve: "MD/INC/AZI/TVD/TVDSS".into(),
            log_set: survey.survey_name.clone(),
            set_version: None,
            set_id: format!("survey:{}:{}:{}", well_id, survey.survey_name, imported_at),
            chosen_curve_id: Some(format!(
                "survey:{}:{}:{}",
                well_id, survey.survey_name, imported_at
            )),
            rule: Some(crate::equations::CurveResolutionRule::ExplicitName),
            rejected_candidates: Vec::new(),
        }],
        parameter_state: crate::equations::parameter_state_for(&parameters),
        parameters,
        zone_scope: crate::equations::AncestryZoneScope::WholeWell,
        actor,
        timestamp_utc_ms: crate::equations::ancestry_timestamp_utc_ms()
            .map_err(db::DbError::LengthMismatch)?,
        outputs: output_curves
            .iter()
            .map(|(curve, _)| crate::equations::AncestryOutput {
                curve: curve.clone(),
                derivation: format!("{module}:{curve}"),
            })
            .collect(),
        depth_frame: None,
        zone_set: None,
        stochastic: None,
        applied_model: None,
        physics_attributes: Vec::new(),
    };
    let log_spec = crate::equations::CompleteLogSetSpec::try_new_with_legacy(
        "DEVIATION",
        ancestry,
        parameters_json,
        "[]",
    )
    .map_err(db::DbError::LengthMismatch)?;
    let (set_id, _) = crate::equations::create_complete_log_set(conn, well_id, &log_spec)
        .map_err(db::DbError::LengthMismatch)?;
    let refs = output_curves
        .iter()
        .map(|(name, values)| (name.as_str(), values.as_slice()))
        .collect::<Vec<_>>();
    crate::equations::write_computed_curves_with_ancestry(conn, well_id, &depth, &refs, &set_id)
        .map_err(db::DbError::LengthMismatch)?;
            Ok(depth.len())
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
pub fn import_core_csv(
    conn: &Connection,
    well_id: &str,
    path: &str,
    depth_datum: &str,
) -> CoreImportResult {
    import_core_csv_with_depth_column(conn, well_id, path, None, depth_datum)
}

/// SB-DBM-031 (DEC-073 item 5): every delivery import DECLARES its depth datum, once,
/// for the whole delivery set. Validated against the shipped vocabulary before anything
/// is written, so a typo refuses the import instead of importing then failing to declare.
fn validated_datum(datum: &str) -> Result<&'static str, String> {
    crate::schema_vocab::DepthDatum::parse(datum)
        .map(|parsed| parsed.as_str())
        .ok_or_else(|| {
            format!(
                "'{datum}' is not a depth datum; the vocabulary is MD | TVD | TVDSS | TVDKB | TWT | OWT | CDEPTH (SB-DBM-031)"
            )
        })
}

pub fn import_core_csv_with_depth_column(
    conn: &Connection,
    well_id: &str,
    path: &str,
    designated_depth_column: Option<usize>,
    depth_datum: &str,
) -> CoreImportResult {
    let depth_datum = match validated_datum(depth_datum) {
        Ok(datum) => datum,
        Err(error) => return CoreImportResult { path: path.to_string(), rows: 0, error: Some(error), index_resolution: None },
    };
    let exists: bool = conn
        .query_row("SELECT 1 FROM wells WHERE well_id = ?1", params![well_id], |_| Ok(true))
        .unwrap_or(false);
    if !exists {
        return CoreImportResult { path: path.to_string(), rows: 0, error: Some(format!("unknown well '{well_id}'")), index_resolution: None };
    }

    let columns = match parsers::parse_core_csv_with_depth_column(path, designated_depth_column) {
        Ok(c) => c,
        Err(e) => {
            return CoreImportResult { path: path.to_string(), rows: 0, error: Some(e.to_string()), index_resolution: None };
    }};
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
        Ok(()) => {
            if let Err(error) = db::declare_set_datum(conn, "core_sets", well_id, None, &set, depth_datum) {
                return CoreImportResult { path: path.to_string(), rows: 0, error: Some(error.to_string()), index_resolution };
            }
            CoreImportResult { path: path.to_string(), rows, error: None, index_resolution }
        }
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
    depth_datum: &str,
) -> CoreTableImportResult {
    let datum = match validated_datum(depth_datum) {
        Ok(datum) => datum,
        Err(error) => {
            return CoreTableImportResult {
                path: path.to_string(),
                rows_imported: 0,
                wells_imported: 0,
                outcomes: vec![],
                skipped_blank_well: 0,
                extra_rows: 0,
                extra_items: vec![],
                precision: parsers::SamplePrecisionReport::new("f64 numeric parse", "f32 storage", 0),
                error: Some(error),
            }
        }
    };
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

    let project_unit = match crate::units::require_project_depth_unit(conn, "core-table import") {
        Ok(unit) => unit,
        Err(error) => return fail(error),
    };
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
                if has_core_measurement {
                    if let Err(error) = db::declare_set_datum(conn, "core_sets", well_id, None, &set, datum) {
                        outcomes.push(CoreWellOutcome {
                            well_name: well_name.to_string(),
                            rows: list.len(),
                            imported: 0,
                            set_name: Some(set.clone()),
                            problem: Some(error.to_string()),
                        });
                        return;
                    }
                }
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
                        Ok(()) => {
                            // The extras ride the core delivery, so they carry its datum too.
                            let _ = db::declare_set_datum(conn, "aux_sets", well_id, Some(&extras_dataset), &aux_set, datum);
                            extra_rows += aux.len();
                        }
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
pub fn import_scal_csv(
    conn: &Connection,
    well_id: &str,
    path: &str,
    ift_lab: f64,
    depth_datum: &str,
) -> ScalImportResult {
    import_scal_files(conn, well_id, &[path.to_string()], "long", "", ift_lab, None, false, depth_datum, None)
}

/// Multi-file, multi-format SCAL Pc import. Each file is parsed with `format` — "long"
/// (flat Pc/Sw CSV), "porous_plate" (Corelab-style wide table: pressure columns × plug
/// rows), "centrifuge" (per-plug key-value blocks + Pc/Sw tables), or "auto" to sniff
/// each file — so a set of single-plug centrifuge exports imports in one shot.
///
/// `depth_unit` is the unit the FILES quote their plug depths in ("m"/"ft"); `None` means the
/// project's own, which is what every import before audit finding 8 assumed. Unlike the tops
/// importer there is no file declaration to fall back on — a Pc export carries no units row that
/// is reliably attached to the depth column — but unlike tops this import has a dialog, so the
/// user is asked outright, which is better evidence than any header sniff.
///
/// The files
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
    depth_datum: &str,
    depth_unit: Option<&str>,
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

    // Audit finding 8, third site. A Pc delivery quoting its plug depths in feet, imported into
    // a metre project, filed every plug 3.28084x too deep — and a Pc curve is read AT a depth:
    // Thomeer and the J-fit QC pair each plug with the log's porosity and permeability there, and
    // `sw_height` carries the fitted A/B back onto that same interval. The depths convert BEFORE
    // the core record is applied below, because `core_depth_pairs` are already on the project's
    // scale — mapping a foot depth through a metre correction would be two errors, not one.
    let project_unit = match crate::units::require_project_depth_unit(conn, "SCAL import") {
        Ok(unit) => unit,
        Err(error) => return fail(error),
    };
    let file_unit = depth_unit.and_then(crate::units::DepthUnit::parse).unwrap_or(project_unit);
    if file_unit != project_unit {
        for rec in &mut records {
            if let Some(d) = rec.depth {
                rec.depth = Some(crate::units::convert_depth(d as f64, file_unit, project_unit) as f32);
            }
        }
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
    let datum = match validated_datum(depth_datum) {
        Ok(datum) => datum,
        Err(error) => return fail(error),
    };
    let set = match db::resolve_scal_set_name(conn, well_id, &desired) {
        Ok(s) => s,
        Err(e) => return fail(e.to_string()),
    };
    if let Err(e) = db::insert_scal_pc(conn, well_id, &set, Some(&joined), &rows) {
        return fail(e.to_string());
    }
    if let Err(e) = db::declare_set_datum(conn, "scal_sets", well_id, None, &set, datum) {
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
    /// The unit the file's depths were READ as ("m"/"ft") — the explicit argument, else what the
    /// file declared, else the project's own. Reported so a conversion is never silent: a tops
    /// import that quietly moved every marker is the one thing worse than one that did not.
    #[serde(default)]
    pub depth_unit: Option<String>,
    pub error: Option<String>,
}

/// Imports formation tops from a CSV/TXT file. Files with a WELL column update every
/// matching well (name match, case-insensitive); files without one need
/// `default_well_id` (the selected well). Tops upsert by (well, name) — re-import
/// updates depths, existing colors are kept.
pub fn import_tops_file(
    conn: &Connection,
    default_well_id: Option<&str>,
    path: &str,
    depth_unit: Option<&str>,
) -> TopsImportResult {
    let fail = |e: String| TopsImportResult {
        path: path.to_string(),
        tops_written: 0,
        wells_matched: 0,
        unmatched_wells: vec![],
        depth_unit: None,
        error: Some(e),
    };
    let (has_well_column, declared_unit, records) = match parsers::parse_tops_file(path) {
        Ok(r) => r,
        Err(e) => return fail(e.to_string()),
    };

    // Audit finding 8, second site. A tops file in feet read into a metre project put every
    // marker 3.28084x too deep — and a top is not one number, it is the boundary of a zone, so
    // the error propagates into every zone parameter, every pay summary and every report drawn
    // from them. Precedence: an explicit argument, else what the FILE declares on the depth
    // column it was read from, else the project's own unit (which is what every import before
    // this one assumed, so nothing already working changes).
    let project_unit = match crate::units::require_project_depth_unit(conn, "tops import") {
        Ok(unit) => unit,
        Err(error) => return fail(error),
    };
    let file_unit = depth_unit
        .and_then(crate::units::DepthUnit::parse)
        .or_else(|| declared_unit.and_then(crate::units::DepthUnit::parse))
        .unwrap_or(project_unit);
    let mut records = records;
    for rec in &mut records {
        rec.depth = crate::units::convert_depth(rec.depth as f64, file_unit, project_unit) as f32;
    }
    let records = records;

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
        match db::upsert_top_with_datum(
            conn,
            &well_id,
            &rec.top_name,
            rec.depth,
            rec.depth_datum,
            None,
        ) {
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
        depth_unit: Some(file_unit.label().to_string()),
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
    depth_datum: &str,
    depth_unit: Option<&str>,
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
    let datum = match validated_datum(depth_datum) {
        Ok(datum) => datum,
        Err(error) => return fail(error),
    };
    let desired_set = set_name
        .map(|s| s.trim().to_uppercase().replace(' ', "_"))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "RAW".to_string());

    let mut data = match parsers::parse_interval_file(path) {
        Ok(d) => d,
        Err(e) => return fail(e.to_string()),
    };

    // Audit finding 8, fourth and last site. A point dataset — XRD, CEC, oil show, petrography,
    // perforations — delivered in feet and read into a metre project filed every sample 3.28084x
    // too deep, so a mineral count sat against the wrong sand and a perforation against the wrong
    // interval. Precedence and order both follow the earlier sites: an explicit argument, else
    // what the FILE declares on its own TOP column, else the project's unit; and the conversion
    // lands HERE, before the core depth record is applied below, because `core_depth_pairs` are
    // already on the project's scale.
    //
    // An interval converts at BOTH ends and stays an interval: a thickness scaled at one end only
    // is not a shallower sample, it is a sample of a different thickness.
    let project_unit = match crate::units::require_project_depth_unit(conn, "point-data import") {
        Ok(unit) => unit,
        Err(error) => return fail(error),
    };
    let file_unit = depth_unit
        .and_then(crate::units::DepthUnit::parse)
        .or_else(|| data.depth_unit.and_then(crate::units::DepthUnit::parse))
        .unwrap_or(project_unit);
    if file_unit != project_unit {
        let to_project = |d: f32| crate::units::convert_depth(d as f64, file_unit, project_unit) as f32;
        for (top, base, _) in &mut data.rows {
            *top = to_project(*top);
            *base = base.map(to_project);
        }
    }
    let data = data;

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
                            if let Err(e) = db::declare_set_datum(conn, &"aux_sets", &ids[0], Some(&dataset), &set, datum) {
                                notes.push(format!("{name}: {e}"));
                                continue;
                            }
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
                if let Err(e) = db::declare_set_datum(conn, "aux_sets", well_id, Some(&dataset), &set, datum) {
                    return fail(e.to_string());
                }
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

    fn import_null_mode_fixture(
        conn: &Connection,
        label: &str,
        channel_nulls: parsers::ChannelNullValues,
    ) -> ImportResult {
        let path = std::env::temp_dir().join(format!(
            "sandibumi-null-mode-{label}-{}.las",
            Uuid::new_v4()
        ));
        let las = format!(
            "~Version\nVERS. 2.0\n~Well\nWELL. NULL_MODE_{label}\nNULL. -999.25\n~Curve\nDEPT.M : Depth\nPWF1. : Waveform amplitude\n~Ascii\n1000.0 -999.25\n1001.0 12.5\n"
        );
        std::fs::write(&path, las).unwrap();
        let options = LasImportOptions {
            channel_nulls,
            ..LasImportOptions::default()
        };
        let result = import_las_files_with(
            conn,
            &[path.to_string_lossy().to_string()],
            None,
            &options,
        )
        .remove(0);
        std::fs::remove_file(path).unwrap();
        result
    }

    /// **A channel declared no null preserves a sentinel-shaped amplitude and reports no null.**
    /// `SB-DIO-003` / `SB-DIO-T04` CORRECTNESS. Source: 21_data-io.md D-3, section 5.2 and
    /// T04 identify `-999.25` as a genuine array/waveform amplitude when `NoNull` is declared.
    #[test]
    fn a_channel_declared_no_null_preserves_a_sentinel_shaped_amplitude_and_reports_no_null() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let mut channel_nulls = parsers::ChannelNullValues::new();
        channel_nulls.insert(
            "PWF1".into(),
            parsers::ChannelNullMode::NoNull(parsers::NoNullMarker::NoNull),
        );

        let result = import_null_mode_fixture(&conn, "DECLARED", channel_nulls);

        assert!(result.error.is_none(), "{:?}", result.error);
        let resolution = result
            .null_resolutions
            .iter()
            .find(|entry| entry.channel == "PWF1")
            .expect("the import result names the resolved source channel");
        assert_eq!(resolution.mode, parsers::ChannelNullResolutionMode::NoNull);
        assert!(resolution.values.is_empty(), "NoNull cannot carry an invented sentinel list");
        let stored: f32 = conn
            .query_row(
                "SELECT s.value FROM curve_samples s JOIN curve_meta m ON m.curve_id = s.curve_id
                 WHERE m.mnemonic = 'PWF1' ORDER BY s.depth LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored, -999.25, "the cited genuine amplitude must survive as data");
    }

    /// **An unset channel screens the same sentinel-shaped amplitude and reports unset.**
    /// `SB-DIO-003` / `SB-DIO-T05` CORRECTNESS. Source: 21_data-io.md T05 requires the same
    /// channel without a declaration to use ordinary screening and expose the difference.
    #[test]
    fn an_unset_channel_screens_the_same_sentinel_shaped_amplitude_and_reports_unset() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();

        let result = import_null_mode_fixture(&conn, "UNSET", parsers::ChannelNullValues::new());

        assert!(result.error.is_none(), "{:?}", result.error);
        let resolution = result
            .null_resolutions
            .iter()
            .find(|entry| entry.channel == "PWF1")
            .expect("the import result names even an unset source channel");
        assert_eq!(resolution.mode, parsers::ChannelNullResolutionMode::Unset);
        assert!(resolution.values.is_empty(), "unset cannot invent a per-channel sentinel list");
        let stored: Option<f32> = conn
            .query_row(
                "SELECT s.value FROM curve_samples s JOIN curve_meta m ON m.curve_id = s.curve_id
                 WHERE m.mnemonic = 'PWF1' ORDER BY s.depth LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        // SB-DBM-030 strengthened this pin: the missing marker is SQL NULL at the store (the
        // reader hands the frontend f32::NAN), never a float that a query could average.
        assert!(stored.is_none(), "ordinary screening must bind SQL NULL at the store");
    }

    /// A normal LAS delivery is one commit boundary. Duplicate source mnemonics force the
    /// generic Arrow insert to fail on (curve_id, depth) only after the well row, standard
    /// projection, metadata, and staged samples have all been touched. None may survive.
    #[test]
    fn new_well_rolls_back_when_all_channel_insert_fails() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let columns = CurveColumns {
            well_name: Some("ROLLBACK-1".into()),
            well_headers: Vec::new(),
            las_version: Some("2.0".into()),
            unread_sections: Vec::new(),
            section_policy: parsers::LAS_SECTION_POLICY_ID.to_string(),
            section_handling: Vec::new(),
            text_encoding: "test fixture".into(),
            depth_unit: Some("M".into()),
            declared_step: Some("0.5".into()),
            declared_step_mismatch_note: None,
            depth: vec![1000.0, 1000.5],
            gr: vec![40.0, 41.0],
            res: vec![2.0, 2.1],
            nphi: vec![0.2, 0.21],
            rhob: vec![2.4, 2.41],
            dt: vec![80.0, 81.0],
            sp: vec![f32::NAN; 2],
            raw_curves: vec![
                parsers::RawLasCurve {
                    mnemonic: "PEF".into(),
                    unit: Some("B/E".into()),
                    values: vec![4.0, 4.1],
                },
                parsers::RawLasCurve {
                    mnemonic: "PEF".into(),
                    unit: Some("B/E".into()),
                    values: vec![5.0, 5.1],
                },
            ],
            alias_decisions: Vec::new(),
            null_resolutions: Vec::new(),
            index_resolution: None,
            unit_designations: Vec::new(),
            undeclared_sentinel_candidates: Vec::new(),
        };

        let result = insert_parsed_well(
            &conn,
            "duplicate-mnemonic.las".into(),
            "ROLLBACK-1".into(),
            columns,
            &LasImportOptions::default(),
        );

        assert!(result.error.is_some(), "an incomplete delivery is an import failure");
        for table in ["wells", "standard_curves", "curve_meta", "curve_samples"] {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get(0))
                .unwrap();
            assert_eq!(count, 0, "{table} must roll back with the delivery");
        }
        assert_eq!(
            crate::units::project_depth_unit(&conn).unwrap(),
            None,
            "a failed first delivery cannot declare the project's depth unit"
        );
    }

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

        let result = import_core_csv(&conn, &ids, csv, "MD");
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
        let again = import_core_csv(&conn, &ids, csv, "MD");
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
        let bad = import_core_csv(&conn, "no-such-well", csv, "MD");
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

    /// SB-CLY-055 (DEC-036 constraint 3): an unknown code on RE-IMPORT refuses the whole
    /// delivery BEFORE anything is written, naming the code and the registry version it
    /// could not resolve - a token whose meaning is not in the reader's table is not a
    /// token. Registry codes and the MISSING absences import untouched.
    #[test]
    fn an_unknown_provenance_code_refuses_the_import_naming_the_code_and_the_registry_version() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let cols = |prov: Vec<f32>| CurveColumns {
            well_name: None,
            well_headers: Vec::new(),
            las_version: None,
            unread_sections: Vec::new(),
            section_policy: parsers::LAS_SECTION_POLICY_ID.to_string(),
            section_handling: Vec::new(),
            text_encoding: "test fixture".into(),
            depth_unit: Some("M".into()),
            declared_step: None,
            declared_step_mismatch_note: None,
            depth: vec![1000.0, 1000.5, 1001.0],
            gr: vec![40.0, 45.0, 50.0],
            res: vec![f32::NAN; 3],
            nphi: vec![f32::NAN; 3],
            rhob: vec![f32::NAN; 3],
            dt: vec![f32::NAN; 3],
            sp: vec![f32::NAN; 3],
            raw_curves: vec![parsers::RawLasCurve {
                mnemonic: "VSH_PROV".into(),
                unit: Some("flag".into()),
                values: prov,
            }],
            alias_decisions: Vec::new(),
            null_resolutions: Vec::new(),
            index_resolution: None,
            unit_designations: Vec::new(),
            undeclared_sentinel_candidates: Vec::new(),
        };

        // A later vocabulary's code: refused by name, and NOTHING is written.
        let refused = insert_parsed_well(
            &conn,
            "bad.las".into(),
            "CLY-RT-BAD".into(),
            cols(vec![0.0, 7.0, 1.0]),
            &LasImportOptions::default(),
        );
        let error = refused.error.expect("an unknown token code must refuse the import");
        assert!(error.contains('7'), "the code is named: {error}");
        assert!(error.contains("registry v1"), "the registry version is named: {error}");
        let wells: i64 =
            conn.query_row("SELECT COUNT(*) FROM wells", [], |r| r.get(0)).unwrap();
        assert_eq!(wells, 0, "a refused delivery writes nothing at all");

        // Every registry code plus a MISSING absence imports untouched.
        let ok = insert_parsed_well(
            &conn,
            "good.las".into(),
            "CLY-RT-OK".into(),
            cols(vec![3.0, f32::NAN, 4.0]),
            &LasImportOptions::default(),
        );
        assert!(ok.error.is_none(), "{:?}", ok.error);
    }

    /// A second LAS import of a well whose name already exists still creates a separate record
    /// (auto-merge needs a confirmation flow) but must surface a warning, not silently fragment.
    #[test]
    fn las_import_warns_on_duplicate_well_name() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();

        let cols = || CurveColumns {
            well_name: None,
            well_headers: Vec::new(),
            las_version: None,
            unread_sections: Vec::new(),
            section_policy: parsers::LAS_SECTION_POLICY_ID.to_string(),
            section_handling: Vec::new(),
            text_encoding: "test fixture".into(),
            depth_unit: Some("M".into()),
            declared_step: None,
            declared_step_mismatch_note: None,
            depth: vec![1000.0, 1000.5, 1001.0],
            gr: vec![40.0, 45.0, 50.0],
            res: vec![f32::NAN; 3],
            nphi: vec![f32::NAN; 3],
            rhob: vec![f32::NAN; 3],
            dt: vec![f32::NAN; 3],
            sp: vec![f32::NAN; 3],
            raw_curves: Vec::new(),
            alias_decisions: Vec::new(),
            null_resolutions: Vec::new(),
            index_resolution: None,
            unit_designations: Vec::new(),
            undeclared_sentinel_candidates: Vec::new(),
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

    /// CORRECTNESS — SB-INS-017 / SB-INS-T21. The distinct `mV` and `mv` source tokens,
    /// absent equivalence declaration, and required drift warning come from dossier section
    /// 2.3 and N-NEW-17. The two raw spellings must survive the product import and its stored
    /// metadata; only the registry-declared spelling may acquire a canonical interpretation.
    #[test]
    fn case_variant_unit_tokens_without_an_explicit_alias_remain_distinct_and_record_a_drift_warning(
    ) {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let path = std::env::temp_dir().join(format!(
            "sandibumi-unit-token-drift-{}-{}.las",
            std::process::id(),
            Uuid::new_v4()
        ));
        std::fs::write(
            &path,
            "~VERSION\nVERS. 2.0 :\n~WELL\nWELL. UNIT-TOKEN-DRIFT :\n\
             ~CURVE\nDEPT.M : depth\nRAW_A.mV : first observed token\n\
             RAW_B.mv : second observed token\n~ASCII\n1000.0 1.0 2.0\n",
        )
        .unwrap();

        let result = import_las_files(
            &conn,
            &[path.to_string_lossy().into_owned()],
            None,
        )
        .remove(0);
        std::fs::remove_file(&path).ok();

        assert!(result.error.is_none(), "fixture import failed: {:?}", result.error);
        let stored = conn
            .prepare(
                "SELECT mnemonic, unit FROM curve_meta \
                 WHERE mnemonic IN ('RAW_A', 'RAW_B') ORDER BY mnemonic",
            )
            .unwrap()
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            stored,
            vec![
                ("RAW_A".to_string(), "mV".to_string()),
                ("RAW_B".to_string(), "mv".to_string()),
            ],
            "source tokens must not be collapsed in stored metadata"
        );

        let first = result
            .unit_tokens
            .iter()
            .find(|token| token.curve == "RAW_A")
            .expect("first raw token is reported");
        let second = result
            .unit_tokens
            .iter()
            .find(|token| token.curve == "RAW_B")
            .expect("second raw token is reported");
        assert_eq!(first.raw_token.as_deref(), Some("mV"));
        assert_eq!(first.canonical_unit.as_deref(), Some("mV"));
        assert_eq!(second.raw_token.as_deref(), Some("mv"));
        assert_eq!(second.canonical_unit, None);
        assert!(result.unit_token_warnings.iter().any(|warning| {
            warning.contains("mV")
                && warning.contains("mv")
                && warning.contains("no explicit alias")
        }));
    }

    /// CORRECTNESS — SB-INS-018 / SB-INS-T23. Absent, empty, placeholder and empty-to-empty
    /// fixtures plus the required zero-registration result come from dossier section 2.3 and
    /// N-NEW-23/N-NEW-28. The valid row is the opposite-side control: a loader that labelled
    /// every row missing would otherwise pass the four refusal fixtures lazily.
    #[test]
    fn absent_empty_placeholder_and_empty_to_empty_units_share_one_missing_state_and_register_zero_mappings(
    ) {
        let source_units = [None, Some(""), Some("-"), Some("?")];
        let raw = source_units
            .iter()
            .enumerate()
            .map(|(index, unit)| parsers::RawLasCurve {
                mnemonic: format!("MISSING_UNIT_{index}"),
                unit: unit.map(str::to_string),
                values: vec![index as f32],
            })
            .collect::<Vec<_>>();
        let observed = raw
            .iter()
            .map(|curve| (curve.mnemonic.clone(), curve.unit.clone()))
            .collect::<Vec<_>>();
        let (tokens, warnings) = crate::curves::observe_unit_tokens(&observed);
        assert!(warnings.is_empty(), "missing spellings are not vocabulary drift");
        assert_eq!(
            tokens
                .iter()
                .map(|token| token.state.clone())
                .collect::<Vec<_>>(),
            vec![crate::curves::UnitTokenState::MissingUnit; 4]
        );
        assert!(serde_json::to_value(&tokens).unwrap().as_array().unwrap().iter().all(|token| {
            token["state"] == "missing_unit"
                && token["canonical_unit"].is_null()
                && token["quantity_kind"].is_null()
        }));

        let prepared = prepare_generic_curves(&[1000.0], &raw, None, None).unwrap();
        assert!(
            prepared.curves.iter().all(|curve| curve.unit.is_none()),
            "the storage boundary must receive one absent unit, never a placeholder"
        );
        assert!(prepared.unit_conversions.is_empty());
        assert!(prepared.unconverted_units.is_empty());

        let missing_fixtures = [
            (None, Some("m")),
            (Some(""), Some("m")),
            (Some("-"), Some("?")),
            (Some(""), Some("")),
        ];
        let states = crate::curves::load_unit_mapping_rows(&missing_fixtures).unwrap();
        assert_eq!(
            states,
            vec![crate::curves::UnitMappingRowState::MissingUnit; 4]
        );
        assert_eq!(
            states
                .iter()
                .filter(|state| matches!(state, crate::curves::UnitMappingRowState::Registered(_)))
                .count(),
            0,
            "none of the missing spellings may register a bridge"
        );

        let valid = crate::curves::load_unit_mapping_rows(&[(Some("mm"), Some("in"))]).unwrap();
        assert!(matches!(
            valid.as_slice(),
            [crate::curves::UnitMappingRowState::Registered(
                crate::curves::ValidatedUnitBridge {
                    quantity_kind: crate::curves::QuantityKind::Length,
                    ..
                }
            )]
        ));
        assert!(crate::curves::UNIT_TOKENS
            .iter()
            .all(|entry| !["", "-", "?"].contains(&entry.token)));
    }

    /// SB-DIO-009 / SB-DIO-T14. The ordered NPHI aliases and finite-coverage
    /// tie-break are specified in `docs/PRD_v2/21_data-io.md` §5.3.
    /// SB-DBM-030's flag-channel half, through the production LAS import: a screened value is
    /// named in the import's own warning by the DELIVERED mnemonic with its count, lands as
    /// SQL NULL in BOTH projections of the delivery (standard and generic - one screened and
    /// one kept would be two truths about the same sample), and the declared LAS NULL keeps
    /// resolving through its own declared mechanism, not this screen.
    #[test]
    fn a_screened_import_names_the_curve_and_count_in_its_own_warning_never_silently() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let path = std::env::temp_dir().join("sandibumi_null_screen_flag_channel.las");
        std::fs::write(
            &path,
            "~VERSION\nVERS. 2.0 :\n~WELL\nNULL. -999.25 :\nWELL. SANDI-SCREEN :\n\
             ~CURVE\nDEPT.M :\nGR.API :\n~ASCII\n\
             1000.0 50.0\n1000.5 -1.0E30\n1001.0 -999.25\n",
        )
        .unwrap();
        let result =
            import_las_files(&conn, &[path.to_str().unwrap().to_string()], None).remove(0);
        std::fs::remove_file(&path).ok();
        assert!(result.error.is_none(), "the fixture must import: {:?}", result.error);
        // A. The flag channel: the warning names the delivered mnemonic and the count.
        let warning = result.warning.clone().unwrap_or_default();
        assert!(
            warning.contains("null screen: 1 large-negative sample(s) on GR stored as missing"),
            "got warning: {warning}"
        );
        // B. Both projections agree: the screened sample is SQL NULL in the standard
        //    projection and in the generic store.
        let well_id = result.well_id.clone().unwrap();
        let std_nulls: i64 = conn
            .query_row(
                "SELECT count(*) FROM standard_curves WHERE well_id = ?1 AND gr IS NULL",
                params![well_id],
                |r| r.get(0),
            )
            .unwrap();
        // one screened sentinel + one declared null resolved at parse
        assert_eq!(std_nulls, 2);
        let generic_nulls: i64 = conn
            .query_row(
                "SELECT count(*) FROM curve_samples s JOIN curve_meta m ON m.curve_id = s.curve_id
                 WHERE m.well_id = ?1 AND m.mnemonic = 'GR' AND s.value IS NULL",
                params![well_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(generic_nulls, 2);
    }

    /// SB-CLY-034 (DEC-037): quarantine and ASK. An undeclared value equal to the known
    /// vendor bad-hole sentinel BLOCKS the import with a question naming the value, every
    /// affected curve and the sample count - nothing is written until a human answers.
    /// Conversion happens only on confirmation; KEEPING the values is recorded too (a kept
    /// sentinel is a finding about the delivery); and an explicit NoNull channel carrying
    /// the same value is NEVER offered - the DIO preserved-amplitude contract untouched.
    /// Precedence pinned from BOTH sides per constraint 3.
    #[test]
    fn an_undeclared_vendor_sentinel_blocks_the_import_and_converts_only_on_the_users_word() {
        let path = std::env::temp_dir().join("sandibumi_undeclared_sentinel.las");
        std::fs::write(
            &path,
            "~VERSION
VERS. 2.0 :
~WELL
NULL. -999.25 :
WELL. SANDI-SNT :
             ~CURVE
DEPT.M :
GR.API :
~ASCII
             1000.0 50.0
1000.5 -999.0
1001.0 60.0
",
        )
        .unwrap();
        let paths = [path.to_str().unwrap().to_string()];

        // A. No decision: the import BLOCKS with the structured question; nothing written.
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let result = import_las_files(&conn, &paths, None).remove(0);
        let question = result.sentinel_question.clone().expect("the question is asked");
        assert_eq!(question.value, -999.0);
        assert_eq!(question.curves.len(), 1);
        assert_eq!(question.curves[0].mnemonic, "GR");
        assert_eq!(question.curves[0].samples, 1, "the sample count is named");
        let error = result.error.clone().expect("a blocked file reads as not imported");
        assert!(error.contains("-999"), "the value is named: {error}");
        let wells: i64 =
            conn.query_row("SELECT count(*) FROM wells", [], |r| r.get(0)).unwrap();
        assert_eq!(wells, 0, "nothing was imported before the answer");

        // B. CONVERT on confirmation: the cell becomes absent, and the answer is recorded
        //    in the import warning AND on the curve's own provenance.
        let convert = LasImportOptions {
            undeclared_sentinel_decision: Some(SentinelDecision::Convert),
            ..LasImportOptions::default()
        };
        let result = import_las_files_with(&conn, &paths, None, &convert).remove(0);
        assert!(result.error.is_none(), "{:?}", result.error);
        assert!(result.sentinel_question.is_none());
        let warning = result.warning.clone().unwrap_or_default();
        assert!(
            warning.contains("converted to absent") && warning.contains("DEC-037"),
            "the answer is recorded: {warning}"
        );
        let well_id = result.well_id.clone().unwrap();
        let absent: i64 = conn
            .query_row(
                "SELECT count(*) FROM standard_curves WHERE well_id = ?1 AND gr IS NULL",
                params![well_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(absent, 1, "the confirmed sentinel is absent, not a shale volume");
        let source: String = conn
            .query_row(
                "SELECT source FROM curve_meta WHERE well_id = ?1 AND mnemonic = 'GR'",
                params![well_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            source.contains("converted to absent") && source.contains("DEC-037"),
            "the answer travels on the curve's provenance: {source}"
        );

        // C. KEEP on a fresh project: the values stay measurements, and THAT answer is
        //    recorded the same two ways.
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let keep = LasImportOptions {
            undeclared_sentinel_decision: Some(SentinelDecision::Keep),
            ..LasImportOptions::default()
        };
        let result = import_las_files_with(&conn, &paths, None, &keep).remove(0);
        assert!(result.error.is_none(), "{:?}", result.error);
        let warning = result.warning.clone().unwrap_or_default();
        assert!(warning.contains("KEPT"), "the keep is recorded: {warning}");
        let well_id = result.well_id.clone().unwrap();
        let kept: i64 = conn
            .query_row(
                "SELECT count(*) FROM standard_curves WHERE well_id = ?1 AND gr = -999.0",
                params![well_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(kept, 1, "a kept value stays a measurement");
        let source: String = conn
            .query_row(
                "SELECT source FROM curve_meta WHERE well_id = ?1 AND mnemonic = 'GR'",
                params![well_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(source.contains("kept as measurements"), "{source}");

        // D. The OTHER side of precedence: the same value on a channel DECLARED NoNull is
        //    never offered - no question, no note, the amplitude preserved as delivered.
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let mut channel_nulls = parsers::ChannelNullValues::new();
        channel_nulls.insert(
            "GR".to_string(),
            parsers::ChannelNullMode::NoNull(parsers::NoNullMarker::NoNull),
        );
        let no_null = LasImportOptions { channel_nulls, ..LasImportOptions::default() };
        let result = import_las_files_with(&conn, &paths, None, &no_null).remove(0);
        assert!(result.error.is_none(), "{:?}", result.error);
        assert!(
            result.sentinel_question.is_none(),
            "a NoNull channel is never offered for conversion"
        );
        assert!(
            !result.warning.clone().unwrap_or_default().contains("DEC-037"),
            "no sentinel note on a NoNull delivery: {:?}",
            result.warning
        );
        let well_id = result.well_id.clone().unwrap();
        let preserved: i64 = conn
            .query_row(
                "SELECT count(*) FROM standard_curves WHERE well_id = ?1 AND gr = -999.0",
                params![well_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(preserved, 1, "the declared-NoNull amplitude is untouched");
        std::fs::remove_file(&path).ok();
    }

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

        let blocked = import_core_csv(&conn, &well_id, path.to_str().unwrap(), "MD");
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
            "MD",
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

        // The source decimals agree exactly. At this depth their f32 reductions do not:
        // comparing the rounded values creates a false re-grid warning on a faithful LAS.
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let path = std::env::temp_dir().join("sandibumi-step-deep-decimal.las");
        std::fs::write(
            &path,
            "~VERSION\nVERS. 2.0 :\nWRAP. NO :\n~WELL\nSTEP.M 0.15240 : declared step\nNULL. -999.25 :\nWELL. STEP-DEEP :\n~CURVE\nDEPT.M : depth\nGR.GAPI : gamma\n~ASCII\n10000.00000 50\n10000.15240 51\n10000.30480 52\n",
        )
        .unwrap();
        let faithful =
            import_las_files(&conn, &[path.to_str().unwrap().to_string()], None).remove(0);
        std::fs::remove_file(&path).ok();
        assert!(
            faithful.error.is_none(),
            "the exact-decimal source imports: {:?}",
            faithful.error
        );
        assert!(
            !faithful
                .warning
                .as_deref()
                .unwrap_or("")
                .contains("possibly re-gridded"),
            "f32 reduction must not fabricate a STEP mismatch: {:?}",
            faithful.warning
        );

        // A missing index row breaks adjacency. The next finite depth is not its neighbour,
        // so the audit must restart there rather than compare across the null gap.
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let path = std::env::temp_dir().join("sandibumi-step-missing-depth.las");
        std::fs::write(
            &path,
            "~VERSION\nVERS. 2.0 :\nWRAP. NO :\n~WELL\nSTEP.M 0.5 : declared step\nNULL. -999.25 :\nWELL. STEP-GAP :\n~CURVE\nDEPT.M : depth\nGR.GAPI : gamma\n~ASCII\n1000.0 50\n-999.25 51\n1001.0 52\n1001.5 53\n",
        )
        .unwrap();
        let gap = import_las_files(&conn, &[path.to_str().unwrap().to_string()], None).remove(0);
        std::fs::remove_file(&path).ok();
        assert!(
            gap.error.is_none(),
            "the source with one missing index row still imports: {:?}",
            gap.error
        );
        assert!(
            !gap.warning
                .as_deref()
                .unwrap_or("")
                .contains("possibly re-gridded"),
            "STEP comparison must not bridge a missing index row: {:?}",
            gap.warning
        );
    }

    fn make_dio020_duplicate_las() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "sandibumi-three-repeated-depths-{}-{}.las",
            std::process::id(),
            Uuid::new_v4()
        ));
        std::fs::write(
            &path,
            "~VERSION\nVERS. 2.0 :\n~WELL\nWELL. REPEATED-DEPTHS :\n\
             ~CURVE\nDEPT.M :\nGR.GAPI :\nPEF.B/E :\n~ASCII\n\
             1000.0 10.0 1.0\n1000.0 20.0 2.0\n1000.0 30.0 3.0\n\
             1000.0 40.0 4.0\n1001.0 50.0 5.0\n",
        )
        .unwrap();
        path
    }

    /// CORRECTNESS - SB-DIO-020 / SB-DIO-T33. `21_data-io.md` section 6 T33
    /// requires the unresolved-policy path to ask and commit nothing. The positive
    /// policy path is pinned separately by T32, so an always-refusing reader cannot pass.
    #[test]
    fn duplicate_depths_commit_nothing_until_a_policy_is_declared() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let path = make_dio020_duplicate_las();
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
        std::fs::remove_file(&path).ok();
    }

    /// CORRECTNESS - SB-DIO-020 / SB-DIO-T32. `21_data-io.md` section 6 T32
    /// supplies the expected three-row count and keep-first outcome. GR and PEF are
    /// independent companion columns, so neither depth-only nor standard-only handling passes.
    #[test]
    fn keep_first_drops_three_repeated_depth_rows_reports_three_and_keeps_first_samples_in_lockstep() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let path = make_dio020_duplicate_las();
        let file = path.to_str().unwrap().to_string();
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
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM wells", [], |row| row.get::<_, i64>(0)).unwrap(),
            0,
            "the declared refuse policy also commits nothing"
        );

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
        let catalog = db::list_generic_curve_catalog(&conn, kept.well_id.as_deref().unwrap()).unwrap();
        let pef = catalog
            .iter()
            .find(|curve| curve.mnemonic == "PEF")
            .expect("the generic companion curve is committed");
        let pef_samples = db::get_curve_samples(&conn, &pef.curve_id).unwrap();
        assert_eq!(
            pef_samples
                .iter()
                .map(|sample| (sample.depth, sample.value))
                .collect::<Vec<_>>(),
            vec![(1000.0, 1.0), (1001.0, 5.0)],
            "the same keep-first rows drive the generic companion curve"
        );

        let make_columns = || CurveColumns {
            well_name: None,
            well_headers: Vec::new(),
            las_version: None,
            unread_sections: Vec::new(),
            section_policy: parsers::LAS_SECTION_POLICY_ID.to_string(),
            section_handling: Vec::new(),
            text_encoding: "test fixture".into(),
            depth_unit: Some("M".into()),
            declared_step: None,
            declared_step_mismatch_note: None,
            depth: vec![1000.0, 1000.0, 1000.0, 1000.0, 1001.0],
            gr: vec![10.0, 20.0, 30.0, 40.0, 50.0],
            res: vec![f32::NAN; 5],
            nphi: vec![f32::NAN; 5],
            rhob: vec![f32::NAN; 5],
            dt: vec![f32::NAN; 5],
            sp: vec![100.0, 200.0, 300.0, 400.0, 500.0],
            raw_curves: vec![parsers::RawLasCurve {
                mnemonic: "PEF".into(),
                unit: Some("B/E".into()),
                values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            }],
            alias_decisions: Vec::new(),
            null_resolutions: Vec::new(),
            index_resolution: None,
            unit_designations: Vec::new(),
            undeclared_sentinel_candidates: Vec::new(),
        };
        let mut last = make_columns();
        assert_eq!(
            parsers::resolve_curve_column_duplicates(&mut last, parsers::DuplicateDepthPolicy::KeepLast),
            3
        );
        assert_eq!(last.gr, vec![40.0, 50.0], "keep-last keeps the last repeated sample");
        assert_eq!(last.sp, vec![400.0, 500.0], "keep-last keeps standard companions aligned");
        assert_eq!(
            last.raw_curves[0].values,
            vec![4.0, 5.0],
            "keep-last keeps generic companions aligned"
        );
        let mut mean = make_columns();
        assert_eq!(
            parsers::resolve_curve_column_duplicates(&mut mean, parsers::DuplicateDepthPolicy::Mean),
            3
        );
        assert_eq!(mean.gr, vec![25.0, 50.0], "mean averages the four finite repeated samples");
        assert_eq!(mean.sp, vec![250.0, 500.0], "mean keeps standard companions aligned");
        assert_eq!(
            mean.raw_curves[0].values,
            vec![2.5, 5.0],
            "mean keeps generic companions aligned"
        );
    }

    fn make_dio015_las(name: &str, unit: &str, well: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        std::fs::write(
            &path,
            format!(
                "~VERSION\nVERS. 2.0 :\n~WELL\nWELL. {well} :\n~CURVE\nDEPT.{unit} : depth\nGR.GAPI :\n~ASCII\n1000 50\n1001 55\n"
            ),
        )
        .unwrap();
        path
    }

    fn import_dio015_las(
        conn: &Connection,
        path: &std::path::Path,
        options: &LasImportOptions,
    ) -> ImportResult {
        import_las_files_with(conn, &[path.to_str().unwrap().to_string()], None, options)
            .into_iter()
            .next()
            .expect("one input path produces one import result")
    }

    #[test]
    fn an_index_with_no_file_or_project_unit_refuses_names_both_sources_and_commits_nothing() {
        // CORRECTNESS — source: docs/PRD_v2/21_data-io.md §6 SB-DIO-T22.
        let fresh = Connection::open_in_memory().unwrap();
        db::create_schema(&fresh).unwrap();
        let no_unit = make_dio015_las("sandibumi_dio015_none.las", "", "UNIT_ABSENT");
        let fresh_result = import_dio015_las(&fresh, &no_unit, &LasImportOptions::default());
        assert!(
            fresh_result.error.as_deref().is_some_and(|e| e.contains("file index") && e.contains("project")),
            "the refusal must name both possible sources: {:?}",
            fresh_result.error
        );
        let wells: i64 = fresh.query_row("SELECT COUNT(*) FROM wells", [], |row| row.get(0)).unwrap();
        assert_eq!(wells, 0, "the unit refusal happens before any well is committed");
        std::fs::remove_file(no_unit).ok();
    }

    #[test]
    fn a_project_unit_never_becomes_an_undeclared_files_unit_without_per_import_confirmation() {
        // CORRECTNESS — source: docs/PRD_v2/21_data-io.md §6 SB-DIO-T23.
        // The 0.3048 m/ft conversion is NIST SP 811, cited by chapter §5.1.
        let metric = Connection::open_in_memory().unwrap();
        db::create_schema(&metric).unwrap();
        crate::units::set_project_depth_unit(&metric, crate::units::DepthUnit::Metres).unwrap();
        let no_unit = make_dio015_las("sandibumi_dio015_confirm.las", "", "UNIT_CONFIRMATION");
        let still_refused = import_dio015_las(&metric, &no_unit, &LasImportOptions::default());
        assert!(
            still_refused.error.as_deref().is_some_and(|e| e.contains("project setting is not a file declaration")),
            "a declared project must not silently lend its unit to the file: {:?}",
            still_refused.error
        );
        let before: i64 = metric.query_row("SELECT COUNT(*) FROM wells", [], |row| row.get(0)).unwrap();
        assert_eq!(before, 0, "the unresolved file unit commits no well");

        let confirmed = LasImportOptions {
            file_depth_unit: Some("FT".into()),
            ..Default::default()
        };
        let accepted = import_dio015_las(&metric, &no_unit, &confirmed);
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
        std::fs::remove_file(no_unit).ok();
    }

    #[test]
    fn characterizes_a_declared_feet_index_into_a_metre_project_as_converted_with_a_report() {
        // CHARACTERIZATION — SB-DIO-T24 labels the conversion report as char; the numeric
        // 0.3048 m/ft check remains independently sourced to NIST SP 811 in chapter §5.1.
        let metric = Connection::open_in_memory().unwrap();
        db::create_schema(&metric).unwrap();
        crate::units::set_project_depth_unit(&metric, crate::units::DepthUnit::Metres).unwrap();
        let declared = make_dio015_las("sandibumi_dio015_declared.las", "FT", "UNIT_DECLARED");
        let declared_result = import_dio015_las(&metric, &declared, &LasImportOptions::default());
        assert!(declared_result.error.is_none(), "a declared file needs no confirmation");
        assert!(
            declared_result.warning.as_deref().unwrap_or("").contains("converted from ft"),
            "the declared-unit conversion is still reported: {:?}",
            declared_result.warning
        );
        let first_depth: f32 = metric
            .query_row("SELECT MIN(depth) FROM standard_curves", [], |row| row.get(0))
            .unwrap();
        assert!((first_depth - 304.8).abs() < 1e-3, "declared feet convert by the cited 0.3048 factor");
        std::fs::remove_file(declared).ok();
    }

    /// SB-DIO-024 / SB-DIO-T39. The international-foot factor is 0.3048 m/ft
    /// (NIST SP 811, chapter §5.1). Reporting alone is not enough: the stored sample
    /// is checked too, so a no-op conversion with a plausible audit record cannot pass.
    #[test]
    fn every_converted_curve_reports_its_from_unit_to_unit_and_factor_and_uses_that_transform() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let las = "~Version\n\
                   VERS. 2.0 :\n\
                   ~Well\n\
                   WELL. DIO-024 :\n\
                   ~Curve\n\
                   DEPT .M    : depth\n\
                   DTCO .US/M : compressional sonic\n\
                   DTSM .US/M : shear sonic\n\
                   ~ASCII\n\
                   1000.0 100.0 150.0\n\
                   1000.5 200.0 250.0\n";
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
        assert_eq!(
            result.unit_conversions.len(),
            2,
            "reporting only the first converted curve is still a silent conversion"
        );
        let warning = result.warning.as_deref().unwrap_or("");
        for curve in ["DTCO", "DTSM"] {
            let conversion = result
                .unit_conversions
                .iter()
                .find(|conversion| conversion.curve == curve)
                .unwrap_or_else(|| panic!("{curve} conversion is reported"));
            assert_eq!(conversion.from_unit, "US/M");
            assert_eq!(conversion.to_unit, "us/ft");
            assert_eq!(conversion.factor, 0.3048_f32);
            assert_eq!(
                conversion.offset, 0.0,
                "a multiplicative conversion carries an explicit zero offset"
            );
            assert!(
                warning.contains(curve)
                    && warning.contains("US/M")
                    && warning.contains("us/ft")
                    && warning.contains("0.3048"),
                "the visible import note must carry the {curve} audit: {:?}",
                result.warning
            );
        }

        let well_id = result.well_id.unwrap();
        let catalog = db::list_generic_curve_catalog(&conn, &well_id).unwrap();
        for (curve, expected) in [("DTCO", 30.48_f32), ("DTSM", 45.72_f32)] {
            let held = catalog
                .iter()
                .find(|entry| entry.mnemonic == curve)
                .unwrap_or_else(|| panic!("{curve} generic curve"));
            let samples = db::get_curve_samples(&conn, &held.curve_id).unwrap();
            assert!(
                (samples[0].value - expected).abs() < 1e-4,
                "the {curve} stored sample must use the reported 0.3048 transform"
            );
        }
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
        let standard_rhob: Option<f32> = conn
            .query_row(
                "SELECT rhob FROM standard_curves WHERE well_id = ?1 ORDER BY depth LIMIT 1",
                params![&well_id],
                |row| row.get(0),
            )
            .unwrap();
        // SB-DBM-030: absence is SQL NULL at the store, not a float NaN.
        assert!(standard_rhob.is_none(), "PPG data must not populate the standard RHOB channel");
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
        let second_path = std::env::temp_dir().join(format!(
            "sandibumi-dio029-second-{}-{}.las",
            std::process::id(),
            Uuid::new_v4()
        ));
        std::fs::write(&second_path, las.replace("WELL. DIO-029", "WELL. DIO-029-SECOND"))
            .unwrap();
        let second_file = second_path.to_string_lossy().to_string();

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

        let designated = Connection::open_in_memory().unwrap();
        db::create_schema(&designated).unwrap();
        let designated_results = import_las_files_with(
            &designated,
            &[file.clone(), second_file.clone()],
            None,
            &LasImportOptions {
                ms_per_ft_meanings: std::collections::HashMap::from([
                    (
                        file.clone(),
                        crate::curves::MsPerFtMeaning::MicrosecondsPerFoot,
                    ),
                    (
                        second_file.clone(),
                        crate::curves::MsPerFtMeaning::MillisiemensPerFoot,
                    ),
                ]),
                ..Default::default()
            },
        );
        let sonic_result = designated_results
            .iter()
            .find(|result| result.path == file)
            .expect("the sonic file has its own import result");
        let conductivity_result = designated_results
            .iter()
            .find(|result| result.path == second_file)
            .expect("the conductivity file has its own import result");
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
        let sonic_curve = db::list_generic_curve_catalog(
            &designated,
            sonic_result.well_id.as_deref().unwrap(),
        )
            .unwrap()
            .into_iter()
            .find(|curve| curve.mnemonic == "DTCO")
            .unwrap();
        assert_eq!(sonic_curve.family.as_deref(), Some("DT"));
        assert_eq!(sonic_curve.unit.as_deref(), Some("us/ft"));
        assert_eq!(
            db::get_curve_samples(&designated, &sonic_curve.curve_id).unwrap()[0].value,
            100.0
        );

        std::fs::remove_file(&path).ok();
        std::fs::remove_file(&second_path).ok();
        assert!(
            conductivity_result.error.is_none(),
            "conductivity designation failed: {:?}",
            conductivity_result.error
        );
        let conductivity_answer = conductivity_result.unit_designations.first().expect("conductivity answer recorded");
        assert_eq!(conductivity_answer.meaning, "millisiemens_per_foot");
        assert_eq!(conductivity_answer.recorded_unit, "MS/FT");
        assert_eq!(conductivity_answer.family, None);
        assert!(
            conductivity_result.warning.as_deref().is_some_and(|note| {
                note.contains("DTCO")
                    && note.contains("MS/FT")
                    && note.contains("millisiemens_per_foot")
            }),
            "the second file's different answer must be visible: {:?}",
            conductivity_result.warning
        );
        let conductivity_well = conductivity_result.well_id.as_deref().unwrap();
        let standard_dt: Option<f32> = designated
            .query_row(
                "SELECT dt FROM standard_curves WHERE well_id = ?1 ORDER BY depth LIMIT 1",
                params![conductivity_well],
                |row| row.get(0),
            )
            .unwrap();
        // SB-DBM-030: absence is SQL NULL at the store, not a float NaN.
        assert!(standard_dt.is_none(), "a conductivity designation must not populate standard DT");
        let conductivity_curve = db::list_generic_curve_catalog(&designated, conductivity_well)
            .unwrap()
            .into_iter()
            .find(|curve| curve.mnemonic == "DTCO")
            .unwrap();
        assert_eq!(conductivity_curve.family, None);
        assert_eq!(conductivity_curve.unit.as_deref(), Some("MS/FT"));
        assert_eq!(
            db::get_curve_samples(&designated, &conductivity_curve.curve_id).unwrap()[0].value,
            100.0
        );
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
                   ~Well\n\
                   WELL. GENERIC-CURVE-CONTROL :\n\
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

        // Deviation survey → TVD/TVDSS. Declared AFTER the LAS import above, which is what
        // decides the project's unit in the ordinary flow; the survey importer now refuses an
        // undeclared project exactly as the core-table importer does.
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let dev = std::env::temp_dir().join(format!("arshilla_dev_test_{ids}.csv"));
        std::fs::write(&dev, "MD,INC,AZI\n0,0,0\n1000,0,0\n2000,60,45\n").unwrap();
        let res = import_deviation_csv(&conn, &ids, dev.to_str().unwrap(), Some(25.0), None, None);
        std::fs::remove_file(&dev).ok();
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.rows, 3);
        let path = db::get_well_path(&conn, &ids).unwrap();
        assert_eq!(path.len(), 3);
        assert!((path[1].tvd - 1000.0).abs() < 1e-2, "vertical section TVD == MD");
        assert!(path[2].tvd < path[2].md, "deviated station TVD shallower than MD");
        assert!((path[1].tvdss - (1000.0 - 25.0)).abs() < 1e-2, "TVDSS = TVD - elevation");
    }

    #[test]
    fn deviation_import_materializes_tvd_tvdss_curves() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "DEV-MAT-1", None, None, None).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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
        let res = import_deviation_csv(&conn, &ids, dev.to_str().unwrap(), Some(25.0), None, None);
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
        // F-17 / SB-DBM-031: TVDSS = TVD - elevation(25) everywhere.
        for (t, ss) in tvd.iter().zip(tvdss.iter()) {
            assert!((ss - (t - 25.0)).abs() < 1e-1, "TVDSS = TVD - 25: {ss} vs {}", t - 25.0);
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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

        assert!(import_deviation_csv(&conn, &ids, &prelim, Some(25.0), Some("PRELIM"), None).error.is_none());
        assert!(
            import_deviation_csv(&conn, &ids, &defin, Some(25.0), Some("DEFINITIVE"), None)
                .error
                .is_none()
        );

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

    /// Audit finding 8. The survey importer was the last depth-bearing importer reading its
    /// file raw: a foot survey delivered into a metre project stored every station 3.28084x
    /// too deep, and TVD/TVDSS carry that onto the log grid, into `sw_height` and into the
    /// saturation-height fits — all of it plausible-looking.
    ///
    /// The two halves are pinned together because either alone would pass a lazier fix: convert
    /// everything and the datum is silently scaled too; convert nothing and the declaration is
    /// decorative.
    #[test]
    fn a_survey_declared_in_feet_lands_on_the_projects_own_depth_scale() {
        let mk = |name: &str| {
            let conn = Connection::open_in_memory().unwrap();
            crate::db::create_schema(&conn).unwrap();
            let wid = uuid::Uuid::new_v4();
            crate::db::insert_well(&conn, wid, name, None, None, None).unwrap();
            crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
            (conn, wid.to_string())
        };
        // Vertical, so TVD == MD and the conversion is readable straight off the stations
        // without re-deriving minimum curvature here.
        let body = "MD,INC,AZI\n0,0,0\n4000,0,0\n8000,0,0\n";
        let write = |tag: &str| -> String {
            let p = std::env::temp_dir().join(format!("sandibumi_devunit_{tag}_{}.csv", uuid::Uuid::new_v4()));
            std::fs::write(&p, body).unwrap();
            p.to_string_lossy().into_owned()
        };

        // --- Declared FT into a metre project: the file's numbers are converted. ---
        let (conn, ids) = mk("SANDI-DEV-FT");
        let path = write("ft");
        let res = import_deviation_csv(&conn, &ids, &path, Some(25.0), Some("DEFINITIVE"), Some("FT"));
        std::fs::remove_file(&path).ok();
        assert!(res.error.is_none(), "{:?}", res.error);
        let stations = db::get_well_path(&conn, &ids).unwrap();
        let deepest = stations.last().unwrap();
        // 8000 ft = 2438.4 m exactly (the foot is defined as 0.3048 m).
        assert!(
            (deepest.md - 2438.4).abs() < 1e-2,
            "8000 ft of hole is 2438.4 m of hole, got {} — the file was read raw",
            deepest.md
        );
        assert!((deepest.tvd - 2438.4).abs() < 1e-2, "vertical survey: TVD == MD, got {}", deepest.tvd);
        // The datum is TYPED in the dialog, which labels it in the project's unit, so it is
        // already metres and must NOT ride the file's conversion. 25 ft would be 7.62 m.
        assert!(
            (deepest.tvdss - (deepest.tvd - 25.0)).abs() < 1e-2,
            "TVDSS = TVD - 25 m; the typed datum is not the file's unit, got {}",
            deepest.tvdss
        );

        // --- Undeclared: unchanged from every import before this one. ---
        let (conn, ids) = mk("SANDI-DEV-ASIS");
        let path = write("asis");
        let res = import_deviation_csv(&conn, &ids, &path, Some(25.0), Some("DEFINITIVE"), None);
        std::fs::remove_file(&path).ok();
        assert!(res.error.is_none(), "{:?}", res.error);
        let stations = db::get_well_path(&conn, &ids).unwrap();
        let deepest = stations.last().unwrap();
        assert!(
            (deepest.md - 8000.0).abs() < 1e-2,
            "no declaration means the file is already the project unit, got {}",
            deepest.md
        );

        // --- Undeclared PROJECT: refused, not guessed — the core importer's own rule. ---
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-DEV-NOUNIT", None, None, None).unwrap();
        let path = write("nounit");
        let res = import_deviation_csv(&conn, &wid.to_string(), &path, Some(25.0), None, Some("FT"));
        std::fs::remove_file(&path).ok();
        let error = res.error.expect("an undeclared project cannot place a foot survey");
        assert!(error.contains("depth unit"), "the refusal names what is missing: {error}");
        assert_eq!(res.rows, 0, "nothing is stored on a refusal");
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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

        // Import a deviated survey (would compute a very DIFFERENT TVDSS = TVD - 25).
        let dev = std::env::temp_dir().join(format!("arshilla_devmat3_{ids}.csv"));
        std::fs::write(&dev, "MD,INC,AZI\n0,0,0\n1000,0,0\n2000,60,45\n3000,60,45\n").unwrap();
        let res = import_deviation_csv(&conn, &ids, dev.to_str().unwrap(), Some(25.0), None, None);
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
                   ~Well\nWELL. DUPLICATE-DEPTH-CONTROL :\n\
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

    /// SB-CORE-002 / SB-CORE-T04. CORRECTNESS: `04_CORE_REQUIREMENTS.md` assigns R4's
    /// import reporting surface to the atomic all-channel contract recorded in
    /// `docs/record_data_tools.md`. The clean control and failed delivery pin both sides:
    /// neither an always-failing importer nor a partial-success fallback can satisfy it.
    #[test]
    fn an_all_channel_import_failure_returns_a_named_error_and_commits_no_partial_well() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let write_las = |well: &str| {
            let path = std::env::temp_dir().join(format!(
                "sandibumi-core002-{}-{}.las",
                std::process::id(),
                Uuid::new_v4()
            ));
            let las = format!(
                "~Version\nVERS. 2.0 :\n~Well\nWELL. {well} :\nNULL. -999.25 :\n\
                 ~Curve\nDEPT.M : depth\nGR.GAPI : gamma\nILD.OHMM : resistivity\n\
                 NPHI.V/V : neutron\nRHOB.G/CC : density\nDT.US/FT : sonic\nSP.MV : spontaneous potential\n\
                 PEF.B/E : photoelectric\n~ASCII\n\
                 1000.0 45.0 20.0 0.18 2.35 80.0 -10.0 3.0\n\
                 1000.5 47.0 22.0 0.19 2.34 79.0 -11.0 3.1\n"
            );
            std::fs::write(&path, las).unwrap();
            path
        };

        let clean_path = write_las("FULL_CURVE_CONTROL");
        let clean = import_las_files_with(
            &conn,
            &[clean_path.to_string_lossy().into_owned()],
            None,
            &LasImportOptions::default(),
        )
        .remove(0);
        std::fs::remove_file(&clean_path).ok();
        assert!(clean.error.is_none(), "clean control import failed: {:?}", clean.error);
        assert!(
            !clean.warning.as_deref().unwrap_or("").contains("only the six standard curves were loaded"),
            "a complete import must not be labelled partial: {:?}",
            clean.warning
        );

        let wells_before: i64 = conn.query_row("SELECT COUNT(*) FROM wells", [], |row| row.get(0)).unwrap();
        let standard_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM standard_curves", [], |row| row.get(0))
            .unwrap();
        let metadata_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM curve_meta", [], |row| row.get(0))
            .unwrap();

        conn.execute_batch("DROP TABLE curve_samples").unwrap();
        let failed_path = write_las("FULL_CURVE_DEPENDENCY_MISSING");
        let failed = import_las_files_with(
            &conn,
            &[failed_path.to_string_lossy().into_owned()],
            None,
            &LasImportOptions::default(),
        )
        .remove(0);
        std::fs::remove_file(&failed_path).ok();

        let error = failed.error.as_deref().expect("the failed delivery must be reported as an error");
        assert!(error.contains("curve_samples"), "the underlying cause must remain actionable: {error}");
        assert!(failed.warning.is_none(), "a failed delivery must not be downgraded to a warning: {:?}", failed.warning);
        assert!(failed.well_id.is_none(), "a rolled-back delivery must not return a committed well id");
        assert_eq!(failed.rows, 0, "a rolled-back delivery must report no committed rows");
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM wells", [], |row| row.get::<_, i64>(0)).unwrap(),
            wells_before,
            "the failed delivery must not leave a partial well"
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM standard_curves", [], |row| row.get::<_, i64>(0)).unwrap(),
            standard_before,
            "the failed delivery must not leave a standard projection"
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM curve_meta", [], |row| row.get::<_, i64>(0)).unwrap(),
            metadata_before,
            "the failed delivery must not leave generic metadata"
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM wells WHERE well_name = 'FULL_CURVE_DEPENDENCY_MISSING'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            0,
            "the failed delivery name must not appear as a committed well"
        );
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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
        let res = import_core_table(&conn, spath, &mapping, Some("ft"), None, Some("core"), None, false, "MD");
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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
            "MD",
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

    /// SB-DIO-048 / SB-DIO-T67. CORRECTNESS - `docs/PRD_v2/21_data-io.md`
    /// D-27 and sections 4.9/6.9 make the LAS `~W WELL` value authoritative. A file
    /// stem is only a proposal that needs an explicit user confirmation when that source
    /// value is absent; it is never an identity merely because the import has a filename.
    #[test]
    fn a_las_header_well_identity_overrides_the_filename_and_an_absent_header_only_offers_the_filename_until_confirmed() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();

        let write_fixture = |label: &str, well_line: &str| {
            let path = std::env::temp_dir().join(format!(
                "{label}-{}.las",
                Uuid::new_v4()
            ));
            std::fs::write(
                &path,
                format!(
                    "~VERSION\nVERS. 2.0 :\n~WELL\n{well_line}NULL. -999.25 :\n~CURVE\nDEPT.M : depth\nGR.GAPI : gamma\n~ASCII\n1000 50\n"
                ),
            )
            .unwrap();
            path
        };

        let header_path = write_fixture(
            "FILENAME-IDENTITY-MUST-NOT-WIN",
            "WELL. CONTAINER-IDENTITY\n",
        );
        let header_probe = parsers::probe_las_well_identity(&header_path).unwrap();
        assert_eq!(
            header_probe.container_well_name.as_deref(),
            Some("CONTAINER-IDENTITY")
        );
        assert_eq!(
            header_probe.filename_proposal, None,
            "a present container identity must suppress rather than compete with a filename proposal"
        );
        let mut header_options = LasImportOptions::default();
        header_options.confirmed_well_names.insert(
            header_path.to_string_lossy().into_owned(),
            "FILENAME-IDENTITY-MUST-NOT-WIN".to_string(),
        );
        let header_result = import_las_files_with(
            &conn,
            &[header_path.to_string_lossy().into_owned()],
            None,
            &header_options,
        )
        .remove(0);
        assert!(header_result.error.is_none(), "{:?}", header_result.error);
        assert_eq!(header_result.well_name.as_deref(), Some("CONTAINER-IDENTITY"));

        let missing_path = write_fixture("FILENAME-PROPOSAL-NEEDS-CONFIRMATION", "");
        let missing_probe = parsers::probe_las_well_identity(&missing_path).unwrap();
        assert_eq!(missing_probe.container_well_name, None);
        assert_eq!(
            missing_probe.filename_proposal,
            missing_path
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned()),
            "the filename is exposed only in the proposal field"
        );
        let missing_result = import_las_files_with(
            &conn,
            &[missing_path.to_string_lossy().into_owned()],
            None,
            &LasImportOptions::default(),
        )
        .remove(0);
        assert!(
            missing_result.error.as_deref().is_some_and(|error| {
                error.contains("source well identity is absent")
                    && error.contains("explicit confirmation")
            }),
            "a filename must be offered, not silently committed: {:?}",
            missing_result.error
        );
        let wells: i64 = conn
            .query_row("SELECT COUNT(*) FROM wells", [], |row| row.get(0))
            .unwrap();
        assert_eq!(wells, 1, "the unconfirmed filename proposal must write no well");

        let mut confirmed_options = LasImportOptions::default();
        confirmed_options.confirmed_well_names.insert(
            missing_path.to_string_lossy().into_owned(),
            "CONFIRMED-IDENTITY".to_string(),
        );
        let confirmed_result = import_las_files_with(
            &conn,
            &[missing_path.to_string_lossy().into_owned()],
            None,
            &confirmed_options,
        )
        .remove(0);
        assert!(confirmed_result.error.is_none(), "{:?}", confirmed_result.error);
        assert_eq!(confirmed_result.well_name.as_deref(), Some("CONFIRMED-IDENTITY"));
        let wells: i64 = conn
            .query_row("SELECT COUNT(*) FROM wells", [], |row| row.get(0))
            .unwrap();
        assert_eq!(wells, 2, "only the explicitly confirmed proposal may create the second well");

        std::fs::remove_file(header_path).ok();
        std::fs::remove_file(missing_path).ok();
    }

    /// SB-DIO-053 / SB-DIO-T76. CORRECTNESS.
    /// Source: `docs/PRD_v2/21_data-io.md` sections 4.9 and 6.9 require a missing UWI,
    /// field, operator and country to remain absent rather than being derived from a filename
    /// or another identity value.
    #[test]
    fn a_file_without_a_uwi_does_not_synthesize_one_from_the_filename_or_any_other_identity() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();

        let path = std::env::temp_dir().join(format!(
            "INVENTED-UWI-FIELD-OPERATOR-COUNTRY-CONTROL-{}.las",
            Uuid::new_v4()
        ));
        std::fs::write(
            &path,
            "~VERSION\nVERS. 2.0 :\n~WELL\nWELL. SOURCE-OWNED-IDENTITY : source identity\nNULL. -999.25 : null\n~CURVE\nDEPT.M : depth\nGR.GAPI : gamma\n~ASCII\n1000 50\n",
        )
        .unwrap();

        let result = import_las_files_with(
            &conn,
            &[path.to_string_lossy().into_owned()],
            None,
            &LasImportOptions::default(),
        )
        .remove(0);
        std::fs::remove_file(&path).ok();

        assert!(result.error.is_none(), "the source-owned WELL identity is sufficient: {:?}", result.error);
        let identities: Vec<parsers::LasWellHeaderField> = result
            .well_headers
            .iter()
            .filter_map(|header| header.mapped_field)
            .filter(|field| {
                matches!(
                    field,
                    parsers::LasWellHeaderField::WellName
                        | parsers::LasWellHeaderField::Uwi
                        | parsers::LasWellHeaderField::Country
                )
            })
            .collect();
        assert_eq!(
            identities,
            [parsers::LasWellHeaderField::WellName],
            "only the identity present in the source may exist after import"
        );
        assert_eq!(
            result
                .well_headers
                .iter()
                .map(|header| header.mnemonic.as_str())
                .collect::<Vec<_>>(),
            ["WELL", "NULL"],
            "no absent UWI, field, operator, country or other header may be added"
        );
        assert!(
            result
                .well_headers
                .iter()
                .all(|header| !header.raw.contains("INVENTED-UWI-FIELD-OPERATOR-COUNTRY-CONTROL")),
            "the filename must not be copied into any carried header"
        );
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();

        let csv = "WELL,TOP,BASE,LITHOLOGY,QUARTZ\n\
                   W-A,1000.0,1002.0,Sandstone,72.1\n\
                   W-B,2000.0,2001.5,Claystone,38.0\n\
                   NOPE-1,3000.0,3001.0,Limestone,5.0\n\
                   ,4000.0,4001.0,Coal,1.0\n";
        let path = std::env::temp_dir().join("sandibumi_aux_v2_test.csv");
        std::fs::write(&path, csv).unwrap();

        let res = import_aux_file(&conn, &wa.to_string(), "PETROGRAPHY", path.to_str().unwrap(), None, false, "MD", None);
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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

        let res = import_scal_csv(&conn, &ids, path.to_str().unwrap(), 72.0, "MD");
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.rows, 12);
        let fit = res.fit.expect("fit should solve");
        assert!((fit.b - -0.5).abs() < 0.05, "b={}", fit.b);
        assert!((fit.a - 0.4).abs() < 0.1, "a={}", fit.a);

        // Re-import replaces rather than duplicates; rows readable back.
        let res2 = import_scal_csv(&conn, &ids, path.to_str().unwrap(), 72.0, "MD");
        std::fs::remove_file(&path).ok();
        assert_eq!(res2.rows, 12);
        let rows = db::get_scal_pc(&conn, &ids).unwrap();
        assert_eq!(rows.len(), 12);
        assert!((rows[0].poro - 0.22).abs() < 1e-4, "percent poro converted to v/v");
        assert!(rows.iter().all(|r| r.sw <= 1.0), "percent Sw converted to v/v");

        // Unknown well errors cleanly.
        let bad = import_scal_csv(&conn, "nope", "x.csv", 72.0, "MD");
        assert!(bad.error.is_some());
    }

    /// Multi-file SCAL import (increment 2): two single-plug centrifuge exports sniffed
    /// by "auto" land in one combined replace-write; a later porous-plate import REPLACES
    /// them (not appends); a bad file fails the whole import with the filename named.
    #[test]
    fn scal_import_files_multi_format_and_replace() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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

        let res = import_scal_files(&conn, &ids, &paths, "auto", "air_brine", 72.0, None, false, "MD", None);
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
            import_scal_files(&conn, &ids, &[p3.to_str().unwrap().to_string()], "porous_plate", "air_brine", 72.0, None, false, "MD", None);
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
            "MD",
            None,
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "SCAL-3", None, None, None).unwrap();
        let ids = well_id.to_string();

        let good = std::env::temp_dir().join(format!("sandibumi_scal_good_{ids}.csv"));
        std::fs::write(&good, "PC,SW\n5,0.55\n10,0.45\n20,0.35\n").unwrap();
        let res = import_scal_files(&conn, &ids, &[good.to_str().unwrap().to_string()], "long", "hg_air", 367.0, None, false, "MD", None);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(db::get_scal_pc(&conn, &ids).unwrap().len(), 3);

        // Header-only export (e.g. a filtered/template sheet) → error, data intact.
        let empty = std::env::temp_dir().join(format!("sandibumi_scal_empty_{ids}.csv"));
        std::fs::write(&empty, "PC,SW\n").unwrap();
        let res2 = import_scal_files(&conn, &ids, &[empty.to_str().unwrap().to_string()], "auto", "hg_air", 367.0, None, false, "MD", None);
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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
        let res = import_tops_file(&conn, None, path.to_str().unwrap(), None);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.tops_written, 3);
        assert_eq!(res.wells_matched, 2, "case-insensitive well matching");
        assert_eq!(res.unmatched_wells, vec!["GHOST-9".to_string()]);

        // Give TOP_A a color, then re-import a new depth: depth moves, color survives.
        db::upsert_top(&conn, &id1, "TOP_A", 1000.0, Some("#ff0000")).unwrap();
        std::fs::write(&path, "WELL,TOP,MD\nSANDI-1,TOP_A,1005.0\n").unwrap();
        let res2 = import_tops_file(&conn, None, path.to_str().unwrap(), None);
        assert!(res2.error.is_none());
        let tops = db::list_tops(&conn, &id1).unwrap();
        let a = tops.iter().find(|t| t.top_name == "TOP_A").unwrap();
        assert!((a.depth - 1005.0).abs() < 1e-3, "re-import updates depth");
        assert_eq!(a.color.as_deref(), Some("#ff0000"), "existing color kept");

        // No WELL column: needs a default well; with one it lands there.
        std::fs::write(&path, "TOP,DEPTH\nTOP_C,1200.0\n").unwrap();
        let need = import_tops_file(&conn, None, path.to_str().unwrap(), None);
        assert!(need.error.is_some(), "no well column and no selection must error");
        let ok = import_tops_file(&conn, Some(&id1), path.to_str().unwrap(), None);
        assert!(ok.error.is_none());
        assert!(db::list_tops(&conn, &id1).unwrap().iter().any(|t| t.top_name == "TOP_C"));
        std::fs::remove_file(&path).ok();
    }

    /// Audit finding 8, second site. A tops file in feet read into a metre project put every
    /// marker 3.28084x too deep — and a top is not one number, it is the boundary of a zone, so
    /// every zone parameter, every pay summary and every report drawn from them inherits it.
    ///
    /// Precedence is pinned in both directions because either half alone would pass a lazier
    /// implementation: believe the file and the caller's declaration is decorative; believe only
    /// the caller and a file that plainly says FEET is ignored.
    #[test]
    fn a_tops_file_that_says_feet_lands_on_the_projects_own_depth_scale() {
        // 5000 ft = 1524.0 m exactly (the foot is defined as 0.3048 m).
        let run = |body: &str, unit: Option<&str>| -> (TopsImportResult, f32) {
            let conn = Connection::open_in_memory().unwrap();
            db::create_schema(&conn).unwrap();
            crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
            let w = Uuid::new_v4();
            db::insert_well(&conn, w, "SANDI-TOPS-UNIT", None, None, None).unwrap();
            let ids = w.to_string();
            let path = std::env::temp_dir().join(format!("sandibumi_topsunit_{ids}.csv"));
            std::fs::write(&path, body).unwrap();
            let res = import_tops_file(&conn, Some(&ids), path.to_str().unwrap(), unit);
            std::fs::remove_file(&path).ok();
            let depth = db::list_tops(&conn, &ids)
                .unwrap()
                .iter()
                .find(|t| t.top_name == "TOP_A")
                .map(|t| t.depth)
                .unwrap_or(f32::NAN);
            (res, depth)
        };

        // The delivery convention: a units row under the header, one cell per column. The unit
        // is read off the DEPTH column's own cell — a "FEET" sitting under some other column
        // says nothing about this one, and is deliberately not accepted as if it did.
        let (res, depth) = run("TOP,MD\n,FEET\nTOP_A,5000.0\n", None);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.tops_written, 1, "the units row is not counted as a marker");
        assert!(
            (depth - 1524.0).abs() < 1e-2,
            "a marker at 5000 ft sits at 1524 m, got {depth} — the file was read raw"
        );
        assert_eq!(res.depth_unit.as_deref(), Some("ft"), "the import says what it read");

        // The other convention: the unit in the depth header itself.
        let (_res, depth) = run("TOP,TOP_MD_FT\nTOP_A,5000.0\n", None);
        assert!((depth - 1524.0).abs() < 1e-2, "a FT depth header is a declaration too, got {depth}");

        // Says nothing: unchanged from every tops import before this one.
        let (res, depth) = run("TOP,MD\nTOP_A,5000.0\n", None);
        assert!((depth - 5000.0).abs() < 1e-2, "no declaration means the project's own unit, got {depth}");
        assert_eq!(res.depth_unit.as_deref(), Some("m"), "and it says so");

        // The caller's declaration OUTRANKS the file's — a mislabelled header is exactly why an
        // override has to exist, and it is worth nothing if the file can overrule it.
        let (_res, depth) = run("TOP,TOP_MD_FT\nTOP_A,5000.0\n", Some("m"));
        assert!(
            (depth - 5000.0).abs() < 1e-2,
            "the caller said metres over a FT header and metres must win, got {depth}"
        );
    }

    #[test]
    fn a_tvd_only_tops_table_commits_the_alias_and_records_the_tvd_reference() {
        // CORRECTNESS — source: docs/PRD_v2/21_data-io.md §6 SB-DIO-T20.
        // Removing TVD from the accepted aliases, or storing it without its reference,
        // must fail this production-import test.
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let well = Uuid::new_v4();
        db::insert_well(&conn, well, "TVD_ONLY", None, None, None).unwrap();
        let path = std::env::temp_dir().join(format!("sandibumi_tvd_only_tops_{well}.csv"));
        std::fs::write(&path, "TOP,TVD\nREFERENCE_MARKER,900.0\n").unwrap();

        let result = import_tops_file(&conn, Some(&well.to_string()), path.to_str().unwrap(), None);
        std::fs::remove_file(&path).ok();

        assert!(result.error.is_none(), "TVD remains an accepted tops alias: {:?}", result.error);
        assert_eq!(result.tops_written, 1);
        let (depth, datum): (f32, String) = conn
            .query_row(
                "SELECT depth, depth_datum FROM tops WHERE well_id = ?1 AND top_name = 'REFERENCE_MARKER'",
                params![well.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("the committed top carries its source depth reference");
        assert_eq!(depth, 900.0, "the delivered TVD value is not rewritten at import");
        assert_eq!(datum, "TVD", "the TVD alias is never relabelled as MD");

        let edit_refusal = db::upsert_top(
            &conn,
            &well.to_string(),
            "REFERENCE_MARKER",
            901.0,
            None,
        )
        .expect_err("an MD-only editor cannot silently replace TVD custody")
        .to_string();
        assert!(
            edit_refusal.contains("TVD") && edit_refusal.contains("MD"),
            "the refused source-reference replacement names both frames: {edit_refusal}"
        );
        let delete_refusal = db::delete_top(&conn, &well.to_string(), "REFERENCE_MARKER")
            .expect_err("an MD-only editor cannot delete TVD custody before recreating it as MD")
            .to_string();
        assert!(
            delete_refusal.contains("TVD") && delete_refusal.contains("MD"),
            "the refused source-reference deletion names both frames: {delete_refusal}"
        );
        let unchanged: (f32, String) = conn
            .query_row(
                "SELECT depth, depth_datum FROM tops WHERE well_id = ?1 AND top_name = 'REFERENCE_MARKER'",
                params![well.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(unchanged, (900.0, "TVD".into()), "the refused edit changes nothing");
    }

    #[test]
    fn a_tvd_top_refuses_md_zones_without_a_deviation_survey_and_uses_the_surveyed_md_with_one() {
        // CORRECTNESS — source: docs/PRD_v2/21_data-io.md §6 SB-DIO-T21.
        // The literal survey stations independently pin TVD 900.0 to MD 1000.0; merely
        // checking that a survey exists while retaining 900.0 on the MD axis must fail.
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let well = Uuid::new_v4();
        let well_id = well.to_string();
        db::insert_well(&conn, well, "REFERENCE_FRAME", None, None, None).unwrap();
        db::insert_standard_curves(
            &conn,
            well,
            vec![1000.0, 1100.0],
            vec![50.0, 51.0],
            vec![f32::NAN; 2],
            vec![f32::NAN; 2],
            vec![f32::NAN; 2],
            vec![f32::NAN; 2],
            vec![f32::NAN; 2],
        )
        .unwrap();
        let path = std::env::temp_dir().join(format!("sandibumi_tvd_join_guard_{well}.csv"));
        std::fs::write(&path, "TOP,TVD\nREFERENCE_MARKER,900.0\n").unwrap();
        let imported = import_tops_file(&conn, Some(&well_id), path.to_str().unwrap(), None);
        std::fs::remove_file(&path).ok();
        assert!(imported.error.is_none(), "fixture import failed: {:?}", imported.error);

        let refusal = db::zones_from_tops(&conn, &well_id)
            .expect_err("a TVD top cannot become an MD zone without a reference frame")
            .to_string();
        assert!(refusal.contains("TVD") && refusal.contains("MD"), "both references are named: {refusal}");
        assert!(refusal.contains("deviation survey"), "the missing reference source is named: {refusal}");
        let before: i64 = conn
            .query_row("SELECT COUNT(*) FROM zones WHERE well_id = ?1", params![&well_id], |row| row.get(0))
            .unwrap();
        assert_eq!(before, 0, "the refused TVD-to-MD join writes no derived zones");

        db::insert_well_path(
            &conn,
            &well_id,
            "REFERENCE_SURVEY",
            Some("SB-DIO-T21 fixture"),
            Some(0.0),
            &[
                crate::deviation::Station { md: 0.0, inc: 0.0, azi: 0.0, tvd: 0.0, tvdss: 0.0 },
                crate::deviation::Station {
                    md: 1000.0,
                    inc: 0.0,
                    azi: 0.0,
                    tvd: 900.0,
                    tvdss: 900.0,
                },
                crate::deviation::Station {
                    md: 1100.0,
                    inc: 0.0,
                    azi: 0.0,
                    tvd: 990.0,
                    tvdss: 990.0,
                },
            ],
        )
        .unwrap();

        let zones = db::zones_from_tops(&conn, &well_id).expect("the active survey supplies the TVD-to-MD frame");
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].top_depth, 1000.0, "TVD 900 maps to the survey station at MD 1000");
        assert_eq!(zones[0].depth_datum, crate::schema_vocab::DepthDatum::Md);
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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
        let res = import_tops_file(&conn, Some(&id1), path.to_str().unwrap(), None);
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let ids = w.to_string();

        let xrd = std::env::temp_dir().join("arshilla_aux_xrd.csv");
        std::fs::write(&xrd, "Depth,Quartz,Illite,Remarks\n2000.0,45.2,12.1,clean\n2001.0,40.0,,silty\n").unwrap();
        let res = import_aux_file(&conn, &ids, "xrd", xrd.to_str().unwrap(), None, false, "MD", None);
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
        let res2 = import_aux_file(&conn, &ids, "PERFORATION", perf.to_str().unwrap(), None, false, "MD", None);
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
        let res3 = import_aux_file(&conn, &ids, "XRD", xrd.to_str().unwrap(), None, false, "MD", None);
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
        let bad = import_aux_file(&conn, "nope", "XRD", "x.csv", None, false, "MD", None);
        assert!(bad.error.is_some());
    }

    /// SCAL plugs ARE core plugs, so their depths are the core report's depths and must be able to
    /// follow the same correction.
    #[test]
    fn scal_points_can_follow_the_core_they_were_cut_from() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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

        let res = import_scal_files(&conn, &w, &[p.clone()], "long", "air_brine", 72.0, Some("FOLLOWED"), true, "MD", None);
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
        let plain = import_scal_files(&conn, &w, &[p], "long", "air_brine", 72.0, Some("ASWRITTEN"), false, "MD", None);
        assert!(plain.error.is_none(), "{:?}", plain.error);
        assert!(plain.note.is_none(), "nothing to report when the box was not ticked");
        let rows = db::get_scal_pc(&conn, &w).unwrap();
        assert!(
            rows.iter().any(|r| r.depth.is_some_and(|d| (d - 2005.0).abs() < 1e-3)),
            "unmapped import keeps the delivered depth"
        );
        std::fs::remove_file(&path).ok();
    }

    /// Audit finding 8, third site. A Pc delivery is read AT a depth — Thomeer and the J-fit QC
    /// pair every plug with the log's porosity and permeability there, and `sw_height` carries the
    /// fitted A/B back onto that same interval — so a delivery quoting feet into a metre project
    /// files each plug 3.28084x too deep and every pairing is with the wrong rock.
    ///
    /// The ORDER is the half that is easy to get wrong and impossible to see afterwards. The core
    /// depth record is already on the project's scale, so the file's depths must reach the project
    /// unit BEFORE they are mapped through it. Converting after would apply a metre correction to
    /// a foot number and then scale the sum — two errors compounding into one plausible depth.
    #[test]
    fn a_scal_delivery_in_feet_is_converted_before_it_follows_the_core() {
        // 6600 ft = 2011.68 m exactly (the foot is defined as 0.3048 m).
        let run = |unit: Option<&str>, follow: bool| -> Vec<f32> {
            let mut conn = Connection::open_in_memory().unwrap();
            db::create_schema(&conn).unwrap();
            crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
            let wid = uuid::Uuid::new_v4();
            db::insert_well(&conn, wid, "SANDI-SCAL-UNIT", None, None, None).unwrap();
            let w = wid.to_string();

            // A cored interval on the PROJECT's scale, shifted 2 m deeper against the log.
            let d: Vec<f32> = (0..20).map(|i| 2000.0 + i as f32).collect();
            let v = vec![0.2f32; 20];
            let nan = vec![f32::NAN; 20];
            db::insert_core_data(&conn, &w, "RAW", None, &d, &v, &nan, &nan, &nan).unwrap();
            db::apply_core_run_shifts(
                &mut conn,
                &w,
                &[db::RunShift { top: 2000.0, base: 2019.0, delta: 2.0, ..Default::default() }],
                &Default::default(),
                &Default::default(),
            )
            .unwrap();

            let path = std::env::temp_dir().join(format!("sandibumi_scalunit_{w}.csv"));
            std::fs::write(&path, "SAMPLE,DEPTH,PERM,PORO,PC,SW\n1,6600,100,0.20,1,1.0\n1,6600,100,0.20,10,0.5\n")
                .unwrap();
            let res = import_scal_files(
                &conn,
                &w,
                &[path.to_string_lossy().into_owned()],
                "long",
                "air_brine",
                72.0,
                Some("DELIVERY"),
                follow,
                "MD",
                unit,
            );
            std::fs::remove_file(&path).ok();
            assert!(res.error.is_none(), "{:?}", res.error);
            db::get_scal_pc(&conn, &w).unwrap().iter().filter_map(|r| r.depth).collect()
        };

        // Declared FT, core record not consulted: the plug lands on the project's scale.
        let depths = run(Some("ft"), false);
        assert!(
            depths.iter().all(|d| (d - 2011.68).abs() < 1e-2),
            "6600 ft is 2011.68 m, got {depths:?} — the delivery was filed raw"
        );

        // Declared FT, following the core: converted FIRST, then corrected by the core's 2 m.
        // Converting after the mapping would give (6600 + 2) x 0.3048 = 2012.29 — a metre
        // correction applied to a foot number, then scaled. That is the number this pins out.
        let depths = run(Some("ft"), true);
        assert!(
            depths.iter().all(|d| (d - 2013.68).abs() < 1e-2),
            "2011.68 m + the core's 2 m = 2013.68, got {depths:?} — the core record was applied \
             to a foot depth"
        );

        // Undeclared: unchanged from every SCAL import before this one.
        let depths = run(None, false);
        assert!(
            depths.iter().all(|d| (d - 6600.0).abs() < 1e-2),
            "no declaration means the project's own unit, got {depths:?}"
        );
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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
        let res = import_core_table(&conn, path, &mapping, None, Some(&w), Some("CEC"), Some("LAB"), false, "MD");
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
        let res2 = import_core_table(&conn, path2.to_str().unwrap(), &m2, None, Some(&w), None, Some("RUN2"), false, "MD");
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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
        let plain = import_core_table(&conn, path, &mapping, None, Some(&w), Some("CEC_ASWRITTEN"), Some("A"), false, "MD");
        assert!(plain.error.is_none(), "{:?}", plain.error);
        let rows = db::list_aux_data(&conn, &w, Some("CEC_ASWRITTEN")).unwrap();
        assert!(rows.iter().any(|r| (r.depth_top - 2005.0).abs() < 1e-3), "unmapped keeps the delivered depth");

        // On: each sample follows the barrel it was cut from — +1 above, +3 below.
        let followed = import_core_table(&conn, path, &mapping, None, Some(&w), Some("CEC_FOLLOWED"), Some("B"), true, "MD");
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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
        let plain = import_aux_file(&conn, &w, "XRD", path, Some("ASWRITTEN"), false, "MD", None);
        assert!(plain.error.is_none(), "{:?}", plain.error);
        let rows = db::list_aux_data(&conn, &w, Some("XRD")).unwrap();
        assert!(
            rows.iter().any(|r| (r.depth_top - 2005.0).abs() < 1e-3),
            "unmapped import keeps the delivered depth"
        );

        // On: each sample follows the barrel it was cut from.
        let followed = import_aux_file(&conn, &w, "XRD", path, Some("FOLLOWED"), true, "MD", None);
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
        let none = import_aux_file(&conn, &w2.to_string(), "XRD", path, None, true, "MD", None);
        assert!(none.error.is_none(), "{:?}", none.error);
        assert!(
            none.notes.unwrap_or_default().contains("no core to follow"),
            "asking to follow a core that is not there must be said out loud"
        );
        std::fs::remove_file(&xrd).ok();
    }

    /// Audit finding 8, fourth and last site. A point delivery — XRD, CEC, oil show, petrography,
    /// perforations — read in feet into a metre project filed every sample 3.28084x too deep, so
    /// a mineral count sat against the wrong sand and a perforation against the wrong interval.
    ///
    /// Two halves, pinned separately. An interval converts at BOTH ends: a thickness scaled at one
    /// end only is not a shallower sample, it is a sample of a different thickness, and a
    /// perforation that grew by 4 m is a completion record nobody can use. And the conversion runs
    /// BEFORE the core depth record, whose corrections are already on the project's scale.
    #[test]
    fn a_point_dataset_in_feet_converts_at_both_ends_before_it_follows_the_core() {
        // 6600 ft = 2011.68 m and 6620 ft = 2017.776 m, both exact (foot = 0.3048 m).
        let run = |body: &str, unit: Option<&str>, follow: bool| -> Vec<(f32, Option<f32>)> {
            let mut conn = Connection::open_in_memory().unwrap();
            db::create_schema(&conn).unwrap();
            crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
            let wid = uuid::Uuid::new_v4();
            db::insert_well(&conn, wid, "SANDI-AUX-UNIT", None, None, None).unwrap();
            let w = wid.to_string();

            // A cored interval on the PROJECT's scale, registered 2 m deeper against the log.
            let d: Vec<f32> = (0..30).map(|i| 2000.0 + i as f32).collect();
            let v = vec![0.2f32; 30];
            let nan = vec![f32::NAN; 30];
            db::insert_core_data(&conn, &w, "RAW", None, &d, &v, &nan, &nan, &nan).unwrap();
            db::apply_core_run_shifts(
                &mut conn,
                &w,
                &[db::RunShift { top: 2000.0, base: 2029.0, delta: 2.0, ..Default::default() }],
                &db::ShiftTargets::default(),
                &Default::default(),
            )
            .unwrap();

            let path = std::env::temp_dir().join(format!("sandibumi_auxunit_{w}.csv"));
            std::fs::write(&path, body).unwrap();
            let res = import_aux_file(&conn, &w, "XRD", path.to_str().unwrap(), Some("DELIVERY"), follow, "MD", unit);
            std::fs::remove_file(&path).ok();
            assert!(res.error.is_none(), "{:?}", res.error);
            let mut out: Vec<(f32, Option<f32>)> = db::list_aux_data(&conn, &w, Some("XRD"))
                .unwrap()
                .iter()
                .map(|r| (r.depth_top, r.depth_base))
                .collect();
            out.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-3);
            out
        };
        // The units row is one cell per column, and the unit is read off the TOP column's own
        // cell — a FEET under some other column says nothing about this one.
        let feet = "TOP,BASE,KAOLINITE\nFEET,FEET,\n6600,6620,0.10\n";

        // Declared by the file, core record not consulted.
        let rows = run(feet, None, false);
        assert_eq!(rows.len(), 1, "the units row is not counted as a sample: {rows:?}");
        assert!((rows[0].0 - 2011.68).abs() < 1e-2, "6600 ft is 2011.68 m, got {:?}", rows[0]);
        assert!(
            rows[0].1.is_some_and(|b| (b - 2017.776).abs() < 1e-2),
            "6620 ft is 2017.776 m — a converted top over an unconverted base would make this \
             sample 4608 m thick, got {:?}",
            rows[0]
        );

        // Following the core: converted FIRST, then the barrel's 2 m, and the base takes the
        // SAME offset so the sample keeps the 6.096 m of rock it measured.
        let rows = run(feet, None, true);
        assert!((rows[0].0 - 2013.68).abs() < 1e-2, "2011.68 + the core's 2 m, got {:?}", rows[0]);
        assert!(
            rows[0].1.is_some_and(|b| (b - 2019.776).abs() < 1e-2),
            "the base takes the top's offset, got {:?}",
            rows[0]
        );

        // Says nothing: unchanged from every point-data import before this one.
        let rows = run("TOP,BASE,KAOLINITE\n6600,6620,0.10\n", None, false);
        assert!((rows[0].0 - 6600.0).abs() < 1e-2, "no declaration means the project's own unit, got {:?}", rows[0]);
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

    /// SB-DIO-044 / SB-DIO-T62. CORRECTNESS - `docs/PRD_v2/21_data-io.md`
    /// D-25 and sections 4.8/6.8 require one version-independent policy: unknown and
    /// malformed headers are ignored and reported, a recognized pre-data order reversal
    /// is accepted and reported, and both ~V and ~W are mandatory before ~A.
    #[test]
    fn a_single_section_policy_reports_unknown_malformed_and_out_of_order_headers_in_las_2_and_3_and_refuses_data_before_version_or_well() {
        let write_fixture = |label: &str, body: &str| {
            let path = std::env::temp_dir().join(format!(
                "sandibumi-dio-044-{label}-{}.las",
                Uuid::new_v4()
            ));
            std::fs::write(&path, body).unwrap();
            path
        };

        let missing_version = write_fixture(
            "missing-version",
            "~WELL\nWELL. VERSION-BEFORE-DATA-CONTROL :\nNULL. -999.25 :\n~CURVE\nDEPT.M : depth\nGR.GAPI : gamma\n~ASCII\n1000 50\n",
        );
        for error in [
            parsers::parse_las_2(&missing_version).unwrap_err().to_string(),
            parsers::parse_las_2_all(&missing_version).unwrap_err().to_string(),
        ] {
            assert!(error.contains("~V") && error.contains("before ~A"), "{error}");
        }

        let invalid_version = write_fixture(
            "invalid-version",
            "~VERSION\nVERS. NOT-A-VERSION :\n~WELL\nWELL. VALID-VERSION-CONTROL :\n~CURVE\nDEPT.M : depth\nGR.GAPI : gamma\n~ASCII\n1000 50\n",
        );
        for error in [
            parsers::parse_las_2(&invalid_version).unwrap_err().to_string(),
            parsers::parse_las_2_all(&invalid_version).unwrap_err().to_string(),
        ] {
            assert!(error.contains("valid ~V") && error.contains("before ~A"), "{error}");
        }

        let missing_well = write_fixture(
            "missing-well",
            "~VERSION\nVERS. 2.0 :\nWRAP. NO :\n~CURVE\nDEPT.M : depth\nGR.GAPI : gamma\n~ASCII\n1000 50\n~WELL\nWELL. WELL-BEFORE-DATA-CONTROL :\n",
        );
        for error in [
            parsers::parse_las_2(&missing_well).unwrap_err().to_string(),
            parsers::parse_las_2_all(&missing_well).unwrap_err().to_string(),
        ] {
            assert!(error.contains("~W") && error.contains("before ~A"), "{error}");
        }

        let mut handling_by_version = Vec::new();
        for version in ["2.0", "3.0"] {
            let body = format!(
                "~VERSION\nVERS. {version} :\nWRAP. NO :\n\
                 ~CURVE\nDEPT.M : depth\nGR.GAPI : gamma\n\
                 ~X\nTHIS UNKNOWN BODY IS IGNORED\n\
                 ~\nTHIS MALFORMED BODY IS IGNORED\n\
                 ~WELL\nWELL. RECOGNIZED-ORDER-CONTROL :\nNULL. -999.25 :\n\
                 ~ASCII\n1000 50\n1001 51\n"
            );
            let path = write_fixture(&format!("reported-{}", version.replace('.', "-")), &body);

            let standard = parsers::parse_las_2(&path).unwrap();
            let all = parsers::parse_las_2_all(&path).unwrap();
            assert_eq!(standard.depth, vec![1000.0, 1001.0]);
            assert_eq!(all.depth, standard.depth, "both parser entry points must accept the same policy");
            assert_eq!(standard.section_policy, parsers::LAS_SECTION_POLICY_ID);
            assert_eq!(all.section_policy, standard.section_policy);
            assert_eq!(all.section_handling, standard.section_handling);

            let conn = Connection::open_in_memory().unwrap();
            db::create_schema(&conn).unwrap();
            let result = import_las_files_with(
                &conn,
                &[path.to_string_lossy().into_owned()],
                None,
                &LasImportOptions::default(),
            )
            .remove(0);
            assert!(result.error.is_none(), "{version}: {:?}", result.error);
            let public = serde_json::to_value(&result).unwrap();
            assert_eq!(public["section_policy"], "las_sections_v1");
            let handling = public["section_handling"]
                .as_array()
                .expect("section handling must be structured IPC data, not only prose");
            let actions: Vec<&str> = handling
                .iter()
                .map(|item| item["action"].as_str().unwrap())
                .collect();
            assert_eq!(
                actions,
                vec![
                    "unknown_section_ignored",
                    "malformed_header_ignored",
                    "out_of_order_section_accepted",
                ]
            );
            assert_eq!(handling[0]["header"], "~X");
            assert_eq!(handling[1]["header"], "~");
            assert_eq!(handling[2]["header"], "~WELL");
            let warning = result.warning.unwrap_or_default();
            assert!(warning.contains("las_sections_v1"), "{warning}");
            assert!(warning.contains("~X") && warning.contains("~WELL"), "{warning}");
            handling_by_version.push(actions.into_iter().map(str::to_string).collect::<Vec<_>>());

            std::fs::remove_file(path).ok();
        }
        assert_eq!(handling_by_version[0], handling_by_version[1]);

        std::fs::remove_file(missing_version).ok();
        std::fs::remove_file(invalid_version).ok();
        std::fs::remove_file(missing_well).ok();
    }

    /// Ad-hoc verification against a real field delivery — whatever LAS files sit in the
    /// configured fixture folder (`SANDIBUMI_FIELD_FIXTURES/las/`). Ignored by default and
    /// skipped with a printed reason when no folder is configured; run explicitly with
    /// `cargo test --release -- --ignored --nocapture test_import_real_field_files`.
    /// CORRECTNESS - `22_database-model.md` SB-DBM-T27 supplies the verification tolerance as
    /// fixture input and cites F-14/T-DB-16 for the 0.1524 m, 40-row, 6.1 m consequence. The
    /// regular control and the legacy refusal prevent an implementation that merely labels every
    /// delivery irregular or treats an absent verdict as permission.
    #[test]
    fn a_forty_row_gap_contradicts_a_regular_sampling_declaration_while_a_verified_regular_set_stays_regular_and_an_unverified_set_cannot_be_frame_read() {
        let make_las = |name: &str, source_indices: &[usize]| {
            let path = std::env::temp_dir().join(format!(
                "sampling-style-{name}-{}.las",
                std::process::id()
            ));
            let mut body = format!(
                "~VERSION\nVERS. 2.0 :\nWRAP. NO :\n~WELL\nSTEP.M 0.1524 :\nNULL. -999.25 :\nWELL. {name} :\n~CURVE\nDEPT.M : depth\nGR.GAPI : gamma\n~ASCII\n"
            );
            for &source_index in source_indices {
                let depth = 1000.0_f64 + source_index as f64 * 0.1524_f64;
                body.push_str(&format!("{depth:.4} {}\n", source_index + 10));
            }
            std::fs::write(&path, body).unwrap();
            path
        };

        let gap_indices: Vec<usize> = (0..=10).chain(51..=60).collect();
        let regular_indices: Vec<usize> = (0..=60).collect();
        let gap_path = make_las("FORTY-ROW-GAP", &gap_indices);
        let regular_path = make_las("REGULAR-CONTROL", &regular_indices);
        let explicit_tolerance: crate::units::DepthTolerance =
            serde_json::from_str(r#"{"value":0.0001,"unit":"M"}"#)
                .expect("the unit-typed frontend input must deserialize without reinterpretation");
        let regular_declaration = LasImportOptions {
            sampling_style: Some(crate::schema_vocab::SamplingStyle::ContinuousRegular),
            sampling_style_verify_tolerance: Some(explicit_tolerance),
            ..LasImportOptions::default()
        };

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let undeclared = LasImportOptions {
            sampling_style: None,
            ..LasImportOptions::default()
        };
        let undeclared_result = import_las_files_with(
            &conn,
            &[gap_path.to_string_lossy().into_owned()],
            None,
            &undeclared,
        )
        .remove(0);
        assert!(
            undeclared_result.error.as_deref().unwrap_or_default().contains("never inferred"),
            "a sampling style cannot appear by default: {:?}",
            undeclared_result.error
        );
        let no_tolerance = LasImportOptions {
            sampling_style: Some(crate::schema_vocab::SamplingStyle::ContinuousRegular),
            sampling_style_verify_tolerance: None,
            ..LasImportOptions::default()
        };
        let no_tolerance_result = import_las_files_with(
            &conn,
            &[gap_path.to_string_lossy().into_owned()],
            None,
            &no_tolerance,
        )
        .remove(0);
        assert!(
            no_tolerance_result.error.as_deref().unwrap_or_default().contains("no default ships"),
            "regular verification cannot borrow another tolerance: {:?}",
            no_tolerance_result.error
        );
        let refused_rows: i64 = conn
            .query_row("SELECT count(*) FROM wells", [], |row| row.get(0))
            .unwrap();
        assert_eq!(refused_rows, 0, "both missing declarations refuse before commit");

        let gap = import_las_files_with(
            &conn,
            &[gap_path.to_string_lossy().into_owned()],
            None,
            &regular_declaration,
        )
        .remove(0);
        assert!(gap.error.is_none(), "a contradicted declaration is retained as irregular: {:?}", gap.error);
        let warning = gap.warning.as_deref().unwrap_or_default();
        assert!(warning.contains("sampling declaration contradicted"), "{warning}");
        assert!(warning.contains("40 missing row"), "{warning}");
        assert!(warning.contains("1007.7724"), "the first post-gap depth must be named: {warning}");
        let gap_well = gap.well_id.as_deref().unwrap();
        let gap_verdict: (String, String, bool, f64, String, i64) = conn
            .query_row(
                "SELECT declared_sampling_style, effective_sampling_style, sampling_verified,
                        verification_tolerance, verification_tolerance_unit, gap_row_count
                 FROM import_sets WHERE well_id = ?1 AND set_name = 'RAW'",
                [gap_well],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )
            .unwrap();
        assert_eq!(
            gap_verdict,
            (
                "CONTINUOUS_REGULAR".into(),
                "CONTINUOUS_IRREGULAR".into(),
                true,
                explicit_tolerance.value,
                "M".into(),
                40,
            )
        );
        let (gap_depth, gap_columns) = crate::equations::fetch_curve_frame_from_set(
            &conn,
            gap_well,
            &["GR".to_string()],
            Some("RAW"),
            None,
        )
        .unwrap();
        let post_gap = gap_depth
            .iter()
            .position(|depth| (*depth - 1007.7724).abs() < 0.1524)
            .expect("the post-gap sample remains at its source depth rather than 6.1 m shallow");
        assert_eq!(gap_columns["GR"][post_gap], 61.0);

        let regular = import_las_files_with(
            &conn,
            &[regular_path.to_string_lossy().into_owned()],
            None,
            &regular_declaration,
        )
        .remove(0);
        assert!(regular.error.is_none(), "the regular control imports: {:?}", regular.error);
        assert!(!regular.warning.as_deref().unwrap_or_default().contains("sampling declaration contradicted"));
        let effective: String = conn
            .query_row(
                "SELECT effective_sampling_style FROM import_sets WHERE well_id = ?1 AND set_name = 'RAW'",
                [regular.well_id.as_deref().unwrap()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(effective, "CONTINUOUS_REGULAR");

        let legacy_well = Uuid::new_v4();
        db::insert_well(&conn, legacy_well, "UNVERIFIED-LEGACY", None, None, None).unwrap();
        db::insert_standard_curves(
            &conn,
            legacy_well,
            vec![1000.0, 1000.1524],
            vec![10.0, 11.0],
            vec![f32::NAN; 2],
            vec![f32::NAN; 2],
            vec![f32::NAN; 2],
            vec![f32::NAN; 2],
            vec![f32::NAN; 2],
        )
        .unwrap();
        let legacy_curve = db::upsert_curve_meta(
            &conn,
            &legacy_well.to_string(),
            "RAW",
            "GR",
            Some("GAPI"),
            Some("GR"),
            Some("legacy fixture"),
            None,
        )
        .unwrap();
        db::insert_curve_samples(
            &conn,
            &legacy_curve,
            &[1000.0, 1000.1524],
            &[10.0, 11.0],
        )
        .unwrap();
        let refusal = crate::equations::fetch_curve_frame_from_set(
            &conn,
            &legacy_well.to_string(),
            &["GR".to_string()],
            Some("RAW"),
            None,
        )
        .expect_err("a frame-indexed read needs a stored verification verdict");
        assert!(refusal.to_string().contains("sampling style has not been verified"), "{refusal}");

        std::fs::remove_file(gap_path).ok();
        std::fs::remove_file(regular_path).ok();
    }

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
