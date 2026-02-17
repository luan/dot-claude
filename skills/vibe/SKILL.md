---
name: vibe
description: "Fully autonomous development workflow from prompt to commit. Chains explore → prepare → implement → commit. Triggers: /vibe, 'vibe this', 'autonomous workflow'."
allowed-tools: Bash, Read, Glob, Skill, TaskCreate, TaskUpdate, TaskGet, TaskList
argument-hint: "<prompt> [--no-branch] [--continue] [--dry-run]"
user-invocable: true
---

# Vibe

Run the full pipeline from a single prompt.

## Arguments

- `<prompt>` — what to build (required unless `--continue`)
- `--no-branch` — skip branch creation, use current branch
- `--continue` — resume from last completed stage
- `--dry-run` — explore + prepare only, stop before implement

If no prompt and no `--continue` → tell user:
`/vibe <what to build>`, stop.

## Resume (`--continue`)

1. `TaskList()` → find task with `metadata.label == "vibe"` and
   `status == "in_progress"`
2. Read `metadata.vibe_stage` for resume point, `metadata.vibe_prompt`
   as prompt
3. Skip to the stage after `vibe_stage`
4. Not found → tell user no pipeline to resume, stop

## Fresh Start

```
TaskCreate(
  subject: "Vibe: <prompt (truncated 60 chars)>",
  description: "<full prompt>",
  activeForm: "Vibing",
  metadata: { label: "vibe", vibe_prompt: "<full prompt>", vibe_stage: "started" }
)
TaskUpdate(taskId, status: "in_progress")
```

## Pipeline

Run stages sequentially. After each succeeds, update
`metadata.vibe_stage` before proceeding.

### Branch (skip if `--no-branch` or already on non-main branch)

Generate slug: `claude-slug "<prompt>"` (outputs kebab-case, max 50 chars).

```
Skill("start", args="luan/<slug>")
```

**Verify**: `git branch --show-current` returns new branch.
**Update**: `vibe_stage: "branch"`

### Explore

```
Skill("explore", args="<prompt>")
```

**Verify**: `claude-planfile latest` succeeds (plan file exists).
**Update**: `vibe_stage: "explore"`

### Prepare

```
Skill("prepare")
```

**Verify**: `TaskList()` → epic task exists with children.
**Update**: `vibe_stage: "prepare"`, `vibe_epic: "<epicId>"`

If `--dry-run` → stop here. Report plan and epic, suggest
`/implement` or `/vibe --continue`.

### Implement

```
Skill("implement")
```

**Verify**: all children of epic completed.
**Update**: `vibe_stage: "implement"`

If some tasks failed, continue to commit if `git diff --stat`
is non-empty.

### Commit

If `git diff --stat` is empty → skip.

```
Skill("commit")
```

**Verify**: `git log -1 --oneline` shows new commit.
**Update**: `vibe_stage: "commit"`

## Finalize

```
TaskUpdate(trackerId, status: "completed")
```

Report summary with one line per stage showing what happened.

## Error Handling

If ANY stage fails:

1. Do NOT update `vibe_stage` (stays at last successful stage)
2. Leave tracker in_progress
3. Report completed stages + failure details
4. Suggest: `/vibe --continue` or `/<failed-skill> [args]`

## Stage Numbering

Adjust `[N/M]` denominator based on flags:
- All stages: 5
- `--no-branch`: 4
- `--dry-run`: 3 (or 2 with `--no-branch`)
