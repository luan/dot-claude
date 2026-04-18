use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use serde::Serialize;

use super::crud::resolve_artifact_path;
use super::{
    ArtifactKind, CtError, SyncError, blueprints_dir, commit_and_push, commit_and_push_paths, fatal,
};

/// Detect the source subfolder (spec/, dive/, plan/, etc.) from a vault path.
/// Falls back to `kind.dir_name()` if the subfolder isn't recognized.
fn detect_subfolder(path: &Path, bp: &Path, kind: ArtifactKind) -> String {
    const VALID_SUBFOLDERS: &[&str] = &["spec", "dive", "plan", "review", "report", "docs"];
    path.strip_prefix(bp)
        .ok()
        .and_then(|rel| rel.components().nth(1)) // skip project component
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .filter(|s| VALID_SUBFOLDERS.contains(&s.as_str()))
        .unwrap_or_else(|| kind.dir_name().to_string())
}

/// Core archive logic for a single file: git notes, move, stage deletion.
/// Does NOT commit or push and does NOT emit user-facing output — the CLI
/// wrapper reports the move.
fn archive_single(
    kind: ArtifactKind,
    path: &Path,
    bp: &Path,
    proj_name: &str,
) -> Result<PathBuf, String> {
    let source_subfolder = detect_subfolder(path, bp, kind);
    // Best-effort: store as git note in the current project repo.
    let git_dir = process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    if let Some(ref gd) = git_dir {
        let notes_ref = match kind {
            ArtifactKind::Plan => "plans",
            ArtifactKind::Spec => "specs",
            ArtifactKind::Review => "reviews",
            ArtifactKind::Report => "reports",
            ArtifactKind::Doc => "docs",
        };
        let _ = process::Command::new("git")
            .args([
                "-C",
                gd,
                "notes",
                &format!("--ref={notes_ref}"),
                "append",
                "-F",
            ])
            .arg(path)
            .arg("HEAD")
            .status();
    }

    let archive_dir = bp.join(proj_name).join("archive").join(&source_subfolder);
    fs::create_dir_all(&archive_dir)
        .map_err(|e| format!("cannot create archive directory: {e}"))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| "cannot determine file name".to_string())?;
    let dest = archive_dir.join(file_name);
    fs::rename(path, &dest).map_err(|e| format!("archiving file: {e}"))?;

    // Stage the deletion of the original.
    if let Ok(src_rel) = path.strip_prefix(bp) {
        let bp_str = bp.to_string_lossy();
        let _ = process::Command::new("git")
            .args(["-C", &bp_str, "add", "--"])
            .arg(src_rel)
            .status();
    }

    Ok(dest)
}

/// Compute where a file would land in the archive without moving it.
fn archive_dest(path: &Path, bp: &Path, proj_name: &str, kind: ArtifactKind) -> PathBuf {
    let subfolder = detect_subfolder(path, bp, kind);
    let archive_dir = bp.join(proj_name).join("archive").join(subfolder);
    let file_name = path.file_name().unwrap_or_default();
    archive_dir.join(file_name)
}

/// Validate a file path for archiving: must exist and live inside the vault.
fn validate_archive_path(file_path: &str, bp: &Path) -> Result<(PathBuf, String), CtError> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(CtError::Validation(format!("file not found: {file_path}")));
    }
    let rel_path = path
        .strip_prefix(bp)
        .map_err(|_| CtError::Validation(format!("file is not inside {}", bp.display())))?;
    let proj_name = rel_path
        .components()
        .next()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .ok_or_else(|| {
            CtError::Validation("cannot determine project from file path".to_string())
        })?;
    Ok((path.to_path_buf(), proj_name))
}

/// Outcome of a successful `archive` call.
#[derive(Debug, Clone, Serialize)]
pub struct ArchiveOutcome {
    pub path: PathBuf,
}

/// Archive a single resolved artifact path. Does NOT print — the CLI wrapper
/// reports `Archived: <src> → <dest>`.
pub fn archive(kind: ArtifactKind, path: &Path) -> Result<ArchiveOutcome, CtError> {
    let bp = blueprints_dir();
    let path_str = path.to_string_lossy();
    let (path, proj_name) = validate_archive_path(&path_str, &bp)?;

    let dest = archive_single(kind, &path, &bp, &proj_name)
        .map_err(|e| CtError::Sync(SyncError::Add(e)))?;

    if let Ok(dest_rel) = dest.strip_prefix(&bp) {
        let slug = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        commit_and_push(dest_rel, &format!("archive({}): {}", proj_name, slug))?;
    }
    Ok(ArchiveOutcome { path: dest })
}

pub fn cmd_archive(kind: ArtifactKind, file_path: &str, dry_run: bool) -> Result<(), SyncError> {
    let bp = blueprints_dir();
    let resolved = match resolve_artifact_path(file_path, kind) {
        Ok(p) => p,
        Err(e) => fatal(&e.to_string()),
    };
    let resolved_str = resolved.to_string_lossy();
    let (path, proj_name) = match validate_archive_path(&resolved_str, &bp) {
        Ok(v) => v,
        Err(e) => fatal(&e.to_string()),
    };

    if dry_run {
        let dest = archive_dest(&path, &bp, &proj_name, kind);
        println!("Would archive: {} → {}", path.display(), dest.display());
        println!("Total: 1 artifact");
        return Ok(());
    }

    let dest = match archive(kind, &path) {
        Ok(o) => o.path,
        Err(CtError::Sync(e)) => return Err(e),
        Err(e) => fatal(&e.to_string()),
    };
    eprintln!("Archived: {} → {}", path.display(), dest.display());
    Ok(())
}

pub fn cmd_archive_batch(
    kind: ArtifactKind,
    file_paths: &[String],
    dry_run: bool,
) -> Result<(), SyncError> {
    let bp = blueprints_dir();

    let validated: Vec<(PathBuf, String)> = file_paths
        .iter()
        .map(|fp| {
            let resolved = match resolve_artifact_path(fp, kind) {
                Ok(p) => p,
                Err(e) => fatal(&e.to_string()),
            };
            let resolved_str = resolved.to_string_lossy();
            match validate_archive_path(&resolved_str, &bp) {
                Ok(v) => v,
                Err(e) => fatal(&e.to_string()),
            }
        })
        .collect();

    if validated.is_empty() {
        return Ok(());
    }

    let proj_name = &validated[0].1;
    for (path, proj) in &validated {
        if proj != proj_name {
            fatal(&format!(
                "mixed projects in batch: expected {proj_name}, got {proj} for {}",
                path.display()
            ));
        }
    }

    if dry_run {
        for (path, _) in &validated {
            let dest = archive_dest(path, &bp, proj_name, kind);
            println!("Would archive: {} → {}", path.display(), dest.display());
        }
        println!("Total: {} artifacts", validated.len());
        return Ok(());
    }

    // If one fails mid-batch, commit what succeeded so files aren't orphaned.
    let mut dests: Vec<PathBuf> = Vec::with_capacity(validated.len());
    let mut batch_err: Option<String> = None;
    for (path, _) in &validated {
        match archive_single(kind, path, &bp, proj_name) {
            Ok(dest) => {
                eprintln!("Archived: {} → {}", path.display(), dest.display());
                dests.push(dest);
            }
            Err(e) => {
                eprintln!("archive failed for {}: {e}", path.display());
                batch_err = Some(e);
                break;
            }
        }
    }

    if dests.is_empty() {
        if let Some(e) = batch_err {
            return Err(SyncError::Add(e));
        }
        return Ok(());
    }

    let n = dests.len();
    let rel_dests: Vec<&Path> = dests
        .iter()
        .filter_map(|d| d.strip_prefix(&bp).ok())
        .collect();
    commit_and_push_paths(&rel_dests, &format!("archive({proj_name}): {n} artifacts"))?;

    if let Some(e) = batch_err {
        eprintln!("committed {n} successful archives, but batch had an error: {e}");
    }

    Ok(())
}
