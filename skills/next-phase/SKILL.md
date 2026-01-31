---
name: next-phase
description: Continue multi-phase implementation from active state
argument-hint: "(none - uses current branch)"
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
6. Spawn Task: subagent_type="general-purpose", prompt below
   - Subagent writes to active-{branch}.md directly
   - Subagent retries on failure before returning error
   - Runs async (main waits for completion)
7. Update state: mark phase `[x]` with timestamp
   - More phases → "Phase N done. /next-phase"
   - Final → archive, prompt commit

## Execution Prompt

```
Continue phase from: {active_state_path}
Source plan: {plan_path}
Branch: {branch}
Phase: {phase_number}

Tasks:
{phase_tasks}

Instructions:
1. Execute tasks sequentially
2. Update {active_state_path} directly:
   - Mark task `[x]` with timestamp on success
   - Update Files Changed section
   - On failure: update Blockers, stop
3. Retry failures once before reporting
4. Auto-continue unless blocker

Return: completion status + files changed + any blockers.
```

## Phase Format

```markdown
## Tasks
**Phase 1: Setup** (completed 2026-01-31T12:00:00)
- [x] Task (timestamp)

**Phase 2: Implementation**
- [ ] Task
```
