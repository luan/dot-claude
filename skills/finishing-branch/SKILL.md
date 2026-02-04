---
name: finishing-branch
description: "Triggers: 'done', 'finished', 'wrap up'. Finalize branch for review."
user-invocable: true
---

# Finishing a Branch

Verify → commit → ready for review. Never submits PRs.

## Steps

1. **Run tests** - stop if failing
2. **Check for uncommitted changes** - `git status`
   - If changes: use Skill tool to invoke `commit`
3. **Squash commits** - `gt squash` (clean history)
4. **Show stack** - `gt log`
5. **Report:** "Branch ready. Run `gt ss` to submit."
