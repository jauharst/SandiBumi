//! Workflow chains (Phase 9): run an ordered sequence of deterministic modules across many
//! wells in one shot, with pollable progress and cooperative cancellation.
//!
//! Steps run *sequentially* (a later step, e.g. sw_indo, consumes an earlier step's outputs
//! like PHIE/PHIT), while the wells inside each step stay rayon-parallel via
//! [`workflow::run_workflow_module`]. Interval/zone parameters still apply: a chain run with
//! empty step params uses each module's manifest defaults, which `zone_params` then override
//! per zone — so a saved chain honours the zone parameters set in the Zones panel.
//!
//! Progress follows the same registry-and-poll model as `inversion.rs` (no Tauri events):
//! the frontend generates the job id, calls `run_workflow_chain`, and polls `get_chain_status`
//! on a timer while the run occupies its own command worker thread. `cancel_workflow_chain`
//! flips a shared flag the runner checks between steps.

use crate::{
    modules,
    workflow::{self, RunModuleRequest},
};
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// One module invocation in a chain. Empty maps fall back to the module manifest defaults
/// (which `zone_params` then override per zone), so the common case serializes compactly.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainStep {
    pub module: String,
    #[serde(default)]
    pub log_inputs: HashMap<String, String>,
    #[serde(default)]
    pub params: HashMap<String, f64>,
    #[serde(default)]
    pub opts: HashMap<String, String>,
}

/// Pollable status of a chain job. `wells_done` reaches `wells_total` as each step finishes
/// (step-level granularity — a step across even 100 wells is seconds).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ChainStatus {
    Queued,
    Running {
        step: usize,
        total_steps: usize,
        module: String,
        wells_done: usize,
        wells_total: usize,
    },
    Completed {
        steps_run: usize,
        curves_written: usize,
        wells: usize,
        errors: Vec<String>,
    },
    Cancelled {
        at_step: usize,
    },
    /// The worker died without reaching a terminal status — today only a panic inside
    /// `run_chain`, caught by `run_workflow_chain`'s `catch_unwind`.
    Failed {
        error: String,
    },
}

pub(crate) struct ChainJob {
    status: ChainStatus,
    cancel: Arc<AtomicBool>,
}

pub(crate) type ChainRegistry = Arc<Mutex<HashMap<Uuid, ChainJob>>>;

pub(crate) fn new_registry() -> ChainRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

fn set_status(registry: &ChainRegistry, job_id: Uuid, status: ChainStatus) {
    if let Some(job) = registry.lock().unwrap().get_mut(&job_id) {
        job.status = status;
    }
}

/// Registers a job id up front (so the poller finds it immediately) and returns its shared
/// cancel flag. Called before `run_chain` starts touching the database.
pub(crate) fn register(registry: &ChainRegistry, job_id: Uuid) -> Arc<AtomicBool> {
    let cancel = Arc::new(AtomicBool::new(false));
    registry
        .lock()
        .unwrap()
        .insert(job_id, ChainJob { status: ChainStatus::Queued, cancel: cancel.clone() });
    cancel
}

/// Records a wholesale failure — the worker stopped without reaching one of `run_chain`'s own
/// terminal statuses.
///
/// **This exists to release the project-switch guard, not only to report.** The registry has no
/// prune (contrast `jobs.rs`, which prunes on every terminal transition): `register` inserts,
/// `set_status` mutates, and nothing ever removes an entry. So a worker that dies mid-run leaves
/// its job `Queued`/`Running` in the map forever, `any_active` keeps answering true, and Open
/// Project / New Project / Compact Project are all refused for the rest of the session — each
/// telling the user to wait for a job that will never finish. The only way out was restarting the
/// app, which on a field project means paying the reopen cost again. `docs/review_triage.md`
/// finding 17.
pub(crate) fn failed(registry: &ChainRegistry, job_id: Uuid, error: String) {
    set_status(registry, job_id, ChainStatus::Failed { error });
}

/// Reads back a job's current status (for the `get_chain_status` command).
pub(crate) fn status(registry: &ChainRegistry, job_id: Uuid) -> Option<ChainStatus> {
    registry.lock().unwrap().get(&job_id).map(|j| j.status.clone())
}

/// True while any chain job is queued or running — switching projects mid-run would
/// make its later steps write into the newly opened database.
pub(crate) fn any_active(registry: &ChainRegistry) -> bool {
    registry
        .lock()
        .unwrap()
        .values()
        .any(|j| matches!(j.status, ChainStatus::Queued | ChainStatus::Running { .. }))
}

/// Requests cancellation of a running job; the runner stops before the next step.
pub(crate) fn cancel(registry: &ChainRegistry, job_id: Uuid) {
    if let Some(job) = registry.lock().unwrap().get(&job_id) {
        job.cancel.store(true, Ordering::SeqCst);
    }
}

fn complete_chain_sets(
    conn: &Connection,
    steps: &[ChainStep],
    well_ids: &[String],
    set_name: &str,
    input_set: Option<&str>,
    custody: &crate::equations::RunCustody,
) -> Result<HashMap<String, crate::equations::CompleteSetId>, String> {
    let manifests: HashMap<String, crate::modules::ModuleSpec> = crate::modules::list_modules()
        .into_iter()
        .map(|spec| (spec.name.clone(), spec))
        .collect();
    let modules: Vec<&str> = steps.iter().map(|step| step.module.as_str()).collect();
    let module_identity = format!("workflow: {}", modules.join(" -> "));
    let mut complete = Vec::with_capacity(well_ids.len());

    for well_id in well_ids {
        let zone_params =
            crate::db::list_zone_params(conn, well_id).map_err(|error| error.to_string())?;
        let mut produced = std::collections::HashSet::new();
        let mut produced_shale_clay_quantities = HashMap::new();
        let mut inputs = Vec::new();
        let mut parameters = Vec::new();
        let mut outputs = Vec::new();

        for (index, step) in steps.iter().enumerate() {
            let manifest = manifests
                .get(&step.module)
                .ok_or_else(|| format!("unknown module '{}'", step.module))?;
            let opts = workflow::build_opts(manifest, &step.opts, &step.log_inputs);
            let parameter_prefix = format!("step[{}].", index + 1);
            let (effective_parameters, _) = workflow::effective_module_parameters(
                manifest,
                &step.params,
                &step.opts,
                &opts,
                custody.source_note.trim(),
                &parameter_prefix,
            )?;
            parameters.extend(effective_parameters);
            let mask = workflow::mask_provenance(&step.opts);
            let mask_name = format!("{parameter_prefix}{}", workflow::MASK_PROVENANCE_KEY);
            if parameters
                .iter()
                .any(|parameter| parameter.name == mask_name)
            {
                return Err(format!(
                    "module '{}' declares an argument that collides with reserved run-provenance key '{}'",
                    step.module,
                    workflow::MASK_PROVENANCE_KEY
                ));
            }
            let mask_is_applied = mask["state"] == workflow::MASK_PROVENANCE_APPLIED;
            parameters.push(crate::equations::AncestryParameter {
                name: mask_name,
                value: mask,
                source: if mask_is_applied {
                    custody.source_note.clone()
                } else {
                    "SB-ENV-028 explicit no-mask run state".into()
                },
                resolution: mask_is_applied.then_some(
                    crate::equations::ParameterResolution::Explicit,
                ),
                manifest_version: None,
                decision: None,
            });

            for arg in &manifest.args {
                if arg.kind == crate::modules::ArgKind::Param {
                    for zone in zone_params
                        .iter()
                        .filter(|entry| entry.param_name == arg.name)
                    {
                        let Some(value) = zone.value_num else {
                            continue;
                        };
                        let source = zone
                            .value_text
                            .as_deref()
                            .map(str::trim)
                            .filter(|text| !text.is_empty())
                            .unwrap_or(custody.source_note.trim());
                        parameters.push(crate::equations::AncestryParameter {
                            name: format!("step[{}].{}@{}", index + 1, arg.name, zone.zone_name),
                            value: serde_json::json!(value),
                            source: source.to_string(),
                            resolution: Some(
                                crate::equations::ParameterResolution::Explicit,
                            ),
                            manifest_version: None,
                            decision: crate::param_sources::decision_for(
                                &arg.sources_topic,
                                &serde_json::json!(value),
                            ),
                        });
                    }
                }
            }

            let mut present_arguments = std::collections::HashSet::new();
            let mut missing_arguments = HashMap::new();
            let resolved_log_args = workflow::resolved_log_args_for_well(
                conn,
                well_id,
                manifest,
                &step.log_inputs,
                input_set,
                None,
                &produced,
            )?;
            for (arg_name, curve) in resolved_log_args {
                let curve = curve.trim().to_uppercase();
                if curve.is_empty() {
                    continue;
                }
                let argument = format!("step[{}].{}", index + 1, arg_name);
                let quantity_contract = manifest
                    .args
                    .iter()
                    .find(|arg| arg.name == arg_name)
                    .filter(|arg| !arg.accepted_shale_clay_quantities.is_empty());
                if produced.contains(&curve) {
                    let input = crate::equations::AncestryInput {
                        well_id: well_id.to_string(),
                        argument,
                        curve: curve.clone(),
                        log_set: set_name.to_string(),
                        set_version: None,
                        set_id: "SELF".into(),
                        chosen_curve_id: Some(format!("SELF:{curve}")),
                        rule: Some(crate::equations::CurveResolutionRule::WorkingInputSet),
                        rejected_candidates: Vec::new(),
                    };
                    if let Some(contract) = quantity_contract {
                        let accepted = contract
                            .accepted_shale_clay_quantities
                            .iter()
                            .map(|quantity| quantity.as_str())
                            .collect::<Vec<_>>()
                            .join(" or ");
                        let actual = produced_shale_clay_quantities
                            .get(&curve)
                            .copied()
                            .ok_or_else(|| {
                                format!(
                                    "module '{}' input '{}' requires typed {accepted} metadata, but chain-produced curve '{curve}' has no VSH/VCL quantity metadata",
                                    step.module, arg_name
                                )
                            })?;
                        if !contract.accepted_shale_clay_quantities.contains(&actual) {
                            return Err(format!(
                                "module '{}' input '{}' requires {accepted}, but chain-produced curve '{curve}' carries {} metadata",
                                step.module,
                                arg_name,
                                actual.as_str()
                            ));
                        }
                        parameters.push(crate::equations::AncestryParameter {
                            name: format!(
                                "{parameter_prefix}{}{}",
                                workflow::INPUT_QUANTITY_PROVENANCE_PREFIX,
                                arg_name
                            ),
                            value: serde_json::to_value(actual).map_err(|error| {
                                format!(
                                    "cannot serialize input quantity for {arg_name}: {error}"
                                )
                            })?,
                            source: "docs/PRD_v2/10_clay-volume.md SB-CLY-043".into(),
                            resolution: None,
                            manifest_version: None,
                            decision: None,
                        });
                    }
                    inputs.push(input);
                    present_arguments.insert(arg_name.clone());
                } else {
                    match crate::equations::resolve_ancestry_input(
                        conn, well_id, &argument, &curve, input_set, None,
                    ) {
                        Ok(input) => {
                            if let Some(contract) = quantity_contract {
                                let accepted = contract
                                    .accepted_shale_clay_quantities
                                    .iter()
                                    .map(|quantity| quantity.as_str())
                                    .collect::<Vec<_>>()
                                    .join(" or ");
                                let actual = workflow::shale_clay_quantity_for_ancestry_input(
                                    conn, &input,
                                )?
                                .ok_or_else(|| {
                                    format!(
                                        "module '{}' input '{}' requires typed {accepted} metadata, but resolved curve '{}' has no VSH/VCL quantity metadata; assign the physical family explicitly instead of relying on its mnemonic",
                                        step.module, arg_name, input.curve
                                    )
                                })?;
                                if !contract.accepted_shale_clay_quantities.contains(&actual) {
                                    return Err(format!(
                                        "module '{}' input '{}' requires {accepted}, but resolved curve '{}' carries {} metadata",
                                        step.module,
                                        arg_name,
                                        input.curve,
                                        actual.as_str()
                                    ));
                                }
                                parameters.push(crate::equations::AncestryParameter {
                                    name: format!(
                                        "{parameter_prefix}{}{}",
                                        workflow::INPUT_QUANTITY_PROVENANCE_PREFIX,
                                        arg_name
                                    ),
                                    value: serde_json::to_value(actual).map_err(|error| {
                                        format!(
                                            "cannot serialize input quantity for {arg_name}: {error}"
                                        )
                                    })?,
                                    source: "docs/PRD_v2/10_clay-volume.md SB-CLY-043".into(),
                                    resolution: None,
                                    manifest_version: None,
                                    decision: None,
                                });
                            }
                            inputs.push(input);
                            present_arguments.insert(arg_name.clone());
                        }
                        Err(error) => {
                            missing_arguments.insert(arg_name.clone(), error);
                        }
                    }
                }
            }
            for arg in manifest
                .args
                .iter()
                .filter(|arg| arg.kind == crate::modules::ArgKind::LogIn && arg.required)
            {
                let present = present_arguments.contains(&arg.name)
                    || arg
                        .required_any_of
                        .iter()
                        .any(|alternate| present_arguments.contains(alternate));
                if !present {
                    return Err(missing_arguments.remove(&arg.name).unwrap_or_else(|| {
                        format!("required input '{}' was not selected", arg.name)
                    }));
                }
            }

            let mut step_output_quantities = HashMap::new();
            for (curve, quantity) in workflow::resolved_shale_clay_output_names(manifest, &opts)? {
                let curve = curve.to_uppercase();
                if let Some(previous) = step_output_quantities.insert(curve.clone(), quantity) {
                    if previous != quantity {
                        return Err(format!(
                            "module '{}' assigns both {} and {} quantity metadata to output '{curve}'",
                            step.module,
                            previous.as_str(),
                            quantity.as_str()
                        ));
                    }
                }
            }
            for output in
                workflow::preview_output_names(&step.module, &step.log_inputs, &step.opts)?
            {
                let curve = output.name.to_uppercase();
                if let Some(quantity) = step_output_quantities.get(&curve).copied() {
                    if let Some(previous) =
                        produced_shale_clay_quantities.insert(curve.clone(), quantity)
                    {
                        if previous != quantity {
                            return Err(format!(
                                "chain assigns both {} and {} quantity metadata to output '{curve}'",
                                previous.as_str(),
                                quantity.as_str()
                            ));
                        }
                    }
                    let name = format!(
                        "{}{curve}",
                        workflow::OUTPUT_QUANTITY_PROVENANCE_PREFIX
                    );
                    let value = serde_json::to_value(quantity).map_err(|error| {
                        format!("cannot serialize output quantity for {curve}: {error}")
                    })?;
                    if let Some(previous) = parameters.iter().find(|parameter| parameter.name == name)
                    {
                        if previous.value != value {
                            return Err(format!(
                                "chain assigns conflicting VSH/VCL quantity metadata to output '{curve}'"
                            ));
                        }
                    } else {
                        parameters.push(crate::equations::AncestryParameter {
                            name,
                            value,
                            source: "docs/PRD_v2/10_clay-volume.md SB-CLY-043".into(),
                            resolution: None,
                            manifest_version: None,
                            decision: None,
                        });
                    }
                }
                produced.insert(curve.clone());
                outputs.push(crate::equations::AncestryOutput {
                    derivation: format!("step[{}] {}:{}", index + 1, step.module, output.arg),
                    curve,
                });
            }
            for (curve, kind) in workflow::resolved_flag_output_names(manifest, &opts)? {
                let name = format!(
                    "{parameter_prefix}{}{curve}",
                    workflow::FLAG_KIND_PROVENANCE_PREFIX
                );
                if parameters.iter().any(|parameter| parameter.name == name) {
                    return Err(format!(
                        "module '{}' declares an argument that collides with reserved flag-kind provenance key '{}'",
                        step.module, name
                    ));
                }
                parameters.push(crate::equations::AncestryParameter {
                    name,
                    value: serde_json::to_value(kind).map_err(|error| {
                        format!("cannot serialize flag kind for {curve}: {error}")
                    })?,
                    source: "SB-ENV-030 typed flag-kind declaration".into(),
                    resolution: None,
                    manifest_version: None,
                    decision: None,
                });
            }
        }

        outputs.sort_by(|left, right| left.curve.cmp(&right.curve));
        outputs.dedup_by(|left, right| left.curve == right.curve);
        let zones = crate::db::list_zones(conn, well_id).map_err(|error| error.to_string())?;
        let zone_scope = if zones.is_empty() {
            crate::equations::AncestryZoneScope::WholeWell
        } else {
            crate::equations::AncestryZoneScope::Defined(
                zones
                    .into_iter()
                    .map(|zone| crate::equations::AncestryZone {
                        name: zone.zone_name,
                        top: zone.top_depth,
                        base: zone.bottom_depth,
                        source: custody.source_note.clone(),
                    })
                    .collect(),
            )
        };
        let inputs_json = serde_json::to_string(&inputs).map_err(|error| error.to_string())?;
        let ancestry = crate::equations::CurveAncestry {
            schema_version: crate::equations::CURVE_ANCESTRY_SCHEMA_VERSION,
            module: module_identity.clone(),
            module_version: env!("CARGO_PKG_VERSION").into(),
            inputs,
            parameter_state: crate::equations::parameter_state_for(&parameters),
            parameters,
            zone_scope,
            actor: custody.actor.clone(),
            timestamp_utc_ms: crate::equations::ancestry_timestamp_utc_ms()?,
            outputs,
            depth_frame: None,
            zone_set: None,
            stochastic: None,
            applied_model: None,
            physics_attributes: Vec::new(),
        };
        let spec = crate::equations::CompleteLogSetSpec::try_new_with_legacy(
            set_name,
            ancestry,
            serde_json::to_value(steps).map_err(|error| error.to_string())?,
            &inputs_json,
        )?;
        complete.push(crate::equations::CompleteWellLogSet {
            well_id: well_id.clone(),
            spec,
        });
    }
    crate::equations::create_complete_log_sets_batch(conn, &complete)
}

/// Runs the chain to completion, updating the registry between steps. Intended to be called
/// on a command worker thread while the UI polls `status`. The job must already be
/// [`register`]ed; `cancel` is that job's shared flag.
pub(crate) fn run_chain(
    db: &Mutex<Connection>,
    registry: &ChainRegistry,
    job_id: Uuid,
    cancel: &AtomicBool,
    steps: &[ChainStep],
    well_ids: &[String],
    output_set: Option<&str>,
    input_set: Option<&str>,
    custody: &crate::equations::RunCustody,
    // Universal Processing panel handle (same `cancel` flag). Reports per-well progress in
    // addition to the chain-specific `ChainStatus` the Workflow Builder polls.
    job: Option<&crate::jobs::JobHandle>,
) {
    let total_steps = steps.len();
    let wells_total = well_ids.len();
    // One "unit" per (well, step): the panel's bar fills smoothly from 0 to steps × wells.
    if let Some(j) = job {
        j.running(total_steps * wells_total);
    }
    let mut curves_written = 0usize;
    let mut errors: Vec<String> = Vec::new();

    if let Err(error) = custody.validate() {
        set_status(
            registry,
            job_id,
            ChainStatus::Failed {
                error: error.clone(),
            },
        );
        if let Some(job) = job {
            job.failed(error);
        }
        return;
    }

    // An option selector is global metadata, so every step can be validated before reading one
    // sample or allocating the chain's shared output set. Keep the per-module validation too: it
    // is the algorithm boundary used by direct runs, Monte Carlo and future callers. This whole-
    // chain pass closes the frame-dependent failure where step one wrote a plausible value and a
    // typo in step two left that earlier value inside a chain reported as completed-with-errors.
    for step in steps {
        if let Err(error) = modules::validate_module_options(&step.module, &step.opts) {
            set_status(registry, job_id, ChainStatus::Failed { error: error.clone() });
            if let Some(job) = job {
                job.failed(error);
            }
            return;
        }
    }

    // ONE set event per well for the whole chain run: every step writes into the same
    // version, so re-running the chain bumps the set to N+1 (never overwrites history).
    let set_name = output_set.map(str::trim).filter(|s| !s.is_empty()).unwrap_or("INTERP");
    let preset_sets: HashMap<String, crate::equations::CompleteSetId> = {
        let conn = db.lock().unwrap();
        match complete_chain_sets(&conn, steps, well_ids, set_name, input_set, custody){
            Ok(sets) => sets,
            Err(error) => {
                set_status(
                    registry,
                    job_id,
                    ChainStatus::Failed {
                        error: error.clone()
    },
                );
                if let Some(job) = job {
                    job.failed(error);
                }
                return;
            }
        }
    };

    for (i, step) in steps.iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            set_status(registry, job_id, ChainStatus::Cancelled { at_step: i });
            if let Some(j) = job {
                // The chain loop itself prevented this step from starting, so this raw-flag
                // observation is evidence that cancellation changed what work ran.
                j.note_cancel_observed();
                j.cancelled();
            }
            return;
        }
        set_status(
            registry,
            job_id,
            ChainStatus::Running {
                step: i,
                total_steps,
                module: step.module.clone(),
                wells_done: 0,
                wells_total,
            },
        );
        if let Some(j) = job {
            j.set_current(Some(format!("Step {}/{}: {}", i + 1, total_steps, step.module)));
        }

        let req = RunModuleRequest {
            module: step.module.clone(),
            well_ids: well_ids.to_vec(),
            log_inputs: step.log_inputs.clone(),
            params: step.params.clone(),
            opts: step.opts.clone(),
            output_set: None, // preset_sets carries the chain-level set event
            // Curves a later step consumes from an earlier one are never in the input
            // set's archive, so they still resolve from the current store — chaining works.
            input_set: input_set.map(str::to_string)
        ,
            custody: custody.clone(),
        };
        let results = workflow::run_workflow_module_into(db, &req, Some(&preset_sets), Some(cancel), job);
        for r in &results {
            curves_written += r.output_curves.len();
            if let Some(e) = &r.error {
                errors.push(format!("{} @ {}: {e}", step.module, r.well_id));
            }
        }

        set_status(
            registry,
            job_id,
            ChainStatus::Running {
                step: i,
                total_steps,
                module: step.module.clone(),
                wells_done: wells_total,
                wells_total,
            },
        );
    }

    // A cancel during the LAST step drains it (wells skip via the per-well check) but there is
    // no next-step iteration to catch the flag, so confirm once more before reporting success —
    // otherwise a late cancel would misreport as Completed.
    let final_cancel_was_observed = job
        .map(|j| j.cancel_was_observed())
        .unwrap_or_else(|| cancel.load(Ordering::SeqCst));
    if final_cancel_was_observed {
        set_status(registry, job_id, ChainStatus::Cancelled { at_step: total_steps.saturating_sub(1) });
        if let Some(j) = job {
            j.cancelled();
        }
        return;
    }

    set_status(
        registry,
        job_id,
        ChainStatus::Completed { steps_run: total_steps, curves_written, wells: wells_total, errors },
    );
    if let Some(j) = job {
        j.complete();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, jobs, modules};
    use duckdb::params;
    use std::collections::BTreeMap;
    use uuid::Uuid as U;

    /// Minimal synthetic well: a clean-ish sand with GR/RHOB/NPHI so vsh_gr → phi_dn → sw_indo
    /// all produce finite outputs.
    fn seed_well(conn: &Connection) -> String {
        let id = U::new_v4();
        db::insert_well(conn, id, "CHAIN_TEST", Some("Synthetic"), None, None).unwrap();
        let n = 200usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let gr: Vec<f32> = (0..n).map(|i| 30.0 + (i % 40) as f32).collect();
        let res: Vec<f32> = vec![10.0; n];
        let nphi: Vec<f32> = vec![0.22; n];
        let rhob: Vec<f32> = vec![2.35; n];
        let dt: Vec<f32> = vec![90.0; n];
        let sp: Vec<f32> = vec![f32::NAN; n];
        db::insert_standard_curves(conn, id, depth, gr, res, nphi, rhob, dt, sp).unwrap();
        // SB-POR-024 (DEC-025): declare the fixture neutron's basis the way an import
        // would, so the N-D boundary refusal does not fire on tests about chain order.
        db::migrate_standard_curves_to_generic_store(conn).unwrap();
        let well = id.to_string();
        if let Some(entry) = db::list_generic_curve_catalog(conn, &well)
            .unwrap()
            .into_iter()
            .find(|entry| entry.mnemonic == "NPHI")
        {
            db::set_curve_neutron_basis(
                conn, &entry.curve_id, "SANDSTONE", "test fixture declaration (DEC-025)",
            )
            .unwrap();
        }
        well
    }

    fn finite(conn: &Connection, well: &str, curve: &str) -> i64 {
        conn.query_row(
            "SELECT count(*) FROM computed_curves WHERE well_id=?1 AND curve_name=?2 AND NOT isnan(value)",
            params![well, curve],
            |r| r.get(0),
        )
        .unwrap_or(0)
    }

    fn step(module: &str) -> ChainStep {
        // CHARACTERIZATION fixtures: pre-SB-CORE-004 manifest values made explicit so this test
        // remains about chain order and completion; they are not restored as shipping defaults.
        let params = match module {
            "vsh_gr" => HashMap::from([("GR_MA".into(), 20.0), ("GR_SH".into(), 120.0)]),
            "phi_dn" => HashMap::from([
                ("RHO_MA".into(), 2.645),
                ("RHO_SH".into(), 2.5),
                ("RHO_FL".into(), 1.0),
                ("NPHI_SH".into(), 0.35),
                ("RHO_DSH".into(), 2.65),
                ("RHO_W".into(), 1.0),
                ("PHIE_MAX".into(), 0.3),
            ]),
            "sw_indo" => HashMap::from([
                ("A".into(), 1.0),
                ("M".into(), 2.0),
                ("N".into(), 2.0),
                ("RT_SH".into(), 5.0),
                ("SWE_IRR".into(), 0.0),
                ("RW".into(), 0.1),
            ]),
            _ => HashMap::new(),
        };
        ChainStep {
            module: module.into(),
            log_inputs: HashMap::new(),
            params,
            opts: HashMap::new(),
        }
    }

    fn test_custody() -> crate::equations::RunCustody {
        crate::equations::RunCustody {
            actor: crate::equations::AncestryActor {
                kind: crate::equations::AncestryActorKind::Human,
                identity: "chain-acceptance-fixture".into(),
            },
            source_note: "characterization fixture values declared in this test".into(),
        }
    }

    fn precondition_custody() -> crate::equations::RunCustody {
        crate::equations::RunCustody {
            actor: crate::equations::AncestryActor {
                kind: crate::equations::AncestryActorKind::Human,
                identity: "precondition-contract-fixture".into(),
            },
            source_note: "SB-ENV-002 correctness fixture; VSH endpoint conditions are sourced by the module manifest"
                .into(),
        }
    }

    fn vsh_request(well_ids: Vec<String>, gr_ma: f64, gr_sh: f64) -> RunModuleRequest {
        RunModuleRequest {
            module: "vsh_gr".into(),
            well_ids,
            log_inputs: HashMap::new(),
            params: HashMap::from([("GR_MA".into(), gr_ma), ("GR_SH".into(), gr_sh)]),
            opts: HashMap::from([("OPT_GR".into(), "LINEAR".into())]),
            output_set: None,
            input_set: None,
            custody: precondition_custody(),
        }
    }

    fn run_with_processing_surface(
        db: &Mutex<Connection>,
        request: &RunModuleRequest,
    ) -> (Vec<workflow::ModuleRunResult>, jobs::JobView) {
        let registry = jobs::new_registry();
        let job_id = U::new_v4();
        let cancel = Arc::new(AtomicBool::new(false));
        let items = request
            .well_ids
            .iter()
            .map(|well_id| (well_id.clone(), "declared-precondition sample".into()))
            .collect();
        let job = jobs::register(
            &registry,
            job_id,
            "Module",
            request.module.clone(),
            items,
            cancel,
            true,
        );
        job.running(request.well_ids.len());
        let shared_cancel = job.cancel.clone();
        let results = workflow::run_workflow_module_into(
            db,
            request,
            None,
            Some(shared_cancel.as_ref()),
            Some(&job),
        );
        job.complete();
        let view = jobs::list(&registry)
            .into_iter()
            .find(|view| view.id == job_id.to_string())
            .expect("the Processing panel can poll the completed module job");
        (results, view)
    }

    fn assert_processing_refusal(
        results: &[workflow::ModuleRunResult],
        view: &jobs::JobView,
        expected: &str,
    ) {
        assert!(!results.is_empty(), "the route must report every requested well");
        for result in results {
            assert_eq!(result.outcome, workflow::ModuleRunOutcome::Failed);
            assert_eq!(result.rows_written, 0);
            assert!(result.output_curves.is_empty(), "a refusal must not claim an output curve");
            assert_eq!(result.error.as_deref(), Some(expected));
        }
        assert_eq!(view.outcome, Some(jobs::JobOutcome::Failed));
        assert_eq!(view.items.len(), results.len());
        for item in &view.items {
            assert_eq!(item.state, jobs::ItemState::Failed);
            assert_eq!(item.message.as_deref(), Some(expected));
        }
    }

    /// CORRECTNESS — `20_envcorr-qc.md` §4.1 SB-ENV-002 and §6.1 SB-ENV-T04.
    /// The 20/120 gAPI valid pair and the 120/20 invalid pair are already source-backed by the
    /// shipping `vsh_gr` manifest (`10_clay-volume.md` §3.2-§3.3; Geolog `vsh_gr.info` L48-L49
    /// and `vsh_gr.lls` L99-L102). The named-zone arm changes only sample zero, so an evaluator
    /// that checks one scalar before zone resolution—or ignores zone arrays—cannot pass.
    #[test]
    fn dialog_chain_batch_and_zone_override_routes_report_the_identical_precondition_refusal() {
        // Compile-time route inventory: the dialog calls the typed IPC wrapper, the wrapper invokes
        // the Tauri command, and that command supplies the same Processing handle used below.
        let dialog_source = include_str!("../../src/ui/moduleDialog.ts");
        let ipc_source = include_str!("../../src/ipc.ts");
        let command_source = include_str!("lib.rs");
        assert!(dialog_source.contains("await runWorkflowModule(req, scope.backend())"));
        assert!(ipc_source.contains("invoke<ModuleRunResult[]>(\"run_workflow_module\""));
        assert!(command_source.contains("workflow::run_workflow_module_into("));
        assert!(command_source.contains("Some(&job.cancel), Some(&job)"));

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let dialog_well = seed_well(&conn);
        let chain_well = seed_well(&conn);
        let batch_a = seed_well(&conn);
        let batch_b = seed_well(&conn);
        let zoned_well = seed_well(&conn);
        // SB-CLY-001 (DEC-036): an INVERTED pair no longer refuses - it is tokenized - so
        // the shared-refusal fixture is a RANGE breach (GR_MA = -1 gAPI), which still
        // refuses at the algorithm boundary with the same source-bearing shape. The route
        // identity under test is unchanged.
        db::upsert_md_zone(&conn, &zoned_well, "RANGE-BREACH", 1000.0, 1000.5).unwrap();
        db::set_zone_param(&conn, &zoned_well, "RANGE-BREACH", "GR_MA", Some(-1.0), None)
            .unwrap();
        db::set_zone_param(&conn, &zoned_well, "RANGE-BREACH", "GR_SH", Some(120.0), None)
            .unwrap();
        let database = Mutex::new(conn);

        let direct_context = modules::ModuleContext {
            n: 1,
            logs: HashMap::from([("GR".into(), vec![70.0])]),
            params: HashMap::from([("GR_MA".into(), vec![-1.0]), ("GR_SH".into(), vec![120.0])]),
            opts: HashMap::from([("OPT_GR".into(), "LINEAR".into())]),
            depth_unit: Default::default(),
        };
        let expected = modules::run_module("vsh_gr", &direct_context)
            .expect_err("the algorithm boundary must refuse a range breach before its body");
        assert!(expected.contains("vsh_gr.gr_ma_range"), "condition id missing: {expected}");
        assert!(expected.contains("value -1 gAPI at sample 0"), "offending sample missing: {expected}");
        assert!(expected.contains("200"), "declared range missing: {expected}");
        assert!(expected.contains("vsh_gr.info"), "condition source missing: {expected}");

        // Dialog/Tauri route: assert both the returned IPC payload and what Processing polls.
        let (dialog_results, dialog_job) =
            run_with_processing_surface(&database, &vsh_request(vec![dialog_well], -1.0, 120.0));
        assert_processing_refusal(&dialog_results, &dialog_job, &expected);

        // Batch route: every well gets the same visible refusal; none is collapsed into summary Ok.
        let (batch_results, batch_job) = run_with_processing_surface(
            &database,
            &vsh_request(vec![batch_a, batch_b], -1.0, 120.0),
        );
        assert_processing_refusal(&batch_results, &batch_job, &expected);

        // Zone route: the dialog/base values are valid; only the first sample's named-zone arrays
        // invert the endpoints. Identical refusal proves validation runs after per-sample resolution.
        let (zone_results, zone_job) =
            run_with_processing_surface(&database, &vsh_request(vec![zoned_well], 20.0, 120.0));
        assert_processing_refusal(&zone_results, &zone_job, &expected);

        // Saved-chain route: assert the chain-specific poll payload and the universal Processing
        // payload, not merely an internal Result returned by the module dispatcher.
        let chain_registry = new_registry();
        let chain_job_id = U::new_v4();
        let chain_cancel = register(&chain_registry, chain_job_id);
        let processing_registry = jobs::new_registry();
        let processing_job_id = U::new_v4();
        let processing_job = jobs::register(
            &processing_registry,
            processing_job_id,
            "Workflow",
            "declared precondition",
            vec![(chain_well.clone(), "declared-precondition sample".into())],
            chain_cancel.clone(),
            true,
        );
        processing_job.running(1);
        let invalid_step = ChainStep {
            module: "vsh_gr".into(),
            log_inputs: HashMap::new(),
            params: HashMap::from([("GR_MA".into(), -1.0), ("GR_SH".into(), 120.0)]),
            opts: HashMap::from([("OPT_GR".into(), "LINEAR".into())]),
        };
        run_chain(
            &database,
            &chain_registry,
            chain_job_id,
            chain_cancel.as_ref(),
            &[invalid_step],
            &[chain_well.clone()],
            None,
            None,
            &precondition_custody(),
            Some(&processing_job),
        );
        processing_job.complete();
        match status(&chain_registry, chain_job_id).expect("the Workflow Builder can poll its job") {
            ChainStatus::Completed { curves_written, errors, .. } => {
                assert_eq!(curves_written, 0);
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0], format!("vsh_gr @ {chain_well}: {expected}"));
            }
            other => panic!("expected a completed chain carrying the per-well refusal, got {other:?}"),
        }
        let chain_processing = jobs::list(&processing_registry)
            .into_iter()
            .find(|view| view.id == processing_job_id.to_string())
            .expect("Processing can poll the completed saved chain");
        assert_eq!(chain_processing.outcome, Some(jobs::JobOutcome::Failed));
        assert_eq!(chain_processing.items[0].state, jobs::ItemState::Failed);
        assert_eq!(chain_processing.items[0].message.as_deref(), Some(expected.as_str()));

        let written: i64 = database
            .lock()
            .unwrap()
            .query_row("SELECT count(*) FROM computed_curves", [], |row| row.get(0))
            .unwrap();
        assert_eq!(written, 0, "no route may write a curve after the shared refusal");
    }

    /// CORRECTNESS — `20_envcorr-qc.md` section 4.1 SB-ENV-009 and section 6.1
    /// SB-ENV-T15. The valid first step uses the source-owned VSH-GR 20/120 gAPI endpoints from
    /// `10_clay-volume.md` sections 3.2-3.3; the second step changes only the method id to a value
    /// outside the declared closed set. The acceptance surface is the saved-chain poll payload and
    /// the persisted curve/set inventory, not only an internal module `Result`.
    #[test]
    fn an_invalid_saved_chain_step_after_a_valid_step_refuses_before_any_previous_value_is_versioned() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let database = Mutex::new(conn);
        let registry = new_registry();
        let job_id = U::new_v4();
        let cancel = register(&registry, job_id);

        let valid_step = ChainStep {
            module: "vsh_gr".into(),
            log_inputs: HashMap::new(),
            params: HashMap::from([("GR_MA".into(), 20.0), ("GR_SH".into(), 120.0)]),
            opts: HashMap::from([("OPT_GR".into(), "LINEAR".into())]),
        };
        let mut invalid_step = valid_step.clone();
        invalid_step.opts.insert("OPT_GR".into(), "TYPO".into());

        run_chain(
            &database,
            &registry,
            job_id,
            cancel.as_ref(),
            &[valid_step, invalid_step],
            &[well.clone()],
            Some("METHOD-VALIDATION"),
            None,
            &precondition_custody(),
            None,
        );

        let error = match status(&registry, job_id).expect("the saved-chain refusal must be pollable") {
            ChainStatus::Failed { error } => error,
            other => panic!("an invalid later selector must fail the whole chain before step one, got {other:?}"),
        };
        assert!(error.contains("OPT_GR"), "selector name missing: {error}");
        assert!(error.contains("TYPO"), "unrecognised value missing: {error}");
        assert!(error.contains("LINEAR"), "permitted set missing: {error}");

        let conn = database.lock().unwrap();
        let set_count: i64 = conn
            .query_row("SELECT count(*) FROM log_sets WHERE well_id = ?1", params![well], |row| row.get(0))
            .unwrap();
        let current_count: i64 = conn
            .query_row("SELECT count(*) FROM computed_curves", [], |row| row.get(0))
            .unwrap();
        let archive_count: i64 = conn
            .query_row("SELECT count(*) FROM computed_curves_archive", [], |row| row.get(0))
            .unwrap();
        assert_eq!(set_count, 0, "an invalid later selector must allocate no chain version");
        assert_eq!(current_count, 0, "the valid first step must not survive as the invalid chain's current value");
        assert_eq!(archive_count, 0, "the valid first step must not survive in the invalid chain's archive");
    }

    /// CORRECTNESS — `20_envcorr-qc.md` section 4.3 SB-ENV-028 and section 6.3
    /// SB-ENV-T28. The expected mask identities are the test inputs themselves; `NONE` is the
    /// requirement's explicit no-mask state. The direct runs are otherwise identical and every
    /// zero-valued mask changes no sample, so only persisted provenance can distinguish them.
    /// The chain arm pins both sides again and requires the existing one-based step derivations,
    /// preventing an implementation that records one chain-wide mask from passing.
    #[test]
    fn every_completed_direct_and_chain_run_records_the_applied_mask_or_explicit_none_and_the_chain_step_position(
    ) {
        let path = std::env::temp_dir().join(format!(
            "sandibumi_mask_provenance_{}.duckdb",
            U::new_v4()
        ));
        let conn = db::init_db(path.to_str().unwrap()).unwrap();
        let direct_well = seed_well(&conn);
        let chain_well = seed_well(&conn);
        let depth: Vec<f32> = (0..200).map(|i| 1000.0 + i as f32 * 0.5).collect();
        crate::equations::write_computed_curve(
            &conn,
            &direct_well,
            &depth,
            "BADHOLE",
            &vec![0.0; depth.len()],
        )
        .unwrap();
        crate::equations::write_computed_curve(
            &conn,
            &chain_well,
            &depth,
            "BADHOLE",
            &vec![0.0; depth.len()],
        )
        .unwrap();
        crate::equations::write_computed_curve(
            &conn,
            &direct_well,
            &depth,
            "NONE",
            &vec![0.0; depth.len()],
        )
        .unwrap();
        let database = Mutex::new(conn);

        let mut direct = vsh_request(vec![direct_well.clone()], 20.0, 120.0);
        direct.output_set = Some("MASK-PROVENANCE-DIRECT".into());
        direct.opts.insert("MASK".into(), "badhole".into());
        let masked = workflow::run_workflow_module(&database, &direct);
        assert!(masked[0].error.is_none(), "masked direct run failed: {:?}", masked[0].error);
        direct.opts.remove("MASK");
        let unmasked = workflow::run_workflow_module(&database, &direct);
        assert!(unmasked[0].error.is_none(), "unmasked direct run failed: {:?}", unmasked[0].error);
        direct.opts.insert("MASK".into(), "NONE".into());
        let curve_named_none = workflow::run_workflow_module(&database, &direct);
        assert!(
            curve_named_none[0].error.is_none(),
            "a real mask curve named NONE failed: {:?}",
            curve_named_none[0].error
        );

        let mut masked_step = step("vsh_gr");
        masked_step.opts.insert("MASK".into(), "BADHOLE".into());
        let chain_steps = vec![masked_step, step("phi_dn")];
        let registry = new_registry();
        let job_id = U::new_v4();
        let cancel = register(&registry, job_id);
        run_chain(
            &database,
            &registry,
            job_id,
            cancel.as_ref(),
            &chain_steps,
            &[chain_well.clone()],
            Some("MASK-PROVENANCE-CHAIN"),
            None,
            &test_custody(),
            None,
        );
        assert!(
            matches!(status(&registry, job_id), Some(ChainStatus::Completed { ref errors, .. }) if errors.is_empty()),
            "the provenance fixture chain must complete: {:?}",
            status(&registry, job_id)
        );

        drop(database);
        let reloaded = db::init_db(path.to_str().unwrap()).unwrap();
        let direct_rows: Vec<(i64, String)> = {
            let mut statement = reloaded
                .prepare(
                    "SELECT version, params_json FROM log_sets
                     WHERE well_id = ?1 AND set_name = 'MASK-PROVENANCE-DIRECT'
                     ORDER BY version",
                )
                .unwrap();
            statement
                .query_map(params![direct_well], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .map(Result::unwrap)
                .collect()
        };
        assert_eq!(direct_rows.len(), 3, "all direct run versions must reload");
        let badhole_mask = serde_json::json!({ "state": "APPLIED", "curve": "BADHOLE" });
        let no_mask = serde_json::json!({ "state": "NONE" });
        let none_curve_mask = serde_json::json!({ "state": "APPLIED", "curve": "NONE" });
        let masked_ancestry = crate::equations::parse_curve_ancestry(&direct_rows[0].1).unwrap();
        let unmasked_ancestry = crate::equations::parse_curve_ancestry(&direct_rows[1].1).unwrap();
        let named_none_ancestry =
            crate::equations::parse_curve_ancestry(&direct_rows[2].1).unwrap();
        assert_eq!(
            masked_ancestry
                .parameters
                .iter()
                .find(|parameter| parameter.name == workflow::MASK_PROVENANCE_KEY)
                .map(|parameter| &parameter.value),
            Some(&badhole_mask)
        );
        assert_eq!(
            unmasked_ancestry
                .parameters
                .iter()
                .find(|parameter| parameter.name == workflow::MASK_PROVENANCE_KEY)
                .map(|parameter| &parameter.value),
            Some(&no_mask)
        );
        assert_eq!(
            named_none_ancestry
                .parameters
                .iter()
                .find(|parameter| parameter.name == workflow::MASK_PROVENANCE_KEY)
                .map(|parameter| &parameter.value),
            Some(&none_curve_mask)
        );

        let chain_json: String = reloaded
            .query_row(
                "SELECT params_json FROM log_sets
                 WHERE well_id = ?1 AND set_name = 'MASK-PROVENANCE-CHAIN'",
                params![chain_well],
                |row| row.get(0),
            )
            .unwrap();
        let chain_ancestry = crate::equations::parse_curve_ancestry(&chain_json).unwrap();
        for (name, expected) in [
            ("step[1].MASK", badhole_mask),
            ("step[2].MASK", no_mask),
        ] {
            assert_eq!(
                chain_ancestry
                    .parameters
                    .iter()
                    .find(|parameter| parameter.name == name)
                    .map(|parameter| &parameter.value),
                Some(&expected),
                "the reloaded chain provenance must record {name}"
            );
        }
        assert!(
            chain_ancestry
                .outputs
                .iter()
                .any(|output| output.derivation.starts_with("step[1] ")),
            "the mask must remain attached to a first-step output position"
        );
        assert!(
            chain_ancestry
                .outputs
                .iter()
                .any(|output| output.derivation.starts_with("step[2] ")),
            "the explicit no-mask state must remain attached to a second-step output position"
        );

        drop(reloaded);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("duckdb.wal"));
    }

    /// CORRECTNESS — `20_envcorr-qc.md` section 4.3 SB-ENV-030 and SB-ENV-T39. The
    /// mask/indicator identities come from the requirement's closed semantic distinction, not
    /// from sample values. Both arms rename the curves so a mnemonic heuristic cannot pass, and
    /// the assertions read only canonical run metadata after database reload.
    #[test]
    fn an_exclusion_mask_and_a_diagnostic_indicator_remain_distinguishable_without_reading_their_values(
    ) {
        let path = std::env::temp_dir().join(format!(
            "sandibumi_flag_kind_provenance_{}.duckdb",
            U::new_v4()
        ));
        let conn = db::init_db(path.to_str().unwrap()).unwrap();
        let direct_well = seed_well(&conn);
        let chain_well = seed_well(&conn);
        let depth: Vec<f32> = (0..200).map(|index| 1000.0 + index as f32 * 0.5).collect();
        for well in [&direct_well, &chain_well] {
            let curve_id = db::upsert_curve_meta(
                &conn,
                well,
                "RAW",
                "DRHO",
                Some("g/cc"),
                Some("DRHO"),
                Some("SB-ENV-T39 synthetic declared-unit fixture"),
                None,
            )
            .unwrap();
            db::insert_curve_samples(&conn, &curve_id, &depth, &vec![0.0; depth.len()]).unwrap();
        }
        let database = Mutex::new(conn);
        let rename_opts = HashMap::from([
            (
                format!("{}BADHOLE", workflow::OUT_NAME_PREFIX),
                "TO_EXCLUDE".into(),
            ),
            (
                format!("{}BADHOLE_DRHO_EVALUATED", workflow::OUT_NAME_PREFIX),
                "DRHO_DIAG".into(),
            ),
            ("DRHO_MAX_UNIT".into(), "g/cc".into()),
        ]);
        // `20_envcorr-qc.md` section 5.2 cites these named presets: 0.15 g/cc from the
        // carbonate/ITB gate and 2 in from delivered-study precedent. They make the synthetic
        // run executable; no flag sample value is used as an expected result.
        let cited_params = HashMap::from([
            ("DRHO_MAX".into(), 0.15),
            ("DCAL_MAX".into(), 2.0),
        ]);

        let direct = RunModuleRequest {
            module: "badhole".into(),
            well_ids: vec![direct_well.clone()],
            log_inputs: HashMap::new(),
            params: cited_params.clone(),
            opts: rename_opts.clone(),
            output_set: Some("FLAG-KIND-DIRECT".into()),
            input_set: None,
            custody: test_custody(),
        };
        let direct_result = workflow::run_workflow_module(&database, &direct);
        assert!(
            direct_result[0].error.is_none(),
            "direct flag-kind fixture failed: {:?}",
            direct_result[0].error
        );

        let mut chain_step = step("badhole");
        chain_step.params = cited_params;
        chain_step.opts = rename_opts;
        let registry = new_registry();
        let job_id = U::new_v4();
        let cancel = register(&registry, job_id);
        run_chain(
            &database,
            &registry,
            job_id,
            cancel.as_ref(),
            &[chain_step],
            &[chain_well.clone()],
            Some("FLAG-KIND-CHAIN"),
            None,
            &test_custody(),
            None,
        );
        assert!(
            matches!(status(&registry, job_id), Some(ChainStatus::Completed { ref errors, .. }) if errors.is_empty()),
            "chain flag-kind fixture failed: {:?}",
            status(&registry, job_id)
        );

        drop(database);
        let reloaded = db::init_db(path.to_str().unwrap()).unwrap();
        let read_kinds = |well: &str, set_name: &str| -> BTreeMap<String, String> {
            let mut statement = reloaded
                .prepare(
                    "SELECT rp.name, rp.value_json
                     FROM run_parameters rp
                     JOIN log_sets ls ON ls.set_id = rp.set_id
                     WHERE ls.well_id = ?1 AND ls.set_name = ?2
                       AND rp.name LIKE '%FLAG_KIND.%'
                     ORDER BY rp.name",
                )
                .unwrap();
            statement
                .query_map(params![well, set_name], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .map(Result::unwrap)
                .collect()
        };
        for (well, set_name, prefix) in [
            (&direct_well, "FLAG-KIND-DIRECT", ""),
            (&chain_well, "FLAG-KIND-CHAIN", "step[1]."),
        ] {
            let kinds = read_kinds(well, set_name);
            assert_eq!(
                kinds.get(&format!("{prefix}FLAG_KIND.TO_EXCLUDE")),
                Some(&"\"EXCLUSION_MASK\"".to_string()),
                "renamed exclusion mask metadata missing for {set_name}"
            );
            assert_eq!(
                kinds.get(&format!("{prefix}FLAG_KIND.DRHO_DIAG")),
                Some(&"\"DIAGNOSTIC_INDICATOR\"".to_string()),
                "renamed diagnostic indicator metadata missing for {set_name}"
            );
        }

        drop(reloaded);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("duckdb.wal"));
    }

    #[test]
    fn chain_runs_steps_in_order_and_completes() {
        let path = std::env::temp_dir().join("arshilla_chain_test.duckdb");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("duckdb.wal"));
        let conn = db::init_db(path.to_str().unwrap()).unwrap();
        let well = seed_well(&conn);

        let reg = new_registry();
        let job = U::new_v4();
        let cancel = register(&reg, job);
        let db = Mutex::new(conn);
        let steps = vec![step("vsh_gr"), step("phi_dn"), step("sw_indo")];

        run_chain(&db, &reg, job, &cancel, &steps, &[well.clone()], None, None, &test_custody(),
            None);

        match status(&reg, job).unwrap() {
            ChainStatus::Completed { steps_run, curves_written, wells, errors } => {
                assert_eq!(steps_run, 3);
                assert_eq!(wells, 1);
                assert!(curves_written > 0);
                assert!(errors.is_empty(), "unexpected errors: {errors:?}");
            }
            other => panic!("expected Completed, got {other:?}"),
        }

        let conn = db.lock().unwrap();
        // vsh_gr → VSH, phi_dn → collision-safe PHIE_DN_LIM, sw_indo → SWE all present and finite.
        assert!(finite(&conn, &well, "VSH") > 0);
        assert!(finite(&conn, &well, "PHIE_DN_LIM") > 0);
        assert_eq!(
            finite(&conn, &well, "PHIE"),
            0,
            "a D-N-only chain must not recreate the shared PHIE collision"
        );
        assert!(finite(&conn, &well, "SWE") > 0);
        // SB-CLY-043: the chain's pre-created SELF set is the producer record seen by later
        // steps, so it must carry the VSH identity plus each typed consumer's received quantity.
        // The mnemonic alone is not evidence: VSH and VCL share v/v and outputs are renameable.
        let params_json: String = conn
            .query_row(
                "SELECT params_json FROM log_sets
                 WHERE well_id = ?1 AND set_name = 'INTERP' AND module LIKE 'workflow:%'
                 ORDER BY version DESC LIMIT 1",
                params![well],
                |row| row.get(0),
            )
            .unwrap();
        let ancestry = crate::equations::parse_curve_ancestry(&params_json).unwrap();
        let quantity_parameters = ancestry
            .parameters
            .iter()
            .filter(|parameter| parameter.name.contains("QUANTITY."))
            .map(|parameter| (parameter.name.as_str(), parameter.value.as_str()))
            .collect::<HashMap<_, _>>();
        assert_eq!(quantity_parameters.get("OUTPUT_QUANTITY.VSH"), Some(&Some("VSH")));
        assert_eq!(
            quantity_parameters.get("step[2].INPUT_QUANTITY.VSH"),
            Some(&Some("VSH"))
        );
        assert_eq!(
            quantity_parameters.get("step[3].INPUT_QUANTITY.VSH"),
            Some(&Some("VSH"))
        );
    }

    #[test]
    fn chain_honours_precancellation() {
        let path = std::env::temp_dir().join("arshilla_chain_cancel.duckdb");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("duckdb.wal"));
        let conn = db::init_db(path.to_str().unwrap()).unwrap();
        let well = seed_well(&conn);

        let reg = new_registry();
        let job = U::new_v4();
        let cancel = register(&reg, job);
        cancel.store(true, Ordering::SeqCst); // cancel before it starts
        let db = Mutex::new(conn);

        run_chain(&db, &reg, job, &cancel, &[step("vsh_gr")], &[well.clone()], None, None, &test_custody(),
            None);

        match status(&reg, job).unwrap() {
            ChainStatus::Cancelled { at_step } => assert_eq!(at_step, 0),
            other => panic!("expected Cancelled, got {other:?}"),
        }
        let conn = db.lock().unwrap();
        assert_eq!(finite(&conn, &well, "VSH"), 0, "no work should have run");
    }

    /// T-SHELL-09. Opening or creating another project while a chain runs must be refused, or
    /// the chain's later steps write their curves into whichever database happens to be live by
    /// then — a well's VSH landing in a project that has never heard of that well. Nothing
    /// downstream can detect that, and the chain reports success.
    ///
    /// `open_project`, `new_project` and `compact_project` all gate on the same predicate
    /// (`lib.rs:267`, `:292`, `:205`), so what has to be true is that `any_active` tells the
    /// truth across a real chain's whole life — not just while it is visibly running.
    ///
    /// The first assertion is the one with a bug behind it. `lib.rs:2428` calls `register`
    /// BEFORE `std::thread::spawn` on `:2468`; move it inside the worker and there is a window
    /// where the command has already returned to the frontend, nothing is registered, and a
    /// switch clicked in that instant is allowed — after which the chain starts writing into the
    /// project the user just opened. The window is small and entirely real: the frontend fronts
    /// the Processing panel the moment Run returns, which is exactly when a user who changed
    /// their mind reaches for Open Project.
    #[test]
    fn a_registered_chain_holds_the_project_switch_shut_until_it_is_really_finished() {
        let path = std::env::temp_dir().join("sandibumi_chain_guard.duckdb");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("duckdb.wal"));
        let conn = db::init_db(path.to_str().unwrap()).unwrap();
        let well = seed_well(&conn);
        let db = Mutex::new(conn);
        let reg = new_registry();

        assert!(!any_active(&reg), "an idle app must let the user switch projects");

        // Queued: registered, not yet started. This is the pre-flight window.
        let job = U::new_v4();
        let cancel = register(&reg, job);
        assert!(matches!(status(&reg, job).unwrap(), ChainStatus::Queued));
        assert!(
            any_active(&reg),
            "the switch must already be shut the instant the job is registered — before the \
             worker thread has run a single step"
        );

        run_chain(&db, &reg, job, &cancel, &[step("vsh_gr")], &[well.clone()], None, None, &test_custody(),
            None);

        assert!(matches!(status(&reg, job).unwrap(), ChainStatus::Completed { .. }));
        assert!(
            !any_active(&reg),
            "a finished chain must release the guard, or the user is locked out of Open Project \
             for the rest of the session"
        );

        // Cancelling must release it too. A user who cancels a long chain specifically because
        // they want to open another project would otherwise still be refused.
        let job2 = U::new_v4();
        let cancel2 = register(&reg, job2);
        cancel2.store(true, Ordering::SeqCst);
        run_chain(&db, &reg, job2, &cancel2, &[step("vsh_gr")], &[well.clone()], None, None,
            &test_custody(), None);
        assert!(matches!(status(&reg, job2).unwrap(), ChainStatus::Cancelled { .. }));
        assert!(!any_active(&reg), "a cancelled chain must release the guard");

        // Two finished jobs must not mask a third that is still queued: the guard is an ANY,
        // and a registry that only looked at the most recent entry would pass every test above.
        let job3 = U::new_v4();
        let _cancel3 = register(&reg, job3);
        assert!(any_active(&reg), "one queued job among finished ones still holds the guard");

        drop(db);
        let _ = std::fs::remove_file(&path);
    }

    /// Nothing ever REMOVES an entry from the chain registry — `register` inserts, `set_status`
    /// mutates, and there is no prune (contrast `jobs.rs`, which prunes finished jobs). So a
    /// worker that dies without reaching a terminal status USED to leave its job `Queued` in the
    /// map forever, with `any_active` answering true: Open Project, New Project and Compact
    /// Project all refused for the rest of the session, each telling the user to wait for a job
    /// that would never finish, and the only way out a restart (`docs/review_triage.md`
    /// finding 17, fixed 2026-08-01 by `catch_unwind` in `run_workflow_chain`).
    ///
    /// The prune is still absent and that is deliberate: the entry has to survive so
    /// `get_chain_status` can tell the Workflow Builder WHY the run stopped. What changed is that
    /// a dead worker now reaches a terminal status, so the guard opens while the reason stays
    /// readable. This test pins both halves — a jam while nothing has reported, and the release.
    #[test]
    fn a_dead_chain_worker_reports_failure_and_releases_the_project_switch() {
        let reg = new_registry();
        let ghost = U::new_v4();
        let _cancel = register(&reg, ghost);

        // Mid-run: the guard is shut, which is correct — switching projects here would make the
        // chain's later steps write into the newly opened database.
        assert!(matches!(status(&reg, ghost).unwrap(), ChainStatus::Queued));
        assert!(any_active(&reg), "a registered job holds the switch shut");

        // The worker dies. `run_workflow_chain`'s catch_unwind reports it here.
        failed(&reg, ghost, "the workflow stopped unexpectedly (boom) — its results are incomplete".into());

        assert!(!any_active(&reg), "a failed chain must not keep the project switch shut");
        match status(&reg, ghost) {
            Some(ChainStatus::Failed { error }) => {
                assert!(error.contains("boom"), "the panic's own message must survive: {error}");
                assert!(
                    error.contains("incomplete"),
                    "and the user must be told the results are partial, not just that it stopped: {error}"
                );
            }
            other => panic!("expected Failed, got {other:?}"),
        }
        assert!(
            status(&reg, ghost).is_some(),
            "the entry stays readable — the reason is the whole point of not pruning it"
        );

        // Completing and cancelling still release it too, so the fix did not narrow the guard to
        // one exit.
        for (id, terminal) in [
            (U::new_v4(), ChainStatus::Completed { steps_run: 1, curves_written: 1, wells: 1, errors: vec![] }),
            (U::new_v4(), ChainStatus::Cancelled { at_step: 0 }),
        ] {
            register(&reg, id);
            set_status(&reg, id, terminal);
        }
        assert!(!any_active(&reg), "no terminal status may leave the guard shut");
    }
}
