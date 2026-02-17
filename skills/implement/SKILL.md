---
name: implement
description: "Execute an epic or task — auto-detects solo vs swarm mode, dispatches subagents to implement. Triggers: \"implement\", \"execute the plan\", \"build this\", \"code this plan\", \"start implementing\", \"ready to implement\", epic/task ID."
argument-hint: "[epic-id|task-id] [--solo]"
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

- ID in `$ARGUMENTS` → use it
- Else: `TaskList()` → first in_progress epic (`metadata.label == "epic"`)
- Else: first pending epic
- Else: first in_progress or pending task with empty blockedBy
- Nothing → suggest `/explore` then `/prepare`, stop

## Step 2: Classify

`TaskGet(taskId)` to inspect. Children = `TaskList()` filtered
by `metadata.parent_id`.

- Single task (no children) → **Solo**
- Multiple children, all independent (no blockedBy) → **Parallel**
- Multiple children with blockedBy dependencies → **Swarm**

## Solo Mode

1. `TaskUpdate(taskId, status: "in_progress")`
2. If has `metadata.parent_id` → `TaskGet(parentId)` for epic context
3. Spawn single Task agent (`subagent_type="general-purpose"`):

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

4. Verify via `TaskGet(taskId)` → status is completed
5. → Continuation Prompt

## Parallel Mode

All tasks independent — fire and forget, no team overhead.

1. `TaskGet(epicId)` → subject + `metadata.design` as epic context
2. `TaskList()` filtered by `metadata.parent_id == epicId` → children
3. **Pre-flight:** children exist and have descriptions → continue.
   Empty or no children → stop, suggest `/prepare`
4. Spawn ALL children as Task agents in a SINGLE message
   (up to 4, queue remainder). Use the Solo worker prompt for each,
   injecting epic context.
5. Wait for all to return. Check `TaskList()` for any incomplete.
6. Incomplete tasks → spawn another batch (max 2 retries per task)
7. → Verify, then Continuation Prompt

## Swarm Mode

Used when tasks have dependency waves (blockedBy relationships).

### Setup

1. `TaskGet(epicId)` → subject + `metadata.design` as epic context
2. `TaskList()` filtered by `metadata.parent_id == epicId` → children
3. **Pre-flight:** children exist and have descriptions → continue.
   Empty or no children → stop, suggest `/prepare`
4. `TeamCreate(team_name="impl-<epicId>")`
   If fails → fall back to Parallel Mode (process waves via
   Task agents without team coordination).
5. Read team config `~/.claude/teams/impl-<epicId>/config.json`
   → extract team lead name for worker prompts

### Wave Loop

```
while true:
  ready = TaskList() filtered by:
    metadata.parent_id == epicId AND
    status == "pending" AND
    blockedBy is empty
  if empty → break

  Spawn ALL ready tasks in SINGLE message (parallel).
  Workers = min(task_count, 4).

  Wait for completion (see Wave Completion below).

  # Recover stuck tasks
  stuck = TaskList() filtered by:
    metadata.parent_id == epicId AND status == "in_progress"
  for each stuck task not in just-completed set:
    TaskUpdate(stuckId, status: "pending", owner: "")

  Shut down wave workers (SendMessage shutdown_request).
  Report: "Wave N: M completed, K stuck"
```

### Worker Prompt

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
