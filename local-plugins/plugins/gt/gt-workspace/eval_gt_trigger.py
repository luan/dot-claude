#!/usr/bin/env python3
"""Eval whether gt skills trigger in a Graphite repo context.

Runs claude -p with each query, checks if Claude calls Skill(gt:*)
or reads the gt skill. Reports pass/fail per query.
"""

import json
import os
import subprocess
import sys
import time
from concurrent.futures import ProcessPoolExecutor, as_completed


QUERIES = [
    {"query": "ok i'm done with the sync engine changes, push everything up and make sure the PR descriptions are current", "should_trigger": True},
    {"query": "i need to split this into two PRs — the proto regen should be separate from the sync logic changes. can you move the proto stuff to its own branch underneath?", "should_trigger": True},
    {"query": "my stack is out of date, main moved forward a bunch since yesterday. sync it up and fix any conflicts", "should_trigger": True},
    {"query": "create a new branch on top of luan/sync-engine-improvements for the test infrastructure changes", "should_trigger": True},
    {"query": "push this but don't create new PRs for the WIP branches, just update the ones that already exist", "should_trigger": True},
    {"query": "check the current stack, i want to see what branches i have and their status", "should_trigger": True},
    {"query": "the changes to SyncEngine.swift should actually be in the parent branch, not this one. can you move them down?", "should_trigger": True},
    {"query": "rebase my branches onto the latest main, there were some conflicts last time so watch out for the proto files", "should_trigger": True},
    {"query": "ship it — push all my branches and create PRs for any that don't have one yet", "should_trigger": True},
    {"query": "commit these changes with a message about fixing the duplication_index convergence test", "should_trigger": False},
    {"query": "the test in SyncEngineTests.swift is failing on line 234, can you take a look?", "should_trigger": False},
    {"query": "review my changes before i submit — i want to make sure the sync batching logic is correct", "should_trigger": False},
    {"query": "stage just the changes to SyncEngine.swift but not the test file, i want to commit them separately", "should_trigger": False},
    {"query": "show me the git log for the last 10 commits on main, i want to see what landed while i was out", "should_trigger": False},
    {"query": "what does the remoteOriginSyncIDs property do? i see it referenced in the convergence test but i'm not sure how it works", "should_trigger": False},
    {"query": "reorganize the commits on this branch — the first commit has both the refactor and the feature mixed together, split them clean", "should_trigger": False},
    {"query": "discard the changes to Package.swift, i don't want those in this branch. keep everything else staged", "should_trigger": False},
]


def run_query(query: str, cwd: str, model: str, timeout: int = 45) -> dict:
    """Run a single query and detect if any gt skill was triggered."""
    env = {k: v for k, v in os.environ.items() if k != "CLAUDECODE"}
    cmd = [
        "claude", "-p", query,
        "--output-format", "stream-json",
        "--verbose",
        "--include-partial-messages",
        "--model", model,
    ]

    process = subprocess.Popen(
        cmd, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, cwd=cwd, env=env,
    )

    gt_triggered = False
    first_tool = None
    accumulated_json = ""
    start = time.time()

    try:
        while time.time() - start < timeout:
            if process.poll() is not None:
                remaining = process.stdout.read()
                if remaining:
                    for line in remaining.decode("utf-8", errors="replace").splitlines():
                        gt_triggered, first_tool, accumulated_json = _check_line(line, gt_triggered, first_tool, accumulated_json)
                break

            line = process.stdout.readline()
            if not line:
                break
            gt_triggered, first_tool, accumulated_json = _check_line(
                line.decode("utf-8", errors="replace").strip(), gt_triggered, first_tool, accumulated_json
            )
            # Early exit: only if we confirmed gt triggered, or first tool is
            # resolved to something non-gt (Skill needs more data to determine which skill)
            if gt_triggered:
                break
            if first_tool and first_tool != "Skill" and "gt" not in first_tool:
                break
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()

    return {"gt_triggered": gt_triggered, "first_tool": first_tool}


def _check_line(line: str, gt_triggered: bool, first_tool: str | None, accumulated_json: str = "") -> tuple[bool, str | None, str]:
    if not line:
        return gt_triggered, first_tool, accumulated_json
    try:
        event = json.loads(line)
    except json.JSONDecodeError:
        return gt_triggered, first_tool, accumulated_json

    # Check stream events for tool_use starts
    if event.get("type") == "stream_event":
        se = event.get("event", {})
        if se.get("type") == "content_block_start":
            cb = se.get("content_block", {})
            if cb.get("type") == "tool_use":
                tool_name = cb.get("name", "")
                if first_tool is None:
                    first_tool = tool_name
                if tool_name == "Skill":
                    accumulated_json = ""

        elif se.get("type") == "content_block_delta":
            delta = se.get("delta", {})
            if delta.get("type") == "input_json_delta":
                partial = delta.get("partial_json", "")
                accumulated_json += partial
                if "gt:" in accumulated_json or "gt:gt" in accumulated_json or "gt:submit" in accumulated_json or "gt:restack" in accumulated_json:
                    gt_triggered = True

        elif se.get("type") == "content_block_stop":
            # Try to parse accumulated JSON to get skill name for reporting
            if first_tool == "Skill" and accumulated_json:
                try:
                    parsed = json.loads(accumulated_json)
                    skill_name = parsed.get("skill", "")
                    if skill_name:
                        first_tool = f"Skill({skill_name})"
                        if "gt" in skill_name:
                            gt_triggered = True
                except json.JSONDecodeError:
                    pass

    # Check full assistant messages
    elif event.get("type") == "assistant":
        for item in event.get("message", {}).get("content", []):
            if item.get("type") != "tool_use":
                continue
            tool_name = item.get("name", "")
            tool_input = item.get("input", {})
            if tool_name == "Skill":
                skill_val = tool_input.get("skill", "")
                resolved = f"Skill({skill_val})"
                if first_tool is None or first_tool == "Skill":
                    first_tool = resolved
                if "gt" in skill_val:
                    gt_triggered = True
            elif first_tool is None:
                first_tool = tool_name

    return gt_triggered, first_tool, accumulated_json


def main():
    model = sys.argv[1] if len(sys.argv) > 1 else "claude-opus-4-6"
    cwd = os.path.dirname(os.path.abspath(__file__))
    workers = 8

    print(f"Running {len(QUERIES)} queries (model={model}, cwd={cwd})")
    print()

    results = []
    with ProcessPoolExecutor(max_workers=workers) as executor:
        futures = {}
        for q in QUERIES:
            f = executor.submit(run_query, q["query"], cwd, model)
            futures[f] = q

        for f in as_completed(futures):
            q = futures[f]
            try:
                r = f.result()
            except Exception as e:
                r = {"gt_triggered": False, "first_tool": f"ERROR: {e}"}

            triggered = r["gt_triggered"]
            expected = q["should_trigger"]
            passed = triggered == expected
            results.append({
                "query": q["query"],
                "should_trigger": expected,
                "gt_triggered": triggered,
                "first_tool": r["first_tool"],
                "passed": passed,
            })

    # Sort: should-trigger first, then should-not
    results.sort(key=lambda r: (not r["should_trigger"], r["query"]))

    # Print results
    tp = fp = tn = fn = 0
    for r in results:
        status = "PASS" if r["passed"] else "FAIL"
        arrow = "→" if r["gt_triggered"] else "✗"
        tool = r["first_tool"] or "none"
        print(f"  [{status}] {arrow} gt={r['gt_triggered']:5}  expected={r['should_trigger']:5}  first_tool={tool:12}  {r['query'][:70]}")

        if r["should_trigger"] and r["gt_triggered"]:
            tp += 1
        elif r["should_trigger"] and not r["gt_triggered"]:
            fn += 1
        elif not r["should_trigger"] and r["gt_triggered"]:
            fp += 1
        else:
            tn += 1

    total = len(results)
    passed = sum(1 for r in results if r["passed"])
    precision = tp / (tp + fp) if (tp + fp) > 0 else 0
    recall = tp / (tp + fn) if (tp + fn) > 0 else 0

    print()
    print(f"Results: {passed}/{total} passed")
    print(f"  Precision: {precision:.0%}  Recall: {recall:.0%}")
    print(f"  TP={tp} FP={fp} TN={tn} FN={fn}")


if __name__ == "__main__":
    main()
