use duckdb::{params, Appender, Connection};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("duckdb error: {0}")]
    DuckDb(#[from] duckdb::Error),
    #[error("column length mismatch: {0}")]
    LengthMismatch(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type DbResult<T> = Result<T, DbError>;

/// Opens (creating if needed) the embedded DuckDB file and applies the schema.
pub fn init_db(path: &str) -> DbResult<Connection> {
    let conn = Connection::open(path)?;
    create_schema(&conn)?;
    Ok(conn)
}

/// Opens the database, self-healing from a corrupted write-ahead log that fails to
/// replay. This happens when the process is killed uncleanly mid-write — in practice
/// `tauri dev` restarting the backend on every source-file change is the single most
/// common trigger, so this must not require a human to intervene each time. On a WAL
/// replay failure the WAL is moved aside as a timestamped `.corrupt-backup-<ts>` file
/// (never deleted — this discards only the writes made since the last checkpoint, so
/// the backup is kept for manual recovery) and the open is retried once against the
/// checkpointed database. Any other kind of failure is returned as-is.
pub fn init_db_resilient(path: &str) -> DbResult<Connection> {
    match init_db(path) {
        Ok(conn) => Ok(conn),
        Err(e) => {
            let msg = e.to_string();
            if !msg.contains("WAL") {
                return Err(e);
            }
            eprintln!("warning: {path} has a corrupted WAL that failed to replay ({msg}); recovering from the last checkpoint");
            let wal_path = format!("{path}.wal");
            if std::path::Path::new(&wal_path).exists() {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let backup = format!("{wal_path}.corrupt-backup-{ts}");
                std::fs::rename(&wal_path, &backup)?;
                eprintln!("warning: moved corrupted WAL to {backup}");
            }
            init_db(path)
        }
    }
}

pub(crate) fn create_schema(conn: &Connection) -> DbResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS wells (
            well_id     UUID PRIMARY KEY,
            well_name   VARCHAR NOT NULL,
            field_name  VARCHAR,
            td          FLOAT,
            kb          FLOAT
        );

        CREATE TABLE IF NOT EXISTS standard_curves (
            well_id     UUID NOT NULL,
            depth       FLOAT NOT NULL,
            gr          FLOAT,
            res_deep    FLOAT,
            nphi        FLOAT,
            rhob        FLOAT,
            dt          FLOAT,
            sp          FLOAT,
            PRIMARY KEY (well_id, depth)
        );
        ALTER TABLE standard_curves ADD COLUMN IF NOT EXISTS dt FLOAT;
        ALTER TABLE standard_curves ADD COLUMN IF NOT EXISTS sp FLOAT;

        CREATE TABLE IF NOT EXISTS high_res_curves (
            well_id     UUID NOT NULL,
            depth       FLOAT NOT NULL,
            micro_res   FLOAT,
            image_pad   FLOAT,
            PRIMARY KEY (well_id, depth)
        );

        CREATE TABLE IF NOT EXISTS lqr_parameters (
            well_id             UUID NOT NULL,
            depth               FLOAT NOT NULL,
            clay_volume         FLOAT,
            capillary_pressure  FLOAT,
            microporosity       FLOAT,
            PRIMARY KEY (well_id, depth)
        );

        CREATE TABLE IF NOT EXISTS array_logs (
            well_id             UUID NOT NULL,
            depth               FLOAT NOT NULL,
            nmr_t2_distribution FLOAT[],
            PRIMARY KEY (well_id, depth)
        );

        -- Long/tall store for module + equation outputs: one row per (well, depth, curve),
        -- so adding a new curve never requires a schema migration.
        --
        -- NO primary key ON PURPOSE (perf). The natural key is (well_id, depth, curve_name),
        -- but a 3-column PRIMARY KEY forces DuckDB to maintain an ART uniqueness index on
        -- every inserted row — measured ~3.7× slower inserts (311k vs 1.16M rows/s), which
        -- dominated field-scale runs (2000 wells). Uniqueness is instead guaranteed by the
        -- WRITE DISCIPLINE: `write_computed_curves_batch` always DELETEs a well's rows for the
        -- curve names it is about to write before appending fresh ones, and the point-update
        -- path (`update_computed_sample`) UPDATEs in place — no code path ever inserts a
        -- duplicate. Existing databases are rebuilt PK-less by `migrate_drop_computed_curves_pk`.
        CREATE TABLE IF NOT EXISTS computed_curves (
            well_id     UUID NOT NULL,
            depth       FLOAT NOT NULL,
            curve_name  VARCHAR NOT NULL,
            value       FLOAT
        );

        -- User-authored petrophysical equations (Rhai scripts), analogous to Geolog's
        -- loglan module registry / IP's formula library.
        CREATE TABLE IF NOT EXISTS equations (
            equation_id     UUID PRIMARY KEY,
            name            VARCHAR NOT NULL UNIQUE,
            description     VARCHAR,
            script          VARCHAR NOT NULL,
            input_curves    VARCHAR NOT NULL, -- comma-separated curve mnemonics
            output_curve    VARCHAR NOT NULL,
            output_units    VARCHAR,
            updated_at      TIMESTAMP NOT NULL DEFAULT now()
        );
        -- 'rhai' (per-sample scripts, legacy) or 'python' (vectorized numpy, default for new).
        ALTER TABLE equations ADD COLUMN IF NOT EXISTS language VARCHAR DEFAULT 'rhai';

        -- Formation tops / interval markers, analogous to Geolog's TOPS_GEO.TOPS interval log.
        CREATE TABLE IF NOT EXISTS tops (
            well_id     UUID NOT NULL,
            top_name    VARCHAR NOT NULL,
            depth       FLOAT NOT NULL,
            color       VARCHAR,
            PRIMARY KEY (well_id, top_name)
        );

        -- Depth intervals per well, analogous to Geolog's zoned interval sets. Modules
        -- resolve their interval parameters per zone at run time.
        CREATE TABLE IF NOT EXISTS zones (
            well_id      UUID NOT NULL,
            zone_name    VARCHAR NOT NULL,
            top_depth    FLOAT NOT NULL,
            bottom_depth FLOAT NOT NULL,
            PRIMARY KEY (well_id, zone_name)
        );

        -- Per-zone interval parameter values (Geolog: interval logs like GR_MA, GR_SH,
        -- RW, M, N). zone_name '*' holds whole-well defaults.
        CREATE TABLE IF NOT EXISTS zone_params (
            well_id      UUID NOT NULL,
            zone_name    VARCHAR NOT NULL,
            param_name   VARCHAR NOT NULL,
            value_num    FLOAT,
            value_text   VARCHAR,
            PRIMARY KEY (well_id, zone_name, param_name)
        );

        -- Core plug measurements (routine core analysis), sparse/irregular depths that do
        -- NOT align with the standard_curves depth grid — kept in its own table rather
        -- than computed_curves so overlay panels can fetch it at its own resolution.
        CREATE TABLE IF NOT EXISTS core_data (
            well_id     UUID NOT NULL,
            depth       FLOAT NOT NULL,
            cpor        FLOAT, -- core porosity, v/v
            cperm       FLOAT, -- core permeability, mD
            cgd         FLOAT, -- core grain density, g/cc
            csw         FLOAT, -- core water saturation, v/v
            PRIMARY KEY (well_id, depth)
        );

        -- Special core analysis: capillary-pressure measurements. Several Pc/Sw points
        -- per plug, so no primary key — re-import replaces per well (like core_data).
        CREATE TABLE IF NOT EXISTS scal_pc (
            well_id     UUID NOT NULL,
            sample_no   INTEGER,      -- plug/sample number as delivered
            depth       FLOAT,        -- plug depth, m (optional)
            perm        FLOAT,        -- plug permeability, mD
            poro        FLOAT,        -- plug porosity, v/v
            pc          FLOAT NOT NULL, -- capillary pressure, psi (lab system)
            sw          FLOAT NOT NULL  -- water saturation, v/v
        );

        -- Named user documents (saved layouts, plot property sets, ...), stored as JSON.
        CREATE TABLE IF NOT EXISTS documents (
            doc_id       UUID NOT NULL,
            doc_type     VARCHAR NOT NULL,
            name         VARCHAR NOT NULL,
            json         VARCHAR NOT NULL,
            updated_at   TIMESTAMP NOT NULL DEFAULT now(),
            PRIMARY KEY (doc_type, name)
        );

        -- Phase 6: generic curve store. Unlike `standard_curves` (fixed 6 mnemonics),
        -- this holds ANY curve at ANY name, in one of several named sets (RAW = as
        -- imported, EDIT = user-edited, FINAL = QC'd for delivery). `curve_meta` is the
        -- catalog row (one per curve per set); `curve_samples` is the long/tall value
        -- store, mirroring the `computed_curves` pattern so new curves never need a
        -- schema migration.
        CREATE TABLE IF NOT EXISTS curve_meta (
            curve_id    UUID PRIMARY KEY,
            well_id     UUID NOT NULL,
            set_name    VARCHAR NOT NULL DEFAULT 'RAW',
            mnemonic    VARCHAR NOT NULL,
            unit        VARCHAR,
            family      VARCHAR,          -- e.g. GR, RES, NPHI, RHOB, DT, SP, PEF, CALI
            source      VARCHAR,          -- e.g. 'LAS import', 'DLIS import', 'computed'
            run_no      INTEGER,
            UNIQUE (well_id, set_name, mnemonic, run_no)
        );

        CREATE TABLE IF NOT EXISTS curve_samples (
            curve_id    UUID NOT NULL,
            depth       FLOAT NOT NULL,
            value       FLOAT,
            PRIMARY KEY (curve_id, depth)
        );

        -- Deviation survey + computed TVD/TVDSS (minimum curvature), one row per
        -- station. `well_path` is empty for vertical wells (MD == TVD assumed).
        CREATE TABLE IF NOT EXISTS well_path (
            well_id     UUID NOT NULL,
            md          FLOAT NOT NULL,
            inc         FLOAT NOT NULL,   -- inclination, degrees
            azi         FLOAT NOT NULL,   -- azimuth, degrees
            tvd         FLOAT,            -- computed, minimum curvature
            tvdss       FLOAT,            -- tvd - kb (or well.kb if datum omitted)
            PRIMARY KEY (well_id, md)
        );

        -- Well groups: user-defined named sets of wells so a large field (2000+ wells) can
        -- be viewed and processed in manageable subsets. `active` flags the single group
        -- currently filtering the whole workspace (0 or 1 active, enforced in code — an
        -- active group hides non-members from the Wells pane and scopes batch runs to its
        -- members). `rule_json` is reserved for future rule-based membership; membership
        -- today is the explicit `well_group_members` list.
        CREATE TABLE IF NOT EXISTS well_groups (
            group_id    UUID PRIMARY KEY,
            name        VARCHAR NOT NULL,
            active      INTEGER NOT NULL DEFAULT 0,
            rule_json   VARCHAR,
            created_at  TIMESTAMP NOT NULL DEFAULT now()
        );
        CREATE TABLE IF NOT EXISTS well_group_members (
            group_id    UUID NOT NULL,
            well_id     UUID NOT NULL,
            PRIMARY KEY (group_id, well_id)
        );
        "#,
    )?;
    Ok(())
}

/// Migrates once, on open: copies every `standard_curves` column into the generic
/// `curve_meta`/`curve_samples` store as set 'RAW', so Phase 6 code (units, TVD-aware
/// resampling, curve catalog) has real data without disturbing anything that still reads
/// `standard_curves` directly. Idempotent — checks `curve_meta` for any row with
/// source = 'standard_curves migration' before doing any work, so it runs at most once
/// per well per column even if called on every launch.
pub fn migrate_standard_curves_to_generic_store(conn: &Connection) -> DbResult<()> {
    const COLUMNS: &[(&str, &str, &str)] = &[
        // (db column, mnemonic, family)
        ("gr", "GR", "GR"),
        ("res_deep", "RES_DEEP", "RES"),
        ("nphi", "NPHI", "NPHI"),
        ("rhob", "RHOB", "RHOB"),
        ("dt", "DT", "DT"),
        ("sp", "SP", "SP"),
    ];
    const UNITS: &[(&str, &str)] =
        &[("GR", "gAPI"), ("RES", "ohm.m"), ("NPHI", "v/v"), ("RHOB", "g/cc"), ("DT", "us/ft"), ("SP", "mV")];

    let mut stmt = conn.prepare("SELECT well_id FROM wells")?;
    let well_ids: Vec<String> = stmt.query_map([], |r| r.get::<_, String>(0))?.filter_map(|r| r.ok()).collect();
    drop(stmt);

    for well_id in well_ids {
        for (col, mnemonic, family) in COLUMNS {
            let already: i64 = conn.query_row(
                "SELECT COUNT(*) FROM curve_meta WHERE well_id = ?1 AND mnemonic = ?2 AND source = 'standard_curves migration'",
                params![well_id, mnemonic],
                |r| r.get(0),
            )?;
            if already > 0 {
                continue;
            }

            let has_data: i64 = conn.query_row(
                &format!(
                    "SELECT COUNT(*) FROM standard_curves WHERE well_id = ?1 AND {col} IS NOT NULL AND NOT isnan({col})"
                ),
                params![well_id],
                |r| r.get(0),
            )?;
            if has_data == 0 {
                continue;
            }

            let curve_id = Uuid::new_v4().to_string();
            let unit = UNITS.iter().find(|(f, _)| f == family).map(|(_, u)| *u);
            conn.execute(
                "INSERT INTO curve_meta (curve_id, well_id, set_name, mnemonic, unit, family, source, run_no)
                 VALUES (?1, ?2, 'RAW', ?3, ?4, ?5, 'standard_curves migration', NULL)",
                params![curve_id, well_id, mnemonic, unit, family],
            )?;
            conn.execute(
                &format!(
                    "INSERT INTO curve_samples (curve_id, depth, value)
                     SELECT ?1, depth, {col} FROM standard_curves WHERE well_id = ?2 AND {col} IS NOT NULL"
                ),
                params![curve_id, well_id],
            )?;
        }
    }
    Ok(())
}

/// One-time migration that drops the legacy 3-column PRIMARY KEY from `computed_curves`.
///
/// Older databases created the table with `PRIMARY KEY (well_id, depth, curve_name)`, whose
/// ART uniqueness index made every inserted row ~3.7× more expensive — the dominant cost of
/// field-scale (2000-well) runs. DuckDB can't drop an unnamed PK constraint in place, so the
/// table is rebuilt without it. Idempotent: `duckdb_constraints()` is consulted first, so on
/// databases already PK-less (including every freshly created one) this is a no-op. Uniqueness
/// is preserved by the write discipline documented on the table (see `create_schema`).
pub fn migrate_drop_computed_curves_pk(conn: &Connection) -> DbResult<()> {
    let has_pk: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duckdb_constraints()
         WHERE table_name = 'computed_curves' AND constraint_type = 'PRIMARY KEY'",
        [],
        |r| r.get(0),
    )?;
    if has_pk == 0 {
        return Ok(());
    }
    // Rebuild PK-less, preserving every row, atomically.
    conn.execute_batch(
        "BEGIN;
         CREATE TABLE computed_curves_new (
             well_id     UUID NOT NULL,
             depth       FLOAT NOT NULL,
             curve_name  VARCHAR NOT NULL,
             value       FLOAT
         );
         INSERT INTO computed_curves_new SELECT well_id, depth, curve_name, value FROM computed_curves;
         DROP TABLE computed_curves;
         ALTER TABLE computed_curves_new RENAME TO computed_curves;
         COMMIT;",
    )?;
    Ok(())
}

/// A single standard LAS curve row, used for deserializing incoming parsed data
/// (LAS 2.0 / Geolog CSV) before batch insertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardCurveRow {
    pub depth: f32,
    pub gr: f32,
    pub res_deep: f32,
    pub nphi: f32,
    pub rhob: f32,
}

/// Bulk-inserts standard curve columns for a single well using DuckDB's Appender,
/// which streams rows without per-row transaction overhead.
pub fn insert_standard_curves(
    conn: &Connection,
    well_id: Uuid,
    depths: Vec<f32>,
    gr: Vec<f32>,
    res_deep: Vec<f32>,
    nphi: Vec<f32>,
    rhob: Vec<f32>,
    dt: Vec<f32>,
    sp: Vec<f32>,
) -> DbResult<()> {
    let n = depths.len();
    if gr.len() != n || res_deep.len() != n || nphi.len() != n || rhob.len() != n || dt.len() != n || sp.len() != n {
        return Err(DbError::LengthMismatch(format!(
            "expected all columns to have length {n}"
        )));
    }

    let well_id_str = well_id.to_string();
    let mut appender: Appender = conn.appender("standard_curves")?;
    for i in 0..n {
        appender.append_row(params![
            well_id_str,
            depths[i],
            gr[i],
            res_deep[i],
            nphi[i],
            rhob[i],
            dt[i],
            sp[i],
        ])?;
    }
    appender.flush()?;
    Ok(())
}

/// Bulk-inserts core plug data for one well, replacing any prior rows for that well
/// (re-import overwrites rather than duplicating).
pub fn insert_core_data(
    conn: &Connection,
    well_id: &str,
    depths: &[f32],
    cpor: &[f32],
    cperm: &[f32],
    cgd: &[f32],
    csw: &[f32],
) -> DbResult<()> {
    let n = depths.len();
    if cpor.len() != n || cperm.len() != n || cgd.len() != n || csw.len() != n {
        return Err(DbError::LengthMismatch(format!("expected all core columns to have length {n}")));
    }
    conn.execute("DELETE FROM core_data WHERE well_id = ?1", params![well_id])?;
    let mut appender: Appender = conn.appender("core_data")?;
    for i in 0..n {
        appender.append_row(params![well_id, depths[i], cpor[i], cperm[i], cgd[i], csw[i]])?;
    }
    appender.flush()?;
    Ok(())
}

/// One capillary-pressure row as imported/fetched (see `scal_pc` table).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScalPcRow {
    pub sample_no: Option<i32>,
    pub depth: Option<f32>,
    pub perm: f32,
    pub poro: f32,
    pub pc: f32,
    pub sw: f32,
}

/// Bulk-inserts SCAL capillary-pressure rows for one well, replacing any prior rows
/// (re-import overwrites, like `insert_core_data`).
pub fn insert_scal_pc(conn: &Connection, well_id: &str, rows: &[ScalPcRow]) -> DbResult<()> {
    conn.execute("DELETE FROM scal_pc WHERE well_id = ?1", params![well_id])?;
    let mut appender: Appender = conn.appender("scal_pc")?;
    for r in rows {
        appender.append_row(params![well_id, r.sample_no, r.depth, r.perm, r.poro, r.pc, r.sw])?;
    }
    appender.flush()?;
    Ok(())
}

pub fn get_scal_pc(conn: &Connection, well_id: &str) -> DbResult<Vec<ScalPcRow>> {
    let mut stmt = conn.prepare(
        "SELECT sample_no, depth, perm, poro, pc, sw FROM scal_pc
         WHERE well_id = ?1 ORDER BY sample_no NULLS FIRST, pc",
    )?;
    let rows = stmt.query_map(params![well_id], |row| {
        Ok(ScalPcRow {
            sample_no: row.get(0)?,
            depth: row.get(1)?,
            perm: row.get::<_, Option<f32>>(2)?.unwrap_or(f32::NAN),
            poro: row.get::<_, Option<f32>>(3)?.unwrap_or(f32::NAN),
            pc: row.get(4)?,
            sw: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn insert_well(
    conn: &Connection,
    well_id: Uuid,
    well_name: &str,
    field_name: Option<&str>,
    td: Option<f32>,
    kb: Option<f32>,
) -> DbResult<()> {
    conn.execute(
        "INSERT INTO wells (well_id, well_name, field_name, td, kb) VALUES (?, ?, ?, ?, ?)",
        params![well_id.to_string(), well_name, field_name, td, kb],
    )?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WellSummary {
    pub well_id: String,
    pub well_name: String,
    pub field_name: Option<String>,
}

/// Lists every well for the object tree, along with which curve tables actually hold data
/// for it (so the tree can show real children instead of a fixed guess).
pub fn list_wells(conn: &Connection) -> DbResult<Vec<WellSummary>> {
    let mut stmt = conn.prepare("SELECT well_id, well_name, field_name FROM wells ORDER BY well_name")?;
    let rows = stmt.query_map([], |row| {
        Ok(WellSummary {
            well_id: row.get(0)?,
            well_name: row.get(1)?,
            field_name: row.get(2)?,
        })
    })?;
    let mut wells = Vec::new();
    for r in rows {
        wells.push(r?);
    }
    Ok(wells)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TopEntry {
    pub top_name: String,
    pub depth: f32,
    pub color: Option<String>,
}

/// Lists the formation tops for one well, ordered by depth (Geolog's TOPS_GEO.TOPS
/// equivalent — the Tops panel's data source).
pub fn list_tops(conn: &Connection, well_id: &str) -> DbResult<Vec<TopEntry>> {
    let mut stmt = conn.prepare("SELECT top_name, depth, color FROM tops WHERE well_id = ?1 ORDER BY depth")?;
    let rows = stmt.query_map(params![well_id], |row| {
        Ok(TopEntry { top_name: row.get(0)?, depth: row.get(1)?, color: row.get(2)? })
    })?;
    let mut tops = Vec::new();
    for r in rows {
        tops.push(r?);
    }
    Ok(tops)
}

/// Upserts a formation top by (well_id, top_name).
pub fn upsert_top(conn: &Connection, well_id: &str, top_name: &str, depth: f32, color: Option<&str>) -> DbResult<()> {
    conn.execute(
        "INSERT INTO tops (well_id, top_name, depth, color) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (well_id, top_name) DO UPDATE SET depth = excluded.depth, color = excluded.color",
        params![well_id, top_name, depth, color],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct ZoneEntry {
    pub zone_name: String,
    pub top_depth: f32,
    pub bottom_depth: f32,
}

pub fn list_zones(conn: &Connection, well_id: &str) -> DbResult<Vec<ZoneEntry>> {
    let mut stmt =
        conn.prepare("SELECT zone_name, top_depth, bottom_depth FROM zones WHERE well_id = ?1 ORDER BY top_depth")?;
    let rows = stmt.query_map(params![well_id], |row| {
        Ok(ZoneEntry { zone_name: row.get(0)?, top_depth: row.get(1)?, bottom_depth: row.get(2)? })
    })?;
    let mut zones = Vec::new();
    for r in rows {
        zones.push(r?);
    }
    Ok(zones)
}

pub fn upsert_zone(conn: &Connection, well_id: &str, zone_name: &str, top_depth: f32, bottom_depth: f32) -> DbResult<()> {
    conn.execute(
        "INSERT INTO zones (well_id, zone_name, top_depth, bottom_depth) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (well_id, zone_name) DO UPDATE SET top_depth = excluded.top_depth, bottom_depth = excluded.bottom_depth",
        params![well_id, zone_name, top_depth, bottom_depth],
    )?;
    Ok(())
}

pub fn delete_zone(conn: &Connection, well_id: &str, zone_name: &str) -> DbResult<()> {
    conn.execute("DELETE FROM zones WHERE well_id = ?1 AND zone_name = ?2", params![well_id, zone_name])?;
    conn.execute("DELETE FROM zone_params WHERE well_id = ?1 AND zone_name = ?2", params![well_id, zone_name])?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct DocumentEntry {
    pub name: String,
    pub json: String,
}

/// Saves (or replaces, by (doc_type, name)) one named JSON document.
pub fn save_document(conn: &Connection, doc_type: &str, name: &str, json: &str) -> DbResult<()> {
    conn.execute(
        "INSERT INTO documents (doc_id, doc_type, name, json) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (doc_type, name) DO UPDATE SET json = excluded.json, updated_at = now()",
        params![Uuid::new_v4().to_string(), doc_type, name, json],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct WellGroupEntry {
    pub group_id: String,
    pub name: String,
    pub active: bool,
    pub member_count: i64,
    /// The member well_ids (explicit membership), so the frontend can filter locally.
    pub well_ids: Vec<String>,
}

/// Lists every well group with its member count and member ids. Groups are few (dozens
/// at most) even when wells number in the thousands, so filling members per group is fine.
pub fn list_well_groups(conn: &Connection) -> DbResult<Vec<WellGroupEntry>> {
    let mut stmt = conn.prepare(
        "SELECT g.group_id, g.name, g.active,
                (SELECT COUNT(*) FROM well_group_members m WHERE m.group_id = g.group_id)
         FROM well_groups g ORDER BY g.name",
    )?;
    let rows = stmt.query_map([], |row| {
        let active: i64 = row.get(2)?;
        Ok(WellGroupEntry {
            group_id: row.get(0)?,
            name: row.get(1)?,
            active: active != 0,
            member_count: row.get(3)?,
            well_ids: Vec::new(),
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    for g in &mut out {
        let mut ms = conn.prepare("SELECT well_id FROM well_group_members WHERE group_id = ?1")?;
        let wr = ms.query_map(params![g.group_id], |row| row.get::<_, String>(0))?;
        for w in wr {
            g.well_ids.push(w?);
        }
    }
    Ok(out)
}

/// Creates a group with an initial (possibly empty) explicit member list; returns its id.
pub fn create_well_group(conn: &Connection, name: &str, well_ids: &[String]) -> DbResult<String> {
    let group_id = Uuid::new_v4().to_string();
    conn.execute("INSERT INTO well_groups (group_id, name, active) VALUES (?1, ?2, 0)", params![group_id, name])?;
    set_well_group_members(conn, &group_id, well_ids)?;
    Ok(group_id)
}

pub fn rename_well_group(conn: &Connection, group_id: &str, name: &str) -> DbResult<()> {
    conn.execute("UPDATE well_groups SET name = ?2 WHERE group_id = ?1", params![group_id, name])?;
    Ok(())
}

pub fn delete_well_group(conn: &Connection, group_id: &str) -> DbResult<()> {
    conn.execute("DELETE FROM well_group_members WHERE group_id = ?1", params![group_id])?;
    conn.execute("DELETE FROM well_groups WHERE group_id = ?1", params![group_id])?;
    Ok(())
}

/// Replaces a group's membership with exactly `well_ids`.
pub fn set_well_group_members(conn: &Connection, group_id: &str, well_ids: &[String]) -> DbResult<()> {
    conn.execute("DELETE FROM well_group_members WHERE group_id = ?1", params![group_id])?;
    for w in well_ids {
        conn.execute(
            "INSERT INTO well_group_members (group_id, well_id) VALUES (?1, ?2)
             ON CONFLICT (group_id, well_id) DO NOTHING",
            params![group_id, w],
        )?;
    }
    Ok(())
}

/// Sets the single active group, or clears it when `group_id` is None. At most one group
/// is ever active — activating one deactivates the rest.
pub fn set_active_well_group(conn: &Connection, group_id: Option<&str>) -> DbResult<()> {
    conn.execute("UPDATE well_groups SET active = 0", [])?;
    if let Some(id) = group_id {
        conn.execute("UPDATE well_groups SET active = 1 WHERE group_id = ?1", params![id])?;
    }
    Ok(())
}

pub fn list_documents(conn: &Connection, doc_type: &str) -> DbResult<Vec<DocumentEntry>> {
    let mut stmt = conn.prepare("SELECT name, json FROM documents WHERE doc_type = ?1 ORDER BY name")?;
    let rows = stmt.query_map(params![doc_type], |row| Ok(DocumentEntry { name: row.get(0)?, json: row.get(1)? }))?;
    let mut docs = Vec::new();
    for r in rows {
        docs.push(r?);
    }
    Ok(docs)
}

pub fn delete_document(conn: &Connection, doc_type: &str, name: &str) -> DbResult<()> {
    conn.execute("DELETE FROM documents WHERE doc_type = ?1 AND name = ?2", params![doc_type, name])?;
    Ok(())
}

/// Builds zones from the well's tops: each top starts a zone named after it, ending at the
/// next top (or the deepest curve sample for the last one). Existing zones are replaced.
pub fn zones_from_tops(conn: &Connection, well_id: &str) -> DbResult<Vec<ZoneEntry>> {
    let tops = list_tops(conn, well_id)?;
    if tops.is_empty() {
        return Ok(Vec::new());
    }
    let max_depth: f32 = conn
        .query_row(
            "SELECT COALESCE(MAX(depth), 0.0) FROM standard_curves WHERE well_id = ?1",
            params![well_id],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    conn.execute("DELETE FROM zones WHERE well_id = ?1", params![well_id])?;
    let mut zones = Vec::new();
    for (i, top) in tops.iter().enumerate() {
        let bottom = tops.get(i + 1).map(|t| t.depth).unwrap_or_else(|| max_depth.max(top.depth));
        upsert_zone(conn, well_id, &top.top_name, top.depth, bottom)?;
        zones.push(ZoneEntry { zone_name: top.top_name.clone(), top_depth: top.depth, bottom_depth: bottom });
    }
    Ok(zones)
}

// ---------------------------------------------------------------------------
// Database inspector (Geolog "Text" equivalent): paged reads over a whitelist
// of tables + explicit single-cell update commands. The frontend never sends
// SQL — table and column names are validated against these specs.
// ---------------------------------------------------------------------------

/// (table, columns, well_scoped, ORDER BY clause)
const TABLE_SPECS: &[(&str, &[&str], bool, &str)] = &[
    ("wells", &["well_id", "well_name", "field_name", "td", "kb"], false, "well_name"),
    ("standard_curves", &["depth", "gr", "res_deep", "nphi", "rhob", "dt", "sp"], true, "depth"),
    ("computed_curves", &["depth", "curve_name", "value"], true, "curve_name, depth"),
    ("tops", &["top_name", "depth", "color"], true, "depth"),
    ("zones", &["zone_name", "top_depth", "bottom_depth"], true, "top_depth"),
    ("zone_params", &["zone_name", "param_name", "value_num", "value_text"], true, "zone_name, param_name"),
    ("core_data", &["depth", "cpor", "cperm", "cgd", "csw"], true, "depth"),
];

#[derive(Debug, Serialize)]
pub struct TablePage {
    pub columns: Vec<String>,
    /// Cells stringified by DuckDB's VARCHAR cast; None = SQL NULL.
    pub rows: Vec<Vec<Option<String>>>,
    pub total_rows: usize,
}

/// One page of a whitelisted table, every cell cast to VARCHAR (uniform frontend
/// handling). Well-scoped tables require `well_id`.
pub fn get_table_page(
    conn: &Connection,
    table: &str,
    well_id: Option<&str>,
    offset: usize,
    limit: usize,
) -> Result<TablePage, String> {
    let spec = TABLE_SPECS
        .iter()
        .find(|(t, ..)| *t == table)
        .ok_or_else(|| format!("unknown table '{table}'"))?;
    let (_, columns, well_scoped, order) = *spec;
    if well_scoped && well_id.is_none() {
        return Err(format!("table '{table}' requires a well"));
    }

    let select_list = columns
        .iter()
        .map(|c| format!("CAST({c} AS VARCHAR)"))
        .collect::<Vec<_>>()
        .join(", ");
    let where_clause = if well_scoped { "WHERE well_id = ?1" } else { "" };
    let limit = limit.clamp(1, 2000);

    let count_sql = format!("SELECT COUNT(*) FROM {table} {where_clause}");
    let page_sql = format!("SELECT {select_list} FROM {table} {where_clause} ORDER BY {order} LIMIT {limit} OFFSET {offset}");

    let run = || -> DbResult<TablePage> {
        let total_rows: usize = if well_scoped {
            conn.query_row(&count_sql, params![well_id.unwrap()], |r| r.get::<_, i64>(0))? as usize
        } else {
            conn.query_row(&count_sql, [], |r| r.get::<_, i64>(0))? as usize
        };

        let mut stmt = conn.prepare(&page_sql)?;
        let map_row = |row: &duckdb::Row| -> duckdb::Result<Vec<Option<String>>> {
            (0..columns.len()).map(|i| row.get::<_, Option<String>>(i)).collect()
        };
        let mut rows = Vec::new();
        if well_scoped {
            let mapped = stmt.query_map(params![well_id.unwrap()], map_row)?;
            for r in mapped {
                rows.push(r?);
            }
        } else {
            let mapped = stmt.query_map([], map_row)?;
            for r in mapped {
                rows.push(r?);
            }
        }
        Ok(TablePage { columns: columns.iter().map(|c| c.to_string()).collect(), rows, total_rows })
    };
    run().map_err(|e| e.to_string())
}

/// Runs one read-only SELECT (Geolog SQL equivalent, but full DuckDB SQL: joins,
/// window functions, aggregates). Anything that isn't a single SELECT/WITH statement
/// is rejected before execution.
pub fn run_readonly_query(conn: &Connection, sql: &str, limit: usize) -> Result<TablePage, String> {
    let trimmed = sql.trim().trim_end_matches(';').trim();
    let lowered = trimmed.to_lowercase();
    if !(lowered.starts_with("select") || lowered.starts_with("with")) {
        return Err("only SELECT queries are allowed here".into());
    }
    if trimmed.contains(';') {
        return Err("one statement at a time".into());
    }

    let limit = limit.clamp(1, 5000);
    let wrapped = format!("SELECT * FROM ({trimmed}) __sandibumi_q LIMIT {limit}");
    let mut stmt = conn.prepare(&wrapped).map_err(|e| e.to_string())?;
    let mut rows_out: Vec<Vec<Option<String>>> = Vec::new();
    {
        let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let mut out_row = Vec::new();
            let mut i = 0;
            loop {
                match row.get_ref(i) {
                    Ok(value) => out_row.push(value_ref_to_string(value)),
                    Err(_) => break,
                }
                i += 1;
            }
            rows_out.push(out_row);
        }
    }
    let columns = stmt.column_names().iter().map(|c| c.to_string()).collect();
    let total = rows_out.len();
    Ok(TablePage { columns, rows: rows_out, total_rows: total })
}

fn value_ref_to_string(value: duckdb::types::ValueRef) -> Option<String> {
    use duckdb::types::ValueRef as V;
    match value {
        V::Null => None,
        V::Boolean(b) => Some(b.to_string()),
        V::TinyInt(v) => Some(v.to_string()),
        V::SmallInt(v) => Some(v.to_string()),
        V::Int(v) => Some(v.to_string()),
        V::BigInt(v) => Some(v.to_string()),
        V::HugeInt(v) => Some(v.to_string()),
        V::UTinyInt(v) => Some(v.to_string()),
        V::USmallInt(v) => Some(v.to_string()),
        V::UInt(v) => Some(v.to_string()),
        V::UBigInt(v) => Some(v.to_string()),
        V::Float(v) => Some(v.to_string()),
        V::Double(v) => Some(v.to_string()),
        V::Text(bytes) => Some(String::from_utf8_lossy(bytes).to_string()),
        other => Some(format!("{other:?}")),
    }
}

#[cfg(test)]
mod inspector_tests {
    use super::*;

    fn mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        conn
    }

    /// Reproduces the exact failure that motivated `init_db_resilient`, using a real
    /// database + WAL pair captured from an actual crash (`tauri dev` restarting the
    /// backend mid-write) — a synthetic garbage WAL doesn't trigger the same DuckDB
    /// internal-error path, so a faithful fixture is the only reliable reproducer.
    /// Recovery must move the WAL aside and open cleanly on the checkpointed data.
    #[test]
    fn resilient_open_recovers_from_corrupt_wal() {
        let dir = std::env::temp_dir().join(format!("arshilla_wal_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("project.duckdb");
        let db_path_str = db_path.to_str().unwrap();
        let wal_path = format!("{db_path_str}.wal");

        std::fs::write(&db_path, include_bytes!("../tests/fixtures/corrupt_torn.duckdb")).unwrap();
        std::fs::write(&wal_path, include_bytes!("../tests/fixtures/corrupt_torn.wal")).unwrap();

        // A plain open must fail (proves the test actually exercises the bug)...
        assert!(init_db(db_path_str).is_err(), "corrupt WAL must fail a plain open");
        // ...but the resilient path recovers by falling back to the checkpointed data.
        let conn = init_db_resilient(db_path_str).expect("resilient open must recover");
        list_wells(&conn).expect("recovered database must be queryable");

        // The corrupt WAL must be preserved as a backup, not deleted — a fresh WAL
        // legitimately reappears afterward (create_schema's ALTER TABLE calls write
        // to it), so presence of *a* WAL isn't the signal; the backup copy is.
        let backups: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains("corrupt-backup"))
            .collect();
        assert_eq!(backups.len(), 1, "the corrupted WAL must be kept as a backup, not deleted");

        drop(conn);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Phase 6 foundation: standard_curves rows must land in the generic curve store as
    /// set RAW with the right family/unit, and the migration must be a no-op the second
    /// time (idempotent — it's called on every launch via lib.rs::run()).
    #[test]
    fn generic_store_migration_and_manual_curve() {
        let conn = mem_db();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "W1", None, None, None).unwrap();
        insert_standard_curves(
            &conn,
            id,
            vec![1000.0, 1000.5, 1001.0],
            vec![55.0, 60.0, f32::NAN],
            vec![10.0, 12.0, 11.0],
            vec![0.3, 0.31, 0.29],
            vec![2.4, 2.41, 2.39],
            vec![f32::NAN, f32::NAN, f32::NAN],
            vec![f32::NAN, f32::NAN, f32::NAN],
        )
        .unwrap();

        let ids = id.to_string();
        migrate_standard_curves_to_generic_store(&conn).unwrap();
        let catalog = list_generic_curve_catalog(&conn, &ids).unwrap();
        let gr = catalog.iter().find(|c| c.mnemonic == "GR").expect("GR migrated");
        assert_eq!(gr.set_name, "RAW");
        assert_eq!(gr.family.as_deref(), Some("GR"));
        assert_eq!(gr.unit.as_deref(), Some("gAPI"));
        assert_eq!(gr.n_samples, 3);
        let samples = get_curve_samples(&conn, &gr.curve_id).unwrap();
        assert_eq!(samples.len(), 3);
        assert!(samples[2].value.is_nan());

        // DT/SP have no real data (all NaN) in this fixture, so they must NOT appear.
        assert!(!catalog.iter().any(|c| c.mnemonic == "DT"));

        // Idempotent: running again must not duplicate the GR curve.
        migrate_standard_curves_to_generic_store(&conn).unwrap();
        let catalog2 = list_generic_curve_catalog(&conn, &ids).unwrap();
        assert_eq!(catalog2.iter().filter(|c| c.mnemonic == "GR").count(), 1);

        // A manually registered curve (e.g. a future DLIS/LAS import target) round-trips
        // through upsert_curve_meta + insert_curve_samples + get_curve_samples.
        let pef_id = upsert_curve_meta(&conn, &ids, "RAW", "PEF", Some("b/e"), Some("PEF"), Some("LAS import"), None).unwrap();
        insert_curve_samples(&conn, &pef_id, &[1000.0, 1000.5], &[5.1, 5.2]).unwrap();
        let pef_samples = get_curve_samples(&conn, &pef_id).unwrap();
        assert_eq!(pef_samples.len(), 2);
        assert_eq!(pef_samples[0].value, 5.1);

        // Re-upserting the same (well, set, mnemonic) reuses the curve_id, doesn't duplicate it.
        let pef_id2 = upsert_curve_meta(&conn, &ids, "RAW", "PEF", Some("b/e"), Some("PEF"), Some("LAS import"), None).unwrap();
        assert_eq!(pef_id, pef_id2);
    }

    #[test]
    fn readonly_query_selects_and_rejects() {
        let conn = mem_db();
        insert_well(&conn, Uuid::new_v4(), "BALAM-1", Some("Balam"), None, None).unwrap();
        let page = run_readonly_query(&conn, "SELECT well_name, field_name FROM wells", 100).unwrap();
        assert_eq!(page.columns, vec!["well_name", "field_name"]);
        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.rows[0][0].as_deref(), Some("BALAM-1"));

        assert!(run_readonly_query(&conn, "DELETE FROM wells", 100).is_err());
        assert!(run_readonly_query(&conn, "SELECT 1; DROP TABLE wells", 100).is_err());
    }

    #[test]
    fn table_page_reads_and_cell_updates() {
        let conn = mem_db();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "W1", None, None, None).unwrap();
        insert_standard_curves(
            &conn,
            id,
            vec![1000.0, 1000.5],
            vec![55.0, 60.0],
            vec![10.0, 12.0],
            vec![0.3, 0.31],
            vec![2.4, 2.41],
            vec![f32::NAN, f32::NAN],
            vec![f32::NAN, f32::NAN],
        )
        .unwrap();

        let ids = id.to_string();
        let page = get_table_page(&conn, "standard_curves", Some(&ids), 0, 100).unwrap();
        assert_eq!(page.total_rows, 2);
        assert_eq!(page.columns[0], "depth");

        update_standard_sample(&conn, &ids, 1000.0, "gr", 99.5).unwrap();
        let page = get_table_page(&conn, "standard_curves", Some(&ids), 0, 1).unwrap();
        let gr_idx = page.columns.iter().position(|c| c == "gr").unwrap();
        assert_eq!(page.rows[0][gr_idx].as_deref(), Some("99.5"));

        assert!(get_table_page(&conn, "pg_shadow", None, 0, 10).is_err(), "non-whitelisted table must fail");
        assert!(update_standard_sample(&conn, &ids, 1000.0, "well_id", 0.0).is_err(), "key columns must not be editable");
    }

    fn pk_count(conn: &Connection, table: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM duckdb_constraints() WHERE table_name = ?1 AND constraint_type = 'PRIMARY KEY'",
            params![table],
            |r| r.get(0),
        )
        .unwrap()
    }

    /// A legacy database (computed_curves WITH the 3-column PK) must be rebuilt PK-less on
    /// launch, losing no rows, and the migration must be idempotent + a no-op on fresh DBs.
    #[test]
    fn drops_legacy_computed_curves_primary_key() {
        let conn = Connection::open_in_memory().unwrap();
        // Simulate an OLD database: create the table exactly as it used to be.
        conn.execute_batch(
            "CREATE TABLE computed_curves (
                 well_id UUID NOT NULL, depth FLOAT NOT NULL, curve_name VARCHAR NOT NULL, value FLOAT,
                 PRIMARY KEY (well_id, depth, curve_name)
             );",
        )
        .unwrap();
        let w = Uuid::new_v4().to_string();
        conn.execute("INSERT INTO computed_curves VALUES (?1, 1000.0, 'PHIE', 0.2)", params![w]).unwrap();
        conn.execute("INSERT INTO computed_curves VALUES (?1, 1000.5, 'PHIE', 0.21)", params![w]).unwrap();
        assert_eq!(pk_count(&conn, "computed_curves"), 1, "fixture starts with a PK");

        migrate_drop_computed_curves_pk(&conn).unwrap();
        assert_eq!(pk_count(&conn, "computed_curves"), 0, "PK dropped");
        let rows: i64 = conn.query_row("SELECT COUNT(*) FROM computed_curves", [], |r| r.get(0)).unwrap();
        assert_eq!(rows, 2, "no rows lost in the rebuild");

        // Idempotent: a second run does nothing (no PK to drop).
        migrate_drop_computed_curves_pk(&conn).unwrap();
        assert_eq!(pk_count(&conn, "computed_curves"), 0);

        // No-op on a fresh (already PK-less) schema.
        let fresh = mem_db();
        assert_eq!(pk_count(&fresh, "computed_curves"), 0);
        migrate_drop_computed_curves_pk(&fresh).unwrap();
        assert_eq!(pk_count(&fresh, "computed_curves"), 0);
    }

    /// Without the PK, uniqueness rests on the write discipline: `write_computed_curves_batch`
    /// must overwrite (not duplicate) a well's curves on re-run, write several curves at once,
    /// keep other wells untouched, and leave `update_computed_sample` working.
    #[test]
    fn batch_write_overwrites_without_duplicating() {
        use crate::equations::write_computed_curves_batch;
        let conn = mem_db();
        let a = Uuid::new_v4().to_string();
        let b = Uuid::new_v4().to_string();
        let depth = [1000.0f32, 1000.5, 1001.0];

        let count = |w: &str, c: &str| -> i64 {
            conn.query_row(
                "SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1 AND curve_name = ?2",
                params![w, c],
                |r| r.get(0),
            )
            .unwrap()
        };

        // Two curves for well A in one batch.
        write_computed_curves_batch(&conn, &a, &depth, &[("VSH", &[0.1, 0.2, 0.3]), ("PHIE", &[0.25, 0.24, 0.23])])
            .unwrap();
        assert_eq!(count(&a, "VSH"), 3);
        assert_eq!(count(&a, "PHIE"), 3);

        // Re-run the same well+curves → overwrite, NOT append (no duplicate rows).
        write_computed_curves_batch(&conn, &a, &depth, &[("VSH", &[0.9, 0.8, 0.7]), ("PHIE", &[0.1, 0.1, 0.1])])
            .unwrap();
        assert_eq!(count(&a, "VSH"), 3, "re-run must overwrite, not duplicate");
        let v: f32 = conn
            .query_row(
                "SELECT value FROM computed_curves WHERE well_id = ?1 AND curve_name = 'VSH' AND depth = 1000.0",
                params![a],
                |r| r.get(0),
            )
            .unwrap();
        assert!((v - 0.9).abs() < 1e-6, "value updated to the re-run result");

        // A different well is independent.
        write_computed_curves_batch(&conn, &b, &depth, &[("VSH", &[0.5, 0.5, 0.5])]).unwrap();
        assert_eq!(count(&b, "VSH"), 3);
        assert_eq!(count(&a, "VSH"), 3, "writing well B leaves well A's rows intact");

        // Point-update still resolves a unique row by (well, depth, curve).
        update_computed_sample(&conn, &a, 1000.5, "PHIE", 0.42).unwrap();
        let p: f32 = conn
            .query_row(
                "SELECT value FROM computed_curves WHERE well_id = ?1 AND curve_name = 'PHIE' AND depth = 1000.5",
                params![a],
                |r| r.get(0),
            )
            .unwrap();
        assert!((p - 0.42).abs() < 1e-6);
    }
}

/// Edits one wells-table field (name/field as text, td/kb as numbers).
pub fn update_well_field(conn: &Connection, well_id: &str, field: &str, value: Option<&str>) -> Result<(), String> {
    match field {
        "well_name" | "field_name" => {
            conn.execute(&format!("UPDATE wells SET {field} = ?1 WHERE well_id = ?2"), params![value, well_id])
                .map_err(|e| e.to_string())?;
        }
        "td" | "kb" => {
            let num: Option<f32> = match value {
                Some(v) if !v.trim().is_empty() => Some(v.trim().parse::<f32>().map_err(|e| e.to_string())?),
                _ => None,
            };
            conn.execute(&format!("UPDATE wells SET {field} = ?1 WHERE well_id = ?2"), params![num, well_id])
                .map_err(|e| e.to_string())?;
        }
        other => return Err(format!("field '{other}' is not editable")),
    }
    Ok(())
}

/// Edits one standard-curve sample value (NaN = missing).
pub fn update_standard_sample(conn: &Connection, well_id: &str, depth: f32, column: &str, value: f32) -> Result<(), String> {
    const EDITABLE: &[&str] = &["gr", "res_deep", "nphi", "rhob", "dt", "sp"];
    if !EDITABLE.contains(&column) {
        return Err(format!("column '{column}' is not editable"));
    }
    conn.execute(
        &format!("UPDATE standard_curves SET {column} = ?1 WHERE well_id = ?2 AND depth = ?3"),
        params![value, well_id, depth],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Applies a constant depth shift to every core plug of one well (core-to-log
/// alignment). Exactly reversible with -delta, so the frontend makes it undoable.
pub fn shift_core_depths(conn: &Connection, well_id: &str, delta: f32) -> DbResult<usize> {
    let n = conn.execute(
        "UPDATE core_data SET depth = depth + ?1 WHERE well_id = ?2",
        params![delta, well_id],
    )?;
    Ok(n)
}

/// Edits one core-plug sample value (NaN = missing).
pub fn update_core_sample(conn: &Connection, well_id: &str, depth: f32, column: &str, value: f32) -> Result<(), String> {
    const EDITABLE: &[&str] = &["cpor", "cperm", "cgd", "csw"];
    if !EDITABLE.contains(&column) {
        return Err(format!("column '{column}' is not editable"));
    }
    conn.execute(
        &format!("UPDATE core_data SET {column} = ?1 WHERE well_id = ?2 AND depth = ?3"),
        params![value, well_id, depth],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Edits one computed-curve sample value.
pub fn update_computed_sample(conn: &Connection, well_id: &str, depth: f32, curve_name: &str, value: f32) -> Result<(), String> {
    conn.execute(
        "UPDATE computed_curves SET value = ?1 WHERE well_id = ?2 AND depth = ?3 AND curve_name = ?4",
        params![value, well_id, depth, curve_name],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Deletes one formation top.
pub fn delete_top(conn: &Connection, well_id: &str, top_name: &str) -> DbResult<()> {
    conn.execute("DELETE FROM tops WHERE well_id = ?1 AND top_name = ?2", params![well_id, top_name])?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneParamEntry {
    pub zone_name: String,
    pub param_name: String,
    pub value_num: Option<f32>,
    pub value_text: Option<String>,
}

pub fn list_zone_params(conn: &Connection, well_id: &str) -> DbResult<Vec<ZoneParamEntry>> {
    let mut stmt = conn.prepare(
        "SELECT zone_name, param_name, value_num, value_text FROM zone_params WHERE well_id = ?1 ORDER BY zone_name, param_name",
    )?;
    let rows = stmt.query_map(params![well_id], |row| {
        Ok(ZoneParamEntry {
            zone_name: row.get(0)?,
            param_name: row.get(1)?,
            value_num: row.get(2)?,
            value_text: row.get(3)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[derive(Debug, Clone, Serialize)]
pub struct GenericCurveCatalogEntry {
    pub curve_id: String,
    pub mnemonic: String,
    pub unit: Option<String>,
    pub family: Option<String>,
    pub set_name: String,
    pub source: Option<String>,
    pub run_no: Option<i32>,
    pub n_samples: i64,
}

/// Lists every curve in the generic store for one well, across all sets — the data
/// source for the Curve Catalog's family/unit/set columns (Phase 6). Named distinctly
/// from `equations::list_curve_catalog` (the existing standard+computed catalog), which
/// remains the frontend's data source until the Phase 6 curve-store migration is wired
/// through the rest of the app (workflow modules, log views, equations).
pub fn list_generic_curve_catalog(conn: &Connection, well_id: &str) -> DbResult<Vec<GenericCurveCatalogEntry>> {
    let mut stmt = conn.prepare(
        "SELECT m.curve_id, m.mnemonic, m.unit, m.family, m.set_name, m.source, m.run_no,
                COUNT(s.depth)
         FROM curve_meta m
         LEFT JOIN curve_samples s ON s.curve_id = m.curve_id
         WHERE m.well_id = ?1
         GROUP BY m.curve_id, m.mnemonic, m.unit, m.family, m.set_name, m.source, m.run_no
         ORDER BY m.set_name, m.family, m.mnemonic",
    )?;
    let rows = stmt.query_map(params![well_id], |row| {
        Ok(GenericCurveCatalogEntry {
            curve_id: row.get(0)?,
            mnemonic: row.get(1)?,
            unit: row.get(2)?,
            family: row.get(3)?,
            set_name: row.get(4)?,
            source: row.get(5)?,
            run_no: row.get(6)?,
            n_samples: row.get(7)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Registers (or reuses, if the (well, set, mnemonic, run_no) already exists) one curve
/// in the generic store and returns its curve_id.
pub fn upsert_curve_meta(
    conn: &Connection,
    well_id: &str,
    set_name: &str,
    mnemonic: &str,
    unit: Option<&str>,
    family: Option<&str>,
    source: Option<&str>,
    run_no: Option<i32>,
) -> DbResult<String> {
    let existing: Option<String> = conn
        .query_row(
            "SELECT curve_id FROM curve_meta WHERE well_id = ?1 AND set_name = ?2 AND mnemonic = ?3
             AND run_no IS NOT DISTINCT FROM ?4",
            params![well_id, set_name, mnemonic, run_no],
            |r| r.get(0),
        )
        .ok();
    if let Some(id) = existing {
        conn.execute(
            "UPDATE curve_meta SET unit = ?1, family = ?2, source = ?3 WHERE curve_id = ?4",
            params![unit, family, source, id],
        )?;
        return Ok(id);
    }
    let curve_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO curve_meta (curve_id, well_id, set_name, mnemonic, unit, family, source, run_no)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![curve_id, well_id, set_name, mnemonic, unit, family, source, run_no],
    )?;
    Ok(curve_id)
}

/// Bulk-replaces the samples for one curve (delete-then-append, mirroring
/// `insert_core_data`'s replace-on-reimport semantics).
pub fn insert_curve_samples(conn: &Connection, curve_id: &str, depths: &[f32], values: &[f32]) -> DbResult<()> {
    if depths.len() != values.len() {
        return Err(DbError::LengthMismatch(format!(
            "depths ({}) and values ({}) must match",
            depths.len(),
            values.len()
        )));
    }
    conn.execute("DELETE FROM curve_samples WHERE curve_id = ?1", params![curve_id])?;
    let mut appender: Appender = conn.appender("curve_samples")?;
    for i in 0..depths.len() {
        appender.append_row(params![curve_id, depths[i], values[i]])?;
    }
    appender.flush()?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct CurveSamplePoint {
    pub depth: f32,
    pub value: f32,
}

/// Reads every sample of one curve, ordered by depth (NaN kept as-is for the frontend
/// to treat as missing, matching the `f32::NAN` = missing convention used everywhere else).
pub fn get_curve_samples(conn: &Connection, curve_id: &str) -> DbResult<Vec<CurveSamplePoint>> {
    let mut stmt = conn.prepare("SELECT depth, value FROM curve_samples WHERE curve_id = ?1 ORDER BY depth")?;
    let rows = stmt.query_map(params![curve_id], |row| {
        Ok(CurveSamplePoint { depth: row.get(0)?, value: row.get::<_, Option<f32>>(1)?.unwrap_or(f32::NAN) })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[derive(Debug, Clone, Serialize)]
pub struct WellPathStation {
    pub md: f32,
    pub inc: f32,
    pub azi: f32,
    pub tvd: f32,
    pub tvdss: f32,
}

/// Replaces the deviation survey (with computed TVD/TVDSS) for one well.
pub fn insert_well_path(conn: &Connection, well_id: &str, stations: &[crate::deviation::Station]) -> DbResult<()> {
    conn.execute("DELETE FROM well_path WHERE well_id = ?1", params![well_id])?;
    let mut appender: Appender = conn.appender("well_path")?;
    for s in stations {
        appender.append_row(params![well_id, s.md, s.inc, s.azi, s.tvd, s.tvdss])?;
    }
    appender.flush()?;
    Ok(())
}

/// Reads one well's deviation survey (ordered by MD) for TVD-aware display.
pub fn get_well_path(conn: &Connection, well_id: &str) -> DbResult<Vec<WellPathStation>> {
    let mut stmt =
        conn.prepare("SELECT md, inc, azi, tvd, tvdss FROM well_path WHERE well_id = ?1 ORDER BY md")?;
    let rows = stmt.query_map(params![well_id], |row| {
        Ok(WellPathStation {
            md: row.get(0)?,
            inc: row.get(1)?,
            azi: row.get(2)?,
            tvd: row.get::<_, Option<f32>>(3)?.unwrap_or(f32::NAN),
            tvdss: row.get::<_, Option<f32>>(4)?.unwrap_or(f32::NAN),
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn set_zone_param(
    conn: &Connection,
    well_id: &str,
    zone_name: &str,
    param_name: &str,
    value_num: Option<f32>,
    value_text: Option<&str>,
) -> DbResult<()> {
    if value_num.is_none() && value_text.is_none() {
        conn.execute(
            "DELETE FROM zone_params WHERE well_id = ?1 AND zone_name = ?2 AND param_name = ?3",
            params![well_id, zone_name, param_name],
        )?;
        return Ok(());
    }
    conn.execute(
        "INSERT INTO zone_params (well_id, zone_name, param_name, value_num, value_text) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (well_id, zone_name, param_name) DO UPDATE SET value_num = excluded.value_num, value_text = excluded.value_text",
        params![well_id, zone_name, param_name, value_num, value_text],
    )?;
    Ok(())
}
