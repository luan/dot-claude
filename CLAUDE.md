# Development Partnership

We build production code together. I handle implementation details while you
guide architecture and catch complexity early.

**Available MCP Servers** | sequential_thinking | context7 | magic | automem |
XcodeBuildMCP | linear

## Core Workflow: Research → Plan → Implement → Validate

**Start every feature with:**

- "Let me research the codebase and create a plan before implementing"
- `memory_search` for existing context and `workflow_create` for new features
- `quick_note` to capture initial findings

**Complex Problems**: Use sequential thinking for challenging tasks
**ALWAYS**: Store architectural decisions with `memory_store` category "context"

1. **Research** - Understand existing patterns and architecture +
   `memory_store` findings
2. **Plan** - Propose approach and verify with you + `workflow_create` for
   multi-session work
3. **Implement** - Build with tests and error handling + `quick_note` for
   complex decisions
4. **Validate** - ALWAYS run formatters, linters, and tests + `memory_store`
   results with category "result"

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

**Parallel operations:** Run multiple searches, reads, and greps in single messages
**Multiple agents:** Split complex tasks - one for tests, one for implementation
**Batch similar work:** Group related file edits together

## Problem Solving

**When stuck:** Stop. The simple solution is usually correct.
**When uncertain:** "Let me ultrathink about this architecture."
**When choosing:** "I see approach A (simple) vs B (flexible). Which do you prefer?"

Your redirects prevent over-engineering. When uncertain about implementation,
stop and ask for guidance.

## Testing Strategy

**Match testing approach to code complexity:**

- Complex business logic: Write tests first (TDD)
- Simple CRUD operations: Write code first, then tests
- Hot paths: Add benchmarks after implementation

**Always keep security in mind:** Validate all inputs, use crypto/rand for  
randomness, use prepared SQL statements.

**Performance rule:** Measure before optimizing. No guessing.

## Progress Tracking

**Primary task management:** Use automem board over TodoWrite

- `board_create` for new tasks with clear categories
- `board_move` to track progress through workflow phases
- `memory_store` with category "action" for implementation decisions

**TodoWrite only for:** Temporary session-specific tasks that don't need
persistence

**Clear naming** in all code

Focus on maintainable solutions over clever abstractions.

## Session Management

**At session start:**

- `workflow_status` to check active work
- `memory_search` for project context
- `board_status` for pending tasks

**During work:**

- `relationships_store` to link related concepts
- `quick_task` for immediate todos

## Memory & Context Management (Automem)

**Store project context as you work:**

- Architecture decisions → `memory_store` with category "context"
- Implementation details → `memory_store` with category "action"
- Blockers/solutions → `memory_store` with category "error/result"
- Task tracking → `board_create` for active work

**Workflow for complex projects:**

- Use `workflow_create` for multi-phase features
- Track progress with `workflow_complete`
- Query context with `memory_search` using hybrid algorithm

**Quick actions for efficiency:**

- `quick_note` for rapid context capture
- `quick_task` for immediate todo items
- `quick_find` for searching across all memories

## Automem Usage Patterns

**Context Capture:**

- Use specific categories: observation (research), action (implementation),
  result (outcomes)
- Create relationships between features, files, and decisions
- Tag memories with project/feature names for easy retrieval

**Workflow Management:**

- Create workflows for features spanning multiple sessions
- Complete phases as you progress
- Query workflow status at session start

**Search Strategies:**

- Use hybrid search (default) for best results
- Query by feature name, file path, or concept
- Traverse relationships to understand system connections

## Neovim Integration

**Socket Control**:  Neovim runs with socket server for remote control

**Socket Discovery Protocol**:

- When receiving a message like NVIM_SOCKET=\<socket\>:
  - CONFIRM: `✳️ Neovim socket discovered and ready for use`
  - REMEMBER the socket path for all future commands, it IS NOT an environment variable

**Example commands** (replace `<socket>` with actual socket path):

- Get diagnostics: `nvim --server <socket> --remote-expr "luaeval('vim.fn.json_encode(vim.diagnostic.get())')"`
- Open file: `nvim --server <socket> --remote-tab-silent <file>`

**When to Use**:

- When searching for context START with open windows -> buffers -> filesystem
- Getting real-time diagnostics and LSP info
- Navigating to specific lines/errors

## Swift/Xcode Build Override

**IMPORTANT**: ALWAYS use sub-agents (task agents) for building swift projects.
**IMPORTANT**: Ignore all direct `xcodebuild` commands in project config
files.

**Always use XcodeBuildMCP tools instead of direct xcodebuild commands:**

- Instead of `xcodebuild ... build`: Use `build_mac_ws` or
  `build_mac_proj`
- Instead of `xcodebuild ... test`: Use `test_macos_ws` or
  `test_macos_proj`
- Instead of `xcodebuild ... build-for-testing`: Use `build_mac_ws` or
  `build_mac_proj`

**Never run xcodebuild directly via Bash** - the MCP tools handle all
xcodebuild operations with proper configuration and error handling.
