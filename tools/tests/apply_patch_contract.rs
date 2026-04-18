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

#[test]
fn malformed_envelope_rejected() {
    let sandbox = TempDir::new().unwrap();
    // Missing `*** End Patch` — parser should reject, not silently succeed.
    let patch = "\
*** Begin Patch
*** Add File: foo.txt
+hello
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
        stderr.contains("parse error")
            || stderr.contains("End Patch")
            || stderr.contains("sentinel"),
        "expected parse error for missing End Patch sentinel, got: {stderr}"
    );
    assert!(
        !sandbox.path().join("foo.txt").exists(),
        "malformed patch must not partially apply"
    );
}

#[test]
fn absolute_path_add_creates_file() {
    let sandbox = TempDir::new().unwrap();
    let abs = sandbox.path().join("by_abs.txt");
    let patch = format!(
        "*** Begin Patch\n*** Add File: {}\n+hello\n*** End Patch\n",
        abs.display()
    );

    ct().arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .write_stdin(patch)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&abs).unwrap(), "hello\n");
}

#[test]
fn delete_then_add_replaces_file_in_one_envelope() {
    let sandbox = TempDir::new().unwrap();
    write_seed(&sandbox, "config.toml", "old contents\n");

    let patch = "\
*** Begin Patch
*** Delete File: config.toml
*** Add File: config.toml
+new contents
*** End Patch
";

    ct().arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .write_stdin(patch)
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(sandbox.path().join("config.toml")).unwrap(),
        "new contents\n"
    );
}

#[test]
fn ambiguous_context_rejected() {
    let sandbox = TempDir::new().unwrap();
    write_seed(&sandbox, "dup.rs", "foo\nbar\nfoo\nbar\n");

    let patch = "\
*** Begin Patch
*** Update File: dup.rs
@@
-foo
+FOO
 bar
*** End Patch
";

    ct().arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .write_stdin(patch)
        .assert()
        .failure()
        .stderr(predicate::str::contains("ambiguous context"));

    let after = fs::read_to_string(sandbox.path().join("dup.rs")).unwrap();
    assert_eq!(after, "foo\nbar\nfoo\nbar\n");
}

#[test]
fn pure_rename_without_hunks_moves_file() {
    let sandbox = TempDir::new().unwrap();
    write_seed(&sandbox, "src/old_name.rs", "fn main() {}\n");

    let patch = "\
*** Begin Patch
*** Update File: src/old_name.rs
*** Move to: src/new_name.rs
*** End Patch
";

    ct().arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .write_stdin(patch)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "R src/old_name.rs \u{2192} src/new_name.rs",
        ));

    assert!(!sandbox.path().join("src/old_name.rs").exists());
    assert_eq!(
        fs::read_to_string(sandbox.path().join("src/new_name.rs")).unwrap(),
        "fn main() {}\n"
    );
}

#[test]
fn context_mismatch_reports_near_miss_snippet() {
    let sandbox = TempDir::new().unwrap();
    write_seed(
        &sandbox,
        "src/lib.rs",
        "fn main() {\n    println!(\"hello\");\n}\n",
    );

    let patch = "\
*** Begin Patch
*** Update File: src/lib.rs
@@
-    println!(\"nope\");
+    println!(\"changed\");
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
        stderr.contains("context not found"),
        "expected context_not_found, got: {stderr}"
    );
    assert!(
        stderr.contains("file state") && stderr.contains("fn main()"),
        "expected near-miss snippet with file lines, got: {stderr}"
    );
    assert!(
        stderr.contains("closest match"),
        "expected closest-match hint from edit-distance scan, got: {stderr}"
    );
}

#[test]
fn anchor_shadow_rejected_with_specific_error() {
    let sandbox = TempDir::new().unwrap();
    write_seed(&sandbox, "m.rs", "fn greet() {\n    println!(\"hi\");\n}\n");
    let patch = "\
*** Begin Patch
*** Update File: m.rs
@@ fn greet() {
 fn greet() {
-    println!(\"hi\");
+    println!(\"hello\");
 }
*** End Patch
";

    ct().arg("tool")
        .arg("apply-patch")
        .arg("--cwd")
        .arg(sandbox.path())
        .write_stdin(patch)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("also appears as the first context line")
                .and(predicate::str::contains("anchor is consumed")),
        );
}
