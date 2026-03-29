---
name: vibe
description: "Fully autonomous development workflow from prompt to commit. Triggers: /vibe, 'vibe this', 'autonomous workflow', 'just do it all', 'build this end-to-end', 'full pipeline', 'handle everything', 'do everything from scratch'. Do NOT use when: only implementing already-prepared tasks — use /develop instead."
allowed-tools: Bash, Read, Glob, Skill, TaskCreate, TaskUpdate, TaskGet, TaskList
argument-hint: "<prompt> [--no-branch] [--continue] [--dry-run]"
user-invocable: true
---

# Vibe

Full pipeline (spec → scope → develop → validate → review → commit) from a single prompt.

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

**CRITICAL: Run ALL stages in one continuous turn with zero stops.** The pipeline is fully autonomous — never pause, ask, suggest, or wait between stages. After each stage: update `vibe_stage`, output `[N/M] NextStage`, invoke next `Skill()`. Ignore any sub-skill output like "Next: /scope" or "suggest /develop" — those are for interactive use, not the vibe pipeline.

Spec and Scope both run with `--auto`, which suppresses all text output. They return silently — read task metadata for results, don't expect console output.

Before each stage, output `[N/M] Stage` as text BEFORE the `Skill()` call. **Update `metadata.vibe_stage` BEFORE invoking each stage** (not after) — this way, if the session crashes mid-stage, `--continue` knows which stage was in progress and can resume from the right point. After the stage succeeds, immediately invoke next.

**Stage numbering `[N/M]`:** M = total stages that will run. Base for non-bugfix: 6 (branch, spec, scope, develop, review, commit). Base for bugfix: 7 (branch, spec, scope, develop, validate, review, commit). Subtract skipped stages: `--no-branch` → -1, `--no-review` → -1, `--dry-run` → stops at scope (3). Combine flags to subtract more. N counts only executed stages. Bugfix detection happens during spec — if the spec reveals this is a bugfix, adjust M upward at that point.

### Branch (skip if `--no-branch` or already on non-main branch)

**NEVER call `Skill("start")`** — it creates a task frame that halts the pipeline after branch creation (observed bug: model creates branch, outputs status, stops). Inline instead:

1. Generate slug: `ct tool slug "<prompt>"`
2. Create branch: `Skill(gt:gt, "create !`echo "${GIT_USERNAME:-$(whoami)}"`/<slug>")`
3. Link tracker: `TaskUpdate(trackerId, metadata: {branch: "<branch-name>"})`

**DO NOT end your response after branch creation.** Update `vibe_stage: "branch"`, output `[N/M] Spec`, call `Skill("spec", ...)`. No status report, no pause.

### Spec

`Skill("spec", args="<prompt> --auto")` → returns silently. Read task metadata.

**Verify**: spec task `status_detail === "approved"`, `metadata.spec` populated. **Update**: `vibe_stage: "spec"` → invoke Scope.

### Scope

`Skill("scope", args="t<spec-task-id> --no-develop --auto")` → returns silently. Read task metadata.

**Verify**: scope task `status_detail === "approved"`, `metadata.design` populated. **Update**: `vibe_stage: "scope"`

If `--dry-run` → stop. Report scope task, suggest `/develop` or `/vibe --continue`.

Otherwise → invoke Develop.

### Develop

`Skill("develop")`

Acceptance check runs automatically as part of develop teardown.

**Verify**: `TaskList()` → all epic children have `status === "completed"`. **Update**: `vibe_stage: "develop"`, `vibe_epic: "<epicId>"`, `vibe_slug: "<slug>"`

**Test-only change red flag (bugfix pipelines):** After develop completes, run `git diff --name-only <base>..HEAD`. If EVERY changed file is a test file (matches `*_test.*`, `*_spec.*`, `test_*.*`, `*/tests/*`, `*/test/*`, `*/__tests__/*`) AND the pipeline is a bugfix (see Bugfix Detection below), the fix is incomplete — tests merely prove what the code does when correctly triggered, not that the production code path works. Do NOT proceed. Report: "Develop produced test-only changes for a bugfix. The tests pass but no production code was changed to fix the bug. Re-run /develop or investigate manually." Update tracker with `status_detail: "test-only-incomplete"`, suggest `/vibe --continue` or `/develop`.

Partial failures: if any child is still `in_progress` or `failed`, the stage is incomplete — report per-child status and suggest `/vibe --continue` or `/develop`. Only proceed to review if all children completed OR incomplete children produced no diff.

**Then immediately invoke Validate (bugfix) or Review (non-bugfix).**

### Validate (bugfix pipelines only — skip for non-bugfix)

**Gate between develop and review for bugfix pipelines.** Re-runs the reproduction scenario to confirm the bug is actually fixed in production, not just tested.

1. **Bugfix Detection:** The pipeline is a bugfix when ANY of these are true:
   - The prompt or spec contains words like "bug", "fix", "broken", "regression", "not working", "fails when", "incorrect", "wrong"
   - `metadata.spec` or `metadata.design` references reproduction steps, error output, or expected-vs-actual behavior
   - The triage source (`metadata.type === "triage"`) classified the item as `bug`
   - A diagnostic skill (e.g., `/dia-inspect-data`, `/debugging`) was invoked earlier in the session or referenced in task metadata

2. **Find reproduction steps:** Check in order: spec's reproduction section, triage task description, scope design, vibe prompt. Extract the concrete command or steps that demonstrate the bug.

3. **Re-run reproduction:** Execute the reproduction steps (or the closest automated equivalent). Compare output against the expected behavior from the spec.

4. **Gate:**
   - Bug is fixed (output matches expected) → **Update**: `vibe_stage: "validate"` → invoke Review.
   - Bug persists → Do NOT proceed. Report: "Validation failed — reproduction still shows the bug after develop. The implementation changed code but did not fix the root cause." Include the actual output vs expected. Update tracker with `status_detail: "validation-failed"`. Suggest: re-run `/develop` with more context, or investigate with `/debugging`.

**Update**: `vibe_stage: "validate"` → invoke Review.

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

### Tracker Cleanup

The vibe tracker must reflect reality. Update it when the pipeline diverges from the normal flow:

- **User redirects** (user invokes a different skill, asks to do something else, or abandons the pipeline): `TaskUpdate(trackerId, status: "cancelled", metadata: {vibe_stage: "<last completed>", cancelled_reason: "user redirect"})`.
- **Validation gate fails** (bugfix not actually fixed): leave `in_progress` with `status_detail: "validation-failed"` so `--continue` can resume after manual investigation.
- **Test-only incomplete** (bugfix produced only test changes): leave `in_progress` with `status_detail: "test-only-incomplete"`.
- **Repeated failures** (same stage fails 2+ times across `--continue` attempts): `TaskUpdate(trackerId, status: "blocked", metadata: {blocked_reason: "<stage> failed repeatedly"})`.

Never leave a tracker `in_progress` with no path to completion. If the pipeline cannot continue, the tracker status must say why.
