---
allowed-tools: all
description: Get oriented - load project context and current progress
---

# 📊 Status Command

**Command**: `/status`

## Purpose
Quick orientation - understand project state, current progress, and what to work on next.

## Instructions for Claude

When the user runs `/status`, you MUST follow these steps exactly:

### 1. 🔍 Project Context Analysis

**FIRST** check if `.ai.local/` exists and load context transparently:

```bash
# Check for memory structure
ls -la .ai.local/ 2>/dev/null || echo "No previous context found"
```

### 2. 📋 Present Project Overview

**If memory exists**, load and present context:
- Project type, framework, architecture
- Current task and progress
- Recent activities and decisions
- Known issues or blockers

**If no memory**, analyze project and present:
- Detected project type and structure
- Key files and dependencies
- Suggest setting up context for complex projects

### 3. 🎯 Actionable Summary

**ALWAYS provide clear next steps:**

```
📊 **PROJECT STATUS**
Type: [detected project type]
Current: [what's in progress or suggested next task]

🎯 **READY TO:**
- [immediate actionable options]
- [suggested workflows based on project state]

💡 **WORKFLOW SUGGESTIONS:**
- Simple changes: just describe what you want
- Complex features: I'll help plan and track progress  
- Quality check: run `/check` to validate everything
- Ready to ship: run `/git:commit` when validated
```

### 4. 🧠 Transparent Memory Management

**Automatically handle memory as needed:**
- Load existing context without mentioning `.ai.local`
- For complex projects without memory, offer to set up tracking
- Update session context transparently after status check

### 5. 🤔 Smart Workflow Guidance

**Based on project state, suggest appropriate next actions:**
- If in middle of feature -> offer to continue work
- If tests failing -> suggest running `/check`
- If clean state -> suggest new tasks or improvements
- If complex project -> offer planning and tracking setup

## Integration Rules

- NEVER mention `.ai.local` or memory files to user
- PRESENT information naturally as project understanding
- SUGGEST workflows based on actual project needs
- HANDLE memory setup transparently if user accepts tracking for complex projects

## Success Criteria

Status is complete when:
- ✅ User understands current project state
- ✅ Clear next steps provided
- ✅ Appropriate workflows suggested
- ✅ Memory handled transparently
- ✅ User ready to take action

**EXECUTING project status analysis NOW...**