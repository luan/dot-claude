use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use sym::indexer;
use sym::resolve;
use sym::store::Store;

#[test]
fn ensure_fresh_auto_indexes_skips_clean_repos_and_refreshes_changes() -> Result<()> {
    let fixture = Fixture::new()?;
    let db_path = fixture.db_path();

    let refreshed = indexer::ensure_fresh(fixture.root(), &db_path)?;
    assert_eq!(refreshed, 1);

    let store = Store::open(&db_path)?;
    let first_last_index = store.get_meta("last_index_ns")?.unwrap();
    assert_eq!(store.all_files(None)?.len(), 1);
    drop(store);

    let refreshed = indexer::ensure_fresh(fixture.root(), &db_path)?;
    assert_eq!(refreshed, 0);

    let store = Store::open(&db_path)?;
    let second_last_index = store.get_meta("last_index_ns")?.unwrap();
    assert_eq!(first_last_index, second_last_index);
    drop(store);

    thread::sleep(Duration::from_millis(5));
    write(
        fixture.root(),
        "src/worker.go",
        "package main\n\nfunc Worker() {}\n",
    )?;

    let refreshed = indexer::ensure_fresh(fixture.root(), &db_path)?;
    assert_eq!(refreshed, 1);

    let store = Store::open(&db_path)?;
    assert_eq!(store.all_files(None)?.len(), 2);
    drop(store);

    thread::sleep(Duration::from_millis(5));
    fs::remove_file(fixture.root().join("src/worker.go"))?;

    let refreshed = indexer::ensure_fresh(fixture.root(), &db_path)?;
    assert_eq!(refreshed, 1);

    let store = Store::open(&db_path)?;
    assert_eq!(store.all_files(None)?.len(), 1);

    Ok(())
}

#[test]
fn shared_open_store_refreshes_before_queries() -> Result<()> {
    let fixture = Fixture::new()?;
    let db_path = fixture.db_path();

    indexer::ensure_fresh(fixture.root(), &db_path)?;

    // Query-time opening should reuse the shared freshness path and see new files.
    unsafe {
        std::env::set_var("SYM_DB", &db_path);
    }
    thread::sleep(Duration::from_millis(5));
    write(
        fixture.root(),
        "src/worker.go",
        "package main\n\nfunc Worker() {}\n",
    )?;

    let store = resolve::open_store(fixture.root())?;
    let names = store
        .search_symbols("Worker", "", "", true, false, 10)?
        .into_iter()
        .map(|symbol| symbol.name)
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["Worker"]);

    unsafe {
        std::env::remove_var("SYM_DB");
    }
    Ok(())
}

struct Fixture {
    root: tempfile::TempDir,
    db_dir: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Result<Self> {
        let root = tempfile::tempdir()?;
        let db_dir = tempfile::tempdir()?;
        fs::create_dir(root.path().join(".git"))?;
        write(root.path(), "src/main.go", "package main\n\nfunc Handle() {}\n")?;
        Ok(Self { root, db_dir })
    }

    fn root(&self) -> &Path {
        self.root.path()
    }

    fn db_path(&self) -> std::path::PathBuf {
        self.db_dir.path().join("index.db")
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
