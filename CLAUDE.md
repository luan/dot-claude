# Claude Code Rules

**Mode**: Production | **Tolerance**: Zero errors | **Philosophy**: Simple > clever

# ⚠️ MANDATORY ACKNOWLEDGMENT

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

**Examples**:

- "🧠 Loading project context for continuation..."
- "🚀 Starting protocol - checking for project files..."
- "🔧 Using rg instead of grep for search..."
- "✅ Running validation checkpoint before implementation..."

## 🔄 Workflow (STRICT)

**MANDATORY Sequence**: research → plan → implement  
**FORBIDDEN**: jump_to_code  
**REQUIRED Response**: "Let me research the codebase and create a plan before implementing."

### 🚀 Startup Protocol (MANDATORY)

**MUST DO ON EVERY SESSION:**

1. **ALWAYS** start with acknowledgment phrase
2. **INTENT RECOGNITION**: Analyze user's request to determine appropriate workflow
3. **PROJECT CONTEXT**: Check if in project directory and load context transparently
4. **WORKFLOW GUIDANCE**: Guide user to appropriate commands based on intent
5. **ALWAYS** check for CLAUDE.md in project root
6. **MANDATORY** announce: "📋 Reading CLAUDE.md for project guidelines..."

### 🎯 Intent Recognition & Workflow Guidance

**ANALYZE user requests and map to appropriate workflows:**

**Simple Changes** (quick fixes, small features):

- Pattern: "fix this", "add small feature", "update X"
- Workflow: Direct `/next` execution with minimal tracking
- Memory: Load context if available, light progress tracking
- **Educate**: "💡 For simple tasks like this, you can use `/next [task]` directly next time"

**Complex Projects** (multi-session, architectural):

- Pattern: "implement system", "build feature with X,Y,Z", "refactor entire X"
- Workflow: `/plan` for structured approach with full tracking
- Memory: Automatic setup and comprehensive progress management
- **Educate**: "💡 For complex projects like this, use `/plan [project description]` to get structured planning and progress tracking"

**Status Inquiries** (orientation, progress check):

- Pattern: "what was I working on?", "where are we?", "what's next?"
- Workflow: `/status` to load context and present current state
- Memory: Load all available context, present naturally
- **Educate**: "💡 Use `/status` anytime you want to get oriented or check progress"

**Quality Validation** (testing, linting, readiness):

- Pattern: "is this ready?", "check quality", "run tests"
- Workflow: `/check` for comprehensive validation
- Memory: Update with any fixes made, track problem areas
- **Educate**: "💡 Use `/check` to validate code quality and fix all issues before shipping"

**Shipping** (commit and finalize):

- Pattern: "ready to commit", "ship this", "finalize changes"
- Workflow: `/ship` for validation + commit process
- Memory: Save final state, record completion
- **Educate**: "💡 Use `/ship` when you're ready to validate everything and commit your changes"

**Troubleshooting** (bugs, issues, problems):

- Pattern: "debug this", "why is X failing?", "reproduce bug"
- Workflow: Investigation mode with systematic debugging
- Memory: Track investigation progress and findings
- **Educate**: "💡 For debugging, describe the issue and I'll help investigate systematically"

### 📚 User Education Protocol

**ALWAYS provide workflow education after completing tasks:**

1. **Identify the workflow used** for the user's request
2. **Explain the appropriate command** they could use next time
3. **Show the pattern** so they recognize it in future
4. **Encourage direct command usage** for efficiency

**Education format:**

```
💡 **WORKFLOW TIP:** For [type of task], you can use `[/command]` directly next time.
This helps you: [specific benefits]
```

**Examples:**

- "💡 **WORKFLOW TIP:** For simple fixes like this, you can use `/next fix the button styling` directly next time. This gets you faster execution with built-in validation."
- "💡 **WORKFLOW TIP:** For complex features like this, start with `/plan implement user dashboard` to get structured planning and progress tracking across sessions."
- "💡 **WORKFLOW TIP:** When you want to check if code is ready to ship, use `/ship` - it validates everything and commits when clean."

### 🧠 Transparent Memory Management

**Memory is an implementation detail - users never interact with .ai.local directly:**

- **Automatic Setup**: Complex projects get tracking without user knowing
- **Context Loading**: Status and continuation commands load context naturally
- **Progress Tracking**: Workflows save progress transparently when meaningful
- **Cross-Session**: Context preserved between sessions without user effort

**ALWAYS mention** `.ai.local`, memory files, or tracking setup to users.

### 🔧 Tools & Problem Solving (MANDATORY USAGE)

- **🤔 Complex problems**: YOU MUST use ultrathink - say "🤔 I need to ultrathink through this challenge"
- **👥 Parallel work**: YOU MUST spawn_agents for concurrent tasks
- **🚫 When stuck**: YOU MUST follow this exact sequence:
  1. **STOP** - Don't spiral into complex solutions
  2. **DELEGATE** - Consider spawning agents for parallel investigation
  3. **ULTRATHINK** - For complex problems, use sequential thinking
  4. **STEP BACK** - Re-read the requirements
  5. **SIMPLIFY** - The simple solution is usually correct
  6. **ASK** - Present options for guidance

**👥 Agent examples**:

- "I'll spawn agents to tackle different aspects of this problem"
- "I'll have an agent investigate the database schema while I analyze the API structure"
- "One agent writes tests while another implements features"

**⚙️ MCP Servers**:

- `sequential_thinking`: Break down complex problems into step-by-step reasoning
- `filesystem`: Navigate and explore codebase structure, read/write files
- `context7`: Maintain context across long conversations and complex tasks
- `magic`: Swiss-army knife for various automation tasks

## 🔍 Research Tools

**Primary**: filesystem for codebase exploration  
**First action**: Look for CLAUDE.md and project-specific rules

**🔧 MANDATORY Tool Preferences** (NEVER use alternatives):

- **ALWAYS** use `rg` instead of `grep`
- **ALWAYS** use `fd` instead of `find`
- **ALWAYS** use `eza` instead of `ls` for directory listings
- **ALWAYS** use `bat` instead of `cat` when syntax highlighting helps

**🌐 MANDATORY Web Research Tools**:

- **Playwright**: YOU MUST use for browser automation and testing
- **browser_tools**: YOU MUST use for quick browser interactions
- **fetch**: YOU MUST use for API testing and validation

## ✅ Validation & Testing

**⛔ MANDATORY Checkpoints** - YOU MUST STOP and validate at these points:

- **BEFORE marking any feature complete** - verify all requirements met
- **BEFORE starting any new component** - confirm architecture and plan
- **WHEN something feels wrong** - STOP immediately and reassess
- **BEFORE claiming done** - run complete validation checklist
- **ON any hook failure** - MUST fix before proceeding

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

### 🚫 ABSOLUTELY FORBIDDEN - NEVER DO THESE

- **NEVER use generic types** like `any`, `object`, `unknown` without constraint
- **NEVER use sleep() or busy waiting** - use proper async patterns
- **NEVER mix old and new code patterns** in same file
- **NEVER create migration/compatibility layers** - clean refactor instead
- **NEVER use versioned names** like `handleSubmitV2` - replace old code
- **NEVER create complex error hierarchies** - use simple, flat errors
- **NEVER leave TODOs in final code** - complete or remove before commit

### ✅ ABSOLUTELY REQUIRED - YOU MUST ALWAYS

- **DELETE old code** when replacing with new implementation
- **USE meaningful, descriptive names** for all variables, functions, classes
- **USE early returns** to reduce nesting and improve readability
- **KEEP errors simple** - clear message, relevant context only
- **WRITE appropriate tests** for all business logic
- **FOLLOW language idioms** - write idiomatic code for the language

### 🔒 Security (ZERO TOLERANCE)

**YOU MUST ALWAYS**:

- **VALIDATE ALL INPUTS** - never trust user data
- **USE secure randomness** - crypto.randomBytes(), not Math.random()
- **USE prepared statements** for all database queries
- **SQL RULE**: NEVER EVER concatenate SQL strings - ONLY prepared statements

**Testing**: Use playwright for web vulnerabilities, fetch for API security

### ⚡ Performance (STRICT RULES)

**YOU MUST**:

- **MEASURE BEFORE OPTIMIZE** - profile first, optimize second
- **NO premature optimization** - get it working correctly first
- **BENCHMARK before claiming faster** - prove performance improvements
- **Load testing**: Use fetch for APIs, playwright for browser

## 🧠 Context Management

**⏰ WHEN context gets long (30+ minutes), YOU MUST**:

- **IMMEDIATELY reread this entire file**
- **REREAD any project-specific CLAUDE.md**
- **ANNOUNCE**: "📋 Re-reading instructional files due to long context..."
- **USE context7 MCP** to maintain task context

**📋 MANDATORY Todo Structure** - YOU MUST use this exact format:

- `[ ]` What we're doing RIGHT NOW (only ONE item in_progress)
- `[x]` What's actually done and tested (mark complete IMMEDIATELY)
- `[ ]` What comes next (plan ahead but don't start)

## 💬 Communication

**📋 File acknowledgment**: "📋 Reading [filename] for [purpose]..."

**📊 Progress format**: "✓/✗ Status (details)"

**💡 Improvement format**: "The current approach works, but I notice [observation]. Would you like me to [specific improvement]?"

**🤔 When choosing**: "I see two approaches: [A] vs [B]. Which do you prefer?"

## 📝 Git Conventions

**For git commits**: Use the `/git:commit` command which handles all git operations according to Claude Code standards.

**When to commit**: Commit when appropriate (task completion, significant milestones) or when user requests it.

## ✅ MANDATORY Completion Checklist

**YOU MUST verify ALL items before claiming task complete:**

- **ALL automated checks MUST be green** (lint, type check, format)
- **ALL tests MUST pass** (unit, integration, E2E as applicable)
- **End-to-end functionality MUST work** as specified
- **ALL old/obsolete code MUST be deleted** - no dead code
- **ALL changes MUST be documented** appropriately
