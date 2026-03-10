---
name: ct:prune
description: "Archive completed tasks older than N days and remove empty task lists. Triggers: 'prune tasks', 'clean up tasks', 'archive old tasks'."
argument-hint: "[--days N] [--dry-run] [--list <id>]"
user-invocable: true
allowed-tools:
  - Bash
---

# ct:prune

Archive completed tasks and clean up empty task lists. Wraps `ct task prune` with a 30-day default threshold.

## Steps

1. Run with passthrough args:

```bash
ct task prune $ARGUMENTS
```

2. Print output. Nothing pruned → "Nothing to prune."

## Flags

- `--days N` — age threshold (default 30)
- `--dry-run` — preview without acting
- `--list <id>` — scope to single task list

## Errors

`ct` not installed or command fails → report error, suggest `ct --help`.
