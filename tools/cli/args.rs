use clap::Subcommand;
use clap_complete::Shell;

#[derive(Subcommand)]
pub enum ToolAction {
    #[command(about = "Generate URL-safe slug from text")]
    Slug {
        #[arg(
            help = "Words to slugify",
            trailing_var_arg = true,
            allow_hyphen_values = true
        )]
        words: Vec<String>,
    },

    #[command(about = "Parse phase markers from plan file")]
    Phases {
        #[arg(help = "Plan file to parse (or stdin if omitted)")]
        file: Option<String>,
    },

    #[command(about = "Generate shell completion scripts")]
    Completion {
        #[arg(help = "Shell type (bash, zsh, fish, powershell, elvish)")]
        shell: Shell,
    },

    #[command(about = "Gather branch context (diff, log, files) for skills")]
    Gitcontext {
        #[arg(long, default_value = "main", help = "Base branch for comparison")]
        base: String,

        #[arg(long, default_value = "text", help = "Output format: text or json", value_parser = ["text", "json"])]
        format: String,

        #[arg(
            long,
            default_value_t = 3000,
            help = "Max total diff lines before truncation"
        )]
        max_total: usize,

        #[arg(long, default_value_t = 200, help = "Per-file diff line threshold")]
        max_file: usize,

        #[arg(long, help = "Output diff --stat instead of full diff")]
        stat: bool,

        #[arg(long, help = "Include co-change candidates in output")]
        cochanges: bool,
    },

    #[command(about = "Check doc references against project filesystem")]
    CheckRefs {
        #[arg(help = "Doc file path or stem")]
        file: String,

        #[arg(
            long,
            help = "Project root path (default: git rev-parse --show-toplevel)"
        )]
        project_root: Option<String>,
    },

    #[command(about = "Find files frequently changed together with current changes")]
    Cochanges {
        #[arg(
            long,
            default_value = "main",
            help = "Base branch/ref for changed-file detection"
        )]
        base: String,

        #[arg(long, default_value_t = 0.3, help = "Min co-change fraction 0.0-1.0")]
        threshold: f64,

        #[arg(long, default_value_t = 5, help = "Min commits a file must appear in")]
        min_commits: usize,

        #[arg(
            long,
            default_value = "20",
            help = "Max output files (integer or 'all')"
        )]
        max_files: String,

        #[arg(
            long,
            default_value_t = 10000,
            help = "How many recent commits to analyze"
        )]
        num_commits: usize,
    },

    #[command(about = "Report per-module LOC and recent git churn")]
    Churn {
        #[arg(
            long,
            help = "Project root path (default: git rev-parse --show-toplevel)"
        )]
        project_root: Option<String>,

        #[arg(
            long,
            default_value = "2w",
            help = "Git log time window (e.g. 2w, 30d, 3m)"
        )]
        since: String,

        #[arg(long, default_value_t = 0, help = "Minimum LOC to include in output")]
        min_loc: usize,
    },

    #[command(about = "Apply a patch (envelope format) to files under cwd")]
    ApplyPatch {
        #[arg(long, help = "Working directory (default: process cwd)")]
        cwd: Option<String>,

        #[arg(long, help = "Preview changes without writing to disk")]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum McpAction {
    #[command(about = "Serve MCP protocol over stdio")]
    Serve,
}

#[derive(Subcommand)]
pub enum ArtifactAction {
    #[command(about = "List artifacts for the current project")]
    List {
        #[arg(long, help = "Output as JSON")]
        json: bool,

        #[arg(long, help = "Show artifacts from all projects")]
        all: bool,

        #[arg(short, long, help = "Filter by project path")]
        project: Option<String>,

        #[arg(long, help = "Show archived artifacts instead of active")]
        archived: bool,

        #[arg(long, help = "Also show dive/ files (spec only)")]
        include_dives: bool,
    },

    #[command(about = "Create a new artifact file")]
    Create {
        #[arg(long, help = "Artifact topic")]
        topic: String,

        #[arg(long, help = "Project path (defaults to current git repo / cwd)")]
        project: Option<String>,

        #[arg(long, help = "Custom slug (auto-generated if omitted)")]
        slug: Option<String>,

        #[arg(long, help = "Source artifact stem for [[wiki-link]]")]
        source: Option<String>,

        #[arg(
            long,
            help = "Comma-separated tags (e.g. domain/combat,stage/research)"
        )]
        tags: Option<String>,

        #[arg(
            long,
            help = "Route to dive/ instead of spec/ (requires --source; spec only)"
        )]
        dive: bool,
    },

    #[command(about = "Read artifact file body or frontmatter")]
    Read {
        #[arg(help = "File path or stem")]
        file: String,

        #[arg(long, help = "Output frontmatter as JSON")]
        frontmatter: bool,
    },

    #[command(about = "Find most recently modified artifact file")]
    Latest {
        #[arg(long, help = "Project path (defaults to git root or cwd)")]
        project: Option<String>,

        #[arg(long, help = "Resolve this file directly instead of mtime heuristic")]
        task_file: Option<String>,

        #[arg(long, help = "Also scan dive/ files when finding latest (spec only)")]
        include_dives: bool,
    },

    #[command(about = "Move an artifact file to archive/ subfolder")]
    Archive {
        #[arg(help = "File path or stem")]
        file: Option<String>,

        #[arg(long, num_args = 1.., help = "Batch archive multiple files")]
        batch: Vec<String>,

        #[arg(long, help = "Preview what would be archived without acting")]
        dry_run: bool,
    },

    #[command(about = "Show artifact content by ID")]
    Show {
        #[arg(help = "Artifact ID or name")]
        id: String,
    },

    #[command(about = "Archive artifact files older than N days")]
    Prune {
        #[arg(long, default_value_t = 30, help = "Age threshold in days")]
        days: u64,

        #[arg(long, help = "Dry run — print what would be archived")]
        dry_run: bool,

        #[arg(short, long, help = "Filter by project path")]
        project: Option<String>,
    },

    #[command(about = "Extract inline HTML comments from an artifact")]
    Comments {
        #[arg(help = "File path or stem")]
        file: String,

        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    #[command(about = "Rename an artifact file and update its frontmatter")]
    Rename {
        #[arg(help = "Current file path or stem")]
        old: String,

        #[arg(help = "New slug for the file")]
        new_slug: String,
    },

    #[command(about = "Fix auto-derived tags (type/*, project/*) in frontmatter")]
    Retag {
        #[arg(help = "File path or stem")]
        file: String,
    },
}

#[derive(Subcommand)]
pub enum VaultAction {
    #[command(about = "Initialize ~/blueprints/ repository")]
    Init,

    #[command(about = "Migrate artifacts from ~/.claude/ to ~/blueprints/")]
    Migrate,

    #[command(about = "Print detected project name")]
    Project,

    #[command(about = "Find related artifacts by topic keyword overlap")]
    Related {
        #[arg(long, help = "Project path (defaults to current git repo / cwd)")]
        project: Option<String>,

        #[arg(help = "Topic to match against")]
        topic: String,

        #[arg(long, help = "Include archived artifacts")]
        archive: bool,
    },

    #[command(about = "Check for unresolved wiki-links (via Obsidian CLI)")]
    Check {
        #[arg(long, help = "Include archived artifacts")]
        archive: bool,
    },

    #[command(about = "Search artifacts (via Obsidian CLI)")]
    Search {
        #[arg(help = "Search query")]
        query: String,

        #[arg(long, help = "Output as JSON")]
        json: bool,

        #[arg(long, help = "Filter by artifact type (spec, plan, review, report, doc)", value_parser = ["spec", "plan", "review", "report", "doc"])]
        r#type: Option<String>,

        #[arg(short, long, help = "Filter by project path")]
        project: Option<String>,

        #[arg(long, help = "Include archived artifacts")]
        archive: bool,
    },

    #[command(about = "Show vault status (git state, artifact count)")]
    Status,
}
