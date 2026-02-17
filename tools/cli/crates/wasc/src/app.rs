use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::editor;
use crate::plan;
use crate::store::{Status, Store, Task, TaskList};
use crate::ui::{confirm, create, detail, help, list, plan_detail, plans, status, theme};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Screen {
    List,
    Detail,
    Status,
    Confirm,
    Create,
    Plans,
    PlanDetail,
    Help,
}

pub struct App {
    store: Store,
    active_list: String,
    task_lists: Vec<TaskList>,
    list_idx: usize,
    screen: Screen,
    prev_screen: Screen,
    list: list::ListState,
    detail: Option<detail::DetailState>,
    status_picker: Option<status::StatusPickerState>,
    confirm: Option<confirm::ConfirmState>,
    create_form: Option<create::CreateState>,
    plans_state: Option<plans::PlansState>,
    plan_detail: Option<plan_detail::PlanDetailState>,
    status_msg: String,
    pub should_quit: bool,
    pub editor_request: Option<EditorRequest>,
}

pub struct EditorRequest {
    pub path: String,
    pub task_id: String,
    pub list_id: String,
}

impl App {
    pub fn new(
        store: Store,
        task_lists: Vec<TaskList>,
        tasks: Vec<Task>,
        active_list: String,
    ) -> Self {
        let list_idx = task_lists
            .iter()
            .position(|l| l.id == active_list)
            .unwrap_or(0);
        Self {
            store,
            active_list,
            task_lists,
            list_idx,
            screen: Screen::List,
            prev_screen: Screen::List,
            list: list::ListState::new(tasks),
            detail: None,
            status_picker: None,
            confirm: None,
            create_form: None,
            plans_state: None,
            plan_detail: None,
            status_msg: String::new(),
            should_quit: false,
            editor_request: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Clear status message on any key
        if !self.status_msg.is_empty() {
            self.status_msg.clear();
        }

        // Help toggle (works from most screens)
        if key.code == KeyCode::Char('?')
            && self.screen != Screen::Help
            && !self.list.searching
            && !self.plans_state.as_ref().is_some_and(|p| p.searching)
        {
            self.prev_screen = self.screen;
            self.screen = Screen::Help;
            return;
        }
        if self.screen == Screen::Help {
            // Any key closes help
            self.screen = self.prev_screen;
            return;
        }

        match self.screen {
            Screen::List => self.handle_list_key(key),
            Screen::Detail => self.handle_detail_key(key),
            Screen::Status => self.handle_status_key(key),
            Screen::Confirm => self.handle_confirm_key(key),
            Screen::Create => self.handle_create_key(key),
            Screen::Plans => self.handle_plans_key(key),
            Screen::PlanDetail => self.handle_plan_detail_key(key),
            Screen::Help => {} // handled above
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) {
        // Search mode
        if self.list.searching {
            match key.code {
                KeyCode::Esc => {
                    self.list.searching = false;
                    self.list.query.clear();
                    self.list.search_input.clear();
                    self.list.rebuild();
                }
                KeyCode::Enter => {
                    self.list.searching = false;
                }
                KeyCode::Backspace => {
                    self.list.search_input.pop();
                    self.list.query = self.list.search_input.clone();
                    self.list.rebuild();
                }
                KeyCode::Char(c) => {
                    self.list.search_input.push(c);
                    self.list.query = self.list.search_input.clone();
                    self.list.rebuild();
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('c')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.should_quit = true;
            }
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.list.next(),
            KeyCode::Char('k') | KeyCode::Up => self.list.prev(),
            KeyCode::Enter => self.open_detail(),
            KeyCode::Char('/') => {
                self.list.searching = true;
                self.list.search_input.clear();
                self.list.query.clear();
            }
            KeyCode::Char('f') => {
                self.list.status_filter = self.list.status_filter.next();
                self.list.rebuild();
            }
            KeyCode::Char('o') => {
                self.list.sort_order = self.list.sort_order.next();
                self.list.rebuild();
            }
            KeyCode::Char('T') => {
                self.list.tree_view = !self.list.tree_view;
            }
            KeyCode::Char('A') => {
                self.list.show_closed = !self.list.show_closed;
                self.list.rebuild();
            }
            KeyCode::Char('F') => {
                self.list.status_filter = crate::store::StatusFilter::Active;
                self.list.sort_order = crate::store::SortOrder::Id;
                self.list.show_closed = false;
                self.list.tree_view = false;
                self.list.query.clear();
                self.list.search_input.clear();
                self.list.rebuild();
            }
            KeyCode::Char('s') => self.open_status_picker(),
            KeyCode::Char('p') => self.quick_status(Status::Pending),
            KeyCode::Char('a') => self.quick_status(Status::InProgress),
            KeyCode::Char('d') => self.quick_status(Status::Completed),
            KeyCode::Char('n') => {
                self.create_form = Some(create::CreateState::new());
                self.screen = Screen::Create;
            }
            KeyCode::Char('e') => self.open_editor(),
            KeyCode::Char('D') => self.open_confirm(),
            KeyCode::Char('R') => {
                self.reload_tasks();
                self.status_msg = "Reloaded".to_string();
            }
            KeyCode::Char('L') => self.cycle_task_list(),
            KeyCode::Char('P') => {
                let all_plans = plan::list_plans();
                self.plans_state = Some(plans::PlansState::new(all_plans));
                self.screen = Screen::Plans;
            }
            _ => {}
        }
    }

    fn handle_detail_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.screen = Screen::List,
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(d) = &mut self.detail {
                    d.scroll_down();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(d) = &mut self.detail {
                    d.scroll_up();
                }
            }
            KeyCode::Char(' ') | KeyCode::PageDown => {
                if let Some(d) = &mut self.detail {
                    d.page_down(10);
                }
            }
            KeyCode::Char('b') | KeyCode::PageUp => {
                if let Some(d) = &mut self.detail {
                    d.page_up(10);
                }
            }
            KeyCode::Char('s') => self.open_status_picker(),
            KeyCode::Char('e') => self.open_editor(),
            KeyCode::Char('D') => self.open_confirm(),
            _ => {}
        }
    }

    fn handle_status_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.screen = self.prev_screen,
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(sp) = &mut self.status_picker {
                    sp.next();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(sp) = &mut self.status_picker {
                    sp.prev();
                }
            }
            KeyCode::Enter => {
                if let Some(sp) = &self.status_picker {
                    let task_id = sp.task_id.clone();
                    let new_status = sp.selected().clone();
                    self.change_status(&task_id, new_status);
                    self.screen = self.prev_screen;
                }
            }
            _ => {}
        }
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(c) = &self.confirm {
                    let task_id = c.task_id.clone();
                    match self.store.delete_task(&self.active_list, &task_id) {
                        Ok(()) => {
                            self.status_msg = format!("Deleted #{task_id}");
                            self.reload_tasks();
                        }
                        Err(e) => self.status_msg = format!("Error: {e}"),
                    }
                }
                self.screen = Screen::List;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.screen = self.prev_screen;
            }
            _ => {}
        }
    }

    fn handle_create_key(&mut self, key: KeyEvent) {
        let Some(form) = &mut self.create_form else {
            return;
        };

        match key.code {
            KeyCode::Esc => self.screen = Screen::List,
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let subject = form.subject().to_string();
                if subject.is_empty() {
                    self.status_msg = "Subject is required".to_string();
                    self.screen = Screen::List;
                    return;
                }
                let description = form.description().to_string();
                let priority = form.priority();
                let task_type = form.task_type().to_string();
                let parent_id = form.parent_id().to_string();

                let task = Task {
                    id: String::new(),
                    subject: subject.clone(),
                    description,
                    active_form: String::new(),
                    status: Status::Pending,
                    blocks: Vec::new(),
                    blocked_by: Vec::new(),
                    priority,
                    task_type,
                    parent_id,
                    branch: String::new(),
                    status_detail: String::new(),
                    project: String::new(),
                    raw: serde_json::Value::Null,
                };

                match self.store.create_task(&self.active_list, &task) {
                    Ok(created) => {
                        self.status_msg = format!("Created #{}: {}", created.id, created.subject);
                        self.reload_tasks();
                    }
                    Err(e) => self.status_msg = format!("Error: {e}"),
                }
                self.screen = Screen::List;
            }
            KeyCode::Tab | KeyCode::Down => form.next_field(),
            KeyCode::BackTab | KeyCode::Up => form.prev_field(),
            KeyCode::Backspace => form.backspace(),
            KeyCode::Char(c) => form.type_char(c),
            _ => {}
        }
    }

    fn handle_plans_key(&mut self, key: KeyEvent) {
        let Some(ps) = &mut self.plans_state else {
            return;
        };

        if ps.searching {
            match key.code {
                KeyCode::Esc => {
                    ps.searching = false;
                    ps.query.clear();
                    ps.search_input.clear();
                    ps.filter();
                }
                KeyCode::Enter => {
                    ps.searching = false;
                }
                KeyCode::Backspace => {
                    ps.search_input.pop();
                    ps.query = ps.search_input.clone();
                    ps.filter();
                }
                KeyCode::Char(c) => {
                    ps.search_input.push(c);
                    ps.query = ps.search_input.clone();
                    ps.filter();
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.screen = Screen::List,
            KeyCode::Char('j') | KeyCode::Down => ps.next(),
            KeyCode::Char('k') | KeyCode::Up => ps.prev(),
            KeyCode::Char('g') => ps.home(),
            KeyCode::Char('G') => ps.end(),
            KeyCode::Char('/') => {
                ps.searching = true;
                ps.search_input.clear();
                ps.query.clear();
            }
            KeyCode::Enter => {
                if let Some(p) = ps.selected_plan().cloned() {
                    self.plan_detail = Some(plan_detail::PlanDetailState::new(p));
                    self.screen = Screen::PlanDetail;
                }
            }
            _ => {}
        }
    }

    fn handle_plan_detail_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.screen = Screen::Plans,
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(pd) = &mut self.plan_detail {
                    pd.scroll_down();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(pd) = &mut self.plan_detail {
                    pd.scroll_up();
                }
            }
            KeyCode::Char(' ') | KeyCode::PageDown => {
                if let Some(pd) = &mut self.plan_detail {
                    pd.page_down(10);
                }
            }
            KeyCode::Char('b') | KeyCode::PageUp => {
                if let Some(pd) = &mut self.plan_detail {
                    pd.page_up(10);
                }
            }
            _ => {}
        }
    }

    // Actions

    fn open_detail(&mut self) {
        if let Some(id) = self.list.selected_id()
            && let Some(task) = self.store.load_task(&self.active_list, &id)
        {
            let children: Vec<Task> = self
                .list
                .tasks
                .iter()
                .filter(|t| t.parent_id == id)
                .cloned()
                .collect();
            self.detail = Some(detail::DetailState::new(task, children));
            self.screen = Screen::Detail;
        }
    }

    fn open_status_picker(&mut self) {
        let (task_id, current_status) = match self.screen {
            Screen::List => {
                if let Some(t) = self.list.selected_task() {
                    (t.id.clone(), t.status.clone())
                } else {
                    return;
                }
            }
            Screen::Detail => {
                if let Some(d) = &self.detail {
                    (d.task.id.clone(), d.task.status.clone())
                } else {
                    return;
                }
            }
            _ => return,
        };
        self.prev_screen = self.screen;
        self.status_picker = Some(status::StatusPickerState::new(task_id, &current_status));
        self.screen = Screen::Status;
    }

    fn quick_status(&mut self, new_status: Status) {
        let task_id = match self.screen {
            Screen::List => self.list.selected_id(),
            Screen::Detail => self.detail.as_ref().map(|d| d.task.id.clone()),
            _ => None,
        };
        if let Some(id) = task_id {
            self.change_status(&id, new_status);
        }
    }

    fn change_status(&mut self, task_id: &str, new_status: Status) {
        if let Some(mut task) = self.store.load_task(&self.active_list, task_id) {
            task.status = new_status.clone();
            match self.store.save_task(&self.active_list, &task) {
                Ok(()) => {
                    self.status_msg = format!("Status → {}", new_status.as_str());
                    self.reload_tasks();
                    if self.screen == Screen::Detail || self.prev_screen == Screen::Detail {
                        self.refresh_detail(task_id);
                    }
                }
                Err(e) => self.status_msg = format!("Error: {e}"),
            }
        }
    }

    fn open_confirm(&mut self) {
        let task_id = match self.screen {
            Screen::List => self.list.selected_id(),
            Screen::Detail => self.detail.as_ref().map(|d| d.task.id.clone()),
            _ => None,
        };
        if let Some(id) = task_id {
            self.prev_screen = self.screen;
            self.confirm = Some(confirm::ConfirmState::new(id, "Delete"));
            self.screen = Screen::Confirm;
        }
    }

    fn open_editor(&mut self) {
        let task_id = match self.screen {
            Screen::List => self.list.selected_id(),
            Screen::Detail => self.detail.as_ref().map(|d| d.task.id.clone()),
            _ => None,
        };
        let Some(id) = task_id else { return };
        let Some(task) = self.store.load_task(&self.active_list, &id) else {
            self.status_msg = "Failed to load task".to_string();
            return;
        };

        let content = editor::marshal_task(&task);
        let tmp_path = format!("/tmp/wasc-task-{}.md", task.id);
        if std::fs::write(&tmp_path, &content).is_err() {
            self.status_msg = "Failed to write temp file".to_string();
            return;
        }

        self.prev_screen = self.screen;
        self.editor_request = Some(EditorRequest {
            path: tmp_path,
            task_id: id,
            list_id: self.active_list.clone(),
        });
    }

    pub fn handle_editor_result(&mut self, task_id: &str, path: &str, list_id: &str) {
        let original = match self.store.load_task(list_id, task_id) {
            Some(t) => t,
            None => {
                self.status_msg = "Failed to reload task".to_string();
                self.screen = self.prev_screen;
                return;
            }
        };

        match std::fs::read_to_string(path) {
            Ok(data) => match editor::unmarshal_task(&data, &original) {
                Ok(updated) => match self.store.save_task(list_id, &updated) {
                    Ok(()) => {
                        self.status_msg = "Task updated".to_string();
                        self.reload_tasks();
                        if self.prev_screen == Screen::Detail {
                            self.refresh_detail(task_id);
                        }
                    }
                    Err(e) => self.status_msg = format!("Save: {e}"),
                },
                Err(e) => self.status_msg = format!("Parse: {e}"),
            },
            Err(e) => self.status_msg = format!("Read: {e}"),
        }
        let _ = std::fs::remove_file(path);
        self.screen = self.prev_screen;
    }

    fn reload_tasks(&mut self) {
        let tasks = self.store.list_tasks(&self.active_list);
        let filters = (
            self.list.status_filter,
            self.list.sort_order,
            self.list.show_closed,
            self.list.tree_view,
            self.list.query.clone(),
        );
        self.list = list::ListState::new(tasks);
        self.list.status_filter = filters.0;
        self.list.sort_order = filters.1;
        self.list.show_closed = filters.2;
        self.list.tree_view = filters.3;
        self.list.query = filters.4.clone();
        self.list.search_input = filters.4;
        self.list.rebuild();
    }

    fn refresh_detail(&mut self, task_id: &str) {
        if let Some(task) = self.store.load_task(&self.active_list, task_id) {
            let children: Vec<Task> = self
                .list
                .tasks
                .iter()
                .filter(|t| t.parent_id == task_id)
                .cloned()
                .collect();
            self.detail = Some(detail::DetailState::new(task, children));
        }
    }

    fn cycle_task_list(&mut self) {
        if self.task_lists.len() <= 1 {
            self.status_msg = "No other task lists".to_string();
            return;
        }
        self.list_idx = (self.list_idx + 1) % self.task_lists.len();
        self.active_list = self.task_lists[self.list_idx].id.clone();
        self.reload_tasks();
        self.status_msg = format!("List: {}", self.active_list);
    }

    // Rendering

    pub fn render(&mut self, f: &mut Frame) {
        let [header_area, body_area, footer_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(f.area());

        match self.screen {
            Screen::List => {
                self.render_header(f, header_area, "tasks");
                let [filter_area, table_area] =
                    Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(body_area);
                list::render_filter_bar(f, filter_area, &self.list);
                list::render_list(f, table_area, &mut self.list);
                self.render_footer(
                    f,
                    footer_area,
                    "?:help  n:new  s:status  P:plans  /:search  q:quit",
                );
            }
            Screen::Detail => {
                let title = self
                    .detail
                    .as_ref()
                    .map(|d| {
                        if d.task.subject.len() > 40 {
                            format!("{}...", &d.task.subject[..37])
                        } else {
                            d.task.subject.clone()
                        }
                    })
                    .unwrap_or_default();
                self.render_header(f, header_area, &title);
                if let Some(d) = &self.detail {
                    detail::render_detail(f, body_area, d);
                }
                self.render_footer(f, footer_area, "?:help  esc:back  s:status  e:edit  q:quit");
            }
            Screen::Status => {
                self.render_header(f, header_area, "change status");
                if let Some(sp) = &self.status_picker {
                    status::render_status_picker(f, body_area, sp);
                }
                self.render_footer(f, footer_area, "j/k:navigate  enter:select  esc:cancel");
            }
            Screen::Confirm => {
                self.render_header(f, header_area, "confirm");
                if let Some(c) = &self.confirm {
                    confirm::render_confirm(f, body_area, c);
                }
                self.render_footer(f, footer_area, "");
            }
            Screen::Create => {
                self.render_header(f, header_area, "new task");
                if let Some(form) = &self.create_form {
                    create::render_create(f, body_area, form);
                }
                self.render_footer(f, footer_area, "");
            }
            Screen::Plans => {
                self.render_header(f, header_area, "plans");
                let [filter_area, table_area] =
                    Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(body_area);
                if let Some(ps) = &self.plans_state {
                    plans::render_plans_filter_bar(f, filter_area, ps);
                }
                if let Some(ps) = &mut self.plans_state {
                    plans::render_plans(f, table_area, ps);
                }
                self.render_footer(
                    f,
                    footer_area,
                    "?:help  enter:open  /:search  esc:back  q:quit",
                );
            }
            Screen::PlanDetail => {
                let title = self
                    .plan_detail
                    .as_ref()
                    .map(|pd| {
                        let t = if pd.plan.title.is_empty() {
                            &pd.plan.name
                        } else {
                            &pd.plan.title
                        };
                        if t.len() > 40 {
                            format!("{}...", &t[..37])
                        } else {
                            t.clone()
                        }
                    })
                    .unwrap_or_default();
                self.render_header(f, header_area, &title);
                if let Some(pd) = &self.plan_detail {
                    plan_detail::render_plan_detail(f, body_area, pd);
                }
                self.render_footer(f, footer_area, "esc:back  q:quit");
            }
            Screen::Help => {
                self.render_header(f, header_area, "help");
                help::render_help(f, body_area);
                self.render_footer(f, footer_area, "");
            }
        }
    }

    fn render_header(&self, f: &mut Frame, area: Rect, title: &str) {
        let left = " wasc ".to_string();
        let list_tag = format!(" {} ", self.active_list);
        let right = format!(" {title} ");

        let left_width = left.len() as u16;
        let list_width = list_tag.len() as u16;
        let right_width = right.len() as u16;
        let total = left_width + list_width + right_width;
        let gap = area.width.saturating_sub(total);

        let line = Line::from(vec![
            Span::styled(left, theme::header_style()),
            Span::styled(list_tag, theme::header_dim_style()),
            Span::styled(" ".repeat(gap as usize), Style::default().bg(theme::ACCENT)),
            Span::styled(right, theme::header_dim_style()),
        ]);
        f.render_widget(line, area);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect, hint: &str) {
        if !self.status_msg.is_empty() {
            let msg = format!(" ✓ {} ", self.status_msg);
            let msg_width = msg.len() as u16;
            let hint_width = hint.len() as u16;
            let gap = area.width.saturating_sub(msg_width + hint_width + 2);

            let line = Line::from(vec![
                Span::styled(format!(" {hint}"), theme::footer_style()),
                Span::styled(
                    " ".repeat(gap as usize),
                    Style::default().bg(theme::SURFACE),
                ),
                Span::styled(msg, theme::status_msg_style()),
            ]);
            f.render_widget(line, area);
        } else {
            let line = Line::from(vec![Span::styled(
                format!(
                    " {hint}{}",
                    " ".repeat(area.width.saturating_sub(hint.len() as u16 + 1) as usize)
                ),
                theme::footer_style(),
            )]);
            f.render_widget(line, area);
        }
    }
}
