---
paths:
  - "**/Cargo.toml"
---

- Latest stable versions unless compatibility required
- Highest unambiguous version (e.g., `^3` not `^3.0`)
- Share deps across workspace
- Flat dep list, no comments/grouping unless needed
- Avoid excessive custom features
