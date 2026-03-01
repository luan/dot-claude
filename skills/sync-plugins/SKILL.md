---
name: sync-plugins
description: "Use when plugins may be out of date, the user wants to pull latest plugin updates, or asks to sync/update their Claude plugins. Runs git pull --ff-only in each git-backed plugin directory and reports what changed."
argument-hint: "[plugin-name]"
---

# Sync Plugins

Pull latest changes from upstream for all (or one) git-backed local plugins.

## Steps

1. Run the sync script:
   ```
   uv run $CLAUDE_PLUGIN_ROOT/skills/sync-plugins/scripts/sync.py [plugin-name]
   ```
2. Report results — which plugins updated, which were already current, which failed.
3. If a plugin failed (non-fast-forward or dirty tree), surface the error and let the user decide — do NOT attempt to fix automatically.

## Arguments

- No argument → sync all git-backed plugins
- Plugin name → sync that one only (e.g. `my-work-plugin`)

## Notes

Only git repos are synced. Plugins that are plain directories (not git repos) are skipped silently.
A failed `--ff-only` means the local branch has diverged — the user needs to decide whether to reset or merge.
