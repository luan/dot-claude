use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData};
use rmcp::schemars::{self, JsonSchema};
use rmcp::{ServerHandler, tool, tool_handler, tool_router};
use serde::Deserialize;

use crate::apply_patch::{
    self, AnchorAttempt, ApplyFailure, ApplyOutcome, ApplyPatchError, CallRecord, ChangeType,
    FileCallEntry, FileChange, Fingerprint, HunkFuzzy, HunkRegion, MAX_PATCH_SIZE_BYTES, Telemetry,
    enrich, sha1_hex,
};

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

/// Map an `ApplyPatchError` to an MCP `ErrorData`. Bad-input variants (parse,
/// path validation, context miss, state conflicts) surface as `invalid_params`;
/// `Io` is a server-side failure and becomes `internal_error`.
fn classify_error(err: &ApplyPatchError) -> &'static str {
    match err {
        ApplyPatchError::Parse(_) => "parse",
        ApplyPatchError::ContextNotFound { .. } => "context_not_found",
        ApplyPatchError::AmbiguousContext { .. } => "ambiguous_context",
        ApplyPatchError::AnchorShadowsFirstContext { .. } => "anchor_shadows",
        ApplyPatchError::DeleteIsDirectory(_) => "delete_is_directory",
        ApplyPatchError::AddTargetExists(_) => "add_target_exists",
        ApplyPatchError::DuplicateUpdate(_) => "duplicate_update",
        ApplyPatchError::MoveTargetExists(_) => "move_target_exists",
        ApplyPatchError::Io { .. } => "io",
    }
}

fn build_file_entries_from_changes(
    changes: &[FileChange],
    attempts: &[AnchorAttempt],
    fingerprints: &[(String, Fingerprint)],
) -> Vec<FileCallEntry> {
    changes
        .iter()
        .map(|c| {
            let chunk_count = attempts.iter().filter(|a| a.file_path == c.path).count();
            let fuzzy_tier_used = c.fuzzy_hunks.first().map(|h| h.tier.as_str().to_string());
            let file_sha1 = fingerprints
                .iter()
                .find(|(p, _)| p == &c.path)
                .map(|(_, fp)| fp.sha1.clone());
            FileCallEntry {
                path: c.path.clone(),
                chunk_count,
                fuzzy_tier_used,
                file_sha1,
            }
        })
        .collect()
}

fn build_file_entries_from_attempts(
    attempts: &[AnchorAttempt],
    fingerprints: &[(String, Fingerprint)],
) -> Vec<FileCallEntry> {
    let mut seen: Vec<String> = Vec::new();
    for a in attempts {
        if !seen.iter().any(|p| p == &a.file_path) {
            seen.push(a.file_path.clone());
        }
    }
    for (p, _) in fingerprints {
        if !seen.iter().any(|s| s == p) {
            seen.push(p.clone());
        }
    }
    seen.into_iter()
        .map(|path| {
            let chunk_count = attempts.iter().filter(|a| a.file_path == path).count();
            let file_sha1 = fingerprints
                .iter()
                .find(|(p, _)| p == &path)
                .map(|(_, fp)| fp.sha1.clone());
            FileCallEntry {
                path,
                chunk_count,
                fuzzy_tier_used: None,
                file_sha1,
            }
        })
        .collect()
}

fn fingerprints_to_json(fps: &[(String, Fingerprint)]) -> String {
    let map: serde_json::Map<String, serde_json::Value> = fps
        .iter()
        .map(|(p, fp)| {
            (
                p.clone(),
                serde_json::json!({ "mtime_ns": fp.mtime_ns, "sha1": fp.sha1 }),
            )
        })
        .collect();
    serde_json::to_string(&serde_json::Value::Object(map)).unwrap_or_else(|_| "{}".to_string())
}

fn record_success(
    tel: &Telemetry,
    outcome: &ApplyOutcome,
    duration_us: u64,
    patch_sha: &str,
    patch_body: &str,
) {
    if let Err(e) = tel.record_patch_body(patch_sha, patch_body) {
        eprintln!("apply-patch telemetry: {e}");
    }
    let files =
        build_file_entries_from_changes(&outcome.changes, &outcome.attempts, &outcome.fingerprints);
    let record = CallRecord {
        outcome: "success".into(),
        error_kind: None,
        files,
        duration_us,
        patch_sha: patch_sha.to_string(),
        fingerprints_json: fingerprints_to_json(&outcome.fingerprints),
    };
    match tel.record_call(&record) {
        Ok(call_id) => {
            if let Err(e) = tel.record_anchor_attempts(call_id, &outcome.attempts) {
                eprintln!("apply-patch telemetry: {e}");
            }
            for (path, fp) in &outcome.fingerprints {
                if let Err(e) = tel.upsert_fingerprint(path, fp) {
                    eprintln!("apply-patch telemetry: {e}");
                }
            }
        }
        Err(e) => eprintln!("apply-patch telemetry: {e}"),
    }
}

/// Record the failure call + anchor attempts + patch body. Does NOT upsert
/// fingerprints — enrichment needs to read the previous-generation fingerprint
/// first (the stale-read hint compares last-seen against the *current* call's
/// fingerprint). Pair with `record_failure_fingerprints` after enrichment.
fn record_failure_call(
    tel: &Telemetry,
    failure: &ApplyFailure,
    duration_us: u64,
    patch_sha: &str,
    patch_body: &str,
) {
    if let Err(e) = tel.record_patch_body(patch_sha, patch_body) {
        eprintln!("apply-patch telemetry: {e}");
    }
    let error_kind = classify_error(&failure.error).to_string();
    let files = build_file_entries_from_attempts(&failure.attempts, &failure.fingerprints);
    let record = CallRecord {
        outcome: error_kind.clone(),
        error_kind: Some(error_kind),
        files,
        duration_us,
        patch_sha: patch_sha.to_string(),
        fingerprints_json: fingerprints_to_json(&failure.fingerprints),
    };
    match tel.record_call(&record) {
        Ok(call_id) => {
            if let Err(e) = tel.record_anchor_attempts(call_id, &failure.attempts) {
                eprintln!("apply-patch telemetry: {e}");
            }
        }
        Err(e) => eprintln!("apply-patch telemetry: {e}"),
    }
}

fn record_failure_fingerprints(tel: &Telemetry, fingerprints: &[(String, Fingerprint)]) {
    for (path, fp) in fingerprints {
        if let Err(e) = tel.upsert_fingerprint(path, fp) {
            eprintln!("apply-patch telemetry: {e}");
        }
    }
}

/// Enrich the failure message using telemetry, then record the call. Returns
/// the enriched error ready to hand back to the MCP caller. Split from the
/// handler so tests can exercise the ordering: enrichment reads
/// `last_fingerprint` BEFORE the call's fingerprints are upserted, so a stale
/// read on disk compares against the previous generation's fingerprint.
fn enrich_and_record_failure(
    tel: &Telemetry,
    failure: &ApplyFailure,
    duration_us: u64,
    patch_sha: &str,
    patch_body: &str,
) -> Option<String> {
    let enriched = enrichable_context(&failure.error).map(|(path, chunk, anchor)| {
        let current_fp = failure
            .fingerprints
            .iter()
            .find(|(p, _)| p == path)
            .map(|(_, fp)| fp);
        let ctx = enrich::EnrichContext {
            file_path: path,
            chunk_index: chunk,
            anchor_text: anchor,
            error_kind: classify_error(&failure.error),
            current_fingerprint: current_fp,
        };
        enrich::enrich(tel, failure.error.to_string(), &ctx).into_message()
    });
    record_failure_call(tel, failure, duration_us, patch_sha, patch_body);
    record_failure_fingerprints(tel, &failure.fingerprints);
    enriched
}

fn apply_patch_error_to_tool(err: ApplyPatchError) -> ErrorData {
    enriched_error_to_tool(&err, err.to_string())
}

/// Map an `ApplyPatchError` to an `ErrorData` using a pre-built message.
/// Mirrors the variant→severity decision of `apply_patch_error_to_tool` but
/// lets the caller swap the displayed text (for example, after enrichment).
fn enriched_error_to_tool(err: &ApplyPatchError, message: String) -> ErrorData {
    match err {
        ApplyPatchError::Parse(_)
        | ApplyPatchError::DeleteIsDirectory(_)
        | ApplyPatchError::AddTargetExists(_)
        | ApplyPatchError::DuplicateUpdate(_)
        | ApplyPatchError::MoveTargetExists(_)
        | ApplyPatchError::ContextNotFound { .. }
        | ApplyPatchError::AmbiguousContext { .. }
        | ApplyPatchError::AnchorShadowsFirstContext { .. } => {
            ErrorData::invalid_params(message, None)
        }
        ApplyPatchError::Io { .. } => ErrorData::internal_error(message, None),
    }
}

/// Pull the (file_path, chunk_index, anchor) enrichment context out of the
/// subset of errors that reference a specific file/chunk. Returns `None` for
/// errors that don't carry that context (parse errors, duplicate updates, io).
fn enrichable_context(err: &ApplyPatchError) -> Option<(&str, usize, Option<&str>)> {
    match err {
        ApplyPatchError::ContextNotFound {
            path,
            chunk,
            change_context,
            ..
        } => Some((path.as_str(), *chunk, change_context.as_deref())),
        ApplyPatchError::AmbiguousContext {
            path,
            chunk,
            change_context,
            ..
        } => Some((path.as_str(), *chunk, change_context.as_deref())),
        ApplyPatchError::AnchorShadowsFirstContext {
            path,
            chunk,
            anchor,
        } => Some((path.as_str(), *chunk, Some(anchor.as_str()))),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Output shape
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
struct FileChangeOut<'a> {
    path: &'a str,
    #[serde(rename = "type")]
    kind: &'static str,
    additions: usize,
    deletions: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    move_path: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fuzzy_hunks: Vec<HunkFuzzy>,
    /// Post-apply snapshot of ~10 lines around each hunk. Empty for Add/Delete
    /// and for any hunk whose change was purely additive at end-of-file.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    post_apply_regions: Vec<HunkRegion>,
    /// Only present on dry_run or when a chunk needed fuzzy matching — skip
    /// the echo on routine exact-match applies to keep responses small.
    #[serde(skip_serializing_if = "Option::is_none")]
    unified_diff: Option<&'a str>,
}

fn kind_str(k: ChangeType) -> &'static str {
    match k {
        ChangeType::Add => "add",
        ChangeType::Update => "update",
        ChangeType::Delete => "delete",
        ChangeType::Move => "move",
    }
}

fn to_out(c: &FileChange, dry_run: bool) -> FileChangeOut<'_> {
    FileChangeOut {
        path: &c.path,
        kind: kind_str(c.kind),
        additions: c.additions,
        deletions: c.deletions,
        move_path: c.move_path.as_deref(),
        fuzzy_hunks: c.fuzzy_hunks.clone(),
        post_apply_regions: c.post_apply_regions.clone(),
        unified_diff: if dry_run || !c.fuzzy_hunks.is_empty() {
            Some(c.unified_diff.as_str())
        } else {
            None
        },
    }
}

// ---------------------------------------------------------------------------
// Input schema
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ApplyPatchIn {
    #[schemars(description = "Patch body in the apply_patch envelope format")]
    patch: String,
    #[schemars(
        description = "Working directory (absolute path). Defaults to the server's process cwd"
    )]
    cwd: Option<String>,
    #[schemars(description = "If true, parse + plan but do not write to disk")]
    dry_run: Option<bool>,
}

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub(super) struct ApplyPatchMcpServer {
    tool_router: ToolRouter<Self>,
    telemetry: Arc<Mutex<HashMap<String, Arc<Telemetry>>>>,
}

impl ApplyPatchMcpServer {
    pub(super) fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            telemetry: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Return the cached `Telemetry` handle for `cwd`'s project, opening one
    /// on demand. Errors are swallowed — telemetry is best-effort observability
    /// and must never break a patch apply.
    fn telemetry_for(&self, cwd: &Path) -> Option<Arc<Telemetry>> {
        let project_name = match crate::mcp::project_input_to_name(Some(cwd.display().to_string()))
        {
            Ok(name) => name,
            Err(e) => {
                eprintln!("apply-patch telemetry: project resolve: {}", e.message);
                return None;
            }
        };
        let mut guard = match self.telemetry.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(existing) = guard.get(&project_name) {
            return Some(Arc::clone(existing));
        }
        match Telemetry::open(&project_name) {
            Ok(t) => {
                let handle = Arc::new(t);
                guard.insert(project_name, Arc::clone(&handle));
                Some(handle)
            }
            Err(e) => {
                eprintln!("apply-patch telemetry: open: {e}");
                None
            }
        }
    }
}

#[tool_router(router = tool_router)]
impl ApplyPatchMcpServer {
    #[tool(
        name = "apply_patch",
        description = r#"Apply a patch (envelope format) to files. The primary file-edit tool — prefer it over Edit/Write for every change, single-file or multi-file. A single envelope can add, update, delete, or move many files at once. No line numbers: `@@` is a section marker, not a position.

Envelope shape:

*** Begin Patch
[ one or more file sections ]
*** End Patch

Each file section starts with one of three headers:

*** Add File: <path>      — create a new file. Every following line is a + line (the initial contents).
*** Delete File: <path>   — remove an existing file. Nothing follows.
*** Update File: <path>   — patch an existing file in place (optionally with a rename).

`*** Update File:` may be immediately followed by `*** Move to: <new path>` to rename. Then one or more hunks, each introduced by `@@` (bare, or followed by an anchor line). Within a hunk each line is prefixed with ` ` (context), `-` (removed), or `+` (added).

Anchors — the anchor line is *consumed* and the pattern search resumes on the line *after* it. So:
- Don't repeat the anchor text as your first ` ` context line. That's the most common shape error and the tool rejects it explicitly.
- Pick something structurally unique *near* the change (an import line above the function, a `struct` or `impl` header, a doc comment), not the function signature you're editing. Signatures collide when several functions share a similar shape.
- Bare `@@` is fine — use it whenever the surrounding context already pins the hunk uniquely.

Context: include *as little as makes the hunk unique* — often 1-2 lines, and zero when the `-` line alone is distinctive (a unique string, a fully qualified name, a rare function signature). Generic lines (`    return;`, `}`, bare `None`) almost always need at least one neighbor for disambiguation. If the hunk matches multiple locations the patch is rejected with every candidate line number; add one line above or below until only one remains, or pin with a `@@ <unique line>` anchor. Wider context isn't safer — it just gives concurrent edits more surface to drift against.

Example combining all operations:

*** Begin Patch
*** Add File: hello.txt
+Hello world
*** Update File: src/app.py
*** Move to: src/main.py
@@ def greet():
-    print("Hi")
+    print("Hello, world!")
*** Delete File: obsolete.txt
*** End Patch

Rules:
- Paths can be relative (joined with cwd) or absolute.
- Include a header (Add/Delete/Update) for each file section. New-file lines are prefixed with `+`.
- Don't issue two `*** Update File:` sections for the same path — combine into one section with multiple hunks.
- To atomically replace a file, put a `*** Delete File:` before the `*** Add File:` (or `*** Move to:`) for the same path in the same envelope.

On context mismatch the error returns a numbered window of the current file state, so you can regenerate the patch without having to re-read the file.

On success each updated file echoes `post_apply_regions` — a small numbered window of the file as it will exist after the patch lands, around each hunk — so you can plan follow-up edits without re-reading.

Set `dry_run` to true to preview the unified diff without writing."#
    )]
    async fn apply_patch(
        &self,
        Parameters(input): Parameters<ApplyPatchIn>,
    ) -> Result<CallToolResult, ErrorData> {
        if input.patch.len() > MAX_PATCH_SIZE_BYTES {
            return Err(ErrorData::invalid_params(
                format!("patch exceeds {MAX_PATCH_SIZE_BYTES} byte limit"),
                None,
            ));
        }
        let cwd = match input.cwd {
            Some(s) => std::path::PathBuf::from(s),
            None => std::env::current_dir()
                .map_err(|e| ErrorData::internal_error(format!("cwd: {e}"), None))?,
        };
        if !cwd.is_dir() {
            return Err(ErrorData::invalid_params(
                format!("cwd is not a directory: {}", cwd.display()),
                None,
            ));
        }
        let dry_run = input.dry_run.unwrap_or(false);
        let tel = self.telemetry_for(&cwd);
        let start = Instant::now();
        let patch_sha = sha1_hex(input.patch.as_bytes());

        match apply_patch::apply(&input.patch, &cwd, dry_run) {
            Ok(outcome) => {
                if let Some(tel) = tel.as_ref() {
                    record_success(
                        tel,
                        &outcome,
                        start.elapsed().as_micros() as u64,
                        &patch_sha,
                        &input.patch,
                    );
                }
                let files: Vec<FileChangeOut> =
                    outcome.changes.iter().map(|c| to_out(c, dry_run)).collect();
                super::json_success(&serde_json::json!({
                    "dry_run": dry_run,
                    "files": files,
                }))
            }
            Err(failure) => {
                let duration_us = start.elapsed().as_micros() as u64;
                let enriched_message = tel.as_ref().and_then(|tel| {
                    enrich_and_record_failure(tel, &failure, duration_us, &patch_sha, &input.patch)
                });
                let ApplyFailure { error, .. } = *failure;
                let err_data = match enriched_message {
                    Some(message) => enriched_error_to_tool(&error, message),
                    None => apply_patch_error_to_tool(error),
                };
                Err(err_data)
            }
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ApplyPatchMcpServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo::new(
            rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info(rmcp::model::Implementation::new(
            "apply-patch",
            env!("CARGO_PKG_VERSION"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: enrichment must read `last_fingerprint` BEFORE the current
    /// call upserts it. Previously `record_failure` upserted first, which
    /// overwrote the previous generation's fingerprint, so the stale-read
    /// comparison always saw equal mtimes and the hint never fired.
    #[test]
    fn stale_read_hint_fires_through_failure_recording_path() {
        let tel = Telemetry::open_in_memory().unwrap();
        // Seed the previous generation: file was last seen at mtime=100.
        tel.upsert_fingerprint(
            "foo.rs",
            &Fingerprint {
                mtime_ns: 100,
                sha1: "aaaaaaaa11".into(),
                ts: 1,
            },
        )
        .unwrap();

        // Current call sees mtime=200 (file changed on disk) and fails with a
        // context-not-found error.
        let failure = ApplyFailure {
            error: ApplyPatchError::ContextNotFound {
                path: "foo.rs".into(),
                chunk: 0,
                change_context: None,
                first_old_line: None,
                near_miss: None,
            },
            attempts: Vec::new(),
            fingerprints: vec![(
                "foo.rs".into(),
                Fingerprint {
                    mtime_ns: 200,
                    sha1: "bbbbbbbb22".into(),
                    ts: 2,
                },
            )],
        };

        let message = enrich_and_record_failure(&tel, &failure, 0, "patchsha", "body")
            .expect("enrichable context produces a message");
        assert!(message.contains("mtime changed"), "message was: {message}");
        assert!(message.contains("aaaaaaaa"), "message was: {message}");
        assert!(message.contains("bbbbbbbb"), "message was: {message}");

        // Post-condition: the upsert still lands, so the NEXT call sees the
        // current fingerprint as the "previous" one.
        let stored = tel.last_fingerprint("foo.rs").unwrap().unwrap();
        assert_eq!(stored.mtime_ns, 200);
        assert_eq!(stored.sha1, "bbbbbbbb22");
    }
}
