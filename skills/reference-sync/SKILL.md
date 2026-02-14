---
name: reference-sync
description: "Sync references/ submodule and analyze upstream changes since last update. Triggers: \"reference sync\", \"check references\", \"update references\", \"what changed upstream\"."
disable-model-invocation: true
user-invocable: true
---

# Reference Sync

Track changes in `references/` git submodule (jfmyers9/claude), incorporate learnings into workflow.

## Steps

1. **Check state**:
   ```bash
   cd references && git rev-parse HEAD    # current pin
   git fetch origin                        # get latest
   git log --oneline HEAD..origin/main     # new commits
   ```

2. **No new commits** → "Already up to date", stop.

3. **Analyze changes** — up to 4 parallel Explore subagents:

   | Agent | Scope | Look for |
   |-------|-------|----------|
   | skills-analyst | `skills/` diff | New/modified skills, patterns |
   | rules-analyst | `rules/ .claude/CLAUDE.md` diff | Rule changes, conventions |
   | agents-analyst | `agents/ .claude/agents/` diff | Agent definitions, workflows |
   | infra-analyst | `*.json *.sh install.sh hooks/ .claude/settings.json` diff | Config, hooks, settings |

   Each agent: read diff for area, compare with `~/.claude/` equivalents, produce: what changed, why it matters, what to adopt

4. **Synthesize**: merge reports, categorize: adopt now / consider later / not relevant

5. **Present**:
   ```
   ## Reference Sync: X new commits
   ### Adopt Now
   - [actionable items with file paths]
   ### Consider Later
   - [non-urgent items]
   ### Not Relevant
   - [doesn't apply]
   ```

6. **After user reviews**, update submodule:
   ```bash
   cd references && git checkout origin/main
   cd .. && git add references
   ```
   Moves watermark forward — next sync shows only new changes.

## Key Constraints

- MANUAL ONLY (`/reference-sync`). Never auto-invoke.
- Compare against existing setup — don't recommend what we already do.
- Focus on novel ideas/patterns, not cosmetic differences.
- Our repo: `~/.claude/` — their repo: `references/`
- Subagents read BOTH repos for informed comparisons.

## Error Handling
- No `references/` directory → "Submodule not initialized. Run: `git submodule update --init`"
- `git fetch` fails → check network, report and stop
- Subagent fails on area → report which area failed, continue with others
