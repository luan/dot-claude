---
name: explore
description: "Triggers: 'explore', 'how does X work', 'understand', 'research', 'plan a feature', 'figure out', 'investigate', 'design', 'architect', 'that's not right', 'try again', 'refine the plan', 'keep improving', 'reconsider'"
argument-hint: "<prompt> or [issue-id] <feedback>"
---

# Explore

Explore codebase → propose approaches → write plan → persist to beads.

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && cat ~/.claude/skills/explore/non-interactive.md || cat ~/.claude/skills/explore/interactive.md`

## Skill Composition

| When | Invoke |
|------|--------|
| Writing plan | `use Skill tool to invoke writing-plans` |
| Plan approved | Output: `To continue: use Skill tool to invoke implement with arg <id>` |
