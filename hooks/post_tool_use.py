#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# dependencies = [
#   "pynvim",
# ]
# ///

import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Optional, Tuple

from pynvim import attach


def load_config_set(config_file: str) -> set:
    """Load configuration from a file, skipping comments and empty lines."""
    config_path = Path.home() / ".claude" / config_file
    items = set()

    try:
        if config_path.exists():
            for line in config_path.read_text().splitlines():
                line = line.strip()
                if line and not line.startswith("#"):
                    items.add(line.lower())  # Normalize to lowercase
    except Exception:
        pass  # Silently fail and use defaults

    return items


def print_success(message: str) -> None:
    """Print success message with formatting."""
    print("", file=sys.stderr)
    print(f"\033[34m🔹 {message}\033[0m", file=sys.stderr)


def print_error(message: str, details: str = "") -> None:
    """Print error message with formatting."""
    print("", file=sys.stderr)
    print(f"\033[31m⛔ {message}\033[0m", file=sys.stderr)
    if details:
        print(details, file=sys.stderr)


def print_warning(message: str) -> None:
    """Print warning message with formatting."""
    print("", file=sys.stderr)
    print(f"\033[33m⚠️  {message}\033[0m", file=sys.stderr)


def find_nvim_socket() -> Optional[str]:
    """Find an active Neovim socket for the current project."""
    try:
        # Get current working directory and generate hash
        cwd = Path.cwd()
        project_hash = hashlib.sha256(str(cwd).encode()).hexdigest()[:8]

        # Check project-specific socket file
        socket_file = Path(f"/tmp/nvim_socket_{project_hash}")
        if socket_file.exists():
            lines = socket_file.read_text().strip().split("\n")
            if len(lines) >= 2:
                socket_path, stored_cwd = lines[0], lines[1]
                # Verify this is the right project and socket still exists
                if stored_cwd == str(cwd) and Path(socket_path).is_socket():
                    return socket_path

        # Fallback: check any socket files for this project
        tmp_dir = Path("/tmp")
        if tmp_dir.exists():
            for socket_file in tmp_dir.glob("nvim_socket_*"):
                try:
                    lines = socket_file.read_text().strip().split("\n")
                    if len(lines) >= 2:
                        socket_path, stored_cwd = lines[0], lines[1]
                        if stored_cwd == str(cwd) and Path(socket_path).is_socket():
                            return socket_path
                except Exception:
                    continue

        # Final fallback: search for socket files in common locations
        possible_locations = ["/tmp", "/var/folders"]

        for base_path in possible_locations:
            if not Path(base_path).exists():
                continue

            try:
                result = subprocess.run(
                    [
                        "find",
                        base_path,
                        "-name",
                        "nvim.*",
                        "-type",
                        "s",
                        "-user",
                        os.getenv("USER", ""),
                    ],
                    capture_output=True,
                    text=True,
                    timeout=5,
                )

                if result.returncode == 0 and result.stdout.strip():
                    socket_path = result.stdout.strip().split("\n")[0]
                    return socket_path
            except (subprocess.TimeoutExpired, FileNotFoundError):
                continue

        return None
    except Exception:
        return None


def check_with_neovim(file_path: str, socket_path: str) -> Tuple[bool, str]:
    """Use Neovim to check the file for diagnostics."""
    try:
        # Load file in background without opening it
        nvim = attach("socket", path=socket_path)
        bufnr = nvim.funcs.bufnr(file_path)
        if bufnr == -1:
            buf = nvim.api.create_buf(True, False)
            bufnr = buf.handle
            nvim.api.buf_set_name(buf, file_path)
            nvim.funcs.bufload(buf)

        nvim.exec_lua(
            f'vim.api.nvim_buf_call({bufnr}, function() vim.cmd("edit | write") end)'
        )

        # Poll for LSP diagnostics with timeout
        max_wait = 15.0  # seconds
        interval = 0.1
        waited = 0
        diagnostics = []

        # Wait for LSP to be idle using lsp-status (required)
        while waited < max_wait:
            # Check if any LSP servers are still working
            lsp_messages = nvim.exec_lua('return require("lsp-status").messages()')
            active_messages = [msg for msg in lsp_messages if msg.get("progress")]

            # Exit if no active progress and minimum wait time passed
            if not active_messages and waited >= 1.0:
                break

            time.sleep(interval)
            waited += interval

        # Get final diagnostics after LSP completion
        diagnostics = nvim.exec_lua(
            f'return vim.diagnostic.get(vim.fn.bufnr("{file_path}"))'
        )
        if diagnostics:
            issues = []
            for diag in diagnostics:
                line = diag.get("lnum", 0) + 1
                message = diag.get("message", "Unknown issue")
                severity = diag.get("severity", 1)
                severity_map = {1: "ERROR", 2: "WARN", 3: "INFO", 4: "HINT"}
                level = severity_map.get(severity, "UNKNOWN")
                issues.append(f"Line {line}: [{level}] {message}")

            return False, "\n".join(issues)
        elif waited >= max_wait:
            return (
                False,
                f"Timed out waiting for LSP diagnostics. Claude can check in manually with neovim socket: {socket_path}",
            )
        else:
            return True, "No issues found"
    except Exception as e:
        return False, f"Error: {e}"


def should_lint_file(file_path: Path) -> bool:
    """Check if this file should be linted."""
    if not file_path.exists():
        return False

    # Load skip configurations from external files
    skip_dirs = load_config_set("skip_dirs")
    skip_extensions = load_config_set("skip_extensions")

    # Fall back to defaults if config files are empty
    if not skip_dirs:
        skip_dirs = {
            "venv",
            ".venv",
            "__pycache__",
            ".git",
            "node_modules",
            ".build",
            "build",
        }

    if not skip_extensions:
        skip_extensions = {
            ".png",
            ".jpg",
            ".jpeg",
            ".gif",
            ".pdf",
            ".zip",
            ".tar",
            ".gz",
            ".exe",
            ".so",
            ".dylib",
        }

    # Check if any part of the path contains a skip directory
    for part in file_path.parts:
        if part.lower() in skip_dirs:
            return False

    # Check file extension
    if file_path.suffix.lower() in skip_extensions:
        return False

    return True


def main() -> None:
    try:
        # Read JSON input from stdin
        input_data = json.load(sys.stdin)

        # Extract event and tool info
        event = input_data.get("hook_event_name", "")
        tool_name = input_data.get("tool_name", "")

        # Only process file edits
        if event != "PostToolUse" or tool_name not in ["Edit", "Write", "MultiEdit"]:
            sys.exit(0)

        # Get the file path that was edited
        tool_input = input_data.get("tool_input", {})
        file_path_str = tool_input.get("file_path", "")

        if not file_path_str:
            sys.exit(0)

        file_path = Path(file_path_str)

        # Only lint supported files
        if not should_lint_file(file_path):
            sys.exit(0)

        # Find Neovim socket
        socket_path = find_nvim_socket()
        if not socket_path:
            sys.exit(0)

        success, message = check_with_neovim(file_path_str, socket_path)

        if success:
            print_success("Style clean. Continue with your task.")
        else:
            print_error(
                "BLOCKING: Must fix ALL errors above before continuing", message
            )

        sys.exit(2)

    except json.JSONDecodeError:
        # Silently exit on JSON decode errors
        sys.exit(2)
    except Exception as e:
        # Log unexpected errors for debugging
        print_error(
            f"Unexpected error in post_tool_use hook: {type(e).__name__}", str(e)
        )
        sys.exit(2)


if __name__ == "__main__":
    main()
