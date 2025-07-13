# /check - Quality Validation & Auto-Fix

Validate code quality and automatically fix all issues.

## Usage
`/check`

## Behavior

### Validation Process
Runs comprehensive quality checks:
- 🔧 Linting (ESLint, Prettier, etc.)
- 🏗️ Type checking (TypeScript, Flow, etc.)
- 🧪 Test suites
- 📦 Build verification
- 🔒 Security scanning

### Auto-Fix Philosophy
**IMPORTANT**: This command doesn't just report issues - it fixes them:
1. Identifies all quality issues
2. 👥 Spawns agents for parallel fixing if needed
3. Re-runs checks until everything passes
4. Won't stop until all checks are green

### Memory Integration
🧠 The model may track validation patterns in `.ai.local/`:
- Common issues and solutions
- Project-specific validation rules
- Performance benchmarks
- Quality trends

### Smart Detection
Automatically detects available tools:
- Checks package.json for scripts
- Identifies linting configurations
- Finds test frameworks
- Adapts to project setup

## Examples
```
/check  # Run all validations and fix issues
```

## Exit Criteria
Only completes when:
- ✅ All linters pass
- ✅ All tests pass
- ✅ Type checking succeeds
- ✅ Build completes successfully
- ✅ No security issues