use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use aho_corasick::AhoCorasick;
use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};

use crate::symbols::{Import, Ref, Symbol};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL,
    rel_path TEXT NOT NULL,
    language TEXT NOT NULL,
    hash TEXT NOT NULL,
    mtime_ns INTEGER NOT NULL,
    size INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS symbols (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    start_line INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    start_col INTEGER NOT NULL,
    end_col INTEGER NOT NULL,
    parent TEXT NOT NULL,
    depth INTEGER NOT NULL,
    signature TEXT NOT NULL,
    language TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS imports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    raw_path TEXT NOT NULL,
    language TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS refs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    line INTEGER NOT NULL,
    name TEXT NOT NULL,
    language TEXT NOT NULL,
    kind TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
CREATE INDEX IF NOT EXISTS idx_symbols_kind ON symbols(kind);
CREATE INDEX IF NOT EXISTS idx_symbols_language ON symbols(language);
CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
CREATE INDEX IF NOT EXISTS idx_refs_name ON refs(name);
CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file_id);
CREATE INDEX IF NOT EXISTS idx_imports_file ON imports(file_id);
"#;

#[derive(Debug)]
pub struct Store {
    conn: Connection,
    files_has_indexed_at: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileCheck {
    pub mtime_ns: i64,
    pub size: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FileRecord {
    pub path: String,
    pub rel_path: String,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SymbolResult {
    pub name: String,
    pub kind: String,
    pub parent: String,
    pub language: String,
    pub rel_path: String,
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
    pub depth: usize,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RefResult {
    pub file: String,
    pub rel_path: String,
    pub line: usize,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ImporterResult {
    pub file: String,
    pub rel_path: String,
    pub import: String,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ImpactResult {
    pub symbol: String,
    pub caller: String,
    pub file: String,
    pub rel_path: String,
    pub line: usize,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TraceResult {
    pub caller: String,
    pub callee: String,
    pub file: String,
    pub rel_path: String,
    pub line: usize,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ImplementorResult {
    pub implementer: String,
    pub target: String,
    pub file: String,
    pub rel_path: String,
    pub line: usize,
    pub language: String,
    pub resolved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RepoStats {
    pub path: String,
    pub file_count: usize,
    pub symbol_count: usize,
    pub languages: std::collections::BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct StructureResult {
    pub repo_root: String,
    pub files: usize,
    pub symbols: usize,
    pub languages: std::collections::BTreeMap<String, usize>,
    pub entry_points: Vec<SymbolResult>,
    pub top_by_refs: Vec<RankedSymbol>,
    pub top_by_import_fan: Vec<RankedFile>,
    pub top_packages: Vec<RankedPackage>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RankedSymbol {
    pub symbol: SymbolResult,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RankedFile {
    pub rel_path: String,
    pub language: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RankedPackage {
    pub path: String,
    pub symbols: usize,
    pub files: usize,
}

impl Store {
    pub fn open(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        let conn = Connection::open(db_path)
            .with_context(|| format!("opening {}", db_path.display()))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch(SCHEMA)?;
        let files_has_indexed_at = table_has_column(&conn, "files", "indexed_at")?;
        Ok(Self {
            conn,
            files_has_indexed_at,
        })
    }

    pub fn upsert_file(
        &self,
        path: &str,
        rel_path: &str,
        language: &str,
        hash: &str,
        modified: SystemTime,
        size: i64,
    ) -> Result<i64> {
        let mtime_ns = modified
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        if self.files_has_indexed_at {
            self.conn.execute(
                r#"
                INSERT INTO files (path, rel_path, language, hash, indexed_at, mtime_ns, size)
                VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP, ?5, ?6)
                ON CONFLICT(path) DO UPDATE SET
                    rel_path = excluded.rel_path,
                    language = excluded.language,
                    hash = excluded.hash,
                    indexed_at = CURRENT_TIMESTAMP,
                    mtime_ns = excluded.mtime_ns,
                    size = excluded.size
                "#,
                params![path, rel_path, language, hash, mtime_ns, size],
            )?;
        } else {
            self.conn.execute(
                r#"
                INSERT INTO files (path, rel_path, language, hash, mtime_ns, size)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(path) DO UPDATE SET
                    rel_path = excluded.rel_path,
                    language = excluded.language,
                    hash = excluded.hash,
                    mtime_ns = excluded.mtime_ns,
                    size = excluded.size
                "#,
                params![path, rel_path, language, hash, mtime_ns, size],
            )?;
        }

        self.conn
            .query_row("SELECT id FROM files WHERE path = ?1", [path], |row| row.get(0))
            .context("loading upserted file id")
    }

    pub fn get_meta(&self, key: &str) -> Result<Option<String>> {
        self.conn
            .query_row("SELECT value FROM meta WHERE key = ?1", [key], |row| row.get(0))
            .optional()
            .map_err(Into::into)
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO meta (key, value) VALUES (?1, ?2)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            "#,
            params![key, value],
        )?;
        Ok(())
    }

    pub fn replace_file_contents(
        &self,
        file_id: i64,
        symbols: &[Symbol],
        imports: &[Import],
        refs: &[Ref],
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM symbols WHERE file_id = ?1", [file_id])?;
        tx.execute("DELETE FROM imports WHERE file_id = ?1", [file_id])?;
        tx.execute("DELETE FROM refs WHERE file_id = ?1", [file_id])?;

        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO symbols (
                    file_id, name, kind, start_line, end_line, start_col, end_col,
                    parent, depth, signature, language
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
            )?;
            for symbol in symbols {
                stmt.execute(params![
                    file_id,
                    symbol.name,
                    symbol.kind,
                    symbol.start_line as i64,
                    symbol.end_line as i64,
                    symbol.start_col as i64,
                    symbol.end_col as i64,
                    symbol.parent,
                    symbol.depth as i64,
                    symbol.signature,
                    symbol.language,
                ])?;
            }
        }

        {
            let mut stmt = tx.prepare(
                "INSERT INTO imports (file_id, raw_path, language) VALUES (?1, ?2, ?3)",
            )?;
            for import in imports {
                stmt.execute(params![file_id, import.raw_path, import.language])?;
            }
        }

        {
            let mut stmt = tx.prepare(
                "INSERT INTO refs (file_id, line, name, language, kind) VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            for reference in refs {
                stmt.execute(params![
                    file_id,
                    reference.line as i64,
                    reference.name,
                    reference.language,
                    reference.kind,
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn search_symbols(
        &self,
        query: &str,
        kind: &str,
        language: &str,
        exact: bool,
        ignore_case: bool,
        limit: usize,
    ) -> Result<Vec<SymbolResult>> {
        let mut sql = String::from(
            r#"
            SELECT s.name, s.kind, s.parent, s.language, f.rel_path, f.path, s.start_line, s.end_line, s.signature
            FROM symbols s
            JOIN files f ON f.id = s.file_id
            WHERE 1 = 1
            "#,
        );

        let query_value = if exact {
            if ignore_case {
                sql.push_str(" AND lower(s.name) = lower(?1)");
            } else {
                sql.push_str(" AND s.name = ?1");
            }
            query.to_string()
        } else {
            sql.push_str(" AND s.name LIKE ?1");
            format!("{query}%")
        };

        let mut params_vec = vec![query_value];
        if !kind.is_empty() {
            sql.push_str(&format!(" AND s.kind = ?{}", params_vec.len() + 1));
            params_vec.push(kind.to_string());
        }
        if !language.is_empty() {
            sql.push_str(&format!(" AND s.language = ?{}", params_vec.len() + 1));
            params_vec.push(language.to_string());
        }
        sql.push_str(" ORDER BY s.name, f.rel_path");
        if limit > 0 {
            sql.push_str(&format!(" LIMIT {limit}"));
        }

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok(SymbolResult {
                name: row.get(0)?,
                kind: row.get(1)?,
                parent: row.get(2)?,
                language: row.get(3)?,
                rel_path: row.get(4)?,
                file: row.get(5)?,
                start_line: row.get::<_, i64>(6)? as usize,
                end_line: row.get::<_, i64>(7)? as usize,
                depth: 0,
                signature: row.get(8)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn file_outline(&self, file_path: &Path) -> Result<Vec<SymbolResult>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT s.name, s.kind, s.parent, s.language, f.rel_path, f.path, s.start_line, s.end_line, s.depth, s.signature
            FROM symbols s
            JOIN files f ON f.id = s.file_id
            WHERE f.path = ?1
            ORDER BY s.start_line, s.depth, s.name
            "#,
        )?;
        let rows = stmt.query_map([file_path.to_string_lossy().as_ref()], |row| {
            Ok(SymbolResult {
                name: row.get(0)?,
                kind: row.get(1)?,
                parent: row.get(2)?,
                language: row.get(3)?,
                rel_path: row.get(4)?,
                file: row.get(5)?,
                start_line: row.get::<_, i64>(6)? as usize,
                end_line: row.get::<_, i64>(7)? as usize,
                depth: row.get::<_, i64>(8)? as usize,
                signature: row.get(9)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn child_symbols(&self, parent_name: &str, limit: usize, file_path: Option<&str>) -> Result<Vec<SymbolResult>> {
        let (sql, params): (&str, Vec<String>) = if let Some(file_path) = file_path {
            (
                r#"
                SELECT s.name, s.kind, s.parent, s.language, f.rel_path, f.path, s.start_line, s.end_line, s.depth, s.signature
                FROM symbols s
                JOIN files f ON s.file_id = f.id
                WHERE s.parent = ?1 AND f.path = ?2
                ORDER BY s.start_line
                LIMIT ?3
                "#,
                vec![parent_name.to_string(), file_path.to_string(), limit.to_string()],
            )
        } else {
            (
                r#"
                SELECT s.name, s.kind, s.parent, s.language, f.rel_path, f.path, s.start_line, s.end_line, s.depth, s.signature
                FROM symbols s
                JOIN files f ON s.file_id = f.id
                WHERE s.parent = ?1
                ORDER BY s.start_line
                LIMIT ?2
                "#,
                vec![parent_name.to_string(), limit.to_string()],
            )
        };

        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(SymbolResult {
                name: row.get(0)?,
                kind: row.get(1)?,
                parent: row.get(2)?,
                language: row.get(3)?,
                rel_path: row.get(4)?,
                file: row.get(5)?,
                start_line: row.get::<_, i64>(6)? as usize,
                end_line: row.get::<_, i64>(7)? as usize,
                depth: row.get::<_, i64>(8)? as usize,
                signature: row.get(9)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn file_imports(&self, file_path: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT i.raw_path
            FROM imports i
            JOIN files f ON i.file_id = f.id
            WHERE f.path = ?1
            ORDER BY i.raw_path
            "#,
        )?;
        let rows = stmt.query_map([file_path], |row| row.get(0))?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn all_files(&self, language: Option<&str>) -> Result<Vec<FileRecord>> {
        let mut sql = String::from("SELECT path, rel_path, language FROM files");
        let mut params_vec = Vec::new();
        if let Some(language) = language {
            sql.push_str(" WHERE language = ?1");
            params_vec.push(language.to_string());
        }
        sql.push_str(" ORDER BY rel_path");
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok(FileRecord {
                path: row.get(0)?,
                rel_path: row.get(1)?,
                language: row.get(2)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn repo_stats(&self) -> Result<RepoStats> {
        let path = self.get_meta("repo_root")?.unwrap_or_default();
        let file_count = self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get::<_, i64>(0))?
            as usize;
        let symbol_count = self
            .conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get::<_, i64>(0))?
            as usize;

        let mut stmt = self.conn.prepare(
            "SELECT language, COUNT(*) FROM files GROUP BY language ORDER BY language",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?;

        let mut languages = std::collections::BTreeMap::new();
        for row in rows {
            let (language, count) = row?;
            languages.insert(language, count);
        }

        Ok(RepoStats {
            path,
            file_count,
            symbol_count,
            languages,
        })
    }

    pub fn structure(&self, limit: usize) -> Result<StructureResult> {
        let limit = limit.max(1);
        let repo_stats = self.repo_stats()?;
        let mut result = StructureResult {
            repo_root: repo_stats.path,
            files: repo_stats.file_count,
            symbols: repo_stats.symbol_count,
            languages: repo_stats.languages,
            entry_points: Vec::new(),
            top_by_refs: Vec::new(),
            top_by_import_fan: Vec::new(),
            top_packages: Vec::new(),
        };

        let mut entry_stmt = self.conn.prepare(
            r#"
            SELECT s.name, s.kind, s.parent, s.language, f.rel_path, f.path, s.start_line, s.end_line, s.depth, s.signature
            FROM symbols s
            JOIN files f ON s.file_id = f.id
            WHERE s.depth = 0 AND s.kind IN ('function', 'method')
              AND (s.name IN ('main', 'init', 'Main', 'Init')
                   OR (substr(s.name, 1, 1) = upper(substr(s.name, 1, 1))
                       AND s.kind = 'function'
                       AND f.rel_path LIKE '%main%'))
            ORDER BY s.name
            LIMIT ?1
            "#,
        )?;
        let entry_rows = entry_stmt.query_map([limit as i64], |row| {
            Ok(SymbolResult {
                name: row.get(0)?,
                kind: row.get(1)?,
                parent: row.get(2)?,
                language: row.get(3)?,
                rel_path: row.get(4)?,
                file: row.get(5)?,
                start_line: row.get::<_, i64>(6)? as usize,
                end_line: row.get::<_, i64>(7)? as usize,
                depth: row.get::<_, i64>(8)? as usize,
                signature: row.get(9)?,
            })
        })?;
        result.entry_points = entry_rows.collect::<std::result::Result<Vec<_>, _>>()?;

        let mut ref_stmt = self.conn.prepare(
            r#"
            WITH top_names AS (
                SELECT name, COUNT(*) AS cnt
                FROM refs
                GROUP BY name
                ORDER BY cnt DESC
                LIMIT ?2
            )
            SELECT t.name, t.cnt, s.kind, s.parent, s.language, f.rel_path, f.path, s.start_line, s.end_line, s.depth, s.signature
            FROM top_names t
            JOIN symbols s ON s.name = t.name AND s.depth = 0
            JOIN files f ON s.file_id = f.id
            ORDER BY t.cnt DESC, f.rel_path
            LIMIT ?1
            "#,
        )?;
        let ref_rows = ref_stmt.query_map([limit as i64, (limit as i64).saturating_mul(4)], |row| {
            Ok(RankedSymbol {
                symbol: SymbolResult {
                    name: row.get(0)?,
                    kind: row.get(2)?,
                    parent: row.get(3)?,
                    language: row.get(4)?,
                    rel_path: row.get(5)?,
                    file: row.get(6)?,
                    start_line: row.get::<_, i64>(7)? as usize,
                    end_line: row.get::<_, i64>(8)? as usize,
                    depth: row.get::<_, i64>(9)? as usize,
                    signature: row.get(10)?,
                },
                count: row.get::<_, i64>(1)? as usize,
            })
        })?;
        result.top_by_refs = ref_rows.collect::<std::result::Result<Vec<_>, _>>()?;

        result.top_by_import_fan = self.compute_import_fan(limit)?;

        let mut pkg_stmt = self.conn.prepare(
            r#"
            SELECT f.rel_path, COUNT(s.id)
            FROM files f
            JOIN symbols s ON s.file_id = f.id
            GROUP BY f.id
            "#,
        )?;
        let pkg_rows = pkg_stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?;
        let mut packages = std::collections::BTreeMap::<String, RankedPackage>::new();
        for row in pkg_rows {
            let (rel_path, symbol_count) = row?;
            let bucket = package_bucket(&rel_path);
            let entry = packages.entry(bucket.clone()).or_insert(RankedPackage {
                path: bucket,
                symbols: 0,
                files: 0,
            });
            entry.symbols += symbol_count;
            entry.files += 1;
        }
        let mut top_packages = packages.into_values().collect::<Vec<_>>();
        top_packages.sort_by(|a, b| b.symbols.cmp(&a.symbols).then_with(|| a.path.cmp(&b.path)));
        top_packages.truncate(limit);
        result.top_packages = top_packages;

        Ok(result)
    }

    fn compute_import_fan(&self, limit: usize) -> Result<Vec<RankedFile>> {
        let files = self.all_files(None)?;
        if files.is_empty() {
            return Ok(Vec::new());
        }

        let mut patterns: Vec<String> = Vec::with_capacity(files.len() * 3);
        let mut pattern_owners: Vec<usize> = Vec::with_capacity(files.len() * 3);
        for (idx, file) in files.iter().enumerate() {
            let rel_without_ext = strip_known_extension(&file.rel_path);
            let stem = file_stem_fragment(&file.rel_path);
            patterns.push(file.rel_path.clone());
            pattern_owners.push(idx);
            if rel_without_ext != file.rel_path {
                patterns.push(rel_without_ext);
                pattern_owners.push(idx);
            }
            if !stem.is_empty() {
                patterns.push(stem);
                pattern_owners.push(idx);
            }
        }
        let ac = AhoCorasick::new(&patterns).context("building aho-corasick automaton")?;

        let mut import_stmt = self.conn.prepare(
            r#"
            SELECT importer.id, i.raw_path
            FROM imports i
            JOIN files importer ON importer.id = i.file_id
            "#,
        )?;
        let import_rows = import_stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut fan_in: Vec<std::collections::HashSet<i64>> =
            (0..files.len()).map(|_| std::collections::HashSet::new()).collect();
        let mut hit_files: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for row in import_rows {
            let (importer_id, raw_path) = row?;
            hit_files.clear();
            for m in ac.find_overlapping_iter(&raw_path) {
                hit_files.insert(pattern_owners[m.pattern().as_usize()]);
            }
            for &file_idx in &hit_files {
                fan_in[file_idx].insert(importer_id);
            }
        }

        let mut top: Vec<RankedFile> = files
            .iter()
            .zip(fan_in.iter())
            .filter_map(|(file, importers)| {
                if importers.len() > 1 {
                    Some(RankedFile {
                        rel_path: file.rel_path.clone(),
                        language: file.language.clone(),
                        count: importers.len(),
                    })
                } else {
                    None
                }
            })
            .collect();
        top.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.rel_path.cmp(&b.rel_path)));
        top.truncate(limit);
        Ok(top)
    }

    pub fn find_references(&self, name: &str, limit: usize, kinds: &[&str]) -> Result<Vec<RefResult>> {
        if kinds.is_empty() {
            let mut stmt = self.conn.prepare(
                r#"
                SELECT f.path, f.rel_path, r.line, r.name
                FROM refs r
                JOIN files f ON r.file_id = f.id
                WHERE r.name = ?1
                ORDER BY f.rel_path, r.line
                LIMIT ?2
                "#,
            )?;
            let rows = stmt.query_map(params![name, limit as i64], |row| {
                Ok(RefResult {
                    file: row.get(0)?,
                    rel_path: row.get(1)?,
                    line: row.get::<_, i64>(2)? as usize,
                    name: row.get(3)?,
                })
            })?;
            return Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?);
        }

        let placeholders = vec!["?"; kinds.len()].join(", ");
        let sql = format!(
            r#"
            SELECT f.path, f.rel_path, r.line, r.name
            FROM refs r
            JOIN files f ON r.file_id = f.id
            WHERE r.name = ? AND r.kind IN ({placeholders})
            ORDER BY f.rel_path, r.line
            LIMIT ?
            "#
        );
        let mut params_vec = Vec::with_capacity(kinds.len() + 2);
        params_vec.push(name.to_string());
        params_vec.extend(kinds.iter().map(|kind| kind.to_string()));
        params_vec.push((limit as i64).to_string());

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok(RefResult {
                file: row.get(0)?,
                rel_path: row.get(1)?,
                line: row.get::<_, i64>(2)? as usize,
                name: row.get(3)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn find_importers(&self, symbol_name: &str, depth: usize, limit: usize) -> Result<Vec<ImporterResult>> {
        let depth = depth.clamp(1, 3);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT DISTINCT f.rel_path
            FROM symbols s
            JOIN files f ON s.file_id = f.id
            WHERE s.name = ?1
            "#,
        )?;
        let rows = stmt.query_map([symbol_name], |row| row.get::<_, String>(0))?;
        let targets = rows.collect::<std::result::Result<Vec<_>, _>>()?;
        if targets.is_empty() {
            return Ok(Vec::new());
        }
        self.find_importers_for_targets(targets, depth, limit)
    }

    pub fn find_importers_by_path(&self, target: &str, depth: usize, limit: usize) -> Result<Vec<ImporterResult>> {
        let depth = depth.clamp(1, 3);
        self.find_importers_for_targets(vec![target.to_string()], depth, limit)
    }

    pub fn enclosing_symbol(&self, file_path: &str, line: usize) -> Result<Option<String>> {
        self.conn
            .query_row(
                r#"
                SELECT s.name
                FROM symbols s
                WHERE s.file_id = (SELECT id FROM files WHERE path = ?1)
                  AND s.start_line <= ?2 AND s.end_line >= ?2
                ORDER BY (s.end_line - s.start_line) ASC
                LIMIT 1
                "#,
                params![file_path, line as i64],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn find_impact(&self, symbol_name: &str, depth: usize, limit: usize) -> Result<Vec<ImpactResult>> {
        let depth = depth.clamp(1, 5);
        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();
        let mut current_symbols = vec![symbol_name.to_string()];

        for current_depth in 1..=depth {
            if current_symbols.is_empty() || (limit > 0 && results.len() >= limit) {
                break;
            }

            let mut next_symbols = Vec::new();
            for symbol in current_symbols {
                for reference in self.find_references(&symbol, limit.max(1), &[])? {
                    let Some(caller) = self.enclosing_symbol(&reference.file, reference.line)? else {
                        continue;
                    };
                    if caller == symbol {
                        continue;
                    }

                    let key = format!("{}@{}", caller, reference.file);
                    if !seen.insert(key) {
                        continue;
                    }

                    results.push(ImpactResult {
                        symbol: symbol.clone(),
                        caller: caller.clone(),
                        file: reference.file,
                        rel_path: reference.rel_path,
                        line: reference.line,
                        depth: current_depth,
                    });
                    next_symbols.push(caller);

                    if limit > 0 && results.len() >= limit {
                        return Ok(results);
                    }
                }
            }

            current_symbols = next_symbols;
        }

        Ok(results)
    }

    pub fn find_trace(
        &self,
        symbol_name: &str,
        depth: usize,
        limit: usize,
        kinds: &[&str],
    ) -> Result<Vec<TraceResult>> {
        let depth = if depth == 0 { 3 } else { depth.min(5) };
        let kinds = if kinds.is_empty() {
            vec![crate::symbols::REF_KIND_CALL]
        } else {
            kinds.to_vec()
        };
        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();
        let mut current_symbols = self.resolve_symbols(symbol_name)?;

        for current_depth in 1..=depth {
            if current_symbols.is_empty() || (limit > 0 && results.len() >= limit) {
                break;
            }

            let mut next_symbols = Vec::new();
            for symbol in current_symbols {
                for edge in self.callees_of(&symbol, &kinds)? {
                    if edge.callee == symbol.name || !is_project_symbol(&edge.callee) {
                        continue;
                    }
                    let key = format!("{}→{}", symbol.name, edge.callee);
                    if !seen.insert(key) {
                        continue;
                    }

                    results.push(TraceResult {
                        caller: symbol.name.clone(),
                        callee: edge.callee.clone(),
                        file: edge.file,
                        rel_path: edge.rel_path,
                        line: edge.line,
                        depth: current_depth,
                    });
                    next_symbols.extend(self.resolve_symbols(&edge.callee)?);

                    if limit > 0 && results.len() >= limit {
                        return Ok(results);
                    }
                }
            }

            current_symbols = next_symbols;
        }

        Ok(results)
    }

    pub fn find_implementors(&self, target: &str, limit: usize) -> Result<Vec<ImplementorResult>> {
        let resolved = self.symbol_exists(target)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT f.path, f.rel_path, r.line
            FROM refs r
            JOIN files f ON r.file_id = f.id
            WHERE r.name = ?1 AND r.kind = ?2
            ORDER BY f.rel_path, r.line
            LIMIT ?3
            "#,
        )?;
        let rows = stmt.query_map(
            params![target, crate::symbols::REF_KIND_IMPLEMENTS, limit.max(1) as i64],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)? as usize,
                ))
            },
        )?;

        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();
        for row in rows {
            let (file, rel_path, line) = row?;
            let Some(owner) = self.enclosing_type_symbol(&file, line)? else {
                continue;
            };
            let key = format!("{}:{}:{}", owner.file, owner.start_line, target);
            if !seen.insert(key) {
                continue;
            }
            results.push(ImplementorResult {
                implementer: owner.name,
                target: target.to_string(),
                file,
                rel_path,
                line,
                language: owner.language,
                resolved,
            });
        }
        Ok(results)
    }

    pub fn find_implements(&self, name: &str, limit: usize) -> Result<Vec<ImplementorResult>> {
        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();

        for symbol in self.resolve_type_symbols(name)? {
            let mut stmt = self.conn.prepare(
                r#"
                SELECT f.path, f.rel_path, r.line, r.name
                FROM refs r
                JOIN files f ON r.file_id = f.id
                WHERE f.path = ?1
                  AND r.line >= ?2 AND r.line <= ?3
                  AND r.kind = ?4
                ORDER BY r.line
                "#,
            )?;
            let rows = stmt.query_map(
                params![
                    symbol.file,
                    symbol.start_line as i64,
                    symbol.end_line as i64,
                    crate::symbols::REF_KIND_IMPLEMENTS,
                ],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)? as usize,
                        row.get::<_, String>(3)?,
                    ))
                },
            )?;

            for row in rows {
                let (file, rel_path, line, target) = row?;
                let Some(owner) = self.enclosing_type_symbol(&file, line)? else {
                    continue;
                };
                if owner.file != symbol.file
                    || owner.start_line != symbol.start_line
                    || owner.end_line != symbol.end_line
                    || owner.kind != symbol.kind
                {
                    continue;
                }
                let key = format!("{}:{}:{}", symbol.file, line, target);
                if !seen.insert(key) {
                    continue;
                }
                results.push(ImplementorResult {
                    implementer: owner.name.clone(),
                    target: target.clone(),
                    file,
                    rel_path,
                    line,
                    language: owner.language.clone(),
                    resolved: self.symbol_exists(&target)?,
                });
                if limit > 0 && results.len() >= limit {
                    return Ok(results);
                }
            }
        }

        Ok(results)
    }

    pub fn all_file_checks(&self) -> Result<HashMap<String, FileCheck>> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, mtime_ns, size FROM files ORDER BY path")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                FileCheck {
                    mtime_ns: row.get(1)?,
                    size: row.get(2)?,
                },
            ))
        })?;
        let mut checks = HashMap::new();
        for row in rows {
            let (path, check) = row?;
            checks.insert(path, check);
        }
        Ok(checks)
    }

    pub fn all_stored_paths(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT path FROM files ORDER BY path")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn delete_stale_paths(&self, current_paths: &[String]) -> Result<usize> {
        let current = current_paths.iter().cloned().collect::<std::collections::HashSet<_>>();
        let stored = self.all_stored_paths()?;
        let stale = stored
            .into_iter()
            .filter(|path| !current.contains(path))
            .collect::<Vec<_>>();
        if stale.is_empty() {
            return Ok(0);
        }

        let tx = self.conn.unchecked_transaction()?;
        let mut deleted = 0;
        for path in stale {
            deleted += tx.execute("DELETE FROM files WHERE path = ?1", [path])?;
        }
        tx.commit()?;
        Ok(deleted)
    }

    pub fn file_hash(&self, path: &str) -> Result<Option<String>> {
        self.conn
            .query_row("SELECT hash FROM files WHERE path = ?1", [path], |row| row.get(0))
            .optional()
            .map_err(Into::into)
    }

    fn find_importers_for_targets(
        &self,
        targets: Vec<String>,
        depth: usize,
        limit: usize,
    ) -> Result<Vec<ImporterResult>> {
        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();
        let mut current_targets = targets;

        for current_depth in 1..=depth {
            if current_targets.is_empty() || (limit > 0 && results.len() >= limit) {
                break;
            }

            let mut next_targets = Vec::new();
            for target in current_targets {
                let raw_pattern = format!("%{target}");
                let rel_pattern = format!("%{}%", file_stem_fragment(&target));
                let mut stmt = self.conn.prepare(
                    r#"
                    SELECT DISTINCT f.path, f.rel_path, i.raw_path
                    FROM imports i
                    JOIN files f ON i.file_id = f.id
                    WHERE i.raw_path LIKE ?1 OR i.raw_path LIKE ?2
                    LIMIT ?3
                    "#,
                )?;
                let rows = stmt.query_map(
                    params![raw_pattern, rel_pattern, limit.max(1) as i64],
                    |row| {
                        Ok(ImporterResult {
                            file: row.get(0)?,
                            rel_path: row.get(1)?,
                            import: row.get(2)?,
                            depth: current_depth,
                        })
                    },
                )?;

                for row in rows {
                    let importer = row?;
                    if !seen.insert(importer.rel_path.clone()) {
                        continue;
                    }
                    next_targets.push(importer.rel_path.clone());
                    results.push(importer);
                    if limit > 0 && results.len() >= limit {
                        return Ok(results);
                    }
                }
            }

            current_targets = next_targets;
        }

        Ok(results)
    }

    fn resolve_symbols(&self, name: &str) -> Result<Vec<SymbolLocation>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT s.name, f.path, s.start_line, s.end_line
            FROM symbols s
            JOIN files f ON s.file_id = f.id
            WHERE s.name = ?1
            "#,
        )?;
        let rows = stmt.query_map([name], |row| {
            Ok(SymbolLocation {
                name: row.get(0)?,
                file: row.get(1)?,
                start_line: row.get::<_, i64>(2)? as usize,
                end_line: row.get::<_, i64>(3)? as usize,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    fn resolve_type_symbols(&self, name: &str) -> Result<Vec<TypeSymbol>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT s.name, s.kind, f.path, s.start_line, s.end_line, s.language
            FROM symbols s
            JOIN files f ON s.file_id = f.id
            WHERE s.name = ?1 AND s.kind IN ('class', 'struct', 'interface', 'trait', 'enum', 'protocol', 'impl')
            ORDER BY s.start_line
            "#,
        )?;
        let rows = stmt.query_map([name], |row| {
            Ok(TypeSymbol {
                name: row.get(0)?,
                kind: row.get(1)?,
                file: row.get(2)?,
                start_line: row.get::<_, i64>(3)? as usize,
                end_line: row.get::<_, i64>(4)? as usize,
                language: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    fn enclosing_type_symbol(&self, file_path: &str, line: usize) -> Result<Option<TypeSymbol>> {
        self.conn
            .query_row(
                r#"
                SELECT s.name, s.kind, f.path, s.start_line, s.end_line, s.language
                FROM symbols s
                JOIN files f ON s.file_id = f.id
                WHERE f.path = ?1
                  AND s.start_line <= ?2 AND s.end_line >= ?2
                  AND s.kind IN ('class', 'struct', 'interface', 'trait', 'enum', 'protocol', 'impl')
                ORDER BY (s.end_line - s.start_line) ASC
                LIMIT 1
                "#,
                params![file_path, line as i64],
                |row| {
                    Ok(TypeSymbol {
                        name: row.get(0)?,
                        kind: row.get(1)?,
                        file: row.get(2)?,
                        start_line: row.get::<_, i64>(3)? as usize,
                        end_line: row.get::<_, i64>(4)? as usize,
                        language: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn symbol_exists(&self, name: &str) -> Result<bool> {
        self.conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM symbols WHERE name = ?1)",
                [name],
                |row| row.get::<_, i64>(0),
            )
            .map(|value| value != 0)
            .map_err(Into::into)
    }

    fn callees_of(&self, symbol: &SymbolLocation, kinds: &[&str]) -> Result<Vec<TraceEdge>> {
        let placeholders = vec!["?"; kinds.len()].join(", ");
        let sql = format!(
            r#"
            SELECT r.name, f.path, f.rel_path, r.line
            FROM refs r
            JOIN files f ON r.file_id = f.id
            WHERE f.path = ? AND r.line >= ? AND r.line <= ?
              AND r.kind IN ({placeholders})
            "#
        );
        let mut params_vec = vec![
            symbol.file.clone(),
            symbol.start_line.to_string(),
            symbol.end_line.to_string(),
        ];
        params_vec.extend(kinds.iter().map(|kind| kind.to_string()));

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok(TraceEdge {
                callee: row.get(0)?,
                file: row.get(1)?,
                rel_path: row.get(2)?,
                line: row.get::<_, i64>(3)? as usize,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let pragma = format!("PRAGMA table_info({table})");
    let mut stmt = conn.prepare(&pragma)?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        if row? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

#[derive(Debug, Clone)]
struct SymbolLocation {
    name: String,
    file: String,
    start_line: usize,
    end_line: usize,
}

#[derive(Debug, Clone)]
struct TraceEdge {
    callee: String,
    file: String,
    rel_path: String,
    line: usize,
}

#[derive(Debug, Clone)]
struct TypeSymbol {
    name: String,
    kind: String,
    file: String,
    start_line: usize,
    end_line: usize,
    language: String,
}

fn file_stem_fragment(target: &str) -> String {
    Path::new(target)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or(target)
        .to_string()
}

fn strip_known_extension(path: &str) -> String {
    Path::new(path)
        .with_extension("")
        .to_string_lossy()
        .into_owned()
}

fn package_bucket(rel_path: &str) -> String {
    let parts = rel_path.split('/').collect::<Vec<_>>();
    match parts.as_slice() {
        [] => ".".into(),
        [single] => (*single).into(),
        [first, second, ..] => format!("{first}/{second}"),
    }
}

fn is_project_symbol(name: &str) -> bool {
    match name {
        "len" | "cap" | "make" | "append" | "close" | "delete" | "copy" | "new"
        | "panic" | "recover" | "int" | "int8" | "int16" | "int32" | "int64"
        | "uint" | "uint8" | "uint16" | "uint32" | "uint64" | "float32"
        | "float64" | "string" | "bool" | "byte" | "rune" | "error" | "nil"
        | "Errorf" | "Sprintf" | "Fprintf" | "Printf" | "Println" | "Error"
        | "String" | "Close" | "Read" | "Write" | "Lock" | "Unlock" | "RLock"
        | "RUnlock" | "Add" | "Load" | "Store" | "Done" | "Wait" | "Begin"
        | "Commit" | "Rollback" | "Exec" | "Query" | "QueryRow" | "Scan" | "Now"
        | "Since" | "Sleep" | "Join" | "Split" | "Contains" | "HasPrefix"
        | "HasSuffix" | "TrimPrefix" | "TrimSuffix" | "Open" | "Create" | "Remove"
        | "Stat" | "Lstat" | "ReadFile" | "WriteFile" | "Abs" | "Dir" | "Base"
        | "Ext" | "Rel" | "Go" | "Next" | "Rows" => false,
        _ => name.len() > 2,
    }
}
