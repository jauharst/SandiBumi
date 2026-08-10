/// Downsamples (depth, value) pairs to `target_pixel_height` points using min/max visual
/// bucketing: each output bucket keeps the min and max value it contains, so spikes and
/// thin-bed excursions survive the downsample instead of being averaged away. NaNs are
/// skipped when picking min/max but never fabricated.
pub fn min_max_decimate(depths: &[f32], values: &[f32], target_pixel_height: usize) -> (Vec<f32>, Vec<f32>) {
    let n = depths.len();
    if target_pixel_height == 0 || n == 0 {
        return (Vec::new(), Vec::new());
    }
    // Two output points (min, max) per pixel bucket, so if the data already fits, pass through.
    if n <= target_pixel_height * 2 {
        return (depths.to_vec(), values.to_vec());
    }

    let bucket_count = target_pixel_height;
    let bucket_size = n as f32 / bucket_count as f32;

    let mut out_depths = Vec::with_capacity(bucket_count * 2);
    let mut out_values = Vec::with_capacity(bucket_count * 2);

    for b in 0..bucket_count {
        let start = (b as f32 * bucket_size) as usize;
        let end = (((b + 1) as f32 * bucket_size) as usize).min(n).max(start + 1);

        let mut min_idx: Option<usize> = None;
        let mut max_idx: Option<usize> = None;
        let mut min_val = f32::INFINITY;
        let mut max_val = f32::NEG_INFINITY;

        for i in start..end {
            let v = values[i];
            if v.is_nan() {
                continue;
            }
            if v < min_val {
                min_val = v;
                min_idx = Some(i);
            }
            if v > max_val {
                max_val = v;
                max_idx = Some(i);
            }
        }

        match (min_idx, max_idx) {
            (Some(mi), Some(xi)) => {
                // Preserve depth order within the bucket regardless of which extreme came first.
                let (first, second) = if mi <= xi { (mi, xi) } else { (xi, mi) };
                out_depths.push(depths[first]);
                out_values.push(values[first]);
                if first != second {
                    out_depths.push(depths[second]);
                    out_values.push(values[second]);
                }
            }
            _ => {
                // Entire bucket is NaN — emit a single NaN point so line breaks render correctly.
                out_depths.push(depths[start]);
                out_values.push(f32::NAN);
            }
        }
    }

    // The initial viewer request uses this disposable reduction to establish the permanent
    // whole-well extent. Min/max extrema alone do not necessarily include either end sample,
    // so carry the true source endpoints as structural points. They are still original rows;
    // no value or depth is synthesized, and the two extra points do not weaken spike retention.
    if out_depths.first().copied() != Some(depths[0]) {
        out_depths.insert(0, depths[0]);
        out_values.insert(0, values[0]);
    }
    if out_depths.last().copied() != Some(depths[n - 1]) {
        out_depths.push(depths[n - 1]);
        out_values.push(values[n - 1]);
    }

    (out_depths, out_values)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The viewer derives its persistent whole-well extent from the initial disposable
    /// decimation. End samples therefore carry structural information even when neither is
    /// a value extreme inside its bucket.
    #[test]
    fn decimation_retains_true_source_endpoints() {
        let depths: Vec<f32> = (0..10).map(|index| 1000.0 + index as f32 * 0.5).collect();
        let values = vec![5.0, 0.0, 10.0, 4.0, 3.0, 2.0, 1.0, 9.0, 0.0, 5.0];

        let (reduced_depth, reduced_value) = min_max_decimate(&depths, &values, 1);

        assert_eq!(reduced_depth.first(), depths.first(), "top extent is a source endpoint");
        assert_eq!(reduced_value.first(), values.first());
        assert_eq!(reduced_depth.last(), depths.last(), "base extent is a source endpoint");
        assert_eq!(reduced_value.last(), values.last());
        assert!(
            reduced_value.contains(&0.0) && reduced_value.contains(&10.0),
            "endpoint retention must not discard bucket extrema"
        );
    }
}
