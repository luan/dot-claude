---
paths:
  - "**/*.rs"
---

**Toolchain:** Latest nightly, latest edition (check project config)

**Zero warnings:**

- `cargo clippy -- -W clippy::all` after every implementation
- Zero warnings before presenting code to user
- Never write code that obviously warns (unused variables, dead code, empty enums making types uninhabited)
- Use simpler constructs that don't warn over complex ones that do
- No `#[allow(...)]` unless user directly instructs it

**Validation:** `cargo fmt` → `cargo clippy -- -W clippy::all` → `cargo test` → `cargo build`

**Dead code:** Remove immediately. `#[cfg(test)]` for test-only.

**Imports:** All `use` at file top. No inline imports.
