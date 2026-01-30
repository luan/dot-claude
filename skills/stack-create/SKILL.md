---
name: stack-create
description: Create new branches in stack (gt create)
user-invocable: true
allowed-tools:
  - "Bash(gt create:*)"
  - "Bash(gt c:*)"
  - "Bash(gt log:*)"
  - "Bash(git status)"
---

# Stack Create

Create new branches in the stack.

## Commands

```bash
gt create -am "commit message"     # Create branch with all changes
gt create my-branch                # Create with specific name
gt create -am "msg" --insert       # Insert mid-stack
gt c -am "msg"                     # Short form
```

## Usage

| User Says | Command |
|-----------|---------|
| "add branch to stack" | `gt create -am "msg"` |
| "new stack" / "start fresh" | `gt create -am "msg"` (from trunk) |
| "insert branch" | `gt create -am "msg" --insert` |

**⚠️ NEVER use `git checkout -b`** - creates untracked branches. Always use `gt create`.
