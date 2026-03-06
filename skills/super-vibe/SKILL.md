---
name: super-vibe
description: "Execute a large plan across multiple stacked PRs — one PR per phase. Triggers: /super-vibe, 'super vibe', 'stacked vibe', 'multi-branch plan', 'stacked PRs', 'one PR per phase'. Do NOT use when: the plan fits in a single PR — use /vibe instead."
allowed-tools: Bash, Read, Glob, Skill, TaskCreate, TaskUpdate, TaskGet, TaskList
argument-hint: "<prompt> [--dry-run]"
user-invocable: true
---

# Super Vibe

Initial scope produces the phase plan. Each phase re-scopes against the actual codebase before executing, so later phases adapt to what was really built. One PR per phase, submitted as a Graphite stack.

## Arguments

- `<prompt>` — what to build (required)
- `--dry-run` — scope only, stop before execution

## Tracker

```
TaskCreate(
  subject: "Super Vibe: <prompt (60 chars)>",
  activeForm: "Super Vibing",
  metadata: { type: "epic", vibe_prompt: "<prompt>", vibe_stage: "started", super_vibe: true, session_id: "424c8b3a-c116-4af4-acc9-3bcf312910c5" }
)
TaskUpdate(taskId, status: "in_progress", owner: "super-vibe")
```

## Pipeline

Run all stages in one continuous turn — **never pause between phases.**

### [1] Initial Scope

`Skill("scope", args="<prompt> --auto")`

**Verify**: scope task with `status_detail === "approved"`, `metadata.spec` and `metadata.design` populated.

Read `metadata.design` from the scope task and identify phase sections (headers like `# Phase N:`). Store phase titles and high-level goals as an ordered list in tracker metadata (`metadata.phases`). Then mark the original scope task `status: "completed"`.

**Update**: `vibe_stage: "scope"`

If `--dry-run` → stop. Report phases, suggest `/super-vibe --continue`.

→ **Immediately begin phase execution.**

### [2..N+1] Per-Phase Loop

For each phase in order:

1. **Branch**: `Skill("gt:gt", args="create <slug>-p<N>")` — stacks on previous branch
2. **Re-scope**: `Skill("scope", args="Phase <N> of <original prompt>: <phase title and goal>. Context: this is phase <N> of <total> in a stacked plan. Prior phases already landed: <list completed phase titles>. Spec from initial scope: <original spec summary>. --auto")` — scope researches the current codebase (which now includes prior phases' code) and produces a design grounded in reality.
3. **Develop**: `Skill("develop")` — detects the phase scope task, creates tasks, executes. Develop marks the phase scope task completed when done.
4. **Simplify**: `Skill("simplify")`
5. **Review**: `Skill("review")` — fix issues inline
6. **Commit**: `Skill("commit")`
7. **Update**: `vibe_stage: "phase-<N>"`

→ **Immediately proceed to next phase.** No pause, no summary.

If a phase fails: stop, report per-phase status (one line per phase: **completed** / **failed**), leave tracker `in_progress`. Suggest `/super-vibe --continue`.

### [N+2] Submit

`Skill("gt:submit")`

**Update**: `vibe_stage: "submitted"`

## Resume (`--continue`)

Find tracker with `metadata.super_vibe === true` and `status === "in_progress"`. If multiple matches: filter by `metadata.session_id` matching the current session. Ignore trackers from other sessions.

Read `vibe_stage`:
- `"scope"` → skip to phase loop, read phases from `metadata.phases`
- `"phase-<N>"` → resume at phase N+1 (e.g., `"phase-3"` → skip to phase 4)

Phase list is stored in `metadata.phases` after initial scope, so resume doesn't need the original scope task.

## Finalize

```
TaskUpdate(trackerId, status: "completed", metadata: {completedAt: "<ISO 8601>"})
```

Report: one line per phase (**completed** / **skipped** / **failed**), stack URL from gt:submit.

## Error Handling

Same as /vibe: don't update `vibe_stage` on failure, leave tracker `in_progress`, report + suggest recovery.
