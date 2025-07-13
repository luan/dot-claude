#!/bin/bash
# /next - Simple task workflow with progress tracking
# Usage: /next [task description]

set -e

# Get task description from arguments
TASK_DESC="$*"

if [[ -z "$TASK_DESC" ]]; then
    echo "🚨 **WORKFLOW ENFORCEMENT:** /next requires a task description"
    echo ""
    echo "Usage: /next [describe your simple task]"
    echo "Example: /next fix the login button styling issue"
    exit 1
fi

# Memory setup - transparent to user
AI_LOCAL_DIR=".ai.local"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
PROJECT_NAME=$(basename "$(pwd)")

# Initialize memory structure if needed
mkdir -p "$AI_LOCAL_DIR"/{context,progress,research,session,templates}

# Initialize git repo for memory tracking if needed
if [[ ! -d "$AI_LOCAL_DIR/.git" ]]; then
    (
        cd "$AI_LOCAL_DIR" || exit 1
        git init --quiet
        git config user.name "Claude Code Workflows"
        git config user.email "workflows@anthropic.com"
        
        # Create .gitignore for sensitive data
        {
            echo "# Claude Code Workflow Memory"
            echo "*.secret"
            echo "*.key"
            echo "temp/"
        } > .gitignore
        
        git add .gitignore
        git commit --quiet -m "init: Initialize workflow memory tracking"
    )
fi

# Check for existing project context and preserve it
SESSION_FILE="$AI_LOCAL_DIR/session/current-session.md"
TASK_TYPE="Simple Task (/next)"
SESSION_TITLE="Simple Task"

# Preserve complex project mode if active
if [[ -f "$SESSION_FILE" ]] && grep -q "Mode.*Complex Project" "$SESSION_FILE"; then
    TASK_TYPE="Complex Project Task (/next)"
    SESSION_TITLE="Complex Project Task"
    echo "🧠 **PRESERVING COMPLEX PROJECT CONTEXT**"
    echo "Adding task to existing complex project workflow"
    echo ""
fi

# Create or update task progress file
PROGRESS_FILE="$AI_LOCAL_DIR/progress/current-task.md"
cat > "$PROGRESS_FILE" <<EOF
# Current Task: $TASK_DESC

**Created**: $TIMESTAMP  
**Project**: $PROJECT_NAME  
**Working Directory**: $(pwd)  
**Git Branch**: $(git branch --show-current 2>/dev/null || echo "N/A")  
**Task Type**: $TASK_TYPE

## Task Description

$TASK_DESC

## Implementation Status

- [ ] Research and understand requirements
- [ ] Plan implementation approach
- [ ] Implement solution
- [ ] Test and validate
- [ ] Complete task

## Progress Log

- **$TIMESTAMP**: Task initiated via /next workflow

## Notes

_Implementation details and decisions will be captured here..._

---
*This progress is automatically maintained by Claude Code workflows*
EOF

# Update session tracking - preserve existing session if complex project
if [[ -f "$SESSION_FILE" ]] && grep -q "Mode.*Complex Project" "$SESSION_FILE"; then
    # Append to existing complex project session instead of overwriting
    {
        echo "- **$TIMESTAMP**: Added task via /next workflow: $TASK_DESC"
        echo ""
        echo "**Last Activity**: $TIMESTAMP - Added task: $TASK_DESC"
    } >> "$SESSION_FILE"
else
    # Create new simple task session
    cat > "$SESSION_FILE" <<EOF
# Current Session: $SESSION_TITLE

**Started**: $TIMESTAMP  
**Mode**: $TASK_TYPE  
**Project**: $PROJECT_NAME  
**Working Directory**: $(pwd)  
**Git Branch**: $(git branch --show-current 2>/dev/null || echo "N/A")

## Current Task
$TASK_DESC

## Session Activities
- **$TIMESTAMP**: Started simple task workflow

**Last Activity**: $TIMESTAMP - Initiated task: $TASK_DESC
EOF
fi

# Commit memory updates
(
    cd "$AI_LOCAL_DIR" || exit 1
    git add -A
    if ! git diff --cached --quiet; then
        git commit --quiet -m "next($PROJECT_NAME): Start simple task - $TASK_DESC"
    fi
)

# Output workflow confirmation
if [[ "$TASK_TYPE" == *"Complex Project"* ]]; then
    echo "🚀 **COMPLEX PROJECT TASK ADDED**"
    echo ""
    echo "**Task**: $TASK_DESC"
    echo "**Mode**: Complex project workflow preserved"
    echo "**Context**: Task added to existing project plan"
else
    echo "🚀 **SIMPLE TASK WORKFLOW INITIATED**"
    echo ""
    echo "**Task**: $TASK_DESC"
    echo "**Mode**: Simple task workflow active"
fi
echo "**Memory tracking**: Progress logged in .ai.local/"
echo ""
echo "✅ Claude will now research → plan → implement this task using the structured workflow."
echo ""
echo "Progress will be tracked automatically. Use '/status' anytime to check progress."