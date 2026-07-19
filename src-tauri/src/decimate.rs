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

    (out_depths, out_values)
}
