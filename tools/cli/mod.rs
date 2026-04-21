use clap::{Parser, Subcommand};

mod args;
mod artifact;
mod tool;

pub use args::{ApplyPatchCmd, ArtifactAction, McpAction, ToolAction, VaultAction};
pub use artifact::{
    ArtifactCreateArgs, run_artifact_archive, run_artifact_comments, run_artifact_create,
    run_artifact_latest, run_artifact_list, run_artifact_prune, run_artifact_read,
    run_artifact_rename, run_artifact_retag, run_artifact_show,
};
pub use tool::{
    run_apply_patch, run_apply_patch_prune, run_apply_patch_stats, run_cochanges, run_completion,
    run_slug,
};

#[derive(Parser)]
#[command(name = "ct")]
#[command(about = "Claude Tool CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(visible_alias = "p", about = "Plan file operations")]
    Plan {
        #[command(subcommand)]
        action: ArtifactAction,
    },

    #[command(visible_alias = "s", about = "Spec file operations")]
    Spec {
        #[command(subcommand)]
        action: ArtifactAction,
    },

    #[command(visible_alias = "r", about = "Review file operations")]
    Review {
        #[command(subcommand)]
        action: ArtifactAction,
    },

    #[command(visible_alias = "rp", about = "Report file operations")]
    Report {
        #[command(subcommand)]
        action: ArtifactAction,
    },

    #[command(visible_alias = "d", about = "Doc file operations")]
    Doc {
        #[command(subcommand)]
        action: ArtifactAction,
    },

    #[command(visible_alias = "v", about = "Vault repository management")]
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },

    #[command(about = "Read artifact by stem (resolves across all types)")]
    Read {
        #[arg(help = "File path or stem")]
        file: String,

        #[arg(long, help = "Output frontmatter as JSON")]
        frontmatter: bool,
    },

    #[command(visible_alias = "n", about = "Handle notification hooks")]
    Notify,

    #[command(visible_alias = "o", about = "Utility tools")]
    Tool {
        #[command(subcommand)]
        action: ToolAction,
    },

    #[command(about = "Code indexing and symbol discovery")]
    Sym(sym::cli::SymArgs),

    #[command(about = "Run the MCP stdio server")]
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },

    #[command(about = "Inspect or prune apply_patch telemetry")]
    ApplyPatch {
        #[command(subcommand)]
        cmd: ApplyPatchCmd,
    },
}

pub(crate) fn handle_sync_error(e: crate::artifact::SyncError) -> ! {
    eprintln!("{e}");
    match e {
        crate::artifact::SyncError::Push(_) => std::process::exit(2),
        _ => std::process::exit(1),
    }
}

pub(crate) fn truncate_at_char_boundary(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut idx = max_bytes;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    &s[..idx]
}
