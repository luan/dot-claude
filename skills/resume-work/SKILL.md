---
name: resume-work
description: "Resume work on a branch/PR after a break. Triggers: 'resume-work', 'resume', 'where was I', 'continue working'."
argument-hint: "[branch-name|PR#]"
user-invocable: true
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - AskUserQuestion
---

# Resume Work

Context recovery after break. Gathers state, suggests next action.

## Context

Branch: !`git branch --show-current`
Status: !`git status -sb 2>/dev/null`
Recent commits: !`git log --oneline -5 2>/dev/null`
Beads in progress: !`bd list --status in_progress -q 2>/dev/null`
Beads ready: !`bd ready -q 2>/dev/null`

## Steps

1. **Resolve branch:**
   - No args → use current branch (above)
   - PR# → `gh pr view <num> --json headRefName`, checkout
   - Branch name → checkout

2. **Gather PR context** (if PR exists):
   - `gh pr view --json title,state,isDraft,reviewDecision,statusCheckRollup`
   - `gh pr checks`
   - PR review comments: `gh api repos/{owner}/{repo}/pulls/{num}/comments`

3. **Check for stale branches:**
   - List all local luan/* branches: `git branch --list 'luan/*' --format='%(refname:short)'`
   - Get in_progress beads: `bd list --status in_progress`
   - Cross-reference: branches without matching in_progress beads are potentially stale
   - Never auto-delete — only surface for user awareness

4. **Summarize:**
   ```
   Branch: <name>
   Commits: last 3 messages
   PR: #N (draft/ready) — title
   Review: Approved | Changes requested | Pending
   CI: Passing | Failing (list failures)
   Comments: N unresolved (summarize)
   Beads: N in progress, M ready
   Stale branches: <list> (no matching in_progress beads)
   ```
   Only show "Stale branches" line if stale branches exist.

5. **Suggest next action (priority order):**
   1. CI failing → "Fix checks"
   2. Changes requested → "Address N review comments"
   3. Unresolved comments → "Respond to feedback"
   4. Beads in progress → "Continue: ..."
   5. Draft PR, all passing → "Mark ready"
   6. Ready PR, approved → "Merge"
   7. No PR → "Create with /commit then /graphite"
   8. All clear → "Wait for review"
