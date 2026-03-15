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

When a skill finishes, there's usually an obvious next step in the pipeline (spec→scope, scope→develop, develop→review).

**Do not passively suggest and stop** ("Next: /review" then silence). Either:
1. **Proceed automatically** — invoke the next skill directly
2. **Confirm then proceed** — AskUserQuestion with a clear action button if the next step is significant enough to warrant a pause

### When to stop vs auto-proceed

**Auto-proceed** when the preceding skill already gated its output.
- spec → scope (spec approval already gates the target; planning is mechanical)
- scope → develop (plan-mode approval already gates the design; task creation is mechanical)

**Stop** when the skill produced output that needs human verification before the next step acts on it.
- brainstorm → user decides next step (spec, scope, or vibe — user picks the executor)
- develop → user verification (user must verify functionality before review, or review risks blessing regressions)

Never present formulaic menus with "Done for now" filler options. If the user wants to stop, they'll just... stop.

## Post-Skill Artifact Freshness

When user feedback after a skill's output substantively changes the design (new approach, different scope, architectural shift — not acknowledgment or minor wording):

- **Update stored artifacts immediately** (task metadata, plan files) — don't defer to the next skill
- Next-skill invocations read from stored artifacts, not conversation context
- Critical for cross-session continuity: user may invoke the next skill in a fresh session
- Stale artifacts + fresh session = wrong plan = wrong implementation

## Cross-Session Handoff

When a skill finishes and the next step is in a different skill (scope→develop), the handoff metadata MUST be written before the session can end:

- `/spec` presenting a spec with `Next:` → set `status_detail: "approved"` at the same time as presenting.
- `/scope` presenting a plan with `Next: /develop` → set `status_detail: "approved"` at the same time as presenting.
- Any task metadata that the next skill reads (`status_detail`, `design`, `spec`) must be fully populated before outputting the `Next:` line.
- The user may run the next skill in a fresh session with no conversation context — only task metadata survives.
