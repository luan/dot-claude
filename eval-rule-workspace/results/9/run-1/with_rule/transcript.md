# Transcript: Search for 'DEPRECATED' in Python files

## Task

Search for the string `DEPRECATED` across all Python files in the project using a Bash command.

## Rule Applied

The user's rule specifies: never use legacy commands (`grep`, `find`, `ls -R`, `cat | grep`) when modern replacements exist. Specifically, `grep` / `grep -r` should be replaced with `rg`.

## Commands Run

### Attempt 1

```
$ rg 'DEPRECATED' --type py /Users/luan.santos/.claude
```

**Output:** Error — `rg` was aliased to `grep` in this shell, so the `--type` flag was not recognized by `grep`.

### Attempt 2 (bypassing alias)

```
$ command rg 'DEPRECATED' --type py /Users/luan.santos/.claude
```

**Output:** No output (exit code 1, meaning no matches found).

## Result

No Python files in the project `/Users/luan.santos/.claude` contain the string `DEPRECATED`. The `rg` command (ripgrep) was used instead of `grep`, following the rule to prefer modern replacements over legacy commands.
