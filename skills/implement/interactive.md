## Mode: Interactive

Human in loop. Ask questions, get approval.

## Flags

- `--fresh`: Summarize progress, get approval, continue

## Steps

1. Find plan:
   - arg → `bd show <arg>`
   - no arg → `bd list --status in_progress` (resume)
   - still nothing → `bd ready` (next unblocked)
   - No issue? → "Nothing to do. Run /explore first"
2. **Create/verify branch** (BEFORE any implementation):
   - Check if already on feature branch for this issue
   - If not: `gt create luan/<issue-id>-<short-description> --no-interactive`
   - Verify with `git branch --show-current`
3. Handle previous session context FIRST (resolve pending from notes)
4. If --fresh: Use `EnterPlanMode`, summarize completed/remaining, get approval
5. Mark in_progress: `bd update <id> --status in_progress`
6. Open questions? Use `AskUserQuestion` FIRST
7. FOR EACH TASK (MANDATORY subagent dispatch):
   - DOCS FIRST: Update docs if new functionality
   - Task tool → implementer subagent (TDD, outside-in)
   - Task tool → spec reviewer → fix if issues
   - Task tool → quality reviewer → fix if issues
   - Update beads notes with progress
   - If task fails: use Skill tool to invoke `debugging`
8. After FIRST task: Run build/check for early verification
9. Use Skill tool to invoke `verification-before-completion`
10. Use Skill tool to invoke `critical-review`
    - If changes exceed scope: commit current, note remainder
11. Update notes: COMPLETED + commit hash, decisions, NEXT tasks
12. PR-sized commits: 150-400 lines
13. Context stale? Suggest `/implement --fresh`
14. Final: `bd close <id>` + use Skill tool to invoke `finishing-branch`

**Hard stop:** One PR-ready commit per session.
