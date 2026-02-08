#!/usr/bin/env -S uv run --script
"""PostToolUse hook: warn when context window exceeds threshold.

Reads context percentage from /tmp/claude-context-pct-{session_id}
(written by statusline, per-session to avoid collisions).
"""

import json
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

    if pct >= CRITICAL_THRESHOLD:
        print(
            f"\n\033[31m⛔ CONTEXT {pct}% — Save state to beads notes NOW. "
            f"Wrap up current task and suggest fresh session.\033[0m",
            file=sys.stderr,
        )
    elif pct >= WARN_THRESHOLD:
        print(
            f"\n\033[33m⚠️  CONTEXT {pct}% — Consider saving progress to beads notes "
            f"and creating a plan for remaining work before context runs out.\033[0m",
            file=sys.stderr,
        )


if __name__ == "__main__":
    main()
