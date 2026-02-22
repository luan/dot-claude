---
name: implement-worker
description: "Internal skill. Implements a single task: claims it, runs TDD + build via sub-worker, then runs refine on changed files. Invoked by implement and other orchestrators."
argument-hint: "<task-id>"
user-invocable: false
allowed-tools:
  - Task
  - Skill
  - TaskGet
  - TaskUpdate
  - TaskCreate
  - Read
  - Bash
---

# Implement Worker

Single-task mini-orchestrator. Receives a task ID, spawns a code-only sub-worker, then runs refine on the result.

## Step 1: Load Task

`TaskGet(<task-id>)` — capture description and `metadata.parent_id`.

If `metadata.parent_id` exists → walk ancestor chain to epic root:
1. Repeatedly `TaskGet(parent_id)` until no `parent_id`
2. Build breadcrumb root-first, excluding current task: `"Epic > Phase > ..."`
3. Source `metadata.design` from root epic

Flat case (direct child): breadcrumb is just the epic subject.

## Step 1.5: Complexity Assessment

**Decompose if ALL hold:**
- 4+ files touching unrelated components, OR 3+ concerns each needing different test context
- `task.metadata.depth` is set AND < 3

`depth` is not guaranteed by all upstream skills. If absent, assume leaf — do NOT decompose, because unbounded recursion on tasks not structured for nesting would waste work and risk loops.

Threshold met → Step 1.6. Otherwise → Step 2.

## Step 1.6: Decompose Path

Skip Steps 2–4. Decompose, dispatch children, verify.

1. **Create children** — one per concern. Each: `metadata: {parent_id: task.id, depth: (task.metadata.depth ?? 0) + 1, ...inherited}`.
2. **Dispatch** — up to 4 concurrent `Task(subagent_type="general-purpose")` using Sub-Worker Prompt (Step 2). Each child's breadcrumb = parent breadcrumb + current task subject.
3. **Wait** for all children.
4. `Skill("acceptance", args=task.id)` — PASS → complete task. FAIL → report, do not complete.

## Step 2: Sub-Worker Prompt

Spawn `Task(subagent_type="general-purpose")`. **Trivial tasks** (single-file, <20 lines changed, no new logic — e.g. rename, config tweak): use `model="sonnet"` to save cost.

```
Implement task <task-id>.

## Task
<description from TaskGet>

## Ancestry
<breadcrumb from Step 1; omit if no parent>

## Epic Context
<root epic subject + metadata.design summary; omit if no parent>

## Protocol
1. TaskUpdate(taskId, status: "in_progress", owner: "solo")
2. Read every file in scope + 2-3 nearby test files to learn conventions.
   TDD: failing test → red → implement → green. No test infra → note, implement directly.
3. Build + test. Same root error 2x → stop + report. 3 distinct errors → report all, stop.
4. TaskUpdate(taskId, status: "completed", metadata: {completedAt: "<ISO 8601>"})

## Rules
- TDD first. Standards: rules/test-quality.md
- Never run git commands or Skill("commit")
- Only modify files in task scope
- Bug elsewhere → TaskCreate(subject: "Found: ...", metadata: {type: "bug", priority: "P2", project: "<repo root>"})
```

## Step 3: Refine

After sub-worker: `git diff --name-only HEAD` for changed files. None → skip. Changed → `Skill("refine")`.

## Step 4: Return

Completion summary (files changed, what implemented). No staging, no commit — caller handles.
