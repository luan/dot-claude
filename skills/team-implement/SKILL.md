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

Coordinated implementation via agent team. Each teammate owns a layer or module, communicates with others to resolve interface mismatches and discovered issues in real time.

## When to Use (vs `implement`)

- Tasks span 3+ independent modules or layers (frontend/backend/tests/infra)
- Shared interfaces between tasks need live coordination
- Epic has 5+ tasks with no dependency chain (parallelizable)
- Otherwise → use regular `implement` (sequential subagent)

## Instructions

1. Read the epic: `bd show <epic-id>` + `bd children <epic-id>`
2. Group tasks by module/layer — each group becomes a teammate's workload
3. Create feature branch: `gt create luan/<short-description>`
4. Create agent team:

```
Teammate tool:
  operation: "spawnTeam"
  team_name: "impl-<short-desc>"
  description: "Implementing <epic summary>"
```

5. Spawn one teammate per module/layer. Use Sonnet. Require plan approval:

```
Task tool (for each):
  subagent_type: "beads:task-agent"
  model: "sonnet"
  mode: "plan"
  team_name: "<team-name>"
  name: "<module-name>"
  prompt: """
  You are implementing the <MODULE> portion of epic <epic-id>.

  ## Your Tasks
  <paste full task descriptions — don't make them read files>

  ## Context
  - Branch: luan/<description> (already created)
  - Run `bd prime` for workflow context
  - Run `bd update <task-id> --status in_progress` before starting each task

  ## Rules
  1. Plan your approach (will be reviewed before you start)
  2. Write failing test FIRST, then minimal implementation
  3. If you need an interface/type/API from another teammate's module, MESSAGE them
  4. If you discover an issue the exploration missed, MESSAGE the team — don't just file a bug
  5. If another teammate asks about your interface, respond with the exact signature/shape
  6. `bd close <task-id>` when each task passes
  7. Do NOT edit files outside your module unless coordinated with the owning teammate

  ## File Ownership
  Your files: <list of files/directories this teammate owns>
  Other teammates own other paths. Coordinate before touching shared files.

  ## Output
  Report: tasks completed, files changed, any unresolved interface questions
  """
```

6. Review and approve each teammate's plan. Verify:
   - No file ownership overlaps
   - Interface assumptions are compatible across teammates
   - If conflicts exist, message teammates to resolve before approving

7. Monitor implementation. Watch for:
   - Interface mismatches — nudge teammates to coordinate
   - Discovered issues — decide: fix now or beads bug for later
   - File conflicts — if two teammates need the same file, pause one

8. When teammates finish:
   - Verify all beads tasks closed: `bd children <epic-id>`
   - Run full test suite
   - Check for merge conflicts between teammates' changes
   - Shut down teammates, clean up team

9. Invoke `finishing-branch` skill

## File Ownership Protocol

**Critical:** Two teammates editing the same file = overwrites. Before spawning:
- Map every task to its primary files
- Assign non-overlapping file sets to each teammate
- Shared files (e.g., types, config) → assign to ONE teammate, others request changes via messages

## Key Rules

- **Sonnet** for teammates (implementation is mechanical, coordination is the value-add)
- **Plan approval required** — verify no file overlaps, compatible interfaces
- **TDD still applies** — each teammate writes failing test first
- **File ownership is strict** — teammates don't touch each other's files without messaging
- **Task atomicity** — teammates finish current task before stopping
- **Beads = source of truth** — teammates close beads tasks, not just team tasks
- **Always clean up team** when done
