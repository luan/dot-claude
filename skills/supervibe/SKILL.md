---
name: supervibe
description: "Goal-directed autonomous development — iterates /vibe until a goal is met. Triggers: /supervibe, 'super vibe', 'multi-phase', 'keep going until done'. Do NOT use when: the work fits in a single /vibe iteration."
allowed-tools: Bash, Read, Glob, Grep, Agent, Skill, TaskCreate, TaskUpdate, TaskGet, TaskList
argument-hint: "<goal> [--continue]"
user-invocable: true
---

# Super Vibe

Goal-directed loop around `/vibe`. Each iteration: assess where we are, decide the next increment, vibe it, check if we're done. No predetermined phase count — the loop discovers the right shape as it goes.

## YOU ARE IN A LOOP

After every `/vibe` call returns, you **MUST** execute the assess step. Do NOT stop, summarize, or mark the epic complete until the assess step confirms the goal is met. Vibe completing its task does NOT mean your goal is met — vibe only knows about its increment, not your end-state.

## Arguments

- `<goal>` — what to build (required unless `--continue`)
- `--continue` — resume from epic metadata

## [1] Setup

```
TaskCreate(
  subject: "Supervibe: <goal (60 chars)>",
  activeForm: "Supervibing",
  metadata: {
    type: "epic",
    super_vibe: true,
    goal: "<full goal text>",
    goal_met: false,
    iterations: []
  }
)
TaskUpdate(epicId, status: "in_progress", owner: "supervibe")
```

## [2] Initial Research

Define the target state — what does "done" look like? NOT a full decomposition into phases.

```
Skill("spec", args="<goal> --auto")
```

Store on epic:
```
metadata.end_state = "<spec target state — present-tense description of the system as if already built>"
metadata.research = "<key findings from spec investigation: file locations, patterns, architecture>"
```

Mark spec task completed.

## [3] Loop

**REPEAT THE FOLLOWING STEPS UNTIL `metadata.goal_met === true`.**

### Step A: Read the goal

```
TaskGet(epicId)
```

Read `metadata.end_state`, `metadata.iterations[]`. Re-ground yourself on what the goal is and what's been done.

### Step B: Assess

Look at the current state of the codebase relative to the goal:

```bash
git log --oneline -20
git diff --stat HEAD~<commits_since_start>  # scope of all changes
```

For each capability described in `metadata.end_state`, check: does it exist in the codebase now? Read key files if needed — don't guess.

**If goal is met**: `TaskUpdate(epicId, metadata: {goal_met: true})` → go to Teardown.

### Step C: Plan next increment

Based on the assessment (Step B), decide the **single most valuable next step** toward the goal. Consider:
- What's already built (from `metadata.iterations[]`)
- What's missing (from the assessment)
- What has the most dependencies downstream (do it first)

Write a focused prompt for vibe — one increment, not the whole remaining plan.

### Step D: Execute

```
Skill("vibe", args="<increment prompt>")
```

Vibe runs its full pipeline (spec → scope → develop → review → commit) on the current branch.

### Step E: Record

After vibe returns, read what it did:

```bash
git log --oneline -5
git diff --stat HEAD~<vibe's commits>
```

```
metadata.iterations.push({
  n: <iteration number>,
  prompt: "<what was asked>",
  commits: [<SHAs>],
  files_changed: [<paths>],
  summary: "<what actually happened>",
  deviations: "<anything unexpected>"
})
TaskUpdate(epicId, metadata: {iterations: <updated>})
```

**GO TO STEP A.**

## Teardown

All capabilities in `metadata.end_state` are realized.

1. `Skill("commit")` if there are uncommitted changes
2. Report: goal, iteration count, files changed
3. `TaskUpdate(epicId, status: "completed", metadata: {completedAt: "<ISO 8601>"})`

## Resume (`--continue`)

Find epic: `super_vibe === true`, `status === "in_progress"`. No match → tell user, stop.

Read `metadata.goal`, `metadata.end_state`, `metadata.research`, `metadata.iterations`.

Enter the loop at **Step A**.

## Key Rules

- Vibe completing ≠ supervibe complete. ALWAYS assess after vibe returns.
- No predetermined phase count. Iterate until done.
- Each iteration re-reads the goal from epic metadata. Never rely on conversation memory.
- Failed vibe? Record what happened, adjust the next increment, keep going.
