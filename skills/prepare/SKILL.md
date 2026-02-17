---
name: prepare
description: "Convert exploration or review findings into epic + phased child tasks. Triggers: 'prepare', 'prepare work', 'create tasks from plan'."
argument-hint: "[t<id>|<task-id>]"
user-invocable: true
allowed-tools:
  - Task
  - Bash
  - Read
  - Glob
  - Grep
  - AskUserQuestion
  - Skill
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
---

# Prepare

Findings → epic + phased tasks with design briefs.
Reads task description, creates implementable task hierarchy.

**Dispatch task creation to subagent.** Main thread handles plan lookup and epic creation (steps 1-4), subagent creates child tasks (step 5).

## Mid-Skill Interviewing

Use AskUserQuestion when facing genuine ambiguity during execution:
- Unclear task boundaries (should X and Y be one task or two?) → ask splitting preference
- Dependency ambiguity (can tasks run parallel or must they serialize?) → confirm with user

Do NOT ask when the answer is obvious or covered by the task brief.

## Steps

1. **Find plan source:**
   - If arg matches a file path → use it directly
   - If arg looks like a task ID → `TaskGet(taskId)`, extract `metadata.design`
   - If no args → `ck plan latest` (finds most recent plan file for current project)
   - If still none → `TaskList()` filtered by status=in_progress + metadata.status_detail==="review" + metadata.label in ["explore", "review", "fix", "brainstorm"], use first match, extract `metadata.design`
   - No plan found → suggest `/explore` or `/review`

2. **Pre-check design quality:**
   - Must have structured sections (Phase, Step, or numbered groups) with file paths
   - Missing file paths or vague descriptions → suggest re-running `/explore` with more detail, STOP

3. **Parse plan:**
   - If source is a plan file: `ck tool phases <file>` → JSON array of `{phase, title, tasks, deps}`
   - If source is task metadata.design: write to temp file, run `ck tool phases <tmpfile>`
   - Extract title from first heading or the plan file's frontmatter topic

4. **Create epic (parent task):**

   Generate slug: `ck tool slug "<title>"` → kebab-case, max 50 chars.

   ```
   TaskCreate:
     subject: "<title>"
     description: "## Problem\n<from design>\n\n## Solution\n<from design>\n\n## Acceptance Criteria\n- [ ] All phases complete\n- [ ] <criteria>"
     activeForm: "Creating epic"
     metadata:
       project: <repo root from git rev-parse --show-toplevel>
       slug: "<slug from ck tool slug>"
       priority: 1
   ```

5. **Create all tasks** — dispatch ONE sonnet subagent (subagent_type="general-purpose", model=sonnet) to create ALL tasks.
   The subagent needs TaskCreate, TaskUpdate, TaskGet in its allowed-tools (specify in the Task dispatch).

   ```
   Create implementation tasks for all phases

   ## All Phases (from description)
   <all phase descriptions, files, approaches>

   ## Epic
   <epic-id>

   ## Project
   <repo root from git rev-parse --show-toplevel>

   ## Job
   For each phase:
   1. Read referenced files to understand current structure
   2. Design exact changes needed
   3. Create task with design brief:

   TaskCreate:
     subject: "Phase N: <task-title>"
     description: "## Context\nEpic: <epic-id>\n\n## Goal\n[what needs to be implemented + why]\n\n## Files\n- Read: exact/path/to/file (why: understand current X)\n- Modify: exact/path/to/file (why: add Y)\n- Create: exact/path/to/test (why: verify Z)\n\n## Approach\n[how to implement: patterns to use, key decisions, implementation strategy]\n\n## Acceptance Criteria\n- [ ] [testable criterion 1]\n- [ ] [testable criterion 2]\n- [ ] No regressions\n\n## Assumptions\n- [what must be true about file structure]\n- [what must be true about imports/exports]\n- [what must be true about dependencies]"
     activeForm: "Creating phase N task"
     metadata:
       project: <repo root from ## Project above>
       type: "chore"
       parent_id: "<epic-id>"

   4. Set dependencies — phase 2+ tasks:
      TaskUpdate(taskId, addBlockedBy: [<previous-phase-task-ids>])

   5. Return all task IDs as a list: "Created: task-1, task-2, task-3"

   ## Quality Requirements
   - Exact file paths with Read/Modify/Create labels
   - Clear implementation approach and key decisions
   - Testable acceptance criteria
   - Explicit assumptions about file structure
   - Each task = one logical unit (one feature/fix/change)
   ```

   Task titles MUST start with "Phase N:" — implement uses this
   for sequencing. Number matches the phase from the design.

   Process all phases in one dispatch (subagent has epic-id for all).

6. **Validate task quality** (subagent-trust.md): spot-check that
   created tasks have real file paths (Read 1-2 referenced files),
   acceptance criteria are testable, and approach is specific enough
   for a worker to implement without guessing. Vague tasks → send
   back to subagent with specific feedback.

7. **Finalize:**
   - `TaskUpdate(epicId, status: "in_progress")`
   - If source was a plan file AND all tasks created successfully → archive it: `ck plan archive <filepath>`
   - If source was a task → `TaskUpdate(sourceId, status: "completed", metadata: {status_detail: null})`

8. **Report:**
   ```
   Epic: <slug> — <title>
   Phases: N tasks created (t<first>–t<last>)

   ┌────┬──────────────────────────────────┬────────────┐
   │    │ Task                             │ Blocked by │
   ├────┼──────────────────────────────────┼────────────┤
   │ p1 │ t<id>: <title>                   │ —          │
   │ p2 │ t<id>: <title>                   │ p1         │
   │ …  │ …                                │ …          │
   └────┴──────────────────────────────────┴────────────┘

   Next: /implement <slug>
   ```

9. **Continuation prompt:**
   Use AskUserQuestion:
   - "Continue to /implement <slug>" (Recommended) — description: "Execute implementation tasks"
   - "Review tasks first" — description: "Inspect the created tasks before implementing"
   - "Done for now" — description: "Leave task active for later /next"

   If user selects "Continue to /implement":
   → Invoke Skill tool: skill="implement", args="<slug>"

## Error Handling
- No description content → "Run `/explore` first to generate a design", stop
- TaskCreate fails for epic → check Task tools available, report error
- Subagent fails on phase → report which phase, continue others, note gap in report
