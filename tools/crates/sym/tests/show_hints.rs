use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Result;

#[test]
fn show_supports_file_symbol_hint() -> Result<()> {
    let fixture = Fixture::new()?;

    let output = run_sym(fixture.root(), ["show", "docs/config.go:Config"])?;

    assert!(output.contains("type Config struct{}"));

    Ok(())
}

#[test]
fn show_supports_parent_symbol_hint() -> Result<()> {
    let fixture = Fixture::new()?;

    let output = run_sym(fixture.root(), ["show", "Cache.read"])?;

    assert!(output.contains("read(): void {}"));
    assert!(!output.contains("export function read()"));

    Ok(())
}

struct Fixture {
    root: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Result<Self> {
        let root = tempfile::tempdir()?;
        fs::create_dir(root.path().join(".git"))?;
        write(root.path(), "src/config.go", "package main\n\ntype Config struct{}\n")?;
        write(root.path(), "docs/config.go", "package docs\n\ntype Config struct{}\n")?;
        write(
            root.path(),
            "src/cache.ts",
            "export class Cache {\n  read(): void {}\n}\n\nexport function read(): void {}\n",
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
