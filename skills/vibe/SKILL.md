---
name: vibe
description: "Fully autonomous development workflow from prompt to commit. Chains spec → develop → review → commit. Triggers: /vibe, 'vibe this', 'autonomous workflow', 'just do it all', 'build this end-to-end', 'full pipeline', 'handle everything'."
allowed-tools: Bash, Read, Glob, Skill, TaskCreate, TaskUpdate, TaskGet, TaskList
argument-hint: "<prompt> [--continue] [--dry-run]"
user-invocable: true
---

# Vibe

Full pipeline (spec → develop → review → commit) from a single prompt.

## Arguments

- `<prompt>` — what to build (required unless `--continue`)
- `--no-review` — skip review stage
- `--continue` — resume from last completed stage
- `--dry-run` — spec only, stop before develop

No prompt and no `--continue` → infer from context:
1. `TaskList()` → find most recent task with `metadata.type === "brainstorm"` and `status === "completed"`
2. Found → use its description as the prompt
3. Not found → tell user: `/vibe <what to build>`, stop

## Resume (`--continue`)

1. `TaskList()` → find task with `metadata.vibe_stage` present and `status == "in_progress"`
2. Read `metadata.vibe_stage` for resume point, `metadata.vibe_prompt` as prompt
3. Skip to the stage after `vibe_stage`

## Pipeline

Create a tracker task:
```
TaskCreate(
  subject: "Vibe: <prompt (truncated 60 chars)>",
  metadata: { type: "vibe", vibe_prompt: "<full prompt>", vibe_stage: "started" }
)
TaskUpdate(taskId, status: "in_progress")
```

**Stage numbering `[N/M]`:** M = total stages that will run. Base: 4 (spec, develop, review, commit). `--no-review` → 3. `--dry-run` → 1.

### [1/M] Spec

Update `vibe_stage: "spec"`, output `[1/M] Spec`.

```
Skill("spec", args="<prompt> --auto")
```

Spec runs silently with `--auto` and returns a file path. Read the spec file path from the output or find via `ct spec latest`.

Verify the spec file exists and has content. Immediately proceed to develop.

### [2/M] Develop

Update `vibe_stage: "develop"`, output `[2/M] Develop`.

If `--dry-run` → stop here. Report the spec file path, suggest `/develop <path>` or `/vibe --continue`.

```
Skill("develop", args="<spec-file-path> --auto")
```

Verify all workers completed. If some failed, report per-worker status and suggest `/vibe --continue`.

**Bugfix detection:** If the spec mentions "bug", "fix", "regression", or includes reproduction steps — after develop completes, re-run the reproduction scenario to confirm the fix works. If reproduction still fails, report and stop (do not proceed to review with a broken fix).

Immediately proceed to review.

### [3/M] Review

Update `vibe_stage: "review"`, output `[3/M] Review`.

Skip if `--no-review`.

```
Skill("review")
```

Fix any critical issues inline. Immediately proceed to commit.

### [4/M] Commit

Update `vibe_stage: "commit"`, output `[4/M] Commit`.

If `git diff --stat` is empty → skip.

```
Skill("commit")
```

## Finalize

```
TaskUpdate(trackerId, status: "completed")
```

Report: one line per stage (completed / skipped / failed).

## Error Handling

If a stage fails with zero progress:
1. Keep `vibe_stage` at last successful stage (so `--continue` resumes correctly)
2. Leave tracker in_progress
3. Report completed stages + failure details
4. Suggest: `/vibe --continue` or `/<failed-skill> [args]`
