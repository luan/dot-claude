---
name: implement
description: "Execute an epic or task — auto-detects solo vs parallel vs swarm mode, dispatches subagents. Triggers: 'implement', 'execute the plan', 'build this', 'code this plan', 'start implementing', epic/task ID. Do NOT use when: a full autonomous end-to-end workflow is needed — use /vibe instead."
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
- Team config exists → re-enter Rolling Scheduler using current metadata counters as starting state. Shut down unresponsive workers, re-dispatch their tasks.
- Config missing (team died) → clear `impl_team`, dispatch remaining pending children sequentially (up to 4) using Standalone Worker Prompt.
- After recovery → skip to Teardown.

## Step 2: Classify

`TaskGet(taskId)` + recursive descendant scan via `metadata.parent_id` chains. **Leaves** = no children.

- No descendants → **Solo**
- 2-3 independent leaves (no blockedBy) → **Parallel**
- 4+ leaves OR any blockedBy dependencies → **Swarm**

**Readiness check (Parallel/Swarm):** If 2+ tasks lack `## Files` or `## Approach` → ask user to continue or refine briefs first.

## Worker Dispatch

All modes dispatch via `Task(subagent_type="general-purpose")`. Trivial tasks (single-file rename, config tweak) use `model="sonnet"`. Two prompt variants — see `references/worker-prompts.md`:

- **Standalone** (Solo/Parallel/fallback): no messaging, worker returns directly
- **Team-based** (Swarm): adds SendMessage to team lead + shutdown handshake

Codex routing: when `codex` CLI is available and task is a leaf, dispatch via Codex first. On failure, fall back to Claude worker. See `references/scheduler.md`.

## Solo Mode

1. Set task in_progress. Walk ancestor chain (`metadata.parent_id`) for epic context.
2. Spawn single standalone worker.
3. Verify completed → `Skill("acceptance")` → Stage Changes.

## Parallel Mode

1. Store `impl_mode: "parallel"`. Pre-flight: children exist with descriptions.
2. Spawn ALL children as standalone workers in a single message (up to 4, queue remainder).
3. Wait. Retry incomplete (max 2 per task).
4. Clear impl_mode → Verify → `Skill("acceptance")` → Stage Changes.

## Swarm Mode

Tasks have dependency waves (blockedBy relationships). Every task dispatches via subagent.

1. **Setup:** `TeamCreate(team_name="impl-<slug>")`. If fails → fall back to standalone sequential dispatch (up to 4 concurrent, same rolling logic). Detect Codex availability via `which codex`.
2. **Rolling Scheduler:** Dispatch leaf tasks as dependencies resolve, up to 4 concurrent workers. See `references/scheduler.md` for full pseudocode and Codex routing.
3. **Verify:** Full test suite. Red → spawn fix agent (max 2 cycles). Still red → escalate to user.
4. **Teardown:** Clear all impl_* metadata, complete epic, TeamDelete, `Skill("acceptance")`, Stage Changes.

## Stage Changes

After all workers complete: `git add -u`, check for untracked files (ask user whether to stage), show `git diff --cached --stat`.

## After Completion

Stop after staging and showing summary. User verifies functionality before review — do not auto-invoke review.
