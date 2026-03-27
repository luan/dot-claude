---
name: vibe
description: "Fully autonomous development workflow from prompt to commit. Triggers: /vibe, 'vibe this', 'autonomous workflow', 'just do it all', 'build this end-to-end', 'full pipeline', 'handle everything', 'do everything from scratch'. Do NOT use when: only implementing already-prepared tasks — use /develop instead."
allowed-tools: Bash, Read, Glob, Skill, TaskCreate, TaskUpdate, TaskGet, TaskList
argument-hint: "<prompt> [--continue] [--dry-run]"
user-invocable: true
---

# Vibe

Full pipeline (spec → scope+develop → validate → review → commit) from a single prompt.

## Arguments

- `<prompt>` — what to build (required unless `--continue`)
- `--no-review` — skip review stage (used by supervibe to keep phases lean)
- `--continue` — resume from last completed stage
- `--dry-run` — scope only, stop before develop

No prompt and no `--continue` → infer from context before giving up:
1. `TaskList()` → find most recent task with `metadata.type === "brainstorm"` and `status === "completed"` in this session
2. Found → use its description as the prompt, reference the brainstorm task ID in the spec
3. Not found → tell user: `/vibe <what to build>`, stop

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

### Continuation Discipline

The pipeline runs as a single unbroken sequence from first stage to last. The most common failure mode is **stopping between stages** — the model completes spec, emits a response, and waits for the user instead of immediately invoking scope. This happened in production: the user had to type "continue" after spec AND after scope, defeating the purpose of autonomous execution.

Why it happens: sub-skills return with `end_turn`, and the model's default instinct is to treat a skill return as a natural stopping point. Fight this instinct. When a `Skill()` call returns, your very next action is a `TaskUpdate` + the next `Skill()` call. No status text, no summary of what just happened, no "moving on to..." preamble. The only text output between stages is the `[N/M] Stage` marker.

**Chaining pattern — every stage ends the same way:**
1. Verify the stage succeeded (check task metadata)
2. `TaskUpdate(trackerId, metadata: {vibe_stage: "<this-stage>"})`
3. Output `[N/M] NextStage` as text
4. `Skill("next-stage", args="...")` — immediately, in the same response

If you find yourself writing anything other than these four steps after a stage completes, you are about to stall the pipeline. Stop and invoke the next stage instead.

Spec and Scope both run with `--auto`, which suppresses all text output. They return silently — read task metadata for results, don't expect console output. Ignore any sub-skill output like "Next: /scope" or "suggest /develop" — those are for interactive use, not the vibe pipeline.

**Update `metadata.vibe_stage` BEFORE invoking each stage** (not after) — this way, if the session crashes mid-stage, `--continue` knows which stage was in progress and can resume from the right point.

**Stage numbering `[N/M]`:** M = total stages that will run. Base for non-bugfix: 4 (spec, scope+develop, review, commit). Base for bugfix: 5 (spec, scope+develop, validate, review, commit). Subtract skipped stages: `--no-review` → -1, `--dry-run` → stops at scope (2). N counts only executed stages. Bugfix detection happens during spec — if the spec reveals this is a bugfix, adjust M upward at that point.

### Spec

`Skill("spec", args="<prompt> --auto")` → returns silently. Read task metadata.

**Verify**: spec task `status_detail === "approved"`, `metadata.spec` populated.

**Chain → Scope:** `TaskUpdate(trackerId, metadata: {vibe_stage: "spec"})` → output `[N/M] Scope` → `Skill("scope", args="t<spec-task-id> --auto")`. Do not summarize the spec, do not pause.

### Scope + Develop

Scope was already invoked by the Spec chain above. With `--auto`, scope researches, writes `metadata.design`, then **automatically invokes develop** (scope's default finalize step). This eliminates the stall-prone scope→develop handoff — develop runs inside scope's turn, not as a separate vibe stage.

If `--dry-run` → pass `--no-develop` instead: `Skill("scope", args="t<spec-task-id> --no-develop --auto")`. Scope returns after planning. Report scope task, suggest `/develop` or `/vibe --continue`.

When scope returns (with develop already completed inside it):

**Verify**: `TaskList()` → all epic children have `status === "completed"`. **Update**: `vibe_stage: "develop"`, `vibe_epic: "<epicId>"`, `vibe_slug: "<slug>"`

**Test-only change red flag (bugfix pipelines):** After develop completes, run `git diff --name-only <base>..HEAD`. If EVERY changed file is a test file (matches `*_test.*`, `*_spec.*`, `test_*.*`, `*/tests/*`, `*/test/*`, `*/__tests__/*`) AND the pipeline is a bugfix (see Bugfix Detection below), the fix is incomplete — tests merely prove what the code does when correctly triggered, not that the production code path works. Do NOT proceed. Report: "Develop produced test-only changes for a bugfix. The tests pass but no production code was changed to fix the bug. Re-run /develop or investigate manually." Update tracker with `status_detail: "test-only-incomplete"`, suggest `/vibe --continue` or `/develop`.

Partial failures: if any child is still `in_progress` or `failed`, the stage is incomplete — report per-child status and suggest `/vibe --continue` or `/develop`. Only proceed to review if all children completed OR incomplete children produced no diff.

**Chain → Validate or Review:** `TaskUpdate(trackerId, metadata: {vibe_stage: "develop"})` → output `[N/M] Validate` or `[N/M] Review` → invoke next stage. Do not summarize scope/develop results, do not pause.

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

**Chain → Commit:** `TaskUpdate(trackerId, metadata: {vibe_stage: "review"})` → output `[N/M] Commit` → invoke Commit. Do not summarize review findings, do not pause.

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
