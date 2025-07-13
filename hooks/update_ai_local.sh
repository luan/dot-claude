#!/bin/bash
# Hook: update_ai_local
# Purpose: Update .ai.local with current progress and session data

AI_LOCAL_DIR=".ai.local"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M')

# Only run if .ai.local exists
if [ -d "$AI_LOCAL_DIR" ]; then
    # Update session info
    SESSION_FILE="$AI_LOCAL_DIR/session/last-session.md"
    echo "## Session Update: $TIMESTAMP" >> "$SESSION_FILE"
    echo "Working directory: $(pwd)" >> "$SESSION_FILE"
    
    # If git repo, add branch info
    if [ -d ".git" ]; then
        BRANCH=$(git branch --show-current 2>/dev/null)
        if [ -n "$BRANCH" ]; then
            echo "Git branch: $BRANCH" >> "$SESSION_FILE"
        fi
    fi
    
    echo "---" >> "$SESSION_FILE"
    echo "✅ Updated .ai.local session info"
fi