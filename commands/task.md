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
🧠 The model will autonomously manage `.ai.local/` to:
- Track project context and understanding
- Record important decisions and patterns
- Maintain progress across sessions
- Organize information as needed

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