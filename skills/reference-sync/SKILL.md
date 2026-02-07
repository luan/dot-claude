---
name: reference-sync
description: "Sync references/ submodule and analyze changes since last update"
disable-model-invocation: true
user-invocable: true
---

# Reference Sync

Track changes in `references/` git submodule (jfmyers9/claude) and incorporate learnings into our workflow.

## Steps

1. **Check current state**:
   ```bash
   cd references && git rev-parse HEAD    # current pin
   git fetch origin                        # get latest
   git log --oneline HEAD..origin/main     # new commits
   ```

2. **If no new commits**: Report "Already up to date" and stop.

3. **Analyze changes** — spawn up to 4 parallel Explore subagents by area:

   | Agent | Scope | What to look for |
   |-------|-------|-----------------|
   | skills-analyst | `git diff HEAD..origin/main -- skills/` | New skills, modified skill patterns, interesting approaches |
   | rules-analyst | `git diff HEAD..origin/main -- rules/ .claude/CLAUDE.md` | Rule changes, new conventions, philosophy shifts |
   | agents-analyst | `git diff HEAD..origin/main -- agents/ .claude/agents/` | New agents, modified agent definitions, workflow patterns |
   | infra-analyst | `git diff HEAD..origin/main -- *.json *.sh install.sh hooks/ .claude/settings.json` | Settings, hooks, install scripts, config |

   Each agent should:
   - Read the full diff for their area
   - Read the current state of changed files for context
   - Compare with our equivalent files in `~/.claude/`
   - Produce: what changed, why it matters, what we could adopt

4. **Synthesize findings**:
   - Merge all agent reports
   - Categorize: adopt now / consider later / not relevant
   - For "adopt now" items, specify exact files to change in our setup

5. **Present to user**:
   ```
   ## Reference Sync: X new commits

   ### Adopt Now
   - [actionable items with specific file paths]

   ### Consider Later
   - [interesting but non-urgent items]

   ### Not Relevant
   - [items that don't apply to our workflow]
   ```

6. **After user reviews**, update the submodule:
   ```bash
   cd references && git checkout origin/main
   cd .. && git add references
   ```
   This moves the watermark forward so next sync only shows new changes.

## Key Constraints

- This skill is MANUAL ONLY (`/reference-sync`). Never auto-invoke.
- Always compare against our existing setup — don't recommend things we already do.
- Focus on novel ideas and patterns, not cosmetic differences.
- Our repo: `~/.claude/` — their repo: `references/`
- Subagents should read BOTH repos to make informed comparisons.
