# Completed Scope With Tasks Fixture

Scope ran to completion. Epic and child tasks exist.

## Scope Tracking Task
```json
{
  "id": "500",
  "subject": "Scope: add webhook retry with exponential backoff",
  "status": "completed",
  "metadata": {
    "project": "/fake/project",
    "type": "scope",
    "priority": "P2",
    "design": "## Recommendation\nRetry with exponential backoff via BullMQ delayed jobs.\n\n## Next Steps\nPhase 1: Retry infrastructure\nPhase 2: Integration",
    "status_detail": "completed"
  }
}
```

## Epic
```json
{
  "id": "600",
  "subject": "Webhook retry with exponential backoff",
  "status": "in_progress",
  "metadata": {
    "project": "/fake/project",
    "type": "epic",
    "priority": "P1",
    "slug": "webhook-retry",
    "design": "<same as scope task design>"
  }
}
```

## Child Tasks
TaskList() with `metadata.parent_id == "600"` returns:
```json
[
  {
    "id": "601",
    "subject": "Phase 1: Retry infrastructure",
    "status": "pending",
    "metadata": { "parent_id": "600", "depth": 1, "design": "..." }
  },
  {
    "id": "602",
    "subject": "Phase 2: Integration",
    "status": "pending",
    "metadata": { "parent_id": "600", "depth": 1, "design": "...", "blockedBy": ["601"] }
  }
]
```
