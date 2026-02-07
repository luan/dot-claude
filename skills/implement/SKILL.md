---
name: implement
description: "Triggers: 'invoke implement', 'implement with arg', beads issue/epic ID, 'execute the plan', 'build this', 'code this plan', 'start implementing', 'now implement', 'time to implement', 'ready to implement', 'begin implementation', 'let me implement', 'To continue: use Skill tool to invoke implement'. Extract issue-id from 'with arg X'."
argument-hint: "[epic-id|task-id] [--fresh]"
user-invocable: true
allowed-tools:
  - Task
  - Skill
  - AskUserQuestion
  - Bash
---

# Implement

## Triage — auto-escalate to team-implement if warranted

1. `bd swarm validate <epic-id> --verbose` — check parallelism
2. **max parallelism >= 3** → invoke `Skill tool: team-implement` with epic-id, STOP
3. Otherwise continue below

**IMMEDIATELY dispatch to subagent.** Never implement on main thread.

## Instructions

Dispatch via Task (subagent_type="beads:task-agent"):

```
Implement: $ARGUMENTS

## Job
1. `bd prime` for context
2. **Create feature branch FIRST**: `gt create luan/<short-description>`
3. Find work: `bd ready` or `bd children <epic-id>`
4. Per task:
   - `bd show <task-id>` — read instructions
   - `bd update <task-id> --claim`
   - Copy test code EXACTLY from description
   - Verify test fails
   - Copy implementation EXACTLY from description
   - Verify test passes
   - `bd close <task-id>`
5. Check PR size before next task (stop at ~250 lines)
6. When done or at size limit: invoke finishing-branch skill

## CRITICAL: Task Atomicity
NEVER stop mid-task. Finish current task before any PR ops.

## Side Quests
Found bug? `bd create "Found: ..." --type bug --validate --deps discovered-from:<current-task-id>`
```

## Key Rules

- **Main thread does NOT implement** — subagent does
- **Copy code EXACTLY** from task descriptions
- **Task atomicity** — never stop mid-task
