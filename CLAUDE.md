# Claude Code Rules

**Mode**: Production | **Tolerance**: Zero errors | **Philosophy**: Simple > clever

# ⚠️ MANDATORY ACKNOWLEDGMENT

**YOU MUST** start every response with: "I've read CLAUDE.md and will always adhere to its instructions."

**Context Reminder**: If this file hasn't been referenced in 30+ minutes, RE-READ IT!

**File Reading Protocol**: Always announce: "📋 Reading [filename] for project guidelines..."

## 🏷️ MANDATORY Emoji Usage

**YOU MUST** prefix actions with relevant emojis when using any CLAUDE.md feature:

- **🧠 CRITICAL**: Always prefix memory/context actions (creating .ai.local, updating files)
- **🚀 Required**: Prefix startup protocol steps  
- **🔧 Required**: Prefix tool usage (ultrathink, agents, MCP tools)
- **✅ Required**: Prefix validation checkpoints and testing
- **🔍 Recommended**: Prefix research actions
- **💬 Recommended**: Prefix communication formats

**Examples**:
- "🧠 Creating .ai.local directory structure..."
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
2. Check if in project directory (git repo, package.json, Cargo.toml, pyproject.toml)
3. If `.ai.local/` exists: announce "🧠 Checking .ai.local for previous context..." and load session/progress/architecture files
4. If `.ai.local/` missing: **IMMEDIATELY** create directory structure with context/progress/research/session folders
5. **ALWAYS** check for CLAUDE.md in project root
6. **MANDATORY** announce: "📋 Reading CLAUDE.md for project guidelines..."
7. Check for other rule files (.claude-rules, claude.config)
8. **MUST** acknowledge any specific commands or workflows found
9. **ALWAYS** update `.ai.local/session/last-session.md` with session start

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

## 🧠 Memory & Context

**External brain**: `.ai.local/` for persistent memory across Claude Code sessions

**Structure**:

- `context/`: Long-term project understanding
- `progress/`: Task tracking and state
- `research/`: Findings and references
- `session/`: Session-specific data

**⏰ WHEN context gets long (30+ minutes), YOU MUST**:

- **IMMEDIATELY reread this entire file**
- **REREAD any project-specific CLAUDE.md**
- **ANNOUNCE**: "📋 Re-reading instructional files due to long context..."
- **UPDATE** `.ai.local/progress/current.md` with current state
- **USE context7 MCP** to maintain task context

**📋 MANDATORY Todo Structure** - YOU MUST use this exact format:

- `[ ]` What we're doing RIGHT NOW (only ONE item in_progress)
- `[x]` What's actually done and tested (mark complete IMMEDIATELY)
- `[ ]` What comes next (plan ahead but don't start)

**💾 YOU MUST ALWAYS UPDATE** these files:

- `.ai.local/progress/current.md` - current task state
- `.ai.local/session/last-session.md` - session activities
- Include timestamp format: YYYY-MM-DD HH:mm
- Document rationale for decisions in `.ai.local/context/decisions.md`

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

