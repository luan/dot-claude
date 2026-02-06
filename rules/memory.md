# Memory & Learning

## Where

- Universal rule → `~/.claude/rules/<topic>.md`
- Project-specific → project's `CLAUDE.md`
- Never use auto-memory (`projects/*/memory/`) — not version-controlled, fragments knowledge

## When to Record

Record when:
- Mistake made that rule would prevent
- User corrects workflow preference (tool choice, naming, process)
- Non-obvious constraint discovered (API quirk, build gotcha)

Don't record:
- One-off task facts
- Already covered by existing rules
- Obvious things

## How to Record

1. Check if existing rule file covers topic — edit if so
2. Otherwise create `~/.claude/rules/<topic>.md`
3. Keep concise — actionable statements, not narratives
4. Remind user to commit in dot-claude repo
