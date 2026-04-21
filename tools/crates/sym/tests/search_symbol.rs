use std::fs;
use std::path::Path;

use anyhow::Result;
use sym::search;

#[test]
fn search_symbols_auto_indexes_repo_and_honors_filters() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    fs::create_dir(temp_dir.path().join(".git"))?;
    write(
        temp_dir.path(),
        "src/main.go",
        "package main\n\nfunc HandleRequest() {}\n",
    )?;
    write(
        temp_dir.path(),
        "src/server.go",
        "package main\n\nfunc HandleServer() {}\n",
    )?;
    write(
        temp_dir.path(),
        "src/main_test.go",
        "package main\n\nfunc HandleRequestTest() {}\n",
    )?;

    let results = search::search_symbols(
        temp_dir.path(),
        "Handle",
        None,
        None,
        20,
        false,
        false,
        &["src/*".to_string()],
        &["*_test.go".to_string()],
    )?;

    assert!(results.iter().any(|result| result.name == "HandleRequest"));
    assert!(results.iter().any(|result| result.name == "HandleServer"));
    assert!(!results.iter().any(|result| result.name == "HandleRequestTest"));

    Ok(())
}

#[test]
fn search_filters_by_kind_and_applies_ranking() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    fs::create_dir(temp_dir.path().join(".git"))?;
    // A Widget function in tests/ should be ranked below a Widget struct in src/,
    // because symbol_score prefers struct kind and penalizes /tests/ paths.
    write(
        temp_dir.path(),
        "src/widget.go",
        "package widget\n\ntype Widget struct {}\n",
    )?;
    write(
        temp_dir.path(),
        "tests/widget_helper.go",
        "package tests\n\nfunc Widget() {}\n",
    )?;

    let all = search::search_symbols(
        temp_dir.path(),
        "Widget",
        None,
        None,
        20,
        false,
        false,
        &[],
        &[],
    )?;
    assert_eq!(
        all.first().map(|r| r.kind.as_str()),
        Some("struct"),
        "struct in src/ should rank above function in tests/"
    );

    let only_structs = search::search_symbols(
        temp_dir.path(),
        "Widget",
        Some("struct"),
        None,
        20,
        false,
        false,
        &[],
        &[],
    )?;
    assert!(only_structs.iter().all(|r| r.kind == "struct"));
    assert!(only_structs.iter().any(|r| r.name == "Widget"));

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
