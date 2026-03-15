---
name: supervibe
description: "Goal-directed autonomous development — iterates /vibe until a goal is met, shipping PRs as branches grow. Triggers: /supervibe, 'super vibe', 'stacked vibe', 'multi-phase plan', 'multi-PR plan'. Do NOT use when: the plan fits in a single PR — use /vibe instead."
allowed-tools: Bash, Read, Glob, Grep, Agent, Skill, TaskCreate, TaskUpdate, TaskGet, TaskList
argument-hint: "<goal> [--continue]"
user-invocable: true
---

# Super Vibe

Goal-directed loop around `/vibe`. Each iteration: assess where we are, decide the next increment, vibe it, check if we're done. No predetermined phase count — the loop discovers the right shape as it goes.

Ship a PR when the branch is reviewable. Start a new branch and keep going if the goal isn't met.

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
    iterations: [],
    prs: []
  }
)
TaskUpdate(epicId, status: "in_progress", owner: "supervibe")
```

## [2] Initial Research

Lightweight scope — understand the codebase enough to plan the first increment. NOT a full decomposition into N phases.

```
Skill("scope", args="<goal>. Research the codebase and produce a spec + plan. The plan should describe the FULL scope of work but I will implement it incrementally — focus the plan on what to build first. --auto")
```

Store on epic:
```
metadata.end_state = "<spec Recommendation — present-tense target state>"
metadata.research = "<key findings: file locations, patterns, architecture>"
```

Mark scope task completed.

## [3] Loop

**REPEAT THE FOLLOWING STEPS UNTIL `metadata.goal_met === true`.**

### Step A: Read the goal

```
TaskGet(epicId)
```

Read `metadata.end_state`, `metadata.iterations[]`, `metadata.prs[]`. Re-ground yourself on what the goal is and what's been done.

### Step B: Assess

Look at the current state of the codebase relative to the goal:

```bash
git log --oneline -20
git diff --stat HEAD~<commits_since_start>  # scope of all changes
```

For each capability described in `metadata.end_state`, check: does it exist in the codebase now? Read key files if needed — don't guess.

**If goal is met**: `TaskUpdate(epicId, metadata: {goal_met: true})` → go to Teardown.

### Step C: Branch check

Count commits on current branch since the last PR (or since start). If the branch has **5+ commits** or **touches 15+ files**, it's big enough to review:

1. `Skill("commit")` if there are uncommitted changes
2. `Skill("gt:submit")` to ship the PR
3. Record: `metadata.prs.push({branch, url, summary})`
4. Create new branch inline: `gt bc -m "supervibe-continues-<N>"`

### Step D: Plan next increment

Based on the assessment (Step B), decide the **single most valuable next step** toward the goal. Consider:
- What's already built (from `metadata.iterations[]`)
- What's missing (from the assessment)
- What has the most dependencies downstream (do it first)

Write a focused prompt for vibe — one increment, not the whole remaining plan.

### Step E: Execute

```
Skill("vibe", args="<increment prompt> --no-branch")
```

Vibe runs its full pipeline (scope → develop → review → commit) on the current branch.

### Step F: Record

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

1. Ship final PR if current branch has uncommitted/unsubmitted work
2. Report: goal, PRs shipped (with URLs), iteration count, files changed
3. `TaskUpdate(epicId, status: "completed", metadata: {completedAt: "<ISO 8601>"})`

## Resume (`--continue`)

Find epic: `super_vibe === true`, `status === "in_progress"`. No match → tell user, stop.

Read `metadata.goal`, `metadata.end_state`, `metadata.research`, `metadata.iterations`, `metadata.prs`.

Enter the loop at **Step A**.

## Key Rules

- Vibe completing ≠ supervibe complete. ALWAYS assess after vibe returns.
- No predetermined phase count. Iterate until done.
- Ship PRs when branches get big. Don't accumulate 50 commits.
- Each iteration re-reads the goal from epic metadata. Never rely on conversation memory.
- Failed vibe? Record what happened, adjust the next increment, keep going. Don't stop on first failure.
