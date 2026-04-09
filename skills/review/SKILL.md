---
name: review
description: "Quick adversarial code review — simplify pre-pass + 2 focused reviewers. Default for everyday reviews. Triggers: 'review', 'review my changes', 'check this code', 'code review'. For deep multi-perspective review use /ultrareview."
argument-hint: "[base..head | file-list | PR#] [--auto critical|high|medium|all]"
user-invocable: true
allowed-tools:
  - Agent
  - Skill
  - Read
  - Glob
  - Grep
  - "Bash(git diff:*)"
  - "Bash(git log:*)"
  - "Bash(git status:*)"
  - "Bash(ct tool:*)"
  - "Bash(ct review:*)"
  - "Bash(gh pr:*)"
  - "Bash(gh api:*)"
---

# Review

Fast adversarial review: simplify pre-pass, then 2 parallel reviewers — one for correctness/security, one adversarial. Covers the highest-value dimensions without the overhead of 5-7 specialists.

**NEVER review inline.** Always dispatch subagents via the Agent tool.

## Step 1: Scope

Resolve BASE: `gt parent 2>/dev/null || gt trunk 2>/dev/null || git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's|refs/remotes/||'`. Args override.

| Input        | Diff source                       |
| ------------ | --------------------------------- |
| (none)       | `git diff $BASE...HEAD`           |
| `main..HEAD` | BASE=main                         |
| file list    | `git diff HEAD -- <files>` + read |
| `#123`       | `gh pr diff 123`                  |

**Bugfix detection:** If commit messages contain "fix"/"bugfix"/"hotfix", classify files as production vs test. ALL test-only → verdict **FAIL** with Critical: "Bugfix contains no production code changes."

## Step 2: Simplify Pre-pass

**ALWAYS run** unless `#<PR>` input.

`Skill("simplify")`

## Step 3: Context

`ct tool gitcontext --base $BASE --stat` → diff-stat, changed-files, log. Fetch PR context via `gh pr view` if available.

## Step 4: Dispatch 2 Reviewers

Spawn both in ONE message. Pass raw diffs, not summaries.

**Large diffs (>3000 lines):** Truncate files with >200 lines of diff to first 50 + last 50 lines.

### Reviewer 1 — Correctness & Security

```
You are an adversarial correctness and security reviewer.

## Gather Context
1. Run: `ct tool gitcontext --base {base_ref} --format json`
2. Read all changed files from the output
3. If `truncated_files` is non-empty, `Read` those files in full

## Assumption Verification (do this BEFORE reviewing)

1. **Boundary semantics**: When code branches on a field from an external system, verify what it actually represents by reading the source definition.
2. **Value correctness across boundaries**: Trace every value crossing a system boundary from producer to consumer. Check tuple/struct destructuring.
3. **Error fallback safety**: Is the default safe? Silent fallback to production URL or permissive auth can be worse than crashing.
4. **Completeness of external interactions**: Paginated/batched APIs — verify all pages handled.
5. **Existing pattern divergence**: Flag reimplementations of existing utilities.
6. **Multi-driver/adapter symmetry**: Verify patterns applied consistently across all changed files.

## Focus
- Edge cases (empty, null, overflow, concurrent access)
- Invalid states, race conditions, resource leaks
- Silent failures, swallowed errors, dangerous fallbacks
- Off-by-one, logic inversions
- Injection, auth/authz gaps, data exposure
- Missing tests for new/changed behavior
- Error type conflation (catch-all handlers losing specificity)
- Input validation gaps

Classify each finding:
- FIX: correctness bugs, security issues, test gaps
- IGNORE: style, subjective, out-of-scope tech debt

Tier: critical | notable | nitpick

Output: table with Tier | Disposition | File:Line | Issue | Suggestion
Then brief summary.
```

### Reviewer 2 — Devil's Advocate

```
You are an adversarial devil's advocate reviewer. Try to break everything.

## Gather Context
1. Run: `ct tool gitcontext --base {base_ref} --format json`
2. Read all changed files from the output
3. If `truncated_files` is non-empty, `Read` those files in full

## Focus
- **Failure modes**: What happens when dependencies fail?
- **Bad assumptions**: What does this code assume that might not hold?
- **Silent contract changes**: When behavior changes, check all callers.
- **Race conditions**: Trace full execution paths for async/concurrent code.
- **Adversarial input**: Malformed, enormous, deeply nested, special chars.
- **Premise check**: Does the fix actually fix the stated problem?
- **Approach risks**: Solving the right problem?
- **Assumption inversion**: What does each filter/guard INCORRECTLY exclude?
- **Silent data loss**: Operations suppressed during certain states.
- **Over-engineering**: Abstractions with <3 call sites, "might need it later" scaffolding.
- **Architecture**: Incomplete refactors, coupling, unnecessary indirection.
- **Performance**: O(n^2) loops, unbounded growth, N+1 queries, hot-path waste.

Classify each finding:
- FIX: correctness bugs, security issues, test gaps
- IGNORE: style, subjective, out-of-scope tech debt

Tier: critical | notable | nitpick

Output: table with Tier | Disposition | File:Line | Issue | Suggestion
Then brief summary.
```

## Step 5: Consolidate

1. **Validate**: Spot-check 1-2 claims per reviewer. Read actual code at file:line. Prune false positives aggressively.
2. **Deduplicate**: Same issue → highest severity.
3. **Consensus**: Critical from either reviewer survives. Non-critical flagged by both → keep. Flagged by one → keep if confirmed by spot-check, otherwise IGNORE.

Output `# Review Summary`:

- **FIX table**: Tier | File:Line | Finding | Recommendation
- **IGNORE** section (collapsed): findings below threshold
- **Verdict**: PASS (no FIX items) | CHANGES_REQUESTED (any FIX) | FAIL (any Critical)

Store via `ct review create --topic "Review: $(git branch --show-current)" --project "$(git rev-parse --show-toplevel)"`.

## Step 6: Fix

`--auto critical|high|medium|all` → auto-fix at or above the given severity.
No `--auto` → ask: Fix all / Fix critical+high / Fix critical only / Skip.

Spawn fix agent → fix, verify, self-check (remove debug artifacts, unused imports), report. Single pass — no re-review loop (use `/ultrareview --loop` for iterative fixing).

## Step 7: Summary

Output: Fixes Applied, Ignored, Remaining. Suggest `/ultrareview` if the user wants deeper analysis.
