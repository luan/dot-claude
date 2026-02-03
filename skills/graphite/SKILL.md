---
name: graphite
description: Graphite stacked PRs - create, sync, modify, submit. Auto-triggers on gt commands, "stack", "branch", "PR", "push", "submit".
user-invocable: true
allowed-tools:
  - "Bash(gt *)"
  - "Bash(git status)"
---

# Graphite

## Quick Reference

```bash
# Create
gt create -am "msg"         # New branch with staged changes
gt c -am "msg"              # Short form

# Navigate
gt up / gt down             # Move through stack
gt top / gt bottom          # Jump to ends
gt log                      # View stack

# Modify
gt modify -a                # Amend changes to current branch
gt squash                   # Squash commits in current branch
gt absorb -a                # Auto-distribute changes to downstack

# Sync
gt sync                     # Pull trunk, restack (NO PUSH)
gt restack                  # Rebase onto updated parents

# Submit
gt submit                   # Push + create/update PRs
gt ss                       # Push entire stack
gt ss -u                    # Update existing PRs only

# Advanced
gt fold                     # Merge into parent
gt move --onto X            # Rebase onto branch X
gt delete                   # Delete from stack

# Recovery
gt continue / gt abort / gt undo
```

## Stack Structure

```
main (trunk)
  └── feature-1  ← BOTTOM (toward main)
        └── feature-2
              └── feature-3  ← TOP (away from main)
```

- **up** = toward top (children)
- **down** = toward bottom (parent/main)

## Common Workflows

| Task | Commands |
|------|----------|
| Start new work | `gt create -am "msg"` |
| Add to stack | `gt create -am "msg"` |
| Push changes | `gt ss` (or `gt ss -u` for existing PRs) |
| Update from main | `gt sync` |
| Amend current | `gt modify -a` |
| View stack | `gt log` |

## Forbidden Git Commands

Never use these on stacked branches:

| Forbidden | Use Instead |
|-----------|-------------|
| `git rebase` | `gt restack` |
| `git push --force` | `gt submit` |
| `git branch -d` | `gt delete` |
| `git checkout -b` | `gt create` |
