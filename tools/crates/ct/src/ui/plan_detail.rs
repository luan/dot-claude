use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::plan::{self, Plan};

pub struct PlanDetailState {
    pub plan: Plan,
    pub content: String,
    pub scroll: u16,
}

impl PlanDetailState {
    pub fn new(plan: Plan) -> Self {
        let content = plan::load_content(&plan.path);
        Self {
            plan,
            content,
            scroll: 0,
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn page_down(&mut self, page: u16) {
        self.scroll = self.scroll.saturating_add(page);
    }

    pub fn page_up(&mut self, page: u16) {
        self.scroll = self.scroll.saturating_sub(page);
    }
}

pub fn render_plan_detail(f: &mut Frame, area: Rect, state: &PlanDetailState) {
    let lines: Vec<Line> = state
        .content
        .lines()
        .map(|l| Line::from(vec![Span::raw(l)]))
        .collect();

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((state.scroll, 0));

    f.render_widget(paragraph, area);
}
