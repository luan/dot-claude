---
name: team-explore
description: "Triggers: 'team explore', 'deep explore', 'explore from multiple angles'. Multi-perspective exploration using agent teams."
argument-hint: "<prompt>"
user-invocable: true
allowed-tools:
  - Teammate
  - Task
  - SendMessage
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
  - EnterPlanMode
  - ExitPlanMode
  - AskUserQuestion
  - Read
  - Glob
  - Grep
  - Bash
---

# Team Explore

Multi-perspective exploration via agent team. Teammates investigate
different angles, challenge each other's findings.

## When to Use (vs `explore`)

- 3+ subsystems with unclear tradeoffs
- Competing architectural approaches
- Cross-cutting concerns needing adversarial analysis
- Otherwise → regular `explore`

## Instructions

1. `EnterPlanMode`
2. Create agent team:

```
Explore: $ARGUMENTS

Spawn 2-3 teammates, each investigating different angle.
Read + include behavioral guidelines from `agents/`:
- `researcher.md` for breadth-first investigators (model: "haiku" — reads lots, reports patterns)
- `architect.md` for design/tradeoff analysts (model: "opus" — deep reasoning)
- `devil.md` for contrarian challengers (model: "opus" — adversarial depth)
Require plan approval before investigating.

Each teammate:
- Focus on assigned perspective
- Message others: share findings, challenge conclusions
- Disagree with evidence, not opinion

After teammates finish:
1. Synthesize into unified plan
2. Create beads epic + ALL child tasks with full code (writing-plans skill)
3. `bd lint` on epic + children (hook handles post-create)
4. Verify: `bd children <epic-id>` — every task has test + impl code
5. Clean up team
6. Return plan summary (EXACT format):

Epic: <epic-id> — <title>
Problem: <1 sentence>
Solution: <1 sentence>

Tasks:
1. <title> (<bd-xxx>) — ready
2. <title> (<bd-yyy>) — blocked by #1

Key decisions:
- <why this approach>
- <tradeoffs resolved>

To continue: use Skill tool to invoke `implement` with arg `<epic-id>`
```

3. Team finishes → `ExitPlanMode`
4. After approval: `To continue: use Skill tool to invoke implement with arg <epic-id>`

## Key Rules

- **Main thread does NOT explore** — team does
- **Per-role models**: haiku for researcher (breadth-first reads), opus for architect/devil (reasoning depth). Sonnet for narrow scope.
- **Plan approval required** — lead approves teammate plans first
- **bd lint** — hook auto-runs after create; manual only for final validation
- **Always clean up team** when done
- **Beads = source of truth** — team task list is ephemeral scratchpad
