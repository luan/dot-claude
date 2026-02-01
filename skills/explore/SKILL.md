---
name: explore
description: "Use when starting fresh investigation - understand how something works, research before coding, plan a new feature. NOT for continuing existing work (use continue-explore)"
argument-hint: "<prompt>"
---

# Explore

Explore codebase → propose approaches → write plan.

## Why Two Files?

| File | Purpose | Lifespan |
|------|---------|----------|
| Plan-mode file | Approval UI display | Session only |
| `.agents/plans/` | Persistent reference | Cross-session |

Both needed: plan-mode for approval flow, .agents/plans for later `/implement` invocation.

## CRITICAL: Two Files Required

Plan mode manages its own file. You MUST ALSO write to `.agents/plans/`:

Current ts: !`date '+%Y%m%d-%H%M%S'`
Current branch: !`git branch --show-current | tr '/' '-'`

1. **Plan mode file**: Let plan mode handle (summary for approval UI)
2. **`.agents/plans/{ts}-{slug}.md`**: Full plan with implement instruction (for longevity)

The `.agents/plans/` file MUST end with:

```
To continue: use Skill tool to invoke `implement` with arg `{filename}`
```

## Steps

1. **Use EnterPlanMode tool** to switch to plan mode
2. Explore: search patterns, analyze code, identify 2-3 approaches
3. **Design process** (for complex features):
   - Ask questions one at a time via `AskUserQuestion`
   - Prefer multiple choice when options are clear
   - Present 2-3 approaches with trade-offs, lead with recommendation
   - Present design in 200-300 word sections, validate each
4. **Write to `.agents/plans/{ts}-{slug}.md`** using **writing-plans** format:
   - Context, approaches, recommendation, tasks
   - Bite-sized task structure (each step = one action, 2-5 min)
   - Exact file paths, complete code, exact commands
   - YAGNI ruthlessly - remove unnecessary features
   - End with: `To continue: use Skill tool to invoke implement with arg {filename}`
5. Write summary to plan mode's file (for approval UI) - MUST also end with:
   `To continue: use Skill tool to invoke implement with arg {filename}`
6. Resolve Open Questions via `AskUserQuestion`
