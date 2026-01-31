---
name: resume-state
description: Load session context - auto on start if <30 min
argument-hint: [branch] (default: current)
---

# Resume State

Load saved session context.

## Auto-Resume

On session start:
1. `git branch --show-current` → sanitize
2. Check `~/.claude/.agents/sessions/{branch}.md`
3. If exists + Updated <30 min → inject
4. Else → proceed without (manual /resume available)

## Manual

1. Branch: arg or current
2. Read `~/.claude/.agents/sessions/{branch}.md` or error "No saved state"
3. Display:
   ```
   Resuming {branch}
   Updated: {time} ({ago})
   Project: {path}
   Summary: {summary}
   Git: {uncommitted} uncommitted, last: {commit}
   Next: {steps}
   Blockers: {or None}
   ```
4. Check `.agents/active-{branch}.md`:
   ```
   Active implementation: {topic}
   Progress: {done}/{total}
   Run /implement to continue.
   ```

Manual /resume ignores freshness.
