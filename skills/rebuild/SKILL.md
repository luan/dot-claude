---
name: rebuild
description: "Collapse branch into working tree and rebuild as clean, refined commits with code improvements per commit. Triggers: 'rebuild branch', 'rebuild commits', 'clean up and improve branch'. Unlike /split-commit which preserves the exact diff, rebuild applies refinements and simplifications to each commit."
argument-hint: "[base-branch] [--test='command'] [--instructions='...'] [--auto]"
user-invocable: true
disable-model-invocation: true
allowed-tools:
  - Task
  - Bash
  - Read
  - Edit
---

# Rebuild

Repackage branch as clean commits with code refinements. End state: same functionality, better code.

## Phase 1: Analyze

Parse: `<base-branch>` (default: !`gt parent 2>/dev/null || gt trunk 2>/dev/null || git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's|refs/remotes/||'`), `--test='command'`, `--instructions='...'` (refinement focus, e.g. "simplify error handling", "remove dead code").

Dispatch analysis subagent:

```
Analyze changes for rebuilding as clean, improved commits.
Base: <base-branch>
Instructions: <--instructions or "general: simplify, remove dead code, improve naming">

1. `git log --oneline <base>..HEAD | wc -l` + `git diff --stat <base>..HEAD | tail -40`
2. Full diff + Read/Grep for context. Design logical sequence: foundational → features → cleanup
3. Per commit: files, changes, refinement opportunities matching instructions

Dep rules: imports with same commit or earlier. Config/locks with introducing feature. Types with first consumer.

Output:
TEST_COMMANDS: <detected or --test>
COMMIT_PLAN:
1. `type(scope): message` — Files: <list>, Changes: <what>, Refinements: <concrete before/after>
```

`--auto` → proceed directly. Without `--auto` → AskUserQuestion: commit count, tests, each message + files + refinements. "Proceed?"

## Phase 2: Execute

Collapse into unstaged changes (soft reset preserves content, second reset unstages for selective re-staging):

```bash
git reset --soft <base> && git reset HEAD
```

Plan declined → ask what to change, re-analyze. Don't exit silently.

Dispatch one subagent per commit, **sequentially** (each depends on previous working tree):

```
Rebuild commit <N>/<total>: `<commit-message>`
Target files: <file list from plan>
Refinements: <from plan>
Instructions: <--instructions or "general refinement">

1. `git-surgeon hunks` — list available hunks
2. Stage target hunks: whole files → all hunks; partial → `git-surgeon show <id>`, stage matching
3. Read staged files. Apply refinements (simplify, rename, remove dead code). Do NOT change functionality.
4. `git add -u` for refinement edits
5. Run tests: <test-commands>
6. Tests FAIL → identify which refinement broke it, revert ONLY that edit. Still failing → find missing dep, stage, retry once. Still failing → report, stop.
7. `git commit -m "<message>"`
8. `git diff --stat` — report remaining unstaged + refinements applied
```

After all commits: `git status` (should be clean), `git log --oneline <base>..HEAD`. Dirty tree → cleanup subagent: stage, test, commit as `chore: remaining changes`. Fails → report.

## Key Rules

- **Per-commit subagents** — fresh context prevents window exhaustion
- **git-surgeon for staging, Edit for refinements** — hunk-level selection without interactive prompts; Edit applies after staging so original hunks anchor each commit
- **Refinements preserve functionality** — same tests, same behavior, cleaner code
- **`--instructions` overrides defaults** — focus refinements on user's specified areas
- **Tests gate every commit** — broken refinement gets reverted, not debugged
- **Sequential execution** — each commit depends on the previous
