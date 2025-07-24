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

### Memory Management (Automem)
🧠 The model will autonomously manage context using Automem:

**Task Context Storage:**
- `memory_store` for implementation details and decisions
- `quick_task` for immediate todo items
- `quick_note` for rapid context capture
- `relationships_store` to link files, features, and decisions

**Progress Tracking:**
- `board_create` for task breakdown
- `board_move` to update task status
- `board_status` to visualize progress
- `memory_store` with category "result" for outcomes

**Context Retrieval:**
- `quick_find` for rapid searches
- `memory_query` for detailed context
- `relationships_query` to understand connections
- `workflow_status` if part of larger project

**Categories Used:**
- "observation" - Research findings and analysis
- "action" - Implementation steps taken
- "result" - Outcomes and validations
- "error" - Issues encountered
- "context" - Architectural decisions

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