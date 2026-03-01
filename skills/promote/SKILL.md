---
name: promote
description: "Use when moving a skill, rule, agent, or hook from personal/experimental ($HOME/.claude) into a shared plugin repo. Handles the file move, preserves directory structure, and stages the change in the target plugin's git repo."
argument-hint: "<path> [plugin-name]"
---

# Promote

Move an experimental artifact from personal `$HOME/.claude` into a shared plugin repo, ready to commit.

## Steps

1. **Identify source** — resolve the full path. Must be under `$HOME/.claude` (skills/, rules/, agents/, hooks/, tools/).
2. **Choose target plugin** — if not specified, list available git-backed plugins and ask. Infer from context when unambiguous.
3. **Run the script:**
   ```
   uv run $CLAUDE_PLUGIN_ROOT/skills/promote/scripts/promote.py <source-path> <plugin-dir>
   ```
4. **Report** what moved and where. Remind user to commit.

## What the script does

- Infers content type from path structure (skills/ → skills/, rules/ → rules/, etc.)
- Moves the file/directory into the matching subdirectory of the target plugin
- Runs `git add` in the target plugin — leaves the commit to the user

## Discovering plugins

Available git-backed plugin dirs are symlinks under `$HOME/.claude/local-plugins/plugins/` that resolve to git repos.

## When to use

User has been experimenting with a new skill/rule in `$HOME/.claude` and decides it's stable enough to share. They want it moved into the OSS or work plugin without manually doing the mv + git add.
