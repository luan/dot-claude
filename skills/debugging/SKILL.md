---
name: debugging
description: "Systematic root cause investigation before proposing fixes. Triggers: bugs, test failures, unexpected behavior, 'why is this failing', 'not working', 'broken'."
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
1b. **Oracle test (when applicable):** Compare current behavior against known-good baseline. Use available worktree from pool (`git worktree list` — detached HEAD = available), check out last green commit there, run same test, diff outputs. Creating new worktree is expensive — ask user first. Return to detached HEAD when done. Also consider: reference implementation as oracle, or reduced test case with known-correct output.
2. **If you identify 3+ plausible hypotheses:** STOP — spawn parallel subagents (`model="sonnet"`, one per hypothesis) to investigate simultaneously. Each subagent traces one hypothesis, reports evidence for/against.
3. SMALLEST possible change to test
4. One variable at a time
5. Worked → Phase 4. Didn't → NEW hypothesis (if this is your 3rd hypothesis, parallelize per step 2)

### Phase 4: Implementation

1. Create failing test that reproduces the bug
2. Single fix - ONE change
3. Verify - test passes, no regressions
4. Fix doesn't work? < 3 attempts → Phase 1. >= 3 → architectural issue

## 3+ Fixes = Question Architecture

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Document concern in task description, attempt one structural fix, surface to caller." || echo "STOP. Discuss with human before more fixes."`

## Red Flags → Return to Phase 1

- "Quick fix for now"
- "Just try changing X"
- "Add multiple changes, run tests"
- "Skip test"
- "It's probably X"
- Proposing solutions before tracing data flow

