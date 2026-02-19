---
name: gt:restack
description: >
  Restack Graphite stack and resolve merge conflicts. Triggers: 'restack',
  'rebase stack', 'update stack', 'resolve conflicts'.
user-invocable: true
context: fork
agent: general-purpose
allowed-tools:
  - "Bash(gt:*)"
  - "Bash(git add:*)"
  - "Bash(git status)"
  - Read
  - Edit
  - Glob
  - Grep
---

# Restack

Rebase Graphite stack onto updated parents and resolve any merge conflicts.

## Steps

1. **Restack**: `gt restack 2>&1`

2. **If clean**: report which branches were restacked. Done.

3. **If conflicts**: loop until resolved —
   a. Read each conflicted file **in full** before editing b. Identify **all** conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`) in the file c. Resolve **every** conflict region in a **single edit** — never leave partial markers d. `git add <file>` each resolved file
   e. `gt continue 2>&1`
   f. If new conflicts appear (next branch in stack), repeat from (a)

4. **Report**: list branches restacked, conflicts resolved, any issues.

## Conflict Resolution Rules

- Read the full file first. Count all `<<<<<<<` markers. Resolve all of them in one pass.
- Take the **semantically correct** merge: understand what both sides changed, combine intent.
- For renames/refactors: apply the rename to the newer code from the child branch.
- After editing, verify no conflict markers remain: `grep -c '<<<<<<<' <file>` before `git add`.
- If a conflict is ambiguous and you can't determine the right resolution, `gt abort` and report to the user.
