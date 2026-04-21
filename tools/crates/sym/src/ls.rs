use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::repo;
use crate::store::{RepoStats, Store};
use crate::walker::{self, TreeNode};

pub fn tree(path: &Path, max_depth: usize) -> Result<TreeNode> {
    walker::build_tree(path, max_depth)
}

pub fn repo_stats(cwd: &Path) -> Result<RepoStats> {
    let db_path = repo::configured_db_path(cwd, None)?;
    crate::indexer::ensure_fresh(cwd, &db_path)?;
    let store = Store::open(&db_path)?;
    store.repo_stats()
}

pub fn list_repos() -> Result<Vec<RepoStats>> {
    let repos_dir = repo::sym_dir()?.join("repos");
    if !repos_dir.exists() {
        return Ok(Vec::new());
    }

    let mut repos = Vec::new();
    for entry in fs::read_dir(&repos_dir).with_context(|| format!("reading {}", repos_dir.display()))? {
        let entry = entry?;
        let db_path = entry.path().join("index.db");
        if !db_path.is_file() {
            continue;
        }
        let store = Store::open(&db_path)?;
        let stats = store.repo_stats()?;
        if !stats.path.is_empty() {
            repos.push(stats);
        }
    }
    repos.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(repos)
}
