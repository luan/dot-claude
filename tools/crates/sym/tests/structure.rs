use std::fs;
use std::path::Path;

use anyhow::Result;
use sym::structure;

#[test]
fn structure_reports_entry_points_hotspots_import_fan_and_packages() -> Result<()> {
    let fixture = Fixture::new()?;

    let result = structure::analyze(fixture.root(), 10)?;

    assert_eq!(result.files, 4);
    assert!(result.symbols >= 4);
    assert!(result.entry_points.iter().any(|symbol| symbol.name == "main"));
    assert!(result.top_by_refs.iter().any(|symbol| symbol.symbol.name == "Run" || symbol.symbol.name == "Handle"));
    assert!(result.top_by_import_fan.iter().any(|file| file.rel_path == "service/service.go"));
    assert!(!result.top_packages.is_empty());

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
            "cmd/main.go",
            "package main\n\nimport (\n    \"repo/handler\"\n    \"repo/worker\"\n)\n\nfunc main() {\n    handler.Handle()\n    worker.RunWorker()\n}\n",
        )?;
        write(
            root.path(),
            "handler/handler.go",
            "package handler\n\nimport \"repo/service\"\n\nfunc Handle() {\n    service.Run()\n}\n",
        )?;
        write(
            root.path(),
            "worker/worker.go",
            "package worker\n\nimport \"repo/service\"\n\nfunc RunWorker() {\n    service.Run()\n}\n",
        )?;
        write(
            root.path(),
            "service/service.go",
            "package service\n\nfunc Run() {}\n",
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
