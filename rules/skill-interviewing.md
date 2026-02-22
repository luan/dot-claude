# Skill Interviewing

## Mid-Skill: Ask on Genuine Ambiguity

Use AskUserQuestion when user preference genuinely matters:
- Multiple viable paths with unclear tradeoffs
- Irreversible/destructive actions (commits, resets)
- Domain clarification (business logic, priority calls)

Match question complexity to situation:
- Simple binary → 2 options
- Nuanced tradeoff → describe tradeoffs, 2-3 options
- Open-ended → free-form question, no fixed options

## End-of-Skill: Continue or Confirm

When a skill finishes, there's usually an obvious next step in the pipeline (explore→prepare, prepare→implement, implement→review).

**Do not passively suggest and stop** ("Next: /review" then silence). Either:
1. **Proceed automatically** — invoke the next skill directly
2. **Confirm then proceed** — AskUserQuestion with a clear action button if the next step is significant enough to warrant a pause

### When to stop vs auto-proceed

**Stop** when the skill produced output that needs human verification before the next step acts on it.
- explore → prepare (findings become tasks — wrong findings → wrong plan)
- brainstorm → prepare (design becomes tasks)
- implement → user verification (user must verify functionality before review, or review risks blessing regressions)

**Auto-proceed** when the next step is mechanical or self-correcting:
- prepare → implement (user already validated the analysis)

Never present formulaic menus with "Done for now" filler options. If the user wants to stop, they'll just... stop.
