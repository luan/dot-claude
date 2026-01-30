---
name: stack-modify
description: Amend, squash, or absorb changes in stack (gt modify, gt squash, gt absorb)
user-invocable: true
allowed-tools:
  - "Bash(gt modify:*)"
  - "Bash(gt m:*)"
  - "Bash(gt squash:*)"
  - "Bash(gt sq:*)"
  - "Bash(gt absorb:*)"
  - "Bash(gt log:*)"
  - "Bash(git status)"
---

# Stack Modify

Amend, squash, or absorb changes in stack branches.

## Commands

```bash
gt modify -a            # Amend all changes to current branch, alias: gt m -a
gt modify -cam "msg"    # Create new commit instead of amending
gt modify --into        # Amend into a downstack branch
gt squash               # Squash all commits into one, alias: gt sq
gt absorb -a            # Auto-distribute changes to relevant downstack commits
```

## Usage

| User Says | Command |
|-----------|---------|
| "amend changes" | `gt modify -a` |
| "squash commits" | `gt squash` |
| "absorb changes" | `gt absorb -a` |

**Note:** These commands do NOT push. Use `/stack-submit` after modifying.
