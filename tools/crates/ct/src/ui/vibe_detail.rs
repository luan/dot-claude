use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::planfile;
use crate::store::{self, Status, Task, meta_str_raw};
use crate::ui::detail::build_text_section;
use crate::ui::theme;

const STAGES: [&str; 6] = ["branch", "scope", "develop", "simplify", "review", "commit"];
const BAR_WIDTH: usize = 12;

pub struct VibeDetailState {
    pub tracker: Task,
    pub children: Vec<Task>,
    pub scroll: u16,
}

impl VibeDetailState {
    pub fn new(tracker: Task, children: Vec<Task>) -> Self {
        Self {
            tracker,
            children,
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

fn status_icon(status: &Status) -> &'static str {
    match status {
        Status::Completed => "✓",
        Status::InProgress => "→",
        _ => "·",
    }
}

fn build_pipeline_line(current_index: usize) -> Line<'static> {
    let mut spans = vec![Span::raw("  ")];
    for (i, stage) in STAGES.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        let (icon, style) = if current_index < STAGES.len() && i < current_index {
            ("✓", Style::default().fg(theme::GREEN))
        } else if current_index < STAGES.len() && i == current_index {
            ("→", Style::default().fg(theme::ACCENT))
        } else {
            (" ", Style::default().fg(theme::MUTED))
        };
        spans.push(Span::styled(format!("[{icon}] {stage}"), style));
    }
    Line::from(spans)
}

fn build_progress_line(current_index: usize, total: usize) -> Line<'static> {
    let completed = current_index.min(total);
    let filled = if total == 0 {
        0
    } else {
        (completed * BAR_WIDTH) / total
    };
    let empty = BAR_WIDTH - filled;
    let pct = if total == 0 {
        0
    } else {
        (completed * 100) / total
    };

    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty),);

    Line::from(vec![
        Span::raw("  "),
        Span::styled(bar, Style::default().fg(theme::ACCENT)),
        Span::raw(format!("  {completed}/{total}  {pct}%")),
    ])
}

fn child_label(c: &Task) -> String {
    let breadcrumb = meta_str_raw(&c.raw, "breadcrumb");
    if breadcrumb.is_empty() {
        c.subject.clone()
    } else {
        format!("{} > {}", breadcrumb, c.subject)
    }
}

pub fn render_vibe_detail(f: &mut Frame, area: Rect, state: &VibeDetailState) {
    let t = &state.tracker;
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::raw(""));

    let field = |name: &str, value: &str, style: Style| -> Line {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{name:>12} "), theme::label_style()),
            Span::styled(value.to_string(), style),
        ])
    };

    if !t.vibe_prompt.is_empty() {
        lines.push(field("Prompt", &t.vibe_prompt, theme::value_style()));
    }
    lines.push(field(
        "Status",
        t.status.as_str(),
        theme::status_style(&t.status),
    ));
    if !t.vibe_stage.is_empty() {
        lines.push(field(
            "Stage",
            &t.vibe_stage,
            Style::default().fg(theme::ACCENT),
        ));
    }
    if !t.branch.is_empty() {
        lines.push(field("Branch", &t.branch, theme::value_style()));
    }
    if !t.slug.is_empty() {
        lines.push(field("Slug", &t.slug, theme::value_style()));
    }
    if !t.project.is_empty() {
        lines.push(field(
            "Project",
            &planfile::project_name(&t.project),
            theme::value_style(),
        ));
    }
    if !t.vibe_epic.is_empty() {
        lines.push(field("Epic", &t.vibe_epic, theme::value_style()));
    }
    if !t.session_id.is_empty() {
        let sid = t.session_id.get(..8).unwrap_or(&t.session_id);
        lines.push(field("Session", sid, theme::muted_style()));
    }
    let completed_at = meta_str_raw(&t.raw, "completedAt");
    if !completed_at.is_empty() {
        lines.push(field("CompletedAt", &completed_at, theme::value_style()));
    }

    let current_index = store::vibe_stage_index(&t.vibe_stage);

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("Pipeline", theme::section_style()),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("─".repeat(60), Style::default().fg(theme::OVERLAY)),
    ]));
    lines.push(Line::raw(""));
    lines.push(build_pipeline_line(current_index));
    lines.push(Line::raw(""));
    lines.push(build_progress_line(current_index, STAGES.len()));

    let design = meta_str_raw(&t.raw, "design");
    if !design.is_empty() {
        lines.extend(build_text_section("Design", &design));
    }

    if !state.children.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("Children", theme::section_style()),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("─".repeat(60), Style::default().fg(theme::OVERLAY)),
        ]));
        lines.push(Line::raw(""));
        for c in &state.children {
            let icon = status_icon(&c.status);
            let label = child_label(c);
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(icon, theme::status_style(&c.status)),
                Span::raw(" "),
                Span::styled(format!("#{}", c.id), theme::muted_style()),
                Span::raw(" "),
                Span::styled(label, theme::value_style()),
            ]));
        }
    }

    lines.push(Line::raw(""));

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((state.scroll, 0));

    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tracker(stage: &str) -> Task {
        Task::from_raw(serde_json::json!({
            "id": "42",
            "subject": "Build feature X",
            "status": "in_progress",
            "metadata": {
                "vibe_stage": stage,
                "vibe_prompt": "build feature X end to end",
                "vibe_epic": "42",
                "branch": "luan/feature-x",
                "slug": "feature-x",
            }
        }))
    }

    fn make_child(id: &str, subject: &str, status: &str) -> Task {
        Task::from_raw(serde_json::json!({
            "id": id,
            "subject": subject,
            "status": status,
            "metadata": { "parent_id": "42" }
        }))
    }

    #[test]
    fn pipeline_completed_stages_get_checkmark() {
        let line = build_pipeline_line(2); // develop is current
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("[✓] branch"));
        assert!(text.contains("[✓] scope"));
        assert!(text.contains("[→] develop"));
        assert!(text.contains("[ ] simplify"));
    }

    #[test]
    fn pipeline_first_stage_has_no_checkmarks_before_it() {
        let line = build_pipeline_line(0); // branch is current
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("[→] branch"));
        assert!(text.contains("[ ] scope"));
    }

    #[test]
    fn pipeline_last_stage_all_preceding_checked() {
        let line = build_pipeline_line(5); // commit is current
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("[✓] branch"));
        assert!(text.contains("[✓] review"));
        assert!(text.contains("[→] commit"));
    }

    #[test]
    fn progress_bar_zero_of_six() {
        let line = build_progress_line(0, 6);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("░░░░░░░░░░░░"));
        assert!(text.contains("0/6"));
        assert!(text.contains("0%"));
    }

    #[test]
    fn progress_bar_three_of_six() {
        let line = build_progress_line(3, 6);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("██████░░░░░░"));
        assert!(text.contains("3/6"));
        assert!(text.contains("50%"));
    }

    #[test]
    fn progress_bar_six_of_six() {
        let line = build_progress_line(6, 6);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("████████████"));
        assert!(text.contains("6/6"));
        assert!(text.contains("100%"));
    }

    #[test]
    fn status_icon_maps_correctly() {
        assert_eq!(status_icon(&Status::Completed), "✓");
        assert_eq!(status_icon(&Status::InProgress), "→");
        assert_eq!(status_icon(&Status::Pending), "·");
    }

    #[test]
    fn scroll_methods_saturate() {
        let tracker = make_tracker("develop");
        let mut state = VibeDetailState::new(tracker, vec![]);
        assert_eq!(state.scroll, 0);

        state.scroll_up();
        assert_eq!(state.scroll, 0); // saturates at 0

        state.scroll_down();
        assert_eq!(state.scroll, 1);

        state.page_down(10);
        assert_eq!(state.scroll, 11);

        state.page_up(100);
        assert_eq!(state.scroll, 0); // saturates at 0
    }

    #[test]
    fn children_section_renders_with_correct_icons() {
        let tracker = make_tracker("develop");
        let children = vec![
            make_child("43", "Create branch", "completed"),
            make_child("44", "Run scope", "in_progress"),
            make_child("45", "Develop feature", "pending"),
        ];
        let state = VibeDetailState::new(tracker, children);

        // Verify children are stored
        assert_eq!(state.children.len(), 3);
        assert_eq!(status_icon(&state.children[0].status), "✓");
        assert_eq!(status_icon(&state.children[1].status), "→");
        assert_eq!(status_icon(&state.children[2].status), "·");
    }

    #[test]
    fn child_with_breadcrumb_shows_breadcrumb_in_label() {
        let child = Task::from_raw(serde_json::json!({
            "id": "284",
            "subject": "Trade handlers",
            "status": "completed",
            "metadata": {
                "parent_id": "42",
                "breadcrumb": "Trading System > Phase 1"
            }
        }));
        let label = child_label(&child);
        assert!(label.contains("Trading System > Phase 1"));
        assert!(label.contains("Trade handlers"));
    }

    #[test]
    fn child_without_breadcrumb_shows_subject_only() {
        let child = make_child("43", "Create branch", "completed");
        let label = child_label(&child);
        assert!(label.contains("Create branch"));
        assert!(!label.contains(">"));
    }
}
