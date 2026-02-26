mod ansi;
mod app;
mod cli;
mod cochanges;
mod editor;
mod gitcontext;
mod notify;
mod phases;
mod plan;
mod planfile;
mod slug;
mod store;
mod ui;

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;

use clap::{CommandFactory, Parser};
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use fs_notify::{RecursiveMode, Watcher};

enum AppEvent {
    Terminal(Event),
    FsChange,
}

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
            cli::TaskAction::List {
                status,
                sort,
                json,
                tree,
            } => {
                let (store, cwd) = store_and_cwd();
                cli::run_list(&store, &cwd, status, sort, json, tree)
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
            cli::TaskAction::Prune {
                days,
                dry_run,
                list,
            } => {
                let (store, _) = store_and_cwd();
                cli::run_prune(&store, days, dry_run, list)
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
            cli::PlanAction::Latest { project, task_file } => {
                cli::run_plan_latest(project, task_file)
            }
            cli::PlanAction::Archive { file } => cli::run_plan_archive(file),
            cli::PlanAction::Show { id } => cli::run_plan_show(&id),
        },
        Some(cli::Command::Project { action }) => match action {
            cli::ProjectAction::List { json } => {
                let (store, _) = store_and_cwd();
                cli::run_projects(&store, json)
            }
            cli::ProjectAction::Show { slug } => {
                let (store, _) = store_and_cwd();
                cli::run_project_show(&store, &slug)
            }
        },
        Some(cli::Command::Notify) => notify::run(),
        Some(cli::Command::Tool { action }) => match action {
            cli::ToolAction::Slug { words } => cli::run_slug(words),
            cli::ToolAction::Phases { file } => phases::run_phases(file),
            cli::ToolAction::Completion { shell } => cli::run_completion(shell),
            cli::ToolAction::Gitcontext {
                base,
                format,
                max_total,
                max_file,
            } => gitcontext::run(base, format, max_total, max_file),
            cli::ToolAction::Cochanges {
                base,
                threshold,
                min_commits,
                max_files,
                num_commits,
            } => cli::run_cochanges(base, threshold, min_commits, max_files, num_commits),
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

    let quit_flag = Arc::new(AtomicBool::new(false));
    let result = run_loop(&mut terminal, &mut app, &editor_cmd, Arc::clone(&quit_flag));

    // Signal the terminal reader thread to stop before tearing down raw mode
    quit_flag.store(true, Ordering::Relaxed);

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
    quit_flag: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();

    // Terminal event reader thread
    let term_tx = tx.clone();
    let reader_quit = Arc::clone(&quit_flag);
    std::thread::spawn(move || {
        loop {
            if reader_quit.load(Ordering::Relaxed) {
                break;
            }
            if event::poll(Duration::from_millis(100)).unwrap_or(false)
                && let Ok(evt) = event::read()
                && term_tx.send(AppEvent::Terminal(evt)).is_err()
            {
                break;
            }
        }
    });

    // Filesystem watcher on the tasks base directory
    let fs_tx = tx;
    let mut watcher =
        fs_notify::recommended_watcher(move |res: fs_notify::Result<fs_notify::Event>| {
            if let Ok(evt) = res {
                let dominated_by_json = evt.paths.iter().any(|p| {
                    p.extension().is_some_and(|ext| ext == "json")
                        && !p
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .starts_with(".tmp-")
                });
                if dominated_by_json {
                    let _ = fs_tx.send(AppEvent::FsChange);
                }
            }
        })?;
    watcher.watch(app.tasks_base_path(), RecursiveMode::Recursive)?;

    loop {
        terminal.draw(|f| app.render(f))?;

        // Check for editor request
        if let Some(req) = app.editor_request.take() {
            terminal::disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;

            let status = std::process::Command::new(editor_cmd)
                .arg(&req.path)
                .status();

            execute!(terminal.backend_mut(), EnterAlternateScreen)?;
            terminal::enable_raw_mode()?;
            terminal.hide_cursor()?;
            terminal.clear()?;

            if status.is_ok() {
                if req.task_id.is_empty() {
                    // Plan edit — file was edited in place, just reload plans
                    app.reload_plans();
                } else {
                    app.handle_editor_result(&req.task_id, &req.path, &req.list_id);
                }
            } else if !req.task_id.is_empty() {
                let _ = std::fs::remove_file(&req.path);
            }
            continue;
        }

        // Collect events: wait for first, then drain pending
        let mut had_fs_change = false;
        let mut key_events = Vec::new();

        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(AppEvent::Terminal(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                key_events.push(key);
            }
            Ok(AppEvent::FsChange) => had_fs_change = true,
            Ok(AppEvent::Terminal(_)) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
        }

        // Drain any additional pending events
        loop {
            match rx.try_recv() {
                Ok(AppEvent::Terminal(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                    key_events.push(key);
                }
                Ok(AppEvent::FsChange) => had_fs_change = true,
                Ok(AppEvent::Terminal(_)) => {}
                Err(_) => break,
            }
        }

        if had_fs_change {
            app.reload_tasks();
        }

        for key in key_events {
            app.handle_key(key);
            if app.should_quit {
                return Ok(());
            }
        }
    }
}
