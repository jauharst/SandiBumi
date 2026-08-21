//! SandiMin — user-defined multi-mineral / fluid optimizer (file renamed from
//! multimin2.rs when the internal names caught up with the product name), modeled on
//! the reference multi-mineral solver and IP's Mineral Solver (spec extracted from both installs, see
//! docs/multimin_ref_spec.md and docs/multimin_ip_spec.md).
//!
//! Formulation (standard convention):
//! - One volume vector per depth frame: minerals + clays (common to both zones) plus
//!   flushed-zone (X / Sxo) and unflushed-zone (U / Sw) fluid sets.
//! - Every tool responds to X-zone fluids except the deep conductivity CT, which sees
//!   the U zone; CXO (flushed conductivity) sees the X zone.
//! - Resistivity enters as CONDUCTIVITY via the DUAL WATER LINEAR transform: with
//!   w = 0.75·m + 0.25·n the response row  Ct^(1/w) = Σ v_i · C_i^(1/w)  is linear in
//!   the volumes (C_i = fluid conductivity endpoints; minerals/hydrocarbons are 0).
//!   The measured resistivity curve is converted (C = 1/R) and transformed before entering
//!   the system. Row uncertainty: U_ct = 0.03·Cw^(1/w), U_cxo = 0.03·Cmf^(1/w).
//! - Hard constraints: Σ(minerals + clays + U-zone fluids) = 1 (UNITY, X fluids excluded)
//!   and per-component box bounds 0 ≤ v ≤ max_vol (fluids default 0.5).
//! - Soft "Tool" constraints at σ = 0.01 (treated as pseudo-measurements):
//!   POROSITY (Σ X fluids = Σ U fluids) and BNDWAT (bound water tied to clay volumes via
//!   k = 96·CEC·ρ_clay / (T°C + 298) · α, the Dual-Water clay-bound-water multiplier).
//!   WATER MUD (Sxo ≥ Sw for water-based mud) is enforced by a re-solve when violated.
//!
//! The solver is a bounded, equality-constrained active-set least squares (KKT system with
//! a unity Lagrange multiplier; components can be fixed at 0 or at their upper bound).
//! RECON is the INCOHERENCE — the σ-weighted RMS of (reconstructed − measured) over the live tool
//! rows. This is the standard goodness-of-fit statistic for a weighted least-squares log-response
//! inversion: the residual vector divided by each tool's own uncertainty, so tools measured to
//! different precisions contribute on comparable terms (Mayer & Sibbit, SPE 9341, "GLOBAL, a new
//! approach to computer-processed log interpretation", 1980 — the primary source for the
//! simultaneous log-response inversion this module implements). A high value flags a model that
//! cannot reproduce the logs. With `recon_qc` the reconstruction is
//! decomposed per tool: `<prefix>_<KEY>_REC` (measurement rebuilt from the volumes, display units)
//! and `<prefix>_<KEY>_DIF` (that tool's σ-unit residual, whose RMS over tools is RECON), so the
//! user can see WHICH log the model fails to honour. The reconstruction only discriminates when the
//! system is over-determined — the reported `dof` says whether that holds.

use crate::equations::fetch_curve_frame;

/// The log set SandiMin output lands in when the caller names none — the value that used to be
/// hardcoded, so an older payload writes exactly where it always did.
const DEFAULT_SANDIMIN_SET: &str = "SANDIMIN";
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Request / result types
// ---------------------------------------------------------------------------

/// One mineral, clay, or fluid component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    /// "mineral" | "clay" | "fluid"
    #[serde(default)]
    pub kind: String,
    /// Fluids only: "X" (flushed / Sxo), "U" (unflushed / Sw), or "" (seen by both zones).
    #[serde(default)]
    pub zone: String,
    /// Fluids only: "water" | "bound_water" | "oil" | "gas".
    #[serde(default)]
    pub fluid_type: String,
    /// Tool-key → endpoint response, in display units (g/cc, v/v, us/ft, API, ...).
    pub endpoints: HashMap<String, f64>,
    /// Tool-key → per-value provenance (SB-MIN-009 / SB-CORE-005, DEC-078). Also carries
    /// `CEC` and `WCP` entries for the two row scalars, and `VP`/`VS` entries recording the
    /// derivation. The library fills this; the run dialog replaces an edited value's entry
    /// with a user-supplied marker, and the whole map rides `params_json` with the
    /// submitted components so every run record carries its endpoint custody.
    #[serde(default)]
    pub endpoint_sources: HashMap<String, String>,
    /// Cation exchange capacity, meq/g (clays; drives the bound-water constraint under the CEC
    /// porosity source).
    #[serde(default)]
    pub cec: f64,
    /// Wet-clay total porosity φ_clay (clays) — the alternative bound-water driver under the
    /// Wet-Clay-Porosity source: v_bw = φ_clay/(1−φ_clay)·v_dryclay (no CEC/T/α). Default 0.
    #[serde(default)]
    pub wet_clay_porosity: f64,
    /// Upper volume bound (default: 1.0 minerals, 0.5 fluids).
    #[serde(default = "default_one")]
    pub max_vol: f64,
}

fn default_one() -> f64 {
    1.0
}

/// A tool (input log) in the inversion. Keys "CT"/"CXO" are conductivity rows: `curve`
/// is a RESISTIVITY curve (ohmm) converted to conductivity (mho/m) per sample; their
/// endpoints come from the fluid properties, not from the endpoints table. `sigma <= 0`
/// on CT/CXO means "auto" (0.03·C^(1/w)).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub key: String,
    pub curve: String,
    pub sigma: f64,
}

/// Saturation model for SandiMin's deep/flushed conductivity tools (Jauhar's Sw-equation request).
///
/// `linear_dw` (default) is the current in-inversion linearised dual-water `Ct^(1/w) = Σ v·C^(1/w)`
/// with a single exponent `w = 0.75m + 0.25n`; it is linear in the volume vector and leaves every
/// already-reviewed number untouched. The shaly-sand forms are **post-solve**: the mineral inversion
/// runs on the lithology tools with NO conductivity row, then Sw is computed from the closed form
/// using the solved effective porosity + shale volume and the deep resistivity, and the U-zone
/// water/HC volumes are redistributed to honour it (so PHIE is unchanged and SWE = the model Sw).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SwModel {
    /// Linearised dual-water, in-inversion. Default — nothing moves.
    #[default]
    LinearDw,
    /// Clavier-Coates-Dumanoir dual-water, exact form honouring m and n separately (Newton form
    /// solved by bisection). Post-solve; the bound-water saturation comes from the solved v_bw.
    DualWaterNonlinear,
    /// Archie (1942) clean-sand, total-porosity form. Post-solve; ignores the clay conductivity term.
    #[serde(alias = "archie")]
    ArchieTotal,
    /// Archie (1942) clean-sand, EFFECTIVE-porosity form (SB-SAT-002): Sw = (a*Rw/(Rt*phie^m))^(1/n)
    /// directly on phie, so bound water never enters the equation and no total->effective back-out
    /// exists. On the dossier reference case it sits 25.0 saturation units from `ArchieTotal` -
    /// the two are separately named because nothing else distinguishes them in an output.
    ArchieEffective,
    /// Poupon-Leveaux "Indonesia" (1971), effective-porosity form. Non-linear in Sw, post-solve.
    Indonesia,
    /// Simandoux / Bardon-Pied form without a `(1-Vsh)` divisor. Post-solve.
    SimandouxBardonPied,
    /// Modified Simandoux / Schlumberger form with a `(1-Vsh)` divisor. The legacy serialized
    /// `simandoux` id selected this equation, so it remains an input-only alias.
    #[serde(alias = "simandoux")]
    SimandouxModifiedSlb,
    /// Juhász (1981) normalized Waxman-Smits, wet-shale excess-conductivity form. Post-solve; the excess
    /// conductivity comes from the shale point (Rsh, φ_sh) rather than a temperature-form Cwb.
    Juhasz,
    /// Waxman-Smits (1968), total-porosity B·Qv form. Post-solve; the excess conductivity is B·Qv with
    /// Qv from the solved clay volumes (Qv = Σ v_clay·CEC·ρ / φt) and B from the Juhász B(T,Rw) fit
    /// (`waxman_b`) unless overridden by FluidProps.ws_b.
    WaxmanSmits,
    /// SB-SAT-026: RtC excess-conductivity (docs/method_lrlc_rtc_imts.md), Jauhar's own method,
    /// computed by `lrlc::sw_rtc` — NOT a SandiMin solver model. It lives in this registry only so
    /// its `SW_METHOD` flag code resolves through the one shared vocabulary.
    SwRtc,
    /// SB-SAT-026: iterative mineral-textural-scaled Waxman-Smits (docs/method_lrlc_rtc_imts.md),
    /// computed by `lrlc::sw_imts` — registry identity only, never solver-selectable.
    SwImts,
    /// SB-SAT-026: saturation-height function (Leverett-J / Skelt-Harrison, `satheight::sw_height`)
    /// — registry identity only, never solver-selectable.
    SwHeight,
}

impl SwModel {
    pub fn id(self) -> &'static str {
        match self {
            SwModel::LinearDw => "linear_dw",
            SwModel::DualWaterNonlinear => "dual_water_nonlinear",
            SwModel::ArchieTotal => "archie_total",
            SwModel::ArchieEffective => "archie_effective",
            SwModel::Indonesia => "indonesia",
            SwModel::SimandouxBardonPied => "simandoux_bardon_pied",
            SwModel::SimandouxModifiedSlb => "simandoux_modified_slb",
            SwModel::Juhasz => "juhasz",
            SwModel::WaxmanSmits => "waxman_smits",
            SwModel::SwRtc => "sw_rtc",
            SwModel::SwImts => "sw_imts",
            SwModel::SwHeight => "sw_height",
        }
    }

    /// SB-SAT-026: whether the SandiMin solver implements this model. The registry deliberately
    /// carries MORE identities than the solver — every saturation method's `SW_METHOD` flag
    /// resolves through one vocabulary — so the solver must refuse the ones it does not compute
    /// rather than let a deserialized request reach a post-solve branch that does not exist.
    pub fn solver_selectable(self) -> bool {
        !matches!(self, SwModel::SwRtc | SwModel::SwImts | SwModel::SwHeight)
    }

    /// Stable numeric encoding used only because computed curve samples are `f32`. These are
    /// categorical identifiers, not petrophysical parameters: callers must resolve them through
    /// [`sw_model_id_from_flag`] and must never perform arithmetic on them.
    pub fn flag_code(self) -> f32 {
        match self {
            SwModel::LinearDw => 1.0,
            SwModel::DualWaterNonlinear => 2.0,
            SwModel::ArchieTotal => 3.0,
            SwModel::Indonesia => 4.0,
            SwModel::SimandouxBardonPied => 5.0,
            SwModel::SimandouxModifiedSlb => 6.0,
            SwModel::Juhasz => 7.0,
            SwModel::WaxmanSmits => 8.0,
            SwModel::ArchieEffective => 9.0,
            SwModel::SwRtc => 10.0,
            SwModel::SwImts => 11.0,
            SwModel::SwHeight => 12.0,
        }
    }

    /// Every model except the default linearised dual-water replaces the in-inversion conductivity row
    /// with a post-solve Sw computed from the solved volumes.
    fn is_post_solve(self) -> bool {
        !matches!(self, SwModel::LinearDw)
    }
}

/// One user-facing saturation-model identity. The id is the value persisted in a new run; the
/// label deliberately leads with that id so a result and the selector can be matched without
/// translating a vendor adjective whose meaning changes between products.
#[derive(Debug, Clone, Serialize)]
pub struct SwModelChoice {
    pub id: &'static str,
    pub label: &'static str,
    /// Exact class code written to the per-sample `SW_METHOD` curve. It is an encoding whose
    /// meaning comes from `id`; it is not a saturation value and is declared as a class curve.
    pub flag_code: f32,
}

pub fn sw_model_catalog() -> Vec<SwModelChoice> {
    vec![
        SwModelChoice {
            id: SwModel::LinearDw.id(),
            label: "linear_dw — linearized dual-water",
            flag_code: SwModel::LinearDw.flag_code(),
        },
        SwModelChoice {
            id: SwModel::DualWaterNonlinear.id(),
            label: "dual_water_nonlinear — Clavier dual-water (m and n separate)",
            flag_code: SwModel::DualWaterNonlinear.flag_code(),
        },
        SwModelChoice {
            id: SwModel::ArchieTotal.id(),
            label: "archie_total — Archie on total porosity",
            flag_code: SwModel::ArchieTotal.flag_code(),
        },
        SwModelChoice {
            id: SwModel::ArchieEffective.id(),
            label: "archie_effective — Archie on effective porosity",
            flag_code: SwModel::ArchieEffective.flag_code(),
        },
        SwModelChoice {
            id: SwModel::Indonesia.id(),
            label: "indonesia — Poupon-Leveaux",
            flag_code: SwModel::Indonesia.flag_code(),
        },
        SwModelChoice {
            id: SwModel::SimandouxBardonPied.id(),
            label: "simandoux_bardon_pied — Simandoux / Bardon-Pied form",
            flag_code: SwModel::SimandouxBardonPied.flag_code(),
        },
        SwModelChoice {
            id: SwModel::SimandouxModifiedSlb.id(),
            label: "simandoux_modified_slb — Modified Simandoux / Schlumberger form",
            flag_code: SwModel::SimandouxModifiedSlb.flag_code(),
        },
        SwModelChoice {
            id: SwModel::Juhasz.id(),
            label: "juhasz — normalized Qv",
            flag_code: SwModel::Juhasz.flag_code(),
        },
        SwModelChoice {
            id: SwModel::WaxmanSmits.id(),
            label: "waxman_smits — B·Qv",
            flag_code: SwModel::WaxmanSmits.flag_code(),
        },
        SwModelChoice {
            id: SwModel::SwRtc.id(),
            label: "sw_rtc — RtC excess-conductivity (LRLC)",
            flag_code: SwModel::SwRtc.flag_code(),
        },
        SwModelChoice {
            id: SwModel::SwImts.id(),
            label: "sw_imts — iterative mineral-textural-scaled Waxman-Smits (LRLC)",
            flag_code: SwModel::SwImts.flag_code(),
        },
        SwModelChoice {
            id: SwModel::SwHeight.id(),
            label: "sw_height — saturation-height function",
            flag_code: SwModel::SwHeight.flag_code(),
        },
    ]
}

/// The models the SandiMin dialog may OFFER — the registry minus the identities other modules own.
/// The one wording of the SB-SAT-026 refusal, shared by the solver and its test probe so the
/// message the user reads is the message the proof pins.
fn solver_refusal(model: SwModel) -> String {
    format!("'{}' is another module's method identity, not a SandiMin solver model", model.id())
}

/// Test seam: the refusal `run_sandimin` issues for a registry-only identity, without a database.
#[cfg(test)]
pub(crate) fn run_sandimin_selectability_probe(model: SwModel) -> String {
    if model.solver_selectable() {
        return String::new();
    }
    solver_refusal(model)
}

pub fn solver_selectable_models() -> Vec<SwModelChoice> {
    sw_model_catalog()
        .into_iter()
        .filter(|choice| {
            !matches!(choice.id, "sw_rtc" | "sw_imts" | "sw_height")
        })
        .collect()
}

/// Resolve a stored method-flag sample to the canonical equation identifier. Exact equality is
/// intentional: writers emit the small integer class codes above, so a fractional value is corrupt
/// categorical data rather than something to round to the nearest method.
pub fn sw_model_id_from_flag(flag: f32) -> Option<&'static str> {
    sw_model_catalog().into_iter().find(|entry| entry.flag_code == flag).map(|entry| entry.id)
}

/// Build the per-sample method-flag curve used by SandiMin. A missing saturation has no producing
/// result to identify and therefore carries `f32::NAN`, matching the product-wide missing-data
/// contract; every produced sample carries the exact categorical code for `model`.
pub(crate) fn saturation_method_flag_curve(
    prefix: &str,
    model: SwModel,
    produced: &[bool],
) -> (String, Vec<f32>) {
    let name = if prefix.is_empty() { "SW_METHOD".to_string() } else { format!("{prefix}_SW_METHOD") };
    let code = model.flag_code();
    debug_assert_eq!(sw_model_id_from_flag(code), Some(model.id()));
    let values = produced.iter().map(|present| if *present { code } else { f32::NAN }).collect();
    (name, values)
}

/// What drives the clay bound-water (BNDWAT) constraint (Jauhar field review, image 2 "Porosity Source").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PorositySource {
    /// Cation exchange capacity: v_bw = α·96·CEC·ρ/(T+298)·v_dryclay. Default — nothing moves.
    #[default]
    Cec,
    /// Wet-clay porosity: v_bw = φ_clay/(1−φ_clay)·v_dryclay (geometric; no CEC/T/α). Moves PHIE
    /// relative to the CEC route, so it's opt-in.
    WetClayPorosity,
}

/// Poupon-Leveaux (`indonesia`) water saturation, effective-porosity form, solved for Sw∈[0,1]:
///   1/√Rt = [ Vsh^(1 − k·Vsh/2)/√Rsh + √(φe^m / (a·Rw)) ] · Sw^(n/2)
/// Rw and Rsh are at formation temperature. Returns NaN on non-physical inputs.
///
/// This is [`sw_indonesia_unlimited`] clamped into [0, 1] - the same relationship the Simandoux
/// pair has, so the working curve and its unlimited diagnostic can never come from two different
/// equations.
pub fn sw_indonesia(
    rt: f64,
    phie: f64,
    vsh: f64,
    rw: f64,
    rsh: f64,
    m: f64,
    n: f64,
    a: f64,
    k: f64,
) -> f64 {
    sw_indonesia_unlimited(rt, phie, vsh, rw, rsh, m, n, a, k).clamp(0.0, 1.0)
}

/// `indonesia` WITHOUT the physical clamp - the raw answer the equation gives for the parameters
/// it was handed, which is what an unlimited diagnostic curve is for (DEC-085's "diagnostics stay
/// raw"; SB-SAT-025). An Rw entered a decade low drives the true root above 1, and a diagnostic
/// that reports 1.000 there is bit-identical to the working curve beside it.
///
/// AUDIT-2026-08-20 finding 51: `modules::sw_indo` used to carry its own EXPANDED transcription
/// of this equation. The two are algebraically identical - the module's `v` is Vsh^(2-k·Vsh), and
/// (f1 + f2 + f3) expands the square of this function's denominator term for term - but the
/// GUARDS diverged, and a pin that keeps two implementations agreeing is the tell that there
/// should be one. `sw_sim` next door had already delegated; this is the family member it skipped.
#[allow(clippy::too_many_arguments)]
pub fn sw_indonesia_unlimited(
    rt: f64,
    phie: f64,
    vsh: f64,
    rw: f64,
    rsh: f64,
    m: f64,
    n: f64,
    a: f64,
    k: f64,
) -> f64 {
    if !(rt > 0.0) || !(phie > 0.0) || !(rw > 0.0) || !(n > 0.0) || !k.is_finite() {
        return f64::NAN;
    }
    let vsh = vsh.clamp(0.0, 1.0);
    let a = a.max(1e-9);
    let term_sh = if rsh > 0.0 { vsh.powf(1.0 - k * vsh / 2.0) / rsh.sqrt() } else { 0.0 };
    let term_sand = (phie.powf(m) / (a * rw)).sqrt();
    let denom = term_sh + term_sand;
    if !(denom > 0.0) {
        return f64::NAN;
    }
    let sw_half = (1.0 / rt).sqrt() / denom; // = Sw^(n/2)
    sw_half.powf(2.0 / n)
}

/// How far above Sw = 1 the unlimited root is chased before the diagnostic saturates. Each step
/// doubles, so 40 reaches ~1.1e12 — far past any reading anyone would quote, while keeping the
/// search bounded for a caller that hands in a degenerate coefficient.
const SIM_ROOT_MAX_DOUBLINGS: usize = 40;

/// The Simandoux family root WITHOUT the physical clamp — the raw answer the equation gives for
/// the parameters it was handed, which is what an unlimited diagnostic curve is for (DEC-085's
/// "diagnostics stay raw"; SB-SAT-025's rule for the saturation-height pair). An Rw entered a
/// decade low drives the true root above 1, and a diagnostic that reports 1.000 there is
/// bit-identical to the working curve beside it — so the pair agrees, the wet leg looks real,
/// and the one ambiguity the unlimited twin exists to break survives (AUDIT-2026-08-20 finding 4).
///
/// This is the ONLY Simandoux root solver: the clipped public entry points
/// ([`sw_simandoux_bardon_pied`], [`sw_simandoux_modified_slb`]) are their unlimited twins
/// clamped into [0, 1], so the working curve and the diagnostic can never come from two
/// different equations.
fn solve_simandoux_root_unlimited(ct: f64, coef_sand: f64, coef_sh: f64, n: f64) -> f64 {
    if !(coef_sand > 0.0) {
        // Degenerate: no sand term — the shale term alone gives Sw (or NaN if no shale either).
        return if coef_sh > 0.0 { ct / coef_sh } else { f64::NAN };
    }
    if (n - 2.0).abs() < 1e-9 {
        // coef_sand·Sw² + coef_sh·Sw − ct = 0 — closed form, so the raw root needs no search.
        let disc = coef_sh * coef_sh + 4.0 * coef_sand * ct;
        if disc < 0.0 {
            return f64::NAN;
        }
        return (-coef_sh + disc.sqrt()) / (2.0 * coef_sand);
    }
    // General n: f(Sw) = coef_sand·Sw^n + coef_sh·Sw − ct is increasing on [0, ∞) for
    // coef_sand > 0, coef_sh ≥ 0, n > 0; f(0) = −ct < 0, so the root is positive and unique.
    let f = |sw: f64| coef_sand * sw.powf(n) + coef_sh * sw - ct;
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    if f(hi) <= 0.0 {
        // The root is at or above 1 — the case the old code answered with a flat 1.0. Widen the
        // bracket instead of truncating the answer.
        let mut bracketed = false;
        for _ in 0..SIM_ROOT_MAX_DOUBLINGS {
            lo = hi;
            hi *= 2.0;
            if f(hi) > 0.0 {
                bracketed = true;
                break;
            }
        }
        if !bracketed {
            // Nothing physical reaches here (Sw^n diverges), but a caller's degenerate
            // coefficients could: report the saturation point rather than looping or lying.
            return hi;
        }
    }
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if f(mid) > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    0.5 * (lo + hi)
}


/// `simandoux_bardon_pied`, effective-porosity form, solved for Sw∈[0,1]:
///   1/Rt = φe^m·Sw^n / (a·Rw) + Vsh·Sw / Rsh
/// Geolog calls this `MODIFIED`; IP calls it plain `Simandoux`. The equation id above is the
/// SandiBumi identity and neither vendor adjective is persisted as the method name.
pub fn sw_simandoux_bardon_pied(
    rt: f64,
    phie: f64,
    vsh: f64,
    rw: f64,
    rsh: f64,
    m: f64,
    n: f64,
    a: f64,
) -> f64 {
    sw_simandoux_bardon_pied_unlimited(rt, phie, vsh, rw, rsh, m, n, a).clamp(0.0, 1.0)
}

/// `simandoux_bardon_pied` WITHOUT the physical clamp — the unlimited diagnostic companion of
/// [`sw_simandoux_bardon_pied`], which is this value clamped into [0, 1]. See
/// [`solve_simandoux_root_unlimited`] for why the raw reading is the one worth keeping.
pub fn sw_simandoux_bardon_pied_unlimited(
    rt: f64,
    phie: f64,
    vsh: f64,
    rw: f64,
    rsh: f64,
    m: f64,
    n: f64,
    a: f64,
) -> f64 {
    if !(rt > 0.0) || !(phie > 0.0) || !(rw > 0.0) || !(n > 0.0) {
        return f64::NAN;
    }
    let vsh = vsh.clamp(0.0, 1.0);
    let coef_sand = phie.powf(m) / (a.max(1e-9) * rw);
    let coef_sh = if rsh > 0.0 { vsh / rsh } else { 0.0 };
    solve_simandoux_root_unlimited(1.0 / rt, coef_sand, coef_sh, n)
}

/// `simandoux_modified_slb`, effective-porosity form, solved for Sw∈[0,1]:
///   1/Rt = φe^m·Sw^n / (a·Rw·(1 − Vsh)) + Vsh^C·Sw / Rsh
/// IP and Techlog call this `Modified Simandoux`; Geolog calls it `SCHLUM`. `C=1` reproduces IP E64.
pub fn sw_simandoux_modified_slb(
    rt: f64,
    phie: f64,
    vsh: f64,
    rw: f64,
    rsh: f64,
    m: f64,
    n: f64,
    a: f64,
    c: f64,
) -> f64 {
    sw_simandoux_modified_slb_unlimited(rt, phie, vsh, rw, rsh, m, n, a, c).clamp(0.0, 1.0)
}

/// `simandoux_modified_slb` WITHOUT the physical clamp — the unlimited diagnostic companion of
/// [`sw_simandoux_modified_slb`], which is this value clamped into [0, 1].
///
/// The `VSH >= 1` answer stays 1.0 in BOTH renderings, and that is not the clamp this function
/// removes: it is the declared singularity convention (the 1/(1−Vsh) term has no value there),
/// which `sw_sim` reports by name through SB-SAT-030 rather than passing off as a computed
/// saturation. A raw reading requires an equation to have been evaluated.
#[allow(clippy::too_many_arguments)]
pub fn sw_simandoux_modified_slb_unlimited(
    rt: f64,
    phie: f64,
    vsh: f64,
    rw: f64,
    rsh: f64,
    m: f64,
    n: f64,
    a: f64,
    c: f64,
) -> f64 {
    if !(rt > 0.0) || !(phie > 0.0) || !(rw > 0.0) || !(n > 0.0) || !(c > 0.0) {
        return f64::NAN;
    }
    if vsh >= 1.0 {
        return 1.0;
    }
    let vsh = vsh.clamp(0.0, 1.0);
    let coef_sand = phie.powf(m) / (a.max(1e-9) * rw * (1.0 - vsh));
    let coef_sh = if rsh > 0.0 { vsh.powf(c) / rsh } else { 0.0 };
    solve_simandoux_root_unlimited(1.0 / rt, coef_sand, coef_sh, n)
}

/// Clavier-Coates-Dumanoir dual-water TOTAL water saturation (SPEJ 1984), exact form honouring m and
/// n separately (unlike the linearised in-inversion row, which folds them into w = 0.75m+0.25n):
///   Ct = (φt^m · Swt^n / a) · [ Cw + (Cwb − Cw)·Swb/Swt ]
/// rearranged to the increasing-in-Swt root equation (Geolog sw_dual CALC_SW):
///   Cw·Swt^n + Swb·(Cwb − Cw)·Swt^(n−1) − a·Ct/φt^m = 0,   Ct = 1/Rt.
/// `swb` is the bound-water saturation (v_bw/φt) taken from the solved bound-water volume — so no Qv
/// is needed. Conductivities are mho/m at formation temperature. Returns SWT∈[0,1] (bisection; the
/// n==2 case closes to a quadratic). NaN on non-physical inputs.
pub fn sw_dual_nonlinear(rt: f64, phit: f64, swb: f64, cw: f64, cwb: f64, m: f64, n: f64, a: f64) -> f64 {
    if !(rt > 0.0) {
        return f64::NAN;
    }
    // Dual water's excess-conductivity coefficient is Swb·(Cwb−Cw); the rest of the algebra is shared
    // with Juhász (differing only in that coefficient), so both route through sw_cond_root.
    let swb = swb.clamp(0.0, 1.0);
    sw_cond_root(phit, 1.0 / rt, cw, swb * (cwb - cw), m, n, a)
}

/// Core conductivity-root solver shared by the excess-conductivity Sw models (dual-water non-linear and
/// Juhász). Solves  `cw·Swt^n + lin·Swt^(n−1) − a·Ct/φt^m = 0`  for the physical SWT∈[0,1], where `lin`
/// is the model's excess-conductivity coefficient — dual water: `Swb·(Cwb−Cw)`; Juhász: `QVN·(Cwsh−Cw)`.
/// Conductivities are mho/m. `n ≥ 1` is required: a sub-linear exponent makes the `Swt^(n−1)` term
/// diverge at Swt→0, blowing up g(0) and collapsing the bisection/short-circuit logic (which assumes g
/// rises from g(0)=−rhs<0) to SWT=0 regardless of Rt — so it's rejected as NaN, and the post-solve
/// caller's is_finite() check then leaves the linear inversion's split untouched rather than zeroing it.
/// At exactly `n = 1` the same term stops vanishing (0^0 = 1), and where the excess-conductivity
/// coefficient alone exceeds the measured `a·Ct/φt^m` there is no root in [0, 1]: that returns NaN
/// too, rather than the SWT = 0 - all hydrocarbon - the algebra would otherwise hand out.
fn sw_cond_root(phit: f64, ct: f64, cw: f64, lin: f64, m: f64, n: f64, a: f64) -> f64 {
    if !(ct > 0.0) || !(phit > 0.0) || !(cw > 0.0) || !(n >= 1.0) {
        return f64::NAN;
    }
    let a = a.max(1e-9);
    let rhs = a * ct / phit.powf(m); // the constant term a·Ct/φt^m (> 0)
    if (n - 2.0).abs() < 1e-9 {
        // cw·Swt² + lin·Swt − rhs = 0. disc = lin² + 4·cw·rhs is always ≥ 0 (cw>0, rhs>0), so the
        // positive root exists; cw>0 makes it the physical branch.
        let disc = lin * lin + 4.0 * cw * rhs;
        return ((-lin + disc.sqrt()) / (2.0 * cw)).clamp(0.0, 1.0);
    }
    // General n: g(Swt) = cw·Swt^n + lin·Swt^(n−1) − rhs. g(0)=−rhs<0; if g(1)≤0 the rock is at/above
    // Swt=1. Between, g is continuous — bisect (cw>0 keeps the high-Swt branch increasing).
    let g = |swt: f64| cw * swt.powf(n) + lin * swt.powf(n - 1.0) - rhs;
    // g(1) <= 0 is the ordinary WET-ZONE clamp every saturation model applies: the rock measures
    // at least the conductivity fully-water-saturated rock would have, so Swt = 1. Archie's own
    // .clamp(0.0, 1.0) does exactly this, and the unlimited diagnostic twins exist for anyone who
    // needs the raw root. Left alone deliberately - it is a value off the end of the scale, not a
    // degenerate equation.
    if g(1.0) <= 0.0 {
        return 1.0;
    }
    // AUDIT-2026-08-20 finding 11. g(0) > 0 is a different animal, and only reachable at EXACTLY
    // n = 1: above it, Swt^(n-1) -> 0 kills the excess-conductivity offset so g(0) = -rhs < 0
    // always. At n = 1, Rust's 0^0 = 1 leaves that offset standing, and g(0) = lin - rhs > 0 says
    // the CLAY TERM ALONE conducts more than the rock actually measures. There is then no root in
    // [0, 1] at all - and the answer this used to return was SWT = 0.0, a hundred per cent
    // hydrocarbon, written as an ordinary curve.
    //
    // That is the optimistic extreme handed out precisely where the model has broken down, and it
    // REWARDS an over-estimated Qv/Cwb/Swb with more pay. What the condition actually evidences is
    // a clay term set too high, or an Rt too high for the bed - not a hydrocarbon leg. SB-SAT-030
    // rules the shape ("MUST NOT return Sw = 1 unflagged... returning a plausible number from a
    // singular equation is exactly the fail-silent pattern"); this is its mirror at the dry end,
    // and SB-SAT-028 rules that a solver with no answer returns null.
    //
    // MISSING rather than a flag because this is a pure function with no run context to record
    // into - and it is the refusal this function's own doc already reasons for at n < 1, where the
    // same Swt^(n-1) term collapses the solve to SWT = 0 regardless of Rt. Same failure, same
    // answer. Every caller already tests is_finite(), so the linear inversion's split is left
    // untouched rather than zeroed, and resultsqc plots the sample as the gap it is.
    if g(0.0) > 0.0 {
        return f64::NAN;
    }
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if g(mid) > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    (0.5 * (lo + hi)).clamp(0.0, 1.0)
}

/// Archie (1942) clean-sand TOTAL water saturation — no shale term:
///   Swt = ( a·Rw / (φt^m · Rt) )^(1/n)
/// The exactly-invertible base case (so there is no separate "linear/non-linear" Archie). Rw at
/// formation temperature. Returns SWT∈[0,1]; NaN on non-physical inputs.
pub fn sw_archie(rt: f64, phit: f64, rw: f64, m: f64, n: f64, a: f64) -> f64 {
    if !(rt > 0.0) || !(phit > 0.0) || !(rw > 0.0) || !(n > 0.0) {
        return f64::NAN;
    }
    let a = a.max(1e-9);
    ((a * rw) / (phit.powf(m) * rt)).powf(1.0 / n).clamp(0.0, 1.0)
}

/// SB-SAT-023: the effective back-out, `SWE = MAX((SWT − Swb)/(1 − Swb), 0)`. `Swb = 1` is all
/// bound water and yields SWE = 1, never a division by zero (IP E78; Geolog `sw_ws.lls:296` is the
/// algebraically identical form).
pub fn swe_from_swt(swt: f64, swb: f64) -> f64 {
    if swb >= 1.0 {
        return 1.0;
    }
    ((swt - swb) / (1.0 - swb)).max(0.0)
}

/// SB-SAT-023: the inverse lift, `SwT = Sw(1 − Swb) + Swb` — and `SxoT = Sxo(1 − Swb) + Swb` is
/// the SAME published formula applied to Sxo, deliberately one implementation. A round trip
/// through the pair is the identity for Swb < 1; at Swb = 1 the map is non-invertible because
/// everything is bound water, and both directions return 1.
pub fn swt_from_swe(swe: f64, swb: f64) -> f64 {
    if swb >= 1.0 {
        return 1.0;
    }
    swe * (1.0 - swb) + swb
}

/// SB-SAT-023: Juhász's own bound-water term — `Qvn = clamp(Vsh·φt_sh/φt, 0, 1)` from the shale
/// point (Geolog `sw_juha.lls:262`; Techlog agrees) — NOT `1 − φe/φt`, and on the dossier fixture
/// the two differ by tens of saturation units.
pub fn juhasz_qvn(vsh: f64, phit_sh: f64, phit: f64) -> f64 {
    if !(phit > 0.0) {
        return f64::NAN;
    }
    (vsh * phit_sh / phit).clamp(0.0, 1.0)
}

/// SB-SAT-023: which `Swb` rule a model's effective back-out uses. Recorded on the run result —
/// the solver's construction makes φt ≡ φe + v_bw, which COLLAPSES `1 − φe/φt` with `v_bw/φt`,
/// so the name is the only place the distinction survives.
pub fn effective_backout_rule(model: SwModel) -> &'static str {
    match model {
        SwModel::Juhasz => "juhasz_qvn",
        _ => "porosity_volume_1_minus_phie_over_phit",
    }
}

/// SB-SAT-023: the per-model effective back-out itself. Returns `(SWE, rule name)`.
pub fn effective_backout(
    model: SwModel,
    swt: f64,
    phie: f64,
    phit: f64,
    vsh: f64,
    phit_sh: f64,
) -> (f64, &'static str) {
    let rule = effective_backout_rule(model);
    let swb = match model {
        SwModel::Juhasz => juhasz_qvn(vsh, phit_sh, phit),
        _ => {
            if !(phit > 0.0) {
                return (f64::NAN, rule);
            }
            1.0 - (phie / phit).clamp(0.0, 1.0)
        }
    };
    if !swb.is_finite() {
        return (f64::NAN, rule);
    }
    (swe_from_swt(swt, swb), rule)
}

/// Juhász (1981) "normalized Waxman-Smits" TOTAL water saturation — the wet-shale excess-conductivity
/// form Jauhar groups with the "use wet parameters straight away" methods. Instead of dual water's
/// temperature-form Cwb, the clay excess conductivity is read straight from the SHALE point:
///   Cwsh = 1/(Rsh·φ_sh^m),  QVN = Vsh·φ_sh/φt  (normalized Qv),
/// and SWT solves  Cw·Swt^n + QVN·(Cwsh−Cw)·Swt^(n−1) = Ct/φt^m  (a = 1). `Rsh` is the shale
/// resistivity at formation temperature and `phit_sh` the wet-clay (shale) total porosity. With Vsh=0
/// (QVN=0) it collapses to clean-sand Archie. Returns SWT∈[0,1]; NaN on non-physical inputs.
pub fn sw_juhasz(
    rt: f64,
    phit: f64,
    vsh: f64,
    cw: f64,
    rsh: f64,
    phit_sh: f64,
    m: f64,
    n: f64,
) -> f64 {
    if !(rt > 0.0) || !(rsh > 0.0) || !(phit_sh > 0.0) || !(phit > 0.0) {
        return f64::NAN;
    }
    let qvn = (vsh * phit_sh / phit).clamp(0.0, 1.0);
    let cwsh = 1.0 / (rsh * phit_sh.powf(m)); // 100%-shale water conductivity from the shale point
    sw_cond_root(phit, 1.0 / rt, cw, qvn * (cwsh - cw), m, n, 1.0)
}

/// Waxman-Smits (1968) water saturation, total-porosity basis. The conductivity model is
///   Ct = (φt^m / a) · (Cw·Swt^n + B·Qv·Swt^(n−1)),   a = 1,
/// i.e. free-water conduction plus an excess clay-counterion term B·Qv that does NOT scale with Swt^n.
/// Rearranged this is exactly the shared root `cw·Swt^n + (B·Qv)·Swt^(n−1) − Ct/φt^m = 0`, so it reuses
/// `sw_cond_root` with `lin = B·Qv`. `qv` is the cation concentration in meq/mL of pore water and `b` the
/// equivalent counterion conductance (mho·mL/(m·meq)); see `waxman_b`. A clean sand (Qv = 0) collapses to
/// Archie. Exponents are the Waxman-Smits m*/n* (passed in as `m`,`n`).
pub fn sw_waxman_smits(rt: f64, phit: f64, qv: f64, cw: f64, b: f64, m: f64, n: f64) -> f64 {
    if !(rt > 0.0) || !(phit > 0.0) || !(cw > 0.0) {
        return f64::NAN;
    }
    let lin = (b * qv).max(0.0); // the excess counterion conductance is non-negative
    sw_cond_root(phit, 1.0 / rt, cw, lin, m, n, 1.0)
}

/// Waxman-Smits counterion conductance B(T, Rw) — Juhász's (1981) closed-form fit of the
/// Waxman-Thomas B chart (Techlog "1972 Waxman B chart original fit" / IP2025 PhiSw, verified against
/// both installs' docs):
///   B = (−1.28 + 0.225·T − 0.0004059·T²) / (1 + (0.045·T − 0.27)·Rw^1.23),
/// with `t_c` the formation temperature in °C and `rw` the formation-water resistivity in ohm·m at
/// that temperature. The result is in mho·mL/(m·meq): paired with Qv in meq/mL and Cw in mho/m it
/// makes the excess term B·Qv come out in mho/m. Clamped ≥ 0 (the numerator dips negative below ~6 °C).
/// This is the *auto* B; a core-measured value can override it (FluidProps.ws_b) because the fit is
/// known to overshoot above ~120 °C. A non-positive Rw (or a degenerate denominator, only reachable at
/// near-freezing formation temperature with ultra-fresh water — outside any real reservoir) falls back
/// to the salinity-independent numerator.
pub fn waxman_b(t_c: f64, rw: f64) -> f64 {
    let num = -1.28 + 0.225 * t_c - 0.0004059 * t_c * t_c;
    if !(rw > 0.0) {
        return num.max(0.0);
    }
    let den = 1.0 + (0.045 * t_c - 0.27) * rw.powf(1.23);
    if !(den > 0.0) {
        return num.max(0.0);
    }
    (num / den).max(0.0)
}

/// Fluid / saturation parameters (needed when CT or CXO participates).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidProps {
    /// Formation water resistivity sample (ohmm) + its temperature (°F).
    pub rw: f64,
    pub rw_temp_f: f64,
    /// Mud filtrate resistivity sample (ohmm) + its temperature (°F).
    pub rmf: f64,
    pub rmf_temp_f: f64,
    /// Formation temperature (°F) at the interval of interest.
    pub ftemp_f: f64,
    /// Archie/dual-water exponents; w = 0.75·m + 0.25·n.
    pub m: f64,
    pub n: f64,
    /// "WATER" | "OIL"
    #[serde(default = "default_mud")]
    pub mud_type: String,
    /// Shale resistivity Rsh (ohmm) at formation temperature — the 100%-shale Rt used by the
    /// shaly-sand Sw models (Indonesia/Simandoux). Ignored by the dual-water models. Default 4.0.
    #[serde(default = "default_rsh")]
    pub rsh: f64,
    /// Archie tortuosity factor a (Indonesia/Simandoux). The dual-water models use a = 1 by
    /// construction, but they are not the only callers of this field, so it carries NO default.
    // SB-SAT-034: `a` ships NoDefault. IP publishes no default for a/m/n at all - the
    // 1.0/2.0/2.0 commonly quoted are Basic Log Analysis values only - and a cementation
    // exponent is a rock property measured on core. A shipped exponent is the
    // highest-consequence silent default in petrophysics, so this field is REQUIRED and
    // deserialization refuses without it, naming the parameter, exactly as `rw` already does.
    pub archie_a: f64,
    /// Indonesia shale exponent coefficient in `Vsh^(2-k·Vsh)`. Geolog's FULL/SIMPLE/TAR_SAND
    /// presets are k=1/0/2; k=1 is the cited FULL default.
    #[serde(default = "default_indonesia_k")]
    pub indonesia_k: f64,
    /// `simandoux_modified_slb` shale exponent C. Geolog validates 1..2 and ships C=1, which
    /// reproduces IP E64. Ignored by `simandoux_bardon_pied`, whose shale term is linear Vsh.
    #[serde(default = "default_simandoux_c")]
    pub simandoux_c: f64,
    /// Wet-clay (shale) total porosity φ_sh — the "wet clay porosity" used by Juhász's normalized Qv
    /// (QVN = Vsh·φ_sh/φt) and shale-point conductivity (Cwsh = 1/(Rsh·φ_sh^m)). Only Juhász reads it.
    /// Default 0.10.
    #[serde(default = "default_phit_sh")]
    pub phit_sh: f64,
    /// Optional core-measured Waxman-Smits B override, mho·mL/(m·meq). 0 (default) ⇒ compute B(T,Rw)
    /// from `waxman_b`. Only the Waxman-Smits model reads it — an escape hatch for when the Juhász
    /// B(T) fit disagrees with a measured B (it overshoots above ~120 °C).
    #[serde(default)]
    pub ws_b: f64,
}

fn default_mud() -> String {
    "WATER".into()
}
fn default_rsh() -> f64 {
    4.0
}
fn default_indonesia_k() -> f64 {
    1.0
}
fn default_simandoux_c() -> f64 {
    1.0
}
fn default_phit_sh() -> f64 {
    0.10
}

/// Derived fluid quantities (also exposed to the dialog via `sandimin_fluid_calc`).
#[derive(Debug, Clone, Serialize)]
pub struct FluidCalc {
    pub w: f64,
    /// Formation-water / mud-filtrate conductivity at formation temperature (mho/m).
    pub cw: f64,
    pub cmf: f64,
    /// Clay-bound-water conductivity (mho/m) and its per-zone α-reduced values.
    pub cbw: f64,
    pub cbw_x: f64,
    pub cbw_u: f64,
    pub alpha_x: f64,
    pub alpha_u: f64,
    pub salinity_w_ppm: f64,
    pub salinity_mf_ppm: f64,
    /// Auto uncertainties for the transformed CT/CXO rows.
    pub u_ct: f64,
    pub u_cxo: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SandiminRequest {
    pub components: Vec<Component>,
    pub tools: Vec<ToolSpec>,
    pub apply_well_ids: Vec<String>,
    #[serde(default = "default_prefix")]
    pub output_prefix: String,
    /// Read every tool curve from THIS log set's stored values (latest version per well) rather
    /// than from whatever the current values are. Curves the set never wrote fall back to normal
    /// resolution, and an empty name means "current values" — the behaviour before this existed.
    ///
    /// Jauhar, 2026-08-05. A mineral inversion is only as reproducible as the logs it was solved
    /// from: re-run the porosity and the same components, endpoints and constraints return a
    /// different mineralogy, with nothing in the stored run able to say which RHOB it saw.
    #[serde(default)]
    pub input_set: Option<String>,
    /// Version the solved volumes into this log set. Defaults to `SANDIMIN` — the value that was
    /// hardcoded — so an older payload writes exactly where it always did.
    #[serde(default)]
    pub output_set: Option<String>,
    pub custody: crate::equations::RunCustody,
    #[serde(default = "default_true")]
    pub unity: bool,
    /// Required when CT or CXO is among the tools.
    #[serde(default)]
    pub fluid: Option<FluidProps>,
    /// Optional per-depth formation-temperature curve name (°F). When set and in the sane range at a
    /// depth (`FTEMP_MIN_F..FTEMP_MAX_F`), the temperature-dependent fluid quantities (Cw, Cmf, Cbw, the
    /// auto CT/CXO σ, the BNDWAT multiplier and the Waxman-Smits B) are recomputed for that sample
    /// instead of using `fluid.ftemp_f`. A missing curve or an out-of-range/non-finite sample (a null
    /// sentinel like ±999.25) falls back to the fixed temperature.
    #[serde(default)]
    pub ftemp_curve: Option<String>,
    /// Emit per-tool reconstruction-QC curves: for each active tool a `<prefix>_<KEY>_REC`
    /// (measurement rebuilt from the solved volumes, in the tool's display units) and
    /// `<prefix>_<KEY>_DIF` (the σ-unit residual = that tool's term of RECON). Off by default
    /// to keep the curve set lean.
    #[serde(default)]
    pub recon_qc: bool,
    /// Saturation model for the conductivity tools. Default `linear_dw` = the current in-inversion
    /// linearised dual-water (nothing moves). `indonesia`/`simandoux` are post-solve shaly-sand forms.
    #[serde(default)]
    pub sw_model: SwModel,
    /// What drives the clay bound-water constraint: `cec` (default) or `wet_clay_porosity`.
    #[serde(default)]
    pub porosity_source: PorositySource,
    /// Soft-constraint enable flags (Jauhar field review, image 2 "Constraints"). All default to on,
    /// so an absent block leaves the solver byte-identical to before. POROSITY ties flushed/virgin
    /// porosity (Σ X fluids = Σ U fluids); BNDWAT ties clay bound water to clay volume; WATER MUD keeps
    /// flushed-zone water ≥ virgin water for water-based mud (invasion ⇒ Sxo ≥ Sw). UNITY has its own
    /// `unity` flag above.
    #[serde(default = "default_true")]
    pub enforce_porosity: bool,
    #[serde(default = "default_true")]
    pub enforce_bndwat: bool,
    #[serde(default = "default_true")]
    pub enforce_water_mud: bool,
    /// Soft-constraint tolerance σ (the row weight is 1/σ). Default 0.01 — the reviewed nominal.
    /// A non-positive value falls back to the default so a stray 0 can't blow up the weight.
    #[serde(default = "default_sigma_constraint")]
    pub sigma_constraint: f64,
}

fn default_sigma_constraint() -> f64 {
    SIGMA_CONSTRAINT
}

fn default_prefix() -> String {
    // "SM" (SandiMin) since the DEC-082-era rename; earlier projects carry MM_* curves
    // from the old default and keep them - a re-run under the new default writes SM_*
    // beside them rather than silently replacing an MM_* interpretation.
    "SM".into()
}
fn default_true() -> bool {
    true
}

/// Agreement between a solved SandiMin output and a routine-core-analysis measurement, over the
/// plugs that tied to a solved sample. `bias` is the mean signed (model − core), so its sign says
/// which way the model reads. Only ever present when at least one plug matched — an absent fit is
/// reported as such rather than as a zero, which would read as a perfect match.
#[derive(Debug, Clone, Serialize)]
pub struct CoreFit {
    pub n: usize,
    pub rms: f32,
    pub bias: f32,
}

#[derive(Debug, Serialize)]
pub struct SandiminWellResult {
    pub well_id: String,
    pub rows_solved: usize,
    pub mean_recon: f32,
    /// Core calibration — RECON says the model reproduces its own input LOGS; these say whether it
    /// reproduces an INDEPENDENT measurement. Core φ is reported against both PHIE and PHIT because
    /// which one a plug should match depends on the drying protocol (oven-dried drives off clay-bound
    /// water → PHIT; humidity-dried retains some → nearer PHIE), so the analyst reads the bracket
    /// rather than being handed one interpretation.
    pub core_phie: Option<CoreFit>,
    pub core_phit: Option<CoreFit>,
    /// Solved grain density vs core ρg — a check on the MINERAL model specifically, and independent
    /// of RHOB when RHOB was not itself an input tool.
    pub core_gd: Option<CoreFit>,
    /// Why the core calibration was not attempted, when it was not. Deliberately not `error`: the
    /// well SOLVED, and only the independent check against core was withheld. Three blank fits
    /// mean "this well has no core" without it, and a cross-datum delivery is the opposite —
    /// core exists and cannot be compared. (`dof_note`'s shape, one level down.)
    pub core_note: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SandiminResult {
    /// SB-SAT-023: which effective back-out Swb rule this run's model applies — recorded because
    /// the solver's construction collapses the first group's two algebraic spellings.
    pub swb_rule: Option<String>,
    pub outputs: Vec<String>,
    pub wells: Vec<SandiminWellResult>,
    /// Model degrees of freedom = (tools + soft constraints + unity) − components. 0 = exactly
    /// determined (residuals are forced to ~0 and can't validate the model); >0 = over-determined,
    /// so RECON/incoherence is a real fit-quality signal.
    pub dof: i64,
    /// Set when `dof == 0` — a heads-up that the reconstruction can't discriminate the model.
    pub dof_note: Option<String>,
    pub error: Option<String>,
}

fn fail(msg: &str) -> SandiminResult {
    SandiminResult { swb_rule: None, outputs: vec![], wells: vec![], dof: 0, dof_note: None, error: Some(msg.to_string()) }
}

/// Curve-safe token for a component name: uppercase, non-alphanumeric → '_'.
fn curve_token(name: &str) -> String {
    let t: String = name
        .trim()
        .to_uppercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    t.trim_matches('_').to_string()
}

// ---------------------------------------------------------------------------
// Fluid property calculations (reference fluid formulas / IP)
// ---------------------------------------------------------------------------

/// Arps temperature conversion of a resistivity (°F form). Shared with the
/// precalc Prep module in modules.rs.
pub(crate) fn arps_f(r: f64, from_f: f64, to_f: f64) -> f64 {
    r * (from_f + 6.77) / (to_f + 6.77)
}

/// Equivalent NaCl salinity (ppm) from water resistivity at 75 °F (Bateman-Konen inverse).
fn salinity_ppm(r_sample: f64, t_sample_f: f64) -> f64 {
    let r75 = arps_f(r_sample, t_sample_f, 75.0);
    if r75 <= 0.0124 {
        return 400_000.0; // saturated brine guard
    }
    10f64.powf((3.562 - (r75 - 0.0123).log10()) / 0.955)
}

/// Dual-water diffuse-layer expansion factor: α = sqrt(20455 / S) below 20,455 ppm NaCl.
fn alpha_expansion(salinity: f64) -> f64 {
    if salinity > 0.0 && salinity < 20_455.0 {
        (20_455.0 / salinity).sqrt().min(5.0)
    } else {
        1.0
    }
}

pub fn fluid_calc(p: &FluidProps) -> FluidCalc {
    fluid_calc_at(p, p.ftemp_f)
}

/// Fluid quantities at an explicit formation temperature `ftemp_f` (°F). This is `fluid_calc` with the
/// temperature supplied by the caller — when an FTEMP curve drives temperature the T-dependent parts
/// (cw, cmf, cbw and the auto CT/CXO uncertainties) are recomputed per depth. The salinities and the α
/// expansion come from the Rw/Rmf *sample* temperatures, so they do NOT vary with formation temperature
/// and stay identical to `fluid_calc(p)`.
pub fn fluid_calc_at(p: &FluidProps, ftemp_f: f64) -> FluidCalc {
    let w = 0.75 * p.m + 0.25 * p.n;
    let w = if w.is_finite() && w > 0.5 { w } else { 2.0 };
    let cw = 1.0 / arps_f(p.rw, p.rw_temp_f, ftemp_f).max(1e-4);
    let cmf = 1.0 / arps_f(p.rmf, p.rmf_temp_f, ftemp_f).max(1e-4);
    let t_c = (ftemp_f - 32.0) * 5.0 / 9.0;
    let cbw = 0.0007 * (t_c + 8.5) * (t_c + 298.0);
    let sal_w = salinity_ppm(p.rw, p.rw_temp_f);
    let sal_mf = salinity_ppm(p.rmf, p.rmf_temp_f);
    let alpha_u = alpha_expansion(sal_w);
    // Oil mud: no filtrate invasion of water — X zone driven by formation water too.
    let alpha_x = if p.mud_type.eq_ignore_ascii_case("OIL") { alpha_u } else { alpha_expansion(sal_mf) };
    FluidCalc {
        w,
        cw,
        cmf,
        cbw,
        cbw_x: cbw / alpha_x,
        cbw_u: cbw / alpha_u,
        alpha_x,
        alpha_u,
        salinity_w_ppm: sal_w,
        salinity_mf_ppm: sal_mf,
        u_ct: 0.03 * cw.powf(1.0 / w),
        u_cxo: 0.03 * cmf.powf(1.0 / w),
    }
}

/// Clay-bound-water multiplier k so that v_bw = k · v_dryclay (reference spec 5.03):
/// k = α · 96 · CEC[meq/g] · ρ_clay[g/cc] / (T°C + 298).
fn bndwat_multiplier(cec: f64, rho_gcc: f64, t_c: f64, alpha: f64) -> f64 {
    alpha * 96.0 * cec * rho_gcc / (t_c + 298.0)
}

/// Bound-water multiplier k (v_bw = k · v_dryclay) for one clay under the chosen porosity source.
///   CEC → α·96·CEC·ρ/(T+298)            (salinity/temperature-dependent, reference spec 5.03)
///   WCP → φ_clay/(1−φ_clay)             (geometric; V_wetclay = V_dryclay/(1−φ))
/// The WCP route uses the geometric form only for a *physical* φ. A degenerate φ ≥
/// [`WCP_PHYSICAL_CEILING`] — e.g. Techlog's smectite placeholder φ = 1.0, which Techlog itself
/// consumes only post-solve with a 1e-4 floor, never as an inversion constraint — falls back to the
/// CEC-calibrated multiplier so the two sources *agree* for that clay (smectite CEC=1.0 → k≈0.6)
/// instead of a 0.95-clamped k≈19 that would swamp the BNDWAT constraint.
fn bound_water_multiplier(source: PorositySource, cec: f64, wcp: f64, rho: f64, t_c: f64, alpha: f64) -> f64 {
    let cec_k = if cec > 0.0 { bndwat_multiplier(cec, rho, t_c, alpha) } else { 0.0 };
    match source {
        PorositySource::Cec => cec_k,
        PorositySource::WetClayPorosity => {
            if wcp > 0.0 && wcp < WCP_PHYSICAL_CEILING {
                wcp / (1.0 - wcp)
            } else if wcp >= WCP_PHYSICAL_CEILING {
                cec_k
            } else {
                0.0
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Wet-clay → dry-clay endpoint conversion (from a multimin parameter workbook)
// ---------------------------------------------------------------------------

/// Wet-clay log readings picked in a shale interval, plus the assumed dry-clay
/// density (2.70 marine / 2.78 deltaic in one study).
#[derive(Debug, Clone, Deserialize)]
pub struct WetClayInput {
    pub rhob_wet: f64,
    pub nphi_wet: f64,
    pub gr_wet: f64,
    #[serde(default)]
    pub dt_wet: Option<f64>,
    pub rho_dry: f64,
    /// Current fluid properties — the CEC equivalent uses their T and α_u so the
    /// solver's BNDWAT constraint reproduces the φ_clay bookkeeping exactly.
    #[serde(default)]
    pub fluid: Option<FluidProps>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DryClayCalc {
    /// Clay-bound porosity — the water fraction of the WET clay volume.
    pub phi_clay: f64,
    pub rhob_dry: f64,
    pub nphi_dry: f64,
    pub gr_dry: f64,
    pub dt_dry: Option<f64>,
    /// Bound-water tie for the dry-clay framework: v_bw = cbw_ratio · v_dryclay
    /// (CBW = φ_clay · V_wetclay and V_wetclay = V_dryclay / (1 − φ_clay)).
    pub cbw_ratio: f64,
    /// CEC (meq/g) that makes the solver's Dual-Water BNDWAT constraint enforce
    /// exactly cbw_ratio at the given fluid conditions — set it on the clay
    /// component together with the dry endpoints.
    pub cec_equiv: f64,
}

/// The xlsx conversion, verbatim (water at 1.00 g/cc and 189 µs/ft):
///   φ_clay   = (ρ_dry − ρ_wet) / (ρ_dry − 1.0)
///   NPHI_dry = (NPHI_wet − φ_clay) / (1 − φ_clay)
///   GR_dry   =  GR_wet / (1 − φ_clay)
///   DT_dry   = (DT_wet − 189·φ_clay) / (1 − φ_clay)
/// Deck slide 59 confirms the bookkeeping: bound water is an explicit solved
/// fluid volume (SWB = V_bw / PHIT), which is what cbw_ratio/cec_equiv feed.
pub fn dry_clay_calc(inp: &WetClayInput) -> Result<DryClayCalc, String> {
    const RHO_W: f64 = 1.0;
    const DT_W: f64 = 189.0;
    if !(inp.rhob_wet > RHO_W) {
        return Err("wet-clay RHOB must exceed 1.0 g/cc (the water density)".into());
    }
    if !(inp.rho_dry > inp.rhob_wet) {
        return Err("dry-clay density must exceed the wet-clay RHOB reading".into());
    }
    if !(inp.nphi_wet > 0.0 && inp.nphi_wet <= 1.0) {
        return Err("wet-clay NPHI must be a fraction in (0, 1] v/v — not percent".into());
    }
    if !(inp.gr_wet > 0.0) {
        return Err("wet-clay GR must be positive".into());
    }
    let phi = (inp.rho_dry - inp.rhob_wet) / (inp.rho_dry - RHO_W);
    let dry = 1.0 - phi;
    if let Some(d) = inp.dt_wet {
        if !(d > DT_W * phi) {
            return Err(format!(
                "wet-clay DT must exceed the water term 189·φ_clay = {:.1} µs/ft",
                DT_W * phi
            ));
        }
    }
    let cbw_ratio = phi / dry;
    let (t_c, alpha) = match &inp.fluid {
        Some(p) => ((p.ftemp_f - 32.0) * 5.0 / 9.0, fluid_calc(p).alpha_u),
        None => (25.0, 1.0),
    };
    // Invert bndwat_multiplier (the clay's RHOB endpoint becomes ρ_dry on apply).
    let cec_equiv = cbw_ratio * (t_c + 298.0) / (alpha * 96.0 * inp.rho_dry);
    Ok(DryClayCalc {
        phi_clay: phi,
        rhob_dry: inp.rho_dry,
        nphi_dry: (inp.nphi_wet - phi) / dry,
        gr_dry: inp.gr_wet / dry,
        dt_dry: inp.dt_wet.map(|d| (d - DT_W * phi) / dry),
        cbw_ratio,
        cec_equiv,
    })
}

// ---------------------------------------------------------------------------
// Fluid-property autofill from the precalc module's output curves
// ---------------------------------------------------------------------------

/// Zone-averaged fluid entries read from precalc outputs (FTEMP_F in °F and RMF
/// in ohmm at formation temperature). `None` when the curve has no finite sample
/// in the interval — precalc has not been run, or the zone is empty.
#[derive(Debug, Clone, Serialize)]
pub struct PrecalcFluid {
    pub ftemp_f: Option<f64>,
    pub rmf: Option<f64>,
    pub n_ftemp: usize,
    pub n_rmf: usize,
}

pub fn fluid_from_precalc(
    db: &Mutex<Connection>,
    well_id: &str,
    top: Option<f64>,
    bottom: Option<f64>,
) -> Result<PrecalcFluid, String> {
    let conn = db.lock().unwrap();
    let names = vec!["FTEMP_F".to_string(), "RMF".to_string()];
    let (depth, cols) = fetch_curve_frame(&conn, well_id, &names).map_err(|e| e.to_string())?;
    let in_range = |d: f32| -> bool {
        let d = d as f64;
        top.is_none_or(|t| d >= t) && bottom.is_none_or(|b| d <= b)
    };
    let mean_of = |name: &str| -> (Option<f64>, usize) {
        let col = cols.get(name).expect("requested curve present");
        let mut s = 0.0;
        let mut n = 0usize;
        for (i, v) in col.iter().enumerate() {
            if v.is_finite() && depth[i].is_finite() && in_range(depth[i]) {
                s += *v as f64;
                n += 1;
            }
        }
        (if n > 0 { Some(s / n as f64) } else { None }, n)
    };
    let (ftemp_f, n_ftemp) = mean_of("FTEMP_F");
    let (rmf, n_rmf) = mean_of("RMF");
    Ok(PrecalcFluid { ftemp_f, rmf, n_ftemp, n_rmf })
}

// ---------------------------------------------------------------------------
// Run
// ---------------------------------------------------------------------------

const SIGMA_CONSTRAINT: f64 = 0.01; // nominal Tool-constraint uncertainty

/// Sane per-sample FTEMP window (°F). Outside it a curve value is bad data — null sentinels
/// (−999.25, +999.25), zero fills, etc. — so a per-depth FTEMP that is not finite or falls outside this
/// range reverts to the constant fluid temperature instead of feeding a nonsensical T into the fluid
/// calc. The floor (below freezing) and ceiling (hotter than any real reservoir, ~315 °C) bracket every
/// physical formation temperature while rejecting the common ±999.25 / 9999 fills.
const FTEMP_MIN_F: f64 = 32.0;
const FTEMP_MAX_F: f64 = 600.0;

/// Wet-Clay-Porosity route: the largest φ_clay we treat as a real geometric porosity.
/// Techlog's real clays sit at φ ≤ 0.156; only smectite carries φ = 1.0, and that value
/// is a *post-solve* placeholder (Techlog floors 1−φ at 1e-4 for wet-clay-volume output,
/// never as an inversion constraint). φ ≥ this ceiling means "bound water ≥ dry-clay
/// volume" — not a usable geometric porosity for a solver constraint — so we defer to the
/// CEC-derived multiplier for that clay instead of letting k = φ/(1−φ) approach the pole.
const WCP_PHYSICAL_CEILING: f64 = 0.5;

struct ZoneSets {
    /// Unity coefficients (1 for minerals/clays/U-and-shared fluids, 0 for X fluids).
    unity: Vec<f64>,
    clays: Vec<usize>,
    x_water: Vec<usize>,
    u_water: Vec<usize>,
    x_hc: Vec<usize>,
    u_hc: Vec<usize>,
    x_bw: Vec<usize>,
    u_bw: Vec<usize>,
    x_fluids: Vec<usize>,
    u_fluids: Vec<usize>,
    has_split: bool,
}

fn classify(comps: &[Component]) -> ZoneSets {
    let n = comps.len();
    let mut z = ZoneSets {
        unity: vec![0.0; n],
        clays: vec![],
        x_water: vec![],
        u_water: vec![],
        x_hc: vec![],
        u_hc: vec![],
        x_bw: vec![],
        u_bw: vec![],
        x_fluids: vec![],
        u_fluids: vec![],
        has_split: false,
    };
    for (i, c) in comps.iter().enumerate() {
        let is_fluid = c.kind.eq_ignore_ascii_case("fluid");
        let zone = c.zone.trim().to_uppercase();
        if !is_fluid {
            z.unity[i] = 1.0;
            if c.kind.eq_ignore_ascii_case("clay") {
                z.clays.push(i);
            }
            continue;
        }
        let in_x = zone != "U"; // X or shared
        let in_u = zone != "X"; // U or shared
        if in_u {
            z.unity[i] = 1.0;
            z.u_fluids.push(i);
        }
        if in_x {
            z.x_fluids.push(i);
        }
        match c.fluid_type.trim().to_lowercase().as_str() {
            "bound_water" => {
                if in_x {
                    z.x_bw.push(i);
                }
                if in_u {
                    z.u_bw.push(i);
                }
            }
            "oil" | "gas" => {
                if in_x {
                    z.x_hc.push(i);
                }
                if in_u {
                    z.u_hc.push(i);
                }
            }
            _ => {
                if in_x {
                    z.x_water.push(i);
                }
                if in_u {
                    z.u_water.push(i);
                }
            }
        }
    }
    // A real X/U split exists only if some fluid is exclusive to each zone.
    let has_x_only = comps.iter().any(|c| c.kind.eq_ignore_ascii_case("fluid") && c.zone.eq_ignore_ascii_case("X"));
    let has_u_only = comps.iter().any(|c| c.kind.eq_ignore_ascii_case("fluid") && c.zone.eq_ignore_ascii_case("U"));
    z.has_split = has_x_only && has_u_only;
    z
}

/// Scale the volumes at `idx` so they sum to `target`, preserving their relative split (even split
/// when they are all ~0). The post-solve Sw models use it to impose Sw·φe on the water set and
/// (1−Sw)·φe on the HC set without changing φe — so PHIE and hard unity stay exactly as solved.
fn set_group(x: &mut [f64], idx: &[usize], target: f64) {
    if idx.is_empty() {
        return;
    }
    let cur: f64 = idx.iter().map(|&c| x[c]).sum();
    if cur > 1e-12 {
        let s = target / cur;
        for &c in idx {
            x[c] *= s;
        }
    } else {
        let each = target / idx.len() as f64;
        for &c in idx {
            x[c] = each;
        }
    }
}

/// Scale a response row by an equation weight (row·w) — one equation's contribution to A.
fn scaled(row: &[f64], w: f64) -> Vec<f64> {
    row.iter().map(|e| e * w).collect()
}

/// A conductivity tool's response row. CT reads the U (virgin) zone against Cw/Cbw_u; CXO the X
/// (flushed) zone against Cmf/Cbw_x. Water and bound-water components take C^(1/w); everything else is
/// 0. The non-zero PATTERN is temperature-independent (it depends only on which components are water in
/// the zone), so it is validated once; an FTEMP curve only moves the values via a per-sample fluid calc.
fn cond_tool_row(is_ct: bool, fc: &FluidCalc, zs: &ZoneSets, n: usize) -> Vec<f64> {
    let inv_w = 1.0 / fc.w;
    let mut row = vec![0.0f64; n];
    if is_ct {
        for &i in &zs.u_water {
            row[i] = fc.cw.powf(inv_w);
        }
        for &i in &zs.u_bw {
            row[i] = fc.cbw_u.powf(inv_w);
        }
    } else {
        for &i in &zs.x_water {
            row[i] = fc.cmf.powf(inv_w);
        }
        for &i in &zs.x_bw {
            row[i] = fc.cbw_x.powf(inv_w);
        }
    }
    row
}

/// BNDWAT soft-constraint rows (Σ k·v_clay − v_bw = 0), one per bound-water set (X and/or U). The
/// multiplier k = `bound_water_multiplier` depends on t_c (and the salinity-driven, T-independent α);
/// the row SET — which clays contribute — is temperature-independent, so callers build it once for the
/// count/DOF and rebuild only the k values per sample when an FTEMP curve varies t_c.
fn bndwat_soft_rows(
    zs: &ZoneSets,
    comps: &[Component],
    source: PorositySource,
    t_c: f64,
    alpha_x: f64,
    alpha_u: f64,
    n: usize,
) -> Vec<Vec<f64>> {
    let mut bw_sets: Vec<(&Vec<usize>, f64)> = Vec::new();
    if !zs.x_bw.is_empty() && zs.x_bw != zs.u_bw {
        bw_sets.push((&zs.x_bw, alpha_x));
    }
    if !zs.u_bw.is_empty() {
        bw_sets.push((&zs.u_bw, alpha_u));
    }
    let mut out = Vec::new();
    for (bw_idx, alpha) in bw_sets {
        let mut row = vec![0.0f64; n];
        let mut any = false;
        for &ci in &zs.clays {
            let c = &comps[ci];
            let rho = *c.endpoints.get("RHOB").unwrap_or(&2.65);
            let k = bound_water_multiplier(source, c.cec, c.wet_clay_porosity, rho, t_c, alpha);
            if k > 0.0 {
                row[ci] = k;
                any = true;
            }
        }
        if any {
            for &bi in bw_idx {
                row[bi] = -1.0;
            }
            out.push(row);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Core calibration — does the solved model reproduce an independent measurement?
// ---------------------------------------------------------------------------

/// Max |depth difference| for tying a core plug to a log sample. Matches the 1.0 m convention
/// already used for core-plug tie-in in `facies_tie.rs`.
const CORE_MATCH_TOL_M: f32 = 1.0;

/// Index of the SOLVED log sample closest to `target` within `CORE_MATCH_TOL_M`, or None.
/// Unsolved samples (NaN) are skipped rather than matched, so a plug either lands on a real
/// solved value or is dropped from the statistic. Linear scan: plug counts are in the tens, and
/// this assumes nothing about the depth grid being ascending.
fn nearest_solved(depth: &[f32], model: &[f32], target: f32) -> Option<usize> {
    let mut best: Option<usize> = None;
    for i in 0..depth.len().min(model.len()) {
        if !model[i].is_finite() {
            continue;
        }
        let dd = (depth[i] - target).abs();
        if dd > CORE_MATCH_TOL_M {
            continue;
        }
        if best.map_or(true, |b| dd < (depth[b] - target).abs()) {
            best = Some(i);
        }
    }
    best
}

/// RMS and mean signed error of `model` against core plugs `(depth, value)`, over the plugs that
/// tie to a solved sample. None when nothing matched.
fn core_fit(depth: &[f32], model: &[f32], plugs: &[(f32, f32)]) -> Option<CoreFit> {
    let mut n = 0usize;
    let mut sse = 0.0f64;
    let mut sum = 0.0f64;
    for &(d, cv) in plugs {
        let Some(i) = nearest_solved(depth, model, d) else { continue ;
        };
        let e = model[i] as f64 - cv as f64;
        sse += e * e;
        sum += e;
        n += 1;
    }
    if n == 0 {
        return None;
    }
    Some(CoreFit { n, rms: (sse / n as f64).sqrt() as f32, bias: (sum / n as f64) as f32 })
}

pub fn run_sandimin(
    db: &Mutex<Connection>,
    req: &SandiminRequest,
    progress: Option<&crate::jobs::JobHandle>,
) -> SandiminResult {
    let n = req.components.len();
    if n < 2 {
        return fail("select at least two components");
    }
    let model = req.sw_model;
    // SB-SAT-026: the registry carries identities the solver does not implement; refuse them by
    // name instead of reaching a post-solve branch that cannot exist.
    if !model.solver_selectable() {
        return fail(&solver_refusal(model));
    }
    let post_solve = model.is_post_solve();
    let tools: Vec<&ToolSpec> = req.tools.iter().filter(|t| !t.curve.trim().is_empty()).collect();
    if tools.is_empty() {
        return fail("select at least one input log");
    }
    if req.apply_well_ids.is_empty() {
        return fail("select at least one well to apply to");
    }

    let has_cond = tools.iter().any(|t| is_cond_key(&t.key));
    let fluid = if has_cond {
        match &req.fluid {
            Some(p) => Some(fluid_calc(p)),
            None => return fail("CT/CXO selected but fluid properties (Rw, Rmf, temperature, m, n) are missing"),
        }
    } else {
        req.fluid.as_ref().map(fluid_calc)
    };

    let zs = classify(&req.components);
    if has_cond && zs.x_water.is_empty() && zs.u_water.is_empty() && zs.x_bw.is_empty() && zs.u_bw.is_empty() {
        return fail("CT/CXO selected but no water component is in the model");
    }
    // Post-solve shaly-sand Sw (Indonesia/Simandoux): the conductivity tool STAYS in the inversion —
    // dropping it would leave the U-zone water/HC split collinear (both invisible to the nuclear
    // tools) and the solve singular. We keep the well-posed linear solve and only REPLACE the
    // reported Sw with the closed form, reading Rt/Rxo straight from the conductivity tools' columns.
    // Sw redistributes φe into (Sw·φe water, (1−Sw)·φe HC), so a U-zone HC component must be present.
    let ct_tool_idx = tools.iter().position(|t| t.key.trim().eq_ignore_ascii_case("CT"));
    let cxo_tool_idx = tools.iter().position(|t| t.key.trim().eq_ignore_ascii_case("CXO"));
    // A shared-zone (zone "") water/HC sits in BOTH the x_* and u_* sets, so the U then X overrides
    // would scale it twice and corrupt PHIE/SWE/unity. The flushed-zone override therefore runs only
    // when the X and U fluid sets are disjoint (the standard zone-exclusive Sxo/Sw model); a shared
    // fluid means no invasion (Sxo = Sw), so leaving the X split as solved is already correct.
    let post_zones_disjoint = zs.x_water.iter().all(|i| !zs.u_water.contains(i))
        && zs.x_hc.iter().all(|i| !zs.u_hc.contains(i));
    if post_solve {
        if ct_tool_idx.is_none() {
            return fail("the non-linear Sw models need the deep-resistivity tool (CT) — add it");
        }
        if zs.u_water.is_empty() || zs.u_hc.is_empty() {
            return fail(
                "the non-linear Sw models need both a U-zone water and a U-zone hydrocarbon component",
            );
        }
    }

    // Static per-tool data: weights, endpoint rows (conductivity rows built from fluid calc).
    let t_c = req.fluid.as_ref().map(|p| (p.ftemp_f - 32.0) * 5.0 / 9.0).unwrap_or(25.0);
    let mut weights: Vec<f64> = Vec::with_capacity(tools.len());
    let mut rows: Vec<Vec<f64>> = Vec::with_capacity(tools.len());
    let mut tkind: Vec<TKind> = Vec::with_capacity(tools.len());
    for t in &tools {
        let key = t.key.trim().to_uppercase();
        if is_cond_key(&key) {
            let fc = fluid.as_ref().unwrap();
            let is_ct = key == "CT";
            let row = cond_tool_row(is_ct, fc, &zs, n);
            // An all-zero conductivity row is the bogus equation 0 = Ct^(1/w): it happens when
            // the model has no water/bound-water in this tool's zone (e.g. CT but only X-zone
            // water). The whole-model no-water case is caught earlier; this catches the
            // per-zone case that slips past it. The pattern is T-independent, so checking it once
            // (here) also covers every per-sample FTEMP rebuild.
            if row.iter().all(|&e| e == 0.0) {
                let need = if is_ct { "U-zone (deep) water or bound-water" } else { "X-zone (flushed) water or bound-water" };
                return fail(&format!(
                    "{key} selected but the model has no {need} component — its response row is all zero"
                ));
            }
            let auto_sigma = if is_ct { fc.u_ct } else { fc.u_cxo };
            let sigma = if t.sigma > 0.0 { t.sigma } else { auto_sigma };
            weights.push(1.0 / sigma.max(1e-9));
            rows.push(row);
            tkind.push(TKind::Cond(fc.w, is_ct));
        } else if is_pef_key(&key) {
            // PEF is a PER-ELECTRON index and does NOT mix by volume; only U = Pe·ρe
            // does. Build the mixing row from the U endpoints and convert the measured
            // PEF curve to U per sample (see the sample loop / rho_e).
            if t.sigma <= 0.0 {
                return fail(&format!("tool {key} needs a positive uncertainty"));
            }
            let row: Vec<f64> = req
                .components
                .iter()
                .map(|c| {
                    if c.kind.eq_ignore_ascii_case("fluid") && c.zone.eq_ignore_ascii_case("U") {
                        0.0
                    } else {
                        *c.endpoints.get("U").unwrap_or(&f64::NAN)
                    }
                })
                .collect();
            if row.iter().any(|e| !e.is_finite()) {
                return fail(
                    "PEF selected but a component is missing its U (b/cc) endpoint — PEF is converted to U before mixing",
                );
            }
            weights.push(1.0 / t.sigma); // base; the per-sample weight also divides by ρe
            rows.push(row);
            tkind.push(TKind::Pef(t.sigma));
        } else {
            if t.sigma <= 0.0 {
                return fail(&format!("tool {key} needs a positive uncertainty"));
            }
            let row: Vec<f64> = req
                .components
                .iter()
                .map(|c| {
                    // U-zone-exclusive fluids are invisible to all non-CT tools.
                    if c.kind.eq_ignore_ascii_case("fluid") && c.zone.eq_ignore_ascii_case("U") {
                        0.0
                    } else {
                        *c.endpoints.get(&key).unwrap_or(&f64::NAN)
                    }
                })
                .collect();
            if row.iter().any(|e| !e.is_finite()) {
                return fail(&format!("tool {key} has a missing endpoint — fill the endpoints table"));
            }
            weights.push(1.0 / t.sigma);
            rows.push(row);
            tkind.push(TKind::Plain);
        }
    }

    // Soft constraint rows (built once; appended after the live tool rows each sample). POROSITY is
    // temperature-independent and lives here; BNDWAT (below) depends on t_c, so it is split out so an
    // FTEMP curve can rebuild only its k values per sample without touching this row.
    let mut soft: Vec<(Vec<f64>, f64)> = Vec::new();
    if zs.has_split && req.enforce_porosity {
        let mut row = vec![0.0f64; n];
        for &i in &zs.x_fluids {
            row[i] += 1.0;
        }
        for &i in &zs.u_fluids {
            row[i] -= 1.0;
        }
        soft.push((row, 0.0)); // POROSITY: Σ X fluids − Σ U fluids = 0
    }
    // BNDWAT soft rows at the constant fluid temperature. α is salinity-driven (T-independent); only k
    // moves with t_c. Kept separate from `soft` and appended after it each sample (identical row order
    // to before), so the constant-T solve is byte-for-byte unchanged.
    let (alpha_x_s, alpha_u_s) = fluid.as_ref().map(|f| (f.alpha_x, f.alpha_u)).unwrap_or((1.0, 1.0));
    let bndwat_static: Vec<Vec<f64>> = if !zs.clays.is_empty() && req.enforce_bndwat {
        bndwat_soft_rows(&zs, &req.components, req.porosity_source, t_c, alpha_x_s, alpha_u_s, n)
    } else {
        Vec::new()
    };
    let sigma = if req.sigma_constraint > 0.0 { req.sigma_constraint } else { SIGMA_CONSTRAINT };
    let soft_weight = 1.0 / sigma;

    // WATER MUD row (used only on violation re-solve): Σ X waters − Σ U waters = 0.
    let water_mud_row: Option<Vec<f64>> = if zs.has_split
        && req.enforce_water_mud
        && req.fluid.as_ref().map(|p| !p.mud_type.eq_ignore_ascii_case("OIL")).unwrap_or(true)
    {
        let mut row = vec![0.0f64; n];
        for &i in zs.x_water.iter().chain(&zs.x_bw) {
            row[i] += 1.0;
        }
        for &i in zs.u_water.iter().chain(&zs.u_bw) {
            row[i] -= 1.0;
        }
        Some(row)
    } else {
        None
    };

    let hi: Vec<f64> = req.components.iter().map(|c| if c.max_vol > 0.0 { c.max_vol.min(1.0) } else { 1.0 }).collect();
    let unity_c = if req.unity { Some(zs.unity.clone()) } else { None };

    // Minimum live tool rows per sample: volumes minus unity/soft-constraint degrees of freedom. The
    // BNDWAT row COUNT is temperature-independent, so `bndwat_static` gives the right count for every
    // sample even when an FTEMP curve later rebuilds its values.
    let n_extra = soft.len() + bndwat_static.len() + usize::from(req.unity);
    let min_tools = n.saturating_sub(n_extra).max(1);
    if tools.len() < min_tools {
        return fail(&format!(
            "need at least {min_tools} input logs to constrain {n} components (have {})",
            tools.len()
        ));
    }
    // Model degrees of freedom with every tool live: equations (tools + soft + unity) − unknowns.
    let dof = tools.len() as i64 + n_extra as i64 - n as i64;
    let dof_note = (dof == 0).then(|| {
        format!(
            "exactly determined: {} equation(s) for {n} component(s), so RECON is forced to ~0 and \
             cannot validate the model — add an input log for a real reconstruction check",
            tools.len() + n_extra
        )
    });
    let recon_qc = req.recon_qc;

    let comp_tokens: Vec<String> = req.components.iter().map(|c| curve_token(&c.name)).collect();
    let vol_names: Vec<String> = comp_tokens.iter().map(|t| format!("VOL_{t}")).collect();
    // Uppercase the output prefix so a re-cased prefix (e.g. "mm" after "MM") can't leave a stale
    // case-shadow row: every computed-curve reader resolves case-insensitively but the delete-then-
    // append writer deletes by exact curve_name. Matches curve_token()'s uppercasing of components.
    let prefix_upper = req.output_prefix.trim().to_uppercase();
    let prefix = if prefix_upper.is_empty() { "SM" } else { prefix_upper.as_str() };

    let fetch_names: Vec<String> = tools.iter().map(|t| t.curve.trim().to_uppercase()).collect();
    // PEF→U conversion needs the density curve even when RHOB is not itself a tool.
    // Resolve the density source and fetch it alongside (without disturbing tool_cols).
    let density_name: Option<String> = if tkind.iter().any(|k| matches!(k, TKind::Pef(_))) {
        Some(
            tools
                .iter()
                .find(|t| t.key.trim().eq_ignore_ascii_case("RHOB"))
                .map(|t| t.curve.trim().to_uppercase())
                .unwrap_or_else(|| "RHOB".to_string()),
        )
    } else {
        None
    };
    // Optional per-depth formation temperature (°F). When set, the T-dependent fluid calc + t_c are
    // recomputed per sample; a missing curve fetches as all-NaN and every sample falls back to the
    // fixed temperature, so selecting it is harmless if the well lacks it.
    let ftemp_name: Option<String> =
        req.ftemp_curve.as_ref().map(|c| c.trim().to_uppercase()).filter(|c| !c.is_empty());
    let mut all_fetch = fetch_names.clone();
    if let Some(d) = &density_name {
        if !all_fetch.contains(d) {
            all_fetch.push(d.clone());
        }
    }
    if let Some(f) = &ftemp_name {
        if !all_fetch.contains(f) {
            all_fetch.push(f.clone());
        }
    }

    let mut out_names: Vec<String> = Vec::new();
    let mut wells: Vec<SandiminWellResult> = Vec::new();

    let n_wells = req.apply_well_ids.len();
    let conn = db.lock().unwrap();
    for (wi, well_id) in req.apply_well_ids.iter().enumerate() {
        // Cooperative cancel + live per-well progress into the Processing panel. The panel polls
        // the (separate) jobs registry, so these updates land even while this holds the DB lock.
        if let Some(p) = progress {
            if p.is_cancelled() {
                break;
            }
            p.set_current(Some(format!("SandiMin: well {}/{}", wi + 1, n_wells)));
            p.start_item(well_id);
        }
        let (depth, cols) = match crate::equations::fetch_curve_frame_from_set(
            &conn, well_id, &all_fetch, req.input_set.as_deref(), None,
        ) {
            Ok(v) => v,
            Err(e) => {
                if let Some(p) = progress {
                    p.finish_item(well_id, crate::jobs::ItemState::Failed, Some(e.to_string()));
                }
                wells.push(SandiminWellResult {
                    well_id: well_id.clone(),
                    rows_solved: 0,
                    mean_recon: f32::NAN,
                    core_phie: None,
                    core_phit: None,
                    core_gd: None,
                    core_note: None,
                    error: Some(e.to_string()),
                });
                continue;
            }
        };
        let ns = depth.len();
        let tool_cols: Vec<&Vec<f32>> = fetch_names.iter().map(|nm| cols.get(nm).unwrap()).collect();
        let rhob_col: Option<&Vec<f32>> = density_name.as_ref().and_then(|d| cols.get(d));
        let ftemp_col: Option<&Vec<f32>> = ftemp_name.as_ref().and_then(|d| cols.get(d));

        let mut vol: Vec<Vec<f32>> = vec![vec![f32::NAN; ns]; n];
        let mut recon = vec![f32::NAN; ns];
        let mut solved = 0usize;
        let mut recon_sum = 0.0f64;
        // Per-tool reconstruction QC (recon_qc): reconstructed measurement + σ-unit residual.
        let mut tool_rec: Vec<Vec<f32>> = if recon_qc { vec![vec![f32::NAN; ns]; tools.len()] } else { Vec::new() };
        let mut tool_dif: Vec<Vec<f32>> = if recon_qc { vec![vec![f32::NAN; ns]; tools.len()] } else { Vec::new() };

        for i in 0..ns {
            let mut a: Vec<Vec<f64>> = Vec::with_capacity(tools.len() + soft.len());
            let mut b: Vec<f64> = Vec::with_capacity(tools.len() + soft.len());
            let mut live_tools = 0usize;
            // (tool index, measured value in solve domain, weight) for the reconstruction QC.
            let mut live: Vec<(usize, f64, f64)> = Vec::new();
            // Per-sample formation temperature: a finite FTEMP curve value recomputes the T-dependent
            // fluid calc + t_c for this depth (conductivity rows, auto CT/CXO σ, BNDWAT k, Waxman-Smits
            // B). Absent/non-finite ⇒ the constant-T static values, byte-identical to before.
            let ftemp_i: Option<f64> = ftemp_col.and_then(|c| {
                let tf = c[i] as f64;
                (tf.is_finite() && tf > FTEMP_MIN_F && tf < FTEMP_MAX_F).then_some(tf)
            });
            let t_c_i = ftemp_i.map(|tf| (tf - 32.0) * 5.0 / 9.0).unwrap_or(t_c);
            let sample_fc: Option<FluidCalc> = match (ftemp_i, req.fluid.as_ref()) {
                (Some(tf), Some(p)) => Some(fluid_calc_at(p, tf)),
                _ => None,
            };
            for (t, tcol) in tool_cols.iter().enumerate() {
                let raw = tcol[i] as f64;
                if !raw.is_finite() {
                    continue;
                }
                // Resolve this tool's (solve-domain value, weight, weighted A-row). Conductivity tools
                // rebuild their row/weight from the per-sample fluid calc when an FTEMP curve is active.
                let (v, w, arow): (f64, f64, Vec<f64>) = match tkind[t] {
                    TKind::Plain => {
                        let w = weights[t];
                        (raw, w, scaled(&rows[t], w))
                    }
                    TKind::Cond(w_exp, is_ct) => {
                        // Resistivity (ohmm) → conductivity (mho/m) → ^(1/w) transform.
                        if raw <= 1e-4 {
                            continue;
                        }
                        let v = (1.0 / raw).powf(1.0 / w_exp);
                        match &sample_fc {
                            Some(fc_i) => {
                                let row = cond_tool_row(is_ct, fc_i, &zs, n);
                                let auto = if is_ct { fc_i.u_ct } else { fc_i.u_cxo };
                                let sig = if tools[t].sigma > 0.0 { tools[t].sigma } else { auto };
                                let w = 1.0 / sig.max(1e-9);
                                (v, w, scaled(&row, w))
                            }
                            None => {
                                let w = weights[t];
                                (v, w, scaled(&rows[t], w))
                            }
                        }
                    }
                    TKind::Pef(sig) => {
                        // U = Pe·ρe (volumetric); its uncertainty in U space is σ_PEF·ρe.
                        let rhob = match rhob_col.map(|c| c[i] as f64) {
                            Some(rb) if rb.is_finite() && rb > 0.0 => rb,
                            _ => continue,
                        };
                        let re = rho_e(rhob);
                        let w = 1.0 / (sig * re).max(1e-9);
                        (raw * re, w, scaled(&rows[t], w))
                    }
                };
                a.push(arow);
                b.push(v * w);
                if recon_qc {
                    live.push((t, v, w));
                }
                live_tools += 1;
            }
            if live_tools < min_tools {
                continue;
            }
            let n_tool_rows = a.len();
            for (row, rhs) in &soft {
                a.push(scaled(row, soft_weight));
                b.push(rhs * soft_weight);
            }
            // BNDWAT rows: rebuilt at the per-sample t_c when an FTEMP curve is active, else the static
            // rows. Row COUNT is identical either way, so A/b stay dimensionally consistent (rhs = 0).
            let bndwat_owned;
            let bndwat: &[Vec<f64>] = if ftemp_i.is_some() && !bndwat_static.is_empty() {
                bndwat_owned = bndwat_soft_rows(
                    &zs,
                    &req.components,
                    req.porosity_source,
                    t_c_i,
                    alpha_x_s,
                    alpha_u_s,
                    n,
                );
                &bndwat_owned
            } else {
                &bndwat_static
            };
            for row in bndwat {
                a.push(scaled(row, soft_weight));
                b.push(0.0);
            }

            let mut x = match solve_bounded_lsq(&a, &b, unity_c.as_deref(), &hi, n) {
                Some(x) => x,
                None => continue,
            };
            // WATER MUD: for WBM, flushed-zone water cannot be less than unflushed water.
            if let Some(wm) = &water_mud_row {
                let s: f64 = wm.iter().zip(&x).map(|(c, v)| c * v).sum();
                if s < -1e-6 {
                    let mut a2 = a.clone();
                    let mut b2 = b.clone();
                    a2.push(wm.iter().map(|e| e * soft_weight).collect());
                    b2.push(0.0);
                    if let Some(x2) = solve_bounded_lsq(&a2, &b2, unity_c.as_deref(), &hi, n) {
                        x = x2;
                    }
                }
            }

            // Post-solve shaly-sand Sw (Indonesia/Simandoux): the linear inversion above fixed φe and
            // Vsh; now REPLACE the water/HC split with the closed-form Sw. φe is preserved, so PHIE
            // and hard unity are untouched and SWE becomes the model Sw. RECON is computed AFTER this,
            // so for these models it also measures how well the chosen Sw coheres with every tool.
            if post_solve {
                // Per-sample fluid calc under an FTEMP curve, else the constant-T one; the closed-form
                // Sw (Cw/Cbw and, for Waxman-Smits, B(t_c,Rw)) then uses this depth's temperature.
                let fc = sample_fc.as_ref().unwrap_or_else(|| fluid.as_ref().unwrap());
                let fp = req.fluid.as_ref().unwrap();
                let (m_exp, n_exp, a_arch, rsh, phit_sh, ws_b) =
                    (fp.m, fp.n, fp.archie_a, fp.rsh, fp.phit_sh, fp.ws_b);
                let vsh = zs.clays.iter().chain(&zs.u_bw).map(|&c| x[c]).sum::<f64>();
                // Waxman-Smits cation term Σ v_clay·CEC·ρ_clay [meq per unit bulk volume]; divided by
                // the zone's φt inside the arm to give Qv in meq/mL of pore. Zone-independent (clay
                // volumes are shared between the flushed and virgin fluids), so it's computed once.
                let qv_num: f64 = if matches!(model, SwModel::WaxmanSmits) {
                    zs.clays
                        .iter()
                        .map(|&ci| {
                            let c = &req.components[ci];
                            x[ci] * c.cec * *c.endpoints.get("RHOB").unwrap_or(&2.65)
                        })
                        .sum()
                } else {
                    0.0
                };
                let read_res = |idx: Option<usize>| -> Option<f64> {
                    idx.map(|t| tool_cols[t][i] as f64).filter(|v| v.is_finite() && *v > 0.0)
                };
                // Returns the EFFECTIVE water fraction (free water / φe) so the φe redistribution below is
                // one code path for every model. Indonesia/Simandoux read Rw = 1/cw; the dual-water form
                // additionally uses the zone's clay-bound-water conductivity `cwb` and solved v_bw.
                let sw_of = |rt: f64, phie: f64, cw: f64, cwb: f64, v_bw: f64| -> f64 {
                    match model {
                        SwModel::Indonesia => {
                            sw_indonesia(
                                rt,
                                phie,
                                vsh,
                                1.0 / cw.max(1e-9),
                                rsh,
                                m_exp,
                                n_exp,
                                a_arch,
                                fp.indonesia_k,
                            )
                        }
                        SwModel::SimandouxBardonPied => {
                            sw_simandoux_bardon_pied(
                                rt,
                                phie,
                                vsh,
                                1.0 / cw.max(1e-9),
                                rsh,
                                m_exp,
                                n_exp,
                                a_arch,
                            )
                        }
                        SwModel::SimandouxModifiedSlb => {
                            sw_simandoux_modified_slb(
                                rt,
                                phie,
                                vsh,
                                1.0 / cw.max(1e-9),
                                rsh,
                                m_exp,
                                n_exp,
                                a_arch,
                                fp.simandoux_c,
                            )
                        }
                        SwModel::DualWaterNonlinear => {
                            // Total-basis dual water: bound-water saturation from the solved v_bw, then
                            // convert Swt back to a free-water/φe fraction. φe (= φt − v_bw) is preserved,
                            // so PHIT/PHIE/unity stay as solved and only the water/HC split moves.
                            let phit = phie + v_bw;
                            if !(phit > 1e-9) || !(phie > 1e-9) {
                                return f64::NAN;
                            }
                            let swb = (v_bw / phit).clamp(0.0, 1.0);
                            let swt = sw_dual_nonlinear(rt, phit, swb, cw, cwb, m_exp, n_exp, a_arch);
                            if !swt.is_finite() {
                                return f64::NAN;
                            }
                            // SB-SAT-023: the shared per-model back-out (first group, where the
                            // construction collapses 1−φe/φt with v_bw/φt).
                            effective_backout(model, swt, phie, phit, vsh, phit_sh).0.clamp(0.0, 1.0)
                        }
                        SwModel::ArchieTotal => {
                            // Clean-sand Archie on total porosity, then free-water/φe (same conversion as
                            // dual water; the total water Swt·φt includes the solved v_bw as bound water).
                            let phit = phie + v_bw;
                            if !(phit > 1e-9) || !(phie > 1e-9) {
                                return f64::NAN;
                            }
                            let swt = sw_archie(rt, phit, 1.0 / cw.max(1e-9), m_exp, n_exp, a_arch);
                            if !swt.is_finite() {
                                return f64::NAN;
                            }
                            effective_backout(model, swt, phie, phit, vsh, phit_sh).0.clamp(0.0, 1.0)
                        }
                        SwModel::ArchieEffective => {
                            // SB-SAT-002: Archie directly on phie. The result IS the free-water/phie
                            // fraction - bound water never enters the equation, so unlike every other
                            // post-solve branch there is no total->effective conversion to apply, and
                            // applying one here would be the 25-saturation-unit trap the row names.
                            if !(phie > 1e-9) {
                                return f64::NAN;
                            }
                            let swe = sw_archie(rt, phie, 1.0 / cw.max(1e-9), m_exp, n_exp, a_arch);
                            if !swe.is_finite() {
                                return f64::NAN;
                            }
                            swe.clamp(0.0, 1.0)
                        }
                        SwModel::SwRtc | SwModel::SwImts | SwModel::SwHeight => f64::NAN,
                        SwModel::Juhasz => {
                            // Normalized Waxman-Smits on total porosity: excess conductivity from the
                            // shale point (Cwsh, QVN=Vsh·φ_sh/φt), then free-water/φe (same split as dual
                            // water — the mineral model owns φe/v_bw; this only remaps conductivity→Swt).
                            let phit = phie + v_bw;
                            if !(phit > 1e-9) || !(phie > 1e-9) {
                                return f64::NAN;
                            }
                            let swt = sw_juhasz(rt, phit, vsh, cw, rsh, phit_sh, m_exp, n_exp);
                            if !swt.is_finite() {
                                return f64::NAN;
                            }
                            // SB-SAT-023: Juhász's back-out is Qvn from the shale point. The
                            // correct Qvn used to be computed and then OVERRIDDEN by the blanket
                            // porosity-volume conversion — the exact defect the row names.
                            effective_backout(model, swt, phie, phit, vsh, phit_sh).0.clamp(0.0, 1.0)
                        }
                        SwModel::WaxmanSmits => {
                            // Total-porosity B·Qv form: Qv from the solved clay volumes, B from the
                            // Juhász B(T,Rw) fit (Rw = 1/cw at formation T — the filtrate for the X
                            // zone, formation water for the U zone) unless overridden. Same free-water/φe
                            // remap as dual water — this only maps conductivity → Swt.
                            let phit = phie + v_bw;
                            if !(phit > 1e-9) || !(phie > 1e-9) {
                                return f64::NAN;
                            }
                            let qv = qv_num / phit;
                            let b = if ws_b > 0.0 { ws_b } else { waxman_b(t_c_i, 1.0 / cw.max(1e-9)) };
                            let swt = sw_waxman_smits(rt, phit, qv, cw, b, m_exp, n_exp);
                            if !swt.is_finite() {
                                return f64::NAN;
                            }
                            effective_backout(model, swt, phie, phit, vsh, phit_sh).0.clamp(0.0, 1.0)
                        }
                        SwModel::LinearDw => f64::NAN,
                    }
                };
                // U zone (deep): Rt against Rw.
                let phie_u = zs.u_water.iter().chain(&zs.u_hc).map(|&c| x[c]).sum::<f64>();
                if phie_u > 1e-6 {
                    if let Some(rt) = read_res(ct_tool_idx) {
                        let v_bw_u = zs.u_bw.iter().map(|&c| x[c]).sum::<f64>();
                        let sw = sw_of(rt, phie_u, fc.cw, fc.cbw_u, v_bw_u);
                        if sw.is_finite() {
                            set_group(&mut x, &zs.u_water, sw * phie_u);
                            set_group(&mut x, &zs.u_hc, (1.0 - sw) * phie_u);
                        }
                    }
                }
                // X zone (flushed): Rxo against Rmf — only with a real, zone-disjoint X/U split and an
                // X-zone HC (post_zones_disjoint keeps a shared-zone fluid from being scaled twice).
                if zs.has_split && post_zones_disjoint && !zs.x_hc.is_empty() {
                    let phie_x = zs.x_water.iter().chain(&zs.x_hc).map(|&c| x[c]).sum::<f64>();
                    if phie_x > 1e-6 {
                        if let Some(rxo) = read_res(cxo_tool_idx) {
                            let v_bw_x = zs.x_bw.iter().map(|&c| x[c]).sum::<f64>();
                            let sxo = sw_of(rxo, phie_x, fc.cmf, fc.cbw_x, v_bw_x);
                            if sxo.is_finite() {
                                set_group(&mut x, &zs.x_water, sxo * phie_x);
                                set_group(&mut x, &zs.x_hc, (1.0 - sxo) * phie_x);
                            }
                        }
                    }
                }
            }

            // Weighted RMS residual over the live tool rows only (σ units).
            let mut sse = 0.0;
            for (row, &bi) in a.iter().zip(&b).take(n_tool_rows) {
                let pred: f64 = row.iter().zip(&x).map(|(ai, xi)| ai * xi).sum();
                let d = pred - bi;
                sse += d * d;
            }
            let rerr = (sse / n_tool_rows as f64).sqrt();

            for c in 0..n {
                vol[c][i] = x[c] as f32;
            }
            recon[i] = rerr as f32;
            recon_sum += rerr;
            solved += 1;

            // Per-tool reconstruction: rebuild each live tool's reading from the solved volumes. Read
            // from the SAME per-sample A-row the solver fitted (`a[k]` = row·w, `live[k]` in step with
            // it) — not the constant-T static `rows[t]` — so a conductivity row rebuilt at this depth's
            // FTEMP temperature is honoured. `rec_native = (a[k]·x)/w` is the tool's SOLVE-domain value;
            // the σ-unit residual (rec_native − v)·w is exactly that tool's term of RECON. In the
            // constant-T path `a[k] = rows[t]·w`, so this is byte-identical to reconstructing from rows[t].
            if recon_qc {
                let rhob_i = rhob_col.map(|c| c[i] as f64);
                for (k, &(t, v, w)) in live.iter().enumerate() {
                    let pred_scaled: f64 = a[k].iter().zip(&x).map(|(ai, xi)| ai * xi).sum();
                    let rec_native = pred_scaled / w;
                    tool_dif[t][i] = ((rec_native - v) * w) as f32;
                    tool_rec[t][i] = recon_display(&tkind[t], rec_native, rhob_i) as f32;
                }
            }
        }

        // Derived output curves from the solved volumes.
        let sum_over = |idx: &[usize], i: usize| -> f32 { idx.iter().map(|&c| vol[c][i]).sum() };
        let make = |f: &dyn Fn(usize) -> f32| -> Vec<f32> {
            (0..ns).map(|i| if vol[0][i].is_finite() { f(i) } else { f32::NAN }).collect()
        };

        // --- Core calibration -------------------------------------------------
        // RECON only says the model reproduces the logs it was fitted to; core plugs are an
        // INDEPENDENT measurement. Plugs sit on their own sparse depths, so each ties to the
        // nearest solved sample within CORE_MATCH_TOL_M. A well with no core (or an all-NULL
        // column) simply leaves these None — nothing is reported as a zero.
        // A refusal is CARRIED, not swallowed into an empty plug list — `unwrap_or_default` made a
        // cross-datum core indistinguishable from a well that was never cored, and the dialog hides
        // the whole core section when every fit is blank, so the reason would have vanished.
        let mut core_note: Option<String> = None;
        let core_plugs = match crate::db::get_core_por_gd(&conn, well_id) {
            Ok(p) => p,
            Err(e) => {
                core_note = Some(e.to_string());
                Vec::new()
            }
        };
        // Validity gates, not just non-null: core φ must be a v/v FRACTION and ρg a rock density.
        // A φ column imported in percent (15.0, not 0.15) or a 999.25-style sentinel would otherwise
        // produce a confidently wrong RMS; dropping it reports "no fit" instead, which is honest.
        let por_plugs: Vec<(f32, f32)> = core_plugs
            .iter()
            .filter(|p| p.depth.is_finite() && p.cpor > 0.0 && p.cpor <= 1.0)
            .map(|p| (p.depth, p.cpor))
            .collect();
        let gd_plugs: Vec<(f32, f32)> = core_plugs
            .iter()
            .filter(|p| p.depth.is_finite() && p.cgd > 1.0 && p.cgd < 6.0)
            .map(|p| (p.depth, p.cgd))
            .collect();
        let mut core_phie = None;
        let mut core_phit = None;
        let mut core_gd = None;

        // Grain density implied by the solved SOLID volumes: ρg = Σ v·ρ / Σ v over the non-fluid
        // components (the same "fluid" test `zone_sets` uses). Routine core analysis measures ρg on
        // a cleaned, DRIED plug, so clay-bound water — a fluid component here — is correctly outside
        // the sum and the clay term is the dry-clay endpoint SandiMin already carries.
        if !gd_plugs.is_empty() {
            let solids: Vec<(usize, f64)> = req
                .components
                .iter()
                .enumerate()
                .filter(|(_, c)| !c.kind.eq_ignore_ascii_case("fluid"))
                .map(|(i, c)| (i, c.endpoints.get("RHOB").copied().unwrap_or(f64::NAN)))
                .collect();
            // Every solid needs a usable density endpoint, else the mixture density is undefined
            // and a partial sum would quietly bias ρg toward whichever minerals happened to have one.
            if !solids.is_empty() && solids.iter().all(|(_, r)| r.is_finite() && *r > 0.0) {
                let gd = make(&|i| {
                    let (mut num, mut den) = (0.0f64, 0.0f64);
                    for &(c, r) in &solids {
                        let v = (vol[c][i] as f64).max(0.0);
                        num += v * r;
                        den += v;
                    }
                    if den > 1e-6 {
                        (num / den) as f32
                    } else {
                        f32::NAN
                    }
                });
                core_gd = core_fit(&depth, &gd, &gd_plugs);
            }
        }

        let mut curves: Vec<(String, Vec<f32>)> = Vec::with_capacity(n + 10);
        for (name, values) in vol_names.iter().zip(&vol) {
            curves.push((name.clone(), values.clone()));
        }
        let has_u_fluids = !zs.u_fluids.is_empty();
        if has_u_fluids {
            let phie = make(&|i| sum_over(&zs.u_water, i) + sum_over(&zs.u_hc, i));
            let phit = make(&|i| sum_over(&zs.u_water, i) + sum_over(&zs.u_hc, i) + sum_over(&zs.u_bw, i));
            let swe = make(&|i| {
                let p = sum_over(&zs.u_water, i) + sum_over(&zs.u_hc, i);
                if p > 1e-6 { sum_over(&zs.u_water, i) / p } else { f32::NAN }
            });
            let swt = make(&|i| {
                let p = sum_over(&zs.u_water, i) + sum_over(&zs.u_hc, i) + sum_over(&zs.u_bw, i);
                if p > 1e-6 { (sum_over(&zs.u_water, i) + sum_over(&zs.u_bw, i)) / p } else { f32::NAN }
            });
            // Core φ against BOTH porosities — the drying protocol decides which one a plug should
            // match, so report the bracket instead of picking one for the analyst.
            if !por_plugs.is_empty() {
                core_phie = core_fit(&depth, &phie, &por_plugs);
                core_phit = core_fit(&depth, &phit, &por_plugs);
            }
            let produced: Vec<bool> = swe.iter().map(|value| value.is_finite()).collect();
            let method_flag = saturation_method_flag_curve(&prefix, model, &produced);
            curves.push((format!("{prefix}_PHIE"), phie));
            curves.push((format!("{prefix}_PHIT"), phit));
            curves.push((format!("{prefix}_SWE"), swe));
            curves.push((format!("{prefix}_SWT"), swt));
            curves.push(method_flag);
        }
        if zs.has_split {
            let sxot = make(&|i| {
                let p = sum_over(&zs.x_water, i) + sum_over(&zs.x_hc, i) + sum_over(&zs.x_bw, i);
                if p > 1e-6 { (sum_over(&zs.x_water, i) + sum_over(&zs.x_bw, i)) / p } else { f32::NAN }
            });
            curves.push((format!("{prefix}_SXOT"), sxot));
            if !zs.u_hc.is_empty() || !zs.x_hc.is_empty() {
                let moved = make(&|i| sum_over(&zs.u_hc, i) - sum_over(&zs.x_hc, i));
                curves.push((format!("{prefix}_MOVEDHC"), moved));
            }
        }
        if !zs.clays.is_empty() {
            let vsh = make(&|i| sum_over(&zs.clays, i) + sum_over(&zs.u_bw, i));
            curves.push((format!("{prefix}_VSH"), vsh));
        }
        // RECON = the incoherence (σ-weighted RMS residual over live tool rows; Quanti.Elan Eq 79).
        curves.push((format!("{prefix}_RECON"), recon));
        // Per-tool reconstruction QC decomposition (opt-in): rebuilt reading + σ-unit residual.
        if recon_qc {
            for (t, tool) in tools.iter().enumerate() {
                let tag = curve_token(&tool.key);
                curves.push((format!("{prefix}_{tag}_REC"), std::mem::take(&mut tool_rec[t])));
                curves.push((format!("{prefix}_{tag}_DIF"), std::mem::take(&mut tool_dif[t])));
            }
        }

        if out_names.is_empty() {
            out_names = curves.iter().map(|(n, _)| n.clone()).collect();
        }
        let refs: Vec<(&str, &[f32])> = curves.iter().map(|(n, v)| (n.as_str(), v.as_slice())).collect();
        let set_name= req
                .output_set
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(DEFAULT_SANDIMIN_SET)
                ;
        let mut inputs = req
            .tools
            .iter()
            .filter(|tool| !tool.curve.trim().is_empty())
            .map(|tool| (well_id.clone(),
            tool.key.clone(), tool.curve.clone()))
            .collect::<Vec<_>>();
        if let Some(curve) = req
            .ftemp_curve
            .as_deref()
            .map(str::trim)
            .filter(|curve| !curve.is_empty())
        {
            inputs.push((well_id.clone(), "FTEMP".into(),
            curve.to_string()));
        }
        let parameters = serde_json::json!({
                "components": req.components,
            "tools": req.tools,
                "output_prefix": prefix,
            "unity": req.unity,
            "fluid": req.fluid.as_ref().map(serde_json::to_value).transpose().ok().flatten().unwrap_or_else(|| serde_json::json!("ABSENT")),
            "ftemp_curve": req.ftemp_curve.clone().unwrap_or_else(|| "ABSENT".into()),
            "recon_qc": req.recon_qc,
                "sw_model": req.sw_model,
            "porosity_source": req.porosity_source,
            "enforce_porosity": req.enforce_porosity,
            "enforce_bndwat": req.enforce_bndwat,
            "enforce_water_mud": req.enforce_water_mud,
            "sigma_constraint": req.sigma_constraint,
        });
        let spec = crate::equations::complete_curve_run_spec(
            &conn,
            well_id,
            set_name,
            "sandimin",
            &req.custody,
            &inputs,
            req.input_set.as_deref(),
            parameters,
            crate::equations::AncestryZoneScope::WholeWell,
            &out_names,
        );
        let write_err = spec
            .and_then(|spec| {
                crate::equations::create_complete_log_set(&conn, well_id, &spec)
            .map(|(id, _)| id)
            })
            .and_then(|set_id| {
                crate::equations::write_computed_curves_with_ancestry(&conn, well_id, &depth, &refs, &set_id)})
            .err();
        if write_err.is_none() && has_u_fluids {
            let method_flag_name = format!("{prefix}_SW_METHOD");
            let _ = crate::db::declare_class_curves(
                &conn,
                well_id,
                &[method_flag_name],
                &format!("sandimin:{}", model.id()),
            );
        }
        if let Some(p) = progress {
            // A write failure is Failed; a solve that produced nothing is a Warned caveat; else Ok.
            let (state, msg) = if let Some(e) = &write_err {
                (crate::jobs::ItemState::Failed, Some(e.clone()))
            } else if solved == 0 {
                (crate::jobs::ItemState::Warned, Some("no solvable samples (too few live input logs)".to_string()))
            } else {
                (crate::jobs::ItemState::Ok, None)
            };
            p.finish_item(well_id, state, msg);
        }
        wells.push(SandiminWellResult {
            well_id: well_id.clone(),
            rows_solved: solved,
            mean_recon: if solved > 0 { (recon_sum / solved as f64) as f32 } else { f32::NAN },
            core_phie,
            core_phit,
            core_gd,
            core_note,
            error: write_err.or_else(|| (solved == 0).then(|| "no solvable samples (too few live input logs)".to_string())),
        });
    }

    SandiminResult {
        // SB-SAT-023: the applied Swb rule travels with the result, never implicit.
        swb_rule: post_solve.then(|| effective_backout_rule(model).to_string()),
        outputs: out_names,
        wells,
        dof,
        dof_note,
        error: None,
    }
}

/// Rebuilds a tool's measurement in its DISPLAY domain from the solved native prediction:
/// Plain tools are already physical; a conductivity row predicts C^(1/w) → resistivity = pred^−w;
/// a PEF row predicts U = Pe·ρe → PEF = U/ρe.
fn recon_display(kind: &TKind, native: f64, rhob: Option<f64>) -> f64 {
    match *kind {
        TKind::Plain => native,
        TKind::Cond(w, _) => {
            if native > 1e-12 {
                native.powf(-w)
            } else {
                f64::NAN
            }
        }
        TKind::Pef(_) => match rhob {
            Some(rb) if rb.is_finite() && rb > 0.0 => native / rho_e(rb),
            _ => f64::NAN,
        },
    }
}

fn is_cond_key(key: &str) -> bool {
    let k = key.trim().to_uppercase();
    k == "CT" || k == "CXO"
}

fn is_pef_key(key: &str) -> bool {
    key.trim().eq_ignore_ascii_case("PEF")
}

/// Litho-Density electron density from bulk density: inverse of ρₐ = 1.0704·ρₑ − 0.1883.
/// Shared with the legacy `multimin` module so both solvers convert Pe↔U with one relation
/// (divergent Pe physics between the two solvers is the hazard this centralises away).
pub(crate) fn rho_e(rhob: f64) -> f64 {
    (rhob + 0.1883) / 1.0704
}

/// Volumetric photoelectric factor U = Pe·ρₑ — the quantity that mixes linearly by
/// volume. Per-electron PEF does NOT, so a measured PEF reading is converted to U here
/// before it enters the linear system. `None` for a non-physical RHOB.
#[allow(dead_code)] // exercised by the pef_converts_to_u_before_mixing test; inline in the hot path
fn pef_to_u(pef: f64, rhob: f64) -> Option<f64> {
    (pef.is_finite() && rhob.is_finite() && rhob > 0.0).then(|| pef * rho_e(rhob))
}

/// How a tool's measurement enters the least-squares system.
#[derive(Clone, Copy)]
enum TKind {
    /// Endpoint response mixes linearly (RHOB, NPHI, DT, GR, U, …).
    Plain,
    /// Resistivity → conductivity^(1/w) transform (CT/CXO); carries w and whether it is the deep
    /// (CT, U-zone) tool — the latter selects the zone/water conductivity when an FTEMP curve rebuilds
    /// the row per sample.
    Cond(f64, bool),
    /// PEF → U = Pe·ρe conversion before mixing; carries the PEF-space σ.
    Pef(f64),
}

// ---------------------------------------------------------------------------
// Solver — bounded (0 ≤ v ≤ hi), optionally equality-constrained (cᵀv = 1)
// active-set least squares.
// ---------------------------------------------------------------------------

/// A^T (A v − b), the gradient of ½‖Av−b‖².
fn grad(a: &[Vec<f64>], b: &[f64], v: &[f64], n: usize) -> Vec<f64> {
    let m = a.len();
    let mut g = vec![0.0; n];
    for i in 0..m {
        let pred: f64 = (0..n).map(|c| a[i][c] * v[c]).sum();
        let r = pred - b[i];
        for (c, gc) in g.iter_mut().enumerate() {
            *gc += a[i][c] * r;
        }
    }
    g
}

#[derive(Clone, Copy, PartialEq)]
enum BState {
    Free,
    AtLo,
    AtHi,
}

/// min‖Av−b‖² s.t. 0 ≤ v ≤ hi and (optionally) cᵀv = 1.
/// Active-set: solve the KKT system on the free set (fixed components folded into the RHS),
/// line-search from the current feasible point to the first bound crossing, and release
/// bound components whose KKT multiplier has the wrong sign.
fn solve_bounded_lsq(
    a: &[Vec<f64>],
    b: &[f64],
    unity_c: Option<&[f64]>,
    hi: &[f64],
    n: usize,
) -> Option<Vec<f64>> {
    if a.is_empty() {
        return None;
    }
    // Feasible start: spread the unity budget over the unity components, capped by hi;
    // non-unity (X-zone fluid) components start mid-box.
    let mut v = vec![0.0f64; n];
    match unity_c {
        Some(c) => {
            let k = c.iter().filter(|&&ci| ci != 0.0).count().max(1);
            let mut budget = 1.0f64;
            let share = 1.0 / k as f64;
            for i in 0..n {
                if c[i] != 0.0 {
                    v[i] = share.min(hi[i]);
                    budget -= c[i] * v[i];
                }
            }
            // Distribute any remainder (from hi caps) over uncapped unity components.
            if budget.abs() > 1e-12 {
                for i in 0..n {
                    if c[i] != 0.0 && v[i] < hi[i] {
                        let add = (budget / c[i]).min(hi[i] - v[i]).max(0.0);
                        v[i] += add;
                        budget -= c[i] * add;
                        if budget.abs() < 1e-12 {
                            break;
                        }
                    }
                }
            }
            for i in 0..n {
                if c[i] == 0.0 {
                    v[i] = (0.25f64).min(hi[i]);
                }
            }
        }
        None => {
            for i in 0..n {
                v[i] = (0.25f64).min(hi[i]);
            }
        }
    }

    let mut state = vec![BState::Free; n];
    let mut solved_once = false;
    let max_outer = 8 * n + 12;

    for _ in 0..max_outer {
        let free: Vec<usize> = (0..n).filter(|&c| state[c] == BState::Free).collect();
        if free.is_empty() {
            return if solved_once { Some(v) } else { None };
        }

        // Contribution of components fixed at their upper bound.
        let fixed_hi: Vec<usize> = (0..n).filter(|&c| state[c] == BState::AtHi).collect();
        let m = a.len();
        let k = free.len();
        let with_unity = unity_c.is_some();
        let dim = k + usize::from(with_unity);
        let mut mat = vec![vec![0.0f64; dim]; dim];
        let mut rhs = vec![0.0f64; dim];
        // b' = b − A_H v_H
        let bprime: Vec<f64> = (0..m)
            .map(|i| b[i] - fixed_hi.iter().map(|&c| a[i][c] * hi[c]).sum::<f64>())
            .collect();
        for p in 0..k {
            for q in 0..k {
                let g: f64 = (0..m).map(|i| a[i][free[p]] * a[i][free[q]]).sum();
                mat[p][q] = 2.0 * g;
            }
            rhs[p] = 2.0 * (0..m).map(|i| a[i][free[p]] * bprime[i]).sum::<f64>();
        }
        if let Some(c) = unity_c {
            for p in 0..k {
                mat[p][k] = c[free[p]];
                mat[k][p] = c[free[p]];
            }
            rhs[k] = 1.0 - fixed_hi.iter().map(|&i| c[i] * hi[i]).sum::<f64>();
        }
        let sol = match solve_linear_opt(mat, rhs) {
            Some(s) if s.iter().all(|x| x.is_finite()) => s,
            _ => return if solved_once { Some(v) } else { None },
        };
        let mut u = vec![0.0f64; n];
        for (idx, &c) in free.iter().enumerate() {
            u[c] = sol[idx];
        }
        for &c in &fixed_hi {
            u[c] = hi[c];
        }
        let mu = if with_unity { sol[k] } else { 0.0 };

        let feasible = free.iter().all(|&c| u[c] >= -1e-12 && u[c] <= hi[c] + 1e-12);
        if feasible {
            for c in 0..n {
                v[c] = u[c].clamp(0.0, hi[c]);
            }
            solved_once = true;
            // KKT sign check on bound components: at-lo needs 2g+μc ≥ 0, at-hi needs ≤ 0.
            let g = grad(a, b, &v, n);
            let cvec = unity_c;
            let mut release = None;
            let mut worst = 1e-9;
            for c in 0..n {
                let cc = cvec.map(|cv| cv[c]).unwrap_or(0.0);
                let kkt = 2.0 * g[c] + mu * cc;
                match state[c] {
                    BState::AtLo if -kkt > worst => {
                        worst = -kkt;
                        release = Some(c);
                    }
                    BState::AtHi if kkt > worst => {
                        worst = kkt;
                        release = Some(c);
                    }
                    _ => {}
                }
            }
            match release {
                Some(c) => state[c] = BState::Free,
                None => return Some(v),
            }
        } else {
            // Step from feasible v toward u until the first free component hits a bound.
            let mut alpha: f64 = 1.0;
            for &c in &free {
                let d = u[c] - v[c];
                if d < -1e-18 && u[c] < 0.0 {
                    alpha = alpha.min(v[c] / -d);
                } else if d > 1e-18 && u[c] > hi[c] {
                    alpha = alpha.min((hi[c] - v[c]) / d);
                }
            }
            let mut any_fixed = false;
            for c in 0..n {
                v[c] += alpha * (u[c] - v[c]);
            }
            for &c in &free {
                if v[c] <= 1e-10 {
                    v[c] = 0.0;
                    state[c] = BState::AtLo;
                    any_fixed = true;
                } else if v[c] >= hi[c] - 1e-10 {
                    v[c] = hi[c];
                    state[c] = BState::AtHi;
                    any_fixed = true;
                }
            }
            if !any_fixed {
                return if solved_once { Some(v) } else { None };
            }
        }
    }
    if solved_once {
        Some(v)
    } else {
        None
    }
}

/// Gaussian elimination with partial pivoting. Returns None for a singular system.
fn solve_linear_opt(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let k = b.len();
    for col in 0..k {
        let mut pivot = col;
        for r in (col + 1)..k {
            if a[r][col].abs() > a[pivot][col].abs() {
                pivot = r;
            }
        }
        if a[pivot][col].abs() < 1e-15 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        for r in (col + 1)..k {
            let f = a[r][col] / a[col][col];
            for c in col..k {
                a[r][c] -= f * a[col][c];
            }
            b[r] -= f * b[col];
        }
    }
    let mut x = vec![0.0; k];
    for col in (0..k).rev() {
        let mut s = b[col];
        for c in (col + 1)..k {
            s -= a[col][c] * x[c];
        }
        x[col] = s / a[col][col];
    }
    Some(x)
}

// ---------------------------------------------------------------------------
// Endpoint library — Jauhar's default component library (owner adjudication
// DEC-078, 2026-08-19; custody history is recorded in docs/IP_PROVENANCE.md
// §2.2 and docs/takeover/DECISIONS.md — it is deliberately not restated here).
// Every value carries per-value provenance via `LibRow::src`: values the
// Schlumberger Log Interpretation Charts (2013 ed.) state in print are cited
// to their page; every other value is the owner's default. Values were
// verified against the book page by page on 2026-08-19 — a citation code may
// only be used where the printed page states the library's exact number.
// Display units: RHOB g/cc, NPHI v/v, DT us/ft, GR API, PEF b/e, U b/cc,
// THOR ppm, POTA %, URAN ppm, VP km/s, VS km/s, EPT ns/m, EATT dB/m, SIGMA c.u.
// CT/CXO endpoints are computed from the fluid properties at run time.
// ---------------------------------------------------------------------------

#[allow(dead_code)] // canonical tool-key ordering, referenced by docs/spec + future validation
pub const TOOL_KEYS: [&str; 14] =
    ["RHOB", "NPHI", "DT", "GR", "PEF", "U", "THOR", "POTA", "URAN", "VP", "VS", "EPT", "EATT", "SIGMA"];

struct LibRow {
    name: &'static str,
    kind: &'static str,
    zone: &'static str,
    fluid_type: &'static str,
    cec: f64,
    /// Wet-clay total porosity φ_clay (clays; the Wet-Clay-Porosity bound-water source).
    /// Owner default per DEC-078; custody history in docs/IP_PROVENANCE.md §2.2. 0 for non-clays.
    wcp: f64,
    max_vol: f64,
    /// [RHOB, NPHI, DT, GR, PEF, U, THOR, POTA, URAN, EPT, SIGMA]  (VP/VS derived from DT)
    v: [f64; 11],
    /// Per-value provenance code, one ASCII char per `v` slot (same order):
    /// `B` = stated in Schlumberger Log Interpretation Charts (2013), Appendix B pp. 279–280;
    /// `C` = stated in Appendix C p. 281; `A` = Appendix B states it as an approximate value;
    /// `W` = carried from chart Por-1 p. 212's stated fluid velocity (5,300 ft/s = 188.7 µs/ft);
    /// `J` = Jauhar's default (owner adjudication DEC-078) — the book does not state this number.
    /// A `B`/`C` code is only legal where the printed page states the exact library value; the
    /// owned test pins the complete matrix and spot-checks cited cells against the printed numbers.
    src: &'static str,
}

const fn m(name: &'static str, v: [f64; 11], src: &'static str) -> LibRow {
    LibRow { name, kind: "mineral", zone: "", fluid_type: "", cec: 0.0, wcp: 0.0, max_vol: 1.0, v, src }
}
const fn clay(name: &'static str, cec: f64, wcp: f64, v: [f64; 11], src: &'static str) -> LibRow {
    LibRow { name, kind: "clay", zone: "", fluid_type: "", cec, wcp, max_vol: 1.0, v, src }
}
const fn fl(name: &'static str, zone: &'static str, fluid_type: &'static str, v: [f64; 11], src: &'static str) -> LibRow {
    LibRow { name, kind: "fluid", zone, fluid_type, cec: 0.0, wcp: 0.0, max_vol: 0.5, v, src }
}

/// Expand a [`LibRow::src`] code into the full machine-readable source string (SB-MIN-009).
const SRC_APPENDIX_B: &str =
    "Schlumberger Log Interpretation Charts (2013 ed.), Appendix B Logging Tool Response in Sedimentary Minerals, pp. 279-280";
const SRC_APPENDIX_C: &str =
    "Schlumberger Log Interpretation Charts (2013 ed.), Appendix C Acoustic Characteristics of Common Formations and Fluids, p. 281";
const SRC_APPENDIX_B_APPROX: &str =
    "Schlumberger Log Interpretation Charts (2013 ed.), Appendix B pp. 279-280, stated as an approximate value";
const SRC_POR1_FLUID: &str =
    "Schlumberger Log Interpretation Charts (2013 ed.), chart Por-1 p. 212 stated fluid velocity 5,300 ft/s (= 188.7 us/ft, carried as 189)";
const SRC_OWNER: &str =
    "Jauhar default endpoint library, owner adjudication DEC-078 (2026-08-19); docs/takeover/DECISIONS.md";
const SRC_DERIVED_FROM_DT: &str =
    "derived at library build: VP = 304.8/DT, VS = VP/1.7 for non-fluids (0 for fluids); provenance follows DT";

fn endpoint_source_text(code: u8) -> &'static str {
    match code {
        b'B' => SRC_APPENDIX_B,
        b'C' => SRC_APPENDIX_C,
        b'A' => SRC_APPENDIX_B_APPROX,
        b'W' => SRC_POR1_FLUID,
        _ => SRC_OWNER,
    }
}

/// Jauhar's default component library (owner adjudication DEC-078), in the dropdown order of
/// his screenshot. Values are unchanged by the custody work; each row's `src` string records,
/// slot by slot, whether the 2013 chartbook states the number in print or the value is his own.
#[rustfmt::skip]
const LIB: &[LibRow] = &[
    //                       RHOB   NPHI    DT     GR    PEF     U    THOR  POTA  URAN   EPT  SIGMA
    m("Calcite",           [2.71,  0.000,  47.5, 11.0,  5.08, 13.8,  0.0,  0.00,  1.4,  9.1,  7.4], "BBJJJBJJJBJ"),
    m("Quartz",            [2.65, -0.050,  55.5,  1.0,  1.81,  4.8,  0.0,  0.00,  0.1,  7.2,  4.7], "JJJJJBJJJBJ"),
    m("Dolomite",          [2.85,  0.025,  43.5,  8.0,  3.14,  9.0,  0.1,  0.00,  0.9,  8.7,  6.9], "BJCJJBJJJBJ"),
    m("Orthoclase",        [2.57, -0.010,  69.0,171.0,  2.86,  8.7,  1.1, 10.21,  0.4,  7.6, 15.3], "JJBJJJJJJJJ"),
    m("Albite",            [2.60, -0.005,  49.0,  8.0,  1.68,  5.6,  0.0,  0.50,  0.0,  7.6, 11.4], "JJBJJJJJJJJ"),
    m("Anhydrite",         [2.98, -0.020,  50.0,  5.0,  5.05, 14.95, 0.2,  0.00,  0.4,  8.4, 12.0], "BBBJJJJJJBB"),
    m("Halite",            [2.04, -0.030,  67.0,  5.0,  4.65,  9.7,  0.2,  0.00,  0.0,  8.2,750.0], "BBBJJJJJJJJ"),
    m("Gypsum",            [2.35,  0.540,  52.0,  5.0,  3.99,  9.46, 0.0,  0.00,  0.3,  6.8, 20.0], "BJBJJJJJJBJ"),
    m("Pyrite",            [4.99,  0.000,  39.2,  5.0, 16.97, 82.0,  0.0,  0.00,  0.0,  0.0, 90.0], "BJBJJJJJJJB"),
    m("Siderite",          [3.88,  0.180,  44.0,  6.0, 14.70, 72.0,  0.4,  0.00,  0.5,  8.9, 54.2], "JJJJJJJJJJJ"),
    m("Muscovite",         [2.85,  0.240,  49.0,130.0,  2.40, 11.5,  0.0,  7.80,  0.7,  8.9, 95.3], "JJBJBJJJJJJ"),
    m("Biotite",           [3.04,  0.130,  50.8,127.0,  6.27, 21.6,  1.5,  7.20,  0.7,  7.8, 54.1], "JJBJJJJJJJJ"),
    clay("Glauconite", 0.20, 0.156, [2.96, 0.410,  49.4,150.0,  5.32, 16.5,  2.8,  5.60,  5.1, 12.0, 89.6], "JJJJJJJJJJJ"),
    clay("Kaolinite",  0.10, 0.058, [2.62, 0.451,  85.3,104.0,  1.83,  5.38,18.9,  0.08,  3.1,  8.0, 20.1], "BJJJJJJJJAJ"),
    clay("Chlorite",   0.15, 0.101, [2.81, 0.520,  85.3, 56.0,  6.30, 21.7, 11.0,  0.67,  3.5,  8.0, 43.7], "JBJJBJJJJAJ"),
    clay("Illite",     0.25, 0.104, [2.78, 0.247,  85.3,160.0,  4.00, 11.12,12.3,  4.48,  4.8,  8.0, 40.6], "JJJJJJJJJAJ"),
    clay("Montmorillonite",1.0, 1.0,[2.63,0.218, 85.3,168.0,  2.70,  7.61,20.6,  0.58,  7.1,  8.0, 20.2], "JJJJJJJJJAJ"),
    clay("Clay",       0.00, 0.120, [2.65, 0.350, 100.0,152.0,  3.50, 10.0,  6.0,  2.00, 12.0,  8.0, 30.0], "JJJJJJJJJJJ"),
    m("Coal",              [1.19,  0.520, 160.0, 10.0,  0.20,  0.24, 0.0,  0.00,  0.0,  0.0,  0.0], "BBBJBBJJJJJ"),
    m("Kerogen",           [1.10,  0.600, 150.0,100.0,  0.24,  0.26, 0.0,  0.00, 10.0,  0.0,  0.0], "JJJJJJJJJJJ"),
    fl("Water Sxo", "X", "water",       [1.00, 1.00, 189.0, 0.0, 0.36, 0.40, 0.0, 0.0, 0.0, 29.0, 50.0], "JJWJJJJJJJJ"),
    fl("Water Sw",  "U", "water",       [1.00, 1.00, 189.0, 0.0, 0.36, 0.40, 0.0, 0.0, 0.0, 29.0, 50.0], "JJWJJJJJJJJ"),
    fl("BoundWater", "", "bound_water", [1.00, 1.00, 189.0, 0.0, 0.36, 0.39, 0.0, 0.0, 0.0, 30.0, 50.0], "JJWJJJJJJJJ"),
    fl("Oil Sxo",   "X", "oil",         [0.80, 1.00, 189.0, 0.0, 0.12, 0.11, 0.0, 0.0, 0.0,  5.0, 21.0], "JJJJJJJJJJJ"),
    fl("Oil Sw",    "U", "oil",         [0.80, 1.00, 189.0, 0.0, 0.12, 0.11, 0.0, 0.0, 0.0,  5.0, 21.0], "JJJJJJJJJJJ"),
    fl("Gas Sxo",   "X", "gas",         [0.20, 0.44, 250.0, 0.0, 0.09, 0.02, 0.0, 0.0, 0.0,  3.3,  5.0], "JJJJJJJJJJJ"),
    fl("Gas Sw",    "U", "gas",         [0.20, 0.44, 250.0, 0.0, 0.09, 0.02, 0.0, 0.0, 0.0,  3.3,  5.0], "JJJJJJJJJJJ"),
];

pub fn sandimin_library() -> Vec<Component> {
    LIB.iter()
        .map(|r| {
            let mut endpoints: HashMap<String, f64> = HashMap::new();
            let mut endpoint_sources: HashMap<String, String> = HashMap::new();
            let [rhob, nphi, dt, gr, pef, u, thor, pota, uran, ept, sigma] = r.v;
            let is_fluid = r.kind == "fluid";
            let vp = if dt > 0.0 { 304.8 / dt } else { 0.0 };
            let vs = if is_fluid { 0.0 } else { vp / 1.7 };
            for (k, val) in [
                ("RHOB", rhob),
                ("NPHI", nphi),
                ("DT", dt),
                ("GR", gr),
                ("PEF", pef),
                ("U", u),
                ("THOR", thor),
                ("POTA", pota),
                ("URAN", uran),
                ("VP", (vp * 100.0).round() / 100.0),
                ("VS", (vs * 100.0).round() / 100.0),
                ("EPT", ept),
                ("EATT", 0.0),
                ("SIGMA", sigma),
            ] {
                endpoints.insert(k.to_string(), val);
            }
            // Per-value provenance (SB-MIN-009): the 11 stored slots expand their row code;
            // VP/VS record the derivation; EATT's structural zero and the two row scalars
            // (CEC, WCP) are the owner's defaults per DEC-078.
            let codes = r.src.as_bytes();
            debug_assert_eq!(codes.len(), 11, "one provenance code per stored slot");
            for (slot, k) in ["RHOB", "NPHI", "DT", "GR", "PEF", "U", "THOR", "POTA", "URAN", "EPT", "SIGMA"]
                .iter()
                .enumerate()
            {
                endpoint_sources
                    .insert(k.to_string(), endpoint_source_text(codes[slot]).to_string());
            }
            for k in ["VP", "VS"] {
                endpoint_sources.insert(k.to_string(), SRC_DERIVED_FROM_DT.to_string());
            }
            for k in ["EATT", "CEC", "WCP"] {
                endpoint_sources.insert(k.to_string(), SRC_OWNER.to_string());
            }
            Component {
                name: r.name.to_string(),
                kind: r.kind.to_string(),
                zone: r.zone.to_string(),
                fluid_type: r.fluid_type.to_string(),
                endpoints,
                endpoint_sources,
                cec: r.cec,
                wet_clay_porosity: r.wcp,
                max_vol: r.max_vol,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lib_get(name: &str) -> Component {
        sandimin_library().into_iter().find(|c| c.name == name).unwrap()
    }

    /// CORRECTNESS — SB-MIN-T09, discharged for SB-CORE-005 under DEC-078 (2026-08-19):
    /// the endpoint library is Jauhar's default library, and every value carries a
    /// machine-readable source. A chartbook citation is only legal where the printed page
    /// states the library's exact number — verified page by page against the owner's copy
    /// of the 2013 edition on 2026-08-19 (Appendix B pp. 279-280, Appendix C p. 281,
    /// chart Por-1 p. 212). The complete 27-row provenance matrix is pinned by
    /// set-equality, so a silent reclassification in either direction fails here by name;
    /// custody history stays recorded in docs/IP_PROVENANCE.md §2.2, never in shipped
    /// source strings.
    #[test]
    fn every_shipped_endpoint_value_carries_a_source_and_a_chartbook_citation_appears_only_where_the_printed_page_states_the_value(
    ) {
        // A — completeness: every endpoint of every component has a non-empty source,
        // and the two row scalars carry their own entries.
        for component in sandimin_library() {
            for key in component.endpoints.keys() {
                let source = component.endpoint_sources.get(key).map(String::as_str).unwrap_or("");
                assert!(
                    !source.trim().is_empty(),
                    "{}.{key} ships without a source",
                    component.name
                );
            }
            for scalar in ["CEC", "WCP"] {
                assert!(
                    component
                        .endpoint_sources
                        .get(scalar)
                        .is_some_and(|source| !source.trim().is_empty()),
                    "{}.{scalar} ships without a source",
                    component.name
                );
            }
        }

        // B — the complete provenance matrix, pinned by set-equality. One line per row,
        // one code per stored slot (RHOB NPHI DT GR PEF U THOR POTA URAN EPT SIGMA).
        let expected: std::collections::BTreeMap<&str, &str> = [
            ("Calcite", "BBJJJBJJJBJ"),
            ("Quartz", "JJJJJBJJJBJ"),
            ("Dolomite", "BJCJJBJJJBJ"),
            ("Orthoclase", "JJBJJJJJJJJ"),
            ("Albite", "JJBJJJJJJJJ"),
            ("Anhydrite", "BBBJJJJJJBB"),
            ("Halite", "BBBJJJJJJJJ"),
            ("Gypsum", "BJBJJJJJJBJ"),
            ("Pyrite", "BJBJJJJJJJB"),
            ("Siderite", "JJJJJJJJJJJ"),
            ("Muscovite", "JJBJBJJJJJJ"),
            ("Biotite", "JJBJJJJJJJJ"),
            ("Glauconite", "JJJJJJJJJJJ"),
            ("Kaolinite", "BJJJJJJJJAJ"),
            ("Chlorite", "JBJJBJJJJAJ"),
            ("Illite", "JJJJJJJJJAJ"),
            ("Montmorillonite", "JJJJJJJJJAJ"),
            ("Clay", "JJJJJJJJJJJ"),
            ("Coal", "BBBJBBJJJJJ"),
            ("Kerogen", "JJJJJJJJJJJ"),
            ("Water Sxo", "JJWJJJJJJJJ"),
            ("Water Sw", "JJWJJJJJJJJ"),
            ("BoundWater", "JJWJJJJJJJJ"),
            ("Oil Sxo", "JJJJJJJJJJJ"),
            ("Oil Sw", "JJJJJJJJJJJ"),
            ("Gas Sxo", "JJJJJJJJJJJ"),
            ("Gas Sw", "JJJJJJJJJJJ"),
        ]
        .into_iter()
        .collect();
        let actual: std::collections::BTreeMap<&str, &str> =
            LIB.iter().map(|row| (row.name, row.src)).collect();
        assert_eq!(actual, expected, "the shipped provenance matrix moved");

        // C — a citation code means the printed page states the exact library number.
        // Coal is the book's Lignite row; Dolomite's DT is Appendix C; the water rows'
        // 189 us/ft carries Por-1's stated 5,300 ft/s.
        let coal = lib_get("Coal");
        for (key, printed) in
            [("RHOB", 1.19), ("NPHI", 0.52), ("DT", 160.0), ("PEF", 0.20), ("U", 0.24)]
        {
            assert_eq!(coal.endpoints[key], printed, "Coal.{key} drifted from the printed value");
            assert!(
                coal.endpoint_sources[key].contains("Appendix B"),
                "Coal.{key} lost its citation: {}",
                coal.endpoint_sources[key]
            );
        }
        let dolomite = lib_get("Dolomite");
        assert_eq!(dolomite.endpoints["DT"], 43.5);
        assert!(dolomite.endpoint_sources["DT"].contains("Appendix C"));
        let anhydrite = lib_get("Anhydrite");
        assert_eq!(anhydrite.endpoints["SIGMA"], 12.0);
        assert!(anhydrite.endpoint_sources["SIGMA"].contains("Appendix B"));
        let water = lib_get("Water Sxo");
        assert_eq!(water.endpoints["DT"], 189.0);
        assert!(
            water.endpoint_sources["DT"].contains("Por-1")
                && water.endpoint_sources["DT"].contains("5,300"),
            "the water slowness must cite the chart's stated velocity: {}",
            water.endpoint_sources["DT"]
        );
        let illite = lib_get("Illite");
        assert_eq!(illite.endpoints["EPT"], 8.0);
        assert!(
            illite.endpoint_sources["EPT"].contains("approximate"),
            "a value the book states only approximately must say so: {}",
            illite.endpoint_sources["EPT"]
        );

        // D — near-miss values stay the owner's: the book prints 5.1 / 2.64 / 754 where
        // the library carries 5.08 / 2.65 / 750, so citing it would be false custody.
        for (name, key) in [("Calcite", "PEF"), ("Quartz", "RHOB"), ("Halite", "SIGMA")] {
            let source = &lib_get(name).endpoint_sources[key];
            assert!(
                source.contains("DEC-078") && !source.contains("Appendix"),
                "{name}.{key} must stay owner-attributed, not book-cited: {source}"
            );
        }

        // E — derived slots record their derivation and follow DT.
        assert!(lib_get("Quartz").endpoint_sources["VP"].contains("follows DT"));
    }

    /// Weighted rows for a set of components over plain (non-conductivity) tools.
    fn weighted(comps: &[&Component], keys: &[&str], sigmas: &[f64], meas: &[f64]) -> (Vec<Vec<f64>>, Vec<f64>) {
        let mut a = Vec::new();
        let mut b = Vec::new();
        for (t, key) in keys.iter().enumerate() {
            let w = 1.0 / sigmas[t];
            let row: Vec<f64> = comps
                .iter()
                .map(|c| {
                    if c.kind == "fluid" && c.zone == "U" {
                        0.0
                    } else {
                        c.endpoints[&key.to_string()] * w
                    }
                })
                .collect();
            a.push(row);
            b.push(meas[t] * w);
        }
        (a, b)
    }

    fn unity_of(comps: &[&Component]) -> Vec<f64> {
        comps
            .iter()
            .map(|c| if c.kind == "fluid" && c.zone == "X" { 0.0 } else { 1.0 })
            .collect()
    }

    /// T-ADV-17. The Output prefix is free text, so nothing stops a re-run being typed `mm` after
    /// the first was `MM`. Both halves of the fix are exercised together, because either one
    /// alone leaves the bug: `run_sandimin` upper-cases the prefix (`sandimin.rs:1202`) and the
    /// computed-curve write DELETEs on `upper(curve_name)` (`equations.rs:623`).
    ///
    /// Without them a re-run writes `mm_PHIE` beside the untouched `MM_PHIE`. Readers resolve
    /// curve names case-insensitively, so the stale row can win: a plot, a module input or a
    /// report reads the FIRST run's answer while the catalog shows a fresh run at a bumped
    /// version. Nothing about that looks wrong, which is why the second run's numbers have to be
    /// genuinely different here — a re-run producing the same values could not tell a live row
    /// from a shadow.
    #[test]
    fn a_re_run_under_a_lowercase_prefix_leaves_no_shadow_rows() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-MM17", None, None, None).unwrap();
        let ids = wid.to_string();

        // Same forward-modelled quartz/illite/water well the recon test uses.
        let q = lib_get("Quartz");
        let ill = lib_get("Illite");
        let mut wat = lib_get("Water Sxo");
        wat.zone = String::new();
        let ep = |c: &Component, k: &str| c.endpoints[k];
        let n = 16usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let (mut gr, mut nphi, mut rhob, mut dt) = (vec![], vec![], vec![], vec![]);
        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let (vi, vw) = (0.10 + 0.25 * t, 0.10 + 0.15 * t);
            let vq = (1.0 - vi - vw).max(0.0);
            let s = vq + vi + vw;
            let (vq, vi, vw) = (vq / s, vi / s, vw / s);
            let mix = |k: &str| vq * ep(&q, k) + vi * ep(&ill, k) + vw * ep(&wat, k);
            gr.push(mix("GR") as f32);
            nphi.push(mix("NPHI") as f32);
            rhob.push(mix("RHOB") as f32);
            dt.push(mix("DT") as f32);
        }
        crate::db::insert_standard_curves(
            &conn, wid, depth.clone(), gr, vec![2.0f32; n], nphi, rhob, dt, vec![f32::NAN; n],
        )
        .unwrap();
        let db = Mutex::new(conn);

        let run = |prefix: &str, comps: Vec<Component>| -> SandiminResult {
            run_sandimin(
                &db,
                &SandiminRequest {
                    input_set: None,
                    output_set: None,
                    custody: crate::workflow::test_run_custody(),
                    components: comps,
                    tools: vec![
                        ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.03 },
                        ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.03 },
                        ToolSpec { key: "DT".into(), curve: "DT".into(), sigma: 2.0 },
                        ToolSpec { key: "GR".into(), curve: "GR".into(), sigma: 6.0 },
                    ],
                    apply_well_ids: vec![ids.clone()],
                    output_prefix: prefix.into(),
                    unity: true,
                    fluid: None,
                    ftemp_curve: None,
                    recon_qc: false,
                    sw_model: SwModel::LinearDw,
                    porosity_source: PorositySource::Cec,
                    enforce_porosity: true,
                    enforce_bndwat: true,
                    enforce_water_mud: true,
                    sigma_constraint: 0.01,
                },
                None,
            )
        };
        let phie = || -> Vec<f32> {
            let c = db.lock().unwrap();
            fetch_curve_frame(&c, &ids, &["MM_PHIE".to_string()]).unwrap().1["MM_PHIE"].clone()
        };
        let mean = |v: &[f32]| {
            let f: Vec<f32> = v.iter().copied().filter(|x| x.is_finite()).collect();
            f.iter().sum::<f32>() / f.len().max(1) as f32
        };

        // First run, prefix typed as MM.
        let first = run("MM", vec![q.clone(), ill.clone(), wat.clone()]);
        assert!(first.error.is_none(), "err={:?}", first.error);
        let first_phie = mean(&phie());
        assert!(first_phie.is_finite() && first_phie > 0.0, "first run produced no porosity");

        // Re-run with a LOWERCASE prefix and a changed endpoint, so the answer really moves.
        let mut ill_wet = ill.clone();
        *ill_wet.endpoints.get_mut("NPHI").unwrap() += 0.10;
        let second = run("mm", vec![q, ill_wet, wat]);
        assert!(second.error.is_none(), "err={:?}", second.error);

        // The outputs are named in UPPERCASE — the prefix was canonicalized, not taken as typed.
        assert!(
            second.outputs.iter().any(|o| o == "MM_PHIE"),
            "a lowercase prefix must still write MM_*: {:?}",
            second.outputs
        );
        assert!(
            !second.outputs.iter().any(|o| o.starts_with("mm_")),
            "no output may carry the typed casing: {:?}",
            second.outputs
        );

        {
            let c = db.lock().unwrap();
            // Not one lowercase row anywhere — the DELETE reclaimed by upper(), and the write
            // stored the canonical name.
            let lower: i64 = c
                .query_row(
                    "SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1 AND curve_name LIKE 'mm!_%' ESCAPE '!'",
                    duckdb::params![&ids],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(lower, 0, "a lowercase shadow curve survived the re-run");

            // Exactly one row per depth for the curve a reader would ask for. Two rows at one
            // depth is the shadow, and a case-insensitive reader can return either.
            let dupes: i64 = c
                .query_row(
                    "SELECT COALESCE(MAX(k), 0) FROM (
                       SELECT COUNT(*) AS k FROM computed_curves
                       WHERE well_id = ?1 AND upper(curve_name) = 'MM_PHIE' GROUP BY depth)",
                    duckdb::params![&ids],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(dupes, 1, "MM_PHIE has more than one row at some depth");
        }

        // And what a reader gets is the SECOND run's answer, not the first's.
        let second_phie = mean(&phie());
        assert!(
            (second_phie - first_phie).abs() > 1e-4,
            "the fixture must move between runs or this proves nothing: {first_phie} then {second_phie}"
        );
        let live = second.wells[0].mean_recon;
        assert!(live.is_finite(), "the re-run reported no reconstruction");
    }

    #[test]
    fn recon_qc_emits_per_tool_curves_and_flags_endpoint_error() {
        // Forward-model a 24-sample quartz/illite/water well from the library's own endpoints, so a
        // clean solve reconstructs the logs exactly (incoherence ~0) and a wrong endpoint inflates it.
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "MM-RECON", None, None, None).unwrap();
        let ids = wid.to_string();
        let q = lib_get("Quartz");
        let ill = lib_get("Illite");
        let mut wat = lib_get("Water Sxo");
        wat.zone = String::new(); // shared water: in unity + seen by every tool
        let ep = |c: &Component, k: &str| c.endpoints[k];
        let n = 24usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let (mut gr, mut nphi, mut rhob, mut dt) = (vec![], vec![], vec![], vec![]);
        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let vi = 0.05 + 0.35 * (t * 3.14).sin().powi(2);
            let vw = 0.05 + 0.20 * t;
            let vq = (1.0 - vi - vw).max(0.0);
            let s = vq + vi + vw;
            let (vq, vi, vw) = (vq / s, vi / s, vw / s);
            let mix = |k: &str| vq * ep(&q, k) + vi * ep(&ill, k) + vw * ep(&wat, k);
            gr.push(mix("GR") as f32);
            nphi.push(mix("NPHI") as f32);
            rhob.push(mix("RHOB") as f32);
            dt.push(mix("DT") as f32);
        }
        crate::db::insert_standard_curves(
            &conn, wid, depth.clone(), gr, vec![2.0f32; n], nphi, rhob.clone(), dt, vec![f32::NAN; n],
        )
        .unwrap();
        let db = Mutex::new(conn);

        let tools = || {
            vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.03 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.03 },
                ToolSpec { key: "DT".into(), curve: "DT".into(), sigma: 2.0 },
                ToolSpec { key: "GR".into(), curve: "GR".into(), sigma: 6.0 },
            ]
        };
        let run = |comps: Vec<Component>, prefix: &str| -> SandiminResult {
            run_sandimin(
                &db,
                &SandiminRequest {
                    input_set: None,
                    output_set: None,
                    custody: crate::workflow::test_run_custody(),
                    components: comps,
                    tools: tools(),
                    apply_well_ids: vec![ids.clone()],
                    output_prefix: prefix.into(),
                    unity: true,
                    fluid: None,
                    ftemp_curve: None,
                    recon_qc: true,
                    sw_model: SwModel::LinearDw,
                    porosity_source: PorositySource::Cec,
                    enforce_porosity: true,
                    enforce_bndwat: true,
                    enforce_water_mud: true,
                    sigma_constraint: 0.01,
                },
                None,
            )
        };
        let read = |names: &[&str]| -> HashMap<String, Vec<f32>> {
            let c = db.lock().unwrap();
            fetch_curve_frame(&c, &ids, &names.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap().1
        };
        let rms = |v: &[f32]| {
            let fin: Vec<f32> = v.iter().copied().filter(|x| x.is_finite()).collect();
            (fin.iter().map(|x| x * x).sum::<f32>() / fin.len().max(1) as f32).sqrt()
        };

        // Clean model: 4 tools + unity − 3 components = 2 DOF, so RECON is a real signal.
        let clean = run(vec![q.clone(), ill.clone(), wat.clone()], "MM");
        assert!(clean.error.is_none(), "err={:?}", clean.error);
        assert_eq!(clean.dof, 2, "dof = 4 tools + unity − 3 comps");
        assert!(clean.dof_note.is_none());
        let mean_clean = clean.wells[0].mean_recon;
        assert!(mean_clean < 0.1, "a perfect forward model should reconstruct exactly, incoherence={mean_clean}");
        for name in ["MM_RHOB_REC", "MM_RHOB_DIF", "MM_NPHI_REC", "MM_DT_REC", "MM_GR_REC"] {
            assert!(clean.outputs.iter().any(|o| o == name), "missing output {name}; got {:?}", clean.outputs);
        }
        // Reconstructed RHOB recovers the measured RHOB.
        let cols = read(&["MM_RHOB_REC"]);
        let maxdiff = cols["MM_RHOB_REC"]
            .iter()
            .zip(&rhob)
            .filter(|(r, _)| r.is_finite())
            .map(|(r, m)| (r - m).abs())
            .fold(0.0f32, f32::max);
        assert!(maxdiff < 0.02, "reconstructed RHOB should match measured, max diff {maxdiff}");

        // Inject a wrong illite density (+0.4 g/cc) → incoherence rises and the density residual is real.
        let mut ill_bad = ill.clone();
        *ill_bad.endpoints.get_mut("RHOB").unwrap() += 0.4;
        let bad = run(vec![q.clone(), ill_bad, wat.clone()], "MB");
        assert!(bad.error.is_none(), "err={:?}", bad.error);
        let mean_bad = bad.wells[0].mean_recon;
        assert!(
            mean_bad > mean_clean * 3.0 + 0.1,
            "an endpoint error should inflate incoherence: clean {mean_clean}, bad {mean_bad}"
        );
        assert!(rms(&read(&["MB_RHOB_DIF"])["MB_RHOB_DIF"]) > 0.1, "the density misfit should show in MB_RHOB_DIF");
    }

    #[test]
    fn dof_note_set_when_exactly_determined() {
        // 3 components, 2 tools + unity = 3 equations → dof 0 → a note, and RECON ~0 regardless.
        let q = lib_get("Quartz");
        let ill = lib_get("Illite");
        let mut wat = lib_get("Water Sxo");
        wat.zone = String::new();
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "MM-DOF", None, None, None).unwrap();
        let n = 6usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        crate::db::insert_standard_curves(
            &conn, wid, depth, vec![40.0; n], vec![2.0; n], vec![0.2; n], vec![2.45; n], vec![80.0; n], vec![f32::NAN; n],
        )
        .unwrap();
        let db = Mutex::new(conn);
        let res = run_sandimin(
            &db,
            &SandiminRequest {
                input_set: None,
                output_set: None,
                custody: crate::workflow::test_run_custody(),
                components: vec![q, ill, wat],
                tools: vec![
                    ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.03 },
                    ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.03 },
                ],
                apply_well_ids: vec![wid.to_string()],
                output_prefix: "MM".into(),
                unity: true,
                fluid: None,
                ftemp_curve: None,
                recon_qc: false,
                sw_model: SwModel::LinearDw,
                porosity_source: PorositySource::Cec,
                enforce_porosity: true,
                enforce_bndwat: true,
                enforce_water_mud: true,
                sigma_constraint: 0.01,
            },
            None,
        );
        assert!(res.error.is_none(), "err={:?}", res.error);
        assert_eq!(res.dof, 0);
        assert!(res.dof_note.is_some(), "exactly-determined model should carry a dof note");
    }

    /// T-RT-18, carried onto the module that actually ships.
    ///
    /// The audit's legacy-multimin finding was that RECON_ERR reads ~0 whenever the system is
    /// exactly determined — the everyday no-PEF case — so a mis-parameterised model passes QC
    /// while its volumes are wrong. Legacy `multimin` is retired and blocked, so that instance is
    /// gone; the question that survives is whether SandiMin inherited it.
    ///
    /// It did — it cannot not, it is linear algebra: with as many equations as components the
    /// solve reproduces the measurements exactly whatever the endpoints are, so the residual
    /// carries no information about them. What SandiMin adds is that it SAYS so, via `dof_note`.
    /// `dof_note_set_when_exactly_determined` already checks the note appears. What was never
    /// asserted is the reason it has to be there, which is the whole of T-RT-18: the model is
    /// wrong, the volumes move, and RECON does not budge.
    ///
    /// The over-determined control is what makes this a statement about the DOF rather than about
    /// this particular endpoint — `recon_qc_emits_per_tool_curves_and_flags_endpoint_error` shows
    /// the same +0.4 g/cc error is caught loudly at dof 2.
    #[test]
    fn an_exactly_determined_model_hides_a_wrong_endpoint_and_only_the_dof_note_says_so() {
        let q = lib_get("Quartz");
        let ill = lib_get("Illite");
        let mut wat = lib_get("Water Sxo");
        wat.zone = String::new();

        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "MM-BLIND", None, None, None).unwrap();
        let n = 8usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        crate::db::insert_standard_curves(
            &conn, wid, depth, vec![40.0; n], vec![2.0; n], vec![0.2; n], vec![2.45; n], vec![80.0; n],
            vec![f32::NAN; n],
        )
        .unwrap();
        let db = Mutex::new(conn);

        // Two tools + unity against three components = exactly determined. This is the ordinary
        // one-log-missing case the audit named, not a contrived one.
        let square = || vec![
            ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.03 },
            ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.03 },
        ];
        let over = || {
            let mut t = square();
            t.push(ToolSpec { key: "DT".into(), curve: "DT".into(), sigma: 2.0 });
            t.push(ToolSpec { key: "GR".into(), curve: "GR".into(), sigma: 6.0 });
            t
        };
        let run = |comps: Vec<Component>, tools: Vec<ToolSpec>, prefix: &str| -> SandiminResult {
            run_sandimin(
                &db,
                &SandiminRequest {
                    input_set: None,
                    output_set: None,
                    custody: crate::workflow::test_run_custody(),
                    components: comps,
                    tools,
                    apply_well_ids: vec![wid.to_string()],
                    output_prefix: prefix.into(),
                    unity: true,
                    fluid: None,
                    ftemp_curve: None,
                    recon_qc: false,
                    sw_model: SwModel::LinearDw,
                    porosity_source: PorositySource::Cec,
                    enforce_porosity: true,
                    enforce_bndwat: true,
                    enforce_water_mud: true,
                    sigma_constraint: 0.01,
                },
                None,
            )
        };
        let mean_vol = |name: &str| -> f64 {
            let c = db.lock().unwrap();
            let cols = fetch_curve_frame(&c, &wid.to_string(), &[name.to_string()]).unwrap().1;
            let fin: Vec<f32> = cols[name].iter().copied().filter(|v| v.is_finite()).collect();
            assert!(!fin.is_empty(), "{name} produced no finite samples");
            fin.iter().map(|v| *v as f64).sum::<f64>() / fin.len() as f64
        };

        // The same endpoint error the over-determined test uses: illite 0.4 g/cc too dense.
        let mut ill_bad = ill.clone();
        *ill_bad.endpoints.get_mut("RHOB").unwrap() += 0.4;

        // --- Exactly determined: the model is wrong and the QC cannot tell. ---
        let good = run(vec![q.clone(), ill.clone(), wat.clone()], square(), "SG");
        assert!(good.error.is_none(), "err={:?}", good.error);
        assert_eq!(good.dof, 0);
        let vol_good = mean_vol("VOL_ILLITE");

        let bad = run(vec![q.clone(), ill_bad.clone(), wat.clone()], square(), "SB");
        assert!(bad.error.is_none(), "err={:?}", bad.error);
        assert_eq!(bad.dof, 0);
        let vol_bad = mean_vol("VOL_ILLITE");

        // The answer moved — this is a materially different clay volume.
        let moved = (vol_bad - vol_good).abs();
        assert!(
            moved > 0.02,
            "the wrong endpoint must actually change the answer, else the test proves nothing: \
             {vol_good} vs {vol_bad}"
        );

        // The QC did not. Both reconstructions are essentially perfect because the system is
        // square, so RECON is describing the arithmetic rather than the model.
        for (label, res) in [("correct", &good), ("wrong endpoint", &bad)] {
            let recon = res.wells[0].mean_recon;
            assert!(
                recon < 0.1,
                "{label}: an exactly-determined solve reconstructs its inputs regardless, recon={recon}"
            );
        }

        // The note is the ONLY thing standing between the user and a silently-perfect QC.
        assert!(bad.dof_note.is_some(), "the dof note must be present — nothing else flags this");
        let note = bad.dof_note.clone().unwrap();
        assert!(
            note.contains("RECON") && note.to_lowercase().contains("add an input log"),
            "the note must name RECON and say what to do about it: {note}"
        );

        // --- Control: add two tools and the reconstruction starts carrying information. ---
        let over_good = run(vec![q.clone(), ill, wat.clone()], over(), "OG");
        let over_bad = run(vec![q, ill_bad, wat], over(), "OB");
        assert_eq!(over_good.dof, 2);
        assert!(over_good.dof_note.is_none(), "an over-determined model needs no warning");
        let (rg, rb) = (over_good.wells[0].mean_recon, over_bad.wells[0].mean_recon);

        // The sharpest statement of the finding, and it needs no endpoint error at all: same well,
        // same logs, same components, CORRECT endpoints throughout — the square solve reports
        // essentially zero incoherence and the over-determined one reports a real number. The
        // difference is not the model, it is whether anything was left over to check it with.
        let square_recon = good.wells[0].mean_recon;
        assert!(
            rg > square_recon + 0.3,
            "identical model and logs: square reports {square_recon}, over-determined reports {rg} — \
             the square figure is arithmetic, not fit quality"
        );
        // And only with those degrees of freedom does the endpoint error move the number.
        assert!(rb > rg * 1.5, "the endpoint error must show once RECON can see it: {rg} -> {rb}");
    }

    #[test]
    fn recovers_known_three_mineral_mix() {
        let (q, ill) = (lib_get("Quartz"), lib_get("Illite"));
        // Zone-less variant of water so it appears in unity and all tools.
        let mut wat = lib_get("Water Sxo");
        wat.zone = "".into();
        let comps = [&q, &ill, &wat];
        let truth = [0.55, 0.15, 0.30];
        let keys = ["RHOB", "NPHI", "DT", "GR"];
        let meas: Vec<f64> = keys
            .iter()
            .map(|k| comps.iter().zip(truth).map(|(c, v)| c.endpoints[&k.to_string()] * v).sum::<f64>())
            .collect();
        let sig = [0.03, 0.03, 5.0, 15.0];
        let (a, b) = weighted(&comps, &keys, &sig, &meas);
        let hi = vec![1.0; 3];
        let v = solve_bounded_lsq(&a, &b, Some(&unity_of(&comps)), &hi, 3).unwrap();
        for (got, want) in v.iter().zip(truth) {
            assert!((got - want).abs() < 0.02, "got {v:?}, want {truth:?}");
        }
    }

    #[test]
    fn unity_is_exact_and_bounds_hold() {
        let (q, ill) = (lib_get("Quartz"), lib_get("Illite"));
        let mut wat = lib_get("Water Sxo");
        wat.zone = "".into();
        let comps = [&q, &ill, &wat];
        let keys = ["RHOB", "NPHI", "DT", "GR"];
        let sig = [0.03, 0.03, 5.0, 15.0];
        let (a, b) = weighted(&comps, &keys, &sig, &[2.40, 0.20, 80.0, 60.0]);
        let hi = vec![1.0, 1.0, 0.25]; // cap water below its natural solution
        let v = solve_bounded_lsq(&a, &b, Some(&unity_of(&comps)), &hi, 3).unwrap();
        let sum: f64 = v.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "unity violated: {sum}");
        assert!(v[2] <= 0.25 + 1e-9, "upper bound violated: {v:?}");
        assert!(v.iter().all(|&x| x >= -1e-9), "lower bound violated: {v:?}");
    }

    #[test]
    fn nonneg_holds_when_truth_is_a_boundary() {
        let (q, ill) = (lib_get("Quartz"), lib_get("Illite"));
        let mut wat = lib_get("Water Sxo");
        wat.zone = "".into();
        let comps = [&q, &ill, &wat];
        let truth = [0.75, 0.0, 0.25];
        let keys = ["RHOB", "NPHI", "DT", "GR"];
        let meas: Vec<f64> = keys
            .iter()
            .map(|k| comps.iter().zip(truth).map(|(c, v)| c.endpoints[&k.to_string()] * v).sum::<f64>())
            .collect();
        let sig = [0.03, 0.03, 5.0, 15.0];
        let (a, b) = weighted(&comps, &keys, &sig, &meas);
        let hi = vec![1.0; 3];
        let v = solve_bounded_lsq(&a, &b, Some(&unity_of(&comps)), &hi, 3).unwrap();
        assert!(v[1] >= -1e-9 && v[1] < 0.02, "illite should be ~0, got {}", v[1]);
        assert!((v[0] - 0.75).abs() < 0.02 && (v[2] - 0.25).abs() < 0.02, "got {v:?}");
    }

    #[test]
    fn xu_split_recovers_sw_and_sxo_from_conductivity() {
        // Shaly sand with invasion: quartz 0.55, illite 0.15, porosity 0.30.
        // U zone: Sw = 0.40 → water_sw 0.12, oil_sw 0.18.
        // X zone: Sxo = 0.80 → water_sxo 0.24, oil_sxo 0.06.
        let q = lib_get("Quartz");
        let ill = lib_get("Illite");
        let wx = lib_get("Water Sxo");
        let wu = lib_get("Water Sw");
        let ox = lib_get("Oil Sxo");
        let ou = lib_get("Oil Sw");
        let comps = [&q, &ill, &wx, &wu, &ox, &ou];
        let n = comps.len();
        let truth = [0.55, 0.15, 0.24, 0.12, 0.06, 0.18];

        let props = FluidProps {
            rw: 0.43,
            rw_temp_f: 77.0,
            rmf: 0.10,
            rmf_temp_f: 62.0,
            ftemp_f: 148.0,
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
            indonesia_k: 1.0,
            simandoux_c: 1.0,
            phit_sh: 0.1,
            ws_b: 0.0,
        };
        let fc = fluid_calc(&props);
        let inv_w = 1.0 / fc.w;

        // Nuclear tools (X-zone fluids visible, U-zone fluids invisible).
        let keys = ["RHOB", "NPHI", "DT", "GR"];
        let sig = [0.0264, 0.014, 1.951, 6.0];
        let meas: Vec<f64> = keys
            .iter()
            .map(|k| {
                comps
                    .iter()
                    .zip(truth)
                    .map(|(c, v)| if c.zone == "U" { 0.0 } else { c.endpoints[&k.to_string()] * v })
                    .sum::<f64>()
            })
            .collect();
        let (mut a, mut b) = weighted(&comps, &keys, &sig, &meas);

        // CT row (U-zone water) and CXO row (X-zone water), dual-water linear transform.
        let ct_t = fc.cw.powf(inv_w) * truth[3]; // only water_sw conducts
        let cxo_t = fc.cmf.powf(inv_w) * truth[2];
        let wct = 1.0 / fc.u_ct;
        let wcxo = 1.0 / fc.u_cxo;
        let mut ct_row = vec![0.0; n];
        ct_row[3] = fc.cw.powf(inv_w) * wct;
        let mut cxo_row = vec![0.0; n];
        cxo_row[2] = fc.cmf.powf(inv_w) * wcxo;
        a.push(ct_row);
        b.push(ct_t * wct);
        a.push(cxo_row);
        b.push(cxo_t * wcxo);

        // POROSITY soft row: X fluids − U fluids = 0.
        let sw = 1.0 / SIGMA_CONSTRAINT;
        a.push(vec![0.0, 0.0, sw, -sw, sw, -sw]);
        b.push(0.0);

        let unity = unity_of(&comps); // X fluids excluded
        let hi = vec![1.0, 1.0, 0.5, 0.5, 0.5, 0.5];
        let v = solve_bounded_lsq(&a, &b, Some(&unity), &hi, n).unwrap();
        for (got, want) in v.iter().zip(truth) {
            assert!((got - want).abs() < 0.02, "got {v:?}, want {truth:?}");
        }
        let phie = v[3] + v[5];
        let sw_calc = v[3] / phie;
        let sxo_calc = v[2] / (v[2] + v[4]);
        assert!((sw_calc - 0.40).abs() < 0.05, "Sw {sw_calc}");
        assert!((sxo_calc - 0.80).abs() < 0.05, "Sxo {sxo_calc}");
    }

    #[test]
    fn bound_water_tracks_clay_volume() {
        // BNDWAT soft constraint: v_bw ≈ k·v_illite with k = 96·CEC·ρ/(T°C+298).
        let q = lib_get("Quartz");
        let ill = lib_get("Illite");
        let bw = lib_get("BoundWater");
        let mut wat = lib_get("Water Sxo");
        wat.zone = "".into();
        let comps = [&q, &ill, &bw, &wat];
        let n = comps.len();
        let t_c = (148.0 - 32.0) * 5.0 / 9.0;
        let k = bndwat_multiplier(ill.cec, ill.endpoints["RHOB"], t_c, 1.0);
        assert!((k - 0.184).abs() < 0.01, "reference multiplier: got {k}");

        let truth = [0.60, 0.20, k * 0.20, 1.0 - 0.60 - 0.20 - k * 0.20];
        let keys = ["RHOB", "NPHI", "DT", "GR"];
        let sig = [0.0264, 0.014, 1.951, 6.0];
        let meas: Vec<f64> = keys
            .iter()
            .map(|k2| comps.iter().zip(truth).map(|(c, v)| c.endpoints[&k2.to_string()] * v).sum::<f64>())
            .collect();
        let (mut a, mut b) = weighted(&comps, &keys, &sig, &meas);
        let sw = 1.0 / SIGMA_CONSTRAINT;
        a.push(vec![0.0, k * sw, -sw, 0.0]);
        b.push(0.0);
        let hi = vec![1.0, 1.0, 0.5, 0.5];
        let v = solve_bounded_lsq(&a, &b, Some(&unity_of(&comps)), &hi, n).unwrap();
        assert!((v[2] - k * v[1]).abs() < 0.02, "bound water should track clay: {v:?}");
        for (got, want) in v.iter().zip(truth) {
            assert!((got - want).abs() < 0.03, "got {v:?}, want {truth:?}");
        }
    }

    #[test]
    fn wet_clay_porosity_bound_water_tie() {
        // Wet-Clay-Porosity source: k = φ_clay/(1−φ_clay). (1) It equals the CEC route with the
        // equivalent cec_equiv (same physics, different driver — matches dry_clay_calc's inversion);
        // (2) it drives the same bounded solve so v_bw tracks the clay volume.
        let ill = lib_get("Illite");
        let phi = ill.wet_clay_porosity; // Techlog WCLP = 0.104
        assert!((phi - 0.104).abs() < 1e-9, "Illite WCLP default");
        let rho = ill.endpoints["RHOB"]; // dry illite = 2.78
        let (t_c, alpha) = (25.0, 1.0);
        let k = phi / (1.0 - phi);
        assert!((k - 0.104 / 0.896).abs() < 1e-12);
        // cec_equiv makes bndwat_multiplier reproduce k exactly (the dry_clay_calc bridge).
        let cec_equiv = k * (t_c + 298.0) / (alpha * 96.0 * rho);
        assert!((bndwat_multiplier(cec_equiv, rho, t_c, alpha) - k).abs() < 1e-12);

        // Same bounded solve as bound_water_tracks_clay_volume, but with the WCP multiplier.
        let q = lib_get("Quartz");
        let bw = lib_get("BoundWater");
        let mut wat = lib_get("Water Sxo");
        wat.zone = "".into();
        let comps = [&q, &ill, &bw, &wat];
        let n = comps.len();
        let truth = [0.60, 0.20, k * 0.20, 1.0 - 0.60 - 0.20 - k * 0.20];
        let keys = ["RHOB", "NPHI", "DT", "GR"];
        let sig = [0.0264, 0.014, 1.951, 6.0];
        let meas: Vec<f64> = keys
            .iter()
            .map(|k2| comps.iter().zip(truth).map(|(c, v)| c.endpoints[&k2.to_string()] * v).sum::<f64>())
            .collect();
        let (mut a, mut b) = weighted(&comps, &keys, &sig, &meas);
        let sw = 1.0 / SIGMA_CONSTRAINT;
        a.push(vec![0.0, k * sw, -sw, 0.0]);
        b.push(0.0);
        let hi = vec![1.0, 1.0, 0.5, 0.5];
        let v = solve_bounded_lsq(&a, &b, Some(&unity_of(&comps)), &hi, n).unwrap();
        assert!((v[2] - k * v[1]).abs() < 0.02, "WCP bound water should track clay: {v:?}");
    }

    #[test]
    fn wcp_degenerate_smectite_falls_back_to_cec() {
        // Techlog carries smectite WCLP = 1.0 as a *post-solve* placeholder (it floors 1−φ at 1e-4
        // for wet-clay-volume output, never as an inversion constraint). Fed naively into the BNDWAT
        // *solver* row as φ/(1−φ) with a 0.95 clamp it yields k ≈ 19 — ~100× any real clay — which
        // swamps the constraint and forces absurd bound water. The WCP route must instead defer to the
        // CEC-calibrated multiplier for such a degenerate φ, so switching porosity source doesn't
        // 30× the smectite bound water. Exercises the real k-selection (`bound_water_multiplier`).
        let (t_c, alpha) = (100.0, 1.0);
        let smec = lib_get("Montmorillonite");
        assert!((smec.wet_clay_porosity - 1.0).abs() < 1e-9, "Techlog smectite WCLP placeholder");
        let rho = smec.endpoints["RHOB"];

        let cec_k = bound_water_multiplier(PorositySource::Cec, smec.cec, smec.wet_clay_porosity, rho, t_c, alpha);
        let wcp_k = bound_water_multiplier(PorositySource::WetClayPorosity, smec.cec, smec.wet_clay_porosity, rho, t_c, alpha);
        // The two sources AGREE for the degenerate clay, and the result is physical (well under 1).
        assert!((wcp_k - cec_k).abs() < 1e-12, "degenerate WCP φ must fall back to CEC: wcp={wcp_k} cec={cec_k}");
        assert!(wcp_k > 0.4 && wcp_k < 0.9, "smectite bound-water multiplier stays physical: {wcp_k}");
        // Guard: the naive 0.95-clamp path this replaces really was catastrophic.
        let naive_phi = smec.wet_clay_porosity.clamp(0.0, 0.95);
        assert!(naive_phi / (1.0 - naive_phi) > 18.0, "sanity: old clamp gave ~19");

        // A real clay (φ = 0.104 < ceiling) still uses the geometric route, unchanged.
        let ill = lib_get("Illite");
        let k_ill = bound_water_multiplier(PorositySource::WetClayPorosity, ill.cec, ill.wet_clay_porosity, ill.endpoints["RHOB"], t_c, alpha);
        assert!((k_ill - 0.104 / 0.896).abs() < 1e-12, "real-clay WCP stays geometric: {k_ill}");
    }

    #[test]
    fn request_defaults_keep_every_constraint_on() {
        // Backward-compat contract: a request JSON WITHOUT the new constraint block must default every
        // constraint ON, σ to the reviewed 0.01, and the porosity source to CEC — so an older frontend
        // (and every reviewed number) solves exactly as before. This guards the "absent = unchanged"
        // invariant the whole increment rests on.
        let req: SandiminRequest =
            serde_json::from_str(r#"{"components":[],"tools":[],"apply_well_ids":[],"custody":{"actor":{"kind":"HUMAN","identity":"automated-test-fixture"},"source_note":"test fixture values declared in the owning test"}}"#,
        ).unwrap();
        assert!(req.unity, "UNITY defaults on");
        assert!(req.enforce_porosity, "POROSITY defaults on");
        assert!(req.enforce_bndwat, "BNDWAT defaults on");
        assert!(req.enforce_water_mud, "WATER MUD defaults on");
        assert!((req.sigma_constraint - 0.01).abs() < 1e-12, "σ defaults to 0.01: {}", req.sigma_constraint);
        assert_eq!(req.porosity_source, PorositySource::Cec);
        // A stray non-positive σ must not blow up the row weight; the solver falls back to the default.
        let bad: SandiminRequest =
            serde_json::from_str(r#"{"components":[],"tools":[],"apply_well_ids":[],"sigma_constraint":0,"custody":{"actor":{"kind":"HUMAN","identity":"automated-test-fixture"},"source_note":"test fixture values declared in the owning test"}}"#,
        ).unwrap();
        let sigma = if bad.sigma_constraint > 0.0 { bad.sigma_constraint } else { SIGMA_CONSTRAINT };
        assert!((sigma - SIGMA_CONSTRAINT).abs() < 1e-12, "non-positive σ falls back to default");
    }

    #[test]
    fn fluid_calc_matches_reference() {
        // reference default-model example: Rw 0.43 @ 77F, Rmf 0.10 @ 62F, FT 148F, m=n=2.
        let props = FluidProps {
            rw: 0.43,
            rw_temp_f: 77.0,
            rmf: 0.10,
            rmf_temp_f: 62.0,
            ftemp_f: 148.0,
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
            indonesia_k: 1.0,
            simandoux_c: 1.0,
            phit_sh: 0.1,
            ws_b: 0.0,
        };
        let fc = fluid_calc(&props);
        assert!((fc.w - 2.0).abs() < 1e-9);
        // The reference shows U FreeW CT response 4.298 mho/m and salinity ~13,048 ppm.
        assert!((fc.cw - 4.3).abs() < 0.2, "Cw {}", fc.cw);
        assert!((fc.salinity_w_ppm - 13_048.0).abs() < 800.0, "S {}", fc.salinity_w_ppm);
        assert!(fc.cmf > fc.cw, "filtrate is saltier here: Cmf {} Cw {}", fc.cmf, fc.cw);
        assert!(fc.u_ct > 0.0 && fc.u_cxo > 0.0);
    }

    #[test]
    fn library_has_expected_shape() {
        let lib = sandimin_library();
        assert_eq!(lib.len(), 27, "IP mineral-dropdown parity");
        for c in &lib {
            for k in TOOL_KEYS {
                assert!(c.endpoints.contains_key(k), "{} missing {k}", c.name);
            }
        }
        assert_eq!(lib.iter().filter(|c| c.kind == "clay").count(), 6);
        assert_eq!(lib.iter().filter(|c| c.kind == "fluid").count(), 7);
        let ill = lib.iter().find(|c| c.name == "Illite").unwrap();
        assert!((ill.cec - 0.25).abs() < 1e-9);
        assert!((ill.wet_clay_porosity - 0.104).abs() < 1e-9, "Illite Techlog WCLP");
        // Smectite carries Techlog's WCLP = 1.0 placeholder verbatim; the WCP-source k-selection
        // treats φ ≥ WCP_PHYSICAL_CEILING as degenerate and falls back to CEC (see
        // wcp_degenerate_smectite_falls_back_to_cec). Real clays stay well under the ceiling.
        let smec = lib.iter().find(|c| c.name == "Montmorillonite").unwrap();
        assert!((smec.wet_clay_porosity - 1.0).abs() < 1e-9, "Techlog smectite WCLP placeholder");
        assert!(
            lib.iter().filter(|c| c.kind == "clay" && c.name != "Montmorillonite").all(|c| c.wet_clay_porosity < WCP_PHYSICAL_CEILING),
            "every non-smectite clay's WCLP is a physical geometric porosity"
        );
        let wsxo = lib.iter().find(|c| c.name == "Water Sxo").unwrap();
        assert_eq!(wsxo.zone, "X");
        assert!((wsxo.max_vol - 0.5).abs() < 1e-9);
        // Minerals carry no wet-clay porosity; every clay has one.
        assert!(lib.iter().filter(|c| c.kind == "mineral").all(|c| c.wet_clay_porosity == 0.0));
        assert!(lib.iter().filter(|c| c.kind == "clay").all(|c| c.wet_clay_porosity > 0.0));
    }

    #[test]
    fn pef_converts_to_u_before_mixing() {
        // U mixes volumetrically; PEF does not. Use a pyritic sand — pyrite's high ρe
        // makes the two paths diverge sharply (a quartz/calcite pair nearly coincides
        // because their ρe are almost equal). The true U is the volume average, and the
        // PEF a tool would read is U/ρe. Converting that PEF back through rho_e must
        // recover U, while linearly mixing the raw PEF endpoints gives a wrong answer.
        let (q, py) = (lib_get("Quartz"), lib_get("Pyrite"));
        let (vq, vp) = (0.85, 0.15);
        let u_true = vq * q.endpoints["U"] + vp * py.endpoints["U"];
        let rhob = vq * q.endpoints["RHOB"] + vp * py.endpoints["RHOB"];
        let pef_read = u_true / rho_e(rhob);

        let u_back = pef_to_u(pef_read, rhob).unwrap();
        assert!((u_back - u_true).abs() < 1e-9, "U round-trip: {u_back} vs {u_true}");

        let pef_linear = vq * q.endpoints["PEF"] + vp * py.endpoints["PEF"];
        assert!(
            (pef_linear - pef_read).abs() > 0.5,
            "raw-PEF volumetric mixing ({pef_linear:.3}) must differ from the U path ({pef_read:.3})"
        );
        assert!(pef_to_u(5.0, -1.0).is_none(), "non-physical RHOB rejected");
        assert!(pef_to_u(f64::NAN, 2.5).is_none(), "non-finite PEF rejected");
    }

    #[test]
    fn rejects_underdetermined_request() {
        // 4 components under hard unity need at least n−1 = 3 independent tool logs; offering
        // only 2 leaves the system under-determined (a whole subspace of vertex solutions).
        // The solver must refuse the run up front rather than emit arbitrary volumes. The
        // gate fires on request shape alone, before any DB access, so an empty db is fine.
        let q = lib_get("Quartz");
        let ill = lib_get("Illite");
        let cal = lib_get("Calcite");
        let mut wat = lib_get("Water Sxo");
        wat.zone = String::new();
        let req = SandiminRequest {
            input_set: None,
            output_set: None,
            custody: crate::workflow::test_run_custody(),
            components: vec![q, ill, cal, wat],
            tools: vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.03 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.03 },
            ],
            apply_well_ids: vec!["dummy-well".into()],
            output_prefix: "MM".into(),
            unity: true,
            fluid: None,
            ftemp_curve: None,
            recon_qc: false,
            sw_model: SwModel::LinearDw,
            porosity_source: PorositySource::Cec,
            enforce_porosity: true,
            enforce_bndwat: true,
            enforce_water_mud: true,
            sigma_constraint: 0.01,
        };
        let conn = Mutex::new(Connection::open_in_memory().unwrap());
        let res = run_sandimin(&conn, &req, None);
        let err = res.error.as_deref().unwrap_or("");
        assert!(
            err.contains("need at least"),
            "expected an under-determination refusal, got {:?}",
            res.error
        );
        assert!(res.wells.is_empty(), "no wells should be processed on a refused request");
    }

    #[test]
    fn rejects_all_zero_conductivity_row() {
        // CT reads U-zone (deep) water, but the model's only water is X-zone (Water Sxo), so
        // the CT response row is all zero — a bogus 0 = Ct^(1/w) equation. The whole-model
        // no-water guard passes (X water exists), so this per-zone case must be caught here.
        let q = lib_get("Quartz");
        let ill = lib_get("Illite");
        let wx = lib_get("Water Sxo"); // zone X only — nothing in the U (deep) zone
        let props = FluidProps {
            rw: 0.3,
            rw_temp_f: 77.0,
            rmf: 0.1,
            rmf_temp_f: 62.0,
            ftemp_f: 148.0,
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
            indonesia_k: 1.0,
            simandoux_c: 1.0,
            phit_sh: 0.1,
            ws_b: 0.0,
        };
        let req = SandiminRequest {
            input_set: None,
            output_set: None,
            custody: crate::workflow::test_run_custody(),
            components: vec![q, ill, wx],
            tools: vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.03 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.03 },
                ToolSpec { key: "CT".into(), curve: "RT".into(), sigma: 0.0 },
            ],
            apply_well_ids: vec!["dummy-well".into()],
            output_prefix: "MM".into(),
            unity: true,
            fluid: Some(props),
            ftemp_curve: None,
            recon_qc: false,
            sw_model: SwModel::LinearDw,
            porosity_source: PorositySource::Cec,
            enforce_porosity: true,
            enforce_bndwat: true,
            enforce_water_mud: true,
            sigma_constraint: 0.01,
        };
        let conn = Mutex::new(Connection::open_in_memory().unwrap());
        let res = run_sandimin(&conn, &req, None);
        let err = res.error.as_deref().unwrap_or("");
        assert!(
            err.contains("all zero"),
            "expected an all-zero conductivity-row refusal, got {:?}",
            res.error
        );
    }

    /// End-to-end smoke test for both reference fixes, driven through the actual DB path
    /// (fetch_curve_frame → run_sandimin with a PEF tool → write; and run_module vsh_dn
    /// with GR). #[ignore] so the normal suite never touches it. If SANDIBUMI_E2E_DB
    /// points at a project.duckdb copy with a RHOB+NPHI+GR well, it runs on that REAL
    /// field data; otherwise it seeds a synthetic well through the real schema + write
    /// path so the new run_sandimin PEF branch and vsh_dn output are still exercised
    /// against DB-resident curves. Run with:
    ///   [SANDIBUMI_E2E_DB=<copy.duckdb>] cargo test --lib -- \
    ///       --ignored --nocapture e2e_pef_and_vsh_on_real_well
    #[test]
    #[ignore]
    fn e2e_pef_and_vsh_on_real_well() {
        // Best RHOB+NPHI+GR well in a DB, preferring one that also has PEF.
        fn pick_well(db: &Mutex<Connection>, wells: &[crate::db::WellSummary]) -> Option<(String, [usize; 5])> {
            let need: Vec<String> =
                ["RHOB", "NPHI", "DT", "GR", "PEF"].iter().map(|s| s.to_string()).collect();
            let cnt = |cols: &HashMap<String, Vec<f32>>, k: &str| {
                cols.get(k).map(|v| v.iter().filter(|x| x.is_finite()).count()).unwrap_or(0)
            };
            let mut best: Option<(String, [usize; 5], usize)> = None;
            for w in wells {
                let cols = match {
                    let c = db.lock().unwrap();
                    fetch_curve_frame(&c, &w.well_id, &need)
                } {
                    Ok((_d, cols)) => cols,
                    Err(_) => continue,
                };
                let cov = [cnt(&cols, "RHOB"), cnt(&cols, "NPHI"), cnt(&cols, "DT"), cnt(&cols, "GR"), cnt(&cols, "PEF")];
                if cov[0] == 0 || cov[1] == 0 || cov[3] == 0 {
                    continue;
                }
                let score = cov[0] + cov[1] + cov[3] + if cov[4] > 0 { 10_000_000 } else { 0 };
                if best.as_ref().map(|b| score > b.2).unwrap_or(true) {
                    best = Some((w.well_id.clone(), cov, score));
                }
            }
            best.map(|(id, cov, _)| (id, cov))
        }

        // Seed a synthetic well by forward-modeling a quartz/illite/water mix with the
        // library's own endpoints, so run_sandimin should recover it near-exactly and the
        // PEF curve (stored as U/ρe, the tool reading) round-trips through the PEF→U path.
        fn build_synthetic() -> (Mutex<Connection>, String, bool) {
            let conn = Connection::open_in_memory().expect("in-memory db");
            crate::db::create_schema(&conn).expect("schema");
            let wid = "11111111-1111-1111-1111-111111111111";
            conn.execute_batch(&format!(
                "INSERT INTO wells (well_id, well_name, field_name) VALUES ('{wid}','SYNTH-1','E2E');"
            ))
            .expect("insert well");
            let lib = sandimin_library();
            let ep = |nm: &str, k: &str| lib.iter().find(|c| c.name == nm).unwrap().endpoints[k];
            let (n, top, step) = (300usize, 2000.0f64, 0.5f64);
            // fetch_curve_frame reads gr/res_deep/nphi/rhob as NON-optional f32, so
            // res_deep must be non-null (every real well has resistivity); give it a constant.
            let mut sc = String::from("INSERT INTO standard_curves (well_id, depth, gr, res_deep, nphi, rhob, dt) VALUES ");
            let mut pf = String::from("INSERT INTO computed_curves (well_id, depth, curve_name, value) VALUES ");
            for i in 0..n {
                let depth = top + i as f64 * step;
                let t = i as f64 / (n - 1) as f64;
                // Smoothly varying, in-bounds mix (water ≤ 0.35, illite ≤ 0.5).
                let vi = 0.05 + 0.40 * (t * 9.42).sin().powi(2);
                let vw = 0.05 + 0.25 * ((t * 6.28).cos() * 0.5 + 0.5);
                let vq = (1.0 - vi - vw).max(0.0);
                let s = vq + vi + vw;
                let (vq, vi, vw) = (vq / s, vi / s, vw / s);
                let mix = |k: &str| vq * ep("Quartz", k) + vi * ep("Illite", k) + vw * ep("Water Sxo", k);
                let (gr, nphi, rhob, dt, u) = (mix("GR"), mix("NPHI"), mix("RHOB"), mix("DT"), mix("U"));
                let pef = u / rho_e(rhob);
                sc += &format!("('{wid}',{depth},{gr},2.0,{nphi},{rhob},{dt}),");
                pf += &format!("('{wid}',{depth},'PEF',{pef}),");
            }
            sc.pop();
            sc.push(';');
            pf.pop();
            pf.push(';');
            conn.execute_batch(&sc).expect("insert standard_curves");
            conn.execute_batch(&pf).expect("insert PEF");
            (Mutex::new(conn), wid.to_string(), true)
        }

        let mut synthetic = false;
        let picked = std::env::var("SANDIBUMI_E2E_DB").ok().and_then(|path| match Connection::open(&path) {
            Ok(conn) => {
                let db = Mutex::new(conn);
                let wells = { let c = db.lock().unwrap(); crate::db::list_wells(&c).unwrap_or_default() };
                match pick_well(&db, &wells) {
                    Some((wid, cov)) => {
                        eprintln!("using REAL db {path}: {} wells; well {wid} cov RHOB/NPHI/DT/GR/PEF = {cov:?}", wells.len());
                        Some((db, wid, cov[4] > 0))
                    }
                    None => {
                        eprintln!("real db {path} opened but has no RHOB+NPHI+GR well — using synthetic");
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("could not open SANDIBUMI_E2E_DB ({e}) — using synthetic");
                None
            }
        });
        let (db, well_id, has_pef) = picked.unwrap_or_else(|| {
            synthetic = true;
            let s = build_synthetic();
            eprintln!("using SYNTHETIC forward-modeled well {} (300 samples, quartz/illite/water)", s.1);
            s
        });

        {
            let c = db.lock().unwrap();
            let raw: i64 = c
                .query_row("SELECT count(*) FROM standard_curves WHERE well_id = ?1", duckdb::params![well_id], |r| r.get(0))
                .unwrap_or(-1);
            let (nd, nr, np) = fetch_curve_frame(&c, &well_id, &["RHOB".into(), "PEF".into()])
                .map(|(d, cols)| {
                    let fin = |k: &str| cols.get(k).map(|v| v.iter().filter(|x| x.is_finite()).count()).unwrap_or(0);
                    (d.len(), fin("RHOB"), fin("PEF"))
                })
                .unwrap_or((999_999, 0, 0));
            eprintln!("debug: standard_curves rows={raw}; fetch depths={nd} finiteRHOB={nr} finitePEF={np}");
        }

        // ---- Fix 1: multimin with a PEF tool (converted to U per sample) ----
        let lib = sandimin_library();
        let get = |nm: &str| lib.iter().find(|c| c.name == nm).cloned().unwrap();
        let q = get("Quartz");
        let ill = get("Illite");
        let mut wat = get("Water Sxo");
        wat.zone = String::new(); // shared water: enters unity + seen by all tools
        let comps = vec![q, ill, wat];

        let base_tools = || {
            vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.0264 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.014 },
                ToolSpec { key: "DT".into(), curve: "DT".into(), sigma: 1.951 },
                ToolSpec { key: "GR".into(), curve: "GR".into(), sigma: 6.0 },
            ]
        };
        let run = |prefix: &str, with_pef: bool| -> SandiminWellResult {
            let mut tools = base_tools();
            if with_pef {
                tools.push(ToolSpec { key: "PEF".into(), curve: "PEF".into(), sigma: 0.3 });
            }
            let req = SandiminRequest {
                input_set: None,
                output_set: None,
                custody: crate::workflow::test_run_custody(),
                components: comps.clone(),
                tools,
                apply_well_ids: vec![well_id.clone()],
                output_prefix: prefix.into(),
                unity: true,
                fluid: None,
                ftemp_curve: None,
                recon_qc: false,
                sw_model: SwModel::LinearDw,
                porosity_source: PorositySource::Cec,
                enforce_porosity: true,
                enforce_bndwat: true,
                enforce_water_mud: true,
                sigma_constraint: 0.01,
            };
            let res = run_sandimin(&db, &req, None);
            assert!(res.error.is_none(), "run_sandimin error: {:?}", res.error);
            res.wells.into_iter().next().unwrap()
        };

        let no_pef = run("MMNOPEF", false);
        eprintln!(
            "multimin (no PEF): solved={} mean_recon={:.4} err={:?}",
            no_pef.rows_solved, no_pef.mean_recon, no_pef.error
        );
        assert!(no_pef.rows_solved > 0, "no samples solved without PEF");

        if has_pef {
            let with_pef = run("MMPEF", true);
            eprintln!(
                "multimin (+PEF→U): solved={} mean_recon={:.4}",
                with_pef.rows_solved, with_pef.mean_recon
            );
            assert!(with_pef.rows_solved > 0, "PEF run solved no samples");
            assert!(with_pef.mean_recon.is_finite(), "PEF run recon not finite");
            if synthetic {
                // PEF was stored as U_true/ρe; the PEF→U path must reconstruct U_true,
                // so the converted row fits the true volumes and RECON stays ~0. A wrong
                // (raw-PEF) mix would leave this row inconsistent and inflate RECON.
                assert!(
                    with_pef.mean_recon < 0.05,
                    "synthetic PEF→U row should fit near-perfectly; recon={}",
                    with_pef.mean_recon
                );
            }
            // Hard-unity guarantees Σvol=1; verify on the written curves.
            let cols = {
                let c = db.lock().unwrap();
                fetch_curve_frame(
                    &c,
                    &well_id,
                    &["VOL_QUARTZ".into(), "VOL_ILLITE".into(), "VOL_WATER_SXO".into()],
                )
                .unwrap()
                .1
            };
            let (mut n_ok, mut sum_err_max) = (0usize, 0.0f32);
            for i in 0..cols["VOL_QUARTZ"].len() {
                let s = cols["VOL_QUARTZ"][i] + cols["VOL_ILLITE"][i] + cols["VOL_WATER_SXO"][i];
                if s.is_finite() {
                    n_ok += 1;
                    sum_err_max = sum_err_max.max((s - 1.0).abs());
                }
            }
            eprintln!("  VOL rows summing to 1: {n_ok}, max |Σvol−1| = {sum_err_max:.2e}");
            assert!(n_ok > 0 && sum_err_max < 1e-3, "unity violated on written VOL curves");
        }

        // ---- Fix 2: vsh_dn clay-type guard on real GR ----
        let (depth, cols) = {
            let c = db.lock().unwrap();
            fetch_curve_frame(&c, &well_id, &["RHOB".into(), "NPHI".into(), "GR".into()]).unwrap()
        };
        let nsamp = depth.len();
        let logs: HashMap<String, Vec<f32>> = ["RHOB", "NPHI", "GR"]
            .iter()
            .map(|k| (k.to_string(), cols[*k].clone()))
            .collect();
        let params: HashMap<String, Vec<f64>> = [
            ("RHO_MA", 2.645),
            ("RHO_SH", 2.5),
            ("RHO_FL", 1.0),
            ("NPHI_MA", -0.02),
            ("NPHI_SH", 0.35),
            ("NPHI_FL", 1.0),
            ("GR_MA", 15.0),
            ("GR_SH", 120.0),
            ("FLAG_TOL", 0.25),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), vec![*v; nsamp]))
        .collect();
        let ctx = crate::modules::ModuleContext {
            n: nsamp,
            logs,
            params,
            opts: HashMap::new(),
            depth_unit: Default::default(),
        };
        let out = crate::modules::run_module("vsh_dn", &ctx).expect("vsh_dn run");
        let flag = &out["VSH_DN_FLAG"];
        let vsh = &out["VSH"];
        let (mut n_flag, mut n_eval, mut vsh_min, mut vsh_max) = (0usize, 0usize, f32::MAX, f32::MIN);
        for i in 0..nsamp {
            if flag[i].is_finite() {
                n_eval += 1;
                if flag[i] == 1.0 {
                    n_flag += 1;
                }
                vsh_min = vsh_min.min(vsh[i]);
                vsh_max = vsh_max.max(vsh[i]);
            }
        }
        eprintln!(
            "vsh_dn: evaluated={n_eval} flagged={n_flag} ({:.1}%), VSH∈[{vsh_min:.3},{vsh_max:.3}]",
            100.0 * n_flag as f32 / n_eval.max(1) as f32
        );
        assert!(n_eval > 0, "vsh_dn produced no evaluated samples");
        assert!(vsh_min >= 0.0 && vsh_max <= 1.0, "limited VSH out of [0,1]");
    }

    #[test]
    fn dry_clay_matches_the_kkt_example() {
        // Multimin Parameters.xlsx, KK-1 Post Main: dry clay 2.70, wet 2.18333,
        // wet NPHI 0.489583, wet GR 110 -> phi_clay 0.3039, NPHI 0.2667, GR 158.0.
        let dc = dry_clay_calc(&WetClayInput {
            rhob_wet: 2.18333,
            nphi_wet: 0.489583,
            gr_wet: 110.0,
            dt_wet: None,
            rho_dry: 2.70,
            fluid: None,
        })
        .unwrap();
        assert!((dc.phi_clay - 0.303922).abs() < 1e-5, "phi_clay {}", dc.phi_clay);
        assert!((dc.nphi_dry - 0.2667).abs() < 3e-4, "nphi_dry {}", dc.nphi_dry);
        assert!((dc.gr_dry - 158.0).abs() < 0.1, "gr_dry {}", dc.gr_dry);
        assert_eq!(dc.rhob_dry, 2.70);
        assert!(dc.dt_dry.is_none(), "no wet DT picked -> no dry DT");
        assert!((dc.cbw_ratio - dc.phi_clay / (1.0 - dc.phi_clay)).abs() < 1e-12);
    }

    #[test]
    fn dry_clay_cec_reproduces_the_bndwat_tie() {
        // The equivalent CEC must make the solver's BNDWAT multiplier equal the
        // phi_clay ratio at the SAME fluid conditions (T and alpha_u), and the DT
        // conversion must follow the 189 us/ft water line.
        let fluid = FluidProps {
            rw: 0.305,
            rw_temp_f: 77.0,
            rmf: 0.10,
            rmf_temp_f: 62.0,
            ftemp_f: 148.0,
            m: 1.86,
            n: 1.78,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
            indonesia_k: 1.0,
            simandoux_c: 1.0,
            phit_sh: 0.1,
            ws_b: 0.0,
        };
        let dc = dry_clay_calc(&WetClayInput {
            rhob_wet: 2.18333,
            nphi_wet: 0.489583,
            gr_wet: 110.0,
            dt_wet: Some(110.0),
            rho_dry: 2.70,
            fluid: Some(fluid.clone()),
        })
        .unwrap();
        let fc = fluid_calc(&fluid);
        let t_c = (fluid.ftemp_f - 32.0) * 5.0 / 9.0;
        let k = bndwat_multiplier(dc.cec_equiv, dc.rhob_dry, t_c, fc.alpha_u);
        assert!((k - dc.cbw_ratio).abs() < 1e-9, "k {} vs ratio {}", k, dc.cbw_ratio);
        let phi = dc.phi_clay;
        let want_dt = (110.0 - 189.0 * phi) / (1.0 - phi);
        assert!((dc.dt_dry.unwrap() - want_dt).abs() < 1e-9);
    }

    #[test]
    fn dry_clay_rejects_degenerate_densities() {
        let base = WetClayInput {
            rhob_wet: 2.18,
            nphi_wet: 0.49,
            gr_wet: 110.0,
            dt_wet: None,
            rho_dry: 2.70,
            fluid: None,
        };
        let wet_too_light = WetClayInput { rhob_wet: 0.9, ..base.clone() };
        assert!(dry_clay_calc(&wet_too_light).is_err(), "wet RHOB below water density");
        let dry_below_wet = WetClayInput { rho_dry: 2.10, ..base.clone() };
        assert!(dry_clay_calc(&dry_below_wet).is_err(), "dry density below wet reading");
        assert!(dry_clay_calc(&base).is_ok());
    }

    #[test]
    fn dry_clay_rejects_unphysical_picks() {
        // percent-entry habit, blank-field zero coercion, and a wet DT
        // below the 189·φ water term must all error instead of producing
        // negative dry endpoints silently.
        let base = WetClayInput {
            rhob_wet: 2.18333,
            nphi_wet: 0.489583,
            gr_wet: 110.0,
            dt_wet: None,
            rho_dry: 2.70,
            fluid: None,
        };
        let pct = WetClayInput { nphi_wet: 48.9583, ..base.clone() };
        assert!(dry_clay_calc(&pct).is_err(), "percent NPHI entry");
        let blank_nphi = WetClayInput { nphi_wet: 0.0, ..base.clone() };
        assert!(dry_clay_calc(&blank_nphi).is_err(), "blank NPHI coerced to 0");
        let blank_gr = WetClayInput { gr_wet: 0.0, ..base.clone() };
        assert!(dry_clay_calc(&blank_gr).is_err(), "blank GR coerced to 0");
        // phi_clay ~0.3039 -> water term 189*phi ~57.4 us/ft.
        let dt_low = WetClayInput { dt_wet: Some(50.0), ..base.clone() };
        assert!(dry_clay_calc(&dt_low).is_err(), "DT below the water term");
        let dt_ok = WetClayInput { dt_wet: Some(110.0), ..base.clone() };
        assert!(dry_clay_calc(&dt_ok).is_ok());
    }

    // --- Saturation models (Jauhar's Sw-equation request) --------------------

    #[test]
    fn sw_indonesia_round_trips() {
        // Forward-model Rt from a known Sw via the Indonesia equation, then recover it.
        let (phie, vsh, rw, rsh, m, n, a): (f64, f64, f64, f64, f64, f64, f64) =
            (0.22, 0.18, 0.08, 3.5, 2.0, 2.0, 1.0);
        let d = vsh.powf(1.0 - vsh / 2.0) / rsh.sqrt() + (phie.powf(m) / (a * rw)).sqrt();
        for sw_true in [0.15f64, 0.35, 0.55, 0.8, 1.0] {
            let rt = 1.0 / (d * d * sw_true.powf(n)); // 1/Rt = D²·Sw^n
            let sw = sw_indonesia(rt, phie, vsh, rw, rsh, m, n, a, 1.0);
            assert!((sw - sw_true).abs() < 1e-6, "Indonesia round-trip: got {sw}, want {sw_true}");
        }
        // A non-2 saturation exponent inverts exactly too (Sw^(n/2) is isolated in closed form).
        let (n2, sw_true): (f64, f64) = (1.8, 0.4);
        let rt = 1.0 / (d * d * sw_true.powf(n2));
        assert!((sw_indonesia(rt, phie, vsh, rw, rsh, m, n2, a, 1.0) - sw_true).abs() < 1e-6);
    }

    #[test]
    fn sw_simandoux_round_trips() {
        // Forward-model Rt from a known Sw via modified Simandoux, then recover it — exercising
        // both the n==2 quadratic path and the general-n bisection path.
        let (phie, vsh, rw, rsh, m, a): (f64, f64, f64, f64, f64, f64) = (0.25, 0.2, 0.06, 4.0, 2.0, 1.0);
        for &n in &[2.0f64, 1.7, 2.3] {
            for sw_true in [0.2f64, 0.45, 0.7, 0.95] {
                let ct = phie.powf(m) * sw_true.powf(n) / (a * rw * (1.0 - vsh)) + vsh * sw_true / rsh;
                let sw = sw_simandoux_modified_slb(1.0 / ct, phie, vsh, rw, rsh, m, n, a, 1.0);
                assert!((sw - sw_true).abs() < 1e-4, "Simandoux n={n} round-trip: got {sw}, want {sw_true}");
            }
        }
    }

    #[test]
    fn sw_equations_reject_nonphysical_inputs() {
        assert!(sw_indonesia(-1.0, 0.2, 0.1, 0.1, 4.0, 2.0, 2.0, 1.0, 1.0).is_nan(), "Rt<=0");
        assert!(sw_indonesia(10.0, 0.0, 0.1, 0.1, 4.0, 2.0, 2.0, 1.0, 1.0).is_nan(), "phie<=0");
        assert!(sw_simandoux_modified_slb(-1.0, 0.2, 0.1, 0.1, 4.0, 2.0, 2.0, 1.0, 1.0).is_nan(), "Rt<=0");
        assert!(sw_simandoux_modified_slb(10.0, 0.2, 0.1, 0.0, 4.0, 2.0, 2.0, 1.0, 1.0).is_nan(), "Rw<=0");
        // A very conductive Rt (fresh, high-φ) clamps Sw to 1, never above.
        assert!((sw_indonesia(0.01, 0.3, 0.0, 0.1, 4.0, 2.0, 2.0, 1.0, 1.0) - 1.0).abs() < 1e-9);
        assert!((sw_simandoux_modified_slb(0.01, 0.3, 0.0, 0.1, 4.0, 2.0, 2.0, 1.0, 1.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn sw_equations_match_hand_computed_points() {
        // INDEPENDENT of the round-trip tests: the expected Sw values are hand-computed NUMERIC
        // LITERALS (not built from the functions' own expressions), so a shale-term or exponent
        // transcription error would fail here instead of being confirmed by a self-referential forward.

        // Clean sand (Vsh=0, m=n=2, a=1) reduces to Archie: Sw² = Rw/(φ²·Rt).
        // φ=0.2, Rw=0.05, Rt=20 ⇒ Sw² = 0.05/(0.04·20) = 0.0625 ⇒ Sw = 0.25 (exact by hand).
        assert!((sw_indonesia(20.0, 0.2, 0.0, 0.05, 4.0, 2.0, 2.0, 1.0, 1.0) - 0.25).abs() < 1e-6, "Indonesia→Archie");
        assert!((sw_simandoux_modified_slb(20.0, 0.2, 0.0, 0.05, 4.0, 2.0, 2.0, 1.0, 1.0) - 0.25).abs() < 1e-6, "Simandoux→Archie");

        // Indonesia WITH shale (exercises Vsh^(1−Vsh/2)/√Rsh): Vsh=0.5, Rsh=4, φ=0.2, Rw=0.1, m=n=2.
        //   term_sh = 0.5^0.75/2 = 0.297302 ; term_sand = √(0.04/0.1) = 0.632456 ; denom = 0.929758
        //   Sw=0.4 ⇒ 1/√Rt = 0.929758·0.4 = 0.371903 ⇒ Rt = 7.230045 (hand-computed).
        assert!((sw_indonesia(7.230045, 0.2, 0.5, 0.1, 4.0, 2.0, 2.0, 1.0, 1.0) - 0.4).abs() < 1e-3, "Indonesia shale point");

        // Modified Simandoux WITH shale (exercises the Vsh·Sw/Rsh term): Vsh=0.4, Rsh=3, φ=0.25,
        // Rw=0.08, m=n=2. coef_sand=0.0625/0.048=1.302083 ; coef_sh=0.133333 ; Sw=0.5 ⇒
        //   1/Rt = 1.302083·0.25 + 0.133333·0.5 = 0.392188 ⇒ Rt = 2.549795 (hand-computed).
        assert!((sw_simandoux_modified_slb(2.549795, 0.25, 0.4, 0.08, 3.0, 2.0, 2.0, 1.0, 1.0) - 0.5).abs() < 1e-3, "Simandoux shale point");
    }

    #[test]
    fn indonesia_post_solve_recovers_known_sw() {
        // Full X/U model: the nuclear tools see the flushed (X) fluids and fix φe; the deep
        // resistivity is forward-modelled from a known deep Sw via Indonesia (Vsh=0 ⇒ Archie). CT
        // stays in the inversion (keeping the U-split well-posed); the Indonesia model then post-solve
        // OVERRIDES SWE = Sw, leaving PHIE (= 1 − quartz) untouched.
        let q = lib_get("Quartz");
        let wsxo = lib_get("Water Sxo");
        let osxo = lib_get("Oil Sxo");
        let wsw = lib_get("Water Sw");
        let osw = lib_get("Oil Sw");
        let ep = |c: &Component, k: &str| c.endpoints[&k.to_string()];
        let (vq, vwx, vox) = (0.70, 0.15, 0.15); // flushed Sxo = 0.5
        let (phie, sw_true, rw): (f64, f64, f64) = (0.30, 0.35, 0.10); // deep Sw = 0.35; Rw at formation T
        let d = (phie.powf(2.0) / (1.0 * rw)).sqrt(); // Vsh=0 ⇒ Indonesia = Archie
        let rt = 1.0 / (d * d * sw_true.powf(2.0));

        let n = 6usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let mix = |k: &str| (vq * ep(&q, k) + vwx * ep(&wsxo, k) + vox * ep(&osxo, k)) as f32;
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "MM-IND", None, None, None).unwrap();
        crate::db::insert_standard_curves(
            &conn,
            wid,
            depth,
            vec![mix("GR"); n],
            vec![rt as f32; n],
            vec![mix("NPHI"); n],
            vec![mix("RHOB"); n],
            vec![mix("DT"); n],
            vec![f32::NAN; n],
        )
        .unwrap();
        let db = Mutex::new(conn);

        let props = FluidProps {
            rw,
            rw_temp_f: 100.0,
            rmf: 0.1,
            rmf_temp_f: 100.0,
            ftemp_f: 100.0,
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
            indonesia_k: 1.0,
            simandoux_c: 1.0,
            phit_sh: 0.1,
            ws_b: 0.0,
        };
        let req = SandiminRequest {
            input_set: None,
            output_set: None,
            custody: crate::workflow::test_run_custody(),
            components: vec![q, wsxo, osxo, wsw, osw],
            tools: vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.0264 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.014 },
                ToolSpec { key: "DT".into(), curve: "DT".into(), sigma: 1.951 },
                ToolSpec { key: "GR".into(), curve: "GR".into(), sigma: 6.0 },
                ToolSpec { key: "CT".into(), curve: "RES_DEEP".into(), sigma: 0.0 },
            ],
            apply_well_ids: vec![wid.to_string()],
            output_prefix: "MM".into(),
            unity: true,
            fluid: Some(props),
            ftemp_curve: None,
            recon_qc: false,
            sw_model: SwModel::Indonesia,
            porosity_source: PorositySource::Cec,
            enforce_porosity: true,
            enforce_bndwat: true,
            enforce_water_mud: true,
            sigma_constraint: 0.01,
        };
        let res = run_sandimin(&db, &req, None);
        assert!(res.error.is_none(), "err={:?}", res.error);
        assert!(res.wells[0].rows_solved > 0, "no samples solved");
        let c = db.lock().unwrap();
        let cols = fetch_curve_frame(&c, &wid.to_string(), &["MM_SWE".into(), "MM_PHIE".into()]).unwrap().1;
        let mean = |v: &[f32]| {
            let f: Vec<f32> = v.iter().copied().filter(|x| x.is_finite()).collect();
            assert!(!f.is_empty(), "no finite samples");
            f.iter().sum::<f32>() / f.len() as f32
        };
        let swe = mean(&cols["MM_SWE"]);
        let phie_out = mean(&cols["MM_PHIE"]);
        assert!((swe - sw_true as f32).abs() < 0.02, "post-solve SWE {swe}, want {sw_true}");
        assert!((phie_out - phie as f32).abs() < 0.02, "PHIE {phie_out}, want {phie}");
    }

    #[test]
    fn core_fit_rms_bias_tolerance_and_nan_skip() {
        // Hand-computed NUMERIC LITERALS so a slip in the RMS/bias expression fails here rather
        // than being confirmed against itself.
        let depth = [2000.0f32, 2000.5, 2001.0, 2001.5];
        let model = [0.20f32, 0.25, f32::NAN, 0.30];
        let plugs = [
            (2000.05f32, 0.22f32), // → sample 0 (dd 0.05):            e = 0.20 − 0.22 = −0.02
            (2001.02, 0.26),       // sample 2 is NaN → next nearest is sample 3 (dd 0.48 < 0.52):
            //                        e = 0.30 − 0.26 = +0.04
            (2500.0, 0.50), // nothing within 1.0 m → dropped entirely
        ];
        let f = core_fit(&depth, &model, &plugs).expect("two plugs should match");
        assert_eq!(f.n, 2, "the out-of-tolerance plug must be dropped, not matched");
        // sse = 0.0004 + 0.0016 = 0.002 → rms = sqrt(0.001)
        assert!((f.rms - 0.031_622_78).abs() < 1e-5, "rms {}", f.rms);
        // bias = (−0.02 + 0.04)/2 = +0.01 — signed, so it says the model reads HIGH on balance.
        assert!((f.bias - 0.01).abs() < 1e-6, "bias {}", f.bias);

        // The NaN sample is skipped, never matched: a plug essentially on top of it ties to the
        // nearest SOLVED neighbour (2001.5 at 0.48 beats 2000.5 at 0.52) instead of being lost.
        assert_eq!(nearest_solved(&depth, &model, 2001.02), Some(3));
        // Exactly equidistant between two solved samples, the first wins — deterministic, so the
        // statistic never depends on iteration order.
        assert_eq!(nearest_solved(&depth, &model, 2001.0), Some(1));

        // No plugs, and plugs that all miss, report absence — not a zero that would read as a
        // perfect fit.
        assert!(core_fit(&depth, &model, &[]).is_none());
        assert!(core_fit(&depth, &model, &[(3000.0, 0.2)]).is_none());
        // An all-NaN model can never match.
        assert!(core_fit(&depth, &[f32::NAN; 4], &[(2000.0, 0.2)]).is_none());
    }

    #[test]
    fn multimin_reports_core_fits_only_for_wells_with_core() {
        // Quartz + water, forward-modelled at vq = 0.70 / vw = 0.30 so the solver recovers them.
        // With a single SOLID component the predicted grain density is the quartz endpoint exactly,
        // whatever the volumes — so core ρg planted AT that endpoint must give ~0 RMS, and a plug
        // offset by +0.10 must give rms ≈ 0.10 with a NEGATIVE bias (model reads low vs core).
        let q = lib_get("Quartz");
        // Zone-less water so the nuclear tools see it: a zone-"U" fluid is invisible to RHOB/NPHI/GR
        // (those read the FLUSHED zone, i.e. the X fluids), which would leave only unity to place the
        // water. Same trick the mineral-recovery solver tests use.
        let mut w = lib_get("Water Sw");
        w.zone = "".into();
        let ep = |c: &Component, k: &str| c.endpoints[&k.to_string()];
        let (vq, vw) = (0.70f64, 0.30f64);
        let rho_q = ep(&q, "RHOB");
        let n = 6usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let mix = |k: &str| (vq * ep(&q, k) + vw * ep(&w, k)) as f32;

        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let cored = uuid::Uuid::new_v4();
        let dry = uuid::Uuid::new_v4();
        for (id, name) in [(cored, "MM-CORE"), (dry, "MM-NOCORE")] {
            crate::db::insert_well(&conn, id, name, None, None, None).unwrap();
            crate::db::insert_standard_curves(
                &conn,
                id,
                depth.clone(),
                vec![mix("GR"); n],
                vec![f32::NAN; n],
                vec![mix("NPHI"); n],
                vec![mix("RHOB"); n],
                vec![f32::NAN; n],
                vec![f32::NAN; n],
            )
            .unwrap();
        }
        // Three good plugs on the cored well: two ρg exactly on the quartz endpoint, one +0.10 high.
        // A FOURTH carries unit garbage — φ in percent and a 999.25 sentinel ρg — and sits within
        // depth tolerance of a real sample, so only the VALUE gate can reject it.
        let cd = vec![2000.1f32, 2001.05, 2002.4, 2002.45];
        let cpor = vec![0.30f32, 0.30, 0.30, 30.0];
        let cgd = vec![rho_q as f32, rho_q as f32, rho_q as f32 + 0.10, 999.25];
        let nanv = vec![f32::NAN; 4];
        crate::db::insert_core_data(&conn, &cored.to_string(), "RAW", None, &cd, &cpor, &nanv, &cgd, &nanv).unwrap();
        let db = Mutex::new(conn);

        let req = SandiminRequest {
            input_set: None,
            output_set: None,
            custody: crate::workflow::test_run_custody(),
            components: vec![q.clone(), w.clone()],
            tools: vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.0264 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.014 },
                ToolSpec { key: "GR".into(), curve: "GR".into(), sigma: 6.0 },
            ],
            apply_well_ids: vec![cored.to_string(), dry.to_string()],
            output_prefix: "MM".into(),
            unity: true,
            fluid: None,
            ftemp_curve: None,
            recon_qc: false,
            sw_model: SwModel::LinearDw,
            porosity_source: PorositySource::Cec,
            enforce_porosity: true,
            enforce_bndwat: true,
            enforce_water_mud: true,
            sigma_constraint: 0.01,
        };
        let res = run_sandimin(&db, &req, None);
        assert!(res.error.is_none(), "err={:?}", res.error);
        let cw = res.wells.iter().find(|w| w.well_id == cored.to_string()).expect("cored well result");
        let dw = res.wells.iter().find(|w| w.well_id == dry.to_string()).expect("dry well result");
        assert!(cw.rows_solved > 0, "no samples solved");

        let gd = cw.core_gd.as_ref().expect("grain-density fit on the cored well");
        assert_eq!(gd.n, 3, "the three valid plugs tie in; the 999.25 sentinel must be gated out");
        // Two exact + one 0.10 off → rms = sqrt(0.01/3) ≈ 0.0577, bias = −0.10/3 ≈ −0.0333.
        assert!((gd.rms - 0.057_735).abs() < 2e-3, "grain-density rms {}", gd.rms);
        assert!(gd.bias < 0.0, "model reads LOW vs core here, so bias must be negative: {}", gd.bias);
        assert!((gd.bias + 0.033_33).abs() < 2e-3, "grain-density bias {}", gd.bias);

        let phit = cw.core_phit.as_ref().expect("PHIT fit on the cored well");
        let phie = cw.core_phie.as_ref().expect("PHIE fit on the cored well");
        assert_eq!(phit.n, 3, "the percent-unit φ plug must be gated out, not fitted");
        assert_eq!(phie.n, 3);
        assert!(phit.rms < 0.02, "solved PHIT should sit on the planted 0.30 core φ: {}", phit.rms);
        assert!(phie.rms < 0.02, "solved PHIE should sit on the planted 0.30 core φ: {}", phie.rms);

        // The well without core rows reports absence on every channel.
        assert!(dw.core_gd.is_none() && dw.core_phit.is_none() && dw.core_phie.is_none());
    }

    #[test]
    fn sw_dual_nonlinear_hand_computed_and_conversion() {
        // Hand-computed NUMERIC LITERAL (independent of the function's own expression) — a coefficient
        // or exponent slip fails here rather than being confirmed self-referentially.
        // φt=0.3, Swb=0.2, Cw=2, Cwb=5, m=n=2, a=1, SWT_true=0.6:
        //   CT = (φt^m·SWT^n/a)·(Cw + (Cwb−Cw)·Swb/SWT) = (0.09·0.36)·(2 + 3·0.2/0.6) = 0.0324·3 = 0.0972
        //   ⇒ Rt = 1/0.0972 = 10.2880658…
        let swt = sw_dual_nonlinear(10.2880658436, 0.3, 0.2, 2.0, 5.0, 2.0, 2.0, 1.0);
        assert!((swt - 0.6).abs() < 1e-4, "dual-water SWT {swt}, want 0.6");

        // The effective conversion the run path applies: v_bw = Swb·φt = 0.06 ⇒ φe = 0.24;
        // free water = (SWT−Swb)·φt = 0.4·0.3 = 0.12 ⇒ SWE = 0.12/0.24 = 0.5 (hand-computed).
        let (phit, v_bw): (f64, f64) = (0.3, 0.06);
        let swe = ((swt * phit - v_bw) / (phit - v_bw)).clamp(0.0, 1.0);
        assert!((swe - 0.5).abs() < 1e-3, "dual-water SWE {swe}, want 0.5");

        // General n exercises the bisection branch: forward-model CT, invert, recover SWT.
        let (m, n, a, cw, cwb): (f64, f64, f64, f64, f64) = (1.8, 2.3, 1.0, 1.5, 4.0);
        let (phit2, swb2, swt2): (f64, f64, f64) = (0.28, 0.15, 0.45);
        let ct = (phit2.powf(m) * swt2.powf(n) / a) * (cw + (cwb - cw) * swb2 / swt2);
        let back = sw_dual_nonlinear(1.0 / ct, phit2, swb2, cw, cwb, m, n, a);
        assert!((back - swt2).abs() < 1e-3, "dual-water general-n round trip {back}, want {swt2}");

        // High conductivity (very low Rt) saturates to SWT = 1; non-physical inputs → NaN.
        assert!((sw_dual_nonlinear(0.01, 0.3, 0.2, 2.0, 5.0, 2.0, 2.0, 1.0) - 1.0).abs() < 1e-9);
        assert!(sw_dual_nonlinear(-1.0, 0.3, 0.2, 2.0, 5.0, 2.0, 2.0, 1.0).is_nan());
        assert!(sw_dual_nonlinear(10.0, 0.0, 0.2, 2.0, 5.0, 2.0, 2.0, 1.0).is_nan());
        assert!(sw_dual_nonlinear(10.0, 0.3, 0.2, 0.0, 5.0, 2.0, 2.0, 1.0).is_nan());
        // Sub-linear n<1 (non-physical; would diverge at Swt→0 and silently zero SWE) → NaN, not 0.
        assert!(sw_dual_nonlinear(4.63, 0.3, 0.2, 2.0, 5.0, 2.0, 0.5, 1.0).is_nan(), "n<1 must be rejected");
    }

    #[test]
    fn sw_archie_hand_computed() {
        // Hand-computed literals: Swt = (a·Rw/(φt^m·Rt))^(1/n).
        // φt=0.2, Rw=0.1, m=n=2, a=1 ⇒ Swt²=0.1/(0.04·Rt); Rt=10 ⇒ Swt²=0.25 ⇒ Swt=0.5.
        assert!((sw_archie(10.0, 0.2, 0.1, 2.0, 2.0, 1.0) - 0.5).abs() < 1e-9, "Archie n=2");
        // n=3: Swt³=0.1/(0.04·Rt); Rt=20 ⇒ Swt³=0.125 ⇒ Swt=0.5.
        assert!((sw_archie(20.0, 0.2, 0.1, 2.0, 3.0, 1.0) - 0.5).abs() < 1e-9, "Archie n=3");
        // Clean 100% water (very low Rt) clamps to 1; non-physical inputs → NaN.
        assert!((sw_archie(0.01, 0.2, 0.1, 2.0, 2.0, 1.0) - 1.0).abs() < 1e-9);
        assert!(sw_archie(-1.0, 0.2, 0.1, 2.0, 2.0, 1.0).is_nan());
        assert!(sw_archie(10.0, 0.0, 0.1, 2.0, 2.0, 1.0).is_nan());
        assert!(sw_archie(10.0, 0.2, 0.0, 2.0, 2.0, 1.0).is_nan());
        // Archie ≡ Indonesia with Vsh=0 (both reduce to the clean-sand power law).
        let arch = sw_archie(15.0, 0.22, 0.08, 2.0, 2.0, 1.0);
        let indo0 = sw_indonesia(15.0, 0.22, 0.0, 0.08, 4.0, 2.0, 2.0, 1.0, 1.0);
        assert!((arch - indo0).abs() < 1e-9, "Archie vs Indonesia(Vsh=0): {arch} vs {indo0}");
    }

    /// SB-SAT-023 / SB-SAT-T34-T36. Source: `12_saturation.md:1337-1354` — the effective back-out
    /// `SWE = MAX((SWT − Swb)/(1 − Swb), 0)` takes a PER-MODEL `Swb`: `1 − φe/φt` for the
    /// Archie/Waxman-Smits/dual-water group, **`Qvn = clamp(Vsh·φt_sh/φt, 0, 1)` for Juhász**
    /// (Geolog `sw_juha.lls:262`, Techlog agree; the two forms are NOT equal). `Swb = 1` yields
    /// `SWE = 1`, never a divide-by-zero. The inverse pair `SwT = Sw(1−Swb) + Swb` ships, and a
    /// round trip through the pair is the identity.
    ///
    /// The dossier fixture is the arm that proves the back-out is genuinely per model: Qvn 0.42
    /// against `1 − φe/φt` 0.20 moves SWE by tens of saturation units **while SWT matches
    /// exactly** — the defect was precisely that the correct Qvn was computed at `:456` and then
    /// overridden by the blanket porosity-volume conversion.
    #[test]
    fn the_effective_backout_swb_is_per_model_and_the_inverse_pair_round_trips_as_the_identity() {
        // A. Swb = 1 is all bound water: SWE = 1, not a division by zero, in both directions.
        assert_eq!(swe_from_swt(0.73, 1.0), 1.0);
        assert_eq!(swt_from_swe(0.73, 1.0), 1.0);

        // B. The pair is the inverse pair, and the round trip is the identity (Swb < 1 — at
        //    Swb = 1 the map is deliberately non-invertible: everything is bound water).
        for swb in [0.0, 0.2, 0.42, 0.9] {
            for sw in [0.0, 0.31, 0.5, 1.0] {
                let there = swt_from_swe(sw, swb);
                assert!(
                    (swe_from_swt(there, swb) - sw).abs() < 1e-12,
                    "round trip must be the identity at Swb {swb}, Sw {sw}"
                );
            }
        }
        // The floor is MAX(.., 0): an SWT below Swb reads all-bound, never negative.
        assert_eq!(swe_from_swt(0.1, 0.2), 0.0);

        // C. The dossier fixture: φt 0.25, φe 0.20 → porosity-volume Swb 0.20; Vsh 0.35 with
        //    φt_sh 0.30 → Qvn 0.42. One SWT, two rules, two answers tens of units apart.
        let (phit, phie, vsh, phit_sh) = (0.25, 0.20, 0.35, 0.30);
        assert!((juhasz_qvn(vsh, phit_sh, phit) - 0.42).abs() < 1e-12);
        let swt = 0.60;
        let (swe_arch, rule_arch) =
            effective_backout(SwModel::ArchieTotal, swt, phie, phit, vsh, phit_sh);
        let (swe_juh, rule_juh) = effective_backout(SwModel::Juhasz, swt, phie, phit, vsh, phit_sh);
        assert!((swe_arch - 0.50).abs() < 1e-9, "porosity-volume rule: (0.60-0.20)/0.80");
        assert!((swe_juh - (0.60 - 0.42) / 0.58).abs() < 1e-9, "Juhász rule uses Qvn");
        assert!(
            (swe_arch - swe_juh) > 0.15,
            "the two rules must disagree by tens of saturation units on the same SWT"
        );

        // D. Which rule applied is RECORDED, never implicit — the solver's construction collapses
        //    1 − φe/φt with v_bw/φt, so the name is the only place the distinction survives.
        assert_eq!(rule_arch, "porosity_volume_1_minus_phie_over_phit");
        assert_eq!(rule_juh, "juhasz_qvn");
        assert_eq!(effective_backout_rule(SwModel::WaxmanSmits), "porosity_volume_1_minus_phie_over_phit");
        assert_eq!(effective_backout_rule(SwModel::DualWaterNonlinear), "porosity_volume_1_minus_phie_over_phit");
        assert_eq!(effective_backout_rule(SwModel::Juhasz), "juhasz_qvn");

        // E. The SxoT lift is the SAME published formula on Sxo — one implementation, stated.
        assert!((swt_from_swe(0.85, 0.2) - (0.85 * 0.8 + 0.2)).abs() < 1e-12);

        // F. The rule is recorded ON THE RESULT of a real run, not merely derivable — this is the
        //    "record which rule was applied" clause, and it survives to the UI.
        let (db, wid, _phie) = ftemp_test_well("MM-SWBRULE", 100.0, 0.40);
        let res = run_sandimin(&db, &ftemp_req(wid, 100.0, None), None);
        assert!(res.error.is_none(), "err={:?}", res.error);
        assert_eq!(
            res.swb_rule.as_deref(),
            Some("porosity_volume_1_minus_phie_over_phit"),
            "a post-solve run must state its effective back-out rule"
        );
    }

    #[test]
    fn sw_juhasz_hand_computed() {
        // Normalized-Qv wet-shale form. φt=0.3, Vsh=0.2, φ_sh=0.1, Cw=10 (Rw=0.1), Rsh=4, m=n=2:
        //   QVN = 0.2·0.1/0.3 = 1/15;  Cwsh = 1/(4·0.1²) = 25;  lin = QVN·(25−10) = 1.0.
        // Forward at SWT=0.5: Ct/φt^m = 10·0.25 + 1.0·0.5 = 3.0 ⇒ Ct = 0.27 ⇒ Rt = 3.703703703703…
        assert!(
            (sw_juhasz(1.0 / 0.27, 0.3, 0.2, 10.0, 4.0, 0.1, 2.0, 2.0) - 0.5).abs() < 1e-9,
            "Juhász n=2 closed form"
        );
        // General n=3 via bisection: Ct/φt^m = 10·0.125 + 1.0·0.25 = 1.5 ⇒ Ct = 0.135 ⇒ Rt = 7.407…
        assert!(
            (sw_juhasz(1.0 / 0.135, 0.3, 0.2, 10.0, 4.0, 0.1, 2.0, 3.0) - 0.5).abs() < 1e-6,
            "Juhász n=3 bisection"
        );
        // Vsh=0 ⇒ QVN=0 ⇒ clean-sand Archie (Rw = 1/Cw, a = 1).
        let juh0 = sw_juhasz(12.0, 0.28, 0.0, 10.0, 4.0, 0.1, 2.0, 2.0);
        let arch = sw_archie(12.0, 0.28, 0.1, 2.0, 2.0, 1.0);
        assert!((juh0 - arch).abs() < 1e-9, "Juhász(Vsh=0) vs Archie: {juh0} vs {arch}");
        // Non-physical inputs → NaN (rsh, φ_sh, rt, and the sub-linear-n guard inherited from the core).
        assert!(sw_juhasz(3.7, 0.3, 0.2, 10.0, 0.0, 0.1, 2.0, 2.0).is_nan());
        assert!(sw_juhasz(3.7, 0.3, 0.2, 10.0, 4.0, 0.0, 2.0, 2.0).is_nan());
        assert!(sw_juhasz(-1.0, 0.3, 0.2, 10.0, 4.0, 0.1, 2.0, 2.0).is_nan());
        assert!(sw_juhasz(3.7, 0.3, 0.2, 10.0, 4.0, 0.1, 2.0, 0.5).is_nan());
    }

    #[test]
    fn sw_waxman_smits_hand_computed() {
        // Ct/φt^m = Cw·Swt^n + B·Qv·Swt^(n−1). φt=0.2, Cw=5, Qv=0.3, B=4, m=n=2, SWT=0.5:
        //   Ct/φt^m = 5·0.25 + 4·0.3·0.5 = 1.85 ⇒ Ct = 1.85·0.04 = 0.074 ⇒ Rt = 13.513513…
        assert!(
            (sw_waxman_smits(1.0 / 0.074, 0.2, 0.3, 5.0, 4.0, 2.0, 2.0) - 0.5).abs() < 1e-9,
            "Waxman-Smits n=2 closed form"
        );
        // General n=3 via bisection: Ct/φt^m = 5·0.125 + 4·0.3·0.25 = 0.925 ⇒ Ct = 0.037 ⇒ Rt = 27.027…
        assert!(
            (sw_waxman_smits(1.0 / 0.037, 0.2, 0.3, 5.0, 4.0, 2.0, 3.0) - 0.5).abs() < 1e-6,
            "Waxman-Smits n=3 bisection"
        );
        // Qv=0 ⇒ excess term vanishes ⇒ clean-sand Archie (Rw = 1/Cw, a = 1).
        let ws0 = sw_waxman_smits(12.0, 0.28, 0.0, 10.0, 4.0, 2.0, 2.0);
        let arch = sw_archie(12.0, 0.28, 0.1, 2.0, 2.0, 1.0);
        assert!((ws0 - arch).abs() < 1e-9, "Waxman-Smits(Qv=0) vs Archie: {ws0} vs {arch}");
        // B=0 is also the clean-sand limit (no counterion conductance).
        assert!((sw_waxman_smits(12.0, 0.28, 0.5, 10.0, 0.0, 2.0, 2.0) - arch).abs() < 1e-9, "B=0 ⇒ Archie");
        // Adding Qv (excess conductivity) at fixed Rt LOWERS the apparent Sw vs the clean-sand read.
        let ws_shaly = sw_waxman_smits(12.0, 0.28, 0.5, 10.0, 4.0, 2.0, 2.0);
        assert!(ws_shaly < ws0, "excess conductivity lowers Sw: {ws_shaly} vs {ws0}");
        // Non-physical inputs → NaN (rt, phit, cw, and the sub-linear-n guard from the core root).
        assert!(sw_waxman_smits(-1.0, 0.2, 0.3, 5.0, 4.0, 2.0, 2.0).is_nan());
        assert!(sw_waxman_smits(13.5, 0.0, 0.3, 5.0, 4.0, 2.0, 2.0).is_nan());
        assert!(sw_waxman_smits(13.5, 0.2, 0.3, 0.0, 4.0, 2.0, 2.0).is_nan());
        assert!(sw_waxman_smits(13.5, 0.2, 0.3, 5.0, 4.0, 2.0, 0.5).is_nan());
    }

    #[test]
    fn waxman_b_matches_juhasz_fit() {
        // Juhász B(T,Rw) = (−1.28 + 0.225T − 0.0004059T²)/(1 + (0.045T − 0.27)·Rw^1.23).
        // Hand values (T °C, Rw ohm·m):
        //   B(25, 0.10): num = 4.0913125, den = 1 + 0.855·0.1^1.23 = 1.0503461 ⇒ 3.89520
        assert!((waxman_b(25.0, 0.10) - 3.89520).abs() < 2e-3, "B(25,0.1) got {}", waxman_b(25.0, 0.10));
        //   B(100, 0.05): num = 17.161, den = 1 + 4.23·0.05^1.23 = 1.1061360 ⇒ 15.5144
        assert!((waxman_b(100.0, 0.05) - 15.5144).abs() < 5e-3, "B(100,0.05) got {}", waxman_b(100.0, 0.05));
        // B rises with temperature (counterion mobility) at fixed Rw…
        assert!(waxman_b(100.0, 0.10) > waxman_b(25.0, 0.10), "B increases with T");
        // …and falls as the water freshens (Rw up) at fixed T, since (0.045T−0.27) > 0 for T > 6 °C.
        assert!(waxman_b(80.0, 1.0) < waxman_b(80.0, 0.05), "B decreases as Rw rises");
        // Rw ≤ 0 (or unset) ⇒ salinity-independent numerator, the saline-limit B.
        assert!((waxman_b(25.0, 0.0) - 4.0913125).abs() < 1e-9, "Rw=0 falls back to numerator");
        assert!((waxman_b(25.0, -1.0) - 4.0913125).abs() < 1e-9, "Rw<0 falls back to numerator");
        // Below ~6 °C the numerator is negative; B is clamped to 0 (never a negative conductance).
        assert_eq!(waxman_b(0.0, 0.10), 0.0, "sub-zero-numerator B clamps to 0");
        // Always finite and non-negative across a wide reservoir range.
        for &t in &[10.0f64, 60.0, 120.0, 200.0] {
            for &rw in &[0.01f64, 0.1, 1.0, 10.0] {
                let b = waxman_b(t, rw);
                assert!(b.is_finite() && b >= 0.0, "B({t},{rw}) = {b}");
            }
        }
    }

    #[test]
    fn dual_water_nonlinear_post_solve_recovers_known_sw() {
        // Same forward model as indonesia_post_solve_recovers_known_sw but sw_model = DualWaterNonlinear.
        // With no clay/bound-water component Swb=0, so the dual-water form reduces to Archie; a deep Sw
        // forward-modelled through Archie must come back as SWE, PHIE untouched. Exercises the run-path
        // wiring (post-solve gate, CT-stays-in inversion, φe redistribution) for the new model.
        let q = lib_get("Quartz");
        let wsxo = lib_get("Water Sxo");
        let osxo = lib_get("Oil Sxo");
        let wsw = lib_get("Water Sw");
        let osw = lib_get("Oil Sw");
        let ep = |c: &Component, k: &str| c.endpoints[&k.to_string()];
        let (vq, vwx, vox) = (0.70, 0.15, 0.15);
        let (phie, sw_true, rw): (f64, f64, f64) = (0.30, 0.35, 0.10);
        let d = (phie.powf(2.0) / (1.0 * rw)).sqrt();
        let rt = 1.0 / (d * d * sw_true.powf(2.0));

        let n = 6usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let mix = |k: &str| (vq * ep(&q, k) + vwx * ep(&wsxo, k) + vox * ep(&osxo, k)) as f32;
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "MM-DWNL", None, None, None).unwrap();
        crate::db::insert_standard_curves(
            &conn,
            wid,
            depth,
            vec![mix("GR"); n],
            vec![rt as f32; n],
            vec![mix("NPHI"); n],
            vec![mix("RHOB"); n],
            vec![mix("DT"); n],
            vec![f32::NAN; n],
        )
        .unwrap();
        let db = Mutex::new(conn);
        let props = FluidProps {
            rw,
            rw_temp_f: 100.0,
            rmf: 0.1,
            rmf_temp_f: 100.0,
            ftemp_f: 100.0,
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
            indonesia_k: 1.0,
            simandoux_c: 1.0,
            phit_sh: 0.1,
            ws_b: 0.0,
        };
        let req = SandiminRequest {
            input_set: None,
            output_set: None,
            custody: crate::workflow::test_run_custody(),
            components: vec![q, wsxo, osxo, wsw, osw],
            tools: vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.0264 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.014 },
                ToolSpec { key: "DT".into(), curve: "DT".into(), sigma: 1.951 },
                ToolSpec { key: "GR".into(), curve: "GR".into(), sigma: 6.0 },
                ToolSpec { key: "CT".into(), curve: "RES_DEEP".into(), sigma: 0.0 },
            ],
            apply_well_ids: vec![wid.to_string()],
            output_prefix: "MM".into(),
            unity: true,
            fluid: Some(props),
            ftemp_curve: None,
            recon_qc: false,
            sw_model: SwModel::DualWaterNonlinear,
            porosity_source: PorositySource::Cec,
            enforce_porosity: true,
            enforce_bndwat: true,
            enforce_water_mud: true,
            sigma_constraint: 0.01,
        };
        let res = run_sandimin(&db, &req, None);
        assert!(res.error.is_none(), "err={:?}", res.error);
        assert!(res.wells[0].rows_solved > 0, "no samples solved");
        let c = db.lock().unwrap();
        let cols = fetch_curve_frame(&c, &wid.to_string(), &["MM_SWE".into(), "MM_PHIE".into()]).unwrap().1;
        let mean = |v: &[f32]| {
            let f: Vec<f32> = v.iter().copied().filter(|x| x.is_finite()).collect();
            assert!(!f.is_empty(), "no finite samples");
            f.iter().sum::<f32>() / f.len() as f32
        };
        let swe = mean(&cols["MM_SWE"]);
        let phie_out = mean(&cols["MM_PHIE"]);
        assert!((swe - sw_true as f32).abs() < 0.02, "post-solve SWE {swe}, want {sw_true}");
        assert!((phie_out - phie as f32).abs() < 0.02, "PHIE {phie_out}, want {phie}");
    }

    #[test]
    fn waxman_smits_post_solve_recovers_known_sw() {
        // Clean-sand forward model (no clay ⇒ Qv = 0 ⇒ Waxman-Smits collapses to Archie), the same
        // rock as dual_water_nonlinear_post_solve_recovers_known_sw. Exercises the run-path wiring for
        // the model: post-solve gate, qv_num from (empty) clays, waxman_b(T,Rw) lookup, φe redistribution.
        // A deep Sw forward-modelled through Archie must return as SWE with PHIE untouched.
        let q = lib_get("Quartz");
        let wsxo = lib_get("Water Sxo");
        let osxo = lib_get("Oil Sxo");
        let wsw = lib_get("Water Sw");
        let osw = lib_get("Oil Sw");
        let ep = |c: &Component, k: &str| c.endpoints[&k.to_string()];
        let (vq, vwx, vox) = (0.70, 0.15, 0.15);
        let (phie, sw_true, rw): (f64, f64, f64) = (0.30, 0.35, 0.10);
        let d = (phie.powf(2.0) / (1.0 * rw)).sqrt();
        let rt = 1.0 / (d * d * sw_true.powf(2.0));

        let n = 6usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let mix = |k: &str| (vq * ep(&q, k) + vwx * ep(&wsxo, k) + vox * ep(&osxo, k)) as f32;
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "MM-WS", None, None, None).unwrap();
        crate::db::insert_standard_curves(
            &conn,
            wid,
            depth,
            vec![mix("GR"); n],
            vec![rt as f32; n],
            vec![mix("NPHI"); n],
            vec![mix("RHOB"); n],
            vec![mix("DT"); n],
            vec![f32::NAN; n],
        )
        .unwrap();
        let db = Mutex::new(conn);
        let props = FluidProps {
            rw,
            rw_temp_f: 100.0,
            rmf: 0.1,
            rmf_temp_f: 100.0,
            ftemp_f: 100.0,
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
            indonesia_k: 1.0,
            simandoux_c: 1.0,
            phit_sh: 0.1,
            ws_b: 0.0,
        };
        let req = SandiminRequest {
            input_set: None,
            output_set: None,
            custody: crate::workflow::test_run_custody(),
            components: vec![q, wsxo, osxo, wsw, osw],
            tools: vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.0264 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.014 },
                ToolSpec { key: "DT".into(), curve: "DT".into(), sigma: 1.951 },
                ToolSpec { key: "GR".into(), curve: "GR".into(), sigma: 6.0 },
                ToolSpec { key: "CT".into(), curve: "RES_DEEP".into(), sigma: 0.0 },
            ],
            apply_well_ids: vec![wid.to_string()],
            output_prefix: "MM".into(),
            unity: true,
            fluid: Some(props),
            ftemp_curve: None,
            recon_qc: false,
            sw_model: SwModel::WaxmanSmits,
            porosity_source: PorositySource::Cec,
            enforce_porosity: true,
            enforce_bndwat: true,
            enforce_water_mud: true,
            sigma_constraint: 0.01,
        };
        let res = run_sandimin(&db, &req, None);
        assert!(res.error.is_none(), "err={:?}", res.error);
        assert!(res.wells[0].rows_solved > 0, "no samples solved");
        let c = db.lock().unwrap();
        let cols = fetch_curve_frame(&c, &wid.to_string(), &["MM_SWE".into(), "MM_PHIE".into()]).unwrap().1;
        let mean = |v: &[f32]| {
            let f: Vec<f32> = v.iter().copied().filter(|x| x.is_finite()).collect();
            assert!(!f.is_empty(), "no finite samples");
            f.iter().sum::<f32>() / f.len() as f32
        };
        let swe = mean(&cols["MM_SWE"]);
        let phie_out = mean(&cols["MM_PHIE"]);
        assert!((swe - sw_true as f32).abs() < 0.02, "post-solve SWE {swe}, want {sw_true}");
        assert!((phie_out - phie as f32).abs() < 0.02, "PHIE {phie_out}, want {phie}");
    }

    #[test]
    fn waxman_smits_shaly_recovers_known_sw() {
        // Shaly-sand round trip that EXERCISES the Qv assembly (Σ v_clay·CEC·ρ / φt), which the clean-sand
        // test above cannot (Qv=0 there). A known clay volume + deep Sw forward-model Rt through the full
        // Waxman-Smits conductivity; the solve must recover Vsh, PHIE and — via B·Qv — the deep Sw. Qv here
        // is hand-assembled longhand, so a factor/φt/zone bug in the solver's qv_num breaks the round trip.
        let q = lib_get("Quartz");
        let clay = lib_get("Illite");
        let wsxo = lib_get("Water Sxo");
        let osxo = lib_get("Oil Sxo");
        let wsw = lib_get("Water Sw");
        let osw = lib_get("Oil Sw");
        let ep = |c: &Component, k: &str| c.endpoints[&k.to_string()];
        let (vq, vcl, vwx, vox) = (0.55, 0.15, 0.15, 0.15); // matrix 0.70, φe 0.30, Vsh 0.15, flushed Sxo 0.5
        let (phie, sw_true, rw): (f64, f64, f64) = (0.30, 0.35, 0.10); // deep Sw; Rw at formation T
        let (m, n): (f64, f64) = (2.0, 2.0);
        // Cw at formation T: rw_temp == ftemp below, so Cw = 1/Rw exactly. t_c from the same ftemp.
        let cw = 1.0 / rw;
        let t_c = (100.0 - 32.0) * 5.0 / 9.0;
        let b = waxman_b(t_c, rw);
        // Qv = v_clay·CEC·ρ_clay / φt (meq/mL), written longhand — independent of the solver's qv_num.
        // φt = φe here (no bound-water component in the model).
        let qv = vcl * clay.cec * ep(&clay, "RHOB") / phie;
        // Forward Ct = φt^m·(Cw·Swt^n + B·Qv·Swt^(n−1)); Rt = 1/Ct.
        let ct = phie.powf(m) * (cw * sw_true.powf(n) + b * qv * sw_true.powf(n - 1.0));
        let rt = 1.0 / ct;

        let nrows = 6usize;
        let depth: Vec<f32> = (0..nrows).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let mix =
            |k: &str| (vq * ep(&q, k) + vcl * ep(&clay, k) + vwx * ep(&wsxo, k) + vox * ep(&osxo, k)) as f32;
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "MM-WS-SH", None, None, None).unwrap();
        crate::db::insert_standard_curves(
            &conn,
            wid,
            depth,
            vec![mix("GR"); nrows],
            vec![rt as f32; nrows],
            vec![mix("NPHI"); nrows],
            vec![mix("RHOB"); nrows],
            vec![mix("DT"); nrows],
            vec![f32::NAN; nrows],
        )
        .unwrap();
        let db = Mutex::new(conn);
        let props = FluidProps {
            rw,
            rw_temp_f: 100.0,
            rmf: 0.1,
            rmf_temp_f: 100.0,
            ftemp_f: 100.0,
            m,
            n,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
            indonesia_k: 1.0,
            simandoux_c: 1.0,
            phit_sh: 0.1,
            ws_b: 0.0,
        };
        let req = SandiminRequest {
            input_set: None,
            output_set: None,
            custody: crate::workflow::test_run_custody(),
            components: vec![q, clay, wsxo, osxo, wsw, osw],
            tools: vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.0264 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.014 },
                ToolSpec { key: "DT".into(), curve: "DT".into(), sigma: 1.951 },
                ToolSpec { key: "GR".into(), curve: "GR".into(), sigma: 6.0 },
                ToolSpec { key: "CT".into(), curve: "RES_DEEP".into(), sigma: 0.0 },
            ],
            apply_well_ids: vec![wid.to_string()],
            output_prefix: "MM".into(),
            unity: true,
            fluid: Some(props),
            ftemp_curve: None,
            recon_qc: false,
            sw_model: SwModel::WaxmanSmits,
            porosity_source: PorositySource::Cec,
            enforce_porosity: true,
            enforce_bndwat: true,
            enforce_water_mud: true,
            sigma_constraint: 0.01,
        };
        let res = run_sandimin(&db, &req, None);
        assert!(res.error.is_none(), "err={:?}", res.error);
        assert!(res.wells[0].rows_solved > 0, "no samples solved");
        let c = db.lock().unwrap();
        let cols = fetch_curve_frame(&c, &wid.to_string(), &["MM_SWE".into(), "MM_PHIE".into(), "MM_VSH".into()])
            .unwrap()
            .1;
        let mean = |v: &[f32]| {
            let f: Vec<f32> = v.iter().copied().filter(|x| x.is_finite()).collect();
            assert!(!f.is_empty(), "no finite samples");
            f.iter().sum::<f32>() / f.len() as f32
        };
        let swe = mean(&cols["MM_SWE"]);
        let phie_out = mean(&cols["MM_PHIE"]);
        let vsh_out = mean(&cols["MM_VSH"]);
        // Clay (hence Qv) is recovered, so the excess-conductivity term is real…
        assert!((vsh_out - vcl as f32).abs() < 0.02, "VSH {vsh_out}, want {vcl}");
        // …and Waxman-Smits inverts Rt back to the true deep Sw. An Archie read of the same Rt would be
        // ~0.44 (markedly high), so recovering 0.35 proves B·Qv propagated through qv_num end to end.
        assert!((swe - sw_true as f32).abs() < 0.02, "post-solve SWE {swe}, want {sw_true}");
        assert!((phie_out - phie as f32).abs() < 0.02, "PHIE {phie_out}, want {phie}");
    }

    #[test]
    fn fluid_calc_at_matches_and_moves_with_temperature() {
        let p = FluidProps {
            rw: 0.10,
            rw_temp_f: 77.0,
            rmf: 0.05,
            rmf_temp_f: 70.0,
            ftemp_f: 150.0,
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
            indonesia_k: 1.0,
            simandoux_c: 1.0,
            phit_sh: 0.1,
            ws_b: 0.0,
        };
        // fluid_calc IS fluid_calc_at at p.ftemp_f — bit-identical.
        let base = fluid_calc(&p);
        let same = fluid_calc_at(&p, p.ftemp_f);
        assert_eq!(base.cw, same.cw);
        assert_eq!(base.cmf, same.cmf);
        assert_eq!(base.cbw, same.cbw);
        assert_eq!(base.u_ct, same.u_ct);
        // Hotter formation ⇒ water more conductive (cw, cmf, cbw up); the α expansion and salinities
        // come from the Rw/Rmf sample temperatures, so they do NOT move with formation temperature.
        let hot = fluid_calc_at(&p, 250.0);
        assert!(hot.cw > base.cw, "cw rises with T: {} vs {}", hot.cw, base.cw);
        assert!(hot.cmf > base.cmf, "cmf rises with T");
        assert!(hot.cbw > base.cbw, "cbw rises with T");
        assert_eq!(hot.alpha_u, base.alpha_u, "α is salinity-driven, T-independent");
        assert_eq!(hot.alpha_x, base.alpha_x);
        assert_eq!(hot.salinity_w_ppm, base.salinity_w_ppm);
        assert_eq!(hot.w, base.w);
    }

    /// Shared clean-sand well for the FTEMP-curve integration tests: Rt is forward-modelled through
    /// Archie at `t_forward` °F for a known Sw. Returns (db, well id, components, sw_true).
    #[cfg(test)]
    fn ftemp_test_well(name: &str, t_forward: f64, sw_true: f64) -> (Mutex<Connection>, uuid::Uuid, f64) {
        let q = lib_get("Quartz");
        let wsxo = lib_get("Water Sxo");
        let osxo = lib_get("Oil Sxo");
        let ep = |c: &Component, k: &str| c.endpoints[&k.to_string()];
        let (vq, vwx, vox) = (0.70, 0.15, 0.15);
        let phie: f64 = 0.30;
        let rw_fwd = arps_f(0.10, 77.0, t_forward); // Rw at the forward formation temperature
        let rt = rw_fwd / (phie.powf(2.0) * sw_true.powf(2.0)); // clean sand, a=1, m=n=2
        let nrows = 6usize;
        let depth: Vec<f32> = (0..nrows).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let mix = |k: &str| (vq * ep(&q, k) + vwx * ep(&wsxo, k) + vox * ep(&osxo, k)) as f32;
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, name, None, None, None).unwrap();
        crate::db::insert_standard_curves(
            &conn,
            wid,
            depth.clone(),
            vec![mix("GR"); nrows],
            vec![rt as f32; nrows],
            vec![mix("NPHI"); nrows],
            vec![mix("RHOB"); nrows],
            vec![mix("DT"); nrows],
            vec![f32::NAN; nrows],
        )
        .unwrap();
        // A constant FTEMP_F curve at the forward temperature.
        crate::equations::write_computed_curve(&conn, &wid.to_string(), &depth, "FTEMP_F", &vec![t_forward as f32; nrows])
            .unwrap();
        (Mutex::new(conn), wid, phie)
    }

    #[cfg(test)]
    fn ftemp_req(wid: uuid::Uuid, scalar_ftemp: f64, ftemp_curve: Option<String>) -> SandiminRequest {
        let props = FluidProps {
            rw: 0.10,
            rw_temp_f: 77.0,
            rmf: 0.10,
            rmf_temp_f: 77.0,
            ftemp_f: scalar_ftemp,
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
            indonesia_k: 1.0,
            simandoux_c: 1.0,
            phit_sh: 0.1,
            ws_b: 0.0,
        };
        SandiminRequest {
            input_set: None,
            output_set: None,
            custody: crate::workflow::test_run_custody(),
            components: vec![
                lib_get("Quartz"),
                lib_get("Water Sxo"),
                lib_get("Oil Sxo"),
                lib_get("Water Sw"),
                lib_get("Oil Sw"),
            ],
            tools: vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.0264 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.014 },
                ToolSpec { key: "DT".into(), curve: "DT".into(), sigma: 1.951 },
                ToolSpec { key: "GR".into(), curve: "GR".into(), sigma: 6.0 },
                ToolSpec { key: "CT".into(), curve: "RES_DEEP".into(), sigma: 0.0 },
            ],
            apply_well_ids: vec![wid.to_string()],
            output_prefix: "MM".into(),
            unity: true,
            fluid: Some(props),
            ftemp_curve,
            recon_qc: false,
            sw_model: SwModel::ArchieTotal,
            porosity_source: PorositySource::Cec,
            enforce_porosity: true,
            enforce_bndwat: true,
            enforce_water_mud: true,
            sigma_constraint: 0.01,
        }
    }

    #[cfg(test)]
    fn ftemp_mean_swe(db: &Mutex<Connection>, wid: uuid::Uuid, req: &SandiminRequest) -> f32 {
        let res = run_sandimin(db, req, None);
        assert!(res.error.is_none(), "err={:?}", res.error);
        assert!(res.wells[0].rows_solved > 0, "no samples solved");
        let c = db.lock().unwrap();
        let cols = fetch_curve_frame(&c, &wid.to_string(), &["MM_SWE".into()]).unwrap().1;
        let f: Vec<f32> = cols["MM_SWE"].iter().copied().filter(|x| x.is_finite()).collect();
        assert!(!f.is_empty(), "no finite SWE");
        f.iter().sum::<f32>() / f.len() as f32
    }

    #[test]
    fn ftemp_curve_constant_equals_fixed_temperature() {
        // An FTEMP curve equal to the fixed temperature at every depth must reproduce the constant-T
        // run exactly (100 °F is exact in f32, so the per-sample fluid calc is bit-identical). Proves
        // the per-sample path collapses to the reviewed behaviour when temperature is uniform.
        let (db, wid, _phie) = ftemp_test_well("MM-FTEMP-EQ", 100.0, 0.40);
        let without = ftemp_mean_swe(&db, wid, &ftemp_req(wid, 100.0, None));
        let with = ftemp_mean_swe(&db, wid, &ftemp_req(wid, 100.0, Some("FTEMP_F".into())));
        assert_eq!(with, without, "constant FTEMP curve must match the fixed temperature exactly");
    }

    #[test]
    fn ftemp_curve_overrides_scalar_temperature() {
        // Rt is forward-modelled at a HOT formation temperature (200 °F). Reading it back with the
        // FTEMP curve (hot) recovers the true Sw; the fixed COLD scalar (100 °F) reads Sw too high —
        // colder water is more resistive, so the same Rt looks less water-bearing than it is. Confirms
        // the curve is applied and drives Cw the right way.
        let sw_true = 0.40;
        let (db, wid, _phie) = ftemp_test_well("MM-FTEMP-HOT", 200.0, sw_true);
        let with_curve = ftemp_mean_swe(&db, wid, &ftemp_req(wid, 100.0, Some("FTEMP_F".into())));
        let cold_scalar = ftemp_mean_swe(&db, wid, &ftemp_req(wid, 100.0, None));
        assert!((with_curve - sw_true as f32).abs() < 0.02, "FTEMP-curve SWE {with_curve}, want {sw_true}");
        assert!(cold_scalar > with_curve + 0.10, "cold fixed T over-reads Sw: {cold_scalar} vs {with_curve}");
    }

    #[test]
    fn ftemp_curve_out_of_range_falls_back() {
        // FTEMP samples outside the sane window — null sentinels below the floor (−999.25) OR above the
        // ceiling (9999) — must be ignored so the sample reverts to the fixed temperature. Selecting the
        // curve then can't corrupt a well whose temperature column is all bad data. Rt is forward-modelled
        // at the scalar 100 °F, so the fallback recovers the same Sw as running with no curve at all.
        let (db, wid, _phie) = ftemp_test_well("MM-FTEMP-NULL", 100.0, 0.40);
        {
            let c = db.lock().unwrap();
            let depth: Vec<f32> = (0..6).map(|i| 2000.0 + i as f32 * 0.5).collect();
            let vals: Vec<f32> = (0..6).map(|i| if i % 2 == 0 { 9999.0 } else { -999.25 }).collect();
            crate::equations::write_computed_curve(&c, &wid.to_string(), &depth, "FTEMP_F", &vals).unwrap();
        }
        let with = ftemp_mean_swe(&db, wid, &ftemp_req(wid, 100.0, Some("FTEMP_F".into())));
        let without = ftemp_mean_swe(&db, wid, &ftemp_req(wid, 100.0, None));
        assert_eq!(with, without, "out-of-range FTEMP samples must fall back to the fixed temperature");
    }

    #[test]
    fn ftemp_curve_recon_qc_decomposition_holds() {
        // Under an FTEMP curve the per-tool RECON decomposition must still satisfy Σ DIF²/n_tools = RECON²
        // — the conductivity tool's DIF has to be rebuilt from the per-sample (hot) row, not the static
        // constant-T row. The model is over-determined (dof = 2), so residuals are real; a decomposition
        // that used the wrong CT row would break the identity. Forward Rt is at 200 °F, scalar is 100 °F.
        let (db, wid, _phie) = ftemp_test_well("MM-FTEMP-RQC", 200.0, 0.40);
        let mut req = ftemp_req(wid, 100.0, Some("FTEMP_F".into()));
        req.recon_qc = true;
        let res = run_sandimin(&db, &req, None);
        assert!(res.error.is_none(), "err={:?}", res.error);
        let c = db.lock().unwrap();
        let dif_names = ["MM_RHOB_DIF", "MM_NPHI_DIF", "MM_DT_DIF", "MM_GR_DIF", "MM_CT_DIF"];
        let mut names: Vec<String> = vec!["MM_RECON".into()];
        names.extend(dif_names.iter().map(|s| s.to_string()));
        let cols = fetch_curve_frame(&c, &wid.to_string(), &names).unwrap().1;
        let n_tools = dif_names.len() as f64;
        let mut checked = 0;
        for i in 0..cols["MM_RECON"].len() {
            let recon = cols["MM_RECON"][i] as f64;
            if !recon.is_finite() {
                continue;
            }
            let ssq: f64 = dif_names
                .iter()
                .map(|k| {
                    let d = cols[*k][i] as f64;
                    d * d
                })
                .sum();
            assert!(
                (ssq / n_tools - recon * recon).abs() < 1e-4,
                "sample {i}: Σdif²/n = {} vs RECON² = {}",
                ssq / n_tools,
                recon * recon
            );
            checked += 1;
        }
        assert!(checked > 0, "no finite RECON samples checked");
    }
    /// AUDIT-2026-08-20 finding 11. The shared conductivity root has a `g(0) > 0` arm that is
    /// reachable at EXACTLY n = 1 (above it `Swt^(n-1) -> 0` kills the offset; Rust's 0^0 = 1
    /// leaves it standing). It means the clay term alone conducts more than the rock measures -
    /// no root in [0, 1] - and it used to return SWT = 0.0: a hundred per cent hydrocarbon,
    /// written as an ordinary curve, from the exact condition that evidences an over-estimated
    /// Qv rather than a pay leg.
    ///
    /// Pinned from BOTH sides, because the obvious over-correction - refusing n = 1 outright -
    /// passes the first half and blanks whole legal curves. n = 1 sits inside every declared
    /// N range, so a wet zone and an ordinary shaly-sand answer at n = 1 must both survive.
    #[test]
    fn a_conductivity_root_with_no_solution_refuses_instead_of_reporting_all_hydrocarbon() {
        // Rw = 1 ohm.m (Cw = 1 mho/m), phit = 0.20, m = 2, B.Qv = 2.0 mho/m - a heavily shaly
        // interval. a.Ct/phit^m is then 25.Ct, so Rt alone decides which arm fires.
        let ws = |rt: f64| sw_waxman_smits(rt, 0.20, 2.0, 1.0, 1.0, 2.0, 1.0);

        // A - Rt = 20: the rock measures 25 x 0.05 = 1.25 against a clay term of 2.0. The model
        // has broken down; the honest answer is MISSING, not zero water.
        assert!(
            ws(20.0).is_nan(),
            "an excess-conductivity term above the measured conductivity has no root in [0, 1]              and must refuse, not report all hydrocarbon; got {}",
            ws(20.0)
        );

        // B - Rt = 0.5: an ordinary WET zone. Every saturation model clamps here and so must this
        // one - refusing n = 1 wholesale would blank it.
        assert!(
            (ws(0.5) - 1.0).abs() < 1e-9,
            "a wet zone at n = 1 must still read Swt = 1, got {}",
            ws(0.5)
        );

        // C - Rt = 10: rhs = 2.5, so Swt = (2.5 - 2.0)/1.0 = 0.5 exactly. The ordinary interior
        // answer at n = 1 must be untouched.
        assert!(
            (ws(10.0) - 0.5).abs() < 1e-9,
            "the ordinary root at n = 1 must be unchanged, got {}",
            ws(10.0)
        );
    }
}
