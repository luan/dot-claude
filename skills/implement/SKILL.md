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

Before dispatching, check epic scope:
1. `bd children <epic-id>` — count tasks
2. If **5+ tasks spanning 3+ modules/layers with no dependency chain** → invoke `Skill tool: team-implement` with the epic-id and STOP
3. Otherwise continue below

**IMMEDIATELY dispatch to subagent.** Do not implement on main thread.

## Instructions

Dispatch implementation to subagent:

```
Task tool with subagent_type="beads:task-agent" and prompt:
"""
Implement: $ARGUMENTS

## Your Job
1. Run `bd prime` for context
2. **Create feature branch FIRST**: `gt create luan/<short-description>` (e.g., `gt create luan/fix-container-minimize`)
3. Find work: `bd ready` or `bd children <epic-id>`
4. For each task:
   - `bd show <task-id>` - read instructions
   - `bd update <task-id> --status in_progress`
   - Copy test code EXACTLY from description
   - Verify test fails
   - Copy implementation EXACTLY from description
   - Verify test passes
   - `bd close <task-id>`
5. Check PR size before starting next task (stop at ~250 lines)
6. When done or at size limit: invoke finishing-branch skill

## CRITICAL: Task Atomicity
NEVER stop mid-task. Finish current task before any PR operations.

## Side Quests
Found bug during impl? `bd create "Found: ..." --type bug` then `bd dep add <current> <new> --type discovered-from`
"""
```

## Key Rules

- **Main thread does NOT implement** - subagent does
- **Copy code EXACTLY** from task descriptions
- **Task atomicity** - never stop mid-task
