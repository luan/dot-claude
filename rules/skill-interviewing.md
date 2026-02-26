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

When a skill finishes, there's usually an obvious next step in the pipeline (scope→develop, develop→review).

**Do not passively suggest and stop** ("Next: /review" then silence). Either:
1. **Proceed automatically** — invoke the next skill directly
2. **Confirm then proceed** — AskUserQuestion with a clear action button if the next step is significant enough to warrant a pause

### When to stop vs auto-proceed

**Auto-proceed** when the preceding skill already gated its output.
- scope → develop (plan-mode approval already gates the design; task creation is mechanical)

**Stop** when the skill produced output that needs human verification before the next step acts on it.
- brainstorm → scope (design needs technical validation before becoming tasks)
- develop → user verification (user must verify functionality before review, or review risks blessing regressions)

Never present formulaic menus with "Done for now" filler options. If the user wants to stop, they'll just... stop.

## Post-Skill Artifact Freshness

When user feedback after a skill's output substantively changes the design (new approach, different scope, architectural shift — not acknowledgment or minor wording):

- **Update stored artifacts immediately** (task metadata, plan files) — don't defer to the next skill
- Next-skill invocations read from stored artifacts, not conversation context
- Critical for cross-session continuity: user may invoke the next skill in a fresh session
- Stale artifacts + fresh session = wrong plan = wrong implementation
