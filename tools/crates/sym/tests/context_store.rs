use std::time::SystemTime;

use anyhow::Result;
use sym::store::Store;
use sym::symbols::{Import, ParseResult, Symbol};

#[test]
fn store_child_symbols_are_file_scoped_and_file_imports_are_sorted() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let db_path = temp.path().join("sym.db");
    let store = Store::open(&db_path)?;

    let file_a = store.upsert_file(
        "/repo/cache.ts",
        "cache.ts",
        "typescript",
        "hash-a",
        SystemTime::UNIX_EPOCH,
        10,
    )?;
    let file_b = store.upsert_file(
        "/repo/other.ts",
        "other.ts",
        "typescript",
        "hash-b",
        SystemTime::UNIX_EPOCH,
        10,
    )?;

    store.replace_file_contents(
        file_a,
        &[
            symbol("Cache", "class", "/repo/cache.ts", 1, 10, "", 0),
            symbol("read", "method", "/repo/cache.ts", 2, 3, "Cache", 1),
            symbol("write", "method", "/repo/cache.ts", 5, 6, "Cache", 1),
        ],
        &[
            Import {
                raw_path: "./base".into(),
                language: "typescript".into(),
            },
            Import {
                raw_path: "./helpers".into(),
                language: "typescript".into(),
            },
        ],
        &ParseResult::default().refs,
    )?;
    store.replace_file_contents(
        file_b,
        &[
            symbol("Cache", "class", "/repo/other.ts", 1, 6, "", 0),
            symbol("flush", "method", "/repo/other.ts", 2, 3, "Cache", 1),
        ],
        &[],
        &[],
    )?;

    let all = store.child_symbols("Cache", 50, None)?;
    assert_eq!(all.len(), 3);

    let scoped = store.child_symbols("Cache", 50, Some("/repo/cache.ts"))?;
    assert_eq!(scoped.iter().map(|symbol| symbol.name.as_str()).collect::<Vec<_>>(), vec!["read", "write"]);

    let imports = store.file_imports("/repo/cache.ts")?;
    assert_eq!(imports, vec!["./base", "./helpers"]);

    Ok(())
}

fn symbol(name: &str, kind: &str, file: &str, start_line: usize, end_line: usize, parent: &str, depth: usize) -> Symbol {
    Symbol {
        name: name.into(),
        kind: kind.into(),
        file: file.into(),
        start_line,
        end_line,
        start_col: 0,
        end_col: 0,
        parent: parent.into(),
        depth,
        signature: String::new(),
        language: "typescript".into(),
    }
}
