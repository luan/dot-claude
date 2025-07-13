#!/bin/bash
# /status - Load context and display current progress
# Usage: /status

set -e

AI_LOCAL_DIR=".ai.local"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
PROJECT_NAME=$(basename "$(pwd)")

# Check if memory tracking exists
if [[ ! -d "$AI_LOCAL_DIR" ]]; then
    echo "🧠 **NO ACTIVE WORKFLOW**"
    echo ""
    echo "No workflow memory found in this directory."
    echo ""
    echo "To start a workflow:"
    echo "- Use '/plan [description]' for complex projects"
    echo "- Use '/next [description]' for simple tasks"
    exit 0
fi

echo "🧠 **WORKFLOW STATUS REPORT**"
echo ""
echo "**Project**: $PROJECT_NAME"
echo "**Directory**: $(pwd)"
echo "**Git Branch**: $(git branch --show-current 2>/dev/null || echo "N/A")"
echo "**Report Time**: $TIMESTAMP"
echo ""

# Load and display current session
SESSION_FILE="$AI_LOCAL_DIR/session/current-session.md"
if [[ -f "$SESSION_FILE" ]]; then
    echo "## 🚀 Current Session"
    echo ""
    # Extract key info from session file
    if grep -q "Mode.*Complex Project" "$SESSION_FILE"; then
        echo "**Workflow Mode**: Complex Project (/plan)"
        
        # Show project scope if available
        if grep -q "Project Scope" "$SESSION_FILE"; then
            echo ""
            echo "**Project Scope**:"
            sed -n '/## Project Scope/,/## /p' "$SESSION_FILE" | sed '1d;$d' | sed 's/^/  /'
        fi
        
    elif grep -q "Mode.*Simple Task" "$SESSION_FILE"; then
        echo "**Workflow Mode**: Simple Task (/next)"
        
        # Show current task if available
        if grep -q "Current Task" "$SESSION_FILE"; then
            echo ""
            echo "**Current Task**:"
            sed -n '/## Current Task/,/## /p' "$SESSION_FILE" | sed '1d;$d' | sed 's/^/  /'
        fi
    fi
    
    # Show last activity
    if grep -q "Last Activity" "$SESSION_FILE"; then
        echo ""
        LAST_ACTIVITY=$(grep "Last Activity" "$SESSION_FILE" | head -1)
        echo "**$LAST_ACTIVITY**"
    fi
    
    echo ""
fi

# Load and display current progress
PROGRESS_FILE="$AI_LOCAL_DIR/progress/current-task.md"
PLAN_FILE="$AI_LOCAL_DIR/context/project-plan.md"

if [[ -f "$PROGRESS_FILE" ]]; then
    echo "## ✅ Task Progress"
    echo ""
    
    # Extract and display task status
    if grep -q "Implementation Status" "$PROGRESS_FILE"; then
        sed -n '/## Implementation Status/,/## /p' "$PROGRESS_FILE" | sed '1d;$d' | while read -r line; do
            if [[ "$line" =~ ^\- ]]; then
                echo "$line"
            fi
        done
    fi
    echo ""
    
elif [[ -f "$PLAN_FILE" ]]; then
    echo "## 📋 Project Progress"
    echo ""
    
    # Extract and display project progress
    if grep -q "Progress Tracking" "$PLAN_FILE"; then
        sed -n '/## Progress Tracking/,/## /p' "$PLAN_FILE" | sed '1d;$d' | while read -r line; do
            if [[ "$line" =~ ^\- ]]; then
                echo "$line"
            fi
        done
    fi
    echo ""
fi

# Show recent file changes if any
CHANGES_FILE="$AI_LOCAL_DIR/progress/file-changes.md"
if [[ -f "$CHANGES_FILE" ]]; then
    echo "## 📁 Recent File Changes"
    echo ""
    
    # Show last 5 file modifications
    grep -E "^\- \*\*File\*\*:" "$CHANGES_FILE" | tail -5 | while read -r line; do
        echo "$line"
    done
    echo ""
fi

# Update session with status check
echo "- **$TIMESTAMP**: Checked workflow status" >> "$SESSION_FILE"

# Commit memory update
(
    cd "$AI_LOCAL_DIR" || exit 1
    git add -A 2>/dev/null || true
    if ! git diff --cached --quiet 2>/dev/null; then
        git commit --quiet -m "status($PROJECT_NAME): Status check at $TIMESTAMP" 2>/dev/null || true
    fi
) || true

echo "---"
echo ""
echo "🎯 **Next Steps**:"
echo "- Continue working on current task/project"
echo "- Use '/check' to validate progress before completion"
echo "- Use '/ship' when ready to commit and finalize"