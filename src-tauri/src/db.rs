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
        -- Surface location for the Field Map (Wave E item 22). Easting/northing are stored
        -- as DOUBLE (UTM metres reach ~10,000,000 in the southern hemisphere — beyond FLOAT's
        -- ~1 m precision at that magnitude). `utm_zone` is the metre grid's zone label (e.g.
        -- "50S" for the Mahakam Delta, "48S"/"49S" for ONWJ) so multi-zone fields can be
        -- distinguished; coordinates are plotted in their raw easting/northing. Added via
        -- ALTER so existing databases converge on the same shape.
        ALTER TABLE wells ADD COLUMN IF NOT EXISTS surface_x DOUBLE;
        ALTER TABLE wells ADD COLUMN IF NOT EXISTS surface_y DOUBLE;
        ALTER TABLE wells ADD COLUMN IF NOT EXISTS utm_zone VARCHAR;

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

        -- P1-c log-set versioning (2026-07-19). `set_id` tags every current row with the
        -- run event that produced it (NULL = legacy/unversioned). Added via ALTER so old
        -- and fresh databases converge on the same 5-column shape from one declaration.
        ALTER TABLE computed_curves ADD COLUMN IF NOT EXISTS set_id UUID;

        -- One row per RUN EVENT into a named log set: "re-run = version N+1, never
        -- overwrite". `version` counts up per (well, set_name); module/params/inputs are
        -- the per-curve provenance Jauhar asked for (what made this curve, from what,
        -- when). Deleting a set version keeps current values (their set_id goes NULL).
        CREATE TABLE IF NOT EXISTS log_sets (
            set_id      UUID PRIMARY KEY,
            well_id     UUID NOT NULL,
            set_name    VARCHAR NOT NULL,
            version     INTEGER NOT NULL,
            module      VARCHAR NOT NULL,     -- module name, 'workflow (N steps)', 'equation:X', ...
            params_json VARCHAR,              -- parameters of the run
            inputs_json VARCHAR,              -- resolved input curve mnemonics
            created_at  TIMESTAMP NOT NULL DEFAULT now()
        );

        -- Append-only history: every versioned run's full output rows, tagged by set_id.
        -- `computed_curves` stays the fast "current" store every panel reads; this table
        -- is what makes re-runs non-destructive (restore any version back into current).
        -- No PK on purpose — same appender-perf reasoning as computed_curves.
        CREATE TABLE IF NOT EXISTS computed_curves_archive (
            set_id      UUID NOT NULL,
            well_id     UUID NOT NULL,
            depth       FLOAT NOT NULL,
            curve_name  VARCHAR NOT NULL,
            value       FLOAT
        );

        -- User-authored petrophysical equations (Rhai scripts): a per-project formula/module registry.
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

        -- Formation tops / interval markers (a per-well tops interval log).
        CREATE TABLE IF NOT EXISTS tops (
            well_id     UUID NOT NULL,
            top_name    VARCHAR NOT NULL,
            depth       FLOAT NOT NULL,
            color       VARCHAR,
            PRIMARY KEY (well_id, top_name)
        );

        -- Depth intervals per well (zoned interval sets). Modules
        -- resolve their interval parameters per zone at run time.
        CREATE TABLE IF NOT EXISTS zones (
            well_id      UUID NOT NULL,
            zone_name    VARCHAR NOT NULL,
            top_depth    FLOAT NOT NULL,
            bottom_depth FLOAT NOT NULL,
            PRIMARY KEY (well_id, zone_name)
        );

        -- Per-zone interval parameter values (interval logs like GR_MA, GR_SH,
        -- RW, M, N). zone_name '*' holds whole-well defaults.
        CREATE TABLE IF NOT EXISTS zone_params (
            well_id      UUID NOT NULL,
            zone_name    VARCHAR NOT NULL,
            param_name   VARCHAR NOT NULL,
            value_num    FLOAT,
            value_text   VARCHAR,
            PRIMARY KEY (well_id, zone_name, param_name)
        );

        -- Informal colored depth-interval highlights for the log view (mark pay, bad hole,
        -- intervals of interest). Unlike zones they carry a color + free label and need no
        -- unique name, so they are keyed by a client-generated id and may overlap freely.
        CREATE TABLE IF NOT EXISTS highlights (
            well_id      UUID NOT NULL,
            highlight_id VARCHAR NOT NULL,
            top_depth    FLOAT NOT NULL,
            bottom_depth FLOAT NOT NULL,
            color        VARCHAR,
            label        VARCHAR,
            PRIMARY KEY (well_id, highlight_id)
        );

        -- Fluid contacts (OWC/GWC/GOC/GDT/ODT/FWL) for the correlation view. A contact is
        -- a single depth, not an interval, and is flat in TVDSS across a field (is_tvdss=true).
        -- Scope: well_id set -> that well only; field_name set (well_id NULL) -> every well in
        -- that field; both NULL -> a global datum applied to every well. Keyed by a
        -- client-generated id so several contacts (and duplicates) may coexist freely.
        CREATE TABLE IF NOT EXISTS fluid_contacts (
            contact_id   VARCHAR NOT NULL,
            field_name   VARCHAR,           -- field scope (NULL when well-scoped or global)
            well_id      VARCHAR,           -- well scope (NULL when field-scoped or global)
            contact_type VARCHAR NOT NULL,  -- OWC | GWC | GOC | GDT | ODT | FWL | custom
            depth        DOUBLE NOT NULL,
            is_tvdss     BOOLEAN NOT NULL,  -- true = depth is TVDSS (flat across wells), false = MD
            color        VARCHAR,
            label        VARCHAR,
            PRIMARY KEY (contact_id)
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

        -- Tops-style auxiliary datasets (petrography, XRD, perforations, …): sparse
        -- point or interval samples in long format. One row per (depth, item); values
        -- may be numeric (mineral %, grain size) or text (status, lithology remarks).
        -- Import replaces per (well, dataset) — same discipline as core_data.
        CREATE TABLE IF NOT EXISTS aux_data (
            well_id    UUID NOT NULL,
            dataset    VARCHAR NOT NULL,  -- 'PETROGRAPHY' | 'XRD' | 'PERFORATION' | custom
            depth_top  FLOAT NOT NULL,
            depth_base FLOAT,             -- NULL = point sample
            item       VARCHAR NOT NULL,  -- source column (QUARTZ, STATUS, …)
            value_num  FLOAT,
            value_text VARCHAR
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
        -- Lab fluid system per point (increment 2 hardening): which system the Pc was
        -- measured in ('air_brine', 'hg_air', 'oil_brine', ...) and its sigma·cosθ
        -- (dyn/cm) as entered at import. Lets mixed deliveries be told apart and
        -- standardized to one system later (Thomeer / J-from-SCAL). Added via ALTER so
        -- existing databases converge on the same shape; NULL = imported before this.
        ALTER TABLE scal_pc ADD COLUMN IF NOT EXISTS system VARCHAR;
        ALTER TABLE scal_pc ADD COLUMN IF NOT EXISTS ift FLOAT;

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
            pinned      INTEGER DEFAULT 0,  -- 1 = user-promoted winner for its (well,set,mnemonic)
            UNIQUE (well_id, set_name, mnemonic, run_no)
        );
        -- `pinned` added via ALTER so existing project databases converge on the same shape.
        ALTER TABLE curve_meta ADD COLUMN IF NOT EXISTS pinned INTEGER DEFAULT 0;

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
        -- Pinned wells: a lightweight, persisted "favourites" subset, independent of groups, so
        -- a handful of wells of interest stay one click away in every run dialog (the ★ toggle in
        -- the Wells pane). There is only ever one pinned set per project and, unlike an active
        -- group, it never filters the workspace on its own — it is purely a selection shortcut.
        CREATE TABLE IF NOT EXISTS well_pins (
            well_id     UUID PRIMARY KEY
        );
        -- Marks that the one-time standard_curves -> generic-store backfill has completed for a
        -- well (ALL six columns processed, whether they had data or not). Without this the
        -- migration re-scanned standard_curves for absent columns (DT/SP) on EVERY launch —
        -- ~20 s on a 540-well project. A well is recorded once fully processed, so later opens
        -- skip it; a newly imported well is simply absent here and gets migrated on the next open.
        CREATE TABLE IF NOT EXISTS curve_migration_done (
            well_id     UUID PRIMARY KEY
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

    // Only wells not yet fully backfilled. Once a well is in curve_migration_done it is skipped
    // entirely, so this whole function is ~instant on an already-migrated project instead of
    // re-scanning standard_curves for every well's absent columns on each launch.
    let mut stmt = conn
        .prepare("SELECT well_id FROM wells WHERE well_id NOT IN (SELECT well_id FROM curve_migration_done)")?;
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
        // All six columns processed (data-bearing ones copied, absent ones skipped) — record the
        // well so subsequent opens never re-scan it. Idempotent: on a crash between a partial
        // copy and this insert, the next run re-processes the well and the per-column
        // `already`-migrated check keeps it from duplicating curves.
        conn.execute(
            "INSERT INTO curve_migration_done (well_id) VALUES (?1) ON CONFLICT DO NOTHING",
            params![well_id],
        )?;
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
/// (LAS 2.0 / generic curve CSV) before batch insertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // deserialization DTO kept for the standard-curve CSV path
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

/// Runs `f` inside a single transaction: BEGIN, then COMMIT on Ok / ROLLBACK on Err. Makes a
/// delete-then-append sequence atomic, so an unclean process kill mid-write (the app's most
/// common failure mode — `tauri dev` restarts on every source change; see `init_db_resilient`)
/// can't leave the DELETE committed with the never-flushed append lost. NOTE: DuckDB has no
/// nested transactions — never call a `with_txn`-wrapped writer from inside another one.
pub fn with_txn<T, E, F>(conn: &Connection, f: F) -> Result<T, E>
where
    F: FnOnce(&Connection) -> Result<T, E>,
    E: From<duckdb::Error>,
{
    conn.execute_batch("BEGIN")?;
    match f(conn) {
        Ok(v) => {
            conn.execute_batch("COMMIT")?;
            Ok(v)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
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
    with_txn(conn, |conn| {
        conn.execute("DELETE FROM core_data WHERE well_id = ?1", params![well_id])?;
        let mut appender: Appender = conn.appender("core_data")?;
        for i in 0..n {
            appender.append_row(params![well_id, depths[i], cpor[i], cperm[i], cgd[i], csw[i]])?;
        }
        appender.flush()?;
        Ok(())
    })
}

/// One long-format row of a tops-style auxiliary dataset (see `aux_data` table).
#[derive(Debug, Clone, serde::Serialize)]
pub struct AuxRow {
    pub dataset: String,
    pub depth_top: f32,
    pub depth_base: Option<f32>,
    pub item: String,
    pub value_num: Option<f32>,
    pub value_text: Option<String>,
}

/// Replaces one well's rows of ONE dataset (petrography / XRD / perforation import).
pub fn insert_aux_data(conn: &Connection, well_id: &str, dataset: &str, rows: &[AuxRow]) -> DbResult<()> {
    with_txn(conn, |conn| {
        conn.execute(
            "DELETE FROM aux_data WHERE well_id = ?1 AND dataset = ?2",
            params![well_id, dataset],
        )?;
        let mut appender: Appender = conn.appender("aux_data")?;
        for r in rows {
            appender.append_row(params![
                well_id,
                dataset,
                r.depth_top,
                r.depth_base,
                r.item,
                r.value_num,
                r.value_text
            ])?;
        }
        appender.flush()?;
        Ok(())
    })
}

/// One well's auxiliary rows, all datasets or one, ordered by depth then item.
pub fn list_aux_data(conn: &Connection, well_id: &str, dataset: Option<&str>) -> DbResult<Vec<AuxRow>> {
    let mut stmt = conn.prepare(
        "SELECT dataset, depth_top, depth_base, item, value_num, value_text
         FROM aux_data
         WHERE well_id = ?1 AND (?2 IS NULL OR dataset = ?2)
         ORDER BY dataset, depth_top, item",
    )?;
    let rows = stmt.query_map(params![well_id, dataset], |row| {
        Ok(AuxRow {
            dataset: row.get(0)?,
            depth_top: row.get(1)?,
            depth_base: row.get(2)?,
            item: row.get(3)?,
            value_num: row.get(4)?,
            value_text: row.get(5)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Which auxiliary datasets a well has, with row counts (for panels/dialogs).
pub fn list_aux_datasets(conn: &Connection, well_id: &str) -> DbResult<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT dataset, COUNT(*) FROM aux_data WHERE well_id = ?1 GROUP BY dataset ORDER BY dataset",
    )?;
    let rows = stmt.query_map(params![well_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
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
    /// Lab fluid system ('air_brine', 'hg_air', 'oil_brine', ...); None = legacy import.
    pub system: Option<String>,
    /// sigma·cosθ of that system (dyn/cm) as entered at import.
    pub ift: Option<f32>,
}

/// Bulk-inserts SCAL capillary-pressure rows for one well, replacing any prior rows
/// (re-import overwrites, like `insert_core_data`).
pub fn insert_scal_pc(conn: &Connection, well_id: &str, rows: &[ScalPcRow]) -> DbResult<()> {
    with_txn(conn, |conn| {
        conn.execute("DELETE FROM scal_pc WHERE well_id = ?1", params![well_id])?;
        let mut appender: Appender = conn.appender("scal_pc")?;
        for r in rows {
            appender
                .append_row(params![well_id, r.sample_no, r.depth, r.perm, r.poro, r.pc, r.sw, r.system, r.ift])?;
        }
        appender.flush()?;
        Ok(())
    })
}

pub fn get_scal_pc(conn: &Connection, well_id: &str) -> DbResult<Vec<ScalPcRow>> {
    let mut stmt = conn.prepare(
        "SELECT sample_no, depth, perm, poro, pc, sw, system, ift FROM scal_pc
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
            system: row.get(6)?,
            ift: row.get(7)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// One routine-core-analysis plug: depth + porosity/permeability (NaN when the column was
/// blank). Used by the HFU clustering pane; core φ-k is the classic FZI clustering input.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CorePlugRow {
    pub depth: f32,
    pub cpor: f32,
    pub cperm: f32,
}

/// One well's core plugs (depth ascending) with porosity/permeability only. NULL φ or k
/// become NaN so the caller can skip them.
pub fn get_core_plugs(conn: &Connection, well_id: &str) -> DbResult<Vec<CorePlugRow>> {
    let mut stmt = conn.prepare(
        "SELECT depth, cpor, cperm FROM core_data WHERE well_id = ?1 ORDER BY depth",
    )?;
    let rows = stmt.query_map(params![well_id], |row| {
        Ok(CorePlugRow {
            depth: row.get(0)?,
            cpor: row.get::<_, Option<f32>>(1)?.unwrap_or(f32::NAN),
            cperm: row.get::<_, Option<f32>>(2)?.unwrap_or(f32::NAN),
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
    /// Total depth and Kelly-bushing elevation (metres). None until entered in the well
    /// header. Surfaced here so the header dialog can prefill them instead of opening blank —
    /// KB silently drives TVDSS in deviation import, so a blind edit poisons every TVDSS.
    pub td: Option<f32>,
    pub kb: Option<f32>,
    /// Surface easting/northing (UTM metres) and zone label, for the Field Map. None until
    /// imported (Import Well Locations) or entered in the well header.
    pub surface_x: Option<f64>,
    pub surface_y: Option<f64>,
    pub utm_zone: Option<String>,
}

/// Lists every well for the object tree, along with which curve tables actually hold data
/// for it (so the tree can show real children instead of a fixed guess).
pub fn list_wells(conn: &Connection) -> DbResult<Vec<WellSummary>> {
    let mut stmt = conn.prepare(
        "SELECT well_id, well_name, field_name, td, kb, surface_x, surface_y, utm_zone FROM wells ORDER BY well_name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(WellSummary {
            well_id: row.get(0)?,
            well_name: row.get(1)?,
            field_name: row.get(2)?,
            td: row.get(3)?,
            kb: row.get(4)?,
            surface_x: row.get(5)?,
            surface_y: row.get(6)?,
            utm_zone: row.get(7)?,
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

/// Lists the formation tops for one well, ordered by depth (a formation-tops
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
         ON CONFLICT (well_id, top_name) DO UPDATE SET depth = excluded.depth,
             color = COALESCE(excluded.color, tops.color)",
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

#[derive(Debug, Clone, Serialize)]
pub struct HighlightEntry {
    pub highlight_id: String,
    pub top_depth: f32,
    pub bottom_depth: f32,
    pub color: Option<String>,
    pub label: Option<String>,
}

pub fn list_highlights(conn: &Connection, well_id: &str) -> DbResult<Vec<HighlightEntry>> {
    let mut stmt = conn.prepare(
        "SELECT highlight_id, top_depth, bottom_depth, color, label FROM highlights WHERE well_id = ?1 ORDER BY top_depth",
    )?;
    let rows = stmt.query_map(params![well_id], |row| {
        Ok(HighlightEntry {
            highlight_id: row.get(0)?,
            top_depth: row.get(1)?,
            bottom_depth: row.get(2)?,
            color: row.get(3)?,
            label: row.get(4)?,
        })
    })?;
    let mut highlights = Vec::new();
    for r in rows {
        highlights.push(r?);
    }
    Ok(highlights)
}

pub fn upsert_highlight(
    conn: &Connection,
    well_id: &str,
    highlight_id: &str,
    top_depth: f32,
    bottom_depth: f32,
    color: Option<&str>,
    label: Option<&str>,
) -> DbResult<()> {
    conn.execute(
        "INSERT INTO highlights (well_id, highlight_id, top_depth, bottom_depth, color, label) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (well_id, highlight_id) DO UPDATE SET
             top_depth = excluded.top_depth, bottom_depth = excluded.bottom_depth,
             color = excluded.color, label = excluded.label",
        params![well_id, highlight_id, top_depth, bottom_depth, color, label],
    )?;
    Ok(())
}

pub fn delete_highlight(conn: &Connection, well_id: &str, highlight_id: &str) -> DbResult<()> {
    conn.execute(
        "DELETE FROM highlights WHERE well_id = ?1 AND highlight_id = ?2",
        params![well_id, highlight_id],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct FluidContact {
    pub contact_id: String,
    pub field_name: Option<String>,
    pub well_id: Option<String>,
    pub contact_type: String,
    pub depth: f64,
    pub is_tvdss: bool,
    pub color: Option<String>,
    pub label: Option<String>,
}

/// Every fluid contact in the project. There are few of these (one per reservoir/field),
/// so the correlation view fetches them all and decides per well which apply.
pub fn list_fluid_contacts(conn: &Connection) -> DbResult<Vec<FluidContact>> {
    let mut stmt = conn.prepare(
        "SELECT contact_id, field_name, well_id, contact_type, depth, is_tvdss, color, label
         FROM fluid_contacts ORDER BY depth",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(FluidContact {
            contact_id: row.get(0)?,
            field_name: row.get(1)?,
            well_id: row.get(2)?,
            contact_type: row.get(3)?,
            depth: row.get(4)?,
            is_tvdss: row.get(5)?,
            color: row.get(6)?,
            label: row.get(7)?,
        })
    })?;
    let mut contacts = Vec::new();
    for r in rows {
        contacts.push(r?);
    }
    Ok(contacts)
}

#[allow(clippy::too_many_arguments)]
pub fn upsert_fluid_contact(
    conn: &Connection,
    contact_id: &str,
    field_name: Option<&str>,
    well_id: Option<&str>,
    contact_type: &str,
    depth: f64,
    is_tvdss: bool,
    color: Option<&str>,
    label: Option<&str>,
) -> DbResult<()> {
    conn.execute(
        "INSERT INTO fluid_contacts (contact_id, field_name, well_id, contact_type, depth, is_tvdss, color, label)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT (contact_id) DO UPDATE SET
             field_name = excluded.field_name, well_id = excluded.well_id,
             contact_type = excluded.contact_type, depth = excluded.depth,
             is_tvdss = excluded.is_tvdss, color = excluded.color, label = excluded.label",
        params![contact_id, field_name, well_id, contact_type, depth, is_tvdss, color, label],
    )?;
    Ok(())
}

pub fn delete_fluid_contact(conn: &Connection, contact_id: &str) -> DbResult<()> {
    conn.execute("DELETE FROM fluid_contacts WHERE contact_id = ?1", params![contact_id])?;
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
    with_txn(conn, |conn| {
        conn.execute("DELETE FROM well_group_members WHERE group_id = ?1", params![group_id])?;
        for w in well_ids {
            conn.execute(
                "INSERT INTO well_group_members (group_id, well_id) VALUES (?1, ?2)
                 ON CONFLICT (group_id, well_id) DO NOTHING",
                params![group_id, w],
            )?;
        }
        Ok(())
    })
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

/// The pinned well ids — a persisted favourites subset, independent of groups (see `well_pins`).
pub fn list_pinned_wells(conn: &Connection) -> DbResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT well_id FROM well_pins")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Pins or unpins a single well (idempotent).
pub fn set_well_pin(conn: &Connection, well_id: &str, pinned: bool) -> DbResult<()> {
    if pinned {
        conn.execute("INSERT INTO well_pins (well_id) VALUES (?1) ON CONFLICT (well_id) DO NOTHING", params![well_id])?;
    } else {
        conn.execute("DELETE FROM well_pins WHERE well_id = ?1", params![well_id])?;
    }
    Ok(())
}

/// Replaces the whole pinned set (used by "pin selection" and "clear pins").
pub fn set_pinned_wells(conn: &Connection, well_ids: &[String]) -> DbResult<()> {
    with_txn(conn, |conn| {
        conn.execute("DELETE FROM well_pins", [])?;
        for w in well_ids {
            conn.execute("INSERT INTO well_pins (well_id) VALUES (?1) ON CONFLICT (well_id) DO NOTHING", params![w])?;
        }
        Ok(())
    })
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

    with_txn(conn, |conn| {
        conn.execute("DELETE FROM zones WHERE well_id = ?1", params![well_id])?;
        let mut zones = Vec::new();
        for (i, top) in tops.iter().enumerate() {
            let bottom = tops.get(i + 1).map(|t| t.depth).unwrap_or_else(|| max_depth.max(top.depth));
            upsert_zone(conn, well_id, &top.top_name, top.depth, bottom)?;
            zones.push(ZoneEntry { zone_name: top.top_name.clone(), top_depth: top.depth, bottom_depth: bottom });
        }
        Ok(zones)
    })
}

// ---------------------------------------------------------------------------
// Database inspector (spreadsheet-grid equivalent): paged reads over a whitelist
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
    ("aux_data", &["dataset", "depth_top", "depth_base", "item", "value_num", "value_text"], true, "dataset, depth_top, item"),
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

/// Runs one read-only SELECT (a SQL console, full DuckDB SQL: joins,
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

    /// Launch-perf fix: the migration records each processed well in `curve_migration_done` so it
    /// is never re-scanned on later opens (the ~20 s-per-launch cost on 540 wells), while a well
    /// imported AFTER a migration still gets backfilled on the next run.
    #[test]
    fn migration_marks_wells_done_and_only_touches_new_wells() {
        let conn = mem_db();
        let a = Uuid::new_v4();
        insert_well(&conn, a, "A", None, None, None).unwrap();
        // GR/RES/NPHI/RHOB have data; DT/SP are absent (all NaN) — the columns that used to be
        // re-scanned every boot because no sentinel was planted for them.
        insert_standard_curves(
            &conn, a, vec![1000.0, 1001.0],
            vec![50.0, 60.0], vec![10.0, 11.0], vec![0.3, 0.3], vec![2.4, 2.4],
            vec![f32::NAN, f32::NAN], vec![f32::NAN, f32::NAN],
        )
        .unwrap();
        migrate_standard_curves_to_generic_store(&conn).unwrap();

        // Well A is now marked done (all six columns, incl. absent DT/SP).
        let done_a: i64 = conn
            .query_row("SELECT COUNT(*) FROM curve_migration_done WHERE well_id = ?1", params![a.to_string()], |r| r.get(0))
            .unwrap();
        assert_eq!(done_a, 1, "A should be recorded as migrated");

        // Import a second well AFTER the first migration — only it should be processed next run.
        let b = Uuid::new_v4();
        insert_well(&conn, b, "B", None, None, None).unwrap();
        insert_standard_curves(
            &conn, b, vec![2000.0, 2001.0],
            vec![70.0, 80.0], vec![5.0, 6.0], vec![0.2, 0.2], vec![2.5, 2.5],
            vec![f32::NAN, f32::NAN], vec![f32::NAN, f32::NAN],
        )
        .unwrap();
        migrate_standard_curves_to_generic_store(&conn).unwrap();

        // B is now migrated and marked; A is untouched (still exactly one GR, not duplicated).
        let cat_b = list_generic_curve_catalog(&conn, &b.to_string()).unwrap();
        assert!(cat_b.iter().any(|c| c.mnemonic == "GR"), "new well B migrates on the next open");
        let done_all: i64 = conn.query_row("SELECT COUNT(*) FROM curve_migration_done", [], |r| r.get(0)).unwrap();
        assert_eq!(done_all, 2, "both wells recorded as migrated");
        let cat_a = list_generic_curve_catalog(&conn, &a.to_string()).unwrap();
        assert_eq!(cat_a.iter().filter(|c| c.mnemonic == "GR").count(), 1, "A must not be re-migrated/duplicated");
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

        // A depth that matches no row must ERROR (0-row UPDATE), not report silent success —
        // otherwise the DB inspector shows a phantom edit and pushes a bogus undo entry.
        assert!(
            update_computed_sample(&conn, &a, 4321.0, "PHIE", 0.1).is_err(),
            "editing a non-existent depth must error, not silently affect 0 rows"
        );
    }

    /// P1-c log-set versioning: re-runs bump the version and preserve history in the
    /// archive; any version can be restored into current; deleting a version keeps
    /// current values (provenance tag cleared); the catalog reports provenance + stats.
    #[test]
    fn log_set_versioning_never_overwrites() {
        use crate::equations::{
            create_log_set, delete_log_set, list_computed_catalog, list_log_sets, restore_log_set,
            write_computed_curves_versioned, LogSetSpec,
        };
        let conn = mem_db();
        let w = Uuid::new_v4().to_string();
        let depth = [1000.0f32, 1000.5, 1001.0];
        let spec = LogSetSpec {
            set_name: "INTERP".into(),
            module: "vsh_gr".into(),
            params_json: "{\"GR_MA\":25}".into(),
            inputs_json: "[\"GR\"]".into(),
        };

        // Run 1 → version 1; run 2 (different values) → version 2.
        let (set1, v1) = create_log_set(&conn, &w, &spec).unwrap();
        write_computed_curves_versioned(&conn, &w, &depth, &[("VSH", &[0.10, 0.20, 0.30])], &set1).unwrap();
        let (set2, v2) = create_log_set(&conn, &w, &spec).unwrap();
        write_computed_curves_versioned(&conn, &w, &depth, &[("VSH", &[0.90, 0.80, 0.70])], &set2).unwrap();
        assert_eq!((v1, v2), (1, 2), "re-run bumps the version");

        let current = |d: f32| -> f32 {
            conn.query_row(
                "SELECT value FROM computed_curves WHERE well_id = ?1 AND curve_name = 'VSH' AND depth = ?2",
                params![w, d],
                |r| r.get(0),
            )
            .unwrap()
        };
        let n_current: i64 = conn
            .query_row("SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1 AND curve_name = 'VSH'", params![w], |r| r.get(0))
            .unwrap();
        assert_eq!(n_current, 3, "current store holds exactly one generation");
        assert!((current(1000.0) - 0.90).abs() < 1e-6, "current = latest run");
        let n_archive: i64 = conn
            .query_row("SELECT COUNT(*) FROM computed_curves_archive WHERE well_id = ?1", params![w], |r| r.get(0))
            .unwrap();
        assert_eq!(n_archive, 6, "archive keeps BOTH generations — nothing overwritten");

        // Version history lists newest first with provenance + curve names.
        let sets = list_log_sets(&conn, &w).unwrap();
        assert_eq!(sets.len(), 2);
        assert_eq!((sets[0].version, sets[1].version), (2, 1));
        assert_eq!(sets[0].module, "vsh_gr");
        assert_eq!(sets[0].curve_names, vec!["VSH".to_string()]);
        assert!(sets[0].is_current && !sets[1].is_current);

        // Restore version 1 → current shows the old values again; archive untouched.
        let restored = restore_log_set(&conn, &set1).unwrap();
        assert_eq!(restored, 3);
        assert!((current(1000.0) - 0.10).abs() < 1e-6, "restored to version 1");

        // Catalog: provenance of the current value now points at version 1, stats sane.
        let cat = list_computed_catalog(&conn, &w).unwrap();
        let vsh = cat.iter().find(|e| e.curve_name == "VSH").unwrap();
        assert_eq!(vsh.set_name.as_deref(), Some("INTERP"));
        assert_eq!(vsh.version, Some(1));
        assert_eq!(vsh.n_samples, 3);
        assert!((vsh.min.unwrap() - 0.10).abs() < 1e-6 && (vsh.max.unwrap() - 0.30).abs() < 1e-6);

        // Deleting version 2's history keeps current values; v1 remains restorable.
        delete_log_set(&conn, &set2).unwrap();
        assert_eq!(list_log_sets(&conn, &w).unwrap().len(), 1);
        assert!((current(1000.0) - 0.10).abs() < 1e-6, "delete never changes current values");
        let n_archive: i64 = conn
            .query_row("SELECT COUNT(*) FROM computed_curves_archive WHERE well_id = ?1", params![w], |r| r.get(0))
            .unwrap();
        assert_eq!(n_archive, 3, "only version 2's history removed");
    }

    /// Batched multi-well versioned write (the field-scale write path): many wells land in ONE
    /// transaction via the grouped-DELETE + single-appender-per-table fast path. Locks the two
    /// things that path must not break — (1) grouping wells by curve-set and deleting the exact
    /// (wells × curves) cross product never touches a curve a well doesn't have, and (2) a re-run
    /// replaces current values while the archive keeps every generation, per well independently.
    #[test]
    fn batched_versioned_write_is_correct_across_wells_and_reruns() {
        use crate::equations::{
            create_log_sets_batch, list_log_sets, write_computed_curves_versioned_batch, LogSetSpec,
            WellWrite,
        };
        let conn = mem_db();
        let w1 = Uuid::new_v4().to_string();
        let w2 = Uuid::new_v4().to_string();
        let depth = vec![1000.0f32, 1000.5, 1001.0];
        let spec = LogSetSpec {
            set_name: "INTERP".into(),
            module: "phi_den".into(),
            params_json: "{}".into(),
            inputs_json: "[\"RHOB\"]".into(),
        };
        // Deliberately DIFFERENT curve-sets per well → two DELETE groups, so a bug that deletes
        // the union cross product would wipe w2's non-existent PHIE row or strand w1's PHIE.
        let run = |conn: &Connection, vsh1: [f32; 3], phie1: [f32; 3], vsh2: [f32; 3]| {
            let ids = [w1.clone(), w2.clone()];
            let sets = create_log_sets_batch(conn, &ids, &spec).unwrap();
            let writes = vec![
                WellWrite {
                    well_id: w1.clone(),
                    depth: depth.clone(),
                    curves: vec![("VSH".into(), vsh1.to_vec()), ("PHIE".into(), phie1.to_vec())],
                    set_id: sets[&w1].clone(),
                },
                WellWrite {
                    well_id: w2.clone(),
                    depth: depth.clone(),
                    curves: vec![("VSH".into(), vsh2.to_vec())],
                    set_id: sets[&w2].clone(),
                },
            ];
            write_computed_curves_versioned_batch(conn, &writes).unwrap();
        };
        let cur = |well: &str, curve: &str, d: f32| -> f32 {
            conn.query_row(
                "SELECT value FROM computed_curves WHERE well_id = ?1 AND curve_name = ?2 AND depth = ?3",
                params![well, curve, d],
                |r| r.get(0),
            )
            .unwrap()
        };
        let count = |sql: &str, well: &str| -> i64 {
            conn.query_row(sql, params![well], |r| r.get(0)).unwrap()
        };

        run(&conn, [0.10, 0.20, 0.30], [0.15, 0.16, 0.17], [0.40, 0.50, 0.60]);
        assert!((cur(&w1, "VSH", 1000.0) - 0.10).abs() < 1e-6);
        assert!((cur(&w1, "PHIE", 1000.0) - 0.15).abs() < 1e-6);
        assert!((cur(&w2, "VSH", 1000.0) - 0.40).abs() < 1e-6);
        assert_eq!(count("SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1", &w1), 6);
        assert_eq!(count("SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1", &w2), 3);

        // Re-run: current is replaced (still one generation each), archive accumulates both.
        run(&conn, [0.90, 0.80, 0.70], [0.25, 0.26, 0.27], [0.44, 0.55, 0.66]);
        assert!((cur(&w1, "VSH", 1000.0) - 0.90).abs() < 1e-6, "re-run replaces current");
        assert!((cur(&w1, "PHIE", 1000.0) - 0.25).abs() < 1e-6);
        assert!((cur(&w2, "VSH", 1000.0) - 0.44).abs() < 1e-6);
        assert_eq!(count("SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1", &w1), 6, "current still one generation");
        assert_eq!(count("SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1", &w2), 3);
        assert_eq!(count("SELECT COUNT(*) FROM computed_curves_archive WHERE well_id = ?1", &w1), 12, "archive keeps both runs");
        assert_eq!(count("SELECT COUNT(*) FROM computed_curves_archive WHERE well_id = ?1", &w2), 6);

        assert_eq!(list_log_sets(&conn, &w1).unwrap().len(), 2, "two versions recorded per well");
        assert_eq!(list_log_sets(&conn, &w2).unwrap().len(), 2);
    }

    /// Input-set selection (the read half of "set in/out"): a module asking for VSH from
    /// set FINAL gets FINAL's archived values even after a later INTERP run replaced the
    /// current VSH — while curves the set never wrote (GR) still resolve normally.
    #[test]
    fn input_set_selection_reads_archived_values() {
        use crate::equations::{
            create_log_set, fetch_curve_frame_from_set, write_computed_curves_versioned, LogSetSpec,
        };
        let conn = mem_db();
        let w = Uuid::new_v4();
        insert_well(&conn, w, "SET_IN_TEST", None, None, None).unwrap();
        let depth = vec![1000.0f32, 1000.5, 1001.0];
        let gr = vec![45.0f32, 60.0, 75.0];
        let filler = vec![1.0f32; 3];
        insert_standard_curves(&conn, w, depth.clone(), gr, filler.clone(), filler.clone(), filler.clone(), filler.clone(), filler).unwrap();
        let w = w.to_string();

        let spec = |set: &str| LogSetSpec {
            set_name: set.into(),
            module: "vsh_gr".into(),
            params_json: String::new(),
            inputs_json: String::new(),
        };
        let (final_set, _) = create_log_set(&conn, &w, &spec("FINAL")).unwrap();
        write_computed_curves_versioned(&conn, &w, &depth, &[("VSH", &[0.10, 0.20, 0.30])], &final_set).unwrap();
        let (interp_set, _) = create_log_set(&conn, &w, &spec("INTERP")).unwrap();
        write_computed_curves_versioned(&conn, &w, &depth, &[("VSH", &[0.90, 0.80, 0.70])], &interp_set).unwrap();

        let names = vec!["VSH".to_string(), "GR".to_string()];
        // No input set → current values (the later INTERP run).
        let (_, cols) = fetch_curve_frame_from_set(&conn, &w, &names, None, None).unwrap();
        assert!((cols["VSH"][0] - 0.90).abs() < 1e-6, "default = current store");
        // FINAL (case-insensitive) → its archived VSH; GR falls back to standard curves.
        let (_, cols) = fetch_curve_frame_from_set(&conn, &w, &names, Some("final"), None).unwrap();
        assert!((cols["VSH"][0] - 0.10).abs() < 1e-6, "reads the chosen set's archive");
        assert!((cols["GR"][1] - 60.0).abs() < 1e-6, "unwritten curves fall back normally");
        // Unknown set name degrades to the plain frame, not an error.
        let (_, cols) = fetch_curve_frame_from_set(&conn, &w, &names, Some("NOPE"), None).unwrap();
        assert!((cols["VSH"][0] - 0.90).abs() < 1e-6);
        // Chain protection: when this run's OWN set already wrote VSH (an earlier step),
        // the input set must not shadow it — the fresh current value wins.
        let (_, cols) =
            fetch_curve_frame_from_set(&conn, &w, &names, Some("FINAL"), Some(&interp_set)).unwrap();
        assert!((cols["VSH"][0] - 0.90).abs() < 1e-6, "own-run outputs beat the input set");
    }
}

/// Edits one wells-table field (name/field/utm_zone as text, td/kb as f32, surface_x/y as f64).
pub fn update_well_field(conn: &Connection, well_id: &str, field: &str, value: Option<&str>) -> Result<(), String> {
    match field {
        "well_name" | "field_name" | "utm_zone" => {
            let text = value.map(str::trim).filter(|s| !s.is_empty());
            conn.execute(&format!("UPDATE wells SET {field} = ?1 WHERE well_id = ?2"), params![text, well_id])
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
        "surface_x" | "surface_y" => {
            let num: Option<f64> = match value {
                Some(v) if !v.trim().is_empty() => Some(v.trim().parse::<f64>().map_err(|e| e.to_string())?),
                _ => None,
            };
            conn.execute(&format!("UPDATE wells SET {field} = ?1 WHERE well_id = ?2"), params![num, well_id])
                .map_err(|e| e.to_string())?;
        }
        other => return Err(format!("field '{other}' is not editable")),
    }
    Ok(())
}

/// Sets (or clears, with None) a well's surface location in one write — the target of the
/// Import Well Locations CSV path. `zone` is trimmed; empty becomes NULL.
pub fn set_well_location(
    conn: &Connection,
    well_id: &str,
    x: Option<f64>,
    y: Option<f64>,
    zone: Option<&str>,
) -> DbResult<()> {
    let zone = zone.map(str::trim).filter(|s| !s.is_empty());
    conn.execute(
        "UPDATE wells SET surface_x = ?2, surface_y = ?3, utm_zone = ?4 WHERE well_id = ?1",
        params![well_id, x, y, zone],
    )?;
    Ok(())
}

/// Edits one standard-curve sample value (NaN = missing).
pub fn update_standard_sample(conn: &Connection, well_id: &str, depth: f32, column: &str, value: f32) -> Result<(), String> {
    const EDITABLE: &[&str] = &["gr", "res_deep", "nphi", "rhob", "dt", "sp"];
    if !EDITABLE.contains(&column) {
        return Err(format!("column '{column}' is not editable"));
    }
    let n = conn
        .execute(
            &format!("UPDATE standard_curves SET {column} = ?1 WHERE well_id = ?2 AND depth = ?3"),
            params![value, well_id, depth],
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err(format!(
            "no standard-curve sample matched depth {depth} — the row may have moved or been rewritten; refresh and retry"
        ));
    }
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
    let n = conn
        .execute(
            &format!("UPDATE core_data SET {column} = ?1 WHERE well_id = ?2 AND depth = ?3"),
            params![value, well_id, depth],
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err(format!(
            "no core sample matched depth {depth} — the row may have moved or been rewritten; refresh and retry"
        ));
    }
    Ok(())
}

/// Edits one computed-curve sample value.
pub fn update_computed_sample(conn: &Connection, well_id: &str, depth: f32, curve_name: &str, value: f32) -> Result<(), String> {
    let n = conn
        .execute(
            "UPDATE computed_curves SET value = ?1 WHERE well_id = ?2 AND depth = ?3 AND curve_name = ?4",
            params![value, well_id, depth, curve_name],
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err(format!(
            "no {curve_name} sample matched depth {depth} — the row may have moved or been rewritten; refresh and retry"
        ));
    }
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
    /// True when the user has promoted this curve to win its (well, set, mnemonic) group in
    /// curve resolution (the DLIS/LAS same-mnemonic shadow tiebreak).
    pub pinned: bool,
}

/// Lists every curve in the generic store for one well, across all sets — the data
/// source for the Curve Catalog's family/unit/set columns (Phase 6). Named distinctly
/// from `equations::list_curve_catalog` (the existing standard+computed catalog), which
/// remains the frontend's data source until the Phase 6 curve-store migration is wired
/// through the rest of the app (workflow modules, log views, equations).
pub fn list_generic_curve_catalog(conn: &Connection, well_id: &str) -> DbResult<Vec<GenericCurveCatalogEntry>> {
    let mut stmt = conn.prepare(
        "SELECT m.curve_id, m.mnemonic, m.unit, m.family, m.set_name, m.source, m.run_no,
                COUNT(s.depth), COALESCE(m.pinned, 0)
         FROM curve_meta m
         LEFT JOIN curve_samples s ON s.curve_id = m.curve_id
         WHERE m.well_id = ?1
         GROUP BY m.curve_id, m.mnemonic, m.unit, m.family, m.set_name, m.source, m.run_no, m.pinned
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
            pinned: row.get::<_, i32>(8)? != 0,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Deletes one generic-store curve — its `curve_meta` row and all its `curve_samples` — by id.
/// Irreversible; used by the Curve Catalog to remove a shadowing/duplicate imported curve.
pub fn delete_generic_curve(conn: &Connection, curve_id: &str) -> DbResult<()> {
    with_txn(conn, |conn| {
        conn.execute("DELETE FROM curve_samples WHERE curve_id = ?1", params![curve_id])?;
        conn.execute("DELETE FROM curve_meta WHERE curve_id = ?1", params![curve_id])?;
        Ok(())
    })
}

/// Promotes one generic curve to WIN its (well, set, mnemonic) group in curve resolution:
/// clears `pinned` on every sibling sharing that key, then sets `pinned = 1` on this curve.
/// At most one curve per (well, set, mnemonic) group is ever pinned. The resolvers apply that
/// pin only when the REQUEST is that exact mnemonic (see `fetch_generic_curve_aligned`), so it
/// resolves DLIS/LAS same-mnemonic shadowing without hijacking a family-name request that
/// happens to match a different mnemonic in the same family — and without deleting the loser.
pub fn promote_generic_curve(conn: &Connection, curve_id: &str) -> DbResult<()> {
    with_txn(conn, |conn| {
        let (well, set, mnem): (String, String, String) = conn.query_row(
            "SELECT well_id, set_name, mnemonic FROM curve_meta WHERE curve_id = ?1",
            params![curve_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        conn.execute(
            "UPDATE curve_meta SET pinned = 0
             WHERE well_id = ?1 AND set_name = ?2 AND upper(mnemonic) = upper(?3)",
            params![well, set, mnem],
        )?;
        conn.execute("UPDATE curve_meta SET pinned = 1 WHERE curve_id = ?1", params![curve_id])?;
        Ok(())
    })
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
    with_txn(conn, |conn| {
        conn.execute("DELETE FROM curve_samples WHERE curve_id = ?1", params![curve_id])?;
        let mut appender: Appender = conn.appender("curve_samples")?;
        for i in 0..depths.len() {
            appender.append_row(params![curve_id, depths[i], values[i]])?;
        }
        appender.flush()?;
        Ok(())
    })
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
    with_txn(conn, |conn| {
        conn.execute("DELETE FROM well_path WHERE well_id = ?1", params![well_id])?;
        let mut appender: Appender = conn.appender("well_path")?;
        for s in stations {
            appender.append_row(params![well_id, s.md, s.inc, s.azi, s.tvd, s.tvdss])?;
        }
        appender.flush()?;
        Ok(())
    })
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
