---
name: gh-stack:submit
description: >
  Push code to remote and create or update stacked PRs using gh-stack. REPLACES git push and
  gh pr create — never use those directly. Triggers: 'push', 'push my changes', 'ship it',
  'send this up', 'submit', 'update PRs', 'create PR', 'push stack', 'send PRs'.
user-invocable: true
context: fork
agent: general-purpose
allowed-tools:
  - "Bash(gh stack:*)"
  - "Bash(git status)"
  - "Bash(git branch:*)"
  - Skill
---

# Submit

Push gh-stack branches and create/update PRs.

## Modes

| Mode | Command | When |
|------|---------|------|
| **Default** | `gh stack push` | Push only, no PR creation |
| Create PRs | `gh stack submit --auto` | User says "create PR" / "create PRs" |
| Draft PRs | `gh stack submit --auto --draft` | User says "draft PR" / "draft PRs" |

## Steps

1. **Check stack health**: `gh stack view --json 2>&1`
   - If any branch has `needsRebase: true`, run `Skill(gh-stack:restack)` first.

2. **Submit**:
   ```bash
   gh stack push 2>&1          # default: push only
   gh stack submit --auto 2>&1 # create PRs
   ```

3. **PR descriptions** (create modes only):
   When creating new PRs, run `Skill(pr-descr)` for each newly created PR. Skip for push-only mode.

4. **Report**: list GitHub PR URLs from stderr output.
