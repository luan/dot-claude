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
Explore + create implementation plan for: $ARGUMENTS

## Job
1. `bd prime` for workflow context
2. Check existing: `bd list --status in_progress --type epic`
3. Explore codebase (use Explore subagent if needed)
4. Design approach — 2-3 options, choose best
5. Create epic + tasks: `bd create --type epic --validate`
6. Label each task: `bd label add <id> architecture|implementation|testing`
7. `bd lint` ALL issues — fix errors
8. Return: epic-id, task count, key files

## Task Quality
Each task must have:
- Complete test code (copy-pasteable)
- Complete implementation code (copy-pasteable)
- Exact file paths
- Exact commands with expected output

## Escalation Check
If ANY of these → STOP, return "ESCALATE: team-explore — [reason]"
+ findings so far. Do NOT create epic.

Triggers:
- 3+ viable approaches with unclear tradeoffs
- Feature spans 3+ independent subsystems
- Cross-cutting concerns needing adversarial analysis
- Architecture decision where perspectives diverge

## Chemistry
Ephemeral exploration. Epic+tasks persist, exploration doesn't.
```

3. Check for escalation in response:
   - Contains "ESCALATE: team-explore" → invoke `Skill tool: team-explore` with original args + findings
   - Otherwise → `ExitPlanMode`
4. After approval, output exactly:
   ```
   To continue: use Skill tool to invoke implement with arg <epic-id>
   ```
   Never use Task tool directly.

## Key Rules

- **Main thread does NOT explore** — subagent does
- **bd lint REQUIRED** — not optional
- **Auto-escalation** — ESCALATE response → invoke team-explore immediately
- **Chemistry:** `bd mol squash <id>` on approval, `bd mol burn <id> --force` on rejection
