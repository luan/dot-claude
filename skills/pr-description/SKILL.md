---
name: pr:description
description: "Update PR title and description from branch context. Triggers: 'pr description', 'update PR', 'PR title', 'describe PR'."
user-invocable: true
disable-model-invocation: true
allowed-tools:
  - "Bash(git status:*)"
  - "Bash(git diff:*)"
  - "Bash(git log:*)"
  - "Bash(git branch:*)"
  - "Bash(git show:*)"
  - "Bash(gh pr view:*)"
  - "Bash(gh pr edit:*)"
  - "Bash(gt log:*)"
  - Read
  - AskUserQuestion
---

# PR Description

Update an existing PR's title and description from branch context.

**Assumes PR already exists.** Use `gt submit` to create PRs.

## Step 1: Get Context

```bash
gh pr view --json number,title,body,headRefName -q '{number,title,headRefName}'
gt log --short
git status -sb
```

If no PR found, tell user and stop.

**Handle edge cases:**

- **Uncommitted changes**: "You have uncommitted changes. Describe PR from just committed changes, or include uncommitted too?"
- **Untracked files**: "Untracked files exist (list them). Include in description or ignore?"
- **No commits ahead**: "Branch has no commits ahead. Describe uncommitted changes?"

If state is clear (commits on branch, nothing dirty), proceed directly.

## Step 2: Get Diff

Diff against stack parent (not main) to handle stacked PRs:

```bash
# Stack parent from gt log, or fall back to origin/!`gt parent 2>/dev/null || gt trunk`
git diff <stack-parent>...HEAD
git log <stack-parent>..HEAD --oneline
```

The `...` syntax finds the common ancestor — prevents showing unrelated changes if parent moved.

If including uncommitted changes (per Step 1):

```bash
git diff HEAD  # uncommitted on top
```

If diff is large, use `--stat` first and read key files. If context is unclear from diff alone, check commit messages and read relevant source.

## Step 3: Generate Title

Use conventional commit format — `type(scope): description`, max 72 chars, lowercase, no period, imperative mood.
Types: feat|fix|refactor|perf|docs|test|style|build|ci|chore|revert. Scope: primary area or omit if global.
Multi-line: blank line then body wrapping at 72 chars explaining motivation not mechanics.

## Step 4: Generate Body

Explain WHY the change is being made, with high-level HOW.
Keep concise. Don't list changes obvious from diff.

Format:

```
<1-3 sentences explaining motivation and approach>
```

No bullet lists, no headers, no changelog. Just prose.

## Step 5: Preview and Observations

Show suggested title + body, then add observations if relevant:

- Is the WHY unclear and needs more context from user?
- Would this be easier to review as multiple PRs?
- Are there unrelated changes mixed in?

Don't force observations — skip if everything looks clean.

## Step 6: Update

AskUserQuestion: "Update PR with this title and description?"

If confirmed:

```bash
gh pr edit <NUMBER> --title "<title>" --body "<body>"
```

Show PR URL when done.
