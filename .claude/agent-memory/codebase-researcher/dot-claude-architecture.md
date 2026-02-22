# ~/.claude Architecture Notes

## Skill System

Skills are SKILL.md files with YAML frontmatter. Claude loads them based on description field triggers.
- `user-invocable: true` → user can type `/skill-name`
- `context: fork` → runs as isolated subagent
- `allowed-tools` → restricts tool access

## Implement Skill State Model

The implement skill (`skills/implement/SKILL.md`) tracks state only via:
1. Task status fields (pending/in_progress/completed)
2. Task `metadata.owner` ("solo", "worker-<taskId>")
3. Team config file at `~/.claude/teams/<name>/config.json`
4. Rolling scheduler loop in orchestrator's context window

**No durable session state survives compaction** — the rolling loop is ephemeral.

## Teams

Teams are created via `TeamCreate(team_name="impl-<slug>")`.
Config stored at `~/.claude/teams/<name>/config.json` with:
- leadAgentId, leadSessionId
- members array with worker prompts, tmux pane IDs, isActive flags

Workers communicate via `SendMessage` to team-lead inbox.
Main thread reads team-lead inbox for completion signals.

## Context Compaction Problem

What SURVIVES compaction:
- CLAUDE.md (always injected fresh as system-reminder)
- All rules/*.md files (injected fresh)
- All skill SKILL.md files currently loaded
- Task state in Linear (queryable via TaskGet/TaskList)
- Team config files (readable from filesystem)
- Git state (queryable)

What is LOST during compaction:
- The rolling scheduler loop state (active_count, dispatch_count map)
- Knowledge that a team exists and is active
- Knowledge that `implement` skill is active/in-progress
- Which tasks are "in flight" (workers running but task still in_progress)
- The orchestration pattern (main thread = orchestrator, not implementer)

## Vibe Skill Recovery

Vibe skill stores `metadata.vibe_stage` in a task — survives compaction.
`/vibe --continue` reads vibe_stage to resume from last checkpoint.
But implement has no equivalent checkpoint/resume mechanism.

## Context Warning Hook

`hooks/context_warning.py` warns at 73% and 80% context usage (blocking exit code 2).
At 80%: "Compaction imminent. Save active task context..."
The hook fires per tool use — Claude can respond by saving state.
