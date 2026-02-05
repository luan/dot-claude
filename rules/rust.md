---
paths:
  - "**/*.rs"
---

**Toolchain:** Latest nightly, latest edition (don't assume - check project config)

**Zero warnings:**
- `cargo clippy -- -W clippy::all` after EVERY implementation
- Zero warnings before complete
- Document any #[allow(...)] with reason

**Validation:**
1. `cargo fmt`
2. `cargo clippy -- -W clippy::all`
3. `cargo test`
4. `cargo build --release`

**Dead code:** Remove immediately. Use #[cfg(test)] for test-only code.

**Imports:** All `use` statements at file top only. No inline imports in function bodies.
