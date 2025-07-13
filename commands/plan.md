---
allowed-tools: all
description: Strategic planning for complex multi-session projects
---

# 📋 Plan Command

**Command**: `/plan [project description]`

## Purpose
Plan and set up tracking for complex projects that span multiple sessions or require structured approach.

## Instructions for Claude

When the user runs `/plan [project]`, you MUST follow these steps exactly:

### 1. 🔍 Project Complexity Assessment

**ANALYZE the requested project:**
- Scope and complexity level
- Multiple components or phases
- Cross-session coordination needed
- Dependencies and prerequisites

**DETERMINE if planning is appropriate:**
- Complex features requiring multiple sessions
- New system components or architecture
- Large refactoring efforts
- Multi-step implementation processes

### 2. 🧠 Automatic Context Setup

**FOR complex projects**, transparently initialize tracking:
- Set up project context structure
- Capture current codebase state
- Create planning workspace
- Initialize progress tracking

**Handle completely transparently** - user sees planning, not memory setup.

**FIRST**: Run memory initialization script:
```bash
~/.claude/workflows/plan.sh "$ARGUMENTS"
```

### 3. 📊 Project Analysis Phase

**RESEARCH the codebase thoroughly:**
- Understand existing architecture
- Identify integration points
- Assess current patterns and conventions
- Find relevant existing implementations

**IDENTIFY requirements:**
- Core functionality needed
- Quality standards to meet
- Testing requirements
- Performance considerations

### 4. 📋 Strategic Planning

**CREATE comprehensive project plan:**
```markdown
## [Project] - Implementation Plan

### 🎯 Overview
[What we're building and why]

### 🏗️ Architecture  
[Integration approach and design decisions]

### 📝 Phases
**Phase 1: [Foundation]**
- [ ] [Specific tasks]

**Phase 2: [Core Features]** 
- [ ] [Specific tasks]

**Phase 3: [Integration & Testing]**
- [ ] [Specific tasks]

### 🧪 Testing Strategy
### 🚨 Risk Factors  
### 📏 Success Criteria
```

### 5. 🎯 Implementation Strategy
- Break into single-session tasks with clear acceptance criteria
- Prioritize: critical path, quick wins, risk mitigation
- Design session boundaries and continuation points

### 6. 🚀 Execution Kickoff
Begin Phase 1, use `/next` for tasks, track progress, checkpoint at phase boundaries

## Workflow Integration
**Connects to**: `/status` (progress check), `/next` (task execution), `/ship` (phase completion)

## Success Criteria
Planning complete when: project analyzed, comprehensive plan created, tasks manageable, first phase ready

**Execute comprehensive planning workflow now.**