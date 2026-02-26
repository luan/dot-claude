# Spec Review Fixture

Scope task after spec is synthesized and stored, awaiting user approval. Plan has NOT been generated yet.

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
    "spec": "## Problem\nWebhooks are fire-and-forget with no retry. Failed deliveries are permanently lost.\n\n## Recommendation\nAdd retry with exponential backoff using existing BullMQ infrastructure. Max 5 attempts, base delay 1s, factor 2x, jitter +/-20%. Dead letter queue for permanent failures. Idempotency keys to prevent duplicates.\n\n## Key Files\n- Modify: src/webhooks/sender.ts (queue-based delivery)\n- Modify: src/webhooks/types.ts (delivery attempt tracking)\n- Create: src/webhooks/retry.ts (backoff calculator)\n- Leverage: src/lib/queue.ts (existing BullMQ wrapper)\n\n## Risks\n- Queue backpressure under high failure rates\n- Duplicate delivery if job completes but ack fails\n- No existing monitoring for retry queue depth",
    "status_detail": "spec_review"
  }
}
```
