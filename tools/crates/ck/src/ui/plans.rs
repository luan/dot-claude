use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Row, Table, TableState};

use crate::plan::{self, Plan};
use crate::planfile;
use crate::ui::theme;

pub struct PlansState {
    pub plans: Vec<Plan>,
    pub filtered: Vec<Plan>,
    pub table_state: TableState,
    pub searching: bool,
    pub query: String,
    pub search_input: String,
    pub archived: bool,
    pub project_filter: Option<String>,
}

impl PlansState {
    pub fn new(plans: Vec<Plan>) -> Self {
        let mut table_state = TableState::default();
        if !plans.is_empty() {
            table_state.select(Some(0));
        }
        Self {
            filtered: plans.clone(),
            plans,
            table_state,
            searching: false,
            query: String::new(),
            search_input: String::new(),
            archived: false,
            project_filter: None,
        }
    }

    pub fn reload_plans(&mut self) {
        let raw = if self.archived {
            plan::list_archived_plans()
        } else {
            plan::list_plans()
        };
        self.plans = if let Some(ref proj) = self.project_filter {
            raw.into_iter().filter(|p| &p.project == proj).collect()
        } else {
            raw
        };
        self.filter();
    }

    pub fn toggle_archived(&mut self) {
        self.archived = !self.archived;
        self.reload_plans();
    }

    pub fn filter(&mut self) {
        if self.query.is_empty() {
            self.filtered = self.plans.clone();
        } else {
            let q = self.query.to_lowercase();
            self.filtered = self
                .plans
                .iter()
                .filter(|p| {
                    p.title.to_lowercase().contains(&q)
                        || p.name.to_lowercase().contains(&q)
                        || planfile::project_name(&p.project)
                            .to_lowercase()
                            .contains(&q)
                })
                .cloned()
                .collect();
        }
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

    pub fn selected_plan(&self) -> Option<&Plan> {
        self.table_state
            .selected()
            .and_then(|i| self.filtered.get(i))
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

    pub fn home(&mut self) {
        if !self.filtered.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    pub fn end(&mut self) {
        if !self.filtered.is_empty() {
            self.table_state.select(Some(self.filtered.len() - 1));
        }
    }
}

pub fn render_plans(f: &mut Frame, area: Rect, state: &mut PlansState) {
    let header = Row::new(vec!["Project", "Date", "Size", "Title"])
        .style(
            Style::default()
                .fg(theme::SUBTEXT)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(0);

    let rows: Vec<Row> = state
        .filtered
        .iter()
        .map(|p| {
            let title = if p.title.is_empty() {
                &p.name
            } else {
                &p.title
            };
            let proj = planfile::project_name(&p.project);
            Row::new(vec![
                Cell::from(Span::styled(proj, theme::muted_style())),
                Cell::from(Span::styled(
                    plan::format_date(p.mod_time),
                    theme::muted_style(),
                )),
                Cell::from(Span::styled(
                    plan::format_size(p.size),
                    theme::muted_style(),
                )),
                Cell::from(Span::raw(title)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(6),
        Constraint::Fill(1),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(theme::selected_style())
        .column_spacing(1);

    f.render_stateful_widget(table, area, &mut state.table_state);
}

pub fn render_plans_filter_bar(f: &mut Frame, area: Rect, state: &PlansState) {
    let mut spans = Vec::new();

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

    if state.archived {
        spans.push(Span::styled(" archived ", theme::filter_tag_style()));
        spans.push(Span::raw(" "));
    }

    if let Some(ref proj) = state.project_filter {
        spans.push(Span::styled(
            format!(" project:{} ", planfile::project_name(proj)),
            theme::filter_tag_style(),
        ));
        spans.push(Span::raw(" "));
    }

    spans.push(Span::styled(
        format!("{} plans", state.filtered.len()),
        theme::muted_style(),
    ));

    f.render_widget(Line::from(spans), area);
}
