---
allowed-tools: all
description: Strategic planning for complex multi-session projects
---

# 📋 Complex Project Planning

**Command**: `/plan [project description]`

## Planning Workflow

### 1. 🔍 Project Assessment
- Analyze scope, components, dependencies
- Confirm complexity warrants planning (multi-session features, architecture, large refactoring)

### 2. 🧠 Context Setup
Set up project tracking transparently (user sees planning only, not memory operations)

### 3. 📊 Research & Analysis
**Codebase**: Architecture, integration points, existing patterns
**Requirements**: Core functionality, quality standards, testing needs

### 4. 📋 Strategic Plan Template
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