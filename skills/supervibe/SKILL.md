---
name: supervibe
description: "Execute a large plan across multiple stacked PRs — one PR per phase. Triggers: /supervibe, 'super vibe', 'stacked vibe', 'multi-branch plan', 'stacked PRs', 'one PR per phase'. Do NOT use when: the plan fits in a single PR — use /vibe instead."
allowed-tools: Bash, Read, Glob, Skill, TaskCreate, TaskUpdate, TaskGet, TaskList, CronCreate, CronDelete, CronList
argument-hint: "<prompt> [--dry-run]"
user-invocable: true
---

# Super Vibe

Large plans across stacked PRs. Each phase produces a working, right-sized PR — verified before the next begins.

## Arguments

- `<prompt>` — what to build (required)
- `--dry-run` — scope + validate, stop before execution

## Tracker

```
TaskCreate(
  subject: "Super Vibe: <prompt (60 chars)>",
  activeForm: "Super Vibing",
  metadata: { type: "epic", vibe_prompt: "<prompt>", vibe_stage: "started", super_vibe: true, session_id: "${CLAUDE_SESSION_ID}" }
)
TaskUpdate(taskId, status: "in_progress", owner: "supervibe")
```

## Pipeline

### [1] Initial Scope

`Skill("scope", args="<prompt>. IMPORTANT — this is a supervibe stacked-PR plan. Each phase becomes its own PR. Design phases as vertical slices that each produce a working, independently reviewable PR. Avoid micro-phases (just a model, just a route) and mega-phases (entire features). Maximum 5 PRs per supervibe run. --auto")`

**Verify**: scope task with `status_detail === "approved"`, `metadata.spec` and `metadata.design` populated.

**Post-scope extraction** (all stored on the tracker):
- **End-state vision** (`metadata.end_state`): from `metadata.spec` Recommendation section — the north star for every phase
- **Phases** (`metadata.phases`): ordered list of phase titles and goals from `metadata.design`
- **Scope estimate** (`metadata.scope_estimate`): project size estimate (files, modules, complexity) — determines whether 2-5 PRs is right

**Validate phase plan** — re-invoke scope with specific feedback if any fail:
1. Phases collectively cover the full end-state (every capability maps to a phase)
2. Each phase is a vertical slice (not a single-layer change)
3. Phase count ≤5 — consolidate if more
4. Phases roughly balanced in size (no phase >3x another)

Mark original scope task `status: "completed"`. **Update**: `vibe_stage: "scope"`

If `--dry-run` → stop. Report end-state, phases with sizing, suggest `/supervibe --continue`.

### [2] Watchdog Setup

```
CronCreate(schedule: "*/20 * * * *", prompt: "/supervibe --continue", recurring: true)
```

Store cron job ID in `metadata.cron_id`. **Update**: `vibe_stage: "watchdog"` → Begin phase execution.

### [3..N+2] Per-Phase Loop

For each phase in order:

1. **Lock**: `TaskUpdate(trackerId, metadata: { active_phase: <N>, locked_at: "<ISO 8601>" })` — prevents watchdog re-entry. Lock valid for 30 min.
2. **Branch**: `Skill("gt:gt", args="create <slug>-p<N>")` — stacks on previous branch
3. **Re-scope**: `Skill("scope", args="Phase <N> of <original prompt>: <phase title and goal>. Context: phase <N> of <total> in a stacked plan. Prior phases landed: <list completed phase titles>. End-state vision: <metadata.end_state>. Spec: <original spec summary>. Vertical slice producing one reviewable PR — balance with the other <total> phases. --auto")`
4. **Develop**: `Skill("develop")`
5. **Acceptance**: `Skill("acceptance", args="--auto")` — auto-fixes up to 2 iterations on PARTIAL/FAIL
6. **Build check**: Detect build system (`Makefile` → `make build`, `package.json` → `npm run build`, `Cargo.toml` → `cargo build`, `go.mod` → `go build ./...`, else skip). **Broken build = stop.** Don't stack broken code.
7. **PR size check**: Log diff size to `metadata.phase_sizes`. After phase 2+, flag >3x difference vs siblings (warning, not hard stop).
8. **Simplify**: `Skill("simplify")`
9. **Review**: `Skill("review")` — fix issues inline
10. **Commit**: `Skill("commit")`
11. **Progress check**: Compare completed phases against `metadata.end_state`. Store `metadata.progress` (capabilities realized so far). Last phase + unrealized capability → don't submit, report gap, suggest additional phases.
12. **Unlock + update**: `TaskUpdate(trackerId, metadata: { active_phase: null, locked_at: null, vibe_stage: "phase-<N>" })`

→ Proceed to next phase.

Phase failure: stop, leave `active_phase` set (watchdog retries after lock expires), report which step failed, leave tracker `in_progress`. Suggest `/supervibe --continue`.

### [N+3] Teardown & Submit

`CronDelete(metadata.cron_id)`

**Final progress gate**: Read `metadata.progress`. If any end-state capability is unrealized across all completed phases → report gaps, do NOT submit. Suggest adding phases for missing work.

`Skill("gt:submit")`

**Update**: `vibe_stage: "submitted"`

## Resume (`--continue`)

Find tracker with `metadata.super_vibe === true` and `status === "in_progress"`. Multiple → filter by `metadata.session_id`. No match → tell user no pipeline to resume, stop.

**Lock check**: `active_phase` set + `locked_at` <30 min ago → exit: "Pipeline active (phase N, locked X ago). Wait or use `--force`."

Resume by `vibe_stage`:
- `"scope"` / `"watchdog"` → set up watchdog if missing, begin phase 1. Read from `metadata.phases` and `metadata.end_state`.
- `"phase-<N>"` → resume at phase N+1

## Finalize

```
TaskUpdate(trackerId, status: "completed", metadata: {completedAt: "<ISO 8601>"})
```

Report: one line per phase (**completed** / **skipped** / **failed**), end-state coverage assessment, stack URL.

## Error Handling

- Don't update `vibe_stage` on failure — preserves resume point
- Leave tracker `in_progress` with `active_phase` set
- Report completed stages + which step failed
- Suggest `/supervibe --continue`
- Watchdog retries after lock expires (30 min)
