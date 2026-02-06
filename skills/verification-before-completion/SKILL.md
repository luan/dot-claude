---
name: verification-before-completion
description: Use before claiming work complete, fixed, or passing. Evidence before assertions - run verification, read output, THEN claim result.
---

# Verification Before Completion

Claiming completion without verification = dishonesty, not efficiency.

**Iron Law:** No completion claims without fresh verification evidence.

## The Gate

```
BEFORE claiming success/completion:

1. IDENTIFY: What command proves this claim?
2. RUN: Execute FULL command (fresh, complete)
3. READ: Full output, check exit code, count failures
4. VERIFY: Output confirms claim?
   - NO → State actual status with evidence
   - YES → State claim WITH evidence
5. ONLY THEN: Make claim

Skip any step = lying, not verifying
```

## Claims + Requirements

| Claim | Requires | Not Sufficient |
|-------|----------|----------------|
| "Tests pass" | Test output: 0 failures | Previous run, "should pass" |
| "Build succeeds" | Build: exit 0 | Linter passing |
| "Bug fixed" | Original symptom test passes | "Code changed" |
| "Linter clean" | Linter output: 0 errors | Partial check |
| "Requirements met" | Line-by-line checklist | Tests passing |
| "Agent completed" | VCS diff shows changes | Agent reports "success" |

## Forbidden Patterns

Never use before verification: "Should work now" / "Probably passes" / "Seems to be working" / "Great!" / "Perfect!" / "Done!" / "I'm confident it works"

**Instead:** Run verification → read output → state result with evidence.

## Verification Patterns

```
Tests:     ✅ [Run npm test] "47/47 pass" → claim    ❌ "Should pass now"
Build:     ✅ [Run cargo build] "exit 0" → claim      ❌ "Linter passed so build should work"
Regression: ✅ test → pass → revert fix → MUST FAIL → restore → pass  ❌ "I've written a regression test"
Reqs:      ✅ Re-read plan → checklist each → evidence ❌ "Tests pass, must be complete"
Delegation: ✅ Agent reports → check VCS diff → verify  ❌ Trust agent report blindly
```

## Common Rationalizations

| Excuse | Reality |
|--------|---------|
| "Should work now" | RUN verification |
| "I'm confident" | Confidence ≠ evidence |
| "Just this once" | No exceptions |
| "Partial check is enough" | Partial proves nothing |
| "Agent said success" | Verify independently |

## When to Apply

Always before: success/completion claims, expressions of satisfaction, commits, PR creation, task completion, moving to next task, trusting agent delegation.

## Integration

- **tdd**: verify RED-GREEN cycle
- **debugging**: verify fix works
- **implement**: verify each task completion
- **review**: verify fixes before claiming done
