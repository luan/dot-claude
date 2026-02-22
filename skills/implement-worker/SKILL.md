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

If `metadata.parent_id` exists → walk the ancestor chain:
1. `TaskGet(metadata.parent_id)` — capture subject and its own `metadata.parent_id`
2. Repeat until a task has no `parent_id` — that is the epic root
3. Build breadcrumb root-first, excluding the current task: `"Epic Subject > Phase Subject > ..."`
4. Source `metadata.design` from the root epic (not the immediate parent)

Flat case (direct child of epic): chain has one element; breadcrumb is just the epic subject.

## Step 1.5: Complexity Assessment

Inspect the loaded task to decide: decompose or implement directly.

**Decompose if ALL of the following hold:**
- 4+ distinct files in the task's `## Files` section touching unrelated components, OR 3+ distinct concerns in `## Approach` each requiring a different test context
- `task.metadata.depth` is set AND `< 3` (if `depth` is absent, assume leaf — do NOT decompose)

If the threshold is met → **DECOMPOSE** (go to Step 1.6). Otherwise → continue to Step 2.

## Step 1.6: Mini-Orchestrator (DECOMPOSE path only)

Skip Steps 2–4 entirely. Decompose the task, dispatch children, then verify.

1. **Create child tasks** — one per distinct concern or file group identified in Step 1.5. For each child:
   ```
   TaskCreate(subject: "<child concern>", description: "<scoped description>",
     metadata: { parent_id: task.id, depth: (task.metadata.depth ?? 0) + 1,
                 <other metadata inherited from parent, e.g. project> })
   ```
2. **Build child ancestry breadcrumb** — append current task subject to parent's breadcrumb:
   `"<existing breadcrumb> > <current task subject>"`. Use this when filling each child's Sub-Worker Prompt.
3. **Dispatch** — spawn up to 4 children concurrently as `Task(subagent_type="general-purpose")` using the Sub-Worker Prompt from Step 2 (one per child task-id/breadcrumb).
4. **Wait** — collect all child results before continuing.
5. **Acceptance** — `Skill("acceptance", args=task.id)`.
   - PASS → `TaskUpdate(task.id, status: "completed", metadata: {completedAt: "<current ISO 8601 timestamp>"})`
   - FAIL → report failure details; do **not** mark task completed.

## Step 2: Spawn Sub-Worker

Spawn a single `Task(subagent_type="general-purpose")` with this prompt (fill in from Step 1). For trivial tasks (single-file rename, config tweak, simple find-and-replace), use `model="sonnet"` to save cost:

```
Implement task <task-id>.

## Task
<task description from TaskGet>

## Ancestry
<breadcrumb from Step 1, e.g. "Epic Subject > Phase Subject"; omit section if no parent>

## Epic Context
<root epic subject + metadata.design summary; omit section if no parent>

## Protocol
1. TaskUpdate(taskId, status: "in_progress", owner: "solo")
2. Read every file in scope. Read 2-3 existing test files in the same module/directory to learn conventions (imports, framework, base classes, assertion patterns, naming, fixtures). Match their style. No nearby tests → use rules/test-quality.md defaults.
   Follow TDD: write failing tests, confirm red, implement until green. No test infra → note in report, implement directly.
3. Build + test. All green → continue.
   On failure: deduplicate errors (strip paths/line numbers). Same root error 2x → stop, report with context. 3 distinct errors → report all, stop.
4. TaskUpdate(taskId, status: "completed", metadata: {completedAt: "<current ISO 8601 timestamp>"})

## Rules
- TDD: test first. Standards: rules/test-quality.md
- Never run git commands — orchestrator handles commits
- Never invoke Skill("commit") — orchestrator handles commits
- Only modify files in your task scope
- Bug found elsewhere → TaskCreate(subject: "Found: ...", metadata: {type: "bug", priority: "P2", project: "<repo root>"})
```

## Step 3: Refine

After sub-worker returns:

1. `Bash("git diff --name-only HEAD")` — collect changed files
2. No changed files → skip refine, go to Step 4
3. Changed files exist → `Skill("refine")`

## Step 4: Return

Return a completion summary (files changed, what was implemented). No staging, no commit — caller handles those.
