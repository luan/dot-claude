---
name: acceptance
description: "Validate implementation against epic acceptance criteria and plan. Triggers: 'accept', 'acceptance check', 'verify implementation'."
argument-hint: "[<task-id>]"
user-invocable: true
allowed-tools:
  - Task
  - TaskGet
  - TaskList
  - TaskUpdate
  - Bash
  - AskUserQuestion
  - Read
  - Glob
  - Grep
---

# Acceptance

Validates implementation against epic acceptance criteria and plan adherence. Advisory gate — findings are reported, user decides how to proceed.

## Step 1: Resolve Epic

- Argument provided → `TaskGet(taskId)`. Verify `metadata.type == "epic"`. If not an epic, check `metadata.parent_id` → use that as epicId.
- No argument → `TaskList()` filtered by `metadata.type == "epic"` AND `status` in `["completed", "in_progress"]`. Pick the highest ID.
- No epic found → stop: "No epic found. Pass a task ID or ensure an epic exists."

## Step 2: Gather Plan Context

From `TaskGet(epicId)`:

- Extract `metadata.design` (plan/design brief stored during prepare)
- Extract `metadata.plan_file`

If `metadata.plan_file` is set:

```bash
ck plan latest --task-file <plan_file>
```

Capture full plan content. If command fails or plan_file is absent, use `metadata.design` as the plan. If neither exists, note "No plan found — checking criteria only."

## Step 3: Gather Child Task Criteria

`TaskList()` filtered by `metadata.parent_id == epicId`.

For each child task, extract its `## Acceptance Criteria` section from the description. Build a structured list:

```
Task <id>: <subject>
- [ ] criterion 1
- [ ] criterion 2
```

No children → stop: "Epic has no child tasks. Run /prepare first."

## Step 4: Get Diff

Run in parallel:

```bash
git diff HEAD
git diff --cached
```

If both are empty (no staged or unstaged changes):

```bash
git diff HEAD~1..HEAD
```

Use whichever produces output. If still empty, note "No diff found — verifying against HEAD state."

## Step 5: Spawn Verifier and Breaker

Spawn two `Task(subagent_type="general-purpose")` in parallel:

**Verifier** — confirms criteria are met:

```
You are an acceptance validator. Your job: determine whether the implementation satisfies the acceptance criteria and adheres to the plan.

## Plan / Design
<plan content or "No plan available">

## Acceptance Criteria by Task
<structured list from Step 3>

## Diff
<git diff output>

For each acceptance criterion, evaluate it against the diff and output:

| Task | Criterion | Status | Evidence |
|------|-----------|--------|----------|

Status values:
- PASS: criterion is met — point to specific lines in the diff
- FAIL: criterion is not met — explain what is missing
- PARTIAL: partially met — explain what is missing
- N/A: criterion not verifiable from diff alone (no code change expected)

After the table, add a "Plan Deviations" section:
- List any implementation choices that diverge from the plan design
- For each deviation: Is it justified (simplification, discovered constraint) or problematic (missing feature, wrong approach)?

End with a one-line verdict: PASS (all criteria met), PARTIAL (some gaps), or FAIL (critical criteria unmet).
```

**Breaker** — adversarial agent that assumes the implementation is subtly wrong:

```
You are an adversarial breaker. Assume the implementation is subtly wrong. Your job: hunt for gaps the verifier would miss.

## Plan / Design
<plan content or "No plan available">

## Acceptance Criteria by Task
<structured list from Step 3>

## Diff
<git diff output>

Investigate each of these angles:

1. **Implied requirements** — what does the plan/criteria assume but never state? Check if the implementation handles those assumptions.
2. **Edge cases** — empty inputs, boundary values, concurrent access, error paths. Are they covered or silently ignored?
3. **Integration points** — does the diff touch interfaces consumed by other code? Could callers break with these changes?
4. **Technically-met-but-functionally-incomplete** — does the implementation satisfy the letter of a criterion while missing its intent?
5. **Missing negatives** — things the implementation should NOT do (security, side effects, regressions) that aren't in the criteria.

For each finding, output:

| # | Category | Finding | Severity | Related Criterion |
|---|----------|---------|----------|--------------------|

Severity values:
- HIGH: likely bug or missing behavior that would surface in real use
- MEDIUM: gap that could cause issues under specific conditions
- LOW: minor concern, style, or unlikely edge case

If you find nothing substantive, say "No significant gaps found." Do not invent findings to justify your role.
```

## Step 6: Reconcile and Present

**Reconciliation logic** (applied before presenting to user):

1. If the verifier verdict is FAIL → overall verdict is **FAIL** (regardless of breaker).
2. If the verifier verdict is PARTIAL → overall verdict is **PARTIAL** (breaker findings add context but don't change the verdict).
3. If the verifier verdict is PASS and the breaker found HIGH-severity findings the verifier marked as PASS or N/A → overall verdict is **PARTIAL**.
4. If the verifier verdict is PASS and the breaker found no HIGH findings → overall verdict is **PASS**.

Present both outputs with clear labels ("Verifier Report" / "Breaker Report") followed by the reconciled verdict.

**If verdict is PASS:**

Print a green summary. Return — no further action needed unless user continues.

**If verdict is PARTIAL or FAIL:**

```
AskUserQuestion: "Acceptance check found gaps. How do you want to proceed?"
options:
  - "Fix gaps before committing"
  - "Commit anyway (override)"
  - "File gaps as follow-up tasks"
  - "Re-run after manual fixes"
```

Handle response:
- **Fix gaps before committing** → spawn a general-purpose fix agent with the FAIL/PARTIAL criteria as scope. After fix agent returns, re-run from Step 4.
- **Commit anyway** → note override in findings, proceed to Step 7.
- **File gaps as follow-up tasks** → for each FAIL/PARTIAL criterion, `TaskCreate(subject: "Gap: <criterion>", description: "From acceptance check on epic <epicId>.\n\nCriterion: <criterion>\n\nEvidence: <subagent evidence>", metadata: {type: "bug", priority: "P2", parent_id: epicId})`. Proceed to Step 7.
- **Re-run after manual fixes** → wait for user signal, then re-run from Step 4.

## Step 7: Store Findings

```bash
PLAN_FILE=$(echo "<findings>" | ck plan create --prefix "acceptance" 2>/dev/null)
```

If command fails or returns empty, warn: "Plan file creation failed — findings in task metadata only."

```
TaskUpdate(epicId, metadata: {
  acceptance_result: "<PASS|PARTIAL|FAIL>",
  acceptance_plan_file: "<PLAN_FILE or omit if empty>"
})
```
