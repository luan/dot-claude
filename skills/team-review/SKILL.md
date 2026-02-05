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

Adversarial multi-lens code review via agent team. Reviewers cross-examine each other's findings.

## When to Use (vs `review-and-fix`)

- Security-sensitive changes
- Performance-critical paths
- 5+ files across subsystems
- Otherwise → use regular `review-and-fix`

## Instructions

1. Parse input (commit range, files, or PR number)
2. Tell Claude to create an agent team:

```
Create an agent team to review: $ARGUMENTS

Spawn 4 teammates. Use Opus for each teammate.
Require plan approval before teammates begin reviewing.

Teammates:
1. **Security reviewer**: authentication, authorization, injection, data exposure, secrets
2. **Performance reviewer**: algorithmic complexity, memory, concurrency, caching, I/O
3. **Spec/test reviewer**: spec compliance, test coverage, edge cases, error handling
4. **Quality reviewer**: naming, readability, code structure, duplication, idiomatic patterns, maintainability

Each reviewer should:
- State their review approach (plan approval required)
- Review the diff through their specific lens
- Message other reviewers to challenge findings ("You flagged X, but it's mitigated by Y")
- Classify findings by severity: critical / high / medium / low

After reviewers finish:
1. Synthesize consensus findings, severity-ranked
2. Present summary table to user
3. Clean up the team
```

3. Use `AskUserQuestion` to determine fix scope (critical only / critical+high / all / user picks)
4. Dispatch fixes via regular subagent (NOT team — fixes are fire-and-forget)
5. Re-review fixed code → loop until approved or user stops

## Key Rules

- **Opus** for teammates (security/perf review needs depth)
- **Plan approval required** — reviewers state approach, lead approves
- **Fixes use subagents, not teams** — no discussion needed for fixing
- **Always clean up team** when done
