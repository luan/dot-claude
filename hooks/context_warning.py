#!/usr/bin/env -S uv run --script
"""PostToolUse hook: warn when context window exceeds threshold.

Reads context percentage from /tmp/claude-context-pct-{session_id}
(written by statusline, per-session to avoid collisions).
Uses exit code 2 (blocking) so Claude sees the warning.
Warns once per threshold crossing to avoid noise.
"""

import json
import os
import sys

WARN_THRESHOLD = 65
CRITICAL_THRESHOLD = 85


def main():
    try:
        if sys.stdin.isatty():
            return
        input_data = json.load(sys.stdin)
    except (json.JSONDecodeError, Exception):
        return

    sid = input_data.get("session_id", "")
    if not sid:
        return

    try:
        with open(f"/tmp/claude-context-pct-{sid}") as f:
            pct = int(f.read().strip())
    except (OSError, ValueError):
        return

    flag_file = f"/tmp/claude-context-warned-{sid}"

    # Read previous warning level (0=none, 1=warned, 2=critical)
    prev_level = 0
    try:
        with open(flag_file) as f:
            prev_level = int(f.read().strip())
    except (OSError, ValueError):
        pass

    if pct >= CRITICAL_THRESHOLD and prev_level < 2:
        with open(flag_file, "w") as f:
            f.write("2")
        print(
            f"⛔ CONTEXT {pct}% — Save state to beads notes NOW. "
            f"Wrap up current task and suggest fresh session.",
            file=sys.stderr,
        )
        sys.exit(2)
    elif pct >= WARN_THRESHOLD and prev_level < 1:
        with open(flag_file, "w") as f:
            f.write("1")
        print(
            f"⚠️  CONTEXT {pct}% — Consider saving progress to beads notes "
            f"and creating a plan for remaining work before context runs out.",
            file=sys.stderr,
        )
        sys.exit(2)


if __name__ == "__main__":
    main()
