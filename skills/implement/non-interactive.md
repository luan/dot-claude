## Mode: Non-Interactive

No human in loop. Make best-judgment decisions, document in notes.

## Steps

1. Find plan:
   - arg → `bd show <arg>`
   - no arg → `bd list --status in_progress` (resume)
   - still nothing → `bd ready` (next unblocked)
   - No issue? → exit (stop hook handles)
2. Handle previous session context FIRST (resolve pending from notes)
3. Mark in_progress: `bd update <id> --status in_progress`
4. Open questions? Make best-judgment call, document assumption in notes
5. FOR EACH TASK (MANDATORY subagent dispatch):
   - DOCS FIRST: Update docs if new functionality
   - Task tool → implementer subagent (TDD, outside-in)
   - Task tool → spec reviewer → fix if issues
   - Task tool → quality reviewer → fix if issues
   - Update beads notes with progress
   - If task fails: use Skill tool to invoke `debugging`
6. After FIRST task: Run build/check for early verification
7. Use Skill tool to invoke `verification-before-completion`
8. Use Skill tool to invoke `critical-review`
   - If changes exceed scope: commit current, note remainder
9. Update notes: COMPLETED + commit hash, decisions, NEXT tasks
10. PR-sized commits: 150-400 lines
11. Final: `bd close <id>` + use Skill tool to invoke `finishing-branch --pr`

Stop hook continues with next issue if more work exists.
