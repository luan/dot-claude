# Rolling Scheduler

Dispatch tasks as soon as their dependencies are met, not in batch waves. Up to 4 workers run concurrently at any time.

## Pseudocode

```
# descendants(epicId): TaskList() filtered by metadata.parent_id == epicId → one level;
#   repeat for each child until a level returns empty; collect all nodes.
# leaf(task): task has no children — TaskList() filtered by metadata.parent_id == task.id is empty.

# Initial dispatch
ready = [t for t in descendants(epicId) if t.status == "pending" AND t.blockedBy is empty AND leaf(t)]

Spawn ready tasks (up to 4) using Team-based Worker Prompt (swarm) / Standalone Worker Prompt (no team)

active_count = len(spawned)
dispatch_count = {}       # task_id → number of dispatch attempts
dispatched = set(t.id for t in spawned)  # all task_ids ever dispatched (avoid re-dispatch)

# Rolling loop
while tasks remain incomplete:
  Wait for ANY worker to complete (Task returns or SendMessage received).

  On each completion:
    0. If worker output contains "RESCOPE:" → halt all dispatch, break loop immediately.
       Do NOT spawn newly_ready tasks. Return RESCOPE signal to orchestrator.
    1. If worker completed its task → active_count--
       If worker returned without completing → check TaskList():
         task still in_progress → stuck, TaskUpdate(id, status: "pending", owner: ""), active_count--
       TaskUpdate(epicId, metadata: {
         impl_completed: <count of completed children>,
         impl_active: active_count,
         impl_pending: <count of pending children>
       })
    2. Shut down completed team-based workers (SendMessage shutdown_request)
    3. # Re-scan: worker may have created child tasks (decomposition); former leaf may now be a grouping node
       fresh_descendants = descendants(epicId)  # re-query full subtree
       newly_ready = [t for t in fresh_descendants if t.status == "pending" AND t.blockedBy is empty AND leaf(t) AND t.id not in dispatched]
    4. Skip any task where dispatch_count[task_id] >= 2 (mark as failed, report to user)
    5. slots = 4 - active_count
    6. Spawn min(len(newly_ready), slots) tasks → active_count += spawned
       dispatched.update(id for each spawned task)
       dispatch_count[id]++ for each spawned task
    7. If active_count == 0 and no pending tasks remain → break

  Report progress: "N completed, M active, K pending, F failed"
```
