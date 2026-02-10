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
5. Create beads: epic + ALL child tasks with full code (writing-plans skill)
6. `bd label add <id> architecture|implementation|testing`
7. `bd lint` on epic + children (catches post-update gaps)
8. Verify: `bd children <epic-id>` — every task has test + impl code in description
9. Return plan summary (format below)

## CRITICAL: Beads ARE the plan
Create ALL beads with complete code BEFORE returning.
Plan summary = view of beads, NOT a standalone plan.
No beads → implement pre-flight fails → wasted session.
Missing info → AskUserQuestion. Never "create tickets later".

## Plan Summary Format (return EXACTLY this)

Epic: <epic-id> — <title>
Problem: <1 sentence>
Solution: <1 sentence>

Tasks:
1. <title> (<bd-xxx>) — ready
2. <title> (<bd-yyy>) — blocked by #1

Key decisions:
- <why this approach>

To continue: use Skill tool to invoke `implement` with arg `<epic-id>`

## Task Quality
Each task description must contain:
- Complete copy-pasteable test + implementation code
- Exact file paths
- Exact commands with expected output
Implement copies code EXACTLY from descriptions. Missing code = failed task.

## Escalation
ANY → STOP, return "ESCALATE: team-explore — [reason]" + findings. No epic.
- 3+ viable approaches, unclear tradeoffs
- Spans 3+ independent subsystems
- Cross-cutting concerns needing adversarial analysis
- Architecture decision with diverging perspectives

## Chemistry
Ephemeral exploration. Epic+tasks persist, exploration doesn't.
```

3. Check subagent response:
   - "ESCALATE: team-explore" → `Skill tool: team-explore` with args + findings
   - No epic-id in response → `AskUserQuestion` (exploration failed)
   - Otherwise → output plan summary verbatim, then `ExitPlanMode`
4. After approval, output exactly:
   ```
   To continue: use Skill tool to invoke implement with arg <epic-id>
   ```
   Never Task tool directly.

## Key Rules

- Main thread does NOT explore — subagent does
- **Beads MUST exist before ExitPlanMode** — no standalone plan text
- `bd lint` REQUIRED
- Auto-escalation — ESCALATE → invoke team-explore immediately
- Chemistry: `bd mol squash <id>` on approval, `bd mol burn <id> --force` on rejection
