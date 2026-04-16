---
name: gh-stack:restack
description: >
  Rebase branches, sync with main/trunk, and resolve merge conflicts using gh-stack.
  REPLACES git rebase — never use it directly. Triggers: 'restack', 'rebase', 'rebase on
  main', 'sync with main', 'update stack', 'resolve conflicts', 'branches out of date'.
user-invocable: true
context: fork
agent: general-purpose
allowed-tools:
  - "Bash(gh stack:*)"
  - "Bash(git add:*)"
  - "Bash(git status)"
  - Read
  - Edit
  - Glob
  - Grep
---

# Restack

Rebase gh-stack branches onto updated parents and resolve merge conflicts.

## Modes

| Mode | Command | When |
|------|---------|------|
| **Full sync** | `gh stack sync` | Default — fetch, rebase, push, sync PR state |
| Rebase only | `gh stack rebase` | User says "rebase" / "restack" without push |
| Upstack only | `gh stack rebase --upstack` | After mid-stack changes |

## Steps

1. **Sync**: `gh stack sync 2>&1`

2. **If clean**: report which branches were rebased. Done.

3. **If conflicts** (exit code 3): run `gh stack rebase` then loop until resolved —
   a. Read each conflicted file **in full** before editing
   b. Identify **all** conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`)
   c. Resolve **every** conflict region in a **single edit**
   d. `git add <file>` each resolved file
   e. `gh stack rebase --continue 2>&1`
   f. If new conflicts appear, repeat from (a)

4. **Push** after rebase: `gh stack push 2>&1`

5. **Report**: list branches rebased, conflicts resolved, any issues.

## Conflict Resolution Rules

- Read the full file first. Count all `<<<<<<<` markers. Resolve all of them in one pass.
- Take the **semantically correct** merge: understand what both sides changed, combine intent.
- After editing, verify no conflict markers remain: `rg -c '<<<<<<<' <file>` before `git add`.
- If a conflict is ambiguous, `gh stack rebase --abort` and report to the user.
