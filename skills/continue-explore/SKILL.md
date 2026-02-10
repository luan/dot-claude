---
name: continue-explore
description: "Refine existing plan with feedback. Triggers: 'refine the plan', 'try again', 'reconsider', 'keep improving', 'that's not right', 'change the approach'"
argument-hint: "[epic-id] <feedback>"
user-invocable: true
allowed-tools:
  - Task
  - EnterPlanMode
  - ExitPlanMode
  - AskUserQuestion
---

# Continue Explore

Iterate on an existing beads epic based on user feedback.

**IMMEDIATELY dispatch to subagent.** Do not explore on main thread.

## Instructions

1. Use `EnterPlanMode` tool
2. Dispatch refinement to subagent:

```
Task tool with subagent_type="general-purpose" and prompt:
"""
Refine an existing plan based on user feedback.

## Parse Arguments

$ARGUMENTS contains:
- Optional epic-id (looks like a beads ID)
- Feedback text (required)

If no epic-id: run `bd list --status in_progress --type epic` and use most recent.
If no epic found: exit with "No active epic. Use /explore to create one."
If no feedback text: exit with "Please provide feedback. Example: /continue-explore approach B looks better"

## Load Existing Plan

1. `bd show <epic-id>` — read full epic with notes and tasks
2. `bd list --parent <epic-id>` — list all child tasks
3. For each task: `bd show <task-id>` to get full details

## Incorporate Feedback

Understand what the user wants:
- Different approach? → redesign tasks
- More detail? → expand specific tasks
- Answer to open question? → update plan accordingly
- Wrong direction? → rethink from findings

## Investigate If Needed

If feedback requires new context:
- Use Glob/Grep/Read to gather info
- Trace code paths the original exploration missed
- Update understanding before changing plan

## Update Plan

1. Update epic notes with revised approach: `bd update <epic-id> --notes "..."`
2. Delete tasks that no longer apply: `bd delete <task-id>`
3. Create new tasks as needed: `bd create --parent <epic-id> --validate`
4. Update existing tasks that need changes: `bd update <task-id>`
5. Final `bd lint` on updated issues (hook handles post-create)

## Verify Before Returning
`bd children <epic-id>` — every task has test + impl code in description.
No code in descriptions → implement pre-flight fails → wasted session.

## Return Value (EXACT format)

Epic: <epic-id> — <title>
Changed: <what changed — 2-3 sentences>

Tasks:
1. <title> (<bd-xxx>) — ready
2. <title> (<bd-yyy>) — blocked by #1

Open questions (if any):
- <question>

To continue: use Skill tool to invoke `implement` with arg `<epic-id>`
"""
```

3. Use `ExitPlanMode`
4. After approval, output: `To continue: use Skill tool to invoke implement with arg <epic-id>`

## Key Rules

- **Main thread does NOT explore** — subagent does
- **bd lint** — hook auto-runs after create; manual only for final validation
- **Preserve what's valuable** — don't delete findings, update them
- **Chemistry:** same epic, tasks evolve
