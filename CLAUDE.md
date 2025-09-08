# Development Partnership

We build production code together. I handle implementation details while you guide architecture and catch complexity early.

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

**Zero Dead Code Policy:**

- **Delete immediately**: Unused functions, imports, variables, commented code
- **Before every commit**: Remove unreachable code, unused parameters
- **During refactoring**: Delete old implementations completely  
- **No preservation**: No "for reference", "might need later", or commented blocks
- **Use version control**: Git preserves history, no need to keep dead code

**Prefer explicit over implicit:**

- Clear function names over clever abstractions
- Obvious data flow over hidden magic
- Direct dependencies over service locators

## Maximize Efficiency

- **Parallel operations:** Run multiple searches, reads, and greps in single messages
- **Multiple sub-agents:** Aggressively split tasks into multiple sub-agents
- **Use your sub-agents:** Aggressively delegate tasks to sub-agents available in the system
- **Use MCP tools**: Aggressively use MCP tools like cclsp, context7 and others
- **Batch similar work:** Group related file edits together

### Mandatory Agent Usage

**⛔ ALWAYS REQUIRED - NO EXCEPTIONS:**
- **ALL commits** → `committer` agent (NEVER commit without this agent)
- **ALL Rust code** → `rust-engineer` agent (NEVER write Rust without this agent)
- **ALL quality issues** → `quality-control-enforcer` agent 
- **ALL CLAUDE.md compliance** → `claude-md-checker` agent

**🚨 AGENT BYPASS = SESSION FAILURE**  
If you find yourself doing any of the above without the agent, STOP immediately and invoke the appropriate agent.

## Problem Solving

- **When stuck:** Stop. The simple solution is usually correct.
- **When uncertain:** "Let me ultrathink about this architecture."
- **When choosing:** "I see approach A (simple) vs B (flexible). Which do you prefer?"

Your redirects prevent over-engineering. When uncertain about implementation,
stop and ask for guidance.

## Testing Strategy

- **TDD**: Let tests guide your design during implementation.
- **When no tests exist**: Ask if we're working on a throwaway prototype before giving up on tests.

## Agent-Based Workflow

### Start of Every Feature
1. **Research** with appropriate domain agents
2. **Plan** with `architect` for cross-language decisions
3. **Create todo list** with TodoWrite

### During Implementation
1. **Delegate** to specialized agents (rust-engineer, build, etc.)
2. **Use quality-control-enforcer** at stopping points to prevent abandonment
3. **Monitor** for repeated issues and use quality-control-enforcer proactively

### End of Every Task
1. **Run claude-md-checker** to validate CLAUDE.md compliance  
2. **Use quality-control-enforcer** before marking complete
3. **ALL commits** must use `committer` agent

## Progress Tracking

- **TodoWrite** for task management
- **EVERY todo list must END with**: "Run claude-md-checker agent"
- **Every 5 interactions**: MANDATORY `quality-control-enforcer` check
- **At natural stopping points**: Use `quality-control-enforcer` to prevent task abandonment  
- **Before session end**: Validate with both quality agents
- **Clear naming** in all code

## CRITICAL: Agent Non-Negotiables

**⛔ VIOLATIONS THAT REQUIRE IMMEDIATE CORRECTION**:
1. Attempting ANY commit without `committer` agent
2. Writing ANY Rust code without `rust-engineer` agent
3. Marking task complete without `claude-md-checker`
4. Going 5+ interactions without `quality-control-enforcer`
5. Ignoring domain-specific agents for their areas

**🚨 If you catch yourself bypassing agents:**
1. STOP immediately
2. Invoke the appropriate agent
3. Use `quality-control-enforcer` to review the violation

Focus on maintainable solutions over clever abstractions.

## Language Specific

ALWAYS read @lang/ for the appropriate language if it's present
