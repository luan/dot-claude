// Derives `derive_new_contents` from openai/codex apply-patch/src/lib.rs
// https://github.com/openai/codex/tree/fe7c959e90d46abb8311e4a0b369e6cb32bf337e
// Licensed under Apache License 2.0. See NOTICE at workspace root.

use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::diff::unified_diff;
use super::parser::Hunk;
use super::parser::UpdateFileChunk;
use super::parser::parse_patch;
use super::seek_sequence::MatchQuality;
use super::seek_sequence::SeekOutcome;
use super::seek_sequence::seek_sequence;
use super::telemetry::{AnchorAttempt, Fingerprint, sha1_hex};

#[derive(Debug, serde::Serialize)]
pub struct FileChange {
    pub path: String,
    #[serde(rename = "type")]
    pub kind: ChangeType,
    pub additions: usize,
    pub deletions: usize,
    pub unified_diff: String,
    pub move_path: Option<String>,
    /// Per-hunk fuzzy-match report. Empty when every chunk matched exactly.
    /// Entries name only the chunks that needed fuzzy matching (trim /
    /// normalise); callers may echo the diff when the list is non-empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fuzzy_hunks: Vec<HunkFuzzy>,
    /// Post-apply snapshot around each hunk. A small numbered window of the
    /// file as it will exist after commit — lets callers plan subsequent
    /// edits without re-reading the file. Empty for Add/Delete.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub post_apply_regions: Vec<HunkRegion>,
    #[serde(skip)]
    pub new_content: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HunkFuzzy {
    /// 0-based index of the chunk within the file's Update section.
    pub chunk: usize,
    pub tier: MatchQuality,
}

/// Numbered window of the post-apply file state around one hunk. `start_line`
/// is 1-based so the caller can cite file positions directly.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HunkRegion {
    /// 0-based index of the chunk within the file's Update section.
    pub chunk: usize,
    pub start_line: usize,
    pub lines: Vec<String>,
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
    #[error(
        "context not found in {path} at chunk #{chunk}{}{}{}",
        .change_context.as_deref().map(|c| format!(" (@@ {c})")).unwrap_or_default(),
        .first_old_line.as_deref().map(|l| format!(": first expected line was {l:?}")).unwrap_or_default(),
        .near_miss.as_deref().unwrap_or("")
    )]
    ContextNotFound {
        path: String,
        chunk: usize,
        change_context: Option<String>,
        first_old_line: Option<String>,
        near_miss: Option<String>,
    },
    #[error(
        "ambiguous context in {path} at chunk #{chunk}{} — matched at lines {candidates:?}; widen the context or use a more specific @@ anchor",
        .change_context.as_deref().map(|c| format!(" (@@ {c})")).unwrap_or_default(),
    )]
    AmbiguousContext {
        path: String,
        chunk: usize,
        change_context: Option<String>,
        candidates: Vec<usize>,
    },
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
    #[error(
        "in {path} chunk #{chunk}: anchor `@@ {anchor}` also appears as the first context line. The anchor is consumed and the pattern search starts on the line *after* it — drop either the anchor or that first context line."
    )]
    AnchorShadowsFirstContext {
        path: String,
        chunk: usize,
        anchor: String,
    },
    #[error("io error ({path}): {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

enum ChunkFailure {
    NotFound {
        chunk_index: usize,
        change_context: Option<String>,
        first_old_line: Option<String>,
        near_miss: Option<String>,
    },
    Ambiguous {
        chunk_index: usize,
        change_context: Option<String>,
        candidates: Vec<usize>,
    },
    AnchorShadowsFirstContext {
        chunk_index: usize,
        anchor: String,
    },
}

#[derive(Debug)]
pub struct ApplyOutcome {
    pub changes: Vec<FileChange>,
    pub attempts: Vec<AnchorAttempt>,
    pub fingerprints: Vec<(String, Fingerprint)>,
}

#[derive(Debug)]
pub struct ApplyFailure {
    pub error: ApplyPatchError,
    pub attempts: Vec<AnchorAttempt>,
    pub fingerprints: Vec<(String, Fingerprint)>,
}

impl From<ApplyPatchError> for Box<ApplyFailure> {
    fn from(error: ApplyPatchError) -> Self {
        Box::new(ApplyFailure {
            error,
            attempts: Vec::new(),
            fingerprints: Vec::new(),
        })
    }
}

pub fn apply(patch: &str, cwd: &Path, dry_run: bool) -> Result<ApplyOutcome, Box<ApplyFailure>> {
    let outcome = plan(patch, cwd)?;
    if !dry_run {
        commit(cwd, &outcome.changes).map_err(|error| {
            Box::new(ApplyFailure {
                error,
                attempts: outcome.attempts.clone(),
                fingerprints: outcome.fingerprints.clone(),
            })
        })?;
    }
    Ok(outcome)
}

pub fn plan(patch: &str, cwd: &Path) -> Result<ApplyOutcome, Box<ApplyFailure>> {
    let hunks =
        parse_patch(patch).map_err(|e| Box::<ApplyFailure>::from(ApplyPatchError::from(e)))?;
    let cwd_canon = cwd.canonicalize().map_err(|source| {
        Box::<ApplyFailure>::from(ApplyPatchError::Io {
            path: cwd.display().to_string(),
            source,
        })
    })?;

    // Detect two `*** Update File:` sections targeting the same canonical path.
    // Independent plans would both read the original and the second write would
    // silently overwrite the first — reject up front and ask the caller to
    // combine hunks.
    let mut seen_updates: HashSet<PathBuf> = HashSet::new();
    for hunk in &hunks {
        if let Hunk::UpdateFile { path, .. } = hunk {
            let abs = resolve_path(&cwd_canon, path)?;
            if !seen_updates.insert(abs) {
                return Err(Box::<ApplyFailure>::from(ApplyPatchError::DuplicateUpdate(
                    display_rel(path),
                )));
            }
        }
    }

    // Paths scheduled for Delete in this envelope. Add and Move-to may target
    // these even if they currently exist on disk — the envelope expresses an
    // atomic replace.
    let mut pending_deletes: HashSet<PathBuf> = HashSet::new();
    let mut changes = Vec::with_capacity(hunks.len());
    let mut attempts: Vec<AnchorAttempt> = Vec::new();
    let mut fingerprints: Vec<(String, Fingerprint)> = Vec::new();

    let fail = |error: ApplyPatchError,
                attempts: &[AnchorAttempt],
                fingerprints: &[(String, Fingerprint)]|
     -> Box<ApplyFailure> {
        Box::new(ApplyFailure {
            error,
            attempts: attempts.to_vec(),
            fingerprints: fingerprints.to_vec(),
        })
    };

    for hunk in hunks {
        match hunk {
            Hunk::AddFile { path, contents } => {
                let abs = resolve_path(&cwd_canon, &path)
                    .map_err(|e| fail(e, &attempts, &fingerprints))?;
                let rel = display_rel(&path);
                if abs.exists() && !pending_deletes.contains(&abs) {
                    return Err(fail(
                        ApplyPatchError::AddTargetExists(rel),
                        &attempts,
                        &fingerprints,
                    ));
                }
                let (diff_text, additions, deletions) = unified_diff(&rel, "", &contents);
                changes.push(FileChange {
                    path: rel,
                    kind: ChangeType::Add,
                    additions,
                    deletions,
                    unified_diff: diff_text,
                    move_path: None,
                    fuzzy_hunks: Vec::new(),
                    post_apply_regions: Vec::new(),
                    new_content: Some(contents),
                });
            }
            Hunk::DeleteFile { path } => {
                let abs = resolve_path(&cwd_canon, &path)
                    .map_err(|e| fail(e, &attempts, &fingerprints))?;
                let rel = display_rel(&path);
                let (original, fp) =
                    read_file(&abs, &rel).map_err(|e| fail(e, &attempts, &fingerprints))?;
                fingerprints.push((rel.clone(), fp));
                let (diff_text, additions, deletions) = unified_diff(&rel, &original, "");
                pending_deletes.insert(abs);
                changes.push(FileChange {
                    path: rel,
                    kind: ChangeType::Delete,
                    additions,
                    deletions,
                    unified_diff: diff_text,
                    move_path: None,
                    fuzzy_hunks: Vec::new(),
                    post_apply_regions: Vec::new(),
                    new_content: None,
                });
            }
            Hunk::UpdateFile {
                path,
                move_path,
                chunks,
            } => {
                let abs = resolve_path(&cwd_canon, &path)
                    .map_err(|e| fail(e, &attempts, &fingerprints))?;
                let rel = display_rel(&path);
                let (original, fp) =
                    read_file(&abs, &rel).map_err(|e| fail(e, &attempts, &fingerprints))?;
                fingerprints.push((rel.clone(), fp));
                let (new_content, fuzzy_hunks, post_apply_regions) =
                    derive_new_contents(&original, &chunks, &rel, &mut attempts).map_err(
                        |err| fail(chunk_failure_to_error(err, &rel), &attempts, &fingerprints),
                    )?;

                match move_path {
                    Some(dest) => {
                        let dest_abs = resolve_path(&cwd_canon, &dest)
                            .map_err(|e| fail(e, &attempts, &fingerprints))?;
                        let dest_rel = display_rel(&dest);
                        if dest_abs != abs
                            && dest_abs.exists()
                            && !pending_deletes.contains(&dest_abs)
                        {
                            return Err(fail(
                                ApplyPatchError::MoveTargetExists(dest_rel),
                                &attempts,
                                &fingerprints,
                            ));
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
                            fuzzy_hunks,
                            post_apply_regions,
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
                            fuzzy_hunks,
                            post_apply_regions,
                            new_content: Some(new_content),
                        });
                    }
                }
            }
        }
    }

    Ok(ApplyOutcome {
        changes,
        attempts,
        fingerprints,
    })
}

pub fn commit(cwd: &Path, changes: &[FileChange]) -> Result<(), ApplyPatchError> {
    let cwd_canon = cwd.canonicalize().map_err(|source| ApplyPatchError::Io {
        path: cwd.display().to_string(),
        source,
    })?;
    for change in changes {
        let source_abs = resolve_path(&cwd_canon, Path::new(&change.path))?;
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
                let dest_abs = resolve_path(&cwd_canon, Path::new(dest_rel))?;
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

fn read_file(abs: &Path, rel: &str) -> Result<(String, Fingerprint), ApplyPatchError> {
    let contents = std::fs::read_to_string(abs).map_err(|source| ApplyPatchError::Io {
        path: rel.to_string(),
        source,
    })?;
    let meta = std::fs::metadata(abs).map_err(|source| ApplyPatchError::Io {
        path: rel.to_string(),
        source,
    })?;
    let mtime_ns = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0);
    let sha1 = sha1_hex(contents.as_bytes());
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    Ok((contents, Fingerprint { mtime_ns, sha1, ts }))
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

/// Resolve a patch-referenced path. Absolute paths are accepted as-is;
/// relative paths are joined with `cwd_canon`. The canonicalisation walk
/// handles not-yet-existing targets (e.g. Add) by canonicalising the longest
/// existing prefix and appending the rest verbatim.
fn resolve_path(cwd_canon: &Path, rel: &Path) -> Result<PathBuf, ApplyPatchError> {
    let target = if rel.is_absolute() {
        rel.to_path_buf()
    } else {
        cwd_canon.join(rel)
    };
    canonicalize_with_existing_prefix(&target)
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
    file_rel: &str,
    attempts: &mut Vec<AnchorAttempt>,
) -> Result<(String, Vec<HunkFuzzy>, Vec<HunkRegion>), ChunkFailure> {
    let mut original_lines: Vec<String> = original.split('\n').map(String::from).collect();
    if original_lines.last().is_some_and(String::is_empty) {
        original_lines.pop();
    }

    let (replacements, fuzzy_hunks) =
        compute_replacements(&original_lines, chunks, file_rel, attempts)?;
    let regions_meta = plan_post_apply_regions(&replacements);
    let mut new_lines = apply_replacements(original_lines, replacements);
    if !new_lines.last().is_some_and(String::is_empty) {
        new_lines.push(String::new());
    }
    let regions = materialize_regions(&new_lines, &regions_meta);
    Ok((new_lines.join("\n"), fuzzy_hunks, regions))
}

#[derive(Debug)]
struct Replacement {
    chunk: usize,
    start: usize,
    old_len: usize,
    new: Vec<String>,
}

type Replacements = Vec<Replacement>;

/// Amount of context on each side of a hunk in the post-apply echo. Small
/// enough to keep responses cheap, large enough that the agent can see the
/// surrounding neighborhood for a follow-up edit.
const POST_APPLY_PAD: usize = 3;

/// Walk the (sorted) replacement list once to compute each hunk's post-apply
/// line range. Each replacement shifts every later replacement's position by
/// `new.len() - old_len`.
fn plan_post_apply_regions(replacements: &[Replacement]) -> Vec<(usize, usize, usize)> {
    let mut out = Vec::with_capacity(replacements.len());
    let mut delta: isize = 0;
    for r in replacements {
        let new_start = (r.start as isize + delta).max(0) as usize;
        out.push((r.chunk, new_start, r.new.len()));
        delta += r.new.len() as isize - r.old_len as isize;
    }
    out
}

/// Slice a padded window out of the post-apply file for each hunk.
fn materialize_regions(
    new_lines: &[String],
    regions_meta: &[(usize, usize, usize)],
) -> Vec<HunkRegion> {
    let mut regions = Vec::with_capacity(regions_meta.len());
    for &(chunk_idx, new_start, new_len) in regions_meta {
        let start = new_start.saturating_sub(POST_APPLY_PAD);
        let end = (new_start + new_len + POST_APPLY_PAD).min(new_lines.len());
        if start >= end {
            continue;
        }
        regions.push(HunkRegion {
            chunk: chunk_idx,
            start_line: start + 1,
            lines: new_lines[start..end].to_vec(),
        });
    }
    regions
}

fn compute_replacements(
    original_lines: &[String],
    chunks: &[UpdateFileChunk],
    file_rel: &str,
    attempts: &mut Vec<AnchorAttempt>,
) -> Result<(Replacements, Vec<HunkFuzzy>), ChunkFailure> {
    let mut replacements: Replacements = Vec::new();
    let mut line_index: usize = 0;
    let mut fuzzy_hunks: Vec<HunkFuzzy> = Vec::new();

    for (chunk_index, chunk) in chunks.iter().enumerate() {
        let mut chunk_worst = MatchQuality::Exact;
        if let Some(ctx_line) = &chunk.change_context {
            // Catch the anchor-shadow footgun *before* seeking: the anchor
            // consumes its line and the pattern search resumes below, so if
            // the first pattern line is the same text it will either miss
            // entirely or (worse) match a later duplicate. Specific error
            // beats a confusing "context not found".
            if chunk.old_lines.first() == Some(ctx_line) {
                attempts.push(AnchorAttempt {
                    file_path: file_rel.to_string(),
                    chunk_index,
                    anchor_text: Some(ctx_line.clone()),
                    success: false,
                    fuzzy_tier: None,
                });
                return Err(ChunkFailure::AnchorShadowsFirstContext {
                    chunk_index,
                    anchor: ctx_line.clone(),
                });
            }
            match seek_sequence(
                original_lines,
                std::slice::from_ref(ctx_line),
                line_index,
                false,
            ) {
                SeekOutcome::Unique { idx, quality } => {
                    line_index = idx + 1;
                    chunk_worst = worse_of(chunk_worst, quality);
                }
                SeekOutcome::Ambiguous { matches, .. } => {
                    attempts.push(AnchorAttempt {
                        file_path: file_rel.to_string(),
                        chunk_index,
                        anchor_text: Some(ctx_line.clone()),
                        success: false,
                        fuzzy_tier: None,
                    });
                    return Err(ChunkFailure::Ambiguous {
                        chunk_index,
                        change_context: chunk.change_context.clone(),
                        candidates: one_based(&matches),
                    });
                }
                SeekOutcome::NotFound => {
                    attempts.push(AnchorAttempt {
                        file_path: file_rel.to_string(),
                        chunk_index,
                        anchor_text: Some(ctx_line.clone()),
                        success: false,
                        fuzzy_tier: None,
                    });
                    return Err(ChunkFailure::NotFound {
                        chunk_index,
                        change_context: chunk.change_context.clone(),
                        first_old_line: chunk.old_lines.first().cloned(),
                        near_miss: near_miss_snippet(
                            original_lines,
                            line_index,
                            Some(ctx_line.as_str()),
                        ),
                    });
                }
            }
        }

        if chunk.old_lines.is_empty() {
            let insertion_idx = if original_lines.last().is_some_and(String::is_empty) {
                original_lines.len() - 1
            } else {
                original_lines.len()
            };
            replacements.push(Replacement {
                chunk: chunk_index,
                start: insertion_idx,
                old_len: 0,
                new: chunk.new_lines.clone(),
            });
            attempts.push(AnchorAttempt {
                file_path: file_rel.to_string(),
                chunk_index,
                anchor_text: chunk.change_context.clone(),
                success: true,
                fuzzy_tier: Some(chunk_worst.as_str().to_string()),
            });
            continue;
        }

        let mut pattern: &[String] = &chunk.old_lines;
        let mut new_slice: &[String] = &chunk.new_lines;
        let mut outcome = seek_sequence(original_lines, pattern, line_index, chunk.is_end_of_file);

        if matches!(outcome, SeekOutcome::NotFound) && pattern.last().is_some_and(String::is_empty)
        {
            pattern = &pattern[..pattern.len() - 1];
            if new_slice.last().is_some_and(String::is_empty) {
                new_slice = &new_slice[..new_slice.len() - 1];
            }
            outcome = seek_sequence(original_lines, pattern, line_index, chunk.is_end_of_file);
        }

        match outcome {
            SeekOutcome::Unique { idx, quality } => {
                replacements.push(Replacement {
                    chunk: chunk_index,
                    start: idx,
                    old_len: pattern.len(),
                    new: new_slice.to_vec(),
                });
                line_index = idx + pattern.len();
                chunk_worst = worse_of(chunk_worst, quality);
            }
            SeekOutcome::Ambiguous { matches, .. } => {
                attempts.push(AnchorAttempt {
                    file_path: file_rel.to_string(),
                    chunk_index,
                    anchor_text: chunk.change_context.clone(),
                    success: false,
                    fuzzy_tier: None,
                });
                return Err(ChunkFailure::Ambiguous {
                    chunk_index,
                    change_context: chunk.change_context.clone(),
                    candidates: one_based(&matches),
                });
            }
            SeekOutcome::NotFound => {
                attempts.push(AnchorAttempt {
                    file_path: file_rel.to_string(),
                    chunk_index,
                    anchor_text: chunk.change_context.clone(),
                    success: false,
                    fuzzy_tier: None,
                });
                return Err(ChunkFailure::NotFound {
                    chunk_index,
                    change_context: chunk.change_context.clone(),
                    first_old_line: chunk.old_lines.first().cloned(),
                    near_miss: near_miss_snippet(
                        original_lines,
                        line_index,
                        chunk.old_lines.first().map(String::as_str),
                    ),
                });
            }
        }

        attempts.push(AnchorAttempt {
            file_path: file_rel.to_string(),
            chunk_index,
            anchor_text: chunk.change_context.clone(),
            success: true,
            fuzzy_tier: Some(chunk_worst.as_str().to_string()),
        });

        if chunk_worst != MatchQuality::Exact {
            fuzzy_hunks.push(HunkFuzzy {
                chunk: chunk_index,
                tier: chunk_worst,
            });
        }
    }

    replacements.sort_by_key(|r| r.start);
    Ok((replacements, fuzzy_hunks))
}

fn worse_of(a: MatchQuality, b: MatchQuality) -> MatchQuality {
    let rank = |q: MatchQuality| match q {
        MatchQuality::Exact => 0,
        MatchQuality::TrimEnd => 1,
        MatchQuality::Trim => 2,
        MatchQuality::Normalized => 3,
    };
    if rank(a) >= rank(b) { a } else { b }
}

fn one_based(zero: &[usize]) -> Vec<usize> {
    zero.iter().map(|&i| i + 1).collect()
}

/// Numbered window of the original file, centered on the closest fuzzy match
/// for `expected` (the line the patch thought it was removing). Falls back to
/// `cursor` when `expected` is absent or too dissimilar. Included in
/// ContextNotFound errors so callers can regenerate the patch without
/// re-reading the file.
fn near_miss_snippet(
    original_lines: &[String],
    cursor: usize,
    expected: Option<&str>,
) -> Option<String> {
    if original_lines.is_empty() {
        return None;
    }
    const LEAD: usize = 5;
    const WINDOW: usize = 14;

    // Trust the closest match only when it's plausibly the same line — within
    // half the expected character length, floor 3. Beyond that the "closest"
    // line is probably unrelated and would mis-center the window.
    let closest = expected.filter(|e| !e.is_empty()).and_then(|exp| {
        let (idx, dist) = closest_line(exp, original_lines)?;
        let budget = exp.chars().count().div_ceil(2).max(3);
        (dist <= budget).then_some((idx, dist))
    });

    let center = closest.map(|(idx, _)| idx).unwrap_or(cursor);
    let start = center.saturating_sub(LEAD);
    let end = (center + WINDOW).min(original_lines.len());
    if start >= end {
        return None;
    }

    let width = ((end as f64).log10().floor() as usize) + 1;
    let mut out = String::from("\nfile state");
    if let Some((idx, dist)) = closest {
        out.push_str(&format!(
            " (closest match: line {} at edit distance {})",
            idx + 1,
            dist,
        ));
    }
    out.push_str(":\n");
    for (offset, line) in original_lines[start..end].iter().enumerate() {
        out.push_str(&format!(
            "{:>width$}: {}\n",
            start + offset + 1,
            line,
            width = width,
        ));
    }
    Some(out)
}

/// Scan `lines` for the one with smallest Levenshtein distance to `expected`.
/// Prunes by length gap — if `|len(a) - len(b)| ≥ best_dist`, no way this line
/// is closer than the current best, so skip the full DP.
fn closest_line(expected: &str, lines: &[String]) -> Option<(usize, usize)> {
    if lines.is_empty() {
        return None;
    }
    let exp_len = expected.chars().count();
    let mut best: Option<(usize, usize)> = None;
    for (idx, line) in lines.iter().enumerate() {
        let line_len = line.chars().count();
        let len_gap = exp_len.abs_diff(line_len);
        if let Some((_, best_dist)) = best
            && len_gap >= best_dist
        {
            continue;
        }
        let dist = levenshtein(expected, line);
        if best.is_none_or(|(_, bd)| dist < bd) {
            best = Some((idx, dist));
            if dist == 0 {
                return best;
            }
        }
    }
    best
}

/// Character-wise Levenshtein distance. Two-row DP — O(m·n) time, O(n) space.
fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let (m, n) = (a_chars.len(), b_chars.len());
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr: Vec<usize> = vec![0; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = usize::from(a_chars[i - 1] != b_chars[j - 1]);
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

fn chunk_failure_to_error(fail: ChunkFailure, path: &str) -> ApplyPatchError {
    match fail {
        ChunkFailure::NotFound {
            chunk_index,
            change_context,
            first_old_line,
            near_miss,
        } => ApplyPatchError::ContextNotFound {
            path: path.to_string(),
            chunk: chunk_index,
            change_context,
            first_old_line,
            near_miss,
        },
        ChunkFailure::Ambiguous {
            chunk_index,
            change_context,
            candidates,
        } => ApplyPatchError::AmbiguousContext {
            path: path.to_string(),
            chunk: chunk_index,
            change_context,
            candidates,
        },
        ChunkFailure::AnchorShadowsFirstContext {
            chunk_index,
            anchor,
        } => ApplyPatchError::AnchorShadowsFirstContext {
            path: path.to_string(),
            chunk: chunk_index,
            anchor,
        },
    }
}

fn apply_replacements(mut lines: Vec<String>, replacements: Replacements) -> Vec<String> {
    // Iterate in reverse so earlier edits' offsets stay valid as later ones
    // land. `splice` is O(n) once per hunk (vs O(k·n) for remove+insert) and
    // moves owned `String`s into place without cloning.
    for r in replacements.into_iter().rev() {
        let start = r.start.min(lines.len());
        let end = (r.start + r.old_len).min(lines.len());
        lines.splice(start..end, r.new);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn resolve_path_accepts_relative_and_absolute() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("foo");
        fs::create_dir_all(&sub).unwrap();
        let canon = tmp.path().canonicalize().unwrap();

        let via_relative = resolve_path(&canon, Path::new("foo/bar.rs")).unwrap();
        assert!(via_relative.starts_with(&canon));
        assert!(via_relative.ends_with("bar.rs"));

        let abs = canon.join("zed.txt");
        let via_absolute = resolve_path(&canon, &abs).unwrap();
        assert_eq!(via_absolute, abs);
    }

    #[test]
    fn plan_add_returns_add_change() {
        let tmp = TempDir::new().unwrap();
        let patch = "*** Begin Patch\n*** Add File: hello.txt\n+hi\n+world\n*** End Patch\n";
        let outcome = plan(patch, tmp.path()).unwrap();
        let changes = &outcome.changes;
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

        let outcome = plan(patch, tmp.path()).unwrap();
        let changes = outcome.changes;
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
        let outcome = plan(patch, tmp.path()).unwrap();
        let changes = &outcome.changes;
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

    #[test]
    fn post_apply_region_echoes_window_around_each_hunk() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("lib.rs"),
            "fn one() {}\nfn two() {}\nfn three() {}\nfn four() {}\nfn five() {}\n",
        )
        .unwrap();
        let patch = concat!(
            "*** Begin Patch\n",
            "*** Update File: lib.rs\n",
            "@@\n",
            " fn two() {}\n",
            "-fn three() {}\n",
            "+fn THREE() {}\n",
            " fn four() {}\n",
            "*** End Patch\n",
        );

        let outcome = plan(patch, tmp.path()).unwrap();
        let changes = &outcome.changes;
        let regions = &changes[0].post_apply_regions;
        assert_eq!(regions.len(), 1, "one hunk → one region");
        assert_eq!(regions[0].chunk, 0);
        assert!(
            regions[0].lines.iter().any(|l| l == "fn THREE() {}"),
            "post-apply window must include the new line, got: {:?}",
            regions[0].lines,
        );
        // `start_line` is 1-based — verify the first echoed line really sits
        // at that file position after apply.
        let expected_first = &regions[0].lines[0];
        let new_content = changes[0].new_content.as_deref().unwrap();
        let new_lines: Vec<&str> = new_content.lines().collect();
        assert_eq!(new_lines[regions[0].start_line - 1], expected_first);
    }

    #[test]
    fn near_miss_hints_closest_line_via_edit_distance() {
        // "println!(\"nope\")" is a 5-edit mutation of "println!(\"hello\")";
        // the near-miss hint should center on line 2 and name its distance.
        let lines: Vec<String> = ["fn main() {", "    println!(\"hello\");", "}"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let hint = near_miss_snippet(&lines, 0, Some("    println!(\"nope\");")).unwrap();
        assert!(hint.contains("closest match: line 2"), "{hint}");
        assert!(hint.contains("edit distance 5"), "{hint}");
        assert!(hint.contains("println!(\"hello\")"), "{hint}");
    }

    #[test]
    fn near_miss_falls_back_to_cursor_when_no_close_match() {
        let lines: Vec<String> = (0..10).map(|i| format!("unrelated line {i}")).collect();
        let hint = near_miss_snippet(&lines, 5, Some("totally different content here")).unwrap();
        // Distance is huge → no closest-match header; window stays around cursor.
        assert!(!hint.contains("closest match"), "{hint}");
        assert!(hint.contains("unrelated line 5"), "{hint}");
    }

    #[test]
    fn anchor_shadowing_first_context_returns_specific_error() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("m.rs"),
            "fn greet() {\n    println!(\"hi\");\n}\n",
        )
        .unwrap();
        // The offender: `@@ fn greet() {` followed by ` fn greet() {` as first
        // context line. Anchor is consumed, then the pattern search tries to
        // find `fn greet() {` on the next line — fails. Before this check it
        // surfaced as a confusing "context not found"; now the error names it.
        let patch = concat!(
            "*** Begin Patch\n",
            "*** Update File: m.rs\n",
            "@@ fn greet() {\n",
            " fn greet() {\n",
            "-    println!(\"hi\");\n",
            "+    println!(\"hello\");\n",
            " }\n",
            "*** End Patch\n",
        );
        let err = plan(patch, tmp.path()).unwrap_err().error;
        match err {
            ApplyPatchError::AnchorShadowsFirstContext { anchor, chunk, .. } => {
                assert_eq!(anchor, "fn greet() {");
                assert_eq!(chunk, 0);
            }
            other => panic!("expected AnchorShadowsFirstContext, got: {other:?}"),
        }
    }

    #[test]
    fn zero_context_unique_removal_applies() {
        // The `-` line alone is distinctive enough to be unique in the file,
        // so no context is needed. Keeps hunks tight and reduces drift risk.
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("code.rs"),
            "fn a() {}\nfn UNIQUE_TARGET() {}\nfn b() {}\n",
        )
        .unwrap();
        let patch = concat!(
            "*** Begin Patch\n",
            "*** Update File: code.rs\n",
            "@@\n",
            "-fn UNIQUE_TARGET() {}\n",
            "+fn RENAMED() {}\n",
            "*** End Patch\n",
        );
        let outcome = plan(patch, tmp.path()).unwrap();
        let changes = &outcome.changes;
        assert_eq!(
            changes[0].new_content.as_deref(),
            Some("fn a() {}\nfn RENAMED() {}\nfn b() {}\n"),
        );
    }

    #[test]
    fn zero_context_ambiguous_removal_rejected_with_candidates() {
        // Same shape, but the `-` line occurs twice — ambiguity detection
        // kicks in and reports both line numbers so the caller can add one
        // line of context instead of re-reading the file.
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("code.rs"),
            "let x = 1;\nlet y = 2;\nlet x = 1;\n",
        )
        .unwrap();
        let patch = concat!(
            "*** Begin Patch\n",
            "*** Update File: code.rs\n",
            "@@\n",
            "-let x = 1;\n",
            "+let z = 3;\n",
            "*** End Patch\n",
        );
        let err = plan(patch, tmp.path()).unwrap_err().error;
        match err {
            ApplyPatchError::AmbiguousContext { candidates, .. } => {
                assert_eq!(candidates, vec![1, 3]);
            }
            other => panic!("expected AmbiguousContext, got: {other:?}"),
        }
    }

    #[test]
    fn single_context_line_resolves_ambiguity() {
        // Neither `    return;` nor `pub fn two() {` alone is unique, but the
        // pair is — one context line is enough to pin the edit.
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("code.rs"),
            "pub fn one() {}\npub fn two() {\n    return;\n}\npub fn three() {\n    return;\n}\n",
        )
        .unwrap();
        let patch = concat!(
            "*** Begin Patch\n",
            "*** Update File: code.rs\n",
            "@@\n",
            " pub fn two() {\n",
            "-    return;\n",
            "+    return 42;\n",
            "*** End Patch\n",
        );
        let outcome = plan(patch, tmp.path()).unwrap();
        let changes = &outcome.changes;
        let content = changes[0].new_content.as_deref().unwrap();
        assert!(
            content.contains("pub fn two() {\n    return 42;\n}"),
            "hunk must land in fn two, got: {content}",
        );
        assert!(
            content.contains("pub fn three() {\n    return;\n}"),
            "fn three must stay untouched, got: {content}",
        );
    }
}
