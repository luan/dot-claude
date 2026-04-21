use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::pathfilters::{include_path, widen_path_filter_limit};
use crate::resolve;
use crate::walker::{WalkOptions, walk};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TextResult {
    pub rel_path: PathBuf,
    pub line: usize,
    pub snippet: String,
}

pub type SymbolResult = crate::store::SymbolResult;

pub fn normalize_search_mode(exact: bool, ignore_case: bool, text_mode: bool) -> Result<bool> {
    if !ignore_case {
        return Ok(exact);
    }
    if text_mode {
        bail!("--ignore-case is not supported with --text");
    }
    Ok(true)
}

pub fn search_text(
    root: &Path,
    query: &str,
    lang: Option<&str>,
    limit: usize,
    includes: &[String],
    excludes: &[String],
    _exact: bool,
) -> Result<Vec<TextResult>> {
    let fetch_limit = widen_path_filter_limit(limit, !includes.is_empty() || !excludes.is_empty());
    let files = walk(
        root,
        &WalkOptions {
            parseable_only: false,
            ignore: Vec::new(),
        },
    )?;

    let mut results = Vec::new();
    for file in files {
        if let Some(lang) = lang {
            if file.language != lang {
                continue;
            }
        }
        if !include_path(&file.rel_path, includes, excludes) {
            continue;
        }

        let contents = fs::read_to_string(&file.path)?;
        for (index, line) in contents.lines().enumerate() {
            if !line.contains(query) {
                continue;
            }
            results.push(TextResult {
                rel_path: file.rel_path.clone(),
                line: index + 1,
                snippet: line.trim().to_string(),
            });
            if fetch_limit > 0 && results.len() >= fetch_limit {
                return Ok(truncate_results(results, limit));
            }
        }
    }

    Ok(truncate_results(results, limit))
}

pub fn search_symbols(
    cwd: &Path,
    query: &str,
    lang: Option<&str>,
    limit: usize,
    exact: bool,
    ignore_case: bool,
    includes: &[String],
    excludes: &[String],
) -> Result<Vec<SymbolResult>> {
    let fetch_limit = widen_path_filter_limit(limit, !includes.is_empty() || !excludes.is_empty());
    let store = resolve::open_store(cwd)?;
    let mut results = store.search_symbols(
        query,
        "",
        lang.unwrap_or(""),
        exact,
        ignore_case,
        fetch_limit,
    )?;
    results.retain(|result| include_path(Path::new(&result.rel_path), includes, excludes));
    if limit > 0 && results.len() > limit {
        results.truncate(limit);
    }
    Ok(results)
}

fn truncate_results(mut results: Vec<TextResult>, limit: usize) -> Vec<TextResult> {
    if limit > 0 && results.len() > limit {
        results.truncate(limit);
    }
    results
}
