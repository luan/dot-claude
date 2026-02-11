# Test Quality Standards

## The Gate

Every test must answer: **"What bug would this catch?"** No realistic bug scenario = delete the test.

## Banned Patterns

- **Tautology** — mock returns what you told it to
- **Getter/setter** — testing language features
- **Implementation mirroring** — test duplicates production formula
- **Happy-path-only** — no error/edge/boundary tests
- **Coverage padding** — executes code without asserting outcomes

```
// BAD: same formula in test and production
expect(total(10, 5, 2)).toBe(10 * 5 + 2)
// GOOD: known-answer test
expect(total(10, 5, 2)).toBe(52)
```

## What to Test Instead

- Boundary conditions (empty, one, many, overflow)
- Error paths (invalid input, network failure, timeout, permission denied)
- State transitions (A→B allowed, A→C forbidden)
- Race conditions and ordering dependencies
- Integration between real components

## Mock Discipline

Mocks are a last resort:
- Mock external services (network, filesystem, clock, third-party APIs)
- Do NOT mock the thing you're testing
- Do NOT mock collaborators you own — use the real implementation
- 3+ mocks in one test = the design is too coupled. Simplify.

## The Deletion Test

After writing a test, ask: "If I delete this test and introduce a bug, will any other test catch it?" If yes, this test is redundant. Delete it.
