use std::collections::{HashMap, HashSet};

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Cell, Row, Table, TableState};

use crate::store::{
    SortOrder, Status, StatusFilter, Task, filter_and_sort, tree_order, tree_prefix,
};
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
    pub collapsed: HashSet<String>,
    pub expanded_ids: HashSet<String>,
    pub pending_z: bool,
    pub visible_count: usize,
}

impl ListState {
    pub fn new(tasks: Vec<Task>) -> Self {
        let filtered = filter_and_sort(&tasks, StatusFilter::All, SortOrder::Id, false, "");
        let visible_count = filtered.len();
        let mut table_state = TableState::default();
        if !filtered.is_empty() {
            table_state.select(Some(0));
        }
        Self {
            tasks,
            filtered,
            table_state,
            status_filter: StatusFilter::All,
            sort_order: SortOrder::Id,
            show_closed: false,
            tree_view: false,
            searching: false,
            query: String::new(),
            search_input: String::new(),
            collapsed: HashSet::new(),
            expanded_ids: HashSet::new(),
            pending_z: false,
            visible_count,
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
        let len = if self.tree_view {
            self.visible_count
        } else {
            self.filtered.len()
        };
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
    let completed_ids: HashSet<&str> = state
        .tasks
        .iter()
        .filter(|t| t.status == Status::Completed)
        .map(|t| t.id.as_str())
        .collect();

    let tree_view = state.tree_view;
    let (display_tasks, dim_ids) = if tree_view {
        build_tree_display(&state.tasks, &state.filtered, &state.collapsed)
    } else {
        (state.filtered.clone(), HashSet::new())
    };

    // Update visible count and clamp selection
    state.visible_count = display_tasks.len();
    if let Some(i) = state.table_state.selected()
        && i >= display_tasks.len()
    {
        state.table_state.select(if display_tasks.is_empty() {
            None
        } else {
            Some(display_tasks.len() - 1)
        });
    }

    let header = Row::new(vec!["ID", "Status", "Pri", "Type", "Owner", "Subject"])
        .style(
            Style::default()
                .fg(theme::SUBTEXT)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(0);

    let expanded_ids = &state.expanded_ids;
    let rows: Vec<Row> = display_tasks
        .iter()
        .map(|t| {
            let active_blocker_ids: Vec<&str> = t
                .blocked_by
                .iter()
                .filter(|dep| !completed_ids.contains(dep.as_str()))
                .map(|s| s.as_str())
                .collect();
            let blocked = !active_blocker_ids.is_empty();
            let dim = dim_ids.contains(&t.id);

            let id_style = if dim {
                theme::muted_style()
            } else {
                Style::default().fg(theme::LAVENDER)
            };
            let status_style = if dim {
                theme::muted_style()
            } else if blocked {
                Style::default().fg(theme::ORANGE)
            } else {
                theme::status_style(&t.status)
            };
            let pri_style = if dim {
                theme::muted_style()
            } else {
                theme::priority_style(&t.priority)
            };
            let type_style = if dim {
                theme::muted_style()
            } else {
                Style::default().fg(theme::type_color(&t.task_type))
            };
            let owner_style = if dim {
                theme::muted_style()
            } else {
                Style::default().fg(theme::SUBTEXT)
            };
            let subject_style = if dim {
                theme::muted_style()
            } else {
                Style::default()
            };

            let has_desc = !t.description.is_empty();
            let is_expanded = has_desc && expanded_ids.contains(&t.id);

            let subject_cell = if is_expanded {
                let desc_preview: String = t.description.chars().take(80).collect();
                Cell::from(Text::from(vec![
                    Line::from(Span::styled(format!("+ {}", t.subject), subject_style)),
                    Line::from(Span::styled(desc_preview, theme::muted_style())),
                ]))
            } else if tree_view && blocked {
                Cell::from(Line::from(vec![
                    Span::styled(
                        if has_desc {
                            format!("+ {}", t.subject)
                        } else {
                            t.subject.clone()
                        },
                        subject_style,
                    ),
                    Span::styled(
                        format!(" ← {}", active_blocker_ids.join(", ")),
                        Style::default().fg(theme::ORANGE),
                    ),
                ]))
            } else if has_desc {
                Cell::from(Span::styled(format!("+ {}", t.subject), subject_style))
            } else {
                Cell::from(Span::styled(t.subject.as_str(), subject_style))
            };

            let row = Row::new(vec![
                Cell::from(Span::styled(t.id.as_str(), id_style)),
                Cell::from(Span::styled(t.status.as_str(), status_style)),
                Cell::from(Span::styled(t.priority.as_str(), pri_style)),
                Cell::from(Span::styled(
                    if t.task_type.is_empty() {
                        "--"
                    } else {
                        &t.task_type
                    },
                    type_style,
                )),
                Cell::from(Span::styled(
                    if t.owner.is_empty() { "" } else { &t.owner },
                    owner_style,
                )),
                subject_cell,
            ]);

            if is_expanded { row.height(2) } else { row }
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Length(12),
        Constraint::Length(4),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Fill(1),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(theme::selected_style())
        .column_spacing(1);

    f.render_stateful_widget(table, area, &mut state.table_state);
}

fn build_tree_display(
    all_tasks: &[Task],
    filtered: &[Task],
    collapsed: &HashSet<String>,
) -> (Vec<Task>, HashSet<String>) {
    let matching_ids: HashSet<&str> = filtered.iter().map(|t| t.id.as_str()).collect();

    // Walk up parent chains to include ancestors for tree structure
    let all_by_id: HashMap<&str, &Task> = all_tasks.iter().map(|t| (t.id.as_str(), t)).collect();
    let mut needed: HashSet<String> = filtered.iter().map(|t| t.id.clone()).collect();
    let mut queue: Vec<String> = filtered
        .iter()
        .filter(|t| !t.parent_id.is_empty() && !matching_ids.contains(t.parent_id.as_str()))
        .map(|t| t.parent_id.clone())
        .collect();
    while let Some(pid) = queue.pop() {
        if needed.contains(&pid) {
            continue;
        }
        if let Some(parent) = all_by_id.get(pid.as_str()) {
            needed.insert(pid);
            if !parent.parent_id.is_empty() {
                queue.push(parent.parent_id.clone());
            }
        }
    }

    let has_extra = needed.len() > matching_ids.len();

    // Build expanded task set preserving original order
    let expanded: Vec<Task> = if has_extra {
        all_tasks
            .iter()
            .filter(|t| needed.contains(&t.id))
            .cloned()
            .collect()
    } else {
        filtered.to_vec()
    };

    // Which tasks have children in the expanded set
    let tree_parent_ids: HashSet<&str> = expanded
        .iter()
        .filter(|t| !t.parent_id.is_empty())
        .map(|t| t.parent_id.as_str())
        .collect();

    let rows = tree_order(&expanded);

    // Filter out collapsed subtrees
    let mut result: Vec<Task> = Vec::new();
    let mut skip_depth: Option<u8> = None;
    for row in rows {
        if let Some(sd) = skip_depth {
            if row.depth > sd {
                continue;
            }
            skip_depth = None;
        }
        let is_collapsed = collapsed.contains(&row.task.id);
        if is_collapsed {
            skip_depth = Some(row.depth);
        }

        let prefix = tree_prefix(&row);
        let has_kids = tree_parent_ids.contains(row.task.id.as_str());
        let indicator = if has_kids {
            if is_collapsed { "▸ " } else { "▾ " }
        } else {
            ""
        };

        let mut t = row.task;
        if !prefix.is_empty() || !indicator.is_empty() {
            t.subject = format!("{prefix}{indicator}{}", t.subject);
        }
        result.push(t);
    }

    let dim_ids: HashSet<String> = if has_extra {
        needed
            .into_iter()
            .filter(|id| !matching_ids.contains(id.as_str()))
            .collect()
    } else {
        HashSet::new()
    };

    (result, dim_ids)
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
        state.visible_count
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
