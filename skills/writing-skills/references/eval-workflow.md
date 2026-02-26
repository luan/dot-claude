# Eval Workflow

Two modes: **Eval** (measure) and **Improve** (iterate). Both share the same grading pipeline.

## Eval Mode

Runs the skill against test cases and reports pass/fail. Use to check acceptance criteria or regression-test after edits.

1. **Init workspace** — `uv run scripts/init_workspace.py <skill-path>` creates `v0/skill/`, `history.json`, `grading/` in the workspace, and `<skill>/evals/evals.json` if missing
2. **Define test cases** — populate `<skill>/evals/evals.json` with prompts and expectations (see `references/schemas.md`). Fixtures go in `<skill>/evals/fixtures/`.
3. **Execute** — `agents/executor.md` runs the skill on each test case, writes output to workspace
4. **Grade** — `agents/grader.md` scores each output against expectations, writes `grading/<case_id>_v<N>_run<R>.json`
5. **Aggregate** — `uv run scripts/aggregate_results.py <workspace>` computes pass rates and scores, prints dual tables (pass-rate + score)
6. **Report** — display aggregate tables, flag failing cases

## Improve Mode

Iterates on the skill until it meets quality targets. Use when eval reveals the skill isn't good enough.

1. **Baseline** — run Eval mode to establish v0 scores
2. **Iterate** — for each cycle:
   - Executor runs 3x per test case (captures variance)
   - Grader scores all runs → `grading/`
   - Blind comparator (`agents/comparator.md`) picks winner between current and previous version
   - Analyzer (`agents/analyzer.md`) identifies what improved and suggests next changes
   - Apply suggestions to skill files
   - Snapshot: `uv run scripts/copy_version.py <workspace> --description "what changed"`
3. **Stop when** any of:
   - Target pass rate reached
   - No improvement in 2 consecutive iterations
   - Time budget exhausted
   - User says stop
4. **Best version wins** — not necessarily the latest; pick the version with highest pass rate and score from `history.json`. Compare using score trends (mean ± range across runs) when pass rates are tied.

## Eval Quality Loop

Run this check before improving the skill itself. Lax evals waste improvement cycles.

**If grader passes everything on the first run (100% v0 pass rate), the evals are too easy.** Tighten `evals.json` before proceeding. Similarly, **if all criteria score 5 on baseline**, the evals lack discrimination — a perfect score on attempt one means the rubric isn't testing anything hard.

Signs of lax grading:
- All cases pass on baseline
- All criteria score 5 on baseline (ceiling effect)
- Criteria use vague words ("appropriate", "reasonable", "good")
- Every criterion is `required: false`
- No criterion tests a concrete, falsifiable property

Fix: rewrite expectations with specific, measurable criteria. Add `required: true` to must-have properties. Add edge-case test cases that probe known failure modes.

## Graceful Degradation (No Subagents)

When subagents are unavailable (permission restrictions, context limits), the main agent runs the pipeline inline with reduced rigor.

| Full pipeline | Degraded pipeline |
|---|---|
| Executor runs 3x per case | Single run per case |
| Separate grader agent | Main agent follows `agents/grader.md` inline |
| Blind comparator picks winner | Skip — can't blind yourself |
| Analyzer suggests changes | Main agent analyzes diffs directly |

Acknowledge reduced rigor in the output: "Running in single-agent mode — results have higher variance and no blind comparison."

The workflow steps remain the same; only the execution method changes. All schemas and file formats are identical.
