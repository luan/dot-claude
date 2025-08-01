# /check - Quality Validation & Auto-Fix

Validate code quality and automatically fix all issues using the specialized code-quality-validator agent.

## Usage

`/check`

## Behavior

Uses the `code-quality-validator` agent to:

1. Discover available quality tools in the project
2. Run all validations in parallel (linting, tests, type checking, builds)
3. Automatically fix all issues found
4. Re-run checks until everything passes

The agent doesn't just report issues - it actively fixes them through intelligent analysis and parallel processing when needed.

## Examples

```
/check  # Run all validations and fix issues
```

## Implementation

When this command is invoked, it launches the code-quality-validator agent which will:

- Detect project structure and available tools
- Run comprehensive quality checks
- Fix issues automatically
- Only complete when all checks pass
