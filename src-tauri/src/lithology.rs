//! Lithology identification from APPARENT MATRIX properties — the inputs to the
//! Schlumberger MID (matrix identification) plot, chart Lith-6 (former CP-21), which is
//! already digitized on the frontend as the `lith6_mid` crossplot overlay.
//!
//! The idea: strip the fluid contribution out of the bulk density and the volumetric
//! photoelectric factor, leaving what the ROCK MATRIX alone would read. Cross-plotting
//! those two apparent matrix properties separates quartz, calcite and dolomite (and, off
//! to the sides, K-feldspar, the clays and anhydrite) without assuming a lithology first.
//!
//! Chain, per sample:
//!   rho_e  = (RHOB + 0.1883) / 1.0704        electron density from the tool's reading
//!   U      = PEF * rho_e                      volumetric photoelectric factor, b/cm3
//!   RHOMAA = (RHOB   - phi*RHO_FL) / (1 - phi)
//!   UMAA   = (U      - phi*U_FL  ) / (1 - phi)
//!
//! `RHOB` is used as LOGGED (the tool reports apparent density calibrated so that a
//! fresh-water-filled limestone reads its true bulk density). That is the same scale the
//! chart's own axis uses, which is what makes the mineral points line up: quartz has true
//! density 2.654 and 2Z/A = 0.9985, so rho_e = 2.650 and the tool reads
//! 1.0704*2.650 - 0.1883 = 2.648 — the chart plots quartz at RHOMAA 2.6489. Converting
//! RHOB to a "true" density before this step would put every point off the chart.
//!
//! FIELD TRAP: in BARITE mud the photoelectric curve is useless — barite's huge Pe swamps
//! the formation signal wherever mud invades or the hole is rugose, and every affected
//! sample flies off the right of the chart. Mask those intervals (the run's MASK option
//! takes a BADHOLE flag) rather than reading them.
//!
//! Shale is NOT corrected for. That is deliberate and matches the chart's design: shaly
//! samples drift toward the kaolinite and illite points, which Lith-6 plots, so the drift
//! is itself the reading.

use crate::modules::{log_in, log_out, opt, param, ModuleContext, ModuleOutputs, ModuleSpec};
use std::collections::HashMap;

/// Density-tool calibration: the tool measures electron density rho_e and reports
/// `RHOB = 1.0704*rho_e - 0.1883`, chosen so fresh water reads 1.00 and calcite 2.71
/// (Schlumberger, Log Interpretation Principles/Applications). Inverted here because the
/// photoelectric factor is per ELECTRON, so U must be built on rho_e, not on RHOB.
const RHOE_SLOPE: f64 = 1.0704;
const RHOE_OFFSET: f64 = 0.1883;

/// Electron density (g/cc) from the logged bulk density (g/cc).
pub(crate) fn electron_density(rhob_logged: f64) -> f64 {
    (rhob_logged + RHOE_OFFSET) / RHOE_SLOPE
}

pub fn midplot_spec() -> ModuleSpec {
    ModuleSpec {
        name: "midplot".into(),
        title: "Apparent Matrix (MID plot: UMAA / RHOMAA)".into(),
        category: "Lithology".into(),
        doc: "Apparent matrix density RHOMAA and apparent matrix volumetric photoelectric \
              factor UMAA — the two axes of the Schlumberger Lith-6 MID plot (crossplot X = UMAA, \
              Y = RHOMAA, then switch on the 'Lith-6 Umaa-Rhomaa MID plot' chart overlay). \
              U = PEF * rho_e with rho_e = (RHOB + 0.1883)/1.0704; the fluid is then removed from \
              both: RHOMAA = (RHOB - phi*RHO_FL)/(1 - phi), UMAA = (U - phi*U_FL)/(1 - phi). \
              \
              PHI here is the APPARENT porosity and it is the one judgement call in the method. \
              OPT_PHIA = XPLOT (default) averages the apparent-limestone density and neutron \
              porosities, the textbook basis; it is analytic, not a chart lookup, so it drags \
              points toward the assumed RHO_MA_A: at zero porosity quartz lands ~0.013 g/cc heavy \
              (UMAA within 0.001 of the chart) and dolomite ~0.06 g/cc light and ~0.34 b/cm3 left \
              of the chart's dolomite point. All three minerals still resolve to their own chart \
              point by a wide margin. NEUTRON uses the neutron alone (no density circularity, but \
              carries the neutron's matrix effect); LOG takes a porosity curve you already trust \
              (e.g. from SandiMin or a chart-based workflow) and is the accurate route. \
              \
              Density-only apparent porosity is deliberately NOT offered: it is algebraically \
              degenerate (it returns RHO_MA_A for every sample). Barite mud makes PEF, and \
              therefore UMAA, unreadable — mask those intervals."
            .into(),
        args: vec![
            opt(
                "OPT_PHIA",
                "Apparent porosity basis (see the method note — this choice moves the points)",
                "XPLOT",
                &["XPLOT", "NEUTRON", "LOG"],
            ),
            // 2.71 (limestone), NOT the 2.645 sandstone default the porosity modules use: the
            // neutron leg is apparent-LIMESTONE porosity, so the density leg has to be on the
            // same apparent-limestone basis or averaging the two is meaningless.
            param("RHO_MA_A", "Apparent matrix density for the density leg (limestone basis)", "g/cc", 2.71, 2.0, 3.2),
            param("RHO_FL", "Fluid density", "g/cc", 1.0, 0.5, 1.5),
            // Fresh water: Pe 0.358 x rho_e 1.111 = 0.398 b/cm3. Saline filtrate is higher
            // (chlorine is a strong photoelectric absorber), so the range allows well past it.
            param("U_FL", "Fluid volumetric photoelectric factor (0.398 = fresh water)", "b/cm3", 0.398, 0.0, 3.0),
            // (1 - phi) is the denominator of both outputs; near phi = 1 it explodes into values
            // that would wreck the crossplot's auto-range without meaning anything. A matrix
            // property read from a >50% porosity sample is not a rock property anyway.
            param("PHIA_MAX", "Reject samples whose apparent porosity exceeds this", "v/v", 0.5, 0.1, 0.9),
            log_in("RHOB", "Bulk density log, as logged", "g/cc", "RHOB", true),
            log_in("NPHI", "Neutron porosity log, apparent limestone", "v/v", "NPHI", true),
            log_in("PEF", "Photoelectric factor", "b/e", "PEF", true),
            log_in("PHI_IN", "Apparent porosity curve, used only when OPT_PHIA = LOG", "v/v", "PHIT", false),
            log_out("U", "Volumetric photoelectric factor", "b/cm3"),
            log_out("RHOMAA", "Apparent matrix density", "g/cc"),
            log_out("UMAA", "Apparent matrix volumetric photoelectric factor", "b/cm3"),
            log_out("PHIA", "Apparent porosity actually used", "v/v"),
        ],
    }
}

/// Which apparent porosity the run is built on. `Xplot` is the textbook average of the
/// apparent-limestone density and neutron porosities.
enum PhiBasis {
    Xplot,
    Neutron,
    Log,
}

pub fn midplot(ctx: &ModuleContext) -> ModuleOutputs {
    let rhob = ctx.log("RHOB");
    let nphi = ctx.log("NPHI");
    let pef = ctx.log("PEF");
    let phi_in = ctx.log("PHI_IN");
    let basis = match ctx.o("OPT_PHIA") {
        "NEUTRON" => PhiBasis::Neutron,
        "LOG" => PhiBasis::Log,
        _ => PhiBasis::Xplot,
    };

    let mut u_out = vec![f32::NAN; ctx.n];
    let mut rhomaa = vec![f32::NAN; ctx.n];
    let mut umaa = vec![f32::NAN; ctx.n];
    let mut phia_out = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let rb = rhob[i] as f64;
        let pe = pef[i] as f64;
        if rb.is_nan() || pe.is_nan() {
            continue;
        }
        let rho_ma_a = ctx.p("RHO_MA_A", i);
        let rho_fl = ctx.p("RHO_FL", i);
        let u_fl = ctx.p("U_FL", i);
        let phia_max = ctx.p("PHIA_MAX", i);
        if rho_ma_a.is_nan() || rho_fl.is_nan() || u_fl.is_nan() || phia_max.is_nan() {
            continue;
        }

        let phi = match basis {
            PhiBasis::Xplot => {
                let np = nphi[i] as f64;
                if np.is_nan() || rho_ma_a == rho_fl {
                    continue;
                }
                let phi_d = (rho_ma_a - rb) / (rho_ma_a - rho_fl);
                (phi_d + np) / 2.0
            }
            PhiBasis::Neutron => nphi[i] as f64,
            PhiBasis::Log => phi_in[i] as f64,
        };
        if phi.is_nan() || phi > phia_max {
            continue;
        }

        // U is per unit VOLUME: the photoelectric factor is per electron, so it scales by
        // electron density, not by the tool's reported bulk density.
        let u = pe * electron_density(rb);
        let one_minus_phi = 1.0 - phi;

        u_out[i] = u as f32;
        phia_out[i] = phi as f32;
        rhomaa[i] = ((rb - phi * rho_fl) / one_minus_phi) as f32;
        umaa[i] = ((u - phi * u_fl) / one_minus_phi) as f32;
    }

    HashMap::from([
        ("U".to_string(), u_out),
        ("RHOMAA".to_string(), rhomaa),
        ("UMAA".to_string(), umaa),
        ("PHIA".to_string(), phia_out),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::DepthUnit;

    /// Chart Lith-6's plotted mineral points (from the digitized `lith6_mid` overlay in
    /// `src/ui/chartOverlays.ts`): (UMAA b/cm3, RHOMAA g/cc).
    const QUARTZ_PT: (f64, f64) = (4.823, 2.6489);
    const CALCITE_PT: (f64, f64) = (13.836, 2.7113);
    const DOLOMITE_PT: (f64, f64) = (9.048, 2.8718);

    /// What the density tool reads for a zero-porosity mineral of the given TRUE density and
    /// 2Z/A ratio — the forward direction of the calibration `electron_density` inverts.
    fn logged_density(true_density: f64, two_z_over_a: f64) -> f64 {
        RHOE_SLOPE * (true_density * two_z_over_a) - RHOE_OFFSET
    }

    fn run(
        rhob: f64,
        nphi: f64,
        pef: f64,
        basis: &str,
        params: &[(&str, f64)],
    ) -> HashMap<String, Vec<f32>> {
        let mut p: HashMap<String, Vec<f64>> = HashMap::from([
            ("RHO_MA_A".to_string(), vec![2.71]),
            ("RHO_FL".to_string(), vec![1.0]),
            ("U_FL".to_string(), vec![0.398]),
            ("PHIA_MAX".to_string(), vec![0.5]),
        ]);
        for (k, v) in params {
            p.insert((*k).to_string(), vec![*v]);
        }
        let ctx = ModuleContext {
            n: 1,
            logs: HashMap::from([
                ("RHOB".to_string(), vec![rhob as f32]),
                ("NPHI".to_string(), vec![nphi as f32]),
                ("PEF".to_string(), vec![pef as f32]),
            ]),
            params: p,
            opts: HashMap::from([("OPT_PHIA".to_string(), basis.to_string())]),
            depth_unit: DepthUnit::Metres,
        };
        midplot(&ctx)
    }

    /// The calibration must round-trip: fresh water reads 1.00 and calcite 2.71 on the tool.
    #[test]
    fn electron_density_calibration_matches_the_tool() {
        // Water: 2Z/A = 20/18.015 = 1.1102, true density 1.0.
        assert!((logged_density(1.0, 1.1102) - 1.0).abs() < 0.002);
        // Calcite: 2Z/A = 100/100.09 = 0.99910, true density 2.710.
        assert!((logged_density(2.710, 0.99910) - 2.710).abs() < 0.002);
        // And the inverse used by the module returns the electron density it started from.
        let rho_e = 2.650;
        assert!((electron_density(RHOE_SLOPE * rho_e - RHOE_OFFSET) - rho_e).abs() < 1e-9);
    }

    /// With zero apparent porosity there is no fluid to strip, so the apparent matrix
    /// properties must be exactly the sample's own — this pins the formula itself, free of
    /// any porosity-basis question.
    #[test]
    fn zero_apparent_porosity_returns_the_sample_itself() {
        let rhob = 2.7095;
        let pef = 5.084;
        let out = run(rhob, 0.0, pef, "NEUTRON", &[]);
        let u = pef * electron_density(rhob);
        assert!((out["RHOMAA"][0] as f64 - rhob).abs() < 1e-5);
        assert!((out["UMAA"][0] as f64 - u).abs() < 1e-5);
        assert!((out["U"][0] as f64 - u).abs() < 1e-5);
        assert_eq!(out["PHIA"][0], 0.0);
    }

    /// The real test of the method: a zero-porosity sample of each mineral must land nearer
    /// its OWN chart point than either other mineral's. Distances are normalized by the
    /// chart's axis spans (UMAA ~0-16 b/cm3, RHOMAA ~2.2-3.1 g/cc) so the two axes are
    /// comparable — an un-normalized distance would be decided almost entirely by UMAA.
    ///
    /// Neutron readings are the apparent-LIMESTONE values at zero true porosity, taken from
    /// the chartbook tables already digitized in `neutron_charts.rs` (Por-5 CNL NPHI):
    /// sandstone reaches zero porosity at about -0.019, dolomite at about +0.027. Guessing
    /// these matters — an earlier +0.010 guess for dolomite reversed which porosity basis
    /// looked better, which is why they come from the tables.
    #[test]
    fn each_mineral_lands_nearest_its_own_chart_point() {
        let minerals: [(&str, f64, f64, f64, f64, (f64, f64)); 3] = [
            // name, true density, 2Z/A, Pe, apparent-limestone NPHI at zero porosity, chart point
            ("quartz", 2.654, 0.99850, 1.806, -0.019, QUARTZ_PT),
            ("calcite", 2.710, 0.99910, 5.084, 0.0, CALCITE_PT),
            ("dolomite", 2.870, 0.99778, 3.142, 0.027, DOLOMITE_PT),
        ];
        // Normalized distance to a chart point.
        let dist = |u: f64, r: f64, pt: (f64, f64)| -> f64 {
            let du = (u - pt.0) / 16.0;
            let dr = (r - pt.1) / 0.9;
            (du * du + dr * dr).sqrt()
        };

        for (name, density, zoa, pe, nphi, own) in minerals {
            let out = run(logged_density(density, zoa), nphi, pe, "XPLOT", &[]);
            let u = out["UMAA"][0] as f64;
            let r = out["RHOMAA"][0] as f64;
            let d_own = dist(u, r, own);
            for other in [QUARTZ_PT, CALCITE_PT, DOLOMITE_PT] {
                if other == own {
                    continue;
                }
                assert!(
                    d_own < dist(u, r, other),
                    "{name} plotted at ({u:.3}, {r:.4}), closer to {other:?} than to its own {own:?}",
                );
            }
            // And the bias stays inside what the module's doc string promises.
            assert!(d_own < 0.12, "{name} is {d_own:.3} from its chart point — doc says the analytic basis stays well inside this");
        }
    }

    /// Quartz is the case the analytic average handles best: UMAA should sit essentially on
    /// the chart's quartz point, with the whole error in RHOMAA. Pinning the documented
    /// numbers means a change to the physics cannot quietly move them.
    #[test]
    fn quartz_bias_matches_the_documented_numbers() {
        let out = run(logged_density(2.654, 0.99850), -0.019, 1.806, "XPLOT", &[]);
        let u = out["UMAA"][0] as f64;
        let r = out["RHOMAA"][0] as f64;
        assert!((u - QUARTZ_PT.0).abs() < 0.01, "UMAA {u:.4} should be within 0.01 of {}", QUARTZ_PT.0);
        let heavy = r - QUARTZ_PT.1;
        assert!((heavy - 0.013).abs() < 0.005, "RHOMAA bias {heavy:.4} should be the documented ~+0.013 g/cc");
    }

    /// A density-derived apparent porosity is algebraically degenerate — RHOMAA collapses to
    /// the assumed RHO_MA_A for EVERY sample, a constant curve that still plots. The module
    /// therefore offers no such option; this test states the trap so nobody adds one back.
    #[test]
    fn density_only_apparent_porosity_would_be_degenerate() {
        let (rho_ma_a, rho_fl): (f64, f64) = (2.71, 1.0);
        for rhob in [2.35f64, 2.55, 2.648, 2.87] {
            let phi_d = (rho_ma_a - rhob) / (rho_ma_a - rho_fl);
            let rhomaa = (rhob - phi_d * rho_fl) / (1.0 - phi_d);
            assert!(
                (rhomaa - rho_ma_a).abs() < 1e-9,
                "density-only phi returns the assumed matrix density, carrying no information",
            );
        }
    }

    /// Samples too porous to carry a matrix reading, and samples missing a tool, drop out as
    /// NaN rather than as a large finite number that would look like data.
    #[test]
    fn rejects_over_porous_and_incomplete_samples() {
        let over = run(2.0, 0.6, 3.0, "NEUTRON", &[]);
        assert!(over["RHOMAA"][0].is_nan() && over["UMAA"][0].is_nan());
        // Raising the limit lets the same sample through — the rejection is the parameter,
        // not a hidden constant.
        let allowed = run(2.0, 0.6, 3.0, "NEUTRON", &[("PHIA_MAX", 0.7)]);
        assert!(allowed["RHOMAA"][0].is_finite() && allowed["UMAA"][0].is_finite());

        let no_pef = run(2.5, 0.1, f64::NAN, "NEUTRON", &[]);
        assert!(no_pef["UMAA"][0].is_nan() && no_pef["RHOMAA"][0].is_nan());
        // XPLOT needs the neutron; NEUTRON basis without it must not invent one either.
        let no_nphi = run(2.5, f64::NAN, 3.0, "XPLOT", &[]);
        assert!(no_nphi["RHOMAA"][0].is_nan());
    }
}
