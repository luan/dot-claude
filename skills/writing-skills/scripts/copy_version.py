#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
"""Snapshot current skill into a new versioned directory."""

import argparse
import json
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


def get_git_hash() -> str | None:
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            timeout=5,
        )
        if result.returncode == 0:
            return result.stdout.strip()
    except (FileNotFoundError, subprocess.TimeoutExpired):
        pass
    return None


def compute_average_score(workspace: Path, version: int) -> float | None:
    """Compute mean score across all grading files for a given version."""
    grading_dir = workspace / "grading"
    if not grading_dir.is_dir():
        return None

    scores: list[int] = []
    for path in grading_dir.glob(f"*_v{version}_run*.json"):
        try:
            data = json.loads(path.read_text())
        except (json.JSONDecodeError, OSError):
            continue
        for result in data.get("results", []):
            if "score" in result:
                scores.append(result["score"])
            elif result.get("passed", False):
                scores.append(4)
            else:
                scores.append(1)

    if not scores:
        return None
    return round(sum(scores) / len(scores), 1)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Snapshot current skill into a new versioned directory."
    )
    parser.add_argument("workspace", type=Path, help="Path to the eval workspace")
    parser.add_argument("--description", default="", help="Description for this version")
    args = parser.parse_args()

    workspace: Path = args.workspace.resolve()
    history_path = workspace / "history.json"

    if not history_path.exists():
        print(f"Error: no history.json found in {workspace}", file=sys.stderr)
        sys.exit(1)

    history = json.loads(history_path.read_text())
    current_version = history["current_version"]
    next_version = current_version + 1

    # Find the current skill to copy
    current_skill = workspace / f"v{current_version}" / "skill"
    if not current_skill.is_dir():
        print(f"Error: current skill not found at {current_skill}", file=sys.stderr)
        sys.exit(1)

    # Create new version directory and copy skill
    new_version_dir = workspace / f"v{next_version}"
    new_skill_dir = new_version_dir / "skill"
    if new_version_dir.exists():
        print(f"Error: v{next_version}/ already exists", file=sys.stderr)
        sys.exit(1)

    new_skill_dir.mkdir(parents=True)
    shutil.copytree(current_skill, new_skill_dir, dirs_exist_ok=True)

    # Compute average score from grading files for current version
    average_score = compute_average_score(workspace, current_version)

    # Record version metadata
    version_entry = {
        "version": next_version,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "description": args.description or f"version {next_version}",
        "git_hash": get_git_hash(),
        "path": f"v{next_version}/skill",
        "average_score": average_score,
    }

    history["versions"].append(version_entry)
    history["current_version"] = next_version
    history_path.write_text(json.dumps(history, indent=2) + "\n")

    print(f"Created v{next_version} from v{current_version}")
    print(f"  Path: {new_skill_dir}")
    if args.description:
        print(f"  Description: {args.description}")
    if average_score is not None:
        print(f"  Average score: {average_score}")


if __name__ == "__main__":
    main()
