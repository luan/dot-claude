---
paths:
  - "**/*.py"
  - "skills/**/*.md
  - "**/pyproject.toml"
---

# Python

## Execution

- Never invoke `python3` or `python` directly.
- Use `uv run` for scripts and commands.
- Standalone scripts: `#!/usr/bin/env -S uv run --script` shebang
  with inline dependency metadata.
- Projects with pyproject.toml: `uv run <command>`.

## Dependencies

- `uv add` / `uv remove` — never `pip install`.
- Pin versions in pyproject.toml, not requirements.txt.
