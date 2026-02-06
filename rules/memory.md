# Memory & Learning Rules

## Where Learnings Go

- **Universal rule** (applies across projects) → `~/.claude/rules/<topic>.md`
- **Project-specific context** → that project's `CLAUDE.md`
- **Never use auto-memory** (`projects/*/memory/`). It's not version-controlled, not reviewable, and fragments knowledge.

## When to Record

Record a learning when:
- A mistake was made that a rule would prevent next time
- User corrects a workflow preference (tool choice, naming convention, process)
- A non-obvious constraint is discovered (API quirk, build system gotcha)

Don't record:
- One-off facts about a specific task
- Things already covered by existing rules
- Obvious things that don't need a rule

## How to Record

1. Check if an existing rule file covers the topic — edit it if so
2. Otherwise create a new `~/.claude/rules/<topic>.md`
3. Keep rules concise — actionable statements, not narratives
4. After writing, remind user to commit in dot-claude repo
