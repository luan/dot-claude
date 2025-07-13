---
allowed-tools: all
description: Project orientation and progress overview
---

# 📊 Status Command

Quick project orientation and progress overview.

## Status Workflow

### 1. 🧠 Memory Load & Context Analysis
**FIRST**: Run memory tracking script:
```bash
~/.claude/workflows/status.sh
```

**THEN**: Continue with context analysis:
- Check `.ai.local/` for project memory and load context transparently

### 2. 📋 Project Overview
**If memory exists**: Present project type, current task, recent activities, known issues
**If no memory**: Analyze project structure, key files, suggest context setup for complex projects

### 3. 🎯 Actionable Summary
```
📊 **PROJECT STATUS**
Type: [detected project type]
Current: [in progress or suggested next task]

🎯 **READY TO:**
- [immediate actionable options]
- [suggested workflows based on state]

💡 **WORKFLOW SUGGESTIONS:**
- Simple changes: describe what you want
- Complex features: plan and track progress  
- Quality check: run `/check` to validate
- Ready to ship: run `/ship` when validated
```

### 4. 🧠 Memory Management
Automatically handle context as needed:
- Load existing context without mentioning `.ai.local`
- For complex projects without memory, offer tracking setup
- Update session context transparently

### 5. 🤔 Smart Workflow Guidance
Based on project state, suggest appropriate actions:
- Continue work in progress → offer to continue
- Tests failing → suggest `/check`
- Clean state → suggest new tasks or improvements
- Complex project → offer planning and tracking

## Integration
- Never mention `.ai.local` or memory files to user
- Present information naturally as project understanding
- Suggest workflows based on actual project needs
- Handle memory setup transparently if user accepts

## Success Criteria
Status complete when: user understands project state, clear next steps provided, appropriate workflows suggested, memory handled transparently, user ready to take action

**Execute project status analysis now.**