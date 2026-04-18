//! CLI contract baseline — locks the stdout/file shape of `ct <kind> create`
//! before any refactor. If a future change alters the exposed contract these
//! tests must fail loudly.

use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Set up a blueprints vault with a remote so `commit_and_push` succeeds.
/// Returns `(blueprints, bare_remote)` — both must outlive the test.
fn setup_blueprints() -> (TempDir, TempDir) {
    let bp = tempfile::tempdir().expect("create blueprints tempdir");
    let remote = tempfile::tempdir().expect("create remote tempdir");

    run_git(remote.path(), &["init", "--bare", "--initial-branch=main"]);

    run_git(bp.path(), &["init", "--initial-branch=main"]);
    run_git(bp.path(), &["config", "user.name", "ct-test"]);
    run_git(bp.path(), &["config", "user.email", "ct-test@example.com"]);
    run_git(bp.path(), &["config", "commit.gpgsign", "false"]);

    // Seed an initial commit so HEAD exists before push.
    fs::write(bp.path().join(".gitkeep"), "").expect("write .gitkeep");
    run_git(bp.path(), &["add", ".gitkeep"]);
    run_git(bp.path(), &["commit", "-m", "init"]);

    let remote_url = remote.path().to_string_lossy().to_string();
    run_git(bp.path(), &["remote", "add", "origin", &remote_url]);
    run_git(bp.path(), &["push", "-u", "origin", "main"]);

    (bp, remote)
}

fn run_git(cwd: &Path, args: &[&str]) {
    let status = StdCommand::new("git")
        .current_dir(cwd)
        .args(args)
        .env("GIT_AUTHOR_NAME", "ct-test")
        .env("GIT_AUTHOR_EMAIL", "ct-test@example.com")
        .env("GIT_COMMITTER_NAME", "ct-test")
        .env("GIT_COMMITTER_EMAIL", "ct-test@example.com")
        .status()
        .unwrap_or_else(|e| panic!("git {args:?} failed to spawn: {e}"));
    assert!(status.success(), "git {args:?} exited {status}");
}

/// Build a `ct` command wired to the test vault with deterministic git identity.
fn ct_cmd(bp: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ct").expect("locate ct binary");
    cmd.env("CT_BLUEPRINTS_DIR", bp)
        .env("GIT_AUTHOR_NAME", "ct-test")
        .env("GIT_AUTHOR_EMAIL", "ct-test@example.com")
        .env("GIT_COMMITTER_NAME", "ct-test")
        .env("GIT_COMMITTER_EMAIL", "ct-test@example.com");
    cmd
}

/// Make a throwaway project directory — project_name() uses the last path
/// component, so the tempdir name becomes the project slug in the vault.
fn project_dir() -> TempDir {
    tempfile::tempdir().expect("create project tempdir")
}

#[test]
fn spec_create_prints_absolute_path_then_newline() {
    let (bp, _remote) = setup_blueprints();
    let project = project_dir();

    let assert = ct_cmd(bp.path())
        .args([
            "spec",
            "create",
            "--topic",
            "test-contract",
            "--project",
            &project.path().to_string_lossy(),
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    assert!(
        stdout.ends_with('\n'),
        "stdout must end with newline, got {stdout:?}"
    );
    let path_str = stdout.trim_end_matches('\n');
    assert!(
        !path_str.contains('\n'),
        "stdout must be exactly one line, got {stdout:?}"
    );

    let path = Path::new(path_str);
    assert!(path.is_absolute(), "path {path_str} must be absolute");
    assert!(path.exists(), "created file {path_str} must exist");

    // Strict equality — no ANSI, no extra lines, nothing but `<path>\n`.
    let expected = format!("{path_str}\n");
    assert!(
        predicate::eq(expected).eval(&stdout),
        "stdout must exactly equal <path>\\n, got {stdout:?}"
    );

    let content = fs::read_to_string(path).expect("read created file");
    assert!(
        content.starts_with("---\ntopic: test-contract\n"),
        "file must start with frontmatter opening, got:\n{content}"
    );
}

#[test]
fn plan_create_with_source_includes_wiki_link() {
    let (bp, _remote) = setup_blueprints();
    let project = project_dir();

    let assert = ct_cmd(bp.path())
        .args([
            "plan",
            "create",
            "--topic",
            "child",
            "--source",
            "my-spec-stem",
            "--project",
            &project.path().to_string_lossy(),
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    let path_str = stdout.trim_end_matches('\n');
    let content = fs::read_to_string(path_str).expect("read created file");

    assert!(
        content.contains("source: \"[[my-spec-stem]]\""),
        "frontmatter must wiki-link the source, got:\n{content}"
    );
}

#[test]
fn create_writes_frontmatter_only_body() {
    let (bp, _remote) = setup_blueprints();
    let project = project_dir();

    let assert = ct_cmd(bp.path())
        .args([
            "spec",
            "create",
            "--topic",
            "no-body-here",
            "--project",
            &project.path().to_string_lossy(),
        ])
        // Force stdin to be a non-tty empty pipe — otherwise ct reads stdin.
        .write_stdin("")
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    let path_str = stdout.trim_end_matches('\n');
    let content = fs::read_to_string(path_str).expect("read created file");

    // Frontmatter only: file opens with `---\n`, has exactly one closing
    // `---\n` delimiter, and nothing after. (The literal spec regex
    // `^---\n[^-]*\n---\n$` rejects real frontmatter — `created: 2026-04-16`
    // and tag lines contain `-` — so we encode the underlying intent.)
    assert!(
        content.starts_with("---\n"),
        "must open with frontmatter delimiter, got:\n{content}"
    );
    assert!(
        content.ends_with("\n---\n"),
        "must end with frontmatter close and nothing after, got:\n{content}"
    );
    let close_count = content.match_indices("\n---\n").count();
    assert_eq!(
        close_count, 1,
        "expected exactly one frontmatter close, got {close_count}:\n{content}"
    );
}
