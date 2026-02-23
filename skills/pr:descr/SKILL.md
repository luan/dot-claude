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
  - "Bash(gh pr list:*)"
  - "Bash(cat *PULL_REQUEST_TEMPLATE*)"
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
cat .github/PULL_REQUEST_TEMPLATE.md 2>/dev/null
gh pr list --state merged --limit 3 --json body -q '.[].body' | head -80
```

If no PR found, tell user and stop.

**Edge cases — ask before proceeding:**
- **On main:** "You're on main. Did you mean to be on a feature branch?"
- **Uncommitted changes:** "Describe from just committed, or include uncommitted too?"
- **No commits ahead:** "Branch has no commits ahead. Describe uncommitted changes?"

If state is clear, proceed directly.

## Step 2: Get Diff

Detect base: `gt parent 2>/dev/null || gt trunk 2>/dev/null || git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's|refs/remotes/||' || echo main`

This base detection pattern is shared across pr: skills — finds the stack parent (Graphite) or falls back to trunk.

```bash
git diff <base>...HEAD        # three-dot finds common ancestor
git log <base>..HEAD --oneline
```

If including uncommitted (per Step 1): `git diff HEAD`

If diff is large, use `--stat` first and read key files.

## Step 3: Generate Title and Body

**Title**: conventional commit per /commit skill — `type(scope): description`. Max 72 chars — GitHub truncates longer titles in list views.

**Body**: Follow the repo's PR template if one exists — use its headings and fill each section with content from the diff. Otherwise, if recent merged PRs share a consistent format (e.g., test plan section, changelog), match that structure. If neither exists or history is inconsistent, default to 1-3 sentences explaining WHY with high-level HOW. Don't list changes obvious from diff.

## Step 4: Preview and Update

Show title + body. Add observations only if genuinely useful:
- WHY is unclear → ask user for context
- Unrelated changes mixed in → suggest splitting
- Too large for one review → suggest multiple PRs

AskUserQuestion: "Update PR with this title and description?"

```bash
gh pr edit <NUMBER> --title "<title>" --body "<body>"
```

Show PR URL when done.
