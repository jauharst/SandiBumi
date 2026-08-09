# PRD v2 · §15 — Cross-cutting requirements (`SB-CORE`)

These are the requirements no single domain owns. **Domain chapters may cite them; only this file
allocates them.** A chapter that needs a new `SB-CORE` id proposes it here rather than minting it.

Format, priority scale, status vocabulary and RFC-2119 verb usage are defined in `CONTRACT.md`
§1.3–§1.4 and §3. In summary: **P0** blocks the first sale, **P1** is required for 1.0, **P2** is
1.5, **P3** is v2, **P4** is horizon. Status is one of `ABSENT` / `PARTIAL` / `PRESENT-OK` /
`PRESENT-DIVERGENT` / `PRESENT-UNVERIFIED`.

---

## 15.1 Correctness and honesty — P0

*These seven are the reason `06_SEQUENCING_AND_GATES.md` inserts a tier below Tier 0. Nothing this
product is sold on can be claimed while any of them stands open.*

**Three of the seven — `SB-CORE-002`, `-006`, `-007` — are the same underlying failure seen from three
angles: the product is internally inconsistent in ways the user cannot see.** They are listed
separately because they need different fixes and different tests, not because they are different
problems.

### `SB-CORE-001` — Depth unit is a first-class, carried property   [P0] [PARTIAL]

**Requirement.** SandiBumi MUST carry the depth unit of every log frame from import through to every
computation and every export. It MUST NOT discard a declared depth unit. Any module whose maths
depends on the depth unit MUST read it rather than assume metres, and MUST refuse to run when it is
undeclared.

**Correction, 2026-08-07 — this requirement was recorded as `ABSENT` and that was wrong. Re-verified
at source, line by line.** `15_sat-height-rocktyping.md` escalated that the status text was stale for
its domain and, correctly, did not edit this file. The earlier rationale claimed the carrier did not
exist, that the frontend held no occurrence of `depth_unit`, and that capillary-pressure code
multiplied by 3.28084 on a metres assumption. **The first two are no longer true and the third is
half true.**

**What is closed, verified:**

- The carrier exists — `units.rs:38` `DepthUnit`, `units.rs:116` `to_feet(value, from)`.
- **The production module-run path carries it.** `workflow.rs:420-422` reads
  `crate::units::project_depth_unit_or_default(&conn)` and `workflow.rs:595` passes it into
  `ModuleContext`. The only other two `ModuleContext` constructions that hardcode a unit sit at
  `workflow.rs:2921` and `:3262`, both **after** the `#[cfg(test)]` boundary at `workflow.rs:1554` —
  test scaffolding, not production.
- **The Leverett capillary-pressure law is unit-correct.** `satheight.rs:189` computes
  `PSI_PER_FT_PER_SG * (rho_w - rho_hc) * crate::units::to_feet(h, ctx.depth_unit)`, with the fix
  recorded in the comment at `satheight.rs:185-188`. It is regression-tested at `satheight.rs:246`,
  which describes one physical well twice — 100 m and the identical 328.084 ft — and asserts an
  identical answer.
- **The frontend has it.** `lib.rs:3136-3137` register `get_project_depth_unit` and
  `set_project_depth_unit` as Tauri commands.

**What is still open, and it is worse than the closed half:**

1. **The Skelt-Harrison branch converts nothing at all.** `satheight.rs:175` evaluates
   `1.0 - a * (-(b / (h + dd)).powf(c)).exp()`, where `b` is `SH_B` and `dd` is `SH_D` — both declared
   **`"m"`** at `satheight.rs:117` and `:119`, with the module doc at `satheight.rs:101` stating
   "h in metres". The height `h` arrives in whatever unit the project declares. No `to_feet`, no
   guard. `15_sat-height-rocktyping.md` quantifies the divergence at **up to 47.7 saturation units**
   in the transition zone; that magnitude is the chapter's, not independently recomputed here, but
   the mechanism is verified.
2. **The regression test cannot catch it.** `satheight.rs:251` pins `("OPT_SWH", "LEVERETT")`, so the
   test that exists exercises only the branch that was already fixed. **A green test is currently
   evidence for the wrong half of the requirement** — which is more dangerous than no test, because
   it reads as coverage.
3. **An undeclared unit defaults instead of refusing.** `units.rs:179-180`
   `project_depth_unit_or_default` ends `.unwrap_or(DepthUnit::Metres)`. The comment at
   `units.rs:176-178` gives a considered reason — `wells.kb`/`td` and the Field Map's UTM coordinates
   are already stored in metres — so this is a deliberate choice, not an oversight. It nonetheless
   contradicts this requirement's final sentence and leaves `SB-CORE-T02` open. **Jauhar's call:
   amend the requirement to permit a declared, surfaced default, or make the refusal real.**
4. **The LAS writer hardcodes metres, and the file fails SandiBumi's own round trip.**
   `export.rs:77-79` writes `STRT.M`, `STOP.M`, `STEP.M` and `:85` writes `DEPT.M` — four metre
   labels, with no reference to the project's declared unit. `ingest.rs:162` by contrast is careful:
   it reads `project_depth_unit(conn)`, parses the file's own declared unit, and reconciles them
   through `resolve_index_unit`. **So a foot-declared project exports foot-valued depths labelled
   metres, and SandiBumi's own reader then divides by 0.3048 on re-import.** The product emits a file
   it cannot itself read back. Reported by `21_data-io.md`, verified here. The existing round-trip
   test cannot catch it because its fixture project is declared in metres.
5. **The DLIS path has no unit handling at all.** `dlis.rs` contains **zero** `units::` calls at any
   line — verified by direct count. Its sidecar builds a unit map across channels but skips the
   index, so every *curve* unit is correct while the *depth* is wrong by 3.28×. That is the worst
   available combination, because the curves being right is exactly what stops anyone looking at the
   depth.

**The pattern under items 4 and 5 is the finding, not the two instances.** `21_data-io.md`'s central
result is that SandiBumi's **read** paths are careful — in two places ahead of every incumbent —
while its **write** paths are not. Unit discipline was applied where data enters and never where it
leaves. Any remedy that fixes only these two sites treats the symptom; the requirement's scope
sentence already says "through to every computation **and every export**", and it is the export half
that is unbuilt.

**Why it stays P0 at `PARTIAL`.** The status improved; the risk did not, because the surviving defect
is the one that is invisible. Every other defect in this register produces a *visible* failure or a
*disclosed* limitation. This one produces a wrong number that plots correctly, exports correctly, and
enters a client's reserves calculation. It is the exact failure mode `01_PRODUCT.md` §3.1 names.

**This is the second time the spine has been stale where a chapter was right** — `SB-CORE-002` was
the first. Both were caught by a chapter re-reading the source rather than trusting this document.
That is the stale-measurement pattern §11 names, committed twice by the document that names it, and
it is the argument for the machine verification sweep before sign-off.

**Verified by.** `SB-CORE-T01` — a foot-declared project reproduces a metre-declared project's Pc to
within tolerance. **`SB-CORE-T01b` (new)** — the same equivalence asserted on the **Skelt-Harrison**
branch, which `satheight.rs:246` does not cover. `SB-CORE-T02` — an undeclared unit refuses rather
than defaults, pending the ruling in open item 3.

**Owning chapters.** `15_sat-height-rocktyping.md` (the arithmetic), `21_data-io.md` (the parse and
carry), `23_plotting-interactivity.md` (the renderer constant).

---

### `SB-CORE-002` — A degraded or failed result is never presented as a clean one   [P0] [PRESENT-OK]

**Requirement.** Every computation path MUST propagate failure to the surface that reports it. A
batch operation MUST report per-item failure counts. A result composed of NaN MUST NOT be reported as
a success. A section omitted from a deliverable because its computation failed MUST be reported as
omitted, in the deliverable and in the run record.

**Rationale.** This is the product's own stated cardinal rule, and the 2026-08 as-built audit found it
violated in seven shipped paths (R15).

**Adjudication, 2026-08-09.** The original seven are recoverable without inference from the recorded
R4, R18, R19 and R21 findings in `docs/playbook_build_progress.md`. Current source and existing tests
were then checked against those seven exact paths. A test counts here only when it asserts the
reported artefact; a test of an internal helper, an internal `Result`, or the persistence side effect
alone is supporting coverage, not the acceptance test for this requirement.

The earlier claim that **four remain open** is not supported by current evidence. All seven recorded
behaviors are now closed and regression-locked at their reporting surfaces. Six named tests pin the
already-shipped corrections; `SB-CORE-T07` first reproduced the remaining report-batch defect, then
closed it by carrying the Pay Summary degradation beside the still-written PDF in the batch result.
The requirement is therefore `PRESENT-OK`.

Each original path now owns exactly one non-overlapping reporting-surface contract:

| Test | Original recorded violation | Reporting-surface acceptance contract | Current adjudication and evidence |
|---|---|---|---|
| `SB-CORE-T03` — `a_monte_carlo_chain_failure_is_reported_in_the_job_and_never_as_a_zero_uncertainty_result` | R4: Monte Carlo swallowed module errors and presented an all-NaN/all-zero uncertainty result as success. | Given one selected well whose chain step fails on every realization, the returned Monte Carlo result MUST name that well and the underlying module error, and the job item MUST be `Failed`; no clean P10/P50/P90 zero table may be reported. | **CLOSED — regression test present.** `core_reporting_tests.rs` asserts the returned error and failed job item, with an explicit successful-gate control. |
| `SB-CORE-T04` — `a_partial_generic_curve_import_returns_a_named_warning_while_the_standard_curves_remain_successful` | R4: a failed generic-store full-curve import was written only to stderr and omitted from `ImportResult`. | Given a delivery whose six standard curves load but whose full-curve load fails, the returned per-file `ImportResult` MUST remain a successful well import and MUST carry a warning naming the partial load and its cause; a clean sibling import MUST carry no such warning. | **CLOSED — regression test present.** `ingest.rs` asserts the standard rows and successful result survive while the returned warning names the partial full-curve load; a clean sibling carries no such warning. |
| `SB-CORE-T05` — `an_uninterpreted_pay_summary_renders_absent_values_while_a_real_zero_net_zone_renders_zero` | R4: the pay summary fabricated Net, N/G and HPV zeros for a well with no classified samples. | Given one uninterpreted row and one evaluated row whose genuine result is zero, the rendered summary MUST show absent marks plus the not-classified explanation for the first and numeric zeros for the second. | **CLOSED — regression test present.** `frontend-acceptance.test.mjs` drives `renderPaySummaryTable` and inspects both rendered rows plus the explanation. |
| `SB-CORE-T06` — `a_partial_ml_run_reports_the_written_count_and_an_all_failed_run_writes_no_success_history` | R4: the ML dialog claimed every scoped well when only a subset was written. | Given two scoped wells with one successful write, the visible status and permanent History MUST report one of two written and one needing attention; given zero successful writes, no success History entry may be recorded. | **CLOSED — regression test present.** `frontend-acceptance.test.mjs` inspects the visible status, global status and real process-history store for partial and all-failed runs. |
| `SB-CORE-T07` — `a_failed_pay_summary_is_named_in_the_pdf_and_in_the_batch_run_record` | R18: report generation silently omitted the Pay Summary on error and reported zero batch errors. | Given a forced Pay Summary error for one well, the emitted PDF MUST contain the Pay Summary heading and its error note, while the returned batch/run record MUST name that well's degraded section and report one failure; the composite PDF may still be listed as written. | **CLOSED — regression test present.** `report.rs` forces a write-only Pay Summary failure, inspects the emitted PDF, and asserts one named degradation beside the written file in the returned batch record. |
| `SB-CORE-T08` — `a_stats_only_dashboard_run_says_no_flag_curves_were_written` | R19: the Field Dashboard claimed `FLAG_*` curves were written on its stats-only, write-nothing path. | After a stats-only Dashboard computation, the visible status MUST say that no `FLAG_*` curves were written and MUST name the action that persists them; it MUST NOT claim that curves were written. | **CLOSED — regression test present.** `frontend-acceptance.test.mjs` inspects the Dashboard status element for the refusal and the separate persistence action. |
| `SB-CORE-T09` — `a_training_well_that_contributes_no_samples_is_warned_in_the_rendered_ml_result` | R21: supervised ML silently dropped selected training wells that contributed zero usable samples. | Given two selected training wells where one contributes no usable samples, the rendered ML result MUST warn that one of two contributed nothing, name the no-usable-samples condition, and say that the model used the remaining one; when both contribute, that warning MUST be absent. | **CLOSED — regression test present.** `frontend-acceptance.test.mjs` drives `renderResults`, inspects the rendered warning, and asserts the clean control has none. |

The contracts above derive only from the seven recorded violations. They do not extend
`SB-CORE-002` to new failure classes, and none treats today's internal behavior as the expected
reporting artefact.

`03_EVIDENCE_BASE.md` §14.3 cannot be claimed as a differentiator while these stand. Selling
"fail-loud where the incumbents fail silent" from a product that fabricates zeros is the single
fastest way to convert a strength into a credibility loss.

**Verified by.** `SB-CORE-T03`, `SB-CORE-T04`, `SB-CORE-T05`, `SB-CORE-T06`, `SB-CORE-T07`,
`SB-CORE-T08`, `SB-CORE-T09`, as assigned above.

**Owning chapters.** `14_cutoffs-summation-mc.md` (Monte Carlo, pay summary), `22_database-model.md`
(job results), `23_plotting-interactivity.md` (report emission).

---

### `SB-CORE-006` — One name, one equation   [P0] [PRESENT-DIVERGENT]

**Requirement.** A method name MUST identify exactly one equation across every engine that offers it.
Where the standalone module library and the SandiMin solver both expose a named model, they MUST
return the same number for the same inputs, within a stated numerical tolerance. Where a vendor's
label for a method differs from the literature's, SandiBumi MUST record both and MUST NOT silently
adopt one — the emitted method flag, the UI label, the doc comment and the equation MUST agree.

**Rationale.** SandiBumi ships two saturation engines and they disagree about what "Simandoux" means.
`modules.rs:2279-2283` (`sw_sim`, default `OPT_SIM=MODIFIED`) computes `Ct = φe^m·Sw^n/(a·Rw) +
Vsh·Sw/Rsh` — the Bardon-Pied form, no `(1−Vsh)` divisor — which correctly matches Geolog's own
`MODIFIED` label (T1, `sw_sim.lls:207-218`). `multimin2.rs:174` computes `φe^m/(a·Rw·(1−Vsh))` — the
Schlumberger form — while its doc comment at `:164` and its enum comment at `:115` both call it
*"Modified Simandoux (Bardon-Pied)"*. **The label and the equation are two different methods.** A
user running the module at its default and `SwModel::Simandoux` in the solver on the same well gets
results **7.3 saturation units and ~19 % HCPV apart** (`12_saturation.md` §2.2, §3.1).

The naming is genuinely inverted *between vendors* — Geolog's `MODIFIED` is IP's plain "Simandoux" —
so the trap is not SandiBumi's invention. Reproducing it inside one product is.

**Why it is P0 rather than hygiene.** It defeats `SB-CORE-010` at the exact point lineage is supposed
to pay: the deliverable records a method *name*, and that name no longer determines the equation. A
provenance record that says "Simandoux" is, today, ambiguous by 7.3 su. Selling reproducibility from
a product in that state is `01_PRODUCT.md` §6's overclaim rule in action.

**Verified by.** `SB-CORE-T17` — for every model exposed by both engines, module and solver agree on a
shared fixture within tolerance. `SB-CORE-T18` — every method's emitted flag curve, UI label and doc
comment resolve to the same equation identifier.

**Domain instance.** `SB-SAT-047` in `12_saturation.md`. **Owning chapters.** `12_saturation.md`,
`13_mineral-solver.md`, and any later chapter whose method is offered by both engines.

---

### `SB-CORE-007` — One definition for every constant and every transform   [P0] [PRESENT-DIVERGENT]

**Requirement.** A petrophysical constant MUST have exactly one definition site. A transform MUST have
exactly one implementation. An **output curve mnemonic** MUST have exactly one producing module, or
every producer of it MUST agree **at its own shipped defaults**, not merely when handed identical
parameters. Where a second copy is unavoidable, a test MUST assert the copies agree, and that test
MUST fail if either copy changes alone.

**Rationale — verified at source 2026-08-07.** The clean and shale gamma-ray endpoints are defined
four times inside one product, with three different clean values and two different shale values:

| Site | `GR_MA` | `GR_SH` | IGR at GR = 70 gAPI |
|---|---|---|---|
| `ssc.rs:95` | 10.0 | 150.0 | 0.4286 |
| `modules.rs:521` | 20.0 | 120.0 | 0.5000 |
| `modules.rs:597` | 15.0 | 120.0 | 0.5238 |
| `modules.rs:2631` | 20.0 | 120.0 | *(normalization reference pair)* |

That is a **22.2 % spread in `Vsh` at a single ordinary GR reading**, decided by which code path the
user happened to enter through. Separately, the eight-transform GR ladder (`LINEAR`, `STIEBER1-3`,
`LARINOV1-3`, `CLAVIER`) is duplicated verbatim at `ssc.rs:57-68` against its `modules.rs` original,
with **no test asserting the two copies agree** — so a corrected coefficient in one is a silent
divergence in the other. The same defect shape appears in the cutoff defaults, copy-pasted into six
panes in two disagreeing sets (`14_cutoffs-summation-mc.md`).

**The sharpest instance, verified 2026-08-07 — a fix applied to one twin and not the other.** The
`ssc.rs` module header states at `:22` that an earlier gas-conditioning weight *"overshot the midpoint
and was fixed 2026-07-29"*, and the comment at `:172-178` explains the fix at length: the old form
*"weighted the pull by 1.6/2 = 0.8 per side, which overshoots the midpoint and **inverts** the D-N
crossover (phid² became 0.2·φD² + 0.8·φN²)"*. **`ssc.rs:433` — the `sspw` twin in the same file —
still runs the 1.6 form**, returning porosity **4.72 p.u. low in gas**. The correct expression is
written down eleven lines above the wrong copy, and the header records the fix without qualification.

This is the strongest argument in the document for the requirement: duplication does not merely risk
divergence, it defeats the fix. Every hour spent finding and correcting the SSC gas weight bought
nothing for `sspw`, and nothing in the tree reports that.

**A second verified instance, and this one is already documented rather than fixed.** The sandstone
matrix density ships as **2.645** at `modules.rs:591`, `:687`, `:775`, `:1280`, `:4092` and as
**2.65** at `modules.rs:1631`, `:2968`, `:2991`, `:3016`, `:4551`, `:4587`, `:4607` and
`lithology.rs:198`, `:342` — while `lithology.rs:201` carries a comment explicitly noting the value
is *"NOT the 2.645 sandstone default the porosity modules use."* The codebase knows about the split
and records it in one location instead of resolving it. `gascorr` (2.65) ships a doc string
instructing the user to chain it with `phi_den` (2.645).

**A third verified instance, 2026-08-07 — two modules, one output mnemonic, 33.1 °C apart.** Two
shipped modules both declare and write a curve named `FTEMP`: `ftemp_grad` (`modules.rs:1025`,
written at `:1055`) and `precalc` (`:1109`, written at `:1170`). At **2 000 m TVD, each run at its
own shipped defaults**, they disagree by **33.1 °C**:

| Module | Defaults | At 2 000 m TVD |
|---|---|---|
| `ftemp_grad` | `TSURF` 26.7 °C (`:1021`), `TGRAD` 0.03 °C/m (`:1022`) | 26.7 + 60 = **86.7 °C** |
| `precalc` | `OPT_TU` = degF (`:1086`), `SURF_TEMP` 77 °F (`:1090`), `TEMP_GRAD` 0.026 °F/ft (`:1091`) | 77 + 0.026 × 6 561.7 = 247.6 °F = **119.8 °C** |

Neither is wrong in its own terms — `precalc`'s doc states plainly that its defaults are one study's
feet-based fits. The defect is that **`FTEMP` does not identify which one produced it.** Every
downstream consumer reads the bare mnemonic (`nphi_env_corr` at `modules.rs:1877`, `gascorr` at
`:1704`, the Rw resolution at `:2021`, `:2129`, `:2236`), so the answer depends on which module ran
last, and nothing in the curve records it. The 33.1 °C propagates through the Arps `Rw(T)` conversion
— ratio (188.1 + 6.77)/(247.6 + 6.77) = 0.766 — and then through `Sw ∝ √Rw` to **14.3 % relative on
`Sw`**. Feed the same TVDSS in feet to `ftemp_grad`'s metric defaults and a third answer appears
(53.9 °C), which is `SB-CORE-001`'s depth-unit problem arriving through the same door.

This is why the requirement above says *at its own shipped defaults*. `SB-CORE-006`'s test `T17`
hands both engines one fixture and checks they agree — and here they would **agree**, because the
two modules compute the same linear trend and the divergence lives entirely in the defaults. A test
whose fixture supplies the parameters cannot see a defaults disagreement. That is `SB-CORE-015`'s
non-default-fixture principle, arriving a second time and from a different direction.

**Relationship to the neighbours.** `SB-CORE-004` requires each of these to carry a source;
`SB-CORE-006` requires one name to mean one equation. Neither catches this: four *sourced* copies of
one endpoint still drift; none of the four sites disagrees about what "linear `Vsh`" *means*; and the
two `FTEMP` producers agree about what a geothermal gradient is while disagreeing by 33.1 °C about
what it is here. This is the third distinct internal-consistency failure the chapters have surfaced,
which is why it gets its own id rather than a footnote on either.

**Why it is P0.** `03_EVIDENCE_BASE.md` §14.1 sells SandiBumi on the incumbents' internal
inconsistency — a constant 192× off its own manual, a coefficient labelled as one mineral in one
module and another elsewhere. The product currently commits the same class of defect against itself.
That is not a gap to disclose; it is a claim that cannot be made until it is fixed.

**Verified by.** `SB-CORE-T19` — a build-time check that no petrophysical constant name is bound to
more than one default. `SB-CORE-T20` — for any deliberately duplicated transform, a test evaluates
both copies on a shared vector and asserts equality. `SB-CORE-T23` — for every output mnemonic
declared by more than one module, each producer is run **at its own shipped defaults** on one shared
depth column and the results are asserted equal within the curve's stated tolerance. The fixture MUST
NOT supply parameters to either module: a fixture that parameterises both cannot see a defaults
divergence, which is the whole failure mode (`SB-CORE-015`).

**Owning chapters.** `10_clay-volume.md` (the GR endpoints and the ladder),
`14_cutoffs-summation-mc.md` (the cutoff panes), `20_envcorr-qc.md` (the dual-`FTEMP` producers,
`SB-ENV-043`), and any chapter whose §5 table finds a second definition site.

---

### `SB-CORE-004` — No parameter ships without a source   [P0] [PARTIAL]

**Requirement.** Every shipped default MUST carry a machine-readable source string. A parameter with
no defensible source MUST ship absent, and a run that requires it MUST refuse until the interpreter
supplies a value. **The build MUST fail if any registered parameter has a default and no source.**

**Rationale.** `03_EVIDENCE_BASE.md` §12.2. The discipline exists on this project as a written
convention and is honoured in the specs, but it is not machine-enforced anywhere, and the thinnest
place is already visible: the pay cutoff defaults are copy-pasted into six panes in **two disagreeing
sets**, with no documented source for either. A convention that is not gated decays at exactly the
moment a deadline arrives.

**The worked example, from the corpus rather than from principle.** Geolog's Simandoux module is the
only member of its own saturation family that defaults `A = 0.8`; every sibling defaults `A = 1`
(`sw_arch`, `sw_indo`, `sw_nige`, `sw_ws`, `sw_dual`, `sw_tot`; `sw_juha` has no `A` at all — T1,
`sw_sim.info`). Re-running the same interval at `a = 0.8` instead of `a = 1` moves Sw from 0.625 to
0.579 — **4.6 saturation units from a default nobody changes.** `sw_sim.info` does carry a References
block naming Simandoux 1963 and Bardon & Pied 1969; what it does not do is attribute **the 0.8
itself** to either paper. That is precisely the gap this requirement closes: a citation on the
*method* is not a citation on the *number*.

**Note on the gate.** The build gate is the requirement, not a nice-to-have implementation detail.
The difference between a convention and a contract is whether a machine enforces it, and the entire
provenance claim in `SB-CORE-010` rests on the source string existing for every parameter.

**Verified by.** `SB-CORE-T10` — a parameter registered with a default and an empty source fails the
build. `SB-CORE-T11` — a module requiring a source-less parameter refuses at run time with an
actionable message.

**Owning chapters.** All eighteen, via each chapter's §5 parameter table.

---

### `SB-CORE-040` — Verification is indexed by capability   [P0] [ABSENT]

**Requirement.** SandiBumi MUST maintain a verification matrix indexed by capability, not by round,
showing for each capability whether it has been exercised against real well data and when. It MUST be
derivable mechanically rather than by reading history.

**Rationale.** `01_PRODUCT.md` §4.0 and R5. At 6.7 % and a backlog growing 250× faster than it
retires, the ratio is going to be asked about, and "read 88 rounds and reconstruct it" is not an
answer during an evaluation.

**Why it is P0 despite being documentation.** It is the single cheapest thing in this document that
changes how the product is perceived. It also changes what can be *decided*: without it, nobody —
including Jauhar — can answer which capabilities are safe to demonstrate.

**Verified by.** `SB-CORE-T12` — the matrix is generated by a committed script from
`REVIEW.md` plus a capability map, and the generation is part of the gate.

---

### `SB-CORE-041` — The tree builds and tests from a fresh clone   [P0] [PRESENT-DIVERGENT]

**Requirement.** A fresh clone MUST build and pass the test suite with no manual fixture placement.

**Rationale.** A test fixture referenced by `include_bytes!` is gitignored and untracked while its
sibling is tracked, so the pair is half-committed and `cargo test` cannot compile from a clean
checkout.

**Why a one-line fix is P0.** It blocks three separate commercial commitments — CI (`SB-CORE-042`), a
second maintainer (R9), and **source-code escrow** (`05_STRATEGY.md` §22.2). An escrow deposit a
third party cannot build is worth nothing, which puts a `.gitignore` entry on the critical path of a
procurement answer.

**Verified by.** `SB-CORE-T13` — CI clones clean and runs the gate.

---

### `SB-CORE-015` — No artifact ships that SandiBumi's own reader rejects   [P0] [PRESENT-DIVERGENT]

**Requirement.** Every file SandiBumi writes MUST be readable by SandiBumi, and MUST re-import to the
same values it exported. Every writer MUST declare units, nulls and index conventions that match what
it actually wrote. A format's export path MUST be round-trip tested against its own import path, on a
fixture whose declared conventions are **not** the writer's defaults.

**Rationale — two independent instances, one external and one internal, which is why this is a
requirement and not a bug report.**

- **The vendor case.** `21_data-io.md` records IP at factory defaults writing a DLIS file that its
  own loader then states it cannot read. This is `03_EVIDENCE_BASE.md` §14.1 in its purest form: a
  three-decade product shipping a file it cannot consume.
- **The SandiBumi case, verified at source 2026-08-07.** `export.rs:77-79` writes `STRT.M`, `STOP.M`,
  `STEP.M` and `:85` writes `DEPT.M` — four hardcoded metre labels, no reference to the project's
  declared unit — while `ingest.rs:162` correctly reconciles the declared and file units through
  `resolve_index_unit`. On a foot-declared project SandiBumi exports foot values labelled metres, and
  **SandiBumi's own importer then divides them by 0.3048.** The product cannot read back what it
  wrote.

**Why P0.** A round-trip failure is not a formatting complaint. The exported LAS is the deliverable —
it is what goes to the client, into their corporate store, and into the next contractor's project.
A file that is wrong *and self-consistently labelled* will be trusted by every tool that opens it,
because nothing in it announces the error. This is the failure mode `01_PRODUCT.md` §3.1 names,
committed at the last step, after every correct computation upstream.

**Relationship to other requirements.** It is the export-side half of `SB-CORE-001`, the
deliverable-side half of `SB-CORE-010`, and it is what makes `SB-CORE-011`'s byte-identical re-run
meaningful outside the application. Listed separately because the fix is neither of theirs: it is a
test discipline — **round-trip every writer against its own reader, on a non-default fixture** — that
no existing requirement states.

**The fixture clause is the load-bearing part.** SandiBumi already has a LAS round-trip test. It
passes, and it cannot catch this, because its fixture project is declared in metres — the writer's
hardcoded assumption. A round-trip test whose fixture shares the writer's defaults tests nothing.

**Verified by.** `SB-CORE-T14` — a foot-declared project exports LAS and re-imports to identical
depths. `SB-CORE-T15` — the same for DLIS, where `dlis.rs` currently makes **zero** `units::` calls.
`SB-CORE-T16` — every shipped writer has a round-trip test whose fixture is non-default.

**Owning chapters.** `21_data-io.md` (the writers and the round trips), `22_database-model.md` (the
export-side provenance record).

---

## 15.2 Reproducibility and provenance — the differentiator

*Axis 1 of `05_STRATEGY.md` §18.1. These are what make the product's central claim true.*

### `SB-CORE-010` — Every computed curve answers "how was I made?"   [P1] [ABSENT]

**Requirement.** SandiBumi MUST record, for every computed curve: the module and its version; every
input curve and its log set; every parameter value **and that value's source string**; the zone
definition; the operator; and the timestamp. The UI MUST show this ancestry for any curve on demand,
**and the ancestry MUST travel into every deliverable that carries the curve's numbers** — report,
export, LAS header, and any downstream artefact.

**Scope resolved 2026-08-07 — the deliverable, not just the UI.** `24_ml-advanced.md` escalated that
this requirement's own text stopped at the UI while `03_EVIDENCE_BASE.md` §14.4 states the thesis as
provenance carried *"through the computation, into the deliverable"*, and that other chapters may
have been citing a guarantee this text did not make. §14.4 is the product thesis and wins; the
narrower phrasing was an omission, not a decision. The escalation is not theoretical — **verified at
source 2026-08-07, `report.rs` and `export.rs` contain zero references to `ml`, `facies`, `cluster`,
`hfu` or `leaderboard`**, so every trained-model number reaching a client today is unreconstructable
by construction. A lineage graph the deliverable cannot see is an internal debugging aid, not the
property this product is sold on.

**Rationale.** `03_EVIDENCE_BASE.md` §14.4, and the roadmap's Phase 11. The foundations are further
along than the roadmap admits — the append-only write discipline, log-set provenance and seeded Monte
Carlo are down-payments already made.

**Dependency.** Requires `SB-CORE-004`: ancestry that records a parameter *value* without its
*source* is an activity log, which is precisely what the incumbents already have.

**Verified by.** `SB-CORE-T14` — every curve written by any module has a complete ancestry record.
`SB-CORE-T15` — an ancestry record round-trips through project save/load.

---

### `SB-CORE-011` — A project re-runs byte-identically   [P1] [PARTIAL]

**Requirement.** Re-running a recorded interpretation from raw import to pay summary MUST produce
byte-identical outputs. Any deliberate non-determinism MUST be seeded and the seed recorded.

**Rationale.** This is what a reserve audit, a PSC submission or a partner data room actually needs,
and no incumbent can produce it. Monte Carlo is already seeded from `(seed, index)`.

**Verified by.** `SB-CORE-T16` — a full field re-run produces byte-identical curve blobs and an
identical pay summary.

---

### `SB-CORE-014` — A learned model carries its training provenance   [P1] [ABSENT]

**Requirement.** Where a curve is produced by a fitted model rather than a deterministic equation,
`SB-CORE-010`'s ancestry MUST additionally record: the identity of the training rows (which wells,
which depth intervals, which log set); the feature set and its column order; every hyperparameter;
the random seed; the fitted preprocessing state (scaler means and variances, encoders); the model
artifact's own identity hash; and the library set and versions that produced it. Re-running a
recorded model prediction MUST reproduce it exactly, or MUST refuse and say which recorded element
could not be restored.

**Rationale.** `24_ml-advanced.md` escalated that `SB-CORE-010`'s enumeration — module, inputs,
parameters, zone, operator, timestamp — is complete for a deterministic module and **structurally
insufficient for a learned one.** Training rows, a fitted scaler, an artifact identity and a library
set are none of: a parameter, an input curve to the predicted well, or a module version. They
determine the number completely and `SB-CORE-010` does not ask for any of them.

A trained model is the least reproducible object in petrophysics, and none of the three incumbents
carries this into a deliverable either. That makes it the sharpest available instance of
`03_EVIDENCE_BASE.md` §14.4 rather than a compliance chore.

**Why P1 and not P0.** Unlike `SB-CORE-001` and `-006`, this does not make a shipped number *wrong*;
it makes it *undefendable*. That is the 1.0 bar, not the first-sale bar — but it is squarely the
failure mode of the user in `01_PRODUCT.md` §3.1.

**Scope note.** This chapter covers it in-domain. **Any future chapter that fits anything inherits
the gap** — rock-typing regressions, saturation-height fits, permeability transforms and electrofacies
all produce fitted objects, and none of their chapters has been written yet.

**Verified by.** `SB-CORE-T21` — a recorded prediction re-runs to an identical curve from its
provenance record alone. `SB-CORE-T22` — a provenance record missing any enumerated element causes a
refusal, not a silent re-fit.

---

### `SB-CORE-012` — Named interpretation scenarios with A/B diff   [P2] [ABSENT]

**Requirement.** A named parameter set MUST be savable, re-runnable and comparable against another,
with a diff that reports which parameters changed and which numbers moved.

**Rationale.** The asset team in `01_PRODUCT.md` §3.2 fails when a deliverable disagrees with the
last one for reasons nobody can articulate. This is the requirement that answers them directly.

---

### `SB-CORE-013` — Vendor divergence is visible at the point of choice   [P2] [ABSENT]

**Requirement.** Where the corpus records that the incumbents ship differing values for a parameter,
the parameter's editor MUST be able to show those values with their sources and tiers, and the
interpreter's choice MUST be recorded as a decision.

**Rationale.** `03_EVIDENCE_BASE.md` §14.2. This is the requirement that turns the whole cross-tool
evidence base into a shipped feature rather than a design input — and it is the one competitors
structurally cannot copy.

**Boundary.** This surfaces *values with sources*, never vendor algorithms, tables or text. See
`CONTRACT.md` §2.1: no vendor lookup-table data is transcribed, and nothing here changes that.

---

## 15.3 Portfolio scale — the v2 scope change

*The unit of work moved from field to portfolio (`01_PRODUCT.md` §1). These six are what that costs.*

### `SB-CORE-030` — Portfolio-scale target is declared and measured   [P1] [UNMEASURED]

**Requirement.** SandiBumi MUST state one portfolio-scale target — well count, project size, and the
operations that must remain usable at that scale — and MUST demonstrate it on a fixture before any
customer-facing copy claims it.

**Rationale.** `01_PRODUCT.md` §7.1. The claim in circulation is 2000+ wells; the measurements are
100 wells for a chain and 540 wells for a 15-minute project open. One of those two has to move.

**Blocked on.** Jauhar — `06_SEQUENCING_AND_GATES.md` §26 decision 5. The number cannot be invented
here.

---

### `SB-CORE-031` — A benchmark harness exists and is part of the gate   [P1] [ABSENT]

**Requirement.** Performance figures MUST come from a repeatable harness committed to the tree, not
from one-off measurements recorded in prose.

**Rationale.** Every performance number in this document is a single measurement or a static
estimate, and several documented cost centres (`01_PRODUCT.md` §7.2) have never been measured at all.
Without a harness, a regression is invisible until a client finds it — and it is the same failure
`02_RISKS_AND_CONTRADICTIONS.md` §11 names: a document stating a measurement rather than the method
that produces it.

---

### `SB-CORE-032` — The compute path does not hold the global lock across long work   [P1] [PRESENT-DIVERGENT]

**Requirement.** No operation whose duration scales with well count or sample count may hold the
global database mutex for its duration. Long operations MUST run off the main event-loop thread.

**Rationale.** Synchronous commands take the global mutex on the main event-loop thread, including
the log-viewer data path; a 30-well solver run holds it across an inversion estimated at up to
~800,000 bounded-least-squares calls. At portfolio scale this is the difference between a product and
a demo.

**Measurement re-taken 2026-08-07 — it moved the wrong way.** The audit figure quoted here was
*64 of 82* synchronous commands. `22_database-model.md` re-counted against current source and found
**109 of 130 synchronous, plus 17 of 79 asynchronous**, with **128 `db.0.lock()` sites in `lib.rs`**
— the last figure verified directly. The lock-taking surface has roughly doubled since the audit,
which means it is growing with the codebase rather than being contained. **The priority stays P1 on
correctness grounds — nothing here produces a wrong number — but the trend is the argument for
acting before the surface doubles again.**

**Note.** The single-writer `Mutex<Connection>` itself is correct and stays (`01_PRODUCT.md` §7.6).
The requirement is about *hold duration*, not about introducing concurrent writers.

---

### `SB-CORE-033` — Compute results are cached on content, not recomputed   [P2] [ABSENT — designed, parked]

**Requirement.** A computed result MUST be reusable when its inputs, parameters and module version
are unchanged.

**Rationale.** A DAG-and-cache design exists (`compute_dag_cache_design.md`, ~1,150 production lines,
content-hash keys, twelve named tests) and was **parked by Jauhar's direction** in favour of other
work. Nothing about that decision was wrong at the time; portfolio scale is the condition that would
reopen it.

**Precondition.** Its own header requires re-verifying its assumptions against current code before
implementing anything — chains are strictly sequential today and there is no hashing anywhere in the
tree. A design parked for a week is a design; parked for a quarter it is a hypothesis.

---

### `SB-CORE-034` — Interactive surfaces stay responsive at portfolio scale   [P2] [PRESENT-DIVERGENT]

**Requirement.** Pan, zoom and hover MUST remain responsive with a portfolio-sized project open, and
MUST NOT recompute more than the frame requires.

**Rationale.** Pan handlers redraw directly from mousemove with no frame throttling, and at least one
panel double-computes its axis auto-range on every pan frame. Panels leak per-well retained memory
across open/close cycles at a rate that is negligible on 40 wells and is not on 2,000.

---

### `SB-CORE-035` — Well scoping is enforced in the backend   [P1] [PRESENT-DIVERGENT]

**Requirement.** The active well group MUST scope every operation that claims to be scoped, enforced
at the backend rather than by frontend convention.

**Rationale.** Group scoping is a frontend-only filter today; the backend deliberately does not
enforce it. Several operations snapshot group membership once at build time and then act on a stale
set. At portfolio scale, an operation that quietly acts on 2,000 wells instead of 40 is both a
correctness failure and a performance one.

---

### `SB-CORE-036` — Cancellation is honest   [P1] [PRESENT-DIVERGENT]

**Requirement.** A job that offers Cancel MUST honour it. A job that cannot be cancelled MUST NOT
offer the control, and MUST NOT report "Cancelled" for work that ran to completion and committed.

**Rationale.** Only a minority of job kinds read the cancel flag; the rest complete, commit their
writes, and are then reported as cancelled.

**Correction, 2026-08-07 — the cheap half is already built and this document had it wrong.** The
earlier rationale claimed *"the job view carries no cancellable flag to check"* and called the fix a
one-field change still to be made. `22_database-model.md` reported otherwise and it is verified at
source: `cancellable` is a field on `Job` (`jobs.rs:89`) and on `JobView` (`jobs.rs:107`), threaded
through `run_job` (`jobs.rs:266`, `:275`) and `run_simple_job` (`jobs.rs:355`, `:386`), mapped at
`jobs.rs:428`, and **consumed by the frontend** at `src/ui/processingPanel.ts:203`
(`if (active && job.cancellable)`). The doc comment at `jobs.rs:106` states the intent explicitly —
show the button on cancellable jobs and "an honest 'can't be interrupted' tag on the rest".

**The requirement stands; only its remaining scope narrows.** What is left is the expensive half —
making more workers genuinely interruptible, and ensuring no job reports "Cancelled" for work that
ran to completion and committed. Status stays `PRESENT-DIVERGENT` for that reason, and this
requirement needs a regression test locking the flag's honesty so the closed half cannot silently
reopen.

**Relationship to `SB-CORE-002`.** This is a result-honesty violation wearing a UI costume.

---

## 15.4 Evidence and stewardship

*`SB-CORE-040` and `-041` are P0 and are stated in §15.1. These two complete the set.*

### `SB-CORE-042` — A green gate that a machine enforces   [P1] [PARTIAL]

**Requirement.** Build, lint and test MUST run automatically on every change, not manually.

**Rationale.** R6. No CI, no clippy configuration, no lint gate, and a frontend test harness that is
explicitly never part of the gate. `tools\check.ps1` is a good gate that depends on a human
remembering to run it.

**Blocked on.** `SB-CORE-041` — CI cannot run on a tree that does not build from a clean clone.

---

### `SB-CORE-043` — Architecture and decisions are written down   [P1] [ABSENT]

**Requirement.** `ARCHITECTURE.md` and decision records MUST exist and MUST be current.

**Rationale.** R9, unmoved since v1. The prompts that exist to produce these have still not been run.
Two payoffs for one job: the onboarding cost of a first engineering hire is otherwise paid in review
hours at exactly the wrong moment (`05_STRATEGY.md` §22.1), and the same artefacts answer the
procurement continuity question (§22.2).

---

### `SB-CORE-044` — Tier-C boundary is a shipped, auditable policy   [P1] [PARTIAL]

**Requirement.** The Tier-C register MUST be maintained in the repository, and any capability serving
a similar need MUST be documented as a design-around with its own primary sources.

**Rationale.** `01_PRODUCT.md` non-goal §5.8. A register that lives only in a prompt file is not a
policy a buyer's counsel can be shown.

---

## 15.5 Requirement summary

| ID | Title | Priority | Status |
|---|---|---|---|
| `SB-CORE-001` | Depth unit carried and enforced | **P0** | PARTIAL — Leverett closed, Skelt-Harrison open |
| `SB-CORE-002` | No degraded result presented as clean | **P0** | PRESENT-OK — seven reporting surfaces regression-locked |
| `SB-CORE-003` | Validity conditions are enforced preconditions | P1 | ABSENT |
| `SB-CORE-004` | No parameter ships without a source | **P0** | PARTIAL |
| `SB-CORE-006` | One name, one equation | **P0** | PRESENT-DIVERGENT |
| `SB-CORE-007` | One definition per constant and transform | **P0** | PRESENT-DIVERGENT |
| `SB-CORE-005` | Vendor-derived defaults re-sourced to primary literature | P1 | ABSENT |
| `SB-CORE-010` | Every computed curve answers "how was I made?" | P1 | ABSENT |
| `SB-CORE-011` | A project re-runs byte-identically | P1 | PARTIAL |
| `SB-CORE-012` | Named scenarios with A/B diff | P2 | ABSENT |
| `SB-CORE-013` | Vendor divergence visible at the point of choice | P2 | ABSENT |
| `SB-CORE-014` | A learned model carries its training provenance | P1 | ABSENT |
| `SB-CORE-015` | No artifact ships that our own reader rejects | **P0** | PRESENT-DIVERGENT |
| `SB-CORE-030` | Portfolio-scale target declared and measured | P1 | UNMEASURED |
| `SB-CORE-031` | Benchmark harness in the gate | P1 | ABSENT |
| `SB-CORE-032` | No global lock held across long work | P1 | PRESENT-DIVERGENT |
| `SB-CORE-033` | Content-hash compute cache | P2 | ABSENT (parked) |
| `SB-CORE-034` | Interactive surfaces responsive at scale | P2 | PRESENT-DIVERGENT |
| `SB-CORE-035` | Well scoping enforced in the backend | P1 | PRESENT-DIVERGENT |
| `SB-CORE-036` | Cancellation is honest | P1 | PRESENT-DIVERGENT |
| `SB-CORE-040` | Verification indexed by capability | **P0** | ABSENT |
| `SB-CORE-041` | Fresh clone builds and tests | **P0** | PRESENT-DIVERGENT |
| `SB-CORE-042` | Machine-enforced green gate | P1 | PARTIAL |
| `SB-CORE-043` | Architecture and decisions written down | P1 | ABSENT |
| `SB-CORE-044` | Tier-C boundary as shipped policy | P1 | PARTIAL |

*`SB-CORE-003` and `-005` are stated in full below — they belong to §15.1's subject matter but are P1
rather than P0.*

---

## 15.6 The two P1 correctness requirements

### `SB-CORE-003` — Validity conditions are enforced preconditions   [P1] [ABSENT]

**Requirement.** Every method whose source states a validity condition MUST carry that condition as
machine-readable data on its module spec, and the runner MUST evaluate it before computing. A
violated precondition MUST produce a labelled, actionable refusal or an explicitly flagged result —
never an unmarked number.

**Rationale.** `03_EVIDENCE_BASE.md` §14.3. Geolog's fail-loud reputation is a property of its
`.info` manifests, not its code; a port that lifts the algorithm without the validation columns
inherits a fail-silent version. Conditions already known from the corpus include a documented
anisotropy threshold beyond which a weak-anisotropy substitution is invalid, near-parallel exclusion
zones in dip processing where two vendors disagree on policy, and depth-range and lithology-province
limits on several pore-pressure methods.

**Design note.** This is the same mechanism as `SB-CORE-004`'s source string — a declarative field on
the module spec that the runner reads — which is why the two should be built together rather than as
separate efforts.

**Owning chapters.** `20_envcorr-qc.md` (the manifest finding), `18_geomech-ppfg.md` (depth and
province limits), `17_thinbed-laminated.md` (the anisotropy threshold), `25_fluidsub-rockphysics.md`.

---

### `SB-CORE-005` — Vendor-derived defaults are re-sourced to primary literature   [P1] [ABSENT]

**Requirement.** Every endpoint in the mineral library MUST cite a primary source. Rows that cannot be
re-sourced MUST be marked as vendor-derived in the UI and in the deliverable.

**Rationale.** R2. This is worth doing regardless of the legal answer, because it converts "merged
from two vendor installs, in one vendor's dropdown order" into "sourced from the literature,
cross-checked against two implementations" — which is a stronger claim, a smaller exposure, and the
foundation of `03_EVIDENCE_BASE.md` §14.4.

**Owning chapter.** `13_mineral-solver.md`.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
