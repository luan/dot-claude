---
name: rule-creator
description: "Create, evaluate, and iterate on CLAUDE.md rules. Drafts rules from behavioral intent, tests whether they actually change Claude's behavior via isolated baselines, and produces benchmark reports with an interactive HTML viewer. Supports project-scoped evals that live alongside the rules they test. Triggers: /rule-creator, 'test this rule', 'does this rule work', 'evaluate rule', 'benchmark rule', 'rule effectiveness', 'create a rule', 'new rule', 'write a rule'. Use whenever someone wants to create, evaluate, or iterate on rules in CLAUDE.md or .claude/rules/."
allowed-tools: Agent, Bash, Read, Glob, Grep, Write, Edit
argument-hint: "<rule-id-or-path | --new 'intent'> [--runs N] [--naked] [--init]"
user-invocable: true
---

# Rule Creator

Create and test CLAUDE.md rules. Two modes:

- **Create**: Draft a rule from behavioral intent, generate evals, test it, iterate until effective
- **Evaluate**: Test an existing rule to see if it actually changes Claude's behavior

## Arguments

- `<rule-id-or-path>` — evaluate an existing rule (numeric ID, file path, or inline text)
- `--new "intent"` — create a new rule from a behavioral description
- `--runs N` — runs per eval for variance reduction (default: 1)
- `--naked` — baseline uses fully stripped config (all rules + CLAUDE.md disabled, not just the target rule)
- `--init` — scaffold `.rule-creator/` eval infrastructure for the current project

## Creating Rules (`--new`)

When the user wants a new rule, the goal is to go from vague intent to a tested, effective rule file.

### Step 1: Understand the Intent

Ask what behavior they want Claude to adopt. Good rules override a default tendency — if Claude already does it naturally, a rule is unnecessary. Probe for:

- What should Claude do differently?
- When should this behavior apply? (always, or only in certain contexts?)
- What's the failure mode without the rule? (what bad behavior are they seeing?)

### Step 2: Draft the Rule

Write a concise rule file. Effective rules share these properties:

- **Specific**: "Use fd instead of find in Bash" beats "use modern tools"
- **Observable**: The behavioral change is visible in tool calls or output, not just internal reasoning
- **Explains why**: Claude follows rules better when it understands the motivation
- **Scoped**: Include `paths` frontmatter if the rule only applies to certain file types or directories

Write to `~/.claude/rules/<topic>.md` for global rules, or `<project>/.claude/rules/<topic>.md` for project rules.

### Step 3: Generate Evals

Create 2-4 eval prompts that test the rule's behavioral effect. Each eval should:

- Be realistic (sounds like something a user would actually type)
- Force the behavioral choice the rule governs (the prompt should create a situation where the rule matters)
- Use `tools` to restrict available tools when testing Bash-level behavior (prevents sidestepping to dedicated tools)

Save to the appropriate `evals.json` (project or global scope).

### Step 4: Run the Eval Pipeline

Execute the full evaluation pipeline (see below) to verify the rule works. The key metric: does `with_rule` behave differently from `clean_baseline`?

### Step 5: Iterate

If the rule doesn't show a clear behavioral signal:
- The rule text may be too vague — make it more specific
- The evals may not test the right thing — revise the prompts
- The rule may be unnecessary — Claude already does it

If the rule shows regressions (baseline is better), the rule may be too aggressive or poorly scoped.

## Evaluating Rules

The evaluation pipeline answers: "Does this rule actually change how Claude behaves?"

### Project-Scoped Evals

Evals belong where the rules live:
1. `<project>/.rule-creator/evals.json` — project-specific evals (preferred)
2. `${CLAUDE_SKILL_DIR}/evals/evals.json` — bundled evals for global rules

### Eval Schema

```json
{
  "id": "bash-fd-over-find",
  "prompt": "Find all .yaml files recursively. Use the Bash tool.",
  "expectations": ["Uses fd or rg --files, not find"],
  "rule_id": "bash-tools",
  "tools": "Bash"
}
```

The `tools` field is critical for behavioral evals — it controls which tools the `claude -p` baseline subprocess gets via `--tools`. Without it, Claude can sidestep to dedicated tools (Glob, Grep) that make the Bash-level rule irrelevant. Set `tools` to the minimal set needed to test the rule's behavioral domain.

See `${CLAUDE_SKILL_DIR}/references/schemas.md` for the full schema including `setup`, `context`, `assertions`, and `teardown` fields.

### Pipeline Overview

1. **Resolve rule** — find the rule file, extract toggle text
2. **Load evals** — filter evals matching this rule
3. **Set up workspace** — create `iteration-<N>/` directory structure
4. **Run with_rule** — in-session subagents with rule in context (parallel)
5. **Run clean_baseline** — `claude -p` subprocess with rule disabled on disk (sequential)
6. **Grade** — grader agents evaluate expectations against transcripts
7. **Aggregate** — benchmark.json with pass rates, timing, tokens
8. **Analyze** — surface non-discriminating evals, regressions, patterns
9. **Present** — text summary + static HTML viewer

### Phase 5: Isolation Mechanism (the hard part)

Subagents inherit the parent session's full CLAUDE.md context, so in-session "without rule" tests are meaningless — the rule is already loaded. True isolation requires a separate `claude -p` subprocess with the rule file temporarily absent from disk.

#### with_rule (in-session subagent)

```
Agent(
  prompt: "IMPORTANT INSTRUCTION — follow this rule:\n\n<toggle_text>\n\n---\n\n<eval prompt>",
  mode: "auto"
)
```

Run all with_rule agents in parallel as background tasks.

#### clean_baseline (isolated subprocess)

Run via a bash script that temporarily disables the rule, spawns `claude -p`, and restores:

```bash
#!/usr/bin/env bash
set -euo pipefail

RULE_PATH="$1"     # e.g. ~/.claude/rules/bash-tools.md
PROMPT="$2"         # the eval prompt
TOOLS="${3:-}"      # optional: tool restriction (e.g. "Bash")
BACKUP="${RULE_PATH}.evaloff"

# Safety: restore on any exit
cleanup() { [ -f "$BACKUP" ] && mv "$BACKUP" "$RULE_PATH"; }
trap cleanup EXIT INT TERM

# Disable rule
mv "$RULE_PATH" "$BACKUP"

# Build claude -p command
CMD=(claude -p "$PROMPT" --dangerously-skip-permissions)
[ -n "$TOOLS" ] && CMD+=(--tools "$TOOLS")

# Run in clean project dir (no project-level CLAUDE.md)
mkdir -p /tmp/rule-creator-eval
TRANSCRIPT=$(cd /tmp/rule-creator-eval && "${CMD[@]}" 2>&1)

# trap handles restore
echo "$TRANSCRIPT"
```

For `--naked` mode (full isolation), disable ALL config:
```bash
# Move entire rules directory and CLAUDE.md
mv ~/.claude/rules ~/.claude/rules.evaloff
mv ~/.claude/CLAUDE.md ~/.claude/CLAUDE.md.evaloff
# ... run claude -p ...
# trap restores both
```

**Parallelism**: clean_baseline runs are sequential (they mutate shared files). Run all with_rule agents first in parallel, then clean_baseline runs one at a time.

**Timing**: Record wall clock around each `claude -p` call. For subagents, capture `total_tokens` and `duration_ms` from task completion notifications.

**Transcript capture**: `claude -p` stdout is the transcript. Append "Show me the EXACT command/tool you chose" to the prompt so tool choices are visible in text output. Save to both `transcript.md` and `outputs/transcript.md` (the viewer requires the `outputs/` directory).

### Phase 7: Aggregation

```bash
SCRIPTS_DIR="$HOME/.claude/plugins/cache/claude-plugins-official/skill-creator"
SCRIPTS_DIR="$(fd -t d 'skill-creator' "$SCRIPTS_DIR" | head -1)/skills/skill-creator"
cd "$SCRIPTS_DIR" && python3 -m scripts.aggregate_benchmark \
  <workspace>/iteration-<N> \
  --skill-name "rule-<RULE_ID>"
```

### Phase 9: Viewer

Use `--static` to generate a standalone HTML file (the server mode can be unreliable):

```bash
VIEWER="$SCRIPTS_DIR/eval-viewer/generate_review.py"
python3 "$VIEWER" <workspace>/iteration-<N> \
  --skill-name "rule-<RULE_ID>" \
  --benchmark <workspace>/iteration-<N>/benchmark.json \
  --static /tmp/rule-eval-<RULE_ID>.html
open /tmp/rule-eval-<RULE_ID>.html
```

For iteration 2+, add `--previous-workspace <workspace>/iteration-<N-1>`.

### Present Summary

```
Rule: bash-tools — Bash Tool Replacements
Source: ~/.claude/rules/bash-tools.md
Mode: single-rule isolation | --naked
Runs per eval: 1

| Eval                 | with_rule | clean_baseline | Delta |
|----------------------|-----------|----------------|-------|
| bash-fd-over-find    | PASS      | FAIL           | +1    |
| bash-rg-over-grep    | PASS      | FAIL           | +1    |
| bash-rg-over-cat-grep| PASS      | FAIL           | +1    |

Aggregate: with_rule 100% — clean_baseline 0%
Verdict: RULE IS EFFECTIVE
```

**Verdicts:**
- **EFFECTIVE**: majority of evals show behavioral difference in expected direction
- **INEFFECTIVE**: no evals show meaningful difference
- **HARMFUL**: regressions outnumber improvements
- **INCONCLUSIVE**: high variance — suggest `--runs 3`

## Project Init (`--init`)

Scaffold eval infrastructure for the current project:

1. Scan `CLAUDE.md` and `.claude/rules/` for rules
2. Create `.rule-creator/evals.json` with eval stubs referencing actual project paths and patterns
3. Present discovered rules and generated evals for user review

## Eval Writing Guide

**Discriminating**: The eval must produce different behavior with vs without the rule. If Claude already does X natively, testing for X is useless.

**Tool-constrained**: Use the `tools` field to force the behavioral domain. Testing a Bash rule? Set `"tools": "Bash"` so Claude can't sidestep to Glob/Grep.

**Realistic**: Prompts should sound like a real user — specific details, casual phrasing. Not "Find files" but "I need to find all .yaml files in this project. Use the Bash tool to do it."

**Edge-case-weighted**: Test situations where Claude's default conflicts with the rule — where the rule has to actively override a natural tendency.
