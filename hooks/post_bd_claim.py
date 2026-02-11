#!/usr/bin/env -S uv run --script
"""
PostToolUse hook: register session→ticket mapping for statusline.
Detects `bd update <id> --claim` and `bd close <id>` commands.
"""

import json
import os
import shlex
import sys


def parse_bd_command(command: str) -> tuple[str, str] | None:
    """
    Parse bd command and return (subcommand, ticket_id) or None.
    Returns ('claim', 'bd-abc123') for `bd update <id> --claim`
    Returns ('close', 'bd-abc123') for `bd close <id>`
    """
    try:
        tokens = shlex.split(command)
    except ValueError:
        return None

    if "bd" not in tokens:
        return None

    bd_idx = tokens.index("bd")
    remaining = tokens[bd_idx + 1 :]

    if not remaining:
        return None

    subcommand = remaining[0]

    # bd update <id> --claim
    if subcommand == "update" and "--claim" in remaining:
        if len(remaining) < 2:
            return None
        ticket_id = remaining[1]
        return ("claim", ticket_id)

    # bd close <id>
    if subcommand == "close":
        if len(remaining) < 2:
            return None
        ticket_id = remaining[1]
        return ("close", ticket_id)

    return None


def main() -> None:
    try:
        if sys.stdin.isatty():
            sys.exit(0)

        input_data = json.load(sys.stdin)

        if input_data.get("tool_name") != "Bash":
            sys.exit(0)

        command = input_data.get("tool_input", {}).get("command", "")
        session_id = input_data.get("session_id", "")

        if not session_id:
            sys.exit(0)

        result = parse_bd_command(command)
        if not result:
            sys.exit(0)

        action, ticket_id = result
        mapping_file = f"/tmp/claude-beads-task-{session_id}"

        if action == "claim":
            with open(mapping_file, "w") as f:
                f.write(ticket_id)
        elif action == "close":
            try:
                os.remove(mapping_file)
            except FileNotFoundError:
                pass

        sys.exit(0)

    except json.JSONDecodeError:
        sys.exit(0)
    except Exception:
        sys.exit(0)


if __name__ == "__main__":
    main()
