---
name: writing-skills
description: "Create, edit, improve, evaluate, and troubleshoot Claude Code agent skills. Use when making a new skill, editing ANY file under a skills/ directory (SKILL.md, references, scripts, evals), modifying skill metadata, refactoring skill instructions, improving skill discoverability, debugging why a skill isn't activating, running skill evals, or reviewing skill best practices. Also activates via rules/skills-editing.md when any skills/**/* file is touched. Triggers: 'new skill', 'edit skill', 'update skill', 'fix skill', 'skill not working', 'SKILL.md', 'skill metadata', 'eval skill', 'test skill', 'improve skill', 'run evals', 'skill quality'."
argument-hint: "[--run-evals [<skill-path>|all]]"
---

# Writing Skills

TDD for process docs. Write test (pressure scenario) → fail → write skill → pass → refactor.

## Editing a Skill

1. **Pre-check:** `<skill>/evals/evals.json` exists? Missing → stop and create evals (at least one case covering existing behavior) before editing. "I'll add evals after" defeats the gate — the pre-check catches regressions your edit might introduce.
2. **Read** the skill top-to-bottom — understand current structure so you don't silently break existing behavior.
3. **Edit.**
4. **Update evals:** new/changed behavior → add covering criteria. Stale evals on changed behavior prove nothing.
5. **Grade → aggregate → report scores to user.** Edit isn't done until scores confirm no regressions.

If evals are missing and you can't create them, tell the user: "This skill has no evals — I need to create them before editing."

Untested edits cause silent regressions. Non-negotiable.

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
| Personal | `$HOME/.claude/skills/<name>/SKILL.md` | All projects |
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
    evals/                # Version-controlled test definitions
      evals.json          # Test cases + expectations
      fixtures/           # Test data files referenced by cases
  skill-name-workspace/   # Runtime artifacts (gitignored)
    grading/              # Grading output files
    history.json          # Version tracking
    v0/skill/             # Versioned snapshots
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

A "workflow detail" is anything describing HOW the skill works internally — mechanisms, enforcement methods, internal steps. The description should only say what outcomes it produces and what situations trigger it.

```yaml
# BAD: mechanism leaked ("enforcing a structured checklist" is HOW, not WHEN)
description: Enforces code review quality by requiring a structured checklist before approval

# BAD: internal steps leaked
description: Use when executing plans - dispatches subagent per task with review

# GOOD: outcomes + situations only
description: Handles SpeedReader server lifecycle (build, startup, shutdown) and web page rebuild/refresh. Use when you need to verify a web page works, view it, test UI interactions, or see how a page behaves. Also covers development tasks.

# GOOD: discipline skill — describes situations, not enforcement method
description: Use when reviewing PRs, approving code changes, or assessing code quality. Prevents shallow reviews and missed issues in security, tests, and error handling.
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

**Hard gate:** After drafting a skill, count the SKILL.md body words (exclude frontmatter). Over budget → cut before finalizing. Never ship over budget.

**Cutting priority** (highest savings first):
1. Reference tables and lookup data → `references/` directory (saves 50-150 words)
2. Multi-line examples → collapse to single inline example (saves 30-80 words)
3. Output format templates → 3-line skeleton, not full example (saves 20-50 words)
4. Overlapping guidance → deduplicate (one location, reference elsewhere)

## RED-GREEN-REFACTOR

### RED: Baseline
Run pressure scenario WITHOUT skill. Document: choices, rationalizations (verbatim), pressures triggering violations.

### GREEN: Minimal
Address rationalizations. Run WITH skill → compliance. Then immediately test against a different real case — the first scenario shaped the skill, the second tests whether it generalizes.

### REFACTOR: Fresh Eyes
Re-read the skill as if seeing it for the first time. Cut instructions that don't change behavior, add reasoning to rigid rules missing their WHY, close gaps a creative model might exploit. New rationalization → add counter → re-test until bulletproof.

## Bulletproofing Discipline Skills

Discipline skills enforce process rules (TDD, verification, code review checklists). They need adversarial hardening because the model rationalizes shortcuts under pressure. This does NOT apply to knowledge or technique skills — see the next section.

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

## Writing Knowledge & Technique Skills

Knowledge and technique skills teach rather than enforce. Different failure mode: the risk isn't rule-dodging — it's that the model memorizes surface patterns without understanding the reasoning, then misapplies them in novel situations.

**Explain WHY, not just WHAT.** Every instruction should carry its reasoning. If you write ALWAYS or NEVER in caps, that's a yellow flag — reframe as an explanation so the model understands the goal and can generalize.

```markdown
# BAD: rigid rule
NEVER use inline styles.

# GOOD: reasoning the model can generalize
Avoid inline styles — they bypass the cascade, making themes
and responsive overrides impossible. Styles in one place
(stylesheet/module) propagate changes; inline styles require
hunting through markup.
```

**Theory of mind.** Explain the goal so the model generalizes to cases you didn't anticipate, rather than breaking on the first edge case your rules didn't cover.

**Generalize from examples.** Include examples to illustrate principles, but frame them as instances of the underlying pattern. If the skill only works when input looks exactly like your examples, it's overfitted.

**Keep the prompt lean.** Every instruction competes for attention. After drafting, do an editing pass: remove lines that don't change behavior. If removing an instruction doesn't degrade output quality, it wasn't pulling its weight.

## Eval & Improve

Edit procedure: see **Editing a Skill** at the top.

**Eval definitions** (`<skill>/evals/`) are version-controlled — commit them with the skill. **Workspaces** (`<skill>-workspace/`) are gitignored runtime artifacts. Never delete, modify, or flag workspaces during reviews.

### Building Blocks

| Component | Path | Role |
|-----------|------|------|
| Executor | `agents/executor.md` | Runs skill against test prompts, produces transcripts |
| Grader | `agents/grader.md` | Scores outputs against expectations (1-5 rubric) |
| Comparator | `agents/comparator.md` | Blind A/B comparison between skill versions |
| Analyzer | `agents/analyzer.md` | Post-hoc analysis with improvement suggestions |

### Eval Mode (Measure)

1. Init workspace: `uv run scripts/init_workspace.py <skill-path>`
2. Define test cases in `<skill>/evals/evals.json` (see `references/schemas.md`). Fixtures in `<skill>/evals/fixtures/`.
3. Execute each case with `agents/executor.md`
4. Grade each output with `agents/grader.md` (1-5 scored rubric)
5. Aggregate: `uv run scripts/aggregate_results.py <workspace>` (dual tables: pass-rate + scores)

### Improve Mode (Iterate)

1. Run Eval mode to establish v0 baseline — if 100% pass or all scores 5, evals are too easy; tighten first
2. Per iteration: execute 3x per case → grade → blind compare → analyze
3. Apply analyzer suggestions, snapshot: `uv run scripts/copy_version.py <workspace>`
4. Stop when: target reached, no improvement for 2 iterations, or user says stop
5. Best version wins (not necessarily latest) — check `history.json` scores

### --run-evals Mode

`/writing-skills --run-evals <skill-path>` or `/writing-skills --run-evals all`

1. **Find skills:** specific path, or glob all `skills/*/evals/evals.json`
2. **Per skill:** init workspace if needed → execute each case → grade → aggregate
3. **Report:** per-skill score summary, flag regressions vs previous version

Use after any skill edit to verify no regressions. Use `all` before major commits touching multiple skills.

### Without Subagents

Run each step inline instead of spawning agents:

| Full pipeline | Single-agent fallback |
|---|---|
| Executor agent runs cases | Main agent runs inline |
| Separate grader agent | Main agent grades following `agents/grader.md` inline |
| Blind comparator picks winner | Skipped — can't blind yourself |
| Analyzer suggests changes | Main agent analyzes diffs directly |

Always mention when using this path: _"Running in single-agent mode — no blind comparison, reduced rigor."_

For detailed workflow and schemas, see `references/eval-workflow.md` and `references/schemas.md`.

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Not triggering | Check description keywords, try `/skill-name` |
| Triggers too often | More specific description, add `disable-model-invocation` |
| Claude doesn't see | Check `/context` for character budget warning |

## Checklist

**RED:** pressure scenarios (3+ for discipline) → run WITHOUT skill → document baseline → identify rationalizations

**GREEN:** YAML frontmatter (name + description) → description triggers only → address baseline → one excellent example

**REFACTOR:** fresh-eyes re-read → cut dead instructions → identify NEW rationalizations → add counters → re-test

**Knowledge/Technique extra:** every instruction carries WHY → no unexplained ALWAYS/NEVER → examples illustrate patterns (not overfit) → editing pass for leanness

**EVAL:** evals exist in `<skill>/evals/` → new criteria cover the edit → grade passes → aggregate shows no regressions

**Deploy:** commit + push
