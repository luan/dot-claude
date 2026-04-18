use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::SystemTime;

use rusqlite::{Connection, Transaction, params};
use serde_json::Value;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

use crate::artifact::{current_project, project_name};

include!(concat!(env!("OUT_DIR"), "/grammar_tags.rs"));

const SCHEMA: &str = "\
    CREATE TABLE IF NOT EXISTS symbols (\
        name TEXT NOT NULL, \
        kind TEXT NOT NULL, \
        file TEXT NOT NULL, \
        line INTEGER NOT NULL\
    ); \
    CREATE INDEX IF NOT EXISTS symbols_by_name ON symbols(name); \
    CREATE INDEX IF NOT EXISTS symbols_by_file ON symbols(file);\
";

struct Grammar {
    crate_name: &'static str,
    extensions: &'static [&'static str],
    language: fn() -> tree_sitter::Language,
}

static GRAMMARS: &[Grammar] = &[
    Grammar {
        crate_name: "tree-sitter-rust",
        extensions: &["rs"],
        language: || tree_sitter_rust::LANGUAGE.into(),
    },
    Grammar {
        crate_name: "tree-sitter-swift",
        extensions: &["swift"],
        language: || tree_sitter_swift::LANGUAGE.into(),
    },
    Grammar {
        crate_name: "tree-sitter-python",
        extensions: &["py", "pyi"],
        language: || tree_sitter_python::LANGUAGE.into(),
    },
    Grammar {
        crate_name: "tree-sitter-go",
        extensions: &["go"],
        language: || tree_sitter_go::LANGUAGE.into(),
    },
    Grammar {
        crate_name: "tree-sitter-ruby",
        extensions: &["rb"],
        language: || tree_sitter_ruby::LANGUAGE.into(),
    },
    Grammar {
        crate_name: "tree-sitter-typescript",
        extensions: &["ts"],
        language: || tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
    },
    Grammar {
        crate_name: "tree-sitter-typescript",
        extensions: &["tsx"],
        language: || tree_sitter_typescript::LANGUAGE_TSX.into(),
    },
    Grammar {
        crate_name: "tree-sitter-javascript",
        extensions: &["js", "jsx", "mjs", "cjs"],
        language: || tree_sitter_javascript::LANGUAGE.into(),
    },
];

fn grammar_for(path: &Path) -> Option<&'static Grammar> {
    let ext = path.extension().and_then(|e| e.to_str())?;
    GRAMMARS.iter().find(|g| g.extensions.contains(&ext))
}

fn tags_query_for(crate_name: &str) -> Option<&'static str> {
    GRAMMAR_TAGS
        .iter()
        .find(|(n, _)| *n == crate_name)
        .map(|(_, q)| *q)
}

/// Compiled query + owned Language, cached per Grammar to avoid recompiling
/// the tags.scm query on every parse. Query compile dominates per-file cost
/// otherwise — easily 100-500ms on a 60-line tags file.
struct CompiledGrammar {
    language: tree_sitter::Language,
    query: Query,
}

fn compiled(grammar: &Grammar) -> Option<&'static CompiledGrammar> {
    // One slot per entry in GRAMMARS, indexed by pointer identity.
    static CACHE: OnceLock<Vec<Option<CompiledGrammar>>> = OnceLock::new();
    let idx = (grammar as *const Grammar as usize - GRAMMARS.as_ptr() as usize)
        / std::mem::size_of::<Grammar>();
    let cache = CACHE.get_or_init(|| {
        GRAMMARS
            .iter()
            .map(|g| {
                let language = (g.language)();
                let tags_src = tags_query_for(g.crate_name)?;
                let query = Query::new(&language, tags_src).ok()?;
                Some(CompiledGrammar { language, query })
            })
            .collect()
    });
    cache.get(idx).and_then(|opt| opt.as_ref())
}

struct Sym {
    kind: &'static str,
    name: String,
    line: usize,
}

fn db_path_for(project_root: &str) -> PathBuf {
    let base = dirs::home_dir()
        .map(|h| h.join(".local/state"))
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    base.join("ct")
        .join("symbols")
        .join(format!("{}.db", project_name(project_root)))
}

fn open_db(project_root: &str) -> Result<Connection, Box<dyn std::error::Error>> {
    let path = db_path_for(project_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(&path)?;
    // WAL cuts fsync overhead dramatically for the many tiny writes the
    // hook path does. synchronous=NORMAL is safe with WAL and ~2x faster.
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

/// Worktree root from cwd — `git rev-parse --show-toplevel` without the
/// worktree-to-main-repo resolution that `current_project()` applies.
fn worktree_root() -> String {
    Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
}

/// Map a tags.scm kind suffix (after `definition.`) to our internal label.
/// Returns None for kinds we don't index (e.g. `reference.*`, `call`).
fn classify_kind(suffix: &str) -> Option<&'static str> {
    Some(match suffix {
        "function" => "FN",
        "method" => "FN",
        "macro" => "MACRO",
        "class" => "CLASS",
        "interface" => "INTERFACE",
        "struct" => "STRUCT",
        "enum" => "ENUM",
        "trait" => "TRAIT",
        "type" => "TYPE",
        "module" => "MOD",
        "constant" | "const" => "CONST",
        _ => return None,
    })
}

fn parse_file(source: &str, grammar: &Grammar) -> Option<Vec<Sym>> {
    let compiled = compiled(grammar)?;

    let mut parser = Parser::new();
    parser.set_language(&compiled.language).ok()?;
    let tree = parser.parse(source, None)?;

    let capture_names = compiled.query.capture_names();
    let bytes = source.as_bytes();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&compiled.query, tree.root_node(), bytes);

    let mut out = Vec::new();
    while let Some(m) = matches.next() {
        let mut name: Option<String> = None;
        let mut kind: Option<&'static str> = None;
        let mut line: usize = 0;
        for cap in m.captures {
            let cap_name = capture_names[cap.index as usize];
            if cap_name == "name"
                && let Ok(text) = cap.node.utf8_text(bytes)
            {
                name = Some(text.to_string());
            } else if let Some(suffix) = cap_name.strip_prefix("definition.")
                && let Some(label) = classify_kind(suffix)
            {
                kind = Some(label);
                line = cap.node.start_position().row + 1;
            }
        }
        if let (Some(n), Some(k)) = (name, kind)
            && !n.is_empty()
        {
            out.push(Sym {
                kind: k,
                name: n,
                line,
            });
        }
    }
    Some(out)
}

fn reindex_one(
    conn: &mut Connection,
    project_root: &Path,
    abs_file: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let tx = conn.transaction()?;
    let count = reindex_in_tx(&tx, project_root, abs_file)?;
    tx.commit()?;
    Ok(count)
}

/// Per-file reindex inside an existing transaction. Bulk-index batches many
/// files under one fsync by sharing a single tx.
fn reindex_in_tx(
    tx: &Transaction,
    project_root: &Path,
    abs_file: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let Some(grammar) = grammar_for(abs_file) else {
        return Ok(0);
    };
    let rel = abs_file
        .strip_prefix(project_root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| abs_file.to_string_lossy().to_string());

    tx.execute("DELETE FROM symbols WHERE file = ?1", params![&rel])?;

    let Ok(source) = fs::read_to_string(abs_file) else {
        return Ok(0);
    };
    let Some(syms) = parse_file(&source, grammar) else {
        return Ok(0);
    };

    let mut stmt =
        tx.prepare_cached("INSERT INTO symbols (name, kind, file, line) VALUES (?1, ?2, ?3, ?4)")?;
    let mut count = 0;
    for s in syms {
        stmt.execute(params![s.name, s.kind, rel, s.line as i64])?;
        count += 1;
    }
    Ok(count)
}

pub fn cmd_reindex(file: String) -> Result<(), Box<dyn std::error::Error>> {
    let worktree = worktree_root();
    let abs = fs::canonicalize(&file).unwrap_or_else(|_| PathBuf::from(&file));
    let mut conn = open_db(&current_project())?;
    let n = reindex_one(&mut conn, Path::new(&worktree), &abs)?;
    if env::var("CT_SYM_QUIET").is_err() {
        eprintln!("indexed {n} symbols from {}", abs.display());
    }
    Ok(())
}

pub fn cmd_find(name: String, limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = open_db(&current_project())?;
    if row_count(&conn)? == 0 {
        let worktree = worktree_root();
        eprintln!("index empty; bulk-indexing {worktree}...");
        bulk_index(&mut conn, Path::new(&worktree))?;
    }
    let mut stmt = conn.prepare(
        "SELECT kind, file, line FROM symbols WHERE name = ?1 ORDER BY file, line LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![name, limit as i64], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    let mut any = false;
    for row in rows {
        let (kind, file, line) = row?;
        println!("{kind} {name} {file}:{line}");
        any = true;
    }
    if !any {
        eprintln!("no symbol named {name}");
    }
    Ok(())
}

fn row_count(conn: &Connection) -> Result<i64, Box<dyn std::error::Error>> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get(0))?;
    Ok(n)
}

pub fn cmd_bulk() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = open_db(&current_project())?;
    bulk_index(&mut conn, Path::new(&worktree_root()))?;
    // Reclaim disk from deleted rows. Must run outside any transaction.
    conn.execute_batch("VACUUM")?;
    Ok(())
}

fn bulk_index(
    conn: &mut Connection,
    project_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch("DELETE FROM symbols")?;
    let paths = enumerate_source_files(project_root);

    let tx = conn.transaction()?;
    let mut total = 0usize;
    let mut files = 0usize;
    for path in &paths {
        if let Ok(n) = reindex_in_tx(&tx, project_root, path) {
            total += n;
            files += 1;
        }
    }
    tx.commit()?;
    eprintln!("indexed {total} symbols across {files} files");
    Ok(())
}

/// Enumerate source files: prefer `git ls-files` (honors .gitignore, fast).
/// Fall back to a manual walk for non-git projects.
fn enumerate_source_files(project_root: &Path) -> Vec<PathBuf> {
    if let Some(paths) = git_ls_files(project_root) {
        return paths;
    }
    let mut out = Vec::new();
    walk_project(project_root, &mut |p| out.push(p.to_path_buf()));
    out
}

fn git_ls_files(project_root: &Path) -> Option<Vec<PathBuf>> {
    let output = Command::new("git")
        // --cached: tracked files. --others --exclude-standard: untracked-but-
        // not-gitignored (so newly-created source files are indexed without
        // requiring a `git add` first).
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(project_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let mut out = Vec::new();
    for chunk in output.stdout.split(|&b| b == 0) {
        if chunk.is_empty() {
            continue;
        }
        let Ok(rel) = std::str::from_utf8(chunk) else {
            continue;
        };
        let path = project_root.join(rel);
        if grammar_for(&path).is_some() {
            out.push(path);
        }
    }
    Some(out)
}

fn walk_project(root: &Path, visit: &mut dyn FnMut(&Path)) {
    fn rec(dir: &Path, visit: &mut dyn FnMut(&Path)) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if s.starts_with('.') || s == "target" || s == "node_modules" || s == ".build" {
                continue;
            }
            if path.is_dir() {
                if path.join(".git").is_file() {
                    continue;
                }
                rec(&path, visit);
            } else if grammar_for(&path).is_some() {
                visit(&path);
            }
        }
    }
    rec(root, visit);
}

pub fn cmd_hook() -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let payload: Value = serde_json::from_str(&buf).unwrap_or(Value::Null);
    let Some(file) = payload
        .get("tool_input")
        .and_then(|t| t.get("file_path"))
        .and_then(|f| f.as_str())
    else {
        return Ok(());
    };
    let _ = Command::new(env::current_exe()?)
        .args(["sym", "reindex", file])
        .env("CT_SYM_QUIET", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    Ok(())
}

/// Best-effort incremental reindex. Swallows errors so apply_patch never fails
/// because of indexing.
pub fn reindex_files<I, P>(files: I)
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let Ok(mut conn) = open_db(&current_project()) else {
        return;
    };
    let worktree = worktree_root();
    let root = Path::new(&worktree);
    for f in files {
        let abs = fs::canonicalize(f.as_ref()).unwrap_or_else(|_| f.as_ref().to_path_buf());
        let _ = reindex_one(&mut conn, root, &abs);
    }
    if should_prune_now() {
        let _ = prune_missing(&mut conn, root);
    }
}

pub fn cmd_prune() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = open_db(&current_project())?;
    let worktree = worktree_root();
    let removed = prune_missing(&mut conn, Path::new(&worktree))?;
    eprintln!("pruned {removed} orphan file(s)");
    Ok(())
}

fn prune_missing(
    conn: &mut Connection,
    worktree: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let files: Vec<String> = {
        let mut stmt = conn.prepare("SELECT DISTINCT file FROM symbols")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.filter_map(|r| r.ok()).collect()
    };
    let tx = conn.transaction()?;
    let mut removed = 0usize;
    for file in &files {
        if !worktree.join(file).exists() {
            tx.execute("DELETE FROM symbols WHERE file = ?1", params![file])?;
            removed += 1;
        }
    }
    tx.commit()?;
    Ok(removed)
}

/// ~2% of calls. Amortizes cleanup across edits; doesn't need uniform distribution.
fn should_prune_now() -> bool {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() % 50 == 0)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_db(path: &Path) -> Connection {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch(SCHEMA).unwrap();
        conn
    }

    #[test]
    fn rust_symbols_via_tags_query() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("src.rs");
        fs::write(
            &file,
            "pub fn alpha() {}\npub struct Beta;\npub enum Gamma { A }",
        )
        .unwrap();

        let mut conn = setup_db(&tmp.path().join("idx.db"));
        reindex_one(&mut conn, tmp.path(), &file).unwrap();

        for name in &["alpha", "Beta", "Gamma"] {
            let n: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM symbols WHERE name = ?1",
                    [name],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(n >= 1, "expected {name} in index, got {n}");
        }
    }

    #[test]
    fn python_symbols_via_tags_query() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("m.py");
        fs::write(&file, "def foo():\n    pass\n\nclass Bar:\n    pass\n").unwrap();

        let mut conn = setup_db(&tmp.path().join("idx.db"));
        reindex_one(&mut conn, tmp.path(), &file).unwrap();

        for name in &["foo", "Bar"] {
            let n: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM symbols WHERE name = ?1",
                    [name],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(n >= 1, "expected {name} in index, got {n}");
        }
    }

    #[test]
    fn reindex_replaces_previous_entries() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("src.rs");
        fs::write(&file, "pub fn gone() {}").unwrap();

        let mut conn = setup_db(&tmp.path().join("idx.db"));
        reindex_one(&mut conn, tmp.path(), &file).unwrap();

        fs::write(&file, "pub fn kept() {}").unwrap();
        reindex_one(&mut conn, tmp.path(), &file).unwrap();

        let gone: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM symbols WHERE name = 'gone'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let kept: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM symbols WHERE name = 'kept'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(gone, 0);
        assert_eq!(kept, 1);
    }
}
