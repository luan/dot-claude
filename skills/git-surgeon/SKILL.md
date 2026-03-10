---
name: git-surgeon
description: 'Non-interactive hunk-level git operations — stage, unstage, discard, undo, fixup, split by hunk ID.'
---

# git-surgeon

Hunk-level git ops without interactive prompts. Every operation uses stable hunk IDs (7-char hex from file path + content).

## Commands

```bash
# List hunks — always start here to get IDs
git-surgeon hunks                          # unstaged (ID, file, +/- counts, preview)
git-surgeon hunks --staged                 # staged
git-surgeon hunks --file=src/main.rs       # filter by file
git-surgeon hunks --commit <HEAD/sha>      # from commit
git-surgeon hunks --commit <sha> --full    # with line numbers

# Inspect a hunk
git-surgeon show <id>
git-surgeon show <id> --commit HEAD

# Stage / unstage / discard — all accept multiple IDs and optional --lines
git-surgeon stage <id1> <id2> ...
git-surgeon stage <id> --lines 5-30
git-surgeon unstage <id1> <id2> ...
git-surgeon discard <id1> <id2> ...

# Stage + commit in one step
git-surgeon commit <id1> <id2> ... -m "message"
git-surgeon commit <id>:1-11 <id2> -m "message"   # inline line ranges

# Fixup earlier commit (HEAD → amend; older → autosquash rebase)
git-surgeon fixup <commit>

# Reword (multiple -m for subject + body)
git-surgeon reword HEAD -m "new message"
git-surgeon reword <commit> -m "subject" -m "body"

# Undo hunks from commit (reverse-apply → working tree)
git-surgeon undo <id1> <id2> ... --from <commit>
git-surgeon undo-file <file1> <file2> ... --from <commit>

# Split commit into multiple
git-surgeon split HEAD \
  --pick <id1> <id2> -m "first" \
  --rest-message "remaining"

# Split with line ranges (comma for non-contiguous)
git-surgeon split <commit> \
  --pick <id>:1-11,20-30 <id2> -m "partial"

# Split into 3+ commits
git-surgeon split HEAD \
  --pick <id1> -m "first" \
  --pick <id2> -m "second" \
  --rest-message "rest"
```

## Workflows

**Selective commit**: `hunks` → `show <id>` → `commit <id1> <id2> -m "msg"`

**Fixup**: `stage <id1> <id2>` → `fixup <sha>`. Unstaged changes preserved.

**Undo**: `hunks --commit <sha>` → `undo <id> --from <sha>` → reversed changes appear unstaged.

**Split**: `hunks --commit <sha> --full` → `split <sha> --pick <id1> -m "first" --rest-message "second"`. HEAD splits via reset; older commits via rebase. Requires clean working tree.

## Troubleshooting

- **IDs shift** when the diff changes. If "ID not found", re-run `hunks` for fresh IDs.
- **Duplicate content** across files → IDs get `-2`, `-3` suffixes.
