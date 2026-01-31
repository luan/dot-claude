---
paths:
  - "**/Cargo.toml"
---

- Latest stable versions unless compatibility requires otherwise
- Use highest unambiguous version (e.g., `^3` not `^3.0`)
- Share deps across workspace members
- Keep dep list flat, no comments/grouping unless needed
- Avoid excessive custom features
