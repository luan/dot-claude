---
name: explore
description: "Research, investigate, and design via subagent dispatch with auto-escalation for complex work. Triggers: 'explore', 'how does X work', 'understand', 'research', 'plan a feature', 'figure out', 'investigate', 'design', 'architect', 'best way to', 'state of the art', 'which lib/tool'. Also use when an implementation request contains an unresolved technology choice."
argument-hint: "<prompt> [--continue]"
user-invocable: true
allowed-tools:
  - Task
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
  - Skill
  - AskUserQuestion
  - Bash
  - Write
---

# Explore

Research, investigate, design. Findings stored via plan-storage.
Auto-escalates to team for complex multi-system work.

**IMMEDIATELY dispatch to subagent.** Never explore on main thread.

## Context

Active parent issues: Use TaskList() to view current tasks.

## Mid-Skill Interviewing

Use AskUserQuestion when facing genuine ambiguity during execution:
- Multiple viable approaches with unclear tradeoffs → ask which direction to pursue
- Domain ambiguity (business logic interpretation) → clarify with user before deep-diving

Do NOT ask when the answer is obvious or covered by the task brief.

## Instructions

### New Exploration

1. Create exploration task:
   ```
   TaskCreate:
     subject: "Explore: <topic>"
     description: "## Acceptance Criteria\n- Findings stored in task description\n- Structured as Current State, Recommendation, and phased Next Steps\n- Each phase includes file paths and is independently actionable"
     activeForm: "Creating exploration task"
     metadata:
       project: <repo root from git rev-parse --show-toplevel>
       label: "explore"
       priority: 2
   ```

2. Start task: `TaskUpdate(taskId, status: "in_progress")`

3. Dispatch via Task (subagent_type="codebase-researcher"):

```
Research <topic> thoroughly. Return COMPLETE findings as text
(do NOT write files, do NOT create tasks).

## Job
1. Explore codebase
2. Design approach — 2-3 options, choose best

## Output Structure

1. **Current State**: What exists now (files, patterns, architecture)
2. **Recommendation**: Chosen approach with rationale
3. **Key Files**: Exact paths of files to modify/create
4. **Risks**: What could go wrong, edge cases
5. **Next Steps**: Phased implementation plan using format:

**Phase N: <title>**
Files: exact/path/to/file.ts, exact/path/to/other.ts
Approach: <what to change and why>
1. <specific step>
2. <specific step>

Each phase must include file paths and approach hints —
downstream task creation depends on this detail.

## Escalation
If the problem spans 3+ independent subsystems, has 3+ viable
approaches with unclear tradeoffs, or needs adversarial analysis
of cross-cutting concerns, return: "ESCALATE: team — <reason>"
```

4. **Validate findings** (subagent-trust.md): spot-check ALL
   architectural claims + 50% of file/behavioral claims before storing.
   If echo suspected or key claims fail → send targeted follow-up.

5. **Store findings:**
   1. `echo "<findings>" | ck plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "explore"`
   2. `TaskUpdate(taskId, metadata: {design: "<findings>", plan_file: "<filename from stdout>", status_detail: "review"}, description: "Explore: <topic> — findings in plan file and metadata.design")`

6. Output summary:
```
Explore: t<id> — <topic>
Problem: <1 sentence>
Recommendation: <1 sentence>

Phases:
1. <title> — <key files>
2. <title> — <key files>

Key decisions:
- <why this approach>

Next: /prepare t<id>
```

7. → See Continuation Prompt below.

### Continuation (--continue flag)

1. Resolve task ID:
   - If $ARGUMENTS matches a task ID → use it
   - If --continue → `TaskList()` filtered by `metadata.label === "explore"` and either `metadata.status_detail === "review"` or `status === "in_progress"`, use first result
2. Load existing: `TaskGet(taskId)` → extract `metadata.design`
3. Move back to active: `TaskUpdate(taskId, status: "in_progress", metadata: {status_detail: null})`
4. Dispatch subagent with: "Previous findings:\n<metadata.design>\n\nContinue exploring: <new prompt>"
5. Update plan file and task, updating existing file at `<metadata.plan_file>`:
   1. `echo "<findings>" | ck plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "explore"`
   2. `TaskUpdate(taskId, metadata: {design: "<findings>", plan_file: "<filename from stdout>", status_detail: "review"}, description: "Explore: <topic> — findings in plan file and metadata.design")`
6. Output updated summary → See Continuation Prompt below.

### Continuation Prompt

Use AskUserQuestion:
- "Continue to /prepare t<id>" (Recommended) — description: "Create epic + implementation tasks from findings"
- "Re-explore with different focus" — description: "Investigate a different angle on the same topic"
- "Done for now" — description: "Leave task active for later /next"

If user selects "Continue to /prepare":
→ Invoke Skill tool: skill="prepare", args="<task-id>"

If user selects "Re-explore":
→ Ask what to focus on, then re-run from step 3 with updated prompt

### Escalation

Subagent returns "ESCALATE: team — [reason]":

Spawn 2-3 Task agents in parallel:
- **Researcher** (model: opus): breadth-first investigation
- **Architect** (model: opus): design analysis, tradeoffs
- **Devil's Advocate** (model: opus): challenges assumptions

Each investigates independently, returns findings text.
Synthesize into unified findings, store in description.

Escalation triggers:
- 3+ viable approaches, unclear tradeoffs
- Spans 3+ independent subsystems
- Cross-cutting concerns needing adversarial analysis

## Key Rules

- Main thread does NOT explore — subagent does
- Findings stored via plan-storage (plan file + task metadata)
- Next Steps must include file paths (prepare depends on it)
