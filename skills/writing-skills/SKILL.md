---
name: writing-skills
description: "Use when creating new skills, editing existing skills, or verifying skills work before deployment"
---

# Writing Skills

TDD applied to process documentation. Write test (pressure scenario) → watch fail (baseline) → write skill → watch pass → refactor (close loopholes).

**Iron Law:** No skill without failing test first. Same as TDD for code.

## When to Create

**Create when:**

- Technique wasn't intuitively obvious
- Would reference across projects
- Pattern applies broadly
- Others would benefit

**Don't create for:**

- One-off solutions
- Standard practices documented elsewhere
- Project-specific conventions (use CLAUDE.md)

## Skill Types

- **Technique:** Concrete method with steps (condition-based-waiting)
- **Pattern:** Way of thinking (flatten-with-flags)
- **Reference:** API docs, syntax guides

## Skill Locations

| Location | Path                               | Scope                |
| -------- | ---------------------------------- | -------------------- |
| Personal | `~/.claude/skills/<name>/SKILL.md` | All your projects    |
| Project  | `.claude/skills/<name>/SKILL.md`   | This project only    |
| Plugin   | `<plugin>/skills/<name>/SKILL.md`  | Where plugin enabled |

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

Keep `SKILL.md` under 500 lines. Move heavy reference to separate files.

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

| Field                            | Effect                           |
| -------------------------------- | -------------------------------- |
| `disable-model-invocation: true` | User-only (deploy, commit)       |
| `user-invocable: false`          | Claude-only (background context) |
| `context: fork`                  | Runs in isolated subagent        |

## String Substitutions

| Variable                | Description                   |
| ----------------------- | ----------------------------- |
| `$ARGUMENTS`            | All arguments passed          |
| `$ARGUMENTS[N]` or `$N` | Specific argument (0-indexed) |
| `${CLAUDE_SESSION_ID}`  | Current session ID            |

Example: `Fix GitHub issue $0 following our coding standards.`

## Dynamic Context Injection

Syntax: `!` followed by command in backticks runs shell before sending to Claude.

Example in a skill file:

```
Current branch: !\`git branch --show-current\`
```

Commands execute during preprocessing → output replaces placeholder.

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

Symptoms and use cases. When NOT to use.

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
# BAD: Summarizes workflow - Claude follows this instead of reading skill
description: Use when executing plans - dispatches subagent per task with code review

# GOOD: Just triggers
description: Use when executing implementation plans with independent tasks
```

**Why:** Testing showed Claude shortcuts workflow summaries in descriptions, skipping the actual skill content.

## Token Efficiency

- getting-started skills: <150 words
- Frequently-loaded: <200 words
- Other skills: <500 words
- Move details to `--help`, cross-reference other skills
- One excellent example beats many mediocre ones

## RED-GREEN-REFACTOR for Skills

### RED: Baseline Test

Run pressure scenario WITHOUT skill. Document:

- What choices agent made
- Exact rationalizations used (verbatim)
- Which pressures triggered violations

### GREEN: Minimal Skill

Write skill addressing those specific rationalizations. Run same scenario WITH skill. Agent should comply.

### REFACTOR: Close Loopholes

New rationalization found? Add explicit counter. Re-test until bulletproof.

## Bulletproofing Discipline Skills

For skills enforcing rules (TDD, verification):

1. **Close loopholes explicitly:**

   ```markdown
   Write code before test? Delete it.

   - Don't keep as "reference"
   - Don't "adapt" while testing
   - Delete means delete
   ```

2. **Add foundational principle:**

   ```markdown
   **Violating the letter is violating the spirit.**
   ```

3. **Build rationalization table** from baseline failures
4. **Create red flags list** for self-checking

## Troubleshooting

| Problem                  | Solution                                                       |
| ------------------------ | -------------------------------------------------------------- |
| Skill not triggering     | Check description keywords, try `/skill-name` directly         |
| Triggers too often       | Make description more specific, add `disable-model-invocation` |
| Claude doesn't see skill | Check `/context` for character budget warning                  |

## Checklist

**RED Phase:**

- [ ] Create pressure scenarios (3+ pressures for discipline skills)
- [ ] Run WITHOUT skill, document baseline verbatim
- [ ] Identify rationalization patterns

**GREEN Phase:**

- [ ] YAML frontmatter: name + description only
- [ ] Description: "Use when..." (triggers only, no workflow)
- [ ] Address specific baseline failures
- [ ] One excellent example

**REFACTOR Phase:**

- [ ] Identify NEW rationalizations
- [ ] Add explicit counters
- [ ] Build rationalization table
- [ ] Re-test until bulletproof

**Deployment:**

- [ ] Commit and push
