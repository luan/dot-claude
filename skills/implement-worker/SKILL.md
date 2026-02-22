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
  - Read
  - Bash
---

# Implement Worker

Single-task mini-orchestrator. Receives a task ID, spawns a code-only sub-worker, then runs refine on the result.

## Step 1: Load Task

`TaskGet(<task-id>)` — capture description and `metadata.parent_id`.

If `metadata.parent_id` exists → `TaskGet(parentId)` for epic subject + `metadata.design` as epic context.

## Step 2: Spawn Sub-Worker

Spawn a single `Task(subagent_type="general-purpose", model="sonnet")` with this prompt (fill in from Step 1):

```
Implement task <task-id>.

## Task
<task description from TaskGet>

## Epic Context (if applicable)
<epic subject + metadata.design summary>

## Protocol
1. TaskUpdate(taskId, status: "in_progress", owner: "solo")
2. Read every file in scope. Read 2-3 existing test files in the same module/directory to learn conventions (imports, framework, base classes, assertion patterns, naming, fixtures). Match their style. No nearby tests → use rules/test-quality.md defaults.
   Follow TDD: write failing tests, confirm red, implement until green. No test infra → note in report, implement directly.
3. Build + test. All green → continue.
   On failure: deduplicate errors (strip paths/line numbers). Same root error 2x → stop, report with context. 3 distinct errors → report all, stop.
4. TaskUpdate(taskId, status: "completed")

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
3. Changed files exist → `Skill("refine")`. When refine asks about committing, select "Done for now" — caller handles commits.

## Step 4: Return

Return a completion summary (files changed, what was implemented). No staging, no commit — caller handles those.
