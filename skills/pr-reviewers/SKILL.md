---
name: pr-reviewers
description: Recommend PR reviewers based on code ownership - spreads load to avoid always picking same people
user-invocable: true
disable-model-invocation: true
allowed-tools:
  - "Bash(gh pr view:*)"
  - "Bash(gh pr list:*)"
  - "Bash(gh pr edit:*)"
  - "Bash(gh api:*)"
  - "Bash(gh repo view:*)"
  - "Bash(git blame:*)"
  - "Bash(git log:*)"
  - "Bash(git diff:*)"
  - "Bash(git branch:*)"
  - Read
  - Glob
  - Grep
---

# PR Reviewer Picker

Recommend reviewers based on code expertise while spreading load.

**CRITICAL: "Reviewed your PRs" = PENALTY, not positive signal.**

## Steps

1. **Detect PR**: `gh pr view --json number,headRefName,author -q '.number'`
2. **Get changed files**: `gh pr view <PR> --json files -q '.files[].path'`

3. **Gather candidates** (parallel):
   - Blame: `git blame --line-porcelain <file> | grep "^author " | sort | uniq -c`
   - Recent commits: `git log --since="6 months ago" --pretty=format:"%an <%ae>" -- <files>`
   - Recent reviewers (for PENALTY): `gh pr list --author @me --state merged --limit 10 --json reviews`
   - CODEOWNERS: check `.github/CODEOWNERS` or `CODEOWNERS`

4. **Validate**: Remove PR author, non-members, bots

5. **Score**:
   | Factor | Weight |
   |--------|--------|
   | Blame ownership | 45% |
   | Recent commits | 35% |
   | CODEOWNERS match | 20% |

   **Diversity penalty** (mandatory):
   - 3+ of last 5 PRs: -30%
   - 2 of last 5 PRs: -15%

6. **Present top 3** with CODE EXPERTISE reasons only:
   - "Owns 60% of changed lines"
   - "12 commits to these files"
   - "CODEOWNERS for src/auth/"

   **Never**: "Reviewed your recent PRs"

   Ask: "Add these reviewers?"

7. **Add**: `gh pr edit <PR> --add-reviewer alice,bob,carol`
