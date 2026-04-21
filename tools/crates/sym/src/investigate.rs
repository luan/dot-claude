use std::path::Path;

use anyhow::Result;

use crate::context::{is_type_like, open_store, read_symbol_source, resolve_symbol};
use crate::store::{ImplementorResult, ImpactResult, RefResult, SymbolResult};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct InvestigateResult {
    pub symbol: SymbolResult,
    pub source: String,
    pub kind: String,
    pub refs: Vec<RefResult>,
    pub impact: Vec<ImpactResult>,
    pub members: Vec<SymbolResult>,
    pub implementors: Vec<ImplementorResult>,
    pub implements: Vec<ImplementorResult>,
}

pub fn investigate(cwd: &Path, symbol_name: &str) -> Result<InvestigateResult> {
    let symbol = resolve_symbol(cwd, symbol_name)?;
    let store = open_store(cwd)?;

    let mut result = InvestigateResult {
        source: if is_type_like(&symbol.kind) {
            read_symbol_source(&symbol, Some(60))?
        } else {
            read_symbol_source(&symbol, None)?
        },
        kind: symbol.kind.clone(),
        symbol,
        refs: Vec::new(),
        impact: Vec::new(),
        members: Vec::new(),
        implementors: Vec::new(),
        implements: Vec::new(),
    };

    match result.symbol.kind.as_str() {
        "function" | "method" => {
            result.kind = "function".into();
            result.refs = store.find_references(&result.symbol.name, 20, &[])?;
            result.impact = store.find_impact(&result.symbol.name, 2, 20)?;
        }
        kind if is_type_like(kind) => {
            result.kind = "type".into();
            result.members = store.child_symbols(&result.symbol.name, 50, Some(&result.symbol.file))?;
            result.refs = store.find_references(&result.symbol.name, 20, &[])?;
            result.implementors = store.find_implementors(&result.symbol.name, 20)?;
            result.implements = store.find_implements(&result.symbol.name, 20)?;
        }
        _ => {
            result.refs = store.find_references(&result.symbol.name, 20, &[])?;
        }
    }

    Ok(result)
}
