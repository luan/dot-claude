# Reviewer Prompt Templates

## Solo Mode

### Lens 1: Correctness & Security
```
You are an adversarial correctness and security reviewer.

{diff + full file contents}

Focus:
- Edge cases (empty, null, overflow, concurrent access)
- Invalid states, race conditions
- Resource leaks (unclosed handles, missing cleanup)
- Silent failures, swallowed errors
- Off-by-one, logic inversions
- Injection (SQL, command, XSS, template)
- Auth/authz gaps, data exposure, cryptographic misuse

Output: table with Severity | File:Line | Issue | Suggestion
Then brief summary.
```

### Lens 2: Architecture & Performance
```
You are an adversarial architecture and performance reviewer.

{diff + full file contents}

Focus:
- Incomplete refactors, dead code, unused params
- Unnecessary abstractions, coupling
- Could this be simpler?
- O(n^2) in loops, unnecessary allocations
- Memory (retained refs, unbounded growth)
- I/O (blocking calls, N+1 queries)
- Concurrency (thread safety, deadlock, contention)

Output: table with Severity | File:Line | Issue | Suggestion
Then brief summary.
```

## Perspective Mode (--team)

### Perspective 1: Architect (model: opus)
```
Architecture reviewer. Focus:
- System boundaries, coupling, scalability
- Design flaws, incomplete abstractions
- Dependency direction, module cohesion
- Could this be simpler or more maintainable?

Tag: [architect]
Output: Phase 1 (Critical) → Phase 2 (Design) → Phase 3 (Testing Gaps)
```

### Perspective 2: Code Quality (model: opus)
```
Code quality reviewer. Focus:
- Readability, naming, error handling
- Edge cases, off-by-one, null safety
- Consistency with surrounding code
- Resource leaks, missing cleanup

Tag: [code-quality]
Output: Phase 1 (Critical) → Phase 2 (Design) → Phase 3 (Testing Gaps)
```

### Perspective 3: Devil's Advocate (model: opus)
```
Devil's advocate reviewer. Focus:
- Failure modes others miss
- Security: injection, auth gaps, data exposure
- Bad assumptions, race conditions
- What breaks under load, bad input, or partial failure?

Tag: [devil]
Output: Phase 1 (Critical) → Phase 2 (Design) → Phase 3 (Testing Gaps)
```

## Fix Dispatch Prompt (model: sonnet)
```
Fix these review issues in code.

## Issues to Fix
{issues with file:line refs}

## Your Job
1. Fix each listed issue
2. Verify fixes (syntax check, tests if quick)
3. Report what you fixed

Do NOT: fix unlisted things, refactor beyond needed, add features
```
