# Claude Code Framework

An intelligent workflow and quality assurance framework for Claude AI that provides structured development workflows, autonomous memory management, and automated quality validation.

## What is Claude Code?

Claude Code is a configuration framework that enhances Claude AI's capabilities for software development. It provides:

- **🤖 Intelligent Workflows**: Auto-detects user intent and executes appropriate development workflows
- **🧠 Autonomous Memory**: Manages project context and progress across sessions transparently
- **✅ Quality Assurance**: Automated linting, testing, and validation with blocking enforcement
- **🔧 Multi-Language Support**: Works with Go, Python, JavaScript/TypeScript, Rust, Nix, Shell, and Tilt
- **📝 Git Integration**: Automated commit workflows with meaningful messages

## Core Philosophy

**Quality First** - All changes must pass automated validation before completion. Hook failures are blocking and must be fixed immediately.

**Memory is Invisible** - You focus on what you want to accomplish, not on managing context files. The system handles progress tracking transparently.

**Simple > Clever** - Code should be straightforward, well-tested, and follow established patterns.

## Installation & Setup

The framework is installed in your `~/.claude/` directory with these key components:

```
~/.claude/
├── CLAUDE.md              # Core behavioral instructions
├── README.md              # This documentation
├── settings.json          # Permissions and hook configurations
├── commands/              # Workflow command definitions
│   ├── check.md          # Quality validation workflow
│   ├── commit.md         # Git commit workflow
│   ├── plan.md           # Complex project planning
│   └── task.md           # Simple task execution
├── hooks/                # Quality assurance scripts
│   ├── go.sh            # Go linting and testing
│   ├── python.sh        # Python validation
│   ├── javascript.sh    # JS/TS validation
│   └── ...              # Other language hooks
├── projects/             # Session persistence
└── todos/               # Task management
```

## Framework Architecture

### Intelligent Auto-Detection

Claude Code automatically detects the most appropriate workflow based on your request patterns. You don't need to memorize commands - the system analyzes your intent and executes the right workflow while providing helpful tips for future use.

### Quality Assurance System

**Automated Validation**: Every file edit triggers language-specific linting and testing
**Blocking Enforcement**: All hook failures must be fixed before proceeding
**Project-Aware**: Supports `.claude-hooks-config.sh` for project-specific settings
**Multi-Language**: Go, Python, JavaScript/TypeScript, Rust, Nix, Shell, and Tilt support

### Memory Management

**Autonomous**: Manages `.ai.local/` directory for project context automatically
**Session Persistence**: Maintains progress across multiple sessions
**Privacy-First**: Automatically gitignores memory files
**Intelligent**: Learns project patterns and conventions over time

### Workflow Principles

1. **Research → Plan → Implement**: Never skip directly to implementation
2. **Validation at Every Step**: Quality checkpoints are mandatory
3. **Hook Failures are Blocking**: Must fix all failures before proceeding
4. **Tests are Required**: Complex logic needs tests before implementation
5. **Clean Code Only**: No TODOs, dead code, or versioned names in final code

## Available Workflows

### 🚀 Task Execution: `/task [description]`

**When to use:** Simple changes, bug fixes, small features

**What it does:**
- Research → Plan → Implement workflow
- Automatic quality validation
- Progress tracking for complex tasks
- Handles both simple and multi-step work

**Example usage:**
```
/task fix the login button styling
/task add validation to the email field
/task update the API endpoint to handle errors
```

**Auto-triggered by:** "fix this", "add feature", "update X", "change Y to Z"

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

**Auto-triggered by:** "implement system", "build feature with X,Y,Z", "refactor entire X"

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

**Auto-triggered by:** "is this ready?", "run tests", "validate code"

### 📝 Git Workflow: `/commit`

**When to use:** Ready to commit changes, finalizing work

**What it does:**
- Automatic quality validation (runs `/check` first)
- Analyzes changes and creates meaningful commit messages
- Handles pre-commit hooks and validation
- Follows project's commit message conventions
- Confirms successful commit

**Example usage:**
```
/commit
```

**Auto-triggered by:** "ready to commit", "ship this", "finalize changes"

## Quality Assurance Hooks

### Automated Validation

Every file edit triggers language-specific quality checks:

**Go Projects**: `go fmt`, `go vet`, `golangci-lint`, `go test`
**Python Projects**: `black`, `ruff`, `mypy`, `pytest`
**JavaScript/TypeScript**: `eslint`, `prettier`, `tsc`, `npm test`
**Rust Projects**: `cargo fmt`, `cargo clippy`, `cargo test`
**Shell Scripts**: `shellcheck`, `shfmt`

### Hook Configuration

Projects can customize validation with `.claude-hooks-config.sh`:

```bash
# Project-specific hook configuration
GO_LINT_ENABLED=true
PYTHON_MYPY_ENABLED=false
JAVASCRIPT_ESLINT_CONFIG=".eslintrc.json"
```

### Blocking Enforcement

**🚨 Hook failures are BLOCKING** - you must:
1. Stop immediately when any hook fails
2. Fix ALL failures before continuing
3. Verify fixes work by re-running
4. Never ignore or bypass failures

## Intent Recognition System

### Automatic Workflow Detection

The system uses pattern matching to identify your intent and automatically execute the appropriate workflow:

```
User Request → Pattern Analysis → Auto-Execute Workflow → Provide Educational Tips
```

### Pattern Recognition Examples

**Simple Tasks** → Auto-executes `/task`
- "fix this", "add validation", "update styling", "change X to Y"

**Complex Projects** → Auto-suggests `/plan`
- "implement system", "build feature with X,Y,Z", "refactor entire X"

**Quality Checks** → Auto-executes `/check`
- "is this ready?", "run tests", "validate code", "check quality"

**Git Operations** → Auto-executes `/commit`
- "ready to commit", "ship this", "finalize changes"

**Status Inquiries** → Auto-loads context
- "what was I working on?", "where are we?", "what's next?"

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
| `/task [description]` | Execute specific task | Simple changes, bug fixes, features | "fix this", "add feature", "update X" |
| `/plan [project]` | Structure complex work | Multi-session projects, major features | "implement system", "build X with Y,Z" |
| `/check` | Validate quality | Before committing, catching issues | "is this ready?", "run tests" |
| `/commit` | Validate and commit | Finalizing completed work | "ready to commit", "ship this" |

## Tool Ecosystem

### Core Development Tools
- **File Operations**: Read, Write, Edit, MultiEdit for code changes
- **Search**: Grep (rg), Glob (fd), Task for complex searches
- **Code Navigation**: LS, NotebookRead/Edit for project exploration
- **Version Control**: Bash (git commands), TodoWrite for task management
- **Web Tools**: WebFetch, WebSearch, playwright for web interactions

### MCP Server Integration
- **sequential_thinking**: Complex problem solving with ultrathink
- **context7**: Library documentation and API reference lookup
- **magic**: UI component generation and refinement
- **playwright**: Browser automation and end-to-end testing

### Preferred CLI Tools
- `rg` (ripgrep) instead of grep - faster search
- `fd` instead of find - better file discovery
- `eza` instead of ls - enhanced directory listing
- `bat` instead of cat - syntax highlighting
- `jq` for JSON processing

### Language-Specific Tools
- **Go**: `go fmt`, `go vet`, `golangci-lint`, `go test`
- **Python**: `black`, `ruff`, `mypy`, `pytest`
- **JavaScript/TypeScript**: `eslint`, `prettier`, `tsc`, `npm test`
- **Rust**: `cargo fmt`, `cargo clippy`, `cargo test`
- **Shell**: `shellcheck`, `shfmt`

## Code Standards & Best Practices

### 🚫 Forbidden Practices
- Generic types without constraints (`any`, `object`, `unknown`)
- Sleep() or busy waiting (use proper async patterns)
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
- Validate all inputs (never trust user data)
- Use secure randomness (crypto.randomBytes(), not Math.random())
- Use prepared statements for database queries

### 🔒 Security Requirements
- Validate all inputs at boundaries
- Use secure randomness for cryptographic operations
- Never concatenate SQL strings - use prepared statements
- Never commit secrets or keys to repository
- Never introduce code that logs or exposes sensitive data

### ⚡ Performance Guidelines
- Profile before optimizing
- No premature optimization (get it working correctly first)
- Benchmark before claiming performance improvements
- Use appropriate load testing for performance-critical code

### 🧪 Testing Strategy
- **Complex logic**: Write tests BEFORE implementation
- **Simple CRUD**: Write tests AFTER implementation
- **Performance-critical paths**: Add benchmarks
- **Skip tests only for**: main functions, simple CLI parsing

## Getting Started

### First Session
1. Claude automatically acknowledges CLAUDE.md instructions
2. Describe what you want to accomplish naturally
3. System auto-detects appropriate workflow and provides tips
4. Focus on your work - the system handles quality and progress

### Development Flow
1. **Research** → **Plan** → **Implement** (never skip to implementation)
2. Automated quality validation after every change
3. Fix any hook failures immediately (blocking)
4. Use `/check` frequently to catch issues early
5. Commit with `/commit` when ready

### Quality Checkpoints
**MANDATORY validation points where you must stop and verify:**
- Before marking any feature complete
- Before starting new components
- When something feels wrong
- Before claiming done
- On any hook failure (MUST fix immediately)

## Project Structure

The framework integrates with your existing projects and adds:

```
your-project/
├── .ai.local/              # Autonomous memory (gitignored)
│   ├── context.md         # Project understanding
│   ├── progress.md        # Current progress
│   └── decisions.md       # Architecture decisions
├── .claude-hooks-config.sh # Project-specific hook config
└── ... (your existing code)
```

## Philosophy

**The system is designed to be invisible** - focus on your work, not on managing the tools. Claude Code handles quality, memory, and progress automatically while you concentrate on solving problems and building features.

**Quality is non-negotiable** - every change must pass validation. This prevents technical debt and ensures maintainable code.

**Simple is better than clever** - code should be straightforward, well-tested, and follow established patterns rather than being overly complex or "smart".