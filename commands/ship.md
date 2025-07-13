---
allowed-tools: all
description: Complete validation and commit workflow for shipping
---

# 🚀 Ship Command

**Command**: `/ship`

## Purpose

Complete validation workflow - ensure code quality, run tests, and commit changes when everything is ready.

## Instructions for Claude

When the user runs `/ship`, you MUST follow these steps exactly:

### 1. 🔍 Pre-Ship Assessment

**IMMEDIATELY** assess current state:

```bash
git status
git diff --name-only HEAD~1..HEAD
```

**Determine scope of changes:**

- New features implemented
- Bug fixes applied
- Refactoring completed
- Configuration changes

**FIRST**: Run memory tracking script:
```bash
~/.claude/workflows/ship.sh
```

### 3. 🧠 Transparent Progress Tracking

**If complex changes**, automatically save progress context:

- Update progress state with completed work
- Record decisions made during implementation
- Note any important architectural changes
- Track file changes and their purpose

**Handle transparently** - user doesn't need to know about memory files.

### 4. 🎯 Smart Commit Decision

**Analyze readiness for commit:**

**IF everything is clean:**

- Automatically proceed to `/git:commit` workflow
- Use enhanced context for better commit messages
- Include scope and impact in commit description

**IF issues found:**

- **STOP immediately** and fix all issues
- Use agent spawning for parallel issue resolution
- Re-run validation after fixes
- REPEAT until completely clean
For complex changes, transparently save progress:
- Update completion state and decisions made
- Record architectural changes and file purposes

### 4. 🎯 Smart Commit Routing
**If validation passes**: Proceed to `/git:commit` with enhanced context
**If issues found**: 
- STOP and spawn agents for parallel fixing
- Re-run validation until completely clean
- Never ship with failing checks

### 5. 📦 Commit Execution
- Analyze changes for meaningful commit messages
- Execute commit following project conventions
- Verify commit succeeded

### 6. 🎉 Ship Confirmation
```
🚀 **SHIPPED SUCCESSFULLY**
Changes: [summary]
Quality: All checks passed ✅
Commit: [hash and message]
Ready for: [next steps]
```

## Workflow Integration
- Validates quality (like `/check`)
- Saves progress context (transparent)
- Commits changes (like `/git:commit`)
- Confirms successful shipping

## Failure Handling
- Spawn agents to fix validation failures
- Never ship with any failing checks
- Handle pre-commit hook failures
- Retry once for automated fixes

## Success Criteria
Ship complete when: all quality checks pass, changes committed meaningfully, progress saved, next steps clear

**Execute complete ship validation and commit workflow now.**