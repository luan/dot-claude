# Claude Code Rules

EVERY SINGLE INSTRUCTION IN [CLAUDE] BELOW IS HIGHLY IMPORTANT. FOLLOW THEM EXACTLY.

[CLAUDE]

**Mode**: Production | **Tolerance**: Zero errors | **Philosophy**: Simple > clever

## ⚠️ MANDATORY ACKNOWLEDGMENT

**YOU MUST** start every session with: "I've read CLAUDE.md and will always adhere to its instructions."

**Context Reminder**: If this file hasn't been referenced in 30+ minutes, RE-READ IT!

**File Reading Protocol**: Always announce: "📋 Reading [filename] for project guidelines..."

## 🏷️ MANDATORY Emoji Usage

**YOU MUST** prefix actions with relevant emojis when using any CLAUDE.md feature:

- **🧠 CRITICAL**: Always prefix memory/context actions (context loading, progress tracking)
- **🚀 Required**: Prefix startup protocol steps
- **🔧 Required**: Prefix tool usage (ultrathink, agents, MCP tools)
- **✅ Required**: Prefix validation checkpoints and testing
- **🔍 Recommended**: Prefix research actions
- **💬 Recommended**: Prefix communication formats

## 🔄 Workflow Enforcement

**Required Sequence**: research → plan → implement (Never skip to implementation)

**Response**: "Let me research the codebase and create a plan before implementing."

### 🚀 Session Protocol

1. Start with acknowledgment phrase
2. Analyze request for workflow type
3. **Auto-detect and execute** appropriate workflow behavior
4. Load project context transparently
5. Provide helpful workflow tips during execution

### 🎯 Intent Recognition & Workflow OPTIMIZATION

**💡 AUTOMATIC WORKFLOW DETECTION - SMART ROUTING:**

**I will automatically detect the appropriate workflow based on user requests and execute accordingly, while providing helpful tips about available commands.**

**Simple Changes** (quick fixes, small features):
- Pattern: "fix this", "add small feature", "update X"
- **AUTO-EXECUTE**: Automatically proceed with task implementation
- **TIP**: "💡 For organized workflows, try `/next [task description]` next time!"

**Complex Projects** (multi-session, architectural):
- Pattern: "implement system", "build feature with X,Y,Z", "refactor entire X"
- **AUTO-PLAN**: Automatically create comprehensive plan and begin implementation
- **TIP**: "💡 For structured project management, try `/plan [project description]` next time!"

**Status Inquiries** (orientation, progress check):
- Pattern: "what was I working on?", "where are we?", "what's next?"
- **AUTO-STATUS**: Automatically check context and provide current status
- **TIP**: "💡 For detailed status tracking, try `/status` next time!"

**Quality Validation** (testing, linting, readiness):
- Pattern: "is this ready?", "check quality", "run tests"
- **AUTO-VALIDATE**: Automatically run tests, linters, and quality checks
- **TIP**: "💡 For comprehensive validation workflows, try `/check` next time!"

**Shipping** (commit and finalize):
- Pattern: "ready to commit", "ship this", "finalize changes"
- **AUTO-SHIP**: Automatically validate, test, and commit changes
- **TIP**: "💡 For structured shipping workflows, try `/ship` next time!"

**Troubleshooting** (bugs, issues, problems):
- Pattern: "debug this", "why is X failing?", "reproduce bug"
- **AUTO-DEBUG**: Automatically investigate and provide solutions
- **TIP**: "💡 For complex debugging sessions, try `/plan debug [issue description]` next time!"

### 💡 Smart Workflow Detection Protocol

**🤖 INTELLIGENT AUTO-ROUTING:**

**FOR every user request, I will:**
1. **ANALYZE** the request pattern to identify the most appropriate workflow
2. **AUTO-EXECUTE** using the detected workflow behavior
3. **PROVIDE** helpful tips about available workflow commands
4. **PROCEED** directly with implementation while maintaining quality standards
5. **SUGGEST** workflow commands for future use when appropriate

**✅ SMART BEHAVIORS:**
- **AUTOMATICALLY** detect and execute the most appropriate workflow
- **PROVIDE** educational tips about workflow commands
- **MAINTAIN** all quality standards and validation checkpoints
- **SUGGEST** structured workflows when they would be beneficial

### 🧠 Memory Management

**MANDATORY TRANSPARENCY**: Always inform users about memory operations.

**REQUIRED ACTIONS**:
- **ALWAYS** write project context, progress, and decisions to `.ai.local/` directory
- **ANNOUNCE** memory operations: "🧠 Writing [context/progress/decision] to memory..."
- **MANDATORY** for ALL workflow commands in `/commands/` (check, git:commit, next, plan, prompt, ship, status)
- **CREATE** `.ai.local/` structure as needed for persistent context tracking
- **UPDATE** memory files when project state changes significantly

**Memory Structure**:
- `.ai.local/context.md` - Current project understanding and architecture
- `.ai.local/progress.md` - Task completion status and next steps  
- `.ai.local/decisions.md` - Key technical and design decisions made

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

**⛔ MANDATORY Checkpoints** - YOU MUST STOP and validate at these points:

- **🧠 BEFORE ANY TASK EXECUTION** - analyze request and auto-select appropriate workflow
- **BEFORE marking any feature complete** - verify all requirements met
- **BEFORE starting any new component** - confirm architecture and plan
- **WHEN something feels wrong** - STOP immediately and reassess
- **BEFORE claiming done** - run complete validation checklist
- **ON any hook failure** - MUST fix before proceeding
- **💡 DURING EXECUTION** - provide helpful workflow tips when appropriate

**🚨 Hook Failures = BLOCKING** - YOU MUST:

1. **STOP immediately** when any hook fails
2. **FIX ALL failures** before any other action
3. **VERIFY fixes work** by re-running
4. **ONLY THEN continue** with original task
5. **NEVER ignore or bypass** hook failures

**🧪 MANDATORY Testing Strategy**:

- **Complex logic**: YOU MUST write tests BEFORE implementation
- **Simple CRUD**: YOU MUST write tests AFTER implementation
- **Performance-critical paths**: YOU MUST add benchmarks
- **ONLY skip tests for**: main functions, simple CLI parsing

**🤖 MANDATORY Test Automation** - YOU MUST use these tools:

- **E2E testing**: playwright
- **API validation**: fetch tool for HTTP requests
- **File-based testing**: filesystem MCP for file operations

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

## ✅ MANDATORY Completion Checklist

**YOU MUST verify ALL items before claiming task complete:**

- **ALL automated checks MUST be green** (lint, type check, format)
- **ALL tests MUST pass** (unit, integration, E2E as applicable)
- **End-to-end functionality MUST work** as specified
- **ALL old/obsolete code MUST be deleted** - no dead code
- **ALL changes MUST be documented** appropriately

[/CLAUDE]
