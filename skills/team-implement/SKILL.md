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

Coordinated agent team. Teammates self-select from beads queue, coordinate via messaging on interface overlaps.

## When (vs `implement`)

- `bd swarm validate` shows max parallelism >= 3
- Shared interfaces need live coordination
- Otherwise → regular `implement`

## Steps

1. `bd show <epic-id>` + `bd children <epic-id>`
2. `bd swarm validate <epic-id> --verbose` — cycles, parallelism, ready fronts
3. `bd swarm create <epic-id>`
4. `bd merge-slot create`
5. **File ownership:** Each task needs file ownership in metadata. Two ready tasks share files → `bd dep add` to serialize. Re-validate.
6. `gt create luan/<short-description>`
7. Create team:

```
Teammate tool:
  operation: "spawnTeam"
  team_name: "impl-<short-desc>"
  description: "Implementing <epic summary>"
```

8. Spawn workers. Count = `min(max_parallelism, 4)`. Sonnet (model: "sonnet") — follows detailed plans with TDD. Plan approval required:

```
Task tool (each):
  subagent_type: "beads:task-agent"
  mode: "plan"
  team_name: "<team-name>"
  name: "worker-<n>"
  prompt: """
  Worker-<n> on epic <epic-id>.

  ## Work Loop
  1. `bd ready --parent <epic-id> --unassigned`
  2. `bd show <id>` → `bd update <id> --claim` (fails if claimed → step 1)
  3. `bd agent state worker-<n> working`
  4. Respect file ownership — YOUR files while in_progress
  5. Failing test FIRST → minimal implementation
  5b. Build check (justfile/Makefile/package.json/CLAUDE.md). Fix until clean.
  6. `bd merge-slot acquire --wait` → git add/commit/push → `bd merge-slot release`
  7. `bd close <id>` → `bd agent heartbeat`
  8. → step 1. Empty → `bd agent state worker-<n> done` + report

  ## Fix Loop
  Stay idle after completion. Lead sends failures:
  1. Read failure + paths → fix
  2. `bd merge-slot acquire --wait` → commit/push → release
  3. Report → idle

  ## Context
  Branch: luan/<description> (already created). `bd prime` for context.

  ## Coordination
  Need interface from teammate → MESSAGE them
  Discover issue → MESSAGE team

  ## File Boundaries (HARD RULE)
  NEVER edit files outside your task ownership.
  Need a change in another worker's file:
  1. MESSAGE the file owner
  2. Owner idle → MESSAGE team lead
  3. Never edit directly — even "just one line"
  Violation = task failure.

  ## Output
  Report: tasks completed, files changed, unresolved interface questions
  """
```

9. Review plans. Verify no file ownership overlap, compatible interfaces. Conflicts → add deps or message.

10. Monitor: `bd swarm status <epic-id>`. Watch:
    - Overlapping file claims → pause + add dep
    - Interface mismatches → nudge coordination
    - Discovered issues → fix or beads bug
    - Done: Ready: 0, Active: 0

11. Verify-fix loop (max 2 cycles):
    - All closed → full test suite (workers idle)
    - **Green** → step 12
    - **Red** → match errors to owners via `bd children` + `bd show`, message workers w/ failure output, re-run
    - 2 failed → escalate to user

12. Shut down teammates + clean up (only after green or user approval)

13. Invoke `finishing-branch` skill

## File Ownership

Per-task in beads metadata, not per-teammate.
- `bd update <id> --claim` = atomic (assignee + in_progress, fails if claimed)
- Shared files → one task owns, others MESSAGE owner
- Two ready tasks share files → lead adds dep

## Key Rules

- **Sonnet** workers (follow detailed plans). Opus only if task requires cross-system reasoning.
- **Plan approval** — verify no concurrent file ownership overlaps
- **Self-selecting** — `bd ready --parent <epic-id> --unassigned`
- **Atomic claims** — `--claim` fails if claimed
- **TDD** — failing test first
- **Task atomicity** — finish task before stopping
- **Merge serialization** — `bd merge-slot acquire --wait` then `release`
- **Agent tracking** — `bd agent state` + `bd agent heartbeat`
- **Swarm = truth** — `bd swarm status`, not manual polling
- **Beads = truth** — close beads tasks, not just team tasks
- **Verify before shutdown** — test suite while idle, failures → owners
- **Fix cycles capped** — max 2 → escalate
- **Always clean up** after green or user approval
