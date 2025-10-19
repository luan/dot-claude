#!/usr/bin/env uv run
"""
Standalone statusline module inspired by cc-sessions design.
Provides beautiful visual progress bars and status information.
"""

import sys
import os
from enum import Enum
from typing import Optional, Dict, Any


class IconStyle(Enum):
    NERD_FONTS = "nerd_fonts"
    EMOJI = "emoji"
    ASCII = "ascii"


class Statusline:
    """Beautiful statusline with progress bars and visual indicators."""

    def __init__(self, icon_style: str = "nerd_fonts", enable_colors: bool = True):
        """
        Initialize statusline.

        Args:
            icon_style: "nerd_fonts", "emoji", or "ascii"
            enable_colors: Force disable colors (useful for non-ANSI terminals)
        """
        self.icon_style = IconStyle(icon_style)
        self.enable_colors = enable_colors and self._supports_ansi()
        self._setup_colors()
        self._setup_icons()

    def _supports_ansi(self) -> bool:
        """Check if the current environment supports ANSI color codes."""

        # Windows detection
        if sys.platform == "win32":
            wt_session = os.environ.get("WT_SESSION")
            pwsh_version = os.environ.get("POWERSHELL_DISTRIBUTION_CHANNEL")

            if wt_session:
                return True

            if pwsh_version and "PSCore" in pwsh_version:
                return True

            try:
                import platform

                win_ver = platform.version()
                if int(win_ver.split(".")[2]) >= 14393:
                    import ctypes

                    kernel32 = ctypes.windll.kernel32
                    stdout_handle = kernel32.GetStdHandle(-11)
                    mode = ctypes.c_ulong()
                    kernel32.GetConsoleMode(stdout_handle, ctypes.byref(mode))
                    mode.value |= 0x0004
                    kernel32.SetConsoleMode(stdout_handle, mode.value)
                    return True
            except:
                pass
            return False

        # Unix-like systems support ANSI
        return True

    def _setup_colors(self):
        """Setup Ayu Dark color scheme."""
        if self.enable_colors:
            self.colors = {
                "green": "\033[38;5;114m",
                "orange": "\033[38;5;215m",
                "red": "\033[38;5;203m",
                "gray": "\033[38;5;242m",
                "l_gray": "\033[38;5;250m",
                "cyan": "\033[38;5;111m",
                "purple": "\033[38;5;183m",
                "reset": "\033[0m",
            }
        else:
            # No color support
            self.colors = {
                key: ""
                for key in [
                    "green",
                    "orange",
                    "red",
                    "gray",
                    "l_gray",
                    "cyan",
                    "purple",
                    "reset",
                ]
            }

    def _setup_icons(self):
        """Setup icons based on selected style."""
        if self.icon_style == IconStyle.NERD_FONTS:
            self.icons = {
                "context": "󱃖 ",
                "task": "󰒓 ",
                "mode_implement": "󰷫 ",
                "mode_discuss": "󰭹 ",
                "branch": "󰘬 ",
                "tasks": "󰈙 ",
                "edit": "✎ ",
            }
        elif self.icon_style == IconStyle.EMOJI:
            self.icons = {
                "context": "",
                "task": "⚙️ ",
                "mode_implement": "🛠️ ",
                "mode_discuss": "💬 ",
                "branch": "Branch: ",
                "tasks": "💼 ",
                "edit": "✎ ",
            }
        else:  # ASCII
            self.icons = {
                "context": "",
                "task": "Task: ",
                "mode_implement": "Mode:",
                "mode_discuss": "Mode:",
                "branch": "Branch: ",
                "tasks": "",
                "edit": "✎ ",
            }

    def _format_tokens(self, count: Optional[int]) -> str:
        """Format token count as 'k' notation."""
        if count is None:
            return "0k"
        return f"{count // 1000}k" if count >= 1000 else str(count)

    def _create_progress_bar(self, current: int, limit: int) -> str:
        """
        Create a visual progress bar with block characters.

        Args:
            current: Current usage/progress
            limit: Maximum limit

        Returns:
            Formatted progress bar string
        """
        if limit <= 0:
            return f"{self.colors['gray']}No progress data{self.colors['reset']}"

        # Calculate percentage
        pct = min((current * 100) / limit, 100)
        progress_pct = f"{pct:.1f}"
        progress_int = int(pct)

        # Progress bar blocks (0-10)
        filled_blocks = min(progress_int // 10, 10)
        empty_blocks = 10 - filled_blocks

        # Choose color based on percentage
        if progress_int < 50:
            bar_color = self.colors["green"]
        elif progress_int < 80:
            bar_color = self.colors["orange"]
        else:
            bar_color = self.colors["red"]

        # Format token counts
        formatted_current = self._format_tokens(current)
        formatted_limit = self._format_tokens(limit)

        # Build progress bar
        progress_parts = []
        progress_parts.append(f"{bar_color}{self.icons['context']}")
        progress_parts.append(bar_color + ("█" * filled_blocks))
        progress_parts.append(self.colors["gray"] + ("░" * empty_blocks))
        progress_parts.append(
            f"{self.colors['reset']} {self.colors['l_gray']}{progress_pct}% ({formatted_current}/{formatted_limit}){self.colors['reset']}"
        )

        return "".join(progress_parts)

    def _format_model(self, model) -> str:
        """Format model information from various input formats."""
        if model is None:
            return None

        if isinstance(model, dict):
            # Handle model dict format
            return model.get("display_name", model.get("id", str(model)))
        elif isinstance(model, str):
            # Handle string format
            return model
        else:
            # Fallback for other types
            return str(model)

    def render(
        self,
        context_usage: Optional[int] = None,
        context_limit: int = 160000,
        model: Optional[Any] = None,
        custom_status: Optional[str] = None,
        git_branch: Optional[str] = None,
        edited_files: int = 0,
        custom_info: Optional[Dict[str, str]] = None,
    ) -> str:
        """
        Render the complete statusline.

        Args:
            context_usage: Current context usage in tokens
            context_limit: Maximum context limit
            model: Model information (string, dict, or other)
            custom_status: Custom status message to display
            git_branch: Git branch name
            edited_files: Number of edited/uncommitted files
            custom_info: Additional custom information to display

        Returns:
            Formatted statusline string
        """
        # Line 1: Progress bar | Status (with optional model)
        progress_bar = self._create_progress_bar(context_usage or 0, context_limit)

        # Format model info
        formatted_model = self._format_model(model)

        # Build line 1: Progress bar | Model (or custom status)
        if custom_status:
            status_part = f"{self.colors['cyan']}{custom_status}{self.colors['reset']}"
            if formatted_model:
                model_part = (
                    f"{self.colors['l_gray']}({formatted_model}){self.colors['reset']}"
                )
                line1 = f"{progress_bar} | {status_part} {model_part}"
            else:
                line1 = f"{progress_bar} | {status_part}"
        elif formatted_model:
            # Show model as primary status when no custom status
            line1 = f"{progress_bar} | {self.colors['purple']}{formatted_model}{self.colors['reset']}"
        else:
            line1 = progress_bar

        # Build line 2: Edited files | Git branch | Custom info
        line2_parts = []

        if edited_files > 0:
            edited_part = f"{self.colors['orange']}{self.icons['edit']} {edited_files}{self.colors['reset']}"
            line2_parts.append(edited_part)

        if git_branch:
            branch_part = f"{self.colors['l_gray']}{self.icons['branch']}{git_branch}{self.colors['reset']}"
            line2_parts.append(branch_part)

        # Add custom info if provided
        if custom_info:
            for key, value in custom_info.items():
                if value is None:
                    # Directory names (without prefix)
                    line2_parts.append(
                        f"{self.colors['cyan']}{key}{self.colors['reset']}"
                    )
                else:
                    # Regular key: value pairs
                    line2_parts.append(
                        f"{self.colors['l_gray']}{key}: {self.colors['cyan']}{value}{self.colors['reset']}"
                    )

        # Return appropriate number of lines
        if line2_parts:
            line2 = " | ".join(line2_parts)
            return f"{line1}\n{line2}"
        else:
            return line1

    def _format_duration(self, duration_ms: Optional[int]) -> str:
        """Format duration in milliseconds to readable format."""
        if duration_ms is None:
            return None

        seconds = duration_ms // 1000
        if seconds < 60:
            return f"{seconds}s"
        elif seconds < 3600:
            minutes = seconds // 60
            return f"{minutes}m"
        else:
            hours = seconds // 3600
            minutes = (seconds % 3600) // 60
            return f"{hours}h{minutes}m"

    def _format_cost(self, cost_usd: Optional[float]) -> str:
        """Format cost in USD to readable format."""
        if cost_usd is None:
            return None

        if cost_usd < 0.01:
            return f"${cost_usd * 1000:.1f}m"
        else:
            return f"${cost_usd:.3f}"

    def _format_path(self, path: Optional[str], max_length: int = 30) -> str:
        """Format and truncate path for display."""
        if path is None:
            return None

        # Replace home directory with ~
        home = os.path.expanduser("~")
        if path.startswith(home):
            path = "~" + path[len(home) :]

        # Truncate if too long
        if len(path) > max_length:
            # Keep the beginning and end, truncate middle
            half = max_length // 2
            path = path[:half] + "..." + path[-half:]

        return path

    def _extract_context_usage(
        self, transcript_path: Optional[str], session_id: Optional[str]
    ) -> Optional[int]:
        """Extract context usage from transcript file like the original cc-sessions implementation."""
        if not transcript_path or not os.path.exists(transcript_path):
            return None

        try:
            import json as json_module
            from datetime import datetime, timezone

            with open(
                transcript_path, "r", encoding="utf-8", errors="backslashreplace"
            ) as f:
                lines = f.readlines()

            most_recent_usage = None
            most_recent_timestamp = None

            for line in lines:
                try:
                    data = json_module.loads(line.strip())
                    # Skip sidechain entries (subagent calls)
                    if data.get("isSidechain", False):
                        continue

                    # Check for usage data in main-chain messages
                    if data.get("message", {}).get("usage"):
                        timestamp = data.get("timestamp")
                        if timestamp and (
                            not most_recent_timestamp
                            or timestamp > most_recent_timestamp
                        ):
                            most_recent_timestamp = timestamp
                            most_recent_usage = data["message"]["usage"]
                except:
                    continue

            # Calculate context length (input + cache tokens only, NOT output)
            if most_recent_usage:
                context_length = (
                    most_recent_usage.get("input_tokens", 0)
                    + most_recent_usage.get("cache_read_input_tokens", 0)
                    + most_recent_usage.get("cache_creation_input_tokens", 0)
                )
                # Set minimum to 17000 like original
                return max(context_length, 17000)
        except:
            pass

        return None

    def render_from_claude_data(self, data: Dict[str, Any]) -> str:
        """
        Render statusline from Claude Code's JSON data structure.

        Args:
            data: The JSON data passed from Claude Code

        Returns:
            Formatted statusline string
        """
        # Extract basic information
        model = data.get("model")
        workspace = data.get("workspace", {})
        cwd = data.get("cwd", workspace.get("current_dir"))
        project_dir = workspace.get("project_dir")
        session_id = data.get("session_id")
        transcript_path = data.get("transcript_path")
        version = data.get("version")

        # Extract cost and metrics
        cost_data = data.get("cost", {})
        total_cost = cost_data.get("total_cost_usd")
        duration = cost_data.get("total_duration_ms")
        lines_added = cost_data.get("lines_added", 0)
        lines_removed = cost_data.get("lines_removed", 0)

        # Extract context usage from transcript
        context_usage = self._extract_context_usage(transcript_path, session_id)

        # Determine context limit based on model
        context_limit = 160000
        if model:
            model_id = model.get("id", "").lower()
            if "[1m]" in model_id:
                context_limit = 800000
            elif "opus" in model_id:
                context_limit = 200000

        # Format directory info
        formatted_cwd = self._format_path(cwd)
        formatted_project = self._format_path(project_dir)

        # Build custom info
        custom_info = {}

        # Add line changes if any
        if lines_added > 0 or lines_removed > 0:
            lines_text = []
            if lines_added > 0:
                lines_text.append(f"+{lines_added}")
            if lines_removed > 0:
                lines_text.append(f"-{lines_removed}")
            custom_info["Lines"] = "".join(lines_text)

        # Add directory info (without prefix)
        if formatted_project and formatted_cwd != formatted_project:
            custom_info[formatted_cwd] = None
        elif formatted_cwd:
            # Show just the current directory name if it's the project
            custom_info[os.path.basename(formatted_cwd) or formatted_cwd] = None

        # Get git branch
        git_branch = None
        try:
            if cwd:
                import subprocess

                branch_cmd = ["git", "-C", cwd, "branch", "--show-current"]
                branch = subprocess.check_output(
                    branch_cmd,
                    stderr=subprocess.PIPE,
                    encoding="utf-8",
                    errors="replace",
                ).strip()
                if branch:
                    git_branch = branch
        except:
            pass

        # Get edited files count
        edited_files = 0
        try:
            if cwd:
                import subprocess

                # Count unstaged changes
                unstaged_cmd = ["git", "-C", cwd, "diff", "--name-only"]
                unstaged_files = (
                    subprocess.check_output(
                        unstaged_cmd,
                        stderr=subprocess.PIPE,
                        encoding="utf-8",
                        errors="replace",
                    )
                    .strip()
                    .split("\n")
                )
                unstaged_count = len([f for f in unstaged_files if f])

                # Count staged changes
                staged_cmd = ["git", "-C", cwd, "diff", "--cached", "--name-only"]
                staged_files = (
                    subprocess.check_output(
                        staged_cmd,
                        stderr=subprocess.PIPE,
                        encoding="utf-8",
                        errors="replace",
                    )
                    .strip()
                    .split("\n")
                )
                staged_count = len([f for f in staged_files if f])

                edited_files = unstaged_count + staged_count
        except:
            pass

        return self.render(
            context_usage=context_usage,
            context_limit=context_limit,
            model=model,
            custom_status=None,
            git_branch=git_branch,
            edited_files=edited_files,
            custom_info=custom_info,
        )


# Simple CLI usage
if __name__ == "__main__":
    import json

    # Read JSON input from stdin if available
    try:
        if not sys.stdin.isatty():
            data = json.load(sys.stdin)
            # Use Claude Code data
            statusline = Statusline(icon_style="nerd_fonts")
            print(statusline.render_from_claude_data(data))
        else:
            # Default values for testing
            context_usage = 45000
            context_limit = 160000
            model = {"id": "claude-sonnet-4-5-20250929", "display_name": "Sonnet 4.5"}
            custom_status = "Working on feature"
            git_branch = "main"
            edited_files = 3
            custom_info = {"Lines": "+5/-2", "~/.claude": None}

            statusline = Statusline(icon_style="nerd_fonts")
            print(
                statusline.render(
                    context_usage=context_usage,
                    context_limit=context_limit,
                    model=model,
                    custom_status=custom_status,
                    git_branch=git_branch,
                    edited_files=edited_files,
                    custom_info=custom_info,
                )
            )
    except Exception as e:
        # Fallback on error
        statusline = Statusline(icon_style="nerd_fonts")
        print(
            statusline.render(
                context_usage=45000,
                context_limit=160000,
                model={
                    "id": "claude-sonnet-4-5-20250929",
                    "display_name": "Sonnet 4.5",
                },
                custom_status="Statusline Error",
            )
        )

