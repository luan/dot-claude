---
name: split-commit
description: "Collapse a branch into working tree and repackage as clean, tested, vertical commits. Triggers: 'split commits', 'repackage commits', 'reorganize commits', 'clean up branch history', 'consolidate commits into clean ones'"
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

Dispatch analysis subagent (general-purpose — has Read/Grep/Glob for file context beyond the diff):

```
Analyze changes for repackaging into clean vertical commits.
Base branch: <base-branch>

Steps:
1. Count: `git log --oneline <base>..HEAD | wc -l`
2. Overview: `git diff --stat <base>..HEAD | tail -40`
3. Full diff: `git diff <base>..HEAD` — use Explore subagents for large diffs
4. Read files with Read/Grep when diff context is insufficient
5. Trace cross-file deps (imports, includes, use/require)
6. Group into vertical features that compile independently
7. Order: foundational first, features next, cleanup last
8. For shared files, note which hunks belong where

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

After approval, soft reset on main thread:
```bash
git reset --soft <base> && git reset HEAD
```

Dispatch one subagent per commit (model="sonnet"), **sequentially** — each depends on prior state:

```
Create commit <N>/<total>: `<commit-message>`
Target files: <file list + hunk descriptions from plan>

1. `git-surgeon hunks` — list available hunks with IDs
2. Stage target hunks: whole files → all hunks; partial → `git-surgeon show <id>`, stage matching; line precision → `git-surgeon stage <id> --lines X-Y`
3. Verify: `git diff --cached --stat`
4. Run tests: <test-commands>
5. Tests FAIL → read error, find missing dep, stage it, retry until passing
6. Commit FAILS (hook rejection, etc.) → read error, fix, retry once. Still failing → report to main thread, stop.
7. `git commit -m "<message>"`
8. `git diff --stat` — report remaining unstaged
```

After all commits, verify:
```bash
git status          # should be clean
git log --oneline <base>..HEAD   # N clean commits
```

Dirty tree after last commit → dispatch cleanup subagent: stage remaining, run tests, commit as `chore: clean up remaining changes`. If cleanup also fails tests, report remaining changes to user rather than forcing a broken commit.

## Key Rules

- **No tasks** — pure git operation, one-shot
- **Subagents for everything** — analysis + each commit in own subagent
- **git-surgeon always** — hunk-level precision, never plain git add
- **Every commit compiles** — fix test failures by staging missing deps
- **Sequential** — commits depend on prior state, cannot parallelize
- **Plan > rigidity** — tests demand extra files → include them
