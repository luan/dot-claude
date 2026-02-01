---
name: parallel-agents
description: Dispatch parallel subagents for 3+ independent failures or tasks with no shared state
---

# Parallel Agents

Dispatch one agent per independent problem domain. Let them work concurrently.

## When to Use

- 3+ failures in different test files/subsystems
- Each problem understood without context from others
- No shared state between investigations

## When NOT to Use

- Related failures (fix one → might fix others)
- Need full system context
- Agents would interfere (same files, shared resources)
- Exploratory debugging (don't know what's broken yet)

## Process

1. **Group by domain:** Identify independent problem areas
2. **Dispatch focused agents:** One Task per domain
3. **Review + integrate:** Read summaries, verify no conflicts, run full suite

## Agent Prompt Structure

Good prompts are focused, self-contained, specific about output:

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

- Too broad ("Fix all tests") → agent gets lost
- No context ("Fix race condition") → where?
- No constraints → agent refactors everything
- Vague output ("Fix it") → don't know what changed

## Verification

After agents return:
1. Review each summary
2. Check for conflicts (same code edited?)
3. Run full test suite
4. Spot check (agents can make systematic errors)
