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

## Triage — auto-escalate

1. `bd swarm validate <epic-id> --verbose`
2. **max parallelism >= 3** → `Skill tool: team-implement` with epic-id, STOP
3. Otherwise continue

**IMMEDIATELY dispatch to subagent.** Never implement on main thread.

## Instructions

Dispatch via Task (subagent_type="beads:task-agent"):

```
Implement: $ARGUMENTS

## Job
1. `bd prime` for context
2. **Pre-flight:** `bd children <epic-id>` — no children or tasks
   lack code in description → STOP, return "explore phase
   incomplete — no implementable tasks". Do NOT create tasks.
3. `gt create luan/<short-description>`
4. `bd ready` or `bd children <epic-id>`
5. Per task:
   - `bd show <task-id>` + `bd update <task-id> --claim`
   - Copy test code EXACTLY → verify fails
   - Copy implementation EXACTLY → verify passes
   - **Build check:** detect build cmd from justfile/Makefile/
     package.json/CLAUDE.md. Fix until clean. No close with errors.
   - `bd close <task-id>`
6. Stop at ~250 lines PR size
7. Done/size limit → invoke finishing-branch skill

## Task Atomicity
NEVER stop mid-task. Finish before any PR ops.

## Side Quests
Bug found? `bd create "Found: ..." --type bug --validate --deps discovered-from:<current-task-id>`
```

## Key Rules

- Main thread does NOT implement — subagent does
- Copy code EXACTLY from task descriptions
- Task atomicity — never stop mid-task
- Pre-flight required — bail if no implementable tasks
