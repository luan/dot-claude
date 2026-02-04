## Mode: Interactive

Human in loop. Ask questions, get approval.

## Steps

1. Find or create issue:
   - arg is issue-id → refine that issue
   - no arg → `bd list --status in_progress` or recent open
   - existing? → load notes, consume session context FIRST
   - no existing? → fresh exploration
2. Use `EnterPlanMode` tool
3. Explore via Task tool (subagent_type=Explore)
4. Verify state (parallel subagents): implemented vs claimed, specs accurate
5. Spec hygiene (if specs exist):
   - Audit: duplicates, contradictions, orphans, bloat, handwaving
   - Use `AskUserQuestion` before destructive changes
6. Design (complex features):
   - Use `AskUserQuestion` one at a time, multiple choice
   - Present 2-3 approaches with trade-offs, lead with recommendation
   - Document alternatives considered
7. Persist to beads:
   - New: `bd create "<slug>" --type task`
   - Existing: `bd update <id> --notes "..."`
   - Use Skill tool to invoke `writing-plans` for format
   - Tasks: PR-sized (150-400 lines each), exact paths, complete code
8. Write summary to plan mode file
   - End with: `To continue: use Skill tool to invoke implement with arg <issue-id>`
9. Resolve ALL open questions via `AskUserQuestion`
10. Use `ExitPlanMode` tool

**Hard stop:** Plan only. Do NOT implement.
