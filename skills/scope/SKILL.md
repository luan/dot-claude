---
name: scope
description: "Research an existing codebase and create phased implementation tasks with design context. Triggers: 'scope', 'research', 'investigate', 'design', 'architect', 'plan a feature', 'how does X work', 'figure out', 'best way to', 'state of the art', 'which lib/tool', 'create tasks from plan'. Also use when an implementation request contains an unresolved technology choice. Do NOT use when: the user wants to brainstorm design options for a greenfield feature — use /brainstorm instead."
argument-hint: "<prompt> [--continue] [--no-develop] [--auto-approve]"
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
  - Read
  - Glob
  - Grep
  - Write
  - TeamCreate
  - TeamDelete
  - SendMessage
  - EnterPlanMode
  - ExitPlanMode
---

# Scope

Research → optimistic epic + tasks → plan file → plan mode approval → develop. **Never research on main thread.**

## Interviewing

See rules/skill-interviewing.md.

## New Scope

!`test -n "$CLAUDE_CODE_TASK_LIST_ID" && echo "" || echo "⚠ CLAUDE_CODE_TASK_LIST_ID is not set — TaskCreate/TaskUpdate/TaskList/TaskGet will not work. Tell the user to set it in .claude/settings.json under env, then retry. Stop here."`

1. **Create tracking task:** TaskCreate: subject "Scope: \<topic\>", metadata `{project: <repo root>, type: "scope", priority: "P2"}`. TaskUpdate(taskId, status: "in_progress", owner: "scope").

2. **Research:** Dispatch Task (subagent_type="Explore"):

```
Research <topic>. Return findings as text (do NOT write files or create tasks).

## Output
1. **Current State**: per file — path, exports/defines, patterns
2. **Recommendation**: chosen approach + rationale
3. **Key Files**: exact paths to modify/create
4. **Risks**: edge cases, failure modes
5. **Next Steps** — per phase: title, file paths, approach, steps

## Escalation
3+ independent subsystems or 3+ viable approaches → "ESCALATE: team — <reason>"
```

   **On "ESCALATE: team":** TeamCreate, dispatch 3 agents (mode: "plan") — Researcher, Architect, Skeptic. Synthesize: Architect's approach + contradictions vs Skeptic. TeamDelete.

3. **Validate research:** spot-check ALL architectural claims. File/behavioral claims: check every odd-numbered claim (1st, 3rd, 5th...), minimum 3. Each check: Grep or Read a few lines to confirm existence — do NOT read entire files. Failed check → follow-up subagent.

4. **Store findings:** TaskUpdate(taskId, metadata: {design: "\<findings\>", status_detail: "review"}). The design field must be substantive — full recommendation, per-phase breakdown with file paths, risks. If a reader can't understand the plan from metadata.design alone, it's too sparse.

5. **Create epic:** TaskCreate with title, Problem/Solution/Acceptance, `metadata: {project: REPO_ROOT, slug: <topic-slug>, type: "epic", priority: "P1", design: <source design>}`.

6. **Create tasks:** Dispatch ONE subagent (model="sonnet"). Read `references/task-creation-prompt.md` and use it verbatim as the subagent prompt — do NOT write an ad-hoc prompt. Each task gets `metadata.design` with goal + key files + approach.

7. **Validate tasks:** spot-check 1-2 task file paths (Read), acceptance criteria, approach. Plus all decomposed sub-tasks. Vague → send back to subagent.

8. **Finalize epic:** Collect child task IDs from step 6. Two updates, both required:
   - `TaskUpdate(epicId, status: "in_progress", metadata: {children: [<child IDs>]})` — the `children` array is the canonical epic→task link (without it, discovering children requires walking all tasks by `parent_id`).
   - `TaskUpdate(trackingTaskId, status: "completed")` — closes the scope tracking task created in step 1.

9. **Write plan file:** Write `plans/<slug>.md`. This file is injected into fresh sessions as context — its format must trigger develop reliably.

   Example (webhook-retry epic #600, tasks #601, #602):

   ```markdown
   # /develop webhook-retry
   Epic: #600 | Children: #601, #602
   Status: approved — execute immediately

   Add exponential backoff retry to webhook delivery.

   ## Tasks
   | Task | Title | Files | Approach | Blocked By |
   |------|-------|-------|----------|------------|
   | #601 | Add retry queue | Create: RetryQueue.swift | Redis-backed FIFO with delay scheduling | — |
   | #602 | Wire into webhook sender | Modify: WebhookSender.swift | Replace fire-and-forget with queue dispatch | #601 |

   ## Context
   <design details, key decisions, verification steps>
   ```

   **Why this format:** The H1 `# /develop <slug>` is the first line — when the plan file is loaded into a fresh session, natural language routing matches `/develop` and invokes the skill with the slug. Line 2 gives develop the epic + children IDs without searching. All values must be real IDs from the steps above — no placeholders.

10. **Plan mode:** EnterPlanMode. First line: `Scope: #<tracking task ID>`. Present the plan file content — the user is reviewing real tasks with real IDs, not a hypothetical outline. Reference the plan file path.

   **If `--auto-approve`:** skip EnterPlanMode/ExitPlanMode — output plan as text, proceed to step 11. Used by `/vibe` for autonomous execution.

   Otherwise: user reviews. On approval → ExitPlanMode, proceed to step 11.

   **ExitPlanMode rejected:** See Rejection Handling below.

11. **Report + Develop:** Output scope task ID, epic ID + slug, phased task table (task ID, title, blockedBy per row). Then `Skill("develop", "<slug>")` immediately. Skip develop only if `--no-develop` was passed.

## Rejection Handling

When ExitPlanMode is rejected, ask what to change (AskUserQuestion or text). Then:

- **Minor changes** (wording, acceptance criteria, approach tweaks): update existing tasks via TaskUpdate, update plan file, re-present in plan mode, retry ExitPlanMode.
- **Scope change** (different approach, tasks need restructuring): delete affected tasks, optionally delete epic if slug changes. Re-run from step 5 or 6 depending on what changed. Update plan file.
- **Full rejection** (wrong direction entirely): delete epic + all tasks. Re-run from step 2 (new research).

Do not dead-end. Do not leave orphaned tasks from a rejected plan.

## Continuation (--continue)

Resolve task: argument → task ID; bare → TaskList `type === "scope"` + `status_detail` in `["review", "approved"]`, most recent. Extract `metadata.design`.

- `status_detail === "approved"` + no epic children exist → plan was approved but tasks weren't created (interrupted session). Resume from step 5 using existing `metadata.design`.
- Epic + children exist but no plan file → resume from step 9.
- Otherwise → dispatch subagent with prior + new prompt, merge findings. Re-enter from step 4.

## Design Refinement

If user feedback during plan mode changes the recommendation, revise from stored findings + user feedback only. Do NOT read codebase files — if refinement needs new data, use `--continue` instead. Update tasks and plan file, re-propose in plan mode.

## Key Rules

- Main thread does NOT research the codebase — subagent does. Refinement uses stored findings only.
- Epic + tasks are created **before** plan mode (optimistically). User reviews real tasks with real IDs, not a hypothetical outline. On rejection, update or delete — never present an empty plan.
- Plan file at `plans/<slug>.md` is required — it's the durable artifact that survives session end and references task IDs + develop handoff.
- Design stored in `metadata.design` on tracking task and epic. Per-task briefs in each child's `metadata.design`.
- Research Next Steps must include file paths — task creation depends on them.
