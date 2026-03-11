---
model: opus
name: brainstorm
description: "Collaborative design for greenfield features and new ideas. Triggers: 'brainstorm', 'ideate', 'new feature design', 'help me think through', 'what should we build', 'help me design', 'think through X with me', 'I want to build something new'. Do NOT use when: the user wants to investigate an existing codebase or research a specific technical question — use /scope instead."
argument-hint: "<idea or topic> [--auto]"
user-invocable: true
allowed-tools:
  - Task
  - Skill
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

No implementation (skills, code, actions) until design is presented and user-approved. "Simple" projects are where unexamined assumptions waste the most work.

## Instructions

### 1. Create Work Task

TaskCreate: subject "Brainstorm: <topic>", metadata `{project: <repo root>, type: "brainstorm", priority: "P2"}`. Then `TaskUpdate(taskId, status: "in_progress", owner: "brainstorm")`.

### 2. Scan Project Context + Start Interview

Dispatch Task (subagent_type="Explore", run_in_background=true): scan for tech stack, relevant patterns, adjacent code, constraints. Under 30 lines. Empty/new project → skip scan, ask stack preferences in interview.

**Don't wait** — begin interview immediately. Scan results feed Step 4.

### 3. Interview

`--auto` → skip interview. Infer purpose, scope, constraints, and success criteria from prompt + project context.

Without `--auto`: AskUserQuestion, ONE per turn. Prefer multiple-choice.

**Skip interview only if** the prompt has ALL three: explicit scope boundaries (non-goals stated), measurable constraints, and testable success criteria. Acknowledge by citing 2+ concrete details. When in doubt, interview. **Exception:** even when all three are met, if the prompt describes a solution without a stated problem (see `references/problem-framing.md`), ask ONE problem-framing question before proceeding to approaches.

**Sequence** (adapt, skip irrelevant):

1. **Purpose** — What problem? Who's it for?
2. **Scope** — Minimum useful version? (YAGNI gate)
3. **Constraints** — Performance, compatibility, security, timeline?
4. **Prior art** — Similar code in codebase or elsewhere?
5. **Success criteria** — How will you know it works?

Stop when you can propose approaches. Usually 3-5 questions, never >7. Stay technology-agnostic — specific tech belongs in step 4.

**Challenge gate:** Triggers: (1) answer contradicts a previously stated constraint, (2) prompt is solution-shaped (describes a thing to build without stating a problem), (3) user states something as fact without evidence (unvalidated assumption). When any trigger fires, ask ONE reframing question before proceeding — do not block the interview, redirect it. See `references/problem-framing.md` for patterns and push-back phrasing. Do not silently incorporate contradictions or unvalidated claims. Never expose internal mechanics to the user — no "the challenge gate fires", no naming triggers or skill steps. Just ask the question naturally.

**Mid-dialogue pivot:** If direction shifts fundamentally, acknowledge, discard stale context, restart from the relevant question.

> **Reference:** `references/problem-framing.md` — challenge patterns for solution-shaped or assumption-laden inputs.

### 4. Propose 2-3 Approaches

Check background scan completed. Incorporate findings into approaches.

Lead with recommendation + justification referencing user's constraints. Non-recommended: 2-3 sentences + downside vs recommended. Be opinionated. `--auto` → auto-select the recommended approach. Without `--auto` → ask user to pick or refine. If the user rejects the recommendation, state what the non-recommended approach gives up compared to the recommended one — do not just acknowledge and pivot. All rejected → ask what's missing, propose new approaches.

### 5. Present Design Sections

Scale to complexity. `--auto` → skip per-section confirmations. Without `--auto` → confirm before proceeding after each section.

Include only relevant: architecture, data flow, API surface, error handling, testing.

### 6. Store Design

Once approved, store in metadata.design:

1. `PLAN_FILE=$(echo "<findings>" | ct plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "brainstorm" 2>/dev/null)`
2. `TaskUpdate(taskId, metadata: {design: "<findings>", plan_file: "$PLAN_FILE" (omit if empty), status_detail: "review"}, description: "Brainstorm: <topic> — findings in metadata.design")`

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
