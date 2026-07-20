//! Planar geometry for the Field Map (Wave E item 22): the point-in-polygon test behind
//! "draw a polygon on the map → assign the enclosed wells to a group". Coordinates are
//! raw UTM easting/northing (metres); the map draws in that space, so no projection is
//! involved here — this is pure 2-D geometry.

use crate::db;

/// Ray-casting point-in-polygon test (the classic PNPOLY algorithm). `poly` is the ordered
/// vertex ring `[(x, y), …]`, given open — the closing edge from the last vertex back to
/// the first is implied. Returns true when `(px, py)` is inside, handling concave and
/// self-touching rings correctly via the half-open crossing rule (each edge counts its
/// lower endpoint, `[y_min, y_max)`), which also keeps a point exactly on a shared boundary
/// between two abutting polygons from landing in both. A point lying precisely on an edge is
/// a boundary case that may report either way — acceptable for selecting wells with a
/// hand-drawn polygon.
pub fn point_in_polygon(px: f64, py: f64, poly: &[(f64, f64)]) -> bool {
    if poly.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[j];
        // Does the rightward horizontal ray at `py` cross edge j→i? Half-open in y so a
        // vertex shared by two edges is counted once; when true, yi != yj is guaranteed
        // (one endpoint is strictly above py, the other at-or-below), so the divide is safe.
        if (yi > py) != (yj > py) {
            let x_int = xi + (py - yi) / (yj - yi) * (xj - xi);
            if px < x_int {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

/// Filters `wells` to those whose surface location lies inside `polygon`. Wells without
/// coordinates — or with non-finite ones — are skipped rather than counted at the origin.
pub fn wells_in_polygon(wells: &[db::WellSummary], polygon: &[(f64, f64)]) -> Vec<db::WellSummary> {
    wells
        .iter()
        .filter(|w| match (w.surface_x, w.surface_y) {
            (Some(x), Some(y)) if x.is_finite() && y.is_finite() => point_in_polygon(x, y, polygon),
            _ => false,
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn well(name: &str, x: Option<f64>, y: Option<f64>) -> db::WellSummary {
        db::WellSummary {
            well_id: name.to_string(),
            well_name: name.to_string(),
            field_name: None,
            td: None,
            kb: None,
            surface_x: x,
            surface_y: y,
            utm_zone: None,
        }
    }

    #[test]
    fn square_inside_and_outside() {
        let sq = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        assert!(point_in_polygon(5.0, 5.0, &sq), "centre is inside");
        assert!(!point_in_polygon(15.0, 5.0, &sq), "east of the square is outside");
        assert!(!point_in_polygon(5.0, -1.0, &sq), "below the square is outside");
        assert!(!point_in_polygon(-3.0, -3.0, &sq), "far corner is outside");
    }

    #[test]
    fn concave_notch_is_outside() {
        // A C-shape: unit square with a rectangular bite taken out of the right side so the
        // notch (x≈8, y=5) sits between the two prongs and must read as OUTSIDE.
        let c = [
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 4.0),
            (6.0, 4.0),
            (6.0, 6.0),
            (10.0, 6.0),
            (10.0, 10.0),
            (0.0, 10.0),
        ];
        assert!(point_in_polygon(3.0, 5.0, &c), "left body is inside");
        assert!(!point_in_polygon(8.0, 5.0, &c), "the notch is outside despite being between prongs");
        assert!(point_in_polygon(8.0, 2.0, &c), "lower prong is inside");
        assert!(point_in_polygon(8.0, 8.0, &c), "upper prong is inside");
    }

    #[test]
    fn degenerate_polygons_never_contain() {
        assert!(!point_in_polygon(0.0, 0.0, &[]), "empty");
        assert!(!point_in_polygon(0.0, 0.0, &[(0.0, 0.0)]), "single vertex");
        assert!(!point_in_polygon(0.0, 0.0, &[(0.0, 0.0), (1.0, 1.0)]), "a segment has no interior");
    }

    #[test]
    fn realistic_utm_northing_precision() {
        // Southern-hemisphere UTM northings run to ~9.4e6; a 200 m box must still resolve a
        // well one metre inside its edge (this is why coordinates are f64, not f32).
        let e = 480_000.0;
        let n = 9_400_000.0;
        let box_ = [(e, n), (e + 200.0, n), (e + 200.0, n + 200.0), (e, n + 200.0)];
        assert!(point_in_polygon(e + 1.0, n + 1.0, &box_), "1 m inside the SW corner");
        assert!(!point_in_polygon(e - 1.0, n + 100.0, &box_), "1 m outside the west edge");
    }

    #[test]
    fn wells_filter_skips_coordless_and_nonfinite() {
        let sq = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        let wells = vec![
            well("IN", Some(5.0), Some(5.0)),
            well("OUT", Some(50.0), Some(50.0)),
            well("NOCOORD", None, None),
            well("HALF", Some(5.0), None),
            well("NAN", Some(f64::NAN), Some(5.0)),
        ];
        let hit = wells_in_polygon(&wells, &sq);
        assert_eq!(hit.len(), 1, "only the one well inside with finite coords");
        assert_eq!(hit[0].well_name, "IN");
    }
}
