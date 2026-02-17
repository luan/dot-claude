use std::fs;
use std::io::{self, IsTerminal, Read};

#[derive(Debug)]
pub struct Phase {
    pub phase: u32,
    pub title: String,
    pub tasks: Vec<String>,
    pub deps: Vec<u32>,
}

fn strip_frontmatter(content: &str) -> &str {
    if !content.starts_with("---\n") {
        return content;
    }
    let rest = &content[4..];
    if let Some(end) = rest.find("\n---\n") {
        &rest[end + 5..]
    } else if rest.ends_with("\n---") {
        ""
    } else {
        content
    }
}

fn parse_bold_phase(line: &str) -> Option<(u32, String)> {
    // **Phase N: Description**
    let s = line.strip_prefix("**Phase ")?.strip_suffix("**")?;
    let (num_str, title) = s.split_once(':')?;
    let num: u32 = num_str.trim().parse().ok()?;
    Some((num, title.trim().to_string()))
}

fn parse_heading_phase(line: &str) -> Option<(u32, String)> {
    // ### Phase N: Description
    let s = line.strip_prefix("### Phase ")?;
    let (num_str, title) = s.split_once(':')?;
    let num: u32 = num_str.trim().parse().ok()?;
    Some((num, title.trim().to_string()))
}

fn parse_numbered_item(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let dot_pos = trimmed.find('.')?;
    let num_str = &trimmed[..dot_pos];
    if !num_str.chars().all(|c| c.is_ascii_digit()) || num_str.is_empty() {
        return None;
    }
    let rest = trimmed[dot_pos + 1..].trim();
    if rest.is_empty() {
        return None;
    }
    Some(rest.to_string())
}

fn is_sub_item(line: &str) -> Option<String> {
    // Indented line starting with - or *
    if !line.starts_with("  ") && !line.starts_with('\t') {
        return None;
    }
    let trimmed = line.trim();
    let rest = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))?;
    Some(rest.to_string())
}

fn is_independent(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("independent of") || lower.contains("no dependency")
}

pub fn parse_phases(content: &str) -> Vec<Phase> {
    let body = strip_frontmatter(content);
    let lines: Vec<&str> = body.lines().collect();

    let mut phases: Vec<Phase> = Vec::new();
    let mut last_task_idx: Option<usize> = None;

    for line in &lines {
        let trimmed = line.trim();

        // Check for phase markers
        if let Some((num, title)) =
            parse_bold_phase(trimmed).or_else(|| parse_heading_phase(trimmed))
        {
            phases.push(Phase {
                phase: num,
                title,
                tasks: Vec::new(),
                deps: Vec::new(),
            });
            last_task_idx = None;
            continue;
        }

        let Some(current) = phases.last_mut() else {
            continue;
        };

        // Sub-items before numbered items
        if let Some(sub) = is_sub_item(line) {
            if let Some(idx) = last_task_idx
                && let Some(task) = current.tasks.get_mut(idx)
            {
                task.push_str(" — ");
                task.push_str(&sub);
            }
            continue;
        }

        // Numbered list items
        if let Some(text) = parse_numbered_item(trimmed) {
            current.tasks.push(text);
            last_task_idx = Some(current.tasks.len() - 1);
            continue;
        }
    }

    // Compute dependencies
    for i in 0..phases.len() {
        let combined = format!("{} {}", phases[i].title, phases[i].tasks.join(" "));
        if is_independent(&combined) || i == 0 {
            phases[i].deps = vec![];
        } else {
            phases[i].deps = vec![phases[i - 1].phase];
        }
    }

    phases
}

pub fn to_json(phases: &[Phase]) -> String {
    let mut out = String::from("[\n");
    for (i, p) in phases.iter().enumerate() {
        if i > 0 {
            out.push_str(",\n");
        }
        let tasks_json: Vec<String> = p
            .tasks
            .iter()
            .map(|t| {
                let escaped = t.replace('\\', "\\\\").replace('"', "\\\"");
                format!("\"{escaped}\"")
            })
            .collect();

        let deps_json: Vec<String> = p.deps.iter().map(|d| d.to_string()).collect();

        let title_escaped = p.title.replace('\\', "\\\\").replace('"', "\\\"");

        out.push_str(&format!(
            "  {{\n    \"phase\": {},\n    \"title\": \"{}\",\n    \"tasks\": [{}],\n    \"deps\": [{}]\n  }}",
            p.phase,
            title_escaped,
            tasks_json.join(", "),
            deps_json.join(", ")
        ));
    }
    out.push_str("\n]");
    out
}

pub fn run_phases(file_arg: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let content = if let Some(file) = file_arg {
        fs::read_to_string(file)?
    } else if !io::stdin().is_terminal() {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        println!("[]");
        return Ok(());
    };

    let phases = parse_phases(&content);
    println!("{}", to_json(&phases));
    Ok(())
}
