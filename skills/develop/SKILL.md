---
name: develop
description: "Execute implementation for an epic or individual task. Triggers: 'develop', 'execute the plan', 'build this', 'code this plan', 'kick off the tasks', 'run this epic', 'run the plan', epic/task ID. Do NOT use when: a full autonomous end-to-end workflow is needed — use /vibe instead."
argument-hint: "[<epic-slug>|t<id>|<id>] [--solo] [--auto]"
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
  - Read
  - Bash
  - Glob
  - Grep
  - Write
---

# Develop

**IMMEDIATELY dispatch.** Never implement on main thread — subagents do all coding so the main thread stays responsive for orchestration.

## Step 1: Find Work

Resolve argument:

- Slug (non-numeric) → `TaskList()`, match `metadata.slug`
- `t<N>` or bare number → `TaskGet(N)`
- No argument → first in_progress epic, else first pending epic, else first scope task with `status_detail === "approved"`, else first unblocked task
- Nothing found → suggest `/scope`, stop

**Scope task resolution:** if resolved task has `metadata.type === "scope"` and `status_detail === "approved"` → Preparation mode (Step 1b).

**Epic without children:** if resolved task has `metadata.type === "epic"` with `metadata.design` but no children → Preparation mode (Step 1b), skip epic creation.

**Recovery:** Before classifying, check for orphaned epics (`metadata.impl_team` set AND `status == "in_progress"`). Only auto-recover when no explicit argument given.

- **Children check first:** scan all children. All completed → clear `impl_team`, skip to Teardown. Some pending + some completed → only dispatch pending children (completed children's work is preserved unconditionally — never reset or re-run them).
- Team config exists → re-enter Rolling Scheduler from current metadata counters.
- Config missing → clear `impl_team`, dispatch remaining pending children via Standalone (up to 4 concurrent).
- After recovery → skip to Teardown.

## Step 1b: Prepare (from scope findings)

Source: scope task's `metadata.design` (or epic's `metadata.design` if epic exists). If `metadata.design` is empty/missing and `metadata.plan_file` is set, read the plan file as fallback: `ct plan latest --task-file <plan_file>`.

1. **Pre-check design quality:**
   - Must have structured sections with file paths
   - Standalone testing phase → merge into implementation phases
   - Single phase spanning 3+ subsystems → `--auto`: proceed anyway. Otherwise AskUserQuestion.
   - Missing paths or approach under 20 words → `--auto`: proceed with best-effort. Otherwise AskUserQuestion.

2. **Create epic** (skip if epic already exists):
   TaskCreate with title, Problem/Solution/Acceptance,
   `metadata: {project: REPO_ROOT, slug: <topic-slug>, type: "epic", priority: "P1", design: <source design>, spec: <source spec if available>}`

3. **Create tasks directly** — no subagent. Read `${CLAUDE_SKILL_DIR}/references/task-creation-prompt.md` for decomposition rules, quality requirements, and TaskCreate format. The orchestrator already has the design context; task creation is mechanical. Read referenced files as needed, then call TaskCreate/TaskUpdate inline.

4. **Validate tasks:** spot-check 1-2 file paths (Read), acceptance criteria,
   approach. Vague → fix inline.

5. **Finalize:** Collect child task IDs. Two updates:
   - `TaskUpdate(epicId, status: "in_progress", metadata: {children: [<child IDs>]})`
   - Scope task (if source) → `TaskUpdate(scopeTaskId, status: "completed")`

6. **Report:** epic ID + slug, phased task table with blockedBy.

→ **Immediately fall through to Step 2 (Classify).** Do NOT pause, summarize, or ask the user to confirm. Preparation → dispatch is one continuous flow.

## Step 2: Classify

`TaskGet(taskId)` + recursive descendant scan via `metadata.parent_id` chains. **Leaves** = no children.

- No descendants → **Solo**
- 2+ leaves → **Team**

**Readiness check (Team):** 2+ tasks lack `## Files` or `## Approach` → `--auto`: proceed with dispatch anyway. Otherwise AskUserQuestion before dispatching any workers.

## Pre-compute Context

After classifying, write `metadata: {breadcrumb: "Epic > Phase > ...", epic_design: "<design>"}` to each leaf task. Source `epic_design` from the root epic's `metadata.design` (populated by scope). Implement-worker and standalone workers use this instead of re-walking ancestors.

## Worker Dispatch

All modes use `Task(subagent_type="general-purpose")`. Trivial tasks use `model="sonnet"`. Cap: 4 concurrent, 2 retries. Prompt variants in `${CLAUDE_SKILL_DIR}/references/worker-prompts.md`:

- **Standalone** (Solo/fallback): no messaging, returns directly
- **Team-based** (Team): adds SendMessage + shutdown handshake

**Re-scope escape hatch:** Worker output containing `RESCOPE:` signals a fundamental design conflict (wrong approach, missing prerequisite). Immediately halt — do NOT dispatch remaining workers or retry. Invoke `Skill("scope", "--continue <epicId>")` to re-scope, then restart from Step 2.

## Solo Mode

1. Set task in_progress. Walk ancestor chain for epic context.
2. Spawn single standalone worker.
3. Verify completed → Stage Changes.

## Team Mode

Every task dispatches via subagent. TeamCreate always runs.

1. **Setup:** `TeamCreate(team_name="impl-<slug>")`. If fails → fall back to standalone sequential dispatch (up to 4 concurrent).
2. **Dispatch:** 4+ leaves with blockedBy chains → **Rolling Scheduler:** dispatch unblocked tasks (up to 4 concurrent), re-scan after each completion to dispatch newly unblocked tasks. See `${CLAUDE_SKILL_DIR}/references/scheduler.md`. Otherwise → dispatch all tasks at once (up to 4 concurrent).
3. **Verify:** Full test suite. Red → spawn fix agent (max 2 cycles). Still red → escalate to user.
4. **Teardown:** Clear all impl\_\* metadata, complete epic, TeamDelete → Stage Changes.

## Completion Summary

After teardown (all workers finished, epic verified), before staging:

1. Collect child task IDs from `metadata.children`. Count N = number of completed children.
2. Run `git diff --stat HEAD~<N>` to get changed files list. Run `git rev-parse HEAD~<N>` and `git rev-parse HEAD` for commit range.
3. Derive a one-line summary from child task subjects (combine into a single sentence describing what was built).
4. `TaskUpdate(epicId, metadata: {completion_summary: {files_changed: [<paths from diff --stat>], commit_range: "<base_sha>..<head_sha>", summary: "<one-line description>"}})`.

## Stage Changes

After all workers:

1. `Skill("acceptance", args="<epicId>")`.
2. **Reconcile spec** — if `metadata.spec` exists on epic (or parent scope task):
   Dispatch a subagent (model="sonnet") with spec content + full `git diff`. Prompt: "Compare spec against implementation diff. Update Recommendation and Architecture Context to match what was actually built. Timeless present-tense — no transition language. Return updated spec or 'NO_CHANGES'." If updated: overwrite `metadata.spec`, `metadata.spec_file` if set, and repo copy (e.g. `docs/specs/`) if committed.
3. `git add -u`, ask about untracked files, show `git diff --cached --stat`. Stop — user verifies before review.
