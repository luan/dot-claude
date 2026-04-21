use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use sha2::{Digest, Sha256};

pub fn sym_dir() -> Result<PathBuf> {
    if let Ok(cache_dir) = std::env::var("XDG_CACHE_HOME") {
        if !cache_dir.is_empty() {
            return Ok(PathBuf::from(cache_dir).join("sym"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home).join("Library/Caches/sym"));
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        return Ok(PathBuf::from(home).join(".cache/sym"));
    }

    bail!("cannot determine cache directory")
}

pub fn repo_db_path(repo_root: &Path) -> Result<PathBuf> {
    let base = sym_dir()?;
    let repo_root = repo_root.canonicalize()?;
    let hash = Sha256::digest(repo_root.to_string_lossy().as_bytes());
    Ok(base.join("repos").join(hex::encode(&hash[..8])).join("index.db"))
}

pub fn configured_db_path(cwd: &Path, explicit: Option<&Path>) -> Result<PathBuf> {
    let env_db = std::env::var_os("SYM_DB")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    select_db_path(cwd, explicit, env_db.as_deref())
}

pub fn select_db_path(cwd: &Path, explicit: Option<&Path>, env_db: Option<&Path>) -> Result<PathBuf> {
    if let Some(explicit) = explicit {
        return Ok(explicit.to_path_buf());
    }
    if let Some(env_db) = env_db {
        return Ok(env_db.to_path_buf());
    }

    let root = find_git_root(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    repo_db_path(&root)
}

pub fn find_git_root(dir: &Path) -> Result<PathBuf> {
    let mut current = dir.canonicalize()?;
    loop {
        let dot_git = current.join(".git");
        if let Ok(metadata) = std::fs::metadata(&dot_git) {
            if metadata.is_dir() || metadata.is_file() {
                return Ok(current);
            }
        }

        if !current.pop() {
            break;
        }
    }

    bail!("no git repository found from {}", dir.display())
}
