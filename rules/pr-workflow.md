# PR & Shared Resources

## Graphite First
- `gt submit` for PRs. Never `gh pr create`.
- `Skill tool: graphite` for PR ops.
- "Review this branch" = diff vs stack parent (`gt log`), not trunk.
- Always return the Graphite URL (`app.graphite.com/...`), not GitHub URL.

## PR Titles
- Always use `Skill tool: commit` to generate PR titles. Never write them manually.

## Never Destructively Fix Visible Things
- Don't close/delete PRs, issues, comments — update in place.
- `gt submit --force` adopts existing PRs.

## Always Draft
- Never mark "ready for review" unless user requests.

## General
- Shared/visible systems: additive fixes > destructive.
