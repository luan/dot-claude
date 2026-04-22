1. Delete dead code completely. No commented-out code, shims, or "just in case."
2. Comments for WHY / edge cases / surprises only.
3. Use `ct sym` for lookup and code exploration.
4. All tests pass before committing. You own every failure you can see.
5. Prefer `apply_patch` for file edits, renames, creates, deletes.
