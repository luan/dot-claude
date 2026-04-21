use std::fs;
use std::path::Path;

use anyhow::Result;
use sym::{context, investigate};

#[test]
fn context_returns_source_callers_imports_and_conformance() -> Result<()> {
    let fixture = Fixture::new()?;

    let result = context::symbol_context(fixture.root(), "Cache", 10)?;

    assert_eq!(result.symbol.name, "Cache");
    assert!(result.source.contains("export class Cache implements Reader"));
    assert_eq!(result.file_imports, vec!["./base"]);
    assert!(result.callers.iter().any(|reference| reference.rel_path == "src/main.ts"));
    assert_eq!(result.implements.len(), 1);
    assert_eq!(result.implements[0].target, "Reader");
    assert!(result.implementors.is_empty());

    Ok(())
}

#[test]
fn investigate_adapts_for_functions_and_types() -> Result<()> {
    let fixture = Fixture::new()?;

    let function = investigate::investigate(fixture.root(), "load")?;
    assert_eq!(function.kind, "function");
    assert!(function.source.contains("function load()"));
    assert!(function.refs.iter().any(|reference| reference.rel_path == "src/main.ts" && reference.line > 0));
    assert!(function.impact.iter().any(|impact| impact.caller == "main"));

    let ty = investigate::investigate(fixture.root(), "Cache")?;
    assert_eq!(ty.kind, "type");
    assert!(ty.members.iter().any(|member| member.name == "read"));
    assert!(ty.refs.iter().any(|reference| reference.rel_path == "src/main.ts"));
    assert_eq!(ty.implements.len(), 1);
    assert_eq!(ty.implements[0].target, "Reader");

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
            "src/base.ts",
            "export interface Reader {\n  read(): void;\n}\n",
        )?;
        write(
            root.path(),
            "src/cache.ts",
            "import { Reader } from \"./base\";\n\nexport class Cache implements Reader {\n  read() {}\n}\n",
        )?;
        write(
            root.path(),
            "src/main.ts",
            "import { Cache } from \"./cache\";\n\nfunction load() {\n  new Cache().read();\n}\n\nfunction main() {\n  load();\n}\n",
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
