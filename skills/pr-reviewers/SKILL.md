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

Recommend reviewers based on code expertise while spreading review load.

**"Reviewed your PRs" = PENALTY.** Rotating reviewers prevents knowledge silos and balances workload — the same reviewer on every PR misses patterns that fresh perspectives catch and creates review bottlenecks when that person is unavailable.

## Steps

1. **Detect PR**: `gh pr view --json number,headRefName,author -q '.number'`
2. **Get changed files**: `gh pr view <PR> --json files -q '.files[].path'`
   - Separate **existing** vs **new** files (additions where `deletions == 0` and `status == "added"`)
   - **Exclude generated files**: `*.generated.*`, `*.pb.go`, `*.pb.swift`, `*_generated.rs`, `*.g.dart`, lock files, `vendor/`, `node_modules/`, `*.min.*`, `*.snap`, `__snapshots__/`. Also check for `@generated` marker in first 5 lines. Generated files skew blame toward whoever ran the generator.

3. **Gather candidates** (parallel): Read `references/scoring.md` for candidate gathering commands, scoring weights, and penalty multipliers.

4. **Validate**: Remove PR author, non-members, bots

5. **Score**: Apply scoring algorithm from `references/scoring.md`.

6. **Present results** — up to 3 candidates (don't pad if fewer exist). For each:
   - Score breakdown with concrete numbers
   - Expertise reason: "Owns 60% of changed lines", "12 commits to these files", "CODEOWNERS for src/auth/"
   - If penalized: "(currently reviewing 6 PRs)"

   **Never say** "Reviewed your recent PRs" as positive signal.

   Ask: "Add these reviewers?"

7. **Add**: `gh pr edit <PR> --add-reviewer alice,bob,carol`
