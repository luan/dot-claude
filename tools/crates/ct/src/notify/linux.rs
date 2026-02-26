use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::icon;

pub fn has_display() -> bool {
    std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok()
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}

fn default_icon() -> PathBuf {
    expand_tilde("~/.claude/claude.png")
}

fn id_file_for(session: &str) -> PathBuf {
    let safe = icon::sanitize_session(session);
    PathBuf::from(format!("/tmp/ct-notify-{safe}.id"))
}

fn read_replace_id(session: &str) -> Option<String> {
    std::fs::read_to_string(id_file_for(session))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Shell-escape a string for use inside single quotes.
/// Replaces ' with '\'' (end quote, escaped quote, start quote).
pub fn shell_escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}

fn terminal_name() -> String {
    std::env::var("CT_TERMINAL").unwrap_or_else(|_| "ghostty".to_string())
}

/// Returns the name of the currently focused window, if detectable.
/// Uses xdotool (X11 / XWayland). Returns None on native Wayland
/// compositors without XWayland — a known limitation; the safe default
/// is to always send the notification.
fn focused_window_name() -> Option<String> {
    Command::new("xdotool")
        .args(["getactivewindow", "getwindowname"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Returns true if the given terminal app appears to be the focused window.
/// Case-insensitive match against the window title.
pub fn is_terminal_focused(terminal_name: &str) -> bool {
    focused_window_name()
        .map(|name| name.to_lowercase().contains(&terminal_name.to_lowercase()))
        .unwrap_or(false)
}

/// Returns true if the tmux client's active session matches the given session name.
pub fn is_session_active(session: &str) -> bool {
    if std::env::var("TMUX").is_err() {
        return false;
    }

    let client_tty = Command::new("tmux")
        .args(["display-message", "-p", "#{client_tty}"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    let tty = match client_tty {
        Some(t) if !t.is_empty() => t,
        _ => return false,
    };

    Command::new("tmux")
        .args(["display-message", "-p", "-t", &tty, "#{client_session}"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim() == session)
        .unwrap_or(false)
}

pub fn notify(
    session: Option<&str>,
    target: Option<&str>,
    subtitle: &str,
    message: &str,
    icon_path: Option<&Path>,
    terminal_focused: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !has_display() {
        return Ok(());
    }

    // Skip entirely if terminal is focused and viewing this session
    if terminal_focused
        && let Some(sess) = session
        && is_session_active(sess)
    {
        return Ok(());
    }

    let summary = session.unwrap_or("Claude Code");
    let fallback = default_icon();
    let icon = icon_path.unwrap_or(&fallback);
    let icon_str = shell_escape(&icon.to_string_lossy());
    let body = format!("{subtitle}\n{message}");

    if let Some(sess) = session {
        // With session: dedup + click-to-focus.
        // Spawns a background process that blocks on notify-send --action,
        // surviving after ct exits. On click, switches tmux to the session:window.
        let switch_target = target.unwrap_or(sess);
        let safe_target = shell_escape(switch_target);
        let safe_summary = shell_escape(summary);
        let safe_body = shell_escape(&body);
        let id_file = id_file_for(sess);
        let id_file_str = shell_escape(&id_file.to_string_lossy());

        let mut replace_args = String::new();
        if let Some(prev_id) = read_replace_id(sess) {
            replace_args = format!("-r {prev_id} ");
        }

        let script = format!(
            concat!(
                "RESULT=$(notify-send -p {replace}",
                "--app-name='Claude Code' ",
                "--icon='{icon}' ",
                "--urgency=normal ",
                "--action=default=default ",
                "-- '{summary}' '{body}'); ",
                "NEW_ID=$(printf '%s' \"$RESULT\" | head -1); ",
                "ACTION=$(printf '%s' \"$RESULT\" | tail -1); ",
                "[ -n \"$NEW_ID\" ] && printf '%s' \"$NEW_ID\" > '{id_file}'; ",
                "if [ \"$ACTION\" = 'default' ]; then ",
                "  if command -v xdotool >/dev/null 2>&1; then ",
                "    WID=$(xdotool search --name '{terminal}' 2>/dev/null | head -1); ",
                "    [ -n \"$WID\" ] && xdotool windowactivate \"$WID\" 2>/dev/null; ",
                "  fi; ",
                "  tmux switch-client -t '{target}' 2>/dev/null; ",
                "  tmux select-window -t '{target}' 2>/dev/null; ",
                "fi"
            ),
            replace = replace_args,
            icon = icon_str,
            summary = safe_summary,
            body = safe_body,
            id_file = id_file_str,
            target = safe_target,
            terminal = shell_escape(&terminal_name()),
        );

        Command::new("sh")
            .args(["-c", &script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    } else {
        // No session: no dedup, but still offer click-to-focus terminal
        let safe_summary = shell_escape(summary);
        let safe_body = shell_escape(&body);

        let script = format!(
            concat!(
                "notify-send ",
                "--app-name='Claude Code' ",
                "--icon='{icon}' ",
                "--urgency=normal ",
                "--action=default=default ",
                "-- '{summary}' '{body}'; ",
                "if command -v xdotool >/dev/null 2>&1; then ",
                "  WID=$(xdotool search --name '{terminal}' 2>/dev/null | head -1); ",
                "  [ -n \"$WID\" ] && xdotool windowactivate \"$WID\" 2>/dev/null; ",
                "fi"
            ),
            icon = icon_str,
            summary = safe_summary,
            body = safe_body,
            terminal = shell_escape(&terminal_name()),
        );

        Command::new("sh")
            .args(["-c", &script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_display_reflects_env() {
        let expected = std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
        assert_eq!(has_display(), expected);
    }

    #[test]
    fn shell_escape_plain_string_unchanged() {
        assert_eq!(shell_escape("hello world"), "hello world");
    }

    #[test]
    fn shell_escape_single_quotes() {
        assert_eq!(shell_escape("it's"), "it'\\''s");
    }

    #[test]
    fn id_file_path_uses_sanitized_session() {
        let path = id_file_for("my.session/name");
        assert_eq!(path, PathBuf::from("/tmp/ct-notify-my_session_name.id"));
    }

    #[test]
    fn default_icon_is_absolute() {
        let icon = default_icon();
        assert!(icon.is_absolute(), "icon path should be absolute: {icon:?}");
    }

    #[test]
    fn expand_tilde_expands_home() {
        let path = expand_tilde("~/.claude/test");
        assert!(
            !path.to_string_lossy().starts_with('~'),
            "tilde should be expanded: {path:?}"
        );
        assert!(
            path.to_string_lossy().ends_with("/.claude/test"),
            "suffix should be preserved: {path:?}"
        );
    }

    #[test]
    fn expand_tilde_leaves_absolute_paths() {
        let path = expand_tilde("/usr/bin/test");
        assert_eq!(path, PathBuf::from("/usr/bin/test"));
    }
}
