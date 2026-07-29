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
- [ ] **SSC WIP** (other session, not this one): reconcile
      `ssc_swirr_floor_pads_capillary_water` so the full tree gate is green unstashed.

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
| PR #2 brought up to master | `99722f4` pushed | master merged into `docs/prd-and-security-hardening` in a throwaway worktree (main tree untouched); REVIEW.md conflict resolved keeping all rounds newest-first; merged tree `cargo check` clean; branch pushed. The PR now carries R30 + gate + R-A/R-B/R-C + the CSP/docs commits — **your merge click is the only step left** |

**Side effect to know about (one-time, cosmetic):** driving the packaged exe shares the
installed SandiBumi's WebView2 localStorage (same `http://tauri.localhost` origin), so your
installed app's saved dockview layout was overwritten and then cleared — next launch of the
INSTALLED app starts with the default workspace layout once. Project data untouched; dev-mode
(`tauri dev`) layout lives on a different origin and is unaffected.

---

*Update this file (and Section SHIP statuses) in the same commit as anything further this
session ships.*
