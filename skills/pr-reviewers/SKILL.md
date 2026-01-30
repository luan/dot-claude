---
name: pr-reviewers
description: Recommend and add PR reviewers based on code ownership, git blame, and CODEOWNERS - spreads review load to avoid always picking the same people
user-invocable: true
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

Intelligently recommend reviewers for a PR based on code expertise (blame, commits, CODEOWNERS) while spreading review load to avoid always picking the same people.

## ⚠️ CRITICAL: DO NOT CREATE FEEDBACK LOOPS

**NEVER use "reviewed your recent PRs" as a POSITIVE signal.**

This is WRONG:
```
@alice - High confidence - Reviewed 3 of your recent PRs  ❌ WRONG
```

This is CORRECT:
```
@alice - High confidence - Owns 45% of changed code, 8 commits to these files  ✓ CORRECT
```

**"Reviewed your PRs" = NEGATIVE signal (penalty), not positive.**

If someone reviewed 3+ of your last 5 PRs, they should be DEPRIORITIZED, not recommended.

## Step 1: Identify the PR

Try to auto-detect the PR from the current branch:
```bash
gh pr view --json number,headRefName,author -q '.number'
```

If no PR exists, inform the user they need to create one first.

Get the current user:
```bash
gh api user -q '.login'
```

## Step 2: Analyze Changed Files

Get the list of files changed in the PR:
```bash
gh pr view <PR_NUMBER> --json files -q '.files[].path'
```

## Step 3: Gather Reviewer Candidates

Run these analyses **in parallel** to find potential reviewers:

### 3a. Git Blame Analysis
For each changed file (limit to ~5 most significant files by lines changed):
```bash
git blame --line-porcelain <file> | grep "^author " | sort | uniq -c | sort -rn | head -5
```

Extract unique authors who have contributed to these files.

### 3b. Recent Commit Authors
Find who has committed to these files recently:
```bash
git log --since="6 months ago" --pretty=format:"%an <%ae>" -- <file1> <file2> ... | sort | uniq -c | sort -rn | head -10
```

### 3c. Recent PR Reviewers (For Diversity Check)
Find who has reviewed your recent PRs (to AVOID over-assigning):
```bash
gh pr list --author @me --state merged --limit 10 --json number,reviews -q '.[].reviews[].author.login' | sort | uniq -c | sort -rn
```

**Note:** This is used to DEPRIORITIZE frequent reviewers, not to boost them.

### 3d. CODEOWNERS (if exists)
Check for CODEOWNERS file:
```bash
cat .github/CODEOWNERS 2>/dev/null || cat CODEOWNERS 2>/dev/null || cat docs/CODEOWNERS 2>/dev/null
```

Parse to find owners for the changed paths.

## Step 4: Validate Candidates

Get list of valid reviewers (org members or collaborators):
```bash
# For org repos - get org members
gh api orgs/<ORG>/members --paginate -q '.[].login'

# Or for any repo - get collaborators with push access
gh api repos/<OWNER>/<REPO>/collaborators -q '.[].login'
```

**Filter candidates:**
- Remove the PR author (can't review own PR)
- Remove anyone not in the valid reviewers list
- Remove bots (usernames ending in `[bot]` or containing `bot`)

## Step 5: Score and Rank Candidates

**ONLY these factors contribute POSITIVELY to score:**

| Factor | Weight | Description |
|--------|--------|-------------|
| Blame ownership | 45% | % of changed lines they authored |
| Recent commits | 35% | Commits to these files in last 6 months |
| CODEOWNERS match | 20% | Listed as owner for changed paths |

**"Reviewed your PRs recently" is NOT a scoring factor. It's a PENALTY.**

### Diversity Penalty (Mandatory)

**Apply PENALTY to frequent reviewers:**

| Reviewed your last N PRs | Penalty |
|--------------------------|---------|
| 3+ of last 5 PRs | **-30% score** |
| 2 of last 5 PRs | **-15% score** |
| 0-1 of last 5 PRs | No penalty |

**If someone has NO code expertise but reviewed your PRs, they score ZERO.**

**Tiebreaker:** Prefer people who haven't reviewed your PRs recently.

## Step 6: Present Recommendations

Display the top 3 recommendations. **Reasons must be about CODE EXPERTISE:**

### Valid Reasons (USE THESE):
- "Owns 60% of changed lines in `src/auth/`"
- "12 commits to these files in last 6 months"
- "CODEOWNERS for `src/api/`"
- "Wrote the original implementation"
- "Active maintainer of this module"

### Invalid Reasons (NEVER USE):
- ❌ "Reviewed 3 of your recent PRs"
- ❌ "Frequently reviews your code"
- ❌ "Has reviewed similar PRs"

### Example Output:

```
## Recommended Reviewers

1. **@alice** (score: 85)
   - Owns 60% of changed lines in `src/auth/`
   - 12 commits to these files in last 6 months
   - CODEOWNERS for `src/auth/`

2. **@bob** (score: 72)
   - CODEOWNERS for `src/api/`
   - 8 commits to these files recently

3. **@carol** (score: 65)
   - Owns 40% of changed lines in `src/utils/`
   - Active contributor to this area

(Note: @dave has expertise but reviewed 3 of your last 5 PRs - deprioritized to spread load)
```

Then use **AskUserQuestion**:

```
Question: "Add these reviewers to your PR?"
Header: "Reviewers"
Options:
  1. "Add all 3" - "Add @alice, @bob, @carol as reviewers"
  2. "Add top 2" - "Add @alice, @bob as reviewers"
  3. "Add top 1" - "Add @alice as reviewer"
  4. "Other" - "Custom selection"
```

## Step 7: Add Reviewers

Based on user selection, add reviewers:
```bash
gh pr edit <PR_NUMBER> --add-reviewer alice,bob,carol
```

Confirm success:
```
Added reviewers: @alice, @bob, @carol
```

## Edge Cases

### No Valid Candidates Found
If no candidates pass validation:
```
Could not find suitable reviewers. Possible reasons:
- Changed files have no git history from current team members
- No recent collaboration history found

You may need to manually select reviewers.
```

### Fewer Than 3 Candidates
If only 1-2 valid candidates found, present what's available and note the limitation.

### Large PRs (Many Files)
For PRs with >20 files, focus analysis on:
1. Files with most lines changed
2. Files in core paths (src/, lib/, pkg/)
3. Skip generated files, tests (unless PR is test-focused)

## Notes

- **Prioritize code expertise** over collaboration history
- **Spread review load** - penalize frequent reviewers to avoid feedback loops
- Always validate reviewers against org/repo membership before suggesting
- Never suggest the PR author as a reviewer
- Prefer active contributors over historical ones
- If CODEOWNERS exists, weight it heavily
- Mention when an expert was deprioritized due to review frequency
