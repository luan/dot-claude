use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::process;
use std::time::SystemTime;

pub(crate) fn fatal(msg: &str) -> ! {
    eprintln!("artifact: {msg}");
    process::exit(1);
}

pub(crate) fn home_dir() -> String {
    env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| fatal("cannot determine home directory"))
}

// ---------------------------------------------------------------------------
// ArtifactKind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    Plan,
    Spec,
    Review,
    Report,
    Doc,
}

/// Priority order for universal stem resolution: Doc > Report > Review > Plan > Spec.
pub const ALL_KINDS: [ArtifactKind; 5] = [
    ArtifactKind::Doc,
    ArtifactKind::Report,
    ArtifactKind::Review,
    ArtifactKind::Plan,
    ArtifactKind::Spec,
];

impl ArtifactKind {
    pub fn dir_name(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Spec => "spec",
            Self::Review => "review",
            Self::Report => "report",
            Self::Doc => "docs",
        }
    }

    pub fn notes_ref(self) -> &'static str {
        match self {
            Self::Plan => "plans",
            Self::Spec => "specs",
            Self::Review => "reviews",
            Self::Report => "reports",
            Self::Doc => "docs",
        }
    }

    /// Legacy directory name used in ~/.claude/ (before blueprints migration).
    pub fn legacy_dir_name(self) -> &'static str {
        match self {
            Self::Plan => "plans",
            Self::Spec => "specs",
            Self::Review => "reviews",
            Self::Report => "reports",
            Self::Doc => "docs",
        }
    }
}

// ---------------------------------------------------------------------------
// Artifact struct (replaces Plan and Spec)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Artifact {
    pub name: String,
    pub path: PathBuf,
    pub title: String,
    pub project: String,
    pub mod_time: SystemTime,
    pub size: u64,
}

// ---------------------------------------------------------------------------
// Blueprints directory
// ---------------------------------------------------------------------------

/// Returns the blueprints vault directory or fatal if it doesn't exist.
///
/// Resolution order: `CT_BLUEPRINTS_DIR` env var, then `~/blueprints/`.
pub fn blueprints_dir() -> PathBuf {
    let dir = blueprints_dir_unchecked();
    if !dir.is_dir() {
        fatal(&format!(
            "{} does not exist. Run `ct vault init` first.",
            dir.display()
        ));
    }
    dir
}

/// Returns the blueprints vault directory without checking existence (for init).
pub fn blueprints_dir_unchecked() -> PathBuf {
    if let Ok(custom) = env::var("CT_BLUEPRINTS_DIR") {
        return PathBuf::from(custom);
    }
    let home = home_dir();
    Path::new(&home).join("blueprints")
}

/// Returns <vault>/<project>/<kind>/
pub fn artifact_dir(project_path: &str, kind: ArtifactKind) -> PathBuf {
    let bp = blueprints_dir();
    let name = project_name(project_path);
    bp.join(name).join(kind.dir_name())
}

/// Test helper: artifact dir with custom base.
#[cfg(test)]
pub fn artifact_dir_with_base(project_path: &str, kind: &str, base: &Path) -> PathBuf {
    let name = project_name(project_path);
    base.join(name).join(kind)
}

// ---------------------------------------------------------------------------
// Worktree → main repo resolution
// ---------------------------------------------------------------------------

/// If `toplevel` is a git worktree, return the main repo root instead.
/// Uses `git rev-parse --git-common-dir` — when running inside a worktree it
/// returns an absolute path to the main repo's `.git` dir.
pub fn resolve_repo_root(toplevel: &str) -> String {
    let common = process::Command::new("git")
        .args(["-C", toplevel, "rev-parse", "--git-common-dir"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    if let Some(ref common_dir) = common {
        let p = Path::new(common_dir);
        // Absolute path means we're in a worktree; parent of `.git` is the main repo
        if let Some(parent) = p.parent().filter(|_| p.is_absolute()) {
            return parent.to_string_lossy().to_string();
        }
    }
    toplevel.to_string()
}

// ---------------------------------------------------------------------------
// Project name derivation
// ---------------------------------------------------------------------------

pub fn project_name(project_path: &str) -> String {
    if project_path.is_empty() {
        return String::from("(no project)");
    }
    let path = Path::new(project_path);
    let components: Vec<&str> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    for comp in &components {
        if comp.ends_with(".git") {
            // All worktrees of a repo share the same project name
            let name = comp.strip_suffix(".git").unwrap_or(comp);
            return name.replace('.', "_");
        }
    }

    let name = path
        .file_name()
        .unwrap_or_else(|| fatal("invalid project path"))
        .to_string_lossy();
    // Dots break Obsidian wiki-links and tag rendering
    name.replace('.', "_")
}

// ---------------------------------------------------------------------------
// YAML / frontmatter utilities
// ---------------------------------------------------------------------------

pub fn yaml_quote(s: &str) -> String {
    if s.contains(':')
        || s.contains('{')
        || s.contains('}')
        || s.contains('[')
        || s.contains(']')
        || s.contains('&')
        || s.contains('*')
        || s.contains('?')
        || s.contains('|')
        || s.contains('>')
        || s.contains('!')
        || s.contains('%')
        || s.contains('@')
        || s.contains('`')
        || s.contains('#')
        || s.contains(',')
        || s.contains('"')
        || s.contains('\'')
        || s.contains('\n')
        || s.contains('\\')
        || s.starts_with(' ')
        || s.ends_with(' ')
    {
        let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

pub fn parse_frontmatter(content: &str) -> (Option<&str>, &str) {
    let delim = "---\n";
    if !content.starts_with(delim) {
        return (None, content);
    }

    let rest = &content[delim.len()..];
    if let Some(end) = rest.find("\n---\n") {
        let yaml = &rest[..end];
        let body = &rest[end + 5..];
        (Some(yaml), body)
    } else if let Some(yaml) = rest.strip_suffix("\n---") {
        (Some(yaml), "")
    } else {
        (None, content)
    }
}

pub fn parse_yaml_map(yaml: &str) -> Vec<(String, String)> {
    yaml.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let idx = line.find(':')?;
            let key = line[..idx].trim().to_string();
            let mut val = line[idx + 1..].trim().to_string();
            if val.len() >= 2
                && ((val.starts_with('"') && val.ends_with('"'))
                    || (val.starts_with('\'') && val.ends_with('\'')))
            {
                val = val[1..val.len() - 1].to_string();
                val = val.replace("\\\"", "\"").replace("\\\\", "\\");
            }
            Some((key, val))
        })
        .collect()
}

struct DateTime {
    year: i64,
    month: u32,
    day: u32,
    hours: u32,
    minutes: u32,
    seconds: u32,
}

fn now_utc() -> DateTime {
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    let mut days = (secs / 86400) as i64;
    let day_secs = (secs % 86400) as u32;

    days += 719468;
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = (days - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };

    DateTime {
        year: if m <= 2 { y + 1 } else { y },
        month: m,
        day: d,
        hours: day_secs / 3600,
        minutes: (day_secs % 3600) / 60,
        seconds: day_secs % 60,
    }
}

pub fn chrono_rfc3339() -> String {
    let dt = now_utc();
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        dt.year, dt.month, dt.day, dt.hours, dt.minutes, dt.seconds
    )
}

/// Compact date+hour for filenames: YYYYMMDD-HH
pub fn chrono_compact() -> String {
    let dt = now_utc();
    format!("{:04}{:02}{:02}-{:02}", dt.year, dt.month, dt.day, dt.hours)
}

/// Split a git note that may contain multiple appended frontmatter documents.
pub fn split_notes(content: &str) -> Vec<String> {
    let mut docs = Vec::new();
    let mut current = String::new();
    let mut in_frontmatter = false;
    let mut seen_frontmatter = false;

    for line in content.lines() {
        if line.trim() == "---" {
            if !seen_frontmatter {
                in_frontmatter = true;
                seen_frontmatter = true;
                current.push_str(line);
                current.push('\n');
            } else if in_frontmatter {
                in_frontmatter = false;
                current.push_str(line);
                current.push('\n');
            } else {
                if !current.trim().is_empty() {
                    docs.push(std::mem::take(&mut current));
                }
                in_frontmatter = true;
                current.push_str(line);
                current.push('\n');
            }
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }
    if !current.trim().is_empty() {
        docs.push(current);
    }
    docs
}

// ---------------------------------------------------------------------------
// Formatting utilities (moved from plan.rs)
// ---------------------------------------------------------------------------

pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes}B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    }
}

pub fn format_date(time: SystemTime) -> String {
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs() as i64;

    let days = secs / 86400;
    let mut y = 1970i32;
    let mut remaining_days = days;

    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }

    let months_days: &[i64] = if is_leap(y) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 0;
    for (i, &md) in months_days.iter().enumerate() {
        if remaining_days < md {
            m = i;
            break;
        }
        remaining_days -= md;
    }

    let d = remaining_days + 1;
    let month_names = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let now_secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let now_days = now_secs / 86400;
    let mut current_year = 1970i32;
    let mut rd = now_days;
    loop {
        let diy = if is_leap(current_year) { 366 } else { 365 };
        if rd < diy {
            break;
        }
        rd -= diy;
        current_year += 1;
    }

    if y != current_year {
        format!("{} {y}", month_names[m])
    } else {
        format!("{} {d:02}", month_names[m])
    }
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

// ---------------------------------------------------------------------------
// Commit + push helper
// ---------------------------------------------------------------------------

/// Git sync failure — distinguishes add/commit (hard) from push (partial success).
#[derive(Debug)]
pub enum SyncError {
    Add(String),
    Commit(String),
    Push(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add(msg) => write!(f, "git add failed: {msg}"),
            Self::Commit(msg) => write!(f, "git commit failed: {msg}"),
            Self::Push(msg) => write!(f, "git push failed: {msg}"),
        }
    }
}

impl std::error::Error for SyncError {}

/// Commit and push a file in the blueprints repo.
pub fn commit_and_push(relative_path: &Path, message: &str) -> Result<(), SyncError> {
    let bp = blueprints_dir();
    let bp_str = bp.to_string_lossy();

    let add_ok = process::Command::new("git")
        .args(["-C", &bp_str, "add"])
        .arg(relative_path)
        .status()
        .is_ok_and(|s| s.success());

    if !add_ok {
        return Err(SyncError::Add(format!(
            "git add failed in {}",
            bp.display()
        )));
    }

    let commit_output = process::Command::new("git")
        .args(["-C", &bp_str, "commit", "-m", message])
        .output();

    match commit_output {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            if stderr.contains("nothing to commit") {
                return Ok(());
            }
            return Err(SyncError::Commit(
                stderr.trim().to_string(),
            ));
        }
        Err(e) => {
            return Err(SyncError::Commit(format!(
                "failed to run git commit in {}: {e}",
                bp.display()
            )));
        }
    }

    let push_ok = process::Command::new("git")
        .args(["-C", &bp_str, "push"])
        .status()
        .is_ok_and(|s| s.success());

    if !push_ok {
        return Err(SyncError::Push(format!(
            "commit saved locally in {}, push manually",
            bp.display()
        )));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Generic CRUD (replaces planfile.rs / specfile.rs)
// ---------------------------------------------------------------------------

pub fn cmd_create(
    kind: ArtifactKind,
    topic: &str,
    project: &str,
    slug_override: Option<&str>,
    source: Option<&str>,
    user_tags: &[String],
    mut body: String,
) -> Result<(), SyncError> {
    // Resolve worktree paths to the main repo root
    let project = &resolve_repo_root(project);
    let s = match slug_override {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => crate::slug::slug(topic),
    };
    if s.is_empty() {
        fatal("could not derive slug from topic");
    }

    let ts = chrono_compact();
    let filename = format!("{ts}-{s}.md");

    let dir = artifact_dir(project, kind);
    fs::create_dir_all(&dir).unwrap_or_else(|e| fatal(&format!("cannot create directory: {e}")));

    let full_path = dir.join(&filename);

    // Read body from stdin if not provided and stdin is piped
    if body.is_empty() && !io::stdin().is_terminal() {
        io::stdin()
            .read_to_string(&mut body)
            .unwrap_or_else(|e| fatal(&format!("reading stdin: {e}")));
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
    if !body.is_empty() {
        buf.push_str(&body);
        if !body.ends_with('\n') {
            buf.push('\n');
        }
    }

    fs::write(&full_path, &buf).unwrap_or_else(|e| fatal(&format!("writing file: {e}")));
    // Print path before sync — skills capture stdout
    println!("{}", full_path.display());

    // Commit + push
    if let Ok(rel) = full_path.strip_prefix(blueprints_dir()) {
        commit_and_push(rel, &format!("{}({}): {}", kind.dir_name(), proj_name, s))?;
    }
    Ok(())
}

/// Resolve an artifact file argument to a real path.
/// Accepts: absolute/relative path, `project/kind/stem`, or bare stem.
/// For bare stems, scans ~/blueprints/*/kind/ for a unique match.
pub fn resolve_artifact_path(file_arg: &str, kind: ArtifactKind) -> PathBuf {
    let p = Path::new(file_arg);
    // Exact path (absolute or relative)
    if p.exists() {
        return p.to_path_buf();
    }
    // Try with .md extension
    let with_ext = p.with_extension("md");
    if with_ext.exists() {
        return with_ext;
    }

    let bp = blueprints_dir();
    let kind_dir = kind.dir_name();

    // Try as project-relative: <project>/<kind>/stem[.md]
    let bp_path = bp.join(file_arg);
    if bp_path.exists() {
        return bp_path;
    }
    let bp_path_ext = bp_path.with_extension("md");
    if bp_path_ext.exists() {
        return bp_path_ext;
    }

    // Bare stem — scan all projects for ~/blueprints/*/kind/stem.md
    let stem = p.file_stem().unwrap_or(p.as_os_str());
    let mut matches = Vec::new();
    if let Ok(entries) = fs::read_dir(&bp) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let candidate = entry.path().join(kind_dir);
            if !candidate.is_dir() {
                continue;
            }
            if let Ok(files) = fs::read_dir(&candidate) {
                for f in files.flatten() {
                    let fp = f.path();
                    if fp.extension().and_then(|e| e.to_str()) == Some("md")
                        && fp.file_stem() == Some(stem)
                    {
                        matches.push(fp);
                    }
                }
            }
        }
    }

    match matches.len() {
        0 => fatal(&format!("artifact not found: {file_arg}")),
        1 => matches.remove(0),
        _ => {
            let list: Vec<_> = matches.iter().map(|m| m.display().to_string()).collect();
            fatal(&format!(
                "ambiguous stem '{file_arg}', matches:\n  {}",
                list.join("\n  ")
            ))
        }
    }
}

pub fn cmd_read(file_path: &str, kind: ArtifactKind, frontmatter_mode: bool) {
    let resolved = resolve_artifact_path(file_path, kind);
    cmd_read_resolved(&resolved, frontmatter_mode);
}

// ---------------------------------------------------------------------------
// Inline comment extraction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Comment {
    pub line: usize,
    pub highlight: Option<String>,
    #[serde(rename = "comment")]
    pub text: String,
}

/// Extract all single-line HTML comments from a markdown body.
/// Returns `Comment` entries with 1-based line numbers relative to the body.
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

            // Check for ==highlight== immediately before the comment
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
        // 2 delimiter lines + yaml content lines
        (Some(yaml), _) => yaml.lines().count() + 2,
    }
}

pub fn cmd_comments(file_path: &str, kind: ArtifactKind, json: bool) {
    let resolved = resolve_artifact_path(file_path, kind);
    let content = fs::read_to_string(&resolved)
        .unwrap_or_else(|e| fatal(&format!("cannot read {}: {e}", resolved.display())));

    let fm_lines = frontmatter_line_count(&content);
    let (_, body) = parse_frontmatter(&content);
    let mut comments = parse_comments(body);

    let file_display = resolved
        .file_name()
        .unwrap_or(resolved.as_os_str())
        .to_string_lossy();

    // Adjust line numbers to be absolute (account for frontmatter)
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

pub fn cmd_latest(kind: ArtifactKind, project: Option<&str>, task_file: Option<&str>) {
    let mut project = project.unwrap_or("").to_string();

    if project.is_empty() && task_file.is_none() {
        let output = process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output();
        match output {
            Ok(o) if o.status.success() => {
                project = resolve_repo_root(
                    String::from_utf8_lossy(&o.stdout).trim(),
                );
            }
            _ => {
                project = env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| fatal("cannot determine working directory"));
            }
        }
    }

    match latest_artifact(kind, task_file, &project) {
        Ok(p) => println!("{}", p.display()),
        Err(e) => fatal(&e),
    }
}

/// Core logic for finding the latest artifact of a given kind.
pub fn latest_artifact(
    kind: ArtifactKind,
    task_file: Option<&str>,
    project: &str,
) -> Result<PathBuf, String> {
    if let Some(tf) = task_file {
        let p = PathBuf::from(tf);
        if p.exists() {
            return Ok(p);
        }
        return Err(format!("task-file not found: {tf}"));
    }

    let dir = artifact_dir(project, kind);
    let entries = fs::read_dir(&dir).map_err(|e| {
        format!(
            "cannot read {} directory {}: {e}",
            kind.dir_name(),
            dir.display()
        )
    })?;

    let mut latest_path: Option<PathBuf> = None;
    let mut latest_time = SystemTime::UNIX_EPOCH;

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

    latest_path.ok_or_else(|| format!("no {} files found in {}", kind.dir_name(), dir.display()))
}

pub fn cmd_archive(kind: ArtifactKind, file_path: &str) -> Result<(), SyncError> {
    let path = Path::new(file_path);
    if !path.exists() {
        fatal(&format!("file not found: {file_path}"));
    }

    // Derive project name from file's location in <vault>/<project>/<kind>/
    let bp = blueprints_dir();
    let rel_path = path
        .strip_prefix(&bp)
        .unwrap_or_else(|_| fatal(&format!("file is not inside {}", bp.display())));
    let proj_name = rel_path
        .components()
        .next()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .unwrap_or_else(|| fatal("cannot determine project from file path"));

    // Best-effort: store as git note in the current project repo
    let git_dir = process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    if let Some(ref gd) = git_dir {
        let _ = process::Command::new("git")
            .args([
                "-C",
                gd,
                "notes",
                &format!("--ref={}", kind.notes_ref()),
                "append",
                "-F",
            ])
            .arg(path)
            .arg("HEAD")
            .status();
    }

    // Move to archive/
    let archive_dir = bp.join(&proj_name).join("archive").join(kind.dir_name());
    fs::create_dir_all(&archive_dir)
        .unwrap_or_else(|e| fatal(&format!("cannot create archive directory: {e}")));
    let file_name = path
        .file_name()
        .unwrap_or_else(|| fatal("cannot determine file name"));
    let dest = archive_dir.join(file_name);
    fs::rename(path, &dest).unwrap_or_else(|e| fatal(&format!("archiving file: {e}")));
    eprintln!("Archived: {file_path} → {}", dest.display());

    // Stage both the deletion of the original and the new archive file
    if let (Ok(src_rel), Ok(dest_rel)) = (path.strip_prefix(&bp), dest.strip_prefix(&bp)) {
        let bp_str = bp.to_string_lossy();
        // Stage the deleted original
        let _ = process::Command::new("git")
            .args(["-C", &bp_str, "add", "--"])
            .arg(src_rel)
            .status();
        let slug = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        commit_and_push(dest_rel, &format!("archive({}): {}", proj_name, slug))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Generic listing (replaces plan.rs / spec.rs listing)
// ---------------------------------------------------------------------------

pub fn list_artifacts(kind: ArtifactKind) -> Vec<Artifact> {
    list_artifacts_filtered(kind, false)
}

pub fn list_archived_artifacts(kind: ArtifactKind) -> Vec<Artifact> {
    list_artifacts_filtered(kind, true)
}

fn list_artifacts_filtered(kind: ArtifactKind, archived: bool) -> Vec<Artifact> {
    let bp = blueprints_dir();
    // Walk all project subdirs in ~/blueprints/
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

        if archived {
            let archive_dir = proj_path.join("archive").join(kind.dir_name());
            collect_artifacts(&bp, &archive_dir, &proj_name, &mut artifacts);
        } else {
            let kind_dir = proj_path.join(kind.dir_name());
            collect_artifacts(&bp, &kind_dir, &proj_name, &mut artifacts);
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
        let (title, _) = extract_frontmatter(&path);
        out.push(Artifact {
            name,
            path,
            title,
            project: fallback_project.to_string(),
            mod_time: info.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            size: info.len(),
        });
    }
}

/// List artifacts from git notes across all known projects.
pub fn list_git_notes_artifacts_all(kind: ArtifactKind) -> Vec<Artifact> {
    let mut projects: HashSet<String> = HashSet::new();

    // Collect project names from blueprints directory structure (no file I/O)
    if let Ok(entries) = fs::read_dir(blueprints_dir()) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name != "archive" && name != ".git" {
                    projects.insert(name);
                }
            }
        }
    }

    // Also check the current directory's git repo (resolve worktrees to main)
    if let Some(cwd_project) = process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| resolve_repo_root(String::from_utf8_lossy(&o.stdout).trim()))
    {
        projects.insert(cwd_project);
    }

    let mut all = Vec::new();
    for project in &projects {
        all.extend(list_git_notes_artifacts(kind, project));
    }
    all.sort_by_key(|a| std::cmp::Reverse(a.mod_time));
    all
}

fn list_git_notes_artifacts(kind: ArtifactKind, project: &str) -> Vec<Artifact> {
    let git_dir = process::Command::new("git")
        .args(["-C", project, "rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let Some(git_dir) = git_dir else {
        return Vec::new();
    };

    let list_output = process::Command::new("git")
        .args([
            "-C",
            &git_dir,
            "notes",
            &format!("--ref={}", kind.notes_ref()),
            "list",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success());
    let Some(list_output) = list_output else {
        return Vec::new();
    };

    let list_text = String::from_utf8_lossy(&list_output.stdout);
    let mut artifacts = Vec::new();

    for line in list_text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let commit_sha = parts[1];

        let commit_time = process::Command::new("git")
            .args(["-C", &git_dir, "log", "-1", "--format=%ct", commit_sha])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse::<u64>()
                    .ok()
            })
            .map(|secs| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs))
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let note_output = process::Command::new("git")
            .args([
                "-C",
                &git_dir,
                "notes",
                &format!("--ref={}", kind.notes_ref()),
                "show",
                commit_sha,
            ])
            .output()
            .ok()
            .filter(|o| o.status.success());
        let Some(note_output) = note_output else {
            continue;
        };
        let content = String::from_utf8_lossy(&note_output.stdout).to_string();

        let short_sha = &commit_sha[..7.min(commit_sha.len())];
        for (idx, chunk) in split_notes(&content).into_iter().enumerate() {
            let (title, proj) = extract_frontmatter_from_str(&chunk);
            let label = if title.is_empty() {
                format!("note:{short_sha}#{idx}")
            } else {
                title.clone()
            };
            artifacts.push(Artifact {
                name: format!("git-note:{short_sha}/{label}"),
                path: PathBuf::from(format!("git-notes://{git_dir}/{commit_sha}")),
                title,
                project: if proj.is_empty() {
                    project.to_string()
                } else {
                    proj
                },
                mod_time: commit_time,
                size: chunk.len() as u64,
            });
        }
    }
    artifacts
}

pub fn load_content(kind: ArtifactKind, path: &PathBuf) -> String {
    let path_str = path.to_string_lossy();
    if let Some(rest) = path_str.strip_prefix("git-notes://") {
        // Format: git-notes://<git-dir>/<40-char-sha>
        // Can't use rfind('/') because git_dir contains slashes.
        // Commit SHAs are 40 hex chars, so split at len-40.
        if rest.len() > 41 {
            let split = rest.len() - 40;
            let git_dir = &rest[..split - 1]; // -1 for the separator '/'
            let commit_sha = &rest[split..];
            return process::Command::new("git")
                .args([
                    "-C",
                    git_dir,
                    "notes",
                    &format!("--ref={}", kind.notes_ref()),
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

// ---------------------------------------------------------------------------
// Frontmatter extraction
// ---------------------------------------------------------------------------

fn strip_yaml_quotes(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.len() >= 2
        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
    {
        let inner = &trimmed[1..trimmed.len() - 1];
        inner.replace("\\\"", "\"").replace("\\\\", "\\")
    } else {
        trimmed.to_string()
    }
}

fn extract_frontmatter_from_str(content: &str) -> (String, String) {
    let mut title = String::new();
    let mut project = String::new();
    let mut in_frontmatter = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if in_frontmatter {
                break;
            }
            in_frontmatter = true;
            continue;
        }
        if in_frontmatter {
            if let Some(val) = trimmed.strip_prefix("topic:") {
                title = strip_yaml_quotes(val);
            } else if let Some(val) = trimmed.strip_prefix("project:") {
                project = strip_yaml_quotes(val);
            }
        } else if let Some(t) = trimmed.strip_prefix("# ") {
            title = t.to_string();
            break;
        }
    }
    (title, project)
}

fn extract_frontmatter(path: &Path) -> (String, String) {
    let Ok(content) = fs::read_to_string(path) else {
        return (String::new(), String::new());
    };
    extract_frontmatter_from_str(&content)
}

// ---------------------------------------------------------------------------
// Universal stem resolution (across all artifact kinds)
// ---------------------------------------------------------------------------

/// Resolve a bare stem across ALL artifact kinds in priority order:
/// Doc > Report > Review > Plan > Spec.
///
/// If the stem is an existing path (absolute or relative), returns it directly.
/// Otherwise scans `blueprints_dir()/*/kind/` for a matching file_stem.
pub fn resolve_stem_universal(stem: &str) -> PathBuf {
    let p = Path::new(stem);
    // Exact path (absolute or relative)
    if p.exists() {
        return p.to_path_buf();
    }
    let with_ext = p.with_extension("md");
    if with_ext.exists() {
        return with_ext;
    }

    let bp = blueprints_dir();

    // Try as project-relative path inside vault
    let bp_path = bp.join(stem);
    if bp_path.exists() {
        return bp_path;
    }
    let bp_path_ext = bp_path.with_extension("md");
    if bp_path_ext.exists() {
        return bp_path_ext;
    }

    // Bare stem — scan all projects × all kinds in priority order
    let file_stem = p.file_stem().unwrap_or(p.as_os_str());
    let mut matches: Vec<(ArtifactKind, PathBuf)> = Vec::new();

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
                        if fp.extension().and_then(|e| e.to_str()) == Some("md")
                            && fp.file_stem() == Some(file_stem)
                        {
                            matches.push((kind, fp));
                        }
                    }
                }
            }
        }
    }

    if matches.is_empty() {
        fatal(&format!("artifact not found: {stem}"));
    }

    if matches.len() == 1 {
        return matches.remove(0).1;
    }

    // Multiple matches — check if they span different kinds
    let kinds_seen: HashSet<&str> = matches.iter().map(|(k, _)| k.dir_name()).collect();
    if kinds_seen.len() > 1 {
        // Return highest-priority (ALL_KINDS is already in priority order)
        for &kind in &ALL_KINDS {
            if let Some(pos) = matches.iter().position(|(k, _)| *k == kind) {
                return matches.remove(pos).1;
            }
        }
    }

    // Same kind, different projects — ambiguous
    let list: Vec<_> = matches.iter().map(|(_, p)| p.display().to_string()).collect();
    fatal(&format!(
        "ambiguous stem '{stem}', matches:\n  {}",
        list.join("\n  ")
    ))
}

/// Read and print an artifact from a resolved path (no kind needed).
pub fn cmd_read_resolved(resolved: &Path, frontmatter_mode: bool) {
    let content = fs::read_to_string(resolved)
        .unwrap_or_else(|e| fatal(&format!("reading file: {e}")));

    let (yaml, body) = parse_frontmatter(&content);

    if frontmatter_mode {
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
        print!("{body}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worktrees_share_repo_name() {
        assert_eq!(project_name("/Users/me/src/repo.git/wt1"), "repo");
        assert_eq!(project_name("/Users/me/src/repo.git/wt2"), "repo");
    }

    #[test]
    fn bare_git_dir_uses_stem() {
        assert_eq!(project_name("/Users/me/src/repo.git"), "repo");
    }

    #[test]
    fn nested_worktree_uses_repo_name() {
        assert_eq!(project_name("/Users/me/src/mono.git/apps/web"), "mono");
    }

    #[test]
    fn normal_path_uses_last_component() {
        assert_eq!(project_name("/Users/me/src/myapp/src/core"), "core");
    }

    #[test]
    fn dots_replaced_with_underscores() {
        assert_eq!(project_name("/Users/me/src/.claude"), "_claude");
        assert_eq!(project_name("/Users/me/src/my.project"), "my_project");
    }

    #[test]
    fn task_file_returns_specified_path() {
        let tmp = std::env::temp_dir().join(format!("ck-latest-test-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let plan = tmp.join("my-plan.md");
        std::fs::write(&plan, "# plan\n").unwrap();

        let result = latest_artifact(ArtifactKind::Plan, Some(plan.to_str().unwrap()), "");
        assert!(result.is_ok(), "expected Ok, got {result:?}");
        assert_eq!(
            result.unwrap().canonicalize().unwrap(),
            plan.canonicalize().unwrap(),
            "--task-file should return the specified path"
        );

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn task_file_flag_errors_when_file_missing() {
        let result = latest_artifact(ArtifactKind::Plan, Some("/nonexistent/path/plan.md"), "");
        assert!(result.is_err(), "expected Err for missing task-file");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("task-file not found"),
            "error message should mention task-file, got: {msg}"
        );
    }

    #[test]
    fn frontmatter_has_no_status_field() {
        let tmp = std::env::temp_dir().join(format!("ck-test-{}", std::process::id()));
        let project_path = "/some/project";

        let project_dir = artifact_dir_with_base(project_path, "plans", &tmp);
        std::fs::create_dir_all(&project_dir).unwrap();

        let slug = crate::slug::slug("Test Topic");
        let file_path = project_dir.join(format!("{slug}.md"));

        let now = chrono_rfc3339();
        let mut buf = String::new();
        buf.push_str("---\n");
        buf.push_str(&format!("topic: {}\n", yaml_quote("Test Topic")));
        buf.push_str(&format!("created: {now}\n"));
        buf.push_str("---\n");
        std::fs::write(&file_path, &buf).unwrap();

        let content = std::fs::read_to_string(&file_path).unwrap();

        let (yaml, _) = parse_frontmatter(&content);
        let yaml = yaml.expect("frontmatter must be present");
        let keys: Vec<_> = parse_yaml_map(yaml).into_iter().map(|(k, _)| k).collect();
        assert!(
            !keys.contains(&"status".to_string()),
            "frontmatter must not contain a 'status' field, got keys: {keys:?}"
        );

        std::fs::remove_dir_all(&tmp).ok();
    }

    fn create_artifact_file(base: &Path, project: &str, kind: ArtifactKind, stem: &str) -> PathBuf {
        let dir = base.join(project).join(kind.dir_name());
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join(format!("{stem}.md"));
        std::fs::write(&file, "# test\n").unwrap();
        file
    }

    #[test]
    fn universal_resolve_picks_highest_priority_kind() {
        let tmp = std::env::temp_dir().join(format!("ct-univ-prio-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let _spec = create_artifact_file(&tmp, "myproj", ArtifactKind::Spec, "widget");
        let doc = create_artifact_file(&tmp, "myproj", ArtifactKind::Doc, "widget");

        let prev = env::var("CT_BLUEPRINTS_DIR").ok();
        unsafe { env::set_var("CT_BLUEPRINTS_DIR", &tmp) };

        let result = resolve_stem_universal("widget");
        assert_eq!(result, doc, "Doc should take priority over Spec");

        match prev {
            Some(v) => unsafe { env::set_var("CT_BLUEPRINTS_DIR", v) },
            None => unsafe { env::remove_var("CT_BLUEPRINTS_DIR") },
        }
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn universal_resolve_single_match() {
        let tmp = std::env::temp_dir().join(format!("ct-univ-single-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let plan = create_artifact_file(&tmp, "myproj", ArtifactKind::Plan, "deploy");

        let prev = env::var("CT_BLUEPRINTS_DIR").ok();
        unsafe { env::set_var("CT_BLUEPRINTS_DIR", &tmp) };

        let result = resolve_stem_universal("deploy");
        assert_eq!(result, plan);

        match prev {
            Some(v) => unsafe { env::set_var("CT_BLUEPRINTS_DIR", v) },
            None => unsafe { env::remove_var("CT_BLUEPRINTS_DIR") },
        }
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn universal_resolve_report_over_plan() {
        let tmp = std::env::temp_dir().join(format!("ct-univ-rp-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let _plan = create_artifact_file(&tmp, "myproj", ArtifactKind::Plan, "auth");
        let report = create_artifact_file(&tmp, "myproj", ArtifactKind::Report, "auth");

        let prev = env::var("CT_BLUEPRINTS_DIR").ok();
        unsafe { env::set_var("CT_BLUEPRINTS_DIR", &tmp) };

        let result = resolve_stem_universal("auth");
        assert_eq!(result, report, "Report should take priority over Plan");

        match prev {
            Some(v) => unsafe { env::set_var("CT_BLUEPRINTS_DIR", v) },
            None => unsafe { env::remove_var("CT_BLUEPRINTS_DIR") },
        }
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn parse_comments_with_highlight() {
        let comments = parse_comments("==foo==<!--bar-->");
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].line, 1);
        assert_eq!(comments[0].highlight.as_deref(), Some("foo"));
        assert_eq!(comments[0].text, "bar");
    }

    #[test]
    fn parse_comments_without_highlight() {
        let comments = parse_comments("<!--bar-->");
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].line, 1);
        assert_eq!(comments[0].highlight, None);
        assert_eq!(comments[0].text, "bar");
    }

    #[test]
    fn parse_comments_multiple_on_one_line() {
        let comments = parse_comments("<!--a--> <!--b-->");
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].line, 1);
        assert_eq!(comments[0].text, "a");
        assert_eq!(comments[1].line, 1);
        assert_eq!(comments[1].text, "b");
    }

    #[test]
    fn parse_comments_no_comments() {
        let comments = parse_comments("just text");
        assert!(comments.is_empty());
    }

    #[test]
    fn parse_comments_highlight_without_comment() {
        let comments = parse_comments("==foo==");
        assert!(comments.is_empty());
    }

    #[test]
    fn parse_comments_on_later_line() {
        let comments = parse_comments("line1\nline2\n<!--here-->");
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].line, 3);
        assert_eq!(comments[0].text, "here");
    }
}
