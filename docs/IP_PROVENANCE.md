# Reference-data provenance register

**Created 2026-07-29 in response to `docs/PRD.md` §9 risks R1–R3.**

SandiBumi is intended to be **licensed to third parties** (`docs/PRD.md` §8). That changes the
status of every piece of reference data in the repository: what is unremarkable in a personal tool
becomes something a buyer's counsel may ask to see documented. Until now the discipline existed —
`docs/sandibumi_maturation_prompt.md` defines a four-tier IP system and the "re-derive, don't port"
doctrine, and the code carries provenance comments — but **no single record collected it in one
place**, so "show me where this came from" had no answer short of reading the source.

This file is that record. It is a **factual inventory, not a legal opinion.** Neither Jauhar nor
Claude is qualified to give one, and nothing here should be read as clearance. Its purpose is to
make each question precise enough that a lawyer can answer it in minutes rather than days.

**Maintenance rule:** any new asset derived from an external source gets a row here in the same
increment that introduces it. A row added later is a row that was forgotten once already.

**Machine-enforced slice:** `THIRD-PARTY-LICENSES.md` is generated from Cargo's normal dependency
edges and npm's installed production graph. `tools/check.ps1` runs the generator in `--check` mode,
so a dependency change cannot leave that distributed-package inventory stale. This is only the
dependency slice. Source code cannot reveal an origin record that was never written, so the
asset/default rows below remain an explicit human-maintained register; unresolved rows continue to
block first sale rather than being treated as cleared by a green build.

---

## 1. Tier definitions

Carried from `sandibumi_maturation_prompt.md` so this file stands alone.

| Tier | Meaning | Treatment |
|---|---|---|
| **A** | Factual/catalog data with no creative authorship claim asserted by us — mnemonics, units, family names, physical constants | Used directly |
| **B** | Published method with a citable primary source (SPE / SPWLA / textbook) | **Re-derived** from the publication, never transcribed from a vendor implementation; source cited in code |
| **C** | Material we believe is protected, or whose status is unclear | **Blocked** — not used until cleared; tracked in the standing Tier-C register |
| **D** | Our own work | Unrestricted |

The doctrine that matters: **algorithm code from a vendor install may be *read to identify which
published equation a method implements*, and the SandiBumi implementation is then written from that
publication.** No vendor source is copied, and no vendor file is bundled outside the read-only
research folders.

---

## 2. Register

### 2.1 Digitized chart data — **the highest-exposure item (PRD R1)**

| | |
|---|---|
| **Assets** | `src/ui/chartOverlays.ts` (19 chart definitions) · `src-tauri/src/neutron_charts.rs` (Por-4 / Por-5 equivalence tables) |
| **Source** | *Schlumberger Log Interpretation Charts*, 2013 edition |
| **How derived** | Vector digitization by `tools/chartdig` from a PDF of the chartbook. Both files declare it in their own headers. |
| **Source file in repo?** | **No.** `chartbook.pdf` is explicitly excluded and lives outside the repository; `tools/chartdig` locates it through the `CHARTBOOK_PDF` environment variable (hard-coded to a reference-machine path until 2026-07-31). |
| **What ships** | The extracted numeric values — curve graduation coordinates, mineral points, region polygons — as TypeScript/Rust data, plus code that renders them as plot overlays. |
| **Tier as recorded** | Treated as A (factual data points describing physical relationships). |
| **The open question** | Whether the *coordinates of a published chart's curves* are a protectable expression of that chart or unprotectable facts about a physical relationship — and whether the answer differs when redistributed inside commercial software. **This is a lawyer question and it is the single most exposed item in the product.** |
| **If the answer is unfavourable** | Fallback options, in increasing cost: (a) cite the chart and require the user to own the chartbook; (b) re-derive the same relationships from the underlying primary publications and tool-physics papers rather than from the chart images; (c) drop the overlays as a shipped feature and expose the digitizer as a user-side tool operating on a chartbook the user owns. Option (c) preserves the code and moves the data question to the customer. |
| **Owner ruling (2026-08-19, DEC-078)** | Route (b) for the ten definitions a primary source exists for (the Por-20/22 equation curves; the lithology charts computable from cited mineral constants) — each re-derivation replaces the digitized coordinates as it is executed. The nine neutron **tool-response** definitions (`por11`–`por19` set, listed in `src/ui/chartOverlayPolicy.ts`) have no independent primary source in principle; Jauhar retains them cited + fail-closed **through Gate 5** for manual/field verification, with delete-at-final-commercial kept a single named operation (regenerate `chartOverlays.ts` without those ids). Counsel disposition under CLAIM-013 remains open for whatever ships at first sale. |
| **Urgency** | **Before first sale.** |

### 2.2 Vendor-derived parameter defaults (PRD R2)

| | |
|---|---|
| **Assets** | `src-tauri/src/multimin2.rs` — the 27-entry component `LIB` and its endpoint matrix across 14 tool keys (RHOB, NPHI, DT, GR, PEF, U, THOR, POTA, URAN, VP, VS, EPT, EATT, SIGMA) |
| **Source** | Endpoint defaults merged from two vendor installs, in Interactive Petrophysics' mineral-dropdown order. Specs extracted to `docs/multimin_ref_spec.md` and `docs/multimin_ip_spec.md`. |
| **In-code note** | `multimin2.rs:2048` — "Merged reference/IP default library, in IP's mineral-dropdown order". |
| **Tier as recorded** | A (published mineral physical properties — grain density, neutron response, Pe and so on are measured facts appearing in many public sources). |
| **The open question** | Individual mineral endpoints are physical constants and are published widely. **The compiled, ordered selection may be a different question from the values themselves** — a curated table can carry more claim than any of its rows. |
| **Recommended action, independent of the legal answer** | Add a primary-literature citation per row where one exists (Schön, Ellis & Singer, SPWLA references). That converts "merged from vendor installs" into "sourced from the literature, cross-checked against vendor defaults", which is both a better engineering record and a materially stronger position. **This is work SandiBumi should do anyway** and it is currently unscheduled. |
| **Outcome (2026-08-19, DEC-078)** | Done, in the owner-adjudicated form: Jauhar ruled the library **his default library**, and per-value custody now ships in code (`multimin2.rs` `LibRow::src` + `Component::endpoint_sources`, carried through the SandiMin dialog and every run record's `params_json`). Every value was verified against his copy of the *Schlumberger Log Interpretation Charts* (2013): values the book states in print (Appendix B pp. 279–280, Appendix C p. 281, chart Por-1 p. 212) cite the page; every other value — including all near-misses, where the printed number differs from the library's — is owner-attributed to DEC-078. Shipped strings no longer claim vendor-install custody; **this section remains the custody history and is deliberately not rewritten**. CLAIM-012's counsel review stays open as a first-sale item. |
| **Urgency** | Engineering custody complete; counsel disposition (CLAIM-012) before first sale. |

### 2.3 Mnemonic and family dictionary

| | |
|---|---|
| **Asset** | `registry/unit-registry.json` (alias → family, typed unit canonicalization), including the independently assembled project mnemonic table; generated Rust, import-UI, documentation and test consumers carry one version and digest. |
| **Source** | Assembled from public mnemonic conventions and the project workflow record; `tools/unit-registry.mjs --check` rejects drift or dimension disagreement without adding an alias or interpreting an opaque vendor file. |
| **Tier** | A — mnemonics and units are industry-standard identifiers, not authored content. |
| **Assessment** | Lowest exposure in this register. The source custody explicitly records an independently assembled vocabulary rather than a wholesale vendor alias-file transcription; any later alias still needs its own reviewed source. |
| **Urgency** | Confirm at leisure. |

### 2.4 Method implementations

| | |
|---|---|
| **Assets** | `ssc.rs` (SSC / SSPW) · `lrlc.rs` (RtC / IMTS) · `modules.rs` saturation, porosity, VSH and permeability families · `satheight.rs` · `multimin2.rs` solver |
| **Source** | Published methods (Archie, Indonesia, Simandoux, Waxman-Smits, dual-water, Thomas-Stieber, Passey, Leverett-J, Thomeer …) plus **Jauhar's own research and reference projects** for SSC/SSPW and the LRLC methods. |
| **Tier** | B for published methods, **D for Jauhar's own** — SSC/SSPW and LRLC RtC/IMTS originate in his work and are specified in this repo (`docs/method_ssc_sspw.md`, `docs/method_lrlc_rtc_imts.md`). |
| **Assessment** | Strongest position in the register. Published equations are not protectable; the implementations are original; the specs are banked in-repo and predate the code. The house rule that `docs/` wins over code when they conflict also produces the audit trail. |
| **Action** | Keep citing the primary source in a comment when a method is added — already the convention (`CLAUDE.md`, collaboration protocol §5). |

### 2.5 Third-party names in shipped code and copy (PRD R3)

| | |
|---|---|
| **Assets** | `src/theme.ts` theme ids `pertamina`, `halliburton`, `schlumberger`, `lapi-itb`, with matching `:root[data-theme=…]` blocks in `src/styles.css` |
| **Purpose** | Client-branded palettes, so a deliverable or a screen shown to a given client matches that client's visual identity. This is a deliberate feature, not decoration. |
| **Status** | **Unchanged, and escalated rather than decided.** These are third-party trademarks shipped inside software intended for sale. |
| **Why no code change was made** | Two of the four (Pertamina, LAPI-ITB) are Jauhar's *clients*; Halliburton and Schlumberger are service companies he may also deliver to. Renaming them to generic labels would remove the feature's entire purpose — a user cannot pick "the Halliburton one" from a list of colour names. Whether a client-branded theme is acceptable use, nominative use, or requires permission is **a legal question that a silent rename would paper over rather than answer**. |
| **Options for Jauhar** | (a) leave as-is pending advice; (b) obtain written permission per client — plausible, since these are relationships he already has; (c) ship neutral palette names and let the user rename a theme locally, moving the mark off the shipped artefact; (d) ship only `light`/`dark`/`white` and supply client palettes on request. |
| **Urgency** | Before first sale. |

**Resolved in the same pass:** `README.md` described the module library as "the reference suite-class"
and SandiMin as "the reference suite-Multimin-class" — describing the product by reference to a
competitor's product line, in the primary customer-facing document. Replaced with descriptive
wording ("a full library of deterministic petrophysical modules", "simultaneous multi-log
inversion"). The same phrasing persists in internal documents (`CLAUDE.md`, `ROADMAP.md`), which is
acceptable while they stay internal, and would need the same treatment before publication.

### 2.6 Third-party software dependencies

Ordinary open-source consumption: Tauri, DuckDB, dockview-core, CodeMirror, Vega/Vega-Lite,
rayon, tokio, bytemuck, and the Python-side `numpy`, `dlisio` and `scikit-learn` (invoked as
subprocesses, not linked).

**Current inventory:** `THIRD-PARTY-LICENSES.md` is generated from the installed distributed
dependency graphs and is freshness-checked by the full gate. It reports every package's declared
licence, calls out the current MPL-family weak-copyleft set, and refuses a stale file. This is a
factual inventory, not counsel's compatibility decision.

**Action before first sale:** counsel reviews the generated attention items and confirms the notice,
source-offer and redistribution obligations for the paid binary. Python packages are **not currently
distributed** with SandiBumi; they are prerequisites invoked as subprocesses. If the Gate 3 offline
runtime pack bundles them, that pack's exact locked contents and notices must enter this generated
inventory before release rather than inheriting the present prerequisite-only statement.

---

### 2.7 Client-derived material — **the tier this register did not have**

Added 2026-07-31 by the provenance sweep (`docs/provenance_sweep_prompt.md`). Sections 2.1–2.6
answer *"where did this vendor data come from?"* They do not ask the other question: **which
parts of this repository came out of a client's wells.** That question has a different answer,
a different counterparty and, unlike the vendor questions, a written contract that already
governs it.

**No client is named in this file.** The identified findings, with `file:line`, live in
`docs/commercial/PROVENANCE_SWEEP.local.md`, which is gitignored — a register that leaks the
identifiers it exists to control would be self-defeating.

| | |
|---|---|
| **What was found** | Four kinds. (a) **Analytical work product shipping as a default** — a two-point GR normalization reference fitted on 562 wells of one operator's field, shipping in the module manifest as though it were a constant, and an excess-conductivity regression from a single study. (b) **Client measurements in the tree** — a tracked CSV of real core-plug analyses (porosity, permeability, saturations, grain density, lithology descriptions) from a named well, referenced by no code. (c) **Delivery manifests** — twenty absolute paths in `#[ignore]`d tests, each naming operator, contract, project and well. (d) **Study citations** in method specs, source comments and research documents. |
| **How derived** | Consulting studies performed under confidentiality agreements. The methods are Jauhar's; the *calibrations* were fitted to, and the *files* delivered from, client data. |
| **What ships in the binary** | Only (a) and the comments in (d). (b) and (c) never reach a user — but they travel with any copy of the repository, which is the exposure that matters for source escrow, a technical due diligence, or an employee. |
| **Tier** | Split, and the split is the whole point. The **method** is Tier D (Jauhar's own — §2.4). The **calibration fitted to a client's wells** is not obviously his to redistribute, and is treated as C pending advice. A number is not a method. |
| **The open question** | *Under a typical Indonesian upstream consulting agreement, who owns a regression coefficient fitted to the client's log data — and does shipping it inside licensed software constitute disclosure of the client's data?* The physics is unquestionably Jauhar's. Whether the constants are is the question, and it is the same question for every one of his 50+ studies, so answering it once is worth more than any individual fix. |
| **What was fixed without waiting for the answer** | The GR reference pair is now the app's own generic clean/clay endpoints with a doc string telling the user to derive their own — **which is also the petrophysically correct default**: a reference from one basin is silently wrong in another, and normalized GR always looks plausible. The twenty delivery paths became a configurable fixture folder. The test data became the synthetic example wells. None of that needed a lawyer; all of it makes the product better. |
| **What was deliberately NOT touched** | Every honest attribution — the study citation in `lrlc.rs`, the tooltip telling a user which vendor tables seeded a default, the comments recording why a parser rule exists. Removing an attribution while its values still ship destroys the record and looks like concealment. Attribution comes out only when the asset comes out. |
| **Not resolvable in code** | Git history retains the removed files. Surfaced as a decision, not executed. |
| **Urgency** | The shipped default: **done**. The ownership question: **before first sale**, because it recurs across every study and every future calibration. |

## 3. Summary — what is actually blocking

| Item | Exposure | Blocking |
|---|---|---|
| 2.1 Digitized chart data | **High** — values ship, source is a copyrighted chartbook | First sale |
| 2.5 Client-brand theme names | **Medium** — third-party marks in a sold product | First sale |
| 2.2 Vendor-merged endpoint library | **Medium** — values are public facts; the curated selection is the question | First sale |
| 2.6 Dependency licences | **Low, inventoried but not legally cleared** — generated notice is gate-current | First sale |
| 2.3 Mnemonic dictionary | Low | Confirm at leisure |
| 2.4 Method implementations | **Lowest** — published or original, specs banked in-repo | None |

**The one-sentence version for a lawyer:** *this product ships numeric data digitized from a
copyrighted industry chartbook, a mineral-property table assembled with reference to two commercial
competitors' defaults, and four theme palettes named after real companies — are any of those a
problem when the software is licensed commercially in Indonesia?*
