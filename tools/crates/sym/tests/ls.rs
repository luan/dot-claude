use std::fs;
use std::path::Path;

use anyhow::Result;
use sym::ls;

#[test]
fn ls_tree_respects_depth_and_skips_ignored_dirs() -> Result<()> {
    let fixture = Fixture::new()?;

    let tree = ls::tree(fixture.root(), 2)?;

    let child_names = tree
        .children
        .iter()
        .map(|child| child.name.as_str())
        .collect::<Vec<_>>();
    assert!(child_names.contains(&"src"));
    assert!(child_names.contains(&"docs"));
    assert!(!child_names.contains(&"node_modules"));
    assert!(!child_names.contains(&".hidden"));

    let src = tree.children.iter().find(|child| child.name == "src").unwrap();
    assert!(src.children.iter().any(|child| child.name == "nested"));
    let nested = src.children.iter().find(|child| child.name == "nested").unwrap();
    assert!(nested.children.is_empty());

    Ok(())
}

#[test]
fn ls_stats_reports_repo_file_symbol_and_language_counts() -> Result<()> {
    let fixture = Fixture::new()?;

    let stats = ls::repo_stats(fixture.root())?;

    assert_eq!(stats.file_count, 2);
    assert!(stats.symbol_count >= 2);
    assert_eq!(stats.languages.get("go"), Some(&1));
    assert_eq!(stats.languages.get("python"), Some(&1));

    Ok(())
}

struct Fixture {
    root: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Result<Self> {
        let root = tempfile::tempdir()?;
        fs::create_dir(root.path().join(".git"))?;
        write(root.path(), "src/main.go", "package main\n\nfunc HandleRequest() {}\n")?;
        write(root.path(), "src/nested/worker.py", "def run():\n    pass\n")?;
        write(root.path(), "docs/readme.md", "# docs\n")?;
        write(root.path(), "node_modules/pkg/index.js", "module.exports = {}\n")?;
        write(root.path(), ".hidden/secret.go", "package hidden\n")?;
        Ok(Self { root })
    }

    fn root(&self) -> &Path {
        self.root.path()
    }
}

fn write(root: &Path, rel_path: &str, contents: &str) -> Result<()> {
    let path = root.join(rel_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}
