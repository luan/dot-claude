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

## Interviewing

See rules/skill-interviewing.md. Skill-specific triggers:
- Unclear task boundaries (should X and Y be one task or two?) → ask splitting preference
- Dependency ambiguity (can tasks run parallel or must they serialize?) → confirm with user

## Steps

1. **Find plan source:**
   - If arg matches a file path → use it directly
   - If arg looks like a task ID → `TaskGet(taskId)`, extract `metadata.design`; if `metadata.plan_file` is set, run `ck plan latest --task-file <metadata.plan_file>` to resolve the plan file deterministically, otherwise fall through to bare `ck plan latest`
   - If no args → `ck plan latest` (finds most recent plan file for current project)
   - If still none → `TaskList()` filtered by status=in_progress + metadata.status_detail==="review" + metadata.type in ["explore", "review", "fix"], use first match, extract `metadata.design` (note: brainstorm tasks use type="explore" and are found by this filter)
   - No plan found → suggest `/explore` or `/review`

2. **Pre-check design quality:**
   - Must have structured sections (Phase, Step, or numbered groups) with file paths
   - Missing file paths or vague descriptions → suggest re-running `/explore` with more detail, STOP
   - If plan has a standalone testing/test phase → merge its test items into the implementation phases they cover before proceeding (TDD: tests live with the code they verify, not in a later phase)

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
       type: "epic"
       priority: "P1"
   ```

5. **Create all tasks** — dispatch ONE sonnet subagent (subagent_type="general-purpose", model=sonnet) to create ALL tasks. The subagent needs TaskCreate, TaskUpdate, TaskGet in its allowed-tools (specify in the Task dispatch).

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
     description: "## Context\nEpic: <epic-id>\n\n## Goal\n[what needs to be implemented + why]\n\n## Files\n- Read: exact/path/to/file (why: understand current X)\n- Modify: exact/path/to/file (why: add Y)\n- Create: exact/path/to/test (why: verify Z)\n\n## Approach\n[how to implement: patterns to use, key decisions, implementation strategy]\n- TDD: write failing tests first, then implement\n\n## Acceptance Criteria\n- [ ] [testable criterion 1]\n- [ ] [testable criterion 2]\n- [ ] Tests written before implementation (or noted if no test infra)\n- [ ] No regressions\n\n## Assumptions\n- [what must be true about file structure]\n- [what must be true about imports/exports]\n- [what must be true about dependencies]"
     activeForm: "Creating phase N task"
     metadata:
       project: <repo root from ## Project above>
       type: <infer from task content: "bug" if fixing broken behavior or correcting a defect, "feature" if adding new user-visible capability, "chore" for everything else (refactoring, cleanup, config, tests)>
       priority: <inherit from epic, default "P2">
       parent_id: "<epic-id>"

   4. Set dependencies based on actual data/code dependencies between tasks:
      TaskUpdate(taskId, addBlockedBy: [<ids of tasks that produce files, APIs, or state this task consumes>])
      Heuristic: if task B modifies a file that task A creates, or task B imports/uses an API that task A defines, B is blocked by A. If two tasks modify different files, they are independent.
      Independent tasks across phases should NOT block each other.

   5. Return all task IDs as a list: "Created: task-1, task-2, task-3"

   ## Quality Requirements
   - Exact file paths with Read/Modify/Create labels
   - Clear implementation approach and key decisions
   - Testable acceptance criteria
   - Explicit assumptions about file structure
   - Each task = one logical unit (one feature/fix/change)
   - **TDD is per-task, never a separate phase.** Every implementation task includes writing tests as part of its workflow (red-green-refactor). If the source plan has a dedicated "testing" or "add tests" phase, fold those tests into the implementation tasks they verify. Never create a task whose sole purpose is writing tests for work done in earlier tasks.
   ```

   Task titles MUST start with "Phase N:" — implement uses this for sequencing. Number matches the phase from the design.

   Process all phases in one dispatch (subagent has epic-id for all).

6. **Validate task quality** (subagent-trust.md): spot-check that created tasks have real file paths (Read 1-2 referenced files), acceptance criteria are testable, and approach is specific enough for a worker to implement without guessing. Vague tasks → send back to subagent with specific feedback.

7. **Finalize:**
   - `TaskUpdate(epicId, status: "in_progress", owner: "prepare")`
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

9. After outputting the summary, proceed: Invoke Skill tool: skill="implement", args="<slug>"

## Error Handling
- No description content → "Run `/explore` first to generate a design", stop
- TaskCreate fails for epic → check Task tools available, report error
- Subagent fails on phase → report which phase, continue others, note gap in report
