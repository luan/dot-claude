---
name: eval-rule
description: "Evaluate whether a CLAUDE.md rule actually changes Claude's behavior. Runs the same prompts with and without the rule, then grades the difference. Triggers: /eval-rule, 'test this rule', 'does this rule work', 'evaluate rule', 'benchmark rule', 'rule effectiveness'. Use whenever someone questions whether a rule in CLAUDE.md is pulling its weight."
allowed-tools: Agent, Bash, Read, Glob, Grep, Skill
argument-hint: "<rule-id-or-path> [--runs N]"
user-invocable: true
---

# Eval Rule

Behavioral evaluation for CLAUDE.md rules. Answers: "Does this rule actually change how Claude behaves?" by running identical prompts with and without the rule, grading outputs against expectations, and presenting a pass/fail comparison.

## Arguments

- `<rule-id-or-path>` — numeric rule ID (e.g., `7`) matching `rules-manifest.json`, or a file path (e.g., `rules/bash-output.md`) (required)
- `--runs N` — number of times to run each eval for variance reduction (default: 1)

## Pipeline

### [1] Parse Args

Resolve the target rule:

1. If numeric (e.g., `7`): look up in `${CLAUDE_SKILL_DIR}/references/rules-manifest.json` by `rule_id`
2. If a path (e.g., `rules/bash-output.md`): match against `source` field in the manifest
3. Fail with a clear message if no manifest entry matches

Extract from the manifest entry:
- `rule_id` — canonical identifier
- `rule_text` — the full rule content
- `source` — file path where the rule lives
- `toggle_text` — the exact text block to remove/restore (used by Phase 2 toggle)
- `eval_ids` — list of eval IDs to run

Parse `--runs N` if provided (default 1).

### [2] Load Evals

Read `${CLAUDE_SKILL_DIR}/evals/evals.json`. Filter to entries whose `id` is in the manifest's `eval_ids` list.

Each eval has:
```json
{
  "id": "rule7-grep-over-bash",
  "prompt": "Find all TODO comments in the src/ directory",
  "expectations": [
    "Agent uses Grep tool instead of bash grep/rg",
    "No raw grep/find commands in Bash calls"
  ],
  "rule_id": 7
}
```

Fail if any `eval_id` from the manifest has no matching eval entry.

### [3] Toggle Rule

Produces two execution environments via **prompt injection** — the rule text is included or omitted from the agent's prompt directly. CLAUDE.md is never modified.

**Why prompt injection, not file mutation**: CLAUDE.md is loaded into system context at session start and cached. Editing it mid-session has no effect on spawned subagents — they still see the original version. Prompt injection is the only reliable way to A/B test rules within a single session.

**with_rule agent**: The eval prompt is prepended with:
```
IMPORTANT INSTRUCTION — follow this rule for all tool selection decisions:

<rule toggle_text from manifest>

---

<eval prompt>
```

**without_rule agent**: The eval prompt is sent as-is, with no rule prepended.

This produces a clean A/B comparison: the only difference between the two agents is whether the rule text appears in their prompt. No file mutation, no lock files, no restore logic needed.

### [4] Spawn Agents

For each eval, for each run (1..N):

**with_rule agent:**
```
Agent(
  prompt: "<eval prompt>",
  mode: "auto"
)
```
Save transcript to: `${WORKSPACE}/results/<eval_id>/run-<N>/with_rule/transcript.md`

**without_rule agent:**
```
Agent(
  prompt: "<eval prompt>",
  mode: "auto"
)
```
Save transcript to: `${WORKSPACE}/results/<eval_id>/run-<N>/without_rule/transcript.md`

Where `${WORKSPACE}` is `${CLAUDE_SKILL_DIR}/../eval-rule-workspace/`.

Spawn with_rule and without_rule for the same eval in the same turn to maximize parallelism.

### [5] Grade

For each transcript (with_rule and without_rule, per eval, per run), spawn a grader agent.

**Grader agent prompt:**
```
You are a grader agent. Read and follow the instructions in ${CLAUDE_SKILL_DIR}/agents/grader.md completely.

Inputs:
- eval_id: "<eval_id>"
- variant: "<with_rule or without_rule>"
- expectations: <expectations array from evals.json for this eval>
- transcript_path: "<path to transcript.md>"

Grade each expectation against the transcript and write grading.json.
```

Spawn with `Agent(prompt: <above>, mode: "auto")`.

**Grading output**: The grader writes `grading.json` as a sibling to the transcript directory:
```
${WORKSPACE}/results/<eval_id>/run-<N>/<variant>/grading.json
```

**Schema** (fields: `text`, `passed`, `evidence`):
```json
{
  "eval_id": "rule7-grep-over-bash",
  "variant": "with_rule",
  "expectations": [
    {
      "text": "Agent uses Grep tool instead of bash grep/rg",
      "passed": true,
      "evidence": "Transcript shows tool call: Grep(pattern='TODO') at turn 2. No Bash grep calls appear."
    }
  ],
  "summary": {
    "passed": 2,
    "failed": 0,
    "total": 2,
    "pass_rate": 1.0
  },
  "pass": true,
  "eval_feedback": {
    "suggestions": [],
    "overall": "No suggestions, evals look solid."
  }
}
```

**Parallelism**: Grade all transcripts from a single eval's run in the same turn (with_rule and without_rule can be graded simultaneously since they read independent transcripts).

**Validation**: After grading completes, read each `grading.json` and verify:
1. Every expectation from evals.json has a corresponding entry
2. Each entry has all three fields: `text`, `passed`, `evidence`
3. `pass` is `true` only when all expectations passed

### [6] Aggregate

Reuse skill-creator's aggregation script to produce `benchmark.json` and `benchmark.md`:

```bash
python -m scripts.aggregate_benchmark \
  ${WORKSPACE}/results \
  --skill-name "eval-rule-${RULE_ID}"
```

The script lives in the skill-creator plugin directory. Locate it dynamically:
```bash
SKILL_CREATOR_DIR=$(find ~/.claude/plugins -path "*/skill-creator/scripts/aggregate_benchmark.py" -exec dirname {} \; | head -1)
python "${SKILL_CREATOR_DIR}/../scripts/aggregate_benchmark.py" \
  ${WORKSPACE}/results \
  --skill-name "eval-rule-${RULE_ID}"
```

This produces per-eval and aggregate pass rates for with_rule vs without_rule, with mean and stddev when `--runs N > 1`.

### [7] Present

Display a summary table:

```
Rule: <rule_id> — <rule description>
Runs per eval: N

| Eval ID                | with_rule | without_rule | Delta |
|------------------------|-----------|--------------|-------|
| rule7-grep-over-bash   | PASS      | FAIL         | +1    |
| rule7-find-alternatives | PASS      | FAIL         | +1    |
| rule7-mixed-search     | PASS      | PASS         |  0    |

Aggregate: with_rule 100% — without_rule 33%
```

Highlight:
- **Regressions**: evals where without_rule passes but with_rule fails (rule makes things worse)
- **Non-discriminating**: evals where both variants produce the same result (eval doesn't test the rule's effect)
- **Effective**: evals where with_rule passes and without_rule fails (rule is working)

If `--runs N > 1`, show pass_rate as percentage instead of PASS/FAIL, and flag high-variance evals (stddev > 0.3).

End with a verdict:
- **Rule is effective** if majority of evals show behavioral difference in the expected direction
- **Rule is ineffective** if no evals show difference
- **Rule is harmful** if regressions outnumber improvements
