use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Row, Table, TableState};

use crate::plan;
use crate::planfile;
use crate::spec::{self, Spec};
use crate::ui::theme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpecSource {
    Active,
    Archived,
}

impl SpecSource {
    pub fn next(self) -> Self {
        match self {
            Self::Active => Self::Archived,
            Self::Archived => Self::Active,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

pub struct SpecsState {
    pub specs: Vec<Spec>,
    pub filtered: Vec<Spec>,
    pub table_state: TableState,
    pub searching: bool,
    pub query: String,
    pub search_input: String,
    pub source: SpecSource,
}

impl SpecsState {
    pub fn new(specs: Vec<Spec>) -> Self {
        let mut table_state = TableState::default();
        if !specs.is_empty() {
            table_state.select(Some(0));
        }
        Self {
            filtered: specs.clone(),
            specs,
            table_state,
            searching: false,
            query: String::new(),
            search_input: String::new(),
            source: SpecSource::Active,
        }
    }

    pub fn reload_specs(&mut self) {
        self.specs = match self.source {
            SpecSource::Active => spec::list_specs(),
            SpecSource::Archived => spec::list_archived_specs(),
        };
        self.filter();
    }

    pub fn cycle_source(&mut self) {
        self.source = self.source.next();
        self.reload_specs();
    }

    pub fn filter(&mut self) {
        if self.query.is_empty() {
            self.filtered = self.specs.clone();
        } else {
            let q = self.query.to_lowercase();
            self.filtered = self
                .specs
                .iter()
                .filter(|s| {
                    s.title.to_lowercase().contains(&q)
                        || s.name.to_lowercase().contains(&q)
                        || planfile::project_name(&s.project)
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

    pub fn selected_spec(&self) -> Option<&Spec> {
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

pub fn render_specs(f: &mut Frame, area: Rect, state: &mut SpecsState) {
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
        .map(|s| {
            let title = if s.title.is_empty() {
                &s.name
            } else {
                &s.title
            };
            let proj = planfile::project_name(&s.project);
            Row::new(vec![
                Cell::from(Span::styled(proj, theme::muted_style())),
                Cell::from(Span::styled(
                    plan::format_date(s.mod_time),
                    theme::muted_style(),
                )),
                Cell::from(Span::styled(
                    plan::format_size(s.size),
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

pub fn render_specs_filter_bar(f: &mut Frame, area: Rect, state: &SpecsState) {
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

    if state.source != SpecSource::Active {
        spans.push(Span::styled(
            format!(" {} ", state.source.label()),
            theme::filter_tag_style(),
        ));
        spans.push(Span::raw(" "));
    }

    spans.push(Span::styled(
        format!("{} specs", state.filtered.len()),
        theme::muted_style(),
    ));

    f.render_widget(Line::from(spans), area);
}
