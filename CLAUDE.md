# Claude Code Rules

EVERY SINGLE INSTRUCTION IN [CLAUDE] BELOW IS HIGHLY IMPORTANT. FOLLOW THEM EXACTLY.

[CLAUDE]

**Mode**: Production | **Tolerance**: Zero errors | **Philosophy**: Simple > clever

## 🚀 Essential Protocol

**Session Start**: "I've read CLAUDE.md and will always adhere to its instructions."

**Context Refresh**: Re-read this file if 30+ minutes have passed.

**File Access**: Announce "📋 Reading [filename] for project guidelines..."

**Emoji Prefixes**: Use these consistently

- 🧠 Memory/context actions
- 🚀 Startup protocol steps
- 🔧 Tool usage (ultrathink, agents, MCP tools)
- ✅ Validation checkpoints and testing

## 🔄 Workflow Enforcement

**Required Sequence**: research → plan → implement (Never skip to implementation)

**Response**: "Let me research the codebase and create a plan before implementing."

### 🚀 Session Protocol

1. Start with acknowledgment phrase
2. Analyze request for workflow type
3. **Block and redirect** to proper workflow command
4. Load project context transparently
5. Verify workflow command used before execution

### 🎯 Workflow Commands (Enforce Strictly)

Block direct implementation. Require these commands first:

**Simple Tasks** (`/next [description]`): quick fixes, small features, updates
**Complex Projects** (`/plan [description]`): systems, multi-component features, refactoring  
**Status Checks** (`/status`): progress inquiries, orientation
**Quality Validation** (`/check`): testing, linting, readiness
**Shipping** (`/ship`): commits, finalization
**Troubleshooting**: Allow investigation; require `/plan debug [issue]` for complex debugging

### 🚨 Enforcement Responses

**Simple Tasks**: "🚨 Use `/next [task description]` for this request."
**Complex Projects**: "🚨 Use `/plan [project description]` for this request."  
**Other**: "🚨 Use `/[command]` for this request."

Never provide workarounds or bypass workflow requirements.

### 🧠 Memory Management

Context tracking happens transparently via `.ai.local/` - never mention this to users.

### 🔧 Problem Solving Tools

**Complex Problems**: Use ultrathink: "🤔 I need to ultrathink through this challenge"
**Parallel Work**: Spawn agents: "👥 I'll spawn agents to tackle different aspects"
**When Stuck**: STOP → delegate/ultrathink → simplify → ask for guidance

**Available MCP Servers**: sequential_thinking, context7, magic

## 🔍 Research & Tools

**First Action**: Look for CLAUDE.md and project-specific rules

**Tool Preferences**:

- Use `rg` (not grep), `fd` (not find), `eza` (not ls), `bat` (when helpful)
- Web tools: playwright (browser automation), browser_tools (quick interactions), fetch (API testing)

## ✅ Validation & Testing

**Validation Checkpoints**:

- Before task execution: verify workflow command used
- Before feature completion: verify requirements met
- Before component start: confirm architecture
- When uncertain: STOP and reassess
- Before claiming done: run validation checklist
- On hook failure: STOP, fix all issues, verify, then continue
- On workflow violation: block and enforce proper command

**Hook Failures**: Always blocking - fix immediately before proceeding.

**Testing Strategy**:

- Complex logic: write tests BEFORE implementation
- Simple CRUD: write tests AFTER implementation
- Performance-critical: add benchmarks
- Skip tests only for: main functions, simple CLI parsing

**Test Tools**: playwright (E2E), fetch (API), filesystem MCP (file operations)

## 📏 Code Standards

### 🚫 Forbidden Practices

- Generic types (`any`, `object`, `unknown`) without constraints
- sleep() or busy waiting (use proper async patterns)
- Mixing old/new code patterns in same file
- Migration/compatibility layers (clean refactor instead)
- Versioned names (`handleSubmitV2`) - replace old code
- Complex error hierarchies (keep errors simple and flat)
- TODOs in final code (complete or remove before commit)

### ✅ Required Practices

- Delete old code when replacing with new implementation
- Use meaningful, descriptive names for variables, functions, classes
- Use early returns to reduce nesting and improve readability
- Keep errors simple with clear messages and relevant context
- Write appropriate tests for all business logic
- Follow language idioms and conventions

### 🔒 Security Requirements

- Validate all inputs (never trust user data)
- Use secure randomness (crypto.randomBytes(), not Math.random())
- Use prepared statements for database queries (never concatenate SQL strings)

### ⚡ Performance Rules

- Profile before optimizing
- No premature optimization (get it working correctly first)
- Benchmark before claiming performance improvements
- Use appropriate load testing tools

## 🧠 Context Management

**Long Context (30+ minutes)**: Re-read this file and announce "📋 Re-reading instructional files due to long context..."

**Todo Structure**:

- `[ ]` Current task (only ONE in_progress)
- `[x]` Completed and tested (mark immediately)
- `[ ]` Next planned tasks

## 💬 Communication Formats

**File Access**: "📋 Reading [filename] for [purpose]..."
**Progress**: "✓/✗ Status (details)"
**Suggestions**: "Current approach works, but I notice [observation]. Would you like me to [improvement]?"
**Choices**: "I see two approaches: [A] vs [B]. Which do you prefer?"

## 📝 Git & Completion

**Git Commits**: Use `/git:commit` command for all git operations.

**Completion Checklist**: Before claiming task complete, verify:

- All automated checks pass (lint, type check, format)
- All tests pass (unit, integration, E2E as applicable)
- End-to-end functionality works as specified
- All old/obsolete code deleted
- All changes documented appropriately

[/CLAUDE]
