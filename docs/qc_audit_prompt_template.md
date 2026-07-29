# SandiBumi tool-by-tool QC audit prompt

A reusable prompt for auditing one SandiBumi tool/module at a time, end to end:
database → backend → domain correctness → frontend → UI/UX → cross-function integration.
Built from a live architecture scan (2026-07-21) so it names real tables, real commands,
real files, and the project's own known landmines instead of generic advice.

**How to use it:** pick one tool from the inventory in §2, fill in the `{{TOOL}}` /
`{{FILES}}` / `{{DOCS}}` placeholders in §1 from that tool's inventory row, and run it as a
fresh Claude Code session (or hand it to an agent). Do one tool per pass — this project's
convention (confirmed 2026-07-21) is serial QC/fixes in the main working tree, not
parallel/branched work. Findings and fixes still land directly in this working tree, per
that same convention; only commit when Jauhar asks.

Domain knowledge lives here in `docs/`, not in machine-local memory (per this repo's own
CLAUDE.md convention) — so this file is the one to update if the checklist itself needs to
evolve, not a memory note.

---

## 1. The master prompt (copy, fill in `{{...}}`, run)

```
Run a full A-to-Z QC audit of the SandiBumi tool "{{TOOL}}" at D:\XX. SandiBumi.

Files involved: {{FILES}}
Relevant spec-of-record doc(s), if any: {{DOCS}}   (docs/ wins over code when they conflict —
read it before judging correctness; if there's no doc, say so rather than inventing one.)

Before finding anything new: grep REVIEW.md and ROADMAP.md for this tool's name so you don't
re-report something already known, already fixed, or already deliberately deferred by Jauhar
(deferrals are recorded as explicit decisions, not oversights — don't flag those as bugs).

Follow this project's own house QC method (used in AUDIT-2026-07-20.md): review each
dimension below, then adversarially try to REFUTE every finding you raise before reporting
it — a finding only survives if you couldn't talk yourself out of it. Report survivors using
this schema: Title / Area / Effort / Where (file:line) / Evidence / Suggested fix / how you
tried to refute it and why that failed.

### A. Database
- Which tables does {{TOOL}} actually read/write? Check against the real schema: wells,
  standard_curves, computed_curves (+ _archive), log_sets, curve_meta/curve_samples, tops,
  zones/zone_params, core_data, aux_data, scal_pc, documents, well_groups/_members, well_path.
- computed_curves and its archive are deliberately PK-less — uniqueness is a write-discipline
  contract, not a DB constraint. Confirm any writer here does delete-then-append via
  db::with_txn, never an upsert/ON CONFLICT path.
- Does it record provenance correctly — a log_sets row with module/params_json/inputs_json —
  so list_computed_catalog can show what produced its output curves? Or does it correctly use
  skip_version (e.g. a dashboard-style overwrite-in-place) — and is that the RIGHT choice for
  this tool, not just a copy-pasted default?
- NaN-as-missing discipline preserved end to end? Any place a bare 0 or Some(0.0) could leak in
  for a missing sample instead of NaN?
- If it's an importer: does it survive the known malformed-LAS/DLIS defect family — duplicate
  or non-monotonic depth, all-null depth column, non-DEPT-named depth index (TDEP/MD via the
  column-0 fallback), truncated/misaligned rows? This exact bug class has recurred 3+ times.

### B. Backend (Rust)
- Confirm the actual .rs file(s) and #[tauri::command](s) match what's documented for this tool
  — flag if it's calling something orphaned or half-wired (precedent: petrophysics.rs is dead
  code with no `mod` declaration; inversion.rs's start_inversion is a hardcoded stub still
  exposed over IPC).
- Error handling: Result<T,String> surfaced usefully to the user; batch operations isolate a
  single well/file's failure rather than aborting the whole run (ImportResult/ModuleRunResult
  -style per-item results)?
- Concurrency: if it processes many wells, does it separate an in-memory rayon-parallel compute
  phase from a single batched DB-write phase (the workflow.rs pattern), rather than locking the
  shared Mutex<Connection> once per well?
- Does {{TOOL}} correctly consume the workflow-level MASK option in BOTH its inputs and its
  outputs? (MASK-not-applied-to-inputs was a real, repeated bug class — gr_normalize and
  log_predict both got it wrong before being fixed.)
- Singularity/edge behavior: what happens at φ=0, VSH=1, PHIT=0, all-inputs-missing, or a
  saturation solve at its bounds? Non-finite propagation at these edges is the single most
  repeated defect class in this codebase's own audit history (6+ separate recurrences).
- Unit tests: do they exist, do they cover the singularities above (not just a happy path), and
  is there a real-field-data regression test (pipeline_blso_test-style) or is this module only
  synthetic-tested? Note if `cargo test` / `cargo check` aren't currently clean.

### C. Domain / petrophysical correctness
- Does the implementation match its spec doc line-for-line where the doc gives exact formulas
  or constants? Quote the doc section you checked against.
- If it touches chart-digitized data (chartOverlays.ts / neutron_charts.rs), does it reproduce
  the source chart within this project's own stated tolerance convention (e.g. the ~0.04 pu
  worked-example check documented in CLAUDE.md)?
- If no doc exists for this tool, say so explicitly — don't silently invent a reference.

### D. Frontend (TypeScript wiring)
- Confirm the actual panel/dialog file(s) and which ipc.ts calls they make.
- Cross-cutting behaviors this tool's panel is expected to participate in — check each
  explicitly, don't assume:
  - dataVersion subscription (auto-refresh after an import/run/edit/undo elsewhere)
  - themeVersion subscription (canvas panels must re-read CSS vars on theme switch, not just
    on next redraw)
  - a local race-guard generation counter on every async reload (gen/reloadGen/loadGen pattern)
  - filterByActiveGroup applied to its well list — remember the BACKEND does not enforce well-
    group scoping at all; a dialog that forgets this call will silently run on the wrong wells
  - recordProcess(...) called for user-visible actions, so it shows up in the History panel
  - undo/redo: per this project's rule, module/equation *runs* are deliberately NOT undoable
    (they're versioned instead) — only UI/data edits should push an undo entry. Confirm this
    tool is on the correct side of that line.
  - dispose/cleanup: does closing this panel actually unsubscribe/remove its listeners, or does
    it leak (check for a returned cleanup closure wired into workspace.ts's DomPanel.dispose)?
- Does the frontend validate/shape params before calling the backend, or silently forward
  garbage and rely on the backend to reject it usefully?

### E. UI/UX
- Is it actually reachable from where a user would look (ribbon tab / Advance tab / dock pane
  menu), and does the auto-generated parameter dialog (if it's a manifest-driven module) read
  as a sane, labeled form rather than a raw params dump?
- Does a failed run surface a clear, specific error, or fail silently / show a generic message?
- Does it register with the shared jobs.rs "Processing" panel for progress/cancel if it's a
  long-running/batch operation, or does it block the UI with no feedback?
- Does it repaint correctly under every theme (dark/light + the client-brand palettes:
  Pertamina/Halliburton/Schlumberger/LAPI-ITB/white)?
- Any of this project's known dockview footguns apply here (theme prop must be a className not
  an object; dock.layout() needs a manual ResizeObserver kick; initialWidth is unreliable before
  first layout; detached/inactive tabs; `[hidden]` vs CSS display)?

### F. Cross-function / integration
- Trace {{TOOL}}'s curve provenance chain both ways: what upstream tool(s) produce its inputs
  (via fetch_curve_frame's precedence: standard_curves → computed_curves → generic RAW store),
  and what downstream tool(s)/plots consume its outputs?
- If it can run inside a Workflow Builder chain: does its own-set-id protection correctly stop
  an older "input set" selection from shadowing an earlier step's fresh output in the same run?
- Does it show up correctly afterward in list_computed_catalog / the Curve Catalog so a human
  (or another tool) can tell what produced a given curve, from what params?
- Does it read or mutate any shared global UI state that other panels also depend on
  (appState.selectedInterval, hoverDepth, pinnedWellId, activeWellGroup, dataVersion) in a way
  that could surprise an unrelated panel?

Report only findings that survived your own refutation attempt. If you find zero real issues,
say that plainly — don't manufacture low-value findings to fill out the report.
```

---

## 2. Tool inventory (fill `{{TOOL}}`/`{{FILES}}`/`{{DOCS}}` from here)

### Deterministic modules (`modules.rs` unless noted) — run via `run_workflow_module`
| Tool | File | Category | Spec doc |
|---|---|---|---|
| vsh_gr | modules.rs | VSH | — |
| vsh_dn | modules.rs | VSH | — |
| phi_den | modules.rs | Porosity | — |
| phi_dn | modules.rs | Porosity | — |
| phi_son | modules.rs | Porosity | — |
| phimax | modules.rs | Porosity | — |
| ssc | ssc.rs | Porosity | docs/method_ssc_sspw.md |
| sspw | ssc.rs | Porosity | docs/method_ssc_sspw.md |
| ftemp_grad | modules.rs | Prep | — |
| precalc | modules.rs | Prep | docs/research_2026-07/ref_kkt_onwj_wave_e.md |
| badhole | modules.rs | Prep | — |
| condflag | modules.rs | Prep | — |
| nphimat | modules.rs | Prep | (neutron_charts.rs, SLB 2013 charts) |
| gascorr | modules.rs | Prep | docs/research_2026-07/ref_kkt_onwj_wave_e.md |
| gr_hole_corr | modules.rs | Prep | — |
| nphi_env_corr | modules.rs | Prep | — |
| rhob_hole_corr | modules.rs | Prep | — |
| gr_normalize | modules.rs | Prep | docs/workflow_standards.md |
| log_predict | modules.rs | Prep | — |
| sw_arch | modules.rs | Saturation | — |
| sw_indo | modules.rs | Saturation | — |
| sw_sim | modules.rs | Saturation | — |
| sw_rtc | lrlc.rs | Saturation | docs/method_lrlc_rtc_imts.md |
| sw_imts | lrlc.rs | Saturation | docs/method_lrlc_rtc_imts.md |
| multimin (legacy 4-comp) | multimin.rs | Saturation | — (superseded in UI by SandiMin) |
| perm_wyllie_rose | modules.rs | Permeability | — |
| perm_coates | modules.rs | Permeability | — |
| perm_transform | modules.rs | Permeability | — |
| thin_bed_ts | modules.rs | ThinBeds | — |
| depth_shift | modules.rs | Prep | — |
| splice | modules.rs | Prep | — |
| sw_height | satheight.rs | Saturation | — |
| electrofacies | facies.rs | Facies | — |
| gmm_facies | facies.rs | Facies | — |

### Standalone / batch / scripting tools
| Tool | File | Command(s) | Spec doc |
|---|---|---|---|
| SandiMin (generalized multimin) | multimin2.rs | run_multimin, multimin_library, multimin_fluid_calc, multimin_dry_clay, multimin_fluid_from_precalc | docs/multimin_ref_spec.md, docs/multimin_ip_spec.md |
| Pay summary | workflow.rs | run_pay_summary | docs/workflow_standards.md |
| Cutoff sensitivity sweep | workflow.rs | run_cutoff_sweep | docs/research_2026-07/ref_kkt_onwj_wave_e.md |
| Monte Carlo | montecarlo.rs | run_monte_carlo | — |
| ML bridge | ml.rs | run_ml | — |
| Workflow chains | chain.rs/workflow.rs | run_workflow_chain, get_chain_status, cancel_workflow_chain | — |
| Legacy inversion (STUB — flag as dead/demo) | inversion.rs | start_inversion, get_inversion_status | — |
| Equations (Rhai + Python) | equations.rs, python_engine.rs | save_equation, run_equation, python_status | — |

### Data import
| Tool | File | Command |
|---|---|---|
| LAS import | ingest.rs/parsers.rs | import_las_files |
| Core CSV | ingest.rs | import_core_csv |
| Tops CSV | ingest.rs | import_tops_csv |
| Aux data | ingest.rs | import_aux_data |
| Deviation survey | ingest.rs/deviation.rs | import_deviation_csv |
| DLIS | dlis.rs | import_dlis_file |
| SCAL Pc/Sw | ingest.rs/satheight.rs | import_scal_csv |
| Well locations | ingest.rs/geo.rs | import_well_locations |

### Viz / reporting / editing
| Tool | File | Command(s) |
|---|---|---|
| Composite log plot | composite.rs | render_composite, export_composite_svg, export_composite_pdf |
| Report generator | report.rs | render_report, export_report_pdf, export_report_batch |
| LAS export | export.rs | export_las |
| Curve edit (shift/set/blank/interpolate/scale) | curve_edit.rs | edit_curve, restore_curve_values |

### Frontend panels/dialogs (pair with the backend tool(s) they call)
moduleDialog · workflowDialog · monteCarloDialog · mlDialog · multiminDialog · summaryDialog ·
cutoffDialog · dashboardPanel · mapPanel · zonesDialog · autoCorrDialog · compositeDialog ·
reportDialog · logViewPanel · curveEditDialog · topsEditor · layoutPropsDialog · histogramPanel ·
crossplotPanel · pickettPanel · correlationPanel · objectTree · topsPanel · wellGroups ·
inspectorPanel · dbInspectorPanel · sqlQueryPanel · historyPanel · processingPanel

(Full file-by-file description of what each does and which IPC calls it makes is in this
session's exploration — re-derive with a quick grep of `src/ui/*.ts` if this file goes stale;
don't trust this list blindly after major refactors.)

---

## 3. Suggested sequencing (highest-value first)

1. **SandiMin (multimin2.rs)** and the **RtC/IMTS saturation methods (lrlc.rs)** — most complex
   physics, most recently rebuilt, has dedicated spec docs to check against.
2. Anything still `[ ]` unchecked in REVIEW.md (150 of 222 items as of 2026-07-21) — these are
   unit-tested and tsc-clean but never human-verified against real field data. Start with the
   two P0-tier items still unchecked despite being code-complete: DLIS null-sentinel screening
   and SandiMin under-determined-model rejection.
3. **Well-group scoping across every batch dialog** — the backend enforces nothing here; a QC
   pass that just re-checks "does this dialog actually call filterByActiveGroup" across
   moduleDialog/workflowDialog/monteCarloDialog/mlDialog/multiminDialog/summaryDialog/
   cutoffDialog/dashboardPanel/autoCorrDialog/correlationPanel/reportDialog is cheap and finds
   real bugs if any dialog was added later without it.
4. **Dead/stub code**: petrophysics.rs (fully orphaned) and inversion.rs (hardcoded stub still
   on the IPC surface) — decide whether to delete or clearly quarantine them so they don't get
   mistaken for live paths.
5. Everything else, module-by-module, cheapest/lowest-risk last (simple single-formula modules
   like perm_transform, splice, depth_shift).
