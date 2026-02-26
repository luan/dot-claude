use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Spec {
    pub name: String,
    pub path: PathBuf,
    pub title: String,
    pub project: String,
    pub mod_time: SystemTime,
    pub size: u64,
}

pub fn list_specs() -> Vec<Spec> {
    list_specs_filtered(false)
}

pub fn list_archived_specs() -> Vec<Spec> {
    list_specs_filtered(true)
}

fn list_specs_filtered(archived: bool) -> Vec<Spec> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    let dir = home.join(".claude").join("specs");

    let mut specs = Vec::new();
    collect_specs(&dir, &dir, archived, &mut specs);
    specs.sort_by_key(|a| std::cmp::Reverse(a.mod_time));
    specs
}

fn collect_specs(base: &Path, dir: &Path, archived: bool, out: &mut Vec<Spec>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let is_archive = path.file_name().is_some_and(|n| n == "archive");
            if is_archive != archived {
                if is_archive {
                    continue;
                }
                collect_specs(base, &path, archived, out);
                continue;
            }
            collect_specs(base, &path, archived, out);
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
        let (title, project) = extract_frontmatter(&path);
        out.push(Spec {
            name,
            path,
            title,
            project,
            mod_time: info.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            size: info.len(),
        });
    }
}

pub fn load_content(path: &PathBuf) -> String {
    let path_str = path.to_string_lossy();
    if let Some(rest) = path_str.strip_prefix("git-notes://") {
        if let Some(slash) = rest.rfind('/') {
            let git_dir = &rest[..slash];
            let commit_sha = &rest[slash + 1..];
            return Command::new("git")
                .args(["-C", git_dir, "notes", "--ref=specs", "show", commit_sha])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_else(|| "Error: could not read git note".to_string());
        }
        return "Error: invalid git-notes path".to_string();
    }
    fs::read_to_string(path).unwrap_or_else(|e| format!("Error loading spec: {e}"))
}

fn extract_frontmatter(path: &Path) -> (String, String) {
    let Ok(f) = fs::File::open(path) else {
        return (String::new(), String::new());
    };
    let reader = BufReader::new(f);
    let mut title = String::new();
    let mut project = String::new();
    let mut in_frontmatter = false;

    for line in reader.lines().map_while(Result::ok) {
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
                title = val.trim().to_string();
            } else if let Some(val) = trimmed.strip_prefix("project:") {
                project = val.trim().to_string();
            }
        } else if let Some(t) = trimmed.strip_prefix("# ") {
            title = t.to_string();
            break;
        }
    }
    (title, project)
}
