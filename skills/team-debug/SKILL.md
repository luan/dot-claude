---
name: team-debug
description: "Triggers: 'team debug', 'competing hypotheses'. Competing hypothesis debugging using agent teams."
argument-hint: "<bug description>"
user-invocable: true
allowed-tools:
  - Task
  - AskUserQuestion
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

## Key Rules

- **Sonnet** for teammates (hypothesis testing is exploratory)
- **Plan approval required** — each teammate's investigation plan approved by lead
- **Adversarial** — teammates must try to disprove each other
- **Fix uses debugging skill**, not team — investigation done, fix is straightforward
- **Always clean up team** when done
