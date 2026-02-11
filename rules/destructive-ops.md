# Destructive Operations

## git checkout -- <file>

NEVER `git checkout` file to "restore" if user might have uncommitted changes. Ask first. Always.

Right approach: read file, make targeted edits.
If you broke it, undo your specific changes — don't nuke whole file.

## replace_all

`replace_all: true` on Edit is dangerous on config files.
Only use for simple renames where old string is unique + unambiguous. Never for config surgery.
