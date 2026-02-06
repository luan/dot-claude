---
name: review-and-fix
description: Review existing changes, identify issues, fix with subagent pattern. Start from commits/files instead of plan.
argument-hint: "[base..head | file-list | PR#] [--against <issue-id>]"
user-invocable: true
allowed-tools:
  - Task
  - Skill
  - AskUserQuestion
  - Read
  - Bash
---

# Review and Fix

Review existing changes → identify issues → fix with subagent pattern.

## Step 0: Triage — Auto-Escalate

- Run `git diff --stat $ARGUMENTS` (or `gh pr diff $ARGUMENTS --stat`)
- Count: files changed, unique top-level directories touched
- If **5+ files across 3+ directories**, OR diff touches auth/security/crypto patterns → invoke `Skill tool: team-review` with the same arguments and STOP
- Otherwise continue below

## Step 1: Determine Scope

Parse $ARGUMENTS for:
- `--against <issue-id>`: beads issue to compare against (plan adherence)
- Remaining args: one of:

| Input | How to get diff |
|-------|----------------|
| (none) | `git diff HEAD` (uncommitted) |
| `main..HEAD` | `git diff main..HEAD` |
| file list | Read those files + `git diff HEAD -- <files>` |
| `#123` | `gh pr diff 123` |

## Step 2: Gather Context

1. Get the diff
2. If `--against`: run `bd show <issue-id>` to get plan from notes
3. Get list of changed files
4. Read all changed files in parallel (full context, not just diff)

## Step 3: Dispatch Reviewer via Task

Spawn a general-purpose agent via Task with this prompt:

```
Review this code change.

## Diff

{paste full diff here}

## Changed Files (full content)

{paste full file contents here}

## Plan to Compare Against

{if --against: paste beads issue notes here}
{if no plan: "No plan provided. Quality review only."}

## Review Criteria

### Plan Adherence (if plan provided)
- Does implementation match what was planned?
- Missing planned features?
- Unplanned additions?
- Deviations justified?

### Architecture & Design
- Follows existing codebase patterns?
- Complexity justified?
- Abstractions at right level?
- Could be simpler?

### Code Quality
- Readable, maintainable?
- Edge cases handled?
- Clear naming?
- Functions focused on one thing?

### Standards
- Follows project style?
- Comments explain "why" not "what"?
- Consistent with codebase?

### Security & Performance
- Obvious security concerns?
- Input validated at boundaries?
- Performance bottlenecks?

## Output Format

Return structured findings:

| Severity | File:Line | Issue | Suggestion |
|----------|-----------|-------|------------|
| Critical | path:N | ... | ... |
| High | path:N | ... | ... |
| Medium | path:N | ... | ... |
| Low | path:N | ... | ... |

Then a brief summary: what's good, what needs work, overall assessment.
```

## Step 4: Present & Ask

Show review summary to user.

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Return findings to caller. Don't fix." || echo "Use AskUserQuestion: Fix all / Fix critical only / Skip fixes"`

## Step 5: Dispatch Fixes

Spawn a general-purpose agent via Task with this prompt:

```
Fix these review issues in the code.

## Issues to Fix

{list of issues with file:line references from review}

## Context

{what the code is supposed to do}

## Your Job

1. Fix each issue listed
2. Verify fixes (syntax check, tests if quick)
3. Report what you fixed

Do NOT:
- Fix things not listed
- Refactor beyond what's needed
- Add features
```

## Step 6: Re-review

After fixes applied, re-run Step 3 to verify. Loop until clean or user stops.

## Skill Composition

| When | Invoke |
|------|--------|
| Complex bug found | `use Skill tool to invoke debugging` |
| Before claiming done | `use Skill tool to invoke verification-before-completion` |
| User has feedback | `use Skill tool to invoke feedback` |
