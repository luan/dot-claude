use std::fs;
use std::path::Path;

use anyhow::Result;
use sym::impls;

#[test]
fn impls_queries_auto_index_for_incoming_and_inverse_directions() -> Result<()> {
    let fixture = Fixture::new()?;

    let incoming = impls::find_implementors(fixture.root(), "Reader", None, 50, &[], &[], false, false)?;
    assert_eq!(incoming.len(), 1);
    assert_eq!(incoming[0].implementer, "Cache");
    assert!(incoming[0].resolved);

    let inverse = impls::find_implements(fixture.root(), "Cache", None, 50, &[], &[], false, false)?;
    assert_eq!(inverse.len(), 1);
    assert_eq!(inverse[0].target, "Reader");

    Ok(())
}

#[test]
fn impls_queries_support_path_filters_and_resolved_filters() -> Result<()> {
    let fixture = Fixture::new()?;

    let filtered = impls::find_implementors(
        fixture.root(),
        "Reader",
        None,
        50,
        &["src/*".to_string()],
        &["src/cache.rs".to_string()],
        false,
        false,
    )?;
    assert!(filtered.is_empty());

    let unresolved = impls::find_implementors(
        fixture.root(),
        "ExternalReader",
        None,
        50,
        &[],
        &[],
        false,
        true,
    )?;
    assert_eq!(unresolved.len(), 1);
    assert_eq!(unresolved[0].implementer, "RemoteCache");
    assert!(!unresolved[0].resolved);

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
            "src/cache.rs",
            "trait Reader {}\n\nstruct Cache;\n\nimpl Reader for Cache {}\n\nstruct RemoteCache;\n\nimpl ExternalReader for RemoteCache {}\n",
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
