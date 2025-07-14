#!/usr/bin/env bash
# smart-lint.sh - Intelligent project-aware code quality checks for Claude Code
#
# SYNOPSIS
#   Claude Code PostToolUse hook that runs linting after file edits
#
# DESCRIPTION
#   Automatically detects project type and runs ALL quality checks.
#   Every issue found is blocking - code must be 100% clean to proceed.
#   Test comment to trigger hooks.
#
# INPUT
#   JSON via stdin with PostToolUse event data
#
# OUTPUT
#   Optional JSON response for advanced control
#
# EXIT CODES
#   0 - Continue with operation
#   1 - General error (missing dependencies, etc.)
#   2 - Block operation (linting issues found)
#
# CONFIGURATION
#   Project-specific overrides can be placed in .claude-hooks-config.sh
#   See inline documentation for all available options.
#
#   Go-specific options:
#     CLAUDE_HOOKS_GO_DEADCODE_ENABLED=false  # Disable deadcode analysis (default: true)
#                                             # Note: deadcode can take 5-10 seconds on large projects

# Don't use set -e - we need to control exit codes carefully
set +e

# Source common helpers
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/common-helpers.sh"

# Debug output after sourcing helpers so we have log_debug
log_debug "smart-lint.sh started"

# ============================================================================
# PROJECT DETECTION
# ============================================================================

# Enhanced project type detection with Tilt support
detect_project_type() {
    local types=()
    local project_type="unknown"
    
    # Check for Go projects
    if [[ -f "go.mod" ]] || [[ -f "go.sum" ]] || [[ -n "$(find . -maxdepth 3 -name "*.go" -type f -print -quit 2>/dev/null)" ]]; then
        types+=("go")
    fi
    
    # Check for Python projects
    if [[ -f "pyproject.toml" ]] || [[ -f "setup.py" ]] || [[ -f "requirements.txt" ]] || [[ -n "$(find . -maxdepth 3 -name "*.py" -type f -print -quit 2>/dev/null)" ]]; then
        types+=("python")
    fi
    
    # Check for JavaScript/TypeScript projects
    if [[ -f "package.json" ]] || [[ -f "tsconfig.json" ]] || [[ -n "$(find . -maxdepth 3 \( -name "*.js" -o -name "*.ts" -o -name "*.jsx" -o -name "*.tsx" \) -type f -print -quit 2>/dev/null)" ]]; then
        types+=("javascript")
    fi
    
    # Check for Rust projects
    if [[ -f "Cargo.toml" ]] || [[ -n "$(find . -maxdepth 3 -name "*.rs" -type f -print -quit 2>/dev/null)" ]]; then
        types+=("rust")
    fi
    
    # Check for Nix projects
    if [[ -f "flake.nix" ]] || [[ -f "default.nix" ]] || [[ -f "shell.nix" ]]; then
        types+=("nix")
    fi
    
    # Check for shell projects
    if [[ -n "$(find . -maxdepth 3 -name "*.sh" -type f -print -quit 2>/dev/null)" ]] || [[ -n "$(find . -maxdepth 3 -name "*.bash" -type f -print -quit 2>/dev/null)" ]]; then
        types+=("shell")
    fi
    
    # Check for Tilt projects
    if [[ -f "Tiltfile" ]] || [[ -n "$(find . -maxdepth 3 -name "Tiltfile" -type f -print -quit 2>/dev/null)" ]] || [[ -n "$(find . -maxdepth 3 -name "*.tiltfile" -type f -print -quit 2>/dev/null)" ]]; then
        types+=("tilt")
    fi
    
    # Check for Swift projects
    if [[ -f "Package.swift" ]] || [[ -n "$(find . -maxdepth 3 -name "*.xcodeproj" -type d -print -quit 2>/dev/null)" ]] || [[ -n "$(find . -maxdepth 3 -name "*.xcworkspace" -type d -print -quit 2>/dev/null)" ]] || [[ -n "$(find . -maxdepth 3 -name "*.swift" -type f -print -quit 2>/dev/null)" ]]; then
        types+=("swift")
    fi
    
    # Return result
    if [[ ${#types[@]} -eq 1 ]]; then
        project_type="${types[0]}"
    elif [[ ${#types[@]} -gt 1 ]]; then
        project_type="mixed:$(IFS=,; echo "${types[*]}")"
    fi
    
    log_debug "Detected project type: $project_type"
    echo "$project_type"
}


# ============================================================================
# ERROR TRACKING (extends common-helpers.sh)
# ============================================================================

# Enhanced error handling with immediate feedback
add_error() {
    local message="$1"
    ((CLAUDE_HOOKS_ERROR_COUNT += 1))
    CLAUDE_HOOKS_ERRORS+=("${RED}❌${NC} $message")
    echo -e "${RED}❌${NC} $message" >&2
}

print_error_summary() {
    if [[ $CLAUDE_HOOKS_ERROR_COUNT -gt 0 ]]; then
        echo -e "\n${RED}❌ Found $CLAUDE_HOOKS_ERROR_COUNT blocking issue(s) - fix all above${NC}" >&2
    fi
}

# ============================================================================
# CONFIGURATION LOADING
# ============================================================================

# Configuration defaults
set_defaults() {
    # Core settings
    export CLAUDE_HOOKS_ENABLED="${CLAUDE_HOOKS_ENABLED:-true}"
    export CLAUDE_HOOKS_FAIL_FAST="${CLAUDE_HOOKS_FAIL_FAST:-false}"
    export CLAUDE_HOOKS_SHOW_TIMING="${CLAUDE_HOOKS_SHOW_TIMING:-false}"
    
    # Language toggles
    local languages=("GO" "PYTHON" "JS" "RUST" "NIX" "TILT" "SHELL" "SWIFT")
    for lang in "${languages[@]}"; do
        local var="CLAUDE_HOOKS_${lang}_ENABLED"
        export "$var"="${!var:-true}"
    done
    
    # Project command settings
    export CLAUDE_HOOKS_USE_PROJECT_COMMANDS="${CLAUDE_HOOKS_USE_PROJECT_COMMANDS:-true}"
    export CLAUDE_HOOKS_MAKE_LINT_TARGETS="${CLAUDE_HOOKS_MAKE_LINT_TARGETS:-lint}"
    export CLAUDE_HOOKS_SCRIPT_LINT_NAMES="${CLAUDE_HOOKS_SCRIPT_LINT_NAMES:-lint}"
    
    # Per-language project command settings
    local project_langs=("GO" "PYTHON" "JAVASCRIPT" "RUST" "NIX" "SHELL" "TILT" "SWIFT")
    for lang in "${project_langs[@]}"; do
        local var="CLAUDE_HOOKS_${lang}_USE_PROJECT_COMMANDS"
        export "$var"="${!var:-true}"
    done
}

load_config() {
    set_defaults
    
    # Load project-specific overrides
    if [[ -f ".claude-hooks-config.sh" ]]; then
        # shellcheck disable=SC1091
        source ".claude-hooks-config.sh" || {
            log_error "Failed to load .claude-hooks-config.sh"
            exit 2
        }
    fi
    
    # Early exit if hooks disabled
    if [[ "$CLAUDE_HOOKS_ENABLED" != "true" ]]; then
        log_info "Claude hooks are disabled"
        log_debug "Exiting because CLAUDE_HOOKS_ENABLED=$CLAUDE_HOOKS_ENABLED"
        [[ "${CLAUDE_HOOKS_DEBUG:-0}" == "1" ]] && exit 2
        exit 0
    fi
}

# ============================================================================
# LANGUAGE-SPECIFIC LINTERS
# ============================================================================

# Source language-specific linting functions
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source Go linting if available
if [[ -f "${SCRIPT_DIR}/lint-go.sh" ]]; then
    # shellcheck disable=SC1091
    source "${SCRIPT_DIR}/lint-go.sh"
fi

# Source Tilt linting if available
if [[ -f "${SCRIPT_DIR}/lint-tilt.sh" ]]; then
    # shellcheck disable=SC1091
    source "${SCRIPT_DIR}/lint-tilt.sh"
fi

# Common file finding and filtering logic
find_filtered_files() {
    local pattern="$1"
    local exclude_pattern="$2"
    local max_files="${3:-100}"
    
    local files
    files=$(find . -type f "$pattern" | grep -v -E "$exclude_pattern" | head -"$max_files")
    
    if [[ -z "$files" ]]; then
        return 1
    fi
    
    local filtered_files=""
    for file in $files; do
        if ! should_skip_file "$file"; then
            filtered_files="$filtered_files$file "
        fi
    done
    
    if [[ -z "$filtered_files" ]]; then
        return 1
    fi
    
    echo "$filtered_files"
    return 0
}

# Format files with error handling
format_files() {
    local formatter="$1"
    local files="$2"
    shift 2
    
    if command_exists "$formatter"; then
        if ! echo "$files" | xargs "$formatter" --check "$@" >/dev/null 2>&1; then
            local format_output
            if ! format_output=$(echo "$files" | xargs "$formatter" "$@" 2>&1); then
                add_error "$formatter formatting failed"
                echo "$format_output" >&2
            fi
        fi
    fi
}

# Lint files with error handling
lint_files() {
    local linter="$1"
    local files="$2"
    shift 2
    
    if command_exists "$linter"; then
        local linter_output
        if ! linter_output=$(echo "$files" | xargs "$linter" "$@" 2>&1); then
            add_error "$linter found issues"
            echo "$linter_output" >&2
        fi
    fi
}

lint_python() {
    if [[ "${CLAUDE_HOOKS_PYTHON_ENABLED:-true}" != "true" ]]; then
        log_debug "Python linting disabled"
        return 0
    fi
    
    log_debug "Running Python linters..."
    
    local py_files
    if ! py_files=$(find_filtered_files "-name '*.py'" "(venv/|\.venv/|__pycache__|\.git/)" 100); then
        log_debug "No Python files found or all were skipped"
        return 0
    fi
    
    # Format and lint
    format_files "black" "$py_files"
    lint_files "ruff" "$py_files" "check" "--fix"
    if ! command_exists ruff; then
        lint_files "flake8" "$py_files"
    fi
    
    return 0
}

lint_javascript() {
    if [[ "${CLAUDE_HOOKS_JS_ENABLED:-true}" != "true" ]]; then
        log_debug "JavaScript linting disabled"
        return 0
    fi
    
    log_debug "Running JavaScript/TypeScript linters..."
    
    local js_files
    if ! js_files=$(find . \( -name "*.js" -o -name "*.ts" -o -name "*.jsx" -o -name "*.tsx" \) -type f | grep -v -E "(node_modules/|dist/|build/|\.git/)" | head -100); then
        log_debug "No JavaScript/TypeScript files found"
        return 0
    fi
    
    # Filter files
    local filtered_files=""
    for file in $js_files; do
        if ! should_skip_file "$file"; then
            filtered_files="$filtered_files$file "
        fi
    done
    
    if [[ -z "$filtered_files" ]]; then
        log_debug "All JavaScript/TypeScript files were skipped"
        return 0
    fi
    
    # ESLint via npm
    if [[ -f "package.json" ]] && grep -q "eslint" package.json 2>/dev/null && command_exists npm; then
        local eslint_output
        if ! eslint_output=$(npm run lint --if-present 2>&1); then
            add_error "ESLint found issues"
            echo "$eslint_output" >&2
        fi
    fi
    
    # Prettier formatting
    if [[ -f ".prettierrc" ]] || [[ -f "prettier.config.js" ]] || [[ -f ".prettierrc.json" ]]; then
        if command_exists prettier; then
            format_files "prettier" "$filtered_files" "--write"
        elif command_exists npx; then
            if ! echo "$filtered_files" | xargs npx prettier --check >/dev/null 2>&1; then
                local format_output
                if ! format_output=$(echo "$filtered_files" | xargs npx prettier --write 2>&1); then
                    add_error "Prettier formatting failed"
                    echo "$format_output" >&2
                fi
            fi
        fi
    fi
    
    return 0
}

lint_rust() {
    if [[ "${CLAUDE_HOOKS_RUST_ENABLED:-true}" != "true" ]]; then
        log_debug "Rust linting disabled"
        return 0
    fi
    
    log_debug "Running Rust linters..."
    
    if ! find_filtered_files "-name '*.rs'" "(target/|\.git/)" 100 >/dev/null; then
        log_debug "No Rust files found or all were skipped"
        return 0
    fi
    
    if command_exists cargo; then
        # Format check and fix
        if ! cargo fmt -- --check >/dev/null 2>&1; then
            local format_output
            if ! format_output=$(cargo fmt 2>&1); then
                add_error "Rust formatting failed"
                echo "$format_output" >&2
            fi
        fi
        
        # Clippy linting
        local clippy_output
        if ! clippy_output=$(cargo clippy --quiet -- -D warnings 2>&1); then
            add_error "Clippy found issues"
            echo "$clippy_output" >&2
        fi
    else
        log_debug "Cargo not found, skipping Rust checks"
    fi
    
    return 0
}

lint_nix() {
    if [[ "${CLAUDE_HOOKS_NIX_ENABLED:-true}" != "true" ]]; then
        log_debug "Nix linting disabled"
        return 0
    fi
    
    log_debug "Running Nix linters..."
    
    local nix_files
    if ! nix_files=$(find_filtered_files "-name '*.nix'" "(result/|/nix/store/)" 20); then
        log_debug "No Nix files found or all were skipped"
        return 0
    fi
    
    # Format with nixpkgs-fmt or alejandra
    if command_exists nixpkgs-fmt; then
        format_files "nixpkgs-fmt" "$nix_files"
    elif command_exists alejandra; then
        format_files "alejandra" "$nix_files"
    fi
    
    # Static analysis with statix
    if command_exists statix; then
        local statix_output
        if ! statix_output=$(statix check 2>&1); then
            add_error "Statix found issues"
            echo "$statix_output" >&2
        fi
    fi
    
    return 0
}

# ============================================================================
# SHELL SCRIPT LINTING
# ============================================================================

# Find shell scripts by extension and shebang
find_shell_scripts() {
    local shell_files
    shell_files=$(find . -type f \( -name "*.sh" -o -name "*.bash" -o -name "*.zsh" \) | grep -v -E "(\.git/|node_modules/|venv/)" | head -50)
    
    local shebang_files
    shebang_files=$(grep -r -l "^#!.*\(bash\|sh\|zsh\)" . --include="*" 2>/dev/null | grep -v -E "(\.git/|node_modules/|venv/)" | head -50)
    
    # Combine and deduplicate
    echo -e "$shell_files\n$shebang_files" | sort -u | grep -v "^$"
}

lint_swift() {
    if [[ "${CLAUDE_HOOKS_SWIFT_ENABLED:-true}" != "true" ]]; then
        log_debug "Swift linting disabled"
        return 0
    fi
    
    log_debug "Running Swift linters..."
    
    local swift_files
    swift_files=$(find . -name "*.swift" -type f | grep -v -E "(build/|DerivedData/|\.git/|Pods/)" | head -100)
    
    if [[ -z "$swift_files" ]]; then
        log_debug "No Swift files found"
        return 0
    fi
    
    # Filter files
    local filtered_files=""
    for file in $swift_files; do
        if ! should_skip_file "$file"; then
            filtered_files="$filtered_files$file "
        fi
    done
    
    if [[ -z "$filtered_files" ]]; then
        log_debug "All Swift files were skipped"
        return 0
    fi
    
    swift_files="$filtered_files"
    
    # SwiftFormat - formatting
    if command_exists swiftformat; then
        format_files "swiftformat" "$swift_files"
    fi
    
    # SwiftLint - linting
    if command_exists swiftlint; then
        local swiftlint_output
        if ! swiftlint_output=$(swiftlint --quiet 2>&1); then
            add_error "SwiftLint found issues"
            echo "$swiftlint_output" >&2
        fi
    fi
    
    return 0
}

lint_shell() {
    if [[ "${CLAUDE_HOOKS_SHELL_ENABLED:-true}" != "true" ]]; then
        log_debug "Shell linting disabled"
        return 0
    fi
    
    log_debug "Running Shell linters..."
    
    local shell_files
    shell_files=$(find_shell_scripts)
    
    if [[ -z "$shell_files" ]]; then
        log_debug "No shell scripts found"
        return 0
    fi
    
    # Filter files
    local filtered_files=""
    for file in $shell_files; do
        if ! should_skip_file "$file"; then
            filtered_files="$filtered_files$file "
        fi
    done
    
    if [[ -z "$filtered_files" ]]; then
        log_debug "All shell scripts were skipped"
        return 0
    fi
    
    # Shellcheck
    if command_exists shellcheck; then
        log_debug "Running shellcheck..."
        local shellcheck_errors=false
        
        for file in $filtered_files; do
            if ! shellcheck -x "$file" 2>&1; then
                shellcheck_errors=true
                add_error "shellcheck violations in $file"
            fi
        done
        
        [[ "$shellcheck_errors" == "false" ]] && log_success "shellcheck passed"
    else
        log_debug "shellcheck not found - skipping shell script validation"
    fi
    
    # Format check with shfmt
    if command_exists shfmt; then
        log_debug "Running shfmt..."
        local format_errors=false
        
        for file in $filtered_files; do
            if ! shfmt -d "$file" >/dev/null 2>&1; then
                format_errors=true
                echo -e "${RED}❌ Formatting issues in: $file${NC}" >&2
                echo "Run: shfmt -w $file" >&2
                add_error "Shell formatting issues in $file"
            fi
        done
        
        [[ "$format_errors" == "false" ]] && log_success "shfmt passed"
    else
        log_debug "shfmt not found - skipping shell formatting check"
    fi
    
    return 0
}

# ============================================================================
# HOOK INPUT PARSING
# ============================================================================

# This script only works as a Claude Code hook (JSON on stdin)
JSON_INPUT=""

# Ensure jq is available for JSON parsing
if ! command_exists jq; then
    log_error "jq is required for JSON parsing but not found"
    exit 1
fi

if [ ! -t 0 ]; then
    # We have input on stdin - try to read it
    log_debug "Reading JSON from stdin"
    JSON_INPUT=$(cat)
    
    # Check if it's valid JSON
    if echo "$JSON_INPUT" | jq . >/dev/null 2>&1; then
        log_debug "Valid JSON input"
        
        
        # Extract relevant fields from the JSON
        EVENT=$(echo "$JSON_INPUT" | jq -r '.hook_event_name // empty')
        TOOL_NAME=$(echo "$JSON_INPUT" | jq -r '.tool_name // empty')
        TOOL_INPUT=$(echo "$JSON_INPUT" | jq -r '.tool_input // empty')
        
        log_debug "Event: $EVENT, Tool: $TOOL_NAME"
        
        # Only process edit-related tools
        if [[ "$EVENT" == "PostToolUse" ]] && [[ "$TOOL_NAME" =~ ^(Edit|Write|MultiEdit)$ ]]; then
            log_debug "Processing $TOOL_NAME operation"
            # Extract file path(s) that were edited
            if [[ "$TOOL_NAME" == "MultiEdit" ]]; then
                FILE_PATH=$(echo "$TOOL_INPUT" | jq -r '.file_path // empty')
            else
                FILE_PATH=$(echo "$TOOL_INPUT" | jq -r '.file_path // empty')
            fi
            
            # Change to the directory of the edited file
            if [[ -n "$FILE_PATH" ]] && [[ -f "$FILE_PATH" ]]; then
                FILE_DIR=$(dirname "$FILE_PATH")
                cd "$FILE_DIR" || true
                log_debug "Changed to file directory: $(pwd)"
                # Update FILE_PATH to just the basename since we've changed directories
                FILE_PATH=$(basename "$FILE_PATH")
                log_debug "FILE_PATH is now: $FILE_PATH"
            fi
        else
            # Not an edit operation - exit silently
            log_debug "Not an edit operation (Event: $EVENT, Tool: $TOOL_NAME), exiting silently"
            [[ "${CLAUDE_HOOKS_DEBUG:-0}" == "1" ]] && exit 2
            exit 0
        fi
    else
        # Invalid JSON input
        log_error "Invalid JSON input provided"
        exit 1
    fi
else
    # No input on stdin
    log_error "No JSON input provided. This hook only works with Claude Code."
    exit 1
fi

# ============================================================================
# PROJECT COMMAND INTEGRATION
# ============================================================================

# Try to use project-specific lint command (make target or script)
try_project_command() {
    local file_path="$1"
    local language="$2"
    
    # Check if project commands are disabled globally
    if [[ "${CLAUDE_HOOKS_USE_PROJECT_COMMANDS:-true}" != "true" ]]; then
        log_debug "Project commands disabled globally"
        return 1
    fi
    
    # Check language-specific opt-out
    local opt_out_var
    opt_out_var="CLAUDE_HOOKS_$(echo "$language" | tr '[:lower:]' '[:upper:]')_USE_PROJECT_COMMANDS"
    if [[ "${!opt_out_var:-true}" != "true" ]]; then
        log_debug "Project commands disabled for $language"
        return 1
    fi
    
    # Get file directory (absolute path)
    local file_dir
    # Since we already cd'd to the file directory, file_path is just the basename
    # So we need to use PWD as the file directory
    file_dir="$PWD"
    
    # Find command root (Makefile or scripts/)
    local cmd_root
    cmd_root=$(find_project_command_root "$file_dir")
    if [[ -z "$cmd_root" ]]; then
        log_debug "No project command root found"
        return 1
    fi
    
    log_debug "Found project command root: $cmd_root"
    
    # Calculate relative path from command root to file
    local rel_path
    # Since we're already in the file's directory, file_path is just the basename
    # We need to calculate the path from command root to the current directory + filename
    if [[ "$cmd_root" == "$PWD" ]]; then
        # Command root is the current directory, just use the filename
        rel_path="$file_path"
    else
        # Calculate relative path from command root to current directory
        local dir_rel_path
        dir_rel_path=$(calculate_relative_path "$cmd_root" "$PWD")
        if [[ "$dir_rel_path" == "." ]]; then
            rel_path="$file_path"
        else
            rel_path="$dir_rel_path/$file_path"
        fi
    fi
    
    log_debug "Relative path from command root: $rel_path"
    
    # Get configured targets/scripts
    local config_output
    if ! config_output=$(get_project_command_config "lint"); then
        log_debug "Failed to get project command config"
        return 1
    fi
    
    local make_targets
    local script_names
    make_targets=$(echo "$config_output" | head -1)
    script_names=$(echo "$config_output" | tail -1)
    
    # Try make targets first
    if [[ -f "$cmd_root/Makefile" ]]; then
        log_debug "Checking make targets: $make_targets"
        for target in $make_targets; do
            if check_make_target "$target" "$cmd_root"; then
                # Run make command with FILE argument
                local make_output
                local make_exit_code
                
                # Change to command root and run make
                if make_output=$(cd "$cmd_root" && make "$target" FILE="$rel_path" 2>&1); then
                    make_exit_code=0
                    log_debug "Make command succeeded"
                else
                    make_exit_code=$?
                    log_debug "Make command failed with exit code: $make_exit_code"
                fi
                
                # Output information if it failed OR if in test mode
                if [[ $make_exit_code -ne 0 ]] || [[ "${CLAUDE_HOOKS_TEST_MODE:-0}" == "1" ]]; then
                    log_info "🔨 Running 'make $target' from $cmd_root"
                    if [[ -n "$make_output" ]]; then
                        echo "$make_output" >&2
                    fi
                fi
                
                # Return make's exit code
                return $make_exit_code
            fi
        done
    fi
    
    # Try scripts if no make target worked
    if [[ -d "$cmd_root/scripts" ]]; then
        log_debug "Checking scripts: $script_names"
        for script in $script_names; do
            if check_script_exists "$script" "$cmd_root/scripts"; then
                # Run script with file argument
                local script_output
                local script_exit_code
                
                # Change to command root and run script
                if script_output=$(cd "$cmd_root" && "./scripts/$script" "$rel_path" 2>&1); then
                    script_exit_code=0
                    log_debug "Script succeeded"
                else
                    script_exit_code=$?
                    log_debug "Script failed with exit code: $script_exit_code"
                fi
                
                # Output information if it failed OR if in test mode
                if [[ $script_exit_code -ne 0 ]] || [[ "${CLAUDE_HOOKS_TEST_MODE:-0}" == "1" ]]; then
                    log_info "📜 Running 'scripts/$script' from $cmd_root"
                    if [[ -n "$script_output" ]]; then
                        echo "$script_output" >&2
                    fi
                fi
                
                # Return script's exit code
                return $script_exit_code
            fi
        done
    fi
    
    log_debug "No project commands found"
    return 1
}

# ============================================================================
# MAIN EXECUTION
# ============================================================================

# This script only works as a Claude Code hook - no CLI mode support

# Print header only in debug mode
[[ "${CLAUDE_HOOKS_DEBUG:-0}" == "1" ]] && {
    echo "" >&2
    echo "🔍 Style Check - Validating code formatting..." >&2
    echo "────────────────────────────────────────────" >&2
}

# Load configuration
load_config

# Start timing
START_TIME=$(time_start)

# Detect project type
PROJECT_TYPE=$(detect_project_type)

# Main execution
main() {
    # Handle mixed project types
    if [[ "$PROJECT_TYPE" == mixed:* ]]; then
        local type_string="${PROJECT_TYPE#mixed:}"
        IFS=',' read -ra TYPE_ARRAY <<< "$type_string"
        
        for type in "${TYPE_ARRAY[@]}"; do
            # Try project command first
            if try_project_command "$FILE_PATH" "$type"; then
                log_debug "Used project command for $type linting"
            else
                # Fall back to language-specific linters
                case "$type" in
                    "go") lint_go ;;
                    "python") lint_python ;;
                    "javascript") lint_javascript ;;
                    "rust") lint_rust ;;
                    "nix") lint_nix ;;
                    "shell") lint_shell ;;
                    "swift") lint_swift ;;
                    "tilt") 
                        if type -t lint_tilt &>/dev/null; then
                            lint_tilt
                        else
                            log_debug "Tilt linting function not available"
                        fi
                        ;;
                esac
            fi
            
            # Fail fast if configured
            if [[ "$CLAUDE_HOOKS_FAIL_FAST" == "true" && $CLAUDE_HOOKS_ERROR_COUNT -gt 0 ]]; then
                break
            fi
        done
    else
        # Single project type
        # Try project command first
        if [[ "$PROJECT_TYPE" != "unknown" ]] && try_project_command "$FILE_PATH" "$PROJECT_TYPE"; then
            log_debug "Used project command for $PROJECT_TYPE linting"
        else
            # Fall back to language-specific linters
            case "$PROJECT_TYPE" in
                "go") lint_go ;;
                "python") lint_python ;;
                "javascript") lint_javascript ;;
                "rust") lint_rust ;;
                "nix") lint_nix ;;
                "shell") lint_shell ;;
                "swift") lint_swift ;;
                "tilt") 
                    if type -t lint_tilt &>/dev/null; then
                        lint_tilt
                    else
                        log_debug "Tilt linting function not available"
                    fi
                    ;;
                "unknown") 
                    log_debug "No recognized project type, skipping checks"
                    ;;
            esac
        fi
    fi
    
    # Show timing if enabled
    time_end "$START_TIME"
    
    # Print summary
    print_error_summary
    
    # Return exit code - any issues mean failure
    if [[ $CLAUDE_HOOKS_ERROR_COUNT -gt 0 ]]; then
        return 2
    else
        return 0
    fi
}

# Run main function
main
exit_code=$?

# Final message and exit
if [[ $exit_code -eq 2 ]]; then
    echo -e "${RED}⛔ BLOCKING: Must fix ALL errors above before continuing${NC}" >&2
    exit 2
else
    # Debug mode handling
    if [[ "${CLAUDE_HOOKS_DEBUG:-0}" == "1" ]]; then
        echo -e "${CYAN}[DEBUG]${NC} Hook completed successfully (debug mode active)" >&2
        exit 2
    fi
    # Success exit
    exit_with_success_message "${YELLOW}👉 Style clean. Continue with your task.${NC}"
fi