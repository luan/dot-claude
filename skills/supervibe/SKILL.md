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

**Extract end-state vision**: From `metadata.spec`, extract the Recommendation section — this describes the target system. Store as `metadata.end_state` in the tracker. This is the north star for every phase.

**Extract phases**: Read `metadata.design`, identify phase sections (headers like `# Phase N:`). Store phase titles and goals as an ordered list in `metadata.phases`.

**Assess total scope**: Estimate the overall project size from the plan (number of files touched, systems involved, complexity). Store as `metadata.scope_estimate` (e.g., "small: ~5 files", "medium: ~15 files across 3 modules", "large: ~30+ files across multiple systems"). This determines whether 2, 3, 4, or 5 PRs is right — not a fixed number.

**Validate phase plan**:
1. Phases collectively cover the full end-state (every capability maps to at least one phase)
2. Final phase, once complete, fully realizes the end-state
3. Each phase is a vertical slice (not a single-layer change)
4. Phase count is ≤5 — if scope produced more, consolidate related phases
5. Phases are roughly balanced in size (no phase should be 3x larger than another)

If gaps, imbalance, or >5 phases: re-invoke scope with specific feedback. Don't proceed with an incomplete or poorly-sized plan.

Mark original scope task `status: "completed"`.

**Update**: `vibe_stage: "scope"`

If `--dry-run` → stop. Report end-state, phases with sizing assessment, suggest `/supervibe --continue`.

### [2] Watchdog Setup

```
CronCreate(
  schedule: "*/20 * * * *",
  prompt: "/supervibe --continue",
  recurring: true
)
```

Store cron job ID in `metadata.cron_id`.

**Update**: `vibe_stage: "watchdog"`

→ **Begin phase execution.**

### [3..N+2] Per-Phase Loop

For each phase in order:

1. **Lock**: `TaskUpdate(trackerId, metadata: { active_phase: <N>, locked_at: "<ISO 8601>" })` — prevents watchdog from re-entering the same phase. `--continue` checks: if `active_phase` exists and `locked_at` is <30 min ago, the session is still working — exit without action.
2. **Branch**: `Skill("gt:gt", args="create <slug>-p<N>")` — stacks on previous branch
3. **Re-scope**: `Skill("scope", args="Phase <N> of <original prompt>: <phase title and goal>. Context: phase <N> of <total> in a stacked plan. Prior phases landed: <list completed phase titles>. End-state vision: <metadata.end_state>. Spec: <original spec summary>. This phase should be a vertical slice producing one reviewable PR — balance with the other <total> phases. --auto")`
4. **Develop**: `Skill("develop")`
5. **Acceptance**: `Skill("acceptance", args="--auto")` — verifies against phase criteria. Auto-fixes up to 2 iterations on PARTIAL/FAIL.
6. **Build check**: Run the project's build command. Detect build system from repo root: `Makefile` → `make build`, `package.json` with build script → `npm run build`, `Cargo.toml` → `cargo build`, `go.mod` → `go build ./...`, else skip. **Broken build = stop.** Don't stack broken code.
7. **PR size check**: `git diff HEAD~1 --stat | tail -1` — log the diff size to `metadata.phase_sizes`. After phase 2+, compare against previous phases. Flag if a phase is drastically smaller or larger than its siblings (>3x difference) — suggests unbalanced scoping. Warnings logged to tracker, not hard stops.
8. **Simplify**: `Skill("simplify")`
9. **Review**: `Skill("review")` — fix issues inline
10. **Commit**: `Skill("commit")`
11. **Progress check**: Compare completed phases against `metadata.end_state`. Store `metadata.progress` as a list of end-state capabilities realized so far. If this is the last phase and any end-state capability is unrealized → don't submit. Report the gap, suggest re-scoping the missing work as additional phases.
12. **Unlock + update**: `TaskUpdate(trackerId, metadata: { active_phase: null, locked_at: null, vibe_stage: "phase-<N>" })`

→ **Proceed to next phase.**

If a phase fails at any step: stop, leave `active_phase` set (watchdog will retry after lock expires), report per-phase status with which step failed, leave tracker `in_progress`. Suggest `/supervibe --continue`.

### [N+3] Teardown & Submit

`CronDelete(metadata.cron_id)`

**Final progress gate**: Read `metadata.progress`. If any end-state capability is unrealized across all completed phases → report gaps, do NOT submit. Suggest adding phases for missing work.

`Skill("gt:submit")`

**Update**: `vibe_stage: "submitted"`

## Resume (`--continue`)

Find tracker with `metadata.super_vibe === true` and `status === "in_progress"`. If multiple: filter by `metadata.session_id`. Ignore other sessions.

**Lock check**: If `metadata.active_phase` is set and `metadata.locked_at` is <30 min ago → another session is actively working. Exit: "Pipeline is active (phase <N>, locked <time> ago). Wait or run with `--force` to override."

Read `vibe_stage`:
- `"scope"` or `"watchdog"` → set up watchdog if missing, begin phase loop from phase 1. Read phases from `metadata.phases`, end-state from `metadata.end_state`.
- `"phase-<N>"` → resume at phase N+1

If no in-progress tracker → tell user no pipeline to resume, stop.

## Finalize

```
TaskUpdate(trackerId, status: "completed", metadata: {completedAt: "<ISO 8601>"})
```

Report: one line per phase (**completed** / **skipped** / **failed**), end-state coverage assessment, stack URL.

## Error Handling

- Don't update `vibe_stage` on failure — preserves resume point
- Leave tracker `in_progress` with `active_phase` set
- Report completed stages + failure details + which step failed
- Suggest `/supervibe --continue`
- Watchdog retries after lock expires (30 min)
