# Task Creation Reference

Inline reference for the develop orchestrator. Follow these rules when creating tasks in Step 1b.

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
  activeForm: "<present-continuous of task work, e.g. 'Implementing auth middleware'>"
  metadata:
    project: <repo root from ## Project above>
    type: <infer: "bug" if fixing broken behavior, "feature" if new capability, "chore" for everything else>
    priority: <inherit from epic, default "P2">
    parent_id: "<epic-id>"
    depth: 1  # 1 = phase task; sub-tasks get depth 2; leaves get depth 3
    design: "<compact summary: goal + key files + approach strategy>"

Decomposition rule: if a phase has 3+ distinct implementation concerns
(each touching different files/components), split into sub-tasks.
This is about task *scope boundaries*, not code abstraction — a phase
with 3 concerns touching different modules warrants 3 focused tasks
even if code within each task stays flat.
- Create phase task as grouping node (parent_id: "<epic-id>", depth: 1)
  with summary description
- Each concern → sub-task: parent_id: "<phase-task-id>", depth: 2
- Depth limit: never exceed depth 3 (1=phase, 2=sub-task, 3=leaf).
  Flatten to ≤3 if needed.
- Flat phases (1-2 concerns) stay as single depth-1 tasks

4. Set dependencies — default: sequential phases block in order (phase 2 blocked by phase 1, etc.).
   Override: if two phases touch completely independent files/subsystems with no data flow between them, they MAY be independent.
   Within a phase: sub-tasks block siblings when one produces what another consumes.
   TaskUpdate(taskId, addBlockedBy: [<ids of predecessor tasks>])

5. Return all task IDs: "Created: task-1, task-2, task-3"

## Quality Requirements
- Exact file paths with Read/Modify/Create labels
- Approach must use domain terms from the source phase description (preserve terminology traceability)
- Testable acceptance criteria
- Explicit assumptions about file structure
- Each task = one logical unit (one feature/fix/change)
- **TDD is per-task, never a separate phase.** Every task includes writing tests
  (red-green-refactor). Dedicated "testing" phases → fold into implementation tasks.
- Every task must have `metadata.design` — compact summary of goal + key files + approach

## Upstream Bug Prevention
When creating tasks, embed these verification steps in the Approach section to prevent bugs at design time:
- **Existing utility search**: If a task involves creating a helper (version string, env detection, header builder), add "Search codebase for existing implementations before writing new one" to the approach
- **Domain assumption verification**: If a task depends on how an external field/API/protocol behaves, add "Read source definition of [field] to verify [assumption]" to the approach. Example: if filtering by author_id, verify whether it means "last modifier" or "original creator"
- **Return value accounting**: If a task involves calling a function that returns multiple values, add "Account for all return values — do not discard without confirming consumer needs"
- **Error handling specificity**: If a task adds error handling, add "Distinguish transient (network) vs permanent (auth) vs cancelled errors — do not map all to one type"
- **Safe defaults**: If a task involves fallback behavior on error, add "Never silently fall back to production/permissive default — propagate or warn"
```

Task titles MUST start with "Phase N:" — develop uses this for sequencing.
