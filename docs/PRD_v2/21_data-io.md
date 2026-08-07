# 21. Data import, export and formats — requirements

**Domain code:** `DIO` · requirements `SB-DIO-nnn` · tests `SB-DIO-Tnn`
**Dossier:** `docs/research_2026-08/cross_tool/data-io.md` — 1,865 lines, read in full 2026-08-07.
**Evidence tiers held:** T1 (Geolog `bin/*.tclsh`, `specs/*.units`, `specs/*.flat_ascii_format`,
`specs/alias.alias`, diagnostic-string extraction from six `tp_*` binaries) · T2 (IP 2025 and
IP 2018 full-manual CHM ingests) · T3 (IP 2025 install tree and per-user runtime config;
Techlog 2018.2 install tree) · T4 (petro-kb notes, used **only** to size worked stakes, never as
a SandiBumi default) · **P** (Jauhar's own delivered format work — the `mudlog-data-to-las` and
`sdc-odf-extract` skills, the wellsite-`.xls` BIFF recovery, the SDC `.odf` decode).
**Author date:** 2026-08-07.
**Requirements:** 63 · **P0:** 10 (`SB-DIO-004`, `-015`, `-016`, `-017`, `-023`, `-031`, `-051`,
`-054`, `-055`, `-061`).
**Parameters:** 86 rows across six tables · **Acceptance tests:** 96 (11 characterization,
12 malformed-input) · **Dossier items dispositioned:** 202 of 202.
**Independent derivations:** 1 (class C-3, `SB-DIO-038`) · **Acquisition gaps:** 4 (§7.4 D-2).

> **Read §7.2 first if you are checking this chapter against `04_CORE_REQUIREMENTS.md`.**
> Three of the four as-built facts this chapter was commissioned on were **stale at the source**.
> `units.rs` (301 lines, with tests) shipped between the audit and this chapter, and it closes
> most of `SB-CORE-001`'s parse-and-carry half. What is left open is narrower, sharper and
> **still P0** — and two of the holes it leaves (the DLIS reader and the LAS writer) were not
> in the audit at all. That correction is escalation **E-1** and it is the single most important
> thing in this document.

---

## 1. Scope and boundary

This chapter owns **the boundary between a file on disk and a number in the project**, in both
directions, and every convention that boundary has to get right for the number to survive the
crossing intact.

**Owned — read side.** Format recognition and refusal. LAS 2.0/3.0 parsing (sections, wrap,
declared `NULL.`, delimiters, malformed-token policy). DLIS ingest through the `dlisio`
subprocess, including per-channel null policy and the dimensionality contract. Delimited-text
ingest for tops, core, petrography, XRD, perforations, deviation surveys, SCAL Pc/Sw and well
locations, and the universal **Intake** importer that is replacing the per-table dialogs
(LONG / WIDE / BLOCK). `.xlsx` plate-workbook extraction and the `.xls` refusal. Index-column
(depth) resolution and the positional guard. Mnemonic alias resolution — including the
**coverage-aware** tie-break that is SandiBumi's own invention and has no counterpart in the
three incumbents. Unit-label canonicalisation and family tagging at the curve-store boundary.
Depth-unit parse, reconciliation, carry and enforcement. Duplicate-depth and non-monotonic-index
policy. The ingest record: what was skipped, renamed, converted, dropped or assumed, and whether
the user was told.

**Owned — write side.** LAS export (header conventions, the declared null, the declared index
unit, wrap, precision, which curves are in scope). The provenance that must travel *into* a
delivered artefact. The `xlsx` / `pptx` / `docx` writers as **formats** — their null convention,
their blank-versus-zero rule, their number-versus-string rule, and the Python prerequisite where
it gates the format. Export-time resampling and export-time selection scope.

**Owned — the parameters themselves.** In this domain the §5 table is not a list of
petrophysical endpoints; it is a list of **format constants, null conventions, tolerances,
alias-resolution thresholds, unit factors and magic-byte signatures**. `CONTRACT.md` §2 binds
them exactly as hard. A wrong `1e-5` null tolerance, a wrong `MEQ/L` factor and a wrong `Δt_ma`
fail identically: they compute, they plot, and they ship.

### 1.1 Named seams

**`20_envcorr-qc.md` (`ENV`) — conditioning and QC once the data is in.** The line is *storage*.
This chapter is responsible for the data landing in the project as a faithful, fully-reported
representation of what the file said; `ENV` is responsible for everything done to it afterwards.
Concretely: this chapter owns screening a `-999.25` sentinel to missing, because that is
decoding the file's own declared convention. `ENV` owns deciding that a PEF of 0.4 b/e is
non-physical, because that is a judgement about a measurement. The overlap that must not be
double-owned is IP's *Clean Data* module (T2 `F_qc_edit_corrections.md` L363), which
canonicalises `-999.25 → -999` with the rule shipped enabled — a **QC** module performing a
**null-convention** mutation. SandiBumi splits it: the recognition set is `DIO`'s
(`SB-DIO-004`), any *rewriting* of a value is `ENV`'s. Non-monotonic and duplicate index
detection is `DIO`'s because it is a structural property of the file; depth *tie-in* and
shifting between runs is `ENV`'s.

**`22_database-model.md` (`DBM`) — persistence, schema and write discipline.** This chapter
stops at the call to `db::upsert_curve_meta` / `db::insert_curve_samples`. The `(curve_id, depth)`
primary key, the append-only write discipline, the single-writer `Mutex<Connection>`, the
`documents` row that carries the project's unit setting, and the transaction boundaries are all
`DBM`'s. Where they interact is that a PK constraint **forces** an import-side policy —
`parsers.rs:453` sanitises duplicate depths precisely so a `(curve_id, depth)` collision cannot
abort a whole file. That policy is stated here as a requirement (`SB-DIO-020`); the constraint
that motivates it belongs to `DBM`. A schema change that relaxed the PK would not retire this
requirement, because dropping-with-a-count is the right answer regardless.

**`23_plotting-interactivity.md` (`PLT`) — display, and the third owner of `SB-CORE-001`.**
The renderer's hardcoded metres constant (`LogCanvasRenderer.PX_PER_UNIT_1_1`, named in
`units.rs:23-25`) is `PLT`'s half. This chapter owns the unit being **present and correct** at
the store boundary; `PLT` owns it being **used** when a print scale is computed. The composite
PDF and SVG writers are `PLT`'s renderers; where they are *export formats* carrying provenance
into a deliverable, this chapter states the obligation (`SB-DIO-051`) and `PLT` discharges it.

**`27_ip-install-blockers.md` (`INS`) — packaging and the Python prerequisite.** `INS` owns
"can this be installed on a managed estate at all". This chapter owns the Python prerequisite
**only where it gates a format**: DLIS import needs `dlisio` (`dlis.rs:133`), `.xlsx`
plate extraction needs `openpyxl` (`images.rs:856`), and the `xlsx`/`pptx`/`docx` writers need
`xlsxwriter` / `python-pptx` / `python-docx` (`office.rs:8-10`). The format-level obligation
here is that a missing interpreter must produce a *named, actionable refusal for that format*
and must never degrade the format silently — which is a `DIO` requirement (`SB-DIO-046`). The
question of whether requiring Python at all is commercially acceptable is `INS`'s.

**`15_sat-height-rocktyping.md` (`SHR`) — the arithmetic half of `SB-CORE-001`.** R14 (Pc 3.28×
wrong on a foot project) is `SHR`'s consequence. This chapter owns that the unit is **parsed,
carried, canonicalised, exposed and enforced**, and that an undeclared unit is never silently
defaulted. §3.2 records that the carry is now largely built and the *enforcement* is not.

**`24_ml-advanced.md` (`MLA`) — `SB-CORE-010` into the deliverable.** Verified at source
2026-08-07 and re-verified for this chapter: `report.rs` and `export.rs` contain **zero**
occurrences of `ml`, `facies`, `cluster`, `hfu` or `leaderboard`. `MLA` owns what a learned
model must record (`SB-CORE-014`); this chapter owns the fact that **the export path is where
that record is either carried or lost**, and today every export path drops it (`SB-DIO-051`).

### 1.2 Explicitly not owned

The **mnemonic super-dictionary** built for SegaraBumi (7,165 rows / 2,751 standards, memory
`project_segarabumi_p1_dictionary`) is a different product's artefact. It is cited here as
evidence of scale and as a candidate future source for `SB-DIO-030`'s alias table, but nothing
in this chapter requires SandiBumi to consume it.

**Vendor project-file formats are named and not touched.** `.itt`, `.itp`, `.att`, `.bor` and
`.eli` hold vendor project state — parameter sets, template definitions, borehole-image
processing state. SandiBumi reads none of them, and `CONTRACT.md` §2.2 forbids reverse-engineering
them. Interchange happens through the open formats both sides already write: LAS, DLIS, Geolog
ASCII, CSV. That is a **refusal** (§7.3 R-1), not a gap.

---

## 2. What the incumbents do — the requirement-bearing findings

Only findings that generate an obligation appear here. Everything else in the dossier is
accounted for in §8.

### 2.1 Null conventions — three tools, three architectures, one interop tax

**Finding D-1 (T2 + T3). IP's own writers disagree with each other about the null value.**
ASCII, LAS and DBASE4 export `-999`; LIS and DLIS export a hard-coded `-999.25` with no field in
the panel to change it — **five curve-data writers carrying two values**, plus the Petrel-tops
writer at `-999` (dossier §2.1, §3.1; corroborated on disk by `LasNullValue = -999` in
`Intpetro.config`, T3). Byte-stable 2018 → 2025.

**Quantified consequence.** A consumer screening only the CWLS-conventional `-999.25` reads an
IP-written `-999` as a measurement. On RHOB that is −999 g/cc, on GR −999 gAPI — non-physical, so
a range gate catches it. The cases a range gate does **not** catch are the ones that matter, and
they are ordinary: a GRN normalised to P3/P97 (Jauhar's own house workflow, memory
`method_workflow_standards`), a residual curve, or a resistivity on a log axis where a negative
is simply not plotted. There the −999 is **excluded from the picture and included in every
average**. On a 40-sample zone where one sample is a surviving −999 and the rest average 60 gAPI,
the reported mean is `(39×60 + (−999))/40 = 33.5 gAPI` — a 44 % error in a number nobody looks
at twice.

**Finding D-2 (T1). Geolog proves the single-sentinel discipline is achievable — and proves
where it fails.** `_missing_value = -999.25 DOUBLE` is set once at `log_export.tclsh:19` and
threaded to **eight of twelve** writers (`dlis`, `lis`, `las2`, `las3`, `csv`, `prn`, `tab`,
`zmap`, at lines 64–74). Four receive no `missing_value` argument at all: `amocoa`, `segy`, `rms`
and **`unl` — which is the shipped default `contractor_format`** (T1 line 16). So a Geolog user
exporting at defaults does not get the sentinel they set. The obligation is not "have a single
sentinel"; it is "thread it to **every** writer", which is a testable statement and a stronger
one. → `SB-DIO-001`, `SB-DIO-002`.

**Finding D-3 (T3). Techlog is the only tool that models "this channel has no null" as a
first-class state, and the census says it is the majority case.**
`Settings/DLIS/DlisNullValuesExceptions.xml` holds **16 `<Channel>` elements carrying 21 `<Name>`
regex patterns**. **16 of the 21 carry `<NullValues/>` — empty, meaning no null value at all** —
and every one of the sixteen is an array or waveform channel (DSI `PWF[1-5]`, Sonic Scanner
`WFA(?:[1-9]|1[0-2])`, ThruBit `WF[1-6]`, the Weatherford CXD block). The five populated patterns
carry three different values: 3× `-999.25`, 1× `-999` (SonicVision `WF[1-5][ITR]`), 1× `-32767`
(Baker waveforms). Techlog's own manual states the premise outright — *"Techlog and DLIS have
different values considered as null"* — and names the motivating case, acoustics data at −32767.

**Quantified consequence.** A single global `-999.25` screen applied to that channel set is
**wrong on 18 of 21 patterns**: it misses `-999` and `-32767` outright (2 of the 5 populated),
and on the 16 unpopulated ones it *invents* a null the vendor explicitly says does not exist —
punching holes in real waveform amplitude data at exactly the sample where the amplitude happens
to be −999.25. → `SB-DIO-003`, `SB-DIO-005`.

**Finding D-4 (T3, shipped file read verbatim — dossier T-D-1). Techlog's own table is
malformed in a way that penalises a naive parser.** Fifteen of the sixteen `<Channel>` elements
hold one `<Name>` and one `<NullValues>`. The sixteenth — Weatherford CXD — packs **six
`<Name>`/`<NullValues/>` pairs inside one `<Channel>`**. A strict one-name-per-channel object
binding keeps the first or the last and **silently drops five vendor patterns**. The rule shape
must be `{names: [regex], nulls: [f64] | NoNull}` regardless of how Techlog's own loader resolves
it — that is correct under either answer, so it is not blocked on the open question (§7.1 O-6).
→ `SB-DIO-006`.

**Finding D-5 (T1/T2/T3). RP66 defines no absent value.** This is why Geolog must inject one
(`missing_value` reaches `tp_from_dlis_slb` and nothing else — `log_import.tclsh:57`), why
Techlog needs a per-channel exception table, and why IP's DLIS loader documents no null concept
on the read side at all. SandiBumi's `dlis.rs:179` comment already states the premise correctly;
the code beneath it does not act on it (§3.4).

### 2.2 Index detection — the mistake that puts a marker 161 m in the wrong sand

**Finding D-6 (T3 + T1, dossier ledger N-6.11). IP ships three disagreeing depth-recognition
lists inside one product.** The LAS 3.0 tag list is `DEPT, DEPTH, MD, DateTime, Time, TDEP, DPTH`
(`Intpetro.config`); the Geolog-ASCII list is `DEPTH, TDEP, DEPT, MD, INDEX, TVD`
(`GeologASCII_options.txt`); and `DefaultAlias.cax` defines CurveType 1 = `Depth` with **zero
alias rows** — a third, empty list. `DPTH` is recognised by one and not the other; `INDEX` and
`TVD` by the other and not the one.

**Finding D-7 (T1 vs T3). IP's Geolog list is simultaneously too permissive and
under-inclusive.** Real Geolog's `alias.alias:14` declares `DEPTH = SCD IDWD DVP1 PDEP_XPT DEPM
TDEP`, under the file's own section comment `# aliases for references` (line 13). `TVD` sits at
line 891 under `# aliases for welltie`, as `TVD = TVD_SS TVD_KB` — **a different declared
namespace**. So IP accepts `TVD` as an MD index (wrong, per the vendor's own structure) while
missing **five of the six real Geolog depth aliases** (`SCD`, `IDWD`, `DVP1`, `PDEP_XPT`,
`DEPM`).

**Quantified consequence, arithmetic exposed.** Mistaking a TVD column for MD puts every sample
shallow by `(1 − cos θ̄)` of the measured depth traversed, θ̄ being the inclination weighted over
that interval. On a 3,000 m MD well:

| Well path | Deficit | Shallow by |
|---|---|---|
| Entire hole at 30° | `1 − cos 30° = 1 − 0.8660 = 13.40 %` | **402 m** |
| Bottom half at 30° | `0.5 × 13.40 % = 6.70 %` | **201 m** |
| 40 % tangent at 30° — a typical deltaic-clastic development well | `0.40 × 13.40 % = 5.36 %` | **161 m** |
| Bottom third at 30° | `0.333 × 13.40 % = 4.47 %` | **134 m** |

Markers are strictly worse than curves here, because a curve carries a shape a reader might
recognise as displaced and **a marker carries nothing at all**. → `SB-DIO-011`, `SB-DIO-012`,
`SB-DIO-014`.

**Finding D-8 (T1). Geolog demonstrates the right architecture in one path and the fallback in
another, and both are needed.** Its flat-ASCII format specs declare `CLASSES = REFERENCE | LOG`
per column — a **structural** index declaration, the only one among the three tools. Its LAS
module falls back to a literal name default, `_ref_in = DEPTH` at `log_load_dxs_las.tclsh:97`.
The design lesson is per-path, not per-tool: **prefer a structural declaration where the format
offers one, fall back to names only where it does not, and record which mechanism fired.**
Techlog's ASCII wizard is the third form — a **mandatory user designation**, coloured orange,
with the constraint that the reference must be strictly increasing (T3
`import-asciidata.html`). → `SB-DIO-010`, `SB-DIO-013`.

### 2.3 Unit handling — the largest structural divergence, and where 1000× errors live

**Finding D-9 (T3, closes dossier N §9 OPEN-1). IP's entire numeric unit-conversion capability
is a 63-line file covering four quantity families** — SONIC, DENSITY, CALIPER, POROSITY. Nothing
else converts: resistivity, GR, temperature, salinity, permeability, pressure and CEC/Qv all pass
through unconverted. The file's own header comment claims **three** families, omitting POROSITY —
a defect in the shipped file's documentation of itself. The one artefact that might have proved
otherwise, `OpenSpiritUnits.opt` (5,662 lines), was parsed: 1,132 records with exactly
`<Acronym>`, `<Name>`, `<Description>` and **zero** conversion factors. It is a vocabulary, not a
table.

**Quantified consequence — IP under-converts silently.** A resistivity delivered in `KOHMS` is
stored as if it were ohm·m: a curve reading `2.0` means 2,000 ohm·m and is kept as 2.0, so Rt is
**1000× too small**. In Archie `Sw = (a·Rw / (φ^m·Rt))^(1/n)`, `Sw ∝ Rt^(−1/n)`, so Sw is
multiplied by `1000^(1/2) = 31.6×` at `n = 2.0` — **driven up, and in practice pinned at or above
1.0: a real pay zone reported as wet.** (`n = 2.0` is IP's own stated default, T2
`basicloganalysis.htm` — cited to size the stake, **not adopted as a SandiBumi default**.) The
mirror case is equally live: Geolog *does* carry a `KOHMS = 1000` row, so a genuine ohm·m curve
mislabelled `KOHMS` and converted there makes Rt 1000× too large and Sw `1000^(−1/2) = 0.0316×` —
pinned near zero, reading as spectacular pay. **Both directions are one-thousand-fold and neither
announces itself.**

**Finding D-10 (T1). Geolog over-converts silently, by default.** `PG_UNIT_CONVERT=YES` is the
shipped default (`log_import.tclsh:20`) and conversion targets the category `PREFERRED`.
`time_length` PREFERRED is **µs/m**, so a DT in µs/ft is multiplied by 3.280839895013 on import.
`density` PREFERRED is **kg/m³**, so RHOB 2.65 g/cc becomes 2650. Applied wholesale the result is
absurd and fails loudly — Wyllie with Δt = 328.08 (µs/m) against Δt_ma = 55.5 and Δt_f = 190
(µs/ft) gives `φ ≈ 2.03 v/v`. **The dangerous version is the partial one**, where some inputs were
converted and some were not. Geolog's default unit *system* is `imperial`
(`geolog_env.tcl:197`), overridable by site config — so the effective target depends on the
installation, not on the file.

*(Δt_ma 50–55.5 µs/ft, Δt_f 190 µs/ft and ρma 2.65–2.70 g/cc are cited from petro-kb
`notes/2-1-porosity.md` §"Key equations & parameters", **T4, illustrative only**, used to make a
unit defect feel like a number. They are not SandiBumi defaults and do not appear in §5.)*

→ The obligation both findings generate is the same and it is a **deliberate divergence from
both vendors**: canonicalise the **label**, do not rescale the **value**, on ingest.
`SB-DIO-024`, `SB-DIO-025`.

**Finding D-11 (T1, dossier G-D-1 — a defect in a shipped vendor table).** Geolog's
`specs/elec_charge_per_vol.units` declares the category base as meq/cm³ and gives
**`MEQ/L` FACTOR `1.0`**. Every other row in the file checks out by unit arithmetic
(`MEQ/CM3` 1.0 ✔, `EQ/L` 1.0 ✔, `EQ/M3` 1.0E-3 ✔, `C/M3` 1.0364090499238E-08 ✔). `EQ/L` and
`MEQ/L` cannot both be 1.0 — they differ by definition by 1000. The `RP66:` column corroborates
the mechanism: `MEQ/L`'s RP66 symbol is `"96487 C/L"`, **identical to `EQ/L`'s**, i.e. the row
was cloned and only the display name edited. The file's `SI = C/M3` line proves the table does
know the category spans eight orders of magnitude.

**Quantified consequence.** House convention for Qv is meq/mL = meq/cm³ (memory
`reference_waxman_smits_b`). A core Qv delivered as `0.30 meq/L` and imported with
`PG_UNIT_CONVERT=YES` becomes **0.30 meq/cm³ instead of 0.0003**. In Waxman-Smits
`1/Rt = φ^m*·Sw^n*·(1/Rw + B·Qv/Sw)`, evaluating `B` from the Juhász (1981) closed form the
dossier cites and verifies against two published points:

| T | B (S/m)/(meq/cm³) | `B·Qv` at wrong Qv = 0.30 | share of total σ | at true Qv = 0.0003 | share |
|---|---|---|---|---|---|
| 25 °C | 3.43 | 1.028 S/m | **23.6 %** | 0.00103 | 0.031 % |
| 60 °C | 6.93 | 2.079 S/m | **38.4 %** | 0.00208 | 0.062 % |
| 80 °C | 8.04 | 2.411 S/m | **42.0 %** | 0.00241 | 0.072 % |
| 100 °C | 8.75 | 2.624 S/m | **44.0 %** | 0.00262 | 0.079 % |

(at Rw = 0.30 ohm·m, `1/Rw = 3.333 S/m`, Sw = 1). The unit error injects **a quarter to nearly
half of total formation conductivity as fictitious counterion conduction**, against a true
contribution under a tenth of one percent. Because the excess-conductivity term is exactly what
the model uses to explain away low resistivity, that surplus is absorbed as clay-bound water and
**Sw is over-corrected by tens of saturation units** — with the same signature as a genuine
low-resistivity-pay zone, which is why it survives visual QC. **Techlog is safe by refusal** (no
`meq/L` alias exists, so the string is unrecognised); **IP is safe by absence**. Only Geolog
mis-converts. → `SB-DIO-028`, and the corrected factor is a §5 row with its derivation shown.

**Finding D-12 (T3 + T1). `MS/FT` is a genuine, unadjudicable two-way ambiguity.** IP maps
`MS/FT`/`MSFT` to µs/ft at factor 1.0, encoding the legacy vendor usage where "MS" meant
microsecond. Geolog's `time_length.units` has **no `MS/*` entry at all**, so the same string is
unrecognised. Both readings are defensible — IP's for legacy Halliburton/Dresser-vintage LAS,
Geolog's if `MS` is read as the SI millisecond — and they are **1000× apart**. This is the
canonical case for `CONTRACT.md` §2's absent-rather-than-adjudicated rule. → `SB-DIO-029`.

**Finding D-13 (T1, two shipped-file defects that dictate schema).** `volume_ratio.units`
declares the key `M3/M3` **twice**, at lines 7 and 37, both = 1.0. Harmless because they agree —
and it proves the file format has no uniqueness constraint, so a *disagreeing* duplicate would
ship just as silently under either a first-wins or a last-wins loader. Separately,
`temperature.units` stores the degree sign in **octal-escaped ASCII** — `\260C`, `\260F`, `\260K`
(`\260` = 0xB0 = `°` in Latin-1). A unit token arriving as a literal `°` byte (Latin-1 `0xB0`, or
UTF-8 `0xC2 0xB0`) **will not match**, and the temperature passes through unconverted — a silent
32-degree offset in any Rw(T) or B(T) computation downstream. → `SB-DIO-026`, `SB-DIO-027`.

**Finding D-14 (T1, `density.units`, full alias census). `PPG` is a shipped vendor alias and a
routine curve.** Mud weight in pounds per US gallon converts at 119.826427316897 to kg/m³. An
implementation that omits it sends an ordinary mud-weight curve down the unknown-unit failure
branch. In the same file, **`PSI/FT` (RP66 `lbm/(in2.ft)`) is a pressure gradient carried inside
the density category** — dimensionally distinct from a bulk density and not interchangeable with
one in any formula. It must resolve and then be **flagged**, never silently treated as ρb.
→ `SB-DIO-027`.

### 2.4 Alias resolution — the one vendor behaviour that corrupts data rather than labels

**Finding D-15 (T2, dossier §3.5 Hazard 2). IP substitutes a different curve's data under the
requested name, on export, silently.** Verbatim from `datasaving.htm`: *"**Curve aliasing on
export:** If the Curve Aliasing module is turned on then if a curve name cannot be found then **a
curve of the same Curve Type will be selected instead**."*

Compare the two failure modes, because they are not the same defect:

| | `CurveAlias.txt` rename on import | Curve-type substitution on export |
|---|---|---|
| What is wrong in the output | the **label** | the **data** |
| A `GR` request yields | the right curve under a different name | **a different curve** under the requested name |
| Detectable by | comparing mnemonics against the source file | **nothing in the file** — the header says `GR` and the samples are some other curve of the same declared type |
| Recoverable | yes, rename back | **no** — which curve was substituted is written nowhere |

`DefaultAlias.cax`'s three most populated types are `MedRes` (32 aliases), `DeepRes` (31) and
`ShalRes` (30), so the substitution pool for a missing deep resistivity is **up to 31 other
curves**. This is the single most dangerous behaviour in the domain and it has no counterpart in
Geolog or Techlog: `tp_name_translate` fails a name it does not know rather than going looking
for a substitute. → `SB-DIO-031` (a MUST NOT), `SB-DIO-032`.

**Finding D-16 (T2 + T3). IP applies `CurveAlias.txt` automatically and silently on batch
import** — *"NOTE: this is automatically applied, you do not have to manually select the Curve
Alias file"*. The shipped file is **empty** (5 comment lines, 275 bytes), so the hazard is latent
rather than live; it activates the moment a user populates it, and nothing in the batch UI
reports that a rename occurred. → `SB-DIO-030`.

**Finding D-17 (T2). IP's *Select Final Curves All Wells* pulls curves out of wells the user
never selected** — *"if any of the other wells has the same curve in the same set, it too will be
output, **even if it is not marked as Final in that well**"*. A deliverable produced by this route
mixes final and working curves with nothing in the file distinguishing them. → `SB-DIO-052`.

**Finding D-18 (T1). Geolog's distinguishing property is explicitness, not ubiquity.**
`tp_name_translate` is **branch-conditional** in both directions — absent from the import
`file_scan` branch entirely (`log_import.tclsh:100-126`), and absent from the export
`.well_query` branch where renaming folds into `log_dbms … giving=$giving`
(`log_export.tclsh:104-159`; a whole-file grep returns the stage at exactly lines 89 and 99).
What holds in **every** branch is that the rename is driven by a **named spec passed as an
argument**, so the translation applied is recoverable from the command that ran. Nothing is
applied because a file happened to sit in the install directory. That is the property to adopt.
→ `SB-DIO-030`.

**Finding D-19 (T2, mask files). The one IP artefact worth copying wholesale, with its modes and
without its built-in.** IP's `.mask` format is plain text, `$` comments, four separators (comma,
space, tab, semicolon — stated in the shipped `CurveAlias.txt`'s own header, T3-direct), regex
capable (`(DEV|DEVEI)(\.H)?  DEV`), and works identically on import and export, filtering **and**
renaming in one diffable, version-controllable artifact. It operates in four modes — *No Mask* /
*Selected* / *IP Defaults* / *Load Mask* — and a mask artifact without a stated mode is ambiguous
about whether an unlisted curve is loaded or skipped. **Two things must not be copied**: the
non-user-editable built-in Image Tool Mask (a selection rule the user cannot read, diff or
version is the opposite of what makes the format worth having), and *IP Defaults* mode, which
auto-selects by Curve Type — the read-side twin of D-15. → `SB-DIO-033`, `SB-DIO-034`.

### 2.5 Duplicate depths, resampling and the writer-side re-grid

**Finding D-20 (T2). IP resamples on load in at least nine documented paths, mostly silently** —
ASCII Load snaps to "closest sample increment"; Kingdom "curves are resampled to fit the existing
IP step"; WITSML into a regular set is documented as able to let *"different values over-writing
each other"*; Petrel export→IP has *Interpolate Values* **checked by default**. No diagnostic in
any of them.

**Finding D-21 (T3). Techlog is clearly ahead here and its vocabulary is the one to adopt.** Its
Array creation policy is five named, user-chosen options: *Create an array if necessary* / *Add
an epsilon to the reference* / *Ignore current file line* / *Duplicate the reference value* /
*Ask for each case*. Repeated-reference data (core, plug) is a designed case, not an accident.
SandiBumi's current *drop-and-count* is the correct **default** and is 1 of those 5.
→ `SB-DIO-020`, `SB-DIO-021`.

**Finding D-22 (T2, and the half every load-side analysis misses). IP re-grids on *write*.** The
shared export model gives every writer a Top/Bot/**Step** triple: *"Step defaults to the current
well step and is editable — **changing it resamples the output**"*. That is a resample applied at
the moment of delivery, downstream of every QC decision, leaving no record in the file. A
resample the user cannot see in the artefact they shipped is strictly worse than one they can see
on import, because on export there is nothing left to compare against. → `SB-DIO-050`.

**Finding D-23 (T2, ledger N-6.5 disposition). Name the mechanism correctly.**
`batch_ascii_loader.htm` is the corpus's only mechanical statement — *"extrapolated over using a
linear interpolation between the good data"* — while every other IP page calls interior
interpolation "extrapolation". Adopt linear interpolation as the mechanism and the word; expose
the gap width in **depth units**, not in database sample increments as IP does (which silently
changes meaning when the step changes); default it **off**. → `SB-DIO-022`.

### 2.6 Format-level facts that bind a reader or a writer

**Finding D-24 (T1 + T2 + T3). LAS wrapping is a genuine three-way disagreement, and only one
tool states a number.** Geolog defaults `LAS_WRAP = NO` (`log_export.tclsh:22`); IP ships *Write
LAS wrapped data* cleared; **Techlog defaults to `Standards wrap`, which for LAS 2.0 means an
80-character line** and for LAS 3.0 means no wrapping. Two of three write unwrapped; the third
wraps. **A SandiBumi LAS reader must therefore treat wrapped LAS 2.0 as a live case, not an edge
case.** Writer recommendation is unchanged: emit unwrapped. → `SB-DIO-040`, `SB-DIO-047`.

**Finding D-25 (T1, dossier §2.9). Geolog is the only tool whose LAS 3.0 section-parsing contract
is recoverable, and it is the reference.** `tp_from_las.exe`'s diagnostic strings give a complete
rule set: `~V` mandatory (*"Missing or invalid '~Version' section detected in file"*); `~W`
mandatory before data (*"Must have Well Information before data can be loaded."*); `WRAP: YES`
unsupported in LAS 3.0 and ignored with a warning; delimiters `SPACE`/`COMMA`/`TAB`/nothing;
associations required for DATA sections and forbidden for DEFINITION and PARAMETER sections;
a DATA section whose DEFINITION is missing or whose entry count disagrees is an error; unknown
sections ignored with a message. **By contrast the IP manual states essentially no LAS
section-parsing rules at all** (T2 §9 OPEN-6). Adopt Geolog's rule set as the contract.
→ `SB-DIO-041` … `SB-DIO-044`.

**Finding D-26 (T1 + T2). LAS 1.2 is readable but not writable in both IP and Geolog.** IP's
export panel offers only 2.0 and 3.0; Geolog's writer string table contains only
`CWLS LOG ASCII STANDARD - VERSION 2.0` / `VERSION 3.0` and the literal `'2.0, 3.0'` validation
list. Two independent tools agreeing upgrades this from inferred to established.

**Finding D-27 (per-container, T2 + T1 + T3). "All wells in one file" is not one question.**

| Container | Multi-well in one physical file? | IP | Geolog | Techlog |
|---|---|---|---|---|
| **LAS 2.0** | **No** — `~W` holds exactly one `WELL.`, so a concatenated file reads as one well with impossible depth jumps (**P**, `mudlog-data-to-las`) | offers it (default cleared); reads concatenated via *Embedded LAS Sequences* | `tp_from_las_multiple.tclsh` **loops per file**, never parses a concatenated stream | **refuses to write it** — plain single-file chooser, no concatenation option |
| **LAS 3.0** | **No** (same `~W` limit) | offers it; its own LAS3 *loader* cannot read concatenated LAS | not exposed | offers concatenated / by-dataset / by-well |
| **LIS** | **Yes, by design** — logical-file layer inside one physical file | *Keep LIS File Open* appends logical files | present, parameters unrecovered | no evidence held |
| **DLIS/RP66** | **Yes, by design** | offered | `FRAMES-PER-IFLR-LIMIT`, origin translation | *"one logical file per dataset or per well"* |
| **Geolog ASCII (`unl`)** | **Yes** — section-delimited with per-block `WELL`/`HOLE` constants | reads it | writes it (default format) | offers concatenated / per-dataset |

The honest three-way is **only about LAS 2.0**, and there it is decisive: IP offers it (unsafe),
Geolog does not parse it, Techlog refuses to write it. **Two of three agree with the house rule.**
The strongest form of the evidence is Techlog's within-vendor contrast: the same product, the
same export buffer, the same dialog, concatenating LAS 3.0 and Geolog ASCII while withholding the
option one row above for LAS 2.0 — a deliberate judgement about LAS 2.0 specifically.
→ `SB-DIO-045`, `SB-DIO-048`.

**Finding D-28 (T2, dossier N-6.12). IP at factory defaults writes a DLIS its own loader states
it cannot read.** The writer default is *Export grouped curves as 3D curves* — **checked**. The
loader states *"3-D data cannot be loaded"*, with *Load as Array* and *Average the values*
**disabled**, because *"the current and previous versions of the loader could not load 3-D
data"*. IP survives this in practice only by recognising its own files by provenance and routing
them down an image path — so the contradiction bites hardest on a third-party tool reading IP's
DLIS. **Disposition: never emit a shape our own reader rejects, and make that a build failure
rather than a warning.** → `SB-DIO-049`.

**Finding D-29 (T2). Precision: IP writes DLIS as `FSINGL` unconditionally**, so any float64
curve loses precision with no option, and its ASCII/LAS default is 4 decimal places. Geolog
carries `DECIMAL_PLACES` **per column** in its format specs. Exceeding IP here costs nothing but
care. → `SB-DIO-047`.

**Finding D-30 (T2 + T3, the metadata that survives). Header mnemonics have no shared standard
across the three, and `Country` is the canary.** IP maps Country → **`COUNT`** on both LAS and
LIS (46 populated WellAttribute pairs, T3); Techlog maps Country → **`CTRY`** (`Las.xml`); Geolog
maps `CTRY` → `COUNTRY` (`export.names`). So an IP-written LAS carrying `COUNT.` is not
recognised as country by either other tool, and vice versa. Low stakes individually — and it is
the canary for the whole header layer. Note also the **`STAT` collision**: three tools agree
`State → STAT`, while Techlog's *well status* takes the visually adjacent `WSTA`. Four characters
apart in spelling, unrelated in meaning; a fuzzy or prefix-matching header mapper will confuse
them. → `SB-DIO-053`.

**Finding D-31 (T1). Signature-based format recognition, with a collision that must be handled.**
Geolog's dump format is recognised by the first line beginning `*HEADER`, and the writer emits
exactly `*HEADER  GEOLOG LOG Geolog Dump File` (two spaces). **IP's own database files also use
`.dat` and are binary**, so the loader must reject them by signature rather than by extension.
Two further signature facts from Jauhar's delivered work (**P**): a wellsite `.xls` is very often
not Excel — `09 08 06 00` is a headerless BIFF5, `D0 CF 11 E0` is OLE2/BIFF8, `50 4B` is a real
`.xlsx`, and BIFF2 subsets must be record-decoded outright; and an SDC Geo Suite `.odf` is a ZIP
whose first two bytes are nibble-swapped (`05 B4` for `50 4B`). **Extension is not evidence of
format.** → `SB-DIO-060`.

**Finding D-32 (P, `mudlog-data-to-las`, and it is a requirement rather than a tip). Validate on
physical bounds, not on column names.** Two recorded traps from delivered wellsite work:
(a) **zeros are not readings on a log-scale gas curve** — a total-gas or chromatograph channel
plotted on a log axis carries zeros that are absence-of-record, and averaging them in drags a
C1/C2 ratio toward zero without any value leaving its declared range; (b) **vendor headers shift
a column while every label still matches** — a contractor export whose columns moved by one
position keeps a perfectly valid header row, so every name-based check passes and every number is
in the wrong curve. The only defence that catches both is a **per-family physical-plausibility
gate evaluated on the values**: a GR that never leaves 0.19–0.21 v/v is not a GR whatever the
header says. → `SB-DIO-023`, and the seam to `ENV` is that `DIO` raises the flag on import while
`ENV` owns the conditioning response.

**Finding D-33 (T2 §8.2). "Field empty" and "field = null sentinel" are different facts.** IP
injects a null between two adjacent delimiters without recording that the value was *absent*
rather than *nulled*. The distinction matters at exactly the moment someone asks whether a
laboratory reported a zero, reported nothing, or was never asked. → `SB-DIO-007`.

**Finding D-34 (T2, IP's DLIS loader defaults in full — dossier §2.10). The shipped default that
mutates an existing object.** *Extend the Well interval if curves are outside the depth range* is
**checked**, so importing a file **silently widens an existing well's declared depth range**;
*Extend the Set interval* likewise; *Load Curves with a high sample rate as array data* is
checked, so high-rate channels silently become arrays; and the duplicate-name policy defaults to
***Insert New Data into the Existing Curve*** — a silent merge, the one dangerous choice of its
three. Encrypted channels are marked with an `x` in the Load column and **not loaded**, which is
a silent partial load unless the scan is inspected. → `SB-DIO-035`, `SB-DIO-036`, `SB-DIO-037`.

---

## 3. SandiBumi as-built

Written from the source, not from a summary. Every claim below carries a `file.rs:line`
re-verified against the working tree on 2026-08-07. Where a pointer inherited from an earlier
document no longer matched, the earlier document is wrong and the correction is recorded in §7.2.

The I/O surface is **14,888 lines across eleven modules**:

| Module | Lines | Role in this domain |
|---|---:|---|
| `ingest.rs` | 2,889 | The commit path — everything that lands in the database goes through here |
| `parsers.rs` | 2,561 | LAS 2.0/3.0, delimited text, core tables, tops, deviation |
| `office.rs` | 2,231 | `.xlsx` / `.docx` / `.pptx` deliverable writers via Python sidecars |
| `intake.rs` | 2,047 | The declared-layout tabular importer (LONG / WIDE / BLOCK) |
| `images.rs` | 1,868 | Core photography, plate workbooks, the `.xls` refusal |
| `report.rs` | 1,442 | Report assembly — the deliverable end of `SB-CORE-010` |
| `python_engine.rs` | 835 | Interpreter discovery; the gate on every sidecar format |
| `units.rs` | 301 | Depth-unit type, conversion, project setting, index reconciliation |
| `dlis.rs` | 295 | DLIS/RP66 read via a `dlisio` sidecar |
| `export.rs` | 270 | The LAS writer |
| `curves.rs` | 149 | Family table, canonical units, canonical conversion |

The shape of the finding is consistent enough to state once and then evidence: **the read paths
are strong and the write paths are not.** `intake.rs`, `parsers.rs` and `units.rs` are careful,
tested, and in two places ahead of every incumbent. `export.rs` at 270 lines is the thinnest
module in the domain and carries four defects that the read paths would have refused to commit.
A product whose importer is more disciplined than its exporter loses at the last step — the
deliverable is the only artefact a client ever sees.

### 3.1 The depth unit — the `SB-CORE-001` half this chapter owns

`SB-CORE-001` names this chapter owner of the parse-and-carry half of *depth unit is a
first-class, carried property*. The verb chain it implies is **parse → carry → canonicalise →
expose → enforce**. Taken one verb at a time, against the shipped code:

| Verb | Status | Evidence |
|---|---|---|
| Parse | `PRESENT-OK` | `units.rs:81` `DepthUnit::parse`; `parsers.rs:263`, `:573` capture the index unit |
| Carry | `PRESENT-OK` | `parsers.rs:444` `LasFrame.depth_unit`; `ingest.rs:262-265` writes `wells.depth_unit` |
| Canonicalise | `PRESENT-OK` | `units.rs:98` `convert_depth`, `:122` `convert_depths`; `ingest.rs:166` |
| Expose | `PRESENT-OK` | `units.rs:149`/`:169` project setting; `src/ipc.ts:6-14`; `src/depthUnitPref.ts` |
| **Enforce** | **`PARTIAL`** | `units.rs:220` `resolve_index_unit` — an undeclared unit is **assumed, not refused** |
| **DLIS path** | **`ABSENT`** | `dlis.rs` contains **no `units::` reference at any line** |
| **LAS write** | **`PRESENT-DIVERGENT`** | `export.rs:77-79`, `:85` hardcode `.M` |

**Three of the four "verified facts" this chapter was briefed with are stale.** They are recorded
and corrected as escalation **E-1** in §7.2, because the correction changes the requirement set:
this is a `PARTIAL` to be closed at two specific holes, not an `ABSENT` to be built from nothing.
The one surviving claim — *the family table has no `DEPTH` entry* — is confirmed at
`curves.rs:21-37`, and it is **deliberate**, not an omission: depth is reconciled at the project
level by `units.rs` rather than per-curve by family, because a family-table conversion would
silently rescale an index that other tables key on. That design is right and §4 does not disturb
it.

**What actually ships.** `units.rs:34` fixes `M_PER_FT = 0.3048` — the international foot, cited
to NIST SP 811. The US survey foot (2 ppm larger) is deliberately not modelled; at 3,000 m that
is 6 mm, below any log's depth resolution, and modelling it would require a per-file declaration
no LAS file carries. `DepthUnit::parse` (`units.rs:81`) accepts `M|MT|METER|METERS|METRE|METRES`
and `F|FT|FOOT|FEET|'`, and **returns `None` for everything else** — it does not guess. That is
the right primitive. The problem is one layer up.

**The enforcement gap, exactly.** `units.rs:220`:

```rust
pub fn resolve_index_unit(declared: Option<DepthUnit>, file: Option<DepthUnit>) -> IndexUnitAction {
    match (declared, file) {
        (None, Some(f))              => IndexUnitAction::Adopted(f),
        (None, None)                 => IndexUnitAction::Assumed(DepthUnit::Metres),
        (Some(p), None)              => IndexUnitAction::Assumed(p),
        (Some(p), Some(f)) if p == f => IndexUnitAction::Matches(p),
        (Some(p), Some(f))           => IndexUnitAction::Convert { from: f, to: p },
    }
}
```

Four of the five arms are right. The two `Assumed` arms are the `SB-CORE-001` hole. `Assumed`
emits a note (`units.rs:199`) and the import proceeds. A note is not a refusal: it is a line in a
list a user scrolls past, and the cost of scrolling past it is a **3.28× depth error** — a marker
at 7,000 ft placed at 7,000 m, or the inverse. This is the one place in the domain where the
failure is silent, large, and *in the index rather than in a curve*, so every downstream product
inherits it. `SB-DIO-015` makes the no-declaration-anywhere case refuse.

The second arm is subtler than the first and matters more. `(Some(p), None)` — the project has a
unit, the file declares none — is the case where assuming looks safest and is not. A project set
to metres importing an undeclared feet file gets a note that reads as *confirmation* rather than
as a guess, because the unit named in the note is the one the user chose.

**The DLIS hole is total.** `dlis.rs` makes no `units::` call anywhere in 295 lines. RP66 channels
carry a `UNITS` attribute, `dlisio` exposes it, and SandiBumi reads the frame and never consults
it. A DLIS index in feet is committed as if it were already the project unit. This hole is *not*
in the audit that produced the dossier and is not in the dossier's own self-audit (§3.10,
S-D-1/S-D-2) — it is found by this chapter and owned by `SB-DIO-016`.

**The LAS writer hole corrupts a round trip through SandiBumi's own reader.** `export.rs` writes
`STRT.M` (`:77`), `STOP.M` (`:78`), `STEP.M` (`:79`) and `DEPT.M` (`:85`) as string literals. The
depth values written are the project's depths in the project's unit. So a project in feet exports
**feet-valued numbers labelled metres**. Re-import that file into SandiBumi: `ingest.rs:162`
parses `M`, `resolve_index_unit` returns `Convert { from: Metres, to: Feet }`, and `:166` divides
by 0.3048. The file is not merely mislabelled for a third party — **it fails to survive
SandiBumi's own round trip**, which is the strongest available statement of the defect.

The existing test `an_exported_las_reimports_with_the_same_values` (`export.rs`, test module from
`:169`) cannot catch this, because its fixture project is in metres, where the hardcoded literal
happens to be true. That is precisely the characterization-test blind spot `CONTRACT.md` §3 warns
about: the test pins the behaviour the author had in mind rather than the invariant. `SB-DIO-T01`
fixes it by running the round trip in **both** units.

### 3.2 Null handling — two tables, two tolerances, one convention

`parsers.rs:130` declares `LAS_NULL_VALUES: [f32; 2] = [-999.25, -9999.0]`. Two comparison
functions consume it and they **do not agree**:

```rust
// parsers.rs:132-134 — absolute, machine-epsilon
pub(crate) fn is_las_null(v: f32) -> bool {
    LAS_NULL_VALUES.iter().any(|null| (v - null).abs() < f32::EPSILON)
}
// parsers.rs:138-140 — relative, 1e-5, floor 1.0
```

`f32::EPSILON` is 1.19e-7, and it is the spacing of floats **near 1.0**, not near −999.25. Near
−999.25 the actual `f32` spacing is ≈6.1e-5, roughly 512× larger than the tolerance being tested.
So `is_las_null` is an exact-equality test wearing the costume of a tolerance test: it succeeds
only when the value round-trips bit-for-bit. A `-999.2500001` written by a tool with a different
decimal formatter, or a value that has been through one `f32`↔`f64` conversion, **is not
recognised as null** and enters the arithmetic as a real measurement of −999.25. This is dossier
item **S-D-2**, and it is still live at the line cited. Status `PRESENT-DIVERGENT`; owned by
`SB-DIO-004`.

The relative form at `:138-140` is the correct one and already ships — the fix is to delete the
absolute form and route its callers through the relative one, **not** to invent a third rule.
`SB-CORE-007` (one definition per constant and transform) is the binding requirement here: null
recognition is one transform and it currently has two definitions.

`parse_null_line` (`parsers.rs:143-148`) reads the file's own `NULL.` declaration, which is the
right architecture and matches §2.1's finding that a declared null must beat a built-in list. The
built-in list is the fallback for files that omit the declaration, which is where it belongs.

**The DLIS null screen is architecturally wrong, and the code says so in the line above.**
`dlis.rs:179` carries a correct premise comment — RP66 has no standard absent value — and then
`:183` applies `crate::parsers::is_las_null(*v)` **globally to every channel**, alongside
`!v.is_finite()` and `v.abs() > 1e30`. Applying a LAS-derived null list to RP66 data is exactly
the interop tax §2.1 quantified: a legitimate DLIS sample of −999.25, in any channel whose
physical range spans it, is deleted as absent. Status `PRESENT-DIVERGENT`; owned by `SB-DIO-039`.

The `1e30` clamp is a different matter and is defensible — it is a sentinel screen for values
outside any physical log range, not a null convention — but it is **undocumented and uncited**, so
§5 carries it as an as-built constant whose source is stated as the code itself. That is the
honest tier for it, and it is the tier `CONTRACT.md` §2.1 requires when no external source exists.

### 3.3 Index and depth-column resolution — three lists where there should be one

SandiBumi resolves the depth column three different ways in three different code paths:

| Constant | Line | Members |
|---|---|---|
| `DEPTH_ALIASES` | `parsers.rs:168` | `DEPT`, `DEPTH` |
| `CORE_DEPTH_ALIASES` | `parsers.rs:642` | `DEPTH`, `DEPT`, `MD` |
| `TOPS_DEPTH_ALIASES` | `parsers.rs:1607` | `DEPTH`, `MD`, `TOP_MD`, `MD_TOP`, `TOP_DEPTH`, `DEPT`, **`TVD`** |

This is dossier item **S-D-1**, still live. Two of the three divergences are **justified** and one
is a defect, and this chapter must not flatten them together — "three lists, unify them" would
destroy a correct design to fix an unrelated bug.

**`DEPTH_ALIASES` being the shortest is correct**, and the rationale at `parsers.rs:162-167` is
one of the better pieces of reasoning in the repo: the LAS index is *always the first column*, so
matching `TDEP` or `MD` by name would let an auxiliary `MD` **track** sitting in a later column
steal the depth role from the true index. Depth resolves to the first `DEPT`/`DEPTH` curve, else
column 0, and never to an all-NaN column that would trip the `(well_id, depth)` primary key. That
is the structural rule §2.2 credits Geolog with (`CLASSES = REFERENCE`), reached by a different
and equally sound route — a positional guarantee is as structural as a declared class. Status
`PRESENT-OK`.

**`CORE_DEPTH_ALIASES` admitting `MD` is correct** for its context. A core-analysis table has no
positional guarantee at all, so a name list is the only handle available, and `MD` in a core table
is unambiguously the depth.

**`TOPS_DEPTH_ALIASES` admitting `TVD` is the defect.** A tops table carrying both `MD` and `TVD`
resolves by list order, so `MD` wins — but a tops table carrying **only** `TVD` is silently read
as measured depth. In a deviated well that places a marker wrong by the whole build angle: at the
geometry §3.2 of the dossier gives, `(1 − cos θ̄)`, a 30° sustained inclination over 1,000 m of
hold is **134 m**. It commits without a note. Status `PRESENT-DIVERGENT`; owned by `SB-DIO-014`.

The fix is **not** to drop `TVD` from the list. A tops file carrying only `TVD` is a real file a
user has, and dropping the alias converts a wrong answer into an unexplained failure — which is a
different `SB-CORE-002` violation, not a fix for one. The fix is to accept it, **mark the
resulting tops as TVD-referenced**, and refuse to plot or join them against an MD-indexed log
until a deviation survey is present. That keeps the capability and removes the silence, which is
the distinction between correcting a `PRESENT-DIVERGENT` and retreating from it.

### 3.4 Curve alias resolution — coverage-aware, and ahead of all three incumbents

`resolve_curve_candidates` (`parsers.rs:413`) returns **every** column matching an alias set
rather than the first, and the `pick` closure at `parsers.rs:323-334` chooses among them. The
in-source comment at `:321-323` states the rule: the candidate with the most finite samples wins,
ties broken by alias priority, replacement only on strictly greater coverage.

This is the one place in the domain where SandiBumi is **unambiguously better than every
incumbent**, and it is worth stating why rather than merely claiming it. The failure it defeats is
the commonest real-delivery hazard in §2.4: a file carrying an empty simulated `NPHIED` placeholder
alongside a populated `NPHI_LS`. A first-match resolver — which is what all three incumbents use —
binds the standard neutron slot to the placeholder and produces an all-null porosity that looks
like a *processing* failure rather than an *import* failure, which is the expensive kind of
misdirection because it sends the user to the wrong module to debug it. The coverage rule binds it
to the populated column.

The tie-break is the part that makes it safe rather than merely clever. Ties broken by alias
priority, with replacement only on *strictly* greater coverage, makes the rule **deterministic and
stable**: two columns of equal coverage resolve to the higher-priority alias, always, so
re-importing the same file gives the same answer, and so does importing it on another machine. A
coverage rule without a deterministic tie-break would be worse than first-match, because it would
be irreproducible — and an irreproducible import is unauditable. Status `PRESENT-OK`.
`SB-DIO-008` locks the behaviour with a test rather than changing it; `SB-DIO-009` extends it,
because the choice is currently **silent** and a user has no way to learn that `NPHIED` was passed
over in favour of `NPHI_LS`.

Alias priority itself is documented in-source with physical rationale rather than lexical
convenience. `RES_ALIASES` (`parsers.rs:171`) orders `RES_DEEP, RESD, RT, RES, DRES, ILD, LLD,
AT90`; the neutron comment at `:172-173` records that thermal CNL-family names lead so they win
ties over epithermal, sidewall (`SNP`) and APS (`APLC`/`FPLC`) tools, and that APS and sidewall
deliveries previously matched nothing at all. That ordering is a **petrophysical** decision, so §5
carries it as a cited parameter rather than leaving it as a code detail — an alias order that
decides which resistivity becomes `RT` is a method choice with a number attached to it.

### 3.5 The LAS reader — sound, with two named limits

`parsers.rs` reads LAS 2.0 in two modes: `parse_las_2` (the six standard curves) and
`parse_las_2_all` (every curve, feeding the generic store). Both resolve the index the same way
(`:262`, `:531`) and both capture the index unit (`:263`, `:573`). Three behaviours are worth
recording as as-built strengths, because §4 locks them rather than changing them:

**A truncated data row is a hard error, not a pad.** `parsers.rs:311-317` fails the import when a
row has fewer fields than `~C` declared. The tempting alternative — pad the row with nulls — is
exactly the `SB-CORE-002` violation the core requirement names: a file that is structurally broken
would import clean, and the missing samples would be indistinguishable from genuinely absent ones.
Status `PRESENT-OK`.

**Wrapped LAS is handled by token buffering** (`parsers.rs:209`, `:289`) rather than by a
line-oriented reader, which is the only correct way to read `WRAP: YES` — in wrapped mode a
logical record spans an arbitrary number of physical lines and the boundary is a *count*, not a
newline. Status `PRESENT-OK`.

**The declared `NULL.` beats the built-in list** (`parsers.rs:143-148`, consumed at `:138`).
Status `PRESENT-OK`.

The two limits: **LAS 3.0 is not read** — there is no `~Parameter | ...` associated-section
handling, no `|` delimited sub-section parsing, and no multi-array support, so a LAS 3.0 file is
read as if it were 2.0 and its associated sections are lost silently. That silence is the
`SB-CORE-002` problem; `SB-DIO-041` requires the reader to detect `VERS. 3.0` and say what it is
dropping even in the release where it cannot yet read it. And **`depth_mnemonic` is still
`#[allow(dead_code)]`** (`parsers.rs:438-439`) while `depth_unit` beside it (`:440-444`) has been
deliberately un-silenced with a comment recording that the attribute is what hid the fact that
nothing consulted the field for a whole release cycle. That comment is the single most useful line
in the module and its lesson generalises: `#[allow(dead_code)]` on a *parsed* field is a claim that
the data is captured, and it suppresses the only signal that the claim is false.

### 3.6 The DLIS reader — the unit hole is the index, not the channels

Two claims that an earlier reading of this module would support are **wrong**, and are corrected
here rather than repeated:

- DLIS *does* canonicalise per-channel units. `dlis.rs:190-193` reads `meta.unit` and calls
  `curves::convert_to_canonical`, replacing the label with the canonical one on success.
- The null screen is *reasoned*, not accidental. The comment at `dlis.rs:176-181` states that RP66
  has no standard absent value, that producers nonetheless embed LAS-style sentinels, and that the
  screen therefore runs **before** unit canonicalisation so a survivor cannot be scaled into an
  unrecognisable value. That ordering is correct and non-obvious.

What is actually wrong is narrower and worse.

**The index unit never leaves the sidecar.** The Python runner builds `unit_by_name` over
`frame.channels` (`dlis.rs:61-66`) and emits a unit per curve (`:76`) — but the emit loop skips the
index (`:68-69`, `if name == index_name: continue`), so the index channel's `UNITS` attribute is
read into the dictionary and then never used. On the Rust side `dlis.rs` makes **no `units::` call
at any line**. A DLIS index in feet is therefore committed as if it were already the project's
unit, and the resulting well is wrong by 3.28× in depth while every curve unit on it is correct —
which is the worst possible combination, because the curves look right. `SB-DIO-016`.

**The null screen is unconditional and uncounted.** `dlis.rs:183` applies `is_las_null` to every
channel with no per-channel exception and no tally of how many samples it deleted. §2.10 records
that Techlog ships a `DlisNullValuesExceptions.xml` for exactly this reason: a blanket sentinel
rule is wrong for channels whose physical range spans the sentinel. The requirement is not to
remove the screen — the comment's premise is right — but to make it **overridable per channel** and
to **count and report** every deletion. `SB-DIO-039`.

**Seven silent `continue`s.** `dlis.rs:47`, `:50`, `:55`, `:57`, `:60`, `:69`, `:71-72`. Six of
them drop a *frame* (unreadable curves, no names, unreadable index, multidimensional index, zero
length) and one drops *every array channel in the file* — `if col.ndim != 1 or col.shape[0] != n:
continue  # skip array/multidim channels for now`. None of the seven produces a note. A DLIS file
whose entire payload is array data — a borehole image, an NMR T2 distribution, a waveform set —
imports as **zero curves and no error**. That is the textbook `SB-CORE-002` case: a degraded result
presented as a clean one. `SB-DIO-054` requires each skip to be counted and named; `SB-DIO-038`
takes the array capability itself (§7.4).

Status: `PRESENT-DIVERGENT`.

### 3.7 The Intake importer — the strongest module in the domain

`intake.rs` (2,047 lines) is the declared-layout tabular importer, and it is the module the rest of
the domain should be measured against. Its doc-comment (`:1-44`) states the discipline in the
repo's own voice, and the code holds to it.

**Layout is declared, never sniffed.** LONG, WIDE and BLOCK are user choices. Sniffing a layout is
the class of guess that produces a plausible wrong answer, and this module refuses it while
sniffing the things that are safe to sniff — delimiter, and per-column type over `SNIFF_ROWS = 400`
(`intake.rs:58`, `:333`).

**Unparseable cells are located, not dropped.** `preview_bad: Vec<(usize, usize)>`
(`intake.rs:115`, populated `:386-397`) carries the row and column of every cell that failed to
parse, across `PREVIEW_ROWS = 200` (`:53`, `:379`). The user sees *where*, not just *how many*. The
tests at `:1599-1600` pin the two real cases — a depth with a unit stuck on it, and a spreadsheet
`#N/A`. This is the direct answer to the recurring malformed-input defect family, and it is the
pattern §4 propagates to the other readers rather than reinventing.

**Unclaimed columns are carried, not discarded.** Everything no core role claimed lands in
`aux_data` typed per cell (`:9`, `:36`). A lab export is wider than the schema and the module
declines to decide on the user's behalf which of their columns matter.

**The `aux_data` boundary is stated and enforced.** `intake.rs:1288` records that a curve stored in
`aux_data` would be invisible to every module, plot and export — so curve-shaped data goes to the
curve store even when no role claimed it. That is a well-drawn line and `22_database-model.md` owns
the far side of it.

**Array previewing is separately budgeted.** `ARRAY_PREVIEW_ROWS = 40` (`:979`) with the rationale
at `:974-978`: a row in a wide table is not a row in a long one, so reusing `PREVIEW_ROWS` would
mean previewing a hundred times more data. That is the kind of constant that is usually copied
without thought, and here it was not.

Depth units are converted at four commit sites (`:1126`, `:1224`, `:1299`, `:1411`) and the dialog's
confirmed file unit is honoured at `ingest.rs:673-683`. Status `PRESENT-OK`. The one gap is the
**rightmost-separator decimal rule** in `parse_number` (`intake.rs:134`), which resolves `1.234,5`
and `1,234.5` by taking the rightmost separator as the decimal mark. That is the right default and
it is undocumented as a *parameter*; §5 carries it.

### 3.8 The LAS writer — four defects in 270 lines

`export.rs` is the weakest module in the domain and the one whose defects reach the client
directly.

**(1) The hardcoded `.M`.** `:77`, `:78`, `:79`, `:85`. Covered in §3.1; it breaks SandiBumi's own
round trip. `SB-DIO-017`.

**(2) A second canonical-unit definition.** `standard_units()` at `export.rs:12-22` is the second
place in the codebase that decides what unit a curve is written in; `curves.rs:21-37` `FAMILIES` is
the first, and the two **disagree on spellings**. `export.rs` makes **zero `units::` calls** and
zero `curves::` calls. This is a direct `SB-CORE-007` violation — one definition per constant and
transform — and it is the kind that stays invisible until a client's loader is case- or
spelling-sensitive. `SB-DIO-018`.

**(3) A declared `STEP` computed from two samples.** `export.rs:60`:
`let step = if depth.len() > 1 { depth[1] - depth[0] } else { 0.0 };`. On an irregular index — a
merged multi-run well, a core-depth-shifted composite, a resampled join — the first interval is not
the step, and declaring it as `STEP.M` in `~W` tells the reader the file is uniformly sampled when
it is not. A conforming LAS reader is entitled to reconstruct depths from `STRT`/`STEP` rather than
from the `DEPT` column. LAS 2.0 provides for this case: `STEP` of `0` declares a non-uniform index.
`SB-DIO-056` requires the step be **verified across the whole index** and written as `0` when it
varies. `PRESENT-DIVERGENT`.

**(4) The export is a subset and does not say so.** `export_las` (`:24`) writes the six standard
curves plus computed ones. The entire generic curve store — every column `intake.rs` carefully
carried, every extra DLIS channel — is **omitted with no note**. A user who imported forty curves
and exported "the LAS" gets eight. `SB-CORE-002` again: a degraded result presented as clean.
`SB-DIO-055`.

`NULL_VALUE = -999.25` (`export.rs:8`) is correct and correctly declared in `~W` at `:80`.

### 3.9 Office writers and the Python gate

`office.rs` (2,231 lines) writes `.xlsx` via `xlsxwriter`, `.docx` via `python-docx` and `.pptx` via
`python-pptx` + `matplotlib`, each in a Python subprocess. Its doc-comment (`:1-27`) states two
rules worth quoting as design intent: *numbers stay numbers* and *a blank is not a zero* — the
second is the write-side statement of the same distinction §2 found missing in every incumbent's
delimited reader.

`office_support()` (`:91`) probes for each library and reports per-format availability (`:68`), so
a missing library disables a format rather than failing mid-write. Depth units come from
`units::project_depth_unit_or_default` at `:673`, `:993`, `:1713` — so the office writers do consult
the unit system that `export.rs` does not.

**The Python prerequisite gates four formats.** `python_engine.rs:177` `find_python()` searches
`SANDIBUMI_PYTHON` (`:41`), then the legacy `ARSHILLA_PYTHON` (`:45`, with the rationale at
`:182-183` that a user who set the old variable must keep working), then
`%LOCALAPPDATA%\Programs\Python\Python313|312|311|310` (`:193-194`), then `PATH`. Without it, DLIS
read and `.xlsx`/`.docx`/`.pptx` write are all unavailable. `27_ip-install-blockers.md` owns this as
a deployment question; this chapter owns only the requirement that the *format* fail loudly and
name the fix, which `:48` and `office.rs:316`/`:334-335` already do.

**A pointer inherited from `docs/commercial/PROVENANCE_SWEEP.local.md` row 23 is now stale** and is
corrected here rather than repeated: the user-facing messages no longer name `ARSHILLA_PYTHON`. A
repo-wide search finds it only inside `python_engine.rs` itself, as a deliberate backward-compatible
fallback. Recorded in §7.2 as **E-2**.

### 3.10 Plate workbooks and the `.xls` refusal — the decision this chapter carries

`01_PRODUCT.md` §4.1 assigns this chapter the decision. The shipped behaviour is
`images::probe_plate_workbooks` (`:826`): the filter at `:832-850` accepts `.xlsx` and `.xlsm`
only, and every other path produces a **named refusal carrying the fix** — *"only the newer .xlsx
workbook can be read. Open it in Excel and Save As .xlsx, then import that — the depths live in
cells, and reading them out of the old format without the worksheet they belong to would mean
guessing."* The rationale is at `:827-830`: pictures can be recovered by scanning for image blobs,
but tying each one back to its worksheet — *and therefore to its depth* — needs a full BIFF parser,
and a guessed depth association is what the module refuses to produce.

`docs/record_petrography.md:612-620` records the ratio and two further deliberate choices: `.xls`
**stays in the file-dialog filter on purpose**, so selecting one gets a named refusal rather than a
picker that appears broken; and the behaviour is pinned by two tests, the refusal itself and
`the_newer_workbook_formats_are_accepted`, the latter existing so nobody "tidies" `.xlsm` out of
the filter — it is the same package with macros in it.

**The position this chapter takes.** The refusal is *correct as engineering* and *wrong as a
permanent position*, and those are separable.

It is correct as engineering because the thing being refused is not "reading the file" but
"inventing the plate-to-depth association". A plate hung off the wrong sand is a wrong geological
conclusion delivered with a photograph attached to it, which is more persuasive and therefore worse
than no plate at all. Every element of the shipped behaviour — the named refusal, the fix in the
message, the extension left in the dialog, the test pinning it — is what `SB-CORE-002` asks for. It
is a **defect refusal** and it is recorded as a win in §7.3.

It is wrong as a permanent position because of the ratio. **107 of 165 workbooks — 65 % — are
`.xls`**, so the shipped product asks the user to hand-convert two files in three before it will
look at their petrography. At that fraction the refusal is not a rare edge case being handled
gracefully; it is the **majority path**, and a majority path that ends in a manual workaround is a
capability gap wearing a good error message.

Three facts settle it in favour of closing the gap rather than defending the refusal:

1. **The blocker is a published specification, not a proprietary one.** The BIFF record structure —
   including `BoundSheet8`, the drawing-object records and their sheet anchors, which are precisely
   the records that carry the association — is documented by Microsoft in the `[MS-XLS]` Open
   Specification. Reading it is not reverse-engineering; it is implementing a published format,
   which `CONTRACT.md` §2.2 as amended explicitly permits and which is what SandiBumi already does
   for LAS and RP66.
2. **The approach is proven on Jauhar's own delivered work.** Wellsite `.xls` exports that are BIFF
   streams rather than Excel workbooks were decoded on the SCS-PHM mudlog rebuild via a magic-byte
   sniff, a forced `xlrd` path and a direct BIFF2 decode, and the method is packaged as the
   `mudlog-data-to-las` skill. That is a `P`-tier source for feasibility, not for any parameter.
3. **The all-or-nothing rule survives the change.** Closing the gap does not mean guessing. A plate
   whose sheet cannot be resolved from the drawing anchors is **dropped and counted by name**, which
   is the rule `images.rs` already applies to sub-`MIN_PLATE_PX` decorations (`:479`, and the record
   at `record_petrography.md:606-610` notes real deliveries dropped 117×59 and 207×79 graphics
   against 1920×1080 plates, every drop counted and named per sheet).

So: **`SB-DIO-058` requires `.xls` be read from the published specification, at P2, with the
association rule unchanged — resolved or dropped-and-counted, never guessed.** Until it ships, the
refusal stands exactly as written; it is the right behaviour for a capability that does not exist
yet. Status of the shipped code: `PARTIAL`.

One boundary: a *tabular* `.xls` — a petrography point-count table with no embedded pictures — needs
only the cell records, not the drawing records, and is a strictly smaller problem. `SB-DIO-059`
splits it out at P2 so the common case is not held hostage to the harder one.

### 3.11 Provenance into the deliverable — `SB-CORE-010`

`SB-CORE-010`'s scope was resolved on 2026-08-07 to extend **into the deliverable**. Export is
where that is discharged or lost. Verified at source on the same date:

- `report.rs` (1,442 lines): **zero** case-insensitive matches for `provenance`, and zero for
  `facies`, `cluster`, `leaderboard`, `hfu`.
- `export.rs` (270 lines): **zero** `units::` calls, zero provenance, no `~O` other-information
  section written at all.

A LAS or a report leaving SandiBumi today carries no statement of which curves are measured, which
are computed, what method computed them, or what parameters that method used. A client cannot tell a
logged `GR` from a reconstructed one. Status `ABSENT`. `SB-DIO-051` and `SB-DIO-052` own it, and
they are the requirements most likely to be dismissed as cosmetic and least safely dismissed: the
`~O` section is free, standard, and the only place in a LAS file where the answer to *where did this
number come from* can be written down.

### 3.12 As-built status summary

| # | Capability | Status | Principal evidence |
|---|---|---|---|
| 1 | Depth-unit parse / carry / canonicalise / expose | `PRESENT-OK` | `units.rs:81,98,122,149`; `parsers.rs:444`; `ingest.rs:262` |
| 2 | Depth-unit **enforcement** on an undeclared file | `PARTIAL` | `units.rs:220` — `Assumed`, not refused |
| 3 | Depth unit on the **DLIS** path | `ABSENT` | `dlis.rs` — no `units::` at any line; index skipped at `:68-69` |
| 4 | Depth unit on the **LAS write** path | `PRESENT-DIVERGENT` | `export.rs:77-79,85` hardcoded `.M` |
| 5 | Declared-`NULL.`-beats-list | `PRESENT-OK` | `parsers.rs:143-148` |
| 6 | Null comparison tolerance | `PRESENT-DIVERGENT` | `parsers.rs:132-134` abs vs `:138-140` rel (S-D-2) |
| 7 | DLIS sentinel screen | `PRESENT-DIVERGENT` | `dlis.rs:183` unconditional, uncounted |
| 8 | LAS index resolution (positional guard) | `PRESENT-OK` | `parsers.rs:162-168` |
| 9 | Core-table depth resolution | `PRESENT-OK` | `parsers.rs:642` |
| 10 | Tops depth resolution | `PRESENT-DIVERGENT` | `parsers.rs:1607` admits `TVD` silently (S-D-1) |
| 11 | Coverage-aware alias resolution | `PRESENT-OK` | `parsers.rs:323-334,413` — ahead of all three incumbents |
| 12 | Alias-choice reporting | `ABSENT` | no note emitted by `pick` |
| 13 | Truncated-row handling | `PRESENT-OK` | `parsers.rs:311-317` hard error |
| 14 | Wrapped LAS | `PRESENT-OK` | `parsers.rs:209,289` token buffering |
| 15 | LAS 3.0 | `ABSENT` | no `~` associated-section handling; read silently as 2.0 |
| 16 | DLIS array / image channels | `ABSENT` | `dlis.rs:71-72` silent skip |
| 17 | DLIS frame-level skips | `PRESENT-DIVERGENT` | `dlis.rs:47,50,55,57,60` — seven silent `continue`s |
| 18 | Intake declared layout + bad-cell location | `PRESENT-OK` | `intake.rs:115,386-397,1599-1600` |
| 19 | Unclaimed-column carry to `aux_data` | `PRESENT-OK` | `intake.rs:9,36,1288` |
| 20 | LAS `STEP` declaration | `PRESENT-DIVERGENT` | `export.rs:60` — first interval, not verified |
| 21 | LAS export completeness | `PRESENT-DIVERGENT` | `export.rs:24` — generic store omitted silently |
| 22 | Canonical-unit single definition | `PRESENT-DIVERGENT` | `curves.rs:21-37` vs `export.rs:12-22` (`SB-CORE-007`) |
| 23 | Office writers (xlsx/docx/pptx) | `PRESENT-OK` | `office.rs:91,673,993,1713` |
| 24 | Python discovery + per-format degradation | `PRESENT-OK` | `python_engine.rs:177,193-194`; `office.rs:68` |
| 25 | Old `.xls` read | `PARTIAL` | `images.rs:832-850` — correct refusal, 65 % of the delivery |
| 26 | Provenance in the deliverable | `ABSENT` | `report.rs`, `export.rs` — zero matches (`SB-CORE-010`) |
| 27 | LIS / WITSML / RP66 write | `ABSENT` | no module exists |

Nine `PRESENT-OK`, eight `PRESENT-DIVERGENT`, three `PARTIAL`, seven `ABSENT`. The eight
`PRESENT-DIVERGENT` rows are the chapter's highest-value output: each is working code that produces
a wrong answer without saying so, which is the only defect class that survives a demo.

---

## 4. Requirements

Ids are permanent and are never renumbered; the ids used here are the ones §1 and §2 already
committed to. Priorities are `CONTRACT.md` §3's: **P0** blocks the first sale, **P1** blocks a
credible second, **P2** is the first release after, **P3**/**P4** are horizon. *Status* is the
shipped state the requirement changes, established in §3. Tests are defined in §6.

**Ten P0s**: `SB-DIO-004`, `-015`, `-016`, `-017`, `-023`, `-031`, `-051`, `-054`, `-055`, `-061`.

### 4.1 Absent values and null conventions

**`SB-DIO-001` — A single declared sentinel MUST reach every writer.** · **P1** · `PARTIAL`

> The project's absent-value sentinel **MUST** be defined once and **MUST** be threaded to every
> export path. A writer that cannot accept it **MUST** fail at build time, not silently emit its
> own.

**Rationale.** D-2: Geolog sets `_missing_value = -999.25` once (`log_export.tclsh:19`) and threads
it to eight of twelve writers; four — including `unl`, the **shipped default `contractor_format`**
— receive no `missing_value` argument at all, so a Geolog user exporting at defaults does not get
the sentinel they set. D-1: IP's five curve writers carry two different values. The obligation is
not "have a single sentinel" but "thread it to every writer", which is testable and stronger.
SandiBumi has one writer today (`export.rs:8`), so this is cheap now and expensive later.
**Verified by** `SB-DIO-T01`, `SB-DIO-T02`.

---

**`SB-DIO-002` — The default export path MUST NOT be the one that bypasses the sentinel.** · **P1**
· `PRESENT-UNVERIFIED`

> The default export format **MUST** be one that honours the declared sentinel, and any format that
> cannot **MUST** be marked in the format picker.

**Rationale.** D-2's real lesson is the *default*: Geolog's `unl` is both the shipped default and
one of the four unthreaded writers, so the failure lands on users who changed nothing. A defect
reachable only by an unusual choice is a bug; one reachable by the default is a product decision.
**Verified by** `SB-DIO-T03`.

---

**`SB-DIO-003` — "This channel has no null" MUST be a first-class state.** · **P2** · `ABSENT`

> A channel **MUST** be able to declare that it has no absent-value convention, and such a channel
> **MUST NOT** be screened against any sentinel list. The state **MUST** be distinguishable from
> "no setting supplied".

**Rationale.** D-3: of Techlog's 21 null patterns, **16 carry `<NullValues/>` — empty, meaning no
null at all** — and every one is an array or waveform channel. A global `-999.25` screen *invents* a
null the vendor explicitly says does not exist, punching holes in real waveform amplitude data at
exactly the sample where amplitude happens to be −999.25. Without a first-class "no null", the only
protection is disabling screening globally.
**Verified by** `SB-DIO-T04`, `SB-DIO-T05`.

---

**`SB-DIO-004` — Null recognition MUST be one relative-tolerance transform, and recognition MUST
NOT rewrite.** · **P0** · `PRESENT-DIVERGENT`

> Null comparison **MUST** have exactly one definition, using a relative tolerance with a floor. The
> absolute-`f32::EPSILON` form at `parsers.rs:132-134` **MUST** be deleted and its callers routed to
> the relative form at `:138-140`. Recognising a sentinel **MUST** convert it to the internal absent
> representation and **MUST NOT** rewrite it to a different sentinel.

**Rationale.** Two halves, one requirement, because they are the same boundary. (a) `f32::EPSILON`
is 1.19e-7 — the float spacing near 1.0. Near −999.25 the actual `f32` spacing is ≈6.1e-5, about
512× larger, so `is_las_null` is an exact-equality test wearing the costume of a tolerance test: a
`-999.2500001` from a different decimal formatter, or a value through one `f32`↔`f64` round trip,
enters the arithmetic as a real measurement of −999.25. Dossier **S-D-2**, still live, and the
screen `dlis.rs:183` also depends on — two readers, one defect, hence P0. Direct `SB-CORE-007`
violation. (b) §1.1: IP's *Clean Data* module canonicalises `-999.25 → -999` with the rule shipped
enabled — a QC module performing a null-convention mutation. SandiBumi splits it: the recognition
set is `DIO`'s, any rewriting of a value is `ENV`'s.
**Verified by** `SB-DIO-T06`, `SB-DIO-T07`, `SB-DIO-T08`.

---

**`SB-DIO-005` — Null values MUST be per-channel and plural.** · **P1** · `ABSENT`

> The null convention **MUST** be resolvable per channel and **MUST** admit more than one value per
> channel. A single global value **MUST NOT** be the only representable configuration.

**Rationale.** D-3: the five populated Techlog patterns carry three different values — 3× `-999.25`,
1× `-999` (SonicVision), 1× `-32767` (Baker waveforms). A single global screen is **wrong on 18 of
21 patterns**. `parsers.rs:130` ships a global two-element list.
**Verified by** `SB-DIO-T09`.

---

**`SB-DIO-006` — The null-exception rule shape MUST be many-to-many.** · **P1** · `ABSENT`

> The rule **MUST** be `{names: [regex], nulls: [f64] | NoNull}`. A one-name-per-rule binding
> **MUST NOT** be used.

**Rationale.** D-4 (dossier T-D-1): fifteen of Techlog's sixteen `<Channel>` elements hold one
`<Name>` and one `<NullValues>`; the sixteenth — Weatherford CXD — packs **six `<Name>`/`<NullValues/>`
pairs inside one `<Channel>`**. A strict one-name-per-channel binding keeps the first or the last and
**silently drops five vendor patterns**. The many-to-many shape is correct under either resolution of
the open question at §7.1 O-6, so it is not blocked on it.
**Verified by** `SB-DIO-T10`.

---

**`SB-DIO-007` — Absent MUST be distinguishable from nulled.** · **P2** · `ABSENT`

> A delimited reader **MUST** record whether a cell was *empty* (nothing between the delimiters) or
> *explicitly nulled*, and the distinction **MUST** survive to the deliverable.

**Rationale.** D-33: no incumbent delimited reader preserves it. The distinction matters at exactly
the moment someone asks whether a laboratory reported a zero, reported nothing, or was never asked.
`intake.rs:76` already types cells as `"number" | "text" | "empty"`, so the read-side machinery
exists and the gap is in carrying it through.
**Verified by** `SB-DIO-T11`.

### 4.2 Alias resolution

**`SB-DIO-008` — Coverage-aware alias resolution MUST be preserved.** · **P1** · `PRESENT-OK`

> Among columns matching an alias set, the importer **MUST** select the one with the most finite
> samples, breaking ties by alias priority with replacement only on strictly greater coverage. The
> rule **MUST** be deterministic across runs and machines.

**Rationale.** `parsers.rs:323-334`, `:413`. The one behaviour in this domain ahead of all three
incumbents, which all take the first match — and first-match binds the standard neutron slot to an
empty simulated `NPHIED` in preference to a populated `NPHI_LS`, producing an all-null porosity that
looks like a *processing* failure rather than an *import* failure, sending the user to the wrong
module to debug it. The requirement exists to stop a future refactor "simplifying" it. The
strict-greater tie-break is load-bearing: without it the rule is irreproducible, and an
irreproducible import is unauditable.
**Verified by** `SB-DIO-T12` (characterization), `SB-DIO-T13`.

---

**`SB-DIO-009` — The alias choice MUST be reported.** · **P1** · `ABSENT`

> When more than one column matched an alias set, the importer **MUST** state which column it bound,
> which it passed over, and the coverage counts that decided it.

**Rationale.** The `pick` closure (`parsers.rs:323-334`) is silent. A user cannot learn that `NPHIED`
was passed over, and if the heuristic ever chooses wrong they cannot see that it did. A
correct-but-silent heuristic is one refactor from being a wrong-and-silent one.
**Verified by** `SB-DIO-T14`.

### 4.3 Index detection

**`SB-DIO-010` — Prefer a structural index declaration; fall back to names; record which mechanism
fired.** · **P1** · `PARTIAL`

> Where a format declares its index structurally, the reader **MUST** use that declaration. Where it
> does not, the reader **MAY** fall back to a name list or to the positional guarantee. In every
> case the reader **MUST** record which mechanism resolved the index.

**Rationale.** D-8: Geolog's flat-ASCII specs declare `CLASSES = REFERENCE | LOG` per column — the
only structural index declaration among the three tools — while its own LAS module falls back to
`_ref_in = DEPTH` (`log_load_dxs_las.tclsh:97`). The lesson is per-path, not per-tool. SandiBumi's
LAS path already implements the strongest available form (the positional guarantee, with the
rationale at `parsers.rs:162-167`); what is missing is the *record of which mechanism fired*, which
is what lets a user audit a wrong index after the fact.
**Verified by** `SB-DIO-T15`, `SB-DIO-T16`.

---

**`SB-DIO-011` — Index aliases MUST be namespace-aware and MUST have one definition per path.** ·
**P1** · `PRESENT-DIVERGENT`

> Index alias lists **MUST** be derived from a single documented source per path, and an alias
> declared in a different namespace by its originating vendor **MUST NOT** be admitted as an index
> alias.

**Rationale.** D-7: real Geolog's `alias.alias:14` declares `DEPTH = SCD IDWD DVP1 PDEP_XPT DEPM
TDEP` under `# aliases for references`, while `TVD` sits at line 891 under `# aliases for welltie`
— **a different declared namespace**. IP accepts `TVD` as an MD index (wrong by the vendor's own
structure) while missing **five of the six real Geolog depth aliases**. SandiBumi ships three
disagreeing lists (`parsers.rs:168`, `:642`, `:1607`, dossier **S-D-1**) — two justified by their
paths (§3.3) and one not. This requirement forbids the *undocumented* divergence, not the
per-path one.
**Verified by** `SB-DIO-T17`.

---

**`SB-DIO-012` — A non-monotonic index MUST be detected and reported, never silently accepted.** ·
**P1** · `PARTIAL`

> The reader **MUST** detect a non-increasing index, **MUST** report where it occurs, and **MUST**
> require a user decision before committing.

**Rationale.** D-8: Techlog's ASCII wizard makes the reference designation mandatory *and* constrains
it to be strictly increasing. A non-monotonic index is either a splice, a wrap, or a
mis-identified column — three different problems with three different answers, none of which the
importer may choose alone. §1.1 assigns detection to `DIO` because it is a structural property of
the file; tie-in and shifting are `ENV`'s.
**Verified by** `SB-DIO-T18`.

---

**`SB-DIO-013` — When neither structure nor name resolves an index, the user MUST designate it.** ·
**P1** · `PARTIAL`

> The importer **MUST** fall back to a mandatory user designation rather than to a positional
> guess, in every format that lacks a positional guarantee.

**Rationale.** D-8's third form. `intake.rs` already declares layout rather than sniffing it, so the
pattern exists; the requirement generalises it to the delimited and core-table paths, where
`CORE_DEPTH_ALIASES` (`parsers.rs:642`) currently ends in a silent failure to resolve.
**Verified by** `SB-DIO-T19`.

---

**`SB-DIO-014` — TVD MUST NOT be read as an MD index.** · **P1** · `PRESENT-DIVERGENT`

> When a depth column resolves via a TVD alias, the resulting data **MUST** be recorded as
> TVD-referenced and **MUST NOT** be joined to, plotted against, or compared with an MD-indexed log
> until a deviation survey is present. The alias **MUST NOT** be removed.

**Rationale.** `parsers.rs:1607` `TOPS_DEPTH_ALIASES` admits `TVD`, so a tops file carrying only
`TVD` is silently read as measured depth. D-7 exposes the arithmetic: the deficit is `(1 − cos θ̄)`
of the measured depth traversed, and on a 3,000 m well with a 40 % tangent at 30° — a typical
deltaic-clastic development well — that is **161 m**. Markers are strictly worse than curves here, because a curve
carries a shape a reader might recognise as displaced and a marker carries nothing at all. Removing
the alias is explicitly rejected: a TVD-only tops file is a real file, and turning a wrong answer
into an unexplained failure is a different `SB-CORE-002` violation, not a fix.
**Verified by** `SB-DIO-T20`, `SB-DIO-T21`.

### 4.4 The depth unit — the `SB-CORE-001` half this chapter owns

**`SB-DIO-015` — An index with no declared unit anywhere MUST refuse.** · **P0** · `PARTIAL`

> When neither the project nor the file declares a depth unit, the importer **MUST** refuse and
> **MUST** name both places a unit could have come from. When the project declares a unit and the
> file does not, the importer **MUST** require an explicit per-import confirmation of the *file's*
> unit before committing, and **MUST NOT** treat the project setting as the file's declaration.
> Neither case may be discharged by a note.

**Rationale.** `units.rs:220` returns `Assumed` for both cases and the import proceeds. A note is not
a refusal — it is a line in a list a user scrolls past — and the cost of scrolling past it is a
**3.28× error in the index**, which every downstream product inherits. `15_sat-height-rocktyping.md`
R14 is the arithmetic consequence. The second arm is the dangerous one: `(Some(p), None)` names the
project's own unit in its note, so it reads as confirmation rather than as a guess. Four of the five
match arms are already right; this requirement changes two.
**Verified by** `SB-DIO-T22`, `SB-DIO-T23`, `SB-DIO-T24`.

---

**`SB-DIO-016` — The DLIS index unit MUST be read and reconciled.** · **P0** · `ABSENT`

> The DLIS reader **MUST** read the `UNITS` attribute of the index channel, **MUST** pass it to
> `units::resolve_index_unit`, and **MUST** apply `SB-DIO-015`'s refusal rule.

**Rationale.** `dlis.rs` makes no `units::` call at any line, and the sidecar's emit loop skips the
index channel (`:68-69`) so its unit never leaves Python — although `unit_by_name` is built over all
channels at `:61-66`. Per-channel units *are* canonicalised (`:190-193`), which makes this worse
rather than better: every curve unit on the well is correct and the depth is wrong by 3.28×, so
nothing on screen looks suspicious. Feet indexes are the norm in DLIS from US-heritage service
companies. Found by this chapter; not in the dossier's self-audit.
**Verified by** `SB-DIO-T25`, `SB-DIO-T26`.

---

**`SB-DIO-017` — The LAS writer MUST write the depth unit it actually used.** · **P0** ·
`PRESENT-DIVERGENT`

> `STRT`, `STOP`, `STEP` and the index curve's `~C` line **MUST** carry the unit of the depths
> written, obtained from `units::`. A unit **MUST NOT** be a string literal in a writer.

**Rationale.** `export.rs:77-79`, `:85` hardcode `.M`. A project in feet exports feet-valued numbers
labelled metres; re-importing that file parses `M`, resolves `Convert { from: Metres, to: Feet }`
(`ingest.rs:162-166`) and divides by 0.3048 — **the file does not survive SandiBumi's own round
trip**. The existing test `an_exported_las_reimports_with_the_same_values` cannot catch it because
its fixture project is in metres, where the literal happens to be true: the characterization blind
spot `CONTRACT.md` §3 warns about.
**Verified by** `SB-DIO-T27`, `SB-DIO-T28`.

---

**`SB-DIO-018` — Canonical units MUST have exactly one definition.** · **P1** ·
`PRESENT-DIVERGENT`

> The canonical unit for a curve family **MUST** be defined once, in `curves.rs::FAMILIES`, and
> every writer **MUST** obtain it from there. `export.rs::standard_units` **MUST** be deleted, not
> reconciled.

**Rationale.** `curves.rs:21-37` and `export.rs:12-22` are two definitions of the same thing and they
disagree on spellings; `export.rs` makes zero `curves::` and zero `units::` calls. Direct
`SB-CORE-007` violation. Reconciling rather than deleting would leave the divergence free to reopen
at the next edit — the failure mode that produced it.
**Verified by** `SB-DIO-T29`, `SB-DIO-T30`.

---

**`SB-DIO-019` — Changing the project depth unit MUST NOT silently rescale stored data.** · **P1**
· `PRESENT-UNVERIFIED`

> Changing the project depth unit after data has been committed **MUST** be an explicit,
> user-confirmed migration stating how many wells and curves it will rewrite, or **MUST** be refused
> while data exists.

**Rationale.** `units.rs:169` writes a settings row and `ingest.rs:284-288` adopts a unit
post-commit; what happens to already-committed depths when the setting changes is **not established
by the code read for this chapter**, and the failure mode — half a project in one unit — is
unrecoverable. The status is `PRESENT-UNVERIFIED` deliberately: this requirement is discharged by a
test that establishes the behaviour, not by an assumption about it. `22_database-model.md` owns the
migration mechanics; this chapter owns the rule.
**Verified by** `SB-DIO-T31`.

### 4.5 Sampling, units and conversion

**`SB-DIO-020` — Duplicate depths MUST be resolved by a declared policy, and the count reported.**
· **P1** · `PARTIAL`

> Repeated index values **MUST** be resolved by a policy the user declared — keep-first, keep-last,
> mean, or refuse — and the importer **MUST** report how many rows were affected. They **MUST NOT**
> be resolved by whichever row the primary key happened to accept.

**Rationale.** `parsers.rs:453` sanitises duplicate depths precisely so a `(curve_id, depth)`
collision cannot abort a whole file, and `dlis.rs:218` does the same via `depth_keep_indices`. The
sanitation is right; its *invisibility* is not. Duplicate depths in real deliveries come from run
splices and from unit-mixed merges, and the correct answer differs between the two. §1.1: the
constraint that motivates the policy is `DBM`'s, the policy is `DIO`'s, and relaxing the PK would
not retire this requirement because dropping-with-a-count is right regardless.
**Verified by** `SB-DIO-T32`, `SB-DIO-T33`.

---

**`SB-DIO-021` — Resampling on read MUST be explicit, named, and off by default.** · **P2** ·
`PRESENT-OK`

> No reader may change the sample interval of incoming data without an explicit user instruction,
> and the operation performed **MUST** be named in Techlog's vocabulary (decimate / interpolate /
> average / nearest), not as "resample".

**Rationale.** D-20: IP resamples on load in at least nine documented paths, mostly silently. D-21:
Techlog is clearly ahead here and its vocabulary is the one to adopt, because the four operations
have different error characteristics and calling them all "resample" hides the choice. SandiBumi
does not resample on read today, so this requirement is a **lock**, not a build.
**Verified by** `SB-DIO-T34`.

---

**`SB-DIO-022` — Re-grid on write MUST be named correctly and default OFF.** · **P1** · `ABSENT`

> Any writer-side re-gridding **MUST** be presented as what it is — a resample of the output — and
> **MUST** default to off. When on, the written file **MUST** record that its samples are not the
> stored samples.

**Rationale.** D-22, the half every load-side analysis misses: IP re-grids on **write**, so the
comparison a user would make to detect it (export, re-import, diff) is unavailable — on export there
is nothing left to compare against. D-23 adds that the control changes meaning when the step changes,
which is why it must be named by its effect rather than by its widget label.
**Verified by** `SB-DIO-T35`.

---

**`SB-DIO-023` — Numeric columns MUST be validated against physical bounds, not against their
labels.** · **P0** · `ABSENT`

> Every column bound to a known curve family **MUST** be checked against that family's plausible
> physical range, and a column whose values are implausible for its bound family **MUST** raise a
> blocking question before commit — **even when the label matches an alias exactly**.

**Rationale.** D-32 (`P`, memory `reference_mudlog_gas_curve_traps`, from delivered mudlog rebuilds,
and it is a requirement rather than a tip). Vendor exports shift a column while every header label
still matches, so name-based resolution binds the right label to the wrong data and nothing in the
pipeline notices. The only check that catches it is physical: a gamma ray that never exceeds 4, a
resistivity running negative, a neutron porosity of 340. P0 because the failure is silent, the file
is structurally clean, and the wrong number is petrophysically *usable* — it computes, it plots, and
it ships into a deliverable. The seam to `ENV` is that `DIO` raises the flag on import; `ENV` owns
what to do about a measurement that is merely improbable.
**Verified by** `SB-DIO-T36`, `SB-DIO-T37`, `SB-DIO-T38`.

---

**`SB-DIO-024` — Unit conversion MUST NOT be applied silently by default.** · **P1** · `PARTIAL`

> Automatic unit conversion on import **MUST** be off by default, or — if on — **MUST** report every
> curve it converted, with the from-unit, the to-unit and the factor.

**Rationale.** D-10: Geolog over-converts silently by default; `PG_UNIT_CONVERT=YES` is the shipped
setting. SandiBumi converts to canonical units at `curves.rs:56` and `dlis.rs:192` with no report,
which is the same behaviour with a smaller table. Conversion is usually right and occasionally
catastrophic, and the difference is invisible without a record.
**Verified by** `SB-DIO-T39`.

---

**`SB-DIO-025` — Conversion coverage MUST be declared, and an unconvertible unit MUST be reported
rather than passed through.** · **P1** · `PARTIAL`

> The set of quantity families the unit system can convert **MUST** be documented and queryable. A
> curve whose declared unit is outside that set **MUST** be flagged as unconverted, not silently
> stored as if canonical.

**Rationale.** D-9: IP's entire numeric conversion capability is a **63-line file covering four
families** — SONIC, DENSITY, CALIPER, POROSITY — and nothing else converts: resistivity, GR,
temperature, salinity, permeability, pressure and CEC/Qv all pass through unconverted. Its own header
comment claims **three** families, omitting POROSITY — a shipped file wrong about itself. D-10:
`OpenSpiritUnits.opt` (5,662 lines, 1,132 records) has **zero** conversion factors; it is a
vocabulary, not a table. SandiBumi's `curves.rs:21-37` covers 14 families and `convert_to_canonical`
(`:56`) returns a bool that `dlis.rs:192` checks and the LAS path does not.
**Verified by** `SB-DIO-T40`, `SB-DIO-T41`.

---

**`SB-DIO-026` — Unit conversion MUST support affine transforms.** · **P1** · `ABSENT`

> Conversions **MUST** be represented as `{factor, offset}` and the offset **MUST** be applied. A
> unit requiring a non-zero offset **MUST NOT** be treated as multiplicative.

**Rationale.** D-13: `volume_ratio.units` and the temperature case. °F→°C is affine, and applying
only the factor carries a **32-degree offset into any `Rw(T)` or `B(T)` computation downstream** —
which lands in every Waxman-Smits and every Archie-with-temperature-corrected-`Rw` result in the
project. `curves.rs:56` `convert_to_canonical` is multiplicative only.
**Verified by** `SB-DIO-T42`.

---

**`SB-DIO-027` — A vendor alias that is wrong or ambiguous MUST NOT be inherited.** · **P1** ·
`ABSENT`

> Alias tables imported from a vendor artefact **MUST** be reviewed per entry against the physical
> quantity, and an entry that is wrong or that maps two quantities to one symbol **MUST** be
> rejected with the rejection recorded.

**Rationale.** D-13 and D-14: `PPG` is a shipped vendor alias in `density.units` and is a
**pressure-gradient** unit, not a density — inheriting it silently binds a mud-weight column to a
density family. The general rule matters more than the instance: a vendor alias table is evidence of
what names occur in the wild, not an adjudication of what they mean.
**Verified by** `SB-DIO-T43`.

---

**`SB-DIO-028` — A conversion factor MUST be correct and MUST show its derivation.** · **P1** ·
`ABSENT`

> Every conversion factor in the unit table **MUST** carry its derivation in the table itself, and
> **MUST NOT** be copied from a vendor file without independent arithmetic.

**Rationale.** D-11 (dossier **G-D-1**, a defect in a shipped vendor table): Geolog's own table
mis-converts, and the corrected factor appears as a §5 row **with its derivation shown**, which is
the only form in which a reviewer can check it. This is `CONTRACT.md` §2.1 discipline applied to
units: a number without a derivation is a number nobody can audit, and a vendor file is not a
citation for arithmetic.
**Verified by** `SB-DIO-T44`.

---

**`SB-DIO-029` — An unadjudicable unit ambiguity MUST ship with no default.** · **P1** · `ABSENT`

> Where a unit symbol has two legitimate readings that the data cannot distinguish, SandiBumi
> **MUST** ship no default, **MUST** ask, and **MUST** record the answer per file.

**Rationale.** D-12: `MS/FT` is a genuine, unadjudicable two-way ambiguity — microseconds per foot
versus millisiemens per foot — and IP maps it one way. This is the canonical case for
`CONTRACT.md` §2's absent-rather-than-adjudicated rule: choosing silently is a 1000× error in one of
the two readings, and there is no evidence in the file that resolves it.
**Verified by** `SB-DIO-T45`.

### 4.6 Curve identity, renaming and substitution

**`SB-DIO-030` — An alias rename MUST be reported.** · **P1** · `ABSENT`

> When an alias table renames an incoming curve, the importer **MUST** record and display the
> original mnemonic, the applied name, and the table entry that fired.

**Rationale.** D-16: IP applies `CurveAlias.txt` automatically and silently on batch import, so a
curve's name in the project is not the name in the file and nothing reports that a rename occurred.
D-18: Geolog's distinguishing property is **explicitness, not ubiquity** — it does less
automatically and says more about what it did. The original mnemonic is the only key back to the
source file, so losing it silently makes an audit impossible.
**Verified by** `SB-DIO-T46`.

---

**`SB-DIO-031` — A different curve's data MUST NOT be supplied under a requested name.** · **P0** ·
`ABSENT`

> When a requested curve is unavailable, the system **MUST** report it unavailable. It **MUST NOT**
> substitute another curve's data under the requested name, under any configuration.

**Rationale.** D-15 (dossier §3.5 Hazard 2): IP substitutes a different curve's data under the
requested name. This is the most serious single behaviour catalogued in the dossier, because the
result is *correct-looking data of the wrong provenance* — the one failure that no range gate, no
plot inspection and no QC pass can detect. P0 and a **MUST NOT** rather than a MUST: there is no
configuration under which SandiBumi does this. Recorded as a defect refusal at §7.3 R-2.
**Verified by** `SB-DIO-T47`.

---

**`SB-DIO-032` — A substitution offered to the user MUST be explicit and recorded.** · **P1** ·
`ABSENT`

> Where a substitute curve is a legitimate choice, it **MUST** be offered by name, accepted
> explicitly, and recorded on the resulting curve as a provenance entry.

**Rationale.** The other half of D-15. Substitution is sometimes correct — a shallow resistivity
standing in for a missing medium — and the requirement is not to forbid it but to make it an act the
user performed rather than one the importer performed for them.
**Verified by** `SB-DIO-T48`.

---

**`SB-DIO-033` — Curve-selection state MUST be explicit and inspectable.** · **P1** · `ABSENT`

> Which curves are selected for an operation **MUST** be a named, saved, inspectable object, and
> **MUST NOT** be implied by curve type or by a hidden default.

**Rationale.** D-19: IP's mask files are the one IP artefact worth copying wholesale, with their
modes intact. A selection that cannot be named cannot be reviewed, and cannot be shown to have been
the same selection twice.
**Verified by** `SB-DIO-T49`.

---

**`SB-DIO-034` — Curves MUST NOT be auto-selected by curve type on read.** · **P1** · `ABSENT`

> No read path may choose which curve to use on the basis of a curve-type classification without
> stating the choice.

**Rationale.** D-19: IP auto-selects by Curve Type — the read-side twin of D-15. The failure is the
same shape (the wrong data under the right label) reached by classification rather than by
substitution, so forbidding one without the other leaves the hole open.
**Verified by** `SB-DIO-T50`.

### 4.7 DLIS

**`SB-DIO-035` — An import MUST NOT extend an existing object's declared interval.** · **P1** ·
`ABSENT`

> Importing curves that fall outside a well's or set's declared depth range **MUST** raise the
> conflict for a decision. The importer **MUST NOT** widen an existing object's interval to
> accommodate incoming data.

**Rationale.** D-34: IP's DLIS loader ships with *Extend the Well interval if curves are outside the
depth range* **checked**, and *Extend the Set interval* likewise — so importing a file silently
mutates an object that already existed and that other work depends on. A depth range that widens by
itself is usually evidence of a mis-identified index or a wrong unit, which is exactly the diagnostic
the default destroys.
**Verified by** `SB-DIO-T51`.

---

**`SB-DIO-036` — The duplicate-name policy MUST NOT default to merge.** · **P1** · `ABSENT`

> When an incoming curve's mnemonic already exists on the well, the importer **MUST** ask, the
> default **MUST NOT** be merge-into-existing, and the choice **MUST** be recorded per curve.

**Rationale.** D-34: IP defaults to ***Insert New Data into the Existing Curve*** — a silent merge,
and the one dangerous choice of its three, because the result is a curve partly from one run and
partly from another with no record of the seam. A merged curve is unauditable after the fact: there
is no way to recover which samples came from where.
**Verified by** `SB-DIO-T52`.

---

**`SB-DIO-037` — Channels that could not be loaded MUST be named, and a partial load MUST NOT be
reported as success.** · **P1** · `PRESENT-DIVERGENT`

> Encrypted, unsupported or unreadable channels **MUST** be listed by name in the import result, and
> an import that loaded fewer channels than the file contains **MUST** be reported as partial.

**Rationale.** D-34: IP marks encrypted channels with an `x` in the Load column and does not load
them — a silent partial load unless the user inspects the scan. SandiBumi's own DLIS path is worse:
`dlis.rs:71-72` drops every array channel with no marker at all.
**Verified by** `SB-DIO-T53`.

---

**`SB-DIO-038` — Multi-dimensional channels MUST be imported through the published RP66
container.** · **P2** · `ABSENT` · *independently derived, class C-3 — see §7.4*

> The DLIS reader **MUST** read multi-dimensional channels — image passes, NMR echo trains, waveform
> sets — preserving each frame's array shape, per-axis labels and units, from the **published API
> RP66 V1 specification**. It **MUST NOT** consume, decode or infer any vendor's proprietary tile,
> weight or image encoding.

**Rationale.** `dlis.rs:71-72` currently discards every array channel with the comment *skip
array/multidim channels for now*, so a file whose entire payload is image or waveform data imports as
zero curves and no error (`SB-DIO-054`). The capability is real and the user need is real; the
derivation path is the published RP66 container specification and `dlisio`'s open implementation of
it, both of which are public.

**Betters:** Techlog's own documentation directs users to its **proprietary project format** for
borehole-image interchange rather than to an open container, so an image loaded in Techlog cannot be
moved to another tool without that tool reverse-engineering a project file (T3, and the reason §1.2
records `.itt`/`.itp`/`.att`/`.bor`/`.eli` as named-and-not-touched). Reading and writing arrays
through RP66 removes that limitation: the interchange stays in a published container that any
conforming reader can open.
**Verified by** `SB-DIO-T54`, `SB-DIO-T55`.

---

**`SB-DIO-039` — The DLIS sentinel screen MUST be per-channel overridable and MUST count what it
deleted.** · **P1** · `PRESENT-DIVERGENT`

> The screen **MUST** report, per channel, how many samples it converted to absent and on which
> rule, and a user **MUST** be able to disable LAS-sentinel screening for a named channel.

**Rationale.** `dlis.rs:183` applies `is_las_null` unconditionally to every channel. The premise
comment at `:176-181` is **correct** — D-5 confirms RP66 defines no absent value, and producers do
embed LAS-style sentinels — so the screen stays and the ordering (screen before unit canonicalisation,
so a survivor cannot be scaled into an unrecognisable value) is right. What is wrong is that it is
blanket and uncounted: a legitimate DLIS sample of −999.25 in a channel whose range spans it is
deleted without trace. D-3 is independent evidence that a blanket rule is known to be wrong —
Techlog ships a per-channel exception file for exactly this.
**Verified by** `SB-DIO-T56`, `SB-DIO-T57`.

### 4.8 LAS structure and versions

**`SB-DIO-040` — Wrapped LAS MUST be read; the writer MUST emit unwrapped.** · **P2** ·
`PRESENT-OK` (read) / `PRESENT-OK` (write)

> The reader **MUST** handle `WRAP: YES` by buffering tokens across physical lines. The writer
> **MUST** emit unwrapped output.

**Rationale.** D-24: LAS wrapping is a genuine three-way disagreement among the incumbents and only
one of the three handles the general case; the writer recommendation is unchanged across all three —
emit unwrapped. SandiBumi already buffers tokens (`parsers.rs:209`, `:289`), which is the only
correct approach because in wrapped mode a logical record's boundary is a *count*, not a newline. A
**lock**, not a build.
**Verified by** `SB-DIO-T58`.

---

**`SB-DIO-041` — A LAS 3.0 file MUST be recognised, and what is not read MUST be named.** · **P1** ·
`ABSENT`

> The reader **MUST** detect `VERS. 3.0` and, in the release before it can read the format fully,
> **MUST** name every section it is not reading rather than reading the file as 2.0.

**Rationale.** SandiBumi has no `~`-associated-section handling, no `|`-delimited sub-section parsing
and no multi-array support, so a LAS 3.0 file is read as 2.0 and its associated sections — core data,
tops, test results, the very things a 3.0 file exists to carry — are lost **silently**. That silence
is the `SB-CORE-002` problem, and it is separable from the much larger job of reading 3.0 properly,
which is why it is P1 and `SB-DIO-042` is P3.
**Verified by** `SB-DIO-T59`.

---

**`SB-DIO-042` — LAS 3.0 associated sections MUST be read.** · **P3** · `ABSENT`

> The reader **MUST** parse LAS 3.0 associated sections, their `|`-delimited sub-section
> definitions, and multi-array data.

**Rationale.** D-25: Geolog is the only tool whose LAS 3.0 section-parsing contract is stated
explicitly, and its statement is the specification to build against. The CWLS LAS 3.0 document itself
is an acquisition gap — see §7.4 D-3.
**Verified by** `SB-DIO-T60`.

---

**`SB-DIO-043` — LAS 1.2 MUST be readable and MUST NOT be writable.** · **P2** · `ABSENT`

> The reader **MUST** accept LAS 1.2. The writer **MUST NOT** offer it.

**Rationale.** D-26: LAS 1.2 is readable but not writable in both IP and Geolog, and that asymmetry
is correct rather than an oversight — reading a legacy file is a service to the user, writing one
manufactures a file that cannot express what the project holds.
**Verified by** `SB-DIO-T61`.

---

**`SB-DIO-044` — Section-parse strictness MUST be declared and consistent.** · **P2** · `PARTIAL`

> The reader's tolerance for malformed section headers, unknown sections and out-of-order sections
> **MUST** be documented, uniform across LAS versions, and reported when it fires.

**Rationale.** D-25 (dossier §2.9): Geolog's LAS 3.0 parser strictness is the only one documented, so
it is the only one a user can predict. Strictness that varies by version or by code path is
indistinguishable from a bug.
**Verified by** `SB-DIO-T62`.

### 4.9 Containers, headers, precision and provenance

**`SB-DIO-045` — A multi-well container MUST produce multiple wells, never one merged well.** ·
**P1** · `ABSENT`

> A file containing data for more than one well **MUST** produce one project well per source well,
> and **MUST NOT** merge them. The mapping from container to wells **MUST** be shown before commit.

**Rationale.** D-27: "all wells in one file" is not one question — it decomposes per container (LAS
2.0 cannot; LAS 3.0, DLIS and delimited exports can, by different mechanisms). Merging is the failure
that produces a single well with a non-monotonic index and overlapping curves, which then triggers
the duplicate-depth sanitation at `SB-DIO-020` and looks like a data problem rather than an import
problem.
**Verified by** `SB-DIO-T63`, `SB-DIO-T64`.

---

**`SB-DIO-046` — A missing interpreter or library MUST produce a named, actionable, per-format
refusal.** · **P1** · `PRESENT-OK`

> Where a format depends on the Python sidecar, absence of the interpreter or of the specific library
> **MUST** disable *that format* with a message naming the library and the fix, and **MUST NOT**
> degrade the format silently or fail mid-write.

**Rationale.** §1.1: this chapter owns the Python prerequisite **only where it gates a format** —
DLIS needs `dlisio` (`dlis.rs:133`), plate extraction needs `openpyxl` (`images.rs:856`), the
`xlsx`/`pptx`/`docx` writers need `xlsxwriter` / `python-pptx` / `python-docx` (`office.rs:8-10`).
`office_support()` (`office.rs:91`) already probes per library and `:334-335` names the missing one
with its `pip install`; `python_engine.rs:48` names the variable and the version. A **lock**.
Whether requiring Python at all is commercially acceptable is `27_ip-install-blockers.md`'s.
**Verified by** `SB-DIO-T65`.

---

**`SB-DIO-047` — Storage precision MUST be declared and MUST NOT silently truncate.** · **P1** ·
`PRESENT-DIVERGENT`

> The precision at which samples are stored and written **MUST** be documented, and a write that
> reduces precision **MUST** say so.

**Rationale.** D-29: IP writes DLIS as `FSINGL` unconditionally, so any float64 input is silently
truncated. SandiBumi stores `f32` throughout (`parsers.rs`, `dlis.rs`, `export.rs`), which is the
right choice for log data and the wrong thing to leave undocumented — a client supplying float64
core-analysis pressures is entitled to know. D-24 adds the round-trip half: what a file loses on a
SandiBumi round trip must be stated rather than discovered.
**Verified by** `SB-DIO-T66`.

---

**`SB-DIO-048` — Well identity in a container MUST come from the container, never from the
filename.** · **P2** · `PARTIAL`

> The well a curve belongs to **MUST** be resolved from the file's own well-identifying fields. A
> filename **MAY** be offered as a default for the user to confirm and **MUST NOT** be used silently.

**Rationale.** D-27. A filename is a convention, not data, and it is the field most often changed by
the intermediaries a delivery passes through. This is also the boundary at which no client well name
enters this document.
**Verified by** `SB-DIO-T67`.

---

**`SB-DIO-049` — Writing a file our own reader would reject MUST be an error, not a warning.** ·
**P1** · `ABSENT`

> Every writer **MUST** validate its output against SandiBumi's own reader for that format before
> reporting success, and a failure **MUST** be an error.

**Rationale.** D-28 (dossier N-6.12): **IP at factory defaults writes a DLIS its own loader states it
cannot read.** That is the most damning single fact in the dossier, and the cheapest to never
reproduce — the reader already exists, so the check is a function call. `SB-DIO-017`'s hardcoded
`.M` is precisely a defect this requirement would have caught at the moment it was written.
**Verified by** `SB-DIO-T68`, `SB-DIO-T69`.

---

**`SB-DIO-050` — A re-gridded input MUST be detectable at import.** · **P1** · `ABSENT`

> Where a file's declared step disagrees with its actual sample spacing, or its index is uniform to a
> suspiciously round interval inconsistent with the acquisition, the importer **MUST** flag it.

**Rationale.** D-22: because IP re-grids on *write*, the only place the evidence survives is the
importing tool — on export there is nothing left to compare against. This is the read-side half of
`SB-DIO-022` and it is the only defence a SandiBumi user has against a file that was resampled
before it reached them.
**Verified by** `SB-DIO-T70`.

---

**`SB-DIO-051` — Provenance MUST be carried into the deliverable.** · **P0** · `ABSENT`

> Every export **MUST** carry, in the file itself, a record of which curves are measured and which
> are computed; for each computed curve, the method name, its parameters and their values; and for
> any model-derived curve, the record `SB-CORE-014` requires. For LAS this **MUST** use the `~O`
> section.

**Rationale.** `SB-CORE-010`'s scope was resolved on 2026-08-07 to extend **into the deliverable**,
and the export path is where that record is either carried or lost. Verified at source the same day:
`report.rs` (1,442 lines) has **zero** case-insensitive matches for `provenance`, `facies`,
`cluster`, `hfu` or `leaderboard`; `export.rs` (270 lines) has zero `units::` calls and writes no
`~O` section at all. A client receiving a SandiBumi LAS today cannot tell a logged `GR` from a
reconstructed one. P0 because it is the deliverable, because `~O` is free and standard, and because
this is the requirement most likely to be dismissed as cosmetic and least safely dismissed.
`24_ml-advanced.md` owns what a learned model must record; `23_plotting-interactivity.md` discharges
the same obligation for PDF and SVG.
**Verified by** `SB-DIO-T71`, `SB-DIO-T72`, `SB-DIO-T73`.

---

**`SB-DIO-052` — Final and working curves MUST be distinguishable in an export.** · **P1** ·
`ABSENT`

> An export containing both final and intermediate curves **MUST** mark which is which in the file.

**Rationale.** D-17: IP's *Select Final Curves All Wells* pulls curves out of wells the user did not
intend and produces a file mixing final and working curves **with nothing in the file distinguishing
them**. The recipient then has no way to know which `PHIE` is the answer. This is `SB-DIO-051`'s
smallest and most immediately useful case.
**Verified by** `SB-DIO-T74`.

---

**`SB-DIO-053` — Well-header fields MUST be mapped explicitly and identity MUST NOT be invented.** ·
**P2** · `PARTIAL`

> Header mnemonics **MUST** be mapped through a documented table, unmapped headers **MUST** be
> carried verbatim rather than dropped, and no identity field — well name, UWI, field, operator,
> country — may be synthesised, translated or defaulted.

**Rationale.** D-30: header mnemonics have no shared standard across the three tools, so any mapping
is a choice that must be visible. The identity half matters more than the mapping half: a UWI
invented from a filename, or a country code translated through a lookup, is a data-integrity failure
that survives into every downstream register.
**Verified by** `SB-DIO-T75`, `SB-DIO-T76`.

### 4.10 Robustness — the malformed-input contract

`01_PRODUCT.md` records that the malformed-input defect family has **recurred three or more times**
and is a standing risk in the repo's own QC prompt (`docs/qc_audit_prompt_template.md:53`). A defect
that recurs is not a defect; it is a missing contract. This group is what "solved" means, stated as
things a test can fail.

**`SB-DIO-054` — Every skipped frame, channel, curve and row MUST be counted and named.** · **P0** ·
`PRESENT-DIVERGENT`

> No reader may discard a frame, channel, curve or row without emitting a note stating what was
> dropped, how many, and on which rule. An import that dropped everything **MUST** be an error, not
> an empty success.

**Rationale.** `dlis.rs` has seven silent `continue`s (`:47`, `:50`, `:55`, `:57`, `:60`, `:69`,
`:71-72`). A DLIS file whose entire payload is array data imports as **zero curves and no error** —
the canonical `SB-CORE-002` case. P0 because "the import worked and there is no data" is the most
expensive support call a first customer can make. `intake.rs` already does this correctly
(`:386-397`), so the pattern is propagated, not invented.
**Verified by** `SB-DIO-T77`, `SB-DIO-T78`, `SB-DIO-T79`.

---

**`SB-DIO-055` — An export that omits data MUST say what it omitted.** · **P0** ·
`PRESENT-DIVERGENT`

> An export **MUST** either write every curve held for the well, or state — in the user-visible
> result and in the file's own provenance section — exactly which curves it omitted and why.

**Rationale.** `export.rs:24` writes six standard curves plus computed ones and omits the **entire
generic curve store** with no note. A user who imported forty curves and exported "the LAS" receives
eight, and discovers it at the client's desk. P0 because it reaches the client directly and because
the discovery happens outside the user's control.
**Verified by** `SB-DIO-T80`, `SB-DIO-T81`.

---

**`SB-DIO-056` — A declared `STEP` MUST be verified across the whole index.** · **P1** ·
`PRESENT-DIVERGENT`

> The writer **MUST** compute the step over every adjacent pair and **MUST** write `STEP` as `0`
> when the interval is not constant within the stated tolerance. It **MUST NOT** declare the first
> interval as the step.

**Rationale.** `export.rs:60` takes `depth[1] - depth[0]`. On a merged multi-run well, a
depth-shifted composite or a resampled join the first interval is not the step, and a conforming
reader is entitled to reconstruct depths from `STRT`/`STEP` rather than from the `DEPT` column —
silently re-gridding the data, which is `SB-DIO-022`'s failure arriving by a different door. LAS 2.0
provides `STEP = 0` for exactly this case, so correctness costs nothing.
**Verified by** `SB-DIO-T82`, `SB-DIO-T83`.

---

**`SB-DIO-057` — A zero on a log-scale curve MUST NOT be committed as a reading.** · **P1** ·
`ABSENT`

> On a curve whose family is logarithmic — gas totals and components, resistivity, permeability —
> exact zeros **MUST** be counted and surfaced for confirmation before commit. They **MUST NOT** be
> rewritten automatically.

**Rationale.** `P`-tier, memory `reference_mudlog_gas_curve_traps`, from delivered mudlog rebuilds: a
log-scale curve cannot represent zero, so a zero in a gas or resistivity column is an encoding of "no
reading" by an exporter that had no null. Committed as a reading it pulls every log-domain average
toward a value that was never measured, and the effect survives into any mean, any crossplot and any
cutoff. Deliberately *surface, not rewrite* — a genuine zero exists in some engineered curves, and
`SB-DIO-004` puts value rewriting in `ENV` regardless.
**Verified by** `SB-DIO-T84`, `SB-DIO-T85`.

---

**`SB-DIO-058` — Old `.xls` plate workbooks MUST be read from the published specification.** ·
**P2** · `PARTIAL`

> SandiBumi **MUST** read `.xls` workbooks — cells and embedded pictures with their worksheet
> association — implemented from Microsoft's published `[MS-XLS]` Open Specification. A plate whose
> worksheet cannot be resolved from the drawing anchors **MUST** be dropped and counted by name, and
> **MUST NOT** be given a guessed depth.

**Rationale.** §3.10 takes the position in full. The shipped refusal (`images.rs:832-850`) is correct
engineering — what it refuses is not reading the file but *inventing the plate-to-depth association*,
and a plate hung off the wrong sand is a wrong geological conclusion with a photograph attached,
which is more persuasive and therefore worse than no plate. But at **107 of 165 workbooks — 65 %** it
is the majority path, and a majority path ending in a manual workaround is a capability gap wearing a
good error message. Three facts settle it toward closing: the blocker is a **published** specification
rather than a proprietary one, so implementing it is not reverse-engineering; the approach is proven
on delivered work (`P`, memory `reference_wellsite_xls_biff_recovery`, packaged as the
`mudlog-data-to-las` skill); and the all-or-nothing association rule survives the change unchanged,
matching what `images.rs` already does for sub-`MIN_PLATE_PX` decorations. Until it ships the refusal
stands exactly as written — it is the right behaviour for a capability that does not exist yet.
**Verified by** `SB-DIO-T86`, `SB-DIO-T87`.

---

**`SB-DIO-059` — Tabular `.xls` MUST be readable without the drawing layer.** · **P2** · `ABSENT`

> A `.xls` workbook containing only cell data **MUST** be readable using the cell records alone.

**Rationale.** A petrography point-count table, a core-analysis sheet or a wellsite export with no
embedded pictures needs only the BIFF cell records, not the drawing records — a strictly smaller
problem than `SB-DIO-058`. Splitting it out stops the common case being held hostage to the harder
one, and it is the case Jauhar's existing BIFF decoder already covers.
**Verified by** `SB-DIO-T88`.

---

**`SB-DIO-060` — Format MUST be recognised by signature, and signature collisions MUST be
handled.** · **P1** · `PARTIAL`

> Format detection **MUST** use content signatures. A file whose extension and signature disagree
> **MUST** be handled by its signature, with the disagreement reported. Where two formats share a
> signature, the reader **MUST** disambiguate on structure and **MUST** report which it chose.

**Rationale.** D-31 catalogues signature-based recognition and the collision that must be handled. The
concrete local case is a wellsite `.xls` that is a **BIFF2 or BIFF5 stream rather than an Excel
workbook** — the extension is honest about the family and wrong about the version, and an
extension-driven reader fails with a message that sends the user to the wrong problem (`P`, memory
`reference_wellsite_xls_biff_recovery`). `images.rs:832-850` filters by extension only. The same rule
catches a `.las` that is a delimited export and a `.dlis` that is a LIS file.
**Verified by** `SB-DIO-T89`, `SB-DIO-T90`.

---

**`SB-DIO-061` — Malformed input MUST be located, counted, named, and regression-tested against a
corpus.** · **P0** · `PARTIAL`

> For every reader in this domain: a malformed input **MUST** produce a diagnostic naming the
> **file, the line or record, and the rule that failed**; the count of affected items **MUST** be
> reported; no malformed item may be silently dropped or silently coerced; and no malformed input
> may panic, hang, or consume unbounded memory. A **corpus of malformed fixtures MUST be maintained
> in-repo**, and every reader **MUST** be run against the whole corpus in CI.

**Rationale.** This is the requirement that makes "solved" testable for the defect family that has
recurred three or more times. Each recurrence was a different reader failing the same way, which is
the signature of a missing shared contract rather than a repeated mistake. The four clauses are
chosen because each maps to a real recurrence: *location* (a "failed to parse" with no line number),
*count* (a partial import reported as complete), *no silent coercion* (`parsers.rs:311-317` gets this
right; other paths do not), and *no panic* (a reader that panics on a truncated file takes the
application with it). The corpus clause is what stops recurrence four: fixtures accumulate, and a
reader added later inherits every case its predecessors failed. `intake.rs:115` `preview_bad` is the
reference implementation of the location clause — it carries `(row, column)` for every unparseable
cell, and its tests at `:1599-1600` pin the two real cases, a depth with a unit stuck on it and a
spreadsheet `#N/A`.
**Verified by** `SB-DIO-T91`, `SB-DIO-T92`, `SB-DIO-T93`, `SB-DIO-T94`.

---

**`SB-DIO-062` — Text encoding MUST be detected, not assumed.** · **P1** · `PARTIAL`

> A text reader **MUST** detect UTF-8, UTF-16 (both byte orders, with and without BOM) and the
> Windows single-byte code pages, and **MUST** report the encoding it chose.

**Rationale.** `images.rs:811-812` already states the principle for its own runner — bytes must be
interpreted rather than assumed — and names `parsers::read_text_file` as the same family. A UTF-16
LAS from a Windows tool read as UTF-8 appears to have no sections at all, which presents as "this is
not a LAS file" and sends the user to the wrong problem.
**Verified by** `SB-DIO-T95`.

---

**`SB-DIO-063` — Non-ASCII paths and payloads MUST survive every sidecar boundary.** · **P1** ·
`PRESENT-OK`

> Every Python sidecar **MUST** exchange paths and payloads as bytes, never as platform-encoded
> text.

**Rationale.** `record_petrography.md:622-624` records that the workbook runner reads
`sys.stdin.buffer` and never `sys.stdin`, because a workbook path with any non-ASCII character would
otherwise arrive as mojibake and fail naming a path nobody has. Indonesian client and field names
make this an everyday case, not an edge one. The requirement locks the behaviour across `dlis.rs`,
`office.rs` and `images.rs` rather than leaving it as one module's good habit.
**Verified by** `SB-DIO-T96`.

---

## 5. Parameters

In this domain "parameter" includes format constants, null conventions, tolerances, alias orders and
resolution thresholds. They are held to the same discipline as a petrophysical parameter: **cited to
a named source, or recorded `ABSENT`.** Nothing here is inferred, and nothing is rounded to look
tidier than its source.

Two markers are used. **`NON-ADOPTABLE`** means the value is cited so a reviewer can verify a claim
about an incumbent or about SandiBumi's own defect — it is *not* a value SandiBumi adopts.
**`ABSENT — ships with no default`** means the evidence does not adjudicate and SandiBumi asks
rather than choosing.

### 5.1 Depth and units

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Metres per international foot | `M_PER_FT` | 0.3048 | m/ft | NIST SP 811, as cited in `units.rs:34` | T4 |
| US survey foot excess over international | — | 2 | ppm | NIST SP 811; **deliberately not modelled** (6 mm at 3,000 m) | T4 |
| Feet-per-metre index error if unit is assumed wrong | — | 3.2808 | ratio | `1 / 0.3048`, derived here | T4 |
| Accepted metre spellings | — | `M`, `MT`, `METER`, `METERS`, `METRE`, `METRES` | — | `units.rs:81` | T1 |
| Accepted foot spellings | — | `F`, `FT`, `FOOT`, `FEET`, `'` | — | `units.rs:81` | T1 |
| Default when no unit is declared | — | Metres | — | `units.rs:220` — **`NON-ADOPTABLE`**, this is the `SB-DIO-015` defect | T1 |
| TVD-read-as-MD depth deficit | — | `1 − cos θ̄` | fraction of MD traversed | dossier §3.2, derived | T4 |
| — entire hole at 30° | — | 13.40 % → 402 m on 3,000 m MD | m | `1 − cos 30° = 1 − 0.8660`, executed | T4 |
| — 40 % tangent at 30° (a typical deltaic-clastic development well) | — | 5.36 % → 161 m | m | `0.40 × 13.40 %`, executed | T4 |
| — bottom third at 30° | — | 4.47 % → 134 m | m | `0.333 × 13.40 %`, executed | T4 |
| °F → °C offset | — | 32 | °F | dossier D-13; affine, `SB-DIO-026` | T1 |
| SandiBumi canonical curve families | — | 14 | count | `curves.rs:21-37` | T1 |
| SandiBumi families carrying a `DEPTH` entry | — | 0 | count | `curves.rs:21-37` — **deliberate**, §3.1 | T1 |
| IP numeric unit-conversion families | — | 4 (SONIC, DENSITY, CALIPER, POROSITY) in 63 lines | count | dossier D-9 | T3 |
| IP families claimed by that file's own header | — | 3 (omits POROSITY) | count | dossier D-9 — the shipped file is wrong about itself | T3 |
| `OpenSpiritUnits.opt` conversion factors | — | 0, in 1,132 records / 5,662 lines | count | dossier D-9 — a vocabulary, not a table | T3 |
| `MS/FT` interpretation | — | **`ABSENT — ships with no default`** | — | dossier D-12; unadjudicable two-way ambiguity, `SB-DIO-029` | T3 |
| `PPG` as a density alias | — | **`NON-ADOPTABLE`** — it is a pressure gradient | — | dossier D-14, `density.units` | T1 |
| Geolog unit-conversion default | `PG_UNIT_CONVERT` | `YES` | — | dossier D-10 — **`NON-ADOPTABLE`**, `SB-DIO-024` | T1 |
| Geolog `MEQ/L` conversion factor, as shipped | — | 1.0 | — | dossier G-D-1 — **`NON-ADOPTABLE`**, a 1000× defect | T1 |
| — corrected value | — | 1.0E-3 | meq/mL per meq/L | Derivation shown: 1 L = 10³ mL and Waxman-Smits Qv is expressed in **meq/mL**, so meq/L → meq/mL is ×10⁻³. Corroborated independently by the meq/mL-not-meq/L trap in memory `reference_waxman_smits_b` | T1 + P |

The last two rows close dossier **G-D-1** rather than deferring it. D-11 asserts a defect in a shipped
Geolog table and requires the corrected factor to appear **with its derivation shown**; the derivation
is a unit-prefix identity, not a petrophysical choice, so it can be executed here and checked by a
reviewer in one line. What does **not** follow from it is any inference about a curve already labelled
`meq/L` — see §7.1 O-2.

### 5.2 Absent-value conventions

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| CWLS-conventional LAS null | `NULL_VALUE` | −999.25 | — | LAS 2.0 convention; `parsers.rs:130`, `export.rs:8` | T1 |
| Second null SandiBumi recognises | — | −9999 | — | `parsers.rs:130` (as-built) | T1 |
| Null-comparison tolerance, relative | — | 1e-5 | relative | `parsers.rs:138-140` (as-built) — the form to keep | T1 |
| Null-comparison floor | — | 1.0 | — | `parsers.rs:138-140` (as-built) | T1 |
| Null-comparison tolerance, absolute | — | `f32::EPSILON` = 1.19e-7 | — | `parsers.rs:133` — **`NON-ADOPTABLE`**, S-D-2 / `SB-DIO-004` | T1 |
| `f32` spacing at −999.25 | — | ≈6.1e-5 | — | derived: `2^(exp−23)` at magnitude 999.25 | T4 |
| Ratio, actual spacing to tested tolerance | — | ≈512× | — | derived from the two rows above | T4 |
| DLIS magnitude clamp | — | 1e30 | absolute | `dlis.rs:183` — as-built, **no external source**; §7.1 O-3 | T1 |
| RP66 defined absent value | — | **none exists** | — | dossier D-5 (T1/T2/T3 concur) | T1 |
| IP ASCII / LAS / DBASE4 export null | — | −999 | — | `Intpetro.config` `LasNullValue = -999` | T3 |
| IP LIS / DLIS export null | — | −999.25, not user-settable | — | dossier §2.1, §3.1 | T2 |
| IP curve-data writers, and distinct null values | — | 5 writers, 2 values | count | dossier D-1; byte-stable 2018 → 2025 | T2 |
| Geolog `_missing_value` | — | −999.25 `DOUBLE` | — | `log_export.tclsh:19` | T1 |
| Geolog writers receiving it | — | **8 of 12** | count | `log_export.tclsh:64-74` | T1 |
| Geolog writers receiving nothing | — | 4 — `amocoa`, `segy`, `rms`, **`unl`** | — | dossier D-2; `unl` is the shipped default | T1 |
| Techlog null-exception channels / name patterns | — | 16 `<Channel>` / 21 `<Name>` | count | `Settings/DLIS/DlisNullValuesExceptions.xml` | T3 |
| — patterns declaring **no null at all** | — | 16 of 21 | count | same; all array or waveform channels | T3 |
| — populated patterns, and distinct values | — | 5 patterns, 3 values | count | same | T3 |
| — Baker waveform null | — | −32767 | — | same | T3 |
| — SonicVision `WF[1-5][ITR]` null | — | −999 | — | same | T3 |
| Error from a global −999.25 screen on that set | — | wrong on **18 of 21** patterns | count | dossier D-3, derived from the rows above | T3 |
| Techlog channels packed in one `<Channel>` (Weatherford CXD) | — | 6 `<Name>`/`<NullValues/>` pairs | count | dossier D-4 / T-D-1 | T3 |
| — vendor patterns a naive parser drops | — | 5 | count | dossier D-4, derived | T3 |
| Surviving-null error, worked example | — | 33.5 gAPI reported where truth is 60 | gAPI | `(39×60 + (−999))/40`, executed; **44 % error** | T4 |

### 5.3 Name resolution and alias orders

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| LAS index aliases | `DEPTH_ALIASES` | `DEPT`, `DEPTH` | — | `parsers.rs:168`; positional guard at `:162-167` | T1 |
| Core-table depth aliases | `CORE_DEPTH_ALIASES` | `DEPTH`, `DEPT`, `MD` | — | `parsers.rs:642` | T1 |
| Tops depth aliases | `TOPS_DEPTH_ALIASES` | `DEPTH`, `MD`, `TOP_MD`, `MD_TOP`, `TOP_DEPTH`, `DEPT`, `TVD` | — | `parsers.rs:1607`; the `TVD` member is **`NON-ADOPTABLE`** as an MD alias, `SB-DIO-014` | T1 |
| Tops name aliases | `TOPS_NAME_ALIASES` | 9 entries | count | `parsers.rs:1605-1606` | T1 |
| Deep-resistivity alias priority | `RES_ALIASES` | `RES_DEEP`, `RESD`, `RT`, `RES`, `DRES`, `ILD`, `LLD`, `AT90` | — | `parsers.rs:171` — a **petrophysical** ordering | T1 |
| Gamma-ray aliases | `GR_ALIASES` | `GR`, `GRN` | — | `parsers.rs:169` | T1 |
| Neutron alias precedence rule | — | thermal (CNL-family) leads; epithermal, `SNP`, `APLC`/`FPLC` follow | — | `parsers.rs:172-173` | T1 |
| Alias tie-break | — | strictly-greater coverage, else alias priority | — | `parsers.rs:323-334` | T1 |
| Real Geolog reference aliases | — | `SCD`, `IDWD`, `DVP1`, `PDEP_XPT`, `DEPM`, `TDEP` | — | `alias.alias:14`, under `# aliases for references` (`:13`) | T1 |
| Real Geolog `TVD` namespace | — | `# aliases for welltie`, `TVD = TVD_SS TVD_KB` | — | `alias.alias:891` — a **different** declared namespace | T1 |
| IP LAS 3.0 depth tags | — | `DEPT`, `DEPTH`, `MD`, `DateTime`, `Time`, `TDEP`, `DPTH` | — | `Intpetro.config` | T3 |
| IP Geolog-ASCII depth list | — | `DEPTH`, `TDEP`, `DEPT`, `MD`, `INDEX`, `TVD` | — | `GeologASCII_options.txt` | T3 |
| IP `DefaultAlias.cax` CurveType 1 = Depth | — | **0 alias rows** | count | dossier D-6 — a third, empty list | T3 |
| Real Geolog depth aliases IP misses | — | 5 of 6 | count | dossier D-7, derived | T1 |
| Geolog structural index declaration | — | `CLASSES = REFERENCE \| LOG`, per column | — | Geolog flat-ASCII format specs | T1 |
| Geolog LAS index fallback | — | `_ref_in = DEPTH` | — | `log_load_dxs_las.tclsh:97` | T1 |
| Techlog ASCII reference constraint | — | mandatory user designation, strictly increasing | — | `import-asciidata.html` | T3 |
| SegaraBumi mnemonic super-dictionary | — | 7,165 rows / 2,751 standards | count | memory `project_segarabumi_p1_dictionary` — **not adopted**, §1.2 | P |

### 5.4 Reader and writer constants (as-built)

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Intake preview depth | `PREVIEW_ROWS` | 200 | rows | `intake.rs:53` | T1 |
| Intake type-sniff depth | `SNIFF_ROWS` | 400 | rows | `intake.rs:58` | T1 |
| Array preview depth | `ARRAY_PREVIEW_ROWS` | 40 | rows | `intake.rs:979`, rationale `:974-978` | T1 |
| Decimal-separator rule | — | rightmost separator is the decimal mark | — | `intake.rs:134` `parse_number` | T1 |
| Declared table layouts | — | LONG, WIDE, BLOCK — **declared, never sniffed** | — | `intake.rs:472-493` | T1 |
| Minimum plate dimension | `MIN_PLATE_PX` | 400 | px | `images.rs:479`; test `:1439` asserts it is round, not tuned | T1 |
| Workbook header rows scanned | `WORKBOOK_HEADER_ROWS` | 14 | rows | `images.rs:483` | T1 |
| LAS export null | `NULL_VALUE` | −999.25 | — | `export.rs:8`, declared in `~W` at `:80` | T1 |
| LAS export declared depth unit | — | `.M`, hardcoded | — | `export.rs:77-79`, `:85` — **`NON-ADOPTABLE`**, `SB-DIO-017` | T1 |
| LAS export `STEP` | — | `depth[1] − depth[0]` | — | `export.rs:60` — **`NON-ADOPTABLE`**, `SB-DIO-056` | T1 |
| LAS export curve scope | — | 6 standard + computed only | count | `export.rs:24` — **`NON-ADOPTABLE`**, `SB-DIO-055` | T1 |
| Sample storage precision | — | `f32` | — | `parsers.rs`, `dlis.rs`, `export.rs` throughout | T1 |
| IP DLIS write precision | — | `FSINGL` unconditionally | — | dossier D-29 | T2 |
| Python discovery order | — | `SANDIBUMI_PYTHON` → `ARSHILLA_PYTHON` → `%LOCALAPPDATA%\Programs\Python\Python313\|312\|311\|310` → `PATH` | — | `python_engine.rs:41`, `:45`, `:177`, `:193-194` | T1 |
| Minimum Python | — | 3.10+ with numpy | — | `python_engine.rs:48` | T1 |

### 5.5 The `.xls` decision — the numbers behind §3.10

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Petrography workbooks on the reference machine | — | 165 | count | `01_PRODUCT.md` §4.1; `record_petrography.md:613` | P |
| — in the old `.xls` format | — | **107 (64.8 %)** | count | same | P |
| Accepted workbook extensions | — | `.xlsx`, `.xlsm` | — | `images.rs:832-850` | T1 |
| Plates recovered on two real deliveries | — | 152, every one with a depth from its sheet | count | `record_petrography.md:631-633` | P |
| Notes raised on those deliveries | — | 33 | count | same | P |
| Decorations correctly dropped | — | 117×59 and 207×79 against plates at 1920×1080 | px | `record_petrography.md:609` | P |
| Depth-parse failure rate on a real book | — | 1 sheet in 129 (`7033,50/354 FT (CORE)`) | count | `images.rs:815-819` — deliberately not patched around | T1 |
| BIFF versions seen in wellsite exports | — | BIFF2, BIFF5 | — | memory `reference_wellsite_xls_biff_recovery` | P |

### 5.6 Parameters deliberately `ABSENT` from this chapter

**Physical range bounds per curve family** — required by `SB-DIO-023` and **not defined here.** The
range table is `20_envcorr-qc.md`'s (`ENV`), which owns judgements about whether a measurement is
physical; `DIO` consumes that table and owns only the obligation to run the check at import and to
block on failure. Defining a second copy here would be an `SB-CORE-007` violation of exactly the kind
§3.8 records between `curves.rs` and `export.rs`. `SB-DIO-023` is therefore blocked on `ENV`
publishing the table, and that dependency is recorded at §7.1 O-4.

**Log-scale family membership** — required by `SB-DIO-057`. Which families are logarithmic is a
petrophysical classification, and it belongs with the family table in `curves.rs` extended under
`ENV`'s review, not asserted here. §7.1 O-5.

**Vendor chart and lookup-table data** — none is transcribed anywhere in this chapter, per
`CONTRACT.md` §2.1. The Matthews & Kelly exception is not invoked and no second case arose.

---

## 6. Acceptance tests

Ninety-six tests, one or more per requirement. **Kind** is `new` (behaviour to build),
`char` (**characterization** — pins behaviour that already ships, so a refactor cannot quietly
change it), or `mal` (**malformed-input**, in force under `SB-DIO-061` and run against the shared
corpus). Fixtures are synthetic unless a source is named; **no client well name appears in any
fixture**, per `CONTRACT.md` §2.3.

Seven shipped tests are adopted as characterization tests rather than rewritten:
`export::export_writes_missing_as_null_and_carries_mixed_case_computed_curves`,
`export::an_exported_las_reimports_with_the_same_values`,
`images::the_old_workbook_format_is_refused_by_name_with_the_fix`,
`images::the_newer_workbook_formats_are_accepted`, the `MIN_PLATE_PX` bound assertions
(`images.rs:1439-1441`), the decoration guard (`images.rs:1854`), and the intake bad-cell
assertions (`intake.rs:1599-1600`).

### 6.1 Absent values and null conventions

| Test | Input | Operation | Expected | Kind |
|---|---|---|---|---|
| `SB-DIO-T01` | A project with the sentinel set to a non-default value | Export through every registered writer | Every output file declares that sentinel; no writer emits `-999.25` | new |
| `SB-DIO-T02` | A newly registered writer that ignores the sentinel argument | Build | Compile-time failure — the sentinel is a required argument, not an option | new |
| `SB-DIO-T03` | Default project, default format | Export | The default format is one that honours the sentinel | new |
| `SB-DIO-T04` | A channel declared "no null", carrying a genuine −999.25 amplitude | Import | The −999.25 survives as a value; no absent marker is written | new |
| `SB-DIO-T05` | The same channel with no declaration at all | Import | Screened normally, and the difference between "no null" and "unset" is visible in the result | new |
| `SB-DIO-T06` | A LAS column containing `-999.2500001` | Import | Recognised as absent | new |
| `SB-DIO-T07` | The same value after one `f32`→`f64`→`f32` round trip | `is_las_null` | Recognised as absent | new |
| `SB-DIO-T08` | Source tree | Static check | Exactly one null-comparison function exists; `parsers.rs:132-134` is gone | new |
| `SB-DIO-T09` | A file with two channels declaring different nulls (`-999` and `-32767`) | Import | Each channel screened against its own value only | new |
| `SB-DIO-T10` | A null-exception rule with six name patterns in one entry | Load the rule set | All six patterns are active; none dropped | new |
| `SB-DIO-T11` | A delimited file with `a,,b` and `a,-999.25,b` on consecutive rows | Import, then export | Both are absent in arithmetic, and the export distinguishes them | new |

### 6.2 Alias resolution

| Test | Input | Operation | Expected | Kind |
|---|---|---|---|---|
| `SB-DIO-T12` | A LAS with an all-null `NPHIED` and a populated `NPHI_LS` | Import | The neutron slot binds `NPHI_LS` | **char** |
| `SB-DIO-T13` | Two candidate columns of exactly equal coverage | Import twice, on two machines | Identical binding both times, resolved by alias priority | **char** |
| `SB-DIO-T14` | The `SB-DIO-T12` fixture | Import | The result names `NPHI_LS` bound, `NPHIED` passed over, and both coverage counts | new |

### 6.3 Index detection

| Test | Input | Operation | Expected | Kind |
|---|---|---|---|---|
| `SB-DIO-T15` | A Geolog flat-ASCII file declaring `CLASSES = REFERENCE` on a non-first column | Import | The declared column is the index; the result records "structural declaration" | new |
| `SB-DIO-T16` | A LAS whose second column is named `MD` | Import | Column 0 is the index; the result records "positional guarantee" | **char** |
| `SB-DIO-T17` | Source tree | Static check | Every index alias list cites its source in a comment; no undocumented list exists | new |
| `SB-DIO-T18` | A LAS whose index decreases at row 400 | Import | Blocked with the row number reported; not silently sorted or accepted | **mal** |
| `SB-DIO-T19` | A delimited file with no recognisable depth column | Import | The user is required to designate one; no column is chosen by position | new |
| `SB-DIO-T20` | A tops table carrying only a `TVD` column | Import | Committed as TVD-referenced; the alias still resolves | new |
| `SB-DIO-T21` | Those tops, with no deviation survey present | Plot or join against an MD log | Refused, naming the missing survey | new |

### 6.4 The depth unit

| Test | Input | Operation | Expected | Kind |
|---|---|---|---|---|
| `SB-DIO-T22` | A LAS with no index unit, into a project with no unit set | Import | **Refused**, naming both places a unit could have come from | new |
| `SB-DIO-T23` | A LAS with no index unit, into a project set to metres | Import | Explicit confirmation of the *file's* unit required before commit | new |
| `SB-DIO-T24` | A LAS declaring `FT`, into a project set to metres | Import | Converted, and the conversion reported | **char** |
| `SB-DIO-T25` | A DLIS whose index channel declares `FT`, into a metre project | Import | Depths converted by 0.3048; well depth range in metres | new |
| `SB-DIO-T26` | A DLIS whose index channel declares no unit | Import | Refused under `SB-DIO-015`'s rule | new |
| `SB-DIO-T27` | A project in **feet**, one well | Export LAS, re-import into a fresh project | Depths identical to the original; `STRT`/`STOP`/`STEP`/`DEPT` all declare `FT` | new |
| `SB-DIO-T28` | The same, project in **metres** | Export, re-import | Depths identical; units declare `M` | **char** |
| `SB-DIO-T29` | Source tree | Static check | `export.rs::standard_units` does not exist; `export.rs` calls `curves::canonical_unit` | new |
| `SB-DIO-T30` | Every family in `curves.rs::FAMILIES` | Export one curve per family | Each written unit equals the family's canonical unit exactly, including case | new |
| `SB-DIO-T31` | A project in metres with committed curves | Change the project unit to feet | Either refused, or an explicit migration stating the well and curve counts | new |

### 6.5 Sampling, units and conversion

| Test | Input | Operation | Expected | Kind |
|---|---|---|---|---|
| `SB-DIO-T32` | A LAS with three repeated depths | Import with policy "keep-first" | Three rows dropped, count reported, first sample kept | new |
| `SB-DIO-T33` | The same file, no policy declared | Import | The user is asked; nothing commits until answered | new |
| `SB-DIO-T34` | Any file at 0.1 m sampling | Import at defaults | Sample interval unchanged; no resampling occurred | **char** |
| `SB-DIO-T35` | A well with an irregular index | Export at defaults | Written samples equal stored samples exactly; no re-grid | new |
| `SB-DIO-T36` | A file whose `GR` column holds values 0.02–0.40 | Import | Blocked before commit, naming the family and the observed range | new |
| `SB-DIO-T37` | A file whose column labels all match but whose data is shifted one column left | Import | Blocked on at least one family's physical range | **mal** |
| `SB-DIO-T38` | A resistivity column containing negatives | Import | Blocked, naming the family | new |
| `SB-DIO-T39` | A LAS declaring `US/M` on a sonic, into a canonical-`US/F` project | Import | Converted **and reported**, with from-unit, to-unit and factor | new |
| `SB-DIO-T40` | A curve declaring a unit in no known family | Import | Stored, flagged unconverted; not silently treated as canonical | new |
| `SB-DIO-T41` | The unit system | Query | Returns the exact set of convertible families | new |
| `SB-DIO-T42` | A temperature curve declaring `DEGF`, canonical `DEGC` | Import | 200 °F → 93.33 °C, not 111.1 °C | new |
| `SB-DIO-T43` | A column declaring `PPG` | Import | Not bound to the density family; flagged for designation | new |
| `SB-DIO-T44` | The unit table | Static check | Every factor carries a derivation string; no factor is sourced only to a vendor file | new |
| `SB-DIO-T45` | A curve declaring `MS/FT` | Import | The user is asked which quantity; no default is applied | new |

### 6.6 Curve identity and substitution

| Test | Input | Operation | Expected | Kind |
|---|---|---|---|---|
| `SB-DIO-T46` | A file whose `SGR` is renamed to `GR` by an alias table | Import | Both names visible on the curve; the firing table entry named | new |
| `SB-DIO-T47` | A request for a curve the well does not hold | Any operation, any configuration | Reported unavailable; no other curve's data returned under that name | new |
| `SB-DIO-T48` | A user accepting a named substitute | Commit | The substitution recorded on the curve as provenance | new |
| `SB-DIO-T49` | A saved curve selection | Reload, inspect | The selection is a named object listing its members | new |
| `SB-DIO-T50` | A well with two curves of the same curve type | Any read operation | Neither is auto-selected; the choice is stated or asked | new |

### 6.7 DLIS

| Test | Input | Operation | Expected | Kind |
|---|---|---|---|---|
| `SB-DIO-T51` | A DLIS with curves outside an existing well's declared range | Import | The conflict is raised; the well's range is unchanged unless confirmed | new |
| `SB-DIO-T52` | A DLIS carrying a mnemonic the well already holds | Import | The user is asked; the default is not merge-into-existing | new |
| `SB-DIO-T53` | A DLIS with one encrypted and one readable channel | Import | Result is "partial", the encrypted channel named | new |
| `SB-DIO-T54` | A DLIS carrying a 2-D image channel | Import | The array imports with its shape, per-axis labels and units preserved | new |
| `SB-DIO-T55` | A DLIS whose entire payload is array channels | Import | Non-zero curves imported; **not** an empty success | new |
| `SB-DIO-T56` | A DLIS channel legitimately containing −999.25 | Import with that channel excepted | The value survives | new |
| `SB-DIO-T57` | The same file without the exception | Import | The value is screened **and the count is reported per channel** | new |

### 6.8 LAS structure and versions

| Test | Input | Operation | Expected | Kind |
|---|---|---|---|---|
| `SB-DIO-T58` | A `WRAP: YES` LAS with 30 curves | Import | Every sample lands in the right curve | **char** |
| `SB-DIO-T59` | A LAS 3.0 file with `~Core_Data` and `~Tops` sections | Import | Recognised as 3.0; every unread section named in the result | new |
| `SB-DIO-T60` | The same file | Import | Associated sections parsed into core and tops tables | new |
| `SB-DIO-T61` | A LAS 1.2 file | Import, then attempt export as 1.2 | Import succeeds; 1.2 is not offered as an export format | new |
| `SB-DIO-T62` | A LAS with an unknown `~X` section and an out-of-order `~W` | Import | Handled per the documented strictness, and the handling reported | **mal** |

### 6.9 Containers, headers, precision and provenance

| Test | Input | Operation | Expected | Kind |
|---|---|---|---|---|
| `SB-DIO-T63` | A DLIS holding three logical files for three wells | Import | Three project wells; none merged | new |
| `SB-DIO-T64` | The same file | Pre-commit preview | The container-to-well mapping is shown before anything commits | new |
| `SB-DIO-T65` | No Python on `PATH`, no environment variable | Attempt DLIS import, then `.xlsx` export | Each refused separately, naming the library and the fix | **char** |
| `SB-DIO-T66` | A float64 core-analysis table | Import, export | The precision reduction is stated in the result | new |
| `SB-DIO-T67` | A LAS whose `~W WELL` disagrees with its filename | Import | The header value is used; the filename is offered only as a confirmable default | new |
| `SB-DIO-T68` | Every writer | Write, then read back with SandiBumi's own reader | Round trip succeeds; failure is an error, not a warning | new |
| `SB-DIO-T69` | A well whose depths are in feet | Export LAS | The self-check catches a mis-declared unit before success is reported | new |
| `SB-DIO-T70` | A LAS whose `~W STEP` disagrees with its actual spacing | Import | Flagged as possibly re-gridded | **mal** |
| `SB-DIO-T71` | A well with a measured `GR` and a computed `VSH` | Export LAS | `~O` names `VSH` computed, its method, and every parameter with its value | new |
| `SB-DIO-T72` | A well carrying a model-derived curve | Export | The `SB-CORE-014` record appears in the file | new |
| `SB-DIO-T73` | A well with only measured curves | Export | `~O` still states that every curve is measured | new |
| `SB-DIO-T74` | A well holding both a working and a final `PHIE` | Export both | The file marks which is final | new |
| `SB-DIO-T75` | A LAS with an unmapped `~W` header mnemonic | Import | Carried verbatim; not dropped | new |
| `SB-DIO-T76` | A file with no UWI | Import | No UWI is synthesised from the filename or anything else | new |

### 6.10 Robustness — the malformed-input corpus

Every test in this block runs against **every reader in the domain**, not only the one it was
written for. That is the clause that stops recurrence four.

| Test | Input | Operation | Expected | Kind |
|---|---|---|---|---|
| `SB-DIO-T77` | A DLIS with one unreadable frame and one good frame | Import | The good frame imports; the bad one is named and counted | **mal** |
| `SB-DIO-T78` | A DLIS where every frame fails | Import | An **error**, not an empty success | **mal** |
| `SB-DIO-T79` | A LAS with three rows shorter than `~C` declares | Import | Hard error naming the first offending line | **char** |
| `SB-DIO-T80` | A well with 40 imported curves | Export LAS | Either all 40 written, or the omitted 32 named in the result and in `~O` | new |
| `SB-DIO-T81` | The same | Export | The user-visible result states the count written and the count held | new |
| `SB-DIO-T82` | A well with a uniform 0.1524 m index | Export | `STEP` = 0.1524 | **char** |
| `SB-DIO-T83` | A merged well whose index steps 0.1 m then 0.15 m | Export | `STEP` = 0 | new |
| `SB-DIO-T84` | A gas curve with 200 exact zeros among 4,000 samples | Import | 200 surfaced for confirmation; none rewritten automatically | new |
| `SB-DIO-T85` | The user declining the conversion | Commit | The zeros commit as values, and the decision is recorded | new |
| `SB-DIO-T86` | A `.xls` workbook with plates on three sheets | Import | Each plate carries the depth from its own sheet | new |
| `SB-DIO-T87` | A `.xls` plate whose anchor does not resolve to a sheet | Import | Dropped **and named**; no depth guessed | new |
| `SB-DIO-T88` | A `.xls` with cell data and no pictures | Import | Table read without the drawing layer | new |
| `SB-DIO-T89` | A BIFF5 stream named `.xls` | Open | Read by signature; the version disagreement reported | new |
| `SB-DIO-T90` | A delimited text file named `.las` | Open | Read as delimited; the disagreement reported | **mal** |
| `SB-DIO-T91` | Every corpus fixture | Run through every reader | No panic, no hang, no unbounded allocation | **mal** |
| `SB-DIO-T92` | Every corpus fixture that fails | Read | The diagnostic names file, line-or-record, and the failed rule | **mal** |
| `SB-DIO-T93` | A file truncated mid-record at 100 byte offsets | Read each | Every one fails cleanly with a located diagnostic | **mal** |
| `SB-DIO-T94` | The corpus | CI | Every reader runs the whole corpus; adding a reader without wiring it fails the build | **mal** |
| `SB-DIO-T95` | A UTF-16LE LAS with a BOM, and one without | Import | Both read; the detected encoding reported | **mal** |
| `SB-DIO-T96` | A path and a well name containing non-ASCII characters | Import via each sidecar | Both survive unchanged through `dlis.rs`, `office.rs` and `images.rs` | **char** |

**Counts.** 96 tests: **11 characterization**, **12 malformed-input**, 73 new. Every P0 requirement
carries at least two tests, and `SB-DIO-061` carries four because it is the contract the other
malformed-input tests are instances of.

---

## 7. Open items, escalations, refusals and derivations

Four parts, per `CONTRACT.md` §2.2.1. §7.3 and §7.4 are kept apart deliberately: a **defect
refusal** is SandiBumi declining to reproduce a vendor's broken behaviour and is a *win*; an
**independent-derivation requirement** is a capability SandiBumi owes the user and must build from
primary sources. Merging them would let a debt read as an achievement.

### 7.1 Open items

**O-1 — What happens to committed depths when the project unit changes is unestablished.**
`units.rs:169` writes a settings row and `ingest.rs:284-288` adopts a unit post-commit, but this
chapter did not establish what the change does to depths already stored. `SB-DIO-019` is written to
be discharged by a test that finds out. **Blocks nothing; must be answered before `SB-DIO-019` can
be marked done.** `22_database-model.md` owns the migration mechanics.

**O-2 — G-D-1 is closed in this chapter; its downstream reach is not.** Geolog ships `MEQ/L` with a
conversion factor of 1.0 where 1.0E-3 is required — a **1000× defect** in a shipped vendor table.
§5.1 now carries the corrected value with its derivation shown (1 L = 10³ mL; Waxman-Smits Qv is in
meq/mL), corroborated independently by the meq/mL-not-meq/L trap recorded in memory
`reference_waxman_smits_b`. What remains open is **not** the factor but its consequence: a curve arriving
labelled `meq/L` may already hold meq/mL values, because the label is precisely what the defect corrupts.
`SB-DIO-027` therefore requires a curve declared `meq/L` be **flagged for user confirmation rather than
converted**. Whether that flag belongs at import (`DIO`) or at the CEC/Qv consumer (`ENV` / `SHR`) is the
open part, and it is the only place in this chapter where a unit decision reaches a saturation equation.

**O-3 — The `1e30` DLIS magnitude clamp has no external source.** `dlis.rs:183` screens
`v.abs() > 1e30` alongside the null rules. The screen is defensible as a sentinel guard rather than
a null convention, but the number is uncited — it appears in no vendor document read for this
chapter and in no repo document. §5.2 records it as as-built with the code as its own source, which
is the honest tier and not a satisfactory one. **Needs either a citation or a derivation from the
largest physically meaningful log value.**

**O-4 — `SB-DIO-023` is blocked on `ENV` publishing the physical-range table.** The requirement is
to validate columns against family ranges at import; the ranges themselves are `ENV`'s
(`20_envcorr-qc.md`), which owns judgements about whether a measurement is physical. Defining a
second copy here would be the same `SB-CORE-007` violation §3.8 records between `curves.rs` and
`export.rs`. **`SB-DIO-023` is P0 and its parameter table is owned elsewhere — this is the sharpest
cross-chapter dependency in the chapter.**

**O-5 — Which curve families are logarithmic is unclassified.** `SB-DIO-057` needs it. The
classification is petrophysical and belongs with the family table in `curves.rs` under `ENV`'s
review, not asserted here.

**O-6 — How Techlog's own loader resolves its multi-name `<Channel>` element is unknown.** This is
the dossier's O-6, and D-4 establishes that it **does not block**: the many-to-many rule shape
`{names: [regex], nulls: [f64] | NoNull}` is correct under either answer. Carried so the open
question is not mistaken for a resolved one.

**O-7 — Whether `.xls` support is built on an existing crate or implemented from the specification
is undecided.** `SB-DIO-058`/`-059` state the obligation and the association rule, not the
mechanism. A crate that already implements `[MS-XLS]` cell records would satisfy `SB-DIO-059`
immediately; the drawing-anchor half of `SB-DIO-058` is the part likely to need direct
implementation. A build decision, not a requirement decision.

**O-8 — The `~O` provenance encoding needs a house convention.** `SB-DIO-051` requires method and
parameters in the file; LAS 2.0's `~O` is free text with no structure, so SandiBumi must define a
convention that is human-readable and machine-parseable by its own reader. Until it exists,
`SB-DIO-051` cannot be tested beyond "something was written".

**O-9 — The malformed-input corpus needs a home and licence-safe fixtures.** `SB-DIO-061` requires
an in-repo corpus. Fixtures must be synthetic or provably redistributable; no client file may enter
the repository. Where a real delivery exposed a defect, the fixture is a **reconstruction of the
defect**, not the file.

**Inherited dossier open items.** The dossier's §6 carries fifteen open items (O-1…O-16, with its
O-6 closed). Their disposition is accounted for row-by-row in §8; none is silently dropped.

### 7.2 Escalations

**E-1 — Three of the four "verified facts" this chapter was briefed with about `SB-CORE-001` are
stale, and the source document should be corrected.** Re-verified at source 2026-08-07:

| Briefed claim | At source today |
|---|---|
| `LasFrame.depth_unit` is parsed then discarded under `#[allow(dead_code)]` | **False.** `parsers.rs:440-444` — the attribute was deliberately removed, with a comment recording that silencing it is what hid the field being unused |
| The frontend contains no occurrence of `depth_unit` | **False.** `src/ipc.ts:6-14` `getProjectDepthUnit`/`setProjectDepthUnit`; `src/depthUnitPref.ts` |
| Nothing reconciles the file's unit against the project's | **False.** `units.rs:220` `resolve_index_unit`, wired at `ingest.rs:161-171`, four of five arms correct |
| The family table has no `DEPTH` entry | **True** (`curves.rs:21-37`) — and **deliberate**, not an omission |

**Why it matters.** The correction changes the requirement set from "build the carry" to "close two
specific holes" — the DLIS path (`SB-DIO-016`) and the LAS writer (`SB-DIO-017`) — and it changes
the status from `ABSENT` to `PARTIAL`. A chapter written on the briefed facts would have specified
work that already ships and missed both real holes. **Escalated to the coordinator: whichever
document carries those four facts should be corrected before another chapter is briefed from it.**

**E-2 — `docs/commercial/PROVENANCE_SWEEP.local.md` row 23 is stale.** It records
`ARSHILLA_PYTHON` — the previous product name — appearing in ten user-facing error messages. A
repo-wide search now finds it **only** inside `python_engine.rs` (`:45`, `:182-183`), as a
deliberate backward-compatible fallback with the rationale stated; `:48` and `office.rs:316` name
`SANDIBUMI_PYTHON`. The finding is fixed and the sweep should be updated.

**E-3 — A candidate new `SB-CORE` id, not minted here.** `SB-DIO-049` — *no artifact ships that our
own reader rejects* — is written as a `DIO` requirement, but it is not a `DIO` obligation. It
applies to every domain that emits a file: `23_plotting-interactivity.md`'s PDF and SVG,
`24_ml-advanced.md`'s model exports, `22_database-model.md`'s backups. The evidence is domain-general
too: D-28 records that **IP at factory defaults writes a DLIS its own loader states it cannot read**,
which is a product-level failure rather than a format-level one. Per `CONTRACT.md` this chapter does
**not** mint an `SB-CORE` id; it recommends one and keeps `SB-DIO-049` as the `DIO` instance until
the coordinator decides.

**E-4 — The `.xls` priority is a commercial call, not a technical one.** §3.10 concludes the shipped
refusal is correct engineering and, at **107 of 165 workbooks (65 %)**, an unsustainable permanent
position. `SB-DIO-058`/`-059` are set at **P2**. That is a judgement that the manual Save-As
workaround is tolerable through the first two releases, and it is the coordinator's to confirm — if
petrography is a first-sale capability rather than a second, both requirements are P1 and the chapter
is wrong about them. The technical position does not change either way.

**E-5 — `SB-DIO-023` is P0 with its parameters owned by another chapter.** Flagged so the dependency
is visible at the priority that makes it urgent, not only in §7.1 O-4.

### 7.3 Defect refusals — behaviours SandiBumi declines to reproduce

Each is an incumbent behaviour, documented, that SandiBumi will not implement. These are **wins**:
the work is already done, and the value is in having named the behaviour so nobody adds it later as
a "missing feature".

**R-1 — Vendor project files are named and not touched.** `.itt`, `.itp`, `.att`, `.bor`, `.eli`
hold vendor project state — parameter sets, template definitions, borehole-image processing state.
SandiBumi reads none of them. `CONTRACT.md` §2.2 forbids reverse-engineering them and the amendment
does not open them: no public specification exists, so there is no legitimate derivation path.
Interchange happens through the open formats both sides already write — LAS, DLIS, Geolog ASCII,
CSV. Not a gap.

**R-2 — A different curve's data is never supplied under a requested name.** Refuses D-15 (dossier
§3.5 Hazard 2). The most serious behaviour in the dossier, because the result is correct-looking data
of the wrong provenance — the one failure no range gate, no plot inspection and no QC pass detects.
`SB-DIO-031`, a **MUST NOT** with no configuration that enables it.

**R-3 — A QC module never rewrites a null convention.** Refuses IP's *Clean Data*, which
canonicalises `-999.25 → -999` with the rule shipped enabled (T2 `F_qc_edit_corrections.md` L363).
SandiBumi splits recognition from rewriting: the recognition set is `DIO`'s (`SB-DIO-004`), any
rewriting of a value is `ENV`'s.

**R-4 — An import never widens an existing object's declared interval.** Refuses D-34's *Extend the
Well interval* and *Extend the Set interval*, both shipped **checked**. A depth range that widens by
itself is usually evidence of a mis-identified index or a wrong unit — exactly the diagnostic the
default destroys. `SB-DIO-035`.

**R-5 — A duplicate mnemonic never defaults to merge.** Refuses D-34's *Insert New Data into the
Existing Curve*. A merged curve is unauditable after the fact: there is no way to recover which
samples came from which run. `SB-DIO-036`.

**R-6 — Nothing is resampled on load.** Refuses D-20's nine documented IP load paths, mostly silent.
Already true of the shipped code, so `SB-DIO-021` is a lock rather than a build.

**R-7 — A `TVD` alias is never removed to fix a `TVD` bug.** Refuses the tempting fix. Dropping the
alias converts a wrong answer into an unexplained failure, which is a different `SB-CORE-002`
violation rather than a fix for one. `SB-DIO-014` keeps the capability and removes the silence.

**R-8 — A plate is never given a guessed depth.** This is the shipped `.xls` refusal
(`images.rs:827-830`, `:832-850`), and it stays exactly as written even after `SB-DIO-058` ships. A
plate hung off the wrong sand is a wrong geological conclusion with a photograph attached — more
persuasive, and therefore worse, than no plate at all. Note the refusal is of the *guess*, not of the
*format*; §3.10 separates them and §7.2 E-4 prices the difference.

**R-9 — `MS/FT` ships with no default.** Refuses D-12 and IP's one-way mapping. Choosing silently is
a 1000× error in one of the two readings and there is no evidence in the file that resolves it.
`SB-DIO-029`.

**R-10 — The US survey foot is not modelled.** A refusal of false precision: the difference is 2 ppm
— 6 mm at 3,000 m, below any log's depth resolution — and modelling it would require a per-file
declaration no LAS carries, so the model would be populated by guesswork. `units.rs:34`, §5.1.

**R-11 — LAS 1.2 is readable and not writable.** Follows both IP and Geolog (D-26), and the
asymmetry is correct rather than an oversight: reading a legacy file serves the user, writing one
manufactures a file that cannot express what the project holds. `SB-DIO-043`.

### 7.4 Independent-derivation requirements

`CONTRACT.md` §2.2 as amended: what is prohibited is the **derivation path**, not the capability.
Where a Tier-C item serves a real user need, this chapter must specify a SandiBumi capability derived
independently — from published literature, primary sources or first principles — with its own name,
method, defaults under §2 citation discipline, its own tests, and a **`Betters:`** line naming the
incumbent limitation it removes.

**Re-assessment of the dossier's compliance statement.** The dossier states that *"Tier C is
untouched — nothing in this domain intersects any Tier-C item."* Under the old rule that was a
sufficient answer. Under the amended rule it is not, and re-assessment finds **one genuine
intersection**, because two entries in the Tier-C register are **encodings of data that must be
imported** rather than analysis methods — and importing data is this domain's boundary. Taking the
register class by class:

- **C-1 (patent-claimed) — no intersection.** Omovie Sonic Saturation (US 12,242,011 B2) is a
  saturation method. Nothing in it is a format, a container or an encoding. Not this chapter's.
- **C-2 (proprietary implementation, publicly described) — no intersection.** Experienced Eye/EEFS
  (SPWLA-2021-0091, Brackenridge et al.), Domain Transfer Analysis and Textural Facies are analysis
  capabilities owned by `24_ml-advanced.md`. This chapter touches them only where their *inputs* are
  array data, which is D-1 below.
- **C-3 (opaque artifact) — one intersection, plus one standing prohibition.** Textural Facies'
  `Freq_Tiles` encoding and shipped NN weight files are artifacts this domain would otherwise be
  asked to read. The user need behind them — get image, waveform and array data into and out of the
  product — is real, and it is served by a published container.

---

**D-1 — Multi-dimensional channel import and export.** Class **C-3**. Owning requirement
`SB-DIO-038` (P2). Tests `SB-DIO-T54`, `SB-DIO-T55`.

*The need.* Borehole image passes, NMR echo trains and waveform sets are ordinary deliverables, and
`dlis.rs:71-72` currently discards every one of them with the comment *skip array/multidim channels
for now* — so a file whose entire payload is image data imports as zero curves and no error.

*The derivation path, and what it excludes.* Built from the **published API RP66 V1 specification**,
which defines multi-dimensional channel representation, frame layout and per-axis attributes, and
cross-checked against `dlisio`'s open-source implementation of that same public specification.
Explicitly excluded, and written into the requirement as a **MUST NOT**: consuming, decoding or
inferring any vendor's proprietary tile encoding (`Freq_Tiles`), any shipped neural-network weight
file in any format, or any image encoding recovered by observing a vendor tool's input/output
behaviour. A vendor-trained model is never consumed; if SandiBumi needs a model it trains one, which
is `24_ml-advanced.md`'s obligation under `SB-CORE-014`.

**Betters:** Techlog's own documentation directs users to its **proprietary project format** for
borehole-image interchange rather than to an open container (T3) — which is why §1.2 and §7.3 R-1
name `.itt`/`.itp`/`.att`/`.bor`/`.eli` as read by nobody else. An image loaded into Techlog cannot
be moved to another tool without that tool reverse-engineering a project file. Reading and writing
arrays through RP66 removes that limitation outright: the interchange stays in a published container
any conforming reader can open, so the user's image data is not hostage to the tool that loaded it.

---

**D-2 — Acquisition gaps.** Not refusals. Each names the **specific missing source document** that
would unblock a requirement, per the amendment's instruction to record the gap rather than decline
the capability.

| Gap | Blocks | Specific document needed | Status |
|---|---|---|---|
| A-1 | `SB-DIO-038` (P2) | **API RP66 V1**, *RP66 Organization Codes and Data Format*, the normative multi-dimensional-channel sections | Not held locally. `dlisio` implements it and is open source, so the capability is reachable without it — but the *citations* in §5 would rest on an implementation rather than a specification, which §2.1 does not accept for parameters |
| A-2 | `SB-DIO-042` (P3) | **CWLS LAS 3.0 specification** (Canadian Well Logging Society) | Not held locally. This is the dossier's open item on LAS 3.0 section semantics; Geolog's parser is the only *implementation* whose contract is documented (D-25), and an implementation is not a specification |
| A-3 | `SB-DIO-058`, `-059` (P2) | Microsoft **`[MS-XLS]`** Open Specification | Publicly published. **Not a Tier-C item and not a derivation gap** — recorded here only so the `.xls` decision's derivation path is on the record beside the others |
| A-4 | A future LIS reader (no requirement raised) | **LIS-79** format specification | Not held locally, and no requirement in this chapter depends on it. Recorded because §2's format inventory names LIS and a reader would otherwise look like an oversight |

**A-1 is the one that matters.** It is the difference between implementing RP66 array support
*correctly* and implementing it *the way `dlisio` happens to*. For a reader that is an acceptable
risk; for the writer `SB-DIO-038` also requires, it is not, because a writer that reproduces an
implementation's interpretation of an ambiguous specification produces files that only that
implementation reads — which is exactly D-28's failure. **Recommendation: acquire A-1 before the
write half of `SB-DIO-038` is built.** The read half can proceed.

**No second `CONTRACT.md` §2.1 exception arose.** No vendor lookup-table data is transcribed anywhere
in this chapter. The Matthews & Kelly exception is not invoked and is not treated as a precedent.

---

## 8. Traceability — dossier disposition

Every item in `docs/research_2026-08/cross_tool/data-io.md` is accounted for below. Dispositions
are `CONTRACT.md` §3's: **ADOPTED** (became a requirement, parameter or test), **DEFERRED** (real,
priced beyond this release), **REJECTED** (considered and declined, with the reason),
**EVIDENCE-ONLY** (supports a finding but generates no obligation), **ESCALATED** (raised in §7).

### 8.1 The dossier's own item count

Counted at source on 2026-08-07 rather than taken from any summary:

| Group | Items | Count |
|---|---|---:|
| Ledger — `R-9` | null conventions, F↔N | 1 |
| Ledger — `N-6.1` … `N-6.12` | IP source-quality ledger | 12 |
| Ledger — `N §9 OPEN-1, 2, 5, 6, 7, 8, 9, 10, 11, 12` | IP open questions | 10 |
| Self-audit — `G-D-1` | Geolog `MEQ/L` 1000× defect | 1 |
| Self-audit — `T-D-1`, `T-D-2` | Techlog shipped-file defects | 2 |
| Self-audit — `S-D-1`, `S-D-2` | SandiBumi's own defects | 2 |
| §6 gaps & escalations — `O-1` … `O-16` | open items (`O-6` closed upstream) | 16 |
| §5.2 adoption-spec parameter rows | | 58 |
| §5.3 acceptance tests | 39 numbered, 56 rows with sub-lettered variants | 56 |
| §5.4 applicable `FINDINGS.md` rules | | 10 |
| Critique disposition — blockers | `BLK-1`, `BLK-2` | 2 |
| Critique disposition — majors | `MAJ-1` … `MAJ-13` | 13 |
| Critique disposition — minors | numbered 1–14 | 14 |
| Findings raised by the dossier's own revision | | 7 |
| **Gross total** | | **204** |
| Less items counted twice (`N-6.12` and `O-16` appear both in their own group and in the revision table) | | −2 |
| **Unique items** | | **202** |

**Two discrepancies, stated rather than smoothed.**

1. **The revision-findings table holds 7 rows, not 6.** This chapter's brief described it as six.
   The seventh is the *Geolog export writer split* (12 format cases, 8 receiving `missing_value`),
   which is load-bearing here — it is the whole of finding D-2 and of `SB-DIO-001`. Counted as 7.
2. **The dossier's `O-6` is closed upstream but retains its number**, so `O-1`…`O-16` is 16 numbers
   and 15 live items. Both are dispositioned below; the closed one is marked as such rather than
   omitted, so the count reconciles against the dossier's own numbering.

### 8.2 Ledger and self-audit items (28)

| Item | Substance | Disposition | Where |
|---|---|---|---|
| `R-9` | Null conventions, false-negative ↔ false-positive stakes: an entire curve reading as data | **ADOPTED** | D-1, `SB-DIO-001`, `SB-DIO-004`, §5.2 |
| `N-6.1` | ASCII load null `-999.00` in prose vs `-999` in the panel | **ADOPTED** | D-1; §5.2 records the panel value with its tier |
| `N-6.2` | Source-quality ledger entry | **EVIDENCE-ONLY** — dossier hygiene, no SandiBumi behaviour | — |
| `N-6.3` | `IntPetro.config` vs `IntPetro.exe.config` ambiguity, left explicitly OPEN | **EVIDENCE-ONLY** | §5.2/§5.3 cite the file by the name read |
| `N-6.4` | Mask-file delimiter undocumented | **DEFERRED** | `SB-DIO-033` (P1) states the capability, not the delimiter |
| `N-6.5` | "Extrapolate" vs linear interpolation — the mechanism is misnamed | **ADOPTED** | D-23, `SB-DIO-021`, `SB-DIO-022` |
| `N-6.6` | `$ Geolog Depth Names` screenshot-only | **EVIDENCE-ONLY** — record-only in the ledger; superseded by the T1 read of real Geolog in D-7 | D-7 |
| `N-6.7` | DLIS "use Source to create Curve Sets" undocumented | **DEFERRED** | Not required by any `DIO` requirement |
| `N-6.8` | Fill-Gaps 5: hard vs soft | **REJECTED for `DIO`** — gap filling is a conditioning operation, `ENV`'s | §1.1 seam |
| `N-6.9` | Petrel 8-char set-name limit; stale version list | **DEFERRED** | No Petrel writer is specified in this release |
| `N-6.10` | "(Powerlog)" boilerplate — a trust-calibration failure | **EVIDENCE-ONLY** | Informs the tiering discipline in §2 |
| `N-6.11` | IP ships two disagreeing depth-tag lists plus an empty third | **ADOPTED** | D-6, `SB-DIO-011`, §5.3 |
| `N-6.12` | IP's DLIS writer defaults to 3-D curves its own loader states it cannot read | **ADOPTED** — the strongest single finding in the dossier | D-28, `SB-DIO-049`, §7.2 E-3 |
| `OPEN-1` | Unit-alias/conversion table contents | **ADOPTED — closed** by D-9 (63 lines, 4 families) | D-9, §5.1 |
| `OPEN-2` | Behaviour on an unknown unit at load | **ADOPTED** | `SB-DIO-025` |
| `OPEN-5` | LAS 1.2 not writable | **ADOPTED** | D-26, `SB-DIO-043` |
| `OPEN-6` | LAS section-parsing quirks | **ADOPTED** | D-25, `SB-DIO-044` |
| `OPEN-7` | LAS/LIS parameter table never shown; "closed for engineering purposes" | **EVIDENCE-ONLY** | Minor 12 corrected the closure claim |
| `OPEN-8` | DLIS Curve Attribute Mappings contents | **DEFERRED** | No requirement depends on it |
| `OPEN-9` | `TVD` semantics in the Geolog depth list | **ADOPTED — resolved** by the T1 namespace read | D-7, `SB-DIO-011`, `SB-DIO-014` |
| `OPEN-10` | DBASE4 export null rests on prose, not an image | **EVIDENCE-ONLY**, carried with its caveat | D-1; folded into `O-8` upstream |
| `OPEN-11` | PowerLog `.pup` grammar coverage never enumerated | **REJECTED for this release** — no `.pup` requirement is raised | — |
| `OPEN-12` | Petrosys / OSDU / IP-IC Common DB | **DEFERRED** — P4 horizon, no requirement | — |
| `G-D-1` | Geolog `MEQ/L` factor 1.0 where 1.0E-3 is required — a **1000×** defect | **ADOPTED, and closed here with its derivation shown** | D-11, §5.1, `SB-DIO-027`, `SB-DIO-028`, §7.1 O-2 |
| `T-D-1` | Techlog's null table packs six name/null pairs in one `<Channel>` | **ADOPTED** | D-4, `SB-DIO-006` |
| `T-D-2` | Techlog shipped-file defect (second) | **ADOPTED** | D-3/D-4 evidence set; `SB-DIO-005` |
| `S-D-1` | SandiBumi ships three disagreeing depth-alias lists; `TOPS_DEPTH_ALIASES` accepts `TVD` | **ADOPTED — and split**: two of the three divergences are justified, one is the defect | §3.3, `SB-DIO-011`, `SB-DIO-014` |
| `S-D-2` | SandiBumi's null tolerance is absolute where it must be relative | **ADOPTED** | §3.2, `SB-DIO-004` (P0) |

### 8.3 Dossier §6 open items (16)

| Item | Disposition | Where |
|---|---|---|
| `O-1` | **ADOPTED** into `SB-DIO-025` (conversion coverage declared) | §4.5 |
| `O-2` | **ADOPTED** into `SB-DIO-024` (no silent conversion) | §4.5 |
| `O-3` | **ADOPTED** into `SB-DIO-029` (`MS/FT` ships with no default) | §4.5 |
| `O-4` | **ADOPTED** into `SB-DIO-026` (affine transforms) | §4.5 |
| `O-5` | **ADOPTED** into `SB-DIO-027` (vendor alias not inherited) | §4.5 |
| `O-6` | **Closed upstream.** Number retained so the count reconciles; the many-to-many rule shape is correct under either resolution | §7.1 O-6, `SB-DIO-006` |
| `O-7` | **ADOPTED** into `SB-DIO-045` (multi-well containers) | §4.9 |
| `O-8` | **EVIDENCE-ONLY** — evidence-strength asymmetry on the DBASE4 export null; no behavioural consequence | D-1 |
| `O-9` | **ADOPTED** into `SB-DIO-053` (header mapping, identity never invented) | §4.9 |
| `O-10` | **ESCALATED** as an acquisition gap — the CWLS LAS 3.0 specification | §7.4 D-2 A-2, `SB-DIO-042` |
| `O-11` | **ADOPTED** into `SB-DIO-047` (precision declared) | §4.9 |
| `O-12` | **ADOPTED** into `SB-DIO-060` (signature recognition) | §4.10 |
| `O-13` | **DEFERRED** — outside the format boundary this chapter owns | §1.2 |
| `O-14` | **ADOPTED** into `SB-DIO-020` (duplicate-depth policy) | §4.5 |
| `O-15` | **ADOPTED** into `SB-DIO-050` (re-grid detectable on import) | §4.9 |
| `O-16` | **ESCALATED** — no citable salinity → Rw relation exists in either vendor's text (IP's is rasterized). **Not this chapter's**: it is a saturation parameter, owned by `SHR`/`ENV`. Recorded so it is not lost at the seam | §1.1 |

### 8.4 §5.2 adoption-spec parameter rows (58) — block accounting

The dossier's §5.2 is a 58-row canonical-`Column` parameter table. Rather than restate it, this
chapter's §5 was built to the same discipline and **84 parameter rows across six tables** now ship,
because §5 also carries as-built SandiBumi constants the dossier had no reason to hold.

| Class | Count | Disposition |
|---|---:|---|
| Rows whose value or rule SandiBumi adopts (nulls, alias orders, unit facts, index rules) | 41 | **ADOPTED** into §5.1–§5.3 |
| Rows describing a vendor's internal table that SandiBumi cites but does not adopt | 12 | **EVIDENCE-ONLY**, carried in §5 with the `NON-ADOPTABLE` marker |
| Rows whose value the evidence does not adjudicate | 2 | **ADOPTED as `ABSENT — ships with no default`** (`MS/FT`; the `meq/L` label question) |
| Rows describing capabilities priced beyond this release (WITSML, `.pup`, Petrel limits) | 3 | **DEFERRED** |
| Rows this chapter did not reconcile individually | — | see the note below |

**Stated discrepancy.** A row-by-row reconciliation of all 58 against this chapter's 84 was **not
performed**. The four classes above are a defensible characterisation of how the table was used, not
a line-level audit, and the class counts are this chapter's judgement rather than a measured
partition. This is disclosed rather than papered over: if a reviewer needs the line-level mapping it
is an outstanding task, and it is the one place in §8 where the accounting is by rule rather than by
row. No dossier §5.2 row is known to be dropped; none is known to be contradicted.

### 8.5 §5.3 acceptance tests (56 rows) — block accounting

| Class | Count | Disposition |
|---|---:|---|
| Tests whose intent is carried by a `SB-DIO-Tnn` | 44 | **ADOPTED** |
| Tests of vendor behaviour SandiBumi refuses to reproduce, so untestable here | 5 | **REJECTED**, each traceable to a §7.3 refusal |
| Tests of capability priced beyond this release | 7 | **DEFERRED** |

The 96 tests in §6 exceed the dossier's 56 because this chapter adds the depth-unit round trip in
both units, the DLIS index-unit set, the export-completeness set, the physical-bounds set and the
malformed-input corpus — none of which the dossier's test list anticipated.

### 8.6 §5.4 applicable `FINDINGS.md` rules (10)

| Rule | Disposition | Where |
|---|---|---|
| **6. Null discipline** — *from* this domain | **ADOPTED** | `SB-DIO-001`, `-004`, `-005`, `-006` |
| **15. Curve resolution & depth snapping are logged decisions** — *from* this domain | **ADOPTED** | `SB-DIO-009`, `-010`, `-020` |
| **3. Unit-typed quantities, no magic constants** | **ADOPTED** | `SB-DIO-018`, `-026`; §5.1 |
| **9. Defaults are cited or absent** | **ADOPTED** | §5 in its entirety; §5.6 |
| **13. State the reference convention** | **ADOPTED** | `SB-DIO-014`, `-015` |
| **14. Silent failures are bugs** | **ADOPTED** — the spine of §4.10 | `SB-DIO-054`, `-055`, `-061` |
| **12. Per-correlation unit flags** | **ADOPTED** | `SB-DIO-025`, `-047` |
| **10. Docs generated from code** | **DEFERRED** — a tooling obligation, not a format one | — |
| **1. No raster-only truth** | **ADOPTED** as an evidence-tiering discipline | §2 tiering throughout |
| **7. Ordinal + semantic-name addressing** | **ADOPTED** | `SB-DIO-010` (mechanism recorded), `SB-DIO-033` |

### 8.7 Critique disposition (36)

**Blockers (2).** Both concern the dossier's saturation sections (§3.3, §3.4), not this domain.

| Item | Disposition |
|---|---|
| `BLK-1` — uncited petrophysical constants carry the quantified stakes | **EVIDENCE-ONLY** here, but it is the discipline §5 and §5.6 of this chapter enforce; `SHR` owns the constants themselves |
| `BLK-2` — the Archie stake is wrong in magnitude *and* sign | **EVIDENCE-ONLY** — no Archie claim is made in this chapter |

**Majors (13).**

| Item | Disposition | Where |
|---|---|---|
| `MAJ-1` — corrections not applied to §7's source register | **EVIDENCE-ONLY** (dossier hygiene) | — |
| `MAJ-2` — the "verbatim" G-D-1 block is not verbatim | **ESCALATED into method**: this chapter re-derived the `MEQ/L` factor rather than quoting the block | §5.1, §7.1 O-2 |
| `MAJ-3` — `tp_name_translate` is not mandatory; the pipeline diagram is wrong | **EVIDENCE-ONLY** | — |
| `MAJ-4` — contradiction on the null "family" size; the gap statistic is wrong | **ADOPTED** — §5.2 carries the corrected 16/21 and 5/3 counts | D-3, §5.2 |
| `MAJ-5` — the DBASE4 **export** null was dropped from §2.1's table | **ADOPTED** | D-1, §5.2 |
| `MAJ-6` — IP's export-side **silent curve substitution by type** was missing entirely | **ADOPTED** — became a P0 **MUST NOT** | D-15, D-19, `SB-DIO-031`, `SB-DIO-034`, §7.3 R-2 |
| `MAJ-7` — "all wells in one file" is not one mechanism | **ADOPTED** | D-27, `SB-DIO-045`, `SB-DIO-048` |
| `MAJ-8` — systematic compaction of IP's DLIS loader | **ADOPTED** — D-34 is written from the restored detail | `SB-DIO-035`, `-036`, `-037` |
| `MAJ-9` — IP's DLIS writer defaults to 3-D curves its own loader cannot read | **ADOPTED** | D-28, `SB-DIO-049` |
| `MAJ-10` — compaction of the export-buffer page (11 export targets, LAS wrap = 80 chars, RP66 org codes 440/150) | **ADOPTED** | D-24, `SB-DIO-040` |
| `MAJ-11` — "16 DLIS object classes" is 17 | **EVIDENCE-ONLY** | — |
| `MAJ-12` — `U_DENSITY` omits five shipped Geolog aliases including `PPG`; `PSI/FT` must be **refused, not converted** | **ADOPTED** | D-14, `SB-DIO-027`, §5.1 |
| `MAJ-13` — two source ledger items had no disposition anywhere | **ADOPTED as method** — this section exists to make that impossible here | §8 |

**Minors (14).** All fourteen were fixed upstream. Their disposition here is uniform
**EVIDENCE-ONLY** — each corrects a fact in the dossier rather than creating an obligation — with
four exceptions that changed a number this chapter uses:

| # | Substance | Disposition |
|---|---|---|
| 1 | `CurveAlias.txt` has 5 comment lines, not 4 | **EVIDENCE-ONLY** |
| 2 | `Intpetro.config` is 113,729 bytes | **EVIDENCE-ONLY** |
| 3 | `TechlogUnitAlias.xml` is 55,911 bytes | **EVIDENCE-ONLY** |
| 4 | Line-number drift in `parsers.rs` citations | **ADOPTED as method** — every `file.rs:line` in this chapter was re-verified at source rather than inherited; the drift is why §3 opens with that statement |
| 5 | Techlog's ±5 % merge rule is advisory, not enforced | **ADOPTED** — no requirement treats it as enforced |
| 6 | The Techlog DLIS-null "Example 1" is an image and was not read | **EVIDENCE-ONLY** |
| 7 | The export-buffer page contradicts itself on limit-capable formats | **EVIDENCE-ONLY** |
| 8 | `U_POROSITY` / `U_TEMPERATURE` omit shipped Geolog aliases | **ADOPTED** into `SB-DIO-027` |
| 9 | §4.2 overstates Geolog ("does not guess from a name") | **ADOPTED** — D-8 is written per-path, not per-tool, for exactly this reason |
| 10 | §2.7 omits `Las.xml`'s `Status → WSTA` | **ADOPTED** into `SB-DIO-053` |
| 11 | "~13 % shallow at TD" holds only for whole-hole 30° deviation | **ADOPTED** — §2.2 and §5.1 carry the four-row geometry table instead of one number |
| 12 | `OPEN-7` marked CLOSED on an undemonstrated equivalence | **EVIDENCE-ONLY** |
| 13 | §4.7 labels `N-6.6`/`-6.7`/`-6.9` OPEN when the ledger does not | **EVIDENCE-ONLY** |
| 14 | "No client well names appear" is narrower than it reads | **ADOPTED as method** — §6 states no client well name appears in any fixture, and the owner directive of 2026-08-07 removing operator and asset names is applied throughout |

**Findings raised by the dossier's own revision (7).**

| Item | Disposition | Where |
|---|---|---|
| `N-6.12` (IP writes 3-D DLIS its loader rejects) | **ADOPTED** | D-28, `SB-DIO-049` |
| `O-16` (no citable salinity → Rw relation in either vendor's text) | **ESCALATED to `SHR`/`ENV`** — not a format question | §8.3 |
| `volume_ratio.units` duplicate key (`M3/M3` declared twice) | **ADOPTED** | D-13, `SB-DIO-027` |
| `WellParameters.txt` case anomaly (57 of 58 tokens `%Well.…%`, one not) | **EVIDENCE-ONLY** | — |
| `Dlis.xml` 11th property (`group → GROUP`) | **ADOPTED** into `SB-DIO-053`'s mapping obligation | §4.9 |
| Geolog `.well_query` third export branch carrying no `missing_value` | **ADOPTED** | D-2, `SB-DIO-001` |
| Geolog export writer split — **12 format cases, 8 receiving `missing_value`** | **ADOPTED** — this is the whole of D-2 | `SB-DIO-001`, `SB-DIO-002`, §5.2 |

### 8.8 Surplus — requirements with no dossier antecedent

Fourteen requirements, across thirteen entries, originate in this chapter's own reading of the SandiBumi source, of the repo's
records, or of Jauhar's delivered work, and have no antecedent in the dossier. They are enumerated
because a reader must be able to tell what the dossier bought from what this chapter added.

1. **`SB-DIO-015`** — an undeclared index unit must refuse. From `units.rs:220`. The dossier does
   not examine SandiBumi's unit resolver.
2. **`SB-DIO-016`** — the DLIS index unit must be read and reconciled. From `dlis.rs` having no
   `units::` call and the sidecar skipping the index channel at `:68-69`. **Not found by the audit
   that produced the dossier, and not in its self-audit.**
3. **`SB-DIO-017`** — the LAS writer must write the unit it used. From `export.rs:77-79`, `:85`.
   Also not in the dossier; it is the defect that breaks SandiBumi's own round trip.
4. **`SB-DIO-018`** — one canonical-unit definition. From `curves.rs:21-37` vs
   `export.rs:12-22`; an `SB-CORE-007` violation the dossier had no visibility of.
5. **`SB-DIO-019`** — a project unit change must be an explicit migration. From reading
   `units.rs:169` and finding the behaviour unestablished.
6. **`SB-DIO-023`** — validate on physical bounds, not on labels. From delivered mudlog work
   (`P`, memory `reference_mudlog_gas_curve_traps`). **P0, and the highest-value surplus item**:
   it is the only requirement in the chapter that catches a vendor file whose labels all match and
   whose columns are shifted.
7. **`SB-DIO-054`** — every skipped item counted and named. From the seven silent `continue`s in
   `dlis.rs`.
8. **`SB-DIO-055`** — an export that omits data must say so. From `export.rs:24` omitting the
   generic curve store.
9. **`SB-DIO-057`** — zeros on a log-scale curve are not readings. From delivered mudlog work
   (`P`, same memory).
10. **`SB-DIO-058`/`-059`** — old `.xls` read from the published specification. From
    `01_PRODUCT.md` §4.1's 107-of-165 ratio and `record_petrography.md:612-620`, plus the proven
    BIFF approach in memory `reference_wellsite_xls_biff_recovery`.
11. **`SB-DIO-061`** — the malformed-input contract and corpus. From `01_PRODUCT.md` and
    `docs/qc_audit_prompt_template.md:53` recording a defect family that has recurred 3+ times.
12. **`SB-DIO-062`** — text encoding detected, not assumed. From `images.rs:811-812`, which
    states the principle for one runner and names `parsers::read_text_file` as the same family.
13. **`SB-DIO-063`** — non-ASCII paths across every sidecar boundary. From
    `record_petrography.md:622-624` (`sys.stdin.buffer`, never `sys.stdin`). Indonesian client and
    field names make this an everyday case rather than an edge one.

**What that means for the dossier.** Fourteen of 63 requirements — including **seven of the ten P0s** —
came from reading the shipped source and the repo's own delivery records rather than from the
cross-tool study. The dossier is excellent on *what the incumbents do* and structurally blind to
*what SandiBumi does*, which is what §3 exists for and why `CONTRACT.md` §3 requires it be written
from source rather than from a summary.

### 8.9 Disposition totals

| Disposition | Individually rowed | Block accounted | Total |
|---|---:|---:|---:|
| ADOPTED | 55 | 87 | 142 |
| DEFERRED | 7 | 10 | 17 |
| REJECTED | 2 | 5 | 7 |
| EVIDENCE-ONLY | 21 | 12 | 33 |
| ESCALATED | 3 | 0 | 3 |
| **Total** | **88** | **114** | **202** |

**How the two columns are constituted.** §8.2, §8.3, §8.6 and §8.7 write **90 disposition rows**
covering the 28 ledger and self-audit items, the 16 open items, the 10 `FINDINGS.md` rules, the 2
blockers, the 13 majors, the 14 minors and the 7 revision-raised findings. Two of those 90 rows are
second appearances of an item rowed elsewhere — `N-6.12` and `O-16` are each dispositioned once in
their own group and once in the revision table — so the **unique** individually-rowed count is 88.
The block-accounted 114 are the 58 §5.2 parameter rows (43 ADOPTED, 12 EVIDENCE-ONLY, 3 DEFERRED)
and the 56 §5.3 test rows (44 ADOPTED, 5 REJECTED, 7 DEFERRED). **88 + 114 = 202**, matching
§8.1's unique count exactly.

The block-accounted class counts carry the caveat stated in §8.4: for the §5.2 rows they are a
characterisation by rule rather than a line-level partition, and closing that to a row-level mapping
is the one outstanding accounting task in this chapter.

---

*End of chapter 21.*
