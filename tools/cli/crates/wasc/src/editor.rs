use crate::store::{Priority, Status, Task};
use serde_json::Value;
use std::fmt::Write;

pub fn marshal_task(task: &Task) -> String {
    let mut b = String::new();
    writeln!(b, "Subject: {}", task.subject).unwrap();
    writeln!(b, "Status: {}", task.status.as_str()).unwrap();
    writeln!(b, "Priority: {}", task.priority.as_str()).unwrap();
    writeln!(b, "Type: {}", task.task_type).unwrap();
    writeln!(b, "Branch: {}", task.branch).unwrap();
    writeln!(b, "Parent: {}", task.parent_id).unwrap();
    writeln!(b).unwrap();
    if !task.description.is_empty() {
        writeln!(b, "{}", task.description).unwrap();
    }
    writeln!(b).unwrap();
    write!(b, "# ID: {}", task.id).unwrap();
    if !task.active_form.is_empty() {
        write!(b, " | ActiveForm: {}", task.active_form).unwrap();
    }
    writeln!(b).unwrap();
    writeln!(b, "# Lines starting with '#' are ignored.").unwrap();
    writeln!(
        b,
        "# Edit the fields above and the description below the blank line."
    )
    .unwrap();
    b
}

pub fn unmarshal_task(text: &str, original: &Task) -> Result<Task, String> {
    let mut header_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut past_header = false;

    for line in text.lines() {
        if line.starts_with('#') {
            continue;
        }
        if !past_header {
            if line.is_empty() {
                past_header = true;
            } else {
                header_lines.push(line);
            }
        } else {
            body_lines.push(line);
        }
    }

    let mut headers = std::collections::HashMap::new();
    for line in &header_lines {
        if let Some((key, val)) = line.split_once(':') {
            headers.insert(key.trim().to_string(), val.trim().to_string());
        }
    }

    let subject = headers.get("Subject").cloned().unwrap_or_default();
    if subject.is_empty() {
        return Err("subject is required".to_string());
    }

    let mut task = original.clone();
    task.subject = subject;
    task.description = body_lines.join("\n").trim().to_string();

    if let Some(s) = headers.get("Status")
        && !s.is_empty()
    {
        task.status = Status::from_str(s);
    }
    if let Some(p) = headers.get("Priority") {
        task.priority = Priority::from_str(p);
    }
    task.task_type = headers.get("Type").cloned().unwrap_or_default();
    task.branch = headers.get("Branch").cloned().unwrap_or_default();
    task.parent_id = headers.get("Parent").cloned().unwrap_or_default();

    // Merge back into raw JSON preserving unknown fields
    let mut raw = original.raw.clone();
    if let Some(obj) = raw.as_object_mut() {
        obj.insert("subject".to_string(), Value::String(task.subject.clone()));
        obj.insert(
            "status".to_string(),
            Value::String(task.status.as_str().to_string()),
        );
        obj.insert(
            "description".to_string(),
            Value::String(task.description.clone()),
        );
        let meta = obj
            .entry("metadata")
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        if let Some(m) = meta.as_object_mut() {
            set_or_delete(m, "priority", task.priority.as_str());
            set_or_delete(m, "type", &task.task_type);
            set_or_delete(m, "branch", &task.branch);
            set_or_delete(m, "parent_id", &task.parent_id);
        }
    }
    task.raw = raw;

    Ok(task)
}

fn set_or_delete(map: &mut serde_json::Map<String, Value>, key: &str, value: &str) {
    if value.is_empty() || value == "--" {
        map.remove(key);
    } else {
        map.insert(key.to_string(), Value::String(value.to_string()));
    }
}
