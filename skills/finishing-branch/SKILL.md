---
name: finishing-branch
description: "Triggers: 'done', 'finished', 'ready to merge', 'create PR', 'what now'. Implementation complete, ready to integrate."
argument-hint: "[--merge|--pr|--keep|--discard]"
user-invocable: true
---

# Finishing a Development Branch

Verify tests → Execute action → Clean up.

## Step 1: Verify Tests

```bash
npm test / cargo test / pytest / go test ./...
```

If tests fail: report and stop.

## Step 2: Determine Action

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Requires action flag: --merge, --pr, --keep, or --discard" || echo "Use AskUserQuestion: 1) Merge locally 2) Create PR 3) Keep as-is 4) Discard (requires typed confirmation)"`

## Actions

### Merge (--merge)
```bash
git checkout {base}
git pull
git merge {feature}
git branch -d {feature}
```

### PR (--pr)
```bash
git push -u origin {feature}
gh pr create --title "{title}" --body "..."
```

### Discard (--discard)
```bash
git checkout {base}
git branch -D {feature}
```

## Skill Composition

| When | Invoke |
|------|--------|
| Selective staging | `use Skill tool to invoke git-surgeon` |
| Commit message | `use Skill tool to invoke commit` |
| Tests fail | `use Skill tool to invoke debugging` |
