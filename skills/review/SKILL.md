---
name: review
description: "Adversarial code review with parallel reviewers. Triggers: 'review', 'review my changes', 'check this code', 'code review'. Use --team for 3-perspective mode."
argument-hint: "[base..head | file-list | PR#] [--against <issue-id>] [--team] [--continue]"
user-invocable: true
allowed-tools:
  - Task
  - Skill
  - AskUserQuestion
  - Read
  - Bash
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
  - Write
---

# Adversarial Review

Three modes: solo (default), file-split (auto for large diffs), perspective (--team). All consolidate findings into phase-structured output.

## Interviewing

See rules/skill-interviewing.md. Skill-specific triggers:
- Severity judgment borderline (medium vs high) → ask user's priority
- Pattern violation unclear (style preference vs correctness issue) → clarify importance

## Step 1: Scope + Mode

Parse $ARGUMENTS:

- `--against <task-id>`: task for plan adherence
- `--team`: force 3-perspective mode
- Remaining args:

| Input        | Diff source                             |
| ------------ | --------------------------------------- |
| (none)       | `git diff HEAD`                         |
| `main..HEAD` | `git diff main..HEAD`                   |
| file list    | `git diff HEAD -- <files>` + read files |
| `#123`       | `gh pr diff 123`                        |

Count files: `git diff --stat`

Choose mode:

- `--team` → **Perspective Mode** (3 specialists)
- 15+ files, no `--team` → **File-Split Mode**
- Otherwise → **Solo Mode** (2 lenses)

Size check: from the `git diff --stat` output already gathered, count changed files and total lines (+/-).
Set CODEX_TRIGGERED=true if: file_count >= 5 OR total_lines >= 200.

## Step 1b: Create Review Issue

```
TaskCreate:
  subject: "Review: <scope-summary>"
  description: "Adversarial review of <scope-details>"
  activeForm: "Creating review task"
  metadata:
    project: <repo root from git rev-parse --show-toplevel>
    type: "review"
    priority: "P2"

TaskUpdate(taskId, status: "in_progress", owner: "review")
```

If `--continue`: skip creation, find existing:

- $ARGUMENTS matches task ID → use it
- Else: `TaskList()` filtered by `metadata.type === "review"` and `status === "in_progress"`, use first result
- `TaskGet(taskId)` → extract `metadata.design`
- Prepend to reviewers: "Previous findings:\n<metadata.design>\n\nContinue reviewing..."

## Step 2: Gather Context

1. Resolve base ref — already expanded at load: !`gt parent 2>/dev/null || gt trunk 2>/dev/null || git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's|refs/remotes/||'`
2. Run in parallel:
   <!-- Lightweight commands instead of gitcontext to avoid pulling the full diff onto the main thread. -->
   - `git diff --stat $BASE...HEAD` → file list with change summary
   - `git diff --name-only $BASE...HEAD` → clean file list for mode selection and splitting
   - `git log --oneline $BASE..HEAD` → commit summary
   - `ck tool cochanges --base $BASE` → cochange candidates; store as COCHANGES (empty output or command failure → skip completeness lens silently)
3. If `--against`: `TaskGet(issueId)` for plan
4. Pass `BASE` ref to reviewer subagents — they fetch their own diff.

## Step 3: Dispatch Reviewers

When constructing reviewer prompts from references/prompts.md, replace `{base_ref}` with the resolved BASE value and `{files}` with the file list for the group.

### Solo Mode (2 or 3 lenses)

Spawn 2 Task agents (persistent-reviewer) in SINGLE message. Pass BASE ref. Reviewer gathers its own diff.

If COCHANGES is non-empty, spawn a third Task agent (persistent-reviewer) in the same message using the Completeness Lens prompt, passing `{changed_files}` = the file list and `{cochange_candidates}` = the cochange output.

Read references/prompts.md for Solo Mode and Completeness Lens prompt templates.

### File-Split Mode (>15 files)

Split files into groups of ~8. Spawn parallel Task agents, one per group. Pass BASE ref and file group.

Read references/prompts.md for File-Split Mode prompt template.

### Perspective Mode (--team)

Spawn EXACTLY 3 Task agents in SINGLE message (4 if COCHANGES non-empty). Pass BASE ref. Each perspective gathers its own diff (no splitting).

If COCHANGES is non-empty, spawn a 4th Task agent (persistent-reviewer) in the same message using the Completeness Lens prompt, passing `{changed_files}` = the file list and `{cochange_candidates}` = the cochange output.

Read references/prompts.md for Perspective Mode and Completeness Lens prompt templates.

If `--against`: append "Check plan adherence: implementation match plan? Missing/unplanned features? Deviations justified? Plan: {issue description}"

If CODEX_TRIGGERED=true, run the Codex invocation from references/prompts.md via Bash. This runs synchronously on the main thread — wait for output before proceeding to Step 4. If `codex` command fails or is not found, log a warning and skip; do not abort the review.

## Step 4: Consolidate + Present

0. **Validate reviewer output** (subagent-trust.md): spot-check 1-2 specific file:line claims from each reviewer before consolidating. If a claimed issue doesn't exist at that location → discard it.
   For [external] findings from codex: spot-check ALL file:line claims before including. If a codex finding duplicates a human reviewer finding, keep the human version. [external] tag persists through to output.
1. Deduplicate (same issue from multiple lenses → highest severity)
2. Sort by severity. **NEVER truncate validated findings.** Output EVERY finding that survived validation.
3. --team only: tag [architect]/[code-quality]/[devil], detect consensus (2+ flag same issue), note disagreements
4. Completeness findings are always DEFER regardless of what the reviewer classifies them as. They appear in the DEFER section of the output table.

Output: `# Adversarial Review Summary`

- **FIX items** (sorted by severity): table with Severity | File:Line | Issue | Suggestion
- **IGNORE items** (grouped by category, one line each): collapsed summary
- **DEFER items** (listed last — most visible to user): table with Severity | File:Line | Issue | Suggestion
- --team adds: Consensus (top, above FIX items), Disagreements (bottom, after DEFER items)
- Footer: Verdict (APPROVE/COMMENTS/CHANGES), Blocking count, Review task-id, "Clean review → /commit", "New work discovered → /prepare <task-id>"

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Return findings to caller. Don't fix." || echo "Use AskUserQuestion: Fix all FIX items / Fix critical+high FIX items only / Fix critical FIX items only / Skip fixes"`

## Step 4b: Store Findings

Store findings using `reviewId` as the task:

1. `PLAN_FILE=$(echo "<findings>" | ck plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "review" 2>/dev/null)` — if command fails or `$PLAN_FILE` is empty, warn user: "Plan file creation failed — findings stored in task metadata only."
2. `TaskUpdate(taskId, metadata: {design: "<findings>", plan_file: "$PLAN_FILE" (omit key if empty), status_detail: "review"}, description: "Review: <topic> — findings in plan file and metadata.design")`

## Step 5: Dispatch Fixes

Spawn general-purpose agent (model="sonnet"). Read references/prompts.md for fix dispatch prompt template.

After fix agent returns, invoke `Skill("refine")` on changed files.

## Step 6: Re-review

Re-run Step 3 after fixes. Track iteration count starting at 1 (max 4 re-review iterations; the initial review doesn't count).

Before re-running:
- Maintain `fixed_issues` set: `(file, issue-description)` pairs from previous iterations (not file:line — line numbers shift after fixes)
- When consolidating new findings in Step 4: skip any finding matching a `fixed_issues` entry

On each iteration: announce "Re-review iteration N/4"

If a DEFER item from a previous round no longer appears:
- Do NOT assume resolved — investigate first
- Line deleted entirely → resolved, remove from DEFER list
- Code changed but concern persists → keep as DEFER

Loop exits when:
- All FIX items resolved (no new FIX findings survive consolidation)
- OR user selects "Stop fixing"
- OR iteration count reaches 4 (report remaining FIX items as unresolved)

## Step 6a: Review Summary

Output structured summary before closing:

### Fixes Applied (N)
- [file:line] Description of fix

### Ignored Issues (N)
- [Severity] Description (grouped by type)

### Deferred Issues (N)
- [Severity] [file:line] Description → task-ID

For each DEFER finding, create a task:
```
TaskCreate:
  subject: "[DEFER] <one-line issue description>"
  description: "From review <reviewId>.\n\nFile: <file:line>\nSeverity: <severity>\n\n<full issue description + suggestion>"
  activeForm: "Creating deferred issue task"
  metadata:
    project: <repoRoot>
    type: "deferred-review"
    source_review: "<reviewId>"
    priority: "<P2 for high/critical, P3 for medium/low>"
```

Append created task IDs to the Deferred Issues list. Store summary in `metadata.design` via TaskUpdate.

## Step 6b: Close Review Issue

After review complete (user approves or skips fixes):
```
TaskUpdate(reviewId, status: "completed")
```

Review issues can be directly completed since the user is present. Do NOT auto-approve implementation work — user must explicitly request approval.

## Step 7: Interactive Continuation

Note: Fix selection happens in Step 4 above. This step handles pipeline continuation after review completes.

Check for implementation issues in review: `TaskList()` filtered by tasks with `metadata.status_detail === "review"` If any exist, note them in the prompt so the user knows approval is pending.

Present next step based on review outcome — use AskUserQuestion only when there's a genuine choice:

- **Clean review** → "Approve + commit" or "Refine before commit" or "Test plan"
- **Issues found and fixed** → "Re-review to verify?" or "Approve + commit" or "Refine before commit"
- **Issues found but not all fixed** → "Continue fixing?" or "Approve as-is" or "Refine before commit"

Skill dispatch:
- Approve + commit → `TaskList()` filtered by `metadata.project === repoRoot` and `status_detail === "review"` → `TaskUpdate(id, status: "completed", metadata: {status_detail: null})` for each, then `Skill("commit")`
- Re-review → `Skill("review")`
- Continue fixing → Resume fix loop at Step 5
- Refine → `Skill("refine")`
- Test plan → `Skill("test-plan")`

## Receiving Feedback

- **Verify** claims against code before agreeing/disagreeing
- Respond with evidence, not performative agreement
- Push back when: breaks functionality, violates YAGNI, incorrect
- No "done" claims without fresh evidence
