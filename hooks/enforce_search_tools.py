#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# dependencies = []
# ///
"""PreToolUse hook: block raw grep/find in Bash, suggest rg/fd or built-in tools."""

import json
import re
import sys

# Match grep/find in command position: after start-of-line, pipe, chain (&&/||/;),
# subshell ($( or `), optionally preceded by env assignments (FOO=bar) or
# command wrappers (command, env, sudo, xargs). Uses \b to avoid matching
# substrings like grep_tool. Does NOT match git grep or --grep= (no separator).
PATTERN = re.compile(
    r"(?:^|[|;&\n]|\$\(|`)\s*"
    r"(?:\S+=\S*\s+)*"
    r"(?:(?:command|env|sudo|xargs)\s+)*"
    r"(grep|find)\b",
    re.IGNORECASE | re.MULTILINE,
)

MESSAGE = """\
**[enforce-search-tools]**
Do not use raw `grep` or `find` in Bash.

- **Text search** → use the **Grep** tool (ripgrep-backed, correct permissions)
- **File search** → use the **Glob** tool (fast pattern matching)
- **Bash text search** → `rg` (ripgrep) is always available
- **Bash file search** → `fd` (fd-find) is always available
- **Semantic search** → `ck` for concept-level code search
"""


_HEREDOC = re.compile(r"<<'?(\w+)'?\n.*?\n\1\b", re.DOTALL)
_SINGLE_QUOTED = re.compile(r"'[^']*'")


def _strip_literals(cmd):
    """Remove heredoc bodies and single-quoted strings so literal text
    mentioning grep/find (e.g. in commit messages) doesn't trigger a match."""
    cmd = _HEREDOC.sub("", cmd)
    cmd = _SINGLE_QUOTED.sub("''", cmd)
    return cmd


def main():
    try:
        data = json.load(sys.stdin)
    except (json.JSONDecodeError, EOFError):
        sys.exit(0)

    tool_name = data.get("tool_name", "")
    if tool_name != "Bash":
        sys.exit(0)

    command = data.get("tool_input", {}).get("command", "")
    if not PATTERN.search(_strip_literals(command)):
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
