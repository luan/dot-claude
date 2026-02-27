---
paths:
  - "**/*.rs"
---

**Toolchain:** Latest nightly, latest edition (check project config)

**Zero warnings:**

- `cargo clippy -- -W clippy::all` after EVERY implementation
- Zero warnings before complete
- Avoid #[allow(...)] with reason unless DIRECTLY instructed by user.

**Validation:**

1. `cargo fmt`
2. `cargo clippy -- -W clippy::all`
3. `cargo test`
4. `cargo build`

**Dead code:** Remove immediately. Use #[cfg(test)] for test-only.

**Imports:** All `use` at file top. No inline imports.

**No section-divider comments:** Do not generate `// ─── Section Name ───` or similar box-drawing/ruler comments above functions. The function name is the label.
