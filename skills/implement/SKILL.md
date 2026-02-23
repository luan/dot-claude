---
name: implement
description: "Execute an epic or task — auto-detects solo vs parallel vs swarm mode, dispatches subagents. Triggers: 'implement', 'execute the plan', 'build this', 'code this plan', 'start implementing', 'go implement', 'kick off the tasks', 'run this epic', epic/task ID. Do NOT use when: a full autonomous end-to-end workflow is needed — use /vibe instead."
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

# Implement

**IMMEDIATELY dispatch.** Never implement on main thread.

## Step 1: Find Work

Resolve argument:
- Slug (non-numeric) → `TaskList()`, match `metadata.slug`
- `t<N>` or bare number → `TaskGet(N)`
- No argument → first in_progress epic, else first pending epic, else first unblocked task
- Nothing found → suggest `/explore` then `/prepare`, stop

**Recovery:** Before classifying, check for orphaned epics (`metadata.impl_team` set AND `status == "in_progress"`). Only auto-recover when no explicit argument given.
- Team config exists → re-enter Rolling Scheduler from current metadata counters. Re-dispatch unresponsive workers.
- Config missing → clear `impl_team`, dispatch remaining pending children sequentially (up to 4) via Standalone prompts.
- After recovery → skip to Teardown.

## Step 2: Classify

`TaskGet(taskId)` + recursive descendant scan via `metadata.parent_id` chains. **Leaves** = no children.

- No descendants → **Solo**
- 2-3 independent leaves (no blockedBy) → **Parallel**
- 4+ leaves OR any blockedBy → **Swarm**

**Readiness check (Parallel/Swarm):** 2+ tasks lack `## Files` or `## Approach` → AskUserQuestion before dispatching any workers.

## Worker Dispatch

All modes use `Task(subagent_type="general-purpose")`. Trivial tasks use `model="sonnet"`. Cap: 4 concurrent, 2 retries. Prompt variants in `references/worker-prompts.md`:

- **Standalone** (Solo/Parallel/fallback): no messaging, returns directly
- **Team-based** (Swarm): adds SendMessage + shutdown handshake

Codex routing: `codex` available + leaf task → Codex first, Claude fallback. See `references/scheduler.md`.

## Solo Mode

1. Set task in_progress. Walk ancestor chain for epic context.
2. Spawn single standalone worker.
3. Verify completed → `Skill("acceptance")` → Stage Changes.

## Parallel Mode

1. Store `impl_mode: "parallel"`. Pre-flight: children exist with descriptions.
2. Spawn ALL children as standalone workers in a single message (up to 4, queue remainder).
3. Wait. Retry incomplete (max 2 per task).
4. Clear impl_mode → Verify → `Skill("acceptance")` → Stage Changes.

## Swarm Mode

Tasks have dependency waves (blockedBy relationships). Every task dispatches via subagent.

1. **Setup:** `TeamCreate(team_name="impl-<slug>")`. If fails → fall back to standalone sequential dispatch (up to 4 concurrent). Detect Codex via `which codex`.
2. **Rolling Scheduler:** Pre-compute dependency waves: wave 1 = unblocked tasks (no pending blockedBy); subsequent waves = tasks whose blockedBy all resolved. Dispatch wave 1 immediately (up to 4 concurrent), advance as workers complete. See `references/scheduler.md`.
3. **Verify:** Full test suite. Red → spawn fix agent (max 2 cycles). Still red → escalate to user.
4. **Teardown:** Clear all impl_* metadata, complete epic, TeamDelete, `Skill("acceptance")`, Stage Changes.

## Stage Changes

After all workers: `git add -u`, ask about untracked files, show `git diff --cached --stat`. Stop — user verifies before review.
