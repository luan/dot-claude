use std::fs;
use std::path::Path;

use anyhow::Result;
use sym::search::{normalize_search_mode, search_text};

#[test]
fn normalize_search_mode_matches_sym_rules() {
    let cases = [
        (false, false, false, Some(false), None),
        (true, false, false, Some(true), None),
        (false, true, false, Some(true), None),
        (true, true, false, Some(true), None),
        (false, true, true, None, Some("not supported with --text")),
    ];

    for (exact, ignore_case, text_mode, expected, error) in cases {
        let actual = normalize_search_mode(exact, ignore_case, text_mode);
        match (actual, expected, error) {
            (Ok(actual), Some(expected), None) => assert_eq!(actual, expected),
            (Err(err), None, Some(expected)) => {
                assert!(err.to_string().contains(expected), "{err}")
            }
            other => panic!("unexpected case result: {other:?}"),
        }
    }
}

#[test]
fn search_text_respects_language_and_path_filters() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    write(
        temp_dir.path(),
        "src/main.go",
        "package main\nfunc main() {}\n",
    )?;
    write(
        temp_dir.path(),
        "frontend/app.ts",
        "export function main() { return 'main'; }\n",
    )?;
    write(temp_dir.path(), "docs/readme.md", "main appears here too\n")?;

    let results = search_text(
        temp_dir.path(),
        "main",
        Some("go"),
        20,
        &["src/*".to_string()],
        &["*_test.go".to_string()],
        false,
    )?;

    assert_eq!(results.len(), 2);
    assert!(
        results
            .iter()
            .all(|result| result.rel_path == Path::new("src/main.go"))
    );

    Ok(())
}

#[test]
fn search_text_applies_excludes_after_widening_limit() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    for index in 0..110 {
        write(
            temp_dir.path(),
            &format!("generated/file{index}.go"),
            "package generated\nfunc target() {}\n",
        )?;
    }
    write(
        temp_dir.path(),
        "src/keep.go",
        "package main\nfunc target() {}\n",
    )?;

    let results = search_text(
        temp_dir.path(),
        "target",
        Some("go"),
        1,
        &[],
        &["generated/**".to_string()],
        false,
    )?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].rel_path, Path::new("src/keep.go"));

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
