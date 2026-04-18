#!/bin/sh
# Inject a strong nudge to load apply_patch on turn 1.
# apply_patch (mcp__apply-patch__apply_patch) is a deferred MCP tool; without a
# ToolSearch call its schema isn't loaded and the model falls back to
# Edit/Write. This hook reminds the model to load it before editing.

cat <<'EOF'
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "File-edit tool: `mcp__apply-patch__apply_patch` is a deferred MCP tool. Before your first file edit in this session, call `ToolSearch` with query `select:mcp__apply-patch__apply_patch` to load its schema. Then use it for every file change — single-line, single-file, multi-file, creates, deletes, renames alike. Do not fall back to Edit/Write because a change is \"just one line\" or \"just one file\"; that is the documented failure mode. Use Edit/Write only when apply_patch genuinely cannot express the change (e.g. binary files) and say why in the same turn."
  }
}
EOF
