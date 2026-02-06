---
name: writing-skills
description: "Use when creating new skills, editing existing skills, or verifying skills work before deployment"
---

# Writing Skills

TDD for process docs. Write test (pressure scenario) → fail (baseline) → write skill → pass → refactor (close loopholes).

**Iron Law:** No skill without failing test first.

## When to Create

**Create:** non-obvious technique, cross-project reference, broadly applicable pattern.

**Skip:** one-off solutions, standard practices documented elsewhere, project-specific conventions (use CLAUDE.md).

## Skill Types

- **Technique:** concrete steps (condition-based-waiting)
- **Pattern:** way of thinking (flatten-with-flags)
- **Reference:** API docs, syntax guides

## Skill Locations

| Location | Path | Scope |
|----------|------|-------|
| Personal | `~/.claude/skills/<name>/SKILL.md` | All projects |
| Project | `.claude/skills/<name>/SKILL.md` | This project |
| Plugin | `<plugin>/skills/<name>/SKILL.md` | Where enabled |

Priority: enterprise > personal > project. Plugin uses `plugin:skill` namespace.

## Directory Structure

```
skills/
  skill-name/
    SKILL.md              # Main reference (required)
    template.md           # Template for Claude to fill in
    examples/             # Example outputs
    scripts/              # Executable utilities
```

Keep `SKILL.md` under 500 lines. Heavy reference → separate files.

## Frontmatter Reference

```yaml
---
name: my-skill # Display name (default: directory name)
description: "Use when..." # When to use (REQUIRED for discovery)
argument-hint: "[issue-number]" # Autocomplete hint
disable-model-invocation: true # Only user can invoke via /name
user-invocable: false # Only Claude can invoke (background knowledge)
allowed-tools: Read, Grep, Glob # Tools without permission prompt
model: opus # Model override
context: fork # Run in subagent
agent: Explore # Subagent type (with context: fork)
---
```

| Field | Effect |
|-------|--------|
| `disable-model-invocation: true` | User-only (deploy, commit) |
| `user-invocable: false` | Claude-only (background context) |
| `context: fork` | Runs in isolated subagent |

## String Substitutions

| Variable | Description |
|----------|-------------|
| `$ARGUMENTS` | All arguments passed |
| `$ARGUMENTS[N]` or `$N` | Specific argument (0-indexed) |
| `${CLAUDE_SESSION_ID}` | Current session ID |

## Dynamic Context Injection

`!` + command in backticks → runs shell before sending to Claude.

```
Current branch: !\`git branch --show-current\`
```

## SKILL.md Structure

```markdown
---
name: skill-name-with-hyphens
description: "Use when [triggering conditions only - NO workflow summary]"
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
What goes wrong + fixes.
```

## Description Field (Critical)

**ONLY triggering conditions. NEVER workflow summary.**

```yaml
# BAD: Claude follows this instead of reading skill
description: Use when executing plans - dispatches subagent per task with code review

# GOOD: Just triggers
description: Use when executing implementation plans with independent tasks
```

**Why:** Claude shortcuts workflow summaries in descriptions, skipping actual skill content.

## Token Efficiency

- getting-started: <150 words
- Frequently-loaded: <200 words
- Other: <500 words
- Move details to `--help`, cross-reference skills
- One excellent example > many mediocre ones

## RED-GREEN-REFACTOR for Skills

### RED: Baseline Test
Run pressure scenario WITHOUT skill. Document: agent choices, exact rationalizations (verbatim), which pressures triggered violations.

### GREEN: Minimal Skill
Address those specific rationalizations. Run same scenario WITH skill → agent should comply.

### REFACTOR: Close Loopholes
New rationalization found → add explicit counter → re-test until bulletproof.

## Bulletproofing Discipline Skills

For rule-enforcing skills (TDD, verification):

1. **Close loopholes explicitly:**
   ```markdown
   Write code before test? Delete it.
   - Don't keep as "reference"
   - Don't "adapt" while testing
   - Delete means delete
   ```

2. **Add:** `**Violating the letter is violating the spirit.**`
3. **Build rationalization table** from baseline failures
4. **Create red flags list** for self-checking

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Not triggering | Check description keywords, try `/skill-name` directly |
| Triggers too often | More specific description, add `disable-model-invocation` |
| Claude doesn't see | Check `/context` for character budget warning |

## Checklist

**RED:** pressure scenarios (3+ for discipline) → run WITHOUT skill, document baseline → identify rationalization patterns

**GREEN:** YAML frontmatter (name + description only) → description triggers only → address baseline failures → one excellent example

**REFACTOR:** identify NEW rationalizations → add counters → rationalization table → re-test until bulletproof

**Deploy:** commit + push
