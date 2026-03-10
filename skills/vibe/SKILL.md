---
name: vibe
description: "Fully autonomous development workflow from prompt to commit. Triggers: /vibe, 'vibe this', 'autonomous workflow', 'just do it all', 'build this end-to-end', 'full pipeline', 'handle everything', 'do everything from scratch'. Do NOT use when: only implementing already-prepared tasks — use /develop instead."
allowed-tools: Bash, Read, Glob, Skill, TaskCreate, TaskUpdate, TaskGet, TaskList
argument-hint: "<prompt> [--no-branch] [--continue] [--dry-run]"
user-invocable: true
---

# Vibe

Full pipeline (scope → develop → simplify → review → commit) from a single prompt.

## Arguments

- `<prompt>` — what to build (required unless `--continue`)
- `--no-branch` — skip branch creation, use current branch
- `--continue` — resume from last completed stage
- `--dry-run` — scope only, stop before develop

No prompt and no `--continue` → tell user: `/vibe <what to build>`, stop.

## Resume (`--continue`)

1. `TaskList()` → find task with `metadata.vibe_stage` present and `status == "in_progress"`
2. Multiple matches → filter by `metadata.session_id` matching current session. Ignore other sessions.
3. Read `metadata.vibe_stage` for resume point, `metadata.vibe_prompt` as prompt
4. Skip to the stage after `vibe_stage`
5. Not found → tell user no pipeline to resume, stop

## Fresh Start

```
TaskCreate(
  subject: "Vibe: <prompt (truncated 60 chars)>",
  description: "<full prompt>",
  activeForm: "Vibing",
  metadata: { type: "epic", priority: "P2", vibe_prompt: "<full prompt>", vibe_stage: "started", session_id: "${CLAUDE_SESSION_ID}" }
)
TaskUpdate(taskId, status: "in_progress", owner: "vibe")
```

## Pipeline

Run stages sequentially in one continuous turn — the whole point of /vibe is zero user intervention. Before each stage, output `[N/M] Stage` as text BEFORE the `Skill()` call. After each succeeds, update `metadata.vibe_stage` and immediately invoke next.

**Stage numbering `[N/M]`:** M = total stages that will run. Base: 6 (branch, scope, develop, simplify, review, commit). Subtract skipped stages: `--no-branch` → 5, `--dry-run` → 2 (or 1 with both flags). N counts only executed stages.

### Branch (skip if `--no-branch` or already on non-main branch)

Generate slug: `ct tool slug "<prompt>"`. `Skill("start", args="` !`echo "${GIT_USERNAME:-$(whoami)}"` `/<slug> <trackerId>")`

Pass the tracker task ID as second arg so `/start` links to it instead of creating a new task.

**Verify**: `git branch --show-current` returns new branch. **Update**: `vibe_stage: "branch"`

→ Immediately invoke Scope. Ignore any suggestions from `/start`.

### Scope

`Skill("scope", args="<prompt> --no-develop --auto")`

`--auto` skips both spec and plan review gates — scope auto-approves both artifacts instead of halting twice for feedback.

**Verify**: scope task with `status_detail === "approved"`, `metadata.spec` and `metadata.design` populated. **Update**: `vibe_stage: "scope"`

If `--dry-run` → stop. Report scope task, suggest `/develop` or `/vibe --continue`.

→ Immediately invoke Develop. Do not output scope results or pause.

### Develop

`Skill("develop")`

Acceptance check runs automatically as part of develop teardown.

**Verify**: `TaskList()` → all epic children have `status === "completed"`. **Update**: `vibe_stage: "develop"`, `vibe_epic: "<epicId>"`, `vibe_slug: "<slug>"`

Partial failures: if any child is still `in_progress` or `failed`, the stage is incomplete — report per-child status and suggest `/vibe --continue` or `/develop`. Only proceed to simplify if all children completed OR incomplete children produced no diff.

→ Immediately invoke Simplify.

### Simplify

`Skill("simplify")`

Reviews changed code for reuse, quality, and efficiency, then fixes issues.

**Update**: `vibe_stage: "simplify"` → Immediately invoke Review.

### Review

`Skill("review")`

Adversarial code review. Fix any surfaced issues inline before proceeding.

**Update**: `vibe_stage: "review"` → Immediately invoke Commit.

### Commit

If `git diff --stat` is empty → skip.

`Skill("commit")`

**Verify**: `git log -1 --oneline` shows new commit. **Update**: `vibe_stage: "commit"`

## Finalize

```
TaskUpdate(trackerId, status: "completed", metadata: {completedAt: "<ISO 8601>"})
```

Report summary: one line per stage (**completed** / **skipped** / **failed**).

## Error Handling

If a stage completely fails (skill errors out, zero progress):
1. Do NOT update `vibe_stage` — stays at last successful stage so `--continue` resumes correctly
2. Leave tracker `in_progress`
3. Report completed stages + failure details
4. Suggest: `/vibe --continue` or `/<failed-skill> [args]`
