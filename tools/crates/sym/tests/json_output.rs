use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use serde_json::Value;

#[test]
fn version_json_wraps_results_in_versioned_envelope() -> Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_sym"))
        .args(["--json", "version"])
        .output()?;

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let value: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(value["version"], "0.1");
    assert!(value["results"].as_str().unwrap_or_default().starts_with("sym "));
    Ok(())
}

#[test]
fn search_json_outputs_symbol_results() -> Result<()> {
    let fixture = Fixture::new()?;

    let output = Command::new(env!("CARGO_BIN_EXE_sym"))
        .current_dir(fixture.root())
        .args(["--json", "search", "HandleRequest"])
        .output()?;

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let value: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(value["version"], "0.1");
    let results = value["results"].as_array().unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0]["name"], "HandleRequest");
    assert_eq!(results[0]["kind"], "function");
    assert_eq!(results[0]["rel_path"], "src/main.go");
    Ok(())
}

#[test]
fn ls_stats_json_outputs_repo_stats() -> Result<()> {
    let fixture = Fixture::new()?;

    let output = Command::new(env!("CARGO_BIN_EXE_sym"))
        .current_dir(fixture.root())
        .args(["--json", "ls", "--stats"])
        .output()?;

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let value: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(value["version"], "0.1");
    let canonical_root = fixture.root().canonicalize()?;
    assert_eq!(value["results"]["path"], canonical_root.to_string_lossy().as_ref());
    assert_eq!(value["results"]["file_count"], 2);
    assert!(value["results"]["symbol_count"].as_u64().unwrap_or_default() >= 2);
    assert_eq!(value["results"]["languages"]["go"], 1);
    assert_eq!(value["results"]["languages"]["python"], 1);
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
        write(root.path(), "src/worker.py", "def run():\n    pass\n")?;
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
