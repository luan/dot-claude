---
name: handoff
description: "Snapshot session state into a handoff task before compaction. Triggers: '/handoff', 'hand off', 'transfer session', 'prep for new session', 'context is full'. Do NOT trigger for: 'hand this off to <person>' — person-directed handoffs."
argument-hint: "[--resume [task-id]]"
user-invocable: true
allowed-tools:
  - "Bash(git branch:*)"
  - "Bash(git status:*)"
  - "Bash(git log:*)"
  - TaskList
  - TaskGet
  - TaskCreate
  - TaskUpdate
  - Skill
---

# Handoff

Snapshot session state (git context + active tasks) into a structured handoff task before compacting.

## Context

Branch: !`git branch --show-current 2>/dev/null || echo "(detached)"`
Status: !`git status -sb 2>/dev/null | head -20`
Recent: !`git log --oneline -5 2>/dev/null`

## Arguments

- `--resume [task-id]` → Resume Mode
- No arguments → Create Mode

If `--resume` is present, go to Resume Mode. Otherwise, go to Create Mode.

## Resume Mode

### 1. Load handoff task

- If `--resume t<id>` given → `TaskGet(id)`. Validate: `metadata.type === "handoff"` AND `status === "in_progress"`. If either fails → stop: "t<id> is not a pending handoff task (type=<type>, status=<status>). Check the task ID or run `/handoff --resume` to find the latest."
- If `--resume` with no id → `TaskList()` → filter `metadata.type === "handoff"` AND `status === "in_progress"` → sort by `metadata.created_at` descending → take first
- If none found → "No pending handoff task found. Run `/handoff` first." Stop.

### 2. Stale guard

Parse `metadata.created_at` from the handoff task. If >24h ago → warn:

"Handoff task t<id> is from <date> (>24h old) — state may be stale. Proceed? [y/N]"

Stop if user says no.

### 3. Surface state

Print: `Resuming handoff t<id> from <created_at>`

- If stored `metadata.branch` differs from current branch (from Context above) → warn: "Branch mismatch: handoff was on `<stored>`, current is `<current>`. Continuing may apply state to the wrong branch."
- If `metadata.dirty_files` was non-empty at handoff time → note: "Note: <N> uncommitted files were present at handoff time — verify working tree."

### 4. Dispatch continuations

Process `metadata.active_tasks` in priority order (highest first):

| Priority | Condition | Action |
|---|---|---|
| 1 | `vibe_stage` present AND not `"commit"` | `Skill("vibe", "--continue")` |
| 2 | `super_vibe === true` | `Skill("supervibe", "--continue")` |
| 3 | `impl_team` set | `Skill("develop", "t<id>")` |
| 4 | `status_detail === "approved"` | `Skill("develop", "t<id>")` |
| 5 | `status_detail` is `"spec_review"` or `"review"` | `Skill("scope", "--continue t<id>")` |
| 6 | anything else in_progress | Surface: "Remaining: t<id> [<type>] <subject> — handle manually" |

If multiple items match → dispatch the single highest-priority item first, surface the rest as "Remaining" lines.

### 5. Complete handoff task

Only if step 4 dispatch succeeded (no error or failure signal from the Skill() call). If dispatch failed → stop: "Dispatch failed — handoff task t<id> left open. Fix the error and re-run `/handoff --resume t<id>`." Do NOT call TaskUpdate.

`TaskUpdate(handoffId, status: "completed")`

## Create Mode

### 1. Collect active tasks

`TaskList()` → filter to tasks with `status === "in_progress"`.

Before proceeding: check if any of the in-progress tasks has `metadata.type === "handoff"`. If found → warn: "Existing handoff t<id> has not been resumed yet. Create a new one anyway? [y/N]" Stop if user says no.

### 2. Build compact summaries

For each in-progress task, `TaskGet(id)` and extract:

| Field | Source |
|---|---|
| id | task ID |
| subject | task subject |
| type | metadata.type |
| status_detail | metadata.status_detail (null if absent) |
| vibe_stage | metadata.vibe_stage (null if absent) |
| super_vibe | true if metadata.super_vibe is truthy, else false |
| impl_team | metadata.impl_team name if set, else null |
| children_count | number of child tasks |

Do NOT include metadata.design, metadata.spec, or metadata.plan content — too large for a handoff snapshot.

### 3. Create handoff task

Parse dirty files from the Context git status output above — skip any line starting with `##` (the branch tracking header), then extract filenames and strip the two-character status prefix (e.g., ` M`, `??`, `A `).

```
TaskCreate(
  subject: "Handoff: <YYYY-MM-DD HH:MM>",
  description: "Session state snapshot for continuity across compaction.",
  metadata: {
    type: "handoff",
    session_id: "${CLAUDE_SESSION_ID}",
    branch: "<from Context above>",
    dirty_files: ["<filenames from git status, prefix stripped>"],
    active_tasks: [
      {id, subject, type, status_detail, vibe_stage, super_vibe, impl_team, children_count}
    ],
    created_at: "<ISO 8601>"
  }
)
```

### 4. Activate handoff task

`TaskUpdate(handoffId, status: "in_progress")`

### 5. Print handoff block

```
## Session Handoff — <date>

Branch: <branch>
Dirty files: <count> (<filenames if ≤5, else "N files — check git status">)

Active work:
  t<id> [<type>] <subject> (<status_detail if set>)
  ...
  (none) if no in_progress tasks

Handoff task: t<handoffId>
Resume: /handoff --resume t<handoffId>
```

### 6. Prompt compaction

Print: `Run /compact when ready.`

## Key Rules

- No working tree modifications — never call git add, git stash, git commit, or any write operation on the repo.
- Handoff task metadata is the source of truth — the printed block is for human readability only.
- One handoff per invocation. If a previous handoff task exists, create a new one (old ones are historical records).
