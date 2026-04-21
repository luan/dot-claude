use std::path::PathBuf;

use anyhow::Result;

#[test]
fn select_db_path_prefers_explicit_override() -> Result<()> {
    let root = tempfile::tempdir()?;
    std::fs::create_dir(root.path().join(".git"))?;
    let explicit = root.path().join("custom/index.db");
    let env = root.path().join("env/index.db");

    let selected = sym::repo::select_db_path(root.path(), Some(&explicit), Some(&env))?;
    assert_eq!(selected, explicit);
    Ok(())
}

#[test]
fn select_db_path_uses_env_before_repo_hash() -> Result<()> {
    let root = tempfile::tempdir()?;
    std::fs::create_dir(root.path().join(".git"))?;
    let env = root.path().join("env/index.db");

    let selected = sym::repo::select_db_path(root.path(), None, Some(&env))?;
    assert_eq!(selected, env);
    Ok(())
}

#[test]
fn select_db_path_falls_back_to_repo_scoped_hash() -> Result<()> {
    let root = tempfile::tempdir()?;
    std::fs::create_dir(root.path().join(".git"))?;

    let selected = sym::repo::select_db_path(root.path(), None, None)?;
    assert_eq!(selected.file_name(), Some(std::ffi::OsStr::new("index.db")));
    assert!(selected.to_string_lossy().contains("/sym/repos/"));
    assert_ne!(selected, PathBuf::from("index.db"));
    Ok(())
}
