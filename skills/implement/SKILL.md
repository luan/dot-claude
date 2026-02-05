---
name: implement
description: "Triggers: 'To continue: use Skill tool to invoke implement', 'invoke implement', 'implement with arg', beads issue/epic ID, 'execute the plan', 'build this', 'code this plan'. Extract issue-id from 'with arg X'."
argument-hint: "[issue-id|epic-id] [--fresh]"
---

# Implement

Execute plan from explore, tracked via beads. NEVER implement manually—ALWAYS use this skill.

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && cat ~/.claude/skills/implement/non-interactive.md || cat ~/.claude/skills/implement/interactive.md`

## Skill Composition

| When | Invoke |
|------|--------|
| Task fails | `use Skill tool to invoke debugging` |
| Before claiming done | `use Skill tool to invoke verification-before-completion` |
| Quality check | `use Skill tool to invoke critical-review` |
| All tasks complete | `use Skill tool to invoke finishing-branch` |

## Subagent Prompts

- `subagent-driven-development/implementer-prompt.md`
- `subagent-driven-development/spec-reviewer-prompt.md`
- `subagent-driven-development/code-quality-reviewer-prompt.md`

Key: paste full task text + context. Don't make subagent read files.
