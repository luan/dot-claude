# Model Tiering

## Tiers

| Tier | Model | When |
|------|-------|------|
| Deep | opus (inherit) | Default for anything with ambiguity, judgment, or multiple valid approaches |
| Standard | sonnet | Purely mechanical tasks with zero ambiguity (copy from spec, apply known fix) |
| Fast | haiku | Commit messages, compression |

**Default to opus.** Only drop to sonnet when the task is truly
mechanical with a single obvious approach. When in doubt, opus.

## Skill Assignments

| Skill | Dispatch Model | Nested Model |
|-------|---------------|--------------|
| commit | haiku | - |
| compress-prompt | sonnet | haiku |
| fix | inherit | - |
| prepare | inherit | inherit (task creation) |
| review | inherit (lenses) | sonnet (fixes) |
| split-commit | inherit (analysis) | sonnet (commits) |

Agent behavior inlined in skill prompts. Model specified per-dispatch.

## Team Assignments (inline in skills)

| Skill | Role | Model |
|-------|------|-------|
| explore (escalation) | researcher | opus |
| explore (escalation) | architect | opus |
| explore (escalation) | devil | opus |
| implement (swarm) | workers | sonnet (mechanical) or opus (ambiguous) |
| review (perspective) | architect | opus |
| review (perspective) | code-quality | opus |
| review (perspective) | devil | opus |

## Effort Level

`CLAUDE_CODE_EFFORT_LEVEL` session-only (env var). Cannot set in frontmatter or per-subagent.

## Escalation

sonnet fails or produces low-quality output → already using opus is preferred over reactive escalation.

## Env Var

`CLAUDE_CODE_SUBAGENT_MODEL` overrides subagent model globally. Frontmatter `model:` overrides per-skill/agent.
