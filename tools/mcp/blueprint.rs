use std::path::{Path, PathBuf};

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData};
use rmcp::schemars::{self, JsonSchema};
use rmcp::{ServerHandler, tool, tool_handler, tool_router};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{ct_error_to_tool, json_success, project_input_to_name, require_vault, resolve};
use crate::artifact::{self, ArtifactKind, CreateOpts, CtError};
use crate::vault::{self, SearchFilters};

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

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub(super) struct BlueprintMcpServer {
    tool_router: ToolRouter<Self>,
}

impl BlueprintMcpServer {
    pub(super) fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router(router = tool_router)]
impl BlueprintMcpServer {
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
        let project = match input.project {
            Some(p) if !p.contains('/') && !p.contains('\\') => {
                artifact::validate_project_name(&p).map_err(ct_error_to_tool)?;
                p
            }
            Some(p) => p,
            None => artifact::current_project(),
        };
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
            let proj_name = project_input_to_name(input.project)?;
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
        let proj_name = project_input_to_name(input.project)?;
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

        // Canonicalize via the shared vault-containment check: resolves `..`
        // and symlinks, then verifies the result stays inside the vault root.
        let (full_canonical, bp_canonical) = artifact::canonicalize_in_vault(&full_path)
            .map_err(|e| ct_error_to_tool(CtError::from(e)))?;
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
            .expect("canonicalize_in_vault guarantees prefix")
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
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for BlueprintMcpServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo::new(
            rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info(rmcp::model::Implementation::new(
            "blueprint",
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
/// from a vault-relative path like `myproj/spec/20260416-10-foo.md`.
fn default_edit_message(rel_path: &Path) -> String {
    let components: Vec<&str> = rel_path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    let project = components.first().copied().unwrap_or("unknown");
    let kind_dir = components.get(1).copied().unwrap_or("edit");
    let kind = ArtifactKind::from_dir_name(kind_dir)
        .map(|k| k.commit_name().to_string())
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
