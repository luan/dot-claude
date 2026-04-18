Neither file exists yet. This is a greenfield question, not a codebase investigation.

**Create a new one.** Two different database setups means two different concerns — the integration test helper needs to set up a real (or containerized) database connection, while the unit test helper in `src/api/` presumably doesn't. Forcing one to depend on the other couples your test layers.

If both helpers share the same `User` struct field defaults, extract those defaults into the struct itself (e.g., a `User::test_default()` or `impl Default for User` if it's test-only), then have each helper call that and layer on its own database setup. That way you share the boring field values without coupling the database concerns.
