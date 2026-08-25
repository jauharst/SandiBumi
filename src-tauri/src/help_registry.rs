//! Per-module Help-card content - the brief method statement, the equations in plain
//! Unicode, and the PUBLISHED references (paper or publication only). Internal
//! provenance (PRD sections, Geolog line numbers, vendor helpfiles) stays in the
//! manifests' own source strings and in the guidebook chapter, where it belongs; it
//! never appears on this card.
//!
//! A registry keyed by module id, like `param_sources`, rather than new fields on
//! `ModuleSpec` - adding a card never touches the 52 manifest literals. The dump gate
//! (`manifest_reference_test.rs`) merges these entries into
//! `docs/generated/module_manifests.json` under a `help` key, so the HTML guidebook
//! (`tools/gen-guidebook.mjs`) renders the same card from the same single home.
//!
//! CITATION DISCIPLINE (docs/guidebook_prompt.md): a reference is COPIED from where
//! the repo records it, never written from memory. For the vsh_gr transforms the repo
//! deliberately records only author-year leads - docs/PRD_v2/10_clay-volume.md
//! escalation E4 marks the primary citations unverified, records that IP2018's
//! reported citations for these three are fabricated, and forbids closing the gap
//! from a secondary source (refusal R8). So the card carries exactly the recorded
//! leads and says its citations are pending verification. When E4 is resolved the
//! full citations replace the leads HERE and flow to card and book together.

use std::path::{Path, PathBuf};

#[derive(Clone, serde::Serialize)]
pub struct ModuleHelp {
    /// Two or three sentences: the method statement, without implementation caveats.
    pub summary: &'static str,
    /// Plain Unicode lines, copied from the module's own arithmetic - never LaTeX.
    pub equations: &'static [&'static str],
    /// Published references exactly as the repo records them. Empty for utilities.
    pub references: &'static [&'static str],
    /// Optional user-facing caveat under the reference lines. Empty means none.
    pub note: &'static str,
}

pub fn module_help(module: &str) -> Option<ModuleHelp> {
    match module {
        // Equations copied from `modules::vsh_gr` (the match on OPT_GR); the exact
        // normalised Larionov forms are DEC-096, the published decimals kept for parity.
        "vsh_gr" => Some(ModuleHelp {
            summary: "Shale volume from the gamma-ray log. A gamma-ray index is taken \
                      between a clean-sand endpoint (GR_MA) and a shale endpoint (GR_SH), \
                      then optionally passed through a published non-linear transform - \
                      Stieber, Larionov or Clavier. VSH is the result limited to 0-1; \
                      VSH_GR keeps the unlimited value beside it.",
            equations: &[
                "IGR = (GR − GR_MA) / (GR_SH − GR_MA)",
                "LINEAR: VSH = IGR",
                "Stieber: VSH = IGR / (3 − 2·IGR)   [variants: IGR / (2 − IGR), IGR / (4 − 3·IGR)]",
                "Larionov, Mesozoic and older: VSH = (2^(2·IGR) − 1) / (2² − 1)   [published decimal: 0.33·(2^(2·IGR) − 1)]",
                "Larionov, Tertiary / unconsolidated: VSH = (2^(3.7·IGR) − 1) / (2^3.7 − 1)   [published decimal: 0.083·(2^(3.7·IGR) − 1)]",
                "Clavier: VSH = 1.7 − √(3.38 − (IGR + 0.7)²)",
            ],
            references: &[
                "Larionov (1969) - the Mesozoic-and-older and Tertiary transforms",
                "Stieber (1970/71) - the three ratio forms",
                "Clavier et al. (1971)",
            ],
            note: "Author and year as this project's method ledger records them; the full \
                   primary citations are pending verification.",
        }),
        // Equations copied from `modules::vsh_dn_rearrangement`. No primary publication
        // is recorded for the two-log crossplot rearrangement (METHOD_DERIVATIONS cites
        // the PRD chapter and vendor artefacts only), and the card says so.
        "vsh_dn" => Some(ModuleHelp {
            summary: "Shale volume from the density-neutron crossplot: the (RHOB, NPHI) \
                      point's position between the clean matrix line and the shale point. \
                      The neutron shale endpoint is clay-type sensitive, so an optional GR \
                      comparison raises VSH_DN_FLAG where the two indicators diverge or the \
                      point falls off the matrix-shale-fluid triangle. VSH is the result \
                      limited to 0-1; VSH_DN keeps the unlimited value.",
            equations: &[
                "VSH_DN = (A − B) / (C − D), where on the (NPHI, RHOB) crossplot:",
                "A = (RHO_MA − RHO_FL)·(NPHI_FL − NPHI)",
                "B = (RHOB − RHO_FL)·(NPHI_FL − NPHI_MA)",
                "C = (RHO_MA − RHO_FL)·(NPHI_FL − NPHI_SH)",
                "D = (RHO_SH − RHO_FL)·(NPHI_FL − NPHI_MA)",
                "VSH = VSH_DN limited to 0–1",
            ],
            references: &[],
            note: "No published primary reference is recorded in this project's method \
                   ledger for the two-log crossplot rearrangement.",
        }),
        // Equations copied from the module's own manifest doc (modules::phi_den_spec).
        "phi_den" => Some(ModuleHelp {
            summary: "Density porosity with shale correction. Effective porosity reads the \
                      density log between the matrix and fluid densities, minus the shale \
                      term; total porosity adds the shale's own porosity back. Above 95% \
                      VSH the sample is treated as shale.",
            equations: &[
                "PHIE = (RHO_MA − RHOB)/(RHO_MA − RHO_FL) − VSH·(RHO_MA − RHO_SH)/(RHO_MA − RHO_FL)",
                "PHIT = PHIE + VSH·PHIT_SH",
                "PHIT_SH = (RHO_DSH − RHO_SH)/(RHO_DSH − RHO_W)",
            ],
            references: &[],
            note: "The density-porosity transform is the cross-vendor-agreed definitional \
                   form; no separate primary publication is recorded in this project's \
                   method ledger.",
        }),
        // Equations copied from the module's own manifest doc (modules::phi_dn_spec).
        "phi_dn" => Some(ModuleHelp {
            summary: "Quick-look density-neutron porosity: shale-reduces RHOB and NPHI, \
                      converts each to porosity, and combines them as the simple average or \
                      the gas RMS. A comparison shortcut only - deliberately not a crossplot \
                      porosity method; for the analytic crossplot use Porosity from \
                      Bateman-Konen.",
            equations: &[
                "AVERAGE: PHIX = (PHID + PHIN)/2",
                "GAS_RMS: PHIX = √((PHID² + PHIN²)/2)",
                "PHIE = PHIX·(1 − VSH)",
                "PHIT = PHIE + VSH·PHIT_SH",
            ],
            references: &[],
            note: "Field quick-look shortcuts. No primary publication is recorded, and the \
                   vendor manuals themselves say these combinations should not be used \
                   beyond comparison.",
        }),
        // Equations copied from `modules::bk_pseudo_mineral` and the B-6/B-7 solve;
        // the full primary citation is recorded above those constants in modules.rs.
        "phi_dnbk" => Some(ModuleHelp {
            summary: "The chart-free analytic neutron-density crossplot, solved as a \
                      two-pseudo-mineral system in limestone units. The apparent matrix \
                      density RHOMAA_BK is an output, not an input; shale reduction clamps \
                      the neutron side only, per the method's own source.",
            equations: &[
                "Pseudo-mineral pair (Appendix B), by side of the density-porosity line:",
                "φN ≥ φD:  φNa = 0.7 − 10^−(5·φN + 0.16)   [B-11/B-12, ρ2 = 4.00]",
                "φN < φD:  φDa = 1,  φNa = −(1.17 + 2.06·φN) + 10^−(0.4 + 16·φN)   [B-9/B-10]",
                "PHIX = (φDa·φN − φD·φNa) / (φDa − φNa)   [B-6]",
                "RHOMAA_BK = (RHOBsr − PHIX·RHO_FL) / (1 − PHIX)   [B-7]",
                "PHIE = PHIX·(1 − VSH),  PHIT = PHIE + VSH·PHIT_SH",
            ],
            references: &[
                "Bateman, R.M. & Konen, C.E., Wellsite Log Analysis and the Programmable \
                 Pocket Calculator, SPWLA 18th Annual Logging Symposium, June 1977, \
                 Appendix B",
            ],
            note: "",
        }),
        // Equations copied from the module's own manifest doc (modules::phi_son_spec);
        // the RHG80 constants are paper-verified per DEC-079.
        "phi_son" => Some(ModuleHelp {
            summary: "Sonic porosity with three transforms, each named for what it \
                      computes: the Wyllie time-average, the genuine three-segment \
                      Raymer-Hunt-Gardner 1980 transform, and the field-observation \
                      transform with a cited coefficient. Wyllie is shale-corrected \
                      subtractively and optionally compaction-corrected; the other two use \
                      the normalised shale convention.",
            equations: &[
                "WYLLIE: PHIT = (DT − DT_MA)/(DT_FL − DT_MA)   [÷ Cp, Cp = DT_SH/100, when OPT_CP = ON]",
                "RHG80: φ < 37%: invert V = (1−φ)²·Vma + φ·Vf;  φ > 47%: fluid-suspension form;  37–47%: Δt-linear interpolation",
                "FIELD_OBSERVED: PHI = CFO·(DT − DT_MA)/DT",
                "Non-Wyllie shale convention: dtsr = (DT − VSH·DT_SH)/(1 − VSH), floored at DT_MA; PHIE = transform(dtsr)·(1 − VSH)",
            ],
            references: &[
                "Raymer, Hunt & Gardner, SPWLA 21st Annual Logging Symposium, 1980, paper P",
                "Wyllie (1956/1958) - the time-average transform",
            ],
            note: "The Raymer-Hunt-Gardner constants are verified against a copy of the \
                   paper held in the project library; the Wyllie citation is the \
                   author-year lead as this project's method ledger records it.",
        }),
        // Equations copied from the module's own manifest doc (modules::phimax_spec);
        // Athy (1930) is the author-year lead as the ingest bibliography records it.
        "phimax" => Some(ModuleHelp {
            summary: "Caps an input porosity at the field's compaction-controlled ceiling: \
                      a constant, a linear TVDSS trend, or an Athy exponential trend. \
                      Writes the capped curve and the ceiling curve for QC; the input \
                      porosity is never modified.",
            equations: &[
                "constant: φmax = PHIMAX0",
                "linear: φmax = PHIMAX0 − PHIMAX_GRAD·(TVDSS − TVDSS_REF)/1000",
                "athy: φmax = PHIMAX0·exp(−ATHY_K·(TVDSS − TVDSS_REF)/1000)",
                "<PHI>_CAP = min(PHI, φmax)",
            ],
            references: &["Athy (1930) - the exponential compaction trend"],
            note: "Author and year as this project's records hold them; the full primary \
                   citation is pending verification.",
        }),
        // Equations copied from docs/method_ssc_sspw.md (the ported Loglan's own spec);
        // the Kuttan citation is copied from ssc.rs's module header.
        "ssc" => Some(ModuleHelp {
            summary: "Sand-Silt-Clay model on the neutron-density crossplot: gas-conditions \
                      the (RHOB, NPHI) pair, projects it onto the dry line of a six-point \
                      framework, splits the rock into sand, silt and clay fractions, and \
                      carries the split through porosity, bound-water and \
                      irreducible-saturation arithmetic.",
            equations: &[
                "Gas conditioning (Δ = |PHIDI² − NPHI²|, c = GAS_C):",
                "PHID = √(PHIDI² − c·Δ/2),  NPHI_COR = √(NPHI² + c·Δ/2)",
                "PHIT = (RHOMA − RHOB_COR)/(RHOMA − RHOB_FL),  RHOMA = Σ fraction·ρ over sand/silt/clay",
                "PHIE = PHIT − VWCL·PHIT_CL,  PHIFF = PHIT − CBW − CWSH",
                "SWIRR_T = BW/PHIT,  SWIRR_EFF = 1 − PHIT·(1 − SWIRR_T)/PHIE",
            ],
            references: &[
                "Kuttan et al., Log Interpretation in the Malay Basin, 21st SPWLA Annual \
                 Logging Symposium",
            ],
            note: "The modification and the two tight-rock conditioning rules are \
                   SandiBumi's own additions to the published model.",
        }),
        // Equations copied from docs/method_ssc_sspw.md. SSPW is the interpreter's own
        // bound-water workflow, reconstructed from its specification - no publication.
        "sspw" => Some(ModuleHelp {
            summary: "Bound-water porosity split: effective porosity is total porosity \
                      minus clay-bound water only, capillary-bound water stays inside PHIE, \
                      and PHIFF is the porosity that flows. Outputs the bound-water volumes \
                      and irreducible saturation.",
            equations: &[
                "PHIE = PHIT − CBW  (clay-bound water only)",
                "PHIFF = PHIT − CBW − CAPBW",
                "Gas conditioning corrects the density leg only (same c parameter as SSC)",
            ],
            references: &[],
            note: "SandiBumi's own bound-water workflow - no published primary reference. \
                   Reconstructed from the method specification; validation against the \
                   reference-suite exports is an open review item.",
        }),
        _ => None,
    }
}

/// Where this module's guidebook chapter lives, if it exists. The bundled app carries
/// the book as a resource (`guide/book/`); a dev checkout falls back to the repo's own
/// `docs/guide/book/`, so the link works in both layouts and simply hides when the
/// chapter is not written yet.
pub fn guide_chapter_path(resource_dir: Option<PathBuf>, module: &str) -> Option<PathBuf> {
    if module.is_empty()
        || !module
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return None;
    }
    let file = format!("{module}.html");
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(dir) = resource_dir {
        candidates.push(dir.join("guide").join("book").join(&file));
    }
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../docs/guide/book")
            .join(&file),
    );
    candidates.into_iter().find(|p| p.is_file())
}

/// Open a chapter in the OS default browser - `cmd start`, zero dependencies. The
/// path was just verified to exist by `guide_chapter_path`, and its file name is
/// constrained to `[a-z0-9_].html` there, so nothing user-shaped reaches the shell.
pub fn open_in_default_browser(path: &Path) -> Result<(), String> {
    let mut cmd = std::process::Command::new("cmd");
    cmd.arg("/c").arg("start").arg("").arg(path);
    crate::python_engine::hide_console(&mut cmd);
    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("Could not open the guidebook chapter: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Module ids that carry a help card. The test below pins that every id here is a
    /// real module, so a typo in a `module_help` match arm cannot sit unreachable.
    /// Test-only on purpose: production reaches cards through `module_help` alone.
    const MODULES_WITH_HELP: &[&str] = &[
        "vsh_gr", "vsh_dn", "phi_den", "phi_dn", "phi_dnbk", "phi_son", "phimax", "ssc",
        "sspw",
    ];

    /// Pins the card's own rule from both sides: every registered id is a real module
    /// (a typo'd match arm cannot sit unreachable), and no reference line carries
    /// internal provenance - PRD sections, Loglan files and section marks belong to
    /// the guidebook and the manifests, never to the published-reference card.
    #[test]
    fn the_help_card_carries_publications_never_internal_provenance() {
        let module_ids: Vec<String> = crate::modules::list_modules()
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert!(!MODULES_WITH_HELP.is_empty());
        for id in MODULES_WITH_HELP {
            assert!(
                module_ids.iter().any(|m| m == id),
                "help registry names '{id}', which is not a module id"
            );
            let help = module_help(id).unwrap_or_else(|| {
                panic!("'{id}' is listed in MODULES_WITH_HELP but module_help returns None")
            });
            assert!(!help.summary.is_empty() && !help.equations.is_empty());
            for r in help.references {
                for marker in ["docs/", ".lls", ".info", "PRD", "\u{a7}", ".htm"] {
                    assert!(
                        !r.contains(marker),
                        "reference on the '{id}' card carries internal provenance ('{marker}'): {r}"
                    );
                }
            }
        }
        // The other side: an id with a card must be listed, or the dump and the list
        // disagree about what exists.
        for id in &module_ids {
            if module_help(id).is_some() {
                assert!(
                    MODULES_WITH_HELP.contains(&id.as_str()),
                    "module '{id}' has a help card but is missing from MODULES_WITH_HELP"
                );
            }
        }
    }

    /// The path helper refuses anything that is not a plain module id - the one gate
    /// between an IPC string and a shell `start`.
    #[test]
    fn a_guide_path_is_only_ever_a_plain_module_id() {
        for bad in ["", "vsh gr", "../evil", "vsh_gr.html", "VSH_GR", "a&b"] {
            assert!(guide_chapter_path(None, bad).is_none(), "accepted '{bad}'");
        }
    }
}
