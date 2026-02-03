---
name: explore
description: "Use for investigation or plan refinement. Triggers: 'explore', 'how does X work', 'understand', 'research', 'plan a feature', 'figure out', 'investigate', 'design', 'architect', 'that's not right', 'try again', 'refine the plan', 'keep improving', 'reconsider'"
argument-hint: "<prompt> or [issue-id] <feedback>"
---

# Explore

Explore codebase → propose approaches → write plan → persist to beads.

Auto-detects existing work: if in_progress/open issue exists → refines it; else → creates new.

## Steps

1. **Check for existing issue**:
   - arg is issue-id → refine that issue
   - no arg → check `bd list --status in_progress` or recent open
   - existing issue? → load plan from notes, refine based on feedback
   - no existing? → fresh exploration
2. **Use EnterPlanMode tool** to switch to plan mode
3. **Explore via Task tool** (subagent_type=Explore):
   - Fresh: "Explore how [X] works, identify patterns/files involved, suggest 2-3 approaches"
   - Refine: "Given feedback [X], re-examine [Y], suggest revised approach"
4. **Design process** (for complex features):
   - Ask questions one at a time via `AskUserQuestion`
   - Prefer multiple choice when options are clear
   - Present 2-3 approaches with trade-offs, lead with recommendation
5. **Create or update beads issue**:
   - New: `bd create "<feature-slug>" --type task`
   - Existing: `bd update <id> --notes "..."`
   - Use notes field for full plan (writing-plans format)
   - Bite-sized tasks, exact file paths, complete code
   - YAGNI ruthlessly
6. **Write summary to plan mode file** (for approval UI):
   - End with: `To continue: use Skill tool to invoke implement with arg <issue-id>`
7. **MANDATORY: Resolve ALL Open Questions** via `AskUserQuestion` before ExitPlanMode
