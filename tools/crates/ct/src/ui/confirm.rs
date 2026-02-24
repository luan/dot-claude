use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::ui::theme;

pub struct ConfirmState {
    pub task_id: String,
    pub action: String,
}

impl ConfirmState {
    pub fn new(task_id: String, action: &str) -> Self {
        Self {
            task_id,
            action: action.to_string(),
        }
    }
}

pub fn render_confirm(f: &mut Frame, area: Rect, state: &ConfirmState) {
    let lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{} task #{}?", state.action, state.task_id),
                theme::section_style(),
            ),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("y", theme::help_key_style()),
            Span::raw(" confirm  "),
            Span::styled("n", theme::help_key_style()),
            Span::raw(" cancel"),
        ]),
        Line::raw(""),
    ];

    f.render_widget(Paragraph::new(lines), area);
}
