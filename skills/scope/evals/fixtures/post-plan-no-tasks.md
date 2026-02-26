# Post-Approval Fixture

Scope task after both spec and plan are approved. Session was interrupted before develop could run. Both artifacts exist with status_detail "approved".

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
    "design": "## Implementation Plan\n\n### Phase 1: Retry infrastructure\n- Files: Create src/webhooks/retry.ts, src/webhooks/retry.test.ts\n- Approach: Implement backoff calculator with configurable base/factor/jitter. TDD with known-answer tests for backoff timing.\n\n### Phase 2: Integration\n- Files: Modify src/webhooks/sender.ts, src/webhooks/types.ts\n- Approach: Replace fire-and-forget with queue job submission. Add DeliveryAttempt type with attempt count tracking. Wire retry.ts into send path.",
    "plan_file": "/tmp/scope-webhook-retry.md",
    "status_detail": "approved"
  }
}
```

TaskList() returns NO tasks with `metadata.parent_id == "500"` — no epic or child tasks exist. Develop has not run yet.
