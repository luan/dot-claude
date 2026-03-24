# git-surgeon: Selective Git Operations

Before running any of these raw git commands, use the `/git-surgeon` skill instead:

| Instead of | Use |
|---|---|
| `git checkout HEAD -- <file>` | `git-surgeon discard <hunk-ids>` |
| `git restore <file>` | `git-surgeon discard <hunk-ids>` |
| `git add -p` | `git-surgeon hunks` → `git-surgeon stage <ids>` |
| `git reset HEAD <file>` | `git-surgeon unstage <hunk-ids>` |
| `git stash` + `git stash drop` (to discard) | `git-surgeon discard <hunk-ids>` |
| `echo "y\nn\n" \| git add -p` | `git-surgeon stage <ids>` |
| `git commit --fixup` + rebase | `git-surgeon stage` → `git-surgeon fixup <sha>` |

The workflow is always: `git-surgeon hunks` first to get stable IDs, then operate on those IDs.

Exception: whole-file `git add <file>` for committing is fine — git-surgeon is for when you need to be selective within or across files.
