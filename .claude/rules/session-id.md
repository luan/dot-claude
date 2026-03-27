# Session ID Resolution

The user may reference a session by its shortest unique prefix (e.g. `806` for `8061f49f-fa86-4dd8-a4bc-482c2f75ffe5`). The prefix matches only the **start** of the session UUID, not mid-string.

To resolve a prefix to a full session ID and transcript:

```bash
fd '^<prefix>.*\.jsonl$' ~/.claude/projects/
```

This matches filenames starting with the prefix. Do NOT use `rg` on the full path — that catches mid-UUID and directory hits.

Transcript files are named `{session_id}.jsonl` under `~/.claude/projects/`. If multiple files match, ask the user to provide more characters.
