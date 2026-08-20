//! Phase 6: deviation surveys and true vertical depth via the minimum-curvature method.
//!
//! A deviation survey is a list of stations `(md, inc, azi)` — measured depth along the
//! borehole, inclination from vertical (deg), and azimuth (deg). Minimum curvature is the
//! industry-standard interpolation between two stations: it fits a circular arc through
//! both, which is more accurate than tangential/balanced-tangential for the coarse surveys
//! real wells actually record. TVD is the vertical component of that path; TVDSS is TVD
//! referenced to a datum (subsea), i.e. `TVDSS = TVD - datum_elevation`: depth is positive
//! down while the datum elevation is positive up from mean sea level.

/// One survey station with its computed vertical position.
#[derive(Debug, Clone, Copy)]
pub struct Station {
    pub md: f32,
    pub inc: f32,
    pub azi: f32,
    pub tvd: f32,
    pub tvdss: f32,
}

/// Computes TVD/TVDSS for every station by minimum curvature. `datum_elevation` is the
/// KB (or chosen reference) elevation above mean sea level; TVDSS = TVD - datum.
///
/// The first station anchors the path: its TVD is taken as its own MD (i.e. the survey is
/// assumed to start vertical at surface). Inputs must be sorted by MD ascending and equal
/// length; a survey with fewer than one station returns empty.
pub fn minimum_curvature(md: &[f32], inc_deg: &[f32], azi_deg: &[f32], datum_elevation: f32) -> Vec<Station> {
    let n = md.len();
    if n == 0 || inc_deg.len() != n || azi_deg.len() != n {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(n);
    // Anchor: first station. If it isn't at MD 0 we still take its TVD as its MD, which is
    // the usual convention (vertical from surface to the first recorded station).
    let mut tvd = md[0];
    out.push(Station {
        md: md[0],
        inc: inc_deg[0],
        azi: azi_deg[0],
        tvd,
        tvdss: tvd - datum_elevation,
    });

    for i in 1..n {
        let d_md = md[i] - md[i - 1];
        let i1 = inc_deg[i - 1].to_radians();
        let i2 = inc_deg[i].to_radians();
        let a1 = azi_deg[i - 1].to_radians();
        let a2 = azi_deg[i].to_radians();

        // Dogleg angle between the two tool orientations.
        let cos_dl = (i1.cos() * i2.cos()) + (i1.sin() * i2.sin() * (a2 - a1).cos());
        let cos_dl = cos_dl.clamp(-1.0, 1.0);
        let dl = cos_dl.acos();

        // Ratio factor: (2/dl)·tan(dl/2), → 1 as dl → 0 (straight segment).
        let rf = if dl.abs() < 1e-6 { 1.0 } else { (2.0 / dl) * (dl / 2.0).tan() };

        // Vertical increment = (ΔMD/2)·(cos i1 + cos i2)·rf.
        let d_tvd = (d_md / 2.0) * (i1.cos() + i2.cos()) * rf as f32;
        tvd += d_tvd;

        out.push(Station {
            md: md[i],
            inc: inc_deg[i],
            azi: azi_deg[i],
            tvd,
            tvdss: tvd - datum_elevation,
        });
    }
    out
}

/// Linearly interpolates `(TVD, TVDSS)` at an arbitrary MD from a computed survey — for
/// resampling the survey onto a curve's depth grid. Returns `(md, NaN)` when the survey is
/// empty (vertical-well fallback: TVD == MD, TVDSS undefined). Surveys are short (dozens of
/// stations), so a linear scan is fine.
///
/// **Outside the surveyed range this used to CLAMP to the end stations, and both ends were
/// wrong in their own way.** Above the first station the clamp returned that station's TVD at
/// every shallower MD, so a survey delivered from 300 m down put TVD 300 at MD 0 — a TVD deeper
/// than its own MD, which no well can be. Below the last station it froze TVD, i.e. claimed a
/// **zero vertical increment over real hole**, which no trajectory can produce; on a well logged
/// to 3000 m with stations to 2000 m that was a 1000 m finite plateau that height, correlation
/// and report calculations consumed as geometry.
pub fn sample_at(stations: &[Station], md: f32) -> (f32, f32) {
    if stations.is_empty() {
        return (md, f32::NAN);
    }
    let first = stations[0];
    if md <= first.md {
        // ABOVE the first station the survey states nothing — but `minimum_curvature`'s anchor
        // already does, taking the first station's TVD as its own MD because the hole is assumed
        // vertical from surface down to it. Continuing vertically UP from that station is the
        // only reading consistent with the path we computed, and it is written as a relative
        // step rather than `TVD = MD` so it stays continuous with the anchor whatever produced
        // the stations.
        let up = first.md - md;
        return (first.tvd - up, first.tvdss - up);
    }
    let last = stations[stations.len() - 1];
    if md > last.md {
        // BELOW the last station there is no convention to continue. Freezing is not one, and
        // continuing the last inclination would be an extrapolation nobody authorized. The
        // geometry is MISSING — which is what `materialize_tvd_curves`' own comment has always
        // claimed this function returned outside the survey's MD range.
        return (f32::NAN, f32::NAN);
    }
    for w in stations.windows(2) {
        let (a, b) = (w[0], w[1]);
        if md >= a.md && md <= b.md {
            let t = if (b.md - a.md).abs() < 1e-9 { 0.0 } else { (md - a.md) / (b.md - a.md) };
            return (a.tvd + t * (b.tvd - a.tvd), a.tvdss + t * (b.tvdss - a.tvdss));
        }
    }
    (last.tvd, last.tvdss)
}

/// TVD only at an MD (see [`sample_at`]). Kept for the log/correlation TVD-depth-scale views.
#[allow(dead_code)]
pub fn tvd_at(stations: &[Station], md: f32) -> f32 {
    sample_at(stations, md).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertical_well_tvd_equals_md() {
        // All inclinations zero → path is straight down → TVD == MD.
        let md = [0.0, 1000.0, 2000.0, 3000.0];
        let inc = [0.0, 0.0, 0.0, 0.0];
        let azi = [0.0, 0.0, 0.0, 0.0];
        let s = minimum_curvature(&md, &inc, &azi, 30.0);
        for st in &s {
            assert!((st.tvd - st.md).abs() < 1e-3, "vertical well TVD must equal MD");
        }
        // F-17 / SB-DBM-031: TVDSS is positive down; elevation is positive up.
        assert!((s[1].tvdss - (1000.0 - 30.0)).abs() < 1e-3);
    }

    #[test]
    fn deviated_well_tvd_less_than_md() {
        // A well kicking off to 60° must gain less TVD than MD over the build.
        let md = [0.0, 1000.0, 2000.0];
        let inc = [0.0, 0.0, 60.0];
        let azi = [0.0, 0.0, 45.0];
        let s = minimum_curvature(&md, &inc, &azi, 0.0);
        // 0→1000 vertical: TVD == 1000.
        assert!((s[1].tvd - 1000.0).abs() < 1e-2);
        // 1000→2000 building 0→60°: vertical gain must be well under 1000 but positive.
        let gain = s[2].tvd - s[1].tvd;
        assert!(gain > 700.0 && gain < 950.0, "build-section TVD gain out of range: {gain}");
        assert!(s[2].tvd < s[2].md, "deviated TVD must be shallower than MD");
    }

    #[test]
    fn interpolation_between_and_outside_stations() {
        let md = [0.0, 1000.0, 2000.0];
        let inc = [0.0, 0.0, 0.0];
        let azi = [0.0, 0.0, 0.0];
        let s = minimum_curvature(&md, &inc, &azi, 0.0);
        assert!((tvd_at(&s, 1500.0) - 1500.0).abs() < 1e-2);
        // Shallower than the first station: vertical continuation, not the old clamp to that
        // station's TVD. Deeper than the last: MISSING. See the named pin below for why.
        assert!((tvd_at(&s, -50.0) - (-50.0)).abs() < 1e-2, "vertical continuation above station 1");
        assert!(tvd_at(&s, 9999.0).is_nan(), "past the last station the geometry is unknown");
        assert!((tvd_at(&[], 1234.0) - 1234.0).abs() < 1e-6, "empty survey → MD passthrough");
    }

    #[test]
    fn sample_at_interpolates_both_tvd_and_tvdss() {
        // F-17 / SB-DBM-031: 30 m elevation is positive up, so TVDSS = TVD - 30 m.
        let md = [0.0, 1000.0, 2000.0];
        let inc = [0.0, 0.0, 0.0];
        let azi = [0.0, 0.0, 0.0];
        let s = minimum_curvature(&md, &inc, &azi, 30.0);
        let (tvd, tvdss) = sample_at(&s, 1500.0);
        assert!((tvd - 1500.0).abs() < 1e-2, "tvd={tvd}");
        assert!((tvdss - (1500.0 - 30.0)).abs() < 1e-2, "tvdss={tvdss}");
        // Above the first station TVDSS follows the vertical continuation, so it keeps stepping
        // 1:1 with MD against the same datum; below the last station it is unknown, not frozen.
        assert!((sample_at(&s, -50.0).1 - (-50.0 - 30.0)).abs() < 1e-2);
        assert!(sample_at(&s, 9999.0).1.is_nan());
        // Empty survey → (MD, NaN).
        let (t0, ss0) = sample_at(&[], 1234.0);
        assert!((t0 - 1234.0).abs() < 1e-6 && ss0.is_nan());
    }

    /// Codex whole-repository review, P1. `sample_at` CLAMPED to the end stations outside the
    /// surveyed range, and `materialize_tvd_curves` calls it for every sample on the full log
    /// grid — so a partial survey produced long finite plateaus that height, correlation and
    /// report calculations consumed as real geometry. `ingest.rs`' own comment on that call has
    /// always described NaN outside the survey's MD range; the function never did it.
    ///
    /// Both ends were wrong, in different ways, and the fix is different at each because only one
    /// of them has a convention to fall back on.
    #[test]
    fn a_survey_states_no_geometry_past_its_last_station_and_says_so() {
        // A vertical survey stopping at 2000 m, in a well logged to 3000 m. Vertical on purpose:
        // the true trajectory here is not in doubt, so anything the function returns below 2000 m
        // is the function's own invention rather than a hard geometry question.
        let s = minimum_curvature(&[0.0, 1000.0, 2000.0], &[0.0; 3], &[0.0; 3], 30.0);

        // DEEPER THAN THE LAST STATION: missing, not frozen. The old clamp answered 2000 at both
        // 2500 and 3000 - a zero vertical increment over 1000 m of hole, which no trajectory can
        // produce, presented as a normal curve.
        for md in [2000.1, 2500.0, 3000.0] {
            let (tvd, tvdss) = sample_at(&s, md);
            assert!(
                tvd.is_nan() && tvdss.is_nan(),
                "past the last station both must be missing, got tvd {tvd} tvdss {tvdss} at MD {md}"
            );
        }
        // The last station itself is still answered - the boundary belongs to the survey.
        let (tvd_last, _) = sample_at(&s, 2000.0);
        assert!((tvd_last - 2000.0).abs() < 1e-2, "the last station itself still answers: {tvd_last}");

        // SHALLOWER THAN THE FIRST STATION there IS a convention, and the clamp contradicted it.
        // `minimum_curvature` anchors by taking the first station's TVD as its own MD - the hole
        // is assumed vertical from surface down to it - so the continuation is vertical too.
        let deep_start = minimum_curvature(&[300.0, 1000.0], &[0.0, 0.0], &[0.0, 0.0], 30.0);
        let (shallow, shallow_ss) = sample_at(&deep_start, 0.0);
        assert!(
            (shallow - 0.0).abs() < 1e-2,
            "vertical continuation puts MD 0 at TVD 0, not at the first station's 300: {shallow}"
        );
        assert!(
            (shallow_ss - (0.0 - 30.0)).abs() < 1e-2,
            "and TVDSS steps with it against the same datum: {shallow_ss}"
        );
        // The reason that half matters at all: the clamp answered 300 here, a TVD DEEPER than its
        // own MD, which is impossible for any well and was reachable from any survey delivered
        // from below surface.
        for md in [0.0, 100.0, 299.0] {
            let (tvd, _) = sample_at(&deep_start, md);
            assert!(tvd <= md + 1e-2, "TVD can never exceed MD: got {tvd} at MD {md}");
        }
    }
}
