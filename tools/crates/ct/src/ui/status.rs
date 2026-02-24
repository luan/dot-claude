use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::store::Status;
use crate::ui::theme;

pub struct StatusPickerState {
    pub task_id: String,
    pub options: Vec<Status>,
    pub cursor: usize,
}

impl StatusPickerState {
    pub fn new(task_id: String, current: &Status) -> Self {
        let all = vec![Status::Pending, Status::InProgress, Status::Completed];
        let options: Vec<Status> = all.into_iter().filter(|s| s != current).collect();
        Self {
            task_id,
            options,
            cursor: 0,
        }
    }

    pub fn next(&mut self) {
        if self.cursor < self.options.len() - 1 {
            self.cursor += 1;
        }
    }

    pub fn prev(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn selected(&self) -> &Status {
        &self.options[self.cursor]
    }
}

pub fn render_status_picker(f: &mut Frame, area: Rect, state: &StatusPickerState) {
    let mut lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Change status", theme::section_style()),
        ]),
        Line::raw(""),
    ];

    for (i, status) in state.options.iter().enumerate() {
        let style = if i == state.cursor {
            theme::selected_style()
        } else {
            theme::status_style(status)
        };
        let indicator = if i == state.cursor { "▸ " } else { "  " };
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{indicator}{}", status.as_str()), style),
        ]));
    }

    lines.push(Line::raw(""));

    f.render_widget(Paragraph::new(lines), area);
}
