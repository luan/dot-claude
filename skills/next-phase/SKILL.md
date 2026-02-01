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
3. Execute phase tasks using subagent-driven pattern (see `implement` skill)
   - If task fails → **use Skill tool** to invoke `debugging`
4. Mark phase complete, update state
5. More phases → **use Skill tool** to invoke `next-phase` | Final → present completion options to user
