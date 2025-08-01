# /commit - Commit Changes

Commit changes using the specialized commit-message-writer agent.

## Usage

- `/commit` - Auto-generate commit message from changes
- `/commit [message]` - Use custom commit message

## Behavior

Uses the `commit-message-writer` agent to:

1. Analyze git status and diffs in parallel
2. Generate conventional commit messages
3. Handle pre-commit hooks gracefully

The agent follows the Conventional Commits specification and focuses on explaining WHY changes were made rather than just WHAT changed.

## Examples

```
/commit  # Commit with auto-generated message
/commit "feat: add user authentication with JWT"
```

## Implementation

When this command is invoked:

1. If a custom message is provided, use it directly
2. Otherwise, launch the commit-message-writer agent to analyze changes and create an appropriate commit message
