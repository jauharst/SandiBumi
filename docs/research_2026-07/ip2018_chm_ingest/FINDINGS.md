# Interactive Petrophysics 2018 help-manual ingest — FINDINGS

Ingested `C:\Program Files\IP2018\Interact.chm` (212 MB compiled HTML help, Interactive
Petrophysics **2018**; publisher PGL / Lloyd's Register / Geoactive) on 2026-08-06, plus the
IP2018 install tree's legacy tool-definition files and ASCII module-parameter references.
Source treated **read-only**; all outputs in this folder.

**Third in the vendor-intelligence series**, after `../techlog_ingest/FINDINGS.md` (Techlog
2018.2) and `../ip_ingest/FINDINGS.md` (IP 2025.3). Different in kind from both: those two mined
*install trees* (catalogs, charts, parameter files); this one mines the **prose manual** — the
only place a vendor explains *why* a default is what it is.

**IP-cleanliness note.** Tier A (reference data, conventions, market intelligence) extracted
verbatim. Tier B (published methods) recorded as name + citation + constants, to be
reimplemented from the primary paper. **Tier C registered by existence only — 32 items
(`F_tierC_register.json`); no protected algorithm read, decompiled, or approximated.** No
vendor tool-definition file, chart table, or `.PAR` was copied into this repo. `.t83` binaries
(94 files) were excluded by instruction and not decoded.

Run as a 7-agent extraction (A–G) plus two deterministic scripted passes (H, I) by the session
model. **No work in this ingest was delegated to a cheaper model** — every numeric extraction
carried silent-wrongness risk.

---

## 0. Headline

**The IP 2018 manual is roughly half rasterized, and that single fact defines the whole ingest.**
926 equation images across 278 pages. The manual renders its equations as GIF/PNG rather than
text — so the most valuable constants in it (**Larionov, Clavier, Stieber Vsh coefficients; the
Waxman-Smits B(T,Rw) form; the lab-to-reservoir Pc conversion**) are *images*, not recoverable
characters.

Four findings follow from that, in descending order of usefulness:

1. **The same manual documents the same math twice — once rasterized, once in ASCII.**
   `swequationsandmethodology.htm` (the *theory* page) carries **103 equation images**. Its
   sibling `swparameters.htm` (the *parameter* page) states much of the same math in plain
   text, because parameter reference entries are prose by nature. **Heuristic, transferable to
   any vendor doc: when a theory page is rasterized, go read the parameter/reference page for
   the same module.** This is what made targets A and D productive at all.

2. **We deliberately did NOT OCR the lost equations.** Vision-reading an exponent produces a
   number indistinguishable from an invented one once it is written to a file, and a wrong
   Larionov coefficient computes, plots, and ships. The GIF paths are recorded
   (`embim23.gif`…`embim38.gif` for the clay chain) so a human can inspect them. **This is the
   single most important discipline decision in the ingest** — it is why the outputs can be
   trusted at all.

3. **The `.hlp` module-parameter files close the IP2025 ingest's stated gap.** That ingest
   concluded IP "does NOT ship core openhole deterministic defaults — they live in the compiled
   exe panels." Not quite: `ClayVol.hlp`, `PhiSw.hlp` and `Cutoff.hlp` are plain ASCII parameter
   references shipping **479 parameters** (70 / 189 / 220), 30 with stated defaults, present in
   **both** IP2018 and IP2025. **Zero mentions of `.hlp` in either prior ingest** — genuinely new,
   and extracted by script, so no model invented anything (`H_module_parameter_reference.json`).

4. **IP2018 ships 539 files that IP2025 does not.** 224 of them are legacy borehole-image,
   dipmeter and acoustic-waveform **tool definitions** the newer release stopped shipping —
   parsed 224/224 with zero errors, all 47 cross-file references resolving. **The old install is
   the only source for this**, and its design conventions (§1-G) are the most directly reusable
   thing in the entire ingest.

**Honest counterweight.** This ingest is *narrow but deep*. 44 of 278 pages were worked — the
petrophysically load-bearing ones. 620 of the 926 formula images sit on pages no agent opened
(§4). And the manual's own core-method defaults remain thin: **no Archie a/m/n in the PhiSw
module, no Rw default, no temperature-gradient default, no Gardner/Bellotti/Lindseth
coefficients** — method names and citations only (`A_porosity_sw.json` gaps). For core constants
the Techlog ingest and the primary literature remain the sources.

---

## 1. Per-target results

### A — Porosity & water saturation ✅ complete (the rasterization workaround)
| Artifact | Count | Output |
|---|---|---|
| Methods documented (name, kind, tier, params, equation, citation) | **34** | `A_porosity_sw.json` |
| Verbatim default statements | **87** | " |
| Distinct parameters catalogued | 126 | " |
| Tier-C flags | 3 | " |
| Recorded gaps | 13 | " |

The productive discovery: **`swparameters.htm` carries ASCII equations** where
`swequationsandmethodology.htm` carries 103 images. *(I initially doubted this claim; a targeted
re-grep proved the agent right and my first grep had silently hit its output limit.)*

**Stated gaps that matter more than the extractions:** no a/m/n in PhiSw (the only Archie
defaults anywhere are Basic Log Analysis's `a=1.0, m=2.0, n=2.0`); no Rw, formation-temperature,
gradient, surface-temperature or reference-depth defaults; **Rw-from-SP gives no equations at
all**, delegating to Schlumberger chart SP-2; no citations for Archie, Simandoux, Indonesian,
Dual Water, Waxman-Smits, Juhász, Wyllie or Raymer-Hunt. `B fact W-S` is rasterized
(`embim119`) — see the verified Juhász B formula already in memory instead.

### B — Clay volume & lithology ✅ complete (and it is mostly a loss report)
| Artifact | Count | Output |
|---|---|---|
| Methods | **25** | `B_clay_volume.json` |
| Verbatim defaults | 53 | " |
| Lithology codes | 39 | " |
| Citations harvested | 9 | " |

**The raster verdict, stated plainly by the agent:** *"Are the Larionov / Clavier / Stieber
coefficients recoverable as text? **NO — lost to raster.** All six GR Vcl forms, plus SP,
Neutron, Resistivity, Other-Linear and all four double-indicator forms, are rasterized GIFs."*

What **did** survive as text, and is verified: **Curved-method branch thresholds `0.55` and
`0.73`**, the identity branch `VclGr = Z for 0.73 < Z < 1.0`, and **Stieber Constant `STB = 2.0`**
(confirmed twice by independent grep). Also the `Clip Low = 0% / Clip High = 98%` pair — see
D-01, the ingest's most consequential resolution.

### C — Cut-offs, summations & Monte Carlo ✅ complete (richest defaults haul)
| Artifact | Count | Output |
|---|---|---|
| Verbatim defaults | **101** | `C_cutoffs_defaults_mc.json` |
| Summation outputs | 28 | " |
| Validation rules | 31 | " |
| Cut-off definitions | 10 | " |

Architecture: *per-zone, per-report multi-criterion flag model* — Zones × Reports × Input
curves, each report evaluating an independent cut-off set per zone.

Monte Carlo specifics worth carrying into SandiBumi:
- Workflow spans **Clay Volume, Porosity SW, Mineral Solver, Basic Log Analysis, NMR, Cutoff,
  Formula, Multi-Line Formula, Fuzzy Logic, Neural Networks** — i.e. error propagation across
  the *whole chain*, not one module.
- **Percentile convention is sign-flipped for Sw**: "the 10th percentile will be the 10th percent
  lowest value of all the simulation results, **except for Sw** where it will be the 10th percent
  highest" — verified verbatim, and exactly the kind of asymmetry that silently inverts a P10/P90
  report.
- **Percentile-independence caveat** (vendor's own): parameters are ranked *independently* at the
  end of the run, so a given percentile's parameter set is not a single coherent realisation.
- Output mnemonics: `XXX MN` mean, `XXX PSD` +1σ, `XXX MSD` −1σ.
- `MonteCarloDefaults.par` read directly from the install and **byte-identical in IP2025**:
  `Results / Yes 10 50 90 -999 -999`, where `-999` means "do not use" and `Yes` means the 10 %
  is the 10-percentile *lowest*.

### D — Mineral solver & Sand-Silt-Clay / Malay model ✅ complete (a headline negative)
| Artifact | Count | Output |
|---|---|---|
| Mineral records | 20 | `D_mineral_solver_ssc.json` |
| Response-equation types | **30** | " |
| Auxiliary relations printed | 17 | " |
| SSC parameters | 31 | " |

**Headline finding is a negative, and a useful one:** *"The IP 2018 help publishes NO mineral
end-point table anywhere in the Mineral Solver chapter"* — verified across all six target pages.
Endpoints live in `MINDEF.PAR` / `MINEQDEF.PAR`. Selecting the equation type *first*, then the
minerals, auto-populates endpoints from those files.

**Consequence: the four-way endpoint cross-check must be a file diff, not a manual read** (§3).

Market intelligence worth acting on — **every customisation surface in IP is a plain ASCII file
in the install directory, not a database** (`MINDEF.PAR`, `MINEQDEF.PAR`, `.neu` tables,
`ElanToIPMapping.par`, `UnitConversion.par`, `MonteCarloDefaults.par`). Plus:
- **Elan import**: `Load Model > File Type 'Elan Model'` reads Schlumberger Elan `.elp` files.
- **Audit-trail pattern**: `Print Parameters to File` writes *"all the models, parameters and
  mixings used in the analysis"* to a `.txt` named after the parameter set. **Adopt this.**
- **Portability trap**: models move between wells via Load/Save *Parameter Sets*, **not** Save/Load
  *Model* — only the former carries the Mixings.
- **Destructive behaviours to design against**: deleted model numbers are never reused; deleting
  a model also deletes its curve set (rename first); Calibration Reset clears silently.
- **No-calculation flag**: any flag value other than 0 or −999 disables calculation over an
  interval, in which zones `Sw = 1.0`, `porosity = 0.0`, mineral volumes zeroed.

### E — Capillary pressure / saturation-height & rock typing ✅ complete (largest single haul)
| Artifact | Count | Output |
|---|---|---|
| SHF functions | **33** | `E_shf_rocktyping.json` |
| Caveats (what invalidates a fit) | 24 | " |
| Recorded gaps | 12 | " |

32 fitted forms for the log-Sw-vs-height route alone, and the manual independently confirms the
count (*"there are 32 models"* = 4 base × 8 variants), which is a genuine internal
cross-validation rather than an agent assertion.

**Tier-C flags: NONE.** No patents, trademarks, "proprietary" language or shipped model files
anywhere in the Pc/SHF chapter — unusual, and it makes this the cleanest chapter to build from.

**Highest-value gap:** laboratory contact-angle and IFT defaults for Mercury Injection /
Centrifuge / Porous Plate are *not stated*; the lab-to-reservoir Pc conversion equation itself is
rasterized. Four SHF functions (Porosity & Pc Function 1/2/3, Porosity & Pc Lambda) exist only as
dialog screenshots.

### F — Environmental corrections, Tier-C register & citations ✅ complete
| Artifact | Count | Output |
|---|---|---|
| **Citations harvested** (as printed, with what each supports) | **124** | `F_citations.json` |
| **Tier-C register** | **32** | `F_tierC_register.json` |
| Manual page map | 278 pages | `F_manual_map.md` |

The 124-citation table is the ingest's best Tier-B asset: it names, for each method, the primary
paper to reimplement *from*, so nothing has to be reconstructed from IP's paraphrase.

### G — Legacy tool definitions ✅ complete (highest reusable-design value)
Targets the **539 files present in IP2018 and absent from IP2025**.

| Format | Count | |
|---|---|---|
| `.itt` | 72 | tool definitions |
| `.att` | 60 | (two schema generations coexisting) |
| `.itp` | 58 | pad definitions |
| `.bor` | 18 | coordinate tables |
| `.eli` | 16 | pad tables |
| **Total parsed** | **224 / 224, zero errors** | 47/47 `ExternalPadID` refs resolve, zero orphans |
| Defaults extracted | 161 | `G_legacy_tool_definitions.json` |
| Mnemonic conventions | 29 | " |

**Reuse assessment is explicit per item** — and the split matters:

| Adopt the convention (no IP risk) | Do not copy (HIGH risk) |
|---|---|
| Tool-class taxonomy (PadBasedTool / LWDTool / AcousticTool / DipmeterTool / CaliperTool / MITTool / Image360Tool / Scan360Tool) | **Per-tool button geometry** — `.itp` ButtonRow offsets, `.bor` coordinate tables, `.eli` pad tables. *This is the redistributable-data core.* |
| Unit declarations in comment headers (inches / degrees / µs / µs-per-ft) | |
| `;`-separated **priority-ordered alias list** on every Curve element | |
| ButtonRow procedural encoding + SourceCurve `[n]` indirection | |
| **The "is this already corrected?" boolean family** — `ButtonsAligned`, `PadsAligned`, `SpeedCorrected`, `SwingArmCorrected`, `MagneticDeclinationApplied`, `CenterReProjectionNotRequired`, `StagesAligned`, `NavigationAligned` — flagged **ADOPT, HIGH VALUE** | |

> The correction-state boolean family is the best single design idea recovered in this ingest.
> It makes "has this processing step already been applied?" an explicit, queryable property of
> the data rather than tribal knowledge — which is precisely the class of silent error that
> ruins a deliverable. SandiBumi should adopt it wholesale.

The role→mnemonic mapping (DEVI/DEV/INC, HAZI/DAZ/AZI1, P1AZ/P1NO, RB, CVEL/SPD/MSPD, AX/AY/AZ +
FX/FY/FZ triads) is marked **adopt as evidence, re-derive the catalogue** — individual mnemonics
are industry facts, but the curated list as a compiled work is not ours.

### H — Module parameter reference ✅ complete (deterministic, no model involved)
`ClayVol.hlp` **70** + `PhiSw.hlp` **189** + `Cutoff.hlp` **220** = **479 parameters**, 30 with
stated defaults. Format is `~ParamName` + description + `(Parameter #N)`. Present in **both**
IP2018 and IP2025. Parsed by script — `stated_default` is populated *only* where the file
literally states one. → `H_module_parameter_reference.json`

### I — Page recoverability audit ✅ complete (and it corrected an earlier error of mine)
Per-page formula/screenshot image counts for all 278 pages. → `I_page_recoverability.json`

**This target exists because my first recoverability estimate was wrong.** I originally reported
"252 of 278 pages fully recoverable, 581 formula images, 26 pages affected." Agent D found my
preprocessing counted only `embim*.gif` and missed `_intclip*.png`. Re-auditing with a
dimension-based classifier (PNG IHDR bytes 16–24 big-endian; GIF header bytes 6–10
little-endian — reading geometry directly, no image library) also surfaced a *third* equation
family, `equation#.zoom#.png` (43 images, 98 % short-and-wide). Corrected figures below.

---

## 2. Recoverability — the corrected numbers

| Measure | Value |
|---|---|
| Pages | 278 |
| **Formula images** | **926** |
| Screenshot images | 4,009 |
| **Pages with zero formula images (fully recoverable as text)** | **179** (64 %) |
| **Pages losing ≥1 formula to raster** | **99** (36 %) |

Worst-affected pages:

| Formulas lost | Page | Covered? |
|---|---|---|
| 103 | `swequationsandmethodology.htm` | ✅ A, B (via `swparameters.htm`) |
| 84 | `easteuroperescorrections.htm` | ❌ |
| 76 | `minsolveeqandmeth.htm` | ✅ D |
| 56 | `fluidsubstitution.htm` | ❌ |
| 51 | `ucr.htm` | ❌ |
| 41 | `rock_strength.htm` | ❌ |
| 38 | `laminatedfluidsubs.htm` | ❌ |
| 35 | `nmrinterpretation.htm` | ❌ |
| 22 | `production_log_analysis_module.htm` | ❌ |
| 20 | `geosteering.htm` | ❌ |
| 18 | `porepressurecalculations2.htm` | ❌ |
| 16 | `clayequationsandmethodology.htm` | ✅ B, D |

**Classifier caveat, stated so nobody over-trusts 926:** the discriminator is geometric — short
and wide ⇒ formula, large in both axes ⇒ screenshot. On UI/IO pages a count of 1–2 is very likely
a misclassified button glyph. **926 is an upper bound**; the figure is reliable in the
double-digit range that actually matters and noisy in the tail.

---

## 3. Discrepancy / review list

Seven internal contradictions found; **five resolved using only evidence internal to the manual,
two left explicitly OPEN rather than guessed.** Full reasoning in `DISCREPANCIES.md`.

| ID | Issue | Status |
|---|---|---|
| **D-01** | `Clip Low %` = 0 % or 98 %? | **RESOLVED — Low 0 / High 98.** `basicloganalysis.htm` duplicated the Clip *High* text block onto the Clip *Low* row, carrying its 98 % with it — proven by the orphaned trailing sentence and the missing Clip High default. **Adopting 98 % would have silently discarded 98 % of the GR population before percentile picking.** The ingest's most consequential catch. |
| **D-02** | Hingle plot Y-axis defined two ways on one page | **OPEN** — no internal evidence settles it. Do not adopt a Hingle convention from this manual. |
| **D-03** | `Stieber` vs `Steiber` | **NOTED, not a defect.** Both spellings are IP's own; alias matching must accept both. No numeric consequence. |
| **D-04** | `RQI` defined two ways | **RESOLVED — namespace collision, not an error.** HFU uses `RQI = 0.0314 × √(K/Φ)`; log-Sw-vs-height uses bare `√(K/ϕ)`. **Do not unify** — a shared `rqi()` silently rescales every fitted coefficient in one path. |
| **D-05** | Pc↔height with/without `0.433` | **RESOLVED — 0.433 psi/ft, depth-unit aware.** Verified by back-solving IP's own printed report (`Pc = (3128.8 − TVDSS) × 1.29659 psi`) to **0.08 %** in metres. The no-constant form cannot reproduce it. *Sub-item open:* back-solving implies ≈0.4333, so IP may carry more digits than it documents — read off the live UI if it matters. |
| **D-06** | Pore-size array starts 0.01 or 0.1 µm? | **RESOLVED — 0.01 µm**, by the manual's own arithmetic (80 elements ÷ 20 per decade = 4 decades; 0.01→100 µm *is* 4 decades). |
| **D-07** | Clay-bound-water factor has unbalanced brackets: `F = 1 - [0.6425 * ( Salinity ^ (-0.5) + 0.22 ] * Qv ]` | **OPEN.** Three `]`, one `[`, one unmatched `(`. No second statement, no worked example — no internal evidence to adjudicate. **Do not repair from a remembered Hill-Shirley-Klein form.** Unit traps the manual *does* state: Salinity in **Kppm**, Qv in **meq/ml**. |

### Cross-check still owed
**Four-way mineral endpoint comparison** — IP2018 `MINDEF.PAR` vs IP2025 `MINDEF.PAR` vs Techlog
`QM_MineralTable` vs SandiMin. **Must be a file diff** (§1-D: the IP2018 help prints no endpoint
table). Treat divergence as independent library vintages until proven otherwise. Known open
item carried from the IP2025 ingest: **SandiMin smectite density** (dry-grain 2.63 vs wet-clay
2.02/2.12).

---

## 4. Completeness & what was skipped

**Covered: 44 of 278 pages** — the petrophysically load-bearing set, carrying 306 of the 926
formula images.

**Not covered: 234 pages.** The honest breakdown:

| | Pages | Formula images |
|---|---|---|
| Untouched, **zero** formulas (plot styling, DB links, UI chrome, loaders) | 157 | 0 |
| Untouched, **≥1** formula | **77** | **620** |

Of those 77, the tail is mostly UI/IO pages with 1–2 likely-misclassified glyphs. **The genuine
method-page gaps, ranked:**

| Formulas | Page | Note |
|---|---|---|
| 84 | `easteuroperescorrections.htm` | largest single uncovered block; East-European resistivity corrections |
| 56 | `fluidsubstitution.htm` | Gassmann chain |
| 51 | `ucr.htm` | |
| 41 | `rock_strength.htm` | geomechanics |
| 38 | `laminatedfluidsubs.htm` | laminated fluid substitution — **relevant to Jauhar's thin-bed work** |
| 35 | `nmrinterpretation.htm` | |
| 22 | `production_log_analysis_module.htm` | |
| 20 | `geosteering.htm` | |
| 18 | `porepressurecalculations2.htm` | |
| 15 | `plotting_image_analysis_data.htm` | pairs with target G |
| 14 | `cementeval.htm`, `ft_equations_and_methodology.htm` | formation testing |
| 11 | `sigma.htm` | |

**Deliberately not done, and these are decisions rather than omissions:**
- **No OCR of rasterized equations** (§0.2). The reason is the whole point of the discipline.
- **No `.t83` decoding** (94 binaries) — excluded by instruction; nothing inferred from them.
- **No decompilation** of any IP binary.
- **No vendor data copied into this repo** — tool geometry, chart tables, `.PAR` files and
  TestData all stayed in the read-only install.

---

## 5. The extractor — reusable, and archived here

`hh.exe -decompile` **failed silently** — exit code 0, zero files, no error message, twice
(including with a short output path via `Start-Process -Wait`). Diagnosed by confirming the CHM
header was a valid `ITSF` v3, then routed around entirely.

`ChmExtract.cs` drives the Windows **InfoTech Storage System** (`itss.dll`) COM provider
directly: Windows performs the LZX decompression, the shim walks the `IStorage` tree and dumps
every stream. `IStorage` is declared with placeholder methods (`_CreateStream`, `_CreateStorage`,
`_CopyTo`, `_MoveElementTo`, `_Commit`, `_Revert`) purely to preserve vtable slot ordering — only
`OpenStream`, `OpenStorage` and `EnumElements` are real.

```
csc /nologo /platform:x64 /out:ChmExtract.exe ChmExtract.cs
ChmExtract.exe "C:\Program Files\IP2018\Interact.chm" %TEMP%\c18
→ files=5298 bytes=244555815 errors=0
```

**Generic — works on any CHM**, not just this one. Archived as `ChmExtract.cs`.

Post-processing produced `_text\*.txt` (278 clean-text pages, 3.93 MB) with
`<img src="embim…">` replaced by `[[EQUATION_IMAGE: embimNN.gif]]`, so equation *loss* is visible
in the text rather than silent. Every agent read those files.

---

## 6. Feature shortlist for SandiBumi (ranked by value-per-effort)

1. **Correction-state boolean family** (§1-G) — make "already corrected?" an explicit data
   property. Cheap, and kills a whole class of silent error.
2. **Print-Parameters-to-File audit trail** (§1-D) — every model, parameter and mixing dumped to
   a named `.txt`. Aligns exactly with SandiBumi's existing audit posture.
3. **479-parameter module reference** (`H_…json`) — a ready-made parameter vocabulary to diff
   SandiBumi's own module surface against.
4. **Priority-ordered `;`-separated curve alias lists** (§1-G) — a better alias mechanism than a
   flat map, and a convention rather than IP.
5. **Monte Carlo error propagation across the whole module chain**, with the Sw percentile flip
   and the percentile-independence caveat implemented *and documented* (§1-C).
6. **124-citation Tier-B table** (`F_citations.json`) — the reimplementation reading list.
7. **Tool-class taxonomy + unit-declaration headers** (§1-G).

---

## 7. Output file manifest

| File | Contents |
|---|---|
| `A_porosity_sw.md` / `.json` | 34 methods, 126 params, 87 verbatim defaults, 13 gaps |
| `B_clay_volume.md` / `.json` | 25 methods, 53 defaults, 39 lithology codes, raster verdict |
| `C_cutoffs_defaults_mc.md` / `.json` | 101 defaults, 10 cut-off defs, 28 summation outputs, 31 validation rules |
| `D_mineral_solver_ssc.md` / `.json` | 20 minerals, 30 response-equation types, 31 SSC params, market intel |
| `E_shf_rocktyping.md` / `.json` | 33 SHF functions, 24 caveats, 12 gaps |
| `F_envcorr_tierc_citations.md` | Environmental corrections narrative |
| `F_citations.json` | **124** citations as printed, each with what it supports |
| `F_tierC_register.json` | **32** Tier-C items, existence + reasoning only |
| `F_manual_map.md` | 278-page manual map |
| `G_legacy_tool_definitions.md` / `.json` | **224** tools, 161 defaults, 29 mnemonic conventions, per-item reuse verdicts |
| `H_module_parameter_reference.json` | **479** params from `ClayVol`/`PhiSw`/`Cutoff` `.hlp` (scripted) |
| `I_page_recoverability.json` | Per-page formula/screenshot counts, all 278 pages (scripted) |
| `DISCREPANCIES.md` | 7 contradictions: 5 resolved, 2 open, with reasoning |
| `ChmExtract.cs` | Reusable generic CHM extractor |
| `FINDINGS.md` | This file |

Sibling: `../ip_ingest/EE_capability_dossier.md` (Experienced Eye competitive dossier, from
public vendor webinars).

---

*Ingested 2026-08-06. Source `C:\Program Files\IP2018` — read-only, unmodified.*
