# ~/.claude

Claude Code configuration. Skills, rules, and workflows
for AI-assisted development.

## Quick Start

```bash
/vibe "add user authentication"      # Full pipeline, hands-off
/scope "add user authentication"     # Research, design, create tasks
/develop <epic-id>                   # Execute (solo or swarm)
/review                              # Adversarial code review
/commit                              # Conventional commit
```

## Pipeline

```
brainstorm → scope → develop → split-commit → review → commit
```

- **brainstorm**: Collaborative design for greenfield features
- **scope**: Research codebase, design, and create implementation tasks
- **develop**: Workers own TDD from briefs (auto solo/swarm)
- **split-commit**: Repackage branch into clean, tested commits
- **review**: Adversarial review with built-in fix loop + polish
- **commit**: Conventional commit

Other skills: `/test-plan`, `/triage`, `/debugging`, `/next`, `/start`,
`/split-commit`, `/gt`, `/acceptance`, `/pr:descr`,
`/pr:comments`, `/frontend-design`

## License

Do whatever you want with this.
