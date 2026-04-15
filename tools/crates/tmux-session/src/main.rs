use std::collections::HashSet;
use std::env;
use std::process::{Command, Stdio};

mod chooser;
mod color;
mod filter;
mod group;
mod logging;
mod order;
mod palette;
mod picker;
mod process;
mod project;
mod sidebar;
mod status;
mod tmux;
mod usage;
mod usage_bars;

use color::compute_color;
use group::GroupMeta;
use order::compute_order;
use picker::{TextInputAction, TextInputConfig, run_text_input};
use project::{rename_parts, rename_session};
use status::{render_bar, render_windows};
use tmux::{query_state, query_system_info, query_windows, tmux as tmux_cmd};

fn cmd_order(args: &[String]) {
    let include_all = args.iter().any(|a| a == "--all");
    let alive: HashSet<String> = tmux_cmd(&["list-sessions", "-F", "#S"])
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    for s in compute_order(&alive, include_all) {
        println!("{s}");
    }
}

fn cmd_update_with_args(args: &[String]) {
    let st = query_state();
    let current = args
        .first()
        .filter(|s| !s.is_empty())
        .map_or(&st.current, |s| s);
    let client_width: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(200);

    let sessions = compute_order(&st.alive, false);
    let meta = GroupMeta::new(&sessions);

    let pre_colors: Vec<(String, String)> = {
        let c = status::compute_all_colors(&sessions, &meta);
        c.iter()
            .map(|(n, col, _)| (n.clone(), col.clone()))
            .collect()
    };
    let cur_color = pre_colors
        .iter()
        .find(|(n, _)| n == current)
        .map_or("#FFFFFF", |(_, c)| c.as_str());

    let bar = render_bar(&sessions, current, &meta, &st.attn, client_width);
    let windows = query_windows();
    let win_str = render_windows(&windows, cur_color);

    // If the sidebar is open, hide the session list in the status bar
    let sidebar_open = tmux_cmd(&["show-option", "-gv", "@sidebar_open"]) == "1";
    let left = if sidebar_open { "" } else { bar.left.as_str() };

    // Build status-format[0]: left=sessions, centre=windows, right=system-info
    let _status_fmt = format!(
        "#[align=left]{left}#[align=centre]{win}#[align=right]#(tmux-session system-info)",
        left = left,
        win = win_str,
    );

    let mut tmux_args: Vec<String> = vec![
        "set-option".into(),
        "-t".into(),
        current.clone(),
        "-u".into(),
        "@attention".into(),
    ];
    for (name, color) in &bar.colors {
        tmux_args.extend([
            ";".into(),
            "set-option".into(),
            "-t".into(),
            name.clone(),
            "@session_color".into(),
            color.clone(),
        ]);
    }
    tmux_args.extend([
        ";".into(),
        "set".into(),
        "-t".into(),
        current.clone(),
        "@session_color".into(),
        cur_color.into(),
    ]);
    tmux_args.extend([";".into(), "refresh-client".into(), "-S".into()]);

    let refs: Vec<&str> = tmux_args.iter().map(String::as_str).collect();
    tmux_cmd(&refs);

}

fn cmd_list() {
    let st = query_state();
    let sessions = compute_order(&st.alive, false);
    let meta = GroupMeta::new(&sessions);
    let bar = render_bar(&sessions, &st.current, &meta, &st.attn, 200);
    print!("{}", bar.left);

    if !bar.colors.is_empty() {
        let mut args: Vec<String> = Vec::new();
        for (i, (name, color)) in bar.colors.iter().enumerate() {
            if i > 0 {
                args.push(";".into());
            }
            args.extend([
                "set-option".into(),
                "-t".into(),
                name.clone(),
                "@session_color".into(),
                color.clone(),
            ]);
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        tmux_cmd(&refs);
    }
}

fn cmd_click(args: &[String]) {
    let range = args.first().map_or("", String::as_str);
    if let Some(session) = range.strip_prefix("s:") {
        tmux_cmd(&["switch-client", "-t", session]);
    } else if let Some(window) = range.strip_prefix("w:") {
        tmux_cmd(&["select-window", "-t", &format!(":{window}")]);
    } else if range == "caffeine" {
        toggle_caffeine();
    }
}

#[cfg(target_os = "macos")]
fn toggle_caffeine() {
    let running = Command::new("pgrep")
        .args(["-x", "caffeinate"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());

    if running {
        let _ = Command::new("pkill").args(["-x", "caffeinate"]).status();
    } else {
        let _ = Command::new("caffeinate")
            .args(["-di"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
    tmux_cmd(&["refresh-client", "-S"]);
}

#[cfg(not(target_os = "macos"))]
fn toggle_caffeine() {}

fn cmd_system_info() {
    let system = query_system_info();
    print!("{}", status::render_system_info(&system));
}

fn cmd_color(args: &[String]) {
    let mut mode = "color";
    let (mut pos, mut total, mut gpos, mut gtotal) = (0, 0, 0, 0);
    let mut i = 0;
    while i < args.len().saturating_sub(1) {
        match args[i].as_str() {
            "--dim" => {
                mode = "dim";
                i += 1;
            }
            "--both" => {
                mode = "both";
                i += 1;
            }
            "--pos" => {
                pos = args[i + 1].parse().unwrap_or(0);
                i += 2;
            }
            "--total" => {
                total = args[i + 1].parse().unwrap_or(0);
                i += 2;
            }
            "--group-pos" => {
                gpos = args[i + 1].parse().unwrap_or(0);
                i += 2;
            }
            "--group-total" => {
                gtotal = args[i + 1].parse().unwrap_or(0);
                i += 2;
            }
            _ => break,
        }
    }
    let name = args.last().map_or("", String::as_str);
    let (c, d) = compute_color(name, pos, total, gpos, gtotal);
    match mode {
        "dim" => println!("{d}"),
        "both" => println!("{c}\t{d}"),
        _ => println!("{c}"),
    }
}

fn cmd_switch(args: &[String]) {
    let direction = args.first().map_or("next", String::as_str);
    let st = query_state();
    let sessions = compute_order(&st.alive, false);
    if sessions.is_empty() {
        return;
    }
    let idx = sessions.iter().position(|s| s == &st.current);
    let target = match (idx, direction) {
        (Some(i), "prev") => &sessions[(i + sessions.len() - 1) % sessions.len()],
        (Some(i), _) => &sessions[(i + 1) % sessions.len()],
        (None, "prev") => sessions.last().unwrap(),
        (None, _) => &sessions[0],
    };
    tmux_cmd(&["switch-client", "-t", target]);
}

fn cmd_move(args: &[String]) {
    let direction = args.first().map_or("", String::as_str);
    let st = query_state();
    let current = if args.len() > 1 {
        args[1].clone()
    } else {
        st.current.clone()
    };

    let mut store = order::SessionStore::load();
    store.prune(&st.alive);
    if store.move_session(&current, direction) {
        store.save();
    }
    // Fork the status-bar refresh into background so tmux unblocks immediately
    // (allows rapid repeated moves without losing keypresses to serialization)
    let exe = env::current_exe().unwrap_or_else(|_| "tmux-session".into());
    let _ = Command::new(exe)
        .args(["update"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn cmd_rename(args: &[String]) {
    let old_name = args
        .first()
        .filter(|s| !s.is_empty())
        .cloned()
        .unwrap_or_else(|| tmux_cmd(&["display-message", "-p", "#S"]));

    let (prefix, suffix) = rename_parts(&old_name);

    let new_suffix = match run_text_input(TextInputConfig {
        prompt: "\u{f044}  Rename".to_string(),
        initial: suffix.clone(),
        placeholder: "session name...".to_string(),
        prefix: prefix.clone(),
    }) {
        TextInputAction::Confirmed(s) => s.trim().to_string(),
        TextInputAction::Cancelled => return,
    };

    if new_suffix.is_empty() {
        return;
    }

    let new_name = format!("{prefix}{new_suffix}");
    let _ = rename_session(&old_name, &new_name);
}

fn cmd_select(args: &[String]) {
    let index: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    if index == 0 {
        return;
    }
    let st = query_state();
    let sessions = compute_order(&st.alive, false);
    if let Some(target) = sessions.get(index - 1) {
        tmux_cmd(&["switch-client", "-t", target]);
    }
}

fn cmd_attention() {
    let st = query_state();
    if let Some(target) = st
        .attn
        .iter()
        .find(|(_, v)| *v == "1")
        .map(|(k, _)| k.as_str())
    {
        tmux_cmd(&["switch-client", "-t", target]);
    }
}

fn cmd_hide_toggle(args: &[String]) {
    let session = match args.first() {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return,
    };
    let path = order::hidden_file();
    let mut lines = order::load_lines(&path);
    if let Some(pos) = lines.iter().position(|l| l == &session) {
        lines.remove(pos);
    } else {
        lines.push(session);
    }
    order::save_lines(&path, &lines);
}

fn cmd_picker(args: &[String]) {
    let action = args.first().map_or("chooser", String::as_str);

    // If sidebar is open, route the action into it via tmux send-keys
    if tmux_cmd(&["show-option", "-gv", "@sidebar_open"]) == "1" {
        // Find the sidebar pane: look for a pane running tmux-session
        let panes = tmux_cmd(&["list-panes", "-a", "-F", "#{pane_id}\t#{pane_current_command}"]);
        for line in panes.lines() {
            let parts: Vec<&str> = line.splitn(2, '\t').collect();
            if parts.len() == 2 && parts[1].contains("tmux-session") {
                let key = match action {
                    "rename" => "r",
                    "chooser" => "/",
                    "new-session" => "n",
                    "new-worktree" => "w",
                    "ditch" => "x",
                    _ => "/",
                };
                tmux_cmd(&["send-keys", "-t", parts[0], key, ""]);
                return;
            }
        }
    }

    // Sidebar not open or not found: fall back to popup
    let popup_args = match action {
        "rename" => vec!["display-popup", "-E", "-B", "-w", "50", "-h", "3",
                         "tmux-session", "rename"],
        "new-session" => vec!["display-popup", "-E", "-B", "-w", "60%", "-h", "70%",
                              "tmux-session", "new-session"],
        "ditch" => vec!["display-popup", "-E", "-B", "-w", "60%", "-h", "50%",
                        "tmux-session", "ditch"],
        _ => vec!["display-popup", "-E", "-B", "-w", "60%", "-h", "70%",
                  "tmux-session", "chooser"],
    };
    tmux_cmd(&popup_args);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map_or("list", String::as_str);
    let rest: Vec<String> = args.iter().skip(2).cloned().collect();
    match cmd {
        "order" => cmd_order(&rest),
        "list" => cmd_list(),
        "color" => cmd_color(&rest),
        "switch" => cmd_switch(&rest),
        "move" => cmd_move(&rest),
        "chooser-list" => chooser::cmd_chooser_list(),
        "chooser" => chooser::cmd_chooser(),
        "project-list" => project::cmd_project_list(&rest),
        "toggle-favorite" => project::cmd_toggle_favorite(&rest),
        "new-session" => project::cmd_new_session(),
        "new-worktree" => project::cmd_new_worktree(&rest),
        "ditch" => project::cmd_ditch(&rest),
        "rename" => cmd_rename(&rest),
        "select" => cmd_select(&rest),
        "attention" => cmd_attention(),
        "hide-toggle" => cmd_hide_toggle(&rest),
        "update" => cmd_update_with_args(&rest),
        "click" => cmd_click(&rest),
        "sidebar" => sidebar::cmd_sidebar(),
        "picker" => cmd_picker(&rest),
        "system-info" => cmd_system_info(),
        _ => {
            eprintln!("Unknown: {cmd}");
            std::process::exit(1);
        }
    }
}
