use std::path::{Path, PathBuf};

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData};
use rmcp::schemars::{self, JsonSchema};
use rmcp::transport::stdio;
use rmcp::{ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::apply_patch::{ApplyPatchError, MAX_PATCH_SIZE_BYTES};
use crate::artifact::{self, ArtifactKind, CreateOpts, CtError, ResolveError};
use crate::vault::{self, SearchFilters};

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

/// Map a `CtError` to an MCP `ErrorData`. Push failures still include the
/// scaffolded path so callers can recover.
fn ct_error_to_tool(err: CtError) -> ErrorData {
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

/// Map an `ApplyPatchError` to an MCP `ErrorData`. Bad-input variants (parse,
/// path validation, context miss, state conflicts) surface as `invalid_params`;
/// `Io` is a server-side failure and becomes `internal_error`.
fn apply_patch_error_to_tool(err: ApplyPatchError) -> ErrorData {
    match err {
        ApplyPatchError::Parse(_)
        | ApplyPatchError::PathEscape(_)
        | ApplyPatchError::AbsolutePath(_)
        | ApplyPatchError::DeleteIsDirectory(_)
        | ApplyPatchError::AddTargetExists(_)
        | ApplyPatchError::DuplicateUpdate(_)
        | ApplyPatchError::MoveTargetExists(_)
        | ApplyPatchError::ContextNotFound { .. } => {
            ErrorData::invalid_params(err.to_string(), None)
        }
        ApplyPatchError::Io { .. } => ErrorData::internal_error(err.to_string(), None),
    }
}

/// Wrap a serializable value as a successful tool result.
fn json_success<T: serde::Serialize>(value: &T) -> Result<CallToolResult, ErrorData> {
    let v = serde_json::to_value(value)
        .map_err(|e| ErrorData::internal_error(format!("serialize: {e}"), None))?;
    Ok(CallToolResult::structured(v))
}

/// Resolve a stem (optionally scoped to a kind) to a vault path.
fn resolve(stem: &str, kind: Option<ArtifactKind>) -> Result<PathBuf, ErrorData> {
    let result = match kind {
        Some(k) => artifact::resolve_artifact_path(stem, k),
        None => artifact::resolve_stem_universal(stem),
    };
    result.map_err(|e| ct_error_to_tool(CtError::from(e)))
}

/// Ensure the vault directory exists before a handler does real work. A missing
/// vault at request time used to call `fatal()` and exit the server process —
/// this turns it into a per-request validation error instead.
fn require_vault() -> Result<(), ErrorData> {
    artifact::blueprints_dir_checked()
        .map(|_| ())
        .map_err(ct_error_to_tool)
}

/// Turn a user-supplied project hint into a project name. Bare names (no path
/// separator) skip the git round-trip that `resolve_repo_root` would perform.
fn project_input_to_name(input: Option<String>) -> String {
    match input {
        Some(s) if !s.contains('/') && !s.contains('\\') => s,
        Some(path) => artifact::project_name(&artifact::resolve_repo_root(&path)),
        None => artifact::project_name(&artifact::current_project()),
    }
}

// ---------------------------------------------------------------------------
// Input schemas
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ArtifactCreateIn {
    #[schemars(description = "Artifact kind: spec, plan, review, report, or doc")]
    kind: ArtifactKind,
    #[schemars(description = "Short human-readable topic; drives the slug when slug is omitted")]
    topic: String,
    #[schemars(description = "Project path or name; defaults to the current project")]
    project: Option<String>,
    #[schemars(description = "Override the auto-derived slug")]
    slug: Option<String>,
    #[schemars(description = "Stem of a parent artifact to wiki-link from frontmatter")]
    source: Option<String>,
    #[schemars(description = "Additional tags appended to auto-derived type/ and project/ tags")]
    tags: Option<Vec<String>>,
    #[schemars(
        description = "If true, route the artifact to the project's dive/ subfolder (spec only, requires source)"
    )]
    dive: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ArtifactReadIn {
    #[schemars(description = "Filename stem, vault-relative path, or absolute path")]
    stem: String,
    #[schemars(
        description = "If provided, restricts resolution to this kind; otherwise universal"
    )]
    kind: Option<ArtifactKind>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ArtifactListIn {
    #[schemars(description = "Artifact kind to list")]
    kind: ArtifactKind,
    #[schemars(description = "Project path or name; defaults to the current project")]
    project: Option<String>,
    #[schemars(description = "If true, list artifacts across all projects (default false)")]
    all: Option<bool>,
    #[schemars(description = "If true, list archived artifacts instead of active ones")]
    archived: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ArtifactLatestIn {
    #[schemars(description = "Artifact kind to look up")]
    kind: ArtifactKind,
    #[schemars(description = "Project path or name; defaults to the current project")]
    project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ArtifactArchiveIn {
    #[schemars(description = "Filename stem, vault-relative path, or absolute path")]
    stem: String,
    #[schemars(description = "If provided, restricts resolution to this kind")]
    kind: Option<ArtifactKind>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ArtifactCommitEditsIn {
    #[schemars(description = "Absolute or vault-relative path to the edited file")]
    path: String,
    #[schemars(
        description = "Custom commit message; defaults to '<kind>(<project>): edit <slug>'"
    )]
    message: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct VaultSearchIn {
    #[schemars(description = "Free-text query passed to the Obsidian search CLI")]
    query: String,
    #[schemars(description = "Optional kind filter")]
    kind: Option<ArtifactKind>,
    #[schemars(description = "Optional project filter (path or name)")]
    project: Option<String>,
    #[schemars(description = "If true, include archived artifacts in results")]
    archived: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct VaultRelatedIn {
    #[schemars(description = "Topic string whose keywords are matched against artifact slugs")]
    topic: String,
    #[schemars(description = "Project path or name; defaults to the current project")]
    project: Option<String>,
    #[schemars(description = "If true, also scan archived artifacts")]
    include_archive: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct VaultCheckIn {
    #[schemars(description = "If true, include wiki-links found under archive/ in the report")]
    include_archive: Option<bool>,
}

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
struct CtMcpServer {
    tool_router: ToolRouter<Self>,
}

impl CtMcpServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router(router = tool_router)]
impl CtMcpServer {
    #[tool(
        name = "blueprint_create",
        description = "Create a new blueprint (spec/plan/review/report/doc) in the blueprints vault. \
                       Scaffolds frontmatter only; the caller fills in the body via file edits."
    )]
    async fn blueprint_create(
        &self,
        Parameters(input): Parameters<ArtifactCreateIn>,
    ) -> Result<CallToolResult, ErrorData> {
        require_vault()?;
        let project = input.project.unwrap_or_else(artifact::current_project);
        let tags: Vec<String> = input.tags.unwrap_or_default();
        let outcome = artifact::create(CreateOpts {
            kind: input.kind,
            topic: &input.topic,
            project: &project,
            slug_override: input.slug.as_deref(),
            source: input.source.as_deref(),
            user_tags: &tags,
            dive: input.dive.unwrap_or(false),
        })
        .map_err(ct_error_to_tool)?;
        json_success(&outcome)
    }

    #[tool(
        name = "blueprint_read",
        description = "Read a blueprint by stem (or path) and return parsed frontmatter, body, \
                       and inline HTML comments."
    )]
    async fn blueprint_read(
        &self,
        Parameters(input): Parameters<ArtifactReadIn>,
    ) -> Result<CallToolResult, ErrorData> {
        require_vault()?;
        let path = resolve(&input.stem, input.kind)?;
        let outcome = artifact::read(&path).map_err(ct_error_to_tool)?;
        json_success(&outcome)
    }

    #[tool(
        name = "blueprint_list",
        description = "List blueprints of a given kind. Defaults to the current project; set all=true \
                       for all projects or archived=true for the archive."
    )]
    async fn blueprint_list(
        &self,
        Parameters(input): Parameters<ArtifactListIn>,
    ) -> Result<CallToolResult, ErrorData> {
        require_vault()?;
        let kind = input.kind;
        let archived = input.archived.unwrap_or(false);
        let all = input.all.unwrap_or(false);

        let items = if all {
            if archived {
                artifact::list_archived_artifacts(kind)
            } else {
                artifact::list_artifacts(kind, false)
            }
        } else {
            let proj_name = project_input_to_name(input.project);
            if archived {
                artifact::list_archived_artifacts_for_project(kind, &proj_name)
            } else {
                artifact::list_artifacts_for_project(kind, false, &proj_name)
            }
        };

        json_success(&json!({ "artifacts": items }))
    }

    #[tool(
        name = "blueprint_latest",
        description = "Return the most recently modified blueprint of a given kind in a project, \
                       or null when none exists."
    )]
    async fn blueprint_latest(
        &self,
        Parameters(input): Parameters<ArtifactLatestIn>,
    ) -> Result<CallToolResult, ErrorData> {
        require_vault()?;
        let proj_name = project_input_to_name(input.project);
        // Already sorted by mod_time desc — first match is latest.
        let latest = artifact::list_artifacts_for_project(input.kind, false, &proj_name)
            .into_iter()
            .next();
        match latest {
            Some(a) => json_success(&a),
            None => json_success(&Value::Null),
        }
    }

    #[tool(
        name = "blueprint_archive",
        description = "Archive a blueprint: store its content in a git note and move the file under \
                       the project's archive/ directory. Commits and pushes the change."
    )]
    async fn blueprint_archive(
        &self,
        Parameters(input): Parameters<ArtifactArchiveIn>,
    ) -> Result<CallToolResult, ErrorData> {
        require_vault()?;
        let path = resolve(&input.stem, input.kind)?;
        let kind: ArtifactKind = match input.kind {
            Some(k) => k,
            None => infer_kind_from_path(&path).ok_or_else(|| {
                ErrorData::invalid_params(
                    format!("cannot infer kind from path: {}", path.display()),
                    None,
                )
            })?,
        };
        let outcome = artifact::archive(kind, &path).map_err(ct_error_to_tool)?;
        json_success(&outcome)
    }

    #[tool(
        name = "blueprint_commit",
        description = "Commit and push edits made to an existing blueprint file. Use after writing to \
                       a path returned from blueprint_create or blueprint_read."
    )]
    async fn blueprint_commit(
        &self,
        Parameters(input): Parameters<ArtifactCommitEditsIn>,
    ) -> Result<CallToolResult, ErrorData> {
        let bp = artifact::blueprints_dir_checked().map_err(ct_error_to_tool)?;
        let path = Path::new(&input.path);
        let full_path: PathBuf = if path.is_absolute() {
            path.to_path_buf()
        } else {
            bp.join(path)
        };

        // Canonicalize both paths before comparing: textual strip_prefix lets
        // `bp.join("../../etc/hosts")` pass even though the real target is
        // outside the vault. Canonicalization resolves `..` and symlinks.
        let full_canonical = full_path.canonicalize().map_err(|e| {
            ErrorData::invalid_params(format!("cannot resolve {}: {e}", full_path.display()), None)
        })?;
        let bp_canonical = bp.canonicalize().map_err(|e| {
            ErrorData::internal_error(format!("cannot resolve vault {}: {e}", bp.display()), None)
        })?;
        if !full_canonical.starts_with(&bp_canonical) {
            return Err(ErrorData::invalid_params(
                format!("file is not inside {}", bp_canonical.display()),
                None,
            ));
        }
        // Refuse git-internal paths and non-markdown files — a stray `.git/`
        // component would let a caller commit git config, and only `.md`
        // artifacts belong in the edit flow.
        if full_canonical.components().any(|c| c.as_os_str() == ".git") {
            return Err(ErrorData::invalid_params(
                "path contains a .git component".to_string(),
                None,
            ));
        }
        if full_canonical.extension().and_then(|e| e.to_str()) != Some("md") {
            return Err(ErrorData::invalid_params(
                "only .md artifacts can be committed through this tool".to_string(),
                None,
            ));
        }

        let rel_path = full_canonical
            .strip_prefix(&bp_canonical)
            .expect("starts_with verified above")
            .to_path_buf();

        let message = input
            .message
            .unwrap_or_else(|| default_edit_message(&rel_path));

        // Short-circuit when the file has no pending changes. This avoids the
        // network round-trip of `git push` just to hit "nothing to commit".
        if no_pending_changes(&bp_canonical, &rel_path) {
            return json_success(&json!({
                "committed": false,
                "pushed": false,
                "message": "nothing to commit",
            }));
        }

        artifact::commit_and_push(&rel_path, &message)
            .map_err(|e| ct_error_to_tool(CtError::from(e)))?;

        json_success(&json!({
            "committed": true,
            "pushed": true,
            "message": message,
        }))
    }

    #[tool(
        name = "vault_search",
        description = "Search the vault via the Obsidian CLI with optional kind/project filters."
    )]
    async fn vault_search(
        &self,
        Parameters(input): Parameters<VaultSearchIn>,
    ) -> Result<CallToolResult, ErrorData> {
        require_vault()?;
        let filters = SearchFilters {
            kind: input.kind,
            project: input.project,
            archived: input.archived.unwrap_or(false),
        };
        let hits = vault::search(&input.query, filters).map_err(ct_error_to_tool)?;
        json_success(&json!({ "hits": hits }))
    }

    #[tool(
        name = "vault_related",
        description = "Find artifacts whose slugs overlap with the given topic keywords (2+ word \
                       overlap, or 1+ for short topics)."
    )]
    async fn vault_related(
        &self,
        Parameters(input): Parameters<VaultRelatedIn>,
    ) -> Result<CallToolResult, ErrorData> {
        require_vault()?;
        let hits = vault::related(
            &input.topic,
            input.project.as_deref(),
            input.include_archive.unwrap_or(false),
        )
        .map_err(ct_error_to_tool)?;
        json_success(&json!({ "hits": hits }))
    }

    #[tool(
        name = "vault_check",
        description = "Report unresolved Obsidian wiki-links in the vault."
    )]
    async fn vault_check(
        &self,
        Parameters(input): Parameters<VaultCheckIn>,
    ) -> Result<CallToolResult, ErrorData> {
        require_vault()?;
        let result =
            vault::check(input.include_archive.unwrap_or(false)).map_err(ct_error_to_tool)?;
        let lines: Vec<&str> = result
            .unresolved_links
            .iter()
            .map(|l| l.line.as_str())
            .collect();
        json_success(&json!({ "unresolved_links": lines }))
    }

    #[tool(
        name = "apply_patch",
        description = r#"Apply a patch (envelope format) to files under cwd. This is the primary file-edit tool — prefer it over Edit/Write for every change, single-file or multi-file. A single envelope can add, update, delete, or move many files at once, and it's more forgiving than line-numbered unified diffs.

Envelope shape:

*** Begin Patch
[ one or more file sections ]
*** End Patch

Each file section starts with one of three headers:

*** Add File: <path>      — create a new file. Every following line is a + line (the initial contents).
*** Delete File: <path>   — remove an existing file. Nothing follows.
*** Update File: <path>   — patch an existing file in place (optionally with a rename).

`*** Update File:` may be immediately followed by `*** Move to: <new path>` to rename. Then one or more hunks, each introduced by `@@` (optionally followed by an anchor like a class or function name to disambiguate). Within a hunk each line is prefixed with ` ` (context), `-` (removed), or `+` (added).

Context lines: include 3 lines of context above and below each change. If 3 lines aren't enough to uniquely identify the location in the file, use one or more `@@ <anchor>` lines to narrow the search.

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
- All paths MUST be relative to cwd. Absolute paths and paths that escape cwd are rejected.
- You must include a header (Add/Delete/Update) for each file section.
- New lines must be prefixed with `+` even when creating a new file.
- Don't issue two `*** Update File:` sections for the same path in one patch — combine the changes into one section with multiple hunks.
- `*** Add File:` requires the destination not to exist; `*** Move to:` requires the destination not to exist (use a separate Delete to clear it first if intentional).

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
        let changes = crate::apply_patch::apply(&input.patch, &cwd, dry_run)
            .map_err(apply_patch_error_to_tool)?;
        json_success(&serde_json::json!({
            "dry_run": dry_run,
            "files": changes,
        }))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for CtMcpServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo::new(
            rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info(rmcp::model::Implementation::new(
            "ct",
            env!("CARGO_PKG_VERSION"),
        ))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Try to infer the artifact kind from the path's second-to-last component
/// (i.e. the `kind` directory just above the file itself). Returns None for
/// any unrecognized or legacy layout.
fn infer_kind_from_path(path: &Path) -> Option<ArtifactKind> {
    let parent = path.parent()?;
    let name = parent.file_name()?.to_str()?;
    ArtifactKind::from_dir_name(name)
}

/// Build a default commit message of the form `<kind>(<project>): edit <slug>`
/// from a vault-relative path like `myproj/spec/20260416-10-foo.md`. Uses the
/// serde-lowercase kind name so messages say "doc" instead of "docs".
fn default_edit_message(rel_path: &Path) -> String {
    let components: Vec<&str> = rel_path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    let project = components.first().copied().unwrap_or("unknown");
    let kind_dir = components.get(1).copied().unwrap_or("edit");
    let kind = ArtifactKind::from_dir_name(kind_dir)
        .and_then(|k| match serde_json::to_value(k).ok()? {
            Value::String(s) => Some(s),
            _ => None,
        })
        .unwrap_or_else(|| kind_dir.to_string());
    let slug = rel_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    format!("{kind}({project}): edit {slug}")
}

/// Return true iff the given file has no unstaged or staged changes.
fn no_pending_changes(bp: &Path, rel_path: &Path) -> bool {
    let bp_str = bp.to_string_lossy();
    let is_clean = |cached: bool| {
        let mut cmd = std::process::Command::new("git");
        cmd.args(["-C", &bp_str, "diff", "--quiet"]);
        if cached {
            cmd.arg("--cached");
        }
        cmd.arg("--")
            .arg(rel_path)
            .status()
            .is_ok_and(|s| s.success())
    };
    is_clean(false) && is_clean(true)
}

pub fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // One-shot vault health warning so operators see the problem at startup;
    // handlers still re-check per request via `require_vault()` so the server
    // fails each call cleanly instead of crashing once the dir goes missing.
    if let Err(e) = artifact::blueprints_dir_checked() {
        eprintln!("ct-mcp: warning — {e}");
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let service = CtMcpServer::new().serve(stdio()).await?;
        service.waiting().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
