# Manual check plan — session of 2026-07-28/29 (PRD + hardening + R-chain)

**The app tests from this session now live in `docs/manual_test_plan.md` → Section SHIP**
(T-SHIP-01…06: packaged-build CSP, R30, R-A, R-B, green gate) — run them there so results
land in the one UAT ledger. This file keeps only the **non-app items** and the session's
machine-verification record. Updated in realtime as items close.

Legend: `[ ]` not yet checked by Jauhar · `[x]` accepted · `[!]` found wrong.

---

## 1. Non-app items — decisions and actions only you can take

- [ ] **Documents read-through**: `docs/PRD.md`, `docs/V1_SCOPE.md` (scope gate Q1–Q7),
      `docs/TARGET_ARCHITECTURE.md`, `docs/RELEASE.md` — do the non-goals match your intent?
- [ ] **IP_PROVENANCE lawyer questions** (`docs/IP_PROVENANCE.md` §2.1/§2.2/§2.5): chart
      data, vendor endpoints, client-brand theme names — pick one of the four §2.5 options
      before any external release.
- [ ] **Python-prerequisite decision** (PRD R4): bundle, document, or gate at install time.
- [ ] **Opener-plugin removal**: if any UI path used to open external links/folders, click
      it once — should do nothing rather than crash (no callers were found in code).
- [ ] **PR #2 merge**: `docs/prd-and-security-hardening` now carries every master commit
      (merge `99722f4` pushed 2026-07-29) — review and click merge with your own credentials.
      Until it merges, **master still has `csp: null`**: the hardened CSP ships only via
      this PR.
- [x] **RENAME THE FOLDER — DONE 2026-07-29.** Jauhar ran `tools/rename-to-sandibumi.ps1`
      with Claude Code closed (Claude cannot do it from inside a session: a process's
      working directory is an open handle, and the project folder is Claude Code's own cwd,
      so Windows refuses the rename). Verified after: `D:\XX. SandiBumi` live, old path
      gone, git clean, **both worktrees repaired (0 prunable)**, 0 stale path references
      outside `.vs` cache and the dev playbook. The script itself keeps its old-path
      references on purpose — it is the migration record, and it is idempotent (a re-run
      now reports "Already renamed").
- [x] **SSC gate blocker — RESOLVED** (was: another session's WIP). The work landed in your
      `d1f0c1e`; the stale test it left behind is corrected. Gate is green unstashed.
      **But `d1f0c1e` changed SSC numbers** (gas conditioning → RMS midpoint) — REVIEW.md
      Round 95 has the field re-check.

## 2. Machine-verification record (spot-check optional)

| Item | Commit | Proof |
|---|---|---|
| R30 shared `preferredCurveSelect`, 9 call sites | `a90a18c` | tsc + browser probe of all 3 dialogs |
| Green gate script `tools/check.ps1` | `2197086` | proven green, red (real failure), and fail-fast |
| R-A format stamp + refuse-newer | `1842bc8` | 3 cargo tests; refusal leaves file unmodified |
| R-B pre-migration backup | `0ba199b` | 2 real-file cargo tests: backup-first (openable, PK intact, all rows); no backup on fresh/no-op opens; collision → timestamped. Engine copy (`COPY FROM DATABASE`) — DuckDB's exclusive file lock blocks `fs::copy`, caught by the test's first run. Gate 378/0/7 |
| CSP syntax + `connect-src ipc:` | `61b2c80` | app boots in dev; packaged check = T-SHIP-01/02 |
| PRD/V1_SCOPE/ARCH/RELEASE numbers | `61b2c80`/`18da8b0` | every count measured from the tree, not quoted |
| Packaged-build CSP (T-SHIP-01/02) | verified 2026-07-29 | packaged debug exe driven over the WebView2 debug port with the PR's CSP applied: policy live (probe violations quote it; remote fetch + inline script blocked), eval/Function allowed, full UI + Vega scatter rendered, zero unexpected violations. NOTE: an earlier same-day run was invalid — master's config still has `csp: null`; only the PR branch carries the CSP, so the verified build used the PR's `tauri.conf.json` |
| R-C exit checkpoint (T-SHIP-07) | *(same commit as this file)* | found by the packaged-build run: EVERY close (window ✕ included) abandoned a live WAL → next open could fail replay → writes since last checkpoint silently lost (reproduced 2/2, a fresh import vanished). Fixed with a `RunEvent::Exit` CHECKPOINT; verified by the exact failing scenario — no WAL after close, import survives relaunch. Gate 378/0/7 |
| Folder rename prep (247 refs → `D:\XX. SandiBumi`) | *(same commit as this file)* | Four path spellings swept incl. the escaped `D:\\` form used in JSON/JS; guards proved the GitHub repo name `XX.-Arshilla` (2) and `ARSHILLA_PYTHON` (13) untouched; 0 stale paths left. `tools/rename-to-sandibumi.ps1` fixture-tested on a throwaway repo with a linked worktree: rename, both worktrees repaired (0 prunable), linked branch resolves, idempotent re-run |
| T-SHIP-02 remaining legs (Equation Editor, Composite PDF, printCanvas) | verified 2026-07-29, post-rename | Packaged build with the PR's CSP, proven enforced in that build by blocked probes quoting our own directives. CodeMirror mounts; Rhai + Python runs wrote 20 rows each with exact values; composite PDF written (`%PDF-`, 5,634 B); printCanvas iframe/`data:`/inline-style path clean. 0 violations, 0 console errors. **Post-rename gotcha found**: cargo's target cache had the OLD absolute path baked into Tauri's build-script output, so every Rust build failed until `cargo clean -p sandibumi -p tauri` |
| SSC stale test corrected | *(same commit as this file)* | Test asserted `SWIRR_T >= SWIRR_MIN`, contradicting its own name and both references; the floor pads CWSH while SWIRR_T is the pre-conditioning ratio (`.lls` 213-216 + method_ssc_sspw.md §8). No physics touched. Gate **378/0/7 GREEN in 54s, nothing stashed** |
| PR #2 brought up to master | `99722f4` pushed | master merged into `docs/prd-and-security-hardening` in a throwaway worktree (main tree untouched); REVIEW.md conflict resolved keeping all rounds newest-first; merged tree `cargo check` clean; branch pushed. The PR now carries R30 + gate + R-A/R-B/R-C + the CSP/docs commits — **your merge click is the only step left** |

**Side effect to know about (one-time, cosmetic):** driving the packaged exe shares the
installed SandiBumi's WebView2 localStorage (same `http://tauri.localhost` origin), so your
installed app's saved dockview layout was overwritten and then cleared — next launch of the
INSTALLED app starts with the default workspace layout once. Project data untouched; dev-mode
(`tauri dev`) layout lives on a different origin and is unaffected.

---

*Update this file (and Section SHIP statuses) in the same commit as anything further this
session ships.*
