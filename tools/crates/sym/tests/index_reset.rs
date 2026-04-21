use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Result;

#[test]
fn index_reset_replaces_invalid_existing_db() -> Result<()> {
    let fixture = Fixture::new()?;
    let db_path = fixture.root().join("broken.db");
    fs::write(&db_path, "not a sqlite database")?;

    let failed = Command::new(env!("CARGO_BIN_EXE_sym"))
        .current_dir(fixture.root())
        .args(["--db", db_path.to_string_lossy().as_ref(), "index", "."])
        .output()?;
    assert!(
        !failed.status.success(),
        "plain index unexpectedly succeeded: {}",
        String::from_utf8_lossy(&failed.stdout)
    );

    let reset = Command::new(env!("CARGO_BIN_EXE_sym"))
        .current_dir(fixture.root())
        .args([
            "--db",
            db_path.to_string_lossy().as_ref(),
            "index",
            ".",
            "--reset",
        ])
        .output()?;

    assert!(reset.status.success(), "{}", String::from_utf8_lossy(&reset.stderr));
    assert!(db_path.exists());
    assert!(String::from_utf8_lossy(&reset.stdout).contains("Indexed 1 parseable files"));

    Ok(())
}

struct Fixture {
    root: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Result<Self> {
        let root = tempfile::tempdir()?;
        fs::create_dir(root.path().join(".git"))?;
        write(root.path(), "main.go", "package main\n\nfunc HandleRequest() {}\n")?;
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
