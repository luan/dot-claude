#!/usr/bin/env python3
"""
PostToolUse hook: auto-lint beads issues after `bd create` commands.
Skips bulk creates (`bd create --file`).
"""

import json
import shlex
import subprocess
import sys


def print_success(message: str) -> None:
    print(f"\033[34m🔹 {message}\033[0m", file=sys.stderr)


def print_error(message: str) -> None:
    print(f"\033[31m⛔ {message}\033[0m", file=sys.stderr)


def is_bd_create(command: str) -> bool:
    """Check if command is a bd create (not bulk). Uses shlex for robustness."""
    try:
        tokens = shlex.split(command)
    except ValueError:
        return False

    if "bd" not in tokens:
        return False

    bd_idx = tokens.index("bd")
    remaining = tokens[bd_idx + 1 :]

    if "create" not in remaining:
        return False

    if "--file" in remaining or "-f" in remaining:
        return False

    return True


def main() -> None:
    try:
        if sys.stdin.isatty():
            sys.exit(1)

        input_data = json.load(sys.stdin)

        if input_data.get("tool_name") != "Bash":
            sys.exit(0)

        command = input_data.get("tool_input", {}).get("command", "")

        if not is_bd_create(command):
            sys.exit(0)

        result = subprocess.run(
            ["bd", "lint"],
            capture_output=True,
            text=True,
            timeout=10,
        )

        if result.returncode != 0:
            print_error("BLOCKING: bd lint failed after bd create")
            output = (result.stdout + result.stderr).strip()
            if output:
                print(output, file=sys.stderr)
            sys.exit(2)

        print_success("bd lint passed")
        sys.exit(0)

    except json.JSONDecodeError:
        sys.exit(0)
    except FileNotFoundError:
        print_error("bd not found in PATH")
        sys.exit(1)
    except subprocess.TimeoutExpired:
        print_error("bd lint timed out (10s)")
        sys.exit(1)


if __name__ == "__main__":
    main()
