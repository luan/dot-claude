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

## Maximize Efficiency

- **Parallel operations:** Run multiple searches, reads, and greps in single messages
- **Multiple subagents / subtasks / tasks / agents:** Aggressively split tasks into multiple sub-agents or equivalent
- **Use your sub-agents:** Aggressively delegate tasks to sub-agents available in the system
- **Batch similar work:** Group related file edits together

## Problem Solving

- **When stuck:** Stop. The simple solution is usually correct.
- **When uncertain:** Ask clarification.

## Testing Strategy

- **TDD**: Let tests guide your design during implementation.
- **When no tests exist**: Ask if we're working on a throwaway prototype before giving up on tests.
