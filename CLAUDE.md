## Plan Mode

- Make the plan extremely concise. Sacrifice grammar for the sake of concision.
- At the end of each plan, give me a list of unresolved questions to answer, if any.

## Code Organization

**Keep functions small and focused:**

- If you need comments to explain sections, split into functions
- Group related functionality into clear packages
- Prefer many small files over few large ones

## Architecture Principles

- Delete old code completely - no deprecation needed
- No semantic prefix or suffix (OptimizedProcessor, FastHandler, ClientImpl)
- No versioned names (processV2, handleNew, ClientOld)
- No migration code unless explicitly requested
- No "removed code" comments - just delete it
- Don't add unnecessary comments. Only add doc comments if it's the style of the existing project.
- Comments that explain something subtle or surprising are OK.

## Maximize Efficiency

- **Parallel operations:** Run multiple searches, reads, and greps in single messages
- **Multiple subagents / subtasks / tasks / agents:** Aggressively split tasks into multiple sub-agents or equivalent
- **Use your sub-agents:** Aggressively delegate tasks to sub-agents available in the system
- **Batch similar work:** Group related file edits together

## Problem Solving

- **When stuck:** Stop. The simple solution is usually correct.
- **When uncertain:** Ask clarification.

## Asking Questions

- **Use `AskUserQuestion` tool** when asking the user questions - faster than text output
- Prefer multiple choice when options are clear
- Use for: clarifications, design decisions, continue/abort, ready to proceed

## Testing Strategy

**TDD is the default.** No production code without a failing test first.

- Write test → watch it fail → minimal code to pass → refactor
- If you write code before test: delete it, start over with test
- Bug fix? Write failing test that reproduces bug first
- "Already manually tested" or "too simple to test" are not valid reasons to skip TDD

**Exceptions (ask first):** throwaway prototypes, generated code, config-only changes.

## Agent Workflow (beads)

State tracking for exploration → implementation flows via beads issues.

**Skills:**

- `/explore <prompt>` - subagent explores, writes plan to beads issue
- `/implement [issue-id] [--fresh]` - execute plan, track state via beads (--fresh clears context first)

Session state (save/resume) handled automatically via beads notes field.

**CRITICAL: After plan approval**, when you see "To continue: use Skill tool to invoke implement with arg X":
- **IMMEDIATELY** use Skill tool with skill="implement" args="X"
- Do NOT manually implement - the skill handles subagent dispatch
- User saying "yes", "go ahead", "proceed" after plan = invoke implement NOW
