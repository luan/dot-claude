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

## Agent Workflow (.agents/)

State tracking for exploration → implementation flows.

**Global** (`~/.claude/.agents/`):

- `sessions/{branch}.md` - session context, auto-loaded if <30 min
- `archive/` - historical sessions

**Per-project** (`.agents/`):

- `plans/{ts}-{slug}.md` - exploration outputs
- `active-{branch}.md` - implementation progress
- `archive/` - completed work

**Skills:**

- `/explore <prompt>` - subagent explores, writes plan
- `/implement [plan]` - execute plan, track state
- `/next-phase` - continue multi-phase work
- `/save-state [summary]` - save session
- `/resume-state` - load session (auto on start)

**CRITICAL: After plan approval**, when you see "To continue: use Skill tool to invoke implement with arg X":
- **IMMEDIATELY** use Skill tool with skill="implement" args="X"
- Do NOT manually implement - the skill handles subagent dispatch
- User saying "yes", "go ahead", "proceed" after plan = invoke implement NOW
