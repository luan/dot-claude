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

Multi-perspective exploration via agent team. Teammates investigate different angles and challenge each other's findings.

## When to Use (vs `explore`)

- 3+ subsystems involved with unclear tradeoffs
- Competing architectural approaches
- Cross-cutting concerns needing adversarial analysis
- Otherwise → use regular `explore`

## Instructions

1. Use `EnterPlanMode` tool
2. Tell Claude to create an agent team:

```
Create an agent team to explore: $ARGUMENTS

Spawn 2-3 teammates, each investigating a different angle.
Use Opus for each teammate. Read and include behavioral guidelines from `agents/`:
- `researcher.md` for breadth-first investigators
- `architect.md` for design/tradeoff analysts
- `devil.md` for contrarian challengers
Require plan approval before teammates begin investigating.

Each teammate should:
- Focus on their assigned perspective
- Message other teammates to share findings and challenge conclusions
- When disagreeing with another teammate, explain why with evidence

After teammates finish:
1. Synthesize findings into a unified plan
2. Create beads epic + tasks: `bd create --type epic --validate`
3. Run `bd lint` on ALL issues
4. Clean up the team
5. Report: epic-id, task count, key files, tradeoffs resolved
```

3. When team finishes, use `ExitPlanMode`
4. After approval: `To continue: use Skill tool to invoke implement with arg <epic-id>`

## Key Rules

- **Main thread does NOT explore** — team does
- **Opus** for teammates (nuanced cross-system analysis needs depth). Use Sonnet for narrow-scope explorations.
- **Plan approval required** — lead approves teammate plans before they investigate
- **bd lint is REQUIRED** — not optional
- **Always clean up team** when done
- **Beads = source of truth** — team task list is ephemeral scratchpad
