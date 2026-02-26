# Codebase Researcher Memory

## ~/.claude — Claude Code Configuration Repo

Skill pipeline: brainstorm → scope → develop [acceptance] → review → commit
Skills live in `/Users/luan/.claude/skills/<name>/SKILL.md`
Rules live in `/Users/luan/.claude/rules/<name>.md`

Key develop skill: `/Users/luan/.claude/skills/develop/SKILL.md`
- Solo / Team modes
- Team uses TeamCreate + team-based worker prompts with SendMessage
- Main thread orchestrates; workers never commit
- Team configs written to `~/.claude/teams/<name>/config.json`

Context warning hook: `/Users/luan/.claude/hooks/context_warning.py`
- Warns at 73%, critical at 80%
- Uses exit code 2 (blocking) to force Claude to see the warning

State stored in Tasks: TaskCreate/TaskUpdate/TaskGet/TaskList
- Epics track `metadata.slug`, `metadata.design`, `metadata.vibe_stage`
- Design stored in `metadata.design` (single source of truth)

See `dot-claude-architecture.md` for full architecture notes.
