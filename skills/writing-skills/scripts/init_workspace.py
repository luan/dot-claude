#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
"""Initialize a new eval workspace for a skill under evaluation."""

import argparse
import json
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Initialize a new eval workspace for a skill under evaluation."
    )
    parser.add_argument("skill_path", type=Path, help="Path to the skill directory to evaluate")
    parser.add_argument(
        "--workspace",
        type=Path,
        help="Workspace directory (default: <skill-name>-workspace as sibling to skill dir)",
    )
    args = parser.parse_args()

    skill_path: Path = args.skill_path.resolve()
    if not skill_path.is_dir():
        print(f"Error: skill path is not a directory: {skill_path}", file=sys.stderr)
        sys.exit(1)

    workspace: Path = (
        args.workspace.resolve()
        if args.workspace
        else skill_path.parent / f"{skill_path.name}-workspace"
    )

    # Ensure skill has evals/ directory with evals.json
    evals_dir = skill_path / "evals"
    evals_path = evals_dir / "evals.json"
    if not evals_path.exists():
        evals_dir.mkdir(parents=True, exist_ok=True)
        evals = {"version": "1.0", "skill": str(skill_path), "cases": []}
        evals_path.write_text(json.dumps(evals, indent=2) + "\n")
        print(f"Created {evals_path}")
    else:
        print(f"Evals found: {evals_path}")

    # Create workspace directory structure (runtime artifacts only)
    v0_skill = workspace / "v0" / "skill"
    grading_dir = workspace / "grading"
    v0_skill.mkdir(parents=True, exist_ok=True)
    grading_dir.mkdir(parents=True, exist_ok=True)

    # Copy skill into v0/skill/ (only if empty — idempotent)
    if not any(v0_skill.iterdir()):
        shutil.copytree(skill_path, v0_skill, dirs_exist_ok=True)
        print(f"Copied skill into {v0_skill}")
    else:
        print("v0/skill/ already populated, skipping copy")

    # Write empty history.json (won't overwrite)
    history_path = workspace / "history.json"
    if not history_path.exists():
        history = {
            "versions": [
                {
                    "version": 0,
                    "timestamp": datetime.now(timezone.utc).isoformat(),
                    "description": "baseline",
                    "git_hash": None,
                    "path": "v0/skill",
                    "average_score": None,
                }
            ],
            "current_version": 0,
        }
        history_path.write_text(json.dumps(history, indent=2) + "\n")
        print(f"Created {history_path}")
    else:
        print("history.json already exists, skipping")

    print(f"\nWorkspace ready: {workspace}")
    print(f"Evals: {evals_path}")


if __name__ == "__main__":
    main()
