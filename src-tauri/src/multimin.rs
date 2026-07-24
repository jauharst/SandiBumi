//! Multimin — RETIRED legacy mineral/fluid inversion.
//!
//! This fixed four-component (SAND/CLAY/WATER/HYDROCARBON) weighted-NNLS solver has been
//! superseded by SandiMin (`multimin2`), the generalized probabilistic multi-mineral solver,
//! and per Jauhar's design mineral inversion is now independent of Sw. The module is **retired**:
//! `modules::run_module` blocks it via `modules::retired_module`, returning a message that points
//! at SandiMin, so a saved workflow chain (or a `module:multimin` dockview panel) that still
//! references it fails loudly and actionably instead of silently running the superseded physics.
//!
//! The spec below is deliberately kept — and still returned by `list_modules` — only so such a
//! saved step resolves by name and can render its stored parameters while the user re-does it in
//! SandiMin. The former solver body and its tests were removed with the retirement; the volumetric
//! Pe↔U relation R17 introduced here now lives solely in `multimin2::rho_e`, which SandiMin uses.

use crate::modules::{log_in, log_out, param, ModuleSpec};

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
            param("RHOB_SAND", "Sand grain density", "g/cc", 2.65, 2.0, 3.2),
            param("RHOB_CLAY", "Clay density", "g/cc", 2.55, 2.0, 3.2),
            param("RHOB_WATER", "Water density", "g/cc", 1.0, 0.8, 1.3),
            param("RHOB_HC", "Hydrocarbon density", "g/cc", 0.8, 0.1, 1.1),
            param("NPHI_SAND", "Sand neutron", "v/v", -0.02, -0.15, 0.5),
            param("NPHI_CLAY", "Clay neutron", "v/v", 0.30, 0.0, 0.8),
            param("NPHI_WATER", "Water neutron", "v/v", 1.0, 0.5, 1.2),
            param("NPHI_HC", "Hydrocarbon neutron", "v/v", 0.55, 0.0, 1.2),
            param("DT_SAND", "Sand transit time", "us/ft", 55.5, 40.0, 70.0),
            param("DT_CLAY", "Clay transit time", "us/ft", 90.0, 60.0, 150.0),
            param("DT_WATER", "Water transit time", "us/ft", 189.0, 150.0, 220.0),
            param("DT_HC", "Hydrocarbon transit time", "us/ft", 210.0, 150.0, 260.0),
            param("PEF_SAND", "Sand photoelectric factor", "b/e", 1.81, 1.0, 6.0),
            param("PEF_CLAY", "Clay photoelectric factor", "b/e", 3.10, 1.0, 6.0),
            param("PEF_WATER", "Water photoelectric factor", "b/e", 0.36, 0.0, 2.0),
            param("PEF_HC", "Hydrocarbon photoelectric factor", "b/e", 0.12, 0.0, 2.0),
            // Per-tool measurement sigma (equation weight = 1/sigma).
            param("SIG_RHOB", "RHOB uncertainty", "g/cc", 0.03, 0.005, 0.5),
            param("SIG_NPHI", "NPHI uncertainty", "v/v", 0.03, 0.005, 0.5),
            param("SIG_DT", "DT uncertainty", "us/ft", 5.0, 0.5, 50.0),
            param("SIG_PEF", "PEF uncertainty", "b/e", 0.30, 0.02, 3.0),
            param("W_UNITY", "Unity-constraint weight", "", 1000.0, 1.0, 1e6),
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
