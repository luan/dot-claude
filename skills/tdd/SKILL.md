---
name: tdd
description: "DEFAULT for all implementation. Triggers: 'implement', 'build', 'add feature', 'fix bug', 'write code', starting any coding task. Iron law: no production code without failing test first. Skip only for explicit prototypes."
---

# Test-Driven Development

Write test → watch fail → minimal code → pass → refactor.

**Iron Law:** No production code without failing test first. Violate → delete code, start over.

## Red-Green-Refactor Cycle

### RED - Write Failing Test

One minimal test for desired behavior.

```typescript
test('rejects empty email', async () => {
  const result = await submitForm({ email: '' });
  expect(result.error).toBe('Email required');
});
```

Requirements: one behavior, clear name, real code (mocks only if unavoidable).

### Verify RED

**MANDATORY.**

```bash
npm test path/to/test.test.ts
```

Confirm: test fails (not errors), failure = feature missing (not typo). Passes immediately? Fix test.

### GREEN - Minimal Code

Simplest code to pass. No extras, no refactoring yet.

### Verify GREEN

**MANDATORY.** Test passes + other tests pass + output pristine (no warnings).

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
| "TDD slows me down" | Faster than debugging production. |
| "No tests in existing code" | Improving it? Add tests for touched code. |

## Verification Checklist

- [ ] Every new function has test
- [ ] Watched each test fail before implementing
- [ ] Each failed for expected reason
- [ ] Wrote minimal code to pass
- [ ] All tests pass, output pristine
- [ ] Real code tested (minimal mocks)
- [ ] Edge cases + errors covered

Can't check all → skipped TDD → start over.

## Red Flags - STOP + Restart

Code before test, test passes immediately, can't explain failure, "just this once", "keep as reference", "already spent X hours", "this is different because..."

All mean: delete code, start over with TDD.

## When Stuck

| Problem | Solution |
|---------|----------|
| Don't know how to test | Write wished-for API. Ask human. |
| Test too complicated | Design too complicated. Simplify. |
| Must mock everything | Too coupled. Use DI. |
| Test setup huge | Extract helpers or simplify design. |

## Bug Fixes

Bug → write failing test reproducing it → TDD cycle → test proves fix + prevents regression. Never fix without test.

## See Also

- **debugging** → root cause investigation before fixing
- **verification-before-completion** → evidence before claiming done
