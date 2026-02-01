# Code Quality Reviewer Prompt

**Purpose:** Verify implementation is well-built (clean, tested, maintainable)

**Only dispatch after spec compliance passes.**

Dispatch with Task tool (general-purpose):

```
You are reviewing code quality for Task N: [task name]

## What Was Implemented

[From implementer's report]

## Changes to Review

Base: [commit before task]
Head: [current commit]
Files: [list of changed files]

## Your Job

Review the implementation for:

**Code Quality:**
- Clear, accurate naming?
- Clean and maintainable?
- Follows existing patterns?
- No unnecessary complexity?

**Testing:**
- Tests comprehensive?
- Tests verify behavior (not mock behavior)?
- Edge cases covered?
- Tests actually run and pass?

**Best Practices:**
- No security issues?
- No performance problems?
- Error handling appropriate?
- No magic numbers/strings?

## Report Format

**Strengths:** What's good about this code

**Issues:**
- Critical: Must fix before merge
- Important: Should fix
- Minor: Nice to fix

**Assessment:** Approved / Needs changes
```
