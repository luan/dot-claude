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
            ("j/k", "move up / down"),
            ("enter", "open task detail"),
            ("esc", "back"),
            ("q", "quit"),
        ],
    },
    Section {
        title: "Tabs",
        keys: &[
            ("tab / 2", "switch to Plans tab"),
            ("1", "switch to Tasks tab"),
        ],
    },
    Section {
        title: "Actions (list / detail)",
        keys: &[
            ("s", "change status"),
            ("p / a / d", "pending / active / done"),
            ("e", "edit in $EDITOR"),
            ("n", "new task"),
            ("D", "delete task"),
            ("x", "expand / collapse description"),
        ],
    },
    Section {
        title: "Detail",
        keys: &[
            ("j/k", "scroll"),
            ("space / b", "page down / up"),
            ("p", "browse plans for project"),
        ],
    },
    Section {
        title: "Filters (list)",
        keys: &[
            ("f", "cycle status filter"),
            ("A", "toggle completed"),
            ("o", "cycle sort order"),
            ("T", "toggle tree view"),
            ("F", "clear all filters"),
            ("/", "search by subject"),
        ],
    },
    Section {
        title: "Tree view",
        keys: &[
            ("space", "toggle collapse / expand"),
            (">", "expand selected node"),
            ("<", "collapse selected node"),
            ("zM", "collapse all"),
            ("zR", "expand all"),
        ],
    },
    Section {
        title: "Plans tab",
        keys: &[
            ("j/k", "move up / down"),
            ("enter", "open plan"),
            ("A", "toggle archived"),
            ("/", "search"),
        ],
    },
    Section {
        title: "Other",
        keys: &[
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
