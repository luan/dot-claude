---
name: next-phase
description: Continue multi-phase implementation from active state
argument-hint: (none - uses current branch)
---

# Next Phase

Continue multi-phase work.

## Steps

1. `git branch --show-current` → sanitize
2. Read `.agents/active-{branch}.md` or error "Run /implement first"
3. Verify branch matches (warn if different, continue)
4. Find first incomplete phase (`[ ]` tasks under `**Phase N:**`)
   - All done → "All phases complete! /commit"
5. Load source plan, find phase's tasks
6. Execute like /implement
7. Update state: mark phase `[x]` with timestamp
   - More phases → "Phase N done. /next-phase"
   - Final → archive, prompt commit

## Phase Format

```markdown
## Tasks
**Phase 1: Setup** (completed 2026-01-31T12:00:00)
- [x] Task (timestamp)

**Phase 2: Implementation**
- [ ] Task
```
