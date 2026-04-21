use std::fs;
use std::path::Path;

use anyhow::Result;
use sym::graph;
use sym::symbols::{REF_KIND_CALL, REF_KIND_USE};

#[test]
fn graph_queries_auto_index_and_filter_references() -> Result<()> {
    let fixture = RepoFixture::new()?;

    let refs = graph::find_references(
        fixture.root(),
        "Handle",
        20,
        &["api.go".to_string()],
        &[],
    )?;

    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].rel_path, "api.go");

    Ok(())
}

#[test]
fn graph_queries_find_importers_and_impact() -> Result<()> {
    let fixture = RepoFixture::new()?;

    let importers = graph::find_importers(fixture.root(), "Handle", 2, 20, &[], &[])?;
    assert_eq!(importers.len(), 2);
    assert_eq!(importers[0].rel_path, "api.go");
    assert_eq!(importers[1].rel_path, "main.go");

    let by_path = graph::find_importers_by_path(fixture.root(), "svc.go", 2, 20)?;
    assert_eq!(by_path.len(), 2);

    let impact = graph::find_impact(fixture.root(), "Handle", 3, 20)?;
    assert_eq!(impact.len(), 2);
    assert_eq!(impact[0].caller, "Serve");
    assert_eq!(impact[1].caller, "main");

    Ok(())
}

#[test]
fn graph_queries_trace_defaults_to_calls_and_can_widen() -> Result<()> {
    let fixture = RepoFixture::new()?;

    let default = graph::find_trace(fixture.root(), "main", 3, 20, &[])?;
    assert!(default.iter().any(|edge| edge.callee == "Serve"));
    assert!(default.iter().any(|edge| edge.callee == "Handle"));
    assert!(default.iter().any(|edge| edge.callee == "DoWork"));
    assert!(!default.iter().any(|edge| edge.callee == "Widget"));

    let wide = graph::find_trace(fixture.root(), "main", 3, 20, &[REF_KIND_CALL, REF_KIND_USE])?;
    assert!(wide.iter().any(|edge| edge.callee == "Widget"));

    Ok(())
}

struct RepoFixture {
    temp_dir: tempfile::TempDir,
}

impl RepoFixture {
    fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        fs::create_dir(temp_dir.path().join(".git"))?;
        write(
            temp_dir.path(),
            "svc.go",
            "package svc\n\nimport \"worker\"\n\nfunc Handle() {\n    DoWork()\n    _ = Widget{}\n}\n",
        )?;
        write(
            temp_dir.path(),
            "worker.go",
            "package worker\n\nfunc DoWork() {}\n\ntype Widget struct{}\n",
        )?;
        write(
            temp_dir.path(),
            "api.go",
            "package api\n\nimport \"svc\"\n\nfunc Serve() {\n    Handle()\n}\n",
        )?;
        write(
            temp_dir.path(),
            "main.go",
            "package main\n\nimport \"api\"\n\nfunc main() {\n    Serve()\n}\n",
        )?;
        Ok(Self { temp_dir })
    }

    fn root(&self) -> &Path {
        self.temp_dir.path()
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
