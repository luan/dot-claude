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

Spawn 3 teammates with per-role models. Read + include behavioral guidelines from `agents/`:
- `reviewer.md` for security reviewer
- `architect.md` for architecture/performance reviewer
- `devil.md` for spec/test reviewer (contrarian edge-case focus)
Require plan approval before reviewing.

Teammates:
1. **Security** (model: "opus"): auth, injection, data exposure, secrets — needs depth
2. **Architecture & Performance** (model: "sonnet"): structure, complexity, memory, I/O, concurrency — pattern matching
3. **Spec/test** (model: "opus"): compliance, coverage, edge cases, error handling — adversarial

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

- **Per-role models**: opus for security + spec/test (depth), sonnet for arch/perf (pattern matching)
- **Plan approval required** — reviewers state approach, lead approves
- **Fixes use subagents, not teams** — no discussion needed
- **Always clean up team** when done
