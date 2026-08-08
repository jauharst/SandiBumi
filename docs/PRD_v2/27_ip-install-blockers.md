# 27. Installation, deployment and runtime blockers — requirements

> **Dossier:** `docs/research_2026-08/cross_tool/ip-install-blockers.md` — 3,478 lines — read in full 2026-08-08
>
> **Critique:** `docs/research_2026-08/cross_tool/ip-install-blockers_critique.md` — 633 lines — read in full 2026-08-08
>
> **Evidence tiers held:** T1 install-tree and source-code evidence; T2 manuals; T3 adjacent-tool installation evidence
>
> **Requirements:** 26 · **P0:** 11 · **Parameters:** 18 (5 `ABSENT`) · **Acceptance tests:** 30

## 1. Scope and boundary

This chapter owns whether a released SandiBumi desktop package can be installed, upgraded,
diagnosed and removed on a managed Windows estate without hidden prerequisites or mutable files in
the installation directory. It owns the boundary between the native application and optional
Python-backed capabilities, the dependency/capability manifest, immutable-template versus per-user
configuration, corporate policy precedence, reproducible support evidence, and release claims about
what the installer contains.

**`DIO` — file formats.** `21_data-io.md:91-96` assigns commercial acceptability of the Python
prerequisite here. `DIO` continues to own the format-local failure and recovery semantics for DLIS,
spreadsheet plate extraction and Office-format delivery. This chapter owns installing, detecting and
explaining the runtime that those paths require.

**`MLA` — machine learning and equations.** This chapter owns runtime discovery, package inventory
and deployment. `MLA` owns algorithms, model provenance and feature-level numerical tests.

**`CUT`, `MIN`, `NMR`, `GEO`, `FSR`, `SAT` — scientific parameters and methods.** The dossier closed
or corrected scientific values in all six domains. None becomes an installation default here. Each
value is routed to its method chapter, whose parameter discipline and acceptance tests govern it.

**`CORE` and `DBM` — projects and persistence.** This chapter owns upgrade/uninstall preservation,
configuration migration and diagnostics. The core and database chapters own the schema, transaction
and recovery contracts inside a project.

**`PLI` — plotting.** This chapter owns packaging of fonts and other runtime resources. `PLI` owns
rendering correctness and interaction.

## 2. What the incumbents do — the requirement-bearing findings

### 2.1 Installed templates and live user settings are different evidence

**Finding INS-1 (T1, dossier §2.6).** The incumbent separates a build-generated application config
in the installation directory from a live per-user settings store. It also distinguishes files
copied into the user profile from files migrated from a previous version. The live settings file is
mutable and version-inherited: its size changed during the dossier revision, so a count read from it
cannot be asserted as a factory default without a clean-install copy and a digest. **Obligation:**
SandiBumi needs immutable shipped templates, explicit first-run materialisation, migration semantics,
and provenance for every effective setting.

### 2.2 Positional registries need both an ordinal and a semantic key

**Finding INS-2 (T1, dossier §§2.4, 3.8).** One incumbent help registry has 220 positional entries,
seven duplicate labels and no explicit parameter-number tags. Two neighbouring registries do carry
explicit ordinals. Names also drift between files while retaining their referent. **Obligation:** a
SandiBumi configuration pack must carry a stable semantic identifier and an ordinal; duplicate names,
missing ordinals and identifier/ordinal disagreement must be load errors rather than guessed joins.

### 2.3 String-only unit plumbing permits dimension changes

**Finding INS-3 (T1/T3, dossier §§2.3, 2.13, 2.16).** Five installed curve-type populations contain
416, 52, 76, 408 and 8 members respectively and do not form one canonical vocabulary. Only one of
the 408 live curve types has a dimensional category. A bridge maps a permeability symbol to a length
symbol; an adjacent tool's declared permeability dimension proves that mapping is dimensionally
wrong. Six empty-to-empty mapping rows and three distinct missing-unit encodings create further
silent-success paths. **Obligation:** units must be typed before conversion; dimensional mismatch,
blank mappings and unknown symbols must not silently pass.

### 2.4 Lexical normalisation can hide evidence

**Finding INS-4 (T1, dossier §§2.1, 2.3).** Shipped configuration includes CP1252 text in an otherwise
plain-text defaults file and case-variant unit spellings. A UTF-8-only parser fails on the former;
an unconditional case fold hides the latter. **Obligation:** configuration ingestion must declare
its encoding, preserve the observed token for provenance, and apply only registry-declared
equivalences.

### 2.5 A module tree is a public parameter surface, not an implementation

**Finding INS-5 (T1, dossier §§2.15, 3.4).** Sixty module directories expose plain-text `Parameters`
registries while calculation libraries remain compiled. The plain registries are inspectable
configuration evidence; compiled coefficients are not. **Obligation:** SandiBumi must expose its own
module parameters through a versioned public schema while refusing to obtain methods or defaults by
binary inspection.

### 2.6 Optional runtime dependencies must be stated per capability

**Finding INS-6 (source audit and `DIO` seam).** The native executable has no link-time Python
dependency and can launch without Python (`python_engine.rs:5-11`). The equation engine nevertheless
requires Python 3.10+ with NumPy (`python_engine.rs:47-48`); SciPy is optional
(`python_engine.rs:13-19`). DLIS needs `dlisio` (`dlis.rs:133-134`), spreadsheet plate extraction
needs `openpyxl` (`images.rs:855-857`), and Office deliverables need `xlsxwriter`, `python-docx`,
`python-pptx` and `matplotlib` (`office.rs:310-335,881-901,1426-1447`). **Obligation:** the release
must publish and probe a per-capability dependency matrix; a blanket “no runtime dependencies” claim
is false for those capabilities.

### 2.7 Reproducibility requires the exact runtime, not “Python is installed”

**Finding INS-7 (source audit).** Runtime discovery checks an environment override, per-user Python
3.13 through 3.10 locations, then `python3` and `python`, and selects the first interpreter able to
import NumPy (`python_engine.rs:176-209`). The Office probe reports the selected path and five package
booleans (`office.rs:48-109`), while the equation probe reports only the path and optional SciPy
version (`python_engine.rs:212-255`). **Obligation:** every optional capability must use one resolved
interpreter and report the exact executable, package versions and resolution reason.

### 2.8 Release packaging exists, managed deployment does not yet

**Finding INS-8 (source audit).** Tauri bundling is active and targets all configured formats
(`tauri.conf.json:27-36`); the embedded database is compiled with its bundled feature
(`Cargo.toml:20-28`). The repository contains no product-specific installer policy for silent
deployment, offline optional runtimes, upgrade/rollback, per-machine versus per-user choice, or
uninstall data preservation. **Obligation:** a produced bundle is not yet a managed-estate product;
those behaviours need explicit release gates.

### 2.9 A generated licence inventory is evidence, not legal sign-off

**Finding INS-9 (source audit).** `THIRD-PARTY-LICENSES.md:1-17` is generated from distributed Rust
and JavaScript dependency graphs and explicitly excludes Python packages because the product does
not distribute them. It calls itself factual inventory rather than legal advice. **Obligation:** keep
the generated inventory reproducible, add every actually bundled optional component to its scope,
and retain a separate human legal-approval gate.

## 3. SandiBumi as-built

| Capability | Status | Direct source finding |
|---|---|---|
| Release bundle enabled | `PARTIAL` | `tauri.conf.json:27-36` enables bundling and `targets: "all"`; no product-specific managed-deployment policy is present. |
| Stable application identity and aligned version | `PRESENT-OK` | Product version `0.1.0` and identifier are declared in `tauri.conf.json:3-5`; the Rust and JavaScript package versions are also `0.1.0` (`Cargo.toml:1-6`; `package.json:1-5`). |
| Native core launch without Python | `PRESENT-OK` | Python is a subprocess with no link-time dependency; absence cannot prevent launch (`python_engine.rs:5-11`). DuckDB is bundled (`Cargo.toml:27`). |
| Accurate “no external runtime” product claim | `PRESENT-DIVERGENT` | `README.md:6` claims no external runtime dependency, but the capabilities and packages in Finding INS-6 require one. The divergence is capability-level, not a numerical delta. |
| One Python resolution algorithm | `PRESENT-OK` | `find_python()` is shared and cached (`python_engine.rs:176-203`). |
| Equation runtime preflight | `PARTIAL` | Path and optional SciPy version are exposed (`python_engine.rs:212-255`); NumPy version and resolution reason are not. |
| Office runtime preflight | `PARTIAL` | One probe returns the interpreter and five availability booleans (`office.rs:48-109`); package versions are absent. |
| DLIS and plate-extraction preflight | `PARTIAL` | Both fail locally with actionable package messages (`dlis.rs:133-134`; `images.rs:855-857`), but neither is represented in one release-wide matrix. |
| Offline optional-runtime deployment | `ABSENT` | No bundled Python distribution, wheelhouse or documented offline dependency pack exists in the packaging configuration. |
| Shipped-template / per-user-settings split | `ABSENT` | No product configuration deployment or migration manifest equivalent to the T1 pattern was found. |
| Corporate policy and precedence | `ABSENT` | No product corporate policy layer or precedence contract was found. |
| Versioned external parameter registry | `ABSENT` | Curve families and aliases are code-resident (`curves.rs:19-37`), not a signed/versioned external registry. |
| Typed unit registry | `PARTIAL` | Curve families carry canonical unit strings (`curves.rs:9-37`); conversion matches string pairs (`curves.rs:52-94`) and lowercases tokens (`curves.rs:96-103`). Only depth has a dedicated unit type (`units.rs:20-69`). |
| Upgrade, rollback and uninstall preservation policy | `ABSENT` | No updater configuration or product-specific installer script establishes these behaviours. |
| Generated third-party inventory | `PRESENT-OK` | The inventory states its generated scope and exclusions (`THIRD-PARTY-LICENSES.md:1-17`); the generator writes it from dependency graphs (`tools/gen-third-party-licenses.mjs:1,140-146`). |
| Clean-machine installer qualification | `ABSENT` | No clean-machine, locked-down-user, offline or upgrade installer test is present. |

## 4. Requirements

#### SB-INS-001 — Ship a qualified native Windows installer [P0] [status: PARTIAL]

**Requirement.** Every release MUST produce an installer with the declared application identity and
version, install for a non-developer user without Rust or Node.js, and record its package type,
digest and build provenance in the release manifest.

**Rationale.** Bundling is active, but “targets all” does not prove a usable managed-estate path.

#### SB-INS-002 — Keep native core launch independent of Python [P0] [status: PRESENT-OK]

**Requirement.** Missing, incompatible or misconfigured Python MUST NOT prevent application launch,
project open, native petrophysics, plotting or native export. Only the capability that needs Python
may be unavailable.

**Rationale.** This preserves the source contract at `python_engine.rs:5-11`.

#### SB-INS-003 — Publish truthful capability-level prerequisites [P0] [status: PRESENT-DIVERGENT]

**Requirement.** Product copy, installer UI, release notes and in-app support MUST be generated from
one capability manifest and MUST NOT claim runtime independence for a capability that invokes an
external interpreter or package.

**Rationale.** Finding INS-6 and `README.md:6` presently disagree.

#### SB-INS-004 — Maintain one dependency/capability manifest [P0] [status: ABSENT]

**Requirement.** A machine-readable manifest MUST map each optional capability to its interpreter,
packages, minimum supported versions, offline availability and owning domain. Detection and user
messages MUST consume this manifest rather than duplicate package lists.

**Rationale.** Current package knowledge is split across four Rust modules.

#### SB-INS-005 — Resolve one interpreter with explainable precedence [P0] [status: PARTIAL]

**Requirement.** All Python-backed capabilities MUST use one session-resolved interpreter. Support
UI MUST show the exact executable, which precedence rule selected it, and why higher-priority
candidates were rejected.

**Rationale.** The path is shared today, but resolution provenance is not surfaced.

#### SB-INS-006 — Probe packages and versions before work begins [status: PARTIAL]

**Requirement.** Capability UI MUST probe required packages and versions before opening a costly or
destructive workflow, state the exact missing/incompatible item, and never defer a known dependency
failure until save or batch execution.

**Rationale.** Office and equation probes establish the pattern; DLIS and plate extraction remain
outside a unified preflight.

#### SB-INS-007 — Give interpreter-specific remediation [status: PARTIAL]

**Requirement.** Every missing-package message MUST name the selected interpreter and provide a
copyable installation command targeted to that executable. It MUST also provide a re-probe action.

**Rationale.** Existing messages name packages and the environment override, but some omit the
selected executable.

#### SB-INS-008 — Support offline and managed deployment [P0] [status: ABSENT]

**Requirement.** The release MUST define one supported offline deployment route for every claimed
capability: bundled dependency, signed offline dependency pack, or explicit exclusion from the
offline feature set. Silent install and detection MUST require no public network access.

**Rationale.** Managed estates cannot rely on an interactive package download.

#### SB-INS-009 — Pin and attest optional runtime contents [status: ABSENT]

**Requirement.** Any runtime or package distributed by SandiBumi MUST have an exact version, digest,
licence disposition and vulnerability-review status in the release manifest. User-supplied
interpreters MUST be labelled external and their observed versions recorded.

**Rationale.** The current licence inventory correctly excludes packages that are not distributed;
that scope must change if deployment changes.

#### SB-INS-010 — Separate immutable templates from user configuration [P0] [status: ABSENT]

**Requirement.** Installed defaults MUST remain immutable. First run MUST materialise writable user
configuration outside the installation directory, record the originating template version and
digest, and never treat a later-mutated user file as evidence of a factory default.

**Rationale.** Finding INS-1 demonstrates why installed and live copies cannot be conflated.

#### SB-INS-011 — Migrate configuration explicitly and reversibly [status: ABSENT]

**Requirement.** Upgrade MUST inventory eligible prior-version settings, present the migration set,
preserve the pre-migration copy, migrate through a versioned transformation, and report every
accepted, renamed, defaulted or rejected entry.

**Rationale.** A list of files to migrate is safer than copying an entire mutable profile blindly.

#### SB-INS-012 — Provide a corporate policy layer [status: ABSENT]

**Requirement.** Administrators MUST be able to provide a read-only, signed policy layer for
approved runtimes, configuration packs, data locations and disabled capabilities without editing
the installed application or a user's working copy.

**Rationale.** The T1 installation separates corporate folders from ordinary user settings.

#### SB-INS-013 — Make precedence visible and deterministic [status: ABSENT]

**Requirement.** Effective configuration MUST resolve in documented order across shipped template,
corporate policy, migrated user settings and current user settings. For every effective value,
support UI MUST name the winning layer and any shadowed values.

**Rationale.** Hidden precedence makes two nominally identical machines irreproducible.

#### SB-INS-014 — Key parameters by semantic identifier and ordinal [P0] [status: ABSENT]

**Requirement.** Every parameter-pack row MUST carry a stable semantic identifier, module schema
version and ordinal. Names are display labels only. Duplicate labels are permitted only when the
identifier and ordinal remain unique.

**Rationale.** Finding INS-2 shows that name-only joins fail on duplicates and drift.

#### SB-INS-015 — Refuse registry mismatch and ambiguity [P0] [status: ABSENT]

**Requirement.** Loading MUST stop before use when an identifier and ordinal disagree, an ordinal is
missing, two rows claim the same key, an empty key is mapped, or the schema version is unsupported.
The refusal MUST name the file and conflicting rows.

**Rationale.** Guessing a positional join silently assigns a valid value to the wrong parameter.

#### SB-INS-016 — Use a canonical typed unit registry [P0] [status: PARTIAL]

**Requirement.** Every unit token MUST resolve to a quantity kind and canonical internal unit before
conversion. A conversion between different quantity kinds MUST be refused even when both strings
are individually recognised.

**Rationale.** Finding INS-3 includes a demonstrated permeability-to-length mapping.

#### SB-INS-017 — Preserve observed unit and encoding tokens [status: PARTIAL]

**Requirement.** Ingestion MUST retain the raw unit token, decoded text encoding and canonical
interpretation. Case folding or punctuation normalisation MAY occur only after the raw token is
recorded and only under an explicit alias rule.

**Rationale.** Finding INS-4 makes vocabulary drift observable rather than silently erasing it.

#### SB-INS-018 — Reject missing and empty unit mappings [status: PARTIAL]

**Requirement.** Absent unit elements, empty elements, placeholder symbols and empty-to-empty mapping
rows MUST enter one explicit missing-unit state; they MUST NOT create a successful mapping.

**Rationale.** The dossier found three missing-unit encodings and six blank mapping rows.

#### SB-INS-019 — Generate aliases, families and units from one registry [status: ABSENT]

**Requirement.** Runtime lookup, import UI, documentation and tests MUST be generated from one
versioned canonical registry. Release validation MUST fail when generated populations or declared
dimensions disagree.

**Rationale.** Five inconsistent incumbent vocabularies and the code-resident SandiBumi table show
the cost of parallel sources of truth.

#### SB-INS-020 — Version and attest configuration packs [status: ABSENT]

**Requirement.** Every shipped, corporate or user parameter pack MUST carry schema version, content
digest, source class, creation time and compatibility range. A changed digest MUST create a new
provenance event before computation.

**Rationale.** A mutable live file changed during evidence collection; file name alone is not an
identity.

#### SB-INS-021 — Produce a reproducible support report [status: PARTIAL]

**Requirement.** A one-action support report MUST include application version and build digest,
installer type, OS architecture, active configuration layers and digests, selected interpreter and
resolution rule, package versions, optional capability matrix, and redacted failure diagnostics.
It MUST exclude project data and user secrets by default.

**Rationale.** Current probes expose useful fragments but not one reproducible environment record.

#### SB-INS-022 — Preserve user data through upgrade and uninstall [status: ABSENT]

**Requirement.** Upgrade, rollback and uninstall MUST leave project files and user-authored
configuration intact by default. Removing either requires a separate, explicit, enumerated consent
step and a recoverable backup path.

**Rationale.** Installer lifecycle actions are destructive unless their data boundary is explicit.

#### SB-INS-023 — Gate releases on clean-machine scenarios [P0] [status: ABSENT]

**Requirement.** A release MUST pass clean-machine tests for standard user, locked-down user, offline
install, no-Python core use, supported external Python, missing package, upgrade, rollback and
uninstall preservation before the installer is publishable.

**Rationale.** Developer-machine success does not verify a customer installer.

#### SB-INS-024 — Generate and review third-party obligations [status: PRESENT-OK]

**Requirement.** Every release MUST regenerate the distributed-dependency inventory, fail on a
missing licence declaration, separately enumerate bundled optional runtimes, and record human legal
approval. Generated inventory MUST never be represented as legal advice.

**Rationale.** This retains the current inventory's correct scope distinction.

#### SB-INS-025 — Enforce the evidence-acquisition firewall [status: ABSENT]

**Requirement.** Installation and migration tooling MUST NOT parse proprietary key files, compiled
libraries, opaque model weights or vendor chart payloads to recover methods or defaults. It MAY
inventory such an artefact's presence and route the user need to the owning method chapter for
independent derivation.

**Rationale.** Finding INS-5 and `CONTRACT.md:146-182` separate public parameter surfaces from
prohibited reconstruction.

#### SB-INS-026 — Keep release claims derived from executable evidence [status: ABSENT]

**Requirement.** The installer feature list, documentation prerequisite table and in-app capability
matrix MUST be generated from the same manifest and tested probes. A release MUST fail if any public
claim says a capability is available while its declared runtime path is unavailable.

**Rationale.** This closes the present source-versus-product-copy divergence.

## 5. Parameters

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---:|---|---|---|
| Product version | — | `0.1.0` | semantic version | `tauri.conf.json:4`; `Cargo.toml:3`; `package.json:4` | T1 |
| Application identifier | — | `com.sandibumi.petro` | identifier | `tauri.conf.json:5` | T1 |
| Bundle active | — | `true` | boolean | `tauri.conf.json:27-29` | T1 |
| Bundle targets | — | `all` | Tauri target selector | `tauri.conf.json:27-29` | T1 |
| Embedded database mode | — | `bundled` | Cargo feature | `Cargo.toml:27` | T1 |
| Minimum Python version named by runtime errors | — | `3.10+` | Python version | `python_engine.rs:47-48`; `dlis.rs:133-134` | T1 |
| Current interpreter override | — | `SANDIBUMI_PYTHON` | environment variable | `python_engine.rs:39-48` | T1 |
| Per-user discovery versions | — | `3.13`, `3.12`, `3.11`, `3.10` | Python minor version | `python_engine.rs:193-199` | T1 |
| Equation-engine required package | — | `numpy` | Python package | `python_engine.rs:17-18,205-209` | T1 |
| Equation-engine optional package | — | `scipy` | Python package | `python_engine.rs:13-19,226-242` | T1 |
| DLIS package | — | `dlisio` | Python package | `dlis.rs:24-28,133-134` | T1 |
| Spreadsheet-plate packages | — | `openpyxl`, `Pillow` | Python packages | `images.rs:490-493,855-857` | T1 |
| Office packages | — | `xlsxwriter`, `python-docx`, `python-pptx`, `matplotlib` | Python packages | `office.rs:48-68,310-335,881-901,1426-1447` | T1 |
| Corporate/user/template precedence | — | `ABSENT — ships with no default` | precedence order | no source-backed product policy | — |
| Offline runtime distribution mode | — | `ABSENT — ships with no default` | deployment mode | no source-backed product policy | — |
| Supported installer package type | — | `ABSENT — ships with no default` | installer type | `targets: "all"` does not select a qualified release format | — |
| Configuration-pack text encoding | — | `ABSENT — ships with no default` | encoding | dossier §2.1 proves a CP1252 case but does not license a product default | — |
| Unit-token case policy | — | `ABSENT — ships with no default` | matching policy | dossier §2.3 proves drift; no adjudicated product policy | — |

## 6. Acceptance tests

| Test | Input | Operation | Expected value | Source of expected value |
|---|---|---|---|---|
| `SB-INS-T01` | Signed release installer on a clean supported Windows image with no developer tools | Install as standard user, then launch | Install and launch succeed; manifest reports the same version and identifier as the installed executable | `SB-INS-001`; `tauri.conf.json:3-5,27-36` |
| `SB-INS-T02` | Machine with no Python executable | Launch, open an existing project, render a log and run a native method | All four native operations succeed; Python-backed capabilities alone show unavailable | `SB-INS-002`; `python_engine.rs:5-11` |
| `SB-INS-T03` | Same no-Python machine | Open prerequisite/help surfaces | Every Python-backed capability is named unavailable; no page claims all capabilities have no runtime dependency | `SB-INS-003`, `SB-INS-026`; Finding INS-6 |
| `SB-INS-T04` | Capability manifest fixture containing equation, DLIS, plate, workbook, document and deck features | Validate manifest | Each feature maps to the exact packages listed in §5; no source module carries a second package list | `SB-INS-004`; §5 package rows |
| `SB-INS-T05` | Two interpreters: higher-precedence lacks NumPy; lower-precedence imports NumPy | Run discovery | Lower candidate is selected; report names both candidates and the rejection reason | `SB-INS-005`; `python_engine.rs:176-209` |
| `SB-INS-T06` | Valid override pointing to a NumPy-capable interpreter | Run all Python-backed probes | Every probe reports that exact executable | `SB-INS-005`; `python_engine.rs:39-41,176-203`; `office.rs:91-109` |
| `SB-INS-T07` | Selected interpreter with NumPy but no SciPy | Open equation editor and use a SciPy-only function | Editor is available; SciPy feature is unavailable before run and remediation names that interpreter | `SB-INS-006`, `SB-INS-007`; `python_engine.rs:13-19,212-255` |
| `SB-INS-T08` | Selected interpreter without `dlisio` | Open DLIS import | Preflight refuses before file parsing, names `dlisio`, the executable and a re-probe action | `SB-INS-006`, `SB-INS-007`; `dlis.rs:24-28,133-134` |
| `SB-INS-T09` | Selected interpreter missing each Office package in turn | Open corresponding deliverable action | Only the affected action is disabled; message names the exact missing package and executable | `SB-INS-006`, `SB-INS-007`; `office.rs:48-109,310-335,881-901,1426-1447` |
| `SB-INS-T10` | Offline clean machine plus release media | Silent-install every capability claimed in the offline feature set | No network request occurs; every claimed capability passes its probe | `SB-INS-008`; release capability manifest |
| `SB-INS-T11` | Release containing a bundled optional runtime | Verify release manifest and licence inventory | Exact runtime/package versions and digests appear in both; legal-review status is present | `SB-INS-009`, `SB-INS-024`; `THIRD-PARTY-LICENSES.md:7-17` |
| `SB-INS-T12` | Installed template, first launch, then user edit | Compare installation directory and user copy | Installed template digest is unchanged; user copy records template version and digest | `SB-INS-010`; dossier §2.6 |
| `SB-INS-T13` | Prior-version user settings containing supported, renamed and invalid entries | Upgrade | Pre-upgrade copy remains; report classifies every input entry as accepted, renamed, defaulted or rejected | `SB-INS-011`; dossier §2.6 |
| `SB-INS-T14` | Signed corporate policy fixing an approved interpreter and disabling one capability | Launch as ordinary user and attempt overrides | Approved interpreter wins; disabled capability remains disabled; provenance names corporate policy | `SB-INS-012`, `SB-INS-013`; dossier §2.6 |
| `SB-INS-T15` | Four layers assigning different values to one setting | Resolve effective configuration | One deterministic winner matches documented precedence; report lists all shadowed values | `SB-INS-013`; `SB-INS-T14` fixture |
| `SB-INS-T16` | Pack with two identical display labels but unique identifiers/ordinals | Load pack | Both rows load and remain separately addressable | `SB-INS-014`; dossier §2.4 seven-label collision |
| `SB-INS-T17` | Pack whose semantic identifier says one parameter and ordinal points to another | Load pack | Load fails before computation and names both rows | `SB-INS-015`; dossier §3.8 |
| `SB-INS-T18` | Pack with missing ordinal, duplicate key, unsupported schema and empty key as four fixtures | Load each | All four fail; none guesses or partially activates | `SB-INS-015`; dossier §§2.4, 2.13 |
| `SB-INS-T19` | Unit bridge mapping permeability token `md` to length token `m` | Validate registry | Validation fails with quantity-kind mismatch; no numeric conversion is produced | `SB-INS-016`; dossier §§2.13, 2.16.1, N-NEW-5 |
| `SB-INS-T20` | Recognised length-to-length and slowness-to-slowness mappings | Validate and convert known samples | Mappings pass only within their quantity kind; results match each mapping's cited exact factor | `SB-INS-016`; `curves.rs:64-80` |
| `SB-INS-T21` | Unit tokens `mV` and `mv` under a registry with no equivalence declaration | Ingest | Raw tokens remain distinct and a drift warning is recorded | `SB-INS-017`; dossier §2.3, N-NEW-17 |
| `SB-INS-T22` | CP1252 configuration containing byte `0x92` and a declared CP1252 encoding | Load and export provenance | Load succeeds; encoding and original byte representation are recorded | `SB-INS-017`; dossier §§2.1, 3.9 N-NEW-12 |
| `SB-INS-T23` | Missing unit encoded as absent element, empty element, placeholder symbol and empty-to-empty row | Load four fixtures | All resolve to the same explicit missing-unit state; zero mappings are registered | `SB-INS-018`; dossier §2.3, N-NEW-23/N-NEW-28 |
| `SB-INS-T24` | Canonical registry fixture with generated runtime, UI and documentation artifacts | Regenerate and compare populations | All artifacts carry the same registry version and equal family/unit populations | `SB-INS-019`; dossier N-NEW-7/N-NEW-10 |
| `SB-INS-T25` | Signed pack, then one-byte mutation without version change | Load twice | Original loads; mutated copy is refused or creates a new explicit provenance event, never reuses identity | `SB-INS-020`; dossier §2.6 digest evidence |
| `SB-INS-T26` | Machine with two candidate interpreters and mixed package versions | Export support report | Report contains every field in `SB-INS-021`, exact executable and versions; contains no project samples or secrets | `SB-INS-021`; Findings INS-6/INS-7 |
| `SB-INS-T27` | Installation with user settings and two project files | Upgrade, rollback, then uninstall with default choices | User settings and both project files remain byte-identical after each action | `SB-INS-022`; lifecycle data-boundary requirement |
| `SB-INS-T28` | Clean-machine matrix named in `SB-INS-023` with one deliberately failing scenario | Run release gate | Gate is red and installer is not publishable; report names the failing scenario | `SB-INS-023`; §2.8 |
| `SB-INS-T29` | Dependency graph containing one package with no declared licence | Generate inventory | Generation/release gate fails and names the package; no “approved” statement is inferred | `SB-INS-024`; `THIRD-PARTY-LICENSES.md:7-12,33` |
| `SB-INS-T30` | Opaque compiled library, weight file and proprietary chart payload placed beside an importable pack | Inventory and migrate | Presence may be logged; contents are not parsed and no method/default is emitted | `SB-INS-025`; `CONTRACT.md:153-177`; dossier §§2.15, 7.3 |

## 7. Open items, escalations and refusals

### 7.1 Open items

- **O-INS-1 — qualified installer format and install scope.** `targets: "all"` is build
  configuration, not a product decision. Select and source the supported package type and per-user or
  per-machine scope before `SB-INS-001` can be fully parameterised.
- **O-INS-2 — offline Python strategy.** Choose bundled runtime, signed dependency pack, or reduced
  offline feature set. Until decided, §5 correctly ships no default.
- **O-INS-3 — configuration precedence.** The required layers are known; their order is deliberately
  absent until security and operations review it.
- **O-INS-4 — supported Windows matrix.** Developer prerequisites name Windows 10/11
  (`CONTRIBUTING.md:8-16`), but no released support matrix or lifecycle is cited.
- **O-INS-5 — case and encoding policy.** The dossier proves both hazards but does not justify one
  universal encoding or case rule. The registry must declare them per format.
- **O-INS-6 — legal approval.** Generated licence facts exist; human approval of the shipped bundle
  remains a release gate.

### 7.2 Escalations

- **E-INS-1 — deployment-owner decision:** resolve O-INS-1 through O-INS-3 together because package
  scope, offline runtime ownership and corporate precedence interact.
- **E-INS-2 — security review:** define signing, policy-pack trust and support-report redaction before
  `SB-INS-012`, `SB-INS-020` and `SB-INS-021` are implementation-ready.
- **E-INS-3 — legal review:** re-evaluate inventory scope if any Python runtime or package becomes
  distributed rather than user supplied.
- **E-INS-4 — method-domain acquisition:** the dossier's remaining scientific gaps E1-E8 and E10-E15
  are routed in §8; installation work must not fill them by adopting a neighbouring value.

### 7.3 Refusals

- **Dimension-changing bridges are refused.** SandiBumi validates quantity kinds and rejects the
  demonstrated permeability-to-length mapping instead of preserving incumbent compatibility.
- **Blank mapping success is refused.** Empty keys and the three missing-unit spellings become an
  explicit missing-unit state; they never register a conversion.
- **Name-only positional joins are refused.** Duplicate or drifting display labels cannot select a
  parameter; semantic identifier and ordinal must agree.
- **Mutable-user-file defaults are refused.** A live profile value is evidence about that profile,
  not a factory default. Shipped defaults come only from an immutable, attested template.
- **Binary-derived methods and defaults are refused.** Compiled libraries, opaque weights,
  proprietary key files and chart payloads are inventoried at most; they are never decoded or
  behaviourally reconstructed.
- **False dependency-free claims are refused.** Availability is stated per capability from the
  executable manifest and probe results.

### 7.4 Independent-derivation requirements

No Tier-C item falls in this domain.

## 8. Traceability — dossier disposition

### 8.1 Dossier targets, blockers and findings

| Dossier item | Disposition in this chapter |
|---|---|
| `T1` Monte Carlo defaults | Scientific values routed to `CUT`; encoding and registry implications adopted in INS-4, `SB-INS-014`/`-017`; no values transcribed here. |
| `T2` mineral defaults | Scientific values routed to `MIN`, `NMR` and `SAT`; provenance/units implications adopted in `SB-INS-016`/`-020`. |
| `T3` curve parameter vocabulary | Adopted architecturally in INS-3/INS-4 and `SB-INS-016` through `-019`. |
| `T4` set dictionary | Accounted as absent; no requirement depends on a guessed replacement. |
| `T5` default aliases | Adopted as the canonical-registry obligation `SB-INS-019`; corrected two-table reading retained. |
| `T6` unit conversions | Routed to `DIO` for numerical conversion; registry consistency owned by `SB-INS-016`/`-019`. |
| `T7` application/user config split | Adopted in INS-1 and `SB-INS-010` through `-013`, `-020`. |
| `T8` NMR defaults file | Absence routed to `NMR`; no values promoted here. |
| `T9` overlay registry | Presence accounted; plotting schema routed to `PLI`; payload is not transcribed. |
| `A1`, `A2`, `A9`, `A10` geomechanics files/gaps | Routed to `GEO`; opaque table values remain outside this chapter. `A10` also supports `SB-INS-025`. |
| `A3` fluid-substitution defaults | Routed to `FSR`; no scientific value transcribed. |
| `A4` mineral-equation weights | Routed to `MIN`; distinct-from-Monte-Carlo warning retained through `SB-INS-020`. |
| `A5` capillary-pressure defaults | Routed to `SAT`; no scientific value transcribed. |
| `A6` help registries | INS-2 and `SB-INS-014`/`-015`; all three files and the partial ordinal closure accounted. |
| `A7` bridge mapping | Namespace defect routed to `MIN`; identifier/ordinal contract adopted in `SB-INS-014`/`-015`. |
| `A8` unit mapping/config | INS-3 and `SB-INS-016` through `-019`. |
| Blocker `5` | Still-open mineral-sigma endpoint routed to `NMR`/`MIN`; absent here. |
| Blocker `6` | Closed scientific widths routed to `CUT`; encoding and parameter-pack provenance retained. |
| Blocker `13` | Closed Qv/CEC unit distinction routed to `SAT`/`MIN`; typed-unit obligation retained. |
| Blocker `18` | Corrected to 2 of 4 closed and routed to `GEO`; no compiled coefficient recovery allowed by `SB-INS-025`. |
| Blocker `19` | Three of four files plus attribute lists closed; configuration architecture adopted, absent file not invented. |
| Blocker `20` | Partial closure adopted as INS-2; positional inference is not treated as vendor assertion. |
| `N-6.3` | Resolved and adopted as INS-1. |
| `O-OPEN-8` | Closed but downgraded to mutable-profile evidence; supports `SB-INS-020`, not a shipped default. |
| `I-OPEN-1`, `I-OPEN-2`, `I-OPEN-3`, `R-11` | Routed to `FSR`; no values transcribed. |
| `H-D-4`, `H-D-5` | Confirmed open and routed to `NMR`; values remain absent. |
| Overlay presence | Accounted under `T9`; schema routed to `PLI`. |
| `N-NEW-1` | Routed to `NMR`/`MIN`; competing sigma libraries must not be mixed. |
| `N-NEW-2` | Routed to `GEO`; discontinuity not imported here. |
| `N-NEW-3` | Routed to `DIO`; typed-unit registry obligation retained. |
| `N-NEW-4` | Routed to `SAT`; incorrect unit label refused by `SB-INS-016`. |
| `N-NEW-5` | Adopted as INS-3, `SB-INS-016` and refusal §7.3. |
| `N-NEW-6` | Withdrawn by the corrected dossier; no requirement generated. |
| `N-NEW-7` | Adopted in INS-3 and `SB-INS-019`. |
| `N-NEW-8` | Stale-header defect supports manifest-generated claims, `SB-INS-026`. |
| `N-NEW-9` | Routed to `NMR`; no bin geometry transcribed. |
| `N-NEW-10` | Adopted in INS-3 and `SB-INS-016`/`-019`. |
| `N-NEW-11` | Routed to `MIN`; label/value disagreement must not become a default. |
| `N-NEW-12` | Adopted in INS-4 and `SB-INS-017`; test `SB-INS-T22`. |
| `N-NEW-13` | Routed to `CUT`/`MIN`; false corroboration rejected. |
| `N-NEW-14` | Routed to `MIN`; also supports identifier/ordinal validation. |
| `N-NEW-15` | Routed to `CUT`; no global percentile polarity adopted. |
| `N-NEW-16` | Adopted in `SB-INS-019`; denominator correction retained. |
| `N-NEW-17` | Adopted in INS-4 and `SB-INS-017`; test `SB-INS-T21`. |
| `N-NEW-18` | Adopted in INS-2 and `SB-INS-014`/`-015`. |
| `N-NEW-19` | Compliance scope defect adopted in `SB-INS-025`; values not transcribed. |
| `N-NEW-20` | Public-registry versus compiled-implementation boundary adopted in INS-5/`SB-INS-025`; method values routed. |
| `N-NEW-21` | Routed to `MIN`; namespace validation adopted in `SB-INS-015`. |
| `N-NEW-22` | Routed to `MIN`; proven copy-down defect is refused, not copied. |
| `N-NEW-23` | Adopted in `SB-INS-018`; test `SB-INS-T23`. |
| `N-NEW-24` | Adopted in INS-1 and `SB-INS-010`/`-020`. |
| `N-NEW-25` | Routed to `PLI`; incomplete metadata cannot support an exact census claim. |
| `N-NEW-26` | Positive typed-unit counter-example adopted in `SB-INS-016`; scientific values routed to `MIN`. |
| `N-NEW-27` | Routed to `DIO`/`MIN`; dimension-safe conversion obligation retained. |
| `N-NEW-28` | Adopted in `SB-INS-018`; test `SB-INS-T23`. |
| `N-NEW-29` | Corrected block count routed to `CUT`; stale counts support `SB-INS-020`. |
| `N-NEW-30` | Measurement artefact accounted; robust parser/test-fixture counting belongs to pack validation. |

### 8.2 Adoption, refusals, clamps, regressions and escalations

| Dossier section/items | Disposition |
|---|---|
| §5.1 Monte Carlo priors/reporting/width map | Routed to `CUT`; no scientific parameter adopted by INS. |
| §5.1 mineral weights, mineral endpoints, fracture-gradient, fluid substitution, saturation-height and Qv/CEC rows | Routed respectively to `MIN`, `GEO`, `FSR`, `SAT`; every parameter remains subject to its owning chapter. |
| §5.1 unit and alias architecture | Adopted in `SB-INS-014` through `-020`. |
| §5.2 all eleven “do not adopt” items | Preserved: scientific defects route to owning domains; empty mappings, name-only ordinals, mutable-config defaults and compiled internals are explicit §7.3 refusals. |
| §5.3 all six clamp/validity items | Routed to their scientific domains; none is an installation parameter. |
| `R1`–`R15`, `R17`, `R18` | Routed to the owning scientific/data chapters; no method test is duplicated here. |
| `R16` | Routed to `CUT`; corrected width-to-sigma interpretation retained only as traceability. |
| `E1`–`E8` | Live-session scientific acquisitions routed to `NMR`, `MIN`, `GEO` and `FSR`; INS neither guesses nor reconstructs. |
| `E9` | Corrected/rescoped Monte Carlo question routed to `CUT`; no shipped asymmetric row requires an INS action. |
| `E10` | External scientific validation routed to `GEO`/`MIN`. |
| `E11` | Remaining adjacent-tool evidence acquisition routed to owning scientific chapters. |
| `E12` | Unit declaration gap retained as provenance and typed-unit obligation; scientific K/Th/U use routed to `MIN`. |
| `E13` | Compiled coefficient gap routed to `GEO`; `SB-INS-025` forbids decompilation. |
| `E14` | Unswept plain-text registry surface is an evidence task, not an installation default; route by module. |
| `E15` | Unit-system evidence acquisition supports `DIO`/INS registry design; no vendor registry is copied wholesale. |

### 8.3 Critique disposition

| Critique item | Chapter disposition |
|---|---|
| `BL-1` | Corrected width-to-sigma interpretation accepted and routed to `CUT`; the old 2× error is not repeated. |
| `BL-2` | K/Th/U unit overclaim removed; typed-unit obligation retained, scientific values routed. |
| `BL-3` | Cutoff registry is explicitly partial; no nonexistent ordinal tags are claimed. |
| `BL-4` | Corrected blocker-18 accounting retained and routed to `GEO`. |
| `BL-5` | Withdrawn alias “correction” stays withdrawn; two-table model retained. |
| `MJ-1` | Per-parameter percentile polarity routed to `CUT`; no global switch specified. |
| `MJ-2`, `MJ-3` | Wet/dry explanation rejected and 5.66× corrected magnitude retained; routed to `NMR`/`MIN`. |
| `MJ-4` | Solver weights and Monte Carlo widths remain distinct; false corroboration not used. |
| `MJ-5`, `MJ-6` | Corrected 76/408 denominators and full missing-unit bucket adopted in INS-3. |
| `MJ-7` | Opaque overburden file family accounted and protected by `SB-INS-025`. |
| `MJ-8` | Broken bridge rows routed to `MIN`; namespace validation adopted. |
| `MJ-9` | Adjacent-tool evidence now represented; remaining gap remains an escalation. |
| `MJ-10` | No “complete/verbatim” claim is made for truncated source extracts. |
| `MJ-11` | Provable mineral-row defect is routed and refused, not adopted. |
| Minor `1`–`9` | All corrections are inherited from the revised dossier: counts, headings, paths, encoding, scope language and source-register qualifications are not reverted here. |

### 8.4 Requirement coverage

| Requirement ids | Evidence and verification |
|---|---|
| `SB-INS-001`–`-003` | Findings INS-6/INS-8; tests `T01`–`T03`. |
| `SB-INS-004`–`-009` | Findings INS-6/INS-7/INS-9; tests `T04`–`T11`. |
| `SB-INS-010`–`-013` | Finding INS-1; tests `T12`–`T15`. |
| `SB-INS-014`–`-015` | Finding INS-2; tests `T16`–`T18`. |
| `SB-INS-016`–`-019` | Findings INS-3/INS-4; tests `T19`–`T24`. |
| `SB-INS-020`–`-021` | Mutable-file and runtime-provenance evidence; tests `T25`–`T26`. |
| `SB-INS-022`–`-024` | Installer lifecycle and licence evidence; tests `T27`–`T29`. |
| `SB-INS-025`–`-026` | Finding INS-5 and dependency-claim divergence; tests `T30`, `T03`–`T04`. |
