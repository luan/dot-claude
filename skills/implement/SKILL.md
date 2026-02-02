---
name: implement
description: "MANDATORY trigger on: 'To continue: use Skill tool to invoke implement', 'invoke implement', 'implement with arg', ANY .agents/plans/ filename, 'execute the plan', 'build this', 'code this plan', plan with Tasks section. Extract filename from 'with arg X' as skill argument. NEVER manually implement - ALWAYS use this skill."
argument-hint: "[plan-file] (default: most recent)"
---

# Implement

Current branch: !`git branch --show-current | tr '/' '-'`

Execute plan from `explore`.

## Steps

1. Find plan: arg → `.agents/plans/{arg}` or most recent
2. **CHECK FOR OPEN QUESTIONS** in plan:
   - If plan has unresolved "Open Questions" section → resolve via `AskUserQuestion` FIRST
   - Do NOT proceed with tasks until questions answered
3. Create/resume `.agents/active-{branch}.md` (source, branch, status, tasks)
4. **FOR EACH TASK** (MANDATORY subagent dispatch):
   ```
   a. Task tool → implementer subagent (paste full task text, don't reference files)
   b. Task tool → spec reviewer subagent → if issues: implementer fixes → re-review
   c. Task tool → quality reviewer subagent → if issues: implementer fixes → re-review
   d. Mark task complete in active file
   ```
   - NEVER implement tasks yourself - ALWAYS dispatch via Task tool
   - If task fails → **use Skill tool** to invoke `debugging`
5. **EARLY VERIFICATION**: After FIRST task, run build/check to catch design flaws
6. Multi-phase → **use Skill tool** to invoke `next-phase`
7. Final → **use Skill tool** to invoke `finishing-branch`

## Subagent Prompts (read these, use as Task tool prompt)

- **subagent-driven-development**/`implementer-prompt.md`
- **subagent-driven-development**/`spec-reviewer-prompt.md`
- **subagent-driven-development**/`code-quality-reviewer-prompt.md`

Key: paste full task text + context into prompt. Don't make subagent read files.

## Errors

- No plan: "Run explore first"
- Task fail: preserve state, report
