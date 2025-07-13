---
allowed-tools: all
description: Plan complex multi-session projects with progress tracking
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
## [Project Name] - Implementation Plan

### 🎯 Overview
[Clear description of what we're building and why]

### 🏗️ Architecture Approach
[How this fits into existing codebase]
[Key design decisions and rationale]

### 📝 Implementation Phases
**Phase 1: [Foundation]**
- [ ] [Specific task]
- [ ] [Specific task]

**Phase 2: [Core Features]**
- [ ] [Specific task]
- [ ] [Specific task]

**Phase 3: [Integration & Testing]**
- [ ] [Specific task]
- [ ] [Specific task]

### 🧪 Testing Strategy
[How we'll validate each phase]

### 🚨 Risk Factors
[Potential blockers and mitigation]

### 📏 Success Criteria
[How we know we're done]
```

### 5. 🎯 Implementation Roadmap

**BREAK DOWN into manageable tasks:**
- Each task completable in single session
- Clear acceptance criteria
- Dependencies clearly marked
- Progress tracking built in

**PRIORITIZE tasks:**
- Critical path identification
- Quick wins for momentum
- Risk mitigation ordering

### 6. 🔄 Session Planning

**FOR multi-session projects:**
- Design session boundaries
- Plan progress checkpoints
- Set up continuation points
- Create session handoff process

### 7. 🚀 Execution Kickoff

**AFTER planning approval:**
- Begin Phase 1 implementation
- Use `/next` for individual tasks
- Track progress transparently
- Checkpoint at phase boundaries

## Workflow Integration

**Plan connects to other workflows:**
- `/status` - Check progress against plan
- `/next` - Execute individual planned tasks
- `/ship` - Complete and commit phases
- Natural continuation across sessions

## Success Criteria

Planning is complete when:
- ✅ Project thoroughly analyzed and understood
- ✅ Comprehensive implementation plan created
- ✅ Tasks broken down into manageable pieces
- ✅ Progress tracking established (transparently)
- ✅ User approves approach and ready to start
- ✅ First phase ready for execution

## Integration Rules

- ONLY use for genuinely complex projects
- HANDLE tracking setup completely transparently
- FOCUS on implementation strategy, not process
- CONNECT plan to existing development workflows
- MAKE progress visible and actionable

**EXECUTING comprehensive project planning NOW...**