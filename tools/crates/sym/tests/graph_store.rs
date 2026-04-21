use std::time::SystemTime;

use anyhow::Result;
use sym::store::Store;
use sym::symbols::{Import, Ref, Symbol, REF_KIND_CALL, REF_KIND_USE};

#[test]
fn store_finds_references_by_name() -> Result<()> {
    let fixture = GraphFixture::new()?;

    let refs = fixture.store.find_references("Handle", 20, &[])?;

    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].rel_path, "api.go");
    assert_eq!(refs[0].line, 4);

    Ok(())
}

#[test]
fn store_finds_importers_by_symbol_and_path_transitively() -> Result<()> {
    let fixture = GraphFixture::new()?;

    let by_symbol = fixture.store.find_importers("Handle", 2, 20)?;
    assert_eq!(by_symbol.len(), 2);
    assert_eq!(by_symbol[0].rel_path, "api.go");
    assert_eq!(by_symbol[0].depth, 1);
    assert_eq!(by_symbol[1].rel_path, "main.go");
    assert_eq!(by_symbol[1].depth, 2);

    let by_path = fixture.store.find_importers_by_path("svc.go", 2, 20)?;
    assert_eq!(by_path.len(), 2);
    assert_eq!(by_path[0].rel_path, "api.go");
    assert_eq!(by_path[1].rel_path, "main.go");

    Ok(())
}

#[test]
fn store_finds_impact_transitively() -> Result<()> {
    let fixture = GraphFixture::new()?;

    let impact = fixture.store.find_impact("Handle", 3, 20)?;

    assert_eq!(impact.len(), 2);
    assert_eq!(impact[0].caller, "Serve");
    assert_eq!(impact[0].depth, 1);
    assert_eq!(impact[1].caller, "main");
    assert_eq!(impact[1].depth, 2);

    Ok(())
}

#[test]
fn store_trace_defaults_to_call_kind_and_can_widen() -> Result<()> {
    let fixture = GraphFixture::new()?;

    let default = fixture.store.find_trace("Handle", 3, 20, &[])?;
    assert!(default.iter().any(|edge| edge.callee == "DoWork"));
    assert!(!default.iter().any(|edge| edge.callee == "Widget"));

    let wide = fixture
        .store
        .find_trace("Handle", 3, 20, &[REF_KIND_CALL, REF_KIND_USE])?;
    assert!(wide.iter().any(|edge| edge.callee == "Widget"));

    Ok(())
}

struct GraphFixture {
    _temp_dir: tempfile::TempDir,
    store: Store,
}

impl GraphFixture {
    fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("graph.db");
        let store = Store::open(&db_path)?;

        seed_file(
            &store,
            "/repo/svc.go",
            "svc.go",
            vec![Symbol {
                name: "Handle".to_string(),
                kind: "function".to_string(),
                file: "/repo/svc.go".to_string(),
                start_line: 1,
                end_line: 6,
                start_col: 0,
                end_col: 20,
                parent: String::new(),
                depth: 0,
                signature: "Handle()".to_string(),
                language: "go".to_string(),
            }],
            vec![Import {
                raw_path: "repo/worker".to_string(),
                language: "go".to_string(),
            }],
            vec![
                Ref {
                    name: "DoWork".to_string(),
                    line: 3,
                    language: "go".to_string(),
                    kind: REF_KIND_CALL.to_string(),
                },
                Ref {
                    name: "Widget".to_string(),
                    line: 4,
                    language: "go".to_string(),
                    kind: REF_KIND_USE.to_string(),
                },
            ],
        )?;

        seed_file(
            &store,
            "/repo/worker.go",
            "worker.go",
            vec![
                Symbol {
                    name: "DoWork".to_string(),
                    kind: "function".to_string(),
                    file: "/repo/worker.go".to_string(),
                    start_line: 1,
                    end_line: 4,
                    start_col: 0,
                    end_col: 20,
                    parent: String::new(),
                    depth: 0,
                    signature: "DoWork()".to_string(),
                    language: "go".to_string(),
                },
                Symbol {
                    name: "Widget".to_string(),
                    kind: "struct".to_string(),
                    file: "/repo/worker.go".to_string(),
                    start_line: 6,
                    end_line: 8,
                    start_col: 0,
                    end_col: 14,
                    parent: String::new(),
                    depth: 0,
                    signature: "Widget".to_string(),
                    language: "go".to_string(),
                },
            ],
            vec![],
            vec![],
        )?;

        seed_file(
            &store,
            "/repo/api.go",
            "api.go",
            vec![Symbol {
                name: "Serve".to_string(),
                kind: "function".to_string(),
                file: "/repo/api.go".to_string(),
                start_line: 1,
                end_line: 5,
                start_col: 0,
                end_col: 18,
                parent: String::new(),
                depth: 0,
                signature: "Serve()".to_string(),
                language: "go".to_string(),
            }],
            vec![Import {
                raw_path: "repo/svc".to_string(),
                language: "go".to_string(),
            }],
            vec![Ref {
                name: "Handle".to_string(),
                line: 4,
                language: "go".to_string(),
                kind: REF_KIND_CALL.to_string(),
            }],
        )?;

        seed_file(
            &store,
            "/repo/main.go",
            "main.go",
            vec![Symbol {
                name: "main".to_string(),
                kind: "function".to_string(),
                file: "/repo/main.go".to_string(),
                start_line: 1,
                end_line: 5,
                start_col: 0,
                end_col: 16,
                parent: String::new(),
                depth: 0,
                signature: "main()".to_string(),
                language: "go".to_string(),
            }],
            vec![Import {
                raw_path: "repo/api".to_string(),
                language: "go".to_string(),
            }],
            vec![Ref {
                name: "Serve".to_string(),
                line: 3,
                language: "go".to_string(),
                kind: REF_KIND_CALL.to_string(),
            }],
        )?;

        Ok(Self {
            _temp_dir: temp_dir,
            store,
        })
    }
}

fn seed_file(
    store: &Store,
    path: &str,
    rel_path: &str,
    symbols: Vec<Symbol>,
    imports: Vec<Import>,
    refs: Vec<Ref>,
) -> Result<()> {
    let file_id = store.upsert_file(path, rel_path, "go", "hash", SystemTime::now(), 100)?;
    store.replace_file_contents(file_id, &symbols, &imports, &refs)?;
    Ok(())
}
