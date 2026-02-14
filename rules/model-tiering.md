# Model Tiering

## Tiers

| Tier | Model | When |
|------|-------|------|
| Deep | opus (inherit) | Default for anything with ambiguity, judgment, or multiple valid approaches |
| Standard | sonnet | Purely mechanical tasks with zero ambiguity (copy from spec, apply known fix) |
| Fast | haiku | Commit messages, compression |

**Default to opus.** Only drop to sonnet when ALL of these hold:
1. Zero design decisions — the task specifies exactly what to write
2. Single file or isolated files with no cross-file coordination
3. No error handling, state management, or API surface changes
4. Pattern is copy/paste from existing code with substitutions only

If any criterion fails → opus. When in doubt, opus.

### What "Mechanical" Actually Means

Sonnet-eligible (rare): rename a variable across a file, add a config
entry matching an existing pattern exactly, copy an existing test for a
new fixture with only name/value substitutions.

Opus-required (default): new functions, new error paths, new tests for
untested behavior, refactors that change structure, anything touching
public API, anything the brief describes with "should" or "consider",
any task where substitutions change types/signatures/control flow.

**Insufficient brief = opus.** If the task description lacks enough
detail to confidently evaluate all 4 criteria above, use opus. Never
infer mechanical from a title alone.

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
| implement (swarm) | workers | opus (default) — sonnet only if ALL mechanical criteria met |
| review (perspective) | architect | opus |
| review (perspective) | code-quality | opus |
| review (perspective) | devil | opus |

## Effort Level

`CLAUDE_CODE_EFFORT_LEVEL` session-only (env var). Cannot set in frontmatter or per-subagent.

## Escalation

sonnet fails or produces low-quality output → already using opus is preferred over reactive escalation.

## Env Var

`CLAUDE_CODE_SUBAGENT_MODEL` overrides subagent model globally. Frontmatter `model:` overrides per-skill/agent.
