use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::resolve;
use crate::store::SymbolResult;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LineEntry {
    pub line: usize,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ShownSymbol {
    pub symbol: SymbolResult,
    pub lines: Vec<LineEntry>,
    pub content: String,
}

pub fn show_symbol(cwd: &Path, name: &str, context: usize, show_all: bool) -> Result<Vec<ShownSymbol>> {
    let resolved = resolve::resolve_symbol(cwd, name)?;
    let mut results = resolved.matches;
    if !show_all {
        results.truncate(1);
    }

    results
        .into_iter()
        .map(|symbol| {
            let lines = show_file(Path::new(&symbol.file), Some((symbol.start_line, symbol.end_line)), context)?;
            let content = lines
                .iter()
                .map(|line| line.content.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            Ok(ShownSymbol {
                symbol,
                lines,
                content: format!("{content}\n"),
            })
        })
        .collect()
}

pub fn show_file(path: &Path, range: Option<(usize, usize)>, context: usize) -> Result<Vec<LineEntry>> {
    let path = path.canonicalize().with_context(|| format!("resolving {}", path.display()))?;
    let contents = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;

    let (start, end) = range.unwrap_or((1, usize::MAX));
    let start = if range.is_some() {
        start.saturating_sub(context).max(1)
    } else {
        1
    };
    let end = if range.is_some() {
        end.saturating_add(context)
    } else {
        usize::MAX
    };

    Ok(contents
        .lines()
        .enumerate()
        .filter_map(|(index, content)| {
            let line = index + 1;
            (line >= start && line <= end).then(|| LineEntry {
                line,
                content: content.to_string(),
            })
        })
        .collect())
}
