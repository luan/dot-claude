#!/bin/bash
# Hook: update_ai_local
# Purpose: Update .ai.local with current progress and session data using git

# Prevent creating .ai.local inside .ai.local
CURRENT_DIR=$(basename "$(pwd)")
if [[ "$CURRENT_DIR" == ".ai.local" ]] || [[ "$(pwd)" == *"/.ai.local"* ]]; then
    exit 0  # Silently exit if already inside .ai.local
fi

AI_LOCAL_DIR=".ai.local"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M')

# Create .ai.local structure if missing
if [ ! -d "$AI_LOCAL_DIR" ]; then
    mkdir -p "$AI_LOCAL_DIR"
fi

# Initialize git repo in .ai.local if not already done
if [ ! -d "$AI_LOCAL_DIR/.git" ]; then
    (
        cd "$AI_LOCAL_DIR" || exit 1
        git init --quiet
        git config user.name "Claude Code"
        git config user.email "claude@anthropic.com"
        
        # Create .gitignore for sensitive data
        {
            echo "# Claude Code AI Local Memory"
            echo "*.secret"
            echo "*.key"
            echo "temp/"
        } > .gitignore
    )
fi

# Create required subdirectories per CLAUDE.md
mkdir -p "$AI_LOCAL_DIR/context"
mkdir -p "$AI_LOCAL_DIR/progress" 
mkdir -p "$AI_LOCAL_DIR/research"
mkdir -p "$AI_LOCAL_DIR/session"

# Update session info
SESSION_FILE="$AI_LOCAL_DIR/session/last-session.md"
echo "## Session Update: $TIMESTAMP" >> "$SESSION_FILE"
echo "Working directory: $(pwd)" >> "$SESSION_FILE"

# If parent directory is git repo, add branch info
if [ -d ".git" ]; then
    BRANCH=$(git branch --show-current 2>/dev/null)
    if [ -n "$BRANCH" ]; then
        echo "Git branch: $BRANCH" >> "$SESSION_FILE"
    fi
fi

echo "---" >> "$SESSION_FILE"

# Commit changes to .ai.local git repo
(
    cd "$AI_LOCAL_DIR" || exit 1
    git add -A 2>/dev/null
    if ! git diff --cached --quiet 2>/dev/null; then
        # Create meaningful commit message with context
        PROJECT_NAME=$(basename "$(dirname "$(pwd)")")
        
        # Get what files were modified/added for context
        CHANGES=$(git diff --cached --name-only | head -3 | tr '\n' ' ' | sed 's/ $//')
        
        # Build contextual commit message
        COMMIT_MSG="mem($PROJECT_NAME): $TIMESTAMP"
        if [ -n "$CHANGES" ]; then
            COMMIT_MSG="$COMMIT_MSG - updated $CHANGES"
        fi
        
        git commit --quiet -m "$COMMIT_MSG" 2>/dev/null
    fi
)