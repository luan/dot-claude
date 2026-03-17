# Bash Tool Replacements

When using the Bash tool, never use legacy commands that have modern replacements:

- `find` → `fd` or `rg --files`
- `grep` / `grep -r` → `rg`
- `ls -R` → `rg --files` or `fd`
- `cat file | grep` → `rg pattern file`
