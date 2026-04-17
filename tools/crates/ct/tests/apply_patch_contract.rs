//! End-to-end contract tests for `ct tool apply-patch`.
//!
//! Each test provisions a throwaway sandbox via `tempfile::TempDir`, shells
//! `ct tool apply-patch --cwd <sandbox>` with a patch on stdin, and asserts
//! exit code, stdout, stderr, and the final filesystem state.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn write_seed(dir: &TempDir, rel: &str, content: &str) {
    let path = dir.path().join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn ct() -> Command {
    Command::cargo_bin("ct").unwrap()
}

#[test]
fn add_creates_file() {
    let sandbox = TempDir::new().unwrap();
    let patch = "\
*** Begin Patch
*** Add File: new.txt
+line1
+line2
*** End Patch
";

    ct().arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .write_stdin(patch)
        .assert()
        .success()
        .stdout(predicate::str::contains("A new.txt"));

    let written = fs::read_to_string(sandbox.path().join("new.txt")).unwrap();
    assert_eq!(written, "line1\nline2\n");
}

#[test]
fn update_applies_hunk() {
    let sandbox = TempDir::new().unwrap();
    write_seed(
        &sandbox,
        "src/foo.rs",
        "fn one() {}\nfn two() {}\nfn three() {}\n",
    );

    let patch = "\
*** Begin Patch
*** Update File: src/foo.rs
@@
 fn one() {}
-fn two() {}
+fn TWO() {}
 fn three() {}
*** End Patch
";

    ct().arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .write_stdin(patch)
        .assert()
        .success()
        .stdout(predicate::str::contains("M src/foo.rs"));

    let written = fs::read_to_string(sandbox.path().join("src/foo.rs")).unwrap();
    assert_eq!(written, "fn one() {}\nfn TWO() {}\nfn three() {}\n");
}

#[test]
fn delete_removes_file() {
    let sandbox = TempDir::new().unwrap();
    write_seed(&sandbox, "old.txt", "goodbye\n");

    let patch = "\
*** Begin Patch
*** Delete File: old.txt
*** End Patch
";

    ct().arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .write_stdin(patch)
        .assert()
        .success()
        .stdout(predicate::str::contains("D old.txt"));

    assert!(!sandbox.path().join("old.txt").exists());
}

#[test]
fn move_renames_file() {
    let sandbox = TempDir::new().unwrap();
    write_seed(&sandbox, "a.txt", "alpha\nbeta\ngamma\n");

    let patch = "\
*** Begin Patch
*** Update File: a.txt
*** Move to: b.txt
@@
-alpha
+ALPHA
 beta
 gamma
*** End Patch
";

    ct().arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .write_stdin(patch)
        .assert()
        .success()
        .stdout(predicate::str::contains("R a.txt \u{2192} b.txt"));

    assert!(!sandbox.path().join("a.txt").exists());
    let written = fs::read_to_string(sandbox.path().join("b.txt")).unwrap();
    assert_eq!(written, "ALPHA\nbeta\ngamma\n");
}

#[test]
fn dry_run_writes_nothing() {
    let sandbox = TempDir::new().unwrap();
    write_seed(&sandbox, "src/foo.rs", "original\n");

    let patch = "\
*** Begin Patch
*** Update File: src/foo.rs
@@
-original
+modified
*** End Patch
";

    ct().arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .arg("--dry-run")
        .write_stdin(patch)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("---")
                .and(predicate::str::contains("+++"))
                .and(predicate::str::contains("+modified"))
                .and(predicate::str::contains("-original")),
        )
        .stderr(predicate::str::is_empty());

    let after = fs::read_to_string(sandbox.path().join("src/foo.rs")).unwrap();
    assert_eq!(after, "original\n");
}

#[test]
fn add_existing_file_rejected() {
    let sandbox = TempDir::new().unwrap();
    write_seed(&sandbox, "existing.txt", "old");

    let patch = "\
*** Begin Patch
*** Add File: existing.txt
+new
*** End Patch
";

    ct().arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .write_stdin(patch)
        .assert()
        .failure()
        .stderr(predicate::str::contains("add target already exists"));

    let after = fs::read_to_string(sandbox.path().join("existing.txt")).unwrap();
    assert_eq!(after, "old");
}

#[test]
fn duplicate_update_rejected() {
    let sandbox = TempDir::new().unwrap();
    write_seed(&sandbox, "foo.rs", "a\nb\nc\n");

    let patch = "\
*** Begin Patch
*** Update File: foo.rs
@@
 a
-b
+B
 c
*** Update File: foo.rs
@@
 a
 B
-c
+C
*** End Patch
";

    ct().arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .write_stdin(patch)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "multiple updates target the same path",
        ));

    let after = fs::read_to_string(sandbox.path().join("foo.rs")).unwrap();
    assert_eq!(after, "a\nb\nc\n");
}

#[test]
fn move_to_existing_rejected() {
    let sandbox = TempDir::new().unwrap();
    write_seed(&sandbox, "a.txt", "hi");
    write_seed(&sandbox, "b.txt", "bye");

    let patch = "\
*** Begin Patch
*** Update File: a.txt
*** Move to: b.txt
@@
-hi
+HI
*** End Patch
";

    ct().arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .write_stdin(patch)
        .assert()
        .failure()
        .stderr(predicate::str::contains("move target already exists"));

    assert_eq!(
        fs::read_to_string(sandbox.path().join("a.txt")).unwrap(),
        "hi"
    );
    assert_eq!(
        fs::read_to_string(sandbox.path().join("b.txt")).unwrap(),
        "bye"
    );
}

#[cfg(unix)]
#[test]
fn symlink_escape_rejected() {
    let sandbox = TempDir::new().unwrap();
    let external = sandbox.path().parent().unwrap().join("escape_target.txt");
    fs::write(&external, "external\n").unwrap();

    std::os::unix::fs::symlink(&external, sandbox.path().join("link")).unwrap();

    let patch = "\
*** Begin Patch
*** Update File: link
@@
-anything
+injected
*** End Patch
";

    let result = ct()
        .arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .write_stdin(patch)
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&result.get_output().stderr).to_string();
    assert!(
        stderr.contains("path escapes cwd") || stderr.contains("escape"),
        "expected containment failure, got: {stderr}",
    );

    assert_eq!(fs::read_to_string(&external).unwrap(), "external\n");
    fs::remove_file(&external).ok();
}

#[test]
fn traversal_move_rejected() {
    let sandbox = TempDir::new().unwrap();
    write_seed(&sandbox, "foo.txt", "hi\n");

    let patch = "\
*** Begin Patch
*** Update File: foo.txt
*** Move to: ../escape.txt
@@
-hi
+bye
*** End Patch
";

    ct().arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .write_stdin(patch)
        .assert()
        .failure()
        .stderr(predicate::str::contains("path escapes cwd"));

    let after = fs::read_to_string(sandbox.path().join("foo.txt")).unwrap();
    assert_eq!(after, "hi\n");

    let escape_path = sandbox.path().parent().unwrap().join("escape.txt");
    assert!(
        !escape_path.exists(),
        "traversal must not create file outside sandbox: {escape_path:?}"
    );
}
