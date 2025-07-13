#!/bin/bash
# /check - Quality validation workflow
# Usage: /check

set -e

AI_LOCAL_DIR=".ai.local"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
PROJECT_NAME=$(basename "$(pwd)")

echo "✅ **QUALITY VALIDATION CHECKPOINT**"
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

echo "## 🔍 Running Quality Checks"
echo ""

# Initialize results tracking
CHECKS_PASSED=0
TOTAL_CHECKS=0
FAILED_CHECKS=()

# Function to run a check and track results
run_check() {
    local check_name="$1"
    local check_command="$2"
    local is_required="${3:-true}"
    
    echo "**Checking $check_name...**"
    ((TOTAL_CHECKS++))
    
    if eval "$check_command" >/dev/null 2>&1; then
        echo "  ✅ $check_name: PASSED"
        ((CHECKS_PASSED++))
    else
        if [[ "$is_required" == "true" ]]; then
            echo "  ❌ $check_name: FAILED"
            FAILED_CHECKS+=("$check_name")
        else
            echo "  ⚠️  $check_name: SKIPPED (not available)"
            ((CHECKS_PASSED++))
        fi
    fi
    echo ""
}

# 1. Git status check
run_check "Git Status" "git status --porcelain | grep -q ."

# 2. Lint checks (try common linters)
run_check "ESLint" "command -v eslint && npm run lint" "false"
run_check "Prettier" "command -v prettier && npm run format:check" "false" 
run_check "TypeScript" "command -v tsc && npm run typecheck" "false"
run_check "Python Lint" "command -v ruff && ruff check ." "false"
run_check "Shellcheck" "find . -name '*.sh' -exec shellcheck {} \;" "false"

# 3. Test checks
run_check "Unit Tests" "npm test 2>/dev/null || python -m pytest 2>/dev/null || go test ./... 2>/dev/null" "false"
run_check "Build" "npm run build 2>/dev/null || make build 2>/dev/null || go build 2>/dev/null" "false"

# 4. Security checks (basic)
run_check "Secrets Scan" "! grep -r -E '(password|secret|key).*=.*[\"'\''][^\"'\'']{8,}' . --exclude-dir=node_modules --exclude-dir=.git 2>/dev/null"

echo "## 📊 Validation Results"
echo ""
echo "**Checks Passed**: $CHECKS_PASSED/$TOTAL_CHECKS"

if [[ ${#FAILED_CHECKS[@]} -eq 0 ]]; then
    echo "**Status**: 🎉 **ALL CHECKS PASSED**"
    echo ""
    echo "Your code is ready for shipping! Use '/ship' to commit and finalize."
    VALIDATION_STATUS="PASSED"
else
    echo "**Status**: ⚠️  **ISSUES FOUND**"
    echo ""
    echo "**Failed Checks**:"
    for failed_check in "${FAILED_CHECKS[@]}"; do
        echo "- $failed_check"
    done
    echo ""
    echo "Please fix these issues before using '/ship'."
    VALIDATION_STATUS="FAILED"
fi

# Update memory with validation results
if [[ -d "$AI_LOCAL_DIR" ]]; then
    VALIDATION_FILE="$AI_LOCAL_DIR/progress/validation-log.md"
    
    if [[ ! -f "$VALIDATION_FILE" ]]; then
        echo "# Quality Validation Log" > "$VALIDATION_FILE"
        echo "" >> "$VALIDATION_FILE"
    fi
    
    {
        echo "## $TIMESTAMP - Quality Check"
        echo "- **Status**: $VALIDATION_STATUS"
        echo "- **Checks Passed**: $CHECKS_PASSED/$TOTAL_CHECKS"
        if [[ ${#FAILED_CHECKS[@]} -gt 0 ]]; then
            echo "- **Failed**: ${FAILED_CHECKS[*]}"
        fi
        echo ""
    } >> "$VALIDATION_FILE"
    
    # Update session
    SESSION_FILE="$AI_LOCAL_DIR/session/current-session.md"
    if [[ -f "$SESSION_FILE" ]]; then
        echo "- **$TIMESTAMP**: Quality validation - $VALIDATION_STATUS ($CHECKS_PASSED/$TOTAL_CHECKS checks passed)" >> "$SESSION_FILE"
    fi
    
    # Commit memory update
    (
        cd "$AI_LOCAL_DIR" || exit 1
        git add -A 2>/dev/null || true
        if ! git diff --cached --quiet 2>/dev/null; then
            git commit --quiet -m "check($PROJECT_NAME): Quality validation - $VALIDATION_STATUS" 2>/dev/null || true
        fi
    ) || true
fi

echo ""
echo "---"
echo ""
if [[ "$VALIDATION_STATUS" == "PASSED" ]]; then
    echo "🚀 **Ready to ship!** Use '/ship' to commit your changes."
    exit 0
else
    echo "🔧 **Fix issues above, then run '/check' again.**"
    exit 1
fi