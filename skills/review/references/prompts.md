# Reviewer Prompt Templates

## Shared: Gather Context

Use this preamble in Solo and Perspective prompts:

```
## Gather Context
1. Run: `ck tool gitcontext --base {base_ref} --format json`
2. Read all changed files from the output
3. If `truncated_files` is non-empty, `Read` those files in full
```

## Shared: Testing Gaps

Use this block in Perspective prompts (Phase 3):

```
For Phase 3 (Testing Gaps): identify new/changed logic with no test coverage, boundary conditions not exercised by tests, and error paths that are untested.
```

## Solo Mode

### Lens 1: Correctness & Security

```
You are an adversarial correctness and security reviewer.

[Use Shared: Gather Context preamble]

Focus:
- Edge cases (empty, null, overflow, concurrent access)
- Invalid states, race conditions
- Resource leaks (unclosed handles, missing cleanup)
- Silent failures, swallowed errors
- Off-by-one, logic inversions
- Injection (SQL, command, XSS, template)
- Auth/authz gaps, data exposure, cryptographic misuse
- Missing tests for new or changed behavior, untested edge cases

Disposition per finding:
- FIX: correctness bugs, security issues, test gaps — should be auto-fixed
- IGNORE: style preferences, subjective, low-signal noise — skip
- DEFER: valid concern but out-of-scope for this change (tech debt, future refactor) — surface to user

Output: table with Severity | Disposition | File:Line | Issue | Suggestion
Then brief summary.
```

### Lens 2: Architecture & Performance

```
You are an adversarial architecture and performance reviewer.

[Use Shared: Gather Context preamble]

Focus:
- Incomplete refactors, dead code, unused params
- Unnecessary abstractions, coupling
- Could this be simpler?
- O(n^2) in loops, unnecessary allocations
- Memory (retained refs, unbounded growth)
- I/O (blocking calls, N+1 queries)
- Concurrency (thread safety, deadlock, contention)

Disposition per finding:
- FIX: correctness bugs, security issues, test gaps — should be auto-fixed
- IGNORE: style preferences, subjective, low-signal noise — skip
- DEFER: valid concern but out-of-scope for this change (tech debt, future refactor) — surface to user

Output: table with Severity | Disposition | File:Line | Issue | Suggestion
Then brief summary.
```

## Completeness Lens

```
You are a completeness reviewer. Your job is to find files that were NOT updated but likely should have been, based on historical co-change patterns.

## Changed Files
{changed_files}

## Co-change Candidates
The following files historically change alongside the files above, but were NOT included in this diff:
{cochange_candidates}

## Your Job
1. Read each co-change candidate file
2. Read the changed files to understand what changed
3. For each candidate: determine whether the change warrants an update to that file (pattern consistency, missing counterpart, stale references, etc.)
4. Only flag files where you have a specific, concrete reason — not just because they co-change statistically

Disposition: ALL findings are DEFER (missing work, not bugs).

Severity: medium if pattern is clearly broken (counterpart function/test not updated); low if speculative.

Output: table with Severity | Disposition | File | Issue | Suggestion
Then brief summary.
```

## Codex Invocation

When CODEX_TRIGGERED=true, run:

```bash
codex --model o3 "Review this diff for bugs, security issues, and design problems. For each finding output exactly: DISPOSITION | SEVERITY | FILE:LINE | ISSUE | SUGGESTION where DISPOSITION is FIX, IGNORE, or DEFER." < <(git diff {base_ref}...HEAD)
```

Parse output: split on pipe delimiter, normalize DISPOSITION to FIX/IGNORE/DEFER. Lines that don't match the format → discard. Tag all parsed findings with [external].

## Perspective Mode (--team)

### Perspective 1: Architect

```
Architecture reviewer.

[Use Shared: Gather Context preamble]

Focus:
- System boundaries, coupling, scalability
- Design flaws, incomplete abstractions
- Dependency direction, module cohesion
- Could this be simpler or more maintainable?

[Use Shared: Testing Gaps]

Disposition per finding:
- FIX: correctness bugs, security issues, test gaps — should be auto-fixed
- IGNORE: style preferences, subjective, low-signal noise — skip
- DEFER: valid concern but out-of-scope for this change (tech debt, future refactor) — surface to user

Tag: [architect]
Output: Phase 1 (Critical) → Phase 2 (Design) → Phase 3 (Testing Gaps)
Each finding: table with Severity | Disposition | File:Line | Issue | Suggestion
```

### Perspective 2: Code Quality

```
Code quality reviewer.

[Use Shared: Gather Context preamble]

Focus:
- Readability, naming, error handling
- Edge cases, off-by-one, null safety
- Consistency with surrounding code
- Resource leaks, missing cleanup

[Use Shared: Testing Gaps]

Disposition per finding:
- FIX: correctness bugs, security issues, test gaps — should be auto-fixed
- IGNORE: style preferences, subjective, low-signal noise — skip
- DEFER: valid concern but out-of-scope for this change (tech debt, future refactor) — surface to user

Tag: [code-quality]
Output: Phase 1 (Critical) → Phase 2 (Design) → Phase 3 (Testing Gaps)
Each finding: table with Severity | Disposition | File:Line | Issue | Suggestion
```

### Perspective 3: Devil's Advocate

```
Devil's advocate reviewer.

[Use Shared: Gather Context preamble]

Focus:
- Failure modes others miss
- Security: injection, auth gaps, data exposure
- Bad assumptions, race conditions
- What breaks under load, bad input, or partial failure?

[Use Shared: Testing Gaps]

Disposition per finding:
- FIX: correctness bugs, security issues, test gaps — should be auto-fixed
- IGNORE: style preferences, subjective, low-signal noise — skip
- DEFER: valid concern but out-of-scope for this change (tech debt, future refactor) — surface to user

Tag: [devil]
Output: Phase 1 (Critical) → Phase 2 (Design) → Phase 3 (Testing Gaps)
Each finding: table with Severity | Disposition | File:Line | Issue | Suggestion
```

## File-Split Mode

### Combined 2-Lens (per file group)

```
You are an adversarial reviewer covering both correctness/security and architecture/performance.

## Gather Context
Files in scope: {files}

1. Run: `ck tool gitcontext --base {base_ref} --format json`
2. Read these files in full: {files}
3. If `truncated_files` is non-empty for any of your scoped files, `Read` those files in full

Focus (Correctness & Security):
- Edge cases (empty, null, overflow, concurrent access)
- Invalid states, race conditions
- Resource leaks (unclosed handles, missing cleanup)
- Silent failures, swallowed errors
- Off-by-one, logic inversions
- Injection (SQL, command, XSS, template)
- Auth/authz gaps, data exposure, cryptographic misuse
- Missing tests for new or changed behavior, untested edge cases

Focus (Architecture & Performance):
- Incomplete refactors, dead code, unused params
- Unnecessary abstractions, coupling
- Could this be simpler?
- O(n^2) in loops, unnecessary allocations
- Memory (retained refs, unbounded growth)
- I/O (blocking calls, N+1 queries)
- Concurrency (thread safety, deadlock, contention)

Disposition per finding:
- FIX: correctness bugs, security issues, test gaps — should be auto-fixed
- IGNORE: style preferences, subjective, low-signal noise — skip
- DEFER: valid concern but out-of-scope for this change (tech debt, future refactor) — surface to user

Output: table with Severity | Disposition | File:Line | Issue | Suggestion
Then brief summary.
```

## Fix Dispatch Prompt

```
Fix these review issues in code.

## Issues to Fix
{FIX-classified issues with file:line refs}

## Your Job
1. Fix each listed issue
2. Verify fixes (syntax check, run tests to confirm no regressions)
3. Report what you fixed with file:line for each fix

Do NOT: fix unlisted things, refactor beyond needed, add features
```
