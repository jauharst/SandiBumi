use duckdb::{
    arrow::{
        array::{ArrayRef, Float32Array, StringArray},
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    },
    params, params_from_iter, Appender, Connection, OptionalExt,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("duckdb error: {0}")]
    DuckDb(#[from] duckdb::Error),
    #[error("column length mismatch: {0}")]
    LengthMismatch(String),
    #[error("columnar import batch error: {0}")]
    ColumnarBatch(String),
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
pub const FORMAT_VERSION: i64 = 2;

/// Opens (creating if needed) the embedded DuckDB file and applies the schema.
///
/// The format check runs BEFORE `create_schema` on purpose: `CREATE TABLE IF NOT EXISTS`
/// is itself a mutation, and a file written by a newer SandiBumi must be refused
/// untouched, not first edited into a hybrid of two formats.
pub fn init_db(path: &str) -> DbResult<Connection> {
    let conn = Connection::open(path)?;
    tune_connection(&conn);
    let source_format_version = check_and_stamp_format(&conn)?;
    remember_migration_source_format(&conn, source_format_version)?;
    create_schema(&conn)?;
    migrate_tvdss_positive_down(&conn, if path == ":memory:" { None } else { Some(path) })?;
    stamp_current_format(&conn)?;
    Ok(conn)
}

/// Caps DuckDB's memory appetite. The engine's factory default allows itself ~80% of the
/// machine's RAM, which it will happily fill during a large scan, migration backup or
/// COPY FROM DATABASE — on a 2.5 GB field project that showed up as ~6 GB of the user's
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

/// The file's own block accounting: (total_bytes, free_bytes), from DuckDB's
/// `pragma_database_size()`. Free blocks are space the file holds but no live row uses —
/// what DELETE leaves behind, since a DuckDB file never shrinks in place. Columns are
/// selected BY NAME so a future pragma reordering fails loudly instead of swapping the
/// two counts.
pub fn dead_space(conn: &Connection) -> DbResult<(u64, u64)> {
    conn.query_row(
        "SELECT block_size, total_blocks, free_blocks FROM pragma_database_size()",
        [],
        |r| {
            let block: i64 = r.get(0)?;
            let total: i64 = r.get(1)?;
            let free: i64 = r.get(2)?;
            Ok((
                (block.max(0) as u64) * (total.max(0) as u64),
                (block.max(0) as u64) * (free.max(0) as u64),
            ))
        },
    )
    .map_err(Into::into)
}

/// The dead fraction past which an open mentions Compact Project. Geolog repacks a well
/// file automatically when it drops below 75% full — its WELL_FULL default (T2 ingest of
/// database_03_database_format_hc.3.3.html; banked in docs/PRD_v2/22_database-model.md,
/// the tracked home) — so a quarter dead is the level a field tool
/// already treats as worth acting on. SandiBumi never repacks on its own (compaction
/// rewrites the whole file and parks the original, which is the user's call to make), so
/// the same threshold drives a NOTE instead.
pub const COMPACT_NOTE_DEAD_FRACTION: f64 = 0.25;

/// Below this much reclaimable space the note stays quiet regardless of fraction: a 40 MB
/// project that is half dead would win back 20 MB, and a boot notice about 20 MB is noise.
/// An engineering floor, chosen not cited.
pub const COMPACT_NOTE_MIN_DEAD_BYTES: u64 = 64 * 1024 * 1024;

/// The Compact Project suggestion, or None while the file is lean. Pure arithmetic on the
/// two counts `dead_space` returns, so the threshold is testable without a bloated file.
pub fn compact_suggestion(total_bytes: u64, free_bytes: u64) -> Option<String> {
    if total_bytes == 0 || free_bytes < COMPACT_NOTE_MIN_DEAD_BYTES {
        return None;
    }
    let frac = free_bytes as f64 / total_bytes as f64;
    if frac < COMPACT_NOTE_DEAD_FRACTION {
        return None;
    }
    Some(format!(
        "This project file holds {} of dead space ({}% of {}) left behind by re-runs and purges - a DuckDB file never shrinks in place. Compact Project (Data ribbon, Tools menu) rewrites it at its live size",
        fmt_bytes(free_bytes),
        (frac * 100.0).round() as u64,
        fmt_bytes(total_bytes)
    ))
}

/// Human-readable byte counts for the boot notes (1 decimal past MiB, whole numbers below).
fn fmt_bytes(b: u64) -> String {
    const MIB: f64 = 1024.0 * 1024.0;
    let b = b as f64;
    if b >= 1024.0 * MIB {
        format!("{:.1} GiB", b / (1024.0 * MIB))
    } else if b >= MIB {
        format!("{:.0} MiB", b / MIB)
    } else {
        format!("{:.0} KiB", b / 1024.0)
    }
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
///   definition ≤ this build's format, so create the table but stamp only after migration.
/// - stamped ≤ `FORMAT_VERSION` — open normally; retain an older stamp until every
///   format-defining migration succeeds, then advance it through `stamp_current_format`.
/// - stamped > `FORMAT_VERSION` — refuse, naming both versions and the app that wrote
///   the file. Silently misreading a newer project is the one unacceptable behaviour.
fn check_and_stamp_format(conn: &Connection) -> DbResult<i64> {
    let has_meta: i64 = conn.query_row(
        "SELECT count(*) FROM duckdb_tables() WHERE table_name = 'project_meta'",
        [],
        |r| r.get(0),
    )?;
    if has_meta == 0 {
        conn.execute_batch("CREATE TABLE project_meta (key VARCHAR PRIMARY KEY, value VARCHAR);")?;
        return Ok(0);
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
    Ok(ver)
}

/// Retains the format observed before schema work for every destructive migration in this
/// connection. `stamp_current_format` deliberately runs before some legacy migrations owned by
/// `project::open_and_migrate`; a connection-local TEMP row prevents that target stamp from erasing
/// the source identity those later backups must carry. Nothing is persisted into the project.
fn remember_migration_source_format(conn: &Connection, source_format_version: i64) -> DbResult<()> {
    conn.execute_batch(
        "CREATE TEMP TABLE __sandibumi_migration_source_format (source_format_version BIGINT NOT NULL);",
    )?;
    conn.execute(
        "INSERT INTO __sandibumi_migration_source_format VALUES (?1)",
        params![source_format_version],
    )?;
    Ok(())
}

/// Returns the source-format identity captured at open. Focused recovery tools and tests that
/// operate on an already-open connection fall back to its persistent stamp; a pre-stamp file is
/// format 0, matching `check_and_stamp_format`.
fn migration_source_format(conn: &Connection) -> DbResult<i64> {
    let has_context: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duckdb_tables()
         WHERE temporary AND table_name = '__sandibumi_migration_source_format'",
        [],
        |row| row.get(0),
    )?;
    if has_context > 0 {
        return conn
            .query_row(
                "SELECT source_format_version FROM __sandibumi_migration_source_format LIMIT 1",
                [],
                |row| row.get(0),
            )
            .map_err(DbError::from);
    }

    let has_meta: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duckdb_tables() WHERE table_name = 'project_meta'",
        [],
        |row| row.get(0),
    )?;
    if has_meta == 0 {
        return Ok(0);
    }
    let raw: String = conn.query_row(
        "SELECT coalesce(max(CASE WHEN key = 'format_version' THEN value END), '0') FROM project_meta",
        [],
        |row| row.get(0),
    )?;
    Ok(raw.parse::<i64>().unwrap_or(0))
}

/// Stamps the current file format only after every format-defining migration in `init_db`
/// succeeds. Stamping first would let a failed TVDSS conversion leave a format-2 label on
/// format-1 values, after which the next open would have no trustworthy way to detect the mix.
fn stamp_current_format(conn: &Connection) -> DbResult<()> {
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
        params![concat!("SandiBumi ", env!("CARGO_PKG_VERSION"))],
    )?;
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
    crate::schema_vocab::validate_schema_vocabularies().map_err(DbError::Invalid)?;
    let standard = crate::schema_vocab::standard_projections();
    let schema = format!(
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
        -- "50S" for the Mahakam Delta, "48S"/"49S" for the Java Sea) so multi-zone fields can be
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
{standard_table_ddl},
            PRIMARY KEY (well_id, depth)
        );

        CREATE TABLE IF NOT EXISTS high_res_curves (
            well_id     UUID NOT NULL,
            depth       FLOAT NOT NULL,
            micro_res   FLOAT,
            image_pad   FLOAT,
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
            PRIMARY KEY (well_id, set_name, curve_name, depth),
            -- What each value in `samples` IS a measurement AT: the pressure of a Pc step, the
            -- T2 time of an NMR bin, the wavelength of a spectrum. NULL means the values are an
            -- index-ordered set with no axis of their own (Monte Carlo realizations, which is
            -- what this table held first) -- absent, never a made-up 0,1,2,....
            --
            -- Must stay the LAST column: `create_schema` and the migration have to agree, and a
            -- migrated database gets it appended.
            axis        BLOB
        );

        -- Long/tall store for module + equation outputs: one row per (well, depth, curve),
        -- so adding a new curve never requires a schema migration.
        --
        -- NO primary key ON PURPOSE (perf). The natural key is (well_id, depth, curve_name),
        -- but a 3-column PRIMARY KEY forces DuckDB to maintain an ART uniqueness index on
        -- every inserted row — measured ~3.7× slower inserts (311k vs 1.16M rows/s), which
        -- dominated field-scale runs (2000 wells). Uniqueness is instead guaranteed by the
        -- WRITE DISCIPLINE: `ancestry::write_versioned_rows_raw` always DELETEs a well's rows
        -- for the curve names it is about to write before appending fresh ones, and the point-update
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
            -- SB-DBM-009 / DEC-022: a provenance timestamp is an unambiguous UTC INSTANT,
            -- converted only at display. now() alone lands the session's local wall clock.
            created_at  TIMESTAMP NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
            -- 'STANDARD' = written on the well's own depth grid (every module).
            -- 'OWN'      = the set carries its own depth column (reframe.rs), and every read
            --              through it runs on that frame instead. Declared, never inferred.
            frame       VARCHAR NOT NULL DEFAULT '{standard_frame}',
            -- Declared by the writer, never inferred from coincidentally regular depths. Legacy
            -- rows remain NULL because their original declaration cannot be recovered.
            sampling_style VARCHAR,
            duplicate_resolution VARCHAR,
            -- NULL = legacy/unclassified. Every complete production write starts CLEAN and is
            -- changed atomically to DEGRADED when a structured event is persisted with its rows.
            outcome_state VARCHAR CHECK (outcome_state IN ('CLEAN', 'DEGRADED')),
            -- DEC-045/DEC-039: the per-VERSION free-text comment - the branch a POR module took
            -- and every limit that bound, or what the user did on that run. Versions never
            -- inherit it: a comment describes ONE run. NULL = no comment recorded.
            comment VARCHAR,
            -- SB-ENV-005 (DEC-031(b), signed DRAFT_ENV005 under DEC-076): the one authoritative
            -- applied-step manifest, riding the versioned interpretation it describes. Written in
            -- the SAME transaction that allocates the version - the manifest and its version
            -- exist atomically or not at all. NULL = a pre-contract version whose step history
            -- cannot be recovered: preserved as UNKNOWN, never backfilled and never read as an
            -- empty step list (an empty list claims "nothing was applied", which is an answer).
            applied_steps_json VARCHAR
        );

        -- SB-DBM-011 (DEC-020/022/023): the STRUCTURED audit - Geolog's taxonomy adopted
        -- wholesale (T2 AuditTrail). This sits BESIDE log_sets ("how was this curve made")
        -- and beside the legacy processLog text history, which stays visible and is not
        -- relabelled: the audit answers "what did someone do to this project" as queryable
        -- rows. The chapter's `user` field is stored as operator + operator_kind because
        -- DEC-020 requires the explicit HUMAN/AUTOMATED classification (never inferred from
        -- the Windows account). entry_seq makes "uninterrupted" decidable by ORDER, not by
        -- an invented elapsed-time window - AUDIT_ENTRY_COLLAPSE_WINDOW ships ABSENT by
        -- design because Geolog's rule is uninterruptedness, not time.
        CREATE SEQUENCE IF NOT EXISTS audit_entry_counter;
        CREATE TABLE IF NOT EXISTS audit_entry (
            entry_id      UUID PRIMARY KEY,
            entry_seq     BIGINT NOT NULL DEFAULT nextval('audit_entry_counter'),
            well_id       UUID,
            ts_utc        TIMESTAMP NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
            operator      VARCHAR NOT NULL,
            operator_kind VARCHAR NOT NULL CHECK (operator_kind IN ('HUMAN', 'AUTOMATED')),
            view          VARCHAR NOT NULL,
            source        VARCHAR NOT NULL,
            comment       VARCHAR,
            -- DEC-023's narrow seam: rename or move a top and the same run means something
            -- different, so a zone-scoped entry names the zone-set identity it saw.
            zone_set_version INTEGER,
            zone_set_digest  VARCHAR,
            -- Geolog collapses uninterrupted repeats into ONE entry; the count keeps the
            -- collapse honest about how many gestures it absorbed.
            repeat_count  INTEGER NOT NULL DEFAULT 1
        );
        CREATE TABLE IF NOT EXISTS audit_detail (
            entry_id UUID NOT NULL,
            seq      INTEGER NOT NULL,
            location VARCHAR NOT NULL CHECK (
                location IN ('PARAMETER', 'COMMENT', 'SET', 'CONSTANT', 'INTERVAL', 'LOG', 'ATTRIBUTE')
            ),
            mode     VARCHAR NOT NULL CHECK (
                mode IN ('INPUT', 'OUTPUT', 'DELETE', 'RENAME', 'SAVE', 'SAVE_AS', 'SAVE_CANCEL')
            ),
            unit     VARCHAR,
            name     VARCHAR NOT NULL,
            value    VARCHAR,
            PRIMARY KEY (entry_id, seq)
        );
        -- DEC-023: the zone-set identity/version seam. A digest of the well's zones in
        -- depth order; a new version row appears only when the zones actually change.
        CREATE TABLE IF NOT EXISTS zone_set_versions (
            well_id    UUID NOT NULL,
            version    INTEGER NOT NULL,
            digest     VARCHAR NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
            PRIMARY KEY (well_id, version)
        );

        -- Structured reasons a durable run is DEGRADED. Multiple workflow steps may append to one
        -- set_id, so position is monotone per run rather than keyed by a message someone might edit.
        CREATE TABLE IF NOT EXISTS run_degradations (
            set_id      UUID NOT NULL,
            position    INTEGER NOT NULL,
            module      VARCHAR NOT NULL,
            kind        VARCHAR NOT NULL CHECK (
                kind IN ('CLAMPED', 'DEFAULTED', 'TRUNCATED', 'SUBSTITUTED_INPUT', 'ENDPOINT_INVALID')
            ),
            detail      VARCHAR NOT NULL,
            occurrences BIGINT NOT NULL CHECK (occurrences > 0),
            PRIMARY KEY (set_id, position)
        );

        -- Queryable parameter custody for one run. The full ancestry JSON remains the portable
        -- record; this relation is the indexed project query required by SB-DBM-003. A present
        -- value always has a non-empty source. A deliberately unsupplied required parameter is
        -- represented only by the named REQUIRED_UNSET state with both value and source NULL.
        CREATE TABLE IF NOT EXISTS run_parameters (
            set_id      UUID NOT NULL,
            position    INTEGER NOT NULL,
            name        VARCHAR NOT NULL,
            value_json  VARCHAR,
            source      VARCHAR,
            state       VARCHAR,
            resolution  VARCHAR,
            manifest_version VARCHAR,
            PRIMARY KEY (set_id, position),
            CHECK (
                (state = 'REQUIRED_UNSET' AND value_json IS NULL AND source IS NULL
                    AND resolution IS NULL AND manifest_version IS NULL)
                OR
                (state IS NULL AND value_json IS NOT NULL AND source IS NOT NULL
                    AND length(trim(source)) > 0 AND (
                        (resolution IS NULL AND manifest_version IS NULL)
                        OR (resolution = 'EXPLICIT' AND manifest_version IS NULL)
                        OR (resolution = 'DEFAULTED' AND manifest_version IS NOT NULL
                            AND length(trim(manifest_version)) > 0)
                    ))
            )
        );
        CREATE INDEX IF NOT EXISTS idx_run_parameters_state ON run_parameters(state);

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
            depth_datum VARCHAR,
            color       VARCHAR,
            PRIMARY KEY (well_id, top_name)
        );
        -- Existing tops predate source-reference custody. Keep them NULL rather than silently
        -- inventing MD; every new writer supplies its actual reference.
        ALTER TABLE tops ADD COLUMN IF NOT EXISTS depth_datum VARCHAR;

        -- Depth intervals per well (zoned interval sets). Modules
        -- resolve their interval parameters per zone at run time.
        CREATE TABLE IF NOT EXISTS zones (
            well_id      UUID NOT NULL,
            zone_name    VARCHAR NOT NULL,
            top_depth    FLOAT NOT NULL,
            bottom_depth FLOAT NOT NULL,
            depth_datum   VARCHAR,
            PRIMARY KEY (well_id, zone_name)
        );
        -- Legacy zones stay NULL until an operator/source declares their datum. Assigning MD here
        -- would turn an absent reference into invented provenance.
        ALTER TABLE zones ADD COLUMN IF NOT EXISTS depth_datum VARCHAR;

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
        --
        -- `compartment` names the fault block or segment the contact belongs to, and it is the
        -- other half of what makes a contact identifiable. Two compartments of one field routinely
        -- sit on different contacts because they are not in pressure communication -- so pooling
        -- them into one plane fit produces a surface neither is on and then flags every well as
        -- disagreeing with it, which is the opposite of what a QC is for. NULL means not stated,
        -- which stays a real answer: an undivided field has no compartments to name.
        CREATE TABLE IF NOT EXISTS fluid_contacts (
            contact_id   VARCHAR NOT NULL,
            field_name   VARCHAR,           -- field scope (NULL when well-scoped or global)
            well_id      VARCHAR,           -- well scope (NULL when field-scoped or global)
            contact_type VARCHAR NOT NULL,  -- OWC | GWC | GOC | GDT | ODT | FWL | custom
            depth        DOUBLE NOT NULL,
            is_tvdss     BOOLEAN NOT NULL,  -- true = depth is TVDSS (flat across wells), false = MD
            depth_datum  VARCHAR NOT NULL DEFAULT 'MD',
            color        VARCHAR,
            label        VARCHAR,
            compartment  VARCHAR,           -- named fault block / segment; NULL = not stated
            PRIMARY KEY (contact_id)
        );
        ALTER TABLE fluid_contacts ADD COLUMN IF NOT EXISTS depth_datum VARCHAR;
        UPDATE fluid_contacts
           SET depth_datum = CASE WHEN is_tvdss THEN 'TVDSS' ELSE 'MD' END
         WHERE depth_datum IS NULL;

        -- Which MARKERS a contact governs. A link table rather than a column on the contact,
        -- because the relationship is genuinely many-to-one in BOTH the ways a field is built:
        -- two stacked sands can each have their own contact, and several stacked sands in one
        -- hydraulic unit can share ONE contact. A single column can say the first and not the
        -- second, and a comma-separated list in a column is not a list, it is a bug waiting.
        --
        -- No rows for a contact = no marker stated, which stays a real answer: a field-wide datum
        -- cuts across markers, and that is the whole reason the plane fit exists.
        CREATE TABLE IF NOT EXISTS contact_zones (
            contact_id VARCHAR NOT NULL,
            zone_name  VARCHAR NOT NULL,
            PRIMARY KEY (contact_id, zone_name)
        );

        -- Core plug measurements (routine core analysis), sparse/irregular depths that do
        -- NOT align with the standard_curves depth grid — kept in its own table rather
        -- than computed_curves so overlay panels can fetch it at its own resolution.
        -- `set_name` versions the delivery (T-IMP-08): a well can hold RCAL, a SCAL plug
        -- set and a corrected re-delivery side by side, and an import NEVER overwrites an
        -- earlier one (names auto-suffix, as curve sets do). Unlike curve sets, core sets
        -- do NOT union: two deliveries measure the SAME plugs, so exactly one set is
        -- ACTIVE per well and every reader sees only that one (`core_sets.active`).
        -- `depth` is where the rock IS (after registration); `depth_orig` is where the lab
        -- wrote it. Keeping both is what lets a later delivery follow: an XRD table arrives
        -- months afterwards at the SAME depths the core report used, and without the original
        -- there is no way to know how far those depths have since moved. It also means a
        -- registration is never lost — depth_orig is written once at import and never shifted.
        -- MUST stay the LAST column: the Appender is positional, and a migrated database gets
        -- it appended, so fresh and migrated schemas have to agree.
        CREATE TABLE IF NOT EXISTS core_data (
            well_id     UUID NOT NULL,
            set_name    VARCHAR NOT NULL DEFAULT 'RAW',
            depth       FLOAT NOT NULL,
            cpor        FLOAT, -- core porosity, v/v
            cperm       FLOAT, -- core permeability, mD
            cgd         FLOAT, -- core grain density, g/cc
            csw         FLOAT, -- core water saturation, v/v
            depth_orig  FLOAT, -- as delivered; NULL only in a project older than this column
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

        -- Why a core sits where it does. One row per range moved, written in the SAME
        -- transaction as the move (see `shift_core_depths` / `apply_core_run_shifts`), so a
        -- shift can never commit without its reason.
        --
        -- This is an EVENT LOG, not a state table: an undo appends its own reversal rather
        -- than deleting the row it reverses. Deleting would make the record agree with the
        -- current depths at the cost of the only question it exists to answer — a core that
        -- was registered, judged wrong and put back is not the same as a core nobody ever
        -- touched, and the second reading is what a re-run would otherwise conclude.
        --
        -- `top`/`base` are NULL for a whole-core shift, which is a statement rather than a
        -- missing field: no range was declared, the correction applied everywhere.
        --
        -- `seq` counts within (well, set) rather than keying on the timestamp, because two
        -- applies can land in the same microsecond and a primary-key collision there would
        -- fail the SHIFT, not just its record.
        CREATE TABLE IF NOT EXISTS core_registrations (
            well_id     UUID NOT NULL,
            set_name    VARCHAR NOT NULL,   -- the delivery that moved, as it was then
            seq         INTEGER NOT NULL,
            applied_at  TIMESTAMP NOT NULL DEFAULT now(),
            kind        VARCHAR NOT NULL,   -- 'proposed' | 'manual' | 'undo'
            top         FLOAT,              -- NULL = the whole core
            base        FLOAT,
            delta       FLOAT NOT NULL,
            log_curve   VARCHAR,            -- what it was matched against
            reference   VARCHAR,            -- the core measurement used
            pairing     VARCHAR,            -- 'like-for-like' | 'proxy (direct)' | 'proxy (inverse)'
            correlation FLOAT,              -- agreement AT THE APPLIED SHIFT, not at the peak
            n_pairs     INTEGER,
            note        VARCHAR,
            PRIMARY KEY (well_id, set_name, seq)
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
            -- 1 = this delivery sits on the CORE's depth scale, so a later core registration
            -- must carry it along. Recorded at import from the user's own declaration; without
            -- it the app could not tell a core-depth delivery from a log-depth one, and moving
            -- the wrong one is silent.
            on_core_depths INTEGER NOT NULL DEFAULT 0,
            -- Aux deliveries are the shipped POINT store: rows are independent observations,
            -- not a sampled continuous frame. The writer declares how same-depth observations
            -- were kept, rather than leaving the PK-less store's behavior implicit.
            sampling_style VARCHAR NOT NULL,
            duplicate_resolution VARCHAR NOT NULL,
            perturbation_value DOUBLE,
            perturbation_unit VARCHAR,
            PRIMARY KEY (well_id, dataset, set_name)
        );

        CREATE TABLE IF NOT EXISTS aux_duplicate_depth_resolutions (
            well_id     UUID NOT NULL,
            dataset     VARCHAR NOT NULL,
            set_name    VARCHAR NOT NULL,
            item        VARCHAR NOT NULL,
            source_row  INTEGER NOT NULL,
            original_depth FLOAT NOT NULL,
            stored_depth FLOAT NOT NULL,
            resolution  VARCHAR NOT NULL,
            perturbation_value DOUBLE,
            perturbation_unit VARCHAR,
            PRIMARY KEY (well_id, dataset, set_name, item, source_row)
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
            -- How big the rock is, and how the section was prepared. Both are DECLARED and both
            -- default to absent, because a delivery holds plates of both kinds (Jauhar,
            -- 2026-07-31: "sometimes yes, sometimes not" for the scale, "sometimes stained and
            -- epoxy, sometimes not" for the preparation).
            --
            -- `fov_um` is the WIDTH OF THE WHOLE PICTURE in micrometres, not a um/px ratio,
            -- because the stored copy is resampled to a long-edge cap: a ratio would silently
            -- belong to whichever copy it was measured on, while a field of view survives any
            -- resampling. um/px of any copy is fov_um / that copy's pixel width. NULL means no
            -- scale was declared, and nothing dimensional may run on such a plate — reporting
            -- pixels under a micron label is the same class of error as a wrong `m`.
            --
            -- `prepared` and `stain` are INDEPENDENT: a section can be impregnated, stained,
            -- both or neither. NULL/'' on `prepared` is UNKNOWN, not "plain" — a blue-epoxy
            -- pore rule run over an unimpregnated section returns a porosity built from
            -- blue-ish feldspar and edge artefact, so unknown must be refused rather than
            -- assumed either way. The stain protocol comes from the laboratory report, so it is
            -- free text rather than a vocabulary invented here.
            fov_um      FLOAT,
            prepared    VARCHAR,   -- '' / NULL = unknown | 'blue_epoxy' | 'plain'
            stain       VARCHAR,   -- as the lab report names it; NULL = none or not stated
            -- Core-photograph conditioning (crop, deskew, white balance, tone). NON-DESTRUCTIVE,
            -- and the two columns together are what makes that true rather than a claim.
            --
            -- `recipe` is the settings as JSON; NULL/'' means the picture is exactly as imported.
            -- `source_data` holds the UN-conditioned display copy and is written once, the first
            -- time a recipe is baked. Everything afterwards is rendered FROM it, so a recipe can
            -- be edited any number of times without conditioning an already-conditioned picture,
            -- and clearing it restores the import byte for byte.
            --
            -- The conditioned pixels go into `data` — they are BAKED rather than applied at
            -- render time, and that is not laziness. Every reader downstream already takes `data`
            -- as the picture, and the PDF exporter embeds those bytes UNTOUCHED through a
            -- /DCTDecode XObject: a render-time recipe would leave the print showing the
            -- unconditioned photograph while the screen showed the corrected one, silently.
            -- Baking also means the log view, the composite and the PDF cannot disagree, because
            -- there is nothing left for them to disagree about.
            --
            -- `source_meta` carries the kept copy's own `WxH;mime`. Without it a restore would
            -- leave `width`/`height` describing the cropped picture while `data` held the whole
            -- one, and every renderer would draw the plate at the wrong aspect ratio — the one
            -- thing this app never does to a photograph.
            recipe      VARCHAR,
            source_data BLOB,
            source_meta VARCHAR,
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
            on_core_depths INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (well_id, dataset, set_name)
        );

        -- Trained machine-learning models, kept as ARTIFACTS rather than dying with the run.
        --
        -- Until now a run carried its training wells and its apply wells in one call, so the
        -- fitted model was thrown away: there was no way to train on the cored wells and apply
        -- THAT SAME model to the rest of the field later. A refit on different data is a
        -- different model, and "which model produced this PERM curve?" had no answer.
        --
        -- `data` is a joblib dump of BOTH the scaler and the estimator. The scaler must travel
        -- with the model — refitting a StandardScaler on the apply wells would be a different
        -- transform, and the predictions would be quietly wrong rather than obviously broken.
        -- `feature_curves` is an ORDERED JSON array and is the contract: applying the model
        -- resolves exactly those curves in exactly that order, and fails a well by name when one
        -- is missing rather than substituting or reordering.
        --
        -- PRIMARY KEY here is the `well_images` argument, not a `computed_curves` inconsistency:
        -- one index entry per MODEL is free, and a duplicate would make a cited model ambiguous.
        CREATE TABLE IF NOT EXISTS ml_models (
            model_id        UUID NOT NULL,
            name            VARCHAR NOT NULL,
            task            VARCHAR NOT NULL,
            algorithm       VARCHAR NOT NULL,
            feature_curves  VARCHAR NOT NULL,   -- JSON array, ORDER IS PART OF THE CONTRACT
            target_curve    VARCHAR,
            params_json     VARCHAR NOT NULL,
            metrics_json    VARCHAR NOT NULL,
            trained_on      VARCHAR NOT NULL,   -- JSON array of well names (provenance)
            n_train         INTEGER NOT NULL,
            standardize     INTEGER NOT NULL,
            sklearn_version VARCHAR,
            note            VARCHAR,
            created_at      TIMESTAMP NOT NULL DEFAULT now(),
            data            BLOB NOT NULL,
            -- SB-MLA-003. A fingerprint of the exact training matrix: feature names in order, the
            -- feature and target values, and the row order. `trained_on` + `n_train` narrows a
            -- re-run but does not pin it — the same wells at a later log-set version are different
            -- rows with the same names and possibly the same count. NULL on a model saved before
            -- this existed, which is an honest "not recorded" rather than a hash that means nothing.
            train_hash      VARCHAR,
            -- SB-MLA-002 + SB-MLA-004. The per-well training roster: for each well, what it
            -- contributed, the log set its rows were READ FROM (name, id, version), and how many of
            -- its samples the run mask removed. `trained_on` answers "which wells"; this answers
            -- "which rock", which is the question a re-run has to match. JSON array.
            training_json   VARCHAR,
            -- SB-MLA-005. The interpreter and every library that participated in fitting or
            -- serialising the artifact — the blob is a pickle, so it is loadable only under a
            -- compatible set, and joblib is the serialiser, not a bystander. JSON object.
            runtime_json    VARCHAR,
            PRIMARY KEY (model_id)
        );
        -- Added via ALTER so projects written before 2026-08-07 converge on the same shape.
        ALTER TABLE ml_models ADD COLUMN IF NOT EXISTS train_hash VARCHAR;
        ALTER TABLE ml_models ADD COLUMN IF NOT EXISTS training_json VARCHAR;
        ALTER TABLE ml_models ADD COLUMN IF NOT EXISTS runtime_json VARCHAR;

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
            on_core_depths INTEGER NOT NULL DEFAULT 0,
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

        CREATE SEQUENCE IF NOT EXISTS curve_meta_modified_seq START 1;

        -- Phase 6: generic curve store. Unlike `standard_curves` (fixed 6 mnemonics),
        -- this holds ANY curve at ANY name, in one of several named sets. Set labels describe
        -- custody; the separate curve-level Final flag records a resolving decision. `curve_meta` is the
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
            set_version INTEGER NOT NULL DEFAULT 1,
            final_flag  INTEGER NOT NULL DEFAULT 0,
            modified_seq BIGINT NOT NULL DEFAULT nextval('curve_meta_modified_seq'),
            UNIQUE (well_id, set_name, mnemonic, run_no)
        );
        -- `pinned` added via ALTER so existing project databases converge on the same shape.
        ALTER TABLE curve_meta ADD COLUMN IF NOT EXISTS pinned INTEGER DEFAULT 0;
        -- SB-DBM-017 / DEC-025: the neutron matrix basis is DECLARED curve metadata, never
        -- inferred from contractor, tool, salinity or a matrix default. NULL is the honest
        -- absence - a limestone-unit neutron read against a sandstone matrix is ~0.04 v/v low
        -- in clean water sand, and nothing can refuse a wrong basis that was never recorded.
        ALTER TABLE curve_meta ADD COLUMN IF NOT EXISTS neutron_basis VARCHAR;
        ALTER TABLE curve_meta ADD COLUMN IF NOT EXISTS neutron_basis_source VARCHAR;
        ALTER TABLE curve_meta ADD COLUMN IF NOT EXISTS set_version INTEGER DEFAULT 1;
        UPDATE curve_meta SET set_version = 1 WHERE set_version IS NULL;
        ALTER TABLE curve_meta ADD COLUMN IF NOT EXISTS final_flag INTEGER;
        -- A historical set name is not evidence that one particular curve identity was selected
        -- from duplicate runs in that set. Existing rows therefore start unflagged rather than
        -- receiving an invented Final decision during migration.
        UPDATE curve_meta SET final_flag = 0 WHERE final_flag IS NULL;
        ALTER TABLE curve_meta ALTER COLUMN final_flag SET DEFAULT 0;
        -- Existing rows predate a recoverable modification order and stay NULL. New or edited
        -- rows receive a monotonic revision so SB-DBM-006 can truthfully apply MRU; the resolver
        -- refuses an otherwise-tied legacy collision instead of inventing which old row was last.
        ALTER TABLE curve_meta ADD COLUMN IF NOT EXISTS modified_seq BIGINT;
        ALTER TABLE curve_meta ALTER COLUMN modified_seq SET DEFAULT nextval('curve_meta_modified_seq');
        -- SB-DIO-007 (signed DRAFT_DIO007 under DEC-076): the versioned source-cell-state mask,
        -- one byte per sample in ASCENDING DEPTH order behind a one-byte version prefix
        -- (0 = measured, 1 = empty cell, 2 = explicitly nulled). Both absent states store
        -- f32::NAN in the samples - rule 2 is untouched by construction; the mask is consulted
        -- only by exporters and custody surfaces, never by arithmetic. NULL = a pre-contract
        -- import whose cell states cannot be recovered: preserved as unknown, never backfilled.
        -- Added LAST via ALTER (the additive-column precedent above).
        ALTER TABLE curve_meta ADD COLUMN IF NOT EXISTS state_mask BLOB;

        -- SB-MLA-055. A row here DECLARES that a curve's values are class identifiers — a facies
        -- code, a litho code, a predicted class — and not a quantity. Averaging or interpolating
        -- one produces a number that is not any class: the mean of facies 1 and facies 4 is 2.5,
        -- which plots, exports and reads back as a facies that was never in the scheme.
        --
        -- Its own table rather than a column on `curve_meta`, because `curve_meta` covers only the
        -- generic IMPORT store and the curves that need this most are module outputs, which live in
        -- `computed_curves` and have no metadata row anywhere. Keyed by NAME for the same reason —
        -- that is the only identifier `computed_curves` carries.
        --
        -- DECLARED, not detected. `reframe::looks_discrete` still guesses from the values so a
        -- LITH curve off a LAS is handled sensibly, but a guess may only pick the DEFAULT method:
        -- a caliper that happens to read whole inches must stay averageable when the user says so.
        -- A declaration is the producer's statement about what the numbers MEAN, so it overrides.
        -- SB-MLA-035. The unit a COMPUTED curve is in, declared by whatever wrote it.
        --
        -- `list_curve_catalog` could only ever get a unit for a computed curve by joining
        -- `equations.output_units`, so a curve written by a module or by an ML run had no unit
        -- anywhere in the product. That absence is what makes the log-transform trap possible: a
        -- permeability predicted in log10 space and a permeability in mD are different quantities,
        -- and with no unit to disagree with, a mean of -0.4 reads as a permeability rather than as
        -- 0.398 mD in log units.
        --
        -- Per WELL like `curve_class`, and for the same reason: the same mnemonic can be produced
        -- by different runs on different wells, and a project-wide row would have to pick one.
        CREATE TABLE IF NOT EXISTS curve_unit (
            well_id     UUID NOT NULL,
            curve_name  VARCHAR NOT NULL,
            unit        VARCHAR,
            PRIMARY KEY (well_id, curve_name)
        );

        CREATE TABLE IF NOT EXISTS curve_class (
            well_id     UUID NOT NULL,
            curve_name  VARCHAR NOT NULL,
            source      VARCHAR,   -- what declared it, e.g. 'electrofacies', 'ml:classification'
            PRIMARY KEY (well_id, curve_name)
        );

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

        -- One declaration and one verified verdict per imported continuous curve set. Legacy
        -- curve_meta rows intentionally have no matching row: a frame-indexed reader must refuse
        -- them rather than infer regularity from coincidentally regular samples.
        CREATE TABLE IF NOT EXISTS import_sets (
            well_id                     UUID NOT NULL,
            set_name                    VARCHAR NOT NULL,
            declared_sampling_style     VARCHAR NOT NULL,
            effective_sampling_style    VARCHAR NOT NULL,
            sampling_verified           BOOLEAN NOT NULL,
            verification_tolerance      DOUBLE,
            verification_tolerance_unit VARCHAR,
            verification_warning        VARCHAR,
            gap_depth                   FLOAT,
            gap_row_count               INTEGER,
            PRIMARY KEY (well_id, set_name)
        );

        -- SB-DBM-027. Integrity cleanup is a QUARANTINE, never an irreversible DELETE.
        -- The checker itself is read-only. An explicit prune moves only the bounded orphan
        -- classes named below into typed tables in one transaction; the batch remains in the
        -- project so Ctrl+Z (or a later reopen) can restore the exact rows without JSON-float
        -- round-tripping. Duplicate samples and unresolved ML provenance are report-only:
        -- choosing a survivor or deleting a trained artifact would be a product decision.
        CREATE TABLE IF NOT EXISTS integrity_prune_batches (
            batch_id       UUID PRIMARY KEY,
            state          VARCHAR NOT NULL,
            classes        VARCHAR NOT NULL,
            created_at     TIMESTAMP NOT NULL DEFAULT now(),
            changed_at     TIMESTAMP NOT NULL DEFAULT now(),
            CHECK (state IN ('ACTIVE', 'RESTORED'))
        );
        CREATE TABLE IF NOT EXISTS integrity_quarantine_computed (
            batch_id       UUID NOT NULL,
            source_table   VARCHAR NOT NULL,
            set_id         UUID,
            well_id        UUID NOT NULL,
            depth          FLOAT NOT NULL,
            curve_name     VARCHAR NOT NULL,
            value          FLOAT,
            CHECK (source_table IN ('computed_curves', 'computed_curves_archive')),
            FOREIGN KEY (batch_id) REFERENCES integrity_prune_batches(batch_id)
        );
        CREATE TABLE IF NOT EXISTS integrity_quarantine_group_members (
            batch_id       UUID NOT NULL,
            group_id       UUID NOT NULL,
            well_id        UUID NOT NULL,
            PRIMARY KEY (batch_id, group_id, well_id),
            FOREIGN KEY (batch_id) REFERENCES integrity_prune_batches(batch_id)
        );
        CREATE TABLE IF NOT EXISTS integrity_quarantine_curve_samples (
            batch_id       UUID NOT NULL,
            curve_id       UUID NOT NULL,
            depth          FLOAT NOT NULL,
            value          FLOAT,
            PRIMARY KEY (batch_id, curve_id, depth),
            FOREIGN KEY (batch_id) REFERENCES integrity_prune_batches(batch_id)
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
        standard_table_ddl = standard.table_ddl,
        standard_frame = crate::schema_vocab::LogSetFrame::Standard.as_str(),
    );
    conn.execute_batch(&schema)?;
    conn.execute_batch(&standard.migration_ddl)?;
    // Additive migration for projects whose log_sets rows predate declared sampling style. NULL is
    // intentional for those historical rows: neither regularity nor point semantics can be
    // reconstructed merely by looking at their stored depths.
    conn.execute_batch(&format!(
        "ALTER TABLE log_sets ADD COLUMN IF NOT EXISTS sampling_style VARCHAR;
         -- SB-DBM-031 (DEC-073 item 5): a delivery declares its depth datum ONCE, on its
         -- SET row - one delivery is one datum, and a per-row column would break the
         -- positional-Appender contracts. NULL = legacy unknown, PRESERVED as unknown:
         -- backfilling MD would be exactly the inference the ruling forbids.
         ALTER TABLE core_sets  ADD COLUMN IF NOT EXISTS depth_datum VARCHAR;
         ALTER TABLE aux_sets   ADD COLUMN IF NOT EXISTS depth_datum VARCHAR;
         ALTER TABLE scal_sets  ADD COLUMN IF NOT EXISTS depth_datum VARCHAR;
         ALTER TABLE image_sets ADD COLUMN IF NOT EXISTS depth_datum VARCHAR;
         ALTER TABLE log_sets ADD COLUMN IF NOT EXISTS duplicate_resolution VARCHAR;
         ALTER TABLE log_sets ADD COLUMN IF NOT EXISTS outcome_state VARCHAR;
         ALTER TABLE log_sets ADD COLUMN IF NOT EXISTS comment VARCHAR;
         ALTER TABLE aux_sets ADD COLUMN IF NOT EXISTS sampling_style VARCHAR;
         ALTER TABLE aux_sets ADD COLUMN IF NOT EXISTS duplicate_resolution VARCHAR;
         ALTER TABLE aux_sets ADD COLUMN IF NOT EXISTS perturbation_value DOUBLE;
         ALTER TABLE aux_sets ADD COLUMN IF NOT EXISTS perturbation_unit VARCHAR;
         -- SB-ENV-005: the applied-step manifest column, added LAST (positional rule). NULL on
         -- migrated rows is the pre-contract state, preserved as unknown - never backfilled.
         ALTER TABLE log_sets ADD COLUMN IF NOT EXISTS applied_steps_json VARCHAR;
         UPDATE aux_sets SET sampling_style = '{}' WHERE sampling_style IS NULL;
         UPDATE aux_sets SET duplicate_resolution = '{}'
             WHERE duplicate_resolution IS NULL;",
        crate::schema_vocab::SamplingStyle::Point.as_str(),
        crate::schema_vocab::DuplicateDepthResolution::Preserve.as_str()
    ))?;
    // Projects first opened by SB-DBM-003 already own the indexed parameter relation. Preserve
    // those rows and extend it additively; historical records remain unclassified rather than
    // being relabelled as explicit/defaulted without evidence.
    conn.execute_batch(
        "ALTER TABLE run_parameters ADD COLUMN IF NOT EXISTS resolution VARCHAR;
         ALTER TABLE run_parameters ADD COLUMN IF NOT EXISTS manifest_version VARCHAR;",
    )?;
    backfill_run_parameters(conn)?;
    Ok(())
}

/// Builds SB-DBM-003's indexed parameter view for complete ancestry written before the relation
/// existed. The migration is deliberately evidence-preserving: it accepts only a sourced value
/// or the exact historical ABSENT/ABSENT pair. Malformed legacy JSON is left unclassified instead
/// of being repaired with an invented value, source, or state.
fn backfill_run_parameters(conn: &Connection) -> DbResult<()> {
    #[derive(Debug)]
    struct BackfillRow {
        set_id: String,
        position: i64,
        name: String,
        value_json: Option<String>,
        source: Option<String>,
        state: Option<String>,
        resolution: Option<String>,
        manifest_version: Option<String>,
    }

    let candidates: Vec<(String, String)> = {
        let mut statement = conn.prepare(
            "SELECT CAST(log_sets.set_id AS VARCHAR), log_sets.params_json
             FROM log_sets
             WHERE log_sets.params_json IS NOT NULL
               AND NOT EXISTS (
                   SELECT 1 FROM run_parameters
                   WHERE run_parameters.set_id = log_sets.set_id
               )
             ORDER BY log_sets.set_id",
        )?;
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<duckdb::Result<_>>()?
    };

    let mut rows = Vec::new();
    for (set_id, params_json) in candidates {
        let Ok(payload) = serde_json::from_str::<serde_json::Value>(&params_json) else {
            continue;
        };
        let Some(parameters) = payload
            .get(crate::ancestry::CURVE_ANCESTRY_KEY)
            .and_then(|ancestry| ancestry.get("parameters"))
            .and_then(serde_json::Value::as_array)
        else {
            continue;
        };

        let mut set_rows = Vec::with_capacity(parameters.len());
        let mut complete = true;
        for (position, parameter) in parameters.iter().enumerate() {
            let Some(name) = parameter
                .get("name")
                .and_then(serde_json::Value::as_str)
                .filter(|name| !name.trim().is_empty())
            else {
                complete = false;
                break;
            };
            let value = parameter.get("value");
            let source = parameter.get("source");
            let state = parameter.get("state").and_then(serde_json::Value::as_str);
            let resolution = parameter
                .get("resolution")
                .and_then(serde_json::Value::as_str);
            let manifest_version = parameter
                .get("manifest_version")
                .and_then(serde_json::Value::as_str);
            let historical_required_unset = state.is_none()
                && value.and_then(serde_json::Value::as_str)
                    == Some(crate::modules::ABSENT_DEFAULT_SOURCE)
                && source.and_then(serde_json::Value::as_str)
                    == Some(crate::modules::ABSENT_DEFAULT_SOURCE)
                && resolution.is_none()
                && manifest_version.is_none();
            let canonical_required_unset =
                state == Some(crate::ancestry::REQUIRED_UNSET_PARAMETER_STATE)
                    && value.is_some_and(serde_json::Value::is_null)
                    && source.is_some_and(serde_json::Value::is_null)
                    && resolution.is_none()
                    && manifest_version.is_none();

            let row = if historical_required_unset || canonical_required_unset {
                BackfillRow {
                    set_id: set_id.clone(),
                    position: position as i64,
                    name: name.to_string(),
                    value_json: None,
                    source: None,
                    state: Some(crate::ancestry::REQUIRED_UNSET_PARAMETER_STATE.to_string()),
                    resolution: None,
                    manifest_version: None,
                }
            } else if state.is_none() {
                let Some(value) = value.filter(|value| !value.is_null()) else {
                    complete = false;
                    break;
                };
                let Some(source) = source
                    .and_then(serde_json::Value::as_str)
                    .filter(|source| !source.trim().is_empty())
                else {
                    complete = false;
                    break;
                };
                let legal_resolution = match (resolution, manifest_version) {
                    (None, None) | (Some("EXPLICIT"), None) => true,
                    (Some("DEFAULTED"), Some(version)) if !version.trim().is_empty() => true,
                    _ => false,
                };
                if !legal_resolution {
                    complete = false;
                    break;
                }
                BackfillRow {
                    set_id: set_id.clone(),
                    position: position as i64,
                    name: name.to_string(),
                    value_json: Some(value.to_string()),
                    source: Some(source.to_string()),
                    state: None,
                    resolution: resolution.map(str::to_string),
                    manifest_version: manifest_version.map(str::to_string),
                }
            } else {
                complete = false;
                break;
            };
            set_rows.push(row);
        }
        if complete {
            rows.extend(set_rows);
        }
    }

    if rows.is_empty() {
        return Ok(());
    }
    with_txn(conn, |conn| {
        for row in rows {
            conn.execute(
                "INSERT INTO run_parameters
                    (set_id, position, name, value_json, source, state, resolution, manifest_version)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    row.set_id,
                    row.position,
                    row.name,
                    row.value_json,
                    row.source,
                    row.state,
                    row.resolution,
                    row.manifest_version
                ],
            )?;
        }
        Ok::<(), DbError>(())
    })
}

/// Migrates once, on open: copies every `standard_curves` column into the generic
/// `curve_meta`/`curve_samples` store as set 'RAW', so Phase 6 code (units, TVD-aware
/// resampling, curve catalog) has real data without disturbing anything that still reads
/// `standard_curves` directly. Idempotent — checks `curve_meta` for any row with
/// source = 'standard_curves migration' before doing any work, so it runs at most once
/// per well per column even if called on every launch.
pub fn migrate_standard_curves_to_generic_store(conn: &Connection) -> DbResult<()> {
    // Only wells not yet fully backfilled. Once a well is in curve_migration_done it is skipped
    // entirely, so this whole function is ~instant on an already-migrated project instead of
    // re-scanning standard_curves for every well's absent columns on each launch.
    let mut stmt = conn
        .prepare("SELECT well_id FROM wells WHERE well_id NOT IN (SELECT well_id FROM curve_migration_done)")?;
    let well_ids: Vec<String> = stmt.query_map([], |r| r.get::<_, String>(0))?.filter_map(|r| r.ok()).collect();
    drop(stmt);

    for well_id in well_ids {
        for column in crate::schema_vocab::STANDARD_COLUMNS
            .iter()
            .filter(|column| column.editable)
        {
            let col = column.storage_column;
            let mnemonic = column.mnemonic;
            let family = crate::curves::family_for(mnemonic).ok_or_else(|| {
                DbError::Invalid(format!(
                    "registered standard column '{mnemonic}' has no curve-family definition"
                ))
            })?;
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
            conn.execute(
                "INSERT INTO curve_meta
                    (curve_id, well_id, set_name, mnemonic, unit, family, source, run_no,
                     set_version, final_flag, modified_seq)
                 VALUES (?1, ?2, 'RAW', ?3, ?4, ?5, 'standard_curves migration', NULL,
                         1, 0, nextval('curve_meta_modified_seq'))",
                params![
                    curve_id,
                    well_id,
                    mnemonic,
                    family.canonical_unit,
                    family.family
                ],
            )?;
            // SB-DBM-030: a data-bearing column migrates its FULL frame - a NULL sample is a
            // measurement that is absent at that depth, and dropping the row would conflate
            // "logged but missing" with "never sampled". The has_data gate above already skips
            // columns that were never supplied at all.
            conn.execute(
                &format!(
                    "INSERT INTO curve_samples (curve_id, depth, value)
                     SELECT ?1, depth, {col} FROM standard_curves WHERE well_id = ?2"
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

/// Records that a newly imported well already has its standard projection represented in the
/// native generic store. A current LAS import writes both views from the same decoded columns in
/// one outer transaction, so running the legacy backfill on the next open would create duplicate
/// RAW identities with fresh UUIDs and make otherwise identical project copies cite different
/// input ancestry. This marker belongs in that same import transaction: a failed native write must
/// not suppress the legacy repair on the next open.
pub(crate) fn mark_standard_curve_migration_done(
    conn: &Connection,
    well_id: &str,
) -> DbResult<()> {
    conn.execute(
        "INSERT INTO curve_migration_done (well_id) VALUES (?1) ON CONFLICT DO NOTHING",
        params![well_id],
    )?;
    Ok(())
}

/// RELEASE.md §3.2 (requirement R-B): before a migration that rewrites or drops data, copy
/// the project file beside itself as `<name>.pre-<SOURCE_FORMAT>-backup.duckdb`. Purely
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
    let source_format = migration_source_format(conn)?;
    let mut backup = format!("{stem}.pre-{source_format}-backup.duckdb");
    if std::path::Path::new(&backup).exists() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        backup = format!("{stem}.pre-{source_format}-backup-{ts}.duckdb");
        let mut collision = 1_u64;
        while std::path::Path::new(&backup).exists() {
            backup = format!("{stem}.pre-{source_format}-backup-{ts}-{collision}.duckdb");
            collision += 1;
        }
    }
    engine_copy_to(conn, &backup)?;
    Ok(backup)
}

const TVDSS_CONVENTION_KEY: &str = "tvdss_sign_convention";
const TVDSS_POSITIVE_DOWN: &str = "F17_POSITIVE_DOWN_V1";

/// Converts the pre-SB-DBM-031 TVDSS stores from elevation-minus-TVD to F-17 positive-down
/// TVD-minus-elevation. The project-meta marker makes the rewrite idempotent.
///
/// This is a DESTRUCTIVE migration (RELEASE §3.2): a real project is copied before the first
/// affected row is changed, and a failed backup ABORTS rather than proceeding — an un-migrated
/// project still opens (TVDSS simply keeps its old sign), so refusing costs nothing, while
/// rewriting every depth after the promised copy failed breaks the exact guarantee the backup
/// exists to make. `path: None` is for in-memory test databases only.
///
/// Only stores whose existing schema already declares TVDSS are rewritten: `well_path.tvdss`,
/// explicitly TVDSS contacts, and the system's materialized current/archive TVDSS curves. Generic
/// imported curves are deliberately not selected by mnemonic here because their source sign was
/// never declared; treating a name as a reference-frame declaration would invent provenance.
pub fn migrate_tvdss_positive_down(conn: &Connection, path: Option<&str>) -> DbResult<()> {
    // `init_db` creates this before calling us, but focused migrations and recovery tools
    // may operate on a schema-only connection. The marker table is additive; creating it
    // here keeps the destructive sign rewrite independently safe and idempotent.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS project_meta (key VARCHAR PRIMARY KEY, value VARCHAR);",
    )?;
    let convention: Option<String> = conn
        .query_row(
            "SELECT value FROM project_meta WHERE key = ?1",
            params![TVDSS_CONVENTION_KEY],
            |row| row.get(0),
        )
        .optional()?;
    match convention.as_deref() {
        Some(TVDSS_POSITIVE_DOWN) => return Ok(()),
        Some(other) => {
            return Err(DbError::Invalid(format!(
                "unsupported TVDSS sign convention '{other}'; expected {TVDSS_POSITIVE_DOWN}"
            )));
        }
        None => {}
    }

    let affected: i64 = conn.query_row(
        "SELECT
             (SELECT COUNT(*) FROM well_path WHERE tvdss IS NOT NULL)
           + (SELECT COUNT(*) FROM fluid_contacts WHERE depth_datum = 'TVDSS')
           + (SELECT COUNT(*) FROM computed_curves
                WHERE upper(curve_name) = 'TVDSS' AND value IS NOT NULL)
           + (SELECT COUNT(*) FROM computed_curves_archive
                WHERE upper(curve_name) = 'TVDSS' AND value IS NOT NULL)",
        [],
        |row| row.get(0),
    )?;
    let backup = if affected > 0 {
        match path {
            Some(path) => Some(backup_before_destructive_migration(conn, path)?),
            None => None,
        }
    } else {
        None
    };

    with_txn(conn, |conn| -> DbResult<()> {
        conn.execute("UPDATE well_path SET tvdss = -tvdss WHERE tvdss IS NOT NULL", [])?;
        conn.execute(
            "UPDATE fluid_contacts SET depth = -depth WHERE depth_datum = 'TVDSS'",
            [],
        )?;
        conn.execute(
            "UPDATE computed_curves SET value = -value
             WHERE upper(curve_name) = 'TVDSS' AND value IS NOT NULL",
            [],
        )?;
        conn.execute(
            "UPDATE computed_curves_archive SET value = -value
             WHERE upper(curve_name) = 'TVDSS' AND value IS NOT NULL",
            [],
        )?;
        conn.execute(
            "INSERT INTO project_meta (key, value) VALUES (?1, ?2)",
            params![TVDSS_CONVENTION_KEY, TVDSS_POSITIVE_DOWN],
        )?;
        Ok(())
    })?;

    if let Some(backup) = backup {
        boot_note(format!(
            "One-time TVDSS sign upgrade ({affected} stored value(s)): project backed up first to {backup}"
        ));
    }
    Ok(())
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

/// SB-CLY-001 (DEC-036): `run_degradations.kind` gained the documented fifth member
/// `ENDPOINT_INVALID` - the zone-bearing run message rides the degradation channel. A
/// pre-existing project carries the four-member CHECK, which would refuse the row at
/// persist time, so the table is rebuilt in place with the extended CHECK and every row
/// copied verbatim. Idempotent: a table whose CHECK already names the member is left alone.
pub fn migrate_run_degradations_endpoint_invalid(conn: &Connection) -> DbResult<()> {
    let outdated: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duckdb_constraints()
         WHERE table_name = 'run_degradations' AND constraint_type = 'CHECK'
           AND constraint_text LIKE '%kind IN%'
           AND constraint_text NOT LIKE '%ENDPOINT_INVALID%'",
        [],
        |r| r.get(0),
    )?;
    if outdated == 0 {
        return Ok(());
    }
    conn.execute_batch(
        "BEGIN;
         CREATE TABLE run_degradations_v2 (
            set_id      UUID NOT NULL,
            position    INTEGER NOT NULL,
            module      VARCHAR NOT NULL,
            kind        VARCHAR NOT NULL CHECK (
                kind IN ('CLAMPED', 'DEFAULTED', 'TRUNCATED', 'SUBSTITUTED_INPUT', 'ENDPOINT_INVALID')
            ),
            detail      VARCHAR NOT NULL,
            occurrences BIGINT NOT NULL CHECK (occurrences > 0),
            PRIMARY KEY (set_id, position)
         );
         INSERT INTO run_degradations_v2
             SELECT set_id, position, module, kind, detail, occurrences FROM run_degradations;
         DROP TABLE run_degradations;
         ALTER TABLE run_degradations_v2 RENAME TO run_degradations;
         COMMIT;",
    )?;
    boot_note("One-time storage upgrade: run_degradations now accepts the ENDPOINT_INVALID kind (SB-CLY-001)".to_string());
    Ok(())
}

/// SB-DBM-009 / DEC-022: converts every pre-migration `log_sets.created_at` from WIB
/// (UTC+7) local wall time to a UTC instant, and re-points the column DEFAULT at UTC so new
/// rows can never reintroduce the local meaning. The zone is DECLARED, not measured: Jauhar
/// ruled (DEC-022, 2026-08-17) that every legacy record was written on a machine set to
/// Western Indonesia time, and the ruling itself is recorded as the converted values' SOURCE
/// in the marker document below, so a later reader sees the offset was declared by the
/// product owner rather than inferred from the data. Idempotent: the marker gates the
/// subtraction, because running it twice would move history by another seven hours.
pub fn migrate_log_set_timestamps_to_utc(conn: &Connection) -> DbResult<()> {
    let already: i64 = conn.query_row(
        "SELECT count(*) FROM documents WHERE doc_type = 'migration' AND name = 'DEC-022-created-at-utc'",
        [],
        |r| r.get(0),
    )?;
    if already > 0 {
        return Ok(());
    }
    let converted = conn.execute(
        "UPDATE log_sets SET created_at = created_at - INTERVAL 7 HOUR",
        [],
    )?;
    conn.execute_batch(
        "ALTER TABLE log_sets ALTER COLUMN created_at SET DEFAULT (now() AT TIME ZONE 'UTC')",
    )?;
    conn.execute(
        "INSERT INTO documents (doc_id, doc_type, name, json) VALUES (gen_random_uuid(), 'migration', 'DEC-022-created-at-utc', ?1)",
        params![format!(
            "{{\"declared_zone\":\"WIB (UTC+7)\",\"source\":\"DEC-022 (RULED 2026-08-17): every pre-migration log_sets.created_at was written on a machine set to Western Indonesia time; the offset is declared by the product owner, not measured from the data\",\"rows_converted\":{converted}}}"
        )],
    )?;
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
    migrate_drop_computed_curves_pk_with_backup(conn, path, backup_before_destructive_migration)
}

fn migrate_drop_computed_curves_pk_with_backup<F>(
    conn: &Connection,
    path: Option<&str>,
    backup: F,
) -> DbResult<()>
where
    F: FnOnce(&Connection, &str) -> DbResult<String>,
{
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
        let backup_path = backup(conn, path)?;
        boot_note(format!("One-time storage upgrade (write-speed index removal): project backed up first to {backup_path}"));
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

/// One-time migration dropping the Phase-0 per-depth parameters stub whose table name carried
/// a client study acronym — the provenance rule (no client identifier in the tree) applied to
/// the SCHEMA, where the name ships inside every project file that travels between operators.
///
/// Like the `array_logs` stub above, **no code path in any build ever read or wrote a single
/// row of it** (`git log -S` finds only the Phase-0 scaffold, and the SQL panel is read-only,
/// so no user could have written one either) — the table is empty in every project file in
/// existence, dropping it loses nothing, and no backup is taken. `IF EXISTS` is the whole
/// idempotency story: there is no new shape to detect or rebuild.
///
/// The table name below is deliberately the LAST occurrence of the acronym in the tree — it
/// exists only to remove itself from old project files, and stays for as long as pre-2026-08
/// projects may still be opened.
pub fn migrate_drop_study_named_stub(conn: &Connection) -> DbResult<()> {
    conn.execute_batch("DROP TABLE IF EXISTS lqr_parameters;")?;
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
/// The write discipline mirrors `ancestry::write_versioned_rows_raw`: DELETE the (well, set, curve)
/// rows first, then insert fresh ones — a re-run replaces its own output and never unions two
/// runs' realizations into one distribution. `depths` and `samples` must be the same length;
/// a depth whose vector is EMPTY is skipped rather than stored, so "no realizations survived
/// here" reads back as an absent depth instead of a zero-width distribution.
///
/// **The whole replacement is ONE transaction**, which delete-then-append demands: without it a
/// failure part way through the inserts leaves the DELETE committed and only some of the new rows
/// written — a realization matrix silently missing depths, which is not a visible breakage but a
/// biased percentile at every depth that vanished. The `with_txn` doc names the same hazard for
/// the same reason; this writer simply predated its use here. Neither caller
/// (`montecarlo::persist_realizations`, `intake::commit_arrays`) is itself inside a transaction,
/// so there is no nesting — DuckDB has none.
///
/// **A duplicated depth is refused BY NAME before anything is written.** `array_logs` is keyed
/// (well, set, curve, depth) and holds ONE vector per depth, so a repeat is a constraint violation;
/// letting the engine raise it gives a message naming an internal table and no depth, on an import
/// the user has just been told succeeded. Refusing here protects every caller rather than only the
/// one whose front end happens to check.
pub fn write_array_log(
    conn: &Connection,
    well_id: &str,
    set_name: &str,
    curve_name: &str,
    depths: &[f32],
    samples: &[Vec<f32>],
    axis: Option<&[f32]>,
) -> DbResult<usize> {
    if depths.len() != samples.len() {
        return Err(DbError::LengthMismatch(format!(
            "array log {curve_name}: {} depths against {} value vectors",
            depths.len(),
            samples.len()
        )));
    }
    // Checked over the rows that would actually be INSERTED, not over the whole input: a depth
    // whose vector is empty is skipped below and never reaches the table, so counting it here
    // would refuse a write the store would have accepted.
    let storable: Vec<f32> =
        depths.iter().zip(samples).filter(|(d, v)| !v.is_empty() && d.is_finite()).map(|(d, _)| *d).collect();
    let mut dupes: Vec<f32> = Vec::new();
    for (i, d) in storable.iter().enumerate() {
        if storable[..i].contains(d) && !dupes.contains(d) {
            dupes.push(*d);
        }
    }
    if !dupes.is_empty() {
        let list = dupes.iter().map(|d| format!("{d:.2}")).collect::<Vec<_>>().join(", ");
        return Err(DbError::Invalid(format!(
            "array log {curve_name}: {} depth(s) appear more than once ({list}). One vector is \
             stored per depth, so the repeats cannot be kept — give them their own depths, or \
             split the delivery.",
            dupes.len()
        )));
    }
    with_txn(conn, |conn| {
        conn.execute(
            "DELETE FROM array_logs WHERE well_id = ? AND set_name = ? AND upper(curve_name) = upper(?)",
            duckdb::params![well_id, set_name, curve_name],
        )?;
        let mut stmt = conn.prepare(
            "INSERT INTO array_logs (well_id, set_name, curve_name, depth, samples, axis)
             VALUES (?, ?, ?, ?, ?, ?)",
        )?;
        // The axis is stored on EVERY row rather than once per curve. One row is then
        // self-describing — a reader that fetches a single depth has the axis in hand — and the
        // cost is a few hundred bytes against a samples blob of the same order.
        let axis_blob = axis.map(encode_samples);
        let mut written = 0usize;
        for (d, vals) in depths.iter().zip(samples) {
            if vals.is_empty() || !d.is_finite() {
                continue;
            }
            stmt.execute(duckdb::params![well_id, set_name, curve_name, d, encode_samples(vals), axis_blob])?;
            written += 1;
        }
        Ok(written)
    })
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

/// Records that these curves hold class identifiers, not quantities (`curve_class`, SB-MLA-055).
///
/// Called by whatever PRODUCED the curve, because that is the only place the answer is known
/// rather than guessed. A classifier writes both a class curve and its probability curves in one
/// run, and the probabilities are ordinary continuous quantities — so this takes the output names
/// it is given and never infers a family from a prefix.
///
/// Idempotent: a re-run of the same module re-declares the same curves, and a declaration that
/// disappeared on the second run would leave the curve protected only until it was recomputed.
pub fn declare_class_curves(conn: &Connection, well_id: &str, names: &[String], source: &str) -> DbResult<()> {
    for name in names {
        conn.execute(
            "INSERT INTO curve_class (well_id, curve_name, source) VALUES (?, ?, ?)
             ON CONFLICT (well_id, curve_name) DO UPDATE SET source = excluded.source",
            duckdb::params![well_id, name.to_uppercase(), source],
        )?;
    }
    Ok(())
}

/// Every curve declared as a class curve on this well, upper-cased.
///
/// Returned as a set for the whole well in one query rather than a per-curve lookup: the callers
/// are transform paths that loop over curves, and a query per curve inside that loop would put a
/// round trip between every column and the next.
pub fn class_curves_for_well(conn: &Connection, well_id: &str) -> DbResult<std::collections::HashSet<String>> {
    let mut stmt = conn.prepare("SELECT upper(curve_name) FROM curve_class WHERE well_id = ?")?;
    let rows = stmt
        .query_map([well_id], |r| r.get::<_, String>(0))?
        .collect::<duckdb::Result<Vec<_>>>()?;
    Ok(rows.into_iter().collect())
}

/// Records the UNIT a computed curve is in (`curve_unit`, SB-MLA-035).
///
/// Declared by whatever produced the curve, for the same reason as [`declare_class_curves`]: the
/// writer is the only place the answer is known rather than guessed. A prediction of permeability
/// made in log10 space and a permeability in mD are DIFFERENT QUANTITIES, and until this existed
/// there was nowhere for them to disagree — a computed curve's unit could only ever come from an
/// `equations.output_units` row, so anything written by a module or an ML run had none at all.
///
/// An empty or blank unit stores NULL rather than `""`. "This quantity is dimensionless" and "we do
/// not know" are different statements, and only the second should let a reader fall back to a guess.
pub fn declare_curve_units(conn: &Connection, well_id: &str, units: &[(String, String)]) -> DbResult<()> {
    for (name, unit) in units {
        let u = unit.trim();
        conn.execute(
            "INSERT INTO curve_unit (well_id, curve_name, unit) VALUES (?, ?, ?)
             ON CONFLICT (well_id, curve_name) DO UPDATE SET unit = excluded.unit",
            duckdb::params![well_id, name.to_uppercase(), (!u.is_empty()).then_some(u)],
        )?;
    }
    Ok(())
}

/// The declared unit of one computed curve, if it has one.
pub fn curve_unit_for(conn: &Connection, well_id: &str, curve_name: &str) -> Option<String> {
    conn.query_row(
        "SELECT unit FROM curve_unit WHERE well_id = ? AND upper(curve_name) = upper(?)",
        duckdb::params![well_id, curve_name],
        |r| r.get::<_, Option<String>>(0),
    )
    .ok()
    .flatten()
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
/// Adds `core_data.depth_orig` (as-delivered depth) to a project that predates it.
///
/// Non-destructive: one ADD COLUMN and one back-fill, no table rebuild, so unlike
/// `migrate_point_data_sets` it needs no backup. The back-fill sets `depth_orig = depth`, which
/// says the honest thing about an old project — **whatever shifts were applied before this column
/// existed are not recoverable**, so the core is treated as if it had been delivered where it now
/// sits. New data imported against it will follow from here on, just not backwards.
///
/// Idempotent via `duckdb_columns()`; a no-op on a freshly created database and on every launch
/// after the first.
pub fn migrate_core_depth_orig(conn: &Connection) -> DbResult<()> {
    let has: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duckdb_columns()
         WHERE table_name = 'core_data' AND column_name = 'depth_orig'",
        [],
        |r| r.get(0),
    )?;
    if has == 0 {
        conn.execute_batch("ALTER TABLE core_data ADD COLUMN depth_orig FLOAT;")?;
    }
    // Runs regardless: a row could have been written by an older code path after the column
    // was added (and NULL there would silently break the map).
    conn.execute_batch("UPDATE core_data SET depth_orig = depth WHERE depth_orig IS NULL;")?;
    Ok(())
}

/// Adds `on_core_depths` to the point-data, SCAL and image delivery registries.
///
/// Non-destructive (ADD COLUMN only, no rebuild, no backup needed) and idempotent. Existing
/// deliveries get **0** — "not known to be on core depths" — which is the safe answer: a later
/// core registration will leave them alone rather than moving data that may already be on the
/// log's scale. The user can still tick them by hand; the flag only chooses the default.
pub fn migrate_delivery_depth_basis(conn: &Connection) -> DbResult<()> {
    for table in ["aux_sets", "scal_sets", "image_sets"] {
        let has: i64 = conn.query_row(
            "SELECT COUNT(*) FROM duckdb_columns()
             WHERE table_name = ?1 AND column_name = 'on_core_depths'",
            params![table],
            |r| r.get(0),
        )?;
        if has == 0 {
            conn.execute_batch(&format!(
                "ALTER TABLE {table} ADD COLUMN on_core_depths INTEGER NOT NULL DEFAULT 0;"
            ))?;
        }
    }
    Ok(())
}

/// Adds `axis` to the array-log store: what each stored value is a measurement AT.
///
/// ADD COLUMN only, no rebuild, no backup. Existing rows get NULL, which is not a guess — the
/// only writer before this was `montecarlo::persist_realizations`, whose values genuinely have no
/// axis (realization 7 is not a measurement at 7 of anything).
///
/// It must stay the LAST column, the `depth_orig` rule: a migrated database gets it appended, and
/// a fresh one is built by `create_schema`, so the two shapes have to agree.
pub fn migrate_array_log_axis(conn: &Connection) -> DbResult<()> {
    let has: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duckdb_columns() WHERE table_name = 'array_logs' AND column_name = 'axis'",
        [],
        |r| r.get(0),
    )?;
    if has == 0 {
        conn.execute_batch("ALTER TABLE array_logs ADD COLUMN axis BLOB;")?;
    }
    Ok(())
}

/// Adds `frame` to the log-set registry: `'STANDARD'` (the well's own depth grid, which is what
/// every module writes on) or `'OWN'` (the set carries its own depth column — see
/// [`crate::reframe`]).
///
/// ADD COLUMN only, no rebuild, no backup. Existing sets get `'STANDARD'`, which is not a guess:
/// nothing before `reframe` could write a set on any other frame, so every row that exists really
/// is on the well's grid.
///
/// **Explicit rather than inferred from the depths.** A set that happens to fall on the standard
/// grid and one deliberately re-framed onto it hold identical rows, and reading the difference off
/// the data would make the behaviour of every existing project depend on a coincidence.
pub fn migrate_log_set_frame(conn: &Connection) -> DbResult<()> {
    let has: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duckdb_columns() WHERE table_name = 'log_sets' AND column_name = 'frame'",
        [],
        |r| r.get(0),
    )?;
    if has == 0 {
        conn.execute_batch(&format!(
            "ALTER TABLE log_sets ADD COLUMN frame VARCHAR NOT NULL DEFAULT '{}';",
            crate::schema_vocab::LogSetFrame::Standard.as_str()
        ))?;
    }
    Ok(())
}

/// Adds the plate's scale and preparation columns. ADD COLUMN only — no rebuild, no backup —
/// and existing plates get NULL, which is the honest answer: nothing in a stored JPEG says how
/// wide the field of view was or whether the section was impregnated, so an older delivery is
/// UNKNOWN rather than assumed calibrated or assumed plain.
/// Adds the core-photograph conditioning columns. ADD COLUMN only — no rebuild, so no backup, the
/// [`migrate_plate_scale_and_prep`] precedent. Existing pictures get NULL, which reads as "exactly
/// as imported" and is the honest answer for a delivery nobody has conditioned.
///
/// They must stay the LAST columns for the same reason every other late column here does.
pub fn migrate_core_image_recipe(conn: &Connection) -> DbResult<()> {
    for (col, ty) in [("recipe", "VARCHAR"), ("source_data", "BLOB"), ("source_meta", "VARCHAR")] {
        let has: i64 = conn.query_row(
            "SELECT COUNT(*) FROM duckdb_columns()
             WHERE table_name = 'well_images' AND column_name = ?1",
            params![col],
            |r| r.get(0),
        )?;
        if has == 0 {
            conn.execute_batch(&format!("ALTER TABLE well_images ADD COLUMN {col} {ty};"))?;
        }
    }
    Ok(())
}

pub fn migrate_plate_scale_and_prep(conn: &Connection) -> DbResult<()> {
    for (col, ty) in [("fov_um", "FLOAT"), ("prepared", "VARCHAR"), ("stain", "VARCHAR")] {
        let has: i64 = conn.query_row(
            "SELECT COUNT(*) FROM duckdb_columns()
             WHERE table_name = 'well_images' AND column_name = ?1",
            params![col],
            |r| r.get(0),
        )?;
        if has == 0 {
            conn.execute_batch(&format!("ALTER TABLE well_images ADD COLUMN {col} {ty};"))?;
        }
    }
    Ok(())
}

/// Gives an existing project's fluid contacts a compartment and a marker link.
///
/// ADD COLUMN plus a CREATE TABLE — no rebuild, so unlike `migrate_point_data_sets` it needs no
/// backup. Existing contacts get a NULL compartment and no marker rows, which is the honest answer
/// rather than the safe-looking one: nothing in a stored contact says which sand or which fault
/// block it was picked in, and inventing an association would be worse than admitting there is
/// none. An unassigned contact is its own QC group, never a member of every group.
///
/// `compartment` must stay the LAST column — `create_schema` puts it last too, so a fresh database
/// and a migrated one agree about column order.
pub fn migrate_fluid_contact_zone(conn: &Connection) -> DbResult<()> {
    let has: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duckdb_columns()
         WHERE table_name = 'fluid_contacts' AND column_name = 'compartment'",
        [],
        |r| r.get(0),
    )?;
    if has == 0 {
        conn.execute_batch("ALTER TABLE fluid_contacts ADD COLUMN compartment VARCHAR;")?;
    }
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS contact_zones (
            contact_id VARCHAR NOT NULL,
            zone_name  VARCHAR NOT NULL,
            PRIMARY KEY (contact_id, zone_name)
        );",
    )?;
    Ok(())
}

/// Brings a pre-set-era project onto the universal delivery-set model: core plugs, SCAL Pc,
/// deviation surveys and every point dataset gain a `set_name`, existing rows become RAW and
/// active, and the readings stay byte-identical. `core_data` and `well_path` are REBUILT because
/// the set name joins their primary key; `aux_data` and `scal_pc` are only ALTERed, back-filled
/// and registered.
///
/// A rebuild is a DESTRUCTIVE migration (RELEASE §3.2), so when one is actually going to run,
/// `path` is backed up first and a failed backup ABORTS: an un-migrated project still opens, so
/// refusing costs nothing, while rewriting after the promised copy failed breaks the exact
/// guarantee the backup exists to make. `path: None` is for in-memory test databases only.
/// Idempotent: a project whose `core_data` already carries `set_name` is left alone.
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
    conn.execute_batch(&format!(
        "ALTER TABLE aux_sets ADD COLUMN IF NOT EXISTS sampling_style VARCHAR;
         ALTER TABLE aux_sets ADD COLUMN IF NOT EXISTS duplicate_resolution VARCHAR;
         ALTER TABLE aux_sets ADD COLUMN IF NOT EXISTS perturbation_value DOUBLE;
         ALTER TABLE aux_sets ADD COLUMN IF NOT EXISTS perturbation_unit VARCHAR;
         -- SB-DBM-031: rebuilt pre-set-era stores converge on the datum column too; the
         -- value stays NULL - legacy unknown is preserved, never inferred to MD.
         ALTER TABLE core_sets  ADD COLUMN IF NOT EXISTS depth_datum VARCHAR;
         ALTER TABLE aux_sets   ADD COLUMN IF NOT EXISTS depth_datum VARCHAR;
         ALTER TABLE scal_sets  ADD COLUMN IF NOT EXISTS depth_datum VARCHAR;
         UPDATE aux_sets SET sampling_style = '{}' WHERE sampling_style IS NULL;
         UPDATE aux_sets SET duplicate_resolution = '{}' WHERE duplicate_resolution IS NULL;",
        crate::schema_vocab::SamplingStyle::Point.as_str(),
        crate::schema_vocab::DuplicateDepthResolution::Preserve.as_str()
    ))?;
    conn.execute_batch(&format!(
        "UPDATE aux_data SET set_name = 'RAW' WHERE set_name IS NULL;
         INSERT INTO aux_sets
             (well_id, dataset, set_name, active, sampling_style, duplicate_resolution)
         SELECT DISTINCT a.well_id, a.dataset, a.set_name, 1, '{}', '{}' FROM aux_data a
         WHERE NOT EXISTS (SELECT 1 FROM aux_sets s
                           WHERE s.well_id = a.well_id AND s.dataset = a.dataset);
         UPDATE scal_pc SET set_name = 'RAW' WHERE set_name IS NULL;
         INSERT INTO scal_sets (well_id, set_name, active)
         SELECT DISTINCT p.well_id, p.set_name, 1 FROM scal_pc p
         WHERE NOT EXISTS (SELECT 1 FROM scal_sets s WHERE s.well_id = p.well_id);",
        crate::schema_vocab::SamplingStyle::Point.as_str(),
        crate::schema_vocab::DuplicateDepthResolution::Preserve.as_str()
    ))?;

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

/// The seeding a pre-generic-store project holds, followed by the back-fill that
/// `project::open_and_migrate` runs on every open - so a fixture reads the way a real project
/// reads, instead of relying on somebody repairing it mid-test.
///
/// #129: production has exactly ONE writer of `standard_curves`, the LAS import, and it marks its
/// own wells done inside the same import transaction (`ingest.rs`). A fixture that writes those
/// columns directly is therefore a legacy project, and a legacy project that has not been OPENED
/// is a state no reader ever meets. Until 2026-08-23 `ancestry::try_resolve_ancestry_input`
/// papered over that by running this back-fill lazily from inside a read, which is what broke the
/// connection pool - N reader connections each ran the whole project-wide write and collided on
/// `curve_meta`'s primary key (`PERF-ATTEMPTS.md` §4). The repair is the open's job; a fixture
/// that wants a readable project asks for one HERE.
///
/// **Which door a fixture takes is decided by what it writes.** A fixture that writes ONLY the
/// standard columns is a legacy project and takes this one. A fixture that also writes its own
/// `curve_meta`/`curve_samples` rows is an IMPORTED project - the import wrote both views from one
/// delivery and marked the well done - so it takes the plain `insert_standard_curves` and must
/// not be back-filled: the back-fill would add a competing `standard_curves migration` identity,
/// which is a third candidate in a candidate-selection test and four extra curves in an export
/// count. Both of those were caught that way.
#[cfg(test)]
pub(crate) fn insert_standard_curves_as_opened_project(
    conn: &Connection,
    well_id: Uuid,
    depths: Vec<f32>,
    gr: Vec<f32>,
    res_deep: Vec<f32>,
    nphi: Vec<f32>,
    rhob: Vec<f32>,
    dt: Vec<f32>,
    sp: Vec<f32>,
) -> DbResult<Vec<(String, usize)>> {
    let screened =
        insert_standard_curves(conn, well_id, depths, gr, res_deep, nphi, rhob, dt, sp)?;
    migrate_standard_curves_to_generic_store(conn)?;
    Ok(screened)
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
) -> DbResult<Vec<(String, usize)>> {
    let n = depths.len();
    if gr.len() != n || res_deep.len() != n || nphi.len() != n || rhob.len() != n || dt.len() != n || sp.len() != n {
        return Err(DbError::LengthMismatch(format!(
            "expected all columns to have length {n}"
        )));
    }

    let well_id_str = well_id.to_string();
    let mut appender: Appender = conn.appender("standard_curves")?;
    // SB-DBM-030: the standard projection screens and NULL-binds exactly as the generic store
    // does - one delivery lands in both, and a value screened in one but kept in the other
    // would be two truths about the same sample.
    let mut screened: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for i in 0..n {
        let supplied = |mnemonic: &str| match mnemonic {
            "DEPTH" => Some(depths[i]),
            "GR" => Some(gr[i]),
            "RES_DEEP" => Some(res_deep[i]),
            "NPHI" => Some(nphi[i]),
            "RHOB" => Some(rhob[i]),
            "DT" => Some(dt[i]),
            "SP" => Some(sp[i]),
            _ => None,
        };
        let mut values = Vec::with_capacity(crate::schema_vocab::STANDARD_COLUMNS.len() + 1);
        values.push(duckdb::types::Value::Text(well_id_str.clone()));
        for column in crate::schema_vocab::STANDARD_COLUMNS {
            match supplied(column.mnemonic) {
                // The depth index is never screened: a NULL index row is not a missing
                // measurement, it is a broken frame, and the parser already rejects one.
                Some(value) if column.mnemonic == "DEPTH" => {
                    values.push(duckdb::types::Value::Float(value))
                }
                Some(value) if value.is_nan() => values.push(duckdb::types::Value::Null),
                Some(value) if is_large_negative_null(value) => {
                    *screened.entry(column.mnemonic).or_insert(0) += 1;
                    values.push(duckdb::types::Value::Null);
                }
                Some(value) => values.push(duckdb::types::Value::Float(value)),
                None if column.required => {
                    return Err(DbError::Invalid(format!(
                        "required standard column '{}' has no insert projection",
                        column.mnemonic
                    )));
                }
                None => values.push(duckdb::types::Value::Null),
            }
        }
        appender.append_row(duckdb::appender_params_from_iter(values.iter()))?;
    }
    appender.flush()?;
    Ok(screened.into_iter().map(|(mnemonic, count)| (mnemonic.to_string(), count)).collect())
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

/// What one delivery-set rename moved, table by table, so the receipt can be checked
/// against the delivery rather than taken on faith.
#[derive(Debug, Clone, Serialize)]
pub struct SetRenameReceipt {
    pub rows_moved: usize,
    /// Core only: same-named aux rows (extras and other riders) that travelled with the
    /// core set. Zero for every other kind.
    pub rider_rows_moved: usize,
}

/// Renames one delivery set, moving EVERY row that carries the name in one transaction —
/// a name left behind in any table is a delivery silently split in two. Kinds: core /
/// scal / survey / aux / image / curve (`dataset` names the aux or image dataset and is
/// ignored elsewhere). The rename is audited (mode RENAME) under the session operator,
/// validated BEFORE anything moves — the purge engine's DEC-020 posture.
///
/// Three refusals carry the data-integrity reasoning:
/// - **Curve set RAW is never renamed, in either direction.** RAW has absolute priority
///   in curve resolution and the frame declarations are keyed to it; renaming it (or
///   renaming another set TO it) silently re-decides which delivery answers every
///   mnemonic.
/// - **A rename never merges.** A target name already naming a delivery of the same kind
///   on the well is refused — the same rule that makes an import auto-suffix instead of
///   overwrite.
/// - **A core set's riders follow the core, never travel alone.** Aux rows sharing a
///   core set's name ARE that core's extras (the active-core-set reader correlates on
///   the bare name), so renaming the core moves them too, and renaming an aux delivery
///   that shares a core set's name is refused by name — rename the core set instead.
pub fn rename_delivery_set(
    conn: &Connection,
    kind: &str,
    well_id: &str,
    dataset: Option<&str>,
    old: &str,
    new: &str,
    operator: &str,
    operator_kind: &str,
    view: &str,
) -> DbResult<SetRenameReceipt> {
    let new = new.trim();
    if new.is_empty() || new == old {
        return Err(DbError::Invalid(
            "rename refused: the new name is empty or unchanged - type a different name in Data Sets".into(),
        ));
    }
    if operator.trim().is_empty() {
        return Err(DbError::Invalid(
            "rename refused: enter the session operator identity - every rename is audited and the operator is never inferred (DEC-020)".into(),
        ));
    }
    let exists = |table: &str, name_col: &str, name: &str| -> DbResult<bool> {
        let sql = match dataset {
            Some(_) if matches!(kind, "aux" | "image") => format!(
                "SELECT COUNT(*) FROM {table} WHERE well_id = ?1 AND dataset = ?3 AND {name_col} = ?2"
            ),
            _ => format!("SELECT COUNT(*) FROM {table} WHERE well_id = ?1 AND {name_col} = ?2"),
        };
        let n: i64 = match dataset {
            Some(ds) if matches!(kind, "aux" | "image") => {
                conn.query_row(&sql, params![well_id, name, ds], |r| r.get(0))?
            }
            _ => conn.query_row(&sql, params![well_id, name], |r| r.get(0))?,
        };
        Ok(n > 0)
    };
    // (registry table, name column, [data tables sharing that name column])
    let (registry, name_col, data_tables): (&str, &str, &[&str]) = match kind {
        "core" => ("core_sets", "set_name", &["core_data", "core_registrations"]),
        "scal" => ("scal_sets", "set_name", &["scal_pc"]),
        "survey" => ("well_surveys", "survey_name", &["well_path"]),
        "aux" => ("aux_sets", "set_name", &["aux_data", "aux_duplicate_depth_resolutions"]),
        "image" => ("image_sets", "set_name", &["well_images"]),
        "curve" => ("curve_meta", "set_name", &["import_sets", "array_logs"]),
        other => {
            return Err(DbError::Invalid(format!(
                "rename refused: '{other}' is not a delivery kind (core/scal/survey/aux/image/curve)"
            )))
        }
    };
    if matches!(kind, "aux" | "image") && dataset.is_none() {
        return Err(DbError::Invalid(
            "rename refused: an aux or image delivery is named per dataset - pass the dataset it belongs to".into(),
        ));
    }
    if kind == "curve" && (old == "RAW" || new == "RAW") {
        return Err(DbError::Invalid(
            "rename refused: curve set RAW cannot be renamed or taken - RAW has absolute priority in curve resolution and the frame declarations are keyed to it. Import under the name you want instead (Import LAS, set dialog)".into(),
        ));
    }
    if !exists(registry, name_col, old)? {
        return Err(DbError::Invalid(format!(
            "rename refused: no {kind} delivery named '{old}' on this well - refresh Data Sets and pick from the list"
        )));
    }
    if exists(registry, name_col, new)? {
        return Err(DbError::Invalid(format!(
            "rename refused: '{new}' already names a {kind} delivery on this well - an import never overwrites and a rename never merges. Pick an unused name"
        )));
    }
    // The rider coupling is by BARE NAME across registries (see ACTIVE_CORE_SET's aux
    // reader), so it is checked across registries too.
    let core_named = |name: &str| -> DbResult<bool> {
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM core_sets WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, name],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    };
    if kind == "aux" {
        if core_named(old)? {
            return Err(DbError::Invalid(format!(
                "rename refused: '{old}' rides the core delivery of the same name (its rows follow the active core set) - rename the CORE set in Data Sets and the riders move with it"
            )));
        }
        if core_named(new)? {
            return Err(DbError::Invalid(format!(
                "rename refused: '{new}' names a core delivery on this well, and an aux set under a core set's name becomes its rider - pick a name no core set carries"
            )));
        }
    }
    if kind == "core" {
        let aux_taken: i64 = conn.query_row(
            "SELECT COUNT(*) FROM aux_sets WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, new],
            |r| r.get(0),
        )?;
        if aux_taken > 0 {
            return Err(DbError::Invalid(format!(
                "rename refused: '{new}' already names a point-data delivery on this well, and the core set's riders would merge into it - pick an unused name"
            )));
        }
    }
    // One transaction: the registry, every data table, and — for a core set — the aux
    // riders that share its name. All of it or none of it.
    let (rows_moved, rider_rows_moved) = with_txn(conn, |conn| {
        let mut rows = 0usize;
        let mut riders = 0usize;
        let rename_in = |conn: &Connection, table: &str, col: &str| -> Result<usize, duckdb::Error> {
            match dataset {
                Some(ds) if matches!(kind, "aux" | "image") => conn.execute(
                    &format!("UPDATE {table} SET {col} = ?3 WHERE well_id = ?1 AND dataset = ?4 AND {col} = ?2"),
                    params![well_id, old, new, ds],
                ),
                _ => conn.execute(
                    &format!("UPDATE {table} SET {col} = ?3 WHERE well_id = ?1 AND {col} = ?2"),
                    params![well_id, old, new],
                ),
            }
        };
        rows += rename_in(conn, registry, name_col)?;
        for table in data_tables {
            rows += rename_in(conn, table, name_col)?;
        }
        if kind == "core" {
            for table in ["aux_sets", "aux_data", "aux_duplicate_depth_resolutions"] {
                riders += conn.execute(
                    &format!("UPDATE {table} SET set_name = ?3 WHERE well_id = ?1 AND set_name = ?2"),
                    params![well_id, old, new],
                )?;
            }
        }
        Ok::<_, duckdb::Error>((rows, riders))
    })?;
    // Mutate-then-audit, the house order (`set_zone_param_audited`); '.' is replaced in the
    // audited names because the dotted-name rule reserves it for attribute changes.
    record_audit_entry(
        conn,
        Some(well_id),
        operator,
        operator_kind,
        view,
        "rename_delivery_set",
        None,
        None,
        &[AuditDetail {
            location: "SET".into(),
            mode: "RENAME".into(),
            unit: None,
            name: old.replace('.', "_"),
            value: Some(format!("{kind} -> {}", new.replace('.', "_"))),
        }],
    )?;
    Ok(SetRenameReceipt { rows_moved, rider_rows_moved })
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
            // depth_orig starts equal to depth and is never shifted afterwards — it is the
            // record of where this delivery said the rock was.
            appender.append_row(params![
                well_id,
                set_name,
                depths[i],
                cpor[i],
                cperm[i],
                cgd[i],
                csw[i],
                depths[i]
            ])?;
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
        conn.execute(
            "DELETE FROM aux_duplicate_depth_resolutions
             WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
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
    insert_aux_data_with_resolution(
        conn,
        well_id,
        dataset,
        set_name,
        source,
        rows,
        crate::schema_vocab::DuplicateDepthResolution::Preserve,
        None,
    )
}

#[derive(Debug)]
struct AuxDuplicateDecision {
    item: String,
    source_row: i64,
    original_depth: f32,
    stored_depth: f32,
}

fn depth_identity(depth: f32) -> u32 {
    if depth == 0.0 {
        0.0_f32.to_bits()
    } else {
        depth.to_bits()
    }
}

/// Point-data form of SB-DBM-026. The ordinary import path calls this with explicit PRESERVE;
/// PERTURB is available only when its caller supplies a positive unit-typed offset. No numeric
/// fallback exists.
pub fn insert_aux_data_with_resolution(
    conn: &Connection,
    well_id: &str,
    dataset: &str,
    set_name: &str,
    source: Option<&str>,
    rows: &[AuxRow],
    resolution: crate::schema_vocab::DuplicateDepthResolution,
    perturbation: Option<crate::units::DepthOffset>,
) -> DbResult<()> {
    use crate::schema_vocab::DuplicateDepthResolution::{Perturb, Preserve, Refuse};

    match (resolution, perturbation) {
        (Refuse, _) => {
            return Err(DbError::Invalid(
                "POINT duplicate resolution must preserve or perturb, not refuse".into(),
            ))
        }
        (Preserve, Some(_)) => {
            return Err(DbError::Invalid(
                "POINT PRESERVE must not carry a perturbation offset".into(),
            ))
        }
        (Perturb, None) => {
            return Err(DbError::Invalid(
                "POINT PERTURB requires an explicit positive unit-typed offset; no default ships"
                    .into(),
            ))
        }
        (Perturb, Some(offset)) if !offset.value.is_finite() || offset.value <= 0.0 => {
            return Err(DbError::Invalid(
                "POINT PERTURB offset must be finite and greater than zero".into(),
            ))
        }
        _ => {}
    }
    if resolution == Perturb && rows.iter().any(|row| row.depth_base.is_some()) {
        return Err(DbError::Invalid(
            "POINT PERTURB refuses interval rows; changing only an interval top would change its meaning"
                .into(),
        ));
    }

    let project_unit = if resolution == Perturb {
        Some(
            crate::units::require_project_depth_unit(conn, "POINT duplicate perturbation")
                .map_err(DbError::Invalid)?,
        )
    } else {
        None
    };
    let offset_in_project = match (perturbation, project_unit) {
        (Some(offset), Some(unit)) => Some(
            crate::units::convert_depth(offset.value, offset.unit, unit) as f32,
        ),
        _ => None,
    };

    let mut group_counts = std::collections::HashMap::<(String, u32), usize>::new();
    let mut original_depths = std::collections::HashMap::<String, std::collections::HashSet<u32>>::new();
    for row in rows {
        let item = row.item.trim().to_ascii_uppercase();
        let key = depth_identity(row.depth_top);
        *group_counts.entry((item.clone(), key)).or_default() += 1;
        original_depths.entry(item).or_default().insert(key);
    }
    let mut occurrences = std::collections::HashMap::<(String, u32), usize>::new();
    let mut resolved_depths = std::collections::HashMap::<String, std::collections::HashSet<u32>>::new();
    let mut stored_rows = Vec::with_capacity(rows.len());
    let mut decisions = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let item_key = row.item.trim().to_ascii_uppercase();
        let original_key = depth_identity(row.depth_top);
        let occurrence = occurrences
            .entry((item_key.clone(), original_key))
            .or_default();
        let mut stored = row.clone();
        if let Some(offset) = offset_in_project.filter(|_| *occurrence > 0) {
            stored.depth_top = row.depth_top + offset * (*occurrence as f32);
            if !stored.depth_top.is_finite() {
                return Err(DbError::Invalid(format!(
                    "POINT duplicate perturbation refused for item '{}' source row {}: stored depth is not finite",
                    row.item,
                    index + 1
                )));
            }
            let stored_key = depth_identity(stored.depth_top);
            if stored_key == original_key {
                return Err(DbError::Invalid(format!(
                    "POINT duplicate perturbation refused for item '{}' source row {}: the unit-typed offset rounds to zero on the stored depth",
                    row.item,
                    index + 1
                )));
            }
            if original_depths
                .get(&item_key)
                .is_some_and(|depths| depths.contains(&stored_key))
                || resolved_depths
                    .get(&item_key)
                    .is_some_and(|depths| depths.contains(&stored_key))
            {
                return Err(DbError::Invalid(format!(
                    "POINT duplicate perturbation refused for item '{}' source row {}: stored depth {} collides with another source row",
                    row.item,
                    index + 1,
                    stored.depth_top
                )));
            }
        }
        if resolution == Perturb {
            resolved_depths
                .entry(item_key.clone())
                .or_default()
                .insert(depth_identity(stored.depth_top));
        }
        if group_counts
            .get(&(item_key, original_key))
            .copied()
            .unwrap_or(0)
            > 1
        {
            decisions.push(AuxDuplicateDecision {
                item: row.item.clone(),
                source_row: (index + 1) as i64,
                original_depth: row.depth_top,
                stored_depth: stored.depth_top,
            });
        }
        stored_rows.push(stored);
        *occurrence += 1;
    }

    with_txn(conn, |conn| {
        conn.execute(
            "DELETE FROM aux_data WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
            params![well_id, dataset, set_name],
        )?;
        let mut appender: Appender = conn.appender("aux_data")?;
        for r in &stored_rows {
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
        drop(appender);
        conn.execute(
            "DELETE FROM aux_sets WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
            params![well_id, dataset, set_name],
        )?;
        conn.execute(
            "DELETE FROM aux_duplicate_depth_resolutions
             WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
            params![well_id, dataset, set_name],
        )?;
        conn.execute(
            "UPDATE aux_sets SET active = 0 WHERE well_id = ?1 AND dataset = ?2",
            params![well_id, dataset],
        )?;
        conn.execute(
            "INSERT INTO aux_sets
                (well_id, dataset, set_name, active, source, sampling_style,
                 duplicate_resolution, perturbation_value, perturbation_unit)
             VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7, ?8)",
            params![
                well_id,
                dataset,
                set_name,
                source,
                crate::schema_vocab::SamplingStyle::Point.as_str(),
                resolution.as_str(),
                perturbation.map(|offset| offset.value),
                perturbation.map(|offset| offset.unit.code())
            ],
        )?;
        for decision in &decisions {
            conn.execute(
                "INSERT INTO aux_duplicate_depth_resolutions
                    (well_id, dataset, set_name, item, source_row, original_depth, stored_depth,
                     resolution, perturbation_value, perturbation_unit)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    well_id,
                    dataset,
                    set_name,
                    decision.item,
                    decision.source_row,
                    decision.original_depth,
                    decision.stored_depth,
                    resolution.as_str(),
                    perturbation.map(|offset| offset.value),
                    perturbation.map(|offset| offset.unit.code())
                ],
            )?;
        }
        Ok(())
    })
}

/// One well's auxiliary rows from the ACTIVE set of each dataset, ordered by depth then item.
pub fn list_aux_data(conn: &Connection, well_id: &str, dataset: Option<&str>) -> DbResult<Vec<AuxRow>> {
    refuse_non_md_active_set(conn, "aux_sets", well_id, dataset)?;
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

/// One measurement name inside a point dataset, with what is actually stored under it.
#[derive(Debug, Clone, Serialize)]
pub struct AuxItemInfo {
    pub dataset: String,
    pub item: String,
    /// Wells carrying it in their ACTIVE delivery.
    pub wells: i64,
    pub rows: i64,
    /// Rows whose value is a NUMBER. A dialog that needs a measurement to compute with — the
    /// S-factor calibration wants lab CEC — must be able to tell a numeric item from a
    /// descriptive one, because a lithology description cannot set a scaling factor and
    /// offering it as a choice invites a run that fails for reasons the user cannot see.
    pub numeric_rows: i64,
}

/// Every measurement name in the project's point data, from the ACTIVE delivery of each dataset.
///
/// Deliberately unfiltered by well, for the same reason [`list_well_param_overrides`] is: one
/// scan of a grouped aggregate beats either N round trips or an `IN (...)` list long enough to
/// hit a binding limit on a 2000-well project. The result is a project-wide catalogue of what a
/// dataset/item box could name, which is the question a picker actually asks — a run's own
/// exclusion counts still report what the chosen wells turned out to hold.
pub fn list_aux_item_catalog(conn: &Connection) -> DbResult<Vec<AuxItemInfo>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT a.dataset, a.item, COUNT(DISTINCT a.well_id), COUNT(*), COUNT(a.value_num)
         FROM aux_data a WHERE a.set_name = {ACTIVE_AUX_SET}
         GROUP BY a.dataset, a.item ORDER BY a.dataset, a.item"
    ))?;
    let rows = stmt.query_map([], |row| {
        Ok(AuxItemInfo {
            dataset: row.get(0)?,
            item: row.get(1)?,
            wells: row.get(2)?,
            rows: row.get(3)?,
            numeric_rows: row.get(4)?,
        })
    })?;
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
#[derive(Debug, Clone, Default, Serialize)]
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
    /// Width of the WHOLE picture in micrometres. None = no scale declared, and nothing
    /// dimensional may run on this plate.
    pub fov_um: Option<f32>,
    /// '' = unknown (refused by anything that needs to know), 'blue_epoxy', 'plain'.
    pub prepared: String,
    /// As the laboratory report names it; empty = none or not stated.
    pub stain: String,
}

/// One picture on its way INTO the store (the import commit path).
#[derive(Debug, Clone, Default)]
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
    pub fov_um: Option<f32>,
    pub prepared: Option<String>,
    pub stain: Option<String>,
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
                                      source_path, printable, data, fov_um, prepared, stain)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
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
                img.fov_um,
                img.prepared,
                img.stain,
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
    refuse_non_md_active_set(conn, "image_sets", well_id, dataset)?;
    let mut stmt = conn.prepare(&format!(
        "SELECT CAST(i.image_id AS VARCHAR), i.dataset, i.set_name, i.depth_top, i.depth_base,
                i.name, i.caption, i.mime, i.width, i.height, i.src_width, i.src_height,
                i.source_path, i.printable, octet_length(i.data),
                i.fov_um, COALESCE(i.prepared, ''), COALESCE(i.stain, '')
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
            fov_um: r.get(15)?,
            prepared: r.get(16)?,
            stain: r.get(17)?,
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

/// The picture conditioning must start from: the un-conditioned display copy where one was kept,
/// otherwise the picture itself.
///
/// **Never `data` when a recipe has been baked.** Editing a recipe means re-rendering from the
/// import, not stacking a second correction on top of the first — a brightness raised twice by
/// eye is a photograph nobody can get back to, and that is exactly what "non-destructive" has to
/// rule out.
pub fn get_well_image_source(conn: &Connection, image_id: &str) -> DbResult<(String, Vec<u8>)> {
    let row = conn.query_row(
        "SELECT mime, COALESCE(source_data, data) FROM well_images WHERE image_id = ?1",
        params![image_id],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, Vec<u8>>(1)?)),
    )?;
    Ok(row)
}

/// Bakes a conditioned picture, keeping the import.
///
/// The kept copy and its `WxH;mime` are filled only the FIRST time, by the `COALESCE`s below rather
/// than by a read-then-write: two applies in flight could otherwise both see them empty and the
/// second would file the first's output as the original, quietly making that correction permanent.
pub fn bake_image_conditioned(
    conn: &Connection,
    image_id: &str,
    recipe: &str,
    data: &[u8],
    mime: &str,
    width: i32,
    height: i32,
) -> DbResult<usize> {
    Ok(conn.execute(
        "UPDATE well_images
            SET source_data = COALESCE(source_data, data),
                source_meta = COALESCE(source_meta, width || 'x' || height || ';' || mime),
                data = ?2, recipe = ?3, mime = ?4, width = ?5, height = ?6
          WHERE image_id = ?1",
        params![image_id, data, recipe, mime, width, height],
    )?)
}

/// Puts a conditioned picture back exactly as it was imported, and drops the kept copy — a picture
/// with nothing left to undo should not carry a second blob for the life of the project.
///
/// **`width`, `height` and `mime` are restored from `source_meta`, not left as they were.** A crop
/// changes the picture's shape, so leaving the baked dimensions behind would have every renderer
/// draw the restored plate at the wrong aspect ratio — the one thing this app never does to a
/// photograph. A row with no kept copy was never conditioned and is left alone.
pub fn clear_image_conditioning(conn: &Connection, image_id: &str) -> DbResult<usize> {
    let meta: Option<String> = conn
        .prepare("SELECT source_meta FROM well_images WHERE image_id = ?1 AND source_data IS NOT NULL")?
        .query_map(params![image_id], |r| r.get::<_, Option<String>>(0))?
        .next()
        .transpose()?
        .flatten();
    let Some(meta) = meta else {
        // Nothing kept means nothing baked. Still clear any stray recipe, so a row cannot claim a
        // conditioning that was never applied to its pixels.
        return Ok(conn.execute(
            "UPDATE well_images SET recipe = NULL WHERE image_id = ?1 AND source_data IS NULL",
            params![image_id],
        )?);
    };
    let (dims, mime) = meta.split_once(';').unwrap_or((meta.as_str(), "image/jpeg"));
    let (w, h) = dims.split_once('x').unwrap_or(("0", "0"));
    let (w, h) = (w.parse::<i32>().unwrap_or(0), h.parse::<i32>().unwrap_or(0));
    Ok(conn.execute(
        "UPDATE well_images
            SET data = source_data, source_data = NULL, source_meta = NULL, recipe = NULL,
                mime = ?2, width = ?3, height = ?4
          WHERE image_id = ?1",
        params![image_id, mime, w, h],
    )?)
}

/// The conditioning recipes of one dataset's live delivery, keyed by picture. Empty string where a
/// picture is exactly as imported. Never reads a blob.
pub fn list_image_recipes(
    conn: &Connection,
    well_id: &str,
    dataset: &str,
) -> DbResult<Vec<(String, String)>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT CAST(i.image_id AS VARCHAR), COALESCE(i.recipe, '')
         FROM well_images i
         WHERE i.well_id = ?1 AND i.dataset = ?2 AND i.set_name = {ACTIVE_IMAGE_SET}
         ORDER BY i.depth_top, i.name"
    ))?;
    let rows = stmt.query_map(params![well_id, dataset], |r| Ok((r.get(0)?, r.get(1)?)))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Every printable picture of one dataset in a depth window, pixels included — the composite
/// exporter's read path. Non-printable rows come back too (with their bytes) so the exporter
/// can draw a labelled placeholder rather than silently dropping a plate.
///
/// AUDIT-2026-08-20 finding 25. The datum guard was on [`list_well_images`] (screen) and NOT
/// here, so a delivery declaring TVD or TVDSS showed nothing on screen and printed every plate
/// — the deliverable disagreeing with the thing it was checked against, and in the direction
/// that ships. The plate is placed against the MD log frame exactly as a core plug is, so a
/// cross-datum delivery puts it beside the wrong rock; same guard, same reason, both sides.
pub fn read_images_for_print(
    conn: &Connection,
    well_id: &str,
    dataset: &str,
    depth_top: f32,
    depth_bottom: f32,
) -> DbResult<Vec<(ImageInfo, Vec<u8>)>> {
    refuse_non_md_active_set(conn, "image_sets", well_id, Some(dataset))?;
    let mut stmt = conn.prepare(&format!(
        "SELECT CAST(i.image_id AS VARCHAR), i.dataset, i.set_name, i.depth_top, i.depth_base,
                i.name, i.caption, i.mime, i.width, i.height, i.src_width, i.src_height,
                i.source_path, i.printable, octet_length(i.data),
                i.fov_um, COALESCE(i.prepared, ''), COALESCE(i.stain, ''), i.data
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
                fov_um: r.get(15)?,
                prepared: r.get(16)?,
                stain: r.get(17)?,
            },
            r.get::<_, Vec<u8>>(18)?,
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

/// How big the rock in one plate is, and how the section was prepared.
///
/// Separate from [`update_well_image`] because they are different facts with different lifetimes:
/// a depth is corrected by registering the core, while a field of view is a property of the
/// microscope that took the picture and does not change when the core moves.
///
/// Every argument is written as given, `None` included — clearing a wrongly-typed scale has to be
/// possible, and a scale that cannot be cleared is worse than one that was never entered.
pub fn set_image_details(
    conn: &Connection,
    image_id: &str,
    fov_um: Option<f32>,
    prepared: Option<&str>,
    stain: Option<&str>,
) -> DbResult<usize> {
    Ok(conn.execute(
        "UPDATE well_images SET fov_um = ?2, prepared = ?3, stain = ?4 WHERE image_id = ?1",
        params![image_id, fov_um, prepared, stain],
    )?)
}

/// The same three facts across a whole live delivery, in one statement.
///
/// A delivery is usually uniform — one microscope, one preparation run — and correcting it plate
/// by plate would be hundreds of IPC round trips to apply one decision, the same argument
/// [`shift_well_images`] is built on. Per-plate editing stays available for the delivery that is
/// genuinely mixed, which is the case that made these fields per-plate rather than per-set.
pub fn set_image_delivery_details(
    conn: &Connection,
    well_id: &str,
    dataset: &str,
    fov_um: Option<f32>,
    prepared: Option<&str>,
    stain: Option<&str>,
) -> DbResult<usize> {
    Ok(conn.execute(
        &format!(
            "UPDATE well_images AS i SET fov_um = ?3, prepared = ?4, stain = ?5
             WHERE i.well_id = ?1 AND i.dataset = ?2 AND i.set_name = {ACTIVE_IMAGE_SET}"
        ),
        params![well_id, dataset, fov_um, prepared, stain],
    )?)
}

/// Moves every picture of one dataset's ACTIVE delivery (or of every dataset, when `dataset` is
/// None) by a constant depth — the plate equivalent of `shift_core_depths`, for a delivery whose
/// depths were all read off the same mis-registered tally.
///
/// One statement rather than N round trips: a core-photograph delivery is routinely hundreds of
/// plates, and `update_well_image` per plate would be hundreds of IPC calls to apply one decision.
///
/// `depth_base + delta` is NULL-safe in SQL, which is the point: **a plate with no base is a POINT
/// sample and must stay one.** A thin section is cut from a plug and has no thickness (see the
/// `well_images` note); a shift may move it but must never give it one.
pub fn shift_well_images(
    conn: &Connection,
    well_id: &str,
    dataset: Option<&str>,
    delta: f32,
) -> DbResult<usize> {
    Ok(conn.execute(
        &format!(
            "UPDATE well_images AS i SET depth_top = i.depth_top + ?2, depth_base = i.depth_base + ?2
             WHERE i.well_id = ?1 AND (?3 IS NULL OR i.dataset = ?3) AND i.set_name = {ACTIVE_IMAGE_SET}"
        ),
        params![well_id, delta, dataset],
    )?)
}

// ---------------------------------------------------------------------------
// Trained ML models
// ---------------------------------------------------------------------------

/// A saved model's record WITHOUT its bytes. Listing every model must stay cheap — a random
/// forest is megabytes, and the picker only ever needs the description.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MlModelInfo {
    pub model_id: String,
    pub name: String,
    pub task: String,
    pub algorithm: String,
    /// ORDERED — the order is part of the apply contract, not a display detail.
    pub feature_curves: Vec<String>,
    pub target_curve: Option<String>,
    pub params_json: String,
    pub metrics_json: String,
    pub trained_on: Vec<String>,
    pub n_train: i64,
    pub standardize: bool,
    pub sklearn_version: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
    pub bytes: i64,
    /// SB-MLA-003 — fingerprint of the exact training rows. `None` on a model saved before the
    /// column existed: "not recorded" is the truth about such a model, and it must not be
    /// confusable with a hash.
    pub train_hash: Option<String>,
    /// SB-MLA-002 + SB-MLA-004 — the per-well training roster (JSON array of
    /// [`crate::ml::TrainWellRecord`]). `None` on a model saved before it existed.
    pub training_json: Option<String>,
    /// SB-MLA-005 — the runtime that fitted and serialised the artifact (JSON object). `None` on a
    /// model saved before it existed, which is why the apply-side check says "not recorded" rather
    /// than reporting a mismatch it cannot actually see.
    pub runtime_json: Option<String>,
}

fn json_names(s: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(s).unwrap_or_default()
}

/// A model must be citable, so its name must be unique. Same rule as a delivery set: an
/// existing name is auto-suffixed rather than overwritten — retraining produces a NEW model,
/// and silently replacing the one a delivered curve was made with would destroy its provenance.
pub fn resolve_model_name(conn: &Connection, desired: &str) -> DbResult<String> {
    let base = desired.trim();
    let base = if base.is_empty() { "MODEL" } else { base };
    let taken: Vec<String> = {
        let mut stmt = conn.prepare("SELECT name FROM ml_models")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    if !taken.iter().any(|t| t.eq_ignore_ascii_case(base)) {
        return Ok(base.to_string());
    }
    for i in 1..10_000 {
        let candidate = format!("{base}_{i}");
        if !taken.iter().any(|t| t.eq_ignore_ascii_case(&candidate)) {
            return Ok(candidate);
        }
    }
    Ok(format!("{base}_{}", Uuid::new_v4()))
}

#[allow(clippy::too_many_arguments)]
/// Everything a saved model records, as one named value.
///
/// A struct rather than the positional argument list this used to be. Fifteen parameters had grown
/// to include two ADJACENT `&str` JSON blobs — `params_json` and `metrics_json` — so transposing
/// them compiled, ran, and produced a model whose recorded settings were its scores. That is the
/// silent-wrongness class exactly: nothing downstream can catch it, and the whole point of these
/// fields is that somebody will one day read them to answer a question about a delivered curve.
/// Named fields make the transposition impossible rather than unlikely, and let SB-MLA-002/004/005
/// be added without lengthening a list nobody can check by eye.
pub struct NewMlModel<'a> {
    pub name: &'a str,
    pub task: &'a str,
    pub algorithm: &'a str,
    /// ORDERED — the order is part of the apply contract.
    pub feature_curves: &'a [String],
    pub target_curve: Option<&'a str>,
    pub params_json: &'a str,
    pub metrics_json: &'a str,
    pub trained_on: &'a [String],
    pub n_train: usize,
    pub standardize: bool,
    pub note: Option<&'a str>,
    pub data: &'a [u8],
    /// SB-MLA-003 — fingerprint of the exact training matrix. `None` only where the caller genuinely
    /// cannot compute one; a blank string is never stored, because "not recorded" and "hashed to
    /// nothing" must stay distinguishable.
    pub train_hash: Option<&'a str>,
    /// SB-MLA-002 + SB-MLA-004 — the per-well training roster: what each well contributed, which log
    /// set it was read from, and how many of its samples the mask removed. JSON array.
    pub training_json: Option<&'a str>,
    /// SB-MLA-005 — the interpreter and every library that participated in fitting or serialising
    /// the artifact. JSON object.
    pub runtime_json: Option<&'a str>,
    /// Kept as its own column because it predates `runtime_json` and readers select it by name; it
    /// is the one runtime component the artifact cannot be loaded without.
    pub sklearn_version: Option<&'a str>,
}

pub fn insert_ml_model(conn: &Connection, m: &NewMlModel<'_>) -> DbResult<(String, String)> {
    let name = resolve_model_name(conn, m.name)?;
    let id = Uuid::new_v4().to_string();
    // A fn, not a closure: a closure would infer one lifetime for every call and the three
    // arguments borrow from different places.
    fn blank_to_none(s: Option<&str>) -> Option<&str> {
        s.map(str::trim).filter(|v| !v.is_empty())
    }
    conn.execute(
        "INSERT INTO ml_models (model_id, name, task, algorithm, feature_curves, target_curve,
                                params_json, metrics_json, trained_on, n_train, standardize,
                                sklearn_version, note, data, train_hash, training_json, runtime_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        params![
            id,
            name,
            m.task,
            m.algorithm,
            serde_json::to_string(m.feature_curves).unwrap_or_else(|_| "[]".into()),
            m.target_curve,
            m.params_json,
            m.metrics_json,
            serde_json::to_string(m.trained_on).unwrap_or_else(|_| "[]".into()),
            m.n_train as i64,
            i32::from(m.standardize),
            m.sklearn_version,
            m.note,
            m.data,
            blank_to_none(m.train_hash),
            blank_to_none(m.training_json),
            blank_to_none(m.runtime_json),
        ],
    )?;
    Ok((id, name))
}

/// Every saved model, newest first. Never selects `data`.
pub fn list_ml_models(conn: &Connection) -> DbResult<Vec<MlModelInfo>> {
    let mut stmt = conn.prepare(
        "SELECT model_id, name, task, algorithm, feature_curves, target_curve, params_json,
                metrics_json, trained_on, n_train, standardize, sklearn_version, note,
                strftime(created_at, '%Y-%m-%d %H:%M'), octet_length(data), train_hash,
                training_json, runtime_json
         FROM ml_models ORDER BY created_at DESC, name",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(MlModelInfo {
            model_id: r.get(0)?,
            name: r.get(1)?,
            task: r.get(2)?,
            algorithm: r.get(3)?,
            feature_curves: json_names(&r.get::<_, String>(4)?),
            target_curve: r.get(5)?,
            params_json: r.get(6)?,
            metrics_json: r.get(7)?,
            trained_on: json_names(&r.get::<_, String>(8)?),
            n_train: r.get(9)?,
            standardize: r.get::<_, i32>(10)? != 0,
            sklearn_version: r.get(11)?,
            note: r.get(12)?,
            created_at: r.get(13)?,
            bytes: r.get(14)?,
            train_hash: r.get(15)?,
            training_json: r.get(16)?,
            runtime_json: r.get(17)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// The record AND its bytes — only the apply path asks for this.
pub fn get_ml_model(conn: &Connection, model_id: &str) -> DbResult<(MlModelInfo, Vec<u8>)> {
    let info = conn.query_row(
        "SELECT model_id, name, task, algorithm, feature_curves, target_curve, params_json,
                metrics_json, trained_on, n_train, standardize, sklearn_version, note,
                strftime(created_at, '%Y-%m-%d %H:%M'), octet_length(data), train_hash,
                training_json, runtime_json, data
         FROM ml_models WHERE model_id = ?1",
        params![model_id],
        |r| {
            Ok((
                MlModelInfo {
                    model_id: r.get(0)?,
                    name: r.get(1)?,
                    task: r.get(2)?,
                    algorithm: r.get(3)?,
                    feature_curves: json_names(&r.get::<_, String>(4)?),
                    target_curve: r.get(5)?,
                    params_json: r.get(6)?,
                    metrics_json: r.get(7)?,
                    trained_on: json_names(&r.get::<_, String>(8)?),
                    n_train: r.get(9)?,
                    standardize: r.get::<_, i32>(10)? != 0,
                    sklearn_version: r.get(11)?,
                    note: r.get(12)?,
                    created_at: r.get(13)?,
                    bytes: r.get(14)?,
                    train_hash: r.get(15)?,
                    training_json: r.get(16)?,
                    runtime_json: r.get(17)?,
                },
                r.get::<_, Vec<u8>>(18)?,
            ))
        },
    )?;
    Ok(info)
}

pub fn rename_ml_model(conn: &Connection, model_id: &str, new_name: &str) -> DbResult<String> {
    let name = resolve_model_name(conn, new_name)?;
    conn.execute("UPDATE ml_models SET name = ?2 WHERE model_id = ?1", params![model_id, name])?;
    Ok(name)
}

pub fn delete_ml_model(conn: &Connection, model_id: &str) -> DbResult<()> {
    conn.execute("DELETE FROM ml_models WHERE model_id = ?1", params![model_id])?;
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

/// SB-DBM-031 (DEC-073 item 5, RULED): the delivery-set stores that carry a per-SET
/// depth datum. `true` marks the two whose sets are keyed by (well, dataset, set).
const SET_DATUM_STORES: &[(&str, bool)] = &[
    ("core_sets", false),
    ("aux_sets", true),
    ("scal_sets", false),
    ("image_sets", true),
];

fn set_datum_store(store: &str) -> DbResult<bool> {
    SET_DATUM_STORES
        .iter()
        .find(|(name, _)| *name == store)
        .map(|(_, has_dataset)| *has_dataset)
        .ok_or_else(|| DbError::Invalid(format!("'{store}' is not a datum-bearing delivery store")))
}

/// Declare the datum of ONE registered delivery set. The token is validated against the
/// shipped vocabulary; an unknown token refuses naming it, and a set that does not exist
/// refuses rather than silently declaring nothing (SB-DBM-031).
pub fn declare_set_datum(
    conn: &Connection,
    store: &str,
    well_id: &str,
    dataset: Option<&str>,
    set_name: &str,
    datum: &str,
) -> DbResult<()> {
    let has_dataset = set_datum_store(store)?;
    let datum = crate::schema_vocab::DepthDatum::parse(datum)
        .ok_or_else(|| {
            DbError::Invalid(format!(
                "'{datum}' is not a depth datum; the vocabulary is MD | TVD | TVDSS | TVDKB | TWT | OWT | CDEPTH (SB-DBM-031)"
            ))
        })?
        .as_str();
    let n = if has_dataset {
        let dataset = dataset.ok_or_else(|| {
            DbError::Invalid(format!("'{store}' sets are keyed by dataset; none was named"))
        })?;
        conn.execute(
            &format!(
                "UPDATE {store} SET depth_datum = ?4 WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3"
            ),
            params![well_id, dataset, set_name, datum],
        )?
    } else {
        conn.execute(
            &format!("UPDATE {store} SET depth_datum = ?3 WHERE well_id = ?1 AND set_name = ?2"),
            params![well_id, set_name, datum],
        )?
    };
    if n == 0 {
        return Err(DbError::Invalid(format!(
            "cannot declare a datum on '{set_name}': no such {store} delivery is registered"
        )));
    }
    Ok(())
}

/// SB-DBM-031's comparison guard, shared by every depth-pairing reader of the four
/// delivery stores so the refusal text cannot drift. The log frame is MD; an ACTIVE
/// delivery whose DECLARED datum differs is refused NAMING BOTH datums - comparing an MD
/// log depth with, say, a TVDSS plug depth is a category error that silently produces a
/// number (F-17). A legacy set with no declaration (NULL) is the preserved unknown and
/// passes exactly as it always did; refusing it would relabel unknown as wrong.
pub(crate) fn refuse_non_md_active_set(
    conn: &Connection,
    store: &str,
    well_id: &str,
    dataset: Option<&str>,
) -> DbResult<()> {
    let has_dataset = set_datum_store(store)?;
    let mut stmt = if has_dataset {
        match dataset {
            Some(_) => conn.prepare(&format!(
                "SELECT dataset, set_name, depth_datum FROM {store} \
                 WHERE well_id = ?1 AND active = 1 AND dataset = ?2 AND depth_datum IS NOT NULL"
            ))?,
            None => conn.prepare(&format!(
                "SELECT dataset, set_name, depth_datum FROM {store} \
                 WHERE well_id = ?1 AND active = 1 AND depth_datum IS NOT NULL"
            ))?,
        }
    } else {
        conn.prepare(&format!(
            "SELECT '' AS dataset, set_name, depth_datum FROM {store} \
             WHERE well_id = ?1 AND active = 1 AND depth_datum IS NOT NULL"
        ))?
    };
    let rows: Vec<(String, String, String)> = if has_dataset && dataset.is_some() {
        stmt.query_map(params![well_id, dataset.unwrap()], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .collect::<Result<_, _>>()?
    } else {
        stmt.query_map(params![well_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<Result<_, _>>()?
    };
    for (dataset, set_name, datum) in rows {
        if datum != crate::schema_vocab::DepthDatum::Md.as_str() {
            let label = if dataset.is_empty() {
                set_name
            } else {
                format!("{dataset}/{set_name}")
            };
            return Err(DbError::Invalid(format!(
                "cross-datum comparison refused: active {store} delivery '{label}' declares \
                 datum {datum} but the log frame is MD - import the delivery on MD or \
                 convert it before pairing (SB-DBM-031)"
            )));
        }
    }
    Ok(())
}

/// One well's capillary-pressure points, from the ACTIVE SCAL delivery.
pub fn get_scal_pc(conn: &Connection, well_id: &str) -> DbResult<Vec<ScalPcRow>> {
    refuse_non_md_active_set(conn, "scal_sets", well_id, None)?;
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
///
/// Every caller pairs these depths against an MD log frame — HFU/FZI clustering and the facies
/// core-permeability tie both find the nearest log sample to a plug — so the datum guard applies
/// here exactly as it does to the sibling readers.
pub fn get_core_plugs(conn: &Connection, well_id: &str) -> DbResult<Vec<CorePlugRow>> {
    refuse_non_md_active_set(conn, "core_sets", well_id, None)?;
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
    refuse_non_md_active_set(conn, "core_sets", well_id, None)?;
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
///
/// SandiMin ties each plug to the nearest solved sample, which sits on the MD log frame, so the
/// datum guard applies here too.
pub fn get_core_por_gd(conn: &Connection, well_id: &str) -> DbResult<Vec<CoreQcRow>> {
    refuse_non_md_active_set(conn, "core_sets", well_id, None)?;
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
/// Kept for project-wide diagnostics and integration fixtures; production IPC must use
/// `list_wells_by_ids` after the backend resolves or declares scope.
#[allow(dead_code)]
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

/// Lists only the backend-authorized wells. The `IN` predicate is deliberate: resolving a
/// 12-well group and then loading all 540 summaries before filtering would preserve the original
/// SB-DBM-037 defect behind a correctly scoped id list.
pub fn list_wells_by_ids(conn: &Connection, well_ids: &[String]) -> DbResult<Vec<WellSummary>> {
    if well_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = std::iter::repeat("?")
        .take(well_ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT well_id, well_name, field_name, td, kb, surface_x, surface_y, utm_zone
         FROM wells WHERE well_id IN ({placeholders}) ORDER BY well_name, well_id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(well_ids.iter()), |row| {
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
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TopEntry {
    pub top_name: String,
    /// Depth on the MD frame consumed by the existing log, correlation and zone surfaces.
    pub depth: f32,
    /// Delivered value retained without rewriting, with its declared source reference.
    pub source_depth: f32,
    pub source_depth_datum: Option<crate::schema_vocab::DepthDatum>,
    pub color: Option<String>,
}

/// Lists the formation tops for one well, ordered by depth (a formation-tops
/// equivalent — the Tops panel's data source). A TVD source is converted to MD only through
/// the active deviation survey; without that frame the MD consumer is refused by name.
pub fn list_tops(conn: &Connection, well_id: &str) -> DbResult<Vec<TopEntry>> {
    let mut stmt = conn.prepare(
        "SELECT top_name, depth, depth_datum, color FROM tops WHERE well_id = ?1 ORDER BY depth",
    )?;
    let rows = stmt.query_map(params![well_id], |row| {
        let raw_datum: Option<String> = row.get(2)?;
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, f32>(1)?,
            raw_datum,
            row.get::<_, Option<String>>(3)?,
        ))
    })?;
    let mut raw_tops = Vec::new();
    for r in rows {
        raw_tops.push(r?);
    }

    let needs_tvd_frame = raw_tops
        .iter()
        .any(|(_, _, datum, _)| datum.as_deref().and_then(crate::schema_vocab::DepthDatum::parse)
            == Some(crate::schema_vocab::DepthDatum::Tvd));
    let survey = if needs_tvd_frame {
        let path = get_well_path(conn, well_id)?;
        if path.is_empty() {
            return Err(DbError::Invalid(
                "TVD-referenced tops cannot be plotted, joined or compared on an MD log: the well has no active deviation survey"
                    .into(),
            ));
        }
        Some(path)
    } else {
        None
    };

    let mut tops = Vec::with_capacity(raw_tops.len());
    for (top_name, source_depth, raw_datum, color) in raw_tops {
        let source_depth_datum = match raw_datum.as_deref() {
            Some(value) => Some(crate::schema_vocab::DepthDatum::parse(value).ok_or_else(|| {
                DbError::Invalid(format!("formation top '{top_name}' has unknown depth datum '{value}'"))
            })?),
            None => None,
        };
        let depth = match source_depth_datum {
            Some(crate::schema_vocab::DepthDatum::Md) | None => source_depth,
            Some(crate::schema_vocab::DepthDatum::Tvd) => md_at_tvd(
                survey.as_deref().expect("TVD rows require a loaded survey"),
                source_depth,
                &top_name,
            )?,
            Some(other) => {
                return Err(DbError::Invalid(format!(
                    "formation top '{top_name}' is {}-referenced and cannot be used on an MD log without an implemented reference transform",
                    other.as_str()
                )))
            }
        };
        tops.push(TopEntry { top_name, depth, source_depth, source_depth_datum, color });
    }
    tops.sort_by(|left, right| left.depth.total_cmp(&right.depth));
    Ok(tops)
}

fn md_at_tvd(stations: &[WellPathStation], tvd: f32, top_name: &str) -> DbResult<f32> {
    let mut candidates = stations
        .iter()
        .filter(|station| station.tvd == tvd)
        .map(|station| station.md)
        .collect::<Vec<_>>();
    for pair in stations.windows(2) {
        let (left, right) = (&pair[0], &pair[1]);
        if left.tvd == right.tvd {
            continue;
        }
        let inside = (tvd > left.tvd && tvd < right.tvd) || (tvd < left.tvd && tvd > right.tvd);
        if inside {
            let fraction = (tvd - left.tvd) / (right.tvd - left.tvd);
            candidates.push(left.md + fraction * (right.md - left.md));
        }
    }
    candidates.sort_by(f32::total_cmp);
    candidates.dedup();
    match candidates.as_slice() {
        [md] => Ok(*md),
        [] => Err(DbError::Invalid(format!(
            "TVD-referenced top '{top_name}' at {tvd} cannot be placed on the MD log because the active deviation survey does not cover that TVD"
        ))),
        _ => Err(DbError::Invalid(format!(
            "TVD-referenced top '{top_name}' at {tvd} cannot be placed uniquely on the MD log by the active deviation survey"
        ))),
    }
}

/// Upserts a formation top by (well_id, top_name).
pub fn upsert_top(conn: &Connection, well_id: &str, top_name: &str, depth: f32, color: Option<&str>) -> DbResult<()> {
    let existing_datum: Option<Option<String>> = conn
        .query_row(
            "SELECT depth_datum FROM tops WHERE well_id = ?1 AND top_name = ?2",
            params![well_id, top_name],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(Some(existing_datum)) = existing_datum {
        if existing_datum != crate::schema_vocab::DepthDatum::Md.as_str() {
            return Err(DbError::Invalid(format!(
                "{existing_datum}-referenced top '{top_name}' cannot be rewritten by the MD tops editor; re-import it with an explicit source reference"
            )));
        }
    }
    upsert_top_with_datum(
        conn,
        well_id,
        top_name,
        depth,
        crate::schema_vocab::DepthDatum::Md,
        color,
    )
}

/// Upserts a formation top while retaining the reference declared by its source.
pub fn upsert_top_with_datum(
    conn: &Connection,
    well_id: &str,
    top_name: &str,
    depth: f32,
    depth_datum: crate::schema_vocab::DepthDatum,
    color: Option<&str>,
) -> DbResult<()> {
    conn.execute(
        "INSERT INTO tops (well_id, top_name, depth, depth_datum, color) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (well_id, top_name) DO UPDATE SET depth = excluded.depth,
             depth_datum = excluded.depth_datum,
             color = COALESCE(excluded.color, tops.color)",
        params![well_id, top_name, depth, depth_datum.as_str(), color],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct ZoneEntry {
    pub zone_name: String,
    pub top_depth: f32,
    pub bottom_depth: f32,
    pub depth_datum: crate::schema_vocab::DepthDatum,
}

pub fn list_zones(conn: &Connection, well_id: &str) -> DbResult<Vec<ZoneEntry>> {
    let mut stmt = conn.prepare(
        "SELECT zone_name, top_depth, bottom_depth, depth_datum
         FROM zones WHERE well_id = ?1 ORDER BY top_depth",
    )?;
    let rows = stmt.query_map(params![well_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, f32>(1)?,
            row.get::<_, f32>(2)?,
            row.get::<_, Option<String>>(3)?,
        ))
    })?;
    let mut zones = Vec::new();
    for r in rows {
        let (zone_name, top_depth, bottom_depth, datum) = r?;
        let datum = datum.ok_or_else(|| {
            DbError::Invalid(format!(
                "zone '{zone_name}' has no declared depth datum; assign one before reading or comparing it"
            ))
        })?;
        let depth_datum = crate::schema_vocab::DepthDatum::parse(&datum).ok_or_else(|| {
            DbError::Invalid(format!("zone '{zone_name}' has unsupported depth datum '{datum}'"))
        })?;
        zones.push(ZoneEntry { zone_name, top_depth, bottom_depth, depth_datum });
    }
    Ok(zones)
}

/// Explicit measured-depth convenience for writers whose input is already on the standard MD
/// reference. The name is intentionally not datum-neutral: callers must not use it for an
/// unclassified legacy depth.
pub fn upsert_md_zone(conn: &Connection, well_id: &str, zone_name: &str, top_depth: f32, bottom_depth: f32) -> DbResult<()> {
    upsert_zone_with_datum(
        conn,
        well_id,
        zone_name,
        top_depth,
        bottom_depth,
        crate::schema_vocab::DepthDatum::Md,
    )
}

pub fn upsert_zone_with_datum(
    conn: &Connection,
    well_id: &str,
    zone_name: &str,
    top_depth: f32,
    bottom_depth: f32,
    depth_datum: crate::schema_vocab::DepthDatum,
) -> DbResult<()> {
    conn.execute(
        "INSERT INTO zones (well_id, zone_name, top_depth, bottom_depth, depth_datum)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (well_id, zone_name) DO UPDATE SET
             top_depth = excluded.top_depth,
             bottom_depth = excluded.bottom_depth,
             depth_datum = excluded.depth_datum",
        params![well_id, zone_name, top_depth, bottom_depth, depth_datum.as_str()],
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
    pub depth_datum: crate::schema_vocab::DepthDatum,
    pub is_tvdss: bool,
    pub color: Option<String>,
    pub label: Option<String>,
    /// The fault block or segment this contact belongs to. `None` = not stated.
    pub compartment: Option<String>,
    /// The markers this contact governs, sorted. EMPTY = no marker stated, which is a real answer:
    /// a field-wide datum cuts across markers. SEVERAL = stacked sands in one hydraulic unit
    /// sharing one contact, which a single column could not express.
    pub zones: Vec<String>,
}

/// Shared loader for either every project contact or only contacts owned by a backend-authorized
/// well set. The scoped branch constrains both contact rows and marker links in SQL.
fn list_fluid_contacts_scoped(
    conn: &Connection,
    well_ids: Option<&[String]>,
) -> DbResult<Vec<FluidContact>> {
    if well_ids.is_some_and(|ids| ids.is_empty()) {
        return Ok(Vec::new());
    }
    let placeholders = well_ids.map(|ids| {
        std::iter::repeat("?").take(ids.len()).collect::<Vec<_>>().join(", ")
    });
    let where_clause = placeholders
        .as_ref()
        .map(|items| format!(" WHERE well_id IN ({items})"))
        .unwrap_or_default();
    let contact_sql = format!(
        "SELECT contact_id, field_name, well_id, contact_type, depth, depth_datum, color, label, compartment
         FROM fluid_contacts{where_clause} ORDER BY depth"
    );
    let mut stmt = conn.prepare(&contact_sql)?;
    let mut read_row = |row: &duckdb::Row<'_>| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, f64>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, Option<String>>(8)?,
        ))
    };
    let rows = match well_ids {
        Some(ids) => stmt.query_map(params_from_iter(ids.iter()), &mut read_row)?,
        None => stmt.query_map([], &mut read_row)?,
    };
    let mut contacts = Vec::new();
    for r in rows {
        let (contact_id, field_name, well_id, contact_type, depth, datum, color, label, compartment) = r?;
        let depth_datum = crate::schema_vocab::DepthDatum::parse(&datum).ok_or_else(|| {
            DbError::Invalid(format!("contact '{contact_id}' has unsupported depth datum '{datum}'"))
        })?;
        contacts.push(FluidContact {
            contact_id,
            field_name,
            well_id,
            contact_type,
            depth,
            depth_datum,
            is_tvdss: depth_datum == crate::schema_vocab::DepthDatum::Tvdss,
            color,
            label,
            compartment,
            zones: Vec::new(),
        });
    }
    // One scan of the link table rather than a query per contact: there are few contacts, but a
    // per-row query is how a list turns into N round trips on a field-scale project.
    let zone_where = placeholders
        .as_ref()
        .map(|items| {
            format!(
                " WHERE contact_id IN (SELECT contact_id FROM fluid_contacts WHERE well_id IN ({items}))"
            )
        })
        .unwrap_or_default();
    let zone_sql = format!(
        "SELECT contact_id, zone_name FROM contact_zones{zone_where} ORDER BY zone_name"
    );
    let mut zstmt = conn.prepare(&zone_sql)?;
    let mut read_link = |row: &duckdb::Row<'_>| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    };
    let links = match well_ids {
        Some(ids) => zstmt.query_map(params_from_iter(ids.iter()), &mut read_link)?,
        None => zstmt.query_map([], &mut read_link)?,
    };
    let mut by_id: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for l in links {
        let (id, zone) = l?;
        by_id.entry(id).or_default().push(zone);
    }
    for c in &mut contacts {
        if let Some(z) = by_id.remove(&c.contact_id) {
            c.zones = z;
        }
    }
    Ok(contacts)
}

/// Every fluid contact in the project. There are few of these (one per reservoir/field),
/// so the correlation view fetches them all and decides per well which apply.
pub fn list_fluid_contacts(conn: &Connection) -> DbResult<Vec<FluidContact>> {
    list_fluid_contacts_scoped(conn, None)
}

/// Only contacts owned by the backend-authorized wells. Project/field contacts with no well id
/// are deliberately absent: cross-well consistency and FWL agreement operate on well picks only.
pub fn list_fluid_contacts_for_wells(
    conn: &Connection,
    well_ids: &[String],
) -> DbResult<Vec<FluidContact>> {
    list_fluid_contacts_scoped(conn, Some(well_ids))
}

#[allow(clippy::too_many_arguments)]
pub fn upsert_fluid_contact_with_datum(
    conn: &Connection,
    contact_id: &str,
    field_name: Option<&str>,
    well_id: Option<&str>,
    contact_type: &str,
    depth: f64,
    depth_datum: crate::schema_vocab::DepthDatum,
    color: Option<&str>,
    label: Option<&str>,
    compartment: Option<&str>,
    zones: &[String],
) -> DbResult<()> {
    let is_tvdss = depth_datum == crate::schema_vocab::DepthDatum::Tvdss;
    conn.execute(
        "INSERT INTO fluid_contacts
             (contact_id, field_name, well_id, contact_type, depth, is_tvdss, depth_datum, color, label, compartment)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT (contact_id) DO UPDATE SET
             field_name = excluded.field_name, well_id = excluded.well_id,
             contact_type = excluded.contact_type, depth = excluded.depth,
             is_tvdss = excluded.is_tvdss, depth_datum = excluded.depth_datum,
             color = excluded.color, label = excluded.label,
             compartment = excluded.compartment",
        params![
            contact_id,
            field_name,
            well_id,
            contact_type,
            depth,
            is_tvdss,
            depth_datum.as_str(),
            color,
            label,
            compartment
        ],
    )?;
    // Replace the marker links wholesale. An upsert that only ADDED would make removing a marker
    // impossible, and a contact silently governing a sand the user took it off is the same class
    // of error as a parameter that cannot be cleared.
    conn.execute("DELETE FROM contact_zones WHERE contact_id = ?1", params![contact_id])?;
    for z in zones {
        let z = z.trim();
        if z.is_empty() {
            continue;
        }
        conn.execute(
            "INSERT INTO contact_zones (contact_id, zone_name) VALUES (?1, ?2)
             ON CONFLICT (contact_id, zone_name) DO NOTHING",
            params![contact_id, z],
        )?;
    }
    Ok(())
}

pub fn delete_fluid_contact(conn: &Connection, contact_id: &str) -> DbResult<()> {
    conn.execute("DELETE FROM contact_zones WHERE contact_id = ?1", params![contact_id])?;
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
            upsert_md_zone(conn, well_id, &top.top_name, top.depth, bottom)?;
            zones.push(ZoneEntry {
                zone_name: top.top_name.clone(),
                top_depth: top.depth,
                bottom_depth: bottom,
                depth_datum: crate::schema_vocab::DepthDatum::Md,
            });
        }
        Ok(zones)
    })
}

// ---------------------------------------------------------------------------
// Database inspector (spreadsheet-grid equivalent): paged reads over a whitelist
// of tables + explicit single-cell update commands. The frontend never sends
// SQL — table and column names are validated against these specs.
// ---------------------------------------------------------------------------

struct TableSpec {
    table: &'static str,
    columns: Vec<&'static str>,
    well_scoped: bool,
    order: &'static str,
}

fn table_specs() -> Vec<TableSpec> {
    vec![
        TableSpec {
            table: "wells",
            columns: vec!["well_id", "well_name", "field_name", "td", "kb"],
            well_scoped: false,
            order: "well_name",
        },
        TableSpec {
            table: "standard_curves",
            columns: crate::schema_vocab::standard_projections().inspector_columns,
            well_scoped: true,
            order: crate::schema_vocab::STANDARD_COLUMNS[0].storage_column,
        },
        TableSpec {
            table: "computed_curves",
            columns: vec!["depth", "curve_name", "value"],
            well_scoped: true,
            order: "curve_name, depth",
        },
        TableSpec {
            table: "tops",
            columns: vec!["top_name", "depth", "depth_datum", "color"],
            well_scoped: true,
            order: "depth",
        },
        TableSpec {
            table: "zones",
            // depth_datum must be on the page: the Inspector's zone edit re-reads the row's own
            // datum to re-declare it on write, so a page without the column refuses every edit.
            columns: vec!["zone_name", "top_depth", "bottom_depth", "depth_datum"],
            well_scoped: true,
            order: "top_depth",
        },
        TableSpec {
            table: "zone_params",
            columns: vec!["zone_name", "param_name", "value_num", "value_text"],
            well_scoped: true,
            order: "zone_name, param_name",
        },
        // set_name is listed (read-only, like every non-editable column) so a well carrying
        // several core deliveries can be told apart in the grid; edits still target the
        // ACTIVE set only (see `update_core_sample`).
        TableSpec {
            table: "core_data",
            columns: vec!["set_name", "depth", "cpor", "cperm", "cgd", "csw"],
            well_scoped: true,
            order: "set_name, depth",
        },
        TableSpec {
            table: "aux_data",
            columns: vec![
                "dataset",
                "depth_top",
                "depth_base",
                "item",
                "value_num",
                "value_text",
            ],
            well_scoped: true,
            order: "dataset, depth_top, item",
        },
        // SB-DBM-041 T42 (2026-08-19): the provenance and audit registry is BROWSABLE.
        // These grids are read-only by construction - inspector writes go through the
        // explicit per-table commands (rule 6) and none exists for any of them: an audit
        // or provenance row edited in a grid is a falsified record. `ml_models`
        // deliberately omits `data` - the joblib blob follows the same never-select rule
        // as `list_ml_models`.
        TableSpec {
            table: "log_sets",
            columns: vec![
                "set_id", "well_id", "set_name", "version", "module", "params_json",
                "inputs_json", "created_at", "frame", "sampling_style",
                "duplicate_resolution", "outcome_state", "comment", "applied_steps_json",
            ],
            well_scoped: true,
            order: "set_name, version",
        },
        TableSpec {
            table: "audit_entry",
            // NOT well-scoped: well_id is nullable and project-level gestures carry none,
            // so a mandatory well filter would hide exactly the entries it exists to show.
            columns: vec![
                "entry_id", "entry_seq", "well_id", "ts_utc", "operator", "operator_kind",
                "view", "source", "comment", "zone_set_version", "zone_set_digest",
                "repeat_count",
            ],
            well_scoped: false,
            order: "entry_seq",
        },
        TableSpec {
            table: "audit_detail",
            columns: vec!["entry_id", "seq", "location", "mode", "unit", "name", "value"],
            well_scoped: false,
            order: "entry_id, seq",
        },
        TableSpec {
            table: "zone_set_versions",
            columns: vec!["well_id", "version", "digest", "created_at"],
            well_scoped: true,
            order: "version",
        },
        TableSpec {
            table: "run_parameters",
            columns: vec![
                "set_id", "position", "name", "value_json", "source", "state", "resolution",
                "manifest_version",
            ],
            well_scoped: false,
            order: "set_id, position",
        },
        TableSpec {
            table: "run_degradations",
            columns: vec!["set_id", "position", "module", "kind", "detail", "occurrences"],
            well_scoped: false,
            order: "set_id, position",
        },
        TableSpec {
            table: "computed_curves_archive",
            columns: vec!["set_id", "well_id", "depth", "curve_name", "value"],
            well_scoped: true,
            order: "set_id, curve_name, depth",
        },
        TableSpec {
            table: "curve_meta",
            columns: vec![
                "curve_id", "well_id", "set_name", "mnemonic", "unit", "family", "source",
                "run_no", "pinned", "set_version", "final_flag", "neutron_basis",
                "neutron_basis_source",
            ],
            well_scoped: true,
            order: "set_name, mnemonic, set_version",
        },
        TableSpec {
            table: "ml_models",
            columns: vec![
                "model_id", "name", "task", "algorithm", "feature_curves", "target_curve",
                "params_json", "metrics_json", "trained_on", "n_train", "standardize",
                "sklearn_version", "note", "created_at", "train_hash", "training_json",
                "runtime_json",
            ],
            well_scoped: false,
            order: "name",
        },
    ]
}

#[derive(Debug, Serialize)]
pub struct TablePage {
    pub columns: Vec<String>,
    /// Cells stringified by DuckDB's VARCHAR cast; None = SQL NULL.
    pub rows: Vec<Vec<Option<String>>>,
    pub total_rows: usize,
    /// Always false on this inspector path: `total_rows` is a real COUNT(*), separate from the
    /// number of rows returned in this page.
    pub truncated: bool,
}

/// SQL-console response. Deliberately not [`TablePage`]: `returned_rows` is the page size after
/// the cap, never the inspector's true total, and `count_is_total = false` states that distinction
/// on the wire instead of relying on explanatory UI text.
#[derive(Debug, Serialize)]
pub struct QueryPage {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Option<String>>>,
    pub returned_rows: usize,
    pub count_is_total: bool,
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
    let specs = table_specs();
    let spec = specs
        .iter()
        .find(|spec| spec.table == table)
        .ok_or_else(|| format!("unknown table '{table}'"))?;
    let columns = &spec.columns;
    let well_scoped = spec.well_scoped;
    let order = spec.order;
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

pub const INTEGRITY_CURRENT_LOG_SET_CLASS: &str = "computed_curves_missing_log_set";
pub const INTEGRITY_ARCHIVE_LOG_SET_CLASS: &str = "computed_curves_archive_missing_log_set";
pub const INTEGRITY_WELL_GROUP_MEMBER_CLASS: &str = "well_group_members_missing_well";
pub const INTEGRITY_CURVE_SAMPLE_CLASS: &str = "curve_samples_missing_curve_meta";
pub const INTEGRITY_ML_TRAINING_WELL_CLASS: &str = "ml_models_unresolved_training_wells";
pub const INTEGRITY_CURRENT_DUPLICATE_CLASS: &str = "computed_curves_duplicate_depths";
pub const INTEGRITY_ARCHIVE_DUPLICATE_CLASS: &str = "computed_curves_archive_duplicate_depths";

/// AUDIT-2026-08-20 finding 52. One prunable integrity class, described ONCE.
///
/// Quarantine, restore and redo used to hand-transcribe each class's SQL in four separate
/// functions - seven statements per class, with the IDENTITY KEY written out four times and no
/// database constraint to catch a slip. `computed_curves` is deliberately primary-key-less (its
/// uniqueness is upheld by the write discipline, not by an index), so a mistyped identity here
/// has nothing underneath it: a restore would silently duplicate rows, or a redo would delete
/// rows it never quarantined. That is not a class of bug this code path can afford.
///
/// So the identity is stated once and every statement is built from it. `{live}` is substituted
/// with the live table's name, because a `DELETE` names the table where a `SELECT` uses an alias.
struct PrunableClass {
    class_id: &'static str,
    /// The live table rows are quarantined FROM and restored back INTO.
    live: &'static str,
    /// The quarantine table for this class.
    quarantine: &'static str,
    /// `source_table` discriminator where two classes share one quarantine table; `None` where
    /// the quarantine table serves a single class.
    source_tag: Option<&'static str>,
    /// The columns carried into quarantine and back out, in one order for both directions.
    columns: &'static str,
    /// What makes a live row the SAME ROW as a quarantined one. Written once, read by the
    /// restore collision check, the redo liveness check and the redo delete.
    identity: &'static str,
    /// What makes a live row an ORPHAN - the reason it is prunable at all.
    orphan: &'static str,
}

impl PrunableClass {
    /// The quarantine-side filter: this batch, and this class's rows within it.
    fn batch_filter(&self) -> String {
        match self.source_tag {
            Some(tag) => format!("q.batch_id = ?1 AND q.source_table = '{tag}'"),
            None => "q.batch_id = ?1".to_string(),
        }
    }

    /// The same filter for a statement with no `q` alias in scope.
    fn plain_batch_filter(&self) -> String {
        match self.source_tag {
            Some(tag) => format!("batch_id = ?1 AND source_table = '{tag}'"),
            None => "batch_id = ?1".to_string(),
        }
    }

    /// `EXISTS (... this batch's quarantined twin of `live_ref`)`, the one identity comparison.
    fn quarantined_twin_exists(&self, live_ref: &str) -> String {
        format!(
            "EXISTS (SELECT 1 FROM {} q WHERE {} AND {})",
            self.quarantine,
            self.batch_filter(),
            self.identity.replace("{live}", live_ref)
        )
    }
}

/// The four prunable classes. `PRUNABLE_INTEGRITY_CLASSES` is derived from this, so the offered
/// list and the executed list cannot disagree.
const PRUNABLE_CLASSES: [PrunableClass; 4] = [
    PrunableClass {
        class_id: INTEGRITY_CURRENT_LOG_SET_CLASS,
        live: "computed_curves",
        quarantine: "integrity_quarantine_computed",
        source_tag: Some("computed_curves"),
        columns: "set_id, well_id, depth, curve_name, value",
        // SB-DBM-026's identity for a CURRENT row. `set_id` is deliberately absent: a current row
        // is one interpretation, so well+curve+depth names it.
        identity: "q.well_id = {live}.well_id AND q.curve_name = {live}.curve_name                    AND q.depth = {live}.depth",
        // Legacy NULL set_id rows are reported but never quarantined - they are labelled and
        // visible, not broken references.
        orphan: "{live}.set_id IS NOT NULL                  AND NOT EXISTS (SELECT 1 FROM log_sets l WHERE l.set_id = {live}.set_id)",
    },
    PrunableClass {
        class_id: INTEGRITY_ARCHIVE_LOG_SET_CLASS,
        live: "computed_curves_archive",
        quarantine: "integrity_quarantine_computed",
        source_tag: Some("computed_curves_archive"),
        columns: "set_id, well_id, depth, curve_name, value",
        // Versions legitimately repeat a tuple across set_id values, so an ARCHIVE row's identity
        // includes the set - the difference from the current class above, and the reason the two
        // identities must not be shared.
        identity: "q.set_id = {live}.set_id AND q.well_id = {live}.well_id                    AND q.curve_name = {live}.curve_name AND q.depth = {live}.depth",
        orphan: "NOT EXISTS (SELECT 1 FROM log_sets l WHERE l.set_id = {live}.set_id)",
    },
    PrunableClass {
        class_id: INTEGRITY_WELL_GROUP_MEMBER_CLASS,
        live: "well_group_members",
        quarantine: "integrity_quarantine_group_members",
        source_tag: None,
        columns: "group_id, well_id",
        identity: "q.group_id = {live}.group_id AND q.well_id = {live}.well_id",
        orphan: "NOT EXISTS (SELECT 1 FROM wells w WHERE w.well_id = {live}.well_id)",
    },
    PrunableClass {
        class_id: INTEGRITY_CURVE_SAMPLE_CLASS,
        live: "curve_samples",
        quarantine: "integrity_quarantine_curve_samples",
        source_tag: None,
        columns: "curve_id, depth, value",
        identity: "q.curve_id = {live}.curve_id AND q.depth = {live}.depth",
        orphan: "NOT EXISTS (SELECT 1 FROM curve_meta m WHERE m.curve_id = {live}.curve_id)",
    },
];

const PRUNABLE_INTEGRITY_CLASSES: [&str; 4] = [
    PRUNABLE_CLASSES[0].class_id,
    PRUNABLE_CLASSES[1].class_id,
    PRUNABLE_CLASSES[2].class_id,
    PRUNABLE_CLASSES[3].class_id,
];

fn prunable_class(class_id: &str) -> Option<&'static PrunableClass> {
    PRUNABLE_CLASSES.iter().find(|class| class.class_id == class_id)
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IntegrityClassReport {
    pub class_id: String,
    pub name: String,
    /// Rows for dangling-reference classes; unresolved models for ML; duplicate key groups for
    /// the two PK-less computed stores. The unit is named in `name`, never inferred by the UI.
    pub count: usize,
    pub prunable_count: usize,
    pub can_prune: bool,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IntegrityPruneOffer {
    pub offered: bool,
    pub prunable_findings: usize,
    pub class_ids: Vec<String>,
    pub recovery: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RecoverableIntegrityPrune {
    pub batch_id: String,
    pub created_at: String,
    pub class_ids: Vec<String>,
    pub pruned_findings: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IntegrityReport {
    pub classes: Vec<IntegrityClassReport>,
    pub checked_class_count: usize,
    pub finding_count: usize,
    pub summary: String,
    pub prune: IntegrityPruneOffer,
    /// ACTIVE quarantine batches survive an app restart and remain restorable from the checker.
    pub recoverable_prunes: Vec<RecoverableIntegrityPrune>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IntegrityPruneClassReceipt {
    pub class_id: String,
    pub pruned_findings: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IntegrityPruneReceipt {
    pub batch_id: String,
    pub pruned_findings: usize,
    pub classes: Vec<IntegrityPruneClassReceipt>,
}

fn integrity_count(conn: &Connection, sql: &str) -> Result<usize, String> {
    conn.query_row(sql, [], |row| row.get::<_, i64>(0))
        .map(|count| count.max(0) as usize)
        .map_err(|error| error.to_string())
}

/// Counts models whose stored `trained_on` JSON is absent in substance, malformed, or does not
/// resolve every recorded well name to exactly one current well. It deliberately does not infer a
/// model-to-well identity from sample data or another metadata field.
fn unresolved_ml_training_model_count(conn: &Connection) -> Result<usize, String> {
    let mut stmt = conn.prepare("SELECT trained_on FROM ml_models ORDER BY model_id").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0)).map_err(|e| e.to_string())?;
    // One grouped scan rather than a count per recorded name, which ran inside a loop inside a
    // loop. Counting per name rather than testing existence keeps the exactly-one rule intact: a
    // duplicated well name is still unresolved, which is what this function is here to catch.
    let mut names_stmt = conn
        .prepare("SELECT COALESCE(well_name, ''), count(*) FROM wells GROUP BY well_name")
        .map_err(|e| e.to_string())?;
    let well_name_counts: std::collections::HashMap<String, i64> = names_stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    let mut unresolved = 0usize;
    for row in rows {
        let encoded = row.map_err(|e| e.to_string())?;
        let Ok(names) = serde_json::from_str::<Vec<String>>(&encoded) else { unresolved += 1; continue };
        if names.is_empty() { unresolved += 1; continue; }
        let mut resolves = true;
        for name in names {
            if name.trim().is_empty() {
                resolves = false;
                break;
            }
            if well_name_counts.get(&name).copied().unwrap_or(0) != 1 { resolves = false; break; }
        }
        if !resolves { unresolved += 1; }
    }
    Ok(unresolved)
}

fn active_integrity_prunes(conn: &Connection) -> Result<Vec<RecoverableIntegrityPrune>, String> {
    let mut stmt = conn.prepare(
        "SELECT CAST(b.batch_id AS VARCHAR), CAST(b.created_at AS VARCHAR), b.classes,
                (SELECT count(*) FROM integrity_quarantine_computed q WHERE q.batch_id = b.batch_id) +
                (SELECT count(*) FROM integrity_quarantine_group_members q WHERE q.batch_id = b.batch_id) +
                (SELECT count(*) FROM integrity_quarantine_curve_samples q WHERE q.batch_id = b.batch_id)
         FROM integrity_prune_batches b WHERE b.state = 'ACTIVE'
         ORDER BY b.created_at DESC, b.batch_id"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        let classes: String = row.get(2)?;
        Ok(RecoverableIntegrityPrune {
            batch_id: row.get(0)?,
            created_at: row.get(1)?,
            class_ids: classes.split(',').filter(|item| !item.is_empty()).map(str::to_string).collect(),
            pruned_findings: row.get::<_, i64>(3)?.max(0) as usize,
        })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// Read-only, exhaustive SB-DBM-027 checker. Every class is emitted even at zero; the summary
/// always names how many classes were checked, so an empty finding set can never collapse to a
/// content-free "clean" badge.
pub fn check_referential_integrity(conn: &Connection) -> Result<IntegrityReport, String> {
    // AUDIT-2026-08-20 finding 52: the checker counts with the SAME orphan predicate the prune
    // selects on, so what a report OFFERS to quarantine and what the prune actually takes cannot
    // drift apart - which, on a primary-key-less table, is the difference between a clean redo
    // and a delete nothing recorded.
    let orphans = |class: &PrunableClass| -> Result<usize, String> {
        integrity_count(conn, &format!(
            "SELECT count(*) FROM {live} WHERE {orphan}",
            live = class.live,
            orphan = class.orphan.replace("{live}", class.live),
        ))
    };
    // The one count that is NOT the prune predicate, and deliberately so: a legacy NULL set_id is
    // reported because it is unresolvable, but it is not a broken reference and is never
    // quarantined - it stays labelled and visible. That is why this class has two counts.
    let current_missing = integrity_count(conn,
        "SELECT count(*) FROM computed_curves c LEFT JOIN log_sets l ON l.set_id = c.set_id WHERE l.set_id IS NULL")?;
    let current_prunable = orphans(&PRUNABLE_CLASSES[0])?;
    let archive_missing = orphans(&PRUNABLE_CLASSES[1])?;
    let group_missing = orphans(&PRUNABLE_CLASSES[2])?;
    let sample_missing = orphans(&PRUNABLE_CLASSES[3])?;
    let ml_unresolved = unresolved_ml_training_model_count(conn)?;
    let current_duplicates = integrity_count(conn,
        "SELECT count(*) FROM (SELECT well_id, curve_name, depth FROM computed_curves
         GROUP BY well_id, curve_name, depth HAVING count(*) > 1) duplicate_keys")?;
    // Versions legitimately repeat a tuple across set_id values, so archive uniqueness belongs
    // inside one declared set — the identity SB-DBM-026 actually writes.
    let archive_duplicates = integrity_count(conn,
        "SELECT count(*) FROM (SELECT set_id, well_id, curve_name, depth FROM computed_curves_archive
         GROUP BY set_id, well_id, curve_name, depth HAVING count(*) > 1) duplicate_keys")?;
    let classes = vec![
        IntegrityClassReport { class_id: INTEGRITY_CURRENT_LOG_SET_CLASS.into(), name: "Current computed rows without a resolvable log set (rows; includes legacy NULL set_id)".into(), count: current_missing, prunable_count: current_prunable, can_prune: true, action: "Quarantine broken non-NULL references; keep legacy NULL rows labelled and visible.".into() },
        IntegrityClassReport { class_id: INTEGRITY_ARCHIVE_LOG_SET_CLASS.into(), name: "Archived computed rows without a resolvable log set (rows)".into(), count: archive_missing, prunable_count: archive_missing, can_prune: true, action: "Quarantine the orphan archive rows; restore remains available by batch.".into() },
        IntegrityClassReport { class_id: INTEGRITY_WELL_GROUP_MEMBER_CLASS.into(), name: "Well-group memberships whose well is missing (rows)".into(), count: group_missing, prunable_count: group_missing, can_prune: true, action: "Quarantine the dangling membership rows; restore remains available by batch.".into() },
        IntegrityClassReport { class_id: INTEGRITY_CURVE_SAMPLE_CLASS.into(), name: "Curve samples whose curve metadata is missing (rows)".into(), count: sample_missing, prunable_count: sample_missing, can_prune: true, action: "Quarantine the orphan samples without serialising their numeric payload through IPC.".into() },
        IntegrityClassReport { class_id: INTEGRITY_ML_TRAINING_WELL_CLASS.into(), name: "ML models with unresolved trained-on well names (models)".into(), count: ml_unresolved, prunable_count: 0, can_prune: false, action: "Repair the stored training provenance or retire the model explicitly; never infer or auto-delete it.".into() },
        IntegrityClassReport { class_id: INTEGRITY_CURRENT_DUPLICATE_CLASS.into(), name: "Duplicate current computed curve-depth keys (duplicate tuples)".into(), count: current_duplicates, prunable_count: 0, can_prune: false, action: "Supply the declared SB-DBM-026 resolution; the checker never chooses a survivor.".into() },
        IntegrityClassReport { class_id: INTEGRITY_ARCHIVE_DUPLICATE_CLASS.into(), name: "Duplicate archived set/curve-depth keys (duplicate tuples)".into(), count: archive_duplicates, prunable_count: 0, can_prune: false, action: "Supply the declared SB-DBM-026 resolution; the checker never chooses a survivor.".into() },
    ];
    let checked_class_count = classes.len();
    let finding_count = classes.iter().map(|class| class.count).sum();
    let prunable_findings = classes.iter().map(|class| class.prunable_count).sum();
    Ok(IntegrityReport {
        classes, checked_class_count, finding_count,
        summary: format!("Checked {checked_class_count} integrity classes; {finding_count} findings."),
        prune: IntegrityPruneOffer {
            offered: true,
            prunable_findings,
            class_ids: PRUNABLE_INTEGRITY_CLASSES.iter().map(|class| (*class).to_string()).collect(),
            recovery: "Selected orphan rows move to typed project quarantine; Undo/Redo and post-restart restore keep the exact values.".into(),
        },
        recoverable_prunes: active_integrity_prunes(conn)?,
    })
}

/// Moves selected, explicitly prunable classes into typed quarantine in one transaction. No
/// frontend SQL, no numeric sample arrays over IPC, and no automatic ML/duplicate resolution.
pub fn prune_referential_integrity(conn: &Connection, class_ids: &[String]) -> Result<IntegrityPruneReceipt, String> {
    if class_ids.is_empty() { return Err("select at least one prunable integrity class".into()); }
    let mut selected = Vec::<&str>::new();
    for class_id in class_ids {
        if selected.contains(&class_id.as_str()) {
            return Err(format!("integrity class '{class_id}' was selected more than once"));
        }
        if !PRUNABLE_INTEGRITY_CLASSES.contains(&class_id.as_str()) {
            return Err(format!("integrity class '{class_id}' is report-only; resolving it requires an explicit identity or survivor decision"));
        }
        selected.push(class_id);
    }
    let batch_id = Uuid::new_v4().to_string();
    let classes_csv = selected.join(",");
    with_txn(conn, |conn| -> DbResult<IntegrityPruneReceipt> {
        conn.execute(
            "INSERT INTO integrity_prune_batches (batch_id, state, classes) VALUES (?1, 'ACTIVE', ?2)",
            params![batch_id, classes_csv],
        )?;
        let mut classes = Vec::new();
        for class_id in &selected {
            // Validated against PRUNABLE_INTEGRITY_CLASSES above, which IS this table's class ids.
            let class = prunable_class(class_id).ok_or_else(|| {
                DbError::Invalid(format!("integrity class '{class_id}' has no prune descriptor"))
            })?;
            let orphan = class.orphan.replace("{live}", class.live);
            let (tag_column, tag_value) = match class.source_tag {
                Some(tag) => (", source_table".to_string(), format!("?1, '{tag}', ")),
                None => (String::new(), "?1, ".to_string()),
            };
            let pruned = conn.execute(
                &format!(
                    "INSERT INTO {q} (batch_id{tag_column}, {cols}) \
                     SELECT {tag_value}{cols} FROM {live} WHERE {orphan}",
                    q = class.quarantine,
                    cols = class.columns,
                    live = class.live,
                ),
                params![batch_id],
            )?;
            // The SAME orphan predicate the quarantine INSERT selected on, so the delete can
            // never take a row the quarantine did not keep.
            conn.execute(&format!("DELETE FROM {} WHERE {orphan}", class.live), [])?;
            classes.push(IntegrityPruneClassReceipt { class_id: (*class_id).into(), pruned_findings: pruned });
        }
        let pruned_findings = classes.iter().map(|class| class.pruned_findings).sum();
        if pruned_findings == 0 {
            return Err(DbError::Invalid(
                "the selected integrity classes no longer contain quarantinable findings; run the checker again".into(),
            ));
        }
        Ok(IntegrityPruneReceipt { batch_id: batch_id.clone(), pruned_findings, classes })
    }).map_err(|error| error.to_string())
}

fn require_integrity_batch_state(conn: &Connection, batch_id: &str, expected: &str) -> DbResult<()> {
    let state = conn.query_row(
        "SELECT state FROM integrity_prune_batches WHERE batch_id = ?1",
        params![batch_id],
        |row| row.get::<_, String>(0),
    ).optional()?;
    match state {
        None => Err(DbError::Invalid(format!("integrity prune batch '{batch_id}' does not exist"))),
        Some(actual) if actual != expected => Err(DbError::Invalid(format!(
            "integrity prune batch '{batch_id}' is {actual}; expected {expected}"
        ))),
        Some(_) => Ok(()),
    }
}

fn count_bound(conn: &Connection, sql: &str, batch_id: &str) -> DbResult<usize> {
    conn.query_row(sql, params![batch_id], |row| row.get::<_, i64>(0))
        .map(|count| count.max(0) as usize)
        .map_err(DbError::from)
}

/// Restores one persisted quarantine batch exactly. Any identity collision refuses the whole
/// transaction rather than duplicating a PK-less curve or overwriting work created after prune.
pub fn restore_referential_integrity_prune(conn: &Connection, batch_id: &str) -> Result<usize, String> {
    with_txn(conn, |conn| -> DbResult<usize> {
        require_integrity_batch_state(conn, batch_id, "ACTIVE")?;
        for class in PRUNABLE_CLASSES.iter() {
            let query = format!(
                "SELECT count(*) FROM {live} WHERE {twin}",
                live = class.live,
                twin = class.quarantined_twin_exists(class.live),
            );
            if count_bound(conn, &query, batch_id)? > 0 {
                // Named, not numbered. This used to report "restore class {index + 1}" - a
                // position in a local array, which tells a reader nothing and silently
                // misattributes the moment anyone reorders it.
                return Err(DbError::Invalid(format!(
                    "integrity prune batch '{batch_id}' cannot be restored: identity collision in class '{}'",
                    class.class_id
                )));
            }
        }
        let mut restored = 0usize;
        for class in PRUNABLE_CLASSES.iter() {
            restored += conn.execute(
                &format!(
                    "INSERT INTO {live} ({cols}) SELECT {cols} FROM {q} WHERE {filter}",
                    live = class.live,
                    cols = class.columns,
                    q = class.quarantine,
                    filter = class.plain_batch_filter(),
                ),
                params![batch_id],
            )?;
        }
        conn.execute(
            "UPDATE integrity_prune_batches SET state = 'RESTORED', changed_at = now() WHERE batch_id = ?1",
            params![batch_id],
        )?;
        Ok(restored)
    }).map_err(|error| error.to_string())
}

fn ensure_reapply_rows(
    conn: &Connection,
    batch_id: &str,
    expected_sql: &str,
    live_sql: &str,
    missing_sql: &str,
    class_name: &str,
) -> DbResult<()> {
    // `class_name` is the class ID (AUDIT-2026-08-20 finding 52), so the refusal names the class
    // a caller can act on rather than a prose label only this function knew.
    let expected = count_bound(conn, expected_sql, batch_id)?;
    if expected == 0 { return Ok(()); }
    let live = count_bound(conn, live_sql, batch_id)?;
    let missing = count_bound(conn, missing_sql, batch_id)?;
    if live != expected || missing != 0 {
        return Err(DbError::Invalid(format!(
            "integrity prune batch '{batch_id}' cannot be redone: {class_name} changed after restore"
        )));
    }
    Ok(())
}

/// Reapplies the exact persisted batch for Ctrl+Y. It refuses if any restored row was edited,
/// removed, or collided since undo; redo must never broaden to newly discovered findings.
pub fn reapply_referential_integrity_prune(conn: &Connection, batch_id: &str) -> Result<usize, String> {
    with_txn(conn, |conn| -> DbResult<usize> {
        require_integrity_batch_state(conn, batch_id, "RESTORED")?;
        for class in PRUNABLE_CLASSES.iter() {
            let expected = format!(
                "SELECT count(*) FROM {q} WHERE {filter}",
                q = class.quarantine,
                filter = class.plain_batch_filter(),
            );
            let live = format!(
                "SELECT count(*) FROM {live} WHERE {twin}",
                live = class.live,
                twin = class.quarantined_twin_exists(class.live),
            );
            // EXCEPT ALL, so an EDITED value counts as missing even though its identity still
            // matches - redo must refuse a row somebody changed after the undo, not delete it.
            let missing = format!(
                "SELECT count(*) FROM (SELECT {cols} FROM {q} WHERE {filter} \
                 EXCEPT ALL SELECT {cols} FROM {live}) missing",
                cols = class.columns,
                q = class.quarantine,
                filter = class.plain_batch_filter(),
                live = class.live,
            );
            ensure_reapply_rows(conn, batch_id, &expected, &live, &missing, class.class_id)?;
        }
        let expected = count_bound(conn,
            "SELECT (SELECT count(*) FROM integrity_quarantine_computed WHERE batch_id = ?1) +
                    (SELECT count(*) FROM integrity_quarantine_group_members WHERE batch_id = ?1) +
                    (SELECT count(*) FROM integrity_quarantine_curve_samples WHERE batch_id = ?1)",
            batch_id,
        )?;
        for class in PRUNABLE_CLASSES.iter() {
            // The same identity the collision check and the liveness check used, so a redo can
            // only ever delete rows this batch actually quarantined.
            conn.execute(
                &format!(
                    "DELETE FROM {live} WHERE {twin}",
                    live = class.live,
                    twin = class.quarantined_twin_exists(class.live),
                ),
                params![batch_id],
            )?;
        }
        conn.execute(
            "UPDATE integrity_prune_batches SET state = 'ACTIVE', changed_at = now() WHERE batch_id = ?1",
            params![batch_id],
        )?;
        Ok(expected)
    }).map_err(|error| error.to_string())
}

/// Runs one read-only SELECT (a SQL console, full DuckDB SQL: joins,
/// window functions, aggregates). Anything that isn't a single SELECT/WITH statement
/// is rejected before execution.
pub fn run_readonly_query(conn: &Connection, sql: &str, limit: usize) -> Result<QueryPage, String> {
    let trimmed = sql.trim().trim_end_matches(';').trim();
    // The keyword test runs on the first REAL token, not the first byte: a query opening with
    // `--` comment lines or blanks is ordinary SQL, and refusing it as "not a SELECT" told users
    // their SELECT was not a SELECT (finding 23). Skipping the comments makes the guard
    // STRICTER, not looser — the token it inspects is the one DuckDB will execute.
    let first_code_line = trimmed
        .lines()
        .map(str::trim_start)
        .find(|line| !line.is_empty() && !line.starts_with("--"))
        .unwrap_or("");
    let lowered = first_code_line.to_lowercase();
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
    //
    // The wrapper's suffix sits on its OWN line: a query ending in a `--` comment would
    // otherwise swallow the closing paren and the LIMIT, and DuckDB would report a syntax
    // error against a query that is valid on its own (finding 23's second half).
    let wrapped = format!("SELECT * FROM ({trimmed}\n) __sandibumi_q LIMIT {}", limit + 1);
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
    let returned_rows = rows_out.len();
    Ok(QueryPage {
        columns,
        rows: rows_out,
        returned_rows,
        count_is_total: false,
        truncated,
    })
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

    /// An accepted calibration writes its whole coefficient set at a NAMED zone, and that must
    /// not disturb the whole-well scope — the two are different statements about the same well,
    /// and at run time the zone value wins inside the zone while `*` covers everything else.
    #[test]
    fn a_named_zone_batch_leaves_the_whole_well_scope_alone() {
        let mut conn = db();
        let w1 = well();
        set_well_param_overrides(&mut conn, &[(w1.clone(), "RSF".into(), Some(2.25))]).unwrap();
        set_zone_param_batch(
            &mut conn,
            "SAND_A",
            &[
                (w1.clone(), "A_CAP".into(), Some(0.4512)),
                (w1.clone(), "B_QV".into(), Some(0.005731)),
                (w1.clone(), "RSF".into(), Some(3.0)),
            ],
        )
        .unwrap();

        let rows = list_zone_params(&conn, &w1).unwrap();
        let at = |zone: &str, param: &str| {
            rows.iter()
                .find(|z| z.zone_name == zone && z.param_name == param)
                .and_then(|z| z.value_num)
        };
        assert!((at("SAND_A", "A_CAP").unwrap() - 0.4512).abs() < 1e-6);
        assert!((at("SAND_A", "RSF").unwrap() - 3.0).abs() < 1e-6);
        // The whole-well RSF is a separate row and keeps its own value.
        assert!((at("*", "RSF").unwrap() - 2.25).abs() < 1e-6);
        assert_eq!(list_well_param_overrides(&conn).unwrap().len(), 1);
    }

    /// Undo of an accepted calibration replays the PREVIOUS values through the same call, and a
    /// `None` there must clear the row rather than write a zero — a parameter silently pinned to
    /// zero is a wrong answer that keeps computing.
    #[test]
    fn a_none_in_a_zone_batch_clears_the_row_instead_of_writing_zero() {
        let mut conn = db();
        let w1 = well();
        set_zone_param_batch(&mut conn, "SAND_A", &[(w1.clone(), "A_CAP".into(), Some(0.45))]).unwrap();
        set_zone_param_batch(&mut conn, "SAND_A", &[(w1.clone(), "A_CAP".into(), None)]).unwrap();

        let rows = list_zone_params(&conn, &w1).unwrap();
        assert!(
            !rows.iter().any(|z| z.zone_name == "SAND_A" && z.param_name == "A_CAP"),
            "clearing must remove the row, not zero it: {rows:?}"
        );
    }
}

#[cfg(test)]
mod inspector_tests {
    use super::*;
    use sha2::{Digest, Sha256};

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

    fn file_sha256(path: &str) -> Vec<u8> {
        Sha256::digest(std::fs::read(path).unwrap()).to_vec()
    }

    /// CORRECTNESS — the three seeded counts and the explicit zero come from
    /// `22_database-model.md` SB-DBM-T26; the complete class inventory comes from
    /// SB-DBM-027. Dossier D-25 / T-DB-17 is the cited reporting-shape source.
    #[test]
    fn the_integrity_checker_names_every_class_including_zero_counts_offers_a_reversible_prune_and_never_says_clean_without_checking(
    ) {
        let conn = mem_db();
        let dangling_set = Uuid::new_v4().to_string();
        let referenced_well = Uuid::new_v4().to_string();
        let group_id = Uuid::new_v4().to_string();
        let missing_well = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO computed_curves_archive (set_id, well_id, depth, curve_name, value)
             VALUES (?1, ?2, 0.0, 'INTEGRITY_FIXTURE', NULL)",
            params![dangling_set, referenced_well],
        ).unwrap();
        conn.execute(
            "INSERT INTO well_groups (group_id, name) VALUES (?1, 'INTEGRITY_FIXTURE')",
            params![group_id],
        ).unwrap();
        conn.execute(
            "INSERT INTO well_group_members (group_id, well_id) VALUES (?1, ?2)",
            params![group_id, missing_well],
        ).unwrap();

        let rows_before: i64 = conn.query_row(
            "SELECT (SELECT count(*) FROM computed_curves_archive) +
                    (SELECT count(*) FROM well_group_members) +
                    (SELECT count(*) FROM curve_samples)",
            [], |row| row.get(0),
        ).unwrap();
        let report = check_referential_integrity(&conn).unwrap();
        let count = |class_id: &str| report.classes.iter()
            .find(|class| class.class_id == class_id)
            .unwrap_or_else(|| panic!("missing named integrity class {class_id}"))
            .count;
        assert_eq!(report.checked_class_count, 7, "every SB-DBM-027 class must be enumerated");
        assert_eq!(count(INTEGRITY_ARCHIVE_LOG_SET_CLASS), 1);
        assert_eq!(count(INTEGRITY_WELL_GROUP_MEMBER_CLASS), 1);
        assert_eq!(count(INTEGRITY_CURVE_SAMPLE_CLASS), 0, "T26 requires the empty class by name");
        assert_eq!(count(INTEGRITY_CURRENT_LOG_SET_CLASS), 0);
        assert_eq!(count(INTEGRITY_ML_TRAINING_WELL_CLASS), 0);
        assert_eq!(count(INTEGRITY_CURRENT_DUPLICATE_CLASS), 0);
        assert_eq!(count(INTEGRITY_ARCHIVE_DUPLICATE_CLASS), 0);
        assert_eq!(report.finding_count, 2);
        assert!(report.prune.offered, "the bounded quarantine prune must be offered");
        assert_eq!(report.prune.prunable_findings, 2);
        assert!(report.prune.class_ids.contains(&INTEGRITY_ARCHIVE_LOG_SET_CLASS.to_string()));
        assert!(report.prune.class_ids.contains(&INTEGRITY_WELL_GROUP_MEMBER_CLASS.to_string()));
        assert_ne!(report.summary.trim().to_ascii_lowercase(), "clean");
        assert!(report.summary.starts_with("Checked 7 integrity classes;"), "summary: {}", report.summary);
        let rows_after_check: i64 = conn.query_row(
            "SELECT (SELECT count(*) FROM computed_curves_archive) +
                    (SELECT count(*) FROM well_group_members) +
                    (SELECT count(*) FROM curve_samples)",
            [], |row| row.get(0),
        ).unwrap();
        assert_eq!(rows_after_check, rows_before, "the checker itself is read-only");

        let receipt = prune_referential_integrity(
            &conn,
            &[INTEGRITY_ARCHIVE_LOG_SET_CLASS.to_string(), INTEGRITY_WELL_GROUP_MEMBER_CLASS.to_string()],
        ).unwrap();
        assert_eq!(receipt.pruned_findings, 2);
        let after_prune = check_referential_integrity(&conn).unwrap();
        assert_eq!(after_prune.finding_count, 0);
        assert_eq!(after_prune.summary, "Checked 7 integrity classes; 0 findings.");
        assert!(after_prune.prune.offered, "the cleanup surface remains explicit even when nothing is eligible");

        restore_referential_integrity_prune(&conn, &receipt.batch_id).unwrap();
        let restored = check_referential_integrity(&conn).unwrap();
        assert_eq!(restored.classes.iter().find(|c| c.class_id == INTEGRITY_ARCHIVE_LOG_SET_CLASS).unwrap().count, 1);
        assert_eq!(restored.classes.iter().find(|c| c.class_id == INTEGRITY_WELL_GROUP_MEMBER_CLASS).unwrap().count, 1);
        reapply_referential_integrity_prune(&conn, &receipt.batch_id).unwrap();
        assert_eq!(check_referential_integrity(&conn).unwrap().finding_count, 0);
        restore_referential_integrity_prune(&conn, &receipt.batch_id).unwrap();
        assert_eq!(check_referential_integrity(&conn).unwrap().finding_count, 2);
    }

    /// AUDIT-2026-08-20 finding 52. Quarantine, restore and redo each hand-transcribed every
    /// class's identity key, four times over, on a deliberately primary-key-less table. The
    /// identity is stated ONCE now, so this pins the two things that statement has to get right.
    ///
    /// Pinned from BOTH sides, because one shared identity for both computed classes would pass
    /// the first half and fail the second.
    #[test]
    fn a_quarantined_rows_identity_is_its_own_class_and_a_collision_names_that_class() {
        let conn = mem_db();
        let group_id = Uuid::new_v4().to_string();
        let missing_well = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO well_groups (group_id, name) VALUES (?1, 'IDENTITY_FIXTURE')",
            params![group_id],
        ).unwrap();
        conn.execute(
            "INSERT INTO well_group_members (group_id, well_id) VALUES (?1, ?2)",
            params![group_id, missing_well],
        ).unwrap();

        // A - a REAL collision. The membership is quarantined, then somebody recreates it, so
        // restoring would duplicate a row on a table with no key to stop it. The refusal must
        // name the CLASS: it used to say "restore class 2", a position in a local array that
        // tells a reader nothing and misattributes the moment anyone reorders it.
        let receipt = prune_referential_integrity(
            &conn, &[INTEGRITY_WELL_GROUP_MEMBER_CLASS.to_string()],
        ).unwrap();
        conn.execute(
            "INSERT INTO well_group_members (group_id, well_id) VALUES (?1, ?2)",
            params![group_id, missing_well],
        ).unwrap();
        let refusal = restore_referential_integrity_prune(&conn, &receipt.batch_id).unwrap_err();
        assert!(
            refusal.contains(INTEGRITY_WELL_GROUP_MEMBER_CLASS),
            "a collision must name the class it happened in, got: {refusal}"
        );

        // B - and NOT a collision. The two computed classes have deliberately different
        // identities: a CURRENT row is named by well+curve+depth, an ARCHIVED row by those plus
        // its SET, because versions legitimately repeat a tuple across sets. Same well, same
        // curve, same depth, different set is a different archived row, and a restore that
        // refused here would strand a batch nobody could ever undo.
        let conn = mem_db();
        let well = Uuid::new_v4().to_string();
        let (set_a, set_b) = (Uuid::new_v4().to_string(), Uuid::new_v4().to_string());
        conn.execute(
            "INSERT INTO computed_curves_archive (set_id, well_id, depth, curve_name, value)
             VALUES (?1, ?2, 0.0, 'IDENTITY_FIXTURE', NULL)",
            params![set_a, well],
        ).unwrap();
        let receipt = prune_referential_integrity(
            &conn, &[INTEGRITY_ARCHIVE_LOG_SET_CLASS.to_string()],
        ).unwrap();
        assert_eq!(receipt.pruned_findings, 1);
        conn.execute(
            "INSERT INTO computed_curves_archive (set_id, well_id, depth, curve_name, value)
             VALUES (?1, ?2, 0.0, 'IDENTITY_FIXTURE', NULL)",
            params![set_b, well],
        ).unwrap();
        restore_referential_integrity_prune(&conn, &receipt.batch_id).unwrap_or_else(|error| {
            panic!("a different SET is a different archived row, not a collision: {error}")
        });
        let rows: i64 = conn
            .query_row("SELECT count(*) FROM computed_curves_archive", [], |row| row.get(0))
            .unwrap();
        assert_eq!(rows, 2, "both archived versions must survive the restore");
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

    /// Both sides of the compact note, so neither a meter that always nags nor one that
    /// never speaks would pass: it FIRES at exactly the quarter-dead threshold (Geolog's
    /// own WELL_FULL default, see COMPACT_NOTE_DEAD_FRACTION) naming Compact Project, and
    /// stays QUIET one byte under the fraction, and quiet again when the reclaimable
    /// amount is under the 64 MiB floor however dead the fraction looks.
    #[test]
    fn the_compact_note_fires_at_a_quarter_dead_and_stays_quiet_below_either_floor() {
        const MIB: u64 = 1024 * 1024;
        // Exactly 25% of 1 GiB dead: at the boundary, the note fires.
        let total = 1024 * MIB;
        let note = compact_suggestion(total, total / 4)
            .expect("a file exactly a quarter dead must earn the note");
        assert!(note.contains("Compact Project"), "the note must name the fix: {note}");
        assert!(note.contains("25%"), "the note states the measured fraction: {note}");
        // One step under the fraction: quiet.
        assert!(
            compact_suggestion(total, total / 4 - 1).is_none(),
            "just under a quarter dead must stay quiet"
        );
        // 90% dead but only 36 MiB reclaimable: under the byte floor, quiet.
        assert!(
            compact_suggestion(40 * MIB, 36 * MIB).is_none(),
            "a small file must stay quiet however dead its fraction"
        );
        // An empty accounting can never divide by zero into a note.
        assert!(compact_suggestion(0, 0).is_none());
    }

    /// Pins the pragma contract itself: `pragma_database_size()` on a real file-backed
    /// project still carries block_size / total_blocks / free_blocks under those names,
    /// and the derived byte counts are coherent (a file has size; free never exceeds
    /// total). If DuckDB renames or drops a column, this fails here rather than as a
    /// silently absent boot note.
    #[test]
    fn dead_space_reads_duckdbs_own_block_accounting_from_a_real_file() {
        let path = tmp_db("deadspace");
        let conn = init_db(&path).unwrap();
        // A brand-new schema still lives in the WAL; the accounting counts checkpointed
        // blocks only. A real open has already replayed its WAL by the time the meter runs.
        conn.execute_batch("CHECKPOINT").unwrap();
        let (total, free) = dead_space(&conn).expect("the pragma must answer on a file-backed db");
        assert!(total > 0, "a file-backed project occupies at least one block");
        assert!(free <= total, "free blocks are a subset of the file's blocks");
        drop(conn);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{path}.wal"));
    }

    /// A core delivery's name is a cross-reference: the plugs, the registration events and
    /// the same-named aux riders (extras) all correlate on it. Renaming moves ALL of them in
    /// one transaction — a name left behind in any table is a delivery silently split in two
    /// — and is audited under mode RENAME. The other side: a taken target name refuses BY
    /// NAME and moves NOTHING, so a lazier rename that updates only the registry, or one
    /// that merges into an existing delivery, would fail one half or the other.
    #[test]
    fn renaming_a_core_delivery_moves_plugs_riders_and_registrations_or_nothing() {
        let conn = mem_db();
        let wid_uuid = Uuid::new_v4();
        insert_well(&conn, wid_uuid, "SANDI-REN", None, None, None).unwrap();
        let wid = wid_uuid.to_string();
        let d = [2000.0f32, 2001.0, 2002.0];
        let nan = [f32::NAN; 3];
        insert_core_data(&conn, &wid, "RAW", None, &d, &[0.2, 0.21, 0.19], &nan, &nan, &nan).unwrap();
        conn.execute(
            "INSERT INTO core_registrations (well_id, set_name, seq, kind, delta) VALUES (?1, 'RAW', 0, 'manual', 1.5)",
            params![wid],
        )
        .unwrap();
        insert_aux_data(
            &conn,
            &wid,
            "CORE",
            "RAW",
            None,
            &[AuxRow {
                dataset: "CORE".into(),
                depth_top: 2000.0,
                depth_base: None,
                item: "LITH".into(),
                value_num: None,
                value_text: Some("ss".into()),
            }],
        )
        .unwrap();

        let receipt =
            rename_delivery_set(&conn, "core", &wid, None, "RAW", "CORE2024", "QC Lead", "HUMAN", "Data Sets")
                .unwrap();
        // 1 registry row + 3 plugs + 1 registration; riders = 1 aux_sets + 1 aux_data.
        assert_eq!(receipt.rows_moved, 5, "registry, plugs and registrations all move");
        assert_eq!(receipt.rider_rows_moved, 2, "the extras ride the core's name");
        let count = |sql: &str, name: &str| -> i64 {
            conn.query_row(sql, params![wid, name], |r| r.get(0)).unwrap()
        };
        for table in ["core_sets", "core_data", "core_registrations", "aux_sets", "aux_data"] {
            let sql = format!("SELECT COUNT(*) FROM {table} WHERE well_id = ?1 AND set_name = ?2");
            assert_eq!(count(&sql, "RAW"), 0, "{table} must hold nothing under the old name");
            assert!(count(&sql, "CORE2024") > 0, "{table} must hold the delivery under the new name");
        }
        // The delivery is still the ACTIVE one under its new name, so every core reader
        // (which follows the active set) sees the same plugs it saw before the rename.
        let active: String = conn
            .query_row(
                "SELECT set_name FROM core_sets WHERE well_id = ?1 AND active = 1",
                params![wid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(active, "CORE2024");
        let audited: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM audit_detail WHERE mode = 'RENAME' AND name = 'RAW'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(audited, 1, "the rename is on the audit trail");

        // The other side: a taken name refuses and NOTHING moves.
        insert_core_data(&conn, &wid, "OTHER", None, &d[..1], &[0.2], &nan[..1], &nan[..1], &nan[..1])
            .unwrap();
        let err = rename_delivery_set(&conn, "core", &wid, None, "CORE2024", "OTHER", "QC Lead", "HUMAN", "Data Sets")
            .unwrap_err()
            .to_string();
        assert!(err.contains("never merges"), "the collision refusal names the rule: {err}");
        assert_eq!(
            count("SELECT COUNT(*) FROM core_data WHERE well_id = ?1 AND set_name = ?2", "CORE2024"),
            3,
            "a refused rename changes nothing"
        );
        let err = rename_delivery_set(&conn, "core", &wid, None, "GONE", "X", "QC Lead", "HUMAN", "Data Sets")
            .unwrap_err()
            .to_string();
        assert!(err.contains("no core delivery named"), "a missing source refuses by name: {err}");
        let err = rename_delivery_set(&conn, "core", &wid, None, "CORE2024", "Y", "", "HUMAN", "Data Sets")
            .unwrap_err()
            .to_string();
        assert!(err.contains("operator"), "a rename without an operator is refused before anything moves: {err}");
    }

    /// The two refusals that carry the resolution contracts, pinned by their naming phrases,
    /// and the curve-set success path beside them: curve set RAW is never renamed in either
    /// direction (RAW's absolute priority in curve resolution would silently re-decide which
    /// delivery answers every mnemonic), an aux delivery sharing a core set's name is that
    /// core's rider and never travels alone — and a non-RAW curve set rename moves
    /// curve_meta, its frame declaration in import_sets and its array curves together.
    #[test]
    fn curve_raw_is_never_renamed_and_an_aux_rider_names_its_core() {
        let conn = mem_db();
        let wid_uuid = Uuid::new_v4();
        insert_well(&conn, wid_uuid, "SANDI-REN2", None, None, None).unwrap();
        let wid = wid_uuid.to_string();
        let nan = [f32::NAN; 1];
        insert_core_data(&conn, &wid, "FPROOH", None, &[2000.0], &[0.2], &nan, &nan, &nan).unwrap();

        let err = rename_delivery_set(&conn, "curve", &wid, None, "RAW", "EDIT1", "QC", "HUMAN", "Data Sets")
            .unwrap_err()
            .to_string();
        assert!(err.contains("absolute priority"), "renaming RAW away is refused: {err}");
        let err = rename_delivery_set(&conn, "curve", &wid, None, "EDIT1", "RAW", "QC", "HUMAN", "Data Sets")
            .unwrap_err()
            .to_string();
        assert!(err.contains("absolute priority"), "renaming onto RAW is refused: {err}");

        insert_aux_data(
            &conn,
            &wid,
            "CORE",
            "FPROOH",
            None,
            &[AuxRow {
                dataset: "CORE".into(),
                depth_top: 2000.0,
                depth_base: None,
                item: "SO".into(),
                value_num: Some(1.0),
                value_text: None,
            }],
        )
        .unwrap();
        let err = rename_delivery_set(
            &conn, "aux", &wid, Some("CORE"), "FPROOH", "SOLO", "QC", "HUMAN", "Data Sets",
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("rides the core delivery"), "a rider never travels alone: {err}");

        // Success path for a curve set: meta, frame declaration and array curves move together.
        let cid = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO curve_meta (curve_id, well_id, set_name, mnemonic) VALUES (?1, ?2, 'NMR22', 'GR')",
            params![cid, wid],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO import_sets (well_id, set_name, declared_sampling_style, effective_sampling_style, sampling_verified)
             VALUES (?1, 'NMR22', 'CONTINUOUS_REGULAR', 'CONTINUOUS_REGULAR', TRUE)",
            params![wid],
        )
        .unwrap();
        write_array_log(&conn, &wid, "NMR22", "T2DIST", &[2000.0], &[vec![0.5, 0.4]], None).unwrap();
        let receipt =
            rename_delivery_set(&conn, "curve", &wid, None, "NMR22", "NMR22_QC", "QC", "HUMAN", "Data Sets")
                .unwrap();
        assert_eq!(receipt.rows_moved, 3, "curve_meta + import_sets + array_logs all move");
        for table in ["curve_meta", "import_sets", "array_logs"] {
            let n: i64 = conn
                .query_row(
                    &format!("SELECT COUNT(*) FROM {table} WHERE well_id = ?1 AND set_name = 'NMR22'"),
                    params![wid],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(n, 0, "{table} must not keep the old name");
        }
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
        assert_eq!(
            read_meta(&conn, "format_version").as_deref(),
            Some(FORMAT_VERSION.to_string().as_str())
        );
        drop(conn);
        let _ = std::fs::remove_file(&path);
    }

    /// CHARACTERIZATION — `22_database-model.md` SB-DBM-T01, sourced to the shipped
    /// format gate at `db.rs::check_and_stamp_format`, requires both version identities,
    /// the writer identity and exact byte preservation on refusal.
    #[test]
    fn a_newer_format_is_refused_with_both_versions_and_its_writer_while_the_project_bytes_remain_identical() {
        let path = tmp_db("future");
        {
            // A file from a hypothetical future format: it carries a stamp but NOT the
            // current schema (a future format may have renamed any table) — so if
            // create_schema ran despite the refusal, `wells` would appear.
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch("CREATE TABLE project_meta (key VARCHAR PRIMARY KEY, value VARCHAR);")
                .unwrap();
            conn.execute(
                "INSERT INTO project_meta VALUES ('format_version', ?1), ('written_by', 'SandiBumi future-writer')",
                params![(FORMAT_VERSION + 1).to_string()],
            )
            .unwrap();
        }
        let before = file_sha256(&path);
        let err = init_db(&path).err().expect("a newer file must be refused");
        let msg = err.to_string();
        assert!(
            msg.contains(&format!("format {}", FORMAT_VERSION + 1)),
            "must name the file's format: {msg}"
        );
        assert!(
            msg.contains(&format!("format {FORMAT_VERSION}")),
            "must name the running build's format: {msg}"
        );
        assert!(msg.contains("SandiBumi future-writer"), "must name the writer: {msg}");
        assert!(msg.contains("upgrade SandiBumi"), "must say what to do: {msg}");
        assert_eq!(file_sha256(&path), before, "a refused open must leave every project byte unchanged");
        // The refusal must have mutated nothing: no schema, stamp intact.
        let conn = Connection::open(&path).unwrap();
        let wells: i64 = conn
            .query_row("SELECT count(*) FROM duckdb_tables() WHERE table_name = 'wells'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(wells, 0, "create_schema must not have run on a refused file");
        assert_eq!(
            read_meta(&conn, "format_version").as_deref(),
            Some((FORMAT_VERSION + 1).to_string().as_str()),
            "stamp must be untouched"
        );
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

    /// SB-DBM-017 / DEC-025: the neutron matrix basis is DECLARED curve metadata. Absence
    /// stays absent - never inferred from the unit, the family, the contractor, the tool or a
    /// matrix default - a declaration is scoped to the one curve it names, it carries its
    /// source, and an empty declaration is refused rather than stored blank.
    #[test]
    fn the_neutron_matrix_basis_is_declared_never_inferred_and_absence_stays_absent() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        // Idempotent additive schema: a legacy project converges on the same shape.
        create_schema(&conn).unwrap();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "SANDI-NB", None, None, None).unwrap();
        let well = id.to_string();
        let nphi = upsert_curve_meta(
            &conn, &well, "RAW", "NPHI", Some("v/v"), Some("NPHI"), Some("LAS import"), None,
        )
        .unwrap();
        let gr = upsert_curve_meta(
            &conn, &well, "RAW", "GR", Some("gapi"), Some("GR"), Some("LAS import"), None,
        )
        .unwrap();
        let basis_of = |curve: &str| -> Option<String> {
            list_generic_curve_catalog(&conn, &well)
                .unwrap()
                .into_iter()
                .find(|c| c.curve_id == curve)
                .unwrap()
                .neutron_basis
        };
        // A. An imported neutron curve carries NO basis until somebody declares one - the
        //    unit and the family are not a declaration.
        assert_eq!(basis_of(&nphi), None, "absence stays absent; nothing is inferred");
        // B. An empty declaration is refused, not stored blank - and a missing source too:
        //    a declaration without an authority is a guess wearing a declaration's clothes.
        assert!(set_curve_neutron_basis(&conn, &nphi, "", "user").is_err());
        assert!(set_curve_neutron_basis(&conn, &nphi, "LIMESTONE", "  ").is_err());
        assert_eq!(basis_of(&nphi), None, "a refused declaration must write nothing");
        // C. A real declaration lands on exactly the curve it names, with its source.
        set_curve_neutron_basis(
            &conn, &nphi, "LIMESTONE", "declared at import by the user (DEC-025)",
        )
        .unwrap();
        assert_eq!(basis_of(&nphi).as_deref(), Some("LIMESTONE"));
        assert_eq!(basis_of(&gr), None, "the declaration is scoped to the curve it names");
        let source: String = conn
            .query_row(
                "SELECT neutron_basis_source FROM curve_meta WHERE curve_id = ?1",
                params![nphi],
                |r| r.get(0),
            )
            .unwrap();
        assert!(source.contains("DEC-025"), "the declaration records its authority: {source}");
        // D. An unknown curve is refused by name - a WELL-FORMED id that matches no row, so
        //    the refusal comes from the zero-row guard, not from a cast error upstream of it.
        let absent = Uuid::new_v4().to_string();
        assert!(set_curve_neutron_basis(&conn, &absent, "SANDSTONE", "user").is_err());
    }

    /// The basis declaration is a CLOSED vocabulary, stored canonically. A typo accepted
    /// here becomes permanent metadata no module check can ever satisfy (the workflow
    /// boundary compares the stored string against tokens like "LIMESTONE"), so an unknown
    /// token is refused naming the vocabulary, and a recognized short form lands as the
    /// canonical spelling every consumer pins. Both sides: the synonym stores, the typo
    /// writes nothing.
    #[test]
    fn a_basis_declaration_is_a_closed_vocabulary_stored_canonically() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "SANDI-NBV", None, None, None).unwrap();
        let well = id.to_string();
        let nphi = upsert_curve_meta(
            &conn, &well, "RAW", "NPHI", Some("v/v"), Some("NPHI"), Some("LAS import"), None,
        )
        .unwrap();
        let stored = |conn: &Connection| -> Option<String> {
            conn.query_row(
                "SELECT neutron_basis FROM curve_meta WHERE curve_id = ?1",
                params![nphi],
                |r| r.get(0),
            )
            .unwrap()
        };
        // A short-form synonym is accepted and stored as the canonical full token.
        set_curve_neutron_basis(&conn, &nphi, "ls", "user").unwrap();
        assert_eq!(stored(&conn).as_deref(), Some("LIMESTONE"));
        // An unknown token is refused by name, names the vocabulary, and writes nothing.
        let err = set_curve_neutron_basis(&conn, &nphi, "CHALK", "user")
            .unwrap_err()
            .to_string();
        assert!(err.contains("CHALK"), "the refused token is named: {err}");
        assert!(err.contains("SANDSTONE/SS"), "the vocabulary is named: {err}");
        assert_eq!(stored(&conn).as_deref(), Some("LIMESTONE"), "a refusal writes nothing");
    }

    /// SB-DBM-011 / exact SB-DBM-T11 (DEC-020, DEC-022, DEC-023): the audit is STRUCTURED
    /// rows with the controlled vocabulary, uninterrupted repeats of the same type collapse
    /// into ONE entry (an interruption breaks the chain - both sides pinned), the timestamp
    /// column's own default is UTC, the operator is explicit and refused when absent, and a
    /// zone-scoped entry names the zone-set identity, whose version moves when a zone moves.
    #[test]
    fn the_audit_is_structured_collapses_uninterrupted_repeats_and_carries_operator_utc_and_zone_set(
    ) {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "SANDI-AUD", None, None, None).unwrap();
        let well = id.to_string();
        upsert_md_zone(&conn, &well, "MIOCENE_A", 1000.0, 1100.0).unwrap();
        upsert_md_zone(&conn, &well, "MIOCENE_B", 1100.0, 1200.0).unwrap();
        let entries = |conn: &Connection| -> i64 {
            conn.query_row("SELECT count(*) FROM audit_entry", [], |r| r.get(0)).unwrap()
        };

        // (i) three zone-parameter changes: one entry each, PARAMETER/INPUT rows with unit,
        // name and value, the zone riding as an INTERVAL row, the zone set at version 1.
        for (param, value) in [("GR_MA", 20.0_f32), ("GR_SH", 120.0), ("RHO_MA", 2.65)] {
            set_zone_param_audited(
                &conn, &well, "MIOCENE_A", param, Some(value), None, "Jauhar-QC", "HUMAN",
                "Zones",
            )
            .unwrap();
        }
        assert_eq!(entries(&conn), 3, "three distinct changes are three entries");
        let listed = list_audit_entries(&conn, 10).unwrap();
        let newest = &listed[0];
        assert_eq!(newest.operator, "Jauhar-QC");
        assert_eq!(newest.operator_kind, "HUMAN");
        assert_eq!(newest.zone_set_version, Some(1), "zone-scoped entries carry the zone set");
        assert!(newest.zone_set_digest.is_some());
        assert_eq!(newest.details.len(), 2);
        assert_eq!(
            (newest.details[0].location.as_str(), newest.details[0].mode.as_str()),
            ("INTERVAL", "INPUT")
        );
        assert_eq!(newest.details[0].name, "MIOCENE_A");
        let parameter = &newest.details[1];
        assert_eq!((parameter.location.as_str(), parameter.mode.as_str()), ("PARAMETER", "INPUT"));
        assert_eq!(parameter.name, "RHO_MA");
        assert_eq!(parameter.unit.as_deref(), Some("g/cc"), "the manifest unit rides the row");
        assert_eq!(parameter.value.as_deref(), Some("2.65"));
        let gr_ma_entry = &listed[2];
        assert_eq!(gr_ma_entry.details[1].unit.as_deref(), Some("gAPI"));

        // (iii) forty uninterrupted drags of the same handle: ONE entry, not forty - with
        // the count honest and the value the LAST gesture's.
        for step in 0..40 {
            set_zone_param_audited(
                &conn, &well, "MIOCENE_A", "GR_MA", Some(20.0 + step as f32), None,
                "Jauhar-QC", "HUMAN", "Crossplot",
            )
            .unwrap();
        }
        assert_eq!(entries(&conn), 4, "forty uninterrupted repeats collapse to ONE entry");
        let collapsed = &list_audit_entries(&conn, 1).unwrap()[0];
        assert_eq!(collapsed.repeat_count, 40);
        assert_eq!(collapsed.details[1].value.as_deref(), Some("59"));

        // The other side: an INTERRUPTION breaks the chain, so the same action again is a
        // NEW entry rather than a late collapse into the old one.
        set_zone_param_audited(
            &conn, &well, "MIOCENE_B", "GR_MA", Some(30.0), None, "Jauhar-QC", "HUMAN",
            "Crossplot",
        )
        .unwrap();
        set_zone_param_audited(
            &conn, &well, "MIOCENE_A", "GR_MA", Some(25.0), None, "Jauhar-QC", "HUMAN",
            "Crossplot",
        )
        .unwrap();
        assert_eq!(entries(&conn), 6, "an interrupted repeat is a new entry, never a merge");

        // (ii) a curve rename is mode RENAME on the LOG, and a unit change is the
        // dotted-name ATTRIBUTE case.
        let curve_id =
            upsert_curve_meta(&conn, &well, "RAW", "GRX", Some("gAPI"), Some("GR"), None, None)
                .unwrap();
        update_curve_meta_audited(
            &conn, &curve_id, "GRY", Some("api"), Some("GR"), "Jauhar-QC", "HUMAN",
            "Curve Catalog",
        )
        .unwrap();
        let renamed = &list_audit_entries(&conn, 1).unwrap()[0];
        let rename_row = renamed
            .details
            .iter()
            .find(|detail| detail.mode == "RENAME")
            .expect("a mnemonic change audits as RENAME");
        assert_eq!(rename_row.location, "LOG");
        assert_eq!(rename_row.name, "GRX");
        assert_eq!(rename_row.value.as_deref(), Some("GRY"));
        let attribute_row = renamed
            .details
            .iter()
            .find(|detail| detail.location == "ATTRIBUTE")
            .expect("a unit change audits as the dotted-name ATTRIBUTE case");
        assert!(attribute_row.name.contains('.'), "{}", attribute_row.name);

        // (iv) UTC by the column's own default - structural, not a wall-clock race.
        let default_expr: String = conn
            .query_row(
                "SELECT column_default FROM duckdb_columns()
                 WHERE table_name = 'audit_entry' AND column_name = 'ts_utc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(default_expr.to_uppercase().contains("UTC"), "{default_expr}");

        // DEC-023: moving a zone changes the zone-set identity, and the next zone-scoped
        // entry says so with a bumped version and a different digest.
        let digest_before = collapsed.zone_set_digest.clone().unwrap();
        upsert_md_zone(&conn, &well, "MIOCENE_B", 1105.0, 1200.0).unwrap();
        set_zone_param_audited(
            &conn, &well, "MIOCENE_A", "GR_SH", Some(110.0), None, "Jauhar-QC", "HUMAN",
            "Zones",
        )
        .unwrap();
        let moved = &list_audit_entries(&conn, 1).unwrap()[0];
        assert_eq!(moved.zone_set_version, Some(2), "a moved top is a NEW zone-set version");
        assert_ne!(moved.zone_set_digest.as_deref(), Some(digest_before.as_str()));

        // DEC-020: the operator is explicit or the audit refuses - never inferred - and the
        // controlled vocabularies refuse by name.
        let refusal = record_audit_entry(
            &conn, Some(&well), "  ", "HUMAN", "Zones", "test", None, None,
            &[AuditDetail {
                location: "PARAMETER".into(),
                mode: "INPUT".into(),
                unit: None,
                name: "GR_MA".into(),
                value: None,
            }],
        )
        .unwrap_err();
        assert!(refusal.to_string().contains("DEC-020"), "{refusal}");
        let refusal = record_audit_entry(
            &conn, Some(&well), "Jauhar-QC", "HUMAN", "Zones", "test", None, None,
            &[AuditDetail {
                location: "GESTURE".into(),
                mode: "INPUT".into(),
                unit: None,
                name: "GR_MA".into(),
                value: None,
            }],
        )
        .unwrap_err();
        assert!(
            refusal.to_string().contains("GESTURE")
                && refusal.to_string().contains("PARAMETER"),
            "the refusal names the offending value and the permitted set: {refusal}"
        );
        let refusal = record_audit_entry(
            &conn, Some(&well), "Jauhar-QC", "HUMAN", "Zones", "test", None, None,
            &[AuditDetail {
                location: "ATTRIBUTE".into(),
                mode: "INPUT".into(),
                unit: None,
                name: "GRX".into(),
                value: None,
            }],
        )
        .unwrap_err();
        assert!(
            refusal.to_string().contains("dotted"),
            "ATTRIBUTE without a dotted name breaks the chapter's rule: {refusal}"
        );
    }

    /// SB-CLY-001 (DEC-036): a pre-existing project's four-member kind CHECK is rebuilt to
    /// accept ENDPOINT_INVALID with every stored row copied verbatim - and the migration is
    /// idempotent, so a project already carrying the member is left alone.
    #[test]
    fn an_old_degradation_table_accepts_the_endpoint_invalid_kind_after_migration_and_keeps_its_rows(
    ) {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        // Recreate the PRE-migration shape: the four-member CHECK.
        conn.execute_batch(
            "DROP TABLE run_degradations;
             CREATE TABLE run_degradations (
                set_id      UUID NOT NULL,
                position    INTEGER NOT NULL,
                module      VARCHAR NOT NULL,
                kind        VARCHAR NOT NULL CHECK (
                    kind IN ('CLAMPED', 'DEFAULTED', 'TRUNCATED', 'SUBSTITUTED_INPUT')
                ),
                detail      VARCHAR NOT NULL,
                occurrences BIGINT NOT NULL CHECK (occurrences > 0),
                PRIMARY KEY (set_id, position)
             );",
        )
        .unwrap();
        let set_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO run_degradations VALUES (?1, 0, 'phi_den', 'CLAMPED', 'kept', 3)",
            params![set_id],
        )
        .unwrap();
        // The old CHECK refuses the fifth kind - the exact failure the migration removes.
        assert!(conn
            .execute(
                "INSERT INTO run_degradations VALUES (?1, 1, 'vsh_gr', 'ENDPOINT_INVALID', 'x', 1)",
                params![set_id],
            )
            .is_err());
        migrate_run_degradations_endpoint_invalid(&conn).unwrap();
        migrate_run_degradations_endpoint_invalid(&conn).unwrap(); // idempotent
        conn.execute(
            "INSERT INTO run_degradations VALUES (?1, 1, 'vsh_gr', 'ENDPOINT_INVALID', 'x', 1)",
            params![set_id],
        )
        .expect("the rebuilt CHECK accepts the documented fifth member");
        let kept: (String, i64) = conn
            .query_row(
                "SELECT kind, occurrences FROM run_degradations WHERE position = 0",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(kept, ("CLAMPED".to_string(), 3), "existing rows are copied verbatim");
    }

    /// SB-DBM-009 / DEC-022: legacy `log_sets.created_at` values are WIB (UTC+7) wall time
    /// and are converted to UTC instants EXACTLY ONCE, with the declared zone and the ruling
    /// recorded as the converted values' SOURCE - so a later reader sees the offset was
    /// declared by the product owner, never measured from the data - and new rows default to
    /// UTC so the local meaning cannot creep back in.
    #[test]
    fn legacy_wib_timestamps_convert_to_utc_exactly_once_and_the_declared_zone_is_the_recorded_source(
    ) {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "SANDI-TS", None, None, None).unwrap();
        // A legacy row: written at 12:00 WIB wall time on the pre-migration schema.
        conn.execute(
            "INSERT INTO log_sets (set_id, well_id, set_name, version, module, created_at)
             VALUES (gen_random_uuid(), ?1, 'RAW', 1, 'legacy', TIMESTAMP '2026-08-01 12:00:00')",
            params![id.to_string()],
        )
        .unwrap();
        migrate_log_set_timestamps_to_utc(&conn).unwrap();
        let stored = || -> String {
            conn.query_row(
                "SELECT strftime(created_at, '%Y-%m-%d %H:%M:%S') FROM log_sets WHERE module = 'legacy'",
                [],
                |r| r.get(0),
            )
            .unwrap()
        };
        // A. 12:00 WIB is 05:00 UTC - seven hours, the declared offset.
        assert_eq!(stored(), "2026-08-01 05:00:00");
        // B. Idempotent: running the migration again must NOT move history by another seven
        //    hours - the marker document gates the subtraction.
        migrate_log_set_timestamps_to_utc(&conn).unwrap();
        assert_eq!(stored(), "2026-08-01 05:00:00", "a second run must not subtract again");
        // C. The source is RECORDED: the marker names the declared zone and DEC-022, so the
        //    conversion is traceable to the owner's declaration rather than to a guess.
        let record: String = conn
            .query_row(
                "SELECT json FROM documents WHERE doc_type = 'migration' AND name = 'DEC-022-created-at-utc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(record.contains("WIB (UTC+7)"), "the declared zone is recorded: {record}");
        assert!(record.contains("DEC-022"), "the ruling is the recorded source: {record}");
        assert!(record.contains("declared by the product owner"), "{record}");
        // D. A row written AFTER the migration defaults to a UTC instant, not local wall
        //    time - proven by pinning the SESSION zone to Jakarta first, so a default that
        //    silently reverted to now() would land seven hours off.
        conn.execute_batch("SET TimeZone = 'Asia/Jakarta'").unwrap();
        conn.execute(
            "INSERT INTO log_sets (set_id, well_id, set_name, version, module)
             VALUES (gen_random_uuid(), ?1, 'RAW', 2, 'fresh')",
            params![id.to_string()],
        )
        .unwrap();
        let drift_seconds: f64 = conn
            .query_row(
                "SELECT abs(epoch(created_at) - epoch(now() AT TIME ZONE 'UTC'))
                 FROM log_sets WHERE module = 'fresh'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            drift_seconds < 60.0,
            "a fresh row must be a UTC instant; it is {drift_seconds}s from UTC now - a local default on a UTC+7 machine would read ~25200s"
        );
        // E. The DEFAULT itself declares UTC. This bundled build's now() happens to sit on
        //    UTC whatever the session zone is set to, so arm D alone cannot catch a default
        //    quietly reverted to bare now() - but a build with ICU zone support would then
        //    write local wall time again. The declaration is pinned structurally.
        let default_expr: String = conn
            .query_row(
                "SELECT column_default FROM duckdb_columns()
                 WHERE table_name = 'log_sets' AND column_name = 'created_at'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            default_expr.contains("UTC"),
            "created_at's default must declare UTC, got: {default_expr}"
        );
    }

    /// SB-DBM-030. Source: Geolog `cgg.h` `MISS_FLOAT = -1.0e30` (and the manuals' `-1.0D38`),
    /// DEC-027/DEC-061/DEC-062. The store's null discipline: an undeclared large-negative
    /// sentinel is screened to SQL NULL by a strict inequality against a bound COMPUTED one
    /// decade inside the cited constant, the screen is COUNTED (the flag channel - never
    /// silent), NaN binds SQL NULL so absence is not representable as a number at the store,
    /// and a value exactly ON the bound is DATA that survives bit for bit.
    #[test]
    fn an_undeclared_large_negative_null_is_screened_to_sql_null_and_counted_and_a_value_on_the_bound_stays_data(
    ) {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "SANDI-NULL", None, None, None).unwrap();
        let well = id.to_string();
        let curve =
            upsert_curve_meta(&conn, &well, "RAW", "GR", Some("gapi"), Some("GR"), None, None)
                .unwrap();
        let bound = GEOLOG_MISS_FLOAT / 10.0;
        let depths = [1000.0, 1001.0, 1002.0, 1003.0, 1004.0, 1005.0];
        let values = [-999.25f32, GEOLOG_MISS_FLOAT, -1.0e38, bound, 3.5, f32::NAN];
        let screened = insert_curve_samples(&conn, &curve, &depths, &values).unwrap();
        let samples = get_curve_samples(&conn, &curve).unwrap();
        // A. A value exactly ON the computed bound is DATA, bit for bit - a `<=` or a
        //    hand-typed decimal would have coerced it. Asserted FIRST so a bound slip fires
        //    here, distinctly, before the count can.
        assert_eq!(
            samples[3].value.to_bits(),
            bound.to_bits(),
            "a value exactly on the bound is DATA"
        );
        // B. cgg.h's MISS_FLOAT and the manual's -1.0D38 are BOTH caught - an equality against
        //    either sentinel would miss the other - and the count is the flag channel.
        assert_eq!(screened, 2);
        // C. At the store, absence is SQL NULL: the two screened sentinels and the NaN all
        //    bind NULL, and nothing else does - "no value" is never a number.
        let nulls: i64 = conn
            .query_row(
                "SELECT count(*) FROM curve_samples WHERE curve_id = ?1 AND value IS NULL",
                params![curve],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(nulls, 3);
        // D. Reload: screened samples read back as the NaN missing convention, and the rest
        //    of the DATA survives bit for bit - the declared-null lookalike -999.25 (declared
        //    nulls are resolved at parse, never re-guessed here) and an ordinary reading.
        assert!(samples[1].value.is_nan() && samples[2].value.is_nan());
        assert!(samples[5].value.is_nan());
        assert_eq!(samples[0].value.to_bits(), (-999.25f32).to_bits());
        assert_eq!(samples[4].value.to_bits(), 3.5f32.to_bits());
    }

    /// AUDIT-2026-08-20 finding 65. `migrate_drop_computed_curves_pk` is the shipped example of
    /// a destructive migration, and its doc block — including the R-B guarantee that a failed
    /// backup ABORTS — had drifted onto `migrate_run_degradations_endpoint_invalid`, a function
    /// that takes no path and copies nothing. Two of the three destructive migrations were then
    /// bare, and the one guarantee a reader most needs to find sat on the wrong function.
    ///
    /// A doc comment is not compiled, so nothing caught it. This is what catches it: a migration
    /// that copies the project before rewriting it must SAY SO on itself — which policy this is
    /// (RELEASE §3.2), and that a failed copy aborts instead of proceeding. Both halves matter
    /// and neither implies the other: "we take a backup" without "a failed backup aborts" is the
    /// reading under which rewriting a field-scale project after the copy failed looks allowed.
    #[test]
    fn every_migration_that_copies_the_project_first_documents_that_a_failed_copy_aborts() {
        let source = include_str!("db.rs");
        let lines: Vec<&str> = source.split('\n').collect();
        let mut checked: Vec<&str> = Vec::new();
        let mut silent: Vec<(&str, &str)> = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            let Some(rest) = line.strip_prefix("pub fn ").or_else(|| line.strip_prefix("fn ")) else {
                continue;
            };
            let Some(name) = rest.split(['<', '(']).next() else { continue };
            let end = lines[index + 1..]
                .iter()
                .position(|l| l.starts_with('}'))
                .map(|offset| index + 1 + offset)
                .unwrap_or(lines.len() - 1);
            let body = lines[index + 1..=end].join("\n");
            // The helper itself, and the `_with_backup` seam that takes the copier as a
            // parameter, are the mechanism rather than a migration that uses it.
            if name == "backup_before_destructive_migration" || name.ends_with("_with_backup") {
                continue;
            }
            if !body.contains("backup_before_destructive_migration") {
                continue;
            }
            checked.push(name);
            let mut doc_start = index;
            while doc_start > 0 && lines[doc_start - 1].starts_with("///") {
                doc_start -= 1;
            }
            let doc = lines[doc_start..index].join("\n");
            for token in ["RELEASE", "ABORTS"] {
                if !doc.contains(token) {
                    silent.push((name, token));
                }
            }
        }
        assert_eq!(
            checked,
            [
                "migrate_tvdss_positive_down",
                "migrate_drop_computed_curves_pk",
                "migrate_point_data_sets",
            ],
            "these three migrations copy the project before rewriting it; one that stops going \
             through the shared copier, or a new one that starts, must be seen here knowingly"
        );
        assert!(
            silent.is_empty(),
            "a migration that copies the project first must document it on ITSELF: {silent:?}"
        );
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
        assert_eq!(
            gr.n_valid, 2,
            "finite samples are distinct from stored rows"
        );
        assert_eq!(gr.n_missing, 1, "the stored NaN is reported as missing");
        assert_eq!(gr.min, Some(55.0));
        assert_eq!(gr.max, Some(60.0));
        assert_eq!(gr.mean, Some(57.5));
        let inventory = list_generic_curve_inventory(&conn, &ids).unwrap();
        let gr_inventory = inventory
            .iter()
            .find(|c| c.mnemonic == "GR")
            .expect("GR inventoried");
        assert_eq!(gr_inventory.set_name, "RAW");
        assert_eq!(gr_inventory.unit.as_deref(), Some("gAPI"));
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

    /// A delivery is one write unit. Validation failure in any curve must leave every
    /// existing sibling untouched; a valid batch replaces all siblings together.
    #[test]
    fn generic_curve_sample_batch_is_atomic_across_curves() {
        let conn = mem_db();
        let well = Uuid::new_v4();
        insert_well(&conn, well, "SANDI-BATCH", None, None, None).unwrap();
        let well = well.to_string();
        let gr = upsert_curve_meta(
            &conn,
            &well,
            "WIRE",
            "GR",
            Some("GAPI"),
            Some("GR"),
            None,
            None,
        )
        .unwrap();
        let pef = upsert_curve_meta(
            &conn,
            &well,
            "WIRE",
            "PEF",
            Some("B/E"),
            Some("PEF"),
            None,
            None,
        )
        .unwrap();
        insert_curve_samples(&conn, &gr, &[1.0, 2.0], &[10.0, 20.0]).unwrap();
        insert_curve_samples(&conn, &pef, &[1.0, 2.0], &[5.0, 6.0]).unwrap();

        let bad = insert_curve_samples_batch(
            &conn,
            &[1.0, 2.0, 3.0],
            &[(&gr, &[100.0, 200.0, 300.0][..]), (&pef, &[50.0, 60.0][..])],
        );
        assert!(bad.is_err());
        assert_eq!(
            get_curve_samples(&conn, &gr)
                .unwrap()
                .iter()
                .map(|p| p.value)
                .collect::<Vec<_>>(),
            vec![10.0, 20.0]
        );
        assert_eq!(
            get_curve_samples(&conn, &pef)
                .unwrap()
                .iter()
                .map(|p| p.value)
                .collect::<Vec<_>>(),
            vec![5.0, 6.0]
        );

        insert_curve_samples_batch(
            &conn,
            &[1.0, 2.0, 3.0],
            &[
                (&gr, &[100.0, 200.0, 300.0][..]),
                (&pef, &[50.0, 60.0, 70.0][..]),
            ],
        )
        .unwrap();
        assert_eq!(get_curve_samples(&conn, &gr).unwrap().len(), 3);
        assert_eq!(get_curve_samples(&conn, &pef).unwrap()[2].value, 70.0);

        // Reach a failure only after DELETE and Arrow staging: the duplicate curve id creates
        // duplicate (curve_id, depth) keys in the final INSERT. Rollback must restore the
        // complete previously committed delivery.
        let staged_failure = insert_curve_samples_batch(
            &conn,
            &[1.0, 2.0, 3.0],
            &[
                (&gr, &[900.0, 901.0, 902.0][..]),
                (&gr, &[800.0, 801.0, 802.0][..]),
            ],
        );
        assert!(staged_failure.is_err(), "duplicate staged primary keys must fail");
        assert_eq!(
            get_curve_samples(&conn, &gr)
                .unwrap()
                .iter()
                .map(|point| point.value)
                .collect::<Vec<_>>(),
            vec![100.0, 200.0, 300.0],
            "post-staging failure rolls the prior curve back exactly"
        );
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
        insert_well(&conn, Uuid::new_v4(), "SANDI-1", Some("Sandi"), None, None).unwrap();
        let page = run_readonly_query(&conn, "SELECT well_name, field_name FROM wells", 100).unwrap();
        assert_eq!(page.columns, vec!["well_name", "field_name"]);
        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.rows[0][0].as_deref(), Some("SANDI-1"));

        assert!(run_readonly_query(&conn, "DELETE FROM wells", 100).is_err());
        assert!(run_readonly_query(&conn, "SELECT 1; DROP TABLE wells", 100).is_err());
    }

    /// `zones_from_tops` had no test at all, and it is what turns a tops delivery into the zone
    /// intervals every module's per-zone parameters resolve against. Four claims.
    ///
    /// **Zones are contiguous top-down**: each zone's base is the NEXT top's depth. A gap would
    /// leave rock belonging to no zone, so a cutoff or an override would silently skip it.
    ///
    /// **The last zone runs to TD**, taken from the deepest logged sample — otherwise the
    /// deepest interval, often the reservoir, would have no bottom at all.
    ///
    /// **A well with no tops yields NO zones** — not one phantom zone spanning the well, and not
    /// a panic. A phantom would quietly become "the whole well is one zone" in every summary.
    ///
    /// **It REBUILDS rather than appends.** Re-running after a tops edit must not leave the old
    /// intervals behind beside the new ones; two overlapping zones would make "which zone is this
    /// sample in?" unanswerable.
    #[test]
    fn zones_from_tops_are_contiguous_and_absent_tops_make_no_zones() {
        let conn = mem_db();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "SANDI-Z", None, None, None).unwrap();
        let well = id.to_string();

        // No tops yet — and this must not invent one.
        assert!(
            zones_from_tops(&conn, &well).unwrap().is_empty(),
            "a well with no tops must produce no zones at all"
        );

        // Logged interval 2000..2050, so TD for the last zone is 2050.
        let n = 101usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let nan = vec![f32::NAN; n];
        insert_standard_curves(
            &conn,
            id,
            depth,
            vec![50.0; n],
            nan.clone(),
            nan.clone(),
            nan.clone(),
            nan.clone(),
            nan,
        )
        .unwrap();

        upsert_top(&conn, &well, "A", 2005.0, None).unwrap();
        upsert_top(&conn, &well, "B", 2020.0, None).unwrap();
        upsert_top(&conn, &well, "C", 2035.0, None).unwrap();

        let zones = zones_from_tops(&conn, &well).unwrap();
        assert_eq!(zones.len(), 3);
        assert_eq!(zones[0].zone_name, "A");
        assert_eq!(zones[0].top_depth, 2005.0);
        assert_eq!(zones[0].bottom_depth, 2020.0, "a zone's base is the next top");
        assert_eq!(zones[1].bottom_depth, 2035.0);
        assert_eq!(zones[2].bottom_depth, 2050.0, "the last zone runs to TD");

        // Contiguous: no rock between two zones belongs to neither.
        for pair in zones.windows(2) {
            assert_eq!(
                pair[0].bottom_depth, pair[1].top_depth,
                "zones must meet exactly, leaving no unassigned interval"
            );
        }

        // Rebuild, don't append: move a top and drop one, then re-run.
        upsert_top(&conn, &well, "B", 2025.0, None).unwrap();
        conn.execute("DELETE FROM tops WHERE well_id = ?1 AND top_name = 'C'", params![&well])
            .unwrap();
        let rebuilt = zones_from_tops(&conn, &well).unwrap();
        assert_eq!(rebuilt.len(), 2, "the removed top must not leave its zone behind");
        assert_eq!(rebuilt[0].bottom_depth, 2025.0, "the moved top must move its neighbour's base");
        assert_eq!(rebuilt[1].bottom_depth, 2050.0);

        let stored = list_zones(&conn, &well).unwrap();
        assert_eq!(stored.len(), 2, "the zones TABLE must be rebuilt, not accumulated");
    }

    /// A top deeper than anything logged must not produce an INVERTED zone. `max_depth.max(top)`
    /// is what prevents it, and a base above its own top would make every thickness negative —
    /// net pay would come out negative and sum against the other zones.
    #[test]
    fn a_top_below_the_logged_interval_never_makes_an_inverted_zone() {
        let conn = mem_db();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "SANDI-ZD", None, None, None).unwrap();
        let well = id.to_string();

        let n = 11usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        insert_standard_curves(
            &conn,
            id,
            depth,
            vec![50.0; n],
            nan.clone(),
            nan.clone(),
            nan.clone(),
            nan.clone(),
            nan,
        )
        .unwrap();

        // Logged to 2010; this top is below every sample.
        upsert_top(&conn, &well, "DEEP", 2099.0, None).unwrap();
        let zones = zones_from_tops(&conn, &well).unwrap();
        assert_eq!(zones.len(), 1);
        assert!(
            zones[0].bottom_depth >= zones[0].top_depth,
            "a zone's base must never sit above its own top (got {} above {})",
            zones[0].bottom_depth,
            zones[0].top_depth,
        );
    }

    /// The SQL panel is the one place a user types raw SQL, and rule 6 says it is read-only.
    /// `readonly_query_selects_and_rejects` proves a bare `DELETE` and a `;`-smuggled `DROP`
    /// are refused; it does NOT cover the other write verbs, nor the shape that actually gets
    /// past the prefix check.
    ///
    /// That shape is a CTE prefix. The guard admits anything starting `with`, so
    /// `WITH x AS (SELECT 1) DELETE FROM wells` clears both checks — no leading write verb, no
    /// semicolon. What stops it is the SUBQUERY WRAP: the statement is executed as
    /// `SELECT * FROM (<user sql>) __sandibumi_q`, and a DELETE is not a valid subquery, so it
    /// dies in the parser. **The wrap is a security boundary, not just a LIMIT mechanism** —
    /// anyone tempted to run the user's SQL directly and apply the cap another way must know
    /// that it is the only thing refusing this statement.
    ///
    /// A legitimate CTE must still work, or the guard would have been "fixed" by banning `with`.
    #[test]
    fn readonly_query_refuses_every_write_shape_including_a_cte_prefix() {
        let conn = mem_db();
        insert_well(&conn, Uuid::new_v4(), "SANDI-1", Some("Sandi"), None, None).unwrap();
        let count = |c: &Connection| -> i64 {
            c.query_row("SELECT COUNT(*) FROM wells", [], |r| r.get(0)).unwrap()
        };
        assert_eq!(count(&conn), 1);

        // Every write verb on its own, plus mixed case — the check lowercases, so a shouty
        // DROP must fare no better than a quiet one.
        for sql in [
            "UPDATE wells SET well_name = 'X'",
            "INSERT INTO wells (well_id, well_name) VALUES ('x', 'X')",
            "DROP TABLE wells",
            "ALTER TABLE wells ADD COLUMN zzz INTEGER",
            "CREATE TABLE zzz (a INTEGER)",
            "TRUNCATE wells",
            "DeLeTe FROM wells",
            // A leading comment must not disguise a write as a query.
            "/* select */ DELETE FROM wells",
            "-- select\nDELETE FROM wells",
        ] {
            assert!(
                run_readonly_query(&conn, sql, 100).is_err(),
                "must be refused: {sql}"
            );
        }

        // The CTE-prefixed write: passes the prefix check and carries no semicolon.
        for sql in [
            "WITH x AS (SELECT 1) DELETE FROM wells",
            "WITH x AS (SELECT 1) UPDATE wells SET well_name = 'X'",
            "WITH x AS (SELECT 1) INSERT INTO wells (well_id, well_name) VALUES ('x', 'X')",
        ] {
            assert!(
                run_readonly_query(&conn, sql, 100).is_err(),
                "CTE-prefixed write must be refused: {sql}"
            );
        }

        // Nothing above may have touched the data, whatever route it was refused by.
        assert_eq!(count(&conn), 1, "no refused statement may modify the project");

        // A real CTE is the point of allowing `with` at all and must still run.
        let page = run_readonly_query(
            &conn,
            "WITH w AS (SELECT well_name FROM wells) SELECT well_name FROM w",
            100,
        )
        .expect("a legitimate CTE query must still be allowed");
        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.rows[0][0].as_deref(), Some("SANDI-1"));
    }

    /// Finding 23 (closed 2026-08-20): an ordinary SQL comment must not break the read-only
    /// console. A leading `--` line used to hide the keyword, so a valid SELECT was refused
    /// with a message saying it was not a SELECT — the panel's own starter opened that way.
    /// And the LIMIT wrapper was single-line, so a TRAILING `--` swallowed the closing paren
    /// and DuckDB reported a syntax error against a query that is valid on its own. The refusal
    /// side of the guard is pinned next door (`readonly_query_refuses_every_write_shape…`):
    /// `-- select` above a DELETE still refuses, because the token inspected is the real one.
    #[test]
    fn readonly_query_reads_through_leading_and_trailing_comments() {
        let conn = mem_db();
        insert_well(&conn, Uuid::new_v4(), "SANDI-1", Some("Sandi"), None, None).unwrap();

        // Leading comment and blank lines above a real SELECT — the starter's original shape.
        let page = run_readonly_query(
            &conn,
            "-- wells in this project\n\n  -- (edit freely)\nSELECT well_name FROM wells",
            100,
        )
        .expect("a SELECT under leading comment lines is a SELECT");
        assert_eq!(page.rows[0][0].as_deref(), Some("SANDI-1"));

        // Trailing comment on the last line — must not swallow the wrapper's paren and LIMIT.
        let page = run_readonly_query(
            &conn,
            "SELECT well_name FROM wells -- just the names",
            100,
        )
        .expect("a trailing comment must not break the wrapped query");
        assert_eq!(page.rows[0][0].as_deref(), Some("SANDI-1"));
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
        assert_eq!(capped.returned_rows, 3);
        assert!(!capped.count_is_total);
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

    /// CORRECTNESS - SB-DBM-039 / SB-DBM-T41. The exact 10,000-row input, 100-row
    /// console cap and expected count meanings come from `docs/PRD_v2/22_database-model.md`
    /// section 6, SB-DBM-T41, sourced there to SB-CORE-002. No expected value is copied
    /// from the implementation: the fixture creates exactly the two cited cardinalities.
    #[test]
    fn the_inspector_reports_the_true_ten_thousand_row_total_while_the_hundred_row_console_page_names_its_count_as_returned_not_total(
    ) {
        let conn = mem_db();
        conn.execute(
            "INSERT INTO wells (well_id, well_name)
             SELECT uuid(), 'ROW-' || CAST(i AS VARCHAR) FROM range(10000) AS fixture(i)",
            [],
        )
        .unwrap();

        let inspector = get_table_page(&conn, "wells", None, 0, 100).unwrap();
        let console = run_readonly_query(&conn, "SELECT well_name FROM wells ORDER BY well_name", 100)
            .unwrap();

        assert_eq!(inspector.rows.len(), 100, "the inspector returns one requested page");
        assert_eq!(inspector.total_rows, 10_000, "the inspector's total is the true COUNT(*)");
        assert!(!inspector.truncated, "the inspector's page count and true total are distinct fields");

        assert_eq!(console.rows.len(), 100, "the SQL console returns only its requested page");
        assert_eq!(console.returned_rows, 100, "the console names this value as rows returned");
        assert!(!console.count_is_total, "the console explicitly says its page count is not a total");
        assert!(console.truncated, "the LIMIT+1 probe proves more than 100 rows exist");

        let wire = serde_json::to_value(&console).unwrap();
        assert_eq!(wire.get("returned_rows"), Some(&serde_json::json!(100)));
        assert_eq!(wire.get("count_is_total"), Some(&serde_json::json!(false)));
        assert!(
            wire.get("total_rows").is_none(),
            "one field name must never carry the inspector's true-total meaning and the console's page-count meaning"
        );
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

    /// T-REP-14. `table_page_reads_and_cell_updates` above browses ONE table. The inspector's
    /// dropdown offers every entry in `TABLE_SPECS`, and a table whose query is malformed does
    /// not degrade — it throws, and the grid shows an error where the user expected their data.
    ///
    /// So every spec is exercised: each returns exactly the columns it declares, in order, and
    /// the well-scoped ones refuse rather than silently returning the whole project. That last
    /// point is the one worth a test: a well-scoped read that quietly dropped its filter would
    /// show one well's grid full of another well's samples, which looks like data, not an error.
    #[test]
    fn every_inspector_table_returns_the_columns_it_declares() {
        let conn = mem_db();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "SANDI-INSP", Some("Sandi Field"), Some(2000.0), Some(25.0)).unwrap();
        let w = id.to_string();

        for spec in table_specs() {
            let scope = if spec.well_scoped { Some(w.as_str()) } else { None };
            let page = get_table_page(&conn, spec.table, scope, 0, 200)
                .unwrap_or_else(|e| panic!("table '{}' failed to browse: {e}", spec.table));
            assert_eq!(
                page.columns,
                spec.columns.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
                "table '{}' returned the wrong column set",
                spec.table
            );
            assert!(!page.truncated, "the paginated path always knows its true count");

            if spec.well_scoped {
                assert!(
                    get_table_page(&conn, spec.table, None, 0, 200).is_err(),
                    "well-scoped table '{}' must refuse rather than return the whole project",
                    spec.table
                );
            }
        }
    }

    /// The Inspector applies a cell edit by re-stating the rest of the row: a zone edit re-declares
    /// the row's own depth datum, a top edit re-sends its colour. Those values are read back out of
    /// the row ON SCREEN, so a column the page does not carry is a value the edit cannot supply -
    /// and `depth_datum` shipped on a zone while the page was still the older three-column one, so
    /// every zone edit was refused with "zone X needs a declared depth datum", naming the one thing
    /// the user had no way to give it.
    ///
    /// `every_inspector_table_returns_the_columns_it_declares` above cannot catch that. It asks
    /// each spec for the columns it declares itself, so it agrees with whatever the spec says and
    /// passed throughout. The contract that actually broke couples the two sides, and this pins it
    /// for every editable table rather than only for zones - the same omission in `tops` or
    /// `core_data` fails the same silent way, refusing an edit for a column nobody can see.
    #[test]
    fn an_inspector_edit_reads_its_row_back_from_a_page_that_carries_the_column() {
        let frontend = include_str!("../../src/ui/dbInspectorPanel.ts");
        let specs = table_specs();

        // `commitEdit`'s switch: one arm per table, each re-reading its row through `cell("name")`.
        let switch_start = frontend
            .find("switch (def.key)")
            .expect("dbInspectorPanel.ts must switch on the table key to apply an edit");
        let body = &frontend[switch_start..];

        let mut reads = 0usize;
        for spec in &specs {
            let needle = format!("case \"{}\":", spec.table);
            let arm_start = match body.find(&needle) {
                Some(index) => index + needle.len(),
                None => continue,
            };
            let rest = &body[arm_start..];
            // The arm runs to the next arm, or to the switch's own default.
            let arm_end = rest
                .find("case \"")
                .into_iter()
                .chain(rest.find("default:"))
                .min()
                .unwrap_or(rest.len());

            for (index, matched) in rest[..arm_end].match_indices("cell(\"") {
                let after = &rest[index + matched.len()..arm_end];
                let name = &after[..after.find('"').expect("a cell() read names its column")];
                assert!(
                    spec.columns.contains(&name),
                    "the inspector reads '{}' back out of the '{}' row it is editing, but the page \
                     does not carry that column - every edit of that table is refused for a value \
                     the user cannot see or supply",
                    name,
                    spec.table
                );
                reads += 1;
            }
        }

        // The other side: a scan that matched nothing would satisfy the assertion above perfectly.
        assert!(
            reads >= 6,
            "only {reads} row reads were found across the edit arms - the switch shape changed and \
             this test is no longer looking at the edit path"
        );
    }



    /// SB-DBM-031 residue (DEC-073 item 5, RULED 2026-08-18: source-declared rows migrate,
    /// unknown legacy meaning is preserved as unknown, cross-datum comparison is refused).
    /// The T31 core - typed zone/contact custody, positive-down TVDSS, framed comparison -
    /// is pinned by its own test; what closes here is the delivery-set half: a delivery
    /// declares the datum its depths are quoted in ONCE, on its SET row (the per-row
    /// alternative would break the positional-Appender contracts); an unknown token
    /// refuses naming the vocabulary; a legacy set stays NULL - the preserved unknown,
    /// never inferred to MD - and behaves exactly as before; and every depth-pairing
    /// reader of the four stores refuses a KNOWN non-MD delivery NAMING both datums,
    /// because an MD log depth against a TVDSS plug depth is F-17's category error that
    /// silently produces a number.
    #[test]
    fn a_delivery_declares_its_datum_once_and_a_known_non_md_set_refuses_log_depth_pairing_naming_both(
    ) {
        let conn = mem_db();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "SANDI-DATUM", None, None, None).unwrap();
        let w = id.to_string();

        // A - the import boundary declares: a core delivery lands with its datum on the
        // set row, and the pairing reader works.
        insert_core_data(&conn, &w, "CORE", None, &[1000.0, 1001.0], &[0.18, 0.19], &[10.0, 11.0], &[2.65, 2.65], &[0.3, 0.3]).unwrap();
        declare_set_datum(&conn, "core_sets", &w, None, "CORE", "MD").unwrap();
        let stored: Option<String> = conn
            .query_row(
                "SELECT depth_datum FROM core_sets WHERE well_id = ?1 AND set_name = 'CORE'",
                params![w],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored.as_deref(), Some("MD"), "the declaration lands on the SET row");
        assert!(!get_core_point_series(&conn, &w).unwrap().is_empty(), "MD pairs freely");

        // B - an unknown token refuses NAMING the vocabulary, and declares nothing.
        let bad = declare_set_datum(&conn, "core_sets", &w, None, "CORE", "DRILLER")
            .expect_err("an unknown token must refuse");
        assert!(
            bad.to_string().contains("MD | TVD | TVDSS | TVDKB | TWT | OWT | CDEPTH"),
            "{bad}"
        );
        // ... and so does a declaration on a set that does not exist.
        let missing = declare_set_datum(&conn, "core_sets", &w, None, "NO_SUCH", "MD")
            .expect_err("a missing set cannot be declared");
        assert!(missing.to_string().contains("NO_SUCH"), "{missing}");

        // C - lowercase parses to the same token: the vocabulary is canonical, not
        // case-sensitive, so a guard comparing raw strings cannot half-work.
        declare_set_datum(&conn, "core_sets", &w, None, "CORE", "md").unwrap();
        assert!(!get_core_point_series(&conn, &w).unwrap().is_empty(), "md == MD");

        // D - a KNOWN non-MD delivery refuses log-depth pairing NAMING BOTH datums, on
        // every guarded store.
        declare_set_datum(&conn, "core_sets", &w, None, "CORE", "TVDSS").unwrap();
        let refused = get_core_point_series(&conn, &w).expect_err("TVDSS plugs cannot pair with MD logs");
        let text = refused.to_string();
        assert!(
            text.contains("TVDSS") && text.contains("MD") && text.contains("CORE"),
            "both datums and the delivery are named: {text}"
        );
        insert_scal_pc(&conn, &w, "SCAL", None, &[ScalPcRow {
            sample_no: Some(1), depth: Some(1000.0), perm: 10.0, poro: 0.2, pc: 5.0, sw: 0.6,
            system: None, ift: None,
        }]).unwrap();
        declare_set_datum(&conn, "scal_sets", &w, None, "SCAL", "TVDSS").unwrap();
        assert!(get_scal_pc(&conn, &w).expect_err("scal too").to_string().contains("TVDSS"));
        insert_aux_data(&conn, &w, "XRD", "RAW", None, &[AuxRow {
            dataset: "XRD".into(), depth_top: 1000.0, depth_base: None, item: "ILLITE".into(),
            value_num: Some(12.0), value_text: None,
        }]).unwrap();
        declare_set_datum(&conn, "aux_sets", &w, Some("XRD"), "RAW", "TVD").unwrap();
        let aux_refused = list_aux_data(&conn, &w, Some("XRD")).expect_err("aux too").to_string();
        assert!(aux_refused.contains("TVD") && aux_refused.contains("XRD/RAW"), "{aux_refused}");
        conn.execute(
            "INSERT INTO image_sets (well_id, dataset, set_name, active, depth_datum) VALUES (?1, 'CORE PHOTO', 'RAW', 1, 'TWT')",
            params![w],
        )
        .unwrap();
        assert!(list_well_images(&conn, &w, None).expect_err("images too").to_string().contains("TWT"));

        // E - back on MD, everything pairs again: the guard is about a WRONG datum, not
        // about having declared one.
        declare_set_datum(&conn, "core_sets", &w, None, "CORE", "MD").unwrap();
        assert!(!get_core_point_series(&conn, &w).unwrap().is_empty());

        // F - the preserved unknown: a legacy delivery with no declaration (NULL) behaves
        // exactly as it always did, on a fresh well - unknown is unknown, not wrong.
        let legacy = Uuid::new_v4();
        insert_well(&conn, legacy, "SANDI-LEGACY", None, None, None).unwrap();
        let lw = legacy.to_string();
        insert_core_data(&conn, &lw, "CORE", None, &[1000.0], &[0.2], &[10.0], &[2.65], &[0.4]).unwrap();
        let datum: Option<String> = conn
            .query_row(
                "SELECT depth_datum FROM core_sets WHERE well_id = ?1 AND set_name = 'CORE'",
                params![lw],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(datum, None, "nothing backfills MD onto an undeclared delivery");
        assert!(!get_core_point_series(&conn, &lw).unwrap().is_empty(), "and it still pairs");
    }

    /// Codex whole-repository review, P1: the guard above says it is shared by EVERY depth-pairing
    /// reader, and three core readers never called it.
    ///
    /// `get_core_point_series` and `get_scal_pc` were guarded and are pinned by the test above;
    /// `get_core_plugs` (HFU/FZI clustering, the facies core-permeability tie), `get_core_por_gd`
    /// (SandiMin's φ and ρg calibration) and `equations::fetch_core_series` (the plotted log and
    /// crossplot overlay) were not. Each pairs a plug depth against the MD log frame — nearest
    /// sample, or the track a plug is drawn in — so a delivery declared TVD or TVDSS put the plug
    /// beside the wrong rock and every downstream number stayed finite and plausible.
    ///
    /// Pinned from both sides. A refusal alone would also be produced by a reader that had simply
    /// stopped returning core, so MD must still pair before and after, and the refusal must NAME
    /// both datums and the delivery — the guard is about a WRONG datum, never about having
    /// declared one.
    #[test]
    fn every_core_reader_refuses_a_cross_datum_delivery_and_not_merely_the_two_that_already_did() {
        let conn = mem_db();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "SANDI-DATUM-2", None, None, None).unwrap();
        let w = id.to_string();
        insert_core_data(
            &conn, &w, "CORE", None,
            &[1000.0, 1001.0], &[0.18, 0.19], &[10.0, 11.0], &[2.65, 2.65], &[0.3, 0.3],
        )
        .unwrap();

        // The three readers the guard was missing from, each reached the way its caller reaches it.
        let plugs = |c: &Connection| get_core_plugs(c, &w);
        let por_gd = |c: &Connection| get_core_por_gd(c, &w);
        let series = |c: &Connection| crate::equations::fetch_core_series(c, &w);

        // Declared MD: all three pair, as they always have.
        declare_set_datum(&conn, "core_sets", &w, None, "CORE", "MD").unwrap();
        assert_eq!(plugs(&conn).unwrap().len(), 2, "MD plugs pair with an MD log frame");
        assert_eq!(por_gd(&conn).unwrap().len(), 2, "and so do the SandiMin calibration plugs");
        assert!(
            series(&conn).unwrap().iter().any(|s| s.point_count > 0),
            "and the overlay has something to draw"
        );

        // Declared TVDSS: all three refuse, NAMING both datums and the delivery.
        declare_set_datum(&conn, "core_sets", &w, None, "CORE", "TVDSS").unwrap();
        for (what, text) in [
            ("get_core_plugs", plugs(&conn).expect_err("HFU/facies must refuse").to_string()),
            ("get_core_por_gd", por_gd(&conn).expect_err("SandiMin must refuse").to_string()),
            ("fetch_core_series", series(&conn).expect_err("the overlay must refuse").to_string()),
        ] {
            assert!(
                text.contains("TVDSS") && text.contains("MD") && text.contains("CORE"),
                "{what} must name both datums and the delivery, got: {text}"
            );
        }

        // Back on MD everything pairs again — the refusal is about the datum, not about the
        // reader having been taught to fail.
        declare_set_datum(&conn, "core_sets", &w, None, "CORE", "MD").unwrap();
        assert_eq!(plugs(&conn).unwrap().len(), 2);
        assert_eq!(por_gd(&conn).unwrap().len(), 2);
        assert!(series(&conn).unwrap().iter().any(|s| s.point_count > 0));

        // And the preserved unknown still passes: a legacy delivery that declared nothing is
        // unknown, never inferred to be wrong.
        let legacy = Uuid::new_v4();
        insert_well(&conn, legacy, "SANDI-DATUM-2-LEGACY", None, None, None).unwrap();
        let lw = legacy.to_string();
        insert_core_data(&conn, &lw, "CORE", None, &[1000.0], &[0.2], &[10.0], &[2.65], &[0.4])
            .unwrap();
        assert_eq!(get_core_plugs(&conn, &lw).unwrap().len(), 1, "undeclared still pairs");
        assert_eq!(get_core_por_gd(&conn, &lw).unwrap().len(), 1);
        assert!(crate::equations::fetch_core_series(&conn, &lw).unwrap().iter().any(|s| s.point_count > 0));
    }

    /// SB-DBM-041 exact T42 (`22_database-model.md` section 6), unblocked by SB-DBM-011's
    /// audit tables (DEC-020/022/023, landed 2026-08-18): the inspector exposes the
    /// COMPLETE provenance and audit registry - log sets, structured audit, zone-set
    /// versions, run parameters and degradations, the curve archive, the curve catalog and
    /// the model registry - and none of it is editable. The audit rows come through the
    /// REAL writer (`record_audit_entry`); `ml_models` is pinned to omit the joblib blob,
    /// and the frontend catalog is pinned read-only with matching well scoping, so the
    /// grid cannot grow an edit affordance the backend never offered. T41's true-total
    /// contract is untouched - these pages report `total_rows` through the same path.
    #[test]
    fn the_inspector_exposes_the_complete_provenance_and_audit_registry_and_none_of_it_is_editable(
    ) {
        let conn = mem_db();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "SANDI-T42", None, None, None).unwrap();
        let w = id.to_string();

        // A - the registry is complete, and the blob column is NOT offered.
        let specs = table_specs();
        let registry = [
            "log_sets", "audit_entry", "audit_detail", "zone_set_versions", "run_parameters",
            "run_degradations", "computed_curves_archive", "curve_meta", "ml_models",
        ];
        for table in registry {
            assert!(
                specs.iter().any(|spec| spec.table == table),
                "the inspector must expose provenance table '{table}'"
            );
        }
        let models = specs.iter().find(|spec| spec.table == "ml_models").unwrap();
        assert!(
            !models.columns.contains(&"data"),
            "the joblib blob is never selected - the list_ml_models rule"
        );

        // B - the audit half browses REAL rows written by the real writer.
        record_audit_entry(
            &conn,
            Some(&w),
            "jauhar",
            "HUMAN",
            "ZONES",
            "zone_params",
            Some("T42 fixture"),
            None,
            &[AuditDetail {
                location: "PARAMETER".into(),
                mode: "INPUT".into(),
                unit: Some("gAPI".into()),
                name: "GR_MA".into(),
                value: Some("25".into()),
            }],
        )
        .unwrap();
        let entries = get_table_page(&conn, "audit_entry", None, 0, 50).unwrap();
        assert_eq!(entries.total_rows, 1, "T41 unchanged: the count is the true total");
        let operator_column =
            entries.columns.iter().position(|column| column == "operator").unwrap();
        assert_eq!(entries.rows[0][operator_column].as_deref(), Some("jauhar"));
        let details = get_table_page(&conn, "audit_detail", None, 0, 50).unwrap();
        let name_column = details.columns.iter().position(|column| column == "name").unwrap();
        assert_eq!(details.rows[0][name_column].as_deref(), Some("GR_MA"));

        // C - the run-provenance half: seed one row per table and read it back through the
        // declared columns (each table's WRITER is pinned by its own suite; this pins the
        // read path and the declared column lists against the live schema).
        let set = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO log_sets (set_id, well_id, set_name, version, module) VALUES (?1, ?2, 'INTERP', 1, 'phi_den')",
            params![set, w],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO run_parameters (set_id, position, name, value_json, source) VALUES (?1, 0, 'RHO_SH', '2.5', 'T42 fixture source')",
            params![set],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO run_degradations (set_id, position, module, kind, detail, occurrences) VALUES (?1, 0, 'phi_den', 'DEFAULTED', 'T42 fixture', 3)",
            params![set],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO computed_curves_archive (set_id, well_id, depth, curve_name, value) VALUES (?1, ?2, 1000.0, 'PHIE', 0.21)",
            params![set, w],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO zone_set_versions (well_id, version, digest) VALUES (?1, 1, 'sha:fixture')",
            params![w],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO curve_meta (curve_id, well_id, set_name, mnemonic, unit) VALUES (?1, ?2, 'RAW', 'GR', 'gAPI')",
            params![Uuid::new_v4().to_string(), w],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ml_models (model_id, name, task, algorithm, feature_curves, params_json, metrics_json, trained_on, n_train, standardize, data) \
             VALUES (?1, 'PERM_RF', 'regression', 'rf', '[]', '{}', '{}', '[]', 10, 1, ?2)",
            params![Uuid::new_v4().to_string(), vec![0u8; 4]],
        )
        .unwrap();
        let expect = |table: &str, well: Option<&str>, column: &str, value: &str| {
            let page = get_table_page(&conn, table, well, 0, 50)
                .unwrap_or_else(|error| panic!("{table}: {error}"));
            assert_eq!(page.total_rows, 1, "{table} true total");
            let position = page.columns.iter().position(|name| name == column).unwrap();
            assert_eq!(page.rows[0][position].as_deref(), Some(value), "{table}.{column}");
        };
        expect("log_sets", Some(&w), "module", "phi_den");
        expect("run_parameters", None, "source", "T42 fixture source");
        expect("run_degradations", None, "kind", "DEFAULTED");
        expect("computed_curves_archive", Some(&w), "curve_name", "PHIE");
        expect("zone_set_versions", Some(&w), "digest", "sha:fixture");
        expect("curve_meta", Some(&w), "mnemonic", "GR");
        expect("ml_models", None, "name", "PERM_RF");

        // D - READ-ONLY, both sides. The backend offers no update command for any of these
        // (writes are the explicit rule-6 commands, none of which names a registry table),
        // and the frontend catalog is pinned: every registry entry exists with an EMPTY
        // editable list and the SAME well scoping the backend enforces.
        let frontend = include_str!("../../src/ui/dbInspectorPanel.ts");
        for table in registry {
            let spec = specs.iter().find(|spec| spec.table == table).unwrap();
            let entry_start = frontend
                .find(&format!("key: \"{table}\""))
                .unwrap_or_else(|| panic!("the inspector UI must offer '{table}'"));
            // One catalog entry is one source line; taking the line avoids an unbalanced
            // brace literal, which would derail the cfg-test stripper's brace counting.
            let entry = frontend[entry_start..].lines().next().expect("the entry is one line");
            assert!(
                entry.contains("editable: []"),
                "'{table}' must be read-only in the grid"
            );
            assert!(
                entry.contains(&format!("wellScoped: {}", spec.well_scoped)),
                "'{table}' well scoping must match the backend"
            );
        }
    }

    /// T-REP-14, the pager. Off-by-one arithmetic here is not cosmetic: a last page that drops
    /// its final row hides a sample, and one that repeats a row makes the grid disagree with the
    /// count printed above it. Checked on a count that does NOT divide evenly by the page size,
    /// because a total that happens to be a multiple passes both mistakes.
    #[test]
    fn the_inspector_pager_lands_exactly_on_the_last_partial_page() {
        let conn = mem_db();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "SANDI-PAGE", None, None, None).unwrap();
        let w = id.to_string();

        // 250 samples, paged 100 at a time: 100 + 100 + 50.
        let n = 250usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let nan = vec![f32::NAN; n];
        insert_standard_curves(
            &conn, id, depth.clone(), vec![50.0; n], nan.clone(), nan.clone(), nan.clone(),
            nan.clone(), nan,
        )
        .unwrap();

        let page = |offset: usize| get_table_page(&conn, "standard_curves", Some(&w), offset, 100).unwrap();
        assert_eq!(page(0).total_rows, n, "the count is the whole table, not the page");
        assert_eq!(page(0).rows.len(), 100);
        assert_eq!(page(100).rows.len(), 100);
        assert_eq!(page(200).rows.len(), 50, "the last page is the remainder, not a full one");
        assert!(page(250).rows.is_empty(), "one page past the end is empty, not an error");

        // Every sample appears exactly once across the three pages, in depth order — no row
        // dropped at a boundary and none repeated.
        let mut seen: Vec<String> = Vec::new();
        for off in [0usize, 100, 200] {
            for row in page(off).rows {
                seen.push(row[0].clone().expect("depth is never NULL"));
            }
        }
        assert_eq!(seen.len(), n);
        let mut sorted = seen.clone();
        sorted.dedup();
        assert_eq!(sorted.len(), n, "a depth appeared on two pages");
        for pair in seen.windows(2) {
            let (a, b) = (pair[0].parse::<f64>().unwrap(), pair[1].parse::<f64>().unwrap());
            assert!(a < b, "depths must increase across the page boundary: {a} then {b}");
        }
    }

    /// T-REP-16. A cell edit whose row is no longer there must FAIL, not report success having
    /// changed nothing. The grid on screen is a snapshot: a module re-run, a depth shift or a
    /// re-import between the read and the double-click leaves the user editing a row that has
    /// moved. A silent 0-row UPDATE is the worst outcome — the cell shows the new value, the
    /// status bar says it was written, an undo entry is pushed for a change that never happened,
    /// and the database still holds the old number.
    ///
    /// All three sample editors are checked, because they are three separate `execute` calls
    /// with three separate guards, and one of them regressing would be invisible.
    #[test]
    fn an_inspector_edit_on_a_row_that_moved_fails_instead_of_reporting_success() {
        let conn = mem_db();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "SANDI-STALE", None, None, None).unwrap();
        let w = id.to_string();
        let nan = vec![f32::NAN; 2];
        insert_standard_curves(
            &conn, id, vec![1000.0, 1000.5], vec![55.0, 60.0], nan.clone(), nan.clone(),
            nan.clone(), nan.clone(), nan,
        )
        .unwrap();
        crate::equations::write_computed_curve(&conn, &w, &[1000.0, 1000.5], "VSH", &[0.3, 0.4])
            .unwrap();
        insert_core_data(
            &conn, &w, "RAW", None, &[1000.0], &[0.20], &[50.0], &[f32::NAN], &[f32::NAN],
        )
        .unwrap();

        // The depth that IS there edits cleanly — the control for all three refusals below.
        update_standard_sample(&conn, &w, 1000.0, "gr", 77.0).unwrap();
        update_computed_sample(&conn, &w, 1000.0, "VSH", 0.55).unwrap();
        update_core_sample(&conn, &w, 1000.0, "cpor", 0.25).unwrap();

        // A depth that is not. Half a sample off — exactly what a re-run on a shifted grid
        // leaves behind, and close enough that nothing about the request looks wrong.
        for (what, err) in [
            ("standard", update_standard_sample(&conn, &w, 1000.25, "gr", 77.0)),
            ("computed", update_computed_sample(&conn, &w, 1000.25, "VSH", 0.55)),
            ("core", update_core_sample(&conn, &w, 1000.25, "cpor", 0.25)),
        ] {
            let e = match err {
                Ok(()) => panic!("{what}: a 0-row update must not report success"),
                Err(e) => e,
            };
            assert!(
                e.contains("1000.25") && e.contains("refresh"),
                "{what}: the message must name the depth and say what to do: {e}"
            );
        }

        // A curve name that does not exist takes the same path — the user's grid can be stale
        // about WHICH curves a well has, not only about where its samples are.
        assert!(update_computed_sample(&conn, &w, 1000.0, "PHIE", 0.2).is_err());

        // The FOURTH editor the inspector uses — `update_well_field`, behind the Wells grid —
        // used to be the gap: it validated the COLUMN and then ran the UPDATE without checking
        // that anything matched, so an edit against a well that is no longer there reported
        // success and wrote nothing (`docs/review_triage.md` finding 20, fixed 2026-08-01).
        //
        // The route is the Wells grid left open while the well is deleted in the Wells & Tops
        // pane. Rarer than a moved curve sample — a well_id does not drift the way a depth does
        // — and the same silent outcome: the cell shows the new value and an undo entry is
        // pushed for a change that never happened.
        update_well_field(&conn, &w, "field_name", Some("SANDI FIELD")).expect("the well that IS there edits");
        let e = update_well_field(&conn, "00000000-0000-0000-0000-000000000000", "field_name", Some("x"))
            .expect_err("editing a well that does not exist must not report success");
        assert!(
            e.contains("no longer in the project") && e.contains("refresh"),
            "the message must say what happened and what to do — the identity here is a UUID the \
             user never sees, so naming it would help nobody: {e}"
        );
        // The column check still works, and is a DIFFERENT refusal — a bad column is a
        // programming error, a missing row is a stale grid.
        assert!(update_well_field(&conn, &w, "depth", Some("x")).unwrap_err().contains("not editable"));
    }

    /// T-REP-16 step 4. Aux Data is browsable but not editable, and that is a data-integrity
    /// rule rather than a missing feature: a point sample is what a laboratory reported, so
    /// correcting it means re-importing the delivery, which keeps the set model and the
    /// provenance intact. The inspector offers no editable column for it — and there is no
    /// backend writer either, which is what makes the rule hold rather than merely be observed.
    ///
    /// Pinned by exhaustion over the whitelist: every column `aux_data` exposes is rejected by
    /// every sample editor that exists. A new editor accepting one of these names would fail
    /// here rather than quietly making a lab result editable in place.
    #[test]
    fn aux_data_can_be_browsed_but_no_editor_will_write_to_it() {
        let conn = mem_db();
        let id = Uuid::new_v4();
        insert_well(&conn, id, "SANDI-AUX", None, None, None).unwrap();
        let w = id.to_string();
        insert_aux_data(
            &conn,
            &w,
            "XRD",
            "RAW",
            None,
            &[AuxRow {
                dataset: "XRD".into(), depth_top: 1000.0, depth_base: None,
                item: "ILLITE".into(), value_num: Some(12.5), value_text: None,
            }],
        )
        .unwrap();

        let page = get_table_page(&conn, "aux_data", Some(&w), 0, 200).unwrap();
        assert_eq!(page.total_rows, 1, "the row is readable");
        let item_idx = page.columns.iter().position(|c| c == "item").unwrap();
        assert_eq!(page.rows[0][item_idx].as_deref(), Some("ILLITE"));

        // Nothing writes to it. The numeric editors reject every one of its columns by name,
        // including `value_num`, which is the one a reader would assume is editable.
        let specs = table_specs();
        let aux_columns = &specs.iter().find(|spec| spec.table == "aux_data").unwrap().columns;
        for col in aux_columns {
            assert!(
                update_standard_sample(&conn, &w, 1000.0, col, 1.0).is_err(),
                "the standard-curve editor accepted aux column '{col}'"
            );
            assert!(
                update_core_sample(&conn, &w, 1000.0, col, 1.0).is_err(),
                "the core editor accepted aux column '{col}'"
            );
            assert!(
                update_well_field(&conn, &w, col, Some("x")).is_err(),
                "the wells editor accepted aux column '{col}'"
            );
        }
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

    /// Provenance rule applied to the schema: the Phase-0 stub table named after a client
    /// study is dropped from old projects and never created in new ones. Pinned from BOTH
    /// sides — a migration that drops it while `create_schema` still declares it would pass
    /// the first half alone, and vice versa. The literal below is confined to the migration
    /// and this test; it exists to remove itself.
    #[test]
    fn a_study_named_stub_table_is_dropped_from_old_projects_and_never_created_in_new_ones() {
        let has_table = |conn: &Connection| -> i64 {
            conn.query_row(
                "SELECT COUNT(*) FROM duckdb_tables() WHERE table_name = 'lqr_parameters'",
                [],
                |r| r.get(0),
            )
            .unwrap()
        };

        // Side 1: an OLD database carries the stub exactly as Phase 0 declared it.
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE lqr_parameters (
                 well_id UUID NOT NULL, depth FLOAT NOT NULL, clay_volume FLOAT,
                 capillary_pressure FLOAT, microporosity FLOAT, PRIMARY KEY (well_id, depth)
             );",
        )
        .unwrap();
        assert_eq!(has_table(&conn), 1, "fixture starts with the stub");
        migrate_drop_study_named_stub(&conn).unwrap();
        assert_eq!(has_table(&conn), 0, "stub dropped");
        migrate_drop_study_named_stub(&conn).unwrap();
        assert_eq!(has_table(&conn), 0, "idempotent");

        // Side 2: a fresh schema never creates it in the first place.
        let fresh = mem_db();
        assert_eq!(has_table(&fresh), 0, "create_schema no longer declares the stub");
        migrate_drop_study_named_stub(&fresh).unwrap();
        assert_eq!(has_table(&fresh), 0, "and the migration is a no-op on it");
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

    /// The item picker's catalogue. Two contracts: it follows the ACTIVE delivery like every
    /// other point-data reader (a superseded CEC suite must not appear as a choice), and it
    /// separates NUMERIC items from descriptive ones — a lithology description cannot set a
    /// scaling factor, and offering it would produce a run that fails for invisible reasons.
    #[test]
    fn the_aux_item_catalog_follows_the_active_delivery_and_flags_text_only_items() {
        let conn = mem_db();
        let wid = Uuid::new_v4();
        insert_well(&conn, wid, "SANDI-CAT", None, None, Some(0.0)).unwrap();
        let w = wid.to_string();
        let row = |item: &str, num: Option<f32>, text: Option<&str>| AuxRow {
            dataset: "CEC".into(),
            depth_top: 1000.0,
            depth_base: None,
            item: item.into(),
            value_num: num,
            value_text: text.map(str::to_string),
        };
        // First delivery, then a second one that supersedes it with a DIFFERENT item name.
        insert_aux_data(&conn, &w, "CEC", "RAW", None, &[row("CEC_OLD", Some(4.0), None)]).unwrap();
        insert_aux_data(
            &conn,
            &w,
            "CEC",
            "LAB2024",
            None,
            &[
                row("CEC", Some(4.2), None),
                row("CEC", Some(5.1), None),
                row("METHOD", None, Some("ammonium acetate")),
            ],
        )
        .unwrap();

        let cat = list_aux_item_catalog(&conn).unwrap();
        let names: Vec<&str> = cat.iter().map(|c| c.item.as_str()).collect();
        assert!(!names.contains(&"CEC_OLD"), "a superseded delivery is not a choice: {names:?}");

        let cec = cat.iter().find(|c| c.item == "CEC").expect("CEC listed");
        assert_eq!((cec.rows, cec.numeric_rows, cec.wells), (2, 2, 1));

        let method = cat.iter().find(|c| c.item == "METHOD").expect("METHOD listed");
        assert_eq!(method.numeric_rows, 0, "a descriptive item carries no number to fit");
        assert_eq!(method.rows, 1);
    }

    /// A core depth shift must move the measurements that were made ON those plugs, or the
    /// porosity registers against the log while the core gamma that justified the shift does not.
    /// The separately-delivered dataset must NOT move just because its set is also called RAW.
    #[test]
    fn a_core_shift_carries_the_plug_extras_and_leaves_other_deliveries_alone() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        insert_well(&conn, wid, "SANDI-SHIFT", None, None, None).unwrap();
        let w = wid.to_string();

        let d = [2000.0f32, 2001.0, 2002.0];
        let nan = [f32::NAN; 3];
        insert_core_data(&conn, &w, "RAW", None, &d, &[0.2, 0.21, 0.19], &nan, &nan, &nan).unwrap();

        // Extras of THIS core delivery: written under the core set's own name (see ingest.rs).
        let extra = |item: &str, depth: f32, base: Option<f32>| AuxRow {
            dataset: "CORE".into(),
            depth_top: depth,
            depth_base: base,
            item: item.into(),
            value_num: Some(55.0),
            value_text: None,
        };
        insert_aux_data(
            &conn,
            &w,
            "CORE",
            "RAW",
            None,
            &[extra("CORE_GR", 2000.0, None), extra("KVKH", 2001.0, Some(2001.5))],
        )
        .unwrap();
        // A separate delivery whose set is ALSO called RAW — the collision the naive rule hits.
        insert_aux_data(&conn, &w, "XRD", "RAW", None, &[extra("KAOLINITE", 2000.0, None)]).unwrap();

        let auto = core_extra_datasets(&conn, &w).unwrap();
        assert_eq!(
            auto,
            vec![("CORE".to_string(), 2), ("XRD".to_string(), 1)],
            "both share the core set's name, so both are OFFERED — the user decides"
        );

        let moved = shift_core_depths(&mut conn, &w, 2.5, &ShiftTargets::aux(vec!["CORE".to_string()]), &Default::default()).unwrap();
        assert_eq!((moved.plugs, moved.extras), (3, 2));

        let plug: f32 = conn
            .query_row("SELECT MIN(depth) FROM core_data WHERE well_id = ?1", params![w], |r| r.get(0))
            .unwrap();
        assert!((plug - 2002.5).abs() < 1e-4, "plugs moved");

        let gr: f32 = conn
            .query_row(
                "SELECT depth_top FROM aux_data WHERE well_id = ?1 AND item = 'CORE_GR'",
                params![w],
                |r| r.get(0),
            )
            .unwrap();
        assert!((gr - 2002.5).abs() < 1e-4, "the core gamma moved WITH its plugs");

        let (top, base): (f32, f32) = conn
            .query_row(
                "SELECT depth_top, depth_base FROM aux_data WHERE well_id = ?1 AND item = 'KVKH'",
                params![w],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!((top - 2003.5).abs() < 1e-4 && (base - 2004.0).abs() < 1e-4, "an interval keeps its thickness");

        let xrd: f32 = conn
            .query_row(
                "SELECT depth_top FROM aux_data WHERE well_id = ?1 AND item = 'KAOLINITE'",
                params![w],
                |r| r.get(0),
            )
            .unwrap();
        assert!((xrd - 2000.0).abs() < 1e-4, "a dataset not named in the call must not move");

        // Exactly reversible, which is what makes it undoable.
        shift_core_depths(&mut conn, &w, -2.5, &ShiftTargets::aux(vec!["CORE".to_string()]), &Default::default()).unwrap();
        let back: f32 = conn
            .query_row(
                "SELECT depth_top FROM aux_data WHERE well_id = ?1 AND item = 'CORE_GR'",
                params![w],
                |r| r.get(0),
            )
            .unwrap();
        assert!((back - 2000.0).abs() < 1e-4);
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
            ..Default::default()
        }
    }

    /// Conditioning a core photograph must be reversible to the byte, and reversible to the SHAPE.
    ///
    /// Three claims, and the third is the one that would have shipped broken. The import is kept
    /// the first time and never again, so editing a recipe re-renders from the photograph rather
    /// than stacking a second correction on the first. Clearing restores those exact bytes. And it
    /// restores `width`/`height`/`mime` as well — a crop changes the picture's shape, so a restore
    /// that left the cropped dimensions behind would have every renderer draw the whole photograph
    /// into the cropped one's box, at the wrong aspect ratio.
    #[test]
    fn conditioning_keeps_the_import_and_a_restore_puts_back_its_shape() {
        let conn = mem_db();
        let wid = Uuid::new_v4();
        insert_well(&conn, wid, "SANDI-CP", None, None, None).unwrap();
        let w = wid.to_string();
        let import: &[u8] = b"\xFF\xD8the-photograph-as-delivered\xFF\xD9";
        insert_well_images(&conn, &w, "CORE PHOTO", "RUN1", None, &[a_plate("BOX-1", 1000.0, Some(1003.0), import)])
            .unwrap();
        let id = list_well_images(&conn, &w, None).unwrap()[0].image_id.clone();

        // Nothing conditioned yet: the source IS the picture, and the recipe is empty.
        assert_eq!(get_well_image_source(&conn, &id).unwrap().1, import);
        assert_eq!(list_image_recipes(&conn, &w, "CORE PHOTO").unwrap(), vec![(id.clone(), String::new())]);

        bake_image_conditioned(&conn, &id, r#"{"exposure":0.4}"#, b"first-bake", "image/jpeg", 700, 500)
            .unwrap();
        assert_eq!(get_well_image(&conn, &id).unwrap().1, b"first-bake");
        assert_eq!(get_well_image_source(&conn, &id).unwrap().1, import, "the import is kept");
        let info = &list_well_images(&conn, &w, None).unwrap()[0];
        assert_eq!((info.width, info.height), (700, 500));

        // A second bake re-renders from the IMPORT. If the kept copy moved, the correction would be
        // permanent and the next edit would be conditioning an already-conditioned photograph.
        bake_image_conditioned(&conn, &id, r#"{"exposure":0.9}"#, b"second-bake", "image/jpeg", 640, 480)
            .unwrap();
        assert_eq!(get_well_image_source(&conn, &id).unwrap().1, import, "kept once, never again");
        assert_eq!(
            list_image_recipes(&conn, &w, "CORE PHOTO").unwrap()[0].1,
            r#"{"exposure":0.9}"#,
            "the recipe on the row is the one its pixels were made with"
        );

        clear_image_conditioning(&conn, &id).unwrap();
        assert_eq!(get_well_image(&conn, &id).unwrap().1, import, "back to the delivered bytes");
        let info = &list_well_images(&conn, &w, None).unwrap()[0];
        assert_eq!((info.width, info.height), (800, 600), "and back to the delivered shape");
        assert_eq!(info.mime, "image/jpeg");
        assert_eq!(list_image_recipes(&conn, &w, "CORE PHOTO").unwrap()[0].1, "");
        // The kept copy is dropped, so a photograph with nothing to undo stops carrying two blobs.
        let kept: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM well_images WHERE image_id = ?1 AND source_data IS NOT NULL",
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(kept, 0);

        // Clearing a picture nobody conditioned is a no-op, not an error and not a wipe.
        clear_image_conditioning(&conn, &id).unwrap();
        assert_eq!(get_well_image(&conn, &id).unwrap().1, import);
    }

    /// Pictures follow the universal delivery-set rule: a second delivery lands BESIDE the
    /// first and only one is live, so a re-shot core cannot double the plates on a track.
    #[test]
    fn a_second_image_delivery_lands_beside_the_first_and_only_one_is_live() {
        let conn = mem_db();
        let wid = Uuid::new_v4();
        insert_well(&conn, wid, "SANDI-IMG", None, None, None).unwrap();
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
        insert_well(&conn, wid, "SANDI-IMG2", None, None, None).unwrap();
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

    /// AUDIT-2026-08-20 finding 25. The datum guard sat on the SCREEN reader only, so a picture
    /// delivery declaring TVD or TVDSS showed NOTHING on screen and printed EVERY plate - and
    /// the print is the half that ships. (The core pair had the same split in the opposite
    /// direction and was closed by the core-datum increment; this is the other half of the same
    /// finding.) A plate is placed against the MD log frame exactly as a core plug is, so a
    /// cross-datum delivery puts it beside the wrong rock.
    ///
    /// Pinned from BOTH sides. A refusal alone would also be produced by a print reader that had
    /// simply stopped returning pictures, so an MD delivery must still print; and the refusal
    /// must NAME both datums and the delivery, because the guard is about a WRONG datum and
    /// never about having declared one.
    #[test]
    fn the_print_reader_refuses_a_cross_datum_picture_delivery_exactly_as_the_screen_does() {
        let conn = mem_db();
        let wid = Uuid::new_v4();
        insert_well(&conn, wid, "SANDI-IMG-DATUM", None, None, None).unwrap();
        let w = wid.to_string();
        let bytes = b"\xFF\xD8_______________\xFF\xD9";
        insert_well_images(&conn, &w, "CORE PHOTO", "RAW", None, &[a_plate("CP-1", 1000.0, Some(1001.0), bytes)])
            .unwrap();
        let print = |c: &Connection| read_images_for_print(c, &w, "CORE PHOTO", 999.0, 1010.0);
        let screen = |c: &Connection| list_well_images(c, &w, Some("CORE PHOTO"));

        // Declared MD: both sides carry the plate, as they always have.
        declare_set_datum(&conn, "image_sets", &w, Some("CORE PHOTO"), "RAW", "MD").unwrap();
        assert_eq!(screen(&conn).unwrap().len(), 1, "an MD delivery draws on screen");
        assert_eq!(print(&conn).unwrap().len(), 1, "and prints");

        // Declared TVDSS: both refuse, naming both datums and the delivery.
        declare_set_datum(&conn, "image_sets", &w, Some("CORE PHOTO"), "RAW", "TVDSS").unwrap();
        for (what, text) in [
            ("list_well_images", screen(&conn).expect_err("the screen must refuse").to_string()),
            ("read_images_for_print", print(&conn).expect_err("the print must refuse too").to_string()),
        ] {
            assert!(
                text.contains("TVDSS") && text.contains("MD") && text.contains("CORE PHOTO"),
                "{what} must name both datums and the delivery, got: {text}"
            );
        }
    }

    #[test]
    fn deleting_the_live_image_delivery_hands_over_to_the_next_newest() {
        let conn = mem_db();
        let wid = Uuid::new_v4();
        insert_well(&conn, wid, "SANDI-IMG3", None, None, None).unwrap();
        let w = wid.to_string();
        insert_well_images(&conn, &w, "SEM", "RUN1", None, &[a_plate("A", 1000.0, None, b"\xFF\xD8a\xFF\xD9")]).unwrap();
        insert_well_images(&conn, &w, "SEM", "RUN2", None, &[a_plate("B", 1001.0, None, b"\xFF\xD8b\xFF\xD9")]).unwrap();
        assert_eq!(list_well_images(&conn, &w, None).unwrap()[0].name, "B");

        assert_eq!(delete_image_set(&conn, &w, "SEM", "RUN2").unwrap(), 1);
        let live = list_well_images(&conn, &w, None).unwrap();
        assert_eq!(live.len(), 1, "the survivor takes over rather than leaving the track blank");
        assert_eq!(live[0].name, "A");
    }

    /// Sets up a well with plugs every metre from `top`, all at their delivered depths.
    fn cored_well(conn: &Connection, top: f32, n: usize) -> String {
        let wid = Uuid::new_v4();
        insert_well(conn, wid, "SANDI-RUN", None, None, None).unwrap();
        let w = wid.to_string();
        let d: Vec<f32> = (0..n).map(|i| top + i as f32).collect();
        let v: Vec<f32> = (0..n).map(|i| 0.20 + 0.001 * i as f32).collect();
        let nan = vec![f32::NAN; n];
        insert_core_data(conn, &w, "RAW", None, &d, &v, &nan, &nan, &nan).unwrap();
        w
    }

    /// "Why is this core at this depth?" has to have an answer next year, and the answer has to
    /// be written by the same transaction that moved it — a shift that commits without its reason
    /// is the state the record exists to prevent.
    #[test]
    fn a_shift_records_why_the_core_moved() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let w = cored_well(&conn, 2000.0, 20);

        let note = RegistrationNote {
            kind: "proposed".into(),
            log_curve: "GR".into(),
            reference: "CORE_GR".into(),
            pairing: "like-for-like".into(),
            correlation: Some(0.87),
            n_pairs: Some(18),
            note: String::new(),
        };
        shift_core_depths(&mut conn, &w, 2.0, &Default::default(), &note).unwrap();

        let log = list_core_registrations(&conn, &w).unwrap();
        assert_eq!(log.len(), 1);
        let e = &log[0];
        assert_eq!(e.set_name, "RAW", "the delivery that moved is named, not resolved later");
        assert!((e.delta - 2.0).abs() < 1e-6);
        assert_eq!(e.log_curve, "GR");
        assert_eq!(e.reference, "CORE_GR");
        assert_eq!(e.pairing, "like-for-like");
        assert!((e.correlation.unwrap() - 0.87).abs() < 1e-6);
        assert_eq!(e.n_pairs, Some(18));
        // A whole-core shift declared no range, and that is a statement rather than a gap.
        assert!(e.top.is_none() && e.base.is_none(), "no range was declared");
    }

    /// The record is an EVENT LOG. A core that was registered, judged wrong and put back is not
    /// the same as a core nobody ever touched — and deleting the reversed row is exactly what
    /// would make those two read alike.
    #[test]
    fn an_undo_appends_a_reversal_instead_of_erasing_the_record() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let w = cored_well(&conn, 2000.0, 20);

        shift_core_depths(&mut conn, &w, 2.0, &Default::default(), &RegistrationNote {
            kind: "proposed".into(),
            reference: "CORE_GR".into(),
            ..Default::default()
        })
        .unwrap();
        shift_core_depths(&mut conn, &w, -2.0, &Default::default(), &RegistrationNote {
            kind: "undo".into(),
            ..Default::default()
        })
        .unwrap();

        let log = list_core_registrations(&conn, &w).unwrap();
        assert_eq!(log.len(), 2, "the reversal is appended, not swapped for the row it reverses");
        // Newest first: the undo, then what it undid.
        assert_eq!(log[0].kind, "undo");
        assert!((log[0].delta + 2.0).abs() < 1e-6);
        assert_eq!(log[1].kind, "proposed");
        assert_eq!(log[1].reference, "CORE_GR");
        assert!(log[0].seq > log[1].seq, "seq orders the history within a delivery");

        // And the core really is back where it started, so the log is the ONLY thing that still
        // remembers it moved.
        let pairs = core_depth_pairs(&conn, &w).unwrap();
        assert!(pairs.iter().all(|(orig, now)| (orig - now).abs() < 1e-3));
    }

    /// Two barrels corrected by different amounts is the case the record has to preserve —
    /// collapsing it to one line would describe a shift that was never applied.
    #[test]
    fn each_barrel_gets_its_own_line_in_the_record() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let w = cored_well(&conn, 2000.0, 20);

        apply_core_run_shifts(
            &mut conn,
            &w,
            &[
                RunShift { top: 2000.0, base: 2009.0, delta: 1.0, correlation: Some(0.91), n_pairs: Some(9) },
                RunShift { top: 2010.0, base: 2019.0, delta: 3.0, correlation: Some(0.62), n_pairs: Some(8) },
            ],
            &Default::default(),
            &RegistrationNote { kind: "proposed".into(), log_curve: "GR".into(), ..Default::default() },
        )
        .unwrap();

        let mut log = list_core_registrations(&conn, &w).unwrap();
        assert_eq!(log.len(), 2);
        log.sort_by(|a, b| a.seq.cmp(&b.seq));
        assert_eq!((log[0].top, log[0].base), (Some(2000.0), Some(2009.0)));
        assert!((log[0].delta - 1.0).abs() < 1e-6);
        assert_eq!((log[1].top, log[1].base), (Some(2010.0), Some(2019.0)));
        assert!((log[1].delta - 3.0).abs() < 1e-6);
        assert!(log.iter().all(|e| e.log_curve == "GR"), "one apply, one reason");
        // Each barrel was proposed on its own correlogram, so the confidence is per range. One
        // number for the apply would file the well-matched barrel's r against the doubtful one.
        assert!((log[0].correlation.unwrap() - 0.91).abs() < 1e-6);
        assert!((log[1].correlation.unwrap() - 0.62).abs() < 1e-6);
        assert_eq!((log[0].n_pairs, log[1].n_pairs), (Some(9), Some(8)));
    }

    /// A well with no core has no depth history. A row saying nothing happened is noise in the
    /// one place that has to stay readable.
    #[test]
    fn a_shift_that_moved_nothing_writes_no_record() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        insert_well(&conn, wid, "SANDI-DRY", None, None, None).unwrap();
        let w = wid.to_string();

        let n = shift_core_depths(&mut conn, &w, 2.0, &Default::default(), &Default::default()).unwrap();
        assert_eq!(n.plugs, 0);
        assert!(list_core_registrations(&conn, &w).unwrap().is_empty());
    }

    /// Each barrel carries its own tally error, so the shifts differ down the hole — and the
    /// delivered depth is kept untouched so a later delivery can still be placed.
    #[test]
    fn each_barrel_can_be_shifted_by_its_own_amount() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let w = cored_well(&conn, 2000.0, 30); // 2000 .. 2029

        let runs = [
            RunShift { top: 2000.0, base: 2009.0, delta: 1.0, ..Default::default() },
            RunShift { top: 2010.0, base: 2019.0, delta: 2.0, ..Default::default() },
            RunShift { top: 2020.0, base: 2029.0, delta: 3.5, ..Default::default() },
        ];
        let n = apply_core_run_shifts(&mut conn, &w, &runs, &Default::default(), &Default::default()).unwrap();
        assert_eq!(n.plugs, 30);

        let pairs = core_depth_pairs(&conn, &w).unwrap();
        let at = |orig: f32| pairs.iter().find(|p| (p.0 - orig).abs() < 1e-4).unwrap().1;
        assert!((at(2000.0) - 2001.0).abs() < 1e-3, "first barrel moved 1");
        assert!((at(2015.0) - 2017.0).abs() < 1e-3, "second barrel moved 2");
        assert!((at(2029.0) - 2032.5).abs() < 1e-3, "third barrel moved 3.5");
        assert_eq!(pairs.len(), 30, "the delivered depths are all still recorded");
    }

    /// The rule that cannot be relaxed: no set of shifts may put deeper rock above shallower rock.
    #[test]
    fn a_shift_that_would_reorder_the_core_is_refused_and_changes_nothing() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let w = cored_well(&conn, 2000.0, 10); // 2000 .. 2009

        // Push the upper barrel 6 m down and leave the lower one — they would cross.
        let runs = [RunShift { top: 2000.0, base: 2004.0, delta: 6.0, ..Default::default() }];
        let err = apply_core_run_shifts(&mut conn, &w, &runs, &Default::default(), &Default::default()).unwrap_err();
        assert!(err.contains("reorders the core"), "{err}");

        let pairs = core_depth_pairs(&conn, &w).unwrap();
        assert!(
            pairs.iter().all(|(o, d)| (o - d).abs() < 1e-4),
            "a refused shift must leave every plug exactly where it was"
        );
    }

    /// Undoing per-barrel shifts must put every plug back exactly.
    ///
    /// The obvious inverse — negate each delta and shift the user's own ranges — is wrong, and
    /// quietly. Barrels that never overlapped can land on ranges that DO once each moves by a
    /// different amount, and the first matching range wins, so some plugs come back by their
    /// neighbour's correction. This is the case that caught it.
    #[test]
    fn undoing_per_barrel_shifts_returns_every_plug_to_where_it_started() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let w = cored_well(&conn, 2000.0, 60); // 2000 .. 2059

        // Declared ranges reach past the plugs they hold — exactly how a user types a barrel —
        // and the upper barrel moves 0.5 m FURTHER than the lower one. That is legal (the plugs
        // stay 0.5 m apart at the join, having been 1 m apart) yet it makes the naive inverse
        // ranges overlap by 0.4 m, right where the lower barrel's first plug lands.
        let runs = [
            RunShift { top: 1995.0, base: 2029.5, delta: 2.0, ..Default::default() },
            RunShift { top: 2029.6, base: 2065.0, delta: 1.5, ..Default::default() },
        ];
        let before = core_depth_pairs(&conn, &w).unwrap();
        let res = apply_core_run_shifts(&mut conn, &w, &runs, &Default::default(), &Default::default()).unwrap();
        assert_eq!(res.plugs, 60);

        // The naive inverse — the caller's own ranges, shifted, deltas negated — DOES overlap.
        // Proving that here is the point: it is what makes the computed inverse necessary.
        let naive: Vec<(f32, f32)> =
            runs.iter().map(|r| (r.top + r.delta, r.base + r.delta)).collect();
        assert!(
            naive[1].0 <= naive[0].1,
            "this test is pointless unless the naive inverse really overlaps: {naive:?}"
        );

        // The computed inverse meets at a single point instead of overlapping across 0.4 m. That
        // point is the midpoint of two distinct plug depths, so it is strictly between them and no
        // plug can sit on it — which is what matters. The exact round trip below is the real proof.
        assert_eq!(res.inverse.len(), 2);
        assert!(
            res.inverse[1].top >= res.inverse[0].base,
            "the computed inverse must not overlap, got {:?}",
            res.inverse
        );
        let overlap = res.inverse[0].base - res.inverse[1].top;
        assert!(overlap <= 0.0, "overlap of {overlap} would make an undo ambiguous");

        apply_core_run_shifts(&mut conn, &w, &res.inverse, &Default::default(), &Default::default()).unwrap();
        let after = core_depth_pairs(&conn, &w).unwrap();
        assert_eq!(before.len(), after.len());
        for (b, a) in before.iter().zip(&after) {
            assert!(
                (b.0 - a.0).abs() < 1e-3 && (b.1 - a.1).abs() < 1e-3,
                "plug {b:?} came back as {a:?}"
            );
        }
    }

    /// Re-registering a core months later must carry the deliveries that sit on ITS depth scale —
    /// the XRD, the SCAL plugs, the thin sections — and must leave alone anything that does not.
    #[test]
    fn a_later_registration_carries_the_deliveries_that_sit_on_core_depths() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let w = cored_well(&conn, 2000.0, 20); // 2000 .. 2019
        let jpg = b"\xFF\xD8x\xFF\xD9";

        let aux_row = |ds: &str, item: &str, d: f32| AuxRow {
            dataset: ds.into(),
            depth_top: d,
            depth_base: None,
            item: item.into(),
            value_num: Some(1.0),
            value_text: None,
        };
        // On core depths (declared at import) …
        insert_aux_data(&conn, &w, "XRD", "LAB", None, &[aux_row("XRD", "KAOLINITE", 2005.0)]).unwrap();
        mark_aux_set_on_core(&conn, &w, "XRD", "LAB").unwrap();
        // … and NOT (a perforation record is on the driller's/log scale).
        insert_aux_data(&conn, &w, "PERFORATION", "RAW", None, &[aux_row("PERFORATION", "SHOTS", 2005.0)]).unwrap();

        insert_scal_pc(
            &conn,
            &w,
            "SCAL",
            None,
            &[ScalPcRow {
                sample_no: Some(1),
                depth: Some(2005.0),
                perm: 100.0,
                poro: 0.2,
                pc: 1.0,
                sw: 1.0,
                system: None,
                ift: Some(72.0),
            }],
        )
        .unwrap();
        mark_scal_set_on_core(&conn, &w, "SCAL").unwrap();

        insert_well_images(&conn, &w, "THIN SECTION", "LAB", None, &[a_plate("TS-1", 2005.0, None, jpg)]).unwrap();
        mark_image_set_on_core(&conn, &w, "THIN SECTION", "LAB").unwrap();

        // What the dialog would show, and how it would pre-tick.
        let cands = core_shift_candidates(&conn, &w).unwrap();
        let on: Vec<&str> = cands.iter().filter(|c| c.on_core_depths).map(|c| c.kind.as_str()).collect();
        assert_eq!(on.len(), 3, "XRD, SCAL and the sections are on core depths: {cands:?}");
        assert!(
            cands.iter().any(|c| c.dataset == "PERFORATION" && !c.on_core_depths),
            "a log-depth delivery is offered but NOT pre-ticked: {cands:?}"
        );

        let targets = ShiftTargets {
            aux_datasets: vec!["XRD".into()],
            scal: true,
            image_datasets: vec!["THIN SECTION".into()],
        };
        let n = apply_core_run_shifts(&mut conn, &w, &[RunShift { top: 2000.0, base: 2019.0, delta: 3.0, ..Default::default() }], &targets, &Default::default())
            .unwrap();
        assert_eq!((n.plugs, n.extras, n.scal, n.plates), (20, 1, 1, 1));

        let at = |sql: &str| -> f32 { conn.query_row(sql, params![w], |r| r.get(0)).unwrap() };
        assert!(
            (at("SELECT depth_top FROM aux_data WHERE well_id = ?1 AND item = 'KAOLINITE'") - 2008.0).abs() < 1e-3,
            "the XRD moved with the core"
        );
        assert!(
            (at("SELECT depth FROM scal_pc WHERE well_id = ?1") - 2008.0).abs() < 1e-3,
            "the SCAL plug moved with the core"
        );
        assert!(
            (at("SELECT depth_top FROM well_images WHERE well_id = ?1") - 2008.0).abs() < 1e-3,
            "the thin section moved with its plug"
        );
        assert!(
            (at("SELECT depth_top FROM aux_data WHERE well_id = ?1 AND item = 'SHOTS'") - 2005.0).abs() < 1e-3,
            "a delivery on the LOG's scale must not be dragged along"
        );
    }

    /// Two barrels cannot claim the same rock — with overlapping ranges "which barrel was this
    /// plug in?" stops having an answer.
    #[test]
    fn overlapping_barrel_ranges_are_refused() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let w = cored_well(&conn, 2000.0, 20);
        let runs = [
            RunShift { top: 2000.0, base: 2010.0, delta: 0.5, ..Default::default() },
            RunShift { top: 2008.0, base: 2019.0, delta: 0.5, ..Default::default() },
        ];
        let err = apply_core_run_shifts(&mut conn, &w, &runs, &Default::default(), &Default::default()).unwrap_err();
        assert!(err.contains("overlap"), "{err}");

        // Adjacent barrels written the natural way — sharing one depth — are NOT an overlap.
        let touching = [
            RunShift { top: 2000.0, base: 2010.0, delta: 0.5, ..Default::default() },
            RunShift { top: 2010.0, base: 2019.0, delta: 0.5, ..Default::default() },
        ];
        assert!(apply_core_run_shifts(&mut conn, &w, &touching, &Default::default(), &Default::default()).is_ok());
    }

    /// The payoff Jauhar asked for: a laboratory sends XRD months later at the depths the core
    /// report used, and it lands where that rock now is — including where a barrel moved by a
    /// different amount than its neighbour.
    #[test]
    fn a_later_delivery_follows_the_core_that_was_already_shifted() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let w = cored_well(&conn, 2000.0, 30);
        let runs = [
            RunShift { top: 2000.0, base: 2009.0, delta: 1.0, ..Default::default() },
            RunShift { top: 2010.0, base: 2029.0, delta: 3.0, ..Default::default() },
        ];
        apply_core_run_shifts(&mut conn, &w, &runs, &Default::default(), &Default::default()).unwrap();
        let pairs = core_depth_pairs(&conn, &w).unwrap();

        // A sample on a plug lands exactly on that plug.
        let (d, ex) = map_core_depth(&pairs, 2005.0);
        assert!((d - 2006.0).abs() < 1e-3, "got {d}");
        assert!(!ex);
        let (d, ex) = map_core_depth(&pairs, 2025.0);
        assert!((d - 2028.0).abs() < 1e-3, "got {d}");
        assert!(!ex);

        // Between plugs the correction is interpolated — pieces move inside a barrel, so the
        // offset really does vary along the core.
        let (d, _) = map_core_depth(&pairs, 2009.5);
        assert!((d - 2011.5).abs() < 1e-3, "half way between a 1 m and a 3 m shift: got {d}");

        // Outside the cored interval there is no evidence, so the end correction is held AND the
        // caller is told it was extrapolated rather than measured.
        let (d, ex) = map_core_depth(&pairs, 1990.0);
        assert!((d - 1991.0).abs() < 1e-3);
        assert!(ex, "above the core is a guess and must say so");
        let (_, ex) = map_core_depth(&pairs, 2100.0);
        assert!(ex, "below the core is a guess and must say so");
    }

    /// An un-shifted core maps every depth to itself — the feature costs nothing until used.
    #[test]
    fn an_unregistered_core_maps_every_depth_to_itself() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let w = cored_well(&conn, 2000.0, 10);
        let pairs = core_depth_pairs(&conn, &w).unwrap();
        for probe in [1999.0, 2000.0, 2004.5, 2009.0, 2050.0] {
            let (d, _) = map_core_depth(&pairs, probe);
            assert!((d - probe).abs() < 1e-4, "{probe} moved to {d}");
        }
        assert_eq!(map_core_depth(&[], 1234.0), (1234.0, false), "no core at all is a no-op");
    }

    /// A whole-well shift and a per-barrel shift must agree about the record they leave behind,
    /// or the two routes would disagree about where a later delivery goes.
    #[test]
    fn a_plain_shift_leaves_the_same_record_a_run_shift_does() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let w = cored_well(&conn, 2000.0, 10);
        shift_core_depths(&mut conn, &w, 2.5, &Default::default(), &Default::default()).unwrap();
        let pairs = core_depth_pairs(&conn, &w).unwrap();
        assert!(pairs.iter().all(|(o, d)| (d - o - 2.5).abs() < 1e-3));
        let (d, ex) = map_core_depth(&pairs, 2003.0);
        assert!((d - 2005.5).abs() < 1e-3 && !ex, "got {d}");
    }

    /// Re-registering a plate delivery: the shift follows the ACTIVE set like every other reader,
    /// leaves other datasets alone, and — the part that matters petrophysically — never gives a
    /// point sample a thickness it does not have.
    #[test]
    fn shifting_plates_moves_the_live_delivery_and_keeps_a_point_a_point() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        insert_well(&conn, wid, "SANDI-IMG4", None, None, None).unwrap();
        let w = wid.to_string();
        let jpg = b"\xFF\xD8x\xFF\xD9";

        // A thin section (point) and a core photograph (real interval), plus a superseded set.
        insert_well_images(&conn, &w, "THIN SECTION", "LAB", None, &[a_plate("TS-1", 1000.0, None, jpg)]).unwrap();
        insert_well_images(&conn, &w, "CORE PHOTO", "RAW", None, &[a_plate("CP-1", 1000.0, Some(1001.0), jpg)])
            .unwrap();
        insert_well_images(&conn, &w, "THIN SECTION", "OLD", None, &[a_plate("TS-OLD", 900.0, None, jpg)]).unwrap();
        set_active_image_set(&conn, &w, "THIN SECTION", "LAB").unwrap();

        assert_eq!(shift_well_images(&conn, &w, Some("THIN SECTION"), 2.5).unwrap(), 1);
        let live = list_well_images(&conn, &w, None).unwrap();
        let ts = live.iter().find(|i| i.name == "TS-1").unwrap();
        assert!((ts.depth_top - 1002.5).abs() < 1e-4);
        assert!(ts.depth_base.is_none(), "a section is cut from one plug and gains no thickness from a shift");
        let cp = live.iter().find(|i| i.name == "CP-1").unwrap();
        assert!((cp.depth_top - 1000.0).abs() < 1e-4, "another dataset must not move");

        // The superseded delivery stays where it was — it is not what anyone is looking at.
        let old: f32 = conn
            .query_row(
                "SELECT depth_top FROM well_images WHERE well_id = ?1 AND set_name = 'OLD'",
                params![w],
                |r| r.get(0),
            )
            .unwrap();
        assert!((old - 900.0).abs() < 1e-4);

        // No dataset given = every live plate in the well, and exactly reversible.
        assert_eq!(shift_well_images(&conn, &w, None, -2.5).unwrap(), 2);
        let live = list_well_images(&conn, &w, None).unwrap();
        assert!((live.iter().find(|i| i.name == "TS-1").unwrap().depth_top - 1000.0).abs() < 1e-4);
        let cp = live.iter().find(|i| i.name == "CP-1").unwrap();
        assert!((cp.depth_top - 997.5).abs() < 1e-4);
        assert!(
            (cp.depth_base.unwrap() - 998.5).abs() < 1e-4,
            "an interval keeps its thickness through a shift"
        );
    }

    /// CHARACTERIZATION — `22_database-model.md` SB-DBM-T02 and dossier T-DB-11:
    /// destructive work is preceded by a reported, non-overwriting copy; additive work
    /// creates none; and a failed copy leaves the source structurally un-migrated.
    #[test]
    fn a_destructive_upgrade_backs_up_before_writing_never_overwrites_reports_the_path_and_aborts_on_backup_failure_while_an_additive_open_takes_no_backup() {
        let dir = std::env::temp_dir().join(format!("sandibumi-rb-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("field.duckdb");
        let db_path_str = db_path.to_str().unwrap().to_string();
        let count_backups = |stem: &str| -> usize {
            std::fs::read_dir(&dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    name.starts_with(stem) && name.contains("-backup")
                })
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
        take_boot_notes();
        let conn = crate::project::open_and_migrate(&db_path_str).unwrap();
        assert_eq!(pk_count(&conn, "computed_curves"), 0, "live file migrated");

        let backup = dir.join("field.pre-0-backup.duckdb");
        assert!(backup.exists(), "an unstamped legacy project is source format 0");
        assert!(
            take_boot_notes().iter().any(|note| note.contains(backup.to_str().unwrap())),
            "the user-facing boot record must name the exact recovery copy"
        );
        {
            let bconn = Connection::open(backup.to_str().unwrap()).unwrap();
            assert_eq!(pk_count(&bconn, "computed_curves"), 1, "backup is the PRE-migration file: PK intact");
            let rows: i64 = bconn.query_row("SELECT COUNT(*) FROM computed_curves", [], |r| r.get(0)).unwrap();
            assert_eq!(rows, 2, "backup holds every pre-migration row (engine copy reads WAL state)");
        }
        let first_backup_hash = file_sha256(backup.to_str().unwrap());

        // Already-migrated open: no second backup.
        migrate_drop_computed_curves_pk(&conn, Some(&db_path_str)).unwrap();
        assert_eq!(count_backups("field.pre-"), 1, "a non-destructive open must not write a backup");
        drop(conn);

        // Collision: a NEW legacy file at the same path must not overwrite the old backup.
        std::fs::remove_file(&db_path).unwrap();
        let _ = std::fs::remove_file(dir.join("field.duckdb.wal"));
        make_legacy_file(&db_path_str);
        let conn = Connection::open(&db_path_str).unwrap();
        migrate_drop_computed_curves_pk(&conn, Some(&db_path_str)).unwrap();
        assert_eq!(count_backups("field.pre-"), 2, "second destructive run takes a timestamped name, never overwrites");
        assert_eq!(
            file_sha256(backup.to_str().unwrap()),
            first_backup_hash,
            "the original recovery copy must remain byte-identical"
        );
        drop(conn);

        // Deterministic failure injection at the exact copy boundary: the migration must
        // return before its first DROP/CREATE/INSERT and the original must remain openable.
        let failed_path = dir.join("failed.duckdb");
        let failed_path_str = failed_path.to_str().unwrap().to_string();
        make_legacy_file(&failed_path_str);
        let conn = Connection::open(&failed_path_str).unwrap();
        let err = migrate_drop_computed_curves_pk_with_backup(
            &conn,
            Some(&failed_path_str),
            |_conn, _path| {
                Err(DbError::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "injected backup refusal",
                )))
            },
        )
        .expect_err("a failed backup must abort the migration");
        assert!(err.to_string().contains("injected backup refusal"));
        assert_eq!(pk_count(&conn, "computed_curves"), 1, "no destructive statement may run after copy failure");
        drop(conn);
        let reopened = Connection::open(&failed_path_str).unwrap();
        assert_eq!(pk_count(&reopened, "computed_curves"), 1, "the un-migrated project must remain openable");
        let rows: i64 = reopened.query_row("SELECT COUNT(*) FROM computed_curves", [], |row| row.get(0)).unwrap();
        assert_eq!(rows, 2, "copy failure must preserve every source row");
        drop(reopened);

        // A current, additive-only open must leave no recovery copy at all.
        let additive_path = dir.join("additive.duckdb");
        let additive = crate::project::open_and_migrate(additive_path.to_str().unwrap()).unwrap();
        drop(additive);
        assert_eq!(count_backups("additive.pre-"), 0, "an additive open must not bury meaningful backups");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// CORRECTNESS — `22_database-model.md` SB-DBM-T43 adopts F-07: a backup is
    /// labelled by the source format it restores, independently of the target version.
    #[test]
    fn consecutive_destructive_upgrades_name_each_backup_for_the_source_format_it_restores() {
        let dir = std::env::temp_dir().join(format!("sandibumi-source-shelf-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("project.duckdb");
        let db_path_str = db_path.to_str().unwrap();
        let conn = Connection::open(db_path_str).unwrap();
        conn.execute_batch(
            "CREATE TABLE project_meta (key VARCHAR PRIMARY KEY, value VARCHAR);
             INSERT INTO project_meta VALUES ('format_version', '0'), ('written_by', 'SandiBumi source-0');
             CREATE TABLE migration_probe (value INTEGER);
             INSERT INTO migration_probe VALUES (0);",
        )
        .unwrap();

        let source_zero = backup_before_destructive_migration(&conn, db_path_str).unwrap();
        conn.execute_batch(
            "UPDATE migration_probe SET value = 1;
             UPDATE project_meta SET value = '1' WHERE key = 'format_version';",
        )
        .unwrap();
        let source_one = backup_before_destructive_migration(&conn, db_path_str).unwrap();
        conn.execute_batch(
            "UPDATE migration_probe SET value = 2;
             UPDATE project_meta SET value = '2' WHERE key = 'format_version';",
        )
        .unwrap();

        assert!(source_zero.ends_with("project.pre-0-backup.duckdb"), "first shelf label: {source_zero}");
        assert!(source_one.ends_with("project.pre-1-backup.duckdb"), "second shelf label: {source_one}");
        assert_ne!(source_zero, source_one, "source-labelled backups need no timestamp to distinguish upgrade steps");
        for (path, expected_version, expected_value) in [(&source_zero, "0", 0_i64), (&source_one, "1", 1_i64)] {
            let restored = Connection::open(path).unwrap();
            assert_eq!(read_meta(&restored, "format_version").as_deref(), Some(expected_version));
            assert_eq!(
                restored.query_row("SELECT value FROM migration_probe", [], |row| row.get::<_, i64>(0)).unwrap(),
                expected_value,
                "each shelf item must contain the state its label promises"
            );
        }
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

    /// Without the PK, uniqueness rests on `ancestry::write_versioned_rows_raw`: a re-run must
    /// overwrite (not duplicate) a well's curves, write several curves at once, keep other wells
    /// untouched, and leave `update_computed_sample` working.
    ///
    /// Driven through the `write_computed_curves_batch` FIXTURE, which builds a TEST_FIXTURE
    /// ancestry and then calls the production writer - so what is exercised below is the
    /// production discipline reached through a shorter door, not a second implementation of it.
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
    /// archive; restoring a version appends another version; ordinary deletion is refused;
    /// the catalog reports the new current provenance + stats.
    #[test]
    fn log_set_versioning_never_overwrites() {
        use crate::equations::{list_computed_catalog};
        use crate::ancestry::{LogSetSpec, create_log_set, delete_log_set, list_log_sets, restore_log_set, write_computed_curves_versioned};
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

        // Restore version 1 → version 3 becomes current with the old values; v1/v2 remain.
        let restored = restore_log_set(&conn, &set1).unwrap();
        assert_eq!(restored.rows_restored, 3);
        assert_eq!(restored.new_version, 3);
        assert_eq!(restored.restored_from.source_version, 1);
        assert!((current(1000.0) - 0.10).abs() < 1e-6, "restored to version 1");

        // Catalog: provenance of the current value points at the appended restore version.
        let cat = list_computed_catalog(&conn, &w).unwrap();
        let vsh = cat.iter().find(|e| e.curve_name == "VSH").unwrap();
        assert_eq!(vsh.set_name.as_deref(), Some("INTERP"));
        assert_eq!(vsh.version, Some(3));
        assert_eq!(vsh.n_samples, 3);
        assert!((vsh.min.unwrap() - 0.10).abs() < 1e-6 && (vsh.max.unwrap() - 0.30).abs() < 1e-6);

        // Ordinary deletion cannot mutate immutable history.
        let delete_error = delete_log_set(&conn, &set2).expect_err("archive deletion must refuse");
        assert!(delete_error.contains("append-only"), "{delete_error}");
        assert_eq!(list_log_sets(&conn, &w).unwrap().len(), 3);
        assert!((current(1000.0) - 0.10).abs() < 1e-6, "delete refusal leaves current values");
        let n_archive: i64 = conn
            .query_row("SELECT COUNT(*) FROM computed_curves_archive WHERE well_id = ?1", params![w], |r| r.get(0))
            .unwrap();
        assert_eq!(n_archive, 9, "versions 1, 2 and the appended restore all remain");
    }

    /// Batched multi-well versioned write (the field-scale write path): many wells land in ONE
    /// transaction via the grouped-DELETE + single-appender-per-table fast path. Locks the two
    /// things that path must not break — (1) grouping wells by curve-set and deleting the exact
    /// (wells × curves) cross product never touches a curve a well doesn't have, and (2) a re-run
    /// replaces current values while the archive keeps every generation, per well independently.
    #[test]
    fn batched_versioned_write_is_correct_across_wells_and_reruns() {
        use crate::ancestry::{LogSetSpec, WellWrite, create_log_sets_batch, list_log_sets, write_computed_curves_versioned_batch};
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
                    degradation_module: None,
                    degradations: None,
                },
                WellWrite {
                    well_id: w2.clone(),
                    depth: depth.clone(),
                    curves: vec![("VSH".into(), vsh2.to_vec())],
                    set_id: sets[&w2].clone(),
                    degradation_module: None,
                    degradations: None,
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
        use crate::equations::{fetch_curve_frame_from_set};
        use crate::ancestry::{LogSetSpec, create_log_set, write_computed_curves_versioned};
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
/// The depth range the well is actually LOGGED over — `MIN`/`MAX` across its standard curves,
/// falling back to its computed curves for a well that carries only derived logs.
///
/// The report cover used to read its interval off the composite PAGINATION, which honours the
/// render's depth window — so setting a print window re-dated the whole report, including the
/// tables the window never touched (`docs/review_triage.md` finding 18). A pay table covering every
/// zone in the well would sit under a cover announcing 5 m, and on a tables-only render there were
/// no log pages left to show the reader that the window was only ever a print setting.
///
/// Cheap on purpose: two aggregates over the leading column of a primary key. That is also what
/// lets a tables-only report state a real interval without rendering a composite it then discards.
pub fn logged_interval(conn: &Connection, well_id: &str) -> Option<(f32, f32)> {
    for table in ["standard_curves", "computed_curves"] {
        let got: Option<(Option<f32>, Option<f32>)> = conn
            .query_row(
                &format!("SELECT CAST(MIN(depth) AS FLOAT), CAST(MAX(depth) AS FLOAT) FROM {table} WHERE well_id = ?1"),
                params![well_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        // An empty table gives one row of NULLs, not zero rows — hence the inner Options.
        if let Some((Some(lo), Some(hi))) = got {
            return Some((lo, hi));
        }
    }
    None
}

pub fn update_well_field(conn: &Connection, well_id: &str, field: &str, value: Option<&str>) -> Result<(), String> {
    let n = match field {
        "well_name" | "field_name" | "utm_zone" => {
            let text = value.map(str::trim).filter(|s| !s.is_empty());
            conn.execute(&format!("UPDATE wells SET {field} = ?1 WHERE well_id = ?2"), params![text, well_id])
                .map_err(|e| e.to_string())?
        }
        "td" | "kb" => {
            let num: Option<f32> = match value {
                Some(v) if !v.trim().is_empty() => Some(v.trim().parse::<f32>().map_err(|e| e.to_string())?),
                _ => None,
            };
            conn.execute(&format!("UPDATE wells SET {field} = ?1 WHERE well_id = ?2"), params![num, well_id])
                .map_err(|e| e.to_string())?
        }
        "surface_x" | "surface_y" => {
            let num: Option<f64> = match value {
                Some(v) if !v.trim().is_empty() => Some(v.trim().parse::<f64>().map_err(|e| e.to_string())?),
                _ => None,
            };
            conn.execute(&format!("UPDATE wells SET {field} = ?1 WHERE well_id = ?2"), params![num, well_id])
                .map_err(|e| e.to_string())?
        }
        other => return Err(format!("field '{other}' is not editable")),
    };
    // The same 0-row check `update_standard_sample`, `update_computed_sample` and
    // `update_core_sample` carry. Without it an edit against a well that has since been deleted
    // in the Wells & Tops pane returns Ok: the cell shows the new value, the status bar reports
    // the edit, and an undo entry is pushed for a change that never happened
    // (`docs/review_triage.md` finding 20). The message names what to DO rather than what row was
    // missed — the identity here is a UUID the user never sees.
    if n == 0 {
        return Err("that well is no longer in the project — it may have been deleted; refresh the Wells grid".into());
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

/// What a core depth shift moved. Reported in two parts because the second one is easy to
/// forget and impossible to see afterwards — see [`shift_core_depths`].
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct CoreShiftCounts {
    /// Rows moved in `core_data`.
    pub plugs: usize,
    /// Rows moved in `aux_data` — the extras that rode in on those same plugs.
    pub extras: usize,
    /// SCAL Pc rows moved.
    pub scal: usize,
    /// Pictures moved.
    pub plates: usize,
    /// The ranges that put this operation back, in the depths that exist AFTER it. Empty for a
    /// whole-well shift, whose inverse is simply the negated delta.
    ///
    /// Computed here rather than by the caller because it needs the plug positions. Negating the
    /// deltas and shifting the caller's own ranges LOOKS right and is not: two barrels moved by
    /// different amounts can end up with overlapping ranges even though the barrels themselves
    /// never overlap, and the first matching range wins — so an undo would quietly move some
    /// plugs by their neighbour's correction.
    pub inverse: Vec<RunShift>,
}

/// Why a shift was applied, as the caller knows it at the moment of applying.
///
/// Passed to the shift functions rather than written afterwards by a separate call: a depth
/// registration that committed without its reason is precisely the state this exists to prevent,
/// and "the frontend will remember to log it" is not a guarantee anything can check later.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RegistrationNote {
    /// `"proposed"` (a correlation-backed registration), `"manual"` (a typed amount), `"undo"`.
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub log_curve: String,
    #[serde(default)]
    pub reference: String,
    #[serde(default)]
    pub pairing: String,
    /// Agreement **at the shift actually applied** — not the peak of the scan. The user is free
    /// to overrule the proposal, and recording the peak would then describe an alignment nobody
    /// chose.
    #[serde(default)]
    pub correlation: Option<f32>,
    #[serde(default)]
    pub n_pairs: Option<i64>,
    #[serde(default)]
    pub note: String,
}

impl Default for RegistrationNote {
    /// A shift with nothing said about it is a manual one. Recording is the default behaviour:
    /// there is no "do not record" value, because the whole point is that it cannot be skipped.
    fn default() -> Self {
        Self {
            kind: "manual".into(),
            log_curve: String::new(),
            reference: String::new(),
            pairing: String::new(),
            correlation: None,
            n_pairs: None,
            note: String::new(),
        }
    }
}

/// One line of a core's depth history, newest first.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RegistrationEntry {
    pub set_name: String,
    pub seq: i32,
    pub applied_at: Option<String>,
    pub kind: String,
    /// `None` for a whole-core shift — no range was declared.
    pub top: Option<f32>,
    pub base: Option<f32>,
    pub delta: f32,
    pub log_curve: String,
    pub reference: String,
    pub pairing: String,
    pub correlation: Option<f32>,
    pub n_pairs: Option<i64>,
    pub note: String,
}

/// One moved range together with the evidence for it. The agreement is per RANGE, not per apply:
/// each barrel is proposed against its own correlogram, so one number for the whole operation
/// would attribute one barrel's confidence to another's shift.
struct RegRow {
    top: Option<f32>,
    base: Option<f32>,
    delta: f32,
    correlation: Option<f32>,
    n_pairs: Option<i64>,
}

/// Appends one row per moved range. Takes a `&Connection` so it can be handed the open
/// transaction: the record and the move commit together or neither does.
fn write_registration(
    conn: &Connection,
    well_id: &str,
    ranges: &[RegRow],
    note: &RegistrationNote,
) -> DbResult<()> {
    if ranges.is_empty() {
        return Ok(());
    }
    // The set that is live NOW is the one being moved, and it is stored rather than resolved at
    // read time: switching the active delivery later must not rewrite what this one has been
    // through.
    let set_name: String =
        conn.query_row(&format!("SELECT {ACTIVE_CORE_SET}"), params![well_id], |r| r.get(0))?;
    let mut seq: i32 = conn.query_row(
        "SELECT COALESCE(MAX(seq), -1) + 1 FROM core_registrations WHERE well_id = ?1 AND set_name = ?2",
        params![well_id, &set_name],
        |r| r.get(0),
    )?;
    for r in ranges {
        let (top, base, delta) = (r.top, r.base, r.delta);
        conn.execute(
            "INSERT INTO core_registrations
               (well_id, set_name, seq, kind, top, base, delta, log_curve, reference, pairing,
                correlation, n_pairs, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                well_id,
                &set_name,
                seq,
                &note.kind,
                top,
                base,
                delta,
                &note.log_curve,
                &note.reference,
                &note.pairing,
                r.correlation.or(note.correlation),
                r.n_pairs.or(note.n_pairs),
                &note.note
            ],
        )?;
        seq += 1;
    }
    Ok(())
}

/// A core's depth history across every delivery it has, newest first.
pub fn list_core_registrations(conn: &Connection, well_id: &str) -> DbResult<Vec<RegistrationEntry>> {
    let mut stmt = conn.prepare(
        "SELECT set_name, seq, CAST(applied_at AS VARCHAR), kind, top, base, delta,
                COALESCE(log_curve, ''), COALESCE(reference, ''), COALESCE(pairing, ''),
                correlation, n_pairs, COALESCE(note, '')
         FROM core_registrations WHERE well_id = ?1
         ORDER BY applied_at DESC, seq DESC",
    )?;
    let rows = stmt.query_map(params![well_id], |r| {
        Ok(RegistrationEntry {
            set_name: r.get(0)?,
            seq: r.get(1)?,
            applied_at: r.get(2)?,
            kind: r.get(3)?,
            top: r.get(4)?,
            base: r.get(5)?,
            delta: r.get(6)?,
            log_curve: r.get(7)?,
            reference: r.get(8)?,
            pairing: r.get(9)?,
            correlation: r.get(10)?,
            n_pairs: r.get(11)?,
            note: r.get(12)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Applies a constant depth shift to the ACTIVE core set (core-to-log alignment).
/// Exactly reversible with -delta, so the frontend makes it undoable. Other deliveries of
/// the same well keep their own depths — a shift belongs to the set it was judged on.
///
/// **The plugs and their extras move TOGETHER, in one transaction.** A core table's extra
/// columns (core gamma, lithology description, Kv/Kh, sample ids …) are stored in `aux_data`
/// under the core delivery's OWN set name, at the plug depths they were measured at — see
/// `ingest::parse_core_table_mapped`. Moving `core_data` alone would leave every one of them
/// behind, silently decoupling a measurement from the plug it was made on: the porosity would
/// register against the log and the core gamma that JUSTIFIED the shift would not, so a second
/// pass would compute a fresh non-zero shift from the same core. Nothing downstream can detect
/// that, which is exactly why it is done here rather than left to each caller.
///
/// `datasets` names the point datasets that move along. It is NOT inferred from the set name
/// alone: a separately-imported XRD delivery is also called `RAW` by default, so matching on the
/// name would sweep up data that was never part of this core. [`core_extra_datasets`] returns the
/// ones that provably were, and the caller shows the list before applying — because whether an
/// XRD or CEC suite belongs to these plugs is a core-handling judgement, not something to guess.
pub fn shift_core_depths(
    conn: &mut Connection,
    well_id: &str,
    delta: f32,
    targets: &ShiftTargets,
    note: &RegistrationNote,
) -> DbResult<CoreShiftCounts> {
    let datasets = &targets.aux_datasets;
    let tx = conn.transaction()?;
    let plugs = tx.execute(
        &format!("UPDATE core_data SET depth = depth + ?2 WHERE well_id = ?1 AND set_name = {ACTIVE_CORE_SET}"),
        params![well_id, delta],
    )?;
    let mut scal = 0usize;
    if targets.scal {
        scal = tx.execute(
            &format!(
                "UPDATE scal_pc AS p SET depth = p.depth + ?2
                 WHERE p.well_id = ?1 AND p.depth IS NOT NULL AND p.set_name = {ACTIVE_SCAL_SET}"
            ),
            params![well_id, delta],
        )?;
    }
    let mut plates = 0usize;
    for dataset in &targets.image_datasets {
        plates += tx.execute(
            &format!(
                "UPDATE well_images AS i SET depth_top = i.depth_top + ?2, depth_base = i.depth_base + ?2
                 WHERE i.well_id = ?1 AND i.dataset = ?3 AND i.set_name = {ACTIVE_IMAGE_SET}"
            ),
            params![well_id, delta, dataset],
        )?;
    }
    let mut extras = 0usize;
    for dataset in datasets {
        // `depth_base + delta` is NULL-safe in SQL: a point sample stays a point sample.
        // `a` is the alias ACTIVE_AUX_SET correlates on, so only the LIVE delivery moves.
        extras += tx.execute(
            &format!(
                "UPDATE aux_data AS a SET depth_top = a.depth_top + ?2, depth_base = a.depth_base + ?2
                 WHERE a.well_id = ?1 AND a.dataset = ?3 AND a.set_name = {ACTIVE_AUX_SET}"
            ),
            params![well_id, delta, dataset],
        )?;
    }
    // Only if something actually moved: a well with no core has no depth history to write, and a
    // row saying "nothing was registered" is noise in the one place that must stay readable.
    if plugs > 0 {
        let row = RegRow { top: None, base: None, delta, correlation: None, n_pairs: None };
        write_registration(&tx, well_id, &[row], note)?;
    }
    tx.commit()?;
    Ok(CoreShiftCounts { plugs, extras, scal, plates, inverse: Vec::new() })
}

/// Records that a delivery sits on the CORE's depth scale, so a later core registration carries it
/// along. Called after an import the user declared as core-depth; harmless when the set does not
/// exist yet, which keeps the import paths free of ordering rules.
pub fn mark_aux_set_on_core(conn: &Connection, well_id: &str, dataset: &str, set_name: &str) -> DbResult<()> {
    conn.execute(
        "UPDATE aux_sets SET on_core_depths = 1 WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
        params![well_id, dataset, set_name],
    )?;
    Ok(())
}

pub fn mark_scal_set_on_core(conn: &Connection, well_id: &str, set_name: &str) -> DbResult<()> {
    conn.execute(
        "UPDATE scal_sets SET on_core_depths = 1 WHERE well_id = ?1 AND set_name = ?2",
        params![well_id, set_name],
    )?;
    Ok(())
}

pub fn mark_image_set_on_core(conn: &Connection, well_id: &str, dataset: &str, set_name: &str) -> DbResult<()> {
    conn.execute(
        "UPDATE image_sets SET on_core_depths = 1 WHERE well_id = ?1 AND dataset = ?2 AND set_name = ?3",
        params![well_id, dataset, set_name],
    )?;
    Ok(())
}

/// One thing that could ride along with a core depth shift.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ShiftCandidate {
    /// `"aux"`, `"scal"` or `"image"`.
    pub kind: String,
    /// Dataset name; empty for SCAL, which has only one live delivery per well.
    pub dataset: String,
    pub set_name: String,
    pub rows: i64,
    /// True when this delivery was imported as being on the core's depth scale. The dialog
    /// pre-ticks these and leaves the rest alone — moving a log-depth delivery would be wrong,
    /// and nothing downstream could tell.
    pub on_core_depths: bool,
}

/// Everything in a well that a core registration could carry with it: the point datasets, the live
/// SCAL delivery and each live image delivery.
///
/// Reported with `on_core_depths` rather than filtered by it, because the flag is only known for
/// deliveries imported since it existed — an older project would otherwise show nothing and look
/// as though there were nothing to move.
pub fn core_shift_candidates(conn: &Connection, well_id: &str) -> DbResult<Vec<ShiftCandidate>> {
    let mut out: Vec<ShiftCandidate> = Vec::new();

    let mut stmt = conn.prepare(&format!(
        "SELECT a.dataset, a.set_name, COUNT(*),
                COALESCE((SELECT s.on_core_depths FROM aux_sets s
                          WHERE s.well_id = ?1 AND s.dataset = a.dataset AND s.set_name = a.set_name), 0)
         FROM aux_data a
         WHERE a.well_id = ?1 AND a.set_name = {ACTIVE_AUX_SET}
         GROUP BY a.dataset, a.set_name ORDER BY a.dataset"
    ))?;
    for row in stmt.query_map(params![well_id], |r| {
        Ok(ShiftCandidate {
            kind: "aux".into(),
            dataset: r.get(0)?,
            set_name: r.get(1)?,
            rows: r.get(2)?,
            on_core_depths: r.get::<_, i64>(3)? != 0,
        })
    })? {
        out.push(row?);
    }

    let mut stmt = conn.prepare(&format!(
        "SELECT p.set_name, COUNT(*),
                COALESCE((SELECT s.on_core_depths FROM scal_sets s
                          WHERE s.well_id = ?1 AND s.set_name = p.set_name), 0)
         FROM scal_pc p
         WHERE p.well_id = ?1 AND p.set_name = {ACTIVE_SCAL_SET}
         GROUP BY p.set_name"
    ))?;
    for row in stmt.query_map(params![well_id], |r| {
        Ok(ShiftCandidate {
            kind: "scal".into(),
            dataset: String::new(),
            set_name: r.get(0)?,
            rows: r.get(1)?,
            on_core_depths: r.get::<_, i64>(2)? != 0,
        })
    })? {
        out.push(row?);
    }

    let mut stmt = conn.prepare(&format!(
        "SELECT i.dataset, i.set_name, COUNT(*),
                COALESCE((SELECT s.on_core_depths FROM image_sets s
                          WHERE s.well_id = ?1 AND s.dataset = i.dataset AND s.set_name = i.set_name), 0)
         FROM well_images i
         WHERE i.well_id = ?1 AND i.set_name = {ACTIVE_IMAGE_SET}
         GROUP BY i.dataset, i.set_name ORDER BY i.dataset"
    ))?;
    for row in stmt.query_map(params![well_id], |r| {
        Ok(ShiftCandidate {
            kind: "image".into(),
            dataset: r.get(0)?,
            set_name: r.get(1)?,
            rows: r.get(2)?,
            on_core_depths: r.get::<_, i64>(3)? != 0,
        })
    })? {
        out.push(row?);
    }
    Ok(out)
}

/// What a core shift should carry with it, as the caller chose it.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ShiftTargets {
    /// Point datasets (by dataset name; the ACTIVE delivery of each moves).
    #[serde(default)]
    pub aux_datasets: Vec<String>,
    /// Move the live SCAL delivery's plug depths.
    #[serde(default)]
    pub scal: bool,
    /// Image datasets (by dataset name; the ACTIVE delivery of each moves).
    #[serde(default)]
    pub image_datasets: Vec<String>,
}

impl ShiftTargets {
    pub fn aux(datasets: Vec<String>) -> Self {
        Self { aux_datasets: datasets, ..Default::default() }
    }
}

/// One barrel's (or one piece's) correction: everything currently between `top` and `base` moves
/// by `delta`. Ranges are in CURRENT depths — what you read off the log view — because that is
/// what the user is looking at when they draw the interval.
#[derive(Debug, Clone, Copy, Default, serde::Deserialize, serde::Serialize)]
pub struct RunShift {
    pub top: f32,
    pub base: f32,
    pub delta: f32,
    /// Agreement at THIS range's own shift, for the depth record. `#[serde(default)]` so an older
    /// payload — and the computed inverse, which was never proposed against anything — still
    /// deserializes. Absent means "not measured", never zero.
    #[serde(default)]
    pub correlation: Option<f32>,
    #[serde(default)]
    pub n_pairs: Option<i64>,
}

/// Builds the `CASE` that maps an old depth to a new one. Written as ONE set-wise UPDATE rather
/// than a row per plug, because the primary key contains `depth`: shifting 1000→1001 row by row
/// collides with the plug already at 1001, even when the finished result is perfectly valid.
/// `cond` is the column the range is tested against, `target` the one that moves. They differ for
/// an interval sample: it is placed by its TOP (so a barrel boundary cannot split one sample into
/// two different shifts) while its base moves by the same amount, keeping the logged thickness.
/// Every value is finite by the time this is called, so the formatted literals are always valid SQL.
fn run_shift_case_on(runs: &[RunShift], cond: &str, target: &str) -> String {
    let mut sql = String::from("CASE ");
    for r in runs {
        sql.push_str(&format!(
            "WHEN {cond} >= {:?} AND {cond} <= {:?} THEN {target} + {:?} ",
            r.top, r.base, r.delta
        ));
    }
    sql.push_str(&format!("ELSE {target} END"));
    sql
}

fn run_shift_case(runs: &[RunShift], column: &str) -> String {
    run_shift_case_on(runs, column, column)
}

/// Applies a per-barrel (or finer) set of corrections to the ACTIVE core delivery.
///
/// Core comes up a barrel at a time and each barrel carries its own tally error, so one number for
/// a whole well is right in the middle of the cored interval and wrong at both ends. Pieces can
/// also move INSIDE a barrel between the core face and the lab bench, which is why the ranges here
/// are free intervals rather than a fixed barrel length.
///
/// **Refuses anything that would reorder the core.** Two barrels shifted into each other's depths
/// would put deeper rock above shallower rock, and no downstream reader could tell. The check is
/// done in Rust on the finished depths, not approximated by a smoothness constraint, and names the
/// two plugs that would cross.
///
/// `depth_orig` is deliberately untouched: the record of where the delivery said the rock was is
/// what lets a later import follow ([`core_depth_pairs`]).
pub fn apply_core_run_shifts(
    conn: &mut Connection,
    well_id: &str,
    runs: &[RunShift],
    targets: &ShiftTargets,
    note: &RegistrationNote,
) -> Result<CoreShiftCounts, String> {
    let datasets = &targets.aux_datasets;
    if runs.is_empty() {
        return Ok(CoreShiftCounts::default());
    }
    for r in runs {
        if !(r.top.is_finite() && r.base.is_finite() && r.delta.is_finite()) {
            return Err("a shift range or amount is not a number".into());
        }
        if r.base < r.top {
            return Err(format!("range {} to {} is upside down", r.top, r.base));
        }
    }

    // Dry-run on the plug depths first. Nothing is written unless the result is still in order.
    let plugs: Vec<f32> = {
        let mut stmt = conn
            .prepare(&format!(
                "SELECT depth FROM core_data WHERE well_id = ?1 AND set_name = {ACTIVE_CORE_SET} ORDER BY depth"
            ))
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![well_id], |r| r.get::<_, f32>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
    };
    // Two barrels may not claim the same rock — across a real overlap the first match silently
    // wins, and which barrel a plug belonged to stops being answerable.
    //
    // Ranges that TOUCH at a single depth are allowed: `2000–2010` and `2010–2020` is the natural
    // way to write two adjacent barrels, and the shared depth goes to the first range listed. The
    // computed inverse below relies on this, since its boundaries sit exactly halfway between two
    // plugs — a point no plug can occupy.
    let mut sorted: Vec<&RunShift> = runs.iter().collect();
    sorted.sort_by(|a, b| a.top.total_cmp(&b.top));
    for pair in sorted.windows(2) {
        if pair[1].top < pair[0].base {
            return Err(format!(
                "the ranges {}–{} and {}–{} overlap; a plug can only belong to one barrel",
                pair[0].top, pair[0].base, pair[1].top, pair[1].base
            ));
        }
    }

    let run_for = |d: f32| -> Option<usize> { runs.iter().position(|r| d >= r.top && d <= r.base) };
    let delta_for = |d: f32| -> f32 { run_for(d).map(|i| runs[i].delta).unwrap_or(0.0) };
    let moved: Vec<f32> = plugs.iter().map(|&d| d + delta_for(d)).collect();
    for i in 1..moved.len() {
        if moved[i] <= moved[i - 1] {
            return Err(format!(
                "these shifts would put the plug from {} at {} and the one from {} at {} — that \
                 reorders the core, so nothing was changed",
                plugs[i - 1],
                moved[i - 1],
                plugs[i],
                moved[i]
            ));
        }
    }

    // Where each run's plugs ended up. Runs that moved nothing are dropped: they have no inverse
    // because they did nothing.
    let mut spans: Vec<(usize, f32, f32)> = Vec::new();
    for (idx, &old) in plugs.iter().enumerate() {
        if let Some(ri) = run_for(old) {
            let new = moved[idx];
            match spans.iter_mut().find(|(r, _, _)| *r == ri) {
                Some(s) => {
                    s.1 = s.1.min(new);
                    s.2 = s.2.max(new);
                }
                None => spans.push((ri, new, new)),
            }
        }
    }
    spans.sort_by(|a, b| a.1.total_cmp(&b.1));
    // Boundaries sit halfway between one run's deepest plug and the next run's shallowest, so
    // every plug a run moved is inside its own inverse range, no plug is inside two, and a point
    // sample that sits in the gap between barrels still rides back with the barrel above it.
    let inverse: Vec<RunShift> = spans
        .iter()
        .enumerate()
        .map(|(i, &(ri, lo, hi))| {
            let top = if i == 0 {
                lo - 0.5
            } else {
                0.5 * (spans[i - 1].2 + lo)
            };
            let base = if i + 1 == spans.len() {
                hi + 0.5
            } else {
                0.5 * (hi + spans[i + 1].1)
            };
            RunShift { top, base, delta: -runs[ri].delta, ..Default::default() }
        })
        .collect();

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let case = run_shift_case(runs, "depth");
    let plugs_moved = tx
        .execute(
            &format!(
                "UPDATE core_data SET depth = {case}
                 WHERE well_id = ?1 AND set_name = {ACTIVE_CORE_SET}"
            ),
            params![well_id],
        )
        .map_err(|e| e.to_string())?;

    let mut scal = 0usize;
    if targets.scal {
        // A Pc point with no depth is left alone: there is nothing to correct, and NULL + delta
        // would stay NULL anyway — the filter is there so the count reports what really moved.
        scal = tx
            .execute(
                &format!(
                    "UPDATE scal_pc AS p SET depth = {}
                     WHERE p.well_id = ?1 AND p.depth IS NOT NULL AND p.set_name = {ACTIVE_SCAL_SET}",
                    run_shift_case(runs, "p.depth")
                ),
                params![well_id],
            )
            .map_err(|e| e.to_string())?;
    }
    let mut plates = 0usize;
    for dataset in &targets.image_datasets {
        plates += tx
            .execute(
                &format!(
                    "UPDATE well_images AS i SET depth_top = {}, depth_base = {}
                     WHERE i.well_id = ?1 AND i.dataset = ?2 AND i.set_name = {ACTIVE_IMAGE_SET}",
                    run_shift_case(runs, "i.depth_top"),
                    run_shift_case_on(runs, "i.depth_top", "i.depth_base")
                ),
                params![well_id, dataset],
            )
            .map_err(|e| e.to_string())?;
    }

    let mut extras = 0usize;
    let top_case = run_shift_case(runs, "a.depth_top");
    let base_case = run_shift_case_on(runs, "a.depth_top", "a.depth_base");
    for dataset in datasets {
        extras += tx
            .execute(
                &format!(
                    "UPDATE aux_data AS a SET depth_top = {top_case}, depth_base = {base_case}
                     WHERE a.well_id = ?1 AND a.dataset = ?2 AND a.set_name = {ACTIVE_AUX_SET}"
                ),
                params![well_id, dataset],
            )
            .map_err(|e| e.to_string())?;
    }
    if plugs_moved > 0 {
        // One line per barrel, in the ranges the user drew — not one line for the whole apply.
        // Two barrels corrected by different amounts is exactly the case the record has to
        // preserve; collapsing it to an average would describe a shift that was never applied.
        let ranges: Vec<RegRow> = runs
            .iter()
            .map(|r| RegRow {
                top: Some(r.top),
                base: Some(r.base),
                delta: r.delta,
                correlation: r.correlation,
                n_pairs: r.n_pairs,
            })
            .collect();
        write_registration(&tx, well_id, &ranges, note).map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(CoreShiftCounts { plugs: plugs_moved, extras, scal, plates, inverse })
}

/// Every plug of the ACTIVE core delivery as `(where the lab said it was, where it is now)`,
/// ordered by the delivered depth. This IS the well's core depth record, kept in the core itself
/// rather than in a side table of shift history — it survives per-barrel shifts, single-plug
/// nudges and re-registrations without any bookkeeping, and it cannot drift out of sync with the
/// data it describes.
pub fn core_depth_pairs(conn: &Connection, well_id: &str) -> DbResult<Vec<(f32, f32)>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT COALESCE(depth_orig, depth), depth FROM core_data
         WHERE well_id = ?1 AND set_name = {ACTIVE_CORE_SET} ORDER BY 1"
    ))?;
    let rows = stmt.query_map(params![well_id], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Where a depth written by the lab now sits, given the core's own record.
///
/// Returns `(mapped depth, extrapolated)`. Between plugs the correction is interpolated, which is
/// the right behaviour when pieces moved inside a barrel: the offset genuinely varies along the
/// core, and a single number for the whole delivery would be wrong everywhere except one point.
///
/// **Outside the cored interval the nearest end's correction is held and `extrapolated` is true.**
/// There is no evidence out there — the core is what recorded the movement — so the caller must
/// show which samples were guessed rather than measured, instead of quietly placing them.
pub fn map_core_depth(pairs: &[(f32, f32)], delivered: f32) -> (f32, bool) {
    if pairs.is_empty() || !delivered.is_finite() {
        return (delivered, false);
    }
    let offset_at = |i: usize| pairs[i].1 - pairs[i].0;
    if delivered <= pairs[0].0 {
        let ex = delivered < pairs[0].0;
        return (delivered + offset_at(0), ex);
    }
    let last = pairs.len() - 1;
    if delivered >= pairs[last].0 {
        let ex = delivered > pairs[last].0;
        return (delivered + offset_at(last), ex);
    }
    let i = pairs.partition_point(|p| p.0 < delivered);
    let (d0, d1) = (pairs[i - 1].0, pairs[i].0);
    let (o0, o1) = (offset_at(i - 1), offset_at(i));
    let t = if d1 > d0 { (delivered - d0) / (d1 - d0) } else { 0.0 };
    (delivered + o0 + (o1 - o0) * t, false)
}

/// The point datasets that were delivered as part of the well's ACTIVE core table — those whose
/// own live delivery carries the core set's name, which is how `ingest::import_core_table` writes
/// the extra columns (core gamma, lithology, Kv/Kh …) so that switching a well's core switches
/// its extras with it.
///
/// This is the honest default for [`shift_core_depths`]: certain where it can be, and everything
/// else left for the user to add deliberately.
pub fn core_extra_datasets(conn: &Connection, well_id: &str) -> DbResult<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT a.dataset, COUNT(*) FROM aux_data a
         WHERE a.well_id = ?1 AND a.set_name = {ACTIVE_AUX_SET}
           AND a.set_name = {ACTIVE_CORE_SET}
         GROUP BY a.dataset ORDER BY a.dataset"
    ))?;
    let rows = stmt.query_map(params![well_id], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
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
#[cfg(test)]
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
    let existing_datum: Option<Option<String>> = conn
        .query_row(
            "SELECT depth_datum FROM tops WHERE well_id = ?1 AND top_name = ?2",
            params![well_id, top_name],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(Some(existing_datum)) = existing_datum {
        if existing_datum != crate::schema_vocab::DepthDatum::Md.as_str() {
            return Err(DbError::Invalid(format!(
                "{existing_datum}-referenced top '{top_name}' cannot be deleted by the MD tops editor; remove it through a source-reference-aware workflow"
            )));
        }
    }
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
    pub set_version: i32,
    pub final_flag: bool,
    /// Monotonic metadata revision used for the declared MRU resolution stage. NULL means a
    /// pre-SB-DBM-006 row whose historical order cannot be recovered.
    pub modified_seq: Option<i64>,
    /// Every stored row, including missing/non-finite values.
    pub n_samples: i64,
    /// Finite values eligible for numeric statistics.
    pub n_valid: i64,
    /// Stored rows excluded from numeric statistics because their value is non-finite.
    pub n_missing: i64,
    /// SB-DBM-017 / DEC-025: the DECLARED neutron matrix basis; None is the honest absence.
    pub neutron_basis: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>,
    /// True when the user has promoted this curve to win its (well, set, mnemonic) group in
    /// curve resolution (the DLIS/LAS same-mnemonic shadow tiebreak).
    pub pinned: bool,
}

/// Metadata-only curve identity for navigation surfaces. Deliberately has no sample count or
/// statistics: Wells/Set expansion must never scan `curve_samples` merely to draw a tree.
#[derive(Debug, Clone, Serialize)]
pub struct GenericCurveInventoryEntry {
    pub curve_id: String,
    pub mnemonic: String,
    pub unit: Option<String>,
    pub family: Option<String>,
    pub set_name: String,
    pub source: Option<String>,
    pub run_no: Option<i32>,
    pub set_version: i32,
    pub final_flag: bool,
    pub modified_seq: Option<i64>,
    pub pinned: bool,
}

pub fn list_generic_curve_inventory(
    conn: &Connection,
    well_id: &str,
) -> DbResult<Vec<GenericCurveInventoryEntry>> {
    let mut stmt = conn.prepare(
        "SELECT curve_id, mnemonic, unit, family, set_name, source, run_no, set_version,
                COALESCE(final_flag, 0), modified_seq,
                COALESCE(pinned, 0)
         FROM curve_meta
         WHERE well_id = ?1
         ORDER BY set_name, family, mnemonic, run_no NULLS FIRST, curve_id",
    )?;
    let rows = stmt.query_map(params![well_id], |row| {
        Ok(GenericCurveInventoryEntry {
            curve_id: row.get(0)?,
            mnemonic: row.get(1)?,
            unit: row.get(2)?,
            family: row.get(3)?,
            set_name: row.get(4)?,
            source: row.get(5)?,
            run_no: row.get(6)?,
            set_version: row.get(7)?,
            final_flag: row.get::<_, i32>(8)? != 0,
            modified_seq: row.get(9)?,
            pinned: row.get::<_, i32>(10)? != 0,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Lists every curve in the generic store for one well, across all sets — the data
/// source for the Curve Catalog's family/unit/set columns (Phase 6). Named distinctly
/// from `equations::list_curve_catalog` (the existing standard+computed catalog), which
/// remains the frontend's data source until the Phase 6 curve-store migration is wired
/// through the rest of the app (workflow modules, log views, equations).
/// SB-DBM-017 / DEC-025: records the DECLARED neutron matrix basis on one curve, with the
/// declaration's source. Refuses an empty basis or source - an empty declaration is not a
/// declaration - and never fills a default: ABSENT stays absent, and inference from the
/// contractor, the tool or the salinity is exactly what this field exists to replace.
pub fn set_curve_neutron_basis(
    conn: &Connection,
    curve_id: &str,
    basis: &str,
    source: &str,
) -> DbResult<()> {
    let basis = basis.trim();
    let source = source.trim();
    if basis.is_empty() || source.is_empty() {
        return Err(DbError::Invalid(
            "a neutron matrix basis is DECLARED: both the basis and its source are required, \
             and an absent basis stays absent rather than being written blank"
                .into(),
        ));
    }
    // The vocabulary is CLOSED and the stored spelling canonical. Every consumer compares
    // the stored string (required_neutron_basis pins "LIMESTONE"; the workflow boundary and
    // nphimat both match tokens), so a spelling outside the list would be stored as a basis
    // no module check can ever satisfy: a typo becoming permanent metadata.
    let canonical = match basis.to_uppercase().as_str() {
        "LS" | "LIMESTONE" => "LIMESTONE",
        "SS" | "SANDSTONE" => "SANDSTONE",
        "DOL" | "DOLOMITE" => "DOLOMITE",
        other => {
            return Err(DbError::Invalid(format!(
                "'{other}' is not a neutron matrix basis this declaration can store: the \
                 recognized tokens are LIMESTONE/LS, SANDSTONE/SS and DOLOMITE/DOL, and a \
                 spelling outside them would be a basis no module check could ever match; \
                 correct the token and declare again"
            )));
        }
    };
    let n = conn.execute(
        "UPDATE curve_meta SET neutron_basis = ?2, neutron_basis_source = ?3 WHERE curve_id = ?1",
        params![curve_id, canonical, source],
    )?;
    if n == 0 {
        return Err(DbError::Invalid(format!(
            "no curve '{curve_id}' to declare a neutron basis on"
        )));
    }
    Ok(())
}

pub fn list_generic_curve_catalog(conn: &Connection, well_id: &str) -> DbResult<Vec<GenericCurveCatalogEntry>> {
    let mut stmt = conn.prepare(
        "SELECT m.curve_id, m.mnemonic, m.unit, m.family, m.set_name, m.source, m.run_no,
                m.set_version, COALESCE(m.final_flag, 0), m.modified_seq,
                COUNT(s.depth),
                COUNT(*) FILTER (WHERE s.depth IS NOT NULL AND isfinite(CAST(s.value AS DOUBLE))),
                COUNT(s.depth) - COUNT(*) FILTER (WHERE s.depth IS NOT NULL AND isfinite(CAST(s.value AS DOUBLE))),
                MIN(CAST(s.value AS DOUBLE)) FILTER (WHERE isfinite(CAST(s.value AS DOUBLE))),
                MAX(CAST(s.value AS DOUBLE)) FILTER (WHERE isfinite(CAST(s.value AS DOUBLE))),
                AVG(CAST(s.value AS DOUBLE)) FILTER (WHERE isfinite(CAST(s.value AS DOUBLE))),
                COALESCE(m.pinned, 0), m.neutron_basis
         FROM curve_meta m
         LEFT JOIN curve_samples s ON s.curve_id = m.curve_id
         WHERE m.well_id = ?1
         GROUP BY m.curve_id, m.mnemonic, m.unit, m.family, m.set_name, m.source, m.run_no,
                  m.set_version, m.final_flag, m.modified_seq, m.pinned, m.neutron_basis
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
            set_version: row.get(7)?,
            final_flag: row.get::<_, i32>(8)? != 0,
            modified_seq: row.get(9)?,
            n_samples: row.get(10)?,
            n_valid: row.get(11)?,
            n_missing: row.get(12)?,
            min: row.get(13)?,
            max: row.get(14)?,
            mean: row.get(15)?,
            pinned: row.get::<_, i32>(16)? != 0,
            neutron_basis: row.get(17)?,
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
            "UPDATE curve_meta
             SET pinned = 0, set_version = set_version + 1
             WHERE well_id = ?1 AND set_name = ?2 AND upper(mnemonic) = upper(?3)
               AND curve_id <> ?4 AND COALESCE(pinned, 0) <> 0",
            params![well, set, mnem, curve_id],
        )?;
        conn.execute(
            "UPDATE curve_meta
             SET pinned = 1, set_version = set_version + 1
             WHERE curve_id = ?1 AND COALESCE(pinned, 0) <> 1",
            params![curve_id],
        )?;
        Ok(())
    })
}

/// Marks one generic curve as the Final member of its resolved quantity family. The previous
/// Final curve id is returned so the frontend can place the metadata edit on the undo stack.
/// At most one curve per well/family is Final; clearing a flag affects only the named curve.
pub fn set_generic_curve_final(
    conn: &Connection,
    curve_id: &str,
    is_final: bool,
) -> DbResult<Option<String>> {
    with_txn(conn, |conn| {
        let (well_id, family, mnemonic, current_final): (String, Option<String>, String, bool) = conn.query_row(
            "SELECT well_id, family, mnemonic, COALESCE(final_flag, 0) <> 0
             FROM curve_meta WHERE curve_id = ?1",
            params![curve_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
        let key = family
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(mnemonic.as_str())
            .to_uppercase();
        let previous: Option<String> = if !is_final {
            current_final.then(|| curve_id.to_string())
        } else {
            match conn.query_row(
                "SELECT curve_id FROM curve_meta
                 WHERE well_id = ?1
                   AND COALESCE(NULLIF(upper(trim(family)), ''), upper(mnemonic)) = ?2
                   AND COALESCE(final_flag, 0) = 1
                 ORDER BY curve_id LIMIT 1",
                params![well_id, key],
                |row| row.get(0),
            ) {
                Ok(previous) => Some(previous),
                Err(duckdb::Error::QueryReturnedNoRows) => None,
                Err(error) => return Err(error.into()),
            }
        };
        if is_final {
            conn.execute(
                "UPDATE curve_meta
                 SET final_flag = 0, set_version = set_version + 1
                 WHERE well_id = ?1
                   AND COALESCE(NULLIF(upper(trim(family)), ''), upper(mnemonic)) = ?2
                   AND COALESCE(final_flag, 0) = 1 AND curve_id <> ?3",
                params![well_id, key, curve_id],
            )?;
        }
        conn.execute(
            "UPDATE curve_meta
             SET final_flag = ?2, set_version = set_version + 1
             WHERE curve_id = ?1 AND COALESCE(final_flag, 0) <> ?2",
            params![curve_id, i32::from(is_final)],
        )?;
        Ok(previous)
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
            "UPDATE curve_meta
             SET mnemonic = ?2, unit = ?3, family = ?4,
                 set_version = set_version + 1,
                 modified_seq = nextval('curve_meta_modified_seq')
             WHERE curve_id = ?1",
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
            "UPDATE curve_meta
             SET unit = ?1, family = ?2, source = ?3,
                 set_version = set_version + 1,
                 modified_seq = nextval('curve_meta_modified_seq')
             WHERE curve_id = ?4",
            params![unit, family, source, id],
        )?;
        return Ok(id);
    }
    let curve_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO curve_meta
            (curve_id, well_id, set_name, mnemonic, unit, family, source, run_no,
             set_version, final_flag, modified_seq)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1,
                 0,
                 nextval('curve_meta_modified_seq'))",
        params![curve_id, well_id, set_name, mnemonic, unit, family, source, run_no],
    )?;
    Ok(curve_id)
}

/// Bulk-replaces the samples for one curve (delete-then-append, mirroring
/// `insert_core_data`'s replace-on-reimport semantics).
/// SB-DBM-030: Geolog's null-sentinel family. `cgg.h` defines `MISS_FLOAT = -1.0e30` and the
/// manuals also show `-1.0D38`; both are "large negative", and neither is guaranteed to arrive
/// as an exact bit pattern after a unit conversion or a float round-trip. The screen is
/// therefore a strict inequality against a bound COMPUTED one decade inside the cited
/// constant - an equality against one sentinel misses the other entirely, and a hand-typed
/// decimal could land on the wrong side of a boundary sample.
// SB-DBM-025: the value is registered in `param_sources::CROSS_MODULE_CONSTANTS` with its
// citation; this re-export keeps the screen and the registry one object.
pub const GEOLOG_MISS_FLOAT: f32 = crate::param_sources::GEOLOG_MISS_FLOAT;

/// True when a value is in the undeclared large-negative null family: strictly below one tenth
/// of `GEOLOG_MISS_FLOAT`. A value exactly ON the bound is DATA. NaN is not screened here - it
/// is already the missing convention and binds SQL NULL at the writer.
pub fn is_large_negative_null(value: f32) -> bool {
    value < GEOLOG_MISS_FLOAT / 10.0
}

/// Returns how many samples the large-negative null screen bound to SQL NULL for this curve -
/// the write path's flag channel. A caller importing external data must surface a non-zero
/// count; screening is never silent.
pub fn insert_curve_samples(conn: &Connection, curve_id: &str, depths: &[f32], values: &[f32]) -> DbResult<usize> {
    let screened = insert_curve_samples_batch(conn, depths, &[(curve_id, values)])?;
    Ok(screened.into_iter().map(|(_, count)| count).sum())
}

/// SB-DIO-057 / DEC-076: the interpreter's recorded word on exact zeros in one log-scale
/// curve. `keep` commits them as VALUES; `!keep` converts them to MISSING at commit. Stored
/// as a document so the decision survives with the project and T85's "the decision is
/// recorded" is a row, not a log line.
pub fn confirm_log_scale_zeros(
    conn: &Connection,
    well_id: &str,
    mnemonic: &str,
    keep: bool,
) -> DbResult<()> {
    let name = format!("{well_id}:{}", mnemonic.trim().to_uppercase());
    let json = serde_json::json!({
        "decision": if keep { "keep" } else { "convert" },
        "requirement": "SB-DIO-057",
    })
    .to_string();
    save_document(conn, "zero-decision", &name, &json)
}

fn log_scale_zero_decision(conn: &Connection, well_id: &str, mnemonic: &str) -> Option<String> {
    let name = format!("{well_id}:{}", mnemonic.trim().to_uppercase());
    let json: Option<String> = conn
        .query_row(
            "SELECT json FROM documents WHERE doc_type = 'zero-decision' AND name = ?1",
            params![name],
            |row| row.get(0),
        )
        .ok();
    json.and_then(|text| {
        serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|value| value.get("decision").and_then(|d| d.as_str()).map(str::to_string))
    })
}

/// SB-DIO-057 (DEC-076 signed registry): counts exact zeros on a log-scale-family curve and
/// returns the interpreter's recorded decision, refusing BY NAME when zeros are present and
/// no decision exists — surfaced before commit, never rewritten automatically. A curve with
/// no meta row, no family, or a non-logarithmic family passes untouched; so does a
/// log-family curve carrying no exact zero.
fn screen_log_scale_zeros(
    conn: &Connection,
    curve_id: &str,
    values: &[f32],
) -> DbResult<Option<String>> {
    let zero_count = values.iter().filter(|value| **value == 0.0).count();
    if zero_count == 0 {
        return Ok(None);
    }
    let meta: Option<(String, String, Option<String>)> = conn
        .query_row(
            "SELECT well_id, mnemonic, family FROM curve_meta WHERE curve_id = ?1",
            params![curve_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok();
    let Some((well_id, mnemonic, family)) = meta else { return Ok(None) };
    let Some(family) = family else { return Ok(None) };
    if !crate::curves::LOG_SCALE_FAMILIES.contains(&family.as_str()) {
        return Ok(None);
    }
    match log_scale_zero_decision(conn, &well_id, &mnemonic) {
        Some(decision) => Ok(Some(decision)),
        None => Err(DbError::Invalid(format!(
            "curve {mnemonic} (family {family}) carries {zero_count} exact zero(s): a zero on a \
             log-scale curve cannot be committed as a reading without your word — it is usually an \
             exporter's encoding of 'no reading' (SB-DIO-057). Confirm keep-as-values or \
             convert-to-missing for this curve (db::confirm_log_scale_zeros), then re-import; \
             nothing was written."
        ))),
    }
}

// ---------------------------------------------------------------------------
// SB-DIO-007 (signed DRAFT_DIO007 under DEC-076): the source-cell-state mask.
// "Field empty" and "field = null sentinel" are different facts (PRD D-33) -
// the mask records which, per sample, and the distinction survives to the
// delimited deliverable. Auxiliary custody only: never a gate on measurements.
// ---------------------------------------------------------------------------

/// Version prefix of every written mask blob. A reader seeing an unknown version
/// refuses to interpret STATES; the curve's values are unaffected.
pub const CELL_STATE_MASK_VERSION: u8 = 1;
/// The cell held a measurement.
pub const CELL_STATE_MEASURED: u8 = 0;
/// Nothing between the delimiters (or the row ended before the column).
pub const CELL_STATE_EMPTY: u8 = 1;
/// The cell held the file's null token (the SB-DBM-030 large-negative family).
pub const CELL_STATE_NULLED: u8 = 2;

/// Encodes per-sample states (ascending depth order) behind the version byte.
pub fn encode_state_mask(states: &[u8]) -> Vec<u8> {
    let mut blob = Vec::with_capacity(states.len() + 1);
    blob.push(CELL_STATE_MASK_VERSION);
    blob.extend_from_slice(states);
    blob
}

/// Decodes a stored mask against the curve's sample count. Refuses an unknown
/// version by name, and refuses a length mismatch WHOLE - a mask that misaligns
/// with its array attributes states to the wrong depths, which is worse than no
/// mask at all (draft rule 6).
pub fn decode_state_mask(blob: &[u8], expected_samples: usize) -> Result<Vec<u8>, String> {
    let Some((&version, states)) = blob.split_first() else {
        return Err("state mask is empty: refused whole rather than misread (SB-DIO-007)".into());
    };
    if version != CELL_STATE_MASK_VERSION {
        return Err(format!(
            "state mask carries version {version}, and this build reads version \
             {CELL_STATE_MASK_VERSION}: cell states are refused rather than misread; the \
             curve's values are unaffected (SB-DIO-007)"
        ));
    }
    if states.len() != expected_samples {
        return Err(format!(
            "state mask carries {} state(s) for {expected_samples} sample(s): a misaligned mask \
             attributes states to the wrong depths, so it is refused whole (SB-DIO-007)",
            states.len()
        ));
    }
    Ok(states.to_vec())
}

/// Stores (or clears) a curve's mask. Refuses by name when the curve does not
/// exist - a mask exists only beside the samples it describes.
pub fn set_curve_state_mask(conn: &Connection, curve_id: &str, mask: Option<&[u8]>) -> DbResult<()> {
    let updated = conn.execute(
        "UPDATE curve_meta SET state_mask = ?2 WHERE curve_id = ?1",
        params![curve_id, mask],
    )?;
    if updated == 0 {
        return Err(DbError::Invalid(format!(
            "no curve {curve_id}: a state mask exists only beside the samples it describes (SB-DIO-007)"
        )));
    }
    Ok(())
}

/// The raw stored blob; `None` = pre-contract import (unknown states).
pub fn get_curve_state_mask(conn: &Connection, curve_id: &str) -> DbResult<Option<Vec<u8>>> {
    let blob: Option<Vec<u8>> = conn.query_row(
        "SELECT state_mask FROM curve_meta WHERE curve_id = ?1",
        params![curve_id],
        |row| row.get(0),
    )?;
    Ok(blob)
}

/// Atomically replaces every curve in one imported delivery using one transaction and one
/// DuckDB appender. The complete batch is validated before any DELETE occurs. Returns the
/// SB-DBM-030 flag channel: per curve id, how many samples the large-negative null screen
/// bound to SQL NULL (only non-zero entries).
pub fn insert_curve_samples_batch(conn: &Connection, depths: &[f32], curves: &[(&str, &[f32])]) -> DbResult<Vec<(String, usize)>> {
    with_txn(conn, |conn| insert_curve_samples_batch_in_transaction(conn, depths, curves))
}

/// Transaction-free inner form for callers already committing metadata and samples as one
/// unit. DuckDB does not support nested transactions; callers must wrap this themselves.
pub(crate) fn insert_curve_samples_batch_in_transaction(
    conn: &Connection,
    depths: &[f32],
    curves: &[(&str, &[f32])],
) -> DbResult<Vec<(String, usize)>> {
    for (curve_id, values) in curves {
        if depths.len() != values.len() {
            return Err(DbError::LengthMismatch(format!(
                "curve {curve_id}: depths ({}) and values ({}) must match",
                depths.len(),
                values.len()
            )));
        }
    }
    // SB-DIO-057: the zero gate runs BEFORE the DELETEs — an undecided refusal must leave
    // the store exactly as it was, not empty. `convert` decisions are collected here and
    // applied where the staging arrays are built.
    let mut convert_zero_curves: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (curve_id, values) in curves {
        if let Some(decision) = screen_log_scale_zeros(conn, curve_id, values)? {
            if decision == "convert" {
                convert_zero_curves.insert(*curve_id);
            }
        }
    }
    for (curve_id, _) in curves {
        conn.execute("DELETE FROM curve_samples WHERE curve_id = ?1", params![curve_id])?;
    }
    // DuckDB's ordinary `append_row` API still crosses the Rust/C boundary once per
    // sample. A field LAS can contain millions of long-table rows, so stage each curve
    // as Arrow vectors and let DuckDB consume native-sized data chunks instead. The
    // staging column is VARCHAR because Arrow has no UUID logical type; the final INSERT
    // performs DuckDB's checked UUID cast inside the caller's transaction.
    let staging_table = format!("curve_samples_import_{}", Uuid::new_v4().simple());
    conn.execute_batch(&format!(
        "CREATE TEMP TABLE {staging_table} (
             curve_id VARCHAR NOT NULL,
             depth FLOAT NOT NULL,
             value FLOAT
         )"
    ))?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("curve_id", DataType::Utf8, false),
        Field::new("depth", DataType::Float32, false),
        Field::new("value", DataType::Float32, true),
    ]));
    let depth_array: ArrayRef = Arc::new(Float32Array::from_iter_values(depths.iter().copied()));
    let mut appender: Appender = conn.appender(&staging_table)?;
    let mut screened: Vec<(String, usize)> = Vec::new();
    for (curve_id, values) in curves {
        let curve_id_array: ArrayRef = Arc::new(StringArray::from_iter_values(
            std::iter::repeat(*curve_id).take(depths.len()),
        ));
        // SB-DBM-030: the store's null discipline. A NaN is the missing convention and binds
        // SQL NULL, so at the store absence is never representable as a number; a value in the
        // large-negative family is an undeclared vendor null sentinel, screened to NULL and
        // COUNTED - the count is the flag channel every importer surfaces. Never silent.
        let mut screened_here = 0usize;
        // SB-DIO-057: a recorded `convert` decision turns this curve's exact zeros into
        // MISSING at commit — explicit, per-curve, never automatic (the gate above refused
        // if no decision existed).
        let convert_zeros = convert_zero_curves.contains(*curve_id);
        let value_array: ArrayRef = Arc::new(
            values
                .iter()
                .map(|&value| {
                    if value.is_nan() {
                        None
                    } else if is_large_negative_null(value) {
                        screened_here += 1;
                        None
                    } else if convert_zeros && value == 0.0 {
                        None
                    } else {
                        Some(value)
                    }
                })
                .collect::<Float32Array>(),
        );
        let batch = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![curve_id_array, Arc::clone(&depth_array), value_array],
        )
        .map_err(|err| DbError::ColumnarBatch(err.to_string()))?;
        appender.append_record_batch(batch)?;
        if screened_here > 0 {
            screened.push(((*curve_id).to_string(), screened_here));
        }
    }
    appender.flush()?;
    drop(appender);
    conn.execute_batch(&format!(
        "INSERT INTO curve_samples (curve_id, depth, value)
         SELECT CAST(curve_id AS UUID), depth, value FROM {staging_table};
         DROP TABLE {staging_table};"
    ))?;
    Ok(screened)
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

// ---------------------------------------------------------------------------
// SB-DBM-011 - the structured audit writer (DEC-020 operator, DEC-022 UTC,
// DEC-023 zone-set seam). ONE backend-owned atomic writer: entry + details in
// one transaction, with Geolog's uninterrupted-collapse rule applied here so
// no caller can produce forty entries for one crossplot drag.
// ---------------------------------------------------------------------------

pub const AUDIT_LOCATIONS: [&str; 7] =
    ["PARAMETER", "COMMENT", "SET", "CONSTANT", "INTERVAL", "LOG", "ATTRIBUTE"];
pub const AUDIT_MODES: [&str; 7] =
    ["INPUT", "OUTPUT", "DELETE", "RENAME", "SAVE", "SAVE_AS", "SAVE_CANCEL"];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AuditDetail {
    pub location: String,
    pub mode: String,
    pub unit: Option<String>,
    pub name: String,
    pub value: Option<String>,
}

/// DEC-023's seam: the current zone-set identity and version for one well. The digest is a
/// stable SHA-256 over the zones in depth order (name, top, bottom); a version row is
/// appended only when the digest changes, so the version is monotone and an audit entry can
/// name exactly which zone configuration it saw. SB-DBM-008, when scheduled, inherits this
/// rather than designing freely - the accepted cost of the narrow route.
pub fn current_zone_set(conn: &Connection, well_id: &str) -> DbResult<(i64, String)> {
    use sha2::{Digest, Sha256};
    let zones = list_zones(conn, well_id)?;
    let mut hasher = Sha256::new();
    for zone in &zones {
        hasher.update(zone.zone_name.as_bytes());
        hasher.update([0u8]);
        hasher.update(zone.top_depth.to_le_bytes());
        hasher.update(zone.bottom_depth.to_le_bytes());
        hasher.update([1u8]);
    }
    let digest = hasher.finalize()[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let latest: Option<(i64, String)> = conn
        .query_row(
            "SELECT version, digest FROM zone_set_versions WHERE well_id = ?1
             ORDER BY version DESC LIMIT 1",
            params![well_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();
    match latest {
        Some((version, stored)) if stored == digest => Ok((version, digest)),
        other => {
            let version = other.map(|(version, _)| version + 1).unwrap_or(1);
            conn.execute(
                "INSERT INTO zone_set_versions (well_id, version, digest) VALUES (?1, ?2, ?3)",
                params![well_id, version, digest],
            )?;
            Ok((version, digest))
        }
    }
}

/// The one audit writer. Refusals are BY NAME with the permitted vocabulary stated: an
/// empty operator (DEC-020 forbids inferring one), an empty detail list, a location or
/// mode outside the controlled sets, and the dotted-name rule both ways - a dotted name
/// MUST denote an attribute change on the named object, so ATTRIBUTE requires a dot and a
/// dot requires ATTRIBUTE. Uninterrupted repeats of the same type (identical well, actor,
/// view, source and detail signature) COLLAPSE into the latest entry: its values and
/// timestamp advance and repeat_count counts the gestures. Any different action between
/// two repeats breaks the chain by construction, because "latest entry" is decided by
/// entry_seq order - never by an invented time window.
pub fn record_audit_entry(
    conn: &Connection,
    well_id: Option<&str>,
    operator: &str,
    operator_kind: &str,
    view: &str,
    source: &str,
    comment: Option<&str>,
    zone_set: Option<(i64, &str)>,
    details: &[AuditDetail],
) -> DbResult<String> {
    if operator.trim().is_empty() {
        return Err(DbError::Invalid(
            "audit refused: the session operator is explicit and never inferred (DEC-020) - enter an operator identity".into(),
        ));
    }
    if !matches!(operator_kind, "HUMAN" | "AUTOMATED") {
        return Err(DbError::Invalid(format!(
            "audit refused: operator kind '{operator_kind}' is not in the controlled set HUMAN/AUTOMATED (DEC-020)"
        )));
    }
    if details.is_empty() {
        return Err(DbError::Invalid("audit refused: an entry needs at least one detail row".into()));
    }
    for detail in details {
        if !AUDIT_LOCATIONS.contains(&detail.location.as_str()) {
            return Err(DbError::Invalid(format!(
                "audit refused: location '{}' is not in the controlled vocabulary {}",
                detail.location,
                AUDIT_LOCATIONS.join("/")
            )));
        }
        if !AUDIT_MODES.contains(&detail.mode.as_str()) {
            return Err(DbError::Invalid(format!(
                "audit refused: mode '{}' is not in the controlled vocabulary {}",
                detail.mode,
                AUDIT_MODES.join("/")
            )));
        }
        let dotted = detail.name.contains('.');
        let attribute = detail.location == "ATTRIBUTE";
        if attribute != dotted {
            return Err(DbError::Invalid(format!(
                "audit refused: a dotted name denotes an attribute change on the named object and nothing else - '{}' with location {} breaks that both-ways rule",
                detail.name, detail.location
            )));
        }
    }

    // The collapse check: the LATEST entry, by sequence - uninterruptedness is order.
    let latest: Option<(String, String, String, String, String, String)> = conn
        .query_row(
            "SELECT entry_id, COALESCE(well_id::VARCHAR, ''), operator, operator_kind, view, source
             FROM audit_entry ORDER BY entry_seq DESC LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .ok();
    if let Some((entry_id, latest_well, latest_op, latest_kind, latest_view, latest_source)) =
        latest
    {
        let same_head = latest_well == well_id.unwrap_or("")
            && latest_op == operator
            && latest_kind == operator_kind
            && latest_view == view
            && latest_source == source;
        if same_head {
            let mut stmt = conn.prepare(
                "SELECT location, mode, name, COALESCE(unit, '') FROM audit_detail
                 WHERE entry_id = ?1 ORDER BY seq",
            )?;
            let signature: Vec<(String, String, String, String)> = stmt
                .query_map(params![entry_id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })?
                .collect::<Result<_, _>>()?;
            let incoming: Vec<(String, String, String, String)> = details
                .iter()
                .map(|detail| {
                    (
                        detail.location.clone(),
                        detail.mode.clone(),
                        detail.name.clone(),
                        detail.unit.clone().unwrap_or_default(),
                    )
                })
                .collect();
            if signature == incoming {
                with_txn(conn, |conn| {
                    conn.execute(
                        "UPDATE audit_entry SET ts_utc = (now() AT TIME ZONE 'UTC'),
                             repeat_count = repeat_count + 1, comment = ?2,
                             zone_set_version = ?3, zone_set_digest = ?4
                         WHERE entry_id = ?1",
                        params![
                            entry_id,
                            comment,
                            zone_set.map(|(version, _)| version),
                            zone_set.map(|(_, digest)| digest)
                        ],
                    )?;
                    let mut stmt = conn.prepare(
                        "UPDATE audit_detail SET value = ?3 WHERE entry_id = ?1 AND seq = ?2",
                    )?;
                    for (seq, detail) in details.iter().enumerate() {
                        stmt.execute(params![entry_id, seq as i64, detail.value])?;
                    }
                    Ok::<(), DbError>(())
                })?;
                return Ok(entry_id);
            }
        }
    }

    let entry_id = uuid::Uuid::new_v4().to_string();
    with_txn(conn, |conn| {
        conn.execute(
            "INSERT INTO audit_entry
                (entry_id, well_id, operator, operator_kind, view, source, comment,
                 zone_set_version, zone_set_digest)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                entry_id,
                well_id,
                operator,
                operator_kind,
                view,
                source,
                comment,
                zone_set.map(|(version, _)| version),
                zone_set.map(|(_, digest)| digest)
            ],
        )?;
        for (seq, detail) in details.iter().enumerate() {
            conn.execute(
                "INSERT INTO audit_detail (entry_id, seq, location, mode, unit, name, value)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    entry_id,
                    seq as i64,
                    detail.location,
                    detail.mode,
                    detail.unit,
                    detail.name,
                    detail.value
                ],
            )?;
        }
        Ok::<(), DbError>(())
    })?;
    Ok(entry_id)
}

/// The zone-parameter surface, audited: the write and its audit entry are ONE atomic
/// gesture record. A cleared row is mode DELETE, an applied value mode INPUT; the zone
/// itself rides as an INTERVAL row so the same parameter dragged in two different zones
/// can never collapse into one entry. The parameter's unit comes from the first module
/// manifest that declares it - the same declaration the dialog shows.
pub fn set_zone_param_audited(
    conn: &Connection,
    well_id: &str,
    zone_name: &str,
    param_name: &str,
    value_num: Option<f32>,
    value_text: Option<&str>,
    operator: &str,
    operator_kind: &str,
    view: &str,
) -> DbResult<()> {
    set_zone_param(conn, well_id, zone_name, param_name, value_num, value_text)?;
    let zone_set = current_zone_set(conn, well_id)?;
    let unit = crate::modules::list_modules()
        .iter()
        .flat_map(|module| module.args.iter())
        .find(|argument| {
            argument.kind == crate::modules::ArgKind::Param
                && argument.name == param_name
                && !argument.unit.is_empty()
        })
        .map(|argument| argument.unit.clone());
    let cleared = value_num.is_none() && value_text.is_none();
    let details = [
        AuditDetail {
            location: "INTERVAL".into(),
            mode: if cleared { "DELETE" } else { "INPUT" }.into(),
            unit: None,
            name: zone_name.into(),
            value: None,
        },
        AuditDetail {
            location: "PARAMETER".into(),
            mode: if cleared { "DELETE" } else { "INPUT" }.into(),
            unit,
            name: param_name.into(),
            value: value_num
                .map(|value| value.to_string())
                .or_else(|| value_text.map(str::to_string)),
        },
    ];
    record_audit_entry(
        conn,
        Some(well_id),
        operator,
        operator_kind,
        view,
        "set_zone_param",
        None,
        Some((zone_set.0, zone_set.1.as_str())),
        &details,
    )?;
    Ok(())
}

/// The curve-identity surface, audited: a mnemonic change is mode RENAME on the LOG, and a
/// unit or family change is the dotted-name ATTRIBUTE case the chapter defines.
pub fn update_curve_meta_audited(
    conn: &Connection,
    curve_id: &str,
    mnemonic: &str,
    unit: Option<&str>,
    family: Option<&str>,
    operator: &str,
    operator_kind: &str,
    view: &str,
) -> DbResult<CurveMetaEdit> {
    let well_id: Option<String> = conn
        .query_row(
            "SELECT well_id::VARCHAR FROM curve_meta WHERE curve_id = ?1",
            params![curve_id],
            |row| row.get(0),
        )
        .ok();
    let before = update_curve_meta_fields(conn, curve_id, mnemonic, unit, family)?;
    let mut details = Vec::new();
    let renamed = !before.mnemonic.eq_ignore_ascii_case(mnemonic.trim());
    if renamed {
        details.push(AuditDetail {
            location: "LOG".into(),
            mode: "RENAME".into(),
            unit: None,
            name: before.mnemonic.clone(),
            value: Some(mnemonic.trim().to_uppercase()),
        });
    }
    if before.unit.as_deref() != unit {
        details.push(AuditDetail {
            location: "ATTRIBUTE".into(),
            mode: "INPUT".into(),
            unit: None,
            name: format!("{}.unit", before.mnemonic),
            value: unit.map(str::to_string),
        });
    }
    if before.family.as_deref() != family {
        details.push(AuditDetail {
            location: "ATTRIBUTE".into(),
            mode: "INPUT".into(),
            unit: None,
            name: format!("{}.family", before.mnemonic),
            value: family.map(str::to_string),
        });
    }
    if !details.is_empty() {
        record_audit_entry(
            conn,
            well_id.as_deref(),
            operator,
            operator_kind,
            view,
            "update_curve_meta",
            None,
            None,
            &details,
        )?;
    }
    Ok(before)
}

/// SB-DBM-011: the structured audit, newest first, visible on demand. `details` rides along
/// so the panel needs one call, not one per entry.
#[derive(serde::Serialize)]
pub struct AuditEntryView {
    pub entry_id: String,
    pub well_id: Option<String>,
    pub ts_utc: String,
    pub operator: String,
    pub operator_kind: String,
    pub view: String,
    pub source: String,
    pub comment: Option<String>,
    pub zone_set_version: Option<i64>,
    pub zone_set_digest: Option<String>,
    pub repeat_count: i64,
    pub details: Vec<AuditDetail>,
}

pub fn list_audit_entries(conn: &Connection, limit: usize) -> DbResult<Vec<AuditEntryView>> {
    let mut stmt = conn.prepare(
        "SELECT entry_id, well_id::VARCHAR, ts_utc::VARCHAR, operator, operator_kind, view,
                source, comment, zone_set_version, zone_set_digest, repeat_count
         FROM audit_entry ORDER BY entry_seq DESC LIMIT ?1",
    )?;
    let mut entries: Vec<AuditEntryView> = stmt
        .query_map(params![limit as i64], |row| {
            Ok(AuditEntryView {
                entry_id: row.get(0)?,
                well_id: row.get(1)?,
                ts_utc: row.get(2)?,
                operator: row.get(3)?,
                operator_kind: row.get(4)?,
                view: row.get(5)?,
                source: row.get(6)?,
                comment: row.get(7)?,
                zone_set_version: row.get(8)?,
                zone_set_digest: row.get(9)?,
                repeat_count: row.get(10)?,
                details: Vec::new(),
            })
        })?
        .collect::<Result<_, _>>()?;
    // One scan of the detail table rather than a query per entry - the panel opens at the IPC
    // default of 200 entries, and a per-row query is how a list turns into N round trips on a
    // field-scale project, which is the convention this file states for the contact link table.
    // The same LIMIT selects the same entries, so no binding list is needed either.
    let mut stmt = conn.prepare(
        "SELECT entry_id, location, mode, unit, name, value FROM audit_detail
         WHERE entry_id IN (SELECT entry_id FROM audit_entry ORDER BY entry_seq DESC LIMIT ?1)
         ORDER BY entry_id, seq",
    )?;
    let mut by_entry: std::collections::HashMap<String, Vec<AuditDetail>> =
        std::collections::HashMap::new();
    let rows = stmt.query_map(params![limit as i64], |row| {
        Ok((
            row.get::<_, String>(0)?,
            AuditDetail {
                location: row.get(1)?,
                mode: row.get(2)?,
                unit: row.get(3)?,
                name: row.get(4)?,
                value: row.get(5)?,
            },
        ))
    })?;
    for row in rows {
        let (entry_id, detail) = row?;
        by_entry.entry(entry_id).or_default().push(detail);
    }
    for entry in &mut entries {
        // An entry that recorded no detail rows keeps the empty vector it was built with.
        if let Some(details) = by_entry.remove(&entry.entry_id) {
            entry.details = details;
        }
    }
    Ok(entries)
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

/// Applies a batch of parameter overrides at ONE zone scope in ONE transaction: a `Some` value
/// upserts, a `None` clears that well's override so the step (or manifest) value takes over
/// again. `zone_name` is `*` for the whole-well scope or a named zone. Returns how many rows
/// were written or cleared.
///
/// Atomic on purpose, and that reason is the same in both callers. The parameter grid's
/// fill-column and paste actions touch every well at once; a calibration accepted from
/// Advance ▸ Calibrate… writes its coefficients to every well it was fitted from. A
/// half-applied sweep would leave a field carrying two different parameter sets with no record
/// of where the boundary fell — which for a saturation calibration means two different answers
/// in one study and nothing on the log to say so.
pub fn set_zone_param_batch(
    conn: &mut Connection,
    zone_name: &str,
    entries: &[(String, String, Option<f32>)],
) -> DbResult<usize> {
    let tx = conn.transaction()?;
    let mut n = 0usize;
    for (well_id, param_name, value) in entries {
        match value {
            Some(v) => {
                tx.execute(
                    "INSERT INTO zone_params (well_id, zone_name, param_name, value_num, value_text)
                     VALUES (?1, ?2, ?3, ?4, NULL)
                     ON CONFLICT (well_id, zone_name, param_name)
                     DO UPDATE SET value_num = excluded.value_num, value_text = NULL",
                    params![well_id, zone_name, param_name, v],
                )?;
            }
            None => {
                tx.execute(
                    "DELETE FROM zone_params WHERE well_id = ?1 AND zone_name = ?2 AND param_name = ?3",
                    params![well_id, zone_name, param_name],
                )?;
            }
        }
        n += 1;
    }
    tx.commit()?;
    Ok(n)
}

/// Whole-well scope of [`set_zone_param_batch`] — the parameter grid's own call.
pub fn set_well_param_overrides(
    conn: &mut Connection,
    entries: &[(String, String, Option<f32>)],
) -> DbResult<usize> {
    set_zone_param_batch(conn, "*", entries)
}

/// SB-DIO-057 / DEC-076: the zero gate at the shared curve-commit boundary — every import
/// path (LAS, DLIS, intake) funnels through `insert_curve_samples*`, so the gate is
/// inherited, never re-implemented per importer.
#[cfg(test)]
mod log_scale_zero_tests {
    use super::*;

    fn seed(conn: &Connection) -> String {
        create_schema(conn).unwrap();
        let well = uuid::Uuid::new_v4();
        insert_well(conn, well, "ZERO-GATE", None, None, None).unwrap();
        well.to_string()
    }

    #[test]
    fn a_zero_on_a_log_scale_curve_is_surfaced_before_commit_and_the_recorded_decision_governs()
    {
        let conn = Connection::open_in_memory().unwrap();
        let well = seed(&conn);
        let depths = [1000.0f32, 1000.5, 1001.0, 1001.5];
        let with_zeros = [12.5f32, 0.0, 3.4, 0.0];

        // T84: an undecided zero-bearing resistivity curve REFUSES by name, before commit —
        // nothing is written and nothing is rewritten.
        let res = upsert_curve_meta(
            &conn, &well, "RAW", "RES_DEEP", Some("ohm.m"), Some("RES_DEEP"), None, None,
        )
        .unwrap();
        let error = insert_curve_samples(&conn, &res, &depths, &with_zeros)
            .expect_err("undecided zeros on a log-scale family must refuse")
            .to_string();
        assert!(error.contains("RES_DEEP"), "the refusal names the curve and family: {error}");
        assert!(error.contains("2 exact zero"), "and counts them: {error}");
        assert!(error.contains("SB-DIO-057"), "and cites the rule: {error}");
        let committed: i64 = conn
            .query_row(
                "SELECT count(*) FROM curve_samples WHERE curve_id = ?1",
                params![res],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(committed, 0, "a refusal writes nothing");

        // T85: the interpreter DECLINES the conversion — zeros commit as VALUES, and the
        // decision is a recorded row.
        confirm_log_scale_zeros(&conn, &well, "RES_DEEP", true).unwrap();
        insert_curve_samples(&conn, &res, &depths, &with_zeros).unwrap();
        let kept: Vec<Option<f32>> = conn
            .prepare("SELECT value FROM curve_samples WHERE curve_id = ?1 ORDER BY depth")
            .unwrap()
            .query_map(params![res], |row| row.get(0))
            .unwrap()
            .collect::<duckdb::Result<_>>()
            .unwrap();
        assert_eq!(kept[1], Some(0.0), "a declined conversion keeps the zero as a value");
        assert_eq!(kept[3], Some(0.0));
        let recorded: i64 = conn
            .query_row(
                "SELECT count(*) FROM documents WHERE doc_type = 'zero-decision'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(recorded, 1, "the decision is recorded, not just applied");

        // The explicit CONVERT decision — zeros become MISSING at commit, never silently.
        let rxo = upsert_curve_meta(
            &conn, &well, "RAW", "RXO", Some("ohm.m"), Some("RXO"), None, None,
        )
        .unwrap();
        confirm_log_scale_zeros(&conn, &well, "RXO", false).unwrap();
        insert_curve_samples(&conn, &rxo, &depths, &with_zeros).unwrap();
        let converted: Vec<Option<f32>> = conn
            .prepare("SELECT value FROM curve_samples WHERE curve_id = ?1 ORDER BY depth")
            .unwrap()
            .query_map(params![rxo], |row| row.get(0))
            .unwrap()
            .collect::<duckdb::Result<_>>()
            .unwrap();
        assert_eq!(converted[0], Some(12.5), "real readings survive a convert decision");
        assert_eq!(converted[1], None, "a converted zero is MISSING, not a reading");
        assert_eq!(converted[3], None);

        // Both negative arms: a LINEAR family with zeros passes ungated (a genuine zero is
        // a legitimate reading there), and a log-family curve with no zeros never asks.
        let gr = upsert_curve_meta(
            &conn, &well, "RAW", "GR", Some("gAPI"), Some("GR"), None, None,
        )
        .unwrap();
        insert_curve_samples(&conn, &gr, &depths, &with_zeros)
            .expect("a linear-family curve with zeros commits without a gate");
        let clean = [12.5f32, 8.0, 3.4, 9.9];
        let resm = upsert_curve_meta(
            &conn, &well, "RAW", "RES_MED", Some("ohm.m"), Some("RES_MED"), None, None,
        )
        .unwrap();
        insert_curve_samples(&conn, &resm, &depths, &clean)
            .expect("a zero-free log-family curve commits without a gate");
    }
}
