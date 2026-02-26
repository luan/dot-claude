use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
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
        title: "Actions",
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
        title: "Detail view",
        keys: &[("j/k", "scroll"), ("space / b", "page down / up")],
    },
    Section {
        title: "Filters",
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
        title: "Other",
        keys: &[
            ("L", "switch task list"),
            ("R", "reload from disk"),
            ("?", "toggle this help"),
        ],
    },
];

fn build_section_lines(section: &Section) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![Span::styled(
        section.title,
        theme::section_style(),
    )]));
    for (key, desc) in section.keys {
        lines.push(Line::from(vec![
            Span::styled(format!("  {key:<12}"), theme::help_key_style()),
            Span::styled(*desc, Style::default().fg(theme::SUBTEXT)),
        ]));
    }
    lines
}

fn render_sections(f: &mut Frame, area: Rect, sections: &[Section], scroll: u16) -> u16 {
    let section_heights: Vec<usize> = sections.iter().map(|s| 1 + s.keys.len()).collect();
    let total_lines: usize =
        section_heights.iter().sum::<usize>() + sections.len().saturating_sub(1);

    // Find split point that balances column heights
    let half = total_lines / 2;
    let mut left_height = 0;
    let mut split_at = sections.len();
    for (i, &h) in section_heights.iter().enumerate() {
        let with_gap = if i > 0 { h + 1 } else { h };
        if left_height + with_gap > half && i > 0 {
            split_at = i;
            break;
        }
        left_height += with_gap;
    }

    let left_sections = &sections[..split_at];
    let right_sections = &sections[split_at..];

    let mut left_lines = Vec::new();
    for (i, section) in left_sections.iter().enumerate() {
        if i > 0 {
            left_lines.push(Line::raw(""));
        }
        left_lines.extend(build_section_lines(section));
    }

    let mut right_lines = Vec::new();
    for (i, section) in right_sections.iter().enumerate() {
        if i > 0 {
            right_lines.push(Line::raw(""));
        }
        right_lines.extend(build_section_lines(section));
    }

    let max_height = left_lines.len().max(right_lines.len()) as u16;
    let visible = area.height;
    let max_scroll = max_height.saturating_sub(visible);
    let clamped = scroll.min(max_scroll);

    let [left_area, right_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);

    let left_inner = Rect {
        x: left_area.x + 2,
        width: left_area.width.saturating_sub(2),
        ..left_area
    };
    let right_inner = Rect {
        x: right_area.x + 1,
        width: right_area.width.saturating_sub(1),
        ..right_area
    };

    f.render_widget(Paragraph::new(left_lines).scroll((clamped, 0)), left_inner);
    f.render_widget(
        Paragraph::new(right_lines).scroll((clamped, 0)),
        right_inner,
    );

    clamped
}

pub fn render_help(f: &mut Frame, area: Rect, scroll: u16) -> u16 {
    render_sections(f, area, SECTIONS, scroll)
}
