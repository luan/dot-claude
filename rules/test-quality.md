# Test Quality Standards

## The Gate

Every test must answer: **"What bug would this catch?"** No realistic bug scenario = delete test.

## Banned Patterns

- **Tautology** — mock returns what you told it
- **Getter/setter** — testing language features
- **Implementation mirroring** — test duplicates production formula
- **Happy-path-only** — no error/edge/boundary tests
- **Coverage padding** — executes code without asserting outcomes

```
// BAD: same formula in test + production
expect(total(10, 5, 2)).toBe(10 * 5 + 2)
// GOOD: known-answer test
expect(total(10, 5, 2)).toBe(52)
```

## What to Test

- Boundary conditions (empty, one, many, overflow)
- Error paths (invalid input, network failure, timeout, permission denied)
- State transitions (A→B allowed, A→C forbidden)
- Race conditions + ordering dependencies
- Integration between real components

## Mock Discipline

Mocks are last resort:
- Mock external services (network, filesystem, clock, third-party APIs)
- Do NOT mock thing you're testing
- Do NOT mock collaborators you own — use real implementation
- 3+ mocks in one test = design too coupled. Simplify.

## The Deletion Test

After writing test: "If I delete this + introduce bug, will any other test catch it?" If yes, redundant. Delete it.
