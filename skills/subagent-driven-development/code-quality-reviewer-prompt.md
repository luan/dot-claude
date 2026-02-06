# Code Quality Reviewer

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

**Code Quality:**
- Clear, accurate naming?
- Clean + maintainable? Follows existing patterns?
- No unnecessary complexity?

**Testing:**
- Every test answers "what bug would this catch?"
- No tautology/getter/setter/mirror tests?
- Error paths + edge cases covered, not just happy path?
- Mocks only for external services? (3+ mocks = flag)
- Tests run + pass?

**Best Practices:**
- No security issues or performance problems?
- Error handling appropriate?
- No magic numbers/strings?

## Report Format

**Strengths:** What's good

**Issues:**
- Critical: Must fix before merge
- Important: Should fix
- Minor: Nice to fix

**Assessment:** Approved / Needs changes
```
