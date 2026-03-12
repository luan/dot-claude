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
- `--no-review` — skip review stage (used by supervibe to keep phases lean)
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

**CRITICAL: Run ALL stages in one continuous turn with zero stops.** The entire pipeline executes autonomously — never pause, never ask, never suggest next steps, never wait for user input between stages. After each stage completes, update `vibe_stage` and invoke the next `Skill()` call in the same response. If a sub-skill outputs suggestions like "Next: /scope" or "suggest /develop", ignore that text completely and proceed to the next stage. A common failure mode is ending the turn after Branch — the model creates the branch, outputs a status line, and stops. This is wrong. Branch is a silent setup step; after it completes you must continue to Scope in the same turn without ending the response.

Before each stage, output `[N/M] Stage` as text BEFORE the `Skill()` call. After each succeeds, update `metadata.vibe_stage` and immediately invoke next.

**Stage numbering `[N/M]`:** M = total stages that will run. Base: 6 (branch, scope, develop, simplify, review, commit). Subtract skipped stages: `--no-branch` → 5, `--no-review` → 5, `--dry-run` → 2. Combine flags to subtract more. N counts only executed stages.

### Branch (skip if `--no-branch` or already on non-main branch)

**NEVER call `Skill("start")`** — it creates a task frame that halts the pipeline after branch creation (observed bug: model creates branch, outputs status, stops). Inline instead:

1. Generate slug: `ct tool slug "<prompt>"`
2. Create branch: `Skill(gt:gt, "create <username>/<slug>")`
3. Link tracker: `TaskUpdate(trackerId, metadata: {branch: "<branch-name>"})`

**After branch creation, DO NOT end your response.** Immediately continue: **Update** `vibe_stage: "branch"`, output `[N/M] Scope`, call `Skill("scope", ...)`. No status report, no pause.

### Scope

`Skill("scope", args="<prompt> --no-develop --auto")`

`--auto` skips both spec and plan review gates — scope auto-approves both artifacts instead of halting twice for feedback.

**Verify**: scope task with `status_detail === "approved"`, `metadata.spec` and `metadata.design` populated. **Update**: `vibe_stage: "scope"`

If `--dry-run` → stop. Report scope task, suggest `/develop` or `/vibe --continue`.

**Then immediately invoke Develop** — do not output scope results, do not pause.

### Develop

`Skill("develop")`

Acceptance check runs automatically as part of develop teardown.

**Verify**: `TaskList()` → all epic children have `status === "completed"`. **Update**: `vibe_stage: "develop"`, `vibe_epic: "<epicId>"`, `vibe_slug: "<slug>"`

Partial failures: if any child is still `in_progress` or `failed`, the stage is incomplete — report per-child status and suggest `/vibe --continue` or `/develop`. Only proceed to simplify if all children completed OR incomplete children produced no diff.

**Then immediately invoke Simplify.**

### Simplify

`Skill("simplify")`

Reviews changed code for reuse, quality, and efficiency, then fixes issues.

**Update**: `vibe_stage: "simplify"` — **then immediately invoke Review.**

### Review (skip if `--no-review`)

`Skill("review")`

Adversarial code review. Fix any surfaced issues inline before proceeding.

**Update**: `vibe_stage: "review"` — **then immediately invoke Commit.**

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
