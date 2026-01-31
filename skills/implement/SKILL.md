---
name: implement
description: "Execute exploration plan - tracks progress in .agents/active-{branch}.md"
argument-hint: "[plan-file] (default: most recent)"
---

# Implement

Execute plan from `/explore`, track in active state file.

## Steps

1. Find plan: arg → `.agents/plans/{arg}` or most recent by timestamp
2. `git branch --show-current` → sanitize
3. Check `.agents/active-{branch}.md`:
   - exists + same source → resume
   - exists + different source → use `AskUserQuestion`: "Active state exists for different plan. Continue with new plan or abort?" (options: Continue, Abort)
   - not exists → create
4. Parse Next Steps: `- [ ]` lines, detect phases (`**Phase N:**`)
5. Multi-phase → load Phase 1 only

Create/update active state (**use compress-prompt** - AI consumption):
```markdown
# Implementation: {topic}
Source: {plan path}
Started: {ISO}
Branch: {branch}
Status: in_progress

## Tasks
- [ ] ...

## Files Changed
(updated during work)

## Blockers
None
```

6. Spawn Task: subagent_type="general-purpose", prompt below
   - Subagent writes to active-{branch}.md directly
   - Subagent retries on failure before returning error
   - Runs async (main waits for completion)

7. Completion:
   - Multi-phase → "Phase N done. /next-phase for next."
   - Final → archive to `.agents/archive/{ts}-implemented-{slug}.md`, prompt commit

## Execution Prompt

```
Implement tasks from: {active_state_path}
Source plan: {plan_path}
Branch: {branch}

Tasks:
{task_list}

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

## Errors

- No plan: "Run /explore first"
- Parse fail: "Check Next Steps format"
- Task fail: preserve state, report
