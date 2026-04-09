---
name: brainstorm
description: "Collaborative design for greenfield features and new ideas. Triggers: 'brainstorm', 'ideate', 'new feature design', 'help me think through', 'what should we build', 'help me design', 'think through X with me', 'I want to build something new'."
argument-hint: "<idea or topic> [--auto]"
user-invocable: true
allowed-tools:
  - Agent
  - Bash
  - Read
  - Glob
  - Grep
---

# Brainstorm

Turn vague ideas into actionable designs through collaborative dialogue.

**Main thread only.** Interactive dialogue stays here; context scanning uses a background subagent.

## Hard Gate

Present and get approval on a design before any implementation. "Simple" projects are where unexamined assumptions waste the most work.

## Instructions

### 1. Scan Project Context + Start Interview

Dispatch Agent (subagent_type="Explore", run_in_background=true): scan for tech stack, relevant patterns, adjacent code, constraints. Under 30 lines. Empty/new project → skip scan, ask stack preferences in interview.

Begin the interview immediately — scan results feed Step 3.

### 2. Interview

`--auto` → skip interview. Infer purpose, scope, constraints, and success criteria from prompt + project context.

Without `--auto`: AskUserQuestion, ONE per turn. Prefer multiple-choice.

**Skip interview only if** the prompt has ALL three: explicit scope boundaries (non-goals stated), measurable constraints, and testable success criteria. Acknowledge by citing 2+ concrete details. When in doubt, interview.

**Sequence** (adapt, skip irrelevant):

1. **Purpose** — What problem? Who's it for?
2. **Scope** — Minimum useful version? (YAGNI gate)
3. **Constraints** — Performance, compatibility, security, timeline?
4. **Prior art** — Similar code in codebase or elsewhere?
5. **Success criteria** — How will you know it works?

Stop when you can propose approaches. Usually 3-5 questions, never >7.

**Mid-dialogue pivot:** If direction shifts fundamentally, acknowledge, discard stale context, restart from the relevant question.

### 3. Propose 2-3 Approaches

Check background scan completed. Incorporate findings into approaches.

Lead with recommendation + justification referencing user's constraints. Non-recommended: 2-3 sentences + downside vs recommended. Be opinionated. `--auto` → auto-select the recommended approach. Without `--auto` → ask user to pick or refine. All rejected → ask what's missing, propose new approaches.

### 4. Present Design

Scale to complexity. `--auto` → skip per-section confirmations. Without `--auto` → confirm after each section.

Include only relevant sections: architecture, data flow, API surface, error handling, testing.

### 5. Summary + Next

```
Brainstorm: <topic>
Problem: <1 sentence>
Approach: <1 sentence>
Next: /spec
```

The design lives in this conversation. When ready to formalize, run `/spec` — the spec is the durable artifact.

## Key Principles

- YAGNI: push back on scope creep during interview
- Design is the deliverable — implementation details belong in the spec
