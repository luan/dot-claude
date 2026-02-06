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

- Tasks span 3+ independent modules or layers (frontend/backend/tests/infra)
- Shared interfaces between tasks need live coordination
- Epic has 5+ tasks with no dependency chain (parallelizable)
- Otherwise → use regular `implement` (sequential subagent)

## Instructions

1. Read the epic: `bd show <epic-id>` + `bd children <epic-id>`
2. **Prep file ownership:** Ensure each task has file ownership in beads metadata. If two ready tasks share files, `bd dep add` between them to serialize. This is the lead's critical prep step.
3. Create feature branch: `gt create luan/<short-description>`
4. Create agent team:

```
Teammate tool:
  operation: "spawnTeam"
  team_name: "impl-<short-desc>"
  description: "Implementing <epic summary>"
```

5. Spawn 2-4 worker teammates. Use Sonnet. Require plan approval:

```
Task tool (for each):
  subagent_type: "beads:task-agent"
  model: "sonnet"
  mode: "plan"
  team_name: "<team-name>"
  name: "worker-<n>"
  prompt: """
  You are a worker on epic <epic-id>.

  ## Work Loop
  1. `bd ready` — find next unblocked task
  2. `bd show <id>` — read task instructions
  3. `bd update <id> --status in_progress` — claim it
  4. `bd show <id>` — verify you're the owner (if someone else claimed it, go to step 1)
  5. Respect file ownership metadata on the task — those are YOUR files while in_progress
  6. Write failing test FIRST, then minimal implementation
  7. `bd close <id>` when done
  8. Go to step 1. When `bd ready` returns nothing, report completion to lead.

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

6. Review and approve each teammate's plan. Verify:
   - File ownership metadata on concurrent tasks doesn't overlap
   - Interface assumptions are compatible across teammates
   - If conflicts exist, add beads dependencies to serialize or message teammates to resolve

7. Monitor implementation. Watch for:
   - Two teammates claiming tasks with overlapping files — pause one, add dependency
   - Interface mismatches — nudge teammates to coordinate
   - Discovered issues — decide: fix now or beads bug for later
   - `bd ready` returning nothing for all teammates = done

8. When teammates finish:
   - Verify all beads tasks closed: `bd children <epic-id>`
   - Run full test suite
   - Shut down teammates, clean up team

9. Invoke `finishing-branch` skill

## File Ownership Protocol

Ownership is per-task, stored in beads metadata — not per-teammate.

- `bd update <id> --status in_progress` = claim. Only one agent works a task at a time.
- If two ready tasks share files, lead adds a beads dependency so they serialize.
- Shared files (types, config) owned by one task; other tasks that need changes MESSAGE the owner.
- Teammates verify ownership after claiming (`bd show` to confirm).

## Key Rules

- **Sonnet** for teammates (implementation is mechanical, coordination is the value-add)
- **Plan approval required** — verify no file ownership overlaps on concurrent tasks
- **Self-selecting** — teammates pull from `bd ready`, not pre-assigned by lead
- **Claim verification** — `bd show` after `bd update --status in_progress` to confirm ownership
- **TDD still applies** — each teammate writes failing test first
- **File ownership is strict** — per-task metadata, not per-teammate
- **Task atomicity** — teammates finish current task before stopping
- **Beads = source of truth** — teammates close beads tasks, not just team tasks
- **Always clean up team** when done
