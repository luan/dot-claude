use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use regex::Regex;
use rusqlite::params;
use serde::Deserialize;

use super::{Fingerprint, Telemetry, TelemetryError};

pub struct EnrichContext<'a> {
    pub file_path: &'a str,
    pub chunk_index: usize,
    pub anchor_text: Option<&'a str>,
    pub error_kind: &'a str,
    pub current_fingerprint: Option<&'a Fingerprint>,
}

pub struct EnrichedError {
    pub base: String,
    pub hints: Vec<String>,
}

impl EnrichedError {
    pub fn into_message(self) -> String {
        if self.hints.is_empty() {
            self.base
        } else {
            format!("{}\n\n{}", self.base, self.hints.join("\n\n"))
        }
    }
}

pub fn enrich(tel: &Telemetry, base: String, ctx: &EnrichContext) -> EnrichedError {
    let mut hints = Vec::new();
    match stale_read_hint(tel, ctx) {
        Ok(Some(h)) => hints.push(h),
        Ok(None) => {}
        Err(e) => eprintln!(
            "apply-patch enrich (stale_read, chunk #{}): {e}",
            ctx.chunk_index
        ),
    }
    match anchor_reliability_hint(tel, ctx) {
        Ok(Some(h)) => hints.push(h),
        Ok(None) => {}
        Err(e) => eprintln!(
            "apply-patch enrich (anchor_reliability, chunk #{}): {e}",
            ctx.chunk_index
        ),
    }
    match transition_hint(tel, ctx) {
        Ok(Some(h)) => hints.push(h),
        Ok(None) => {}
        Err(e) => eprintln!(
            "apply-patch enrich (transition, chunk #{}): {e}",
            ctx.chunk_index
        ),
    }
    EnrichedError { base, hints }
}

fn stale_read_hint(tel: &Telemetry, ctx: &EnrichContext) -> Result<Option<String>, TelemetryError> {
    let Some(current) = ctx.current_fingerprint else {
        return Ok(None);
    };
    let Some(last) = tel.last_fingerprint(ctx.file_path)? else {
        return Ok(None);
    };
    if last.mtime_ns == current.mtime_ns && last.sha1 == current.sha1 {
        return Ok(None);
    }
    let old_short = &last.sha1[..last.sha1.len().min(8)];
    let new_short = &current.sha1[..current.sha1.len().min(8)];
    Ok(Some(format!(
        "this file's mtime changed from {} to {} since apply_patch last saw it (sha1 {old_short} \u{2192} {new_short}). Re-read the file and regenerate the patch.",
        last.mtime_ns, current.mtime_ns
    )))
}

fn anchor_reliability_hint(
    tel: &Telemetry,
    ctx: &EnrichContext,
) -> Result<Option<String>, TelemetryError> {
    let file = ctx.file_path;
    let anchor = ctx.anchor_text;
    let (n, ok): (i64, i64) = tel.with_conn(|conn| {
        conn.query_row(
            "SELECT COUNT(*) n, COALESCE(SUM(success), 0) ok
             FROM anchor_attempts
             WHERE file_path = ?1 AND anchor_text IS ?2",
            params![file, anchor],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
    })?;
    if n < 3 || ok * 2 >= n {
        return Ok(None);
    }

    let alternatives: Vec<(Option<String>, f64, i64)> = tel.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT anchor_text, SUM(success)*1.0/COUNT(*) AS rate, COUNT(*) AS n
             FROM anchor_attempts
             WHERE file_path = ?1 AND anchor_text IS NOT ?2
             GROUP BY anchor_text
             HAVING COUNT(*) >= 2
             ORDER BY rate DESC, n DESC
             LIMIT 3",
        )?;
        let rows = stmt
            .query_map(params![file, anchor], |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok::<_, rusqlite::Error>(rows)
    })?;

    let fails = n - ok;
    let anchor_repr = format_anchor(anchor);
    let mut out = format!("anchor {anchor_repr} has failed {fails}/{n} times on this file.");
    if !alternatives.is_empty() {
        out.push_str(" Alternatives that worked: ");
        let parts: Vec<String> = alternatives
            .iter()
            .map(|(a, rate, _)| {
                let pct = (rate * 100.0).round() as i64;
                format!("{} ({pct}% success)", format_anchor(a.as_deref()))
            })
            .collect();
        out.push_str(&parts.join(", "));
        out.push('.');
    }
    Ok(Some(out))
}

fn format_anchor(anchor: Option<&str>) -> String {
    match anchor {
        Some(a) => format!("`@@ {a}`"),
        None => "bare `@@`".to_string(),
    }
}

#[derive(Debug, Deserialize)]
struct FingerprintEntry {
    mtime_ns: i64,
}

fn transition_hint(tel: &Telemetry, ctx: &EnrichContext) -> Result<Option<String>, TelemetryError> {
    let cutoff = now_ms() - 90 * 86_400 * 1000;
    // Spec scope: "prior failures of this kind in this project" — project-wide,
    // not per-file. The per-file filter used to live here; the project boundary
    // is enforced at the DB level (one db per project).
    let rows: Vec<CallRow> = tel.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT session_id, ts, outcome, error_kind, patch_sha, files_json, fingerprints_json
             FROM calls
             WHERE ts >= ?1
             ORDER BY ts ASC, id ASC",
        )?;
        let rows = stmt
            .query_map(params![cutoff], |row| {
                Ok(CallRow {
                    session_id: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    ts: row.get(1)?,
                    outcome: row.get(2)?,
                    error_kind: row.get(3)?,
                    patch_sha: row.get(4)?,
                    files_json: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                    fingerprints_json: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok::<_, rusqlite::Error>(rows)
    })?;

    let pairs = pair_failures_with_successes(&rows, ctx.error_kind);

    if pairs.len() < 20 {
        return Ok(None);
    }

    // Load patch bodies for all involved shas in one query.
    let mut shas: Vec<String> = Vec::new();
    for (a, b) in &pairs {
        shas.push(a.patch_sha.clone());
        shas.push(b.patch_sha.clone());
    }
    shas.sort();
    shas.dedup();
    let bodies = load_bodies(tel, &shas)?;

    let mut counts: HashMap<&'static str, i64> = HashMap::new();
    for (fail, win) in &pairs {
        let fail_body = bodies
            .get(&fail.patch_sha)
            .map(String::as_str)
            .unwrap_or("");
        let win_body = bodies.get(&win.patch_sha).map(String::as_str).unwrap_or("");
        let cat = classify_fix(
            fail_body,
            win_body,
            ctx.file_path,
            &fail.fingerprints_json,
            &win.fingerprints_json,
        );
        *counts.entry(cat).or_insert(0) += 1;
    }

    let total: i64 = counts.values().sum();
    if total == 0 {
        return Ok(None);
    }
    let mut by_rank: Vec<(&&'static str, &i64)> = counts.iter().collect();
    by_rank.sort_by(|a, b| b.1.cmp(a.1));
    let top: Vec<String> = by_rank
        .iter()
        .take(4)
        .map(|(name, n)| {
            let pct = (**n as f64 / total as f64 * 100.0).round() as i64;
            format!("{} ({pct}%)", display_category(name))
        })
        .collect();
    Ok(Some(format!(
        "prior failures of this kind in this project were resolved by: {}",
        top.join(", ")
    )))
}

#[derive(Clone)]
struct CallRow {
    session_id: String,
    ts: i64,
    outcome: String,
    error_kind: Option<String>,
    patch_sha: String,
    files_json: String,
    fingerprints_json: String,
}

/// Pair each failure of `error_kind` with the IMMEDIATE next success in the
/// same session whose files overlap with the failure's files. A single fix
/// pairs with exactly one failure — the success is consumed. Rows are expected
/// pre-sorted by `ts` ASC.
fn pair_failures_with_successes(rows: &[CallRow], error_kind: &str) -> Vec<(CallRow, CallRow)> {
    let mut by_session: HashMap<String, Vec<&CallRow>> = HashMap::new();
    for r in rows {
        by_session.entry(r.session_id.clone()).or_default().push(r);
    }
    for calls in by_session.values_mut() {
        calls.sort_by_key(|c| c.ts);
    }
    let mut pairs: Vec<(CallRow, CallRow)> = Vec::new();
    for calls in by_session.values() {
        let mut consumed = vec![false; calls.len()];
        for i in 0..calls.len() {
            let cur = calls[i];
            if cur.error_kind.as_deref() != Some(error_kind) {
                continue;
            }
            let fail_paths = paths_from_files_json(&cur.files_json);
            // Walk forward, pair with the first un-consumed success whose
            // paths overlap with the failure's paths.
            for j in (i + 1)..calls.len() {
                if consumed[j] || calls[j].outcome != "success" {
                    continue;
                }
                let win_paths = paths_from_files_json(&calls[j].files_json);
                if paths_overlap(&fail_paths, &win_paths) {
                    pairs.push((cur.clone(), calls[j].clone()));
                    consumed[j] = true;
                    break;
                }
            }
        }
    }
    pairs
}

fn paths_from_files_json(files_json: &str) -> Vec<String> {
    if files_json.is_empty() {
        return Vec::new();
    }
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

fn load_bodies(
    tel: &Telemetry,
    shas: &[String],
) -> Result<HashMap<String, String>, TelemetryError> {
    if shas.is_empty() {
        return Ok(HashMap::new());
    }
    tel.with_conn(|conn| {
        let placeholders: String = (0..shas.len())
            .map(|i| format!("?{}", i + 1))
            .collect::<Vec<_>>()
            .join(",");
        let sql =
            format!("SELECT patch_sha, body FROM patch_bodies WHERE patch_sha IN ({placeholders})");
        let mut stmt = conn.prepare(&sql)?;
        let params_vec: Vec<&dyn rusqlite::ToSql> =
            shas.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        let rows = stmt
            .query_map(params_vec.as_slice(), |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok::<_, rusqlite::Error>(rows.into_iter().collect())
    })
    .map_err(TelemetryError::from)
}

fn classify_fix(
    fail_body: &str,
    win_body: &str,
    file_path: &str,
    fail_fp_json: &str,
    win_fp_json: &str,
) -> &'static str {
    let fail_anchor = first_anchor(fail_body, file_path);
    let win_anchor = first_anchor(win_body, file_path);
    if fail_anchor != win_anchor {
        return "anchor_changed";
    }

    if let (Some(f), Some(w)) = (
        lookup_mtime(fail_fp_json, file_path),
        lookup_mtime(win_fp_json, file_path),
    ) && f != w
    {
        return "re_read_signal";
    }

    let fail_ctx = context_lines(fail_body);
    let win_ctx = context_lines(win_body);
    let fail_trim: Vec<String> = fail_ctx.iter().map(|s| s.trim().to_string()).collect();
    let win_trim: Vec<String> = win_ctx.iter().map(|s| s.trim().to_string()).collect();
    if fail_trim == win_trim && fail_ctx != win_ctx {
        return "whitespace_adjusted";
    }
    if win_ctx.len() > fail_ctx.len() {
        return "context_widened";
    }
    if win_ctx.len() < fail_ctx.len() {
        return "context_narrowed";
    }
    "other"
}

/// First `@@ <anchor>` line that lives *inside* the `file_path` section of the
/// patch body. Scanning stops at the next `*** <verb> File:` marker so
/// multi-file patches classify each file independently. Returns `None` when
/// the file isn't present or the section only contains bare `@@`.
fn first_anchor(body: &str, file_path: &str) -> Option<String> {
    let section_re = Regex::new(r"(?m)^\*\*\* (?:Update|Add|Delete|Move) File: (.+)$").ok()?;
    let anchor_re = Regex::new(r"^@@ ([^\n]+)$").ok()?;
    for cap in section_re.captures_iter(body) {
        let header = cap.get(0)?;
        let path = cap.get(1)?.as_str().trim();
        if path != file_path {
            continue;
        }
        let start = header.end();
        let end = section_re
            .find_at(body, start)
            .map(|m| m.start())
            .unwrap_or(body.len());
        for line in body[start..end].lines() {
            if let Some(cap) = anchor_re.captures(line) {
                return cap.get(1).map(|m| m.as_str().to_string());
            }
        }
        return None;
    }
    None
}

fn context_lines(body: &str) -> Vec<String> {
    // Collect ` ` context lines inside hunks (not envelope sentinels).
    body.lines()
        .filter(|l| l.starts_with(' '))
        .map(|l| l[1..].to_string())
        .collect()
}

fn lookup_mtime(fp_json: &str, file_path: &str) -> Option<i64> {
    if fp_json.is_empty() {
        return None;
    }
    let map: HashMap<String, FingerprintEntry> = serde_json::from_str(fp_json).ok()?;
    map.get(file_path).map(|f| f.mtime_ns)
}

fn display_category(cat: &str) -> &str {
    match cat {
        "anchor_changed" => "anchor change",
        "re_read_signal" => "re-read + regenerate",
        "whitespace_adjusted" => "whitespace adjusted",
        "context_widened" => "context widened",
        "context_narrowed" => "context narrowed",
        _ => "other",
    }
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

    fn insert_attempts(tel: &Telemetry, call_id: i64, attempts: &[AnchorAttempt]) {
        tel.record_anchor_attempts(call_id, attempts).unwrap();
    }

    fn record_basic(tel: &Telemetry, outcome: &str, error_kind: Option<&str>) -> i64 {
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
            patch_sha: "sha".into(),
            fingerprints_json: "{}".into(),
        })
        .unwrap()
    }

    #[test]
    fn stale_read_hint_fires_on_mtime_change() {
        let t = Telemetry::open_in_memory().unwrap();
        t.upsert_fingerprint(
            "foo.rs",
            &Fingerprint {
                mtime_ns: 100,
                sha1: "aaaaaaaa11".into(),
                ts: 1,
            },
        )
        .unwrap();
        let current = Fingerprint {
            mtime_ns: 200,
            sha1: "bbbbbbbb22".into(),
            ts: 2,
        };
        let ctx = EnrichContext {
            file_path: "foo.rs",
            chunk_index: 0,
            anchor_text: None,
            error_kind: "context_not_found",
            current_fingerprint: Some(&current),
        };
        let got = stale_read_hint(&t, &ctx).unwrap().unwrap();
        assert!(got.contains("mtime changed"), "{got}");
        assert!(got.contains("aaaaaaaa"), "{got}");
        assert!(got.contains("bbbbbbbb"), "{got}");
    }

    #[test]
    fn stale_read_hint_silent_when_equal() {
        let t = Telemetry::open_in_memory().unwrap();
        t.upsert_fingerprint(
            "foo.rs",
            &Fingerprint {
                mtime_ns: 100,
                sha1: "aaa".into(),
                ts: 1,
            },
        )
        .unwrap();
        let current = Fingerprint {
            mtime_ns: 100,
            sha1: "aaa".into(),
            ts: 2,
        };
        let ctx = EnrichContext {
            file_path: "foo.rs",
            chunk_index: 0,
            anchor_text: None,
            error_kind: "context_not_found",
            current_fingerprint: Some(&current),
        };
        assert!(stale_read_hint(&t, &ctx).unwrap().is_none());
    }

    #[test]
    fn anchor_reliability_hint_fires_after_three_plus_failures() {
        let t = Telemetry::open_in_memory().unwrap();
        let cid = record_basic(&t, "error", Some("context_not_found"));
        // ("foo.rs", "fn bar(") — 1 success in 4 attempts (25%).
        insert_attempts(
            &t,
            cid,
            &[
                AnchorAttempt {
                    file_path: "foo.rs".into(),
                    chunk_index: 0,
                    anchor_text: Some("fn bar(".into()),
                    success: false,
                    fuzzy_tier: None,
                },
                AnchorAttempt {
                    file_path: "foo.rs".into(),
                    chunk_index: 0,
                    anchor_text: Some("fn bar(".into()),
                    success: false,
                    fuzzy_tier: None,
                },
                AnchorAttempt {
                    file_path: "foo.rs".into(),
                    chunk_index: 0,
                    anchor_text: Some("fn bar(".into()),
                    success: false,
                    fuzzy_tier: None,
                },
                AnchorAttempt {
                    file_path: "foo.rs".into(),
                    chunk_index: 0,
                    anchor_text: Some("fn bar(".into()),
                    success: true,
                    fuzzy_tier: None,
                },
                AnchorAttempt {
                    file_path: "foo.rs".into(),
                    chunk_index: 0,
                    anchor_text: Some("impl Baz".into()),
                    success: true,
                    fuzzy_tier: None,
                },
                AnchorAttempt {
                    file_path: "foo.rs".into(),
                    chunk_index: 0,
                    anchor_text: Some("impl Baz".into()),
                    success: true,
                    fuzzy_tier: None,
                },
            ],
        );

        let ctx = EnrichContext {
            file_path: "foo.rs",
            chunk_index: 0,
            anchor_text: Some("fn bar("),
            error_kind: "context_not_found",
            current_fingerprint: None,
        };
        let got = anchor_reliability_hint(&t, &ctx).unwrap().unwrap();
        assert!(got.contains("failed 3/4"), "hint was: {got}");
        assert!(got.contains("@@ impl Baz"), "hint was: {got}");
    }

    #[test]
    fn anchor_reliability_hint_silent_below_threshold() {
        let t = Telemetry::open_in_memory().unwrap();
        let cid = record_basic(&t, "error", Some("context_not_found"));
        insert_attempts(
            &t,
            cid,
            &[
                AnchorAttempt {
                    file_path: "foo.rs".into(),
                    chunk_index: 0,
                    anchor_text: Some("fn bar(".into()),
                    success: false,
                    fuzzy_tier: None,
                },
                AnchorAttempt {
                    file_path: "foo.rs".into(),
                    chunk_index: 0,
                    anchor_text: Some("fn bar(".into()),
                    success: false,
                    fuzzy_tier: None,
                },
            ],
        );
        let ctx = EnrichContext {
            file_path: "foo.rs",
            chunk_index: 0,
            anchor_text: Some("fn bar("),
            error_kind: "context_not_found",
            current_fingerprint: None,
        };
        assert!(anchor_reliability_hint(&t, &ctx).unwrap().is_none());
    }

    fn seed_transition_pair(
        tel: &Telemetry,
        session: &str,
        fail_anchor: &str,
        win_anchor: &str,
        fail_sha: &str,
        win_sha: &str,
    ) {
        // Record two calls directly via SQL so we can control session_id and
        // ts ordering precisely.
        tel.record_patch_body(fail_sha, &format!("*** Begin Patch\n*** Update File: foo.rs\n@@ {fail_anchor}\n- old\n+ new\n*** End Patch\n")).unwrap();
        tel.record_patch_body(win_sha, &format!("*** Begin Patch\n*** Update File: foo.rs\n@@ {win_anchor}\n- old\n+ new\n*** End Patch\n")).unwrap();

        let files_json =
            "[{\"path\":\"foo.rs\",\"chunk_count\":1,\"fuzzy_tier_used\":null,\"file_sha1\":null}]";
        let now = now_ms();
        tel.with_conn(|conn| {
            conn.execute(
                "INSERT INTO calls (ts, session_id, outcome, error_kind, files_json, duration_us, patch_sha, fingerprints_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![now, session, "error", "context_not_found", files_json, 0_i64, fail_sha, "{}"],
            ).unwrap();
            conn.execute(
                "INSERT INTO calls (ts, session_id, outcome, error_kind, files_json, duration_us, patch_sha, fingerprints_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![now + 1, session, "success", Option::<String>::None, files_json, 0_i64, win_sha, "{}"],
            ).unwrap();
        });
    }

    #[test]
    fn transition_hint_silent_below_twenty() {
        let t = Telemetry::open_in_memory().unwrap();
        for i in 0..5 {
            seed_transition_pair(
                &t,
                &format!("s{i}"),
                "old_anchor",
                "new_anchor",
                &format!("fail{i}"),
                &format!("win{i}"),
            );
        }
        let ctx = EnrichContext {
            file_path: "foo.rs",
            chunk_index: 0,
            anchor_text: None,
            error_kind: "context_not_found",
            current_fingerprint: None,
        };
        assert!(transition_hint(&t, &ctx).unwrap().is_none());
    }

    #[test]
    fn transition_hint_classifies_at_twenty() {
        let t = Telemetry::open_in_memory().unwrap();
        for i in 0..20 {
            seed_transition_pair(
                &t,
                &format!("s{i}"),
                &format!("old_{i}"),
                &format!("new_{i}"),
                &format!("fail{i}"),
                &format!("win{i}"),
            );
        }
        let ctx = EnrichContext {
            file_path: "foo.rs",
            chunk_index: 0,
            anchor_text: None,
            error_kind: "context_not_found",
            current_fingerprint: None,
        };
        let got = transition_hint(&t, &ctx).unwrap().unwrap();
        assert!(got.contains("anchor change"), "hint was: {got}");
        assert!(got.contains('%'), "hint was: {got}");
    }

    #[test]
    fn pair_walk_dedups_multiple_failures_against_one_success() {
        // Session order: F1 F2 F3 S1. A single fix shouldn't produce three
        // pairs — only the first (oldest) unpaired failure should claim the
        // success, matching the "immediate next" spec (one success consumed
        // per fix).
        let rows = vec![
            CallRow {
                session_id: "s".into(),
                ts: 1,
                outcome: "error".into(),
                error_kind: Some("context_not_found".into()),
                patch_sha: "f1".into(),
                files_json: r#"[{"path":"foo.rs"}]"#.into(),
                fingerprints_json: String::new(),
            },
            CallRow {
                session_id: "s".into(),
                ts: 2,
                outcome: "error".into(),
                error_kind: Some("context_not_found".into()),
                patch_sha: "f2".into(),
                files_json: r#"[{"path":"foo.rs"}]"#.into(),
                fingerprints_json: String::new(),
            },
            CallRow {
                session_id: "s".into(),
                ts: 3,
                outcome: "error".into(),
                error_kind: Some("context_not_found".into()),
                patch_sha: "f3".into(),
                files_json: r#"[{"path":"foo.rs"}]"#.into(),
                fingerprints_json: String::new(),
            },
            CallRow {
                session_id: "s".into(),
                ts: 4,
                outcome: "success".into(),
                error_kind: None,
                patch_sha: "s1".into(),
                files_json: r#"[{"path":"foo.rs"}]"#.into(),
                fingerprints_json: String::new(),
            },
        ];
        let pairs = pair_failures_with_successes(&rows, "context_not_found");
        assert_eq!(pairs.len(), 1, "expected 1 pair, got {}", pairs.len());
        assert_eq!(pairs[0].0.patch_sha, "f1");
        assert_eq!(pairs[0].1.patch_sha, "s1");
    }

    #[test]
    fn pair_walk_matches_cross_file_by_path_overlap() {
        // Failure patched foo.rs; success patched foo.rs + bar.rs. Paths
        // overlap on foo.rs so they pair despite files_json differing.
        let rows = vec![
            CallRow {
                session_id: "s".into(),
                ts: 1,
                outcome: "error".into(),
                error_kind: Some("context_not_found".into()),
                patch_sha: "f1".into(),
                files_json: r#"[{"path":"foo.rs"}]"#.into(),
                fingerprints_json: String::new(),
            },
            CallRow {
                session_id: "s".into(),
                ts: 2,
                outcome: "success".into(),
                error_kind: None,
                patch_sha: "s1".into(),
                files_json: r#"[{"path":"foo.rs"},{"path":"bar.rs"}]"#.into(),
                fingerprints_json: String::new(),
            },
        ];
        let pairs = pair_failures_with_successes(&rows, "context_not_found");
        assert_eq!(pairs.len(), 1);
    }

    #[test]
    fn first_anchor_scopes_to_file_section() {
        let body = "*** Begin Patch\n\
                    *** Update File: a.rs\n\
                    @@ impl Alpha\n\
                    - old\n\
                    + new\n\
                    *** Update File: b.rs\n\
                    @@ impl Beta\n\
                    - old\n\
                    + new\n\
                    *** End Patch\n";
        assert_eq!(first_anchor(body, "a.rs").as_deref(), Some("impl Alpha"));
        assert_eq!(first_anchor(body, "b.rs").as_deref(), Some("impl Beta"));
        assert!(first_anchor(body, "missing.rs").is_none());
    }

    #[test]
    fn enriched_error_into_message_no_hints() {
        let e = EnrichedError {
            base: "base".into(),
            hints: vec![],
        };
        assert_eq!(e.into_message(), "base");
    }

    #[test]
    fn enriched_error_into_message_with_hints() {
        let e = EnrichedError {
            base: "base".into(),
            hints: vec!["one".into(), "two".into()],
        };
        assert_eq!(e.into_message(), "base\n\none\n\ntwo");
    }
}
