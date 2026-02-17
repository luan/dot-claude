use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Pending,
    InProgress,
    Completed,
    Other(String),
}

impl Status {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Other(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => Self::Pending,
            "in_progress" => Self::InProgress,
            "completed" => Self::Completed,
            other => Self::Other(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Priority {
    P1,
    P2,
    P3,
    None,
}

impl Priority {
    pub fn as_str(&self) -> &str {
        match self {
            Self::P1 => "P1",
            Self::P2 => "P2",
            Self::P3 => "P3",
            Self::None => "--",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "P1" | "1" => Self::P1,
            "P2" | "2" => Self::P2,
            "P3" | "3" => Self::P3,
            _ => Self::None,
        }
    }

    pub fn sort_key(&self) -> u8 {
        match self {
            Self::P1 => 1,
            Self::P2 => 2,
            Self::P3 => 3,
            Self::None => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub subject: String,
    pub description: String,
    pub active_form: String,
    pub status: Status,
    pub blocks: Vec<String>,
    pub blocked_by: Vec<String>,
    pub priority: Priority,
    pub task_type: String,
    pub parent_id: String,
    pub branch: String,
    pub status_detail: String,
    pub project: String,
    pub raw: Value,
}

#[derive(Deserialize)]
struct RawTask {
    id: Option<String>,
    subject: Option<String>,
    description: Option<String>,
    #[serde(rename = "activeForm")]
    active_form: Option<String>,
    status: Option<String>,
    blocks: Option<Vec<String>>,
    #[serde(rename = "blockedBy")]
    blocked_by: Option<Vec<String>>,
    metadata: Option<Value>,
}

impl Task {
    fn from_raw(raw_val: Value) -> Self {
        let raw: RawTask = serde_json::from_value(raw_val.clone()).unwrap_or(RawTask {
            id: None,
            subject: None,
            description: None,
            active_form: None,
            status: None,
            blocks: None,
            blocked_by: None,
            metadata: None,
        });

        let meta = raw.metadata.as_ref().and_then(|v| v.as_object());
        let meta_str = |key: &str| -> String {
            meta.and_then(|m| m.get(key))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        };

        Task {
            id: raw.id.unwrap_or_default(),
            subject: raw.subject.unwrap_or_default(),
            description: raw.description.unwrap_or_default(),
            active_form: raw.active_form.unwrap_or_default(),
            status: Status::from_str(raw.status.as_deref().unwrap_or("pending")),
            blocks: raw.blocks.unwrap_or_default(),
            blocked_by: raw.blocked_by.unwrap_or_default(),
            priority: Priority::from_str(&meta_str("priority")),
            task_type: meta_str("type"),
            parent_id: meta_str("parent_id"),
            branch: meta_str("branch"),
            status_detail: meta_str("status_detail"),
            project: meta_str("project"),
            raw: raw_val,
        }
    }

    pub fn to_json(&self) -> Value {
        let mut val = self.raw.clone();
        if let Some(obj) = val.as_object_mut() {
            obj.insert("subject".into(), Value::String(self.subject.clone()));
            obj.insert(
                "status".into(),
                Value::String(self.status.as_str().to_string()),
            );
            obj.insert(
                "description".into(),
                Value::String(self.description.clone()),
            );

            let meta = obj
                .entry("metadata")
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
            if let Some(m) = meta.as_object_mut() {
                set_or_delete(m, "priority", self.priority.as_str());
                set_or_delete(m, "type", &self.task_type);
                set_or_delete(m, "parent_id", &self.parent_id);
                set_or_delete(m, "branch", &self.branch);
                set_or_delete(m, "status_detail", &self.status_detail);
            }
        }
        val
    }
}

fn set_or_delete(map: &mut serde_json::Map<String, Value>, key: &str, value: &str) {
    if value.is_empty() || value == "--" {
        map.remove(key);
    } else {
        map.insert(key.to_string(), Value::String(value.to_string()));
    }
}

#[derive(Debug, Clone)]
pub struct TaskList {
    pub id: String,
}

pub struct Store {
    base: PathBuf,
}

impl Store {
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("no home directory");
        Self {
            base: home.join(".claude").join("tasks"),
        }
    }

    pub fn list_task_lists(&self) -> Vec<TaskList> {
        let mut lists = Vec::new();
        let Ok(entries) = fs::read_dir(&self.base) else {
            return lists;
        };
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                lists.push(TaskList { id: name });
            }
        }
        // Named lists (non-UUID) first, then by name
        lists.sort_by(|a, b| {
            let a_uuid = is_uuid(&a.id);
            let b_uuid = is_uuid(&b.id);
            a_uuid.cmp(&b_uuid).then_with(|| a.id.cmp(&b.id))
        });
        lists
    }

    pub fn list_tasks(&self, list_id: &str) -> Vec<Task> {
        let dir = self.base.join(list_id);
        let Ok(entries) = fs::read_dir(&dir) else {
            return Vec::new();
        };
        let mut tasks: Vec<Task> = entries
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
            .filter_map(|e| {
                let data = fs::read_to_string(e.path()).ok()?;
                let val: Value = serde_json::from_str(&data).ok()?;
                Some(Task::from_raw(val))
            })
            .collect();
        tasks.sort_by(|a, b| {
            let a_num: u64 = a.id.parse().unwrap_or(u64::MAX);
            let b_num: u64 = b.id.parse().unwrap_or(u64::MAX);
            a_num.cmp(&b_num)
        });
        tasks
    }

    pub fn load_task(&self, list_id: &str, task_id: &str) -> Option<Task> {
        let path = self.base.join(list_id).join(format!("{task_id}.json"));
        let data = fs::read_to_string(path).ok()?;
        let val: Value = serde_json::from_str(&data).ok()?;
        Some(Task::from_raw(val))
    }

    pub fn save_task(&self, list_id: &str, task: &Task) -> Result<(), String> {
        let path = self.base.join(list_id).join(format!("{}.json", task.id));
        let json =
            serde_json::to_string_pretty(&task.to_json()).map_err(|e| format!("serialize: {e}"))?;
        atomic_write(&path, &json)
    }

    pub fn create_task(&self, list_id: &str, task: &Task) -> Result<Task, String> {
        let dir = self.base.join(list_id);
        let next_id = self.next_id(list_id);

        let mut new_task = task.clone();
        new_task.id = next_id.to_string();

        // Build raw JSON
        let mut raw = serde_json::Map::new();
        raw.insert("id".into(), Value::String(new_task.id.clone()));
        raw.insert("subject".into(), Value::String(new_task.subject.clone()));
        raw.insert(
            "description".into(),
            Value::String(new_task.description.clone()),
        );
        raw.insert(
            "status".into(),
            Value::String(new_task.status.as_str().to_string()),
        );

        let mut meta = serde_json::Map::new();
        set_or_delete(&mut meta, "priority", new_task.priority.as_str());
        set_or_delete(&mut meta, "type", &new_task.task_type);
        set_or_delete(&mut meta, "parent_id", &new_task.parent_id);
        if !meta.is_empty() {
            raw.insert("metadata".into(), Value::Object(meta));
        }

        new_task.raw = Value::Object(raw);

        let path = dir.join(format!("{next_id}.json"));
        let json = serde_json::to_string_pretty(&new_task.to_json())
            .map_err(|e| format!("serialize: {e}"))?;
        atomic_write(&path, &json)?;
        Ok(new_task)
    }

    pub fn delete_task(&self, list_id: &str, task_id: &str) -> Result<(), String> {
        let path = self.base.join(list_id).join(format!("{task_id}.json"));
        fs::remove_file(path).map_err(|e| format!("delete: {e}"))
    }

    fn next_id(&self, list_id: &str) -> u64 {
        // Check .highwatermark first
        let hwm_path = self.base.join(list_id).join(".highwatermark");
        let hwm = fs::read_to_string(&hwm_path)
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);

        // Also scan existing files
        let max_file = self
            .list_tasks(list_id)
            .iter()
            .filter_map(|t| t.id.parse::<u64>().ok())
            .max()
            .unwrap_or(0);

        let next = hwm.max(max_file) + 1;
        let _ = fs::write(&hwm_path, next.to_string());
        next
    }

    pub fn discover_lists(&self, cwd: &str) -> Vec<TaskList> {
        let mut found = Vec::new();

        // 1. Check CLAUDE_CODE_TASK_LIST_ID env var
        if let Ok(id) = std::env::var("CLAUDE_CODE_TASK_LIST_ID")
            && !id.is_empty()
            && self.base.join(&id).exists()
        {
            found.push(TaskList { id });
            return found;
        }

        // 2. Check settings files
        for name in &["settings.local.json", "settings.json"] {
            let settings_path = Path::new(cwd).join(".claude").join(name);
            if let Ok(data) = fs::read_to_string(&settings_path)
                && let Ok(val) = serde_json::from_str::<Value>(&data)
                && let Some(id) = val
                    .get("env")
                    .and_then(|e| e.get("CLAUDE_CODE_TASK_LIST_ID"))
                    .and_then(|v| v.as_str())
                && !id.is_empty()
                && self.base.join(id).exists()
            {
                found.push(TaskList { id: id.to_string() });
                return found;
            }
        }

        // 3. Scan all lists for matching project
        let all = self.list_task_lists();
        for list in &all {
            let tasks = self.list_tasks(&list.id);
            for task in &tasks {
                if !task.project.is_empty() && cwd.contains(&task.project) {
                    found.push(list.clone());
                    break;
                }
            }
        }

        if found.is_empty() {
            return all;
        }
        found
    }
}

fn is_uuid(s: &str) -> bool {
    s.len() > 30 && s.contains('-')
}

fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let dir = path.parent().ok_or("no parent directory")?;
    let tmp = dir.join(format!(".tmp-{}", std::process::id()));
    fs::write(&tmp, content).map_err(|e| format!("write tmp: {e}"))?;
    fs::rename(&tmp, path).map_err(|e| format!("rename: {e}"))
}

// Filtering and sorting

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusFilter {
    All,
    Active,
    Pending,
    InProgress,
    Completed,
}

impl StatusFilter {
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Active,
            Self::Active => Self::Pending,
            Self::Pending => Self::InProgress,
            Self::InProgress => Self::Completed,
            Self::Completed => Self::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Active => "active",
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
        }
    }

    pub fn matches(self, status: &Status) -> bool {
        match self {
            Self::All => true,
            Self::Active => *status != Status::Completed,
            Self::Pending => *status == Status::Pending,
            Self::InProgress => *status == Status::InProgress,
            Self::Completed => *status == Status::Completed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOrder {
    Id,
    Priority,
    Subject,
}

impl SortOrder {
    pub fn next(self) -> Self {
        match self {
            Self::Id => Self::Priority,
            Self::Priority => Self::Subject,
            Self::Subject => Self::Id,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Priority => "priority",
            Self::Subject => "subject",
        }
    }
}

pub fn filter_and_sort(
    tasks: &[Task],
    status_filter: StatusFilter,
    sort_order: SortOrder,
    show_closed: bool,
    query: &str,
) -> Vec<Task> {
    let mut result: Vec<Task> = tasks
        .iter()
        .filter(|t| {
            if !show_closed
                && t.status == Status::Completed
                && status_filter != StatusFilter::Completed
            {
                return false;
            }
            if !status_filter.matches(&t.status) {
                return false;
            }
            if !query.is_empty() {
                let q = query.to_lowercase();
                if !t.subject.to_lowercase().contains(&q) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    match sort_order {
        SortOrder::Id => result.sort_by(|a, b| {
            let a_num: u64 = a.id.parse().unwrap_or(u64::MAX);
            let b_num: u64 = b.id.parse().unwrap_or(u64::MAX);
            a_num.cmp(&b_num)
        }),
        SortOrder::Priority => result.sort_by_key(|a| a.priority.sort_key()),
        SortOrder::Subject => result.sort_by_key(|a| a.subject.to_lowercase()),
    }

    result
}

pub struct TreeRow {
    pub task: Task,
    pub depth: u8,
    pub is_last: bool,
}

pub fn tree_order(tasks: &[Task]) -> Vec<TreeRow> {
    let mut roots: Vec<&Task> = Vec::new();
    let mut children_map: std::collections::HashMap<&str, Vec<&Task>> =
        std::collections::HashMap::new();

    for t in tasks {
        if t.parent_id.is_empty() {
            roots.push(t);
        } else {
            children_map
                .entry(t.parent_id.as_str())
                .or_default()
                .push(t);
        }
    }

    let mut result = Vec::new();
    for root in &roots {
        result.push(TreeRow {
            task: (*root).clone(),
            depth: 0,
            is_last: false,
        });
        if let Some(kids) = children_map.get(root.id.as_str()) {
            for (i, kid) in kids.iter().enumerate() {
                result.push(TreeRow {
                    task: (*kid).clone(),
                    depth: 1,
                    is_last: i == kids.len() - 1,
                });
            }
        }
    }
    result
}
