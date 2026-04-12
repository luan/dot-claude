use std::path::{Path, PathBuf};
use std::process::Command;

const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "ts", "py", "go", "js", "tsx", "jsx", "c", "cpp", "h", "lua",
];
const IGNORED_DIRS: &[&str] = &[
    "target",
    "node_modules",
    ".git",
    "vendor",
    "build",
    "dist",
    "__pycache__",
];

fn is_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| SOURCE_EXTENSIONS.contains(&ext))
}

fn is_ignored_dir(name: &str) -> bool {
    IGNORED_DIRS.contains(&name)
}

fn count_loc(dir: &Path) -> usize {
    let mut total = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries = match std::fs::read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if !is_ignored_dir(&name_str) {
                    stack.push(path);
                }
            } else if is_source_file(&path)
                && let Ok(contents) = std::fs::read_to_string(&path)
            {
                total += contents.lines().count();
            }
        }
    }
    total
}

fn git_commits_since(dir: &Path, project_root: &Path, since: &str) -> usize {
    let rel = match dir.strip_prefix(project_root) {
        Ok(r) => r,
        Err(_) => return 0,
    };
    let output = Command::new("git")
        .args([
            "-C",
            &project_root.to_string_lossy(),
            "log",
            "--oneline",
            &format!("--since={since}"),
            "--",
            &rel.to_string_lossy(),
        ])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout);
            s.lines().filter(|l| !l.trim().is_empty()).count()
        }
        _ => 0,
    }
}

fn git_last_change(dir: &Path, project_root: &Path) -> Option<String> {
    let rel = match dir.strip_prefix(project_root) {
        Ok(r) => r,
        Err(_) => return None,
    };
    let output = Command::new("git")
        .args([
            "-C",
            &project_root.to_string_lossy(),
            "log",
            "-1",
            "--format=%aI",
            "--",
            &rel.to_string_lossy(),
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn has_source_files(dir: &Path) -> bool {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries = match std::fs::read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if !is_ignored_dir(&name_str) {
                    stack.push(path);
                }
            } else if is_source_file(&path) {
                return true;
            }
        }
    }
    false
}

fn discover_modules(project_root: &Path) -> Vec<PathBuf> {
    let mut modules = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Check crates/ directory
    let crates_dir = project_root.join("crates");
    if crates_dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(&crates_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && has_source_files(&path) {
                seen.insert(path.clone());
                modules.push(path);
            }
        }
    }

    // Check src/ directory
    let src_dir = project_root.join("src");
    if src_dir.is_dir() {
        let mut has_subdirs = false;
        if let Ok(entries) = std::fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && has_source_files(&path) {
                    has_subdirs = true;
                    if seen.insert(path.clone()) {
                        modules.push(path);
                    }
                }
            }
        }
        if !has_subdirs && has_source_files(&src_dir) && seen.insert(src_dir.clone()) {
            modules.push(src_dir);
        }
    }

    // Check other top-level directories
    if let Ok(entries) = std::fs::read_dir(project_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if is_ignored_dir(&name_str) {
                continue;
            }
            // Skip already-handled directories
            if name_str == "crates" || name_str == "src" {
                continue;
            }
            if has_source_files(&path) && seen.insert(path.clone()) {
                modules.push(path);
            }
        }
    }

    modules
}

fn git_toplevel() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !output.status.success() {
        return Err("not a git repository".into());
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(s))
}

pub fn run(
    project_root: Option<String>,
    since: String,
    min_loc: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = match project_root {
        Some(p) => PathBuf::from(p),
        None => git_toplevel()?,
    };

    let modules = discover_modules(&root);

    let mut entries: Vec<serde_json::Value> = Vec::new();

    for module_path in &modules {
        let loc = count_loc(module_path);
        if loc < min_loc {
            continue;
        }
        let commits = git_commits_since(module_path, &root, &since);
        let last_change = git_last_change(module_path, &root);

        let rel = module_path
            .strip_prefix(&root)
            .unwrap_or(module_path)
            .to_string_lossy()
            .to_string();

        entries.push(serde_json::json!({
            "module": rel,
            "loc": loc,
            "commits": commits,
            "last_change": last_change,
        }));
    }

    entries.sort_by(|a, b| {
        let a_loc = a["loc"].as_u64().unwrap_or(0);
        let b_loc = b["loc"].as_u64().unwrap_or(0);
        b_loc.cmp(&a_loc)
    });

    println!("{}", serde_json::to_string_pretty(&entries)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn count_loc_sums_source_files() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        fs::write(
            root.join("main.rs"),
            "fn main() {\n    println!(\"hi\");\n}\n",
        )
        .unwrap();
        fs::write(root.join("lib.py"), "def f():\n    pass\n").unwrap();
        // Non-source file should be ignored
        fs::write(root.join("readme.md"), "# Title\ntext\n").unwrap();

        assert_eq!(count_loc(root), 5); // 3 + 2
    }

    #[test]
    fn count_loc_skips_ignored_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "line1\nline2\n").unwrap();

        fs::create_dir_all(root.join("target/debug")).unwrap();
        fs::write(root.join("target/debug/build.rs"), "ignored\n").unwrap();

        fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
        fs::write(root.join("node_modules/pkg/index.js"), "ignored\n").unwrap();

        assert_eq!(count_loc(root), 2);
    }

    #[test]
    fn discover_modules_finds_crates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        fs::create_dir_all(root.join("crates/foo/src")).unwrap();
        fs::write(root.join("crates/foo/src/main.rs"), "fn main() {}\n").unwrap();

        fs::create_dir_all(root.join("crates/bar/src")).unwrap();
        fs::write(root.join("crates/bar/src/lib.rs"), "pub fn x() {}\n").unwrap();

        // target/ should not appear
        fs::create_dir_all(root.join("target/release")).unwrap();
        fs::write(root.join("target/release/build.rs"), "nope\n").unwrap();

        let modules = discover_modules(root);
        let rel_names: Vec<String> = modules
            .iter()
            .map(|p| p.strip_prefix(root).unwrap().to_string_lossy().to_string())
            .collect();

        assert!(rel_names.contains(&"crates/foo".to_string()));
        assert!(rel_names.contains(&"crates/bar".to_string()));
        assert!(!rel_names.iter().any(|n| n.contains("target")));
    }

    #[test]
    fn discover_modules_includes_src_itself_when_flat() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();

        let modules = discover_modules(root);
        let rel_names: Vec<String> = modules
            .iter()
            .map(|p| p.strip_prefix(root).unwrap().to_string_lossy().to_string())
            .collect();

        assert!(rel_names.contains(&"src".to_string()));
    }

    #[test]
    fn discover_modules_lists_src_subdirs_when_present() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        fs::create_dir_all(root.join("src/api")).unwrap();
        fs::write(root.join("src/api/handler.ts"), "export {};\n").unwrap();
        fs::create_dir_all(root.join("src/db")).unwrap();
        fs::write(root.join("src/db/query.ts"), "export {};\n").unwrap();

        let modules = discover_modules(root);
        let rel_names: Vec<String> = modules
            .iter()
            .map(|p| p.strip_prefix(root).unwrap().to_string_lossy().to_string())
            .collect();

        assert!(rel_names.contains(&"src/api".to_string()));
        assert!(rel_names.contains(&"src/db".to_string()));
    }
}
