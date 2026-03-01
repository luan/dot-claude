use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::plan::{self, Plan};
use crate::spec::{self, Spec};
use crate::store::Task;
use crate::ui::theme;

pub struct DetailState {
    pub task: Task,
    pub children: Vec<Task>,
    pub related_plans: Vec<Plan>,
    pub related_specs: Vec<Spec>,
    pub scroll: u16,
}

impl DetailState {
    pub fn new(task: Task, children: Vec<Task>) -> Self {
        // Phase 3: If task has a linked plan_file, show that plan specifically.
        // Otherwise fall back to project-wide matching.
        let related_plans: Vec<Plan> = if !task.plan_file.is_empty() {
            plan::list_plans()
                .into_iter()
                .filter(|p| p.path.to_string_lossy() == task.plan_file)
                .take(1)
                .collect()
        } else if !task.project.is_empty() {
            plan::list_plans()
                .into_iter()
                .filter(|p| !p.project.is_empty() && p.project == task.project)
                .take(3)
                .collect()
        } else {
            Vec::new()
        };

        let related_specs: Vec<Spec> = if !task.spec_file.is_empty() {
            spec::list_specs()
                .into_iter()
                .filter(|s| s.path.to_string_lossy() == task.spec_file)
                .take(1)
                .collect()
        } else if !task.project.is_empty() {
            spec::list_specs()
                .into_iter()
                .filter(|s| !s.project.is_empty() && s.project == task.project)
                .take(3)
                .collect()
        } else {
            Vec::new()
        };

        Self {
            task,
            children,
            related_plans,
            related_specs,
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

pub fn render_detail(f: &mut Frame, area: Rect, state: &DetailState) {
    let t = &state.task;
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::raw(""));

    let field = |name: &str, value: &str, style: Style| -> Line {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{name:>12} "), theme::label_style()),
            Span::styled(value.to_string(), style),
        ])
    };

    lines.push(field("ID", &t.id, Style::default().fg(theme::LAVENDER)));
    lines.push(field("Subject", &t.subject, theme::value_style()));
    lines.push(field(
        "Status",
        t.status.as_str(),
        theme::status_style(&t.status),
    ));
    lines.push(field(
        "Priority",
        t.priority.as_str(),
        theme::priority_style(&t.priority),
    ));

    if !t.owner.is_empty() {
        lines.push(field("Owner", &t.owner, theme::value_style()));
    }
    if !t.task_type.is_empty() {
        lines.push(field(
            "Type",
            &t.task_type,
            Style::default().fg(theme::type_color(&t.task_type)),
        ));
    }
    if !t.branch.is_empty() {
        lines.push(field("Branch", &t.branch, theme::value_style()));
    }
    if !t.parent_id.is_empty() {
        lines.push(field("Parent", &t.parent_id, theme::value_style()));
    }
    if !t.status_detail.is_empty() {
        lines.push(field("Detail", &t.status_detail, theme::value_style()));
    }
    if !t.active_form.is_empty() {
        lines.push(field("ActiveForm", &t.active_form, theme::value_style()));
    }
    if !t.plan_file.is_empty() {
        lines.push(field("Plan File", &t.plan_file, theme::value_style()));
    }
    if !t.spec_file.is_empty() {
        lines.push(field("Spec File", &t.spec_file, theme::value_style()));
    }
    if !t.slug.is_empty() {
        lines.push(field("Slug", &t.slug, theme::value_style()));
    }
    if !t.blocks.is_empty() {
        lines.push(field("Blocks", &t.blocks.join(", "), theme::value_style()));
    }
    if !t.blocked_by.is_empty() {
        lines.push(field(
            "BlockedBy",
            &t.blocked_by.join(", "),
            theme::value_style(),
        ));
    }

    if !t.description.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("Description", theme::section_style()),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("─".repeat(60), Style::default().fg(theme::OVERLAY)),
        ]));
        lines.push(Line::raw(""));
        for line in t.description.lines() {
            lines.push(Line::from(vec![Span::raw("  "), Span::raw(line)]));
        }
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
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{:<4}", c.id), theme::muted_style()),
                Span::raw("  "),
                Span::styled(
                    format!("{:<12}", c.status.as_str()),
                    theme::status_style(&c.status),
                ),
                Span::raw("  "),
                Span::styled(&c.subject, theme::value_style()),
            ]));
        }
    }

    if !state.related_specs.is_empty() {
        let label = if !t.spec_file.is_empty() {
            "Linked Spec"
        } else {
            "Related Specs"
        };
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(label, theme::section_style()),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("─".repeat(60), Style::default().fg(theme::OVERLAY)),
        ]));
        lines.push(Line::raw(""));
        for s in &state.related_specs {
            let title = if s.title.is_empty() {
                &s.name
            } else {
                &s.title
            };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(plan::format_date(s.mod_time), theme::muted_style()),
                Span::raw("  "),
                Span::styled(title.clone(), theme::value_style()),
            ]));
        }
    }

    if !state.related_plans.is_empty() {
        let label = if !t.plan_file.is_empty() {
            "Linked Plan"
        } else {
            "Related Plans"
        };
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(label, theme::section_style()),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("─".repeat(60), Style::default().fg(theme::OVERLAY)),
        ]));
        lines.push(Line::raw(""));
        for p in &state.related_plans {
            let title = if p.title.is_empty() {
                &p.name
            } else {
                &p.title
            };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(plan::format_date(p.mod_time), theme::muted_style()),
                Span::raw("  "),
                Span::styled(title.clone(), theme::value_style()),
            ]));
        }
    }

    lines.push(Line::raw(""));

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((state.scroll, 0));

    f.render_widget(paragraph, area);
}
