---
name: pr-superfresh
description: Complete PR refresh - sync, restack, fix GHA failures, fix comments, submit
user-invocable: true
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

Complete PR refresh workflow: sync → restack → fix GHA → fix comments → submit.

**⚠️ SAFETY: Confirms before destructive operations. Uses Graphite for all git operations.**

## Workflow

```
┌─────────────────────────────────────────┐
│ 1. Sync & Restack (gt sync)             │
├─────────────────────────────────────────┤
│ 2. Fix GHA Failures (if any)            │
├─────────────────────────────────────────┤
│ 3. Fix PR Comments (if any)             │
├─────────────────────────────────────────┤
│ 4. Submit Stack (gt ss)                 │
└─────────────────────────────────────────┘
```

## Step 1: Sync & Restack

```bash
gt sync --force
```

If conflicts occur, help resolve them:
```bash
# After manual conflict resolution:
gt continue
```

Show stack status after sync:
```bash
gt log
```

## Step 2: Check & Fix GHA Failures

Get PR for current branch:
```bash
gh pr view --json number -q '.number'
```

Check for failed CI:
```bash
gh pr checks <PR> --json name,bucket,link
```

**If failures exist:**
1. Fetch failed logs: `gh run view <RUN_ID> --log-failed`
2. Analyze and plan fixes
3. Use **AskUserQuestion** to confirm fix plan
4. Apply fixes
5. Commit: `git add <files> && git commit -m "fix: resolve CI failures"`

**If no failures:** Skip to Step 3.

## Step 3: Check & Fix PR Comments

Fetch unresolved comments:
```bash
python3 ~/.claude/skills/pr-fix-comments/scripts/pr_threads.py fetch --pr <PR>
```

**If unresolved comments exist:**
1. Display comment list
2. Use **AskUserQuestion**: "Fix these comments?"
3. If yes: plan fixes, confirm, apply
4. Commit: `git add <files> && git commit -m "pr-fix-comments"`
5. Post replies and resolve threads

**If no comments:** Skip to Step 4.

## Step 4: Submit Stack

Use **AskUserQuestion**:
```
Question: "Ready to push the stack?"
Header: "Submit"
Options:
  1. "Submit" - "Push stack and update PRs (gt ss --update-only)"
  2. "Skip" - "Don't push yet"
```

If user confirms:
```bash
gt ss --update-only
```

## Summary Output

At the end, show:
```
## PR Superfresh Complete

✓ Synced with trunk
✓ Fixed N GHA failures
✓ Fixed M PR comments
✓ Pushed to remote

PR: <URL>
```

## Notes

- Each step is optional based on what's needed
- Always confirms before making changes
- Uses `gt ss --update-only` to avoid creating new PRs
- If any step fails, stops and reports the issue
