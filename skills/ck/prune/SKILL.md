---
name: ck:prune
description: "Archive completed tasks older than N days and remove empty task lists. Triggers: 'prune tasks', 'clean up tasks', 'archive old tasks'."
argument-hint: "[--days N] [--dry-run] [--list <id>]"
user-invocable: true
allowed-tools:
  - Bash
---

# ck:prune

Archive completed tasks and remove empty task lists.

## Steps

1. Run `ck task prune` with any passthrough args:

```
ck task prune <args>
```

2. Print output. If empty or no tasks pruned, print: "Nothing to prune."
