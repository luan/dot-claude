# Claude Code Workflow Documentation

This document explains the intelligent workflow system for Claude Code. The system recognizes your intent and guides you to the right approach automatically.

## Core Philosophy

**Memory is invisible** - You focus on what you want to accomplish, not on managing context files. The system handles progress tracking transparently based on the complexity of your request.

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

The system automatically recognizes patterns in your requests:

### Simple Changes
- **Patterns:** "fix this", "add small feature", "update X"
- **Auto-suggests:** `/next` workflow with minimal tracking

### Complex Projects  
- **Patterns:** "implement system", "build feature with X,Y,Z", "refactor entire X"
- **Auto-suggests:** `/plan` workflow with full tracking

### Status Inquiries
- **Patterns:** "what was I working on?", "where are we?", "what's next?"
- **Auto-suggests:** `/status` workflow

### Quality Validation
- **Patterns:** "is this ready?", "check quality", "run tests"  
- **Auto-suggests:** `/check` workflow

### Shipping
- **Patterns:** "ready to commit", "ship this", "finalize changes"
- **Auto-suggests:** `/ship` workflow

### Troubleshooting
- **Patterns:** "debug this", "why is X failing?", "reproduce bug"
- **Auto-suggests:** Investigation workflow with systematic debugging

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

The system handles context preservation automatically:

- **Simple tasks:** Minimal tracking, focuses on execution
- **Complex projects:** Full progress tracking across sessions
- **Session continuity:** Context preserved between sessions without user effort
- **Cross-session handoff:** Pick up exactly where you left off

**You never interact with memory files directly** - they're managed transparently based on your workflow needs.

## Best Practices

### Starting a Session
1. Run `/status` to get oriented
2. Let the system guide you to the appropriate workflow
3. Focus on describing what you want to accomplish

### During Work  
1. Use `/next` for individual tasks
2. Use `/check` frequently to catch issues early
3. Let the system handle progress tracking automatically

### Finishing Work
1. Use `/ship` when ready to finalize and commit
2. The system will validate everything before committing
3. Complex projects automatically save progress for next session

### Getting Unstuck
1. Use `/status` to understand current state
2. Ask natural questions about what you're trying to accomplish
3. Let the system suggest the appropriate approach

## Command Reference

| Command | Purpose | Best For |
|---------|---------|----------|
| `/status` | Get oriented, check progress | Starting sessions, checking status |
| `/next [task]` | Execute specific task | Simple changes, focused work |
| `/plan [project]` | Structure complex work | Multi-session projects, major features |
| `/check` | Validate quality | Before shipping, catching issues |
| `/ship` | Validate and commit | Finalizing completed work |
| `/git:commit` | Manual commit control | When you want commit control |

## The system is designed to be invisible - focus on your work, not on managing the tools.