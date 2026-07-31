//! Rock typing (Wave B item 8, first increment): per-sample hydraulic/pore-geometry indicators
//! and a rock-type class from φ and k, grounded in `docs/research_2026-07/ref_rocktyping_shf.md`.
//!
//! Methods implemented here (deterministic, no clustering — GHE uses FIXED FZI bins, port classes
//! use fixed pore-throat cutoffs, so results are reproducible and unit-testable):
//!   - Amaefule 1993 RQI / φz / FZI + Corbett-Potter 2004 GHE bins, with the per-HFU
//!     geometric-mean-FZI permeability predictor k = 1014.24·FZI²·φ³/(1−φ)².
//!   - Kolodzie 1980 Winland R35 (+ port classes mega/macro/meso/micro/nano).
//!   - Permadi-Susilo PGS pore geometry √(k/φ) and pore structure k/φ^PS_EXP, PS_EXP default 3.0.
//!
//! Also here (increment 2): Lucia RFN (carbonate), cutoff-based electrofacies RT, and the Pittman
//! (1992) full r10–r75 pore-throat table with a selectable apex → port class. Interactive
//! Ward/histogram HFU clustering lives in `hfu.rs` (cross-well pane, reads core φ-k).
//! Still deferred (see the reference doc): MICP-fitted LOCAL Winland/Pittman coefficients.
//!
//! NOTE: VERIFIED 2026-07-22 (see docs/constants_verification_2026-07-22.md). GHE bins were
//! corrected to the Corbett-Potter 2004 ×2 series (…1.5, 3, 6, 12, 24), and PGS to √(k/φ) / k/φ³
//! (exponent 3, per the Kozeny-Carman derivation and the ACS Omega 2024 Permadi-Susilo review;
//! the old k/φ + 3.5 were unverified reference-doc recall). PS_EXP stays a param for override;
//! confirm PGS against the primary SPE 125350 if a copy becomes available.

use crate::modules::{log_in, log_out, opt, param, ModuleContext, ModuleOutputs, ModuleSpec};
use std::collections::HashMap;

/// Corbett & Potter (2004) global hydraulic element FZI boundaries (µm). A sample's GHE class is
/// 1 + (number of boundaries its FZI exceeds), so FZI < 0.0938 → GHE1 … FZI ≥ 24 → GHE10.
/// Corbett-Potter 2004 ×2 geometric series (corrected 2026-07-22; was …1.5, 2.5, 4, 6, 8).
const GHE_BOUNDS: [f64; 9] = [0.0938, 0.1875, 0.375, 0.75, 1.5, 3.0, 6.0, 12.0, 24.0];

/// Winland-R35 port-class pore-throat cutoffs (µm): nano <0.1, micro 0.1–0.5, meso 0.5–2.5,
/// macro 2.5–10, mega ≥10 → classes 1..5.
const PORT_BOUNDS: [f64; 4] = [0.1, 0.5, 2.5, 10.0];

fn ghe_class(fzi: f64) -> f64 {
    (1 + GHE_BOUNDS.iter().filter(|&&b| fzi >= b).count()) as f64
}

fn port_class(r35: f64) -> f64 {
    (1 + PORT_BOUNDS.iter().filter(|&&b| r35 >= b).count()) as f64
}

pub fn rocktyping_spec() -> ModuleSpec {
    ModuleSpec {
        name: "rocktyping".into(),
        title: "Rock Typing (FZI / R35 / PGS)".into(),
        category: "Rock Typing".into(),
        doc: "Per-sample rock-typing indicators from porosity and permeability. Writes \
              RQI = 0.0314·√(k/φ), PHIZ = φ/(1−φ), FZI = RQI/PHIZ (Amaefule 1993); Winland \
              R35 = 10^(0.732 + 0.588·log10 k − 0.864·log10 φ%) (Kolodzie 1980); and the \
              Permadi-Susilo PGS pair PGEOM = √(k/φ), PSTRUC = k/φ^PS_EXP. RT is the rock-type \
              class from the chosen METHOD — GHE fixed FZI bins (Corbett-Potter 2004) or Winland \
              port classes (nano..mega). PERM_RT is the class-grouped permeability estimate \
              k = 1014.24·FZI_mean(RT)²·φ³/(1−φ)² using each class's GEOMETRIC-MEAN FZI over this \
              well. k in mD, φ in v/v; samples with φ∉(0,1) or k≤0 stay MISSING. GHE bins follow \
              the Corbett-Potter 2004 ×2 series and PGS uses √(k/φ) / k/φ³ (verified 2026-07-22)."
            .into(),
        args: vec![
            opt("METHOD", "Rock-type class basis", "ghe", &["ghe", "winland_port"]),
            param("PS_EXP", "PGS pore-structure exponent (k/φ^PS_EXP)", "-", 3.0, 1.0, 6.0),
            log_in("PHI", "Effective porosity", "v/v", "PHIE", true),
            log_in("PERM", "Permeability", "mD", "PERM", true),
            log_out("RQI", "Reservoir quality index 0.0314·√(k/φ)", "um"),
            log_out("PHIZ", "Normalized porosity φ/(1−φ)", "-"),
            log_out("FZI", "Flow zone indicator RQI/PHIZ", "um"),
            log_out("R35", "Winland R35 pore-throat radius", "um"),
            log_out("PGEOM", "PGS pore geometry √(k/φ)", "-"),
            log_out("PSTRUC", "PGS pore structure k/φ^PS_EXP", "-"),
            log_out("RT", "Rock-type class (GHE 1..10 or port 1..5)", "-"),
            log_out("PERM_RT", "Class-grouped permeability estimate", "mD"),
        ],
    }
}

pub fn rocktyping(ctx: &ModuleContext) -> ModuleOutputs {
    let phi_log = ctx.log("PHI");
    let perm_log = ctx.log("PERM");
    let method = ctx.o("METHOD").to_string();
    let ps_exp_raw = ctx.p("PS_EXP", 0);
    let ps_exp = if ps_exp_raw.is_finite() { ps_exp_raw.clamp(1.0, 6.0) } else { 3.0 };

    let n = ctx.n;
    let mut rqi = vec![f32::NAN; n];
    let mut phiz = vec![f32::NAN; n];
    let mut fzi = vec![f32::NAN; n];
    let mut r35 = vec![f32::NAN; n];
    let mut pgeom = vec![f32::NAN; n];
    let mut pstruc = vec![f32::NAN; n];
    let mut rt = vec![f32::NAN; n];
    let mut perm_rt = vec![f32::NAN; n];

    // Pass 1: per-sample indicators + rock-type class.
    for i in 0..n {
        let phi = phi_log[i] as f64;
        let k = perm_log[i] as f64;
        if !(phi.is_finite() && k.is_finite()) || phi <= 0.0 || phi >= 1.0 || k <= 0.0 {
            continue; // leave MISSING
        }
        let rqi_i = 0.0314 * (k / phi).sqrt();
        let phiz_i = phi / (1.0 - phi);
        let fzi_i = if phiz_i > 0.0 { rqi_i / phiz_i } else { f64::NAN };
        let r35_i = 10f64.powf(0.732 + 0.588 * k.log10() - 0.864 * (phi * 100.0).log10());
        rqi[i] = rqi_i as f32;
        phiz[i] = phiz_i as f32;
        fzi[i] = fzi_i as f32;
        r35[i] = r35_i as f32;
        pgeom[i] = (k / phi).sqrt() as f32;
        pstruc[i] = (k / phi.powf(ps_exp)) as f32;
        if fzi_i.is_finite() && fzi_i > 0.0 {
            rt[i] = if method == "winland_port" { port_class(r35_i) } else { ghe_class(fzi_i) } as f32;
        }
    }

    // Pass 2: geometric-mean FZI per rock-type class (Σ ln FZI / count, exponentiated).
    let mut ln_sum: HashMap<i64, (f64, usize)> = HashMap::new();
    for i in 0..n {
        if rt[i].is_finite() && fzi[i].is_finite() && (fzi[i] as f64) > 0.0 {
            let e = ln_sum.entry(rt[i] as i64).or_insert((0.0, 0));
            e.0 += (fzi[i] as f64).ln();
            e.1 += 1;
        }
    }
    let fzi_mean: HashMap<i64, f64> =
        ln_sum.iter().map(|(&c, &(s, cnt))| (c, (s / cnt.max(1) as f64).exp())).collect();

    // Pass 3: class-grouped permeability estimate k = 1014.24·FZI_mean²·φ³/(1−φ)².
    for i in 0..n {
        let phi = phi_log[i] as f64;
        if !rt[i].is_finite() || !phi.is_finite() || phi <= 0.0 || phi >= 1.0 {
            continue;
        }
        if let Some(&fm) = fzi_mean.get(&(rt[i] as i64)) {
            let k = 1014.24 * fm * fm * phi.powi(3) / (1.0 - phi).powi(2);
            perm_rt[i] = k as f32;
        }
    }

    HashMap::from([
        ("RQI".into(), rqi),
        ("PHIZ".into(), phiz),
        ("FZI".into(), fzi),
        ("R35".into(), r35),
        ("PGEOM".into(), pgeom),
        ("PSTRUC".into(), pstruc),
        ("RT".into(), rt),
        ("PERM_RT".into(), perm_rt),
    ])
}

// --------------------------------------------------------------------------------------------
// Lucia Rock-Fabric Number (Wave B item 8, increment 2) — carbonate rock typing (Lucia 1995;
// Jennings & Lucia 2003, SPE 78740). Global transform log10 k = (A − B·log10 RFN) +
// (C − D·log10 RFN)·log10 φip. It is LINEAR in r = log10 RFN, so RFN inverts analytically:
//   r = (A + C·log10 φip − log10 k) / (B + D·log10 φip).
// Classes: RFN 0.5–1.5 = Class 1 (grainstone), 1.5–2.5 = Class 2, 2.5–4 = Class 3 (mud-dominated).
// Mahakam is clastic-dominated, so this is secondary (carbonate stringers). CONSTANTS from the
// paper via ref_rocktyping_shf.md — VERIFY before field release.
// --------------------------------------------------------------------------------------------

const LUCIA_A: f64 = 9.7982;
const LUCIA_B: f64 = 12.0838;
const LUCIA_C: f64 = 8.6711;
const LUCIA_D: f64 = 8.2965;

/// Rock-fabric number from interparticle porosity (frac) and permeability (mD); NaN if the
/// transform is ill-conditioned (denominator ~0) or inputs are out of range.
fn lucia_rfn(phi_ip: f64, k: f64) -> f64 {
    if !(phi_ip.is_finite() && k.is_finite()) || phi_ip <= 0.0 || phi_ip >= 1.0 || k <= 0.0 {
        return f64::NAN;
    }
    let lphi = phi_ip.log10();
    let denom = LUCIA_B + LUCIA_D * lphi;
    if denom.abs() < 1e-6 {
        return f64::NAN;
    }
    let r = (LUCIA_A + LUCIA_C * lphi - k.log10()) / denom;
    10f64.powf(r)
}

fn lucia_class(rfn: f64) -> f64 {
    if !rfn.is_finite() {
        f64::NAN
    } else if rfn < 1.5 {
        1.0
    } else if rfn < 2.5 {
        2.0
    } else if rfn <= 4.0 {
        3.0
    } else {
        f64::NAN // outside the calibrated 0.5–4 band
    }
}

pub fn lucia_rfn_spec() -> ModuleSpec {
    ModuleSpec {
        name: "lucia_rfn".into(),
        title: "Lucia Rock-Fabric Number (carbonate)".into(),
        category: "Rock Typing".into(),
        doc: "Carbonate rock typing by Lucia rock-fabric number (Jennings & Lucia 2003). Inverts \
              the global transform log10 k = (A − B·log10 RFN) + (C − D·log10 RFN)·log10 φip \
              analytically for RFN, then bins: RFN 0.5–1.5 = Class 1 (grainstone), 1.5–2.5 = \
              Class 2, 2.5–4 = Class 3 (mud-dominated). PHI should be INTERPARTICLE porosity \
              (subtract vuggy/separate-vug porosity if available); k in mD. Writes RFN and \
              RT_LUCIA (1–3; MISSING outside the 0.5–4 band). Clastic-dominated fields use this \
              only for carbonate stringers. Constants transcribed from the paper — verify first."
            .into(),
        args: vec![
            log_in("PHI", "Interparticle porosity", "v/v", "PHIE", true),
            log_in("PERM", "Permeability", "mD", "PERM", true),
            log_out("RFN", "Lucia rock-fabric number", "-"),
            log_out("RT_LUCIA", "Lucia class (1–3)", "-"),
        ],
    }
}

pub fn lucia_rfn_module(ctx: &ModuleContext) -> ModuleOutputs {
    let phi = ctx.log("PHI");
    let perm = ctx.log("PERM");
    let mut rfn = vec![f32::NAN; ctx.n];
    let mut rt = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let v = lucia_rfn(phi[i] as f64, perm[i] as f64);
        rfn[i] = v as f32;
        rt[i] = lucia_class(v) as f32;
    }
    HashMap::from([("RFN".into(), rfn), ("RT_LUCIA".into(), rt)])
}

// --------------------------------------------------------------------------------------------
// Cutoff-based electrofacies / rock-type classification (Wave B item 8, increment 2) — the
// log-domain half of the electrofacies tie-in (ref_rocktyping_shf.md §Cutoff-based electrofacies
// tie-in): assign a 3-class rock-type ladder from Vsh + PHIE cutoffs so core rock types can be
// propagated to uncored intervals. RT 1 = best (clean, porous), 2 = moderate, 3 = non-net.
// --------------------------------------------------------------------------------------------

pub fn rt_cutoff_spec() -> ModuleSpec {
    ModuleSpec {
        name: "rt_cutoff".into(),
        title: "Rock Type from Cutoffs (electrofacies)".into(),
        category: "Rock Typing".into(),
        doc: "Log-domain rock-type class from a Vsh + PHIE cutoff ladder — the electrofacies half \
              of the rock-typing tie-in. RT_LOG = 1 (best: Vsh ≤ VSH1 and PHIE ≥ PHI1), 2 (moderate: \
              Vsh ≤ VSH2 and PHIE ≥ PHI2), else 3 (non-net). Requires VSH1 ≤ VSH2 and PHI1 ≥ PHI2. \
              Feed the result to the confusion-matrix QC (Rock Typing ▸ Facies Tie-in) to validate \
              it against a core-derived RT curve, then attach per-class phi-k / SHF laws. Samples \
              with missing Vsh or PHIE stay MISSING."
            .into(),
        args: vec![
            param("VSH1", "Vsh cutoff for RT1 (best)", "v/v", 0.15, 0.0, 1.0),
            param("PHI1", "PHIE cutoff for RT1 (best)", "v/v", 0.12, 0.0, 1.0),
            param("VSH2", "Vsh cutoff for RT2 (moderate)", "v/v", 0.35, 0.0, 1.0),
            param("PHI2", "PHIE cutoff for RT2 (moderate)", "v/v", 0.06, 0.0, 1.0),
            log_in("VSH", "Shale volume", "v/v", "VSH", true),
            log_in("PHIE", "Effective porosity", "v/v", "PHIE", true),
            log_out("RT_LOG", "Cutoff rock-type class (1/2/3)", "-"),
        ],
    }
}

pub fn rt_cutoff(ctx: &ModuleContext) -> ModuleOutputs {
    let vsh = ctx.log("VSH");
    let phie = ctx.log("PHIE");
    let mut rt = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let v = vsh[i] as f64;
        let p = phie[i] as f64;
        if !(v.is_finite() && p.is_finite()) {
            continue;
        }
        let (vsh1, phi1, vsh2, phi2) = (ctx.p("VSH1", i), ctx.p("PHI1", i), ctx.p("VSH2", i), ctx.p("PHI2", i));
        rt[i] = if v <= vsh1 && p >= phi1 {
            1.0
        } else if v <= vsh2 && p >= phi2 {
            2.0
        } else {
            3.0
        };
    }
    HashMap::from([("RT_LOG".into(), rt)])
}

// --------------------------------------------------------------------------------------------
// Pittman full pore-throat aperture table (Wave B item 8, increment 2) — Pittman (1992, AAPG
// Bulletin v76) regressed the pore-throat radius at mercury saturations 10..75 % against φ and k
// for a sandstone set. Each rX is log10 rX = C0 + C1·log10 k + C2·log10 φ, k in mD, φ in PERCENT,
// rX in µm. The r35 row (0.255, 0.565, −0.523) matches ref_rocktyping_shf.md, which anchors the
// transcription; the FULL nine-row set is from the paper's table and is flagged verify-before-
// release (same policy as PGS 3.5 / Swanson / Lucia). Pittman's "apex" — the rX that best predicts
// k for a given rock family (coarse rocks apex near r25–r35, finer near r50–r75) — is selectable.
// --------------------------------------------------------------------------------------------

/// (mnemonic, C0, C1, C2) for each mercury-saturation radius; ordered by saturation 10..75 %.
const PITTMAN_RX: [(&str, f64, f64, f64); 9] = [
    ("PR10", 0.459, 0.500, -0.385),
    ("PR15", 0.333, 0.509, -0.344),
    ("PR20", 0.218, 0.519, -0.303),
    ("PR25", 0.204, 0.531, -0.350),
    ("PR30", 0.215, 0.547, -0.420),
    ("PR35", 0.255, 0.565, -0.523),
    ("PR40", 0.360, 0.582, -0.680),
    ("PR50", 0.609, 0.608, -0.974),
    ("PR75", 1.243, 0.674, -1.517),
];

/// Mercury saturation (%) of each PITTMAN_RX row, for the APEX selector ("r35" → 35).
const PITTMAN_PCT: [u32; 9] = [10, 15, 20, 25, 30, 35, 40, 50, 75];

/// Pore-throat radius (µm) for one Pittman row from k (mD) and φ (PERCENT).
fn pittman_radius(coef: (f64, f64, f64), k: f64, phi_pct: f64) -> f64 {
    let (c0, c1, c2) = coef;
    10f64.powf(c0 + c1 * k.log10() + c2 * phi_pct.log10())
}

/// Index into PITTMAN_RX for an APEX option like "r35" (default r35); parses the trailing digits.
fn pittman_apex_idx(apex: &str) -> usize {
    let pct: u32 = apex.trim_start_matches(['r', 'R']).parse().unwrap_or(35);
    PITTMAN_PCT.iter().position(|&p| p == pct).unwrap_or(5)
}

pub fn pittman_rx_spec() -> ModuleSpec {
    ModuleSpec {
        name: "pittman_rx".into(),
        title: "Pittman Pore-Throat Radii (r10–r75)".into(),
        category: "Rock Typing".into(),
        doc: "Pittman (1992) pore-throat aperture family: writes PR10..PR75 = pore-throat radius \
              (µm) at mercury saturation 10..75 %, each log10 rX = C0 + C1·log10 k + C2·log10 φ% \
              (k mD, φ in PERCENT). RAPEX is the radius at the chosen APEX saturation and RT_PITT \
              its Hartmann-Beaumont port class (nano<0.1, micro 0.1–0.5, meso 0.5–2.5, macro \
              2.5–10, mega ≥10 µm → 1..5). Pick APEX = the rX that best correlates with k for your \
              rock family (coarse rocks apex near r25–r35, finer near r50–r75); r35 is the common \
              default and matches the Winland concept. Samples with φ∉(0,1) or k≤0 stay MISSING. \
              NOTE: the coefficient table is transcribed from Pittman 1992 (r35 cross-checks the \
              reference doc) — verify the full set against the paper before field release."
            .into(),
        args: vec![
            opt(
                "APEX",
                "Controlling mercury-saturation radius for RT (port class)",
                "r35",
                &["r10", "r15", "r20", "r25", "r30", "r35", "r40", "r50", "r75"],
            ),
            log_in("PHI", "Effective porosity", "v/v", "PHIE", true),
            log_in("PERM", "Permeability", "mD", "PERM", true),
            log_out("PR10", "Pittman pore-throat radius at 10 % Hg", "um"),
            log_out("PR15", "Pittman pore-throat radius at 15 % Hg", "um"),
            log_out("PR20", "Pittman pore-throat radius at 20 % Hg", "um"),
            log_out("PR25", "Pittman pore-throat radius at 25 % Hg", "um"),
            log_out("PR30", "Pittman pore-throat radius at 30 % Hg", "um"),
            log_out("PR35", "Pittman pore-throat radius at 35 % Hg", "um"),
            log_out("PR40", "Pittman pore-throat radius at 40 % Hg", "um"),
            log_out("PR50", "Pittman pore-throat radius at 50 % Hg", "um"),
            log_out("PR75", "Pittman pore-throat radius at 75 % Hg", "um"),
            log_out("RAPEX", "Radius at the chosen APEX saturation", "um"),
            log_out("RT_PITT", "Port class of RAPEX (1..5)", "-"),
        ],
    }
}

pub fn pittman_rx(ctx: &ModuleContext) -> ModuleOutputs {
    let phi_log = ctx.log("PHI");
    let perm_log = ctx.log("PERM");
    let apex_idx = pittman_apex_idx(ctx.o("APEX"));

    let n = ctx.n;
    let mut radii: Vec<Vec<f32>> = (0..9).map(|_| vec![f32::NAN; n]).collect();
    let mut rapex = vec![f32::NAN; n];
    let mut rt = vec![f32::NAN; n];

    for i in 0..n {
        let phi = phi_log[i] as f64;
        let k = perm_log[i] as f64;
        if !(phi.is_finite() && k.is_finite()) || phi <= 0.0 || phi >= 1.0 || k <= 0.0 {
            continue; // leave MISSING
        }
        let phi_pct = phi * 100.0;
        for (j, &(_, c0, c1, c2)) in PITTMAN_RX.iter().enumerate() {
            radii[j][i] = pittman_radius((c0, c1, c2), k, phi_pct) as f32;
        }
        let ap = radii[apex_idx][i] as f64;
        rapex[i] = ap as f32;
        if ap.is_finite() && ap > 0.0 {
            rt[i] = port_class(ap) as f32;
        }
    }

    let mut out: ModuleOutputs = HashMap::new();
    for (j, &(name, ..)) in PITTMAN_RX.iter().enumerate() {
        out.insert(name.into(), std::mem::take(&mut radii[j]));
    }
    out.insert("RAPEX".into(), rapex);
    out.insert("RT_PITT".into(), rt);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a minimal ModuleContext with PHI + PERM logs and the given options/params.
    fn ctx(phi: Vec<f32>, perm: Vec<f32>, method: &str, ps_exp: f64) -> ModuleContext {
        let n = phi.len();
        let mut logs = HashMap::new();
        logs.insert("PHI".to_string(), phi);
        logs.insert("PERM".to_string(), perm);
        let mut opts = HashMap::new();
        opts.insert("METHOD".to_string(), method.to_string());
        let mut params = HashMap::new();
        params.insert("PS_EXP".to_string(), vec![ps_exp]);
        ModuleContext { n, logs, params, opts, depth_unit: Default::default() }
    }

    /// Builds a ModuleContext for `rt_cutoff` from VSH/PHIE and the four ladder cutoffs.
    fn cutoff_ctx(vsh: Vec<f32>, phie: Vec<f32>, vsh1: f64, phi1: f64, vsh2: f64, phi2: f64) -> ModuleContext {
        let n = vsh.len();
        let logs = HashMap::from([("VSH".to_string(), vsh), ("PHIE".to_string(), phie)]);
        let params = HashMap::from([
            ("VSH1".to_string(), vec![vsh1; n]),
            ("PHI1".to_string(), vec![phi1; n]),
            ("VSH2".to_string(), vec![vsh2; n]),
            ("PHI2".to_string(), vec![phi2; n]),
        ]);
        ModuleContext { n, logs, params, opts: HashMap::new(), depth_unit: Default::default() }
    }

    /// T-RT-07 — the RT_LOG ladder on sane cutoffs, and what an inverted one actually does.
    ///
    /// The module doc requires VSH1 ≤ VSH2 and PHI1 ≥ PHI2, and nothing enforces it: the dialog
    /// range-checks each field against 0–1 independently, so `VSH1 = 0.50, VSH2 = 0.20` runs.
    /// The plan's step 4 asks what happens. It is worse than "no warning" — the ladder does not
    /// merely shift, it SCATTERS. Because class 1 is tested first and its Vsh gate is now the
    /// looser one, moderately shaly rock splits: the porous half is promoted to BEST and the
    /// tight half is demoted to non-net, with class 2 left meaning something else entirely.
    ///
    /// Pinned AS-IS, not endorsed — a cross-field validation is a UI decision, and RT_LOG feeds
    /// the facies tie-in, so silently repairing the ladder would change published class counts.
    #[test]
    fn an_inverted_cutoff_ladder_is_accepted_and_scatters_the_middle_class() {
        // best | moderate+porous | moderate+tight | non-net | missing Vsh | missing PHIE
        let vsh = vec![0.10f32, 0.30, 0.30, 0.60, f32::NAN, 0.10];
        let phie = vec![0.20f32, 0.20, 0.08, 0.03, 0.20, f32::NAN];

        // Sane defaults from the manifest: VSH1 0.15, PHI1 0.12, VSH2 0.35, PHI2 0.06.
        let sane = rt_cutoff(&cutoff_ctx(vsh.clone(), phie.clone(), 0.15, 0.12, 0.35, 0.06))["RT_LOG"].clone();
        assert_eq!(sane[0], 1.0, "clean and porous is the best class");
        assert_eq!(sane[1], 2.0, "moderately shaly but porous is the middle class");
        assert_eq!(sane[2], 2.0, "moderately shaly and tighter is still the middle class");
        assert_eq!(sane[3], 3.0, "shaly and tight is non-net");
        assert!(sane[4].is_nan() && sane[5].is_nan(), "a missing input stays MISSING, never class 3");

        // Inverted, exactly as step 4 asks: VSH1 0.50 > VSH2 0.20.
        let bad = rt_cutoff(&cutoff_ctx(vsh, phie, 0.50, 0.12, 0.20, 0.06))["RT_LOG"].clone();
        assert_eq!(bad[0], 1.0, "the genuinely best rock is unaffected");
        assert_eq!(bad[1], 1.0, "middle rock is PROMOTED to best — the damaging direction");
        assert_eq!(bad[2], 3.0, "and its tighter half is DEMOTED to non-net in the same run");
        assert_eq!(bad[3], 3.0);
        assert!(bad[4].is_nan() && bad[5].is_nan(), "MISSING is unaffected by the cutoffs");

        // Stated as the reader would see it: one sample moved two classes without any warning.
        assert_ne!(sane[1], bad[1]);
        assert_ne!(sane[2], bad[2]);
    }

    /// Builds a ModuleContext for `pittman_rx` from PHI/PERM and the APEX selector.
    fn pittman_ctx(phi: Vec<f32>, perm: Vec<f32>, apex: &str) -> ModuleContext {
        let n = phi.len();
        let logs = HashMap::from([("PHI".to_string(), phi), ("PERM".to_string(), perm)]);
        let opts = HashMap::from([("APEX".to_string(), apex.to_string())]);
        ModuleContext { n, logs, params: HashMap::new(), opts, depth_unit: Default::default() }
    }

    /// T-RT-08 — the Pittman family, the APEX selector, and the ordering claim that does NOT hold.
    ///
    /// The physics is not in doubt: mercury enters the widest throats first, so the radius quoted
    /// at a higher saturation must be the smaller one, and r75 < r50 always. The plan's Expected
    /// says the monotone ordering "holds everywhere both curves are populated".
    ///
    /// It does not. The nine rows are nine INDEPENDENT regressions, not a nested family, so
    /// nothing in the arithmetic makes them agree. PR50 − PR75 in log space is
    /// `−0.634 − 0.066·log k + 0.543·log φ%`, which changes sign at about **79 mD at 25 % porosity**
    /// — ordinary reservoir sand, not a corner. Above that the transcribed table reports a LARGER
    /// throat at 75 % mercury than at 50 %, which cannot happen in rock.
    ///
    /// The module's own doc already flags the full set as transcribed and "verify before field
    /// release". This is that verification, and the r50/r75 pair fails it. Pinned AS-IS because
    /// correcting a coefficient needs the 1992 paper in hand, not a guess — see
    /// docs/review_triage.md finding 9.
    #[test]
    fn the_pittman_radius_family_inverts_between_r50_and_r75_in_good_sand() {
        // Good sand: φ = 25 %, k = 100 mD. Plus the two invalid shapes the doc promises to skip.
        let phi = vec![0.25f32, 0.25, 1.0, 0.25];
        let perm = vec![100.0f32, 1.0, 100.0, -5.0];
        let out = pittman_rx(&pittman_ctx(phi.clone(), perm.clone(), "r35"));

        // All eleven outputs exist.
        for name in ["PR10", "PR15", "PR20", "PR25", "PR30", "PR35", "PR40", "PR50", "PR75", "RAPEX", "RT_PITT"] {
            assert!(out.contains_key(name), "missing output {name}");
        }

        // The head of the family IS monotone in good sand, which is what makes the tail's failure
        // a transcription question rather than a wholesale problem with the method.
        let at = |name: &str| out[name][0] as f64;
        let head = ["PR10", "PR15", "PR20", "PR25", "PR30", "PR35", "PR40", "PR50"];
        for pair in head.windows(2) {
            assert!(
                at(pair[0]) > at(pair[1]),
                "{} ({}) must exceed {} ({})",
                pair[0],
                at(pair[0]),
                pair[1],
                at(pair[1])
            );
        }

        // The tail does not. Pinned with the measured numbers so it cannot drift unnoticed.
        assert!(
            at("PR75") > at("PR50"),
            "the inversion this test exists to record has gone — re-read finding 9 before deleting it"
        );
        assert!((at("PR50") - 2.907).abs() < 0.01, "PR50 {}", at("PR50"));
        assert!((at("PR75") - 2.953).abs() < 0.01, "PR75 {}", at("PR75"));

        // At 1 mD the same pair is the right way round, which locates the problem in the
        // coefficients rather than in any one sample.
        let low_k = |name: &str| out[name][1] as f64;
        assert!(low_k("PR50") > low_k("PR75"), "at 1 mD the ordering holds: {} vs {}", low_k("PR50"), low_k("PR75"));

        // RAPEX follows the selector exactly, and RT_PITT is its port class.
        assert_eq!(out["RAPEX"][0], out["PR35"][0], "default APEX r35 must BE PR35, not merely near it");
        let r50 = pittman_rx(&pittman_ctx(phi.clone(), perm.clone(), "r50"));
        assert_eq!(r50["RAPEX"][0], r50["PR50"][0], "APEX r50 must track PR50");
        assert_ne!(out["RAPEX"][0], r50["RAPEX"][0], "and the selector must actually change the answer");
        for v in [out["RT_PITT"][0], r50["RT_PITT"][0]] {
            assert!((1.0..=5.0).contains(&v) && v.fract() == 0.0, "RT_PITT must be an integer 1..5, was {v}");
        }

        // An invalid sample is blank in ALL eleven — a half-written row would let one bad depth
        // poison a catalog Min/Max while the rest of the family looked fine.
        for name in ["PR10", "PR15", "PR20", "PR25", "PR30", "PR35", "PR40", "PR50", "PR75", "RAPEX", "RT_PITT"] {
            assert!(out[name][2].is_nan(), "{name} must be MISSING at phi = 1.0");
            assert!(out[name][3].is_nan(), "{name} must be MISSING at k <= 0");
        }
    }

    #[test]
    fn fzi_and_rqi_match_amaefule_formula() {
        // φ=0.20, k=100 mD → RQI=0.0314·√500=0.702..; φz=0.25; FZI=RQI/φz.
        let c = ctx(vec![0.20], vec![100.0], "ghe", 3.5);
        let out = rocktyping(&c);
        let rqi = out["RQI"][0] as f64;
        let fzi = out["FZI"][0] as f64;
        assert!((rqi - 0.0314 * 500f64.sqrt()).abs() < 1e-4, "rqi={rqi}");
        assert!((fzi - rqi / 0.25).abs() < 1e-4, "fzi={fzi}");
        // FZI ≈ 2.808 → between GHE bounds 1.5 and 3 (Corbett-Potter ×2 series) → GHE class 6.
        assert_eq!(out["RT"][0], 6.0, "fzi={fzi}");
    }

    #[test]
    fn winland_r35_and_port_class() {
        // Kolodzie: log10 R35 = 0.732 + 0.588·log10(100) − 0.864·log10(20).
        let c = ctx(vec![0.20], vec![100.0], "winland_port", 3.5);
        let out = rocktyping(&c);
        let expect = 10f64.powf(0.732 + 0.588 * 100f64.log10() - 0.864 * 20f64.log10());
        assert!((out["R35"][0] as f64 - expect).abs() < 1e-3, "r35={}", out["R35"][0]);
        // R35 ≈ 6.5 µm → macro (2.5–10) → port class 4.
        assert_eq!(out["RT"][0], 4.0);
    }

    #[test]
    fn perm_rt_recovers_input_for_single_class() {
        // Two samples on the SAME FZI (same k/φ ratio structure) → one GHE class; the
        // class-grouped predictor must reproduce each sample's own permeability.
        let c = ctx(vec![0.20, 0.25], vec![100.0, 216.0], "ghe", 3.5);
        let out = rocktyping(&c);
        // Both fall in the same GHE bin only if FZI matches; assert PERM_RT is finite and
        // positive and within a factor of the class-mean prediction.
        for i in 0..2 {
            assert!(out["PERM_RT"][i].is_finite() && out["PERM_RT"][i] > 0.0);
        }
    }

    #[test]
    fn missing_and_out_of_range_inputs_stay_missing() {
        let c = ctx(vec![f32::NAN, 0.0, 1.0, 0.2], vec![10.0, 10.0, 10.0, -5.0], "ghe", 3.5);
        let out = rocktyping(&c);
        for i in 0..4 {
            assert!(!out["FZI"][i].is_finite(), "sample {i} should be MISSING");
            assert!(!out["RT"][i].is_finite());
        }
    }

    fn rel(a: f64, b: f64) -> f64 {
        (a - b).abs() / b.abs().max(1e-12)
    }

    #[test]
    fn lucia_rfn_round_trips_and_classes() {
        // Forward-compute k from a known RFN via the same transform, then invert: RFN must return.
        let fwd = |rfn: f64, phi: f64| {
            let r = rfn.log10();
            let lk = (LUCIA_A - LUCIA_B * r) + (LUCIA_C - LUCIA_D * r) * phi.log10();
            10f64.powf(lk)
        };
        let k1 = fwd(1.0, 0.20);
        assert!(rel(lucia_rfn(0.20, k1), 1.0) < 1e-4, "got {}", lucia_rfn(0.20, k1));
        assert_eq!(lucia_class(lucia_rfn(0.20, k1)), 1.0);
        let k3 = fwd(3.0, 0.15);
        assert!(rel(lucia_rfn(0.15, k3), 3.0) < 1e-4, "got {}", lucia_rfn(0.15, k3));
        assert_eq!(lucia_class(lucia_rfn(0.15, k3)), 3.0);
        // Out-of-range inputs → MISSING.
        assert!(!lucia_rfn(0.0, 10.0).is_finite());
        assert!(!lucia_rfn(0.2, -1.0).is_finite());
    }

    /// Minimal ModuleContext with PHI + PERM and an APEX option, for the Pittman module.
    fn pitt_ctx(phi: Vec<f32>, perm: Vec<f32>, apex: &str) -> ModuleContext {
        let n = phi.len();
        let mut logs = HashMap::new();
        logs.insert("PHI".to_string(), phi);
        logs.insert("PERM".to_string(), perm);
        let mut opts = HashMap::new();
        opts.insert("APEX".to_string(), apex.to_string());
        ModuleContext { n, logs, params: HashMap::new(), opts, depth_unit: Default::default() }
    }

    #[test]
    fn pittman_r35_matches_published_regression() {
        // log10 r35 = 0.255 + 0.565·log10 k − 0.523·log10 φ%  (Pittman 1992; anchors the table).
        let (phi, k) = (0.20f64, 100.0f64);
        let out = pittman_rx(&pitt_ctx(vec![phi as f32], vec![k as f32], "r35"));
        let expect = 10f64.powf(0.255 + 0.565 * k.log10() - 0.523 * (phi * 100.0).log10());
        assert!((out["PR35"][0] as f64 - expect).abs() < 1e-3, "pr35={}", out["PR35"][0]);
        // RAPEX at r35 must equal PR35, and RT_PITT is its port class.
        assert!((out["RAPEX"][0] - out["PR35"][0]).abs() < 1e-4);
        assert_eq!(out["RT_PITT"][0], port_class(expect) as f32);
    }

    #[test]
    fn pittman_apex_selector_switches_controlling_radius() {
        // Same rock, different APEX → RAPEX tracks the chosen row (r25 vs r50 differ here).
        let (phi, k) = (vec![0.18f32], vec![50.0f32]);
        let r25 = pittman_rx(&pitt_ctx(phi.clone(), k.clone(), "r25"));
        let r50 = pittman_rx(&pitt_ctx(phi.clone(), k.clone(), "r50"));
        assert!((r25["RAPEX"][0] - r25["PR25"][0]).abs() < 1e-4);
        assert!((r50["RAPEX"][0] - r50["PR50"][0]).abs() < 1e-4);
        assert_ne!(r25["RAPEX"][0], r50["RAPEX"][0], "r25 and r50 should differ");
        // All nine radii are emitted and finite for a valid plug.
        for (name, ..) in PITTMAN_RX {
            assert!(r25[name][0].is_finite(), "{name} missing");
        }
    }

    #[test]
    fn pittman_missing_inputs_stay_missing() {
        let out = pittman_rx(&pitt_ctx(
            vec![f32::NAN, 0.0, 1.0, 0.2],
            vec![10.0, 10.0, 10.0, -5.0],
            "r35",
        ));
        for i in 0..4 {
            assert!(!out["PR35"][i].is_finite(), "sample {i} should be MISSING");
            assert!(!out["RAPEX"][i].is_finite());
            assert!(!out["RT_PITT"][i].is_finite());
        }
    }

    #[test]
    fn rt_cutoff_ladders_by_vsh_and_phie() {
        let n = 4;
        let mut logs = HashMap::new();
        logs.insert("VSH".to_string(), vec![0.10f32, 0.30, 0.60, f32::NAN]);
        logs.insert("PHIE".to_string(), vec![0.20f32, 0.10, 0.03, 0.20]);
        let mut params = HashMap::new();
        params.insert("VSH1".to_string(), vec![0.15; n]);
        params.insert("PHI1".to_string(), vec![0.12; n]);
        params.insert("VSH2".to_string(), vec![0.35; n]);
        params.insert("PHI2".to_string(), vec![0.06; n]);
        let c = ModuleContext { n, logs, params, opts: HashMap::new(), depth_unit: Default::default() };
        let out = rt_cutoff(&c);
        assert_eq!(out["RT_LOG"][0], 1.0); // clean + porous → RT1
        assert_eq!(out["RT_LOG"][1], 2.0); // moderate → RT2
        assert_eq!(out["RT_LOG"][2], 3.0); // shaly + tight → non-net RT3
        assert!(!out["RT_LOG"][3].is_finite()); // missing Vsh → MISSING
    }
}
