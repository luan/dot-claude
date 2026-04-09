#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# dependencies = []
# ///

"""
Auto-formatting post tool use hook for Claude Code.
Automatically formats and lints files after they're edited using appropriate tools.
"""

import json
import subprocess
import sys
import shutil
from pathlib import Path
from typing import List, Tuple


def print_success(message: str) -> None:
    """Print success message with formatting."""
    print("", file=sys.stderr)
    print(f"\033[34m🔹 {message}\033[0m", file=sys.stderr)


def print_error(message: str) -> None:
    """Print error message with formatting."""
    print("", file=sys.stderr)
    print(f"\033[31m⛔ {message}\033[0m", file=sys.stderr)


def print_warning(message: str) -> None:
    """Print warning message with formatting."""
    print("", file=sys.stderr)
    print(f"\033[33m⚠️  {message}\033[0m", file=sys.stderr)


def print_info(message: str) -> None:
    """Print info message with formatting."""
    print("", file=sys.stderr)
    print(f"\033[36mℹ️  {message}\033[0m", file=sys.stderr)


def command_exists(command: str) -> bool:
    """Check if a command exists in PATH."""
    return shutil.which(command) is not None


def should_skip_file(file_path: Path) -> bool:
    """Check if file should be skipped based on path patterns."""
    skip_patterns = {
        "node_modules",
        ".git",
        "__pycache__",
        ".venv",
        "venv",
        "build",
        "dist",
        ".build",
        "target",
        "vendor",
        ".next",
        ".cache",
        ".tmp",
        "tmp",
        "temp",
    }

    # Check if any part of the path contains skip patterns
    for part in file_path.parts:
        if part in skip_patterns:
            return True

    return False


def find_config_file(file_path: Path, config_name: str) -> Path | None:
    """Find a config file by searching from the file's directory up to the repository root."""
    current_dir = file_path.parent if file_path.is_file() else file_path

    # First try to find git repository root
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            cwd=current_dir,
            capture_output=True,
            text=True,
            timeout=10,
        )
        if result.returncode == 0:
            repo_root = Path(result.stdout.strip())
            # Check if config exists in repo root
            config_path = repo_root / config_name
            if config_path.exists():
                return config_path.resolve()
            # If not in root, search from current dir up to repo root
            search_dir = current_dir
            while search_dir != repo_root and search_dir != search_dir.parent:
                config_path = search_dir / config_name
                if config_path.exists():
                    return config_path.resolve()
                search_dir = search_dir.parent
            return None
    except (subprocess.TimeoutExpired, Exception):
        pass

    # Fallback: search up directory tree without git
    while current_dir != current_dir.parent:
        config_path = current_dir / config_name
        if config_path.exists():
            return config_path.resolve()
        current_dir = current_dir.parent

    return None


def run_command(
    command: List[str], file_path: Path, timeout: int = 60
) -> Tuple[bool, str]:
    """Run a command with timeout and return success status and output."""
    try:
        result = subprocess.run(
            command,
            cwd=file_path.parent,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
        return result.returncode == 0, result.stdout + result.stderr
    except subprocess.TimeoutExpired:
        return False, f"Command timed out after {timeout} seconds"
    except Exception as e:
        return False, f"Command failed: {e}"


def format_go_file(file_path: Path) -> bool:
    """Format and lint Go file using golangci-lint."""
    has_errors = False

    if command_exists("golangci-lint"):
        print_info(f"Running golangci-lint on {file_path.name}")
        success, output = run_command(
            ["golangci-lint", "run", "--fix", str(file_path)], file_path
        )
        if not success:
            print_error(f"golangci-lint found issues in {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True
        else:
            print_success("golangci-lint completed successfully")
    else:
        print_warning("golangci-lint not found, skipping Go linting")

    return not has_errors


def format_swift_file(file_path: Path) -> bool:
    """Format and lint Swift file."""
    has_errors = False

    # SwiftFormat
    if command_exists("swiftformat"):
        print_info(f"Running SwiftFormat on {file_path.name}")
        success, output = run_command(["swiftformat", str(file_path)], file_path)
        if not success:
            print_error(f"SwiftFormat failed on {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True
        else:
            print_success("SwiftFormat completed successfully")

    # SwiftLint with fix
    if command_exists("swiftlint"):
        # Find SwiftLint config file
        swiftlint_config = find_config_file(file_path, ".swiftlint.yml")

        print_info(f"Running SwiftLint on {file_path.name}")

        # Build SwiftLint command with optional config
        fix_command = ["swiftlint", "lint", "--fix", "--quiet"]
        if swiftlint_config:
            fix_command.extend(["--config", str(swiftlint_config)])
        fix_command.append(str(file_path))

        success, output = run_command(fix_command, file_path)
        if not success:
            print_error(f"SwiftLint (fix) could not fix issues in {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True

        # Build SwiftLint lint command with optional config
        lint_command = ["swiftlint", "lint", "--strict", "--quiet"]
        if swiftlint_config:
            lint_command.extend(["--config", str(swiftlint_config)])
        lint_command.append(str(file_path))

        success, output = run_command(lint_command, file_path)
        if not success:
            print_error(f"SwiftLint found issues in {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True
        else:
            print_success("SwiftLint completed successfully")

    return not has_errors


def format_rust_file(file_path: Path) -> bool:
    """Format and lint Rust file using rustfmt and cargo clippy."""
    has_errors = False

    # Format with rustfmt
    if command_exists("rustfmt"):
        print_info(f"Running rustfmt on {file_path.name}")
        success, output = run_command(["rustfmt", str(file_path)], file_path)
        if not success:
            print_error(f"rustfmt failed on {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True
        else:
            print_success("rustfmt completed successfully")
    else:
        print_warning("rustfmt not found, skipping Rust formatting")

    # Lint with cargo clippy (strict mode - no warnings allowed)
    if command_exists("cargo"):
        print_info(f"Running cargo clippy on {file_path.name}")
        success, output = run_command(
            ["cargo", "clippy", "--", "-D", "warnings"],
            file_path,
        )
        if not success:
            print_error(f"cargo clippy found issues in {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True
        else:
            print_success("cargo clippy completed successfully")
    else:
        print_warning("cargo not found, skipping Rust linting")

    return not has_errors


def format_python_file(file_path: Path) -> bool:
    """Format and lint Python file."""
    has_errors = False

    # Black formatting
    if command_exists("black"):
        print_info(f"Running black on {file_path.name}")
        success, output = run_command(["black", str(file_path)], file_path)
        if not success:
            print_error(f"black failed on {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True

    # Ruff formatting and linting
    if command_exists("ruff"):
        # Format with ruff
        print_info(f"Running ruff format on {file_path.name}")
        success, output = run_command(["ruff", "format", str(file_path)], file_path)
        if not success:
            print_error(f"ruff format failed on {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True

        # Lint with ruff
        print_info(f"Running ruff check on {file_path.name}")
        success, output = run_command(
            ["ruff", "check", "--fix", str(file_path)], file_path
        )
        if not success:
            print_error(f"ruff check found issues in {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True

        if not has_errors:
            print_success("ruff completed successfully")

    return not has_errors


def format_svelte_file(file_path: Path) -> bool:
    """Format Svelte file with prettier and run project-level svelte-check."""
    has_errors = False

    # Format with prettier
    if command_exists("prettier"):
        print_info(f"Running prettier on {file_path.name}")
        success, output = run_command(
            ["prettier", "--write", str(file_path)], file_path
        )
        if not success:
            print_error(f"prettier failed on {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True
        else:
            print_success("prettier completed successfully")

    # Project-level type check with svelte-check
    svelte_config = find_config_file(file_path, "svelte.config.js")
    if not svelte_config:
        svelte_config = find_config_file(file_path, "svelte.config.ts")
    if svelte_config:
        project_root = svelte_config.parent
        print_info(f"Running svelte-check from {project_root}")
        try:
            result = subprocess.run(
                ["npx", "svelte-check", "--threshold", "error",
                 "--output", "human"],
                cwd=project_root,
                capture_output=True,
                text=True,
                timeout=60,
            )
            if result.returncode != 0:
                combined = (result.stdout + result.stderr).strip()
                if combined:
                    lines = combined.split("\n")
                    print_error("svelte-check found errors:")
                    print("\n".join(lines[:30]), file=sys.stderr)
                has_errors = True
            else:
                print_success("svelte-check passed")
        except subprocess.TimeoutExpired:
            print_warning("svelte-check timed out (60s), skipping")
        except Exception as e:
            print_warning(f"svelte-check failed to run: {e}")

    return not has_errors


def format_javascript_typescript_file(file_path: Path) -> bool:
    """Format JavaScript/TypeScript file."""
    has_errors = False

    # Prettier formatting
    if command_exists("prettier"):
        print_info(f"Running prettier on {file_path.name}")
        success, output = run_command(
            ["prettier", "--write", str(file_path)], file_path
        )
        if not success:
            print_error(f"prettier failed on {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True
        else:
            print_success("prettier completed successfully")

    # ESLint with fix (if available)
    if command_exists("eslint"):
        print_info(f"Running eslint on {file_path.name}")
        success, output = run_command(["eslint", "--fix", str(file_path)], file_path)
        if not success:
            print_warning(f"eslint found issues in {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)

    return not has_errors


def format_c_cpp_file(file_path: Path) -> bool:
    """Format C/C++/Objective-C file using clang-format."""
    has_errors = False

    if command_exists("clang-format"):
        print_info(f"Running clang-format on {file_path.name}")
        success, output = run_command(["clang-format", "-i", str(file_path)], file_path)
        if not success:
            print_error(f"clang-format failed on {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True
        else:
            print_success("clang-format completed successfully")
    else:
        print_warning("clang-format not found, skipping C/C++ formatting")

    return not has_errors


def format_lua_file(file_path: Path) -> bool:
    """Format Lua file using stylua."""
    has_errors = False

    if command_exists("stylua"):
        print_info(f"Running stylua on {file_path.name}")
        success, output = run_command(["stylua", str(file_path)], file_path)
        if not success:
            print_error(f"stylua failed on {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True
        else:
            print_success("stylua completed successfully")
    else:
        print_warning("stylua not found, skipping Lua formatting")

    return not has_errors


def format_shell_file(file_path: Path) -> bool:
    """Format and lint shell file."""
    has_errors = False

    # Skip Fish shell files - they have different syntax than bash/sh
    if file_path.suffix.lower() == ".fish":
        print_info(f"Skipping shellcheck for Fish shell file {file_path.name}")
        return True

    # Format with shfmt
    if command_exists("shfmt"):
        print_info(f"Running shfmt on {file_path.name}")
        success, output = run_command(["shfmt", "-w", str(file_path)], file_path)
        if not success:
            print_error(f"shfmt failed on {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True

    # Lint with shellcheck (no autofix available) - skip for Fish shell
    if command_exists("shellcheck") and file_path.suffix.lower() != ".fish":
        print_info(f"Running shellcheck on {file_path.name}")
        success, output = run_command(["shellcheck", str(file_path)], file_path)
        if not success:
            print_error(f"shellcheck found issues in {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True

    if not has_errors:
        print_success("Shell formatting/linting completed successfully")

    return not has_errors


def format_yaml_file(file_path: Path) -> bool:
    """Format YAML file using prettier if available."""
    has_errors = False

    if command_exists("prettier"):
        print_info(f"Running prettier on {file_path.name}")
        success, output = run_command(
            ["prettier", "--write", str(file_path)], file_path
        )
        if not success:
            print_error(f"prettier failed on {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True
        else:
            print_success("prettier completed successfully")
    else:
        print_warning("prettier not found, skipping YAML formatting")

    return not has_errors


def format_terraform_file(file_path: Path) -> bool:
    """Format and lint Terraform file."""
    has_errors = False

    # Format with terraform fmt
    if command_exists("terraform"):
        print_info(f"Running terraform fmt on {file_path.name}")
        success, output = run_command(["terraform", "fmt", str(file_path)], file_path)
        if not success:
            print_error(f"terraform fmt failed on {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True

    # Lint with tflint
    if command_exists("tflint"):
        print_info(f"Running tflint on {file_path.name}")
        success, output = run_command(["tflint", "--fix", str(file_path)], file_path)
        if not success:
            print_warning(f"tflint found issues in {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)

    if not has_errors:
        print_success("Terraform formatting completed successfully")

    return not has_errors


def format_cmake_file(file_path: Path) -> bool:
    """Format CMake file using cmake-format."""
    has_errors = False

    if command_exists("cmake-format"):
        print_info(f"Running cmake-format on {file_path.name}")
        success, output = run_command(["cmake-format", "-i", str(file_path)], file_path)
        if not success:
            print_error(f"cmake-format failed on {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True
        else:
            print_success("cmake-format completed successfully")
    else:
        print_warning("cmake-format not found, skipping CMake formatting")

    return not has_errors


def format_dockerfile(file_path: Path) -> bool:
    """Lint Dockerfile using hadolint."""
    has_errors = False

    if command_exists("hadolint"):
        print_info(f"Running hadolint on {file_path.name}")
        success, output = run_command(["hadolint", str(file_path)], file_path)
        if not success:
            print_error(f"hadolint found issues in {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            has_errors = True
        else:
            print_success("hadolint completed successfully")
    else:
        print_warning("hadolint not found, skipping Dockerfile linting")

    return not has_errors


def format_file(file_path: Path) -> bool:
    """Format file based on its extension."""
    if should_skip_file(file_path):
        print_info(f"Skipping {file_path.name} (in ignored directory)")
        return True

    if not file_path.exists():
        print_warning(f"File {file_path} does not exist")
        return True

    suffix = file_path.suffix.lower()
    stem = file_path.stem.lower()

    # Go files
    if suffix == ".go":
        return format_go_file(file_path)

    # Swift files
    elif suffix == ".swift":
        return format_swift_file(file_path)

    # Rust files
    elif suffix == ".rs":
        return format_rust_file(file_path)

    # Python files
    elif suffix == ".py":
        return format_python_file(file_path)

    # Svelte files
    elif suffix == ".svelte":
        return format_svelte_file(file_path)

    # JavaScript/TypeScript files
    elif suffix in {".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs"}:
        return format_javascript_typescript_file(file_path)

    # C/C++/Objective-C files
    elif suffix in {
        ".c",
        ".cpp",
        ".cc",
        ".cxx",
        ".c++",
        ".h",
        ".hpp",
        ".hh",
        ".hxx",
        ".h++",
        ".m",
        ".mm",
    }:
        return format_c_cpp_file(file_path)

    # Lua files
    elif suffix == ".lua":
        return format_lua_file(file_path)

    # Shell files
    elif suffix in {".sh", ".bash", ".zsh", ".fish"}:
        return format_shell_file(file_path)

    # YAML files
    elif suffix in {".yml", ".yaml"}:
        return format_yaml_file(file_path)

    # Terraform files
    elif suffix == ".tf" or suffix == ".tfvars":
        return format_terraform_file(file_path)

    # CMake files
    elif suffix == ".cmake" or stem == "cmakelists":
        return format_cmake_file(file_path)

    # Dockerfile
    elif stem == "dockerfile" or file_path.name.lower().startswith("dockerfile"):
        return format_dockerfile(file_path)

    # JSON files (prettier)
    elif suffix == ".json" and command_exists("prettier"):
        print_info(f"Running prettier on {file_path.name}")
        success, output = run_command(
            ["prettier", "--write", str(file_path)], file_path
        )
        if not success:
            print_error(f"prettier failed on {file_path.name}")
            if output.strip():
                print(output, file=sys.stderr)
            return False
        print_success("prettier completed successfully")
        return True

    else:
        print_info(f"No formatter configured for {file_path.name} ({suffix})")
        return True


def main() -> None:
    try:
        # Read JSON input from stdin
        if sys.stdin.isatty():
            print_error("This hook requires JSON input from Claude Code")
            sys.exit(1)

        input_data = json.load(sys.stdin)

        # Extract event and tool info
        event = input_data.get("hook_event_name", "")
        tool_name = input_data.get("tool_name", "")

        # Only process file edits
        if event != "PostToolUse" or tool_name not in ["Edit", "Write", "MultiEdit"]:
            sys.exit(0)

        # Get the file path that was edited
        tool_input = input_data.get("tool_input", {})
        file_path_str = tool_input.get("file_path", "")

        if not file_path_str:
            sys.exit(0)

        file_path = Path(file_path_str)

        # Format the file
        success = format_file(file_path)

        if success:
            print_success("Formatting completed successfully")
            sys.exit(0)
        else:
            print_error(
                "BLOCKING: Fix formatting/linting issues above before continuing"
            )
            sys.exit(2)

    except json.JSONDecodeError:
        print_error("Invalid JSON input")
        sys.exit(1)
    except Exception as e:
        print_error(f"Unexpected error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
