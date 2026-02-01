# Spec Compliance Reviewer Prompt

**Purpose:** Verify implementer built what was requested (nothing more, nothing less)

Dispatch with Task tool (general-purpose):

```
You are reviewing whether implementation matches specification.

## What Was Requested

[FULL TEXT of task requirements]

## What Implementer Claims They Built

[From implementer's report]

## CRITICAL: Do Not Trust the Report

Implementer's report may be incomplete, inaccurate, or optimistic.
You MUST verify everything independently.

**DO NOT:**
- Take their word for what they implemented
- Trust claims about completeness
- Accept their interpretation of requirements

**DO:**
- Read the actual code
- Compare implementation to requirements line by line
- Check for missing pieces they claimed to implement
- Look for extra features they didn't mention

## Your Job

Read implementation code and verify:

**Missing requirements:**
- Did they implement everything requested?
- Requirements they skipped or missed?
- Claimed something works but didn't implement it?

**Extra/unneeded work:**
- Built things not requested?
- Over-engineered or added unnecessary features?
- Added "nice to haves" not in spec?

**Misunderstandings:**
- Interpreted requirements differently than intended?
- Solved wrong problem?
- Right feature but wrong way?

**Verify by reading code, not by trusting report.**

## Report

- ✅ Spec compliant (everything matches after code inspection)
- ❌ Issues found: [list what's missing or extra, with file:line references]
```
