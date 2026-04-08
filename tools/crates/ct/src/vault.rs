use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

use crate::artifact::{
    ALL_KINDS, blueprints_dir, blueprints_dir_unchecked, fatal, home_dir, project_name,
};

pub fn cmd_init() {
    let bp = blueprints_dir_unchecked();
    if bp.is_dir() {
        eprintln!("{} already exists", bp.display());
        return;
    }

    fs::create_dir_all(&bp)
        .unwrap_or_else(|e| fatal(&format!("cannot create {}: {e}", bp.display())));

    let init_ok = process::Command::new("git")
        .args(["-C", &bp.to_string_lossy(), "init"])
        .status()
        .is_ok_and(|s| s.success());

    if !init_ok {
        fatal(&format!("git init failed in {}", bp.display()));
    }

    eprintln!("Initialized {} as a git repository", bp.display());
}

pub fn cmd_migrate() {
    let bp = blueprints_dir();
    let home = home_dir();

    let mut migrated = 0u32;

    for kind in ALL_KINDS {
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

                if let Err(e) = fs::copy(&src, &dest) {
                    eprintln!("warning: failed to copy {}: {e}", src.display());
                    continue;
                }

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

    eprintln!("Migrated {migrated} artifact(s) to {}", bp.display());
}

pub fn cmd_project() {
    let toplevel = process::Command::new("git")
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
    let project = crate::artifact::resolve_repo_root(&toplevel);
    println!("{}", project_name(&project));
}

pub fn cmd_related(project: &str, topic: &str) {
    let topic_words: HashSet<&str> = topic
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3)
        .collect();

    if topic_words.is_empty() {
        return;
    }

    let bp = blueprints_dir();
    let resolved = crate::artifact::resolve_repo_root(project);
    let proj_name = project_name(&resolved);
    let proj_dir = bp.join(&proj_name);

    if !proj_dir.is_dir() {
        return;
    }

    let mut seen = HashSet::new();

    for kind in ALL_KINDS {
        let kind_dir = proj_dir.join(kind.dir_name());
        let Ok(entries) = fs::read_dir(&kind_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "md") {
                continue;
            }
            let stem = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            // Strip YYYYMMDD-HH- or legacy YYYYMMDD- date prefix for keyword matching
            let slug_part = if stem.len() > 12
                && stem[..8].chars().all(|c| c.is_ascii_digit())
                && stem.as_bytes()[8] == b'-'
                && stem[9..11].chars().all(|c| c.is_ascii_digit())
                && stem.as_bytes()[11] == b'-'
            {
                &stem[12..]
            } else if stem.len() > 9
                && stem[..8].chars().all(|c| c.is_ascii_digit())
                && stem.as_bytes()[8] == b'-'
            {
                &stem[9..]
            } else {
                &stem
            };
            let slug_words: HashSet<&str> = slug_part
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() >= 3)
                .collect();
            let overlap = topic_words.intersection(&slug_words).count();
            if (overlap >= 2 || (topic_words.len() <= 2 && overlap >= 1))
                && seen.insert(stem.clone())
            {
                println!("[[{stem}]]");
            }
        }
    }
}

pub fn cmd_check() {
    let bp = blueprints_dir();
    let status = process::Command::new("obsidian")
        .args(["unresolved"])
        .current_dir(&bp)
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(_) => eprintln!("obsidian unresolved reported issues"),
        Err(e) => eprintln!("failed to run obsidian cli: {e}"),
    }
}

pub fn cmd_search(query: &str, json: bool, kind_filter: Option<&str>, project: Option<&str>) {
    let bp = blueprints_dir();
    let mut args = vec!["search".to_string(), format!("query={query}")];
    if json {
        args.push("format=json".to_string());
    }
    let output = process::Command::new("obsidian")
        .args(&args)
        .current_dir(&bp)
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            let proj_prefix = project.map(|p| {
                let resolved = crate::artifact::resolve_repo_root(p);
                let name = project_name(&resolved);
                format!("{name}/")
            });
            let kind_dir = kind_filter.map(|k| match k {
                "doc" => "docs/".to_string(),
                other => format!("{other}/"),
            });
            for line in text.lines() {
                let matches_kind = kind_dir.as_deref().is_none_or(|d| line.contains(d));
                let matches_proj = proj_prefix.as_deref().is_none_or(|p| line.contains(p));
                if matches_kind && matches_proj {
                    println!("{line}");
                }
            }
        }
        Ok(o) => {
            eprint!("{}", String::from_utf8_lossy(&o.stderr));
        }
        Err(e) => eprintln!("failed to run obsidian cli: {e}"),
    }
}

pub fn cmd_status() {
    let bp = blueprints_dir();
    let bp_str = bp.to_string_lossy();

    // Git status: clean or dirty
    let status_output = process::Command::new("git")
        .args(["-C", &bp_str, "status", "--porcelain"])
        .output();
    match &status_output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            if text.trim().is_empty() {
                println!("working tree: clean");
            } else {
                let dirty_count = text.lines().count();
                println!("working tree: {dirty_count} dirty file(s)");
            }
        }
        Ok(_) | Err(_) => println!("working tree: unknown (git status failed)"),
    }

    // Unpushed commits
    let log_output = process::Command::new("git")
        .args(["-C", &bp_str, "log", "--oneline", "@{u}..HEAD"])
        .output();
    match &log_output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            let count = text.lines().filter(|l| !l.is_empty()).count();
            println!("unpushed commits: {count}");
        }
        // No upstream configured or other error — report 0
        Ok(_) | Err(_) => println!("unpushed commits: 0 (no upstream)"),
    }

    // Total artifact count
    let mut total = 0usize;
    let Ok(projects) = fs::read_dir(&bp) else {
        println!("artifacts: 0");
        return;
    };
    for proj_entry in projects.flatten() {
        let proj_dir = proj_entry.path();
        if !proj_dir.is_dir() {
            continue;
        }
        // Skip .git and hidden dirs
        if proj_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .starts_with('.')
        {
            continue;
        }
        for kind in ALL_KINDS {
            let kind_dir = proj_dir.join(kind.dir_name());
            let Ok(entries) = fs::read_dir(&kind_dir) else {
                continue;
            };
            for entry in entries.flatten() {
                if entry.path().extension().is_some_and(|ext| ext == "md") {
                    total += 1;
                }
            }
        }
    }
    println!("artifacts: {total}");
}
