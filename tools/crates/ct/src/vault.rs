use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use serde::Serialize;

use crate::artifact::{
    ALL_KINDS, ArtifactKind, CtError, blueprints_dir, blueprints_dir_unchecked, fatal, home_dir,
    project_name,
};

pub fn cmd_init() {
    let bp = blueprints_dir_unchecked();
    if bp.is_dir() {
        eprintln!("{} already exists", bp.display());
        return;
    }

    fs::create_dir_all(&bp)
        .unwrap_or_else(|e| fatal(&format!("cannot create {}: {e}", bp.display())));

    let init_ok = process::Command::new("git")
        .args(["-C", &bp.to_string_lossy(), "init"])
        .status()
        .is_ok_and(|s| s.success());

    if !init_ok {
        fatal(&format!("git init failed in {}", bp.display()));
    }

    eprintln!("Initialized {} as a git repository", bp.display());
}

pub fn cmd_migrate() {
    let bp = blueprints_dir();
    let home = home_dir();

    let mut migrated = 0u32;

    for kind in ALL_KINDS {
        let legacy_base = Path::new(&home)
            .join(".claude")
            .join(kind.legacy_dir_name());
        let Ok(project_dirs) = fs::read_dir(&legacy_base) else {
            continue;
        };

        for dir_entry in project_dirs.flatten() {
            let proj_dir = dir_entry.path();
            if !proj_dir.is_dir() {
                continue;
            }
            let proj_name = proj_dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if proj_name == "archive" {
                continue;
            }

            let dest_dir = bp.join(&proj_name).join(kind.dir_name());
            fs::create_dir_all(&dest_dir)
                .unwrap_or_else(|e| fatal(&format!("cannot create {}: {e}", dest_dir.display())));

            let Ok(files) = fs::read_dir(&proj_dir) else {
                continue;
            };
            for file_entry in files.flatten() {
                let src = file_entry.path();
                if src.is_dir() || src.extension().is_none_or(|ext| ext != "md") {
                    continue;
                }
                let file_name = src
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let dest = dest_dir.join(&file_name);

                if let Err(e) = fs::copy(&src, &dest) {
                    eprintln!("warning: failed to copy {}: {e}", src.display());
                    continue;
                }

                let archive_dir = proj_dir.join("archive");
                fs::create_dir_all(&archive_dir).ok();
                if let Err(e) = fs::rename(&src, archive_dir.join(&file_name)) {
                    eprintln!("warning: failed to archive original {}: {e}", src.display());
                }

                migrated += 1;
            }
        }
    }

    if migrated > 0 {
        let bp_str = bp.to_string_lossy();
        let _ = process::Command::new("git")
            .args(["-C", &bp_str, "add", "."])
            .status();
        let _ = process::Command::new("git")
            .args([
                "-C",
                &bp_str,
                "commit",
                "-m",
                &format!("migrate: {migrated} artifacts from ~/.claude/"),
            ])
            .status();
        let _ = process::Command::new("git")
            .args(["-C", &bp_str, "push"])
            .status();
    }

    eprintln!("Migrated {migrated} artifact(s) to {}", bp.display());
}

pub fn cmd_project() {
    let toplevel = process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| fatal("cannot determine working directory"))
        });
    let project = crate::artifact::resolve_repo_root(&toplevel);
    println!("{}", project_name(&project));
}

/// A related artifact found by topic keyword overlap.
#[derive(Debug, Clone, Serialize)]
pub struct RelatedHit {
    pub stem: String,
    pub path: PathBuf,
    pub title: String,
}

/// Core: find artifacts in `project` whose slug shares enough keywords with `topic`.
/// `project` — if None, uses the current repo root. Wiki-link stems only, deduped.
pub fn related(
    topic: &str,
    project: Option<&str>,
    include_archive: bool,
) -> Result<Vec<RelatedHit>, CtError> {
    let topic_words: HashSet<&str> = topic
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3)
        .collect();

    if topic_words.is_empty() {
        return Ok(Vec::new());
    }

    let bp = blueprints_dir();
    let project_str = project
        .map(|p| p.to_string())
        .unwrap_or_else(crate::artifact::current_project);
    let resolved = crate::artifact::resolve_repo_root(&project_str);
    let proj_name = project_name(&resolved);
    let proj_dir = bp.join(&proj_name);

    if !proj_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut seen = HashSet::new();
    let mut hits = Vec::new();

    for kind in ALL_KINDS {
        let mut dirs_to_scan = vec![proj_dir.join(kind.dir_name())];
        if include_archive {
            dirs_to_scan.push(proj_dir.join("archive").join(kind.dir_name()));
        }
        for scan_dir in dirs_to_scan {
            let Ok(entries) = fs::read_dir(&scan_dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_none_or(|ext| ext != "md") {
                    continue;
                }
                let stem = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let slug_part = crate::artifact::strip_date_prefix(&stem);
                let slug_words: HashSet<&str> = slug_part
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|w| w.len() >= 3)
                    .collect();
                let overlap = topic_words.intersection(&slug_words).count();
                if (overlap >= 2 || (topic_words.len() <= 2 && overlap >= 1))
                    && seen.insert(stem.clone())
                {
                    hits.push(RelatedHit {
                        stem,
                        path,
                        title: String::new(),
                    });
                }
            }
        }
    }
    Ok(hits)
}

pub fn cmd_related(project: &str, topic: &str, include_archive: bool) {
    let hits = match related(topic, Some(project), include_archive) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    for hit in &hits {
        println!("[[{}]]", hit.stem);
    }
}

/// An unresolved wiki-link, as reported by the Obsidian CLI.
#[derive(Debug, Clone, Serialize)]
pub struct UnresolvedLink {
    pub line: String,
}

/// Result of a vault-wide link check.
#[derive(Debug, Clone, Serialize)]
pub struct VaultCheckResult {
    pub unresolved_links: Vec<UnresolvedLink>,
}

/// Core: run `obsidian unresolved` in the vault and collect its output lines.
/// Always returns the lines actually emitted; stderr/exit status are surfaced
/// via `CtError::Validation` when the tool fails to run at all.
pub fn check(include_archive: bool) -> Result<VaultCheckResult, CtError> {
    let bp = blueprints_dir();
    let output = process::Command::new("obsidian")
        .args(["unresolved"])
        .current_dir(&bp)
        .output()
        .map_err(|e| CtError::Validation(format!("failed to run obsidian cli: {e}")))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let mut unresolved = Vec::new();
    for line in text.lines() {
        if !include_archive && line.contains("archive/") {
            continue;
        }
        unresolved.push(UnresolvedLink {
            line: line.to_string(),
        });
    }
    Ok(VaultCheckResult {
        unresolved_links: unresolved,
    })
}

pub fn cmd_check(include_archive: bool) {
    match check(include_archive) {
        Ok(result) => {
            for link in &result.unresolved_links {
                println!("{}", link.line);
            }
        }
        Err(e) => eprintln!("{e}"),
    }
}

/// Filters for `search`.
#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub kind: Option<ArtifactKind>,
    pub project: Option<String>,
    pub archived: bool,
}

/// A single search hit. Only fields the Obsidian CLI actually gives us are
/// exposed — title/kind/project aren't cheaply recoverable from plain output
/// and were previously returned as empty strings.
#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub path: PathBuf,
    pub raw_line: String,
}

/// Core: shell out to `obsidian search` with the given query, filter lines by
/// kind/project/archive, and return structured hits. CLI wrapper prints lines
/// verbatim (JSON or plain) so existing skills don't break.
pub fn search(query: &str, filters: SearchFilters) -> Result<Vec<SearchHit>, CtError> {
    let bp = blueprints_dir();
    let args = vec!["search".to_string(), format!("query={query}")];
    let output = process::Command::new("obsidian")
        .args(&args)
        .current_dir(&bp)
        .output()
        .map_err(|e| CtError::Validation(format!("failed to run obsidian cli: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CtError::Validation(stderr.trim().to_string()));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let proj_prefix = filters.project.as_deref().map(|p| {
        let resolved = crate::artifact::resolve_repo_root(p);
        let name = project_name(&resolved);
        format!("{name}/")
    });
    let kind_dir = filters.kind.map(|k| format!("{}/", k.dir_name()));

    let mut hits = Vec::new();
    for line in text.lines() {
        if !filters.archived && line.contains("archive/") {
            continue;
        }
        let matches_kind = kind_dir.as_deref().is_none_or(|d| line.contains(d));
        let matches_proj = proj_prefix.as_deref().is_none_or(|p| line.contains(p));
        if matches_kind && matches_proj {
            hits.push(SearchHit {
                path: PathBuf::from(line),
                raw_line: line.to_string(),
            });
        }
    }
    Ok(hits)
}

pub fn cmd_search(
    query: &str,
    json: bool,
    kind_filter: Option<ArtifactKind>,
    project: Option<&str>,
    include_archive: bool,
) {
    // JSON mode routes through obsidian's own JSON formatter so downstream
    // skills keep parsing the same shape. Plain-text mode uses the core
    // `search()` so CLI + MCP share filter logic.
    if json {
        cmd_search_json(query, kind_filter, project, include_archive);
        return;
    }

    let filters = SearchFilters {
        kind: kind_filter,
        project: project.map(|s| s.to_string()),
        archived: include_archive,
    };
    match search(query, filters) {
        Ok(hits) => {
            for hit in &hits {
                // Skills parse obsidian's raw line output — keep it verbatim.
                println!("{}", hit.raw_line);
            }
        }
        Err(e) => eprintln!("{e}"),
    }
}

/// JSON path: preserve obsidian's exact JSON shape by not re-rendering it.
fn cmd_search_json(
    query: &str,
    kind_filter: Option<ArtifactKind>,
    project: Option<&str>,
    include_archive: bool,
) {
    let bp = blueprints_dir();
    let args = vec![
        "search".to_string(),
        format!("query={query}"),
        "format=json".to_string(),
    ];
    let output = process::Command::new("obsidian")
        .args(&args)
        .current_dir(&bp)
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            let proj_prefix = project.map(|p| {
                let resolved = crate::artifact::resolve_repo_root(p);
                let name = project_name(&resolved);
                format!("{name}/")
            });
            let kind_dir = kind_filter.map(|k| format!("{}/", k.dir_name()));
            for line in text.lines() {
                if !include_archive && line.contains("archive/") {
                    continue;
                }
                let matches_kind = kind_dir.as_deref().is_none_or(|d| line.contains(d));
                let matches_proj = proj_prefix.as_deref().is_none_or(|p| line.contains(p));
                if matches_kind && matches_proj {
                    println!("{line}");
                }
            }
        }
        Ok(o) => {
            eprint!("{}", String::from_utf8_lossy(&o.stderr));
        }
        Err(e) => eprintln!("failed to run obsidian cli: {e}"),
    }
}

pub fn cmd_status() {
    let bp = blueprints_dir();
    let bp_str = bp.to_string_lossy();

    // Git status: clean or dirty
    let status_output = process::Command::new("git")
        .args(["-C", &bp_str, "status", "--porcelain"])
        .output();
    match &status_output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            if text.trim().is_empty() {
                println!("working tree: clean");
            } else {
                let dirty_count = text.lines().count();
                println!("working tree: {dirty_count} dirty file(s)");
            }
        }
        Ok(_) | Err(_) => println!("working tree: unknown (git status failed)"),
    }

    // Unpushed commits
    let log_output = process::Command::new("git")
        .args(["-C", &bp_str, "log", "--oneline", "@{u}..HEAD"])
        .output();
    match &log_output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            let count = text.lines().filter(|l| !l.is_empty()).count();
            println!("unpushed commits: {count}");
        }
        // No upstream configured or other error — report 0
        Ok(_) | Err(_) => println!("unpushed commits: 0 (no upstream)"),
    }

    // Total artifact count
    let mut total = 0usize;
    let Ok(projects) = fs::read_dir(&bp) else {
        println!("artifacts: 0");
        return;
    };
    for proj_entry in projects.flatten() {
        let proj_dir = proj_entry.path();
        if !proj_dir.is_dir() {
            continue;
        }
        // Skip .git and hidden dirs
        if proj_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .starts_with('.')
        {
            continue;
        }
        for kind in ALL_KINDS {
            let kind_dir = proj_dir.join(kind.dir_name());
            let Ok(entries) = fs::read_dir(&kind_dir) else {
                continue;
            };
            for entry in entries.flatten() {
                if entry.path().extension().is_some_and(|ext| ext == "md") {
                    total += 1;
                }
            }
        }
    }
    println!("artifacts: {total}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_blueprints_dir<F: FnOnce()>(tmp: &Path, f: F) {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = env::var("CT_BLUEPRINTS_DIR").ok();
        unsafe { env::set_var("CT_BLUEPRINTS_DIR", tmp) };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        match prev {
            Some(v) => unsafe { env::set_var("CT_BLUEPRINTS_DIR", v) },
            None => unsafe { env::remove_var("CT_BLUEPRINTS_DIR") },
        }
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    }

    fn seed(base: &Path, project: &str, kind: ArtifactKind, stem: &str) -> PathBuf {
        let dir = base.join(project).join(kind.dir_name());
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join(format!("{stem}.md"));
        fs::write(&file, format!("---\ntopic: {stem}\n---\n")).unwrap();
        file
    }

    #[test]
    fn related_core_returns_hits_with_stem_and_path() {
        let tmp = std::env::temp_dir().join(format!("ct-related-core-{}", std::process::id()));
        fs::create_dir_all(&tmp).unwrap();

        let plan = seed(&tmp, "myproj", ArtifactKind::Plan, "20260411-auth-flow");
        let _ = seed(&tmp, "myproj", ArtifactKind::Spec, "20260411-ui-theme");

        with_blueprints_dir(&tmp, || {
            // Pass project path that project_name() resolves to "myproj".
            let proj_path = tmp.join("myproj");
            let hits = related(
                "auth flow improvements",
                Some(proj_path.to_str().unwrap()),
                false,
            )
            .expect("related ok");
            assert_eq!(hits.len(), 1, "only auth-flow should overlap");
            assert_eq!(hits[0].stem, "20260411-auth-flow");
            assert_eq!(hits[0].path, plan);
        });
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn related_core_empty_topic_returns_empty() {
        let tmp = std::env::temp_dir().join(format!("ct-related-empty-{}", std::process::id()));
        fs::create_dir_all(&tmp).unwrap();

        with_blueprints_dir(&tmp, || {
            let hits = related("", None, false).expect("empty topic ok");
            assert!(hits.is_empty());
        });
        fs::remove_dir_all(&tmp).ok();
    }
}
