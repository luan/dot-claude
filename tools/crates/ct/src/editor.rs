use crate::store::{Priority, Status, Task};
use serde_json::Value;
use std::fmt::Write;

pub fn marshal_task(task: &Task) -> String {
    let mut b = String::new();
    writeln!(b, "Subject: {}", task.subject).unwrap();
    writeln!(b, "Status: {}", task.status.as_str()).unwrap();
    if !task.active_form.is_empty() {
        writeln!(b, "ActiveForm: {}", task.active_form).unwrap();
    }

    // Show all metadata fields
    if let Some(meta) = task.raw.get("metadata").and_then(|v| v.as_object()) {
        writeln!(b).unwrap();
        writeln!(b, "## Metadata").unwrap();
        let mut keys: Vec<&String> = meta.keys().collect();
        keys.sort();
        for key in keys {
            let val = match &meta[key] {
                Value::String(s) => s.clone(),
                Value::Null => continue,
                other => other.to_string(),
            };
            writeln!(b, "{key}: {val}").unwrap();
        }
    }

    writeln!(b).unwrap();
    writeln!(b, "## Description").unwrap();
    if !task.description.is_empty() {
        writeln!(b, "{}", task.description).unwrap();
    }

    writeln!(b).unwrap();
    writeln!(b, "# ID: {}", task.id).unwrap();
    writeln!(
        b,
        "# Edit fields above. Lines starting with '#' are ignored."
    )
    .unwrap();
    writeln!(
        b,
        "# Empty metadata values are removed. Use -- to explicitly clear."
    )
    .unwrap();
    b
}

pub fn unmarshal_task(text: &str, original: &Task) -> Result<Task, String> {
    let mut subject = String::new();
    let mut status_str = String::new();
    let mut active_form = String::new();
    let mut metadata = serde_json::Map::new();
    let mut description_lines = Vec::new();

    #[derive(PartialEq)]
    enum Section {
        Header,
        Metadata,
        Description,
    }
    let mut section = Section::Header;

    for line in text.lines() {
        if line.starts_with('#') && !line.starts_with("## ") {
            continue;
        }

        // Section transitions
        if line.trim() == "## Metadata" {
            section = Section::Metadata;
            continue;
        }
        if line.trim() == "## Description" {
            section = Section::Description;
            continue;
        }

        match section {
            Section::Header => {
                if line.is_empty() {
                    continue;
                }
                if let Some((key, val)) = line.split_once(':') {
                    let key = key.trim();
                    let val = val.trim();
                    match key {
                        "Subject" => subject = val.to_string(),
                        "Status" => status_str = val.to_string(),
                        "ActiveForm" => active_form = val.to_string(),
                        _ => {}
                    }
                }
            }
            Section::Metadata => {
                if line.is_empty() {
                    continue;
                }
                if let Some((key, val)) = line.split_once(':') {
                    let key = key.trim().to_string();
                    let val = val.trim();
                    if val.is_empty() || val == "--" {
                        metadata.insert(key, Value::Null); // mark for deletion
                    } else {
                        metadata.insert(key, Value::String(val.to_string()));
                    }
                }
            }
            Section::Description => {
                description_lines.push(line);
            }
        }
    }

    if subject.is_empty() {
        return Err("subject is required".to_string());
    }

    let mut task = original.clone();
    task.subject = subject;
    task.description = description_lines.join("\n").trim().to_string();
    if !status_str.is_empty() {
        task.status = Status::from_str(&status_str);
    }
    task.active_form = active_form;

    // Sync derived fields from metadata
    if let Some(Value::String(p)) = metadata.get("priority") {
        task.priority = Priority::from_str(p);
    }
    if let Some(Value::String(t)) = metadata.get("type") {
        task.task_type = t.clone();
    }
    if let Some(Value::String(b)) = metadata.get("branch") {
        task.branch = b.clone();
    }
    if let Some(Value::String(p)) = metadata.get("parent_id") {
        task.parent_id = p.clone();
    }

    // Merge metadata into raw JSON
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
        if !task.active_form.is_empty() {
            obj.insert(
                "activeForm".to_string(),
                Value::String(task.active_form.clone()),
            );
        }

        let raw_meta = obj
            .entry("metadata")
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        if let Some(m) = raw_meta.as_object_mut() {
            for (key, val) in &metadata {
                if val.is_null() {
                    m.remove(key);
                } else {
                    m.insert(key.clone(), val.clone());
                }
            }
        }
    }
    task.raw = raw;

    Ok(task)
}
