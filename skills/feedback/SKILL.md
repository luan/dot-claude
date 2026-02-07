---
name: feedback
description: "Apply user feedback to recent implementation. Triggers: 'feedback', 'the X doesn't work', 'change Y to Z', 'fix this'"
argument-hint: "<feedback> [--type=bug|quality|change]"
user-invocable: true
allowed-tools:
  - Task
---

# Feedback Skill

Process user feedback on recent implementation and apply fixes.

**Chemistry:** No molecules. Feedback is quick, single-session work—no audit trail needed.

- Uses TaskCreate/TaskUpdate for internal tracking (not beads)
- Checks `bd mol current` for active molecule context
- Side quests → create beads issue with `discovered-from` dependency
- Large changes → recommend `/explore` (which uses wisp)

## Instructions

Spawn a general-purpose agent via Task with this prompt:

```
Process user feedback on recent implementation.

## Parse Arguments

$ARGUMENTS contains:
- `--type=TYPE`: bug|quality|change (optional, infer if missing)
- Feedback text (required)

No feedback text → exit: "Please provide feedback. Example: /feedback The button doesn't work"

## Find Recent Context

In order:
1. `git diff --name-only HEAD` (uncommitted changes)
2. `git diff --name-only HEAD~3..HEAD` (recent commits)
3. `bd list --status in_progress` (active beads issue)
4. `bd mol current` (active molecule if any)

No context found → exit: "No recent changes found. What files does this apply to?"

If beads issue found, note it as `<context-issue>` for discovered-from linking.

## Categorize Feedback

If --type not given, infer:
- **Bug**: "doesn't work", "fails", "error", "broken", "crash", "expected X got Y"
- **Quality**: "naming", "readability", "confusing", "inconsistent", "messy"
- **Change**: "add", "change", "modify", "instead", "also need", "should have"
- Default: change

## Analyze & Fix

### Bugs
1. Identify symptom
2. Read relevant files completely
3. Find likely cause (missing error handling, wrong logic, edge case)
4. Apply minimal fix via Edit

### Quality
1. Identify specific concern
2. Read files to understand patterns
3. Apply improvement preserving behavior

### Changes
1. Understand what's requested
2. Assess scope: small (inline fix), medium (multiple edits), large (needs /explore)
3. Small/medium: apply. Large: recommend `/explore` instead.

## Fix Categorization

**Apply automatically:**
- Renaming, null checks, typos, imports
- Simple logic corrections
- Adding error handling
- Multi-line edits within a file

**Defer to /explore:**
- Architecture changes
- New features
- Breaking API changes
- Cross-cutting concerns

**Create beads issue for:**
- Bugs found that aren't immediate fix (link with discovered-from)
- Follow-up improvements noticed during fix
- Technical debt discovered

```bash
# If feedback reveals related issue during fix:
bd create "Found: <description>" --type bug --validate --deps discovered-from:<context-issue>
```

## Workflow

1. Create tasks via TaskCreate for each fix
2. For each task:
   - TaskUpdate to in_progress
   - Read file completely
   - Apply fix via Edit
   - Verify syntax
   - TaskUpdate to completed
3. If fix fails: note reason, continue to next

## Return Value

Return:
- Type: Bug|Quality|Change
- Status: Fixed|Partial|Deferred
- Files modified (list)
- What was done (brief)
- Verification steps
- If deferred: recommend `/explore "{description}"`

## Guidelines

- Only fix what feedback mentions
- Don't expand scope
- Preserve existing patterns
- When unsure: ask, don't guess
- Large changes → /explore
```

## Skill Composition

| When | Invoke |
|------|--------|
| Large change needed | Recommend `/explore` |
| Before claiming done | Run verification — evidence before assertions |
| Complex bug | `use Skill tool to invoke debugging` |
