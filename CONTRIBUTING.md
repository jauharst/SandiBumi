# Contributing to SandiBumi

SandiBumi is a Windows desktop petrophysics application: **Tauri v2** (Rust backend +
WebView2 frontend), **DuckDB** for project storage, **vanilla TypeScript** + WebGPU for
the UI (no frontend framework). This guide gets a new developer from clone to running
app, and explains how we work.

## 1. Prerequisites (Windows 10/11)

| Tool | Notes |
|---|---|
| **Rust** (stable, MSVC) | `rustup` default. |
| **Visual Studio Build Tools** | C++ workload **with the v14.29 (VS2019) toolset component**. Newer toolsets have been broken on the reference machine (see CLAUDE.md "Dev commands"); 14.29 is the pinned, known-good one. |
| **Node.js LTS** | for Vite + TypeScript. |
| **Python 3.10+** with `numpy`, `scikit-learn` | powers the Equation Editor and ML modules (runs as a subprocess — see CLAUDE.md rule 7). `xgboost` optional. |
| **WebView2 runtime** | preinstalled on Win 11. |

## 2. Build & run

```sh
npm install          # once
npx tsc --noEmit     # fast frontend type check
cd src-tauri && cargo check   # fast Rust-only check
cargo test           # Rust unit/integration tests (real petrophysics math is tested)
```

Full app (note the pinned toolset — adapt the VS path to your install):

```
cmd.exe /c "call \"C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat\" -vcvars_ver=14.29 && set PATH=C:\Program Files\nodejs;%USERPROFILE%\.cargo\bin;%PATH% && npm run tauri dev"
```

`npm run tauri build` produces the production installer.

**Never force-kill the dev app** (task-kill, shell timeout) — an unclean kill mid-write
can corrupt the project's DuckDB WAL. Let it exit on its own. Recovery exists
(`db::init_db_resilient`) but don't lean on it.

## 3. Where things live

| Path | What |
|---|---|
| `src/` | TypeScript UI. `main.ts` boots; `ui/ribbon.ts` = ribbon; `ui/*Panel.ts` = dock panels; `LogCanvasRenderer.ts` = WebGPU log view; `state.ts` = observables (well selection, theme, hover depth); `ipc.ts` = typed Tauri commands. |
| `src-tauri/src/` | Rust backend. `modules.rs` = petrophysics module manifests + dispatch; `equations.rs` = curve read/write; `multimin2.rs` = SandiMin solver; `composite.rs` = print/PDF; `db.rs` = DuckDB. |
| `CLAUDE.md` | **Read first.** Hard-won machine/runtime rules (Python discovery, WAL resilience, toolset pin, dockview quirks). |
| `ROADMAP.md` | Full plan + **§4 = current field-review backlog** (priority-ordered). |
| `REVIEW.md` | Click-through checklist for features awaiting field verification. Mark `[o]` = OK, `[x]` = wrong, leave `[ ]` untested. |
| `docs/` | Method specs (the reference suite/IP mineral-solver extraction, etc.). |

## 4. How we work

- **Branches**: `master` is the integration branch. Work on feature branches
  (`feat/<topic>`, `fix/<topic>`) and merge after review — don't push broken `master`.
- **Never commit data**: `*.duckdb` / WAL files and `.db-backups/` are git-ignored on
  purpose. Well data stays out of the repo.
- **Every user-facing change** gets a check item in `REVIEW.md` so Jauhar (or any
  interpreter) can verify it against real field data.
- **Modules**: a new petrophysics method = a Rust function + a manifest entry in
  `modules.rs` — the dialog UI is auto-generated. Keep physics in Rust, single-sourced.
- **Verification bar** before merging: `npx tsc --noEmit` clean, `cargo check` clean,
  `cargo test` green, and a manual smoke of the touched panel.
- **Petrophysics defaults** come from documented sources (the reference suite `.info` exports, field
  studies) — cite the source in a comment when adding constants.

## 5. Remote / sharing

The repo is local-first. To collaborate, add any git remote the team agrees on
(GitHub/GitLab private repo, or a bare repo on a shared drive):

```sh
git remote add origin <url>
git push -u origin master
```

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
