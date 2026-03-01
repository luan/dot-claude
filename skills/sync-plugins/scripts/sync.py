#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Sync git-backed local Claude plugins via git pull --ff-only.

Usage:
  uv run sync.py              # sync all plugins
  uv run sync.py <name>       # sync one plugin by name
"""

import subprocess
import sys
from pathlib import Path


def find_plugins(plugins_dir: Path) -> list[tuple[str, Path]]:
    if not plugins_dir.exists():
        return []
    result = []
    for entry in sorted(plugins_dir.iterdir()):
        resolved = entry.resolve()
        if (resolved / ".git").exists():
            result.append((entry.name, resolved))
    return result


def sync_plugin(name: str, path: Path) -> None:
    print(f"  {name} ({path})")
    result = subprocess.run(
        ["git", "pull", "--ff-only"],
        cwd=path,
        capture_output=True,
        text=True,
    )
    if result.returncode == 0:
        msg = result.stdout.strip() or result.stderr.strip()
        print(f"    ✓ {msg}")
    else:
        err = (result.stderr or result.stdout).strip()
        print(f"    ✗ FAILED: {err}")


def main() -> None:
    plugins_dir = Path.home() / ".claude" / "local-plugins" / "plugins"
    plugins = find_plugins(plugins_dir)

    if not plugins:
        print("No git-backed plugins found.")
        return

    filter_name = sys.argv[1] if len(sys.argv) > 1 else None

    if filter_name:
        plugins = [(n, p) for n, p in plugins if n == filter_name]
        if not plugins:
            print(f"Plugin '{filter_name}' not found or not git-backed.")
            sys.exit(1)

    print(f"Syncing {len(plugins)} plugin(s)...\n")
    for name, path in plugins:
        sync_plugin(name, path)


if __name__ == "__main__":
    main()
