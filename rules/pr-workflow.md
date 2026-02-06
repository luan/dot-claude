# PR & Shared Resource Workflow

## Graphite First
- Always use Graphite (`gt submit`) for PRs. Never `gh pr create`.
- Use `Skill tool: graphite` for all PR operations.

## Never Destructively Fix Visible Things
- Don't close/delete PRs, issues, or comments to fix mistakes — update in place.
- Graphite adopts existing PRs with `gt submit --force`.
- Closing PRs leaves visible trail of garbage for team.

## Always Draft
- NEVER mark a PR as "ready for review" unless the user explicitly asks.
- Leave PRs in draft state by default.

## General Principle
- Correcting mistakes in shared/visible systems (PRs, issues, messages): prefer additive fixes over destructive ones.
