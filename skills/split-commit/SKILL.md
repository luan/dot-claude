---
name: split-commit
description: "Collapse a branch into working tree and repackage as clean, tested, vertical commits. Triggers: 'split commits', 'repackage commits', 'reorganize commits', 'clean up branch history', 'consolidate commits into clean ones'. Do NOT use when: branch already has a single clean commit or only needs amending — use /commit instead."
argument-hint: "[base-branch] [--test='command']"
user-invocable: true
allowed-tools:
  - Task
  - AskUserQuestion
  - Bash
---

# Split Commit

Repackage branch changes into clean vertical commits. Each commit compiles + passes tests independently.

## Phase 1: Analyze

Parse: `<base-branch>` (default: !`gt parent 2>/dev/null || gt trunk 2>/dev/null || git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's|refs/remotes/||'`), optional `--test='command'`.

Dispatch analysis subagent (general-purpose):

```
Analyze changes for repackaging into clean vertical commits.
Base branch: <base-branch>

Steps:
1. `git log --oneline <base>..HEAD | wc -l` + `git diff --stat <base>..HEAD | tail -40`
2. Full diff: `git diff <base>..HEAD` — Explore subagents for large diffs; Read/Grep when diff context is insufficient
3. Trace cross-file deps, group into vertical features, order: foundational → features → cleanup
4. For shared files, note which hunks belong where

Auto-detect test commands from justfile, Makefile, package.json, Cargo.toml. Use --test if provided.

Cross-file dep rules:
- File A imports B → same commit or B earlier
- Config/lock files go with the feature introducing the dep
- New types/interfaces go with first consumer

Output:
TEST_COMMANDS: <detected or provided>
COMMIT_PLAN:
1. `type(scope): message` — Files: <list>, Partial: <hunks>, Rationale: <why>
DEPENDENCY_NOTES: <hunk splitting, ordering constraints>
```

Present plan via AskUserQuestion: commit count, test commands, each commit message + key files. "Proceed?"

## Phase 2: Execute

After approval, collapse all commits into unstaged changes (soft reset preserves content, second reset unstages so git-surgeon can re-stage hunks selectively):
```bash
git reset --soft <base> && git reset HEAD
```

Dispatch a **single execution subagent** (model="sonnet") for ALL commits — eliminates per-commit spawn latency:

```
Create <total> commits from unstaged changes.
Commit plan: <full plan from analysis>
Test commands: <test-commands>

Per commit:
1. `git-surgeon hunks` — list available hunks
2. Stage target hunks: whole files → all hunks; partial → `git-surgeon show <id>`, stage matching
3. Run tests. FAIL → find missing dep, stage, retry once. Still failing → STOP, return with commit number + error.
4. `git commit -m "<message>"`

After last commit: `git diff --stat` — report remaining unstaged.
```

**On failure mid-sequence**: main thread spawns a dedicated fix subagent (fresh context for iteration). After fix, re-invoke execution subagent for remaining commits.

After all commits, verify:
```bash
git status          # should be clean
git log --oneline <base>..HEAD   # N clean commits
```

Dirty tree → dispatch cleanup subagent: stage remaining, run tests, commit as `chore: clean up remaining changes`. Cleanup also fails → report to user rather than forcing a broken commit.

## Key Rules

- **No tasks** — pure git operation, one-shot
- **Two subagents** — analysis + single execution subagent for all commits (spawn dedicated fix subagent only on failure)
- **git-surgeon always** — hunk-level precision avoids accidentally staging unrelated changes when files are partially targeted
- **Every commit compiles** — a broken commit in history is worse than a plan deviation
- **Sequential** — each commit's working tree depends on what the previous commit staged, so parallelizing would create conflicts
- **Plan > rigidity** — test failures from missing deps override the original grouping plan
