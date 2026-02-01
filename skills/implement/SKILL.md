---
name: implement
description: "Use when: 'Implement the following plan', 'To continue: use Skill tool to invoke implement', executing a plan from .agents/plans/, or user wants to build/code an explored plan"
argument-hint: "[plan-file] (default: most recent)"
---

# Implement

Current branch: !`git branch --show-current | tr '/' '-'`

Execute plan from `explore`.

## Steps

1. Find plan: arg → `.agents/plans/{arg}` or most recent
2. Create/resume `.agents/active-{branch}.md` (source, branch, status, tasks)
3. Execute tasks using **subagent-driven-development** pattern:
   - Fresh subagent per task via Task tool
   - Two-stage review: spec compliance → code quality
   - Use **verification-before-completion** before marking task done
   - If task fails → **use Skill tool** to invoke `debugging`
4. Multi-phase → **use Skill tool** to invoke `next-phase`
5. Final → **use Skill tool** to invoke `finishing-branch`

## Subagent Pattern

Use prompts from **subagent-driven-development** skill:
- `implementer-prompt.md` - dispatch per task
- `spec-reviewer-prompt.md` - verify spec compliance first
- `code-quality-reviewer-prompt.md` - verify quality second

Key: provide full task text + context to subagent, don't make them read files.

## Errors

- No plan: "Run explore first"
- Task fail: preserve state, report
