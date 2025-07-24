#!/usr/bin/env python3
"""Shared utilities for Claude Code hooks."""

import json
import sys
from typing import Any, Dict, NoReturn, Optional


# Pre-tool use response builders
def allow() -> Dict[str, Any]:
    """Allow the tool call."""
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
        }
    }


def deny(reason: str) -> Dict[str, Any]:
    """Deny the tool call with a reason."""
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }


def ask(reason: str) -> Dict[str, Any]:
    """Ask user for confirmation."""
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "ask",
            "permissionDecisionReason": reason,
        }
    }


def respond(response: Dict[str, Any]) -> NoReturn:
    """Send JSON response and exit."""
    print(json.dumps(response))
    sys.exit(0)


# Common error handling
def handle_hook_error(default_response: Optional[Dict[str, Any]] = None) -> NoReturn:
    """Handle errors in hooks gracefully."""
    if default_response:
        respond(default_response)
    else:
        sys.exit(0)


# Feedback helpers for post-tool hooks
def print_success(message: str) -> None:
    """Print success message to stderr with formatting."""
    print("", file=sys.stderr)
    print(f"\033[34m🔹 {message}\033[0m", file=sys.stderr)


def print_error(message: str) -> None:
    """Print error message to stderr with formatting."""
    print("", file=sys.stderr)
    print(f"\033[31m⛔ {message}\033[0m", file=sys.stderr)


def print_warning(message: str) -> None:
    """Print warning message to stderr with formatting."""
    print("", file=sys.stderr)
    print(f"\033[33m⚠️  {message}\033[0m", file=sys.stderr)


def print_info(message: str) -> None:
    """Print info message to stderr with formatting."""
    print("", file=sys.stderr)
    print(f"\033[36mℹ️  {message}\033[0m", file=sys.stderr)

