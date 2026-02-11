---
name: writing-skills
description: "Use when creating new skills, editing existing skills, or verifying skills work before deployment"
---

# Writing Skills

TDD for process docs. Write test (pressure scenario) → fail → write skill → pass → refactor.

**Iron Law:** No skill without failing test first.

## When to Create

**Create:** non-obvious technique, cross-project reference, broadly applicable.

**Skip:** one-off, standard practices, project-specific (use CLAUDE.md).

## Skill Types

- **Technique:** concrete steps
- **Pattern:** way of thinking
- **Reference:** API docs, syntax

## Locations

| Location | Path | Scope |
|----------|------|-------|
| Personal | `~/.claude/skills/<name>/SKILL.md` | All projects |
| Project | `.claude/skills/<name>/SKILL.md` | This project |
| Plugin | `<plugin>/skills/<name>/SKILL.md` | Where enabled |

Priority: enterprise > personal > project. Plugin: `plugin:skill` namespace.

## Directory Structure

```
skills/
  skill-name/
    SKILL.md              # Main reference (required)
    template.md           # Template
    examples/             # Example outputs
    scripts/              # Utilities
```

Keep `SKILL.md` <500 lines. Heavy reference → separate files.

## Frontmatter

```yaml
---
name: my-skill
description: "Use when..." # REQUIRED for discovery
argument-hint: "[issue-number]"
disable-model-invocation: true # User-only
user-invocable: false # Claude-only
allowed-tools: Read, Grep, Glob
model: opus
context: fork # Subagent
agent: Explore # Subagent type
---
```

| Field | Effect |
|-------|--------|
| `disable-model-invocation: true` | User-only |
| `user-invocable: false` | Claude-only |
| `context: fork` | Isolated subagent |

## String Substitutions

| Variable | Description |
|----------|-------------|
| `$ARGUMENTS` | All args |
| `$ARGUMENTS[N]` or `$N` | Specific arg (0-indexed) |
| `${CLAUDE_SESSION_ID}` | Session ID |

## Dynamic Context

**!** + command in backticks → shell before Claude.

```
Current branch: !\`git branch --show-current\`
```

## SKILL.md Structure

```markdown
---
name: skill-name-with-hyphens
description: "Use when [triggers only - NO workflow summary]"
---

# Skill Name

## Overview
Core principle in 1-2 sentences.

## When to Use
Symptoms + use cases. When NOT to use.

## Core Pattern
Before/after code, key steps.

## Quick Reference
Table for scanning.

## Common Mistakes
What fails + fixes.
```

## Description Field (Critical)

**ONLY triggers. NEVER workflow summary.**

```yaml
# BAD: Claude shortcuts instead of reading skill
description: Use when executing plans - dispatches subagent per task with review

# GOOD: Just triggers
description: Use when executing implementation plans with independent tasks
```

**Why:** Claude shortcuts workflow summaries, skips actual content.

## Token Efficiency

- getting-started: <150 words
- Frequently-loaded: <200 words
- Other: <500 words
- Move details to `--help`, cross-reference
- One excellent example > many mediocre

## RED-GREEN-REFACTOR

### RED: Baseline
Run pressure scenario WITHOUT skill. Document: choices, rationalizations (verbatim), pressures triggering violations.

### GREEN: Minimal
Address rationalizations. Run WITH skill → compliance.

### REFACTOR: Close Loopholes
New rationalization → add counter → re-test until bulletproof.

## Bulletproofing Discipline Skills

For rule-enforcing (TDD, verification):

1. **Close loopholes explicitly:**
   ```markdown
   Write code before test? Delete it.
   - Don't keep as "reference"
   - Don't "adapt" while testing
   - Delete means delete
   ```

2. **Add:** `**Violating the letter is violating the spirit.**`
3. **Build rationalization table** from baseline
4. **Create red flags list** for self-checking

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Not triggering | Check description keywords, try `/skill-name` |
| Triggers too often | More specific description, add `disable-model-invocation` |
| Claude doesn't see | Check `/context` for character budget warning |

## Checklist

**RED:** pressure scenarios (3+ for discipline) → run WITHOUT skill → document baseline → identify rationalizations

**GREEN:** YAML frontmatter (name + description) → description triggers only → address baseline → one excellent example

**REFACTOR:** identify NEW rationalizations → add counters → rationalization table → re-test

**Deploy:** commit + push
