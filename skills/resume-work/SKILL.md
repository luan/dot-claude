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

Context recovery after a break. Gathers state, suggests next action.

## Steps

1. **Resolve branch:**
   - No args → `git branch --show-current`
   - PR# → `gh pr view <num> --json headRefName`, checkout
   - Branch name → checkout

2. **Gather context (parallel):**
   - `git log --oneline -5`
   - `git status -sb`
   - `gh pr view --json title,state,isDraft,reviewDecision,statusCheckRollup` (if PR exists)
   - `gh pr checks` (if PR exists)
   - PR review comments: `gh api repos/{owner}/{repo}/pulls/{num}/comments`
   - `bd list --status in_progress`
   - `bd ready`

3. **Summarize:**
   ```
   Branch: <name>
   Commits: last 3 messages
   PR: #N (draft/ready) — title
   Review: Approved | Changes requested | Pending
   CI: Passing | Failing (list failures)
   Comments: N unresolved (summarize)
   Beads: N in progress, M ready
   ```

4. **Suggest next action (priority order):**
   1. CI failing → "Fix checks"
   2. Changes requested → "Address N review comments"
   3. Unresolved comments → "Respond to feedback"
   4. Beads in progress → "Continue: ..."
   5. Draft PR, all passing → "Mark ready"
   6. Ready PR, approved → "Merge"
   7. No PR → "Create with /commit then /graphite"
   8. All clear → "Wait for review"
