# UAT — Rounds 5–8 (constants / TVD / ML MASK / DLIS shadow)

Structured acceptance tests for the four feature_work items shipped 2026-07-22
(commits `8e7b64f`, `34a22bf`, `c006fbe`, `7701b1a`). This is a **separate** file from
`manual_test_plan.md` — add or fold these in as you see fit.

Run in `npm run tauri dev` on a real project (BLSO fixtures / `Testdata.las`, plus at least one
**deviated** well with a deviation survey, and one well imported from **both** an LAS and a DLIS).
Mark each case `PASS` / `FAIL` (+ note). A `FAIL` is anything where the observed result differs from
**Expected**; capture the numbers like the 540-well notes so a fix can be logged.

---

## Round 5 — Rock-typing constants (rocktyping.rs)

Changed: GHE bin upper bounds are now `3, 6, 12, 24` (were `2.5, 4, 6, 8`); the Permadi-Susilo PGS
default exponent is `3.0` (was `3.5`) and the geometric factor is `sqrt(k/phi)`. Pittman r75 row was
intentionally **left unchanged** (pending the AAPG paper).

| # | Precondition | Steps | Expected |
|---|---|---|---|
| 5.1 | A well with PHIE + a permeability curve (core or computed) | Run Rock Typing (Amaefule/GHE) | Samples with FZI just above/below the new boundaries land in the **new** GHE class (e.g. FZI≈6 sits on the 3–6/6–12 edge, not the old 4/6 edge). No sample errors or NaN classes. |
| 5.2 | Same well | Inspect the GHE class track / legend | Bin edges read 3 / 6 / 12 / 24; classes are monotonic in FZI (ascending). |
| 5.3 | Same well | Run Permadi-Susilo PGS with default exponent | Exponent field defaults to **3.0**; PGS values are finite and monotonic with k/φ. Re-running with 3.5 (manual) gives a visibly different curve — confirms the default actually drives the result. |
| 5.4 | A well with core Pc/MICP if available | Compare Pittman r35 vs r75 outputs | r35/r50 behave; note that **r75 is a known-held row** — if r75 looks inconsistent with r35/r50, that is the parked item, not a new regression. |

---

## Round 6 — TVD/TVDSS as fetchable curves (deviation.rs / ingest.rs)

Changed: importing a deviation survey now materializes `TVD` and `TVDSS` as **computed** curves on
the log depth grid, so height-based tools can consume them by name.

| # | Precondition | Steps | Expected |
|---|---|---|---|
| 6.1 | A **deviated** well with logs loaded | Data ▸ Import Deviation… a survey | `TVD` and `TVDSS` appear as computed curves (Curve Catalog, and any module's log-input dropdown). In the built interval TVD < MD; TVDSS = KB − TVD. |
| 6.2 | Same well | Run Saturation-Height picking the new `TVD` for the TVD input | HAFWL / SWH use true vertical depth, not MD. On a strongly deviated well this changes the height and reduces optimistic pay vs the old MD fallback. |
| 6.3 | Same well | In Cuddy FOIL / Brooks-Corey / Skelt / Thomeer, pick `TVDSS` as the vertical-depth input | The fit runs and uses TVDSS. |
| 6.4 | Two+ deviated wells with contacts | Correlation panel, switch to TVDSS depth mode | Depth mode works from the survey (not only from an imported TVDSS log); a flat contact reads flat in TVDSS. |
| 6.5 | Logs imported **after** the survey, or a KB edit | Data ▸ Recompute TVD/TVDSS Curves | Status reports "computed for X of Y surveyed well(s), N samples"; surveyed-but-log-less wells count as pending. **Note:** survey-derived TVDSS lives in the computed store and **outranks** an imported TVDSS log of the same name — verify that is the intended winner for your data. |

---

## Round 7 — MASK support in the ML pipeline (ml.rs)

Changed: an optional flag curve in the ML dialog. Convention (same as module MASK): a mask value
**== 1.0 excludes** a sample; 0 / NaN / absent keeps it.

| # | Precondition | Steps | Expected |
|---|---|---|---|
| 7.1 | A well with a 0/1 flag curve (BADHOLE / FLAG_PAY / COAL) | ML Models → pick a **Mask (exclude)** curve → run a regression or classification | The output curve is **blank (NaN)** at flagged depths, and the per-well "Predicted samples" count drops by the flagged count. |
| 7.2 | Same well | Run clustering or PCA with the same mask | Flagged samples are kept out of the fit **and** left blank — facies with vs without the mask differ (bad-hole must not shape facies). |
| 7.3 | A training set where the mask empties one whole training well | Compare algorithms with that mask | The header shows the **true contributing-well count**, and a note says blind-well CV fell back to random KFold (it must not silently report the requested well count while CV has collapsed). |
| 7.4 | Any well, no mask | Run ML without selecting a mask | Behaviour is unchanged from before this feature (regression check). |

---

## Round 8 — DLIS/LAS mnemonic-shadow resolution (inspector ▸ Curve Catalog)

Changed: same-mnemonic collisions in the imported (RAW) store are detected; the resolver's winner is
badged; Promote/Delete let you resolve them. Resolution priority is **standard log column → computed
curve → imported (RAW) store**, so Promote only has an effect where the RAW store actually resolves.

| # | Precondition | Steps | Expected |
|---|---|---|---|
| 8.1 | A well where a DLIS and an LAS both carry a **non-standard** mnemonic (`PEF`, `CALI`, `DTS`, or a core `PERM` with no computed PERM) | Inspector ▸ Curve Catalog | The two rows show **`resolves`** / **`shadowed`** badges. |
| 8.2 | (from 8.1) | Click **Promote** on the shadowed row | It flips to `resolves` (+ `pinned`); any plot/module reading that curve now shows the promoted values. |
| 8.3 | (from 8.1) | **Delete** the losing sibling (two-click confirm) | It's removed; the surviving curve resolves. |
| 8.4 | A well with LAS+DLIS `GR` (or RHOB/NPHI/DT/SP) | Curve Catalog, look at those rows | They show a neutral **`served by log`** badge and **Promote is disabled** (tooltip explains the standard log column resolves). Clicking is impossible — no false "it now wins" toast. |
| 8.5 | A well with a computed curve (e.g. `PERM` from Coates) **and** an imported raw curve of the same name | Curve Catalog, the raw row | Shows **`served by computed`**, Promote disabled — promoting the raw one would be a silent no-op, so it's blocked. |
| 8.6 | A well whose deep resistivity feeds Sw, plus an unrelated same-mnemonic shadow | Promote the unrelated shadow, then re-run Sw | **Sw is unchanged.** A pin on one mnemonic must not change which curve a family (e.g. RES_DEEP) request resolves — and the choice is stable across re-import / reopen. |

---

_Regression bar for all four: `cargo test --lib` 255/0/7 and `tsc --noEmit` clean as of `7701b1a`._
