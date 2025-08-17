# Development Partnership

We build production code together. I handle implementation details while you guide architecture and catch complexity early.

## Using tools

When editing, search, or reading code, check if using the serena MCP tools is appropriate before using general ones.

## Core Workflow: Research → Plan → Tests ->Implement → Validate

**Start every feature with:** "Let me research the codebase and create a plan before implementing."
**Complex Problems**: Use sequential thinking for challenging tasks

1. **Research** - Understand existing patterns and architecture
2. **Plan** - Propose approach and verify with you
3. **Tests** - Guide implementation with tests. TDD as much as possible.
4. **Implement** - Build with tests and error handling
5. **Validate** - ALWAYS run formatters, linters, and tests after implementation

## Code Organization

**Keep functions small and focused:**

- If you need comments to explain sections, split into functions
- Group related functionality into clear packages
- Prefer many small files over few large ones

## Architecture Principles

**This is always a feature branch:**

- Delete old code completely - no deprecation needed
- No versioned names (processV2, handleNew, ClientOld)
- No migration code unless explicitly requested
- No "removed code" comments - just delete it

**Prefer explicit over implicit:**

- Clear function names over clever abstractions
- Obvious data flow over hidden magic
- Direct dependencies over service locators

## Maximize Efficiency

- **Parallel operations:** Run multiple searches, reads, and greps in single messages
- **Multiple agents:** Aggressively split complex tasks - one for tests, one for implementation
- **Use your sub-agents:** Aggressively delegate tasks to sub-agents available in the system
- **Use MCP tools**: Aggressively use MCP tools like serena, context7 and others
- **Batch similar work:** Group related file edits together

## Problem Solving

- **When stuck:** Stop. The simple solution is usually correct.
- **When uncertain:** "Let me ultrathink about this architecture."
- **When choosing:** "I see approach A (simple) vs B (flexible). Which do you prefer?"

Your redirects prevent over-engineering. When uncertain about implementation,
stop and ask for guidance.

## Testing Strategy

- **TDD**: Let tests guide your design during implementation.
- **When no tests exist**: Ask if we're working on a throwaway prototype before giving up on tests.

## Progress Tracking

- **TodoWrite** for task management
- **Clear naming** in all code

Focus on maintainable solutions over clever abstractions.

## Neovim Integration

**Socket Control**:  Neovim runs with socket server for remote control

**Socket Discovery Protocol**:

- When receiving a message like NVIM_SOCKET=\<socket\>:
  - CONFIRM: `✳️ Neovim socket discovered and ready for use`
  - REMEMBER the socket path for all future commands, it IS NOT an environment variable

**Example commands** (replace `<socket>` with actual socket path):

- Get diagnostics: `nvim --server <socket> --remote-expr "luaeval('vim.fn.json_encode(vim.diagnostic.get())')"`
- Open file: `nvim --server <socket> --remote-tab-silent <file>`
