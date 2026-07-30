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
    #[error("{0}")]
    FormatTooNew(String),
    /// A caller-supplied value the database refuses (e.g. a blank curve name). Carries a
    /// message written for the user, not a diagnostic.
    #[error("{0}")]
    Invalid(String),
}

pub type DbResult<T> = Result<T, DbError>;

/// The project-file format this build reads and writes. Bump rules live in
/// `docs/RELEASE.md` §2.1: additive tables/columns do NOT bump this; a change an older
/// build would silently misread DOES (and is a MAJOR release). The stamp exists so an
/// OLDER app can refuse a NEWER file by name instead of opening it, finding only the
/// tables it knows, and presenting a partial project as the whole thing (RELEASE §3.1).
pub const FORMAT_VERSION: i64 = 1;

/// Opens (creating if needed) the embedded DuckDB file and applies the schema.
///
/// The format check runs BEFORE `create_schema` on purpose: `CREATE TABLE IF NOT EXISTS`
/// is itself a mutation, and a file written by a newer SandiBumi must be refused
/// untouched, not first edited into a hybrid of two formats.
pub fn init_db(path: &str) -> DbResult<Connection> {
    let conn = Connection::open(path)?;
    tune_connection(&conn);
    check_and_stamp_format(&conn)?;
    create_schema(&conn)?;
    Ok(conn)
}

/// Caps DuckDB's memory appetite. The engine's factory default allows itself ~80% of the
/// machine's RAM, which it will happily fill during a large scan, migration backup or
/// COPY FROM DATABASE — on the 2.5 GB BLSO project that showed up as ~6 GB of the user's
/// 8 GB machine. A desktop app sharing the machine gets a QUARTER of that default
/// (≈20% of RAM), clamped to [1 GiB, 4 GiB]; anything bigger spills to DuckDB's on-disk
/// temp space (enabled by default for file-backed databases). `SANDIBUMI_DB_MEMORY`
/// overrides the cap verbatim (e.g. "8GB") for power users on big field machines.
///
/// Never fatal: a database that opens with the default limit is strictly better than one
/// that refuses to open over a tuning pragma, so every failure here just logs.
fn tune_connection(conn: &Connection) {
    let limit = match std::env::var("SANDIBUMI_DB_MEMORY") {
        Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => {
            // The default limit is 80% of physical RAM, so it doubles as a RAM probe:
            // default/4 ≈ 20% of the machine, without any OS-specific API.
            let default_bytes = conn
                .query_row("SELECT current_setting('memory_limit')", [], |r| r.get::<_, String>(0))
                .ok()
                .and_then(|s| parse_mem_bytes(&s));
            const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
            let capped = (default_bytes.unwrap_or(8.0 * GIB) / 4.0).clamp(GIB, 4.0 * GIB);
            format!("{}MiB", (capped / (1024.0 * 1024.0)).round() as u64)
        }
    };
    if let Err(e) = conn.execute_batch(&format!("SET memory_limit='{}'", limit.replace('\'', ""))) {
        boot_note(format!("memory cap '{limit}' not applied ({e}); running with the engine default"));
    } else {
        boot_note(format!("DuckDB memory capped at {limit}"));
    }
}

/// Parses DuckDB's human-readable memory sizes ("6.4 GiB", "512.0 MiB") into bytes.
fn parse_mem_bytes(s: &str) -> Option<f64> {
    let mut parts = s.split_whitespace();
    let num: f64 = parts.next()?.parse().ok()?;
    let mult = match parts.next()? {
        "B" => 1.0,
        "KiB" | "KB" => 1024.0,
        "MiB" | "MB" => 1024.0 * 1024.0,
        "GiB" | "GB" => 1024.0 * 1024.0 * 1024.0,
        "TiB" | "TB" => 1024.0f64.powi(4),
        _ => return None,
    };
    Some(num * mult)
}

/// Boot/maintenance notices the USER should see (one-time migration backups, memory caps,
/// compaction results). `eprintln!` alone is invisible in a built exe
/// (`windows_subsystem = "windows"` has no console), which is exactly how a 15-minute
/// one-time migration ended up looking like a hang — so noteworthy events are queued here
/// and the frontend drains them into the status line / process history via `boot_report`.
static BOOT_NOTES: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

pub fn boot_note(msg: String) {
    eprintln!("[boot] {msg}");
    BOOT_NOTES.lock().unwrap().push(msg);
}

/// Drains the queued notices (each is returned exactly once).
pub fn take_boot_notes() -> Vec<String> {
    std::mem::take(&mut *BOOT_NOTES.lock().unwrap())
}

/// RELEASE.md §3.1 (requirement R-A). Three cases:
/// - no `project_meta` table — a fresh file OR a legacy pre-stamp project; both are by
///   definition ≤ this build's format, so create the table and stamp them (additive —
///   exempt from the R-B backup rule).
/// - stamped ≤ `FORMAT_VERSION` — open normally; when older, re-stamp to current (the
///   launch migrations in `project::open_and_migrate` bring the schema forward anyway,
///   so after this open the file IS the current format — one-way, per RELEASE §3.3).
/// - stamped > `FORMAT_VERSION` — refuse, naming both versions and the app that wrote
///   the file. Silently misreading a newer project is the one unacceptable behaviour.
fn check_and_stamp_format(conn: &Connection) -> DbResult<()> {
    let has_meta: i64 = conn.query_row(
        "SELECT count(*) FROM duckdb_tables() WHERE table_name = 'project_meta'",
        [],
        |r| r.get(0),
    )?;
    let stamp = |written_by: &str| -> DbResult<()> {
        conn.execute(
            "UPDATE project_meta SET value = ? WHERE key = 'format_version'",
            params![FORMAT_VERSION.to_string()],
        )?;
        conn.execute(
            "INSERT INTO project_meta SELECT 'format_version', ? WHERE NOT EXISTS
                 (SELECT 1 FROM project_meta WHERE key = 'format_version')",
            params![FORMAT_VERSION.to_string()],
        )?;
        conn.execute("DELETE FROM project_meta WHERE key = 'written_by'", [])?;
        conn.execute(
            "INSERT INTO project_meta VALUES ('written_by', ?)",
            params![written_by],
        )?;
        Ok(())
    };
    let app = concat!("SandiBumi ", env!("CARGO_PKG_VERSION"));
    if has_meta == 0 {
        conn.execute_batch("CREATE TABLE project_meta (key VARCHAR PRIMARY KEY, value VARCHAR);")?;
        return stamp(app);
    }
    // A present table with a missing/unparsable version row is treated as legacy (0),
    // never as newer — refusing must require positive evidence of a newer writer.
    let (ver, written_by): (i64, String) = conn
        .query_row(
            "SELECT
                coalesce(max(CASE WHEN key = 'format_version' THEN value END), '0'),
                coalesce(max(CASE WHEN key = 'written_by' THEN value END), 'an unknown SandiBumi build')
             FROM project_meta",
            [],
            |r| Ok((r.get::<_, String>(0)?.parse::<i64>().unwrap_or(0), r.get(1)?)),
        )?;
    if ver > FORMAT_VERSION {
        return Err(DbError::FormatTooNew(format!(
            "this project was written by {written_by} (file format {ver}); this build reads \
             format {FORMAT_VERSION} and lower - upgrade SandiBumi to open it (the file was \
             left unmodified)"
        )));
    }
    if ver < FORMAT_VERSION {
        stamp(app)?;
    }
    Ok(())
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
        -- The unit every depth stored for this well is in. Equal to the project's declared
        -- depth unit (units.rs) — a file whose index was in the other unit is converted on
        -- import, so this records what the numbers MEAN, not what the source file said.
        -- Null on wells imported before unit handling existed; those are read as the
        -- project unit, which is what they already were in practice.
        ALTER TABLE wells ADD COLUMN IF NOT EXISTS depth_unit VARCHAR;
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

        -- Array logs: one row per (well, set, curve, depth) holding a WHOLE VECTOR of values
        -- at that depth — Monte Carlo realizations, NMR T2 distributions, sonic waveforms.
        --
        -- `samples` is a BLOB of little-endian f32 (bytemuck), NOT a DuckDB FLOAT[] list.
        -- Rule 1 allows either; the blob wins here because it is exactly 4 bytes per value with
        -- no text round-trip, and because rule 3 already requires arrays to reach the frontend
        -- as bytemuck bytes cast to a Float32Array — so the stored bytes ARE the wire format
        -- and the read path never re-encodes.
        --
        -- Unlike `computed_curves` this table DOES carry a primary key, and that is not an
        -- inconsistency: the ART index that dominated computed_curves costs one entry per
        -- SAMPLE, whereas here one row holds a thousand samples, so the same index is ~1000x
        -- cheaper per value — while the protection matters far more. A duplicated depth row
        -- would silently double a realization count and bias every percentile drawn from it.
        CREATE TABLE IF NOT EXISTS array_logs (
            well_id     UUID NOT NULL,
            set_name    VARCHAR NOT NULL DEFAULT 'RAW',
            curve_name  VARCHAR NOT NULL,
            depth       FLOAT NOT NULL,
            samples     BLOB NOT NULL,
            PRIMARY KEY (well_id, set_name, curve_name, depth)
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
        -- `set_name` versions the delivery (T-IMP-08): a well can hold RCAL, a SCAL plug
        -- set and a corrected re-delivery side by side, and an import NEVER overwrites an
        -- earlier one (names auto-suffix, as curve sets do). Unlike curve sets, core sets
        -- do NOT union: two deliveries measure the SAME plugs, so exactly one set is
        -- ACTIVE per well and every reader sees only that one (`core_sets.active`).
        CREATE TABLE IF NOT EXISTS core_data (
            well_id     UUID NOT NULL,
            set_name    VARCHAR NOT NULL DEFAULT 'RAW',
            depth       FLOAT NOT NULL,
            cpor        FLOAT, -- core porosity, v/v
            cperm       FLOAT, -- core permeability, mD
            cgd         FLOAT, -- core grain density, g/cc
            csw         FLOAT, -- core water saturation, v/v
            PRIMARY KEY (well_id, set_name, depth)
        );

        -- Registry of a well's core deliveries: which exist, where they came from, and
        -- which one is live. Exactly 0 or 1 active per well, enforced in code (same
        -- discipline as `well_groups.active`).
        CREATE TABLE IF NOT EXISTS core_sets (
            well_id     UUID NOT NULL,
            set_name    VARCHAR NOT NULL,
            active      INTEGER NOT NULL DEFAULT 0,
            source      VARCHAR,           -- file the delivery came from
            imported_at TIMESTAMP NOT NULL DEFAULT now(),
            PRIMARY KEY (well_id, set_name)
        );

        -- Tops-style auxiliary datasets (petrography, XRD, CEC, oil show, perforations,
        -- core extras, …): sparse point or interval samples in long format. One row per
        -- (depth, item); values may be numeric (mineral %, grain size) or text (status,
        -- lithology remarks).
        --
        -- `set_name` versions the DELIVERY, exactly as core sets and curve sets do: a
        -- second XRD or CEC delivery lands beside the first instead of replacing it, and
        -- exactly ONE set per (well, dataset) is ACTIVE — two deliveries describe the same
        -- samples, so reading both would double every count (`aux_sets.active`).
        -- set_name is LAST on purpose: the Appender writes positionally, and databases
        -- migrated by ALTER get the column appended, so fresh and migrated schemas must
        -- agree on the order.
        CREATE TABLE IF NOT EXISTS aux_data (
            well_id    UUID NOT NULL,
            dataset    VARCHAR NOT NULL,  -- 'PETROGRAPHY' | 'XRD' | 'PERFORATION' | custom
            depth_top  FLOAT NOT NULL,
            depth_base FLOAT,             -- NULL = point sample
            item       VARCHAR NOT NULL,  -- source column (QUARTZ, STATUS, …)
            value_num  FLOAT,
            value_text VARCHAR,
            set_name   VARCHAR NOT NULL DEFAULT 'RAW'
        );
        -- Pre-set-era projects converge on the same shape (additive, no rebuild needed —
        -- aux_data has no PRIMARY KEY). DuckDB refuses ADD COLUMN with a constraint, so the
        -- added column is plain and back-filled here; on a migrated database it is nullable
        -- where a fresh one has NOT NULL, which changes nothing for readers or the Appender
        -- (position and type match, and every writer passes a value).
        -- `migrate_point_data_sets` then registers those rows in `aux_sets`.
        ALTER TABLE aux_data ADD COLUMN IF NOT EXISTS set_name VARCHAR;
        UPDATE aux_data SET set_name = 'RAW' WHERE set_name IS NULL;

        -- Registry of point-data deliveries, one row per (well, dataset, set).
        CREATE TABLE IF NOT EXISTS aux_sets (
            well_id     UUID NOT NULL,
            dataset     VARCHAR NOT NULL,
            set_name    VARCHAR NOT NULL,
            active      INTEGER NOT NULL DEFAULT 0,
            source      VARCHAR,
            imported_at TIMESTAMP NOT NULL DEFAULT now(),
            PRIMARY KEY (well_id, dataset, set_name)
        );

        -- Depth-registered PICTURES: petrographic thin sections, core photographs, SEM
        -- plates, FMI snapshots — anything a lab delivers as a raster beside the plugs.
        --
        -- Deliberately its own store rather than an `aux_data` item, for the same reason a
        -- point series is not a `CurveStyle`: an aux row carries ONE number or string, and
        -- a picture is neither. Storing megabytes in `value_text` would also put a blob in
        -- the middle of every point-data scan.
        --
        -- `depth_base IS NULL` means a POINT sample — a thin section is cut from one plug
        -- and has no thickness, so it is anchored at its depth rather than stretched over a
        -- guessed interval. A core photograph delivered with a base depth spans it for real.
        --
        -- `data` is the DISPLAY copy: a normalized JPEG (see `images.rs`), because the
        -- viewer, the SVG export and the PDF exporter all need one decodable form and a
        -- 6000x4000 camera original would bloat a field project for no visible gain at
        -- track width. `source_path` records where the delivered file came from, and
        -- `src_width`/`src_height` its true pixel size, so the original is always traceable.
        --
        -- PRIMARY KEY here costs one index entry per PICTURE, not per sample — the opposite
        -- of the `computed_curves` case (see its comment) — while a duplicated row would
        -- print the same plate twice. `image_id` is a UUID, so the key is unique by
        -- construction and re-import replaces per set.
        CREATE TABLE IF NOT EXISTS well_images (
            well_id     UUID NOT NULL,
            dataset     VARCHAR NOT NULL,   -- 'THIN SECTION' | 'CORE PHOTO' | custom
            set_name    VARCHAR NOT NULL DEFAULT 'RAW',
            image_id    UUID NOT NULL,
            depth_top   FLOAT NOT NULL,
            depth_base  FLOAT,              -- NULL = point sample (no thickness)
            name        VARCHAR NOT NULL,   -- label drawn on the track
            caption     VARCHAR,
            mime        VARCHAR NOT NULL,   -- of `data` (the display copy)
            width       INTEGER NOT NULL,   -- pixels of `data`
            height      INTEGER NOT NULL,
            src_width   INTEGER,            -- pixels of the delivered original
            src_height  INTEGER,
            source_path VARCHAR,
            printable   INTEGER NOT NULL DEFAULT 1, -- 0 = viewer only, cannot embed in a PDF
            data        BLOB NOT NULL,
            PRIMARY KEY (well_id, dataset, set_name, image_id)
        );

        -- Registry of image deliveries, one row per (well, dataset, set) — the same
        -- one-active-delivery rule as core, SCAL, surveys and point data.
        CREATE TABLE IF NOT EXISTS image_sets (
            well_id     UUID NOT NULL,
            dataset     VARCHAR NOT NULL,
            set_name    VARCHAR NOT NULL,
            active      INTEGER NOT NULL DEFAULT 0,
            source      VARCHAR,
            imported_at TIMESTAMP NOT NULL DEFAULT now(),
            PRIMARY KEY (well_id, dataset, set_name)
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
        -- …and the delivery it came in, like every other point store: a second SCAL
        -- report lands beside the first, one set per well is ACTIVE. Added by ALTER (no
        -- PK to rebuild) and back-filled; the column is LAST because the Appender writes
        -- positionally. `migrate_point_data_sets` registers pre-set-era rows.
        ALTER TABLE scal_pc ADD COLUMN IF NOT EXISTS set_name VARCHAR;
        UPDATE scal_pc SET set_name = 'RAW' WHERE set_name IS NULL;

        CREATE TABLE IF NOT EXISTS scal_sets (
            well_id     UUID NOT NULL,
            set_name    VARCHAR NOT NULL,
            active      INTEGER NOT NULL DEFAULT 0,
            source      VARCHAR,
            imported_at TIMESTAMP NOT NULL DEFAULT now(),
            PRIMARY KEY (well_id, set_name)
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
        -- `survey_name` versions the survey the same way core sets version plugs
        -- (T-IMP-12): a definitive survey can sit beside the preliminary one it replaced,
        -- and re-import never silently overwrites. Exactly one survey is ACTIVE per well;
        -- it is the one that drives TVD/TVDSS everywhere (`well_surveys.active`).
        CREATE TABLE IF NOT EXISTS well_path (
            well_id     UUID NOT NULL,
            survey_name VARCHAR NOT NULL DEFAULT 'RAW',
            md          FLOAT NOT NULL,
            inc         FLOAT NOT NULL,   -- inclination, degrees
            azi         FLOAT NOT NULL,   -- azimuth, degrees
            tvd         FLOAT,            -- computed, minimum curvature
            tvdss       FLOAT,            -- tvd - kb (or well.kb if datum omitted)
            PRIMARY KEY (well_id, survey_name, md)
        );

        CREATE TABLE IF NOT EXISTS well_surveys (
            well_id     UUID NOT NULL,
            survey_name VARCHAR NOT NULL,
            active      INTEGER NOT NULL DEFAULT 0,
            source      VARCHAR,
            datum       FLOAT,             -- KB/datum elevation the TVDSS was computed at
            imported_at TIMESTAMP NOT NULL DEFAULT now(),
            PRIMARY KEY (well_id, survey_name)
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

/// RELEASE.md §3.2 (requirement R-B): before a migration that rewrites or drops data, copy
/// the project file beside itself as `<name>.pre-<FORMAT_VERSION>-backup.duckdb`. Purely
/// additive migrations are exempt — a backup on every open would bury the one that matters.
///
/// The copy is made BY THE ENGINE (`ATTACH` + `COPY FROM DATABASE`), not by the filesystem:
/// DuckDB holds the project file with exclusive sharing on Windows, so `std::fs::copy` of an
/// open database fails with a sharing violation (os error 32) — and the engine copy is better
/// anyway: it needs no CHECKPOINT (the engine reads its own current state, WAL included) and
/// it carries schema WITH constraints, so the backup keeps the very PK being migrated away.
/// An existing backup is NEVER overwritten (it may be the only good copy of an earlier
/// format); on a name collision the copy gets a unix-timestamp suffix, the same convention
/// as the WAL recovery's `.corrupt-backup-<ts>`.
fn backup_before_destructive_migration(conn: &Connection, path: &str) -> DbResult<String> {
    let stem = path.strip_suffix(".duckdb").unwrap_or(path);
    let mut backup = format!("{stem}.pre-{FORMAT_VERSION}-backup.duckdb");
    if std::path::Path::new(&backup).exists() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        backup = format!("{stem}.pre-{FORMAT_VERSION}-backup-{ts}.duckdb");
    }
    engine_copy_to(conn, &backup)?;
    Ok(backup)
}

/// Engine copy of the CURRENT database to a fresh file at `dest` (`ATTACH` +
/// `COPY FROM DATABASE`). Two properties matter to every caller:
///
/// - it reads the engine's own live state (WAL included), so no CHECKPOINT is needed and
///   no filesystem sharing violation is possible (unlike `std::fs::copy` of an open DB);
/// - it writes ONLY live rows, so the copy is **compacted**: none of the dead space that
///   months of DELETE+append module re-runs leave in the source file (DuckDB reuses freed
///   pages internally but never shrinks the file) comes along. This is what makes it the
///   engine of both the migration backups and "Compact Project" / "Save As".
///
/// `dest` must not exist — attaching an existing file would open it as a live database
/// and merge into it instead of producing a clean copy.
pub fn engine_copy_to(conn: &Connection, dest: &str) -> DbResult<()> {
    let src: String = conn.query_row("SELECT current_database()", [], |r| r.get(0))?;
    conn.execute_batch(&format!(
        "ATTACH '{}' AS rb_copy_target;
         COPY FROM DATABASE \"{}\" TO rb_copy_target;
         DETACH rb_copy_target;",
        dest.replace('\'', "''"),
        src.replace('"', "\"\""),
    ))?;
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
///
/// This rebuild is the shipped example of a DESTRUCTIVE migration (RELEASE §3.2), so when it
/// is actually going to run, `path` is backed up first. A failed backup ABORTS the migration:
/// the un-migrated file still opens fine (the PK only makes writes slower), so refusing costs
/// nothing, while rewriting a field-scale project after the promised copy failed breaks the
/// exact guarantee R-B exists to make. `path: None` is for in-memory test databases only —
/// every real caller must pass the project-file path.
pub fn migrate_drop_computed_curves_pk(conn: &Connection, path: Option<&str>) -> DbResult<()> {
    let has_pk: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duckdb_constraints()
         WHERE table_name = 'computed_curves' AND constraint_type = 'PRIMARY KEY'",
        [],
        |r| r.get(0),
    )?;
    if has_pk == 0 {
        return Ok(());
    }
    if let Some(path) = path {
        let backup = backup_before_destructive_migration(conn, path)?;
        boot_note(format!("One-time storage upgrade (write-speed index removal): project backed up first to {backup}"));
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

/// One-time migration replacing the never-used `array_logs` stub with the real array store.
///
/// The original shape was `(well_id, depth, nmr_t2_distribution FLOAT[])`, declared in the
/// very first schema as a placeholder for a later phase. **No code path ever wrote a single
/// row to it** — `dlis.rs` skips array channels with a comment pointing here, and nothing
/// else mentions the table — so dropping it loses nothing, and this deliberately does NOT
/// take a backup: there is no data to protect and a field-scale project should not pay a
/// whole-file copy for an empty table.
///
/// Detection is by column name rather than by row count, so the migration is idempotent and
/// a database already carrying the new shape short-circuits on the first query.
pub fn migrate_array_logs_store(conn: &Connection) -> DbResult<()> {
    let old_shape: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duckdb_columns()
         WHERE table_name = 'array_logs' AND column_name = 'nmr_t2_distribution'",
        [],
        |r| r.get(0),
    )?;
    if old_shape == 0 {
        return Ok(());
    }
    conn.execute_batch(
        "DROP TABLE IF EXISTS array_logs;
         CREATE TABLE array_logs (
            well_id     UUID NOT NULL,
            set_name    VARCHAR NOT NULL DEFAULT 'RAW',
            curve_name  VARCHAR NOT NULL,
            depth       FLOAT NOT NULL,
            samples     BLOB NOT NULL,
            PRIMARY KEY (well_id, set_name, curve_name, depth)
         );",
    )?;
    Ok(())
}

/// One depth of an array log: the depth, and every value the array holds there.
#[derive(Debug, Clone)]
pub struct ArrayRow {
    pub depth: f32,
    pub samples: Vec<f32>,
}

/// One array curve present on a well, for catalogs and pickers.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ArrayCurveInfo {
    pub set_name: String,
    pub curve_name: String,
    /// Number of depths the array covers.
    pub depths: i64,
    /// Values per depth in the WIDEST row — realization counts are uniform in practice, but
    /// a ragged array (an NMR tool that changed bin count mid-run) must not be misreported.
    pub width: i64,
    pub depth_min: f32,
    pub depth_max: f32,
}

/// Encodes a value vector as the stored blob: explicit little-endian f32, 4 bytes per value.
/// Explicit rather than `bytemuck::cast_slice` (which is native-endian) so the on-disk format
/// is a stated contract, not a property of whichever machine wrote the file.
fn encode_samples(values: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len() * 4);
    for v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

/// Decodes a stored blob back to values. A trailing partial value (impossible unless the file
/// was truncated) is DROPPED rather than read as garbage.
fn decode_samples(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Replaces one array curve on one well, wholesale.
///
/// The write discipline mirrors `write_computed_curves_batch`: DELETE the (well, set, curve)
/// rows first, then insert fresh ones — a re-run replaces its own output and never unions two
/// runs' realizations into one distribution. `depths` and `samples` must be the same length;
/// a depth whose vector is EMPTY is skipped rather than stored, so "no realizations survived
/// here" reads back as an absent depth instead of a zero-width distribution.
pub fn write_array_log(
    conn: &Connection,
    well_id: &str,
    set_name: &str,
    curve_name: &str,
    depths: &[f32],
    samples: &[Vec<f32>],
) -> DbResult<usize> {
    if depths.len() != samples.len() {
        return Err(DbError::LengthMismatch(format!(
            "array log {curve_name}: {} depths against {} value vectors",
            depths.len(),
            samples.len()
        )));
    }
    conn.execute(
        "DELETE FROM array_logs WHERE well_id = ? AND set_name = ? AND upper(curve_name) = upper(?)",
        duckdb::params![well_id, set_name, curve_name],
    )?;
    let mut stmt = conn.prepare(
        "INSERT INTO array_logs (well_id, set_name, curve_name, depth, samples) VALUES (?, ?, ?, ?, ?)",
    )?;
    let mut written = 0usize;
    for (d, vals) in depths.iter().zip(samples) {
        if vals.is_empty() || !d.is_finite() {
            continue;
        }
        stmt.execute(duckdb::params![well_id, set_name, curve_name, d, encode_samples(vals)])?;
        written += 1;
    }
    Ok(written)
}

/// Reads one array curve, ordered by depth. `set_name` of `None` takes whichever set holds the
/// curve, preferring the alphabetically first — array logs are produced outputs, so a well
/// normally carries exactly one set per curve name.
pub fn read_array_log(
    conn: &Connection,
    well_id: &str,
    set_name: Option<&str>,
    curve_name: &str,
) -> DbResult<Vec<ArrayRow>> {
    let sql = match set_name {
        Some(_) => {
            "SELECT depth, samples FROM array_logs
             WHERE well_id = ? AND upper(curve_name) = upper(?) AND set_name = ?
             ORDER BY depth"
        }
        None => {
            "SELECT depth, samples FROM array_logs
             WHERE well_id = ? AND upper(curve_name) = upper(?)
               AND set_name = (SELECT min(set_name) FROM array_logs
                               WHERE well_id = ? AND upper(curve_name) = upper(?))
             ORDER BY depth"
        }
    };
    let mut stmt = conn.prepare(sql)?;
    let map = |r: &duckdb::Row| -> duckdb::Result<ArrayRow> {
        let depth: f32 = r.get(0)?;
        let bytes: Vec<u8> = r.get(1)?;
        Ok(ArrayRow { depth, samples: decode_samples(&bytes) })
    };
    let rows = match set_name {
        Some(set) => stmt.query_map(duckdb::params![well_id, curve_name, set], map)?.collect::<duckdb::Result<Vec<_>>>()?,
        None => stmt
            .query_map(duckdb::params![well_id, curve_name, well_id, curve_name], map)?
            .collect::<duckdb::Result<Vec<_>>>()?,
    };
    Ok(rows)
}

/// Every array curve on a well, for the layout dialog's picker and the object tree.
pub fn list_array_curves(conn: &Connection, well_id: &str) -> DbResult<Vec<ArrayCurveInfo>> {
    let mut stmt = conn.prepare(
        "SELECT set_name, curve_name, COUNT(*), max(octet_length(samples)) / 4, min(depth), max(depth)
         FROM array_logs WHERE well_id = ?
         GROUP BY set_name, curve_name ORDER BY set_name, curve_name",
    )?;
    let rows = stmt
        .query_map([well_id], |r| {
            Ok(ArrayCurveInfo {
                set_name: r.get(0)?,
                curve_name: r.get(1)?,
                depths: r.get(2)?,
                width: r.get(3)?,
                depth_min: r.get(4)?,
                depth_max: r.get(5)?,
            })
        })?
        .collect::<duckdb::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Deletes one array curve from a well (the Data Sets dialog's remove action).
pub fn delete_array_log(conn: &Connection, well_id: &str, set_name: &str, curve_name: &str) -> DbResult<usize> {
    let n = conn.execute(
        "DELETE FROM array_logs WHERE well_id = ? AND set_name = ? AND upper(curve_name) = upper(?)",
        duckdb::params![well_id, set_name, curve_name],
    )?;
    Ok(n)
}

/// One-time migration that brings every DELIVERY-shaped store onto the set model: a
/// `set_name` on `core_data` and `aux_data`, a `survey_name` on `well_path`
/// (T-IMP-08 / T-IMP-12), so a well can hold several core deliveries, several surveys and
/// several point-data deliveries (XRD, CEC, oil show …) instead of each import replacing
/// the last.
///
/// `core_data` and `well_path` carry a PRIMARY KEY that must gain a column, which DuckDB
/// cannot alter in place, so those two are rebuilt; `aux_data` has no PK, so `create_schema`
/// simply ALTERs the column in and this only registers the rows. Existing rows become the
/// set/survey named **RAW**, registered ACTIVE — a migrated project reads exactly the
/// numbers it read before. Idempotent: the column list is consulted first, and the aux
/// registration only fills gaps, so this is a no-op on freshly created databases and on
/// every launch after the first.
///
/// Destructive (a table rebuild), so it follows the RELEASE §3.2 rule: when it is actually
/// going to run, `path` is backed up first and a failed backup ABORTS the migration.
/// `path: None` is for in-memory test databases only.
pub fn migrate_point_data_sets(conn: &Connection, path: Option<&str>) -> DbResult<()> {
    let has_set: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duckdb_columns()
         WHERE table_name = 'core_data' AND column_name = 'set_name'",
        [],
        |r| r.get(0),
    )?;
    let has_survey: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duckdb_columns()
         WHERE table_name = 'well_path' AND column_name = 'survey_name'",
        [],
        |r| r.get(0),
    )?;
    // Point-data rows predating the registry: adopt them as RAW/active so readers (which
    // filter on the active set) can still see them. Cheap, gap-filling and idempotent, so
    // it runs before the early return — a project may have been rebuilt already while a
    // later aux import path was still writing unregistered rows.
    conn.execute_batch(
        "UPDATE aux_data SET set_name = 'RAW' WHERE set_name IS NULL;
         INSERT INTO aux_sets (well_id, dataset, set_name, active)
         SELECT DISTINCT a.well_id, a.dataset, a.set_name, 1 FROM aux_data a
         WHERE NOT EXISTS (SELECT 1 FROM aux_sets s
                           WHERE s.well_id = a.well_id AND s.dataset = a.dataset);
         UPDATE scal_pc SET set_name = 'RAW' WHERE set_name IS NULL;
         INSERT INTO scal_sets (well_id, set_name, active)
         SELECT DISTINCT p.well_id, p.set_name, 1 FROM scal_pc p
         WHERE NOT EXISTS (SELECT 1 FROM scal_sets s WHERE s.well_id = p.well_id);",
    )?;

    if has_set > 0 && has_survey > 0 {
        return Ok(());
    }
    if let Some(path) = path {
        let backup = backup_before_destructive_migration(conn, path)?;
        boot_note(format!("One-time storage upgrade (delivery sets for core/surveys): project backed up first to {backup}"));
    }

    if has_set == 0 {
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE core_data_new (
                 well_id     UUID NOT NULL,
                 set_name    VARCHAR NOT NULL DEFAULT 'RAW',
                 depth       FLOAT NOT NULL,
                 cpor        FLOAT,
                 cperm       FLOAT,
                 cgd         FLOAT,
                 csw         FLOAT,
                 PRIMARY KEY (well_id, set_name, depth)
             );
             INSERT INTO core_data_new
                 SELECT well_id, 'RAW', depth, cpor, cperm, cgd, csw FROM core_data;
             DROP TABLE core_data;
             ALTER TABLE core_data_new RENAME TO core_data;
             INSERT INTO core_sets (well_id, set_name, active, source)
                 SELECT DISTINCT well_id, 'RAW', 1, NULL FROM core_data;
             COMMIT;",
        )?;
    }
    if has_survey == 0 {
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE well_path_new (
                 well_id     UUID NOT NULL,
                 survey_name VARCHAR NOT NULL DEFAULT 'RAW',
                 md          FLOAT NOT NULL,
                 inc         FLOAT NOT NULL,
                 azi         FLOAT NOT NULL,
                 tvd         FLOAT,
                 tvdss       FLOAT,
                 PRIMARY KEY (well_id, survey_name, md)
             );
             INSERT INTO well_path_new
                 SELECT well_id, 'RAW', md, inc, azi, tvd, tvdss FROM well_path;
             DROP TABLE well_path;
             ALTER TABLE well_path_new RENAME TO well_path;
             INSERT INTO well_surveys (well_id, survey_name, active, source)
                 SELECT DISTINCT well_id, 'RAW', 1, NULL FROM well_path;
             COMMIT;",
        )?;
    }
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

/// SQL fragment naming a well's ACTIVE core set: the flagged one, else the most recently
/// imported, else 'RAW'. Every core reader filters on this — a missed filter would union
/// two deliveries of the same plugs and silently double the φ–k cloud. `?1` is the well id
/// (bound once; the placeholder is reused, as `list_aux_data` already does with `?2`).
const ACTIVE_CORE_SET: &str = "COALESCE((SELECT set_name FROM core_sets WHERE well_id = ?1
                                         ORDER BY active DESC, imported_at DESC LIMIT 1), 'RAW')";

/// Same, for deviation surveys.
const ACTIVE_SURVEY: &str = "COALESCE((SELECT survey_name FROM well_surveys WHERE well_id = ?1
                                       ORDER BY active DESC, imported_at DESC LIMIT 1), 'RAW')";

/// One core delivery of one well, as the set manager shows it.
#[derive(Debug, Clone, Serialize)]
pub struct CoreSetInfo {
    pub set_name: String,
    pub rows: i64,
    pub active: bool,
    pub source: Option<String>,
    pub imported_at: Option<String>,
}

/// A well's core sets, active first then newest, with plug counts.
pub fn list_core_sets(conn: &Connection, well_id: &str) -> DbResult<Vec<CoreSetInfo>> {
    let mut stmt = conn.prepare(
        "SELECT s.set_name, s.active, s.source, CAST(s.imported_at AS VARCHAR),
                (SELECT COUNT(*) FROM core_data d WHERE d.well_id = s.well_id AND d.set_name = s.set_name)
         FROM core_sets s WHERE s.well_id = ?1
         ORDER BY s.active DESC, s.imported_at DESC, s.set_name",
    )?;
    let rows = stmt.query_map(params![well_id], |r| {
        Ok(CoreSetInfo {
            set_name: r.get(0)?,
            active: r.get::<_, i32>(1)? != 0,
            source: r.get(2)?,
            imported_at: r.get(3)?,
            rows: r.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// The name a new core delivery will actually be stored under: `desired` when the well
/// does not have it yet, else `desired_1`, `_2`, … — an import NEVER overwrites an earlier
/// delivery (identical rule to `ingest::resolve_set_name` for curves).
pub fn resolve_core_set_name(conn: &Connection, well_id: &str, desired: &str) -> DbResult<String> {
    let base = {
        let t = desired.trim().to_uppercase().replace(' ', "_");
        if t.is_empty() { "CORE".to_string() } else { t }
    };
    let taken = |name: &str| -> DbResult<bool> {
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM core_sets WHERE well_id = ?1 AND upper(set_name) = ?2",
            params![well_id, name],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    };
    if !taken(&base)? {
        return Ok(base);
    }
    for i in 1..1000 {
        let cand = format!("{base}_{i}");
        if !taken(&cand)? {
            return Ok(cand);
        }
    }
    Err(DbError::LengthMismatch(format!("too many core sets named {base}")))
}

/// Makes one core set the well's live one (0 or 1 active per well).
pub fn set_active_core_set(conn: &Connection, well_id: &str, set_name: &str) -> DbResult<()> {
    with_txn(conn, |conn| {
        conn.execute("UPDATE core_sets SET active = 0 WHERE well_id = ?1", params![well_id])?;
        let n = conn.execute(
            "UPDATE core_sets SET active = 1 WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, set_name],
        )?;
        if n == 0 {
            return Err(DbError::LengthMismatch(format!("no core set '{set_name}' on this well")));
        }
        Ok(())
    })
}

/// Deletes one core delivery outright. If it was the active one, the newest survivor takes
/// over — a well is never left with plugs no reader can see.
pub fn delete_core_set(conn: &Connection, well_id: &str, set_name: &str) -> DbResult<usize> {
    let removed = with_txn(conn, |conn| -> DbResult<usize> {
        let n = conn.execute(
            "DELETE FROM core_data WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, set_name],
        )?;
        conn.execute(
            "DELETE FROM core_sets WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, set_name],
        )?;
        Ok(n)
    })?;
    let has_active: i64 = conn.query_row(
        "SELECT COUNT(*) FROM core_sets WHERE well_id = ?1 AND active = 1",
        params![well_id],
        |r| r.get(0),
    )?;
    if has_active == 0 {
        let next: Option<String> = conn
            .query_row(
                "SELECT set_name FROM core_sets WHERE well_id = ?1 ORDER BY imported_at DESC LIMIT 1",
                params![well_id],
                |r| r.get(0),
            )
            .ok();
        if let Some(next) = next {
            set_active_core_set(conn, well_id, &next)?;
        }
    }
    Ok(removed)
}

/// Bulk-inserts one core DELIVERY for a well under `set_name`, replacing only that set's
/// rows (re-importing the same set name overwrites it; a NEW name leaves earlier deliveries
/// untouched — callers pass a name from `resolve_core_set_name`). The stored set becomes the
/// well's active one: it is what the user just imported and expects to see.
pub fn insert_core_data(
    conn: &Connection,
    well_id: &str,
    set_name: &str,
    source: Option<&str>,
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
        conn.execute(
            "DELETE FROM core_data WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, set_name],
        )?;
        let mut appender: Appender = conn.appender("core_data")?;
        for i in 0..n {
            appender.append_row(params![well_id, set_name, depths[i], cpor[i], cperm[i], cgd[i], csw[i]])?;
        }
        appender.flush()?;
        conn.execute(
            "DELETE FROM core_sets WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, set_name],
        )?;
        conn.execute("UPDATE core_sets SET active = 0 WHERE well_id = ?1", params![well_id])?;
        conn.execute(
            "INSERT INTO core_sets (well_id, set_name, active, source) VALUES (?1, ?2, 1, ?3)",
            params![well_id, set_name, source],
        )?;
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

/// SQL fragment naming the ACTIVE set of the point dataset in the row being tested — the
/// aux twin of `ACTIVE_CORE_SET`, correlated on `a.dataset` so one query can span every
/// dataset (XRD, CEC, oil show, core extras …) and still see one delivery of each. Every
/// aux reader uses it; a reader that forgets would union two deliveries of the same samples
/// and double every count silently. Requires the aux_data table to be aliased `a`.
const ACTIVE_AUX_SET: &str = "COALESCE((SELECT s.set_name FROM aux_sets s
                                        WHERE s.well_id = a.well_id AND s.dataset = a.dataset
                                        ORDER BY s.active DESC, s.imported_at DESC LIMIT 1), 'RAW')";

/// One point-data delivery, as the set manager and the Wells tree show it.
#[derive(Debug, Clone, Serialize)]
pub struct AuxSetInfo {
    pub dataset: String,
    pub set_name: String,
    pub rows: i64,
    pub active: bool,
    pub source: Option<String>,
    pub imported_at: Option<String>,
}

/// Every point-data delivery of a well, grouped by dataset (active first, then newest).
pub fn list_aux_sets(conn: &Connection, well_id: &str) -> DbResult<Vec<AuxSetInfo>> {
    let mut stmt = conn.prepare(
        "SELECT s.dataset, s.set_name, s.active, s.source, CAST(s.imported_at AS VARCHAR),
                (SELECT COUNT(*) FROM aux_data d
                 WHERE d.well_id = s.well_id AND d.dataset = s.dataset AND d.set_name = s.set_name)
         FROM aux_sets s WHERE s.well_id = ?1
         ORDER BY s.dataset, s.active DESC, s.imported_at DESC, s.set_name",
    )?;
    let rows = stmt.query_map(params![well_id], |r| {
        Ok(AuxSetInfo {
            dataset: r.get(0)?,
            set_name: r.get(1)?,
            active: r.get::<_, i32>(2)? != 0,
            source: r.get(3)?,
            imported_at: r.get(4)?,
            rows: r.get(5)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// The name a new point-data delivery will be stored under within its dataset — `desired`,
/// else `desired_1`, `_2`, …; an import never overwrites an earlier delivery.
pub fn resolve_aux_set_name(conn: &Connection, well_id: &str, dataset: &str, desired: &str) -> DbResult<String> {
    let base = {
        let t = desired.trim().to_uppercase().replace(' ', "_");
        if t.is_empty() { "RAW".to_string() } else { t }
    };
    let taken = |name: &str| -> DbResult<bool> {
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM aux_sets WHERE well_id = ?1 AND dataset = ?2 AND upper(set_name) = ?3",
            params![well_id, dataset, name],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    };
    if !taken(&base)? {
        return Ok(base);
    }
    for i in 1..1000 {
        let cand = format!("{base}_{i}");
        if !taken(&cand)? {
            return Ok(cand);
        }
    }
    Err(DbError::LengthMismatch(format!("too many {dataset} sets named {base}")))
}

/// Makes one delivery the live one for its dataset (other datasets are untouched).
pub fn set_active_aux_set(conn: &Connection, well_id: &str, dataset: &str, set_name: &str) -> DbResult<()> {
    with_txn(conn, |conn| {
        conn.execute(
            "UPDATE aux_sets SET active = 0 WHERE well_id = ?1 AND dataset = ?2",
            params![well_id, dataset],
        )?;
        let n = conn.execute(
            "UPDATE aux_sets SET active = 1 WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
            params![well_id, dataset, set_name],
        )?;
        if n == 0 {
            return Err(DbError::LengthMismatch(format!("no {dataset} set '{set_name}' on this well")));
        }
        Ok(())
    })
}

/// Deletes one point-data delivery; the newest survivor of that dataset takes over.
pub fn delete_aux_set(conn: &Connection, well_id: &str, dataset: &str, set_name: &str) -> DbResult<usize> {
    let removed = with_txn(conn, |conn| -> DbResult<usize> {
        let n = conn.execute(
            "DELETE FROM aux_data WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
            params![well_id, dataset, set_name],
        )?;
        conn.execute(
            "DELETE FROM aux_sets WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
            params![well_id, dataset, set_name],
        )?;
        Ok(n)
    })?;
    let has_active: i64 = conn.query_row(
        "SELECT COUNT(*) FROM aux_sets WHERE well_id = ?1 AND dataset = ?2 AND active = 1",
        params![well_id, dataset],
        |r| r.get(0),
    )?;
    if has_active == 0 {
        let next: Option<String> = conn
            .query_row(
                "SELECT set_name FROM aux_sets WHERE well_id = ?1 AND dataset = ?2
                 ORDER BY imported_at DESC LIMIT 1",
                params![well_id, dataset],
                |r| r.get(0),
            )
            .ok();
        if let Some(next) = next {
            set_active_aux_set(conn, well_id, dataset, &next)?;
        }
    }
    Ok(removed)
}

/// Stores one point-data DELIVERY (petrography / XRD / CEC / oil show / perforation / core
/// extras …) under `set_name`, replacing only that set's rows and making it the live one for
/// its dataset. Earlier deliveries of the same dataset are untouched — callers pass a name
/// from `resolve_aux_set_name`.
pub fn insert_aux_data(
    conn: &Connection,
    well_id: &str,
    dataset: &str,
    set_name: &str,
    source: Option<&str>,
    rows: &[AuxRow],
) -> DbResult<()> {
    with_txn(conn, |conn| {
        conn.execute(
            "DELETE FROM aux_data WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
            params![well_id, dataset, set_name],
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
                r.value_text,
                set_name
            ])?;
        }
        appender.flush()?;
        conn.execute(
            "DELETE FROM aux_sets WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
            params![well_id, dataset, set_name],
        )?;
        conn.execute(
            "UPDATE aux_sets SET active = 0 WHERE well_id = ?1 AND dataset = ?2",
            params![well_id, dataset],
        )?;
        conn.execute(
            "INSERT INTO aux_sets (well_id, dataset, set_name, active, source) VALUES (?1, ?2, ?3, 1, ?4)",
            params![well_id, dataset, set_name, source],
        )?;
        Ok(())
    })
}

/// One well's auxiliary rows from the ACTIVE set of each dataset, ordered by depth then item.
pub fn list_aux_data(conn: &Connection, well_id: &str, dataset: Option<&str>) -> DbResult<Vec<AuxRow>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT a.dataset, a.depth_top, a.depth_base, a.item, a.value_num, a.value_text
         FROM aux_data a
         WHERE a.well_id = ?1 AND (?2 IS NULL OR a.dataset = ?2) AND a.set_name = {ACTIVE_AUX_SET}
         ORDER BY a.dataset, a.depth_top, a.item"
    ))?;
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

/// Which auxiliary datasets a well has, with the ACTIVE delivery's row counts (for
/// panels/dialogs) — never the sum across deliveries.
pub fn list_aux_datasets(conn: &Connection, well_id: &str) -> DbResult<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT a.dataset, COUNT(*) FROM aux_data a
         WHERE a.well_id = ?1 AND a.set_name = {ACTIVE_AUX_SET}
         GROUP BY a.dataset ORDER BY a.dataset"
    ))?;
    let rows = stmt.query_map(params![well_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Depth-registered images (thin sections, core photographs) — see `well_images`
// ---------------------------------------------------------------------------

/// One picture's METADATA. Deliberately without the pixels: a catalog listing of a well
/// that carries 300 core photographs must cost kilobytes, not a gigabyte, so every listing
/// path uses this and the bytes are fetched one image at a time by `get_well_image`.
#[derive(Debug, Clone, Serialize)]
pub struct ImageInfo {
    pub image_id: String,
    pub dataset: String,
    pub set_name: String,
    pub depth_top: f32,
    pub depth_base: Option<f32>,
    pub name: String,
    pub caption: Option<String>,
    pub mime: String,
    pub width: i32,
    pub height: i32,
    pub src_width: Option<i32>,
    pub src_height: Option<i32>,
    pub source_path: Option<String>,
    /// False = the viewer can show it but the PDF exporter cannot embed it (see `images.rs`).
    pub printable: bool,
    /// Stored size of the display copy, bytes.
    pub bytes: i64,
}

/// One picture on its way INTO the store (the import commit path).
#[derive(Debug, Clone)]
pub struct NewImage {
    pub depth_top: f32,
    pub depth_base: Option<f32>,
    pub name: String,
    pub caption: Option<String>,
    pub mime: String,
    pub width: i32,
    pub height: i32,
    pub src_width: Option<i32>,
    pub src_height: Option<i32>,
    pub source_path: Option<String>,
    pub printable: bool,
    pub data: Vec<u8>,
}

/// SQL fragment naming the ACTIVE image delivery of the dataset in the row being tested —
/// the image twin of `ACTIVE_AUX_SET`, correlated on `i.dataset` so one query spans every
/// dataset (thin sections, core photos, SEM …) and still sees one delivery of each. A
/// reader that forgets it would show the same plate twice from two deliveries of the same
/// core. Requires the `well_images` table to be aliased `i`.
const ACTIVE_IMAGE_SET: &str = "COALESCE((SELECT s.set_name FROM image_sets s
                                          WHERE s.well_id = i.well_id AND s.dataset = i.dataset
                                          ORDER BY s.active DESC, s.imported_at DESC LIMIT 1), 'RAW')";

/// One image delivery of a well, as the set manager and the Wells tree show it.
#[derive(Debug, Clone, Serialize)]
pub struct ImageSetInfo {
    pub dataset: String,
    pub set_name: String,
    pub images: i64,
    pub active: bool,
    pub source: Option<String>,
    pub imported_at: Option<String>,
    /// Total stored bytes of the delivery — the one store where a user genuinely needs to
    /// see the cost before deciding what to keep.
    pub bytes: i64,
}

/// Every image delivery of a well, grouped by dataset (active first, then newest).
pub fn list_image_sets(conn: &Connection, well_id: &str) -> DbResult<Vec<ImageSetInfo>> {
    let mut stmt = conn.prepare(
        "SELECT s.dataset, s.set_name, s.active, s.source, CAST(s.imported_at AS VARCHAR),
                (SELECT COUNT(*) FROM well_images d
                 WHERE d.well_id = s.well_id AND d.dataset = s.dataset AND d.set_name = s.set_name),
                (SELECT COALESCE(SUM(octet_length(d.data)), 0) FROM well_images d
                 WHERE d.well_id = s.well_id AND d.dataset = s.dataset AND d.set_name = s.set_name)
         FROM image_sets s WHERE s.well_id = ?1
         ORDER BY s.dataset, s.active DESC, s.imported_at DESC, s.set_name",
    )?;
    let rows = stmt.query_map(params![well_id], |r| {
        Ok(ImageSetInfo {
            dataset: r.get(0)?,
            set_name: r.get(1)?,
            active: r.get::<_, i32>(2)? != 0,
            source: r.get(3)?,
            imported_at: r.get(4)?,
            images: r.get(5)?,
            bytes: r.get(6)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// The name a new image delivery will be stored under within its dataset — `desired`, else
/// `desired_1`, `_2`, …; an import never overwrites an earlier delivery.
pub fn resolve_image_set_name(conn: &Connection, well_id: &str, dataset: &str, desired: &str) -> DbResult<String> {
    let base = {
        let t = desired.trim().to_uppercase().replace(' ', "_");
        if t.is_empty() { "RAW".to_string() } else { t }
    };
    let taken = |name: &str| -> DbResult<bool> {
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM image_sets WHERE well_id = ?1 AND dataset = ?2 AND upper(set_name) = ?3",
            params![well_id, dataset, name],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    };
    if !taken(&base)? {
        return Ok(base);
    }
    for i in 1..1000 {
        let cand = format!("{base}_{i}");
        if !taken(&cand)? {
            return Ok(cand);
        }
    }
    Err(DbError::LengthMismatch(format!("too many {dataset} image sets named {base}")))
}

/// Makes one image delivery the live one for its dataset (other datasets are untouched).
pub fn set_active_image_set(conn: &Connection, well_id: &str, dataset: &str, set_name: &str) -> DbResult<()> {
    with_txn(conn, |conn| {
        conn.execute(
            "UPDATE image_sets SET active = 0 WHERE well_id = ?1 AND dataset = ?2",
            params![well_id, dataset],
        )?;
        let n = conn.execute(
            "UPDATE image_sets SET active = 1 WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
            params![well_id, dataset, set_name],
        )?;
        if n == 0 {
            return Err(DbError::LengthMismatch(format!("no {dataset} image set '{set_name}' on this well")));
        }
        Ok(())
    })
}

/// Deletes one image delivery; the newest survivor of that dataset takes over.
pub fn delete_image_set(conn: &Connection, well_id: &str, dataset: &str, set_name: &str) -> DbResult<usize> {
    let removed = with_txn(conn, |conn| -> DbResult<usize> {
        let n = conn.execute(
            "DELETE FROM well_images WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
            params![well_id, dataset, set_name],
        )?;
        conn.execute(
            "DELETE FROM image_sets WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
            params![well_id, dataset, set_name],
        )?;
        Ok(n)
    })?;
    let has_active: i64 = conn.query_row(
        "SELECT COUNT(*) FROM image_sets WHERE well_id = ?1 AND dataset = ?2 AND active = 1",
        params![well_id, dataset],
        |r| r.get(0),
    )?;
    if has_active == 0 {
        let next: Option<String> = conn
            .query_row(
                "SELECT set_name FROM image_sets WHERE well_id = ?1 AND dataset = ?2
                 ORDER BY imported_at DESC LIMIT 1",
                params![well_id, dataset],
                |r| r.get(0),
            )
            .ok();
        if let Some(next) = next {
            set_active_image_set(conn, well_id, dataset, &next)?;
        }
    }
    Ok(removed)
}

/// Stores one image DELIVERY under `set_name`, replacing only that set's rows and making it
/// the live one for its dataset. Earlier deliveries are untouched — callers pass a name from
/// `resolve_image_set_name`.
///
/// Plain prepared INSERTs rather than an Appender: a delivery is tens of rows, not millions,
/// and each row carries a multi-megabyte blob, so the per-row overhead the Appender saves is
/// noise next to the bytes themselves.
pub fn insert_well_images(
    conn: &Connection,
    well_id: &str,
    dataset: &str,
    set_name: &str,
    source: Option<&str>,
    images: &[NewImage],
) -> DbResult<usize> {
    with_txn(conn, |conn| {
        conn.execute(
            "DELETE FROM well_images WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
            params![well_id, dataset, set_name],
        )?;
        let mut stmt = conn.prepare(
            "INSERT INTO well_images (well_id, dataset, set_name, image_id, depth_top, depth_base,
                                      name, caption, mime, width, height, src_width, src_height,
                                      source_path, printable, data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        )?;
        let mut n = 0usize;
        for img in images {
            let id = uuid::Uuid::new_v4().to_string();
            stmt.execute(params![
                well_id,
                dataset,
                set_name,
                id,
                img.depth_top,
                img.depth_base,
                img.name,
                img.caption,
                img.mime,
                img.width,
                img.height,
                img.src_width,
                img.src_height,
                img.source_path,
                if img.printable { 1i32 } else { 0i32 },
                img.data,
            ])?;
            n += 1;
        }
        drop(stmt);
        conn.execute(
            "DELETE FROM image_sets WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
            params![well_id, dataset, set_name],
        )?;
        conn.execute(
            "UPDATE image_sets SET active = 0 WHERE well_id = ?1 AND dataset = ?2",
            params![well_id, dataset],
        )?;
        conn.execute(
            "INSERT INTO image_sets (well_id, dataset, set_name, active, source) VALUES (?1, ?2, ?3, 1, ?4)",
            params![well_id, dataset, set_name, source],
        )?;
        Ok(n)
    })
}

/// Metadata for a well's pictures, from the ACTIVE delivery of each dataset, ordered by
/// depth. `dataset = None` spans every dataset. NEVER selects `data` — see [`ImageInfo`].
pub fn list_well_images(conn: &Connection, well_id: &str, dataset: Option<&str>) -> DbResult<Vec<ImageInfo>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT CAST(i.image_id AS VARCHAR), i.dataset, i.set_name, i.depth_top, i.depth_base,
                i.name, i.caption, i.mime, i.width, i.height, i.src_width, i.src_height,
                i.source_path, i.printable, octet_length(i.data)
         FROM well_images i
         WHERE i.well_id = ?1 AND (?2 IS NULL OR i.dataset = ?2) AND i.set_name = {ACTIVE_IMAGE_SET}
         ORDER BY i.dataset, i.depth_top, i.name"
    ))?;
    let rows = stmt.query_map(params![well_id, dataset], |r| {
        Ok(ImageInfo {
            image_id: r.get(0)?,
            dataset: r.get(1)?,
            set_name: r.get(2)?,
            depth_top: r.get(3)?,
            depth_base: r.get(4)?,
            name: r.get(5)?,
            caption: r.get(6)?,
            mime: r.get(7)?,
            width: r.get(8)?,
            height: r.get(9)?,
            src_width: r.get(10)?,
            src_height: r.get(11)?,
            source_path: r.get(12)?,
            printable: r.get::<_, i32>(13)? != 0,
            bytes: r.get(14)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Which image datasets a well has, with the ACTIVE delivery's counts — never the sum
/// across deliveries.
pub fn list_image_datasets(conn: &Connection, well_id: &str) -> DbResult<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT i.dataset, COUNT(*) FROM well_images i
         WHERE i.well_id = ?1 AND i.set_name = {ACTIVE_IMAGE_SET}
         GROUP BY i.dataset ORDER BY i.dataset"
    ))?;
    let rows = stmt.query_map(params![well_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// The pixels of ONE picture, with its mime type. The only path that reads a blob.
pub fn get_well_image(conn: &Connection, image_id: &str) -> DbResult<(String, Vec<u8>)> {
    let row = conn.query_row(
        "SELECT mime, data FROM well_images WHERE image_id = ?1",
        params![image_id],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, Vec<u8>>(1)?)),
    )?;
    Ok(row)
}

/// Every printable picture of one dataset in a depth window, pixels included — the composite
/// exporter's read path. Non-printable rows come back too (with their bytes) so the exporter
/// can draw a labelled placeholder rather than silently dropping a plate.
pub fn read_images_for_print(
    conn: &Connection,
    well_id: &str,
    dataset: &str,
    depth_top: f32,
    depth_bottom: f32,
) -> DbResult<Vec<(ImageInfo, Vec<u8>)>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT CAST(i.image_id AS VARCHAR), i.dataset, i.set_name, i.depth_top, i.depth_base,
                i.name, i.caption, i.mime, i.width, i.height, i.src_width, i.src_height,
                i.source_path, i.printable, octet_length(i.data), i.data
         FROM well_images i
         WHERE i.well_id = ?1 AND i.dataset = ?2 AND i.set_name = {ACTIVE_IMAGE_SET}
           AND COALESCE(i.depth_base, i.depth_top) >= ?3 AND i.depth_top <= ?4
         ORDER BY i.depth_top, i.name"
    ))?;
    let rows = stmt.query_map(params![well_id, dataset, depth_top, depth_bottom], |r| {
        Ok((
            ImageInfo {
                image_id: r.get(0)?,
                dataset: r.get(1)?,
                set_name: r.get(2)?,
                depth_top: r.get(3)?,
                depth_base: r.get(4)?,
                name: r.get(5)?,
                caption: r.get(6)?,
                mime: r.get(7)?,
                width: r.get(8)?,
                height: r.get(9)?,
                src_width: r.get(10)?,
                src_height: r.get(11)?,
                source_path: r.get(12)?,
                printable: r.get::<_, i32>(13)? != 0,
                bytes: r.get(14)?,
            },
            r.get::<_, Vec<u8>>(15)?,
        ))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Deletes one picture (the set manager's per-image remove).
pub fn delete_well_image(conn: &Connection, image_id: &str) -> DbResult<usize> {
    Ok(conn.execute("DELETE FROM well_images WHERE image_id = ?1", params![image_id])?)
}

/// Edits one picture's depth registration / labels — core-to-log alignment for pictures,
/// the twin of `update_core_sample`. `depth_base = None` makes it a point sample again.
pub fn update_well_image(
    conn: &Connection,
    image_id: &str,
    depth_top: f32,
    depth_base: Option<f32>,
    name: &str,
    caption: Option<&str>,
) -> DbResult<usize> {
    Ok(conn.execute(
        "UPDATE well_images SET depth_top = ?2, depth_base = ?3, name = ?4, caption = ?5
         WHERE image_id = ?1",
        params![image_id, depth_top, depth_base, name, caption],
    )?)
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

/// SQL fragment naming a well's ACTIVE SCAL delivery — the last of the four point stores
/// to follow the set model. Two Pc reports describe the same plugs, so reading both would
/// double every Pc curve and skew a Leverett-J or Thomeer fit.
const ACTIVE_SCAL_SET: &str = "COALESCE((SELECT set_name FROM scal_sets WHERE well_id = ?1
                                         ORDER BY active DESC, imported_at DESC LIMIT 1), 'RAW')";

/// One SCAL delivery of one well, as the set manager and the Wells tree show it.
#[derive(Debug, Clone, Serialize)]
pub struct ScalSetInfo {
    pub set_name: String,
    pub rows: i64,
    pub active: bool,
    pub source: Option<String>,
    pub imported_at: Option<String>,
}

/// A well's SCAL deliveries, active first then newest, with point counts.
pub fn list_scal_sets(conn: &Connection, well_id: &str) -> DbResult<Vec<ScalSetInfo>> {
    let mut stmt = conn.prepare(
        "SELECT s.set_name, s.active, s.source, CAST(s.imported_at AS VARCHAR),
                (SELECT COUNT(*) FROM scal_pc d WHERE d.well_id = s.well_id AND d.set_name = s.set_name)
         FROM scal_sets s WHERE s.well_id = ?1
         ORDER BY s.active DESC, s.imported_at DESC, s.set_name",
    )?;
    let rows = stmt.query_map(params![well_id], |r| {
        Ok(ScalSetInfo {
            set_name: r.get(0)?,
            active: r.get::<_, i32>(1)? != 0,
            source: r.get(2)?,
            imported_at: r.get(3)?,
            rows: r.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// The name a new SCAL delivery will be stored under — `desired`, else `desired_1`, … .
pub fn resolve_scal_set_name(conn: &Connection, well_id: &str, desired: &str) -> DbResult<String> {
    let base = {
        let t = desired.trim().to_uppercase().replace(' ', "_");
        if t.is_empty() { "SCAL".to_string() } else { t }
    };
    let taken = |name: &str| -> DbResult<bool> {
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM scal_sets WHERE well_id = ?1 AND upper(set_name) = ?2",
            params![well_id, name],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    };
    if !taken(&base)? {
        return Ok(base);
    }
    for i in 1..1000 {
        let cand = format!("{base}_{i}");
        if !taken(&cand)? {
            return Ok(cand);
        }
    }
    Err(DbError::LengthMismatch(format!("too many SCAL sets named {base}")))
}

/// Makes one SCAL delivery the well's live one.
pub fn set_active_scal_set(conn: &Connection, well_id: &str, set_name: &str) -> DbResult<()> {
    with_txn(conn, |conn| {
        conn.execute("UPDATE scal_sets SET active = 0 WHERE well_id = ?1", params![well_id])?;
        let n = conn.execute(
            "UPDATE scal_sets SET active = 1 WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, set_name],
        )?;
        if n == 0 {
            return Err(DbError::LengthMismatch(format!("no SCAL set '{set_name}' on this well")));
        }
        Ok(())
    })
}

/// Deletes one SCAL delivery; the newest survivor takes over.
pub fn delete_scal_set(conn: &Connection, well_id: &str, set_name: &str) -> DbResult<usize> {
    let removed = with_txn(conn, |conn| -> DbResult<usize> {
        let n = conn.execute(
            "DELETE FROM scal_pc WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, set_name],
        )?;
        conn.execute(
            "DELETE FROM scal_sets WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, set_name],
        )?;
        Ok(n)
    })?;
    let has_active: i64 = conn.query_row(
        "SELECT COUNT(*) FROM scal_sets WHERE well_id = ?1 AND active = 1",
        params![well_id],
        |r| r.get(0),
    )?;
    if has_active == 0 {
        let next: Option<String> = conn
            .query_row(
                "SELECT set_name FROM scal_sets WHERE well_id = ?1 ORDER BY imported_at DESC LIMIT 1",
                params![well_id],
                |r| r.get(0),
            )
            .ok();
        if let Some(next) = next {
            set_active_scal_set(conn, well_id, &next)?;
        }
    }
    Ok(removed)
}

/// Bulk-inserts one SCAL DELIVERY for a well under `set_name`, replacing only that set's
/// points and making it the live one. Earlier reports are untouched.
pub fn insert_scal_pc(
    conn: &Connection,
    well_id: &str,
    set_name: &str,
    source: Option<&str>,
    rows: &[ScalPcRow],
) -> DbResult<()> {
    with_txn(conn, |conn| {
        conn.execute(
            "DELETE FROM scal_pc WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, set_name],
        )?;
        let mut appender: Appender = conn.appender("scal_pc")?;
        for r in rows {
            appender.append_row(params![
                well_id, r.sample_no, r.depth, r.perm, r.poro, r.pc, r.sw, r.system, r.ift, set_name
            ])?;
        }
        appender.flush()?;
        conn.execute(
            "DELETE FROM scal_sets WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, set_name],
        )?;
        conn.execute("UPDATE scal_sets SET active = 0 WHERE well_id = ?1", params![well_id])?;
        conn.execute(
            "INSERT INTO scal_sets (well_id, set_name, active, source) VALUES (?1, ?2, 1, ?3)",
            params![well_id, set_name, source],
        )?;
        Ok(())
    })
}

/// One well's capillary-pressure points, from the ACTIVE SCAL delivery.
pub fn get_scal_pc(conn: &Connection, well_id: &str) -> DbResult<Vec<ScalPcRow>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT sample_no, depth, perm, poro, pc, sw, system, ift FROM scal_pc
         WHERE well_id = ?1 AND set_name = {ACTIVE_SCAL_SET} ORDER BY sample_no NULLS FIRST, pc"
    ))?;
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

/// One well's core plugs (depth ascending) with porosity/permeability only, from the
/// ACTIVE core set. NULL φ or k become NaN so the caller can skip them.
pub fn get_core_plugs(conn: &Connection, well_id: &str) -> DbResult<Vec<CorePlugRow>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT depth, cpor, cperm FROM core_data
         WHERE well_id = ?1 AND set_name = {ACTIVE_CORE_SET} ORDER BY depth"
    ))?;
    let rows = stmt.query_map(params![well_id], |row| {
        Ok(CorePlugRow {
            depth: row.get(0)?,
            cpor: row.get::<_, Option<f32>>(1)?.unwrap_or(f32::NAN),
            cperm: row.get::<_, Option<f32>>(2)?.unwrap_or(f32::NAN),
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// One core plug's porosity + grain density (NaN when the column was blank) — the SandiMin
/// core-calibration inputs. Kept separate from `CorePlugRow`, which carries permeability for the
/// HFU/FZI panes and never reads the grain-density column.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CoreQcRow {
    pub depth: f32,
    pub cpor: f32,
    pub cgd: f32,
}

/// One well's core plugs as `(property, depth, value)` triples from the ACTIVE core set —
/// the reader for point-data tracks, which take any plug property by name rather than the
/// fixed pairs the φ–k and φ–ρg readers return. NULL cells are dropped, not turned into
/// zeros, so an unfilled column contributes no samples instead of a false cloud at 0.
pub fn get_core_point_series(conn: &Connection, well_id: &str) -> DbResult<Vec<(String, f32, f32)>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT depth, cpor, cperm, cgd, csw FROM core_data
         WHERE well_id = ?1 AND set_name = {ACTIVE_CORE_SET} ORDER BY depth"
    ))?;
    let rows = stmt.query_map(params![well_id], |r| {
        Ok((
            r.get::<_, f32>(0)?,
            r.get::<_, Option<f32>>(1)?,
            r.get::<_, Option<f32>>(2)?,
            r.get::<_, Option<f32>>(3)?,
            r.get::<_, Option<f32>>(4)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (depth, cpor, cperm, cgd, csw) = row?;
        for (name, v) in [("CPOR", cpor), ("CPERM", cperm), ("CGD", cgd), ("CSW", csw)] {
            if let Some(v) = v.filter(|v| v.is_finite()) {
                out.push((name.to_string(), depth, v));
            }
        }
    }
    Ok(out)
}

/// One well's core plugs (depth ascending) with porosity + grain density only, from the
/// ACTIVE core set. NULL φ or ρg become NaN so the caller can skip them.
pub fn get_core_por_gd(conn: &Connection, well_id: &str) -> DbResult<Vec<CoreQcRow>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT depth, cpor, cgd FROM core_data
         WHERE well_id = ?1 AND set_name = {ACTIVE_CORE_SET} ORDER BY depth"
    ))?;
    let rows = stmt.query_map(params![well_id], |row| {
        Ok(CoreQcRow {
            depth: row.get(0)?,
            cpor: row.get::<_, Option<f32>>(1)?.unwrap_or(f32::NAN),
            cgd: row.get::<_, Option<f32>>(2)?.unwrap_or(f32::NAN),
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
    // set_name is listed (read-only, like every non-editable column) so a well carrying
    // several core deliveries can be told apart in the grid; edits still target the
    // ACTIVE set only (see `update_core_sample`).
    ("core_data", &["set_name", "depth", "cpor", "cperm", "cgd", "csw"], true, "set_name, depth"),
    ("aux_data", &["dataset", "depth_top", "depth_base", "item", "value_num", "value_text"], true, "dataset, depth_top, item"),
];

#[derive(Debug, Serialize)]
pub struct TablePage {
    pub columns: Vec<String>,
    /// Cells stringified by DuckDB's VARCHAR cast; None = SQL NULL.
    pub rows: Vec<Vec<Option<String>>>,
    pub total_rows: usize,
    /// True when `total_rows` is a display cap rather than a true count — the SQL console's
    /// `LIMIT + 1` probe found more rows than it returned, so the real result is larger. The
    /// paginated inspector path always leaves this false: its `total_rows` is a real COUNT(*).
    pub truncated: bool,
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
        Ok(TablePage { columns: columns.iter().map(|c| c.to_string()).collect(), rows, total_rows, truncated: false })
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
    // Fetch one row beyond the cap so we can tell a result that fills the cap exactly (complete)
    // from one the cap actually truncated. We return at most `limit` rows; the extra row only
    // sets `truncated`, so the panel never reports a capped count as the true total.
    let wrapped = format!("SELECT * FROM ({trimmed}) __sandibumi_q LIMIT {}", limit + 1);
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
    let truncated = rows_out.len() > limit;
    rows_out.truncate(limit);
    let total = rows_out.len();
    Ok(TablePage { columns, rows: rows_out, total_rows: total, truncated })
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
mod well_param_override_tests {
    use super::*;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        conn
    }

    /// `zone_params.well_id` is a UUID column, so a well id has to be a real UUID string —
    /// a readable stand-in like "W1" fails the conversion rather than inserting.
    fn well() -> String {
        Uuid::new_v4().to_string()
    }

    /// The grid's whole contract in one pass: a batch upserts, re-running it updates in place
    /// rather than duplicating, and a `None` clears a well back to the step value.
    #[test]
    fn batch_upserts_updates_and_clears() {
        let mut conn = db();
        let (w1, w2) = (well(), well());
        let n = set_well_param_overrides(
            &mut conn,
            &[
                (w1.clone(), "RW".into(), Some(0.08)),
                (w2.clone(), "RW".into(), Some(0.12)),
                (w1.clone(), "RHO_MA".into(), Some(2.68)),
            ],
        )
        .unwrap();
        assert_eq!(n, 3);
        assert_eq!(list_well_param_overrides(&conn).unwrap().len(), 3);

        // Same key again updates in place — the grid re-sends a well's value on every edit.
        set_well_param_overrides(&mut conn, &[(w1.clone(), "RW".into(), Some(0.09))]).unwrap();
        let rows = list_well_param_overrides(&conn).unwrap();
        assert_eq!(rows.len(), 3, "an upsert must not add a second row for the same well+param");
        let rw1 = rows.iter().find(|r| r.well_id == w1 && r.param_name == "RW").unwrap();
        assert!((rw1.value_num - 0.09).abs() < 1e-6);

        // Clearing removes the row so the step value takes over again.
        set_well_param_overrides(&mut conn, &[(w1.clone(), "RW".into(), None)]).unwrap();
        let rows = list_well_param_overrides(&conn).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(!rows.iter().any(|r| r.well_id == w1 && r.param_name == "RW"));
    }

    /// The grid writes whole-well rows (`zone_name = '*'`) and must never surface, or collide
    /// with, a per-zone value — those still override it per zone at run time.
    #[test]
    fn per_zone_values_are_untouched_and_unlisted() {
        let mut conn = db();
        let w1 = well();
        set_zone_param(&conn, &w1, "SAND_A", "RW", Some(0.05), None).unwrap();
        set_well_param_overrides(&mut conn, &[(w1.clone(), "RW".into(), Some(0.10))]).unwrap();

        let listed = list_well_param_overrides(&conn).unwrap();
        assert_eq!(listed.len(), 1, "only the whole-well row belongs in the grid");
        assert_eq!(listed[0].param_name, "RW");
        assert!((listed[0].value_num - 0.10).abs() < 1e-6);

        // The zone row is still there, with its own value.
        let zone_rows = list_zone_params(&conn, &w1).unwrap();
        let sand = zone_rows.iter().find(|z| z.zone_name == "SAND_A").unwrap();
        assert!((sand.value_num.unwrap() - 0.05).abs() < 1e-6);
    }

    /// A text-valued override is not a number the grid can edit, so it must stay invisible
    /// there rather than render as a blank cell inviting a numeric overwrite.
    #[test]
    fn text_valued_overrides_are_not_listed() {
        let mut conn = db();
        let w1 = well();
        set_zone_param(&conn, &w1, "*", "OPT_NOTE", None, Some("checked")).unwrap();
        set_well_param_overrides(&mut conn, &[(w1, "RW".into(), Some(0.07))]).unwrap();
        let rows = list_well_param_overrides(&conn).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].param_name, "RW");
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

    fn tmp_db(tag: &str) -> String {
        let p = std::env::temp_dir().join(format!("sandibumi_fmt_{tag}_{}.duckdb", Uuid::new_v4()));
        p.to_str().unwrap().to_string()
    }

    fn read_meta(conn: &Connection, key: &str) -> Option<String> {
        conn.query_row("SELECT value FROM project_meta WHERE key = ?", params![key], |r| r.get(0))
            .ok()
    }

    #[test]
    fn fresh_project_is_stamped_with_current_format() {
        let path = tmp_db("fresh");
        let conn = init_db(&path).unwrap();
        assert_eq!(read_meta(&conn, "format_version").as_deref(), Some(FORMAT_VERSION.to_string().as_str()));
        let by = read_meta(&conn, "written_by").unwrap();
        assert!(by.starts_with("SandiBumi "), "written_by should name the app: {by}");
        drop(conn);
        let _ = std::fs::remove_file(&path);
    }

    /// Every file-backed open must cap DuckDB's memory appetite: the engine default (~80%
    /// of RAM) is what let a 2.5 GB field project eat ~6 GB of an 8 GB machine. The cap is
    /// default/4 clamped to [1 GiB, 4 GiB], so whatever this machine's RAM, the applied
    /// setting must land inside that clamp.
    #[test]
    fn init_db_caps_the_engine_memory_limit() {
        let path = tmp_db("memcap");
        let conn = init_db(&path).unwrap();
        let lim: String = conn
            .query_row("SELECT current_setting('memory_limit')", [], |r| r.get(0))
            .unwrap();
        let bytes = parse_mem_bytes(&lim).unwrap_or_else(|| panic!("unparsable limit: {lim}"));
        const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
        assert!(bytes <= 4.0 * GIB * 1.01, "cap must be at most 4 GiB, got {lim}");
        assert!(bytes >= 0.9 * GIB, "cap must be at least ~1 GiB, got {lim}");
        drop(conn);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{path}.wal"));
    }

    #[test]
    fn legacy_project_without_stamp_is_stamped_on_open() {
        let path = tmp_db("legacy");
        {
            // A pre-R-A project: full schema, no project_meta at all.
            let conn = Connection::open(&path).unwrap();
            create_schema(&conn).unwrap();
        }
        let conn = init_db(&path).unwrap();
        assert_eq!(read_meta(&conn, "format_version").as_deref(), Some("1"));
        drop(conn);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn future_format_is_refused_and_left_unmodified() {
        let path = tmp_db("future");
        {
            // A file from a hypothetical future format: it carries a stamp but NOT the
            // current schema (a future format may have renamed any table) — so if
            // create_schema ran despite the refusal, `wells` would appear.
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE project_meta (key VARCHAR PRIMARY KEY, value VARCHAR);
                 INSERT INTO project_meta VALUES ('format_version', '999'), ('written_by', 'SandiBumi 9.9.9');",
            )
            .unwrap();
        }
        let err = init_db(&path).err().expect("a newer file must be refused");
        let msg = err.to_string();
        assert!(msg.contains("format 999"), "must name the file's format: {msg}");
        assert!(msg.contains("SandiBumi 9.9.9"), "must name the writer: {msg}");
        assert!(msg.contains("upgrade SandiBumi"), "must say what to do: {msg}");
        // The refusal must have mutated nothing: no schema, stamp intact.
        let conn = Connection::open(&path).unwrap();
        let wells: i64 = conn
            .query_row("SELECT count(*) FROM duckdb_tables() WHERE table_name = 'wells'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(wells, 0, "create_schema must not have run on a refused file");
        assert_eq!(read_meta(&conn, "format_version").as_deref(), Some("999"), "stamp must be untouched");
        drop(conn);
        let _ = std::fs::remove_file(&path);
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

    /// Editing a curve's identity from the Wells pane: metadata changes, SAMPLES DO NOT, and
    /// the previous identity comes back so the edit can be undone. The rename is the point —
    /// a delivery whose mnemonic is GRN_CS is invisible to a module asking for GR until it is
    /// renamed — so this also pins the normalization (trim + upper-case) that makes the
    /// renamed curve resolvable, and the blank-name refusal that would otherwise orphan it.
    #[test]
    fn curve_meta_edit_renames_without_touching_samples_and_is_reversible() {
        let conn = mem_db();
        let w = Uuid::new_v4();
        insert_well(&conn, w, "W", None, None, None).unwrap();
        let ids = w.to_string();
        let id = upsert_curve_meta(&conn, &ids, "FPROOH", "GRN_CS", Some("GAPI"), Some("GR"), None, None).unwrap();
        insert_curve_samples(&conn, &id, &[1000.0, 1000.5], &[42.0, 43.0]).unwrap();

        let before = update_curve_meta_fields(&conn, &id, "  gr  ", Some("gAPI"), Some("gr")).unwrap();
        assert_eq!(before.mnemonic, "GRN_CS", "the caller needs the OLD name to offer an undo");
        assert_eq!(before.unit.as_deref(), Some("GAPI"));

        let after = list_generic_curve_catalog(&conn, &ids).unwrap();
        let c = after.iter().find(|c| c.curve_id == id).expect("the curve survives a rename");
        assert_eq!(c.mnemonic, "GR", "trimmed and upper-cased, the way imports store mnemonics");
        assert_eq!(c.family.as_deref(), Some("GR"), "family upper-cased too");
        assert_eq!(c.n_samples, 2, "a rename is metadata only — no sample may be lost");
        assert_eq!(get_curve_samples(&conn, &id).unwrap()[0].value, 42.0, "values untouched");

        // Undo restores the previous identity exactly.
        update_curve_meta_fields(&conn, &id, &before.mnemonic, before.unit.as_deref(), before.family.as_deref())
            .unwrap();
        let back = list_generic_curve_catalog(&conn, &ids).unwrap();
        let c = back.iter().find(|c| c.curve_id == id).unwrap();
        assert_eq!(c.mnemonic, "GRN_CS");
        assert_eq!(c.unit.as_deref(), Some("GAPI"));

        // A blank unit means "no unit", stored as NULL rather than an empty string, so the
        // catalog has one representation of absent.
        update_curve_meta_fields(&conn, &id, "GRN_CS", Some("   "), None).unwrap();
        let c = list_generic_curve_catalog(&conn, &ids).unwrap().into_iter().find(|c| c.curve_id == id).unwrap();
        assert!(c.unit.is_none(), "blank unit must be NULL, got {:?}", c.unit);

        // A curve may never be left nameless — resolution is by name, so a blank would orphan it.
        assert!(update_curve_meta_fields(&conn, &id, "   ", None, None).is_err());
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

    /// The SQL console must not report a LIMIT-capped count as the true total. A result larger
    /// than the cap comes back flagged `truncated`; one at or under the cap is complete. The
    /// exactly-at-the-cap case is the one a naive `rows.len() == limit` heuristic gets wrong —
    /// the `LIMIT + 1` probe proves it complete.
    #[test]
    fn readonly_query_flags_truncation_at_the_cap() {
        let conn = mem_db();
        for i in 0..5 {
            insert_well(&conn, Uuid::new_v4(), &format!("W{i}"), None, None, None).unwrap();
        }

        // Cap BELOW the true count: exactly `limit` rows, and truncated is set.
        let capped = run_readonly_query(&conn, "SELECT well_name FROM wells", 3).unwrap();
        assert_eq!(capped.rows.len(), 3, "returns exactly the cap");
        assert_eq!(capped.total_rows, 3);
        assert!(capped.truncated, "a result larger than the cap must be flagged truncated");

        // Cap ABOVE the true count: complete result, not truncated.
        let full = run_readonly_query(&conn, "SELECT well_name FROM wells", 100).unwrap();
        assert_eq!(full.rows.len(), 5);
        assert!(!full.truncated, "a result under the cap is complete");

        // Cap EXACTLY the true count: complete, not truncated (the heuristic's false positive).
        let exact = run_readonly_query(&conn, "SELECT well_name FROM wells", 5).unwrap();
        assert_eq!(exact.rows.len(), 5);
        assert!(!exact.truncated, "a result that fills the cap exactly is complete, not truncated");
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

        migrate_drop_computed_curves_pk(&conn, None).unwrap();
        assert_eq!(pk_count(&conn, "computed_curves"), 0, "PK dropped");
        let rows: i64 = conn.query_row("SELECT COUNT(*) FROM computed_curves", [], |r| r.get(0)).unwrap();
        assert_eq!(rows, 2, "no rows lost in the rebuild");

        // Idempotent: a second run does nothing (no PK to drop).
        migrate_drop_computed_curves_pk(&conn, None).unwrap();
        assert_eq!(pk_count(&conn, "computed_curves"), 0);

        // No-op on a fresh (already PK-less) schema.
        let fresh = mem_db();
        assert_eq!(pk_count(&fresh, "computed_curves"), 0);
        migrate_drop_computed_curves_pk(&fresh, None).unwrap();
        assert_eq!(pk_count(&fresh, "computed_curves"), 0);
    }

    /// A pre-set-era project (core_data / well_path without the set columns) must come
    /// forward reading EXACTLY the numbers it read before: every plug and station becomes
    /// the RAW set/survey, registered active, and the readers return them unchanged.
    /// Idempotent — a second run is a no-op, and a fresh database never migrates at all.
    #[test]
    fn point_data_set_migration_preserves_every_row_and_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        // A LEGACY shape: no set_name / survey_name anywhere.
        conn.execute_batch(
            "CREATE TABLE core_data (
                 well_id UUID NOT NULL, depth FLOAT NOT NULL,
                 cpor FLOAT, cperm FLOAT, cgd FLOAT, csw FLOAT,
                 PRIMARY KEY (well_id, depth));
             CREATE TABLE well_path (
                 well_id UUID NOT NULL, md FLOAT NOT NULL, inc FLOAT NOT NULL, azi FLOAT NOT NULL,
                 tvd FLOAT, tvdss FLOAT,
                 PRIMARY KEY (well_id, md));
             CREATE TABLE core_sets (
                 well_id UUID NOT NULL, set_name VARCHAR NOT NULL, active INTEGER NOT NULL DEFAULT 0,
                 source VARCHAR, imported_at TIMESTAMP NOT NULL DEFAULT now(),
                 PRIMARY KEY (well_id, set_name));
             CREATE TABLE well_surveys (
                 well_id UUID NOT NULL, survey_name VARCHAR NOT NULL, active INTEGER NOT NULL DEFAULT 0,
                 source VARCHAR, datum FLOAT, imported_at TIMESTAMP NOT NULL DEFAULT now(),
                 PRIMARY KEY (well_id, survey_name));",
        )
        .unwrap();
        // Legacy point data too: no set_name column at all, the pre-registry shape.
        conn.execute_batch(
            "CREATE TABLE aux_data (
                 well_id UUID NOT NULL, dataset VARCHAR NOT NULL, depth_top FLOAT NOT NULL,
                 depth_base FLOAT, item VARCHAR NOT NULL, value_num FLOAT, value_text VARCHAR);
             ALTER TABLE aux_data ADD COLUMN IF NOT EXISTS set_name VARCHAR;
             CREATE TABLE aux_sets (
                 well_id UUID NOT NULL, dataset VARCHAR NOT NULL, set_name VARCHAR NOT NULL,
                 active INTEGER NOT NULL DEFAULT 0, source VARCHAR,
                 imported_at TIMESTAMP NOT NULL DEFAULT now(),
                 PRIMARY KEY (well_id, dataset, set_name));
             CREATE TABLE scal_pc (
                 well_id UUID NOT NULL, sample_no INTEGER, depth FLOAT, perm FLOAT, poro FLOAT,
                 pc FLOAT NOT NULL, sw FLOAT NOT NULL, system VARCHAR, ift FLOAT);
             ALTER TABLE scal_pc ADD COLUMN IF NOT EXISTS set_name VARCHAR;
             CREATE TABLE scal_sets (
                 well_id UUID NOT NULL, set_name VARCHAR NOT NULL, active INTEGER NOT NULL DEFAULT 0,
                 source VARCHAR, imported_at TIMESTAMP NOT NULL DEFAULT now(),
                 PRIMARY KEY (well_id, set_name));",
        )
        .unwrap();
        let w = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO aux_data (well_id, dataset, depth_top, item, value_num) VALUES (?1, 'XRD', 2000.0, 'QUARTZ', 45.2)",
            params![w],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO scal_pc (well_id, sample_no, pc, sw) VALUES (?1, 1, 10.0, 0.42)",
            params![w],
        )
        .unwrap();
        conn.execute("INSERT INTO core_data VALUES (?1, 2001.0, 0.22, 150.0, 2.65, 0.35)", params![w]).unwrap();
        conn.execute("INSERT INTO core_data VALUES (?1, 2002.0, 0.18, 20.0, 2.66, 0.42)", params![w]).unwrap();
        conn.execute("INSERT INTO well_path VALUES (?1, 0.0, 0.0, 0.0, 0.0, 25.0)", params![w]).unwrap();
        conn.execute("INSERT INTO well_path VALUES (?1, 1000.0, 0.0, 0.0, 1000.0, -975.0)", params![w]).unwrap();

        migrate_point_data_sets(&conn, None).unwrap();

        // Same numbers, now readable through the set-aware readers.
        let plugs = get_core_plugs(&conn, &w).unwrap();
        assert_eq!(plugs.len(), 2, "no plug lost or duplicated by the rebuild");
        assert!((plugs[0].cpor - 0.22).abs() < 1e-6 && (plugs[1].cperm - 20.0).abs() < 1e-3);
        let path = get_well_path(&conn, &w).unwrap();
        assert_eq!(path.len(), 2);
        assert!((path[1].tvdss - (-975.0)).abs() < 1e-3);

        // Registered as RAW and ACTIVE, so the manager shows something real.
        let sets = list_core_sets(&conn, &w).unwrap();
        assert_eq!(sets.len(), 1);
        assert!(sets[0].active && sets[0].set_name == "RAW" && sets[0].rows == 2);
        let surveys = list_surveys(&conn, &w).unwrap();
        assert!(surveys.len() == 1 && surveys[0].active && surveys[0].stations == 2);
        // Legacy point data is adopted as RAW/active, so the set-filtered readers see it.
        let aux = list_aux_data(&conn, &w, Some("XRD")).unwrap();
        assert_eq!(aux.len(), 1, "unregistered aux rows must stay readable after migration");
        let aux_sets = list_aux_sets(&conn, &w).unwrap();
        assert!(aux_sets.len() == 1 && aux_sets[0].active && aux_sets[0].dataset == "XRD");
        // …and the same for SCAL, the fourth point store.
        assert_eq!(get_scal_pc(&conn, &w).unwrap().len(), 1, "legacy Pc points stay readable");
        let scal_sets = list_scal_sets(&conn, &w).unwrap();
        assert!(scal_sets.len() == 1 && scal_sets[0].active && scal_sets[0].set_name == "RAW");

        // Idempotent, and a no-op on a database that was created with the current schema.
        migrate_point_data_sets(&conn, None).unwrap();
        assert_eq!(get_core_plugs(&conn, &w).unwrap().len(), 2);
        assert_eq!(list_core_sets(&conn, &w).unwrap().len(), 1);
        let fresh = mem_db();
        migrate_point_data_sets(&fresh, None).unwrap();
    }

    fn a_plate(name: &str, top: f32, base: Option<f32>, bytes: &[u8]) -> NewImage {
        NewImage {
            depth_top: top,
            depth_base: base,
            name: name.into(),
            caption: None,
            mime: "image/jpeg".into(),
            width: 800,
            height: 600,
            src_width: Some(4000),
            src_height: Some(3000),
            source_path: Some(format!("D:/plates/{name}.jpg")),
            printable: true,
            data: bytes.to_vec(),
        }
    }

    /// Pictures follow the universal delivery-set rule: a second delivery lands BESIDE the
    /// first and only one is live, so a re-shot core cannot double the plates on a track.
    #[test]
    fn a_second_image_delivery_lands_beside_the_first_and_only_one_is_live() {
        let conn = mem_db();
        let wid = Uuid::new_v4();
        insert_well(&conn, wid, "BLSO-IMG", None, None, None).unwrap();
        let w = wid.to_string();

        let first = resolve_image_set_name(&conn, &w, "THIN SECTION", "PETRO").unwrap();
        assert_eq!(first, "PETRO");
        insert_well_images(&conn, &w, "THIN SECTION", &first, Some("lab-2024"), &[
            a_plate("TS-1", 1010.0, None, b"\xFF\xD8jpeg-one\xFF\xD9"),
            a_plate("TS-2", 1020.0, None, b"\xFF\xD8jpeg-two\xFF\xD9"),
        ])
        .unwrap();

        // Same name again must NOT overwrite — it suffixes, exactly as core and curve sets do.
        let second = resolve_image_set_name(&conn, &w, "THIN SECTION", "PETRO").unwrap();
        assert_eq!(second, "PETRO_1");
        insert_well_images(&conn, &w, "THIN SECTION", &second, Some("lab-2026"), &[a_plate(
            "TS-9", 1015.0, None, b"\xFF\xD8jpeg-nine\xFF\xD9",
        )])
        .unwrap();

        // The newest delivery is live, and a reader sees ONE of them — never the union.
        let live = list_well_images(&conn, &w, Some("THIN SECTION")).unwrap();
        assert_eq!(live.len(), 1, "two deliveries must never both be drawn");
        assert_eq!(live[0].name, "TS-9");

        set_active_image_set(&conn, &w, "THIN SECTION", "PETRO").unwrap();
        let live = list_well_images(&conn, &w, Some("THIN SECTION")).unwrap();
        assert_eq!(live.len(), 2, "switching back restores the earlier delivery whole");
        assert_eq!(live[0].name, "TS-1");

        // A different dataset is activated independently.
        insert_well_images(&conn, &w, "CORE PHOTO", "RAW", None, &[a_plate(
            "CP-1", 1000.0, Some(1001.0), b"\xFF\xD8jpeg-core\xFF\xD9",
        )])
        .unwrap();
        assert_eq!(list_well_images(&conn, &w, None).unwrap().len(), 3, "one delivery of EACH dataset");
        let sets = list_image_sets(&conn, &w).unwrap();
        assert_eq!(sets.len(), 3);
        assert_eq!(list_image_datasets(&conn, &w).unwrap(), vec![("CORE PHOTO".into(), 1), ("THIN SECTION".into(), 2)]);
    }

    #[test]
    fn a_listing_reports_the_stored_size_without_reading_the_pixels() {
        // The whole reason `ImageInfo` has no `data`: a well of 300 core photographs must
        // list in kilobytes. This pins that the size comes from the row, not from a blob the
        // caller had to load.
        let conn = mem_db();
        let wid = Uuid::new_v4();
        insert_well(&conn, wid, "BLSO-IMG2", None, None, None).unwrap();
        let w = wid.to_string();
        let bytes = b"\xFF\xD8_______________\xFF\xD9";
        insert_well_images(&conn, &w, "CORE PHOTO", "RAW", None, &[a_plate("CP-1", 1000.0, Some(1001.0), bytes)])
            .unwrap();

        let info = &list_well_images(&conn, &w, None).unwrap()[0];
        assert_eq!(info.bytes, bytes.len() as i64);
        assert_eq!(info.depth_base, Some(1001.0));
        assert_eq!(info.src_width, Some(4000), "the delivered original's size stays traceable");
        // …and the pixels come back byte-identical when actually asked for.
        let (mime, data) = get_well_image(&conn, &info.image_id).unwrap();
        assert_eq!(mime, "image/jpeg");
        assert_eq!(data, bytes.to_vec());

        // The print reader is depth-windowed and set-filtered, and an interval plate counts
        // as present when any part of it is on the page.
        assert_eq!(read_images_for_print(&conn, &w, "CORE PHOTO", 1000.5, 1010.0).unwrap().len(), 1);
        assert_eq!(read_images_for_print(&conn, &w, "CORE PHOTO", 1002.0, 1010.0).unwrap().len(), 0);
    }

    #[test]
    fn deleting_the_live_image_delivery_hands_over_to_the_next_newest() {
        let conn = mem_db();
        let wid = Uuid::new_v4();
        insert_well(&conn, wid, "BLSO-IMG3", None, None, None).unwrap();
        let w = wid.to_string();
        insert_well_images(&conn, &w, "SEM", "RUN1", None, &[a_plate("A", 1000.0, None, b"\xFF\xD8a\xFF\xD9")]).unwrap();
        insert_well_images(&conn, &w, "SEM", "RUN2", None, &[a_plate("B", 1001.0, None, b"\xFF\xD8b\xFF\xD9")]).unwrap();
        assert_eq!(list_well_images(&conn, &w, None).unwrap()[0].name, "B");

        assert_eq!(delete_image_set(&conn, &w, "SEM", "RUN2").unwrap(), 1);
        let live = list_well_images(&conn, &w, None).unwrap();
        assert_eq!(live.len(), 1, "the survivor takes over rather than leaving the track blank");
        assert_eq!(live[0].name, "A");
    }

    /// R-B (RELEASE §3.2): when the destructive PK-drop migration actually fires against a
    /// real file, a complete pre-migration copy must exist beside it FIRST — openable, PK
    /// still present, every row intact. Opens that don't migrate must write no backup, and
    /// an existing backup must never be overwritten (collision → timestamped name).
    #[test]
    fn destructive_migration_backs_up_the_project_file_first() {
        let dir = std::env::temp_dir().join(format!("sandibumi-rb-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("field.duckdb");
        let db_path_str = db_path.to_str().unwrap().to_string();
        let count_backups = || -> usize {
            std::fs::read_dir(&dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().contains("-backup"))
                .count()
        };
        let make_legacy_file = |path: &str| {
            // A LEGACY project: computed_curves still carries the 3-column PK.
            let conn = Connection::open(path).unwrap();
            conn.execute_batch(
                "CREATE TABLE computed_curves (
                     well_id UUID NOT NULL, depth FLOAT NOT NULL, curve_name VARCHAR NOT NULL, value FLOAT,
                     PRIMARY KEY (well_id, depth, curve_name));",
            )
            .unwrap();
            let w = Uuid::new_v4().to_string();
            conn.execute("INSERT INTO computed_curves VALUES (?1, 1000.0, 'PHIE', 0.2)", params![w]).unwrap();
            conn.execute("INSERT INTO computed_curves VALUES (?1, 1000.5, 'PHIE', 0.21)", params![w]).unwrap();
        };

        make_legacy_file(&db_path_str);
        let conn = Connection::open(&db_path_str).unwrap();
        migrate_drop_computed_curves_pk(&conn, Some(&db_path_str)).unwrap();
        assert_eq!(pk_count(&conn, "computed_curves"), 0, "live file migrated");

        let backup = dir.join(format!("field.pre-{FORMAT_VERSION}-backup.duckdb"));
        assert!(backup.exists(), "backup must exist beside the project, named per RELEASE 3.2");
        {
            let bconn = Connection::open(backup.to_str().unwrap()).unwrap();
            assert_eq!(pk_count(&bconn, "computed_curves"), 1, "backup is the PRE-migration file: PK intact");
            let rows: i64 = bconn.query_row("SELECT COUNT(*) FROM computed_curves", [], |r| r.get(0)).unwrap();
            assert_eq!(rows, 2, "backup holds every pre-migration row (engine copy reads WAL state)");
        }

        // Already-migrated open: no second backup.
        migrate_drop_computed_curves_pk(&conn, Some(&db_path_str)).unwrap();
        assert_eq!(count_backups(), 1, "a non-destructive open must not write a backup");
        drop(conn);

        // Collision: a NEW legacy file at the same path must not overwrite the old backup.
        std::fs::remove_file(&db_path).unwrap();
        let _ = std::fs::remove_file(dir.join("field.duckdb.wal"));
        make_legacy_file(&db_path_str);
        let conn = Connection::open(&db_path_str).unwrap();
        migrate_drop_computed_curves_pk(&conn, Some(&db_path_str)).unwrap();
        assert_eq!(count_backups(), 2, "second destructive run takes a timestamped name, never overwrites");
        drop(conn);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// A fresh project (born PK-less) must never see a backup file: the migration is a no-op
    /// and the R-A stamp/generic-store migrations are additive, so opening writes nothing
    /// beside the file.
    #[test]
    fn fresh_project_open_writes_no_backup() {
        let dir = std::env::temp_dir().join(format!("sandibumi-rb-fresh-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("fresh.duckdb");
        let db_path_str = db_path.to_str().unwrap().to_string();

        let conn = init_db(&db_path_str).unwrap();
        migrate_drop_computed_curves_pk(&conn, Some(&db_path_str)).unwrap();
        let backups = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains("-backup"))
            .count();
        assert_eq!(backups, 0, "fresh projects must not accumulate backup files");
        drop(conn);
        std::fs::remove_dir_all(&dir).unwrap();
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

/// Applies a constant depth shift to the ACTIVE core set's plugs (core-to-log alignment).
/// Exactly reversible with -delta, so the frontend makes it undoable. Other deliveries of
/// the same well keep their own depths — a shift belongs to the set it was judged on.
pub fn shift_core_depths(conn: &Connection, well_id: &str, delta: f32) -> DbResult<usize> {
    let n = conn.execute(
        &format!("UPDATE core_data SET depth = depth + ?2 WHERE well_id = ?1 AND set_name = {ACTIVE_CORE_SET}"),
        params![well_id, delta],
    )?;
    Ok(n)
}

/// Edits one core-plug sample value (NaN = missing) in the ACTIVE core set.
pub fn update_core_sample(conn: &Connection, well_id: &str, depth: f32, column: &str, value: f32) -> Result<(), String> {
    const EDITABLE: &[&str] = &["cpor", "cperm", "cgd", "csw"];
    if !EDITABLE.contains(&column) {
        return Err(format!("column '{column}' is not editable"));
    }
    let n = conn
        .execute(
            &format!(
                "UPDATE core_data SET {column} = ?3 WHERE well_id = ?1 AND depth = ?2
                 AND set_name = {ACTIVE_CORE_SET}"
            ),
            params![well_id, depth, value],
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

/// One generic curve's editable identity — what `update_curve_meta_fields` returns so the
/// caller can offer an undo without a second query.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CurveMetaEdit {
    pub mnemonic: String,
    pub unit: Option<String>,
    pub family: Option<String>,
}

/// Renames / re-units / re-families one imported curve, returning its PREVIOUS identity so
/// the edit can be pushed onto the undo stack (rule 8 — data edits are undoable).
///
/// This is metadata only: not one sample is touched, so it is exactly reversible. It matters
/// more than cosmetics though — the mnemonic and family are what `fetch_generic_curve_aligned`
/// resolves module inputs by, so renaming a curve REPOINTS what modules read. Blank names are
/// refused for that reason; the mnemonic is upper-cased and trimmed to match how imports store
/// them (resolution is case-insensitive, but a mixed-case catalog reads as a mess). An empty
/// unit/family string is stored as NULL rather than "", so "no unit" has one representation.
pub fn update_curve_meta_fields(
    conn: &Connection,
    curve_id: &str,
    mnemonic: &str,
    unit: Option<&str>,
    family: Option<&str>,
) -> DbResult<CurveMetaEdit> {
    let mnemonic = mnemonic.trim().to_uppercase();
    if mnemonic.is_empty() {
        return Err(DbError::Invalid("a curve must keep a name".into()));
    }
    let blank_to_none = |s: Option<&str>| s.map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
    let unit = blank_to_none(unit);
    let family = blank_to_none(family).map(|f| f.to_uppercase());

    with_txn(conn, |conn| {
        let before: CurveMetaEdit = conn.query_row(
            "SELECT mnemonic, unit, family FROM curve_meta WHERE curve_id = ?1",
            params![curve_id],
            |r| Ok(CurveMetaEdit { mnemonic: r.get(0)?, unit: r.get(1)?, family: r.get(2)? }),
        )?;
        conn.execute(
            "UPDATE curve_meta SET mnemonic = ?2, unit = ?3, family = ?4 WHERE curve_id = ?1",
            params![curve_id, mnemonic, unit, family],
        )?;
        Ok(before)
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

/// One deviation survey of one well, as the set manager shows it.
#[derive(Debug, Clone, Serialize)]
pub struct SurveyInfo {
    pub survey_name: String,
    pub stations: i64,
    pub active: bool,
    pub source: Option<String>,
    pub datum: Option<f32>,
    pub imported_at: Option<String>,
}

/// A well's surveys, active first then newest, with station counts.
pub fn list_surveys(conn: &Connection, well_id: &str) -> DbResult<Vec<SurveyInfo>> {
    let mut stmt = conn.prepare(
        "SELECT s.survey_name, s.active, s.source, s.datum, CAST(s.imported_at AS VARCHAR),
                (SELECT COUNT(*) FROM well_path p WHERE p.well_id = s.well_id AND p.survey_name = s.survey_name)
         FROM well_surveys s WHERE s.well_id = ?1
         ORDER BY s.active DESC, s.imported_at DESC, s.survey_name",
    )?;
    let rows = stmt.query_map(params![well_id], |r| {
        Ok(SurveyInfo {
            survey_name: r.get(0)?,
            active: r.get::<_, i32>(1)? != 0,
            source: r.get(2)?,
            datum: r.get(3)?,
            imported_at: r.get(4)?,
            stations: r.get(5)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// The name a new survey will be stored under — `desired`, else `desired_1`, `_2`, …
/// (never overwrites an earlier survey; same rule as core sets and curve sets).
pub fn resolve_survey_name(conn: &Connection, well_id: &str, desired: &str) -> DbResult<String> {
    let base = {
        let t = desired.trim().to_uppercase().replace(' ', "_");
        if t.is_empty() { "SURVEY".to_string() } else { t }
    };
    let taken = |name: &str| -> DbResult<bool> {
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM well_surveys WHERE well_id = ?1 AND upper(survey_name) = ?2",
            params![well_id, name],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    };
    if !taken(&base)? {
        return Ok(base);
    }
    for i in 1..1000 {
        let cand = format!("{base}_{i}");
        if !taken(&cand)? {
            return Ok(cand);
        }
    }
    Err(DbError::LengthMismatch(format!("too many surveys named {base}")))
}

/// Makes one survey the well's live one. The caller must re-materialize TVD/TVDSS
/// afterwards (`ingest::materialize_tvd_curves`) — the stored curves follow the active
/// survey, and leaving them stale would silently keep the old geometry in every height
/// calculation.
pub fn set_active_survey(conn: &Connection, well_id: &str, survey_name: &str) -> DbResult<()> {
    with_txn(conn, |conn| {
        conn.execute("UPDATE well_surveys SET active = 0 WHERE well_id = ?1", params![well_id])?;
        let n = conn.execute(
            "UPDATE well_surveys SET active = 1 WHERE well_id = ?1 AND survey_name = ?2",
            params![well_id, survey_name],
        )?;
        if n == 0 {
            return Err(DbError::LengthMismatch(format!("no survey '{survey_name}' on this well")));
        }
        Ok(())
    })
}

/// Deletes one survey; the newest survivor becomes active so a well is never left with
/// stations no reader can see.
pub fn delete_survey(conn: &Connection, well_id: &str, survey_name: &str) -> DbResult<usize> {
    let removed = with_txn(conn, |conn| -> DbResult<usize> {
        let n = conn.execute(
            "DELETE FROM well_path WHERE well_id = ?1 AND survey_name = ?2",
            params![well_id, survey_name],
        )?;
        conn.execute(
            "DELETE FROM well_surveys WHERE well_id = ?1 AND survey_name = ?2",
            params![well_id, survey_name],
        )?;
        Ok(n)
    })?;
    let has_active: i64 = conn.query_row(
        "SELECT COUNT(*) FROM well_surveys WHERE well_id = ?1 AND active = 1",
        params![well_id],
        |r| r.get(0),
    )?;
    if has_active == 0 {
        let next: Option<String> = conn
            .query_row(
                "SELECT survey_name FROM well_surveys WHERE well_id = ?1 ORDER BY imported_at DESC LIMIT 1",
                params![well_id],
                |r| r.get(0),
            )
            .ok();
        if let Some(next) = next {
            set_active_survey(conn, well_id, &next)?;
        }
    }
    Ok(removed)
}

/// Stores one deviation survey (with computed TVD/TVDSS) under `survey_name`, replacing
/// only that survey's stations and making it the well's active one. Earlier surveys of the
/// same well are untouched.
pub fn insert_well_path(
    conn: &Connection,
    well_id: &str,
    survey_name: &str,
    source: Option<&str>,
    datum: Option<f32>,
    stations: &[crate::deviation::Station],
) -> DbResult<()> {
    with_txn(conn, |conn| {
        conn.execute(
            "DELETE FROM well_path WHERE well_id = ?1 AND survey_name = ?2",
            params![well_id, survey_name],
        )?;
        let mut appender: Appender = conn.appender("well_path")?;
        for s in stations {
            appender.append_row(params![well_id, survey_name, s.md, s.inc, s.azi, s.tvd, s.tvdss])?;
        }
        appender.flush()?;
        conn.execute(
            "DELETE FROM well_surveys WHERE well_id = ?1 AND survey_name = ?2",
            params![well_id, survey_name],
        )?;
        conn.execute("UPDATE well_surveys SET active = 0 WHERE well_id = ?1", params![well_id])?;
        conn.execute(
            "INSERT INTO well_surveys (well_id, survey_name, active, source, datum) VALUES (?1, ?2, 1, ?3, ?4)",
            params![well_id, survey_name, source, datum],
        )?;
        Ok(())
    })
}

/// Reads one well's ACTIVE deviation survey (ordered by MD) for TVD-aware display.
pub fn get_well_path(conn: &Connection, well_id: &str) -> DbResult<Vec<WellPathStation>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT md, inc, azi, tvd, tvdss FROM well_path
         WHERE well_id = ?1 AND survey_name = {ACTIVE_SURVEY} ORDER BY md"
    ))?;
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

/// One well's whole-well override of a module parameter — a `zone_params` row whose zone is
/// `*`. At run time `workflow::resolve_param_arrays` fills the whole curve with it before any
/// named zone overrides it, so it sits between the step's value and the per-zone values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellParamOverride {
    pub well_id: String,
    pub param_name: String,
    pub value_num: f32,
}

/// Every whole-well parameter override in the project, for the per-well parameter grid.
///
/// Deliberately unfiltered by well: the grid shows hundreds to thousands of rows, and one
/// scan of a table holding a handful of rows per well beats either N round trips or an
/// `IN (...)` list long enough to hit a binding limit. Text-valued overrides are skipped —
/// the grid edits numeric module parameters, and silently rendering a text override as an
/// empty cell would invite overwriting it with a number.
pub fn list_well_param_overrides(conn: &Connection) -> DbResult<Vec<WellParamOverride>> {
    let mut stmt = conn.prepare(
        "SELECT well_id, param_name, value_num FROM zone_params
         WHERE zone_name = '*' AND value_num IS NOT NULL
         ORDER BY well_id, param_name",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(WellParamOverride { well_id: r.get(0)?, param_name: r.get(1)?, value_num: r.get(2)? })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Applies a batch of whole-well parameter overrides in ONE transaction: a `Some` value
/// upserts, a `None` clears that well's override so the step (or manifest) value takes over
/// again. Returns how many rows were written or cleared.
///
/// Atomic on purpose. The grid's fill-column and paste actions touch every well at once, and
/// undo replays the previous values the same way — a half-applied sweep would leave a field
/// with two different parameter sets and no record of where the boundary fell.
pub fn set_well_param_overrides(
    conn: &mut Connection,
    entries: &[(String, String, Option<f32>)],
) -> DbResult<usize> {
    let tx = conn.transaction()?;
    let mut n = 0usize;
    for (well_id, param_name, value) in entries {
        match value {
            Some(v) => {
                tx.execute(
                    "INSERT INTO zone_params (well_id, zone_name, param_name, value_num, value_text)
                     VALUES (?1, '*', ?2, ?3, NULL)
                     ON CONFLICT (well_id, zone_name, param_name)
                     DO UPDATE SET value_num = excluded.value_num, value_text = NULL",
                    params![well_id, param_name, v],
                )?;
            }
            None => {
                tx.execute(
                    "DELETE FROM zone_params WHERE well_id = ?1 AND zone_name = '*' AND param_name = ?2",
                    params![well_id, param_name],
                )?;
            }
        }
        n += 1;
    }
    tx.commit()?;
    Ok(n)
}
