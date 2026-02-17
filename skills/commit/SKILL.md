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
  - TaskList
---

# Commit

Create conventional commits explaining WHY changes made.

## Context

Status: !`git status -sb 2>/dev/null`
Staged diff: !`git diff --cached --stat 2>/dev/null`
Recent commits: !`git log --oneline -5 2>/dev/null`

**Context:** Main thread / foreground only. Workers never commit.

## Steps

1. **Analyze**: review context above. If nothing staged, read full `git diff`. If staged, read `git diff --cached` for details.

2. **Message**: use conventional commit format — `type(scope): description`, max 72 chars, lowercase, no period, imperative mood.
   Types: feat|fix|refactor|perf|docs|test|style|build|ci|chore|revert. Scope: primary area or omit if global.
   Multi-line: blank line then body wrapping at 72 chars explaining motivation not mechanics.
   If task active (TaskList, filter by project + status=in_progress), append ID: `fix(auth): handle token expiry (task-<id>)`

3. **Confirm** via AskUserQuestion: "Commit with this message?"

4. **Execute**:
   ```bash
   git commit -m "$(cat <<'EOF'
   type(scope): description
   EOF
   )"
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

