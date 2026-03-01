use std::fs;
use std::io::{self, IsTerminal, Read};

#[derive(Debug, PartialEq)]
pub struct Task {
    pub text: String,
    pub sub_tasks: Vec<String>,
}

#[derive(Debug)]
pub struct Phase {
    pub phase: u32,
    pub title: String,
    pub tasks: Vec<Task>,
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
                task.sub_tasks.push(sub);
            }
            continue;
        }

        // Numbered list items
        if let Some(text) = parse_numbered_item(trimmed) {
            current.tasks.push(Task {
                text,
                sub_tasks: Vec::new(),
            });
            last_task_idx = Some(current.tasks.len() - 1);
            continue;
        }
    }

    // Compute dependencies
    for i in 0..phases.len() {
        let task_texts: Vec<&str> = phases[i].tasks.iter().map(|t| t.text.as_str()).collect();
        let combined = format!("{} {}", phases[i].title, task_texts.join(" "));
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
                let text_escaped = t
                    .text
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t");
                let subs: Vec<String> = t
                    .sub_tasks
                    .iter()
                    .map(|s| {
                        let escaped = s
                            .replace('\\', "\\\\")
                            .replace('"', "\\\"")
                            .replace('\n', "\\n")
                            .replace('\r', "\\r")
                            .replace('\t', "\\t");
                        format!("\"{escaped}\"")
                    })
                    .collect();
                format!(
                    "{{\"text\": \"{text_escaped}\", \"sub_tasks\": [{}]}}",
                    subs.join(", ")
                )
            })
            .collect();

        let deps_json: Vec<String> = p.deps.iter().map(|d| d.to_string()).collect();

        let title_escaped = p
            .title
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");

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

#[cfg(test)]
mod tests {
    use super::*;

    fn phase_titles(phases: &[Phase]) -> Vec<&str> {
        phases.iter().map(|p| p.title.as_str()).collect()
    }

    #[test]
    fn empty_input_returns_no_phases() {
        assert!(parse_phases("").is_empty());
    }

    fn task(text: &str) -> Task {
        Task {
            text: text.to_string(),
            sub_tasks: Vec::new(),
        }
    }

    fn task_with_subs(text: &str, subs: &[&str]) -> Task {
        Task {
            text: text.to_string(),
            sub_tasks: subs.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn heading_style_phase_markers_parsed() {
        let input = "### Phase 1: Setup\n1. Install deps\n\n### Phase 2: Build\n1. Run build";
        let phases = parse_phases(input);
        assert_eq!(phase_titles(&phases), vec!["Setup", "Build"]);
        assert_eq!(phases[0].tasks, vec![task("Install deps")]);
        assert_eq!(phases[1].tasks, vec![task("Run build")]);
    }

    #[test]
    fn bold_style_phase_markers_parsed() {
        let input = "**Phase 1: Alpha**\n1. Step one\n\n**Phase 2: Beta**\n1. Step two";
        let phases = parse_phases(input);
        assert_eq!(phase_titles(&phases), vec!["Alpha", "Beta"]);
    }

    #[test]
    fn tasks_attributed_to_correct_phase() {
        let input = "### Phase 1: First\n1. task-a\n1. task-b\n\n### Phase 2: Second\n1. task-c";
        let phases = parse_phases(input);
        assert_eq!(phases[0].tasks, vec![task("task-a"), task("task-b")]);
        assert_eq!(phases[1].tasks, vec![task("task-c")]);
    }

    #[test]
    fn first_phase_has_no_deps() {
        let input = "### Phase 1: Only\n1. do it";
        let phases = parse_phases(input);
        assert!(phases[0].deps.is_empty());
    }

    #[test]
    fn sequential_phases_get_predecessor_dep() {
        let input =
            "### Phase 1: A\n1. first\n\n### Phase 2: B\n1. second\n\n### Phase 3: C\n1. third";
        let phases = parse_phases(input);
        assert!(phases[0].deps.is_empty());
        assert_eq!(phases[1].deps, vec![1]);
        assert_eq!(phases[2].deps, vec![2]);
    }

    #[test]
    fn independent_phrase_clears_dep() {
        let input =
            "### Phase 1: A\n1. first\n\n### Phase 2: B — independent of phase 1\n1. second";
        let phases = parse_phases(input);
        assert!(phases[1].deps.is_empty());
    }

    #[test]
    fn frontmatter_stripped_before_parsing() {
        let input = "---\ntopic: test\n---\n### Phase 1: Real\n1. actual task";
        let phases = parse_phases(input);
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].title, "Real");
    }

    #[test]
    fn sub_items_nested_under_parent_task() {
        let input = "### Phase 1: Setup\n1. Install deps\n  - use npm\n  - check versions";
        let phases = parse_phases(input);
        assert_eq!(
            phases[0].tasks,
            vec![task_with_subs(
                "Install deps",
                &["use npm", "check versions"]
            )]
        );
    }

    #[test]
    fn task_with_no_sub_items_has_empty_sub_tasks() {
        let input = "### Phase 1: Setup\n1. Install deps\n1. Run build";
        let phases = parse_phases(input);
        assert_eq!(
            phases[0].tasks,
            vec![task("Install deps"), task("Run build")]
        );
        assert!(phases[0].tasks[0].sub_tasks.is_empty());
        assert!(phases[0].tasks[1].sub_tasks.is_empty());
    }

    #[test]
    fn single_sub_item_nested() {
        let input = "### Phase 1: Setup\n1. Install deps\n  - use npm";
        let phases = parse_phases(input);
        assert_eq!(
            phases[0].tasks,
            vec![task_with_subs("Install deps", &["use npm"])]
        );
    }

    #[test]
    fn sub_items_on_multiple_tasks() {
        let input = "### Phase 1: Setup\n1. Install deps\n  - use npm\n1. Configure\n  - set env\n  - add secrets";
        let phases = parse_phases(input);
        assert_eq!(
            phases[0].tasks,
            vec![
                task_with_subs("Install deps", &["use npm"]),
                task_with_subs("Configure", &["set env", "add secrets"]),
            ]
        );
    }

    #[test]
    fn phase_number_preserved() {
        let input = "### Phase 3: Non-sequential\n1. task";
        let phases = parse_phases(input);
        assert_eq!(phases[0].phase, 3);
    }

    #[test]
    fn test_three_level_nesting() {
        let input = "### Phase 1: Foundation\n1. Setup database\n  - create schema\n  - add migrations\n1. Configure auth\n  - jwt setup\n  - session store\n\n### Phase 2: Features\n1. User CRUD\n  - create endpoint\n  - delete endpoint\n1. API gateway";

        let phases = parse_phases(input);

        // Correct phase count
        assert_eq!(phases.len(), 2);
        assert_eq!(phases[0].title, "Foundation");
        assert_eq!(phases[1].title, "Features");

        // Task count per phase
        assert_eq!(phases[0].tasks.len(), 2);
        assert_eq!(phases[1].tasks.len(), 2);

        // sub_tasks populated correctly
        assert_eq!(phases[0].tasks[0].text, "Setup database");
        assert_eq!(
            phases[0].tasks[0].sub_tasks,
            vec!["create schema", "add migrations"]
        );

        assert_eq!(phases[0].tasks[1].text, "Configure auth");
        assert_eq!(
            phases[0].tasks[1].sub_tasks,
            vec!["jwt setup", "session store"]
        );

        assert_eq!(phases[1].tasks[0].text, "User CRUD");
        assert_eq!(
            phases[1].tasks[0].sub_tasks,
            vec!["create endpoint", "delete endpoint"]
        );

        // Task with no sub-items stays flat
        assert_eq!(phases[1].tasks[1].text, "API gateway");
        assert!(phases[1].tasks[1].sub_tasks.is_empty());

        // to_json contains "sub_tasks" key with correct nesting
        let json = to_json(&phases);
        assert!(json.contains("\"sub_tasks\""));
        assert!(json.contains("\"create schema\""));
        assert!(json.contains("\"add migrations\""));
        assert!(json.contains("\"jwt setup\""));
        assert!(json.contains("\"session store\""));
        assert!(json.contains("\"create endpoint\""));
        assert!(json.contains("\"delete endpoint\""));
        // Flat task still emits empty sub_tasks array
        assert!(json.contains(r#""text": "API gateway", "sub_tasks": []"#));
    }

    #[test]
    fn to_json_emits_task_objects_with_sub_tasks() {
        let phases = vec![Phase {
            phase: 1,
            title: "Test".to_string(),
            tasks: vec![
                task("do thing"),
                task_with_subs("setup", &["install", "configure"]),
            ],
            deps: vec![],
        }];
        let json = to_json(&phases);
        assert!(json.contains(r#""text": "do thing""#));
        assert!(json.contains(r#""sub_tasks": []"#));
        assert!(json.contains(r#""text": "setup""#));
        assert!(json.contains(r#""sub_tasks": ["install", "configure"]"#));
    }

    #[test]
    fn to_json_escapes_newlines_and_tabs() {
        let phases = vec![Phase {
            phase: 1,
            title: "Test".to_string(),
            tasks: vec![task_with_subs("line1\nline2", &["sub\twith\ttabs"])],
            deps: vec![],
        }];
        let json = to_json(&phases);
        assert!(json.contains(r#""text": "line1\nline2""#));
        assert!(json.contains(r#""sub\twith\ttabs""#));
        assert!(!json.contains('\n') || !json.contains("line1\nline2"));
    }
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
