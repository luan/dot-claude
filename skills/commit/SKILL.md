---
name: commit
description: "Use when the user wants to commit, save changes, create a conventional commit, amend a commit, make a fixup commit, or squash commits."
user-invocable: true
context: fork
agent: general-purpose
model: haiku
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
  - AskUserQuestion
---

# Commit

Create conventional commits explaining WHY changes made.

## Steps

1. **Analyze** (parallel):
   ```bash
   git status
   git diff --cached  # or git diff if nothing staged
   git log --oneline -5
   ```

2. **Message**: `type(scope): description`
   - Max 72 chars, lowercase, no period, imperative
   - Types: feat|fix|refactor|perf|docs|test|style|build|ci|chore|revert
   - Scope: primary area (auth, api, ui, db) or omit if global
   - If beads issue in_progress (`bd list --status in_progress -q`), append ID: `fix(auth): handle token expiry (bd-abc123)`

3. **Confirm** via AskUserQuestion: "Commit with this message?"

4. **Execute**:
   ```bash
   git commit -m "$(cat <<'EOF'
   type(scope): description
   EOF
   )"
   ```

5. **Beads sync** (after successful commit):
   ```bash
   bd epic close-eligible 2>/dev/null || true
   bd sync 2>/dev/null || true
   ```

## Hook failures

Hooks modify files: `git add -u && git commit --amend --no-edit`
Hooks fail: show error, suggest fix, let user decide.

## Special ops

- **Amend**: analyze prev + new, use `--amend` or `--amend --no-edit`
- **Squash**: combine commits, unify message with primary purpose
- **Fixup**: `git commit --fixup=<SHA>`

## Edge cases

- Nothing staged → ask "Stage all changes?" (all vs tracked vs select)
- Multiple unrelated changes → use **git-surgeon** to split
- Clean tree → "No changes to commit"

## Good vs bad

Good: `feat(search): add fuzzy matching for better UX`
Bad: `updated files`, `fix bug`, `WIP`
