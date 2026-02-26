---
name: brainstorm
description: "Collaborative design for greenfield features and new ideas. Triggers: 'brainstorm', 'ideate', 'new feature design', 'help me think through', 'what should we build', 'help me design', 'think through X with me', 'I want to build something new'. Do NOT use when: the user wants to investigate an existing codebase or research a specific technical question — use /scope instead."
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

Turn vague ideas into actionable designs through collaborative dialogue. For greenfield work — use `/scope` for existing system research.

**Main thread only.** Interactive dialogue can't be delegated; context scanning uses a subagent.

## Hard Gate

Do NOT invoke implementation skills, write code, or take implementation action until design is presented and user-approved. "Simple" projects are where unexamined assumptions waste the most work.

## Instructions

### 1. Create Work Task

TaskCreate: subject "Brainstorm: <topic>", acceptance criteria (design stored as Problem/Approaches/Design/Next Steps, user approved each section), metadata `{project: <repo root>, type: "brainstorm", priority: "P2"}`. Then `TaskUpdate(taskId, status: "in_progress", owner: "brainstorm")`.

### 2. Scan Project Context + Start Interview

Dispatch Task (subagent_type="Explore", run_in_background=true): scan for tech stack, relevant patterns, adjacent code, constraints. Under 30 lines — feeds dialogue, not implementation. Empty/new project → skip scan, ask user for stack preferences in interview instead.

**Don't wait** — begin interview immediately. Scan results are only needed at Step 4.

### 3. Interview

AskUserQuestion, ONE per turn — wait for answer before next. Prefer multiple-choice.

**Upfront spec:** Only skip interview if the prompt has ALL three: explicit scope boundaries (non-goals stated), measurable constraints, and testable success criteria. Acknowledge by citing 2+ concrete details from the user's spec. When in doubt, interview.

**Sequence** (adapt, skip irrelevant):
1. **Purpose** — What problem? Who's it for?
2. **Scope** — Minimum useful version? (YAGNI gate)
3. **Constraints** — Performance, compatibility, security, timeline?
4. **Prior art** — Similar code in codebase or elsewhere?
5. **Success criteria** — How will you know it works?

Stop when you can propose approaches. Usually 3-5 questions, never >7. Stay technology-agnostic during interview — specific technologies belong in step 4.

**Mid-dialogue pivot:** If direction shifts fundamentally, acknowledge, discard stale context, restart from the relevant question.

### 4. Propose 2-3 Approaches

Check background scan completed. Incorporate findings into approaches.

Lead with recommendation + justification referencing user's constraints. Non-recommended: 2-3 sentences + downside vs recommended. Be opinionated. Ask user to pick or refine. All rejected → ask what's missing, propose new approaches.

### 5. Present Design Sections

Scale to complexity. After each section, confirm before proceeding.

Include only relevant: architecture, data flow, API surface, error handling, testing.

### 6. Store Design

Once approved, store in metadata.design:

`TaskUpdate(taskId, metadata: {design: "<findings>", status_detail: "review"}, description: "Brainstorm: <topic> — findings in metadata.design")`

Findings format: `## Problem` → `## Chosen Approach` (with rationale) → `## Design` (approved sections) → `## Next Steps` (Phase N: title, Files, Approach, steps).

### 7. Output Summary

```
Brainstorm: t<id> — <topic>
Problem: <1 sentence>
Approach: <1 sentence>
Phases:
1. <title> — <key files>
Next: /scope t<id>
```

### 8. Stop

User reviews design before proceeding — do not auto-invoke scope.

## Key Rules

- Next Steps must include file paths — scope depends on them
- YAGNI: push back on scope creep during interview
