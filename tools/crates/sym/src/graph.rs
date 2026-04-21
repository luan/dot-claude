use std::path::Path;

use anyhow::Result;

use crate::pathfilters::{include_path, widen_path_filter_limit};
use crate::resolve;
use crate::store::{ImpactResult, ImporterResult, RefResult, Store, TraceResult};

pub fn find_references(
    cwd: &Path,
    name: &str,
    limit: usize,
    includes: &[String],
    excludes: &[String],
) -> Result<Vec<RefResult>> {
    let fetch_limit = widen_path_filter_limit(limit, !includes.is_empty() || !excludes.is_empty());
    let store = open_store(cwd)?;
    let mut results = store.find_references(name, fetch_limit.max(1), &[])?;
    results.retain(|result| include_path(Path::new(&result.rel_path), includes, excludes));
    if limit > 0 && results.len() > limit {
        results.truncate(limit);
    }
    Ok(results)
}

pub fn find_importers(
    cwd: &Path,
    name: &str,
    depth: usize,
    limit: usize,
    includes: &[String],
    excludes: &[String],
) -> Result<Vec<ImporterResult>> {
    let fetch_limit = widen_path_filter_limit(limit, !includes.is_empty() || !excludes.is_empty());
    let store = open_store(cwd)?;
    let mut results = store.find_importers(name, depth, fetch_limit.max(1))?;
    results.retain(|result| include_path(Path::new(&result.rel_path), includes, excludes));
    if limit > 0 && results.len() > limit {
        results.truncate(limit);
    }
    Ok(results)
}

pub fn find_importers_by_path(cwd: &Path, target: &str, depth: usize, limit: usize) -> Result<Vec<ImporterResult>> {
    let store = open_store(cwd)?;
    store.find_importers_by_path(target, depth, limit.max(1))
}

pub fn find_impact(cwd: &Path, name: &str, depth: usize, limit: usize) -> Result<Vec<ImpactResult>> {
    let store = open_store(cwd)?;
    store.find_impact(name, depth, limit.max(1))
}

pub fn find_trace(
    cwd: &Path,
    name: &str,
    depth: usize,
    limit: usize,
    kinds: &[&str],
) -> Result<Vec<TraceResult>> {
    let store = open_store(cwd)?;
    store.find_trace(name, depth, limit.max(1), kinds)
}

fn open_store(cwd: &Path) -> Result<Store> {
    resolve::open_store(cwd)
}
