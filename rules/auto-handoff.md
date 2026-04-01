# Auto-Handoff

When context usage exceeds 80%, proactively initiate handoff before starting new work.

## How to check

Read `/tmp/claude-context-pct-${CLAUDE_SESSION_ID}` as a plain integer (0-100).
If the file does not exist, skip silently.

## When to check

- Before starting a new major task
- After completing a task
- Before dispatching subagents
- When entering a new pipeline stage

## Action

Warn the user: "Context is at N%. Initiating handoff."
Then invoke `/handoff`.
Do not silently proceed.

## Do not trigger when

- Context is below 80%
- You are mid-step in a multi-step task — finish the current step first
- The user has explicitly said to continue
- An in-progress task has `metadata.vibe2_stage` or `metadata.vibe_stage` set (active pipeline — let it finish)

## Idle sessions

If the session has been idle for more than 5 minutes, the cache TTL has expired.
Starting a fresh session is cheaper than resuming a stale one (see `cache-cost-model.md` for pricing).
Suggest `/handoff` at the start of any resumed idle session regardless of context percentage.
