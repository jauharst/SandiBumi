//! Measurements taken off a thin section, starting with pore area from blue-dyed epoxy.
//!
//! The deliverable is an **area fraction per plate**, which under the standard stereological
//! argument (Delesse) estimates the volume fraction. It is deliberately the first of the three
//! measurement families because it is **dimensionless**: an area fraction needs no micrometres per
//! pixel, so it runs on every plate rather than only the calibrated ones (see
//! `docs/plan_image_analysis.md` §2.0).
//!
//! Four rules hold this together, and each closes a way of being confidently wrong.
//!
//! **A plate must be DECLARED impregnated, and an undeclared one is refused by name.** This is the
//! whole reason `well_images.prepared` exists. A blue rule run over a section nobody impregnated
//! does not fail — it returns a porosity assembled from blue-ish feldspar, stain bleed and edge
//! artefact, and that number then plots against core helium porosity looking entirely reasonable.
//! Nor can the app work it out from the pixels: the evidence for "this is blue epoxy" is the blue
//! it was about to measure, which is the same circle as reading a water zone off the saturation
//! being calibrated.
//!
//! **The colour band is the user's, not the app's.** The defaults here are a plain blue band —
//! round numbers, no calibration behind them — offered as a starting point for a VISUAL tuning
//! task, never as a constant that ships silently. The dialog shows the mask over the plate and the
//! user adjusts until it matches what they see down the microscope.
//!
//! **The preview comes from the SAME code as the measurement.** Drawing the mask in the frontend
//! would put the segmentation in two languages, and the two would drift — the mistake this repo
//! keeps a standing warning about for `composite.rs` against the log-view renderer. So the runner
//! returns the overlay PNG, and what the user tunes against is literally what gets measured.
//!
//! **No morphological cleaning.** Opening or closing a mask needs a structuring element measured
//! in PIXELS, which is a size — and a plate may carry no scale at all, so that size could not be
//! stated in microns for every plate. Rather than pick a pixel count that means a different
//! physical distance on every plate, nothing is smoothed and the speckle is left visible in the
//! preview where it can be judged.
//!
//! Results are POINT DATA at the plate's depth, not a curve. A thin section is cut from one plug
//! and measures that plug; joining a column of them with a line would state a continuity the data
//! does not have — the same argument that made point data a track kind rather than a `CurveStyle`.

use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Command, Stdio};

use crate::python_engine::{find_python, hide_console};

/// Plates per subprocess. Bounds the bytes held in memory and piped at once: a core-photograph
/// delivery can be hundreds of plates at roughly a megabyte each, and one batch of all of them
/// would be a gigabyte in flight for no gain.
const CHUNK: usize = 16;

/// The colour rule, in HSV. Hue in degrees, saturation and value 0..1.
///
/// **These defaults are a generic blue band, not a calibration.** Blue-dyed epoxy sits in the
/// blue-to-violet part of the wheel on any microscope; where exactly depends on the dye, the lamp,
/// the white balance and the scan, none of which this app knows. They exist so the preview has
/// something to draw on the first click, and they are round numbers on purpose — a two-decimal
/// threshold would be a regression result, and there is no regression behind these.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoreColorBand {
    pub hue_lo: f32,
    pub hue_hi: f32,
    pub sat_min: f32,
    pub val_min: f32,
}

impl Default for PoreColorBand {
    fn default() -> Self {
        Self { hue_lo: 180.0, hue_hi: 260.0, sat_min: 0.15, val_min: 0.10 }
    }
}

/// One run of the pore measurement over a well's live image delivery.
#[derive(Debug, Clone, Deserialize)]
pub struct PoreSpec {
    pub well_id: String,
    pub dataset: String,
    #[serde(default)]
    pub band: PoreColorBand,
    /// Draw the overlay for this plate and return it. `None` measures without a picture back.
    #[serde(default)]
    pub preview_image_id: Option<String>,
    /// Measure only this plate. Used by the tuning preview so adjusting a slider does not
    /// re-measure a 300-plate delivery.
    #[serde(default)]
    pub only_image_id: Option<String>,
    /// Store the results as point data under this delivery name. `None` measures without writing —
    /// tuning must not leave a trail of half-judged answers in the project.
    #[serde(default)]
    pub set_name: Option<String>,
}

/// What one plate came to.
#[derive(Debug, Clone, Serialize)]
pub struct PlatePore {
    pub image_id: String,
    pub name: String,
    pub depth_top: f32,
    pub depth_base: Option<f32>,
    /// Pore area as a fraction of the plate, v/v.
    pub pore_fraction: f32,
    /// Pixels examined — the whole plate, since nothing is masked out.
    pub pixels: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PoreResult {
    pub plates: Vec<PlatePore>,
    /// Plates left out and why, one entry each — never a silent subset.
    pub skipped: Vec<String>,
    /// Base64 PNG of the mask drawn over the requested plate, when one was asked for.
    pub preview_png: Option<String>,
    pub preview_width: i32,
    pub preview_height: i32,
    /// Point dataset and delivery written, when `set_name` was given.
    pub written: Option<(String, String)>,
    pub notes: Vec<String>,
}

/// The point-data item a measured pore fraction is stored as.
pub const PORE_ITEM: &str = "VPORE_TS";
/// The point dataset it lands in. Deliberately its own dataset rather than the image delivery's
/// name: it is a MEASUREMENT derived from the pictures, not part of the delivery, and re-running
/// the measurement must not look like a second delivery of plates.
pub const PORE_DATASET: &str = "PETROGRAPHY";

const PORE_RUNNER: &str = r#"
import sys, json, io, base64
try:
    import numpy as np
    from PIL import Image
except Exception as e:
    sys.stderr.write("needs numpy and Pillow: %s\n" % e)
    sys.exit(1)

# stdin.buffer, never stdin: a piped child's TEXT stdin decodes with the Windows ANSI codepage
# while serde_json emits UTF-8, so a plate name with any non-ASCII character arrives as mojibake.
header = json.loads(sys.stdin.buffer.readline().decode("utf-8"))
band = header["band"]
sizes = header["sizes"]
ids = header["ids"]
preview = header.get("preview")

blobs = []
for n in sizes:
    blobs.append(sys.stdin.buffer.read(n))

def mask_of(img):
    a = np.asarray(img.convert("RGB"), dtype=np.float32) / 255.0
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    mx = np.max(a, axis=-1)
    mn = np.min(a, axis=-1)
    d = mx - mn
    # Hue in degrees. Where the pixel is grey (d == 0) hue is undefined; it is set to 0 and the
    # saturation floor below is what actually rejects it, so an undefined hue never counts as blue.
    h = np.zeros_like(mx)
    safe = d > 1e-6
    rmax = safe & (mx == r)
    gmax = safe & (mx == g) & ~rmax
    bmax = safe & ~rmax & ~gmax
    with np.errstate(invalid="ignore", divide="ignore"):
        h[rmax] = (60.0 * ((g[rmax] - b[rmax]) / d[rmax])) % 360.0
        h[gmax] = 60.0 * ((b[gmax] - r[gmax]) / d[gmax]) + 120.0
        h[bmax] = 60.0 * ((r[bmax] - g[bmax]) / d[bmax]) + 240.0
    s = np.where(mx > 0, d / np.maximum(mx, 1e-6), 0.0)
    v = mx
    lo = float(band["hue_lo"]); hi = float(band["hue_hi"])
    if lo <= hi:
        inband = (h >= lo) & (h <= hi)
    else:
        # A band written across 0 degrees (e.g. 340 to 20) is two arcs, not an empty range.
        inband = (h >= lo) | (h <= hi)
    return inband & (s >= float(band["sat_min"])) & (v >= float(band["val_min"]))

out = {"results": [], "preview_png": None, "preview_w": 0, "preview_h": 0}
for i, blob in enumerate(blobs):
    try:
        img = Image.open(io.BytesIO(blob))
        img.load()
    except Exception as e:
        out["results"].append({"image_id": ids[i], "error": "cannot decode: %s" % e})
        continue
    m = mask_of(img)
    total = int(m.size)
    hits = int(np.count_nonzero(m))
    out["results"].append({
        "image_id": ids[i],
        "pore_fraction": (hits / total) if total else 0.0,
        "pixels": total,
    })
    if preview is not None and ids[i] == preview:
        # The overlay is drawn from the SAME mask that produced the number above. What the user
        # tunes against is literally what was measured.
        rgb = np.asarray(img.convert("RGB")).copy()
        rgb[m] = (0.35 * rgb[m] + 0.65 * np.array([255, 40, 40], dtype=np.float32)).astype(np.uint8)
        small = Image.fromarray(rgb)
        small.thumbnail((900, 900))
        buf = io.BytesIO()
        small.save(buf, format="PNG")
        out["preview_png"] = base64.b64encode(buf.getvalue()).decode("ascii")
        out["preview_w"] = small.width
        out["preview_h"] = small.height

sys.stdout.write(json.dumps(out))
"#;

const SUPPORT_RUNNER: &str = r#"
import sys
ok = True
try:
    import numpy  # noqa: F401
    from PIL import Image  # noqa: F401
except Exception:
    ok = False
sys.stdout.write("1" if ok else "0")
"#;

/// Can the pore measurement run at all? Probed once so a dialog can say what is missing and name
/// the interpreter to install into, rather than failing at the end of a long run.
pub fn pore_support() -> Result<bool, String> {
    let python = find_python().ok_or("no Python interpreter found")?;
    let mut cmd = Command::new(&python);
    cmd.args(["-c", SUPPORT_RUNNER]).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);
    let out = cmd.output().map_err(|e| format!("failed to start python: {e}"))?;
    Ok(String::from_utf8_lossy(&out.stdout).trim() == "1")
}

#[derive(Deserialize)]
struct RunnerOut {
    results: Vec<RunnerRow>,
    preview_png: Option<String>,
    #[serde(default)]
    preview_w: i32,
    #[serde(default)]
    preview_h: i32,
}

#[derive(Deserialize)]
struct RunnerRow {
    image_id: String,
    #[serde(default)]
    pore_fraction: Option<f32>,
    #[serde(default)]
    pixels: Option<i64>,
    #[serde(default)]
    error: Option<String>,
}

/// Whether a plate may be measured by a blue-epoxy rule, and why not when it may not.
///
/// Split out and public so the test suite can pin the refusal without needing Pillow: this is the
/// rule that matters most and the one that must never quietly become a default.
pub fn epoxy_check(prepared: &str) -> Result<(), &'static str> {
    match prepared.trim() {
        "blue_epoxy" => Ok(()),
        "" => Err("preparation not stated - a blue rule on an unimpregnated section returns a porosity anyway"),
        "plain" => Err("not impregnated"),
        _ => Err("preparation is not blue-dyed epoxy"),
    }
}

/// Measures pore area on a well's live image delivery.
pub fn run_pore_area(conn: &Connection, spec: &PoreSpec) -> Result<PoreResult, String> {
    let python = find_python().ok_or("no Python interpreter found (see SANDIBUMI_PYTHON)")?;

    let all = crate::db::list_well_images(conn, &spec.well_id, Some(&spec.dataset))
        .map_err(|e| e.to_string())?;
    if all.is_empty() {
        return Err(format!("no pictures in {} for this well", spec.dataset));
    }

    let mut skipped: Vec<String> = Vec::new();
    let mut wanted = Vec::new();
    for info in &all {
        if let Some(only) = &spec.only_image_id {
            if &info.image_id != only {
                continue;
            }
        }
        // The refusal is BY NAME and counted. A silent subset reads as a complete answer, which is
        // exactly how a half-measured delivery would end up in a report.
        if let Err(why) = epoxy_check(&info.prepared) {
            skipped.push(format!("{}: {}", info.name, why));
            continue;
        }
        wanted.push(info.clone());
    }
    if wanted.is_empty() {
        return Err(format!(
            "no plate in {} is declared as blue-dyed epoxy. Set the impregnation in Plate Details - \
             it is a fact about the section, not something the picture can be asked.",
            spec.dataset
        ));
    }

    let mut plates: Vec<PlatePore> = Vec::new();
    let mut preview_png = None;
    let mut preview_width = 0;
    let mut preview_height = 0;

    for batch in wanted.chunks(CHUNK) {
        let mut blobs = Vec::with_capacity(batch.len());
        for info in batch {
            let (_, bytes) =
                crate::db::get_well_image(conn, &info.image_id).map_err(|e| e.to_string())?;
            blobs.push(bytes);
        }
        let header = serde_json::json!({
            "band": spec.band,
            "ids": batch.iter().map(|i| i.image_id.clone()).collect::<Vec<_>>(),
            "sizes": blobs.iter().map(|b| b.len()).collect::<Vec<_>>(),
            "preview": spec.preview_image_id,
        });

        let mut cmd = Command::new(&python);
        cmd.args(["-c", PORE_RUNNER]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        hide_console(&mut cmd);
        let mut child = cmd.spawn().map_err(|e| format!("failed to start python: {e}"))?;
        {
            let stdin = child.stdin.as_mut().ok_or("failed to open python stdin")?;
            stdin.write_all(header.to_string().as_bytes()).map_err(|e| e.to_string())?;
            stdin.write_all(b"\n").map_err(|e| e.to_string())?;
            for b in &blobs {
                stdin.write_all(b).map_err(|e| e.to_string())?;
            }
        }
        let output = child.wait_with_output().map_err(|e| e.to_string())?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("pore run failed");
            return Err(last.trim().to_string());
        }
        let parsed: RunnerOut = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("bad pore result: {e}"))?;
        if let Some(p) = parsed.preview_png {
            preview_png = Some(p);
            preview_width = parsed.preview_w;
            preview_height = parsed.preview_h;
        }
        for row in parsed.results {
            let Some(info) = batch.iter().find(|i| i.image_id == row.image_id) else { continue };
            match (row.error, row.pore_fraction) {
                (Some(e), _) => skipped.push(format!("{}: {}", info.name, e)),
                (None, Some(f)) => plates.push(PlatePore {
                    image_id: info.image_id.clone(),
                    name: info.name.clone(),
                    depth_top: info.depth_top,
                    depth_base: info.depth_base,
                    pore_fraction: f,
                    pixels: row.pixels.unwrap_or(0),
                }),
                (None, None) => skipped.push(format!("{}: no result", info.name)),
            }
        }
    }

    plates.sort_by(|a, b| a.depth_top.total_cmp(&b.depth_top));

    let mut notes = Vec::new();
    let mut written = None;
    if let Some(set) = &spec.set_name {
        // Point data, not a curve: a thin section measures the one plug it was cut from, and a
        // line drawn between two of them would claim rock nobody looked at.
        let rows: Vec<crate::db::AuxRow> = plates
            .iter()
            .map(|p| crate::db::AuxRow {
                dataset: PORE_DATASET.to_string(),
                depth_top: p.depth_top,
                depth_base: p.depth_base,
                item: PORE_ITEM.to_string(),
                value_num: Some(p.pore_fraction),
                value_text: None,
            })
            .collect();
        let name = crate::db::resolve_aux_set_name(conn, &spec.well_id, PORE_DATASET, set)
            .map_err(|e| e.to_string())?;
        crate::db::insert_aux_data(conn, &spec.well_id, PORE_DATASET, &name, Some(&spec.dataset), &rows)
            .map_err(|e| e.to_string())?;
        written = Some((PORE_DATASET.to_string(), name));
    }
    if !skipped.is_empty() {
        notes.push(format!("{} plate(s) left out - see the list", skipped.len()));
    }
    notes.push(
        "Area fraction estimates volume fraction by the Delesse relation. Where it disagrees with \
         core helium porosity the disagreement is informative: microporosity below the resolution \
         of the section, plucked grains, or epoxy that did not penetrate."
            .to_string(),
    );

    Ok(PoreResult { plates, skipped, preview_png, preview_width, preview_height, written, notes })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The rule the whole feature rests on. A plate whose preparation was never stated must be
    /// REFUSED, not measured, because the measurement succeeds either way — it returns a porosity
    /// built from blue-ish feldspar and edge artefact instead of failing, and nothing downstream
    /// can tell that number from a real one.
    #[test]
    fn a_plate_that_was_never_declared_impregnated_is_refused() {
        assert!(epoxy_check("blue_epoxy").is_ok());
        assert!(epoxy_check("").is_err(), "unknown must never be treated as impregnated");
        assert!(epoxy_check("plain").is_err());
        assert!(epoxy_check("something else").is_err());
    }

    /// The colour band ships as a generic starting point for a visual tuning task, never as a
    /// calibration. Same discipline as `gr_normalize`'s reference percentiles: a two-decimal
    /// threshold would be somebody's regression result, and this has no regression behind it.
    #[test]
    fn the_default_colour_band_is_generic_not_a_calibration() {
        let b = PoreColorBand::default();
        for v in [b.hue_lo, b.hue_hi] {
            assert_eq!(v.fract(), 0.0, "a fractional hue would be a fitted number, not a starting point");
            assert_eq!(v % 10.0, 0.0, "the band is round numbers on purpose");
        }
        assert!(b.hue_lo < b.hue_hi && b.hue_lo >= 150.0 && b.hue_hi <= 280.0, "a plain blue band");
        // Floors low enough to admit pale, thin epoxy: the user tightens them against the preview.
        assert!(b.sat_min > 0.0 && b.sat_min <= 0.2);
        assert!(b.val_min > 0.0 && b.val_min <= 0.2);
    }

    /// A hue band written across 0 degrees is two arcs. Nothing blue needs it, but a user who
    /// types one must not silently measure zero — the runner handles it and this records why.
    #[test]
    fn a_band_written_across_zero_is_not_an_empty_range() {
        assert!(PORE_RUNNER.contains("(h >= lo) | (h <= hi)"));
    }

    /// The real round trip, on a plate whose blue fraction is known exactly by construction.
    /// `#[ignore]`d because it needs Pillow: the green gate must never depend on an optional
    /// package (rule 7), the same reason the office round-trips are ignored.
    ///
    /// The plate is a quarter blue epoxy, with a pale violet patch that a hue test alone would
    /// count — it is the SATURATION floor that rejects it, which is why the floor exists.
    #[test]
    #[ignore]
    fn a_quarter_blue_plate_measures_a_quarter() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-TS", None, None, None).unwrap();
        let w = wid.to_string();

        // ONE delivery holding all three: exactly one set per (well, dataset) is live, so three
        // separate sets would leave only the last one visible.
        let png = synthetic_plate();
        let plates: Vec<crate::db::NewImage> = [("TS-1", "blue_epoxy"), ("TS-2", ""), ("TS-3", "plain")]
            .iter()
            .enumerate()
            .map(|(i, (name, prepared))| crate::db::NewImage {
                depth_top: 2000.0 + i as f32,
                name: (*name).to_string(),
                mime: "image/bmp".into(),
                width: 200,
                height: 200,
                data: png.clone(),
                printable: true,
                prepared: if prepared.is_empty() { None } else { Some((*prepared).into()) },
                ..Default::default()
            })
            .collect();
        crate::db::insert_well_images(&conn, &w, "THIN SECTION", "LAB", None, &plates).unwrap();

        let spec = PoreSpec {
            well_id: w.clone(),
            dataset: "THIN SECTION".into(),
            band: PoreColorBand::default(),
            preview_image_id: None,
            only_image_id: None,
            set_name: Some("TS".into()),
        };
        let res = run_pore_area(&conn, &spec).expect("pore run");

        // One declared plate measured, two refused BY NAME — a silent subset would read as a
        // complete answer.
        assert_eq!(res.plates.len(), 1, "only the declared plate is measured");
        assert_eq!(res.plates[0].name, "TS-1");
        assert!(
            (res.plates[0].pore_fraction - 0.25).abs() < 1e-4,
            "a quarter-blue plate came to {}",
            res.plates[0].pore_fraction
        );
        assert_eq!(res.plates[0].pixels, 40_000);
        assert_eq!(res.skipped.len(), 2);
        assert!(res.skipped.iter().any(|s| s.starts_with("TS-2") && s.contains("not stated")));
        assert!(res.skipped.iter().any(|s| s.starts_with("TS-3") && s.contains("not impregnated")));

        // Stored as point data under its own dataset, at the plate's depth.
        let (ds, set) = res.written.clone().expect("written");
        assert_eq!(ds, PORE_DATASET);
        let rows = crate::db::list_aux_data(&conn, &w, Some(PORE_DATASET)).unwrap();
        assert_eq!(rows.len(), 1, "one measured plate, one point sample (set {set})");
        assert_eq!(rows[0].item, PORE_ITEM);
        assert!((rows[0].value_num.unwrap() - 0.25).abs() < 1e-4);
        assert!((rows[0].depth_top - 2000.0).abs() < 1e-4);
    }

    /// 200x200: a quarter pure blue epoxy, the rest grey, plus a pale violet square whose hue is
    /// in the band and whose saturation is not.
    #[cfg(test)]
    fn synthetic_plate() -> Vec<u8> {
        // Written as a raw PNG through Pillow would need Python; instead the test is ignored and
        // the fixture is produced by the same runner's dependency. Kept as an uncompressed BMP,
        // which Pillow reads and which needs no encoder here.
        let (w, h) = (200usize, 200usize);
        let row = w * 3;
        let pad = (4 - row % 4) % 4;
        let stride = row + pad;
        let pixels = stride * h;
        let mut out = Vec::with_capacity(54 + pixels);
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&((54 + pixels) as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&54u32.to_le_bytes());
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&(w as i32).to_le_bytes());
        out.extend_from_slice(&(h as i32).to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&24u16.to_le_bytes());
        for _ in 0..6 {
            out.extend_from_slice(&0u32.to_le_bytes());
        }
        for y in 0..h {
            let yy = h - 1 - y; // BMP rows run bottom-up
            for x in 0..w {
                let (r, g, b) = if yy < 100 && x < 100 {
                    (32u8, 64u8, 192u8) // blue epoxy, a quarter of the plate
                } else if yy >= 150 && x >= 150 {
                    (200, 196, 225) // pale violet: right hue, not enough saturation
                } else {
                    (180, 178, 172) // grey matrix
                };
                out.extend_from_slice(&[b, g, r]);
            }
            out.extend(std::iter::repeat(0u8).take(pad));
        }
        out
    }

    /// Results are point data at the plug depth, under their own dataset. Re-running a measurement
    /// must not look like a second delivery of pictures.
    #[test]
    fn the_measurement_is_its_own_point_dataset() {
        assert_eq!(PORE_DATASET, "PETROGRAPHY");
        assert_ne!(PORE_DATASET, "THIN SECTION");
        assert_eq!(PORE_ITEM, "VPORE_TS");
    }
}
