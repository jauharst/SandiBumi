# SandiBumi — agent instructions

> **SandiBumi** (formerly *Arshilla*) — the project folder on disk is still `D:\XX. SandiBumi`; only the
> product/branding was renamed. The compiled binary is `sandibumi.exe`, bundle id `com.sandibumi.petro`.

Desktop application for multi-well (2000+) petrophysical log analysis. Stack: **Tauri (Rust) + DuckDB
(embedded, bundled) + TypeScript/WebGPU**.

## Read `CLAUDE.md` first

**[`CLAUDE.md`](CLAUDE.md) is the single authoritative home for every working rule, implementation
contract and convention in this repository. Read it before changing anything here.**

This file used to be a full copy of those rules. It fell **383 lines** behind — an agent reading it
was working from the 2026-08-01 rules, missing the whole of the log-set and Intake contracts, with
nothing in either file to say so. A copy kept in sync by hand is a copy that eventually is not, and
the drift is silent: both files read as authoritative. So this is a pointer now, and so is
[`.cursorrules`](.cursorrules).

## What `CLAUDE.md` carries

- the eleven critical implementation rules — `f32::NAN` never `Option<f32>`, the `bytemuck` IPC byte
  contract, whitelisted writes, Python as a subprocess, undoable edits, the module manifest
- the DuckDB write discipline, including the deliberately PK-less `computed_curves` contract
- the store contracts: log sets, delivery sets, array logs, well images, core registration
- provenance discipline — what may not ship in this tree, and why attributions stay
- the collaboration protocol, the dev commands, and the green gate
- the Organic design system and the UI conventions

## Where the rest lives

| | |
|---|---|
| method math and solver specs | `docs/` |
| what each increment settled, and why | `docs/record_*.md` — indexed in `CLAUDE.md` under **The build record**, which keeps the binding one-liner for each |
| the backlog | `ROADMAP.md` — §4b is the active correctness/perf backlog |
| the field-verification checklist | `REVIEW.md` |
| new-machine setup | `CONTRIBUTING.md` |

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
