use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::plan;
use crate::planfile;
use crate::spec::{self, Spec};
use crate::store::Task;
use crate::ui::theme;

pub struct SpecDetailState {
    pub spec: Spec,
    pub content: String,
    pub linked_tasks: Vec<Task>,
    pub scroll: u16,
}

impl SpecDetailState {
    pub fn new(spec: Spec, tasks: &[Task]) -> Self {
        let content = spec::load_content(&spec.path);
        let spec_path = spec.path.to_string_lossy();
        let linked_tasks: Vec<Task> = tasks
            .iter()
            .filter(|t| !t.spec_file.is_empty() && t.spec_file == spec_path)
            .cloned()
            .collect();
        Self {
            spec,
            content,
            linked_tasks,
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

pub fn render_spec_detail(f: &mut Frame, area: Rect, state: &SpecDetailState) {
    let mut lines: Vec<Line> = Vec::new();
    let s = &state.spec;

    lines.push(Line::raw(""));

    let field = |name: &str, value: &str, style: Style| -> Line {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{name:>12} "), theme::label_style()),
            Span::styled(value.to_string(), style),
        ])
    };

    if !s.project.is_empty() {
        lines.push(field(
            "Project",
            &planfile::project_name(&s.project),
            theme::value_style(),
        ));
    }
    if !s.title.is_empty() {
        lines.push(field("Topic", &s.title, theme::value_style()));
    }
    lines.push(field(
        "Date",
        &plan::format_date(s.mod_time),
        theme::muted_style(),
    ));

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("─".repeat(60), Style::default().fg(theme::OVERLAY)),
    ]));
    lines.push(Line::raw(""));

    for l in state.content.lines() {
        lines.push(Line::from(vec![Span::raw(format!("  {l}"))]));
    }

    if !state.linked_tasks.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("Linked Tasks", theme::section_style()),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("─".repeat(60), Style::default().fg(theme::OVERLAY)),
        ]));
        lines.push(Line::raw(""));

        for t in &state.linked_tasks {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("#{:<6}", t.id),
                    Style::default().fg(theme::LAVENDER),
                ),
                Span::styled(
                    format!("{:<12}", t.status.as_str()),
                    theme::status_style(&t.status),
                ),
                Span::styled(&t.subject, theme::value_style()),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((state.scroll, 0));

    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{Priority, Status};
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn make_task(id: &str, subject: &str, spec_file: &str, status: Status) -> Task {
        Task {
            id: id.to_string(),
            subject: subject.to_string(),
            description: String::new(),
            active_form: String::new(),
            status,
            owner: String::new(),
            blocks: Vec::new(),
            blocked_by: Vec::new(),
            priority: Priority::None,
            task_type: String::new(),
            parent_id: String::new(),
            branch: String::new(),
            status_detail: String::new(),
            project: String::new(),
            plan_file: String::new(),
            spec_file: spec_file.to_string(),
            slug: String::new(),
            vibe_stage: String::new(),
            vibe_epic: String::new(),
            vibe_prompt: String::new(),
            session_id: String::new(),
            raw: serde_json::Value::Null,
        }
    }

    fn make_spec(path: &str) -> Spec {
        Spec {
            name: "test".to_string(),
            path: PathBuf::from(path),
            title: "Test Spec".to_string(),
            project: "/Users/me/project".to_string(),
            mod_time: SystemTime::UNIX_EPOCH,
            size: 100,
        }
    }

    #[test]
    fn linked_tasks_filters_by_spec_path() {
        let spec = make_spec("/home/user/.claude/specs/my-spec.md");
        let tasks = vec![
            make_task(
                "1",
                "matching",
                "/home/user/.claude/specs/my-spec.md",
                Status::Pending,
            ),
            make_task(
                "2",
                "different spec",
                "/home/user/.claude/specs/other.md",
                Status::Completed,
            ),
            make_task("3", "no spec", "", Status::InProgress),
            make_task(
                "4",
                "also matching",
                "/home/user/.claude/specs/my-spec.md",
                Status::InProgress,
            ),
        ];

        let state = SpecDetailState {
            spec,
            content: String::new(),
            linked_tasks: {
                let spec_path = "/home/user/.claude/specs/my-spec.md";
                tasks
                    .iter()
                    .filter(|t| !t.spec_file.is_empty() && t.spec_file == spec_path)
                    .cloned()
                    .collect()
            },
            scroll: 0,
        };

        assert_eq!(state.linked_tasks.len(), 2);
        assert_eq!(state.linked_tasks[0].id, "1");
        assert_eq!(state.linked_tasks[1].id, "4");
    }

    #[test]
    fn linked_tasks_empty_when_no_matches() {
        let spec = make_spec("/home/user/.claude/specs/my-spec.md");
        let tasks = vec![
            make_task(
                "1",
                "different",
                "/home/user/.claude/specs/other.md",
                Status::Pending,
            ),
            make_task("2", "no spec", "", Status::Completed),
        ];

        let state = SpecDetailState {
            spec,
            content: String::new(),
            linked_tasks: {
                let spec_path = "/home/user/.claude/specs/my-spec.md";
                tasks
                    .iter()
                    .filter(|t| !t.spec_file.is_empty() && t.spec_file == spec_path)
                    .cloned()
                    .collect()
            },
            scroll: 0,
        };

        assert!(state.linked_tasks.is_empty());
    }
}
