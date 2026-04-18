use std::io::{self, Read};

use serde_json::json;

const MAX_MESSAGE_CHARS: usize = 8000;

pub fn run(decision: String, message: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let body = match message {
        Some(m) => m,
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };
    let body = truncate(body);

    let key = match decision.as_str() {
        "allow" => "additionalContext",
        "deny" => "permissionDecisionReason",
        other => return Err(format!("decision must be allow or deny, got {other}").into()),
    };

    let response = json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": decision,
            key: body,
        }
    });
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

fn truncate(s: String) -> String {
    if s.chars().count() <= MAX_MESSAGE_CHARS {
        return s;
    }
    let mut out: String = s.chars().take(MAX_MESSAGE_CHARS).collect();
    out.push_str("\n… (truncated)");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_passes_short_strings() {
        assert_eq!(truncate("hi".to_string()), "hi");
    }

    #[test]
    fn truncate_caps_long_strings() {
        let s = "x".repeat(MAX_MESSAGE_CHARS + 500);
        let out = truncate(s);
        assert!(out.len() < MAX_MESSAGE_CHARS + 64);
        assert!(out.ends_with("(truncated)"));
    }
}
