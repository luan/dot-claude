use std::path::Path;

use anyhow::Result;

use crate::context::open_store;
use crate::store::StructureResult;

pub fn analyze(cwd: &Path, limit: usize) -> Result<StructureResult> {
    let store = open_store(cwd)?;
    store.structure(limit)
}
