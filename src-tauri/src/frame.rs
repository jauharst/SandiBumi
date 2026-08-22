//! **Frame** — the depth-sampling module family: upscaling a curve to beds, and finding the beds
//! to upscale it to. The companion category to [`crate::condition`] (Jauhar, 2026-08-05).
//!
//! ## What is here, and what deliberately is not
//!
//! A module's outputs are written at the RUN's own depth frame — `ancestry::write_versioned_rows_raw`
//! zips each output vector against the depth column the run read. So an operation that changes
//! how often a well is sampled cannot be a module: it would have to write a different depth
//! column, which is a property of the well rather than of one curve. Resample, Regularize and
//! Reverse/Sort are therefore NOT here. Sorting an out-of-order depth column is an import
//! problem and belongs in the intake; re-framing a well is a well-level operation.
//!
//! Blocking IS module-shaped, because a blocked curve is constant within each bed while still
//! being sampled on the original frame — which is also the honest way to store it, since nothing
//! downstream then has to know it was upscaled.
//!
//! ## The rule that makes this petrophysics rather than arithmetic
//!
//! **Upscaling permeability with an arithmetic mean is wrong, and wrong in the flattering
//! direction.** Flow through layered rock takes the arithmetic mean only when the layers are in
//! PARALLEL with the flow; in SERIES it takes the harmonic mean, and for a randomly arranged
//! heterogeneous medium the geometric mean is the standard estimate. The three differ by orders
//! of magnitude on a laminated sand-shale: a bed of 1000 mD sand and 0.01 mD shale in equal
//! parts is 500 mD arithmetically, 0.3 mD geometrically and 0.02 mD harmonically. An arithmetic
//! upscale of a laminated interval hands a simulator a permeability the rock does not have, and
//! it is the ONE of the three that always reads highest — so the error never looks like a
//! problem. `OPT_STAT` therefore has no "obviously right" default and the doc names the case for
//! each; MEAN is kept as the default only because it is right for porosity and for every volume
//! fraction, which is most of what gets blocked.

use crate::modules::{
    log_in, log_out_as, opt_labelled, param, param_open, param_open_when, ModuleContext,
    ModuleOutputs, ModuleSpec, PROJECT_DEPTH_UNIT_TOKEN,
};
use std::collections::HashMap;

const MISSING: f32 = f32::NAN;

/// A run parameter that is constant for the well — a block interval cannot vary by zone without
/// the boundary being two thicknesses at once. Same helper and same reasoning as `condition`.
fn constant(ctx: &ModuleContext, name: &str) -> f64 {
    (0..ctx.n).map(|i| ctx.p(name, i)).find(|v| v.is_finite()).unwrap_or(f64::NAN)
}

fn yes(ctx: &ModuleContext, name: &str, default: bool) -> bool {
    match ctx.o(name) {
        "YES" => true,
        "NO" => false,
        _ => default,
    }
}

// ---------------------------------------------------------------------------
// Bed assignment
// ---------------------------------------------------------------------------

/// Per-sample bed ordinal (NaN where the sample belongs to no bed), by whichever definition the
/// run asked for. Beds are numbered from the top so the index is monotone with depth.
fn assign_beds(ctx: &ModuleContext, vals: &[f32], depth: &[f32]) -> Result<Vec<f32>, String> {
    match ctx.o("OPT_BEDS") {
        "CLASS" => {
            // Each RUN of a constant class value is one bed. The class curve is whatever the user
            // points at — FACIES, a rock type, a lithology code, a 0/1 flag — so the boundaries
            // are where the ROCK changes rather than where a ruler falls. This is the definition
            // a simulation model wants.
            let cls = ctx.log("BEDS");
            if cls.iter().all(|v| !v.is_finite()) {
                return Err("OPT_BEDS = CLASS needs a BEDS curve with values in it — that curve \
                            is what says where one bed ends and the next begins."
                    .into());
            }
            let mut out = vec![MISSING; ctx.n];
            let mut bed = -1.0f32;
            let mut prev: Option<f32> = None;
            for i in 0..ctx.n {
                let v = cls[i];
                if !v.is_finite() {
                    prev = None; // a gap in the class curve ends the bed rather than spanning it
                    continue;
                }
                if prev.map_or(true, |p| p != v) {
                    bed += 1.0;
                }
                out[i] = bed;
                prev = Some(v);
            }
            Ok(out)
        }
        "ZONES" => {
            // The project's own markers, supplied per sample by the runner (`workflow.rs`
            // ZONE_INDEX_ARG) because a module has no database handle.
            let idx: Vec<f32> = (0..ctx.n)
                .map(|i| {
                    let v = ctx.p(crate::workflow::ZONE_INDEX_ARG, i);
                    if v.is_finite() {
                        v as f32
                    } else {
                        MISSING
                    }
                })
                .collect();
            if idx.iter().all(|v| !v.is_finite()) {
                return Err("OPT_BEDS = ZONES needs tops on this well — nothing here falls inside \
                            a zone, so there is no marker interval to average over."
                    .into());
            }
            Ok(idx)
        }
        "AUTO" => {
            let min_bed = constant(ctx, "MIN_BED");
            if !(min_bed > 0.0) {
                return Err("MIN_BED must be set for OPT_BEDS = AUTO — the thinnest bed worth \
                            calling a bed is a property of the tool and the rock, and there is no \
                            value that is right in two basins."
                    .into());
            }
            Ok(detect_beds(vals, depth, constant(ctx, "SENS"), min_bed))
        }
        // INTERVAL and anything unrecognised. A fixed slice ignores the rock — a contact falling
        // mid-slice is averaged across — which is why it is not the only option, but it is the
        // reproducible one and it needs no other curve.
        _ => {
            let interval = constant(ctx, "INTERVAL");
            if !(interval > 0.0) {
                return Err("INTERVAL must be set for OPT_BEDS = INTERVAL — how thick a block is \
                            is the whole content of an upscale."
                    .into());
            }
            // Anchored on the FIRST live depth rather than on zero, so the same setting gives the
            // same blocks whether a well starts at 0 or at 1500 — and so the top block is a full
            // one instead of whatever fraction the well's start happened to leave.
            let origin = depth.iter().copied().find(|d| d.is_finite()).unwrap_or(0.0) as f64;
            Ok((0..ctx.n)
                .map(|i| {
                    let d = depth[i] as f64;
                    if d.is_finite() {
                        ((d - origin) / interval).floor() as f32
                    } else {
                        MISSING
                    }
                })
                .collect())
        }
    }
}

/// Sequential segmentation: a sample opens a new bed when it sits further from the bed's running
/// mean than `sens` times the curve's own robust noise, and the bed already spans `min_bed`.
///
/// Deliberately sequential rather than an optimal partition. An optimal segmentation needs the
/// number of beds up front — which is the answer, not the question — and its boundaries move when
/// a sample far away changes, so two runs over overlapping intervals disagree about the same
/// contact. A sequential rule gives the same boundary wherever the window starts above it.
///
/// The noise scale is the robust spread of the FIRST DIFFERENCES, divided by root two: successive
/// samples of a curve differ by measurement noise plus whatever the rock is doing, and over a
/// bed the rock is doing nothing, so the differences are noise on two samples' worth. Using the
/// spread of the VALUES instead would scale with how much the curve varies across the whole well,
/// so a well with one big shale would find no beds anywhere else.
fn detect_beds(vals: &[f32], depth: &[f32], sens: f64, min_bed: f64) -> Vec<f32> {
    let n = vals.len();
    let mut diffs: Vec<f32> = Vec::new();
    for i in 1..n {
        if vals[i].is_finite() && vals[i - 1].is_finite() {
            diffs.push((vals[i] - vals[i - 1]).abs());
        }
    }
    diffs.sort_by(|a, b| a.partial_cmp(b).expect("finite by construction"));
    let noise = if diffs.is_empty() {
        0.0
    } else {
        // C_MAD x MAD-about-zero of |Δ|, then / sqrt(2) for the two-sample difference.
        (crate::robust::C_MAD * crate::distribution::percentile(&diffs, 50.0) as f64)
            / std::f64::consts::SQRT_2
    };
    let sens = if sens.is_finite() && sens > 0.0 { sens } else { 2.0 };
    let threshold = noise * sens;

    let mut out = vec![MISSING; n];
    let mut bed = 0.0f32;
    let mut sum = 0.0f64;
    let mut count = 0u32;
    let mut bed_top: Option<f64> = None;
    for i in 0..n {
        let (v, d) = (vals[i], depth[i] as f64);
        if !v.is_finite() || !d.is_finite() {
            continue;
        }
        let top = *bed_top.get_or_insert(d);
        let mean = if count > 0 { sum / count as f64 } else { v as f64 };
        // The minimum thickness is checked BEFORE the break, not after: a bed cannot be closed
        // until it is thick enough to be one, which is what stops a noisy interval being cut into
        // a bed per sample. Without it `sens` alone would have to carry that job, and the two are
        // different questions — how far off is unusual, and how thin is too thin to be rock.
        if count > 0 && (d - top) >= min_bed && ((v as f64) - mean).abs() > threshold && threshold > 0.0 {
            bed += 1.0;
            sum = 0.0;
            count = 0;
            bed_top = Some(d);
        }
        out[i] = bed;
        sum += v as f64;
        count += 1;
    }
    out
}

// ---------------------------------------------------------------------------
// BLOCK
// ---------------------------------------------------------------------------

pub fn block_spec() -> ModuleSpec {
    ModuleSpec {
        name: "block".into(),
        title: "Block (Upscale)".into(),
        category: "Frame".into(),
        doc: "Replaces a curve with one value per bed, held across the bed. The curve stays on \
              the well's own depth frame, so nothing downstream has to know it was upscaled — \
              set its draw style to Step in the curve editor, or the log view draws a gradient \
              between two block values that the data never measured.\n\n\
              OPT_BEDS — what a bed is:\n\
              • INTERVAL — equal slices of INTERVAL thickness. Reproducible, and it ignores the \
              rock: a contact falling mid-slice is averaged across.\n\
              • CLASS — each run of a constant value in the BEDS curve (FACIES, a rock type, a \
              flag). Boundaries are where the rock changes; this is what a simulation model wants.\n\
              • ZONES — one value per marker interval. The coarsest, and what a zone-parameter \
              table or a volumetrics summary consumes.\n\
              • AUTO — boundaries found from the curve itself, needing no other curve. They are \
              INFERRED, and the run says so.\n\n\
              OPT_STAT — how a bed's value is taken. **This is a petrophysical choice, not a \
              formatting one.**\n\
              • MEAN — right for porosity and for every volume fraction, because those add.\n\
              • GEOMETRIC — the standard estimate for PERMEABILITY in randomly heterogeneous \
              rock.\n\
              • HARMONIC — permeability across layers in SERIES (flow perpendicular to \
              lamination); the lowest of the three and the one a vertical barrier deserves.\n\
              • MEDIAN / MIN / MAX — order statistics, for a flag or a worst-case screen.\n\
              • MODE — the bed's commonest value, and the ONLY upscale for a class curve (FACIES, \
              a lithology code). A class code is a name written as a number: the mean of facies 1 \
              and facies 4 is 2.5, which is not a facies, and nothing downstream can tell. A curve \
              declared as a class refuses every averaging statistic here for that reason.\n\n\
              An arithmetic upscale of a laminated sand-shale gives a permeability the rock does \
              not have, and it is always the HIGHEST of the three, so the error never looks like \
              a problem: 1000 mD sand with 0.01 mD shale in equal parts is 500 mD arithmetically, \
              0.3 mD geometrically and 0.02 mD harmonically."
            .into(),
        args: vec![
            param_open_when(
                "INTERVAL",
                "Block thickness (OPT_BEDS = INTERVAL)",
                PROJECT_DEPTH_UNIT_TOKEN,
                0.0,
                10000.0,
                &[("OPT_BEDS", "INTERVAL")],
                "docs/PRD_v2/20_envcorr-qc.md §5.3 frame parameters",
            ),
            param_open_when(
                "MIN_BED",
                "Thinnest bed worth calling a bed (OPT_BEDS = AUTO)",
                PROJECT_DEPTH_UNIT_TOKEN,
                0.0,
                10000.0,
                &[("OPT_BEDS", "AUTO")],
                "docs/PRD_v2/20_envcorr-qc.md §5.3 frame parameters",
            ),
            // Generic statistical multiplier, like the Hampel K — round, and not a calibration.
            // Read only on the AUTO branch; with a default it is simply inert elsewhere.
            param(
                "SENS", "AUTO: how far off the bed's mean is a new bed, in noise units", "", 2.0,
                0.5, 20.0,
                "Two-noise-units convention for change detection (same family as the Hampel K in \
                 condition.rs), NOT a field calibration; ruled a shipping starting value by Jauhar \
                 adjudication DEC-077 (2026-08-19); docs/takeover/DECISIONS.md",
            ),
            opt_labelled(
                "OPT_BEDS",
                "What counts as one bed",
                "INTERVAL",
                &[
                    ("INTERVAL", "INTERVAL — equal slices of INTERVAL thickness"),
                    ("CLASS", "CLASS — each run of a constant value in the BEDS curve"),
                    ("ZONES", "ZONES — one value per marker interval"),
                    ("AUTO", "AUTO — boundaries found from the curve itself (inferred)"),
                ],
            ),
            opt_labelled(
                "OPT_STAT",
                "How a bed's value is taken",
                "MEAN",
                &[
                    ("MEAN", "MEAN — arithmetic; right for porosity and volume fractions"),
                    ("GEOMETRIC", "GEOMETRIC — the usual estimate for permeability"),
                    ("HARMONIC", "HARMONIC — permeability across layers in series"),
                    ("MEDIAN", "MEDIAN — the middle sample of the bed"),
                    ("MIN", "MIN — the lowest sample of the bed"),
                    ("MAX", "MAX — the highest sample of the bed"),
                    ("MODE", "MODE — the bed's commonest value; the only upscale for a class curve"),
                ],
            ),
            log_in("CURVE", "Curve to upscale", "", "PHIE", true),
            log_in("BEDS", "Class curve defining the beds (OPT_BEDS = CLASS)", "", "FACIES", false),
            opt_labelled(
                "OPT_FLAG",
                "Write the bed number each sample fell in",
                "YES",
                &[("YES", "YES — write the bed-index curve"), ("NO", "NO — the blocked curve only")],
            ),
            log_out_as("OUT_CURVE", "{CURVE}_BLK", "Blocked curve", ""),
            log_out_as("OUT_BED", "{OUT_CURVE}_BED", "Bed index", ""),
        ],
    }
}

pub fn block(ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    let vals = ctx.log("CURVE");
    let depth = ctx.log("DEPTH");
    let stat = ctx.o("OPT_STAT").to_string();
    // SB-MLA-055. A class code is a name written as a number; the mean of facies 1 and facies 4 is
    // 2.5, and MEDIAN here goes through the R-type-7 percentile, so an even bed of {1, 2} gives
    // 1.5. Refused by NAME with the fix stated, rather than quietly substituting MODE: a module
    // returns a vector and has no channel to report that it did something else.
    if ctx.input_is_class_curve("CURVE") && !matches!(stat.as_str(), "MODE" | "MIN" | "MAX") {
        return Err(format!(
            "{} holds class codes, and {stat} would average them into a value that is not any class \
             (the mean of facies 1 and facies 4 is 2.5). Set OPT_STAT to MODE - the bed's commonest \
             code, and the one upscale that carries a class whole.",
            ctx.in_curve("CURVE")
        ));
    }
    let beds = assign_beds(ctx, &vals, &depth)?;

    // Gather each bed's live samples. A bed key is an ordinal, so a BTreeMap keeps them in depth
    // order for free — which matters only for legibility here, but costs nothing.
    let mut buckets: HashMap<i64, Vec<f32>> = HashMap::new();
    for i in 0..ctx.n {
        if beds[i].is_finite() && vals[i].is_finite() {
            buckets.entry(beds[i] as i64).or_default().push(vals[i]);
        }
    }

    let mut answer: HashMap<i64, f32> = HashMap::new();
    for (key, mut v) in buckets {
        let value = match stat.as_str() {
            "MEDIAN" => {
                v.sort_by(|a, b| a.partial_cmp(b).expect("finite by construction"));
                crate::distribution::percentile(&v, 50.0)
            }
            "MIN" => v.iter().copied().fold(f32::INFINITY, f32::min),
            "MAX" => v.iter().copied().fold(f32::NEG_INFINITY, f32::max),
            // The only upscale a CLASS curve has (SB-MLA-055). The bed's commonest code wins it,
            // which is the same rule `reframe`'s MODE follows — one definition of "upscale a class
            // curve", not two. Ties go to the value seen first in depth order, deterministically.
            "MODE" => {
                let mut best = (v[0], 0usize);
                for x in v.iter() {
                    let c = v.iter().filter(|y| (**y - *x).abs() < 1e-6).count();
                    if c > best.1 {
                        best = (*x, c);
                    }
                }
                best.0
            }
            // Both flow means are undefined at or below zero — a log of a non-positive number,
            // and a division by it. A permeability of exactly 0 is a real reading (a seal), so it
            // is EXCLUDED from the mean and the bed is reported from what is left rather than the
            // whole bed collapsing to MISSING or to zero. A bed that is entirely non-positive has
            // no geometric or harmonic mean and is MISSING, which is the honest answer.
            "GEOMETRIC" => {
                let live: Vec<f64> = v.iter().map(|x| *x as f64).filter(|x| *x > 0.0).collect();
                if live.is_empty() {
                    MISSING
                } else {
                    (live.iter().map(|x| x.ln()).sum::<f64>() / live.len() as f64).exp() as f32
                }
            }
            "HARMONIC" => {
                let live: Vec<f64> = v.iter().map(|x| *x as f64).filter(|x| *x > 0.0).collect();
                if live.is_empty() {
                    MISSING
                } else {
                    (live.len() as f64 / live.iter().map(|x| 1.0 / x).sum::<f64>()) as f32
                }
            }
            _ => (v.iter().map(|x| *x as f64).sum::<f64>() / v.len() as f64) as f32,
        };
        answer.insert(key, value);
    }

    // A sample in no bed, or in a bed with nothing live in it, stays MISSING — never filled from
    // the bed above. Blocking averages what was measured; it does not extend it.
    let out: Vec<f32> = (0..ctx.n)
        .map(|i| {
            if beds[i].is_finite() {
                answer.get(&(beds[i] as i64)).copied().unwrap_or(MISSING)
            } else {
                MISSING
            }
        })
        .collect();

    let mut res: ModuleOutputs = HashMap::from([("OUT_CURVE".to_string(), out)]);
    if yes(ctx, "OPT_FLAG", true) {
        res.insert("OUT_BED".to_string(), beds);
    }
    Ok(res)
}

// ---------------------------------------------------------------------------
// BED DETECT
// ---------------------------------------------------------------------------

pub fn bed_detect_spec() -> ModuleSpec {
    ModuleSpec {
        name: "bed_detect".into(),
        title: "Bed Detect".into(),
        category: "Frame".into(),
        doc: "Writes the bed number each sample falls in, found from the curve's own steps — the \
              same segmentation Block's AUTO mode uses, exposed on its own so the beds can be \
              LOOKED AT on a log before anything is averaged over them.\n\n\
              That order matters: over-segmentation is what a step-finder gets wrong when it gets \
              anything wrong, and a blocked curve computed from beds nobody checked looks \
              perfectly reasonable. Put the bed curve in a track as class blocks, judge it against \
              the log, then run Block with OPT_BEDS = CLASS pointing at it.\n\n\
              A sample opens a new bed when it sits further from the running bed mean than SENS \
              times the curve's own noise AND the bed already spans MIN_BED. The noise is measured \
              from the curve's own sample-to-sample differences, so it is the curve's noise rather \
              than how much it varies across the well.\n\n\
              MIN_BED has no default: the thinnest thing worth calling a bed is a property of the \
              tool and the rock."
            .into(),
        args: vec![
            param_open(
                "MIN_BED",
                "Thinnest bed worth calling a bed",
                PROJECT_DEPTH_UNIT_TOKEN,
                0.0,
                10000.0,
                true,
            ),
            param(
                "SENS",
                "How far off the bed's mean is a new bed, in noise units",
                "",
                2.0,
                0.5,
                20.0,
                "Two-noise-units convention for change detection (same family as the Hampel K in \
                 condition.rs), NOT a field calibration; ruled a shipping starting value by Jauhar \
                 adjudication DEC-077 (2026-08-19); docs/takeover/DECISIONS.md",
            ),
            log_in("CURVE", "Curve to segment", "", "GR", true),
            log_out_as("OUT_CURVE", "{CURVE}_BED", "Bed index", ""),
        ],
    }
}

pub fn bed_detect(ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    let min_bed = constant(ctx, "MIN_BED");
    if !(min_bed > 0.0) {
        return Err("MIN_BED must be set — the thinnest thing worth calling a bed is a property \
                    of the tool and the rock, and no one value is right twice."
            .into());
    }
    let vals = ctx.log("CURVE");
    let depth = ctx.log("DEPTH");
    let beds = detect_beds(&vals, &depth, constant(ctx, "SENS"), min_bed);
    Ok(HashMap::from([("OUT_CURVE".to_string(), beds)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::DepthUnit;

    fn ctx_for(
        depth: &[f32],
        curve: &[f32],
        params: &[(&str, f64)],
        opts: &[(&str, &str)],
    ) -> ModuleContext {
        let n = depth.len();
        let mut logs = HashMap::new();
        logs.insert("DEPTH".to_string(), depth.to_vec());
        logs.insert("CURVE".to_string(), curve.to_vec());
        let mut ps = HashMap::new();
        for (k, v) in params {
            ps.insert(k.to_string(), vec![*v; n]);
        }
        let mut os: HashMap<String, String> = HashMap::new();
        os.insert("__IN_CURVE".to_string(), "PERM".to_string());
        for (k, v) in opts {
            os.insert(k.to_string(), v.to_string());
        }
        ModuleContext { n, logs, params: ps, opts: os, depth_unit: DepthUnit::Metres }
    }

    /// SB-MLA-055 in `block`, pinned from both sides — and the MODE half is not a nicety. The
    /// coreimage pane already tells the user in so many words to *"use Frame > Block with OPT_STAT
    /// = MODE, the one upscale that carries a class code whole"*, and `block` had no MODE arm, so
    /// following the application's own advice fell through to the `_` case and took the MEAN of the
    /// codes. It computed, it plotted, and nothing said otherwise.
    ///
    /// So: MODE exists and returns a real code, and the averaging statistics are refused BY NAME on
    /// a declared class curve. Either half alone would pass a broken implementation — a refusal
    /// with no MODE to switch to is a dead end, and a MODE nobody is steered to leaves MEAN as the
    /// default on a facies curve.
    #[test]
    fn a_class_curve_is_blocked_by_its_commonest_code_and_refuses_every_average() {
        let depth = regular(9, 0.5, 1000.0);
        // One INTERVAL block, wide enough to hold the lot: facies 3 is the commonest code in it,
        // with 1 and 4 also present, and the arithmetic mean of the nine is 2.78 — a code that does
        // not exist.
        let facies = vec![3.0f32, 3.0, 1.0, 3.0, 4.0, 3.0, 1.0, 3.0, 4.0];
        let one_bed: &[(&str, f64)] = &[("INTERVAL", 100.0)];
        // `__IN_CURVE` is how a run states which mnemonic the CURVE arg resolved to, and
        // `__CLASS_CURVES` is the run's declared class list. Both are needed: the rule fires on the
        // INTERSECTION, which is what lets a project hold a FACIES curve and a PHIE curve at once.
        let with = |stat: &'static str| -> Vec<(&'static str, &'static str)> {
            vec![
                ("__IN_CURVE", "FACIES"),
                ("__CLASS_CURVES", "FACIES"),
                ("OPT_BEDS", "INTERVAL"),
                ("OPT_STAT", stat),
            ]
        };

        let out = block(&ctx_for(&depth, &facies, one_bed, &with("MODE")))
            .expect("MODE is a valid upscale for a class curve");
        let v = &out["OUT_CURVE"];
        assert!(
            v.iter().filter(|x| x.is_finite()).all(|x| (*x - 3.0).abs() < 1e-6),
            "the bed's commonest code wins it, got {v:?}",
        );

        // Every statistic that can invent a value is refused, and the message names the curve and
        // the fix. MEDIAN is in the list because it goes through the R-type-7 percentile, so an
        // even-count bed of {1, 2} returns 1.5.
        for stat in ["MEAN", "GEOMETRIC", "HARMONIC", "MEDIAN"] {
            let e = block(&ctx_for(&depth, &facies, one_bed, &with(stat)))
                .expect_err("an averaging statistic over class codes must be refused");
            assert!(e.contains("FACIES"), "{stat}: names the curve refused: {e}");
            assert!(e.contains("MODE"), "{stat}: names the statistic to use instead: {e}");
        }
        // MIN and MAX land on a real sample, so they are allowed — a class curve has an order even
        // where it has no arithmetic.
        for stat in ["MIN", "MAX"] {
            assert!(
                block(&ctx_for(&depth, &facies, one_bed, &with(stat))).is_ok(),
                "{stat} returns a real sample and is not a refusal",
            );
        }

        // The other side, and the half that stops this becoming a heuristic: an UNDECLARED curve is
        // untouched however class-like its values look. A caliper logged in whole inches still
        // averages, because nothing declared it a class.
        let plain: &[(&str, &str)] = &[("OPT_BEDS", "INTERVAL"), ("OPT_STAT", "MEAN")];
        let out = block(&ctx_for(&depth, &facies, one_bed, plain))
            .expect("nothing declared this a class curve, so MEAN is the user's to choose");
        let v = out["OUT_CURVE"].iter().copied().find(|x| x.is_finite()).unwrap();
        assert!((v - 2.777_8).abs() < 0.01, "an undeclared curve still averages: {v}");

        // And a DECLARED name that is not the one being blocked leaves this run alone.
        let other: &[(&str, &str)] =
            &[("OPT_BEDS", "INTERVAL"), ("OPT_STAT", "MEAN"), ("__IN_CURVE", "PHIE"), ("__CLASS_CURVES", "FACIES")];
        assert!(
            block(&ctx_for(&depth, &facies, one_bed, other)).is_ok(),
            "the rule fires on the curve declared, not on the presence of a declaration",
        );
    }

    fn regular(n: usize, step: f32, start: f32) -> Vec<f32> {
        (0..n).map(|i| start + i as f32 * step).collect()
    }

    /// **Upscaling permeability arithmetically overstates flow, always in the same direction.**
    ///
    /// Half a bed at 1000 mD and half at 0.01 mD: 500 arithmetically, ~0.3 geometrically, ~0.02
    /// harmonically. The three are the parallel, random and series flow means, and the arithmetic
    /// one is the largest of the three for any spread at all — so an arithmetic upscale of a
    /// laminated interval hands a simulator a permeability the rock does not have and nothing
    /// downstream reads as wrong.
    #[test]
    fn upscaling_permeability_arithmetically_overstates_flow() {
        let n = 20;
        let depth = regular(n, 0.1, 1000.0);
        let mut perm = vec![1000.0f32; n];
        for p in perm.iter_mut().skip(10) {
            *p = 0.01;
        }
        let block_with = |stat: &str| -> f32 {
            let out = block(&ctx_for(
                &depth,
                &perm,
                &[("INTERVAL", 100.0)], // one block over the whole run
                &[("OPT_BEDS", "INTERVAL"), ("OPT_STAT", stat)],
            ))
            .expect("run");
            out["OUT_CURVE"][0]
        };
        let (a, g, h) = (block_with("MEAN"), block_with("GEOMETRIC"), block_with("HARMONIC"));
        assert!((a - 500.0).abs() < 1.0, "arithmetic mean of 1000 and 0.01 is 500, got {a}");
        assert!((g - 3.16).abs() < 0.2, "geometric mean is sqrt(1000 x 0.01) ~ 3.16, got {g}");
        assert!((h - 0.02).abs() < 0.001, "harmonic mean is ~0.02, got {h}");
        assert!(a > g && g > h, "the three flow means must order arithmetic > geometric > harmonic");
        assert!(
            a / g > 100.0,
            "and on a laminated interval they differ by orders of magnitude, not percent — \
             a/g was only {}",
            a / g
        );
    }

    /// A zero permeability is a real reading (a seal), and it has no logarithm and no reciprocal.
    /// It is excluded and the bed reported from what is left, rather than the whole bed collapsing
    /// — which would delete the seal from the model instead of describing it.
    #[test]
    fn a_zero_sample_does_not_swallow_a_geometric_bed() {
        let n = 10;
        let depth = regular(n, 0.1, 1000.0);
        let mut perm = vec![100.0f32; n];
        perm[3] = 0.0;
        let out = block(&ctx_for(
            &depth,
            &perm,
            &[("INTERVAL", 100.0)],
            &[("OPT_BEDS", "INTERVAL"), ("OPT_STAT", "GEOMETRIC")],
        ))
        .expect("run");
        assert!((out["OUT_CURVE"][0] - 100.0).abs() < 1e-3, "got {}", out["OUT_CURVE"][0]);

        // A bed with nothing positive in it has no geometric mean, and says so rather than
        // returning zero — which would read as a measurement.
        let zeros = vec![0.0f32; n];
        let out = block(&ctx_for(
            &depth,
            &zeros,
            &[("INTERVAL", 100.0)],
            &[("OPT_BEDS", "INTERVAL"), ("OPT_STAT", "GEOMETRIC")],
        ))
        .expect("run");
        assert!(out["OUT_CURVE"][0].is_nan());
    }

    /// **A block interval is a thickness**, so the same setting covers the same rock at any
    /// sampling — the Condition family's rule, and the reason the interval is not a sample count.
    #[test]
    fn a_block_interval_covers_the_same_rock_at_any_sampling() {
        for step in [0.1f32, 0.05] {
            let n = (2.0 / step) as usize;
            let depth = regular(n, step, 1000.0);
            let curve: Vec<f32> = depth.iter().map(|d| (d - 1000.0) * 10.0).collect();
            let out = block(&ctx_for(
                &depth,
                &curve,
                &[("INTERVAL", 0.5)],
                &[("OPT_BEDS", "INTERVAL"), ("OPT_STAT", "MEAN")],
            ))
            .expect("run");
            let beds = &out["OUT_BED"];
            // 2 m of well at 0.5 m blocks = 4 blocks, whatever the sample spacing.
            let distinct: std::collections::BTreeSet<i64> =
                beds.iter().filter(|b| b.is_finite()).map(|b| *b as i64).collect();
            assert_eq!(distinct.len(), 4, "step {step}: 2 m in 0.5 m blocks is 4 blocks");
            // Every sample of a block carries that block's own single value.
            let blocked = &out["OUT_CURVE"];
            for i in 1..n {
                if beds[i] == beds[i - 1] {
                    assert_eq!(blocked[i], blocked[i - 1], "step {step}: sample {i} broke its block");
                }
            }
        }
    }

    /// CLASS beds follow the ROCK: each run of a constant class value is one bed, and a gap in
    /// the class curve ends the bed rather than being spanned — two intervals of the same facies
    /// either side of unlogged section are not one bed.
    #[test]
    fn class_beds_follow_the_rock_and_a_gap_ends_a_bed() {
        let n = 12;
        let depth = regular(n, 0.1, 1000.0);
        let curve: Vec<f32> = (0..n).map(|i| i as f32).collect();
        let mut cls = vec![1.0f32; n];
        for c in cls.iter_mut().take(8).skip(4) {
            *c = 2.0;
        }
        cls[6] = f32::NAN; // a hole inside the facies-2 run
        let mut ctx = ctx_for(&depth, &curve, &[], &[("OPT_BEDS", "CLASS"), ("OPT_STAT", "MEAN")]);
        ctx.logs.insert("BEDS".to_string(), cls);
        let out = block(&ctx).expect("run");
        let beds = &out["OUT_BED"];
        assert_eq!(beds[0], 0.0, "the first facies-1 run is bed 0");
        assert_eq!(beds[4], 1.0, "the facies-2 run opens bed 1");
        assert!(beds[6].is_nan(), "the hole belongs to no bed");
        assert_eq!(beds[7], 2.0, "and the facies-2 run AFTER the hole is a new bed, not bed 1");
        assert!(out["OUT_CURVE"][6].is_nan(), "a sample in no bed stays MISSING");
    }

    /// The AUTO segmentation finds a real contact and refuses to cut a bed thinner than MIN_BED,
    /// which is what stops a noisy interval becoming a bed per sample.
    #[test]
    fn bed_detection_finds_the_contact_and_honours_the_minimum_thickness() {
        let n = 60;
        let depth = regular(n, 0.1, 1000.0);
        // Two clean units with a little noise, 3 m apart in value.
        let mut curve = vec![0.0f32; n];
        for (i, c) in curve.iter_mut().enumerate() {
            let base = if i < 30 { 50.0 } else { 120.0 };
            *c = base + if i % 2 == 0 { 0.4 } else { -0.4 };
        }
        let out = bed_detect(&ctx_for(&depth, &curve, &[("MIN_BED", 0.5), ("SENS", 4.0)], &[]))
            .expect("run");
        let beds = &out["OUT_CURVE"];
        assert_eq!(beds[0], 0.0);
        assert!(beds[35] > 0.0, "the 70-unit contact must open a new bed");
        let count = beds.iter().filter(|b| b.is_finite()).map(|b| *b as i64).max().unwrap() + 1;
        assert!(
            (2..=3).contains(&count),
            "two clean units should give about two beds, not {count} — the noise is 0.8 wide and \
             the contact is 70, so anything more is the segmenter chasing noise"
        );

        // A minimum thickness wider than the whole well can only ever give one bed.
        let out = bed_detect(&ctx_for(&depth, &curve, &[("MIN_BED", 100.0), ("SENS", 4.0)], &[]))
            .expect("run");
        let beds = &out["OUT_CURVE"];
        assert!(beds.iter().all(|b| *b == 0.0), "MIN_BED wider than the well is one bed");
    }

    /// The refusals: a mode needs the thing it is defined by. (Shadowing is checked once for
    /// every module by `workflow::resolve_output_names`, not here — see
    /// `an_output_name_that_would_be_shadowed_is_refused_before_a_single_well_runs`.)
    #[test]
    fn each_bed_mode_refuses_without_what_defines_it() {
        let depth = regular(10, 0.1, 1000.0);
        let curve = vec![1.0f32; 10];
        assert!(block(&ctx_for(&depth, &curve, &[], &[("OPT_BEDS", "INTERVAL")])).is_err());
        assert!(block(&ctx_for(&depth, &curve, &[], &[("OPT_BEDS", "AUTO")])).is_err());
        assert!(block(&ctx_for(&depth, &curve, &[], &[("OPT_BEDS", "CLASS")])).is_err());
        assert!(block(&ctx_for(&depth, &curve, &[], &[("OPT_BEDS", "ZONES")])).is_err());
        assert!(bed_detect(&ctx_for(&depth, &curve, &[], &[])).is_err());
    }
}
