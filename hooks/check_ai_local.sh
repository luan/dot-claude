#!/bin/bash
# Hook: check_ai_local
# Purpose: Automatically check and load .ai.local context when starting work

AI_LOCAL_DIR=".ai.local"

# Check if we're in a git repo or project directory
if [ -d ".git" ] || [ -f "package.json" ] || [ -f "Cargo.toml" ] || [ -f "pyproject.toml" ]; then
    if [ -d "$AI_LOCAL_DIR" ]; then
        echo "🧠 Found .ai.local directory - loading previous context..."
        
        # Check for last session info
        if [ -f "$AI_LOCAL_DIR/session/last-session.md" ]; then
            echo "📝 Last session summary available"
        fi
        
        # Check for current progress
        if [ -f "$AI_LOCAL_DIR/progress/current.md" ]; then
            echo "📊 Active tasks found in progress tracking"
        fi
        
        # Check for architectural context
        if [ -f "$AI_LOCAL_DIR/context/architecture.md" ]; then
            echo "🏗️ Architecture documentation available"
        fi
    else
        echo "🧠 No .ai.local directory found - this appears to be a fresh start"
        echo "💡 Creating .ai.local structure..."
        
        mkdir -p "$AI_LOCAL_DIR"/{context,progress,research,session}
        
        # Copy README template if it exists
        if [ -f ~/.claude/.ai.local/README.md ]; then
            cp ~/.claude/.ai.local/README.md "$AI_LOCAL_DIR/"
            echo "📋 Added .ai.local usage guide"
        fi
        
        # Create initial files
        echo "# Current Tasks" > "$AI_LOCAL_DIR/progress/current.md"
        echo "Created: $(date '+%Y-%m-%d %H:%M')" >> "$AI_LOCAL_DIR/progress/current.md"
        
        echo "# Architecture Notes" > "$AI_LOCAL_DIR/context/architecture.md"
        echo "# Key Decisions" > "$AI_LOCAL_DIR/context/decisions.md"
        echo "# Discovered Patterns" > "$AI_LOCAL_DIR/context/patterns.md"
        
        echo "# Completed Tasks" > "$AI_LOCAL_DIR/progress/completed.md"
        echo "# Task Backlog" > "$AI_LOCAL_DIR/progress/backlog.md"
        echo "# Dependencies Research" > "$AI_LOCAL_DIR/research/dependencies.md"
        echo "# Solutions & Patterns" > "$AI_LOCAL_DIR/research/solutions.md"
        echo "# References & Links" > "$AI_LOCAL_DIR/research/references.md"
        
        echo "✅ .ai.local directory initialized with template files"
    fi
fi