---
name: code-review
description: Request reviews, receive feedback properly, verify before claiming completion. No performative agreement.
---

# Code Review

Three capabilities: requesting, receiving, verifying.

## Requesting Review

**Mandatory triggers:**
- After task completion, before merge
- Significant architecture changes
- Security-sensitive code

**Optional triggers:**
- When stuck on design decision
- Uncertain about approach
- Want second opinion

**Dispatch pattern:**
```
Task subagent:
- subagent_type: "code-reviewer"
- prompt: include changed files + context + specific concerns
```

## Receiving Review

**Process:** READ → UNDERSTAND → VERIFY → EVALUATE → RESPOND → IMPLEMENT

1. **READ** - full feedback, no skimming
2. **UNDERSTAND** - what's actually being said (not assumed)
3. **VERIFY** - check claims against code, run suggested tests
4. **EVALUATE** - is feedback correct? applicable? worth implementing?
5. **RESPOND** - substantive reply (agree/disagree + reasoning)
6. **IMPLEMENT** - apply accepted changes

### Forbidden Responses

- "You're absolutely right!"
- "Great catch!" / "Excellent feedback!"
- "Thanks for catching that!"
- Any performative agreement without verification
- Any gratitude expression (actions > words)

### Required Instead

- "Verified: [evidence]. Implementing."
- "Checked [X]. Disagree because [reason]."
- "Ran test. Confirms issue. Fix: [approach]."
- Or just fix it silently - code shows you heard

### Handling Unclear Feedback

```
IF any item is unclear:
  STOP - do not implement anything yet
  ASK for clarification on ALL unclear items
```

Items may be related. Partial understanding = wrong implementation.

### When to Push Back

Push back when:
- Suggestion breaks existing functionality
- Reviewer lacks full context
- Violates YAGNI (unused feature)
- Technically incorrect for this stack
- Conflicts with prior architectural decisions

**How to push back:**
- Technical reasoning, not defensiveness
- Reference working tests/code
- Ask specific questions

### If You Were Wrong

When you pushed back incorrectly:
```
✅ "You were right - checked [X] and it does [Y]. Implementing."
❌ Long apology or defending why you pushed back
```

State correction factually and move on.

## Verification Iron Law

**Rule:** No completion claims without fresh verification evidence.

**Before claiming "done":**
- Run tests → show output
- Build succeeds → show output
- Behavior correct → demonstrate
- Edge cases handled → prove

**Anti-patterns:**
- "Should work now" (no evidence)
- "Fixed the issue" (no test run)
- "Implemented your suggestion" (no verification)

**Pattern:**
```
Implemented X.
Verification:
- `npm test`: 47 passed, 0 failed
- Manual test: [specific result]
- Edge case Y: [handled because Z]
```

## See Also

- **review-and-fix** - review changes + fix loop with subagent pattern
- **subagent-driven-development** - implement plans with review checkpoints

## Subagent Review Prompt

```
Review this code change:

FILES:
{diff or file contents}

CONTEXT:
{what was changed and why}

CONCERNS:
{specific areas to examine}

Provide:
1. Issues found (with severity: critical/major/minor)
2. Suggested improvements
3. Questions about intent
```
