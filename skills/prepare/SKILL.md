---
name: prepare
description: "Convert exploration or review findings into epic + phased child tasks with design briefs. Triggers: 'prepare', 'prepare work', 'create tasks from plan', 'turn findings into tasks', 'make a task list from this', 'plan out the work'. Do NOT use when: converting user feedback (not explore findings) into tasks — use /fix instead."
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

Findings → epic + phased tasks with design briefs. Main thread handles plan lookup and epic creation (steps 1-4), subagent creates child tasks (step 5).

## Interviewing

See rules/skill-interviewing.md. Ask on: unclear task boundaries (one task or two?), dependency ambiguity (parallel or serial?).

## Steps

1. **Find plan source:**
   - File path arg → use directly
   - Task ID arg → TaskGet, extract metadata.design; if metadata.plan_file set, run `ck plan latest --task-file <metadata.plan_file>`, else fall through
   - No args → `ck plan latest` (most recent plan for current project)
   - Still none → TaskList filtered by status=in_progress + metadata.status_detail==="review" + metadata.type in ["explore", "review", "fix"], use first match's metadata.design
   - No plan found → suggest `/explore` or `/review`, stop

2. **Pre-check design quality:**
   - Must have structured sections (Phase/Step/numbered groups) with file paths
   - Missing paths or vague descriptions → AskUserQuestion: "Findings lack file paths or detail. Continue (workers fill gaps) or re-run /explore?" File paths matter because workers who lack them create wrong files from memory. Continue proceeds as-is; re-run stops.
   - Thin descriptions (one-liner steps, files without rationale) → AskUserQuestion: same choice, but note it's borderline rather than missing.
   - Standalone testing phase → merge test items into the implementation phases they cover. Separate test phases imply testing happens after code; TDD means tests are written in the same task as the code they verify.

3. **Parse plan:**
   - Plan file: `ck tool phases <file>` → JSON array of `{phase, title, tasks, deps}`
   - Task metadata.design: write to temp file, run `ck tool phases <tmpfile>`
   - Extract title from first heading or frontmatter topic

4. **Create epic:** TaskCreate with title, Problem/Solution/Acceptance from design, `metadata: {project: REPO_ROOT, slug: <from ck tool slug>, type: "epic", priority: "P1"}`.

5. **Create tasks:** Dispatch ONE general-purpose subagent (model="sonnet" — mechanical task creation with no architectural decisions, saves cost). Full prompt template in `references/task-creation-prompt.md`. Process all phases in one dispatch.

6. **Validate** (subagent-trust.md): spot-check that created tasks have real file paths (Read 1-2 referenced files), testable acceptance criteria, and specific approach. Vague tasks → send back to subagent with feedback.

7. **Finalize:**
   - TaskUpdate(epicId, status: "in_progress", owner: "prepare")
   - If source was a task → TaskUpdate(sourceId, status: "completed", metadata: {status_detail: null, completedAt: timestamp})

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
