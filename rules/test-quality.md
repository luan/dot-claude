# Test Quality Standards

## The Gate

Every test must answer: **"What bug would this catch?"** No
realistic bug scenario = delete test.

## Banned Patterns

- **Tautology** — mock returns what you told it
- **Getter/setter** — testing that a builder/constructor stores values
- **Implementation mirroring** — test duplicates production formula
- **Constant echo** — `assert_eq!(MY_CONST, 42)` just restates the
  definition
- **Happy-path-only** — no error/edge/boundary tests
- **Coverage padding** — executes code without asserting outcomes
- **No-assertion smoke** — constructs object, asserts nothing

## What to Test

- Boundary conditions (empty, one, many, overflow)
- Error paths (invalid input, network failure, timeout, permission
  denied)
- State transitions (A→B allowed, A→C forbidden)
- Race conditions + ordering dependencies
- Integration between real components
- Round-trip invariants (serialize/deserialize, encode/decode)
- Algorithm correctness with known-answer values

## Mock Discipline

Mocks are last resort:

- Mock external services (network, filesystem, clock, third-party
  APIs)
- Do NOT mock thing you're testing
- Do NOT mock collaborators you own — use real implementation
- 3+ mocks in one test = design too coupled. Simplify.

## The Deletion Test

After writing test: "If I delete this + introduce bug, will any
other test catch it?" If yes, redundant. Delete it.

## Pre-Commit Checklist

Before writing any test:

1. State the bug scenario in one sentence
2. If the "bug" is "field doesn't store value" → don't write it
3. If the assertion mirrors the production formula → use a
   known-answer instead
4. If it tests a constant → don't write it
5. If removing the test + breaking the code would be caught by
   the compiler → don't write it
