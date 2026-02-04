---
name: subagent-driven-development
description: "Triggers: have plan with independent tasks, need structured implementation. Fresh subagent per task + two-stage review."
user-invocable: true
---

# Subagent-Driven Development

Fresh subagent per task + two-stage review.

## When to Use

- Have implementation plan? YES
- Tasks mostly independent? YES
- → Use this skill

## Process

Per Task:
1. Dispatch implementer (`./implementer-prompt.md`)
2. Handle questions (see below)
3. Implementer: implements, tests, commits, self-reviews
4. Dispatch spec reviewer (`./spec-reviewer-prompt.md`)
5. If spec issues: implementer fixes → re-review
6. Dispatch quality reviewer (`./code-quality-reviewer-prompt.md`)
7. If quality issues: implementer fixes → re-review
8. Mark task complete

## Handling Questions

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Make best-judgment call based on codebase patterns, document in beads notes, proceed." || echo "Present to user via AskUserQuestion, get answer, relay to subagent."`

## After All Tasks

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "use Skill tool to invoke finishing-branch --pr" || echo "use Skill tool to invoke finishing-branch"`

## Key Principles

- Controller provides full context (paste task text, don't make subagent read files)
- Spec review FIRST (built what was requested?)
- Quality review SECOND (well-built?)
- Review loops until approved

## Skill Composition

| When | Invoke |
|------|--------|
| Task fails | `use Skill tool to invoke debugging` |
| Before claiming done | `use Skill tool to invoke verification-before-completion` |
| All tasks complete | `use Skill tool to invoke finishing-branch` |
