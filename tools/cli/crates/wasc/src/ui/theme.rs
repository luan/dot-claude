use crate::store::{Priority, Status};
use ratatui::style::{Color, Modifier, Style};

// Catppuccin Mocha palette
pub const BASE: Color = Color::Rgb(30, 30, 46);
pub const SURFACE: Color = Color::Rgb(49, 50, 68);
pub const OVERLAY: Color = Color::Rgb(69, 71, 90);

pub const TEXT: Color = Color::Rgb(205, 214, 244);
pub const SUBTEXT: Color = Color::Rgb(166, 173, 200);
pub const MUTED: Color = Color::Rgb(108, 112, 134);

pub const ACCENT: Color = Color::Rgb(137, 180, 250);
pub const ACCENT_DIM: Color = Color::Rgb(116, 199, 236);
pub const GREEN: Color = Color::Rgb(166, 227, 161);
pub const YELLOW: Color = Color::Rgb(249, 226, 175);
pub const ORANGE: Color = Color::Rgb(250, 179, 135);
pub const RED: Color = Color::Rgb(243, 139, 168);
pub const LAVENDER: Color = Color::Rgb(180, 190, 254);

pub fn status_style(status: &Status) -> Style {
    match status {
        Status::Pending => Style::default().fg(MUTED),
        Status::InProgress => Style::default().fg(ACCENT),
        Status::Completed => Style::default().fg(GREEN),
        Status::Other(_) => Style::default().fg(TEXT),
    }
}

pub fn priority_style(priority: &Priority) -> Style {
    match priority {
        Priority::P1 => Style::default().fg(RED),
        Priority::P2 => Style::default().fg(YELLOW),
        Priority::P3 => Style::default().fg(ACCENT),
        Priority::None => Style::default().fg(MUTED),
    }
}

pub fn type_color(t: &str) -> Color {
    match t {
        "epic" => LAVENDER,
        "feature" => ACCENT_DIM,
        "bug" => ORANGE,
        "chore" => SUBTEXT,
        "explore" => ACCENT,
        "phase" => MUTED,
        _ => TEXT,
    }
}

pub fn header_style() -> Style {
    Style::default()
        .fg(BASE)
        .bg(ACCENT)
        .add_modifier(Modifier::BOLD)
}

pub fn header_dim_style() -> Style {
    Style::default().fg(OVERLAY).bg(ACCENT)
}

pub fn footer_style() -> Style {
    Style::default().fg(SUBTEXT).bg(SURFACE)
}

pub fn status_msg_style() -> Style {
    Style::default().fg(GREEN).bg(SURFACE)
}

pub fn selected_style() -> Style {
    Style::default()
        .fg(TEXT)
        .bg(OVERLAY)
        .add_modifier(Modifier::BOLD)
}

pub fn help_key_style() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn section_style() -> Style {
    Style::default().fg(LAVENDER).add_modifier(Modifier::BOLD)
}

pub fn label_style() -> Style {
    Style::default().fg(SUBTEXT).add_modifier(Modifier::BOLD)
}

pub fn value_style() -> Style {
    Style::default().fg(TEXT)
}

pub fn muted_style() -> Style {
    Style::default().fg(MUTED)
}

pub fn filter_tag_style() -> Style {
    Style::default()
        .fg(ACCENT)
        .bg(OVERLAY)
        .add_modifier(Modifier::BOLD)
}
