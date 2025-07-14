# Claude Code Workflow Documentation

This document explains the intelligent workflow system for Claude Code. The system recognizes your intent and guides you to the right approach automatically.

## Core Philosophy

**Memory is invisible** - You focus on what you want to accomplish, not on managing context files. The system handles progress tracking transparently based on the complexity of your request.

## Framework Overview

### Intelligent Auto-Detection

Claude Code automatically detects the most appropriate workflow based on your request patterns. You don't need to memorize commands - the system analyzes your intent and executes the right workflow while providing helpful tips for future use.

### Key Features

- **🤖 Smart Routing**: Automatically detects workflow type from natural language
- **🧠 Autonomous Memory**: Manages project context without user intervention
- **✅ Quality First**: Built-in validation checkpoints throughout workflows
- **🚀 Progressive Enhancement**: Suggests structured commands as you work
- **🔧 Tool Integration**: Seamlessly uses ultrathink, agents, and MCP tools

### Workflow Principles

1. **Research → Plan → Implement**: Never skip directly to implementation
2. **Validation at Every Step**: Quality checkpoints are mandatory
3. **Hook Failures are Blocking**: Must fix all failures before proceeding
4. **Tests are Required**: Complex logic needs tests before implementation
5. **Clean Code Only**: No TODOs, dead code, or versioned names in final code

## Available Workflows

### 🔍 Getting Oriented: `/status`

**When to use:** Start of session, checking progress, "what was I working on?"

**What it does:**
- Loads project context automatically
- Shows current progress and next steps  
- Presents clear actionable options
- Suggests appropriate workflows for your situation

**Example usage:**
```
/status
```

### 🚀 Simple Changes: `/next [task]`

**When to use:** Quick fixes, small features, straightforward implementations

**What it does:**
- Research → Plan → Implement workflow
- Minimal progress tracking
- Focused execution with validation
- Works great for single-session tasks

**Example usage:**
```
/next fix the login button styling
/next add validation to the email field
/next update the API endpoint to handle errors
```

### 📋 Complex Projects: `/plan [project]`

**When to use:** Multi-session projects, new systems, major refactoring

**What it does:**
- Analyzes project complexity and scope
- Creates structured implementation plan with phases
- Sets up automatic progress tracking across sessions
- Breaks down work into manageable tasks
- Provides session-to-session continuity

**Example usage:**
```
/plan implement user authentication system with JWT
/plan refactor the entire data layer to use TypeScript
/plan build a real-time notification system
```

### ✅ Quality Validation: `/check`

**When to use:** "Is this ready?", testing, quality verification

**What it does:**
- Runs comprehensive quality checks (linting, tests, build)
- Fixes ALL issues found (not just reports them)
- Uses multiple agents for parallel issue resolution
- Zero tolerance - everything must pass
- Updates progress with fixes made

**Example usage:**
```
/check
```

### 🚀 Ready to Ship: `/ship`

**When to use:** "Ready to commit", finalizing completed work

**What it does:**
- Complete validation workflow (same as `/check`)
- Automatic commit with meaningful messages
- Progress tracking of completion
- Confirms successful shipping

**Example usage:**
```
/ship
```

### 📝 Manual Commit: `/git:commit`

**When to use:** You want control over commit timing and message

**What it does:**
- Focused git workflow
- Uses project context for better commit messages
- Handles pre-commit hooks
- Pure git operation without extra validation

**Example usage:**
```
/git:commit
```

## How Intent Recognition Works

### Automatic Workflow Detection

The system uses pattern matching to identify your intent and automatically execute the appropriate workflow:

```
User Request → Pattern Analysis → Auto-Execute Workflow → Provide Tips
```

### Pattern Recognition

### Simple Changes
- **Patterns:** "fix this", "add small feature", "update X", "change Y to Z"
- **Behavior:** Auto-executes task implementation
- **Response:** "Let me research the codebase and create a plan before implementing."
- **Tip:** "💡 For organized workflows, try `/next [description]` next time!"

### Complex Projects  
- **Patterns:** "implement system", "build feature with X,Y,Z", "refactor entire X", "create new architecture"
- **Behavior:** Auto-creates comprehensive plan and begins implementation
- **Response:** Analyzes scope, creates phased plan, sets up tracking
- **Tip:** "💡 For structured project management, try `/plan [project description]` next time!"

### Status Inquiries
- **Patterns:** "what was I working on?", "where are we?", "what's next?", "show progress"
- **Behavior:** Auto-checks context and provides current status
- **Response:** Loads memory, shows progress, suggests next steps
- **Tip:** "💡 Try `/status` to check status next time!"

### Quality Validation
- **Patterns:** "is this ready?", "check quality", "run tests", "validate code"  
- **Behavior:** Auto-runs tests, linters, and quality checks
- **Response:** Executes all checks, fixes issues found
- **Tip:** "💡 For comprehensive validation workflows, try `/check` next time!"

### Shipping
- **Patterns:** "ready to commit", "ship this", "finalize changes", "complete the feature"
- **Behavior:** Auto-validates, tests, and commits changes
- **Response:** Full validation → fix issues → commit with message
- **Tip:** "💡 For structured commit workflows, try `/ship` next time!"

### Troubleshooting
- **Patterns:** "debug this", "why is X failing?", "reproduce bug", "investigate error"
- **Behavior:** Auto-investigates and provides solutions
- **Response:** Systematic debugging with root cause analysis
- **Tip:** "💡 For complex debugging sessions, try `/plan debug [issue description]` next time!"

## Natural Workflow Progressions

### Typical Simple Task
```
User: "Fix the broken navigation menu"
→ System recognizes simple change
→ Executes /next workflow automatically
→ Research → Plan → Implement → Validate
```

### Typical Complex Project
```
User: "Build a user dashboard with analytics"
→ System recognizes complex project  
→ Suggests /plan workflow
→ Creates structured plan with phases
→ Sets up progress tracking
→ User continues with /next for individual tasks
→ Uses /status to check progress
→ Completes with /ship
```

### Typical Session Continuation
```
User: "What should I work on?"
→ System loads context automatically
→ Presents current progress and options
→ Suggests next logical steps
→ User continues with appropriate workflow
```

## Memory Management (Transparent)

### Autonomous Memory System

Claude Code has full autonomy over the `.ai.local/` directory to organize project memory intelligently:

#### Memory Principles
- **🧠 AUTONOMY**: Decides what to remember, when, and how to structure it
- **📝 TRANSPARENCY**: Announces significant operations: "🧠 Writing [type] to memory..."
- **🔄 EVOLUTION**: Structure adapts with project understanding
- **🎯 CONTEXT-AWARE**: Organizes by meaning and relevance
- **💡 INTELLIGENT**: Learns what's important for each unique project

#### When Memory is Written
- Starting new features or major tasks
- Making important technical decisions
- Discovering project patterns or conventions
- Solving complex problems
- Learning from mistakes or insights
- Finding useful resources or documentation

#### Flexible Organization
The system creates appropriate structures based on project needs:
- Architecture decisions and rationale
- Feature implementations and progress
- Debugging contexts and solutions
- Project-specific patterns
- Research findings
- Custom structures as needed

**You never interact with memory files directly** - they're managed transparently based on your workflow needs.

## Best Practices

### Starting a Session
1. Begin with acknowledgment: "I've read CLAUDE.md and will always adhere to its instructions."
2. Run `/status` to get oriented (or just describe what you want)
3. Let the system auto-detect and execute the appropriate workflow
4. Focus on describing what you want to accomplish

### During Work  
1. Trust the auto-detection - just describe your needs naturally
2. Use structured commands when you want more control
3. Watch for workflow tips to learn available commands
4. Use `/check` frequently to catch issues early
5. Let the system handle progress tracking automatically

### Quality Checkpoints
**⛔ MANDATORY validation points:**
- Before marking any feature complete
- Before starting new components
- When something feels wrong
- Before claiming done
- On any hook failure (MUST fix immediately)

### Problem Solving Tools
- **Complex Problems**: "🤔 I need to ultrathink through this challenge"
- **Parallel Work**: "👥 I'll spawn agents to tackle different aspects"
- **When Stuck**: STOP → delegate/ultrathink → simplify → ask for guidance

### Finishing Work
1. Use `/ship` when ready to finalize and commit
2. The system will validate everything before committing
3. ALL automated checks MUST be green
4. ALL tests MUST pass
5. End-to-end functionality MUST work as specified
6. Complex projects automatically save progress for next session

### Getting Unstuck
1. Use `/status` to understand current state
2. Ask natural questions about what you're trying to accomplish
3. Let the system suggest the appropriate approach
4. Use ultrathink for complex reasoning
5. Spawn agents for parallel investigation

## Command Reference

| Command | Purpose | Best For | Auto-Triggered By |
|---------|---------|----------|-------------------|
| `/status` | Get oriented, check progress | Starting sessions, checking status | "what was I working on?", "where are we?" |
| `/next [task]` | Execute specific task | Simple changes, focused work | "fix this", "add feature", "update X" |
| `/plan [project]` | Structure complex work | Multi-session projects, major features | "implement system", "build X with Y,Z" |
| `/check` | Validate quality | Before shipping, catching issues | "is this ready?", "run tests" |
| `/ship` | Validate and commit | Finalizing completed work | "ready to commit", "ship this" |
| `/git:commit` | Manual commit control | When you want commit control | Direct command only |

## Tool Ecosystem

### Core Tools
- **File Operations**: Read, Write, Edit, MultiEdit
- **Search**: Grep (rg), Glob (fd), Task (complex searches)
- **Code Navigation**: LS, NotebookRead/Edit
- **Version Control**: Bash (git commands), TodoWrite
- **Web Tools**: WebFetch, WebSearch, playwright

### MCP Servers
- **sequential_thinking**: For complex problem solving (ultrathink)
- **context7**: Library documentation lookup
- **magic**: UI component generation and refinement
- **playwright**: Browser automation and testing

### Preferred Command Line Tools
- Use `rg` instead of grep
- Use `fd` instead of find  
- Use `eza` instead of ls
- Use `bat` for syntax highlighting

## Code Standards Summary

### 🚫 Forbidden
- Generic types without constraints (`any`, `object`)
- sleep() or busy waiting
- Mixing code patterns in same file
- Migration layers (clean refactor instead)
- Versioned names (`handleSubmitV2`)
- Complex error hierarchies
- TODOs in final code

### ✅ Required
- Delete old code when replacing
- Meaningful, descriptive names
- Early returns to reduce nesting
- Simple errors with clear messages
- Tests for all business logic
- Follow language idioms
- Validate all inputs
- Use secure randomness
- Prepared statements for SQL

## The system is designed to be invisible - focus on your work, not on managing the tools.