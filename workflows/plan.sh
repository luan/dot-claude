#!/bin/bash
# /plan - Complex project planning workflow with transparent memory management
# Usage: /plan [project description]

set -e

# Get project description from arguments
PROJECT_DESC="$*"

if [[ -z "$PROJECT_DESC" ]]; then
    echo "🚨 **WORKFLOW ENFORCEMENT:** /plan requires a project description"
    echo ""
    echo "Usage: /plan [describe your complex project]"
    echo "Example: /plan implement user authentication system with OAuth and JWT"
    exit 1
fi

# Memory setup - transparent to user
AI_LOCAL_DIR=".ai.local"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
PROJECT_NAME=$(basename "$(pwd)")

# Initialize memory structure
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

# Create project plan file
PLAN_FILE="$AI_LOCAL_DIR/context/project-plan.md"
cat > "$PLAN_FILE" <<EOF
# Project Plan: $PROJECT_DESC

**Created**: $TIMESTAMP  
**Project**: $PROJECT_NAME  
**Working Directory**: $(pwd)  
**Git Branch**: $(git branch --show-current 2>/dev/null || echo "N/A")

## Project Objectives

$PROJECT_DESC

## Implementation Plan

_Plan will be developed during Claude interaction..._

## Progress Tracking

- [ ] Initial research and analysis
- [ ] Architecture design
- [ ] Implementation breakdown
- [ ] Development phases
- [ ] Testing strategy
- [ ] Deployment planning

## Notes

_Research findings and decisions will be captured here..._

---
*This plan is automatically maintained by Claude Code workflows*
EOF

# Update session tracking
SESSION_FILE="$AI_LOCAL_DIR/session/current-session.md"
cat > "$SESSION_FILE" <<EOF
# Current Session: Complex Project Planning

**Started**: $TIMESTAMP  
**Mode**: Complex Project (/plan)  
**Project**: $PROJECT_NAME  
**Working Directory**: $(pwd)  
**Git Branch**: $(git branch --show-current 2>/dev/null || echo "N/A")

## Project Scope
$PROJECT_DESC

## Session Activities
- **$TIMESTAMP**: Started complex project planning workflow

**Last Activity**: $TIMESTAMP - Initialized project planning for: $PROJECT_DESC
EOF

# Commit memory updates
(
    cd "$AI_LOCAL_DIR" || exit 1
    git add -A
    if ! git diff --cached --quiet; then
        git commit --quiet -m "plan($PROJECT_NAME): Initialize complex project - $PROJECT_DESC"
    fi
)

# Output workflow confirmation
echo "🧠 **COMPLEX PROJECT PLANNING INITIATED**"
echo ""
echo "**Project**: $PROJECT_DESC"
echo "**Memory tracking**: Initialized transparently in .ai.local/"
echo "**Mode**: Complex project workflow active"
echo ""
echo "🚀 Claude will now research, plan, and implement this complex project using structured workflows."
echo ""
echo "Next steps will be tracked automatically. Use '/status' anytime to check progress."