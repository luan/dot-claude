// Integration tests for the Phase 2 `apply()` return shape. These verify the
// public API of the library without touching the MCP layer or telemetry db —
// a successful apply must surface an anchor attempt + a file fingerprint, and
// a failed apply must still carry whatever was accumulated up to the error.

use std::fs;

use tempfile::TempDir;

use super::apply;
use super::telemetry::{AnchorAttempt, CallRecord, FileCallEntry, Telemetry, enrich, stats};

#[test]
fn success_returns_attempt_and_fingerprint() {
    let tmp = TempDir::new().unwrap();
    let content = "alpha\nbeta\ngamma\n";
    fs::write(tmp.path().join("f.txt"), content).unwrap();

    let patch = concat!(
        "*** Begin Patch\n",
        "*** Update File: f.txt\n",
        "@@\n",
        " alpha\n",
        "-beta\n",
        "+BETA\n",
        " gamma\n",
        "*** End Patch\n",
    );

    let outcome = apply(patch, tmp.path(), true).expect("apply should succeed");
    assert_eq!(outcome.changes.len(), 1);
    assert_eq!(outcome.attempts.len(), 1);
    assert!(outcome.attempts[0].success);
    assert_eq!(outcome.attempts[0].fuzzy_tier.as_deref(), Some("exact"));
    assert_eq!(outcome.attempts[0].file_path, "f.txt");

    assert_eq!(outcome.fingerprints.len(), 1);
    let (path, fp) = &outcome.fingerprints[0];
    assert_eq!(path, "f.txt");
    let expected_sha = crate::apply_patch::sha1_hex(content.as_bytes());
    assert_eq!(fp.sha1, expected_sha);
}

#[test]
fn failure_returns_attempt_and_fingerprint() {
    let tmp = TempDir::new().unwrap();
    let content = "one\ntwo\nthree\n";
    fs::write(tmp.path().join("g.txt"), content).unwrap();

    // `delta` doesn't exist in the file — this hunk will miss context.
    let patch = concat!(
        "*** Begin Patch\n",
        "*** Update File: g.txt\n",
        "@@\n",
        "-delta\n",
        "+DELTA\n",
        "*** End Patch\n",
    );

    let failure = apply(patch, tmp.path(), true).expect_err("apply should fail");
    assert_eq!(failure.attempts.len(), 1);
    assert!(!failure.attempts[0].success);
    assert_eq!(failure.attempts[0].file_path, "g.txt");

    // The file was read before the chunk failure, so fingerprint is recorded.
    assert_eq!(failure.fingerprints.len(), 1);
    let (path, fp) = &failure.fingerprints[0];
    assert_eq!(path, "g.txt");
    let expected_sha = crate::apply_patch::sha1_hex(content.as_bytes());
    assert_eq!(fp.sha1, expected_sha);
}

// Integration: drive enrich() through the Phase 3 surface. Seed history
// against an in-memory Telemetry, then run a real successful apply followed
// by a real failed apply — the failure's enrichment must pick up the anchor
// reliability signal we seeded.
#[test]
fn enrich_appends_anchor_reliability_hint() {
    let tmp = TempDir::new().unwrap();
    let file_path = "h.txt";
    fs::write(tmp.path().join(file_path), "line1\nline2\nline3\n").unwrap();

    let tel = Telemetry::open_in_memory().unwrap();

    // Seed a history where "fn foo" has failed 3/4 times on h.txt, and
    // "impl Bar" has succeeded twice — enough to trip the reliability hint.
    let call_id = tel
        .record_call(&CallRecord {
            outcome: "error".into(),
            error_kind: Some("context_not_found".into()),
            files: vec![FileCallEntry {
                path: file_path.into(),
                chunk_count: 1,
                fuzzy_tier_used: None,
                file_sha1: None,
            }],
            duration_us: 0,
            patch_sha: "seed".into(),
            fingerprints_json: "{}".into(),
        })
        .unwrap();
    let attempts: Vec<AnchorAttempt> = (0..4)
        .map(|i| AnchorAttempt {
            file_path: file_path.into(),
            chunk_index: 0,
            anchor_text: Some("fn foo".into()),
            success: i == 3,
            fuzzy_tier: None,
        })
        .chain((0..2).map(|_| AnchorAttempt {
            file_path: file_path.into(),
            chunk_index: 0,
            anchor_text: Some("impl Bar".into()),
            success: true,
            fuzzy_tier: None,
        }))
        .collect();
    tel.record_anchor_attempts(call_id, &attempts).unwrap();

    let ctx = enrich::EnrichContext {
        file_path,
        chunk_index: 0,
        anchor_text: Some("fn foo"),
        error_kind: "context_not_found",
        current_fingerprint: None,
    };
    let enriched = enrich::enrich(&tel, "base error".into(), &ctx);
    let msg = enriched.into_message();
    assert!(msg.starts_with("base error\n\n"), "msg was: {msg}");
    assert!(msg.contains("failed 3/4"), "msg was: {msg}");
    assert!(msg.contains("@@ impl Bar"), "msg was: {msg}");
}

// End-to-end observability: seed a mix of success/fail calls with different
// error_kinds, then exercise the stats::run report as a black box. Guards
// against report-rendering regressions the unit tests wouldn't catch.
#[test]
fn end_to_end_observability() {
    let tel = Telemetry::open_in_memory().unwrap();

    let mk = |outcome: &str, error_kind: Option<&str>, sha: &str| CallRecord {
        outcome: outcome.into(),
        error_kind: error_kind.map(str::to_string),
        files: vec![FileCallEntry {
            path: "e2e.rs".into(),
            chunk_count: 1,
            fuzzy_tier_used: None,
            file_sha1: None,
        }],
        duration_us: 0,
        patch_sha: sha.into(),
        fingerprints_json: "{}".into(),
    };

    tel.record_call(&mk("success", None, "s1")).unwrap();
    tel.record_call(&mk("success", None, "s2")).unwrap();
    let ctx_fail = tel
        .record_call(&mk("error", Some("context_not_found"), "e1"))
        .unwrap();
    tel.record_call(&mk("error", Some("context_not_found"), "e2"))
        .unwrap();
    tel.record_call(&mk("error", Some("ambiguous_context"), "e3"))
        .unwrap();

    // Seed enough anchor attempts on one anchor to make it top the list.
    let mut attempts = Vec::new();
    for i in 0..5 {
        attempts.push(AnchorAttempt {
            file_path: "e2e.rs".into(),
            chunk_index: 0,
            anchor_text: Some("fn wobble".into()),
            success: i == 4,
            fuzzy_tier: None,
        });
    }
    tel.record_anchor_attempts(ctx_fail, &attempts).unwrap();

    let report = stats::run(&tel, "e2e-project", 30).unwrap();
    assert!(report.contains("project: e2e-project"), "report: {report}");
    assert!(report.contains("calls: 5"), "report: {report}");
    assert!(report.contains("successes: 2 (40%)"), "report: {report}");
    assert!(report.contains("errors: 3"), "report: {report}");
    assert!(report.contains("context_not_found (2)"), "report: {report}");
    assert!(report.contains("ambiguous_context (1)"), "report: {report}");
    assert!(report.contains("@@ fn wobble"), "report: {report}");
    assert!(report.contains("4/5 fail"), "report: {report}");
    assert!(report.contains("below threshold"), "report: {report}");
}
