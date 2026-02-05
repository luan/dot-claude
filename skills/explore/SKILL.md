---
name: explore
description: "Triggers: 'explore', 'how does X work', 'understand', 'research', 'plan a feature', 'figure out', 'investigate', 'design', 'architect', 'that's not right', 'try again', 'refine the plan', 'keep improving', 'reconsider'"
argument-hint: "<prompt> or [epic-id] <feedback>"
user-invocable: true
allowed-tools:
  - Task
  - Skill
  - EnterPlanMode
  - ExitPlanMode
  - AskUserQuestion
---

# Explore

**IMMEDIATELY dispatch to subagent.** Do not explore on main thread.

## Instructions

1. Use `EnterPlanMode` tool
2. Dispatch exploration to subagent:

```
Task tool with subagent_type="general-purpose" and prompt:
"""
Explore and create implementation plan for: $ARGUMENTS

## Your Job
1. Run `bd prime` to get workflow context
2. Check existing work: `bd list --status in_progress --type epic`
3. Explore codebase (use Explore subagent if needed)
4. Design approach - identify 2-3 options, choose best
5. Create epic + tasks with `bd create --type epic --validate`
6. Run `bd lint` on ALL issues - fix any errors
7. Return summary: epic-id, task count, key files

## Task Quality Requirements
Each task must have:
- Complete test code (copy-pasteable)
- Complete implementation code (copy-pasteable)
- Exact file paths
- Exact commands with expected output

## Escalation Check
If you discover ANY of these during exploration, STOP and report back with:
"ESCALATE: team-explore — [reason]"
Do NOT create the epic. Just return the escalation signal + your findings so far.

Escalation triggers:
- 3+ viable approaches with unclear tradeoffs
- Feature spans 3+ independent subsystems
- Cross-cutting concerns needing adversarial analysis
- Architecture decision where different perspectives would produce different answers

## Chemistry
This is exploration - ephemeral. Epic+tasks persist, exploration doesn't.
"""
```

3. When subagent returns, check for escalation:
   - If response contains "ESCALATE: team-explore": invoke `Skill tool: team-explore` with the original arguments + subagent findings as context
   - Otherwise: use `ExitPlanMode`
4. After approval, output: `To continue: use Skill tool to invoke implement with arg <epic-id>`

## Key Rules

- **Main thread does NOT explore** - subagent does
- **bd lint is REQUIRED** - not optional
- **Auto-escalation** - if subagent reports ESCALATE, invoke team-explore immediately
- **Chemistry:** `bd mol squash <id>` on approval, `bd mol burn <id> --force` on rejection
