---
name: save-state
description: Save session context to ~/.claude/.agents/sessions/{branch}.md
argument-hint: [summary]
---

# Save State

Save work context for later resumption.

## Steps

1. `git branch --show-current` + `pwd`
2. Gather git:
   - `git status --porcelain | wc -l`
   - `git log main..HEAD --oneline -10`
   - `git diff --name-only HEAD~5..HEAD`
3. Get summary (arg or ask)
4. Get next steps (ask or from active state)
5. Get blockers (ask or "None")
6. Write `~/.claude/.agents/sessions/{branch}.md` (**use compress-prompt** - AI consumption):

```markdown
# Session: {branch}
Updated: {ISO}
Project: {path}

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
