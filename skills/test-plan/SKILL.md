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

## Context

Stat: !`git diff --stat HEAD 2>/dev/null`
Files: !`git diff --name-only HEAD 2>/dev/null`

## Interviewing

See rules/skill-interviewing.md. Ask on: unclear risk classification, ambiguous test environment.

## Step 1: Scope

Use injected Stat/Files above for the no-args case. Override with $ARGUMENTS:

| Input        | Diff source                |
| ------------ | -------------------------- |
| `main..HEAD` | `git diff main..HEAD`      |
| file list    | `git diff HEAD -- <files>` |
| `#123`       | `gh pr diff 123`           |

**Early exit.** ALL files trivial → `## Test Plan: No Manual Testing Required` + stat output.

Trivial: *.md, *.txt, LICENSE, CHANGELOG, comment-only, whitespace-only, CI metadata.
**Never trivial:** SKILL.md, CLAUDE.md, *.mdx — executable specs that change agent behavior. Analyze with code rigor: what behavior changed, what could break, what to verify.

## Step 2: Analyze

Read the full diff. Classify each changed file by risk and type. Include a 1-sentence justification per file referencing the specific change characteristic (e.g., "Critical — modifies auth token validation logic").

### Risk Levels

| Level | Scope | Verification |
|-------|-------|-------------|
| **Critical** | Data loss or security breach risk (auth, persistence, payments, security, infra) | Test first, most thoroughly |
| **High** | User-visible behavior (UI, API contracts, business logic, error handling, perf paths) | Full verification steps |
| **Medium** | Indirect impact (refactors changing control flow, dep updates, logging, build config) | Targeted verification |
| **Low** | Unlikely user-facing (style fixes, adding tests, code comments, dev tooling) | Spot-check only |

Multiple levels apply → use highest. A refactor touching auth logic is Critical, not Medium.

### Change Types

Tag each file: **new-feature**, **behavior-change**, **refactor**, **bugfix**, **config**, **dependency**.

## Step 3: Generate

Group verification steps by risk (highest first). Every component must reference domain-specific terms from the diff (function names, endpoints, error messages) — no generic language. Each step:
1. **What** — specific behavior to verify
2. **How** — concrete reproducible actions (e.g., "call `paginate(page=1, size=10)`", "POST `/api/auth/login` with expired token"). Spec changes: invoke skill with specific trigger/argument, verify behavioral change.
3. **Expected** — observable correct outcome
4. **Regression** — adjacent functionality to confirm

Output structure:
```
## Test Plan: <scope summary>
Risk: N critical, N high, N medium, N low
Effort: quick (5min — single-file, Low/Medium risk) | moderate (15min — multi-file, High risk) | thorough (30min+ — any Critical risk present)

### Critical Risk
<verification steps>

### High Risk
<verification steps>

### Low Risk (spot-check)
<brief list>

### Regression Checklist
- [ ] <adjacent area>
```

Omit empty risk sections. All Low → spot-check list only.

For **refactors**: focus on behavior preservation — same inputs → same outputs.
For **bugfixes**: include original reproduction steps + edge cases around fix boundary.

## Step 4: Output

Present the plan. `$CLAUDE_NON_INTERACTIVE` → output and stop. Otherwise present and stop.
