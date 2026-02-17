mod app;
mod editor;
mod plan;
mod store;
mod ui;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use std::io;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let editor_cmd = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    let mut app = app::App::new(store, lists, tasks, active_list);

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
