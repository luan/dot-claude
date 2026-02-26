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
---

# Develop

**IMMEDIATELY dispatch.** Never implement on main thread.

## Step 1: Find Work

Resolve argument:
- Slug (non-numeric) → `TaskList()`, match `metadata.slug`
- `t<N>` or bare number → `TaskGet(N)`
- No argument → first in_progress epic, else first pending epic, else first unblocked task
- Nothing found → suggest `/scope`, stop

**Recovery:** Before classifying, check for orphaned epics (`metadata.impl_team` set AND `status == "in_progress"`). Only auto-recover when no explicit argument given.
- **Children check first:** scan all children. All completed → clear `impl_team`, skip to Teardown (no re-dispatch needed). Some pending + some completed → only dispatch pending children. Never reset, re-run, or modify completed children — their work and status are preserved unconditionally.
- Team config exists → re-enter Rolling Scheduler from current metadata counters. Re-dispatch unresponsive workers.
- Config missing → clear `impl_team`, dispatch remaining pending children sequentially (up to 4) via Standalone prompts.
- After recovery → skip to Teardown.

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

**Re-scope escape hatch:** Worker output containing `RESCOPE:` signals a fundamental design conflict (wrong approach, missing prerequisite, contradictory requirements). Do not retry — invoke `Skill("scope", "--continue <epicId>")` to re-scope, then restart dispatch from Step 2 with updated tasks.

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
2. `git add -u`, ask about untracked files, show `git diff --cached --stat`. Stop — user verifies before review.
