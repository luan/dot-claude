---
name: develop
description: "Execute implementation for an epic or individual task. Triggers: 'develop', 'execute the plan', 'build this', 'code this plan', 'kick off the tasks', 'run this epic', 'run the plan', epic/task ID. Do NOT use when: a full autonomous end-to-end workflow is needed — use /vibe instead."
argument-hint: "[<epic-slug>|t<id>|<id>] [--solo]"
user-invocable: true
allowed-tools:
  - Task
  - Skill
  - SendMessage
  - TeamCreate
  - TeamDelete
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
  - AskUserQuestion
  - Read
  - Bash
  - Glob
  - Grep
  - Write
---

# Develop

**IMMEDIATELY dispatch.** Never implement on main thread.

## Step 1: Find Work

Resolve argument:
- Slug (non-numeric) → `TaskList()`, match `metadata.slug`
- `t<N>` or bare number → `TaskGet(N)`
- No argument → first in_progress epic, else first pending epic, else first scope task with `status_detail === "approved"`, else first unblocked task
- Nothing found → suggest `/scope`, stop

**Scope task resolution:** if resolved task has `metadata.type === "scope"` and `status_detail === "approved"` → Preparation mode (Step 1b).

**Epic without children:** if resolved task has `metadata.type === "epic"` with `metadata.design` but no children → Preparation mode (Step 1b), skip epic creation.

**Recovery:** Before classifying, check for orphaned epics (`metadata.impl_team` set AND `status == "in_progress"`). Only auto-recover when no explicit argument given.
- **Children check first:** scan all children. All completed → clear `impl_team`, skip to Teardown (no re-dispatch needed). Some pending + some completed → only dispatch pending children. Never reset, re-run, or modify completed children — their work and status are preserved unconditionally.
- Team config exists → re-enter Rolling Scheduler from current metadata counters. Re-dispatch unresponsive workers.
- Config missing → clear `impl_team`, dispatch remaining pending children sequentially (up to 4) via Standalone prompts.
- After recovery → skip to Teardown.

## Step 1b: Prepare (from scope findings)

Source: scope task's `metadata.design` (or epic's `metadata.design` if epic exists). If `metadata.design` is empty/missing and `metadata.plan_file` is set, read the plan file as fallback: `ct plan latest --task-file <plan_file>`.

1. **Pre-check design quality:**
   - Must have structured sections with file paths
   - Standalone testing phase → merge into implementation phases
   - Single phase spanning 3+ subsystems → AskUserQuestion
   - Missing paths or approach under 20 words → AskUserQuestion

2. **Create epic** (skip if epic already exists):
   TaskCreate with title, Problem/Solution/Acceptance,
   `metadata: {project: REPO_ROOT, slug: <topic-slug>, type: "epic", priority: "P1", design: <source design>, spec: <source spec if available>}`

3. **Create tasks:** Dispatch ONE subagent (model="sonnet"). Read `references/task-creation-prompt.md` and pass its content verbatim as the subagent prompt — do NOT write an ad-hoc prompt. The reference contains decomposition rules, quality requirements, and format specs that must not be paraphrased.

4. **Validate tasks:** spot-check 1-2 file paths (Read), acceptance criteria,
   approach. Vague → send back to subagent.

5. **Finalize:** Collect child task IDs. Two updates:
   - `TaskUpdate(epicId, status: "in_progress", metadata: {children: [<child IDs>]})`
   - Scope task (if source) → `TaskUpdate(scopeTaskId, status: "completed")`

6. **Report:** epic ID + slug, phased task table with blockedBy.

→ **Immediately fall through to Step 2 (Classify).** Do NOT pause, summarize, or ask the user to confirm. Preparation → dispatch is one continuous flow.

## Step 2: Classify

`TaskGet(taskId)` + recursive descendant scan via `metadata.parent_id` chains. **Leaves** = no children.

- No descendants → **Solo**
- 2+ leaves → **Team**

**Readiness check (Team):** 2+ tasks lack `## Files` or `## Approach` → AskUserQuestion before dispatching any workers.

## Pre-compute Context

After classifying, write `metadata: {breadcrumb: "Epic > Phase > ...", epic_design: "<design>"}` to each leaf task. Source `epic_design` from the root epic's `metadata.design` (populated by scope). Implement-worker and standalone workers use this instead of re-walking ancestors.

## Worker Dispatch

All modes use `Task(subagent_type="general-purpose")`. Trivial tasks use `model="sonnet"`. Cap: 4 concurrent, 2 retries. Prompt variants in `references/worker-prompts.md`:

- **Standalone** (Solo/fallback): no messaging, returns directly
- **Team-based** (Team): adds SendMessage + shutdown handshake

Codex routing: `codex` available + leaf task → Codex first, Claude fallback. See `references/scheduler.md`.

**Re-scope escape hatch:** Worker output containing `RESCOPE:` signals a fundamental design conflict. Immediately halt — do NOT dispatch any remaining workers and do NOT retry the RESCOPE worker. Invoke `Skill("scope", "--continue <epicId>")` to re-scope, then restart dispatch from Step 2 with updated tasks.

## Solo Mode

1. Set task in_progress. Walk ancestor chain for epic context.
2. Spawn single standalone worker.
3. Verify completed → Stage Changes.

## Team Mode

Every task dispatches via subagent. TeamCreate always runs.

1. **Setup:** `TeamCreate(team_name="impl-<slug>")`. If fails → fall back to standalone sequential dispatch (up to 4 concurrent). Detect Codex via `which codex`.
2. **Dispatch:** 4+ leaves with blockedBy chains → **Rolling Scheduler:** dispatch unblocked tasks (up to 4 concurrent), re-scan after each completion to dispatch newly unblocked tasks. See `references/scheduler.md`. Otherwise → dispatch all tasks at once (up to 4 concurrent).
3. **Verify:** Full test suite. Red → spawn fix agent (max 2 cycles). Still red → escalate to user.
4. **Teardown:** Clear all impl_* metadata, complete epic, TeamDelete → Stage Changes.

## Stage Changes

After all workers:
1. `Skill("acceptance", args="<epicId>")`.
2. **Reconcile spec** — if `metadata.spec` exists on epic (or parent scope task):
   Dispatch a subagent (model="sonnet"): input is the spec content + `git diff` (full diff, not just names — the subagent needs to see what changed). Prompt: "Compare this spec against the implementation diff. Update the spec so its Recommendation and Architecture Context accurately describe the system as implemented. Timeless present-tense format: no transition language ('changed from X to Y', 'previously', 'was updated'). Return the updated spec only, or 'NO_CHANGES' if it already matches." If updated: overwrite `metadata.spec`; overwrite `metadata.spec_file` if set; grep repo for the spec filename (e.g., `docs/specs/`) and overwrite if committed.
3. `git add -u`, ask about untracked files, show `git diff --cached --stat`. Stop — user verifies before review.
