use std::path::Path;

use anyhow::Result;

use crate::repo;
use crate::indexer;
use crate::store::{Store, SymbolResult};

pub fn file_outline(cwd: &Path, file_path: &Path) -> Result<Vec<SymbolResult>> {
    let db_path = repo::configured_db_path(cwd, None)?;
    indexer::ensure_fresh(cwd, &db_path)?;

    let store = Store::open(&db_path)?;
    store.file_outline(&file_path.canonicalize()?)
}
