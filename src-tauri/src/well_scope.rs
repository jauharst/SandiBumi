use duckdb::Connection;

/// The identity of a multi-well scope, carried to the backend instead of trusting a
/// frontend-resolved list. Group and All membership are deliberately resolved only when the
/// operation begins; Explicit is the user-selected active/pinned/selection/custom alternative.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WellScopeSelection {
    /// Whatever group is active when the backend command begins; All when no group is active.
    ActiveGroup,
    Group { group_id: String },
    All,
    Explicit { well_ids: Vec<String> },
}

/// Whether a backend operation is constrained by a caller-selected backend scope or is
/// deliberately exhaustive. The serialized spelling is part of the IPC disclosure: a command
/// that cannot be scoped must say `PROJECT_WIDE`, never merely return a surprisingly large list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WellIterationPolicy {
    BackendScoped,
    ProjectWide,
}

/// The backend-owned inventory of operations that resolve or deliberately decline well scope.
/// `operation` is also the user-facing prefix used by resolver failures; `function` pins the
/// corresponding Tauri command boundary in SB-DBM-T37.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WellScopeOperation {
    pub function: &'static str,
    pub operation: &'static str,
    pub policy: WellIterationPolicy,
    pub iterates_wells: bool,
}

pub const WELL_SCOPE_OPERATIONS: &[WellScopeOperation] = &[
    WellScopeOperation { function: "export_report_batch", operation: "PDF report batch", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "export_deck", operation: "PowerPoint deck", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "export_report_docx_batch", operation: "Word report batch", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "export_workbook", operation: "Excel workbook", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_equation", operation: "equation run", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "resolve_plot_bindings", operation: "multi-well plot binding", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "resolve_well_scope", operation: "well-scope preview", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_workflow_module", operation: "module run", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "stats_curve_summary", operation: "curve statistics", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "stats_pair_summary", operation: "pair statistics", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "stats_versus_sets", operation: "log-set comparison", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "stats_thickness", operation: "thickness statistics", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "stats_fit", operation: "statistical fit", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_pay_summary", operation: "pay summary", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_cutoff_sweep", operation: "cutoff sweep", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_monte_carlo", operation: "Monte Carlo run", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_ml", operation: "machine-learning run", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "apply_ml_model", operation: "saved-model application", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "curve_sampling", operation: "curve-sampling QC", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_ml_eval", operation: "machine-learning evaluation", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_cuddy_foil", operation: "Cuddy FOIL fit", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_rtc_fit", operation: "RtC fit", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_s_factor_fit", operation: "S-factor fit", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_shf_fit", operation: "saturation-height fit", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_thomeer_fit", operation: "Thomeer fit", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_hfu_cluster", operation: "HFU clustering", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_lorenz", operation: "Lorenz plot", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_facies_confusion", operation: "facies tie", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_sandimin", operation: "SandiMin run", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "set_well_param_overrides", operation: "per-well parameter edit", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "list_plug_choices", operation: "plug-QC choices", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_plug_qc", operation: "plug QC", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_reframe", operation: "Reframe", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "upsert_top", operation: "formation-top edit", policy: WellIterationPolicy::BackendScoped, iterates_wells: false },
    WellScopeOperation { function: "autocorrelate_top", operation: "top autocorrelation", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "autocorrelate_multi", operation: "multi-marker autocorrelation", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "run_workflow_chain", operation: "workflow chain", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "list_wells", operation: "well inventory", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "materialize_tvd", operation: "TVD materialization", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "wells_in_polygon", operation: "map polygon selection", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "check_top_order", operation: "top-order check", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "check_contact_consistency", operation: "contact-consistency check", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "check_fwl_agreement", operation: "FWL-agreement check", policy: WellIterationPolicy::BackendScoped, iterates_wells: true },
    WellScopeOperation { function: "check_referential_integrity", operation: "referential-integrity check", policy: WellIterationPolicy::ProjectWide, iterates_wells: true },
];

fn registered_operation(operation: &str) -> Result<&'static WellScopeOperation, String> {
    WELL_SCOPE_OPERATIONS
        .iter()
        .find(|entry| entry.operation == operation)
        .ok_or_else(|| format!("unregistered backend well-scope operation '{operation}'"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct WellIterationDisclosure {
    pub scope: WellIterationPolicy,
    pub wells_touched: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct ProjectWideResult<T> {
    pub scope: WellIterationPolicy,
    pub wells_touched: usize,
    #[serde(flatten)]
    pub result: T,
}

pub fn project_wide_disclosure(
    conn: &Connection,
    operation: &str,
) -> Result<WellIterationDisclosure, String> {
    let registered = registered_operation(operation)?;
    if registered.policy != WellIterationPolicy::ProjectWide {
        return Err(format!(
            "{} is registered as BACKEND_SCOPED and cannot declare PROJECT_WIDE",
            registered.operation
        ));
    }
    let wells_touched = conn
        .query_row("SELECT count(*) FROM wells", [], |row| row.get::<_, i64>(0))
        .map_err(|error| format!("{operation}: could not count project wells: {error}"))?
        .max(0) as usize;
    Ok(WellIterationDisclosure {
        scope: WellIterationPolicy::ProjectWide,
        wells_touched,
    })
}

pub fn declare_project_wide<T>(
    conn: &Connection,
    operation: &str,
    result: T,
) -> Result<ProjectWideResult<T>, String> {
    let disclosure = project_wide_disclosure(conn, operation)?;
    Ok(ProjectWideResult {
        scope: disclosure.scope,
        wells_touched: disclosure.wells_touched,
        result,
    })
}

pub fn resolve_well_scope(
    conn: &Connection,
    selection: &WellScopeSelection,
    operation: &str,
) -> Result<Vec<String>, String> {
    let registered = registered_operation(operation)?;
    if registered.policy != WellIterationPolicy::BackendScoped {
        return Err(format!(
            "{} is PROJECT_WIDE and cannot accept a narrower well scope",
            registered.operation
        ));
    }
    match selection {
        WellScopeSelection::ActiveGroup => {
            let mut stmt = conn
                .prepare("SELECT group_id FROM well_groups WHERE active = 1 ORDER BY group_id")
                .map_err(|error| format!("{operation}: could not resolve the active well group: {error}"))?;
            let active = stmt
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|error| format!("{operation}: could not resolve the active well group: {error}"))?
                .collect::<duckdb::Result<Vec<_>>>()
                .map_err(|error| format!("{operation}: could not resolve the active well group: {error}"))?;
            match active.as_slice() {
                [] => resolve_well_scope(conn, &WellScopeSelection::All, operation),
                [group_id] => resolve_well_scope(
                    conn,
                    &WellScopeSelection::Group { group_id: group_id.clone() },
                    operation,
                ),
                _ => Err(format!(
                    "{operation}: more than one well group is active; repair the group state before running"
                )),
            }
        }
        WellScopeSelection::Group { group_id } => {
            let exists = conn
                .query_row(
                    "SELECT COUNT(*) FROM well_groups WHERE group_id = ?1",
                    [group_id],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|error| format!("{operation}: could not resolve well group '{group_id}': {error}"))?;
            if exists != 1 {
                return Err(format!(
                    "{operation}: well group '{group_id}' no longer exists; reopen the scope control"
                ));
            }
            let mut stmt = conn
                .prepare(
                    "SELECT w.well_id
                     FROM well_group_members m
                     JOIN wells w ON w.well_id = m.well_id
                     WHERE m.group_id = ?1
                     ORDER BY w.well_name, w.well_id",
                )
                .map_err(|error| format!("{operation}: could not read well group '{group_id}': {error}"))?;
            stmt.query_map([group_id], |row| row.get::<_, String>(0))
                .map_err(|error| format!("{operation}: could not read well group '{group_id}': {error}"))?
                .collect::<duckdb::Result<Vec<_>>>()
                .map_err(|error| format!("{operation}: could not read well group '{group_id}': {error}"))
        }
        WellScopeSelection::All => {
            let mut stmt = conn
                .prepare("SELECT well_id FROM wells ORDER BY well_name, well_id")
                .map_err(|error| format!("{operation}: could not resolve all wells: {error}"))?;
            stmt.query_map([], |row| row.get::<_, String>(0))
                .map_err(|error| format!("{operation}: could not resolve all wells: {error}"))?
                .collect::<duckdb::Result<Vec<_>>>()
                .map_err(|error| format!("{operation}: could not resolve all wells: {error}"))
        }
        WellScopeSelection::Explicit { well_ids } => {
            let mut seen = std::collections::HashSet::new();
            for well_id in well_ids {
                if !seen.insert(well_id) {
                    return Err(format!(
                        "{operation}: explicit well scope repeats '{well_id}'; refresh the scope control"
                    ));
                }
                let exists = conn
                    .query_row(
                        "SELECT COUNT(*) FROM wells WHERE well_id = ?1",
                        [well_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(|error| format!("{operation}: could not validate well '{well_id}': {error}"))?;
                if exists != 1 {
                    return Err(format!(
                        "{operation}: well '{well_id}' no longer exists; refresh the scope control"
                    ));
                }
            }
            Ok(well_ids.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use duckdb::params;

    fn add_well(conn: &Connection, name: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO wells (well_id, well_name) VALUES (?1, ?2)",
            params![id, name],
        )
        .unwrap();
        id
    }

    fn rust_function_source<'a>(source: &'a str, name: &str) -> &'a str {
        let needle = format!("fn {name}(");
        let start = source.find(&needle).unwrap_or_else(|| panic!("missing Rust function {name}"));
        let open = start
            + source[start..]
                .find('{')
                .unwrap_or_else(|| panic!("missing body for Rust function {name}"));
        let mut depth = 0_i32;
        for (offset, ch) in source[open..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return &source[start..=open + offset];
                    }
                }
                _ => {}
            }
        }
        panic!("unterminated Rust function {name}")
    }

    /// CORRECTNESS — SB-CORE-035: the backend, not a dialog snapshot, owns current Group/All
    /// membership for every operation advertised through the shared well-scope control. Explicit
    /// active/pinned/selection/custom scope remains an intentional alternative, but it can name
    /// only wells that still exist. DEC-003 supplies the representative pilot-chain inventory;
    /// the remaining entries close the same contract on every other live shared-scope caller.
    #[test]
    fn every_backend_scoped_operation_uses_current_group_membership_and_refuses_stale_or_unknown_scope() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let first = add_well(&conn, "FIRST");
        let second = add_well(&conn, "SECOND");
        let group_id = db::create_well_group(&conn, "CURRENT", &[first.clone(), second.clone()])
            .unwrap();
        db::set_active_well_group(&conn, Some(&group_id)).unwrap();

        let group: WellScopeSelection = serde_json::from_value(serde_json::json!({
            "kind": "group",
            "group_id": group_id,
        }))
        .unwrap();
        let group_id = match &group {
            WellScopeSelection::Group { group_id } => group_id.clone(),
            _ => panic!("the TypeScript group selector must deserialize as Group"),
        };
        assert_eq!(
            resolve_well_scope(&conn, &group, "well-scope preview").unwrap(),
            vec![first.clone(), second.clone()],
            "the initial backend resolution must include both current members"
        );

        // The dialog's old snapshot still contained SECOND. The backend must not: it resolves the
        // same group identity after membership changes rather than accepting those stale bytes.
        db::set_well_group_members(&conn, &group_id, std::slice::from_ref(&first)).unwrap();
        assert_eq!(
            resolve_well_scope(&conn, &group, "well-scope preview").unwrap(),
            vec![first.clone()],
            "removed membership must disappear without rebuilding the dialog"
        );

        assert_eq!(
            resolve_well_scope(
                &conn,
                &WellScopeSelection::All,
                "well-scope preview",
            )
            .unwrap(),
            vec![first.clone(), second.clone()],
            "All is an explicit alternative and must resolve the current project, not the group"
        );
        assert_eq!(
            resolve_well_scope(
                &conn,
                &WellScopeSelection::Explicit { well_ids: vec![second.clone()] },
                "well-scope preview",
            )
            .unwrap(),
            vec![second.clone()],
            "an explicit Custom/Active/Pinned/Selection scope is not silently replaced by Group"
        );
        let active: WellScopeSelection =
            serde_json::from_value(serde_json::json!({ "kind": "active_group" })).unwrap();
        assert_eq!(
            resolve_well_scope(&conn, &active, "well-scope preview").unwrap(),
            vec![first.clone()],
            "ActiveGroup must resolve membership at the backend command boundary"
        );
        db::set_active_well_group(&conn, None).unwrap();
        assert_eq!(
            resolve_well_scope(&conn, &active, "well-scope preview").unwrap(),
            vec![first.clone(), second.clone()],
            "without an active group, the established active-group UI contract is all current wells"
        );
        assert_eq!(
            serde_json::to_value(WellScopeSelection::Explicit { well_ids: vec![second.clone()] })
                .unwrap(),
            serde_json::json!({ "kind": "explicit", "well_ids": [second] }),
            "the Rust selector must preserve the TypeScript IPC tag and explicit identities"
        );

        let missing_group = resolve_well_scope(
            &conn,
            &WellScopeSelection::Group { group_id: "missing-group".into() },
            "well-scope preview",
        )
        .unwrap_err();
        assert!(missing_group.contains("well-scope preview"), "the operation must be named: {missing_group}");
        assert!(missing_group.contains("group"), "the missing identity must be named: {missing_group}");

        let missing_well = resolve_well_scope(
            &conn,
            &WellScopeSelection::Explicit { well_ids: vec!["missing-well".into()] },
            "well-scope preview",
        )
        .unwrap_err();
        assert!(missing_well.contains("well-scope preview"), "the operation must be named: {missing_well}");
        assert!(missing_well.contains("missing-well"), "the stale identity must be named: {missing_well}");

        let repeated = resolve_well_scope(
            &conn,
            &WellScopeSelection::Explicit { well_ids: vec![first.clone(), first] },
            "well-scope preview",
        )
        .unwrap_err();
        assert!(repeated.contains("repeats"), "duplicate explicit identities must be refused: {repeated}");

        // This is deliberately a command inventory rather than a helper-only test: a wrapper that
        // still accepted the frontend snapshot would otherwise make every resolver assertion above
        // pass while shipping the original defect. TypeScript signatures make the corresponding
        // callers supply `scope.backend()`; `tsc --noEmit` is the second half of that boundary.
        let lib = include_str!("lib.rs");
        for function in [
            "export_deck",
            "export_workbook",
            "run_equation",
            "stats_curve_summary",
            "stats_pair_summary",
            "stats_versus_sets",
            "stats_thickness",
            "stats_fit",
            "run_workflow_module",
            "run_workflow_chain",
            "run_pay_summary",
            "run_cutoff_sweep",
            "run_monte_carlo",
            "run_ml",
            "apply_ml_model",
            "curve_sampling",
            "run_ml_eval",
            "run_cuddy_foil",
            "run_rtc_fit",
            "run_s_factor_fit",
            "run_shf_fit",
            "run_thomeer_fit",
            "run_hfu_cluster",
            "run_lorenz",
            "run_facies_confusion",
            "run_sandimin",
            "set_well_param_overrides",
            "list_plug_choices",
            "run_plug_qc",
            "run_reframe",
            "autocorrelate_top",
            "autocorrelate_multi",
            "export_report_batch",
            "export_report_docx_batch",
            "resolve_plot_bindings",
        ] {
            let source = rust_function_source(lib, function);
            assert!(
                source.contains("scope: well_scope::WellScopeSelection"),
                "{function} must accept a backend scope identity rather than only frontend well ids"
            );
            assert!(
                source.contains("well_scope::resolve_well_scope"),
                "{function} must resolve that identity inside its own backend command"
            );
        }

        let scoped_top_edit = rust_function_source(lib, "upsert_top");
        assert!(
            scoped_top_edit.contains("scope: Option<well_scope::WellScopeSelection>"),
            "upsert_top must accept scope authority for active-group correlation writes"
        );
        assert!(
            scoped_top_edit.contains("well_scope::resolve_well_scope"),
            "upsert_top must validate active-group correlation writes in its backend command"
        );
    }

    /// CORRECTNESS — SB-DBM-T37. The exact 540-well project and 12-well active group are from
    /// `docs/PRD_v2/22_database-model.md` §6 T37; SB-CORE-035 supplies the backend-enforcement
    /// contract. The source inventory is the second side of the proof: exercising the resolver
    /// alone would still pass while an IPC command silently enumerated the whole project.
    #[test]
    fn every_well_iterating_backend_command_scopes_the_sql_to_the_active_twelve_of_five_hundred_and_forty_or_declares_project_wide() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let mut well_ids = Vec::with_capacity(540);
        for index in 0..540 {
            well_ids.push(add_well(&conn, &format!("SCOPE-{index:04}")));
        }
        let active_ids = well_ids[..12].to_vec();
        let group_id = db::create_well_group(&conn, "ACTIVE-TWELVE", &active_ids).unwrap();
        db::set_active_well_group(&conn, Some(&group_id)).unwrap();

        let lib = include_str!("lib.rs");
        let mut functions = std::collections::HashSet::new();
        let mut operations = std::collections::HashSet::new();
        for registered in WELL_SCOPE_OPERATIONS {
            assert!(functions.insert(registered.function), "duplicate command-scope registry function {}", registered.function);
            assert!(operations.insert(registered.operation), "duplicate command-scope registry operation {}", registered.operation);
            if registered.policy == WellIterationPolicy::ProjectWide {
                continue;
            }
            let resolved = resolve_well_scope(
                &conn,
                &WellScopeSelection::ActiveGroup,
                registered.operation,
            )
            .unwrap();
            assert_eq!(
                resolved.len(),
                12,
                "{} must touch only the cited active 12 at its backend SQL boundary",
                registered.function
            );
            assert_eq!(
                resolved,
                active_ids,
                "{} must resolve the current members rather than a client snapshot",
                registered.function
            );
            if !registered.iterates_wells {
                continue;
            }
            let source = rust_function_source(lib, registered.function);
            assert!(
                source.contains("scope: well_scope::WellScopeSelection"),
                "{} must accept a backend scope identity rather than a client-filtered well list",
                registered.function
            );
            assert!(
                source.contains("well_scope::resolve_well_scope"),
                "{} must resolve the current group inside its own backend boundary",
                registered.function
            );
            assert!(
                source.contains(registered.operation),
                "{} must resolve the registry operation identity '{}'",
                registered.function,
                registered.operation
            );
        }

        let listed = db::list_wells_by_ids(&conn, &active_ids).unwrap();
        assert_eq!(listed.len(), 12, "the scoped summary query must materialize 12 rows, not 540 then filter");

        let db_source = include_str!("db.rs");
        let scoped_well_loader = rust_function_source(db_source, "list_wells_by_ids");
        assert!(
            scoped_well_loader.contains("FROM wells WHERE well_id IN"),
            "the scoped well loader must constrain the wells table at the SQL boundary"
        );
        let scoped_contact_loader = rust_function_source(db_source, "list_fluid_contacts_scoped");
        assert!(
            scoped_contact_loader.contains("WHERE well_id IN"),
            "the scoped contact loader must constrain contact and marker-link queries in SQL"
        );

        for (file, source) in [
            ("lib.rs", lib),
            ("contacts.rs", include_str!("contacts.rs")),
            ("tops.rs", include_str!("tops.rs")),
        ] {
            assert!(
                !source.contains("db::list_wells("),
                "{file} must not materialize every project well behind a scoped backend command"
            );
        }
        assert!(
            include_str!("contacts.rs").contains("db::list_fluid_contacts_for_wells"),
            "cross-well contact checks must use the SQL-scoped contact loader"
        );
        assert!(
            include_str!("contacts.rs").contains("db::list_wells_by_ids"),
            "cross-well contact checks must use the SQL-scoped well loader"
        );
        assert!(
            include_str!("tops.rs").contains("db::list_wells_by_ids"),
            "the top-order iterator must use the SQL-scoped well loader"
        );
        let statistics = include_str!("statistics.rs");
        assert!(
            statistics.contains("db::list_wells_by_ids"),
            "statistics labels must load only the backend-authorized well ids"
        );
        assert!(
            !statistics.contains("SELECT well_id, well_name FROM wells"),
            "statistics must not hide a full-project well scan behind scoped requests"
        );

        let integrity = rust_function_source(lib, "check_referential_integrity");
        assert!(
            integrity.contains("well_scope::declare_project_wide"),
            "the deliberately exhaustive integrity command must declare PROJECT_WIDE in its response"
        );
        let disclosed = project_wide_disclosure(&conn, "referential-integrity check").unwrap();
        assert_eq!(disclosed.scope, WellIterationPolicy::ProjectWide);
        assert_eq!(disclosed.wells_touched, 540, "the exhaustive side must name the cited full-project row count");
    }
}
