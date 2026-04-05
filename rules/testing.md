# Test Quality

## Gate

Every test must answer: **"What bug would this catch?"** No realistic scenario = delete.

## TDD

1. Write failing test
2. Confirm red
3. Minimal implementation to pass
4. Confirm green
5. Refactor, stay green

No test infrastructure in project? Note it, proceed without tests.

## Banned

- **Tautology** — mock returns what you told it
- **Getter/setter** — compiler catches this
- **Implementation mirror** — test duplicates production formula
- **Constant echo** — `assert_eq!(MY_CONST, 42)` restates definition
- **Happy-path-only** — bugs live at boundaries
- **Coverage padding** — executes without asserting
- **No-assertion smoke** — constructs object, asserts nothing

## What to Test

Boundaries, error paths, state transitions, race conditions, real integrations, round-trip invariants, known-answer algorithm checks.

## Mocks

Last resort. Every mock removes a real integration.

- Mock external services only (network, filesystem, clock, third-party APIs)
- Never mock the thing under test
- Never mock collaborators you own
- 3+ mocks = design too coupled — simplify the design

## Speed

Fast feedback is non-negotiable.

- No network, disk, or subprocesses in unit tests
- No sleeps — synchronize on events
- No shared mutable state between tests

Unit tests stay fast. Integration tests run separately.

## Deletion Test

"If I delete this test and break the code, does another test catch it?" Yes = redundant. Delete.

## Pre-Commit

Before writing any test:

1. State the bug scenario in one sentence
2. "Field doesn't store value" → don't write it
3. Assertion mirrors production formula → use known-answer
4. Tests a constant → don't write it
5. Compiler catches it → don't write it
