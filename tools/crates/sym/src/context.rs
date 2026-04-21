use std::path::Path;

use anyhow::Result;

use crate::impls;
use crate::resolve;
use crate::show;
use crate::store::{ImplementorResult, RefResult, Store, SymbolResult};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ContextResult {
    pub symbol: SymbolResult,
    pub source: String,
    pub callers: Vec<RefResult>,
    pub file_imports: Vec<String>,
    pub implementors: Vec<ImplementorResult>,
    pub implements: Vec<ImplementorResult>,
}

pub fn symbol_context(cwd: &Path, symbol_name: &str, caller_limit: usize) -> Result<ContextResult> {
    let symbol = resolve_symbol(cwd, symbol_name)?;
    let store = open_store(cwd)?;
    let source = read_symbol_source(&symbol, None)?;
    let callers = store.find_references(&symbol.name, caller_limit.max(1), &[])?;
    let file_imports = store.file_imports(&symbol.file)?;

    let (implementors, implements) = if is_type_like(&symbol.kind) {
        (
            impls::find_implementors(cwd, &symbol.name, None, 20, &[], &[], false, false)?,
            impls::find_implements(cwd, &symbol.name, None, 20, &[], &[], false, false)?,
        )
    } else {
        (Vec::new(), Vec::new())
    };

    Ok(ContextResult {
        symbol,
        source,
        callers,
        file_imports,
        implementors,
        implements,
    })
}

pub(crate) fn open_store(cwd: &Path) -> Result<Store> {
    resolve::open_store(cwd)
}

pub(crate) fn resolve_symbol(cwd: &Path, symbol_name: &str) -> Result<SymbolResult> {
    Ok(resolve::resolve_symbol(cwd, symbol_name)?.symbol)
}

pub(crate) fn read_symbol_source(symbol: &SymbolResult, max_lines: Option<usize>) -> Result<String> {
    let end_line = max_lines
        .map(|max_lines| symbol.end_line.min(symbol.start_line + max_lines.saturating_sub(1)))
        .unwrap_or(symbol.end_line);
    let lines = show::show_file(Path::new(&symbol.file), Some((symbol.start_line, end_line)), 0)?;
    let mut source = lines
        .iter()
        .map(|line| line.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    source.push('\n');
    if let Some(max_lines) = max_lines {
        if symbol.end_line.saturating_sub(symbol.start_line) + 1 > max_lines {
            source.push_str(&format!(
                "... ({} more lines — see sym show {}:{}-{})\n",
                symbol.end_line - end_line,
                symbol.rel_path,
                symbol.start_line,
                symbol.end_line
            ));
        }
    }
    Ok(source)
}

pub(crate) fn is_type_like(kind: &str) -> bool {
    matches!(
        kind,
        "class"
            | "struct"
            | "type"
            | "interface"
            | "trait"
            | "enum"
            | "object"
            | "mixin"
            | "extension"
            | "protocol"
            | "record"
            | "actor"
    )
}
