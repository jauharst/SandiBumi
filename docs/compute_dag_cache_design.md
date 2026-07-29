# Compute DAG + content-addressed result cache — design (PARKED, not implemented)

> Status: **designed 2026-07-29, parked by Jauhar's direction** (priority went to the
> machine-level make.py DAG/cache in `D:\XX. Clauding`). Drafted by an opus subagent against
> the code as of this date; the supervising session independently verified the chain.rs
> execution seams and the one-set-event-per-chain-run subtlety, but the per-module claims
> (depth_shift/splice/log_predict output-name drift, the curve_edit.rs second point-edit
> path, the §0 fact table's line numbers) have **not** been re-verified line-by-line.
> **Re-verify §0 against current code before implementing anything here.**
>
> Implementation-relevant fixes landed the same day (independently of this design):
> ssc.rs gas-conditioning RMS fix, NaN-contract fixes, hardened local `limit` — so line
> numbers in ssc.rs/modules.rs may have drifted slightly already.

---

# Phase A — Compute DAG + Content-Addressed Result Cache for Workflow Chains

**Repo:** `D:\XX. Arshilla` (SandiBumi). All paths below are `src-tauri/src/…` unless noted.
**Scope:** manual chain runs only, no UX change beyond additive reporting. Phase B = hooks only.

## 0. Verification of the established facts

| Claim | Verdict | Evidence |
|---|---|---|
| `ModuleSpec.args` declares I/O via `ArgKind` | verified | `modules.rs:17-26`, `28-48`, `50-57` |
| Execution is `fn(&ModuleContext) -> ModuleOutputs`, dispatched by `run_module` | verified | `modules.rs:146`, `232-283` |
| Runtime output keys "should mirror" spec `LogOut` names | **FALSE for 3+ modules** | see §1.1 |
| Chain = `Vec<ChainStep>`, strictly sequential | verified | `chain.rs:25-34`, loop at `chain.rs:159-212` |
| Later steps consume earlier outputs by curve-name lookup | verified | `workflow.rs:231-249` → `equations.rs:768-826` → `equations.rs:277-359` |
| Resolution order standard → computed → RAW | verified | `equations.rs:317-357` (standard wins only if not all-NaN, `:330-342`) |
| Chains not persisted; outputs versioned via `log_sets` | verified | `equations.rs:577-590`, `651-680`; chains persist as `workflow` *documents* only (`workflowDialog.ts` `WORKFLOW_DOC_TYPE`) |
| `computed_curves` is deliberately PK-less; DELETE-then-append in a txn | verified | `db.rs:115-136`, `equations.rs:596-638`, `700-756`, `979-1014` |
| `resolve_param_arrays` merges dialog + `zone_params`, rejects out-of-spec | verified | `workflow.rs:47-131` (rejection at `:123-129`) |
| `params_json` does not capture zone edits | verified | `workflow.rs:388` serializes `req.params` only |
| Rayon phase 1 + one batched write phase 2 | verified | `workflow.rs:207-363`, `365-439` |
| Single `Arc<Mutex<Connection>>` | verified | `lib.rs:1642` |
| `ALTER TABLE ADD COLUMN IF NOT EXISTS` is the migration idiom | verified | `db.rs:73-75`, `88-89`, `136`, `177`; boot migrations in `project.rs:147-162` |
| No hashing anywhere; no hash crate | verified | `Cargo.toml` deps list |
| `update_computed_sample` point-updates with no `set_id` bump | verified | `db.rs:1944-1958` |
| No module code-version marker; crate version static `0.1.0` | verified | `Cargo.toml:3`; `build.rs` is 3 lines |

**Two corrections to the brief, both load-bearing:**

1. **`ArgKind::LogOut` is dead metadata in the Rust backend.** Constructed (`modules.rs:109-122`) and never read — consumed *only* by the frontend (`src/ui/moduleDialog.ts:214`, `src/ui/workflowDialog.ts:60,396,450`). Nothing today can drift-check it.
2. **Runtime keys demonstrably do NOT match spec names for derived-name modules.** `depth_shift` declares `log_out("CURVE_DS", …)` (`modules.rs:2297`) but emits `format!("{src}_DS")` where `src = ctx.o("__IN_CURVE")` (`modules.rs:2338-2339`, `2350`). Same shape in `splice` (`"SPLICED"` declared at `modules.rs:2368`, emits `{src}_SPL`) and `log_predict` (`"SYN"` declared at `modules.rs:2491`, emits `{src}_SYN`). The frontend papers over this by listing the placeholder as a selectable input name — a user picking `CURVE_DS` as a downstream input gets a curve that never exists.

**A second point-edit path the brief did not list:** `curve_edit::write_curve_inner`'s `CurveStore::Computed` branch (`curve_edit.rs:285-310`) rewrites every value of a computed curve **while preserving each row's `set_id`** and **without touching `computed_curves_archive`**. So current rows and archive rows for the same `set_id` can legitimately diverge. Any cache-validity scheme keyed on `set_id` alone is defeated by this path as well as by `update_computed_sample`.

## 1. The DAG

### 1.1 Nodes and edges

**Node** = one `ChainStep` at its position: `(index: usize, module: String)`. Position is the identity — the same module may appear twice.

**Producer names.** The DAG cannot use `ModuleSpec` `LogOut` names directly (§0 correction 2). Introduce a pure resolver:

```rust
// dag.rs
pub fn resolved_outputs(spec: &ModuleSpec, effective_opts: &BTreeMap<String,String>) -> Vec<String>
```

which walks `spec.args.filter(kind == LogOut)` and, for each, substitutes `{__IN_<arg>}` placeholders out of the same `__IN_*` opt map the runner already injects at `workflow.rs:194-196`. The template lives in the **currently unused `ArgSpec.default` field for `LogOut` args** — `log_out()` hardcodes `default: String::new()` (`modules.rs:114`), and no consumer reads it. So:

```rust
pub(crate) fn log_out_derived(name, desc, unit, template: &str) -> ArgSpec  // default = template
```

Applied to exactly three specs today: `depth_shift` → `"{__IN_CURVE}_DS"`, `splice` → `"{__IN_TOP_CURVE}_SPL"`, `log_predict` → `"{__IN_TARGET}_SYN"`. Zero migration, zero serde change, and the substitution reuses the module's own mechanism so agreement is *structural*, not coincidental.

**Edge** `i → j` (i < j) exists iff any of step *j*'s resolved input mnemonics is in `resolved_outputs(step_i)`. Input mnemonics come from the same rule the runner uses: `req.log_inputs.get(arg.name)` else `arg.default`, trimmed + uppercased (`workflow.rs:183-191`, `:255`). **Dialog `log_inputs` remapping changes edges directly.** The DAG must be built from the *resolved* mnemonics, never from spec defaults.

**Two edge sources easy to miss:**
- `opts["MASK"]` is an input (`workflow.rs:289`) — a real dependency edge.
- `computed_only` inputs (`workflow.rs:263-280`) resolve through `equations::fetch_computed_only_aligned` (`equations.rs:477-529`) which never falls back to RAW — mark these so diagnostics do not offer a RAW curve as satisfaction.

**Last-writer-wins.** If several earlier steps produce the same name, the edge points at the **latest** one — matching runtime DELETE+append semantics.

### 1.2 Guaranteeing spec ↔ runtime agreement

1. **Test harness (the real guarantee).** A `#[test]` in `dag.rs` iterating `modules::list_modules()`: build a synthetic `ModuleContext` from the spec (generalize `ssc.rs:484-504` into `ctx_from_spec`), run `run_module`, assert `outputs.keys() == resolved_outputs(spec, &opts)`. For modules that legitimately emit a subset under some option: degrade to `runtime ⊆ declared` + per-option union check. (~34 modules unverified for conditional omission — the harness enumerates them.)
2. **Debug assertion in the runner** after `modules::run_module` (`workflow.rs:321`), `#[cfg(debug_assertions)]`.
3. **Structural safety net.** The cache stores the **actual runtime output names**, never declared ones — a spec↔runtime disagreement can produce a wrong DAG *diagnostic* but can never corrupt a cached result.

### 1.3 What the DAG validates

Execution order stays exactly as the user wrote it. New read-only command `validate_workflow_chain(steps) -> Vec<ChainDiagnostic>`:

| code | severity | condition |
|---|---|---|
| `forward_reference` | Error | input produced only by a later step (today "works" by accident off a previous run's leftovers — silent staleness) |
| `unknown_module` | Error | not in `list_modules()` |
| `retired_module` | Error | `modules::retired_module()` is `Some` |
| `unsatisfied_required_input` | Error | required LogIn, no earlier producer, absent from catalog + standard six |
| `unsatisfied_optional_input` | Warning | same, `required == false` |
| `computed_only_from_raw` | Warning | computed_only input with no producer and no computed provenance |
| `duplicate_producer` | Warning | later step overwrites an earlier step's output |
| `mask_not_produced` | Warning | `opts["MASK"]` names an unproduced curve |
| `dead_output` | Info | output consumed by nothing downstream, not a chain terminal |

Catalog check uses project-wide `list_curve_catalog` — advisory only (not per-well). Nothing blocks a run in Phase A.

*Rejected:* auto-topological reordering (silently changes last-writer-wins semantics; invalidates saved workflow docs). *Rejected:* parallel step execution (single `Mutex<Connection>` makes the win small).

## 2. Cache key

### 2.1 Composition

One key per **(step, well)**. Domain-separated, length-prefixed BLAKE3 stream — never `format!`-concatenation:

```
H ‖= "sandibumi/compute-cache/v" ‖ CACHE_SCHEMA_VERSION
H ‖= tag("module") ‖ module_name
H ‖= tag("code")   ‖ CODE_IDENTITY
H ‖= tag("depth")  ‖ canon_f32_le(depth)
H ‖= tag("inputs") ‖ for each (arg_name, mnemonic) SORTED: arg ‖ mnemonic_upper ‖ canon_f32_le(post-mask values)
H ‖= tag("params") ‖ for each (name, Vec<f64>) SORTED: name ‖ canon_f64_le(RESOLVED array)
H ‖= tag("opts")   ‖ for each (k,v) SORTED from the EFFECTIVE opts map (incl. __IN_*)
H ‖= tag("mask")   ‖ mask_name_upper ‖ canon_f32_le(mask values)   // or ‖ 0 if none
```

Load-bearing notes:
- **Resolved param arrays** (from `resolve_param_arrays`) — what makes zone-table edits invalidate; `params_json` cannot.
- **Input values post-mask** (that is what the module sees); mask content *also* hashed separately (it is applied to outputs too, `workflow.rs:325-333`).
- **Sort HashMaps into BTreeMap + length-prefix before hashing** — randomized iteration order otherwise makes keys nondeterministic per process. Highest-probability implementation defect; gets its own test.
- **Float canonicalization**: all NaN payloads → `f32::NAN.to_bits()`, `-0.0` → `0.0`, then `to_le_bytes` (`bytemuck::cast_slice` for bulk). DuckDB FLOAT round-trip bit-exactness UNVERIFIED — pin with a test first.
- **Not in the key:** well_id (input content discriminates; separate column for locality), set_id/input_set (effect captured by resolved content), output_set name, timestamps.

### 2.2 Module code identity

**Recommendation: `build.rs` source hash over a declared file list** (`MATH_SOURCES`: modules.rs, ssc.rs, lrlc.rs, satheight.rs, rocktyping.rs, facies.rs, unconventional.rs, multimin.rs, workflow.rs, equations.rs), emitted as `cargo:rustc-env=SANDIBUMI_MODULE_CODE_HASH` + `rerun-if-changed`.

- Manual per-module REV: rejected as sole mechanism — forgotten bump = silent staleness into client reports (the worse failure).
- Coarse source hash: accepted — over-invalidation only bites developers; for a shipped binary the hash is constant.
- git commit hash: rejected (uncommitted working-tree changes = stale identity).
- Residual risk: a new module in a NEW file not added to `MATH_SOURCES` — mitigate with a test asserting `module_source_files()` ⊆ build-emitted list.

### 2.3 Hash function

**`blake3` (default features off).** Non-adversarial workload argues xxh3, but a collision here = silently wrong number in a client deliverable with no detection path; BLAKE3 buys designed collision resistance at ~3+ GB/s where hashing is not the bottleneck. UNVERIFIED: builds cleanly beside `duckdb bundled` under MSVC; `Hasher: Clone`.

## 3. Cache storage and the hit path

### 3.1 Schema

```sql
CREATE TABLE IF NOT EXISTS compute_cache (
    cache_key      VARCHAR NOT NULL,
    well_id        UUID    NOT NULL,
    module         VARCHAR NOT NULL,       -- diagnostics/pruning only, NOT key material
    set_id         UUID,
    output_curves  VARCHAR NOT NULL,       -- JSON array of ACTUAL runtime names, uppercased
    output_digest  VARCHAR NOT NULL,       -- BLAKE3 of aligned output frame — THE VALIDITY WITNESS
    n_depth        BIGINT  NOT NULL,
    created_at     TIMESTAMP NOT NULL DEFAULT now(),
    last_hit_at    TIMESTAMP,
    hits           BIGINT  NOT NULL DEFAULT 0,
    PRIMARY KEY (cache_key, well_id)
);
```

A PK is appropriate HERE (O(160k) rows, point lookups) and does not contradict the `computed_curves` PK-less rationale (millions of appended rows) — say so in the DDL comment.

### 3.2 The hit path is a proof, not a heuristic

`set_id`-existence checks are **insufficient** (two point-edit paths mutate values under an unchanged set_id). Instead:

```
HIT ⟺ row exists ∧ n_depth matches ∧ blake3(aligned CURRENT values of output_curves) == output_digest
```

Witness read uses the same alignment as the readers (`fetch_computed_curves_batch`, promote to pub(crate)). If the witness matches, current store already holds byte-for-byte what recompute-plus-write would leave — skipping is provably a no-op. On miss: compute + write versioned as today, then cache row in its **own** transaction after the write txn commits (DuckDB has no nested txns, `db.rs:534-537`); crash between the two loses only the cache row → safe miss.

### 3.3 Eviction

1. Referential: `DELETE FROM compute_cache WHERE set_id = ?` inside `delete_log_set`'s txn; same for well deletion.
2. Sweep at chain end + project open: delete rows whose set_id/well_id no longer exist.
3. Bounded: keep ≤8 rows per (well_id, module), LRU by `COALESCE(last_hit_at, created_at)`.

Note: `computed_curves_archive` unbounded growth is a pre-existing, orthogonal problem — flagged, not fixed here.

## 4. Invalidation — the witness makes it path-independent

Every write path (versioned/unversioned batch, equation runs, ml/multimin2/montecarlo, point edits via `update_computed_sample` AND `curve_edit` Computed branch, standard/generic edits, re-imports, zone_params edits, restore_log_set, code changes, curve promotion) lands as **MISS** through one of the two mechanisms: input-content key change (upstream) or witness failure (the cached outputs themselves). `delete_log_set` alone remains a correct HIT (values unchanged) and is cleaned referentially anyway.

**The point-edit case:** under set_id-based validity, a hand edit to a cached output would survive a re-run (hit → skip → edit persists) — cache hit ≢ recompute, silent. The value-witness fails on the changed bytes → MISS → recompute overwrites the edit → identical to uncached behavior. "Hash inputs, never outputs" is qualified: output content is not key *material* (circular) — it is a **validity witness stored beside the value, checked at read time**. Spell this out in cache.rs's module doc so nobody "simplifies" it away.

**One genuine regression + required fixes:**
1. On a hit nothing is written, so the run's set version has no archive rows for that step → `own_set` precedence in `fetch_curve_frame_from_set` (`equations.rs:779-788,806-808`) and `fetch_computed_only_aligned` (`:487-496`) breaks under `input_set`-scoped chains: a later step would read ARCHIVED upstream values instead of current. **Required fix:** replace `own_set_id: Option<&str>` with `OwnOutputs { set_id, extra: &HashSet<String> }`; chain accumulates produced-or-cached names per well.
2. Set versions now contain only what actually changed — more truthful, but a visible Sets-manager change; record in `log_sets.module` (e.g. `"workflow: … (2/3 cached)"`).
3. Eager per-well set allocation (`chain.rs:141-157`) → phantom empty versions on all-hit wells. **Recommended:** lazy allocation on first miss; a fully-cached re-run becomes a true no-op.

## 5. Where the code lives

New: `dag.rs` (~250 + tests, pure), `cache.rs` (~300 + tests). Seams S1-S7 in `run_workflow_module_into`:
- S1 after `workflow.rs:196`: step-invariant hash prefix (cloned per well); new `cache: CachePolicy` arg.
- S2 in the per-well closure AFTER mask applied to inputs (`:318`) and after computed_only re-resolution (`:263-280`) — hash the FINAL logs map; hash outside the DB lock.
- S3: `cache::lookup` (short lock); verified hit → skip run_module + output masking.
- S4 `:369-377`: exclude hit wells from set allocation and WellWrite.
- S5 after write txn commits: `cache::record_batch` in its own txn; digest from in-memory outputs.
- S6 result mapping: `ModuleRunResult.cache: CacheOutcome (miss|hit|disabled)`.
- S7 `:236-242,:270-277`: `OwnOutputs` threading.

`chain.rs`: own-outputs accumulation, per-step `StepReport {hits, misses, curves_reused}`, lazy set allocation, prune call. `equations.rs`: OwnOutputs params, promote `fetch_computed_curves_batch`, cache delete in `delete_log_set`. `db.rs`: DDL (+optional TABLE_SPECS row). `lib.rs`: `validate_workflow_chain`, `get_chain_graph`. Frontend: ipc.ts types, workflowDialog.ts diagnostics + "2 recomputed · 1 reused" status line.

Perf restructure (batched lookup/witness) only if the 100-well fixture shows lock contention — measure first.

## 6. Phase B hooks to include in Phase A

1. `ChainStep.step_id: Option<String>` (serde default) — stable step identity; retrofitting means migrating saved workflow docs.
2. `get_chain_graph` query — reactive engine needs `descendants(i)`.
3. Store a separate `config_digest` (module+code+params+opts) beside the composite key — answers "did parameters change?" without re-reading curves.
4. `cache::note_curve_mutation(conn, well_id, names)` — no-op shim now, wired at the ~10 mutation sites (update_computed_sample, curve_edit, both batch writers, restore_log_set, insert/update standard, zone_params writers, ingest.rs:319). Finding these sites IS the hard part of Phase B; Phase A correctness does not depend on them.

## 7. Tests (each pins a specific silent-wrong-number failure)

1. `key_is_stable_across_hashmap_insertion_order` — the highest-probability defect.
2. `float_canonicalization_and_duckdb_roundtrip` — NaN payloads/±0.0; write→read bit-identical after canonicalization. WRITE THIS FIRST.
3. **`rerunning_an_unchanged_chain_is_all_hits_and_byte_identical`** — the headline invariant (cache hit ≡ recompute).
4. `changing_a_dialog_param_misses_and_cascades` — transitive miss via content digests.
5. `changing_a_zone_param_misses` — the params_json-insufficiency fact.
6. `editing_an_input_curve_misses`.
7. **`point_editing_a_cached_output_forces_a_miss_and_restores_the_computed_value`** — the single most valuable test; twin for `curve_edit::edit_curve`.
8. `cached_step_still_shadows_the_input_set_for_later_steps` — the own_set regression.
9. `every_module_runtime_outputs_match_its_declared_outputs` — expect depth_shift/splice/log_predict to fail until templates land; that failure is the value. Add a determinism twin (run twice, bit-identical).
10. `dag_detects_forward_reference / duplicate_producer / derived_name_edge`.
11. `code_identity_participates_in_the_key`.
12. `prune_removes_rows_for_deleted_sets`.

Increments: **A1** dag.rs + templates + harness + validate command + UI diagnostics (no cache, no behavior change) → **A2** cache.rs + build.rs + schema + seams + OwnOutputs fix + tests → **A3** lazy sets, pruning, per-step UI → **A4** batched lookups if profiling demands.

Total ≈ 1,150 production lines + ~270 test lines, 2 new + 10 modified files.

## 8. Top risks (probability × undetectability)

1. HashMap iteration order in key composition (BTreeMap + length-prefix + test).
2. Any module input not in the key — purity contract verified structurally today (no SystemTime/Instant/external RNG; facies RNG is a seeded Param); state the contract in ModuleContext docs + determinism test.
3. New module file missing from MATH_SOURCES (test: module_source_files ⊆ emitted list).
4. DuckDB FLOAT round-trip not bit-exact (test 2 first; failure mode is loud 100%-miss, safe).
5. OwnOutputs regression if skipped — HARD REQUIREMENT (test 8).
6. Future "optimization" dropping the witness for set_id-existence — the §4 hazard returns (module doc + test 7).
7. Mask semantics (hash mask content explicitly).
8. computed_only re-resolution AFTER frame fetch — S2 must hash the final logs map.
9. output_curves must store runtime names, never spec placeholders (all-NaN spurious-match hazard).
10. Case handling: witness must uppercase; `update_computed_sample`/`curve_edit` exact-case matching is a pre-existing bug worth filing separately.

## 9. Unverified assumptions

1. Conditional output omission across the ~34 unread modules (harness enumerates).
2. blake3 build compatibility (MSVC + duckdb bundled); `Hasher: Clone`.
3. DuckDB FLOAT bit-exact round-trip.
4. Hit-path cost profile vs saved DELETE+append (measure on pipeline_blso_test fixture).
5. No frontend dependency on `ArgSpec.default` being empty for LogOut args (dist bundle unsearched).
6. Whether the SQL console/report paths are surprised by the new table in TABLE_SPECS.
