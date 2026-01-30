---
name: stack-submit
description: Push stack and create/update PRs (gt submit, gt ss)
user-invocable: true
allowed-tools:
  - "Bash(gt submit:*)"
  - "Bash(gt ss:*)"
  - "Bash(gt log:*)"
  - "Bash(git status)"
---

# Stack Submit

Push stack branches and create/update PRs.

## Commands

```bash
gt submit               # Push current + downstack, create/update PRs
gt submit --stack       # Push entire stack (up and down), alias: gt ss
gt ss                   # Shorthand for submit --stack
gt ss --update-only     # Push stack, only update existing PRs (no new PRs)
gt ss --draft           # Create new PRs as drafts
gt ss -u                # Short form of --update-only
```

## Usage

| User Says | Command |
|-----------|---------|
| "push stack" / "submit stack" | `gt ss` |
| "update PRs" / "push without new PRs" | `gt ss --update-only` |
| "create PR" / "open PR" | `gt submit` |
| "create PRs for stack" | `gt ss` |

**Use `--update-only` / `-u` when PRs already exist** - avoids accidentally creating new PRs.
