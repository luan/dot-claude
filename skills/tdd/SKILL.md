---
name: tdd
description: "Use when implementing features/bugfixes. Iron law: no production code without failing test first."
---

# Test-Driven Development

Write test first → watch fail → minimal code → pass → refactor.

**Iron Law:** No production code without failing test first. Violate → delete code, start over.

## Red-Green-Refactor Cycle

### RED - Write Failing Test

One minimal test showing desired behavior.

```typescript
test('rejects empty email', async () => {
  const result = await submitForm({ email: '' });
  expect(result.error).toBe('Email required');
});
```

Requirements: one behavior, clear name, real code (mocks only if unavoidable).

### Verify RED

**MANDATORY. Never skip.**

```bash
npm test path/to/test.test.ts
```

Confirm:
- Test fails (not errors)
- Failure = feature missing (not typo)

Test passes? Testing existing behavior. Fix test.

### GREEN - Minimal Code

Simplest code to pass. No extra features, no refactoring yet.

### Verify GREEN

**MANDATORY.**

- Test passes
- Other tests still pass
- Output pristine (no warnings)

### REFACTOR

After green only: remove duplication, improve names, extract helpers. Keep tests green.

### Repeat

Next failing test → next feature.

## When to Use

**Always:** new features, bug fixes, refactoring, behavior changes.

**Exceptions (ask human):** throwaway prototypes, generated code, config files.

## Common Rationalizations

| Excuse | Reality |
|--------|---------|
| "Too simple to test" | Simple code breaks. Test takes 30s. |
| "I'll test after" | Tests passing immediately prove nothing. |
| "Already manually tested" | Ad-hoc != systematic. Can't re-run. |
| "Deleting X hours is wasteful" | Sunk cost. Unverified code = tech debt. |
| "Keep as reference" | You'll adapt it = testing after. Delete means delete. |
| "Need to explore first" | Fine. Throw away exploration, TDD from scratch. |
| "Test hard = skip TDD" | Hard to test = hard to use. Simplify design. |
| "TDD slows me down" | TDD faster than debugging production. |
| "Existing code has no tests" | Improving it? Add tests for touched code. |

## Verification Checklist

Before marking complete:

- [ ] Every new function has test
- [ ] Watched each test fail before implementing
- [ ] Each test failed for expected reason
- [ ] Wrote minimal code to pass
- [ ] All tests pass
- [ ] Output pristine
- [ ] Real code tested (minimal mocks)
- [ ] Edge cases + errors covered

Can't check all? Skipped TDD. Start over.

## Red Flags - STOP and Restart

- Code before test
- Test passes immediately
- Can't explain why test failed
- "Just this once"
- "Keep as reference"
- "Already spent X hours"
- "This is different because..."

All mean: delete code, start over with TDD.

## When Stuck

| Problem | Solution |
|---------|----------|
| Don't know how to test | Write wished-for API. Ask human. |
| Test too complicated | Design too complicated. Simplify. |
| Must mock everything | Code too coupled. Use DI. |
| Test setup huge | Extract helpers or simplify design. |

## Bug Fixes

Bug found → write failing test reproducing it → TDD cycle → test proves fix + prevents regression.

Never fix bugs without test.

## See Also

- **debugging** skill → root cause investigation before fixing
- **verification-before-completion** → evidence before claiming done
