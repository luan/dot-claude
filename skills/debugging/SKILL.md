---
name: debugging
description: "Systematic root cause investigation before proposing fixes. Triggers: bugs, test failures, unexpected behavior, 'why is this failing', 'not working', 'broken'. Do NOT use when: the bug is already identified and the fix is obvious — use /fix instead."
---

# Systematic Debugging

Random fixes waste time + create new bugs.

**Iron Law:** NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST.

## 4 Phases

### Phase 1: Root Cause Investigation

1. Read errors carefully — stack traces, line numbers
2. Reproduce consistently — exact steps. Intermittent? Add logging at component boundaries to capture state on next occurrence.
3. Check recent changes — git diff, new deps
4. Gather evidence at component boundaries
5. Trace data flow to source

### Phase 2: Pattern Analysis

1. Find working examples in codebase
2. Read reference implementation COMPLETELY
3. List ALL differences
4. Understand dependencies, config, assumptions

### Phase 3: Hypothesis + Testing

1. Form single hypothesis: "X is root cause because Y"
1b. **Oracle test (when applicable):** Compare current behavior against a known-good baseline to confirm the bug and isolate its scope. Options: available worktree from pool (`git worktree list` — detached HEAD = available), reference implementation, or reduced test case with known-correct output. Creating a new worktree is expensive — justify only when the bug is non-obvious and affects multiple components. Return worktree to detached HEAD when done.
2. **3+ plausible hypotheses:** STOP — spawn parallel subagents (one per hypothesis, each traces one, reports evidence for/against). At 3+ hypotheses, sequential testing wastes time because each test cycle is slow; parallel investigation covers more ground faster.
3. SMALLEST possible change to test
4. One variable at a time
5. Worked → Phase 4. Didn't → NEW hypothesis (if 3rd, parallelize per step 2)

**Exit condition:** If 3 hypotheses all fail, re-enter Phase 1 with broader scope — re-examine assumptions and widen the investigation boundary. If Phase 1 re-entry also stalls, escalate as architectural (see "3+ Fixes" below).

### Phase 4: Implementation

1. Create failing test that reproduces the bug
2. Single fix — ONE change
3. Verify — test passes, no regressions
4. Fix doesn't work? < 3 attempts → Phase 1. >= 3 → architectural issue

## 3+ Fixes = Question Architecture

Repeated fixes without root cause understanding signal a design problem, not a surface bug. Continuing tactical patches compounds tech debt and masks the real issue.

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Document concern in task description, attempt one structural fix, surface to caller." || echo "STOP. Discuss with human before more fixes."`

## Red Flags → Return to Phase 1

Listed from most dangerous (zero investigation) to least:

- **Proposing solutions before tracing data flow** — guessing, not debugging
- **"It's probably X"** — hypothesis without evidence
- **"Add multiple changes, run tests"** — shotgun debugging masks root cause
- **"Quick fix for now" / "Just try changing X"** — deferral that compounds
- **"Skip test"** — removing the only verification signal
