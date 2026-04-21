use std::path::Path;

use anyhow::{Result, bail};

use crate::indexer;
use crate::repo;
use crate::store::{Store, SymbolResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveResult {
    pub symbol: SymbolResult,
    pub matches: Vec<SymbolResult>,
    pub total_found: usize,
    pub fuzzy: bool,
}

pub fn open_store(cwd: &Path) -> Result<Store> {
    let db_path = repo::configured_db_path(cwd, None)?;
    indexer::ensure_fresh(cwd, &db_path)?;
    Store::open(&db_path)
}

pub fn resolve_symbol(cwd: &Path, arg: &str) -> Result<ResolveResult> {
    let (file_hint, parent_hint, symbol_name) = parse_symbol_arg(arg);
    let store = open_store(cwd)?;

    let mut fuzzy = false;
    let mut results = store.search_symbols(&symbol_name, "", "", true, false, 0)?;
    if results.is_empty() {
        results = store.search_symbols(&symbol_name, "", "", true, true, 0)?;
        if !results.is_empty() {
            fuzzy = true;
        }
    }
    if results.is_empty() {
        results = store.search_symbols(&symbol_name, "", "", false, false, 20)?;
        if !results.is_empty() {
            fuzzy = true;
        }
    }
    if results.is_empty() {
        bail!("symbol not found: {arg}");
    }

    let total_found = results.len();
    if let Some(file_hint) = &file_hint {
        let filtered = filter_by_file_hint(&results, file_hint);
        if !filtered.is_empty() {
            results = filtered;
        }
    }
    if let Some(parent_hint) = &parent_hint {
        let filtered = filter_by_parent_hint(&results, parent_hint);
        if !filtered.is_empty() {
            results = filtered;
        }
    }

    rank_symbols(&mut results);
    let symbol = results[0].clone();

    Ok(ResolveResult {
        symbol,
        matches: results,
        total_found,
        fuzzy,
    })
}

fn parse_symbol_arg(arg: &str) -> (Option<String>, Option<String>, String) {
    if let Some((left, right)) = arg.rsplit_once(':') {
        if !right.is_empty()
            && (left.contains('/') || left.contains('.') || left.contains(std::path::MAIN_SEPARATOR))
            && right.trim_start_matches('L').parse::<usize>().is_err()
        {
            return (Some(left.to_string()), None, right.to_string());
        }
    }

    if let Some((parent, symbol)) = arg.split_once('.') {
        if !parent.is_empty() && !symbol.is_empty() {
            return (None, Some(parent.to_string()), symbol.to_string());
        }
    }

    (None, None, arg.to_string())
}

fn filter_by_file_hint(results: &[SymbolResult], file_hint: &str) -> Vec<SymbolResult> {
    results
        .into_iter()
        .cloned()
        .filter(|result| result.rel_path.ends_with(file_hint) || result.rel_path.contains(file_hint))
        .collect()
}

fn filter_by_parent_hint(results: &[SymbolResult], parent_hint: &str) -> Vec<SymbolResult> {
    results
        .into_iter()
        .cloned()
        .filter(|result| {
            result.parent.eq_ignore_ascii_case(parent_hint) || result.rel_path.contains(parent_hint)
        })
        .collect()
}

pub fn rank_symbols(results: &mut [SymbolResult]) {
    results.sort_by(|left, right| symbol_score(right).cmp(&symbol_score(left)));
}

/// Over-fetch window for ranking: exact queries fetch unbounded (0 means "no
/// SQL LIMIT") so the ranking pass sees every row matching the exact name;
/// prefix/FTS queries cap at min(limit*5, 500) so ranking has headroom
/// without blowing up for large corpora.
pub fn rank_fetch_window(user_limit: usize, exact: bool) -> usize {
    if exact {
        return 0;
    }
    if user_limit == 0 {
        return 500;
    }
    (user_limit * 5).min(500)
}

fn symbol_score(result: &SymbolResult) -> i32 {
    let mut score = 0;
    let path = result.rel_path.to_lowercase();

    score += match result.kind.as_str() {
        "class" | "struct" | "interface" | "type" => 60,
        "function" => 50,
        "method" => 40,
        "enum" => 30,
        "constructor" => 20,
        "impl" => 15,
        "variable" | "constant" => 10,
        _ => 0,
    };

    for segment in [
        "/test/", "/tests/", "/testing/", "_test.go", "_test.", "_spec.", ".test.", ".spec.",
        "/testdata/", "/testutil/", "/testutils/",
    ] {
        if path_matches_segment(&path, segment) {
            score -= 80;
            break;
        }
    }
    for segment in [
        "/playground/", "/example/", "/examples/", "/demo/", "/demos/", "/sample/", "/samples/",
        "/fixture/", "/fixtures/",
    ] {
        if path_matches_segment(&path, segment) {
            score -= 70;
            break;
        }
    }
    for segment in ["/docs/", "/docs_src/", "/doc/", "/documentation/"] {
        if path_matches_segment(&path, segment) {
            score -= 60;
            break;
        }
    }
    for segment in ["/vendor/", "/node_modules/", "/third_party/", "/external/", "/deps/"] {
        if path_matches_segment(&path, segment) {
            score -= 90;
            break;
        }
    }
    for prefix in ["android/", "guava-gwt/"] {
        if path.starts_with(prefix) || path.contains(&format!("/{prefix}")) {
            score -= 50;
            break;
        }
    }
    for segment in [
        ".pb.go", "_pb2.py", "_pb2_grpc.py", "_generated.go", "_gen.go", ".gen.go", ".generated.ts",
        ".generated.js", ".gen.ts", "__generated__", "_pb.d.ts", "_grpc.pb.go", ".g.dart",
    ] {
        if path.ends_with(segment) || path.contains(&format!("{segment}/")) {
            score -= 70;
            break;
        }
    }
    for segment in ["/generated/", "/gen/"] {
        if path_matches_segment(&path, segment) {
            score -= 50;
            break;
        }
    }
    for segment in ["/src/", "/pkg/", "/lib/", "/crates/", "/packages/", "/internal/", "/cmd/"] {
        if path_matches_segment(&path, segment) {
            score += 15;
            break;
        }
    }

    score -= path.matches('/').count() as i32 * 3;
    score -= (path.len() / 10) as i32;
    score
}

fn path_matches_segment(path: &str, segment: &str) -> bool {
    path.contains(segment) || path.starts_with(segment.trim_start_matches('/'))
}
