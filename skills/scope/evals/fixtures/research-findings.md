# Research Findings Fixture

Raw output from research subagent. Input to spec/plan synthesis — not yet split into separate artifacts.

## Research Output
```
## Current State
- src/webhooks/sender.ts: fire-and-forget HTTP POST, no retry logic
- src/webhooks/types.ts: WebhookPayload interface, DeliveryStatus enum
- src/lib/queue.ts: BullMQ queue wrapper, supports delayed jobs and retries

## Key Files
- src/webhooks/sender.ts: main send function, currently synchronous fire-and-forget
- src/webhooks/types.ts: type definitions for webhook payloads
- src/lib/queue.ts: existing BullMQ queue infrastructure
- src/webhooks/retry.ts: does not exist yet

## Architectural Patterns
- Services use dependency injection via constructor
- Queue jobs follow { type, payload, metadata } shape
- Error handling uses Result<T, E> pattern throughout

## Analysis
The codebase already has BullMQ infrastructure. Adding retry to webhooks should leverage the existing queue wrapper rather than building a new retry mechanism. The sender.ts fire-and-forget pattern needs to change to queue-based delivery.

Key considerations:
- Exponential backoff: base 1s, factor 2x, max 5 attempts, jitter +/-20%
- Dead letter queue for permanently failed deliveries
- Idempotency keys to prevent duplicate delivery
- Queue backpressure monitoring under high failure rates

## Recommended Approach
Use BullMQ delayed jobs for retry with exponential backoff. Wrap delivery in a queue job that auto-retries on failure.

## Risks
- Queue backpressure under high failure rates
- Duplicate delivery if job completes but ack fails
- No existing monitoring for retry queue depth

## Suggested Phases
Phase 1: Retry infrastructure — create src/webhooks/retry.ts with backoff calculator + tests
Phase 2: Integration — modify src/webhooks/sender.ts to use queue-based delivery with retry + tests
```
