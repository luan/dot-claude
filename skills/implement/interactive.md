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
2. Handle previous session context FIRST (resolve pending from notes)
3. If --fresh: Use `EnterPlanMode`, summarize completed/remaining, get approval
4. Mark in_progress: `bd update <id> --status in_progress`
5. Open questions? Use `AskUserQuestion` FIRST
6. FOR EACH TASK (MANDATORY subagent dispatch):
   - DOCS FIRST: Update docs if new functionality
   - Task tool → implementer subagent (TDD, outside-in)
   - Task tool → spec reviewer → fix if issues
   - Task tool → quality reviewer → fix if issues
   - Update beads notes with progress
   - If task fails: use Skill tool to invoke `debugging`
7. After FIRST task: Run build/check for early verification
8. Use Skill tool to invoke `verification-before-completion`
9. Use Skill tool to invoke `critical-review`
   - If changes exceed scope: commit current, note remainder
10. Update notes: COMPLETED + commit hash, decisions, NEXT tasks
11. PR-sized commits: 150-400 lines
12. Context stale? Suggest `/implement --fresh`
13. Final: `bd close <id>` + use Skill tool to invoke `finishing-branch`

**Hard stop:** One PR-ready commit per session.
