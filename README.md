# SandiBumi

**SandiBumi** is a Windows desktop application for multi-well petrophysical log analysis,
built for real field workflows (Mahakam Delta / Indonesian basins): field-scale well counts, a
full library of deterministic petrophysical modules, and print-quality deliverables — in a
single native desktop application with an embedded database and capability-level prerequisites.

Stack: **Tauri v2** (Rust) + **DuckDB** (embedded, bundled) + **vanilla TypeScript/WebGPU**.

<!-- capability-prerequisites:start -->
**Runtime prerequisites.** The native core, project open, plotting and native exports do not
require Python. These optional capabilities use one session-resolved Python 3.10+ subprocess:

- **Python equations** — requires numpy; optional scipy (owner: `SB-MLA`).
- **DLIS import** — requires dlisio (owner: `SB-DIO`).
- **Spreadsheet plate extraction** — requires openpyxl, Pillow (owner: `SB-DIO`).
- **Workbook export** — requires xlsxwriter (owner: `SB-DIO`).
- **Document export** — requires python-docx (owner: `SB-DIO`).
- **Deck export** — requires python-pptx, matplotlib (owner: `SB-DIO`).

Offline deployment has one supported route: IT silently deploys the separately signed, versioned
SandiBumi-qualified Python pack per machine. The pack configures `SANDIBUMI_PYTHON` to its
application-local interpreter; qualification blocks public network access. Exact package versions
come only from that release's lock. Open **Project → Help → Prerequisites** for local status.
<!-- capability-prerequisites:end -->

## Highlights

- **Data**: LAS 2.0 in/out, DLIS import, core/XRD/petrography/tops/perforation imports,
  generic curve store with mnemonic-family aliasing, versioned log sets (RAW/EDIT/FINAL)
  with provenance, deviation/TVD, undo everywhere.
- **Petrophysics**: manifest-driven module library (VSH, porosity, Sw families,
  environmental corrections, bad-hole QC + universal mask), Jauhar-method suite
  (SSC/SSPW sand-silt-clay + bound-water, LRLC RtC & IMTS saturation), SandiMin
  multi-mineral optimizer (simultaneous multi-log inversion, 27-component library, conductivity
  coupling), saturation-height, GR normalization, KNN synthetic logs, electrofacies
  (k-means/GMM) + scikit-learn ML suite, workflow chains + Monte Carlo uncertainty,
  pay summary + field dashboard.
- **Plots**: WebGPU log views, histogram/crossplot/Pickett/correlation with synchronized
  hover, Thomas-Stieber, **19 vector-digitized Schlumberger-2013 chart overlays**
  (D-N/sonic/PEF/Th-K/MID), interactive parameter picking that writes zone parameters.
- **Deliverables**: composite log plots at true print scale, vector SVG + multi-page PDF
  report generator (methodology/zone/pay tables).
- **Workspace**: Office-style ribbon, dockable panels, sessions, client-branded themes,
  English / Bahasa Indonesia / Basa Sunda UI.

## Getting started

See **[CONTRIBUTING.md](CONTRIBUTING.md)** for prerequisites and the clone → build → run
walkthrough, and **[CLAUDE.md](CLAUDE.md)** for the engineering rules and machine notes
(read first when developing — with or without Claude).

```sh
npm install
npm run tauri dev
```

Project docs: [ROADMAP.md](ROADMAP.md) (plan + backlog) · [REVIEW.md](REVIEW.md)
(field-verification checklist) · [docs/](docs/) (method math + solver specs).

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
