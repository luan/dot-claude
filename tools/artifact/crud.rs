use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use serde::Serialize;

use super::{
    ALL_KINDS, ArtifactKind, CtError, ResolveError, SyncError, artifact_dir, blueprints_dir,
    chrono_compact, chrono_rfc3339, commit_and_push, commit_and_push_paths, ensure_in_vault, fatal,
    parse_frontmatter, parse_frontmatter_fields, parse_yaml_map, project_name, resolve_repo_root,
    strip_date_prefix, validate_tag, yaml_quote,
};

// ---------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------

pub struct CreateOpts<'a> {
    pub kind: ArtifactKind,
    pub topic: &'a str,
    pub project: &'a str,
    pub slug_override: Option<&'a str>,
    pub source: Option<&'a str>,
    pub user_tags: &'a [String],
    pub dive: bool,
}

/// Result of a successful `create` call. `pushed = false` means the commit
/// stayed local — push failures propagate as `CtError::Sync` instead.
#[derive(Debug, Clone, Serialize)]
pub struct CreateOutcome {
    pub path: PathBuf,
    pub project: String,
    pub kind: ArtifactKind,
    pub pushed: bool,
}

pub fn create(opts: CreateOpts<'_>) -> Result<CreateOutcome, CtError> {
    let CreateOpts {
        kind,
        topic,
        project,
        slug_override,
        source,
        user_tags,
        dive,
    } = opts;
    if dive && kind != ArtifactKind::Spec {
        return Err(CtError::Validation(
            "dive is only valid for spec artifacts".to_string(),
        ));
    }
    if dive && source.is_none() {
        return Err(CtError::Validation(
            "dive requires source (a dive without a hub link is meaningless)".to_string(),
        ));
    }
    for tag in user_tags {
        validate_tag(tag)?;
    }
    let project = &resolve_repo_root(project);
    // slug_override is MCP-client-supplied; sanitize through the same filter
    // the topic path uses so `"../evil"` can't land outside the vault.
    let s = match slug_override {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(CtError::Validation(
                    "slug override is empty after trimming".to_string(),
                ));
            }
            let sanitized = crate::slug::slug(trimmed);
            if sanitized.is_empty() {
                return Err(CtError::Validation(format!(
                    "slug override {raw:?} sanitizes to empty; pick a different slug"
                )));
            }
            sanitized
        }
        None => crate::slug::slug(topic),
    };
    if s.is_empty() {
        return Err(CtError::Validation(
            "could not derive slug from topic".to_string(),
        ));
    }

    let filename = if kind == ArtifactKind::Doc {
        format!("{s}.md")
    } else {
        let ts = chrono_compact();
        format!("{ts}-{s}.md")
    };

    let bp = blueprints_dir();
    let proj_name_for_dir = project_name(project);
    let dir = if dive && kind == ArtifactKind::Spec {
        bp.join(&proj_name_for_dir).join("dive")
    } else {
        artifact_dir(project, kind)
    };
    fs::create_dir_all(&dir)?;

    let full_path = dir.join(&filename);
    // Timestamp prefix is hour-precision — two creates in the same hour with
    // the same slug produce the same filename, so refuse to clobber.
    if full_path.exists() {
        return Err(CtError::Validation(format!(
            "artifact already exists: {}",
            full_path.display()
        )));
    }

    let now = chrono_rfc3339();

    let mut buf = String::new();
    buf.push_str("---\n");
    buf.push_str(&format!("topic: {}\n", yaml_quote(topic)));
    buf.push_str(&format!("created: {now}\n"));
    let author = env::var("GIT_USERNAME")
        .unwrap_or_else(|_| env::var("USER").unwrap_or_else(|_| "unknown".to_string()));
    buf.push_str(&format!("author: {}\n", yaml_quote(&author)));
    if let Some(src) = source {
        buf.push_str(&format!("source: {}\n", yaml_quote(&format!("[[{src}]]"))));
    }
    // Tags: auto-derive type/ and project/, merge with user-supplied
    let proj_name = project_name(project);
    let mut tags = vec![
        format!("type/{}", kind.dir_name()),
        format!("project/{proj_name}"),
    ];
    for t in user_tags {
        if !tags.contains(t) {
            tags.push(t.clone());
        }
    }
    buf.push_str("tags:\n");
    for tag in &tags {
        buf.push_str(&format!("  - {tag}\n"));
    }
    buf.push_str("---\n");

    fs::write(&full_path, &buf)?;

    // Push failure is a partial success: the commit exists locally but was not
    // propagated, so surface that via `pushed: false` instead of an error
    // (retries would create orphan commits with new timestamps).
    let mut pushed = true;
    if let Ok(rel) = full_path.strip_prefix(&bp) {
        let msg = format!("{}({}): {}", kind.commit_name(), proj_name, s);
        match commit_and_push(rel, &msg) {
            Ok(()) => {}
            Err(SyncError::Push(_)) => pushed = false,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(CreateOutcome {
        path: full_path,
        project: proj_name,
        kind,
        pushed,
    })
}

// ---------------------------------------------------------------------------
// Path resolution
// ---------------------------------------------------------------------------

/// Resolve an artifact file argument to a real path.
/// Accepts: absolute/relative path, `project/kind/stem`, or bare stem.
/// Every successful return is canonicalized and verified to live under the
/// vault — path-traversal arguments fail with `ResolveError::NotFound`.
pub fn resolve_artifact_path(file_arg: &str, kind: ArtifactKind) -> Result<PathBuf, ResolveError> {
    let p = Path::new(file_arg);
    if p.exists() {
        return ensure_in_vault(p).map_err(|_| ResolveError::NotFound(file_arg.to_string()));
    }
    let with_ext = p.with_extension("md");
    if with_ext.exists() {
        return ensure_in_vault(&with_ext)
            .map_err(|_| ResolveError::NotFound(file_arg.to_string()));
    }

    let bp = blueprints_dir();
    let kind_dir = kind.dir_name();

    // Try as project-relative: <project>/<kind>/stem[.md]
    let bp_path = bp.join(file_arg);
    if bp_path.exists() {
        return ensure_in_vault(&bp_path).map_err(|_| ResolveError::NotFound(file_arg.to_string()));
    }
    let bp_path_ext = bp_path.with_extension("md");
    if bp_path_ext.exists() {
        return ensure_in_vault(&bp_path_ext)
            .map_err(|_| ResolveError::NotFound(file_arg.to_string()));
    }

    // Bare stem — scan all projects (include dive/ for Spec).
    let stem = p.file_stem().unwrap_or(p.as_os_str());
    let query_slug = stem.to_str().map(strip_date_prefix);
    let mut matches = Vec::new();
    let mut fuzzy_matches = Vec::new();
    if let Ok(entries) = fs::read_dir(&bp) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let scan_dirs: Vec<PathBuf> = if kind == ArtifactKind::Spec {
                vec![entry.path().join(kind_dir), entry.path().join("dive")]
            } else {
                vec![entry.path().join(kind_dir)]
            };
            for candidate in scan_dirs {
                if !candidate.is_dir() {
                    continue;
                }
                if let Ok(files) = fs::read_dir(&candidate) {
                    for f in files.flatten() {
                        let fp = f.path();
                        if fp.extension().and_then(|e| e.to_str()) != Some("md") {
                            continue;
                        }
                        if fp.file_stem() == Some(stem) {
                            matches.push(fp);
                            continue;
                        }
                        if let Some(candidate_stem) = fp.file_stem().and_then(|s| s.to_str()) {
                            let candidate_slug = strip_date_prefix(candidate_stem);
                            let stem_str = stem.to_str().unwrap_or("");
                            if candidate_slug == stem_str
                                || query_slug.is_some_and(|qs| qs == candidate_stem)
                            {
                                fuzzy_matches.push(fp);
                            }
                        }
                    }
                }
            }
        }
    }

    let resolved = if matches.is_empty() {
        &mut fuzzy_matches
    } else {
        &mut matches
    };
    match resolved.len() {
        0 => Err(ResolveError::NotFound(file_arg.to_string())),
        1 => {
            let hit = resolved.remove(0);
            ensure_in_vault(&hit).map_err(|_| ResolveError::NotFound(file_arg.to_string()))
        }
        _ => Err(ResolveError::Ambiguous(std::mem::take(resolved))),
    }
}

/// Resolve a bare stem across ALL artifact kinds in priority order:
/// Doc > Report > Review > Plan > Spec.
pub fn resolve_stem_universal(stem: &str) -> Result<PathBuf, ResolveError> {
    let p = Path::new(stem);
    if p.exists() {
        return ensure_in_vault(p).map_err(|_| ResolveError::NotFound(stem.to_string()));
    }
    let with_ext = p.with_extension("md");
    if with_ext.exists() {
        return ensure_in_vault(&with_ext).map_err(|_| ResolveError::NotFound(stem.to_string()));
    }

    let bp = blueprints_dir();

    let bp_path = bp.join(stem);
    if bp_path.exists() {
        return ensure_in_vault(&bp_path).map_err(|_| ResolveError::NotFound(stem.to_string()));
    }
    let bp_path_ext = bp_path.with_extension("md");
    if bp_path_ext.exists() {
        return ensure_in_vault(&bp_path_ext).map_err(|_| ResolveError::NotFound(stem.to_string()));
    }

    let file_stem = p.file_stem().unwrap_or(p.as_os_str());
    let query_slug = file_stem.to_str().map(strip_date_prefix);
    let mut matches: Vec<(ArtifactKind, PathBuf)> = Vec::new();
    let mut fuzzy_matches: Vec<(ArtifactKind, PathBuf)> = Vec::new();

    if let Ok(projects) = fs::read_dir(&bp) {
        for proj_entry in projects.flatten() {
            if !proj_entry.path().is_dir() {
                continue;
            }
            for &kind in &ALL_KINDS {
                let candidate_dir = proj_entry.path().join(kind.dir_name());
                if !candidate_dir.is_dir() {
                    continue;
                }
                if let Ok(files) = fs::read_dir(&candidate_dir) {
                    for f in files.flatten() {
                        let fp = f.path();
                        if fp.extension().and_then(|e| e.to_str()) != Some("md") {
                            continue;
                        }
                        if fp.file_stem() == Some(file_stem) {
                            matches.push((kind, fp));
                            continue;
                        }
                        if let Some(candidate_stem) = fp.file_stem().and_then(|s| s.to_str()) {
                            let candidate_slug = strip_date_prefix(candidate_stem);
                            let stem_str = file_stem.to_str().unwrap_or("");
                            if candidate_slug == stem_str
                                || query_slug.is_some_and(|qs| qs == candidate_stem)
                            {
                                fuzzy_matches.push((kind, fp));
                            }
                        }
                    }
                }
            }
        }
    }

    let resolved = if matches.is_empty() {
        &mut fuzzy_matches
    } else {
        &mut matches
    };

    if resolved.is_empty() {
        return Err(ResolveError::NotFound(stem.to_string()));
    }

    if resolved.len() == 1 {
        let hit = resolved.remove(0).1;
        return ensure_in_vault(&hit).map_err(|_| ResolveError::NotFound(stem.to_string()));
    }

    // Multiple matches — return highest-priority kind if they span kinds.
    let kinds_seen: HashSet<&str> = resolved.iter().map(|(k, _)| k.dir_name()).collect();
    if kinds_seen.len() > 1 {
        for &kind in &ALL_KINDS {
            if let Some(pos) = resolved.iter().position(|(k, _)| *k == kind) {
                let hit = resolved.remove(pos).1;
                return ensure_in_vault(&hit).map_err(|_| ResolveError::NotFound(stem.to_string()));
            }
        }
    }

    // Same kind, different projects — ambiguous.
    Err(ResolveError::Ambiguous(
        std::mem::take(resolved)
            .into_iter()
            .map(|(_, p)| p)
            .collect(),
    ))
}

// ---------------------------------------------------------------------------
// Read
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct Frontmatter {
    pub topic: Option<String>,
    pub created: Option<String>,
    pub author: Option<String>,
    pub source: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadOutcome {
    pub path: PathBuf,
    pub frontmatter: Option<Frontmatter>,
    pub body: String,
    pub comments: Vec<Comment>,
}

/// Read an artifact file and parse its frontmatter + body + inline comments.
pub fn read(path: &Path) -> Result<ReadOutcome, CtError> {
    let content = fs::read_to_string(path)?;
    let (yaml, body) = parse_frontmatter(&content);
    let frontmatter = yaml.map(|y| {
        let (title, _, created, source, tags, author) = parse_frontmatter_fields(y);
        Frontmatter {
            topic: if title.is_empty() { None } else { Some(title) },
            created,
            author,
            source,
            tags,
        }
    });
    let comments = parse_comments(body);
    Ok(ReadOutcome {
        path: path.to_path_buf(),
        frontmatter,
        body: body.to_string(),
        comments,
    })
}

pub fn cmd_read(file_path: &str, kind: ArtifactKind, frontmatter_mode: bool) {
    let resolved = match resolve_artifact_path(file_path, kind) {
        Ok(p) => p,
        Err(e) => fatal(&e.to_string()),
    };
    cmd_read_resolved(&resolved, frontmatter_mode);
}

/// Read and print an artifact from a resolved path (no kind needed).
pub fn cmd_read_resolved(resolved: &Path, frontmatter_mode: bool) {
    // Frontmatter-as-JSON mode preserves the raw YAML key order from the file
    // (downstream skills parse this line).
    if frontmatter_mode {
        let content =
            fs::read_to_string(resolved).unwrap_or_else(|e| fatal(&format!("reading file: {e}")));
        let (yaml, _) = parse_frontmatter(&content);
        match yaml {
            None => println!("{{}}"),
            Some(y) => {
                let pairs = parse_yaml_map(y);
                print!("{{");
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        print!(",");
                    }
                    let k_escaped = k.replace('\\', "\\\\").replace('"', "\\\"");
                    let v_escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
                    print!("\"{k_escaped}\":\"{v_escaped}\"");
                }
                println!("}}");
            }
        }
    } else {
        let outcome = match read(resolved) {
            Ok(o) => o,
            Err(e) => fatal(&e.to_string()),
        };
        print!("{}", outcome.body);
    }
}

// ---------------------------------------------------------------------------
// Rename
// ---------------------------------------------------------------------------

pub fn cmd_rename(kind: ArtifactKind, old_arg: &str, new_slug: &str) -> Result<(), SyncError> {
    let old_path = match resolve_artifact_path(old_arg, kind) {
        Ok(p) => p,
        Err(e) => fatal(&e.to_string()),
    };
    let bp = blueprints_dir();

    let rel_path = old_path
        .strip_prefix(&bp)
        .unwrap_or_else(|_| fatal(&format!("file is not inside {}", bp.display())));
    let proj_name = rel_path
        .components()
        .next()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .unwrap_or_else(|| fatal("cannot determine project from file path"));

    if new_slug.contains('/') || new_slug.contains('\\') || new_slug.contains("..") {
        fatal("new slug must not contain path separators or '..'");
    }

    let old_stem = old_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Compute new filename — Doc kind has no timestamp prefix
    let new_filename = if kind == ArtifactKind::Doc {
        format!("{new_slug}.md")
    } else {
        let stripped = strip_date_prefix(&old_stem);
        if stripped.len() < old_stem.len() {
            let prefix = &old_stem[..old_stem.len() - stripped.len()];
            format!("{prefix}{new_slug}.md")
        } else {
            format!("{new_slug}.md")
        }
    };

    let new_path = old_path.parent().unwrap().join(&new_filename);

    if new_path.exists() {
        fatal(&format!("target already exists: {}", new_path.display()));
    }

    // Warn about incoming wiki-links to the old stem.
    let link_pattern = format!("[[{old_stem}]]");
    if let Ok(output) = process::Command::new("rg")
        .args(["-lF", &link_pattern])
        .arg(bp.as_os_str())
        .output()
        && output.status.success()
    {
        let hits = String::from_utf8_lossy(&output.stdout);
        let hits = hits.trim();
        if !hits.is_empty() {
            eprintln!("warning: incoming wiki-links to [[{old_stem}]] found in:");
            for line in hits.lines() {
                eprintln!("  {line}");
            }
        }
    }

    let content =
        fs::read_to_string(&old_path).unwrap_or_else(|e| fatal(&format!("reading file: {e}")));
    let (yaml, _body) = parse_frontmatter(&content);

    let updated = if let Some(yaml_str) = yaml {
        // Frontmatter boundaries: "---\n" + yaml + "\n---\n"
        let fm_start = 4; // "---\n"
        let fm_end = fm_start + yaml_str.len();
        let raw_yaml = &content[fm_start..fm_end];

        let mut new_yaml = String::with_capacity(raw_yaml.len() + 64);
        let kind_tag_prefix = "type/";
        let proj_tag_prefix = "project/";
        let new_kind_tag = format!("type/{}", kind.dir_name());
        let new_proj_tag = format!("project/{proj_name}");

        for line in raw_yaml.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("topic:") {
                new_yaml.push_str(&format!("topic: {}", yaml_quote(new_slug)));
            } else if trimmed.starts_with("- type/") || trimmed.starts_with("- project/") {
                let tag = trimmed.strip_prefix("- ").unwrap_or(trimmed);
                let indent = &line[..line.len() - trimmed.len()];
                if tag.starts_with(kind_tag_prefix) {
                    new_yaml.push_str(&format!("{indent}- {new_kind_tag}"));
                } else if tag.starts_with(proj_tag_prefix) {
                    new_yaml.push_str(&format!("{indent}- {new_proj_tag}"));
                } else {
                    new_yaml.push_str(line);
                }
            } else {
                new_yaml.push_str(line);
            }
            new_yaml.push('\n');
        }

        // Original yaml slice has no trailing newline — mirror that.
        if new_yaml.ends_with('\n') {
            new_yaml.pop();
        }

        format!("---\n{new_yaml}{}", &content[fm_end..])
    } else {
        content.clone()
    };

    fs::write(&new_path, &updated).unwrap_or_else(|e| fatal(&format!("writing new file: {e}")));
    fs::remove_file(&old_path).unwrap_or_else(|e| fatal(&format!("removing old file: {e}")));

    let old_rel = old_path
        .strip_prefix(&bp)
        .unwrap_or_else(|_| fatal("cannot compute relative path for old file"));
    let new_rel = new_path
        .strip_prefix(&bp)
        .unwrap_or_else(|_| fatal("cannot compute relative path for new file"));

    let new_stem = new_path.file_stem().unwrap_or_default().to_string_lossy();
    let message = format!("rename({proj_name}): {old_stem} → {new_stem}");

    commit_and_push_paths(&[old_rel, new_rel], &message)
}

// ---------------------------------------------------------------------------
// Retag
// ---------------------------------------------------------------------------

pub fn cmd_retag(kind: ArtifactKind, file_arg: &str) -> Result<(), SyncError> {
    let bp = blueprints_dir();
    let resolved = match resolve_artifact_path(file_arg, kind) {
        Ok(p) => p,
        Err(e) => fatal(&e.to_string()),
    };
    let resolved_str = resolved.to_string_lossy();

    let content = fs::read_to_string(&resolved)
        .unwrap_or_else(|_| fatal(&format!("cannot read: {resolved_str}")));

    let (yaml, _) = parse_frontmatter(&content);
    if yaml.is_none() {
        fatal("no frontmatter found");
    }

    let rel_path = resolved
        .strip_prefix(&bp)
        .unwrap_or_else(|_| fatal(&format!("file is not inside {}", bp.display())));
    let proj_name = rel_path
        .components()
        .next()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .unwrap_or_else(|| fatal("cannot determine project from file path"));

    let correct_type_tag = format!("type/{}", kind.dir_name());
    let correct_project_tag = format!("project/{proj_name}");

    let mut changed = false;
    let mut in_frontmatter = false;
    let mut past_first_delim = false;
    let mut result_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        if line == "---" {
            if !past_first_delim {
                past_first_delim = true;
                in_frontmatter = true;
            } else {
                in_frontmatter = false;
            }
            result_lines.push(line.to_string());
            continue;
        }

        if in_frontmatter {
            if let Some(rest) = line.strip_prefix("  - type/") {
                let new_line = format!("  - {correct_type_tag}");
                if rest != kind.dir_name() {
                    changed = true;
                }
                result_lines.push(new_line);
                continue;
            }
            if line.starts_with("  - project/") {
                let new_line = format!("  - {correct_project_tag}");
                if line != format!("  - {correct_project_tag}") {
                    changed = true;
                }
                result_lines.push(new_line);
                continue;
            }
        }

        result_lines.push(line.to_string());
    }

    if !changed {
        eprintln!("tags already correct");
        return Ok(());
    }

    let mut output = result_lines.join("\n");
    if content.ends_with('\n') {
        output.push('\n');
    }

    fs::write(&resolved, &output)
        .unwrap_or_else(|e| fatal(&format!("cannot write {resolved_str}: {e}")));

    let stem = resolved.file_stem().unwrap_or_default().to_string_lossy();
    let rel = resolved.strip_prefix(&bp).unwrap_or(resolved.as_path());
    commit_and_push(rel, &format!("retag({proj_name}): {stem}"))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Inline comment extraction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Comment {
    pub line: usize,
    pub highlight: Option<String>,
    #[serde(rename = "comment")]
    pub text: String,
}

/// Extract all single-line HTML comments from a markdown body.
pub fn parse_comments(body: &str) -> Vec<Comment> {
    let mut out = Vec::new();
    for (idx, line) in body.lines().enumerate() {
        let line_no = idx + 1;
        let mut rest = line;
        while let Some(start) = rest.find("<!--") {
            let before = &rest[..start];
            let after_open = &rest[start + 4..];
            let Some(end) = after_open.find("-->") else {
                break;
            };
            let comment_text = after_open[..end].trim().to_string();

            let highlight = extract_highlight(before);

            out.push(Comment {
                line: line_no,
                highlight,
                text: comment_text,
            });

            rest = &after_open[end + 3..];
        }
    }
    out
}

/// Look for a trailing `==...==` in the text immediately before a comment marker.
fn extract_highlight(before: &str) -> Option<String> {
    let trimmed = before.trim_end();
    if !trimmed.ends_with("==") {
        return None;
    }
    let inner = &trimmed[..trimmed.len() - 2];
    let start = inner.rfind("==")?;
    let text = &inner[start + 2..];
    if text.is_empty() {
        return None;
    }
    Some(text.to_string())
}

/// Count how many lines the frontmatter occupies (including delimiters).
fn frontmatter_line_count(content: &str) -> usize {
    match parse_frontmatter(content) {
        (None, _) => 0,
        (Some(yaml), _) => yaml.lines().count() + 2,
    }
}

pub fn cmd_comments(file_path: &str, kind: ArtifactKind, json: bool) {
    let resolved = match resolve_artifact_path(file_path, kind) {
        Ok(p) => p,
        Err(e) => fatal(&e.to_string()),
    };
    let content = fs::read_to_string(&resolved)
        .unwrap_or_else(|e| fatal(&format!("cannot read {}: {e}", resolved.display())));

    let fm_lines = frontmatter_line_count(&content);
    let (_, body) = parse_frontmatter(&content);
    let mut comments = parse_comments(body);

    let file_display = resolved
        .file_name()
        .unwrap_or(resolved.as_os_str())
        .to_string_lossy();

    for c in &mut comments {
        c.line += fm_lines;
    }

    if json {
        #[derive(serde::Serialize)]
        struct JsonComment<'a> {
            file: &'a str,
            #[serde(flatten)]
            comment: &'a Comment,
        }
        let entries: Vec<_> = comments
            .iter()
            .map(|c| JsonComment {
                file: &file_display,
                comment: c,
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string(&entries).unwrap_or_else(|e| fatal(&format!("json: {e}")))
        );
    } else {
        for c in &comments {
            match &c.highlight {
                Some(h) => println!("{file_display}:{}: [{h}] {}", c.line, c.text),
                None => println!("{file_display}:{}: {}", c.line, c.text),
            }
        }
    }
}
