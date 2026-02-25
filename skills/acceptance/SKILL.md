---
name: acceptance
description: "Validate implementation against acceptance criteria using dual-agent verification. Works on epics (all descendants) or individual tasks. Triggers: 'accept', 'acceptance check', 'verify implementation', 'did it work', 'check my implementation'. Also invoked automatically by /implement after completion."
argument-hint: "[<task-id>|<epic-id>]"
user-invocable: true
allowed-tools:
  - Task
  - TaskGet
  - TaskList
  - TaskUpdate
  - Bash
  - AskUserQuestion
  - Read
  - Glob
  - Grep
---

# Acceptance

Advisory gate — findings are reported, user decides how to proceed.

## Step 1: Resolve Target

- Argument → `TaskGet(taskId)`.
  - **Epic** (`metadata.type == "epic"`) → epic mode (validate all descendants)
  - **Non-epic task** → task mode (validate this task only)
- No argument → `TaskList()` filtered by `metadata.type == "epic"` AND status in `["completed", "in_progress"]`. Pick highest ID → epic mode.
- None found → stop: "No task or epic found. Pass a task ID."

## Step 2: Gather Plan Context

**Epic mode:** From `TaskGet(epicId)`: extract `metadata.design` and `metadata.plan_file`. If plan_file set → `ct plan latest --task-file <plan_file>`. Falls back to `metadata.design`. Neither exists → "No plan found — checking criteria only."

**Task mode:** Walk `metadata.parent_id` to find root epic. Extract `metadata.design` from root for context. Falls back to task's own description.

## Step 3: Gather Criteria

**Epic mode:** Recursively collect all descendants via `metadata.parent_id` chains. No descendants → stop: "Run /prepare first."

Orphaned task detection: flag any task where `status == "pending"` but parent `status == "completed"`. Advisory warning only — never blocks acceptance. See `references/scenarios.md`.

Group criteria by subtree: grouping nodes become section headers, leaves have `## Acceptance Criteria` extracted. Missing section → `⚠ No acceptance criteria defined`. Format:

```
Phase N: <phase subject>
  Task <id>: <leaf subject>
  - [ ] criterion 1
```

Flat epics (no grandchildren) produce flat `Task <id>: ...` output.

**Task mode:** Extract `## Acceptance Criteria` from the target task's description. Missing → use full description as criteria context.

## Step 4: Get Diff

`git diff HEAD` + `git diff --cached` in parallel. Both empty → `git diff HEAD~1..HEAD`. Still empty → verify against HEAD state.

## Step 5: Spawn Verifier and Breaker

Two parallel `Task(subagent_type="general-purpose")` agents:

**Verifier** evaluates each criterion against the diff: PASS/FAIL/PARTIAL/N/A with line-level evidence. All criteria N/A → PASS with note "no applicable criteria found." Adds "Plan Deviations" section noting justified vs problematic divergences. Ends with one-line verdict.

**Breaker** assumes the implementation is subtly wrong. Hunts five angles: implied requirements, edge cases, integration points, technically-met-but-incomplete criteria, missing negatives. Each finding rated HIGH/MEDIUM/LOW with related criterion. No substantive findings → says so honestly rather than inventing issues.

## Step 6: Reconcile and Present

Reconciliation rules (applied before presenting):
1. Verifier FAIL → **FAIL**
2. Verifier PARTIAL → **PARTIAL**
3. Verifier PASS + breaker HIGH findings on PASS/N/A criteria → **PARTIAL** (breaker caught what verifier missed)
4. Verifier PASS + no breaker HIGH findings → **PASS**

Present both reports with clear labels. **PASS** → concise green summary (max 5 lines, no per-criterion breakdown), done.

**PARTIAL/FAIL** → AskUserQuestion with exactly 4 options:
1. **Fix gaps** → spawn fix agent with FAIL/PARTIAL criteria as scope, re-run from Step 4
2. **Commit anyway (override)** → note override in findings, proceed to Step 7
3. **File as follow-up tasks** → TaskCreate per gap (`metadata: {type: "bug", priority: "P2", parent_id: epicId}`), proceed to Step 7
4. **Re-run after manual fixes** → wait for user signal, re-run from Step 4

## Step 7: Store Findings

`ct plan create --prefix "acceptance"`. TaskUpdate target (epic or task) with `acceptance_result: {verdict, criteria_count, verifier_report, breaker_report}` and `plan_file`.
