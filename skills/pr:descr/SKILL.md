---
name: pr:descr
description: "Update PR title and description from branch context. Triggers: 'pr description', 'update PR', 'PR title', 'describe PR'."
user-invocable: true
model: sonnet
disable-model-invocation: false
allowed-tools:
  - "Bash(git status:*)"
  - "Bash(git diff:*)"
  - "Bash(git log:*)"
  - "Bash(git branch:*)"
  - "Bash(git show:*)"
  - "Bash(gh pr view:*)"
  - "Bash(gh pr edit:*)"
  - Read
  - AskUserQuestion
---

# PR Description

Update an existing PR's title and description from branch context.

**Assumes PR already exists.** This skill NEVER pushes or submits.

## Step 1: Check State

```bash
gh pr view --json number,title,body,headRefName -q '{number,title,headRefName}'
git log --oneline -10
git status -sb
```

If no PR found, tell user and stop.

**Handle edge cases — ask before proceeding:**

1. **On main branch**: "You're on main. Did you mean to be on a feature branch?"
2. **Uncommitted changes**: "You have uncommitted changes. Describe from just committed changes, or include uncommitted too?"
3. **Untracked files**: "Untracked files exist (list them). Include in description or ignore?"
4. **No commits ahead**: "Branch has no commits ahead. Describe uncommitted changes?"

If state is clear (commits on branch, nothing dirty), proceed directly.

## Step 2: Get Diff

Base: !`gt parent 2>/dev/null || gt trunk 2>/dev/null || git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's|refs/remotes/||' || echo main`

```bash
git diff <base>...HEAD
git log <base>..HEAD --oneline
```

The `...` syntax automatically finds the common ancestor — prevents showing irrelevant diffs if parent has moved ahead.

If including uncommitted changes (per Step 1):

```bash
git diff HEAD  # uncommitted on top
```

If diff is large, use `--stat` first and read key files. If context is unclear from diff alone, check commit messages and read relevant source.

## Step 3: Generate Title and Body

**Title**: conventional commit — `type(scope): description`, max 72 chars, lowercase, no period, imperative mood. Types: feat|fix|refactor|perf|docs|test|style|build|ci|chore|revert. Scope: primary area or omit if global.

**Body**: 1-3 sentences explaining WHY the change is being made, with high-level HOW. No bullet lists, no headers, no changelog. Just prose. Don't list changes obvious from the diff.

## Step 4: Preview and Update

Show suggested title + body, then add observations if relevant:

- Is the WHY unclear and needs more context from user?
- Would this be easier to review as multiple PRs?
- Are there unrelated changes mixed in?

Don't force observations — skip if everything looks clean.

AskUserQuestion: "Update PR with this title and description?"

If confirmed:

```bash
gh pr edit <NUMBER> --title "<title>" --body "<body>"
```

Show PR URL when done.
