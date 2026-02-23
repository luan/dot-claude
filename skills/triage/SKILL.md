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

Convert user feedback into ONE task with phased design — directly consumable by `/prepare`. Does NOT implement; creates actionable work items for later scheduling.

## Workflow

### 1. Gather Context (Parallel)

Ground feedback against recent changes so classification reflects actual code state:
```bash
git diff --name-only HEAD~3..HEAD
git log --oneline -5
git branch --show-current
```

If user references specific files, read them.

### 2. Analyze Feedback

Break feedback ($ARGUMENTS) into individual findings:
- Classify each: `bug`, `chore`, or `feature`
- Set priority:
  - **P1**: Blocking bugs, data loss, security — needs immediate attention
  - **P2**: Default. Most feedback lands here
  - **P3**: Polish, nice-to-have, cosmetic — won't block shipping
- Group by type for phase structure

### 3. Create Single Issue with Phased Design

TaskCreate: subject "Triage: <brief-summary>", acceptance criteria (all feedback addressed, phased structure, consumable by /prepare), metadata `{project: <repo root>, type: "triage", priority: "P2"}`.

Description includes phased findings — bugs first because they block testing improvements:
- **Phase 1**: Bugs (highest priority first within phase)
- **Phase 2**: Tasks / improvements
- **Phase 3**: Features / new functionality
- Skip empty phases. Each item: actionable title with file:line when available.

Mark active: `TaskUpdate(taskId, status: "in_progress", owner: "triage")`

**Store findings** — dual storage because plan file provides durable searchable backup while metadata enables quick API access by downstream skills:

1. `PLAN_FILE=$(echo "<findings>" | ck plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "triage" 2>/dev/null)` — warn if fails/empty.
2. `TaskUpdate(taskId, metadata: {design: "<findings>", plan_file: "$PLAN_FILE" (omit if empty), status_detail: "review"}, description: "Triage: <topic> — findings in plan file and metadata.design")`

### 4. Report

```
## Triage: t<id>

**Findings**: N items (X bugs, Y tasks, Z features)

**Next**: TaskUpdate to modify findings if needed, `/prepare t<id>` to create epic.
```

## Error Handling
- TaskCreate fails → show error, retry once, then report
- Ambiguous feedback → AskUserQuestion before creating — wrong classification wastes downstream prepare/implement effort
