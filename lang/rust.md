## Code Quality Standards (Rust Projects)

**Zero Warnings Policy:**

- Run `cargo clippy -- -W clippy::all` after EVERY implementation
- Achieve zero warnings before considering work complete
- If warnings cannot be fixed, explicitly document why with #[allow(...)]

**Validation Workflow for Rust:**
Replace generic "formatters, linters, and tests" with:

1. `cargo fmt` - Format code
2. `cargo clippy -- -W clippy::all` - Lint with zero warnings
3. `cargo test` - Run all tests
4. `cargo build --release` - Verify release build

**Dead Code Philosophy:**

- Remove unused code immediately (no "might need later")
- If code is only used in tests, mark with #[cfg(test)]
- If genuinely needed for future, use #[allow(dead_code)] with TODO comment
