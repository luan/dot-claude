use std::path::Path;

use anyhow::Result;

use crate::pathfilters::{include_path, widen_path_filter_limit};
use crate::resolve;
use crate::store::{ImplementorResult, Store};

pub fn find_implementors(
    cwd: &Path,
    name: &str,
    lang: Option<&str>,
    limit: usize,
    includes: &[String],
    excludes: &[String],
    resolved_only: bool,
    unresolved_only: bool,
) -> Result<Vec<ImplementorResult>> {
    let fetch_limit = widen_path_filter_limit(
        limit,
        !includes.is_empty() || !excludes.is_empty() || lang.is_some() || resolved_only || unresolved_only,
    );
    let store = open_store(cwd)?;
    let mut results = store.find_implementors(name, fetch_limit.max(1))?;
    filter_results(
        &mut results,
        lang,
        includes,
        excludes,
        resolved_only,
        unresolved_only,
        limit,
    );
    Ok(results)
}

pub fn find_implements(
    cwd: &Path,
    name: &str,
    lang: Option<&str>,
    limit: usize,
    includes: &[String],
    excludes: &[String],
    resolved_only: bool,
    unresolved_only: bool,
) -> Result<Vec<ImplementorResult>> {
    let fetch_limit = widen_path_filter_limit(
        limit,
        !includes.is_empty() || !excludes.is_empty() || lang.is_some() || resolved_only || unresolved_only,
    );
    let store = open_store(cwd)?;
    let mut results = store.find_implements(name, fetch_limit.max(1))?;
    filter_results(
        &mut results,
        lang,
        includes,
        excludes,
        resolved_only,
        unresolved_only,
        limit,
    );
    Ok(results)
}

fn filter_results(
    results: &mut Vec<ImplementorResult>,
    lang: Option<&str>,
    includes: &[String],
    excludes: &[String],
    resolved_only: bool,
    unresolved_only: bool,
    limit: usize,
) {
    results.retain(|result| include_path(Path::new(&result.rel_path), includes, excludes));
    if let Some(lang) = lang {
        results.retain(|result| result.language == lang);
    }
    if resolved_only {
        results.retain(|result| result.resolved);
    }
    if unresolved_only {
        results.retain(|result| !result.resolved);
    }
    if limit > 0 && results.len() > limit {
        results.truncate(limit);
    }
}

fn open_store(cwd: &Path) -> Result<Store> {
    resolve::open_store(cwd)
}
