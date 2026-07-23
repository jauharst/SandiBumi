//! Free-form net-flag from a crossplot polygon (P9-C follow-on). Given a polygon the user drew
//! on a crossplot (X vs Y curve, vertices captured in DATA space) plus the axes' log flags, this
//! marks every sample inside the polygon as net reservoir: a discrete 0/1 curve (NaN where a
//! sample can't be evaluated), persisted as a computed curve like any other module output.
//!
//! The point-in-polygon test runs in the SAME transformed plane the polygon was drawn in — log10
//! on a log axis — so "inside the drawn polygon" is exact for log scales (straight screen edges
//! are straight edges in that plane) and matches the crossplot's on-screen cutoff count. A sample
//! is written 1.0 (inside) / 0.0 (outside) / NaN (either input NaN, or ≤0 on a log axis — the same
//! samples the crossplot excludes from its count), over the crossplot's current depth window.

use crate::equations;
use duckdb::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct NetFlagSpec {
    pub well_id: String,
    pub x_curve: String,
    pub y_curve: String,
    #[serde(default)]
    pub x_log: bool,
    #[serde(default)]
    pub y_log: bool,
    /// Polygon vertices in DATA space, axis order (x, y); ≥3 required. The ring is implicitly
    /// closed (last vertex connects back to the first).
    pub polygon: Vec<(f32, f32)>,
    /// Output curve name (e.g. "NET_FLAG"); written to the computed-curve store (upper-cased).
    pub output_curve: String,
    /// Restrict evaluation to this depth window (the crossplot's selected zone); None = whole well.
    #[serde(default)]
    pub depth_top: Option<f32>,
    #[serde(default)]
    pub depth_bottom: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetFlagResult {
    pub output_curve: String,
    /// Samples flagged net (value 1.0).
    pub inside: usize,
    /// Samples actually evaluated (finite, log-valid) — the 0/1 denominator; NaN samples excluded.
    pub evaluated: usize,
    /// Samples written to the curve (the depth-window length, including NaN flags).
    pub written: usize,
}

/// Even–odd ray-casting point-in-polygon in an already-transformed plane. `poly` is treated as a
/// closed ring (last vertex connects to the first). Boundary points may fall either way (floating
/// comparisons) — acceptable for a net cutoff. Returns false for a degenerate ring (< 3 vertices).
pub(crate) fn point_in_polygon(px: f64, py: f64, poly: &[(f64, f64)]) -> bool {
    let n = poly.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[j];
        // The horizontal ray at py crosses edge (j→i) only when the edge straddles py; the guard
        // also guarantees yj != yi, so the division below is safe.
        if (yi > py) != (yj > py) {
            let x_cross = xi + (py - yi) / (yj - yi) * (xj - xi);
            if px < x_cross {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

/// Maps a value into the axis's drawing plane (log10 on a log axis). None when the value cannot be
/// placed there — NaN, or ≤ 0 on a log axis.
fn tf(v: f32, log: bool) -> Option<f64> {
    if v.is_nan() {
        return None;
    }
    let v = v as f64;
    if log {
        if v <= 0.0 { None } else { Some(v.log10()) }
    } else {
        Some(v)
    }
}

pub fn run_net_flag(conn: &Connection, spec: &NetFlagSpec) -> Result<NetFlagResult, String> {
    if spec.polygon.len() < 3 {
        return Err("a net polygon needs at least 3 points".into());
    }
    let out_name = spec.output_curve.trim().to_uppercase();
    if out_name.is_empty() {
        return Err("net-flag curve needs a name".into());
    }
    let x_key = spec.x_curve.trim().to_uppercase();
    let y_key = spec.y_curve.trim().to_uppercase();
    if x_key == y_key {
        return Err("net polygon needs two different curves for X and Y".into());
    }

    let names = vec![spec.x_curve.clone(), spec.y_curve.clone()];
    let (depth, columns) =
        equations::fetch_curve_frame(conn, &spec.well_id, &names).map_err(|e| e.to_string())?;
    if depth.is_empty() {
        return Err("no curve data for this well".into());
    }
    let xs = columns
        .get(&x_key)
        .ok_or_else(|| format!("curve '{}' has no data in this well", spec.x_curve))?;
    let ys = columns
        .get(&y_key)
        .ok_or_else(|| format!("curve '{}' has no data in this well", spec.y_curve))?;

    // Transform the polygon into the axes' drawing plane once (log10 where the axis is log). A
    // vertex that can't be transformed (≤ 0 on a log axis) would corrupt the ring, so bail early.
    let mut poly: Vec<(f64, f64)> = Vec::with_capacity(spec.polygon.len());
    for &(vx, vy) in &spec.polygon {
        match (tf(vx, spec.x_log), tf(vy, spec.y_log)) {
            (Some(px), Some(py)) => poly.push((px, py)),
            _ => return Err("a polygon vertex is off the log axis (≤ 0) — redraw it in range".into()),
        }
    }

    let (top, bot) = match (spec.depth_top, spec.depth_bottom) {
        (Some(a), Some(b)) if a > b => (b, a),
        _ => (
            spec.depth_top.unwrap_or(f32::NEG_INFINITY),
            spec.depth_bottom.unwrap_or(f32::INFINITY),
        ),
    };

    let n = depth.len().min(xs.len()).min(ys.len());
    let mut w_depth: Vec<f32> = Vec::new();
    let mut flags: Vec<f32> = Vec::new();
    let mut inside = 0usize;
    let mut evaluated = 0usize;
    for i in 0..n {
        let d = depth[i];
        if d < top || d > bot {
            continue;
        }
        w_depth.push(d);
        let flag = match (tf(xs[i], spec.x_log), tf(ys[i], spec.y_log)) {
            (Some(px), Some(py)) => {
                evaluated += 1;
                if point_in_polygon(px, py, &poly) {
                    inside += 1;
                    1.0
                } else {
                    0.0
                }
            }
            // NaN input, or ≤ 0 on a log axis — undefined here, mirroring the crossplot's exclusion.
            _ => f32::NAN,
        };
        flags.push(flag);
    }
    if w_depth.is_empty() {
        return Err("no samples in the selected depth window".into());
    }

    equations::write_computed_curve(conn, &spec.well_id, &w_depth, &out_name, &flags)
        .map_err(|e| e.to_string())?;
    Ok(NetFlagResult { output_curve: out_name, inside, evaluated, written: w_depth.len() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use uuid::Uuid;

    #[test]
    fn point_in_polygon_square_and_triangle() {
        // Unit square [0,1]².
        let sq = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        assert!(point_in_polygon(0.5, 0.5, &sq), "centre inside");
        assert!(!point_in_polygon(1.5, 0.5, &sq), "right of the square");
        assert!(!point_in_polygon(-0.1, 0.5, &sq), "left of the square");
        assert!(!point_in_polygon(0.5, 1.5, &sq), "above the square");
        // A square with a triangular notch bitten out of the right edge (apex at (2,2)): the body
        // reads inside, the notch bite reads OUTSIDE — the concavity check.
        let notched = [(0.0, 0.0), (4.0, 0.0), (2.0, 2.0), (4.0, 4.0), (0.0, 4.0)];
        assert!(point_in_polygon(1.0, 2.0, &notched), "left of the notch apex → inside");
        assert!(!point_in_polygon(3.0, 2.0, &notched), "inside the notch bite → outside");
        assert!(!point_in_polygon(5.0, 2.0, &notched), "right of the polygon → outside");
        // Degenerate rings are never "inside".
        assert!(!point_in_polygon(0.0, 0.0, &[(0.0, 0.0), (1.0, 1.0)]));
    }

    fn seed(conn: &Connection) -> String {
        db::create_schema(conn).unwrap();
        let id = Uuid::new_v4();
        db::insert_well(conn, id, "NETFLAG-1", Some("Synthetic"), None, None).unwrap();
        // NPHI ramps 0.00→0.30, RHOB ramps 2.70→2.40 across 31 samples; one NaN pair mid-log.
        let n = 31usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let mut nphi: Vec<f32> = (0..n).map(|i| i as f32 * 0.01).collect();
        let mut rhob: Vec<f32> = (0..n).map(|i| 2.70 - i as f32 * 0.01).collect();
        nphi[5] = f32::NAN;
        rhob[5] = f32::NAN;
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(conn, id, depth, vec![50.0; n], nan.clone(), nphi, rhob, nan.clone(), nan)
            .unwrap();
        id.to_string()
    }

    fn read_curve(conn: &Connection, well: &str, name: &str) -> Vec<(f32, f32)> {
        let mut stmt = conn
            .prepare("SELECT depth, value FROM computed_curves WHERE well_id = ?1 AND curve_name = ?2 ORDER BY depth")
            .unwrap();
        stmt.query_map(duckdb::params![well, name], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    }

    #[test]
    fn writes_flag_curve_over_the_cloud() {
        let conn = Connection::open_in_memory().unwrap();
        let w = seed(&conn);
        // A box around the low-NPHI / high-RHOB corner (the first several samples).
        let spec = NetFlagSpec {
            well_id: w.clone(),
            x_curve: "NPHI".into(),
            y_curve: "RHOB".into(),
            x_log: false,
            y_log: false,
            polygon: vec![(-0.01, 2.55), (0.055, 2.55), (0.055, 2.75), (-0.01, 2.75)],
            output_curve: "net_flag".into(),
            depth_top: None,
            depth_bottom: None,
        };
        let res = run_net_flag(&conn, &spec).unwrap();
        assert_eq!(res.output_curve, "NET_FLAG", "name upper-cased");
        // NPHI 0.00..0.05 with RHOB 2.70..2.65 → samples i=0..5, but i=5 is NaN → 5 inside.
        assert_eq!(res.inside, 5, "five samples inside the corner box");
        assert_eq!(res.written, 31, "one flag per grid sample in the window");
        assert_eq!(res.evaluated, 30, "the NaN pair is not evaluated");

        let curve = read_curve(&conn, &w, "NET_FLAG");
        assert_eq!(curve.len(), 31);
        // First five are net, the NaN sample is NaN, later samples are 0.
        for (i, (_d, v)) in curve.iter().enumerate() {
            if i < 5 {
                assert_eq!(*v, 1.0, "sample {i} net");
            } else if i == 5 {
                assert!(v.is_nan(), "sample 5 (NaN input) → NaN flag");
            } else {
                assert_eq!(*v, 0.0, "sample {i} not net");
            }
        }
    }

    #[test]
    fn depth_window_restricts_written_samples() {
        let conn = Connection::open_in_memory().unwrap();
        let w = seed(&conn);
        let spec = NetFlagSpec {
            well_id: w.clone(),
            x_curve: "NPHI".into(),
            y_curve: "RHOB".into(),
            x_log: false,
            y_log: false,
            polygon: vec![(-1.0, -1.0), (1.0, -1.0), (1.0, 3.0), (-1.0, 3.0)], // everything inside
            output_curve: "NET".into(),
            depth_top: Some(1010.0),
            depth_bottom: Some(1019.0),
        };
        let res = run_net_flag(&conn, &spec).unwrap();
        assert_eq!(res.written, 10, "only the 1010–1019 m window is written");
        let curve = read_curve(&conn, &w, "NET");
        assert!(curve.iter().all(|(d, _)| *d >= 1010.0 && *d <= 1019.0));
    }

    #[test]
    fn log_axis_evaluates_in_log10_space() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = Uuid::new_v4();
        db::insert_well(&conn, id, "NETFLAG-LOG", None, None, None).unwrap();
        // RES ramps by decades on the log X axis; NPHI flat on the linear Y axis.
        let depth: Vec<f32> = vec![1000.0, 1001.0, 1002.0, 1003.0];
        let res: Vec<f32> = vec![1.0, 10.0, 100.0, 1000.0];
        let nphi: Vec<f32> = vec![0.10, 0.10, 0.10, 0.10];
        let nan = vec![f32::NAN; 4];
        db::insert_standard_curves(&conn, id, depth, vec![50.0; 4], res, nphi, nan.clone(), nan.clone(), nan)
            .unwrap();
        let w = id.to_string();
        // A data-space box around the 10..100 decade. In log10 space x spans log10(5)..log10(200)
        // = 0.70..2.30, so only res=10 and res=100 fall inside — NOT res=1 (log 0) or 1000 (log 3).
        let spec = NetFlagSpec {
            well_id: w,
            x_curve: "RES_DEEP".into(),
            y_curve: "NPHI".into(),
            x_log: true,
            y_log: false,
            polygon: vec![(5.0, 0.05), (200.0, 0.05), (200.0, 0.15), (5.0, 0.15)],
            output_curve: "NETL".into(),
            depth_top: None,
            depth_bottom: None,
        };
        let out = run_net_flag(&conn, &spec).unwrap();
        assert_eq!(out.inside, 2, "only the 10 and 100 decades are inside the log-space box");
        assert_eq!(out.evaluated, 4);
        // A polygon vertex at 0 can't be placed on a log axis → rejected.
        let bad = NetFlagSpec { polygon: vec![(0.0, 0.05), (200.0, 0.05), (200.0, 0.15)], ..spec };
        assert!(run_net_flag(&conn, &bad).unwrap_err().contains("off the log axis"));
    }

    #[test]
    fn errors_on_too_few_points_and_same_axes() {
        let conn = Connection::open_in_memory().unwrap();
        let w = seed(&conn);
        let base = NetFlagSpec {
            well_id: w,
            x_curve: "NPHI".into(),
            y_curve: "RHOB".into(),
            x_log: false,
            y_log: false,
            polygon: vec![(0.0, 0.0), (1.0, 1.0)],
            output_curve: "NET".into(),
            depth_top: None,
            depth_bottom: None,
        };
        assert!(run_net_flag(&conn, &base).unwrap_err().contains("at least 3"));
        let same = NetFlagSpec {
            y_curve: "NPHI".into(),
            polygon: vec![(0.0, 0.0), (1.0, 0.0), (0.5, 1.0)],
            ..base
        };
        assert!(run_net_flag(&conn, &same).unwrap_err().contains("different curves"));
    }
}
