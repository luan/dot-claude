---
name: writing-skills
description: Create, edit, improve, and troubleshoot Claude Code agent skills. Use when making a new skill, editing SKILL.md files, modifying skill metadata (types, priority, labels, owner), refactoring skill instructions, improving skill discoverability, debugging why a skill isn't activating, answering questions about skills, or reviewing skill best practices. Triggers: 'new skill', 'edit skill', 'update skill', 'fix skill', 'skill not working', 'SKILL.md', 'skill metadata', 'skill consistency'.
---

# Writing Skills

TDD for process docs. Write test (pressure scenario) → fail → write skill → pass → refactor.

**Iron Law:** No skill without failing test first.

## When to Create

**Create:** non-obvious technique, cross-project reference, broadly applicable.

**Skip:** one-off, standard practices, project-specific (use CLAUDE.md).

## Skill Types

- **Toolbox:** scripts that encapsulate complexity; SKILL.md teaches Claude when/how to invoke them
- **Knowledge Injection:** valuable knowledge Claude didn't have before (CLI usage, domain expertise, nuanced workflows)
- **Technique:** concrete steps for a repeatable process
- **Reference:** API docs, syntax, lookup tables

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
    scripts/              # Automation (python + uv)
    references/           # Heavy reference material
    assets/               # Templates, samples, static data
    examples/             # Example outputs
```

Never add README.md — SKILL.md is the entry point. Keep `SKILL.md` <500 lines; heavy reference → separate files.

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

**Restrictions:** name must NOT contain "claude" or "anthropic". No XML tags in values. Description <1024 chars, single line.

| Field | Effect |
|-------|--------|
| `disable-model-invocation: true` | User-only |
| `user-invocable: false` | Claude-only |
| `context: fork` | Isolated subagent |

## Description Field (Critical)

The description is the **only thing Claude sees** before deciding to load a skill. Claude is too conservative — optimize for discoverability. A false positive (loaded but unused) is cheap. A false negative (not loaded, Claude spirals) ruins the session.

**WHAT it does + WHEN to use. NEVER workflow details.** Don't rely on users saying magic words — think about what *situations* call for this skill, including ones Claude should decide to use on its own.

```yaml
# BAD: workflow details Claude will shortcut
description: Use when executing plans - dispatches subagent per task with review

# GOOD: what + when, broad trigger surface
description: Handles SpeedReader server lifecycle (build, startup, shutdown) and web page rebuild/refresh. Use when you need to verify a web page works, view it, test UI interactions, or see how a page behaves. Also covers development tasks.
```

## String Substitutions

| Variable | Description |
|----------|-------------|
| `$ARGUMENTS` | All args |
| `$ARGUMENTS[N]` or `$N` | Specific arg (0-indexed) |
| `${CLAUDE_SESSION_ID}` | Session ID |

## Dynamic Context

**!** + command in backticks → shell executes before Claude sees content.

```
Current branch: !`git branch --show-current`
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

**Progressive disclosure** — structure content in 3 levels:
1. **Critical** — must-know rules (`## Critical` header)
2. **Important** — common patterns (`## Important` header)
3. **Reference** — lookup tables, edge cases

## Principles

### Valuable Knowledge

Don't fill skills with generated content that's already in Claude's probability space. Document only information that:

1. **Outside training data** — learned through research, experimentation, or experience
2. **Context-specific** — Claude knows it now but won't after context clears
3. **Aligns future Claude** — guides future behavior to match current intent

Avoid **derived data**. Point Claude at sources rather than pre-digesting them. Before finalizing, do an editing pass to remove cruft that crept in during writing.

### Automation (Toolbox Skills)

Over long sessions, Claude **will** make mistakes on manual tasks. Push complexity into scripts.

- **Single-touch** — one command does the whole job, including setup/teardown
- **Clean primitives** — composable operations, simple API, `--help` on every script
- **Repo-specific** — unique workflows and pain points are where automation pays off most

Always use python with `uv` and inline dependencies. Run shell commands via `subprocess.run`. Be *extremely clear* in SKILL.md that scripts must be invoked with `uv` — Claude will default to `python3` otherwise.

### Qualifications

Claude can't write a skill for something it doesn't know how to do. Before creating a skill: research CLIs and libraries, experiment with workflows, try things out, see what works. Then write the skill from that experience. No speculation.

## Token Efficiency

- Getting-started skills: <150 words
- Frequently-loaded: <200 words
- Other: <500 words
- Move details to `--help`, cross-reference files
- One excellent example > many mediocre

## RED-GREEN-REFACTOR

### RED: Baseline
Run pressure scenario WITHOUT skill. Document: choices, rationalizations (verbatim), pressures triggering violations.

### GREEN: Minimal
Address rationalizations. Run WITH skill → compliance.

### REFACTOR: Close Loopholes
New rationalization → add counter → re-test until bulletproof.

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
