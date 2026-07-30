use serde::{Deserialize, Serialize};

/// One curve's display style + scale within a track (curves in the same track can have
/// independent min/max, e.g. an NPHI/RHOB porosity overlay). Optional fill fields shade
/// the area between the curve and a track edge ("left" | "right"), between the curve and
/// another curve in the same track ("curve" — crossover), or draw a discrete class curve
/// as full-width blocks ("blocks").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveStyle {
    pub curve_name: String,
    pub color: String,
    pub min: f32,
    pub max: f32,
    /// "line" (default) joins consecutive sample centres with a straight segment; "step"
    /// holds each sample's value across its whole sampling interval and then jumps. Step is
    /// the honest display for anything that is genuinely piecewise-constant — block-averaged
    /// or upscaled logs, zone-constant parameter curves, coarse core-derived tracks — where a
    /// diagonal join would draw a gradient the data never measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draw_style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    /// `fill = "curve"`: the reference curve to shade against. It must be another curve in
    /// the SAME track, and it is positioned with ITS OWN min/max — that compatible-scaling
    /// is exactly what makes a neutron-density crossover mean anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<String>,
    /// `fill = "curve"`: colour where this curve reads to the RIGHT of `fill_to`
    /// (`fill_color` covers the left side). On a compatible-scaled NPHI/RHOB overlay with
    /// NPHI carrying the style, right-of-RHOB is the gas-effect crossover.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_color2: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_opacity: Option<f32>,
}

/// Plain curve line, no shading.
fn curve(name: &str, color: &str, min: f32, max: f32) -> CurveStyle {
    CurveStyle {
        curve_name: name.into(),
        color: color.into(),
        min,
        max,
        draw_style: None,
        fill: None,
        fill_to: None,
        fill_color: None,
        fill_color2: None,
        fill_opacity: None,
    }
}

/// Discrete class curve (electrofacies, clusters) rendered as full-track-width colored
/// blocks. `fill = "blocks"` is the tag both the WebGPU viewer and the composite exporter
/// understand; per-class colors come from the shared facies palette, not `color`.
fn block_curve(name: &str, max_classes: f32) -> CurveStyle {
    CurveStyle {
        fill: Some("blocks".into()),
        fill_opacity: Some(0.85),
        ..curve(name, "#9c755f", 0.0, max_classes)
    }
}

/// Curve with edge shading ("left" | "right").
fn filled(name: &str, color: &str, min: f32, max: f32, fill: &str, opacity: f32) -> CurveStyle {
    CurveStyle {
        fill: Some(fill.into()),
        fill_color: Some(color.into()),
        fill_opacity: Some(opacity),
        ..curve(name, color, min, max)
    }
}

/// Crossover shading between this curve and another curve in the same track. `left` shades
/// where this curve reads left of `to`, `right` where it reads right of it — the two-colour
/// separation display every neutron-density overlay is read by.
fn crossover(
    name: &str,
    color: &str,
    min: f32,
    max: f32,
    to: &str,
    left: &str,
    right: &str,
) -> CurveStyle {
    CurveStyle {
        fill: Some("curve".into()),
        fill_to: Some(to.into()),
        fill_color: Some(left.into()),
        fill_color2: Some(right.into()),
        fill_opacity: Some(0.35),
        ..curve(name, color, min, max)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScaleType {
    Linear,
    Log,
}

/// What a track draws. "curves" is the normal log track; "well_diagram" ignores curves and
/// instead draws casing / shoe / tubing / perforations from the well's COMPLETION + PERFORATION
/// aux datasets; "point_data" ignores curves too and draws the `points` block — measured
/// samples (core plugs, XRD, CEC, oil show, core extras) rather than a continuous log.
/// `#[serde(default)]` on the field keeps old saved layouts (no `kind`) loading.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrackKind {
    #[default]
    Curves,
    WellDiagram,
    PointData,
    ArrayLog,
}

/// One array log drawn in an `array_log` track — a curve that holds a whole DISTRIBUTION at
/// every depth (Monte Carlo realizations, an NMR T2 distribution, a sonic waveform) rather
/// than a single reading.
///
/// The three displays are three readings of the SAME stored matrix, which is the point: the
/// percentiles become a display setting you change, not a reason to re-run the study. All
/// three go through `crate::distribution`, so a band drawn here and a box plot drawn on a
/// point track answer the same question the same way.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayStyle {
    pub curve_name: String,
    /// Which array set to read. `None` takes whichever set holds the curve — array logs are
    /// produced outputs, so a well normally carries exactly one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set_name: Option<String>,
    pub color: String,
    pub min: f32,
    pub max: f32,
    /// "band" (default) | "spaghetti" | "heatmap".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    /// Band edges as percentiles (defaults 10 / 90). These are the "adjustable" part: with the
    /// matrix stored, moving P10 to P5 is a redraw, not a re-run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub band_lo: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub band_hi: Option<f32>,
    /// Draw the P50 line inside the band (default true).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_median: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_opacity: Option<f32>,
    /// "spaghetti": how many realizations to draw (default 40, clamped to what is stored).
    /// Traces are taken EVENLY across the stored set, never the first N — the first N of a
    /// Latin-hypercube design is a biased corner of the sampled space.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub traces: Option<u32>,
    /// "heatmap": bins across the VALUE axis (default 32). Density is drawn as opacity of the
    /// series colour rather than a colour ramp, so there is no second palette to keep in sync
    /// between the viewer and the print exporter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hist_bins: Option<u32>,
}

impl ArrayStyle {
    pub fn display_kind(&self) -> &str {
        match self.display.as_deref() {
            Some("spaghetti") => "spaghetti",
            Some("heatmap") => "heatmap",
            _ => "band",
        }
    }

    /// Band edges, defaulting to the P10/P90 pair Monte Carlo results are conventionally
    /// reported at. Returned low-first even if the user typed them the other way round.
    pub fn band_edges(&self) -> (f32, f32) {
        let lo = self.band_lo.unwrap_or(10.0);
        let hi = self.band_hi.unwrap_or(90.0);
        if lo <= hi {
            (lo, hi)
        } else {
            (hi, lo)
        }
    }
}

/// One measured-sample series drawn in a `point_data` track.
///
/// The separation from `CurveStyle` is deliberate. A curve is a continuous reading with a
/// value at every depth; a point series is a set of discrete measurements at the depths
/// somebody actually sampled, and the honest displays for the two are different. Interpolating
/// between core plugs to draw a line would state a continuity the data does not have.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointStyle {
    /// "core" reads a plug property (CPOR, CPERM, CGD, CSW …) from the ACTIVE core set;
    /// "aux" reads an item from a point dataset (XRD, CEC, oil show, core extras …).
    pub source: String,
    /// For `source = "aux"`, which dataset. Ignored for core.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset: Option<String>,
    /// The property (core) or item (aux) name within that source.
    pub item: String,
    pub color: String,
    pub min: f32,
    pub max: f32,
    /// "points" (default, one glyph per sample) | "box" (a box plot per depth bin) |
    /// "histogram" (a value-axis histogram per depth bin) | "text" (the sample's text value).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    /// Depth-bin height for "box" / "histogram", in the project's depth unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bin: Option<f32>,
    /// Box edges as percentiles (defaults 25 / 75).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub box_lo: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub box_hi: Option<f32>,
    /// "tukey" (default) | "percentile" | "minmax". See `distribution::Whisker` — this is an
    /// interpretive choice, so it is stored with the layout rather than assumed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whisker: Option<String>,
    /// Tukey multiplier (default 1.5) or, for "percentile", the low/high pair.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whisker_k: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whisker_lo: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whisker_hi: Option<f32>,
    /// Bin count across the value axis for "histogram" (default 12).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hist_bins: Option<u32>,
    /// Draw the individual samples on top of a box/histogram glyph. Off by default; on a
    /// sparse interval seeing the plugs behind the summary is often the point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_samples: Option<bool>,
}

impl PointStyle {
    /// Resolves the stored strings into the whisker rule the shared distribution module takes.
    pub fn whisker_rule(&self) -> crate::distribution::Whisker {
        match self.whisker.as_deref() {
            Some("minmax") => crate::distribution::Whisker::MinMax,
            Some("percentile") => crate::distribution::Whisker::Percentile(
                self.whisker_lo.unwrap_or(10.0),
                self.whisker_hi.unwrap_or(90.0),
            ),
            _ => crate::distribution::Whisker::Tukey(self.whisker_k.unwrap_or(1.5)),
        }
    }

    /// Box edges, defaulting to the conventional quartiles.
    pub fn box_edges(&self) -> (f32, f32) {
        (self.box_lo.unwrap_or(25.0), self.box_hi.unwrap_or(75.0))
    }
}

/// One vertical track in a layout, analogous to a track/scale layout block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub title: String,
    pub width_weight: f32,
    pub scale_type: ScaleType,
    #[serde(default)]
    pub kind: TrackKind,
    pub curves: Vec<CurveStyle>,
    /// Measured-sample series, drawn only when `kind = "point_data"`. `#[serde(default)]`
    /// keeps every layout saved before point tracks existed loading unchanged.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<PointStyle>,
    /// Distribution-per-depth series, drawn only when `kind = "array_log"`. Defaulted for the
    /// same back-compatibility reason as `points`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arrays: Vec<ArrayStyle>,
}

/// A named, reusable track layout — SandiBumi's reusable track-layout registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub name: String,
    pub tracks: Vec<Track>,
}

/// The built-in "Standard Layout": GR (shaded) / deep resistivity (log) / NPHI-RHOB overlay.
pub fn standard_layout() -> Layout {
    Layout {
        name: "Standard Layout".into(),
        tracks: vec![
            Track {
                title: "GR".into(),
                width_weight: 1.0,
                scale_type: ScaleType::Linear,
                kind: TrackKind::Curves,
                points: Vec::new(),
                arrays: Vec::new(),
                curves: vec![filled("GR", "#5f7350", 0.0, 150.0, "left", 0.25)],
            },
            Track {
                title: "RES_DEEP".into(),
                width_weight: 1.0,
                scale_type: ScaleType::Log,
                kind: TrackKind::Curves,
                points: Vec::new(),
                arrays: Vec::new(),
                curves: vec![curve("RES_DEEP", "#a83e2c", 0.2, 2000.0)],
            },
            Track {
                title: "NPHI / RHOB".into(),
                width_weight: 1.0,
                scale_type: ScaleType::Linear,
                kind: TrackKind::Curves,
                points: Vec::new(),
                arrays: Vec::new(),
                curves: vec![
                    // NPHI runs right-to-left (porosity convention) via a reversed min/max.
                    // The two scales are the standard compatible pair (NPHI 0.45→-0.15 v/v
                    // against RHOB 1.95→2.95 g/cc), so the curves overlay in a clean
                    // water-bearing sand and separate either way for a reason: NPHI left of
                    // RHOB is shale/clay-bound water, NPHI right of RHOB is the gas effect.
                    // Colours follow that reading, not the curve colours.
                    crossover("NPHI", "#3d6a9e", 0.45, -0.15, "RHOB", "#9aa5ad", "#e8c33a"),
                    curve("RHOB", "#b5651d", 1.95, 2.95),
                ],
            },
        ],
    }
}

/// The built-in "Interpretation" layout: raw GR/RES plus the deterministic workflow's
/// output curves — VSH, PHIE/PHIT overlay, SWE, PERM (log), and the pay flags (shaded).
pub fn interpretation_layout() -> Layout {
    Layout {
        name: "Interpretation".into(),
        tracks: vec![
            Track {
                title: "GR".into(),
                width_weight: 1.0,
                scale_type: ScaleType::Linear,
                kind: TrackKind::Curves,
                points: Vec::new(),
                arrays: Vec::new(),
                curves: vec![filled("GR", "#5f7350", 0.0, 150.0, "left", 0.25)],
            },
            Track {
                title: "RES_DEEP".into(),
                width_weight: 1.0,
                scale_type: ScaleType::Log,
                kind: TrackKind::Curves,
                points: Vec::new(),
                arrays: Vec::new(),
                curves: vec![curve("RES_DEEP", "#a83e2c", 0.2, 2000.0)],
            },
            Track {
                title: "VSH".into(),
                width_weight: 1.0,
                scale_type: ScaleType::Linear,
                kind: TrackKind::Curves,
                points: Vec::new(),
                arrays: Vec::new(),
                curves: vec![filled("VSH", "#8d6e63", 0.0, 1.0, "left", 0.3)],
            },
            Track {
                title: "PHIE / PHIT".into(),
                width_weight: 1.0,
                scale_type: ScaleType::Linear,
                kind: TrackKind::Curves,
                points: Vec::new(),
                arrays: Vec::new(),
                curves: vec![
                    // Porosity convention: decreasing to the right.
                    curve("PHIT", "#54a0ff", 0.5, 0.0),
                    filled("PHIE", "#1e5fb8", 0.5, 0.0, "right", 0.2),
                ],
            },
            Track {
                title: "SWE".into(),
                width_weight: 1.0,
                scale_type: ScaleType::Linear,
                kind: TrackKind::Curves,
                points: Vec::new(),
                arrays: Vec::new(),
                curves: vec![curve("SWE", "#00b8d4", 0.0, 1.0)],
            },
            Track {
                title: "PERM".into(),
                width_weight: 1.0,
                scale_type: ScaleType::Log,
                kind: TrackKind::Curves,
                points: Vec::new(),
                arrays: Vec::new(),
                curves: vec![curve("PERM", "#b5651d", 0.01, 10000.0)],
            },
            Track {
                title: "PAY".into(),
                width_weight: 0.5,
                scale_type: ScaleType::Linear,
                kind: TrackKind::Curves,
                points: Vec::new(),
                arrays: Vec::new(),
                curves: vec![
                    filled("FLAG_PAY", "#5f7350", 0.0, 1.0, "left", 0.55),
                    filled("FLAG_RESERVOIR", "#c2ac81", 0.0, 1.0, "left", 0.35),
                ],
            },
        ],
    }
}

/// The built-in "Facies" layout: raw GR/RES/porosity context plus the FACIES cluster
/// index from the electrofacies module as a colored block track.
pub fn facies_layout() -> Layout {
    Layout {
        name: "Facies".into(),
        tracks: vec![
            Track {
                title: "GR".into(),
                width_weight: 1.0,
                scale_type: ScaleType::Linear,
                kind: TrackKind::Curves,
                points: Vec::new(),
                arrays: Vec::new(),
                curves: vec![filled("GR", "#5f7350", 0.0, 150.0, "left", 0.25)],
            },
            Track {
                title: "RES_DEEP".into(),
                width_weight: 1.0,
                scale_type: ScaleType::Log,
                kind: TrackKind::Curves,
                points: Vec::new(),
                arrays: Vec::new(),
                curves: vec![curve("RES_DEEP", "#a83e2c", 0.2, 2000.0)],
            },
            Track {
                title: "NPHI / RHOB".into(),
                width_weight: 1.0,
                scale_type: ScaleType::Linear,
                kind: TrackKind::Curves,
                points: Vec::new(),
                arrays: Vec::new(),
                curves: vec![
                    // Same compatible-scaled crossover as the Standard Layout — the porosity
                    // track reads identically wherever it appears.
                    crossover("NPHI", "#3d6a9e", 0.45, -0.15, "RHOB", "#9aa5ad", "#e8c33a"),
                    curve("RHOB", "#b5651d", 1.95, 2.95),
                ],
            },
            Track {
                title: "FACIES".into(),
                width_weight: 0.6,
                scale_type: ScaleType::Linear,
                kind: TrackKind::Curves,
                points: Vec::new(),
                arrays: Vec::new(),
                curves: vec![block_curve("FACIES", 12.0)],
            },
        ],
    }
}

/// All built-in layouts (user-saved layouts live in the `documents` table).
pub fn list_layouts() -> Vec<Layout> {
    vec![standard_layout(), interpretation_layout(), facies_layout()]
}
