# Approved Plan Fixture

Scope task after step 4 (findings stored, plan mode about to start). Represents the state just before EnterPlanMode.

## Scope Tracking Task
```json
{
  "id": "500",
  "subject": "Scope: add webhook retry with exponential backoff",
  "status": "in_progress",
  "description": "Scope: add webhook retry with exponential backoff",
  "metadata": {
    "project": "/fake/project",
    "type": "scope",
    "priority": "P2",
    "design": "## Current State\n- src/webhooks/sender.ts: fire-and-forget HTTP POST, no retry\n- src/webhooks/types.ts: WebhookPayload interface\n- src/lib/queue.ts: BullMQ queue wrapper, supports delayed jobs\n\n## Recommendation\nAdd retry with exponential backoff using BullMQ delayed jobs. Max 5 attempts, base delay 1s, factor 2x, jitter ±20%.\n\n## Key Files\n- Modify: src/webhooks/sender.ts (add retry logic)\n- Modify: src/webhooks/types.ts (add attempt tracking)\n- Create: src/webhooks/retry.ts (backoff calculator)\n- Create: src/webhooks/retry.test.ts\n\n## Risks\n- Queue backpressure under high failure rates\n- Duplicate delivery if job completes but ack fails\n\n## Next Steps\nPhase 1: Retry infrastructure — src/webhooks/retry.ts, backoff calculator + tests\nPhase 2: Integration — src/webhooks/sender.ts, wire retry into send path + tests",
    "status_detail": "review"
  }
}
```
