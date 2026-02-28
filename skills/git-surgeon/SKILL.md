---
name: git-surgeon
description: 'Non-interactive hunk-level git operations — stage, unstage, discard, undo, fixup, and split by hunk ID. Triggers: "stage this hunk", "split commit", "partial stage", "discard hunk", "selective staging", "commit separately", "separate commits".'
---

# git-surgeon

Non-interactive hunk-level git ops. Precise staging, unstaging, discarding, undo control.

## Commands

```bash
# List hunks
git-surgeon hunks                          # unstaged (ID, file, +/- counts, preview)
git-surgeon hunks --staged                 # staged
git-surgeon hunks --file=src/main.rs       # filter by file
git-surgeon hunks --commit <HEAD/sha>      # from commit
git-surgeon hunks --commit <sha> --full    # with line numbers

# Show full diff (lines numbered)
git-surgeon show <id>
git-surgeon show <id> --commit HEAD

# Stage
git-surgeon stage <id1> <id2> ...
git-surgeon stage <id> --lines 5-30

# Stage + commit
git-surgeon commit <id1> <id2> ... -m "message"
git-surgeon commit <id>:1-11 <id2> -m "message"   # inline ranges

# Unstage
git-surgeon unstage <id1> <id2> ...
git-surgeon unstage <id> --lines 5-30

# Discard working tree
git-surgeon discard <id1> <id2> ...
git-surgeon discard <id> --lines 5-30

# Fixup earlier commit
git-surgeon fixup <commit>

# Reword
git-surgeon reword HEAD -m "new message"
git-surgeon reword <commit> -m "subject" -m "body"

# Undo hunks from commit (reverse-apply → working tree)
git-surgeon undo <id1> <id2> ... --from <commit>
git-surgeon undo <id> --from <commit> --lines 2-10

# Undo files
git-surgeon undo-file <file1> <file2> ... --from <commit>

# Split commit
git-surgeon split HEAD \
  --pick <id1> <id2> -m "first" \
  --rest-message "remaining"

git-surgeon split HEAD \
  --pick <id1> -m "Add feature" -m "Body." \
  --rest-message "Other" --rest-message "Body."

# Split with ranges (comma for non-contiguous)
git-surgeon split <commit> \
  --pick <id>:1-11,20-30 <id2> -m "partial"

# Split 3+ commits
git-surgeon split HEAD \
  --pick <id1> -m "first" \
  --pick <id2> -m "second" \
  --rest-message "rest"
```

## Workflows

**Typical:**

1. `hunks` → IDs → `show <id>` → inspect → `commit <id1> <id2> -m "msg"`
2. Partial: `commit <id>:5-30 -m "msg"`

**Fixup:**

1. `stage <id1> <id2>` → `fixup <sha>` (HEAD → amend; older → autosquash rebase)
2. Unstaged changes preserved

**Undo:**

1. `hunks --commit <sha>` → `undo <id> --from <sha>` → changes appear as unstaged mods

**Split:**

1. `hunks --commit <sha>` (use `--full` for line numbers)
2. `split <sha> --pick <id1> -m "first" --rest-message "second"`
3. Multiple `-m` → subject + body. `id:range` for partial; commas for non-contiguous
4. HEAD → direct reset; older → rebase. Requires clean working tree.

## Troubleshooting

- **Hunk IDs** are 7-char hex derived from file path + content. Stable while diff is unchanged; duplicate content gets `-2`, `-3` suffixes.
- **ID not found** → diff changed since last `hunks` listing. Re-run `hunks` to get fresh IDs.
