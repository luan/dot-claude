#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
"""Read all grading results and emit summary."""

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path


def find_grading_files(workspace: Path) -> list[Path]:
    """Find grading JSON files from both eval mode and improve mode locations."""
    files: list[Path] = []

    # Eval mode: workspace/grading/*.json
    grading_dir = workspace / "grading"
    if grading_dir.is_dir():
        files.extend(sorted(grading_dir.glob("*.json")))

    # Improve mode: workspace/vN/runs/run-M/grading.json
    for version_dir in sorted(workspace.glob("v[0-9]*")):
        runs_dir = version_dir / "runs"
        if runs_dir.is_dir():
            files.extend(sorted(runs_dir.glob("run-*/grading.json")))

    return files


def score_fallback(result: dict) -> int:
    """Extract score from a result, with fallback for legacy files without score."""
    if "score" in result:
        return result["score"]
    return 4 if result.get("passed", False) else 1


def main() -> None:
    parser = argparse.ArgumentParser(description="Aggregate grading results and emit summary.")
    parser.add_argument("workspace", type=Path, help="Path to the eval workspace")
    args = parser.parse_args()

    workspace: Path = args.workspace.resolve()
    if not workspace.is_dir():
        print(f"Error: workspace not found: {workspace}", file=sys.stderr)
        sys.exit(1)

    grading_files = find_grading_files(workspace)
    if not grading_files:
        print(f"Error: no grading files found in {workspace}", file=sys.stderr)
        sys.exit(1)

    # Accumulate per-criterion results
    criterion_passes: dict[str, int] = defaultdict(int)
    criterion_totals: dict[str, int] = defaultdict(int)
    criterion_scores: dict[str, list[int]] = defaultdict(list)
    total_pass = 0
    total_count = 0
    cases_pass = 0
    cases_total = 0

    # Per-version score tracking
    version_scores: dict[int, list[float]] = defaultdict(list)

    for path in grading_files:
        try:
            data = json.loads(path.read_text())
        except (json.JSONDecodeError, OSError) as e:
            print(f"Warning: skipping {path}: {e}", file=sys.stderr)
            continue

        if not isinstance(data, dict) or "results" not in data:
            print(f"Warning: skipping {path}: missing 'results' array (expected grading.json schema)", file=sys.stderr)
            continue

        cases_total += 1
        if data.get("overall") == "pass":
            cases_pass += 1

        version = data.get("version")
        file_scores: list[int] = []

        for result in data["results"]:
            name = result.get("criterion", "unknown")
            passed = result.get("passed", False)
            score = score_fallback(result)

            criterion_totals[name] += 1
            criterion_scores[name].append(score)
            file_scores.append(score)
            total_count += 1
            if passed:
                criterion_passes[name] += 1
                total_pass += 1

        if version is not None and file_scores:
            version_scores[version].append(sum(file_scores) / len(file_scores))

    if total_count == 0:
        print("Error: grading files contained no results", file=sys.stderr)
        sys.exit(1)

    name_width = max(len(name) for name in criterion_totals)
    name_width = max(name_width, len("Criterion"))

    # Pass-rate table
    print("## Pass Rate")
    print()
    print(f"{'Criterion':<{name_width}}  Pass  Total  Rate")
    print(f"{'-' * name_width}  ----  -----  ----")

    for name in sorted(criterion_totals):
        passes = criterion_passes[name]
        total = criterion_totals[name]
        rate = passes / total * 100
        print(f"{name:<{name_width}}  {passes:>4}  {total:>5}  {rate:>3.0f}%")

    print(f"{'-' * name_width}  ----  -----  ----")
    overall_rate = total_pass / total_count * 100
    print(f"{'Overall':<{name_width}}  {total_pass:>4}  {total_count:>5}  {overall_rate:>3.0f}%")
    case_rate = cases_pass / cases_total * 100 if cases_total > 0 else 0
    print(f"\nCases passed: {cases_pass}/{cases_total} ({case_rate:.0f}%)")

    # Score table
    print()
    print("## Scores")
    print()
    print(f"{'Criterion':<{name_width}}  Mean  Min  Max")
    print(f"{'-' * name_width}  ----  ---  ---")

    all_scores: list[int] = []
    for name in sorted(criterion_scores):
        scores = criterion_scores[name]
        all_scores.extend(scores)
        mean = sum(scores) / len(scores)
        print(f"{name:<{name_width}}  {mean:>4.1f}  {min(scores):>3}  {max(scores):>3}")

    print(f"{'-' * name_width}  ----  ---  ---")
    overall_mean = sum(all_scores) / len(all_scores)
    print(f"{'Overall':<{name_width}}  {overall_mean:>4.1f}  {min(all_scores):>3}  {max(all_scores):>3}")

    # Per-version summary
    if version_scores:
        print()
        print("## Per-Version Average")
        print()
        for v in sorted(version_scores):
            scores = version_scores[v]
            mean = sum(scores) / len(scores)
            print(f"v{v}: {mean:.1f} ({len(scores)} runs)")

    print(f"\nGrading files: {len(grading_files)}")


if __name__ == "__main__":
    main()
