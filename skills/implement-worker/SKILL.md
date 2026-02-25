---
name: implement-worker
description: "Internal skill. Implements a single task: claims it, runs TDD + build via sub-worker. Invoked by implement and other orchestrators."
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

Single-task mini-orchestrator. Receives a task ID, spawns a code-only sub-worker.

## Step 1: Load Task

`TaskGet(<task-id>)` — capture description and metadata.

**Pre-computed context** (set by implement): if `metadata.breadcrumb` and `metadata.epic_design` exist, use them directly — skip ancestor walk.

**Fallback** (invoked outside implement): if `metadata.parent_id` exists, walk ancestor chain to epic root:
1. Repeatedly `TaskGet(parent_id)` until no `parent_id`
2. Build breadcrumb root-first, excluding current task: `"Epic > Phase > ..."`
3. Source `metadata.design` from root epic

## Step 1.5: Complexity Assessment

**Decompose if ALL hold:**
- 4+ files touching unrelated components, OR 3+ concerns needing different test context
- `task.metadata.depth` is set AND < 3 (depth cap prevents unbounded nesting)

`depth` absent → assume leaf, do NOT decompose (prevents unbounded recursion).

Threshold met → Step 1.6. Otherwise → Step 2.

## Step 1.6: Decompose Path

Skip Steps 2–6. Decompose, dispatch children, verify.

1. **Create children** — one per concern. Each: `metadata: {parent_id: task.id, depth: (task.metadata.depth ?? 0) + 1, ...inherited}`.
2. **Dispatch** — up to 4 concurrent `Task(subagent_type="general-purpose")` using Sub-Worker Prompt (Step 2). Each child's breadcrumb = parent breadcrumb + current task subject.
3. **Wait** for all children.
4. `Skill("acceptance", args=task.id)` — PASS → complete task. FAIL → report, do not complete.

## Step 2: Claim Task

`TaskUpdate(taskId, status: "in_progress", owner: "solo")` — establish ownership before dispatching any work.

## Step 3: Sub-Worker Prompt

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
1. Read every file in scope + 2-3 nearby test files to learn conventions.
   TDD: failing test → red → implement → green. No test infra → note, implement directly.
2. Build + test. Same root error 2x → stop + report. 3 distinct errors → report all, stop.
3. Self-check: re-read changed files. Remove debug artifacts, low-value comments, unused imports. Flatten nesting via early returns. Apply language-idiomatic patterns.

## Rules
- TDD first. Standards: rules/test-quality.md
- Never run git commands or Skill("commit")
- Only modify files in task scope
- Bug elsewhere → TaskCreate(subject: "Found: ...", metadata: {type: "bug", priority: "P2", project: "<repo root>"})
```

## Step 4: Complete Task

`TaskUpdate(taskId, status: "completed", metadata: {completedAt: "<ISO 8601>"})` — mark complete here, not inside sub-worker.

## Step 5: Return

Completion summary (files changed, what implemented). No staging, no commit — caller handles.
