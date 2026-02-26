---
name: triage
description: "Convert user feedback or review findings into phased tasks. Does NOT implement fixes. Triggers: 'triage', 'create issues from feedback', 'file bugs from feedback'. Do NOT use when: the cause is unknown and investigation is needed — use /debugging instead. Do NOT use when: you want to implement the fix immediately — this skill only creates tasks."
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

# Triage

Convert user feedback into ONE task with phased design — directly consumable by `/scope`. Does NOT implement; creates actionable work items for later scheduling.

## Context

Recent files: !`git diff --name-only HEAD~3..HEAD 2>/dev/null`
Log: !`git log --oneline -5 2>/dev/null`
Branch: !`git branch --show-current 2>/dev/null`

## Workflow

### 1. Gather Context

Use injected context above to ground feedback against recent changes. If user references specific files, read them.

### 2. Analyze Feedback

Break feedback ($ARGUMENTS) into individual findings:
- Classify each: `bug`, `chore`, or `feature`
- Set priority:
  - **P1**: Blocking bugs, data loss, security — needs immediate attention
  - **P2**: Default. Most feedback lands here
  - **P3**: Polish, nice-to-have, cosmetic — won't block shipping
- Group by type for phase structure

### 3. Create Single Issue with Phased Design

TaskCreate: subject "Triage: <brief-summary>", acceptance criteria (all feedback addressed, phased structure, consumable by /scope), metadata `{project: <repo root>, type: "triage", priority: "P2"}`.

Description includes phased findings — bugs first because they block testing improvements:
- **Phase 1**: Bugs (highest priority first within phase)
- **Phase 2**: Tasks / improvements
- **Phase 3**: Features / new functionality
- Skip empty phases. Each item: actionable title with file:line when available.

Mark active: `TaskUpdate(taskId, status: "in_progress", owner: "triage")`

**Store findings** in metadata.design:

`TaskUpdate(taskId, metadata: {design: "<findings>", status_detail: "review"}, description: "Triage: <topic> — findings in metadata.design")`

### 4. Report

```
## Triage: t<id>

**Findings**: N items (X bugs, Y tasks, Z features)

**Next**: TaskUpdate to modify findings if needed, `/scope t<id>` to create epic.
```

## Error Handling
- TaskCreate fails → show error, retry once, then report
- Ambiguous feedback → AskUserQuestion before creating — wrong classification wastes downstream scope/develop effort
