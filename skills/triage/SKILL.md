---
name: triage
description: "Convert user feedback or review findings into phased tasks. Does NOT implement fixes. Triggers: 'triage', 'create issues from feedback', 'file bugs from feedback'. Do NOT use when: the cause is unknown and investigation is needed — use /debugging instead. Do NOT use when: you want to implement the fix immediately — this skill only creates tasks."
argument-hint: "<feedback-text> [--auto]"
user-invocable: true
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - TaskCreate
  - TaskUpdate
  - Write
---

# Triage

Convert feedback into ONE task with phased design, consumable by `/scope`. Does not implement anything.

## Context

Recent files: !`git diff --name-only HEAD~3..HEAD 2>/dev/null`
Log: !`git log --oneline -5 2>/dev/null`
Branch: !`git branch --show-current 2>/dev/null`

## Workflow

### 1. Gather Context

Ground feedback against injected context above. If user references specific files, read them.

### 2. Analyze Feedback

Break feedback ($ARGUMENTS) into findings:

- Classify: `bug`, `chore`, or `feature`
- Priority: **P1** (blocking, data loss, security), **P2** (default), **P3** (polish, cosmetic)
- Group by type for phasing

### 3. Create Single Task with Phased Design

TaskCreate: subject "Triage: <brief-summary>", acceptance criteria (all feedback addressed, phased, consumable by /scope), metadata `{project: <repo root>, type: "triage", priority: "P2"}`.

Phases — bugs first because they block testing improvements:

- **Phase 1**: Bugs (highest priority first)
- **Phase 2**: Tasks / improvements
- **Phase 3**: Features / new functionality
- Skip empty phases. Each item: actionable title with file:line when available.

Mark active: `TaskUpdate(taskId, status: "in_progress", owner: "triage")`

**Store findings** in metadata.design:

1. `PLAN_FILE=$(echo "<findings>" | ct plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "triage" 2>/dev/null)`
2. `TaskUpdate(taskId, metadata: {design: "<findings>", plan_file: "$PLAN_FILE" (omit if empty), status_detail: "review"}, description: "Triage: <topic> — findings in metadata.design")`

### 4. Report

```
## Triage: t<id>

**Findings**: N items (X bugs, Y tasks, Z features)

**Next**: TaskUpdate to modify findings if needed, `/scope t<id>` to create epic.
```

## Error Handling

- TaskCreate fails → retry once, then report error
- Ambiguous feedback → `--auto`: best-guess classification. Without `--auto` → AskUserQuestion first — wrong classification wastes downstream scope/develop effort
