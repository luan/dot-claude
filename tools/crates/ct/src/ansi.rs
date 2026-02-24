use std::sync::OnceLock;

use crate::store::{Priority, Status};

fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("NO_COLOR").is_none())
}

// Catppuccin Mocha palette (shared with ui/theme.rs)
fn rgb(r: u8, g: u8, b: u8, text: &str) -> String {
    if !enabled() {
        return text.to_string();
    }
    format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[0m")
}

fn bold_rgb(r: u8, g: u8, b: u8, text: &str) -> String {
    if !enabled() {
        return text.to_string();
    }
    format!("\x1b[1;38;2;{r};{g};{b}m{text}\x1b[0m")
}

// --- Semantic style API (mirrors work's internal/style) ---

/// Identifiers — task IDs, project slugs (lavender)
pub fn id(text: &str) -> String {
    rgb(180, 190, 254, text)
}

/// Field labels — bold subtext
pub fn label(text: &str) -> String {
    bold_rgb(166, 173, 200, text)
}

/// Secondary info — timestamps, metadata (muted)
pub fn dim(text: &str) -> String {
    rgb(108, 112, 134, text)
}

/// Section headers — bold lavender
pub fn section(text: &str) -> String {
    bold_rgb(180, 190, 254, text)
}

/// Bold text — table headers
pub fn bold(text: &str) -> String {
    if !enabled() {
        return text.to_string();
    }
    format!("\x1b[1m{text}\x1b[0m")
}

/// Styled arrow for transitions
pub fn arrow() -> String {
    rgb(69, 71, 90, "→")
}

/// Status-colored text
pub fn for_status(status: &Status, text: &str) -> String {
    match status {
        Status::Pending => dim(text),
        Status::InProgress => rgb(137, 180, 250, text), // accent
        Status::Completed => rgb(166, 227, 161, text),  // green
        Status::Other(_) => text.to_string(),
    }
}

/// Priority-colored text
pub fn for_priority(priority: &Priority, text: &str) -> String {
    match priority {
        Priority::P1 => rgb(243, 139, 168, text), // red
        Priority::P2 => rgb(249, 226, 175, text), // yellow
        Priority::P3 => rgb(137, 180, 250, text), // accent
        Priority::None => dim(text),
    }
}

/// Type-colored text (matches ui/theme.rs type_color)
pub fn for_type(type_str: &str, text: &str) -> String {
    match type_str {
        "epic" => rgb(180, 190, 254, text),    // lavender
        "feature" => rgb(116, 199, 236, text), // accent_dim
        "bug" => rgb(250, 179, 135, text),     // orange
        "chore" => rgb(166, 173, 200, text),   // subtext
        "explore" => rgb(137, 180, 250, text), // accent
        "phase" => rgb(108, 112, 134, text),   // muted
        _ => rgb(205, 214, 244, text),         // text
    }
}

/// Blocked indicator
pub fn blocked(text: &str) -> String {
    rgb(243, 139, 168, text) // red
}
