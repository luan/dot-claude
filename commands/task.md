# /task - Dynamic Task Execution

Execute any task with intelligent workflow detection and memory management.

## Usage
- `/task [description]` - Execute a specific task
- `/task` - Check status and suggest next action

## Behavior

### Automatic Workflow Detection
The model will analyze your request and automatically:
- **Simple tasks**: Execute immediately with production quality
- **Complex tasks**: Create a plan first, then execute
- **Status check**: Show current progress when no task specified

### Memory Management
🧠 The model will autonomously manage `.ai.local/` with comprehensive context:

**Core Task Context:**
- Task objectives, requirements, and acceptance criteria
- Project context and current understanding
- Implementation decisions and rationale
- Progress tracking with detailed status updates
- Validation checkpoints and quality gates

**Enhanced Reboot Context:**
- Codebase research findings and architecture insights
- Technical patterns, conventions, and dependencies discovered
- Specific implementation details (APIs, libraries, approaches)
- Testing strategy and validation methods used
- Debugging context and solutions to blockers
- Code changes made with explanations
- Integration points and system relationships
- Future maintenance considerations and follow-up tasks

### Quality Standards
- ✅ All code follows project conventions
- 🧪 Appropriate tests written
- 🔍 Thorough research before implementation
- 📏 Clean, maintainable solutions

### Complex Task Handling
For complex challenges, the model may:
- 🤔 Use ultrathink for deep analysis
- 👥 Spawn agents for parallel work
- 📋 Break down into manageable steps

## Examples
```
/task implement user authentication
/task fix the login bug
/task refactor database models
/task  # Shows status and suggests next steps
```

## Philosophy
Trust the model to:
- Choose appropriate workflows
- Manage memory effectively
- Maintain quality standards
- Adapt to project needs