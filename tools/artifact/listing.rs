use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::SystemTime;

use super::{
    Artifact, ArtifactKind, artifact_dir, blueprints_dir, extract_frontmatter_full, fatal,
    project_name, resolve_repo_root,
};

pub fn list_artifacts(kind: ArtifactKind, include_dives: bool) -> Vec<Artifact> {
    list_artifacts_filtered(kind, false, include_dives, None)
}

pub fn list_archived_artifacts(kind: ArtifactKind) -> Vec<Artifact> {
    list_artifacts_filtered(kind, true, false, None)
}

/// Like `list_artifacts`, but only scans a single project subdirectory.
pub fn list_artifacts_for_project(
    kind: ArtifactKind,
    include_dives: bool,
    project_name_filter: &str,
) -> Vec<Artifact> {
    list_artifacts_filtered(kind, false, include_dives, Some(project_name_filter))
}

/// Like `list_archived_artifacts`, but only scans a single project subdirectory.
pub fn list_archived_artifacts_for_project(
    kind: ArtifactKind,
    project_name_filter: &str,
) -> Vec<Artifact> {
    list_artifacts_filtered(kind, true, false, Some(project_name_filter))
}

fn list_artifacts_filtered(
    kind: ArtifactKind,
    archived: bool,
    include_dives: bool,
    project_filter: Option<&str>,
) -> Vec<Artifact> {
    let bp = blueprints_dir();
    let mut artifacts = Vec::new();
    let Ok(entries) = fs::read_dir(&bp) else {
        return artifacts;
    };
    for entry in entries.flatten() {
        let proj_path = entry.path();
        if !proj_path.is_dir() {
            continue;
        }
        let proj_name = proj_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if let Some(filter) = project_filter
            && proj_name != filter
        {
            continue;
        }

        if archived {
            let archive_dir = proj_path.join("archive").join(kind.dir_name());
            collect_artifacts(&bp, &archive_dir, &proj_name, &mut artifacts);
            if kind == ArtifactKind::Spec {
                let archive_dive_dir = proj_path.join("archive").join("dive");
                collect_artifacts(&bp, &archive_dive_dir, &proj_name, &mut artifacts);
            }
        } else {
            let kind_dir = proj_path.join(kind.dir_name());
            collect_artifacts(&bp, &kind_dir, &proj_name, &mut artifacts);
            if include_dives && kind == ArtifactKind::Spec {
                let dive_dir = proj_path.join("dive");
                collect_artifacts(&bp, &dive_dir, &proj_name, &mut artifacts);
            }
        }
    }
    artifacts.sort_by_key(|a| std::cmp::Reverse(a.mod_time));
    artifacts
}

fn collect_artifacts(base: &Path, dir: &Path, fallback_project: &str, out: &mut Vec<Artifact>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().is_none_or(|ext| ext != "md") {
            continue;
        }
        let Some(info) = entry.metadata().ok() else {
            continue;
        };
        let name = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .with_extension("")
            .to_string_lossy()
            .to_string();
        let (title, _, created, source, tags, author) = extract_frontmatter_full(&path);
        out.push(Artifact {
            name,
            path,
            title,
            project: fallback_project.to_string(),
            mod_time: info.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            size: info.len(),
            created,
            source,
            tags,
            author,
        });
    }
}

pub fn load_content(kind: ArtifactKind, path: &PathBuf) -> String {
    let path_str = path.to_string_lossy();
    if let Some(rest) = path_str.strip_prefix("git-notes://") {
        // Format: git-notes://<git-dir>/<40-char-sha>. Commit SHAs are 40 hex
        // chars, so split at len-40 rather than rfind('/') (git_dir contains /).
        if rest.len() > 41 {
            let split = rest.len() - 40;
            let git_dir = &rest[..split - 1];
            let commit_sha = &rest[split..];
            let notes_ref = match kind {
                ArtifactKind::Plan => "plans",
                ArtifactKind::Spec => "specs",
                ArtifactKind::Review => "reviews",
                ArtifactKind::Report => "reports",
                ArtifactKind::Doc => "docs",
            };
            return process::Command::new("git")
                .args([
                    "-C",
                    git_dir,
                    "notes",
                    &format!("--ref={notes_ref}"),
                    "show",
                    commit_sha,
                ])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_else(|| "Error: could not read git note".to_string());
        }
        return "Error: invalid git-notes path".to_string();
    }
    fs::read_to_string(path).unwrap_or_else(|e| format!("Error loading {}: {e}", kind.dir_name()))
}

pub fn cmd_latest(
    kind: ArtifactKind,
    project: Option<&str>,
    task_file: Option<&str>,
    include_dives: bool,
) {
    let mut project = project.unwrap_or("").to_string();

    if project.is_empty() && task_file.is_none() {
        let output = process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output();
        match output {
            Ok(o) if o.status.success() => {
                project = resolve_repo_root(String::from_utf8_lossy(&o.stdout).trim());
            }
            _ => {
                project = env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| fatal("cannot determine working directory"));
            }
        }
    } else if !project.is_empty() {
        project = resolve_repo_root(&project);
    }

    match latest_artifact(kind, task_file, &project, include_dives) {
        Ok(p) => println!("{}", p.display()),
        Err(e) => fatal(&e),
    }
}

/// Core logic for finding the latest artifact of a given kind.
pub fn latest_artifact(
    kind: ArtifactKind,
    task_file: Option<&str>,
    project: &str,
    include_dives: bool,
) -> Result<PathBuf, String> {
    if let Some(tf) = task_file {
        let p = PathBuf::from(tf);
        if p.exists() {
            return Ok(p);
        }
        return Err(format!("task-file not found: {tf}"));
    }

    let mut latest_path: Option<PathBuf> = None;
    let mut latest_time = SystemTime::UNIX_EPOCH;

    let dirs_to_scan: Vec<PathBuf> = if include_dives && kind == ArtifactKind::Spec {
        let bp = blueprints_dir();
        let proj_name = project_name(project);
        vec![
            artifact_dir(project, kind),
            bp.join(&proj_name).join("dive"),
        ]
    } else {
        vec![artifact_dir(project, kind)]
    };

    let mut any_dir_readable = false;
    for dir in &dirs_to_scan {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        any_dir_readable = true;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() || path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            if let Ok(meta) = entry.metadata()
                && let Ok(modified) = meta.modified()
                && modified > latest_time
            {
                latest_time = modified;
                latest_path = Some(path);
            }
        }
    }

    if latest_path.is_none() && !any_dir_readable {
        let dir = &dirs_to_scan[0];
        return Err(format!(
            "cannot read {} directory {}: directory not found",
            kind.dir_name(),
            dir.display()
        ));
    }

    latest_path.ok_or_else(|| {
        format!(
            "no {} files found in {}",
            kind.dir_name(),
            dirs_to_scan[0].display()
        )
    })
}
