---
globs:
  - "**/.claude-plugin/plugin.json"
---

# Plugin Updates

When the user asks to update, refresh, or reload a plugin:
run `claude plugin install {plugin-name}@local` (e.g. `claude plugin install commons@local`).
This clears the cache and reloads all skills, hooks, and rules.
