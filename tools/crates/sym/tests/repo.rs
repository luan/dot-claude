use std::fs;

use anyhow::Result;
use sym::repo;

#[test]
fn repo_db_path_is_stable_and_distinct() -> Result<()> {
    let first = tempfile::tempdir()?;
    let second = tempfile::tempdir()?;

    let first_a = repo::repo_db_path(first.path())?;
    let first_b = repo::repo_db_path(first.path())?;
    let second_path = repo::repo_db_path(second.path())?;

    assert_eq!(first_a, first_b);
    assert_ne!(first_a, second_path);
    assert!(first_a.ends_with("index.db"));

    Ok(())
}

#[test]
fn find_git_root_supports_git_directory_and_git_file() -> Result<()> {
    let repo_dir = tempfile::tempdir()?;
    fs::create_dir(repo_dir.path().join(".git"))?;
    fs::create_dir_all(repo_dir.path().join("nested/child"))?;

    let root = repo::find_git_root(&repo_dir.path().join("nested/child"))?;
    assert_eq!(root, repo_dir.path().canonicalize()?);

    let worktree = tempfile::tempdir()?;
    fs::write(worktree.path().join(".git"), "gitdir: /tmp/example\n")?;
    fs::create_dir_all(worktree.path().join("subdir"))?;

    let root = repo::find_git_root(&worktree.path().join("subdir"))?;
    assert_eq!(root, worktree.path().canonicalize()?);

    Ok(())
}
