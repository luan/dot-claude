use std::time::SystemTime;

use anyhow::Result;
use sym::store::Store;
use sym::symbols::{REF_KIND_IMPLEMENTS, Ref, Symbol};

#[test]
fn store_finds_implementors_with_resolved_and_external_targets() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.insert(
        "/repo/Named.swift",
        "Named.swift",
        "swift",
        vec![symbol("Named", "protocol", "/repo/Named.swift", 1, 3)],
        vec![],
    )?;
    fixture.insert(
        "/repo/Types.swift",
        "Types.swift",
        "swift",
        vec![
            symbol("TimerIntent", "class", "/repo/Types.swift", 1, 10),
            symbol("NamedTimer", "class", "/repo/Types.swift", 12, 20),
        ],
        vec![
            implements_ref("LiveActivityIntent", 1, "swift"),
            implements_ref("Named", 12, "swift"),
        ],
    )?;

    let external = fixture.store.find_implementors("LiveActivityIntent", 50)?;
    assert_eq!(external.len(), 1);
    assert_eq!(external[0].implementer, "TimerIntent");
    assert!(!external[0].resolved);

    let local = fixture.store.find_implementors("Named", 50)?;
    assert_eq!(local.len(), 1);
    assert_eq!(local[0].implementer, "NamedTimer");
    assert!(local[0].resolved);

    Ok(())
}

#[test]
fn store_finds_inverse_implements_and_skips_nested_type_edges() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.insert(
        "/repo/Session.swift",
        "Session.swift",
        "swift",
        vec![
            symbol("Session", "class", "/repo/Session.swift", 1, 100),
            symbol("RequestConvertible", "struct", "/repo/Session.swift", 50, 60),
        ],
        vec![
            implements_ref("Sendable", 1, "swift"),
            implements_ref("URLRequestConvertible", 50, "swift"),
        ],
    )?;

    let outer = fixture.store.find_implements("Session", 50)?;
    assert_eq!(outer.len(), 1);
    assert_eq!(outer[0].target, "Sendable");
    assert_eq!(outer[0].implementer, "Session");

    let inner = fixture.store.find_implements("RequestConvertible", 50)?;
    assert_eq!(inner.len(), 1);
    assert_eq!(inner[0].target, "URLRequestConvertible");
    assert_eq!(inner[0].implementer, "RequestConvertible");

    Ok(())
}

#[test]
fn store_resolves_rust_impl_blocks_for_both_directions() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.insert(
        "/repo/frame.rs",
        "frame.rs",
        "rust",
        vec![
            symbol("Frame", "struct", "/repo/frame.rs", 1, 5),
            symbol("Frame", "impl", "/repo/frame.rs", 10, 12),
        ],
        vec![implements_ref("Error", 10, "rust")],
    )?;

    let incoming = fixture.store.find_implementors("Error", 50)?;
    assert_eq!(incoming.len(), 1);
    assert_eq!(incoming[0].implementer, "Frame");

    let outgoing = fixture.store.find_implements("Frame", 50)?;
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].target, "Error");
    assert_eq!(outgoing[0].implementer, "Frame");

    Ok(())
}

struct Fixture {
    _temp_dir: tempfile::TempDir,
    store: Store,
}

impl Fixture {
    fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let store = Store::open(&temp_dir.path().join("index.db"))?;
        Ok(Self {
            _temp_dir: temp_dir,
            store,
        })
    }

    fn insert(
        &self,
        path: &str,
        rel_path: &str,
        language: &str,
        symbols: Vec<Symbol>,
        refs: Vec<Ref>,
    ) -> Result<()> {
        let file_id = self.store.upsert_file(
            path,
            rel_path,
            language,
            "hash",
            SystemTime::UNIX_EPOCH,
            100,
        )?;
        self.store.replace_file_contents(file_id, &symbols, &[], &refs)
    }
}

fn symbol(name: &str, kind: &str, file: &str, start_line: usize, end_line: usize) -> Symbol {
    Symbol {
        name: name.to_string(),
        kind: kind.to_string(),
        file: file.to_string(),
        start_line,
        end_line,
        start_col: 0,
        end_col: 0,
        parent: String::new(),
        depth: 0,
        signature: String::new(),
        language: file.rsplit('.').next().unwrap_or_default().to_string(),
    }
}

fn implements_ref(name: &str, line: usize, language: &str) -> Ref {
    Ref {
        name: name.to_string(),
        line,
        language: language.to_string(),
        kind: REF_KIND_IMPLEMENTS.to_string(),
    }
}
