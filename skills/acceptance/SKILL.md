---
name: acceptance
description: "Validate implementation against acceptance criteria using dual-agent verification. Works on epics (all descendants) or individual tasks. Triggers: 'accept', 'acceptance check', 'verify implementation', 'did it work', 'check my implementation'. Also invoked automatically by /develop after completion."
argument-hint: "[<task-id>|<epic-id>] [--auto]"
user-invocable: true
allowed-tools:
  - Task
  - TaskGet
  - TaskList
  - TaskUpdate
  - Bash
  - Read
  - Glob
  - Grep
  - Write
---

# Acceptance

Advisory gate — reports findings, user decides how to proceed. Never blocks autonomously.

## Step 1: Resolve Target

- Argument → `TaskGet(taskId)`.
  - **Epic** (`metadata.type == "epic"`) → epic mode (validate all descendants)
  - **Non-epic task** → task mode (validate this task only)
- No argument → `TaskList()` filtered by `metadata.type == "epic"` AND status in `["completed", "in_progress"]`. Pick highest ID → epic mode.
- None found → stop: "No task or epic found. Pass a task ID."

## Step 2: Gather Plan Context

**Epic mode:** From `TaskGet(epicId)`: extract `metadata.design`. If `metadata.plan_file` set → `ct plan latest --task-file <plan_file>`. Falls back to metadata.design. Not found → "No plan found — checking criteria only."

**Task mode:** Walk `metadata.parent_id` to find root epic. Extract `metadata.design` from root for context. If root has `metadata.plan_file` → `ct plan latest --task-file <plan_file>`. Falls back to task's own description.

## Step 3: Gather Criteria

**Epic mode:** Recursively collect all descendants via `metadata.parent_id` chains. No descendants → stop: "Run /scope first."

Orphaned task detection: flag any task where `status == "pending"` but parent `status == "completed"`. Advisory warning only — never blocks acceptance. See `${CLAUDE_SKILL_DIR}/references/scenarios.md`.

Group criteria by subtree: grouping nodes become section headers, leaves have `## Acceptance Criteria` extracted. Missing section → `⚠ No acceptance criteria defined`. Format:

```
Phase N: <phase subject>
  Task <id>: <leaf subject>
  - [ ] criterion 1
```

Flat epics (no grandchildren) produce flat `Task <id>: ...` output.

**Task mode:** Extract `## Acceptance Criteria` from the target task's description. Missing → use full description as criteria context.

## Step 4: Get Diff

`git diff HEAD` + `git diff --cached` in parallel. Both empty → `git diff HEAD~1..HEAD`. Still empty → verify against HEAD state.

## Step 5: Spawn Verifier, Breaker, and Test Auditor

Three parallel `Task(subagent_type="general-purpose")` agents:

**Verifier** evaluates each criterion against the diff: PASS/FAIL/PARTIAL/N/A with line-level evidence. All criteria N/A → PASS with note "no applicable criteria found." Adds "Plan Deviations" section noting justified vs problematic divergences. Ends with one-line verdict.

**Breaker** assumes the implementation is subtly wrong. Hunts five angles: implied requirements, edge cases, integration points, technically-met-but-incomplete criteria, missing negatives. Each finding rated HIGH/MEDIUM/LOW with related criterion. Must show per-angle analysis with at least one concrete observation — "no findings" without this evidence is structurally invalid.

**Breaker validation (Step 6 pre-check):** Bare "no findings" without per-angle analysis → re-run breaker with explicit instruction to analyze all 5 angles. A lazy breaker that rubber-stamps PASS defeats the purpose of dual-agent verification.

**Test Auditor** evaluates test sufficiency against the criteria. Prompt from `${CLAUDE_SKILL_DIR}/references/test-auditor-prompt.md`. Per-criterion: maps specific test names to each criterion, identifies gaps (missing coverage, missing edge cases, missing error paths, missing integration, tautological tests), classifies each gap as CLEAR (can write without ambiguity) or AMBIGUOUS (needs domain knowledge). Cross-references Breaker HIGH/MEDIUM findings against test coverage. Verdict: SUFFICIENT or GAPS_FOUND.

**Auditor validation (Step 6 pre-check):** "SUFFICIENT" without per-criterion analysis → re-run auditor. Same rubber-stamp guard as Breaker — every criterion must have explicit test mapping.

## Step 6: Reconcile and Present

After all three agents return, run Step 6b (if applicable) before reconciliation. The auditor result fed into reconciliation is the post-loop result.

Reconciliation rules (evaluated in order; first match wins):

1. Verifier FAIL → **FAIL**
2. Verifier PARTIAL → **PARTIAL**
3. Verifier PASS + breaker HIGH findings on PASS/N/A criteria → **PARTIAL** (breaker caught what verifier missed)
4. Verifier PASS + no breaker HIGH + auditor **GAPS_FOUND** → **PARTIAL**
5. Verifier PASS + no breaker HIGH + auditor SUFFICIENT → **PASS**

Present all reports with clear labels. **PASS** → concise green summary (max 5 lines, no per-criterion breakdown), done.
PASS path includes test audit summary: "Test audit: SUFFICIENT — N criteria fully covered." If the loop ran before convergence, instead: "Test audit: SUFFICIENT after K iteration(s) — N criteria covered, M AMBIGUOUS gaps flagged for review."

**PARTIAL/FAIL** + `--auto` → automatically **Fix gaps** (option 1). After 2 fix iterations with no improvement, **Commit anyway (override)** (option 2).

**PARTIAL/FAIL** without `--auto` → AskUserQuestion with exactly 4 options:

1. **Fix gaps** → spawn fix agent with FAIL/PARTIAL criteria as scope, re-run from Step 4
2. **Commit anyway (override)** → note override in findings, proceed to Step 7
3. **File as follow-up tasks** → TaskCreate per gap (`metadata: {type: "bug", priority: "P2", parent_id: epicId}`), proceed to Step 7
4. **Re-run after manual fixes** → wait for user signal, re-run from Step 4

## Step 6b: Test Sufficiency Loop

Runs only when Test Auditor returned GAPS_FOUND. Executes before the PARTIAL/FAIL user gate (regardless of `--auto`).

Max 3 iterations. Each iteration:

1. **Spawn Test Writer** — fresh `Task(subagent_type="general-purpose")` with `Write` access. Receives: Gap Inventory (CLEAR gaps only), current diff, criteria list. Reads 2-3 nearby test files first to match style. Writes tests for CLEAR gaps. AMBIGUOUS gaps → inline comment block describing what's needed. Never guesses domain logic.
2. **Re-capture diff** — `git diff HEAD` + `git diff --cached` to include new tests.
3. **Spawn fresh Test Auditor** — new agent with updated diff, same criteria. No memory of prior iterations. Subject to the same structural validity guard as the initial auditor (per-criterion mapping required).
4. **Convergence check:**
   - SUFFICIENT → exit loop. If rule 4 was the sole cause of PARTIAL, downgrade to PASS.
   - Only AMBIGUOUS gaps remain → exit loop (all CLEAR gaps resolved)
   - CLEAR gaps remain + iteration < 3 → repeat
   - Iteration 3 with CLEAR gaps → escalate: flag unresolved gaps in report, stop looping

After loop: AMBIGUOUS gaps surface as advisory "Flagged for review" in the report, not blockers. Unresolved CLEAR gaps after 3 iterations → warning in report.

The loop runs automatically regardless of `--auto` (it is pre-gate). After the loop completes, `--auto` applies existing PARTIAL/FAIL logic. AMBIGUOUS test gaps are advisory notes, not blockers for `--auto`.

## Step 7: Store Findings

`ct plan create --prefix "acceptance" --topic "<target subject>" --project "$(git rev-parse --show-toplevel)"`. TaskUpdate target (epic or task) with `acceptance_result: {verdict, criteria_count, verifier_report, breaker_report, auditor_report}` and `plan_file: "$PLAN_FILE"` (omit if empty).
