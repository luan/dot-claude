---
name: stack-sync
description: Sync stack with trunk and restack branches (gt sync, gt restack)
user-invocable: true
allowed-tools:
  - "Bash(gt sync:*)"
  - "Bash(gt restack:*)"
  - "Bash(gt log:*)"
  - "Bash(git status)"
---

# Stack Sync

Sync your stack with trunk and restack branches.

## Commands

```bash
gt sync                 # Pull trunk, delete merged, restack (NO PUSH)
gt sync --force         # Skip confirmations
gt sync --no-restack    # Skip restacking
gt restack              # Rebase branches onto updated parents
```

## Usage

| User Says | Command |
|-----------|---------|
| "sync stack" / "update stack" | `gt sync` |
| "restack" / "rebase stack" | `gt restack` |

**Note:** `gt sync` does NOT push. Use `/stack-submit` to push.

## Recovery

If conflicts occur during restack:
```bash
# Fix conflicts, then:
gt continue    # Resume operation
gt abort       # Cancel operation
```
