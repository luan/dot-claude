---
name: rebuild
description: "Collapse branch into working tree and rebuild as clean, refined commits with code improvements per commit. Triggers: 'rebuild branch', 'rebuild commits', 'clean up and improve branch'. Unlike /split-commit which preserves the exact diff, rebuild applies refinements and simplifications to each commit."
argument-hint: "[base-branch] [--test='command'] [--instructions='...']"
user-invocable: true
disable-model-invocation: true
allowed-tools:
  - Task
  - AskUserQuestion
  - Bash
  - Read
  - Edit
---

# Rebuild

Repackage branch as clean commits, refining code in each. End state may differ from original — same functionality, better code.

## Phase 1: Analyze

Parse: `<base-branch>` (default: !`gt parent 2>/dev/null || gt trunk 2>/dev/null || git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's|refs/remotes/||'`), `--test='command'`, `--instructions='...'` (user-specified refinement focus, e.g. "simplify error handling", "remove dead code").

Dispatch analysis subagent (general-purpose):

```
Analyze changes for rebuilding as clean, improved commits.
Base: <base-branch>
Instructions: <--instructions or "general: simplify, remove dead code, improve naming">

1. `git log --oneline <base>..HEAD | wc -l` + `git diff --stat <base>..HEAD | tail -40`
2. Full diff + Read/Grep for context. Design logical commit sequence: foundational → features → cleanup
3. Per commit: files, changes, AND refinement opportunities matching instructions

Dep rules: imports → same commit or dep earlier. Config/locks with introducing feature. Types with first consumer.

Output:
TEST_COMMANDS: <detected or --test>
COMMIT_PLAN:
1. `type(scope): message` — Files: <list>, Changes: <what>, Refinements: <improvements>
```

Present via AskUserQuestion: commit count, tests, each message + files + planned refinements. "Proceed?"

## Phase 2: Execute

After approval, collapse into unstaged changes (soft reset preserves content, second reset unstages so hunks can be re-staged selectively):
```bash
git reset --soft <base> && git reset HEAD
```

Plan declined → ask what to change, re-run analysis. Don't exit silently.

Dispatch one subagent per commit (general-purpose), **sequentially**:

```
Rebuild commit <N>/<total>: `<commit-message>`
Target files: <file list from plan>
Refinements to apply: <from plan>
User instructions: <--instructions or "general refinement">

1. `git-surgeon hunks` — list available hunks with IDs
2. Stage target hunks: whole files → all hunks; partial → `git-surgeon show <id>`, stage matching
3. Read staged files. Apply refinements (simplify, improve naming, remove dead code, apply user instructions). Do NOT change functionality.
4. Stage refinement edits: `git add -u`
5. Run tests: <test-commands>
6. Tests FAIL → check if refinement broke something. Revert that refinement, retry.
   Still failing → find missing dep, stage it, retry once more.
   Still failing → report to main thread, stop.
7. `git commit -m "<message>"`
8. `git diff --stat` — report remaining unstaged + refinements applied
```

After all commits: `git status` (should be clean), `git log --oneline <base>..HEAD`. Dirty tree → cleanup subagent: stage, test, commit as `chore: remaining changes`. Fails → report to user.

## Key Rules

- **Per-commit subagents** — each commit needs Edit for refinements, and fresh context prevents refinement-fix iteration from exhausting the window
- **git-surgeon for staging, Edit for refinements** — git-surgeon provides hunk-level selection without interactive prompts; Edit applies refinements after staging so the original hunks anchor each commit
- **Refinements preserve functionality** — same tests pass, same behavior. Cleaner code, not different code.
- **User instructions override defaults** — if `--instructions` is set, focus refinements there instead of general cleanup
- **Tests gate every commit** — refinement that breaks tests gets reverted, not debugged
- **Sequential** — each commit's working tree depends on the previous
