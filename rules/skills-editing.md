---
paths:
  - "skills/**/*.md"
---

# Skill Editing

Skills degrade when new features are appended rather than woven
into existing structure. Every edit must preserve cohesion.

## Do Not

- Append new modes/features as standalone sections at the bottom
- Duplicate guidance across sections — e.g., same truncation rule
  in two places
- Order sections by when they were written rather than workflow
  reading order
- Create a separate prompt template when an existing one can be
  extended with callouts

## Do

- **Integrate into existing steps** — new behavior belongs inside
  the workflow step where it's referenced, not in an appendix
- **Consolidate shared structure** — if two modes use similar
  prompts or output formats, use one template with per-mode
  callouts for differences
- **Match section order to workflow order** — sections appear in
  the order the workflow references them, so readers never
  backtrack
- **Extract duplicated guidance** — same rule in two places →
  pull into a shared section and reference it
- **Read top-to-bottom after editing** — read the full skill file
  sequentially to verify it flows without jumps or redundancy
