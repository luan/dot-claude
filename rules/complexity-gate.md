# Complexity Gate

Before introducing any new abstraction (class, wrapper, helper, utility, trait, protocol, manager, service, provider, factory), answer: **"What happens if I inline this instead?"**

If inlining means duplicating 3 or fewer lines — inline it. The abstraction isn't earning its keep.

New abstractions require at least one of:
- 3+ distinct call sites that exist today (not "might exist later")
- Encapsulating a dangerous/tricky operation where getting it wrong has consequences
- Hiding a dependency boundary (network, disk, FFI, third-party API)

If none apply, write the straightforward code. A 10-line function is better than a 4-file abstraction hierarchy that does the same thing.

## Worker instructions

Workers must include in their completion report: number of new types/abstractions introduced. Zero is the ideal number. Any nonzero count must list each one with its justification.
