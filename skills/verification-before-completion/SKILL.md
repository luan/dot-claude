---
name: verification-before-completion
description: Use before claiming work complete, fixed, or passing. Evidence before assertions - run verification, read output, THEN claim result.
---

# Verification Before Completion

Claiming work is complete without verification is dishonesty, not efficiency.

**Iron Law:** No completion claims without fresh verification evidence.

## The Gate

```
BEFORE claiming success/completion:

1. IDENTIFY: What command proves this claim?
2. RUN: Execute the FULL command (fresh, complete)
3. READ: Full output, check exit code, count failures
4. VERIFY: Does output confirm the claim?
   - NO → State actual status with evidence
   - YES → State claim WITH evidence
5. ONLY THEN: Make the claim

Skip any step = lying, not verifying
```

## Common Claims and Requirements

| Claim | Requires | Not Sufficient |
|-------|----------|----------------|
| "Tests pass" | Test command output: 0 failures | Previous run, "should pass" |
| "Build succeeds" | Build command: exit 0 | Linter passing |
| "Bug fixed" | Test original symptom: passes | "Code changed" |
| "Linter clean" | Linter output: 0 errors | Partial check |
| "Requirements met" | Line-by-line checklist | Tests passing |
| "Agent completed" | VCS diff shows changes | Agent reports "success" |

## Forbidden Patterns

Using before verification:
- "Should work now"
- "Probably passes"
- "Seems to be working"
- "Great!", "Perfect!", "Done!"
- "I'm confident it works"

**Instead:** Run verification → read output → state result with evidence.

## Verification Patterns

**Tests:**
```
✅ [Run npm test] "47/47 pass" → "All tests pass"
❌ "Should pass now"
```

**Build:**
```
✅ [Run cargo build] "exit 0" → "Build succeeds"
❌ "Linter passed so build should work"
```

**Regression test (TDD):**
```
✅ Write test → Run (pass) → Revert fix → Run (MUST FAIL) → Restore → Run (pass)
❌ "I've written a regression test"
```

**Requirements:**
```
✅ Re-read plan → Checklist each item → Report with evidence
❌ "Tests pass, must be complete"
```

**Agent delegation:**
```
✅ Agent reports → Check VCS diff → Verify changes exist → Report actual state
❌ Trust agent report blindly
```

## Common Rationalizations

| Excuse | Reality |
|--------|---------|
| "Should work now" | RUN the verification |
| "I'm confident" | Confidence ≠ evidence |
| "Just this once" | No exceptions |
| "Partial check is enough" | Partial proves nothing |
| "Agent said success" | Verify independently |
| "I'm tired" | Exhaustion ≠ excuse |

## When to Apply

**Always before:**
- Any success/completion claim
- Any expression of satisfaction
- Committing, PR creation, task completion
- Moving to next task
- Trusting agent delegation results

## Integration

- **tdd** skill: Verify RED-GREEN cycle
- **debugging** skill: Verify fix actually works
- **implement** skill: Verify each task completion
- **code-review** skill: Verify fixes before claiming done
