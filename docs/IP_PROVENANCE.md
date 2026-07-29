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
| **Source file in repo?** | **No.** `chartbook.pdf` is explicitly excluded; on the reference machine it lives outside the repo at `D:\01. Work\00. Guidebook\chartbook.pdf`. |
| **What ships** | The extracted numeric values — curve graduation coordinates, mineral points, region polygons — as TypeScript/Rust data, plus code that renders them as plot overlays. |
| **Tier as recorded** | Treated as A (factual data points describing physical relationships). |
| **The open question** | Whether the *coordinates of a published chart's curves* are a protectable expression of that chart or unprotectable facts about a physical relationship — and whether the answer differs when redistributed inside commercial software. **This is a lawyer question and it is the single most exposed item in the product.** |
| **If the answer is unfavourable** | Fallback options, in increasing cost: (a) cite the chart and require the user to own the chartbook; (b) re-derive the same relationships from the underlying primary publications and tool-physics papers rather than from the chart images; (c) drop the overlays as a shipped feature and expose the digitizer as a user-side tool operating on a chartbook the user owns. Option (c) preserves the code and moves the data question to the customer. |
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
| **Urgency** | Before first sale. |

### 2.3 Mnemonic and family dictionary

| | |
|---|---|
| **Asset** | `src-tauri/src/curves.rs` `FAMILIES` (alias → family, unit canonicalization), including a merged "Bunga" mnemonic table |
| **Source** | Assembled from public mnemonic conventions; `curves.rs:6` records that it "mirrors what commercial tool/curve dictionaries and IP's `CurveAlias.txt` do, but kept small and code-resident". |
| **Tier** | A — mnemonics and units are industry-standard identifiers, not authored content. |
| **Assessment** | Lowest exposure in this register. Worth confirming the table was assembled rather than transcribed wholesale from any single vendor alias file, since a verbatim copy of a complete list is a different fact pattern from an independently assembled one. |
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
subprocesses, not linked). No licence audit has been performed.

**Action before first sale:** generate a dependency licence manifest (`cargo about` or equivalent
for Rust, `license-checker` for npm) and confirm nothing carries a copyleft obligation incompatible
with distributing a closed binary. This is routine, automatable, and currently not done. Note that
the Python packages are **not distributed** with the product — they are prerequisites the customer
installs — which is a materially different obligation from bundling them, and is a point in favour
of keeping Python as a prerequisite (`docs/PRD.md` §10.4).

---

## 3. Summary — what is actually blocking

| Item | Exposure | Blocking |
|---|---|---|
| 2.1 Digitized chart data | **High** — values ship, source is a copyrighted chartbook | First sale |
| 2.5 Client-brand theme names | **Medium** — third-party marks in a sold product | First sale |
| 2.2 Vendor-merged endpoint library | **Medium** — values are public facts; the curated selection is the question | First sale |
| 2.6 Dependency licences | **Low but unverified** — routine, not yet run | First sale |
| 2.3 Mnemonic dictionary | Low | Confirm at leisure |
| 2.4 Method implementations | **Lowest** — published or original, specs banked in-repo | None |

**The one-sentence version for a lawyer:** *this product ships numeric data digitized from a
copyrighted industry chartbook, a mineral-property table assembled with reference to two commercial
competitors' defaults, and four theme palettes named after real companies — are any of those a
problem when the software is licensed commercially in Indonesia?*
