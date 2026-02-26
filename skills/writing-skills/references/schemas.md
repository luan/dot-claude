# Eval Schemas

All files use JSON. Shown as annotated examples — field descriptions inline.

## evals.json

Test case definitions. Lives in `<skill>/evals/evals.json` (version-controlled with the skill). Created by `scripts/init_workspace.py` if missing, populated by the user.

```json
{
  "version": "1.0",                          // schema version
  "skill": "path/to/skill",                  // absolute path to skill under test
  "cases": [
    {
      "id": "case-01",                       // unique identifier, kebab-case
      "prompt": "Create a skill for managing Docker deployments",
      "files": [],                           // additional files to provide as context (paths relative to workspace)
      "expectations": [
        {
          "criterion": "frontmatter",        // short label for this check
          "description": "Has valid YAML frontmatter with name and description fields",
          "required": true                   // required: fail → overall fail; optional: fail → noted but overall can still pass
        },
        {
          "criterion": "token-efficiency",
          "description": "SKILL.md body is under 500 lines",
          "required": false
        }
      ]
    }
  ]
}
```

**Rules:**
- `id` must be unique across all cases
- At least one `required: true` expectation per case — otherwise the case can never fail
- `description` must be concrete and falsifiable (not "appropriate" or "reasonable")

## history.json

Version tracking. Created by `scripts/init_workspace.py`, updated by `scripts/copy_version.py`.

```json
{
  "versions": [
    {
      "version": 0,                         // integer, 0-indexed
      "timestamp": "2026-02-22T20:00:00Z",  // ISO 8601 UTC
      "description": "baseline",            // what this version represents
      "git_hash": null,                      // commit hash at snapshot time, null if uncommitted
      "path": "v0/skill",                    // directory containing this version's skill files (relative to workspace)
      "average_score": null                  // mean grading score across all cases for this version (null if no scores)
    }
  ],
  "current_version": 0                       // index of the active version being worked on
}
```

## grading.json

Per-run grading output. Written by `agents/grader.md` to `grading/<case_id>_v<version>_run<run>.json`.

### Scoring rubric

| Score | Meaning | Passes? |
|-------|---------|---------|
| 1 | Criterion unaddressed or contradicted | No |
| 2 | Partially addressed; significant gaps | No |
| 3 | Met minimally; superficial | Yes |
| 4 | Met well; nuanced | Yes |
| 5 | Exemplary; all nuances covered | Yes |

Threshold: `score >= 3 → passed: true`. 1-5 scale (not 1-10): fewer false-precision decisions, faster grader calibration, aligns with education rubrics.

```json
{
  "case_id": "case-01",                      // matches evals.json case id
  "version": 0,                             // which skill version was graded
  "run": 1,                                 // run number (1-3 in improve mode, always 1 in eval mode)
  "overall_score": 4.0,                     // mean of all criteria scores
  "required_score": 4.0,                    // mean of required-only criteria scores
  "results": [
    {
      "criterion": "frontmatter",           // matches evals.json criterion label
      "score": 4,                           // integer 1-5 per rubric above
      "passed": true,                       // derived: score >= 3
      "reasoning": "Valid YAML with name and description fields; nuanced triggers"
    }
  ],
  "overall": "pass",                        // "pass" if all required criteria passed, "fail" otherwise
  "convention_compliance": {                 // writing-skills convention checks (beyond explicit expectations)
    "frontmatter": { "passed": true, "details": "Valid YAML, name+description present" },
    "description_field": { "passed": false, "details": "Leaks workflow: 'dispatches subagent per task'" },
    "token_efficiency": { "passed": true, "details": "142 words, under 150 target" },
    "progressive_disclosure": { "passed": true, "details": "Critical > Important > Reference structure" },
    "type_alignment": { "passed": true, "details": "Discipline skill has rationalization table and loophole closers" }
  },
  "claims": [                               // notable claims the executor made (for spot-checking)
    { "claim": "Loophole for 'I already know this' closed", "verified": false, "evidence": "No explicit counter" }
  ],
  "user_notes_summary": "Executor was uncertain about whether to include script examples",
  "eval_feedback": "The token-efficiency criterion is too vague — specify a concrete line count threshold",
  "execution_metrics": {}                    // copied from executor's metrics.json if available, empty object otherwise
}
```

**`score` → `passed` derivation:** `passed = score >= 3`. The `passed` field is always present for backwards compatibility but is derived from `score`. Never set `passed` independently.

**`overall_score`:** Mean of all criteria scores (float, one decimal). `required_score`: mean of scores where the criterion has `required: true`.

**`overall` logic:** `"pass"` only when every `required: true` expectation has `passed: true`. Optional failures don't affect `overall`.

**`eval_feedback`:** Grader's suggestions for improving the test cases themselves (not the skill). Empty string if no feedback.

**`convention_compliance`:** Writing-skills convention checks run by the grader beyond explicit expectations. Keys are fixed: `frontmatter`, `description_field`, `token_efficiency`, `progressive_disclosure`, `type_alignment`.

**`execution_metrics`:** Copied from the executor's `metrics.json` if present. Empty object if unavailable.

## comparison.json

Blind comparison output. Written by `agents/comparator.md` during improve mode.

```json
{
  "eval_prompt": "Create a skill for managing Docker deployments",
  "winner": "A",                            // "A" or "B" — labels are randomized, not tied to version order
  "reasoning": "Output A has a clearer description field and better progressive disclosure structure",
  "scores": {                               // per-dimension scores (1-10, integer)
    "description_quality": { "A": 8, "B": 6 },
    "convention_compliance": { "A": 9, "B": 5 },
    "type_specific_quality": { "A": 8, "B": 4 },
    "token_efficiency": { "A": 7, "B": 7 },
    "content_value": { "A": 8, "B": 6 }
  }
}
```

**Blinding:** The comparator receives outputs labeled A/B with version order randomized. It never sees version numbers during scoring.

## analysis.json

Improvement analysis. Written by `agents/analyzer.md` after comparison.

```json
{
  "winner_version": "v1",                   // actual version label (unblinded after comparison)
  "loser_version": "v0",
  "strengths": [                            // what the winner did better
    "Clearer description triggers",
    "Better progressive disclosure"
  ],
  "weaknesses": [                           // what the loser got wrong
    "Vague 'When to Use' section"
  ],
  "suggestions": [                          // concrete changes to apply next iteration
    {
      "area": "description",                // which part of the skill to change
      "before": "Use when deploying",       // current text (or summary)
      "after": "Use when managing Docker container lifecycle...",
      "reasoning": "Broader trigger surface reduces false negatives"
    }
  ]
}
```

**`suggestions`:** Each entry must include `before`/`after` so the change is directly actionable without re-reading the full skill.
