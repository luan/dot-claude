---
name: agent-tdd
description: "RED→GREEN eval-driven workflow for changing rules and skills. Proves changes work by writing evals BEFORE making changes, confirming the problem exists (RED), then confirming the fix works (GREEN). Use this skill whenever creating or modifying rules in .claude/rules/, CLAUDE.md behavioral lines, or skill SKILL.md files. Also use when the user says 'test this rule', 'does this rule work', 'prove it', 'eval-driven', 'red green', 'write evals first', 'test before changing', or wants evidence that a behavioral change actually has an effect. Prefer this over directly invoking rule-creator or skill-creator for behavioral changes."
user-invocable: true
argument-hint: "<problem description or rule/skill path>"
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - Skill
---

# Agent TDD

Prove behavioral changes work. Every rule or skill change follows: **eval → RED → change → GREEN → regression check**.

The value of this workflow: if you write the change first and the eval after, the eval always passes and you've proven nothing. Writing the eval first and confirming it fails against the current state proves the problem exists. Making it pass proves the change fixed it.

## Arguments

- `<problem description>` — what bad behavior the user is seeing (triggers full flow)
- `<rule/skill path>` — test an existing rule or skill change (skips to RED phase)
- `--continue` — resume from last phase

## Flow

### [1] Problem → Evals

Understand what's going wrong. The user describes a behavior they want to change — Claude does X when it should do Y.

Extract 2-4 eval prompts. Each eval needs:
- A **setup** script that creates a real git repo with real files (the scenario)
- An **action prompt** that tells Claude to DO something (not "what would you do?")
- A **check** that inspects actual repo state after (git log, git diff, file contents)
- **Expectations** describing the desired post-state

Reflective prompts ("what would you do?") are useless for behavioral evals — Claude gives the right answer when asked to reason, but exhibits bad behavior during autonomous execution. The eval must force execution.

```json
{
  "id": "descriptive-name",
  "setup": "git init && create files && git commit",
  "prompt": "action-oriented prompt that triggers the behavior",
  "check": "git log --oneline | wc -l  # did it commit?",
  "expectations": ["post-state description"],
  "rule_id": "target-rule-or-skill"
}
```

Two comparison modes:
- **Bare vs rule**: does bare Claude exhibit the problem? Does the rule fix it?
- **Current config vs fixed config**: does our CURRENT config cause the problem? Does removing/changing it fix it? (Some problems are caused by our own rules, not by Claude's defaults.)

Save evals and present to user for review before proceeding.

### [2] RED — Confirm the problem exists

Run each eval against the current state. The evals should FAIL — that's the point. If they pass, either the problem doesn't exist or the evals don't capture it.

Run evals in a real repo. Two baseline modes depending on the problem:

**Bare baseline** (testing Claude's defaults — is the problem inherent?):
```bash
cd <repo> && \
CLAUDE_CODE_DISABLE_AUTO_MEMORY=1 \
  claude -p \
  --setting-sources "" \
  --permission-mode bypassPermissions \
  --output-format json \
  --max-budget-usd 0.50 \
  "<prompt>"
```
`--setting-sources ""` skips all user/project/local settings. Standard system prompt and tools remain.

**Current-config baseline** (testing if OUR config causes the problem):
```bash
cd <repo> && \
  claude -p \
  --permission-mode bypassPermissions \
  --output-format json \
  --max-budget-usd 0.50 \
  "<prompt>"
```
Loads all current rules, CLAUDE.md, skills. If this fails but bare passes, our config is the problem.

**Run both.** If bare Claude already behaves correctly, the problem is caused by our config — the fix is removal, not addition. If bare Claude also exhibits the problem, a rule might help.

**Grading:** After the run, check actual repo state: `git log --oneline`, `git branch`, `git diff`, file contents. Also parse the JSON output's `result` field for response text. Grade against expectations.

**Gate:** If all evals PASS → the problem doesn't exist in the baseline, or the evals are wrong. Report this and stop. Ask the user to refine the problem description or evals.

If evals FAIL → RED confirmed. Report which evals failed and why. Proceed to Change phase.

### [3] Change — Route to the right tool

Based on what needs to change:

| Target | Action |
|--------|--------|
| Rule file in `.claude/rules/` | `Skill("rule-creator", "--new '<intent>'")` |
| Skill SKILL.md | `Skill("skill-creator", "<skill-path>")` |
| CLAUDE.md line | Direct edit |
| Multiple targets | Handle sequentially |

The change tool does its work — drafting, writing, iterating. Agent-tdd doesn't control HOW the change is made, only that the RED→GREEN sequence is followed.

### [4] GREEN — Confirm the fix works

Re-run the same evals, now with the change applied.

**For rules:**
```bash
CLAUDE_CODE_DISABLE_AUTO_MEMORY=1 \
  claude -p \
  --setting-sources "" \
  --append-system-prompt "<rule text>" \
  --permission-mode bypassPermissions \
  --output-format json \
  --max-budget-usd 0.50 \
  "<prompt>"
```

Same clean baseline, plus `--append-system-prompt` injects ONLY the rule being tested. If the eval passes, it's because of the rule, not because of other configuration.

**For skills:**
```bash
claude -p --permission-mode bypassPermissions "<prompt>"
```

The skill is now modified on disk, so a normal run picks it up.

**For CLAUDE.md:**
```bash
claude -p --permission-mode bypassPermissions "<prompt>"
```

CLAUDE.md is loaded automatically.

**Grading:** Same expectations, same grader. Evals should now PASS.

**Gate:** If evals still FAIL → the change didn't fix the problem. Report which evals still fail and why. The user can iterate (back to Change phase) or adjust evals.

If evals PASS → GREEN confirmed. Proceed to regression check.

### [5] Regression — Check nothing broke

Run existing evals for the affected `rule_id` or skill to confirm the change didn't break other behavior.

For rules: filter `evals.json` entries by `rule_id`, run each via the same `--bare --append-system-prompt` mechanism.

For skills: run the skill's existing `evals/evals.json` suite.

Report any regressions. If regressions found → the change is too broad. User decides whether to adjust the change or accept the tradeoff.

### [6] Report

```
Agent TDD: <target>
RED:   N/M evals failed (problem confirmed)
GREEN: M/M evals pass (fix confirmed)
Regression: 0 failures in K existing evals

Delta: <what changed from RED to GREEN>
```

## Key Principles

- Evals come from the problem, not from the solution. Write them before you know what the fix looks like.
- RED must fail. If baseline already passes, the eval isn't testing the right thing.
- GREEN must pass from the change alone. `--bare` + `--append-system-prompt` isolates the rule's effect.
- Regressions are information, not blockers. Report them and let the user decide.
