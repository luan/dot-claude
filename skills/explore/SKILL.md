---
name: explore
description: "Triggers: 'explore', 'how does X work', 'understand', 'research', 'plan a feature', 'figure out', 'investigate', 'design', 'architect'"
argument-hint: "<prompt>"
user-invocable: true
allowed-tools:
  - Task
  - Skill
  - EnterPlanMode
  - ExitPlanMode
  - AskUserQuestion
---

# Explore

**IMMEDIATELY dispatch to subagent.** Never explore on main thread.

## Instructions

1. `EnterPlanMode`
2. Dispatch via Task (subagent_type="general-purpose"):

```
Explore + plan: $ARGUMENTS

## Job
1. `bd prime` for context
2. Check existing: `bd list --status in_progress --type epic`
3. Explore codebase
4. Design approach — 2-3 options, choose best
5. `bd create --type epic --validate` + tasks
6. `bd label add <id> architecture|implementation|testing`
7. `bd lint` ALL issues — fix errors
8. Return: epic-id, task count, key files

## Task Quality
Each task must have:
- Complete copy-pasteable test + implementation code
- Exact file paths
- Exact commands with expected output

## Escalation
ANY → STOP, return "ESCALATE: team-explore — [reason]" + findings. No epic.
- 3+ viable approaches, unclear tradeoffs
- Spans 3+ independent subsystems
- Cross-cutting concerns needing adversarial analysis
- Architecture decision with diverging perspectives

## CRITICAL: Tickets ARE deliverable
Create all tickets before returning. Never "create tickets later".
Missing info → AskUserQuestion before returning.

## Chemistry
Ephemeral exploration. Epic+tasks persist, exploration doesn't.
```

3. Check escalation in response:
   - "ESCALATE: team-explore" → `Skill tool: team-explore` with args + findings
   - Otherwise → `ExitPlanMode`
4. After approval, output exactly:
   ```
   To continue: use Skill tool to invoke implement with arg <epic-id>
   ```
   Never Task tool directly.

## Key Rules

- Main thread does NOT explore — subagent does
- `bd lint` REQUIRED
- Auto-escalation — ESCALATE → invoke team-explore immediately
- Chemistry: `bd mol squash <id>` on approval, `bd mol burn <id> --force` on rejection
