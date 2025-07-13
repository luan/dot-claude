#!/bin/bash
# smart_memory_capture.sh - Intelligent context capture for Claude Code memory system
# Captures meaningful context from tool usage for enhanced session continuity

set +e

# Source common helpers
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/common-helpers.sh" 2>/dev/null || true

# Prevent infinite loops in .ai.local
CURRENT_DIR=$(basename "$(pwd)")
if [[ "$CURRENT_DIR" == ".ai.local" ]] || [[ "$(pwd)" == *"/.ai.local"* ]]; then
    exit 0
fi

AI_LOCAL_DIR=".ai.local"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

# Ensure .ai.local structure exists
mkdir -p "$AI_LOCAL_DIR"/{context,progress,research,session,templates}

# Parse JSON input
if [ ! -t 0 ]; then
    JSON_INPUT=$(cat)
    
    if echo "$JSON_INPUT" | jq . >/dev/null 2>&1; then
        EVENT=$(echo "$JSON_INPUT" | jq -r '.hook_event_name // empty')
        TOOL_NAME=$(echo "$JSON_INPUT" | jq -r '.tool_name // empty')
        TOOL_INPUT=$(echo "$JSON_INPUT" | jq -r '.tool_input // empty')
        
        # Only process PostToolUse events for file operations
        if [[ "$EVENT" == "PostToolUse" ]] && [[ "$TOOL_NAME" =~ ^(Edit|Write|MultiEdit)$ ]]; then
            FILE_PATH=$(echo "$TOOL_INPUT" | jq -r '.file_path // empty')
            
            # Log file modification
            if [[ -n "$FILE_PATH" ]]; then
                ACTIVITY_FILE="$AI_LOCAL_DIR/progress/file-changes.md"
                
                if [[ ! -f "$ACTIVITY_FILE" ]]; then
                    echo "# File Modification Log" > "$ACTIVITY_FILE"
                    echo "" >> "$ACTIVITY_FILE"
                fi
                
                {
                    echo "## $TIMESTAMP - File Modified"
                    echo "- **File**: \`$FILE_PATH\`"
                    echo "- **Tool**: $TOOL_NAME"
                    echo ""
                } >> "$ACTIVITY_FILE"
            fi
            
            # Update session activity
            SESSION_FILE="$AI_LOCAL_DIR/session/current-session.md"
            
            if [[ ! -f "$SESSION_FILE" ]]; then
                PROJECT_DIR=$(pwd)
                GIT_BRANCH=$(git branch --show-current 2>/dev/null || echo "N/A")
                
                cat > "$SESSION_FILE" <<EOF
# Current Session

**Started**: $TIMESTAMP  
**Working Directory**: $PROJECT_DIR  
**Git Branch**: $GIT_BRANCH

## Activities This Session
EOF
            fi
            
            # Update last activity
            LAST_ACTIVITY="**Last Activity**: $TIMESTAMP - Used $TOOL_NAME"
            if [[ -n "$FILE_PATH" ]]; then
                LAST_ACTIVITY="$LAST_ACTIVITY on \`$FILE_PATH\`"
            fi
            
            if grep -q "Last Activity" "$SESSION_FILE"; then
                # Use a simpler sed approach
                sed -i.bak '/Last Activity/d' "$SESSION_FILE" 2>/dev/null || true
                rm -f "${SESSION_FILE}.bak" 2>/dev/null || true
            fi
            
            echo "" >> "$SESSION_FILE"
            echo "$LAST_ACTIVITY" >> "$SESSION_FILE"
            
            # Commit to memory git repo if it exists
            if [[ -d "$AI_LOCAL_DIR/.git" ]]; then
                (
                    cd "$AI_LOCAL_DIR" || exit 1
                    git add -A 2>/dev/null
                    if ! git diff --cached --quiet 2>/dev/null; then
                        project_name=$(basename "$(dirname "$(pwd)")")
                        git commit --quiet -m "mem($project_name): Context capture at $TIMESTAMP" 2>/dev/null || true
                    fi
                ) || true
            fi
        fi
    fi
fi

exit 0