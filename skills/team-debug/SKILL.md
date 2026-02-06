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

Competing hypothesis debugging via agent team. Each teammate investigates a different theory, actively trying to disprove the others.

## When to Use (vs `debugging`)

- Root cause unclear, multiple plausible explanations
- Flaky test with unclear trigger
- Bug that defies single-thread investigation
- Otherwise → use regular `debugging`

## Instructions

1. Analyze bug symptoms, formulate 3-5 hypotheses
2. Tell Claude to create an agent team:

```
Create an agent team to debug: $ARGUMENTS

Hypotheses:
1. [hypothesis 1]
2. [hypothesis 2]
3. [hypothesis 3]
...

Spawn one teammate per hypothesis. Use Sonnet for each teammate.
Require plan approval — each teammate states their hypothesis and investigation plan, lead approves before they proceed.

Each investigator should:
- State hypothesis + investigation plan (plan approval required)
- Gather evidence for/against their theory
- Message other teammates to share evidence
- Actively try to disprove other teammates' theories
- Report: evidence found, confidence level, whether hypothesis survived

After investigation:
1. Identify the surviving hypothesis (strongest evidence)
2. If multiple survive, present tradeoff to user
3. Clean up the team
4. Report: root cause, evidence, recommended fix approach
```

3. After root cause identified, dispatch fix via `debugging` skill flow (regular subagent)

## Oracle Techniques

Investigators should compare buggy behavior against known-good baselines as hard evidence:

1. **Git worktree baseline:** Check out a known-good commit in a worktree, run the same test, diff output against current. Use an available worktree from the pool (`git worktree list` — detached HEAD = available, `git checkout <last-green-commit>` inside it). Creating a new worktree is expensive — **always ask the user first**. Never use `git stash`/`git checkout` on the main worktree (risks uncommitted work). Return the worktree to detached HEAD when done.
2. **Reference implementation:** Use stdlib or mature library as oracle. If your function should match `hashlib.sha256()` output, test against it directly.
3. **Reduced test case:** Minimize input to smallest case that still fails. Known-correct output for minimal input is often obvious — use it as ground truth.

Oracle results are hard evidence. Use them when arguing for/against hypotheses.

## Key Rules

- **Sonnet** for teammates (hypothesis testing is exploratory)
- **Plan approval required** — each teammate's investigation plan approved by lead
- **Adversarial** — teammates must try to disprove each other
- **Fix uses debugging skill**, not team — investigation done, fix is straightforward
- **Always clean up team** when done
