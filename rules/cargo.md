---
paths:
  - "**/Cargo.toml"
---

**Dependency version preferences:**

- Prefer using the latest stable versions of dependencies unless specific versions are required for compatibility.
- Use the highest unambiguous version when specifying dependencies (e.g., use `^3` instead of `^3.0`).
- Prefer sharing dependencies across workspace members to minimize duplication.
- Regularly update dependencies to benefit from bug fixes and performance improvements.
- Avoid using too many custom "features" unless necessary for the project.
- Avoid commenting or grouping dependencies; keep the list flat and simple unless there is a clear reason to do otherwise.
