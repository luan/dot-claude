#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# dependencies = []
# ///
"""PreToolUse hook: block raw grep/find in Bash, suggest rg/fd or built-in tools."""

import json
import re
import sys

PATTERN = re.compile(r"^\s*(grep|find)\s", re.IGNORECASE)

MESSAGE = """\
**[enforce-search-tools]**
Do not use raw `grep` or `find` in Bash.

- **Text search** → use the **Grep** tool (ripgrep-backed, correct permissions)
- **File search** → use the **Glob** tool (fast pattern matching)
- **Bash text search** → `rg` (ripgrep) is always available
- **Bash file search** → `fd` (fd-find) is always available
- **Semantic search** → `ck` for concept-level code search
"""


def main():
    try:
        data = json.load(sys.stdin)
    except (json.JSONDecodeError, EOFError):
        sys.exit(0)

    tool_name = data.get("tool_name", "")
    if tool_name != "Bash":
        sys.exit(0)

    command = data.get("tool_input", {}).get("command", "")
    if not PATTERN.search(command):
        print("{}")
        sys.exit(0)

    result = {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
        },
        "systemMessage": MESSAGE,
    }
    print(json.dumps(result))
    sys.exit(0)


if __name__ == "__main__":
    main()
