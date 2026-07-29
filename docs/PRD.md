# SandiBumi — Product Requirements Document

**Version 0.1 of this document · 2026-07-29 · describes product version `0.1.0`**

Produced by Prompt 1 Step 1 of `docs/product_definition_prompt.md`, under its §0 rules. Every
capability claim below is derived from the code and cites its evidence. Where the repository could
not settle a question, it appears in §10 as an open question rather than as a finding.

**This document describes the product as it exists on 2026-07-29.** It is deliberately *not* a
plan. The forward-looking scope gate is `docs/V1_SCOPE.md` (Prompt 1 Step 2), which does not exist
yet and must not be written until this document is reviewed and accepted.

---

## Executive summary

SandiBumi is a Windows desktop application for multi-well petrophysical log analysis, built by a
practising petrophysicist for Indonesian E&P assets, intended to be **licensed to operators and
consultancies**. It ships as a single installer with an embedded database and no server component.

Four numbers decide how this product should be talked about today:

| Measure | Value | What it means commercially |
|---|---|---|
| Product version | **0.1.0** | pre-1.0; nothing has been released or sold |
| Backend surface | **118** Tauri commands · **42** registered petrophysics modules · **44** Rust files | the functional breadth is real and unusually large for a solo product |
| Backend regression net | **426** `#[test]` / `#[cfg(test)]` sites | the physics has genuine automated defence |
| **Field-verified share of the review checklist** | **72 of 370 items — 19.5%** | **the single most important number in this document** |

That last row is the product's central tension. The breadth is genuine and the backend is well
tested, but **four out of five checklist items have never been exercised by a human against real
well data.** For a personal tool that is a normal state of affairs. For software sold to an
operator it is the gap between what is demonstrable and what is defensible, and it will be probed
during any serious evaluation.

Everything else in this document is detail. That ratio is the product decision.

---

## 1. Product statement

SandiBumi is a deterministic petrophysical interpretation workstation for field-scale multi-well
work: it ingests LAS and DLIS logs from wells of mixed vintage and mixed mnemonic conventions,
conditions them (environmental corrections, bad-hole flagging, GR normalisation, depth alignment),
evaluates them through a library of shale-volume, porosity, saturation, permeability and
multi-mineral models, propagates parameter uncertainty across a whole field, and emits
print-quality composite logs and PDF reports — for one well or for hundreds, from one installer,
with the entire dataset resident on the interpreter's own machine.

What it is *not*, in one line, because the distinction is the product: it is not a log-viewing tool
that also does maths. The unit of work is **a field**, not a well — the workflow chain, the batch
Monte Carlo, the field dashboard and the pay summary all operate across the whole well set, and the
single-well views exist to inspect and defend what the batch produced.

---

## 2. The problem

**Grounded claims** (traceable to material already in this repository):

The incumbent suites in this market are Interactive Petrophysics, Techlog and Geolog; the
repository holds executed install-level ingests of two of them (`docs/sandibumi_maturation_prompt.md`
§INPUTS, `docs/techlog_ingest_prompt.md`), so their capability surface is a known quantity rather
than an assumption. Jauhar's own working corpus is 1000+ multi-vintage wells with ground-truth
interpretations produced in those packages, plus roughly 50 past projects whose parameter choices
exist as project files rather than as anything queryable.

Three problems are asserted by the design work already on disk and are consistent with what the
code prioritises:

1. **Multi-vintage heterogeneity dominates the cost of field-scale work.** A 1000-well set is not
   a big version of a 10-well set: mnemonics disagree, units disagree, some runs are placeholder
   columns of nulls, depth indices are not always called `DEPT`. The repository's own defect
   history confirms this is where time goes — the malformed-LAS defect family is called out in
   `docs/qc_audit_prompt_template.md` as having recurred 3+ times, and the BLSO real-data pipeline
   test exists specifically because coverage-aware alias resolution had to be fixed against real
   files (`CLAUDE.md`, Phase 9).
2. **Parameter knowledge does not accumulate.** Choices of `m`, `n`, `Rw`, cutoffs and endpoints
   live inside individual project files, per project, and are re-derived rather than reused.
3. **Deliverable production is a separate manual step** from interpretation, so the report lags the
   interpretation and the two can silently disagree.

**Not established, and needed** — see §10: whether these are *the buyer's* top three problems, or
Jauhar's. They are the problems the software was built to solve, which is a different claim. No
customer interview material exists in this repository.

---

## 3. Users

### 3.1 The interpreting petrophysicist — the buyer's user

Runs the software daily. Wants to take a field of mixed-vintage wells to a defensible property
model and a set of deliverables, and to be able to answer *"where did this number come from?"* six
months later.

*Failure looks like:* a number they cannot defend. Not a crash — a plausible result computed from
the wrong input, the wrong zone, or a default they did not know had changed. The entire
data-honesty discipline in this codebase exists for this user, and `REVIEW.md` Round 87 (a tops
pane briefly publishing one well's depth interval under another well's id, re-windowing every plot
in the workspace) is the canonical example of the failure mode.

### 3.2 The asset team and management — consumers who never open the app

Receive composite logs, pay summaries, zone parameter tables and PDF reports. Never see the UI.

*Failure looks like:* a deliverable that disagrees with the last one for reasons nobody can
articulate. This user is why the "numbers that changed" changelog category in `docs/RELEASE.md`
(Prompt 3) is mandatory rather than nice-to-have — they are the ones holding last quarter's number.

### 3.3 The client's IT department — no interest in petrophysics, absolute veto over the sale

Must approve installation on a managed Windows estate. Will ask, in roughly this order: does it
need admin rights; does it call out to the internet; where does our well data go; what is this
Python thing; who supports it.

*Failure looks like:* the evaluation never starting. **Section 7.5 answers three of those five
questions well and one of them badly** — the Python runtime dependency is the most likely single
cause of a stalled deployment, and it is a design consequence, not an oversight (`CLAUDE.md` rule
7: Python runs as a subprocess precisely so that a missing interpreter can never stop the app
launching). The engineering is right. The procurement conversation is still hard.

---

## 4. Capabilities as shipped

**Bucket definitions** (§0.2 of the product-definition prompt — these are never blurred):

- **SHIPPED & FIELD-VERIFIED** — a human has exercised it against real well data and marked it
  `[x]` in `REVIEW.md`.
- **SHIPPED, NOT YET VERIFIED** — code-complete, `tsc`/`cargo` clean, covered by automated tests,
  never clicked through against real data.
- **PLANNED** — in `ROADMAP.md`, not in the app.

### 4.0 The verification ratio, and why it cannot yet be reported per capability

`REVIEW.md` contains **370 checklist items across 88 rounds: 72 marked `[x]`, 298 unmarked
(19.5% verified).** For comparison, `docs/qc_audit_prompt_template.md` recorded 222 items with 150
unchecked on 2026-07-21 — so in the intervening period the list grew by ~148 items while the
verified count grew far less. **The backlog of unverified work is growing faster than it is being
retired.** That trend, not the absolute number, is the finding.

A structural obstacle sits behind it, and it is a product problem rather than a housekeeping one:
**`REVIEW.md` is indexed by round — that is, by time — not by capability.** Answering "is the
Monte Carlo module field-verified?" requires reading 88 rounds and reconstructing which round
touched it. That is tolerable for a solo developer with the history in his head. It is not
tolerable when a client asks for a verification matrix, which is a normal request during
evaluation. This is the same critique `stewardship_prompt.md` makes of rationale being scattered
across time-ordered rounds; the content exists, the retrieval path does not.

**Consequence for this document:** the buckets below are assigned at capability-group level from
the code and the roadmap. Per-capability verification status is *not* asserted, because it cannot
currently be derived without manual reconstruction. Building that index is a §10 open question and
a strong candidate for a v1.0 gate item.

### 4.1 Getting data in

*Shipped.* LAS 2.0 import with coverage-aware alias resolution (skips all-null placeholder
columns) and declared-NULL honouring; DLIS import via a `dlisio` subprocess; CSV/TXT importers for
tops, core, petrography, XRD, perforations, deviation surveys, SCAL Pc/Sw curves and well
locations; a generic curve store that accepts any mnemonic, canonicalises units and tags families,
so modules resolve inputs by mnemonic-then-family rather than requiring a fixed six curves
(`equations::fetch_curve_frame`, `CLAUDE.md` rule 10).

*Evidence:* `ingest.rs`, `parsers.rs`, `dlis.rs`, `deviation.rs`, `geo.rs`, `satheight.rs`;
`pipeline_blso_test.rs` is a real-field-data regression test rather than a synthetic one.

*Caveat that belongs in any demo:* the malformed-input defect family here has recurred repeatedly
and is explicitly named as a standing risk in this repo's own QC prompt. Import robustness is
better than it was and is not a solved problem.

### 4.2 Conditioning and QC

*Shipped.* Environmental corrections (GR hole, neutron, density), bad-hole flagging with a
universal run mask that NaNs flagged samples across every module output, conductivity flagging,
formation-temperature gradient, two-point percentile GR normalisation, KNN synthetic log
prediction, depth shift and splice, versioned log sets (RAW/EDIT/FINAL) carrying provenance, and
an undo stack over data and UI edits.

*Evidence:* `modules.rs`, `curve_edit.rs`, `undo.ts`, `log_sets`.

### 4.3 Petrophysical evaluation

*Shipped.* **42 modules registered in `modules::list_modules()`**, spanning shale volume,
porosity (including the SSC/SSPW sand-silt-clay suite), saturation (Archie, Indonesia, Simandoux,
the LRLC RtC and IMTS methods, dual-water, Waxman-Smits), permeability, thin beds, rock typing
(Lucia, Pittman, cutoff classifier), electrofacies (k-means and GMM), unconventional (Passey TOC,
kerogen, gas-in-place, brittleness) and saturation-height. Alongside them: **SandiMin**
(`multimin2.rs`), a 27-component multi-mineral optimiser with conductivity coupling; a
scikit-learn ML bridge; Rhai and Python (numpy) user equations.

*Evidence:* the 42 `*_spec()` entries at `modules.rs:169-230`; `multimin2.rs`; `ml.rs`;
`equations.rs`; method math banked in `docs/method_ssc_sspw.md`, `docs/method_lrlc_rtc_imts.md`,
`docs/multimin_ref_spec.md`, `docs/multimin_ip_spec.md`.

*Boundary worth stating:* the two legacy items are correctly quarantined rather than hidden —
`multimin.rs` (the fixed 4-component solver) has been gracefully retired in favour of SandiMin, and
`inversion.rs`'s `start_inversion` is a stub. Both are named as such in the QC inventory. A demo
must not route through either.

### 4.4 Field-scale operation

*Shipped.* Workflow chains (ordered module lists across many wells, sequential steps with
rayon-parallel wells, pollable progress and cancellation, per-step parameter overrides); Monte
Carlo uncertainty with Latin hypercube sampling and Iman-Conover correlation, producing P10/P50/P90
net/NTG/PHIE/SWE/HPV per zone entirely in memory; a field dashboard aggregating pay summaries
across every well; cutoff sensitivity sweeps; well groups as a global filter.

*Evidence:* `chain.rs`, `workflow.rs`, `montecarlo.rs`, `dashboardPanel.ts`.

*The one measured performance figure in the product:* a real 100-well × 4-module chain runs in
**21 s**, improved from ~50 s by removing the `computed_curves` primary key and batching each
well's output into a single DELETE plus one Appender (`CLAUDE.md` Phase 9 increment 5;
`ROADMAP.md:385`).

### 4.5 Interpretation and QC visuals

*Shipped.* WebGPU multi-track log views with synchronised hover, layouts and facies block tracks;
histogram, crossplot, Pickett and correlation panels with linked brushing; **19 vector-digitized
Schlumberger-2013 chart overlays**; interactive Thomas-Stieber with draggable endpoints writing
zone parameters; a Results-QC panel (Sw spread, Buckles, unity checks, per-zone scorecard); an
embedded Vega-Lite panel with a spec editor.

*Evidence:* `logViewPanel.ts`, `plotCanvas.ts`, `crossplotPanel.ts`, `chartOverlays.ts`,
`resultsqc.rs`, `vegaPanel.ts`.

### 4.6 Deliverables

*Shipped.* Composite log plots at true print scale exporting vector SVG and a dependency-free
multi-page PDF; a report generator (cover, editable methodology table, zone parameters, pay
summary, composite pages) with batch export per well; LAS export; PNG and true-vector SVG/PDF
export from the chart panels.

*Evidence:* `composite.rs`, `report.rs`, `export.rs`, `svgExport`/`PdfRecorder`.

### 4.7 Workspace and shell

*Shipped.* Dockview docking workspace, Office-style ribbon, named sessions, processing history,
crash safe-mode and autosave, a read-only SQL console, a database inspector with undo, and eight
themes. UI language: English, Bahasa Indonesia, Basa Sunda, with technical terms deliberately left
in English.

### 4.8 PLANNED — in the roadmap, not in the app

`ROADMAP.md` currently stands at 55 `[x]`, 13 `[ ]`, 2 `[~]`. The Critical and Reliability
hardening tiers are complete. Open: async commands (#128) and a connection pool (#129, flagged
high-risk), both needing a live 100-well run to sign off; per-well parameter override tables; lazy
catalog loading and a decimation cache; UI responsiveness during full-field runs; **a 2000-well
stress fixture**; missing-curve synthesis; auto-picks and auto-zonation; PyTorch autoencoders.

**The 2000-well fixture is not a housekeeping item** — see §7.1.

### 4.9 The vision that is not the product — and is deliberately not meant to be

> **Confirmed by Jauhar, 2026-07-29:** this is accepted as written, and the two-agent automation
> vision is **allocated to SegaraBumi**, not to SandiBumi. It is therefore a deliberate product
> boundary rather than a gap — see non-goal §5.7.

`docs/sandibumi_maturation_prompt.md` describes SandiBumi as a **two-agent architecture** — Agent 1
for large-scale conditioning of 1000+ multi-vintage wells, Agent 2 for automated zonation,
parameter identification and interpretation — backed by a decision playbook and a queryable
parameter knowledge base built from 50+ past projects.

**None of that is shipped, and no part of this document should imply otherwise.** What exists today
is a *manual* interpretation workstation with excellent batch execution: the interpreter chooses
the modules, the parameters and the zones, and SandiBumi runs them across the field quickly and
reproducibly. The automation layer — auto-zonation, automated parameter identification, the
knowledge base — is the roadmap's frontier (auto-picks/auto-zonation sit in the carried-forward
deferrals), not the current offering.

**What that decision buys.** Two things, both worth more than the feature would have been. First,
SandiBumi 1.0 gets a scope that can actually close — automation is the single most open-ended thing
on the roadmap, and moving it out is what makes a 1.0 gate arithmetically possible. Second, the two
products get a clean division of labour: SandiBumi is the deterministic, defensible, *auditable*
interpretation engine, and the automation layer lives where a wrong answer is a suggestion rather
than a number in a reserves report.

**What it still requires of this document.** The vision must not appear in SandiBumi's customer-
facing copy, in any form, until it is a shipped SegaraBumi capability with a defined seam. §0.2 is
the rule; this is its most likely violation, because the vision is genuinely compelling and lives
one file away. Positioning SandiBumi today means selling a fast, reproducible, provenance-carrying
manual workstation — which is what it is, and is enough.

---

## 5. Non-goals

Each carries its reason, because a non-goal without a reason is a to-do nobody reached yet.

1. **Not a seismic interpretation package.** The input is well logs. Adding seismic means adding a
   volume data model, a rendering problem and a whole second validation surface, for a user who
   already owns a seismic package.
2. **Not a reservoir simulator.** The deliverable is the property model that *feeds* a simulator.
   Owning both doubles the validation burden for no additional buyer — the same person buys both,
   from different vendors, on purpose.
3. **Not a real-time, wellsite or geosteering tool.** Every design decision assumes an
   after-the-fact dataset on local disk: a single embedded file, a single writer, batch chains
   measured in seconds-to-minutes. Real-time streaming would invalidate that architecture, not
   extend it.
4. **Not a corporate data-management or master-database product.** A project is one file on one
   interpreter's machine. Becoming the system of record means multi-user access control, audit and
   retention — a different product, sold to a different buyer (see §7.6 for how this constrains the
   multi-user future).
5. **Not a core-laboratory workflow system.** Core, XRD and petrography data are *imported as
   calibration and validation inputs*. Managing the laboratory workflow that produces them is a
   separate domain.
6. **Not a general-purpose analytics platform.** The Vega-Lite panel and the SQL console are
   escape hatches for an interpreter who needs one plot the product does not have. They are not a
   promise to be Spotfire, and must not be positioned as one.
7. **Not an automated interpreter.** Auto-zonation, automated parameter identification, the
   decision playbook and the queryable parameter knowledge base are **SegaraBumi's scope**
   (Jauhar, 2026-07-29 — §4.9). SandiBumi's promise is that the *interpreter* decides and the
   software executes across the field quickly, reproducibly and with provenance. The reason this
   is a boundary rather than a backlog item: an automated pick that is wrong is wrong silently and
   at field scale, which is the exact failure mode this product's entire data-honesty discipline
   exists to prevent. Automation belongs where its output is labelled as a suggestion.

---

## 6. Differentiation, honestly

### 6.1 Where SandiBumi genuinely differs

- **Field-scale batch is native, not bolted on.** Workflow chains, batch Monte Carlo and the field
  dashboard were built as the primary interaction, with the single-well views serving them.
- **Jauhar's own method suite is implemented and specified.** SSC/SSPW, LRLC RtC and IMTS exist
  here with their math banked in `docs/`. These are not available in the incumbent suites.
- **Deployment simplicity.** One installer, embedded DuckDB, no server, no external database, no
  licence server contacted at run time (§7.5). For an asset team without dedicated geoscience IT
  support this is a material difference.
- **Provenance is structural.** Versioned log sets, `log_sets` provenance rows, a computed-curve
  catalog and a processing history are in the data model rather than in convention.
- **Local-language UI** (Bahasa Indonesia, Basa Sunda) with technical terms deliberately preserved
  in English.

### 6.2 Where it deliberately does not compete — state this before a buyer discovers it

- **Ecosystem and integration breadth.** The incumbents integrate with corporate data stores,
  seismic packages and geomodelling suites. SandiBumi reads files.
- **Track record and institutional acceptance.** A partner or regulator who expects an
  IP/Techlog/Geolog interpretation is expecting a known quantity. SandiBumi has no such history.
- **Breadth of niche methods.** 42 modules is substantial; a mature suite carries several times
  that across acoustics, image logs, NMR inversion, formation testing and more.
- **Image log and full NMR inversion workflows** are not present.
- **Support organisation.** One person, in one timezone. This is a real product attribute, not a
  detail to be discovered after the sale.

**The rule this section encodes:** a licensed product that claims parity it does not have gets
found out in the first evaluation, and every other claim it made becomes suspect at the same
moment. The asymmetry is severe — an admitted gap costs a feature; a discovered overclaim costs the
deal.

---

## 7. Non-functional requirements

Every figure below is marked **measured**, **target**, or **unmeasured**.

### 7.1 Scale

| Claim | Status |
|---|---|
| 100-well × 4-module chain | **measured**: 21 s |
| 1000+ well corpus (Jauhar's validation set) | **target** — no benchmark recorded in-repo |
| "2000+ wells" (`README.md`, `CLAUDE.md`) | **unmeasured** — the 2000-well stress fixture is an open roadmap item |

**This is the most commercially dangerous line in the document.** The 2000-well figure is stated in
customer-facing copy today and has never been demonstrated. Per Prompt 3's 1.0 bar, exactly one of
two things must happen before a paid release: build the fixture and demonstrate it, or remove the
number from all customer-facing text. Leaving it ambiguous is the only unacceptable option.

### 7.2 Performance

Measured: the 100-well chain (above). Everything else — UI responsiveness during full-field runs,
catalog loading at scale, plot redraw at high sample counts — is **unmeasured**, and UI
responsiveness during full-field runs is an acknowledged open roadmap item. The two remaining
performance items (#128 async commands, #129 connection pool) are explicitly blocked on a live
100-well run for sign-off.

### 7.3 Install footprint

**Measured from configuration:** `productName` SandiBumi, `identifier` `com.sandibumi.petro`,
version `0.1.0`, bundle `targets: "all"`. No external database, no application server, no runtime
framework — DuckDB is compiled in as a bundled Cargo feature, and Windows 11 ships the WebView2
runtime.

**The exception, stated plainly because it is a sales objection: Python.** Three shipped
capabilities require a Python 3.10+ interpreter on the client machine:

| Capability | Package required |
|---|---|
| User equations | `numpy` |
| DLIS import | `dlisio` |
| The entire ML suite | `scikit-learn` |

Four backend files call `find_python` (`python_engine.rs`, `dlis.rs`, `ml.rs`, `lib.rs`).
Discovery order is `ARSHILLA_PYTHON` → `%LOCALAPPDATA%\Programs\Python\Python31x` → `PATH`. The
design is deliberately fail-soft: a missing interpreter degrades exactly those three features and
never prevents the app launching. **The engineering is correct and the procurement problem remains:**
"install Python and three packages on every seat" is a request many managed Windows estates will
refuse or delay. §9 carries it as a risk; §10 carries the unmade decision about bundling.

*Not a product constraint, despite living beside these in `CLAUDE.md`:* the MSVC 14.29 toolchain
pin. That is a broken toolset on one development machine. No customer will ever encounter it, and
it belongs in `CONTRIBUTING.md` only.

### 7.4 Offline capability

**Measured: the application is fully offline-capable.** See §7.5 for the evidence and its limits.
No feature requires connectivity, including licence checks, because no licensing exists yet (§8).

### 7.5 Data security posture

**The claim:** client well data never leaves the machine.

**How that was checked, and what the check does and does not cover.** Three independent searches:

1. **No HTTP client is compiled in.** `src-tauri/Cargo.toml` contains no `reqwest`, `hyper`,
   `ureq`, `curl`, `tungstenite` or `tauri-plugin-http`.
2. **The frontend makes no network calls.** No `fetch(`, `XMLHttpRequest` or `new WebSocket`
   anywhere in `src/`. The only external URLs present are `http://www.w3.org` (an SVG namespace
   literal, not a request) and `https://vega.github.io/schema/vega-lite/v5.json`
   (`vegaPanel.ts:306`), which is a Vega-Lite `$schema` declaration — descriptive metadata that the
   renderer does not fetch.
3. **No auto-updater is configured.** `tauri.conf.json` has no updater block and no endpoints, so
   the app performs no version check.

**FIXED 2026-07-29 — the granted-but-unused capability is gone.** `tauri-plugin-opener` was
registered at `lib.rs:1638` and permitted via `opener:default`, which would have let the app hand a
URL or path to the OS. It had **zero call sites in `src/`**, so nothing was ever passed to it — but
a granted capability the product does not use is exactly what an enterprise security review asks
about. Removed at all four layers: the Rust plugin registration, the `tauri-plugin-opener` crate
dependency, the `opener:default` capability entry, and the `@tauri-apps/plugin-opener` npm package.
A comment at the registration site records that re-adding it means re-adding the capability *and*
revisiting this section.

**Limits of that evidence, stated so the claim stays defensible:** it covers first-party code and
declared dependencies, not transitive dependency behaviour; and the Python subprocess is a separate
process whose temp-file behaviour has not been audited for client data residue (§10.6).

**One weakness fixed, one still open:**

- **FIXED 2026-07-29 — the webview now has a Content Security Policy.** It was `"csp": null`,
  which mattered here in particular: `REVIEW.md` R9 records a fixed LAS-well-name XSS-to-RCE
  vector, where a hostile well name in an imported file reached the DOM. That hole was closed, but
  a null CSP meant no second line of defence behind input sanitisation — and untrusted text arrives
  with every imported file. The policy now set in `tauri.conf.json`:

  ```
  default-src 'self'; script-src 'self' 'unsafe-eval'; style-src 'self' 'unsafe-inline';
  img-src 'self' data: blob:; font-src 'self' data:;
  connect-src 'self' ipc: http://ipc.localhost; frame-src 'self'; worker-src 'self' blob:;
  object-src 'none'; base-uri 'self'; form-action 'none'; frame-ancestors 'none'
  ```

  Two relaxations are deliberate and each was checked against what the app actually does:

  - **`script-src` keeps `'unsafe-eval'`** because Vega 5 compiles chart expressions through the
    `Function` constructor; without it the Vega panel silently stops rendering. Critically, this
    does **not** re-open the R9 class — inline event handlers and inline `<script>` require
    `'unsafe-inline'`, which is *absent*, and `'unsafe-eval'` does not grant it. The residual risk
    is narrow: an attacker would need to reach a `Function`/eval call site with controlled input,
    not merely inject markup. Removing `'unsafe-eval'` entirely is possible by switching Vega to
    `vega-interpreter`; that is a roadmap item, not a blocker.
  - **`style-src` keeps `'unsafe-inline'`** because CodeMirror injects a `<style>` element at
    runtime and the print path writes an inline `<style>` into its hidden iframe. Style injection
    is not a script-execution vector.

  **Verification caveat, stated plainly: this cannot be exercised by `npm run tauri dev`.** With a
  `devUrl` the webview loads the Vite dev server directly and Tauri does not deliver the policy —
  it applies to the packaged app served over the custom protocol. Proving it needs
  `npm run tauri build` and a click-through of the Vega panel, the equation editor and plot
  printing (the three paths the relaxations exist for). This is recorded as a `Try:` line in
  `REVIEW.md`.

- **STILL OPEN — the project database is unencrypted at rest.** No encryption, cipher or AES code
  exists in the backend; the `.duckdb` file is readable by anything with filesystem access. For a
  laptop carrying a client's whole field, this will be asked about. **Deliberately not fixed in
  this pass:** encryption at rest is a feature, not a hardening tweak, and its hard part is key
  management — a lost key means months of interpretation work is unrecoverable, which is a worse
  outcome than the risk it mitigates. It needs a designed answer (where the key lives, how it is
  recovered, what happens on a forgotten passphrase) before any code. Carried as R7.

### 7.6 Deployment model

**Today:** single-user Windows desktop, one project per `.duckdb` file, one writer
(`Mutex<Connection>` — fundamental by design, not a defect). **Named future state:** a shared
multi-user backend. Not built, not scheduled, and deliberately not foreclosed — the architectural
consequences are Prompt 2's subject (`docs/TARGET_ARCHITECTURE.md`), and non-goal §5.4 is the
boundary that keeps "several people share a project" from silently becoming "we are the corporate
system of record."

### 7.7 Quality assurance posture — as a product attribute

An enterprise buyer may ask what assures quality. The honest answer today:

| | Status |
|---|---|
| Backend automated tests | **426** test sites, plus a real-field-data pipeline test |
| Frontend automated tests | **none** — no vitest, no harness in-tree |
| Lint configuration | **none** — no eslint, clippy or rustfmt config committed |
| Continuous integration | **none** — no `.github/workflows` |
| The green gate | `npm run build` (= `tsc && vite build`) and `cargo test`, run manually |
| Independent audit | two adversarial audits on file: 35 confirmed findings (2026-07-20) and a 24-agent full-tool QC pass (2026-07-21) |

The asymmetry matters and is already documented in `stewardship_prompt.md`: **the tested half and
the historically buggy half are different halves.** The 426 tests defend the physics; the 67
untested frontend files are where the wrong-well, stale-data and lifecycle defects have actually
occurred (R24 through R29 are all frontend). The adversarial audit practice is a real and unusual
strength for a solo product and partly compensates — but it is a process, not an artefact, and
cannot be handed to a buyer as evidence.

### 7.8 Localisation

Shipped: English (source language), Bahasa Indonesia, Basa Sunda, via exact-phrase dictionary
lookup with a MutationObserver. Technical terms (Thin Beds, Monte Carlo, Pickett, mnemonics) remain
English by explicit design decision.

---

## 8. Commercial surface

**Nothing in this section is implemented.** No licence key, activation, entitlement or trial code
exists anywhere in `src/` or `src-tauri/src/`. The commercial surface is entirely undecided, which
is the correct state for version 0.1.0 and an unacceptable one at 1.0.

These are decisions Jauhar owes, with consequences, not recommendations:

| Decision | Options | Consequence |
|---|---|---|
| **Licence unit** | per named user · per machine · per site/asset team | A field-scale tool used by 2–3 specialists per asset team makes per-site simplest to sell and hardest to price; per-machine is easiest to enforce and irritates users with a laptop and a workstation. |
| **Activation** | none (trust) · offline key file · online activation | §7.4's fully-offline property is a genuine differentiator for air-gapped or restricted estates. **Online activation would destroy it.** An offline signed key file preserves it. |
| **Perpetual or subscription** | perpetual + maintenance · annual subscription | Subscription requires a support organisation that does not exist yet (§6.2). Perpetual-with-maintenance matches a one-person vendor better and caps the promise. |
| **Update delivery** | manual download · in-app updater | An in-app updater means re-introducing network egress and losing the clean §7.5 answer. Weigh it against the convenience honestly. |
| **Support commitment** | best-effort · defined response time | Any response-time commitment from a single person needs an explicit working-hours and holiday boundary, in writing, before the first sale. |
| **Version support window** | latest only · N-1 · N-2 | Directly drives the project-file compatibility policy in `docs/RELEASE.md` §3. |

**One decision is more urgent than the rest:** whether Python stays a customer prerequisite, gets
bundled with the installer, or whether the three affected capabilities become an optional add-on
module. That choice changes the installer, the IT conversation and the feature matrix
simultaneously (§10.4).

---

## 9. Risk register

Per §0.4: this identifies risks and routes them. **It renders no legal conclusions, and neither
Jauhar nor Claude is qualified to.**

**Status as of 2026-07-29** (Jauhar: *"9 solve it"*). Two risks are closed in code, three are
converted from *undocumented* to *documented and routed* via the new
[`docs/IP_PROVENANCE.md`](IP_PROVENANCE.md), and five remain open because they need a decision or a
feature rather than an edit. Nothing was closed by assertion.

| # | Risk | Status | Who must answer | Urgency |
|---|---|---|---|---|
| R1 | **Chart data provenance.** `chartOverlays.ts` and `neutron_charts.rs` both declare in their own headers that they are digitized from the *Schlumberger Log Interpretation Charts, 2013*. The values ship inside the product; the chartbook PDF does not. | **DOCUMENTED** — `IP_PROVENANCE.md` §2.1 records the asset, the derivation path, the precise legal question, and three costed fallbacks if the answer is unfavourable. | **Lawyer** | Before first sale |
| R2 | **Vendor-derived defaults.** SandiMin's 27-component `LIB` merges endpoint defaults from two vendor installs, in IP's dropdown order (`multimin2.rs:2048`). | **DOCUMENTED** — `IP_PROVENANCE.md` §2.2. Also identifies the action worth doing regardless of the legal answer: cite primary literature per row, converting "merged from vendor installs" into "sourced from the literature, cross-checked". | Jauhar (citations), lawyer (status) | Before first sale |
| R3 | **Third-party names in shipped code and copy.** | **PARTLY FIXED.** The copy is fixed: `README.md` no longer describes the product as "the reference suite-class" / "the reference suite-Multimin-class". The **theme ids are unchanged and escalated** — `halliburton`, `schlumberger`, `pertamina`, `lapi-itb` are *client*-branded palettes, and renaming them would delete the feature's purpose rather than answer the question. Four options in `IP_PROVENANCE.md` §2.5. | Lawyer (marks) | Before first sale |
| R4 | **Python prerequisite.** Three capabilities need a client-side interpreter plus packages. | **OPEN** — product decision (§10.4). Note `IP_PROVENANCE.md` §2.6 adds an argument *for* keeping it a prerequisite: not distributing the packages is a materially lighter licensing obligation than bundling them. | Jauhar | Before first enterprise sale |
| R5 | **19.5% field-verified.** 298 of 370 checklist items never exercised against real data. | **OPEN** — cannot be closed by editing; it is verification effort. The strongest candidate for the v1.0 gate. | Jauhar | **Before first sale** |
| R6 | **No CI, no lint, no frontend tests.** | **OPEN** — see §7.7. Cheap first step exists (`rustfmt.toml`, `clippy.toml`, one green-gate script); not taken here because clippy-as-error on an existing tree is its own increment. | Jauhar | Before first enterprise sale |
| R7 | **Unencrypted project database.** | **OPEN, deliberately.** A feature, not a hardening tweak: the hard part is key management, and a lost key destroys months of interpretation — a worse outcome than the risk. Needs a designed answer first (§7.5). | Jauhar / client security review | Before first enterprise sale |
| R8 | **Null CSP** with untrusted text arriving from every imported LAS/DLIS file. | **FIXED** — a real policy is now set; `'unsafe-inline'` is absent from `script-src`, which is what defeats the R9 class. Full policy, the two deliberate relaxations, and the build-only verification caveat: §7.5. | — | Closed |
| R9 | **Single-person bus factor.** No CI, no second maintainer, no `ARCHITECTURE.md`, no ADRs. | **OPEN** — `stewardship_prompt.md` Prompts 2 and 4 exist precisely to close this and have not been run. | Jauhar | Before first enterprise sale |
| R10 | **Support obligation with no defined boundary** (§8). | **OPEN** — a commercial decision, not an engineering one. | Jauhar | Before first sale |
| R11 | **Granted-but-unused OS capability** (`opener`). | **FIXED** — removed at all four layers (§7.5). Recorded as a numbered risk because it was found by this pass, and because the same check should run on every new plugin. | — | Closed |

---

## 10. Open questions

Each carries the specific thing that would settle it.

1. **Are §2's three problems the *buyer's* problems, or the builder's?** — Settled by: structured
   conversations with 3–5 petrophysicists at target operators. No customer research exists in this
   repository, and §2 is honest about the distinction.
2. **Who signs the purchase order, and what is the budget line?** — Settled by: one conversation
   with a target account. Determines whether §8's licence unit is per-seat or per-asset-team.
3. **What is the 2000-well answer?** — Settled by: building the stress fixture (already a roadmap
   item) *or* deleting the claim from `README.md` and `CLAUDE.md`. Cannot remain open past 1.0
   (§7.1).
4. **Python: prerequisite, bundled, or optional add-on?** — Settled by: a decision from Jauhar plus
   one experiment (attempt an install on a genuinely locked-down machine). The most consequential
   open item in this document.
5. **What must a per-capability verification matrix look like, and what does it cost to build?** —
   Settled by: attempting to derive it from `REVIEW.md` for three capabilities and measuring the
   effort (§4.0).
6. **Does the Python subprocess leave client data in temp files?** — Settled by: reading the
   subprocess protocol in `python_engine.rs`, `dlis.rs` and `ml.rs` for temp-file use. Relevant to
   R7 and to any security questionnaire. *(The related question about `plugin-opener` is answered
   in §7.5: registered and permitted, zero call sites — the remaining action is deciding whether to
   drop the permission, not an open question.)*
7. **Is the automation vision (§4.9) part of the product being sold, or the next product?** — Only
   Jauhar can answer, and the answer changes the positioning entirely.

---

## 11. Where the documents disagree with the code

Per rule 0.1: the code is the fact and the document is the bug. **Reported, not fixed** — these
enter the normal one-at-a-time flow.

1. **`CLAUDE.md` misstated `REVIEW.md`'s own convention — FIXED 2026-07-29.** `CLAUDE.md`
   (Collaboration protocol §3) said `[o]` OK / `[x]` **wrong** / `[ ]` untested, while
   `REVIEW.md`'s own header says `[x]` = confirmed done. *Jauhar's account, 2026-07-29:* he began
   with `[o]` and then switched the mark style to `[x]`, so **`[x]` means accepted**; `CLAUDE.md`
   had preserved the superseded legend. Exactly one `[o]` survives, at `REVIEW.md:4317`.
   *Consequence while it stood:* read through the old legend, 72 accepted items looked like 72
   broken ones — a live trap for any new session. `CLAUDE.md` now states the current convention and
   flags the legacy mark. **The 19.5% figure in §4.0 is unaffected and confirmed**: it counts `[x]`
   as accepted, which is what it means.
2. **`docs/qc_audit_prompt_template.md` §3 carries stale counts** — "150 of 222 as of 2026-07-21"
   against today's 298 of 370. The file already warns against trusting its own list after
   refactors; the numbers should be re-derived, never quoted.
3. **`README.md` and `CLAUDE.md` state "2000+ wells"** as a present capability. The measured figure
   is 100 wells; the 2000-well fixture is an open roadmap item (§7.1).
4. **`README.md` described the module library as "the reference suite-class"** and SandiMin as
   "the reference suite-Multimin-class" — a competitor-referential description in the primary
   customer-facing document. **FIXED 2026-07-29** (R3). The same phrasing persists in `CLAUDE.md`
   and `ROADMAP.md`, which is acceptable while they remain internal and would need the same
   treatment before publication.
5. **`CLAUDE.md` said "four reusable prompts"** while `docs/` held more; corrected on 2026-07-29
   when `product_definition_prompt.md` was added, with the vendor-intelligence prompts named as a
   separate family.
6. **The two-agent product thesis existed only inside a prompt file**
   (`sandibumi_maturation_prompt.md`), not in any product document, and is not shipped.
   **RESOLVED 2026-07-29** by Jauhar's decision to allocate it to SegaraBumi — now recorded in
   §4.9 and as non-goal §5.7. `sandibumi_maturation_prompt.md`'s ROLE section still describes the
   two-agent architecture as SandiBumi's, which is now stale and should be corrected the next time
   that prompt is run.

---

## Acceptance

**Reviewed 2026-07-29.** §4.9 confirmed (automation → SegaraBumi); §9 actioned (two risks fixed in
code, three documented and routed, five open by decision); §11 item 1 corrected from Jauhar's own
account of the mark-style change; §7.5 fixed.

**One item still awaits him: §7.1 — the "2000+ wells" claim.** It appears in `README.md` and
`CLAUDE.md` as a present capability, the measured figure is 100 wells, and the stress fixture is an
open roadmap item. Per §7.1 exactly one of two things must happen before a paid release: build the
fixture and demonstrate it, or delete the number from customer-facing text. It is left untouched
here because choosing between those is a product decision, not an edit.

Once accepted, the sequence continues: `docs/RELEASE.md` (Prompt 3 — the quality bar), then
`docs/V1_SCOPE.md` (Prompt 1 Step 2 — what 1.0 contains), then `docs/TARGET_ARCHITECTURE.md`
(Prompt 2). `ROADMAP.md` is reconciled against `V1_SCOPE.md` last, by Jauhar.
