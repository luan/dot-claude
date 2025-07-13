---
allowed-tools: all
description: Execute production-quality implementation with strict standards
---

🚨 **CRITICAL WORKFLOW - NO SHORTCUTS!** 🚨

You are tasked with implementing: $ARGUMENTS

**MANDATORY SEQUENCE:**
1. 🧠 **MEMORY CHECK** - "📋 Checking .ai.local for project context and previous progress..."
2. 🔍 **RESEARCH FIRST** - "Let me research the codebase and create a plan before implementing"
3. 📋 **PLAN** - Present a detailed plan and verify approach
4. ✅ **IMPLEMENT** - Execute with validation checkpoints

**🧠 MANDATORY MEMORY PROTOCOL:**
- **ALWAYS** check if `.ai.local/` exists and load context
- **FIRST** read `.ai.local/context/project-info.json` if available
- **CHECK** `.ai.local/progress/current.md` for ongoing tasks
- **REVIEW** `.ai.local/session/current-session.md` for recent activity
- **ANNOUNCE**: "📋 Loading project knowledge from .ai.local..."

**YOU MUST SAY:** "📋 Checking .ai.local for project context... Let me research the codebase and create a plan before implementing."

For complex tasks, say: "🤔 Let me ultrathink about this architecture before proposing a solution."

**USE MULTIPLE AGENTS** when the task has independent parts:
"👥 I'll spawn agents to tackle different aspects of this problem"

Consult ~/.claude/CLAUDE.md IMMEDIATELY and follow it EXACTLY.

**FIRST**: Run memory initialization script:
```bash
~/.claude/workflows/next.sh "$ARGUMENTS"
```

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

🛑 **HOOKS ARE WATCHING** 🛑
The smart-lint.sh hook will verify EVERYTHING. It will:
- Block operations if you ignore linter warnings
- Track repeated violations
- Prevent commits with any issues
- Force you to fix problems before proceeding

**Completion Standards (NOT NEGOTIABLE):**
- The task is NOT complete until ALL linters pass with zero warnings (golangci-lint with all checks enabled)
- ALL tests must pass with meaningful coverage of business logic (skip testing main(), simple CLI parsing, etc.)
- The feature must be fully implemented and working end-to-end
- No placeholder comments, TODOs, or "good enough" compromises

**Reality Checkpoints (MANDATORY):**
- After EVERY 3 file edits: Run linters
- After implementing each component: Validate it works
- Before saying "done": Run FULL test suite
- If hooks fail: STOP and fix immediately

**Code Evolution Rules:**
- This is a feature branch - implement the NEW solution directly
- DELETE old code when replacing it - no keeping both versions
- NO migration functions, compatibility layers, or deprecated methods
- NO versioned function names (e.g., processDataV2, processDataNew)
- When refactoring, replace the existing implementation entirely
- If changing an API, change it everywhere - no gradual transitions

**Language-Specific Quality Requirements:**

**For ALL languages:**
- Follow established patterns in the codebase
- Use language-appropriate linters at MAX strictness
- Delete old code when replacing functionality
- No compatibility shims or transition helpers

**For Go specifically:**
- Absolutely NO interface{} or any{} - use concrete types or properly defined interfaces
- Simple, focused interfaces following the Interface Segregation Principle (prefer many small interfaces over large ones)
- Error handling must use simple error returns or well-established patterns (NO custom error structs unless absolutely necessary)
- Avoid unnecessary type assertions and interface casting - if you need to cast, reconsider your design
- Follow standard Go project layout (cmd/, internal/, pkg/ where appropriate)
- NO time.Sleep() or busy waits - use channels and message passing for synchronization

## 🔧 Execution Protocol

**Core Workflow**:

1. Research codebase and create plan
2. Implement with validation checkpoints
3. Run linters after each significant edit
4. Test functionality end-to-end
5. Complete when all checks pass

**Reality Checkpoints**: Validate frequently, fix hook failures immediately.

**For complex tasks**: Use ultrathink and spawn agents for parallel work.

