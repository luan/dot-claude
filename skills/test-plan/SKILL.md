---
name: test-plan
description: "Analyze current diff, classify changes by risk, and produce structured manual test plan. Triggers: 'test plan', 'what should I test', 'manual testing', 'verification steps', 'QA checklist'. Exits early for trivial changes. Do NOT use when: writing automated tests — use /implement with TDD. Do NOT use when: reviewing code quality — use /review instead."
argument-hint: "[base..head | file-list | PR#]"
user-invocable: true
model: sonnet
context: fork
agent: general-purpose
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - AskUserQuestion
---

# Test Plan

Analyze changes, classify by risk, produce structured manual test plan. Auto-exits for trivial changes.

## Interviewing

See rules/skill-interviewing.md. Skill-specific triggers:
- Risk classification unclear (e.g., config change that might affect runtime) → ask
- Testing environment ambiguous (e.g., requires specific device/OS) → clarify setup

## Step 1: Scope

Parse $ARGUMENTS:

| Input        | Diff source                |
| ------------ | -------------------------- |
| (none)       | `git diff HEAD`            |
| `main..HEAD` | `git diff main..HEAD`      |
| file list    | `git diff HEAD -- <files>` |
| `#123`       | `gh pr diff 123`           |

Collect diff and `git diff --stat`.

**Early exit.** If ALL changed files are trivial, output:
```
## Test Plan: No Manual Testing Required
Changes are trivial. No functional impact.
Changed files: <--stat output>
```

Trivial: *.md, *.txt, LICENSE, CHANGELOG, comment-only, whitespace-only, CI metadata (labels, assignees).
**Never trivial:** SKILL.md, CLAUDE.md, *.mdx — these are executable specs that change agent behavior (a typo in a trigger phrase can break skill discovery). Analyze them with the same rigor as code: what behavior changed, what could break, what to verify.

## Step 2: Analyze

Read the full diff. Classify each changed file by risk and type.

### Risk Levels

| Level | Scope | Verification |
|-------|-------|-------------|
| **Critical** | Data loss or security breach risk (auth, persistence, payments, security, infra) | Test first, most thoroughly |
| **High** | User-visible behavior (UI, API contracts, business logic, error handling, perf paths) | Full verification steps |
| **Medium** | Indirect impact (refactors changing control flow, dep updates, logging, build config) | Targeted verification |
| **Low** | Unlikely user-facing (style fixes, adding tests, code comments, dev tooling) | Spot-check only |

When multiple risk levels apply, use the highest — under-testing a critical path is far costlier than over-testing a minor change. A refactor touching auth logic is Critical, not Medium.

### Change Types

Tag each file: **new-feature**, **behavior-change**, **refactor**, **bugfix**, **config**, **dependency**.

## Step 3: Generate

Group verification steps by risk (highest first). Each step:
1. **What** — specific behavior to verify
2. **How** — exact steps
3. **Expected** — observable correct outcome
4. **Regression** — adjacent functionality to confirm

Output structure:
```
## Test Plan: <scope summary>
Risk: N critical, N high, N medium, N low
Effort: quick (5min) | moderate (15min) | thorough (30min+)

### Critical Risk
<verification steps>

### High Risk
<verification steps>

### Low Risk (spot-check)
<brief list>

### Regression Checklist
- [ ] <adjacent area>
```

Omit empty risk sections — only include levels with actual changes. If all changes are Low, the plan is just a spot-check list.

For **refactors**: focus on behavior preservation — same inputs → same outputs.
For **bugfixes**: include original reproduction steps + edge cases around fix boundary.

## Step 4: Output

Present the plan. If `$CLAUDE_NON_INTERACTIVE` is set, output and stop.
Otherwise present and stop — user will request refinements if needed.
