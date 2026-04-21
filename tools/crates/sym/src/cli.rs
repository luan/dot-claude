use std::path::{Path, PathBuf};

use anyhow::bail;
use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;
use serde_json::json;

use crate::context;
use crate::diff;
use crate::graph;
use crate::hook;
use crate::impls;
use crate::indexer;
use crate::investigate;
use crate::ls;
use crate::multisym;
use crate::outline;
use crate::output;
use crate::search;
use crate::show;
use crate::structure;
use crate::version;

#[derive(Args, Debug)]
pub struct SymArgs {
    #[arg(short = 'd', long)]
    pub db: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Serialize)]
struct TargetResult<T> {
    target: String,
    results: T,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "Index a directory for symbol discovery")]
    Index {
        #[arg(default_value = ".")]
        path: PathBuf,

        #[arg(short, long, default_value_t = 0)]
        workers: usize,

        #[arg(short, long, default_value_t = false)]
        force: bool,

        #[arg(long, default_value_t = false)]
        reset: bool,

        #[arg(long = "ignore")]
        ignore: Vec<String>,
    },

    #[command(about = "Search symbols or text across files")]
    Search {
        query: Vec<String>,

        #[arg(short = 't', long, default_value_t = false)]
        text: bool,

        #[arg(short = 'n', long, default_value_t = 20)]
        limit: usize,

        #[arg(short = 'k', long)]
        kind: Option<String>,

        #[arg(short = 'l', long)]
        lang: Option<String>,

        #[arg(short = 'e', long, default_value_t = false)]
        exact: bool,

        #[arg(short = 'i', long = "ignore-case", default_value_t = false)]
        ignore_case: bool,

        #[arg(long = "path")]
        path_filters: Vec<String>,

        #[arg(long = "exclude")]
        excludes: Vec<String>,
    },

    #[command(about = "Show symbols defined in a file")]
    Outline {
        file: PathBuf,

        #[arg(short = 's', long, default_value_t = false)]
        signatures: bool,

        #[arg(long, default_value_t = false)]
        names: bool,
    },

    #[command(about = "Read source by symbol name or file path")]
    Show {
        targets: Vec<String>,

        #[arg(short = 'C', long, default_value_t = 0)]
        context: usize,

        #[arg(long, default_value_t = false)]
        all: bool,

        #[arg(long, default_value_t = false)]
        stdin: bool,
    },

    #[command(about = "Show file tree, repo list, or repo stats")]
    Ls {
        path: Option<PathBuf>,

        #[arg(long, default_value_t = false)]
        repos: bool,

        #[arg(long, default_value_t = false)]
        stats: bool,

        #[arg(short = 'D', long = "depth", default_value_t = 0)]
        depth: usize,
    },

    #[command(about = "Find references to a symbol")]
    Refs {
        targets: Vec<String>,

        #[arg(long, default_value_t = false)]
        importers: bool,

        #[arg(long, default_value_t = false)]
        impact: bool,

        #[arg(short = 'D', long = "depth", default_value_t = 1)]
        depth: usize,

        #[arg(short = 'n', long, default_value_t = 20)]
        limit: usize,

        #[arg(long = "path")]
        path_filters: Vec<String>,

        #[arg(long = "exclude")]
        excludes: Vec<String>,

        #[arg(long, default_value_t = false)]
        stdin: bool,
    },

    #[command(about = "Find files that import a given file or package")]
    Importers {
        target: String,

        #[arg(short = 'D', long = "depth", default_value_t = 1)]
        depth: usize,

        #[arg(short = 'n', long, default_value_t = 50)]
        limit: usize,
    },

    #[command(about = "Find transitive callers of a symbol")]
    Impact {
        targets: Vec<String>,

        #[arg(short = 'D', long = "depth", default_value_t = 2)]
        depth: usize,

        #[arg(short = 'n', long, default_value_t = 50)]
        limit: usize,

        #[arg(long, default_value_t = false)]
        stdin: bool,
    },

    #[command(about = "Follow the call graph downward from a symbol")]
    Trace {
        targets: Vec<String>,

        #[arg(long, default_value_t = 3)]
        depth: usize,

        #[arg(short = 'n', long, default_value_t = 50)]
        limit: usize,

        #[arg(long, default_value = "call")]
        kinds: String,

        #[arg(long, default_value_t = false)]
        stdin: bool,
    },

    #[command(about = "Find types that implement a symbol or what a type implements")]
    Impls {
        targets: Vec<String>,

        #[arg(short = 'l', long)]
        lang: Option<String>,

        #[arg(short = 'n', long, default_value_t = 50)]
        limit: usize,

        #[arg(long = "path")]
        path_filters: Vec<String>,

        #[arg(long = "exclude")]
        excludes: Vec<String>,

        #[arg(long = "of")]
        of: Option<String>,

        #[arg(long, default_value_t = false)]
        resolved: bool,

        #[arg(long, default_value_t = false)]
        unresolved: bool,

        #[arg(long, default_value_t = false)]
        stdin: bool,
    },

    #[command(about = "Bundled context: source, callers, conformance, and file imports")]
    Context {
        targets: Vec<String>,

        #[arg(short = 'n', long, default_value_t = 20)]
        callers: usize,

        #[arg(long, default_value_t = false)]
        stdin: bool,
    },

    #[command(about = "Kind-adaptive investigation for symbols")]
    Investigate {
        targets: Vec<String>,

        #[arg(long, default_value_t = false)]
        stdin: bool,
    },

    #[command(about = "Structural overview of the indexed codebase")]
    Structure {
        #[arg(short = 'n', long, default_value_t = 10)]
        limit: usize,
    },

    #[command(about = "Show git diff scoped to a symbol's definition")]
    Diff {
        target: String,

        #[arg(default_value = "HEAD")]
        base: String,

        #[arg(long, default_value_t = false)]
        stat: bool,
    },

    #[command(about = "Agent-integration hooks (nudge, remind, install)")]
    Hook {
        #[command(subcommand)]
        command: HookCommand,
    },

    #[command(about = "Print sym version information")]
    Version,
}

#[derive(Subcommand, Debug)]
pub enum HookCommand {
    #[command(about = "Suggest a sym equivalent when an agent is about to grep")]
    Nudge {
        #[arg(long, default_value = "claude-code")]
        format: String,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },

    #[command(about = "Print a short reminder block an agent can inject as context")]
    Remind {
        #[arg(long, default_value = "text")]
        format: String,
    },

    #[command(about = "Install sym hooks into the given agent")]
    Install {
        #[arg(value_enum)]
        agent: HookAgent,

        #[arg(long, default_value = "user")]
        scope: String,

        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    #[command(about = "Remove sym hooks from the given agent")]
    Uninstall {
        #[arg(value_enum)]
        agent: HookAgent,

        #[arg(long, default_value = "user")]
        scope: String,

        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum HookAgent {
    #[value(name = "claude-code")]
    ClaudeCode,
}

impl HookAgent {
    pub fn as_str(self) -> &'static str {
        match self {
            HookAgent::ClaudeCode => "claude-code",
        }
    }
}

pub fn run_index(path: &Path, force: bool, reset: bool, ignore: &[String]) -> anyhow::Result<()> {
    let root = path.canonicalize()?;
    let db_path = crate::repo::configured_db_path(&root, None)?;
    if reset {
        indexer::reset_db(&db_path)?;
    }
    let stats = indexer::index(
        &root,
        &indexer::IndexOptions {
            db_path: Some(db_path),
            cli_ignore_patterns: ignore.to_vec(),
            force,
            ..indexer::IndexOptions::default()
        },
    )?;

    if output::json_enabled() {
        return output::write_json(&stats);
    }

    println!(
        "Indexed {} parseable files in {}",
        stats.files_indexed,
        root.display()
    );
    if !stats.ignore_patterns.is_empty() {
        println!("Ignore patterns: {}", stats.ignore_patterns.join(", "));
    }

    Ok(())
}

pub fn run_search(
    query: &[String],
    text: bool,
    limit: usize,
    kind: Option<&str>,
    lang: Option<&str>,
    exact: bool,
    ignore_case: bool,
    path_filters: &[String],
    excludes: &[String],
) -> anyhow::Result<()> {
    let query = query.join(" ");
    if query.trim().is_empty() {
        bail!("search query cannot be empty");
    }

    let effective_exact = search::normalize_search_mode(exact, ignore_case, text)?;
    let root = std::env::current_dir()?;
    if text {
        let results = search::search_text(
            &root,
            &query,
            lang,
            limit,
            path_filters,
            excludes,
            effective_exact,
        )?;
        if results.is_empty() {
            bail!("no results found for '{query}'");
        }

        let mut content = String::new();
        for result in &results {
            content.push_str(&format!("{}:{}: {}\n", result.rel_path.display(), result.line, result.snippet));
        }

        return output::render(
            &results,
            &[("query", query.clone()), ("result_count", results.len().to_string())],
            &content,
        );
    }

    let results = search::search_symbols(
        &root,
        &query,
        kind,
        lang,
        limit,
        effective_exact,
        ignore_case,
        path_filters,
        excludes,
    )?;
    if results.is_empty() {
        bail!("no results found for '{query}'");
    }

    let mut content = String::new();
    for result in &results {
        content.push_str(&format!("{} {} {}:{}\n", result.kind, result.name, result.rel_path, result.start_line));
    }

    output::render(
        &results,
        &[("query", query), ("result_count", results.len().to_string())],
        &content,
    )
}

pub fn run_outline(file: &Path, signatures: bool, names: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let symbols = outline::file_outline(&cwd, file)?;
    if names {
        let mut seen = std::collections::BTreeSet::new();
        let mut names_out = Vec::new();
        for symbol in symbols {
            if seen.insert(symbol.name.clone()) {
                names_out.push(symbol.name);
            }
        }
        if output::json_enabled() {
            return output::write_json(&names_out);
        }
        for name in names_out {
            println!("{name}");
        }
        return Ok(());
    }

    if output::json_enabled() {
        return output::write_json(&symbols);
    }

    let rel_file = file.display().to_string();
    let mut content = String::new();
    for symbol in &symbols {
        let indent = "  ".repeat(symbol.depth);
        if signatures && !symbol.signature.is_empty() {
            content.push_str(&format!(
                "{}{} {}{} (L{}-{})",
                indent, symbol.kind, symbol.name, symbol.signature, symbol.start_line, symbol.end_line
            ));
        } else {
            content.push_str(&format!(
                "{}{} {} (L{}-{})",
                indent, symbol.kind, symbol.name, symbol.start_line, symbol.end_line
            ));
        }
        content.push('\n');
    }

    output::write_frontmatter(
        &[("file", rel_file), ("symbol_count", symbols.len().to_string())],
        &content,
    )
}

pub fn run_show(targets: &[String], context: usize, all: bool, stdin: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let targets = multisym::collect_symbols(targets, stdin)?;

    if output::json_enabled() {
        let mut rendered = Vec::new();
        for target in targets {
            if looks_like_file_target(&target) {
                let (path, range) = parse_file_target(&target);
                let lines = show::show_file(Path::new(&path), range, context)?;
                rendered.push(json!({
                    "target": target,
                    "kind": "file",
                    "results": lines,
                }));
            } else {
                let shown = show::show_symbol(&cwd, &target, context, all)?;
                rendered.push(json!({
                    "target": target,
                    "kind": "symbol",
                    "results": shown,
                }));
            }
        }
        return output::write_json(&rendered);
    }

    for (index, target) in targets.iter().enumerate() {
        print!("{}", multisym::multi_symbol_header(target, index == 0));
        if looks_like_file_target(target) {
            let (path, range) = parse_file_target(target);
            let lines = show::show_file(Path::new(&path), range, context)?;
            for line in lines {
                println!("{}", line.content);
            }
            continue;
        }

        for shown in show::show_symbol(&cwd, target, context, all)? {
            print!("{}", shown.content);
            if !shown.content.ends_with('\n') {
                println!();
            }
        }
    }

    Ok(())
}

pub fn run_ls(path: Option<&Path>, repos: bool, stats: bool, depth: usize) -> anyhow::Result<()> {
    if repos {
        let repos = ls::list_repos()?;
        if output::json_enabled() {
            return output::write_json(&repos);
        }
        if repos.is_empty() {
            return Ok(());
        }
        for repo in repos {
            println!(
                "{}  {} files  {} symbols",
                repo.path, repo.file_count, repo.symbol_count
            );
        }
        return Ok(());
    }

    let cwd = std::env::current_dir()?;
    if stats {
        let stats = ls::repo_stats(&cwd)?;
        let mut content = String::new();
        for (language, count) in &stats.languages {
            content.push_str(&format!("{language}: {count} files\n"));
        }
        return output::render(
            &stats,
            &[
                ("repo", stats.path.clone()),
                ("files", stats.file_count.to_string()),
                ("symbols", stats.symbol_count.to_string()),
            ],
            &content,
        );
    }

    let path = path.unwrap_or_else(|| Path::new("."));
    let tree = ls::tree(path, depth)?;
    if output::json_enabled() {
        return output::write_json(&tree);
    }
    print!("{}", crate::walker::print_tree(&tree));
    Ok(())
}

pub fn run_refs(
    targets: &[String],
    importers: bool,
    impact: bool,
    depth: usize,
    limit: usize,
    path_filters: &[String],
    excludes: &[String],
    stdin: bool,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let targets = multisym::collect_symbols(targets, stdin)?;

    if impact {
        return run_impact(&targets, depth.max(2), limit, false);
    }

    if output::json_enabled() {
        let mut grouped = Vec::new();
        for target in targets {
            if importers {
                let results = graph::find_importers(&cwd, &target, depth, limit, path_filters, excludes)?;
                grouped.push(TargetResult { target, results: json!(results) });
            } else {
                let results = graph::find_references(&cwd, &target, limit, path_filters, excludes)?;
                grouped.push(TargetResult { target, results: json!(results) });
            }
        }
        return output::write_json(&grouped);
    }

    for (index, target) in targets.iter().enumerate() {
        if index > 0 {
            println!();
        }

        if importers {
            let results = graph::find_importers(&cwd, target, depth, limit, path_filters, excludes)?;
            if results.is_empty() {
                println!("No importers found for '{target}'.");
                continue;
            }
            let mut content = String::new();
            for result in &results {
                content.push_str(&format!("{}:{}\n", result.rel_path, result.import));
            }
            output::write_frontmatter(
                &[("symbol", target.clone()), ("importer_count", results.len().to_string())],
                &content,
            )?;
            continue;
        }

        let results = graph::find_references(&cwd, target, limit, path_filters, excludes)?;
        if results.is_empty() {
            println!("No references found for '{target}'.");
            continue;
        }
        let mut content = String::new();
        for result in &results {
            content.push_str(&format!("{}:{}\n", result.rel_path, result.line));
        }
        output::write_frontmatter(
            &[("symbol", target.clone()), ("ref_count", results.len().to_string())],
            &content,
        )?;
    }

    Ok(())
}

pub fn run_importers(target: &str, depth: usize, limit: usize) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let results = graph::find_importers_by_path(&cwd, target, depth, limit)?;
    if results.is_empty() {
        bail!("no importers found for '{target}'");
    }
    if output::json_enabled() {
        return output::write_json(&results);
    }
    let mut content = String::new();
    for result in &results {
        content.push_str(&format!("{}:{}\n", result.rel_path, result.import));
    }
    output::write_frontmatter(
        &[("target", target.to_string()), ("importer_count", results.len().to_string())],
        &content,
    )
}

pub fn run_impact(targets: &[String], depth: usize, limit: usize, stdin: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let targets = multisym::collect_symbols(targets, stdin)?;

    let mut merged: Vec<crate::store::ImpactResult> = Vec::new();
    let mut source_map = std::collections::BTreeMap::<String, Vec<String>>::new();
    let mut seen = std::collections::BTreeMap::<String, usize>::new();

    for target in targets {
        for row in graph::find_impact(&cwd, &target, depth, limit)? {
            let key = format!("{}:{}|{}", row.file, row.line, row.caller);
            if let Some(index) = seen.get(&key).copied() {
                if row.depth < merged[index].depth {
                    merged[index] = row.clone();
                }
            } else {
                seen.insert(key.clone(), merged.len());
                merged.push(row.clone());
            }
            let hits = source_map.entry(key).or_default();
            if !hits.contains(&target) {
                hits.push(target.clone());
            }
        }
    }

    if merged.is_empty() {
        bail!("no callers found");
    }

    if output::json_enabled() {
        let results = merged
            .into_iter()
            .map(|row| {
                let key = format!("{}:{}|{}", row.file, row.line, row.caller);
                let hits = source_map.get(&key).cloned().unwrap_or_default();
                json!({
                    "depth": row.depth,
                    "caller": row.caller,
                    "symbol": row.symbol,
                    "file": row.file,
                    "rel_path": row.rel_path,
                    "line": row.line,
                    "hit_symbols": hits,
                })
            })
            .collect::<Vec<_>>();
        return output::write_json(&results);
    }

    for row in merged {
        let key = format!("{}:{}|{}", row.file, row.line, row.caller);
        let hits = source_map.get(&key).cloned().unwrap_or_default();
        if hits.is_empty() {
            println!("[{}] {} <- {}:{}", row.depth, row.caller, row.rel_path, row.line);
        } else {
            println!(
                "[{}] {} <- {}:{} [{}]",
                row.depth,
                row.caller,
                row.rel_path,
                row.line,
                hits.join(",")
            );
        }
    }

    Ok(())
}

pub fn run_trace(
    targets: &[String],
    depth: usize,
    limit: usize,
    kinds: &str,
    stdin: bool,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let targets = multisym::collect_symbols(targets, stdin)?;

    let kinds = parse_kinds(kinds);
    let mut merged: Vec<crate::store::TraceResult> = Vec::new();
    let mut source_map = std::collections::BTreeMap::<String, Vec<String>>::new();
    let mut seen = std::collections::BTreeMap::<String, usize>::new();

    for target in targets {
        for row in graph::find_trace(&cwd, strip_symbol_hint(&target), depth, limit, &kinds)? {
            let key = format!("{}:{}|{}", row.file, row.line, row.callee);
            if let Some(index) = seen.get(&key).copied() {
                if row.depth < merged[index].depth {
                    merged[index] = row.clone();
                }
            } else {
                seen.insert(key.clone(), merged.len());
                merged.push(row.clone());
            }
            let hits = source_map.entry(key).or_default();
            if !hits.contains(&target) {
                hits.push(target.clone());
            }
        }
    }

    if merged.is_empty() {
        bail!("no outgoing calls found");
    }

    if output::json_enabled() {
        let results = merged
            .into_iter()
            .map(|row| {
                let key = format!("{}:{}|{}", row.file, row.line, row.callee);
                let hits = source_map.get(&key).cloned().unwrap_or_default();
                json!({
                    "depth": row.depth,
                    "caller": row.caller,
                    "callee": row.callee,
                    "file": row.file,
                    "rel_path": row.rel_path,
                    "line": row.line,
                    "hit_symbols": hits,
                })
            })
            .collect::<Vec<_>>();
        return output::write_json(&results);
    }

    for row in merged {
        let key = format!("{}:{}|{}", row.file, row.line, row.callee);
        let hits = source_map.get(&key).cloned().unwrap_or_default();
        if hits.is_empty() {
            println!("[{}] {} -> {} {}:{}", row.depth, row.caller, row.callee, row.rel_path, row.line);
        } else {
            println!(
                "[{}] {} -> {} {}:{} [{}]",
                row.depth,
                row.caller,
                row.callee,
                row.rel_path,
                row.line,
                hits.join(",")
            );
        }
    }

    Ok(())
}

pub fn run_impls(
    targets: &[String],
    lang: Option<&str>,
    limit: usize,
    path_filters: &[String],
    excludes: &[String],
    of: Option<&str>,
    resolved: bool,
    unresolved: bool,
    stdin: bool,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    if let Some(of) = of {
        if !targets.is_empty() || stdin {
            bail!("pass either positional symbols or --of <type>, not both");
        }
        let results = impls::find_implements(
            &cwd,
            of,
            lang,
            limit,
            path_filters,
            excludes,
            resolved,
            unresolved,
        )?;
        if results.is_empty() {
            println!("No implements edges found for '{of}'.");
            return Ok(());
        }
        let mut content = String::new();
        for result in &results {
            let tag = if result.resolved { "" } else { " (external)" };
            content.push_str(&format!("{} {}:{}{}\n", result.target, result.rel_path, result.line, tag));
        }
        return output::render(
            &results,
            &[
                ("symbol", of.to_string()),
                ("direction", "implements (outgoing)".to_string()),
                ("edges", results.len().to_string()),
            ],
            &content,
        );
    }

    let targets = multisym::collect_symbols(targets, stdin)?;

    if output::json_enabled() {
        let mut grouped = Vec::new();
        for target in targets {
            let results = impls::find_implementors(
                &cwd,
                &target,
                lang,
                limit,
                path_filters,
                excludes,
                resolved,
                unresolved,
            )?;
            grouped.push(TargetResult { target, results });
        }
        return output::write_json(&grouped);
    }

    for (index, target) in targets.iter().enumerate() {
        print!("{}", multisym::multi_symbol_header(target, index == 0));
        let results = impls::find_implementors(
            &cwd,
            target,
            lang,
            limit,
            path_filters,
            excludes,
            resolved,
            unresolved,
        )?;
        if results.is_empty() {
            println!("No implementors found for '{target}'.");
            continue;
        }
        let mut content = String::new();
        for result in &results {
            let tag = if result.resolved { "" } else { " (external)" };
            content.push_str(&format!("{} {}:{}{}\n", result.implementer, result.rel_path, result.line, tag));
        }
        output::write_frontmatter(
            &[
                ("symbol", target.clone()),
                ("direction", "implementors (incoming)".to_string()),
                ("implementor_count", results.len().to_string()),
            ],
            &content,
        )?;
    }

    Ok(())
}

pub fn run_context(targets: &[String], callers: usize, stdin: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let targets = multisym::collect_symbols(targets, stdin)?;

    if output::json_enabled() {
        let mut grouped = Vec::new();
        for target in targets {
            let result = context::symbol_context(&cwd, &target, callers)?;
            grouped.push(TargetResult { target, results: result });
        }
        return output::write_json(&grouped);
    }

    for (index, target) in targets.iter().enumerate() {
        print!("{}", multisym::multi_symbol_header(target, index == 0));
        let result = context::symbol_context(&cwd, target, callers)?;
        print!("# Source\n{}", result.source);

        if !result.callers.is_empty() {
            println!("\n# Callers ({})", result.callers.len());
            for caller in result.callers {
                println!("{}:{}", caller.rel_path, caller.line);
            }
        }
        if !result.implementors.is_empty() {
            println!("\n# Implementors ({})", result.implementors.len());
            for implementor in result.implementors {
                let tag = if implementor.resolved { "" } else { " (external)" };
                println!("{} {}:{}{}", implementor.implementer, implementor.rel_path, implementor.line, tag);
            }
        }
        if !result.implements.is_empty() {
            println!("\n# Implements ({})", result.implements.len());
            for edge in result.implements {
                let tag = if edge.resolved { "" } else { " (external)" };
                println!("{} {}:{}{}", edge.target, edge.rel_path, edge.line, tag);
            }
        }
        if !result.file_imports.is_empty() {
            println!("\n# Imports");
            for import in result.file_imports {
                println!("{import}");
            }
        }
    }

    Ok(())
}

pub fn run_investigate(targets: &[String], stdin: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let targets = multisym::collect_symbols(targets, stdin)?;

    if output::json_enabled() {
        let mut grouped = Vec::new();
        for target in targets {
            let result = investigate::investigate(&cwd, &target)?;
            grouped.push(TargetResult { target, results: result });
        }
        return output::write_json(&grouped);
    }

    for (index, target) in targets.iter().enumerate() {
        print!("{}", multisym::multi_symbol_header(target, index == 0));
        let result = investigate::investigate(&cwd, target)?;
        print!("# Source\n{}", result.source);

        if !result.members.is_empty() {
            println!("\n# Members ({})", result.members.len());
            for member in result.members {
                if member.signature.is_empty() {
                    println!("{} {} {}:{}", member.kind, member.name, member.rel_path, member.start_line);
                } else {
                    println!(
                        "{} {} {} {}:{}",
                        member.kind, member.name, member.signature, member.rel_path, member.start_line
                    );
                }
            }
        }

        if !result.refs.is_empty() {
            let label = if result.kind == "function" { "Callers" } else { "References" };
            println!("\n# {label} ({})", result.refs.len());
            for reference in result.refs {
                println!("{}:{}", reference.rel_path, reference.line);
            }
        }

        if !result.impact.is_empty() {
            println!("\n# Impact (depth 2)");
            for edge in result.impact {
                println!("[{}] {} -> {} {}:{}", edge.depth, edge.caller, edge.symbol, edge.rel_path, edge.line);
            }
        }

        if !result.implementors.is_empty() {
            println!("\n# Implementors ({})", result.implementors.len());
            for implementor in result.implementors {
                let tag = if implementor.resolved { "" } else { " (external)" };
                println!("{} {}:{}{}", implementor.implementer, implementor.rel_path, implementor.line, tag);
            }
        }

        if !result.implements.is_empty() {
            println!("\n# Implements ({})", result.implements.len());
            for edge in result.implements {
                let tag = if edge.resolved { "" } else { " (external)" };
                println!("{} {}:{}{}", edge.target, edge.rel_path, edge.line, tag);
            }
        }
    }

    Ok(())
}

pub fn run_structure(limit: usize) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let result = structure::analyze(&cwd, limit)?;

    if output::json_enabled() {
        return output::write_json(&result);
    }

    println!(
        "--- {} ({} files, {} symbols) ---",
        result.repo_root, result.files, result.symbols
    );
    println!();

    if !result.entry_points.is_empty() {
        println!("Entry points:");
        for symbol in result.entry_points {
            println!("  {} {} {}:{}", symbol.kind, symbol.name, symbol.rel_path, symbol.start_line);
        }
        println!();
    }

    if !result.top_by_refs.is_empty() {
        println!("Most referenced symbols:");
        for symbol in result.top_by_refs {
            println!(
                "  {} {} ({} refs) {}:{}",
                symbol.symbol.kind,
                symbol.symbol.name,
                symbol.count,
                symbol.symbol.rel_path,
                symbol.symbol.start_line
            );
        }
        println!();
    }

    if !result.top_packages.is_empty() {
        println!("Largest packages:");
        for package in result.top_packages {
            println!("  {} {} symbols, {} files", package.path, package.symbols, package.files);
        }
        println!();
    }

    if !result.top_by_import_fan.is_empty() {
        println!("Most imported files:");
        for file in result.top_by_import_fan {
            println!("  {} imported by {} files", file.rel_path, file.count);
        }
    }

    Ok(())
}

pub fn run_diff(target: &str, base: &str, stat: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let result = diff::symbol_diff(&cwd, target, base, stat)?;
    if output::json_enabled() {
        return output::write_json(&result);
    }
    if result.content.is_empty() {
        eprintln!(
            "No diff for {} ({}:{}-{}) against {}",
            result.symbol.name,
            result.symbol.rel_path,
            result.symbol.start_line,
            result.symbol.end_line,
            result.base
        );
        return Ok(());
    }
    print!("{}", result.content);
    Ok(())
}

pub fn run_hook(command: HookCommand) -> anyhow::Result<()> {
    match command {
        HookCommand::Nudge { format, command } => {
            let (fields, tool_name) = hook::read_nudge_input(&command)?;
            let field_refs = fields.iter().map(String::as_str).collect::<Vec<_>>();
            let suggestion = hook::detect_search_command(&field_refs, &tool_name);
            let output = hook::emit_nudge(&format, &field_refs, &suggestion)?;
            if !output.stdout.is_empty() {
                print!("{}", output.stdout);
            }
            if !output.stderr.is_empty() {
                eprint!("{}", output.stderr);
            }
            Ok(())
        }
        HookCommand::Remind { format } => {
            print!("{}", hook::emit_remind(&format)?);
            Ok(())
        }
        HookCommand::Install {
            agent,
            scope,
            dry_run,
        } => {
            print!(
                "{}",
                hook::run_hook_install(agent.as_str(), &scope, dry_run, false)?
            );
            Ok(())
        }
        HookCommand::Uninstall {
            agent,
            scope,
            dry_run,
        } => {
            print!(
                "{}",
                hook::run_hook_install(agent.as_str(), &scope, dry_run, true)?
            );
            Ok(())
        }
    }
}

pub fn run_version() -> anyhow::Result<()> {
    if output::json_enabled() {
        return output::write_json(&version::display_version());
    }
    println!("{}", version::display_version());
    Ok(())
}

fn parse_kinds(raw: &str) -> Vec<&str> {
    raw.split(',')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect()
}

fn strip_symbol_hint(target: &str) -> &str {
    target.rsplit_once(':').map(|(_, symbol)| symbol).unwrap_or(target)
}

fn looks_like_file_target(target: &str) -> bool {
    if let Some((path, suffix)) = target.rsplit_once(':') {
        if is_line_range(suffix) {
            return true;
        }
        if looks_like_plain_file(path) {
            return false;
        }
    }

    looks_like_plain_file(target)
}

fn parse_file_target(target: &str) -> (String, Option<(usize, usize)>) {
    let Some((path, range)) = target.rsplit_once(':') else {
        return (target.to_string(), None);
    };
    let Some((start, end)) = range.split_once('-') else {
        return (target.to_string(), None);
    };
    let Ok(start) = start.trim_start_matches('L').parse::<usize>() else {
        return (target.to_string(), None);
    };
    let Ok(end) = end.trim_start_matches('L').parse::<usize>() else {
        return (target.to_string(), None);
    };
    (path.to_string(), Some((start, end)))
}

fn looks_like_plain_file(target: &str) -> bool {
    target.contains('/') || crate::lang::language_for_file(Path::new(target)).is_some()
}

fn is_line_range(value: &str) -> bool {
    let Some((start, end)) = value.split_once('-') else {
        return false;
    };
    start.trim_start_matches('L').parse::<usize>().is_ok()
        && end.trim_start_matches('L').parse::<usize>().is_ok()
}
