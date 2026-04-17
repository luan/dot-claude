// Derives `derive_new_contents` from openai/codex apply-patch/src/lib.rs
// https://github.com/openai/codex/tree/fe7c959e90d46abb8311e4a0b369e6cb32bf337e
// Licensed under Apache License 2.0. See NOTICE at workspace root.

use std::collections::HashSet;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use super::diff::unified_diff;
use super::parser::Hunk;
use super::parser::UpdateFileChunk;
use super::parser::parse_patch;
use super::seek_sequence;

#[derive(Debug, serde::Serialize)]
pub struct FileChange {
    pub path: String,
    #[serde(rename = "type")]
    pub kind: ChangeType,
    pub additions: usize,
    pub deletions: usize,
    pub unified_diff: String,
    pub move_path: Option<String>,
    #[serde(skip)]
    pub new_content: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    Add,
    Update,
    Delete,
    Move,
}

#[derive(Debug, thiserror::Error)]
pub enum ApplyPatchError {
    #[error("parse error: {0}")]
    Parse(#[from] super::parser::ParseError),
    #[error("path escapes cwd: {0}")]
    PathEscape(String),
    #[error("absolute path not allowed: {0}")]
    AbsolutePath(String),
    #[error("context not found in {path}")]
    ContextNotFound { path: String },
    #[error("delete target is a directory: {0}")]
    DeleteIsDirectory(String),
    #[error("add target already exists: {0}")]
    AddTargetExists(String),
    #[error(
        "multiple updates target the same path: {0} (combine into one Update with multiple hunks)"
    )]
    DuplicateUpdate(String),
    #[error("move target already exists: {0}")]
    MoveTargetExists(String),
    #[error("io error ({path}): {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

struct ContextFailure;

pub fn apply(patch: &str, cwd: &Path, dry_run: bool) -> Result<Vec<FileChange>, ApplyPatchError> {
    let changes = plan(patch, cwd)?;
    if !dry_run {
        commit(cwd, &changes)?;
    }
    Ok(changes)
}

pub fn plan(patch: &str, cwd: &Path) -> Result<Vec<FileChange>, ApplyPatchError> {
    let hunks = parse_patch(patch)?;
    let cwd_canon = cwd.canonicalize().map_err(|source| ApplyPatchError::Io {
        path: cwd.display().to_string(),
        source,
    })?;

    // Detect two `*** Update File:` sections targeting the same canonical path.
    // Independent plans would both read the original and the second write would
    // silently overwrite the first — reject up front and ask the caller to
    // combine hunks.
    let mut seen_updates: HashSet<PathBuf> = HashSet::new();
    for hunk in &hunks {
        if let Hunk::UpdateFile { path, .. } = hunk {
            let abs = ensure_in_cwd(&cwd_canon, path)?;
            if !seen_updates.insert(abs) {
                return Err(ApplyPatchError::DuplicateUpdate(display_rel(path)));
            }
        }
    }

    let mut changes = Vec::with_capacity(hunks.len());

    for hunk in hunks {
        match hunk {
            Hunk::AddFile { path, contents } => {
                let abs = ensure_in_cwd(&cwd_canon, &path)?;
                let rel = display_rel(&path);
                if abs.exists() {
                    return Err(ApplyPatchError::AddTargetExists(rel));
                }
                let (diff_text, additions, deletions) = unified_diff(&rel, "", &contents);
                changes.push(FileChange {
                    path: rel,
                    kind: ChangeType::Add,
                    additions,
                    deletions,
                    unified_diff: diff_text,
                    move_path: None,
                    new_content: Some(contents),
                });
            }
            Hunk::DeleteFile { path } => {
                let abs = ensure_in_cwd(&cwd_canon, &path)?;
                let rel = display_rel(&path);
                let original = read_file(&abs, &rel)?;
                let (diff_text, additions, deletions) = unified_diff(&rel, &original, "");
                changes.push(FileChange {
                    path: rel,
                    kind: ChangeType::Delete,
                    additions,
                    deletions,
                    unified_diff: diff_text,
                    move_path: None,
                    new_content: None,
                });
            }
            Hunk::UpdateFile {
                path,
                move_path,
                chunks,
            } => {
                let abs = ensure_in_cwd(&cwd_canon, &path)?;
                let rel = display_rel(&path);
                let original = read_file(&abs, &rel)?;
                let new_content = derive_new_contents(&original, &chunks)
                    .map_err(|_| ApplyPatchError::ContextNotFound { path: rel.clone() })?;

                match move_path {
                    Some(dest) => {
                        let dest_abs = ensure_in_cwd(&cwd_canon, &dest)?;
                        let dest_rel = display_rel(&dest);
                        if dest_abs != abs && dest_abs.exists() {
                            return Err(ApplyPatchError::MoveTargetExists(dest_rel));
                        }
                        let (diff_text, additions, deletions) =
                            unified_diff(&rel, &original, &new_content);
                        changes.push(FileChange {
                            path: rel,
                            kind: ChangeType::Move,
                            additions,
                            deletions,
                            unified_diff: diff_text,
                            move_path: Some(dest_rel),
                            new_content: Some(new_content),
                        });
                    }
                    None => {
                        let (diff_text, additions, deletions) =
                            unified_diff(&rel, &original, &new_content);
                        changes.push(FileChange {
                            path: rel,
                            kind: ChangeType::Update,
                            additions,
                            deletions,
                            unified_diff: diff_text,
                            move_path: None,
                            new_content: Some(new_content),
                        });
                    }
                }
            }
        }
    }

    Ok(changes)
}

pub fn commit(cwd: &Path, changes: &[FileChange]) -> Result<(), ApplyPatchError> {
    let cwd_canon = cwd.canonicalize().map_err(|source| ApplyPatchError::Io {
        path: cwd.display().to_string(),
        source,
    })?;
    for change in changes {
        let source_abs = ensure_in_cwd(&cwd_canon, Path::new(&change.path))?;
        match change.kind {
            ChangeType::Add | ChangeType::Update => {
                let content = change
                    .new_content
                    .as_deref()
                    .expect("Add/Update change missing new_content");
                write_file(&source_abs, &change.path, content)?;
            }
            ChangeType::Move => {
                let dest_rel = change
                    .move_path
                    .as_ref()
                    .expect("Move change missing move_path");
                let dest_abs = ensure_in_cwd(&cwd_canon, Path::new(dest_rel))?;
                let content = change
                    .new_content
                    .as_deref()
                    .expect("Move change missing new_content");
                write_file(&dest_abs, dest_rel, content)?;
                if source_abs != dest_abs {
                    std::fs::remove_file(&source_abs).map_err(|source| ApplyPatchError::Io {
                        path: change.path.clone(),
                        source,
                    })?;
                }
            }
            ChangeType::Delete => {
                let meta =
                    std::fs::metadata(&source_abs).map_err(|source| ApplyPatchError::Io {
                        path: change.path.clone(),
                        source,
                    })?;
                if meta.is_dir() {
                    return Err(ApplyPatchError::DeleteIsDirectory(change.path.clone()));
                }
                std::fs::remove_file(&source_abs).map_err(|source| ApplyPatchError::Io {
                    path: change.path.clone(),
                    source,
                })?;
            }
        }
    }
    Ok(())
}

fn read_file(abs: &Path, rel: &str) -> Result<String, ApplyPatchError> {
    std::fs::read_to_string(abs).map_err(|source| ApplyPatchError::Io {
        path: rel.to_string(),
        source,
    })
}

fn write_file(abs: &Path, rel: &str, content: &str) -> Result<(), ApplyPatchError> {
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent).map_err(|source| ApplyPatchError::Io {
            path: rel.to_string(),
            source,
        })?;
    }
    std::fs::write(abs, content).map_err(|source| ApplyPatchError::Io {
        path: rel.to_string(),
        source,
    })
}

fn display_rel(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Resolve `rel` against an already-canonicalized cwd. Callers MUST pass the
/// canonicalized cwd so we don't re-canonicalize on every hunk.
fn ensure_in_cwd(cwd_canon: &Path, rel: &Path) -> Result<PathBuf, ApplyPatchError> {
    if rel.is_absolute() {
        return Err(ApplyPatchError::AbsolutePath(rel.display().to_string()));
    }

    let mut depth: i32 = 0;
    for component in rel.components() {
        match component {
            Component::Normal(_) => depth += 1,
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return Err(ApplyPatchError::PathEscape(rel.display().to_string()));
                }
            }
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => {
                return Err(ApplyPatchError::AbsolutePath(rel.display().to_string()));
            }
        }
    }

    let joined = cwd_canon.join(rel);
    let canonical_joined = canonicalize_with_existing_prefix(&joined)?;

    if !canonical_joined.starts_with(cwd_canon) {
        return Err(ApplyPatchError::PathEscape(rel.display().to_string()));
    }

    Ok(canonical_joined)
}

/// Canonicalize the longest existing prefix of `path` and append the remaining
/// components verbatim. This lets us resolve symlinks on the parent chain
/// without requiring the leaf (a not-yet-written Add target) to exist.
fn canonicalize_with_existing_prefix(path: &Path) -> Result<PathBuf, ApplyPatchError> {
    let mut remainder: Vec<std::ffi::OsString> = Vec::new();
    let mut cursor = path.to_path_buf();

    let base = loop {
        if cursor.exists() {
            break cursor
                .canonicalize()
                .map_err(|source| ApplyPatchError::Io {
                    path: path.display().to_string(),
                    source,
                })?;
        }
        let Some(name) = cursor.file_name().map(|n| n.to_os_string()) else {
            // Ran out of ancestors without hitting an existing path. Fall back
            // to the original path — shouldn't happen in practice because cwd
            // must exist before we canonicalize it upstream.
            break path.to_path_buf();
        };
        remainder.push(name);
        if !cursor.pop() {
            break path.to_path_buf();
        }
    };

    let mut out = base;
    for name in remainder.into_iter().rev() {
        out.push(name);
    }
    Ok(out)
}

fn derive_new_contents(
    original: &str,
    chunks: &[UpdateFileChunk],
) -> Result<String, ContextFailure> {
    let mut original_lines: Vec<String> = original.split('\n').map(String::from).collect();
    if original_lines.last().is_some_and(String::is_empty) {
        original_lines.pop();
    }

    let replacements = compute_replacements(&original_lines, chunks)?;
    let mut new_lines = apply_replacements(original_lines, &replacements);
    if !new_lines.last().is_some_and(String::is_empty) {
        new_lines.push(String::new());
    }
    Ok(new_lines.join("\n"))
}

fn compute_replacements(
    original_lines: &[String],
    chunks: &[UpdateFileChunk],
) -> Result<Vec<(usize, usize, Vec<String>)>, ContextFailure> {
    let mut replacements: Vec<(usize, usize, Vec<String>)> = Vec::new();
    let mut line_index: usize = 0;

    for chunk in chunks {
        if let Some(ctx_line) = &chunk.change_context {
            if let Some(idx) = seek_sequence(
                original_lines,
                std::slice::from_ref(ctx_line),
                line_index,
                false,
            ) {
                line_index = idx + 1;
            } else {
                return Err(ContextFailure);
            }
        }

        if chunk.old_lines.is_empty() {
            let insertion_idx = if original_lines.last().is_some_and(String::is_empty) {
                original_lines.len() - 1
            } else {
                original_lines.len()
            };
            replacements.push((insertion_idx, 0, chunk.new_lines.clone()));
            continue;
        }

        let mut pattern: &[String] = &chunk.old_lines;
        let mut found = seek_sequence(original_lines, pattern, line_index, chunk.is_end_of_file);
        let mut new_slice: &[String] = &chunk.new_lines;

        if found.is_none() && pattern.last().is_some_and(String::is_empty) {
            pattern = &pattern[..pattern.len() - 1];
            if new_slice.last().is_some_and(String::is_empty) {
                new_slice = &new_slice[..new_slice.len() - 1];
            }
            found = seek_sequence(original_lines, pattern, line_index, chunk.is_end_of_file);
        }

        if let Some(start_idx) = found {
            replacements.push((start_idx, pattern.len(), new_slice.to_vec()));
            line_index = start_idx + pattern.len();
        } else {
            return Err(ContextFailure);
        }
    }

    replacements.sort_by_key(|(idx, _, _)| *idx);
    Ok(replacements)
}

fn apply_replacements(
    mut lines: Vec<String>,
    replacements: &[(usize, usize, Vec<String>)],
) -> Vec<String> {
    for (start_idx, old_len, new_segment) in replacements.iter().rev() {
        let start_idx = *start_idx;
        let old_len = *old_len;

        for _ in 0..old_len {
            if start_idx < lines.len() {
                lines.remove(start_idx);
            }
        }

        for (offset, new_line) in new_segment.iter().enumerate() {
            lines.insert(start_idx + offset, new_line.clone());
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn ensure_in_cwd_accepts_subpath() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("foo");
        fs::create_dir_all(&sub).unwrap();
        let canon = tmp.path().canonicalize().unwrap();
        let resolved = ensure_in_cwd(&canon, Path::new("foo/bar.rs")).unwrap();
        assert!(resolved.starts_with(&canon));
        assert!(resolved.ends_with("bar.rs"));
    }

    #[test]
    fn ensure_in_cwd_rejects_absolute() {
        let tmp = TempDir::new().unwrap();
        let canon = tmp.path().canonicalize().unwrap();
        let err = ensure_in_cwd(&canon, Path::new("/etc/passwd")).unwrap_err();
        assert!(matches!(err, ApplyPatchError::AbsolutePath(_)), "{err:?}");
    }

    #[test]
    fn ensure_in_cwd_rejects_traversal() {
        let tmp = TempDir::new().unwrap();
        let canon = tmp.path().canonicalize().unwrap();
        let err = ensure_in_cwd(&canon, Path::new("../escape.rs")).unwrap_err();
        assert!(matches!(err, ApplyPatchError::PathEscape(_)), "{err:?}");
    }

    #[test]
    fn plan_add_returns_add_change() {
        let tmp = TempDir::new().unwrap();
        let patch = "*** Begin Patch\n*** Add File: hello.txt\n+hi\n+world\n*** End Patch\n";
        let changes = plan(patch, tmp.path()).unwrap();
        assert_eq!(changes.len(), 1);
        let c = &changes[0];
        assert_eq!(c.kind, ChangeType::Add);
        assert_eq!(c.path, "hello.txt");
        assert!(c.additions > 0);
        assert_eq!(c.deletions, 0);
        assert_eq!(c.new_content.as_deref(), Some("hi\nworld\n"));
        // Filesystem untouched by plan alone.
        assert!(!tmp.path().join("hello.txt").exists());
    }

    #[test]
    fn plan_and_commit_round_trip() {
        let tmp = TempDir::new().unwrap();
        // Seed files for Update, Delete, Move.
        fs::write(tmp.path().join("update_me.txt"), "alpha\nbeta\ngamma\n").unwrap();
        fs::write(tmp.path().join("delete_me.txt"), "bye\n").unwrap();
        fs::write(tmp.path().join("move_src.txt"), "one\ntwo\nthree\n").unwrap();

        let patch = concat!(
            "*** Begin Patch\n",
            "*** Add File: added.txt\n",
            "+fresh\n",
            "*** Update File: update_me.txt\n",
            "@@\n",
            " alpha\n",
            "-beta\n",
            "+BETA\n",
            " gamma\n",
            "*** Delete File: delete_me.txt\n",
            "*** Update File: move_src.txt\n",
            "*** Move to: move_dst.txt\n",
            "@@\n",
            " one\n",
            "-two\n",
            "+TWO\n",
            " three\n",
            "*** End Patch\n",
        );

        let changes = plan(patch, tmp.path()).unwrap();
        assert_eq!(changes.len(), 4);
        commit(tmp.path(), &changes).unwrap();

        assert_eq!(
            fs::read_to_string(tmp.path().join("added.txt")).unwrap(),
            "fresh\n"
        );
        assert_eq!(
            fs::read_to_string(tmp.path().join("update_me.txt")).unwrap(),
            "alpha\nBETA\ngamma\n"
        );
        assert!(!tmp.path().join("delete_me.txt").exists());
        assert!(!tmp.path().join("move_src.txt").exists());
        assert_eq!(
            fs::read_to_string(tmp.path().join("move_dst.txt")).unwrap(),
            "one\nTWO\nthree\n"
        );
    }

    #[test]
    fn dry_run_leaves_fs_unchanged() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("keep.txt"), "stay\n").unwrap();
        let patch = concat!(
            "*** Begin Patch\n",
            "*** Update File: keep.txt\n",
            "@@\n",
            "-stay\n",
            "+moved\n",
            "*** End Patch\n",
        );

        // plan() alone must not write.
        let changes = plan(patch, tmp.path()).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].kind, ChangeType::Update);
        assert_eq!(
            fs::read_to_string(tmp.path().join("keep.txt")).unwrap(),
            "stay\n"
        );

        // apply() with dry_run=true must also not write.
        let _ = apply(patch, tmp.path(), true).unwrap();
        assert_eq!(
            fs::read_to_string(tmp.path().join("keep.txt")).unwrap(),
            "stay\n"
        );
    }
}
