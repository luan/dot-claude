---
name: fix
description: "Convert user feedback or review findings into phased tasks. Does NOT implement fixes. Triggers: 'fix', 'create issues from feedback', 'file bugs from feedback'."
argument-hint: "<feedback-text>"
user-invocable: true
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - TaskCreate
  - TaskUpdate
  - Write
  - AskUserQuestion
---

# Fix

Convert user feedback into ONE task with phased design in description — directly consumable by `/prepare`.
Does NOT implement — creates actionable work items for later scheduling.

## Workflow

### 1. Gather Context (Parallel)

Run these in parallel to understand what was recently implemented:
```bash
git diff --name-only HEAD~3..HEAD
git log --oneline -5
git branch --show-current
```

If user references specific files, read those files.

### 2. Analyze Feedback

Break feedback ($ARGUMENTS) into individual findings:
- Classify each: `bug`, `chore`, or `feature`
- Set priority (P1-P3):
  - P1: Critical bugs, blocking issues, high-priority features
  - P2: Normal priority (default for most feedback)
  - P3: Nice-to-have improvements, low-priority items
- Group findings by type for phase structure

### 3. Create Single Issue with Phased Design

Create ONE task containing all findings using TaskCreate:

```
TaskCreate:
  subject: "Fix: <brief-summary-of-feedback>"
  description: |
    ## Acceptance Criteria
    - All feedback items addressed
    - Findings stored as phased structure in description
    - Consumable by /prepare for epic creation

    ## Feedback Analysis

    **Phase 1: Bug Fixes**
    1. Fix X in file.ts:123 — description of bug
    2. Fix Y in module.ts:45 — description of bug

    **Phase 2: Improvements**
    3. Update Z configuration — description of improvement
    4. Add W feature — description of feature

    Each phase groups findings by type (bugs first, then tasks,
    then features). Skip empty phases.
  activeForm: "Creating fix task"
  metadata:
    project: <repo root from git rev-parse --show-toplevel>
    type: "fix"
    priority: "P2"
```

Mark active: `TaskUpdate(taskId, status: "in_progress", owner: "fix")`

**Store findings** — pass the full `description` field from the TaskCreate above (including Acceptance Criteria and all Phase sections) as the findings content:

1. `PLAN_FILE=$(echo "<findings>" | ck plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "fix" 2>/dev/null)` — if command fails or `$PLAN_FILE` is empty, warn user: "Plan file creation failed — findings stored in task metadata only."
2. `TaskUpdate(taskId, metadata: {design: "<findings>", plan_file: "$PLAN_FILE" (omit key if empty), status_detail: "review"}, description: "Fix: <topic> — findings in plan file and metadata.design")`

**Phase grouping rules:**
- Phase 1: Bugs (highest priority first)
- Phase 2: Tasks / improvements
- Phase 3: Features / new functionality
- Skip phases with no findings
- Each item: actionable title with file:line when available

### 4. Report

Output format:
```
## Fix: t<id>

**Findings**: N items (X bugs, Y tasks, Z features)

**Next**: Use TaskUpdate to modify findings if needed, `/prepare t<id>` to create epic with tasks.
```

## Error Handling
- TaskCreate fails → show error, retry once, then report
- Ambiguous feedback → AskUserQuestion for clarification before creating issues
