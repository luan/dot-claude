---
name: implement
description: "MANDATORY trigger on: 'To continue: use Skill tool to invoke implement', 'invoke implement', 'implement with arg', beads issue ID, 'execute the plan', 'build this', 'code this plan', plan with Tasks section. Extract issue-id from 'with arg X' as skill argument. NEVER manually implement - ALWAYS use this skill."
argument-hint: "[issue-id] [--fresh]"
---

# Implement

Execute plan from `explore`, tracked via beads.

## Flags

- `--fresh`: Clear context first (EnterPlanMode → summarize progress → user approves → continue)

## Steps

1. **Find plan**:
   - arg → `bd show <arg>`
   - no arg → `bd list --status in_progress` (resume existing)
   - still nothing → `bd ready` (pick next unblocked)
   - No issue? → "Nothing to do. Run /explore first"
2. **If --fresh**: EnterPlanMode, summarize completed/remaining from notes, get approval
3. **Mark in_progress**: `bd update <id> --status in_progress`
4. **CHECK FOR OPEN QUESTIONS** in plan:
   - If notes has unresolved questions → resolve via `AskUserQuestion` FIRST
5. **FOR EACH TASK** (MANDATORY subagent dispatch):
   ```
   a. Task tool → implementer subagent (paste full task text)
      - Implementer MUST use TDD: failing test → minimal code → pass
   b. Task tool → spec reviewer subagent → if issues: fix → re-review
   c. Task tool → quality reviewer subagent → if issues: fix → re-review
   d. Update beads notes with progress
   ```
   - NEVER implement tasks yourself - ALWAYS dispatch via Task tool
   - If task fails → **use Skill tool** to invoke `debugging`
   - **TDD is mandatory** unless explicitly building throwaway prototype
6. **EARLY VERIFICATION**: After FIRST task, run build/check to catch design flaws
7. **Update notes** after each task: `bd update <id> --notes "COMPLETED: X. NEXT: Y"`
8. **Context getting stale?** → suggest `/implement --fresh` to user
9. Final → `bd close <id>` + **use Skill tool** to invoke `finishing-branch`

## Subagent Prompts

- **subagent-driven-development**/`implementer-prompt.md`
- **subagent-driven-development**/`spec-reviewer-prompt.md`
- **subagent-driven-development**/`code-quality-reviewer-prompt.md`

Key: paste full task text + context into prompt. Don't make subagent read files.
