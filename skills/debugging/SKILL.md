---
name: debugging
description: "Triggers: bugs, test failures, unexpected behavior, 'why is this failing', 'not working', 'broken'. Use BEFORE proposing fixes."
---

# Systematic Debugging

Random fixes waste time + create new bugs.

**Iron Law:** NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST.

## 4 Phases

### Phase 1: Root Cause Investigation

1. Read errors carefully - stack traces, line numbers
2. Reproduce consistently - exact steps
3. Check recent changes - git diff, new deps
4. Gather evidence at component boundaries
5. Trace data flow to source

### Phase 2: Pattern Analysis

1. Find working examples in codebase
2. Read reference implementation COMPLETELY
3. List ALL differences
4. Understand dependencies, config, assumptions

### Phase 3: Hypothesis + Testing

1. Form single hypothesis: "X is root cause because Y"
2. SMALLEST possible change to test
3. One variable at a time
4. Worked → Phase 4. Didn't → NEW hypothesis

### Phase 4: Implementation

1. Create failing test - use Skill tool to invoke `tdd`
2. Single fix - ONE change
3. Verify - test passes, no regressions
4. Fix doesn't work? < 3 attempts → Phase 1. >= 3 → architectural issue

## 3+ Fixes = Question Architecture

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Document concern in beads notes, attempt one structural fix, surface to caller." || echo "STOP. Discuss with human before more fixes."`

## Red Flags → Return to Phase 1

- "Quick fix for now"
- "Just try changing X"
- "Add multiple changes, run tests"
- "Skip the test"
- "It's probably X"
- Proposing solutions before tracing data flow

## Skill Composition

| When | Invoke |
|------|--------|
| Writing tests | `use Skill tool to invoke tdd` |
| Verifying fix | `use Skill tool to invoke verification-before-completion` |
| 3+ plausible root causes | `use Skill tool to invoke team-debug` |
