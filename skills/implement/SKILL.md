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

`TaskGet(taskId)` to inspect. Children = `TaskList()` filtered by `metadata.parent_id`.

- Single task (no children) → **Solo**
- 2-3 independent children (no blockedBy) → **Parallel**
- 4+ children (regardless of dependencies) OR children with blockedBy dependencies → **Swarm**

## Worker Prompt

All modes dispatch work via Task subagent (`subagent_type="general-purpose"`, `model="sonnet"`). Two variants based on coordination needs:

### Standalone Variant

Used by Solo and Parallel modes, and as TeamCreate fallback. No team messaging — worker completes and returns.

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
5. Run `Skill("refine")` on changed files. No changes needed → skip.

## Rules
- TDD: test first. Standards: rules/test-quality.md
- Never run git commands — orchestrator handles commits
- Only modify files in your task scope
- Bug found elsewhere → TaskCreate(subject: "Found: ...", metadata: {type: "bug", priority: "P2", project: "<repo root>"})
```

### Team-based Variant

Used by Swarm mode when TeamCreate succeeded. Adds team lead messaging and shutdown handshake.

```
Implement task <task-id>.

## Task
<task description from TaskGet>

## Epic Context
<epic subject + design summary>

## Protocol
1. TaskUpdate(taskId, status: "in_progress", owner: "worker-<taskId>")
   If fails → someone else claimed it. Report and stop.
2. Read every file in scope. Read 2-3 existing test files in the same module/directory to learn conventions (imports, framework, base classes, assertion patterns, naming, fixtures). Match their style. No nearby tests → use rules/test-quality.md defaults.
   Follow TDD: write failing tests, confirm red, implement until green. No test infra → note in report, implement directly.
3. Build + test. All green → continue.
   On failure: deduplicate errors (strip paths/line numbers). Same root error 2x → stop, report with context. 3 distinct errors → report all, stop.
4. TaskUpdate(taskId, status: "completed")
5. Run `Skill("refine")` on changed files. No changes needed → skip.
6. SendMessage(type="message", recipient="<team-lead-name>",
     content="Completed <task-id>: <summary>",
     summary="Completed <task-id>")
7. Wait for shutdown request. Approve it.

## Rules
- TDD: test first. Standards: rules/test-quality.md
- Only modify files in your task scope
- File conflict or blocker → SendMessage to team lead, wait
- Never run git commands
- Bug found elsewhere → TaskCreate(subject: "Found: ...", metadata: {type: "bug", priority: "P2", project: "<repo root>"})
```

## Solo Mode

1. `TaskUpdate(taskId, status: "in_progress", owner: "solo")`
2. If has `metadata.parent_id` → `TaskGet(parentId)` for epic context
3. Spawn single Task agent using **Standalone Worker Prompt**
4. Verify via `TaskGet(taskId)` → status is completed
5. → Stage Changes, then Continuation Prompt

## Parallel Mode

All tasks independent — fire and forget, no team overhead.

1. `TaskGet(epicId)` → subject + `metadata.design` as epic context
2. `TaskList()` filtered by `metadata.parent_id == epicId` → children
3. **Pre-flight:** children exist and have descriptions → continue. Empty or no children → stop, suggest `/prepare`
4. Spawn ALL children as Task agents in a SINGLE message (up to 4, queue remainder). Each uses **Standalone Worker Prompt** with epic context injected.
5. Wait for all to return. Check `TaskList()` for any incomplete.
6. Incomplete tasks → spawn another batch (max 2 retries per task)
7. → Verify, Stage Changes, then Continuation Prompt

## Swarm Mode

Used when tasks have dependency waves (blockedBy relationships). **Every task in every wave dispatches via Task subagent, including single-task waves.**

### Setup

1. `TaskGet(epicId)` → subject + `metadata.design` as epic context
2. `TaskList()` filtered by `metadata.parent_id == epicId` → children
3. **Pre-flight:** children exist and have descriptions → continue. Empty or no children → stop, suggest `/prepare`
4. `TeamCreate(team_name="impl-<slug>")`  (fall back to `impl-<epicId>` if no slug) If fails → fall back to sequential wave dispatch using **Standalone Worker Prompt**: dispatch unblocked tasks (up to 4), wait for completion, then dispatch newly unblocked tasks. Same rolling logic as Swarm but without team messaging.
5. Read team config `~/.claude/teams/impl-<slug>/config.json` → extract team lead name for worker prompts

### Rolling Scheduler

Dispatch tasks as soon as their dependencies are met, not in batch waves. Up to 4 workers run concurrently at any time.

```
# Initial dispatch
ready = TaskList() filtered by:
  metadata.parent_id == epicId AND
  status == "pending" AND
  blockedBy is empty

Spawn ready tasks (up to 4). Always use Team-based Worker Prompt
when TeamCreate succeeded (swarm mode), Standalone Worker Prompt
when no team exists.

active_count = len(spawned)
dispatch_count = {}  # task_id → number of times dispatched

# Rolling loop
while tasks remain incomplete:
  Wait for ANY worker to complete (Task returns or SendMessage received).

  On each completion:
    1. If worker completed its task → active_count--
       If Standalone worker returned without completing → check TaskList():
         task still in_progress → stuck, TaskUpdate(id, status: "pending", owner: ""), active_count--
    2. Shut down completed team-based workers (SendMessage shutdown_request)
    3. newly_ready = TaskList() filtered by:
         metadata.parent_id == epicId AND
         status == "pending" AND
         blockedBy is empty
    4. Skip any task where dispatch_count[task_id] >= 2 (mark as failed, report to user)
    5. slots = 4 - active_count
    6. Spawn min(len(newly_ready), slots) tasks → active_count += spawned, dispatch_count[id]++
    7. If active_count == 0 and no pending tasks remain → break

  Report progress: "N completed, M active, K pending, F failed"
```

### Verify (after all tasks complete)

1. Full test suite
2. Green → continue
3. Red → spawn fix agent (max 2 cycles)
4. Still red → escalate to user

### Teardown

1. `TaskUpdate(epicId, status: "completed")`
2. TeamDelete
3. → Stage Changes, then Continuation Prompt

## Stage Changes

Run after all workers complete, before prompting the user:

1. `git add -u` — stage all tracked modifications and deletions
2. `git status --short` — check for untracked files. If any exist, ask the user whether to stage them (e.g., new test fixtures, generated files).
3. `git diff --cached --stat` — show the staged summary to the user so they can see what will be committed.

## After Completion

After staging and showing the summary, proceed: `Skill("review")`
