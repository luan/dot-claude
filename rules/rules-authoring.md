---
paths:
  - "rules/*.md"
  - ".claude/rules/*.md"
---

# Rules Authoring

Every rule file must have YAML frontmatter with a `paths`
glob so it only loads in relevant contexts. Omit `paths`
only for rules that genuinely apply everywhere.
