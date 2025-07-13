---
mode: production
tolerance: zero_errors
philosophy: simple > clever
partnership: "We're building production-quality code together"
acknowledgment_required: true
acknowledgment_format: "I've read CLAUDE.md and will always adhere to its instructions."
context_refresh_trigger: 30_minutes
startup_protocol_mandatory: true
hook_failures_blocking: true
tool_preferences:
  search: rg
  find: fd
  file_viewer: bat
  directory_listing: eza
workflow_sequence: [research, plan, implement]
forbidden_actions: [jump_to_code]
---

# Claude Code Rules

# ⚠️ MANDATORY ACKNOWLEDGMENT

**YOU MUST** start every response with: "I've read CLAUDE.md and will always adhere to its instructions."

**Context Reminder**: If this file hasn't been referenced in 30+ minutes, RE-READ IT!

**File Reading Protocol**: Always announce: "📋 Reading [filename] for project guidelines..."

## Workflow (STRICT)

**MANDATORY Sequence**: research → plan → implement  
**FORBIDDEN**: jump_to_code  
**REQUIRED Response**: "Let me research the codebase and create a plan before implementing."

### Startup Protocol (MANDATORY)

**MUST DO ON EVERY SESSION:**

1. **ALWAYS** start with acknowledgment phrase
2. Check if in project directory (git repo, package.json, Cargo.toml, pyproject.toml)
3. If `.ai.local/` exists: announce "🧠 Checking .ai.local for previous context..." and load session/progress/architecture files
4. If `.ai.local/` missing: **IMMEDIATELY** create directory structure with context/progress/research/session folders
5. **ALWAYS** check for CLAUDE.md in project root
6. **MANDATORY** announce: "📋 Reading CLAUDE.md for project guidelines..."
7. Check for other rule files (.claude-rules, claude.config)
8. **MUST** acknowledge any specific commands or workflows found
9. **ALWAYS** update `.ai.local/session/last-session.md` with session start

### Tools & Problem Solving

- **Complex problems**: ultrathink
- **Parallel work**: spawn_agents
- **When stuck**: stop, delegate, ultrathink, simplify, ask

**Agent examples**:
- "I'll spawn agents to tackle different aspects of this problem"
- "I'll have an agent investigate the database schema while I analyze the API structure"
- "One agent writes tests while another implements features"

**MCP Servers**:
- `sequential_thinking`: Break down complex problems into step-by-step reasoning
- `filesystem`: Navigate and explore codebase structure, read/write files
- `context7`: Maintain context across long conversations and complex tasks
- `magic`: Swiss-army knife for various automation tasks

## Research Tools

**Primary**: filesystem for codebase exploration  
**First action**: Look for CLAUDE.md and project-specific rules

**Faster tools** (always prefer):
- `rg` over `grep`
- `fd` over `find`
- `bat` when syntax highlighting helps
- `eza/lsd` in interactive contexts

**Web research**: puppeteer, playwright, browser_tools for automation; fetch for API testing

## Validation & Testing

**Checkpoints**: feature_complete, new_component_start, feels_wrong, before_done, hook_failure

**Hook failures**: BLOCKING severity - stop, fix_all, verify, continue, never_ignore

**Testing strategy**:
- Complex logic: tests_first
- Simple CRUD: tests_after
- Hot paths: add_benchmarks
- Skip: main_functions, simple_cli_parsing

**Automation**: playwright/puppeteer for E2E, fetch for API, filesystem for file-based

## Code Standards

### Forbidden
- generic_types
- sleep_or_busy_wait
- old_new_code_together
- migration_layers
- versioned_names
- complex_error_hierarchies
- todos_in_final

### Required
- delete_old_code
- meaningful_names
- early_returns
- simple_errors
- appropriate_tests
- language_idioms

### Security
- validate_inputs
- secure_randomness
- prepared_statements
- **SQL**: Never concatenate! Use prepared statements

### Performance
- measure_before_optimize
- No premature optimization
- Benchmark before claiming something is faster

## Memory & Context

**External brain**: `.ai.local/` for persistent memory across Claude Code sessions

**Structure**:
- `context/`: Long-term project understanding
- `progress/`: Task tracking and state
- `research/`: Findings and references
- `session/`: Session-specific data

**When context long**:
- Reread this file
- Reread project CLAUDE.md
- Announce rereading
- Update `.ai.local/progress/current.md`
- Use context7 MCP

**Todo structure**:
- `[ ]` What we're doing RIGHT NOW
- `[x]` What's actually done and tested
- `[ ]` What comes next

**Always update**: `.ai.local/progress/current.md`, `.ai.local/session/last-session.md`

## Communication

**File acknowledgment**: "📋 Reading [filename] for [purpose]..."

**Progress format**: "✓/✗ Status (details)"

**Improvement format**: "The current approach works, but I notice [observation]. Would you like me to [specific improvement]?"

**When choosing**: "I see two approaches: [A] vs [B]. Which do you prefer?"

## Git Conventions

**Format**: `type(scope): short description`  
**Length**: 50 characters max  
**Style**: conventional commits, very short, precise

**Types**: feat, fix, docs, style, refactor, test, chore

**Requirements**:
- Use imperative mood
- No period at end
- Keep under 50 characters

**Examples**:
- `feat: add user auth`
- `fix: handle null refs`
- `docs: update API guide`

## Complete Checklist

- all_checks_green
- tests_pass
- works_e2e
- old_deleted
- documented