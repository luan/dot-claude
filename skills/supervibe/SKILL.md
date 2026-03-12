---
name: supervibe
description: "Execute a large plan across multiple commits — one commit per phase, sequential on the current branch. Uses /vibe as the operator for each phase. Triggers: /supervibe, 'super vibe', 'stacked vibe', 'multi-phase plan', 'one commit per phase', 'multi-PR plan'. Do NOT use when: the plan fits in a single PR — use /vibe instead."
allowed-tools: Bash, Read, Glob, Grep, Agent, Skill, TaskCreate, TaskUpdate, TaskGet, TaskList, CronCreate, CronDelete, CronList
argument-hint: "<prompt> [--dry-run] [--continue]"
user-invocable: true
---

# Super Vibe

Large plans across multiple commits. Each phase runs `/vibe` as a subagent on the current branch — one squashed commit per phase, verified before the next begins. The orchestrator stays lean: it reads task metadata, dispatches phases, and records results. All implementation happens inside phase agents.

No worktrees, no merge commits. Phases run sequentially on the same branch, each producing a single clean commit.

## Arguments

- `<prompt>` — what to build (required unless `--continue`)
- `--dry-run` — superscope only, stop before execution
- `--continue` — resume from last completed phase

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

### [1] Superscope

Read `${CLAUDE_SKILL_DIR}/references/superscope.md` and follow it. This is not a regular `/scope` call — it's a dedicated multi-phase research and planning session that produces richer output.

Produces on the tracker:
- `metadata.end_state` — north star from spec Recommendation (present-tense target state)
- `metadata.phases[]` — ordered list, each with: `{ title, goal, files: {read, modify, create}, dependencies, verification }`
- `metadata.superscope_findings` — key research findings (file locations, patterns, architecture context) that warm-start per-phase scope calls so they don't re-research from scratch

Validate:
1. Phases collectively cover the full end-state (every capability maps to a phase)
2. Each phase is a vertical slice (not a single-layer change like "just the DB schema")
3. Phase count ≤5 — consolidate if more
4. Phases roughly balanced in size (no phase >3× another)

Re-invoke scope with specific feedback if any check fails.

Mark scope task `status: "completed"`. **Update**: `vibe_stage: "scoped"`

If `--dry-run` → stop. Report end-state, phase plan with sizing. Suggest `/supervibe --continue`.

### [2] Watchdog Setup

```
CronCreate(schedule: "*/20 * * * *", prompt: "/supervibe --continue", recurring: true)
```

Store cron job ID in `metadata.cron_id`. **Update**: `vibe_stage: "watchdog"` → Begin phase execution.

### [3..N+2] Per-Phase Loop

For each phase in `metadata.phases`:

#### 1. Assemble phase context

Build a rich prompt for the phase agent from task metadata:

```
Phase <N>/<total>: <phase title>

Goal: <phase goal>

File plan:
- Read: <files to read for context>
- Modify: <files to modify>
- Create: <files to create>

Prior phase results:
<for each completed phase in metadata.phase_results:>
  Phase <M>: <summary>. Files: <files_changed>. Deviations: <deviations>.

End-state vision: <metadata.end_state>

Research context (from superscope — use for scope warm-start, don't re-research these):
<metadata.superscope_findings>
```

#### 2. Dispatch phase agent

```
Agent(
  prompt: "Run /vibe with this context: <assembled prompt>. Use flags: --no-branch --no-review --no-commit. Do NOT create any commits — leave all changes staged or unstaged. The orchestrator handles committing.",
  mode: "auto"
)
```

The agent works directly on the current branch. Vibe runs: scope (warm-started by the rich context) → develop → simplify. No commit, no branch creation — the orchestrator squash-commits the result.

#### 3. On success (agent completes without error)

Squash all phase changes into a single commit:

```bash
git add -A
git commit -m "phase(<N>): <phase title>"
```

Record phase results on tracker:
```
metadata.phase_results[N] = {
  phase: N,
  status: "completed",
  commit: "<commit SHA>",
  files_changed: [<from git diff --stat HEAD~1>],
  summary: "<from vibe task's completion data or git log>",
  deviations: "<any divergence from original phase plan>"
}
```

**Update**: `vibe_stage: "phase-<N>"` → proceed to next phase.

#### 4. On failure (agent errors or returns no changes)

Discard uncommitted changes to restore a clean state:

```bash
git checkout -- . && git clean -fd
```

Record:
```
metadata.phase_results[N] = {
  phase: N,
  status: "failed",
  step: "<which step failed>",
  error: "<error details>"
}
```

Do NOT update `vibe_stage` — stays at last successful phase so `--continue` retries the failed phase.

Report which step in which phase failed. Suggest `/supervibe --continue`.

### [N+3] Teardown

`CronDelete(metadata.cron_id)`

**End-state coverage check**: Compare `metadata.phase_results` against `metadata.end_state`. Walk each capability in the end-state and verify it maps to a completed phase. Unrealized capabilities → report gaps, suggest additional phases, do NOT auto-complete.

If all capabilities realized:

```
TaskUpdate(trackerId, status: "completed", metadata: {completedAt: "<ISO 8601>"})
```

Report: one line per phase (**completed** / **failed**), end-state coverage assessment.

## Resume (`--continue`)

Find tracker: `super_vibe === true`, `status === "in_progress"`. Multiple → filter by `session_id`. No match → tell user no pipeline to resume, stop.

Read `metadata.phase_results[]`, `metadata.phases`, `metadata.end_state`, `metadata.superscope_findings`.

Resume logic:
- No phases started → begin phase 1
- Last phase completed → start next phase
- Last phase failed → retry failed phase. The phase agent gets the error context from `phase_results[N].error` — include it in the prompt so scope/develop can account for the prior failure.
- All phases completed → go to teardown

The watchdog cron fires `--continue` every 20 minutes. This handles session crashes: the next cron invocation picks up where the failed session left off, with full context from task metadata.

## Error Handling

- Don't update `vibe_stage` on failure — preserves resume point
- Failed phases: uncommitted changes are discarded, branch stays at last good commit
- Report which step in which phase failed
- Suggest `/supervibe --continue`
- Watchdog cron retries automatically after session crashes
