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

fn project_name(project_path: &str) -> String {
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

fn plans_dir(project_path: &str) -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| fatal("cannot determine home directory"));
    let base = project_name(project_path);
    PathBuf::from(home)
        .join(".claude")
        .join("plans")
        .join(base)
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

fn parse_frontmatter(content: &str) -> (Option<&str>, &str) {
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

fn parse_yaml_map(yaml: &str) -> Vec<(String, String)> {
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

fn cmd_create(args: &[String]) {
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
        claude_slug::slug(&topic)
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
    buf.push_str("status: draft\n");
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

fn cmd_read(args: &[String]) {
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

fn cmd_latest(args: &[String]) {
    let mut project = String::new();

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--project" {
            i += 1;
            project = args.get(i).cloned().unwrap_or_default();
        }
        i += 1;
    }

    if project.is_empty() {
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

    let dir = plans_dir(&project);
    let entries = fs::read_dir(&dir).unwrap_or_else(|_| {
        fatal(&format!(
            "no plans found for project {}",
            Path::new(&project)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ))
    });

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

    match latest_path {
        Some(p) => println!("{}", p.display()),
        None => fatal(&format!("no plan files found in {}", dir.display())),
    }
}

fn cmd_delete(args: &[String]) {
    let file_path = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .unwrap_or_else(|| fatal("usage: planfile delete <file>"));

    if !Path::new(file_path).exists() {
        fatal(&format!("file not found: {file_path}"));
    }

    fs::remove_file(file_path).unwrap_or_else(|e| fatal(&format!("deleting file: {e}")));
    eprintln!("Deleted: {file_path}");
}

const USAGE: &str = "Usage: planfile <command> [options]

Commands:
  create    Create a new plan file
  read      Read plan file body or frontmatter
  latest    Find most recently modified plan file
  delete    Delete a plan file

Run 'planfile <command> --help' for details.";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("{USAGE}");
        process::exit(1);
    }

    let cmd = &args[1];
    let rest = &args[2..];

    match cmd.as_str() {
        "create" => cmd_create(rest),
        "read" => cmd_read(rest),
        "latest" => cmd_latest(rest),
        "delete" => cmd_delete(rest),
        "--help" | "-h" | "help" => println!("{USAGE}"),
        _ => fatal(&format!(
            "unknown command: {cmd}\nRun 'planfile --help' for usage."
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worktree_path_gets_repo_prefix() {
        assert_eq!(
            project_name("/Users/me/src/arc.git/wt1"),
            "arc-wt1"
        );
        assert_eq!(
            project_name("/Users/me/src/arc.git/wt2"),
            "arc-wt2"
        );
    }

    #[test]
    fn bare_git_dir_uses_stem() {
        assert_eq!(
            project_name("/Users/me/src/arc.git"),
            "arc"
        );
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
        assert_eq!(
            project_name("/Users/me/src/chromium/src/arc"),
            "arc"
        );
    }
}
