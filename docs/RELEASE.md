# SandiBumi — Release Policy: what 1.0 means, how versions work, what the product promises across them

**Version 0.1 of this document · 2026-07-29 · applies to product version `0.1.0`**

Produced by Prompt 3 of `docs/product_definition_prompt.md`, from `docs/PRD.md` (reviewed
2026-07-29), `docs/V1_SCOPE.md`, `REVIEW.md`, `ROADMAP.md`, `db.rs` and `CONTRIBUTING.md`. Per the
V1_SCOPE sequencing note, the quality bar was derived there first; **this document adopts
V1_SCOPE §5 verbatim and does not re-derive it.**

House rule carried from the PRD (§0.2) because this document is where it bites hardest: **nothing
below describes an aspiration in the present tense.** Every mechanism is marked **TRUE TODAY**
(measured in the code, cited) or **REQUIREMENT** (written down here for the first time, not yet
built). A release policy that blurs the two is itself a degraded result presented as clean.

---

## 1. The 1.0 bar

Adopted from `V1_SCOPE.md` §5 — seven binary items, all required, none traded:

| # | Bar | Today |
|---|---|---|
| Q1 | Every V1_SCOPE §2 capability field-verified by a human against real data | NOT MET |
| Q2 | Zero open Critical items in ROADMAP §4b | MET (re-check at release) |
| Q3 | One green-gate command (`tools/check.ps1`) | NOT MET |
| Q4 | Clean-machine install, exercised off the dev machine | NOT MET |
| Q5 | The 2000-well claim demonstrated or deleted | NOT MET |
| Q6 | Numbers-that-changed ledger (this document, §4) | policy now exists; no releases yet |
| Q7 | R1/R2/R3 lawyer answers + R10 support boundary written | NOT MET |

This file additionally contributes **Q3's definition** (§5 step 0), **Q6's mechanism** (§4), and
two REQUIREMENTS (§3) that join the 1.0 gate because compatibility promises cannot be retrofitted
after files exist in the field: **R-A (format version stamp)** and **R-B (pre-migration backup)**.

---

## 2. Versioning

Semantic versioning, adapted to the fact this product's user never sees an API. **The
compatibility contract is not the code — it is the project file.** A `.duckdb` project holds
months of interpretation; it must open next year, on a different machine, after an upgrade the
user did not choose. Version numbers exist to make promises about *that*, and only incidentally
about features.

**TRUE TODAY:** the version is `0.1.0`, declared in **three places that nothing keeps in
agreement** — `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `package.json`.
**REQUIREMENT R-C:** the release checklist (§5) treats `tauri.conf.json` as the single source and
verifies the other two against it; drift fails the release.

### 2.1 The axes

| Bump | Meaning — in file terms first, feature terms second |
|---|---|
| **MAJOR** | The project-file format changes such that an **older app can no longer safely read a newer file**. Nothing else forces MAJOR. Adding tables/columns does *not* (older apps ignore them safely — see §3 for why that is currently a lie and what makes it true); changing the meaning or units of existing stored data *does*. |
| **MINOR** | New capability. A project written by this version still opens in any app of the same MAJOR (possibly with new features' data invisible to older apps, per the R-A rules). **Also the floor for any results-affecting change — see 2.2.** |
| **PATCH** | Fixes only. No schema change, no results-affecting change, no new capability. A PATCH is the only release a client may install without reading the changelog, so this definition is strict: if a fix changes any number, it is not a PATCH. |

### 2.2 The petrophysics-specific rule — results-affecting changes

A change to a **default parameter, a physics constant, a method's semantics, or a cutoff's
treatment of missing data** changes the numbers a user gets from unchanged inputs. This class is
easy to miss because it is often a one-line diff and always a correct fix. The rule:

1. **It is at least MINOR, never PATCH** — even a one-character constant correction.
2. **It must appear in the changelog's "Numbers that changed" section (§4), by module name, with
   the reason and the expected direction/magnitude of the change.**
3. **It never alters already-stored results.** Computed curves persisted by an older version stay
   as computed; the new behaviour applies on re-run. The versioned log-set machinery
   (**TRUE TODAY**: `log_sets` version-per-run, append-only archive, `db.rs:133-155`) is exactly
   the mechanism that makes this cheap — a re-run under the new version is version N+1 beside the
   old, never an overwrite.

Why the rule earns its prominence: this repo has already lived it. The pay-summary PERM semantics
fix (AUDIT-2026-07-20: a missing PERM now *fails* an active cutoff) changed real net-pay numbers,
correctly, and was documented loudly. Someone may have put the pre-fix number in a reserves
report; the fix must still ship; what is never optional is saying so at the top of the notes. A
reserves number that silently moves between versions is the commercial form of the cardinal rule.

**Not results-affecting** (for clarity of the boundary): rendering, export formatting, dialog
layout, performance with identical outputs, new optional curves alongside unchanged existing ones.

---

## 3. Project-file compatibility policy

The most important section for a licensed product, and the one where today's honest status is
weakest.

### 3.1 Forward — an OLDER app opens a NEWER file

**The only acceptable behaviours:** refuse with a clear, named-versions message ("this project was
written by SandiBumi 1.3; this is 1.1 — upgrade to open it"), or migrate. **Silently misreading is
the one unacceptable option** — it is the cardinal rule with the user's whole project as the blast
radius: the app would show a plausible interpretation missing whatever the newer version stored.

**TRUE TODAY — and this is the finding: refusal is currently *impossible*.** Measured 2026-07-29:
`db.rs` stamps **no format version anywhere** — no metadata table, no pragma, nothing an older
binary could inspect. Every table is created `IF NOT EXISTS` and read by name. An older app
opening a newer file today would open it, find the tables it knows, ignore the rest, and present
the result as the whole project. The failure mode is not hypothetical; it is the default.

**REQUIREMENT R-A (gates 1.0):** a `project_meta` table written at project creation and updated by
every migration, holding at minimum `format_version INTEGER` and `written_by TEXT` (app version).
On open: `file format_version > app's known format_version` → **refuse, naming both versions**;
`≤` → open, running migrations as needed. The MAJOR axis in §2.1 then has a physical meaning:
MAJOR bumps `format_version` past what older apps accept; MINOR may add tables/columns without
bumping it. Cost: one table, one check, a handful of lines — the reason it gates 1.0 is not
difficulty but that **it only protects files created after it exists.**

### 3.2 Backward — a NEWER app opens an OLDER file

**TRUE TODAY, and genuinely good:** migration-on-open is an established house pattern with two
shipped precedents, both idempotent by *probing actual state* rather than trusting a flag —
`migrate_drop_computed_curves_pk` (re-checks `duckdb_constraints()` on every launch, `db.rs:457`)
and `migrate_standard_curves_to_generic_store` (per-well `curve_migration_done` bookkeeping,
~instant on an already-migrated project, `db.rs:379`). New-feature columns arrive via
`ALTER ... IF NOT EXISTS`-style additive steps (the `set_id` addition, `db.rs:134`). This pattern
— probe, migrate, never trust a marker you didn't verify — is the standard for all future
migrations.

**The gap — REQUIREMENT R-B (gates 1.0):** migrations mutate the file **in place with no backup**.
The house precedent for recoverable copies exists one layer down (WAL recovery moves the corrupt
WAL aside as a timestamped `.corrupt-backup-<ts>`, never deleting it) but does not cover
migrations. Rule: **any migration that rewrites or drops data** (the PK rebuild is the shipped
example) first copies the project file to `<name>.pre-<format_version>-backup.duckdb` beside it,
and says so in the launch log. Purely additive migrations (new empty tables/columns) are exempt —
a backup per column addition would train users to ignore backups.

### 3.3 The support matrix this yields

| File written by | Opened by older app | Opened by newer app |
|---|---|---|
| Same MAJOR | opens; newer features' data dormant (safe once R-A exists) | opens; additive migrations run silently |
| Newer MAJOR | **refused with named versions** (R-A) | — |
| Older MAJOR | — | opens; migrations run; backup taken if destructive (R-B); the file is now the newer format — one-way, stated in the message |

---

## 4. Changelog policy

`CHANGELOG.md`, repo root, one entry per released version, newest first. **Audience: the
interpreting petrophysicist and their manager (PRD §3.1/§3.2) — written in petrophysics terms,
never commit subjects.** "Improved robustness of workflow.rs" is a commit message; "Pay Summary:
wells with no permeability curve are now excluded from net pay when a PERM cutoff is active —
totals may drop" is a changelog entry.

**Mandatory sections, in this order:**

1. **⚠ Numbers that changed** — always present; states "None." when true, so its absence can
   never be ambiguous. Each entry: module name, what changed, why, expected direction/size, and
   what to do about prior work (typically: results stored by earlier versions are untouched;
   re-run to adopt). This section exists for the §3.2 user holding last quarter's number.
2. **New** — capability additions, in job terms.
3. **Fixed** — user-visible fixes only.
4. **Preview** — additions/changes to the V1_SCOPE §3 preview tier, labelled as such, so the
   supported/preview boundary is re-stated release after release rather than eroding.
5. **Compatibility** — format_version if bumped, migrations that run on open, backups taken.

**TRUE TODAY:** no `CHANGELOG.md` exists — nothing has been released. The raw material for a
future backfill is `REVIEW.md`'s 89 rounds; do not backfill speculatively — the changelog starts
at the first versioned release.

---

## 5. The release checklist

The literal sequence, executable by one person in an afternoon. Steps marked ⚙ are automatable
and should end up inside `tools/` scripts; the rest are eyes-on by design.

**0. Green gate ⚙ — Q3, defined here:** `tools/check.ps1` runs, in order, `npx tsc --noEmit`,
`npm run build`, and `cargo test` through the pinned toolchain (vcvars 14.29 on the reference
machine, per CLAUDE.md), exiting non-zero on the first failure. **TRUE TODAY: this script does not
exist** — the three gates are run by hand. Writing it is the cheapest item on the 1.0 bar and a
precondition for every step below meaning anything.

1. **Verification status check.** Every V1_SCOPE §2 capability's REVIEW/matrix row is `[x]`, or
   the release stops here (Q1). Re-confirm Q2 (no open Critical items) against today's ROADMAP,
   not memory.
2. **Version stamp ⚙.** Decide the bump per §2 (results-affecting ⇒ ≥ MINOR; format break ⇒
   MAJOR + `format_version`). Set `tauri.conf.json`; verify `Cargo.toml` and `package.json` agree
   (R-C).
3. **Changelog.** Write the §4 entry now, while the diff is fresh — the "Numbers that changed"
   section first, from the release's results-affecting list (which step 2 already forced you to
   compile).
4. **Build ⚙.** `npm run tauri build` on the pinned toolchain. The dev loop never exercises the
   packaged app, so nothing before this step has tested what ships.
5. **Clean-machine pass** (Q4). Install the bundle on hardware that is not the dev machine,
   following `CONTRIBUTING.md` alone. Exercise: import a real LAS batch, run one chain, export
   one report — plus the packaged-only surfaces: the CSP paths (Vega panel, equation editor,
   plot print — REVIEW.md Round 89's `Try:` line, which cannot be tested in dev), and the
   Python-absent degradation story on a machine without Python.
6. **Compatibility drill.** Open a project file created by the *previous* release in the new
   build: migrations run, backup appears if destructive (R-B), nothing lost. Once R-A exists,
   also confirm the previous release *refuses* a new-format file with the correct message.
7. **Tag and archive ⚙.** `git tag v<version>`; archive the installer with a SHA-256 hash
   alongside the changelog entry. The hash is what a client's IT verifies (PRD §3.3) and what
   answers "which exact build is deployed at client X" a year later (R10's first question).
8. **File the ledger.** Append the release (version, date, format_version, hash, numbers-changed
   summary) to a running table at the top of `CHANGELOG.md`. This table *is* Q6.

---

## 6. Support window

One paragraph, per the prompt — because the absence of this paragraph is what turns a single
support request into an open-ended obligation:

**Proposed (Jauhar decides — this is PRD §8/R10 territory):** fixes land on the **latest MINOR of
the latest MAJOR only**; a customer reporting a defect on an older version is supported by
upgrading (free within a MAJOR, per whatever §8 licence terms say across MAJORs). **Project files
are readable forever forward** — every release must migrate any file back to format_version 1
(§3.2), so "upgrade to get the fix" never costs data, which is what makes latest-only supportable
by one person. What remains for Jauhar in writing before first sale: response expectation
(best-effort vs named response time), the working-hours/holiday boundary, and the channel — R10
stays open until those sentences exist.

---

## Acceptance

Accepted when Jauhar confirms: the §2.2 results-affecting rule (≥ MINOR + mandatory ledger entry,
stored results never silently altered), **R-A and R-B joining the 1.0 gate**, the §5 checklist as
the release ritual, and §6's proposal as the starting position for the support terms. The next
document is nothing — the product-definition suite is complete; what follows is Jauhar's
reconciliation of `ROADMAP.md` against `V1_SCOPE.md` §4, and then the work itself.
