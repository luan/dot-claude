pub mod cli;
pub mod context;
pub mod diff;
pub mod graph;
pub mod hook;
pub mod impls;
pub mod indexer;
pub mod investigate;
pub mod lang;
pub mod ls;
pub mod multisym;
pub mod outline;
pub mod output;
pub mod parser;
pub mod pathfilters;
pub mod repo;
pub mod resolve;
pub mod search;
pub mod show;
pub mod store;
pub mod structure;
pub mod symbols;
pub mod version;
pub mod walker;

pub fn run(args: cli::SymArgs) -> anyhow::Result<()> {
    output::set_json_enabled(args.json);

    if let Some(db) = &args.db {
        // CLI entrypoint runs before any parallel work begins, so this process-wide
        // override is safe and gives --db precedence over inherited SYM_DB.
        unsafe {
            std::env::set_var("SYM_DB", db);
        }
    }

    match args.command {
        cli::Command::Index {
            path,
            workers: _,
            force,
            reset,
            ignore,
        } => cli::run_index(&path, force, reset, &ignore),
        cli::Command::Search {
            query,
            text,
            limit,
            lang,
            exact,
            ignore_case,
            path_filters,
            excludes,
        } => cli::run_search(
            &query,
            text,
            limit,
            lang.as_deref(),
            exact,
            ignore_case,
            &path_filters,
            &excludes,
        ),
        cli::Command::Outline {
            file,
            signatures,
            names,
        } => cli::run_outline(&file, signatures, names),
        cli::Command::Show {
            targets,
            context,
            all,
            stdin,
        } => cli::run_show(&targets, context, all, stdin),
        cli::Command::Ls {
            path,
            repos,
            stats,
            depth,
        } => cli::run_ls(path.as_deref(), repos, stats, depth),
        cli::Command::Refs {
            targets,
            importers,
            impact,
            depth,
            limit,
            path_filters,
            excludes,
            stdin,
        } => cli::run_refs(
            &targets,
            importers,
            impact,
            depth,
            limit,
            &path_filters,
            &excludes,
            stdin,
        ),
        cli::Command::Importers {
            target,
            depth,
            limit,
        } => cli::run_importers(&target, depth, limit),
        cli::Command::Impact {
            targets,
            depth,
            limit,
            stdin,
        } => cli::run_impact(&targets, depth, limit, stdin),
        cli::Command::Trace {
            targets,
            depth,
            limit,
            kinds,
            stdin,
        } => cli::run_trace(&targets, depth, limit, &kinds, stdin),
        cli::Command::Impls {
            targets,
            lang,
            limit,
            path_filters,
            excludes,
            of,
            resolved,
            unresolved,
            stdin,
        } => cli::run_impls(
            &targets,
            lang.as_deref(),
            limit,
            &path_filters,
            &excludes,
            of.as_deref(),
            resolved,
            unresolved,
            stdin,
        ),
        cli::Command::Context {
            targets,
            callers,
            stdin,
        } => cli::run_context(&targets, callers, stdin),
        cli::Command::Investigate { targets, stdin } => cli::run_investigate(&targets, stdin),
        cli::Command::Structure { limit } => cli::run_structure(limit),
        cli::Command::Diff { target, base, stat } => cli::run_diff(&target, &base, stat),
        cli::Command::Hook { command } => cli::run_hook(command),
        cli::Command::Version => cli::run_version(),
    }
}
