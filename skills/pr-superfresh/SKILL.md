---
name: pr-superfresh
description: "Full PR refresh: sync, restack, fix CI, fix comments, submit. Triggers: 'refresh PR', 'superfresh', 'update PR', 'PR is stale'."
user-invocable: true
disable-model-invocation: true
allowed-tools:
  - "Bash(gt sync:*)"
  - "Bash(gt restack:*)"
  - "Bash(gt ss:*)"
  - "Bash(gt log:*)"
  - "Bash(gh pr view:*)"
  - "Bash(gh pr checks:*)"
  - "Bash(gh run view:*)"
  - "Bash(gh run list:*)"
  - "Bash(python3 ~/.claude/skills/pr-fix-comments/scripts/pr_threads.py:*)"
  - "Bash(git branch --show-current)"
  - "Bash(git status)"
  - "Bash(git add:*)"
  - "Bash(git commit:*)"
  - Read
  - Edit
  - Glob
  - Grep
  - AskUserQuestion
---

# PR Superfresh

Complete refresh: sync → restack → fix GHA → fix comments → submit.

**Safety: Confirms before destructive ops. Uses Graphite for git.**

## Steps

1. **Sync**: `gt sync --force`
   - If conflicts: help resolve, then `gt continue`
   - Show stack: `gt log`

2. **Fix GHA** (if failures):
   - `gh pr checks <PR> --json name,bucket,link`
   - Fetch logs, plan fixes, confirm, apply
   - Commit: `fix: resolve CI failures`

3. **Fix comments** (if unresolved):
   - `python3 ~/.claude/skills/pr-fix-comments/scripts/pr_threads.py fetch --pr <PR>`
   - Plan fixes, confirm, apply
   - Commit: `pr-fix-comments`
   - Post replies, resolve threads

4. **Submit** (ask first): `gt ss --update-only`

## Summary

```
✓ Synced with trunk
✓ Fixed N GHA failures
✓ Fixed M PR comments
✓ Pushed to remote
PR: <URL>
```
