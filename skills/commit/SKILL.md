---
name: commit
description: Create conventional commit messages. Explains WHY not WHAT.
user-invocable: true
allowed-tools:
  - "Bash(git status)"
  - "Bash(git diff:*)"
  - "Bash(git log:*)"
  - "Bash(git add:*)"
  - "Bash(git commit:*)"
  - "Bash(git branch:*)"
  - Read
  - Glob
  - Grep
---

# Commit

Create conventional commits explaining WHY changes were made.

## Steps

1. **Analyze** (parallel):
   ```bash
   git status
   git diff --cached  # or git diff if nothing staged
   git log --oneline -5
   ```

2. **Create message**: `type(scope): description`
   - Max 72 chars, lowercase, no period, imperative mood
   - Types: feat|fix|refactor|perf|docs|test|style|build|ci|chore|revert
   - Scope: primary area (auth, api, ui, db) or omit if global

3. **Confirm** with AskUserQuestion: "Commit with this message?"

4. **Execute**:
   ```bash
   git commit -m "$(cat <<'EOF'
   type(scope): description
   EOF
   )"
   ```

## Hook failures

If hooks modify files: `git add -u && git commit --amend --no-edit`
If hooks fail: show error, suggest fix, let user decide.

## Special ops

- **Amend**: analyze prev + new, use `--amend` or `--amend --no-edit`
- **Squash**: combine commits, unify message with primary purpose
- **Fixup**: `git commit --fixup=<SHA>`

## Edge cases

- Nothing staged: ask "Stage all changes?" (all vs tracked vs select)
- Multiple unrelated changes: use **git-surgeon** skill to split
- Clean tree: "No changes to commit"

## Good vs bad

Good: `feat(search): add fuzzy matching for better UX`
Bad: `updated files`, `fix bug`, `WIP`
