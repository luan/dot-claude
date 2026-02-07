---
name: team-debug
description: "Triggers: 'team debug', 'competing hypotheses'. Competing hypothesis debugging using agent teams."
argument-hint: "<bug description>"
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

# Team Debug

Competing hypothesis debugging via agent team. Each teammate
investigates a different theory, actively disproving others.

## When to Use (vs `debugging`)

- Root cause unclear, multiple plausible explanations
- Flaky test with unclear trigger
- Bug defies single-thread investigation
- Otherwise → regular `debugging`

## Instructions

1. Analyze symptoms, formulate 3-5 hypotheses
2. Create agent team:

```
Debug: $ARGUMENTS

Hypotheses:
1. [hypothesis 1]
2. [hypothesis 2]
3. [hypothesis 3]

Spawn one Opus teammate per hypothesis. Include `agents/researcher.md`
behavioral guidelines. Require plan approval — each states hypothesis
+ investigation plan, lead approves before proceeding.

Each investigator:
- State hypothesis + plan (approval required)
- Gather evidence for/against
- Message teammates to share evidence
- Actively disprove other theories
- Report: evidence, confidence, whether hypothesis survived

After investigation:
1. Identify surviving hypothesis (strongest evidence)
2. Multiple survive → present tradeoff to user
3. Clean up team
4. Report: root cause, evidence, recommended fix
```

3. Dispatch fix via `debugging` skill (regular subagent)

## Oracle Techniques

Compare buggy behavior against known-good baselines:

1. **Git worktree baseline:** Check out known-good commit in worktree,
   run same test, diff output. Use available worktree (`git worktree list`
   — detached HEAD = available). Creating new worktree is expensive —
   **ask user first**. Never `git stash`/`git checkout` on main worktree.
   Return to detached HEAD when done.
2. **Reference implementation:** Use stdlib/mature library as oracle.
   Test against it directly.
3. **Reduced test case:** Minimize input to smallest failing case.
   Known-correct output for minimal input is often obvious.

Oracle results = hard evidence. Use when arguing hypotheses.

## Key Rules

- **Opus** for teammates (root cause needs depth). Sonnet for shallow hypotheses.
- **Plan approval required** — each investigator's plan approved by lead
- **Adversarial** — teammates must disprove each other
- **Fix uses debugging skill**, not team — investigation done, fix straightforward
- **Always clean up team** when done
