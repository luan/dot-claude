use rusqlite::Connection;

pub const SCHEMA_VERSION: i32 = 2;

const V2_FRESH: &str = "CREATE TABLE IF NOT EXISTS calls (
    id INTEGER PRIMARY KEY,
    ts INTEGER NOT NULL,
    session_id TEXT,
    outcome TEXT NOT NULL,
    error_kind TEXT,
    files_json TEXT NOT NULL,
    duration_us INTEGER NOT NULL,
    patch_sha TEXT NOT NULL,
    fingerprints_json TEXT
);

CREATE TABLE IF NOT EXISTS anchor_attempts (
    call_id INTEGER NOT NULL REFERENCES calls(id),
    file_path TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    anchor_text TEXT,
    success INTEGER NOT NULL,
    fuzzy_tier TEXT
);

CREATE INDEX IF NOT EXISTS idx_anchor_attempts_file_anchor
    ON anchor_attempts(file_path, anchor_text);

CREATE TABLE IF NOT EXISTS file_fingerprints (
    file_path TEXT PRIMARY KEY,
    last_seen_mtime_ns INTEGER NOT NULL,
    last_seen_sha1 TEXT NOT NULL,
    last_seen_ts INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS patch_bodies (
    patch_sha TEXT PRIMARY KEY,
    body TEXT NOT NULL,
    first_seen_ts INTEGER NOT NULL
);";

const V1_TO_V2: &str = "ALTER TABLE calls ADD COLUMN fingerprints_json TEXT;

CREATE TABLE IF NOT EXISTS patch_bodies (
    patch_sha TEXT PRIMARY KEY,
    body TEXT NOT NULL,
    first_seen_ts INTEGER NOT NULL
);";

/// Indexes that serve the Phase-3 enricher queries. Idempotent — applied on
/// every open so v2 databases that shipped without them get them on next use
/// without requiring a schema bump.
const INDEXES: &str = "CREATE INDEX IF NOT EXISTS idx_calls_ts ON calls(ts);
CREATE INDEX IF NOT EXISTS idx_calls_session_ts ON calls(session_id, ts);
CREATE INDEX IF NOT EXISTS idx_calls_error_kind_ts ON calls(error_kind, ts) WHERE error_kind IS NOT NULL;";

pub fn apply(conn: &Connection) -> rusqlite::Result<()> {
    let current: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    match current {
        0 => {
            conn.execute_batch(V2_FRESH)?;
            conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        1 => {
            conn.execute_batch(V1_TO_V2)?;
            conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        2 => {}
        other => {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                Some(format!(
                    "apply_patch telemetry: unknown schema user_version = {other}; expected 0, 1, or {SCHEMA_VERSION}"
                )),
            ));
        }
    }
    conn.execute_batch(INDEXES)?;
    Ok(())
}
