use std::collections::BTreeMap;

use crate::ansi;
use crate::store::{Priority, SortOrder, Status, StatusFilter, Store, Task, TaskList};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};

#[derive(Parser)]
#[command(name = "ct")]
#[command(about = "Task management CLI and TUI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Handle a SyncError from commit_and_push: push failures exit 2, others exit 1.
fn handle_sync_error(e: crate::artifact::SyncError) -> ! {
    eprintln!("{e}");
    match e {
        crate::artifact::SyncError::Push(_) => std::process::exit(2),
        _ => std::process::exit(1),
    }
}

fn require_lists(store: &Store, cwd: &str) -> Result<Vec<TaskList>, Box<dyn std::error::Error>> {
    let lists = store.discover_lists(cwd);
    if lists.is_empty() {
        Err("No task lists found in ~/.claude/tasks/".into())
    } else {
        Ok(lists)
    }
}

fn find_task(store: &Store, lists: &[TaskList], task_id: &str) -> Option<(String, Task)> {
    lists.iter().find_map(|list| {
        store
            .load_task(&list.id, task_id)
            .map(|t| (list.id.clone(), t))
    })
}

fn truncate_at_char_boundary(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut idx = max_bytes;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    &s[..idx]
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Launch the interactive TUI")]
    Tui,

    #[command(visible_alias = "t", about = "Task operations")]
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },

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

    #[command(visible_alias = "j", about = "Project operations")]
    Project {
        #[command(subcommand)]
        action: ProjectAction,
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
}

#[derive(Subcommand)]
pub enum TaskAction {
    #[command(about = "List tasks")]
    List {
        #[arg(long, help = "Filter by status (pending, in_progress, completed)", value_parser = ["pending", "in_progress", "completed", "active", "all"])]
        status: Option<String>,

        #[arg(long, help = "Sort by field (id, subject, priority)", value_parser = ["id", "subject", "priority"])]
        sort: Option<String>,

        #[arg(long, help = "Output as JSON")]
        json: bool,

        #[arg(long, help = "Display tasks as a tree grouped by parent")]
        tree: bool,
    },

    #[command(about = "Show task details")]
    Show {
        #[arg(help = "Task ID")]
        id: String,

        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    #[command(about = "Create a new task")]
    Create {
        #[arg(help = "Task subject")]
        subject: String,

        #[arg(long, help = "Task description")]
        description: Option<String>,

        #[arg(long, help = "Priority (1-3)")]
        priority: Option<u8>,

        #[arg(long, help = "Parent task ID")]
        parent: Option<String>,
    },

    #[command(about = "Edit an existing task")]
    Edit {
        #[arg(help = "Task ID")]
        id: String,

        #[arg(long, help = "New subject")]
        subject: Option<String>,

        #[arg(long, help = "New status (pending, in_progress, completed)", value_parser = ["pending", "in_progress", "completed"])]
        status: Option<String>,

        #[arg(long, help = "New priority (1-5)")]
        priority: Option<u8>,
    },

    #[command(about = "Update task status")]
    Status {
        #[arg(help = "Task ID")]
        id: String,

        #[arg(help = "New status (pending, in_progress, completed)", value_parser = ["pending", "in_progress", "completed"])]
        status: String,
    },

    #[command(about = "Archive completed tasks older than N days")]
    Prune {
        #[arg(long, default_value_t = 7, help = "Age threshold in days")]
        days: u64,

        #[arg(long, help = "Dry run — print what would be pruned without archiving")]
        dry_run: bool,

        #[arg(long, help = "Only prune tasks from this list ID")]
        list: Option<String>,
    },
}

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
}

#[derive(Subcommand)]
pub enum ProjectAction {
    #[command(about = "List known projects")]
    List {
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    #[command(about = "Show project details")]
    Show {
        #[arg(help = "Project slug")]
        slug: String,
    },
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
    },

    #[command(about = "Create a new artifact file")]
    Create {
        #[arg(long, help = "Artifact topic")]
        topic: String,

        #[arg(long, help = "Project path")]
        project: String,

        #[arg(long, help = "Custom slug (auto-generated if omitted)")]
        slug: Option<String>,

        #[arg(long, help = "Source artifact stem for [[wiki-link]]")]
        source: Option<String>,

        #[arg(
            long,
            help = "Comma-separated tags (e.g. domain/combat,stage/research)"
        )]
        tags: Option<String>,

        #[arg(long, help = "Artifact body content")]
        body: Option<String>,
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
    },

    #[command(about = "Move an artifact file to archive/ subfolder")]
    Archive {
        #[arg(help = "File path or stem")]
        file: String,
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
        #[arg(long, help = "Project path")]
        project: String,

        #[arg(help = "Topic to match against")]
        topic: String,
    },

    #[command(about = "Check for unresolved wiki-links (via Obsidian CLI)")]
    Check,

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
    },

    #[command(about = "Show vault status (git state, artifact count)")]
    Status,
}

pub fn run_list(
    store: &Store,
    cwd: &str,
    status_arg: Option<String>,
    sort_arg: Option<String>,
    json: bool,
    tree: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let lists = require_lists(store, cwd)?;
    let list_id = &lists[0].id;
    let tasks = store.list_tasks(list_id);

    let status_filter = if let Some(status_str) = status_arg {
        match status_str.as_str() {
            "pending" => StatusFilter::Pending,
            "in_progress" => StatusFilter::InProgress,
            "completed" => StatusFilter::Completed,
            "active" => StatusFilter::Active,
            "all" => StatusFilter::All,
            _ => {
                eprintln!("Invalid status filter: {status_str}");
                eprintln!("Valid options: pending, in_progress, completed, active, all");
                return Ok(());
            }
        }
    } else {
        StatusFilter::All
    };

    let sort_order = match sort_arg.as_deref() {
        Some("id") => SortOrder::Id,
        Some("priority") => SortOrder::Priority,
        Some("subject") => SortOrder::Subject,
        Some(other) => {
            eprintln!("Invalid sort field: {other}");
            eprintln!("Valid options: id, priority, subject");
            return Ok(());
        }
        None => SortOrder::Id,
    };

    let filtered = crate::store::filter_and_sort(&tasks, status_filter, sort_order, true, "");

    if json {
        let json_tasks: Vec<_> = filtered.iter().map(|t| t.to_json()).collect();
        println!("{}", serde_json::to_string_pretty(&json_tasks)?);
    } else {
        if filtered.is_empty() {
            println!("{}", ansi::dim("No tasks found."));
            return Ok(());
        }

        println!(
            "{}",
            ansi::bold(&format!(
                "{:<6} {:<12} {:<6} {:<10} {:<12} SUBJECT",
                "ID", "STATUS", "PRI", "TYPE", "OWNER"
            ))
        );
        println!("{}", ansi::dim(&"-".repeat(100)));

        let completed_ids: std::collections::HashSet<&str> = filtered
            .iter()
            .filter(|t| t.status == crate::store::Status::Completed)
            .map(|t| t.id.as_str())
            .collect();

        if tree {
            let rows = crate::store::tree_order(&filtered);
            for row in &rows {
                let task = &row.task;
                print_task_row(task, &crate::store::tree_prefix(row), &completed_ids, true);
            }
        } else {
            for task in &filtered {
                print_task_row(task, "", &completed_ids, false);
            }
        }
    }

    Ok(())
}

fn print_task_row(
    task: &Task,
    prefix: &str,
    completed_ids: &std::collections::HashSet<&str>,
    tree: bool,
) {
    let status_str = task.status.as_str();

    let pri_str = task.priority.as_str();

    let type_str = if task.task_type.is_empty() {
        "--".to_string()
    } else {
        task.task_type.clone()
    };

    let owner_str = if task.owner.is_empty() {
        "--".to_string()
    } else if task.owner.len() > 10 {
        format!("{}...", truncate_at_char_boundary(&task.owner, 7))
    } else {
        task.owner.clone()
    };

    let blocked = !task.blocked_by.is_empty()
        && task
            .blocked_by
            .iter()
            .any(|dep| !completed_ids.contains(dep.as_str()));

    let subject_raw = format!("{prefix}{}", task.subject);
    let subject = if subject_raw.chars().count() > 50 {
        format!("{}...", truncate_at_char_boundary(&subject_raw, 47))
    } else {
        subject_raw
    };
    let subject = if tree && blocked {
        let active_ids: Vec<&str> = task
            .blocked_by
            .iter()
            .filter(|dep| !completed_ids.contains(dep.as_str()))
            .map(|s| s.as_str())
            .collect();
        format!("{subject} [← {}]", active_ids.join(", "))
    } else {
        subject
    };

    let status_col = if blocked {
        ansi::blocked(&format!("{:<12}", "blocked"))
    } else {
        ansi::for_status(&task.status, &format!("{:<12}", status_str))
    };

    println!(
        "{} {} {} {} {} {}",
        ansi::id(&format!("{:<6}", task.id)),
        status_col,
        ansi::for_priority(&task.priority, &format!("{:<6}", pri_str)),
        ansi::for_type(&task.task_type, &format!("{:<10}", type_str)),
        ansi::dim(&format!("{:<12}", owner_str)),
        subject
    );
}

pub fn run_show(
    store: &Store,
    cwd: &str,
    task_id: &str,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let lists = require_lists(store, cwd)?;

    let Some((_list_id, task)) = find_task(store, &lists, task_id) else {
        eprintln!("Task not found: {task_id}");
        return Ok(());
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&task.to_json())?);
    } else {
        let status_str = match task.status {
            crate::store::Status::Pending => "pending",
            crate::store::Status::InProgress => "in_progress",
            crate::store::Status::Completed => "completed",
            crate::store::Status::Other(ref s) => s.as_str(),
        };

        println!("{} {}", ansi::label("ID:"), ansi::id(&task.id));
        println!("{} {}", ansi::label("Subject:"), task.subject);
        println!(
            "{} {}",
            ansi::label("Status:"),
            ansi::for_status(&task.status, status_str)
        );
        println!(
            "{} {}",
            ansi::label("Priority:"),
            ansi::for_priority(&task.priority, task.priority.as_str())
        );

        if !task.owner.is_empty() {
            println!("{} {}", ansi::label("Owner:"), task.owner);
        }

        if !task.description.is_empty() {
            println!("\n{}", ansi::section("Description:"));
            println!("{}", task.description);
        }

        if !task.active_form.is_empty() {
            println!("\n{} {}", ansi::label("Active Form:"), task.active_form);
        }

        if !task.blocks.is_empty() {
            println!("\n{} {}", ansi::label("Blocks:"), task.blocks.join(", "));
        }

        if !task.blocked_by.is_empty() {
            println!(
                "{} {}",
                ansi::label("Blocked By:"),
                task.blocked_by.join(", ")
            );
        }

        if !task.task_type.is_empty() {
            println!("\n{} {}", ansi::label("Type:"), task.task_type);
        }

        if !task.parent_id.is_empty() {
            println!(
                "{} {}",
                ansi::label("Parent ID:"),
                ansi::id(&task.parent_id)
            );
        }

        if !task.branch.is_empty() {
            println!("{} {}", ansi::label("Branch:"), task.branch);
        }

        if !task.status_detail.is_empty() {
            println!("{} {}", ansi::label("Status Detail:"), task.status_detail);
        }

        if !task.project.is_empty() {
            println!("{} {}", ansi::label("Project:"), ansi::id(&task.project));
        }

        if !task.plan_file.is_empty() {
            println!("{} {}", ansi::label("Plan File:"), task.plan_file);
        }

        if !task.spec_file.is_empty() {
            println!("{} {}", ansi::label("Spec File:"), task.spec_file);
        }

        if !task.slug.is_empty() {
            println!("{} {}", ansi::label("Slug:"), task.slug);
        }
    }

    Ok(())
}

pub fn run_create(
    store: &Store,
    cwd: &str,
    subject: String,
    description: Option<String>,
    priority: Option<u8>,
    parent: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let lists = require_lists(store, cwd)?;
    let list_id = &lists[0].id;

    let priority_enum = if let Some(p) = priority {
        let prio = Priority::from_u8(p);
        if p > 3 {
            eprintln!("Warning: invalid priority {p}, using None");
        }
        prio
    } else {
        Priority::None
    };

    let task = Task {
        id: String::new(),
        subject: subject.clone(),
        description: description.unwrap_or_default(),
        active_form: String::new(),
        status: Status::Pending,
        owner: String::new(),
        blocks: Vec::new(),
        blocked_by: Vec::new(),
        priority: priority_enum,
        task_type: String::new(),
        parent_id: parent.unwrap_or_default(),
        branch: String::new(),
        status_detail: String::new(),
        project: String::new(),
        plan_file: String::new(),
        spec_file: String::new(),
        slug: String::new(),
        vibe_stage: String::new(),
        vibe_epic: String::new(),
        vibe_prompt: String::new(),
        session_id: String::new(),
        raw: serde_json::Value::Null,
    };

    let created = store.create_task(list_id, &task)?;
    println!("{}", ansi::id(&format!("t{}", created.id)));

    Ok(())
}

pub fn run_edit(
    store: &Store,
    cwd: &str,
    task_id: &str,
    subject: Option<String>,
    status_arg: Option<String>,
    priority: Option<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let lists = require_lists(store, cwd)?;

    let Some((list_id, mut task)) = find_task(store, &lists, task_id) else {
        eprintln!("Task not found: {task_id}");
        return Ok(());
    };

    if let Some(new_subject) = subject {
        task.subject = new_subject;
    }

    if let Some(new_status) = status_arg {
        task.status = Status::from_str(&new_status);
    }

    if let Some(p) = priority {
        if p > 3 {
            eprintln!("Warning: invalid priority {p}, using None");
        }
        task.priority = Priority::from_u8(p);
    }

    store.save_task(&list_id, &task)?;
    println!("Updated {}", ansi::id(&format!("t{}", task.id)));

    Ok(())
}

pub fn run_status(
    store: &Store,
    cwd: &str,
    task_id: &str,
    new_status: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let lists = require_lists(store, cwd)?;

    let Some((list_id, mut task)) = find_task(store, &lists, task_id) else {
        eprintln!("Task not found: {task_id}");
        return Ok(());
    };
    let old_status = task.status.as_str().to_string();
    task.status = Status::from_str(new_status);
    let new_status_str = task.status.as_str();

    let old_colored = ansi::for_status(&Status::from_str(&old_status), &old_status);
    let new_colored = ansi::for_status(&task.status, new_status_str);
    store.save_task(&list_id, &task)?;
    println!(
        "{}: {} {} {}",
        ansi::id(&format!("t{}", task.id)),
        old_colored,
        ansi::arrow(),
        new_colored
    );

    Ok(())
}

pub fn run_prune(
    store: &Store,
    days: u64,
    dry_run: bool,
    list: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let lists = if let Some(ref id) = list {
        vec![TaskList { id: id.clone() }]
    } else {
        store.list_task_lists()
    };

    let threshold = std::time::Duration::from_secs(days * 86400);
    let now = std::time::SystemTime::now();
    let mut archived_count = 0u32;

    for task_list in &lists {
        let list_dir = store.tasks_base().join(&task_list.id);
        let tasks = store.list_tasks(&task_list.id);

        for task in &tasks {
            if task.status != crate::store::Status::Completed {
                continue;
            }

            let is_old_enough = crate::store::task_completed_time(task, &list_dir)
                .and_then(|t| now.duration_since(t).ok())
                .is_some_and(|elapsed| elapsed >= threshold);

            if !is_old_enough {
                continue;
            }

            if dry_run {
                println!("would archive: {} ({})", task.id, task.subject);
            } else {
                store.archive_task(&task_list.id, &task.id)?;
                archived_count += 1;
            }
        }
    }

    if !dry_run {
        if archived_count > 0 {
            println!("Archived {archived_count} completed task(s)");
        }
        // Only scan all lists for empty-list cleanup when no specific list was targeted.
        // Scoping to a single list would miss other empty lists anyway, and the list
        // specified by --list is unlikely to be empty right after archiving from it.
        if list.is_none() {
            let removed_lists = store.prune_empty_lists();
            if !removed_lists.is_empty() {
                println!("Removed {} empty list(s)", removed_lists.len());
            }
        }
    }

    Ok(())
}

pub fn run_slug(words: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if words.is_empty() {
        return Ok(());
    }
    let input = words.join(" ");
    let result = crate::slug::slug(&input);
    if !result.is_empty() {
        println!("{result}");
    }
    Ok(())
}

pub fn run_projects(store: &Store, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    // slug -> path (empty string for plan-subdir-only entries)
    let mut projects: BTreeMap<String, String> = BTreeMap::new();

    // Source 1: tasks with a non-empty project field
    for list in store.list_task_lists() {
        for task in store.list_tasks(&list.id) {
            if !task.project.is_empty() {
                let slug = crate::artifact::project_name(&task.project);
                projects.entry(slug).or_insert(task.project);
            }
        }
    }

    // Source 2: plans with a non-empty project field
    for plan in crate::artifact::list_artifacts(crate::artifact::ArtifactKind::Plan) {
        if !plan.project.is_empty() {
            let slug = crate::artifact::project_name(&plan.project);
            projects.entry(slug).or_insert(plan.project);
        }
    }

    // Source 3: project subdirectories of the vault
    let bp = crate::artifact::blueprints_dir_unchecked();
    if let Ok(entries) = std::fs::read_dir(&bp)
    {
        for entry in entries.flatten() {
            if entry.path().is_dir()
                && let Some(name) = entry.file_name().to_str()
                && name != "archive"
            {
                projects.entry(name.to_string()).or_default();
            }
        }
    }

    if projects.is_empty() {
        eprintln!("{}", ansi::dim("No projects found."));
        return Ok(());
    }

    if json {
        let json_projects: Vec<_> = projects
            .iter()
            .map(|(slug, path)| {
                if path.is_empty() {
                    serde_json::json!({ "slug": slug })
                } else {
                    serde_json::json!({ "slug": slug, "path": path })
                }
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_projects)?);
    } else {
        println!("{}", ansi::bold(&format!("{:<30} PATH", "SLUG")));
        println!("{}", ansi::dim(&"-".repeat(80)));
        for (slug, path) in &projects {
            println!("{} {}", ansi::id(&format!("{:<30}", slug)), ansi::dim(path));
        }
    }

    Ok(())
}

pub fn run_project_show(store: &Store, slug: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Find project path by matching slug against known projects
    let mut project_path = String::new();
    for list in store.list_task_lists() {
        for task in store.list_tasks(&list.id) {
            if !task.project.is_empty() && crate::artifact::project_name(&task.project) == slug {
                project_path = task.project.clone();
                break;
            }
        }
        if !project_path.is_empty() {
            break;
        }
    }
    if project_path.is_empty() {
        for p in crate::artifact::list_artifacts(crate::artifact::ArtifactKind::Plan) {
            if !p.project.is_empty() && crate::artifact::project_name(&p.project) == slug {
                project_path = p.project.clone();
                break;
            }
        }
    }

    if project_path.is_empty() {
        eprintln!("Project not found: {slug}");
        std::process::exit(1);
    }

    // Header
    println!("{}", ansi::bold(slug));
    println!("{}", ansi::dim(&project_path));
    println!();

    // Tasks by status
    let mut pending = 0u32;
    let mut in_progress = 0u32;
    let mut completed = 0u32;
    let mut active_tasks: Vec<(String, String, String)> = Vec::new(); // (id, status, subject)

    for list in store.list_task_lists() {
        for task in store.list_tasks(&list.id) {
            if task.project != project_path {
                continue;
            }
            match task.status {
                crate::store::Status::Pending => pending += 1,
                crate::store::Status::InProgress => in_progress += 1,
                crate::store::Status::Completed => completed += 1,
                _ => {}
            }
            if task.status != crate::store::Status::Completed {
                active_tasks.push((
                    task.id.clone(),
                    task.status.as_str().to_string(),
                    task.subject.clone(),
                ));
            }
        }
    }

    println!(
        "{} {} pending, {} in progress, {} completed",
        ansi::label("Tasks:"),
        pending,
        in_progress,
        completed
    );
    println!();

    if !active_tasks.is_empty() {
        println!("{}", ansi::section("Active Tasks"));
        for (id, status, subject) in &active_tasks {
            let subj = truncate_at_char_boundary(subject, 60);
            println!(
                "  {} {} {}",
                ansi::id(&format!("{:<5}", id)),
                ansi::for_status(
                    &crate::store::Status::from_str(status),
                    &format!("{:<12}", status)
                ),
                subj
            );
        }
        println!();
    }

    // Recent plans
    let project_plans: Vec<_> =
        crate::artifact::list_artifacts(crate::artifact::ArtifactKind::Plan)
            .into_iter()
            .filter(|p| p.project == project_path)
            .take(5)
            .collect();

    if !project_plans.is_empty() {
        println!("{}", ansi::section("Recent Plans"));
        for p in &project_plans {
            println!("  {} {}", ansi::dim(&p.name), p.title);
        }
    }

    Ok(())
}

// ── Generic artifact operations (for Review, Report, and future types) ──────

pub fn run_artifact_list(
    kind: crate::artifact::ArtifactKind,
    cwd: &str,
    json: bool,
    all: bool,
    project: Option<String>,
    archived: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut items = if archived {
        crate::artifact::list_archived_artifacts(kind)
    } else {
        crate::artifact::list_artifacts(kind)
    };

    items.retain(|a| !a.project.is_empty());

    if let Some(ref proj) = project {
        items.retain(|a| a.project.contains(proj.as_str()));
    } else if !all {
        items.retain(|a| cwd.contains(&a.project));
    }

    let label = kind.dir_name();

    if items.is_empty() {
        if all {
            eprintln!(
                "{}",
                ansi::dim(&format!("No {label}s found in ~/blueprints/"))
            );
        } else {
            eprintln!(
                "{}",
                ansi::dim(&format!(
                    "No {label}s found for current project. Use --all to show all {label}s."
                ))
            );
        }
        return Ok(());
    }

    if json {
        let json_items: Vec<_> = items
            .iter()
            .map(|a| {
                serde_json::json!({
                    "name": a.name,
                    "title": a.title,
                    "project": crate::artifact::project_name(&a.project),
                    "modified": crate::artifact::format_date(a.mod_time),
                    "size": crate::artifact::format_size(a.size),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_items)?);
    } else {
        println!(
            "{}",
            ansi::bold(&format!(
                "{:<12} {:<30} {:<42} {:<12} SIZE",
                "PROJECT", "NAME", "TITLE", "MODIFIED"
            ))
        );
        println!("{}", ansi::dim(&"-".repeat(100)));

        for a in &items {
            let proj = crate::artifact::project_name(&a.project);

            let name = if a.name.len() > 28 {
                format!("{}...", truncate_at_char_boundary(&a.name, 25))
            } else {
                a.name.clone()
            };

            let title = if a.title.len() > 40 {
                format!("{}...", truncate_at_char_boundary(&a.title, 37))
            } else {
                a.title.clone()
            };

            let title_col = format!("{:<42}", title);
            println!(
                "{} {} {} {} {}",
                ansi::id(&format!("{:<12}", proj)),
                ansi::dim(&format!("{:<30}", name)),
                title_col,
                ansi::dim(&format!("{:<12}", crate::artifact::format_date(a.mod_time))),
                ansi::dim(&crate::artifact::format_size(a.size))
            );
        }
    }

    Ok(())
}

pub fn run_artifact_show(
    kind: crate::artifact::ArtifactKind,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let items = crate::artifact::list_artifacts(kind);
    let label = kind.dir_name();

    if items.is_empty() {
        eprintln!(
            "{}",
            ansi::dim(&format!("No {label}s found in ~/blueprints/"))
        );
        return Ok(());
    }

    let normalized_id = id.strip_suffix(".md").unwrap_or(id);

    let found = items.iter().find(|a| {
        a.name == normalized_id || a.name == id || a.path.file_name().is_some_and(|f| f == id)
    });

    let Some(artifact_ref) = found else {
        eprintln!("{label} not found: {id}");
        return Ok(());
    };

    let content = crate::artifact::load_content(kind, &artifact_ref.path);
    println!("{content}");

    Ok(())
}

pub fn run_artifact_create(
    kind: crate::artifact::ArtifactKind,
    topic: String,
    project: String,
    slug: Option<String>,
    source: Option<String>,
    tags: Option<String>,
    body: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tag_list: Vec<String> = tags
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if let Err(e) = crate::artifact::cmd_create(
        kind,
        &topic,
        &project,
        slug.as_deref(),
        source.as_deref(),
        &tag_list,
        body.unwrap_or_default(),
    ) {
        handle_sync_error(e);
    }
    Ok(())
}

pub fn run_artifact_read(
    kind: crate::artifact::ArtifactKind,
    file: String,
    frontmatter: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::artifact::cmd_read(&file, kind, frontmatter);
    Ok(())
}

pub fn run_artifact_latest(
    kind: crate::artifact::ArtifactKind,
    project: Option<String>,
    task_file: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::artifact::cmd_latest(kind, project.as_deref(), task_file.as_deref());
    Ok(())
}

pub fn run_artifact_archive(
    kind: crate::artifact::ArtifactKind,
    file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = crate::artifact::cmd_archive(kind, &file) {
        handle_sync_error(e);
    }
    Ok(())
}

pub fn run_artifact_prune(
    kind: crate::artifact::ArtifactKind,
    days: u64,
    dry_run: bool,
    project: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bp = crate::artifact::blueprints_dir();
    let kind_dir = kind.dir_name();
    let threshold = std::time::Duration::from_secs(days * 86400);
    let now = std::time::SystemTime::now();
    let mut archived_count = 0u32;
    let mut sync_errors = 0u32;

    let Ok(project_dirs) = std::fs::read_dir(&bp) else {
        eprintln!(
            "{}",
            ansi::dim(&format!("No {kind_dir} found in ~/blueprints/"))
        );
        return Ok(());
    };

    for dir_entry in project_dirs.flatten() {
        if !dir_entry.path().is_dir() {
            continue;
        }
        let dir_name = dir_entry.file_name().to_string_lossy().to_string();
        if dir_name == "archive" {
            continue;
        }
        if let Some(ref proj) = project
            && !dir_name.contains(proj.as_str())
        {
            continue;
        }

        let artifact_dir = dir_entry.path().join(kind_dir);
        let Ok(files) = std::fs::read_dir(&artifact_dir) else {
            continue;
        };
        for file_entry in files.flatten() {
            let path = file_entry.path();
            if path.is_dir() || path.extension().is_none_or(|ext| ext != "md") {
                continue;
            }
            let Ok(meta) = file_entry.metadata() else {
                continue;
            };
            let Ok(modified) = meta.modified() else {
                continue;
            };
            let Ok(age) = now.duration_since(modified) else {
                continue;
            };
            if age < threshold {
                continue;
            }

            let path_str = path.to_string_lossy().to_string();
            if dry_run {
                println!("would archive: {path_str}");
            } else {
                match crate::artifact::cmd_archive(kind, &path_str) {
                    Ok(()) => archived_count += 1,
                    Err(e) => {
                        eprintln!("{e}");
                        sync_errors += 1;
                    }
                }
            }
        }
    }

    if !dry_run && archived_count > 0 {
        println!("Archived {archived_count} {} file(s)", kind.dir_name());
    }
    if sync_errors > 0 {
        eprintln!("{sync_errors} file(s) failed to sync");
        std::process::exit(2);
    }

    Ok(())
}

pub fn run_completion(shell: Shell) -> Result<(), Box<dyn std::error::Error>> {
    generate(shell, &mut Cli::command(), "ct", &mut std::io::stdout());
    Ok(())
}

pub fn run_cochanges(
    base: String,
    threshold: f64,
    min_commits: usize,
    max_files_str: String,
    num_commits: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let max_files = if max_files_str.to_lowercase() == "all" {
        None
    } else {
        let n: usize = max_files_str
            .parse()
            .map_err(|_| format!("invalid max-files: {max_files_str}"))?;
        if n == 0 {
            return Err("max-files must be positive or 'all'".into());
        }
        Some(n)
    };
    crate::cochanges::run(base, threshold, min_commits, max_files, num_commits)
}
