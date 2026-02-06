---
name: git-surgeon
description: Non-interactive hunk-level git staging, unstaging, discarding, undoing, fixup, and commit splitting. Use when selectively staging, unstaging, discarding, reverting, or splitting individual diff hunks by ID instead of interactively. Also use when asked to commit changes separately, make separate commits, or split changes into multiple commits.
---

# git-surgeon

Non-interactive hunk-level git operations. Precise control over staging, unstaging, discarding + undoing changes.

## Commands

```bash
# List hunks
git-surgeon hunks                          # unstaged (ID, file, +/- counts, preview)
git-surgeon hunks --staged                 # staged
git-surgeon hunks --file=src/main.rs       # filter by file
git-surgeon hunks --commit <HEAD/sha>      # from specific commit
git-surgeon hunks --commit <sha> --full    # with line numbers (for line-range splits)

# Show full diff for hunk (lines numbered for --lines)
git-surgeon show <id>
git-surgeon show <id> --commit HEAD

# Stage
git-surgeon stage <id1> <id2> ...
git-surgeon stage <id> --lines 5-30

# Stage + commit in one step
git-surgeon commit <id1> <id2> ... -m "message"
git-surgeon commit <id>:1-11 <id2> -m "message"   # inline line ranges

# Unstage
git-surgeon unstage <id1> <id2> ...
git-surgeon unstage <id> --lines 5-30

# Discard working tree changes
git-surgeon discard <id1> <id2> ...
git-surgeon discard <id> --lines 5-30

# Fixup earlier commit with staged changes
git-surgeon fixup <commit>

# Reword commit message
git-surgeon reword HEAD -m "new message"
git-surgeon reword <commit> -m "new message"
git-surgeon reword HEAD -m "subject" -m "body"

# Undo hunks from commit (reverse-apply → working tree)
git-surgeon undo <id1> <id2> ... --from <commit>
git-surgeon undo <id> --from <commit> --lines 2-10

# Undo entire files from commit
git-surgeon undo-file <file1> <file2> ... --from <commit>

# Split commit by hunk selection
git-surgeon split HEAD \
  --pick <id1> <id2> -m "first commit" \
  --rest-message "remaining changes"

# Split with subject + body
git-surgeon split HEAD \
  --pick <id1> -m "Add feature" -m "Detailed description." \
  --rest-message "Other changes" --rest-message "Body for rest."

# Split with line ranges (comma for non-contiguous)
git-surgeon split <commit> \
  --pick <id>:1-11,20-30 <id2> -m "partial split"

# Split into 3+ commits
git-surgeon split HEAD \
  --pick <id1> -m "first" \
  --pick <id2> -m "second" \
  --rest-message "rest"
```

## Typical Workflow

1. `git-surgeon hunks` → list hunks with IDs
2. `git-surgeon show <id>` → inspect (lines numbered)
3. `git-surgeon commit <id1> <id2> -m "message"` (or stage separately → `git commit`)
4. Partial hunk: `git-surgeon commit <id>:5-30 -m "message"`

## Fixup Workflow

1. `git-surgeon stage <id1> <id2>`
2. `git-surgeon fixup <commit-sha>` (HEAD → amend; older → autosquash rebase)
3. Unstaged changes preserved

## Undo Workflow

1. `git-surgeon hunks --commit <sha>`
2. `git-surgeon undo <id> --from <sha>` or `git-surgeon undo-file src/main.rs --from <sha>`
3. Changes appear as unstaged modifications

## Split Workflow

1. `git-surgeon hunks --commit <sha>` (use `--full` for line numbers)
2. `git-surgeon split <sha> --pick <id1> -m "first" --rest-message "second"`
3. Multiple `-m` flags → subject + body
4. `id:range` for partial hunks; commas for non-contiguous: `--pick <id>:2-6,34-37`
5. HEAD → direct reset; older → rebase. Requires clean working tree.

## Hunk IDs

- 7-char hex from file path + hunk content
- Stable while diff unchanged; duplicates get `-2`, `-3` suffixes
- ID not found → re-run `hunks` for fresh IDs
