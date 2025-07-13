#!/bin/bash
# /ship - Commit and finalize workflow
# Usage: /ship

set -e

AI_LOCAL_DIR=".ai.local"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
PROJECT_NAME=$(basename "$(pwd)")

echo "🚀 **SHIPPING WORKFLOW**"
echo ""
echo "**Project**: $PROJECT_NAME"
echo "**Time**: $TIMESTAMP"
echo ""

# Check if workflow is active
if [[ ! -d "$AI_LOCAL_DIR" ]]; then
    echo "🚨 **NO ACTIVE WORKFLOW**"
    echo ""
    echo "No workflow found. Start with '/plan' or '/next' first."
    exit 1
fi

# Check if we're in a git repository
if [[ ! -d ".git" ]]; then
    echo "🚨 **NOT A GIT REPOSITORY**"
    echo ""
    echo "This directory is not a git repository. Initialize git first:"
    echo "  git init"
    echo "  git add ."
    echo "  git commit -m 'Initial commit'"
    exit 1
fi

# Check for uncommitted changes
if ! git diff --cached --quiet 2>/dev/null; then
    echo "📋 **STAGED CHANGES DETECTED**"
    echo ""
    echo "You have staged changes. Proceeding with commit..."
    echo ""
elif ! git diff --quiet 2>/dev/null; then
    echo "📋 **UNSTAGED CHANGES DETECTED**"
    echo ""
    echo "Staging all changes for commit..."
    git add -A
    echo ""
else
    echo "ℹ️  **NO CHANGES TO COMMIT**"
    echo ""
    echo "Working directory is clean. Nothing to ship."
    exit 0
fi

echo "## 🔍 Pre-Ship Validation"
echo ""

# Run final validation check
VALIDATION_PASSED="false"
if [[ -f "$AI_LOCAL_DIR/progress/validation-log.md" ]]; then
    # Check last validation result
    if tail -10 "$AI_LOCAL_DIR/progress/validation-log.md" | grep -q "Status.*PASSED"; then
        echo "✅ **Previous validation: PASSED**"
        VALIDATION_PASSED="true"
    else
        echo "❌ **Previous validation: FAILED**"
        echo ""
        echo "Please run '/check' and fix all issues before shipping."
        exit 1
    fi
else
    echo "⚠️  **No validation history found**"
    echo ""
    echo "Running quick validation..."
    
    # Quick validation
    if command -v npm >/dev/null 2>&1 && [[ -f "package.json" ]]; then
        if npm run lint >/dev/null 2>&1 && npm run typecheck >/dev/null 2>&1; then
            echo "✅ **Quick validation: PASSED**"
            VALIDATION_PASSED="true"
        else
            echo "❌ **Quick validation: FAILED**"
            echo ""
            echo "Please run '/check' to see detailed issues."
            exit 1
        fi
    else
        echo "⚠️  **Skipping validation (no package.json)**"
        VALIDATION_PASSED="true"
    fi
fi

echo ""

if [[ "$VALIDATION_PASSED" == "true" ]]; then
    echo "## 📝 Creating Commit"
    echo ""
    
    # Generate commit message based on workflow type
    COMMIT_MSG=""
    
    # Check workflow type from session
    SESSION_FILE="$AI_LOCAL_DIR/session/current-session.md"
    if [[ -f "$SESSION_FILE" ]]; then
        if grep -q "Mode.*Complex Project" "$SESSION_FILE"; then
            # Complex project commit
            PROJECT_DESC=$(grep -A5 "Project Scope" "$SESSION_FILE" | tail -n +2 | head -1 | sed 's/^[[:space:]]*//')
            if [[ -n "$PROJECT_DESC" ]]; then
                COMMIT_MSG="feat: $PROJECT_DESC"
            else
                COMMIT_MSG="feat: implement complex project features"
            fi
            
        elif grep -q "Mode.*Simple Task" "$SESSION_FILE"; then
            # Simple task commit
            TASK_DESC=$(grep -A5 "Current Task" "$SESSION_FILE" | tail -n +2 | head -1 | sed 's/^[[:space:]]*//')
            if [[ -n "$TASK_DESC" ]]; then
                # Determine commit type based on task description
                if echo "$TASK_DESC" | grep -q -i "fix\|bug\|error\|issue"; then
                    COMMIT_MSG="fix: $TASK_DESC"
                elif echo "$TASK_DESC" | grep -q -i "test\|spec"; then
                    COMMIT_MSG="test: $TASK_DESC"
                elif echo "$TASK_DESC" | grep -q -i "doc\|readme"; then
                    COMMIT_MSG="docs: $TASK_DESC"
                elif echo "$TASK_DESC" | grep -q -i "refactor\|clean\|optimize"; then
                    COMMIT_MSG="refactor: $TASK_DESC"
                else
                    COMMIT_MSG="feat: $TASK_DESC"
                fi
            else
                COMMIT_MSG="feat: implement task"
            fi
        fi
    fi
    
    # Fallback commit message
    if [[ -z "$COMMIT_MSG" ]]; then
        COMMIT_MSG="feat: implement workflow changes"
    fi
    
    echo "**Commit Message**: $COMMIT_MSG"
    echo ""
    
    # Create the commit
    if git commit -m "$COMMIT_MSG"; then
        echo "✅ **COMMIT SUCCESSFUL**"
        COMMIT_HASH=$(git rev-parse --short HEAD)
        echo "**Commit Hash**: $COMMIT_HASH"
        echo ""
        
        # Update memory with shipping completion
        {
            echo "## $TIMESTAMP - Shipped!"
            echo "- **Commit**: $COMMIT_HASH"
            echo "- **Message**: $COMMIT_MSG"
            echo "- **Status**: Successfully shipped"
            echo ""
        } >> "$AI_LOCAL_DIR/progress/shipping-log.md"
        
        # Mark workflow as completed in session
        if [[ -f "$SESSION_FILE" ]]; then
            {
                echo "- **$TIMESTAMP**: 🚀 SHIPPED - Commit $COMMIT_HASH: $COMMIT_MSG"
                echo ""
                echo "**WORKFLOW COMPLETED** ✅"
            } >> "$SESSION_FILE"
        fi
        
        # Final memory commit
        (
            cd "$AI_LOCAL_DIR" || exit 1
            git add -A 2>/dev/null || true
            if ! git diff --cached --quiet 2>/dev/null; then
                git commit --quiet -m "ship($PROJECT_NAME): Completed workflow - $COMMIT_HASH" 2>/dev/null || true
            fi
        ) || true
        
        echo "## 🎉 Workflow Complete!"
        echo ""
        echo "**Summary**:"
        echo "- ✅ Code validated and tested"
        echo "- ✅ Changes committed: $COMMIT_HASH"
        echo "- ✅ Workflow memory preserved"
        echo ""
        echo "🎯 **Next Steps**:"
        echo "- Review the commit: \`git show $COMMIT_HASH\`"
        echo "- Push to remote: \`git push\`"
        echo "- Start new workflow: '/plan' or '/next'"
        
    else
        echo "❌ **COMMIT FAILED**"
        echo ""
        echo "Please check git status and try again."
        exit 1
    fi
else
    echo "❌ **SHIPPING BLOCKED**"
    echo ""
    echo "Validation must pass before shipping. Run '/check' first."
    exit 1
fi