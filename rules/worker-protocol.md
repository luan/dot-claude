# Worker Protocol

Single reference for multi-agent coordination rules.

## Claiming

- `bd update <id> --claim` — atomic (assignee + in_progress)
- Fails if already claimed → pick next task
- Never manually set status + assignee separately

## File Ownership

- Each task owns specific files (from beads metadata or plan)
- NEVER edit files outside your task scope
- Need a change in another worker's file → MESSAGE the owner
- Owner idle → MESSAGE team lead
- Editing another worker's file = task failure

## Build Failures

- **Your changes broke it:** fix before closing
- **Another worker's changes broke it:** report to lead, continue
  on your own files. Don't fix their code.
- **Pre-existing failure (before any worker started):** report
  once, continue working. Don't block on it.

## Completion Checklist

No `bd close` without ALL passing:
1. Build clean (zero errors)
2. Tests pass (new + existing in scope)
3. Linter clean (clippy/eslint/swiftc as applicable)
4. Changes committed via merge-slot

## Merge Serialization

```
bd merge-slot acquire --wait
git add <files> && git commit -m "..."
bd merge-slot release
```
Never hold the slot while running tests or doing non-git work.

## Escalation

- Blocked by another worker > 2 min → message lead
- 2 failed fix attempts → message lead with details
- Interface disagreement → lead arbitrates
