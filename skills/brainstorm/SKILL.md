---
name: brainstorm
description: "Collaborative design for greenfield features and new ideas. Triggers: 'brainstorm', 'ideate', 'new feature design', 'help me think through', 'what should we build'."
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

Turn vague ideas into actionable designs through collaborative
dialogue. For greenfield work where there's no existing code to
investigate — use `/explore` instead when researching existing systems.

**This skill runs on the main thread.** Interactive dialogue can't be
delegated. Context scanning uses a subagent.

## Hard Gate

Do NOT invoke any implementation skill, write any code, or take any
implementation action until you have presented a design and the user
has approved it. This applies regardless of perceived simplicity.
"Simple" projects are where unexamined assumptions waste the most work.

## Instructions

### 1. Create Work Task

```
TaskCreate:
  subject: "Brainstorm: <topic>"
  description: "## Acceptance Criteria\n- Design stored in task description\n- Structured as Problem, Approaches, Chosen Design, Next Steps\n- User approved each design section before storing"
  activeForm: "Creating brainstorm task"
  metadata:
    project: <repo root from git rev-parse --show-toplevel>
    type: "explore"
    priority: "P2"

Then TaskUpdate(taskId, status: "in_progress", owner: "brainstorm")
```

### 2. Scan Project Context

Dispatch via Task (subagent_type="codebase-researcher", model=sonnet):

```
Quick context scan for brainstorming session about: <topic>

Return a concise summary (do NOT write files, do NOT create issues):

1. **Tech stack**: languages, frameworks, key dependencies
2. **Relevant patterns**: existing conventions that a new feature
   should follow (naming, file structure, error handling)
3. **Adjacent code**: modules/files closest to the topic area
4. **Constraints**: anything that would limit design choices

Keep it under 30 lines. This feeds a design dialogue, not
implementation.
```

### 3. Interview — One Question at a Time

Use AskUserQuestion. Ask ONE question per turn. Wait for answer
before asking next. Prefer multiple-choice options when possible.

**Question sequence** (adapt to topic, skip irrelevant ones):

1. **Purpose** — What problem does this solve? Who's it for?
2. **Scope** — What's the minimum that would be useful? (YAGNI gate)
3. **Constraints** — Performance, compatibility, security, timeline?
4. **Prior art** — Anything similar already in the codebase or that
   you've used elsewhere?
5. **Success criteria** — How will you know it works?

Stop interviewing when you have enough to propose approaches.
Usually 3-5 questions. Never more than 7.

### 4. Propose 2-3 Approaches

Present conversationally:
- Lead with your recommendation and why
- Each approach: 2-3 sentences + key tradeoff
- Be opinionated — don't hedge equally between options

Ask user to pick or refine.

### 5. Present Design Sections

Scale each section to its complexity. A few sentences if
straightforward, more detail if nuanced. Ask after each section
whether it looks right.

Sections (include only what's relevant):
- **Architecture** — high-level structure, key components
- **Data flow** — how data moves through the system
- **API surface** — public interfaces, contracts
- **Error handling** — failure modes, recovery
- **Testing strategy** — what to test, how

After each section, ask: "Does this look right, or should we adjust?"

### 6. Store Design

Once all sections approved, store the design:

1. `echo "<findings>" | ck plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "brainstorm"`
2. `TaskUpdate(taskId, metadata: {design: "<findings>", plan_file: "<filename from stdout>", status_detail: "review"}, description: "Brainstorm: <topic> — findings in plan file and metadata.design")`

The design format below is the findings content:

```
## Problem
<from interview>

## Chosen Approach
<selected approach with rationale>

## Design
<approved sections>

## Next Steps
**Phase N: <title>**
Files: expected/path/to/file.ts
Approach: <what to build and why>
1. <specific step>
2. <specific step>
```

### 7. Output Summary

```
Brainstorm: t<id> — <topic>
Problem: <1 sentence>
Approach: <1 sentence>

Phases:
1. <title> — <key files>
2. <title> — <key files>

Next: /prepare t<id>
```

### 8. After Completion

After outputting the summary, proceed: Invoke Skill tool: skill="prepare", args="<task-id>"

## Key Principles

- **One question at a time** — don't overwhelm
- **YAGNI ruthlessly** — cut features that aren't essential
- **Be opinionated** — recommend, don't just list options
- **Incremental approval** — section by section, not all at once
- **Design can be short** — a simple feature gets a simple design.
  Don't pad for ceremony.

## Key Rules

- Main thread handles dialogue — subagent only for context scan
- Findings stored via plan-storage (plan file + task metadata)
- Next Steps must include file paths (prepare depends on it)
- YAGNI: if user describes scope creep, push back
