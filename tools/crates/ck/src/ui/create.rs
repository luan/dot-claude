use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::store::Priority;
use crate::ui::theme;

const FIELD_NAMES: [&str; 5] = ["Subject", "Description", "Priority", "Type", "Parent"];
const PLACEHOLDERS: [&str; 5] = [
    "Task subject",
    "Short description",
    "P1, P2, or P3",
    "epic|feature|chore|bug|explore",
    "parent task ID (optional)",
];

pub struct CreateState {
    pub fields: [String; 5],
    pub focus: usize,
}

impl CreateState {
    pub fn new() -> Self {
        let mut fields: [String; 5] = Default::default();
        fields[2] = "P2".to_string(); // default priority
        Self { fields, focus: 0 }
    }

    pub fn next_field(&mut self) {
        self.focus = (self.focus + 1) % 5;
    }

    pub fn prev_field(&mut self) {
        self.focus = (self.focus + 4) % 5;
    }

    pub fn type_char(&mut self, c: char) {
        self.fields[self.focus].push(c);
    }

    pub fn backspace(&mut self) {
        self.fields[self.focus].pop();
    }

    pub fn subject(&self) -> &str {
        self.fields[0].trim()
    }

    pub fn description(&self) -> &str {
        self.fields[1].trim()
    }

    pub fn priority(&self) -> Priority {
        Priority::from_str(self.fields[2].trim())
    }

    pub fn task_type(&self) -> &str {
        self.fields[3].trim()
    }

    pub fn parent_id(&self) -> &str {
        self.fields[4].trim()
    }
}

pub fn render_create(f: &mut Frame, area: Rect, state: &CreateState) {
    let mut lines = vec![Line::raw("")];

    for (i, name) in FIELD_NAMES.iter().enumerate() {
        let (indicator, label_style) = if i == state.focus {
            (
                Span::styled("▸ ", Style::default().fg(theme::ACCENT)),
                Style::default().fg(theme::ACCENT),
            )
        } else {
            (Span::raw("  "), theme::muted_style())
        };

        let value = if state.fields[i].is_empty() {
            Span::styled(PLACEHOLDERS[i], theme::muted_style())
        } else {
            Span::styled(&state.fields[i], theme::value_style())
        };

        let cursor = if i == state.focus {
            Span::styled("█", Style::default().fg(theme::ACCENT))
        } else {
            Span::raw("")
        };

        lines.push(Line::from(vec![
            Span::raw("  "),
            indicator,
            Span::styled(format!("{name:<14}"), label_style),
            value,
            cursor,
        ]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "tab/shift-tab: navigate  ctrl+d: submit  esc: cancel",
            theme::muted_style(),
        ),
    ]));

    f.render_widget(Paragraph::new(lines), area);
}
