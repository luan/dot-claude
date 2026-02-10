---
name: feedback
description: "Apply user feedback to recent implementation. Triggers: 'feedback', 'the X doesn't work', 'change Y to Z', 'fix this'"
argument-hint: "<feedback> [--type=bug|quality|change]"
user-invocable: true
allowed-tools:
  - Task
---

# Feedback

Process user feedback on recent implementation. Apply fixes.

**Chemistry:** None. Quick single-session work — no audit trail.

- Uses TaskCreate/TaskUpdate internally (not beads)
- Checks `bd mol current` for active molecule context
- Side quests → beads issue with `discovered-from` dep
- Large changes → recommend `/explore`

## Instructions

Spawn general-purpose agent via Task (model: "sonnet"):

```
Process user feedback on recent implementation.

## Parse Arguments

$ARGUMENTS contains:
- `--type=TYPE`: bug|quality|change (optional, infer if missing)
- Feedback text (required)

No feedback text → exit: "Please provide feedback. Example: /feedback The button doesn't work"

## Find Recent Context

In order:
1. `git diff --name-only HEAD` (uncommitted)
2. `git diff --name-only HEAD~3..HEAD` (recent commits)
3. `bd list --status in_progress` (active beads)
4. `bd mol current` (active molecule)

No context → exit: "No recent changes found. What files does this apply to?"

If beads issue found, note as `<context-issue>` for discovered-from linking.

## Categorize

If --type not given, infer:
- **Bug**: "doesn't work", "fails", "error", "broken", "crash"
- **Quality**: "naming", "readability", "confusing", "inconsistent"
- **Change**: "add", "change", "modify", "instead", "also need"
- Default: change

## Analyze & Fix

### Bugs
1. Identify symptom → read files → find cause → minimal fix

### Quality
1. Identify concern → read files → improve preserving behavior

### Changes
1. Understand request → assess scope (small/medium/large)
2. Small/medium: apply. Large: recommend `/explore`.

## Fix Scope

**Apply automatically:** renames, null checks, typos, imports,
simple logic fixes, error handling, multi-line edits within file

**Defer to /explore:** architecture changes, new features,
breaking API changes, cross-cutting concerns

**Create beads issue for:** non-immediate bugs, follow-up
improvements, tech debt discovered
```bash
bd create "Found: <description>" --type bug --validate --deps discovered-from:<context-issue>
```

## Workflow

1. TaskCreate per fix
2. TaskUpdate → in_progress → read file → Edit → verify → completed
3. Fix fails → note reason, continue

## Return

- Type: Bug|Quality|Change
- Status: Fixed|Partial|Deferred
- Files modified
- What was done (brief)
- Verification steps
- If deferred: recommend `/explore "{description}"`
```

## Skill Composition

| When | Invoke |
|------|--------|
| Large change | Recommend `/explore` |
| Before claiming done | Run verification |
| Complex bug | `Skill tool: debugging` |
