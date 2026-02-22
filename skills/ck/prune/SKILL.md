---
name: ck:prune
description: "Archive completed tasks older than N days and remove empty task lists. Triggers: 'prune tasks', 'clean up tasks', 'archive old tasks'."
argument-hint: "[--days N] [--dry-run] [--list <id>]"
user-invocable: true
allowed-tools:
  - Bash
---

# ck:prune

Archive completed tasks and remove empty task lists. Wraps `ck task prune` — archives tasks completed longer than the threshold (default: 30 days) and cleans up any task lists left empty after archival.

## Steps

1. Run with passthrough args:

```bash
ck task prune $ARGUMENTS
```

2. Print output. If empty or no tasks pruned: "Nothing to prune."

## Useful flags

- `--days N` — override age threshold (default 30)
- `--dry-run` — preview what would be pruned without acting
- `--list <id>` — scope to a single task list

## Error handling

If `ck` is not installed or the command fails, report the error and suggest checking `ck --help` for installation guidance.
