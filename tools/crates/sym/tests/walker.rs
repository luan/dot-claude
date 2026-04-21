use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use sym::lang;
use sym::walker::{WalkOptions, walk};
use tempfile::TempDir;

#[test]
fn language_registry_matches_representative_files() {
    let cases = [
        ("index.go", Some(("go", true))),
        ("index.ts", Some(("typescript", true))),
        ("app.py", Some(("python", true))),
        ("Dockerfile", Some(("dockerfile", false))),
        ("Makefile", Some(("make", false))),
        ("notes.md", Some(("markdown", false))),
        ("unknown.xyz", None),
    ];

    for (path, expected) in cases {
        let actual = lang::language_for_file(Path::new(path));
        match (actual, expected) {
            (Some(actual), Some((id, parseable))) => {
                assert_eq!(actual.id, id, "language id mismatch for {path}");
                assert_eq!(actual.parseable, parseable, "parseable mismatch for {path}");
            }
            (None, None) => {}
            (actual, expected) => {
                panic!("unexpected language result for {path}: {actual:?} vs {expected:?}")
            }
        }
    }
}

#[test]
fn walk_skips_default_ignored_directories_and_dot_dirs() -> Result<()> {
    let fixture = Fixture::new()?;

    let files = walk(fixture.root(), &WalkOptions::default())?;
    let rel_paths = collect_rel_paths(files);

    assert!(rel_paths.contains(&PathBuf::from("generated/skip_me.go")));
    assert!(rel_paths.contains(&PathBuf::from("src/main.go")));
    assert!(rel_paths.contains(&PathBuf::from("frontend/app.ts")));
    assert!(!rel_paths.contains(&PathBuf::from("node_modules/pkg/index.js")));
    assert!(!rel_paths.contains(&PathBuf::from(".hidden/secret.go")));

    Ok(())
}

#[test]
fn walk_can_limit_to_parseable_files() -> Result<()> {
    let fixture = Fixture::new()?;

    let files = walk(
        fixture.root(),
        &WalkOptions {
            parseable_only: true,
            ignore: Vec::new(),
        },
    )?;
    let rel_paths = collect_rel_paths(files);

    assert!(rel_paths.contains(&PathBuf::from("src/main.go")));
    assert!(!rel_paths.contains(&PathBuf::from("docs/readme.md")));
    assert!(!rel_paths.contains(&PathBuf::from("Dockerfile")));

    Ok(())
}

#[test]
fn walk_respects_custom_ignore_patterns() -> Result<()> {
    let fixture = Fixture::new()?;

    let files = walk(
        fixture.root(),
        &WalkOptions {
            parseable_only: false,
            ignore: vec!["generated".to_string(), "frontend/**".to_string()],
        },
    )?;
    let rel_paths = collect_rel_paths(files);

    assert!(!rel_paths.contains(&PathBuf::from("generated/skip_me.go")));
    assert!(!rel_paths.contains(&PathBuf::from("frontend/app.ts")));
    assert!(rel_paths.contains(&PathBuf::from("src/main.go")));

    Ok(())
}

fn collect_rel_paths(files: Vec<sym::walker::FileEntry>) -> Vec<PathBuf> {
    files.into_iter().map(|entry| entry.rel_path).collect()
}

struct Fixture {
    temp_dir: TempDir,
}

impl Fixture {
    fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let fixture = Self { temp_dir };

        fixture.write("src/main.go", "package main\n")?;
        fixture.write("frontend/app.ts", "export const value = 1\n")?;
        fixture.write("generated/skip_me.go", "package generated\n")?;
        fixture.write("docs/readme.md", "# hello\n")?;
        fixture.write("Dockerfile", "FROM scratch\n")?;
        fixture.write("node_modules/pkg/index.js", "module.exports = {}\n")?;
        fixture.write(".hidden/secret.go", "package hidden\n")?;

        Ok(fixture)
    }

    fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    fn write(&self, rel_path: &str, contents: &str) -> Result<()> {
        let path = self.root().join(rel_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, contents)?;
        Ok(())
    }
}
