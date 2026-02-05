## Plan Mode

- Make the plan extremely concise. Sacrifice grammar for the sake of concision.
- At the end of each plan, give me a list of unresolved questions to answer, if any.

## Code Organization

**Keep functions small and focused:**

- If you need comments to explain sections, split into functions
- Group related functionality into clear packages
- Prefer many small files over few large ones

## Architecture Principles

- Delete old code completely - no deprecation needed
- No semantic prefix or suffix (OptimizedProcessor, FastHandler, ClientImpl)
- No versioned names (processV2, handleNew, ClientOld)
- No migration code unless explicitly requested
- No "removed code" comments - just delete it
- Don't add unnecessary comments. Only add doc comments if it's the style of the existing project.
- Comments that explain something subtle or surprising are OK.

## Maximize Efficiency

- **Subagents first:** Main thread orchestrates, subagents do heavy lifting. Don't explore/implement on main thread.
- **Parallel operations:** Run multiple searches, reads, and greps in single messages
- **Batch similar work:** Group related file edits together

## Problem Solving

- **When stuck:** Stop. The simple solution is usually correct.
- **When uncertain:** Ask clarification via `AskUserQuestion`.

## Testing Strategy

**TDD is the default.** No production code without a failing test first.

- Write test → watch it fail → minimal code to pass → refactor
- Bug fix? Write failing test that reproduces bug first

**Exceptions (ask first):** throwaway prototypes, generated code, config-only changes.

## Workflow Skills

You have these skills via `Skill` tool. Use them—don't do this work on main thread.

| Invoke | When | Chemistry |
|--------|------|-----------|
| `Skill tool: explore` | Plan feature, investigate, research | WISP |
| `Skill tool: implement` | Execute plan from explore | MOL |
| `Skill tool: feedback` | Quick fix to recent work | None |

**Flow:** explore → plan mode → approval → implement → PR

**After plan approval** (user says "yes", "go ahead", "proceed"):
- **IMMEDIATELY** invoke implement skill with the epic-id
- Do NOT manually implement—skill handles subagent dispatch

## Branch Naming

Always use prefix `luan/` with short description: `gt create luan/<description>`

Examples:
- `luan/fix-container-minimize`
- `luan/add-theme-constants`
- `luan/refactor-drag-drop`

## Beads Commands

```bash
bd ready                    # Find next task (no blockers)
bd show <id>               # Read task instructions
bd update <id> --status in_progress
bd close <id>              # Complete task
bd lint <id>               # Validate issue quality (REQUIRED)
bd mol wisp <formula>      # Ephemeral workflow
bd mol pour <formula>      # Persistent workflow
```

**CRITICAL:** `bd lint` is NOT optional. Run on all issues before claiming plan complete.

Session state survives compaction via beads notes field.
