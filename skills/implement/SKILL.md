---
name: implement
description: "Triggers: 'invoke implement', 'implement with arg', beads issue/epic ID, 'execute the plan', 'build this', 'code this plan', 'start implementing', 'now implement', 'time to implement', 'ready to implement', 'begin implementation', 'let me implement', 'To continue: use Skill tool to invoke implement'. Extract issue-id from 'with arg X'."
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
   - Copy test code EXACTLY → verify fails
   - Copy implementation EXACTLY → verify passes
   - **Completion gate (MANDATORY):** No `bd close` without ALL:
     1. Build clean (zero errors)
     2. Tests pass (new + existing in scope)
     3. Linter clean (clippy/eslint/swiftc as applicable)
     Detect build cmd from justfile/Makefile/package.json/CLAUDE.md.
     ANY failure → fix loop. Never skip.
   - `bd close <task-id>`
6. Done → invoke finishing-branch skill

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
  3. Respect file ownership — YOUR files while in_progress
  4. Failing test FIRST → minimal implementation
  4b. Build check. Fix until clean.
  5. `bd merge-slot acquire --wait` → git add/commit → `bd merge-slot release`
  6. `bd close <id>`
  7. → step 1. Empty → report completion

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
3. Invoke finishing-branch skill

## Key Rules

- Main thread does NOT implement — subagent/team does
- Copy code EXACTLY from task descriptions
- Task atomicity — never stop mid-task
- Pre-flight required — bail if no implementable tasks
- Swarm: sonnet workers, plan approval, merge serialization
- Swarm: spawn ALL wave workers in single message
- Fix cycles capped at 2 → escalate to user
