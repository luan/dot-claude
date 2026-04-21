use std::path::PathBuf;

use anyhow::Result;
use serde_json::Value;
use sym::hook::{
    ClaudeSettings, Suggestion, append_unique_hook_group, claude_hook_entries, detect_search_command,
    emit_nudge, emit_remind, hook_group_has_marker, lookup_hook_adapter, merge_claude_hooks,
    remove_claude_hooks, split_shellish,
};

#[test]
fn detect_ripgrep_and_find_queries() {
    let rg = detect_search_command(&["rg", "-n", "HandleRegister", "."], "");
    assert!(rg.replacement.contains("sym search HandleRegister"));
    assert_eq!(rg.tool, "rg");

    let grep = detect_search_command(&["grep", "-rn", "-e", "OpenStore", "src/"], "");
    assert!(grep.replacement.contains("OpenStore"));

    let find = detect_search_command(&["find", ".", "-name", "UserRepo.go"], "");
    assert_eq!(find.tool, "find");
    assert!(!find.replacement.is_empty());
}

#[test]
fn detect_skips_noise_and_non_shell_tools() {
    assert!(detect_search_command(&["rg", "-n", "ab", "."], "").replacement.is_empty());
    assert!(detect_search_command(&["find", ".", "-name", "*.log"], "").replacement.is_empty());
    assert!(detect_search_command(&["rg", "-n", r"^(foo|bar)+\s*$", "src/"], "")
        .replacement
        .is_empty());
    assert!(detect_search_command(&["rg", "something"], "Edit").replacement.is_empty());
    let fields = split_shellish("ls | wc -l");
    assert!(detect_search_command(&fields.iter().map(String::as_str).collect::<Vec<_>>(), "")
        .replacement
        .is_empty());
}

#[test]
fn emit_nudge_shapes() -> Result<()> {
    let suggestion = Suggestion {
        tool: "rg".into(),
        replacement: "sym search Foo".into(),
        why: "ranked symbol results".into(),
    };

    let claude = emit_nudge("claude-code", &["rg", "Foo"], &suggestion)?;
    let value: Value = serde_json::from_str(&claude.stdout)?;
    assert!(value.get("decision").is_none());
    assert!(value.get("systemMessage").is_none());
    assert_eq!(value["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    assert_eq!(value["hookSpecificOutput"]["permissionDecision"], "allow");
    assert!(value["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap()
        .contains("sym search Foo"));

    let text = emit_nudge("text", &["rg", "Foo"], &suggestion)?;
    assert!(text.stdout.is_empty());
    assert!(text.stderr.contains("sym search Foo"));

    let silent = emit_nudge("claude-code", &["ls", "-la"], &Suggestion::default())?;
    assert!(silent.stdout.is_empty());
    assert!(silent.stderr.is_empty());
    Ok(())
}

#[test]
fn emit_remind_shapes() -> Result<()> {
    let text = emit_remind("text")?;
    assert!(text.contains("sym search"));

    let json = emit_remind("json")?;
    let value: Value = serde_json::from_str(&json)?;
    assert!(value.get("systemMessage").is_some());

    let claude = emit_remind("claude-code")?;
    let value: Value = serde_json::from_str(&claude)?;
    assert!(value.get("systemMessage").is_none());
    assert_eq!(value["hookSpecificOutput"]["hookEventName"], "SessionStart");
    assert!(value["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap()
        .contains("sym search"));
    Ok(())
}

#[test]
fn claude_install_is_idempotent_and_uninstall_preserves_user_hooks() {
    let mut settings = ClaudeSettings::new();
    settings.raw.insert("model".into(), Value::String("sonnet".into()));
    settings.raw.insert(
        "hooks".into(),
        serde_json::json!({
            "PreToolUse": [{
                "matcher": "Bash",
                "hooks": [{"type": "command", "command": "user-owned-thing"}]
            }]
        }),
    );

    merge_claude_hooks(&mut settings);
    merge_claude_hooks(&mut settings);

    let hooks = settings.raw["hooks"].as_object().unwrap();
    assert_eq!(hooks["PreToolUse"].as_array().unwrap().len(), 2);
    assert_eq!(hooks["SessionStart"].as_array().unwrap().len(), 1);

    remove_claude_hooks(&mut settings);
    let hooks = settings.raw["hooks"].as_object().unwrap();
    assert_eq!(settings.raw["model"], "sonnet");
    assert_eq!(hooks["PreToolUse"].as_array().unwrap().len(), 1);
    assert!(hooks.get("SessionStart").is_none());
}

#[test]
fn claude_install_migrates_old_user_prompt_submit_entry() {
    let mut settings = ClaudeSettings::new();
    settings.raw.insert(
        "hooks".into(),
        serde_json::json!({
            "UserPromptSubmit": [
                {"hooks": [{"type": "command", "command": "sym hook remind --format=claude-code", "marker": "sym-hook", "timeout": 5}]},
                {"hooks": [{"type": "command", "command": "user-unrelated-hook"}]}
            ]
        }),
    );

    merge_claude_hooks(&mut settings);

    let hooks = settings.raw["hooks"].as_object().unwrap();
    let user_prompt = hooks["UserPromptSubmit"].as_array().unwrap();
    assert_eq!(user_prompt.len(), 1);
    assert!(!hook_group_has_marker(&user_prompt[0], "sym-hook"));
    let session_start = hooks["SessionStart"].as_array().unwrap();
    assert_eq!(session_start.len(), 1);
    assert!(hook_group_has_marker(&session_start[0], "sym-hook"));
}

#[test]
fn hook_helpers_and_unknown_adapter_error() {
    let (pre_tool, _) = claude_hook_entries();
    let appended = append_unique_hook_group(&Value::Array(vec![]), pre_tool.clone());
    let appended = append_unique_hook_group(&Value::Array(appended), pre_tool);
    assert_eq!(appended.len(), 1);

    let err = lookup_hook_adapter("cursor").unwrap_err();
    assert!(err.to_string().contains("docs/AGENT_HOOKS.md"));
}

#[test]
fn settings_roundtrip_to_disk() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = PathBuf::from(dir.path()).join("settings.json");
    let mut settings = ClaudeSettings::new();
    merge_claude_hooks(&mut settings);
    sym::hook::write_claude_settings(&path, &settings)?;
    let loaded = sym::hook::load_claude_settings(&path)?;
    assert_eq!(loaded.raw["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
    Ok(())
}

#[test]
fn install_preserves_user_key_order_and_is_byte_idempotent() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = PathBuf::from(dir.path()).join("settings.json");

    // User's existing settings with a deliberate key order — $schema first,
    // then env, then permissions. Alphabetical sort would scramble this.
    let original = b"{\n  \"$schema\": \"https://example.com/schema.json\",\n  \"env\": {\n    \"FOO\": \"bar\"\n  },\n  \"permissions\": {\n    \"allow\": []\n  }\n}\n";
    std::fs::write(&path, original)?;

    // First install writes the merged result.
    let mut settings = sym::hook::load_claude_settings(&path)?;
    merge_claude_hooks(&mut settings);
    sym::hook::write_claude_settings(&path, &settings)?;

    let after_first = std::fs::read(&path)?;
    let parsed: serde_json::Value = serde_json::from_slice(&after_first)?;
    let keys: Vec<String> = parsed.as_object().unwrap().keys().cloned().collect();
    assert_eq!(keys, ["$schema", "env", "permissions", "hooks"]);

    // Second install with no changes must not rewrite the file.
    let mtime_before = std::fs::metadata(&path)?.modified()?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    let mut settings = sym::hook::load_claude_settings(&path)?;
    merge_claude_hooks(&mut settings);
    sym::hook::write_claude_settings(&path, &settings)?;
    let mtime_after = std::fs::metadata(&path)?.modified()?;
    assert_eq!(mtime_before, mtime_after, "no-op install must not rewrite");

    Ok(())
}
