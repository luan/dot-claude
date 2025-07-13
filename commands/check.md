---
allowed-tools: all
description: Fix all quality issues - validation and immediate resolution
---

# ✅ Quality Validation & Fixing

**Core Principle**: This is a FIXING task, not reporting. Fix everything found.

## 🧠 Memory-Enhanced Analysis

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