---
name: implement
description: "Execute an epic or task — auto-detects solo vs swarm mode, dispatches subagents to implement. Triggers: \"implement\", \"execute the plan\", \"build this\", \"code this plan\", \"start implementing\", \"ready to implement\", epic/task ID. Do NOT use when: a full autonomous end-to-end workflow (explore through commit) is needed — use /vibe instead."
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

## Recovery

Before classifying, check for orphaned work:

1. `TaskList()` → find any epic where `metadata.impl_team` is set AND `status == "in_progress"`
2. If none found → proceed to Step 2: Classify normally

**If orphaned epic found:**

3. Read `~/.claude/teams/<impl_team>/config.json`
4. **Config exists** (team is alive) → extract team lead name, re-enter Rolling Scheduler using current metadata counters (`impl_completed`, `impl_active`, `impl_pending`) as starting state. Shut down any workers that didn't send completion messages, re-dispatch their tasks as pending.
5. **Config missing** (team died) → clear `impl_team` from metadata (`TaskUpdate(epicId, metadata: {impl_team: null})`). Dispatch remaining pending children sequentially using **Standalone Worker Prompt**, up to 4 at a time, until all complete. Then proceed to Verify.
6. After recovery completes → continue to Teardown (do not re-run Setup)

**Note:** If the argument passed to `/implement` specifies a different epic/task than the orphaned one, prioritize the argument. Only auto-recover when no explicit target is given (no-argument invocation).

## Step 2: Classify

`TaskGet(taskId)` to inspect.

**Recursive descendants:** `TaskList()` filtered by `metadata.parent_id == taskId` → one level. Repeat for each child until a level returns empty. Collect all nodes. **Leaves** = descendants with no children of their own.

- Single task (no descendants) → **Solo**
- 2-3 independent leaves (no blockedBy) → **Parallel**
- 4+ leaves OR any leaf with blockedBy dependencies → **Swarm**
- **Readiness check (Parallel/Swarm only):** Scan child task descriptions for `## Files` and `## Approach` sections. If 2+ tasks lack either → AskUserQuestion: "N tasks lack detailed file lists or approach descriptions — workers may need to guess. Continue or refine task briefs first?" Options: "Continue anyway" / "Refine briefs first". User choosing "Continue" proceeds normally.

## Worker Prompt

All modes dispatch work via Task subagent (`subagent_type="general-purpose"`). For trivial tasks (single-file rename, config tweak, simple find-and-replace), use `model="sonnet"` to save cost. Two variants based on coordination needs:

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
4. TaskUpdate(taskId, status: "completed", metadata: {completedAt: "<current ISO 8601 timestamp>"})
5. Run `Skill("refine")` on changed files. No changes needed → skip.

## Rules
- TDD: test first. Standards: rules/test-quality.md
- Never run git commands — orchestrator handles commits
- Never invoke Skill("commit") — orchestrator handles commits
- Only modify files in your task scope
- Bug found elsewhere → TaskCreate(subject: "Found: ...", metadata: {type: "bug", priority: "P2", project: "<repo root>"})
```

### Codex Conventions Component

Injected into every Codex dispatch prompt — not used by Claude workers.

```
{codex_conventions}

## Project Conventions (injected from ~/.claude)

### Code Style
- Clarity over brevity. No clever one-liners that obscure intent.
- No dead code, commented-out code, "just in case" code.
- Comments for WHY / edge cases / surprising behavior only.
- Three similar lines before abstracting.

### Testing (TDD required)
- Write failing test first, confirm red, then implement.
- Every test must answer: "What bug would this catch?"
- Banned: tautology mocks, getter/setter tests, implementation mirroring, coverage padding.
- Mock only: external services, network, filesystem, clock. Never mock what you own.

### File Structure
- Exact file paths from task description — do not create new files outside scope.
- One logical change per file modification.

### Naming
- Match surrounding code conventions (check 2-3 nearby files first).
- No versioned names (FooV2), no migration wrappers.
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
2. Skill("implement-worker", args="<task-id>")
3. SendMessage(type="message", recipient="<team-lead-name>",
     content="Completed <task-id>: <summary>",
     summary="Completed <task-id>")
4. Wait for shutdown request. Approve it.

## Rules
- TDD: test first. Standards: rules/test-quality.md
- Only modify files in your task scope
- File conflict or blocker → SendMessage to team lead, wait
- Never run git commands
- Never invoke Skill("commit") — orchestrator handles commits
- Bug found elsewhere → TaskCreate(subject: "Found: ...", metadata: {type: "bug", priority: "P2", project: "<repo root>"})
```

## Solo Mode

1. `TaskUpdate(taskId, status: "in_progress", owner: "solo")`
2. If has `metadata.parent_id` → walk ancestor chain: `TaskGet(parent_id)` repeatedly until a task with no `metadata.parent_id`; that root is the epic context
3. Spawn single Task agent using **Standalone Worker Prompt**
4. Verify via `TaskGet(taskId)` → status is completed
5. If task has `metadata.parent_id` → `Skill("acceptance", args="<parentId>")`
6. → Stage Changes, then Continuation Prompt

## Parallel Mode

All tasks independent — fire and forget, no team overhead.

1. `TaskGet(epicId)` → subject + `metadata.design` as epic context
   `TaskUpdate(epicId, metadata: {impl_mode: "parallel"})`
2. `TaskList()` filtered by `metadata.parent_id == epicId` → children
3. **Pre-flight:** children exist and have descriptions → continue. Empty or no children → stop, suggest `/prepare`
4. Spawn ALL children as Task agents in a SINGLE message (up to 4, queue remainder). Each uses **Standalone Worker Prompt** with epic context injected.
5. Wait for all to return. Check `TaskList()` for any incomplete.
6. Incomplete tasks → spawn another batch (max 2 retries per task)
7. `TaskUpdate(epicId, metadata: {impl_mode: null})`
8. → Verify, then `Skill("acceptance", args="<epicId>")`, then Stage Changes, then Continuation Prompt

## Swarm Mode

Used when tasks have dependency waves (blockedBy relationships). **Every task in every wave dispatches via Task subagent, including single-task waves.**

### Setup

1. `TaskGet(epicId)` → subject + `metadata.design` as epic context
2. `TaskList()` filtered by `metadata.parent_id == epicId` → children
3. **Pre-flight:** children exist and have descriptions → continue. Empty or no children → stop, suggest `/prepare`
4. `TeamCreate(team_name="impl-<slug>")`  (fall back to `impl-<epicId>` if no slug) If fails → fall back to sequential wave dispatch using **Standalone Worker Prompt**: dispatch unblocked tasks (up to 4), wait for completion, then dispatch newly unblocked tasks. Same rolling logic as Swarm but without team messaging. On failure: `TaskUpdate(epicId, metadata: {impl_mode: "standalone-fallback"})`
5. Read team config `~/.claude/teams/impl-<slug>/config.json` → extract team lead name for worker prompts
   `TaskUpdate(epicId, metadata: {impl_team: "impl-<slug>", impl_mode: "swarm"})`
6. `CODEX_AVAILABLE = Bash("which codex >/dev/null 2>&1 && echo yes || echo no")` — detect Codex CLI

### Rolling Scheduler

Dispatch tasks as soon as their dependencies are met, not in batch waves. Up to 4 workers run concurrently at any time.

```
# descendants(epicId): TaskList() filtered by metadata.parent_id == epicId → one level;
#   repeat for each child until a level returns empty; collect all nodes.
# leaf(task): task has no children — TaskList() filtered by metadata.parent_id == task.id is empty.

# Initial dispatch
ready = [t for t in descendants(epicId) if t.status == "pending" AND t.blockedBy is empty AND leaf(t)]

Spawn ready tasks (up to 4) using dispatch routing:
  leaf(task) AND CODEX_AVAILABLE:
    → Codex dispatch
  All others:
    → Team-based Worker Prompt (swarm) / Standalone Worker Prompt (no team)

active_count = len(spawned)
dispatch_count = {}       # task_id → number of Claude dispatches
codex_attempted = set()   # task_ids that already had a Codex attempt
dispatched = set(t.id for t in spawned)  # all task_ids ever dispatched (avoid re-dispatch)

# Codex dispatch (leaf tasks only, when CODEX_AVAILABLE):
#   1. Build prompt: {codex_conventions} template + "\n\n## Task\n" + task description
#   2. Bash("codex -q --task '<escaped_prompt>'", timeout=300000, run_in_background=true)
#   3. codex_attempted.add(id)
#   On completion:
#     exit 0 → spawn Claude review worker to run tests, Skill("refine"),
#              and TaskUpdate(taskId, status: "completed")
#     non-zero/timeout → TaskUpdate(id, status: "pending", owner: ""),
#                         re-dispatch immediately as Claude worker (counts as dispatch_count[id]++)

# Rolling loop
while tasks remain incomplete:
  Wait for ANY worker to complete (Task returns, SendMessage received,
  or Codex background task finishes via TaskOutput).

  On each completion:
    1. If worker completed its task → active_count--
       If Codex task completed:
         exit 0 → spawn Claude review worker (test + refine + mark completed),
                   active_count stays (review worker replaces Codex slot)
         non-zero/timeout → active_count--, TaskUpdate(id, status: "pending", owner: ""),
                            add to newly_ready for Claude dispatch
       If Standalone worker returned without completing → check TaskList():
         task still in_progress → stuck, TaskUpdate(id, status: "pending", owner: ""), active_count--
       TaskUpdate(epicId, metadata: {
         impl_completed: <count of completed children>,
         impl_active: active_count,
         impl_pending: <count of pending children>
       })
    2. Shut down completed team-based workers (SendMessage shutdown_request)
    3. # Re-scan: worker may have created child tasks (decomposition); former leaf may now be a grouping node
       fresh_descendants = descendants(epicId)  # re-query full subtree
       newly_ready = [t for t in fresh_descendants if t.status == "pending" AND t.blockedBy is empty AND leaf(t) AND t.id not in dispatched]
    4. Skip any task where dispatch_count[task_id] >= 2 (mark as failed, report to user)
    5. slots = 4 - active_count
    6. Spawn min(len(newly_ready), slots) tasks → active_count += spawned
       dispatched.update(id for each spawned task)
       Codex routing: if CODEX_AVAILABLE AND leaf(task) AND id not in codex_attempted → Codex dispatch
       Else → Claude worker, dispatch_count[id]++
    7. If active_count == 0 and no pending tasks remain → break

  Report progress: "N completed, M active, K pending, F failed"
```

### Verify (after all tasks complete)

1. Full test suite
2. Green → continue
3. Red → spawn fix agent (max 2 cycles)
4. Still red → escalate to user

### Teardown

1. `TaskUpdate(epicId, metadata: {impl_team: null, impl_mode: null, impl_completed: null, impl_active: null, impl_pending: null})`
2. `TaskUpdate(epicId, status: "completed", metadata: {completedAt: "<current ISO 8601 timestamp>"})`
3. TeamDelete
4. `Skill("acceptance", args="<epicId>")`
5. → Stage Changes, then Continuation Prompt

## Stage Changes

Run after all workers complete, before prompting the user:

1. `git add -u` — stage all tracked modifications and deletions
2. `git status --short` — check for untracked files. If any exist, ask the user whether to stage them (e.g., new test fixtures, generated files).
3. `git diff --cached --stat` — show the staged summary to the user so they can see what will be committed.

## After Completion

Stop after staging and showing the summary. Acceptance check runs automatically before staging. The user needs to verify functionality before review — do not auto-invoke review.
