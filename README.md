# ~/.claude

Claude Code configuration. Skills, rules, and workflows
for AI-assisted development.

## Quick Start

```bash
/vibe "add user authentication"      # Full pipeline, hands-off
/explore "add user authentication"   # Research + design
/prepare <task-id>                   # Create epic + phased tasks
/implement <epic-id>                 # Execute (solo or swarm)
/review                              # Adversarial code review
/commit                              # Conventional commit
```

## Pipeline

```
brainstorm|explore → prepare → implement → split-commit → review → commit
```

- **brainstorm**: Collaborative design for greenfield features
- **explore**: Research codebase, produce phased design plan
- **prepare**: Convert plan into epic + task briefs (no code)
- **implement**: Workers own TDD from briefs (auto solo/swarm)
- **split-commit**: Repackage branch into clean, tested commits
- **review**: Adversarial review with built-in fix loop + polish
- **commit**: Conventional commit

Other skills: `/test-plan`, `/fix`, `/debugging`, `/next`, `/start`,
`/split-commit`, `/graphite`, `/git-surgeon`, `/pr-description`,
`/pr-fix-comments`, `/pr-fix-gha`, `/pr-reviewers`,
`/bootstrap:web`, `/bootstrap:caddy`, `/frontend-design`,
`/writing-skills`

## License

Do whatever you want with this.
