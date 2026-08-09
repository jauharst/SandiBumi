# LAS Native-Grid Retrieval and Import Performance Implementation Plan

> **Execution note:** Follow this plan in the current isolated feature branch. Keep the red-green-refactor order and run the listed gate after each group.

**Goal:** Preserve each imported LAS log set's native depth samples in the log viewer, make zoomed views request the visible interval at display-appropriate density, expose truthful finite-value statistics, and remove the avoidable parse/query/write costs that make import and tree expansion slow.

**Architecture:** Existing unqualified curve requests retain standard/computed/RAW resolution. A curve style may additionally name an imported `set_name`; that explicit request resolves one curve identity and returns its native `(depth, value)` samples without aligning to the well standard frame. WebGPU and composite SVG/PDF consume the same set-aware full-resolution frame resolver; the interactive panel reloads only the settled visible interval, and disposable min/max reductions carry the true source endpoints as structural extent points. Full catalog inspection computes finite statistics; the object tree uses a separate metadata-only inventory. LAS intake carries all parsed columns through its existing sanitization path, then commits well identity, standard projection, generic metadata and every native sample in one outer transaction, so the normal import path neither re-opens the LAS nor reports a partial delivery as success.

**Tech stack:** Rust, DuckDB, Tauri IPC, TypeScript, WebGPU canvas renderer, Node test runner through Vite SSR.

---

## Task 1: Pin the backend correctness contracts with failing tests

**Files:**
- Modify: `src-tauri/src/equations.rs`
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/ingest.rs`
- Modify: `src-tauri/src/parsers.rs`

1. Add a curve-retrieval test with a synthetic standard frame and a differently sampled imported set. Assert an explicit set request returns every native depth/value pair, while an unqualified request retains the current standard-frame semantics.
2. Extend that fixture with a requested depth interval and target point count. Assert filtering occurs before decimation, endpoints remain in interval, and the result is bounded without changing stored rows.
3. Add a generic-catalog test containing finite values plus `f32::NAN`. Assert total, finite, missing, min, max, and mean have unambiguous semantics.
4. Add a batched-write atomicity test: a bad second curve must leave neither curve partially replaced; a valid batch must persist both.
5. Add a deep LAS decimal-step regression (`STEP . 0.15240`) whose source tokens advance exactly by that decimal. Assert there is no regridding warning. Keep the existing real-mismatch assertion.
6. Add a parser/import regression proving a non-standard LAS curve survives the primary parse and reaches the generic store without a second parse path.
7. Add regressions for true top/base retention, set-qualified composite export, post-staging rollback, and rollback of a whole new-well delivery when the all-channel insert fails.
8. Run the narrow tests and confirm they fail for the intended missing APIs/semantics before production edits.

## Task 2: Add native-set, interval-aware curve retrieval

**Files:**
- Modify: `src-tauri/src/layout.rs`
- Modify: `src-tauri/src/equations.rs`
- Modify: `src-tauri/src/composite.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/ipc.ts`
- Test: `src-tauri/src/equations.rs`

1. Add backward-compatible optional `set_name` to `CurveStyle` in Rust and TypeScript.
2. Replace mnemonic-only track requests with `{ curve_name, set_name? }`, while preserving layout compatibility for styles without a set.
3. For an explicit imported set, resolve the exact `curve_meta` identity deterministically and query native samples ordered by depth. Do not align onto `standard_curves`.
4. Apply optional depth bounds before decimation. Keep the full-well initial request so the renderer establishes the well extent.
5. Give returned series a stable request key so equal mnemonics from different sets can coexist.
6. Reuse the same full-resolution set-aware frames for SVG/PDF composite output, including display-only crossover interpolation between different native grids.
7. Preserve exact whole-well top/base through initial min/max decimation even when neither endpoint is a value extreme.
8. Run the native-grid/export tests until green, then run all `equations` and `composite` tests.

## Task 3: Separate catalog statistics from the object-tree inventory

**Files:**
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/ipc.ts`
- Modify: `src/ui/inspectorPanel.ts`
- Modify: `src/ui/objectTree.ts`
- Test: `src-tauri/src/db.rs`

1. Extend the full generic catalog query with total, finite, missing, min, max, and mean. Exclude NaN/infinite values from numeric statistics without pretending they are absent rows.
2. Add a metadata-only curve inventory endpoint that never joins or scans `curve_samples`.
3. Use the full query only for Database Inspector and display finite/total counts with the computed statistics.
4. Use the inventory in Wells > Sets. Keep its cache through pure collapse/expand rerenders; invalidate only when data or metadata changes.
5. Run the statistics tests, database tests, and frontend tests.

## Task 4: Parse each LAS once and batch generic writes

**Files:**
- Modify: `src-tauri/src/parsers.rs`
- Modify: `src-tauri/src/ingest.rs`
- Modify: `src-tauri/src/db.rs`
- Test: `src-tauri/src/parsers.rs`
- Test: `src-tauri/src/ingest.rs`
- Test: `src-tauri/src/db.rs`

1. Carry all non-depth curve columns, mnemonics, and units in the primary LAS parse result.
2. Keep duplicate-depth resolution, depth reversal/sorting, null handling, and row sanitation synchronized across standard aliases and all retained curves.
3. Refactor the normal importer to consume those retained curves instead of reopening and reparsing the file. Preserve the public all-curves import helper for callers that intentionally start there.
4. Process multi-file imports in bounded parallel chunks so retaining all curves does not turn the full path list into a memory spike.
5. Add an inner multi-curve write that validates the complete batch and uses one DuckDB transaction/appender. Keep the single-curve function as a compatibility wrapper.
6. Wrap the new well row, project-unit adoption, standard projection, generic metadata and every sample in one outer transaction; retain the narrower atomic transaction for attach/replacement.
7. Run parser, ingest, and database tests until green.

## Task 5: Compare declared LAS sampling from source decimals

**Files:**
- Modify: `src-tauri/src/parsers.rs`
- Modify: `src-tauri/src/ingest.rs`
- Test: `src-tauri/src/ingest.rs`

1. Capture the actual source depth-step relationship before reducing depth values to `f32`.
2. Compare declared and observed steps using exact decimal semantics, including signed/scientific tokens, rather than exact equality between rounded floats.
3. Report a warning only for a true source mismatch; do not weaken detection with a domain-invented tolerance.
4. Run the matching and mismatching STEP regressions.

## Task 6: Wire set selection and viewport reload into the viewer

**Files:**
- Modify: `src/ui/layoutPropsDialog.ts`
- Modify: `src/ui/logViewPanel.ts`
- Modify: `src/LogCanvasRenderer.ts`
- Modify: `src/ipc.ts`
- Modify: `tools/frontend-acceptance.test.mjs`

1. Add a small pure request-key helper and test same-mnemonic/different-set uniqueness through Vite SSR.
2. Present imported set identity beside mnemonic in Layout Properties; blank set retains the current resolved curve behavior, and only sets carrying that mnemonic are selectable.
3. Build track requests from layout styles and use the stable key consistently for rendering, hiding, cursor readout, and crossover lookup.
4. Add a renderer series-replacement path that rebuilds geometry without resetting the full-well depth extent or current view.
5. On a settled pan/zoom, debounce and generation-guard a visible-depth request. Ignore stale completions and deduplicate equivalent requests.
6. Keep initial full-well loading, errors, GPU refusal, layout changes, well switches, and panel disposal correct.
7. Run frontend tests and `npx tsc --noEmit`.

## Task 7: Record the shipped contracts and field checks

**Files:**
- Modify: `docs/record_data_tools.md`
- Modify: `ROADMAP.md`
- Modify: `REVIEW.md`
- Modify: `CLAUDE.md` only if its indexed binding summary needs a new one-line contract

1. Record that explicit imported-set display uses native depths and display decimation never modifies storage.
2. Record the catalog/inventory split and finite-statistic semantics.
3. Record the single-parse, bounded-memory, atomic batch-import path and exact-decimal STEP comparison.
4. Add pending field checks using synthetic/source-neutral language: native sample count, visual detail after zoom, inspector statistics, tree latency, and import timing.

## Task 8: Verify, review, and publish

1. Run the targeted Rust regressions and `npm run test:frontend`.
2. Run `npx tsc --noEmit` and `cargo check`.
3. Start the supported local app path, run focused browser verification of curve-set selection, zoom reload, statistics, and tree collapse/expand, then stop only the process started for this check.
4. Run `tools\check.ps1` and inspect `git diff --check` plus the scoped diff.
5. Request a read-only final review and close any verified silent-wrongness finding with a focused regression before integration.
6. Apply the verification-before-completion and branch-finishing procedures.
7. Commit only the intended files, push `codex/sampling_prob`, open the required pull request, and stop for review.
