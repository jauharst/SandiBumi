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
        // Equations copied from modules::sw_arch (the two named identities); citation
        // copied from docs/PRD_v2/12_saturation.md S2.13 (the recorded References block).
        "sw_arch" => Some(ModuleHelp {
            summary: "Archie's equation as two separately named methods, because the \
                      porosity identity changes the answer: archie_total solves on total \
                      porosity and backs effective saturation out through the bound-water \
                      fraction; archie_effective solves on effective porosity directly and \
                      lifts total saturation through the inverse. The identity is declared \
                      by name, never inferred.",
            equations: &[
                "archie_total: SWT = (A·Rw / (PHIT^M · RT))^(1/N)",
                "SWE = max((SWT − Swb)/(1 − Swb), 0),  Swb = 1 − PHIE/PHIT",
                "archie_effective: SWE = (A·Rw / (PHIE^M · RT))^(1/N)",
                "SWT = SWE·(1 − Swb) + Swb",
            ],
            references: &["Archie, G.E. (1942), Transactions AIME 146, 54-62"],
            note: "",
        }),
        // Equations copied from modules::sw_indo's manifest doc; citation copied from
        // docs/PRD_v2/12_saturation.md S2.13.
        "sw_indo" => Some(ModuleHelp {
            summary: "The Poupon-Leveaux Indonesia equation for shaly sands: total \
                      conductivity is a clay term, a clean-sand Archie term and a \
                      geometric cross-term between them, so the clay contribution needs \
                      no separate clay-water resistivity model. Three published forms of \
                      the shale exponent are offered by name.",
            equations: &[
                "1/RT = (v/RT_SH + PHIE^M/(A·Rw) + 2·√(v·PHIE^M/(A·Rw·RT_SH))) · SW^N",
                "v = VSH^(2−VSH)  (FULL),  VSH²  (SIMPLE),  VSH^(2−2·VSH)  (TAR_SAND)",
            ],
            references: &[
                "Poupon, A. & Leveaux, J. (1971), SPWLA 12th Annual Logging Symposium, \
                 Paper O",
            ],
            note: "",
        }),
        // Equations copied from modules::sw_sim's manifest doc (each persisted id names
        // one equation); citations copied from docs/PRD_v2/12_saturation.md S2.13.
        "sw_sim" => Some(ModuleHelp {
            summary: "The Simandoux family as typed equations: every vendor ships a \
                      different arithmetic under the one name, so here each persisted id \
                      names exactly one published form and the run records which was \
                      used. Solved numerically for SWE.",
            equations: &[
                "simandoux_bardon_pied: 1/RT = PHIE^M·SWE^N/(A·Rw) + VSH·SWE/RT_SH",
                "simandoux_modified_slb: 1/RT = PHIE^M·SWE^N/(A·Rw·(1−VSH)) + VSH^C·SWE/RT_SH",
            ],
            references: &[
                "Simandoux, P. (1963), Revue de l'IFP (SPWLA Shaly Sand Reprint Volume \
                 1982 translation)",
                "Bardon, C. & Pied, B. (1969), SPWLA 10th Annual Logging Symposium, \
                 Paper Z",
            ],
            note: "",
        }),
        // Equations copied from lrlc.rs's sw_rtc manifest doc. SandiBumi's own method
        // for LRLC pay - no publication is cited, by the standing rule that user-facing
        // text never cites the analyst's own studies.
        "sw_rtc" => Some(ModuleHelp {
            summary: "SandiBumi's own excess-conductivity method for low-resistivity \
                      low-contrast pay: the conductivity added by clay chemistry and by \
                      capillary-bound (micropore) water is estimated per sample, removed \
                      from the measured conductivity, and Archie is applied to what \
                      remains. The calibration coefficients never ship as defaults - \
                      they are fitted to declared water zones with Calibrate RtC.",
            equations: &[
                "Cex = (A_CAP·CAPBW + B_QV·Qv + C0)·PHIT·RSF",
                "Sw = [Rw·(1/Rt − Cex) / PHIT^M]^(1/N)",
                "Qv = QV log when present, else CEC·RHOG·(1−PHIT)/(100·PHIT)",
            ],
            references: &[],
            note: "SandiBumi's own method - no published primary reference. A \
                   calibration is a property of the dataset it was fitted on and never \
                   transfers between fields.",
        }),
        // Equations copied from lrlc.rs's sw_imts manifest doc; the published lineage
        // citations copied from docs/PRD_v2/12_saturation.md S2.13. The scaling itself
        // is SandiBumi's own and cites no study.
        "sw_imts" => Some(ModuleHelp {
            summary: "Waxman-Smits-family conductivity with the clay charge built from \
                      mineral volumes: Qv comes from kaolinite and illite volumes with \
                      literature CEC constants, is scaled to laboratory CEC by the S \
                      factor, referenced to the active water through Swirr, and the \
                      conductivity equation is iterated to a stable total saturation. \
                      The mineral-textural scaling is SandiBumi's own extension of the \
                      published model.",
            equations: &[
                "Qv_bulk = (V_kaol·ρ_kaol·CEC_kaol + V_ill·ρ_ill·CEC_ill) / (RHOG·(1−PHIT))",
                "Qv_eff = S · Qv_bulk / (1 − Swirr)",
                "Ct = SwT^N*/F* · (Cw + B·Qv_eff/SwT),  F* = A/PHIT^M*",
                "B = Juhász B(T, Rw);  iterate until SwT is stable;  SWE from CBW",
            ],
            references: &[
                "Waxman, M.H. & Smits, L.J.M. (1968), SPEJ",
                "Waxman, M.H. & Thomas, E.C. (1974), SPEJ",
                "Juhász, I. (1979), SPWLA 20th Annual Logging Symposium, Paper AA; and \
                 (1981), SPWLA 22nd",
            ],
            note: "The S factor ships absent and is fitted to laboratory CEC with \
                   Calibrate S; it is a property of the rock and of the clay curves it \
                   was fitted against.",
        }),
        // multimin (retired) deliberately has NO card: its manifest doc already is the
        // full retirement pointer to SandiMin, and a card must carry equations - a
        // retired stub cannot honestly provide them. The guidebook chapter carries the
        // pointer for the book.
        // Equations copied from satheight.rs (module doc and file header, where the two
        // published constants fix their units); citations copied from docs/ref_shf.md's
        // model table (Leverett 1941 Trans. AIME 142; Skelt & Harrison 1995 SPWLA).
        "sw_height" => Some(ModuleHelp {
            summary: "Water saturation from height above the free-water level, through a \
                      fitted capillary-pressure model: the Leverett J-function (fit \
                      SWH_A/SWH_B from core Pc data via Import SCAL) or the \
                      Skelt-Harrison law. Below the FWL the result is 1; above it the \
                      result is limited to [SWT_IRR, 1]. Height is measured from TVD \
                      when a TVD curve is supplied, so deviated wells are not overstated.",
            equations: &[
                "Pc = 0.433·(RHO_W − RHO_HC)·h_ft  (psi; 0.433 psi/ft per unit sp. gravity)",
                "J = 0.21645·Pc/IFT_RES·√(PERM/PHIE),  SWH = SWH_A·J^SWH_B",
                "SKELT: SWH = 1 − SH_A·exp(−(SH_B/(h + SH_D))^SH_C)  (h in metres)",
            ],
            references: &[
                "Leverett, M.C. (1941), Transactions AIME 142 - the J-function",
                "Skelt, C. & Harrison, B. (1995), SPWLA - the height-saturation law",
            ],
            note: "",
        }),
        // Equations and citations copied from modules::perm_wyllie_rose's manifest doc,
        // which carries the full attribution record including the two mislabels.
        "perm_wyllie_rose" => Some(ModuleHelp {
            summary: "The Wyllie-Rose permeability family: permeability from porosity \
                      and irreducible water saturation, offered as four named constant \
                      sets. Wyllie & Rose replaced Carman-Kozeny's specific-surface term \
                      with irreducible saturation and warned the result carries \
                      order-of-magnitude significance only - a warning every constant \
                      set below inherits.",
            equations: &[
                "PERM = (C · PHIE^D / SWE_IRR^E)²  (mD)",
                "TIMUR: C=100 D=2.25 E=1   MORRIS_BIGGS_OIL: C=250 D=3 E=1",
                "MORRIS_BIGGS_GAS: C=79 D=3 E=1   TIXIER: C=250 D=3 E=1",
            ],
            references: &[
                "Wyllie, M.R.J. & Rose, W.D. (1950), Transactions AIME 189, 105-118",
                "Balan, Mohaghegh & Ameri (1995), SPE 30978 - the lineage review",
            ],
            note: "Two of the four constant sets carry a name their author never \
                   attached to them: TIMUR here is the Schlumberger Chart K-3 curve, \
                   not Timur (1968), and TIXIER is a post-1950 Wyllie-Rose \
                   simplification, not Tixier (1949). The MORRIS_BIGGS attribution is \
                   disputed. The card keeps the trade names because that is what every \
                   vendor calls them; the chapter carries the full record.",
        }),
        // Equations and citation copied from modules::perm_coates' manifest doc.
        "perm_coates" => Some(ModuleHelp {
            summary: "The Coates free-fluid permeability: porosity squared times the \
                      free-to-bound water ratio, squared again. This is the Coates & \
                      Denoo producibility form - deliberately not Coates & Dumanoir \
                      (1974), a different and much heavier model this module does not \
                      implement.",
            equations: &[
                "PERM = (C · PHIE² · (1 − SWE_IRR)/SWE_IRR)²  (mD)",
            ],
            references: &[
                "Coates, G.R. & Denoo, S.A. (1981), The Producibility Answer Product, \
                 Schlumberger Technical Review 29(2)",
            ],
            note: "The constant C is scale-dependent and published values are not \
                   interchangeable between the fractional, percent and NMR forms - \
                   check which convention a quoted C belongs to before entering it.",
        }),
        // A utility transform: the coefficients ARE the user's own core calibration,
        // so there is deliberately nothing to cite.
        "perm_transform" => Some(ModuleHelp {
            summary: "The core-calibrated por-perm transform: a straight line in \
                      log-permeability against porosity, with both coefficients fitted \
                      by you against your own core data. The module ships no \
                      coefficients - a por-perm law is a property of the rock family it \
                      was regressed on.",
            equations: &["log10(PERM) = PT_A·PHIE + PT_B"],
            references: &[],
            note: "PT_A and PT_B are the user's own RCAL regression - fit them against \
                   the core porosity-permeability pairs of the field being worked.",
        }),
        // Equations copied from lithology.rs's manifest doc; the chart identities are
        // the module's own recorded derivation (Lith-6, Por-11).
        "midplot" => Some(ModuleHelp {
            summary: "Apparent matrix density and apparent matrix volumetric \
                      photoelectric factor - the two axes of the MID plot. The fluid is \
                      removed from the density and from the photoelectric absorption at \
                      an apparent porosity read the way you would by hand off the \
                      density-neutron chart; crossplot UMAA against RHOMAA and switch \
                      on the MID-plot chart overlay to read lithology.",
            equations: &[
                "U = PEF · ρe,  ρe = (RHOB + 0.1883)/1.0704",
                "RHOMAA = (RHOB − φa·RHO_FL)/(1 − φa)",
                "UMAA = (U − φa·U_FL)/(1 − φa)",
            ],
            references: &[
                "Schlumberger Log Interpretation Charts - Lith-6 (Umaa-Rhomaa MID \
                 plot) and Por-11 (the apparent-porosity read)",
            ],
            note: "",
        }),
        // Equations and citations copied from rocktyping.rs's manifest doc.
        "rocktyping" => Some(ModuleHelp {
            summary: "Per-sample rock-typing indicators from porosity and permeability: \
                      the FZI family, the Winland R35 pore-throat radius, and the \
                      Permadi-Susilo geometric/structural pair. The rock-type class \
                      comes from the chosen method's own binning, and the class-grouped \
                      permeability estimate uses each class's geometric-mean FZI.",
            equations: &[
                "RQI = 0.0314·√(k/φ),  PHIZ = φ/(1−φ),  FZI = RQI/PHIZ",
                "R35 = 10^(0.732 + 0.588·log₁₀k − 0.864·log₁₀φ%)",
                "PGEOM = √(k/φ),  PSTRUC = k/φ^PS_EXP",
                "PERM_RT = 1014.24·FZI_mean(RT)²·φ³/(1−φ)²",
            ],
            references: &[
                "Amaefule et al. (1993) - RQI/FZI",
                "Kolodzie (1980) - Winland R35",
                "Corbett & Potter (2004) - the GHE bin series",
            ],
            note: "Author-year as this project's records hold them; the constants were \
                   re-verified against the records on 2026-07-22.",
        }),
        // Equations and citation copied from rocktyping.rs's lucia_rfn manifest doc.
        "lucia_rfn" => Some(ModuleHelp {
            summary: "Carbonate rock typing by Lucia rock-fabric number: the global \
                      porosity-permeability transform is inverted analytically for RFN, \
                      then binned into the three Lucia classes. The porosity should be \
                      INTERPARTICLE porosity; on clastic-dominated fields this applies \
                      to carbonate stringers only.",
            equations: &[
                "log₁₀k = (A − B·log₁₀RFN) + (C − D·log₁₀RFN)·log₁₀φip, inverted for RFN",
                "RFN 0.5-1.5 = Class 1 (grainstone), 1.5-2.5 = Class 2, 2.5-4 = Class 3",
            ],
            references: &[
                "Lucia (1995); Jennings & Lucia (2003), SPE 78740",
            ],
            note: "",
        }),
        // Equations and citation copied from rocktyping.rs's pittman_rx manifest doc,
        // including the paper's own two cautions.
        "pittman_rx" => Some(ModuleHelp {
            summary: "Pittman's pore-throat aperture family: the pore-throat radius at \
                      each mercury-saturation level from 10 to 75 percent, from \
                      porosity and permeability. The apex radius is the one that best \
                      correlates with permeability for the rock family, and its \
                      Hartmann-Beaumont port class is written beside it.",
            equations: &[
                "log₁₀ rX = C0 + C1·log₁₀k + C2·log₁₀φ%  (k mD, φ percent, r µm)",
                "Port classes: nano < 0.1 < micro < 0.5 < meso < 2.5 < macro < 10 ≤ mega (µm)",
            ],
            references: &[
                "Pittman, E.D. (1992), AAPG Bulletin v76 no.2, 191-198 - Table 1 in \
                 full, verified against the paper",
            ],
            note: "The paper's own cautions ride along: correlation falls with \
                   saturation (0.926 at r20 to 0.820 at r75), and the family is \
                   non-monotone below ~11% porosity - that is the published \
                   arithmetic, not an implementation artifact.",
        }),
        // The cutoff ladder is the user's own declaration; nothing to cite.
        "rt_cutoff" => Some(ModuleHelp {
            summary: "A log-domain rock-type class from a declared Vsh + PHIE cutoff \
                      ladder - the electrofacies half of the rock-typing tie-in. Class \
                      1 is the best rock, class 3 is non-net; validate the ladder \
                      against a core-derived rock type with the confusion-matrix QC \
                      before attaching per-class laws.",
            equations: &[
                "RT_LOG = 1 if Vsh ≤ VSH1 and PHIE ≥ PHI1",
                "RT_LOG = 2 if Vsh ≤ VSH2 and PHIE ≥ PHI2  (requires VSH1 ≤ VSH2, PHI1 ≥ PHI2)",
                "else RT_LOG = 3",
            ],
            references: &[],
            note: "The cutoffs are the interpreter's own declaration for the field \
                   being worked - they ship absent, and the chapter shows one way to \
                   derive a starting ladder from the data itself.",
        }),
        // Equations copied from modules.rs's thin_bed_ts manifest doc; the citation is
        // the derivation record's.
        "thin_bed_ts" => Some(ModuleHelp {
            summary: "Thomas-Stieber decomposition of bulk shale into laminar and \
                      dispersed fractions, by placing the measured (VSH, PHIT) point \
                      against the pure-laminated and pure-dispersed mixing lines. \
                      Laminar shale reduces net sand; dispersed shale stays within the \
                      sand fraction. Structural shale is not modeled.",
            equations: &[
                "Pure laminated: PHIT = PHI_SD_MAX·(1−VSH) + PHI_SH·VSH",
                "Pure dispersed: PHIT = PHI_SD_MAX − VSH·(1−PHI_SH)",
                "VSAND = 1 − VLAM;  PHIE_LAM = laminar-corrected porosity of the net sand",
            ],
            references: &["Thomas, E.C. & Stieber, S.J. (1975)"],
            note: "Author-year as this project's method record holds it; the full \
                   primary citation is pending verification.",
        }),
        // --- The Prep / Condition / Frame utility shelf (increment 5). Equations are
        // copied from each module's own manifest doc and arithmetic; most are
        // definitional utilities and carry no published reference, per the field's
        // doc comment above. The Arps constant is the one the code computes with
        // (modules.rs precalc); the Hampel scaling is robust::C_MAD.
        "ftemp_grad" => Some(ModuleHelp {
            summary: "Formation temperature from a linear trend. GRADIENT mode takes a \
                      surface temperature plus a constant gradient with depth; BHT mode \
                      interpolates linearly from the surface temperature to a bottom-hole \
                      temperature at its measured depth.",
            equations: &[
                "GRADIENT: FTEMP = TSURF + TGRAD·depth",
                "BHT: FTEMP = TSURF + (BHT − TSURF)·depth/TD_BHT",
            ],
            references: &[],
            note: "",
        }),
        "precalc" => Some(ModuleHelp {
            summary: "Reservoir-condition inputs for saturation and SandiMin work, from \
                      linear trend fits in true vertical depth: formation temperature and \
                      pressure, mud-filtrate resistivity at formation temperature, and the \
                      QC conductivities CT and CXO.",
            equations: &[
                "FTEMP = SURF_TEMP + TEMP_GRAD·TVDSS;  FPRESS = PSURF + PGRAD·TVDSS",
                "ARPS: RMF@FTEMP = RMF·(T_meas + 6.77)/(FTEMP + 6.77)  [temperatures in °F]",
                "TREND: RMF@FTEMP = RMF_A + RMF_B·log10(TVDSS)",
                "CT = 1000/RT;  CXO = 1000/RXO  [mmho/m]",
            ],
            references: &[],
            note: "The Arps temperature conversion is carried by name as this project's \
                   method record holds it. The shipped trend starting values are \
                   feet-based starting points - refit per basin, and convert before a \
                   metric well.",
        }),
        "badhole" => Some(ModuleHelp {
            summary: "Flags samples where the borehole departs from gauge or the density \
                      correction is too large to trust the porosity logs. The flag is 1 \
                      in bad hole, 0 in good hole, and MISSING where no criterion could \
                      be evaluated; companion curves record which criterion fired and \
                      which were evaluable at all.",
            equations: &[
                "BADHOLE = 1 where |DRHO| > DRHO_MAX or |CALI − BS| > DCAL_MAX",
                "BS from the BS curve where present, else the interpreter's BS_INPUT; \
                 no value is substituted when both are absent",
            ],
            references: &[],
            note: "Feed BADHOLE as the Mask on later module runs so flagged intervals go \
                   missing instead of polluting results.",
        }),
        "condflag" => Some(ModuleHelp {
            summary: "Flags samples whose density-neutron readings should not feed \
                      porosity or mineral solving: coal, tight rock, gas crossover, and \
                      the shoulder samples either side of a flagged bed, where the logs \
                      still average across the boundary. COND_FLAG combines them for use \
                      as a run mask.",
            equations: &[
                "COAL_FLAG: RHOB < COAL_RHOB and NPHI > COAL_NPHI (and DT > COAL_DT where a sonic exists); never in bad hole",
                "TIGHT_FLAG: density porosity and NPHI both < TIGHT_PHI",
                "XOVER_FLAG: (RHO_MA − RHOB)/(RHO_MA − RHO_FL) − NPHI > XOVER_MIN",
                "beds thinner than MIN_THICK are dropped as spikes; SHOULDER_FLAG within SHOULDER of a flagged bed",
                "COND_FLAG = coal + tight + bad hole + shoulder (+ crossover when OPT_XCOND = YES)",
            ],
            references: &[],
            note: "Convert the neutron to units consistent with RHO_MA before trusting \
                   the crossover flag, and leave the Mask empty on the condflag run \
                   itself. Run the Bad-Hole QC module first.",
        }),
        "nphimat" => Some(ModuleHelp {
            summary: "Converts a neutron log recorded in one matrix convention into all \
                      three (limestone, sandstone, dolomite) using the chartbook \
                      porosity-equivalence curves - Por-5 for the CNL thermal tools, \
                      Por-4 for the epithermal APS and the sidewall SNP. Limestone units \
                      are the chart's apparent-limestone axis, on which calcite is the \
                      identity.",
            equations: &[
                "φa = the input matrix curve inverted back to the apparent-limestone axis",
                "NPHI_LS = φa;  NPHI_SS = C_SS(φa);  NPHI_DOL = C_DOL(φa)",
                "where C_SS and C_DOL are the chart's matrix curves; the input convention passes through unchanged",
            ],
            references: &[
                "Schlumberger Log Interpretation Charts - Por-5 (CNL thermal neutron) \
                 and Por-4 (APS epithermal and SNP), the porosity-equivalence curve \
                 families",
            ],
            note: "Apply environmental corrections first - the charts assume corrected \
                   logs. SALINITY = INTERPOLATE evaluates the fresh and 250-kppm charts \
                   completely and interpolates the finished answers (TNPH family only).",
        }),
        "gascorr" => Some(ModuleHelp {
            summary: "Removes the gas effect from the density log by an iterated \
                      density-neutron loop: porosity and Archie water saturation are \
                      solved from the current density, the gas volume is replaced with \
                      liquid, and the loop repeats until porosity settles. Gas density \
                      comes from a real-gas calculation at formation pressure and \
                      temperature.",
            equations: &[
                "RHOB_GC = RHOB + PHIT·(1 − SWT)·(RHO_FL − GASDEN)",
                "iterated until |ΔPHIT| < 1e-4 (max 20 passes; non-converging samples stay MISSING)",
                "GASDEN: real-gas density of an SG_GAS gas at FPRESS/FTEMP (Standing pseudo-criticals, Papay z-factor)",
            ],
            references: &[
                "Standing - natural-gas pseudo-critical correlations",
                "Papay (1968) - z-factor correlation",
            ],
            note: "Correlation names as this project's method record holds them; the \
                   full primary citations are pending verification. Feed RHOB_GC to \
                   density porosity - not to a density-neutron combination, whose gas \
                   handling assumes an uncorrected pair.",
        }),
        "gr_hole_corr" => Some(ModuleHelp {
            summary: "Linear borehole-enlargement correction for the gamma-ray log: \
                      counts attenuated by the extra mud annulus in an oversize hole are \
                      restored in proportion to the enlargement.",
            equations: &[
                "GR_EC = GR·(1 + K_GR·(CALI − BS))",
                "BS from the BS curve where present, else BS_DEF",
            ],
            references: &[],
            note: "The run refuses if CALI is missing at any finite GR sample - it never \
                   writes an unmarked uncorrected copy under the corrected name.",
        }),
        "nphi_env_corr" => Some(ModuleHelp {
            summary: "Linearized environmental correction for the neutron log: a \
                      formation-temperature term about a reference temperature and a \
                      formation-salinity term. Requires formation temperature from the \
                      Formation Temperature module; without it only the salinity term \
                      applies.",
            equations: &[
                "NPHI_EC = NPHI + K_TEMP·(FTEMP − T_REF) + K_SAL·(SALW/100000)",
            ],
            references: &[],
            note: "The shipped coefficients are practitioner starting values - replace \
                   them with values read from the applicable CNL chart for the tool in \
                   hand. SALW defaults to the chart reference condition (fresh), so the \
                   salinity term is inert until the study declares its formation \
                   salinity.",
        }),
        "rhob_hole_corr" => Some(ModuleHelp {
            summary: "Restores the density log in oversize holes, where the pad reads \
                      too much mud: a linear correction in the hole enlargement beyond a \
                      reference diameter, using supplied tool-specific chart values.",
            equations: &[
                "RHOB_EC = RHOB + K_RHO·(CALI − HD_REF)  for CALI > HD_REF; unchanged within gauge",
            ],
            references: &[],
            note: "Use with the BADHOLE flag - beyond a few inches of washout no \
                   correction is trustworthy.",
        }),
        "gr_normalize" => Some(ModuleHelp {
            summary: "Two-point percentile normalization of the gamma-ray log onto a \
                      field reference frame, so wells can be compared and pooled. The \
                      well's own percentiles are computed from this run's samples; the \
                      reference percentiles are parameters you must supply.",
            equations: &[
                "GRN = (GR − Plow,well)·(Phigh,ref − Plow,ref)/(Phigh,well − Plow,well) + Plow,ref",
            ],
            references: &[],
            note: "The reference pair ships absent on purpose: a pair from one basin is \
                   the wrong reference in another. Derive it from the field's own \
                   multi-well GR distribution and use the SAME pair for every well in \
                   the study.",
        }),
        "log_predict" => Some(ModuleHelp {
            summary: "Synthetic log by distance-weighted K-nearest-neighbour regression: \
                      trains on this run's samples where the target and every predictor \
                      are present, then predicts the target wherever the predictors \
                      exist. Predictors are z-scored so no one curve's units dominate \
                      the distance.",
            equations: &[
                "prediction = Σ wi·yi / Σ wi over the K nearest training samples",
                "wi = 1/(di + 1e-6), di = Euclidean distance in z-scored predictor space",
                "MAX_RAW: OUT = max(raw, synthetic)  - the washout rule for RHOB, since bad hole only pushes RHOB down",
            ],
            references: &[],
            note: "Mask the run to good-hole intervals so bad samples never train the \
                   model.",
        }),
        "depth_shift" => Some(ModuleHelp {
            summary: "Shifts a curve by a block amount in depth and resamples it back \
                      onto the well's own grid by linear interpolation. Positive shift \
                      moves the feature deeper; the input curve is never modified.",
            equations: &[
                "OUT(z) = CURVE(z − SHIFT), resampled onto the well's depth grid  (+SHIFT = deeper)",
            ],
            references: &[],
            note: "SHIFT is zone-overridable, so different intervals can take different \
                   block shifts.",
        }),
        "splice" => Some(ModuleHelp {
            summary: "The classic run-to-run splice: one curve above the splice depth, \
                      the other at and below it. Inputs are never modified.",
            equations: &[
                "SPLICED(z) = TOP_CURVE(z)  for z < SPLICE_DEPTH",
                "SPLICED(z) = BOT_CURVE(z)  for z ≥ SPLICE_DEPTH",
            ],
            references: &[],
            note: "",
        }),
        "despike" => Some(ModuleHelp {
            summary: "Replaces samples that stand off their neighbours with the local \
                      median, over a window stated as a thickness of rock rather than a \
                      sample count. Four tests are offered, from the robust Hampel \
                      deviation test to a simple rate-of-change limit.",
            equations: &[
                "HAMPEL: replace where |x − median| > K·(c·MAD), c the Gaussian consistency constant  [zero-MAD windows fall back to the mean deviation]",
                "ABS: replace where |x − median| > THRESH",
                "MEDIAN: every sample → window median (no test)",
                "RATE: replace where the change from the previous live sample exceeds MAX_RATE per depth unit",
            ],
            references: &[],
            note: "Set WINDOW narrower than the thinnest bed you intend to keep - a bed \
                   no thicker than the window is indistinguishable from a spike. The \
                   Gaussian consistency constant (the reciprocal of the standard-normal \
                   75th percentile) is a mathematical estimator constant, not a \
                   calibration.",
        }),
        "smooth" => Some(ModuleHelp {
            summary: "Averages a curve over a window stated as a thickness. A missing \
                      sample stays missing and no gap is bridged - smoothing never \
                      fills, because a filled sample is a claim about rock nobody \
                      logged.",
            equations: &[
                "MEAN: arithmetic mean of the live samples in the window",
                "MEDIAN: window median - keeps a step edge where a mean would ramp across it",
                "SAVGOL: local quadratic least-squares fit on the real (depth, value) pairs, evaluated at the sample",
            ],
            references: &[],
            note: "Despike first: a least-squares smoother fits whatever is in the \
                   window, so over an un-despiked curve the spike is not removed but \
                   spread across the window and made to look like rock.",
        }),
        "clip" => Some(ModuleHelp {
            summary: "Holds a curve inside a declared range. An empty bound is a \
                      statement that the curve is unbounded on that side, not an \
                      omission.",
            equations: &[
                "BLANK: x outside [MIN, MAX] → MISSING",
                "CLAMP: x → min(max(x, MIN), MAX)",
            ],
            references: &[],
            note: "BLANK is the default because it is the one that cannot manufacture a \
                   measurement: a resistivity of 1e6 is not a very resistive rock, it is \
                   a reading the tool could not make.",
        }),
        "fill_gaps" => Some(ModuleHelp {
            summary: "Fills holes no wider than a declared maximum and marks every \
                      sample it invented in a companion flag curve, so a filled value \
                      can always be told from a logged one. A gap open at one end is \
                      never filled - that would extrapolate past where the tool \
                      stopped.",
            equations: &[
                "LINEAR: a straight line between the live samples either side (gap ≤ MAX_GAP, bounded both ends)",
                "HOLD: the last live value carried down",
                "<OUT>_FILL = 1 on every invented sample",
            ],
            references: &[],
            note: "Mask on the flag curve to take invented samples back out of any later \
                   run. MAX_GAP has no default: how far interpolation is defensible \
                   depends on why the data is missing.",
        }),
        "flip" => Some(ModuleHelp {
            summary: "Mirrors a curve about a pivot - for an SP recorded with the wrong \
                      sign convention, or any reading delivered inverted.",
            equations: &[
                "OUT = 2·pivot − CURVE",
                "pivot: a given VALUE, or this well's own MIDRANGE or MEAN",
            ],
            references: &[],
            note: "MIDRANGE and MEAN are computed per well, so two wells' flipped curves \
                   are no longer on a common scale - use a VALUE pivot for anything that \
                   feeds a correlation.",
        }),
        "normalize" => Some(ModuleHelp {
            summary: "Maps any curve onto a common reference frame so wells can be \
                      compared and pooled: a two-point percentile map, a min-max range \
                      map, or a z-score to a reference mean and spread, optionally in \
                      log10 space.",
            equations: &[
                "TWO_POINT: OUT = (x − P_LOW)·(REF_HIGH − REF_LOW)/(P_HIGH − P_LOW) + REF_LOW",
                "RANGE: the same map from the curve's own MIN and MAX",
                "MEAN_SD: OUT = (x − mean)/sd·REF_SD + REF_MEAN",
                "SPACE = LOG: mapped in log10 and inverted afterwards; non-positive samples become MISSING",
            ],
            references: &[],
            note: "The reference pair has no default, and that is the point: a pair from \
                   one basin is the wrong pair in another, and normalized output looks \
                   plausible either way. Derive yours from the field's own distribution \
                   and use the same pair for every well.",
        }),
        "block" => Some(ModuleHelp {
            summary: "Replaces a curve with one value per bed, held across the bed, with \
                      the bed definition and the averaging statistic both declared. The \
                      curve stays on the well's own depth frame, so nothing downstream \
                      has to know it was upscaled.",
            equations: &[
                "beds: INTERVAL slices, CLASS runs of a constant value, ZONES marker intervals, or AUTO from the curve itself",
                "MEAN - right for porosity and every volume fraction, because those add",
                "GEOMETRIC: k = (k1·k2·…·kn)^(1/n)  - the standard permeability estimate in randomly heterogeneous rock",
                "HARMONIC: k = n / Σ(1/ki)  - permeability across layers in series",
                "MODE - the commonest value, and the only upscale for a class curve",
            ],
            references: &[],
            note: "Set the blocked curve's draw style to Step, or the log view draws a \
                   gradient between block values the data never measured. A class curve \
                   refuses every averaging statistic: the mean of facies 1 and facies 4 \
                   is 2.5, which is not a facies.",
        }),
        "bed_detect" => Some(ModuleHelp {
            summary: "Finds bed boundaries from a curve's own steps and writes the bed \
                      number each sample falls in - the same segmentation Block's AUTO \
                      mode uses, exposed on its own so the beds can be looked at on a \
                      log before anything is averaged over them.",
            equations: &[
                "a new bed opens where |x − bed mean| > SENS·noise and the bed already spans MIN_BED",
                "noise = robust spread of the curve's first differences / √2  - the curve's noise, not its variability across the well",
            ],
            references: &[],
            note: "SandiBumi's own segmentation heuristic. Over-segmentation is what a \
                   step-finder gets wrong when it gets anything wrong: put the bed curve \
                   in a track as class blocks and judge it against the log before \
                   running Block on it.",
        }),
        // --- Facies + Unconventional (increment 6, the close of the catalog).
        // Citations copied from docs/ref_unconventional.md and the modules' own docs;
        // the two facies modules are standard algorithms with no primary citation
        // recorded, and their cards say so.
        "electrofacies" => Some(ModuleHelp {
            summary: "Unsupervised electrofacies by k-means: this well's samples are \
                      clustered in the space of the supplied curves (z-scored by \
                      default, so mixed units are comparable) into K classes. Labels \
                      are ordered by the mean of the first supplied curve - usually GR \
                      - so FACIES 0 is the cleanest class and the numbering is \
                      monotone in shaliness.",
            equations: &[
                "assign each sample to the nearest of K centroids in z-scored curve space",
                "centroids re-estimated as class means, iterated to convergence (k-means++ seeding, best of several starts)",
                "labels reordered by ascending mean of the first curve",
            ],
            references: &[],
            note: "Standard k-means - no primary citation is recorded in this \
                   project's method ledger; the label-ordering and z-scoring \
                   conventions are SandiBumi's own. Deterministic for a given seed.",
        }),
        "gmm_facies" => Some(ModuleHelp {
            summary: "Soft electrofacies by Gaussian mixture: every sample gets a \
                      membership probability per class rather than a hard assignment. \
                      FPROB is the winning class's posterior - 1.0 is unambiguous, \
                      about 1/K is a boundary or mixed sample - so transitional beds \
                      are visible instead of being forced into a class.",
            equations: &[
                "diagonal-covariance Gaussian mixture fitted by EM, initialized from k-means",
                "FACIES_GMM = the class with the highest posterior;  FPROB = that posterior (0-1)",
                "labels reordered by ascending mean of the first curve",
            ],
            references: &[],
            note: "Standard Gaussian mixture - no primary citation is recorded in \
                   this project's method ledger; conventions as the k-means module. \
                   Deterministic for a given seed.",
        }),
        "toc_passey" => Some(ModuleHelp {
            summary: "Total organic carbon from the Passey ΔlogR overlay - the \
                      separation between deep resistivity and a baselined porosity \
                      curve, converted to TOC with a maturity term - plus the \
                      Schmoker-Hester density TOC as an independent cross-check \
                      whenever a density log is present.",
            equations: &[
                "sonic overlay: ΔlogR = log10(R/R_BASE) + 0.02·(DT − DT_BASE)",
                "density overlay: ΔlogR = log10(R/R_BASE) − 2.5·(RHOB − RHOB_BASE)",
                "TOC = ΔlogR·10^(2.297 − 0.1688·LOM) + TOC_BG  [wt%; ΔlogR < 0 floors to the background]",
                "cross-check: TOC_SCHMOKER = 154.497/RHOB − 57.261",
            ],
            references: &[
                "Passey, Creaney, Kulla, Moretti & Stroud (1990), \"A practical model \
                 for organic richness from porosity and resistivity logs,\" AAPG \
                 Bulletin 74(12): 1777-1794",
                "Schmoker & Hester (1983), \"Organic carbon in Bakken Formation,\" \
                 AAPG Bulletin 67(12): 2165-2174",
            ],
            note: "Baselines are picked on a non-source, clay-rich interval where the \
                   two curves overlie, and TOC is very sensitive to them. LOM 6-12 \
                   (Hood-Gutjahr-Heacock scale); the conversion is calibrated to \
                   LOM <= 12. The Schmoker fit is Bakken-specific - a cross-check, \
                   not a substitute.",
        }),
        "kerogen" => Some(ModuleHelp {
            summary: "Converts TOC (a weight percent) to kerogen volume and corrects \
                      total porosity for the organic matter that low-density kerogen \
                      inflates on the density log. Kerogen occupies a volume \
                      disproportionate to its weight because it is light.",
            equations: &[
                "TOM = K_TOC2OM·TOC/100  [organic-matter weight fraction; the factor covers H/O/N/S beyond carbon]",
                "VKER = TOM·RHOB/RHO_KERO  [kerogen volume fraction of the bulk rock]",
                "PHIT_OMC = PHIT − VKER",
            ],
            references: &[
                "Passey et al. (2010), SPE 131350",
                "Vernik & Nur (1992)",
            ],
            note: "Author-year as this project's method record holds them. The \
                   default kerogen density 1.10 g/cc matches the SandiMin Kerogen \
                   mineral, so VKER reconciles with VOL_KEROGEN; a 1.25 seed from \
                   another vendor tradition is available as an override.",
        }),
        "gip" => Some(ModuleHelp {
            summary: "Per-sample gas-in-place as gas content (scf per ton of rock), \
                      so it composites like any curve: free gas from porosity and \
                      saturation through the gas formation-volume factor, adsorbed \
                      gas through the Langmuir isotherm, and their total. A CBM mode \
                      adds the dry-ash-free correction and the critical desorption \
                      pressure.",
            equations: &[
                "GIP_ADS = VL·P/(PL + P)",
                "GIP_FREE = 32.0368·φ·(1 − Sw)/(RHOB·Bg),  Bg = 0.02827·z·T/P  [T in Rankine]",
                "GIP_TOTAL = GIP_FREE + GIP_ADS",
                "cbm: GIP_ADS·(1 − F_ASH − F_MOIST);  PCD = PL·GC/(VL − GC)",
            ],
            references: &[
                "Langmuir (1918), J. Am. Chem. Soc. 40(9): 1361-1403",
                "Mavor & Nelson (1996) - the GRI petro-application of the isotherm",
                "Ambrose et al. (2010), SPE 131772",
            ],
            note: "The Langmuir pair VL/PL ships absent and requires matching core \
                   desorption or isotherm data - an isotherm from another basin is \
                   the wrong isotherm. The Ambrose pore-volume correction is \
                   deferred, so high-TOC high-pressure free gas reads slightly \
                   high.",
        }),
        "brittleness" => Some(ModuleHelp {
            summary: "Brittleness index, 0 ductile to 1 brittle, two ways: elastic - \
                      dynamic Young's modulus and Poisson's ratio from the sonic \
                      pair and density, normalized and averaged per Rickman - or \
                      mineralogical, from a mineral solve's volume fractions.",
            equations: &[
                "G = ρ·Vs²;  K = ρ·(Vp² − 4/3·Vs²);  ν = (3K − 2G)/(2(3K + G));  E = 9KG/(3K + G)",
                "Rickman: BI = (E_norm + ν_norm)/2, E over E_LO..E_HI (Mpsi), ν over NU_LO..NU_HI",
                "Jarvie: BI = Qz/(Qz + carbonate + clay)",
                "Wang-Gale: BI = (Qz + Dol)/(Qz + Dol + calcite + clay + organic)",
            ],
            references: &[
                "Rickman, Mullen, Petre, Grieser & Kundert (2008), SPE 115258",
                "Jarvie et al. (2007), AAPG Bulletin 91",
                "Wang & Gale (2009), GCAGS Transactions 59",
            ],
            note: "The shipped normalization endpoints (1-8 Mpsi, 0.40-0.15) are \
                   Rickman's own Barnett calibration on dynamic log values - \
                   recalibrate per basin, and apply any static correction after \
                   this index, not before. Mineral volumes come from a SandiMin \
                   run; a missing mineral counts as absent.",
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
        "sspw", "sw_arch", "sw_indo", "sw_sim", "sw_rtc", "sw_imts", "sw_height",
        "perm_wyllie_rose", "perm_coates", "perm_transform", "midplot", "rocktyping",
        "lucia_rfn", "pittman_rx", "rt_cutoff", "thin_bed_ts",
        "ftemp_grad", "precalc", "badhole", "condflag", "nphimat", "gascorr",
        "gr_hole_corr", "nphi_env_corr", "rhob_hole_corr", "gr_normalize", "log_predict",
        "depth_shift", "splice", "despike", "smooth", "clip", "fill_gaps", "flip",
        "normalize", "block", "bed_detect",
        "electrofacies", "gmm_facies", "toc_passey", "kerogen", "gip", "brittleness",
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
