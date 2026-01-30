---
name: stack-ops
description: Advanced stack operations - fold, move, split, reorder, delete (gt fold, gt move, gt split)
user-invocable: true
allowed-tools:
  - "Bash(gt fold:*)"
  - "Bash(gt move:*)"
  - "Bash(gt split:*)"
  - "Bash(gt reorder:*)"
  - "Bash(gt delete:*)"
  - "Bash(gt log:*)"
  - "Bash(git status)"
---

# Stack Operations

Advanced stack manipulation commands.

## Commands

```bash
gt fold             # Merge current into parent
gt fold --keep      # Merge but keep current branch name
gt move --onto X    # Rebase current onto branch X
gt split            # Split branch into multiple
gt reorder          # Interactive reorder via editor
gt delete           # Delete current branch from stack
```

## Recovery

```bash
gt continue    # Resume after conflict resolution
gt abort       # Cancel current operation
gt undo        # Undo last Graphite mutation
```

## Forbidden Operations

**NEVER do these on stacked branches:**

| Forbidden | Why | Use Instead |
|-----------|-----|-------------|
| `git rebase` | Breaks tracking | `gt restack` |
| `git push --force` | Desync risk | `gt submit` |
| `git branch -d/-D` | Breaks stack | `gt delete` |
| `git merge` | History issues | `gt sync` |
| `git checkout -b` | Untracked | `gt create` |
