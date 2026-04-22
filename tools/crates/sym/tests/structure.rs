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
    let service = result
        .top_by_import_fan
        .iter()
        .find(|file| file.rel_path == "service/service.go")
        .expect("service should appear in import fan-in");
    assert_eq!(service.count, 2, "service is imported by handler and worker");
    assert!(!result.top_packages.is_empty());

    Ok(())
}

#[test]
fn import_fan_credits_stem_only_matches_and_dedupes_importers() -> Result<()> {
    let root = tempfile::tempdir()?;
    fs::create_dir(root.path().join(".git"))?;

    // `utils.go` is referenced via bare stem "utils" in two other files. Each importer
    // has multiple import rows naming utils — the fan-in should dedupe to 2 (distinct
    // importers), not 4 (distinct import rows).
    write(
        root.path(),
        "utils/utils.go",
        "package utils\n\nfunc Helper() {}\n",
    )?;
    write(
        root.path(),
        "a/a.go",
        "package a\n\nimport (\n    \"utils\"\n    \"utils/utils\"\n)\n\nfunc A() { utils.Helper() }\n",
    )?;
    write(
        root.path(),
        "b/b.go",
        "package b\n\nimport (\n    \"utils\"\n    \"other/utils\"\n)\n\nfunc B() { utils.Helper() }\n",
    )?;

    let result = structure::analyze(root.path(), 10)?;
    let utils = result
        .top_by_import_fan
        .iter()
        .find(|file| file.rel_path == "utils/utils.go")
        .expect("utils should appear in import fan-in");
    assert_eq!(utils.count, 2, "two distinct importers, regardless of row count");
    Ok(())
}

#[test]
fn top_by_refs_counts_each_reference_once_across_definers() -> Result<()> {
    // Two files both declare a type named `Widget`; a third calls `Widget()` once.
    // The count should be 1 (actual ref count), not 2 (inflated by definer count).
    let root = tempfile::tempdir()?;
    fs::create_dir(root.path().join(".git"))?;

    write(
        root.path(),
        "pkgA/widget.go",
        "package pkgA\n\ntype Widget struct{}\n",
    )?;
    write(
        root.path(),
        "pkgB/widget.go",
        "package pkgB\n\ntype Widget struct{}\n",
    )?;
    write(
        root.path(),
        "user/user.go",
        "package user\n\nimport \"pkgA\"\n\nfunc Use() { _ = pkgA.Widget{} }\n",
    )?;

    let result = structure::analyze(root.path(), 20)?;
    for entry in result.top_by_refs.iter().filter(|e| e.symbol.name == "Widget") {
        assert_eq!(entry.count, 1, "Widget is referenced once, regardless of definer count");
    }
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
