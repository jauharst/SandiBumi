# Manual check plan — session of 2026-07-28/29 (PRD + hardening + R-chain)

**Purpose**: everything this session shipped, split by who can prove it. "Machine-verified"
means a gate (tsc / cargo test / browser probe) already proved it — you can spot-check but
don't have to. "**Needs your eyes**" means no gate CAN prove it — these are the checks to do
before calling this session shipped. Updated in realtime as increments land.

Legend: `[ ]` not yet checked by Jauhar · `[x]` accepted · `[!]` found wrong (file a note).

---

## 1. Needs your eyes — no machine gate covers these

### 1.1 CSP hardening in the PACKAGED build — highest-risk unverified item
The new Content-Security-Policy in `src-tauri/tauri.conf.json` is enforced **only in the
packaged app** — `tauri dev` uses the dev server and ignores it entirely. Nothing in this
session ran a packaged build, so the CSP has never been exercised.

- [ ] Run `npm run tauri build` (through the vcvars 14.29 pin), install/launch the output.
- [ ] App opens to a normal window (a blank white window = CSP blocked the bundle).
- [ ] **Vega panel renders a chart** (Vega needs `'unsafe-eval'` — this is the directive
      most likely to be wrong).
- [ ] Equation Editor opens and a Python + a Rhai equation both run.
- [ ] Composite plot → PDF export works.
- [ ] One module dialog runs end-to-end (any module).

### 1.2 R30 — dialogs must fail loudly on a missing permeability curve
- [ ] On a well **with** a PERM/KLOGH/K curve: open Lorenz, SHF, and Facies-Tie dialogs —
      the perm slot preselects the real curve.
- [ ] On a well **without** one: run each — it must fail with the backend's own
      "curve has no data in this well" message, **not** silently compute on GR.

### 1.3 R-A — format stamp on your real project
- [ ] Open your real project, SQL panel: `SELECT * FROM project_meta` → two rows,
      `format_version = 1` and `written_by = SandiBumi <version>`.

### 1.4 R-B — pre-migration backup (this increment)
Your real projects are already PK-less (increment 5 migrated them), so the destructive
migration will not fire and **the pass condition is the absence of any new
`*.pre-1-backup.duckdb` file** beside the project after opening it.

- [ ] Open your real project → no new `*-backup.duckdb` file appears beside it, launch
      is not slower.
- [ ] (Optional, to see it fire) Open any pre-2026-07-19 project copy that still has the
      old PK → a `<name>.pre-1-backup.duckdb` appears beside it BEFORE the rebuild, and
      the launch log (console) says so.

### 1.5 Green gate — run it yourself once
- [ ] `powershell -ExecutionPolicy Bypass -File tools\check.ps1` from your own shell
      finishes GREEN. **Known blocker**: the uncommitted `ssc.rs` WIP (another session)
      fails `ssc_swirr_floor_pads_capillary_water` — the tree is honestly red until that
      session reconciles its test. Everything else is green with it stashed.

### 1.6 Documents — decisions only you can make
- [ ] `docs/PRD.md` + `docs/V1_SCOPE.md` + `docs/TARGET_ARCHITECTURE.md` + `docs/RELEASE.md`
      read-through: do the scope gate (Q1–Q7) and the non-goals match your intent?
- [ ] `docs/IP_PROVENANCE.md` §2.1/§2.2/§2.5 — the lawyer questions (chart data, vendor
      endpoints, client-brand theme names) are yours to route; pick one of the four §2.5
      options before any external release.
- [ ] Python-prerequisite decision (PRD R4): bundle, document, or gate at install time.
- [ ] Opener-plugin removal: if any UI path used to open external links/folders, click it
      once — it should now do nothing rather than crash (no callers were found in code).

### 1.7 PR #2 — branch is behind master
- [ ] PR `docs/prd-and-security-hardening` lacks the newest master commits (R30, gate,
      R-A, R-B). Merge master in (or rebase); REVIEW.md will conflict on the round list —
      **keep all rounds, newest first**. You push/merge with your own credentials.

---

## 2. Machine-verified — gates already proved these (spot-check optional)

| Item | Commit | Proof |
|---|---|---|
| R30 shared `preferredCurveSelect`, 9 call sites | `a90a18c` | tsc + browser probe of all 3 dialogs |
| Green gate script `tools/check.ps1` | `2197086` | proven green, red (real failure), and fail-fast |
| R-A format stamp + refuse-newer | `1842bc8` | 3 new cargo tests; refusal leaves file unmodified |
| R-B pre-migration backup | *(same commit as this file)* | 2 real-file cargo tests: backup exists BEFORE the rebuild, is provably pre-migration (PK intact, all rows), openable; no backup on fresh/already-migrated opens; existing backups never overwritten (collision → timestamped). Copy is engine-made (`COPY FROM DATABASE`) — DuckDB's exclusive file lock makes `fs::copy` impossible, the test caught this on first run. Gate after: 378/0/7 GREEN in 39s |
| CSP syntax + `connect-src ipc:` | `61b2c80` | app boots in dev; packaged check is §1.1 |
| PRD/V1_SCOPE/ARCH/RELEASE numbers | `61b2c80`/`18da8b0` | every count measured from the tree, not quoted |

---

*Update this file in the same commit as any further increment this session ships.*
