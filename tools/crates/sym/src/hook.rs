use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow, bail};
use serde_json::{Value, json};

pub const CLAUDE_HOOK_MARKER: &str = "sym-hook";
const CLAUDE_NUDGE_CMD: &str = "sym hook nudge --format=claude-code";
const CLAUDE_REMIND_CMD: &str = "sym hook remind --format=claude-code";
const CLAUDE_HOOK_KEYS: &[&str] = &["PreToolUse", "SessionStart", "UserPromptSubmit"];

pub const REMINDER_TEXT: &str = "This project is indexed by sym. Prefer these commands before falling\nback to grep/find:\n\n  sym search <name>        ranked symbol search (add --file, --kind, --lang)\n  sym show <sym>           source for a symbol (or file:L1-L2)\n  sym investigate <sym>    kind-adaptive summary\n  sym impact <sym>         who depends on this?\n  sym trace <sym>          what does this depend on?\n  sym impls <sym>          who implements this interface/protocol?\n\nMulti-symbol: all of the above accept several names in one call, or pipe\nnewline-separated names via --stdin. JSON output is available on every\ncommand with --json.\n\nUse 'sym search --text <pattern>' only for literal text matches sym\ncan't resolve by symbol.";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Suggestion {
    pub replacement: String,
    pub why: String,
    pub tool: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderedOutput {
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Default)]
pub struct ClaudeSettings {
    pub raw: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone)]
pub struct HookAdapter {
    pub install: fn(&str, bool) -> Result<(String, String)>,
    pub uninstall: fn(&str, bool) -> Result<(String, String)>,
}

pub fn detect_search_command(fields: &[&str], tool_name: &str) -> Suggestion {
    if fields.is_empty() && tool_name.is_empty() {
        return Suggestion::default();
    }
    if !tool_name.is_empty() && !is_shell_tool_name(tool_name) {
        return Suggestion::default();
    }
    let Some(tool) = fields.first().map(|tool| basename(tool.trim())) else {
        return Suggestion::default();
    };

    match tool.as_str() {
        "rg" | "grep" | "egrep" | "fgrep" | "ack" | "ag" => {
            let query = extract_search_query(&fields[1..]);
            if query.is_empty() || !looks_like_code_query(&query) {
                return Suggestion::default();
            }
            Suggestion {
                tool,
                replacement: format!("sym search {}", sh_quote_if_needed(&query)),
                why: "Ranked symbol results with file+line, file-scoped with --file, JSON with --json. Faster than scanning every match.".into(),
            }
        }
        "find" => {
            let name = extract_find_name_arg(&fields[1..]);
            if name.is_empty() || !looks_like_code_query(&name) {
                return Suggestion::default();
            }
            Suggestion {
                tool,
                replacement: format!("sym search {}", sh_quote_if_needed(&name)),
                why: "sym search also matches by name and returns symbol locations, not just paths.".into(),
            }
        }
        "fd" | "fdfind" => {
            let query = extract_search_query(&fields[1..]);
            if query.is_empty() || !looks_like_code_query(&query) {
                return Suggestion::default();
            }
            Suggestion {
                tool,
                replacement: format!("sym search {}", sh_quote_if_needed(&query)),
                why: "sym indexes symbols by name; for file discovery use `sym ls --stats`.".into(),
            }
        }
        _ => Suggestion::default(),
    }
}

pub fn split_shellish(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote = None;

    let flush = |out: &mut Vec<String>, cur: &mut String| {
        if !cur.is_empty() {
            out.push(std::mem::take(cur));
        }
    };

    for ch in input.chars() {
        match quote {
            Some(q) => {
                if ch == q {
                    quote = None;
                } else {
                    cur.push(ch);
                }
            }
            None if ch == '\'' || ch == '"' => quote = Some(ch),
            None if ch == ' ' || ch == '\t' => flush(&mut out, &mut cur),
            None if ch == '|' || ch == ';' || ch == '&' => {
                flush(&mut out, &mut cur);
                return out;
            }
            None => cur.push(ch),
        }
    }
    flush(&mut out, &mut cur);
    out
}

pub fn read_nudge_input(args: &[String]) -> Result<(Vec<String>, String)> {
    if !args.is_empty() {
        return Ok((args.to_vec(), String::new()));
    }

    let stdin = io::stdin();
    let stat = stdin.lock();
    if stat.is_terminal() {
        return Ok((Vec::new(), String::new()));
    }

    let mut text = String::new();
    io::stdin().read_to_string(&mut text)?;
    let text = text.trim();
    if text.is_empty() {
        return Ok((Vec::new(), String::new()));
    }
    if text.starts_with('{') {
        let value: Value = serde_json::from_str(text)?;
        let tool_name = value
            .get("tool_name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let command = value
            .get("tool_input")
            .and_then(|input| input.get("command"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !tool_name.is_empty() || !command.is_empty() {
            return Ok((split_shellish(command), tool_name));
        }
    }
    Ok((split_shellish(text), String::new()))
}

pub fn emit_nudge(format: &str, fields: &[&str], suggestion: &Suggestion) -> Result<RenderedOutput> {
    if suggestion.replacement.is_empty() {
        return Ok(RenderedOutput::default());
    }
    let message = format!(
        "sym can answer this faster: `{}`. {}",
        suggestion.replacement, suggestion.why
    );
    let command = fields.join(" ");

    match format {
        "" | "claude-code" => Ok(RenderedOutput {
            stdout: serde_json::to_string_pretty(&json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "allow",
                    "permissionDecisionReason": "sym nudge",
                    "additionalContext": message,
                }
            }))? + "\n",
            stderr: String::new(),
        }),
        "text" => Ok(RenderedOutput {
            stdout: String::new(),
            stderr: format!("{message}\n"),
        }),
        "json" => Ok(RenderedOutput {
            stdout: serde_json::to_string_pretty(&json!({
                "suggest": suggestion.replacement,
                "why": suggestion.why,
                "tool": suggestion.tool,
                "command": command,
            }))? + "\n",
            stderr: String::new(),
        }),
        other => bail!("unknown --format {:?} (want: claude-code, text, json)", other),
    }
}

pub fn emit_remind(format: &str) -> Result<String> {
    match format {
        "" | "text" => Ok(format!("{REMINDER_TEXT}\n")),
        "json" => Ok(serde_json::to_string_pretty(&json!({
            "systemMessage": REMINDER_TEXT,
        }))? + "\n"),
        "claude-code" => Ok(serde_json::to_string_pretty(&json!({
            "hookSpecificOutput": {
                "hookEventName": "SessionStart",
                "additionalContext": REMINDER_TEXT,
            }
        }))? + "\n"),
        other => bail!("unknown --format {:?} (want: text, json, claude-code)", other),
    }
}

impl ClaudeSettings {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn claude_hook_entries() -> (Value, Value) {
    (
        json!({
            "matcher": "Bash",
            "hooks": [{
                "type": "command",
                "command": CLAUDE_NUDGE_CMD,
                "marker": CLAUDE_HOOK_MARKER,
                "timeout": 5,
            }]
        }),
        json!({
            "hooks": [{
                "type": "command",
                "command": CLAUDE_REMIND_CMD,
                "marker": CLAUDE_HOOK_MARKER,
                "timeout": 5,
            }]
        }),
    )
}

pub fn merge_claude_hooks(settings: &mut ClaudeSettings) {
    remove_claude_hooks(settings);
    let (pre_tool, session_start) = claude_hook_entries();
    let hooks = settings
        .raw
        .entry("hooks")
        .or_insert_with(|| Value::Object(Default::default()));
    let hooks = hooks.as_object_mut().expect("hooks must be object");

    let pre_tool_existing = hooks.remove("PreToolUse").unwrap_or_else(|| Value::Array(vec![]));
    hooks.insert(
        "PreToolUse".into(),
        Value::Array(append_unique_hook_group(&pre_tool_existing, pre_tool)),
    );
    let session_existing = hooks.remove("SessionStart").unwrap_or_else(|| Value::Array(vec![]));
    hooks.insert(
        "SessionStart".into(),
        Value::Array(append_unique_hook_group(&session_existing, session_start)),
    );
}

pub fn remove_claude_hooks(settings: &mut ClaudeSettings) {
    let Some(hooks) = settings.raw.get_mut("hooks").and_then(Value::as_object_mut) else {
        return;
    };

    for key in CLAUDE_HOOK_KEYS {
        let Some(entries) = hooks.get(*key).and_then(Value::as_array) else {
            continue;
        };
        let filtered = entries
            .iter()
            .filter(|entry| !hook_group_has_marker(entry, CLAUDE_HOOK_MARKER))
            .cloned()
            .collect::<Vec<_>>();
        if filtered.is_empty() {
            hooks.remove(*key);
        } else {
            hooks.insert((*key).into(), Value::Array(filtered));
        }
    }
    if hooks.is_empty() {
        settings.raw.remove("hooks");
    }
}

pub fn append_unique_hook_group(existing: &Value, group: Value) -> Vec<Value> {
    let mut arr = existing.as_array().cloned().unwrap_or_default();
    if arr.iter().any(|entry| hook_group_has_marker(entry, CLAUDE_HOOK_MARKER)) {
        return arr;
    }
    arr.push(group);
    arr
}

pub fn hook_group_has_marker(entry: &Value, marker: &str) -> bool {
    entry
        .get("hooks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|hook| hook.get("marker").and_then(Value::as_str) == Some(marker))
}

pub fn claude_settings_path(scope: &str) -> Result<PathBuf> {
    match scope {
        "project" => Ok(PathBuf::from(".claude/settings.json")),
        "user" => {
            let home = std::env::var("HOME").map_err(|_| anyhow!("cannot determine HOME"))?;
            Ok(PathBuf::from(home).join(".claude/settings.json"))
        }
        _ => bail!("--scope must be 'user' or 'project'"),
    }
}

pub fn load_claude_settings(path: &Path) -> Result<ClaudeSettings> {
    let mut settings = ClaudeSettings::new();
    let data = match fs::read(path) {
        Ok(data) => data,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(settings),
        Err(err) => return Err(err.into()),
    };
    if data.is_empty() {
        return Ok(settings);
    }
    settings.raw = serde_json::from_slice(&data)?;
    Ok(settings)
}

pub fn write_claude_settings(path: &Path, settings: &ClaudeSettings) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut data = serde_json::to_vec_pretty(&settings.raw)?;
    data.push(b'\n');
    if let Ok(existing) = fs::read(path)
        && existing == data
    {
        return Ok(());
    }
    fs::write(path, data)?;
    Ok(())
}

pub fn install_claude_code(scope: &str, dry_run: bool) -> Result<(String, String)> {
    let path = claude_settings_path(scope)?;
    let mut settings = load_claude_settings(&path)?;
    merge_claude_hooks(&mut settings);
    let summary = serde_json::to_string_pretty(&settings.raw)?;
    if !dry_run {
        write_claude_settings(&path, &settings)?;
    }
    Ok((path.display().to_string(), summary))
}

pub fn uninstall_claude_code(scope: &str, dry_run: bool) -> Result<(String, String)> {
    let path = claude_settings_path(scope)?;
    let mut settings = load_claude_settings(&path)?;
    remove_claude_hooks(&mut settings);
    let summary = serde_json::to_string_pretty(&settings.raw)?;
    if !dry_run {
        write_claude_settings(&path, &settings)?;
    }
    Ok((path.display().to_string(), summary))
}

pub fn lookup_hook_adapter(name: &str) -> Result<HookAdapter> {
    match name {
        "claude-code" => Ok(HookAdapter {
            install: install_claude_code,
            uninstall: uninstall_claude_code,
        }),
        _ => bail!(
            "unknown agent {:?} (supported: claude-code). For other agents see docs/AGENT_HOOKS.md — 'sym hook nudge' and 'sym hook remind' can be wired by hand into any agent's hook point.",
            name
        ),
    }
}

pub fn run_hook_install(agent: &str, scope: &str, dry_run: bool, uninstall: bool) -> Result<String> {
    let adapter = lookup_hook_adapter(agent)?;
    let (target, summary) = if uninstall {
        (adapter.uninstall)(scope, dry_run)?
    } else {
        (adapter.install)(scope, dry_run)?
    };
    if dry_run {
        return Ok(format!("[dry-run] would update {target}\n---\n{summary}\n"));
    }
    let verb = if uninstall { "removed" } else { "installed" };
    Ok(format!("sym hooks {verb} for {agent} ({scope} scope) -> {target}\n"))
}

fn is_shell_tool_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "bash" | "shell" | "sh" | "terminal" | "run"
    )
}

fn extract_search_query(args: &[&str]) -> String {
    let mut index = 0;
    while index < args.len() {
        let arg = args[index];
        if arg.is_empty() {
            index += 1;
            continue;
        }
        if matches!(arg, "-e" | "--regexp" | "--pattern") {
            if let Some(next) = args.get(index + 1) {
                return (*next).to_string();
            }
            index += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--regexp=") {
            return value.to_string();
        }
        if let Some(value) = arg.strip_prefix("--pattern=") {
            return value.to_string();
        }
        if arg.starts_with('-') {
            index += 1;
            continue;
        }
        return arg.to_string();
    }
    String::new()
}

fn extract_find_name_arg(args: &[&str]) -> String {
    for window in args.windows(2) {
        if matches!(window[0], "-name" | "-iname" | "-path" | "-ipath") {
            return window[1].to_string();
        }
    }
    String::new()
}

fn looks_like_code_query(query: &str) -> bool {
    let query = query.trim().trim_matches('"').trim_matches('\'');
    if query.len() < 3 || query.starts_with("*.") {
        return false;
    }
    if !query.chars().any(|ch| ch.is_ascii_alphabetic() || ch == '_') {
        return false;
    }
    let meta = query
        .chars()
        .filter(|ch| matches!(ch, '(' | ')' | '[' | ']' | '{' | '}' | '|' | '^' | '$' | '+' | '?' | '*' | '\\'))
        .count();
    meta * 2 <= query.len()
}

fn sh_quote_if_needed(input: &str) -> String {
    if input.is_empty() {
        return "''".into();
    }
    if input
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '/' | '-'))
    {
        return input.to_string();
    }
    format!("'{}'", input.replace('\'', "'\\''"))
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}
