#!/usr/bin/env bash
# smart-lint.sh - Intelligent project-aware code quality checks for Claude Code

# Check for direct mode configuration to bypass complex project detection
# This allows large projects to opt into lightweight, direct tool execution
if [[ -f ".claude-hooks-config.sh" ]]; then
	# shellcheck disable=SC1091
	source ".claude-hooks-config.sh"

	if [[ "${CLAUDE_HOOKS_DIRECT_MODE:-false}" == "true" ]]; then
		echo "$(date): Using direct mode per project configuration" >>/tmp/claude-hook-output.log

		# Parse JSON input to get the file path
		if [ ! -t 0 ]; then
			JSON_INPUT=$(cat)
			if command -v jq >/dev/null 2>&1 && echo "$JSON_INPUT" | jq . >/dev/null 2>&1; then
				EVENT=$(echo "$JSON_INPUT" | jq -r '.hook_event_name // empty')
				TOOL_NAME=$(echo "$JSON_INPUT" | jq -r '.tool_name // empty')
				if [[ "$EVENT" == "PostToolUse" ]] && [[ "$TOOL_NAME" =~ ^(Edit|Write|MultiEdit)$ ]]; then
					FILE_PATH=$(echo "$JSON_INPUT" | jq -r '.tool_input.file_path // empty')
					echo "$(date): Processing file in direct mode: $FILE_PATH" >>/tmp/claude-hook-output.log

					# Handle different file types based on configuration
					file_handled=false

					# Swift files
					if [[ "$FILE_PATH" == *.swift ]] && [[ -f "$FILE_PATH" ]] && [[ "${CLAUDE_HOOKS_SWIFT_DIRECT_ENABLED:-true}" == "true" ]]; then
						echo "🔍 Running Swift linting on: $(basename "$FILE_PATH")" >&2
						file_handled=true

						# Use configured SwiftFormat command or default
						SWIFTFORMAT_CMD="${CLAUDE_HOOKS_SWIFT_FORMAT_CMD:-swiftformat}"
						if command -v "$SWIFTFORMAT_CMD" >/dev/null 2>&1; then
							echo "Running SwiftFormat..." >&2
							if ! $SWIFTFORMAT_CMD "$FILE_PATH" 2>&1; then
								echo "❌ SwiftFormat found issues in $FILE_PATH" >&2
								echo "$(date): SwiftFormat failed for $FILE_PATH" >>/tmp/claude-hook-output.log
								exit 2
							fi
						fi

						# Use configured SwiftLint command or default
						SWIFTLINT_CMD="${CLAUDE_HOOKS_SWIFT_LINT_CMD:-swiftlint}"
						if command -v "$SWIFTLINT_CMD" >/dev/null 2>&1; then
							echo "Running SwiftLint..." >&2
							SWIFTLINT_ARGS="${CLAUDE_HOOKS_SWIFT_LINT_ARGS:-lint --quiet}"
							# shellcheck disable=SC2086
							if ! $SWIFTLINT_CMD $SWIFTLINT_ARGS "$FILE_PATH" 2>&1; then
								echo "❌ SwiftLint found issues in $FILE_PATH" >&2
								echo "$(date): SwiftLint failed for $FILE_PATH" >>/tmp/claude-hook-output.log
								exit 2
							fi
						fi

						echo "✅ Swift linting completed successfully" >&2
					fi

					# Add more file type handlers here as needed
					# Python files
					if [[ "$FILE_PATH" == *.py ]] && [[ -f "$FILE_PATH" ]] && [[ "${CLAUDE_HOOKS_PYTHON_DIRECT_ENABLED:-false}" == "true" ]]; then
						echo "🔍 Running Python linting on: $(basename "$FILE_PATH")" >&2
						file_handled=true

						PYTHON_FORMAT_CMD="${CLAUDE_HOOKS_PYTHON_FORMAT_CMD:-black}"
						PYTHON_LINT_CMD="${CLAUDE_HOOKS_PYTHON_LINT_CMD:-ruff}"

						if command -v "$PYTHON_FORMAT_CMD" >/dev/null 2>&1; then
							echo "Running $PYTHON_FORMAT_CMD..." >&2
							$PYTHON_FORMAT_CMD "$FILE_PATH" 2>&1
						fi

						if command -v "$PYTHON_LINT_CMD" >/dev/null 2>&1; then
							echo "Running $PYTHON_LINT_CMD..." >&2
							if ! $PYTHON_LINT_CMD check "$FILE_PATH" 2>&1; then
								echo "❌ $PYTHON_LINT_CMD found issues in $FILE_PATH" >&2
								exit 2
							fi
						fi

						echo "✅ Python linting completed successfully" >&2
					fi

					if [[ "$file_handled" == "false" ]]; then
						echo "👉 Skipping file type $(basename "$FILE_PATH") in direct mode." >&2
					fi
				fi
			fi
		fi

		echo "$(date): Direct mode completed" >>/tmp/claude-hook-output.log
		exit 0
	fi
fi
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
echo "$(date): About to source common-helpers.sh" >>/tmp/claude-hook-output.log
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/common-helpers.sh"
echo "$(date): Sourced common-helpers.sh successfully" >>/tmp/claude-hook-output.log

# Debug output after sourcing helpers so we have log_debug
log_debug "smart-lint.sh started"

# Also write to a known file for visibility when output is lost
echo "$(date): Hook started in $(pwd)" >>/tmp/claude-hook-output.log

# Add completion tracking
trap 'echo "$(date): Hook completed with exit code $? in $(pwd)" >> /tmp/claude-hook-output.log' EXIT

# ============================================================================
# PROJECT DETECTION
# ============================================================================

# Enhanced project type detection with Tilt support
detect_project_type() {
	local types=()
	local project_type="unknown"

	log_debug "Starting project type detection"

	# Check for Go projects
	log_debug "Checking for Go projects"
	if [[ -f "go.mod" ]] || [[ -f "go.sum" ]] || [[ -n "$(timeout 2 find . -maxdepth 3 -name "*.go" -type f -print -quit 2>/dev/null)" ]]; then
		types+=("go")
		log_debug "Found Go project"
	fi

	# Check for Python projects
	log_debug "Checking for Python projects"
	if [[ -f "pyproject.toml" ]] || [[ -f "setup.py" ]] || [[ -f "requirements.txt" ]] || [[ -n "$(timeout 2 find . -maxdepth 3 -name "*.py" -type f -print -quit 2>/dev/null)" ]]; then
		types+=("python")
		log_debug "Found Python project"
	fi

	# Check for JavaScript/TypeScript projects
	log_debug "Checking for JavaScript/TypeScript projects"
	if [[ -f "package.json" ]] || [[ -f "tsconfig.json" ]] || [[ -n "$(timeout 2 find . -maxdepth 3 \( -name "*.js" -o -name "*.ts" -o -name "*.jsx" -o -name "*.tsx" \) -type f -print -quit 2>/dev/null)" ]]; then
		types+=("javascript")
		log_debug "Found JavaScript/TypeScript project"
	fi

	# Check for Rust projects
	log_debug "Checking for Rust projects"
	if [[ -f "Cargo.toml" ]] || [[ -n "$(timeout 2 find . -maxdepth 3 -name "*.rs" -type f -print -quit 2>/dev/null)" ]]; then
		types+=("rust")
		log_debug "Found Rust project"
	fi

	# Check for Nix projects
	log_debug "Checking for Nix projects"
	if [[ -f "flake.nix" ]] || [[ -f "default.nix" ]] || [[ -f "shell.nix" ]]; then
		types+=("nix")
		log_debug "Found Nix project"
	fi

	# Check for shell projects
	log_debug "Checking for shell projects"
	if [[ -n "$(timeout 2 find . -maxdepth 3 -name "*.sh" -type f -print -quit 2>/dev/null)" ]] || [[ -n "$(timeout 2 find . -maxdepth 3 -name "*.bash" -type f -print -quit 2>/dev/null)" ]]; then
		types+=("shell")
		log_debug "Found shell project"
	fi

	# Check for Tilt projects
	log_debug "Checking for Tilt projects"
	if [[ -f "Tiltfile" ]] || [[ -n "$(timeout 2 find . -maxdepth 3 -name "Tiltfile" -type f -print -quit 2>/dev/null)" ]] || [[ -n "$(timeout 2 find . -maxdepth 3 -name "*.tiltfile" -type f -print -quit 2>/dev/null)" ]]; then
		types+=("tilt")
		log_debug "Found Tilt project"
	fi

	# Check for Swift projects - be very careful with large codebases
	log_debug "Checking for Swift projects"
	if [[ -f "Package.swift" ]] || [[ -n "$(timeout 1 find . -maxdepth 2 -name "*.xcodeproj" -type d -print -quit 2>/dev/null)" ]] || [[ -n "$(timeout 1 find . -maxdepth 2 -name "*.xcworkspace" -type d -print -quit 2>/dev/null)" ]] || [[ -n "$(timeout 1 find . -maxdepth 2 -name "*.swift" -type f -print -quit 2>/dev/null)" ]]; then
		types+=("swift")
		log_debug "Found Swift project"
	fi

	# Return result
	if [[ ${#types[@]} -eq 1 ]]; then
		project_type="${types[0]}"
	elif [[ ${#types[@]} -gt 1 ]]; then
		project_type="mixed:$(
			IFS=,
			echo "${types[*]}"
		)"
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

# Format files with error handling and timeout protection
format_files() {
	local formatter="$1"
	local files="$2"
	shift 2

	if command_exists "$formatter"; then
		log_debug "format_files: formatter=$formatter, files='$files', pwd=$(pwd)"

		# For single file, don't use xargs
		if [[ "$files" != *" "* ]] && [[ -f "$files" ]]; then
			log_debug "Single file mode: $files"
			if ! timeout 30 "$formatter" --check "$files" "$@" >/dev/null 2>&1; then
				local format_output
				if ! format_output=$(timeout 60 "$formatter" "$files" "$@" 2>&1); then
					local exit_code=$?
					if [[ $exit_code -eq 124 ]]; then
						add_error "$formatter timed out"
					else
						add_error "$formatter formatting failed"
					fi
					echo "$format_output" >&2
				fi
			fi
		else
			log_debug "Multi-file mode using xargs"
			if ! timeout 30 bash -c "echo '$files' | xargs '$formatter' --check \"\$@\"" -- "$@" >/dev/null 2>&1; then
				local format_output
				if ! format_output=$(timeout 60 bash -c "echo '$files' | xargs '$formatter' \"\$@\"" -- "$@" 2>&1); then
					local exit_code=$?
					if [[ $exit_code -eq 124 ]]; then
						add_error "$formatter timed out"
					else
						add_error "$formatter formatting failed"
					fi
					echo "$format_output" >&2
				fi
			fi
		fi
	fi
}

# Lint files with error handling and timeout protection
lint_files() {
	local linter="$1"
	local files="$2"
	shift 2

	if command_exists "$linter"; then
		log_debug "Running $linter on files: $files"
		local linter_output
		# Add 60 second timeout to prevent hanging
		if ! linter_output=$(timeout 60 bash -c "echo '$files' | xargs '$linter' \"\$@\"" -- "$@" 2>&1); then
			local exit_code=$?
			if [[ $exit_code -eq 124 ]]; then
				add_error "$linter timed out after 60 seconds"
			else
				add_error "$linter found issues"
			fi
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
	shell_files=$(find . -type f \( -name "*.sh" -o -name "*.bash" -o -name "*.zsh" \) \( ! -path "./.git/*" ! -path "./node_modules/*" ! -path "./venv/*" ! -path "./.venv/*" \) | head -50)

	local shebang_files
	# Use find with exec instead of recursive grep to avoid performance issues
	shebang_files=$(find . -type f \( ! -path "./.git/*" ! -path "./node_modules/*" ! -path "./venv/*" ! -path "./.venv/*" ! -path "./build/*" ! -path "./dist/*" \) -exec grep -l "^#!.*\(bash\|sh\|zsh\)" {} \; 2>/dev/null | head -50)

	# Combine and deduplicate
	echo -e "$shell_files\n$shebang_files" | sort -u | grep -v "^$"
}

lint_swift() {
	if [[ "${CLAUDE_HOOKS_SWIFT_ENABLED:-true}" != "true" ]]; then
		log_debug "Swift linting disabled"
		return 0
	fi

	log_debug "Running Swift linters..."

	# Quick check for large Swift codebases - skip if too many files
	local swift_count
	swift_count=$(timeout 5 find . -name "*.swift" -type f | wc -l 2>/dev/null || echo "0")
	if [[ $swift_count -gt 10000 ]]; then
		log_debug "Large Swift codebase detected ($swift_count files), skipping comprehensive linting"
		# Only process specific files, skip project-wide linting
		if [[ -n "$FILE_PATH" && "$FILE_PATH" == *.swift ]] && [[ -f "$FILE_PATH" ]] && ! should_skip_file "$FILE_PATH"; then
			log_debug "Processing single Swift file: $FILE_PATH"
			# Only run SwiftFormat on the specific file, skip SwiftLint
			if command_exists swiftformat; then
				format_files "swiftformat" "$FILE_PATH"
			fi
		else
			log_debug "No specific Swift file to process, skipping all Swift linting for large codebase"
		fi
		return 0
	fi

	local swift_files
	# If we have a specific file path and it's a Swift file, just use that
	if [[ -n "$FILE_PATH" && "$FILE_PATH" == *.swift ]]; then
		if [[ -f "$FILE_PATH" ]] && ! should_skip_file "$FILE_PATH"; then
			swift_files="$FILE_PATH"
			log_debug "Processing single Swift file: $FILE_PATH"
		else
			log_debug "Swift file $FILE_PATH was skipped or doesn't exist"
			return 0
		fi
	else
		# For medium-sized codebases, only run SwiftLint on the whole project without individual file processing
		# This avoids the expensive file discovery process
		log_debug "No specific Swift file provided, using SwiftLint on entire project"
		swift_files=""
	fi

	# SwiftFormat - only run on specific files, not entire project
	if [[ -n "$swift_files" ]] && command_exists swiftformat; then
		log_debug "About to format Swift files: '$swift_files'"
		log_debug "Current directory: $(pwd)"
		format_files "swiftformat" "$swift_files"
	else
		log_debug "Skipping SwiftFormat for entire project (only runs on specific files)"
	fi

	# SwiftLint - linting (can run on entire project efficiently)
	# Prefer project-specific SwiftLint if available
	local swiftlint_cmd="swiftlint"
	if [[ -f "Tools/bin/swiftlint" ]]; then
		swiftlint_cmd="Tools/bin/swiftlint"
		log_debug "Using project-specific SwiftLint: $swiftlint_cmd"
	elif command_exists swiftlint; then
		log_debug "Using system SwiftLint: $(which swiftlint)"
	else
		log_debug "SwiftLint not found"
		return 0
	fi

	# SwiftLint with timeout protection
	local swiftlint_output
	log_debug "Running SwiftLint on project"
	if ! swiftlint_output=$(timeout 120 $swiftlint_cmd --quiet 2>&1); then
		local exit_code=$?
		if [[ $exit_code -eq 124 ]]; then
			add_error "SwiftLint timed out after 120 seconds"
		else
			add_error "SwiftLint found issues"
		fi
		echo "$swiftlint_output" >&2
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

			# Keep the full path for FILE_PATH
			if [[ -n "$FILE_PATH" ]] && [[ -f "$FILE_PATH" ]]; then
				log_debug "FILE_PATH is: $FILE_PATH"
				log_debug "Current working directory: $(pwd)"
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
	log_debug "Starting main execution with PROJECT_TYPE: $PROJECT_TYPE, FILE_PATH: $FILE_PATH"

	# Handle mixed project types
	if [[ "$PROJECT_TYPE" == mixed:* ]]; then
		local type_string="${PROJECT_TYPE#mixed:}"
		IFS=',' read -ra TYPE_ARRAY <<<"$type_string"
		log_debug "Processing mixed project types: ${TYPE_ARRAY[*]}"

		for type in "${TYPE_ARRAY[@]}"; do
			log_debug "Processing type: $type"
			# Try project command first
			if try_project_command "$FILE_PATH" "$type"; then
				log_debug "Used project command for $type linting"
			else
				log_debug "Falling back to language-specific linter for $type"
				# Fall back to language-specific linters
				case "$type" in
				"go")
					log_debug "Running lint_go"
					lint_go
					;;
				"python")
					log_debug "Running lint_python"
					lint_python
					;;
				"javascript")
					log_debug "Running lint_javascript"
					lint_javascript
					;;
				"rust")
					log_debug "Running lint_rust"
					lint_rust
					;;
				"nix")
					log_debug "Running lint_nix"
					lint_nix
					;;
				"shell")
					log_debug "Running lint_shell"
					lint_shell
					;;
				"swift")
					log_debug "Running lint_swift"
					lint_swift
					;;
				"tilt")
					if type -t lint_tilt &>/dev/null; then
						log_debug "Running lint_tilt"
						lint_tilt
					else
						log_debug "Tilt linting function not available"
					fi
					;;
				esac
			fi
			log_debug "Completed processing type: $type"

			# Fail fast if configured
			if [[ "$CLAUDE_HOOKS_FAIL_FAST" == "true" && $CLAUDE_HOOKS_ERROR_COUNT -gt 0 ]]; then
				log_debug "Fail fast triggered, breaking"
				break
			fi
		done
	else
		log_debug "Processing single project type: $PROJECT_TYPE"
		# Single project type
		# Try project command first
		if [[ "$PROJECT_TYPE" != "unknown" ]] && try_project_command "$FILE_PATH" "$PROJECT_TYPE"; then
			log_debug "Used project command for $PROJECT_TYPE linting"
		else
			log_debug "Falling back to language-specific linter for $PROJECT_TYPE"
			# Fall back to language-specific linters
			case "$PROJECT_TYPE" in
			"go")
				log_debug "Running lint_go"
				lint_go
				;;
			"python")
				log_debug "Running lint_python"
				lint_python
				;;
			"javascript")
				log_debug "Running lint_javascript"
				lint_javascript
				;;
			"rust")
				log_debug "Running lint_rust"
				lint_rust
				;;
			"nix")
				log_debug "Running lint_nix"
				lint_nix
				;;
			"shell")
				log_debug "Running lint_shell"
				lint_shell
				;;
			"swift")
				log_debug "Running lint_swift"
				lint_swift
				;;
			"tilt")
				if type -t lint_tilt &>/dev/null; then
					log_debug "Running lint_tilt"
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

	log_debug "Main linting completed, showing timing"
	# Show timing if enabled
	time_end "$START_TIME"

	log_debug "Printing error summary"
	# Print summary
	print_error_summary

	log_debug "Main execution completed with $CLAUDE_HOOKS_ERROR_COUNT errors"
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
