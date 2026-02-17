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

Parse: `<base-branch>` (default: !`gt parent 2>/dev/null || gt trunk`), optional `--test='command'`.

Dispatch analysis subagent (general-purpose):

```
Analyze changes for repackaging into clean vertical commits.
Base branch: <base-branch>

## Steps
1. Count: `git log --oneline <base>..HEAD | wc -l`
2. Overview: `git diff --stat <base>..HEAD | tail -40`
3. Full diff: `git diff <base>..HEAD` — Explore subagents for large diffs
4. Trace cross-file deps (imports, includes, use/require)
5. Group into vertical features that compile independently
6. Order: foundational first (build, config), features next, cleanup last
7. For shared files (multi-feature edits), note which hunks belong where

## Auto-detect test commands
Check: justfile, Makefile, package.json, Cargo.toml.
Pick build + lint + test commands.
If --test provided, use that instead.

## Cross-file dependency rules
- File A imports B → same commit or B earlier
- Config files (Cargo.toml, package.json) go with feature introducing dep
- Lock files go with config change triggering them
- New types/interfaces go with first consumer

## Output
TEST_COMMANDS: <detected or provided>
TOTAL_FILES: <count>
TOTAL_INSERTIONS: <approx>

COMMIT_PLAN:
1. `type(scope): message`
   Files: file1 (whole), file2 (whole), dir/* (all new)
   Partial: file3 (only X-related hunks — describe)
   Rationale: <why grouped>
2. ...

DEPENDENCY_NOTES:
- <hunk-level splitting needs>
- <ordering constraints>
- <test failure risks>
```

Present plan via AskUserQuestion:
- Commit count, test commands, estimated size
- Each commit: message + key files
- "Proceed with this plan?"

## Phase 2: Execute

After approval, soft reset on main thread:
```bash
git reset --soft <base> && git reset HEAD
```

Dispatch one subagent per commit (model: "sonnet"), **sequentially**:

```
Create commit <N>/<total>: `<commit-message>`

## Target files
<file list + hunk descriptions from plan>

## Steps
1. `git-surgeon hunks` — list available hunks with IDs
2. Identify hunks for this commit:
   - Whole files: find all hunks, stage all
   - Partial: `git-surgeon show <id>`, stage matching only
   - `git-surgeon stage <id1> <id2> ...`
   - Line precision: `git-surgeon stage <id> --lines X-Y`
3. Verify: `git diff --cached --stat`
4. Run tests: <test-commands>
5. Tests FAIL:
   - Read error → find missing dep → `git-surgeon hunks` → stage it → retry
   - Repeat until passing — do NOT give up
6. `git commit -m "<message>"`
7. `git diff --stat` — report remaining unstaged
```

After all commits, verify on main thread:
```bash
git status          # should be clean
git log --oneline <base>..HEAD   # N clean commits
```

Dirty tree after last commit → dispatch final sonnet subagent:
```
Create cleanup commit for remaining unstaged changes.
1. `git-surgeon hunks` — list remaining hunks
2. `git-surgeon stage <all-ids>`
3. Run tests: <test-commands>
4. `git commit -m "chore: clean up remaining changes"`
5. `git diff --stat` — confirm clean
```

## Key Rules

- **No tasks** — pure git operation, one-shot
- **Subagents for everything** — analysis + each commit in own subagent
- **git-surgeon always** — hunk-level precision, never plain git add
- **Every commit compiles** — fix test failures by staging missing deps
- **Sequential** — commits depend on prior state, cannot parallelize
- **Cross-file deps** — trace imports to keep dependent files together
- **Plan > rigidity** — tests demand extra files → include them
