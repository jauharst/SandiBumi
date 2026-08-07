# 22. Database and project data model — requirements

**Dossier.** `docs/research_2026-08/cross_tool/database-model.md` — 1,549 lines, read in full: the
four-tool object-model inventory (§1.1 IP rows 1–30, §1.2 Techlog rows 1–22, §1.3 Geolog rows 1–36,
§1.4 SandiBumi rows 1–12, §1.5 the SegaraBumi precedent), the eleven comparison tables (§2.1–§2.11),
the twelve differences that matter (§3.1–§3.12), the decision ledger D-1…D-26 with OPEN-DB-3, the
prior-ledger disposition (§4.2, rows R-9, R-10, O-8.1–O-8.4, O-8.8, O-OPEN-2, O-OPEN-5–O-OPEN-8,
N-9.1), the adoption spec (§5.1 object model + 13 invariants, §5.2 parameter table, §5.3 the
curve-resolution contract, §5.4 the parameter-file format, §5.5 the audit-entry schema, §5.6 tests
T-DB-01…T-DB-22, §5.7 the `FINDINGS.md` rule bindings, §5.8 the ordered work list), the gaps and
escalations (§6, E-1…E-12 and the authoritative 8-item OPEN tally), the source register (§7.1–§7.7)
and the authoritative `## Critique disposition` (B-1, M-1…M-8, m-1…m-11). The disposition is
treated as authoritative over any body text it corrects, per CONTRACT §4 rule 2.
`database-model_critique.md` was **not** read, per CONTRACT §4 rule 3.

**Evidence tiers held.** **T1** — SandiBumi's own Rust and TypeScript, read directly at source for
every claim in §3; Geolog V14's shipped C headers (`cgg.h`, `parameters.h`, `loginfo.h`,
`setinfo.h`, `wellinfo.h`, `intervalinfo.h`, `file.h`) and its shipped specs
(`wellinfo.wellinfo`, `setinfo.setinfo`); Techlog 2018.2's shipped `Techlog/Data.py` and
`DatasetStep.csv`. **T2** — the IP 2025 and IP 2018 CHM ingests, and Geolog V14's shipped HTML
helpset read as text. **T3** — a raster-only IP options dialog (the single source for the
irregular-set tolerance, and the reason `OPEN-DB-4` exists), and SandiBumi's own manual test plan.
**T4** — project-kb decision records, used once, for scale calibration only (E-9), and never as a
parameter source.

**A tier caveat this chapter carries forward.** This domain is **entirely vendor-architectural and
has no literature dependency** (dossier §6, closing note: "Named papers/specs that would be needed
and are not held: none"). That is unusual in this corpus and it cuts both ways. It means no
requirement here waits on a paper acquisition. It also means that where the three vendors disagree,
**there is no external arbiter at all** — no SPWLA paper adjudicates how a project database should
version a log set. Four parameters in §5 therefore ship `ABSENT — ships with no default` not because
the research fell short but because adjudicating between two vendors with no third source is the
adjudication-disguised-as-a-default that CONTRACT §2 forbids.

**Author date.** 2026-08-07.

**Requirements.** 43 (`SB-DBM-001` … `SB-DBM-043`). **P0: 17** — plus 18 P1 and 8 P2.

**Parameters.** 45 rows in §5, counted by parsing the table rather than by estimate. Of those,
**8 ship `ABSENT — ships with no default`** and 3 are `NON-ADOPTABLE — cited for verification`. The
remaining 34 carry a value with a checkable source: **14 are SandiBumi's own constants at
`file:line`** (T1), 6 are cross-references to `21_data-io.md`, 2 are recorded as evidence only, and
the rest are vendor constants read from shipped headers, specs or help pages. **No row in this table
is a petrophysical parameter** — §5 says why in its first paragraph.

**Acceptance tests.** 44 (`SB-DBM-T01` … `SB-DBM-T44`). Six are labelled `CHARACTERIZATION`.

**Traceability.** §8 carries **297 disposition rows** against the dossier's items, on a counting
basis stated in §8.0 together with the two places where the dossier's own counts measure different
things. **12 requirements have no dossier antecedent** — they come from reading the shipped source
after the dossier was written, or from the 2026-08-07 contract amendment — and are enumerated in
§8.14.

**Cross-cutting requirements this chapter carries.** `SB-CORE-004` (a parameter carries its
source), `SB-CORE-010` (every computed curve answers "how was I made?"), `SB-CORE-011` (a project
re-runs byte-identically), `SB-CORE-014` (a learned model carries its training provenance),
`SB-CORE-002` (a degraded or failed result is never presented as a clean one), `SB-CORE-007` (one
definition per constant and transform), `SB-CORE-032` (no long operation holds the global lock),
`SB-CORE-033` (the content-hash compute cache — **parked by Jauhar's direction; assessed here, not
implemented**), `SB-CORE-035` (backend well scoping) and `SB-CORE-036` (honest cancellation). All
are defined in `04_CORE_REQUIREMENTS.md` §15.1 and are cited, not restated; **no new `SB-CORE`
identifier is minted here.** Three candidate `SB-CORE` gaps and one stale-evidence correction are
raised for Jauhar in §7.2 rather than minted or edited into that file.

**Contract amendment.** This chapter is written under `CONTRACT.md` §2.2 **as amended 2026-08-07**
(reconstruction prohibited, independent derivation required) and §2.2.1 (defect refusals and
derivation requirements never in one list). §7.3 and §7.4 are separate sections for that reason.
The amendment gives this chapter a job the method chapters do not have, stated in §1.

---

## 1. Scope and boundary

This chapter owns the shape of what SandiBumi stores and the rules that govern writing to it: the
project file and its format contract, the schema of every table that holds a well, a curve, a zone,
a parameter or a model, the write disciplines that stand in for constraints the engine does not
enforce, the provenance record attached to every computed number, the versioning and archive model,
the concurrency model around the single writer, the identity and uniqueness rules for wells, curves,
sets and depths, the migration and backup path, and the integrity checkers that make all of the
above falsifiable rather than aspirational.

It owns those as **storage contracts**. What gets computed belongs to a method chapter; what the
number *means* belongs to a method chapter; **what must be true of the record for that number to
survive an audit belongs here.**

### 1.1 The chapter's thesis, stated once because every section serves it

`05_STRATEGY.md` §18.1 and `03_EVIDENCE_BASE.md` §14.4 make reproducibility the product's central
claim. That claim is not made true by the plotting layer, the equation layer or the report writer.
It is made true or false **in the schema**, because a fact that was never written down cannot be
recovered by any downstream feature, however good.

This has a sharp consequence for how the chapter reads. A provenance requirement that says "the run
is recorded" is worth nothing; every one of the three incumbents records that a run happened. The
requirements below are written at the level where the incumbents actually fail — **which curve,
which version of which module, which parameter value, and from which source that value came** — and
each names the column that carries it and the state it takes when the answer is not available.

`SB-CORE-004` is the discriminating case and it is worth stating plainly at the top. All three
vendors record parameter *values* in their audit trails. None of the three records where a value
came from. IP's history is six flat columns (ID, Event, Date, Item, User_Name, Comments — T2
dossier §2.8); Techlog's `HistoryItem` is three fields (`dateTime`, `userName`, `description` — T1
`Techlog/Data.py:586-618`); Geolog's Details table is the richest of the three and is still
Location/Mode/Unit/Name/Value (T2 `AuditTrail/audit_trail_hc.1.05.html`, `…1.06.html`). **A record
of the value without the source is an activity log, not provenance.** Everything in §4 group A
exists to close that one gap.

### 1.2 Named seams

**`21_data-io.md` owns import, parsing and export formats.** That chapter decides how a LAS, LIS,
DLIS, ASCII or vendor-native file is read and written, what the null-declaration line says on the
way out, and how a name is clamped to a target format's length limit. This chapter owns what
happens to the parsed result once it is in the store: which table it lands in, what identity it
takes, what uniqueness rule applies, and what provenance row accompanies it. The seam is concrete:
`21` owns `NULL_WRITE_LAS`, this chapter owns `NULL_GEOLOG_FLOAT_THRESHOLD` and the store-side rule
that a suspect sentinel is flagged rather than coerced (dossier §5.2, invariant 10). Where a rule
has both a parse half and a store half — the null screen, the name clamp, the sampling-style
verification — this chapter states the store half and cites `21` for the other, and §5 marks those
three rows as cross-references rather than restating a value.

**`23_plotting-interactivity.md` owns display and the interactive read paths.** Panning a log view,
picking a point on a crossplot, rendering a curve at screen resolution and caching what was rendered
are that chapter's. This chapter owns the *shape and cost* of the reads those paths issue, which is
where `SB-CORE-032` and `SB-CORE-035` live: whether a read is `O(interactive set)` or `O(project)`
is a data-model property, not a rendering one. Test `SB-DBM-T30` publishes the scale curve that
`23` needs and does not itself set a frame-rate target.

**`27_ip-install-blockers.md` owns packaging, and R7 — the project database is unencrypted at rest —
as a deployment and key-management question.** This chapter owns only the data-model half of R7: what
the file format must expose so that an encryption decision can be made later without a schema
rewrite (a stamped format version, a single-file container, an engine-copy path that already exists
for backup and compaction). It does **not** specify a cipher, a key store, or a passphrase flow.
Where §4 touches R7 it does so by not foreclosing it.

**`24_ml-advanced.md` owns the estimators; this chapter owns the model store.** The seam matters
because `SB-CORE-014`'s enumeration — training-set identity, seed, library versions, artifact
identity — is none of a parameter, an input curve, or a module version, so the `SB-CORE-010` schema
in group A does not hold it. Group C is the mechanism, and it is specified here because it is
schema. Which algorithm is trained, and whether the training protocol is defensible, is `24`'s.

**`14_cutoffs-summation-mc.md` owns the Monte Carlo method; this chapter owns the seed record.**
`SB-CORE-011` notes the MC path is already seeded from `(seed, index)`. The requirement here is the
generalisation: *what else must be pinned* for a whole project to re-run byte-identically, and where
that pinning is recorded. §4 group B enumerates it.

### 1.3 What this chapter owes the contract amendment

`CONTRACT.md` §2.2 as amended requires that where a Tier-C item serves a real user need, the owning
chapter specifies an independently-derived capability rather than refusing. Three of the register's
items touch this domain, and they are handled in §7.4 with their class, their sources and their
`Betters:` lines.

But the amendment also creates a chapter-level obligation that only the data model can discharge,
and it is worth naming here rather than burying it in §7.4. **An independently-derived capability is
defensible only if the derivation is recorded.** A method built from a published paper and a method
reconstructed from a competitor's binary produce the same numbers in the same columns; what
distinguishes them, years later, in front of a client or a court, is whether the primary source is
written down and travels with the result. Human memory does not survive staff turnover, and a git
history does not travel into a deliverable.

That makes "the provenance record carries a **derivation citation**, not only a parameter value" a
requirement of this chapter and not a documentation practice. It is `SB-DBM-005`, it is P0, and it
is the thing that makes the whole C-2 class safe to build at all: a method whose primary source is
recorded in the project database is auditable by anyone who opens the file; a method whose origin is
remembered by nobody is, on the evidence available to an auditor, **indistinguishable from a
reconstruction.** The amendment permits the derivation path. The schema is what proves it was taken.

### 1.4 One vocabulary warning, because it causes real errors

The word **set** means four incompatible things across the tools in this dossier, and the chapter
uses it in exactly one sense. IP has *Curve Sets* (named groupings of curves within a well) and
*Parameter Sets* (a module's parameter state, optionally per zone). Techlog has *Datasets*, in which
a zonation is itself a dataset. Geolog has *Sets*, each of which owns exactly one reference log and
declares exactly one sampling style — a much stronger object than either. SandiBumi's `log_sets` is
closest to Geolog's, and **that is the sense used throughout this chapter**: a set is a versioned,
provenance-bearing grouping of curves that declares its frame. Where a vendor's sense is meant it is
named with the vendor ("IP Curve Set", "Techlog Dataset"). This is not pedantry: a requirement
written against the wrong sense of "set" produces a schema that silently loses either the frame
declaration or the parameter state.

---

## 2. What the incumbents do — the requirement-bearing findings

Findings are numbered `F-nn` and are referenced by those ids in §4 and §8. Only findings that
generate an obligation appear here; the rest of the dossier's inventory is accounted for in §8.

### 2.1 Provenance and audit — where all three fail the same way

**F-01 — All three tools record what a run did; none records why the value was chosen.**
*Tiers: T2 (IP), T1 (Techlog), T2 (Geolog). Tools: all three.*

The three audit schemas, side by side (dossier §2.8):

| | IP | Techlog | Geolog |
|---|---|---|---|
| Storage | one `IPDBWellxxxx.history` per well | per-variable `HistoryItem` list | comments in the reserved `AUDIT_TRAIL` set *inside the well file* |
| Record shape | 6 flat columns: ID, Event, Date, Item, User_Name, Comments | 3 fields: `dateTime`, `userName`, `description` | 2-level: Well History (Date-Time UTC, User, View, Source, Comment) → Details (Location, Mode, Unit, Name, Value) |
| Source of a value | — | — | — |

Geolog's is by a wide margin the best-shaped of the three: two levels, a controlled vocabulary for
`Location` (Parameter, Comment, Set, Constant, Interval, Log) and `Mode` (Input, Output, Delete,
Rename, Save, Save As, Save Cancel), units carried per detail row, UTC storage. It still has no
column that answers *where did this `m` come from*. IP's `Comments` column is free text and
therefore not queryable as a source. Techlog's `description` is one string.

**Consequence, quantified.** A shaly-sand `Sw` deliverable at 2,000 wells carries on the order of
five to eight cited petrophysical parameters per well per zone. Under any of the three incumbent
schemas the *values* survive and the *citations* do not, so re-defending the study a year later
requires re-deriving every one of them from memory or from a spreadsheet outside the project. This
is precisely the failure `SB-CORE-004` names, and it is the single most valuable thing this chapter
can fix, because closing it costs one column and an enforcement rule.

**F-02 — IP cannot diff its own parameter states without shelling out to third-party software.**
*Tier: T2 (`O` §3.5, `historymodule.htm`; `O` §10.11). Tool: IP.*

IP's History module compares parameter states by invoking **ExamDiff**, an external text differ.
IP2025 adds an editable SQL Row Filter over the history, which improves querying and does not touch
differencing. Geolog's Details table, being a name-value pair list, makes the same operation a
relational join — the dossier's §5.5 states it as a `FULL OUTER JOIN`.

**Consequence.** A structured diff answers "which parameters changed between run 3 and run 4, with
units" as data. A text diff answers it as coloured lines a human must read, cannot be filtered by
magnitude, and cannot be embedded in a deliverable. The obligation is to adopt the name-value shape,
which makes the diff free rather than a feature.

**F-03 — Geolog's audit trail is conditional on the storage backend.**
*Tier: T2 — one page in the entire helpset, `RF01_Database/database_01_overview_hc.1.2.html`,
verified by full-helpset search. Tool: Geolog.*

"The audit trail functionality is only available when an EposData database is used." A Geolog
project not on EposData produces no audit trail at all, and the constraint is documented in exactly
one sentence, in an overview page, not in the Audit Trail chapter itself.

**Consequence.** Provenance that depends on a deployment choice is provenance a user can lose
without being told. SandiBumi ships one storage engine and one file format, so the obligation is
narrow but real: **no configuration, deployment mode or preference may disable the provenance
record.** It is not a feature flag.

**F-04 — IP's curve-resolution chain is powerful and completely opaque at run time.**
*Tier: T2 (`O` §2.11, §10.5). Tools: IP, with Geolog's priority rule as the corroborating design.*

IP resolves which curve feeds a module input through a five-stage chain: explicit name → working
input set → alias grid in one of three modes (`OFF` / `MANUAL` / `AUTOMATIC`) → `Final`-flag filter
→ curve type with a most-recently-modified tie-break. Geolog's equivalent priority is default set →
`setinfo` set order → latest version → alias order (T2 `…hc.2.15.html`). Neither records the outcome.

**Consequence, quantified.** In a well with three GR curves across two sets and one flagged Final —
an entirely ordinary situation in a deltaic clastic sequence after a re-log and a re-processed pass —
the chain has
three plausible answers and the deliverable shows one number. The question *"which GR did this Vsh
actually use"* is unanswerable in both tools. The obligation is not to change the chain, which is a
good chain, but to **record its outcome**: `(input_slot, chosen_curve_id, rule, rejected_candidates)`.

**F-05 — A single header dropdown silently changes numerical results.**
*Tier: T2 (`O` §3.3, `wellheaderinfo.htm`; mitigation `O` §10 item 9). Tool: IP. **The only
documented metadata→physics path in the three tools.***

IP's **Logging Contractor** well-header field selects the neutron/density crossplot overlays, sets
the Neutron Tool Type for Basic Log Analysis and Mineral Solver, and selects the neutron look-up
tables for limestone→sandstone/dolomite matrix conversion and salinity correction. It is edited in a
header tab, not in any module dialog.

**Consequence.** Changing it re-selects ρma look-ups and salinity corrections for every subsequent
run, and nothing in the run record shows that an input changed. The failure computes, plots and
ships. This is the class the project's own `CLAUDE.md` names as never-delegate, and the obligation
follows directly: **an attribute that drives physics is an input of the module that consumes it**,
written into the run record with its value at run time, invalidating that run's outputs when it
changes, and failing with a named error when it is unset rather than defaulting silently.

### 2.2 Versioning — the one place a vendor is unambiguously ahead

**F-06 — Geolog versions logs intrinsically; every create/edit/modify writes `<log>_N`.**
*Tier: T2 (`…hc.2.15.html`). Tool: Geolog; no equivalent in IP or Techlog.*

`<log>_1` is the original. Referencing a log without a version resolves to the latest, subject to
log filtering. The vendor states the concurrency consequence explicitly: "If multiple users are
editing the same log, Aspen Geolog will save the edited logs as different versions." Techlog's
nearest equivalent is a Studio-held version chain, i.e. it requires a separate product (dossier §2.8
availability row). IP has none.

**Consequence.** Version-on-write makes "re-run = version N+1, never overwrite" a property of the
store rather than a discipline the application must remember. SandiBumi already has the discipline
(§3.4); the finding's obligation is that the *version* participates in curve resolution and in the
provenance record, as Geolog's priority rule does, so that a provenance row pins a specific version
rather than a name that has since moved.

**F-07 — Geolog's Database Upgrader names its backup by the source format version.**
*Tier: T2 (`O` §4.3 equivalent; the vendor's own example string `4.7 Upgrade Backup 13 May 2022`).
Tool: Geolog. IP's ad-hoc per-well upgrade path takes no backup at all.*

**Consequence.** A chain of upgrades — 4.7 → 4.8 → 4.9 — leaves a shelf of backups. Labelled by
source, each says what it can restore. Labelled by target, they all say what they became, and the
one that holds the pre-migration data is identifiable only by timestamp. SandiBumi ships the backup
(§3.6) and names it by target; the delta is a naming rule, and it is small.

### 2.3 Identity, naming and vocabularies

**F-08 — A shipped vendor vocabulary fails its own foreign key in 2 of 131 rows.**
*Tier: T1 — `Geolog-V14/specs/setinfo.setinfo` and `include/setinfo/setinfo.h`, re-parsed 2026-08-06.
Tool: Geolog. Dossier `OPEN-DB-3`, E-12, D-22.*

`setinfo.setinfo` maps 131 standard set names to a KIND, against a second 45-row `KIND_NAME` /
`KIND_DESCRIPTION` vocabulary in the same file. Two rows name kinds the file never defines:
`REFERENCE→ALL` and `RECEIVER_CHECKSHOT→RECEIVERS`. Two defined kinds are never used: `RECEIVERCS`
and `REFERENCE`. The header `setinfo.h` publishes `SETINFO_KIND_REFERENCE "REFERENCE"` as an API
constant — so the API expects a kind the shipped spec never assigns to anything.

**Consequence, quantified.** 2 of 131 rows is 1.5 %, which is exactly the rate at which a
fuzzy-matching importer looks like it works. `RECEIVERS` → `RECEIVERCS` is a one-character edit
distance and an importer that "helpfully" coerces it produces a clean-looking import that has
invented a vendor fact. The obligation is that a vocabulary import validates against the file's own
vocabulary and **routes failures to a review queue**; an import that reports "clean" on this file is
wrong.

**F-09 — Geolog publishes its name-length limits; two vendor sources disagree on one of them.**
*Tiers: T1 (`cgg.h` `GG_L_SET 32`, `GG_L_UNITS 16`, `GG_L_WELL 250`; `specs/wellinfo.wellinfo`
`WELL ALPHA*32`) and T2 (`…hc.2.16.html`). Tool: Geolog. Dossier `OPEN-DB-1`, E-2.*

Set names 32 characters. Units 16. Log names 32 total, with the vendor recommending a 29-character
descriptive part "to allow for the possibility of a two digit version number" — the version suffix
lives inside the same budget. Well/PWI name is **250 per the header and the manual, 32 per the well
index spec**, unreconciled by any vendor text.

**Consequence.** These bind the export path, which is `21_data-io.md`'s. What binds *this* chapter is
the shape: a name limit is a property of a target format, not of the store, so the store must not
adopt one. The obligation is that SandiBumi's internal identity is not a name at all.

**F-10 — IP's own limit for its Curve Set short name is stated as both 8 and 4.**
*Tier: T2, two pages of the same manual — `managecurvesets.htm` = 8, `manage-multi-well-curve-sets.htm`
= 4. Tool: IP. Ledger `O-8.2`.*

A leading-digit prohibition is stated once. **Consequence:** same as F-09 — an export-path clamp,
recorded here as an open item rather than adjudicated, and clamped to the safer 8 with renames
logged.

### 2.4 Null, absence and the difference between them

**F-11 — Geolog separates "no data" from "no parameter supplied" at the constant level.**
*Tier: T1 — `cgg.h` (`MISS_INT −2147483647`) and `parameters.h`
(`PAR_DEFAULT_NONE_INT −2147483646`). Tool: Geolog; neither IP nor Techlog does this.*

Two adjacent integers, deliberately distinct, in two different headers of the same product. One
means the measurement is absent; the other means the interpreter never supplied the parameter.

**Consequence.** Collapsing them is how a silently-defaulted parameter becomes indistinguishable
from a deliberately-unset one — which is `SB-CORE-004`'s failure mode expressed in the type system.
The obligation is that the two states are distinguishable at **every** layer: store, IPC, UI and
export.

**F-12 — Geolog's own null magnitude is stated two ways, eight orders of magnitude apart, and the
vendor instructs callers not to test equality.**
*Tiers: T1 `cgg.h` — `MISS_FLOAT = −1.0e30` with
`#define IS_MISS_FLOAT(v) ((v) < (MISS_FLOAT/10.0))` — **vs** T2
`database_05_database_access_hc.5.09.html`, which states the undefined log value as **−1.0D38** and
instructs setting numeric values to −1D38. Tool: Geolog. Dossier `OPEN-DB-2`, E-10.*

The macro is a **strict** inequality against `MISS_FLOAT/10.0`, i.e. −1.0e29. A value exactly at
−1.0e29 is therefore **data**, not null.

**Consequence, quantified.** An equality screen against either magnitude leaks the other straight
into the data as a real number eight orders of magnitude from any petrophysical value — it will not
be caught by a range check on a log scale, and it will destroy any statistic computed over the
curve. The obligation is the threshold form, computed from the same constant that would be
exported rather than hand-typed: a hand-typed `−1e29` decimal can land on the wrong side of the
exact-boundary sample because `MISS_FLOAT` is a `float`-cast constant and the quotient is taken in
double.

**F-13 — Techlog ships its own distinct sentinel, in executable source.**
*Tier: T1 — `Techlog/Data.py:15`, `MissingValue = −9999`. Tool: Techlog.*

**Consequence.** −9999 was already on the recognised-suspect list from ledger R-9; the finding
upgrades it from a convention to a named producer, which is what makes flagging it on import
defensible rather than superstitious.

### 2.5 Sets, frames and sampling style

**F-14 — A Geolog set declares one sampling style and owns one reference log; the vendor documents
that it cannot detect when the declaration is false.**
*Tier: T2 — `…hc.2.06.html` ("only one interpolation mode can exist in a single set") and
`database_05_database_access_hc.5.11.html`: there is "no way … to detect whether the data set … is
periodic or aperiodic other than by checking for a constant depth sample increment". Tool: Geolog.*

**Consequence, and it is the worst failure mode in this dossier.** A set declared
`CONTINUOUS_REGULAR` at 0.1524 m that actually contains a 40-row gap will, on a frame-indexed read,
place every post-gap sample **6.1 m shallow** (40 × 0.1524 m). The curve remains perfectly
continuous, plots without a break, and correlates against a neighbouring well as a real structural
observation. There is no visual signature. The obligation is to verify the declaration against the
reference column on ingest and store the verdict, contradicting the declaration where it is false.

**F-15 — Geolog rounds when interpolating integer logs; the other two have no categorical type.**
*Tier: T2 (`…hc.2.06.html`). Tools: Geolog (has it), IP and Techlog (do not, per the dossier's §3.10).*

**Consequence.** Linear interpolation of a facies code produces codes that do not exist. A facies
curve resampled from 0.1524 m to 0.1 m under a `FLOAT` assumption yields values like 2.37 —
which then either round silently to a class the rock is not, or propagate as a number into a
summation. The obligation is a genuine categorical curve type, with resampling that rounds and
reports boundary crossings.

**F-16 — A module must never write back to the reference column.**
*Tier: T2 — IP's User App API rule, stated flatly: "never write back to the Depth curve" (`O` §2.5).
Tool: IP.*

**Consequence, and it is total.** Editing the depth column of a shared frame re-datums *every other
curve on that frame at once*, and the result plots as a perfectly continuous log. It is F-14's
failure with a larger blast radius and the same absence of a visual signature.

**F-17 — Every depth quantity needs a declared datum, and only one tool prints its sign convention.**
*Tier: T2 (`…hc.2.12.html`). Tool: Geolog; IP and Techlog state no convention in the pages read.*

TVDSS positive down; elevation positive up from the measurement reference.

**Consequence.** Comparing an MD zone top with a TVDSS contact without a reference frame is a
category error that produces a number. The obligation is that a depth quantity carries its datum
from `MD | TVD | TVDSS | TVDKB | TWT | OWT | CDEPTH` and cross-datum comparison is refused unless a
frame exists for that well.

### 2.6 Parameters as first-class objects

**F-18 — IP addresses parameters by sparse ordinal, and the ordinals are append-only.**
*Tier: T2 (`O` §2.9). Tool: IP. Ledger `R-10` is the proof case.*

IP's NMR parameter list numbers `1,2,3,4,5,8,9,10,11,12,14,15,16,17,19,20,23,24,25,26,32,38,…` —
sparse precisely because retired parameters leave gaps rather than being compacted out.

**Consequence, quantified — this is ledger R-10 and it happened.** A parameter file written by one
build and loaded by another, addressed by ordinal alone, silently binds ordinal 41 to whatever now
occupies slot 41. In the recorded ClayVol case that was one clay-volume parameter substituted for
another. It computes. The obligation is a **dual handle**: ordinal *and* semantic key, both
resolving to the same parameter or the load fails naming both. A file carrying one handle loads with
a warning; a file carrying two that disagree does not load.

**F-19 — A parameter's value can be a per-zone tilt, and the tilt is a property of the value.**
*Tier: T2 (`O` §2.4, §10.2). Tool: IP.*

IP's `Lg` prefix marks a value interpolated logarithmically between zone endpoints; interpolation is
**within-zone only** and the parameter steps at a zone boundary.

**Consequence.** Storing a tilted parameter as a scalar loses the physics — `Rw` tilted
logarithmically between 0.28 and 0.19 across a zone is not 0.235. The obligation is that `tilt` is
stored on the value, not carried as a UI display mode.

### 2.7 Scale, materialisation and what "interactive" costs

**F-20 — Geolog separates the Project application from the Well application, and materialises
nothing at project level.**
*Tier: T2 (dossier §1.3, §3, D-4). Tool: Geolog. IP publishes an in-memory cap of 2,000 wells for
its whole working set (T2 `O` §3.1); Techlog publishes no capacity limits at all — a verified
negative over all 3,808 shipped `Doc/` HTML pages (E-3).*

**Consequence.** The tools that scale do so by never materialising the project, and the tool that
publishes a cap publishes it because it materialises. The obligation is architectural and is stated
as a read-shape requirement rather than a number: **the interactive set is the only thing
materialised**, and project-level operations are queries, not loads.

**F-21 — Geolog publishes a live-fraction threshold for compaction.**
*Tier: T2 — `RF01_Database/database_03_database_format_hc.3.3.html`, `GLDBWell` parameter
`WELL_FULL`, default 75 %, range 1–100 %. Tool: Geolog.*

**Consequence.** It is the only vendor-published number in the corpus for *when* a repack is worth
doing, and it is directly usable because SandiBumi's engine-copy path already produces a compacted
file as a side effect (§3.6).

**F-22 — Techlog pages large arrays at a fixed byte size.**
*Tier: T1 — `Techlog/Data.py`, `CacheVarData.__cacheSize`: "A page of data consists of 10 MB of
contiguous values, in row-major". Tool: Techlog.*

**Consequence.** It is a cited precedent for an array-store page size, in a domain where SandiBumi's
own `array_logs` table currently stores a whole array log per row. Recorded as evidence for a future
array read path, not adopted as a constant.

### 2.8 Limits, and how a published limit goes wrong

**F-23 — A vendor published a shipped-artefact count as a capacity limit for seven years.**
*Tier: T2 — ledger `O-8.3`. Tool: IP.*

**Consequence.** The number was hand-typed into documentation, bumped once when the artefact count
changed, and never reconciled again. This is `FINDINGS.md` §6 rule 10's cautionary case and it
generates a direct obligation: **capacity limits published in SandiBumi's docs are emitted from the
source of truth**, and every limit constant carries a unit type and a source string.

**F-24 — IP's Irregular Set Tolerance is 0.2 ft, and whether that is fixed feet or the well's own
depth unit is not established by any page in the corpus.**
*Tier: **T3** — a transcribed dialog screenshot, `options.htm`, image `_tclip0110.png`, reading
literally "Irregular Set Tolerance, depth wells | 0.2 ft". Tool: IP. Dossier `OPEN-DB-4`.*

**Consequence, quantified at 3.28×.** On a metric workflow at 0.1524 m (6 in) steps, 0.2 **ft** is
0.06096 m — 40 % of a step, a sane snapping tolerance. 0.2 **m** is 1.3 steps and would silently
consume a whole sample into its neighbour. The obligation is not to resolve it (nothing on this
machine can) but to store it **unit-typed**, convert explicitly at the comparison site, and log the
snap decision — so a later resolution changes one constant rather than a code path.

### 2.9 Silent failures as a class

**F-25 — Every one of the three tools has at least one documented silent-drop path.**
*Tiers: T2 throughout. Tools: all three. Ledger `O-8.8`; dossier §3.8, §3.9.*

Geolog's include-well path fills unmatched wells with missing values rather than reporting them.
IP's array handling auto-averages rather than declaring a dimension mismatch. Bulk tops paste
across all three drops unmatched names.

**Consequence.** The obligation is a uniform return shape for every bulk operation —
`{matched, unmatched, ambiguous}` — with unmatched and ambiguous rows entering a review queue. Zero
silent drops, stated as a testable contract rather than a value.

**F-26 — IP's duplicate-depth resolution is published as a constant; the class of the decision is
what matters.**
*Tier: T2 (`O` §4.6) — 0.01 ft perturbation for duplicate FPRESS depths. Tool: IP.*

**Consequence.** Three resolutions are defensible for two samples at one depth: reject with a named
error, keep both and mark the set `POINT` (legitimate for core plugs and pressure points), or
perturb by a declared unit-typed constant. What is forbidden is a silent survivor. This interacts
directly with §3.3: SandiBumi's `computed_curves` has no primary key by design, so **nothing in the
engine enforces this** — it is a write-discipline contract and therefore needs a checker to be real.

### 2.10 Concurrency and locking

**F-27 — Geolog's file-locking chapter promises documentation it never delivers, and the substance
turns out not to need it.**
*Tier: T2 — `GS3_Env_Guide/environment_03_project_structure_hc.03.02.html` claims the chapter
"explains … how file locking occurs"; no sub-page does. Re-verified with a word-boundary search:
every `lock` match in `RF01_Database` is the substring `block` in CSS. Tool: Geolog. Dossier E-1,
**downgraded 2026-08-06**.*

Chapter 06 answers it: a Well Data Server serves "a single project for a single user, where user is
defined as a single user ID on a single computer", started on demand and terminating when unused
(`…hc.6.4.html`). With intrinsic log versioning (F-06) and Epos permissions there is no cross-user
lock to document.

**Consequence.** The single-writer model is the vendor-corroborated design, not a limitation to
apologise for. IP's `IPDBLock` self-clears in 4–5 minutes with a 5-minute multi-user grant window
(`O` §3.6) and is recorded as the only lock-timeout precedent, **not adopted**. The obligation this
generates is narrow and is `SB-CORE-032`'s: the single writer is correct; what must be bounded is
**how long it is held**.

### 2.11 The precedent from inside the house

**F-28 — SegaraBumi's indexer sets an interactive query target and meets it without materialising.**
*Tier: T1 — `sonar_ingest/E_indexer_search.md` §3.2, own design target. Tool: SandiBumi's sibling
product. Dossier §1.5.*

An FTS-backed index over a large well corpus with a `< 50 ms` interactive query target, achieved by
indexing rather than loading.

**Consequence.** It is the in-house existence proof for F-20's architecture, on the same machine
with the same constraints, and it is why `SB-DBM-038`'s read-shape requirement is stated as
achievable rather than aspirational. It is an **own design target, not a vendor value**, and §5
marks it as such.

---

## 3. SandiBumi as-built

Written from the source, read at `D:\XX. SandiBumi\src-tauri\src\` and `D:\XX. SandiBumi\src\` on
2026-08-07. Every status claim carries `file:line`. The repository was read-only for this task; no
file in it was modified, no migration was run, and no project database was opened.

A general observation before the detail, because it changes how the rest of §3 reads. **This
schema's comments are unusually good** — several tables carry the reasoning for their shape,
including the measured numbers behind a performance decision and the failure a discipline exists to
prevent. That is rare, and it is the reason so many statuses below are `PRESENT-OK` rather than
`PRESENT-UNVERIFIED`: the intent is recorded, so divergence from it is checkable. Where a status is
`PRESENT-DIVERGENT` it is almost always because a *second* code path was added later without being
reconciled to the comment that says there is only one.

### 3.1 The project file and its format contract — `PRESENT-OK`

One DuckDB file per project. `db.rs:29` declares `pub const FORMAT_VERSION: i64 = 1;`.
`init_db` (`db.rs:36-42`) runs the format check **before** `create_schema`, and the ordering is
deliberate — creating tables into a file written by a newer build is the thing the gate exists to
prevent.

`check_and_stamp_format` (`db.rs:117-167`, documented at `:108-116`) refuses to open a file stamped
newer than the running build, naming **both** versions and the `written_by` string, and **leaves the
file unmodified**. A file with a missing or unparsable version is treated as legacy 0 rather than
rejected. This is the dossier's R-A and it ships.

**Status: `PRESENT-OK`.** No test pins it; `SB-DBM-T01` does.

### 3.2 Connection tuning and resilience — `PRESENT-OK`

`tune_connection` (`db.rs:54-74`) caps the engine's memory at the engine default divided by four,
clamped to `[1 GiB, 4 GiB]`, overridable by `SANDIBUMI_DB_MEMORY`. `init_db_resilient`
(`db.rs:177-199`) detects a corrupt WAL, moves it aside to `.corrupt-backup-<ts>` and retries once.

Boot notes are collected in a process-global (`db.rs:96-106`: `BOOT_NOTES`, `boot_note`,
`take_boot_notes`) and drained by the frontend after startup, which is how a migration or a
moved-aside WAL becomes visible to the user rather than only to a log file.

**Status: `PRESENT-OK`.**

### 3.3 The curve stores, and the primary key that was deliberately removed — `PRESENT-DIVERGENT`

Five stores, with three different uniqueness postures:

| Store | Key | `file:line` |
|---|---|---|
| `standard_curves` | `PRIMARY KEY (well_id, depth)`, seven fixed columns | `db.rs:227-239` |
| `array_logs` | has a primary key; `axis BLOB` NULL means "no axis" | `db.rs:258-287` |
| `computed_curves` | **no primary key, on purpose** | `db.rs:300-305` |
| `computed_curves_archive` | no primary key, same reasoning | `db.rs:335-341` |
| `curve_samples` | `PRIMARY KEY (curve_id, depth)` | `db.rs:760-765` |

The `computed_curves` decision is documented in full at `db.rs:292-299`. The natural key is
`(well_id, depth, curve_name)`; a three-column primary key forces DuckDB to maintain an ART
uniqueness index on every inserted row, measured at **~3.7× slower inserts — 311k rows/s against
1.16 M rows/s** — which "dominated field-scale runs (2000 wells)". Uniqueness is instead guaranteed
by a named **write discipline**: the batch writer always DELETEs a well's rows for the curve names it
is about to write before appending fresh ones, and the point-update path UPDATEs in place. The
comment asserts "no code path ever inserts a duplicate".

**Status: `PRESENT-DIVERGENT`, and the divergence is between the design and its enforcement, not
within the design.** The trade is sound and the reasoning is measured. What is missing is the other
half: **the discipline is asserted by a comment and enforced by nothing.** There is no checker that
looks for a duplicate `(well_id, depth, curve_name)`, and no test that would fail if a future writer
skipped the DELETE. Dossier invariant 12 names exactly this interaction. The magnitude is bounded but
ugly: a duplicated depth in a PK-less table gives last-writer-wins on a read that assumes one row,
and a summation over that curve double-counts the sample.

`migrate_drop_computed_curves_pk` (`db.rs:958-1002`) converts an old database to the PK-less shape.
It is gated on `duckdb_constraints()` so it is a no-op on a fresh file, it takes a backup first, and
it **aborts if the backup fails** — the correct order.

### 3.4 Log-set versioning and the append-only archive — `PRESENT-OK`

`log_sets` (`db.rs:316-329`) is one row per run event:

```
set_id UUID PRIMARY KEY, well_id UUID, set_name VARCHAR, version INTEGER,
module VARCHAR, params_json VARCHAR, inputs_json VARCHAR,
created_at TIMESTAMP DEFAULT now(), frame VARCHAR DEFAULT 'STANDARD'
```

`version` counts up per `(well, set_name)` — the comment states the rule as "re-run = version N+1,
never overwrite" (`db.rs:312-315`). `set_id` was added to `computed_curves` by
`ALTER TABLE … ADD COLUMN IF NOT EXISTS set_id UUID` (`db.rs:310`) so that old and fresh databases
converge on the same five-column shape from one declaration; **NULL means legacy or unversioned**.
`frame ∈ {STANDARD, OWN}` is declared, never inferred (`db.rs:325-328`) — dossier invariant 1,
shipped.

`create_log_set` (`equations.rs:609-624`) computes `version = COALESCE(MAX(version),0)+1` for the
`(well, set_name)` pair and inserts the row. `write_computed_curves_versioned`
(`equations.rs:626-672`) runs inside `with_txn` and does three things in order: DELETE the current
rows for the uppercased curve names, append the new rows to `computed_curves` tagged with `set_id`,
append **identical** rows to `computed_curves_archive`. The archive is what makes a re-run
non-destructive: any prior version can be restored back into current (`db.rs:331-334`).

The batch path (`equations.rs:682-714`) carries a real engine lesson in its shape: it reads **all**
the `MAX(version)` values **before** any INSERT, because reading after an INSERT inside the same
DuckDB transaction trips an internal error. `with_txn` (`db.rs:1543-1564`) wraps
BEGIN/COMMIT/ROLLBACK and documents that DuckDB has no nested transactions, so a `with_txn`-wrapped
writer must never be called from inside another one.

**Status: `PRESENT-OK`** for the versioning mechanism itself. Geolog's F-06 equivalent is matched in
substance, and the archive is arguably better shaped, because it separates the fast current store
from the history rather than making every read version-aware.

### 3.5 What the provenance row actually holds — the `SB-CORE-010` audit — `PARTIAL`

This is the chapter's central as-built finding, so it is stated element by element. For a module
run, the `LogSetSpec` constructed at `workflow.rs:694-706` fills `log_sets` as follows:

| Provenance question | Column | What it actually holds | Verdict |
|---|---|---|---|
| Which run? | `set_id`, `version` | UUID + monotonic version per (well, set_name) | **holds** |
| When? | `created_at` | `TIMESTAMP DEFAULT now()` — local, not UTC | **holds, wrong zone** |
| Which module? | `module` | `req.module.clone()` — **a name, with no version** | **does not hold** |
| Which parameters? | `params_json` | `serde_json::to_string(&req.params)` — the **overrides only** | **partial** |
| From what source? | — | **no column exists** | **does not hold** |
| Which inputs? | `inputs_json` | `log_args` plus `input_set` — **mnemonics, not resolved curve ids** | **partial** |
| Chosen by which rule? | — | **no column exists** | **does not hold** |
| Which depth frame? | `frame` | `STANDARD` or `OWN`, declared | **holds** |
| Which zone set? | — | **no column exists** | **does not hold** |
| Who? | — | **no column exists** | **does not hold** |

Four gaps, in descending order of consequence:

1. **No parameter source.** `params_json` is a value map. This is `SB-CORE-004`, and F-01 shows all
   three incumbents share the gap — which is why closing it is the chapter's highest-value item and
   not merely parity work.
2. **Module identity is a name, not a version.** Running the same module today and in six months
   against the same `params_json` gives two rows indistinguishable in the record that may differ in
   the number. `SB-CORE-011` cannot be satisfied while this is true.
3. **Inputs are mnemonics.** `inputs_json` records that the run asked for `GR`. F-04 is the finding:
   in a well with three GR curves the record does not say which one answered, and SandiBumi's own
   resolution path is a real chain — `equations.rs:301-303` documents that non-standard names fall
   through to `computed_curves` — so the ambiguity is live, not hypothetical.
4. **Parameters are overrides only.** A parameter left at its manifest default appears nowhere in
   `params_json`. If the manifest default later changes, the record reads as though nothing changed.

Two more, smaller. `created_at` is `now()`, which DuckDB resolves as local time, so a project shared
across time zones — or produced either side of a DST transition — orders runs wrongly; F-01's table
shows Geolog storing UTC and displaying local, which is the correct shape. And the equation path
(`equations.rs:1231-1238`) writes `params_json: String::new()` — an **empty string**, not `NULL` and
not `{}` — so an equation run's provenance is structurally indistinguishable from a module run whose
parameters failed to serialise.

**Status: `PARTIAL`.** The mechanism is present, correct in shape and already better than IP's; the
record it carries is missing four elements that `SB-CORE-010` names.

### 3.6 Backup, migration, engine copy and compaction — `PRESENT-OK`

`backup_before_destructive_migration` (`db.rs:908-932`) writes
`<stem>.pre-{FORMAT_VERSION}-backup.duckdb`, **never overwrites an existing backup** (a collision
gets a timestamp suffix), and surfaces the path in a boot note. It is called only by destructive
migrations; the additive ones deliberately do not take one — a backup on every open would bury the
one that matters.

`engine_copy_to` (`db.rs:934-956`) uses `ATTACH` plus `COPY FROM DATABASE`, requires that `dest` does
not exist, and — because it writes only live rows — **is also a compaction**. That is the F-21 hook:
the mechanism a live-fraction threshold would trigger already exists and is already exercised by the
backup path.

`migrate_array_logs_store` (`db.rs:1004-1029`) deliberately takes **no** backup, and the comment
gives the reason: the old stub it replaces was never written to, so there is nothing to lose.

`open_and_migrate` (`project.rs:137-217`) runs twelve migrations in sequence, each timed and emitted
as a `[boot]` note, with one documented ordering constraint at `project.rs:172-173`
(`migrate_core_depth_orig` must run **after** `migrate_point_data_sets`) and a boot note when the
whole open exceeds 10 s (`project.rs:210-215`).

**Status: `PRESENT-OK`.** The dossier's §5.8 item 2 was struck on review for exactly this reason —
it is shipped, not outstanding. One residual delta against F-07: the backup is named by the
**target** format version (`pre-{FORMAT_VERSION}`), so a chain of upgrades leaves backups labelled
by what they became rather than by what they can restore.

### 3.7 Output-name resolution — one place, and it holds — `PRESENT-OK`

`workflow.rs:206-268`, `resolve_output_names(spec, opts)`, documented as "the ONE place a module's
output name is decided". It refuses three classes:

- names containing whitespace, quotes or commas (`workflow.rs:243`);
- a name that collides with `crate::equations::STANDARD_COLUMNS`, refused as **shadowed**
  (`workflow.rs:250-257`);
- two outputs of the same run resolving to one name (`workflow.rs:258-264`).

The shadowing refusal is worth naming, because the reasoning at `equations.rs:290-295` is the
strongest as-built statement of `SB-CORE-002` in this domain: a computed curve written under a
standard column's name "is written, counted and reported — and then invisible, because
`fetch_curve_frame` hands every reader the raw standard column instead", i.e. "a run that reports
success and a project that holds a curve nothing can open".

`build_opts` (`workflow.rs:276-295`) composes manifest defaults, user overrides and `__IN_<arg>`
resolved mnemonics, uppercased. The reserved keys are declared: `ZONE_INDEX_ARG = "__ZONE_INDEX"`
(`workflow.rs:186`), `OUT_PREFIX_OPT` (`:195`), `OUT_NAME_PREFIX = "__OUT_"` (`:204`).

**Status: `PRESENT-OK`.**

### 3.8 The model store — the `SB-CORE-014` audit — `PARTIAL`

`ml_models` (`db.rs:675-692`) holds `model_id`, `name`, `task`, `algorithm`, `feature_curves`,
`target_curve`, `params_json`, `metrics_json`, `trained_on`, `n_train`, `standardize`,
`sklearn_version`, `note`, `created_at`, `data BLOB`, keyed on `model_id`.

What it gets right, and it gets a lot right:

- `feature_curves` is an ordered JSON array and the comment states "ORDER IS PART OF THE CONTRACT"
  (`db.rs:669-671`); applying resolves exactly those curves in that order and **fails a well by name
  when one is missing** rather than substituting or reordering.
- `data` is a joblib dump of **both** the scaler and the estimator, and the comment gives the
  reason (`db.rs:666-668`): refitting a `StandardScaler` on the apply wells would be a different
  transform and "the predictions would be quietly wrong rather than obviously broken". That is a
  `SB-CORE-002` argument made correctly at schema-design time.
- `sklearn_version` is captured.
- The primary key is defended in the comment rather than assumed: "a duplicate would make a cited
  model ambiguous" (`db.rs:673-674`).

What it does not hold: **no training seed**; **no artifact hash**; **no depth-interval identity** —
`trained_on` is a JSON array of well *names* (`db.rs:684`), not well ids and not intervals, so a
model trained on 200–2,400 m of a well and one trained on the whole well are identical in the
record; **no log-set identity** for the training curves, so a later re-run of those inputs leaves the
model's provenance pointing at a name whose values have since changed; **no library set beyond
sklearn**, though numpy and scipy both affect numerics; and **no origin column** distinguishing a
SandiBumi-trained artifact from any other joblib blob.

**And the two write paths disagree.** Train-and-apply (`ml.rs:670-675`) writes
`module: format!("ml:{}:{}", req.task, req.algorithm)`, `params_json` = the request params,
`inputs_json` = the feature curves — **no `model_id`**, and in that path no `ml_models` row is
created at all. Apply-saved-model (`ml.rs:944-953`) writes `module: format!("ml:apply:{}", info.name)`
with `params_json` carrying `model_id`, `model_name`, `algorithm` and `trained_on`, under a comment
(`ml.rs:942-943`) that states the intent exactly: "Provenance names the MODEL, not just the
algorithm: 'which model produced this curve' is the question saving them was meant to answer."

**Status: `PARTIAL`.** The comment states the requirement and one of the two paths meets it. The
other produces a curve whose provenance names an algorithm and cannot be traced to any stored model.

### 3.9 Jobs and cancellation — `PRESENT-OK`, and the core requirement's evidence is stale

`jobs.rs` was read in full. `ItemState { Pending, Running, Ok, Warned, Failed }` (`jobs.rs:28-34`);
`JobPhase { Queued, Running, Completed, Cancelled, Failed }` (`:53-59`).

The cancel model has two flags, not one. `JobHandle` (`jobs.rs:129-137`) holds
`cancel: Arc<AtomicBool>` **and** `observed_cancel: Arc<AtomicBool>`. `is_cancelled()`
(`jobs.rs:152-158`) records the observation as a side effect of the worker asking; `note_cancel_observed`
is at `:162-164`; `run_job` (`jobs.rs:257-299`) finalizes as `cancelled()` only
`if finalize.cancel_was_observed()`, otherwise `complete()` (`jobs.rs:286-290`). **"Cancelled"
therefore means the work actually stopped**, which is the honest half of `SB-CORE-036`.

The visible half also ships. `Job` carries `cancellable: bool` (`jobs.rs:89`, with a comment naming
it as the visible half of the cancel-honesty defect); `JobView` carries it (`jobs.rs:107`); `run_job`
takes it as an explicit parameter (`jobs.rs:266`); `run_simple_job` passes `false` (`jobs.rs:319`);
two dedicated tests exist — `cancellable_flag_reaches_the_view_both_ways` (`jobs.rs:527-539`) and
`cancel_counts_as_cancelled_only_once_a_worker_observes_it` (`jobs.rs:545-578`); and the frontend
consumes it (`src/ipc.ts:896` declares `cancellable: boolean`; `src/ui/processingPanel.ts:203` gates
the Cancel button on `if (active && job.cancellable)`).

`MAX_FINISHED = 24` (`jobs.rs:119`) bounds retained finished jobs, pruned at `jobs.rs:394-409`;
`cancel` at `:436-440`; `any_active` at `:444-446`.

**Status: `PRESENT-OK`.** This is a correction to `04_CORE_REQUIREMENTS.md`'s as-built note for
`SB-CORE-036`, which states that the job view carries no `cancellable` flag to check. That was true
when it was written and is not true of the shipped code. Per the commission I may not edit that
file; the correction is raised as **escalation 2** in §7.2. The requirement itself is unaffected —
what moved is a pointer, not an obligation.

### 3.10 The global mutex — `PRESENT-DIVERGENT`

`lib.rs:73`: `pub struct DbState(pub Arc<Mutex<Connection>>);`, managed at `lib.rs:3122`. One
connection, one writer, for the whole process.

**The single writer is correct and stays.** F-27 shows it is the vendor-corroborated design, and
DuckDB's own concurrency model makes it the right shape. What is divergent is **hold duration**,
which is what `SB-CORE-032` actually claims.

Measured on the current source by a brace-matched scan that classifies each `#[tauri::command]` and
inspects only its own body:

| | Commands | Of which take `db.0.lock()` |
|---|---|---|
| Synchronous | 130 | **109** |
| `async` | 79 | 17 |

`db.0.lock()` appears **128 times** in `lib.rs`. The audit figure `SB-CORE-032` was written against
was "64 of 82 synchronous commands"; the shape is unchanged and the count has grown. In the
module-run path the lock is taken twice — once at `workflow.rs:707` and again for the batched write
at `workflow.rs:746-747` — which is the right instinct (release between phases) applied to a path
whose second phase still scales with well count.

**Status: `PRESENT-DIVERGENT`.** Every long read and every batch write holds the process-wide lock
for its full duration, so a 540-well operation blocks a well-list refresh. `project.rs:144-146`'s own
comment records the measured consequence: a ~5-minute open on a ~540-well, ~2 GB project.

### 3.11 Well groups and scoping — `PARTIAL`

`well_groups` (`db.rs:794-811`) and `well_group_members` exist server-side, with `active` documented
as "enforced in code" rather than by a constraint. `well_pins` (`db.rs:812-818`) is separate.
`set_active_well_group` is exposed at `lib.rs:1397-1399`.

But the filter is applied in the frontend: `src/state.ts:135` exposes
`activeGroupWellIds(): Set<string> | null`, and `src/ui/wellGroups.ts:18-35` snapshots the active
group's `well_ids` for the UI to filter against.

**Status: `PARTIAL`.** The group is persisted and the active one is known to the backend; the
scoping *decision* is made on the client. A backend command that iterates wells does not consult it.
That is `SB-CORE-035` exactly, and it compounds `SB-CORE-032`: a command that should touch 12 wells
touches 540 while holding the lock.

### 3.12 The database inspector and the read-only SQL console — `PARTIAL` / `PRESENT-DIVERGENT`

`TABLE_SPECS` (`db.rs:3476-3495`) is the inspector's whitelist and it contains **eight** tables:
`wells`, `standard_curves`, `computed_curves`, `tops`, `zones`, `zone_params`, `core_data`,
`aux_data`. The schema declares **33 or more**. `log_sets`, `computed_curves_archive`, `ml_models`,
`curve_meta`, `curve_samples`, `well_groups`, `documents`, `equations`, `well_surveys` and the rest
are not inspectable.

`get_table_page` (`db.rs:3511-3564`) clamps `limit` to `1..2000`, issues a real `COUNT(*)`, and
returns `TablePage { columns, rows, total_rows, truncated }` (`db.rs:3497-3507`) — with `truncated`
**always false** on this path, correctly, because `total_rows` is the true total.

`run_readonly_query` (`db.rs:3566-3606`) accepts only `SELECT` or `WITH`, refuses multiple
statements, clamps `limit` to `1..5000`, and uses a `LIMIT+1` probe to set `truncated`. Here
`total_rows` is **the number of rows returned, not the true total** — the same field name carrying a
different meaning than it does one function above.

**Status: `PARTIAL`** for the inspector — the whitelist omits every provenance table, which is
precisely the set a user would want when auditing a curve — and **`PRESENT-DIVERGENT`** for
`total_rows`, whose meaning depends on which command produced the `TablePage`. The magnitude is
small; the class is `SB-CORE-002`, a page size displayed as a total.

### 3.13 Processing history and undo — frontend-only — `PARTIAL`

Processing history (`src/processLog.ts`) is `ProcessEntry { ts, kind, detail, well }`, capped at
5,000 entries with the oldest rolling off (`src/processLog.ts:22`, `:44`), debounced 600 ms
(`:34-39`) and persisted **as one JSON string into the `documents` table** under `doc_type
"history"`, `name "log"` (`:20-21`, `:37`). `documents` (`db.rs:730-737`) is keyed
`PRIMARY KEY (doc_type, name)`. `src/ui/historyPanel.ts` renders it; `processLogToText`
(`src/processLog.ts:84-90`) exports it as plain text.

Undo (`src/undo.ts`) is frontend-only, capped at 100 actions (`:12`, `:41`), **not persisted**, and
cleared on project switch with a stated reason: "replaying one would mutate the newly opened
database with the old project's values" (`src/undo.ts:30-31`). It carries two genuinely good
safeguards documented at `:46-59` — the stacks change only after a reversal resolves, and a rejected
reversal is pushed back rather than vanishing from both stacks; and requests are serialized so a
held Ctrl+Z cannot run two reversals against the single writer at once.

**Status: `PARTIAL`.** The history is a **text log**, which is F-01's shape and not F-02's fix:
`detail` is a human-readable string, so no diff, filter or join is possible over it, and the whole
log is one row that must be rewritten on every append. It is also lossy by design at 5,000 entries —
a 2,000-well batch run can exhaust that in one operation.

### 3.14 A live `SB-CORE-007` instance inside the data model — `PRESENT-DIVERGENT`

`equations.rs:299` declares:

```rust
pub(crate) const STANDARD_COLUMNS: [&str; 7] = ["DEPTH", "GR", "RES_DEEP", "NPHI", "RHOB", "DT", "SP"];
```

with a doc comment at `:296-298` reading "**ONE list**, consulted by
`crate::workflow::resolve_output_names` before a run writes anything. It lived in `condition.rs` and
again in `frame.rs`, which is two places for a seventh standard column to be forgotten."

`curve_edit.rs:81-88` declares:

```rust
const STANDARD_COLUMNS: &[(&str, &str)] = &[
    ("GR", "gr"), ("RES_DEEP", "res_deep"), ("NPHI", "nphi"),
    ("RHOB", "rhob"), ("DT", "dt"), ("SP", "sp"),
];
```

Different type, different arity, different membership — no `DEPTH` — and **no test asserts that the
two agree**. The consolidation the first comment celebrates was undone by a later file, and the
comment still says there is one list.

**Status: `PRESENT-DIVERGENT`.** The magnitude today is contained: `curve_edit.rs`'s list maps
mnemonics to `standard_curves` column names for the edit path, and omitting `DEPTH` is arguably
correct there because depth is not editable as a curve. The magnitude tomorrow is the one the
comment predicts. Adding an eighth standard column — a caliper, a PEF — updates one list and
silently not the other, and the failure surfaces as a curve that is editable but not
shadow-protected, or protected but not editable. This is the natural place to make the class
impossible rather than to fix the instance, and `SB-DBM-023` says how.

### 3.15 Status summary

| # | Area | Status | `file:line` |
|---|---|---|---|
| 3.1 | Format-version gate | `PRESENT-OK` | `db.rs:29`, `:36-42`, `:117-167` |
| 3.2 | Connection tuning, WAL resilience, boot notes | `PRESENT-OK` | `db.rs:54-74`, `:96-106`, `:177-199` |
| 3.3 | PK-less `computed_curves` + write discipline | `PRESENT-DIVERGENT` | `db.rs:292-305`, `:958-1002` |
| 3.4 | Log-set versioning + append-only archive | `PRESENT-OK` | `db.rs:310-341`, `equations.rs:609-672` |
| 3.5 | Provenance record contents | `PARTIAL` | `workflow.rs:694-706`, `equations.rs:1231-1238` |
| 3.6 | Backup, migration, engine copy, compaction | `PRESENT-OK` | `db.rs:908-956`, `project.rs:137-217` |
| 3.7 | Output-name resolution | `PRESENT-OK` | `workflow.rs:206-268` |
| 3.8 | ML model store + apply-path provenance | `PARTIAL` | `db.rs:675-692`, `ml.rs:670-675`, `:944-953` |
| 3.9 | Job model and honest cancellation | `PRESENT-OK` | `jobs.rs:89`, `:107`, `:286-290`, `:527-578` |
| 3.10 | Global mutex hold duration | `PRESENT-DIVERGENT` | `lib.rs:73`, `workflow.rs:707`, `:746-747` |
| 3.11 | Well-group scoping | `PARTIAL` | `db.rs:794-811`, `src/state.ts:135` |
| 3.12 | Inspector whitelist / `total_rows` meaning | `PARTIAL` / `PRESENT-DIVERGENT` | `db.rs:3476-3495`, `:3511-3606` |
| 3.13 | Processing history and undo | `PARTIAL` | `src/processLog.ts`, `src/undo.ts` |
| 3.14 | Two `STANDARD_COLUMNS` | `PRESENT-DIVERGENT` | `equations.rs:299`, `curve_edit.rs:81-88` |
| — | Structured audit entries | `ABSENT` | no table |
| — | Parameter source strings | `ABSENT` | no column |
| — | Method-derivation citation | `ABSENT` | no column |
| — | Seed record for stochastic runs | `ABSENT` | no column in `log_sets` |
| — | Referential-integrity checker | `ABSENT` | — |
| — | Sampling-style verification on ingest | `ABSENT` | — |
| — | Categorical curve type | `ABSENT` | every curve store is `FLOAT` |
| — | Depth-datum declaration | `ABSENT` | `wells.depth_unit` only (`db.rs:204-225`) |
| — | Model-artifact origin / custody | `ABSENT` | `ml_models` has no origin column |
| — | Content-hash compute cache | `ABSENT` — parked | see §7.1 item 6 |

---

## 4. Requirements

Forty-three requirements in six groups. Group A is the chapter's spine: it is the mechanism by which
`SB-CORE-010` and `SB-CORE-004` become true rather than aspirational, and it is where the product's
central claim is either discharged or quietly abandoned.

A note on how these are written. Several of them extend a mechanism that already ships and ships
well — `log_sets` is a better provenance object than any of the three incumbents has. The
requirements are still written as full obligations rather than as deltas, because a delta against a
schema is not testable and a schema is what a future reader will hold in their hand.

### 4.A The provenance record

#### SB-DBM-001 — One run record per computed curve, resolvable in one hop [P0] [status: PARTIAL]

**Requirement.** Every value in a computed-curve store MUST carry a non-NULL reference to exactly
one run record, and that reference MUST resolve without ambiguity to the run event that produced it.
A store that admits a value with no run record MUST treat that value as **legacy**, MUST label it as
such wherever it is displayed or exported, and MUST NOT present it as provenanced.

**Rationale.** `SB-CORE-010` requires every computed curve to answer "how was I made?" The shipped
schema answers it for rows written since log-set versioning landed and answers nothing for rows
written before — `set_id` is nullable and the comment states NULL means legacy or unversioned
(`db.rs:307-310`). An unlabelled NULL is worse than a missing feature: it makes a partially
provenanced project look uniformly provenanced. F-01 shows no incumbent offers even this much, so
the requirement is not parity work; the label is what makes the claim honest at the boundary.

**As-built.** `PARTIAL` — `db.rs:310` (`set_id` nullable by migration design), `equations.rs:626-672`
(every current write path tags it). The gap is the absence of a **label** on the untagged remainder,
not the absence of the mechanism.

**Verified by.** SB-DBM-T03, SB-DBM-T10

---

#### SB-DBM-002 — The run record pins module identity by version, not by name [P0] [status: ABSENT]

**Requirement.** The run record MUST identify the code that produced the curve by a value that
changes when the code changes: a module identifier **plus** a version, where the version is derived
from the module's own build artefact rather than hand-maintained. A module name alone MUST NOT be
accepted as module identity.

**Rationale.** `SB-CORE-011` requires a project to re-run byte-identically. That is unachievable
while the record says only which module *name* ran: two rows recording `porosity` with identical
`params_json` are indistinguishable in the store and may differ in the number if the module changed
between them. F-06 is the incumbent evidence that versioning is the correct shape — Geolog versions
every log intrinsically and resolves an unversioned reference to the latest — and the same logic
applies one level up, to the code.

**As-built.** `ABSENT` — `workflow.rs:694-706` writes `module: req.module.clone()`, a name. The
equation path writes `module: format!("equation:{}", name)` (`equations.rs:1231-1238`), also a name,
and the equation body is user-authored and mutable, which makes the gap larger there than for a
built-in module.

**Verified by.** SB-DBM-T04, SB-DBM-T15

---

#### SB-DBM-003 — Every petrophysical parameter in a run record carries a source string [P0] [status: ABSENT]

**Requirement.** Every numeric petrophysical parameter recorded in a run record MUST carry a
`source` field alongside its value. `source` MUST be a specific, checkable string. A parameter
supplied with no source MUST be recorded with `source = NULL` **and** an explicit state of
`REQUIRED_UNSET`, which is a first-class, legal, queryable state; a silently defaulted value with an
empty source MUST NOT be representable. The store MUST be able to answer "list every parameter in
this project whose source is unset" as a query, not as a scan.

**Rationale.** This is `SB-CORE-004`, and F-01 establishes that **none of the three incumbents can
do it**: IP's six flat history columns, Techlog's three-field `HistoryItem` and Geolog's
Location/Mode/Unit/Name/Value details table all record the value and omit the origin. It is
therefore the single highest-leverage requirement in this chapter — one column and one enforcement
rule buys a capability no competitor has. F-11 supplies the type-level argument: Geolog separates
"no data" from "no parameter supplied" at the constant level (`MISS_INT −2147483647` vs
`PAR_DEFAULT_NONE_INT −2147483646`, T1 `cgg.h`, `parameters.h`), and collapsing the two is exactly
how a silent default becomes indistinguishable from a deliberate abstention.

**Betters:** IP records parameter values in `IPDBWellxxxx.history` with no source field and can
compare two parameter states only by shelling out to third-party ExamDiff (T2 `O` §3.5, §10.11).
SandiBumi's record answers "where did this `m` come from" as a column.

**As-built.** `ABSENT` — `workflow.rs:694-706` writes `params_json` as a value map with no source
field; there is no column and no state.

**Verified by.** SB-DBM-T05, SB-DBM-T09, SB-DBM-T30

---

#### SB-DBM-004 — The run record stores the effective parameter set, not only the overrides [P0] [status: PARTIAL]

**Requirement.** The run record MUST store the **effective** value of every parameter the module
consumed, including those left at a manifest default, together with a flag distinguishing
`EXPLICIT` from `DEFAULTED`. A parameter that was defaulted MUST record the manifest version the
default came from.

**Rationale.** Recording overrides only means a run's record is a diff against a moving baseline.
When a manifest default changes — and manifests are edited — every historical record silently
re-interprets. The failure is `SB-CORE-011`'s: the record reads as though nothing changed and the
re-run produces a different number. F-18 is the structurally identical failure one level down, and
it is documented as having happened: ledger R-10's ClayVol case, where a parameter addressed by a
handle that had moved bound silently to a different parameter.

**As-built.** `PARTIAL` — `build_opts` (`workflow.rs:276-295`) composes manifest defaults with user
overrides into the opts map that the module actually reads, so the effective set exists at run time;
`workflow.rs:694-706` then serialises `req.params` — the overrides — into the record. The
information is in hand and is discarded at the write.

**Verified by.** SB-DBM-T06, SB-DBM-T15

---

#### SB-DBM-005 — The run record carries a method-derivation citation, not only parameter values [P0] [status: ABSENT]

**Requirement.** A module's run record MUST carry, alongside its parameters, a **derivation
citation** for the method itself: the primary source from which the method was derived — a
literature citation, a specification, a patent number, or an explicit `FIRST-PRINCIPLES` marker
naming the derivation held in the module's own documentation. The citation MUST be recorded per run,
from the module's registered metadata, so that it travels with the numbers rather than living only
in a repository the deliverable's reader cannot see. A module registered with no derivation citation
MUST fail registration, not run and record nothing.

**Rationale.** `CONTRACT.md` §2.2 as amended permits an independently-derived capability and
prohibits a reconstruction. Those two produce the same numbers in the same columns. **What
distinguishes them, in front of an auditor years later, is whether the primary source is written
down and travels with the result.** A method whose derivation is recorded in the project database is
checkable by anyone who opens the file; a method whose origin is remembered by nobody is, on the
evidence available, indistinguishable from a reconstruction — regardless of how it was actually
built.

This is what makes the whole C-2 class in §7.4 safe to build at all, and it is why the requirement
sits here rather than in a method chapter: a per-chapter promise to cite is a documentation
practice, and a column is a contract. It also generalises `SB-CORE-004` correctly — that requirement
says a *parameter* carries its source; this one says the *method* does. A project that records
`m = 1.85, source: "core SCAL report §4"` and cannot say where the Archie form itself came from has
provenanced the input and not the transform.

**Betters:** No incumbent records method provenance at all. IP's history records the module name and
its parameters (T2 `O` §3.5); Techlog's `HistoryItem` is `dateTime`/`userName`/`description` (T1
`Techlog/Data.py:586-618`); Geolog's Details rows are Location/Mode/Unit/Name/Value (T2
`AuditTrail/audit_trail_hc.1.05.html`). In all three, "which paper is this equation from" is
answerable only from the vendor's manual, if at all — and for a vendor's proprietary variant, not at
all.

**As-built.** `ABSENT` — no column in `log_sets` (`db.rs:316-329`), and no field in `LogSetSpec`
(`equations.rs:586-593`).

**Verified by.** SB-DBM-T07, SB-DBM-T10

---

#### SB-DBM-006 — Inputs are recorded as resolved identities, with the rule that chose them and the candidates it rejected [P0] [status: PARTIAL]

**Requirement.** For each module input the run record MUST store the tuple
`(input_slot, chosen_curve_id, rule, rejected_candidates)`, where `chosen_curve_id` identifies a
specific stored curve **and its set version**, `rule` names the resolution stage that selected it
from a declared vocabulary, and `rejected_candidates` lists the identities that were considered and
not chosen. Recording the requested mnemonic alone MUST NOT satisfy this requirement.

**Rationale.** F-04, quantified: in a well with three GR curves across two sets and one flagged
Final — an ordinary situation after a re-log — IP's five-stage resolution chain has three plausible
answers and records none of them, and Geolog's priority rule (default set → `setinfo` order → latest
version → alias order, T2 `…hc.2.15.html`) is equally silent. The deliverable shows one number and
*"which GR did this Vsh actually use"* is unanswerable. The chain itself is good and is adopted; what
is added is the record of its outcome. `rejected_candidates` is the part that is easy to drop and
should not be: knowing a curve was chosen is useful, and knowing which curve it beat is what lets a
reviewer see that the wrong one was nearly used.

**Betters:** IP's chain is documented as powerful and is completely opaque at run time (T2 `O` §2.11,
§10.5); SandiBumi's records `(input_slot, chosen_curve_id, rule, candidates)` per run.

**As-built.** `PARTIAL` — `workflow.rs:694-706` writes `inputs_json` from `log_args` plus
`input_set`, i.e. requested mnemonics. `equations.rs:301-303` documents that non-standard names fall
through to `computed_curves`, so a real resolution decision is being made and discarded.

**Verified by.** SB-DBM-T08

---

#### SB-DBM-007 — A missing provenance element is a named state, never an empty string [P1] [status: PRESENT-DIVERGENT]

**Requirement.** Every provenance field MUST be either populated or set to a **named absent state**
from a declared vocabulary (`NOT_APPLICABLE`, `REQUIRED_UNSET`, `LEGACY_UNRECORDED`). An empty
string, an empty JSON object, a zero and a NULL MUST NOT be used interchangeably to mean "absent",
and a reader MUST be able to distinguish "this run had no parameters" from "this run's parameters
failed to serialise".

**Rationale.** F-11 is the vendor precedent — Geolog spends two adjacent integer constants in two
different headers to keep "no data" and "no parameter supplied" apart. The shipped code already has
the failure this prevents: the equation path writes `params_json: String::new()`
(`equations.rs:1231-1238`), an empty string, which is indistinguishable from a serialisation failure
and parses as neither an object nor a null. `SB-CORE-002` is the governing principle — a degraded
record presented as a clean one.

**As-built.** `PRESENT-DIVERGENT` — `equations.rs:1231-1238` (empty string), `db.rs:316-329`
(`params_json` and `inputs_json` are both nullable `VARCHAR` with no declared absent vocabulary).

**Verified by.** SB-DBM-T09

---

#### SB-DBM-008 — The run record names the operator and the zone set in force [P2] [status: ABSENT]

**Requirement.** The run record MUST carry the identity of the person or automated agent that
initiated the run, and — where the module consumed zone-scoped parameters — the identity and version
of the zone set that was in force.

**Rationale.** All three incumbents carry a user field (IP's `User_Name`, Techlog's `userName`,
Geolog's Well History `User`; dossier §2.8) and SandiBumi carries none. The zone-set half is the more
consequential of the two: F-19 establishes that a parameter's value can be a per-zone tilt
interpolated within-zone only, so a parameter record without the zone set it was evaluated against
is not re-evaluable. P2 rather than P0 because SandiBumi is single-user on the desktop today, which
makes the operator field low-value until it is not.

**As-built.** `ABSENT` — no columns in `log_sets` (`db.rs:316-329`).

**Verified by.** SB-DBM-T11

---

#### SB-DBM-009 — Provenance timestamps are stored UTC and displayed local [P2] [status: PRESENT-DIVERGENT]

**Requirement.** Every provenance and audit timestamp MUST be stored in UTC and rendered in the
viewer's local zone. A stored local timestamp MUST NOT be accepted.

**Rationale.** T2 `AuditTrail/audit_trail_hc.1.05.html` — Geolog stores UTC and displays local, and
is the only one of the three to state a policy. The failure is narrow and real: runs produced either
side of a DST transition, or by collaborators in different zones, sort wrongly, and a provenance
record whose ordering is wrong is worse than one with no ordering because it invites a false
inference about which run came last.

**As-built.** `PRESENT-DIVERGENT` — `db.rs:324` (`created_at TIMESTAMP NOT NULL DEFAULT now()`);
DuckDB's `now()` is local.

**Verified by.** SB-DBM-T11

---

#### SB-DBM-010 — Provenance travels into the deliverable [P0] [status: ABSENT]

**Requirement.** Any export that carries computed curves MUST be able to carry their provenance with
them, in a machine-readable sidecar that resolves every curve in the export to its run record,
including the parameter source strings (`SB-DBM-003`) and the derivation citations (`SB-DBM-005`).
An export that drops provenance MUST say so at the point of export rather than silently.

**Rationale.** `SB-CORE-010`'s scope was resolved on 2026-08-07 to extend into the deliverable rather
than stopping at the UI. That resolution is what makes the requirement architectural: a provenance
record readable only inside SandiBumi is a UI feature; one that reaches the client is the product's
central claim. F-03 is the cautionary case — Geolog's audit trail exists but evaporates when the
storage backend is not EposData, documented in exactly one sentence in the entire helpset — and an
export path that silently drops provenance is the same failure with a different trigger.

The seam with `21_data-io.md` is: that chapter owns the sidecar's **format** and its placement
relative to the exported files; this chapter owns its **completeness contract** — that every curve
in the export resolves, and that a curve which cannot resolve is named.

**As-built.** `ABSENT` — no export path reads `log_sets`.

**Verified by.** SB-DBM-T10

---

#### SB-DBM-011 — Structured audit entries, as name-value pairs with a controlled vocabulary [P1] [status: PARTIAL]

**Requirement.** SandiBumi MUST record user and system actions as structured entries, not as
free text:

```
audit_entry(entry_id, well_id, ts_utc, user, view, source, comment)
audit_detail(entry_id, seq, location, mode, unit, name, value)
  location ∈ {PARAMETER, COMMENT, SET, CONSTANT, INTERVAL, LOG, ATTRIBUTE}
  mode     ∈ {INPUT, OUTPUT, DELETE, RENAME, SAVE, SAVE_AS, SAVE_CANCEL}
```

Uninterrupted repeated actions of the same type MUST collapse into one entry. A dotted `name` MUST
denote an attribute change on the named object. The audit MUST live in the project file and MUST
survive a "Save Project As".

**Rationale.** Adopted directly from Geolog's taxonomy (T2 `AuditTrail/audit_trail_hc.1.05.html`,
`…1.06.html`), which is the best-shaped of the three (F-01) and is what dossier D-14 recommends
adopting wholesale. The entry-collapsing rule is Geolog's own and exists for the
crossplot-point-dragging case, where a naive recorder produces thousands of entries for one gesture.
This sits alongside `log_sets` rather than replacing it: `log_sets` answers "how was this curve
made", the audit answers "what did someone do to this project".

**As-built.** `PARTIAL` — `src/processLog.ts` records `{ts, kind, detail, well}` where `detail` is
a human-readable string, capped at 5,000 entries, persisted as one JSON blob into `documents`
(`db.rs:730-737`). It is a text log, so no query, filter or join over it is possible, and appending
rewrites the whole row.

**Verified by.** SB-DBM-T11, SB-DBM-T12

---

#### SB-DBM-012 — A parameter-state diff is a database join, not an external differ [P2] [status: ABSENT]

**Requirement.** Comparing two parameter states MUST be answerable as a query over
`audit_detail` returning structured `(zone, parameter, old, new, unit)` rows. No external
differencing tool may be invoked, and the diff MUST be embeddable in a report.

**Rationale.** F-02: IP shells out to ExamDiff, a third-party text differ, to answer this. Because
`audit_detail` is already a name-value pair list, the same question is a `FULL OUTER JOIN` — dossier
§5.5 states it as such. This is a case where adopting the better vendor's *shape* makes the weaker
vendor's *feature* free.

**Betters:** IP's parameter differencing requires a third-party executable and returns coloured text
that cannot be filtered by magnitude or embedded in a deliverable (T2 `O` §3.5, §10.11).

**As-built.** `ABSENT` — depends on SB-DBM-011.

**Verified by.** SB-DBM-T12

---

#### SB-DBM-013 — No configuration, deployment mode or preference may disable the provenance record [P1] [status: PRESENT-OK]

**Requirement.** The provenance record MUST NOT be a feature flag. There MUST be no setting, storage
mode, licence tier or preference that causes a computed curve to be written without its run record.
Where a write of the run record fails, the curve write MUST fail with it.

**Rationale.** F-03: Geolog's audit trail "is only available when an EposData database is used",
stated on exactly one page of the whole helpset and not in the Audit Trail chapter. A user on the
wrong backend has no audit trail and no notification that they lack one. Provenance that a
deployment choice can silently remove is not provenance.

**As-built.** `PRESENT-OK` by construction — `write_computed_curves_versioned`
(`equations.rs:626-672`) performs the set creation and the curve write in the same `with_txn`, so a
failed set write rolls the curve write back, and `workflow.rs:750-762` downgrades the affected wells
to `ItemState::Failed` rather than reporting success. The requirement is written to **pin** existing
behaviour against a future configuration option, not to close a gap.

**Verified by.** SB-DBM-T13

### 4.B Reproducibility

#### SB-DBM-014 — Every stochastic operation records its seed and its seeding rule [P0] [status: PARTIAL]

**Requirement.** Any operation whose output depends on a pseudo-random sequence MUST record, in its
run record, the root seed, the **seeding rule** by which per-item seeds are derived from it, and the
generator's identity. Re-running with the recorded triple MUST reproduce the output bit for bit.

**Rationale.** `SB-CORE-011`. The Monte Carlo path already derives per-realisation seeds from
`(seed, index)`, which is the correct construction — it makes a realisation reproducible
independently of iteration order and of how many realisations ran. What is missing is that the
triple is not *recorded*, so the property is a code fact rather than a data fact, and a project file
handed to someone else does not carry it. The generalisation matters because Monte Carlo is not the
only stochastic path: model training (group C), any random sub-sampling, and any tie-break that
consults a hash all have the same exposure.

`14_cutoffs-summation-mc.md` owns the Monte Carlo method; this requirement owns the record.

**As-built.** `PARTIAL` — the `(seed, index)` derivation exists (per `04_CORE_REQUIREMENTS.md`
`SB-CORE-011`); `log_sets` has no seed column (`db.rs:316-329`), so a seed reaches the record only if
a module happens to expose it as a parameter.

**Verified by.** SB-DBM-T14, SB-DBM-T15

---

#### SB-DBM-015 — The re-run manifest is enumerated, stored, and checkable [P0] [status: ABSENT]

**Requirement.** SandiBumi MUST define and store, per run, the complete set of facts that must be
pinned for that run to reproduce byte-identically. At minimum the manifest MUST contain: module
identity and version (`SB-DBM-002`); the effective parameter set with sources (`SB-DBM-003`,
`SB-DBM-004`); resolved input curve identities including set version (`SB-DBM-006`); the depth frame
and its sampling (`db.rs:325-328`); the zone set identity where zone-scoped parameters were used
(`SB-DBM-008`); the seed triple where the run is stochastic (`SB-DBM-014`); the identity of any
learned model applied (`SB-DBM-020`); and the values, at run time, of any attribute that drives
physics (`SB-DBM-017`). A "re-run this set" operation MUST verify that every manifest element still
resolves and MUST refuse, naming the element, when one does not.

**Rationale.** `SB-CORE-011` is the product's reproducibility claim and it is not satisfiable by any
single column. Enumerating the manifest is what turns it from an aspiration into something a test
can check, and the refusal clause is what stops a re-run from silently substituting a curve that has
moved — which is F-18's failure (ledger R-10) at the level of a whole run rather than a single
parameter.

**As-built.** `ABSENT` — no manifest concept exists; the elements are individually partial or absent
as listed above.

**Verified by.** SB-DBM-T15, SB-DBM-T16

---

#### SB-DBM-016 — Re-run output does not depend on iteration order [P1] [status: PRESENT-UNVERIFIED]

**Requirement.** No stored output may depend on the iteration order of an unordered collection, on
the physical row order returned by a query without an `ORDER BY`, or on hash-map traversal. Where a
computation aggregates over wells, curves or samples, the order MUST be established explicitly.

**Rationale.** `SB-CORE-011`'s quiet failure mode. Floating-point summation is not associative, so a
mean, a standard deviation or a Monte Carlo aggregate computed over an unordered traversal is
reproducible only by luck. A DuckDB query without `ORDER BY` gives no order guarantee, and the
`computed_curves` store is deliberately PK-less (`db.rs:292-299`), so it has no clustering to fall
back on.

**As-built.** `PRESENT-UNVERIFIED` — no test exercises it. What would verify it: run the same
project twice in a process with a randomised hash seed and compare output byte for byte.

**Verified by.** SB-DBM-T16

---

#### SB-DBM-017 — A metadata attribute that drives physics is an input of the module that consumes it [P1] [status: ABSENT]

**Requirement.** Any well-, set- or curve-level attribute whose value selects an equation, a
tool-response table, a correction chart or a default endpoint MUST (a) be declared as an input of
every module that consumes it, (b) be written into that module's run record with its value at run
time, and (c) mark the run's outputs stale when it changes. A run whose physics-driving attribute is
**unset** MUST fail with a named error rather than defaulting silently.

**Rationale.** F-05, and it is the only documented metadata→physics path in the three tools. IP's
**Logging Contractor** header field selects neutron/density crossplot overlays, sets the Neutron
Tool Type for Basic Log Analysis and Mineral Solver, and selects the neutron look-up tables for
limestone→sandstone/dolomite matrix conversion and salinity correction (T2 `O` §3.3,
`wellheaderinfo.htm`) — so "a single header dropdown … silently changes numerical results". The
vendor's own ingest states the mitigation: "if an attribute drives physics, it must be surfaced in
the run record, not buried in a header tab" (T2 `O` §10 item 9). This is the never-delegate class
from the project's `CLAUDE.md`: a wrong contractor selection changes ρma and salinity look-ups, and
it computes, plots and ships.

**Betters:** IP surfaces the field in a header tab and records nothing of it in the run history; a
change silently re-selects look-up tables for every subsequent run (T2 `O` §3.3).

**As-built.** `ABSENT` — `wells` (`db.rs:204-225`) carries `depth_unit`, `surface_x/y` and
`utm_zone`; no attribute is declared as physics-driving, and `inputs_json` (`workflow.rs:694-706`)
holds curve mnemonics only.

**Verified by.** SB-DBM-T17

### 4.C The learned-model store

#### SB-DBM-018 — Training-set identity is recorded as ids and intervals, not as names [P0] [status: PARTIAL]

**Requirement.** A stored model's training set MUST be identified by stable well ids, the depth
interval used per well, and the **log-set version** of each training curve. Well names MUST NOT be
the identity. A model whose training-set identity cannot be resolved against the current project
MUST be reported as unresolvable at apply time and MUST NOT be applied silently.

**Rationale.** `SB-CORE-014` requires a learned model to carry its training provenance, and the
enumeration is none of a parameter, an input curve, or a module version — so the `SB-CORE-010`
schema in group A does not hold it. `ml_models.trained_on` is a JSON array of well *names*
(`db.rs:684`). Three failures follow: a renamed well breaks the link with no error; a model trained
on 200–2,400 m of a well and one trained on its full 3,100 m are identical in the record; and a
re-run of a training curve under `SB-DBM-001`'s versioning leaves the model pointing at a name whose
values have changed underneath it. F-06's version-in-the-reference principle is the fix.

**As-built.** `PARTIAL` — `db.rs:684` (`trained_on VARCHAR NOT NULL, -- JSON array of well names`),
`db.rs:685` (`n_train`, a count with no interval).

**Verified by.** SB-DBM-T18, SB-DBM-T20

---

#### SB-DBM-019 — A stored model carries its seed, its full library set and an artifact hash [P0] [status: PARTIAL]

**Requirement.** `ml_models` MUST additionally record: the training seed and generator identity
(per `SB-DBM-014`); the versions of **every** library whose numerics affect the fit, not only
scikit-learn; and a cryptographic hash of the serialised artifact, computed at write and verified at
load. A hash mismatch at load MUST be a named error, never a warning.

**Rationale.** `SB-CORE-014` and `SB-CORE-011` together. Without a seed, a "re-train and compare"
check cannot distinguish a real difference from a different random initialisation — which is the
whole point of the check. Without the full library set, a numpy or scipy upgrade changes predictions
with no recorded cause; `sklearn_version` alone (`db.rs:687`) records one of at least three. The
hash is what makes `SB-DBM-021`'s custody rule enforceable rather than declaratory, and it is what
detects a corrupted or hand-edited blob before it produces numbers.

**As-built.** `PARTIAL` — `db.rs:687` (`sklearn_version`) is the only library recorded; no seed
column; no hash column.

**Verified by.** SB-DBM-T19, SB-DBM-T21

---

#### SB-DBM-020 — Both apply paths stamp the model identity into the produced curve's provenance [P0] [status: PARTIAL]

**Requirement.** Every curve produced by a learned model MUST carry that model's `model_id` in its
run record, and the `model_id` MUST resolve to a row in `ml_models`. A path that trains and applies
in one operation MUST persist the model and stamp its id exactly as the apply-saved-model path does;
it MUST NOT record the algorithm name in place of a model identity.

**Rationale.** The shipped code states the requirement and half-meets it. `ml.rs:942-943` reads:
"Provenance names the MODEL, not just the algorithm: 'which model produced this curve' is the
question saving them was meant to answer." The apply-saved-model path honours it (`ml.rs:944-953`,
`params_json` carrying `model_id`, `model_name`, `algorithm`, `trained_on`). The train-and-apply path
does not (`ml.rs:670-675`, `module: format!("ml:{}:{}", req.task, req.algorithm)` and no
`model_id`) — and in that path no `ml_models` row is created at all, so the model that produced the
curve does not exist after the run. Two paths, two provenance shapes, one kind of number: this is
`SB-CORE-007`'s failure applied to a record rather than a constant.

**As-built.** `PARTIAL` — `ml.rs:670-675` (no model id), `ml.rs:944-953` (model id present).

**Verified by.** SB-DBM-T20

---

#### SB-DBM-021 — Model artifacts are native-only; a foreign artifact is refused at the store boundary [P0] [status: ABSENT]

**Requirement.** `ml_models.data` MUST hold only an artifact produced by SandiBumi's own training
path. The table MUST carry an `origin` column, populated at write from the training path itself and
never from user input, and the apply path MUST refuse any row whose `origin` is not native, naming
the row. There MUST be no import path — no command, no file dialog, no SQL — by which an externally
produced model artifact enters this table. A vendor-trained model MUST NOT be consumed in any
format.

**Rationale.** `CONTRACT.md` §2.2 class **C-3**: shipped neural-network weight files are opaque
artifacts; there is nothing to derive from, inferring their internals from behaviour is the
prohibited path, and "a vendor-trained model is never consumed in any format". That bar is a
constraint on **this schema**, because `ml_models.data BLOB` (`db.rs:690`) is precisely the shape
that would accept one. The requirement converts a policy into an enforced column, which is the only
form in which it survives a future contributor who has not read the contract.

The technical argument is the same one `03_EVIDENCE_BASE.md` §14.1 makes: a model derived from a
vendor's implementation inherits that implementation's defects, and the product's competitive claim
rests on not inheriting them.

**Betters:** the incumbents ship pre-trained weight files whose training data, seed and library
versions are not disclosed and whose numerics therefore cannot be audited or reproduced by the user
who ships a deliverable from them. Every SandiBumi model is trained in the user's own project from
identified wells (`SB-DBM-018`) under a recorded seed (`SB-DBM-019`), so its provenance is complete
by construction rather than by vendor disclosure.

**As-built.** `ABSENT` — `db.rs:675-692` has no `origin` column. No import path exists today, which
is the right state; the requirement is to make that a property of the schema rather than of the
current command surface.

**Verified by.** SB-DBM-T21

---

#### SB-DBM-022 — The feature-vector order contract is verified at apply time, not assumed [P1] [status: PRESENT-UNVERIFIED]

**Requirement.** At apply time the resolved feature vector MUST be checked against the stored
`feature_curves` array for both membership and **order**, and a mismatch MUST fail the well by name.
The check MUST NOT be satisfied by set equality.

**Rationale.** `db.rs:669-671` states the contract — "`feature_curves` is an ORDERED JSON array and
is the contract: applying the model resolves exactly those curves in exactly that order, and fails a
well by name when one is missing rather than substituting or reordering" — and the reasoning is
right: a scaler and an estimator fitted on `[GR, RHOB, NPHI]` applied to `[GR, NPHI, RHOB]` produces
numbers, not an error. F-18's ClayVol case (ledger R-10) is the same failure in the parameter
domain, and it happened.

**As-built.** `PRESENT-UNVERIFIED` — the contract is documented at `db.rs:669-671` and the code
resolves in order; no test asserts that a reordered vector is refused rather than accepted.

**Verified by.** SB-DBM-T22

### 4.D One definition, one place — making the class impossible

`SB-CORE-007` is verified live in this product (§3.14). A data model is the natural place to make
this class of defect impossible rather than to fix its instances, because the mechanism that
prevents it — a single registered source of truth that every consumer must go through — is a schema
pattern, not a code review habit.

#### SB-DBM-023 — Schema-level vocabularies live in one registry, and every consumer resolves through it [P1] [status: PRESENT-DIVERGENT]

**Requirement.** Every vocabulary that describes the shape of the store — the standard column set,
the sampling-style enumeration, the frame enumeration, the depth-datum enumeration, the audit
`location` and `mode` vocabularies, the absent-state vocabulary of `SB-DBM-007` — MUST be declared
in exactly one place, exported from there, and consumed by every reader and writer through that
export. A second literal declaration of a registered vocabulary MUST fail the build or a test, not
a review. Where a consumer needs a projection of a vocabulary — a subset, or a mapping to storage
column names — that projection MUST be **derived** from the registered vocabulary rather than
re-typed, so that adding a member updates every projection.

**Rationale.** `SB-CORE-007`, with a live instance in this domain. `equations.rs:296-298` documents
a consolidation — "**ONE list**, consulted by `crate::workflow::resolve_output_names` before a run
writes anything. It lived in `condition.rs` and again in `frame.rs`, which is two places for a
seventh standard column to be forgotten" — and `curve_edit.rs:81-88` re-created the second place
with a different type, a different arity and different membership. Nothing failed, because nothing
checks.

The requirement is stated as **derivation, not agreement**, deliberately. A test that asserts two
lists agree is one more thing to remember when a third list appears. A projection computed from the
registered vocabulary cannot drift, and `curve_edit.rs`'s mnemonic→column mapping is exactly such a
projection: `("GR", "gr")` is the registered name and its lowercased storage column, minus `DEPTH`,
and both operations are expressible.

**As-built.** `PRESENT-DIVERGENT` — `equations.rs:299` (7 entries, `[&str; 7]`, includes `DEPTH`) and
`curve_edit.rs:81-88` (6 entries, `&[(&str, &str)]`, excludes `DEPTH`). No test relates them.

**Verified by.** SB-DBM-T23

---

#### SB-DBM-024 — Every capacity limit is unit-typed, carries a source string, and is the source of its own documentation [P2] [status: ABSENT]

**Requirement.** Every capacity, tolerance and limit constant MUST be declared as a unit-typed
quantity with an attached source string. No limit may appear in user-facing documentation as a
hand-typed number; the documentation table MUST be generated from the declarations.

**Rationale.** F-23 is the seven-year cautionary case (ledger `O-8.3`): a vendor published a shipped
artefact count as a capacity limit, bumped it once, and never reconciled it again. F-24 is why the
unit-typing half is not cosmetic: IP's Irregular Set Tolerance is published as "0.2 ft" from a
raster dialog, and whether that is fixed feet or the well's own depth unit is unresolved
(`OPEN-DB-4`) — a 3.28× difference that on a 0.1524 m workflow is the difference between 40 % of a
step and 1.3 steps, i.e. between a sane snap and silently consuming a sample. Storing it unit-typed
means a later resolution changes one constant rather than a code path. `FINDINGS.md` §6 rules 3
and 10.

**As-built.** `ABSENT` — the limits that exist are bare literals: `MAX_FINISHED = 24`
(`jobs.rs:119`), the `1..2000` and `1..5000` clamps (`db.rs:3511-3564`, `:3566-3606`), `LIMIT = 5000`
and `LIMIT = 100` in the frontend (`src/processLog.ts:22`, `src/undo.ts:12`), the recents
`truncate(12)` (`project.rs:113`).

**Verified by.** SB-DBM-T24

---

#### SB-DBM-025 — A constant that crosses a module boundary is registered with its source [P2] [status: ABSENT]

**Requirement.** Any petrophysical constant referenced by more than one module — a matrix density, a
fluid property, a unit conversion factor — MUST be held in a registry that carries its value, its
unit and its source string, and modules MUST resolve it from the registry. A module MUST NOT declare
its own literal for a registered constant.

**Rationale.** `SB-CORE-007` again, one level up from `SB-DBM-023`: that requirement governs
vocabularies that describe the store, this one governs numbers that describe rock. The
project-standing rule applies without exception — a constant is cited or it is absent, never
inferred and never carried over from a neighbouring vendor. Registering it is what makes the
citation travel to `SB-DBM-003`'s source field automatically rather than by the interpreter
retyping it.

This requirement specifies the **mechanism only**. It does not set any value; the method chapters
own every number that would populate the registry, and this chapter ships no petrophysical constant
of its own (§5).

**As-built.** `ABSENT` — no constants registry exists.

**Verified by.** SB-DBM-T23, SB-DBM-T24

### 4.E Store integrity

#### SB-DBM-026 — Two samples may not share a depth in one curve, and the resolution is declared [P1] [status: PRESENT-DIVERGENT]

**Requirement.** For a set declared `CONTINUOUS_REGULAR` or `CONTINUOUS_IRREGULAR`, a write of two
samples at the same `(well_id, curve_name, depth)` MUST be refused with a named error identifying
the depth and both source rows. For a set declared `POINT`, duplicate depths are legitimate and MUST
be accepted; if the configured resolution is perturbation, the offset MUST be a declared unit-typed
constant and MUST be logged per row. A silent survivor MUST NOT be possible in any set type.

**Rationale.** F-26 — IP publishes 0.01 ft for duplicate FPRESS depths (T2 `O` §4.6), which
establishes the class of the decision rather than the constant. The interaction with §3.3 is what
makes this urgent rather than theoretical: `computed_curves` has **no primary key by design**
(`db.rs:292-305`, for a measured 3.7× insert gain), so nothing in the engine enforces uniqueness and
the write discipline is asserted by a comment. Dossier invariant 12 names the interaction. The
consequence of a duplicate is last-writer-wins on any read that assumes one row, and a double count
in any summation over that curve.

**As-built.** `PRESENT-DIVERGENT` — the discipline is documented (`db.rs:296-299`) and implemented in
the current writers (`equations.rs:626-672`); nothing enforces it and no test would catch a writer
that skipped the DELETE.

**Verified by.** SB-DBM-T25, SB-DBM-T26

---

#### SB-DBM-027 — A referential-integrity checker exists, reports every dangling class by name and count, and never reports "clean" without checking [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST provide a checker that reports, by class, name and count: rows in
`computed_curves` or `computed_curves_archive` whose `set_id` resolves to no `log_sets` row;
`well_group_members` whose well no longer exists; `curve_samples` orphaned from `curve_meta`;
`ml_models` whose `trained_on` wells cannot be resolved; and duplicate `(well_id, curve_name, depth)`
tuples per `SB-DBM-026`. It MUST offer a prune, and it MUST report each class explicitly — including
classes with zero findings — so that "clean" is a positive result rather than an absence of output.

**Rationale.** Dossier D-25, with Geolog's `well_include_check` as the shape precedent: it reports
both the missing-project and missing-well classes rather than one (T2
`database_04_utilities_hc.4.8.html`). This is the enforcement half that §3.3's write discipline
needs to become real, and it is also `SB-DBM-001`'s detector: the checker is what finds the
`set_id IS NULL` remainder that must be labelled legacy.

**As-built.** `ABSENT` — no checker. `db.rs:315` notes that deleting a set version leaves current
values with `set_id` going NULL, which is a designed dangling state with no reporter.

**Verified by.** SB-DBM-T26

---

#### SB-DBM-028 — A declared sampling style is verified against the reference column on ingest, and the verdict is stored [P0] [status: ABSENT]

**Requirement.** On ingest, a set declared `CONTINUOUS_REGULAR` MUST have its reference column
checked for a constant increment within the declared tolerance. A contradicted declaration MUST be
stored as `CONTINUOUS_IRREGULAR` with a named ingest warning identifying the gap depth and the row
count. The declaration MUST NOT be honoured silently, and a frame-indexed read MUST NOT be permitted
on a set whose style has not been verified.

**Rationale.** F-14, and it is the worst failure in this dossier because it has **no visual
signature**. Geolog documents its own inability to detect it: there is "no way … to detect whether
the data set … is periodic or aperiodic other than by checking for a constant depth sample
increment" (T2 `database_05_database_access_hc.5.11.html`). A set declared regular at 0.1524 m
containing a 40-row gap places every post-gap sample **6.1 m shallow** on a frame-indexed read. The
curve plots continuous, correlates against a neighbour as a real structural observation, and nothing
anywhere reports an error. Dossier §5.8 ranks the check with the null screen as small, local and
high-value: one pass over the reference column at import.

**Betters:** Geolog documents that it cannot detect this and leaves the check to the user (T2
`…hc.5.11.html`); SandiBumi performs it at ingest and contradicts the declaration rather than
honouring it.

**As-built.** `ABSENT` — `log_sets.frame` declares `STANDARD` or `OWN` (`db.rs:325-328`) and no
sampling-style column exists at all; there is nothing to verify against yet.

**Verified by.** SB-DBM-T27

---

#### SB-DBM-029 — A module never writes to the reference column of a frame it reads [P1] [status: PRESENT-UNVERIFIED]

**Requirement.** The module-output path MUST refuse, at the API boundary and naming the frame, any
write into the depth/reference column of an existing frame. A module needing a different depth basis
MUST emit a **new** frame declared `OWN`.

**Rationale.** F-16 — IP's User App API states it flatly: "never write back to the Depth curve"
(T2 `O` §2.5). The failure it prevents is silent and total: editing the depth column of a shared
frame re-datums every other curve on that frame at once, and the result plots as a perfectly
continuous log. It is `SB-DBM-028`'s failure with a larger blast radius.

**As-built.** `PRESENT-UNVERIFIED` — `resolve_output_names` (`workflow.rs:206-268`) refuses a name
colliding with `STANDARD_COLUMNS`, and `DEPTH` is a member of that list (`equations.rs:299`), so the
refusal happens today as a side effect of the shadowing rule. Nothing states it as its own contract,
and `curve_edit.rs`'s list — the second `STANDARD_COLUMNS`, §3.14 — does **not** contain `DEPTH`, so
the protection depends on which list a given path consults. `SB-DBM-023` removes that dependency.

**Verified by.** SB-DBM-T28

---

#### SB-DBM-030 — Null discipline: a threshold, not an equality; and "no value" is not "no parameter" [P0] [status: ABSENT]

**Requirement.** The large-negative null family MUST be detected by threshold — `v < −1.0e29` — and
never by equality against a magnitude. The threshold MUST be **computed** from the same constant the
export path would emit rather than hard-coded as a decimal literal. Separately, "measurement absent"
and "parameter not supplied" MUST be distinguishable at every layer — store, IPC, UI and export —
and neither may be representable as a number: SQL `NULL` and absence-of-row respectively in the
store, `Option<f64>` for both in code, never a sentinel.

**Rationale.** F-12, and the boundary matters. `cgg.h`'s macro is
`#define IS_MISS_FLOAT( floatVal ) ((floatVal) < (MISS_FLOAT /10.0))` — a **strict** inequality, so
a value exactly at −1.0e29 is **data**, not null. Geolog states its own sentinel two ways —
`MISS_FLOAT = −1.0e30` in the header (T1) against −1.0D38 in the manual (T2), eight orders of
magnitude apart and unreconciled (`OPEN-DB-2`) — so an equality screen against either magnitude
leaks the other straight into the data, where it will pass any range check on a log scale and
destroy every statistic computed over the curve. Computing the bound rather than typing it matters
because `MISS_FLOAT` is a `float`-cast constant and the quotient is taken in double: a hand-typed
decimal can land on the wrong side of an exact-boundary sample. F-11 and F-13 supply the other two
halves — Geolog's deliberate `MISS_INT` / `PAR_DEFAULT_NONE_INT` separation, and Techlog's own
`MissingValue = −9999` (T1 `Techlog/Data.py:15`).

The seam with `21_data-io.md`: that chapter owns the parse-side suspect screen and the export-side
null declaration; this chapter owns the store-side rule that a suspect value is **flagged, never
silently coerced**, and the type-level separation of the two absence states.

**Betters:** Geolog ships an equality-prone documented magnitude that contradicts its own header
(T1 `cgg.h` vs T2 `…hc.5.09.html`); SandiBumi adopts the threshold form, which is correct under both
readings, and records the export-side magnitude choice in the export log.

**As-built.** `ABSENT` — no null-family screen exists in the store path.

**Verified by.** SB-DBM-T29, SB-DBM-T30

---

#### SB-DBM-031 — Every depth quantity declares its datum, and cross-datum comparison is refused [P1] [status: ABSENT]

**Requirement.** Every stored depth MUST carry a datum from `MD | TVD | TVDSS | TVDKB | TWT | OWT |
CDEPTH`. `TVDSS` is **positive down**; elevation is **positive up** from the measurement reference.
A comparison between two depths of different datums MUST be refused unless a reference frame exists
for that well, and the refusal MUST name both datums.

**Rationale.** F-17 — Geolog is the only one of the three to print its sign convention (T2
`…hc.2.12.html`); IP and Techlog state none in the pages read. Comparing an MD zone top with a
TVDSS contact without a frame is a category error that silently produces a number.

**As-built.** `ABSENT` — `wells` carries `depth_unit` (`db.rs:204-225`), a unit, not a datum;
`well_path` / `well_surveys` (`db.rs:773-792`) hold the survey from which a frame could be built.

**Verified by.** SB-DBM-T31

---

#### SB-DBM-032 — A stored parameter carries a dual handle, and a disagreement is a load failure [P1] [status: ABSENT]

**Requirement.** A persisted parameter MUST be addressed by **both** an ordinal and a semantic key.
On load, both MUST resolve to the same parameter or the load MUST fail, naming both handles. A file
carrying only one handle MUST load with a warning. Ordinals MUST be append-only and MUST NOT be
renumbered; retiring a parameter leaves a gap. Each parameter value MUST additionally carry its unit
and its `tilt` (`NONE | LINEAR | LOG`), where `tilt` is a property of the value and not a display
mode.

**Rationale.** F-18, and ledger R-10 is the case that happened: a parameter file addressed by
ordinal alone bound ordinal 41 to whatever occupied slot 41 in the loading build — one clay-volume
parameter substituted for another, and it computed. IP's own ordinals are sparse
(`1,2,3,4,5,8,9,10,11,12,14,15,16,17,19,20,23,24,25,26,32,38,…` in the NMR list, T2 `O` §2.9)
precisely because it appends rather than compacts. F-19 supplies the `tilt` half: IP's `Lg` prefix
marks a logarithmically interpolated per-zone value, interpolation is within-zone only, and the
parameter steps at a zone boundary — so a tilted `Rw` stored as a scalar has lost physics, not
formatting.

**Betters:** IP addresses parameters by ordinal alone and has no mechanism to detect a handle that
has moved; SandiBumi refuses the load rather than binding the wrong parameter (ledger R-10).

**As-built.** `ABSENT` — `zone_params` (`db.rs:378-388`) holds zone-scoped parameters; no ordinal, no
semantic-key pairing, no tilt, no per-value unit.

**Verified by.** SB-DBM-T32

---

#### SB-DBM-033 — A categorical curve is a distinct type and is never linearly interpolated [P2] [status: ABSENT]

**Requirement.** The store MUST support a `CATEGORICAL` curve type. Resampling a categorical curve
MUST round, MUST produce only values that exist in the source, and MUST report any resample that
crosses a category boundary. Arithmetic on a categorical curve MUST be refused.

**Rationale.** F-15 — Geolog states "When interpolating integer type logs, resultant values are
rounded" (T2 `…hc.2.06.html`); IP and Techlog have no categorical type (dossier §3.10). A facies
code resampled from 0.1524 m to 0.1 m under a `FLOAT` assumption yields values like 2.37, which
either round silently into a class the rock is not, or propagate as a number into a summation.
Dossier D-12 and §5.8 item 6 both rank this as the larger change to defer until a facies deliverable
needs it — with the caveat, adopted here, that no further code should assume `FLOAT` in the
meantime.

**As-built.** `ABSENT` — every value column in every curve store is `FLOAT` (`db.rs:227-239`,
`:300-305`, `:335-341`, `:760-765`).

**Verified by.** SB-DBM-T33

---

#### SB-DBM-034 — Every bulk operation returns `{matched, unmatched, ambiguous}` and drops nothing silently [P1] [status: ABSENT]

**Requirement.** Every bulk match, paste or import MUST return counts of matched, unmatched and
ambiguous rows, and every unmatched or ambiguous row MUST enter a review queue addressable by the
user. A bulk operation MUST NOT report success while having dropped a row.

**Rationale.** F-25 — all three tools have at least one documented silent-drop path: Geolog's
include-well fill-with-missing (dossier §3.9), IP's array auto-averaging (§3.8), and bulk tops paste
across all three. Ledger `O-8.8` named it; `FINDINGS.md` §6 rule 14 generalises it. The quantified
form is dossier T-DB-06: a 100-row tops paste with 3 unmatchable names and 2 ambiguous returns
`{matched: 95, unmatched: 3, ambiguous: 2}` with all five in the review queue.

**Betters:** Geolog's include-well path fills unmatched wells with missing values and reports
nothing (T2, dossier §3.9); SandiBumi returns the three counts and queues every exception.

**As-built.** `ABSENT` — no uniform bulk-result shape exists.

**Verified by.** SB-DBM-T34

---

#### SB-DBM-035 — The archive is append-only, and restoring a prior version is a first-class operation [P1] [status: PARTIAL]

**Requirement.** `computed_curves_archive` MUST be append-only: no code path may UPDATE or DELETE
from it except a version-retention policy that is explicit, user-visible and logged. Restoring a
prior version into the current store MUST be a supported operation that creates a **new** version
recording what it restored — never an overwrite of history.

**Rationale.** `SB-CORE-010` and F-06 together. The archive is what makes the "re-run = version N+1,
never overwrite" rule meaningful (`db.rs:331-334`), and an archive that can be edited is not a
record. Making a restore create a new version rather than rewind is the same principle applied to
the restore itself: a project that has been rolled back should say so.

**As-built.** `PARTIAL` — the table exists and every versioned write appends to it
(`equations.rs:626-672`); the comment states restoration as the purpose (`db.rs:331-334`); no
restore command exists and no rule prevents a future writer from deleting archive rows.

**Verified by.** SB-DBM-T35

### 4.F Concurrency, scale and honest results

#### SB-DBM-036 — No operation whose duration scales with well count holds the global lock [P1] [status: PRESENT-DIVERGENT]

**Requirement.** The single-writer connection model MUST be retained. No operation whose duration
scales with the number of wells, curves or samples may hold the global connection mutex for that
duration. Long operations MUST acquire, do a bounded unit of work, and release — or run against a
separate read connection — such that an interactive command issued during a batch run is not
blocked for more than a bounded interval.

**Rationale.** `SB-CORE-032`, whose claim is **hold duration, not concurrent writers**. F-27 is the
corroboration for keeping the single writer: Geolog's own architecture serves "a single project for
a single user, where user is defined as a single user ID on a single computer" (T2 `…hc.6.4.html`),
and with intrinsic log versioning there is no cross-user lock to document — the vendor's file-locking
chapter promises documentation it never delivers, and the substance turns out not to need it. So the
mutex is right and stays.

What is wrong is measured. 109 of 130 synchronous commands and 17 of 79 async commands take
`db.0.lock()` (§3.10); `project.rs:144-146` records a ~5-minute open on a ~540-well, ~2 GB project.
IP's `IPDBLock` self-clears in 4–5 minutes with a 5-minute grant window (T2 `O` §3.6) and is
recorded as the only lock-timeout precedent in the corpus — **not adopted**, because SandiBumi is
single-writer and a timeout would solve a problem it does not have.

**As-built.** `PRESENT-DIVERGENT` — `lib.rs:73`, `lib.rs:3122`, `workflow.rs:707`, `:746-747`.

**Verified by.** SB-DBM-T36, SB-DBM-T38

---

#### SB-DBM-037 — Well scoping is enforced in the backend, not in the client [P1] [status: PARTIAL]

**Requirement.** Where an active well group is set, every backend command that iterates wells MUST
scope its query to that group at the SQL boundary. The client MUST NOT be the enforcement point for
scope, and a command that cannot be scoped MUST declare itself project-wide rather than silently
iterating everything.

**Rationale.** `SB-CORE-035`. The group is persisted server-side (`db.rs:794-811`) and the active
one is known to the backend (`lib.rs:1397-1399`), while the filter runs on the client
(`src/state.ts:135`, `src/ui/wellGroups.ts:18-35`). The consequence compounds `SB-DBM-036`: a command
the user believes touches 12 wells touches 540, and does so while holding the global lock. F-20 is
the architectural precedent — Geolog splits the Project application from the Well application
precisely so that project-level work never materialises wells.

**As-built.** `PARTIAL` — as cited above.

**Verified by.** SB-DBM-T37

---

#### SB-DBM-038 — The interactive set is the only thing materialised [P2] [status: ABSENT]

**Requirement.** Project-level operations — the well list, faceted counts, search — MUST be answered
by query rather than by loading wells. The cost of a well-list switch, a faceted count and a first
paint MUST be `O(size of the interactive set)`, not `O(project)`. The materialised-set ceiling MUST
be **measured on SandiBumi and published**, not inherited from a vendor.

**Rationale.** F-20: the two tools that scale do so by not materialising the project, and the one
that publishes a cap (IP, 2,000 wells in memory, T2 `O` §3.1) publishes it because it does.
Techlog publishes no capacity limits at all — a verified negative across all 3,808 shipped `Doc/`
HTML pages (E-3). F-28 is the in-house existence proof: SegaraBumi's indexer meets a `< 50 ms`
interactive query target over a large corpus by indexing rather than loading (T1
`sonar_ingest/E_indexer_search.md` §3.2, own design target).

The ceiling is deliberately not given a value here. Dossier E-6 names it "the one blocking item in
this domain" and states why it cannot be inherited: IP's 2,000 is its whole working set, not an
interactive subset, and Geolog states none because it materialises none. §5 records it `ABSENT —
ships with no default` and `SB-DBM-T38` is the measurement that would settle it.

**As-built.** `ABSENT` — a project switch tears down and rebuilds the single global connection
(`project.rs` module doc, `:137-217`); no interactive-set concept exists in the backend.

**Verified by.** SB-DBM-T38

---

#### SB-DBM-039 — A job result distinguishes clean, degraded and failed, and the store records which [P0] [status: PARTIAL]

**Requirement.** A per-well job result MUST distinguish a clean success from a degraded one, and the
degradation MUST be recorded in the run record, not only in the transient job view. A well whose
result was clamped, defaulted, truncated, or computed from a substituted input MUST NOT be reported
as `Ok`. A job whose items are wholly or partly degraded MUST NOT present an aggregate that reads as
clean.

**Rationale.** `SB-CORE-002`, whose surface in this domain is the job result. The transient half
already exists — `ItemState` carries a distinct `Warned` between `Ok` and `Failed`
(`jobs.rs:28-34`), and the module-run path downgrades affected wells to `Failed` when the set or
write fails (`workflow.rs:750-762`). What does not exist is the durable half: `Warned` lives in the
job registry, which is pruned at `MAX_FINISHED = 24` (`jobs.rs:119`, `:394-409`), so the record that
a curve was degraded outlives neither the session nor twenty-four subsequent jobs, while the curve
itself persists indefinitely. Three of seven `SB-CORE-002` violations were verified closed on
2026-08-07 and need regression locks so they stay closed; `SB-DBM-T39` and `SB-DBM-T40` are those
locks.

**As-built.** `PARTIAL` — `jobs.rs:28-34`, `:119`, `:394-409`, `workflow.rs:750-762`. No degradation
column in `log_sets` (`db.rs:316-329`).

**Verified by.** SB-DBM-T39, SB-DBM-T41

---

#### SB-DBM-040 — Cancellation honesty is regression-locked [P1] [status: PRESENT-OK]

**Requirement.** A job MUST be finalized `Cancelled` only when a worker has observed the cancel
request; a job whose worker never checked MUST finalize by its actual outcome. A job that cannot be
cancelled MUST declare so in the view the UI consumes, and the UI MUST NOT offer a control that does
nothing. Both halves MUST remain covered by tests.

**Rationale.** `SB-CORE-036`. This requirement pins shipped behaviour rather than closing a gap —
§3.9 documents the two-flag model (`jobs.rs:129-137`, `:152-158`, `:286-290`), the `cancellable`
field on both `Job` (`:89`) and `JobView` (`:107`), `run_simple_job` passing `false` (`:319`), the
frontend gate (`src/ui/processingPanel.ts:203`) and the two existing tests (`jobs.rs:527-539`,
`:545-578`). A cancel button that reports "Cancelled" while work continues is `SB-CORE-002` wearing a
different hat, and the cheapest way for it to come back is for someone to add a job type and forget
the flag. The requirement is what makes that a test failure.

**As-built.** `PRESENT-OK` — as cited. `04_CORE_REQUIREMENTS.md`'s as-built note for `SB-CORE-036`
is stale on this point; see §7.2 escalation 2.

**Verified by.** SB-DBM-T40

---

#### SB-DBM-041 — A count presented as a total is a total; the inspector exposes the provenance tables [P1] [status: PRESENT-DIVERGENT]

**Requirement.** A field named as a total MUST carry the true count, or the response MUST carry a
distinct field name and a flag stating that it does not. The database inspector's whitelist MUST
include every table a user would need in order to audit a curve — at minimum `log_sets`,
`computed_curves_archive`, `ml_models`, `curve_meta` and the audit tables of `SB-DBM-011`.

**Rationale.** `SB-CORE-002` at small magnitude and `SB-CORE-010` at large. `TablePage.total_rows`
means the true `COUNT(*)` when produced by `get_table_page` (`db.rs:3511-3564`) and means the number
of rows returned when produced by `run_readonly_query` (`db.rs:3566-3606`) — one field name, two
meanings, and the second one reads to a user as a total. Separately, `TABLE_SPECS`
(`db.rs:3476-3495`) whitelists **eight** tables of the 33-plus the schema declares, and the omitted
set contains **every provenance table in the product**. An inspector that cannot show `log_sets` is
an inspector that cannot answer the question this chapter exists to make answerable.

**As-built.** `PRESENT-DIVERGENT` — as cited.

**Verified by.** SB-DBM-T41, SB-DBM-T42

---

#### SB-DBM-042 — The format-version gate and the pre-migration backup are contractual, and the backup names the format it can restore [P0] [status: PRESENT-OK]

**Requirement.** Opening a project stamped a **newer** format version MUST be refused, naming both
versions and the writing application, and MUST leave the file byte-identical. A **destructive**
migration MUST take a backup before its first write, MUST NOT overwrite an existing backup of the
same name, MUST abort the migration if the backup write fails, and MUST tell the user the backup
path. An **additive** migration MUST NOT take a backup. The backup filename MUST identify the
**source** format version it can restore, not the target it was upgraded to.

**Rationale.** Three of the four clauses pin shipped behaviour so it cannot regress: the gate
(`db.rs:117-167`), the backup discipline (`db.rs:908-932`) and the additive exemption, whose reason
is recorded in the source — a backup on every open would bury the one that matters. Dossier §5.8
item 2 was **struck on review** for exactly this reason: an earlier draft listed it as work to do
and it turned out to be shipped. The one genuine delta is the naming clause, from F-07: Geolog's
Database Upgrader names its backup by source (`4.7 Upgrade Backup 13 May 2022`), whereas SandiBumi
writes `<stem>.pre-{FORMAT_VERSION}-backup.duckdb` — the *target*. A chain of upgrades therefore
leaves a shelf of backups each labelled with what it became, identifiable only by timestamp.

**Betters:** IP's ad-hoc per-well upgrade path takes **no** backup at all (T2 `O` §4.3), so a
one-way upgrade there is unrecoverable; SandiBumi refuses to migrate when it cannot first copy.

**As-built.** `PRESENT-OK` for the gate and the backup — `db.rs:36-42`, `:117-167`, `:908-932`,
`:934-956`, `:958-1002`, `:1004-1029`, `project.rs:137-217`. `PRESENT-DIVERGENT` for the naming
clause — `db.rs:908-932` uses `FORMAT_VERSION`, the target.

**Verified by.** SB-DBM-T01, SB-DBM-T02, SB-DBM-T43

---

#### SB-DBM-043 — A deterministic parameter sweep records every trial, uncapped and ordered [P2] [status: ABSENT]

**Requirement.** Where a capability evaluates a method across a grid of parameter combinations, the
store MUST hold each trial as a first-class row carrying its parameter vector, its result, and a
reference to the sweep that produced it, under the same provenance rules as any other run
(`SB-DBM-001` … `SB-DBM-006`). The sweep MUST be **deterministic** — a declared enumeration order,
no random sampling — and MUST NOT be capped at a fixed number of depth levels. Where a sweep must be
subsampled for cost, the subsampling rule MUST be declared, recorded and reproducible from the
recorded seed (`SB-DBM-014`); a random subsample with no recorded seed MUST NOT be possible.

**Rationale.** This is the **storage half** of the C-2 independent-derivation obligation in §7.4, and
it is the half that belongs to this chapter. A cross-product harness over a parameter grid is
useless without somewhere to put its trials, and the incumbent's documented weaknesses are all
storage-shaped: it is capped, it samples randomly, and it records nothing. The method half —
which parameters are swept and how a trial is scored — belongs to the chapter that owns the
petrophysical question.

**Betters:** the incumbent capability in `CONTRACT.md` §2.2's C-2 row is documented in this corpus
as **capped at 475 depth levels** and as sampling **100 depth levels at random** where the same
vendor's standalone tool uses **200 sorted** — established by three exact cross-product
reproductions, i.e. from observable arithmetic, not from its internals. A store that is uncapped,
deterministically ordered, and provenance-recording under `SB-CORE-010` is better on all three axes
and needs none of the vendor's code.

**As-built.** `ABSENT` — no trial or sweep store exists.

**Verified by.** SB-DBM-T44

---

## 5. Parameters

**This chapter ships no petrophysical parameter.** That is worth stating at the top rather than
leaving a reader to infer it from the absence of a matrix density. A data model holds numbers; it
does not assert them. Every value in the table below is a **schema constant, a capacity, a
tolerance, an identity rule or a sentinel** — the units are characters, bytes, rows, wells,
milliseconds and dimensionless magnitudes, and not one of them is a rock or fluid property. Where a
method chapter's parameter must be *stored*, `SB-DBM-003` governs how (value plus source string plus
absent-state), and this chapter does not set it.

Eight rows read `ABSENT — ships with no default` and three read `NON-ADOPTABLE — cited for
verification`. Each of the eleven is explained under the table. The absences are not a research
shortfall: this domain has **no literature dependency at all** (dossier §6: "Named papers/specs that
would be needed and are not held: none"), so where two vendors disagree there is no third source to
adjudicate, and picking one is the adjudication-disguised-as-a-default that `CONTRACT.md` §2 forbids.

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| `NULL_GEOLOG_FLOAT_THRESHOLD` | — | value < −1.0e29 | dimensionless | `Geolog-V14/include/cgg/cgg.h` — `IS_MISS_FLOAT(v) ≡ v < MISS_FLOAT/10.0`, `MISS_FLOAT = −1.0e30`; corroborated by `database_05_database_access_hc.5.09.html`, which instructs callers not to test equality | T1 |
| `NULL_GEOLOG_MAGNITUDE` | — | **NON-ADOPTABLE — cited for verification** (−1.0e30 header vs −1.0D38 manual) | dimensionless | `cgg.h` `MISS_FLOAT`/`MISS_DOUBLE` = −1.0e30 **vs** `…hc.5.09.html` "stores the undefined log value as … −1.0D38". Unreconciled; dossier `OPEN-DB-2` | T1 / T2 |
| `NULL_GEOLOG_INT` | — | −2147483647 | int32 | `cgg.h` `MISS_INT` | T1 |
| `PARAM_NOT_SUPPLIED_GEOLOG` | — | −2147483646 | int32 / float / string | `Geolog-V14/include/parameters/parameters.h` `PAR_DEFAULT_NONE_INT` / `_FLOAT` / `_DOUBLE` / `_VALUE` | T1 |
| `NULL_SCREEN_SUSPECT` | — | {−999, −999.0, −999.00, −999.25, −9999, −99} | dimensionless | Ledger R-9 / `ip2025_chm_ingest/N_data_io.md` §8.1; −9999 additionally has a T1 producer, `Techlog/Data.py:15` `MissingValue = −9999`. **Cross-reference — `21_data-io.md` owns the parse-side screen** | T1 / T2 |
| `NULL_WRITE_LAS` | — | −999.25 | dimensionless | `N_data_io.md` §8.1 + ledger R-9. **Cross-reference — owned by `21_data-io.md`**; listed here only because the store must not coerce toward it | T2 |
| `IRREGULAR_DEPTH_TOLERANCE` | — | 0.2 | ft — **unit scope OPEN**, see note 4 | `O_db_config_infra.md` §4.1 (`options.htm`, image `_tclip0110.png`) — IP's *Irregular Set Tolerance*, depth wells, read from a dialog raster | **T3** |
| `IRREGULAR_TIME_TOLERANCE` | — | 0.5 | s | same source — IP's *Irregular Set Tolerance*, DateTime wells; unit scope unambiguous | **T3** |
| `SAMPLING_STYLE_VERIFY_TOLERANCE` | — | **ABSENT — ships with no default** | depth unit | No vendor publishes a tolerance for *verifying* a declared constant increment; IP's 0.2 ft is a **snap** tolerance for an already-irregular set, which is a different question. See note 5 | — |
| `COMPACT_THRESHOLD_FRACTION` | — | 0.75 | fraction of file live before repack is skipped | `RF01_Database/database_03_database_format_hc.3.3.html` — Geolog `GLDBWell` parameter `WELL_FULL`, default 75 %, range 1–100 % | T2 |
| `LARGE_ARRAY_PAGE_BYTES` | — | 10,000,000 | bytes | `Techlog/Data.py` `CacheVarData.__cacheSize` — "A page of data consists of 10 MB of contiguous values, in row-major". **Evidence only — not adopted**; recorded as the sole vendor precedent for an array page size | T1 |
| `DATASET_STEP_CANDIDATES` | — | 0.001, 0.00254, 0.003048, 0.00381, 0.00508, 0.00762, 0.01, 0.01524, 0.01905, 0.0254, 0.03048, 0.0508, 0.0762, 0.1, 0.125, 0.1524, 0.2, 0.3048, 0.6096, 0.9144 | m | `Techlog 2018.2 (r22885)/DatasetStep.csv` (header row is the unit, `m`). **Evidence only** — a candidate list for step detection, not a constraint | T1 |
| `MAX_SET_NAME_CHARS_GEOLOG_EXPORT` | — | 32 | characters | `cgg.h` `GG_L_SET 32`; `RF01_Database/database_02_well_data_model_hc.2.16.html`. **Cross-reference — export clamping is `21_data-io.md`'s** | T1 / T2 |
| `MAX_LOG_NAME_CHARS_GEOLOG_EXPORT` | — | 32 total / 29 recommended descriptive | characters | `…hc.2.16.html` — "the 32 character limit encompasses an underscore and any version number … limit the log mnemonic to 29 characters to allow for the possibility of a two digit version number". **Cross-reference — `21_data-io.md`** | T2 |
| `MAX_UNITS_CHARS_GEOLOG_EXPORT` | — | 16 | characters | `cgg.h` `GG_L_UNITS 16`; `…hc.2.16.html`. **Cross-reference — `21_data-io.md`** | T1 / T2 |
| `MAX_PWI_CHARS_GEOLOG_EXPORT` | — | **NON-ADOPTABLE — cited for verification** (250 vs 32) | characters | `cgg.h` `GG_L_WELL 250` + `…hc.2.16.html` = 250, **vs** `specs/wellinfo.wellinfo` `WELL ALPHA*32` = 32. Dossier `OPEN-DB-1`; clamp exports to 32 until resolved | T1 / T2 |
| `MAX_SET_NAME_CHARS_IP_EXPORT` | — | **NON-ADOPTABLE — cited for verification** (8 vs 4) | characters | `O_db_config_infra.md` §3.2 / §8.2 — `managecurvesets.htm` = 8, `manage-multi-well-curve-sets.htm` = 4. Ledger `O-8.2`; clamp to 8, reject a leading digit, log renames | T2 |
| `MAX_WELLS_IP_DATABASE` | — | 9,999 | wells | `O_db_config_infra.md` §3.1 (`aboutipdatabase.htm`) — **export-path guard only**, not a SandiBumi limit | T2 |
| `MAX_SAMPLES_PER_SET_IP` | — | 3,000,000 | samples | `O_db_config_infra.md` §3.1 — export-path guard only | T2 |
| `MAX_CURVES_PER_WELL_IP` | — | 20,000 | curves | `O_db_config_infra.md` §3.1 — export-path guard only | T2 |
| `MAX_LOGGING_RUNS_IP` | — | 25 | runs | `O_db_config_infra.md` §1 row 32 (`wellheaderinfo.htm`) — export-path guard only | T2 |
| `AUDIT_TIMESTAMP_STORAGE` | — | UTC (display local) | — | `AuditTrail/audit_trail_hc.1.05.html` | T2 |
| `AUDIT_ENTRY_COLLAPSE_WINDOW` | — | **ABSENT — ships with no default** | s | Geolog collapses "uninterrupted repeated actions of the same type" into one entry (`AuditTrail/audit_trail_hc.1.06.html`) and publishes **no interval**; the vendor's rule is uninterruptedness, not elapsed time. See note 6 | — |
| `WELL_NAME_NORMALISATION_UPPERCASE` | — | true (on Geolog export) | — | `…hc.2.16.html` — "Lowercase letters are converted to uppercase". **Cross-reference — `21_data-io.md`** | T2 |
| `DUPLICATE_DEPTH_PERTURBATION` | — | **ABSENT — ships with no default** | ft or m | IP publishes 0.01 ft for duplicate FPRESS depths (`O` §4.6) — the sole vendor precedent. SandiBumi's declared resolution under `SB-DBM-026` is **refusal** for continuous sets, so no perturbation constant ships. See note 7 | T2 |
| `AUTOSAVE_INTERVAL` | — | **ABSENT — ships with no default** | min | IP has **no autosave**; its Save Reminder minimum is 5 min and non-persistent (`O` §4.3). Techlog states an Automatic save option with no interval in the pages read. Geolog's Project application writes through immediately. No defensible vendor default exists | T2 |
| `INTERACTIVE_SET_CEILING` | — | **ABSENT — ships with no default** | wells | IP's in-memory cap is 2,000 (`O` §3.1) but that is IP's **whole working set**, not an interactive subset; Geolog publishes none because it materialises none. **Must be measured on SandiBumi** — dossier E-6, the one blocking item in this domain | T2 |
| `LOCK_TIMEOUT` | — | **ABSENT — ships with no default** | min | IP's `IPDBLock` self-clears in 4–5 min and its Multi-User grant window is 5 min (`O` §3.6). Recorded as the only vendor precedent; **not adopted** — SandiBumi is single-writer and has no lock to time out | T2 |
| `INTERACTIVE_QUERY_TARGET` | — | < 50 | ms | `sonar_ingest/E_indexer_search.md` §3.2 — **SandiBumi's own design target, not a vendor value** | T1 |
| `MODULE_VERSION_SOURCE` | — | **ABSENT — ships with no default** | — | `SB-DBM-002` requires a build-derived module version; no vendor publishes a form for one (IP, Techlog and Geolog all record a module *name*). The form is an implementation choice, not a citable value | — |
| `ARTIFACT_HASH_ALGORITHM` | — | **ABSENT — ships with no default** | — | `SB-DBM-019` requires a cryptographic hash of a stored model artifact; no vendor precedent exists because no vendor exposes its model artifacts. The choice is an implementation decision | — |
| `FORMAT_VERSION` | — | 1 | — | `db.rs:29` | T1 |
| `DB_MEMORY_CAP` | — | engine default ÷ 4, clamped to [1, 4] | GiB | `db.rs:54-74`; overridable by `SANDIBUMI_DB_MEMORY` | T1 |
| `COMPUTED_CURVES_PRIMARY_KEY` | — | none — deliberate | — | `db.rs:292-305`; uniqueness by write discipline instead | T1 |
| `ART_INDEX_INSERT_PENALTY` | — | ~3.7× (311,000 vs 1,160,000) | rows/s | `db.rs:293-295` — measured, and the reason `computed_curves` has no primary key | T1 |
| `LOG_SET_FRAME_DEFAULT` | — | `'STANDARD'` | — | `db.rs:328`; the only other legal value is `'OWN'` | T1 |
| `SLOW_OPEN_BOOT_NOTE_THRESHOLD` | — | 10 | s | `project.rs:210-215` | T1 |
| `RECENT_PROJECTS_LIMIT` | — | 12 | entries | `project.rs:113` (`list.truncate(12)`) | T1 |
| `MAX_FINISHED_JOBS` | — | 24 | jobs | `jobs.rs:119` (`MAX_FINISHED`), pruned at `jobs.rs:394-409` | T1 |
| `INSPECTOR_PAGE_LIMIT_CLAMP` | — | 1..2000 | rows | `db.rs:3511-3564` | T1 |
| `INSPECTOR_TABLE_WHITELIST_SIZE` | — | 8 | tables | `db.rs:3476-3495` (`TABLE_SPECS`), against 33+ declared in the schema | T1 |
| `SQL_CONSOLE_LIMIT_CLAMP` | — | 1..5000 | rows | `db.rs:3566-3606` | T1 |
| `PROCESS_LOG_LIMIT` | — | 5,000 | entries | `src/processLog.ts:22` | T1 |
| `PROCESS_LOG_SAVE_DEBOUNCE` | — | 600 | ms | `src/processLog.ts:38` | T1 |
| `UNDO_STACK_LIMIT` | — | 100 | actions | `src/undo.ts:12` | T1 |

**45 rows.** 8 `ABSENT — ships with no default`, 3 `NON-ADOPTABLE — cited for verification`, 6
cross-references to `21_data-io.md`, 2 recorded as evidence only. 14 rows are SandiBumi's own
constants at `file:line`.

### Notes on the absences and non-adoptions

1. **`NULL_GEOLOG_MAGNITUDE`** — two vendor sources eight orders of magnitude apart, one an
   executable header and one shipped prose, with no reconciling text. `SB-DBM-030` adopts the
   **threshold** form instead, which is correct under both readings; that is why the magnitude can be
   left non-adopted without leaving a hole. Only the Geolog *export* path needs a magnitude, and
   `21_data-io.md` owns that choice with the logging obligation attached.

2. **`MAX_PWI_CHARS_GEOLOG_EXPORT`** — `cgg.h` and the manual say 250; the shipped well-index spec
   types the field `ALPHA*32`. Two T1-and-T2 readings of the same product. Clamping to the safer 32
   is a mitigation, not an adjudication, and it is recorded as such.

3. **`MAX_SET_NAME_CHARS_IP_EXPORT`** — the same manual states 8 on one page and 4 on another. Ledger
   `O-8.2` has carried this unresolved since it was first recorded.

4. **`IRREGULAR_DEPTH_TOLERANCE` unit scope** — the value 0.2 ft is transcribed byte-exact from a
   **raster dialog** (T3), the weakest tier in this chapter. What the raster cannot tell us is
   whether IP fixes the tolerance in feet regardless of the well's depth unit or displays it in the
   well's own unit. The difference is 3.28×, and on a 0.1524 m workflow it is the difference between
   40 % of a step and 1.3 steps — i.e. between a sane snap and silently consuming a whole sample
   into its neighbour. Adoption rule until resolved (dossier `OPEN-DB-4`): store it as a unit-typed
   `Length`, default it to 0.2 ft = 0.06096 m, convert explicitly at the comparison site, and log the
   snap decision. A later resolution then changes one constant, not a code path.

5. **`SAMPLING_STYLE_VERIFY_TOLERANCE`** — `SB-DBM-028` requires a declared increment to be checked
   against the reference column, which needs a tolerance for "constant". No vendor publishes one,
   because no vendor performs the check — Geolog documents that it *cannot* detect the condition
   (T2 `…hc.5.11.html`). Reusing `IRREGULAR_DEPTH_TOLERANCE` would be a category error: that value
   governs how far apart two depths may be and still be treated as one sample, which is a different
   question from how much increment variation still counts as regular. Recorded absent, and the
   `SB-DBM-T27` fixture is written so that the tolerance is an input rather than an assumption.

6. **`AUDIT_ENTRY_COLLAPSE_WINDOW`** — Geolog's collapsing rule is stated in terms of
   *uninterruptedness*, not elapsed time, and no interval is published. Adopting a time window would
   invent a vendor fact. `SB-DBM-011` therefore states the rule as the vendor does.

7. **`DUPLICATE_DEPTH_PERTURBATION`** — IP's 0.01 ft is real and cited, and it is a resolution
   SandiBumi does not adopt for continuous sets: `SB-DBM-026` refuses the duplicate instead, which is
   the stronger contract. The constant would only be needed if a perturbing mode were added, and it
   is recorded here so that a future implementer reaches for the cited value rather than inventing
   one.

8. **`AUTOSAVE_INTERVAL`, `INTERACTIVE_SET_CEILING`, `LOCK_TIMEOUT`** — three different reasons for
   the same status. The autosave interval has no defensible vendor default (one vendor has no
   autosave at all, one states no interval, one writes through). The interactive-set ceiling is a
   property of SandiBumi's own read path and **must be measured**, not inherited — dossier E-6 calls
   it the one blocking item in this domain and `SB-DBM-T38` is the measurement. The lock timeout has
   exactly one vendor precedent and no problem to solve, because the single-writer model has no lock
   to time out.

9. **`MODULE_VERSION_SOURCE` and `ARTIFACT_HASH_ALGORITHM`** — both are required by a P0 requirement
   (`SB-DBM-002`, `SB-DBM-019`) and neither has a citable value, because no incumbent does the thing.
   They are listed rather than omitted so that the implementation choice is visible as a choice.

---

## 6. Acceptance tests

Forty-four tests. Six are labelled `CHARACTERIZATION` — their expected value is SandiBumi's own
current behaviour rather than an externally sourced number, and they exist to stop a regression
rather than to prove a claim.

A note on tolerances in this domain. Most of these tests are **exact**: a refusal happens or it does
not, a column is populated or it is not, two byte streams are identical or they are not. Where a
tolerance appears it is stated, and where the correct tolerance is itself unknown the test takes it
as an input rather than assuming one (`SB-DBM-T27`).

### 6.1 Format, migration and backup

**SB-DBM-T01 — Format-version gate.** `CHARACTERIZATION`
*Input:* a project file stamped `format_version = FORMAT_VERSION + 1` with a `written_by` string.
*Operation:* open it.
*Expected:* refused; the error names **both** version numbers and the `written_by` string; the file
is **byte-identical** afterwards (compare a hash before and after). Exact.
*Source:* shipped behaviour, `db.rs:117-167`; dossier T-DB-10, which states the test pins an already
implemented path.

**SB-DBM-T02 — One-way upgrade takes a backup, in three cases.** `CHARACTERIZATION`
*Input:* (a) a project stamped older with a **destructive** migration pending; (b) one with only
**additive** migrations pending; (c) a destructive migration whose backup write is made to fail.
*Operation:* open each.
*Expected:* (a) a backup exists **before the first write**, an existing backup of the same name is
**not overwritten** — the second run gets a timestamp suffix — and the path is reported to the user;
(b) **no** backup is taken; (c) the migration **aborts** and the un-migrated file still opens. Exact.
*Source:* shipped behaviour, `db.rs:908-932`, `:958-1002`; dossier T-DB-11, which records that an
earlier draft had this the other way round and that the additive exemption is deliberate.

**SB-DBM-T43 — The backup names the format it can restore.**
*Input:* a project at format version *n*, upgraded through *n+1* to *n+2*, each step destructive.
*Operation:* inspect the backup filenames.
*Expected:* each backup name identifies the **source** version it restores, so the shelf reads
`…pre-n…`, `…pre-(n+1)…`; no two backups are distinguishable only by timestamp. Exact.
*Source:* F-07 — Geolog's Database Upgrader names by source (`4.7 Upgrade Backup 13 May 2022`, T2
`O` §4.3 equivalent). Current behaviour uses the target (`db.rs:908-932`), so this test fails today
by design.

### 6.2 Provenance

**SB-DBM-T03 — Every computed value resolves to one run record.**
*Input:* a project containing (i) curves written by the current path, (ii) rows with `set_id IS NULL`
seeded to simulate a legacy project.
*Operation:* run the provenance resolver over every row of `computed_curves`.
*Expected:* every (i) row resolves to exactly one `log_sets` row; every (ii) row is reported in a
`LEGACY_UNRECORDED` class with a count, and is labelled as such in any display or export of that
curve. Zero rows silently unclassified. Exact.
*Source:* `SB-CORE-010`; `db.rs:307-310` (NULL means legacy or unversioned).

**SB-DBM-T04 — Module identity changes when the module changes.**
*Input:* the same module run twice with identical parameters, with a rebuild in between that alters
the module's compiled artefact.
*Operation:* compare the two run records.
*Expected:* the recorded module identity **differs** between the two records. Exact.
*Source:* `SB-CORE-011`; F-06 (Geolog resolves an unversioned reference to the latest version, which
is only safe because the version exists).

**SB-DBM-T05 — A parameter without a source is a named state, and is queryable.**
*Input:* a module run with three parameters — one carrying a source string, one explicitly marked
`REQUIRED_UNSET`, one supplied by the UI with no source.
*Operation:* write the run record, then query "every parameter in this project whose source is
unset".
*Expected:* the first stores value + source; the second stores `value = NULL, source = NULL, state =
REQUIRED_UNSET`; the third is **refused at write** — a value with an empty source is not
representable. The query returns exactly the second row, from an index, not a scan. Exact.
*Source:* `SB-CORE-004`; F-01; dossier §5.4 rule "`source` is mandatory for any numeric
petrophysical parameter … a *silently defaulted* value is not [a legal state]".

**SB-DBM-T06 — Effective parameters, not overrides.**
*Input:* a module with five declared parameters, of which the user overrides two.
*Operation:* run it; then change one manifest default and inspect the original record.
*Expected:* the record contains all five with `EXPLICIT` on two and `DEFAULTED` on three, and the
three defaulted rows name the manifest version. After the manifest change the original record is
unchanged and still reports the value that was actually used. Exact.
*Source:* `SB-CORE-011`; F-18 / ledger R-10 as the structurally identical failure one level down.

**SB-DBM-T07 — The derivation citation travels.**
*Input:* two registered modules — one with a literature citation, one with a `FIRST-PRINCIPLES`
marker — and one module registered with neither.
*Operation:* register all three, run the first two, export the results.
*Expected:* the third **fails registration** with a named error. The first two produce run records
each carrying their citation, and the exported sidecar carries both citations verbatim. Exact.
*Source:* `CONTRACT.md` §2.2 as amended — an independently-derived capability must be recorded to be
distinguishable from a reconstruction; §1.3 of this chapter.

**SB-DBM-T08 — Curve resolution is a logged decision.**
*Input:* a well holding three GR curves across two sets, one flagged Final.
*Operation:* run a module whose input slot requests `GR`.
*Expected:* the run record names the chosen curve **by identity and set version**, names the rule
that chose it from the declared vocabulary, and lists the two rejected candidates by identity.
Re-running after flagging a different curve Final produces a record naming a different curve and the
same rule. Exact.
*Source:* F-04; dossier T-DB-13 and §5.3; `O` §10.5; `FINDINGS.md` §6 rule 15.

**SB-DBM-T09 — Absent is a named state, never an empty string.**
*Input:* (a) an equation run with genuinely no parameters; (b) a module run whose parameter
serialisation is made to fail.
*Operation:* write both records, then read them back.
*Expected:* (a) stores `NOT_APPLICABLE`; (b) **fails the run** rather than storing anything. Neither
stores `""`. A reader can distinguish the two cases without heuristics. Exact.
*Source:* F-11 (Geolog spends two constants to keep the states apart); `SB-CORE-002`;
`equations.rs:1231-1238` is the current counter-example.

**SB-DBM-T10 — Provenance reaches the deliverable.**
*Input:* a project with 20 wells, computed curves from three module runs and one equation run, plus
two curves in the `set_id IS NULL` legacy class.
*Operation:* export the computed curves with the provenance sidecar.
*Expected:* every exported curve resolves in the sidecar to its run record, including parameter
source strings and derivation citations; the two legacy curves are exported **labelled** as
unprovenanced; the export summary states the count of unprovenanced curves. An export that drops
provenance entirely reports that fact at the point of export. Exact.
*Source:* `SB-CORE-010` with the 2026-08-07 scope resolution; F-03 as the cautionary case.

**SB-DBM-T11 — Audit entry structure and UTC storage.**
*Input:* a session that (i) changes three zone parameters, (ii) renames a curve, (iii) drags a
crossplot point 40 times without interruption, (iv) runs in a machine set to UTC+8 across a DST
boundary in another zone.
*Operation:* inspect `audit_entry` / `audit_detail`.
*Expected:* (i) produces detail rows with `location = PARAMETER`, `mode = INPUT`, unit, name and
value; (ii) produces `mode = RENAME`; (iii) collapses to **one** entry, not 40; (iv) every `ts_utc`
is UTC and renders local. The operator and, for zone-scoped runs, the zone-set identity are present.
Exact.
*Source:* dossier §5.5, adopted from T2 `AuditTrail/audit_trail_hc.1.05.html`, `…1.06.html`.

**SB-DBM-T12 — A parameter diff is a join.**
*Input:* two audit states of the same zone differing in three parameters.
*Operation:* request the diff.
*Expected:* structured `(zone, parameter, old, new, unit)` rows; **no external process is spawned**
(assert on the process table, not on the output); the result is embeddable in a report. Exact.
*Source:* F-02; dossier T-DB-15; `O` §10.11.

**SB-DBM-T13 — Provenance cannot be switched off, and a failed record fails the write.**
*Input:* a module run whose `log_sets` insert is made to fail mid-transaction.
*Operation:* run it; then enumerate every setting, preference and environment variable the app reads.
*Expected:* the curve write is rolled back — `computed_curves` gains no rows for that well — and the
well is reported `Failed`. No setting exists whose value causes a computed curve to be written
without a run record. Exact.
*Source:* F-03; shipped transaction shape at `equations.rs:626-672` and `workflow.rs:750-762`.

### 6.3 Reproducibility

**SB-DBM-T14 — Seed, rule and generator are recorded and sufficient.**
*Input:* a stochastic run (Monte Carlo, N realisations) executed once.
*Operation:* record the run; on a different machine, in a different process, re-run **from the record
alone**.
*Expected:* the recorded triple (root seed, seeding rule, generator identity) is present; the re-run
output is **bit-identical** to the original. Exact — no tolerance; a bitwise comparison.
*Source:* `SB-CORE-011`; the `(seed, index)` derivation it already names.

**SB-DBM-T15 — The re-run manifest is complete and is checked.**
*Input:* a run recorded under the full manifest; then four mutated copies of the project in which,
respectively, the module version differs, an input curve has been re-run to a new version, a
zone-set has been edited, and an applied model has been deleted.
*Operation:* request "re-run this set" against each.
*Expected:* the unmutated project reproduces bit-identically. Each mutated project is **refused**,
and the refusal names the specific manifest element that no longer resolves. A refusal that names no
element fails the test. Exact.
*Source:* `SB-CORE-011`; F-18 / ledger R-10 for the substitute-silently failure this prevents.

**SB-DBM-T16 — Output does not depend on iteration order.**
*Input:* one project, run twice in processes started with different hash seeds.
*Operation:* compare every output curve byte for byte, and compare aggregate statistics.
*Expected:* identical. Exact.
*Source:* `SB-CORE-011`; the PK-less `computed_curves` store has no row-order guarantee
(`db.rs:292-305`).

**SB-DBM-T17 — A physics-driving attribute is a run-record input.**
*Input:* a well whose attribute selects a tool-response table, and a module that consumes it; plus a
second well where that attribute is unset.
*Operation:* run the module; change the attribute; re-run. Separately, run against the unset well.
*Expected:* the attribute and its run-time value appear in the run record; changing it marks the
prior run's outputs **stale**; the unset well **fails with a named error** rather than defaulting.
Exact.
*Source:* F-05 — IP's Logging Contractor, T2 `O` §3.3 and the vendor-ingest mitigation at `O` §10
item 9; dossier invariant 11 and T-DB-19.

### 6.4 The learned-model store

**SB-DBM-T18 — Training-set identity survives a rename and a re-run.**
*Input:* a model trained on five wells over declared depth intervals; then (a) rename one training
well, (b) re-run one training curve producing a new set version, (c) delete one training well.
*Operation:* attempt to apply the model in each state.
*Expected:* (a) applies normally — identity is by id, not name; (b) applies, **and reports** that a
training input has since changed version; (c) is reported unresolvable and is **not applied**. The
stored record names the depth interval per well, so a whole-well model and an interval model are
distinguishable. Exact.
*Source:* `SB-CORE-014`; `db.rs:684` is the current counter-example (`trained_on` = well names).

**SB-DBM-T19 — Seed, library set and artifact hash.**
*Input:* two training runs with identical data and identical hyper-parameters, one with the seed
pinned and one without.
*Operation:* train, store, reload.
*Expected:* the pinned pair produce identical artifacts and identical hashes; the record carries
every library whose numerics affect the fit, not only scikit-learn; a hand-corrupted `data` blob
**fails to load with a named hash-mismatch error**, never a warning. Exact.
*Source:* `SB-CORE-014`, `SB-CORE-011`; `db.rs:687` records `sklearn_version` alone.

**SB-DBM-T20 — Both apply paths stamp the model.**
*Input:* the same estimator applied twice — once through train-and-apply, once through
apply-saved-model.
*Operation:* inspect both produced curves' run records.
*Expected:* both carry a `model_id` that resolves to a row in `ml_models`; the train-and-apply path
has persisted its model. A record naming only an algorithm fails the test. Exact.
*Source:* `ml.rs:942-943` states the requirement; `ml.rs:670-675` is the path that violates it.

**SB-DBM-T21 — A foreign artifact is refused at the store boundary.**
*Input:* a valid joblib artifact produced outside SandiBumi, offered to every write path that reaches
`ml_models` — command, file dialog, and a direct SQL insert through the console.
*Operation:* attempt each.
*Expected:* every path refuses. `origin` cannot be set from user input. The read-only console cannot
insert (it accepts only `SELECT`/`WITH`, `db.rs:3566-3606`). A row whose `origin` is not native is
refused at apply time, naming the row. Exact.
*Source:* `CONTRACT.md` §2.2 class C-3 — "a vendor-trained model is never consumed in any format".

**SB-DBM-T22 — Feature order is verified, not assumed.** `CHARACTERIZATION`
*Input:* a model whose `feature_curves` is `[GR, RHOB, NPHI]`, applied to a well resolving
`[GR, NPHI, RHOB]`, and to a well missing `NPHI`.
*Operation:* apply.
*Expected:* both wells **fail by name**. Set equality does not satisfy the check. Exact.
*Source:* the shipped contract statement at `db.rs:669-671`; labelled `CHARACTERIZATION` because the
expected behaviour is asserted by SandiBumi's own comment rather than by an external source.

### 6.5 One definition, one place

**SB-DBM-T23 — Vocabularies have one source and every projection derives from it.**
*Input:* the current source tree; then a patch adding an eighth standard column to the registered
vocabulary and nothing else.
*Operation:* build and run the suite.
*Expected:* before the patch, the suite fails today — two `STANDARD_COLUMNS` declarations exist
(`equations.rs:299`, `curve_edit.rs:81-88`) with different membership. After the fix, the patch alone
makes every consumer aware of the eighth column, and a re-introduced second literal declaration
fails the build or a test. Exact.
*Source:* `SB-CORE-007`; the consolidation the code itself claims at `equations.rs:296-298`.

**SB-DBM-T24 — Limits are unit-typed and documentation is generated.**
*Input:* the source tree and the published limits table.
*Operation:* assert that every capacity, tolerance and limit constant carries a unit type and a
source string, and that the docs table is generated from the declarations.
*Expected:* no bare numeric limit in the docs; no hand-typed table; `IRREGULAR_DEPTH_TOLERANCE`
specifically is a `Length`, not a float, and its conversion happens at the comparison site. Exact.
*Source:* F-23 / ledger `O-8.3` (a shipped artefact count published as a limit for seven years);
F-24 / `OPEN-DB-4`; `FINDINGS.md` §6 rules 3 and 10; dossier T-DB-14.

### 6.6 Store integrity

**SB-DBM-T25 — Depth uniqueness by set type.**
*Input:* two samples at the same `(well_id, curve_name, depth)`, written into (a) a
`CONTINUOUS_REGULAR` set and (b) a `POINT` set.
*Operation:* write.
*Expected:* (a) refused with a named error identifying the depth and **both** source rows — never a
silent last-writer-wins, which is what a PK-less table gives by default; (b) accepted, and if the
configured resolution is perturbation the offset is the declared unit-typed constant and is logged
per row. Exact.
*Source:* dossier invariant 12 and T-DB-20; F-26 (IP's 0.01 ft FPRESS rule, T2 `O` §4.6);
`db.rs:292-305` for why the engine will not do this for us.

**SB-DBM-T26 — The integrity checker reports every class, including the empty ones.**
*Input:* a project seeded with one dangling `computed_curves_archive.set_id`, one
`well_group_members` row whose well is gone, and zero orphaned `curve_samples`.
*Operation:* run the checker.
*Expected:* all three classes are reported **by name**, two with counts of 1 and one with a count of
0; a prune is offered; the checker never reports a bare "clean". Exact.
*Source:* dossier D-25 and T-DB-17; T2 `database_04_utilities_hc.4.8.html` — Geolog's
`well_include_check` reports both classes.

**SB-DBM-T27 — Sampling style is verified, not honoured.**
*Input:* a set declared `CONTINUOUS_REGULAR` at 0.1524 m whose reference column is missing **40
consecutive rows** mid-interval. The verification tolerance is supplied to the test as an input, not
assumed (§5 note 5).
*Operation:* ingest, then read the set frame-indexed.
*Expected:* the declaration is **contradicted**: the set is stored `CONTINUOUS_IRREGULAR` with a
named ingest warning identifying the gap depth and the row count. The frame-indexed read does **not**
place post-gap samples 6.1 m shallow (40 × 0.1524 m); assert the post-gap depth of a known sample to
within one sample interval.
*Source:* F-14; dossier §3.11, invariant 9, T-DB-16; T2
`database_05_database_access_hc.5.11.html`.

**SB-DBM-T28 — The reference column is not module-writable.**
*Input:* a module configured, through the output path, to write into the depth/reference column of
an existing frame.
*Operation:* run it.
*Expected:* refused at the API boundary, naming the frame; assert additionally that **no other curve
on that frame moved**. A module needing another depth basis emits a new frame declared `OWN`. Exact.
*Source:* F-16 — T2 `O` §2.5, "never write back to the Depth curve"; dossier invariant 13 and
T-DB-21.

**SB-DBM-T29 — Null threshold, including the exact boundary.**
*Input:* the values **−1.0e30**, **−1.0e38**, −9.99e29, **−1.0e29**, −1.0e28, and −1.0e30 after a
µs/ft→µs/m conversion (× 3.28084).
*Operation:* run the store-side null screen.
*Expected:* −1.0e30, −1.0e38, −9.99e29 and the converted value are **null**; −1.0e28 is **data**; and
**−1.0e29 is DATA, not null** — the vendor macro is a **strict** inequality
(`IS_MISS_FLOAT(v) ≡ v < MISS_FLOAT/10.0`), so a value exactly at the threshold falls on the data
side. Second assertion: the implementation **computes** the bound as `MISS_FLOAT/10.0` from the same
constant it would export, rather than hard-coding `−1e29`. **An equality-based implementation fails
this test, and so does one that hard-codes either single magnitude.** Exact.
*Source:* T1 `cgg.h:56-69`; T2 `…hc.5.09.html`; dossier T-DB-02 and `OPEN-DB-2`.

**SB-DBM-T30 — "No value" and "no parameter" are distinguishable at every layer.**
*Input:* a curve sample with no value and a parameter that was never supplied.
*Operation:* carry both through store → IPC → UI → export.
*Expected:* they are distinguishable at each of the four layers, and neither is representable as a
number at any of them. Exact.
*Source:* F-11 — T1 `cgg.h` `MISS_INT` vs `parameters.h` `PAR_DEFAULT_NONE_INT`; dossier T-DB-03 and
`FINDINGS.md` §6 rule 9.

**SB-DBM-T31 — Depth datum is declared and cross-datum comparison is refused.**
*Input:* an MD zone top and a TVDSS contact in a well with no reference frame, and the same pair in a
well that has one.
*Operation:* compare.
*Expected:* refused in the first case, naming **both** datums; permitted in the second. Sign
conventions asserted: TVDSS positive down, elevation positive up from the measurement reference.
Exact.
*Source:* F-17 — T2 `…hc.2.12.html`; dossier invariant 4, T-DB-08, `FINDINGS.md` §6 rule 13.

**SB-DBM-T32 — Ordinal/key mismatch is a hard error.**
*Input:* a parameter file in which ordinal 41 carries `key: "od_curv1_clean1"`, loaded into a build
where ordinal 41 is `od_ot2_clean1`. Also: a file carrying only one handle; and a value carrying
`tilt: LOG` with a two-endpoint range.
*Operation:* load.
*Expected:* the mismatch is a **hard error naming both handles** — no remap, no
warning-and-continue. The single-handle file loads with a warning. The tilted value round-trips with
its tilt and its unit, and is evaluated **within-zone only**, stepping at the zone boundary. Exact.
*Source:* ledger **R-10** (the ClayVol #41 case, literally); dossier §5.4 and T-DB-04; F-19 for the
tilt half (T2 `O` §2.4, §10.2).

**SB-DBM-T33 — A categorical curve is never interpolated.**
*Input:* a facies code curve at 0.1524 m, resampled to 0.1 m; and an arithmetic expression over it.
*Operation:* resample; evaluate.
*Expected:* every output value is an **existing code** — no interpolated intermediate; every
boundary-crossing sample is reported; the arithmetic is refused. Exact.
*Source:* F-15 — T2 `…hc.2.06.html`, "When interpolating integer type logs, resultant values are
rounded"; dossier §3.10, D-12, T-DB-07.

**SB-DBM-T34 — Bulk operations drop nothing.**
*Input:* a bulk tops paste of 100 rows in which 3 well names are unmatchable and 2 are ambiguous.
*Operation:* paste.
*Expected:* returns `{matched: 95, unmatched: 3, ambiguous: 2}`; all 5 exceptions appear in the
review queue and are addressable; **zero silent drops**. Exact.
*Source:* F-25; ledger `O-8.8`; dossier T-DB-06; `FINDINGS.md` §6 rule 14.

**SB-DBM-T35 — The archive is append-only and restore creates a version.**
*Input:* a curve at version 3 with versions 1 and 2 in the archive.
*Operation:* attempt an UPDATE and a DELETE against `computed_curves_archive`; then restore
version 1.
*Expected:* both mutations are refused. The restore produces **version 4**, whose run record states
that it restored version 1; versions 1–3 remain in the archive unchanged. Exact.
*Source:* `SB-CORE-010`; F-06; `db.rs:331-334` states restoration as the table's purpose.

### 6.7 Concurrency, scale and honest results

**SB-DBM-T36 — The global lock is not held for the duration of a long operation.** `CHARACTERIZATION`
*Input:* a project of N wells with a batch module run in flight.
*Operation:* issue an interactive command (a well-list refresh) at a fixed cadence during the batch,
and record its latency distribution.
*Expected:* the interactive command's worst-case latency does **not** scale with N. Publish the
distribution; the gate is the shape, not an absolute figure, because no vendor publishes one and
`INTERACTIVE_QUERY_TARGET` (< 50 ms) is SandiBumi's own design target rather than a vendor value.
*Source:* `SB-CORE-032`; §3.10's measurement (109 of 130 sync commands take the lock);
`project.rs:144-146`'s ~5-minute open on ~540 wells. Labelled `CHARACTERIZATION` because the
threshold is SandiBumi's own.

**SB-DBM-T37 — Scoping is enforced in the backend.**
*Input:* a project of 540 wells with an active group of 12.
*Operation:* invoke each well-iterating backend command **directly**, bypassing the UI, with the
group active.
*Expected:* each command's query touches 12 wells, asserted by query plan or row-count
instrumentation rather than by wall time. A command that cannot be scoped declares itself
project-wide in its response. Exact.
*Source:* `SB-CORE-035`; §3.11 (`src/state.ts:135` is the current enforcement point); F-20.

**SB-DBM-T38 — Interactive-set scale curve.** `CHARACTERIZATION`
*Input:* N ∈ {100, 500, 1000, 2000, 5000} real wells.
*Operation:* time (a) project open, (b) well-list switch, (c) a faceted count query, (d) first paint
of the log view.
*Expected:* publishes a **curve, not a pass/fail**. The acceptance gate is that (b), (c) and (d) are
`O(size of the interactive set)`, not `O(N)`. The measured ceiling settles
`INTERACTIVE_SET_CEILING`, which §5 ships absent.
*Source:* dossier D-4, T-DB-12 and **E-6**, named there as the one blocking item in this domain; F-20.
Must be run on real wells and cannot be inferred.

**SB-DBM-T39 — A degraded result is never reported clean, and the degradation persists.**
*Input:* a batch run in which one well's result is clamped, one is computed from a substituted
input, and one succeeds cleanly; then 25 further jobs to force the finished-job prune.
*Operation:* inspect the job view during the run, and the run records afterwards.
*Expected:* the two degraded wells are `Warned`, never `Ok`; the aggregate does not read as clean;
and **after the prune** the degradation is still recoverable from the run record, not only from the
job registry. Exact.
*Source:* `SB-CORE-002`; `jobs.rs:28-34`, `:119`, `:394-409`.

**SB-DBM-T40 — Cancellation honesty, both halves.** `CHARACTERIZATION`
*Input:* (a) a cancellable job whose worker polls; (b) a cancellable job whose worker never polls;
(c) a non-cancellable job.
*Operation:* request cancel on each; inspect the view and the final phase.
*Expected:* (a) finalizes `Cancelled`; (b) finalizes by its **actual** outcome, not `Cancelled`;
(c) exposes `cancellable = false` and the UI offers no control. Exact.
*Source:* shipped behaviour — `jobs.rs:286-290`, `:89`, `:107`, `:319`, and the two existing tests at
`jobs.rs:527-539` and `:545-578`; `src/ui/processingPanel.ts:203`. Labelled `CHARACTERIZATION`
because it pins shipped behaviour rather than closing a gap.

**SB-DBM-T41 — A total is a total.**
*Input:* a table of 10,000 rows queried through the inspector, and the same table through the SQL
console with a limit of 100.
*Operation:* compare the two responses.
*Expected:* the inspector's `total_rows` is 10,000; the console's response either reports 10,000 or
uses a **differently named** field with an explicit flag stating it is not a total. One field name
carrying two meanings fails the test. Exact.
*Source:* `SB-CORE-002`; `db.rs:3511-3564` versus `db.rs:3566-3606`.

**SB-DBM-T42 — The inspector can answer the provenance question.**
*Input:* a computed curve produced by a module run.
*Operation:* using only the database inspector, trace it to its run record, its parameters, its
inputs and — if a model produced it — the model row.
*Expected:* possible without leaving the inspector. `log_sets`, `computed_curves_archive`,
`ml_models`, `curve_meta` and the audit tables are all in the whitelist. Exact.
*Source:* `SB-CORE-010`; `db.rs:3476-3495` whitelists 8 tables and omits every one of these.

**SB-DBM-T44 — A sweep is deterministic, uncapped and recorded.**
*Input:* a parameter grid of 3 × 4 × 5 combinations over a well with 3,000 depth levels.
*Operation:* run the sweep twice.
*Expected:* both runs enumerate the same 60 trials in the same declared order; every trial is stored
with its parameter vector, its result and its sweep reference, under the provenance rules of
`SB-DBM-001`…`SB-DBM-006`; **no depth cap is applied** — all 3,000 levels participate; and if a
subsample is configured, it is reproducible from the recorded seed. Bit-identical between the two
runs.
*Source:* `CONTRACT.md` §2.2 class C-2 and its worked case — the incumbent is capped at 475 depth
levels and samples 100 at random where the same vendor's standalone tool uses 200 sorted; this test
asserts the three axes on which the independent derivation is better.

---

## 7. Open items, escalations and refusals

Four labelled lists, per `CONTRACT.md` §3 and §2.2.1. The **last two subsections** are `Refusals` and
`Independent-derivation requirements`, in that order, as §2.2.1 requires — they are opposite in
meaning and are never mixed: `Refusals` records where SandiBumi declines to reproduce a vendor's
broken behaviour, which are **wins**, and `Independent-derivation requirements` records the Tier-C
capabilities this domain must derive independently.

### 7.1 Open — needed, not yet answerable

**Carried from the dossier's authoritative 8-item tally (§6 of the dossier).** All eight are
first-class states, not guesses.

1. **`OPEN-DB-1` — Geolog well/PWI name length: 250 or 32?** `cgg.h` `GG_L_WELL 250` and the manual
   say 250; `specs/wellinfo.wellinfo` types the index field `WELL ALPHA*32`. **What would settle
   it:** a live Geolog project holding a well name longer than 32 characters, or the Epos Services /
   PNS Manager documentation, which is not in this install. **Mitigation in force:** clamp exports to
   32. Escalated as item 4 below.

2. **`OPEN-DB-2` — Geolog's null magnitude: −1.0e30 or −1.0D38?** T1 header against T2 manual, eight
   orders of magnitude, unreconciled. **What would settle it:** a live Geolog session writing a null
   and inspecting the stored value, or `MISS_DOUBLE` printed from the shipped `pygg` module.
   **Why it is not blocking:** `SB-DBM-030`'s threshold form is correct under **both** readings —
   that is precisely why it is the adopted design. Only the Geolog *export* path needs a magnitude,
   and `21_data-io.md` owns that with the logging obligation attached.

3. **`OPEN-DB-3` — `setinfo.setinfo` fails its own KIND vocabulary in 2 of 131 rows.**
   `REFERENCE→ALL` and `RECEIVER_CHECKSHOT→RECEIVERS` name undefined kinds; `RECEIVERCS` and
   `REFERENCE` are defined and unused; `setinfo.h` publishes `SETINFO_KIND_REFERENCE` as an API
   constant that no row uses. Two readings — `ALL` as an undocumented wildcard versus a data defect,
   `RECEIVERS` as a typo versus an intended distinct kind — and no vendor text adjudicates either.
   **Not blocking for the design** (the KIND layer is the right shape), **blocking for an importer**,
   which must validate and queue rather than coerce. Covered by `SB-DBM-034`'s review-queue contract.

4. **`OPEN-DB-4` — IP's Irregular Set Tolerance: fixed feet, or the well's own depth unit?** A 3.28×
   difference on a metric workflow. **What would settle it:** the local IP 2025 install, read-only —
   a minutes-scale check. **Mitigation in force:** unit-typed storage, explicit conversion at the
   comparison site, snap decision logged (`SB-DBM-024`, §5 note 4).

5. **`O-8.2` — IP Curve Set short name: 8 or 4 characters?** Two pages of the same manual disagree; a
   leading-digit prohibition is stated once. **Mitigation:** clamp to 8, reject a leading digit, log
   every rename. Owned at the export boundary by `21_data-io.md`.

6. **Autosave interval.** No defensible vendor default exists — IP has no autosave at all (its Save
   Reminder minimum is 5 min and non-persistent), Techlog states an option with no interval in the
   pages read, Geolog's Project application writes through. **Jauhar's call**, and it interacts with
   `SB-DBM-036`: an autosave that takes the global lock is a periodic stall.

7. **Materialised interactive-set ceiling.** Must be measured on SandiBumi, not inherited — IP's
   2,000 is its whole working set and Geolog materialises none. **This is the one blocking item in
   the domain** (dossier E-6) and `SB-DBM-T38` is the measurement. Escalated as item 5 below.

8. **Lock timeout.** Only a vendor precedent exists (IP's `IPDBLock` self-clears in 4–5 min, 5-min
   grant window). Not adopted, because a single-writer model has no lock to time out. Recorded so a
   future multi-writer proposal starts from the precedent rather than from a guess.

**Opened by this chapter.**

9. **`SB-CORE-033` — the content-hash compute cache. Assessed, not implemented, per direction.**
   The design is: hash the inputs that determine a computed result and skip recomputation when the
   hash is unchanged. **The assessment: the design still holds, and its correctness now depends on a
   requirement that did not exist when it was parked.** A content hash is only safe if it covers
   *everything* that determines the output — which is exactly the manifest `SB-DBM-015` enumerates.
   Hashing the parameters and input curves alone, as the shipped provenance record would currently
   support, produces a cache that returns a stale result after a module version change
   (`SB-DBM-002`), after a manifest-default change (`SB-DBM-004`), after a physics-driving attribute
   change (`SB-DBM-017`), or after a model is retrained (`SB-DBM-018`) — and a stale cached number is
   `SB-CORE-002`'s failure in its purest form, because nothing about it looks wrong. **Recommendation:
   keep it parked until `SB-DBM-015` ships, then implement the cache key as the manifest hash rather
   than as a bespoke input list.** That also makes it cheap: the manifest is already being computed
   for provenance, so the cache key is a by-product rather than a second source of truth — which is
   `SB-CORE-007` applied to a cache. **Not implemented here.**

10. **`SAMPLING_STYLE_VERIFY_TOLERANCE` has no source.** `SB-DBM-028` needs a tolerance for
    "constant increment" and no vendor publishes one, because no vendor performs the check. §5 note 5
    explains why `IRREGULAR_DEPTH_TOLERANCE` is not a substitute. **What would settle it:** a
    decision on SandiBumi's own data — the depth-step distribution across the delivered-project LAS
    corpus would show what variation real acquisition produces.

11. **`AUDIT_ENTRY_COLLAPSE_WINDOW` has no source.** Geolog's collapsing rule is stated as
    *uninterruptedness*, not elapsed time. `SB-DBM-011` therefore states the rule as the vendor does,
    and an interval would be an invented vendor fact.

12. **The form of a module version (`MODULE_VERSION_SOURCE`) and of an artifact hash
    (`ARTIFACT_HASH_ALGORITHM`).** Both are required by a P0 requirement and neither has a citable
    value, because no incumbent does the thing. Listed so the implementation choice is visible as a
    choice rather than as an inherited default.

13. **What to do with existing `created_at` values when timestamps move to UTC (`SB-DBM-009`).**
    Existing rows were written by DuckDB's `now()` in the authoring machine's local zone, which is
    not recorded. Back-filling by assuming a zone would **invent data**. Escalated as item 7 below.

### 7.2 Escalations — each is a question with a checkable answer

1. **Three candidate `SB-CORE` gaps. I have not minted an identifier for any of them.**
   Each is stated as a question because the answer may be "it is already inside an existing one":

   **(a) Does provenance carrying a *method-derivation citation* need its own `SB-CORE`, or is it
   inside `SB-CORE-004` and `SB-CORE-010`?** `SB-CORE-004` says a *parameter* carries its source;
   `SB-CORE-010` says a *curve* answers how it was made. Neither says the *method* carries its
   primary source. Under the 2026-08-07 contract amendment this is no longer a nicety — it is what
   makes an independently-derived capability distinguishable from a reconstruction (§1.3). It is
   specified here as `SB-DBM-005` [P0]; the question is whether it should also be a core requirement
   binding every chapter, because a per-chapter promise to cite is a documentation practice and a
   column is a contract.

   **(b) Does *custody* of a model artifact need its own `SB-CORE`, or is it inside `SB-CORE-014`?**
   `SB-CORE-014` says a learned model carries its training provenance. It does not say where the
   artifact may come from. `CONTRACT.md` §2.2's C-3 rule — "a vendor-trained model is never consumed
   in any format" — is a constraint on a *schema column*, and it is specified here as `SB-DBM-021`
   [P0]. The question is whether a contract clause is sufficient or whether it needs a core
   requirement, given that the enforcement point is one nullable column in one table.

   **(c) Does "no configuration may disable the provenance record" need its own `SB-CORE`?**
   Specified here as `SB-DBM-013` [P1] on the strength of F-03 — Geolog's audit trail is available
   only on one storage backend, stated on exactly one page of the whole helpset. SandiBumi satisfies
   it today by construction; the requirement exists to stop a future settings screen from
   un-satisfying it. The question is whether that is a data-model rule or a product rule.

2. **`SB-CORE-036`'s as-built evidence is stale, and `04_CORE_REQUIREMENTS.md` should be corrected.**
   That file states the job view carries no `cancellable` flag to check. The shipped code carries it
   on `Job` (`jobs.rs:89`), on `JobView` (`jobs.rs:107`), takes it as an explicit parameter to
   `run_job` (`jobs.rs:266`), passes `false` from `run_simple_job` (`jobs.rs:319`), tests it twice
   (`jobs.rs:527-539`, `:545-578`) and consumes it in the frontend (`src/ipc.ts:896`,
   `src/ui/processingPanel.ts:203`). The honest-finalization half also ships (`jobs.rs:286-290`).
   **The requirement stands; only its evidence has moved.** I may not edit that file, so the
   correction is raised here — the same shape as the `SB-CORE-002` correction already recorded in
   it. **Exact question: should `SB-CORE-036`'s as-built note be updated to "PRESENT-OK, pinned by
   `SB-DBM-040`", or should the requirement be closed?**

3. **`SB-CORE-032`'s measurement has moved and should be restated.** The audit figure it cites is
   "64 of 82 synchronous commands". Measured on the current source: **109 of 130 synchronous
   commands and 17 of 79 async commands** take `db.0.lock()`; `db.0.lock()` appears 128 times in
   `lib.rs`. The requirement is *more* pressing, not less. **Exact question: restate the figure, or
   leave it as the audit-date snapshot it was?** This is the durability point that file itself makes
   about pointers versus requirements, and it applies to its own text.

4. **A read-only pass over the local IP 2025 install would close `OPEN-DB-4` and part of `E-4`.**
   `C:\Program Files\IP2025` is on this machine. The question — whether IP's Irregular Set Tolerance
   is fixed in feet or displayed in the well's depth unit — is minutes-scale and read-only. It is a
   3.28× difference in a snapping tolerance. **Exact question: is a read-only inspection of that
   install authorised for this purpose?** No vendor file would be copied and no lookup-table data
   transcribed.

5. **`E-6` — the interactive-set ceiling must be measured by Jauhar on real wells.** It cannot be
   delegated or inferred: it is a property of SandiBumi's own read path on a real corpus. It is the
   dossier's one blocking item in this domain and it gates `SB-DBM-038` and `SB-DBM-T38`. **Exact
   question: when can `SB-DBM-T38` be run at N ∈ {100, 500, 1000, 2000, 5000}, and is a synthetic
   corpus acceptable above the 850-well maximum the delivered-project record set contains?**

6. **The C-2 acquisition gap: two of the three C-2 items have no named public source in this
   corpus.** `CONTRACT.md` §2.2 names **SPWLA-2021-0091** (Brackenridge et al.) for Experienced Eye /
   EEFS. It names **no** publication for **Domain Transfer Analysis** or for **Textural Facies**.
   Independent derivation requires a primary source, so those two are acquisition gaps rather than
   refusals. **Exact question: are there named publications for DTA and for Textural Facies to
   acquire, and if not, does the capability get derived from first principles under its own name or
   deferred?** Recorded per §2.2.1's instruction to record the specific missing source.

7. **The UTC migration for `created_at` needs a ruling.** Moving to UTC storage (`SB-DBM-009`) leaves
   existing rows written in an unrecorded local zone. Back-filling by assuming the authoring
   machine's current zone would invent data — the same class of error the parameter rules forbid.
   **Exact question: mark every pre-migration timestamp `ZONE_UNKNOWN` and display it as such, or
   leave existing projects on local time and apply UTC only to new records?** The first is honest and
   makes old projects look degraded; the second means one column carries two meanings, which is the
   failure `SB-DBM-041` names.

8. **No second Matthews & Kelly case exists in this chapter, and I want that on the record.** §5's
   45 rows contain **no vendor lookup-table data**. The two rows that come closest are
   `DATASET_STEP_CANDIDATES` — a 20-value list from `DatasetStep.csv` — and `NULL_SCREEN_SUSPECT`.
   Both were examined against `CONTRACT.md` §2.1. The step list is a **candidate enumeration for
   detecting a sampling rate**, not tabulated chart data whose values are its content, and it is
   marked *evidence only, not adopted*; the null list is a set of sentinel magnitudes, which are
   protocol constants rather than measurements. **Neither is offered as an exception and no exception
   is sought.** This paragraph exists because the commission asked that a second case be escalated
   rather than decided, and the honest report is that there is not one.

### 7.3 Refusals — defect refusals only

These are **competitive wins**, not gaps. Each states what SandiBumi does instead and why it is
correct. They discharge `03_EVIDENCE_BASE.md` §14.1.

**One non-adoption is deliberately NOT in this list.** Declining IP's 4–5 minute `IPDBLock` timeout
is not a defect refusal — the timeout is not broken, it simply solves a problem a single-writer model
does not have. It is recorded as open item 8 in §7.1.

| # | Vendor behaviour refused | What SandiBumi does instead | Requirement |
|---|---|---|---|
| **R-1** | Making the audit trail conditional on the storage backend, documented in one sentence on one page of the whole helpset (F-03, T2 `RF01_Database/database_01_overview_hc.1.2.html`) | Provenance is not a feature flag. A failed provenance write fails the curve write in the same transaction. Correct because provenance a deployment choice can silently remove is not provenance | `SB-DBM-013` |
| **R-2** | Testing a null sentinel by equality against a single magnitude, where the vendor's own two sources differ by eight orders of magnitude (F-12, T1 `cgg.h` vs T2 `…hc.5.09.html`) | Threshold detection, `v < −1.0e29`, with the bound **computed** from the export constant. Correct because it is right under both vendor readings, and because an equality screen leaks the other magnitude into the data where no range check on a log scale will catch it | `SB-DBM-030` |
| **R-3** | Honouring a declared sampling style that the tool itself documents it cannot verify (F-14, T2 `…hc.5.11.html`) | Verify the declaration against the reference column at ingest and **contradict** it where false, with a named warning. Correct because the failure has no visual signature: 40 missing rows at 0.1524 m place every later sample 6.1 m shallow and the curve still plots continuous | `SB-DBM-028` |
| **R-4** | Shelling out to a third-party text differ to compare parameter states (F-02, T2 `O` §3.5, §10.11) | Store audit details as name-value pairs so the diff is a `FULL OUTER JOIN`. Correct because a structured diff can be filtered by magnitude and embedded in a deliverable; coloured text can be neither | `SB-DBM-011`, `SB-DBM-012` |
| **R-5** | Silently dropping unmatched rows in bulk operations — include-well fill-with-missing, array auto-averaging, tops paste (F-25, ledger `O-8.8`, dossier §3.8, §3.9) | `{matched, unmatched, ambiguous}` from every bulk operation, with a review queue. Correct because a bulk operation that reports success while having dropped a row is `SB-CORE-002` at scale | `SB-DBM-034` |
| **R-6** | Letting a well-header dropdown re-select tool-response tables and correction charts with no run-record trace (F-05, T2 `O` §3.3) | Declare the attribute as a module input, record its run-time value, mark outputs stale on change, fail loudly when unset. Correct because the vendor's own ingest states the mitigation, and because a wrong contractor selection changes ρma look-ups and ships without an error | `SB-DBM-017` |
| **R-7** | Addressing a persisted parameter by ordinal alone, in a scheme whose ordinals are sparse and append-only (F-18, T2 `O` §2.9) | Dual handle; a disagreement is a load failure naming both. Correct because the single-handle failure is documented as having happened — ledger R-10's ClayVol #41 — and it computed | `SB-DBM-032` |
| **R-8** | Permitting, at any layer, a write into the reference column of a shared frame (F-16; IP's own API says "never write back to the Depth curve", T2 `O` §2.5) | Refuse at the API boundary, naming the frame; a new depth basis is a new frame declared `OWN`. Correct because the failure re-datums every curve on the frame at once and plots as a perfectly continuous log | `SB-DBM-029` |
| **R-9** | Publishing a hand-typed capacity limit in documentation and never reconciling it — a shipped-artefact count printed as a limit for seven years (F-23, ledger `O-8.3`) | Every limit is a unit-typed constant with a source string, and the docs table is generated from the declarations. Correct because the alternative has a seven-year worked example of failing | `SB-DBM-024` |
| **R-10** | Running a one-way per-well upgrade with no backup at all (T2 `O` §4.3) | Refuse to migrate when the pre-migration copy cannot be written; never overwrite an existing backup; exempt additive migrations so the backup that matters is not buried | `SB-DBM-042` |
| **R-11** | Shipping a vocabulary that violates its own foreign key in 2 of 131 rows and leaving the consumer to guess (F-08, `OPEN-DB-3`) | Validate a vocabulary import against the file's own vocabulary and route failures to a review queue. **Specifically: do not fuzzy-match `RECEIVERS`→`RECEIVERCS` and do not auto-create a kind for `ALL`.** Correct because 1.5 % is exactly the failure rate at which a coercing importer looks like it works, and a "clean" import of that file would have invented a vendor fact | `SB-DBM-034` |
| **R-12** | Presenting a returned-row count under a field named as a total (§3.12, `db.rs:3566-3606`) | One field name, one meaning; a page count is named as one and flagged. Correct because a page size displayed as a total is a small instance of the class this whole chapter exists to prevent — and it is SandiBumi's own defect, not a vendor's, which is why it is listed with them | `SB-DBM-041` |

### 7.4 Independent-derivation requirements

Under `CONTRACT.md` §2.2 as amended, a Tier-C item that serves a real user need in this domain must
be specified as an independently-derived SandiBumi capability, not refused. Three of the register's
items touch the data model. Each carries its class, its primary sources, its `Betters:` line and its
owning requirement id.

**The enabling requirement for the whole class is `SB-DBM-005`,** and it belongs here rather than
only in §4. An independently-derived capability and a reconstruction produce the same numbers in the
same columns; the only durable difference is whether the primary source is recorded and travels with
the result. `SB-DBM-005` makes that a column in the project database, which means a method's
derivation is auditable by anyone who opens the file, years later, without access to this repository
or to anyone's memory. Without it, every capability specified below is defensible only by assertion.

---

**D-1 — Model-artifact custody. Class C-3 (opaque artifact).**

*Items in the register:* shipped neural-network weight files.

*Why it touches this domain:* `ml_models.data BLOB` (`db.rs:690`) is precisely the column that would
accept one. The contract's bar — "a vendor-trained model is never consumed in any format" — is
therefore a schema constraint here, not a policy paragraph elsewhere.

*Primary sources:* none required, and that is the definition of the class. There is nothing to derive
from — the internals are not visible and inferring them from behaviour is the prohibited path. The
capability is built natively: SandiBumi trains its own models, in the user's own project, from
identified wells.

*Betters:* the incumbents ship pre-trained weight files whose training data, seed and library
versions are not disclosed, so a user who ships a deliverable computed from one cannot state where
its numbers came from, cannot reproduce them, and cannot audit them. Every SandiBumi model is trained
from identified wells and intervals (`SB-DBM-018`) under a recorded seed and library set
(`SB-DBM-019`), and stamps its identity into every curve it produces (`SB-DBM-020`). The provenance
is complete by construction rather than by vendor disclosure.

*Owning requirement:* **`SB-DBM-021`** [P0]. Verified by `SB-DBM-T21`.

---

**D-2 — Deterministic uncapped parameter-sweep store. Class C-2 (proprietary implementation,
publicly described).**

*Items in the register:* Experienced Eye / EEFS, Domain Transfer Analysis, Textural Facies.

*Why it touches this domain:* all three are, in storage terms, the same object — a harness that
evaluates a method across many parameter combinations and needs somewhere to put the trials. This
chapter owns the store and the provenance; the method halves (which parameters are swept, how a trial
is scored) belong to the chapters that own the petrophysical questions.

*Primary sources:* **SPWLA-2021-0091** (Brackenridge et al.) for Experienced Eye, named in
`CONTRACT.md` §2.2 as a legitimate primary source. The corpus's characterisation of the incumbent —
a brute-force cross-product harness rather than an algorithm, capped at 475 depth levels, sampling
100 depth levels at random where the same vendor's standalone tool uses 200 sorted — was established
by **three exact cross-product reproductions**, i.e. from observable arithmetic on published
behaviour, not from the vendor's internals.

*Betters:* the incumbent's documented limitations are all storage-shaped and all three are removed.
It is **capped at 475 depth levels**; the SandiBumi store is uncapped. It **samples 100 depth levels
at random**, where the same vendor's own standalone tool uses **200 sorted**; the SandiBumi sweep
enumerates in a declared deterministic order, and any subsample is reproducible from a recorded seed
(`SB-DBM-014`). It **records nothing**; every SandiBumi trial is a provenanced row under
`SB-DBM-001`…`SB-DBM-006`.

*Owning requirement:* **`SB-DBM-043`** [P2]. Verified by `SB-DBM-T44`.

*Acquisition gap, recorded per §2.2.1:* **Domain Transfer Analysis and Textural Facies have no named
publication in this corpus.** `CONTRACT.md` §2.2 names a paper for Experienced Eye only. The store
specified by `SB-DBM-043` is method-agnostic and therefore serves all three, but the *methods* for
DTA and Textural Facies cannot be independently derived until a primary source is identified. This is
an acquisition gap, not a refusal, and it is escalation 6 in §7.2. The specific missing sources: a
published description of Domain Transfer Analysis, and a published description of the Textural Facies
method.

---

**D-3 — Omovie Sonic Saturation. Class C-1 (patent-claimed). Not a data-model capability.**

*Item in the register:* Omovie Sonic Saturation, **US 12,242,011 B2**.

*Why it appears here at all:* to record that this chapter examined it and found no data-model
obligation, so that its absence from §4 is a decision rather than an oversight. A saturation method is
a method-chapter capability. Should it be built, it produces curves like any other and is served by
the existing provenance mechanism — with one specific consequence worth naming: `SB-DBM-005` would
record the **patent number itself** as the derivation citation, because reading granted claims is the
correct way to design around and the contract states that doing so is not reconstruction. That makes
the design-around auditable in the project file.

*Primary sources:* the granted claims of US 12,242,011 B2, which are a published public document.
**The claims have since been read** — `REF_patent_US12242011.md` in this directory is that reading,
produced 2026-08-07 under Jauhar's direction — so the contract's "until the claims are read"
precondition no longer gates the decision. Nothing in that analysis creates a data-model obligation:
a design-around, if one is specified, produces curves that the existing provenance mechanism serves.

*Betters:* not stated, because this chapter specifies no capability here. A `Betters:` line is owed
by whichever chapter specifies the design-around.

*Owning requirement:* **none in this chapter.** The decision — design around, license, or drop —
belongs to `CONTRACT.md` §2.2 and to the chapter that owns acoustic saturation. This chapter's only
contribution is `SB-DBM-005`: whichever way it goes, the derivation basis is recorded in the project
database, and for a design-around that basis **is the patent number itself**, which makes the
design-around auditable rather than asserted.

---

## 8. Traceability — dossier disposition

### 8.0 Counting basis

**297 rows**, one for every numbered finding, comparison table, difference, decision-ledger entry,
prior-ledger row, invariant, parameter, adoption-spec line, test, rule, work item, gap, open item,
source-register section and critique-disposition entry in `database-model.md`. The basis:

| Dossier section | Items | Basis |
|---|---|---|
| §1.1 IP | 32 | rows 1–30 plus 14b and 14c |
| §1.2 Techlog | 22 | rows 1–22 |
| §1.3 Geolog | 38 | rows 1–36 plus 23b and 23c |
| §1.4 SandiBumi as-built | 16 | rows 1–12 plus 3b, 9b, 9c, 9d |
| §1.5 SegaraBumi precedent | 1 | the section as one item |
| §2.1–§2.11 | 11 | one row per comparison table |
| §3.1–§3.12 | 12 | one row per difference |
| §4.1 decisions | 26 | D-1 … D-26 |
| §4.2 prior ledger | 12 | R-10, O-8.1–O-8.4, O-8.8, O-OPEN-2, O-OPEN-5, O-OPEN-6, O-OPEN-7/N-9.1, O-OPEN-8, R-9 |
| §5.1–§5.8 | 79 | 13 invariants + 26 parameter rows + 3 spec sections (§5.3–§5.5) + 22 tests + 8 rule bindings + 7 work items |
| §6 | 20 | E-1 … E-12 plus the 8-item authoritative OPEN tally |
| §7.1–§7.7 | 7 | one row per source-register section |
| Critique disposition | 21 | B-1, M-1 … M-8, m-1 … m-11, plus the one item found during re-verification |
| **Total** | **297** | |

**Two places where the dossier's own counts measure different things, stated rather than silently
reconciled.**

1. **The OPEN tally versus the parameter table.** §6's authoritative tally is **8 items** and §5.2
   carries a different mix of `OPEN` and `CONFLICTED` rows. These are not the same count and neither
   is wrong: the tally counts *unresolved questions*, the parameter table counts *fields that ship
   without a value*. `OPEN-DB-3` appears in the tally and has no §5.2 row at all, because a
   vocabulary-import contract is not a scalar. This chapter follows the dossier's own reconciliation
   note and counts both, in §8.10 and §8.11 respectively, rather than deduplicating them.

2. **`T-DB-21` and `T-DB-22` appear in the dossier in reverse order** — `T-DB-22` is listed before
   `T-DB-21` in §5.6. Both are dispositioned under their own ids in §8.10; the ordering anomaly is
   noted so a reviewer diffing the two files does not read it as a missing row.

**Surplus.** 12 requirements in §4 have no dossier antecedent — they come from reading the shipped
source after the dossier was written, or from the 2026-08-07 contract amendment. They are enumerated
in §8.14.

### 8.1 Dossier §1.1 — Interactive Petrophysics (32 items)

| Dossier item | Disposition | Where it went |
|---|---|---|
| 1 — database is a folder, one `.DAT` per well, first-vacant-slot numbering, lock/list/history files | `EVIDENCE-ONLY` | Architectural contrast to the single-file model; informs §1.2's R7 seam note |
| 2 — Well → Curve Sets → Curves; set short name ≤ 8, `Set:Curve` addressing | `ADOPTED` | F-10 → `SB-DBM-024`; the "set" false friend is §1.4 |
| 3 — Regular vs Irregular curve sets; irregular carries its own depth curve | `ADOPTED` | F-14 → `SB-DBM-028`; frame declaration `db.rs:325-328` |
| 4 — Overwrite = replace-and-concatenate (splice), not whole-curve replace | `ADOPTED` | D-15's naming rule → `SB-DBM-034` review-queue contract and §7.3 R-5 |
| 5 — Zone Sets, Pick sets, TVDss sets; one-way Pick→Zone link | `ADOPTED` | `SB-DBM-008` (zone-set identity in the run record) |
| 6 — Parameter Set = extended Zone Set carrying a full parameter vector per zone | `ADOPTED` | `SB-DBM-032`; D-10 |
| 7 — Parameter-set types: two conflicting enumerations, neither a superset | `ESCALATED` | §7.1 item 5 lineage; de-gated by not enumerating types in the schema (D-ledger disposition) |
| 8 — Parameter value polymorphism: constant \| curve \| attribute lookup \| tilted \| log-tilted | `ADOPTED` | F-19 → `SB-DBM-032` (`tilt` is a property of the value) |
| 9 — Tilt interpolation is within-zone only; parameters step at zone boundaries | `ADOPTED` | F-19 → `SB-DBM-032`, `SB-DBM-T32` |
| 10 — Parameter ordinals: permanent, sparse, never renumbered | `ADOPTED` | F-18 → `SB-DBM-032`, `SB-DBM-T32` |
| 11 — Working parameter set auto-persists on save; named sets to DB, `.set`, or project | `EVIDENCE-ONLY` | Informs §7.1 item 6 (autosave) |
| 12 — Global Parameter Sets anchored to Picks not depths | `DEFERRED` | P3 — trigger: a multi-project parameter-reuse requirement |
| 13 — Global (curve) Sets auto-created in every loaded well | `REJECTED` | Auto-creating named sets in every well conflicts with `SB-DBM-001`'s one-record-per-curve rule |
| 14 — Three attribute namespaces (Well / Log / Curve attributes) | `ADOPTED` | `SB-DBM-017` (the physics-driving subset) |
| 14b — IP raises a visible warning on unmapped loader mnemonics | `EVIDENCE-ONLY` | A vendor doing the right thing; the equivalent obligation is `21_data-io.md`'s |
| 14c — documented metadata→physics path (Logging Contractor) | `ADOPTED` | **F-05** → `SB-DBM-017`, `SB-DBM-T17`, §7.3 R-6 |
| 15 — Curve resolution chain: input set → alias modes → Final filter → type/MRU | `ADOPTED` | **F-04** → `SB-DBM-006`, `SB-DBM-T08` |
| 16 — History module: 6 flat columns, ExamDiff differencing, IP2025 SQL Row Filter | `ADOPTED` | **F-01**, **F-02** → `SB-DBM-003`, `SB-DBM-011`, `SB-DBM-012`, §7.3 R-4 |
| 17 — Pessimistic well lock `IPDBLock`, self-clears 4–5 min | `EVIDENCE-ONLY` | §7.1 item 8 (lock timeout, precedent recorded, not adopted) |
| 18 — Optimistic Multi-User Access, 5-minute grant, four conflict classes | `DEFERRED` | P4 — trigger: a multi-writer requirement; D-17 |
| 19 — Well Security Manager, per-well encrypted ACL | `DEFERRED` | P4 — and the encryption half is `27_ip-install-blockers.md`'s R7 |
| 20 — IP Query: cross-project index in a SQL CE `.sdf` | `EVIDENCE-ONLY` | Cross-project search precedent; F-28 is the in-house answer |
| 21 — Projects vs Well Lists; 10 recent entries; Save Reminder minimum 5 min | `ADOPTED` | §5 `AUTOSAVE_INTERVAL` (`ABSENT`); recents limit is SandiBumi's own 12 (`project.rs:113`) |
| 22 — Defaults precedence: corporate > project > user | `DEFERRED` | P3 — D-19; no SandiBumi corporate-search-folder concept yet |
| 23 — Database Upgrader backs up per folder, named by source version | `ADOPTED` | **F-07** → `SB-DBM-042`, `SB-DBM-T43` |
| 24 — "IP does NOT provide forward compatibility" | `ADOPTED` | `SB-DBM-042`, `SB-DBM-T01`; D-20 |
| 25 — Deployment footprint, .NET, minimum spec | `EVIDENCE-ONLY` | Packaging is `27_ip-install-blockers.md`'s |
| 26 — Array curves: X across × Z sub-depth samples | `EVIDENCE-ONLY` | `array_logs` (`db.rs:258-287`) already carries a typed axis; F-22 for paging |
| 27 — Well & Set Grouping, parent-well grouping for sidetracks | `ADOPTED` | `SB-DBM-037` (backend scoping over `well_groups`) |
| 28 — IP 2025 raised Mineral Solver max user models 20 → 50; Python 3.12 | `EVIDENCE-ONLY` | Method-chapter territory |
| 29 — parameters and curves are the same object to the API | `EVIDENCE-ONLY` | Informs `SB-DBM-032`'s value polymorphism; not adopted as an identity rule |
| 30 — PPFG mandated set-name standard and explicit depth-uniqueness contract | `ADOPTED` | **F-26** → `SB-DBM-026`, `SB-DBM-T25` |

### 8.2 Dossier §1.2 — Techlog (22 items)

| Dossier item | Disposition | Where it went |
|---|---|---|
| 1 — four-level addressing `(database, well, dataset, variable)` | `EVIDENCE-ONLY` | SandiBumi's equivalent is `(project file, well_id, set, curve)`; §1.4's vocabulary note |
| 2 — `TLP` folder vs `TLPX` single compressed file | `EVIDENCE-ONLY` | Single-file is already SandiBumi's model (`db.rs:29`) |
| 3 — project unit system with default `None` = variable storage unit | `DEFERRED` | P3 — trigger: a project-level unit-policy requirement; interacts with `SB-DBM-031` |
| 4 — recent projects by modification date; last opened reopens | `ADOPTED` | Shipped — `project.rs:99-135`; §5 `RECENT_PROJECTS_LIMIT` = 12 |
| 5 — one-directional version compatibility | `ADOPTED` | `SB-DBM-042`, `SB-DBM-T01` |
| 6 — variable object model incl. per-variable `HistoryItem(dateTime, userName, description)` | `ADOPTED` | **F-01** → `SB-DBM-003`, `SB-DBM-011` |
| 7 — array variables with mutable `columnCount`/`columnSize` | `EVIDENCE-ONLY` | `array_logs` axis model (`db.rs:258-287`) |
| 8 — `CacheVarData` explicit paged large-data path | `EVIDENCE-ONLY` | **F-22** → §5 `LARGE_ARRAY_PAGE_BYTES`, recorded not adopted |
| 9 — zonation is a dataset | `EVIDENCE-ONLY` | §1.4's "set" false friend; SandiBumi keeps zones as their own tables (`db.rs:368-388`) |
| 10 — index dataset linking every dataset across reference domains | `ADOPTED` | `SB-DBM-031` (depth datum + reference frame) |
| 11 — composite well with five explicit update modes and variable recovery | `ADOPTED` | The *named modes* principle → `SB-DBM-034`, §7.3 R-5; the composite object itself is `DEFERRED` P4 |
| 12 — object storage levels Project / User / Company | `DEFERRED` | P4 — no multi-tier storage in a single-file desktop model |
| 13 — shared parameter keys harmonising a value across methods | `ADOPTED` | `SB-DBM-025` (a constant crossing a module boundary is registered) |
| 14 — multi-user = Studio repository replication, not file locking | `EVIDENCE-ONLY` | D-17; §7.1 item 8 |
| 15 — hierarchical locks (`Lock all` / `Lock` / descendants) | `DEFERRED` | P4 — with D-17 |
| 16 — version-based conflict detection, four item states | `DEFERRED` | P4 — with D-17 |
| 17 — conflict-resolution methods by object level | `DEFERRED` | P4 — with D-17 |
| 18 — import buffer is read-only for large-project performance | `EVIDENCE-ONLY` | Informs `SB-DBM-038`'s materialisation rule |
| 19 — project tainting is irreversible under an Ocean Core License | `REJECTED` | An irreversible one-way project state is the opposite of `SB-DBM-042`'s posture |
| 20 — shipped reference catalogs: 2,181 curve families, 54 main | `EVIDENCE-ONLY` | Vocabulary source; D-22. **No catalog data transcribed** |
| 21 — `DatasetStep.csv`, 20 canonical steps, unit header `m` | `ADOPTED` | §5 `DATASET_STEP_CANDIDATES`, marked evidence-only, not a constraint |
| 22 — well identification solver for same-name imports | `EVIDENCE-ONLY` | Import identity is `21_data-io.md`'s; the store-side rule is `SB-DBM-034` |

### 8.3 Dossier §1.3 — Geolog (38 items)

| Dossier item | Disposition | Where it went |
|---|---|---|
| 1 — Project = application directory + well database, two independently located halves | `EVIDENCE-ONLY` | SandiBumi is one file; the split is why F-20's read model works |
| 2 — hierarchy Project → Wells → Sets → {Constants, Comments, Logs}; each set owns one reference log | `ADOPTED` | §1.4's adopted sense of "set"; `SB-DBM-028`, `SB-DBM-029` |
| 3 — wells per project unlimited, disk-bound | `ADOPTED` | D-3 — SandiBumi sets no hard cap; §5 `INTERACTIVE_SET_CEILING` is a *materialisation* limit, not a project cap |
| 4 — well identity = PWI | `ADOPTED` | `SB-DBM-018`'s id-not-name principle; SandiBumi's internal identity is a UUID (`db.rs:204-225`) |
| 5 — sampling style is a property of the set, one per set | `ADOPTED` | **F-14** → `SB-DBM-028`, `SB-DBM-T27` |
| 6 — `WELL_HEADER` special set for constants and comments | `EVIDENCE-ONLY` | SandiBumi keeps header fields as `wells` columns (`db.rs:204-225`) |
| 7 — `AUDIT_TRAIL` reserved set, protected from cut/delete/rename | `ADOPTED` | `SB-DBM-011`, `SB-DBM-013` (the protection principle) |
| 8 — `REFERENCE` set, one per well, always starts at depth zero | `ADOPTED` | `SB-DBM-031` (reference frame per well) |
| 9 — directional survey set with mandatory `CALC_METHOD` constants | `EVIDENCE-ONLY` | `well_path`/`well_surveys` (`db.rs:773-792`); the method is a survey-chapter matter |
| 10 — interval sets: names plus start depths, end = next start | `ADOPTED` | `SB-DBM-033`'s `TOPS` style; `tops`/`zones` (`db.rs:358-374`) |
| 11 — log versioning is intrinsic; `<log>_N`, `_1` = original | `ADOPTED` | **F-06** → `SB-DBM-002`, `SB-DBM-018`, `SB-DBM-035` |
| 12 — log resolution priority: default set → `setinfo` order → latest version → alias order | `ADOPTED` | **F-04** → `SB-DBM-006` (version participates in the tie-break) |
| 13 — name/size limits table | `ADOPTED` | **F-09** → §5 rows; export clamping is `21_data-io.md`'s |
| 14 — data types REAL / DOUBLE / ALPHA / INT32 / INT8 / INT16 / UINT8 / UINT16 | `ADOPTED` | **F-15** → `SB-DBM-033` (a genuine categorical type) |
| 15 — 30+ per-LOG database attributes | `EVIDENCE-ONLY` | Informs `curve_meta` (`db.rs:739-758`); no attribute list transcribed |
| 16 — one file per well in a `wells` directory | `EVIDENCE-ONLY` | Architectural contrast; SandiBumi is single-file |
| 17 — well files never shrink on delete; repack on close at a `WELL_FULL` threshold | `ADOPTED` | **F-21** → §5 `COMPACT_THRESHOLD_FRACTION`; the mechanism ships (`db.rs:934-956`) |
| 18 — two applications, two memory models (Well loads, Project does not) | `ADOPTED` | **F-20** → `SB-DBM-038`, `SB-DBM-T38` |
| 19 — summary lists are an explicit cache | `ADOPTED` | D-5 — an explicitly refreshed cache; `SB-DBM-038`'s query-not-load rule |
| 20 — Include Projects: symbolic cross-project well links, editable in the including project | `DEFERRED` | P4 — and D-25's integrity checker is the prerequisite (`SB-DBM-027`) |
| 21 — five directory trees plus a search list | `DEFERRED` | P3 — with D-19 defaults precedence |
| 22 — Epos / PNS project registration over CORBA | `EVIDENCE-ONLY` | **F-27**'s context; no distributed model in SandiBumi |
| 23 — Epos access permissions, three tabs, finer than per-well | `DEFERRED` | P4 — D-18; the re-read that corrected M-2 |
| 23b — a separate coarser mechanism that must not be confused with row 23 | `EVIDENCE-ONLY` | Recorded so the D-18 deferral rests on the corrected reading |
| 23c — documented stand-alone mode with no PNS server | `EVIDENCE-ONLY` | Corroborates F-27: the single-user path is the vendor's own |
| 24 — audit trail is structured, UTC-stored, two-level | `ADOPTED` | **F-01** → `SB-DBM-009`, `SB-DBM-011`, `SB-DBM-012` |
| 25 — GeologSQL over the well database | `ADOPTED` | D-21 — the SQL panel ships (`db.rs:3566-3606`); the two borrowed ideas are `DEFERRED` P3 |
| 26 — Well Inventory: per-well counts including audit-entry count | `ADOPTED` | `SB-DBM-027`'s report-every-class shape |
| 27 — Well Catalog with rolled-up Date Modified and saved searches | `DEFERRED` | P3 — a `23_plotting-interactivity.md` surface over this chapter's queries |
| 28 — shipped `setinfo.setinfo`: 131 set names + 45-row KIND vocabulary | `ADOPTED` | **F-08** → `SB-DBM-034`, §7.3 R-11; **no vocabulary data transcribed** |
| 29 — batch/CLI access `log_dbms`, `log_db_check` | `EVIDENCE-ONLY` | E-11's integration seam; not in scope |
| 30 — `well_include_check` reports both dangling classes | `ADOPTED` | **D-25** → `SB-DBM-027`, `SB-DBM-T26` |
| 31 — "Handling Missings" states a different sentinel from the header | `ADOPTED` | **F-12** → `SB-DBM-030`, §5 `NULL_GEOLOG_MAGNITUDE` (`NON-ADOPTABLE`) |
| 32 — documented Geolog silent failure on missing ASCII depths | `ADOPTED` | **F-14**, **F-25** → `SB-DBM-028`, `SB-DBM-034`, §7.3 R-3 and R-5 |
| 33 — vendor's worked example maps a Geolog missing to −999.25 on export | `EVIDENCE-ONLY` | Export mapping is `21_data-io.md`'s; §5 cross-reference row |
| 34 — PYGG shipped Python API with an explicit audit lifecycle | `ADOPTED` | **D-26** — auditing is explicitly started and flushed; `SB-DBM-011` |
| 35 — Well Data Server: one project, one user, one machine; started on demand | `ADOPTED` | **F-27** → `SB-DBM-036`'s rationale for keeping the single writer |
| 36 — no stated limit on samples per log | `ADOPTED` | D-3's no-hard-cap posture; `SB-DBM-038` measures rather than caps |

### 8.4 Dossier §1.4 — SandiBumi as-built (16 items)

Every row here was **re-verified against the source on 2026-08-07** before being dispositioned, per
the commission's rule that a line pointer repeated from another document must be re-checked. All
sixteen resolve as the dossier states them.

| Dossier item | Disposition | Where it went |
|---|---|---|
| 1 — one project = one DuckDB file; the startup file is chosen, not hard-coded | `ADOPTED` | §3.1, §3.6; `project.rs:126-135`; `SB-DBM-042` |
| 2 — 33 tables | `ADOPTED` | §3.12 — and the count is why `TABLE_SPECS`'s 8-table whitelist is a gap; `SB-DBM-041` |
| 3 — format-version gate already implemented | `ADOPTED` | §3.1; `db.rs:117-167`; `SB-DBM-042`, `SB-DBM-T01` |
| 3b — backup-before-destructive-migration also already implemented | `ADOPTED` | §3.6; `db.rs:908-932`; `SB-DBM-042`, `SB-DBM-T02` |
| 4 — `computed_curves` deliberately has no primary key (3.7×, 311k vs 1.16 M rows/s) | `ADOPTED` | §3.3; §5 `ART_INDEX_INSERT_PENALTY`; **the write-discipline gap** → `SB-DBM-026`, `SB-DBM-027` |
| 5 — `array_logs` does carry a PK; one row holds a whole vector | `ADOPTED` | §3.3 table; the asymmetry is deliberate and is recorded, not flagged |
| 6 — log-set versioning exists with module/params/inputs/created_at/frame | `ADOPTED` | §3.4, §3.5 — the mechanism is `PRESENT-OK`, the record contents are `PARTIAL`; group A |
| 7 — field-scale cost measured: ~15-min migration on the 2.5 GB reference project | `EVIDENCE-ONLY` | §3.10's scale argument; E-9's calibration |
| 8 — open item #129: Rayon over wells defeated by the single global mutex | `ADOPTED` | **§3.10** → `SB-DBM-036`, `SB-DBM-T36`, `SB-DBM-T38` |
| 9 — project picker, switching and recent list all shipped | `ADOPTED` | §3.6; `project.rs:99-135`; the dossier's §5.8 item 5 was struck for this |
| 9b — the recents list lives outside the project database | `ADOPTED` | §5 `RECENT_PROJECTS_LIMIT`; `project.rs:33-61`, `:113` |
| 9c — `save_project_as` is an engine copy and therefore also a compaction | `ADOPTED` | §3.6; `db.rs:934-956`; **F-21**'s hook |
| 9d — `compact_project` carries the numbers that make `WELL_FULL` concrete | `ADOPTED` | §5 `COMPACT_THRESHOLD_FRACTION`; `project.rs:219-227` (`CompactReport`) |
| 10 — startup is staged; an empty in-memory placeholder is handed to the builder | `ADOPTED` | §3.10; `lib.rs:3122`; the swap-under-the-mutex model in `project.rs`'s module doc |
| 11 — well groups already exist | `ADOPTED` | **§3.11** → `SB-DBM-037` (the gap is enforcement, not existence) |
| 12 — scale exercised vs scale expected: 100-well × 4-module chain at 21 s | `EVIDENCE-ONLY` | E-9; calibrates `SB-DBM-T38`'s N range |

### 8.5 Dossier §1.5 — SegaraBumi precedent (1 item)

| Dossier item | Disposition | Where it went |
|---|---|---|
| §1.5 — the sibling data-foundation spec, cited as in-house precedent | `ADOPTED` | **F-28** → §5 `INTERACTIVE_QUERY_TARGET` (< 50 ms, own design target); `SB-DBM-038`'s feasibility argument |

### 8.6 Dossier §2.1–§2.11 — comparison tables (11 items)

| Dossier item | Disposition | Where it went |
|---|---|---|
| §2.1 — object hierarchy, the "set" false friend | `ADOPTED` | **§1.4** of this chapter — the vocabulary warning, stated because a requirement written against the wrong sense of "set" loses either the frame or the parameter state |
| §2.2 — capacity limits, every number any tool states | `ADOPTED` | §5 rows `MAX_*`; **F-23** → `SB-DBM-024` |
| §2.3 — name and identity limits in characters | `ADOPTED` | **F-09**, **F-10** → §5; export clamping is `21_data-io.md`'s |
| §2.4 — null/missing sentinels, the highest silent-wrongness surface | `ADOPTED` | **F-11**, **F-12**, **F-13** → `SB-DBM-030`, `SB-DBM-T29`, `SB-DBM-T30` |
| §2.5 — parameter model | `ADOPTED` | **F-18**, **F-19** → `SB-DBM-032` |
| §2.6 — sampling / interpolation declaration | `ADOPTED` | **F-14**, **F-15** → `SB-DBM-028`, `SB-DBM-033` |
| §2.7 — depth-reference / datum model | `ADOPTED` | **F-17** → `SB-DBM-031` |
| §2.8 — audit trail, three schemas side by side | `ADOPTED` | **F-01** — the table is reproduced in §2.1 of this chapter, because the empty "source of a value" row is the chapter's thesis |
| §2.9 — concurrency and multi-user, three architectures | `ADOPTED` | **F-27** → `SB-DBM-036`'s rationale; D-17 defers the rest |
| §2.10 — configuration and defaults precedence | `DEFERRED` | P3 — D-19; no corporate-search-folder concept in SandiBumi |
| §2.11 — cross-project data reuse | `DEFERRED` | P4 — with Geolog row 20 and D-25's prerequisite checker |

### 8.7 Dossier §3.1–§3.12 — differences that matter (12 items)

| Dossier item | Disposition | Where it went |
|---|---|---|
| §3.1 — six null values, two test styles, one guaranteed corruption | `ADOPTED` | **F-12** → `SB-DBM-030`, `SB-DBM-T29`, §7.3 R-2 |
| §3.2 — Curve Sets per well 500 vs 50 | `EVIDENCE-ONLY` | Ledger `O-8.1`; SandiBumi sets no per-well set cap (D-3) |
| §3.3 — an 8-character set name is the tightest constraint any tool imposes | `ADOPTED` | **F-10** → §5; export path is `21_data-io.md`'s |
| §3.4 — wells at scale: 9,999 hard / 2,000 in memory vs disk-bound vs unstated | `ADOPTED` | **F-20** → `SB-DBM-038`; §5 `INTERACTIVE_SET_CEILING` (`ABSENT`) |
| §3.5 — concurrency: file lock vs versioning vs replication, and what each costs | `ADOPTED` | **F-27** → `SB-DBM-036`; D-17 |
| §3.6 — parameter identity: ordinal vs shared key vs typed manifest | `ADOPTED` | **F-18** → `SB-DBM-032`, `SB-DBM-T32` |
| §3.7 — overwrite semantics: one word, three behaviours | `ADOPTED` | D-15's naming rule → `SB-DBM-035` (restore creates a version, never rewinds) |
| §3.8 — array/sub-sample data: silent averaging vs explicit paging | `ADOPTED` | **F-25** → `SB-DBM-034`, §7.3 R-5; **F-22** for the paging precedent |
| §3.9 — cross-project linking: the include-well trap | `ADOPTED` | **F-25**, **D-25** → `SB-DBM-027`, `SB-DBM-034` |
| §3.10 — numeric typing of stored samples | `ADOPTED` | **F-15** → `SB-DBM-033`; §3.15 records that every SandiBumi curve store is `FLOAT` |
| §3.11 — the declared sampling style can be a lie; Geolog documents its own silent failure | `ADOPTED` | **F-14** → `SB-DBM-028` [P0], `SB-DBM-T27`, §7.3 R-3 |
| §3.12 — project metadata that silently changes physics: IP's Logging Contractor | `ADOPTED` | **F-05** → `SB-DBM-017`, `SB-DBM-T17`, §7.3 R-6 |

### 8.8 Dossier §4.1 — decision ledger D-1 … D-26 (26 items)

| Dossier item | Disposition | Where it went |
|---|---|---|
| D-1 — container hierarchy Project → Well → Set → Curve, set owns a declared frame | `ADOPTED` | Shipped — `db.rs:316-329` (`frame` declared, never inferred); §3.4 |
| D-2 — well identity: internal UUID + declared PWI + alias table | `ADOPTED` | UUID ships (`db.rs:204-225`); the PWI and alias halves inform `SB-DBM-018` |
| D-3 — no hard cap on wells; disk-bound | `ADOPTED` | `SB-DBM-038` measures rather than caps; §5 `MAX_WELLS_IP_DATABASE` is an export guard only |
| D-4 — two-mode split: materialised interactive set, non-materialising bulk | `ADOPTED` | `SB-DBM-038` [P2], `SB-DBM-T38`; the dossier's own blocking item E-6 |
| D-5 — summary/inventory caching as an explicitly refreshed command | `DEFERRED` | P3 — trigger: `SB-DBM-038` lands and a cache becomes worth invalidating |
| D-6 — null discipline: SQL `NULL` in store, screen the sentinel family on import | `ADOPTED` | `SB-DBM-030` [P0], `SB-DBM-T29`; parse half is `21_data-io.md`'s |
| D-7 — "no parameter supplied" is a first-class distinct state | `ADOPTED` | `SB-DBM-003`, `SB-DBM-007`, `SB-DBM-T30` |
| D-8 — parameter identity: ordinal + semantic key, mismatch = load error | `ADOPTED` | `SB-DBM-032`, `SB-DBM-T32`; ledger R-10 is the proof |
| D-9 — adopt IP's parameter value polymorphism wholesale | `ADOPTED` | `SB-DBM-032`'s value model (constant \| curve \| lookup \| tilted) |
| D-10 — parameters hang off zone rows; the zone set is referencable | `ADOPTED` | `SB-DBM-008` (zone-set identity in the run record); `zone_params` ships (`db.rs:378-388`) |
| D-11 — sampling style declared per set, one mode per set | `ADOPTED` | `SB-DBM-028` [P0]; the enumeration is stated in the requirement |
| D-12 — a distinct categorical curve type, never linearly interpolated | `ADOPTED` | `SB-DBM-033` [P2], `SB-DBM-T33`; deferred in priority, not in principle |
| D-13 — one per-well reference table spanning all domains, MD primary, datum declared | `ADOPTED` | `SB-DBM-031`, `SB-DBM-T31` |
| D-14 — structured name-value audit entries, not a text log | `ADOPTED` | `SB-DBM-011`, `SB-DBM-012`, `SB-DBM-T11`, `SB-DBM-T12` |
| D-15 — never use the word "overwrite"; name the mode | `ADOPTED` | `SB-DBM-035` (restore creates a version); the shipped comment already says "never overwrite" (`db.rs:312-315`) |
| D-16 — keep "re-run = version N+1"; make it the concurrency and safety net | `ADOPTED` | Shipped — §3.4; `SB-DBM-035` pins it |
| D-17 — single-writer today; advisory lock + optimistic grant if multi-user arrives | `ADOPTED` | `SB-DBM-036` keeps the single writer; §7.1 item 8 holds the precedent, unadopted |
| D-18 — per-well permission as the minimum unit, SET as the next level | `DEFERRED` | P4 — corrected by critique M-2 before deferral; encryption half is `27_ip-install-blockers.md`'s |
| D-19 — defaults precedence corporate → project → user | `DEFERRED` | P3 — no corporate-scope concept exists yet |
| D-20 — one-directional format compatibility; refuse a newer file loudly; retired fields stay reserved | `ADOPTED` | `SB-DBM-042`, `SB-DBM-T01`; shipped at `db.rs:117-167` |
| D-21 — keep the SQL panel; borrow per-column unit coercion and result typing | `ADOPTED` (panel) / `DEFERRED` P3 (the two ideas) | `SB-DBM-041` fixes the panel's `total_rows` defect first |
| D-22 — Techlog catalogs primary, Geolog's as cross-check | `ADOPTED` | `SB-DBM-034`'s validate-and-queue rule; **F-08** is why the cross-check cannot coerce |
| D-23 — typed axis metadata, explicit paging, never silent averaging | `ADOPTED` | Axis ships (`db.rs:258-287`, `axis BLOB` NULL = absent); paging `DEFERRED` P3 with **F-22** |
| D-24 — compaction is an explicit command with a stated threshold, gated and verified | `ADOPTED` | Mechanism ships (`db.rs:934-956`, `project.rs:219-227`); threshold is §5 `COMPACT_THRESHOLD_FRACTION` |
| D-25 — any reference that can dangle ships with an integrity checker | `ADOPTED` | `SB-DBM-027` [P1], `SB-DBM-T26` |
| D-26 — auditing is explicitly started and flushed, not ambient | `ADOPTED` | `SB-DBM-011`'s lifecycle; Geolog row 34 (PYGG) is the source |

### 8.9 Dossier §4.2 — prior-ledger disposition (12 items)

| Dossier item | Disposition | Where it went |
|---|---|---|
| **R-10** — parameter ordinals as stable cross-file handles | `ADOPTED` | **F-18** → `SB-DBM-032` [P1], `SB-DBM-T32`, §7.3 R-7. The ClayVol #41 case is quoted as the proof |
| **O-8.1** — Curve Sets per well, 500 vs 50 | `EVIDENCE-ONLY` | An IP-internal limit with no SandiBumi analogue (D-3); the 131-set re-verification that corrected m-1 is carried in §2.3 |
| **O-8.2** — Curve Set short name, 8 vs 4 characters, plus the leading-digit rule | `ESCALATED` | §7.1 item 5; §5 `MAX_SET_NAME_CHARS_IP_EXPORT` = `NON-ADOPTABLE`; clamp to 8 |
| **O-8.3** — lithology shadings 39 vs 80 (a shipped bitmap count published as a limit) | `ADOPTED` | **F-23** → `SB-DBM-024`, `SB-DBM-T24`, §7.3 R-9. The cautionary case, not the number |
| **O-8.4** — two parameter-set type enumerations, neither a superset | `REJECTED` | De-gated by not enumerating parameter-set types in the schema at all; the dossier's own §4.2 disposition |
| **O-8.8** — multi-well tops paste silently ignores unmatched wells | `ADOPTED` | **F-25** → `SB-DBM-034` [P1], `SB-DBM-T34`, §7.3 R-5 |
| **O-OPEN-2** — 500 or 50, residual doubt | `EVIDENCE-ONLY` | Folded into `O-8.1`; no independent action, per the dossier |
| **O-OPEN-5** — Zone Sets per well = 500, absent from the 2018 page | `EVIDENCE-ONLY` | An IP-internal limit; no SandiBumi analogue |
| **O-OPEN-6** — are `Splice`, `MonteCarlo`, `TVDss_Set` persisted types? | `REJECTED` | Same de-gating as `O-8.4`; E-5 records that no source settles it |
| **O-OPEN-7 / N-9.1** — IP's unread vocabularies (`DefaultAlias.cax`, `UnitsConversion.par`, `CparmDef.xml`, `SetDictionary.xml`) | `ESCALATED` | §7.2 escalation 4 — a read-only pass over the local IP 2025 install would close it |
| **O-OPEN-8** — the fixed Well/Log/Curve Attribute name lists exist only as CHM rasters | `ESCALATED` | §7.2 escalation 4, same pass. Relevant here because **F-05**'s attribute class needs the list to be enumerable |
| **R-9** — null conventions, IP and the LAS standard mutually corroborating | `ADOPTED` | **F-13** → §5 `NULL_SCREEN_SUSPECT`; the critique's M-1 correction (−9999 **is** in R-9) is carried, not the pre-correction text |

### 8.10 Dossier §5 — adoption spec (79 items)

**§5.1 invariants (13).**

| Dossier item | Disposition | Where it went |
|---|---|---|
| Inv 1 — a set declares its sampling style and its frame; neither is inferred | `ADOPTED` | `SB-DBM-028`; the frame half already ships (`db.rs:325-328`) |
| Inv 2 — a categorical curve is never linearly interpolated | `ADOPTED` | `SB-DBM-033`, `SB-DBM-T33` |
| Inv 3 — missing data and "no parameter supplied" are different states | `ADOPTED` | `SB-DBM-003`, `SB-DBM-007`, `SB-DBM-030`, `SB-DBM-T30` |
| Inv 4 — every depth quantity declares its datum; TVDSS positive down | `ADOPTED` | `SB-DBM-031`, `SB-DBM-T31` |
| Inv 5 — zone-parameter interpolation is within-zone only | `ADOPTED` | `SB-DBM-032` (`tilt` on the value); `SB-DBM-T32` |
| Inv 6 — no bulk operation may skip a row silently | `ADOPTED` | `SB-DBM-034`, `SB-DBM-T34`, §7.3 R-5 and R-11 |
| Inv 7 — curve resolution is a logged decision | `ADOPTED` | `SB-DBM-006` [P0], `SB-DBM-T08` |
| Inv 8 — published capacity limits are emitted from the source of truth | `ADOPTED` | `SB-DBM-024`, `SB-DBM-T24`, §7.3 R-9 |
| Inv 9 — a declared sampling style is verified on ingest and the verdict stored | `ADOPTED` | `SB-DBM-028` [P0], `SB-DBM-T27` |
| Inv 10 — the large-negative null family is detected by threshold, never equality | `ADOPTED` | `SB-DBM-030` [P0], `SB-DBM-T29`, §7.3 R-2 |
| Inv 11 — an attribute that drives physics is a run-record input | `ADOPTED` | `SB-DBM-017`, `SB-DBM-T17`, §7.3 R-6 |
| Inv 12 — two samples may not share a depth; the resolution is declared | `ADOPTED` | `SB-DBM-026`, `SB-DBM-T25`; the PK-less interaction is §3.3 |
| Inv 13 — a module never writes to the reference column of the frame it reads | `ADOPTED` | `SB-DBM-029`, `SB-DBM-T28`, §7.3 R-8 |

**§5.2 parameter table (26 rows).** Every row is carried into §5 of this chapter with its value
transcribed byte-exact, or is marked as a cross-reference where another chapter owns it.

| Dossier item | Disposition | Where it went |
|---|---|---|
| `NULL_WRITE_LAS` = −999.25 | `ADOPTED` | §5, marked cross-reference — `21_data-io.md` owns the write path |
| `NULL_SCREEN_SUSPECT` = {−999, −999.0, −999.00, −999.25, −9999, −99} | `ADOPTED` | §5; `SB-DBM-030`'s store-side flag-never-coerce rule |
| `NULL_GEOLOG_FLOAT_THRESHOLD` = `v < −1.0e29` | `ADOPTED` | §5; `SB-DBM-030` [P0], `SB-DBM-T29` — the exact-boundary assertion included |
| `NULL_GEOLOG_MAGNITUDE` — CONFLICTED | `ESCALATED` | §5 as `NON-ADOPTABLE — cited for verification`; §7.1 item 2 (`OPEN-DB-2`) |
| `NULL_GEOLOG_INT` = −2147483647 | `ADOPTED` | §5 |
| `PARAM_NOT_SUPPLIED_GEOLOG` = −2147483646 | `ADOPTED` | §5; **F-11** → `SB-DBM-003`, `SB-DBM-007` |
| `IRREGULAR_DEPTH_TOLERANCE_FT` = 0.2 ft, unit scope OPEN | `ESCALATED` | §5 with note 4; §7.1 item 4 (`OPEN-DB-4`); §7.2 escalation 4 |
| `IRREGULAR_TIME_TOLERANCE_S` = 0.5 s | `ADOPTED` | §5 |
| `MAX_SET_NAME_CHARS_GEOLOG_EXPORT` = 32 | `ADOPTED` | §5, cross-reference to `21_data-io.md` |
| `MAX_LOG_NAME_CHARS_GEOLOG_EXPORT` = 32 / 29 recommended | `ADOPTED` | §5, cross-reference |
| `MAX_UNITS_CHARS_GEOLOG_EXPORT` = 16 | `ADOPTED` | §5, cross-reference |
| `MAX_PWI_CHARS_GEOLOG_EXPORT` — 250 vs 32 | `ESCALATED` | §5 as `NON-ADOPTABLE`; §7.1 item 1 (`OPEN-DB-1`) |
| `MAX_SET_NAME_CHARS_IP_EXPORT` — 8 vs 4 | `ESCALATED` | §5 as `NON-ADOPTABLE`; §7.1 item 5 (`O-8.2`) |
| `MAX_WELLS_IP_DATABASE` = 9,999 | `ADOPTED` | §5, marked export-path guard only |
| `MAX_SAMPLES_PER_SET_IP` = 3,000,000 | `ADOPTED` | §5, export-path guard only |
| `MAX_CURVES_PER_WELL_IP` = 20,000 | `ADOPTED` | §5, export-path guard only |
| `MAX_LOGGING_RUNS_IP` = 25 | `ADOPTED` | §5, export-path guard only |
| `COMPACT_THRESHOLD_FRACTION` = 0.75 | `ADOPTED` | §5; **F-21**; the mechanism ships at `db.rs:934-956` |
| `LARGE_ARRAY_PAGE_BYTES` = 10,000,000 | `EVIDENCE-ONLY` | §5, marked evidence only — the sole vendor precedent for an array page size |
| `DATASET_STEP_CANDIDATES_M` — 20 values | `EVIDENCE-ONLY` | §5, marked evidence only; examined against CONTRACT §2.1 in §7.2 escalation 8 |
| `INTERACTIVE_QUERY_TARGET_MS` < 50 | `ADOPTED` | §5, marked **own design target, not a vendor value**; `SB-DBM-038` |
| `AUDIT_TIMESTAMP_STORAGE` = UTC, display local | `ADOPTED` | §5; `SB-DBM-009`, `SB-DBM-T11` |
| `WELL_NAME_NORMALISATION_UPPERCASE` = true on Geolog export | `ADOPTED` | §5, cross-reference to `21_data-io.md` |
| Autosave interval — OPEN | `ESCALATED` | §5 `ABSENT — ships with no default`; §7.1 item 6, Jauhar's call |
| Max wells held materialised — OPEN | `ESCALATED` | §5 `ABSENT`; §7.1 item 7, §7.2 escalation 5; `SB-DBM-T38` measures it |
| Lock timeout — OPEN | `EVIDENCE-ONLY` | §5 `ABSENT`; §7.1 item 8, precedent recorded and explicitly not adopted |

**§5.3–§5.5 spec sections (3).**

| Dossier item | Disposition | Where it went |
|---|---|---|
| §5.3 — the curve-resolution contract, five stages, decision logged | `ADOPTED` | `SB-DBM-006` [P0], `SB-DBM-T08`; the `rule` vocabulary comes from here |
| §5.4 — parameter-file format: dual handle, `tilt`, mandatory `source`, append-only ordinals | `ADOPTED` | `SB-DBM-003` [P0], `SB-DBM-032`, `SB-DBM-T05`, `SB-DBM-T32` |
| §5.5 — audit-entry schema, Geolog's taxonomy adopted directly | `ADOPTED` | `SB-DBM-011`, `SB-DBM-012` — the schema is reproduced verbatim in the requirement |

**§5.6 tests T-DB-01 … T-DB-22 (22).**

| Dossier item | Disposition | Where it went |
|---|---|---|
| T-DB-01 — null round-trip matrix across formats | `ADOPTED` | Split: the store half is `SB-DBM-T29`/`T30`; the format half is `21_data-io.md`'s |
| T-DB-02 — Geolog float-null threshold with the exact boundary | `ADOPTED` | `SB-DBM-T29`, including the `MISS_FLOAT/10.0`-computed-not-typed assertion |
| T-DB-03 — sentinel separation, parameter vs sample | `ADOPTED` | `SB-DBM-T30` |
| T-DB-04 — ordinal/key mismatch is a hard error | `ADOPTED` | `SB-DBM-T32` |
| T-DB-05 — splice vs replace under named modes | `ADOPTED` | Folded into `SB-DBM-T35` (restore creates a version) and `SB-DBM-T34`; the "no path named overwrite" rule is D-15 |
| T-DB-06 — bulk tops paste, 3 unmatchable and 2 ambiguous of 100 | `ADOPTED` | `SB-DBM-T34`, with the counts carried verbatim |
| T-DB-07 — categorical resample at 0.1524 → 0.1 m | `ADOPTED` | `SB-DBM-T33` |
| T-DB-08 — depth-datum declaration, MD vs TVDSS | `ADOPTED` | `SB-DBM-T31` |
| T-DB-09 — export-path name clamping | `DEFERRED` | Owned by `21_data-io.md`; §5's six cross-reference rows carry the values |
| T-DB-10 — format-version gate | `ADOPTED` | `SB-DBM-T01`, labelled `CHARACTERIZATION` |
| T-DB-11 — one-way upgrade takes a backup, three cases | `ADOPTED` | `SB-DBM-T02`, labelled `CHARACTERIZATION`; the source-naming residual became `SB-DBM-T43` |
| T-DB-12 — interactive-set scale curve at N ∈ {100…5000} | `ADOPTED` | `SB-DBM-T38`, labelled `CHARACTERIZATION` |
| T-DB-13 — curve-resolution logging with three GR curves | `ADOPTED` | `SB-DBM-T08` |
| T-DB-14 — unit-typed limits and generated docs | `ADOPTED` | `SB-DBM-T24` |
| T-DB-15 — audit diff without an external differ | `ADOPTED` | `SB-DBM-T12`, with the process-table assertion added |
| T-DB-16 — sampling-style verification with a 40-row gap | `ADOPTED` | `SB-DBM-T27`, with the 6.1 m assertion carried |
| T-DB-17 — reference-integrity checker, two dangling classes | `ADOPTED` | `SB-DBM-T26`, extended to report zero-count classes explicitly |
| T-DB-18 — Geolog export null magnitude | `DEFERRED` | Owned by `21_data-io.md`; §5's `NULL_GEOLOG_MAGNITUDE` row states the export rule |
| T-DB-19 — attribute-drives-physics is a run-record input | `ADOPTED` | `SB-DBM-T17` |
| T-DB-20 — depth uniqueness, continuous vs point | `ADOPTED` | `SB-DBM-T25` |
| T-DB-21 — reference column is not module-writable | `ADOPTED` | `SB-DBM-T28`. *(Listed after T-DB-22 in the dossier; see §8.0.)* |
| T-DB-22 — vocabulary-import referential integrity, 2 of 131 unresolvable | `ADOPTED` | `SB-DBM-T34`'s review-queue contract and §7.3 R-11; the no-fuzzy-match rule is stated explicitly |

**§5.7 `FINDINGS.md` rule bindings (8).**

| Dossier item | Disposition | Where it went |
|---|---|---|
| Rule 3 — unit-typed quantities, no magic constants | `ADOPTED` | `SB-DBM-024`, §5 note 4 |
| Rule 6 — null discipline, extended to the Geolog family | `ADOPTED` | `SB-DBM-030` |
| Rule 7 — ordinal + semantic-name parameter addressing | `ADOPTED` | `SB-DBM-032` |
| Rule 9 — defaults are cited or absent | `ADOPTED` | §5's 8 `ABSENT` rows and their notes; `SB-DBM-003`'s `REQUIRED_UNSET` state |
| Rule 10 — docs generated from code | `ADOPTED` | `SB-DBM-024`, `SB-DBM-T24` |
| Rule 13 — state the reference convention | `ADOPTED` | `SB-DBM-031` |
| Rule 14 — silent failures are bugs | `ADOPTED` | `SB-DBM-034`, `SB-DBM-039`, §7.3 R-5 |
| Rule 15 — curve resolution and depth snapping are logged decisions | `ADOPTED` | `SB-DBM-006`, `SB-DBM-026` |

**§5.8 ordered work list (7).**

| Dossier item | Disposition | Where it went |
|---|---|---|
| 1 — extend the null screen, as a threshold not an equality set | `ADOPTED` | `SB-DBM-030` [P0] |
| 1b — verify declared sampling style on ingest | `ADOPTED` | `SB-DBM-028` [P0] — ranked with item 1 by the dossier, and P0 here for the same reason |
| 2 — ~~add the backup-before-one-way-upgrade step~~ **struck: already implemented** | `ADOPTED` (as a pin) | `SB-DBM-042`, `SB-DBM-T02`; the replacement work (name by source version) is `SB-DBM-T43` |
| 3 — publish the scale curve before touching open item #129 | `ADOPTED` | `SB-DBM-T38`; §7.2 escalation 5 |
| 4 — adopt the audit schema alongside `log_sets` | `ADOPTED` | `SB-DBM-011` — additive, exactly as the dossier frames it |
| 5 — ~~project picker + recent list~~ **struck: shipped**; residual is the interactive/bulk split | `ADOPTED` | The residual → `SB-DBM-036`, `SB-DBM-038`; §3.6 records the picker as shipped |
| 6 — categorical curve type, deferred until a facies deliverable needs it | `DEFERRED` | `SB-DBM-033` [P2], with the dossier's caveat that no further code assume `FLOAT` |

### 8.11 Dossier §6 — gaps, escalations and the OPEN tally (20 items)

| Dossier item | Disposition | Where it went |
|---|---|---|
| E-1 — Geolog file locking promised but undocumented (downgraded) | `ADOPTED` | **F-27** → `SB-DBM-036`'s rationale; the residual unknown changes no decision |
| E-2 — Geolog well-name length 250 vs 32 | `ESCALATED` | §7.1 item 1; §5 `NON-ADOPTABLE` |
| E-3 — Techlog states no capacity limits at all (verified negative over 3,808 pages) | `EVIDENCE-ONLY` | **F-20**'s supporting evidence; `SB-DBM-038`'s no-cap posture |
| E-4 — IP's own vocabularies remain unread | `ESCALATED` | §7.2 escalation 4 — the local IP 2025 install would close it read-only |
| E-5 — IP parameter-set types persisted or transient | `REJECTED` | De-gated: the schema does not enumerate parameter-set types |
| E-6 — SandiBumi's interactive-set ceiling is unmeasured | `ESCALATED` | §7.1 item 7, §7.2 escalation 5, `SB-DBM-T38`. **The one blocking item** |
| E-7 — Geolog `.info` manifest format known from a memory note, not re-read | `EVIDENCE-ONLY` | The manifest leg of D-8 is a design idea, not a numeric parameter; no requirement rests on it |
| E-8 — Techlog's `TechlogDatabase` native module is not readable | `EVIDENCE-ONLY` | Reverse-engineering it is out of bounds under CONTRACT §2.2; explicitly not attempted |
| E-9 — no cross-tool project-scale precedent from delivered work | `EVIDENCE-ONLY` | Calibrates `SB-DBM-T38`'s N range; the corrected ~5-minute-open reading is used, not the retracted one |
| E-10 — Geolog's missing magnitude stated two ways | `ESCALATED` | §7.1 item 2; `SB-DBM-030`'s threshold is correct under both |
| E-11 — Geolog chapters 04–07 read but not systematically mined | `EVIDENCE-ONLY` | Nothing in §4 or §5 depends on the unmined pages |
| E-12 — `setinfo.setinfo` fails its own KIND vocabulary | `ESCALATED` | §7.1 item 3; **F-08** → §7.3 R-11 |
| OPEN 1 — `OPEN-DB-1` PWI length | `ESCALATED` | §7.1 item 1 |
| OPEN 2 — `OPEN-DB-2` null magnitude | `ESCALATED` | §7.1 item 2 |
| OPEN 3 — `OPEN-DB-3` KIND vocabulary | `ESCALATED` | §7.1 item 3 |
| OPEN 4 — `OPEN-DB-4` tolerance unit scope | `ESCALATED` | §7.1 item 4 |
| OPEN 5 — `O-8.2` IP set-name length | `ESCALATED` | §7.1 item 5 |
| OPEN 6 — autosave interval | `ESCALATED` | §7.1 item 6 |
| OPEN 7 — materialised interactive-set cap | `ESCALATED` | §7.1 item 7 |
| OPEN 8 — lock timeout | `EVIDENCE-ONLY` | §7.1 item 8 |

### 8.12 Dossier §7 — source register (7 items)

| Dossier item | Disposition | Where it went |
|---|---|---|
| §7.1 — T1 executable/machine-readable vendor source | `EVIDENCE-ONLY` | Tier assignments in this chapter's front matter and in every §2 finding |
| §7.2 — T2 full-manual ingests and shipped reference manuals | `EVIDENCE-ONLY` | Same |
| §7.3 — T3 install-tree / catalog ingests at page granularity | `EVIDENCE-ONLY` | Same; **F-24** is this chapter's only T3-sourced parameter and is flagged as such in §5 |
| §7.4 — T4 claims recorded as claims, not used as evidence | `EVIDENCE-ONLY` | Used once, for scale calibration (E-9); never as a parameter source |
| §7.5 — compliance statement | `EVIDENCE-ONLY` | Rewritten by critique m-2; the corrected version is what this chapter relied on |
| §7.6 — verification pass 2026-08-06 | `ADOPTED` | Its corrections are carried, not the pre-correction text — notably m-10's line ranges, re-verified again at source on 2026-08-07 |
| §7.7 — revision pass 2026-08-06, post-adversarial-review | `ADOPTED` | Same; the struck §5.8 items 2 and 5 are dispositioned as struck in §8.10 |

### 8.13 Critique disposition (21 items)

The dossier's `## Critique disposition` is authoritative over any body text it corrects, per CONTRACT
§4 rule 2. Every one of these was applied in its corrected form.

| Dossier item | Disposition | Where it went |
|---|---|---|
| **B-1** — §1.4 as-built stale and wrong (startup path, picker, `save_project_as`) | `ADOPTED` | §3 of this chapter was written from the source, not from §1.4; all 16 §1.4 rows re-verified 2026-08-07 (§8.4) |
| **M-1** — ledger R-9 misquoted; −9999 **is** in R-9 | `ADOPTED` | §5 `NULL_SCREEN_SUSPECT` carries the corrected attribution; **F-13** |
| **M-2** — D-18 adjudicated on a scope-stripped quote | `ADOPTED` | D-18 is `DEFERRED` on the corrected reading (§8.8), with Geolog rows 23 and 23b both carried |
| **M-3** — §2.2 upgraded an example-program remark into a capacity statement | `ADOPTED` | Geolog row 36 is dispositioned as a *no stated limit*, not as an asserted one (§8.3) |
| **M-4** — the Logging-Contractor silent-physics path was dropped | `ADOPTED` | Restored and made **F-05**, one of this chapter's two most consequential findings → `SB-DBM-017` |
| **M-5** — `O` §4.6 claimed as used but contributing nothing | `ADOPTED` | **F-26** (0.01 ft FPRESS) and IP row 30's depth-uniqueness contract → `SB-DBM-026` |
| **M-6** — `O` §2.5 unrepresented; "never write back to the Depth curve" missing | `ADOPTED` | **F-16** → `SB-DBM-029`, `SB-DBM-T28`, §7.3 R-8 |
| **M-7** — a misquote with a minutes→seconds unit substitution feeding a priority argument | `ADOPTED` | E-9's **corrected** reading is used: the one measured full-project figure is a ~5-minute open, which is why `SB-DBM-036` and `SB-DBM-038` are prioritised as read-path work |
| **M-8** — `setinfo.setinfo` fails its own KIND vocabulary in 2 of 131 rows | `ADOPTED` | **F-08** → §7.3 R-11; `SB-DBM-034`'s validate-and-queue rule |
| **m-1** — §4.2's O-8.1 row said 130 sets | `ADOPTED` | 131 used throughout (**F-08**, §8.3 row 28) |
| **m-2** — §7.5's compliance statement was templated | `ADOPTED` | §8.12 dispositions the rewritten version |
| **m-3** — unit scope not established on `IRREGULAR_DEPTH_TOLERANCE_FT` | `ADOPTED` | **F-24**, §5 note 4, `OPEN-DB-4`; the T3 tier is stated wherever the value appears |
| **m-4** — T-DB-02 left the threshold boundary unpinned | `ADOPTED` | `SB-DBM-T29` asserts −1.0e29 is **data**, and that the bound is computed not typed |
| **m-5** — the OPEN-DB-2 reconciliation hypothesis was stated wrongly | `ADOPTED` | §5's `NULL_GEOLOG_MAGNITUDE` row records the conflict without a reconciling hypothesis |
| **m-6** — three mutually inconsistent open-item counts | `ADOPTED` | §8.0 states the two counting bases rather than silently reconciling them; §7.1 carries the authoritative 8 |
| **m-7** — D-decisions out of order; E-10/E-11 orphaned | `ADOPTED` | §8.8 dispositions a clean D-1…D-26; §8.11 carries E-1…E-12 |
| **m-8** — §7.2's addendum characterisation wrong in detail | `EVIDENCE-ONLY` | No requirement in this chapter rests on the addendum |
| **m-9** — "untested above ~2,000 wells" was an inference stated as fact | `ADOPTED` | §3.10 and E-9 state only measured figures; the 2,000-well number is used as the *decision context* for the PK removal, which is what `db.rs:295` actually says |
| **m-10** — line-range citations drifted on `check_and_stamp_format` | `ADOPTED` | §3.1 cites doc comment `:108-116` and function `:117-167`, **re-verified at source 2026-08-07** |
| **m-11** — D-4 over-attributed the well-list primitive to Geolog | `ADOPTED` | **F-20** attributes the *two-application memory split*, which is what the vendor documents, not a well-list primitive |
| Found during re-verification (not in the critique) | `ADOPTED` | Carried in its corrected form wherever it touches §2 or §5 |

### 8.14 Surplus — requirements with no dossier antecedent (12)

These come from reading the shipped source after the dossier was written, or from the 2026-08-07
contract amendment. They are listed so that a reviewer diffing §4 against the dossier does not read
them as unsourced.

| Requirement | Origin |
|---|---|
| `SB-DBM-002` — module identity by version | §3.5's audit of `workflow.rs:694-706`: the dossier records that `log_sets.module` exists, not that it holds a **name** |
| `SB-DBM-004` — effective parameters, not overrides | §3.5, from reading `build_opts` (`workflow.rs:276-295`) against the write at `:694-706` |
| `SB-DBM-005` — method-derivation citation | **The 2026-08-07 contract amendment** (`CONTRACT.md` §2.2, §2.2.1). Post-dates the dossier entirely |
| `SB-DBM-007` — absent is a named state | §3.5, from `equations.rs:1231-1238` writing `params_json: String::new()` |
| `SB-DBM-009` — UTC provenance timestamps | §3.5, from `db.rs:324` (`DEFAULT now()` is local). The dossier records Geolog's UTC policy but not SandiBumi's divergence |
| `SB-DBM-010` — provenance travels into the deliverable | `SB-CORE-010`'s **scope resolution of 2026-08-07** |
| `SB-DBM-013` — provenance is not configurable off | Generalised from F-03 after §3's reading showed SandiBumi satisfies it only by construction |
| `SB-DBM-018` … `SB-DBM-022` — the model store (5 requirements) | §3.8's audit of `ml_models` (`db.rs:675-692`) and the two divergent write paths (`ml.rs:670-675`, `:944-953`). The dossier's §1.4 inventory does not reach `ml_models`; `SB-DBM-021` additionally derives from CONTRACT §2.2's C-3 class |
| `SB-DBM-023` — one vocabulary registry | §3.14, from the two `STANDARD_COLUMNS` declarations (`equations.rs:299`, `curve_edit.rs:81-88`) |
| `SB-DBM-039` — degraded results persist beyond the job registry | §3.9 and `jobs.rs:119` — the dossier does not cover the job model |
| `SB-DBM-041` — a total is a total; the inspector exposes provenance | §3.12, from `db.rs:3476-3495` and the two meanings of `TablePage.total_rows` |
| `SB-DBM-043` — deterministic uncapped sweep store | **The 2026-08-07 contract amendment**, C-2 class (§7.4 D-2) |
