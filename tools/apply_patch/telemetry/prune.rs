use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::params;

use super::{Telemetry, TelemetryError};

pub struct PruneReport {
    pub calls_deleted: u64,
    pub anchor_attempts_deleted: u64,
    pub patch_bodies_deleted: u64,
}

/// Delete `calls`, `anchor_attempts`, and `patch_bodies` rows older than
/// `days` days, then `VACUUM` to reclaim disk space.
pub fn run(tel: &Telemetry, days: i64) -> Result<PruneReport, TelemetryError> {
    let cutoff = now_ms() - days * 86_400_000;
    let (calls_deleted, anchor_attempts_deleted, patch_bodies_deleted) = tel.with_conn(|conn| {
        let anchor_attempts_deleted = conn.execute(
            "DELETE FROM anchor_attempts
             WHERE call_id IN (SELECT id FROM calls WHERE ts < ?1)",
            params![cutoff],
        )?;
        let calls_deleted = conn.execute("DELETE FROM calls WHERE ts < ?1", params![cutoff])?;
        let patch_bodies_deleted = conn.execute(
            "DELETE FROM patch_bodies WHERE first_seen_ts < ?1",
            params![cutoff],
        )?;
        // VACUUM must run outside any active transaction, so keep it here
        // in the same read-write block — with_conn doesn't open one.
        conn.execute("VACUUM", [])?;
        Ok::<_, rusqlite::Error>((calls_deleted, anchor_attempts_deleted, patch_bodies_deleted))
    })?;

    Ok(PruneReport {
        calls_deleted: calls_deleted as u64,
        anchor_attempts_deleted: anchor_attempts_deleted as u64,
        patch_bodies_deleted: patch_bodies_deleted as u64,
    })
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
    use crate::apply_patch::telemetry::{AnchorAttempt, CallRecord, FileCallEntry};

    fn seed_call(tel: &Telemetry, patch_sha: &str) -> i64 {
        tel.record_call(&CallRecord {
            outcome: "success".into(),
            error_kind: None,
            files: vec![FileCallEntry {
                path: "foo.rs".into(),
                chunk_count: 1,
                fuzzy_tier_used: None,
                file_sha1: None,
            }],
            duration_us: 0,
            patch_sha: patch_sha.into(),
            fingerprints_json: "{}".into(),
        })
        .unwrap()
    }

    fn backdate_call(tel: &Telemetry, call_id: i64, ts: i64) {
        tel.with_conn(|conn| {
            conn.execute(
                "UPDATE calls SET ts = ?1 WHERE id = ?2",
                params![ts, call_id],
            )
            .unwrap();
        });
    }

    fn backdate_body(tel: &Telemetry, patch_sha: &str, ts: i64) {
        tel.with_conn(|conn| {
            conn.execute(
                "UPDATE patch_bodies SET first_seen_ts = ?1 WHERE patch_sha = ?2",
                params![ts, patch_sha],
            )
            .unwrap();
        });
    }

    #[test]
    fn prunes_rows_older_than_days() {
        let t = Telemetry::open_in_memory().unwrap();
        let old_id = seed_call(&t, "old");
        let new_id = seed_call(&t, "new");
        // 100 days ago — outside the 90-day window.
        backdate_call(&t, old_id, now_ms() - 100 * 86_400_000);
        // new_id keeps its fresh ts from record_call.

        let report = run(&t, 90).unwrap();
        assert_eq!(report.calls_deleted, 1);

        let remaining: i64 = t.with_conn(|conn| {
            conn.query_row("SELECT COUNT(*) FROM calls", [], |r| r.get(0))
                .unwrap()
        });
        assert_eq!(remaining, 1);

        let new_still_there: i64 = t.with_conn(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM calls WHERE id = ?1",
                params![new_id],
                |r| r.get(0),
            )
            .unwrap()
        });
        assert_eq!(new_still_there, 1);
    }

    #[test]
    fn prune_cascades_to_anchor_attempts_and_patch_bodies() {
        let t = Telemetry::open_in_memory().unwrap();
        let old_id = seed_call(&t, "old");
        let new_id = seed_call(&t, "new");
        t.record_anchor_attempts(
            old_id,
            &[AnchorAttempt {
                file_path: "foo.rs".into(),
                chunk_index: 0,
                anchor_text: Some("fn old".into()),
                success: true,
                fuzzy_tier: None,
            }],
        )
        .unwrap();
        t.record_anchor_attempts(
            new_id,
            &[AnchorAttempt {
                file_path: "foo.rs".into(),
                chunk_index: 0,
                anchor_text: Some("fn new".into()),
                success: true,
                fuzzy_tier: None,
            }],
        )
        .unwrap();
        t.record_patch_body("old", "*** Begin Patch\nold\n*** End Patch\n")
            .unwrap();
        t.record_patch_body("new", "*** Begin Patch\nnew\n*** End Patch\n")
            .unwrap();

        let stale = now_ms() - 100 * 86_400_000;
        backdate_call(&t, old_id, stale);
        backdate_body(&t, "old", stale);

        let report = run(&t, 90).unwrap();
        assert_eq!(report.calls_deleted, 1);
        assert_eq!(report.anchor_attempts_deleted, 1);
        assert_eq!(report.patch_bodies_deleted, 1);

        let (calls, attempts, bodies): (i64, i64, i64) = t.with_conn(|conn| {
            let c: i64 = conn
                .query_row("SELECT COUNT(*) FROM calls", [], |r| r.get(0))
                .unwrap();
            let a: i64 = conn
                .query_row("SELECT COUNT(*) FROM anchor_attempts", [], |r| r.get(0))
                .unwrap();
            let b: i64 = conn
                .query_row("SELECT COUNT(*) FROM patch_bodies", [], |r| r.get(0))
                .unwrap();
            (c, a, b)
        });
        assert_eq!(calls, 1);
        assert_eq!(attempts, 1);
        assert_eq!(bodies, 1);
    }
}
