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

/// Porosity bracket the crossplot lookup searches. Wider than any reservoir on purpose:
/// the ends are where dense (anhydrite, pyrite) and washed-out samples land, and a sample
/// that wants to sit outside is clamped to the end rather than dropped, so it still plots
/// in the right corner of the MID chart instead of vanishing.
const PHI_SEARCH_LO: f64 = -0.05;
const PHI_SEARCH_HI: f64 = 0.60;

/// Piecewise-linear interpolation of `ys` against strictly-increasing `xs` at `x`, with the
/// end segments extended beyond the ends. Used across the three matrix lines, so the
/// extrapolation is what carries gas-affected and off-family samples.
fn lerp_across(xs: &[f64; 3], ys: &[f64; 3], x: f64) -> f64 {
    let seg = |i: usize, j: usize| -> f64 {
        let dx = xs[j] - xs[i];
        if dx.abs() < 1e-9 {
            return (ys[i] + ys[j]) / 2.0;
        }
        ys[i] + (x - xs[i]) * (ys[j] - ys[i]) / dx
    };
    if x <= xs[1] {
        seg(0, 1)
    } else {
        seg(1, 2)
    }
}

/// Apparent porosity by a genuine density-neutron crossplot LOOKUP — what you do by hand on
/// chart Por-11, rather than averaging the two porosities and hoping.
///
/// The chart is a family of matrix lines (sandstone, limestone, dolomite) graduated in true
/// porosity. Reading it means finding the point on that family matching BOTH tools. That
/// looks like two unknowns, but it collapses to one: at any trial porosity each tool implies
/// a matrix density on its own,
///
///   density says   rho_ma = (RHOB - phi*RHO_FL) / (1 - phi)        — rises with phi
///   neutron says   rho_ma = where NPHI falls across the three lines' readings at phi
///                                                                   — falls with phi
///
/// so their difference is strictly monotone and has exactly one root. Bisection finds it
/// without derivatives or an initial guess, and cannot diverge — which matters because this
/// runs per sample over every well in a field.
///
/// `nphi_ls` must be APPARENT LIMESTONE (the chart's x-axis). Run `nphimat` first if the log
/// is recorded in sandstone or dolomite units.
fn crossplot_porosity(
    rhob: f64,
    nphi_ls: f64,
    rho_fl: f64,
    rho_ma: [f64; 3],
    tables: (&[(f32, f32)], &[(f32, f32)]),
) -> f64 {
    // What each matrix line reads on the apparent-limestone axis at this true porosity.
    // Limestone IS that axis, so it is the identity; the other two come from the chartbook
    // tables, read backwards (true porosity -> apparent limestone).
    let matrix_density_from_neutron = |phi: f64| -> f64 {
        let readings = [
            crate::modules::chart_lerp(tables.0, phi, true),
            phi,
            crate::modules::chart_lerp(tables.1, phi, true),
        ];
        lerp_across(&readings, &rho_ma, nphi_ls)
    };
    let disagreement = |phi: f64| -> f64 {
        (rhob - phi * rho_fl) / (1.0 - phi) - matrix_density_from_neutron(phi)
    };

    let (mut lo, mut hi) = (PHI_SEARCH_LO, PHI_SEARCH_HI);
    let f_lo = disagreement(lo);
    let f_hi = disagreement(hi);
    if !f_lo.is_finite() || !f_hi.is_finite() {
        return f64::NAN;
    }
    // No sign change means the root lies outside the bracket: the sample is denser than any
    // matrix line (clamps low, plotting high on the MID chart, as anhydrite and pyrite
    // should) or more porous than the bracket (clamps high, where PHIA_MAX then rejects it).
    if f_lo > 0.0 {
        return lo;
    }
    if f_hi < 0.0 {
        return hi;
    }
    // Bisection: ~20 halvings take the 0.65-wide bracket below 1e-6 porosity units, three
    // orders finer than any log resolves. The cap is a backstop, not the exit condition.
    for _ in 0..50 {
        if hi - lo < 1e-6 {
            break;
        }
        let mid = 0.5 * (lo + hi);
        if disagreement(mid) > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    0.5 * (lo + hi)
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
              PHI here is the APPARENT porosity, the one judgement call in the method. \
              OPT_PHIA = CHART (default) reads it off the density-neutron crossplot the way you \
              would by hand on Por-11: it solves for the porosity at which the density and the \
              neutron imply the SAME matrix, interpolating across the chartbook's sandstone, \
              limestone and dolomite curves (pick the curve family with TOOL/SALINITY, exactly as \
              in Neutron Matrix Conversion). Build a rock's two tool readings, feed them back, and \
              this returns that rock: porosity to 1e-3 and RHOMAA onto its own matrix line, for \
              sandstone, limestone and dolomite alike. Rocks denser than every matrix line \
              (anhydrite, pyrite) clamp to the end of the search and stay heavy rather than \
              dropping out, and gas pushes points low-left just as it does on the printed chart. \
              \
              XPLOT is the analytic average commercial suites take — kept for comparison, not for \
              accuracy: it drags points toward the assumed RHO_MA_A, leaving dolomite about 0.06 \
              g/cc light and 0.34 b/cm3 left of its chart point. NEUTRON uses the neutron alone. \
              LOG takes a porosity curve you already trust. NPHI must be APPARENT LIMESTONE for \
              CHART and NEUTRON — run Neutron Matrix Conversion first if the log is recorded in \
              sandstone or dolomite units. \
              \
              Density-only apparent porosity is deliberately NOT offered: it is algebraically \
              degenerate (it returns RHO_MA_A for every sample). Barite mud makes PEF, and \
              therefore UMAA, unreadable — mask those intervals."
            .into(),
        args: vec![
            opt(
                "OPT_PHIA",
                "Apparent porosity basis (see the method note — this choice moves the points)",
                "CHART",
                &["CHART", "XPLOT", "NEUTRON", "LOG"],
            ),
            opt(
                "TOOL",
                "Neutron measurement the log comes from, for the CHART lookup (same chart families as Neutron Matrix Conversion)",
                "TNPH",
                &["TNPH", "NPHI", "APLC", "FPLC", "SNP"],
            ),
            opt(
                "SALINITY",
                "Formation salinity for the CHART lookup (TNPH curves only; SALT_250K = 250,000 ppm)",
                "FRESH",
                &["FRESH", "SALT_250K"],
            ),
            // The chart's three matrix lines, in the density scale the tool reports (the
            // chart's own y-axis). Exposed because a arkosic or heavy-mineral sand is not
            // 2.65 — the same reason every porosity module here takes RHO_MA.
            param("RHO_MA_SS", "Sandstone matrix line, CHART lookup", "g/cc", 2.65, 2.0, 3.2),
            param("RHO_MA_LS", "Limestone matrix line, CHART lookup", "g/cc", 2.71, 2.0, 3.2),
            param("RHO_MA_DOL", "Dolomite matrix line, CHART lookup", "g/cc", 2.87, 2.0, 3.2),
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

/// Which apparent porosity the run is built on. `Chart` solves the density-neutron
/// crossplot; `Xplot` is the analytic average of the apparent-limestone density and
/// neutron porosities that commercial suites take.
enum PhiBasis {
    Chart,
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
        "XPLOT" => PhiBasis::Xplot,
        "NEUTRON" => PhiBasis::Neutron,
        "LOG" => PhiBasis::Log,
        _ => PhiBasis::Chart,
    };
    let tables = crate::modules::nphimat_tables(ctx.o("TOOL"), ctx.o("SALINITY") == "SALT_250K");

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
            PhiBasis::Chart => {
                let np = nphi[i] as f64;
                let lines = [ctx.p("RHO_MA_SS", i), ctx.p("RHO_MA_LS", i), ctx.p("RHO_MA_DOL", i)];
                // The lookup interpolates ACROSS the three lines, so they have to stay in
                // chart order; a zone override that crossed them would silently invert the
                // lithology axis.
                if np.is_nan() || lines.iter().any(|v| v.is_nan()) || !(lines[0] < lines[1] && lines[1] < lines[2]) {
                    continue;
                }
                crossplot_porosity(rb, np, rho_fl, lines, tables)
            }
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
            ("RHO_MA_SS".to_string(), vec![2.65]),
            ("RHO_MA_LS".to_string(), vec![2.71]),
            ("RHO_MA_DOL".to_string(), vec![2.87]),
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

    /// The three chart matrix lines, in the order the lookup interpolates across them.
    const MATRIX_LINES: [f64; 3] = [2.65, 2.71, 2.87];

    /// Forward model: what the density and neutron tools read for a rock of the given matrix
    /// (0 = sandstone, 1 = limestone, 2 = dolomite) at the given TRUE porosity. The neutron
    /// side is the chartbook table read backwards — true porosity to apparent limestone —
    /// which is the direction chart Por-11's matrix curves are graduated in. Defaults
    /// (TNPH, fresh), matching what `run` leaves the TOOL option as.
    fn tool_readings(matrix: usize, phi: f64) -> (f64, f64) {
        let (t_ss, t_dol) = crate::modules::nphimat_tables("TNPH", false);
        let rhob = MATRIX_LINES[matrix] * (1.0 - phi) + 1.0 * phi;
        let nphi = match matrix {
            0 => crate::modules::chart_lerp(t_ss, phi, true),
            2 => crate::modules::chart_lerp(t_dol, phi, true),
            _ => phi,
        };
        (rhob, nphi)
    }

    /// The test that decides whether the chart lookup is real: build the two readings a known
    /// rock would produce, hand them back, and require the solver to return that rock.
    /// Porosity must come back to bisection tolerance and RHOMAA must land on the matrix line
    /// itself — no bias left to document, because nothing was approximated.
    #[test]
    fn chart_lookup_recovers_the_rock_it_was_built_from() {
        for (matrix, name) in [(0, "sandstone"), (1, "limestone"), (2, "dolomite")] {
            for phi in [0.0, 0.05, 0.12, 0.25, 0.35] {
                let (rhob, nphi) = tool_readings(matrix, phi);
                let out = run(rhob, nphi, 3.0, "CHART", &[]);
                let got_phi = out["PHIA"][0] as f64;
                let got_rho = out["RHOMAA"][0] as f64;
                assert!((got_phi - phi).abs() < 1e-3, "{name} at phi {phi}: recovered {got_phi:.5}");
                assert!(
                    (got_rho - MATRIX_LINES[matrix]).abs() < 2e-3,
                    "{name} at phi {phi}: RHOMAA {got_rho:.4}, want the {} line",
                    MATRIX_LINES[matrix],
                );
            }
        }
    }

    /// The payoff over the analytic average, on the case that motivated the work: dolomite.
    /// CHART must put it on its own matrix line; XPLOT is the one that drifts.
    #[test]
    fn chart_lookup_removes_the_dolomite_bias_that_the_average_carries() {
        let (rhob, nphi) = tool_readings(2, 0.0);
        let chart = run(rhob, nphi, 3.142, "CHART", &[])["RHOMAA"][0] as f64;
        let xplot = run(rhob, nphi, 3.142, "XPLOT", &[])["RHOMAA"][0] as f64;
        let dolomite = MATRIX_LINES[2];
        assert!((chart - dolomite).abs() < 2e-3, "CHART put dolomite at {chart:.4}");
        assert!(
            (chart - dolomite).abs() * 10.0 < (xplot - dolomite).abs(),
            "CHART {chart:.4} should beat XPLOT {xplot:.4} against the {dolomite} line by a wide margin",
        );
        // And the average drifts in the documented direction: too light.
        assert!(xplot < dolomite);
    }

    /// Off-family samples must stay diagnostic rather than vanish. A rock denser than every
    /// matrix line (anhydrite, pyrite) clamps to the low end of the search and still reports
    /// a heavy RHOMAA — which is exactly the corner of the MID chart it belongs in.
    #[test]
    fn denser_than_any_matrix_line_clamps_instead_of_failing() {
        let out = run(2.98, 0.0, 5.05, "CHART", &[]);
        let rho = out["RHOMAA"][0] as f64;
        assert!(rho.is_finite() && rho > 2.9, "anhydrite should stay heavy, got {rho:.4}");
        assert!(out["UMAA"][0].is_finite());
    }

    /// A zone override that crossed the three matrix lines would silently invert the
    /// lithology axis, so out-of-order lines are refused rather than interpolated.
    #[test]
    fn out_of_order_matrix_lines_are_refused() {
        let (rhob, nphi) = tool_readings(1, 0.1);
        assert!(run(rhob, nphi, 3.0, "CHART", &[])["RHOMAA"][0].is_finite());
        let crossed = run(rhob, nphi, 3.0, "CHART", &[("RHO_MA_DOL", 2.60)]);
        assert!(crossed["RHOMAA"][0].is_nan() && crossed["PHIA"][0].is_nan());
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
