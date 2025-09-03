# Claude Configuration Directory

This repository contains a comprehensive Claude AI configuration setup designed for efficient software development partnerships. It includes custom agents, automated hooks, development guidelines, and project management tools.

## 🚀 Key Features

- **Custom Agents**: Specialized AI agents for codebase research, committing, and prompt engineering
- **Automated Hooks**: Post-tool formatting and validation for code quality
- **Development Guidelines**: Structured workflow with Research → Plan → Test → Implement → Validate
- **Project Management**: Built-in task tracking and todo management
- **Language Support**: Extensible language-specific configurations

## 📁 Directory Structure

```
.claude/
├── agents/                 # Custom AI agents
│   ├── codebase-researcher.md
│   ├── committer.md
│   └── prompt-engineer.md
├── hooks/                  # Automated execution hooks
│   ├── post_tool_use_format.py
│   └── you_are_not_right.sh
├── projects/              # Project-specific configurations
├── lang/                  # Language-specific settings
├── commands/              # Custom command definitions
├── todos/                 # Task tracking directory
├── shell-snapshots/       # Shell execution history
├── statsig/              # Analytics and statistics
├── ide/                  # IDE integration settings
├── plugins/              # Plugin configurations
├── CLAUDE.md             # Core development partnership guidelines
├── settings.json         # Claude permissions and environment
└── README.md            # This file
```

## 🤝 Development Partnership Philosophy

This configuration implements a collaborative approach where:

- **You** handle architectural decisions and complexity management
- **Claude** implements details following established patterns
- **Together** we maintain high code quality through automated validation

### Core Workflow

1. **Research** - Understand existing patterns and architecture
2. **Plan** - Propose approach and get approval
3. **Tests** - Implement with TDD when possible
4. **Implement** - Build with proper error handling
5. **Validate** - Run formatters, linters, and tests

## ⚙️ Configuration Files

### CLAUDE.md

Contains development partnership guidelines and coding standards:

- Architecture principles (explicit over implicit)
- Code organization patterns (small, focused functions)
- Problem-solving strategies
- Testing requirements

### settings.json

Defines Claude permissions and environment settings:

- Allowed commands and tools
- Environment variables
- Security permissions
- MCP server configurations

## 🎯 Custom Agents

### Codebase Researcher

Specialized in understanding and analyzing existing codebases to inform implementation decisions.

### Committer

Handles git operations with proper commit message formatting and change validation.

### Prompt Engineer

Optimizes prompts and AI interactions for better development outcomes.

## 🔧 Automated Hooks

### Post-Tool Formatting

Automatically formats code after tool execution to maintain consistency.

### You Are Not Right Hook

Validates and provides feedback on implementation approaches to prevent over-engineering.

## 🚀 Getting Started

1. **Clone/Copy** this configuration to your `~/.claude` directory
2. **Review** `CLAUDE.md` for development guidelines
3. **Customize** `settings.json` for your environment
4. **Add** language-specific configurations in `lang/` if needed
5. **Start** using Claude with `Research → Plan → Implement` workflow

## 💡 Usage Examples

### Starting a New Feature

```
"Let me research the codebase and create a plan before implementing the user authentication feature."
```

### Code Review Process

```
1. TodoWrite to track implementation tasks
2. Research existing patterns
3. Plan architecture with you
4. Implement with tests
5. Validate with automated hooks
```

## 📝 Best Practices

- **Always start with research** before implementing
- **Use TodoWrite** for task management and progress tracking
- **Keep functions small** and focused
- **Delete old code** completely rather than commenting out
- **Prefer explicit** over implicit implementations
- **Run validation** after every significant change

## 🔄 Continuous Improvement

This configuration evolves based on:

- Development experience and lessons learned
- New Claude capabilities and features
- Project-specific needs and patterns
- Community feedback and contributions

---

_This configuration represents a production-ready approach to AI-assisted development, emphasizing maintainable code, clear communication, and efficient collaboration._

