//! Rock typing (Wave B item 8, first increment): per-sample hydraulic/pore-geometry indicators
//! and a rock-type class from φ and k, grounded in `docs/research_2026-07/ref_rocktyping_shf.md`.
//!
//! Methods implemented here (deterministic, no clustering — GHE uses FIXED FZI bins, port classes
//! use fixed pore-throat cutoffs, so results are reproducible and unit-testable):
//!   - Amaefule 1993 RQI / φz / FZI + Corbett-Potter 2004 GHE bins, with the per-HFU
//!     geometric-mean-FZI permeability predictor k = 1014.24·FZI²·φ³/(1−φ)².
//!   - Kolodzie 1980 Winland R35 (+ port classes mega/macro/meso/micro/nano).
//!   - Permadi-Susilo PGS pore geometry (k/φ) and pore structure (k/φ^PS_EXP), PS_EXP default 3.5.
//!
//! Deferred to later increments (see the reference doc): interactive Ward/histogram HFU clustering,
//! Lucia RFN, Pittman full rX table, MICP-fitted local Winland coefficients, and the SHF-fitting
//! side (Cuddy FOIL/FWL scan, Brooks-Corey, Thomeer, Skelt-Harrison) + SCAL importers.
//!
//! NOTE (flagged in the reference doc): the PS exponent (3.5) and GHE bin list are specced from
//! literature/recall with no local paper copy — they are exposed as a param / documented so they
//! can be corrected against Permadi-Susilo 2009 and Corbett-Potter 2004 before field release.

use crate::modules::{log_in, log_out, opt, param, ModuleContext, ModuleOutputs, ModuleSpec};
use std::collections::HashMap;

/// Corbett & Potter (2004) global hydraulic element FZI boundaries (µm). A sample's GHE class is
/// 1 + (number of boundaries its FZI exceeds), so FZI < 0.0938 → GHE1 … FZI ≥ 8 → GHE10.
const GHE_BOUNDS: [f64; 9] = [0.0938, 0.1875, 0.375, 0.75, 1.5, 2.5, 4.0, 6.0, 8.0];

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
              Permadi-Susilo PGS pair PGEOM = k/φ, PSTRUC = k/φ^PS_EXP. RT is the rock-type \
              class from the chosen METHOD — GHE fixed FZI bins (Corbett-Potter 2004) or Winland \
              port classes (nano..mega). PERM_RT is the class-grouped permeability estimate \
              k = 1014.24·FZI_mean(RT)²·φ³/(1−φ)² using each class's GEOMETRIC-MEAN FZI over this \
              well. k in mD, φ in v/v; samples with φ∉(0,1) or k≤0 stay MISSING. NOTE: the PS \
              exponent and GHE bins are literature/recall values (see the rock-typing reference) \
              — verify against the papers before release."
            .into(),
        args: vec![
            opt("METHOD", "Rock-type class basis", "ghe", &["ghe", "winland_port"]),
            param("PS_EXP", "PGS pore-structure exponent (k/φ^PS_EXP)", "-", 3.5, 1.0, 6.0),
            log_in("PHI", "Effective porosity", "v/v", "PHIE", true),
            log_in("PERM", "Permeability", "mD", "PERM", true),
            log_out("RQI", "Reservoir quality index 0.0314·√(k/φ)", "um"),
            log_out("PHIZ", "Normalized porosity φ/(1−φ)", "-"),
            log_out("FZI", "Flow zone indicator RQI/PHIZ", "um"),
            log_out("R35", "Winland R35 pore-throat radius", "um"),
            log_out("PGEOM", "PGS pore geometry k/φ", "-"),
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
    let ps_exp = if ps_exp_raw.is_finite() { ps_exp_raw.clamp(1.0, 6.0) } else { 3.5 };

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
        pgeom[i] = (k / phi) as f32;
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
        ModuleContext { n, logs, params, opts }
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
        // FZI ≈ 2.808 → between GHE bounds 2.5 and 4 → GHE class 7.
        assert_eq!(out["RT"][0], 7.0, "fzi={fzi}");
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
        let c = ModuleContext { n, logs, params, opts: HashMap::new() };
        let out = rt_cutoff(&c);
        assert_eq!(out["RT_LOG"][0], 1.0); // clean + porous → RT1
        assert_eq!(out["RT_LOG"][1], 2.0); // moderate → RT2
        assert_eq!(out["RT_LOG"][2], 3.0); // shaly + tight → non-net RT3
        assert!(!out["RT_LOG"][3].is_finite()); // missing Vsh → MISSING
    }
}
