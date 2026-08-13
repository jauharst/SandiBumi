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

pub fn resolve_well_scope(
    conn: &Connection,
    selection: &WellScopeSelection,
    operation: &str,
) -> Result<Vec<String>, String> {
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
            resolve_well_scope(&conn, &group, "test operation").unwrap(),
            vec![first.clone(), second.clone()],
            "the initial backend resolution must include both current members"
        );

        // The dialog's old snapshot still contained SECOND. The backend must not: it resolves the
        // same group identity after membership changes rather than accepting those stale bytes.
        db::set_well_group_members(&conn, &group_id, std::slice::from_ref(&first)).unwrap();
        assert_eq!(
            resolve_well_scope(&conn, &group, "test operation").unwrap(),
            vec![first.clone()],
            "removed membership must disappear without rebuilding the dialog"
        );

        assert_eq!(
            resolve_well_scope(
                &conn,
                &WellScopeSelection::All,
                "test operation",
            )
            .unwrap(),
            vec![first.clone(), second.clone()],
            "All is an explicit alternative and must resolve the current project, not the group"
        );
        assert_eq!(
            resolve_well_scope(
                &conn,
                &WellScopeSelection::Explicit { well_ids: vec![second.clone()] },
                "test operation",
            )
            .unwrap(),
            vec![second.clone()],
            "an explicit Custom/Active/Pinned/Selection scope is not silently replaced by Group"
        );
        let active: WellScopeSelection =
            serde_json::from_value(serde_json::json!({ "kind": "active_group" })).unwrap();
        assert_eq!(
            resolve_well_scope(&conn, &active, "test operation").unwrap(),
            vec![first.clone()],
            "ActiveGroup must resolve membership at the backend command boundary"
        );
        db::set_active_well_group(&conn, None).unwrap();
        assert_eq!(
            resolve_well_scope(&conn, &active, "test operation").unwrap(),
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
            "test operation",
        )
        .unwrap_err();
        assert!(missing_group.contains("test operation"), "the operation must be named: {missing_group}");
        assert!(missing_group.contains("group"), "the missing identity must be named: {missing_group}");

        let missing_well = resolve_well_scope(
            &conn,
            &WellScopeSelection::Explicit { well_ids: vec!["missing-well".into()] },
            "test operation",
        )
        .unwrap_err();
        assert!(missing_well.contains("test operation"), "the operation must be named: {missing_well}");
        assert!(missing_well.contains("missing-well"), "the stale identity must be named: {missing_well}");

        let repeated = resolve_well_scope(
            &conn,
            &WellScopeSelection::Explicit { well_ids: vec![first.clone(), first] },
            "test operation",
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
            "run_multimin",
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
}
