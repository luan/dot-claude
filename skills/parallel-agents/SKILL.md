---
name: parallel-agents
description: Dispatch parallel subagents for 3+ independent failures or tasks with no shared state
---

# Parallel Agents

One agent per independent problem domain, working concurrently.

## When to Use

- 3+ failures in different files/subsystems
- Each problem independent (no shared context)
- No shared state between investigations

## When NOT to Use

- Related failures (fix one → might fix others)
- Need full system context
- Agents would interfere (same files/resources)
- Exploratory debugging (unknown root cause)

## Process

1. **Group by domain:** identify independent areas
2. **Dispatch:** one Task per domain
3. **Integrate:** read summaries, check conflicts, run full suite

## Agent Prompt Structure

Focused, self-contained, specific output:

```
Fix 3 failing tests in src/foo/bar.test.ts:

1. "test name A" - expected X got Y
2. "test name B" - timeout waiting for Z
3. "test name C" - wrong count

Root cause likely: timing/race conditions

Task:
1. Read test file, understand what each verifies
2. Identify root cause - timing issues or bugs?
3. Fix with event-based waiting, not arbitrary timeouts

Do NOT just increase timeouts.

Return: Summary of findings + fixes.
```

## Prompt Anti-patterns

- Too broad ("Fix all tests") → lost
- No context ("Fix race condition") → where?
- No constraints → agent refactors everything
- Vague output ("Fix it") → unknown changes

## Verification

1. Review each summary
2. Check conflicts (same code edited?)
3. Run full test suite
4. Spot check (agents make systematic errors)
