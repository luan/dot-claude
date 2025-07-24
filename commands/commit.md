# /commit - Validate & Commit Changes

Ensure code quality and commit changes with intelligent context.

## Usage
- `/commit` - Auto-generate commit message from changes
- `/commit [message]` - Use custom commit message

## Behavior

### Validation First
Automatically runs `/check` before committing:
- Fixes all quality issues
- Ensures tests pass
- Verifies build succeeds
- Only commits when everything is green

### Commit Process
1. **Parallel Analysis**:
   - Git status for untracked files
   - Git diff for changes
   - Git log for commit style
   - Memory context from Automem

2. **Smart Commit Messages**:
   - Follows repository conventions
   - Includes context from memory
   - Focuses on "why" not "what"
   - Uses HEREDOC format for proper formatting

3. **Pre-commit Handling**:
   - Handles hook modifications gracefully
   - Automatically amends if hooks change files
   - Retries once if hooks fail

### Memory Integration (Automem)
🧠 Leverages Automem to:
- Query feature context with `memory_query`
- Check workflow status with `workflow_status`
- Find related issues/PRs with `relationships_query`
- Include architectural decisions from stored memories
- Maintain commit message consistency across sessions

## Examples
```
/commit  # Validate and commit with auto-generated message
/commit "feat: add user authentication with JWT"
```

## Philosophy
- Quality before speed
- Context-aware commits
- Respect project conventions
- Never commit broken code