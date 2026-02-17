mod ansi;
mod app;
mod cli;
mod editor;
mod phases;
mod plan;
mod planfile;
mod store;
mod ui;

use clap::{CommandFactory, Parser};
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use std::io;
use std::time::Duration;

fn store_and_cwd() -> (store::Store, String) {
    let store = store::Store::new();
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    (store, cwd)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();

    match cli.command {
        None => {
            cli::Cli::command().print_help()?;
            println!();
            Ok(())
        }
        Some(cli::Command::Tui) => run_tui(),
        Some(cli::Command::Task { action }) => match action {
            cli::TaskAction::List { status, sort, json } => {
                let (store, cwd) = store_and_cwd();
                cli::run_list(&store, &cwd, status, sort, json)
            }
            cli::TaskAction::Show { id, json } => {
                let (store, cwd) = store_and_cwd();
                cli::run_show(&store, &cwd, &id, json)
            }
            cli::TaskAction::Create {
                subject,
                description,
                priority,
                parent,
            } => {
                let (store, cwd) = store_and_cwd();
                cli::run_create(&store, &cwd, subject, description, priority, parent)
            }
            cli::TaskAction::Edit {
                id,
                subject,
                status,
                priority,
            } => {
                let (store, cwd) = store_and_cwd();
                cli::run_edit(&store, &cwd, &id, subject, status, priority)
            }
            cli::TaskAction::Status { id, status } => {
                let (store, cwd) = store_and_cwd();
                cli::run_status(&store, &cwd, &id, &status)
            }
        },
        Some(cli::Command::Plan { action }) => match action {
            cli::PlanAction::List {
                json,
                all,
                project,
                archived,
            } => {
                let (_, cwd) = store_and_cwd();
                cli::run_plans(&cwd, json, all, project, archived)
            }
            cli::PlanAction::Create {
                topic,
                project,
                slug,
                prefix,
                body,
            } => cli::run_plan_create(topic, project, slug, prefix, body),
            cli::PlanAction::Read { file, frontmatter } => cli::run_plan_read(file, frontmatter),
            cli::PlanAction::Latest { project } => cli::run_plan_latest(project),
            cli::PlanAction::Archive { file } => cli::run_plan_archive(file),
            cli::PlanAction::Show { id } => cli::run_plan_show(&id),
        },
        Some(cli::Command::Project { action }) => match action {
            cli::ProjectAction::List { json } => {
                let (store, _) = store_and_cwd();
                cli::run_projects(&store, json)
            }
            cli::ProjectAction::Show { slug } => {
                println!("Project show not implemented yet: {slug}");
                Ok(())
            }
        },
        Some(cli::Command::Tool { action }) => match action {
            cli::ToolAction::Slug { words } => cli::run_slug(words),
            cli::ToolAction::Phases { file } => phases::run_phases(file),
            cli::ToolAction::Completion { shell } => cli::run_completion(shell),
        },
    }
}

fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    let store = store::Store::new();

    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let lists = store.discover_lists(&cwd);
    if lists.is_empty() {
        eprintln!("No task lists found in ~/.claude/tasks/");
        return Ok(());
    }

    let active_list = lists[0].id.clone();
    let tasks = store.list_tasks(&active_list);

    let mut app = app::App::new(store, lists, tasks, active_list);

    let editor_cmd = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vim".to_string());

    // Terminal setup
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut app, &editor_cmd);

    // Terminal restore
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    app: &mut app::App,
    editor_cmd: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| app.render(f))?;

        // Check for editor request
        if let Some(req) = app.editor_request.take() {
            // Leave TUI, run editor, come back
            terminal::disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;

            let status = std::process::Command::new(editor_cmd)
                .arg(&req.path)
                .status();

            // Re-enter TUI
            execute!(terminal.backend_mut(), EnterAlternateScreen)?;
            terminal::enable_raw_mode()?;
            terminal.hide_cursor()?;
            terminal.clear()?;

            if status.is_ok() {
                app.handle_editor_result(&req.task_id, &req.path, &req.list_id);
            } else {
                let _ = std::fs::remove_file(&req.path);
            }
            continue;
        }

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key);
            if app.should_quit {
                return Ok(());
            }
        }
    }
}
