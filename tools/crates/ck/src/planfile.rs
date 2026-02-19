use std::env;
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::process;
use std::time::SystemTime;

fn fatal(msg: &str) -> ! {
    eprintln!("planfile: {msg}");
    process::exit(1);
}

pub fn project_name(project_path: &str) -> String {
    if project_path.is_empty() {
        return String::from("(no project)");
    }
    let path = Path::new(project_path);
    let components: Vec<&str> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    // Walk backwards looking for a `.git` directory component.
    // If found, use `{repo}-{remaining}` so worktrees get unique names.
    // e.g. /src/arc.git/wt1 → "arc-wt1", /src/arc.git/wt2 → "arc-wt2"
    for (i, comp) in components.iter().enumerate() {
        if comp.ends_with(".git") {
            let stem = comp.strip_suffix(".git").unwrap_or(comp);
            let rest: Vec<&str> = components[i + 1..].to_vec();
            if rest.is_empty() {
                return stem.to_string();
            }
            return format!("{}-{}", stem, rest.join("-"));
        }
    }

    // No .git parent — use the last component as before.
    path.file_name()
        .unwrap_or_else(|| fatal("invalid project path"))
        .to_string_lossy()
        .to_string()
}

pub fn plans_dir(project_path: &str) -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| fatal("cannot determine home directory"));
    plans_dir_with_base(project_path, Path::new(&home))
}

pub fn plans_dir_with_base(project_path: &str, base: &Path) -> PathBuf {
    let name = project_name(project_path);
    base.join(".claude").join("plans").join(name)
}

fn yaml_quote(s: &str) -> String {
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
            // Strip quotes
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

pub fn cmd_create(args: &[String]) {
    let mut topic = String::new();
    let mut project = String::new();
    let mut slug_flag = String::new();
    let mut prefix = String::new();
    let mut body = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--topic" => {
                i += 1;
                topic = args.get(i).cloned().unwrap_or_default();
            }
            "--project" => {
                i += 1;
                project = args.get(i).cloned().unwrap_or_default();
            }
            "--slug" => {
                i += 1;
                slug_flag = args.get(i).cloned().unwrap_or_default();
            }
            "--prefix" => {
                i += 1;
                prefix = args.get(i).cloned().unwrap_or_default();
            }
            "--body" => {
                i += 1;
                body = args.get(i).cloned().unwrap_or_default();
            }
            _ => {}
        }
        i += 1;
    }

    if topic.is_empty() {
        fatal("--topic is required");
    }
    if project.is_empty() {
        fatal("--project is required");
    }

    let s = if slug_flag.is_empty() {
        crate::slug::slug(&topic)
    } else {
        slug_flag
    };
    if s.is_empty() {
        fatal("could not derive slug from topic");
    }

    let filename = if prefix.is_empty() {
        format!("{s}.md")
    } else {
        format!("{prefix}-{s}.md")
    };

    let dir = plans_dir(&project);
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
    buf.push_str(&format!("topic: {}\n", yaml_quote(&topic)));
    buf.push_str(&format!("project: {}\n", yaml_quote(&project)));
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
}

fn chrono_rfc3339() -> String {
    // ISO 8601 / RFC 3339 without external deps
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // days since epoch
    let mut days = (secs / 86400) as i64;
    let day_secs = (secs % 86400) as u32;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;

    // year/month/day from days since epoch
    days += 719468; // shift to year 0
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

pub fn cmd_read(args: &[String]) {
    let mut frontmatter_mode = false;
    let mut file_path = String::new();

    for arg in args {
        match arg.as_str() {
            "--frontmatter" => frontmatter_mode = true,
            _ if !arg.starts_with('-') && file_path.is_empty() => file_path = arg.clone(),
            _ => {}
        }
    }

    if file_path.is_empty() {
        fatal("usage: planfile read [--frontmatter] <file>");
    }

    let content =
        fs::read_to_string(&file_path).unwrap_or_else(|e| fatal(&format!("reading file: {e}")));

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

/// Core logic for `ck plan latest`, extracted for testability.
/// Returns `Ok(path)` on success or `Err(message)` on failure.
pub fn latest_plan(task_file: Option<&str>, project: &str) -> Result<PathBuf, String> {
    // --task-file short-circuits the mtime heuristic entirely.
    if let Some(tf) = task_file {
        let p = PathBuf::from(tf);
        if p.exists() {
            return Ok(p);
        }
        return Err(format!("task-file not found: {tf}"));
    }

    let dir = plans_dir(project);
    let entries = fs::read_dir(&dir)
        .map_err(|e| format!("cannot read plans directory {}: {e}", dir.display()))?;

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

    latest_path.ok_or_else(|| format!("no plan files found in {}", dir.display()))
}

pub fn cmd_latest(args: &[String]) {
    let mut project = String::new();
    let mut task_file: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--project" => {
                i += 1;
                project = args.get(i).cloned().unwrap_or_default();
            }
            "--task-file" => {
                i += 1;
                task_file = args.get(i).cloned();
            }
            _ => {}
        }
        i += 1;
    }

    if project.is_empty() && task_file.is_none() {
        // Try git root
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

    match latest_plan(task_file.as_deref(), &project) {
        Ok(p) => println!("{}", p.display()),
        Err(e) => fatal(&e),
    }
}

pub fn cmd_archive(args: &[String]) {
    let file_path = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .unwrap_or_else(|| fatal("usage: planfile archive <file>"));

    let path = Path::new(file_path);
    if !path.exists() {
        fatal(&format!("file not found: {file_path}"));
    }

    let content = fs::read_to_string(path).unwrap_or_else(|e| fatal(&format!("reading file: {e}")));

    // Extract project path from frontmatter to locate the git repo
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
        fatal("plan has no project field — cannot determine git repo");
    }

    // Find the git toplevel for the project
    let git_dir = process::Command::new("git")
        .args(["-C", &project, "rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| fatal(&format!("not a git repository: {project}")));

    // Store plan content as a git note on HEAD under refs/notes/plans
    let note_status = process::Command::new("git")
        .args(["-C", &git_dir, "notes", "--ref=plans", "append", "-F"])
        .arg(path)
        .arg("HEAD")
        .status()
        .unwrap_or_else(|e| fatal(&format!("running git notes: {e}")));

    if !note_status.success() {
        fatal("git notes append failed — plan file preserved");
    }

    // Note stored successfully — move to archive/ subfolder
    let parent = path
        .parent()
        .unwrap_or_else(|| fatal("cannot determine parent directory"));
    let archive_dir = parent.join("archive");
    fs::create_dir_all(&archive_dir)
        .unwrap_or_else(|e| fatal(&format!("cannot create archive directory: {e}")));
    let file_name = path
        .file_name()
        .unwrap_or_else(|| fatal("cannot determine file name"));
    let dest = archive_dir.join(file_name);
    fs::rename(path, &dest).unwrap_or_else(|e| fatal(&format!("archiving file: {e}")));
    eprintln!("Archived: {file_path} → git notes + {}", dest.display());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn worktree_path_gets_repo_prefix() {
        assert_eq!(project_name("/Users/me/src/arc.git/wt1"), "arc-wt1");
        assert_eq!(project_name("/Users/me/src/arc.git/wt2"), "arc-wt2");
    }

    #[test]
    fn bare_git_dir_uses_stem() {
        assert_eq!(project_name("/Users/me/src/arc.git"), "arc");
    }

    #[test]
    fn nested_worktree_joins_all_segments() {
        assert_eq!(
            project_name("/Users/me/src/mono.git/apps/web"),
            "mono-apps-web"
        );
    }

    #[test]
    fn normal_path_uses_last_component() {
        assert_eq!(project_name("/Users/me/src/chromium/src/arc"), "arc");
    }

    #[test]
    fn task_file_returns_specified_path() {
        let tmp = env::temp_dir().join(format!("ck-latest-test-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let plan = tmp.join("my-plan.md");
        std::fs::write(&plan, "# plan\n").unwrap();

        let result = latest_plan(Some(plan.to_str().unwrap()), "");
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
        let result = latest_plan(Some("/nonexistent/path/plan.md"), "");
        assert!(result.is_err(), "expected Err for missing task-file");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("task-file not found"),
            "error message should mention task-file, got: {msg}"
        );
    }

    #[test]
    fn cmd_create_frontmatter_has_no_status_field() {
        let tmp = env::temp_dir().join(format!("ck-test-{}", std::process::id()));
        let project_path = "/some/project";

        // Use plans_dir_with_base to get the expected directory without mutating HOME.
        let project_dir = plans_dir_with_base(project_path, &tmp);
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

        // Confirm frontmatter does NOT contain a status field.
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
