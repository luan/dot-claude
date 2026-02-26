use std::path::Path;
use std::process::Command;

/// Returns true if the given app name is currently the frontmost application.
pub fn is_app_focused(app_name: &str) -> bool {
    let front = Command::new("lsappinfo")
        .args(["front"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok());

    let front_handle = match front {
        Some(h) => h.trim().to_string(),
        None => return false,
    };

    Command::new("lsappinfo")
        .args(["info", "-only", "name", &front_handle])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|output| parse_lsappinfo_name(&output).map(|n| n == app_name))
        .unwrap_or(false)
}

/// Parse the app name from lsappinfo output like: `"name"="Ghostty"`
pub fn parse_lsappinfo_name(output: &str) -> Option<&str> {
    for line in output.lines() {
        if let Some(eq_pos) = line.find('=') {
            let value = line[eq_pos + 1..].trim();
            // Strip surrounding quotes
            if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                return Some(&value[1..value.len() - 1]);
            }
        }
    }
    None
}

/// Returns true if the tmux client's active session matches the given session name.
/// Requires TMUX to be set.
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

fn home_dir() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(std::path::PathBuf::from)
}

/// Ensure Claude app is registered with grrr. Idempotent.
fn ensure_app_registered() {
    let Some(home) = home_dir() else { return };

    let app_dir = home.join(".growlrrr/apps/Claude.app");
    if !app_dir.exists() {
        let icon = home.join(".claude/claude.png");
        let _ = Command::new("grrr")
            .args([
                "apps",
                "add",
                "--appId",
                "Claude",
                "--appIcon",
                &icon.to_string_lossy(),
            ])
            .output();
    }
}

/// Builds the --execute command string for grrr, inlining focus-session.sh logic with absolute binary paths.
pub fn build_focus_command(session: &str) -> String {
    let tmux_bin = which_bin("tmux").unwrap_or_else(|| "/usr/local/bin/tmux".to_string());
    let grrr_bin = which_bin("grrr").unwrap_or_else(|| "/usr/local/bin/grrr".to_string());

    format!(
        "open -a Ghostty & {grrr_bin} clear 'claude-{session}' >/dev/null 2>&1 & \
         client=$({tmux_bin} list-clients -F '#{{client_tty}}' | head -1) ; \
         [ -n \"$client\" ] && {tmux_bin} switch-client -c \"$client\" -t '{session}'"
    )
}

fn which_bin(name: &str) -> Option<String> {
    Command::new("which")
        .arg(name)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Build and return the grrr Command with all required flags.
/// Does not execute it — caller decides whether to spawn.
pub fn build_grrr_command(
    session: Option<&str>,
    title: &str,
    subtitle: &str,
    sound: &str,
    icon_path: Option<&str>,
    ghostty_focused: bool,
) -> Command {
    let mut cmd = Command::new("grrr");

    cmd.args(["--appId", "Claude"]);
    cmd.args(["--title", title]);
    cmd.args(["--subtitle", subtitle]);

    if ghostty_focused {
        cmd.args(["--sound", "none"]);
    } else {
        cmd.args(["--sound", sound]);
    }

    if let Some(path) = icon_path {
        if Path::new(path).exists() {
            cmd.args(["--image", path]);
        }
    }

    if let Some(sess) = session {
        let thread_id = format!("claude-{sess}");
        cmd.args(["--threadId", &thread_id, "--identifier", &thread_id]);

        let focus_cmd = build_focus_command(sess);
        cmd.args(["--execute", &focus_cmd]);
    } else {
        cmd.args(["--execute", "open -a Ghostty"]);
    }

    cmd
}

/// Send a macOS notification via grrr.
///
/// Skips if all of these hold: TMUX is set, Ghostty is focused, and the
/// client's active session matches `session`.
pub fn notify(
    session: Option<&str>,
    subtitle: &str,
    sound: &str,
    icon_path: Option<&str>,
) -> Result<(), String> {
    let ghostty_focused = is_app_focused("Ghostty");

    if ghostty_focused {
        if let Some(sess) = session {
            if is_session_active(sess) {
                return Ok(());
            }
        }
    }

    ensure_app_registered();

    let title = session.unwrap_or("Claude Code");
    let mut cmd = build_grrr_command(session, title, subtitle, sound, icon_path, ghostty_focused);

    cmd.output().map_err(|e| format!("grrr failed: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lsappinfo_name_extracts_quoted_value() {
        let output = r#"
LSAppInfoItem 0x600003744540 "com.mitchellh.ghostty" (Ghostty)
"name"="Ghostty" "pid"=1234
"#;
        assert_eq!(parse_lsappinfo_name(output), Some("Ghostty"));
    }

    #[test]
    fn parse_lsappinfo_name_returns_none_for_unrecognized_format() {
        let output = "no equals sign here\n";
        assert_eq!(parse_lsappinfo_name(output), None);
    }

    #[test]
    fn parse_lsappinfo_name_returns_none_for_empty() {
        assert_eq!(parse_lsappinfo_name(""), None);
    }

    #[test]
    fn parse_lsappinfo_name_handles_unquoted_value() {
        let output = "\"name\"=Ghostty\n";
        assert_eq!(parse_lsappinfo_name(output), None);
    }

    #[test]
    fn build_focus_command_contains_session_name() {
        let cmd = build_focus_command("my-session");
        assert!(cmd.contains("claude-my-session"));
        assert!(cmd.contains("switch-client"));
        assert!(cmd.contains("open -a Ghostty"));
    }

    #[test]
    fn build_grrr_command_uses_none_sound_when_ghostty_focused() {
        let cmd = build_grrr_command(None, "Claude Code", "Ready", "Hero", None, true);
        let args: Vec<_> = cmd.get_args().collect();
        let args: Vec<&str> = args.iter().map(|a| a.to_str().unwrap()).collect();
        let sound_pos = args.iter().position(|&a| a == "--sound").unwrap();
        assert_eq!(args[sound_pos + 1], "none");
    }

    #[test]
    fn build_grrr_command_uses_provided_sound_when_not_focused() {
        let cmd = build_grrr_command(None, "Claude Code", "Ready", "Hero", None, false);
        let args: Vec<_> = cmd.get_args().collect();
        let args: Vec<&str> = args.iter().map(|a| a.to_str().unwrap()).collect();
        let sound_pos = args.iter().position(|&a| a == "--sound").unwrap();
        assert_eq!(args[sound_pos + 1], "Hero");
    }

    #[test]
    fn build_grrr_command_sets_thread_and_identifier_for_session() {
        let cmd = build_grrr_command(Some("work"), "work", "Done", "Hero", None, false);
        let args: Vec<_> = cmd.get_args().collect();
        let args: Vec<&str> = args.iter().map(|a| a.to_str().unwrap()).collect();
        assert!(args.contains(&"--threadId"));
        assert!(args.contains(&"claude-work"));
        assert!(args.contains(&"--identifier"));
    }

    #[test]
    fn build_grrr_command_no_thread_flags_when_no_session() {
        let cmd = build_grrr_command(None, "Claude Code", "Ready", "Hero", None, false);
        let args: Vec<_> = cmd.get_args().collect();
        let args: Vec<&str> = args.iter().map(|a| a.to_str().unwrap()).collect();
        assert!(!args.contains(&"--threadId"));
        assert!(!args.contains(&"--identifier"));
    }

    #[test]
    fn build_grrr_command_skips_image_flag_when_no_icon() {
        let cmd = build_grrr_command(None, "Claude Code", "Ready", "Hero", None, false);
        let args: Vec<_> = cmd.get_args().collect();
        let args: Vec<&str> = args.iter().map(|a| a.to_str().unwrap()).collect();
        assert!(!args.contains(&"--image"));
    }
}
