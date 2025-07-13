# Git Commit Command

**Command**: `/git:commit`

## Purpose
Commits changes to the current git repository following Claude Code conventions.

## Instructions for Claude

When the user runs `/git:commit`, you MUST follow these steps exactly:

### 1. Pre-commit Analysis (Run in Parallel)
```bash
# Run these commands in parallel using multiple Bash tool calls
git status          # See all untracked files
git diff           # See staged and unstaged changes  
git log --oneline -5   # See recent commit style
```

### 2. Analyze Changes
- 🧠 **MEMORY-ENHANCED ANALYSIS** - Check `.ai.local/progress/file-changes.md` for context
- Review ALL staged changes (both previously staged and newly added)
- **LEVERAGE MEMORY** - Use `.ai.local/session/current-session.md` for recent activity context
- Check for sensitive information (keys, secrets, passwords)
- Understand the nature of changes (feature, fix, refactor, etc.)
- **CONTEXT-AWARE** - Use memory to understand the broader task context
- Draft concise commit message focusing on "why" not "what"

### 3. Create Commit (Run in Parallel)
```bash
# Run these commands in parallel
git add .          # Add relevant untracked files
git commit -m "$(cat <<'EOF'
[commit message here]
EOF
)"                 # Create commit with HEREDOC format
git status         # Verify commit succeeded
```

### 4. Handle Pre-commit Hook Failures
- If commit fails due to pre-commit hooks: retry ONCE to include automated changes
- If fails again: pre-commit hook is preventing commit
- If succeeds but files were modified by hooks: MUST amend commit to include them

## Commit Message Format
- **Format**: `type(scope): short description`
- **Length**: 50 characters max
- **Style**: Conventional commits, imperative mood
- **Types**: feat, fix, docs, style, refactor, test, chore
- **No period** at end

## Examples
- `feat: add user auth`
- `fix: handle null refs` 
- `docs: update API guide`
- `refactor: simplify error handling`

## Important Rules
- NEVER update git config
- NEVER run additional code exploration beyond git commands
- NEVER use TodoWrite or Task tools during commit process
- DO NOT push unless user explicitly requests it
- NEVER use git commands with -i flag (interactive)
- If no changes to commit, do not create empty commit
- ALWAYS use HEREDOC format for commit messages

## Error Handling
- If hooks fail: STOP and fix before proceeding
- If no changes found: inform user and exit
- If sensitive data detected: warn user and request confirmation