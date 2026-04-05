use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::process;
use std::time::SystemTime;

fn fatal(msg: &str) -> ! {
    eprintln!("artifact: {msg}");
    process::exit(1);
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
}

impl ArtifactKind {
    pub fn dir_name(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Spec => "spec",
            Self::Review => "review",
            Self::Report => "report",
        }
    }

    pub fn notes_ref(self) -> &'static str {
        match self {
            Self::Plan => "plans",
            Self::Spec => "specs",
            Self::Review => "reviews",
            Self::Report => "reports",
        }
    }

    /// Legacy directory name used in ~/.claude/ (before blueprints migration).
    pub fn legacy_dir_name(self) -> &'static str {
        match self {
            Self::Plan => "plans",
            Self::Spec => "specs",
            Self::Review => "reviews",
            Self::Report => "reports",
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

/// Returns ~/blueprints/ or fatal if it doesn't exist.
pub fn blueprints_dir() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| fatal("cannot determine home directory"));
    let dir = Path::new(&home).join("blueprints");
    if !dir.is_dir() {
        fatal("~/blueprints/ does not exist. Run `ct blueprint init` first.");
    }
    dir
}

/// Returns ~/blueprints/ without checking existence (for init).
pub fn blueprints_dir_unchecked() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| fatal("cannot determine home directory"));
    Path::new(&home).join("blueprints")
}

/// Returns ~/blueprints/<project>/<kind>/
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
            return comp.strip_suffix(".git").unwrap_or(comp).to_string();
        }
    }

    path.file_name()
        .unwrap_or_else(|| fatal("invalid project path"))
        .to_string_lossy()
        .to_string()
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

pub fn chrono_rfc3339() -> String {
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    let mut days = (secs / 86400) as i64;
    let day_secs = (secs % 86400) as u32;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;

    days += 719468;
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = (days - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    format!("{year:04}-{m:02}-{d:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
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

/// Commit and push a file in the blueprints repo.
/// Failures are warnings, not fatal — the file is the source of truth.
pub fn commit_and_push(relative_path: &Path, message: &str) {
    let bp = blueprints_dir();
    let bp_str = bp.to_string_lossy();

    let add_ok = process::Command::new("git")
        .args(["-C", &bp_str, "add"])
        .arg(relative_path)
        .status()
        .is_ok_and(|s| s.success());

    if !add_ok {
        eprintln!("warning: git add failed in ~/blueprints/");
        return;
    }

    let commit_ok = process::Command::new("git")
        .args(["-C", &bp_str, "commit", "-m", message])
        .status()
        .is_ok_and(|s| s.success());

    if !commit_ok {
        // Nothing to commit is fine (duplicate content)
        return;
    }

    let push_ok = process::Command::new("git")
        .args(["-C", &bp_str, "push"])
        .status()
        .is_ok_and(|s| s.success());

    if !push_ok {
        eprintln!(
            "warning: git push failed in ~/blueprints/ — commit saved locally, push manually"
        );
    }
}

// ---------------------------------------------------------------------------
// Generic CRUD (replaces planfile.rs / specfile.rs)
// ---------------------------------------------------------------------------

pub fn cmd_create(
    kind: ArtifactKind,
    topic: &str,
    project: &str,
    slug_override: Option<&str>,
    prefix: Option<&str>,
    mut body: String,
) {
    let s = match slug_override {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => crate::slug::slug(topic),
    };
    if s.is_empty() {
        fatal("could not derive slug from topic");
    }

    let filename = match prefix {
        Some(p) if !p.is_empty() => format!("{p}-{s}.md"),
        _ => format!("{s}.md"),
    };

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
    buf.push_str(&format!("project: {}\n", yaml_quote(project)));
    buf.push_str(&format!("created: {now}\n"));
    buf.push_str("---\n");
    if !body.is_empty() {
        buf.push_str(&body);
        if !body.ends_with('\n') {
            buf.push('\n');
        }
    }

    fs::write(&full_path, &buf).unwrap_or_else(|e| fatal(&format!("writing file: {e}")));
    println!("{}", full_path.display());

    // Commit + push
    let proj_name = project_name(project);
    if let Ok(rel) = full_path.strip_prefix(blueprints_dir()) {
        commit_and_push(rel, &format!("{}({}): {}", kind.dir_name(), proj_name, s));
    }
}

pub fn cmd_read(file_path: &str, frontmatter_mode: bool) {
    let content =
        fs::read_to_string(file_path).unwrap_or_else(|e| fatal(&format!("reading file: {e}")));

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

pub fn cmd_latest(kind: ArtifactKind, project: Option<&str>, task_file: Option<&str>) {
    let mut project = project.unwrap_or("").to_string();

    if project.is_empty() && task_file.is_none() {
        let output = process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output();
        match output {
            Ok(o) if o.status.success() => {
                project = String::from_utf8_lossy(&o.stdout).trim().to_string();
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

pub fn cmd_archive(kind: ArtifactKind, file_path: &str) {
    let path = Path::new(file_path);
    if !path.exists() {
        fatal(&format!("file not found: {file_path}"));
    }

    let content = fs::read_to_string(path).unwrap_or_else(|e| fatal(&format!("reading file: {e}")));

    let (yaml, _) = parse_frontmatter(&content);
    let project = yaml
        .map(|y| {
            parse_yaml_map(y)
                .into_iter()
                .find(|(k, _)| k == "project")
                .map(|(_, v)| v)
                .unwrap_or_default()
        })
        .unwrap_or_default();

    if project.is_empty() {
        fatal(&format!(
            "{} has no project field — cannot determine git repo",
            kind.dir_name()
        ));
    }

    // Find the git toplevel for the project
    let git_dir = process::Command::new("git")
        .args(["-C", &project, "rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| fatal(&format!("not a git repository: {project}")));

    // Store content as a git note on HEAD under refs/notes/<kind>
    let note_status = process::Command::new("git")
        .args([
            "-C",
            &git_dir,
            "notes",
            &format!("--ref={}", kind.notes_ref()),
            "append",
            "-F",
        ])
        .arg(path)
        .arg("HEAD")
        .status()
        .unwrap_or_else(|e| fatal(&format!("running git notes: {e}")));

    if !note_status.success() {
        fatal(&format!(
            "git notes append failed — {} file preserved",
            kind.dir_name()
        ));
    }

    // Move to archive/ in the blueprints project dir
    let proj_name = project_name(&project);
    let bp = blueprints_dir();
    let archive_dir = bp.join(&proj_name).join("archive").join(kind.dir_name());
    fs::create_dir_all(&archive_dir)
        .unwrap_or_else(|e| fatal(&format!("cannot create archive directory: {e}")));
    let file_name = path
        .file_name()
        .unwrap_or_else(|| fatal("cannot determine file name"));
    let dest = archive_dir.join(file_name);
    fs::rename(path, &dest).unwrap_or_else(|e| fatal(&format!("archiving file: {e}")));
    eprintln!("Archived: {file_path} → git notes + {}", dest.display());

    // Commit + push
    if let Ok(rel) = dest.strip_prefix(&bp) {
        let slug = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        commit_and_push(rel, &format!("archive({}): {}", proj_name, slug));
    }
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
        let (title, fm_project) = extract_frontmatter(&path);
        // Use project from frontmatter if available (preserves full paths for filtering),
        // otherwise fall back to directory name.
        let project = if fm_project.is_empty() {
            fallback_project.to_string()
        } else {
            fm_project
        };
        out.push(Artifact {
            name,
            path,
            title,
            project,
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

    // Also check the current directory's git repo
    if let Some(cwd_project) = process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
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
// Blueprint init / migrate
// ---------------------------------------------------------------------------

pub fn cmd_blueprint_init() {
    let bp = blueprints_dir_unchecked();
    if bp.is_dir() {
        eprintln!("~/blueprints/ already exists");
        return;
    }

    fs::create_dir_all(&bp).unwrap_or_else(|e| fatal(&format!("cannot create ~/blueprints/: {e}")));

    let init_ok = process::Command::new("git")
        .args(["-C", &bp.to_string_lossy(), "init"])
        .status()
        .is_ok_and(|s| s.success());

    if !init_ok {
        fatal("git init failed in ~/blueprints/");
    }

    eprintln!("Initialized ~/blueprints/ as a git repository");
}

pub fn cmd_blueprint_migrate() {
    let bp = blueprints_dir();
    let home = env::var("HOME").unwrap_or_else(|_| fatal("cannot determine home directory"));

    let mut migrated = 0u32;

    for kind in [
        ArtifactKind::Plan,
        ArtifactKind::Spec,
        ArtifactKind::Review,
        ArtifactKind::Report,
    ] {
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

                // Copy to blueprints
                if let Err(e) = fs::copy(&src, &dest) {
                    eprintln!("warning: failed to copy {}: {e}", src.display());
                    continue;
                }

                // Archive original
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
        // Commit all migrated files
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

    eprintln!("Migrated {migrated} artifact(s) to ~/blueprints/");
}

pub fn cmd_blueprint_project() {
    let project = process::Command::new("git")
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
    println!("{}", project_name(&project));
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
        buf.push_str(&format!("project: {}\n", yaml_quote(project_path)));
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
}
