use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::ui::theme;

struct Section {
    title: &'static str,
    keys: &'static [(&'static str, &'static str)],
}

const SECTIONS: &[Section] = &[
    Section {
        title: "Navigation",
        keys: &[
            ("j/k", "up / down"),
            ("enter", "open task"),
            ("esc", "back"),
            ("q", "quit"),
        ],
    },
    Section {
        title: "Actions",
        keys: &[
            ("s", "change status"),
            ("p/a/d", "pending / active / done"),
            ("e", "edit in $EDITOR"),
            ("n", "new task"),
            ("D", "delete task"),
        ],
    },
    Section {
        title: "Filters (list)",
        keys: &[
            ("f", "cycle status filter"),
            ("A", "toggle completed"),
            ("o", "cycle sort order"),
            ("T", "toggle tree view"),
            ("F", "clear filters"),
            ("/", "search by subject"),
        ],
    },
    Section {
        title: "Tree view",
        keys: &[
            ("space/tab", "collapse / expand node"),
            ("zM", "collapse all"),
            ("zR", "expand all"),
        ],
    },
    Section {
        title: "Other",
        keys: &[
            ("P", "browse plans"),
            ("L", "switch task list"),
            ("R", "reload from disk"),
            ("?", "toggle this help"),
        ],
    },
];

pub fn render_help(f: &mut Frame, area: Rect) {
    let mut lines = Vec::new();

    for (si, section) in SECTIONS.iter().enumerate() {
        if si > 0 {
            lines.push(Line::raw(""));
        }
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(section.title, theme::section_style()),
        ]));

        for (key, desc) in section.keys {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("{key:<12}"), theme::help_key_style()),
                Span::styled(*desc, Style::default().fg(theme::SUBTEXT)),
            ]));
        }
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("press ? to close", theme::muted_style()),
    ]));

    f.render_widget(Paragraph::new(lines), area);
}

use ratatui::style::Style;
