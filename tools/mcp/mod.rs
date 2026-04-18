use std::path::PathBuf;

use rmcp::ServiceExt;
use rmcp::model::ErrorData;
use rmcp::transport::stdio;
use serde_json::json;

use crate::artifact::{self, ArtifactKind, CtError, ResolveError};

mod apply_patch;
mod blueprint;

// ---------------------------------------------------------------------------
// Shared error mapping + resolution helpers used by every sub-server.
// ---------------------------------------------------------------------------

/// Map a `CtError` to an MCP `ErrorData`.
pub(crate) fn ct_error_to_tool(err: CtError) -> ErrorData {
    match err {
        CtError::Resolve(ResolveError::NotFound(s)) => {
            ErrorData::invalid_params(format!("artifact not found: {s}"), None)
        }
        CtError::Resolve(ResolveError::Ambiguous(paths)) => {
            let candidates: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
            ErrorData::invalid_params(
                format!("ambiguous stem, matches: {}", candidates.join(", ")),
                Some(json!({ "candidates": candidates })),
            )
        }
        CtError::Validation(msg) => ErrorData::invalid_params(msg, None),
        CtError::Sync(e) => ErrorData::internal_error(e.to_string(), None),
        CtError::Io(e) => ErrorData::internal_error(e.to_string(), None),
    }
}

/// Wrap a serializable value as a successful tool result.
pub(crate) fn json_success<T: serde::Serialize>(
    value: &T,
) -> Result<rmcp::model::CallToolResult, ErrorData> {
    let v = serde_json::to_value(value)
        .map_err(|e| ErrorData::internal_error(format!("serialize: {e}"), None))?;
    Ok(rmcp::model::CallToolResult::structured(v))
}

/// Resolve a stem (optionally scoped to a kind) to a vault path.
pub(crate) fn resolve(stem: &str, kind: Option<ArtifactKind>) -> Result<PathBuf, ErrorData> {
    let result = match kind {
        Some(k) => artifact::resolve_artifact_path(stem, k),
        None => artifact::resolve_stem_universal(stem),
    };
    result.map_err(|e| ct_error_to_tool(CtError::from(e)))
}

/// Ensure the vault directory exists before a handler does real work. A missing
/// vault at request time used to call `fatal()` and exit the server process —
/// this turns it into a per-request validation error instead.
pub(crate) fn require_vault() -> Result<(), ErrorData> {
    artifact::blueprints_dir_checked()
        .map(|_| ())
        .map_err(ct_error_to_tool)
}

/// Turn a user-supplied project hint into a project name. Bare names (no path
/// separator) skip the git round-trip that `resolve_repo_root` would perform,
/// but must still be valid subdirectory names — a bare ".." would otherwise
/// crash the server downstream via `project_name`.
pub(crate) fn project_input_to_name(input: Option<String>) -> Result<String, ErrorData> {
    match input {
        Some(s) if !s.contains('/') && !s.contains('\\') => {
            artifact::validate_project_name(&s).map_err(ct_error_to_tool)?;
            Ok(s)
        }
        Some(path) => Ok(artifact::project_name(&artifact::resolve_repo_root(&path))),
        None => Ok(artifact::project_name(&artifact::current_project())),
    }
}

// ---------------------------------------------------------------------------
// Server entrypoints
// ---------------------------------------------------------------------------

/// Run the blueprint/vault MCP server over stdio.
pub fn run_blueprint_server() -> Result<(), Box<dyn std::error::Error>> {
    // One-shot vault health warning so operators see the problem at startup;
    // handlers still re-check per request via `require_vault()` so the server
    // fails each call cleanly instead of crashing once the dir goes missing.
    if let Err(e) = artifact::blueprints_dir_checked() {
        eprintln!("blueprint-mcp: warning — {e}");
    }
    serve_stdio(blueprint::BlueprintMcpServer::new())
}

/// Run the apply_patch MCP server over stdio.
pub fn run_apply_patch_server() -> Result<(), Box<dyn std::error::Error>> {
    serve_stdio(apply_patch::ApplyPatchMcpServer::new())
}

/// Drive a server struct through rmcp's stdio transport until shutdown.
fn serve_stdio<S>(server: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: rmcp::ServerHandler + Clone + Send + Sync + 'static,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let service = server.serve(stdio()).await?;
        service.waiting().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
