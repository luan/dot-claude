# commons

Claude Code plugin: skills, rules, hooks, and Rust tools for AI-assisted development.

## Setup

1. **Install the plugin**
   ```bash
   claude plugin install ~/AI/commons@local
   ```

2. **Install the `ct` CLI**
   ```bash
   cd ~/AI/commons/tools && cargo install --path crates/ct
   ```
   Requires Rust (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`).

3. **Set up the blueprints vault**
   ```bash
   ct vault init
   ```
   Creates `~/blueprints/` as a git repo for specs, plans, reviews, and reports.
   Override the location with `CT_BLUEPRINTS_DIR`.
   The vault is an [Obsidian](https://obsidian.md) vault for graph navigation, search, and wiki-link resolution.

## Quick Start

```bash
/vibe "add user authentication"      # Full pipeline, hands-off
/spec "add user authentication"      # Research codebase, produce spec
/develop                             # Execute implementation from spec
/review                              # Adversarial code review
/commit                              # Conventional commit
```

## Pipeline

```
brainstorm → spec → develop → split-commit → review → commit
```

- **brainstorm**: Collaborative design for greenfield features
- **spec**: Research codebase and produce a target-state spec
- **develop**: Execute implementation from a spec file
- **split-commit**: Repackage branch into clean, tested commits
- **review**: Adversarial review with built-in fix loop + polish
- **commit**: Conventional commit

## Other Skills

| Category | Skills |
|----------|--------|
| **Workflow** | `/supervibe`, `/simplify`, `/loop`, `/schedule`, `/challenge` |
| **Git & PRs** | `/start`, `/gt`, `/git-surgeon`, `/pr-descr`, `/pr-comments`, `/pr-ci`, `/babysit` |
| **Testing & Debug** | `/test-plan`, `/debugging`, `/agent-tdd` |
| **Creation** | `/rule-creator`, `/skill-creator`, `/frontend-design`, `/claude-api` |
| **Utilities** | `/show-image`, `/update-config`, `/keybindings-help` |

## License

Do whatever you want with this.
