# PR & Shared Resource Workflow

## Graphite First
- **Always use Graphite** (`gt submit`) for PRs. Never `gh pr create`.
- Use `Skill tool: graphite` for all PR operations.

## Never Destructively Fix Visible Things
- Don't close/delete PRs, issues, or comments to "fix" mistakes — update in place.
- Graphite can adopt existing PRs with `gt submit --force`.
- Closing a PR leaves a visible trail of garbage for the whole team.

## General Principle
- When correcting mistakes in shared/visible systems (PRs, issues, messages): prefer additive fixes over destructive ones.
