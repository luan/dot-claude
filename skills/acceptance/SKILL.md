---
name: acceptance
description: "Validate implementation against epic acceptance criteria using dual-agent verification. Triggers: 'accept', 'acceptance check', 'verify implementation', 'did it work', 'check my implementation'. Also invoked automatically by /implement after completion."
argument-hint: "[<task-id>]"
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

## Step 1: Resolve Epic

- Argument → `TaskGet(taskId)`. Not epic? Use `metadata.parent_id`.
- No argument → `TaskList()` filtered by `metadata.type == "epic"` AND status in `["completed", "in_progress"]`. Pick highest ID.
- None found → stop: "No epic found. Pass a task ID or ensure an epic exists."

## Step 2: Gather Plan Context

From `TaskGet(epicId)`: extract `metadata.design` and `metadata.plan_file`.

If plan_file set → `ck plan latest --task-file <plan_file>`. Falls back to `metadata.design`. Neither exists → "No plan found — checking criteria only."

## Step 3: Gather Descendant Criteria

Recursively collect all descendants via `metadata.parent_id` chains. No descendants → stop: "Run /prepare first."

**Orphaned task detection:** While traversing, flag any task where `status == "pending"` but parent `status == "completed"`. Prepend advisory warning — never blocks acceptance. See `references/scenarios.md` for test scenarios.

**Group criteria by subtree:** Grouping nodes (have children) become section headers. Leaves have their `## Acceptance Criteria` extracted and nested under nearest grouping ancestor. Format:

```
Phase N: <phase subject>
  Task <id>: <leaf subject>
  - [ ] criterion 1
```

Flat epics (no grandchildren) produce flat `Task <id>: ...` output.

## Step 4: Get Diff

`git diff HEAD` + `git diff --cached` in parallel. Both empty → `git diff HEAD~1..HEAD`. Still empty → verify against HEAD state (acceptance still runs — diff-less changes like config or dependency updates are valid).

## Step 5: Spawn Verifier and Breaker

Two parallel `Task(subagent_type="general-purpose")` agents — a dual-lens approach because a single reviewer misses adversarial gaps:

**Verifier** evaluates each criterion against the diff: PASS/FAIL/PARTIAL/N/A with line-level evidence. All criteria N/A → PASS with note "no applicable criteria found." Adds "Plan Deviations" section noting justified vs problematic divergences. Ends with one-line verdict.

**Breaker** assumes the implementation is subtly wrong. Hunts five angles: implied requirements, edge cases, integration points, technically-met-but-incomplete criteria, missing negatives. Each finding rated HIGH/MEDIUM/LOW with related criterion. No substantive findings → says so honestly rather than inventing issues.

## Step 6: Reconcile and Present

Reconciliation rules (applied before presenting):
1. Verifier FAIL → **FAIL**
2. Verifier PARTIAL → **PARTIAL**
3. Verifier PASS + breaker HIGH findings on PASS/N/A criteria → **PARTIAL** (breaker caught what verifier missed)
4. Verifier PASS + no breaker HIGH findings → **PASS**

Present both reports with clear labels. **PASS** → green summary, done.

**PARTIAL/FAIL** → AskUserQuestion: "Acceptance check found gaps."
- **Fix gaps** → spawn fix agent with FAIL/PARTIAL criteria as scope, re-run from Step 4
- **Commit anyway (override)** → note override in findings, proceed to Step 7
- **File as follow-up tasks** → TaskCreate per gap (`metadata: {type: "bug", priority: "P2", parent_id: epicId}`), proceed to Step 7
- **Re-run after manual fixes** → wait for user signal, re-run from Step 4

## Step 7: Store Findings

Persist via `ck plan create --prefix "acceptance"`. Store result and plan file path in epic metadata.
