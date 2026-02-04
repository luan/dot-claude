## Mode: Non-Interactive

No human in loop. Make best-judgment decisions, document assumptions.

## Steps

1. Find or create issue:
   - arg is issue-id → refine that issue
   - no arg → `bd list --status in_progress` or recent open
   - existing? → load notes, consume session context FIRST
   - no existing? → fresh exploration
2. Explore via Task tool (subagent_type=Explore)
3. Verify state (parallel subagents): implemented vs claimed, specs accurate
4. Design: pick best approach based on codebase patterns, document trade-offs
5. Persist to beads:
   - New: `bd create "<slug>" --type task`
   - Existing: `bd update <id> --notes "..."`
   - Use writing-plans format
   - Tasks: PR-sized (150-400 lines each), exact paths, complete code
6. Output issue-id and exit (stop hook continues if more work)

**Hard stop:** Plan only. Do NOT implement.
