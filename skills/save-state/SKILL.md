---
name: save-state
description: "Save session context to ~/.claude/.agents/sessions/{branch}.md"
argument-hint: "[summary]"
---

# Save State

Current branch: !`git branch --show-current | tr '/' '-'`
Current path: !`pwd`
Status lines: !`git status --porcelain | wc -l`
Recent commits: !`git log main..HEAD --oneline -10`
Modified files: !`git diff --name-only HEAD~5..HEAD`

Save work context for later resumption.

## Steps

1. Get summary (arg or ask)
2. Get next steps (ask or from active state)
3. Get blockers (ask or "None")
4. Write `~/.claude/.agents/sessions/{branch}.md` (**use compress-prompt** - AI consumption):

```markdown
# Session: {branch}

Updated: !`date '+%Y-%m-%d %H:%M:%S'`
Project: !`basename $(git rev-parse --show-toplevel)`

## Summary

{summary}

## Git State

Branch: {branch}
Uncommitted: {n} files
Commits: {list}
Modified: {list}

## Next Steps

- ...

## Blockers

{or None}
```

Auto-loaded on session start if <30 min old.
