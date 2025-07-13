---
allowed-tools: all
description: Validate everything and commit when ready to ship
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

### 2. ✅ MANDATORY Quality Validation

**RUN comprehensive checks** (same as `/check` but focused on shipping):

```bash
# Run all available quality checks
make lint 2>/dev/null || echo "No make lint available"
~/.claude/hooks/smart-lint.sh 2>/dev/null || echo "No smart lint available"
```

**CRITICAL REQUIREMENT**: ALL checks must pass before shipping
- ZERO linter warnings
- ALL tests passing
- NO build errors
- CLEAN git status after validation

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

### 5. 📦 Commit Execution

**When validation passes**, execute commit workflow:
- Analyze changes for commit message context
- Create meaningful commit following conventions
- Use project context for better descriptions
- Verify commit succeeded

### 6. 🎉 Ship Confirmation

**Provide shipping summary:**
```
🚀 **SHIPPED SUCCESSFULLY**

Changes: [summary of what was shipped]
Quality: All checks passed ✅
Commit: [commit hash and message]

Ready for: [next logical steps like deployment, PR creation, etc.]
```

## Workflow Integration

**Ship represents the complete "done" workflow:**
1. Validate quality (`/check` functionality)
2. Save progress context (transparent memory)
3. Commit changes (`/git:commit` functionality)
4. Confirm successful shipping

## Failure Handling

**IF validation fails:**
- **SPAWN AGENTS** to fix issues in parallel
- **FIX ALL PROBLEMS** before proceeding
- **RE-RUN validation** until clean
- **NEVER ship** with failing checks

**IF commit fails:**
- Handle pre-commit hook failures
- Retry once for automated fixes
- Report any persistent issues

## Success Criteria

Ship is complete when:
- ✅ ALL quality checks pass with zero warnings
- ✅ ALL tests pass successfully
- ✅ Changes committed with meaningful message
- ✅ Progress context saved (transparently)
- ✅ User confirmed ready for next steps

## Integration Rules

- NEVER ship with any failing validation
- HANDLE memory updates transparently
- PROVIDE clear feedback on what was shipped
- SUGGEST logical next steps after shipping

**EXECUTING complete ship validation and commit workflow NOW...**