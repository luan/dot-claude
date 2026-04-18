pub mod enrich;
pub mod prune;
mod schema;
pub mod stats;

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::OptionalExtension;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("path: {0}")]
    Path(String),
    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCallEntry {
    pub path: String,
    pub chunk_count: usize,
    pub fuzzy_tier_used: Option<String>,
    pub file_sha1: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CallRecord {
    pub outcome: String,
    pub error_kind: Option<String>,
    pub files: Vec<FileCallEntry>,
    pub duration_us: u64,
    pub patch_sha: String,
    pub fingerprints_json: String,
}

#[derive(Debug, Clone)]
pub struct AnchorAttempt {
    pub file_path: String,
    pub chunk_index: usize,
    pub anchor_text: Option<String>,
    pub success: bool,
    pub fuzzy_tier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint {
    pub mtime_ns: i64,
    pub sha1: String,
    pub ts: i64,
}

pub struct Telemetry {
    conn: Mutex<Connection>,
    session_id: String,
}

impl Telemetry {
    pub fn open(project_name: &str) -> Result<Self, TelemetryError> {
        let base = dirs::data_local_dir()
            .ok_or_else(|| TelemetryError::Path("no data_local_dir".into()))?;
        let dir: PathBuf = base.join("ct").join("projects").join(project_name);
        std::fs::create_dir_all(&dir)?;
        let db_path = dir.join("apply_patch.db");
        let conn = Connection::open(&db_path)?;
        Self::init_conn(conn)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, TelemetryError> {
        let conn = Connection::open_in_memory()?;
        Self::init_conn(conn)
    }

    fn init_conn(conn: Connection) -> Result<Self, TelemetryError> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        schema::apply(&conn)?;
        let session_id = format!("{}-{}", std::process::id(), now_ms());
        Ok(Self {
            conn: Mutex::new(conn),
            session_id,
        })
    }

    pub fn record_call(&self, rec: &CallRecord) -> Result<i64, TelemetryError> {
        let files_json = serde_json::to_string(&rec.files)?;
        let ts = now_ms();
        let conn = self.lock_conn();
        conn.execute(
            "INSERT INTO calls (ts, session_id, outcome, error_kind, files_json, duration_us, patch_sha, fingerprints_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                ts,
                &self.session_id,
                &rec.outcome,
                &rec.error_kind,
                &files_json,
                rec.duration_us as i64,
                &rec.patch_sha,
                &rec.fingerprints_json,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn record_patch_body(&self, patch_sha: &str, body: &str) -> Result<(), TelemetryError> {
        let conn = self.lock_conn();
        conn.execute(
            "INSERT OR IGNORE INTO patch_bodies (patch_sha, body, first_seen_ts) VALUES (?1, ?2, ?3)",
            params![patch_sha, body, now_ms()],
        )?;
        Ok(())
    }

    pub fn record_anchor_attempts(
        &self,
        call_id: i64,
        attempts: &[AnchorAttempt],
    ) -> Result<(), TelemetryError> {
        let conn = self.lock_conn();
        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO anchor_attempts (call_id, file_path, chunk_index, anchor_text, success, fuzzy_tier)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for a in attempts {
                stmt.execute(params![
                    call_id,
                    &a.file_path,
                    a.chunk_index as i64,
                    &a.anchor_text,
                    i64::from(a.success),
                    &a.fuzzy_tier,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn upsert_fingerprint(
        &self,
        file_path: &str,
        fp: &Fingerprint,
    ) -> Result<(), TelemetryError> {
        let conn = self.lock_conn();
        conn.execute(
            "INSERT INTO file_fingerprints (file_path, last_seen_mtime_ns, last_seen_sha1, last_seen_ts)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(file_path) DO UPDATE SET
                 last_seen_mtime_ns = excluded.last_seen_mtime_ns,
                 last_seen_sha1 = excluded.last_seen_sha1,
                 last_seen_ts = excluded.last_seen_ts",
            params![file_path, fp.mtime_ns, &fp.sha1, fp.ts],
        )?;
        Ok(())
    }

    pub fn last_fingerprint(&self, file_path: &str) -> Result<Option<Fingerprint>, TelemetryError> {
        let conn = self.lock_conn();
        let row = conn
            .query_row(
                "SELECT last_seen_mtime_ns, last_seen_sha1, last_seen_ts
                 FROM file_fingerprints WHERE file_path = ?1",
                params![file_path],
                |row| {
                    Ok(Fingerprint {
                        mtime_ns: row.get(0)?,
                        sha1: row.get(1)?,
                        ts: row.get(2)?,
                    })
                },
            )
            .optional()?;
        Ok(row)
    }

    /// Lock the inner connection, recovering transparently from poisoning.
    /// A poisoned mutex means an earlier call panicked mid-query — the inner
    /// connection is still usable, and the next telemetry write will either
    /// succeed or log to stderr.
    fn lock_conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        match self.conn.lock() {
            Ok(p) => p,
            Err(p) => p.into_inner(),
        }
    }

    /// Run a read-only closure against the underlying connection. Used by the
    /// enricher to run ad-hoc queries without leaking a `MutexGuard` across
    /// module boundaries.
    pub fn with_conn<R>(&self, f: impl FnOnce(&Connection) -> R) -> R {
        let conn = self.lock_conn();
        f(&conn)
    }
}

pub fn sha1_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        write!(&mut out, "{byte:02x}").expect("writing to String never fails");
    }
    out
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_call() -> CallRecord {
        CallRecord {
            outcome: "ok".into(),
            error_kind: None,
            files: vec![FileCallEntry {
                path: "src/main.rs".into(),
                chunk_count: 2,
                fuzzy_tier_used: Some("exact".into()),
                file_sha1: Some("abc123".into()),
            }],
            duration_us: 4321,
            patch_sha: "deadbeef".into(),
            fingerprints_json: "{}".into(),
        }
    }

    #[test]
    fn records_call_and_anchor_attempts() {
        let t = Telemetry::open_in_memory().unwrap();
        let call_id = t.record_call(&sample_call()).unwrap();
        let attempts = vec![
            AnchorAttempt {
                file_path: "src/main.rs".into(),
                chunk_index: 0,
                anchor_text: Some("fn main".into()),
                success: true,
                fuzzy_tier: None,
            },
            AnchorAttempt {
                file_path: "src/main.rs".into(),
                chunk_index: 1,
                anchor_text: Some("let x".into()),
                success: false,
                fuzzy_tier: Some("whitespace".into()),
            },
            AnchorAttempt {
                file_path: "src/lib.rs".into(),
                chunk_index: 0,
                anchor_text: None,
                success: true,
                fuzzy_tier: None,
            },
        ];
        t.record_anchor_attempts(call_id, &attempts).unwrap();

        let conn = t.lock_conn();
        let calls_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM calls", [], |row| row.get(0))
            .unwrap();
        assert_eq!(calls_count, 1);

        let (outcome, patch_sha, duration_us): (String, String, i64) = conn
            .query_row(
                "SELECT outcome, patch_sha, duration_us FROM calls WHERE id = ?1",
                params![call_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(outcome, "ok");
        assert_eq!(patch_sha, "deadbeef");
        assert_eq!(duration_us, 4321);

        let attempt_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM anchor_attempts WHERE call_id = ?1",
                params![call_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(attempt_count, 3);

        let success_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM anchor_attempts WHERE call_id = ?1 AND success = 1",
                params![call_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(success_count, 2);
    }

    #[test]
    fn fingerprint_upsert_latest_wins() {
        let t = Telemetry::open_in_memory().unwrap();
        let path = "src/app.rs";
        t.upsert_fingerprint(
            path,
            &Fingerprint {
                mtime_ns: 100,
                sha1: "aaa".into(),
                ts: 1,
            },
        )
        .unwrap();
        t.upsert_fingerprint(
            path,
            &Fingerprint {
                mtime_ns: 200,
                sha1: "bbb".into(),
                ts: 2,
            },
        )
        .unwrap();

        let got = t.last_fingerprint(path).unwrap().unwrap();
        assert_eq!(
            got,
            Fingerprint {
                mtime_ns: 200,
                sha1: "bbb".into(),
                ts: 2,
            }
        );

        let rows: i64 = t
            .lock_conn()
            .query_row("SELECT COUNT(*) FROM file_fingerprints", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(rows, 1);
    }

    #[test]
    fn last_fingerprint_missing_returns_none() {
        let t = Telemetry::open_in_memory().unwrap();
        assert!(t.last_fingerprint("nope.rs").unwrap().is_none());
    }

    #[test]
    fn user_version_matches_schema_version() {
        let t = Telemetry::open_in_memory().unwrap();
        let version: i32 = t
            .lock_conn()
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, schema::SCHEMA_VERSION);
    }

    #[test]
    fn migrates_v1_to_v2() {
        let conn = Connection::open_in_memory().unwrap();
        // Hand-build a v1 database: `calls` without fingerprints_json, no
        // `patch_bodies` table.
        conn.execute_batch(
            "CREATE TABLE calls (
                id INTEGER PRIMARY KEY,
                ts INTEGER NOT NULL,
                session_id TEXT,
                outcome TEXT NOT NULL,
                error_kind TEXT,
                files_json TEXT NOT NULL,
                duration_us INTEGER NOT NULL,
                patch_sha TEXT NOT NULL
            );
            CREATE TABLE anchor_attempts (
                call_id INTEGER NOT NULL REFERENCES calls(id),
                file_path TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                anchor_text TEXT,
                success INTEGER NOT NULL,
                fuzzy_tier TEXT
            );
            CREATE TABLE file_fingerprints (
                file_path TEXT PRIMARY KEY,
                last_seen_mtime_ns INTEGER NOT NULL,
                last_seen_sha1 TEXT NOT NULL,
                last_seen_ts INTEGER NOT NULL
            );",
        )
        .unwrap();
        conn.pragma_update(None, "user_version", 1i32).unwrap();

        schema::apply(&conn).unwrap();

        let version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 2);

        // New column exists on calls.
        let has_fp: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('calls') WHERE name = 'fingerprints_json'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(has_fp, 1);

        // patch_bodies table exists.
        let has_table: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'patch_bodies'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(has_table, 1);
    }

    #[test]
    fn sha1_hex_known_answer() {
        assert_eq!(sha1_hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
    }

    // Exercises the Phase 2 flow end-to-end without the MCP layer: one call,
    // several anchor attempts, two fingerprint upserts. Guards against a
    // record-path regression that unit tests alone would miss.
    #[test]
    fn phase2_flow_records_call_attempts_and_fingerprints() {
        let t = Telemetry::open_in_memory().unwrap();

        let call_id = t
            .record_call(&CallRecord {
                outcome: "success".into(),
                error_kind: None,
                files: vec![
                    FileCallEntry {
                        path: "a.rs".into(),
                        chunk_count: 2,
                        fuzzy_tier_used: Some("exact".into()),
                        file_sha1: Some("a1".into()),
                    },
                    FileCallEntry {
                        path: "b.rs".into(),
                        chunk_count: 1,
                        fuzzy_tier_used: None,
                        file_sha1: Some("b1".into()),
                    },
                ],
                duration_us: 2000,
                patch_sha: "cafebabe".into(),
                fingerprints_json: "{}".into(),
            })
            .unwrap();

        t.record_anchor_attempts(
            call_id,
            &[
                AnchorAttempt {
                    file_path: "a.rs".into(),
                    chunk_index: 0,
                    anchor_text: Some("fn one".into()),
                    success: true,
                    fuzzy_tier: Some("exact".into()),
                },
                AnchorAttempt {
                    file_path: "a.rs".into(),
                    chunk_index: 1,
                    anchor_text: None,
                    success: true,
                    fuzzy_tier: Some("trim_end".into()),
                },
                AnchorAttempt {
                    file_path: "b.rs".into(),
                    chunk_index: 0,
                    anchor_text: Some("fn two".into()),
                    success: true,
                    fuzzy_tier: Some("exact".into()),
                },
            ],
        )
        .unwrap();

        t.upsert_fingerprint(
            "a.rs",
            &Fingerprint {
                mtime_ns: 10,
                sha1: "a1".into(),
                ts: 100,
            },
        )
        .unwrap();
        t.upsert_fingerprint(
            "b.rs",
            &Fingerprint {
                mtime_ns: 20,
                sha1: "b1".into(),
                ts: 101,
            },
        )
        .unwrap();

        let conn = t.lock_conn();
        let calls: i64 = conn
            .query_row("SELECT COUNT(*) FROM calls", [], |r| r.get(0))
            .unwrap();
        assert_eq!(calls, 1);

        let attempts: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM anchor_attempts WHERE call_id = ?1",
                params![call_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(attempts, 3);

        let fps: i64 = conn
            .query_row("SELECT COUNT(*) FROM file_fingerprints", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fps, 2);
    }
}
