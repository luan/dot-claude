# Scoring Algorithm

## Candidate Gathering Commands

**For existing files** — use `git log` not `git blame` (faster):
```
git log --since="12 months ago" --format='%aN <%aE>' --diff-filter=M -- <files> | sort | uniq -c | sort -rn
```

**For new files** — find experts on parent directory:
```
git log --since="12 months ago" --format='%aN <%aE>' -- <parent_dirs> | sort | uniq -c | sort -rn
```

**Line-weighted ownership** (for top candidates only, not all files):
```
git log --since="12 months ago" --numstat --format='%aN' -- <files>
```
Sum lines touched per author. Normalize: `author_lines / total_lines`.

**Recent reviewers** (for PENALTY):
```
gh pr list --author @me --state merged --limit 10 --json reviews
```

**Current review load**:
```
gh api graphql -f query='{ search(query:"is:pr is:open review-requested:<username>", type:ISSUE) { issueCount } }'
```

**CODEOWNERS**: check `.github/CODEOWNERS` or `CODEOWNERS`. Skip if absent — don't error.

## Scoring Weights

| Factor | Weight | Normalization |
|--------|--------|---------------|
| Line-weighted ownership | 40% | `author_lines / total_lines` across changed files |
| Recent commit count | 30% | `author_commits / max_commits` among candidates |
| CODEOWNERS match | 15% | 1.0 if matched, 0.0 if not (skip if no file) |
| Directory familiarity (new files) | 15% | `dir_commits / max_dir_commits` |

## Penalties (multiply final score)

| Condition | Multiplier |
|-----------|------------|
| Reviewed 3+ of last 5 author PRs | ×0.7 |
| Reviewed 2 of last 5 author PRs | ×0.85 |
| 5+ open review requests | ×0.8 |
| 8+ open review requests | ×0.6 |
