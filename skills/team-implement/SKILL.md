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

Coordinated implementation via agent team. Teammates self-select tasks from beads queue, coordinate via messaging when interfaces overlap.

## When to Use (vs `implement`)

- `bd swarm validate <epic-id> --verbose` shows max parallelism >= 3
- Shared interfaces between tasks need live coordination
- Otherwise → use regular `implement` (sequential subagent)

## Instructions

1. Read the epic: `bd show <epic-id>` + `bd children <epic-id>`
2. `bd swarm validate <epic-id> --verbose` — check cycles, max parallelism, ready fronts
3. `bd swarm create <epic-id>` — register swarm molecule
4. `bd merge-slot create` — create git serialization lock
5. **Prep file ownership:** Ensure each task has file ownership in beads metadata. If two ready tasks share files, `bd dep add` between them to serialize. Re-validate with `bd swarm validate` after adding deps.
6. Create feature branch: `gt create luan/<short-description>`
7. Create agent team:

```
Teammate tool:
  operation: "spawnTeam"
  team_name: "impl-<short-desc>"
  description: "Implementing <epic summary>"
```

8. Spawn workers. Count = `min(max_parallelism from validate, 4)`. Use Opus. Require plan approval:

```
Task tool (for each):
  subagent_type: "beads:task-agent"
  mode: "plan"
  team_name: "<team-name>"
  name: "worker-<n>"
  prompt: """
  You are worker-<n> on epic <epic-id>.

  ## Work Loop
  1. `bd ready --parent <epic-id> --unassigned` — find next unblocked, unclaimed task
  2. `bd show <id>` — read task instructions
  3. `bd update <id> --claim` — atomic claim (fails if already claimed → go to step 1)
  4. `bd agent state worker-<n> working` — register active state
  5. Respect file ownership metadata on the task — those are YOUR files while in_progress
  6. Write failing test FIRST, then minimal implementation
  7. Git ops (commit/push):
     - `bd merge-slot acquire` — wait for lock
     - git add, commit, push
     - `bd merge-slot release` — free lock
  8. `bd close <id>`
  9. `bd agent heartbeat` — signal progress
  10. Go to step 1. When `bd ready --parent <epic-id> --unassigned` returns nothing:
      - `bd agent state worker-<n> done`
      - Report completion to lead

  ## Context
  - Branch: luan/<description> (already created)
  - Run `bd prime` for workflow context

  ## Coordination Rules
  - If you need an interface/type/API from another teammate's module, MESSAGE them
  - If you discover an issue the exploration missed, MESSAGE the team
  - If another teammate asks about your interface, respond with the exact signature/shape
  - Do NOT edit files outside your current task's ownership unless coordinated with the owning teammate

  ## Output
  Report: tasks completed, files changed, any unresolved interface questions
  """
```

9. Review and approve each teammate's plan. Verify:
   - File ownership metadata on concurrent tasks doesn't overlap
   - Interface assumptions are compatible across teammates
   - If conflicts exist, add beads dependencies to serialize or message teammates to resolve

10. Monitor with `bd swarm status <epic-id>`. Watch for:
    - Two teammates claiming tasks with overlapping files — pause one, add dependency
    - Interface mismatches — nudge teammates to coordinate
    - Discovered issues — decide: fix now or beads bug for later
    - Completion = Ready: 0, Active: 0

11. When swarm complete:
    - Verify all beads tasks closed: `bd children <epic-id>`
    - Run full test suite
    - Shut down teammates, clean up team

12. Invoke `finishing-branch` skill

## File Ownership Protocol

Ownership is per-task, stored in beads metadata — not per-teammate.

- `bd update <id> --claim` = atomic claim (assignee + in_progress, fails if already claimed).
- If two ready tasks share files, lead adds a beads dependency so they serialize.
- Shared files (types, config) owned by one task; other tasks that need changes MESSAGE the owner.
- `--claim` is atomic — no separate verify step needed.

## Key Rules

- **Opus** for teammates (correct code needs depth). Use Sonnet if epic is mechanical/low-risk. Include `agents/implementer.md` behavioral guidelines in worker prompts.
- **Plan approval required** — verify no file ownership overlaps on concurrent tasks
- **Self-selecting** — teammates pull from `bd ready --parent <epic-id> --unassigned`, not pre-assigned
- **Atomic claims** — `bd update --claim` fails if already claimed, no verify needed
- **TDD still applies** — each teammate writes failing test first
- **File ownership is strict** — per-task metadata, not per-teammate
- **Task atomicity** — teammates finish current task before stopping
- **Merge serialization** — `bd merge-slot acquire/release` around all git commit/push ops
- **Agent tracking** — `bd agent state` + `bd agent heartbeat` for visibility
- **Swarm = source of truth** — `bd swarm status` for progress, not manual polling
- **Beads = source of truth** — teammates close beads tasks, not just team tasks
- **Always clean up team** when done
