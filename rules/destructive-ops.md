# Destructive Operations

## git checkout -- <file>

NEVER `git checkout` a file to "restore" it if the user
might have uncommitted changes. Ask first. Always.

The right approach: read the file, make targeted edits.
If you broke it, undo your specific changes — don't nuke
the whole file.

## replace_all

`replace_all: true` on Edit is dangerous on config files.
Only use for simple renames where the old string is unique
and unambiguous. Never for config surgery.
