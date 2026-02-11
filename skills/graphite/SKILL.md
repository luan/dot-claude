---
name: graphite
description: Graphite stacked PRs - create, sync, modify, submit. Auto-triggers on gt commands, "stack", "branch", "PR", "push", "submit".
user-invocable: true
allowed-tools:
  - "Bash(gt *)"
  - "Bash(git status)"
  - "Bash(bd sync)"
---

# Graphite

## Quick Reference

```bash
# Create
gt create -am "msg"         # New branch + staged changes
gt c -am "msg"              # Short form

# Navigate
gt up / gt down             # Move through stack
gt top / gt bottom          # Jump to ends
gt log --stack              # View CURRENT stack only (use this by default)
gt log                      # View ALL branches (not just current stack)

# Modify
gt modify -a                # Amend changes to current branch
gt squash                   # Squash commits in current branch
gt absorb -a                # Auto-distribute changes downstack

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

## When to Stack

Split when changes are large or multi-concern. Each PR: small, self-contained, buildable.

**Good split candidates:**
- Utility/support code for feature
- New types, models, protocols feature depends on
- Refactoring to make room (extractions, renames)
- Tests for existing behavior before modifying
- Schema/migration separate from code using them
- API/interface definitions separate from implementations
- UI scaffolding separate from business logic
- Cleanup/tech debt discovered during implementation

**Principles:**
- Each PR independently reviewable — no cross-PR context needed
- Each PR compiles + passes tests — no "fixed in next PR"
- Lower-risk/mechanical changes first, riskier logic later
- Target 100-300 lines per PR

**Before splitting:** present stack table (PR title + summary each), get user feedback first.

Don't over-split. Related + small together → one PR.

## Hunk-Level Splitting

When file has changes for different PRs, use `/git-surgeon`:

```bash
git-surgeon hunks                              # list with IDs
git-surgeon commit <id1> <id2> -m "message"    # stage + commit hunks
git-surgeon commit <id>:5-30 -m "message"      # partial hunk by lines
```

Then `gt create -m "msg"` picks up commit into new stack branch.

## Common Workflows

| Task | Commands |
|------|----------|
| Start new work | `gt create -am "msg"` |
| Add to stack | `gt create -am "msg"` |
| Push changes | `gt ss` (or `gt ss -u` for existing) |
| Update from main | `gt sync` |
| Amend current | `gt modify -a` |
| View current stack | `gt log --stack` |
| View all branches | `gt log` |

## Beads Sync

After state-changing gt operations, sync beads:

| Operation | Sync? |
|-----------|-------|
| `gt create` | Yes |
| `gt submit` / `gt ss` | Yes |
| `gt sync` | Yes |
| `gt restack` | Yes |
| `gt modify` / `gt squash` | No |
| `gt log` / navigation | No |

After each state-changing command:
```bash
bd sync 2>/dev/null || true
```

## Forbidden Git Commands

Never use on stacked branches:

| Forbidden | Use Instead |
|-----------|-------------|
| `git rebase` | `gt restack` |
| `git push --force` | `gt submit` |
| `git branch -d` | `gt delete` |
| `git checkout -b` | `gt create` |
