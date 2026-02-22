# Codebase Researcher Memory

## ~/.claude — Claude Code Configuration Repo

Skill pipeline: explore → prepare → implement → review → commit
Skills live in `/Users/luan/.claude/skills/<name>/SKILL.md`
Agents live in `/Users/luan/.claude/agents/<name>.md`
Rules live in `/Users/luan/.claude/rules/<name>.md`

Key implement skill: `/Users/luan/.claude/skills/implement/SKILL.md`
- Solo / Parallel / Swarm modes
- Swarm uses TeamCreate + team-based worker prompts with SendMessage
- Main thread orchestrates; workers never commit
- Team configs written to `~/.claude/teams/<name>/config.json`

Context warning hook: `/Users/luan/.claude/hooks/context_warning.py`
- Warns at 73%, critical at 80%
- Uses exit code 2 (blocking) to force Claude to see the warning

State stored in Tasks (Linear via MCP): TaskCreate/TaskUpdate/TaskGet/TaskList
- Epics track `metadata.slug`, `metadata.design`, `metadata.vibe_stage`
- Plans stored via `ck plan create` + task `metadata.plan_file`

See `dot-claude-architecture.md` for full architecture notes.
