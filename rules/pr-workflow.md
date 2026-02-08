# PR & Shared Resources

## Graphite First
- Always `gt submit` for PRs. Never `gh pr create`.
- `Skill tool: graphite` for all PR ops.
- "Review this branch" = diff against stack parent (`gt log`), not trunk.

## Never Destructively Fix Visible Things
- Don't close/delete PRs, issues, comments — update in place.
- `gt submit --force` adopts existing PRs.
- Closing PRs leaves visible garbage.

## Always Draft
- Never mark PR "ready for review" unless user explicitly asks.

## General
- Shared/visible systems: prefer additive fixes over destructive.
