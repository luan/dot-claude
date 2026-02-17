---
name: implement
description: "Execute an epic or task — auto-detects solo vs swarm mode, dispatches subagents to implement. Triggers: \"implement\", \"execute the plan\", \"build this\", \"code this plan\", \"start implementing\", \"ready to implement\", epic/task ID."
argument-hint: "[<epic-slug>|t<id>|<id>] [--solo]"
user-invocable: true
allowed-tools:
  - Task
  - Skill
  - SendMessage
  - TeamCreate
  - TeamDelete
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
  - AskUserQuestion
  - Read
  - Bash
---

# Implement

**IMMEDIATELY dispatch.** Never implement on main thread.

## Step 1: Find Work

Resolve argument:
- Slug (non-numeric, e.g. `reopen-windows`) → `TaskList()`, find task where `metadata.slug` matches
- `t<N>` → strip prefix, `TaskGet(N)`
- Bare number `N` → `TaskGet(N)`
- No argument → `TaskList()` → first in_progress epic, else first pending epic, else first unblocked task
- Nothing found → suggest `/explore` then `/prepare`, stop

## Step 2: Classify

`TaskGet(taskId)` to inspect. Children = `TaskList()` filtered
by `metadata.parent_id`.

- Single task (no children) → **Solo**
- Multiple children, all independent (no blockedBy) → **Parallel**
- Multiple children with blockedBy dependencies → **Swarm**

## Worker Prompt

All modes dispatch work via Task subagent (`subagent_type="general-purpose"`).
Two variants based on coordination needs:

### Standalone Variant

Used by Solo and Parallel modes, and as TeamCreate fallback. No team
messaging — worker completes and returns.

```
Implement task <task-id>.

## Task
<task description from TaskGet>

## Epic Context (if applicable)
<epic subject + metadata.design summary>

## Protocol
1. TaskUpdate(taskId, status: "in_progress", owner: "solo")
2. Read every file in scope. Implement from brief.
3. Build + test. All green → continue. 3 failures → report, stop.
4. TaskUpdate(taskId, status: "completed")

## Rules
- Never run git commands — orchestrator handles commits
- Only modify files in your task scope
- Bug found elsewhere → TaskCreate(subject: "Found: ...", metadata: {type: "bug"})
```

### Team-based Variant

Used by Swarm mode when TeamCreate succeeded. Adds team lead
messaging and shutdown handshake.

```
Implement task <task-id>.

## Task
<task description from TaskGet>

## Epic Context
<epic subject + design summary>

## Protocol
1. TaskUpdate(taskId, status: "in_progress", owner: "worker-<taskId>")
   If fails → someone else claimed it. Report and stop.
2. Read full context, implement the work.
3. Build + test. All green → continue. 3 failures → report, stop.
4. TaskUpdate(taskId, status: "completed")
5. SendMessage(type="message", recipient="<team-lead-name>",
     content="Completed <task-id>: <summary>",
     summary="Completed <task-id>")
6. Wait for shutdown request. Approve it.

## Rules
- Only modify files in your task scope
- File conflict or blocker → SendMessage to team lead, wait
- Never run git commands
- Bug found elsewhere → TaskCreate(subject: "Found: ...", metadata: {type: "bug"})
```

## Solo Mode

1. `TaskUpdate(taskId, status: "in_progress")`
2. If has `metadata.parent_id` → `TaskGet(parentId)` for epic context
3. Spawn single Task agent using **Standalone Worker Prompt**
4. Verify via `TaskGet(taskId)` → status is completed
5. → Continuation Prompt

## Parallel Mode

All tasks independent — fire and forget, no team overhead.

1. `TaskGet(epicId)` → subject + `metadata.design` as epic context
2. `TaskList()` filtered by `metadata.parent_id == epicId` → children
3. **Pre-flight:** children exist and have descriptions → continue.
   Empty or no children → stop, suggest `/prepare`
4. Spawn ALL children as Task agents in a SINGLE message
   (up to 4, queue remainder). Each uses **Standalone Worker Prompt**
   with epic context injected.
5. Wait for all to return. Check `TaskList()` for any incomplete.
6. Incomplete tasks → spawn another batch (max 2 retries per task)
7. → Verify, then Continuation Prompt

## Swarm Mode

Used when tasks have dependency waves (blockedBy relationships).
**Every task in every wave dispatches via Task subagent, including
single-task waves.**

### Setup

1. `TaskGet(epicId)` → subject + `metadata.design` as epic context
2. `TaskList()` filtered by `metadata.parent_id == epicId` → children
3. **Pre-flight:** children exist and have descriptions → continue.
   Empty or no children → stop, suggest `/prepare`
4. `TeamCreate(team_name="impl-<slug>")`  (fall back to `impl-<epicId>` if no slug)
   If fails → fall back to Parallel Mode — dispatch waves via
   Task agents using **Standalone Worker Prompt** (no team coordination).
5. Read team config `~/.claude/teams/impl-<slug>/config.json`
   → extract team lead name for worker prompts

### Wave Loop

```
while true:
  ready = TaskList() filtered by:
    metadata.parent_id == epicId AND
    status == "pending" AND
    blockedBy is empty
  if empty → break

  # Always dispatch via Task subagent, even for single-task waves.
  if len(ready) == 1:
    Spawn 1 Task agent using Standalone Worker Prompt
    (no team messaging needed for single worker).
  else:
    Spawn ALL ready tasks using Team-based Worker Prompt.
    Workers = min(task_count, 4).

  Wait for completion (see Wave Completion below).

  # Recover stuck tasks
  stuck = TaskList() filtered by:
    metadata.parent_id == epicId AND status == "in_progress"
  for each stuck task not in just-completed set:
    TaskUpdate(stuckId, status: "pending", owner: "")

  # Shut down team-based wave workers only (not standalone)
  if len(ready) > 1:
    Shut down wave workers (SendMessage shutdown_request).
  Report: "Wave N: M completed, K stuck"
```

### Wave Completion Detection

1. Track spawned_count = N, completed_count = 0
2. Worker sends completion message → completed_count++
3. completed_count == N → wave done
4. Worker goes idle WITHOUT completion message:
   - `TaskList()` → if task still in_progress → stuck
   - Decrement expected count
   - All non-stuck done → proceed

### Verify (after final wave)

1. Full test suite
2. Green → continue
3. Red → spawn fix agent (max 2 cycles)
4. Still red → escalate to user

### Teardown

1. `TaskUpdate(epicId, status: "completed")`
2. TeamDelete
3. → Continuation Prompt

## Continuation Prompt

AskUserQuestion:
- "Continue to /review" (Recommended) — "Review changes before committing"
- "Implement next task" — "Pick up the next pending task"
- "Review changes manually first" — "Inspect the diff before automated review"
- "Done for now" — "Leave for later /next"

If /review → `Skill("review")`
If next task → `Skill("implement")`
