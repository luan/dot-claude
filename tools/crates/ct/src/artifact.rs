use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::SystemTime;

use rmcp::schemars;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
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

    pub fn from_dir_name(name: &str) -> Option<Self> {
        ALL_KINDS.iter().copied().find(|k| k.dir_name() == name)
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

#[derive(Debug, Clone, Serialize)]
pub struct Artifact {
    pub name: String,
    pub path: PathBuf,
    pub title: String,
    #[serde(serialize_with = "serialize_project_name")]
    pub project: String,
    #[serde(rename = "modified", serialize_with = "serialize_mod_time")]
    pub mod_time: SystemTime,
    #[serde(serialize_with = "serialize_size")]
    pub size: u64,
    pub created: Option<String>,
    pub source: Option<String>,
    pub tags: Vec<String>,
    pub author: Option<String>,
}

fn serialize_mod_time<S: serde::Serializer>(t: &SystemTime, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&format_date(*t))
}

fn serialize_size<S: serde::Serializer>(b: &u64, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&format_size(*b))
}

fn serialize_project_name<S: serde::Serializer>(p: &str, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&project_name(p))
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

/// Non-panicking variant for server contexts where `fatal()` would kill the
/// whole MCP server mid-request.
pub fn blueprints_dir_checked() -> Result<PathBuf, CtError> {
    let dir = blueprints_dir_unchecked();
    if !dir.is_dir() {
        return Err(CtError::Validation(format!(
            "blueprints directory missing: {}. Run `ct vault init` first.",
            dir.display()
        )));
    }
    Ok(dir)
}

/// Returns the blueprints vault directory without checking existence (for init).
pub fn blueprints_dir_unchecked() -> PathBuf {
    if let Ok(custom) = env::var("CT_BLUEPRINTS_DIR") {
        return PathBuf::from(custom);
    }
    let home = home_dir();
    Path::new(&home).join("blueprints")
}

/// Assert that `p` resides inside the vault. Canonicalizes both the path and
/// the vault root before comparing so `..` components and symlinks are
/// resolved — defense-in-depth against path-traversal arguments. Returns the
/// original (non-canonicalized) path on success so existing call sites keep
/// working against the same prefix they saw before.
fn ensure_in_vault(p: &Path) -> Result<PathBuf, ResolveError> {
    let bp = blueprints_dir_unchecked();
    let bp_canon = bp
        .canonicalize()
        .map_err(|_| ResolveError::NotFound(p.display().to_string()))?;
    let p_canon = p
        .canonicalize()
        .map_err(|_| ResolveError::NotFound(p.display().to_string()))?;
    if p_canon.starts_with(&bp_canon) {
        Ok(p.to_path_buf())
    } else {
        Err(ResolveError::NotFound(p.display().to_string()))
    }
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
// Date-prefix stripping
// ---------------------------------------------------------------------------

/// Strip YYYYMMDD-HH- or legacy YYYYMMDD- date prefix from an artifact stem.
pub fn strip_date_prefix(stem: &str) -> &str {
    if stem.len() > 12
        && stem.as_bytes()[..8].iter().all(|b| b.is_ascii_digit())
        && stem.as_bytes()[8] == b'-'
        && stem.as_bytes()[9..11].iter().all(|b| b.is_ascii_digit())
        && stem.as_bytes()[11] == b'-'
    {
        &stem[12..]
    } else if stem.len() > 9
        && stem.as_bytes()[..8].iter().all(|b| b.is_ascii_digit())
        && stem.as_bytes()[8] == b'-'
    {
        &stem[9..]
    } else {
        stem
    }
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
        if p.is_absolute() {
            // Bare repo (e.g. arc.git): common-dir IS the repo, not a .git subdir
            let fname = p.file_name().unwrap_or_default().to_string_lossy();
            if fname == ".git" {
                // Normal repo: parent of `.git` is the repo root
                return p.parent().unwrap().to_string_lossy().to_string();
            }
            // Bare repo or non-standard: common-dir itself is the root
            return p.to_string_lossy().to_string();
        }
    }
    toplevel.to_string()
}

/// Auto-detect the current project from cwd: prefer `git rev-parse --show-toplevel`
/// (resolved through worktrees to the main repo root), fall back to cwd.
pub fn current_project() -> String {
    let output = process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();
    if let Ok(o) = output
        && o.status.success()
    {
        return resolve_repo_root(String::from_utf8_lossy(&o.stdout).trim());
    }
    env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| fatal("cannot determine working directory"))
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

/// Stem resolution failure — either nothing matched or multiple candidates did.
#[derive(Debug)]
pub enum ResolveError {
    NotFound(String),
    Ambiguous(Vec<PathBuf>),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(arg) => write!(f, "artifact not found: {arg}"),
            Self::Ambiguous(matches) => {
                let list: Vec<_> = matches.iter().map(|m| m.display().to_string()).collect();
                write!(f, "ambiguous stem, matches:\n  {}", list.join("\n  "))
            }
        }
    }
}

impl std::error::Error for ResolveError {}

/// Unified error type for library-level ct operations.
#[derive(Debug)]
pub enum CtError {
    Sync(SyncError),
    Resolve(ResolveError),
    Io(std::io::Error),
    Validation(String),
}

impl std::fmt::Display for CtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sync(e) => write!(f, "{e}"),
            Self::Resolve(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::Validation(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CtError {}

impl From<SyncError> for CtError {
    fn from(e: SyncError) -> Self {
        Self::Sync(e)
    }
}

impl From<std::io::Error> for CtError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<ResolveError> for CtError {
    fn from(e: ResolveError) -> Self {
        Self::Resolve(e)
    }
}

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
            return Err(SyncError::Commit(stderr.trim().to_string()));
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
            "--dive is only valid for spec artifacts".to_string(),
        ));
    }
    // Resolve worktree paths to the main repo root
    let project = &resolve_repo_root(project);
    // slug_override is MCP-client-supplied; sanitize through the same filter
    // the topic path uses so `"../evil"` can't land outside the vault. An
    // override of only whitespace or filler words is rejected as invalid.
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

    // Commit + push: push failure propagates so callers see it
    if let Ok(rel) = full_path.strip_prefix(&bp) {
        commit_and_push(rel, &format!("{}({}): {}", kind.dir_name(), proj_name, s))?;
    }
    Ok(CreateOutcome {
        path: full_path,
        project: proj_name,
        kind,
        pushed: true,
    })
}

/// Resolve an artifact file argument to a real path.
/// Accepts: absolute/relative path, `project/kind/stem`, or bare stem.
/// For bare stems, scans ~/blueprints/*/kind/ for a unique match.
///
/// Every successful return is canonicalized and verified to live under the
/// vault — path-traversal arguments like `"../../etc/passwd"` fail with
/// `ResolveError::NotFound(file_arg)` even when the underlying file exists.
pub fn resolve_artifact_path(file_arg: &str, kind: ArtifactKind) -> Result<PathBuf, ResolveError> {
    let p = Path::new(file_arg);
    // Exact path (absolute or relative)
    if p.exists() {
        return ensure_in_vault(p).map_err(|_| ResolveError::NotFound(file_arg.to_string()));
    }
    // Try with .md extension
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

    // Bare stem — scan all projects for ~/blueprints/*/kind/stem.md
    // For Spec, also scan dive/ so dive files are findable by bare stem.
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
                        // Fuzzy: strip date prefix from candidate and/or query
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

/// Frontmatter fields an artifact carries. Populated by `read`.
#[derive(Debug, Clone, Serialize)]
pub struct Frontmatter {
    pub topic: Option<String>,
    pub created: Option<String>,
    pub author: Option<String>,
    pub source: Option<String>,
    pub tags: Vec<String>,
}

/// Result of a `read` call: parsed frontmatter, raw body, and inline comments.
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
    let frontmatter = yaml.map(|_| {
        let (title, _, created, source, tags, author) = extract_frontmatter_full_from_str(&content);
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
/// Returns the destination path. Does NOT commit or push and does NOT emit
/// user-facing output — the CLI wrapper reports the move.
fn archive_single(
    kind: ArtifactKind,
    path: &Path,
    bp: &Path,
    proj_name: &str,
) -> Result<PathBuf, String> {
    let source_subfolder = detect_subfolder(path, bp, kind);
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

    // Move to archive/, preserving the source subfolder (spec/ or dive/)
    let archive_dir = bp.join(proj_name).join("archive").join(&source_subfolder);
    fs::create_dir_all(&archive_dir)
        .map_err(|e| format!("cannot create archive directory: {e}"))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| "cannot determine file name".to_string())?;
    let dest = archive_dir.join(file_name);
    fs::rename(path, &dest).map_err(|e| format!("archiving file: {e}"))?;

    // Stage the deletion of the original
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
/// Returns (canonical path, project name).
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

    // Derive project name from vault path (same pattern as validate_archive_path)
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

    // Process frontmatter lines, replacing auto-derived tags
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

    // Preserve trailing newline if original had one
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

/// Outcome of a successful `archive` call. Dry-runs use `ArchiveOutcome` too,
/// signalling the prospective destination without having moved the file.
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

    // Validate all files up front before moving any
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

    // All files must belong to the same project
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

    // Archive all files, collecting destinations.
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

    // Stage all archive destinations and commit once
    let bp_str = bp.to_string_lossy();
    for dest in &dests {
        if let Ok(dest_rel) = dest.strip_prefix(&bp) {
            let _ = process::Command::new("git")
                .args(["-C", &bp_str, "add"])
                .arg(dest_rel)
                .status();
        }
    }

    let n = dests.len();
    let commit_ok = process::Command::new("git")
        .args([
            "-C",
            &bp_str,
            "commit",
            "-m",
            &format!("archive({proj_name}): {n} artifacts"),
        ])
        .output();

    match commit_ok {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            if !stderr.contains("nothing to commit") {
                return Err(SyncError::Commit(stderr.trim().to_string()));
            }
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

    if let Some(e) = batch_err {
        eprintln!("committed {n} successful archives, but batch had an error: {e}");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Generic listing (replaces plan.rs / spec.rs listing)
// ---------------------------------------------------------------------------

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
            let (title, proj, created, source, tags, author) =
                extract_frontmatter_full_from_str(&chunk);
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
                created,
                source,
                tags,
                author,
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

/// Full frontmatter extraction: title, project, created, source, tags, author.
fn extract_frontmatter_full_from_str(
    content: &str,
) -> (
    String,
    String,
    Option<String>,
    Option<String>,
    Vec<String>,
    Option<String>,
) {
    let mut title = String::new();
    let mut project = String::new();
    let mut created = None;
    let mut source = None;
    let mut tags = Vec::new();
    let mut author = None;
    let mut in_frontmatter = false;
    let mut in_tags = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if in_frontmatter {
                break;
            }
            in_frontmatter = true;
            continue;
        }
        if !in_frontmatter {
            if let Some(t) = trimmed.strip_prefix("# ") {
                title = t.to_string();
                break;
            }
            continue;
        }

        // Inside frontmatter: detect YAML list items under `tags:`
        if in_tags {
            if let Some(item) = trimmed.strip_prefix("- ") {
                tags.push(strip_yaml_quotes(item));
                continue;
            }
            // No longer a list item — stop collecting tags
            in_tags = false;
        }

        if let Some(val) = trimmed.strip_prefix("topic:") {
            title = strip_yaml_quotes(val);
        } else if let Some(val) = trimmed.strip_prefix("project:") {
            project = strip_yaml_quotes(val);
        } else if let Some(val) = trimmed.strip_prefix("created:") {
            let v = strip_yaml_quotes(val);
            if !v.is_empty() {
                created = Some(v);
            }
        } else if let Some(val) = trimmed.strip_prefix("source:") {
            let v = strip_yaml_quotes(val);
            if !v.is_empty() {
                // Strip wiki-link brackets: [[stem]] -> stem
                let stripped = v.trim_start_matches("[[").trim_end_matches("]]");
                source = Some(stripped.to_string());
            }
        } else if let Some(val) = trimmed.strip_prefix("tags:") {
            let v = val.trim();
            if v.is_empty() {
                // Tags on following lines as YAML list
                in_tags = true;
            }
            // Inline tags (e.g. `tags: [a, b]`) not used by ct — skip
        } else if let Some(val) = trimmed.strip_prefix("author:") {
            let v = strip_yaml_quotes(val);
            if !v.is_empty() {
                author = Some(v);
            }
        }
    }
    (title, project, created, source, tags, author)
}

fn extract_frontmatter_full(
    path: &Path,
) -> (
    String,
    String,
    Option<String>,
    Option<String>,
    Vec<String>,
    Option<String>,
) {
    let Ok(content) = fs::read_to_string(path) else {
        return (String::new(), String::new(), None, None, Vec::new(), None);
    };
    extract_frontmatter_full_from_str(&content)
}

// ---------------------------------------------------------------------------
// Universal stem resolution (across all artifact kinds)
// ---------------------------------------------------------------------------

/// Resolve a bare stem across ALL artifact kinds in priority order:
/// Doc > Report > Review > Plan > Spec.
///
/// If the stem is an existing path (absolute or relative), returns it directly.
/// Otherwise scans `blueprints_dir()/*/kind/` for a matching file_stem.
pub fn resolve_stem_universal(stem: &str) -> Result<PathBuf, ResolveError> {
    let p = Path::new(stem);
    // Exact path (absolute or relative)
    if p.exists() {
        return ensure_in_vault(p).map_err(|_| ResolveError::NotFound(stem.to_string()));
    }
    let with_ext = p.with_extension("md");
    if with_ext.exists() {
        return ensure_in_vault(&with_ext).map_err(|_| ResolveError::NotFound(stem.to_string()));
    }

    let bp = blueprints_dir();

    // Try as project-relative path inside vault
    let bp_path = bp.join(stem);
    if bp_path.exists() {
        return ensure_in_vault(&bp_path).map_err(|_| ResolveError::NotFound(stem.to_string()));
    }
    let bp_path_ext = bp_path.with_extension("md");
    if bp_path_ext.exists() {
        return ensure_in_vault(&bp_path_ext).map_err(|_| ResolveError::NotFound(stem.to_string()));
    }

    // Bare stem — scan all projects × all kinds in priority order
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
                        // Fuzzy: strip date prefix from candidate and/or query
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

    // Multiple matches — check if they span different kinds
    let kinds_seen: HashSet<&str> = resolved.iter().map(|(k, _)| k.dir_name()).collect();
    if kinds_seen.len() > 1 {
        // Return highest-priority (ALL_KINDS is already in priority order)
        for &kind in &ALL_KINDS {
            if let Some(pos) = resolved.iter().position(|(k, _)| *k == kind) {
                let hit = resolved.remove(pos).1;
                return ensure_in_vault(&hit).map_err(|_| ResolveError::NotFound(stem.to_string()));
            }
        }
    }

    // Same kind, different projects — ambiguous
    Err(ResolveError::Ambiguous(
        std::mem::take(resolved)
            .into_iter()
            .map(|(_, p)| p)
            .collect(),
    ))
}

/// Read and print an artifact from a resolved path (no kind needed).
pub fn cmd_read_resolved(resolved: &Path, frontmatter_mode: bool) {
    // For non-frontmatter mode, use the structured `read` core so CLI and MCP
    // agree on what the body is. Frontmatter-as-JSON mode preserves the raw
    // YAML key order from the file (downstream skills parse this line).
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

    // Derive project name (same pattern as validate_archive_path)
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

    // Warn about incoming wiki-links to old stem
    let bp_str = bp.to_string_lossy();
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

    // Read and update content
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
                // Inside tags list — replace type/* and project/* tags
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

        // Remove trailing newline since the original yaml doesn't include it
        if new_yaml.ends_with('\n') {
            new_yaml.pop();
        }

        format!("---\n{new_yaml}{}", &content[fm_end..])
    } else {
        content.clone()
    };

    // Write new file, delete old
    fs::write(&new_path, &updated).unwrap_or_else(|e| fatal(&format!("writing new file: {e}")));
    fs::remove_file(&old_path).unwrap_or_else(|e| fatal(&format!("removing old file: {e}")));

    // Stage both old (deletion) and new path
    let old_rel = old_path
        .strip_prefix(&bp)
        .unwrap_or_else(|_| fatal("cannot compute relative path for old file"));
    let new_rel = new_path
        .strip_prefix(&bp)
        .unwrap_or_else(|_| fatal("cannot compute relative path for new file"));

    let add_ok = process::Command::new("git")
        .args(["-C", &bp_str, "add", "--"])
        .arg(old_rel)
        .arg(new_rel)
        .status()
        .is_ok_and(|s| s.success());

    if !add_ok {
        return Err(SyncError::Add(format!(
            "git add failed in {}",
            bp.display()
        )));
    }

    let new_stem = new_path.file_stem().unwrap_or_default().to_string_lossy();
    let message = format!("rename({proj_name}): {old_stem} → {new_stem}");

    let commit_output = process::Command::new("git")
        .args(["-C", &bp_str, "commit", "-m", &message])
        .output();

    match commit_output {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            if stderr.contains("nothing to commit") {
                return Ok(());
            }
            return Err(SyncError::Commit(stderr.trim().to_string()));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialize all tests that mutate CT_BLUEPRINTS_DIR to prevent env-var races.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

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

        let result = latest_artifact(ArtifactKind::Plan, Some(plan.to_str().unwrap()), "", false);
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
        let result = latest_artifact(
            ArtifactKind::Plan,
            Some("/nonexistent/path/plan.md"),
            "",
            false,
        );
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

        with_blueprints_dir(&tmp, || {
            let result = resolve_stem_universal("widget").expect("resolve widget");
            assert_eq!(result, doc, "Doc should take priority over Spec");
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn universal_resolve_single_match() {
        let tmp = std::env::temp_dir().join(format!("ct-univ-single-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let plan = create_artifact_file(&tmp, "myproj", ArtifactKind::Plan, "deploy");

        with_blueprints_dir(&tmp, || {
            let result = resolve_stem_universal("deploy").expect("resolve deploy");
            assert_eq!(result, plan);
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn universal_resolve_report_over_plan() {
        let tmp = std::env::temp_dir().join(format!("ct-univ-rp-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let _plan = create_artifact_file(&tmp, "myproj", ArtifactKind::Plan, "auth");
        let report = create_artifact_file(&tmp, "myproj", ArtifactKind::Report, "auth");

        with_blueprints_dir(&tmp, || {
            let result = resolve_stem_universal("auth").expect("resolve auth");
            assert_eq!(result, report, "Report should take priority over Plan");
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn frontmatter_full_all_fields() {
        let content = "\
---
topic: \"My Widget\"
project: myproj
created: 2026-01-15T10:30:00Z
source: \"[[some-spec]]\"
tags:
  - type/plan
  - domain/combat
  - stage/implementing
author: \"Luan\"
---
# Body
";
        let (title, project, created, source, tags, author) =
            extract_frontmatter_full_from_str(content);
        assert_eq!(title, "My Widget");
        assert_eq!(project, "myproj");
        assert_eq!(created.as_deref(), Some("2026-01-15T10:30:00Z"));
        assert_eq!(source.as_deref(), Some("some-spec"));
        assert_eq!(
            tags,
            vec!["type/plan", "domain/combat", "stage/implementing"]
        );
        assert_eq!(author.as_deref(), Some("Luan"));
    }

    #[test]
    fn frontmatter_full_optional_fields_missing() {
        let content = "\
---
topic: Minimal
project: proj
---
";
        let (title, project, created, source, tags, author) =
            extract_frontmatter_full_from_str(content);
        assert_eq!(title, "Minimal");
        assert_eq!(project, "proj");
        assert!(created.is_none());
        assert!(source.is_none());
        assert!(tags.is_empty());
        assert!(author.is_none());
    }

    #[test]
    fn frontmatter_full_tags_list() {
        let content = "\
---
topic: Tags
project: p
tags:
  - alpha
  - \"beta\"
  - 'gamma'
---
";
        let (_, _, _, _, tags, _) = extract_frontmatter_full_from_str(content);
        assert_eq!(tags, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn frontmatter_full_source_wiki_link_brackets() {
        let content = "\
---
topic: Linked
project: p
source: \"[[my-source-spec]]\"
---
";
        let (_, _, _, source, _, _) = extract_frontmatter_full_from_str(content);
        assert_eq!(source.as_deref(), Some("my-source-spec"));
    }

    #[test]
    fn frontmatter_full_source_without_brackets() {
        let content = "\
---
topic: Plain
project: p
source: plain-ref
---
";
        let (_, _, _, source, _, _) = extract_frontmatter_full_from_str(content);
        assert_eq!(source.as_deref(), Some("plain-ref"));
    }

    #[test]
    fn frontmatter_full_falls_back_to_h1() {
        let content = "# Heading Title\nsome body\n";
        let (title, project, created, source, tags, author) =
            extract_frontmatter_full_from_str(content);
        assert_eq!(title, "Heading Title");
        assert!(project.is_empty());
        assert!(created.is_none());
        assert!(source.is_none());
        assert!(tags.is_empty());
        assert!(author.is_none());
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

    // ── dive flag tests ─────────────────────────────────────────────────────

    fn with_blueprints_dir<F: FnOnce()>(tmp: &std::path::Path, f: F) {
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

    #[test]
    fn dive_create_routes_to_dive_folder_with_spec_tag() {
        let tmp = std::env::temp_dir().join(format!("ct-dive-create-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let project = tmp.join("myproj");
        std::fs::create_dir_all(&project).unwrap();

        with_blueprints_dir(&tmp, || {
            // create with dive=true should write to dive/, not spec/
            // git commit/push will fail (no repo) — we ignore the Err.
            // Pass bare stem (no [[...]] wrapping) — create wraps it automatically.
            let _ = create(CreateOpts {
                kind: ArtifactKind::Spec,
                topic: "Sub Topic A",
                project: project.to_str().unwrap(),
                slug_override: Some("hub-sub-topic-a"),
                source: Some("20260411-hub"),
                user_tags: &[],
                dive: true,
            });

            let dive_dir = tmp.join("myproj").join("dive");
            let spec_dir = tmp.join("myproj").join("spec");

            let dive_files: Vec<_> = fs::read_dir(&dive_dir)
                .expect("dive/ directory must exist")
                .flatten()
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
                .collect();
            assert_eq!(dive_files.len(), 1, "exactly one file in dive/");

            let spec_has_files = fs::read_dir(&spec_dir)
                .map(|d| {
                    d.flatten()
                        .any(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
                })
                .unwrap_or(false);
            assert!(!spec_has_files, "spec/ must not contain the dive file");

            let content = fs::read_to_string(dive_files[0].path()).unwrap();
            assert!(
                content.contains("type/spec"),
                "dive file must have type/spec tag"
            );
            // Confirm source is singly wrapped — not double-wrapped.
            assert!(
                content.contains("source: \"[[20260411-hub]]\""),
                "source must be singly wrapped: got\n{content}"
            );
            assert!(
                !content.contains("[[[["),
                "source must not be double-wrapped: got\n{content}"
            );
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn include_dives_flag_toggles_list_visibility() {
        let tmp = std::env::temp_dir().join(format!("ct-dive-list-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let spec_dir = tmp.join("myproj").join("spec");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(spec_dir.join("20260411-hub.md"), "---\ntopic: Hub\n---\n").unwrap();

        let dive_dir = tmp.join("myproj").join("dive");
        std::fs::create_dir_all(&dive_dir).unwrap();
        std::fs::write(
            dive_dir.join("20260411-hub-sub.md"),
            "---\ntopic: Sub\n---\n",
        )
        .unwrap();

        with_blueprints_dir(&tmp, || {
            let without = list_artifacts(ArtifactKind::Spec, false);
            assert_eq!(
                without.len(),
                1,
                "list without --include-dives should show 1 artifact"
            );
            assert!(
                without[0].path.to_string_lossy().contains("spec/"),
                "should be the spec hub"
            );

            let with_dives = list_artifacts(ArtifactKind::Spec, true);
            assert_eq!(
                with_dives.len(),
                2,
                "list with --include-dives should show 2 artifacts"
            );
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn archive_dive_lands_in_archive_dive_not_archive_spec() {
        let tmp = std::env::temp_dir().join(format!("ct-dive-archive-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let dive_dir = tmp.join("myproj").join("dive");
        std::fs::create_dir_all(&dive_dir).unwrap();
        let dive_file = dive_dir.join("20260411-hub-sub.md");
        std::fs::write(&dive_file, "---\ntopic: Sub\n---\n").unwrap();

        with_blueprints_dir(&tmp, || {
            // cmd_archive will fail git note/push steps — we only check the file move.
            let _ = cmd_archive(ArtifactKind::Spec, dive_file.to_str().unwrap(), false);

            let expected = tmp
                .join("myproj")
                .join("archive")
                .join("dive")
                .join("20260411-hub-sub.md");
            let wrong = tmp
                .join("myproj")
                .join("archive")
                .join("spec")
                .join("20260411-hub-sub.md");

            assert!(expected.exists(), "archived dive must be at archive/dive/");
            assert!(
                !wrong.exists(),
                "archived dive must NOT be at archive/spec/"
            );
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    // C.1 — --dive on non-Spec kind is rejected at the library level.
    #[test]
    fn dive_rejected_on_non_spec_kinds() {
        let tmp = std::env::temp_dir().join(format!("ct-dive-nonspec-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let project = tmp.join("myproj");
        std::fs::create_dir_all(&project).unwrap();

        with_blueprints_dir(&tmp, || {
            let result = create(CreateOpts {
                kind: ArtifactKind::Plan,
                topic: "Some Plan",
                project: project.to_str().unwrap(),
                slug_override: None,
                source: Some("foo"),
                user_tags: &[],
                dive: true,
            });
            assert!(
                result.is_err(),
                "create with dive=true on a non-Spec kind must return Err"
            );
            let msg = result.unwrap_err().to_string();
            assert!(
                msg.contains("--dive is only valid for spec artifacts"),
                "error message must mention --dive restriction; got: {msg}"
            );
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    // C.2 — Archived dives must appear in list_archived_artifacts.
    #[test]
    fn archived_dive_is_listable() {
        let tmp =
            std::env::temp_dir().join(format!("ct-dive-archived-list-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let project = tmp.join("myproj");
        std::fs::create_dir_all(&project).unwrap();

        with_blueprints_dir(&tmp, || {
            // Create the dive file directly (cmd_create would need a git repo for commit).
            let dive_dir = tmp.join("myproj").join("dive");
            std::fs::create_dir_all(&dive_dir).unwrap();
            let dive_file = dive_dir.join("20260411-hub-sub.md");
            std::fs::write(&dive_file, "---\ntopic: Sub\ntags:\n  - type/spec\n---\n").unwrap();

            // Archive it — file moves to archive/dive/.
            let _ = cmd_archive(ArtifactKind::Spec, dive_file.to_str().unwrap(), false);

            // Now list archived artifacts — the dive must appear.
            let archived = list_archived_artifacts(ArtifactKind::Spec);
            assert!(
                archived
                    .iter()
                    .any(|a| a.path.to_string_lossy().contains("archive/dive")),
                "archived dive must be visible in list_archived_artifacts; got: {:?}",
                archived
                    .iter()
                    .map(|a| a.path.display().to_string())
                    .collect::<Vec<_>>()
            );
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    // C.3 — resolve_artifact_path finds a dive file by its bare stem.
    #[test]
    fn resolve_artifact_path_finds_dive_by_bare_stem() {
        let tmp = std::env::temp_dir().join(format!("ct-dive-resolve-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let dive_dir = tmp.join("myproj").join("dive");
        std::fs::create_dir_all(&dive_dir).unwrap();
        let dive_file = dive_dir.join("20260411-hub-detail.md");
        std::fs::write(&dive_file, "---\ntopic: Detail\n---\n").unwrap();

        with_blueprints_dir(&tmp, || {
            let resolved = resolve_artifact_path("20260411-hub-detail", ArtifactKind::Spec)
                .expect("resolve dive stem");
            assert!(
                resolved.to_string_lossy().contains("dive/"),
                "resolved path must be inside dive/; got: {}",
                resolved.display()
            );
            assert_eq!(
                resolved.canonicalize().unwrap(),
                dive_file.canonicalize().unwrap(),
                "resolved path must match the dive file"
            );
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    // ── strip_date_prefix tests ─────────────────────────────────────────────

    #[test]
    fn strip_date_prefix_current_format() {
        assert_eq!(
            strip_date_prefix("20260411-07-kdl-derive-reference"),
            "kdl-derive-reference"
        );
    }

    #[test]
    fn strip_date_prefix_legacy_format() {
        assert_eq!(strip_date_prefix("20260408-foo"), "foo");
    }

    #[test]
    fn strip_date_prefix_no_prefix() {
        assert_eq!(strip_date_prefix("no-prefix-here"), "no-prefix-here");
    }

    #[test]
    fn strip_date_prefix_short() {
        assert_eq!(strip_date_prefix("short"), "short");
    }

    #[test]
    fn strip_date_prefix_empty() {
        assert_eq!(strip_date_prefix(""), "");
    }

    // ── fuzzy stem matching tests ──────────────────────────────────────────

    #[test]
    fn resolve_artifact_path_fuzzy_strips_candidate_prefix() {
        let tmp = std::env::temp_dir().join(format!("ct-fuzzy-cand-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let file = create_artifact_file(
            &tmp,
            "myproj",
            ArtifactKind::Doc,
            "20260411-07-kdl-derive-reference",
        );

        with_blueprints_dir(&tmp, || {
            let result = resolve_artifact_path("kdl-derive-reference", ArtifactKind::Doc)
                .expect("resolve kdl stem");
            assert_eq!(result, file);
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn resolve_artifact_path_exact_takes_priority_over_fuzzy() {
        let tmp = std::env::temp_dir().join(format!("ct-fuzzy-exact-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let exact = create_artifact_file(
            &tmp,
            "myproj",
            ArtifactKind::Doc,
            "20260411-07-kdl-derive-reference",
        );
        // Another file whose slug also matches — should not cause ambiguity
        let _other = create_artifact_file(
            &tmp,
            "other",
            ArtifactKind::Doc,
            "20260412-07-kdl-derive-reference",
        );

        with_blueprints_dir(&tmp, || {
            let result =
                resolve_artifact_path("20260411-07-kdl-derive-reference", ArtifactKind::Doc)
                    .expect("resolve kdl stem exact");
            assert_eq!(result, exact);
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn resolve_artifact_path_fuzzy_strips_query_prefix() {
        let tmp = std::env::temp_dir().join(format!("ct-fuzzy-query-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let file = create_artifact_file(&tmp, "myproj", ArtifactKind::Doc, "foo");

        with_blueprints_dir(&tmp, || {
            let result = resolve_artifact_path("20260411-07-foo", ArtifactKind::Doc)
                .expect("resolve foo stem fuzzy");
            assert_eq!(result, file);
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn universal_resolve_fuzzy_strips_candidate_prefix() {
        let tmp = std::env::temp_dir().join(format!("ct-univ-fuzzy-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let file = create_artifact_file(
            &tmp,
            "myproj",
            ArtifactKind::Doc,
            "20260411-07-kdl-derive-reference",
        );

        with_blueprints_dir(&tmp, || {
            let result =
                resolve_stem_universal("kdl-derive-reference").expect("universal resolve kdl");
            assert_eq!(result, file);
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn universal_resolve_fuzzy_strips_query_prefix() {
        let tmp = std::env::temp_dir().join(format!("ct-univ-fuzzy-q-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let file = create_artifact_file(&tmp, "myproj", ArtifactKind::Plan, "bar");

        with_blueprints_dir(&tmp, || {
            let result = resolve_stem_universal("20260411-07-bar").expect("universal resolve bar");
            assert_eq!(result, file);
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn retag_fixes_wrong_auto_derived_tags() {
        let tmp = std::env::temp_dir().join(format!("ct-retag-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let docs_dir = tmp.join("myproj").join("docs");
        std::fs::create_dir_all(&docs_dir).unwrap();

        let file = docs_dir.join("widget.md");
        let content = "---\ntopic: Widget Guide\ntags:\n  - type/spec\n  - project/wrong\n  - domain/ui\n---\n# Widget\n";
        std::fs::write(&file, content).unwrap();

        with_blueprints_dir(&tmp, || {
            // cmd_retag calls commit_and_push which fails in test (no git repo),
            // but the file is already rewritten before that call.
            let _ = cmd_retag(ArtifactKind::Doc, file.to_str().unwrap());

            let result = std::fs::read_to_string(&file).unwrap();
            assert!(
                result.contains("  - type/docs"),
                "type tag should be corrected to type/docs; got:\n{result}"
            );
            assert!(
                result.contains("  - project/myproj"),
                "project tag should be corrected to project/myproj; got:\n{result}"
            );
            assert!(
                result.contains("  - domain/ui"),
                "non-auto-derived tags should be preserved; got:\n{result}"
            );
            assert!(
                !result.contains("  - type/spec"),
                "old type tag should be removed; got:\n{result}"
            );
            assert!(
                !result.contains("  - project/wrong"),
                "old project tag should be removed; got:\n{result}"
            );
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn retag_noop_when_tags_correct() {
        let tmp = std::env::temp_dir().join(format!("ct-retag-noop-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let docs_dir = tmp.join("myproj").join("docs");
        std::fs::create_dir_all(&docs_dir).unwrap();

        let file = docs_dir.join("correct.md");
        let content = "---\ntopic: Already Correct\ntags:\n  - type/docs\n  - project/myproj\n---\n# Correct\n";
        std::fs::write(&file, content).unwrap();

        with_blueprints_dir(&tmp, || {
            let result = cmd_retag(ArtifactKind::Doc, file.to_str().unwrap());
            assert!(result.is_ok(), "should return Ok when no changes needed");

            let after = std::fs::read_to_string(&file).unwrap();
            assert_eq!(after, content, "file should be unchanged");
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn read_returns_structured_frontmatter_and_body() {
        let tmp = std::env::temp_dir().join(format!("ct-read-core-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let file = tmp.join("sample.md");
        std::fs::write(
            &file,
            "---\ntopic: My Topic\nauthor: me\ncreated: 2026-04-16\nsource: \"[[ref]]\"\ntags:\n  - type/spec\n  - domain/x\n---\nbody line 1\n<!--todo-->\n",
        )
        .unwrap();

        let outcome = read(&file).expect("read ok");
        assert_eq!(outcome.path, file);
        assert_eq!(outcome.body, "body line 1\n<!--todo-->\n");
        assert_eq!(outcome.comments.len(), 1, "one inline comment");
        let fm = outcome.frontmatter.expect("frontmatter present");
        assert_eq!(fm.topic.as_deref(), Some("My Topic"));
        assert_eq!(fm.author.as_deref(), Some("me"));
        assert_eq!(fm.created.as_deref(), Some("2026-04-16"));
        assert_eq!(fm.source.as_deref(), Some("ref"));
        assert_eq!(fm.tags, vec!["type/spec", "domain/x"]);

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn create_returns_outcome_with_populated_fields() {
        let tmp = std::env::temp_dir().join(format!("ct-create-core-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let project = tmp.join("myproj");
        std::fs::create_dir_all(&project).unwrap();

        with_blueprints_dir(&tmp, || {
            // commit_and_push will fail (no git repo) — we assert on Err fields
            // via the filesystem state after the write.
            let outcome = create(CreateOpts {
                kind: ArtifactKind::Plan,
                topic: "New Plan",
                project: project.to_str().unwrap(),
                slug_override: Some("new-plan"),
                source: None,
                user_tags: &[],
                dive: false,
            });
            // Either we got CreateOutcome (push succeeded — unlikely in test)
            // or a SyncError (expected). In both cases the file is written.
            let plan_dir = tmp.join("myproj").join("plan");
            let files: Vec<_> = fs::read_dir(&plan_dir)
                .expect("plan/ exists")
                .flatten()
                .collect();
            assert_eq!(files.len(), 1, "exactly one plan file written");

            if let Ok(o) = outcome {
                assert_eq!(o.kind, ArtifactKind::Plan);
                assert_eq!(o.project, "myproj");
                assert!(o.pushed);
                assert!(o.path.exists());
            }
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn archive_core_moves_file_and_returns_destination() {
        let tmp = std::env::temp_dir().join(format!("ct-archive-core-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let spec_dir = tmp.join("myproj").join("spec");
        std::fs::create_dir_all(&spec_dir).unwrap();
        let spec_file = spec_dir.join("20260411-target.md");
        std::fs::write(&spec_file, "---\ntopic: T\n---\n").unwrap();

        with_blueprints_dir(&tmp, || {
            // commit_and_push will fail in the test, so `archive` returns Err
            // after the move already happened. Assert filesystem either way.
            let _ = archive(ArtifactKind::Spec, &spec_file);
            let expected = tmp
                .join("myproj")
                .join("archive")
                .join("spec")
                .join("20260411-target.md");
            assert!(expected.exists(), "file moved to archive/spec/");
            assert!(!spec_file.exists(), "source file removed");
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn resolve_artifact_path_returns_not_found_error() {
        let tmp = std::env::temp_dir().join(format!("ct-nf-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        with_blueprints_dir(&tmp, || {
            let err = resolve_artifact_path("nonexistent-stem", ArtifactKind::Plan)
                .expect_err("no match -> Err");
            assert!(matches!(err, ResolveError::NotFound(_)));
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn resolve_stem_universal_returns_ambiguous_when_same_kind_multiple_projects() {
        let tmp = std::env::temp_dir().join(format!("ct-univ-ambig-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let _a = create_artifact_file(&tmp, "p1", ArtifactKind::Plan, "duplicated");
        let _b = create_artifact_file(&tmp, "p2", ArtifactKind::Plan, "duplicated");

        with_blueprints_dir(&tmp, || {
            let err = resolve_stem_universal("duplicated").expect_err("ambiguous");
            let ResolveError::Ambiguous(matches) = err else {
                panic!("expected Ambiguous");
            };
            assert_eq!(matches.len(), 2, "both duplicated matches returned");
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn ct_error_wraps_component_errors() {
        // Just exercise the From impls and Display — keeps the CtError variants
        // off the dead-code list and guards against their Display drifting.
        let sync: CtError = SyncError::Push("x".to_string()).into();
        assert!(sync.to_string().contains("push"));
        let resolve: CtError = ResolveError::NotFound("y".to_string()).into();
        assert!(resolve.to_string().contains("not found"));
        let io: CtError = std::io::Error::other("z").into();
        assert!(io.to_string().contains('z'));
        let val = CtError::Validation("bad".to_string());
        assert_eq!(val.to_string(), "bad");
    }

    // ── security regression tests ───────────────────────────────────────────

    #[test]
    fn resolve_stem_universal_rejects_path_traversal() {
        // Pre-fix this returned /etc/passwd because exists() was the only gate.
        // The vault dir has to exist for blueprints_dir() not to fatal.
        let tmp = std::env::temp_dir().join(format!("ct-sec-traverse-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        with_blueprints_dir(&tmp, || {
            let err = resolve_stem_universal("../../../etc/passwd")
                .expect_err("path traversal must be rejected");
            assert!(
                matches!(err, ResolveError::NotFound(_)),
                "expected NotFound, got {err:?}"
            );
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn resolve_artifact_path_rejects_path_traversal() {
        let tmp = std::env::temp_dir().join(format!("ct-sec-artpath-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        with_blueprints_dir(&tmp, || {
            let err = resolve_artifact_path("../../../etc/passwd", ArtifactKind::Spec)
                .expect_err("path traversal must be rejected");
            assert!(
                matches!(err, ResolveError::NotFound(_)),
                "expected NotFound, got {err:?}"
            );
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn create_sanitizes_slug_override_with_path_separator() {
        // "../evil" used to pass through verbatim into the filename. After the
        // fix the slug sanitizer must neutralize it: no file lands outside the
        // per-project spec/ directory. A git-push sync error is fine — the
        // write already happened, so inspect the filesystem to verify no
        // traversal occurred.
        let tmp = std::env::temp_dir().join(format!("ct-sec-slug-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let project = tmp.join("myproj");
        std::fs::create_dir_all(&project).unwrap();
        with_blueprints_dir(&tmp, || {
            let outcome = create(CreateOpts {
                kind: ArtifactKind::Spec,
                topic: "Harmless Topic",
                project: project.to_str().unwrap(),
                slug_override: Some("../evil"),
                source: None,
                user_tags: &[],
                dive: false,
            });
            // Validation rejection is the strong outcome; a sync failure is
            // also acceptable as long as whatever got written stays in vault.
            match outcome {
                Err(CtError::Validation(_)) => return,
                Ok(_) | Err(CtError::Sync(_)) => {}
                Err(e) => panic!("unexpected error variant: {e:?}"),
            }
            let spec_dir = tmp.join("myproj").join("spec");
            let files: Vec<PathBuf> = fs::read_dir(&spec_dir)
                .map(|d| d.flatten().map(|e| e.path()).collect())
                .unwrap_or_default();
            assert!(
                !files.is_empty(),
                "sanitized slug should have produced a file under {}",
                spec_dir.display()
            );
            for path in &files {
                assert!(
                    path.starts_with(&spec_dir),
                    "file escaped project/spec: {}",
                    path.display()
                );
                let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                assert!(!fname.contains('/'), "slug leaked / into {fname}");
                assert!(!fname.contains('\\'), "slug leaked \\ into {fname}");
                assert!(!fname.contains(".."), "slug leaked .. into {fname}");
            }
            // No sibling archive/etc/passwd-style spillover under the vault.
            let bad = tmp.join("etc");
            assert!(!bad.exists(), "traversal produced {}", bad.display());
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn create_rejects_whitespace_slug_override() {
        // Pre-fix whitespace passed is_empty() and produced filenames like
        // "20260417-03-   .md".
        let tmp = std::env::temp_dir().join(format!("ct-sec-ws-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let project = tmp.join("myproj");
        std::fs::create_dir_all(&project).unwrap();
        with_blueprints_dir(&tmp, || {
            let outcome = create(CreateOpts {
                kind: ArtifactKind::Spec,
                topic: "Topic",
                project: project.to_str().unwrap(),
                slug_override: Some("   "),
                source: None,
                user_tags: &[],
                dive: false,
            });
            assert!(
                matches!(outcome, Err(CtError::Validation(_))),
                "expected Validation for whitespace slug, got {outcome:?}"
            );
        });
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn create_rejects_duplicate_same_hour() {
        // Two creates in the same hour with the same slug collide at the
        // filename level; the second must error instead of clobbering.
        let tmp = std::env::temp_dir().join(format!("ct-sec-dupe-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let project = tmp.join("myproj");
        std::fs::create_dir_all(&project).unwrap();
        with_blueprints_dir(&tmp, || {
            // First create writes the file. commit_and_push will fail (no git
            // repo) — we only need the write to land.
            let first = create(CreateOpts {
                kind: ArtifactKind::Plan,
                topic: "Some Plan",
                project: project.to_str().unwrap(),
                slug_override: Some("fixed-slug"),
                source: None,
                user_tags: &[],
                dive: false,
            });
            let path_exists = match &first {
                Ok(o) => o.path.exists(),
                Err(CtError::Sync(_)) => {
                    // Sync error after successful write — find the file manually.
                    let plan_dir = tmp.join("myproj").join("plan");
                    fs::read_dir(&plan_dir)
                        .map(|d| d.flatten().next().is_some())
                        .unwrap_or(false)
                }
                Err(e) => panic!("unexpected first-create error: {e:?}"),
            };
            assert!(path_exists, "first create should have written the file");

            let second = create(CreateOpts {
                kind: ArtifactKind::Plan,
                topic: "Some Plan",
                project: project.to_str().unwrap(),
                slug_override: Some("fixed-slug"),
                source: None,
                user_tags: &[],
                dive: false,
            });
            assert!(
                matches!(second, Err(CtError::Validation(ref m)) if m.contains("already exists")),
                "second create must error with 'already exists', got {second:?}"
            );
        });
        std::fs::remove_dir_all(&tmp).ok();
    }
}
