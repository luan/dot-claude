---
name: brainstorm
description: "Collaborative design for greenfield features and new ideas. Triggers: 'brainstorm', 'ideate', 'new feature design', 'help me think through', 'what should we build'. Do NOT use when: the user wants to investigate an existing codebase or research a specific technical question — use /explore instead."
argument-hint: "<idea or topic>"
user-invocable: true
allowed-tools:
  - Task
  - Skill
  - AskUserQuestion
  - Bash
  - Read
  - Glob
  - Grep
  - Write
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
---

# Brainstorm

Turn vague ideas into actionable designs through collaborative dialogue. For greenfield work — use `/explore` for existing system research.

**Main thread only.** Interactive dialogue can't be delegated; context scanning uses a subagent.

## Hard Gate

Do NOT invoke implementation skills, write code, or take implementation action until design is presented and user-approved. "Simple" projects are where unexamined assumptions waste the most work.

## Instructions

### 1. Create Work Task

TaskCreate: subject "Brainstorm: <topic>", acceptance criteria (design stored as Problem/Approaches/Design/Next Steps, user approved each section), metadata `{project: <repo root>, type: "explore", priority: "P2"}`. Then `TaskUpdate(taskId, status: "in_progress", owner: "brainstorm")`.

### 2. Scan Project Context

Dispatch Task (subagent_type="codebase-researcher"): scan for tech stack, relevant patterns, adjacent code, constraints. Under 30 lines — feeds dialogue, not implementation.

### 3. Interview

AskUserQuestion, ONE per turn — wait for answer before next. Prefer multiple-choice.

**Upfront spec:** If the prompt already contains constraints, scope, and success criteria, skip to step 4 (Propose Approaches) with brief acknowledgment of what was provided.

**Sequence** (adapt, skip irrelevant):
1. **Purpose** — What problem? Who's it for?
2. **Scope** — Minimum useful version? (YAGNI gate)
3. **Constraints** — Performance, compatibility, security, timeline?
4. **Prior art** — Similar code in codebase or elsewhere?
5. **Success criteria** — How will you know it works?

Stop when you can propose approaches. Usually 3-5 questions, never >7.

**Mid-dialogue pivot:** If direction shifts fundamentally, acknowledge, discard stale context, restart from the relevant question.

### 4. Propose 2-3 Approaches

Lead with recommendation and why. Each approach: 2-3 sentences + key tradeoff. Be opinionated — don't hedge equally. Ask user to pick or refine. If all rejected, ask what's missing and propose new approaches — don't dead-end.

### 5. Present Design Sections

Scale to complexity. After each section: "Does this look right, or should we adjust?"

Include only relevant: Architecture, Data flow, API surface, Error handling, Testing strategy.

### 6. Store Design

Once approved, store via plan-storage:

1. `PLAN_FILE=$(echo "<findings>" | ck plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "brainstorm" 2>/dev/null)` — warn if fails/empty.
2. `TaskUpdate(taskId, metadata: {design: "<findings>", plan_file: "$PLAN_FILE" (omit if empty), status_detail: "review"}, description: "Brainstorm: <topic> — findings in plan file and metadata.design")`

Findings format: `## Problem` → `## Chosen Approach` (with rationale) → `## Design` (approved sections) → `## Next Steps` (Phase N: title, Files, Approach, steps).

### 7. Output Summary

```
Brainstorm: t<id> — <topic>
Problem: <1 sentence>
Approach: <1 sentence>
Phases:
1. <title> — <key files>
Next: /prepare t<id>
```

### 8. After Completion

Stop after summary. User reviews design before proceeding — do not auto-invoke prepare.

## Key Rules

- Next Steps must include file paths — prepare depends on them
- YAGNI: push back on scope creep during interview
