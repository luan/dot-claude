## Prompt
Should I reuse existing create_test_user() in src/api/test_helpers.rs or create a new one in tests/integration/helpers.rs for integration tests with different database setup?

## Response
This falls squarely in the "cross-module boundaries where coupling would be worse than duplication" exception.

**Recommendation: create the new one in `tests/integration/helpers.rs`, but with care.**

Why not reuse:
- Unit test helpers in `src/` and integration test helpers in `tests/` serve different purposes
- Different database setup = fundamentally different execution context
- Coupling integration tests to internal module structure creates fragile dependencies

What to do:
1. Extract pure `User` struct construction into a shared Default impl or builder
2. Let each helper handle its own database context
3. `create_test_user()` in both locations becomes a thin wrapper: build default user + context-specific setup
