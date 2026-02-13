---
name: pr-description
description: "Update PR title and description from branch context. Triggers: 'pr description', 'update PR', 'PR title', 'describe PR'."
user-invocable: true
allowed-tools:
  - "Bash(git status *)"
  - "Bash(git diff *)"
  - "Bash(git log *)"
  - "Bash(git branch *)"
  - "Bash(git show *)"
  - "Bash(gh pr view *)"
  - "Bash(gh pr edit *)"
  - "Bash(gt log *)"
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
```

If no PR found, tell user and stop.

## Step 2: Get Diff

Diff against stack parent (not main) to handle stacked PRs:

```bash
# Stack parent from gt log, or fall back to origin/main
git diff <stack-parent>...HEAD
git log <stack-parent>..HEAD --oneline
```

If diff is large, use `--stat` first and read key files.

## Step 3: Generate Title

Format: `type(scope): description`
- Max 72 chars, lowercase, no period, imperative mood
- Types: feat|fix|refactor|perf|docs|test|style|build|ci|chore|revert
- Scope: primary area (auth, api, ui, db) or omit if global
- Good: `feat(search): add fuzzy matching for better UX`
- Bad: `updated files`, `fix bug`, `Add new feature for search`

## Step 4: Generate Body

Explain WHY the change is being made, with high-level HOW.
Keep concise. Don't list changes obvious from diff.

Format:
```
<1-3 sentences explaining motivation and approach>
```

No bullet lists, no headers, no changelog. Just prose.

## Step 5: Preview and Update

Show suggested title + body. Then AskUserQuestion: "Update PR?"

If confirmed:
```bash
gh pr edit <NUMBER> --title "<title>" --body "<body>"
```

Show PR URL when done.
