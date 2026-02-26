#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

pub mod icon;
pub mod sound;

use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::process::Command;

use serde::Deserialize;

#[derive(Deserialize)]
struct HookPayload {
    title: Option<String>,
    message: Option<String>,
    notification_type: Option<String>,
}

type ParsedFields = (Option<String>, Option<String>, Option<String>);

pub struct TypeMapping {
    pub sound: &'static str,
    pub symbol: &'static str,
    pub message: &'static str,
    pub color: &'static str,
}

pub fn map_notification_type(notification_type: Option<&str>) -> TypeMapping {
    match notification_type {
        Some("permission_prompt") => TypeMapping {
            sound: "Frog",
            symbol: "lock",
            message: "Permission required",
            color: "#e74c3c",
        },
        Some("idle_prompt") => TypeMapping {
            sound: "Frog",
            symbol: "chat",
            message: "Finished, waiting for your input",
            color: "#3498db",
        },
        Some("elicitation_dialog") => TypeMapping {
            sound: "Frog",
            symbol: "question",
            message: "I have some questions for you",
            color: "#f39c12",
        },
        _ => TypeMapping {
            sound: "Hero",
            symbol: "check",
            message: "Ready",
            color: "#2ecc71",
        },
    }
}

pub fn parse_hook(json: &str) -> Result<ParsedFields, String> {
    let payload: HookPayload =
        serde_json::from_str(json).map_err(|e| format!("JSON parse error: {e}"))?;
    Ok((payload.title, payload.message, payload.notification_type))
}

fn tmux_session() -> Option<String> {
    if std::env::var("TMUX").is_err() {
        return None;
    }
    let out = Command::new("tmux")
        .args(["display-message", "-p", "#S"])
        .output()
        .ok()?;
    out.status.success().then(|| {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    })?
}

/// Returns "session:window" target for switch-client, so clicking
/// a notification switches to the exact window (tab), not just the session.
fn tmux_target() -> Option<String> {
    if std::env::var("TMUX").is_err() {
        return None;
    }
    let out = Command::new("tmux")
        .args(["display-message", "-p", "#S:#I"])
        .output()
        .ok()?;
    out.status.success().then(|| {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    })?
}

fn ring_terminal_bell() {
    if let Ok(mut tty) = OpenOptions::new().write(true).open("/dev/tty") {
        let _ = tty.write_all(b"\x07");
    }
}

fn set_tmux_attention(session: &str) {
    let _ = Command::new("tmux")
        .args(["set-option", "-t", session, "@attention", "1"])
        .output();

    let home = std::env::var("HOME").unwrap_or_default();
    let script = format!("{home}/.config/tmux/scripts/session-list.sh");
    if let Ok(out) = Command::new(&script).output()
        && out.status.success()
    {
        let list = String::from_utf8_lossy(&out.stdout);
        let list = list.trim();
        if !list.is_empty() {
            let status_left = format!(" {list} ");
            let _ = Command::new("tmux")
                .args([
                    "set",
                    "-g",
                    "status-left",
                    &status_left,
                    ";",
                    "refresh-client",
                    "-S",
                ])
                .output();
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("CLAUDE_CODE_SUBAGENT").is_ok() {
        return Ok(());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let (title, subtitle, notification_type) =
        parse_hook(&input).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    let mapping = map_notification_type(notification_type.as_deref());
    let session = tmux_session();
    let target = tmux_target();
    let display_title = session.as_deref().unwrap_or("Claude Code");
    let display_subtitle = subtitle.as_deref().or(title.as_deref()).unwrap_or("");

    if let Some(sess) = session.as_deref() {
        set_tmux_attention(sess);
    }

    // Detect if terminal is focused (platform-specific)
    #[cfg(target_os = "linux")]
    let terminal_focused = {
        let term = std::env::var("CT_TERMINAL").unwrap_or_else(|_| "ghostty".to_string());
        linux::is_terminal_focused(&term)
    };
    #[cfg(target_os = "macos")]
    let terminal_focused = false; // macOS handles this inside macos::notify

    // Skip bell and sound if terminal is focused and viewing this session
    let skip_for_focus = terminal_focused
        && session.as_deref().is_some_and(|s| {
            #[cfg(target_os = "linux")]
            {
                linux::is_session_active(s)
            }
            #[cfg(target_os = "macos")]
            {
                macos::is_session_active(s)
            }
        });

    if !skip_for_focus {
        ring_terminal_bell();
        if !terminal_focused {
            sound::play(notification_type.as_deref());
        }
    }

    let icon_sess = session.as_deref().unwrap_or("default");
    let icon_color = session
        .as_deref()
        .and_then(icon::tmux_session_color)
        .unwrap_or_else(|| mapping.color.to_string());
    let icon_path = icon::generate(&icon_color, mapping.symbol, icon_sess);

    #[cfg(target_os = "linux")]
    {
        let _ = linux::notify(
            session.as_deref(),
            target.as_deref(),
            display_subtitle,
            mapping.message,
            icon_path.as_deref(),
            terminal_focused,
        );
    }

    #[cfg(target_os = "macos")]
    {
        let icon_str = icon_path.as_deref().and_then(|p| p.to_str());
        let _ = macos::notify(
            session.as_deref(),
            display_subtitle,
            mapping.sound,
            icon_str,
        );
    }

    println!("title={display_title}");
    println!("subtitle={display_subtitle}");
    println!("message={}", mapping.message);
    println!("sound={}", mapping.sound);
    println!("symbol={}", mapping.symbol);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hook_idle_prompt() {
        let json = r#"{"notification_type":"idle_prompt","title":"Claude","message":"Done"}"#;
        let (title, message, ntype) = parse_hook(json).unwrap();
        assert_eq!(title.as_deref(), Some("Claude"));
        assert_eq!(message.as_deref(), Some("Done"));
        assert_eq!(ntype.as_deref(), Some("idle_prompt"));
    }

    #[test]
    fn parse_hook_missing_fields_gives_none() {
        let json = r#"{}"#;
        let (title, message, ntype) = parse_hook(json).unwrap();
        assert!(title.is_none());
        assert!(message.is_none());
        assert!(ntype.is_none());
    }

    #[test]
    fn parse_hook_invalid_json_returns_err() {
        let result = parse_hook("not json");
        assert!(result.is_err());
    }

    #[test]
    fn map_permission_prompt() {
        let m = map_notification_type(Some("permission_prompt"));
        assert_eq!(m.sound, "Frog");
        assert_eq!(m.symbol, "lock");
        assert_eq!(m.message, "Permission required");
        assert_eq!(m.color, "#e74c3c");
    }

    #[test]
    fn map_idle_prompt() {
        let m = map_notification_type(Some("idle_prompt"));
        assert_eq!(m.sound, "Frog");
        assert_eq!(m.symbol, "chat");
        assert_eq!(m.message, "Finished, waiting for your input");
        assert_eq!(m.color, "#3498db");
    }

    #[test]
    fn map_elicitation_dialog() {
        let m = map_notification_type(Some("elicitation_dialog"));
        assert_eq!(m.sound, "Frog");
        assert_eq!(m.symbol, "question");
        assert_eq!(m.message, "I have some questions for you");
        assert_eq!(m.color, "#f39c12");
    }

    #[test]
    fn map_unknown_type_falls_through_to_hero() {
        let m = map_notification_type(Some("something_else"));
        assert_eq!(m.sound, "Hero");
        assert_eq!(m.symbol, "check");
        assert_eq!(m.message, "Ready");
        assert_eq!(m.color, "#2ecc71");
    }

    #[test]
    fn map_none_type_falls_through_to_hero() {
        let m = map_notification_type(None);
        assert_eq!(m.sound, "Hero");
        assert_eq!(m.symbol, "check");
        assert_eq!(m.message, "Ready");
        assert_eq!(m.color, "#2ecc71");
    }

    #[test]
    fn each_type_has_distinct_color() {
        let types = [
            map_notification_type(Some("permission_prompt")),
            map_notification_type(Some("idle_prompt")),
            map_notification_type(Some("elicitation_dialog")),
            map_notification_type(None),
        ];
        let colors: Vec<&str> = types.iter().map(|m| m.color).collect();
        for (i, c1) in colors.iter().enumerate() {
            for c2 in &colors[i + 1..] {
                assert_ne!(c1, c2, "notification types should have distinct colors");
            }
        }
    }
}
