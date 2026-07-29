# SandiBumi — Target-State Architecture

**Version 0.1 of this document · 2026-07-29 · derived from `docs/PRD.md` (reviewed 2026-07-29)
and `docs/V1_SCOPE.md`**

Produced by Prompt 2 of `docs/product_definition_prompt.md`. **Boundary, stated first:** this is
*not* a map of the system as built — that is `ARCHITECTURE.md`, commissioned by
`stewardship_prompt.md` Prompt 2 Deliverable 1 and **not yet written**. This document answers the
opposite question: given the accepted PRD (licensed product, desktop now, multi-user someday),
what must be TRUE architecturally for the product to be sellable and to keep its options open.
When `ARCHITECTURE.md` exists, it describes; this file constrains. Where this file needed a fact
about the current system, it was measured from the code on 2026-07-29 and cited — those citations
will drift and the *requirements* will not.

No code changed in producing this document. Everything actionable is listed in §10 as proposed
`ROADMAP.md` entries for Jauhar to adopt.

---

## 1. Deployment model — today, and the named future

**Today (v1.0 posture, per PRD §7.6):** single-user Windows desktop. One project is one `.duckdb`
file on the interpreter's own disk. One writer. No server, no network egress (PRD §7.5, verified),
no shared anything. This is a *product boundary*, not a limitation to apologise for: it is what
makes the zero-egress data-security answer possible, and that answer is the strongest commercial
asset the product has.

**Named future:** a shared multi-user backend — several interpreters on one asset team working one
field study. Not built, not scheduled, and **not foreclosed**. The rest of this document exists to
make that last word true cheaply: not by building server code early (that would be waste), but by
keeping one boundary clean so that the future is an *addition*, not a rewrite.

**Explicitly out, even in the future state, per PRD §5.4:** becoming the corporate system of
record. The multi-user future is "a team shares a project", not "the company's master log
database". The moment access control, retention policy and audit-for-compliance enter the
requirements, that is a different product being asked for, and the answer is no.

---

## 2. What is single-user-shaped, and how deeply

The inventory Prompt 2 demands, measured against the code. Classification: **FUNDAMENTAL** =
rewriting it means rewriting the product; **LOCAL** = an assumption in one layer, liftable without
touching the physics.

| # | Assumption | Where | Class | Why |
|---|---|---|---|---|
| 1 | One writer: `DbState(Arc<Mutex<Connection>>)` | `lib.rs` | **FUNDAMENTAL** | Every write path in the app serialises through this mutex. DuckDB-embedded is single-writer by design; CLAUDE.md states it as fundamental and it is. A multi-user backend does not "fix" this — it *relocates* it: one server process owns the one writer, and clients queue. |
| 2 | One project = one `.duckdb` file, opened at launch | `project.rs`, `db.rs` | **FUNDAMENTAL** | The unit of collaboration, backup, handover and versioning is the file. This is also the *right* boundary for the licensed product (PRD: months of work must remain one portable artefact). |
| 3 | The write discipline: delete-then-append on a PK-less `computed_curves`, uniqueness upheld by convention | `equations::write_computed_curves_batch`, `db.rs` | **FUNDAMENTAL** (coupled to #1) | Correct *only because* exactly one writer exists. Two concurrent writers would corrupt uniqueness silently — the worst failure class this product knows. Any future server tier inherits the discipline by inheriting the single writer, never by adding upserts (the prohibition in CLAUDE.md rule set stands in every future). |
| 4 | `appState` globals: `selectedWell`, `selectedInterval`, `hoverDepth`, `pinnedWellId`, `activeWellGroup`, `dataVersion`, `themeVersion`, `brushedDepths` | `state.ts` | **LOCAL** | These are *per-screen* concepts, not per-project ones. In any multi-user world each client keeps its own; nothing about them needs to be shared. They are single-user-shaped only in the harmless sense that there is one screen. |
| 5 | The undo stack | `undo.ts` | **LOCAL, with one flag** | Undo is per-user by nature and stays client-side. The flag: undo *mutates shared data* (curve edits, tops edits). Under a shared backend, "undo my edit" after someone else's edit lands is a real design problem (operational-transform territory). Resolution is deferred, not free — noted in §9 as part of the migration trigger's cost. |
| 6 | Job registries: `chain::new_registry()`, `jobs::new_registry()` — in-process, pollable | `chain.rs`, `jobs.rs` | **LOCAL** | Already shaped like a job queue (id, progress, cancel). A server tier replaces the in-process map with a server-side one; the polling contract survives. |
| 7 | Well-group scoping enforced by the frontend only (`filterByActiveGroup`), backend enforces nothing | every batch dialog | **LOCAL, and a known soft spot** | Fine for one user (the QC sweep in `qc_audit_prompt_template.md` §3.3 exists because even one user's dialogs drift). Under multi-user this MUST move server-side — a frontend-only scope contract across clients is not a contract. |
| 8 | Sessions, layouts, plot templates, report templates in the `documents` table | `workspace.ts`, `plotCommon.ts` | **LOCAL** | Per-user preferences stored in the shared project file. Under multi-user these need a per-user namespace — a schema addition, not a redesign. |
| 9 | Versioned log sets (RAW/EDIT/FINAL) with provenance | `log_sets` | **Neither — an asset** | The one part of the data model that is *already* multi-user-shaped: versioned, provenance-carrying outputs are exactly what concurrent interpretation needs. Protect it. |
| 10 | Python subprocess per call, stateless | `python_engine.rs`, `dlis.rs`, `ml.rs` | **LOCAL** | Stateless request/response; runs identically behind a server. |

**The honest summary a client could be given today:** SandiBumi's compute and data model could sit
behind a server tier with weeks-scale effort concentrated in three places (the writer relocation
#1/#3, server-side scoping #7, per-user documents #8) — but concurrent *editing* (#5) is unsolved
and would be the actual cost of true multi-user. "Your team of six can share a project" is
therefore **not yet a yes**, and per PRD §6.2's rule, it must not be claimed until it is.

---

## 3. The one rule that keeps the option open

> **The compute layer takes data in and returns data out. It never reads the database, the UI, or
> "the currently open project".**

If this holds, a server tier is *a new caller of existing functions*. If it rots, multi-user is a
rewrite. Prompt 2 requires assessing whether it holds **today**, with named exceptions. Measured
2026-07-29:

**It holds, structurally, for the module suite.** The entire deterministic library runs through
one signature:

```rust
// modules.rs:232 — the seam the whole future hangs on
pub fn run_module(name: &str, ctx: &ModuleContext) -> Result<ModuleOutputs, String>

// modules.rs:127 — the context is pure data: no Connection, no state handle, no well id
pub struct ModuleContext {
    pub n: usize,
    pub logs: HashMap<String, Vec<f32>>,
    pub params: HashMap<String, Vec<f64>>,
    pub opts: HashMap<String, String>,
}
```

Grep proof: `modules.rs`, `ssc.rs`, `lrlc.rs`, `satheight.rs`, `rocktyping.rs`, `facies.rs`,
`unconventional.rs` contain **zero** references to `Connection`. Monte Carlo is the existence
proof that the seam is real: `montecarlo.rs` runs thousands of realizations *entirely in memory*
by calling `run_module` on synthesised contexts, touching the DB only to fetch inputs once and
never to write. That capability exists *because* the rule holds.

**Named exceptions, so they are decisions rather than surprises:**

1. **`run_multimin` (multimin2.rs:990) couples fetch → solve → write in one function** taking
   `&Mutex<Connection>`. Its inner solver (`solve_bounded_lsq`, the per-sample loop) is pure
   array math, and its 19 `Connection` mentions are almost all `#[cfg(test)]` (production has
   exactly two, lines 714/991 — the wrappers). So the coupling is shallow: the pure core exists,
   the entry point just doesn't expose it. Acceptable today; the future refactor is mechanical
   (split fetch/solve/write) and should be done *when a second caller appears*, not before.
2. **Storage access has one front door** — `equations::fetch_curve_frame` is the single function
   that resolves curves by precedence (standard → computed → generic store). This is the good
   kind of exception: compute callers depend on one fetch seam, so relocating storage means
   re-implementing one function, not thirty.
3. **The Rust layer knows no UI concepts at all** — enforced structurally by the Tauri process
   split, not by discipline. The webview cannot leak `appState` into a solver because the solver
   is in another process. This is the strongest boundary in the codebase and it was free.

**The enforcement gap (per stewardship's "enforcement beats intention"):** nothing today *stops* a
future module from taking `&Mutex<Connection>`. The rule lives in this document and in review.
Cheapest structural enforcement, proposed in §10: move the compute files into a workspace crate
(`sandibumi-core`) whose `Cargo.toml` does not depend on `duckdb`. Then violating the rule is a
compile error. Not urgent; do it the first time the rule is *almost* violated, or when SegaraBumi
integration (§7) creates a second consumer anyway.

---

## 4. The licensing and activation surface

Per PRD §8 nothing is implemented, and per V1_SCOPE §7 the commercial decisions are Jauhar's. What
this document owns is the **boundary any scheme must respect**, decided now so a future
implementation inherits constraints instead of inventing them:

1. **Licensing lives in the shell, never in compute or data.** A licence check may gate *launching
   the app* or *enabling a feature surface*. It must never sit inside `run_module`, a parser, an
   exporter, or any write path — a licensing bug that corrupts or withholds a user's own project
   data converts a commercial mechanism into a data-integrity incident.
2. **The user's data is never hostage.** An expired licence must still allow: opening the project
   read-only, exporting LAS, exporting existing reports. The interpretation the client paid to
   produce is theirs; what lapses is the ability to produce *new* work. (This is both the decent
   position and the one an operator's procurement will require in writing.)
3. **Fully offline forever.** PRD §7.4/§7.5: the zero-egress posture is a headline asset, and
   online activation would destroy it. Whatever the scheme is, it must be satisfiable on an
   air-gapped machine — a signed offline key file meets this; a licence server does not. Any
   phone-home behaviour, including "just metrics", re-opens §5.
4. **No dependency on machine identity that breaks on ordinary IT events.** Reimaging, hostname
   changes and hardware refresh are routine in corporate estates; a key that dies with them
   becomes a support obligation R10 has no answer for.

---

## 5. The data boundary, stated as a promise

**The promise, one sentence:** *client well data never leaves the machine — not to us, not to
anyone, not partially, not "anonymised".*

**Verified true today** (PRD §7.5, three independent checks: no HTTP client crate compiled in; no
`fetch`/XHR/WebSocket in the frontend; no updater configured; the one granted-but-unused OS-open
capability was removed 2026-07-29). What must *stay* true, as standing constraints:

- **No network dependency may be added to `src-tauri/Cargo.toml` or `package.json`** without this
  section being revisited in the same change. The PR that adds `reqwest` is the PR that edits this
  file, or it is wrong.
- **`ROADMAP.md` §C4's "auto-update" item is in direct collision** with this promise and is
  flagged for disposition in `V1_SCOPE.md` §4: manual, user-initiated update delivery preserves
  the promise; a background updater trades it away. That trade is allowed only as an explicit §8
  commercial decision, never as a convenience.
- **Future telemetry, if ever proposed, is opt-in and structurally separated** — its own crate or
  feature flag, off by default, so that "the build without telemetry" is a compile-time fact a
  client's security review can verify, not a runtime setting they must trust.
- **Two residues stay on the risk register, not swept under the promise:** the project file is
  unencrypted at rest (PRD R7 — needs a key-management design before any code), and the Python
  subprocess's temp-file behaviour is unaudited (PRD §10.6). The promise as worded above is about
  egress and remains true; a *stolen laptop* is R7's problem and the two must not be conflated in
  customer conversation.

---

## 6. Extension points that carry the product promise

The mechanisms that make growth cheap — and what breaks each one's promise if mishandled:

| Extension point | The promise it carries | What breaks it |
|---|---|---|
| **The module manifest** (`modules.rs::list_modules`, 42 entries; auto-generated parameter dialogs) | New petrophysics = a Rust fn + a manifest entry, zero UI code. This is why 42 methods exist at solo-developer cost — the PRD's breadth claim rides on it. | A bespoke panel for a method the manifest could express. Each one permanently widens the maintenance surface — now a *commercial* cost (support, verification matrix rows), not just untidiness. **Named existing exceptions, so they don't become precedent by accident:** the unconventional ΔlogR/Langmuir panel, SandiMin's dialog, the ML dialog — each justified by genuinely non-manifest interaction (overlay picking, tabbed component matrices, train/apply splits). The bar for a fourth exception is "the manifest *cannot* express it", not "a custom panel would be nicer". |
| **The generic curve store** (`curve_meta`/`curve_samples`, family aliasing, `fetch_curve_frame` precedence) | Any mnemonic from any vintage feeds any module — the multi-vintage claim of PRD §2. | Modules that hardcode the fixed six; render paths that bypass it (`get_track_data` still reads only `standard_curves` — a known, deliberate gap carried in ROADMAP §B4, acceptable until the log view needs generic curves). |
| **Versioned log sets + provenance** (§2 row 9) | "Where did this number come from?" — the §3.1 user's defining question, and the audit-trail down-payment for Phase 11. | Any writer that skips the `log_sets` row (the `skip_version` escape exists for legitimate overwrite-in-place cases; each new use is a provenance hole and needs the same justification bar as a manifest bypass). |
| **Equations (Rhai/Python) and plot/layout/report templates** | The field-study escape hatch — what the interpreter does that the product didn't anticipate. | Scope creep toward §5.6: these are extension points *for the user's project*, not a plugin platform. The Phase-12 "user-defined Python modules" roadmap item is the sanctioned evolution; anything fancier awaits demand. |
| **The chart-overlay pipeline** (`tools/chartdig` → generated `chartOverlays.ts`) | Chart-grade QC overlays, regenerable, never hand-edited. | Hand edits to generated files; and the pipeline's *input* is the R1 provenance question — `IP_PROVENANCE.md` §2.1 governs, this table just points at it. |

One standing duplication to hold the line on (it has bitten before): `FACIES_PALETTE` exists in
both `plotCanvas.ts` and `composite.rs` with a keep-in-sync comment. Under a licensed product,
sync-by-comment contracts like this one belong on the §10 list to be made structural (a build-time
generation or a shared constants file) the next time either side changes.

---

## 7. The SegaraBumi seam

`stewardship_prompt.md` Prompt 5 already owns the seam checklist (contract file, no shared writes,
acyclic dependency, version skew, Tier-2 labelling) — **not duplicated here**. What the accepted
PRD *adds* to it:

1. **The automation boundary is now a product decision, not a design preference.** PRD §4.9/§5.7
   (Jauhar, 2026-07-29): auto-zonation, parameter identification, the decision playbook and the
   knowledge base are SegaraBumi. Consequence for the seam: anything SegaraBumi sends back to
   SandiBumi is *a suggestion with provenance* — it enters as labelled, reviewable input (a
   proposed zone set, a suggested parameter with its source), never as a silently-applied result.
   This is stewardship Prompt 5's Tier-2 rule promoted to the product's cardinal rule: a machine
   suggestion presented as an interpreter's decision is a degraded result presented as a clean one.
2. **A licensed SandiBumi cannot ship a dependency on an unversioned SegaraBumi.** Until
   SegaraBumi has its charter-frozen `CONTRACT.md` (stewardship Prompt 1 Step 2), SandiBumi
   releases must not *require* it — integration ships as optional, degrade-gracefully, the same
   posture as Python (CLAUDE.md rule 7 is the house pattern: a missing collaborator degrades a
   feature, never the app).
3. **The compute-purity rule (§3) is what makes the seam cheap.** SegaraBumi consuming SandiBumi's
   methods means calling `run_module`-shaped functions — which is exactly the second consumer that
   justifies the `sandibumi-core` crate split proposed in §3/§10. The two futures (server tier,
   SegaraBumi integration) want the same refactor; do it once, when the first of them lands.

---

## 8. What we are deliberately not doing

Standing decisions, each of which *looks* like a defect to a newcomer and is not. (These are the
ADR seeds stewardship Prompt 2 Deliverable 2 will formalise; listed here because the target state
depends on them staying decided.)

- **The GPU is render-only.** Compute stays on the CPU (rayon). The WebGPU surface exists for the
  log views; moving physics onto it would couple correctness to driver variance across client
  machines — unacceptable for a product whose output is reserves numbers.
- **Module runs are not undoable; they are versioned.** Undo is for edits; runs are reproducible
  from provenance. Collapsing the two would make the undo stack a second, worse provenance system.
- **`computed_curves` stays PK-less; no upsert path may ever assume a key.** Uniqueness is the
  write discipline's job (§2 row 3). The 2.4× write-path win at field scale *is* this decision.
- **DuckDB stays embedded and single-writer.** Scaling is fewer-bigger-writes, not more writers —
  proven at 100 wells, and the strategy for 2000 (V1_SCOPE Q5 owns demonstrating it).
- **DLIS parsing stays a subprocess** (dlisio), never a native parser, until the bridge itself is
  the measured limit. A missing dependency degrades one importer, never the app.
- **No auto-update, no telemetry, no network** — §5, restated once because every future
  convenience argument will arrive at one of these three doors.
- **No corporate-data-product features** — access control, retention, compliance audit (§1).

---

## 9. Migration triggers — written now, while nobody is under pressure

The multi-user future starts when one of these observable signals fires, and not before:

| Trigger | Signal | First response (not "build the server") |
|---|---|---|
| **T1 — A deal requires it** | A named prospect makes shared projects a condition of purchase | Scope *their actual need* against §2: most "we need multi-user" requests are satisfied by file handover + versioned log sets (row 9) + a merge tool (ROADMAP C3 already plans "merge wells from another project file"). Build merge before building a server. |
| **T2 — Real concurrent contention** | An asset team is passing one `.duckdb` around and losing work to overwrites (reported, not hypothesised) | Same first response: the merge tool plus a file-locking convention costs 1% of a server tier and may retire the pain. |
| **T3 — SegaraBumi integration lands** | The seam (§7) creates the second consumer of the compute layer | Do the `sandibumi-core` crate split (§3) as part of that work — it is the shared prerequisite of every deeper future. |
| **T4 — The 2000-well fixture fails interactively** | V1_SCOPE Q5's stress work shows the UI cannot stay responsive at target scale on client hardware | This is a *performance* trigger, not a multi-user one — the answer is the B4 responsiveness backlog (lazy catalog, decimation cache), not architecture change. Named here so scale pain is not misread as a server requirement. |

**The standing rule between triggers:** every increment keeps §3 true and §5 true. Those two
sections are the entire premium being paid for the future; everything else in this document is
consequence.

---

## 10. Proposed ROADMAP entries (Jauhar adopts; nothing scheduled by this document)

1. **`sandibumi-core` crate split** — compute files into a workspace crate with no `duckdb`
   dependency; §3's rule becomes a compile error. Trigger-gated (T3, or first near-violation).
2. **Split `run_multimin` into fetch / solve / write** — mechanical, done when a second caller
   appears (same trigger).
3. **Server-side well-group scoping** — pre-requisite of any multi-user work (§2 row 7); until
   then, keep the per-dialog `filterByActiveGroup` QC sweep in the audit rotation.
4. **Make the `FACIES_PALETTE` sync structural** (§6) — next time either copy changes.
5. **Project merge tool** (already ROADMAP C3) — promoted in priority: it is the cheap answer to
   T1/T2 and should precede any server-tier conversation.
6. **Key-management design note for encryption at rest** (PRD R7) — a design document, not code;
   unblocks the enterprise-sale answer.
7. **Python subprocess temp-file audit** (PRD §10.6) — closes the last caveat on §5's promise.

---

## Acceptance

Accepted when Jauhar confirms: the FUNDAMENTAL/LOCAL classifications in §2 (especially that
"team of six shares a project" is **not yet a yes**), the §3 rule as standing law with its named
exceptions, the §4 licensing boundary (data never hostage; offline forever), and the §9 triggers.
Disagreement on any classification is a redirect, per the collaboration protocol — the
classifications are the document.
