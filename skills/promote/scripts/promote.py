#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Promote a skill/rule/agent from $HOME/.claude into a shared plugin repo.

Usage:
  uv run promote.py <source-path> <plugin-dir>

Examples:
  uv run promote.py $HOME/.claude/skills/my-skill $HOME/AI/dot-claude-plugin
  uv run promote.py $HOME/.claude/rules/my-rule.md $HOME/AI/dot-claude-work
"""

import shutil
import subprocess
import sys
from pathlib import Path

CONTENT_TYPES = ("skills", "rules", "agents", "hooks", "tools")


def infer_content_type(source: Path) -> str:
    # Walk up to find a known content type dir
    for part in source.parts:
        if part in CONTENT_TYPES:
            return part
    # Fallback: detect by content
    if source.is_dir() and (source / "SKILL.md").exists():
        return "skills"
    if source.suffix == ".md":
        return "rules"
    return "skills"


def main() -> None:
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(1)

    source = Path(sys.argv[1]).expanduser().resolve()
    plugin_dir = Path(sys.argv[2]).expanduser().resolve()

    if not source.exists():
        print(f"Error: source not found: {source}")
        sys.exit(1)

    if not (plugin_dir / ".git").exists():
        print(f"Error: {plugin_dir} is not a git repo")
        sys.exit(1)

    content_type = infer_content_type(source)
    dest_parent = plugin_dir / content_type
    dest_parent.mkdir(parents=True, exist_ok=True)
    dest = dest_parent / source.name

    if dest.exists():
        print(f"Error: destination already exists: {dest}")
        sys.exit(1)

    shutil.move(str(source), str(dest))
    subprocess.run(["git", "add", str(dest)], cwd=plugin_dir, check=True)

    print(f"Promoted: {source.name}")
    print(f"  from: {source.parent}")
    print(f"  to:   {dest}")
    print(f"\nStaged in {plugin_dir.name}. Run /commit when ready.")


if __name__ == "__main__":
    main()
