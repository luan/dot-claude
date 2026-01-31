---
name: implement
description: Execute exploration plan - tracks progress in .agents/active-{branch}.md
argument-hint: [plan-file] (default: most recent)
---

# Implement

Execute plan from `/explore`, track in active state file.

## Steps

1. Find plan: arg → `.agents/plans/{arg}` or most recent by timestamp
2. `git branch --show-current` → sanitize
3. Check `.agents/active-{branch}.md`:
   - exists + same source → resume
   - exists + different source → warn, ask continue/abort
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

6. Execute tasks:
   - TaskUpdate in_progress
   - Execute via superpowers:executing-plans patterns
   - Success → mark `[x]` with timestamp
   - Failure → stop, update Blockers, report
   - Auto-continue unless failure

7. Completion:
   - Multi-phase → "Phase N done. /next-phase for next."
   - Final → archive to `.agents/archive/{ts}-implemented-{slug}.md`, prompt commit

## Errors

- No plan: "Run /explore first"
- Parse fail: "Check Next Steps format"
- Task fail: preserve state, report
