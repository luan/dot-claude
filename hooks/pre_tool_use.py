#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# ///

import json
import sys
import re
from pathlib import Path


def allow():
    """Allow the tool call."""
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
        }
    }


def deny(reason):
    """Deny the tool call with a reason."""
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }


def respond(response):
    """Send response and exit."""
    print(json.dumps(response))
    sys.exit(0)


def is_dangerous_rm_command(command):
    """
    Comprehensive detection of dangerous rm commands.
    Matches various forms of rm -rf and similar destructive patterns.
    """
    # Normalize command by removing extra spaces and converting to lowercase
    normalized = " ".join(command.lower().split())

    # Pattern 1: Standard rm -rf variations
    patterns = [
        r"\brm\s+.*-[a-z]*r[a-z]*f",  # rm -rf, rm -fr, rm -Rf, etc.
        r"\brm\s+.*-[a-z]*f[a-z]*r",  # rm -fr variations
        r"\brm\s+--recursive\s+--force",  # rm --recursive --force
        r"\brm\s+--force\s+--recursive",  # rm --force --recursive
        r"\brm\s+-r\s+.*-f",  # rm -r ... -f
        r"\brm\s+-f\s+.*-r",  # rm -f ... -r
    ]

    # Check for dangerous patterns
    for pattern in patterns:
        if re.search(pattern, normalized):
            return True

    # Pattern 2: Check for rm with recursive flag targeting dangerous paths
    dangerous_paths = [
        r"/",  # Root directory
        r"/\*",  # Root with wildcard
        r"~",  # Home directory
        r"~/",  # Home directory path
        r"\$HOME",  # Home environment variable
        r"\.\.",  # Parent directory references
        r"\*",  # Wildcards in general rm -rf context
        r"\.",  # Current directory
        r"\.\s*$",  # Current directory at end of command
    ]

    if re.search(r"\brm\s+.*-[a-z]*r", normalized):  # If rm has recursive flag
        for path in dangerous_paths:
            if re.search(path, normalized):
                return True

    return False


def load_safe_commands():
    """Load safe commands from ~/.claude/safe file."""
    safe_file = Path.home() / ".claude" / "safe"
    safe_commands = set()

    try:
        if safe_file.exists():
            for line in safe_file.read_text().splitlines():
                line = line.strip()
                # Skip empty lines and comments
                if line and not line.startswith("#"):
                    safe_commands.add(line)
    except Exception:
        # If we can't read the file, fall back to a minimal safe set
        safe_commands = {"cat", "grep", "ls", "echo", "head", "tail", "sort", "uniq"}

    return safe_commands


def is_safe_pipe_command(command):
    """
    Check if a piped command only uses safe, allowlisted commands.
    Returns True if safe, False if potentially dangerous.
    """
    # Load safe commands from file
    safe_commands = load_safe_commands()

    # Split by pipes and check each command
    pipe_segments = command.split("|")

    for segment in pipe_segments:
        segment = segment.strip()
        if not segment:
            continue

        # Extract the base command (first word)
        parts = segment.split()
        if not parts:
            continue

        base_command = parts[0]

        # Remove common prefixes
        if base_command in ["sudo", "time", "nice", "nohup"]:
            if len(parts) > 1:
                base_command = parts[1]
            else:
                return False  # Incomplete command

        # Check if it's in our safe list
        if base_command not in safe_commands:
            # Also check for absolute paths to safe commands
            if "/" in base_command:
                command_name = base_command.split("/")[-1]
                if command_name not in safe_commands:
                    return False
            else:
                return False

    return True


def is_env_file_access(tool_name, tool_input):
    """
    Check if any tool is trying to access .env files containing sensitive data.
    """
    if tool_name in ["Read", "Edit", "MultiEdit", "Write", "Bash"]:
        # Check file paths for file-based tools
        if tool_name in ["Read", "Edit", "MultiEdit", "Write"]:
            file_path = tool_input.get("file_path", "")
            if ".env" in file_path and not file_path.endswith(".env.sample"):
                return True

        # Check bash commands for .env file access
        elif tool_name == "Bash":
            command = tool_input.get("command", "")
            # Pattern to detect .env file access (but allow .env.sample)
            env_patterns = [
                r"\b\.env\b(?!\.sample)",  # .env but not .env.sample
                r"cat\s+.*\.env\b(?!\.sample)",  # cat .env
                r"echo\s+.*>\s*\.env\b(?!\.sample)",  # echo > .env
                r"touch\s+.*\.env\b(?!\.sample)",  # touch .env
                r"cp\s+.*\.env\b(?!\.sample)",  # cp .env
                r"mv\s+.*\.env\b(?!\.sample)",  # mv .env
            ]

            for pattern in env_patterns:
                if re.search(pattern, command):
                    return True

    return False


def main():
    try:
        # Read JSON input from stdin
        input_data = json.load(sys.stdin)

        tool_name = input_data.get("tool_name", "")
        tool_input = input_data.get("tool_input", {})

        # Check for .env file access (blocks access to sensitive environment files)
        if is_env_file_access(tool_name, tool_input):
            respond(
                deny(
                    "Access to .env files containing sensitive data is prohibited. Use .env.sample for template files instead."
                )
            )

        # Check bash commands for safety
        if tool_name == "Bash":
            command = tool_input.get("command", "")

            # Block rm -rf commands with comprehensive pattern matching
            if is_dangerous_rm_command(command):
                respond(deny("Dangerous rm command detected and prevented"))

            # Check for unsafe piped commands
            if "|" in command and not is_safe_pipe_command(command):
                respond(
                    deny(
                        "Unsafe pipe command detected. Only allowlisted commands can be piped together."
                    )
                )

        # Allow all other operations
        respond(allow())

    except (json.JSONDecodeError, Exception):
        # Gracefully handle errors - allow by default
        respond(allow())


if __name__ == "__main__":
    main()
