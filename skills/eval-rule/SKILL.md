---
name: eval-rule
description: "Evaluate whether a CLAUDE.md rule actually changes Claude's behavior. Runs the same prompts with and without the rule, then grades the difference. Produces comprehensive benchmark reports with an interactive HTML viewer. Supports project-scoped evals that live alongside the rules they test. Triggers: /eval-rule, 'test this rule', 'does this rule work', 'evaluate rule', 'benchmark rule', 'rule effectiveness'. Use whenever someone questions whether a rule in CLAUDE.md is pulling its weight."
allowed-tools: Agent, Bash, Read, Glob, Grep, Write, Edit
argument-hint: "<rule-id-or-path> [--runs N] [--init]"
user-invocable: true
---

# Eval Rule

Behavioral evaluation for CLAUDE.md rules. Answers: "Does this rule actually change how Claude behaves?" by running identical prompts with and without the rule, grading outputs against expectations, aggregating benchmark statistics, and presenting results in an interactive HTML viewer.

## Arguments

- `<rule-id-or-path>` — a rule identifier. Can be:
  - Numeric ID from the global `rules-manifest.json` (e.g., `7`)
  - File path to a rule file (e.g., `rules/bash-tools.md`, `.claude/rules/my-rule.md`)
  - Inline text in quotes (e.g., `"Always use fd instead of find"`)
- `--runs N` — number of times to run each eval for variance reduction (default: 1)
- `--init` — initialize eval infrastructure for the current project (creates `.eval-rule/` directory with scaffolded evals for discovered rules)

## Core Concepts

### Project-Scoped Evals

Evals belong where the rules live. When evaluating a rule in a project's CLAUDE.md or `.claude/rules/`, the evals live in that project — not in the eval-rule skill directory.

**Discovery order:**
1. `<project>/.eval-rule/evals.json` — project-specific evals (preferred)
2. `${CLAUDE_SKILL_DIR}/evals/evals.json` — bundled evals for global `~/.claude/rules/` and `~/.claude/CLAUDE.md`

A project's evals test rules in the context where they matter — with the project's real files, real conventions, and real edge cases. Global evals are generic by necessity; project evals can reference actual paths, actual module names, actual patterns.

### Rule Discovery

Rules are discovered dynamically based on where they live:

**Global rules** (always available):
- `~/.claude/CLAUDE.md` numbered rules → looked up in `${CLAUDE_SKILL_DIR}/references/rules-manifest.json`
- `~/.claude/rules/*.md` → matched by filename or `rule_id` in manifest

**Project rules** (when running in a project directory):
- `CLAUDE.md` in project root → parsed for rule blocks
- `.claude/rules/*.md` → each file is a rule (filename = rule ID)
- The rule's `toggle_text` is extracted from the file content

When a rule path points into a project, the project's `.eval-rule/evals.json` is checked first.

## Pipeline

### [1] Parse Args & Discover Rule

1. Parse the rule identifier from arguments
2. Resolve the rule:
   - **Numeric ID**: look up in `${CLAUDE_SKILL_DIR}/references/rules-manifest.json`
   - **File path**: read the file, extract its content as `rule_text` and `toggle_text`
   - **Inline text**: use directly as both `rule_text` and `toggle_text`
3. Determine rule scope:
   - Path under `~/.claude/` → **global** scope, use skill's bundled evals
   - Path under current project → **project** scope, use project's `.eval-rule/evals.json`
4. Extract: `rule_id`, `rule_text`, `source`, `toggle_text`, `eval_ids`

### [2] Load Evals

Based on rule scope, load evals from the appropriate location:

**Project-scoped:**
```
<project>/.eval-rule/evals.json
```

**Global:**
```
${CLAUDE_SKILL_DIR}/evals/evals.json
```

Filter to entries whose `rule_id` matches the target rule. See `${CLAUDE_SKILL_DIR}/references/schemas.md` for the full eval schema.

Each eval has at minimum:
```json
{
  "id": "rule7-grep-over-bash",
  "prompt": "Find all TODO comments in src/",
  "expectations": [
    "Agent uses Grep tool instead of bash grep/rg"
  ],
  "rule_id": "7"
}
```

Enhanced evals can also include:
- `setup`: shell commands to run before the eval (create test files, etc.)
- `context`: additional context injected into the prompt (project-specific details)
- `assertions`: programmatic checks (scripts that verify outputs)
- `teardown`: cleanup commands after the eval

If no evals match the rule, suggest running `--init` to scaffold evals, or offer to help the user write them interactively.

### [3] Set Up Workspace

Create the workspace directory for this evaluation run:

**Project-scoped:**
```
<project>/.eval-rule-workspace/iteration-<N>/
```

**Global:**
```
${CLAUDE_SKILL_DIR}/eval-rule-workspace/iteration-<N>/
```

Determine `<N>` by finding the highest existing iteration number and incrementing. If no previous iterations exist, start at 1.

For each eval, create a directory:
```
iteration-<N>/
├── eval-<id>/
│   ├── eval_metadata.json    ← prompt, expectations, rule context
│   ├── with_rule/
│   │   └── run-<N>/
│   │       ├── transcript.md
│   │       ├── outputs/transcript.md  ← copy for viewer compatibility
│   │       ├── timing.json
│   │       └── grading.json
│   └── clean_baseline/
│       └── run-<N>/
│           ├── transcript.md
│           ├── outputs/transcript.md
│           ├── timing.json
│           └── grading.json
```

Write `eval_metadata.json` for each eval directory:
```json
{
  "eval_id": "rule7-grep-over-bash",
  "eval_name": "grep-over-bash",
  "prompt": "Find all TODO comments in src/",
  "expectations": ["..."],
  "rule_id": "7",
  "rule_text": "...",
  "assertions": []
}
```

### [4] Run Setup Commands

If any evals have a `setup` field, run those commands now to prepare the environment (create test files, seed data, etc.). Track setup success/failure.

### [5] Spawn Eval Agents

Eval agents run in two configurations. The key challenge is **true isolation**: subagents inherit the parent session's full CLAUDE.md context, so spawning an in-session agent "without the rule" is meaningless — the rule is still loaded. The only reliable isolation is running a separate `claude -p` subprocess with the rule file temporarily disabled.

#### Configuration: `with_rule`

Use a normal in-session subagent. The rule is already in the user's loaded config, and we prepend it again for emphasis:

```
Agent(
  prompt: "IMPORTANT INSTRUCTION — follow this rule:\n\n<toggle_text>\n\n---\n\n<eval prompt with context>",
  mode: "auto"
)
```

These can run in parallel via background agents.

#### Configuration: `clean_baseline`

Run via `claude -p` subprocess with the target rule **temporarily disabled on disk**. This gives a true baseline — the model running without the rule in its context.

**For file-based rules** (`~/.claude/rules/bash-tools.md`):
```bash
RULE_PATH="<path to rule file>"
BACKUP="${RULE_PATH}.evaloff"

# Disable
mv "$RULE_PATH" "$BACKUP"

# Run baseline eval in a clean project directory (no project CLAUDE.md)
TRANSCRIPT=$(cd /tmp/eval-clean-project && claude -p "<eval prompt>" \
  --dangerously-skip-permissions \
  --allowedTools "Bash,Read,Write,Edit,Glob,Grep" \
  2>&1)

# Restore immediately
mv "$BACKUP" "$RULE_PATH"
```

**For inline CLAUDE.md rules** (numbered rules in `~/.claude/CLAUDE.md`):
```bash
CLAUDE_MD="$HOME/.claude/CLAUDE.md"
BACKUP="${CLAUDE_MD}.evaloff"

# Backup and edit
cp "$CLAUDE_MD" "$BACKUP"
# Remove the specific rule line(s) from CLAUDE.md
# (use sed or python to strip the target rule text)

# Run baseline
TRANSCRIPT=$(cd /tmp/eval-clean-project && claude -p "<eval prompt>" ...)

# Restore
mv "$BACKUP" "$CLAUDE_MD"
```

**Safety**: Always wrap in a trap to ensure restoration on failure:
```bash
trap 'mv "$BACKUP" "$RULE_PATH" 2>/dev/null' EXIT INT TERM
```

**Clean project directory**: Create `/tmp/eval-clean-project/` (empty dir, no CLAUDE.md) so project-level rules don't contaminate the baseline.

**Why not in-session subagents for baseline?** Subagents inherit the parent's loaded CLAUDE.md context. Even if you don't prepend the rule, it's already in their system context. The only way to truly remove a rule is to prevent it from being loaded at session start — which requires a separate `claude -p` process with the rule file absent from disk.

#### Parallelism

- All `with_rule` agents can run in parallel (background subagents)
- `clean_baseline` runs must be sequential (they mutate shared files on disk)
- Run all `with_rule` agents first in parallel, then run `clean_baseline` sequentially

#### Timing capture

For subagents: save timing from the task completion notification to `timing.json`.
For `claude -p` runs: wrap with `time` or record wall clock before/after.

```json
{
  "total_tokens": 84852,
  "duration_ms": 23332,
  "total_duration_seconds": 23.3
}
```

#### Transcript capture

For subagents: save the full agent response as `transcript.md`.
For `claude -p` runs: the stdout IS the transcript. Save it directly. To capture tool calls, use `--output-format stream-json` and parse, or ask the prompt to "show the exact command you chose" so the tool choice is visible in the text output.

### [6] Grade

For each transcript, spawn a grader agent following `${CLAUDE_SKILL_DIR}/agents/grader.md`.

**Grader prompt:**
```
You are a grader agent. Read and follow the instructions in ${CLAUDE_SKILL_DIR}/agents/grader.md completely.

Inputs:
- eval_id: "<eval_id>"
- variant: "<with_rule or clean_baseline>"
- expectations: <expectations array>
- transcript_path: "<path to transcript.md>"
- output_dir: "<path to run directory>"

Grade each expectation against the transcript and write grading.json.
```

Spawn graders for the same eval's with_rule and clean_baseline in the same turn (they read independent transcripts).

**Run programmatic assertions**: If the eval has an `assertions` field with scripts, run them now and merge results into `grading.json`.

**Validation**: After grading, verify each `grading.json`:
1. Every expectation has a corresponding entry with `text`, `passed`, `evidence`
2. `summary` has correct counts
3. `pass` is true only when all expectations passed

### [7] Aggregate Benchmark

Use skill-creator's aggregation script to produce `benchmark.json` and `benchmark.md`:

```bash
SKILL_CREATOR_DIR=$(fd -t f "aggregate_benchmark.py" ~/.claude/plugins --max-results 1 -x dirname {})
python "${SKILL_CREATOR_DIR}/aggregate_benchmark.py" \
  <workspace>/iteration-<N> \
  --skill-name "eval-rule-${RULE_ID}"
```

This produces per-eval and aggregate pass rates for `with_rule` vs `clean_baseline`, with mean and stddev when `--runs N > 1`.

### [8] Analyst Pass

Spawn an analyzer agent following `${CLAUDE_SKILL_DIR}/agents/analyzer.md` to surface patterns the aggregate stats might hide:

- **Non-discriminating evals**: both variants produce the same result — the eval doesn't test the rule's effect
- **Regressions**: clean_baseline passes but with_rule fails — the rule makes things worse
- **Flaky evals**: high variance across runs (stddev > 0.3) — unreliable signal
- **Per-assertion patterns**: specific expectations that always/never pass regardless of rule
- **Cost analysis**: token/time overhead of the rule vs behavioral improvement

The analyzer writes notes into `benchmark.json`'s `notes` array.

### [9] Launch Viewer

Use skill-creator's HTML viewer for interactive review:

```bash
SKILL_CREATOR_DIR=$(fd -t f "generate_review.py" ~/.claude/plugins --max-results 1 -x dirname {})
nohup python "${SKILL_CREATOR_DIR}/generate_review.py" \
  <workspace>/iteration-<N> \
  --skill-name "eval-rule-${RULE_ID}" \
  --benchmark <workspace>/iteration-<N>/benchmark.json \
  > /dev/null 2>&1 &
VIEWER_PID=$!
```

For iteration 2+, also pass `--previous-workspace <workspace>/iteration-<N-1>`.

The viewer has two tabs:
- **Outputs**: Shows each eval's transcript side-by-side (with_rule vs clean_baseline), grading results, and a feedback textbox
- **Benchmark**: Shows aggregate statistics, per-eval breakdowns, and analyst observations

### [10] Present Summary

Display a text summary alongside the viewer URL:

```
Rule: <rule_id> — <rule description>
Source: <rule source file>
Scope: project | global
Runs per eval: N
Iteration: <N>

┌──────────────────────────┬───────────┬──────────────┬───────┐
│ Eval ID                  │ with_rule │ clean_baseline │ Delta │
├──────────────────────────┼───────────┼──────────────┼───────┤
│ rule7-grep-over-bash     │ 100%      │ 0%           │ +100% │
│ rule7-find-alternatives  │ 100%      │ 33%          │ +67%  │
│ rule7-mixed-search       │ 100%      │ 100%         │  0%   │
└──────────────────────────┴───────────┴──────────────┴───────┘

Aggregate: with_rule 100% — clean_baseline 44% — Delta +56%
Time: with_rule 23.3s avg — clean_baseline 18.1s avg (+5.2s)
Tokens: with_rule 84K avg — clean_baseline 52K avg (+32K)

Analyst Notes:
- "rule7-mixed-search" passes both variants — not testing the rule's effect
- with_rule adds 5.2s avg execution time for +56% pass rate improvement

Verdict: RULE IS EFFECTIVE
  Behavioral improvement in 2/3 evals. 1 eval is non-discriminating.

Viewer: http://localhost:3117
```

**Verdict logic:**
- **EFFECTIVE**: majority of evals show behavioral difference in the expected direction
- **INEFFECTIVE**: no evals show meaningful difference
- **HARMFUL**: regressions outnumber improvements
- **INCONCLUSIVE**: high variance or insufficient data (suggest `--runs 3`)

After presenting, tell the user: "The interactive viewer is open in your browser. The Outputs tab lets you review each eval's transcript and leave feedback. The Benchmark tab shows the quantitative comparison. When you're done, come back here."

### [11] Process Feedback (if user returns)

If the user provides feedback through the viewer or conversation:

1. Read `feedback.json` from the workspace
2. Identify evals that need attention (non-empty feedback)
3. Suggest improvements to:
   - The rule text (if the rule isn't achieving its goal)
   - The eval prompts (if evals aren't testing the right things)
   - The expectations (if assertions are too weak or too strict)

Kill the viewer when done:
```bash
kill $VIEWER_PID 2>/dev/null
```

## Project Init (`--init`)

When invoked with `--init`, scaffold eval infrastructure for the current project:

1. **Discover rules**: Scan the project's CLAUDE.md and `.claude/rules/` for rules
2. **Create directory structure**:
   ```
   <project>/.eval-rule/
   ├── evals.json          ← scaffolded evals for discovered rules
   └── README.md           ← brief explanation of the eval format
   ```
3. **Generate eval stubs**: For each discovered rule, create 2-3 eval entries with:
   - Realistic prompts that test the rule's behavioral effect
   - Expectations that distinguish with-rule from without-rule behavior
   - Context referencing actual project files/patterns where relevant
4. **Present to user**: Show the discovered rules and generated evals, ask the user to review and refine

The generated evals should be *project-specific* — referencing actual paths, modules, and patterns from the project rather than generic examples.

## Iteration Support

Each evaluation run creates a new iteration directory. This enables:

- **A/B testing rule changes**: Modify a rule, re-run evals, compare iterations
- **Eval refinement**: Improve evals based on feedback, re-run to verify
- **Historical tracking**: See how a rule's effectiveness changes over time

The viewer supports `--previous-workspace` to show side-by-side comparisons between iterations.

## Eval Writing Guide

Good evals for rules share these properties:

**Discriminating**: The eval reliably produces different behavior with vs without the rule. If Claude already follows the behavior without the rule, the eval won't show the rule's value.

**Realistic**: The prompt sounds like something a real user would type — specific details, casual phrasing, context about what they're trying to do. Not abstract or robotic.

**Targeted**: Each eval tests one specific aspect of the rule's behavioral effect. Broad evals dilute the signal.

**Edge-case-weighted**: The most valuable evals test situations where Claude's default behavior conflicts with the rule — where the rule has to actively override a natural tendency.

### Anti-patterns

- **Evals that always pass**: If Claude naturally does X without the rule, testing for X is useless
- **Evals that test knowledge, not behavior**: "Does Claude know about fd?" vs "Does Claude use fd when asked to find files?"
- **Evals with weak expectations**: "Agent mentions Grep" vs "Agent's first tool call is Grep, not Bash with grep"
- **Evals with no negative case**: The clean_baseline variant should plausibly fail — otherwise the rule isn't needed
