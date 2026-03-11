---
name: debugging
description: "Systematic root cause investigation before proposing fixes. Triggers: bugs, test failures, unexpected behavior, 'why is this failing', 'not working', 'broken'. Do NOT use when: the bug is already identified and the fix is obvious — use /triage instead."
---

# Systematic Debugging

Fixing without understanding creates new bugs. Investigate first, fix second.

## Phase 1: Investigate

1. Read errors — stack traces, line numbers, exact messages
2. Reproduce — exact steps. Intermittent? Log at component boundaries.
3. Check recent changes — `git diff`, new deps
4. Trace data flow from symptom to source

## Phase 2: Compare

1. Find working examples in codebase
2. Read reference implementation fully
3. Diff all differences — deps, config, assumptions

## Phase 3: Hypothesize + Test

1. Form hypothesis: "X causes this because Y"
2. **Oracle test (when useful):** Compare against known-good baseline — available worktree (`git worktree list`, detached HEAD = available), reference impl, or reduced test case. Only create new worktrees for non-obvious multi-component bugs. Return to detached HEAD when done.
3. **3+ plausible hypotheses → parallelize.** Spawn one subagent per hypothesis (each traces one, reports evidence for/against). Sequential testing wastes cycles when each test is slow.
4. Smallest possible change to test. One variable at a time.
5. Confirmed → Phase 4. Refuted → new hypothesis (3rd+ → parallelize per step 3)

**Dead end:** 3 hypotheses all fail → re-enter Phase 1 with broader scope. Still stuck → escalate as architectural.

## Phase 4: Fix

1. Write failing test that reproduces the bug
2. Single fix — one change
3. Verify — test passes, no regressions
4. Fix fails? < 3 attempts → Phase 1. >= 3 → architectural issue

## 3+ Fixes = Architectural Problem

Repeated patches without root cause understanding signal a design problem. Tactical fixes compound debt and mask the real issue.

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Document concern in task description, attempt one structural fix, surface to caller." || echo "STOP. Discuss with human before more fixes."`

## Red Flags → Back to Phase 1

- Proposing fixes before tracing data flow
- "It's probably X" without evidence
- Multiple changes at once (shotgun debugging)
- "Quick fix for now" / "Just try changing X"
- Skipping or deleting tests
