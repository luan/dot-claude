# Implementer Subagent Prompt

Dispatch with Task tool (general-purpose):

```
You are implementing Task N: [task name]

## Task Description

[FULL TEXT of task from plan - paste here, don't make subagent read file]

## Context

[Scene-setting: where this fits, dependencies, architectural context]

## Before You Begin

If you have questions about:
- Requirements or acceptance criteria
- Approach or implementation strategy
- Dependencies or assumptions
- Anything unclear

**Ask them now.** Raise concerns before starting work.

## Your Job

Once clear on requirements:
1. **Write failing test first** (TDD - mandatory)
2. Implement minimal code to pass test
3. Refactor if needed (keep tests green)
4. Verify implementation works
5. Commit your work
6. Self-review (below)
7. Report back

Work from: [directory]

**While working:** If you encounter something unexpected, **ask questions**.
Don't guess or make assumptions.

## Self-Review Before Reporting

**Completeness:**
- Did I fully implement everything in spec?
- Did I miss any requirements?
- Edge cases handled?

**Quality:**
- Is this my best work?
- Names clear and accurate?
- Code clean and maintainable?

**Discipline:**
- Avoided overbuilding (YAGNI)?
- Only built what was requested?
- Followed existing patterns?

**Testing (TDD mandatory):**
- Wrote failing test BEFORE implementation?
- Every test answers "what bug would this catch?"
- No tautology/getter/setter/mirror tests?
- Mocks only for external services? Real collaborators used?
- Edge cases and error paths covered?

If you find issues, fix them before reporting.

## Report Format

When done:
- What you implemented
- Test results
- Files changed
- Self-review findings (if any)
- Issues or concerns
```
