---
name: continue-explore
description: "Use when continuing/refining existing exploration. Triggers: 'keep improving', 'still wrong', 'made progress but', 'explore more', 'that's not quite right', 'try again', 'reconsider', 'rethink', 'what about', 'actually', 'hmm', 'refine the plan', 'update the approach', 'missed something'"
argument-hint: "[plan-file] <feedback>"
---

# Continue Explore

Refine existing plan with user feedback.

## Why Two Files?

| File | Purpose | Lifespan |
|------|---------|----------|
| Plan-mode file | Approval UI display | Session only |
| `.agents/plans/` | Persistent reference | Cross-session |

Both needed: plan-mode for approval flow, .agents/plans for later `/implement` invocation.

## CRITICAL: Two Files Required

Plan mode manages its own file. You MUST ALSO update `.agents/plans/`:

Current branch: !`git branch --show-current | tr '/' '-'`
Most recent plan: !`ls -1t .agents/plans/ 2>/dev/null | head -1`

1. **Plan mode file**: Let plan mode handle (summary for approval UI)
2. **`.agents/plans/{plan-file}`**: Update full plan (for longevity)

The `.agents/plans/` file MUST end with:

```
To continue: use Skill tool to invoke `implement` with arg `{filename}`
```

## Steps

1. **Use EnterPlanMode tool** to switch to plan mode
2. Find plan: arg → `.agents/plans/{arg}` or most recent
3. Read existing plan
4. If exploration needed for feedback:
   - **Use Task tool** (subagent_type=Explore) to investigate specific areas
   - Let subagent validate assumptions, find edge cases, check patterns
5. If design rethink needed:
   - Ask questions one at a time via `AskUserQuestion`
   - Present 2-3 revised approaches with trade-offs
6. **Update `.agents/plans/{plan-file}`** with changes
   - Keep implement instruction at end
   - Bite-sized task structure (each step = one action)
   - YAGNI ruthlessly
7. Write summary to plan mode's file (for approval UI) - MUST also end with:
   `To continue: use Skill tool to invoke implement with arg {filename}`
8. Resolve new Open Questions via `AskUserQuestion`
