---
name: debugging
description: "Use when encountering bugs, test failures, or unexpected behavior. ALWAYS before proposing fixes."
---

# Systematic Debugging

Random fixes waste time + create new bugs. Quick patches mask underlying issues.

## Iron Law

```
NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST
```

Phase 1 incomplete → cannot propose fixes.

## When to Use

ANY technical issue: test failures, bugs, unexpected behavior, performance problems, build failures, integration issues.

**Especially when:**
- Under time pressure (emergencies make guessing tempting)
- "Quick fix" seems obvious
- Already tried multiple fixes
- Don't fully understand issue

## 4 Phases

Complete each before proceeding.

### Phase 1: Root Cause Investigation

**BEFORE any fix:**

1. **Read errors carefully** - stack traces, line numbers, error codes
2. **Reproduce consistently** - exact steps, reliable trigger
3. **Check recent changes** - git diff, new deps, config changes
4. **Gather evidence** (multi-component systems):
   - Log data at each component boundary
   - Run once → reveals WHERE it breaks
   - Then investigate failing component
5. **Trace data flow** - where does bad value originate? Keep tracing up to source

### Phase 2: Pattern Analysis

1. Find working examples in same codebase
2. Read reference implementation COMPLETELY (don't skim)
3. List ALL differences between working + broken
4. Understand dependencies, config, assumptions

### Phase 3: Hypothesis + Testing

1. Form single hypothesis: "X is root cause because Y"
2. SMALLEST possible change to test
3. One variable at a time
4. Worked → Phase 4. Didn't → NEW hypothesis (don't stack fixes)

### Phase 4: Implementation

1. **Create failing test** - use `tdd` skill
2. **Single fix** - ONE change, no "while I'm here" improvements
3. **Verify** - test passes, no regressions
4. **Fix doesn't work?**
   - Count attempts
   - < 3 → return to Phase 1
   - **>= 3 → STOP, question architecture**

## 3+ Fixes = Question Architecture

Pattern indicating architectural problem:
- Each fix reveals new coupling/shared state
- Fixes require massive refactoring
- Each fix creates symptoms elsewhere

**STOP. Discuss with human before more fixes.** Wrong architecture, not failed hypothesis.

## Red Flags → STOP, Return to Phase 1

- "Quick fix for now, investigate later"
- "Just try changing X and see"
- "Add multiple changes, run tests"
- "Skip the test, manually verify"
- "It's probably X, let me fix that"
- "Don't fully understand but might work"
- Proposing solutions before tracing data flow
- "One more fix attempt" (already tried 2+)

## Common Rationalizations

| Excuse | Reality |
|--------|---------|
| "Issue is simple" | Simple bugs have root causes. Process is fast. |
| "Emergency, no time" | Systematic is FASTER than thrashing. |
| "Just try this first" | First fix sets pattern. Do it right. |
| "Test after confirming" | Untested fixes don't stick. |
| "Multiple fixes saves time" | Can't isolate what worked. |
| "Reference too long" | Partial understanding → bugs. |
| "I see the problem" | Symptoms =/= root cause. |
| "One more attempt" (2+ failures) | 3+ = architectural. Question pattern. |

## Quick Reference

| Phase | Key | Success |
|-------|-----|---------|
| 1. Root Cause | Read errors, reproduce, gather evidence | Understand WHAT + WHY |
| 2. Pattern | Find working examples, compare | Identify differences |
| 3. Hypothesis | Form theory, test minimally | Confirmed or new hypothesis |
| 4. Implement | Create test, fix, verify | Bug resolved, tests pass |

## See Also

- **tdd** skill → test-first implementation in Phase 4
