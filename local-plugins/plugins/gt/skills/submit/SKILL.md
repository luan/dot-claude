---
name: gt:submit
description: >
  Push Graphite stack and update PRs. Triggers: 'submit', 'push',
  'update PRs', 'push stack', 'send PRs'.
user-invocable: true
context: fork
agent: general-purpose
allowed-tools:
  - "Bash(gt:*)"
  - "Bash(git status)"
  - "Bash(git branch:*)"
  - Skill
---

# Submit

Push Graphite stack and create/update PRs.

## Modes

| Mode | Command | When |
|------|---------|------|
| **Default** | `gt ss -u` | Always, unless user specifies otherwise |
| Single PR | `gt submit` | User explicitly says "submit this PR" / "update this PR" |
| Create new | `gt ss` | User explicitly says "create PR" / "create PRs" |

Default is `gt ss -u` (stack, update-only) — avoids accidentally creating PRs for WIP branches.

## Steps

1. **Check stack health**: `gt log --stack 2>&1`
   - If restack needed (diverged parents, conflicts), run `Skill(gt:restack)` first.

2. **Submit**:
   ```bash
   gt ss -u 2>&1    # default
   # or gt submit / gt ss depending on mode
   ```

3. **PR descriptions** (create modes only):
   When mode is **Create new** (`gt ss`) or **Single PR** (`gt submit`) on a branch without an existing PR, run `Skill(pr-descr)` for each newly created PR. Skip for `gt ss -u` (PRs already have descriptions).

4. **Report**: list Graphite URLs (`app.graphite.com/...`) for updated PRs. Never report GitHub URLs.
