//! Multimin — RETIRED legacy mineral/fluid inversion.
//!
//! This fixed four-component (SAND/CLAY/WATER/HYDROCARBON) weighted-NNLS solver has been
//! superseded by SandiMin (`sandimin`), the generalized probabilistic multi-mineral solver,
//! and per Jauhar's design mineral inversion is now independent of Sw. The module is **retired**:
//! `modules::run_module` blocks it via `modules::retired_module`, returning a message that points
//! at SandiMin, so a saved workflow chain (or a `module:multimin` dockview panel) that still
//! references it fails loudly and actionably instead of silently running the superseded physics.
//!
//! The spec below is deliberately kept — and still returned by `list_modules` — only so such a
//! saved step resolves by name and can render its stored parameters while the user re-does it in
//! SandiMin. The former solver body and its tests were removed with the retirement; the volumetric
//! Pe↔U relation R17 introduced here now lives solely in `sandimin::rho_e`, which SandiMin uses.

use crate::modules::{log_in, log_out, param_open, ModuleSpec};

pub(crate) fn multimin_spec() -> ModuleSpec {
    ModuleSpec {
        name: "multimin".into(),
        title: "Multimin — Mineral Inversion (retired · use SandiMin)".into(),
        category: "Saturation".into(),
        doc: "RETIRED — superseded by SandiMin (Advance ▸ Mineral Solver); running this step now \
              returns a message directing you to SandiMin rather than executing the old fixed \
              4-component solver. The spec is kept only so a saved workflow chain that references \
              it still resolves and shows its stored parameters. The former solver produced \
              SAND/CLAY/WATER/HYDROCARBON volumes plus PHIT_MM/VSH_MM/SWT_MM and RECON_ERR from \
              RHOB/NPHI/DT/PEF."
            .into(),
        args: vec![
            // Endpoint responses per component (matrix rows are built from these).
            param_open("RHOB_SAND", "Sand grain density", "g/cc", 2.0, 3.2, true),
            param_open("RHOB_CLAY", "Clay density", "g/cc", 2.0, 3.2, true),
            param_open("RHOB_WATER", "Water density", "g/cc", 0.8, 1.3, true),
            param_open("RHOB_HC", "Hydrocarbon density", "g/cc", 0.1, 1.1, true),
            param_open("NPHI_SAND", "Sand neutron", "v/v", -0.15, 0.5, true),
            param_open("NPHI_CLAY", "Clay neutron", "v/v", 0.0, 0.8, true),
            param_open("NPHI_WATER", "Water neutron", "v/v", 0.5, 1.2, true),
            param_open("NPHI_HC", "Hydrocarbon neutron", "v/v", 0.0, 1.2, true),
            param_open("DT_SAND", "Sand transit time", "us/ft", 40.0, 70.0, true),
            param_open("DT_CLAY", "Clay transit time", "us/ft", 60.0, 150.0, true),
            param_open(
                "DT_WATER",
                "Water transit time",
                "us/ft",
                150.0,
                220.0,
                true,
            ),
            param_open(
                "DT_HC",
                "Hydrocarbon transit time",
                "us/ft",
                150.0,
                260.0,
                true,
            ),
            param_open(
                "PEF_SAND",
                "Sand photoelectric factor",
                "b/e",
                1.0,
                6.0,
                true,
            ),
            param_open(
                "PEF_CLAY",
                "Clay photoelectric factor",
                "b/e",
                1.0,
                6.0,
                true,
            ),
            param_open(
                "PEF_WATER",
                "Water photoelectric factor",
                "b/e",
                0.0,
                2.0,
                true,
            ),
            param_open(
                "PEF_HC",
                "Hydrocarbon photoelectric factor",
                "b/e",
                0.0,
                2.0,
                true,
            ),
            // Per-tool measurement sigma (equation weight = 1/sigma).
            param_open("SIG_RHOB", "RHOB uncertainty", "g/cc", 0.005, 0.5, true),
            param_open("SIG_NPHI", "NPHI uncertainty", "v/v", 0.005, 0.5, true),
            param_open("SIG_DT", "DT uncertainty", "us/ft", 0.5, 50.0, true),
            param_open("SIG_PEF", "PEF uncertainty", "b/e", 0.02, 3.0, true),
            param_open("W_UNITY", "Unity-constraint weight", "", 1.0, 1e6, true),
            log_in("RHOB", "Density log", "g/cc", "RHOB", false),
            log_in("NPHI", "Neutron porosity log", "v/v", "NPHI", false),
            log_in("DT", "Sonic transit time log", "us/ft", "DT", false),
            log_in("PEF", "Photoelectric factor log", "b/e", "PEF", false),
            log_out("VOL_SAND", "Sand (quartz) volume", "v/v"),
            log_out("VOL_CLAY", "Clay volume", "v/v"),
            log_out("VOL_WATER", "Water volume", "v/v"),
            log_out("VOL_HC", "Hydrocarbon volume", "v/v"),
            log_out("PHIT_MM", "Total porosity (water + hc)", "v/v"),
            log_out("VSH_MM", "Shale volume (= clay)", "v/v"),
            log_out("SWT_MM", "Total water saturation", "v/v"),
            log_out("RECON_ERR", "Reconstruction error (sigma units)", ""),
        ],
    }
}
