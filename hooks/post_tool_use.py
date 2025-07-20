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

        # Wait a bit for LSP to process
        time.sleep(0.5)

        # Get diagnostics for the file using pynvim
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
        else:
            return True, "No issues found"
    except Exception as e:
        return False, f"Error: {e}"


def should_lint_file(file_path: Path) -> bool:
    """Check if this file should be linted."""
    if not file_path.exists():
        return False

    # Skip common directories and non-text files
    skip_dirs = {
        "venv",
        ".venv",
        "__pycache__",
        ".git",
        "node_modules",
        ".build",
        "build",
    }
    for part in file_path.parts:
        if part in skip_dirs:
            return False

    # Skip binary files and common non-lintable extensions
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
            print("", file=sys.stderr)
            print(
                "\033[32m👉 Style clean. Continue with your task.\033[0m",
                file=sys.stderr,
            )
        else:
            print("", file=sys.stderr)
            print(
                "\033[31m⛔ BLOCKING: Must fix ALL errors above before continuing\033[0m",
                file=sys.stderr,
            )
            print(f"{message}", file=sys.stderr)

        sys.exit(2)

    except json.JSONDecodeError:
        sys.exit(2)
    except Exception:
        sys.exit(2)


if __name__ == "__main__":
    main()
