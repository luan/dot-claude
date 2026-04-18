use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, params};

use super::{Telemetry, TelemetryError};

/// Produce a text report summarising telemetry for one project db over the
/// last `days` days. The "transitions observed" line is best-effort — it
/// counts failure-then-success pairs per error_kind within the same session
/// on the same file, a coarse proxy for Phase 3's classifier threshold.
pub fn run(tel: &Telemetry, project_name: &str, days: i64) -> Result<String, TelemetryError> {
    let cutoff = now_ms() - days * 86_400_000;
    let summary = summarize(tel, cutoff)?;
    Ok(render(project_name, days, &summary))
}

/// Walk `<data_local>/ct/projects/`, open each project's db, and join the
/// reports. Projects missing a db file are skipped silently — telemetry is
/// best-effort, and a stale directory shouldn't abort the sweep.
pub fn run_all_projects(days: i64) -> Result<String, TelemetryError> {
    let root = projects_root()?;
    if !root.is_dir() {
        return Ok("(no telemetry data — projects directory not found)".into());
    }

    let mut names: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let db = entry.path().join("apply_patch.db");
        if !db.is_file() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_string());
        }
    }
    names.sort();

    if names.is_empty() {
        return Ok("(no telemetry data — no project databases found)".into());
    }

    let mut reports = Vec::new();
    for name in names {
        let tel = Telemetry::open(&name)?;
        reports.push(run(&tel, &name, days)?);
    }
    Ok(reports.join("\n---\n\n"))
}

fn projects_root() -> Result<PathBuf, TelemetryError> {
    let base =
        dirs::data_local_dir().ok_or_else(|| TelemetryError::Path("no data_local_dir".into()))?;
    Ok(base.join("ct").join("projects"))
}

struct Summary {
    total_calls: i64,
    successes: i64,
    errors: i64,
    error_kinds: Vec<(String, i64)>,
    top_anchors: Vec<AnchorRow>,
    transitions: i64,
    threshold_met: bool,
}

struct AnchorRow {
    file_path: String,
    anchor_text: Option<String>,
    total: i64,
    fails: i64,
}

/// Count paired failure→success transitions in the window. Pairs are computed
/// per-session, ordered by ts: each failure matches the IMMEDIATE next success
/// whose `files_json` path set intersects with the failure's paths. A success
/// is consumed once, so a single fix (F1 F2 F3 S1) counts once, not three
/// times.
fn count_transitions(conn: &Connection, cutoff: i64) -> rusqlite::Result<i64> {
    let mut stmt = conn.prepare(
        "SELECT session_id, ts, outcome, files_json
         FROM calls
         WHERE ts >= ?1
         ORDER BY ts ASC, id ASC",
    )?;
    let rows = stmt
        .query_map(params![cutoff], |r| {
            Ok(TransitionRow {
                session_id: r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                ts: r.get(1)?,
                outcome: r.get(2)?,
                files_json: r.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut by_session: HashMap<String, Vec<TransitionRow>> = HashMap::new();
    for r in rows {
        by_session.entry(r.session_id.clone()).or_default().push(r);
    }
    let mut count: i64 = 0;
    for calls in by_session.values_mut() {
        calls.sort_by_key(|c| c.ts);
        let mut consumed = vec![false; calls.len()];
        for i in 0..calls.len() {
            if calls[i].outcome == "success" {
                continue;
            }
            let fail_paths = paths_from_files_json(&calls[i].files_json);
            for j in (i + 1)..calls.len() {
                if consumed[j] || calls[j].outcome != "success" {
                    continue;
                }
                let win_paths = paths_from_files_json(&calls[j].files_json);
                if paths_overlap(&fail_paths, &win_paths) {
                    count += 1;
                    consumed[j] = true;
                    break;
                }
            }
        }
    }
    Ok(count)
}

struct TransitionRow {
    session_id: String,
    ts: i64,
    outcome: String,
    files_json: String,
}

fn paths_from_files_json(files_json: &str) -> Vec<String> {
    let v: serde_json::Value = match serde_json::from_str(files_json) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| {
                    entry
                        .get("path")
                        .and_then(|p| p.as_str())
                        .map(str::to_string)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn paths_overlap(a: &[String], b: &[String]) -> bool {
    a.iter().any(|p| b.contains(p))
}

fn summarize(tel: &Telemetry, cutoff: i64) -> Result<Summary, TelemetryError> {
    tel.with_conn(|conn| {
        let total_calls: i64 = conn.query_row(
            "SELECT COUNT(*) FROM calls WHERE ts >= ?1",
            params![cutoff],
            |r| r.get(0),
        )?;
        let successes: i64 = conn.query_row(
            "SELECT COUNT(*) FROM calls WHERE ts >= ?1 AND outcome = 'success'",
            params![cutoff],
            |r| r.get(0),
        )?;
        let errors = total_calls - successes;

        let mut error_kinds: Vec<(String, i64)> = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT error_kind, COUNT(*) FROM calls
                 WHERE ts >= ?1 AND error_kind IS NOT NULL
                 GROUP BY error_kind
                 ORDER BY COUNT(*) DESC",
            )?;
            let rows = stmt.query_map(params![cutoff], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            })?;
            for row in rows {
                error_kinds.push(row?);
            }
        }

        let mut top_anchors: Vec<AnchorRow> = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT a.file_path, a.anchor_text, COUNT(*) AS total,
                        SUM(a.success) AS ok
                 FROM anchor_attempts a
                 JOIN calls c ON a.call_id = c.id
                 WHERE c.ts >= ?1
                 GROUP BY a.file_path, a.anchor_text
                 HAVING total >= 3
                 ORDER BY (total - ok) * 1.0 / total DESC, total DESC
                 LIMIT 5",
            )?;
            let rows = stmt.query_map(params![cutoff], |r| {
                let file_path: String = r.get(0)?;
                let anchor_text: Option<String> = r.get(1)?;
                let total: i64 = r.get(2)?;
                let ok: i64 = r.get(3)?;
                Ok(AnchorRow {
                    file_path,
                    anchor_text,
                    total,
                    fails: total - ok,
                })
            })?;
            for row in rows {
                let row = row?;
                if row.fails > 0 {
                    top_anchors.push(row);
                }
            }
        }

        // Transitions proxy: count failure-then-success pairs sharing a
        // session_id whose `files_json` path sets overlap. JSON equality
        // (the old approach) falsely failed to match because fingerprints and
        // file_sha1 differ between failure and success. Pair in Rust: walk
        // each session in ts-order, match each failure with the IMMEDIATE next
        // success whose paths overlap, and consume that success so a single
        // fix only counts once.
        let transitions = count_transitions(conn, cutoff)?;

        // Threshold: we enable the Phase-3 classifier hints once we've
        // observed enough paired transitions — not enough *failures of a
        // single kind*, which is what the old `error_kinds >= 20` check
        // measured (they're unrelated metrics).
        let threshold_met = transitions >= 20;

        Ok::<_, rusqlite::Error>(Summary {
            total_calls,
            successes,
            errors,
            error_kinds,
            top_anchors,
            transitions,
            threshold_met,
        })
    })
    .map_err(TelemetryError::from)
}

fn render(project_name: &str, days: i64, s: &Summary) -> String {
    let mut out = String::new();
    out.push_str(&format!("project: {project_name}  (last {days} days)\n"));
    let rate = if s.total_calls > 0 {
        (s.successes as f64 * 100.0 / s.total_calls as f64).round() as i64
    } else {
        0
    };
    out.push_str(&format!(
        "  calls: {}   successes: {} ({}%)   errors: {}\n",
        s.total_calls, s.successes, rate, s.errors
    ));

    if s.error_kinds.is_empty() {
        out.push_str("  error kinds: (none)\n");
    } else {
        let parts: Vec<String> = s
            .error_kinds
            .iter()
            .map(|(k, n)| format!("{k} ({n})"))
            .collect();
        out.push_str(&format!("  error kinds: {}\n", parts.join(", ")));
    }

    if s.top_anchors.is_empty() {
        out.push_str("  top anchors by failure rate (min 3 attempts): (none)\n");
    } else {
        out.push_str("  top anchors by failure rate (min 3 attempts):\n");
        for row in &s.top_anchors {
            let anchor = match &row.anchor_text {
                Some(a) => format!("@@ {a}"),
                None => "bare @@".to_string(),
            };
            out.push_str(&format!(
                "    {anchor}  — {}/{} fail  ({})\n",
                row.fails, row.total, row.file_path
            ));
        }
    }

    let status = if s.threshold_met {
        "threshold met, hints active".to_string()
    } else {
        format!("{}/20 — below threshold", s.transitions)
    };
    out.push_str(&format!(
        "  transitions observed: {} ({status})\n",
        s.transitions
    ));
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
    use crate::apply_patch::telemetry::{AnchorAttempt, CallRecord, FileCallEntry};

    fn seed_call(tel: &Telemetry, outcome: &str, error_kind: Option<&str>) -> i64 {
        tel.record_call(&CallRecord {
            outcome: outcome.into(),
            error_kind: error_kind.map(str::to_string),
            files: vec![FileCallEntry {
                path: "foo.rs".into(),
                chunk_count: 1,
                fuzzy_tier_used: None,
                file_sha1: None,
            }],
            duration_us: 0,
            patch_sha: format!("sha-{}-{}", outcome, error_kind.unwrap_or("none")),
            fingerprints_json: "{}".into(),
        })
        .unwrap()
    }

    #[test]
    fn empty_db_produces_clean_report() {
        let t = Telemetry::open_in_memory().unwrap();
        let out = run(&t, "emptyproj", 30).unwrap();
        assert!(out.contains("project: emptyproj"), "report was: {out}");
        assert!(out.contains("calls: 0"), "report was: {out}");
        assert!(out.contains("error kinds: (none)"), "report was: {out}");
    }

    #[test]
    fn reports_error_distribution() {
        let t = Telemetry::open_in_memory().unwrap();
        for _ in 0..3 {
            seed_call(&t, "success", None);
        }
        for _ in 0..2 {
            seed_call(&t, "error", Some("context_not_found"));
        }
        seed_call(&t, "error", Some("ambiguous_context"));

        let out = run(&t, "test", 30).unwrap();
        assert!(out.contains("calls: 6"), "report was: {out}");
        assert!(out.contains("successes: 3 (50%)"), "report was: {out}");
        assert!(out.contains("context_not_found (2)"), "report was: {out}");
        assert!(out.contains("ambiguous_context (1)"), "report was: {out}");
    }

    #[test]
    fn lists_top_failing_anchors() {
        let t = Telemetry::open_in_memory().unwrap();
        let cid = seed_call(&t, "error", Some("context_not_found"));
        let mut attempts = Vec::new();
        // ("foo.rs", "fn bar(") — 4 attempts, 1 success, 3 fails (75% fail).
        for i in 0..4 {
            attempts.push(AnchorAttempt {
                file_path: "foo.rs".into(),
                chunk_index: 0,
                anchor_text: Some("fn bar(".into()),
                success: i == 0,
                fuzzy_tier: None,
            });
        }
        // ("foo.rs", "impl Baz") — 3 attempts, 2 success, 1 fail (33% fail).
        for i in 0..3 {
            attempts.push(AnchorAttempt {
                file_path: "foo.rs".into(),
                chunk_index: 0,
                anchor_text: Some("impl Baz".into()),
                success: i < 2,
                fuzzy_tier: None,
            });
        }
        // ("foo.rs", "rare") — 2 attempts, filtered by min-3.
        for _ in 0..2 {
            attempts.push(AnchorAttempt {
                file_path: "foo.rs".into(),
                chunk_index: 0,
                anchor_text: Some("rare".into()),
                success: false,
                fuzzy_tier: None,
            });
        }
        t.record_anchor_attempts(cid, &attempts).unwrap();

        let out = run(&t, "test", 30).unwrap();
        assert!(out.contains("@@ fn bar("), "report was: {out}");
        assert!(out.contains("3/4 fail"), "report was: {out}");
        assert!(!out.contains("@@ rare"), "report was: {out}");
        // "@@ fn bar(" appears before "@@ impl Baz" — higher failure rate first.
        let bar_idx = out.find("@@ fn bar(").unwrap();
        let baz_idx = out.find("@@ impl Baz").unwrap();
        assert!(bar_idx < baz_idx, "expected fn bar( first; report: {out}");
    }

    #[test]
    fn threshold_met_tracks_transitions_not_error_kind_count() {
        // Previously this check used error_kind counts — 25 failures of a
        // single kind would flag the threshold as "met" even with zero
        // transitions, which is a different metric entirely.
        let t = Telemetry::open_in_memory().unwrap();
        for _ in 0..25 {
            seed_call(&t, "error", Some("context_not_found"));
        }
        let out = run(&t, "test", 30).unwrap();
        assert!(out.contains("0/20 — below threshold"), "report was: {out}");
        assert!(!out.contains("threshold met"), "report was: {out}");
    }

    #[test]
    fn transitions_count_dedups_multiple_failures_against_one_success() {
        // F1 F2 F3 S1 in the same session should count as ONE transition —
        // the three failures represent iterations toward one fix, not three
        // independent pairs.
        let t = Telemetry::open_in_memory().unwrap();
        for _ in 0..3 {
            seed_call(&t, "error", Some("context_not_found"));
        }
        seed_call(&t, "success", None);
        let out = run(&t, "test", 30).unwrap();
        assert!(out.contains("transitions observed: 1"), "report was: {out}");
    }
}
