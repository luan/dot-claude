use std::fs;
use std::path::Path;

use anyhow::Result;
use sym::resolve;

#[test]
fn resolve_prefers_canonical_symbol_over_docs_and_tests() -> Result<()> {
    let fixture = Fixture::new()?;

    let result = resolve::resolve_symbol(fixture.root(), "Config")?;

    assert_eq!(result.symbol.rel_path, "src/config.go");
    assert_eq!(result.total_found, 3);
    assert!(!result.fuzzy);

    Ok(())
}

#[test]
fn resolve_supports_file_and_parent_hints() -> Result<()> {
    let fixture = Fixture::new()?;

    let file_hint = resolve::resolve_symbol(fixture.root(), "docs/config.go:Config")?;
    assert_eq!(file_hint.symbol.rel_path, "docs/config.go");

    let parent_hint = resolve::resolve_symbol(fixture.root(), "Cache.read")?;
    assert_eq!(parent_hint.symbol.rel_path, "src/cache.ts");
    assert_eq!(parent_hint.symbol.parent, "Cache");

    Ok(())
}

#[test]
fn resolve_falls_back_to_case_insensitive_and_prefix_matching() -> Result<()> {
    let fixture = Fixture::new()?;

    let lowercase = resolve::resolve_symbol(fixture.root(), "config")?;
    assert_eq!(lowercase.symbol.rel_path, "src/config.go");
    assert!(lowercase.fuzzy);

    let prefix = resolve::resolve_symbol(fixture.root(), "Hand")?;
    assert_eq!(prefix.symbol.name, "HandleRequest");
    assert!(prefix.fuzzy);

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
        write(root.path(), "tests/config_test.go", "package tests\n\ntype Config struct{}\n")?;
        write(
            root.path(),
            "src/cache.ts",
            "export class Cache {\n  read(): void {}\n}\n\nexport function read(): void {}\n",
        )?;
        write(
            root.path(),
            "src/handler.go",
            "package main\n\nfunc HandleRequest() {}\n",
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
