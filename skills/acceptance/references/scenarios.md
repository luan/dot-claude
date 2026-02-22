# Orphaned Task Detection — Test Scenarios

| Scenario | Setup | Expected Warning | Behavior |
|----------|-------|------------------|----------|
| A: One orphan | Parent Task 10 `status=completed`; child Task 11 `status=pending`, `parent_id=10` | `⚠ Orphaned tasks detected (parent completed, child still pending):`<br>`- Task 11: <subject> (parent: Task 10)` | Warning prepended; acceptance continues normally |
| B: No orphan | Parent Task 10 `status=completed`; child Task 11 `status=completed`, `parent_id=10` | _(no warning)_ | Traversal continues silently |
| C: Nested orphan | Grandparent Task 10 `status=completed`; parent Task 11 `status=completed`; grandchild Task 12 `status=pending`, `parent_id=11` | `⚠ Orphaned tasks detected (parent completed, child still pending):`<br>`- Task 12: <subject> (parent: Task 11)` | Detected at grandchild level during recursive traversal; acceptance continues |

Scenario C confirms detection fires at any depth, not just direct children of the epic. The warning never blocks acceptance — it is informational only.
