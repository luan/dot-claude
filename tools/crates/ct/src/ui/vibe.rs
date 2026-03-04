use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Row, Table, TableState};

use crate::store::{self, Task};
use crate::ui::theme;

pub struct VibeState {
    pub items: Vec<Task>,
    pub filtered: Vec<Task>,
    pub table_state: TableState,
    pub searching: bool,
    pub query: String,
    pub search_input: String,
    pub show_completed: bool,
}

impl VibeState {
    pub fn new(items: Vec<Task>) -> Self {
        let mut table_state = TableState::default();
        if !items.is_empty() {
            table_state.select(Some(0));
        }
        let filtered = items.clone();
        Self {
            items,
            filtered,
            table_state,
            searching: false,
            query: String::new(),
            search_input: String::new(),
            show_completed: false,
        }
    }

    pub fn filter(&mut self) {
        let q = self.query.to_lowercase();
        self.filtered = self
            .items
            .iter()
            .filter(|t| {
                if !self.show_completed && t.status == store::Status::Completed {
                    return false;
                }
                if !q.is_empty()
                    && !t.subject.to_lowercase().contains(&q)
                    && !t.vibe_prompt.to_lowercase().contains(&q)
                    && !t.vibe_stage.to_lowercase().contains(&q)
                {
                    return false;
                }
                true
            })
            .cloned()
            .collect();
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

    pub fn selected_vibe(&self) -> Option<&Task> {
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

fn stage_display(stage: &str) -> String {
    let idx = store::vibe_stage_index(stage);
    if idx <= 5 {
        format!("[{}/6]", idx + 1)
    } else {
        format!("[?/6]")
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

pub fn render_vibes(f: &mut Frame, area: Rect, state: &mut VibeState) {
    if state.filtered.is_empty() {
        let center_y = area.y + area.height / 2;
        let msg = Line::from(Span::styled("No vibe runs", theme::muted_style()));
        let msg_area = Rect::new(area.x, center_y, area.width, 1);
        f.render_widget(
            ratatui::widgets::Paragraph::new(msg).alignment(ratatui::layout::Alignment::Center),
            msg_area,
        );
        return;
    }

    let header = Row::new(vec!["ID", "Stage", "Status", "Prompt"])
        .style(
            Style::default()
                .fg(theme::SUBTEXT)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(0);

    let rows: Vec<Row> = state
        .filtered
        .iter()
        .map(|t| {
            Row::new(vec![
                Cell::from(Span::styled(&t.id, theme::muted_style())),
                Cell::from(Span::styled(
                    stage_display(&t.vibe_stage),
                    theme::muted_style(),
                )),
                Cell::from(Span::styled(
                    t.status.as_str().to_string(),
                    theme::status_style(&t.status),
                )),
                Cell::from(Span::raw(truncate(&t.vibe_prompt, 80))),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(6),
        Constraint::Length(8),
        Constraint::Length(12),
        Constraint::Fill(1),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(theme::selected_style())
        .column_spacing(1);

    f.render_stateful_widget(table, area, &mut state.table_state);
}

pub fn render_vibes_filter_bar(f: &mut Frame, area: Rect, state: &VibeState) {
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

    if state.show_completed {
        spans.push(Span::styled(" +completed ", theme::filter_tag_style()));
        spans.push(Span::raw(" "));
    }

    spans.push(Span::styled(
        format!("{} vibes", state.filtered.len()),
        theme::muted_style(),
    ));

    f.render_widget(Line::from(spans), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_vibe(id: &str, stage: &str, status: &str, prompt: &str) -> Task {
        Task::from_raw(json!({
            "id": id,
            "subject": format!("vibe {id}"),
            "status": status,
            "metadata": {
                "vibe_stage": stage,
                "vibe_prompt": prompt,
            }
        }))
    }

    #[test]
    fn new_selects_first_when_non_empty() {
        let state = VibeState::new(vec![make_vibe("1", "branch", "in_progress", "test")]);
        assert_eq!(state.table_state.selected(), Some(0));
        assert_eq!(state.filtered.len(), 1);
    }

    #[test]
    fn new_selects_none_when_empty() {
        let state = VibeState::new(vec![]);
        assert_eq!(state.table_state.selected(), None);
    }

    #[test]
    fn filter_hides_completed_by_default() {
        let mut state = VibeState::new(vec![
            make_vibe("1", "branch", "in_progress", "a"),
            make_vibe("2", "commit", "completed", "b"),
        ]);
        state.filter();
        assert_eq!(state.filtered.len(), 1);
        assert_eq!(state.filtered[0].id, "1");
    }

    #[test]
    fn filter_shows_completed_when_toggled() {
        let mut state = VibeState::new(vec![
            make_vibe("1", "branch", "in_progress", "a"),
            make_vibe("2", "commit", "completed", "b"),
        ]);
        state.show_completed = true;
        state.filter();
        assert_eq!(state.filtered.len(), 2);
    }

    #[test]
    fn filter_by_query_matches_prompt() {
        let mut state = VibeState::new(vec![
            make_vibe("1", "branch", "in_progress", "build ui"),
            make_vibe("2", "scope", "in_progress", "fix bug"),
        ]);
        state.query = "build".to_string();
        state.filter();
        assert_eq!(state.filtered.len(), 1);
        assert_eq!(state.filtered[0].id, "1");
    }

    #[test]
    fn filter_by_query_matches_stage() {
        let mut state = VibeState::new(vec![
            make_vibe("1", "branch", "in_progress", "a"),
            make_vibe("2", "scope", "in_progress", "b"),
        ]);
        state.query = "scope".to_string();
        state.filter();
        assert_eq!(state.filtered.len(), 1);
        assert_eq!(state.filtered[0].id, "2");
    }

    #[test]
    fn filter_clamps_selection_when_filtered_shrinks() {
        let mut state = VibeState::new(vec![
            make_vibe("1", "branch", "in_progress", "a"),
            make_vibe("2", "scope", "in_progress", "b"),
            make_vibe("3", "develop", "in_progress", "c"),
        ]);
        state.table_state.select(Some(2));
        state.query = "a".to_string();
        state.filter();
        assert_eq!(state.table_state.selected(), Some(0));
    }

    #[test]
    fn selected_vibe_returns_correct_task() {
        let state = VibeState::new(vec![
            make_vibe("1", "branch", "in_progress", "a"),
            make_vibe("2", "scope", "in_progress", "b"),
        ]);
        let selected = state.selected_vibe().unwrap();
        assert_eq!(selected.id, "1");
    }

    #[test]
    fn next_advances_selection() {
        let mut state = VibeState::new(vec![
            make_vibe("1", "branch", "in_progress", "a"),
            make_vibe("2", "scope", "in_progress", "b"),
        ]);
        state.next();
        assert_eq!(state.table_state.selected(), Some(1));
    }

    #[test]
    fn next_clamps_at_end() {
        let mut state = VibeState::new(vec![make_vibe("1", "branch", "in_progress", "a")]);
        state.next();
        assert_eq!(state.table_state.selected(), Some(0));
    }

    #[test]
    fn prev_decrements_selection() {
        let mut state = VibeState::new(vec![
            make_vibe("1", "branch", "in_progress", "a"),
            make_vibe("2", "scope", "in_progress", "b"),
        ]);
        state.table_state.select(Some(1));
        state.prev();
        assert_eq!(state.table_state.selected(), Some(0));
    }

    #[test]
    fn prev_clamps_at_zero() {
        let mut state = VibeState::new(vec![make_vibe("1", "branch", "in_progress", "a")]);
        state.prev();
        assert_eq!(state.table_state.selected(), Some(0));
    }

    #[test]
    fn home_selects_first() {
        let mut state = VibeState::new(vec![
            make_vibe("1", "branch", "in_progress", "a"),
            make_vibe("2", "scope", "in_progress", "b"),
        ]);
        state.table_state.select(Some(1));
        state.home();
        assert_eq!(state.table_state.selected(), Some(0));
    }

    #[test]
    fn end_selects_last() {
        let mut state = VibeState::new(vec![
            make_vibe("1", "branch", "in_progress", "a"),
            make_vibe("2", "scope", "in_progress", "b"),
        ]);
        state.end();
        assert_eq!(state.table_state.selected(), Some(1));
    }

    #[test]
    fn stage_display_known_stages() {
        assert_eq!(stage_display("branch"), "[1/6]");
        assert_eq!(stage_display("scope"), "[2/6]");
        assert_eq!(stage_display("develop"), "[3/6]");
        assert_eq!(stage_display("simplify"), "[4/6]");
        assert_eq!(stage_display("review"), "[5/6]");
        assert_eq!(stage_display("commit"), "[6/6]");
    }

    #[test]
    fn stage_display_unknown_stage() {
        assert_eq!(stage_display("bogus"), "[?/6]");
    }

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_string_adds_ellipsis() {
        let result = truncate("hello world this is long", 10);
        assert!(result.len() <= 13); // 9 ascii + ellipsis (3 bytes)
        assert!(result.ends_with('…'));
    }
}
