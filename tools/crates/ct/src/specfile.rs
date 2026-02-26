use std::env;
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::process;
use std::time::SystemTime;

use crate::artifact;

fn fatal(msg: &str) -> ! {
    eprintln!("specfile: {msg}");
    process::exit(1);
}

pub fn specs_dir(project_path: &str) -> PathBuf {
    artifact::artifact_dir(project_path, "specs")
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

    let dir = specs_dir(&project);
    fs::create_dir_all(&dir).unwrap_or_else(|e| fatal(&format!("cannot create directory: {e}")));

    let full_path = dir.join(&filename);

    if body.is_empty() && !io::stdin().is_terminal() {
        io::stdin()
            .read_to_string(&mut body)
            .unwrap_or_else(|e| fatal(&format!("reading stdin: {e}")));
    }

    let now = artifact::chrono_rfc3339();

    let mut buf = String::new();
    buf.push_str("---\n");
    buf.push_str(&format!("topic: {}\n", artifact::yaml_quote(&topic)));
    buf.push_str(&format!("project: {}\n", artifact::yaml_quote(&project)));
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
        fatal("usage: specfile read [--frontmatter] <file>");
    }

    let content =
        fs::read_to_string(&file_path).unwrap_or_else(|e| fatal(&format!("reading file: {e}")));

    let (yaml, body) = artifact::parse_frontmatter(&content);

    if frontmatter_mode {
        match yaml {
            None => println!("{{}}"),
            Some(y) => {
                let pairs = artifact::parse_yaml_map(y);
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

pub fn latest_spec(task_file: Option<&str>, project: &str) -> Result<PathBuf, String> {
    if let Some(tf) = task_file {
        let p = PathBuf::from(tf);
        if p.exists() {
            return Ok(p);
        }
        return Err(format!("task-file not found: {tf}"));
    }

    let dir = specs_dir(project);
    let entries = fs::read_dir(&dir)
        .map_err(|e| format!("cannot read specs directory {}: {e}", dir.display()))?;

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

    latest_path.ok_or_else(|| format!("no spec files found in {}", dir.display()))
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

    match latest_spec(task_file.as_deref(), &project) {
        Ok(p) => println!("{}", p.display()),
        Err(e) => fatal(&e),
    }
}

pub fn cmd_archive(args: &[String]) {
    let file_path = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .unwrap_or_else(|| fatal("usage: specfile archive <file>"));

    let path = Path::new(file_path);
    if !path.exists() {
        fatal(&format!("file not found: {file_path}"));
    }

    let content = fs::read_to_string(path).unwrap_or_else(|e| fatal(&format!("reading file: {e}")));

    let (yaml, _) = artifact::parse_frontmatter(&content);
    let project = yaml
        .map(|y| {
            artifact::parse_yaml_map(y)
                .into_iter()
                .find(|(k, _)| k == "project")
                .map(|(_, v)| v)
                .unwrap_or_default()
        })
        .unwrap_or_default();

    if project.is_empty() {
        fatal("spec has no project field — cannot determine git repo");
    }

    let git_dir = process::Command::new("git")
        .args(["-C", &project, "rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| fatal(&format!("not a git repository: {project}")));

    let note_status = process::Command::new("git")
        .args(["-C", &git_dir, "notes", "--ref=specs", "append", "-F"])
        .arg(path)
        .arg("HEAD")
        .status()
        .unwrap_or_else(|e| fatal(&format!("running git notes: {e}")));

    if !note_status.success() {
        fatal("git notes append failed — spec file preserved");
    }

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
