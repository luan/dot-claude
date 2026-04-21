use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use rusqlite::Connection;
use sym::indexer::{self, IndexOptions};
use sym::repo;
use sym::store::Store;

#[test]
fn index_merges_symignore_and_cli_patterns() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    write(
        temp_dir.path(),
        ".symignore",
        "generated\n# comment\n\nfrontend/**\n",
    )?;
    write(
        temp_dir.path(),
        "generated/skip_me.go",
        "package generated\n",
    )?;
    write(
        temp_dir.path(),
        "frontend/app.ts",
        "export const value = 1\n",
    )?;
    write(temp_dir.path(), "src/main.go", "package main\n")?;
    write(temp_dir.path(), "docs/readme.md", "# docs\n")?;

    let stats = indexer::index(
        temp_dir.path(),
        &IndexOptions {
            cli_ignore_patterns: vec!["docs/**".to_string()],
            ..IndexOptions::default()
        },
    )?;

    assert_eq!(
        stats.ignore_patterns,
        vec!["generated", "frontend/**", "docs/**"]
    );

    Ok(())
}

#[test]
fn index_respects_file_and_cli_ignore_patterns() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    write(temp_dir.path(), ".symignore", "generated\n")?;
    write(
        temp_dir.path(),
        "generated/skip_me.go",
        "package generated\n",
    )?;
    write(
        temp_dir.path(),
        "frontend/app.ts",
        "export const value = 1\n",
    )?;
    write(temp_dir.path(), "src/main.go", "package main\n")?;

    let stats = indexer::index(
        temp_dir.path(),
        &IndexOptions {
            cli_ignore_patterns: vec!["frontend/**".to_string()],
            ..IndexOptions::default()
        },
    )?;

    assert_eq!(stats.files_indexed, 1);

    Ok(())
}

#[test]
fn index_persists_symbols_and_supports_incremental_skips() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("index.db");
    write(
        temp_dir.path(),
        "main.go",
        "package main\n\nfunc HandleRequest() {}\n",
    )?;
    write(
        temp_dir.path(),
        "calc.py",
        "class Calculator:\n    pass\n",
    )?;

    let first = indexer::index(
        temp_dir.path(),
        &IndexOptions {
            db_path: Some(db_path.clone()),
            ..IndexOptions::default()
        },
    )?;

    assert_eq!(first.files_indexed, 2);
    assert_eq!(first.files_skipped, 0);
    assert!(first.symbols_found >= 2);

    let store = Store::open(&db_path)?;
    let handle = store.search_symbols("Handle", "", "", false, false, 50)?;
    assert!(handle.iter().any(|symbol| symbol.name == "HandleRequest"));
    let calc = store.search_symbols("Calculator", "", "", true, false, 50)?;
    assert_eq!(calc.len(), 1);

    let second = indexer::index(
        temp_dir.path(),
        &IndexOptions {
            db_path: Some(db_path.clone()),
            ..IndexOptions::default()
        },
    )?;
    assert_eq!(second.files_indexed, 0);
    assert_eq!(second.files_skipped, 2);

    std::thread::sleep(Duration::from_millis(5));
    write(
        temp_dir.path(),
        "main.go",
        "package main\n\nfunc HandleRequest() {}\nfunc Changed() {}\n",
    )?;

    let third = indexer::index(
        temp_dir.path(),
        &IndexOptions {
            db_path: Some(db_path.clone()),
            ..IndexOptions::default()
        },
    )?;
    assert_eq!(third.files_indexed, 1);
    assert_eq!(third.files_skipped, 1);

    let changed = store.search_symbols("Changed", "", "", true, false, 50)?;
    assert_eq!(changed.len(), 1);

    Ok(())
}

#[test]
fn index_force_reindexes_and_prunes_stale_files() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("index.db");
    write(
        temp_dir.path(),
        "main.go",
        "package main\n\ntype Server struct{}\n",
    )?;
    write(
        temp_dir.path(),
        "calc.py",
        "class Calculator:\n    pass\n",
    )?;

    let initial = indexer::index(
        temp_dir.path(),
        &IndexOptions {
            db_path: Some(db_path.clone()),
            ..IndexOptions::default()
        },
    )?;
    assert_eq!(initial.files_indexed, 2);

    let forced = indexer::index(
        temp_dir.path(),
        &IndexOptions {
            db_path: Some(db_path.clone()),
            force: true,
            ..IndexOptions::default()
        },
    )?;
    assert_eq!(forced.files_indexed, 2);
    assert_eq!(forced.files_skipped, 0);

    fs::remove_file(temp_dir.path().join("calc.py"))?;

    let pruned = indexer::index(
        temp_dir.path(),
        &IndexOptions {
            db_path: Some(db_path.clone()),
            ..IndexOptions::default()
        },
    )?;
    assert_eq!(pruned.stale_removed, 1);

    let store = Store::open(&db_path)?;
    let calc = store.search_symbols("Calculator", "", "", true, false, 50)?;
    assert!(calc.is_empty());

    Ok(())
}

#[test]
fn index_uses_repo_db_path_when_not_explicitly_provided() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    write(
        temp_dir.path(),
        "main.go",
        "package main\n\nfunc HandleRequest() {}\n",
    )?;

    let stats = indexer::index(temp_dir.path(), &IndexOptions::default())?;
    assert_eq!(stats.files_indexed, 1);

    let db_path = repo::repo_db_path(temp_dir.path())?;
    assert!(db_path.exists());

    Ok(())
}

#[test]
fn index_supports_legacy_files_table_with_indexed_at() -> Result<()> {
    let repo_dir = tempfile::tempdir()?;
    let db_dir = tempfile::tempdir()?;
    let db_path = db_dir.path().join("index.db");
    write(
        repo_dir.path(),
        "main.go",
        "package main\n\nfunc HandleRequest() {}\n",
    )?;

    let conn = Connection::open(&db_path)?;
    conn.execute_batch(
        r#"
        CREATE TABLE meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT UNIQUE NOT NULL,
            rel_path TEXT NOT NULL,
            language TEXT NOT NULL,
            hash TEXT NOT NULL,
            indexed_at DATETIME NOT NULL,
            mtime DATETIME,
            mtime_ns INTEGER,
            size INTEGER
        );

        CREATE TABLE symbols (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            start_line INTEGER NOT NULL,
            end_line INTEGER NOT NULL,
            start_col INTEGER NOT NULL,
            end_col INTEGER NOT NULL,
            parent TEXT NOT NULL,
            depth INTEGER NOT NULL,
            signature TEXT NOT NULL,
            language TEXT NOT NULL
        );

        CREATE TABLE imports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
            raw_path TEXT NOT NULL,
            language TEXT NOT NULL
        );

        CREATE TABLE refs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
            line INTEGER NOT NULL,
            name TEXT NOT NULL,
            language TEXT NOT NULL,
            kind TEXT NOT NULL
        );
        "#,
    )?;
    drop(conn);

    let stats = indexer::index(
        repo_dir.path(),
        &IndexOptions {
            db_path: Some(db_path.clone()),
            ..IndexOptions::default()
        },
    )?;
    assert_eq!(stats.files_indexed, 1);

    let conn = Connection::open(&db_path)?;
    let indexed_at: String = conn.query_row(
        "SELECT indexed_at FROM files WHERE rel_path = 'main.go'",
        [],
        |row| row.get(0),
    )?;
    assert!(!indexed_at.is_empty());

    Ok(())
}

fn write(root: &Path, rel_path: &str, contents: &str) -> Result<()> {
    let path = root.join(rel_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}
