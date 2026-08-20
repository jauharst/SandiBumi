# SandiBumi adversarial correctness review — whole repository

**Reviewed object:** `a6565bd9cde1397c28a2b67cf764011b6b497c11` (`master`)

**Review date:** 2026-08-20

**Scope:** Repository-wide static correctness review, with deep inspection of the Rust calculation/storage/import/export paths and the TypeScript IPC, editing, display, and reporting paths. The exact tree contains 623 files, including 77 Rust files and 120 TypeScript files. `CLAUDE.md` and every `docs/record_*.md` file were read first as binding contracts. All code evidence below was read from the named Git object with `git show`/`git grep`; the checked-out branch and its working-tree files were not used as the review source.

**Execution constraint:** No build or test command was run because those commands create artifacts and the review contract permits only one new report file. Tests mentioned below were inspected, not executed. Numeric counterexamples were independently evaluated with read-only calculator commands.

**Independence note:** No excluded prior report was opened. One broad source search accidentally surfaced isolated matching lines from `docs/review_sweep/F1.md` and `docs/review_triage.md`; neither file was opened, no candidate learned from those snippets is reported here, and every included finding was derived and refuted independently from production code, binding contracts, allowed build records, callers, and tests.

## Severity summary

| Severity | Count | Meaning used here |
|---|---:|---|
| P0 | 2 | Wrong numbers or physical scale in a client-deliverable path |
| P1 | 16 | Plausible wrong numbers reachable through normal application use |
| P2 | 3 | Edge-case wrongness or a binding contract violation not yet shown to corrupt a normal deliverable |
| P3 | 0 | No hazard-only item met the evidence bar |

## P0 — Composite exports treat foot-project depths as metres, making true-scale client plots 3.28084 times too long.

**Severity:** P0 — this changes the physical scale and pagination of exported SVG/PDF well composites, which are client-deliverable artifacts.

**Decisive code:** `src-tauri/src/units.rs:3-12` defines the binding rule that stored depths are in the declared project unit, which may be feet, while display units are separate. `src-tauri/src/composite.rs:53-64` accepts a scale and raw depth limits but carries no project-depth unit. `src-tauri/src/composite.rs:335-353` always computes `mm_per_m = 1000 / scale`, derives a number of “metres” per page, and adds that directly to the raw stored depth. `src-tauri/src/composite.rs:690-709` likewise maps every raw depth difference to page millimetres with `mm_per_m`. The dialog reinforces the assumption at `src/ui/compositeDialog.ts:83-86` and `src/ui/compositeDialog.ts:134-164`, where raw depth values are sent unchanged and every range/page is labelled `m`.

**Concrete failure scenario:** Create or open a feet-declared project and render a 100 ft interval at 1:500. The interval is 30.48 m, so a true 1:500 depth axis must occupy `30.48 × 1000 / 500 = 60.96 mm`. The exporter instead applies `100 × 1000 / 500` and draws 200 mm. The composite is therefore 3.28084 times too long, its stated 1:500 scale is physically about 1:152.4, and the wrong interval length also changes page breaks. A petrophysicist can reproduce this with any 100 ft logged interval and a ruler on the saved SVG/PDF.

**Self-refutation attempt:** I checked whether the composite command obtains the project unit, whether the frontend pre-converts its requested range, and whether the lower-level page mapper uses a feet-specific multiplier. It does none of those. The interactive log renderer has separate unit-aware scale handling, but the composite code does not call it. Existing composite tests exercise metric-valued depths, so they do not save the feet path.

## P0 — Pay summaries, dashboards, PDFs, and CSV exports label raw feet-project thicknesses as metres.

**Severity:** P0 — the wrong unit is printed beside pay and hydrocarbon-pore-volume numbers in client-facing PDF and CSV deliverables.

**Decisive code:** `src-tauri/src/workflow.rs:4555-4564` derives sample thickness directly from the stored project-depth coordinate, and `src-tauri/src/workflow.rs:4684-4759` accumulates gross, net, and `PHIE × (1-SWE) × thickness` without converting that coordinate. This is correct internal project-unit arithmetic, but the consumers misstate it. `src-tauri/src/report.rs:535-572` prints raw zone depths under `Top (m)` and `Bottom (m)`, and `src-tauri/src/report.rs:634-670` prints raw pay values with `HPV (m)`. `src/ui/dashboardPanel.ts:17-48`, `src/ui/dashboardPanel.ts:205-235`, and `src/ui/dashboardPanel.ts:275-294` hard-code `Net (m)`, `HPV (m)`, and `m` KPI suffixes; `src/ui/dashboardPanel.ts:535-552` exports those same headings and values to CSV. The summary and cutoff panes repeat the assumption at `src/ui/summaryDialog.ts:122-145` and `src/ui/cutoffDialog.ts:450-452`.

**Concrete failure scenario:** In a feet-declared project, define a zone from 1000 to 1100 ft with every sample classified pay, `PHIE = 0.20`, and `SWE = 0.25`. The pay engine correctly accumulates 100 project-depth units of gross/net and `0.20 × 0.75 × 100 = 15` project-depth units of HPV. The dashboard, summary, PDF, and exported CSV present those as 100 m and 15 m. The true metric values are 30.48 m and 4.572 m. Thus a field roll-up can overstate both net pay and hydrocarbon pore-thickness by 3.28084 while remaining numerically plausible.

**Self-refutation attempt:** I checked whether `run_pay_summary` promises metres, whether the report converts the returned rows, and whether a shared unit formatter relabels the UI. None does. The office-workbook path is a useful counterexample: `src-tauri/src/office.rs:474-495` parameterizes the headings with the project unit, and `src-tauri/src/office.rs:798-837` explicitly reads that unit before building sheets. That proves the raw values are intentionally project-native, but it does not repair the PDF, dashboard, summary, cutoff, or dashboard-CSV paths.

## P1 — Core and SCAL files whose explicit percent values have a median at or below 1.5 are silently stored as fractions.

**Severity:** P1 — ordinary tight-rock/core-lab data can enter the project at 100 times its true porosity or saturation and then drive plausible downstream calculations.

**Decisive code:** `src-tauri/src/parsers.rs:2277-2290` decides that a file is percent only when the finite median exceeds 1.5; it does not consult a `%` unit token or header. That helper is applied to routine core porosity/saturation at `src-tauri/src/parsers.rs:2344-2351`, long-form SCAL at `src-tauri/src/parsers.rs:2452-2462`, porous-plate SCAL at `src-tauri/src/parsers.rs:2635-2645`, and centrifuge SCAL at `src-tauri/src/parsers.rs:2738-2747`. The core wizard says percent must be confirmed (`src/ui/coreImportDialog.ts:14-21`), but its actual choice model has role, depth-unit, and depth-datum controls only (`src/ui/coreImportDialog.ts:42-67` and `src/ui/coreImportDialog.ts:273-293`). It merely displays a note after the same median heuristic has already fired (`src/ui/coreImportDialog.ts:403-408`); there is no percent/fraction override.

**Concrete failure scenario:** Import a tight-rock RCAL table headed `PORO (%)` with values 0.8, 1.0, and 1.2. The correct stored values are 0.008, 0.010, and 0.012 v/v. Because the median is 1.0, SandiBumi stores 0.8, 1.0, and 1.2 v/v. The first two even pass the later `<= 1.0` core-porosity validity gate and are read as 80% and 100% porosity. The same error materially changes SCAL: with `Pc = 10 psi`, `IFT = 72 dyn/cm`, `k = 1 mD`, and true `phi = 1%`, `src-tauri/src/satheight.rs:37-64` gives `J = 0.300625`; storing `phi = 1.0` instead gives `J = 0.0300625`, ten times too small.

**Self-refutation attempt:** I checked the file probe, the mapped-import request, the units-row handling, and the UI for a way to honor an explicit `%` header or let the user override the heuristic. High-porosity percent files are detected and announced, but low-valued percent files are neither detected nor overridable. No downstream guard can reconstruct the lost factor of 100.

## P1 — Deviation, tops, and SCAL importers write file depths without reconciling them to the project depth unit.

**Severity:** P1 — these are normal import surfaces, and a feet/metric mismatch moves surveys, formation tops, and plugs by a factor of 3.28084.

**Decisive code:** The binding contract is explicit at `src-tauri/src/units.rs:3-7`: every stored depth uses one project unit, and a file in the other unit is converted on input. LAS follows it at `src-tauri/src/ingest.rs:570-607`, and the core wizard/import follows it at `src-tauri/src/ingest.rs:1831-1846`. In contrast, `import_deviation_csv` has no file-unit argument and feeds parsed MD and datum straight into minimum curvature/storage (`src-tauri/src/ingest.rs:1492-1539`); `DeviationSurvey` itself carries no unit (`src-tauri/src/parsers.rs:3196-3202`). Tops parsing recognizes MD versus TVD datum but no unit (`src-tauri/src/parsers.rs:3368-3448`), and `src-tauri/src/ingest.rs:2362-2443` writes `rec.depth` unchanged. `import_scal_files` likewise has no depth-unit argument and copies or core-maps raw plug depth (`src-tauri/src/ingest.rs:2210-2321`). The SCAL, tops, and survey UIs expose a datum or datum elevation but no file-depth-unit selector (`src/ui/ribbon.ts:1840-1961` and `src/ui/ribbon.ts:1989-2095`). Datum and unit are different properties; choosing MD/TVD cannot perform ft↔m conversion.

**Concrete failure scenario:** Start with a metres-declared project, then import a survey or tops CSV whose MD column contains 8000 ft. The importer stores 8000 in the metric project instead of 2438.4 m. The survey is then sampled against a metric log grid at the wrong stratigraphic position, and the top plots at 8000 m. A SCAL plug at 8000 ft similarly lands at 8000 m unless a separately established core-depth mapping happens to move it. The reverse mismatch shrinks metric input in a feet project by the same factor.

**Self-refutation attempt:** I checked whether the parsers infer units from suffixes/units rows, whether the Tauri command wrappers add a unit argument, whether project-unit conversion occurs just before each database insert, and whether the datum controls double as unit controls. None does. The correct LAS and core paths prove the shared conversion facility exists, but these three import paths bypass it.

## P1 — A blank or absent inclination or azimuth in a deviation survey is silently converted into a measured zero.

**Severity:** P1 — a common incomplete-cell condition produces finite, plausible, wrong TVD/TVDSS curves rather than an import refusal or missing geometry.

**Decisive code:** `src-tauri/src/parsers.rs:3216-3230` makes INC and AZI columns optional. `src-tauri/src/parsers.rs:3232-3248` parses an absent column, blank cell, or invalid numeric cell as `f32::NAN` and then replaces that missing value with `0.0` before returning the survey. `src-tauri/src/ingest.rs:1514-1545` computes minimum curvature, stores the path, and materializes TVD/TVDSS without warning. This also violates the repository’s missing-value contract at `CLAUDE.md:16-18`: missing continuous data is NaN, not zero.

**Concrete failure scenario:** Use a metric survey:

```text
MD,INC,AZI
0,0,90
1000,,90
2000,60,90
```

If the middle survey station was 30° but its cell was lost during export, the complete minimum-curvature survey reaches TVD 1653.99 m at MD 2000 m. The importer replaces the blank with 0° and computes 1826.99 m, a 173.01 m error, then persists and plots it as a valid finite curve. If the whole INC column is missing, an arbitrarily deviated well is accepted as vertical.

**Self-refutation attempt:** I checked for a probe warning, required-column validation, row-rejection rule, and an ingest result note. There is none. The parser comment explicitly says missing INC/AZI is treated as zero, but documenting the coercion does not make “not measured” equivalent to a measured vertical/north station, and no caller asks the user to confirm that assumption.

## P1 — Materialized TVD and TVDSS freeze at the first or last survey station outside survey coverage.

**Severity:** P1 — a partial survey produces long finite plateaus that downstream height, correlation, and report calculations can consume as real geometry.

**Decisive code:** `src-tauri/src/deviation.rs:75-99` clamps every MD below the first station to the first station’s TVD/TVDSS and every MD above the last station to the last station’s values. `src-tauri/src/ingest.rs:1551-1579` calls that function for every sample on the full log grid and writes the resulting curves. The low-level test at `src-tauri/src/deviation.rs:140-164` pins the clamping behavior. The materializer’s own shadowing warning at `src-tauri/src/ingest.rs:1580-1584` describes NaN outside the survey range, but the function actually supplies finite endpoints.

**Concrete failure scenario:** Import a vertical survey with stations at MD 0, 1000, and 2000 m into a well logged to 3000 m. At MD 2500 and 3000, the materialized TVD is 2000 m and TVDSS is the same frozen endpoint minus datum. A continued vertical trajectory would be 2500 and 3000 m; if extrapolation is not authorized, those samples should be missing. The shipped result is neither—it is a physically impossible zero vertical increment over the final 1000 m, but it looks like a valid curve.

**Self-refutation attempt:** I checked whether the import requires survey coverage through log TD, whether materialization restricts its output grid, and whether downstream modules mask values outside station bounds. None does. The test proves the clamp is intentional at the helper level, but it does not make emitting those endpoints as full-length survey-derived curves safe.

## P1 — The saturation-height dialog labels FWL in metres while the backend interprets it in the project’s native depth coordinate.

**Severity:** P1 — following the displayed unit on a feet project can place the contact thousands of feet away and turn an oil-bearing sample into fully water-saturated output.

**Decisive code:** The `sw_height` manifest declares FWL as `m` at `src-tauri/src/satheight.rs:96-114`, and the generic dialog renders that literal unit unchanged at `src/ui/moduleDialog.ts:412-439`. At runtime, however, `src-tauri/src/satheight.rs:171-200` computes `h = FWL - vertical_depth` before converting the resulting height from `ctx.depth_unit` to metres. `src-tauri/src/modules.rs:1109-1115` confirms that depth and depth-derived parameters in the context are project-native. Thus the backend requires FWL in feet in a feet project even though the only unit shown beside the input is metres.

**Concrete failure scenario:** In a feet project, take a sample at TVD 6500 ft and a true FWL at 6565.62 ft, so height above FWL is 65.62 ft = 20 m. With Skelt parameters `A=0.8`, `B=20 m`, `C=1`, `D=0`, the intended saturation is `1 - 0.8 exp(-20/20) = 0.705696`. A user obeying the `m` suffix enters the same contact as 2001.2 m. The backend subtracts that raw number from 6500 ft, gets a negative height, and returns `SWT = 1.0` through the below-FWL branch.

**Self-refutation attempt:** I checked the Skelt and Leverett physics after height is formed; both correctly convert project-native height to the required physical unit. That does not save the coordinate input: no frontend or workflow conversion changes an FWL entered under the `m` suffix into project units. Tests construct FWL directly in the project coordinate and therefore miss the UI/backend disagreement.

## P1 — Depth-editing dialogs label values in metres but send them unchanged to project-unit storage and interpolation.

**Severity:** P1 — ordinary curve edits, core registration edits, and well-header edits move or store depths by the wrong physical distance in a feet project.

**Decisive code:** The curve editor labels shift/top/base as metres and sends the numeric values unchanged (`src/ui/curveEditDialog.ts:38-55` and `src/ui/curveEditDialog.ts:112-141`); `src-tauri/src/curve_edit.rs:773-824` applies them directly to the project-depth grid. The core-shift dialog likewise labels and reports metres (`src/ui/ribbon.ts:1484-1537`), while `src-tauri/src/db.rs:9228-9249` adds the raw delta to core and SCAL depths. The well-header dialog labels TD and KB as metres (`src/ui/ribbon.ts:2230-2246`) and sends their text unchanged (`src/ui/ribbon.ts:2277-2289`); `src-tauri/src/db.rs:8962-8975` parses and stores the number without unit conversion.

**Concrete failure scenario:** In a feet-declared project, enter a `+3 m` curve or core shift. Both backends apply `+3 ft`, only 0.9144 m. For the well header, enter a KB of 30 m as instructed. It is stored as 30 ft instead of 98.4252 ft. At TVD 6500 ft, positive-down TVDSS becomes 6470 ft rather than 6401.5748 ft, an error of 68.4252 ft (20.856 m) that propagates into contacts and saturation-height work.

**Self-refutation attempt:** I checked for use of the project/display unit preference in each dialog, conversion in the IPC wrappers, and conversion in the backend update functions. None exists. Undo makes the changes reversible but does not make the applied physical distance correct.

## P1 — Autocorrelation and contact-analysis windows and thresholds are metre-labelled constants applied in raw project units.

**Severity:** P1 — cross-well marker proposals and contact-consistency flags can change solely because the project stores feet rather than metres.

**Decisive code:** The autocorrelation UI calls its window and search distances metres (`src/ui/autoCorrDialog.ts:107-131`) and sends them unchanged (`src/ui/autoCorrDialog.ts:152-207`). `src-tauri/src/tops.rs:69-90` describes them only as “depth units”, `src-tauri/src/tops.rs:163-199` applies them directly to stored depths, and multi-marker correlation hard-codes raw 8–30-unit windows at `src-tauri/src/tops.rs:344-357`. Contact suggestion uses raw 5.0-unit contrast windows and 2.0-unit merge distances while calling them metres (`src-tauri/src/contacts.rs:54-89`), and contact consistency calls `flag_abs` metres but compares it directly to project-native TVDSS residuals (`src-tauri/src/contacts.rs:442-479`). The command defaults that threshold to 3.0 at `src-tauri/src/lib.rs:3832-3849`, while `src/ui/correlationPanel.ts:1360-1383` prints the resulting residuals/RMS as metres.

**Concrete failure scenario:** In a feet project, a target marker whose true correlation peak is 50 ft from the initial pick is 15.24 m away. A user selecting `Search ±25 m` reasonably requests about ±82.02 ft, but the backend searches only ±25 ft and cannot reach the true peak; a nearer local maximum can become a plausible wrong proposal. Separately, two contact picks differing by 5 ft differ by only 1.524 m and should be below the default 3 m consistency threshold, yet `5 > 3` flags the well as inconsistent.

**Self-refutation attempt:** I checked whether source and target frames are converted before analysis or whether the thresholds are converted to the project unit. All depths are internally consistent project-native values, which is good for arithmetic, but no code preserves the stated physical metre widths. The unit-consistent depth arrays therefore do not save the metre-labelled constants.

## P1 — Lorenz and Results-QC present project-native thickness and capacity totals as metric values without conversion.

**Severity:** P1 — reachable analytical readouts show plausible totals with a 3.28084 unit error in feet projects.

**Decisive code:** `src-tauri/src/lorenz.rs:185-223` derives local thickness directly from stored depth and accumulates `k×h` and `phi×h`; `src-tauri/src/lorenz.rs:313-319` returns those raw totals. `src/ui/lorenzDialog.ts:175-180` always labels them `mD·m` and `m`. Results-QC similarly formats cutoff-sweep net as metres at `src/ui/resultsQcPanel.ts:437-451`, labels every envelope depth axis `Depth (m)` at `src/ui/resultsQcPanel.ts:503-530`, and appends `m` to project-native zone bounds at `src/ui/resultsQcPanel.ts:802-820`.

**Concrete failure scenario:** Run Lorenz on a constant 1 mD, `phi=0.20` interval that is 100 ft thick. The backend returns `total_kh = 100` and `total_phih = 20` in `mD·ft` and ft. The UI displays `100 mD·m` and `20 m`; the true metric totals are 30.48 mD·m and 6.096 m. Results-QC likewise prints a 100 ft net result as `NET 100.0 m` and labels a 1000–1100 ft zone `1000–1100 m`.

**Self-refutation attempt:** I checked whether Lorenz’s dimensionless coefficient itself is affected. A uniform depth-unit factor cancels in its cumulative fractions, so the coefficient remains valid; the reported capacity totals do not. Neither UI reads the project unit or converts the returned depth/thickness values.

## P1 — Statistics “Versus” compares two log sets by array index even when the sets carry different depth frames.

**Severity:** P1 — a normal comparison involving a Reframe/OWN set can report every common sample changed while missing the actual overlap and coverage changes.

**Decisive code:** The build contract at `docs/record_data_tools.md:68-80` states that a log set may carry its own sampling and that `fetch_curve_frame_from_set` replaces the run frame with that set’s OWN depths. The implementation does so at `src-tauri/src/equations.rs:3784-3841`. `src-tauri/src/statistics.rs:414-477` fetches both depth arrays into `_da` and `_db_`, deliberately discards them, takes `min(av.len(), bv.len())`, and compares `av[i]` to `bv[i]`. There is no depth equality check or depth join.

**Concrete failure scenario:** Compare set A with depths `[1000,1001,1002]` and values `[10,20,30]` against set B with depths `[1001,1002,1003]` and values `[20,30,40]`. The code reports `n_common=3`, `n_changed=3`, `mean_diff=+10`, `only_a=0`, and `only_b=0`. A depth-keyed comparison has two common depths, both unchanged, plus one depth unique to A and one unique to B: `n_common=2`, `n_changed=0`, `only_a=1`, `only_b=1`.

**Self-refutation attempt:** I checked whether all callers force both sets onto the standard frame or whether `fetch_curve_frame_from_set` resamples the second set to the first. It does not; OWN sampling is an intentional, normal feature. Standard-frame pairs happen to align, but there is no guard that limits Versus to that case, and no inspected test pins a shifted/OWN-frame comparison.

## P1 — Statistics thickness converts partial TVD coverage into zero-thickness slabs and then uses that incomplete TVD total for the whole row.

**Severity:** P1 — a single finite TVD sample is enough to replace a valid MD thickness and N/G with a plausible but incomplete TVD result.

**Decisive code:** `src-tauri/src/statistics.rs:521-527` promises TVD thickness only where a TVD curve is present and says missing TVD should be blank rather than copied. At `src-tauri/src/statistics.rs:576-599`, however, any finite value anywhere makes the TVD vector present, while a missing current or neighboring TVD value contributes `0.0` thickness. At `src-tauri/src/statistics.rs:662-699`, `any_tvd` becomes true because `vstep` returns `Some(0.0)` even at gaps, and the row’s N/G switches wholesale from MD to the incomplete TVD accumulators.

**Concrete failure scenario:** Use depths `[0,1,2,3]`, TVD `[0,1,NaN,3]`, and a flag `[1,1,0,0]`. The half-step MD method gives gross 3, net 1.5, N/G 0.5. The shipped TVD path gives per-sample thicknesses `[0.5,0.5,0,0]`, reports gross TVD 1, net TVD 1, and N/G 1.0. Missing TVD coverage has been counted as zero rock, doubling the ratio.

**Self-refutation attempt:** I checked for a coverage threshold, an all-finite requirement, interval-level fallback to MD, and a returned coverage warning. There is none. The distinction between no TVD and some TVD is handled, but the equally important distinction between complete and partial TVD is not.

## P1 — Cutoff sensitivity silently disables a requested permeability cutoff when the PERM curve has no finite samples.

**Severity:** P1 — deterministic pay and its sensitivity plot can give contradictory net pay for the same well and declared cutoff.

**Decisive code:** The deterministic pay path treats a requested permeability cutoff as active regardless of data availability (`src-tauri/src/workflow.rs:4549-4553`), and `classify_sample` correctly requires a finite PERM to demonstrate that the cutoff passes (`src-tauri/src/workflow.rs:4893-4901`). The sweep path instead sets `has_perm_cut = perm_min.is_some() && any finite PERM` at `src-tauri/src/workflow.rs:4935-4959`; with an all-missing PERM curve it passes `false` and the classifier ignores the requested gate. The UI sends the finite requested cutoff directly at `src/ui/cutoffDialog.ts:531-550` and does not say it may be disabled.

**Concrete failure scenario:** Use a 10 m clean interval with `VSH=0.10`, `PHIE=0.20`, `SWE=0.30`, no finite PERM anywhere, and request `PERM >= 10 mD` for pay. The deterministic summary returns net 0 because no sample proves it passes PERM. A PHIE/VSH/SWE sensitivity sweep holding that same PERM cutoff returns net 10 m (and N/G 1) because `has_perm_cut` becomes false. Both outputs look valid but answer different classification questions.

**Self-refutation attempt:** I checked whether the sweep emits a no-PERM error/note, whether the UI removes the cutoff before comparing outputs, and whether `classify_sample` independently sees `perm_min`. It only sees the caller’s Boolean, so the sweep’s precondition decisively disables the gate. The comment claiming agreement with deterministic pay describes the opposite of the deterministic rule at this commit.

## P1 — SSPW’s gas branch still uses the superseded 1.6 weighting even though the module declares an RMS midpoint correction.

**Severity:** P1 — normal gas-bearing density-neutron input produces a materially low total porosity under the method the application says it is running.

**Decisive code:** The file-level method declaration says gas/HC conditioning uses `sqrt((phiD²+phiN²)/2)` and that the earlier 1.6 weighting was fixed (`src-tauri/src/ssc.rs:16-22`). The SSC branch implements that RMS midpoint at `src-tauri/src/ssc.rs:182-203`. The SSPW branch claims “Same gas conditioning as SSC” at `src-tauri/src/ssc.rs:488-496` but computes `sqrt(phiD² - 1.6×|phiD²-phiN²|/2)` at `src-tauri/src/ssc.rs:497-502`.

**Concrete failure scenario:** Use `RHOB=2.32 g/cc`, `NPHI=0.05`, `VSH=0.20`, `RHOB_MAT=2.65`, `RHOB_FL=1.00`, and `RHOB_DSH=2.71`. Density porosity is 0.20 and the gas branch fires. The shipped 1.6 expression makes corrected density porosity 0.10, `RHOB_COR=2.485`, and SSPW `PHIT=0.106498`. The declared RMS midpoint is 0.1457738, giving `RHOB_COR=2.409473` and `PHIT=0.151941`. The live result is low by 0.045443 v/v.

**Self-refutation attempt:** I checked the allowed method note `docs/method_ssc_sspw.md:45-56`, which warns that SSPW as a whole still needs reference-suite validation. That caveat does not save an internal contradiction: the production source explicitly selects and explains the RMS branch, SSC implements it, and SSPW’s adjacent comment says it uses the same correction. No alternate caller replaces the SSPW formula.

## P1 — RtC treats a missing per-sample CAPBW value as measured zero capillary-bound water.

**Severity:** P1 — a gap in a supplied CAPBW curve produces a finite, plausible resistivity correction and water saturation instead of a missing result.

**Decisive code:** CAPBW is an optional log input in the RtC manifest (`src-tauri/src/lrlc.rs:115-143`). `src-tauri/src/lrlc.rs:147-173` resolves the primary/fallback CAPBW per sample and then converts NaN to `0.0`. The finite zero enters the excess-conductivity equation at `src-tauri/src/lrlc.rs:176-192`. By contrast, the RtC calibration path requires finite CAPBW and excludes incomplete samples (`src-tauri/src/lrlc.rs:641-700`). A wholly absent optional curve and a hole in a curve the user did supply are therefore collapsed into the same numeric state.

**Concrete failure scenario:** At one sample use `RT=20 ohm.m`, `PHIT=0.25`, `RW=0.3`, `M=N=2`, `QV=0.3 meq/cm3`, `A_CAP=0.45`, `B_QV=0.0057`, `C0=-0.0071`, and `RSF=2.25`. With measured `CAPBW=0.08`, excess conductivity is 0.0172181 mho/m, corrected resistivity is 30.5047 ohm.m, and `SWT=0.396677`. If that CAPBW cell is NaN, the code silently substitutes zero, clamps the resulting negative excess term to zero, returns uncorrected 20 ohm.m, and writes `SWT=0.489898`—a 0.09322 v/v jump caused only by missing data.

**Self-refutation attempt:** I checked whether workflow input validation refuses a partially missing CAPBW curve or whether a provenance flag distinguishes a deliberate QV-only run. It does neither. Treating an entirely omitted optional CAPBW input as a model choice may be legitimate, but the implementation has no representation for that choice separate from a missing sample in a selected curve, so the per-sample failure remains.

## P1 — IMTS turns simultaneous VKAOL and VILL gaps into zero clay charge and a finite clean-rock saturation.

**Severity:** P1 — missing mineral evidence produces a bookable saturation instead of “no result,” with the largest bias in the LRLC intervals the method is meant to address.

**Decisive code:** The IMTS method states that Qv is built from kaolinite/illite volumes (`src-tauri/src/lrlc.rs:231-247`), but both curve inputs are individually optional (`src-tauri/src/lrlc.rs:309-316`). In the live module, each NaN is independently replaced by zero at `src-tauri/src/lrlc.rs:330-369`, so when both are missing the sample gets `QVEFF=0` and the iterative conductivity solver still emits Sw. The S-factor calibration code explicitly applies the missing-data distinction the run omits: `src-tauri/src/lrlc.rs:1139-1149` says one missing mineral can be zero, but both missing means “no clay information” and excludes the point.

**Concrete failure scenario:** Use `RT=10 ohm.m`, `PHIT=0.20`, `RW=0.10`, `TEMP_C=80`, `A=1`, `MSTAR=NSTAR=2`, `S_FACTOR=0.5`, `CEC_KAOL=8`, `CEC_ILL=25`, `RHOG=2.65`, and `SWIRR=0.20`. With `VKAOL=0.20` and `VILL=0.10`, `QVEFF=0.271625 meq/cm3` and the solver converges to `SWT=0.364730`. If both mineral cells are NaN, it writes `QVEFF=0` and `SWT=0.500000`. The result remains finite and plausible even though the clay-charge input vanished.

**Self-refutation attempt:** I checked whether at least one mineral curve is required at workflow validation, whether the module detects both missing, and whether a run-level option explicitly selects a clean-rock fallback. None exists. The calibration’s own both-missing guard demonstrates that the project already distinguishes “one mineral absent” from “no clay information”; the production module does not.

## P1 — Several core-to-log readers ignore the declared core datum and pair TVD/TVDSS plugs directly to an MD log frame.

**Severity:** P1 — normal core overlays, facies validation, and SandiMin core calibration can compare different physical depth references and return plausible wrong matches.

**Decisive code:** Core import records the user’s chosen datum (`src-tauri/src/ingest.rs:1780-1787` and `src/ui/coreImportDialog.ts:290-293`). `src-tauri/src/db.rs:4097-4150` defines the required cross-datum refusal and says it is shared by depth-pairing readers. It is correctly called by SCAL and generic core point-series readers (`src-tauri/src/db.rs:4153-4156` and `src-tauri/src/db.rs:4211-4219`). It is absent from `get_core_plugs` (`src-tauri/src/db.rs:4184-4199`), `get_core_por_gd` (`src-tauri/src/db.rs:4242-4256`), and the direct core-overlay query (`src-tauri/src/equations.rs:300-329`). Those unguarded readers feed the facies core-permeability tie (`src-tauri/src/facies_tie.rs:303-351`), SandiMin’s core porosity/grain-density fits (`src-tauri/src/sandimin.rs:1979-2030`), and plotted log/crossplot overlays (`src-tauri/src/lib.rs:804-812`).

**Concrete failure scenario:** Import a core delivery explicitly declared TVD, with a plug at TVD 1000 m that corresponds to MD 1200 m in a deviated well. The active log frame is MD. The unguarded overlay plots the plug at MD 1000; facies tie and SandiMin find the nearest log sample around MD 1000 instead of MD 1200 and compare the plug to the wrong rock. The results remain finite—class variance reduction, porosity RMS, or grain-density RMS—so the category error is not self-revealing.

**Self-refutation attempt:** I checked active-set selection and found it present, so this is not a stale-delivery bug. I also checked the guarded sibling readers and the importer’s datum declaration. Those safeguards prove the datum is available, but the named readers never invoke the refusal before pairing. No caller pre-converts core datum to MD.

## P1 — The cutoff rock-type module accepts an inverted class ladder and silently promotes and demotes the middle rock class.

**Severity:** P1 — a reachable parameter entry changes published facies counts in non-monotonic, geologically misleading ways.

**Decisive code:** The manifest requires `VSH1 <= VSH2` and `PHI1 >= PHI2` at `src-tauri/src/rocktyping.rs:240-260`. The runtime at `src-tauri/src/rocktyping.rs:264-281` only tests class 1 and then class 2; it performs no cross-parameter validation. Each individual field is merely bounded 0–1. The characterization test at `src-tauri/src/rocktyping.rs:511-548` explicitly confirms that an inverted ladder is accepted and “scatters” the middle class; it pins the defect as current behavior rather than preventing it.

**Concrete failure scenario:** With sane cutoffs `VSH1=0.15`, `PHI1=0.12`, `VSH2=0.35`, and `PHI2=0.06`, a sample at `VSH=0.30`, `PHIE=0.20` is class 2. Enter the still-in-range but inverted `VSH1=0.50`, `VSH2=0.20`: the same sample becomes class 1 because the looser class-1 gate runs first. A second sample at `VSH=0.30`, `PHIE=0.08` moves from class 2 to class 3. One bad ordering simultaneously promotes and demotes rock without a validation error.

**Self-refutation attempt:** I checked the generic module parameter validator, the generated dialog, and the test suite for a relation check. They validate each scalar against its own 0–1 bounds only. The existing test does not save the user; its comments explicitly say the accepted state is “pinned AS-IS, not endorsed.”

## P2 — The advertised one-metre core-to-log matching tolerance becomes one foot in feet projects.

**Severity:** P2 — the wrongness needs a feet project and a sufficiently coarse/reframed log grid, but it can silently remove valid plugs from calibration/QC statistics.

**Decisive code:** Facies tie declares `CORE_MATCH_TOL_M = 1.0` and reports “within 1 m” (`src-tauri/src/facies_tie.rs:113-118` and `src-tauri/src/facies_tie.rs:282-300`), but compares that constant directly to project-native depth differences. SandiMin duplicates the same raw threshold at `src-tauri/src/sandimin.rs:1301-1321` and tells the user the fit used plugs within 1 m at `src/ui/sandiminDialog.ts:1237-1244`. Neither path obtains or converts the project unit.

**Concrete failure scenario:** In a feet project with log samples at 1000 and 1005 ft, place a valid core plug at 1002 ft. The nearest log point is 2 ft = 0.6096 m away, inside the advertised 1 m tolerance. Both matchers compare raw `2 > 1` and reject it. The facies report increments “unmatched,” and the SandiMin core RMS omits the plug.

**Self-refutation attempt:** I checked whether core and log depths themselves share the project unit; they do, so nearest-neighbor ordering is correct. The defect is specifically the physical tolerance. Fine native sampling often keeps the nearest point within 1 ft, which narrows reachability, but Reframe/coarse curves make the counterexample valid and no conversion guard exists.

## P2 — Results-QC and core registration send full per-depth float arrays through Tauri as JSON despite the binding byte-IPC contract.

**Severity:** P2 — this is a direct contract violation with a concrete scale failure, although static review alone does not establish a particular client deliverable already corrupted.

**Decisive code:** `CLAUDE.md:16-18` says raw float arrays must never cross Tauri as JSON and must be packed with `bytemuck` into bytes. Results-QC instead derives `Serialize` on structures containing `Vec<f32>` for depth, every method, and three envelopes (`src-tauri/src/resultsqc.rs:64-88`), returns those vectors from `src-tauri/src/resultsqc.rs:363-440`, and exposes the structure directly from the command at `src-tauri/src/lib.rs:3901-3910`; the frontend expects JSON number/null arrays at `src/ipc.ts:2931-2955`. Registration repeats the pattern for core points, log depth/value, and the scan (`src-tauri/src/registration.rs:244-280` and `src-tauri/src/registration.rs:510-523`), with JSON array types at `src/ipc.ts:4813-4834`.

**Concrete failure scenario:** Run Results-QC on a 500,000-sample high-resolution well with seven available Sw methods. The response contains depth plus seven method arrays plus `sw_min`, `sw_max`, and `spread`: 5.5 million JSON numeric/null elements in one invoke. The packed payload would be about 22 MB of float bytes; this path instead serializes millions of textual/JS-number values, allocates them as a nested JSON object, and parses them on the UI thread. Registration likewise sends its complete log vectors rather than a packed frame. The behavior is deterministic and directly contradicts the application’s 2000+-well performance contract.

**Self-refutation attempt:** I checked whether these commands return `tauri::ipc::Response`, whether a custom serializer packs the vectors, and whether the backend caps/decimates the arrays. They do not. Small wells serialize successfully, and scalar metadata legitimately uses JSON, but these are the exact raw per-depth float arrays the contract singles out. Standard curve/track fetches demonstrate the intended packed-byte implementation.

## P2 — A SQL-NULL core property preserved from a legacy project makes the entire four-property core overlay command fail.

**Severity:** P2 — incomplete core tables are normal, but the failure affects visualization rather than stored values or an automatic client calculation.

**Decisive code:** The schema makes `cpor`, `cperm`, `cgd`, and `csw` nullable (`src-tauri/src/db.rs:638-647`), and the legacy set migration copies those measurement columns unchanged into the new table (`src-tauri/src/db.rs:2291-2306`). `fetch_core_series` says each property is an independent non-NaN series (`src-tauri/src/equations.rs:300-306`), but its query reads every nullable cell directly as `f32` at `src-tauri/src/equations.rs:307-327`. DuckDB cannot convert SQL NULL to `f32`, so iteration returns an error before the later NaN filter at `src-tauri/src/equations.rs:335-341`. `src-tauri/src/lib.rs:804-812` propagates that one row error for the whole `get_core_data` command. The sibling reader at `src-tauri/src/db.rs:4211-4239` shows the correct shape: read `Option<f32>` and independently drop missing properties.

**Concrete failure scenario:** Open an older project whose pre-set `core_data` contains a valid plug at depth 1000 with `CPOR=0.20`, `CGD=2.65`, and SQL NULL in CPERM/CSW. The set migration preserves those NULLs. Opening a core overlay then causes `row.get::<_, f32>` on CPERM or CSW to fail, so the command returns an error and neither the available CPOR nor CGD point is plotted. Replacing the two NULLs with finite values—or merely reading through the sibling `Option<f32>` reader—makes the unrelated porosity/grain-density points available, demonstrating that the failure is null conversion rather than absence of plottable data.

**Self-refutation attempt:** I checked the current import and edit writers and found that they pass `f32::NAN`, not SQL NULL (`src-tauri/src/db.rs:2563-2598` and `src-tauri/src/db.rs:9711-9723`), so a newly imported blank cell does not reproduce this failure; that is why this remains P2. It does not save an already existing nullable row: the schema and legacy migration admit/preserve it, the overlay query has no `Option<f32>`/`COALESCE` guard, and the Tauri caller handles only an all-series success. No inspected migration normalizes nullable measurement columns to NaN, and no test exercises such a migrated row through this command.

## Areas checked and found clean

- **Exact-object and contract checks:** `master` resolved to the requested full hash; code was inspected from that object only. `CLAUDE.md` and all allowed `docs/record_*.md` build records were read before adjudication.
- **LAS/DLIS and core unit reconciliation:** LAS index-unit adoption/conversion, core-wizard depth-unit conversion, project-unit persistence, and LAS export/re-import depth handling correctly preserve one project depth unit. The omissions in the other import surfaces are reported above.
- **Main interactive log scale:** the normal log canvas distinguishes stored project units from display units and applies the physical metre/foot factor; the composite exporter is the exception reported above.
- **Computed-curve write discipline:** production replacement paths inspected use an atomic delete-before-append current write plus append-only archive/version custody. Direct bare inserts found by repository-wide search were schema migrations or test fixtures, not production replacement writers.
- **Missing-value handling in principal curve stores:** standard/generic/computed curve reads, LAS/DLIS null/sentinel ingestion, module output buffers, and ordinary packed curve fetches preserve missing continuous values as NaN/SQL NULL rather than `0` or `-999`; the specific coercions and nullable overlay failure are reported above.
- **Active-set filters:** ordinary readers for SCAL, auxiliary data, well images, surveys, and core deliveries consistently select the active delivery. The remaining core issue is cross-datum refusal on particular pairing readers, not stale-set mixing.
- **Byte IPC on primary data paths:** standard curves, generic curves, array logs, track curves, and core point-series packing use raw byte responses and typed-array reconstruction. The two JSON-array exceptions are reported above.
- **Pay aggregation mechanics:** sample-slab boundary clamping, `gross = net + not-net + unknown` reconciliation, the separation of unknown from evaluated zero, porosity- versus thickness-weighted means, and HPV accumulation were internally consistent. The unit labels and sweep PERM inconsistency are reported above.
- **Core saturation equations:** Archie total/effective, Simandoux, Indonesia, Juhasz/Waxman-Smits solver convergence refusal, and the main SSC RMS gas branch matched their declared algebra and preserved unclipped versus clipped diagnostics. The SSPW, RtC-gap, and IMTS-gap exceptions are reported above.
- **Saturation-height physics after coordinate formation:** Leverett pressure uses feet of column, Skelt uses metres, and HAFWL is converted to metres. The remaining defect is the FWL coordinate unit shown to the user.
- **Thomeer/Swanson:** the Thomeer model uses the declared `log10(Pc/Pd)` denominator, fits displacement pressure in log10 space, and uses logarithmic sampling only for display-curve spacing; no ln/log10 substitution defect was found.
- **HFU and Lorenz core math:** RQI/FZI, porosity screening, Ward ordering, Lorenz cumulative fractions, and the dimensionless Lorenz coefficient were consistent. Only project-unit tolerance and capacity labels failed.
- **Monte Carlo and distribution statistics:** deterministic seeding, finite-value filtering, marginal-preserving correlation reordering, triangular inverse CDF, type-7 percentile interpolation, no-data NaNs, convergence handling, histogram range handling, and the shared Ward dynamic program were internally consistent.
- **Broader application code:** project open/compact/migration, query write-whitelisting, Python subprocess framing, undo custody, ML frame assembly/persistence, map/correlation datum transforms, and build/release tooling received a repository-wide static scan; no additional concrete wrong-number or data-loss scenario survived caller/test/guard refutation.
