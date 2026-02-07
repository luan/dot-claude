---
name: team-review
description: "Triggers: 'team review', 'adversarial review', 'deep review'. Multi-lens adversarial code review using agent teams."
argument-hint: "[base..head | file-list | PR-number]"
user-invocable: true
allowed-tools:
  - Teammate
  - Task
  - SendMessage
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
  - AskUserQuestion
  - Read
  - Glob
  - Grep
  - Bash
---

# Team Review

Adversarial multi-lens code review via agent team. Reviewers
cross-examine each other's findings.

## When to Use (vs `review`)

- Security-sensitive changes
- Performance-critical paths
- 5+ files across subsystems
- Otherwise → regular `review`

## Instructions

1. Parse input (commit range, files, or PR number)
2. Create agent team:

```
Review: $ARGUMENTS

Spawn 4 Opus teammates. Read + include behavioral guidelines from `agents/`:
- `reviewer.md` for security + quality reviewers
- `architect.md` for performance reviewer
- `devil.md` for spec/test reviewer (contrarian edge-case focus)
Require plan approval before reviewing.

Teammates:
1. **Security**: auth, injection, data exposure, secrets
2. **Performance**: complexity, memory, concurrency, caching, I/O
3. **Spec/test**: compliance, coverage, edge cases, error handling
4. **Quality**: naming, readability, structure, duplication, idioms

Each reviewer:
- State approach (approval required)
- Review diff through specific lens
- Message others to challenge ("You flagged X, mitigated by Y")
- Classify: critical / high / medium / low

After reviewers finish:
1. Synthesize consensus, severity-ranked
2. Present summary table
3. Clean up team
```

3. AskUserQuestion: fix scope (critical only / critical+high / all / user picks)
4. Dispatch fixes via regular subagent (NOT team)
5. Re-review → loop until approved or user stops

## Key Rules

- **Opus** for teammates (security/perf review needs depth)
- **Plan approval required** — reviewers state approach, lead approves
- **Fixes use subagents, not teams** — no discussion needed
- **Always clean up team** when done
