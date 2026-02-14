---
name: implement
description: "Execute a beads epic or task — auto-detects solo vs swarm mode, dispatches subagents to implement. Triggers: \"implement\", \"execute the plan\", \"build this\", \"code this plan\", \"start implementing\", \"ready to implement\", beads issue/epic ID."
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
  - Glob
  - Grep
  - Bash
---

# Implement

Detects solo vs swarm automatically. Handles both single-agent + multi-agent parallel execution.

**IMMEDIATELY dispatch.** Never implement on main thread.

## Step 1: Classify Work

1. `bd show <id> --json` (from $ARGUMENTS)
2. `bd swarm validate <id> --verbose`
3. Choose mode:
   - `--solo` flag → **Solo Mode**
   - Single task (no children) → **Solo Mode**
   - max parallelism < 3 → **Solo Mode**
   - max parallelism >= 3 → **Swarm Mode**

## Solo Mode

Dispatch via Task (subagent_type="general-purpose"):

```
Implement: $ARGUMENTS

## Job
1. **Pre-flight:** `bd children <epic-id>` — no children or tasks lack code in description → STOP, return "explore phase incomplete — no implementable tasks". Do NOT create tasks.
3. `gt create luan/<short-description>`
4. `bd ready` or `bd children <epic-id>`
5. Per task:
   - `bd show <task-id>` + `bd update <task-id> --claim`
   - **Step 0 — Understand:** Read EVERY file listed in task. Note indent style (tabs vs spaces + width). Verify prepared code matches current file state. Adapt if diverged.
   - **Indentation pre-flight:** Before first Edit to any file: read file, identify indent char + width. Use EXACTLY that in all edits to that file.
   - Copy test code EXACTLY → verify fails
   - Copy implementation EXACTLY → verify passes
   - **Completion gate (before bd close):**
     1. Detect build cmd: justfile/Makefile/package.json/CLAUDE.md
     2. Run build. Exit != 0 → trace error to root cause, fix (max 3 attempts)
     3. Run tests: new + existing touching modified files
     4. Run linter if applicable
     5. ALL green → `bd close`. ANY red after 3 attempts → report error output, do NOT close
   - **Fix methodology:** Read error → trace to root cause → ONE targeted fix. No guess-and-patch. >10 tool calls on single fix → checkpoint findings + escalate to caller.
   - `bd close <task-id>`
6. Done → `bd sync` + report completion to caller

## Task Atomicity
NEVER stop mid-task. Finish before any PR ops.

## Side Quests
Bug found? `bd create "Found: ..." --type bug --validate --deps discovered-from:<current-task-id>`
```

## Swarm Mode

Orchestrate parallel workers via waves.

### Setup

1. `bd show <epic-id>` + `bd children <epic-id>`
2. `bd swarm validate <epic-id> --verbose`
3. `bd merge-slot create`
4. **File ownership:** Two ready tasks share files → `bd dep add` to serialize. Re-validate.
5. `gt create luan/<short-description>`
6. Create team:
   ```
   TeamCreate:
     team_name: "impl-<short-desc>"
     description: "Implementing <epic summary>"
   ```

### Wave Loop

```
while true:
  ready = `bd ready --parent <epic-id> --unassigned`
  if empty → break

  Spawn ALL ready tasks in SINGLE message (parallel).
  Workers = min(ready_count, 4). Model: sonnet.

  Each worker (Task, subagent_type="general-purpose", mode="plan", team_name="<team>", name="worker-<n>"):

  """
  Worker-<n> on epic <epic-id>.

  ## Work Loop
  1. `bd ready --parent <epic-id> --unassigned`
  2. `bd show <id>` → `bd update <id> --claim` (fails if claimed → step 1)
  3. **Understand first:** Read every file in task. Note indent (char + width). Verify prepared code matches current state. Adapt if diverged.
  4. Respect file ownership — YOUR files while in_progress
  5. Before first Edit per file: read it, match indent exactly.
  6. Failing test FIRST → minimal implementation
  7. **Build gate (max 3 attempts):**
     a. Build cmd from justfile/Makefile/package.json/CLAUDE.md
     b. Build + tests + linter. All green → continue. Red → root-cause, ONE fix.
     c. 3 fails → report error, do NOT close.
     d. >10 tool calls on one fix → checkpoint + escalate.
  8. `bd merge-slot acquire --wait` → git add/commit → `bd merge-slot release`
  9. `bd close <id>`
  10. → step 1. Empty → report completion

  ## File Boundaries (HARD RULE)
  NEVER edit files outside your task ownership.
  Need change in another worker's file:
  1. MESSAGE file owner
  2. Owner idle → MESSAGE team lead
  3. Never edit directly — even "just one line"

  ## Context
  Branch: luan/<description> (already created).
  """

  Wait for all workers to complete.
  Check `bd swarm status <epic-id>` for stuck tasks.
  Stuck → skip, proceed if non-stuck done.
```

### Verify-Fix Loop (max 2 cycles)

1. All closed → full test suite (workers idle)
2. **Green** → continue
3. **Red** → match errors to owners via `bd children`, message workers w/ failure output, re-run
4. 2 failed → escalate to user

### Teardown

1. Shut down teammates (SendMessage shutdown_request)
2. TeamDelete
3. `bd sync` + report completion to caller

## Key Rules

- Main thread does NOT implement — subagent/team does
- Copy code EXACTLY from task descriptions
- Task atomicity — never stop mid-task
- Pre-flight required — bail if no implementable tasks
- Swarm: sonnet workers, plan approval, merge serialization
- Swarm: spawn ALL wave workers in single message
- Fix cycles capped at 2 → escalate to user
