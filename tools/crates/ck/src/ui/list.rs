use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Row, Table, TableState};

use crate::store::{SortOrder, StatusFilter, Task, filter_and_sort, tree_order};
use crate::ui::theme;

pub struct ListState {
    pub tasks: Vec<Task>,
    pub filtered: Vec<Task>,
    pub table_state: TableState,
    pub status_filter: StatusFilter,
    pub sort_order: SortOrder,
    pub show_closed: bool,
    pub tree_view: bool,
    pub searching: bool,
    pub query: String,
    pub search_input: String,
}

impl ListState {
    pub fn new(tasks: Vec<Task>) -> Self {
        let filtered = filter_and_sort(&tasks, StatusFilter::Active, SortOrder::Id, false, "");
        let mut table_state = TableState::default();
        if !filtered.is_empty() {
            table_state.select(Some(0));
        }
        Self {
            tasks,
            filtered,
            table_state,
            status_filter: StatusFilter::Active,
            sort_order: SortOrder::Id,
            show_closed: false,
            tree_view: false,
            searching: false,
            query: String::new(),
            search_input: String::new(),
        }
    }

    pub fn rebuild(&mut self) {
        self.filtered = filter_and_sort(
            &self.tasks,
            self.status_filter,
            self.sort_order,
            self.show_closed,
            &self.query,
        );
        // Clamp selection
        if let Some(i) = self.table_state.selected() {
            if i >= self.filtered.len() {
                self.table_state.select(if self.filtered.is_empty() {
                    None
                } else {
                    Some(self.filtered.len() - 1)
                });
            }
        } else if !self.filtered.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    pub fn selected_task(&self) -> Option<&Task> {
        self.table_state
            .selected()
            .and_then(|i| self.filtered.get(i))
    }

    pub fn selected_id(&self) -> Option<String> {
        self.selected_task().map(|t| t.id.clone())
    }

    pub fn next(&mut self) {
        let len = self.filtered.len();
        if len == 0 {
            return;
        }
        let i = self
            .table_state
            .selected()
            .map_or(0, |i| (i + 1).min(len - 1));
        self.table_state.select(Some(i));
    }

    pub fn prev(&mut self) {
        let i = self
            .table_state
            .selected()
            .map_or(0, |i| i.saturating_sub(1));
        self.table_state.select(Some(i));
    }
}

pub fn render_list(f: &mut Frame, area: Rect, state: &mut ListState) {
    let tasks = if state.tree_view {
        let rows = tree_order(&state.filtered);
        rows.into_iter()
            .map(|r| {
                let mut t = r.task;
                if r.depth == 1 {
                    let prefix = if r.is_last {
                        "└── "
                    } else {
                        "├── "
                    };
                    t.subject = format!("{prefix}{}", t.subject);
                }
                t
            })
            .collect::<Vec<_>>()
    } else {
        state.filtered.clone()
    };

    let header = Row::new(vec!["ID", "Status", "Pri", "Type", "Subject"])
        .style(
            Style::default()
                .fg(theme::SUBTEXT)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(0);

    let rows: Vec<Row> = tasks
        .iter()
        .map(|t| {
            Row::new(vec![
                Cell::from(Span::styled(&*t.id, Style::default().fg(theme::LAVENDER))),
                Cell::from(Span::styled(
                    t.status.as_str(),
                    theme::status_style(&t.status),
                )),
                Cell::from(Span::styled(
                    t.priority.as_str(),
                    theme::priority_style(&t.priority),
                )),
                Cell::from(Span::styled(
                    if t.task_type.is_empty() {
                        "--"
                    } else {
                        &t.task_type
                    },
                    Style::default().fg(theme::type_color(&t.task_type)),
                )),
                Cell::from(Span::raw(&t.subject)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Length(12),
        Constraint::Length(4),
        Constraint::Length(10),
        Constraint::Fill(1),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(theme::selected_style())
        .column_spacing(1);

    f.render_stateful_widget(table, area, &mut state.table_state);
}

pub fn render_filter_bar(f: &mut Frame, area: Rect, state: &ListState) {
    let mut spans = Vec::new();

    // Status filter
    if state.status_filter != StatusFilter::All {
        spans.push(Span::styled(
            format!(" {} ", state.status_filter.label()),
            theme::filter_tag_style(),
        ));
        spans.push(Span::raw(" "));
    }

    // Sort order
    if state.sort_order != SortOrder::Id {
        spans.push(Span::styled(
            format!(" sort:{} ", state.sort_order.label()),
            theme::filter_tag_style(),
        ));
        spans.push(Span::raw(" "));
    }

    // Tree view
    if state.tree_view {
        spans.push(Span::styled(" tree ", theme::filter_tag_style()));
        spans.push(Span::raw(" "));
    }

    // Show closed
    if state.show_closed {
        spans.push(Span::styled(" +closed ", theme::filter_tag_style()));
        spans.push(Span::raw(" "));
    }

    // Search
    if state.searching {
        spans.push(Span::styled("/", Style::default().fg(theme::ACCENT)));
        spans.push(Span::raw(&state.search_input));
        spans.push(Span::styled("█", Style::default().fg(theme::ACCENT)));
        spans.push(Span::raw(" "));
    } else if !state.query.is_empty() {
        spans.push(Span::styled(
            format!(" search:{} ", state.query),
            theme::filter_tag_style(),
        ));
        spans.push(Span::raw(" "));
    }

    // Count
    let display_count = if state.tree_view {
        tree_order(&state.filtered).len()
    } else {
        state.filtered.len()
    };
    spans.push(Span::styled(
        format!("{display_count} tasks"),
        theme::muted_style(),
    ));

    let line = Line::from(spans);
    f.render_widget(line, area);
}
