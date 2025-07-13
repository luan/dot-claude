---
allowed-tools: all
description: Complete validation and commit workflow for shipping
---

# 🚀 Ship Command

Complete validation and commit workflow for shipping-ready code.

## Shipping Workflow

### 1. 🔍 Pre-Ship Assessment
- Check git status and recent changes
- Determine scope: new features, fixes, refactoring, configuration

### 2. ✅ Quality Validation
**Critical**: ALL checks must pass before shipping
- Run comprehensive validation (uses `/check` functionality)
- Zero linter warnings, all tests pass, no build errors
- Clean git status after validation

### 3. 🧠 Context Tracking
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