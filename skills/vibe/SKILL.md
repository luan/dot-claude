---
name: vibe
description: "Fully autonomous development workflow from prompt to commit. Chains explore → prepare → implement → commit. Triggers: /vibe, 'vibe this', 'autonomous workflow'. Do NOT use when: only implementing already-prepared tasks — use /implement instead."
allowed-tools: Bash, Read, Glob, Skill, TaskCreate, TaskUpdate, TaskGet, TaskList
argument-hint: "<prompt> [--no-branch] [--continue] [--dry-run]"
user-invocable: true
---

# Vibe

Run the full pipeline (explore → prepare → implement → commit) from a single prompt.

## Arguments

- `<prompt>` — what to build (required unless `--continue`)
- `--no-branch` — skip branch creation, use current branch
- `--continue` — resume from last completed stage
- `--dry-run` — explore + prepare only, stop before implement

If no prompt and no `--continue` → tell user: `/vibe <what to build>`, stop.

## Resume (`--continue`)

1. `TaskList()` → find task with `metadata.vibe_stage` present and `status == "in_progress"`
2. Read `metadata.vibe_stage` for resume point, `metadata.vibe_prompt` as prompt
3. Skip to the stage after `vibe_stage`
4. Not found → tell user no pipeline to resume, stop

## Fresh Start

```
TaskCreate(
  subject: "Vibe: <prompt (truncated 60 chars)>",
  description: "<full prompt>",
  activeForm: "Vibing",
  metadata: { type: "epic", priority: "P2", vibe_prompt: "<full prompt>", vibe_stage: "started" }
)
TaskUpdate(taskId, status: "in_progress", owner: "vibe")
```

## Pipeline

Run stages sequentially. After each succeeds, update `metadata.vibe_stage` before proceeding.

**Stage numbering `[N/M]`:** M = total stages that will actually run. Base: 5 (branch, explore, prepare, implement, commit). Subtract skipped stages: `--no-branch` → 4, `--dry-run` → 3 (or 2 with both flags). N counts only executed stages, not skipped ones.

### Branch (skip if `--no-branch` or already on non-main branch)

Generate slug: `ck tool slug "<prompt>"`. `Skill("start", args="luan/<slug>")`

**Verify**: `git branch --show-current` returns new branch. **Update**: `vibe_stage: "branch"`

### Explore

`Skill("explore", args="<prompt>")`

**Verify**: `ck plan latest` succeeds. **Update**: `vibe_stage: "explore"`

### Prepare

`Skill("prepare")`

**Verify**: `TaskList()` → epic exists with children and `metadata.slug`. **Update**: `vibe_stage: "prepare"`, `vibe_epic: "<epicId>"`, `vibe_slug: "<slug>"`

If `--dry-run` → stop here. Report plan and epic, suggest `/implement` or `/vibe --continue`.

### Implement

`Skill("implement")`

Note: Acceptance check runs automatically as part of implement teardown.

**Verify**: all children of epic completed. **Update**: `vibe_stage: "implement"`

If some tasks failed, continue to commit if `git diff --stat` is non-empty.

### Commit

If `git diff --stat` is empty → skip.

`Skill("commit")`

**Verify**: `git log -1 --oneline` shows new commit. **Update**: `vibe_stage: "commit"`

## Finalize

```
TaskUpdate(trackerId, status: "completed", metadata: {completedAt: "<ISO 8601>"})
```

Report summary: one line per stage. Mark each as **completed**, **skipped** (excluded by flags like `--no-branch` or `--dry-run`), or **failed** (attempted but errored). Skipped stages were never attempted; failed stages were.

**Limitation:** If vibe fails mid-pipeline, the epic task remains `in_progress`. Orphaned epics are not auto-cleaned — user can resume with `--continue` or manually close the task.

## Error Handling

If ANY stage fails:
1. Do NOT update `vibe_stage` — stays at last successful stage
2. Leave tracker `in_progress`
3. Report completed stages + failure details
4. Suggest: `/vibe --continue` or `/<failed-skill> [args]`
