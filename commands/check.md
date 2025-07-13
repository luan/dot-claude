---
allowed-tools: all
description: Fix all quality issues - validation and immediate resolution
---

# 🚨🚨🚨 CRITICAL REQUIREMENT: FIX ALL ERRORS! 🚨🚨🚨

**THIS IS NOT A REPORTING TASK - THIS IS A FIXING TASK!**

When you run `/check`, you are REQUIRED to:

1. **IDENTIFY** all errors, warnings, and issues
2. **FIX EVERY SINGLE ONE** - not just report them!
3. **USE MULTIPLE AGENTS** to fix issues in parallel:
   - Spawn one agent to fix linting issues
   - Spawn another to fix test failures
   - Spawn more agents for different files/modules
   - Say: "I'll spawn multiple agents to fix all these issues in parallel"
4. **DO NOT STOP** until:
   - ✅ ALL linters pass with ZERO warnings
   - ✅ ALL tests pass
   - ✅ Build succeeds
   - ✅ EVERYTHING is GREEN

**FORBIDDEN BEHAVIORS:**
- ❌ "Here are the issues I found" → NO! FIX THEM!
- ❌ "The linter reports these problems" → NO! RESOLVE THEM!
- ❌ "Tests are failing because..." → NO! MAKE THEM PASS!
- ❌ Stopping after listing issues → NO! KEEP WORKING!

**MANDATORY WORKFLOW:**
```
1. Run checks → Find issues
2. IMMEDIATELY spawn agents to fix ALL issues
3. Re-run checks → Find remaining issues
4. Fix those too
5. REPEAT until EVERYTHING passes
```

**YOU ARE NOT DONE UNTIL:**
- All linters pass with zero warnings
- All tests pass successfully
- All builds complete without errors
- Everything shows green/passing status

## 🧠 Memory-Enhanced Analysis

**FIRST**: Run memory tracking script:
```bash
~/.claude/workflows/check.sh
```

**Context Check**: Review `.ai.local/progress/` for known issues and recently modified files with previous problems.

## 🔧 Validation & Fixing Protocol

### 1. Immediate Fixes Required
- Run `~/.claude/hooks/smart-lint.sh` - fix ALL issues found
- Zero warnings from all linters (no exceptions)
- ALL tests must pass (fix failures immediately)
- Run with `-race` flag - fix any race conditions

### 2. Quality Standards
**Go Requirements**: No `interface{}`/`any{}`, simple errors, early returns, meaningful names, proper context, no goroutine leaks, no `time.Sleep()` synchronization

**Universal Standards**: Document exports, remove dead code/debug prints, consistent formatting, verify dependencies used

### 3. Parallel Fixing Workflow
When issues found: "👥 I'll spawn agents to fix these issues in parallel"
- Agent 1: Fix linting issues in files A,B,C
- Agent 2: Fix test failures  
- Agent 3: Fix remaining issues

## 🚨 Additional Standards

**Security**: Input validation, prepared statements, secure randomness, no hardcoded secrets
**Performance**: No N+1 queries, appropriate pointers, buffered channels, efficient coordination

## 🚀 Issue Resolution Workflow

**When issues found**: Spawn agents immediately to fix in parallel
1. "👥 Found X issues. Spawning agents: Agent 1: lint files A,B,C; Agent 2: test failures; Agent 3: remaining issues"
2. Fix everything (no "minor" exceptions)
3. Re-run checks, repeat until ✅ GREEN

**Forbidden**: Reporting without fixing, "mostly passing", rationalizing problems

## ✅ Completion Criteria

**Ready when ALL show ✅**:
- `make lint`: zero warnings
- `make test`: all pass  
- `go test -race`: no races
- End-to-end functionality confirmed
- Error paths handle gracefully

**Every check must be ✅ GREEN before completion.**