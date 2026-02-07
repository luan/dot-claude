---
name: team-implement
description: "Triggers: 'team implement', 'parallel implement'. Coordinated multi-agent implementation for cross-layer or multi-module work."
argument-hint: "[epic-id]"
user-invocable: true
allowed-tools:
  - Teammate
  - Task
  - SendMessage
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

# Team Implement

Coordinated agent team implementation. Teammates self-select from beads queue, coordinate via messaging on interface overlaps.

## When to Use (vs `implement`)

- `bd swarm validate <epic-id> --verbose` shows max parallelism >= 3
- Shared interfaces need live coordination
- Otherwise → regular `implement`

## Instructions

1. `bd show <epic-id>` + `bd children <epic-id>`
2. `bd swarm validate <epic-id> --verbose` — cycles, parallelism, ready fronts
3. `bd swarm create <epic-id>` — register swarm
4. `bd merge-slot create` — git serialization lock
5. **File ownership prep:** Each task needs file ownership in beads metadata. Two ready tasks share files → `bd dep add` to serialize. Re-validate after.
6. `gt create luan/<short-description>`
7. Create team:

```
Teammate tool:
  operation: "spawnTeam"
  team_name: "impl-<short-desc>"
  description: "Implementing <epic summary>"
```

8. Spawn workers. Count = `min(max_parallelism, 4)`. Opus. Plan approval required:

```
Task tool (for each):
  subagent_type: "beads:task-agent"
  mode: "plan"
  team_name: "<team-name>"
  name: "worker-<n>"
  prompt: """
  Worker-<n> on epic <epic-id>.

  ## Work Loop
  1. `bd ready --parent <epic-id> --unassigned` — next unblocked unclaimed task
  2. `bd show <id>` — read instructions
  3. `bd update <id> --claim` — atomic claim (fails if claimed → step 1)
  4. `bd agent state worker-<n> working`
  5. Respect file ownership metadata — YOUR files while in_progress
  6. Failing test FIRST → minimal implementation
  7. Git ops:
     - `bd merge-slot acquire`
     - git add, commit, push
     - `bd merge-slot release`
  8. `bd close <id>`
  9. `bd agent heartbeat`
  10. → step 1. When `bd ready` returns nothing:
      - `bd agent state worker-<n> done`
      - Report completion to lead

  ## Fix Loop
  After completion, stay idle (don't shut down). Lead may send test failures:
  1. Read failure output + file paths
  2. Fix issue
  3. `bd merge-slot acquire` → commit/push → `bd merge-slot release`
  4. Report fix to lead
  5. Return to idle

  ## Context
  - Branch: luan/<description> (already created)
  - `bd prime` for workflow context

  ## Coordination
  - Need interface/type/API from teammate → MESSAGE them
  - Discover missed issue → MESSAGE team
  - Teammate asks about your interface → respond with exact signature/shape
  - Don't edit files outside task ownership without coordination

  ## Output
  Report: tasks completed, files changed, unresolved interface questions
  """
```

9. Review each teammate's plan. Verify:
   - No file ownership overlap on concurrent tasks
   - Interface assumptions compatible
   - Conflicts → add beads deps to serialize or message teammates

10. Monitor: `bd swarm status <epic-id>`. Watch:
    - Overlapping file claims → pause one, add dependency
    - Interface mismatches → nudge coordination
    - Discovered issues → fix now or beads bug
    - Done when Ready: 0, Active: 0

11. Verify-fix loop (max 2 cycles):
    - `bd children <epic-id>` — all tasks closed
    - Run full test suite (workers still idle)
    - **Green** → step 12
    - **Red** →
      a. Identify affected files from errors
      b. `bd children <epic-id>` → `bd show <task-id>` → match assignee to files
      c. Message worker(s): failure output, file paths, test names
      d. Workers fix + report back
      e. Re-run tests
    - 2 failed cycles → escalate to user with details

12. Shut down teammates, clean up team (only after green or user approval)

13. Invoke `finishing-branch` skill

## File Ownership Protocol

Per-task in beads metadata, not per-teammate.

- `bd update <id> --claim` = atomic (assignee + in_progress, fails if claimed)
- Two ready tasks share files → lead adds dep to serialize
- Shared files (types, config) owned by one task; others MESSAGE owner
- `--claim` atomic — no separate verify needed

## Key Rules

- **Opus** for teammates. Sonnet if mechanical/low-risk. Include `agents/implementer.md` guidelines.
- **Plan approval** — verify no file ownership overlaps on concurrent tasks
- **Self-selecting** — `bd ready --parent <epic-id> --unassigned`, not pre-assigned
- **Atomic claims** — `--claim` fails if claimed, no verify needed
- **TDD** — failing test first
- **File ownership strict** — per-task metadata
- **Task atomicity** — finish current task before stopping
- **Merge serialization** — `bd merge-slot acquire/release` around git ops
- **Agent tracking** — `bd agent state` + `bd agent heartbeat`
- **Swarm = truth** — `bd swarm status`, not manual polling
- **Beads = truth** — close beads tasks, not just team tasks
- **Verify before shutdown** — test suite while workers idle, failures → file owners
- **Fix cycles capped** — max 2 cycles → escalate
- **Always clean up** after green or user approval
