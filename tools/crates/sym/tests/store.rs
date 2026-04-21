use std::time::SystemTime;

use anyhow::Result;
use sym::store::Store;
use sym::symbols::{Import, Ref, Symbol, REF_KIND_CALL};

#[test]
fn store_supports_prefix_exact_and_case_insensitive_search() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("store.db");
    let store = Store::open(&db_path)?;

    let file_id = store.upsert_file(
        "/repo/main.go",
        "main.go",
        "go",
        "hash1",
        SystemTime::now(),
        100,
    )?;
    store.replace_file_contents(
        file_id,
        &[Symbol {
            name: "HandleRequest".to_string(),
            kind: "function".to_string(),
            file: "/repo/main.go".to_string(),
            start_line: 3,
            end_line: 3,
            start_col: 0,
            end_col: 22,
            parent: String::new(),
            depth: 0,
            signature: "HandleRequest()".to_string(),
            language: "go".to_string(),
        }],
        &[Import {
            raw_path: "fmt".to_string(),
            language: "go".to_string(),
        }],
        &[Ref {
            name: "Println".to_string(),
            line: 4,
            language: "go".to_string(),
            kind: REF_KIND_CALL.to_string(),
        }],
    )?;

    let prefix = store.search_symbols("Handle", "", "", false, false, 50)?;
    assert!(prefix.iter().any(|symbol| symbol.name == "HandleRequest"));

    let exact = store.search_symbols("HandleRequest", "function", "go", true, false, 50)?;
    assert_eq!(exact.len(), 1);

    let ignore_case = store.search_symbols("handlerequest", "", "", true, true, 50)?;
    assert_eq!(ignore_case.len(), 1);

    let files = store.all_files(None)?;
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].rel_path, "main.go");

    Ok(())
}

#[test]
fn store_deletes_stale_paths() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("store.db");
    let store = Store::open(&db_path)?;

    let first = store.upsert_file(
        "/repo/main.go",
        "main.go",
        "go",
        "hash1",
        SystemTime::now(),
        10,
    )?;
    store.replace_file_contents(first, &[], &[], &[])?;
    let second = store.upsert_file(
        "/repo/app.py",
        "app.py",
        "python",
        "hash2",
        SystemTime::now(),
        10,
    )?;
    store.replace_file_contents(second, &[], &[], &[])?;

    let deleted = store.delete_stale_paths(&["/repo/main.go".to_string()])?;
    assert_eq!(deleted, 1);

    let paths = store.all_stored_paths()?;
    assert_eq!(paths, vec!["/repo/main.go".to_string()]);

    Ok(())
}
