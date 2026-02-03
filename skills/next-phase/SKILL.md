---
name: next-phase
description: "Use when continuing to the next phase of a multi-phase implementation"
argument-hint: "(none - uses current branch)"
---

# Next Phase

Current branch: !`git branch --show-current | tr '/' '-'`
Continue from `implement`.

## Steps

1. Read `.agents/active-{branch}.md` or error "Run implement first"
2. Find next incomplete phase (`[ ]` tasks under `**Phase N:**`)
   - All done → present completion options to user
3. **Context clearing (default):** Use `EnterPlanMode` to get fresh context
   - Summarize completed work and remaining tasks in plan file
   - User approves → continue with clean context
   - If user wants to refine plan first → **use Skill tool** to invoke `continue-explore`
   - Skip context clearing only if user explicitly requests same context
4. Execute phase tasks using subagent-driven pattern (see `implement` skill)
   - If task fails → **use Skill tool** to invoke `debugging`
5. Mark phase complete, update state
6. More phases → **use Skill tool** to invoke `next-phase` | Final → present completion options to user
