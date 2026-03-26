# Failure Learning

When a review, debugging session, or acceptance check reveals a **codebase-specific antipattern** — a mistake that could recur because nothing in the project structure prevents it — codify it as a project rule.

## What qualifies

- A bug caused by a non-obvious project convention (e.g., "this API returns timestamps in seconds, not milliseconds")
- An architectural constraint that isn't enforced by the type system or linter (e.g., "module A must never import from module B directly")
- A pattern that was tried and failed for project-specific reasons (e.g., "async initialization doesn't work here because the runtime isn't started yet")

## What does NOT qualify

- Generic best practices (already in global rules or widely known)
- One-off bugs with obvious fixes (typo, wrong variable name)
- Temporary workarounds (these expire; use comments instead)

## Action

When you encounter a qualifying antipattern:
1. Write a rule file in `<project>/.claude/rules/<topic>.md` with `paths` frontmatter scoping it to the relevant area
2. Include: what went wrong, why it's non-obvious, what to do instead
3. Reference the specific files/modules where this applies

The goal is to prevent the next session from repeating the same investigation. One rule that prevents a class of bugs is worth more than fixing ten instances manually.
