---
name: next
description: "Context-aware dispatch: resumes branch work if on a feature branch, otherwise picks the next work item from the board. Triggers: 'next', 'what next', 'pick up next task', 'resume', 'where was I', 'continue working'."
argument-hint: "[branch-name|PR#]"
user-invocable: true
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - AskUserQuestion
  - TaskList
  - TaskGet
---

# Next

Figure out where you are, then either resume in-flight work or pick something new.

## Context

Branch: !`git branch --show-current`
Status: !`git status -sb 2>/dev/null`
Recent: !`git log --oneline -5 2>/dev/null`
Trunk: !`gt parent 2>/dev/null || gt trunk 2>/dev/null || git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's|refs/remotes/||' || echo main`

## Route

- **On trunk, no argument** → Trunk Path
- **On feature branch** → Feature Branch Path
- **Explicit argument** (branch or PR#) → resolve (PR# via `gh pr view <num> --json headRefName`), then Feature Branch Path

---

## Feature Branch Path

### 1. Gather State

Branch, status, commits from Context. If PR exists: `gh pr view --json title,state,isDraft,reviewDecision,statusCheckRollup`, `gh pr checks`, `gh api repos/{owner}/{repo}/pulls/{num}/comments`.

### 2. Stale Branch Check

List local feature branches (`git branch --list '<pattern>/*'` — adapt pattern to repo's naming convention). Cross-reference against in_progress tasks' `metadata.branch`. Unmatched branches are potentially stale. **Caveat:** not all skills set `metadata.branch`, so absence doesn't confirm staleness. Never auto-delete; surface for awareness only.

### 3. Summarize + Suggest

```
Branch: <name>  |  Commits: last 3
PR: #N (draft/ready) — title
Review/CI/Comments: <status>
Tasks: N active, M pending
Stale branches: <list> (if any)
```

Suggest next action (priority order): CI failing → fix checks; changes requested → address comments; unresolved comments → respond; active tasks → continue; draft+passing → mark ready; approved → merge; no PR → /commit then /gt:submit; all clear → wait for review.

---

## Trunk Path

### 1. Prune + Read Board

Run `ct task prune --days 7` silently (note if tasks pruned). TaskList filtered by `metadata.project === repoRoot`. Split: pending, in_progress (without status_detail). Sort by priority (P1→P3), then creation order.

### 2. Pick Top Candidate

Highest-priority unblocked item. Skip: completed, deleted, `status_detail === "review"`. A task is blocked if any ID in its `blockedBy` array has status !== "completed" — check via TaskGet. Prefer in_progress over pending at same priority — resuming started work avoids duplicate effort.

### 3. Classify

Read via TaskGet. Route by signal:
- Type `bug` → `/debugging`
- Title "Needs brainstorm" → `/brainstorm`
- Title "Brainstorm:" or "Scope:" → `/scope` (brainstorm done → scope is next)
- Has design, no children → `/develop`
- Has children or is leaf → `/develop`
- No Approach/Design → `/scope`
- **Default** → `/scope` — cheapest to course-correct from

### 4. Present + Dispatch

AskUserQuestion: recommended action first ("(Recommended)"), 1-2 alternatives, brief rationale. Invoke chosen skill with task ID. "Skip" → next candidate.

## Key Rules

- Never dispatch without showing what you picked and why
- Never skip reading the issue — description drives classification
- No actionable tasks → say so plainly
- This skill discovers work, it doesn't create it
