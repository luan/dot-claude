---
name: supervibe
description: "Goal-directed autonomous development — reads the spec, breaks it into chunks, runs each as a vibe cycle. Triggers: /supervibe, 'super vibe', 'multi-phase', 'keep going until done'."
allowed-tools: Bash, Read, Glob, Grep, Agent, Skill, TaskCreate, TaskUpdate, TaskGet, TaskList
argument-hint: "<goal> [--continue] [--max-iterations N]"
user-invocable: true
---

# Super Vibe

Break a goal into independent chunks informed by the spec, run each as a vibe cycle. Structured sequence, not a blind retry loop.

## Arguments

- `<goal>` — what to build (required unless `--continue`)
- `--continue` — resume from tracker metadata
- `--max-iterations N` — hard cap on vibe iterations (default: 5)

## Hard Caps

- **Max iterations**: 5 (override with `--max-iterations`)
- **Per-iteration**: each vibe cycle has its own token budget via `--max-budget-usd`
- Hit any cap → report what was accomplished, what remains, stop.

## [1] Spec

```
Skill("spec", args="<goal> --auto")
```

Read the spec file. The spec's Architecture Context tells you the scope — what modules, files, and patterns are involved.

## [2] Plan Chunks

Read the spec file via `ct spec read <path>`. Break the goal into 2-5 independent chunks based on the spec's Architecture Context. Each chunk:
- Can be implemented and tested independently
- Has a clear "done" state
- Fits in a single vibe cycle

Write the chunk plan as a simple numbered list. Each entry: one sentence describing what to build.

## [3] Execute Chunks

For each chunk, in order:

1. Output `[N/M] <chunk description>`
2. Run `Skill("vibe", args="<chunk prompt> --no-review")`
3. After vibe returns, verify: `git log --oneline -5` and `git diff --stat` to confirm progress
4. If vibe failed with zero progress, record the failure and move to the next chunk (don't retry the same chunk — the approach needs adjusting)

After all chunks complete, run one final review:
```
Skill("review")
Skill("commit")
```

## [4] Report

```
Supervibe: <goal>
Chunks: N/M completed
Spec: <file path>
```

If max iterations hit before completion, report what remains.

## Resume (`--continue`)

Find tracker: `TaskList()` → `metadata.super_vibe === true`, `status === "in_progress"`. Read the spec file path and chunk plan from metadata. Resume from the next unfinished chunk.

## Error Handling

- Chunk fails → record it, continue to next chunk. Failed chunks noted in final report.
- Max iterations hit → stop, report accomplished vs remaining.
- All chunks fail → stop, suggest `/debugging` or manual investigation.
