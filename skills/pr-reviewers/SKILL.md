---
name: pr-reviewers
description: "Recommend PR reviewers based on code ownership. Triggers: 'who should review', 'add reviewers', 'find reviewers'. Spreads load to avoid always picking same people."
model: sonnet
context: fork
agent: general-purpose
user-invocable: true
disable-model-invocation: true
allowed-tools:
  - "Bash(gh pr view:*)"
  - "Bash(gh pr list:*)"
  - "Bash(gh pr edit:*)"
  - "Bash(gh api:*)"
  - "Bash(gh repo view:*)"
  - "Bash(git log:*)"
  - "Bash(git diff:*)"
  - "Bash(git shortlog:*)"
  - "Bash(git branch:*)"
  - "Bash(wc:*)"
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
   - Separate into **existing** vs **new** files (additions where
     `deletions == 0` and `status == "added"`)
   - **Exclude generated files**: drop paths matching common generated
     patterns — `*.generated.*`, `*.pb.go`, `*.pb.swift`,
     `*_generated.rs`, `*.g.dart`, `package-lock.json`,
     `yarn.lock`, `Pods/`, `*.xcodeproj/`, `*.pbxproj`,
     `vendor/`, `node_modules/`, `*.min.js`, `*.min.css`,
     `*.snap`, `__snapshots__/`, `*.lock`.
     Also check for `@generated` marker in first 5 lines of file.
     Generated files skew blame toward whoever ran the generator.

3. **Gather candidates** (parallel):

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

   **CODEOWNERS**: check `.github/CODEOWNERS` or `CODEOWNERS`.
   Skip if absent — don't error.

4. **Validate**: Remove PR author, non-members, bots

5. **Score**:

   | Factor | Weight | Normalization |
   |--------|--------|---------------|
   | Line-weighted ownership | 40% | `author_lines / total_lines` across changed files |
   | Recent commit count | 30% | `author_commits / max_commits` among candidates |
   | CODEOWNERS match | 15% | 1.0 if matched, 0.0 if not (skip if no file) |
   | Directory familiarity (new files) | 15% | `dir_commits / max_dir_commits` |

   **Penalties** (multiply final score):

   | Condition | Multiplier |
   |-----------|------------|
   | Reviewed 3+ of last 5 author PRs | ×0.7 |
   | Reviewed 2 of last 5 author PRs | ×0.85 |
   | 5+ open review requests | ×0.8 |
   | 8+ open review requests | ×0.6 |

6. **Present results**:

   Show **up to 3** candidates. If fewer than 3 exist, show what you
   have — don't pad. For each:
   - Score breakdown with concrete numbers
   - CODE EXPERTISE reason: "Owns 60% of changed lines",
     "12 commits to these files", "CODEOWNERS for src/auth/"
   - If penalized for load: "(currently reviewing 6 PRs)"

   **Never say**: "Reviewed your recent PRs" as a positive signal.

   Ask: "Add these reviewers?"

7. **Add**: `gh pr edit <PR> --add-reviewer alice,bob,carol`
