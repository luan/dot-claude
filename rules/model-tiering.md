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
| fix | sonnet | - |
| review | inherit (lenses) | sonnet (fixes) |
| split-commit | inherit (analysis) | sonnet (commits) |

Agent behavior inlined in skill prompts. Model specified per-dispatch.

## Team Assignments (inline in skills)

| Skill | Role | Model |
|-------|------|-------|
| explore (escalation) | researcher | haiku |
| explore (escalation) | architect | opus |
| explore (escalation) | devil | opus |
| implement (swarm) | workers | sonnet |
| review (perspective) | architect | opus |
| review (perspective) | code-quality | sonnet |
| review (perspective) | devil | opus |

## Effort Level

`CLAUDE_CODE_EFFORT_LEVEL` session-only (env var). Cannot set in frontmatter or per-subagent.

## Escalation

sonnet fails on task requiring deeper reasoning → bump to opus for that dispatch only. Don't change skill default.

## Env Var

`CLAUDE_CODE_SUBAGENT_MODEL` overrides subagent model globally. Frontmatter `model:` overrides per-skill/agent.
