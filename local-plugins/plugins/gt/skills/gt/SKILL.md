---
name: gt
description: >
  Use this skill whenever the user wants to push code, create or update PRs, create branches
  on a stack, rebase or sync branches with trunk, move changes between branches, or inspect
  stack status. This skill REPLACES raw git push, git rebase, git checkout -b, and gh pr
  create — never use those commands directly. Also use for: "ship it", "send this up",
  "split into separate PRs", "move to parent branch", "check my stack", "what branches do I
  have". Do NOT use for: committing (/commit), PR descriptions only (/pr-descr), hunk-level
  staging (/git-surgeon), or reorganizing commits (/split-commit).
user-invocable: true
allowed-tools:
  - "Bash(gt:*)"
  - "Bash(git status)"
argument-hint: "[log|restack|sync|info|amend|up|down|top|bottom|submit|create|...] [flags]"
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

## Commit Messages

`gt create -am` messages follow the same conventional commit format as `/commit`:
`type(scope): description` — max 72 chars, lowercase, no period, imperative mood.

To write a good message: analyze the staged diff (or all changes if nothing staged), determine the primary type and scope, describe the WHY not the WHAT.

## Common Workflows

| Task | Commands |
|------|----------|
| Start new work | `gt create -am "feat(auth): add token refresh"` |
| Add to stack | `gt create -am "fix(auth): handle expired tokens"` |
| Push changes | `gt ss` (or `gt ss -u` for existing) |
| Update from main | `gt sync` |
| Amend current | `gt modify -a` |
| View current stack | `gt log --stack` |
| View all branches | `gt log` |

## Forbidden Git Commands

Never use on stacked branches:

| Forbidden | Use Instead |
|-----------|-------------|
| `git rebase` | `gt restack` |
| `git push --force` | `gt submit` |
| `git branch -d` | `gt delete` |
| `git checkout -b` | `gt create` |
