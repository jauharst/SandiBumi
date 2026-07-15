# Arshilla — Petrophysical Software Engine

Desktop application for multi-well (2000+) petrophysical log analysis. Stack: **Tauri (Rust) + DuckDB (embedded, bundled) + TypeScript/WebGPU**.

This file is the Claude Code equivalent of `.cursorrules` (kept in this repo for Cursor). Keep both in sync if the rules change.

## Critical implementation rules

1. **Data storage**: optimize DuckDB queries for columnar reading. Use compressed binary blobs or `LIST` types for array logs (NMR, waveforms).
2. **Missing values**: never use `Option<f32>` for continuous logs. Missing data is strictly `f32::NAN`, so matrix arithmetic stays branch-free.
3. **Serialization & IPC**: never pass raw data arrays over Tauri's IPC bridge as JSON strings. Convert `Vec<f32>` matrices to raw bytes with `bytemuck`, return `Vec<u8>`, and cast to `Float32Array` on the frontend.
4. **Concurrency**: `rayon` for CPU-bound cell/well-parallel work; `tokio` for background async scheduling (long-running inversions, I/O).
5. **Code delivery**: concise, modular, production-speed-focused. No extensive unit test blocks unless explicitly requested.

## Build phases (from `Prompt/Claude_Implementation_Guide.pdf`)

- **Phase 0** — IDE/environment config (this file + `.cursorrules`, toolchain). Done.
- **Phase 1** — DuckDB schema + ingestion (`wells`, `standard_curves`, `high_res_curves`, `lqr_parameters`, `array_logs`) + batch Appender insert.
- **Phase 2** — LAS 2.0 streaming parser + Geolog CSV parser, `rayon`-parallel over a directory of files.
- **Phase 3** — Decimation (LTTB/min-max) + zero-copy binary IPC bridge (`bytemuck` → `Vec<u8>` → `Float32Array`).
- **Phase 4** — WebGPU/WebGL2 canvas renderer (`LogCanvasRenderer`), GPU buffers, pan/zoom re-triggering backend decimation.
- **Phase 5** — Multi-well deterministic petrophysics engine (Vsh/porosity/Sw) via `rayon::par_iter`, plus `tokio` task queue for long stochastic inversions.

Work through phases sequentially; verify `cargo check` (Rust) and `npm run dev` (frontend) after each before moving on.

## Environment notes (as set up on this machine)

Rust, Node.js, and the MSVC linker are all installed and working — **but new shells may not pick up PATH updates from installers**. If `cargo`/`node`/`npm` report "not found," don't assume they're missing; verify with the full paths below before reinstalling anything:

- `cargo`/`rustc`/`rustup`: `C:\Users\ARUNIKA\.cargo\bin\`
- `node`/`npm`: `C:\Program Files\nodejs\`

Prepend both to `PATH` at the start of a shell session, e.g. (Git Bash):
```sh
export PATH="/c/Program Files/nodejs:$USERPROFILE/.cargo/bin:$PATH"
```

DuckDB uses the `bundled` Cargo feature (compiles from source), so no system DuckDB install is required. The MSVC linker was already present on this machine (verified via a real `cargo build`), so no separate Visual Studio Build Tools install was needed.

## Dev commands

```sh
npm install            # install frontend deps (already done)
npm run tauri dev      # run the full desktop app (Rust + frontend, hot reload)
cd src-tauri && cargo check   # fast Rust-only compile check
npm run build           # production frontend build
npm run tauri build     # production desktop bundle
```

## Project layout

- `src-tauri/` — Rust backend: DuckDB access, parsers, IPC commands, petrophysics engine.
- `src/` — TypeScript frontend: WebGPU log canvas renderer, Tauri IPC calls.
- `Prompt/` — original phase-by-phase spec (`Claude_Implementation_Guide.pdf`).
