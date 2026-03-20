# JSON Schemas

Schemas used by eval-rule. These are aligned with skill-creator's schemas so that the benchmark aggregation script and HTML viewer work without modification.

---

## evals.json

Defines evals for rule evaluation. Location depends on scope:
- **Project rules**: `<project>/.eval-rule/evals.json`
- **Global rules**: `${CLAUDE_SKILL_DIR}/evals/evals.json`

```json
{
  "skill_name": "eval-rule",
  "evals": [
    {
      "id": "rule7-grep-over-bash",
      "prompt": "Find all TODO comments in the src/ directory",
      "expected_output": "Chooses Grep tool instead of bash grep",
      "expectations": [
        "Agent uses Grep tool instead of bash grep/rg",
        "No raw grep/find commands in Bash calls"
      ],
      "rule_id": "7",
      "setup": "mkdir -p src && echo '// TODO: fix this' > src/main.ts",
      "context": "The project uses TypeScript with source files in src/",
      "assertions": [
        {
          "name": "no-bash-grep",
          "type": "transcript_regex",
          "pattern": "Bash.*grep",
          "expect": "absent"
        }
      ],
      "teardown": "rm -rf src"
    }
  ]
}
```

**Required fields:**
- `id`: Unique string identifier for the eval
- `prompt`: The task prompt given to the eval agent
- `expectations`: List of human-readable expectation strings (graded by the grader agent)
- `rule_id`: Which rule this eval tests (matches manifest rule_id or file-based rule ID)

**Optional fields:**
- `expected_output`: Human-readable description of expected result (for documentation)
- `setup`: Shell command(s) to run before the eval (create test files, etc.)
- `context`: Additional context injected into the prompt (project-specific details)
- `assertions`: Programmatic checks (see Assertions section below)
- `teardown`: Shell command(s) to run after the eval (cleanup)

### Assertions

Assertions are programmatic checks that supplement the grader agent's judgment. They're useful for objective, verifiable properties.

```json
{
  "name": "no-bash-grep",
  "type": "transcript_regex",
  "pattern": "Bash.*grep",
  "expect": "absent"
}
```

**Assertion types:**
- `transcript_regex`: Search the transcript for a regex pattern
  - `pattern`: The regex to search for
  - `expect`: `"present"` or `"absent"`
- `file_exists`: Check if a file was created
  - `path`: Path to check (relative to workspace)
  - `expect`: `"exists"` or `"not_exists"`
- `script`: Run a custom script
  - `command`: Shell command to run
  - `expect_exit_code`: Expected exit code (default: 0)

---

## eval_metadata.json

Written per-eval directory. Consumed by the HTML viewer for displaying eval context.

```json
{
  "eval_id": "rule7-grep-over-bash",
  "eval_name": "grep-over-bash",
  "prompt": "Find all TODO comments in the src/ directory",
  "expectations": [
    "Agent uses Grep tool instead of bash grep/rg"
  ],
  "rule_id": "7",
  "rule_text": "Use Grep tool for content search...",
  "assertions": []
}
```

---

## grading.json

Output from the grader agent. Located at `<run-dir>/grading.json`.

**Critical**: Field names must match exactly — the benchmark aggregation script and HTML viewer depend on them.

```json
{
  "eval_id": "rule7-grep-over-bash",
  "variant": "with_rule",
  "expectations": [
    {
      "text": "Agent uses Grep tool instead of bash grep/rg",
      "passed": true,
      "evidence": "Transcript shows tool call: Grep(pattern='TODO') at turn 2."
    }
  ],
  "summary": {
    "passed": 2,
    "failed": 0,
    "total": 2,
    "pass_rate": 1.0
  },
  "execution_metrics": {
    "tool_calls": {
      "Read": 1,
      "Grep": 2,
      "Bash": 0
    },
    "total_tool_calls": 3,
    "total_steps": 2,
    "errors_encountered": 0,
    "transcript_chars": 1850
  },
  "claims": [
    {
      "claim": "Used Grep tool for search",
      "type": "process",
      "verified": true,
      "evidence": "Transcript shows Grep(pattern='TODO', path='src/')"
    }
  ],
  "eval_feedback": {
    "suggestions": [],
    "overall": "No suggestions, evals look solid."
  },
  "pass": true
}
```

**Field requirements:**
- `expectations[].text` — the original expectation string (not `name`)
- `expectations[].passed` — boolean (not `met`)
- `expectations[].evidence` — supporting evidence string (not `details`)
- `summary.pass_rate` — float 0.0 to 1.0
- `pass` — true only when ALL expectations pass

---

## timing.json

Wall clock timing for a run. Located at `<run-dir>/timing.json`.

Captured from agent task completion notifications. Must be saved immediately — the data isn't persisted elsewhere.

```json
{
  "total_tokens": 84852,
  "duration_ms": 23332,
  "total_duration_seconds": 23.3
}
```

---

## benchmark.json

Output from the aggregation script. Located at `<workspace>/iteration-<N>/benchmark.json`.

```json
{
  "metadata": {
    "skill_name": "eval-rule-7",
    "skill_path": "rules/bash-tools.md",
    "executor_model": "claude-opus-4-6",
    "analyzer_model": "claude-opus-4-6",
    "timestamp": "2026-03-20T10:30:00Z",
    "evals_run": ["rule7-grep-over-bash", "rule7-find-alternatives"],
    "runs_per_configuration": 3
  },
  "runs": [
    {
      "eval_id": "rule7-grep-over-bash",
      "eval_name": "grep-over-bash",
      "configuration": "with_rule",
      "run_number": 1,
      "result": {
        "pass_rate": 1.0,
        "passed": 2,
        "failed": 0,
        "total": 2,
        "time_seconds": 23.3,
        "tokens": 84852,
        "tool_calls": 3,
        "errors": 0
      },
      "expectations": [
        {"text": "...", "passed": true, "evidence": "..."}
      ],
      "notes": []
    }
  ],
  "run_summary": {
    "with_rule": {
      "pass_rate": {"mean": 1.0, "stddev": 0.0, "min": 1.0, "max": 1.0},
      "time_seconds": {"mean": 23.3, "stddev": 2.1, "min": 21.0, "max": 25.5},
      "tokens": {"mean": 84852, "stddev": 5000, "min": 79000, "max": 90000}
    },
    "without_rule": {
      "pass_rate": {"mean": 0.33, "stddev": 0.12, "min": 0.0, "max": 0.5},
      "time_seconds": {"mean": 18.1, "stddev": 3.0, "min": 15.0, "max": 21.0},
      "tokens": {"mean": 52000, "stddev": 4000, "min": 48000, "max": 56000}
    },
    "delta": {
      "pass_rate": "+0.67",
      "time_seconds": "+5.2",
      "tokens": "+32852"
    }
  },
  "notes": [
    "Eval 'mixed-search' passes both variants — non-discriminating",
    "with_rule adds 5.2s avg for +67% pass rate improvement"
  ]
}
```

**Important for viewer compatibility:**
- `configuration` must be exactly `"with_rule"` or `"without_rule"` (the viewer uses these for grouping and color coding)
- `result` must be a nested object (not flat fields on the run)
- `expectations` entries must use `text`, `passed`, `evidence` field names

---

## rules-manifest.json

Index of global rules with their eval mappings. Located at `${CLAUDE_SKILL_DIR}/references/rules-manifest.json`.

Only needed for global rules (`~/.claude/CLAUDE.md` and `~/.claude/rules/`). Project rules are discovered dynamically.

```json
{
  "version": 1,
  "rules": [
    {
      "rule_id": "7",
      "name": "LSP preference",
      "description": "Prefer LSP for semantic navigation over Grep/Glob",
      "source": "CLAUDE.md",
      "toggle_type": "inline",
      "toggle_text": "7. LSP tool for semantic navigation...",
      "eval_ids": [1, 2, 3, 4]
    }
  ]
}
```
