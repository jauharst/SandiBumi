# PRD v2 · Part I — The product as it stands

**Sections §1–§8.** Risk register, open questions and document-vs-code contradictions continue in
`02_RISKS_AND_CONTRADICTIONS.md` (§9–§11).

*Absorbed from `docs/PRD.md` v0.1 (2026-07-29) in full. Section numbers preserved. Corrections
carrying the 2026-08-07 verification or the 2026-08 as-built audit are marked inline.*

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

**v2 amendment — the unit of work is now a *portfolio*.** The target scale is thousands of wells
across many fields, not hundreds within one. This is a decided expansion of scope and it changes
requirements in `04_CORE_REQUIREMENTS.md` (`SB-CORE-030` … `SB-CORE-036`), not merely a bigger
number in a benchmark. See §7.1.

---

## 2. The problem

**Grounded claims** (traceable to material already in this repository):

The incumbent suites in this market are Interactive Petrophysics, Techlog and Geolog. **v2
strengthens this from "a known quantity" to a measured one:** the repository now holds
install-level ingests of all three plus an eighteen-domain cross-validation of their shipped
parameter files, executable source and manuals — 42,936 lines of evidence. Their capability
surface, their defaults, and their defects are documented rather than assumed. Jauhar's own working
corpus is 1000+ multi-vintage wells with ground-truth interpretations produced in those packages,
plus roughly 50 past projects whose parameter choices exist as project files rather than as anything
queryable.

Three problems are asserted by the design work already on disk and are consistent with what the
code prioritises:

1. **Multi-vintage heterogeneity dominates the cost of field-scale work.** A 1000-well set is not a
   big version of a 10-well set: mnemonics disagree, units disagree, some runs are placeholder
   columns of nulls, depth indices are not always called `DEPT`. The repository's own defect history
   confirms this is where time goes — the malformed-LAS defect family is called out in
   `docs/qc_audit_prompt_template.md` as having recurred 3+ times, and the BLSO real-data pipeline
   test exists specifically because coverage-aware alias resolution had to be fixed against real
   files.
2. **Parameter knowledge does not accumulate.** Choices of `m`, `n`, `Rw`, cutoffs and endpoints
   live inside individual project files, per project, and are re-derived rather than reused.
3. **Deliverable production is a separate manual step** from interpretation, so the report lags the
   interpretation and the two can silently disagree.

**v2 adds a fourth, and it is the one the cross-tool work uncovered:**

4. **The interpreter cannot see when the tools disagree, and they disagree constantly.** Three
   packages ship three different values for the same constant, apply the same named parameter at
   different points in the same equation, and print equations their own code contradicts. The
   interpreter has no way to know. `03_EVIDENCE_BASE.md` §14 quantifies this; it is the single
   largest differentiation opportunity in the document.

**Not established, and needed** — see `02_RISKS_AND_CONTRADICTIONS.md` §10: whether these are *the
buyer's* top four problems, or Jauhar's. They are the problems the software was built to solve,
which is a different claim. No customer interview material exists in this repository. **Unchanged at
v2 — this remains the largest unvalidated assumption in the product.**

---

## 3. Users

### 3.1 The interpreting petrophysicist — the buyer's user

Runs the software daily. Wants to take a field of mixed-vintage wells to a defensible property model
and a set of deliverables, and to be able to answer *"where did this number come from?"* six months
later.

*Failure looks like:* a number they cannot defend. Not a crash — a plausible result computed from
the wrong input, the wrong zone, or a default they did not know had changed. The entire data-honesty
discipline in this codebase exists for this user, and `REVIEW.md` Round 87 (a tops pane briefly
publishing one well's depth interval under another well's id, re-windowing every plot in the
workspace) is the canonical example of the failure mode.

### 3.2 The asset team and management — consumers who never open the app

Receive composite logs, pay summaries, zone parameter tables and PDF reports. Never see the UI.

*Failure looks like:* a deliverable that disagrees with the last one for reasons nobody can
articulate. This user is why the "numbers that changed" changelog category is mandatory rather than
nice-to-have — they are the ones holding last quarter's number.

### 3.3 The client's IT department — no interest in petrophysics, absolute veto over the sale

Must approve installation on a managed Windows estate. Will ask, in roughly this order: does it need
admin rights; does it call out to the internet; where does our well data go; what is this Python
thing; who supports it.

*Failure looks like:* the evaluation never starting. §7.5 answers three of those five questions well
and one of them badly — the Python runtime dependency is the most likely single cause of a stalled
deployment, and it is a design consequence, not an oversight (Python runs as a subprocess precisely
so that a missing interpreter can never stop the app launching). The engineering is right. The
procurement conversation is still hard.

---

## 4. Capabilities as shipped

**Bucket definitions — these are never blurred:**

- **SHIPPED & FIELD-VERIFIED** — a human has exercised it against real well data and marked it `[x]`
  in `REVIEW.md`.
- **SHIPPED, NOT YET VERIFIED** — code-complete, `tsc`/`cargo` clean, covered by automated tests,
  never clicked through against real data.
- **PLANNED** — in `ROADMAP.md`, not in the app.

### 4.0 The verification ratio — rewritten 2026-08-07

`REVIEW.md` now contains **1,125 checklist items: 75 marked `[x]`, 1,050 unmarked — 6.7 %
verified.** The trajectory, with every point measured rather than estimated:

| Date | Items | Verified | Ratio |
|---|---|---|---|
| 2026-07-21 | 222 | 72 | 32.4 % |
| 2026-07-29 (PRD v1) | 370 | 72 | 19.5 % |
| **2026-08-07** | **1,125** | **75** | **6.7 %** |

In the nine days since PRD v1 the checklist grew by **755 items** and the verified count grew by
**three**. PRD v1 called the trend the finding and was right; the trend is now steeper than the
document that named it.

Two readings are both true and must not be conflated. The generous one: the backlog grew because the
product grew — 209 commands where there were 118, 775 tests where there were 426 — and a larger
honest checklist is better than a smaller dishonest one. The unforgiving one: **the ratio a buyer
will compute is 6.7 %**, and no amount of context changes what that number does in an evaluation.

The structural obstacle behind it is unchanged and is a product problem, not housekeeping:
**`REVIEW.md` is indexed by round — by time — not by capability.** Answering "is the Monte Carlo
module field-verified?" means reading the rounds and reconstructing which touched it. That is
tolerable for a solo developer with the history in his head; it is not tolerable when a client asks
for a verification matrix, which is a normal request during evaluation. The content exists; the
retrieval path does not. **v2 promotes this from an open question to a requirement:
`SB-CORE-040`.**

**Consequence for this document:** the buckets below are assigned at capability-group level from the
code and the roadmap. Per-capability verification status is *not* asserted, because it cannot
currently be derived without manual reconstruction.

### 4.1 Getting data in

*Shipped.* LAS 2.0 import with coverage-aware alias resolution (skips all-null placeholder columns)
and declared-NULL honouring; DLIS import via a `dlisio` subprocess; CSV/TXT importers for tops, core,
petrography, XRD, perforations, deviation surveys, SCAL Pc/Sw curves and well locations; a generic
curve store that accepts any mnemonic, canonicalises units and tags families, so modules resolve
inputs by mnemonic-then-family rather than requiring a fixed six curves. **Added since v1:** a
universal Intake importer for any delimited text with LONG/WIDE/BLOCK layouts declared rather than
sniffed, saved mappings applied by column name, and `.xlsx` petrography plate-workbook extraction.
Export: LAS, PDF, SVG, PNG, xlsx, pptx, docx.

*Caveat that belongs in any demo:* the malformed-input defect family here has recurred repeatedly
and is explicitly named as a standing risk in this repo's own QC prompt. Import robustness is better
than it was and is not a solved problem.

*v2 note:* old `.xls` is refused by name with a fix instruction (Save As `.xlsx`) — and it is the
**majority** format for petrography, 107 of 165 workbooks on the reference machine. Chapter `DIO`
(`21_data-io.md`) carries the decision.

### 4.2 Conditioning and QC

*Shipped.* Environmental corrections (GR hole, neutron, density), bad-hole flagging with a universal
run mask that NaNs flagged samples across every module output, conductivity flagging,
formation-temperature gradient, two-point percentile GR normalisation, KNN synthetic log prediction,
depth shift and splice, versioned log sets (RAW/EDIT/FINAL) carrying provenance, and an undo stack
over data and UI edits. **Added since v1:** a full Condition suite — `despike` with four rejection
rules, `smooth` (mean/median/Savitzky-Golay on real depths), `clip`, `fill_gaps`, `flip`,
`normalize` — plus Frame operations (`block` with four bed definitions, `resample`, `regularize`,
`align_multiwell`) and Reframe.

### 4.3 Petrophysical evaluation

*Shipped.* **51 modules registered in `modules::list_modules()`** (`modules.rs:342-396`), spanning
shale volume, porosity (including the SSC/SSPW sand-silt-clay suite), saturation (Archie, Indonesia,
Simandoux, the LRLC RtC and IMTS methods), permeability, thin beds, rock typing (Lucia RFN, Pittman,
cutoff classifier), electrofacies (k-means and GMM), unconventional (Passey TOC, kerogen,
gas-in-place, brittleness) and saturation-height. Alongside them: **SandiMin** (`multimin2.rs`), an
N-component multi-mineral optimiser with a 27-entry endpoint library across 14 tool keys, hard-unity
simplex NNLS, seven Sw models inside the solver, and conductivity coupling; a scikit-learn ML bridge;
Rhai and Python (numpy) user equations.

**v2 correction — the legacy-solver hazard PRD v1 described is now closed.** v1 recorded
`multimin.rs` (the fixed 4-component solver) as "gracefully retired", and the 2026-08 audit found
that retirement was only skin-deep: hidden from UI pickers but still registered and therefore still
runnable from any saved chain or saved dockview layout, carrying a documented physics error (linear
volumetric mixing of PEF rather than U — a 0.30 b/e systematic residual, exactly 1.0× the default
`SIG_PEF`, biasing clay volume upward, with a complicit test that forward-models the same wrong law
so it passes by construction). **Verified fixed 2026-08-07:** `retired_module()` at
`modules.rs:403` and the guard at `modules.rs:418` now make `run_module` refuse it with an
actionable message rather than silently running superseded physics. The spec stays in the catalog so
a saved step resolves by name and can explain itself. This is the correct shape for a retirement and
chapter `MIN` (`13_mineral-solver.md`) should treat it as the pattern.

### 4.4 Field-scale operation

*Shipped.* Workflow chains (ordered module lists across many wells, sequential steps with
rayon-parallel wells, pollable progress and cancellation, per-step and per-well parameter
overrides); Monte Carlo uncertainty with Latin hypercube sampling and Iman-Conover correlation,
producing P10/P50/P90 net/NTG/PHIE/SWE/HPV per zone, with tornado and Spearman sensitivity; a field
dashboard aggregating pay summaries across every well; cutoff sensitivity sweeps; well groups as a
global filter. Output naming is centralised — `workflow::resolve_output_names` is the single place a
written curve is named, two outputs resolving to one name are refused, and `STANDARD_COLUMNS` is the
one shadowing-refusal list.

*The one measured performance figure in the product:* a real 100-well × 4-module chain runs in
**21 s**, improved from ~50 s by removing the `computed_curves` primary key and batching each well's
output into a single DELETE plus one Appender.

*v2 addition — the one measured figure at real scale, and it is not good:* a real field project
reported at **2.5 GB, 6 GB RAM, a 15-minute open**, with a 540-well project taking "minutes for the
window to appear". Mitigations shipped (a memory cap, Compact Project, engine-copy Save As, a boot
report) but the underlying cost is unaddressed. See §7.1 and `SB-CORE-030`.

### 4.5 Interpretation and QC visuals

*Shipped.* WebGPU multi-track log views with synchronised hover, layouts, facies block tracks,
point-data and array-log tracks (band/spaghetti/heatmap), image tracks and crossover shading;
histogram, crossplot, Pickett and correlation panels with linked brushing and multi-well context
overlays; **19 vector-digitized chart overlays**; interactive Thomas-Stieber with draggable endpoints
writing zone parameters; a Results-QC panel (Sw spread, Buckles, unity checks, per-zone scorecard);
Lorenz, Thomeer, HFU and cutoff-sensitivity panels; an embedded Vega-Lite panel with a spec editor.

### 4.6 Deliverables

*Shipped.* Composite log plots at true print scale (1:200/500/1000) exporting vector SVG and a
dependency-free multi-page PDF; a report generator (cover, editable methodology table, zone
parameters, pay summary, composite pages) with batch export per well; LAS export; xlsx, pptx and
docx twins; PNG and true-vector SVG/PDF export from the chart panels.

### 4.7 Workspace and shell

*Shipped.* Dockview docking workspace, Office-style ribbon (Project / Data / Condition /
Petrophysics / Advance / Plot / View), named sessions, processing history, crash safe-mode and
autosave, a read-only SQL console, a database inspector with undo, and eight themes. UI language:
English, Bahasa Indonesia, Basa Sunda, Javanese, with technical terms deliberately left in English.

### 4.8 PLANNED — in the roadmap, not in the app

`ROADMAP.md` stands at **117 `[x]`, 43 `[ ]`, 2 `[~]`** (2026-08-07; v1 recorded 55/13/2). Open work
spans: a read-only connection pool (`[HIGH-RISK]`, unsignable without a live 100+ well run); lazy
catalog loading and a decimation cache; UI responsiveness during full-field runs; **a 2000-well
stress fixture**; missing-curve synthesis; auto-picks and auto-zonation; NMR; image logs ("largest
single item"); nonlinear Sw inside the SandiMin solve loop; Monte Carlo per-zone distributions and
finalize-to-curves; user-authored Python modules; the 2D map window; an installer; and a user guide.

**The 2000-well fixture is not a housekeeping item** — see §7.1.

### 4.9 The vision that is not the product — and is deliberately not meant to be

> **Confirmed by Jauhar, 2026-07-29:** accepted as written, and the two-agent automation vision is
> **allocated to SegaraBumi**, not to SandiBumi. It is a deliberate product boundary rather than a
> gap — see non-goal §5.7.

`docs/sandibumi_maturation_prompt.md` describes SandiBumi as a **two-agent architecture** — Agent 1
for large-scale conditioning of 1000+ multi-vintage wells, Agent 2 for automated zonation, parameter
identification and interpretation — backed by a decision playbook and a queryable parameter
knowledge base built from 50+ past projects.

**None of that is shipped, and no part of this document should imply otherwise.** What exists is a
*manual* interpretation workstation with excellent batch execution: the interpreter chooses the
modules, the parameters and the zones, and SandiBumi runs them across the field quickly and
reproducibly. The automation layer is the roadmap's frontier, not the current offering.

**What that decision buys.** First, SandiBumi 1.0 gets a scope that can close — automation is the
most open-ended thing on the roadmap, and moving it out is what makes a 1.0 gate arithmetically
possible. Second, the two products get a clean division of labour: SandiBumi is the deterministic,
defensible, *auditable* interpretation engine, and the automation layer lives where a wrong answer
is a suggestion rather than a number in a reserves report.

**What it still requires of this document.** The vision must not appear in SandiBumi's
customer-facing copy, in any form, until it is a shipped SegaraBumi capability with a defined seam.
This is the most likely violation of that rule, because the vision is genuinely compelling and lives
one file away. Positioning SandiBumi today means selling a fast, reproducible, provenance-carrying
manual workstation — which is what it is, and is enough.

---

## 5. Non-goals

Each carries its reason, because a non-goal without a reason is a to-do nobody reached yet.

1. **Not a seismic interpretation package.** The input is well logs. Adding seismic means a volume
   data model, a rendering problem and a whole second validation surface, for a user who already
   owns a seismic package.
2. **Not a reservoir simulator.** The deliverable is the property model that *feeds* a simulator.
   Owning both doubles the validation burden for no additional buyer — the same person buys both,
   from different vendors, on purpose.
3. **Not a real-time, wellsite or geosteering tool.** Every design decision assumes an
   after-the-fact dataset on local disk: a single embedded file, a single writer, batch chains
   measured in seconds-to-minutes. Real-time streaming would invalidate that architecture, not
   extend it.
4. **Not a corporate data-management or master-database product.** A project is one file on one
   interpreter's machine. Becoming the system of record means multi-user access control, audit and
   retention — a different product, sold to a different buyer (see §7.6).
5. **Not a core-laboratory workflow system.** Core, XRD and petrography data are *imported as
   calibration and validation inputs*. Managing the laboratory workflow that produces them is a
   separate domain.
6. **Not a general-purpose analytics platform.** The Vega-Lite panel and the SQL console are escape
   hatches for an interpreter who needs one plot the product does not have. They are not a promise
   to be Spotfire, and must not be positioned as one.
7. **Not an automated interpreter.** Auto-zonation, automated parameter identification, the decision
   playbook and the queryable parameter knowledge base are **SegaraBumi's scope** (Jauhar,
   2026-07-29 — §4.9). SandiBumi's promise is that the *interpreter* decides and the software
   executes across the field quickly, reproducibly and with provenance. The reason this is a boundary
   rather than a backlog item: an automated pick that is wrong is wrong silently and at field scale,
   which is the exact failure mode this product's data-honesty discipline exists to prevent.
   Automation belongs where its output is labelled as a suggestion.

**v2 adds an eighth, and it is a licensing boundary rather than a taste one:**

8. **Not a re-implementation of any Tier-C proprietary capability.** Experienced Eye / EEFS, Domain
   Transfer Analysis, Omovie Sonic Saturation (US 12,242,011 B2), entropy image speed-correction,
   vendor neural-network weight sets, Textural Facies tile encoding and frequency-domain dispersion
   fits are named, understood at capability level, and **never built, approximated or
   reverse-engineered**. Where a genuine user need sits behind one, SandiBumi may ship a
   *design-around* derived from its own primary sources and labelled as such. This is a permanent
   policy, not a phase.

---

## 6. Differentiation, honestly

### 6.1 Where SandiBumi genuinely differs

- **Field-scale batch is native, not bolted on.** Workflow chains, batch Monte Carlo and the field
  dashboard were built as the primary interaction, with single-well views serving them.
- **Jauhar's own method suite is implemented and specified.** SSC/SSPW, LRLC RtC and IMTS exist here
  with their math banked in `docs/`. These are not available in the incumbent suites.
- **Deployment simplicity.** One installer, embedded DuckDB, no server, no external database, no
  licence server contacted at run time. For an asset team without dedicated geoscience IT support
  this is a material difference.
- **Provenance is structural.** Versioned log sets, `log_sets` provenance rows, a computed-curve
  catalog and a processing history are in the data model rather than in convention.
- **Local-language UI** (Bahasa Indonesia, Basa Sunda, Javanese) with technical terms deliberately
  preserved in English. No incumbent will ever do this.

**v2 adds two, both earned by the cross-tool work and neither available to a competitor:**

- **Cross-tool parameter divergence is surfaced, not hidden.** Where the three incumbents ship
  different values for the same constant, SandiBumi can show the interpreter that they do, with
  sources. See `03_EVIDENCE_BASE.md` §14.2.
- **Validity conditions are enforced data, not documentation.** The recurring failure mode across all
  three tools is computing outside a method's stated validity and returning a plausible number. At
  least one vendor ships those conditions as machine-readable manifest columns; carrying them as
  enforced preconditions is cheap and unmatched.

### 6.2 Where it deliberately does not compete — state this before a buyer discovers it

- **Ecosystem and integration breadth.** The incumbents integrate with corporate data stores, seismic
  packages and geomodelling suites. SandiBumi reads files.
- **Track record and institutional acceptance.** A partner or regulator expecting an
  IP/Techlog/Geolog interpretation is expecting a known quantity. SandiBumi has no such history.
- **Breadth of niche methods.** 51 modules is substantial; a mature suite carries several times that
  across acoustics, image logs, NMR inversion, formation testing and more.
- **Image log and full NMR inversion workflows** are not present. Both are roadmapped; neither is
  built.
- **Support organisation.** One person, in one timezone. This is a real product attribute, not a
  detail to be discovered after the sale.

**The rule this section encodes:** a licensed product that claims parity it does not have gets found
out in the first evaluation, and every other claim it made becomes suspect at the same moment. An
admitted gap costs a feature; a discovered overclaim costs the deal.

---

## 7. Non-functional requirements

Every figure is marked **measured**, **target**, or **unmeasured**.

### 7.1 Scale

| Claim | Status |
|---|---|
| 100-well × 4-module chain | **measured**: 21 s |
| Real field project, 540 wells | **measured, and bad**: 2.5 GB on disk, 6 GB RAM, ~15-minute open, "minutes before the window appears" |
| 1000+ well corpus (Jauhar's validation set) | **target** — no benchmark recorded in-repo |
| "2000+ wells" (`README.md`, `CLAUDE.md`) | **unmeasured** — the 2000-well stress fixture remains an open roadmap item |
| **Portfolio scale — thousands of wells across many fields** | **v2 scope decision. Unmeasured.** Requirements in `SB-CORE-030`…`036` |

**This remains the most commercially dangerous line in the document, and v2 makes it worse rather
than better**, because the scope target moved up while the measurement did not. The 2000-well figure
is stated in customer-facing copy today and has never been demonstrated. Exactly one of two things
must happen before a paid release: build the fixture and demonstrate it, or remove the number from
all customer-facing text. Leaving it ambiguous is the only unacceptable option.

**There is no benchmark harness anywhere in the project.** Every performance number in this document
is a single one-off measurement or a static estimate. That is itself a v1.0 gate item —
`SB-CORE-031`.

### 7.2 Performance

Measured: the 100-well chain, and the 540-well open. Everything else — UI responsiveness during
full-field runs, catalog loading at scale, plot redraw at high sample counts — is **unmeasured**.
Known cost centres, each documented in the code or a sweep and none of them benchmarked: a
per-realization Monte Carlo copy cost estimated at ~7.5 MB and ~60 allocations, giving roughly 40 GB
of memcpy for one 5,000-iteration study on one well; 36 string hashes per depth sample in the SSC
loop, estimated at 65–70 % of that loop; three full line-scans of every LAS file on import;
`solve_bounded_lsq` called up to ~800,000 times for a 30-well SandiMin run at typical development-field sample density **with
the global database mutex held for the whole inversion**; and 64 of 82 synchronous Tauri commands
taking that same mutex on the main event-loop thread.

### 7.3 Install footprint

**Measured from configuration:** `productName` SandiBumi, `identifier` `com.sandibumi.petro`,
version `0.1.0`, bundle `targets: "all"`. No external database, no application server, no runtime
framework — DuckDB is compiled in as a bundled Cargo feature, and Windows 11 ships the WebView2
runtime.

**The exception, stated plainly because it is a sales objection: Python.** Three shipped capabilities
require a Python 3.10+ interpreter on the client machine — user equations (`numpy`), DLIS import
(`dlisio`), and the entire ML suite (`scikit-learn`) — plus, since v1, xlsx/docx export
(`xlsxwriter`, `python-docx`) and petrography plate-workbook extraction. Discovery order is
`SANDIBUMI_PYTHON` → `%LOCALAPPDATA%\Programs\Python\Python31x` → `PATH`. The design is deliberately
fail-soft: a missing interpreter degrades exactly those features and never prevents the app
launching. **The engineering is correct and the procurement problem remains:** "install Python and
five packages on every seat" is a request many managed Windows estates will refuse or delay.

*Not a product constraint:* the MSVC 14.29 toolchain pin. That is a broken toolset on one development
machine. No customer will encounter it; it belongs in `CONTRIBUTING.md`.

### 7.4 Offline capability

**Measured: the application is fully offline-capable.** See §7.5 for the evidence and its limits. No
feature requires connectivity, including licence checks, because no licensing exists yet (§8).

### 7.5 Data security posture

**The claim:** client well data never leaves the machine.

**How that was checked, and what the check does and does not cover.** Three independent searches:

1. **No HTTP client is compiled in.** `src-tauri/Cargo.toml` contains no `reqwest`, `hyper`, `ureq`,
   `curl`, `tungstenite` or `tauri-plugin-http`.
2. **The frontend makes no network calls.** No `fetch(`, `XMLHttpRequest` or `new WebSocket`
   anywhere in `src/`. The only external URLs present are an SVG namespace literal and a Vega-Lite
   `$schema` declaration — descriptive metadata the renderer does not fetch.
3. **No auto-updater is configured.** `tauri.conf.json` has no updater block and no endpoints.

**FIXED 2026-07-29 — the granted-but-unused capability is gone.** `tauri-plugin-opener` was
registered and permitted via `opener:default`, which would have let the app hand a URL or path to the
OS. It had zero call sites — but a granted capability the product does not use is exactly what an
enterprise security review asks about. Removed at all four layers, with a comment at the registration
site recording that re-adding it means revisiting this section.

**Limits of that evidence, stated so the claim stays defensible:** it covers first-party code and
declared dependencies, not transitive dependency behaviour; and the Python subprocess is a separate
process whose temp-file behaviour has not been audited for client data residue.

**One weakness fixed, one still open:**

- **FIXED 2026-07-29 — the webview now has a Content Security Policy.** It was `"csp": null`, which
  mattered here in particular: a fixed LAS-well-name XSS-to-RCE vector had reached the DOM through a
  hostile well name in an imported file. That hole was closed, but a null CSP meant no second line of
  defence behind input sanitisation — and untrusted text arrives with every imported file. Two
  relaxations are deliberate: `script-src` keeps `'unsafe-eval'` because Vega 5 compiles chart
  expressions through the `Function` constructor, and critically this does **not** re-open the
  original class, because inline handlers require `'unsafe-inline'`, which is absent; `style-src`
  keeps `'unsafe-inline'` because CodeMirror injects a `<style>` element at runtime and style
  injection is not a script-execution vector. **Verification caveat: this cannot be exercised by
  `npm run tauri dev`** — with a `devUrl` the webview loads Vite directly and Tauri does not deliver
  the policy. Proving it needs `npm run tauri build` and a click-through of the Vega panel, the
  equation editor and plot printing.

- **STILL OPEN — the project database is unencrypted at rest.** No encryption, cipher or AES code
  exists in the backend; the `.duckdb` file is readable by anything with filesystem access. For a
  laptop carrying a client's whole field, this will be asked about. **Deliberately not fixed:**
  encryption at rest is a feature, not a hardening tweak, and its hard part is key management — a
  lost key means months of interpretation work is unrecoverable, which is a worse outcome than the
  risk it mitigates. It needs a designed answer (where the key lives, how it is recovered, what
  happens on a forgotten passphrase) before any code. Carried as R7.

**v2 additions, both found by the 2026-08 audit and both open:**

- **`save_png` is an unrestricted arbitrary-path file write callable from page JS**, and its own doc
  comment asserts a "whitelisted-write pattern" that does not exist anywhere in the backend. A doc
  comment claiming a control that is absent is worse than no comment. Carried as **R12**.
- **A third-party licence *inventory* now exists** (`THIRD-PARTY-LICENSES.md`, generated by
  `tools/gen-third-party-licenses.mjs`, distributed dependencies only). It closes half of what
  `IP_PROVENANCE.md` §2.6 recorded as "no licence audit has been performed" — the file itself is
  careful to say it is *"a factual inventory, not legal advice."* An inventory is not an audit; R13
  carries the remainder.

### 7.6 Deployment model

**Today:** single-user Windows desktop, one project per `.duckdb` file, one writer
(`Mutex<Connection>` — fundamental by design, not a defect). **Named future state:** a shared
multi-user backend. Not built, not scheduled, and deliberately not foreclosed — non-goal §5.4 is the
boundary that keeps "several people share a project" from silently becoming "we are the corporate
system of record."

### 7.7 Quality assurance posture — as a product attribute

| | Status 2026-08-07 |
|---|---|
| Backend automated tests | **775 `#[test]` functions across 54 files**, plus real-field-data pipeline tests |
| Frontend automated tests | **none in the green gate** — an optional Playwright E2E harness exists (77 tests, 20 specs) and is explicitly *"never part of the green gate"*; it has no pixel assertions by design and does not kill the app process |
| Lint configuration | **none** — no `clippy.toml`, no `[lints]` table, no `#![deny(warnings)]`, no `rust-toolchain.toml`, no eslint |
| Continuous integration | **none** — no `.github/workflows` |
| The green gate | `tools\check.ps1`, run manually |
| Fresh-clone buildability | **broken** — an `include_bytes!` test fixture is gitignored and untracked, so `cargo test` cannot compile from a fresh clone |
| Independent audit | three adversarial audits on file (2026-07-20, a 24-agent full-tool QC pass 2026-07-21, and the F1–F5 sweeps 2026-07-24 producing 148 raw findings, 125 surviving verification) |

The asymmetry matters: **the tested half and the historically buggy half are different halves.** The
775 tests defend the physics; the frontend is where the wrong-well, stale-data and lifecycle defects
have actually occurred. The adversarial audit practice is a real and unusual strength for a solo
product and partly compensates — but it is a process, not an artefact, and cannot be handed to a
buyer as evidence.

**v2 adds a warning about the sweep material itself**, because it will otherwise be quoted wrongly:
the F1–F5 sweeps have **no per-finding IDs** (their `F1a`…`F5e` tags are dimension labels reused
across every finding in that dimension), **no open/fixed/deferred field**, and **heading severities
that were never edited after the verifier pass** — F1 alone carries 10 downgrades, F3 carries 16, and
F5 carries three proposed fixes the verifier rejected as ineffective. Their own headline counts
disagree with their bodies in at least a dozen places. **Any document quoting the headings or
priority tables alone will overstate severity and mis-specify three fixes.** Chapters cite finding
bodies, never headings.

### 7.8 Localisation

Shipped: English (source language), Bahasa Indonesia, Basa Sunda, Javanese, via exact-phrase
dictionary lookup with a MutationObserver. Technical terms (Thin Beds, Monte Carlo, Pickett,
mnemonics) remain English by explicit design decision.

---

## 8. Commercial surface

**Nothing in this section is implemented.** No licence key, activation, entitlement or trial code
exists anywhere in `src/` or `src-tauri/src/`. The commercial surface is entirely undecided, which is
the correct state for version 0.1.0 and an unacceptable one at 1.0.

These are decisions Jauhar owes, with consequences, not recommendations:

| Decision | Options | Consequence |
|---|---|---|
| **Licence unit** | per named user · per machine · per site/asset team | A field-scale tool used by 2–3 specialists per asset team makes per-site simplest to sell and hardest to price; per-machine is easiest to enforce and irritates users with a laptop and a workstation |
| **Activation** | none (trust) · offline key file · online activation | §7.4's fully-offline property is a genuine differentiator for air-gapped or restricted estates. **Online activation would destroy it.** An offline signed key file preserves it |
| **Perpetual or subscription** | perpetual + maintenance · annual subscription | Subscription requires a support organisation that does not exist yet. Perpetual-with-maintenance matches a one-person vendor better and caps the promise |
| **Update delivery** | manual download · in-app updater | An in-app updater means re-introducing network egress and losing the clean §7.5 answer. Weigh it against the convenience honestly |
| **Support commitment** | best-effort · defined response time | Any response-time commitment from a single person needs an explicit working-hours and holiday boundary, in writing, before the first sale |
| **Version support window** | latest only · N-1 · N-2 | Directly drives the project-file compatibility policy |

**One decision is more urgent than the rest:** whether Python stays a customer prerequisite, gets
bundled with the installer, or whether the affected capabilities become an optional add-on module.
That choice changes the installer, the IT conversation and the feature matrix simultaneously.

---

**Continues in `02_RISKS_AND_CONTRADICTIONS.md` — §9 risk register, §10 open questions, §11 where the
documents disagree with the code.**

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
