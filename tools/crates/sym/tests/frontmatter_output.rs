use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Result;

#[test]
fn search_text_mode_uses_frontmatter() -> Result<()> {
    let fixture = Fixture::new()?;

    let output = run_sym(fixture.root(), ["search", "Handle"])?;

    assert!(output.starts_with("---\n"));
    assert!(output.contains("query: Handle\n"));
    assert!(output.contains("result_count: 1\n"));
    assert!(output.contains("function Handle svc.go:3\n"));

    Ok(())
}

#[test]
fn ls_stats_text_mode_uses_frontmatter() -> Result<()> {
    let fixture = Fixture::new()?;

    let output = run_sym(fixture.root(), ["ls", "--stats"])?;

    assert!(output.starts_with("---\n"));
    assert!(output.contains(&format!("repo: {}\n", fixture.root().canonicalize()?.display())));
    assert!(output.contains("files: 2\n"));
    assert!(output.contains("symbols: 3\n"));
    assert!(output.contains("go: 2 files\n"));

    Ok(())
}

#[test]
fn outline_text_mode_uses_frontmatter() -> Result<()> {
    let fixture = Fixture::new()?;

    let output = run_sym(fixture.root(), ["outline", "svc.go"])?;

    assert!(output.starts_with("---\n"));
    assert!(output.contains("file: svc.go\n"));
    assert!(output.contains("symbol_count: 2\n"));
    assert!(output.contains("function Handle (L3-3)\n"));

    Ok(())
}

#[test]
fn importers_text_mode_uses_frontmatter() -> Result<()> {
    let fixture = Fixture::new()?;

    let output = run_sym(fixture.root(), ["importers", "svc.go"])?;

    assert!(output.starts_with("---\n"));
    assert!(output.contains("target: svc.go\n"));
    assert!(output.contains("importer_count: 1\n"));
    assert!(output.contains("api.go:svc\n"));

    Ok(())
}

struct Fixture {
    root: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Result<Self> {
        let root = tempfile::tempdir()?;
        fs::create_dir(root.path().join(".git"))?;
        write(
            root.path(),
            "svc.go",
            "package svc\n\nfunc Handle() {}\nfunc helper() {}\n",
        )?;
        write(
            root.path(),
            "api.go",
            "package api\n\nimport \"svc\"\n\nfunc Serve() {\n    Handle()\n}\n",
        )?;
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

fn run_sym<I, S>(cwd: &Path, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let output = Command::new(env!("CARGO_BIN_EXE_sym"))
        .args(args)
        .current_dir(cwd)
        .output()?;
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    Ok(String::from_utf8(output.stdout)?)
}
