---
name: split-commit
description: "Collapse a branch into working tree and repackage as clean, tested, vertical commits. Triggers: 'split commits', 'repackage commits', 'reorganize commits', 'clean up branch history', 'consolidate commits into clean ones'"
argument-hint: "<base-branch> [--test='command']"
user-invocable: true
allowed-tools:
  - Task
  - AskUserQuestion
  - Bash
---

# Split Commit

Repackage a branch's worth of changes into clean vertical commits. Each commit compiles and passes tests independently.

## Phase 1: Analyze

Parse arguments: `<base-branch>` (required), optional `--test='command'`.

Dispatch analysis subagent:

```
Task tool with subagent_type="general-purpose" and prompt:
"""
Analyze changes for repackaging into clean, vertical commits.

Base branch: <base-branch>

## Steps
1. Count: `git log --oneline <base>..HEAD | wc -l`
2. Overview: `git diff --stat <base>..HEAD | tail -40`
3. Full diff: `git diff <base>..HEAD` — use Explore subagents for large diffs
4. Trace cross-file dependencies (imports, includes, use/require statements)
5. Group changes into vertical features that can compile independently
6. Order: foundational first (build tools, config), features next, cleanup/polish last
7. For shared files (edited by multiple features), note which hunks belong where

## Auto-detect test commands
Check in order: justfile (`just --list`), Makefile, package.json scripts, Cargo.toml.
Pick the build + lint + test commands. Example: `just build && just lint && just test`
If --test argument provided, use that instead.

## Cross-file dependency rules
- If file A imports/includes file B, both go in same commit or B goes earlier
- Config files (Cargo.toml, package.json) go with the feature that introduces the dependency
- Lock files go with the config change that triggers them
- New types/interfaces go in the commit that first uses them

## Output format
TEST_COMMANDS: <detected or provided>
TOTAL_FILES: <count>
TOTAL_INSERTIONS: <approx>

COMMIT_PLAN:
1. `type(scope): message`
   Files: file1 (whole), file2 (whole), dir/* (all new)
   Partial: file3 (only changes related to X — describe which hunks)
   Rationale: <why grouped>

2. ...

DEPENDENCY_NOTES:
- <files needing hunk-level splitting across commits>
- <ordering constraints>
- <potential test failure risks>
"""
```

Present plan to user with AskUserQuestion:
- Show commit count, test commands, estimated size
- List each commit: message + key files
- Ask: "Proceed with this plan?"

## Phase 2: Execute

After approval, do the soft reset on main thread:
```bash
git reset --soft <base> && git reset HEAD
```

Then dispatch one subagent per commit, **sequentially** (each depends on prior state):

```
Task tool with subagent_type="general-purpose" and prompt:
"""
Create commit <N>/<total>: `<commit-message>`

## Target files
<file list and hunk descriptions from approved plan>

## Steps
1. `git-surgeon hunks` — list all available hunks with IDs
2. Identify hunks belonging to this commit:
   - For whole files: find all hunks for that file, stage all
   - For partial files: `git-surgeon show <id>` to inspect, stage only matching hunks
   - Use `git-surgeon stage <id1> <id2> ...` for staging
   - For line-level precision: `git-surgeon stage <id> --lines X-Y`
3. Verify staged content: `git diff --cached --stat`
4. Run tests: <test-commands>
5. If tests FAIL:
   - Read the error output
   - Identify what's missing (undefined symbol, missing import, missing type)
   - `git-surgeon hunks` to find the hunk that provides it
   - Stage that hunk
   - Re-run tests
   - Repeat until passing — do NOT give up
6. `git commit -m "<message>"`
7. `git diff --stat` — report remaining unstaged changes

## Rules
- ALWAYS use git-surgeon for staging, never plain `git add`
- Every commit MUST pass tests before proceeding
- If a needed hunk isn't in the plan for this commit, stage it anyway — compiling > plan purity
- Report what you staged (file list + hunk count)
"""
```

After all commits, verify on main thread:
```bash
git status          # should be clean
git log --oneline <base>..HEAD   # should show N clean commits
```

If working tree not clean after last commit, dispatch one final subagent to commit remaining files as a cleanup commit.

## Key Rules

- **No beads** — pure git operation, one-shot
- **Subagents for everything** — analysis + each commit in own subagent (context savings)
- **git-surgeon always** — hunk-level precision, never plain git add
- **Every commit compiles** — fix test failures by staging missing deps, never skip
- **Sequential** — commits depend on prior state, cannot parallelize
- **Cross-file deps** — trace imports to keep dependent files together
- **Plan > rigidity** — if tests demand extra files in a commit, include them
