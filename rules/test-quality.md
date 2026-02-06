# Test Quality Standards

## The Gate

Every test must answer: **"What bug would this catch?"** No realistic bug scenario = delete the test.

## Banned Patterns

**Tautology tests** — testing that mocks return what you told them to:
```
// BAD: proves nothing
mock_db.get_user = () => { name: 'Alice' }
expect(service.get_user()).toEqual({ name: 'Alice' })
```

**Getter/setter tests** — testing language features:
```
// BAD: tests that assignment works
user.name = 'Bob'
expect(user.name).toBe('Bob')
```

**Implementation mirroring** — test duplicates the production formula:
```
// BAD: same formula in test and production
expect(total(10, 5, 2)).toBe(10 * 5 + 2)
// GOOD: known-answer test
expect(total(10, 5, 2)).toBe(52)
```

**Happy-path-only** — only testing success when failure modes exist:
```
// BAD: what about empty input? invalid data? timeouts?
expect(process([1, 2])).toEqual([2, 4])
// Must also test: process([]), process(null), process(huge_array)
```

**Coverage padding** — tests that execute code without asserting outcomes:
```
// BAD: no assertion on correctness
result = process_data([1, 2, 3])
// just checking it doesn't crash is not a test
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
