//! Core-to-log depth registration: proposing the constant shift that puts a well's core back
//! on the log's depth scale.
//!
//! **Nothing here writes.** It reads the core and the log, reports how well they agree at every
//! candidate shift, and proposes the best one. The user accepts it, and the write goes through
//! `db::shift_core_depths` — which moves the plugs and their extras together. A correlation
//! maximum is a suggestion; in a repetitive sand section it can be confidently wrong, which is
//! why the whole correlogram is returned rather than only its peak (see [`RegistrationResult::scan`]).
//!
//! ## This is not a new algorithm
//!
//! Matching a core profile against a wireline log is the same problem `tops.rs` already solves to
//! propagate a marker between wells: slide one series over the other and keep the lag of maximum
//! Pearson correlation. That code is written and tested, and this module borrows its two
//! primitives (`tops::interp`, `tops::pearson`) rather than growing a second implementation.
//!
//! ## The reference, and why its strength is reported
//!
//! Core gamma is not always delivered. When it is, comparing it against the wireline GR is
//! **like-for-like**: two measurements of the same physical quantity, which should agree in sign
//! as well as in shape, and a negative correlation between them is not an alignment at all — it
//! is nonsense, and the search must never select it.
//!
//! When it is not delivered, registration has to fall back on a **proxy**: core porosity against
//! GR, say. Those are different quantities that happen to co-vary — and they co-vary
//! *inversely*, because the shaly intervals that raise GR are the ones that lose porosity. A
//! search that maximised the signed correlation on such a pair would systematically land on the
//! WORST alignment. So the rule is:
//!
//! - like-for-like → maximise **r**;
//! - proxy → maximise **|r|**, and report which sign won.
//!
//! The result says which of the two it did, because a coefficient of −0.82 means "well aligned"
//! in one case and "something is wrong" in the other, and a number that reads the same in both
//! situations is a number that misleads.

use std::sync::Mutex;

use duckdb::Connection;
use serde::{Deserialize, Serialize};

use crate::equations::fetch_curve_frame;
use crate::tops::{interp, pearson};

/// Fewest core samples that may produce a correlation at all. `pearson` itself refuses below 4;
/// this is deliberately higher, because a shift chosen from five plugs is a coincidence with a
/// coefficient attached.
const MIN_PAIRS: usize = 8;

/// A candidate shift must keep at least this fraction of the best-populated shift's pairs.
/// Without it, sliding the core off the end of the log is a legitimate way to win: the few plugs
/// that still overlap can correlate almost perfectly by chance, and the scan would return a large
/// shift with a beautiful coefficient computed from almost no data.
const MIN_PAIR_FRACTION: f32 = 0.75;

// ---------------------------------------------------------------------------
// What a well can be registered on
// ---------------------------------------------------------------------------

/// One thing in this well that carries a value at core depths and could anchor a shift.
#[derive(Debug, Clone, Serialize)]
pub struct CoreReference {
    /// `"core"` = a column of the plug table; `"aux"` = an item of a point dataset.
    pub kind: String,
    /// Point dataset the item belongs to; empty for a plug column.
    pub dataset: String,
    pub item: String,
    /// Human label for the picker.
    pub label: String,
    /// Numeric samples available (blank cells never counted).
    pub n: usize,
    /// Resolved family, or empty when the name is not recognised. Used to decide whether a
    /// pairing is like-for-like; an unrecognised name is treated as a proxy, never guessed.
    pub family: String,
}

/// Families this module understands beyond `curves::FAMILIES`.
///
/// The curve dictionary covers what arrives in a LAS — GR, RHOB, DT, the resistivities. It has no
/// porosity or permeability family because the generic curve store never needed one, and widening
/// it here would change curve resolution for the whole project to settle a labelling question.
/// So the core-side names live in this local table instead.
const CORE_FAMILIES: &[(&str, &[&str])] = &[
    ("POR", &["PHI", "PHIE", "PHIT", "POR", "CPOR", "PORO", "POROSITY", "HEPOR", "PHIH", "PHIA"]),
    ("PERM", &["K", "PERM", "CPERM", "KAIR", "KLIN", "KH", "KV", "PERMEABILITY"]),
    ("GD", &["GD", "CGD", "RHOG", "RHOMA", "GRAINDENSITY", "GRDEN"]),
    ("SW", &["SW", "CSW", "SWT", "SWE", "SWIRR"]),
];

/// Strips the words that mark a name as core-side (`CORE_GR` and `GR` are the same measurement)
/// and returns the bare mnemonic, uppercased and punctuation-free.
fn bare_mnemonic(name: &str) -> String {
    let upper = name.trim().to_uppercase();
    let parts: Vec<&str> = upper
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty() && *t != "CORE" && *t != "PLUG" && *t != "LAB")
        .collect();
    parts.join("")
}

/// Family for either side of the pairing: the project curve dictionary first, then the
/// core-side table above. `None` means unrecognised — which downgrades the pairing to a proxy
/// rather than inventing a match.
pub fn reference_family(name: &str) -> Option<&'static str> {
    let bare = bare_mnemonic(name);
    if bare.is_empty() {
        return None;
    }
    if let Some(f) = crate::curves::family_for(&bare) {
        return Some(f.family);
    }
    CORE_FAMILIES
        .iter()
        .find(|(_, aliases)| aliases.iter().any(|a| *a == bare))
        .map(|(family, _)| *family)
}

/// Everything in this well that could anchor a registration: the plug table's own numeric
/// columns, then every numeric item of every point dataset (which is where a delivered core
/// gamma lands, whether it came as its own dataset or as an extra column of the core table).
pub fn list_core_references(conn: &Connection, well_id: &str) -> Result<Vec<CoreReference>, String> {
    let mut out: Vec<CoreReference> = Vec::new();

    let plugs = crate::db::get_core_point_series(conn, well_id).map_err(|e| e.to_string())?;
    let mut seen: Vec<(String, usize)> = Vec::new();
    for (item, _, _) in &plugs {
        match seen.iter_mut().find(|(n, _)| n == item) {
            Some((_, c)) => *c += 1,
            None => seen.push((item.clone(), 1)),
        }
    }
    for (item, n) in seen {
        out.push(CoreReference {
            kind: "core".into(),
            dataset: String::new(),
            label: format!("Core plugs — {item} ({n} sample(s))"),
            family: reference_family(&item).unwrap_or_default().into(),
            item,
            n,
        });
    }

    let aux = crate::db::list_aux_data(conn, well_id, None).map_err(|e| e.to_string())?;
    let mut keys: Vec<(String, String, usize)> = Vec::new();
    for row in &aux {
        if row.value_num.map(|v| v.is_finite()) != Some(true) {
            continue; // a description cannot anchor a depth shift
        }
        match keys.iter_mut().find(|(d, i, _)| *d == row.dataset && *i == row.item) {
            Some((_, _, c)) => *c += 1,
            None => keys.push((row.dataset.clone(), row.item.clone(), 1)),
        }
    }
    for (dataset, item, n) in keys {
        out.push(CoreReference {
            kind: "aux".into(),
            label: format!("{dataset} — {item} ({n} sample(s))"),
            family: reference_family(&item).unwrap_or_default().into(),
            dataset,
            item,
            n,
        });
    }

    // The core photograph's own proxy trace. Not a general curve-vs-curve registration: these are
    // the only curves in the project that are MEASURED ON THE CORE, so they carry the core's depth
    // error and a shift found from them is a shift for the plugs.
    //
    // And they are the best reference this dialog has. A plug table gives a few dozen samples a
    // foot apart; a photograph gives a reading every few millimetres down the whole cored interval,
    // which is what a cross-correlation wants — the same reason a wireline log is the thing being
    // registered against rather than a set of picks.
    if let Ok(mut stmt) = conn.prepare(
        "SELECT curve_name, COUNT(*) FROM computed_curves
          WHERE well_id = ?1 AND curve_name LIKE 'CPHOTO%'
          GROUP BY curve_name ORDER BY curve_name",
    ) {
        if let Ok(rows) = stmt.query_map(duckdb::params![well_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as usize))
        }) {
            for row in rows.flatten() {
                let (item, n) = row;
                out.push(CoreReference {
                    kind: "curve".into(),
                    dataset: String::new(),
                    label: format!("Core photo — {item} ({n} sample(s))"),
                    family: reference_family(&item).unwrap_or_default().into(),
                    item,
                    n,
                });
            }
        }
    }

    out.retain(|r| r.n >= MIN_PAIRS);
    Ok(out)
}

/// References whose SIGN against a shale indicator is known, unlike a general proxy.
///
/// A photograph's darkness is not a gamma reading, so the shift is still chosen on |r| like any
/// other proxy — two different quantities have no business being forced onto one line. But the
/// expected sign is not a mystery: clay is dark and clay is radioactive, so both rise into shale.
/// A winning peak that is NEGATIVE therefore says the box is laid out the other way up, and
/// Register Depth cannot tell that apart from a genuine shift. Accepting it would bake an
/// upside-down photograph into the core's depths, where nothing downstream could find it.
fn expects_to_rise_with_shale(item: &str) -> bool {
    item.eq_ignore_ascii_case("CPHOTO_DARK")
}

// ---------------------------------------------------------------------------
// The proposal
// ---------------------------------------------------------------------------

fn default_search() -> f32 {
    5.0
}
fn default_step() -> f32 {
    0.05
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistrationRequest {
    pub well_id: String,
    /// Wireline curve to register against (generic-store aware, per rule 11).
    pub log_curve: String,
    /// `"core"` or `"aux"`, matching [`CoreReference::kind`].
    pub ref_kind: String,
    #[serde(default)]
    pub ref_dataset: String,
    pub ref_item: String,
    /// Restrict the comparison to an interval — normally the cored interval, or one core run.
    #[serde(default)]
    pub depth_from: Option<f32>,
    #[serde(default)]
    pub depth_to: Option<f32>,
    /// Half-width of the shift search, in project depth units.
    #[serde(default = "default_search")]
    pub search_range: f32,
    #[serde(default = "default_step")]
    pub step: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegPoint {
    pub depth: f32,
    pub value: f32,
}

/// One rung of the correlogram: how well the two series agree if the core moved by `delta`.
#[derive(Debug, Clone, Serialize)]
pub struct LagPoint {
    pub delta: f32,
    pub r: f32,
    pub n: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistrationResult {
    /// Core samples at their CURRENT depths — the proposal is drawn by offsetting these, so the
    /// caller can show both without a second round trip.
    pub core: Vec<RegPoint>,
    pub log_depth: Vec<f32>,
    pub log_value: Vec<f32>,
    pub proposed_delta: f32,
    /// Signed correlation at the proposed shift.
    pub correlation: f32,
    /// Signed correlation where the core sits now, so "did this improve anything?" is answerable.
    pub current_r: f32,
    pub n_pairs: usize,
    /// True when both sides resolved to the SAME family — see the module note.
    pub like_for_like: bool,
    /// `"direct"` or `"inverse"`: which sign the winning correlation had.
    pub matched_on: String,
    pub log_family: String,
    pub ref_family: String,
    pub reference_label: String,
    pub scan: Vec<LagPoint>,
    pub notes: Vec<String>,
    pub error: Option<String>,
}

/// Everything in [`RegistrationResult`] EXCEPT the per-sample arrays, which travel as packed `f32`
/// columns beside it.
///
/// **`scan` stays in the header on purpose.** It is one rung per LAG, bounded by the scan range and
/// step rather than by the log, so it is metadata-sized — and each rung carries an integer PAIR
/// COUNT. Squeezing a count through `f32` is exact only to 2^24, and a pair count that silently
/// rounds is precisely the kind of quiet wrongness the packed path exists to avoid. Bytes are for
/// measurements, not for counts.
#[derive(Debug, Clone, Serialize)]
struct RegistrationHeader<'a> {
    columns: Vec<&'a str>,
    n_core: usize,
    n_log: usize,
    proposed_delta: f32,
    correlation: f32,
    current_r: f32,
    n_pairs: usize,
    like_for_like: bool,
    matched_on: &'a str,
    log_family: &'a str,
    ref_family: &'a str,
    reference_label: &'a str,
    scan: &'a [LagPoint],
    notes: &'a [String],
    error: Option<&'a str>,
}

/// Packs one envelope for the IPC bridge: scalars, the lag scan and notes as JSON, every per-sample
/// array as raw `f32`.
///
/// Rule 3. `log_depth`/`log_value` are the WELL's full vectors — the arrays the contract is about —
/// and the core columns ride the same envelope rather than a second JSON channel, because a
/// fully cored well is thousands of plugs and because one mechanism needs no per-array judgement
/// about which is "big enough" to deserve bytes.
pub fn pack_registration(res: &RegistrationResult) -> Result<Vec<u8>, String> {
    let (core_depth, core_value): (Vec<f32>, Vec<f32>) =
        res.core.iter().map(|p| (p.depth, p.value)).unzip();
    let header = RegistrationHeader {
        columns: vec!["core_depth", "core_value", "log_depth", "log_value"],
        n_core: core_depth.len(),
        n_log: res.log_depth.len(),
        proposed_delta: res.proposed_delta,
        correlation: res.correlation,
        current_r: res.current_r,
        n_pairs: res.n_pairs,
        like_for_like: res.like_for_like,
        matched_on: &res.matched_on,
        log_family: &res.log_family,
        ref_family: &res.ref_family,
        reference_label: &res.reference_label,
        scan: &res.scan,
        notes: &res.notes,
        error: res.error.as_deref(),
    };
    let json = serde_json::to_string(&header).map_err(|e| e.to_string())?;
    Ok(crate::equations::pack_frame(
        &json,
        &[&core_depth, &core_value, &res.log_depth, &res.log_value],
    ))
}

fn fail(msg: impl Into<String>) -> RegistrationResult {
    RegistrationResult {
        core: Vec::new(),
        log_depth: Vec::new(),
        log_value: Vec::new(),
        proposed_delta: 0.0,
        correlation: f32::NAN,
        current_r: f32::NAN,
        n_pairs: 0,
        like_for_like: false,
        matched_on: "direct".into(),
        log_family: String::new(),
        ref_family: String::new(),
        reference_label: String::new(),
        scan: Vec::new(),
        notes: Vec::new(),
        error: Some(msg.into()),
    }
}

/// Correlation between the core values and the log sampled at each plug's depth **plus `delta`**.
///
/// The log is interpolated onto the core depths rather than the core resampled onto the log:
/// core is sparse and irregular, and resampling it would invent samples between plugs that would
/// then vote on the answer.
fn r_at(core: &[RegPoint], log_d: &[f32], log_v: &[f32], delta: f32) -> (f32, usize) {
    let mut a = Vec::with_capacity(core.len());
    let mut b = Vec::with_capacity(core.len());
    for p in core {
        a.push(p.value);
        b.push(interp(log_d, log_v, p.depth + delta));
    }
    pearson(&a, &b)
}

pub fn propose_registration(db_mx: &Mutex<Connection>, req: &RegistrationRequest) -> RegistrationResult {
    let conn = match db_mx.lock() {
        Ok(c) => c,
        Err(_) => return fail("database is busy"),
    };

    // --- The core side -----------------------------------------------------
    let mut core: Vec<RegPoint> = Vec::new();
    let reference_label;
    if req.ref_kind == "core" {
        reference_label = format!("core {}", req.ref_item);
        let rows = match crate::db::get_core_point_series(&conn, &req.well_id) {
            Ok(r) => r,
            Err(e) => return fail(e.to_string()),
        };
        for (item, depth, value) in rows {
            if item.eq_ignore_ascii_case(&req.ref_item) && value.is_finite() {
                core.push(RegPoint { depth, value });
            }
        }
    } else if req.ref_kind == "curve" {
        reference_label = req.ref_item.clone();
        match fetch_curve_frame(&conn, &req.well_id, &[req.ref_item.clone()]) {
            Ok((d, map)) => match map.get(&req.ref_item.to_uppercase()) {
                Some(v) if v.len() == d.len() => {
                    for (depth, value) in d.iter().zip(v.iter()) {
                        if value.is_finite() {
                            core.push(RegPoint { depth: *depth, value: *value });
                        }
                    }
                }
                _ => return fail(format!("the well carries no curve called {}", req.ref_item)),
            },
            Err(e) => return fail(e.to_string()),
        }
    } else {
        reference_label = format!("{} {}", req.ref_dataset, req.ref_item);
        let rows = match crate::db::list_aux_data(&conn, &req.well_id, Some(&req.ref_dataset)) {
            Ok(r) => r,
            Err(e) => return fail(e.to_string()),
        };
        for row in rows {
            if !row.item.eq_ignore_ascii_case(&req.ref_item) {
                continue;
            }
            let Some(v) = row.value_num.filter(|v| v.is_finite()) else { continue };
            // An interval sample is anchored at its middle, matching the point-track rule.
            let depth = match row.depth_base {
                Some(base) if base.is_finite() => 0.5 * (row.depth_top + base),
                _ => row.depth_top,
            };
            core.push(RegPoint { depth, value: v });
        }
    }
    if let Some(d) = req.depth_from {
        core.retain(|p| p.depth >= d);
    }
    if let Some(d) = req.depth_to {
        core.retain(|p| p.depth <= d);
    }
    core.sort_by(|a, b| a.depth.total_cmp(&b.depth));
    if core.len() < MIN_PAIRS {
        return fail(format!(
            "only {} numeric sample(s) of {reference_label} in that interval — {MIN_PAIRS} are needed \
             before a correlation means anything",
            core.len()
        ));
    }

    // --- The log side ------------------------------------------------------
    let (log_d, map) = match fetch_curve_frame(&conn, &req.well_id, &[req.log_curve.clone()]) {
        Ok(f) => f,
        Err(e) => return fail(e.to_string()),
    };
    let Some(log_v) = map.get(&req.log_curve).cloned() else {
        return fail(format!("the well carries no curve called {}", req.log_curve));
    };
    if log_d.len() < 2 {
        return fail(format!("{} has no samples in this well", req.log_curve));
    }
    drop(conn);

    // --- The pairing, and what kind of pairing it is -----------------------
    let log_family = reference_family(&req.log_curve).unwrap_or_default();
    let ref_family = reference_family(&req.ref_item).unwrap_or_default();
    let like_for_like = !log_family.is_empty() && log_family == ref_family;

    let mut notes: Vec<String> = Vec::new();
    if like_for_like {
        notes.push(format!(
            "Like-for-like: {reference_label} and {} are both {log_family}, so the shift is chosen \
             on the correlation itself — a negative correlation between two measurements of the \
             same quantity is not an alignment.",
            req.log_curve
        ));
    } else {
        notes.push(format!(
            "Proxy pairing: {reference_label} and {} measure different things, so the shift is \
             chosen on the STRENGTH of the relationship and not its sign. Read the coefficient as \
             a shape match, not as agreement.",
            req.log_curve
        ));
    }

    // --- The scan ----------------------------------------------------------
    let step = if req.step.is_finite() && req.step > 0.0 { req.step } else { default_step() };
    let range = if req.search_range.is_finite() && req.search_range > 0.0 {
        req.search_range
    } else {
        default_search()
    };
    let steps = (range / step).round().max(1.0) as i32;

    let mut scan: Vec<LagPoint> = Vec::with_capacity(2 * steps as usize + 1);
    for k in -steps..=steps {
        let delta = k as f32 * step;
        let (r, n) = r_at(&core, &log_d, &log_v, delta);
        scan.push(LagPoint { delta, r, n });
    }
    let n_max = scan.iter().map(|p| p.n).max().unwrap_or(0);
    let n_floor = MIN_PAIRS.max((MIN_PAIR_FRACTION * n_max as f32).round() as usize);

    let (current_r, _) = r_at(&core, &log_d, &log_v, 0.0);

    let mut best: Option<&LagPoint> = None;
    for p in &scan {
        if p.n < n_floor || !p.r.is_finite() {
            continue;
        }
        let score = if like_for_like { p.r } else { p.r.abs() };
        let better = match best {
            None => true,
            Some(b) => score > if like_for_like { b.r } else { b.r.abs() },
        };
        if better {
            best = Some(p);
        }
    }
    let Some(best) = best else {
        return fail(format!(
            "no shift in ±{range} kept at least {n_floor} paired sample(s) — widen the interval or \
             narrow the search"
        ));
    };

    if best.n < n_max {
        notes.push(format!(
            "The proposed shift pairs {} of the {n_max} samples available at the best-populated \
             shift; the rest fall outside the logged interval once moved.",
            best.n
        ));
    }
    if !like_for_like && best.r < 0.0 {
        if expects_to_rise_with_shale(&req.ref_item) {
            // The one proxy in this dialog whose sign is not a mystery — see the helper.
            notes.push(format!(
                "The best match is NEGATIVE ({:+.2}), and darkness should RISE with {} because clay \
                 is both dark and radioactive. That almost always means the photographs are laid \
                 out the other way up rather than that the core is shifted - try Deepest end first \
                 in Condition Core Photos, save the trace again, and re-run this. Do not accept \
                 this shift until the sign is positive: a registration is not able to tell an \
                 upside-down box from a genuine depth error.",
                best.r, req.log_curve
            ));
        } else {
            notes.push(
                "Matched on an INVERSE relationship, which is what a porosity-type reference \
                 against a gamma-type log should show. If you expected them to rise together, the \
                 reference is probably not the one you meant."
                    .into(),
            );
        }
    }
    if best.r.abs() < 0.5 {
        notes.push(format!(
            "|r| is only {:.2} at the best shift. That is a weak match: check the interval, or \
             register on a different reference before accepting this.",
            best.r
        ));
    }
    let peak = best.r.abs();
    let rivals = scan
        .iter()
        .filter(|p| p.n >= n_floor && p.r.is_finite())
        .filter(|p| (p.delta - best.delta).abs() > 5.0 * step && p.r.abs() >= 0.95 * peak)
        .count();
    if rivals > 0 {
        notes.push(format!(
            "{rivals} other shift(s) score within 5% of this one. A repeated sand can correlate \
             almost as well in the wrong place — read the correlogram before accepting."
        ));
    }
    RegistrationResult {
        proposed_delta: best.delta,
        correlation: best.r,
        current_r,
        n_pairs: best.n,
        matched_on: if best.r < 0.0 { "inverse".into() } else { "direct".into() },
        like_for_like,
        log_family: log_family.into(),
        ref_family: ref_family.into(),
        reference_label,
        core,
        log_depth: log_d,
        log_value: log_v,
        scan,
        notes,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use uuid::Uuid;

    fn mem() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        conn
    }

    /// A GR shape with no period short enough to repeat inside the search window, so there is
    /// exactly one right answer. Two incommensurate sinusoids: the beat length is far longer
    /// than the ±5 m searched, which is what makes the "one good alignment" claim true rather
    /// than hopeful.
    fn shape(d: f32) -> f32 {
        60.0 + 25.0 * (d * 0.31).sin() + 15.0 * (d * 0.11).sin()
    }

    /// A well logged 1000–1100 m at ~6 inch sampling, with plugs every 0.5 m over 1020–1080
    /// written `offset` too deep — i.e. a core that must be moved by `-offset` to line up.
    ///
    /// `invert` writes the plug value as a porosity-like quantity that FALLS as the gamma rises,
    /// which is what a real φ-against-GR pairing looks like.
    fn synth(conn: &Connection, offset: f32, invert: bool) -> (String, Vec<f32>, Vec<f32>) {
        let w = Uuid::new_v4();
        db::insert_well(conn, w, "SANDI-REG", None, None, None).unwrap();
        let id = w.to_string();

        let mut depth = Vec::new();
        let mut gr = Vec::new();
        let mut d = 1000.0f32;
        while d <= 1100.0 {
            depth.push(d);
            gr.push(shape(d));
            d += 0.1524;
        }
        let filler = vec![f32::NAN; depth.len()];
        db::insert_standard_curves(
            conn,
            w,
            depth.clone(),
            gr.clone(),
            filler.clone(),
            filler.clone(),
            filler.clone(),
            filler.clone(),
            filler,
        )
        .unwrap();

        let mut pdepth = Vec::new();
        let mut pval = Vec::new();
        let mut pd = 1020.0f32;
        while pd <= 1080.0 {
            pdepth.push(pd + offset);
            pval.push(if invert { 0.30 - 0.0025 * (shape(pd) - 60.0) } else { shape(pd) });
            pd += 0.5;
        }
        let n = pdepth.len();
        let nan = vec![f32::NAN; n];
        db::insert_core_data(conn, &id, "RAW", None, &pdepth, &pval, &nan, &nan, &nan).unwrap();
        (id, pdepth, pval)
    }

    /// Writes the plug values as a point dataset item too, so the same numbers can be offered
    /// as either a plug column or a delivered core-gamma curve.
    fn as_aux(conn: &Connection, well: &str, dataset: &str, item: &str, depths: &[f32], vals: &[f32]) {
        let rows: Vec<db::AuxRow> = depths
            .iter()
            .zip(vals)
            .map(|(&d, &v)| db::AuxRow {
                dataset: dataset.into(),
                depth_top: d,
                depth_base: None,
                item: item.into(),
                value_num: Some(v),
                value_text: None,
            })
            .collect();
        db::insert_aux_data(conn, well, dataset, "RAW", None, &rows).unwrap();
    }

    fn req(well: &str, kind: &str, dataset: &str, item: &str) -> RegistrationRequest {
        RegistrationRequest {
            well_id: well.into(),
            log_curve: "GR".into(),
            ref_kind: kind.into(),
            ref_dataset: dataset.into(),
            ref_item: item.into(),
            depth_from: None,
            depth_to: None,
            search_range: 5.0,
            step: 0.05,
        }
    }

    #[test]
    fn bare_mnemonic_sees_through_the_core_prefix() {
        assert_eq!(bare_mnemonic("CORE_GR"), "GR");
        assert_eq!(bare_mnemonic("core gamma"), "GAMMA");
        assert_eq!(bare_mnemonic("GR-CORE"), "GR");
        assert_eq!(bare_mnemonic("CPOR"), "CPOR");
    }

    #[test]
    fn a_core_gamma_against_the_wireline_gr_is_like_for_like() {
        assert_eq!(reference_family("CORE_GR"), Some("GR"));
        assert_eq!(reference_family("GR"), Some("GR"));
        // …while a porosity reference against the same log is not.
        assert_eq!(reference_family("CPOR"), Some("POR"));
        assert_ne!(reference_family("CPOR"), reference_family("GR"));
    }

    #[test]
    fn an_unrecognised_name_is_a_proxy_rather_than_a_guessed_match() {
        assert_eq!(reference_family("SAMPLE_ID_7"), None);
    }

    /// The straightforward case Jauhar has "sometimes": a delivered core gamma, which is a
    /// measurement of the same quantity as the log and recovers the error directly.
    #[test]
    fn a_delivered_core_gamma_recovers_the_depth_error() {
        let conn = mem();
        let (well, depths, vals) = synth(&conn, 2.0, false);
        as_aux(&conn, &well, "CORE GAMMA", "CORE_GR", &depths, &vals);
        let db = Mutex::new(conn);

        let res = propose_registration(&db, &req(&well, "aux", "CORE GAMMA", "CORE_GR"));
        assert!(res.error.is_none(), "{:?}", res.error);
        assert!(res.like_for_like, "CORE_GR against GR is the same family");
        assert_eq!(res.matched_on, "direct");
        assert!(
            (res.proposed_delta + 2.0).abs() < 0.1,
            "should propose about -2 m, got {}",
            res.proposed_delta
        );
        assert!(res.correlation > 0.95, "r = {}", res.correlation);
        assert!(
            res.current_r < res.correlation,
            "the proposal must beat where the core sits now ({} vs {})",
            res.current_r,
            res.correlation
        );
    }

    /// The core photograph's own trace is the densest reference this dialog has — and the one
    /// proxy whose SIGN is not a mystery.
    ///
    /// Two claims. **A `CPHOTO_*` curve can anchor a registration at all**: those curves are the
    /// only ones in the project measured ON the core, so they carry the core's depth error, and a
    /// photograph gives a reading every few millimetres where a plug table gives a few dozen a foot
    /// apart. **And a negative winning peak is refused in words rather than accepted quietly.**
    /// Darkness rises with gamma because clay is both dark and radioactive, so a negative best
    /// match means the box is laid out the other way up — which a correlogram cannot tell apart
    /// from a genuine shift, and accepting it would bake an upside-down photograph into the core's
    /// depths where nothing downstream could find it.
    #[test]
    fn the_photograph_trace_can_anchor_a_shift_and_says_when_the_box_is_upside_down() {
        for upside_down in [false, true] {
            let conn = mem();
            let (well, _, _) = synth(&conn, 2.0, false);
            // The trace as `extract_core_log` now stores it: on the WELL'S depth frame, carrying
            // the core's own depth error — the photograph was taken of the core, so a feature the
            // log shows at 1030 appears in the picture at the depth the core report gave it, 1032.
            let (frame, map) = fetch_curve_frame(&conn, &well, &["GR".into()]).unwrap();
            let gr = map.get("GR").cloned().unwrap();
            let dark: Vec<f32> = frame
                .iter()
                .map(|d| {
                    let v = (crate::tops::interp(&frame, &gr, d - 2.0) - 40.0) / 120.0;
                    // Upside down: the same shale reads LIGHT. What a box read the wrong way up
                    // produces against this log, and the case the note has to catch.
                    if upside_down { 1.0 - v } else { v }
                })
                .collect();
            let refs: Vec<(&str, &[f32])> = vec![("CPHOTO_DARK", dark.as_slice())];
            crate::equations::write_computed_curves_batch(&conn, &well, &frame, &refs).unwrap();

            let listed = list_core_references(&conn, &well).unwrap();
            let mine = listed.iter().find(|r| r.item == "CPHOTO_DARK").expect("offered as a reference");
            assert_eq!(mine.kind, "curve");
            assert!(mine.label.contains("Core photo"), "{}", mine.label);
            assert!(mine.family.is_empty(), "darkness is not a gamma reading — it must stay a proxy");

            let db = Mutex::new(conn);
            let res = propose_registration(&db, &req(&well, "curve", "", "CPHOTO_DARK"));
            assert!(res.error.is_none(), "{:?}", res.error);
            assert!(!res.like_for_like);

            let warned = res.notes.iter().any(|n| n.contains("other way up"));
            if upside_down {
                assert_eq!(res.matched_on, "inverse", "r = {}", res.correlation);
                assert!(
                    warned,
                    "an inverted photograph must be named, not accepted: {:?}",
                    res.notes
                );
            } else {
                assert_eq!(res.matched_on, "direct", "r = {}", res.correlation);
                assert!(res.correlation > 0.95, "r = {}", res.correlation);
                assert!(
                    (res.proposed_delta + 2.0).abs() < 0.2,
                    "should recover about -2 m, got {}",
                    res.proposed_delta
                );
                assert!(!warned, "nothing to warn about here: {:?}", res.notes);
            }
        }
    }

    /// The case he has the rest of the time. Core porosity against GR is a PROXY, and the two
    /// run in opposite directions — so a search that maximised the signed correlation would
    /// walk away from the true depth. This test is the reason `|r|` is the proxy score: with a
    /// signed score it fails.
    #[test]
    fn a_porosity_proxy_registers_on_the_inverse_relationship() {
        let conn = mem();
        let (well, _, _) = synth(&conn, 2.0, true);
        let db = Mutex::new(conn);

        let res = propose_registration(&db, &req(&well, "core", "", "CPOR"));
        assert!(res.error.is_none(), "{:?}", res.error);
        assert!(!res.like_for_like, "porosity against gamma is not like-for-like");
        assert_eq!(res.matched_on, "inverse");
        assert!(
            (res.proposed_delta + 2.0).abs() < 0.1,
            "should still propose about -2 m, got {}",
            res.proposed_delta
        );
        assert!(res.correlation < -0.95, "r = {}", res.correlation);
        assert!(
            res.notes.iter().any(|n| n.contains("INVERSE")),
            "the inverse match must be stated, not left for the reader to notice: {:?}",
            res.notes
        );
    }

    /// The other half of the same rule, and the one that is easy to get wrong: a like-for-like
    /// pairing must NEVER accept an anti-aligned peak, however strong. Two gamma measurements
    /// that run opposite are not aligned; they are wrong.
    #[test]
    fn a_like_for_like_pairing_never_accepts_an_inverted_alignment() {
        let conn = mem();
        // Plug values inverted, but NAMED as a core gamma — so the pairing is like-for-like
        // while the data anti-correlates. |r| would find a beautiful negative peak here.
        let (well, depths, vals) = synth(&conn, 2.0, true);
        as_aux(&conn, &well, "CORE GAMMA", "CORE_GR", &depths, &vals);
        let db = Mutex::new(conn);

        let res = propose_registration(&db, &req(&well, "aux", "CORE GAMMA", "CORE_GR"));
        assert!(res.error.is_none(), "{:?}", res.error);
        assert!(res.like_for_like);
        assert!(
            res.correlation >= 0.0,
            "a like-for-like search must not return a negative correlation, got {}",
            res.correlation
        );
        assert_eq!(res.matched_on, "direct");
        // And because the data really is anti-correlated, the best DIRECT match is elsewhere —
        // which is the honest outcome: the user is shown a shift that does not match the
        // porosity answer, and that disagreement is the signal.
        assert!(
            (res.proposed_delta + 2.0).abs() > 0.2,
            "the true depth should NOT be the direct-correlation winner for inverted data"
        );
    }

    /// The correlogram is returned, not just its peak, and it is dense enough to read.
    #[test]
    fn the_whole_correlogram_comes_back_so_a_weak_peak_is_visible() {
        let conn = mem();
        let (well, _, _) = synth(&conn, 1.0, false);
        let db = Mutex::new(conn);

        let res = propose_registration(&db, &req(&well, "core", "", "CPOR"));
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.scan.len(), 201, "±5 m at 0.05 m = 201 rungs");
        assert!(res.scan.iter().any(|p| p.delta == 0.0));
        assert!(
            res.scan.iter().all(|p| p.n <= res.core.len()),
            "a shift can never pair more samples than the core has"
        );
        assert!(res.n_pairs >= MIN_PAIRS);
    }

    /// A reference must carry enough numeric samples to mean something, and a description
    /// carries none at all.
    #[test]
    fn the_reference_list_offers_measurements_and_skips_descriptions() {
        let conn = mem();
        let (well, depths, vals) = synth(&conn, 0.0, true);
        as_aux(&conn, &well, "CORE GAMMA", "CORE_GR", &depths, &vals);

        // A text-only item, and a numeric one with too few samples to correlate.
        let text: Vec<db::AuxRow> = depths
            .iter()
            .take(20)
            .map(|&d| db::AuxRow {
                dataset: "CORE".into(),
                depth_top: d,
                depth_base: None,
                item: "LITHOLOGY".into(),
                value_num: None,
                value_text: Some("fine sst".into()),
            })
            .chain(depths.iter().take(3).map(|&d| db::AuxRow {
                dataset: "CORE".into(),
                depth_top: d,
                depth_base: None,
                item: "SPOT_CHECK".into(),
                value_num: Some(1.0),
                value_text: None,
            }))
            .collect();
        db::insert_aux_data(&conn, &well, "CORE", "RAW", None, &text).unwrap();

        let refs = list_core_references(&conn, &well).unwrap();
        let names: Vec<&str> = refs.iter().map(|r| r.item.as_str()).collect();
        assert!(names.contains(&"CPOR"), "the plug porosity column is a reference: {names:?}");
        assert!(names.contains(&"CORE_GR"), "a delivered core gamma is a reference: {names:?}");
        assert!(
            !names.contains(&"LITHOLOGY"),
            "a description cannot anchor a depth shift: {names:?}"
        );
        assert!(
            !names.contains(&"SPOT_CHECK"),
            "three samples is not a correlation: {names:?}"
        );
        let gr = refs.iter().find(|r| r.item == "CORE_GR").unwrap();
        assert_eq!(gr.family, "GR", "the picker can show which pairing is like-for-like");
    }
}
