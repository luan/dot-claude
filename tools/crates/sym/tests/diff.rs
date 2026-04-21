use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use sym::diff;

#[test]
fn filter_diff_hunks_keeps_only_overlapping_ranges() {
    let diff_output = concat!(
        "diff --git a/main.go b/main.go\n",
        "index 1111111..2222222 100644\n",
        "--- a/main.go\n",
        "+++ b/main.go\n",
        "@@ -1,3 +1,4 @@\n",
        " func HandleRequest() {\n",
        "+\tfmt.Println(\"handle\")\n",
        " }\n",
        "@@ -20,3 +21,4 @@\n",
        " func helper() {\n",
        "+\tfmt.Println(\"helper\")\n",
        " }\n",
    );

    let filtered = diff::filter_diff_hunks(diff_output, 1, 4);

    assert!(filtered.contains("HandleRequest"));
    assert!(!filtered.contains("helper"));
}

#[test]
fn symbol_diff_filters_git_diff_to_symbol_range_and_supports_stat() -> Result<()> {
    let fixture = DiffFixture::new()?;

    let scoped = diff::symbol_diff(fixture.root(), "HandleRequest", "HEAD", false)?;
    assert!(scoped.content.contains("HandleRequest"));
    assert!(scoped.content.contains("updated handle"));
    assert!(!scoped.content.contains("helper changed"));

    let stat = diff::symbol_diff(fixture.root(), "HandleRequest", "HEAD", true)?;
    assert!(stat.content.contains("main.go"));

    Ok(())
}

struct DiffFixture {
    root: tempfile::TempDir,
}

impl DiffFixture {
    fn new() -> Result<Self> {
        let root = tempfile::tempdir()?;
        git(root.path(), &["init", "--initial-branch=main"])?;
        git(root.path(), &["config", "user.name", "Sym Tests"])?;
        git(root.path(), &["config", "user.email", "sym@example.com"])?;

        write(root.path(), "main.go", &initial_source())?;
        git(root.path(), &["add", "main.go"])?;
        git(root.path(), &["commit", "-m", "initial"])?;

        write(root.path(), "main.go", &modified_source())?;

        Ok(Self { root })
    }

    fn root(&self) -> &Path {
        self.root.path()
    }
}

fn git(root: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(root)
        .status()
        .with_context(|| format!("running git {:?}", args))?;
    if !status.success() {
        anyhow::bail!("git {:?} failed", args);
    }
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

fn initial_source() -> String {
    [
        "package main",
        "",
        "import \"fmt\"",
        "",
        "func HandleRequest() {",
        "\tfmt.Println(\"handle\")",
        "}",
        "",
        "func spacer1() {}",
        "func spacer2() {}",
        "func spacer3() {}",
        "func spacer4() {}",
        "func spacer5() {}",
        "func spacer6() {}",
        "func spacer7() {}",
        "",
        "func helper() {",
        "\tfmt.Println(\"helper\")",
        "}",
        "",
    ]
    .join("\n")
}

fn modified_source() -> String {
    [
        "package main",
        "",
        "import \"fmt\"",
        "",
        "func HandleRequest() {",
        "\tfmt.Println(\"updated handle\")",
        "}",
        "",
        "func spacer1() {}",
        "func spacer2() {}",
        "func spacer3() {}",
        "func spacer4() {}",
        "func spacer5() {}",
        "func spacer6() {}",
        "func spacer7() {}",
        "",
        "func helper() {",
        "\tfmt.Println(\"helper changed\")",
        "}",
        "",
    ]
    .join("\n")
}
