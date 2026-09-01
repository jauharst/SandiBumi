# Gate 3 program — Windows/offline deployment and recovery

Opened 2026-09-02, after Gate 2 closed formally (DEC-083, 2026-08-20) and its closure was
re-verified physically on the tree (`AUDIT-GATE2-2026-09-02.md`, at the repository root). This is the program document
for the third gate on the takeover ladder: what it owns, what already exists, what evidence closes
it, and the order the work is proposed in. The dashboard row lives in `STATUS.md`; requirement
evidence stays in `requirements.csv`; the human test steps are Section **INS** of
`docs/manual_test_plan.md`.

**Exit evidence.** The dashboard states it as *a clean-machine, offline-runtime, rollback and
recovery matrix*; the adjudication's next action for SB-INS-023 (`evidence/sb-ins.md`) says what
that matrix must be bound to, and this program adopts both: run on the exact release candidate,
every cell green, and every result bound to the MSI digest, the Python-pack digest and the commit
that built them.

## 1. Scope — the 18 rows Gate 2 handed forward

`gate2-program.json` lists 20 `later_gate_only` rows. Eighteen name `G3` as their owner; the other
two (`SB-CORE-040`, `SB-PLT-032`) are Gate 4's and are not in this program.

| Requirement | Contract (short) | Chapter test | Live class at adjudication | Manual test |
|---|---|---|---|---|
| SB-CORE-041 | The tree builds and tests from a fresh clone | SB-CORE-T13 | MISSING — an existing-worktree green gate is not fresh-clone proof | T-INS-01 |
| SB-CORE-042 | A green gate that a machine enforces | — | MISSING — no owned test; the ledger's expected-value note reads "CHARACTERIZATION only - current repository structure"; needs Jauhar's CI decision | T-INS-14 |
| SB-INS-001 | Ship a qualified native Windows installer | T01 | MISSING — validator proven on a fixture, no real MSI seen | T-INS-01, T-INS-02 |
| SB-INS-002 | Native core launch independent of Python | T02 | CORRECTNESS (`missing_python_does_not_block_project_open_native_computation_plotting_or_native_export`) | T-INS-03 |
| SB-INS-003 | Truthful capability-level prerequisites | T03 | MISSING — generated subset only | T-INS-04 |
| SB-INS-004 | One dependency/capability manifest | T04 | MISSING — six rows, consumers outside them | T-INS-04 |
| SB-INS-005 | One interpreter with explainable precedence | T05, T06 | CORRECTNESS | T-INS-07 |
| SB-INS-006 | Probe packages before work begins | T07–T09 | MISSING — not every action is gated | T-INS-06 |
| SB-INS-007 | Interpreter-specific remediation | T07–T09 | MISSING — no re-probe control | T-INS-06 |
| SB-INS-008 | Offline and managed deployment | T10 | MISSING — schema, no pack, no trace | T-INS-05 |
| SB-INS-009 | Pin and attest the optional runtime | T11 | MISSING — no pack, lock or digests | T-INS-05, T-INS-15 |
| SB-INS-010 | Immutable templates vs user configuration | T12 | CORRECTNESS | T-INS-08 |
| SB-INS-011 | Explicit, reversible configuration migration | T13 | MISSING — ABSENT in the tree | T-INS-09 |
| SB-INS-021 | Reproducible support report | T26 | CHARACTERIZATION — fragments, fields missing | T-INS-13 |
| SB-INS-022 | User data survives upgrade and uninstall | T27 | MISSING — ABSENT in the tree | T-INS-09, T-INS-10, T-INS-11 |
| SB-INS-023 | Releases gated on clean-machine scenarios | T28 | MISSING — validator only (`one_failing_clean_machine_scenario_blocks_release_and_names_the_scenario`) | T-INS-14 |
| SB-INS-024 | Third-party obligations generated and reviewed | T11, T29 | MISSING — succeeds on an unknown licence | T-INS-15 |
| SB-INS-026 | Release claims derived from executable evidence | T03 | MISSING | T-INS-04 |

The chapter is `docs/PRD_v2/27_ip-install-blockers.md` (§6 defines T01–T30); the adjudication
receipt with every row's next action is `docs/takeover/evidence/sb-ins.md`. Six SB-INS rows
(014–019) were closed inside Gate 2 (`gate2-program.json` `completed_requirements`) and are not
reopened here. Four more — SB-INS-012, -013, -020 and -025 — belong to no gate at all: they are
in none of `gate2-program.json`'s three lists, `requirements.csv` marks them DEFERRED and
`sb-ins.md` UNDECIDED. They appear in §2 as the decisions they are.

T-INS-12 (recovery on the installed build) evidences the gate's recovery clause directly; no
`later_gate_only` row owns recovery, so that test closes with the gate rather than with a row.

## 2. What is already decided, and what is not

Decided by the product owner during adjudication (`evidence/sb-ins.md`, "Product-owner
direction and evidence boundary"), and treated here as settled:

- **Installer:** a per-machine MSI, installed by IT in system context, launched by a standard
  user. `tauri.conf.json` already selects MSI and the offline WebView2 install mode.
- **Offline Python:** a separately signed, versioned, application-local SandiBumi-qualified Python
  pack, deployed per machine, which configures `SANDIBUMI_PYTHON` to its own interpreter. Exact
  package versions come only from that release's qualification lock — none are written down
  anywhere until the lock exists.
- **Target matrix:** every Microsoft-serviced Windows 11 x64 Pro and Enterprise feature release
  at release time. The list is captured from Microsoft when the candidate is cut, never
  hard-coded.

Still Jauhar's to decide — Gate 3 can run every scenario that does not depend on them, and the
program says which ones do:

| Decision | Blocks | What is needed |
|---|---|---|
| O-INS-3 configuration precedence (template / corporate policy / migrated / current user) | SB-INS-011 migration report; SB-INS-013 (deferred, in no gate) | an order, signed off by whoever owns deployment and security |
| SB-INS-012 — is a signed corporate policy layer mandatory for the first pilot? | T-INS-14's `locked_down_user` row can run without it; the policy row cannot exist | yes / no for the pilot |
| SB-INS-020 — do configuration packs become pilot-reachable, and under what trust model? | T-INS-15's pack attestation | in scope / out of scope |
| SB-INS-025 — is an explicit migration/inventory route in pilot scope? | nothing in this program; recorded so it is not inferred | in scope / out of scope |
| SB-CORE-042 — CI on every proposed change before the pilot, or a manual release freeze? | T-INS-14's enforcement half | one of the two |
| O-INS-6 — human legal approval of the shipped bundle | release, not testing | a signature, outside engineering |

## 3. What exists in the tree today

- `src-tauri/src/installation.rs`: the capability manifest consumer, the installer/offline/clean-
  machine qualification schemas and their validators, the settings-template materialisation, and
  the nine clean-machine scenarios by name (`standard_user`, `locked_down_user`,
  `offline_install`, `no_python_core_use`, `supported_external_python`, `missing_package`,
  `upgrade`, `rollback`, `uninstall_preservation`). The validators refuse a failed or missing cell
  and name it; they have only ever been fed fixtures.
- Nine correctness proofs among the SB-INS rows (002, 005, 010, 014–019) and the startup refusal of
  an invalid unit registry (`startup_validates_the_typed_unit_registry_and_only_same_kind_bridges_convert`).
  The receipt disagrees with itself on that count — its summary says nine correctness rows, its
  requirement-level total says six; the per-requirement receipts, which the table in §1 follows,
  give nine.
- The prerequisites surface: the **Project ▸ Help ▸ Prerequisites** button, opening the dialog
  titled *Capability prerequisites* (`installationSupportDialog.ts`) reading `installation_support`,
  and the generated `docs/INSTALLATION_PREREQUISITES.md`. That payload — selected interpreter and
  rule, every candidate with its reason, capabilities, package status and versions — IS the
  SB-INS-021 support report as adjudicated; it has no export control (the dialog's only button is
  Close) and lacks application/build digest, installer type, OS architecture and configuration
  digests.
- The sendable diagnostic report: **Project ▸ Monitor ▸ Diagnostics…** (`diagnostics.rs`), with the
  redaction rule and the `crash-log.txt` record from 2026-08-22. It prints the version, the OS and
  architecture, whether an interpreter was found and the scipy version — never the interpreter
  path, which carries a username. It is a different artifact from the SB-INS-021 payload.
- Recovery on the dev build: WAL recovery (`db::init_db_resilient`), autosave and the recovery
  dialog, exercised by T-SHELL-18 and T-PERF-08 of the manual plan.
- The packaged-build checks T-SHIP-01 and T-SHIP-02 (CSP under `npm run tauri build`).

## 4. The matrix

One cell per (feature release × edition × scenario). At the time of writing the serviced Windows
11 feature releases are not enumerated here on purpose (see §2); with two editions and nine
scenarios that is 18 cells per feature release. A cell is green only when the scenario ran on
the exact candidate and its result record names the MSI digest, the pack digest and the commit.

| Scenario id | What it proves | Requirement | Manual test |
|---|---|---|---|
| `standard_user` | per-machine install by IT, launch as a standard user, version and identifier match | SB-INS-001 | T-INS-02 |
| `locked_down_user` | the same launch under a restricted profile (no write to Program Files, no admin) | SB-INS-001, SB-INS-012 (policy half pending) | T-INS-02 step 6 |
| `no_python_core_use` | open, log view, native module, native export with no interpreter present | SB-INS-002, -003, -026 | T-INS-03, T-INS-04 |
| `offline_install` | MSI plus pack with the public network blocked; zero requests; every claimed probe passes | SB-INS-008, -009 | T-INS-05 |
| `supported_external_python` | a user-supplied interpreter selected by the documented precedence; override honoured | SB-INS-005 | T-INS-07 |
| `missing_package` | each Python-backed action refuses before work, names the package and the interpreter, and re-probes | SB-INS-006, -007 | T-INS-06 |
| `upgrade` | a newer MSI over an older one; settings and projects byte-identical; migration report | SB-INS-011, -022 | T-INS-09 |
| `rollback` | the older MSI back over the newer; data byte-identical; the project opens | SB-INS-022 | T-INS-10 |
| `uninstall_preservation` | uninstall with default choices leaves user data in place | SB-INS-022 | T-INS-11 |

Beside the matrix, four release-time checks that are not scenarios: the fresh-clone gate receipt
(SB-CORE-041, T-INS-01), the support report's completeness and the diagnostic report's redaction
(SB-INS-021, T-INS-13), recovery on the installed build (the gate's own clause, T-INS-12), and the
third-party inventory that fails on an unknown licence and names the pack (SB-INS-024, T-INS-15).

## 5. Increments, in the order proposed

Each is one PR, gated by `tools\check.ps1`, with a REVIEW entry and the matching T-INS tests
ready to run. The order puts the artifact first, because nothing below it can be measured on a
fixture.

1. **G3-01 — a release candidate that exists.** Build the MSI from a genuinely clean clone on the
   reference machine, record digest, version, identifier and commit, and keep the full-gate
   receipt from that clone (SB-CORE-041 T13; SB-INS-001 first half). *T-INS-01.*
2. **G3-02 — clean-machine install and no-Python core use.** `standard_user`, `locked_down_user`,
   `no_python_core_use`; the prerequisite surfaces inventoried and generated from the manifest,
   with divergence failing release (SB-INS-001, -002, -003, -004, -026). *T-INS-02 to T-INS-04.*
3. **G3-03 — the Python pack and the offline install.** Signed pack, lock and digests; deploy MSI
   plus pack with the network blocked; keep the trace and the probe results; the external-
   interpreter path exercised beside it (SB-INS-008, -009, -005). *T-INS-05, T-INS-07.*
4. **G3-04 — preflight and remediation on every Python-backed action,** plus one real re-probe
   control in the prerequisites dialog (SB-INS-006, -007; `missing_package`). *T-INS-06.*
5. **G3-05 — settings, migration and the install lifecycle.** Preservation by default, separate
   recoverable removal consent, a reversible versioned settings migration with its report — the
   report's precedence half waits on O-INS-3 (SB-INS-010, -011, -022; `upgrade`, `rollback`,
   `uninstall_preservation`). *T-INS-08 to T-INS-11.* First item under it, found while writing
   T-INS-09: `resources/install/settings-template.json` carries a hand-maintained
   `template_version`, and `materialize_user_settings` refuses at setup when it differs from
   `tauri.conf.json`'s version — on every FRESH profile's first launch (an existing user copy is
   left alone). No test ties the two together at build time, so a release bump that forgets the
   template fails only when a new account launches. Pin them before the first upgrade candidate
   is cut.
6. **G3-06 — recovery and the support report on the installed build.** The crash record and the
   WAL recovery exercised on the installed candidate; the support report completed with build
   and install identity, configuration digests and the redaction schema (SB-INS-021).
   *T-INS-12, T-INS-13.*
7. **G3-07 — the release gate itself.** Release-time capture of the serviced Windows list, the
   scenario runner, results bound to MSI/pack/commit, the inventory that fails on an unknown
   licence and enumerates the pack, and — after SB-CORE-042 is decided — the machine that runs
   the unchanged gate on every proposed change (SB-INS-023, -024, SB-CORE-042). *T-INS-14,
   T-INS-15.*

## 6. Exit

Gate 3 closes when, on one named candidate: every matrix cell is green with its result bound to
the artifacts; T-INS-01 to T-INS-15 carry Jauhar's marks; the 18 rows above are moved off
`later_gate_only` with their evidence in `requirements.csv`; and a `DEC-*` row records the
closure. Rows blocked on a decision in §2 close as decided, or are carried to Gate 4 by name —
never silently.

## 7. Boundaries this program keeps

- **No package or runtime version is written down before the qualification lock exists.** The
  prerequisites fragment says so today and stays that way.
- **No client identifier anywhere in the evidence.** Test wells are `SANDI-*`; a diagnostic
  report that travels is redacted by the shipped rule.
- **The evidence firewall holds** (SB-INS-025): an opaque artifact beside a pack is inventoried at
  most, never decoded, and no method or default is ever emitted from one.
- **A validator fed a fixture proves the validator.** Only a result produced on a real machine
  from the real candidate counts toward a cell.
