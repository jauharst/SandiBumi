# SandiBumi engineering-craft review prompt (Track F)

A reusable prompt for reviewing SandiBumi's **code quality** — frontend architecture, Rust idiom
and hot paths, UX/theming, build health, and panel lifecycle. Five passes, F1–F5, one per
session.

**Scope note (2026-07-24):** an earlier draft of this file also carried Tracks A–D, a
skill-driven *domain* sweep that would have checked the physics against the 45 installed
petrophysics skills. Jauhar dropped it — not planned work. Only Track F survives. The A–D
material is recoverable from commit `85e7d69` if it is ever wanted.

## The authority problem, stated honestly

There is **no skill to load for these passes.** Verified 2026-07-24: all 48 skills on this
machine are geoscience (45 in `~/.claude/skills`, plus `petropy`, `well-log-evaluation` and a
duplicate `geolog-loglan` in the plugin marketplace cache). No Anthropic frontend, UX or
code-optimization skill is installed, and none is in the marketplace cache. Authoring house
engineering skills was considered and **declined** — the pipeline that built the petro skills
took a petrophysics PDF library as input, and there is no engineering source material on disk to
distil, so it would be a sourcing project rather than a build step.

So Track F substitutes three other authorities:

1. **`/code-review ultra`** — the multi-agent branch review. **Jauhar triggers it; Claude cannot.**
   The right instrument for diff-scoped craft. Run it BEFORE F1/F2 so a pass does not re-find
   what it already found.
2. **The `code-simplifier` subagent** — clarity and maintainability on a single named file. Not a
   whole-repo sweep.
3. **The checklists in §2**, which encode SandiBumi's own accumulated conventions.

**Why Track F exists at all, given ultra:** ultra reviews a *diff*. It cannot see app-wide
invariants — that all ~30 panels honour the theme contract, that every `DomPanel` unsubscribes
what it subscribed, that the main bundle has not grown. Those are whole-app properties.
**F1/F2 are ultra's territory; F3/F4/F5 are the ones ultra structurally cannot do.**

**Relation to `docs/qc_audit_prompt_template.md`:** that prompt audits ONE TOOL end to end
(DB → backend → domain → frontend → UX → integration). Track F sweeps ONE PROPERTY across the
WHOLE APP. Different axis, both useful.

`artifact-design` / `artifact-capabilities` are the only Anthropic skills present. They build
published web pages and are **not** review authorities.

---

## 1. The master prompt (copy, fill in `{{...}}`, run)

```
Run engineering review pass "{{PASS_ID}} — {{PASS_NAME}}" over SandiBumi at D:\XX. Arshilla.

Checklist for this pass: {{CHECKLIST}}
Surfaces in scope: {{SURFACES}}

## Step 0 — Restate the checklist as an expectation list

There is no skill to load for this pass. Instead, restate the checklist above as a numbered
EXPECTATION LIST — what a healthy codebase would look like on each point — BEFORE reading any
code. Then read the code. This ordering matters: read the implementation first and you will
rationalise whatever it does.

## Step 1 — Don't re-report what is already known

Grep REVIEW.md, ROADMAP.md, docs/playbook_build_progress.md and AUDIT-*.md for the files and
conventions in scope. REVIEW.md is at Round 57+. A recorded deferral is an explicit decision by
Jauhar, not an oversight. If ultra has already run on this branch, read its output first.

## Step 2 — Respect the house conventions (these are NOT defects)

- Module/equation RUNS are deliberately not undoable — they are versioned instead. Only UI/data
  edits push an undo entry.
- computed_curves is deliberately PK-less; uniqueness is a write-discipline contract enforced by
  delete-then-append inside db::with_txn.
- The backend deliberately does not enforce well-group scoping; frontend dialogs must call
  filterByActiveGroup. A dialog missing it IS a bug; the backend not enforcing it is not.
- Heavy panels are deliberately lazy chunks. Main index bundle baseline: 1,125.01 kB.
- The Canvas-2D domain plots and the Vega panel are deliberately complementary, not duplicates.
- linear_dw stays the default.

## Step 3 — Classify

- CODE-BUG / PERF / TYPE-MISMATCH / DEAD-CODE / INCONSISTENCY / THEME-VIOLATION / A11Y /
  BUILD-HEALTH / MAINTAINABILITY / SECURITY.
- HOUSE-DEPARTURE — a deliberate convention that is not written down anywhere. The finding is
  the missing documentation, not the code.
- NO-AUTHORITY — a judgement call with no convention behind it. Name it so a convention can be
  decided rather than re-litigated every time.

## Step 4 — Quality bar

Report a defect only if you can name a CONCRETE failure or cost: an input that panics, a state
that renders wrong, a measurable size/time cost, or a specific maintenance hazard with a named
future change that would trip on it. "Best practice says X" with no concrete cost for THIS app is
not a finding. Big files are not automatically a problem — only report one if you can name the
seam AND the pain it causes today. Cite exact file:line and quote the code you are judging.
Consistency findings need counts to be credible ("9 dialogs do X, this one does Y").

## Step 5 — Refute yourself, then judge value

Two questions per finding, kept separate:
(a) DOES IT EXIST? Adversarially try to refute it — is the precondition guaranteed upstream, is
    the code in a test module, does a helper already handle it, is the path reachable from the UI
    at all, did you misread the control flow? Default to refuted if you cannot positively confirm.
(b) IS IT WORTH FIXING? Would the fix churn a large working area for no user-visible gain? Data-
    honesty issues are always fix-now: this app's cardinal rule is that a degraded or failed
    result must never be presented as a clean success.

## Step 6 — Report

Write docs/review_sweep/{{PASS_ID}}.md: the expectation list with each line marked, then one
block per surviving finding — Title / Class / Severity / Effort / Where (file:line) / Evidence
(quoted) / Failure scenario / Suggested fix / How you tried to refute it and why that failed /
Worth (fix-now, backlog, drop). Close with a plain verdict on the dimension. If it is clean, say
so — do not manufacture findings.

## Rules

- Review only. Do not edit code in this pass. Fixes land afterward as separate serial increments
  in the main working tree, each with tsc + cargo test + a REVIEW.md entry.
- If context runs short, stop at a clean boundary and state the resume point.
```

---

## 2. The five passes

| Pass | Checklist | Surfaces | Instrument |
|---|---|---|---|
| **F1** Frontend architecture | file size vs responsibility (name the seam and today's pain, not just the line count); duplication across dialogs that should be a shared helper — and whether the copies have already DRIFTED; `ipc.ts` types actually matching the Rust structs they mirror (a silent mismatch is `undefined` at runtime with no compile error); `any`/non-null-assertion leakage; dead exports; consistent async/error shape; a generation counter on every async reload | `src/ui/*.ts`, `src/ipc.ts`, `src/state.ts`, `src/workspace.ts` | `/code-review ultra` first; then `code-simplifier` per named file |
| **F2** Rust idiom & hot paths | unwrap/expect/indexing/div-by-zero reachable from user data (separate production paths from `#[cfg(test)]`); allocation and cloning inside per-sample loops at corpus scale; `Result<T,String>` at the IPC edge vs typed inside; error strings that actually tell the user what to do; batch isolation (one well's failure must not abort the run); **silently swallowed errors that yield a successful-looking empty result**; rayon compute / single batched DB-write separation; `Mutex<Connection>` hold time; mid-run cancellability; dead code and orphaned commands | `src-tauri/src/*.rs` | `/code-review ultra` first; then `code-simplifier` per named file |
| **F3** UX & theming sweep | **app-wide**: every panel against the 15-var CSS contract across all 6 palettes (dark, light, Pertamina, Halliburton, Schlumberger, LAPI-ITB) — classify each raw hex as legitimate (colormap, domain colour, PDF/SVG export context) or violation; `themeVersion` subscription on every panel that caches colours; `dataVersion` on every panel showing well data; dialog convention outliers (Run-at-top, multi-column lists, button labels); empty / no-well / no-curve / slow / failed states; **all-NaN presented as success**; a11y beyond the 9D base — modal focus trap and restore, Escape, focus rings, icon-only buttons, colour-as-only-meaning in the QC scorecards | all `src/ui/*`, `src/styles.css` | this prompt only — ultra cannot see it |
| **F4** Build & bundle health | main `index` bundle vs the **1,125.01 kB** baseline; every heavy panel still a lazy chunk; anything eagerly imported that defeats it; dead dependencies, config and assets; open `npm audit` advisories assessed as a real threat model for a Tauri desktop app rather than repeated from the advisory text; tsconfig/clippy strictness flags that would catch real bugs here versus create busywork | `vite.config.ts`, `package.json`, `tsconfig.json`, `src-tauri/tauri.conf.json`, build output | this prompt only — ultra cannot see it |
| **F5** Lifecycle & leaks | **app-wide**: every `DomPanel` unsubscribes what it subscribed (dispose symmetry); `dataVersion`/`themeVersion` subscription pairs; listener accumulation across close→reopen; `filterByActiveGroup` present in every batch dialog (the backend enforces no group scoping at all) | all `src/ui/*`, `src/workspace.ts` | this prompt only — ultra cannot see it |

**Order:** run `/code-review ultra` first, then **F3 → F5 → F4** (the three ultra cannot do), then
F1/F2 for whatever ultra's diff scope missed.

---

## Notes (outside the prompt)

- **F1/F2 findings that ultra also found are not wasted** — agreement between two independent
  instruments raises confidence. But run ultra first so the pass can spend its budget on what
  ultra missed rather than re-deriving it.

- **The `/code-review ultra` split is the load-bearing idea here.** Diff-scoped review and
  whole-app invariant sweeps are different problems; conflating them is why "review the codebase"
  usually produces a pile of style notes. Keep F3/F4/F5 whole-app and let ultra own the diff.

- **Re-check the authority claim** if an Anthropic engineering skill bundle is ever installed —
  Track F should load it instead of relying on §2's inline checklists.
