# Cache Cost Model

## Pricing

| | Input | Output | Cache Write | Cache Read |
|---|---|---|---|---|
| Opus | $15/MTok | $75/MTok | $3.75/MTok | $0.30/MTok |
| Sonnet | $3/MTok | $15/MTok | $0.75/MTok | $0.06/MTok |
| Haiku | $0.80/MTok | $4/MTok | $0.08/MTok | $0.008/MTok |

## Cache TTL

TTL is 5 minutes.
After expiry, the next request forces a full cache-write across all active contexts.

## Resume vs fresh

Resume when idle < 5 min AND < 3 active subagent contexts.
Otherwise start fresh.

Example: 5 subagents at 100K tokens each = 500K tokens * $3.75/MTok = $1.88 in cache writes on Opus.

## Subagent discipline

Each subagent adds linearly to resume cost.
Prefer fewer, focused subagents.
Shut down subagents as soon as their work is done.
