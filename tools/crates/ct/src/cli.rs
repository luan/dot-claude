use std::collections::BTreeMap;

use crate::ansi;
use crate::plan;
use crate::store::{Priority, SortOrder, Status, StatusFilter, Store, Task, TaskList};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};

#[derive(Parser)]
#[command(name = "ck")]
#[command(about = "Task management CLI and TUI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
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
        action: PlanAction,
    },

    #[command(visible_alias = "j", about = "Project operations")]
    Project {
        #[command(subcommand)]
        action: ProjectAction,
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
pub enum PlanAction {
    #[command(about = "List execution plans for the current project")]
    List {
        #[arg(long, help = "Output as JSON")]
        json: bool,

        #[arg(long, help = "Show plans from all projects")]
        all: bool,

        #[arg(short, long, help = "Filter by project path")]
        project: Option<String>,

        #[arg(long, help = "Show archived plans instead of active")]
        archived: bool,
    },

    #[command(about = "Create a new plan file")]
    Create {
        #[arg(long, help = "Plan topic")]
        topic: String,

        #[arg(long, help = "Project path")]
        project: String,

        #[arg(long, help = "Custom slug (auto-generated if omitted)")]
        slug: Option<String>,

        #[arg(long, help = "Filename prefix")]
        prefix: Option<String>,

        #[arg(long, help = "Plan body content")]
        body: Option<String>,
    },

    #[command(about = "Read plan file body or frontmatter")]
    Read {
        #[arg(help = "Plan file path")]
        file: String,

        #[arg(long, help = "Output frontmatter as JSON")]
        frontmatter: bool,
    },

    #[command(about = "Find most recently modified plan file")]
    Latest {
        #[arg(long, help = "Project path (defaults to git root or cwd)")]
        project: Option<String>,

        #[arg(long, help = "Resolve this file directly instead of mtime heuristic")]
        task_file: Option<String>,
    },

    #[command(about = "Move a plan file to archive/ subfolder")]
    Archive {
        #[arg(help = "Plan file path")]
        file: String,
    },

    #[command(about = "Show plan content by ID")]
    Show {
        #[arg(help = "Plan ID or name")]
        id: String,
    },
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

pub fn run_plans(
    cwd: &str,
    json: bool,
    all: bool,
    project: Option<String>,
    archived: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut plans = if archived {
        plan::list_archived_plans()
    } else {
        plan::list_plans()
    };

    // Always exclude plans without a project
    plans.retain(|p| !p.project.is_empty());

    if let Some(ref proj) = project {
        plans.retain(|p| p.project.contains(proj.as_str()));
    } else if !all {
        plans.retain(|p| cwd.contains(&p.project));
    }

    if plans.is_empty() {
        if all {
            eprintln!("{}", ansi::dim("No plans found in ~/.claude/plans/"));
        } else {
            eprintln!(
                "{}",
                ansi::dim("No plans found for current project. Use --all to show all plans.")
            );
        }
        return Ok(());
    }

    if json {
        let json_plans: Vec<_> = plans
            .iter()
            .map(|p| {
                serde_json::json!({
                    "name": p.name,
                    "title": p.title,
                    "project": crate::planfile::project_name(&p.project),
                    "modified": plan::format_date(p.mod_time),
                    "size": plan::format_size(p.size),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_plans)?);
    } else {
        println!(
            "{}",
            ansi::bold(&format!(
                "{:<12} {:<30} {:<42} {:<12} SIZE",
                "PROJECT", "NAME", "TITLE", "MODIFIED"
            ))
        );
        println!("{}", ansi::dim(&"-".repeat(100)));

        for p in &plans {
            let proj = crate::planfile::project_name(&p.project);

            let name = if p.name.len() > 28 {
                format!("{}...", truncate_at_char_boundary(&p.name, 25))
            } else {
                p.name.clone()
            };

            let title = if p.title.len() > 40 {
                format!("{}...", truncate_at_char_boundary(&p.title, 37))
            } else {
                p.title.clone()
            };

            let title_col = format!("{:<42}", title);
            println!(
                "{} {} {} {} {}",
                ansi::id(&format!("{:<12}", proj)),
                ansi::dim(&format!("{:<30}", name)),
                title_col,
                ansi::dim(&format!("{:<12}", plan::format_date(p.mod_time))),
                ansi::dim(&plan::format_size(p.size))
            );
        }
    }

    Ok(())
}

pub fn run_plan(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let plans = plan::list_plans();

    if plans.is_empty() {
        eprintln!("{}", ansi::dim("No plans found in ~/.claude/plans/"));
        return Ok(());
    }

    let normalized_id = id.strip_suffix(".md").unwrap_or(id);

    let found = plans.iter().find(|p| {
        p.name == normalized_id || p.name == id || p.path.file_name().is_some_and(|f| f == id)
    });

    let Some(plan_ref) = found else {
        eprintln!("Plan not found: {id}");
        return Ok(());
    };

    let content = plan::load_content(&plan_ref.path);
    println!("{content}");

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

pub fn run_plan_create(
    topic: String,
    project: String,
    slug: Option<String>,
    prefix: Option<String>,
    body: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = vec![
        "--topic".to_string(),
        topic,
        "--project".to_string(),
        project,
    ];
    if let Some(s) = slug {
        args.push("--slug".to_string());
        args.push(s);
    }
    if let Some(p) = prefix {
        args.push("--prefix".to_string());
        args.push(p);
    }
    if let Some(b) = body {
        args.push("--body".to_string());
        args.push(b);
    }
    crate::planfile::cmd_create(&args);
    Ok(())
}

pub fn run_plan_read(file: String, frontmatter: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = vec![file];
    if frontmatter {
        args.insert(0, "--frontmatter".to_string());
    }
    crate::planfile::cmd_read(&args);
    Ok(())
}

pub fn run_plan_latest(
    project: Option<String>,
    task_file: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = vec![];
    if let Some(p) = project {
        args.push("--project".to_string());
        args.push(p);
    }
    if let Some(tf) = task_file {
        args.push("--task-file".to_string());
        args.push(tf);
    }
    crate::planfile::cmd_latest(&args);
    Ok(())
}

pub fn run_plan_archive(file: String) -> Result<(), Box<dyn std::error::Error>> {
    crate::planfile::cmd_archive(&[file]);
    Ok(())
}

pub fn run_plan_show(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    run_plan(id)
}

pub fn run_projects(store: &Store, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    // slug -> path (empty string for plan-subdir-only entries)
    let mut projects: BTreeMap<String, String> = BTreeMap::new();

    // Source 1: tasks with a non-empty project field
    for list in store.list_task_lists() {
        for task in store.list_tasks(&list.id) {
            if !task.project.is_empty() {
                let slug = crate::planfile::project_name(&task.project);
                projects.entry(slug).or_insert(task.project);
            }
        }
    }

    // Source 2: plans with a non-empty project field
    for plan in plan::list_plans() {
        if !plan.project.is_empty() {
            let slug = crate::planfile::project_name(&plan.project);
            projects.entry(slug).or_insert(plan.project);
        }
    }

    // Source 3: subdirectories of ~/.claude/plans/ (excluding "archive")
    if let Ok(home) = std::env::var("HOME") {
        let plans_base = std::path::PathBuf::from(home).join(".claude").join("plans");
        if let Ok(entries) = std::fs::read_dir(&plans_base) {
            for entry in entries.flatten() {
                if entry.path().is_dir()
                    && let Some(name) = entry.file_name().to_str()
                    && name != "archive"
                {
                    projects.entry(name.to_string()).or_default();
                }
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
            if !task.project.is_empty() && crate::planfile::project_name(&task.project) == slug {
                project_path = task.project.clone();
                break;
            }
        }
        if !project_path.is_empty() {
            break;
        }
    }
    if project_path.is_empty() {
        for p in plan::list_plans() {
            if !p.project.is_empty() && crate::planfile::project_name(&p.project) == slug {
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
    let project_plans: Vec<_> = plan::list_plans()
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

pub fn run_completion(shell: Shell) -> Result<(), Box<dyn std::error::Error>> {
    generate(shell, &mut Cli::command(), "ck", &mut std::io::stdout());
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
