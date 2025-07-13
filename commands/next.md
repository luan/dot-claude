---
allowed-tools: all
description: Intelligent task execution with context-aware workflow
---

# 🧠 Intelligent Context Analysis

**IF NO TASK SPECIFIED** ($ARGUMENTS is empty), analyze context first:

1. **Memory Context Check**:
   - Read `.ai.local/session/current-session.md` for recent activity
   - Check `.ai.local/progress/` for todo lists and current tasks
   - Review recent file changes for patterns

2. **Task Detection Heuristics**:
   - **Sequential Work**: If task N completed → suggest task N+1 from todo list
   - **Error Recovery**: If previous task failed → suggest fix/retry
   - **Project Patterns**: If in optimization project → suggest next optimization
   - **Standard Workflows**: After research → suggest implementation

3. **Smart Routing**:
   - **Obvious next task detected** → Proceed with next task: "🧠 Context detected: [task]. Executing..."
   - **Multiple likely options** → Present choices: "🤔 Detected options: 1)[task1] 2)[task2] Choose or specify:"
   - **Ambiguous context** → Require description: "🚨 Please specify task: `/next [description]`"

# 🚀 Core Implementation Workflow

**Task Specified**: $ARGUMENTS

**Required Sequence**: research → plan → implement (consult CLAUDE.md for full standards)

**Memory Protocol**:

- Load context from `.ai.local/`
- Announce: "📋 Checking project context..."
- For complex tasks: "🤔 Let me ultrathink through this challenge"
- For parallel work: "👥 I'll spawn agents to tackle different aspects"

## ✅ Quality Standards

**Hook Validation**: smart-lint.sh enforces all quality checks - fix issues immediately when detected.

**Completion Requirements**:

- All linters pass with zero warnings
- All tests pass with meaningful coverage
- Feature works end-to-end
- No placeholder/TODO code remains
- Old code deleted when replaced

**Implementation Standards**:

- Follow established codebase patterns
- Use language-appropriate linters at max strictness
- Delete old code when replacing (no versioned names like `funcV2`)
- No migration/compatibility layers - clean refactor instead

**Language-Specific Notes** (reference CLAUDE.md for full details):

- **Go**: No `interface{}`/`any{}`, use channels not `time.Sleep()`, simple error handling
- **All**: Meaningful names, early returns, simple errors, appropriate tests

## 🔧 Execution Protocol

**Core Workflow**:

1. Research codebase and create plan
2. Implement with validation checkpoints
3. Run linters after each significant edit
4. Test functionality end-to-end
5. Complete when all checks pass

**Reality Checkpoints**: Validate frequently, fix hook failures immediately.

**For complex tasks**: Use ultrathink and spawn agents for parallel work.

