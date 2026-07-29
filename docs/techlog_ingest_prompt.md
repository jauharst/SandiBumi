# Techlog ingest prompt (feature mining for SandiBumi)

A reusable prompt for ingesting the **local Techlog 2018.2 material** and extracting
everything implementable in SandiBumi — the Techlog counterpart of the Geolog-V14 install
anatomy work (Jul 2026: `.lls` sources, `.info` manifests, `.paysum` specs, `alias.alias`).

**Source (verified on disk 2026-07-22, no placeholders needed):**
`D:\01. Work\00. Guidebook\03. Guidebooks Techlog\` containing:
- `Techlog 2018.2 (r22885)\` — a real install tree: config/catalog XMLs at root, 538 shipped
  Python sources, 112 digitized vendor-chart XMLs, 360 layout templates, 3,808 offline HTML
  doc pages, Quanti workflow XMLs, mineral tables.
- Root PDFs: `Techlog_Quanti Elan 2015.PDF` (19.5 MB), `Techlog Fundamentals 2015.PDF`,
  `335269406-Techlog-2011-Training-Course.pdf`, `Techlog_Manual.pdf`.
- `Training Techlog\Dataset\` — 36 LAS files + csv/txt (test-fixture material).

**How to use it:** run the master prompt below as a fresh Claude Code session (multiple
sessions fine — targets A–I are independent; do A/D/E first, they're the highest value).
Outputs go to `D:\XX. SandiBumi\docs\research_<YYYY-MM>\techlog_ingest\`. If the `sw-techlog`
skill is available, load it for concept grounding (Families/Aliases, Quanti, Quanti.Elan,
LogView).

Domain knowledge lives in this repo's `docs/`, not machine-local memory — update this file
if the extraction checklist needs to evolve.

---

## 1. The master prompt (copy, optionally trim to a subset of targets, run)

```
Ingest the local Techlog 2018.2 material at
"D:\01. Work\00. Guidebook\03. Guidebooks Techlog" and extract everything useful for
SandiBumi development (the petrophysics application at D:\XX. SandiBumi). The install tree is
"Techlog 2018.2 (r22885)" inside that folder — call it TL below. Write all outputs to
D:\XX. SandiBumi\docs\research_<YYYY-MM>\techlog_ingest\ ; treat the source folder as
strictly READ-ONLY.

Ground rules:
- IP discipline: catalog-style DATA (mnemonics, families, units, chart points, endpoints,
  defaults, validation ranges) may be extracted verbatim into structured files. Algorithm
  CODE (TL\PythonScripts) may be read to identify which published equation a method
  implements — SandiBumi implementations must be written independently from the published
  equations (cite SPE/SPWLA/textbook source in the findings), never copy-pasted. Never
  bundle Techlog files/assets into the repo outside the research folder.
- The file inventory below was verified on disk 2026-07-22. If something is missing, note
  it and move on — don't hunt for hypothetical files.

### Extraction targets (one numbered output file each)

A. FAMILY / ALIAS / UNIT CATALOGS — Techlog's data-model backbone (analog of Geolog's
   alias.alias, far larger). Parse from TL root:
   FamiliesTechlogList.xml, FamiliesCompatibilityTechlogList.xml,
   TechlogFamilyAssignment.xml (+ _CoreDb variant), AliasesTechlogList.xml,
   MnemoTechlogList.xml, MnemoPoscList.xml, MnemoTechcoreList.xml,
   TechlogMnemoAlias.xml, TechcoreMnemoAlias.xml,
   SystemsUnits.xml, TechlogUnitSystem.xml, TechlogUnitAlias.xml, TechcoreUnitAlias.xml,
   VariablePrefixes.csv, VariableSuffixes.csv.
   Export as JSON/CSV: family taxonomy (hierarchy, default unit, expected range),
   mnemonic→family assignment rules (incl. vendor/tool context and priority), unit
   conversion factors, alias tables. Feeds SandiBumi import-time auto-aliasing
   (ingest.rs/parsers.rs) and any future unit system.

B. DIGITIZED VENDOR CHARTS — TL\Charts\ has 112 XML chart files (Baker Hughes, Gearhart,
   General Electric, Schlumberger, generic: neutron-density, neutron-sonic, Bowers, FZI
   poro-perm, …) plus TechlogMnemonicChartAutoSelect.xml (mnemonic→chart auto-selection
   rules). Parse the XML format ONCE, then batch-extract every chart's curve/point data to
   JSON. This is machine-readable chart data — compare coverage and values against
   SandiBumi's PDF-digitized charts (neutron_charts.rs, chartOverlays.ts, tools/chartdig;
   tolerance convention ~0.04 pu per CLAUDE.md). Flag charts we digitized by hand that
   exist here in vector form, and vendor charts we don't have at all.

C. QUANTI WORKFLOW DEFINITIONS & PARAMETERS — TL\Quanti\*.xml (18 workflows: neutron-density
   total/effective, density total/effective, precomputations, CMR suite, pore-pressure,
   synthetic density), TL\WorkflowParameters\, TL\CPMParameters\, TL\ProjectParameters\,
   TL\QuantiTemplates\, TL\Wizards\, TL\Settings\. Extract per-method parameter names,
   defaults, min/max validation, units, enums, conditional logic — the analog of Geolog's
   .info manifests; feeds SandiBumi's auto-generated module dialogs.

D. SHIPPED PYTHON ALGORITHM SOURCES — TL\PythonScripts\ (538 .py in packages: EnvCorr,
   Acoustics, RockPhy, PPP, RST, Preprocessing, Techlog [the scripting API package]).
   Priorities: (1) EnvCorr — environmental/borehole corrections with vendor-specific
   branches (compare against gr_hole_corr / nphi_env_corr / rhob_hole_corr in modules.rs);
   (2) RockPhy; (3) Acoustics; (4) Preprocessing. For each relevant module: path, exact
   formula/constants as implemented, which published equation it matches, and any deviation
   from SandiBumi's implementation (quote both sides). Also map the Techlog package's public
   API surface as a feature checklist for python_engine.rs.

E. QUANTI.ELAN / MINERAL ENDPOINTS — TL\Minerals.xml and TL\QM_MineralTable.xml, plus the
   19.5 MB "Techlog_Quanti Elan 2015.PDF" at the source root (methodology: solver, weights,
   uncertainties, constraints, model combination). Extract the full mineral/fluid endpoint
   library (density, neutron HI, DT, PEF/U, sigma, GR/Th/K, …) to JSON and diff against
   docs/multimin_ref_spec.md — cell-by-cell where minerals overlap. This is a direct
   cross-check of SandiMin (multimin2.rs).

F. DISPLAY / PRESENTATION TEMPLATES — TL\LayoutTemplates\ (360 xml), TL\TemplateTracks\
   (74 xml), TL\Palettes\, TL\Headers\, TL\patterns\, TL\Symbols\, TL\LithologyCatalog\,
   TL\CrossPlots\ (+3D/MultiWells), TL\CPI\, TL\DashBoardTemplates\. Extract the template
   schema and a representative set (standard triple-combo, CPI/results layout, D-N
   crossplot): track widths, curve scales lin/log + limits, colors/line styles, fills
   (crossover, VSH shading), lithology pattern conventions, header layout. Feeds
   composite.rs defaults and a future plot-template system.

G. IMPORT/EXPORT MACHINERY — TL\Dlis.xml, TL\Las.xml, TL\Geolog.xml (import mapping
   configs), TL\dlis\, TL\Data\, TL\Connectors\, DatasetStep.csv, WellParameters.txt,
   wellStatus.xml, propertyDict.xml. Extract mapping/null/depth-index rules and compare
   against SandiBumi's known import defect families (duplicate/non-monotonic depth, null
   depth column, TDEP/MD index naming, DLIS null sentinels — see REVIEW.md). Geolog.xml is
   also a Rosetta stone between the two systems' naming — extract fully.

H. OFFLINE DOCUMENTATION & TRAINING PDFs — TL\Doc\ (3,808 HTML pages): build an index of
   method/equation-level pages; extract equations + parameter definitions for everything
   touched in C–E (docs are more citable than code). The three root training PDFs
   (Fundamentals 2015, 2011 Training Course, Techlog_Manual) are context — mine them only
   where a target above needs methodology explanation, don't summarize them wholesale.

I. TRAINING DATASET — "Training Techlog\Dataset\" (36 LAS + csv/txt): catalog wells, curves,
   depth ranges, and quirks. Evaluate as import/regression test fixtures for SandiBumi
   (pipeline_blso_test-style); note any LAS oddities worth adding to parser tests.

### Findings report (FINDINGS.md)
- Per target: what was found, extraction completeness (% and what was skipped), output file.
- Ranked FEATURE SHORTLIST table: Techlog capability → what SandiBumi has today (name the
  real file: modules.rs, multimin2.rs, composite.rs, ingest.rs, equations.rs,
  neutron_charts.rs…) → gap → effort (S/M/L) → value for Mahakam-delta/LRLC workflows.
  Rank by value-per-effort.
- Discrepancy list: every place Techlog's constants/defaults/formulas/chart values disagree
  with SandiBumi's code or docs/ specs — potential bugs OR deliberate choices; flag, don't
  fix.
- Adversarial pass before reporting: for each shortlist item, try to refute "worth
  implementing" (already exists? Geolog ingest already covered it? niche for our
  workflows?). Only survivors go in the shortlist; say plainly if a target yielded nothing.

Do NOT modify anything in D:\XX. SandiBumi except writing into the research output folder.
Implementation happens later, serially, in the main working tree per house convention.
```

---

## 2. Calibration: Geolog-V14 ingest vs this Techlog tree

| Geolog artifact (mined Jul 2026) | Techlog counterpart (verified on disk) | Target |
|---|---|---|
| `specs\alias.alias` | root family/mnemonic/unit XML catalogs (~15 files) | A |
| `loglan\*.info` manifests | Quanti/Workflow/CPM parameter XMLs | C |
| `loglan\*.lls` (~120 modules) | `PythonScripts\` 538 .py (EnvCorr, RockPhy, …) | D |
| `specs\*.paysum` | Quanti workflow XMLs / Doc pages | C/H |
| — (no Geolog equivalent) | `Charts\` 112 digitized vendor charts | **B** |
| — | `Minerals.xml` + `QM_MineralTable.xml` + Elan PDF | **E** |
| — | 360 LayoutTemplates + 74 TemplateTracks + palettes | F |
| — | `Geolog.xml` cross-system mapping | G |
| — | 36-LAS training dataset | I |

## 3. Practical notes

- This is a 2018.2 tree, not current Techlog — fine for our purpose (catalog structure and
  physics don't churn), but note the vintage in FINDINGS.md when citing values.
- Target B overlaps the chartbook-digitization effort (SLB 2013 chartbook, tools/chartdig):
  the Techlog chart XMLs may replace hand-digitization for vendor charts entirely — check B
  early, it can retire planned work.
- `bin64\` and the bundled Python runtimes are opaque/irrelevant — skip binaries entirely.
- Highest value first: A (aliasing), E (SandiMin cross-check), B (charts), then D, C, F, G,
  H, I.
