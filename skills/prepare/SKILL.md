---
name: prepare
description: "Convert exploration or review findings into epic + phased child tasks with design briefs. Triggers: 'prepare', 'prepare work', 'create tasks from plan', 'turn findings into tasks', 'make a task list from this', 'plan out the work'. Do NOT use when: converting user feedback (not explore findings) into tasks — use /triage instead."
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

Findings → epic + phased tasks with design briefs. Main thread: steps 1-4; subagent: step 5.

## Interviewing

See rules/skill-interviewing.md.

## Steps

1. **Find plan source:**
   - File path arg → use directly
   - Task ID arg → TaskGet, extract metadata.design; if metadata.plan_file, run `ct plan latest --task-file <metadata.plan_file>`
   - No args → `ct plan latest`
   - Still none → TaskList (status_detail==="review", type in ["explore","review","triage"]), first match's metadata.design
   - No plan → suggest `/explore`, stop

2. **Pre-check design quality:**
   - Must have structured sections with file paths
   - Missing paths/vague → AskUserQuestion: "Findings lack detail. Continue or re-run /explore?"
   - Standalone testing phase → merge each test file into the implementation phase for the code it tests (match by subsystem/file prefix, e.g. auth tests → auth phase). TDD means tests live with the code they verify.
   - Single-phase spanning 3+ subsystems → AskUserQuestion: "Split into focused phases or keep as one?"

3. **Parse plan:** `ct tool phases <file>` → JSON array of `{phase, title, tasks, deps}`. For metadata.design: write to temp file first. Extract title from first heading.

4. **Create epic:** TaskCreate with title, Problem/Solution/Acceptance, `metadata: {project: REPO_ROOT, slug: <from ct tool slug>, type: "epic", priority: "P1", design: <source design>}`.

5. **Create tasks:** Dispatch ONE subagent (model="sonnet"). Prompt in `references/task-creation-prompt.md`. Process all phases in one dispatch.

6. **Validate**: spot-check file paths (Read 1-2), acceptance criteria, approach. Vague → send back.

7. **Finalize:** TaskUpdate(epicId, status: "in_progress", owner: "prepare"). Source task → TaskUpdate(sourceId, status: "completed").

8. **Report:**
   ```
   Epic: <slug> — <title>
   Phases: N tasks (t<first>–t<last>)

   ┌────┬──────────────────────────┬────────────┐
   │    │ Task                     │ Blocked by │
   ├────┼──────────────────────────┼────────────┤
   │ p1 │ t<id>: <title>           │ —          │
   │ p2 │ t<id>: <title>           │ p1         │
   └────┴──────────────────────────┴────────────┘
   ```

9. After reporting, auto-proceed: `Skill("implement", "<slug>")`.

## Error Handling

- No content → "Run `/explore` first", stop
- Epic TaskCreate fails → check tools available, report
- Subagent fails on phase → report which, continue others, note gap
