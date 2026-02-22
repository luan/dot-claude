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

## Step 3: Gather Descendant Criteria

Recursively collect all descendants of the epic:

```
descendants(id):
  direct = TaskList() filtered by metadata.parent_id == id
  return direct + flatten(descendants(child.id) for child in direct)

all_tasks = descendants(epicId)
```

No descendants → stop: "Epic has no descendant tasks. Run /prepare first."

**Orphaned task detection:** While traversing, flag any task where `status == "pending"` and its immediate parent task's `status == "completed"`. If any found, prepend a warning to the criteria output:

```
⚠ Orphaned tasks detected (parent completed, child still pending):
- Task <id>: <subject> (parent: Task <parent_id>)
```

This is advisory — the check continues regardless.

**Orphaned task detection — test scenarios:**

| Scenario | Setup | Expected warning | Behavior |
|----------|-------|------------------|----------|
| A: One orphan | Parent Task 10 `status=completed`; child Task 11 `status=pending`, `parent_id=10` | `⚠ Orphaned tasks detected (parent completed, child still pending):`<br>`- Task 11: <subject> (parent: Task 10)` | Warning prepended; acceptance continues normally |
| B: No orphan | Parent Task 10 `status=completed`; child Task 11 `status=completed`, `parent_id=10` | _(no warning)_ | Traversal continues silently |
| C: Nested orphan | Grandparent Task 10 `status=completed`; parent Task 11 `status=completed`; grandchild Task 12 `status=pending`, `parent_id=11` | `⚠ Orphaned tasks detected (parent completed, child still pending):`<br>`- Task 12: <subject> (parent: Task 11)` | Detected at grandchild level during recursive traversal; acceptance continues |

Scenario C confirms detection fires at any depth, not just direct children of the epic. The warning never blocks acceptance — it is informational only.

**Group criteria by subtree:**

For each task in `all_tasks`:
- If the task has children (grouping node) → emit as a section header only; skip its own criteria (grouping nodes have none)
- If the task is a leaf → extract its `## Acceptance Criteria` section and nest under the nearest ancestor that has children

Build a structured list:

```
Phase N: <phase subject>
  Task <id>: <leaf subject>
  - [ ] criterion 1
  - [ ] criterion 2
Phase M: <phase subject>
  Task <id>: <leaf subject>
  - [ ] criterion 1
```

Flat epics (all descendants are direct children with no grandchildren) produce output identical to the prior format:

```
Task <id>: <subject>
- [ ] criterion 1
- [ ] criterion 2
```

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
