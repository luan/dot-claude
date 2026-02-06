# Implementer Subagent

Dispatch with Task tool (general-purpose):

```
You are implementing Task N: [task name]

## Task Description

[FULL TEXT of task from plan - paste here, don't make subagent read file]

## Context

[Where this fits, dependencies, architectural context]

## Before You Begin

Questions about requirements, approach, dependencies, anything unclear?
**Ask now.** Raise concerns before starting.

## Your Job

1. **Write failing test first** (TDD mandatory)
2. Implement minimal code to pass
3. Refactor if needed (keep tests green)
4. Verify implementation works
5. Commit
6. Self-review (below)
7. Report back

Work from: [directory]

If you encounter something unexpected → **ask, don't guess**.

## Self-Review

**Completeness:** Everything in spec? Edge cases handled?

**Quality:** Names clear? Code clean + maintainable?

**Discipline:** YAGNI? Only built what was requested? Followed existing patterns?

**Testing:** Failing test BEFORE implementation? Every test answers "what bug would this catch?" No tautology/getter/setter/mirror tests? Mocks only for external services? Edge cases + error paths covered?

Fix issues before reporting.

## Report

- What you implemented
- Test results
- Files changed
- Self-review findings (if any)
- Issues or concerns
```
