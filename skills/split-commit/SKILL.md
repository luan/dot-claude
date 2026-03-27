---
name: split-commit
description: "Repackage branch into clean, tested, vertical commits. Triggers: 'split commits', 'repackage commits', 'reorganize commits', 'clean up branch history'. Not for single-commit branches — use /commit instead."
argument-hint: "[base-branch] [--test='command'] [--auto]"
user-invocable: true
allowed-tools:
  - Agent
  - Bash
---

# Split Commit

Repackage branch changes into clean vertical commits. Each commit compiles + passes tests independently.

## Phase 1: Analyze

Parse: `<base-branch>` (default: !`gt parent 2>/dev/null || gt trunk 2>/dev/null || git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's|refs/remotes/||'`), optional `--test='command'`.

**Noop check** — `git log --oneline <base>..HEAD | wc -l`. If ≤1 → stop: "Nothing to repackage — use /commit."

Dispatch analysis subagent (general-purpose):

```
Analyze changes for repackaging into clean vertical commits.
Base branch: <base-branch>

Steps:
1. `git log --oneline <base>..HEAD | wc -l` + `git diff --stat <base>..HEAD | tail -40`
2. Full diff: `git diff <base>..HEAD` — Explore subagent for large diffs
3. Trace cross-file deps, group vertically, order: foundational → features → cleanup
4. For shared files, note which hunks belong where

Auto-detect test commands from justfile/Makefile/package.json/Cargo.toml. Use --test if provided.

Cross-file dep rules:
- File A imports B → same commit or B earlier
- Config/lock files go with the feature introducing the dep
- New types/interfaces go with first consumer

Output:
TEST_COMMANDS: <detected or provided>
COMMIT_PLAN:
1. `type(scope): message` — Files: <list>, Partial: <hunks>, Deps: <justification>, Rationale: <why>
DEPENDENCY_NOTES: <hunk splitting, ordering constraints>
```

`--auto` → proceed directly. Otherwise → present plan via AskUserQuestion: commit count, test commands, each commit + key files. "Proceed?"

## Phase 2: Execute

Collapse into unstaged changes:

```bash
git reset --soft <base> && git reset HEAD
```

Dispatch a **single execution subagent** (model="sonnet") for ALL commits:

```
Create <total> commits from unstaged changes.
Commit plan: <full plan from analysis>
Test commands: <test-commands>

Per commit:
1. `git-surgeon hunks` — list available hunks
2. Stage target hunks: whole files → all hunks; partial → `git-surgeon show <id>`, stage matching
3. Run tests. FAIL → find missing dep, stage, retry once. Still failing → STOP with commit number + error.
4. `git commit -m "<message>"`

After last commit: `git diff --stat` — report remaining unstaged.
```

**On failure**: spawn fix subagent, then re-invoke execution subagent for remaining commits only.

Verify after all commits:

```bash
git status          # should be clean
git log --oneline <base>..HEAD   # N clean commits
```

Dirty tree → cleanup subagent: stage remaining, test, commit as `chore: clean up remaining`. Still fails → report.

## Key Rules

- **No tasks** — pure git operation, one-shot
- **Two subagents** — analysis + execution (fix subagent only on failure)
- **git-surgeon always** — hunk-level precision for partial file staging
- **Every commit compiles** — broken history is worse than plan deviation
- **Plan > rigidity** — test failures from missing deps override grouping
