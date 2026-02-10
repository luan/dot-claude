# Model Tiering

## Tiers

| Tier | Model | When |
|------|-------|------|
| Deep | opus (inherit) | Architecture, devil's advocate, complex planning |
| Standard | sonnet | Implementation, testing, review, feedback |
| Fast | haiku | Research, commit messages, compression |

## Skill Assignments

| Skill | Dispatch Model | Nested Model |
|-------|---------------|--------------|
| commit | haiku | - |
| compress-prompt | sonnet | haiku |
| feedback | sonnet | - |
| review | inherit (lenses) | sonnet (fixes) |
| split-commit | inherit (analysis) | sonnet (commits) |

## Agent Assignments

| Agent | Model |
|-------|-------|
| implementer | sonnet |
| tester | sonnet |
| reviewer | sonnet |
| researcher | haiku |
| architect | inherit |
| devil | inherit |

## Team Assignments

| Team Skill | Role | Model |
|------------|------|-------|
| team-explore | researcher | haiku |
| team-explore | architect | opus |
| team-explore | devil | opus |
| team-implement | workers | sonnet |
| team-review | security | opus |
| team-review | arch/perf | sonnet |
| team-review | spec/test | opus |
| team-debug | investigators | opus |

## Effort Level

`CLAUDE_CODE_EFFORT_LEVEL` is session-only (env var).
Cannot be set in frontmatter or per-subagent.

## Escalation

sonnet fails on task requiring deeper reasoning
--> bump to opus for that dispatch only.
Do not change the skill default.

## Env Var

`CLAUDE_CODE_SUBAGENT_MODEL` overrides subagent model globally.
Frontmatter `model:` field overrides per-skill/agent.
