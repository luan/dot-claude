---
name: vibe
description: "Fully autonomous development workflow from prompt to commit. Chains spec → develop → review → commit. Triggers: /vibe, 'vibe this', 'autonomous workflow', 'just do it all', 'build this end-to-end', 'full pipeline', 'handle everything'."
allowed-tools: Bash, Read, Glob, Skill
argument-hint: "<prompt> [--dry-run]"
user-invocable: true
---

# Vibe

Full pipeline (spec → develop → review → commit) from a single prompt.

## Arguments

- `<prompt>` — what to build (required)
- `--no-review` — skip review stage
- `--dry-run` — spec only, stop before develop

No prompt → tell user: `/vibe <what to build>`, stop.

## Pipeline

**Stage numbering `[N/M]`:** M = total stages that will run. Base: 4 (spec, develop, review, commit). `--no-review` → 3. `--dry-run` → 1.

### [1/M] Spec

Output `[1/M] Spec`.

```
Skill("spec", args="<prompt> --auto")
```

Spec runs silently with `--auto` and returns a file path. Read the spec file path from the output or find via `ct spec latest`.

Verify the spec file exists and has content. Immediately proceed to develop.

### [2/M] Develop

Output `[2/M] Develop`.

If `--dry-run` → stop here. Report the spec file path, suggest `/develop <path>`.

```
Skill("develop", args="<spec-file-path> --auto")
```

Verify all workers completed. If some failed, report per-worker status and stop.

**Bugfix detection:** If the spec mentions "bug", "fix", "regression", or includes reproduction steps — after develop completes, re-run the reproduction scenario to confirm the fix works. If reproduction still fails, report and stop (do not proceed to review with a broken fix).

Immediately proceed to review.

### [3/M] Review

Output `[3/M] Review`.

Skip if `--no-review`.

```
Skill("review")
```

Fix any critical issues inline. Immediately proceed to commit.

### [4/M] Commit

Output `[4/M] Commit`.

If `git diff --stat` is empty → skip.

```
Skill("commit")
```

## Finalize

Report: one line per stage (completed / skipped / failed).

## Error Handling

If a stage fails with zero progress:
1. Report completed stages + failure details
2. Suggest: `/<failed-skill> [args]` to retry the failed stage
