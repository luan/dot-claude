use crate::plan;
use crate::store::{Priority, SortOrder, Status, StatusFilter, Store, Task, TaskList};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};

#[derive(Parser)]
#[command(name = "wasc")]
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

    #[command(about = "List tasks")]
    List {
        #[arg(long, help = "Filter by status (pending, in_progress, completed)", value_parser = ["pending", "in_progress", "completed", "active", "all"])]
        status: Option<String>,

        #[arg(long, help = "Sort by field (id, subject, priority)", value_parser = ["id", "subject", "priority"])]
        sort: Option<String>,

        #[arg(long, help = "Output as JSON")]
        json: bool,
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

        #[arg(long, help = "Priority (1-5)")]
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

    #[command(about = "List execution plans for the current project")]
    Plans {
        #[arg(long, help = "Output as JSON")]
        json: bool,

        #[arg(long, help = "Show plans from all projects")]
        all: bool,

        #[arg(short, long, help = "Filter by project path")]
        project: Option<String>,

        #[arg(long, help = "Show archived plans instead of active")]
        archived: bool,
    },

    #[command(about = "Plan file operations")]
    Plan {
        #[command(subcommand)]
        action: PlanAction,
    },

    #[command(about = "List known projects")]
    Projects {
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    #[command(about = "Generate shell completion scripts")]
    Completion {
        #[arg(help = "Shell type (bash, zsh, fish, powershell, elvish)")]
        shell: Shell,
    },

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
}

#[derive(Subcommand)]
pub enum PlanAction {
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
            println!("No tasks found.");
            return Ok(());
        }

        println!("{:<6} {:<12} {:<6} SUBJECT", "ID", "STATUS", "PRI");
        println!("{}", "-".repeat(80));

        for task in &filtered {
            let status_str = match task.status {
                crate::store::Status::Pending => "pending",
                crate::store::Status::InProgress => "in_progress",
                crate::store::Status::Completed => "completed",
                crate::store::Status::Other(ref s) => s.as_str(),
            };

            let pri_str = task.priority.as_str();

            let subject = if task.subject.len() > 60 {
                format!("{}...", truncate_at_char_boundary(&task.subject, 57))
            } else {
                task.subject.clone()
            };

            println!(
                "{:<6} {:<12} {:<6} {}",
                task.id, status_str, pri_str, subject
            );
        }
    }

    Ok(())
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
        println!("ID:          {}", task.id);
        println!("Subject:     {}", task.subject);
        println!(
            "Status:      {}",
            match task.status {
                crate::store::Status::Pending => "pending",
                crate::store::Status::InProgress => "in_progress",
                crate::store::Status::Completed => "completed",
                crate::store::Status::Other(ref s) => s.as_str(),
            }
        );
        println!("Priority:    {}", task.priority.as_str());

        if !task.description.is_empty() {
            println!("\nDescription:\n{}", task.description);
        }

        if !task.active_form.is_empty() {
            println!("\nActive Form: {}", task.active_form);
        }

        if !task.blocks.is_empty() {
            println!("\nBlocks:      {}", task.blocks.join(", "));
        }

        if !task.blocked_by.is_empty() {
            println!("Blocked By:  {}", task.blocked_by.join(", "));
        }

        if !task.task_type.is_empty() {
            println!("\nType:        {}", task.task_type);
        }

        if !task.parent_id.is_empty() {
            println!("Parent ID:   {}", task.parent_id);
        }

        if !task.branch.is_empty() {
            println!("Branch:      {}", task.branch);
        }

        if !task.status_detail.is_empty() {
            println!("Status Detail: {}", task.status_detail);
        }

        if !task.project.is_empty() {
            println!("Project:     {}", task.project);
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
    println!("Created task t{}: {}", created.id, created.subject);

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
    println!("Updated task t{}", task.id);

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

    store.save_task(&list_id, &task)?;
    println!(
        "Task t{} status: {} → {}",
        task.id, old_status, new_status_str
    );

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

    if let Some(ref proj) = project {
        plans.retain(|p| p.project.contains(proj.as_str()));
    } else if !all {
        plans.retain(|p| !p.project.is_empty() && cwd.contains(&p.project));
    }

    if plans.is_empty() {
        if all {
            eprintln!("No plans found in ~/.claude/plans/");
        } else {
            eprintln!("No plans found for current project. Use --all to show all plans.");
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
                    "modified": plan::format_date(p.mod_time),
                    "size": plan::format_size(p.size),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_plans)?);
    } else {
        println!("{:<30} {:<50} {:<12} SIZE", "NAME", "TITLE", "MODIFIED");
        println!("{}", "-".repeat(100));

        for p in &plans {
            let name = if p.name.len() > 28 {
                format!("{}...", truncate_at_char_boundary(&p.name, 25))
            } else {
                p.name.clone()
            };

            let title = if p.title.len() > 48 {
                format!("{}...", truncate_at_char_boundary(&p.title, 45))
            } else {
                p.title.clone()
            };

            println!(
                "{:<30} {:<50} {:<12} {}",
                name,
                title,
                plan::format_date(p.mod_time),
                plan::format_size(p.size)
            );
        }
    }

    Ok(())
}

pub fn run_plan(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let plans = plan::list_plans();

    if plans.is_empty() {
        eprintln!("No plans found in ~/.claude/plans/");
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
    let result = claude_slug::slug(&input);
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

pub fn run_plan_latest(project: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = vec![];
    if let Some(p) = project {
        args.push("--project".to_string());
        args.push(p);
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
    let mut projects = std::collections::BTreeSet::new();

    // Collect projects from tasks
    for list in store.list_task_lists() {
        for task in store.list_tasks(&list.id) {
            if !task.project.is_empty() {
                projects.insert(task.project);
            }
        }
    }

    // Collect projects from plans
    for plan in plan::list_plans() {
        if !plan.project.is_empty() {
            projects.insert(plan.project);
        }
    }

    if projects.is_empty() {
        eprintln!("No projects found.");
        return Ok(());
    }

    if json {
        let json_projects: Vec<_> = projects.iter().collect();
        println!("{}", serde_json::to_string_pretty(&json_projects)?);
    } else {
        for project in &projects {
            println!("{project}");
        }
    }

    Ok(())
}

pub fn run_completion(shell: Shell) -> Result<(), Box<dyn std::error::Error>> {
    generate(shell, &mut Cli::command(), "wasc", &mut std::io::stdout());
    Ok(())
}
