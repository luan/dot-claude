# Deduplication Mandate

Before writing new code that does X, search the codebase for existing code that already does X.

## Required search before creating

- **New function/method**: Grep for keywords from the operation (e.g., before writing `retryWithBackoff`, search for `retry`, `backoff`, `attempt`)
- **New type/struct/class**: Search for types with similar fields or purpose
- **New utility/helper**: Search `utils/`, `helpers/`, `common/`, `shared/`, `lib/` directories
- **New constant/config**: Search for the value itself and synonymous names

## What counts as "existing"

- Same logic, different name — use the existing one, rename if needed
- 80% overlap — extend the existing one instead of creating a parallel version
- Different approach to the same problem — evaluate which is better, keep one

## When duplication is acceptable

- Test fixtures (isolation matters more than DRY)
- Cross-module boundaries where coupling would be worse than duplication
- Performance-critical paths where the "shared" version adds overhead

## Violation signal

If a review or diff shows two functions/types that do substantially the same thing, the newer one should not have been created. Delete it and use the original.
