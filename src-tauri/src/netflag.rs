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

// `deny_unknown_fields` so a TypeScript field this struct does not know fails loudly instead of
// being silently dropped — the silent direction of the camelCase break that made this whole
// feature a no-op. There is no `rename_all` here on purpose: struct DTOs cross the wire in
// snake_case (Tauri camel-cases only the top-level command argument key, not nested fields).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
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

    /// Guards the wire itself, not the maths. The flag-polygon feature shipped broken because
    /// `ipc.ts` declared this struct in camelCase while serde expects the snake_case field names
    /// below, and nothing ever exercised the boundary — the original increment verified a
    /// frontend twin-count that never crossed into Rust, so `run_net_flag` could not deserialize
    /// a single request. The JSON literal here is what `crossplotPanel.ts` actually sends; if the
    /// two sides drift again, this fails instead of the feature silently doing nothing.
    #[test]
    fn spec_deserializes_from_the_exact_json_the_frontend_sends() {
        let sent = r#"{
            "well_id": "W-1",
            "x_curve": "NPHI",
            "y_curve": "RHOB",
            "x_log": false,
            "y_log": true,
            "polygon": [[0.1, 2.4], [0.3, 2.4], [0.3, 2.7]],
            "output_curve": "NET_FLAG",
            "depth_top": 2000.0,
            "depth_bottom": 2050.0
        }"#;
        let spec: NetFlagSpec = serde_json::from_str(sent).expect("frontend JSON must deserialize");
        assert_eq!(spec.well_id, "W-1");
        assert_eq!(spec.x_curve, "NPHI");
        assert_eq!(spec.y_curve, "RHOB");
        assert!(!spec.x_log && spec.y_log, "both axis flags survive the wire independently");
        assert_eq!(spec.polygon.len(), 3);
        assert_eq!(spec.polygon[2], (0.3, 2.7));
        assert_eq!(spec.output_curve, "NET_FLAG");
        assert_eq!(spec.depth_top, Some(2000.0));
        assert_eq!(spec.depth_bottom, Some(2050.0));

        // camelCase is what shipped and what must NOT be accepted: `well_id` carries no
        // serde(default), so the whole request fails rather than silently defaulting.
        let camel = r#"{"wellId":"W-1","xCurve":"NPHI","yCurve":"RHOB","polygon":[[0.0,0.0]],
                        "outputCurve":"NET_FLAG"}"#;
        assert!(
            serde_json::from_str::<NetFlagSpec>(camel).is_err(),
            "camelCase must be rejected, not half-parsed into defaults"
        );

        // A zone-less run omits the depth window entirely (serde(default) → None), which is the
        // whole-well path; it must still parse.
        let no_window = r#"{"well_id":"W-1","x_curve":"NPHI","y_curve":"RHOB",
                            "polygon":[[0.0,0.0],[1.0,0.0],[1.0,1.0]],"output_curve":"NF"}"#;
        let w: NetFlagSpec = serde_json::from_str(no_window).expect("whole-well request parses");
        assert_eq!((w.depth_top, w.depth_bottom), (None, None));
        assert!(!w.x_log && !w.y_log, "omitted axis flags default to linear");
    }

    /// The mirror of the above: the result must reach the status line under the names `ipc.ts`
    /// reads. `res.outputCurve` rendered "undefined" for the same reason.
    #[test]
    fn result_serializes_under_the_names_the_frontend_reads() {
        let json = serde_json::to_value(NetFlagResult {
            output_curve: "NET_FLAG".into(),
            inside: 7,
            evaluated: 10,
            written: 12,
        })
        .expect("result serializes");
        let obj = json.as_object().expect("an object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, RESULT_FIELDS, "the serialized keys ARE the contract");
        assert_eq!(obj["output_curve"], "NET_FLAG");
    }

    /// The canonical wire contract, stated once. Both sides are asserted against it below, so a
    /// rename on either side fails a test instead of silently disabling the feature.
    const SPEC_FIELDS: [&str; 9] = [
        "depth_bottom", "depth_top", "output_curve", "polygon",
        "well_id", "x_curve", "x_log", "y_curve", "y_log",
    ];
    const RESULT_FIELDS: [&str; 4] = ["evaluated", "inside", "output_curve", "written"];

    /// Ask serde what `NetFlagSpec` will actually accept off the wire, rather than trusting the
    /// hand-written list above. `deny_unknown_fields` rejects a probe key with "unknown field
    /// `__probe__`, expected one of `well_id`, `x_curve`, …", which enumerates every field the
    /// struct really binds. Without this the contract was only ever checked against TypeScript,
    /// so a field added to the Rust struct with `#[serde(default)]` — the one shape that
    /// deserializes happily forever — could sit there permanently unknown to `ipc.ts`.
    fn serde_spec_fields() -> Vec<String> {
        let err = serde_json::from_str::<NetFlagSpec>(r#"{"__probe__":0}"#)
            .expect_err("deny_unknown_fields must reject an unknown key");
        let msg = err.to_string();
        let at = msg
            .find("expected one of ")
            .unwrap_or_else(|| panic!("serde's unknown-field message changed shape: {msg}"));
        // …`a`, `b`, `c` at line 1 column N → split on the backticks and keep the odd elements.
        let mut out: Vec<String> =
            msg[at..].split('`').skip(1).step_by(2).map(str::to_string).collect();
        assert!(!out.is_empty(), "no field names parsed out of: {msg}");
        out.sort_unstable();
        out
    }

    /// Pull the field names out of an interface in the real `src/ipc.ts`.
    fn ts_interface_fields(src: &str, iface: &str) -> Vec<String> {
        let head = format!("export interface {iface} {{");
        let at = src
            .find(&head)
            .unwrap_or_else(|| panic!("`{iface}` is no longer declared in ipc.ts"));
        let body = &src[at + head.len()..];
        let end = body.find("\n}").expect("unterminated interface in ipc.ts");
        let mut out: Vec<String> = body[..end]
            .lines()
            .map(str::trim)
            .filter(|l| !(l.is_empty() || l.starts_with("//") || l.starts_with('*') || l.starts_with("/*")))
            .filter_map(|l| l.split(':').next().map(|n| n.trim().trim_end_matches('?').to_string()))
            .filter(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
            .collect();
        out.sort_unstable();
        out
    }

    /// Cross-LANGUAGE guard. The serde tests above pin the Rust side, but they cannot see
    /// `ipc.ts` — and the defect was precisely that the two sides disagreed while each was
    /// internally consistent. This reads the actual frontend source and compares the declared
    /// field names against the same contract serde is held to.
    #[test]
    fn ipc_ts_declares_the_same_wire_names_as_the_rust_structs() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/ipc.ts");
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));

        // Rust side first: the contract must still describe the struct. Checking only TypeScript
        // against it would let a field added on the Rust side drift out of the comparison
        // entirely, taking the ipc.ts check down to a subset of the real wire.
        assert_eq!(
            serde_spec_fields(),
            SPEC_FIELDS,
            "NetFlagSpec's serde fields drifted from the stated contract — update SPEC_FIELDS and \
             declare the field in ipc.ts, or the frontend will never send it"
        );

        assert_eq!(
            ts_interface_fields(&src, "NetFlagSpec"),
            SPEC_FIELDS,
            "ipc.ts NetFlagSpec drifted from the Rust wire names — run_net_flag would stop \
             deserializing and the feature would silently do nothing"
        );
        assert_eq!(
            ts_interface_fields(&src, "NetFlagResult"),
            RESULT_FIELDS,
            "ipc.ts NetFlagResult drifted — the status line would render `undefined`"
        );
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
